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
        // Desempate determinista por permalink ascendente (M2-09a): sin él,
        // `sort_by` (estable) preserva el orden de iteración del `HashMap`
        // de origen, que no es reproducible entre corridas.
        entidades.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
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

/// Normalización BM25 por-query con anclaje β (spec fusión §4.3, D-f1):
/// `f(e) = β · f_raw(e) / f_max(q)`, `f_max(q) = max f_raw` sobre los
/// candidatos FTS de la query. Monótona (preserva el orden FTS) y acotada a
/// `(0, β]` — el top-1 de la query vale exactamente β. Degenerados: lista
/// vacía o `f_max == 0` (bm25 devolviendo 0, teóricamente posible) → canal
/// FTS descartado entero para la query (mapa vacío, sin dividir por 0), NO
/// un mapa con ceros — así `fusiona` los trata igual que "sin candidato FTS"
/// (B2, helper puro testeable sin DB).
fn normaliza_fts(candidatos_fts: &[(String, f64)], beta: f64) -> HashMap<String, f64> {
    let f_max = candidatos_fts
        .iter()
        .map(|(_, f_raw)| *f_raw)
        .fold(0.0_f64, f64::max);

    if f_max == 0.0 {
        return HashMap::new();
    }

    candidatos_fts
        .iter()
        .map(|(permalink, f_raw)| (permalink.clone(), beta * f_raw / f_max))
        .collect()
}

/// Fusión por UNIÓN (spec fusión §4.4/§4.5, D-f2), clave `(entity,
/// permalink)`: `score(e) = max(v,f) + bonus·min(v,f)`, canal ausente = 0.
/// Admite la entidad si aparece en CUALQUIERA de los dos mapas (gate FTS =
/// lectura B, el gate lo realiza el término `bonus·min`, no la admisión).
/// Orden por score fusionado desc, truncado a `limite` DESPUÉS de fusionar
/// (mismo contrato que `busca`/`busca_vector`) — helper puro (B2), sin DB.
fn fusiona(
    v_por_entidad: &HashMap<String, f64>,
    f_por_entidad: &HashMap<String, f64>,
    bonus: f64,
    limite: usize,
) -> Vec<Resultado> {
    let claves: std::collections::HashSet<&String> =
        v_por_entidad.keys().chain(f_por_entidad.keys()).collect();

    let mut resultados: Vec<Resultado> = claves
        .into_iter()
        .map(|permalink| {
            let v = *v_por_entidad.get(permalink).unwrap_or(&0.0);
            let f = *f_por_entidad.get(permalink).unwrap_or(&0.0);
            Resultado {
                permalink: permalink.clone(),
                tipo: "entity".to_string(),
                score: v.max(f) + bonus * v.min(f),
            }
        })
        .collect();

    // Desempate determinista por permalink ascendente (M2-09a): misma razón
    // que en `busca_vector` — el `HashSet` de claves de arriba no garantiza
    // orden reproducible entre corridas cuando el score fusionado empata.
    resultados.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.permalink.cmp(&b.permalink))
    });
    resultados.truncate(limite);
    resultados
}

/// Fusión hybrid FTS+vector (`exo search --type hybrid`, M2-07, spec
/// `2026-07-17-fusion-design.md` §4). Candidatos FTS: hasta **K_c = 50**
/// (constante de implementación, NO parámetro del sweep — §4.2, insensible
/// mientras K_c ≫ `limite`) vía `busca()` (ya no trunca a `limite` porque se
/// le pide K_c directamente, sin refactor necesario). Candidatos vector:
/// exhaustivo con threshold pre-fusión sobre `v` (D-f3), vía `busca_vector`
/// con un límite efectivamente sin techo (mismo threshold/precedencia
/// flags>config que el arm vector puro). Normalización BM25 por-query con
/// anclaje β (`escala_fts`) vía `normaliza_fts`; fusión por unión (D-f2) vía
/// `fusiona`. Orden por score fusionado desc, truncado a `limite` DESPUÉS de
/// fusionar (§4.4).
pub fn busca_hybrid(
    db_ruta: &Path,
    query: &str,
    limite: usize,
    min_similitud: Option<f64>,
    bonus: f64,
    escala_fts: f64,
) -> Result<Busqueda> {
    if !db_ruta.exists() {
        anyhow::bail!("DB no encontrada: {}", db_ruta.display());
    }

    let inicio = Instant::now();

    const K_C: usize = 50;
    let fts = busca(db_ruta, query, K_C)?;
    let candidatos_fts: Vec<(String, f64)> = fts
        .results
        .into_iter()
        .map(|r| (r.permalink, r.score))
        .collect();
    let f_por_entidad = normaliza_fts(&candidatos_fts, escala_fts);

    let vector = busca_vector(db_ruta, query, usize::MAX, min_similitud)?;
    let v_por_entidad: HashMap<String, f64> = vector
        .results
        .into_iter()
        .map(|r| (r.permalink, r.score))
        .collect();

    let results = fusiona(&v_por_entidad, &f_por_entidad, bonus, limite);

    Ok(Busqueda {
        query: query.to_string(),
        search_type: "hybrid".to_string(),
        elapsed_s: inicio.elapsed().as_secs_f64(),
        results,
    })
}

#[cfg(test)]
mod tests_fusion {
    use super::*;

    fn mapa(pares: &[(&str, f64)]) -> HashMap<String, f64> {
        pares.iter().map(|(k, s)| (k.to_string(), *s)).collect()
    }

