//! Ola 1 T1: guarda de modelo de embeddings en `meta`.
//!
//! Dos modelos de la misma dimensión (768d: el jina actual y
//! `multilingual-e5-base`) producen blobs indistinguibles para
//! `vectores::lee` — su chequeo de longitud (`BYTES_ESPERADOS`) no separa
//! por procedencia. Sin esta guarda, un `exo index` incremental tras
//! cambiar el modelo activo mezclaría vectores de ambos en la misma tabla
//! sin una sola queja.
//!
//! Estos tests NUNCA cargan el modelo real: la nota fixture tiene cuerpo
//! vacío, así que `trocea` no produce trozos y `con_embedder_de_proceso`
//! jamás se llama (m2-06). La guarda vive al INICIO de `indexa`, antes de
//! tocar fastembed (decisión del brief T1) — por eso estos tests son
//! rápidos y deterministas, y por eso pueden sembrar `meta` a mano en vez
//! de tocar la config real (`config_embeddings()` lee `$HOME` global y no
//! es inyectable; cambiar `$HOME` en el test redescargaría el modelo).

use exo::indexer::indexa;
use tempfile::TempDir;

/// Nota con frontmatter válido y cuerpo vacío: se indexa (tiene permalink)
/// pero no produce trozos, así que jamás toca el embedder de proceso.
fn kb_vacia() -> TempDir {
    let kb = TempDir::new().unwrap();
    std::fs::write(
        kb.path().join("nota.md"),
        "---\npermalink: kb/nota\ntitle: Nota\n---\n",
    )
    .unwrap();
    kb
}

fn db_temporal() -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("i.db");
    (dir, db)
}

#[test]
fn modelo_distinto_aborta_y_cita_rebuild() {
    let kb = kb_vacia();
    let (_dbdir, db) = db_temporal();

    // Primera corrida: DB nueva, meta.modelo_embeddings ausente todavía ->
    // pasa la guarda y escribe el modelo real de la config.
    indexa(kb.path(), &db).unwrap();

    // Se siembra en meta un modelo DISTINTO, como si este índice viniera de
    // otro modelo (sin tocar la config real, ver comentario del módulo).
    let conn = exo::abre_db(&db).unwrap();
    conn.execute(
        "UPDATE meta SET valor = 'multilingual-e5-base' WHERE clave = 'modelo_embeddings'",
        [],
    )
    .unwrap();
    drop(conn);

    let err = indexa(kb.path(), &db).unwrap_err();
    assert!(
        err.to_string().contains("rebuild"),
        "el mensaje debe citar 'exo rebuild', fue: {err}"
    );
}

#[test]
fn clave_ausente_migra_en_silencio() {
    let kb = kb_vacia();
    let (_dbdir, db) = db_temporal();

    indexa(kb.path(), &db).unwrap();

    // Simula un índice viejo, de antes de que esta guarda existiera.
    let conn = exo::abre_db(&db).unwrap();
    conn.execute("DELETE FROM meta WHERE clave = 'modelo_embeddings'", [])
        .unwrap();
    drop(conn);

    // No debe fallar: sin clave que comparar, la guarda deja pasar y
    // `indexa` la reescribe.
    indexa(kb.path(), &db).unwrap();

    let conn = exo::abre_db(&db).unwrap();
    let valor: String = conn
        .query_row(
            "SELECT valor FROM meta WHERE clave = 'modelo_embeddings'",
            [],
            |r| r.get(0),
        )
        .expect("la clave debe quedar reescrita tras la migración silenciosa");
    assert!(!valor.is_empty());
}

#[test]
fn modelo_igual_no_falla_al_reindexar() {
    let kb = kb_vacia();
    let (_dbdir, db) = db_temporal();

    indexa(kb.path(), &db).unwrap();
    // Segunda corrida sin tocar nada: meta.modelo_embeddings ya coincide
    // con la config -> debe seguir sin error.
    indexa(kb.path(), &db).unwrap();

    let conn = exo::abre_db(&db).unwrap();
    let filas: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM meta WHERE clave = 'modelo_embeddings'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(filas, 1, "modelo_embeddings debe ser upsert, no insert repetido");
}
