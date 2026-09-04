//! Los seis checks de deriva de la KB, portados de `kbx doctor` en bare mode.
//!
//! Son SEIS y no siete: `schema_drift` muere aquí (A7 del plan de G4b). Existía
//! porque kbx y exo eran dos binarios contra un schema compartido; con un solo
//! binario deja de tener objeto.

use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;

// `doctor.go:31-37` deja escrito «do not reintroduce a second copy of it»
// sobre el predicado de scope: dos copias con tolerancias que derivan es el
// modo de fallo que esa nota documenta. Por eso `excluida` y `es_md` se
// importan de `walker` en vez de reimplementarse aquí.
use crate::frontmatter::{orphan_ok, tier, valor};
use crate::presupuesto::TIERS;
use crate::walker::{es_md, excluida, lee_nota};

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
