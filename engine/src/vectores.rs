//! Helpers de bajo nivel sobre la tabla `vectores` (vec0, schema §2 SELLADO).
//! API de sqlite-vec 0.1.9 verificada contra el C source vendorizado en el
//! crate (`vector_from_value` en `sqlite-vec.c`: acepta blob de f32 nativo
//! además de texto JSON) antes de escribir el SQL de este módulo (blindspot
//! nota 1). `rowid` de `vectores` = `trozos.id` (§2, no negociable).

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};

/// Serializa un embedding a blob de f32 little-endian nativo — la forma que
/// `vector_from_value` de sqlite-vec 0.1.9 acepta directamente como
/// `SQLITE_BLOB` sin pasar por parseo de texto JSON (más barato en el batch
/// del indexer). Esta máquina es little-endian (x86_64/aarch64 modernos);
/// `to_le_bytes` es explícito para no depender del endianness nativo del
/// host si el binario corriera en otra arquitectura.
fn serializa(embedding: &[f32]) -> Vec<u8> {
    embedding.iter().flat_map(|f| f.to_le_bytes()).collect()
}

/// Inserta un embedding en `vectores` con `rowid` explícito (soportado por
/// vec0, verificado con `vector_insert_rowid_y_knn_lo_recupera` abajo).
pub fn inserta(conn: &Connection, rowid: i64, embedding: &[f32]) -> Result<()> {
    conn.execute(
        "INSERT INTO vectores(rowid, embedding) VALUES (?1, ?2)",
        params![rowid, serializa(embedding)],
    )
    .with_context(|| format!("insertar vector rowid={rowid}"))?;
    Ok(())
}

/// Lee el embedding almacenado en `rowid`, o `None` si esa fila no existe.
/// Inversa exacta de `serializa` (f32 little-endian). La usa el cache de
/// embeddings del indexer (M6-01b) para reutilizar el vector de un trozo
/// cuyo texto no cambió, en vez de volver a pagar el modelo. Un blob cuya
/// longitud no sea múltiplo de 4 se trata como ausente en vez de producir un
/// vector truncado: un embedding a medias envenenaría el KNN en silencio, y
/// re-embeber es siempre recuperable.
pub fn lee(conn: &Connection, rowid: i64) -> Result<Option<Vec<f32>>> {
    let blob: Option<Vec<u8>> = conn
        .query_row(
            "SELECT embedding FROM vectores WHERE rowid = ?1",
            params![rowid],
            |r| r.get(0),
        )
        .optional()
        .with_context(|| format!("leer vector rowid={rowid}"))?;
    // 768 dims × 4 bytes: se comprueba la longitud EXACTA, no solo que sea
    // múltiplo de 4 (hallazgo del gate M6). Un blob de otra dimensión
    // —modelo cambiado, fila de otra época— pasaría el filtro de %4 y
    // envenenaría el KNN con un vector de tamaño equivocado. Tratarlo como
    // ausente hace que se re-embeba, que es siempre recuperable.
    const BYTES_ESPERADOS: usize = 768 * 4;
    Ok(blob.filter(|b| b.len() == BYTES_ESPERADOS).map(|b| {
        // `.0` descarta el remainder de `as_chunks`: seguro porque el filter
        // de arriba ya garantiza `b.len() == BYTES_ESPERADOS` (768*4), múltiplo
        // exacto de 4 — el remainder es vacío por construcción, no por suerte.
        b.as_chunks::<4>()
            .0
            .iter()
            .map(|c| f32::from_le_bytes(*c))
            .collect()
    }))
}

/// Borra el vector de `rowid` dado (cascada de borrado extendida a
/// `vectores`, M2-06; soporte de DELETE en vec0 0.1.9 verificado con
/// `vector_delete_por_rowid_desaparece_del_knn` abajo — blindspot nota 3).
pub fn borra(conn: &Connection, rowid: i64) -> Result<()> {
    conn.execute("DELETE FROM vectores WHERE rowid = ?1", params![rowid])
        .with_context(|| format!("borrar vector rowid={rowid}"))?;
    Ok(())
}

/// Un vecino del KNN: `rowid` (= `trozos.id`) + distancia nativa de vec0.
/// `distance_metric` por defecto de la DDL sellada (sin `distance_metric=
/// cosine` explícito en `schema.rs`) es `VEC0_DISTANCE_METRIC_L2`, que en
/// sqlite-vec 0.1.9 despacha a `distance_l2_sqr_float` → `l2_sqr_float` /
/// `_avx` / `_neon` — las tres terminan en `return sqrt(res)`, así que esto
/// es L2 **llana** (`||a-b||`), NO L2 al cuadrado pese al nombre de la
/// función C (confirmado empíricamente en
/// `tests::vec0_metric_l2_default_es_distancia_llana_no_al_cuadrado` más
/// abajo). Conversión a una similitud monótona (no un coseno exacto — ver
/// doc de `buscador::similitud_desde_l2`) vive en `buscador::busca_vector`,
/// no aquí.
pub struct VecinoKnn {
    pub rowid: i64,
    pub distancia: f64,
}

