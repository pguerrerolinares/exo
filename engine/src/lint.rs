//! Los checks de deriva de la KB, portados de `kbx doctor` en bare mode, más
//! `index_stale`, que kbx no tiene.
//!
//! De los seis de kbx sobreviven cinco tal cual y `orphan` cambia de vecino:
//! `schema_drift` muere aquí (A7 del plan de G4b) — existía porque kbx y exo
//! eran dos binarios contra un schema compartido, y con un solo binario deja
//! de tener objeto. `index_stale` es nuevo (divergencia 8 del pre-registro):
//! tapa el agujero por el que `orphan`, único check que lee el índice, sale en
//! silencio sobre un índice vacío o desfasado. El recuento sigue en siete
//! tipos a cada lado, pero no son los mismos siete.

use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;

// `doctor.go:31-37` deja escrito «do not reintroduce a second copy of it»
// sobre el predicado de scope: dos copias con tolerancias que derivan es el
// modo de fallo que esa nota documenta. Por eso `excluida` y `es_md` se
// importan de `walker` en vez de reimplementarse aquí.
use crate::frontmatter::{budget_max, orphan_ok, tier, valor};
use crate::presupuesto::{Clase, Presupuestos, TIERS};
use crate::walker::{es_md, excluida, lee_nota, walk_kb_excluyendo};

/// Un hallazgo de lint: qué check lo produjo (`tipo`), qué ruta relativa
/// afecta (`ruta`) y el detalle legible del porqué (`detalle`). Las claves
/// serializan en inglés (`type`/`path`/`detail`) porque el consumidor es el
/// envelope JSON, no un lector en castellano.
#[derive(Serialize, Debug, Clone)]
pub struct Hallazgo {
    #[serde(rename = "type")]
    pub tipo: String,
    #[serde(rename = "path")]
    pub ruta: String,
    #[serde(rename = "detail")]
    pub detalle: String,
}

impl Hallazgo {
    fn nuevo(tipo: &str, ruta: impl Into<String>, detalle: impl Into<String>) -> Self {
        Hallazgo {
            tipo: tipo.to_string(),
            ruta: ruta.into(),
            detalle: detalle.into(),
        }
    }
}

/// Un mismo basename de directorio colgando de dos padres distintos. Es la
/// forma en que una KB se bifurca sin que nadie lo decida.
///
/// Toma los directorios ya recorridos en vez de volver a andar el árbol: `lint`
/// hace un solo walk y reparte.
pub fn dirs_duplicados(dirs: &[(String, String)], excluidos: &[&str]) -> Result<Vec<Hallazgo>> {
    let mut grupos: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for (basename, rel) in dirs.iter().cloned() {
        if excluida(&rel, excluidos) {
            continue;
        }
        grupos.entry(basename).or_default().push(rel);
    }
    let mut hallazgos = Vec::new();
    for (basename, mut rutas) in grupos {
        if rutas.len() < 2 {
            continue;
        }
        rutas.sort();
        hallazgos.push(Hallazgo::nuevo("duplicate_dir", basename, rutas.join(", ")));
    }
    // Defensiva, no falsable: `grupos` es un BTreeMap<String, _> y ya itera
    // por basename en orden, así que `hallazgos` sale de la línea de arriba
    // sorted por `ruta` (= basename) sin este sort_by. Se deja explícito por
    // si el tipo de `grupos` cambia algún día y deja de garantizarlo.
    hallazgos.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    Ok(hallazgos)
}

