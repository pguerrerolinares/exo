//! H4: `resuelve_destinos` corre en cada `--refresh` (≈86% de los prompts) y
//! hacía un UPDATE autocommit por arista aunque nada cambiara. Sin modelo: SQL puro.
use exo::aristas::resuelve_destinos;
use rusqlite::Connection;

fn db_con_aristas() -> Connection {
    let conn = exo::abre_db_en_memoria().unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    conn.execute_batch(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch) VALUES
           ('kb/a', 'a.md', 'A', NULL, 1.0, NULL),
           ('kb/b', 'b.md', 'B', NULL, 1.0, NULL);
         INSERT INTO aristas (origen, destino_texto) VALUES
           ('kb/a', 'B'), ('kb/b', 'kb/a'), ('kb/a', 'nadie'), ('kb/b', 'A|alias');",
    )
    .unwrap();
    conn
}

fn destino(conn: &Connection, origen: &str, texto: &str) -> Option<String> {
    conn.query_row(
        "SELECT destino_permalink FROM aristas WHERE origen = ?1 AND destino_texto = ?2",
        [origen, texto],
        |r| r.get(0),
    )
    .unwrap()
}

#[test]
fn resuelve_por_titulo_por_permalink_con_alias_y_deja_null_el_roto() {
    let conn = db_con_aristas();
    resuelve_destinos(&conn).unwrap();
    assert_eq!(destino(&conn, "kb/a", "B").as_deref(), Some("kb/b"));
    assert_eq!(destino(&conn, "kb/b", "kb/a").as_deref(), Some("kb/a"));
    assert_eq!(destino(&conn, "kb/b", "A|alias").as_deref(), Some("kb/a"));
    assert_eq!(destino(&conn, "kb/a", "nadie"), None);
}

#[test]
fn una_segunda_pasada_sin_cambios_no_escribe_ninguna_fila() {
    let conn = db_con_aristas();
    resuelve_destinos(&conn).unwrap();
    let antes = conn.total_changes();
    resuelve_destinos(&conn).unwrap();
    assert_eq!(
        conn.total_changes() - antes,
        0,
        "sin cambios en notas/aristas no hay nada que escribir"
    );
}

/// Por esto NO vale saltarse la pasada con `indexadas == 0 && borradas == 0`:
/// un abort entre los commits por nota y esta pasada dejaba aristas a NULL, y
/// la corrida siguiente, sin nada que indexar, no las curaría nunca.
#[test]
fn una_arista_desresuelta_se_cura_en_la_pasada_siguiente_escribiendo_solo_esa() {
    let conn = db_con_aristas();
    resuelve_destinos(&conn).unwrap();
    conn.execute(
        "UPDATE aristas SET destino_permalink = NULL WHERE destino_texto = 'B'",
        [],
    )
    .unwrap();
    let antes = conn.total_changes();
    resuelve_destinos(&conn).unwrap();
    assert_eq!(destino(&conn, "kb/a", "B").as_deref(), Some("kb/b"));
    assert_eq!(conn.total_changes() - antes, 1);
}

#[test]
fn una_nota_nueva_que_cura_un_link_roto_escribe_solo_esa_arista() {
    let conn = db_con_aristas();
    resuelve_destinos(&conn).unwrap();
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES ('kb/nadie', 'nadie.md', 'nadie', NULL, 1.0, NULL)",
        [],
    )
    .unwrap();
    let antes = conn.total_changes();
    resuelve_destinos(&conn).unwrap();
    assert_eq!(destino(&conn, "kb/a", "nadie").as_deref(), Some("kb/nadie"));
    assert_eq!(conn.total_changes() - antes, 1);
}
