use crate::abre_db;
use crate::con_embedder_de_proceso;
use anyhow::{Context, Result};
use rusqlite::params;
use serde::Serialize;
use std::collections::HashMap;
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

/// Conversión distancia→similitud (blindspot nota 1 del brief M2-06): la
/// DDL sellada de `vectores` (`schema.rs` §2, `CREATE VIRTUAL TABLE
/// vectores USING vec0(embedding float[768])`) NO declara
/// `distance_metric=cosine`, así que vec0 usa su métrica por defecto: L2 al
/// cuadrado (verificado contra el C vendorizado de sqlite-vec 0.1.9 —
/// `VEC0_DISTANCE_METRIC_L2` es el default en `vec0_column_config`, y el
/// motor de KNN usa `distance_l2_sqr_float`, NO la raíz cuadrada). fastembed
/// normaliza sus embeddings a norma unidad SIEMPRE (verificado:
/// `transformer_with_precedence` de fastembed 5.17.3 aplica
/// `common::normalize` sin condición al output de `TextEmbedding`). Para
/// dos vectores unitarios, `||a-b||² = 2 - 2·cos(a,b)`, luego
/// `cos(a,b) = 1 - ||a-b||²/2` — la conversión que usa esta función para
/// comparar contra `semantic_min_similarity` (threshold pensado en escala
/// coseno, config de producción de basic-memory, hoy 0.35).
fn similitud_desde_l2_cuadrado(distancia_l2_cuadrado: f64) -> f64 {
    1.0 - distancia_l2_cuadrado / 2.0
}

/// Precedencia flags > config (D6): `min_similitud` es el valor de
/// `--min-similitud` si se pasó; si no, cae a `semantic_min_similarity` de
/// `~/.basic-memory/config.json` (RO).
fn min_similitud_efectivo(min_similitud: Option<f64>) -> Result<f64> {
    match min_similitud {
        Some(v) => Ok(v),
        None => crate::min_similitud_de_config(),
    }
}

/// Búsqueda vectorial (`exo search --type vector`, M2-06): embed de la
/// query con el mismo modelo del indexer (jina-es/768, `Embedder` de
/// proceso), KNN EXHAUSTIVO sobre `vectores` (`k = COUNT(*)`: sqlite-vec
/// 0.1.9 sin partición ya hace un scan lineal internamente para vec0 float,
/// así que pedir menos vecinos no ahorra trabajo real y sí arriesga dejar
/// fuera la mejor coincidencia de una entidad — decisión declarada, no
/// aproximación silenciosa), conversión a similitud coseno, filtro por
/// `semantic_min_similarity` y agregación **chunk→entidad por máxima
/// similitud por permalink** (decisión declarada del Task 3 del brief: el
/// ground truth del eval es a nivel de nota — spec M2 §4 — así que "la nota
/// entra si su MEJOR trozo entra" es la agregación obvia; promediar o sumar
/// castigaría notas largas con más trozos sin motivo). `results` truncados
/// a `limite` tras ordenar por score descendente.
///
/// DB sin filas en `vectores` (aún no poblada, o corpus vacío tras
/// `rebuild` sin notas) → **0 resultados, no error** (Task 3, declarado):
/// mismo contrato que FTS ("sin hits = éxito con `results: []`") — un
/// `exo rebuild` recién corrido y una query vacía son casos operativos
/// normales, no fallos.
pub fn busca_vector(
    db_ruta: &Path,
    query: &str,
    limite: usize,
    min_similitud: Option<f64>,
) -> Result<Busqueda> {
    if !db_ruta.exists() {
        anyhow::bail!("DB no encontrada: {}", db_ruta.display());
    }

    let inicio = Instant::now();
    let conn = abre_db(db_ruta)?;

    let total_vectores: i64 = conn
        .query_row("SELECT count(*) FROM vectores", [], |r| r.get(0))
        .context("contar filas de vectores")?;

    let results = if total_vectores == 0 || query.trim().is_empty() {
        Vec::new()
    } else {
        let umbral = min_similitud_efectivo(min_similitud)?;

        let mut embeddings =
            con_embedder_de_proceso(|embedder| embedder.embebe_batch(&[query.to_string()]))
                .context("embed de la query")?;
        let embedding = embeddings.pop().expect("un embedding de la query");

        let vecinos = crate::vectores::knn(&conn, &embedding, total_vectores as usize)
            .context("KNN exhaustivo sobre vectores")?;

        let permalinks: HashMap<i64, String> = {
            let mut stmt = conn.prepare("SELECT id, permalink FROM trozos")?;
            stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
                .collect::<rusqlite::Result<_>>()
                .context("leer permalinks de trozos")?
        };

        let mut mejor_por_entidad: HashMap<String, f64> = HashMap::new();
        for vecino in vecinos {
            let sim = similitud_desde_l2_cuadrado(vecino.distancia);
            if sim < umbral {
                continue;
            }
            let Some(permalink) = permalinks.get(&vecino.rowid) else {
                continue; // trozo huérfano (no debería pasar; defensivo)
            };
            mejor_por_entidad
                .entry(permalink.clone())
                .and_modify(|actual| {
                    if sim > *actual {
                        *actual = sim;
                    }
                })
                .or_insert(sim);
        }

        let mut entidades: Vec<(String, f64)> = mejor_por_entidad.into_iter().collect();
        entidades.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        entidades.truncate(limite);

        entidades
            .into_iter()
            .map(|(permalink, score)| Resultado {
                permalink,
                tipo: "entity".to_string(),
                score,
            })
            .collect()
    };

    Ok(Busqueda {
        query: query.to_string(),
        search_type: "vector".to_string(),
        elapsed_s: inicio.elapsed().as_secs_f64(),
        results,
    })
}