/// Falta la clave `tier`, o su valor no es uno de los tres legales. La
/// distinción entre las dos importa: `NOTIER` es "nadie lo declaró" y
/// `tier ilegal` es "lo declaró mal", y se arreglan distinto. La presencia se
/// decide con `frontmatter::valor(..., "tier").is_some()`, no con que
/// `frontmatter::tier()` devuelva vacío: un `tier:` sin valor legal pero
/// PRESENTE no debe colarse como NOTIER.
pub fn frontmatter_malo(kb: &Path, rutas: &[String], excluidos: &[&str]) -> Result<Vec<Hallazgo>> {
    let mut hallazgos = Vec::new();
    for rel in rutas {
        if excluida(rel, excluidos) {
            continue;
        }
        let absoluta = kb.join(rel);
        let contenido = lee_nota(&absoluta)?;
        if valor(&contenido, "tier").is_none() {
            hallazgos.push(Hallazgo::nuevo("bad_frontmatter", rel.clone(), "NOTIER"));
            continue;
        }
        // A1: `tier()` solo quita whitespace ASCII, así que un NBSP interno
        // (`co<NBSP>re`) deja el valor ilegal en vez de normalizarlo a `core`.
        let t = tier(&contenido);
        if !TIERS.contains(&t.as_str()) {
            hallazgos.push(Hallazgo::nuevo(
                "bad_frontmatter",
                rel.clone(),
                format!("tier ilegal: {t}"),
            ));
        }
    }
    hallazgos.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    Ok(hallazgos)
}

/// Ficheros no-nota en la raíz de la KB. Profundidad 0 solamente, saltando
/// dotfiles y directorios.
///
/// **No consulta la lista de exclusión, y es correcto** (A6). Ojo con el
/// porqué, porque el intuitivo es falso: `excluida("docs", EXCLUIDOS)` devuelve
/// `true` —el primer segmento de `"docs"` es `"docs"`—, así que el predicado
/// SÍ casaría si se le aplicara. La razón real es de dominio: la lista excluye
/// **directorios fuera del scope de la KB**, y un fichero de profundidad 0 no
/// está dentro de ninguno de ellos. Aplicárselo sería un choque accidental de
/// nombres, y un fichero llamado `docs` en la raíz dejaría de reportarse.
pub fn ficheros_en_raiz(kb: &Path) -> Result<Vec<Hallazgo>> {
    let mut hallazgos = Vec::new();
    let entradas =
        std::fs::read_dir(kb).with_context(|| format!("leer raíz de la KB {}", kb.display()))?;
    for entrada in entradas {
        let entrada = entrada.with_context(|| format!("entrada de {}", kb.display()))?;
        let nombre = entrada.file_name().to_string_lossy().into_owned();
        let tipo = entrada
            .file_type()
            .with_context(|| format!("file_type de {nombre}"))?;
        if nombre.starts_with('.') || tipo.is_dir() || es_md(&nombre) {
            continue;
        }
        hallazgos.push(Hallazgo::nuevo(
            "root_file",
            nombre,
            "fichero no-nota en la raíz",
        ));
    }
    hallazgos.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    Ok(hallazgos)
}

/// Notas sin ninguna relación en el índice, en ninguna de las dos direcciones.
///
/// **`AND destino_permalink IS NOT NULL` es load-bearing, no limpieza.** Con
/// una sola arista sin resolver, `NOT IN` sobre un subquery que contiene NULL
/// evalúa a NULL para TODAS las filas y la query devuelve cero huérfanas: verde,
/// en silencio, con el check apagado. Medido en kbx: 0 sin la guarda, 7 con
/// ella, sobre 23 aristas sin resolver de 573.
///
/// **Sin filtro por `tipo`.** El viejo `note_type = 'note'` decía excluir
/// assets y en realidad escondía 57 de 138 notas markdown reales. Retirado en
/// M6-04 T3 como cambio de scope deliberado.
///
/// Único de los seis checks que lee `notas.ruta` de la DB en vez del walk del
/// disco: `indexer::ruta_relativa` la guarda con el separador NATIVO, así que
/// en Windows llega como `archive\x.md` y hay que normalizarla antes de pasarla
/// por `excluida` o de devolverla en un `Hallazgo` (el JSON del envelope no
/// debe llevar `\`).
pub fn huerfanas(
    conn: &rusqlite::Connection,
    kb: &Path,
    excluidos: &[&str],
) -> Result<(Vec<Hallazgo>, Vec<Hallazgo>)> {
    let mut stmt = conn
        .prepare(
            "SELECT ruta, permalink
             FROM notas
             WHERE permalink NOT IN (SELECT origen FROM aristas)
               AND permalink NOT IN (
                     SELECT destino_permalink FROM aristas WHERE destino_permalink IS NOT NULL
                   )
             ORDER BY ruta",
        )
        .context("preparar la query de huérfanas")?;
    let filas = stmt
        .query_map([], |f| Ok((f.get::<_, String>(0)?, f.get::<_, String>(1)?)))
        .context("consultar huérfanas")?;

    let (mut hallazgos, mut waived) = (Vec::new(), Vec::new());
    for fila in filas {
        let (ruta, permalink) = fila.context("leer fila de huérfana")?;
        let ruta = ruta.replace('\\', "/");
        if excluida(&ruta, excluidos) {
            continue;
        }
        // Fallo hacia rojo: una nota ilegible (deriva índice/disco) NO se
        // absuelve — el marcador que la absolvería es justo lo que no se puede
        // leer.
        let absuelta = std::fs::read(kb.join(&ruta))
            .map(|b| orphan_ok(&String::from_utf8_lossy(&b)))
            .unwrap_or(false);
        if absuelta {
            waived.push(Hallazgo::nuevo(
                "orphan",
                ruta,
                format!("{permalink} (waived: kbx_orphan_ok)"),
            ));
        } else {
            hallazgos.push(Hallazgo::nuevo("orphan", ruta, permalink));
        }
    }
    Ok((hallazgos, waived))
}

