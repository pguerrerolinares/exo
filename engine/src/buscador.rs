use crate::abre_db;
use anyhow::{Context, Result};
use rusqlite::params;
use serde::Serialize;
use std::path::Path;
use std::time::Instant;

/// Un resultado de búsqueda, forma EXACTA del contrato §4.1 de
/// `2026-07-17-indexer-design.md` (sellado, gateado en 52e3080).
#[derive(Debug, Serialize, PartialEq)]
pub struct Resultado {
    pub permalink: String,
    /// Siempre `"entity"` en v1 — resultados a nivel entidad, jamás filas
    /// observation (gotcha M0, spec M2 §4).
    #[serde(rename = "type")]
    pub tipo: String,
    /// Escala informativa, no contractual (spec §4.1 literal).
    pub score: f64,
}

/// `data` del envelope de `exo search`, forma EXACTA del contrato §4.1.
#[derive(Debug, Serialize, PartialEq)]
pub struct Busqueda {
    pub query: String,
    pub search_type: String,
    pub elapsed_s: f64,
    pub results: Vec<Resultado>,
}

/// Prepara la query cruda para FTS5 (interpretación adjudicada en el brief
/// m2-05, provisional — la calibración de retrieval es de M2-07/M2-09):
/// divide por whitespace, envuelve cada token en comillas dobles (escapando
/// `"` internas duplicándolas), une con espacio (AND implícito de FTS5).
/// Dentro de `"..."` no queda sintaxis FTS5 activa, así que tokens con
/// guiones (`agent-develop`), acentos o `/` nunca revientan la sintaxis de
/// MATCH — la fuente de verdad de esta regla es que ninguna de las 56
/// queries de `eval.jsonl` produzca error (oráculo m2-05 paso 2).
fn prepara_query(cruda: &str) -> String {
    cruda
        .split_whitespace()
        .map(|tok| format!("\"{}\"", tok.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Búsqueda FTS5 mínima sobre `notas_fts` (spec §4/§4.1). `score` usa
/// `-bm25(notas_fts)` (bm25 nativo de SQLite es "menor = mejor"; se niega
/// para que "mayor = mejor", consistente con `results` ordenados por score
/// descendente — spec §4.1: "su escala es informativa, no contractual").
/// Query sin hits = éxito con `results: []` (no es un error). DB inexistente
/// = error claro, JAMÁS se crea un fichero vacío como side-effect (a
/// diferencia de `rusqlite::Connection::open`, que crea el fichero si falta).
pub fn busca(db_ruta: &Path, query: &str, limite: usize) -> Result<Busqueda> {
    if !db_ruta.exists() {
        anyhow::bail!("DB no encontrada: {}", db_ruta.display());
    }

    let inicio = Instant::now();
    let conn = abre_db(db_ruta)?;
    let fts_query = prepara_query(query);

    // Query vacía tras normalizar (p.ej. solo whitespace): éxito con
    // resultados vacíos, sin invocar MATCH con cadena vacía (que revienta
    // FTS5 con "syntax error").
    let results = if fts_query.is_empty() {
        Vec::new()
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT permalink, -bm25(notas_fts) AS score
                 FROM notas_fts
                 WHERE notas_fts MATCH ?1
                 ORDER BY score DESC
                 LIMIT ?2",
            )
            .context("preparar consulta FTS5")?;

        stmt.query_map(params![fts_query, limite as i64], |r| {
            Ok(Resultado {
                permalink: r.get(0)?,
                tipo: "entity".to_string(),
                score: r.get(1)?,
            })
        })
        .with_context(|| format!("ejecutar MATCH FTS5 para query preparada: {fts_query}"))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("leer resultados FTS5")?
    };

    Ok(Busqueda {
        query: query.to_string(),
        search_type: "fts".to_string(),
        elapsed_s: inicio.elapsed().as_secs_f64(),
        results,
    })
}
