use exo::abre_db;

#[test]
fn abre_db_deja_la_db_en_wal() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ruta = dir.path().join("indice.db");

    let conn = abre_db(&ruta).expect("abre_db");
    let modo: String = conn
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .expect("leer journal_mode");
    assert_eq!(modo, "wal", "abre_db debe dejar la DB en WAL, no en {modo}");
}

#[test]
fn wal_es_persistente_entre_aperturas() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ruta = dir.path().join("indice.db");

    {
        let conn = abre_db(&ruta).expect("primera apertura");
        conn.execute_batch("CREATE TABLE t (x INTEGER);")
            .expect("crear tabla");
    }

    // Apertura cruda, sin pasar por abre_db: si WAL no fuese persistente en el
    // fichero, aquí saldría 'delete'.
    let conn = rusqlite::Connection::open(&ruta).expect("segunda apertura cruda");
    let modo: String = conn
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .expect("leer journal_mode");
    assert_eq!(modo, "wal");
}
