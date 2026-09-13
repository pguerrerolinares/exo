use crate::abre_db;
use crate::con_embedder_de_proceso;
use anyhow::{Context, Result};
use rusqlite::{OptionalExtension, params};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

/// Un resultado de búsqueda, forma EXACTA del contrato §4.1 de
/// `2026-07-17-indexer-design.md` (sellado, gateado en 4912295).
#[derive(Debug, Serialize, PartialEq)]
pub struct Resultado {
    pub permalink: String,
    /// Siempre `"entity"` en v1 — resultados a nivel entidad, jamás filas
    /// observation (gotcha M0, spec M2 §4).
    #[serde(rename = "type")]
    pub tipo: String,
    /// Escala informativa, no contractual (spec §4.1 literal).
    pub score: f64,
    /// Ruta relativa a la raíz de la KB (M4 §2). **Campo aditivo**: no sube
    /// `SCHEMA_VERSION` (envelope §4).
    ///
    /// Existe porque la ruta NO es derivable del permalink: el slug come
    /// acentos, espacios y em-dashes (`kb-demo/projects/exo-framework-…`
    /// vive en `projects/exo — framework unificado de trabajo agéntico.md`) y
    /// eso no se invierte. Sin este campo, cuando muera basic-memory el agente
    /// no tiene forma de localizar el fichero que va a editar con `Edit`.
    /// `None` solo si el permalink no está en `notas` (índice rancio).
    #[serde(rename = "path")]
    pub ruta: Option<String>,
}

/// `data` del envelope de `exo search`, forma EXACTA del contrato §4.1.
#[derive(Debug, Serialize, PartialEq)]
pub struct Busqueda {
    pub query: String,
    pub search_type: String,
    pub elapsed_s: f64,
    pub results: Vec<Resultado>,
    /// Degradaciones que el consumidor NO puede inferir de `results` (campo
    /// aditivo: omitido cuando está vacío, no sube `SCHEMA_VERSION`).
    ///
    /// Existe por el modo mudo del arm vector: `busca_hybrid` fusionaba una
    /// lista vacía sin distinguir "el vector no encontró nada" de "la tabla
    /// `vectores` está vacía o a medio poblar", y devolvía FTS puro
    /// etiquetado `hybrid` (25/55 donde el instrumento promete 48/55).
    /// `search_type` NO cambia a propósito: lo comparan los scripts del eval.
    #[serde(rename = "warnings", skip_serializing_if = "Vec::is_empty")]
    pub avisos: Vec<String>,
}

/// Avisos de cobertura del arm vector: compara filas de `vectores` contra
/// filas de `trozos`, que es la relación 1:1 que mantiene el indexer.
///
/// Corpus vacío (`trozos == 0`) no avisa: una DB recién creada no está
/// degradada, está vacía. Avisar ahí sería el falso rojo simétrico.
fn avisos_cobertura_vector(conn: &rusqlite::Connection) -> Result<Vec<String>> {
    let trozos: i64 = conn
        .query_row("SELECT count(*) FROM trozos", [], |f| f.get(0))
        .context("contar filas de trozos")?;
    if trozos == 0 {
        return Ok(Vec::new());
    }
    let vectores: i64 = conn
        .query_row("SELECT count(*) FROM vectores", [], |f| f.get(0))
        .context("contar filas de vectores")?;

    Ok(if vectores == 0 {
        vec![format!(
            "arm vector INERTE: 0 vectores para {trozos} trozos.              El resultado sale de FTS puro aunque se etiquete hybrid;              reindexa con `exo index` antes de fiarte del ranking."
        )]
    } else if vectores < trozos {
        vec![format!(
            "cobertura vectorial PARCIAL: {vectores} de {trozos} trozos embebidos.              El arm vector no puede aportar los que faltan."
        )]
    } else {
        Vec::new()
    })
}

