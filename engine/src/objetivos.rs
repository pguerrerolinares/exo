//! `exo targets` — candidatas de la KB para un tema, portado de
//! `kbx/internal/targets`.

use anyhow::{Result, bail};
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

    // Ningún operador FTS5 debe sobrevivir al quoting: ni prefijo, ni OR, ni
    // NEAR, ni filtro por columna. Todo es texto literal.
    #[test]
    fn ningun_operador_fts5_sobrevive_al_quoting() {
        for tema in [
            "metodolog*",
            "foo OR bar\"",
            "NEAR(alpha bitacora)",
            "title:alpha",
        ] {
            let q = construye_match_query(tema).unwrap();
            assert!(q.starts_with('"'), "{tema} -> {q}");
            assert!(q.ends_with('"'), "{tema} -> {q}");
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