/// Tope duro de `k` en el KNN de vec0: `#define SQLITE_VEC_VEC0_K_MAX 4096`
/// (sqlite-vec 0.1.9, `sqlite-vec.c:7111`). Pedir más es `SQLITE_ERROR`.
const K_MAX_VEC0: usize = 4096;

/// KNN sobre `vectores`: los `k` vecinos más cercanos a `query`, ordenados
/// por la `distance` nativa de vec0. `vectores` vacía ⇒ `Ok(vec![])`, jamás
/// error. Hasta `K_MAX_VEC0` usa el KNN de vec0 (`embedding MATCH ?1 AND k =
/// ?2`); por encima, `barrido_completo` (H27): `busca_vector` pide
/// `k = COUNT(*)` y una KB de más de 4.096 trozos dejaba el arm vector
/// muerto con exit 1.
pub fn knn(conn: &Connection, query: &[f32], k: usize) -> Result<Vec<VecinoKnn>> {
    if k > K_MAX_VEC0 {
        return barrido_completo(conn, query, k);
    }
    let mut stmt = conn.prepare(
        "SELECT rowid, distance
         FROM vectores
         WHERE embedding MATCH ?1 AND k = ?2
         ORDER BY distance",
    )?;
    let filas = stmt
        .query_map(params![serializa(query), k as i64], |r| {
            Ok(VecinoKnn {
                rowid: r.get(0)?,
                distancia: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("leer resultados KNN de vectores")?;
    Ok(filas)
}

/// Mismo contrato que el KNN de vec0 pero sin su tope: la distancia se
/// calcula fila a fila con `vec_distance_l2`, la MISMA función que usa vec0
/// para la métrica L2 (`sqlite-vec.c:1235` y `:6879`, ambas vía
/// `distance_l2_sqr_float`), así que las distancias coinciden; solo el orden
/// entre empates puede variar. Coste lineal, igual que el KNN sin partición.
fn barrido_completo(conn: &Connection, query: &[f32], k: usize) -> Result<Vec<VecinoKnn>> {
    let mut stmt = conn.prepare(
        "SELECT rowid, vec_distance_l2(embedding, ?1) AS distancia
         FROM vectores
         ORDER BY distancia
         LIMIT ?2",
    )?;
    let limite = i64::try_from(k).unwrap_or(i64::MAX);
    let filas = stmt
        .query_map(params![serializa(query), limite], |r| {
            Ok(VecinoKnn {
                rowid: r.get(0)?,
                distancia: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("leer barrido completo de vectores")?;
    Ok(filas)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abre_db_en_memoria;
    use crate::schema::crea_schema;

    fn db_con_schema() -> Connection {
        let conn = abre_db_en_memoria().expect("db en memoria");
        crea_schema(&conn).expect("crea_schema");
        conn
    }

    fn vector_768(valor_primer_componente: f32) -> Vec<f32> {
        let mut v = vec![0.0f32; 768];
        v[0] = valor_primer_componente;
        v[1] = 1.0; // evita el vector cero (norma 0 rompe la distancia coseno de verdad, no aquí, pero es más realista)
        v
    }

    #[test]
    fn vector_insert_rowid_y_knn_lo_recupera() {
        let conn = db_con_schema();
        inserta(&conn, 42, &vector_768(1.0)).expect("insertar rowid=42");

        let vecinos = knn(&conn, &vector_768(1.0), 1).expect("knn");
        assert_eq!(vecinos.len(), 1);
        assert_eq!(vecinos[0].rowid, 42);
        assert!(
            vecinos[0].distancia < 1e-6,
            "vector idéntico ⇒ distancia ~0"
        );
    }

    #[test]
    fn knn_ordena_por_distancia_ascendente() {
        let conn = db_con_schema();
        inserta(&conn, 1, &vector_768(1.0)).unwrap();
        inserta(&conn, 2, &vector_768(5.0)).unwrap();
        inserta(&conn, 3, &vector_768(2.0)).unwrap();

        let vecinos = knn(&conn, &vector_768(1.0), 3).expect("knn");
        assert_eq!(vecinos.len(), 3);
        // el más cercano a vector_768(1.0) es rowid=1 (idéntico), luego 3, luego 2
        assert_eq!(vecinos[0].rowid, 1);
        assert_eq!(vecinos[1].rowid, 3);
        assert_eq!(vecinos[2].rowid, 2);
        assert!(vecinos[0].distancia <= vecinos[1].distancia);
        assert!(vecinos[1].distancia <= vecinos[2].distancia);
    }

    #[test]
    fn vector_delete_por_rowid_desaparece_del_knn() {
        let conn = db_con_schema();
        inserta(&conn, 7, &vector_768(1.0)).expect("insertar rowid=7");
        inserta(&conn, 8, &vector_768(9.0)).expect("insertar rowid=8");

        borra(&conn, 7).expect("borrar rowid=7");

        let vecinos = knn(&conn, &vector_768(1.0), 5).expect("knn");
        assert_eq!(vecinos.len(), 1);
        assert_eq!(vecinos[0].rowid, 8);
    }

    #[test]
    fn knn_sobre_tabla_vacia_devuelve_cero_resultados() {
        let conn = db_con_schema();
        let vecinos = knn(&conn, &vector_768(1.0), 5).expect("knn sobre tabla vacía no es error");
        assert!(vecinos.is_empty());
    }

    /// H27: vec0 0.1.9 rechaza `k > 4096` (`SQLITE_VEC_VEC0_K_MAX`) y
    /// `busca_vector` pide `k = COUNT(*)`. La KB real iba por 3.290 trozos el
    /// 2026-09-13: al cruzar el tope, vector/hybrid/recall --query salían con
    /// exit 1 en cada prompt.
    #[test]
    fn knn_por_encima_del_tope_de_vec0_devuelve_todos_los_vecinos() {
        let conn = db_con_schema();
        let n = K_MAX_VEC0 + 1;
        for i in 0..n {
            inserta(&conn, i as i64 + 1, &vector_768(i as f32)).unwrap();
        }
        let vecinos = knn(&conn, &vector_768(0.0), n).expect("k > 4096 no puede ser un error");
        assert_eq!(vecinos.len(), n);
        assert_eq!(vecinos[0].rowid, 1, "el idéntico a la query va primero");
        assert!(
            vecinos.windows(2).all(|w| w[0].distancia <= w[1].distancia),
            "orden ascendente por distancia"
        );
    }

    /// El barrido tiene que dar la MISMA distancia que el KNN de vec0 para cada
    /// rowid: si difiriera, cruzar el tope cambiaría el ranking y los umbrales.
    #[test]
    fn barrido_y_knn_de_vec0_dan_las_mismas_distancias() {
        let conn = db_con_schema();
        for i in 0..50 {
            inserta(&conn, i + 1, &vector_768(i as f32 * 0.1)).unwrap();
        }
        let q = vector_768(2.05);
        let de_vec0: std::collections::HashMap<i64, f64> = knn(&conn, &q, 50)
            .unwrap()
            .into_iter()
            .map(|v| (v.rowid, v.distancia))
            .collect();
        let barrido = barrido_completo(&conn, &q, 50).unwrap();
        assert_eq!(barrido.len(), 50);
        for v in &barrido {
            assert!(
                (de_vec0[&v.rowid] - v.distancia).abs() < 1e-9,
                "rowid {}: vec0={} barrido={}",
                v.rowid,
                de_vec0[&v.rowid],
                v.distancia
            );
        }
    }

    /// Test falsable de H28: fija empíricamente qué convención de distancia
    /// usa la vec0 0.1.9 vendorizada, en vez de dejarlo en un doc-comment
    /// que nadie ejecuta (justo lo que pasó antes de H28: el comentario
    /// decía "L2 al cuadrado" durante meses y nada lo desmentía). Para dos
    /// vectores UNITARIOS ORTOGONALES la respuesta analítica se bifurca
    /// lejos de cualquier epsilon de float: `||a-b||₂ = √2 ≈ 1.41421`
    /// (L2 llana) contra `||a-b||₂² = 2.0` (L2 al cuadrado). Este test
    /// DEBE fallar el día que sqlite-vec cambie de convención — hoy nada lo
    /// detectaría salvo este assert.
    #[test]
    fn vec0_metric_l2_default_es_distancia_llana_no_al_cuadrado() {
        let conn = db_con_schema();
        let mut a = vec![0.0f32; 768];
        a[0] = 1.0; // unitario: (1, 0, 0, ...)
        let mut b = vec![0.0f32; 768];
        b[1] = 1.0; // unitario y ortogonal a `a`: (0, 1, 0, ...)
        inserta(&conn, 1, &a).expect("insertar a");

        let vecinos = knn(&conn, &b, 1).expect("knn");
        assert_eq!(vecinos.len(), 1);

        let l2_llana = std::f64::consts::SQRT_2; // ≈ 1.41421356
        let l2_al_cuadrado = 2.0f64;
        assert!(
            (vecinos[0].distancia - l2_llana).abs() < 1e-4,
            "vec0 debería devolver L2 llana (√2≈{l2_llana:.5}), no L2² \
             ({l2_al_cuadrado}) — obtenido={}. Si esto falla, sqlite-vec \
             cambió de convención y hay que revisar \
             `buscador::similitud_desde_l2` y `VecinoKnn::distancia`.",
            vecinos[0].distancia
        );
    }
}