/// Notas que rebasan su presupuesto. **No reimplementa la clasificación**:
/// llama a `presupuesto::clasifica`, la misma que usa `exo budget`. En kbx eran
/// dos implementaciones (`budget.Run` y `doctor.budgetExceededFindings`)
/// vigiladas por una regresión dirigida; aquí no pueden divergir porque solo
/// hay una.
///
/// Las notas sin tier o con tier ilegal se saltan: ya disparan
/// `bad_frontmatter`. Una deriva, un tipo de hallazgo.
pub fn presupuesto_excedido(
    kb: &Path,
    rutas: &[String],
    presupuestos: Presupuestos,
    excluidos: &[&str],
) -> Result<(Vec<Hallazgo>, Vec<Hallazgo>)> {
    let (mut hallazgos, mut waived) = (Vec::new(), Vec::new());
    for rel in rutas {
        if excluida(rel, excluidos) {
            continue;
        }
        let absoluta = kb.join(rel);
        let contenido = lee_nota(&absoluta)?;
        let t = tier(&contenido);
        let Some(tier_presupuesto) = presupuestos.para_tier(&t) else {
            continue;
        };
        let tamano = std::fs::metadata(&absoluta)
            .with_context(|| format!("stat de {}", absoluta.display()))?
            .len() as i64;
        match crate::presupuesto::clasifica(tier_presupuesto, budget_max(&contenido), tamano) {
            Clase::Infractora { presupuesto } => hallazgos.push(Hallazgo::nuevo(
                "budget_exceeded",
                rel.clone(),
                format!("{tamano}B > {presupuesto}B ({t})"),
            )),
            Clase::Waived { presupuesto } => waived.push(Hallazgo::nuevo(
                "budget_exceeded",
                rel.clone(),
                format!("{tamano}B ≤ {presupuesto}B (waived: kbx_budget_max)"),
            )),
            // El aviso de aire es de `budget`, no de `lint`: lint gatea y el
            // aire no debe gatear.
            Clase::SinAire { .. } | Clase::Ok => {}
        }
    }
    // Defensiva, no falsable con los tests de integración actuales: `rutas`
    // llega de `walk_notas`/`walk_kb_excluyendo`, que ya la entrega ordenada
    // alfabéticamente (`ficheros.sort()` en `walker.rs`), así que `hallazgos`
    // y `waived` salen ya en orden de `ruta` sin este `sort_by`. Mismo caso
    // que el de `dirs_duplicados` más arriba: se deja explícito por si el
    // contrato de orden de `rutas` cambia algún día y deja de garantizarlo.
    hallazgos.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    waived.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    Ok((hallazgos, waived))
}

/// Una línea habla de presupuestos si menciona "presupuesto"/"budget". Solo
/// esas se escanean: acota el check a prosa que dice estar citando un
/// presupuesto, en vez de a cualquier número que caiga cerca de un tier.
static LINEA_DE_PRESUPUESTO: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"(?i)presupuesto|budget").unwrap());