/// Rellena `ruta` en los resultados a partir de `notas` (M4 §2). Un único
/// punto para las tres búsquedas: se llama justo antes de devolver, cuando la
/// lista ya está ordenada y truncada, así que consulta como mucho `limite`
/// filas. Un permalink ausente de `notas` deja `None` (índice rancio) en vez
/// de fallar: la ruta es un campo aditivo, no puede tumbar una búsqueda.
fn enriquece_rutas(conn: &rusqlite::Connection, results: &mut [Resultado]) -> Result<()> {
    if results.is_empty() {
        return Ok(());
    }
    let mut stmt = conn
        .prepare("SELECT ruta FROM notas WHERE permalink = ?1")
        .context("preparar consulta de ruta por permalink")?;

    for r in results.iter_mut() {
        r.ruta = stmt
            .query_row(params![r.permalink], |fila| fila.get::<_, String>(0))
            .optional()
            .with_context(|| format!("resolver ruta de {}", r.permalink))?;
    }
    Ok(())
}

/// Todos los permalinks indexados. Lo usa el dup-gate del write-path, que
/// compara slugs sin tocar el modelo de embeddings.
pub fn permalinks(db_ruta: &Path) -> Result<Vec<String>> {
    if !db_ruta.exists() {
        anyhow::bail!("DB no encontrada: {}", db_ruta.display());
    }
    let conn = abre_db(db_ruta)?;
    let mut stmt = conn
        .prepare("SELECT permalink FROM notas")
        .context("preparar consulta de permalinks")?;
    let filas = stmt
        .query_map([], |f| f.get::<_, String>(0))
        .context("leer permalinks")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("materializar permalinks")?;
    Ok(filas)
}

/// Ruta relativa de un permalink concreto, o `None` si no está indexado.
/// La usa el write-path para localizar el fichero de una bitácora sin tener
/// que invertir el slug (que no es invertible).
pub fn ruta_de(db_ruta: &Path, permalink: &str) -> Result<Option<String>> {
    if !db_ruta.exists() {
        anyhow::bail!("DB no encontrada: {}", db_ruta.display());
    }
    let conn = abre_db(db_ruta)?;
    conn.query_row(
        "SELECT ruta FROM notas WHERE permalink = ?1",
        params![permalink],
        |fila| fila.get::<_, String>(0),
    )
    .optional()
    .with_context(|| format!("resolver ruta de {permalink}"))
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
                ruta: None,
            })
        })
        .with_context(|| format!("ejecutar MATCH FTS5 para query preparada: {fts_query}"))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("leer resultados FTS5")?
    };

    let mut results = results;
    enriquece_rutas(&conn, &mut results)?;

    Ok(Busqueda {
        query: query.to_string(),
        search_type: "fts".to_string(),
        elapsed_s: inicio.elapsed().as_secs_f64(),
        results,
        avisos: Vec::new(),
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
/// comparar contra `[embeddings] min_similarity` (threshold pensado en escala
/// coseno, config propia de `~/.exo/config.toml`, hoy 0.35).
fn similitud_desde_l2_cuadrado(distancia_l2_cuadrado: f64) -> f64 {
    1.0 - distancia_l2_cuadrado / 2.0
}

/// Precedencia flags > config (D6): `min_similitud` es el valor de
/// `--min-similarity` si se pasó; si no, cae a `[embeddings] min_similarity`
/// de `~/.exo/config.toml`.
fn min_similitud_efectivo(min_similitud: Option<f64>) -> Result<f64> {
    match min_similitud {
        Some(v) => Ok(v),
        None => crate::min_similitud_de_config(),
    }
}

/// Búsqueda vectorial (`exo search --type vector`, M2-06): embed de la
/// query con el mismo modelo del indexer (jina-es/768, `Embedder` de
/// proceso), KNN cuyo `k` lo fija la propia consulta (H29, ver
/// `busca_vector_con_embedding`), conversión a similitud coseno, filtro por
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

        busca_vector_con_embedding(&conn, &embedding, limite, umbral, total_vectores as usize)
            .context("KNN acotado por consulta")?
    };

    let mut results = results;
    enriquece_rutas(&conn, &mut results)?;
    let avisos = avisos_cobertura_vector(&conn)?;

    Ok(Busqueda {
        query: query.to_string(),
        search_type: "vector".to_string(),
        elapsed_s: inicio.elapsed().as_secs_f64(),
        results,
        avisos,
    })
}

