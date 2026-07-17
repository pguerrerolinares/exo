use exo::abre_db_en_memoria;

#[test]
fn fts5_disponible() {
    let db = abre_db_en_memoria().expect("db en memoria");
    db.execute_batch(
        "CREATE VIRTUAL TABLE t USING fts5(cuerpo);
         INSERT INTO t(cuerpo) VALUES ('el engine indexa la kb');",
    )
    .expect("FTS5 compilado en el bundle");
    let n: i64 = db
        .query_row("SELECT count(*) FROM t WHERE t MATCH 'engine'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 1);
}

#[test]
fn sqlite_vec_disponible() {
    let db = abre_db_en_memoria().expect("db en memoria");
    let version: String = db
        .query_row("SELECT vec_version()", [], |r| r.get(0))
        .expect("extensión sqlite-vec registrada");
    assert!(!version.is_empty());
    db.execute_batch("CREATE VIRTUAL TABLE v USING vec0(embedding float[768]);")
        .expect("tabla vec0 de 768 dims");
}

// Descarga el modelo (~0.6 GB) la primera vez: se corre explícito, no en CI de cada merge.
#[test]
#[ignore]
fn jina_es_embebe_a_768() {
    // Tupla renombrada respecto al plan (brief m2-01): el primer elemento es
    // el vector de embedding, no un "modelo" — `embedder_desde_config` ya
    // devuelve (embedding, dims) directamente.
    let (embedding, dims) = exo::embedder_desde_config().expect("fastembed con jina-es");
    assert_eq!(dims, 768);
    assert_eq!(embedding.len(), 768);
}