/// Un tier seguido inmediatamente de una cifra: "core 8.500 B", "stable 12500".
/// El "." como separador de miles porque la KB está escrita en castellano.
/// Exigir que la cifra vaya pegada al tier es lo que evita que la frase
/// "el presupuesto y las 3 notas core" cuente como cita.
static TIER_Y_CIFRA: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"\b(core|stable|log)\b[:\s]+([0-9]+(?:\.[0-9]{3})*)\s*(?:B\b|bytes\b)?")
        .unwrap()
});

/// Notas `core` cuya prosa cita una cifra de presupuesto que ya no es la que el
/// binario aplica. El caso motivador: un `core-index.md` diciendo "≤3.900
/// bytes" cuando el tool aplicaba 8.500 — una cifra fantasma que no imponía
/// nadie.
///
/// Solo `core`: son las notas que se inyectan cada sesión, donde un número
/// obsoleto hace daño real. Sin waiver posible: si una nota necesita citar una
/// cifra histórica a propósito, eso va en una `stable`/`log`, que este check no
/// mira.
pub fn deriva_de_prosa(
    kb: &Path,
    rutas: &[String],
    presupuestos: Presupuestos,
    excluidos: &[&str],
) -> Result<Vec<Hallazgo>> {
    let mut hallazgos = Vec::new();
    for rel in rutas {
        if excluida(rel, excluidos) {
            continue;
        }
        let absoluta = kb.join(rel);
        let contenido = lee_nota(&absoluta)?;
        if tier(&contenido) != "core" {
            continue;
        }
        for linea in contenido.lines() {
            if !LINEA_DE_PRESUPUESTO.is_match(linea) {
                continue;
            }
            for c in TIER_Y_CIFRA.captures_iter(linea) {
                let tier_citado = &c[1];
                // Una cifra que no parsea se salta: inventar un hallazgo desde
                // una línea no parseada es cómo un gate empieza a gritar y
                // acaba ignorado.
                let Ok(citada) = c[2].replace('.', "").parse::<i64>() else {
                    continue;
                };
                let aplicada = presupuestos.para_tier(tier_citado).unwrap_or(0);
                if citada != aplicada {
                    hallazgos.push(Hallazgo::nuevo(
                        "budget_prose_drift",
                        rel.clone(),
                        format!("cita {tier_citado} {citada}B, el tool aplica {aplicada}B"),
                    ));
                }
            }
        }
    }
    // Defensiva, no falsable con los tests de integración actuales: mismo
    // razonamiento que en `presupuesto_excedido` — `rutas` ya llega alfabética
    // de `walk_notas`, así que el orden de `hallazgos` coincide con el de
    // `rutas` (y, dentro de un mismo fichero, con el de aparición de las
    // líneas) sin este `sort_by`.
    hallazgos.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    Ok(hallazgos)
}

