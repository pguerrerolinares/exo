//! `exo targets` — candidatas de la KB para un tema, portado de
//! `kbx/internal/targets`.

use crate::{frontmatter, gitx};
use anyhow::{Context, Result, bail};
use rusqlite::Connection;
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;
use std::sync::LazyLock;

/// Headings de nivel 1 a 3. El espacio tras las almohadillas es obligatorio,
/// que es lo que deja fuera a `####` sin necesidad de contarlas.
static PATRON_HEADING: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^#{1,3} (.+)$").unwrap());

/// Convierte un tema en una query FTS5 de literales citados.
///
/// Cada término separado por whitespace se envuelve en comillas dobles y las
/// comillas internas se duplican (el escape de string-literal de FTS5). El
/// efecto es que **ningún** operador sobrevive: ni `*` de prefijo, ni `OR`,
/// ni `NEAR(...)`, ni `col:term`. Todo lo que teclee el usuario es texto.
pub fn construye_match_query(tema: &str) -> Result<String> {
    let terminos: Vec<String> = tema
        .split_whitespace()
        .map(|t| format!("\"{}\"", t.replace('"', "\"\"")))
        .collect();
    if terminos.is_empty() {
        bail!("targets: el tema está vacío");
    }
    Ok(terminos.join(" "))
}

/// Headings de nivel 1-3 del fichero, en orden de aparición.
///
/// Best-effort a propósito: un fichero ilegible o inexistente devuelve lista
/// vacía, nunca error. La candidata sigue apareciendo aunque su fichero no se
/// pueda leer, porque el índice la conoce.
pub fn extrae_headings(ruta: &Path) -> Vec<String> {
    // `read` + `from_utf8_lossy` y no `read_to_string`: en Go esto es un
    // `bufio.Scanner` sobre bytes, que sigue produciendo headings en un
    // fichero con UTF-8 inválido. `read_to_string` fallaría y devolvería lista
    // vacía — una divergencia silenciosa con el binario contra el que se mide
    // la paridad.
    let Ok(bytes) = std::fs::read(ruta) else {
        return Vec::new();
    };
    let contenido = String::from_utf8_lossy(&bytes);
    let mut headings = Vec::new();
    let mut en_valla = false;
    for linea in contenido.lines() {
        if linea.trim_start().starts_with("```") {
            en_valla = !en_valla;
            continue;
        }
        if en_valla {
            continue;
        }
        if let Some(m) = PATRON_HEADING.captures(linea) {
            headings.push(m[1].to_string());
        }
    }
    headings
}

/// El SQL es el de `kbx/internal/targets` literal, con dos detalles que NO se
/// tocan:
///
/// - `snippet(notas_fts, 1, ...)`: el `1` es el índice ordinal de la columna
///   `cuerpo` dentro de `notas_fts(titulo, cuerpo, permalink UNINDEXED)`.
///   Insertar una columna antes de `cuerpo` haría que los snippets salieran de
///   la columna equivocada **en silencio** — ningún check de esquema mira el
///   orden ordinal, solo la presencia por nombre.
/// - **Sin `LIMIT`**: el truncado se aplica en Rust y DESPUÉS del dedup. Meter
///   `LIMIT ?` aquí es la optimización obvia y rompe la semántica el día que
///   `notas_fts` deje de ser 1:1 con `notas`: truncaría antes de deduplicar y
///   devolvería menos candidatas únicas de las que hay.
const CONSULTA_CANDIDATAS: &str = "SELECT notas.permalink,
       notas.ruta,
       COALESCE(snippet(notas_fts, 1, '', '', '…', 12), '') AS snip
FROM notas_fts
JOIN notas ON notas.permalink = notas_fts.permalink
WHERE notas_fts MATCH ?1
ORDER BY rank";

#[derive(Serialize)]
pub struct Objetivos {
    #[serde(rename = "topic")]
    pub tema: String,
    #[serde(rename = "candidates")]
    pub candidatos: Vec<Candidato>,
}

#[derive(Serialize)]
pub struct Candidato {
    pub permalink: String,
    pub tier: String,
    #[serde(rename = "size_bytes")]
    pub tamano_bytes: i64,
    pub headings: Vec<String>,
    #[serde(rename = "last_commit")]
    pub ultimo_commit: String,
    pub snippet: String,
}

