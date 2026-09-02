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

/// Un vecino del KNN: `rowid` (= `trozos.id`) + distancia nativa de vec0
/// (L2² por ser el `distance_metric` por defecto de la DDL sellada — sin
/// `distance_metric=cosine` explícito en `schema.rs`; conversión a
/// similitud coseno vive en `buscador::busca_vector`, no aquí).
pub struct VecinoKnn {
    pub rowid: i64,
    pub distancia: f64,
}

/// KNN sobre `vectores`: los `k` vecinos más cercanos a `query`, forma del
/// SQL verificada contra el patrón documentado de sqlite-vec (blindspot
/// nota 1): `embedding MATCH ?1 AND k = ?2`. `vectores` vacía (DB sin
/// población) devuelve `Ok(vec![])`, jamás error — el llamador declara qué
/// hacer con 0 resultados (Task 3 del brief).
pub fn knn(conn: &Connection, query: &[f32], k: usize) -> Result<Vec<VecinoKnn>> {
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
}