/// Notas que están en disco y el índice no conoce.
///
/// El check `orphan` es el único de los seis que lee el índice, así que sobre un
/// índice vacío o desfasado **no encuentra nada y `lint` sale verde**: el
/// instrumento no puede ver y responde "limpio". kbx tiene el mismo agujero;
/// aquí no, porque `analiza` ya tiene el walk de disco delante y compararlos es
/// barato.
///
/// El índice **vacío** se reporta como UN hallazgo, no uno por nota: es una
/// condición del entorno con una sola acción (`exo index`), y N hallazgos
/// idénticos convierten un informe accionable en ruido.
///
/// Ese hallazgo agregado lleva `ruta` vacía a propósito, no por omisión: no
/// señala una nota concreta (ninguna de las de disco tiene más derecho que
/// otra a cargar con el aviso), señala la KB entera. Convención de este
/// módulo, no un valor por defecto sin decidir.
///
/// El resto de notas no indexadas SÍ reciben remedio real, y no siempre el
/// mismo. `rutas` llega del walk de `lint` (`walk_kb_excluyendo` → `es_md`,
/// A5, case-insensitive), pero el indexer real nunca ve el mundo así: su walk
/// (`walker::walk_kb`) compara `Some("md")` exacto — case-sensitive, deuda ya
/// declarada en `docs/backlog.md`, fuera de esta ola — y su parser
/// (`nota::parsea_nota`, `nota.rs:49-51`) descarta en silencio (bueno, con
/// `eprintln!`, ver `indexer.rs:183-190`) cualquier nota sin `permalink:` en
/// el frontmatter (§6.2 regla 1). Antes de este comentario, `indice_rancio`
/// no distinguía: cualquier ruta en disco y ausente de `notas` recibía el
/// mismo `"en disco y no en el índice — corre \`exo index\`"`, remedio FALSO
/// para esas dos clases — correr `exo index`, aunque sea dos veces, jamás las
/// mete en el índice, y `lint` se quedaba en rojo permanente gritando un
/// arreglo que no arregla nada. Es el mismo modo de fallo que tenía
/// `es_repo_git` antes de separar sus dos condiciones (`gitx.rs`) y
/// exactamente el que describe el comentario de `deriva_de_prosa` sobre la
/// línea no parseada, más arriba en este fichero: «un gate que grita y acaba
/// ignorado». La alternativa — filtrar esas dos clases del check en silencio
/// — se descartó a propósito: callar el síntoma no es distinto de que
/// `orphan` calle sobre un índice vacío, y este módulo existe justo para no
/// hacer eso (de ahí `index_stale`). Se optó por diagnosticar la causa real
/// por nota, un hallazgo por causa, cada uno con su remedio verdadero (o sin
/// remedio automático cuando no lo hay, como en el caso de la extensión).
pub fn indice_rancio(
    conn: &rusqlite::Connection,
    kb: &Path,
    rutas: &[String],
) -> Result<Vec<Hallazgo>> {
    let mut stmt = conn
        .prepare("SELECT ruta FROM notas")
        .context("leer rutas del índice")?;
    let indexadas: std::collections::BTreeSet<String> = stmt
        .query_map([], |f| f.get::<_, String>(0))
        .context("consultar rutas del índice")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("leer fila del índice")?
        .into_iter()
        // Mismo motivo que en `huerfanas`: `notas.ruta` lleva separador nativo.
        .map(|r| r.replace('\\', "/"))
        .collect();

    if rutas.is_empty() {
        return Ok(Vec::new());
    }
    if indexadas.is_empty() {
        return Ok(vec![Hallazgo::nuevo(
            "index_stale",
            "",
            format!(
                "{} nota(s) en disco y el índice vacío — corre `exo index`",
                rutas.len()
            ),
        )]);
    }

    let mut hallazgos: Vec<Hallazgo> = rutas
        .iter()
        .filter(|r| !indexadas.contains(*r))
        .map(|r| -> Result<Hallazgo> {
            // Causa (a): extensión que el walk del indexer no reconoce
            // (case-sensitive, `Some("md")` exacto) aunque `es_md` (A5, este
            // check) sí la vea como nota. Sin lectura de disco: la extensión
            // sola ya certifica que jamás entrará en `notas`.
            if Path::new(r.as_str()).extension().and_then(|e| e.to_str()) != Some("md") {
                return Ok(Hallazgo::nuevo(
                    "index_stale",
                    r.clone(),
                    "extensión distinta de `.md` en minúscula (el walk del indexer \
                     compara exacto, case-sensitive) — no entrará en el índice hasta \
                     que se renombre",
                ));
            }
            // Causa (b): extensión correcta pero sin `permalink:` indexable
            // — `parsea_nota` la salta a propósito (§6.2 regla 1), así que
            // ni un `exo index` nuevo la va a meter mientras no se le añada
            // la clave.
            let contenido = lee_nota(&kb.join(r))?;
            if valor(&contenido, "permalink").is_none() {
                return Ok(Hallazgo::nuevo(
                    "index_stale",
                    r.clone(),
                    "sin `permalink` en el frontmatter — el indexer la salta a \
                     propósito (nota.rs, §6.2 regla 1); añádele la clave para que el \
                     próximo paso por el indexador la recoja",
                ));
            }
            // Causa (c): extensión correcta, permalink presente — deriva
            // genuina, el único caso donde el remedio SÍ es correr `exo
            // index`. Texto sin tocar respecto a antes de este cambio.
            Ok(Hallazgo::nuevo(
                "index_stale",
                r.clone(),
                "en disco y no en el índice — corre `exo index`",
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    // Defensiva, no falsable con los tests actuales: mismo caso que en
    // `presupuesto_excedido` y `deriva_de_prosa` — `rutas` llega alfabética
    // desde `walk_kb_excluyendo`, así que `filter` ya preserva ese orden sin
    // este `sort_by`. Se deja explícito por si el contrato de orden de
    // `rutas` cambia algún día y deja de garantizarlo.
    hallazgos.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    Ok(hallazgos)
}

/// El informe completo de `lint`: `ok` es cierto solo si `hallazgos` está
/// vacío (los `waived` no gatean, por definición). Clave JSON `findings` en
/// vez de `hallazgos`: el consumidor es el envelope, en inglés, igual que el
/// resto de tipos serializables del módulo.
#[derive(Serialize)]
pub struct InformeLint {
    pub ok: bool,
    #[serde(rename = "findings")]
    pub hallazgos: Vec<Hallazgo>,
    pub waived: Vec<Hallazgo>,
}

/// El pipeline de los siete checks, en orden fijo:
/// `duplicate_dir → orphan → bad_frontmatter → root_file → budget_exceeded →
/// budget_prose_drift → index_stale`. `schema_drift` no está y no vuelve (A7);
/// `index_stale` sí es nuevo frente a kbx (divergencia 8 del pre-registro): es
/// el único de los siete que compara el índice contra el disco en vez de leer
/// solo uno de los dos.
pub fn analiza(
    conn: &rusqlite::Connection,
    kb: &Path,
    presupuestos: Presupuestos,
    excluidos: &[&str],
) -> Result<InformeLint> {
    // Un solo walk del árbol para los seis checks. `root_file` lee la raíz
    // aparte porque solo mira profundidad 0.
    let (dirs, rutas) = walk_kb_excluyendo(kb, excluidos)?;
    let (mut hallazgos, mut waived) = (Vec::new(), Vec::new());

    hallazgos.extend(dirs_duplicados(&dirs, excluidos)?);
    let (h, w) = huerfanas(conn, kb, excluidos)?;
    hallazgos.extend(h);
    waived.extend(w);
    hallazgos.extend(frontmatter_malo(kb, &rutas, excluidos)?);
    hallazgos.extend(ficheros_en_raiz(kb)?);
    let (h, w) = presupuesto_excedido(kb, &rutas, presupuestos, excluidos)?;
    hallazgos.extend(h);
    waived.extend(w);
    hallazgos.extend(deriva_de_prosa(kb, &rutas, presupuestos, excluidos)?);
    hallazgos.extend(indice_rancio(conn, kb, &rutas)?);

    Ok(InformeLint {
        ok: hallazgos.is_empty(),
        hallazgos,
        waived,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // El test de integración homónimo (`dirs_duplicados_agrupa_por_basename_...`
    // en tests/lint.rs) alimenta `dirs_duplicados` con el `dirs` que devuelve
    // `walk_kb_excluyendo`, que YA llega ordenado por (basename, rel) — así que
    // ese test no falsa el `rutas.sort()` de aquí abajo: sin él, el orden de
    // inserción coincidiría igual con el esperado. Mismo motivo que
    // `orden_por_exceso_desempata_por_ruta_con_entrada_desordenada` en
    // presupuesto.rs: construir el vec a mano, deliberadamente desordenado.
    #[test]
    fn dirs_duplicados_ordena_el_detalle_aunque_la_entrada_llegue_desordenada() {
        let dirs = vec![
            ("notas".to_string(), "z/notas".to_string()),
            ("notas".to_string(), "a/notas".to_string()),
        ];
        let h = dirs_duplicados(&dirs, &[]).unwrap();
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].detalle, "a/notas, z/notas");
    }
}
