use exo::abre_db_en_memoria;
use exo::schema::crea_schema;

#[test]
fn schema_crea_todas_las_tablas() {
    let conn = abre_db_en_memoria().expect("db en memoria");
    crea_schema(&conn).expect("crea_schema");
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type IN ('table','view')")
        .unwrap();
    let nombres: Vec<String> = stmt
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    for esperado in ["notas", "notas_fts", "aristas", "trozos", "vectores"] {
        assert!(
            nombres.iter().any(|n| n == esperado),
            "falta tabla {esperado} en {nombres:?}"
        );
    }
}

#[test]
fn crea_schema_es_idempotente() {
    let conn = abre_db_en_memoria().expect("db en memoria");
    crea_schema(&conn).expect("primera creación");
    crea_schema(&conn).expect("segunda creación no debe fallar");
}
