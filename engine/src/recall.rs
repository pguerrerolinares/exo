//! `exo recall` (M2-08): sucesor de `basic-memory-recall.sh` (arranque) y
//! `compose-inject.sh` de reflex (subagentes). Sirve **contenido de la KB**
//! desde el índice propio de exo; no conoce reflex ni perfiles de agentes —
//! eso lo compone el consumidor, este módulo solo sirve.
//!
//! Dos modos (brief M2-08, contrato CLI fijado por el orquestador):
//! - **arranque** (sin `--query`): notas `tier: core` en orden de ruta
//!   estable + las `--limite` notas más recientes por `git_epoch`.
//! - **consulta** (con `--query`): `busca_hybrid` con los defaults sellados
//!   de M2-07.
//!
//! El cap de bytes (`--cap-bytes`, trunca por líneas ENTERAS) se aplica una
//! sola vez (`aplica_cap`/`renderiza`) sobre una representación línea-a-línea
//! común a ambos modos de salida (texto y `--json`): una nota entra en el
//! resultado si TODAS sus líneas (la principal, más la de snippet en modo
//! consulta) caben enteras en el presupuesto — nunca queda "a medias" en
//! ninguno de los dos formatos (decisión declarada, brief no lo fija
//! explícito).
//!
//! `recall_arranque` resuelve rutas absolutas inline (necesita `kb` para
//! releer `tier` del `.md`, así que ya lo tiene a mano). `recall_consulta`
//! solo lee el índice — no necesita `kb` para nada más — y deja las rutas
//! relativas; `resuelve_rutas_absolutas` las convierte después, para que un
//! error de `--kb`/`kb_desde_config()` en el CLI no se confunda con un
//! error de la query en sí.

use crate::abre_db;
use crate::nota::parsea_nota;
use anyhow::{Context, Result};
use rusqlite::{params, OptionalExtension};
use serde::Serialize;
use std::path::Path;

/// Una nota del recall, forma EXACTA del contrato §"Salida `--json`" del
/// brief M2-08. `score`/`snippet` son `None` en modo arranque (`null` en
/// JSON, spec literal).
#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct NotaRecall {
    pub permalink: String,
    pub ruta: String,
    pub titulo: String,
    pub tier: Option<String>,
    pub score: Option<f64>,
    pub snippet: Option<String>,
}

/// `data` del envelope de `exo recall` (contrato §"Salida `--json`").
#[derive(Debug, Serialize, PartialEq)]
pub struct Recall {
    pub modo: String,
    pub query: Option<String>,
    pub cap_bytes: usize,
    pub truncado: bool,
    pub notas: Vec<NotaRecall>,
}

/// Resultado crudo de un modo (arranque o consulta), ANTES de aplicar el
/// cap de bytes — `aplica_cap` es la única que decide qué entra en el
/// bloque final.
pub struct RecallBruto {
    pub modo: String,
    pub query: Option<String>,
    pub notas: Vec<NotaRecall>,
}

const CABECERA: &str = "=== Recall exo (PARCIAL — no sustituye tu brief) ===";

/// Recorta `texto` a lo sumo `max_bytes` bytes UTF-8, cortando en frontera
/// de carácter completo (jamás a mitad de un carácter multibyte — la KB
/// tiene acentos, contar/cortar por bytes crudos partiría uno). Usado para
/// el snippet de modo consulta ("recortada a ~200 bytes", brief).
fn recorta_bytes(texto: &str, max_bytes: usize) -> String {
    if texto.len() <= max_bytes {
        return texto.to_string();
    }
    let mut fin = 0;
    for (i, c) in texto.char_indices() {
        let siguiente = i + c.len_utf8();
        if siguiente > max_bytes {
            break;
        }
        fin = siguiente;
    }
    texto[..fin].to_string()
}

/// Techo del snippet individual (brief: "~200 bytes por línea entera").
const SNIPPET_MAX_BYTES: usize = 200;

