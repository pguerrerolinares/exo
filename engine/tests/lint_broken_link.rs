use exo::lint;
use rusqlite::Connection;
use std::fs;

/// KB mínima con una arista sin resolver desde `viva.md` y otra, también sin
/// resolver, desde `archive/log/vieja.md`.
fn kb_e_indice() -> (tempfile::TempDir, Connection) {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("archive/log")).unwrap();
    for rel in ["viva.md", "archive/log/vieja.md"] {
        fs::write(dir.path().join(rel), "---\ntier: log\n---\n# n\n").unwrap();
    }
    let conn = exo::abre_db_en_memoria().unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    for (permalink, ruta) in [("kb/viva", "viva.md"), ("kb/vieja", "archive/log/vieja.md")] {
        conn.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
             VALUES (?1, ?2, ?1, 'note', 0.0, NULL)",
            rusqlite::params![permalink, ruta],
        )
        .unwrap();
    }
    for (origen, texto) in [
        ("kb/viva", "roto vivo|alias"),
        ("kb/vieja", "roto archivado"),
    ] {
        conn.execute(
            "INSERT INTO aristas (origen, destino_texto, destino_permalink) VALUES (?1, ?2, NULL)",
            rusqlite::params![origen, texto],
        )
        .unwrap();
    }
    conn.execute(
        "INSERT INTO aristas (origen, destino_texto, destino_permalink)
         VALUES ('kb/viva', 'kb/vieja', 'kb/vieja')",
        [],
    )
    .unwrap();
    (dir, conn)
}

#[test]
fn link_roto_fuera_de_archive_es_hallazgo_y_el_de_archive_no() {
    let (_dir, conn) = kb_e_indice();
    let h = lint::links_rotos(&conn, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert_eq!(h.len(), 1, "{h:?}");
    assert_eq!(h[0].tipo, "broken_link");
    assert_eq!(h[0].ruta, "viva.md");
    assert_eq!(h[0].detalle, "[[roto vivo|alias]]");
}

#[test]
fn analiza_incluye_broken_link_y_gatea() {
    let (dir, conn) = kb_e_indice();
    let r = lint::analiza(
        &conn,
        dir.path(),
        exo::presupuesto::NOMINALES,
        &exo::presupuesto::EXCLUIDOS,
    )
    .unwrap();
    assert!(r.hallazgos.iter().any(|h| h.tipo == "broken_link"));
    assert!(!r.ok);
}
