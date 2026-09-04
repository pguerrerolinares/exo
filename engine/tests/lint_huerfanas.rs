use exo::lint;
use rusqlite::Connection;
use std::fs;

/// KB + índice con: una huérfana pura, una huérfana con waiver, una nota
/// enlazada, y —lo que hace válido el test— una arista SIN resolver.
fn kb_e_indice() -> (tempfile::TempDir, Connection) {
    let dir = tempfile::tempdir().unwrap();
    for (rel, extra) in [
        ("huerfana.md", ""),
        ("waivada.md", "kbx_orphan_ok: true\n"),
        ("hub.md", ""),
        ("enlazada.md", ""),
    ] {
        fs::write(
            dir.path().join(rel),
            format!("---\ntier: log\n{extra}---\n# {rel}\n"),
        )
        .unwrap();
    }
    let conn = exo::abre_db_en_memoria().unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    for (permalink, ruta) in [
        ("kb/huerfana", "huerfana.md"),
        ("kb/waivada", "waivada.md"),
        ("kb/hub", "hub.md"),
        ("kb/enlazada", "enlazada.md"),
    ] {
        conn.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
             VALUES (?1, ?2, ?1, 'note', 0.0, NULL)",
            rusqlite::params![permalink, ruta],
        )
        .unwrap();
    }
    conn.execute(
        "INSERT INTO aristas (origen, destino_texto, destino_permalink)
         VALUES ('kb/hub', 'enlazada', 'kb/enlazada')",
        [],
    )
    .unwrap();
    // La arista SIN resolver: es la que arma la trampa del NOT IN.
    conn.execute(
        "INSERT INTO aristas (origen, destino_texto, destino_permalink)
         VALUES ('kb/hub', 'nota que no existe', NULL)",
        [],
    )
    .unwrap();
    (dir, conn)
}

#[test]
fn el_fixture_contiene_una_arista_sin_resolver() {
    // Auto-invalidación: si esto deja de ser cierto, el test de abajo pasaría
    // igual con la guarda IS NOT NULL quitada y dejaría de probar nada.
    let (_dir, conn) = kb_e_indice();
    let sin_resolver: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM aristas WHERE destino_permalink IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        sin_resolver >= 1,
        "el fixture ya no tiene aristas sin resolver: el test de huérfanas deja de falsar su guarda"
    );
}

#[test]
fn las_huerfanas_sobreviven_a_las_aristas_sin_resolver() {
    let (dir, conn) = kb_e_indice();
    let (hallazgos, _) = lint::huerfanas(&conn, dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(
        !hallazgos.is_empty(),
        "cero huérfanas con aristas sin resolver: falta la guarda IS NOT NULL"
    );
    let rutas: Vec<&str> = hallazgos.iter().map(|h| h.ruta.as_str()).collect();
    assert!(rutas.contains(&"huerfana.md"));
    assert!(
        !rutas.contains(&"enlazada.md"),
        "una nota enlazada no es huérfana"
    );
    assert!(
        !rutas.contains(&"hub.md"),
        "el origen de una arista no es huérfano"
    );
}

#[test]
fn el_marcador_manda_la_huerfana_a_waived_y_no_a_hallazgos() {
    let (dir, conn) = kb_e_indice();
    let (hallazgos, waived) =
        lint::huerfanas(&conn, dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(!hallazgos.iter().any(|h| h.ruta == "waivada.md"));
    assert_eq!(waived.len(), 1);
    assert_eq!(waived[0].ruta, "waivada.md");
    assert!(waived[0].detalle.contains("waived: kbx_orphan_ok"));
}

#[test]
fn una_nota_ilegible_falla_hacia_rojo() {
    // Deriva índice/disco: el marcador no se puede leer, así que no absuelve.
    let (dir, conn) = kb_e_indice();
    fs::remove_file(dir.path().join("waivada.md")).unwrap();
    let (hallazgos, waived) =
        lint::huerfanas(&conn, dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(hallazgos.iter().any(|h| h.ruta == "waivada.md"));
    assert!(waived.is_empty());
}

#[test]
fn una_huerfana_bajo_archive_no_se_reporta_ni_en_windows() {
    // Dos cosas a la vez: que la exclusión se aplica a este check (Go tiene
    // TestOrphanExcludesNoteUnderArchive), y que se aplica aunque la fila venga
    // de la DB con el separador nativo de Windows, que es como la escribe
    // `indexer::ruta_relativa`.
    let (dir, conn) = kb_e_indice();
    fs::create_dir_all(dir.path().join("archive")).unwrap();
    fs::write(dir.path().join("archive/vieja.md"), "---\ntier: log\n---\n").unwrap();
    for ruta in ["archive/vieja.md", "archive\\otra.md"] {
        conn.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
             VALUES (?1, ?2, ?1, 'note', 0.0, NULL)",
            rusqlite::params![format!("kb/{ruta}"), ruta],
        )
        .unwrap();
    }
    let (hallazgos, _) = lint::huerfanas(&conn, dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(
        !hallazgos.iter().any(|h| h.ruta.contains("archive")),
        "una nota bajo archive/ no es huérfana reportable: {hallazgos:?}"
    );
}

#[test]
fn ninguna_ruta_reportada_lleva_separador_de_windows() {
    let (dir, conn) = kb_e_indice();
    conn.execute(
        "UPDATE notas SET ruta = 'sub\\huerfana.md' WHERE permalink = 'kb/huerfana'",
        [],
    )
    .unwrap();
    let (hallazgos, _) = lint::huerfanas(&conn, dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(
        hallazgos.iter().all(|h| !h.ruta.contains('\\')),
        "un `\\` se cuela en el JSON: {hallazgos:?}"
    );
}

#[test]
fn no_hay_filtro_por_tipo_de_nota() {
    // El viejo filtro note_type='note' escondía 57 de 138 notas reales;
    // retirado en M6-04 T3. Reintroducirlo apaga el check en silencio.
    let (dir, conn) = kb_e_indice();
    conn.execute(
        "UPDATE notas SET tipo = 'project' WHERE permalink = 'kb/huerfana'",
        [],
    )
    .unwrap();
    let (hallazgos, _) = lint::huerfanas(&conn, dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(hallazgos.iter().any(|h| h.ruta == "huerfana.md"));
}