/// Una unidad renderizable del bloque: la nota final (ya con snippet
/// recortado) más las líneas de texto que le corresponden — 1 en modo
/// arranque, 2 (nota + snippet) en modo consulta.
struct Unidad {
    nota: NotaRecall,
    lineas: Vec<String>,
}

/// Resultado de aplicar el cap: el bloque de texto plano (newline-terminado,
/// listo para stdout), el `Recall` final (para `--json`, mismo conjunto de
/// notas que el texto) y cuántas líneas se perdieron (para el aviso stderr).
pub struct ResultadoCap {
    pub texto: String,
    pub recall: Recall,
    pub lineas_perdidas: usize,
}

/// Aplica el cap de bytes (brief: "trunca por líneas enteras: imprime cada
/// línea completa mientras quepa en el presupuesto; la primera línea que no
/// cabe entera se descarta y ahí para. Cuenta bytes, no caracteres").
/// Cada línea cuenta su longitud en bytes MÁS 1 (el `\n` que la termina).
/// Una nota entra en `notas`/el texto solo si TODAS sus líneas cupieron
/// completas; en cuanto una línea no cabe, el proceso para (ni esa nota
/// parcial ni ninguna posterior aparecen).
fn aplica_cap(modo: &str, query: Option<String>, notas: Vec<NotaRecall>, cap_bytes: usize) -> ResultadoCap {
    let con_snippet = notas.iter().any(|n| n.snippet.is_some());

    let unidades: Vec<Unidad> = notas
        .into_iter()
        .map(|n| {
            let principal = format!("- {} — {}", n.ruta, n.titulo);
            let mut lineas = vec![principal];
            if let Some(s) = &n.snippet {
                lineas.push(format!("  · {s}"));
            }
            Unidad { nota: n, lineas }
        })
        .collect();
    let _ = con_snippet; // documental: confirma la forma de `lineas` arriba

    let mut texto = String::new();
    let mut bytes_usados = 0usize;
    let mut notas_finales = Vec::new();
    let mut truncado = false;
    let mut lineas_perdidas = 0usize;

    let cabecera_linea = format!("{CABECERA}\n");
    if bytes_usados + cabecera_linea.len() <= cap_bytes {
        texto.push_str(&cabecera_linea);
        bytes_usados += cabecera_linea.len();
    } else {
        truncado = true;
        lineas_perdidas += 1;
    }

    let mut cortado = truncado;
    for unidad in unidades {
        if cortado {
            lineas_perdidas += unidad.lineas.len();
            continue;
        }
        let bytes_unidad: usize = unidad.lineas.iter().map(|l| l.len() + 1).sum();
        if bytes_usados + bytes_unidad <= cap_bytes {
            for linea in &unidad.lineas {
                texto.push_str(linea);
                texto.push('\n');
            }
            bytes_usados += bytes_unidad;
            notas_finales.push(unidad.nota);
        } else {
            truncado = true;
            cortado = true;
            lineas_perdidas += unidad.lineas.len();
        }
    }

    ResultadoCap {
        texto,
        recall: Recall {
            modo: modo.to_string(),
            query,
            cap_bytes,
            truncado,
            notas: notas_finales,
        },
        lineas_perdidas,
    }
}

/// Punto de entrada único: renderiza un `RecallBruto` aplicando el cap.
pub fn renderiza(bruto: RecallBruto, cap_bytes: usize) -> ResultadoCap {
    aplica_cap(&bruto.modo, bruto.query, bruto.notas, cap_bytes)
}