/// Resuelve `permalink` para un conjunto concreto de rowids de `trozos`
/// (H29): reemplaza el `SELECT id, permalink FROM trozos` completo que
/// pagaba una tabla entera por consulta cuando el KNN solo necesitaba unas
/// decenas de filas. `rowids` vacío ⇒ mapa vacío sin tocar la DB.
fn permalinks_de_rowids(
    conn: &rusqlite::Connection,
    rowids: &[i64],
) -> Result<HashMap<i64, String>> {
    if rowids.is_empty() {
        return Ok(HashMap::new());
    }
    let placeholders = std::iter::repeat_n("?", rowids.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!("SELECT id, permalink FROM trozos WHERE id IN ({placeholders})");
    let mut stmt = conn
        .prepare(&sql)
        .context("preparar permalinks por rowid")?;
    let params_dyn: Vec<&dyn rusqlite::ToSql> =
        rowids.iter().map(|r| r as &dyn rusqlite::ToSql).collect();
    stmt.query_map(params_dyn.as_slice(), |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
    })?
    .collect::<rusqlite::Result<_>>()
    .context("leer permalinks de trozos por rowid")
}

/// Agregación chunk→entidad por máxima similitud por permalink (MaxP): un
/// `vecino` cuya similitud no llega a `umbral` no cuenta, y uno cuyo rowid
/// no está en `permalinks` es un trozo huérfano (defensivo, no debería
/// pasar). Pura, sin DB — la comparten `busca_vector_con_embedding` y el
/// test de equivalencia exhaustiva.
fn agrega_maxp(
    vecinos: &[crate::vectores::VecinoKnn],
    permalinks: &HashMap<i64, String>,
    umbral: f64,
) -> HashMap<String, f64> {
    let mut mejor_por_entidad: HashMap<String, f64> = HashMap::new();
    for vecino in vecinos {
        let sim = similitud_desde_l2_cuadrado(vecino.distancia);
        if sim < umbral {
            continue;
        }
        let Some(permalink) = permalinks.get(&vecino.rowid) else {
            continue;
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
    mejor_por_entidad
}

/// Tamaño mínimo de la ventana KNN inicial (H29): con `limite` chico
/// (p.ej. 1), `limite·K_FACTOR_INICIAL` sería demasiado pequeño para
/// absorber la dispersión típica de trozos por nota.
const K_MIN: usize = 64;
/// Factor de la ventana inicial sobre `limite` (H29): 8 trozos candidatos
/// por resultado pedido, calibrado contra la medida del consultor (a 95k
/// trozos, `limite=10` con `sim≥0.35` ya resuelve en la primera ventana el
/// grueso de las 6 queries naturales de `NOTAS-medidas.md`).
const K_FACTOR_INICIAL: usize = 8;
/// Factor de crecimiento de la ventana cuando no alcanza (H29): 4× por
/// iteración llega de la ventana inicial al tope de vec0 (4096) en como
/// mucho 4-5 vueltas incluso para `limite` grande.
const K_FACTOR_CRECIMIENTO: usize = 4;

/// KNN cuyo `k` lo fija la propia consulta, no el tamaño del corpus (H29,
/// hotfix del bug medido por el consultor: `busca_vector` pedía
/// `k = COUNT(*)` siempre, y por encima de 4.096 vecinos vec0 cae al
/// `barrido_completo` de `vectores.rs` a ~130× el costo del KNN nativo —
/// `exo recall --query` a 5.000 notas tardaba 10,2 s, muy por encima del
/// timeout de 5 s del hook).
///
/// **Exacto por construcción** — es el Threshold Algorithm de Fagin, Lotem
/// y Naor (JCSS 2003) con una sola lista ordenada por distancia y
/// agregación MaxP: arranca en
/// `k = min(total_vectores, max(K_MIN, limite·K_FACTOR_INICIAL))` y
/// multiplica `k` por `K_FACTOR_CRECIMIENTO` (tope `total_vectores`) hasta
/// que se cumple alguna de:
///   - ya hay `limite` permalinks distintos con similitud ≥ `umbral` en la
///     ventana actual: como el KNN de vec0 devuelve en orden de distancia
///     creciente (= similitud coseno decreciente, `similitud_desde_l2_cuadrado`
///     es monótona decreciente en la distancia), cualquier permalink que
///     todavía no apareció tiene, como mejor trozo, uno con similitud ≤ la
///     del último vecino de la ventana — que ya es ≤ la de cualquiera de
///     los `limite` ya vistos. No puede desplazar a ninguno de los `limite`
///     mejores;
///   - el último vecino de la ventana ya no pasa `umbral`: la similitud es
///     no creciente en la distancia, así que nada más lejano puede pasarlo
///     tampoco — seguir agrandando `k` es inútil;
///   - `k` alcanzó `total_vectores`: no queda corpus que mirar (aquí el
///     KNN cae al `barrido_completo` de `vectores.rs` si `total_vectores >
///     4096`, mismo fallback patológico de antes, ahora solo alcanzado
///     cuando de verdad hace falta).
///
/// Inyectable sin pasar por el embedder de proceso (recibe el embedding ya
/// calculado) — así lo ejercita el test de equivalencia contra la versión
/// exhaustiva sin cargar el modelo real.
pub fn busca_vector_con_embedding(
    conn: &rusqlite::Connection,
    embedding: &[f32],
    limite: usize,
    umbral: f64,
    total_vectores: usize,
) -> Result<Vec<Resultado>> {
    let mut k = total_vectores.min(K_MIN.max(limite.saturating_mul(K_FACTOR_INICIAL)));

    let mejor_por_entidad = loop {
        let vecinos =
            crate::vectores::knn(conn, embedding, k).context("KNN acotado por consulta")?;
        let rowids: Vec<i64> = vecinos.iter().map(|v| v.rowid).collect();
        let permalinks = permalinks_de_rowids(conn, &rowids)?;
        let mejor_por_entidad = agrega_maxp(&vecinos, &permalinks, umbral);

        let ultimo_pasa_umbral = vecinos
            .last()
            .map(|u| similitud_desde_l2_cuadrado(u.distancia) >= umbral)
            .unwrap_or(false);

        if mejor_por_entidad.len() >= limite || !ultimo_pasa_umbral || k >= total_vectores {
            break mejor_por_entidad;
        }
        k = total_vectores.min(k.saturating_mul(K_FACTOR_CRECIMIENTO));
    };

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

    Ok(entidades
        .into_iter()
        .map(|(permalink, score)| Resultado {
            permalink,
            tipo: "entity".to_string(),
            score,
            ruta: None,
        })
        .collect())
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
                ruta: None,
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
/// le pide K_c directamente, sin refactor necesario). Candidatos vector: con
/// `bonus == 0` (sellado en producción, `BONUS_SELLADO` en `main.rs`, pero
/// override-able con `--bonus`), acotado a `limite` (H29) — la fórmula de
/// fusión (`score = max(v,f) + bonus·min(v,f)`) colapsa a `max(v,f)`, y
/// ningún permalink fuera del top-`limite` por `v` puede desplazar a uno de
/// los `limite` mejores fusionados: el mismo argumento del Threshold
/// Algorithm de `busca_vector_con_embedding` (el `limite`-ésimo por `v` ya
/// domina a cualquier candidato no visto) aplica aquí porque el canal FTS
/// no depende de cuántos candidatos de `v` se pidan. **Con `bonus != 0` esa
/// garantía NO vale** (un permalink con `v` bajo puede colar por el término
/// `bonus·min(v,f)` si su `f` es alto) — ahí el arm vector vuelve a pedirse
/// exhaustivo (mismo comportamiento pre-H29), guardado explícitamente abajo
/// en vez de arriesgar un resultado distinto al pre-fix. Normalización BM25
/// por-query con anclaje β (`escala_fts`) vía `normaliza_fts`; fusión por
/// unión (D-f2) vía `fusiona`. Orden por score fusionado desc, truncado a
/// `limite` DESPUÉS de fusionar (§4.4).
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

    // Guarda explícita (H29, ver doc de la función): el atajo top-`limite`
    // solo es exacto con `bonus == 0.0`. Cualquier `bonus` distinto — hoy
    // solo alcanzable con `--bonus` explícito, `BONUS_SELLADO` es 0.0 —
    // vuelve al arm vector exhaustivo de siempre.
    let limite_vector = if bonus == 0.0 { limite } else { usize::MAX };
    let vector = busca_vector(db_ruta, query, limite_vector, min_similitud)?;
    let avisos = vector.avisos;
    let v_por_entidad: HashMap<String, f64> = vector
        .results
        .into_iter()
        .map(|r| (r.permalink, r.score))
        .collect();

    let mut results = fusiona(&v_por_entidad, &f_por_entidad, bonus, limite);
    // Conexión propia: los dos arms de arriba ya cerraron las suyas, y aquí
    // solo quedan `limite` filas que resolver.
    enriquece_rutas(&abre_db(db_ruta)?, &mut results)?;

    Ok(Busqueda {
        query: query.to_string(),
        search_type: "hybrid".to_string(),
        elapsed_s: inicio.elapsed().as_secs_f64(),
        results,
        avisos,
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

/// Test falsable de H29 (brief `fix-knn-k-por-consulta`): `busca_vector_con_embedding`
/// debe ser BIT A BIT idéntico a un KNN exhaustivo (`k = total_vectores`)
/// para cualquier corpus, sin cargar el embedder real — el embedding de la
/// query es sintético e inyectado directamente.
#[cfg(test)]
mod tests_knn_por_consulta {
    use super::*;
    use crate::abre_db_en_memoria;
    use crate::schema::crea_schema;
    use rusqlite::Connection;

    /// PRNG determinista (SplitMix64): el objetivo es reproducibilidad
    /// semilla→corpus, no calidad criptográfica, así que no hace falta la
    /// crate `rand` (no es dependencia de `exo`).
    struct Rng(u64);
    impl Rng {
        fn semilla(s: u64) -> Self {
            Self(s.wrapping_add(0x9E37_79B9_7F4A_7C15))
        }
        fn u64(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }
        /// Entero uniforme en `[min, max_incl]`.
        fn en_rango(&mut self, min: usize, max_incl: usize) -> usize {
            let span = (max_incl - min + 1) as u64;
            min + (self.u64() % span) as usize
        }
        /// `f32` uniforme en `[-1, 1)`.
        fn f32_signado(&mut self) -> f32 {
            let u = (self.u64() >> 11) as f64 / (1u64 << 53) as f64; // [0,1)
            (u * 2.0 - 1.0) as f32
        }
    }

    /// Vector unitario aleatorio de `dims` componentes — los embeddings
    /// reales (fastembed) siempre tienen norma 1 (ver doc de
    /// `similitud_desde_l2_cuadrado`), y la conversión L2²→coseno solo vale
    /// bajo esa premisa.
    fn vector_unitario(rng: &mut Rng, dims: usize) -> Vec<f32> {
        let mut v: Vec<f32> = (0..dims).map(|_| rng.f32_signado()).collect();
        let norma = v
            .iter()
            .map(|x| (*x as f64) * (*x as f64))
            .sum::<f64>()
            .sqrt();
        if norma > 1e-9 {
            for x in v.iter_mut() {
                *x = (*x as f64 / norma) as f32;
            }
        } else {
            v[0] = 1.0; // degenerado (no debería pasar con 768 dims), evita 0/0
        }
        v
    }

    /// Corpus sintético: `n_notas` notas, cada una con un número aleatorio
    /// (1..=40) de trozos, cada trozo con un embedding unitario aleatorio.
    /// Devuelve la conexión poblada y el total de vectores insertados.
    fn corpus_sintetico(rng: &mut Rng, n_notas: usize) -> (Connection, usize) {
        let mut conn = abre_db_en_memoria().expect("db en memoria");
        crea_schema(&conn).expect("crea_schema");

        let tx = conn.transaction().expect("abrir transacción");
        let mut id: i64 = 0;
        for nota in 0..n_notas {
            let n_trozos = rng.en_rango(1, 40);
            let permalink = format!("nota-{nota}");
            tx.execute(
                "INSERT INTO notas (permalink, ruta, titulo, mtime) VALUES (?1, ?2, ?3, 0.0)",
                rusqlite::params![permalink, format!("{permalink}.md"), permalink],
            )
            .expect("insertar nota");
            for orden in 0..n_trozos {
                id += 1;
                tx.execute(
                    "INSERT INTO trozos (id, permalink, orden, texto) VALUES (?1, ?2, ?3, 'x')",
                    rusqlite::params![id, permalink, orden as i64],
                )
                .expect("insertar trozo");
                let emb = vector_unitario(rng, 768);
                crate::vectores::inserta(&tx, id, &emb).expect("insertar vector");
            }
        }
        tx.commit().expect("commit del corpus sintético");
        (conn, id as usize)
    }

    /// Variante con exactamente un trozo por nota — para el caso k > 4096
    /// (H27) sin pagar hasta 40× más inserts.
    fn corpus_sintetico_un_trozo_por_nota(rng: &mut Rng, n_notas: usize) -> (Connection, usize) {
        let mut conn = abre_db_en_memoria().expect("db en memoria");
        crea_schema(&conn).expect("crea_schema");
        let tx = conn.transaction().expect("abrir transacción");
        for nota in 0..n_notas {
            let id = (nota + 1) as i64;
            let permalink = format!("nota-{nota}");
            tx.execute(
                "INSERT INTO notas (permalink, ruta, titulo, mtime) VALUES (?1, ?2, ?3, 0.0)",
                rusqlite::params![permalink, format!("{permalink}.md"), permalink],
            )
            .expect("insertar nota");
            tx.execute(
                "INSERT INTO trozos (id, permalink, orden, texto) VALUES (?1, ?2, 0, 'x')",
                rusqlite::params![id, permalink],
            )
            .expect("insertar trozo");
            let emb = vector_unitario(rng, 768);
            crate::vectores::inserta(&tx, id, &emb).expect("insertar vector");
        }
        tx.commit().expect("commit del corpus sintético");
        (conn, n_notas)
    }

    /// Oráculo: el comportamiento PRE-H29 (`k = total_vectores` siempre, sin
    /// el bucle por-consulta) — comparte `agrega_maxp`/`permalinks_de_rowids`
    /// con la función bajo test porque esas dos NO son lo que se está
    /// verificando (la agregación MaxP ya tenía sus propios tests); lo que
    /// se verifica es que la ventana `k` recortada y creciente del bucle
    /// llegue exactamente al mismo resultado que mirar todo el corpus de
    /// una sola vez.
    fn referencia_exhaustiva(
        conn: &Connection,
        embedding: &[f32],
        limite: usize,
        umbral: f64,
        total: usize,
    ) -> Vec<Resultado> {
        let vecinos = crate::vectores::knn(conn, embedding, total).expect("knn exhaustivo");
        let rowids: Vec<i64> = vecinos.iter().map(|v| v.rowid).collect();
        let permalinks = permalinks_de_rowids(conn, &rowids).expect("permalinks");
        let mejor_por_entidad = agrega_maxp(&vecinos, &permalinks, umbral);

        let mut entidades: Vec<(String, f64)> = mejor_por_entidad.into_iter().collect();
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
                ruta: None,
            })
            .collect()
    }

    /// Test falsable central: 200 semillas × 3 umbrales × 3 `limite` = 1.800
    /// comparaciones exactas (permalinks, orden Y scores) contra la
    /// referencia exhaustiva. Corpus: 3-30 notas, 1-40 trozos/nota,
    /// vectores normalizados a norma unidad (como los embeddings reales).
    ///
    /// Mutación probada a mano (no se deja en el árbol, ver PR): parar en
    /// `mejor_por_entidad.len() >= limite.saturating_sub(1)` en vez de
    /// `>= limite` pone este test en rojo (falta la última entidad del
    /// top-`limite`, p.ej. semilla=61, umbral=0, limite=10: devuelve 9
    /// resultados en vez de 10, `nota-7` desaparece). Quitar la rama
    /// `!ultimo_pasa_umbral` en cambio NO lo enrojece: esa condición es una
    /// poda de rendimiento (evita seguir creciendo `k` cuando ya no puede
    /// aparecer nada más sobre el umbral), no de corrección — sin ella el
    /// bucle simplemente sigue creciendo `k` de más hasta topar con
    /// `distintas >= limite` o `k == total`, mismo resultado final, más
    /// lento. Verificado con las dos mutaciones antes de commitear.
    #[test]
    fn equivalencia_exacta_contra_exhaustiva() {
        let umbrales = [0.0, 0.3, 0.45];
        let limites = [1usize, 4, 10];

        for semilla in 0u64..200 {
            let mut rng = Rng::semilla(semilla);
            let n_notas = rng.en_rango(3, 30);
            let (conn, total) = corpus_sintetico(&mut rng, n_notas);
            let query = vector_unitario(&mut rng, 768);

            for &umbral in &umbrales {
                for &limite in &limites {
                    let obtenido = busca_vector_con_embedding(&conn, &query, limite, umbral, total)
                        .unwrap_or_else(|e| {
                            panic!("semilla={semilla} umbral={umbral} limite={limite}: {e}")
                        });
                    let esperado = referencia_exhaustiva(&conn, &query, limite, umbral, total);
                    assert_eq!(
                        obtenido, esperado,
                        "semilla={semilla} n_notas={n_notas} total={total} umbral={umbral} limite={limite}"
                    );
                }
            }
        }
    }

    /// Cubre el caso k > 4096 (H27+H29): con `total_vectores` por encima del
    /// tope de vec0, el bucle tiene que escalar `k` hasta cruzarlo (donde
    /// `vectores::knn` cae a `barrido_completo`) y seguir siendo exacto.
    #[test]
    fn equivalencia_exacta_por_encima_del_tope_vec0() {
        let mut rng = Rng::semilla(999_983);
        let n_notas = 4200; // 1 trozo/nota ⇒ total_vectores = 4200 > 4096
        let (conn, total) = corpus_sintetico_un_trozo_por_nota(&mut rng, n_notas);
        assert!(total > 4096, "total={total} debe superar el tope de vec0");
        let query = vector_unitario(&mut rng, 768);

        for &umbral in &[0.0, 0.3] {
            for &limite in &[1usize, 10] {
                let obtenido =
                    busca_vector_con_embedding(&conn, &query, limite, umbral, total).unwrap();
                let esperado = referencia_exhaustiva(&conn, &query, limite, umbral, total);
                assert_eq!(
                    obtenido, esperado,
                    "umbral={umbral} limite={limite} total={total}"
                );
            }
        }
    }
}