    /// Test contractual 1 (spec §7): con v, f y bonus conocidos, score exacto.
    #[test]
    fn fusion_formula_ambos_canales() {
        let v = mapa(&[("a", 0.6)]);
        let f = mapa(&[("a", 0.4)]);
        let resultados = fusiona(&v, &f, 0.25, 10);
        assert_eq!(resultados.len(), 1);
        assert_eq!(resultados[0].permalink, "a");
        let esperado = 0.6_f64.max(0.4) + 0.25 * 0.6_f64.min(0.4);
        assert!((resultados[0].score - esperado).abs() < 1e-12);
    }

    /// Test contractual 2: candidato solo-vector entra con score == v.
    #[test]
    fn fusion_conserva_candidato_solo_vector() {
        let v = mapa(&[("a", 0.6)]);
        let f = HashMap::new();
        let resultados = fusiona(&v, &f, 0.2, 10);
        assert_eq!(resultados.len(), 1);
        assert_eq!(resultados[0].permalink, "a");
        assert_eq!(resultados[0].score, 0.6);
    }

    /// Test contractual 3: dual del anterior, candidato solo-FTS, score == f.
    #[test]
    fn fusion_conserva_candidato_solo_fts() {
        let v = HashMap::new();
        let f = mapa(&[("a", 0.4)]);
        let resultados = fusiona(&v, &f, 0.2, 10);
        assert_eq!(resultados.len(), 1);
        assert_eq!(resultados[0].permalink, "a");
        assert_eq!(resultados[0].score, 0.4);
    }

    /// Test contractual 5: la misma entidad en ambos canales produce UNA
    /// fila fusionada, no dos (clave = permalink, D-f2).
    #[test]
    fn fusion_clave_entidad_una_fila_por_permalink() {
        let v = mapa(&[("a", 0.6)]);
        let f = mapa(&[("a", 0.4)]);
        let resultados = fusiona(&v, &f, 0.2, 10);
        assert_eq!(resultados.len(), 1, "{:?}", resultados);
    }

    /// Test contractual 6: la normalización preserva el orden FTS y acota a
    /// (0, β]; el top-1 de la query vale exactamente β.
    #[test]
    fn normalizacion_bm25_monotona() {
        let candidatos = vec![
            ("top".to_string(), 10.0),
            ("segundo".to_string(), 5.0),
            ("tercero".to_string(), 1.0),
        ];
        let f = normaliza_fts(&candidatos, 0.8);
        assert_eq!(f["top"], 0.8);
        assert!(f["top"] > f["segundo"], "{:?}", f);
        assert!(f["segundo"] > f["tercero"], "{:?}", f);
        for val in f.values() {
            assert!(*val > 0.0 && *val <= 0.8, "{val} fuera de (0, β]");
        }
    }

    /// Test contractual 7: f_max == 0 descarta el canal FTS sin dividir por 0.
    #[test]
    fn normalizacion_bm25_query_sin_fmax() {
        let candidatos = vec![("a".to_string(), 0.0), ("b".to_string(), 0.0)];
        let f = normaliza_fts(&candidatos, 0.8);
        assert!(f.is_empty(), "{:?}", f);
    }

    /// Test contractual 8: bonus = 0 ⇒ score == max(v,f).
    #[test]
    fn fusion_bonus_cero_es_max() {
        let v = mapa(&[("a", 0.6)]);
        let f = mapa(&[("a", 0.9)]);
        let resultados = fusiona(&v, &f, 0.0, 10);
        assert_eq!(resultados.len(), 1);
        assert_eq!(resultados[0].score, 0.9);
    }

    /// Test contractual 10: orden por score fusionado desc, truncado a
    /// `limite` DESPUÉS de fusionar.
    #[test]
    fn fusion_orden_desc_truncado_post_fusion() {
        let v = mapa(&[("a", 0.9), ("b", 0.5), ("c", 0.1)]);
        let f = HashMap::new();
        let resultados = fusiona(&v, &f, 0.2, 2);
        assert_eq!(resultados.len(), 2, "{:?}", resultados);
        assert_eq!(resultados[0].permalink, "a");
        assert_eq!(resultados[1].permalink, "b");
    }

    /// M2-09a: desempate determinista por permalink ascendente cuando el
    /// score fusionado empata (2/56 queries del corpus tienen empates
    /// reales — no es hipotético). Cinco claves con score idéntico,
    /// insertadas en dos órdenes distintos: el orden de salida debe ser
    /// SIEMPRE alfabético por permalink, sin importar el orden de llegada
    /// (antes del fix, `sort_by` con `partial_cmp` puro es un sort estable
    /// que preserva el orden de iteración del `HashSet` interno de
    /// `fusiona`, no reproducible).
    #[test]
    fn fusion_desempate_determinista_por_permalink() {
        fn mapa_empatado(orden: [&str; 5], score: f64) -> HashMap<String, f64> {
            orden.into_iter().map(|k| (k.to_string(), score)).collect()
        }

        let f_vacio = HashMap::new();
        let v1 = mapa_empatado(["e", "c", "a", "d", "b"], 0.5);
        let r1 = fusiona(&v1, &f_vacio, 0.2, 10);

        let v2 = mapa_empatado(["b", "d", "a", "c", "e"], 0.5);
        let r2 = fusiona(&v2, &f_vacio, 0.2, 10);

        for r in [&r1, &r2] {
            let permalinks: Vec<&str> = r.iter().map(|res| res.permalink.as_str()).collect();
            assert_eq!(
                permalinks,
                vec!["a", "b", "c", "d", "e"],
                "empate quíntuple debe desempatar por permalink ascendente: {r:?}"
            );
        }
    }
}