/// Candidatas de la KB para `tema`, ordenadas por rank bm25.
///
/// La asimetría de fallo es deliberada y está cubierta por tests: leer el
/// fichero de disco (tier, tamaño, headings) es **best-effort** y degrada a
/// valores vacíos sin excluir a la candidata, porque el índice la conoce;
/// leer git es **fail-loud** y aborta el resultado entero. Unificar los dos
/// manejos rompe el contrato.
pub fn busca_objetivos(
    conn: &Connection,
    kb: &Path,
    tema: &str,
    limite: usize,
) -> Result<Objetivos> {
    if limite < 1 {
        bail!("targets: --limit tiene que ser >= 1, se recibió {limite}");
    }
    let match_query = construye_match_query(tema)?;

    let mut stmt = conn
        .prepare(CONSULTA_CANDIDATAS)
        .context("preparar la consulta de candidatas")?;
    let mut filas = stmt
        .query(rusqlite::params![match_query])
        .context("ejecutar la consulta de candidatas")?;

    let mut candidatos = Vec::new();
    let mut vistas: HashSet<String> = HashSet::new();

    while candidatos.len() < limite {
        let Some(fila) = filas.next().context("leer una candidata")? else {
            break;
        };
        let permalink: String = fila.get(0)?;
        let ruta_rel: String = fila.get(1)?;
        let snippet: String = fila.get(2)?;

        if !vistas.insert(ruta_rel.clone()) {
            continue;
        }

        let ultimo_commit = gitx::ultimo_commit(kb, &ruta_rel)?;

        // `read` y no `read_to_string`: `size_bytes` es el tamaño en BYTES y
        // el gate de paridad lo compara exacto. Un fichero con UTF-8 inválido
        // le da a Go su tamaño real y a `read_to_string` un Err — es decir, un
        // 0 silencioso justo en el campo que se está midiendo.
        let absoluta = kb.join(&ruta_rel);
        let (tier, tamano_bytes) = match std::fs::read(&absoluta) {
            Ok(bytes) => {
                let contenido = String::from_utf8_lossy(&bytes);
                (frontmatter::tier(&contenido), bytes.len() as i64)
            }
            Err(_) => (String::new(), 0),
        };

        candidatos.push(Candidato {
            permalink,
            tier,
            tamano_bytes,
            headings: extrae_headings(&absoluta),
            ultimo_commit,
            snippet,
        });
    }

    Ok(Objetivos {
        tema: tema.to_string(),
        candidatos,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cita_cada_termino_como_literal_fts5() {
        assert_eq!(
            construye_match_query("metodologia").unwrap(),
            "\"metodologia\""
        );
        assert_eq!(
            construye_match_query("estructura headings").unwrap(),
            "\"estructura\" \"headings\""
        );
    }

    #[test]
    fn duplica_las_comillas_internas() {
        assert_eq!(
            construye_match_query("say \"hi\"").unwrap(),
            "\"say\" \"\"\"hi\"\"\""
        );
    }

    #[test]
    fn un_tema_vacio_es_error() {
        assert!(construye_match_query("").is_err());
        assert!(construye_match_query("   ").is_err());
        assert!(construye_match_query("\t\n").is_err());
    }

    // Un operador de FTS5 en el tema sale citado como texto, término a
    // término. Los cuatro casos son las cuatro clases de operador: prefijo,
    // booleano con comilla suelta, proximidad y filtro por columna.
    //
    // Se compara la cadena EXACTA y no `starts_with('"')`: una forma "empieza
    // y acaba por comilla" la cumpliría también una implementación que no
    // escapase las comillas internas, que es precisamente el fallo que hay que
    // descartar aquí.
    //
    // Lo que este test NO prueba: que FTS5 trate esa cadena como literal. Eso
    // solo lo demuestra una query contra el motor de verdad, y **todavía no
    // existe** — lo añade la Task 4 del plan de G4a como
    // `tests/objetivos.rs::ningun_operador_fts5_se_ejecuta_como_operador`.
    // Hasta entonces, la neutralización está razonada, no medida.
    #[test]
    fn los_operadores_fts5_salen_citados_termino_a_termino() {
        let casos = [
            ("metodolog*", r#""metodolog*""#),
            ("foo OR bar\"", r#""foo" "OR" "bar""""#),
            ("NEAR(alpha bitacora)", r#""NEAR(alpha" "bitacora)""#),
            ("title:alpha", r#""title:alpha""#),
        ];
        for (tema, esperado) in casos {
            assert_eq!(
                construye_match_query(tema).unwrap(),
                esperado,
                "tema: {tema}"
            );
        }
    }

    fn escribe(dir: &std::path::Path, nombre: &str, contenido: &str) -> std::path::PathBuf {
        let p = dir.join(nombre);
        std::fs::write(&p, contenido).unwrap();
        p
    }

    #[test]
    fn extrae_headings_de_nivel_1_a_3_en_orden() {
        let dir = tempfile::tempdir().unwrap();
        let p = escribe(dir.path(), "a.md", "# uno\ntexto\n## dos\n### tres\n");
        assert_eq!(extrae_headings(&p), vec!["uno", "dos", "tres"]);
    }

    // Nivel 4+ no matchea porque tras tres `#` el patrón exige un espacio
    // literal, y un cuarto `#` no lo es. Y lo que hay dentro de una valla de
    // código no es un heading.
    #[test]
    fn ignora_el_nivel_4_y_lo_que_hay_dentro_de_una_valla() {
        let dir = tempfile::tempdir().unwrap();
        let p = escribe(
            dir.path(),
            "a.md",
            "# real\n#### profundo\n```sh\n# falso\n```\n## otro\n",
        );
        assert_eq!(extrae_headings(&p), vec!["real", "otro"]);
    }

    #[test]
    fn un_fichero_ilegible_da_lista_vacia_nunca_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(extrae_headings(&dir.path().join("no-existe.md")).is_empty());
    }

    #[test]
    fn los_headings_sobreviven_a_crlf() {
        let dir = tempfile::tempdir().unwrap();
        let p = escribe(dir.path(), "a.md", "# uno\r\n## dos\r\n");
        assert_eq!(extrae_headings(&p), vec!["uno", "dos"]);
    }
}