/// Modo arranque (brief §Tarea 2): notas `tier: core` (frontmatter, releído
/// del `.md` en disco — NO está en el índice, ver `nota::Nota::tier`) en
/// orden de ruta estable, seguidas de las `limite` notas más recientes por
/// `git_epoch` (recencia = git, §6.2 regla 2; NUNCA mtime). Las notas ya
/// incluidas como core se excluyen del bloque de recientes (decisión
/// declarada: mostrar la misma nota dos veces en el bloque de arranque no
/// aporta nada al consumidor).
pub fn recall_arranque(db_ruta: &Path, kb: &Path, limite: usize) -> Result<RecallBruto> {
    if !db_ruta.exists() {
        anyhow::bail!("DB no encontrada: {}", db_ruta.display());
    }
    let conn = abre_db(db_ruta)?;

    struct Fila {
        permalink: String,
        ruta_rel: String,
        titulo: String,
        git_epoch: Option<i64>,
    }

    let mut filas: Vec<Fila> = {
        let mut stmt =
            conn.prepare("SELECT permalink, ruta, titulo, git_epoch FROM notas ORDER BY ruta")?;
        stmt.query_map([], |r| {
            Ok(Fila {
                permalink: r.get(0)?,
                ruta_rel: r.get(1)?,
                titulo: r.get(2)?,
                git_epoch: r.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()
        .context("leer notas para recall arranque")?
    };
    // `ORDER BY ruta` ya deja orden de ruta estable (SQLite ORDER BY sobre
    // TEXT es byte-order, coherente con `walker::walk_kb`), pero se
    // reafirma en Rust para no depender de la collation de SQLite.
    filas.sort_by(|a, b| a.ruta_rel.cmp(&b.ruta_rel));

    let mut cores: Vec<NotaRecall> = Vec::new();
    let mut permalinks_core = std::collections::HashSet::new();

    for fila in &filas {
        let ruta_abs = kb.join(&fila.ruta_rel);
        let tier = tier_de(&ruta_abs);
        if tier.as_deref() == Some("core") {
            permalinks_core.insert(fila.permalink.clone());
            cores.push(NotaRecall {
                permalink: fila.permalink.clone(),
                ruta: ruta_abs.display().to_string(),
                titulo: fila.titulo.clone(),
                tier,
                score: None,
                snippet: None,
            });
        }
    }

    let mut recientes: Vec<&Fila> = filas
        .iter()
        .filter(|f| !permalinks_core.contains(&f.permalink))
        .collect();
    recientes.sort_by(|a, b| match (a.git_epoch, b.git_epoch) {
        (Some(x), Some(y)) => y.cmp(&x),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.ruta_rel.cmp(&b.ruta_rel),
    });
    recientes.truncate(limite);

    let mut notas = cores;
    notas.extend(recientes.into_iter().map(|fila| {
        let ruta_abs = kb.join(&fila.ruta_rel);
        NotaRecall {
            permalink: fila.permalink.clone(),
            ruta: ruta_abs.display().to_string(),
            titulo: fila.titulo.clone(),
            tier: None,
            score: None,
            snippet: None,
        }
    }));

    Ok(RecallBruto {
        modo: "arranque".to_string(),
        query: None,
        notas,
    })
}

/// Lee SOLO el `tier` del frontmatter de `ruta` (releído del `.md` en
/// disco). `None` en cualquier fallo (fichero movido/borrado tras indexar,
/// frontmatter ilegible, sin `tier:`) — nunca aborta el recall completo por
/// una nota individual (mismo patrón de tolerancia que el indexer, spec §3).
fn tier_de(ruta: &Path) -> Option<String> {
    parsea_nota(ruta).ok().flatten().and_then(|n| n.tier)
}

/// Modo consulta (brief §Tarea 2): `busca_hybrid` con los defaults sellados
/// (`bonus`/`escala_fts` los pasa el llamador — M2-07, `BONUS_SELLADO`/
/// `ESCALA_FTS_SELLADA` viven en `main.rs`, no se duplican aquí) y
/// `--min-similitud` con el mismo default de config que `search`. El
/// snippet de cada nota es su PRIMER trozo (`orden = 0`): la fusión
/// hybrid no expone qué trozo individual disparó el match de una entidad
/// (agrega por máxima similitud, spec fusión) y recalcular esa similitud
/// por trozo aquí duplicaría trabajo para un dato puramente informativo
/// (decisión declarada, ver reporte).
pub fn recall_consulta(
    db_ruta: &Path,
    query: &str,
    limite: usize,
    min_similitud: Option<f64>,
    bonus: f64,
    escala_fts: f64,
) -> Result<RecallBruto> {
    if !db_ruta.exists() {
        anyhow::bail!("DB no encontrada: {}", db_ruta.display());
    }

    let resultado =
        crate::buscador::busca_hybrid(db_ruta, query, limite, min_similitud, bonus, escala_fts)?;

    let conn = abre_db(db_ruta)?;
    let mut notas = Vec::with_capacity(resultado.results.len());
    for r in resultado.results {
        let Some((ruta_rel, titulo)) = fila_notas(&conn, &r.permalink)? else {
            continue; // huérfano defensivo: entidad en `vectores`/FTS sin fila en `notas` (no debería pasar)
        };
        let snippet = primer_trozo(&conn, &r.permalink)?
            .map(|texto| recorta_bytes(&texto.replace('\n', " "), SNIPPET_MAX_BYTES));
        notas.push(NotaRecall {
            permalink: r.permalink,
            ruta: ruta_rel, // se resuelve a absoluta en `recall_consulta_con_kb`
            titulo,
            tier: None,
            score: Some(r.score),
            snippet,
        });
    }

    Ok(RecallBruto {
        modo: "consulta".to_string(),
        query: Some(query.to_string()),
        notas,
    })
}

fn fila_notas(conn: &rusqlite::Connection, permalink: &str) -> Result<Option<(String, String)>> {
    conn.query_row(
        "SELECT ruta, titulo FROM notas WHERE permalink = ?1",
        params![permalink],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
    )
    .optional()
    .with_context(|| format!("leer ruta/titulo de {permalink}"))
}

fn primer_trozo(conn: &rusqlite::Connection, permalink: &str) -> Result<Option<String>> {
    conn.query_row(
        "SELECT texto FROM trozos WHERE permalink = ?1 ORDER BY orden LIMIT 1",
        params![permalink],
        |r| r.get::<_, String>(0),
    )
    .optional()
    .with_context(|| format!("leer primer trozo de {permalink}"))
}

/// Resuelve las `ruta` (relativas a la KB, tal como salen de
/// `recall_consulta`) a rutas ABSOLUTAS bajo `kb` — mismo tratamiento que
/// `recall_arranque` ya hace inline. Separado porque `recall_consulta` no
/// necesita `kb` para nada más (solo lee el índice), y así el error de
/// resolver `--kb`/`kb_desde_config()` en el CLI no oculta errores de la
/// query en sí.
pub fn resuelve_rutas_absolutas(bruto: &mut RecallBruto, kb: &Path) {
    for nota in &mut bruto.notas {
        nota.ruta = kb.join(&nota.ruta).display().to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nota(permalink: &str, ruta: &str, titulo: &str, snippet: Option<&str>) -> NotaRecall {
        NotaRecall {
            permalink: permalink.to_string(),
            ruta: ruta.to_string(),
            titulo: titulo.to_string(),
            tier: None,
            score: snippet.map(|_| 0.5),
            snippet: snippet.map(|s| s.to_string()),
        }
    }

    #[test]
    fn recorta_bytes_no_corta_si_ya_cabe() {
        assert_eq!(recorta_bytes("café", 200), "café");
    }

    /// "café" repetido: cada 'é' ocupa 2 bytes en UTF-8. Un límite que cae
    /// justo a mitad de un carácter multibyte debe recortar ANTES de ese
    /// carácter, nunca partirlo (produciría bytes UTF-8 inválidos).
    #[test]
    fn recorta_bytes_no_parte_caracter_multibyte() {
        let texto = "é".repeat(10); // 20 bytes, 10 chars
        let recortado = recorta_bytes(&texto, 15); // 15 cae a mitad del carácter 8 (byte 15 de 16)
        assert!(recortado.len() <= 15, "len={}", recortado.len());
        assert!(
            std::str::from_utf8(recortado.as_bytes()).is_ok(),
            "debe ser UTF-8 válido"
        );
        assert_eq!(recortado, "é".repeat(7)); // 7*2=14 <= 15, el 8vo (byte 15-16) no cabe
    }

    #[test]
    fn aplica_cap_todo_cabe_no_trunca() {
        let notas = vec![nota("a", "/kb/a.md", "A", None), nota("b", "/kb/b.md", "B", None)];
        let r = aplica_cap("arranque", None, notas, 2048);
        assert!(!r.recall.truncado);
        assert_eq!(r.recall.notas.len(), 2);
        assert_eq!(r.lineas_perdidas, 0);
        assert!(r.texto.starts_with(CABECERA));
        assert!(r.texto.contains("/kb/a.md"));
        assert!(r.texto.contains("/kb/b.md"));
    }

    /// Cap justo para la cabecera + la primera nota, no la segunda: la
    /// segunda nota se descarta ENTERA (ninguna nota "a medias").
    #[test]
    fn aplica_cap_corta_por_lineas_enteras_y_para() {
        let notas = vec![nota("a", "/kb/a.md", "A", None), nota("b", "/kb/b.md", "B", None)];
        let linea_a = format!("- /kb/a.md — A\n");
        let cabecera = format!("{CABECERA}\n");
        let cap = cabecera.len() + linea_a.len(); // exacto para cabecera+a, no para b

        let r = aplica_cap("arranque", None, notas, cap);
        assert!(r.recall.truncado);
        assert_eq!(r.recall.notas.len(), 1);
        assert_eq!(r.recall.notas[0].permalink, "a");
        assert!(!r.texto.contains("/kb/b.md"));
        assert!(r.lineas_perdidas >= 1);
    }

    #[test]
    fn aplica_cap_extremo_ni_cabecera_cabe() {
        let notas = vec![nota("a", "/kb/a.md", "A", None)];
        let r = aplica_cap("arranque", None, notas, 1);
        assert!(r.recall.truncado);
        assert_eq!(r.recall.notas.len(), 0);
        assert_eq!(r.texto, "");
    }

    /// Modo consulta: nota con snippet cuya línea de snippet no cabe ⇒ la
    /// nota entera se descarta (no aparece "a medias" sin snippet).
    #[test]
    fn aplica_cap_nota_con_snippet_que_no_cabe_se_descarta_entera() {
        let notas = vec![nota("a", "/kb/a.md", "A", Some("un snippet bastante largo de verdad"))];
        // cap justo para la cabecera + la línea principal, no el snippet
        let cabecera = format!("{CABECERA}\n");
        let linea_a = "- /kb/a.md — A\n";
        let cap = cabecera.len() + linea_a.len();

        let r = aplica_cap("consulta", Some("q".to_string()), notas, cap);
        assert!(r.recall.truncado);
        assert_eq!(r.recall.notas.len(), 0, "{:?}", r.recall.notas);
    }

    #[test]
    fn recall_json_forma_del_contrato() {
        let notas = vec![nota("a", "/kb/a.md", "A", Some("snippet"))];
        let r = aplica_cap("consulta", Some("q".to_string()), notas, 2048);
        let valor = serde_json::to_value(&r.recall).unwrap();
        let obj = valor.as_object().unwrap();
        for clave in ["modo", "query", "cap_bytes", "truncado", "notas"] {
            assert!(obj.contains_key(clave), "falta {clave}: {obj:?}");
        }
        let primera = &obj["notas"][0];
        for clave in ["permalink", "ruta", "titulo", "tier", "score", "snippet"] {
            assert!(primera.get(clave).is_some(), "falta {clave} en nota: {primera:?}");
        }
    }
}
