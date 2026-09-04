use exo::lint;
use exo::presupuesto::NOMINALES;
use std::fs;

fn kb_y_conn(notas: &[&str], indexadas: &[&str]) -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempfile::tempdir().unwrap();
    for rel in notas {
        let ruta = dir.path().join(rel);
        fs::create_dir_all(ruta.parent().unwrap()).unwrap();
        // `permalink:` real: desde el fix de `indice_rancio`, la causa de
        // "no indexada" se decide leyendo la nota en disco, y sin esta clave
        // cualquier nota de este fixture caería en la rama "sin permalink"
        // en vez de en la de deriva genuina que estos tests quieren probar.
        fs::write(
            ruta,
            format!("---\ntier: log\npermalink: kb/{rel}\n---\n# x\n"),
        )
        .unwrap();
    }
    let conn = exo::abre_db_en_memoria().unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    for rel in indexadas {
        conn.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
             VALUES (?1, ?2, ?1, 'note', 0.0, NULL)",
            rusqlite::params![format!("kb/{rel}"), rel],
        )
        .unwrap();
    }
    (dir, conn)
}

#[test]
fn un_indice_vacio_sobre_una_kb_con_notas_no_puede_dar_verde() {
    // EL caso: sin `exo index`, `orphan` no ve nada y `lint` decía "ok".
    let (dir, conn) = kb_y_conn(&["a.md", "b.md"], &[]);
    let informe =
        lint::analiza(&conn, dir.path(), NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(
        !informe.ok,
        "índice vacío y KB con notas: no puede salir ok"
    );
    let stale: Vec<_> = informe
        .hallazgos
        .iter()
        .filter(|h| h.tipo == "index_stale")
        .collect();
    assert_eq!(
        stale.len(),
        1,
        "el índice vacío es UN hallazgo, no uno por nota"
    );
    // El conteo va en el detalle: un `rutas.len()` mutado a un literal fijo
    // seguiría conteniendo "exo index" y este test no lo vería.
    assert_eq!(
        stale[0].detalle,
        "2 nota(s) en disco y el índice vacío — corre `exo index`"
    );
    // El hallazgo agregado no apunta a una nota concreta, apunta a la KB
    // entera: `ruta` vacía es la convención, no un descuido.
    assert_eq!(stale[0].ruta, "");
}

#[test]
fn una_nota_en_disco_que_el_indice_no_conoce_es_un_hallazgo_por_nota() {
    let (dir, conn) = kb_y_conn(&["a.md", "b.md", "c.md"], &["a.md"]);
    let h = lint::indice_rancio(
        &conn,
        dir.path(),
        &["a.md".into(), "b.md".into(), "c.md".into()],
    )
    .unwrap();
    let rutas: Vec<&str> = h.iter().map(|f| f.ruta.as_str()).collect();
    assert_eq!(rutas, vec!["b.md", "c.md"]);
    assert!(h.iter().all(|f| f.tipo == "index_stale"));
    assert!(
        h.iter()
            .all(|f| f.detalle == "en disco y no en el índice — corre `exo index`")
    );
}

#[test]
fn un_indice_al_dia_no_dice_nada() {
    // Error del brief: `kb_y_conn` indexa "a.md" sin ninguna arista, y con la
    // nota YA en el índice `huerfanas` (el otro check que lee `notas`) la
    // marca huérfana — el informe salía sucio por un motivo ajeno a
    // `index_stale`. Se le da una arista real para que este test pruebe
    // solo lo que dice probar: un índice al día no dispara `index_stale`.
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.md"), "---\ntier: log\n---\n# x\n").unwrap();
    let conn = exo::abre_db_en_memoria().unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES ('kb/a.md', 'a.md', 'kb/a.md', 'note', 0.0, NULL)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO aristas (origen, destino_texto, destino_permalink)
         VALUES ('kb/a.md', 'destino', NULL)",
        [],
    )
    .unwrap();

    let h = lint::indice_rancio(&conn, dir.path(), &["a.md".into()]).unwrap();
    assert!(h.is_empty(), "índice al día: {h:?}");
    // Y el informe entero sale limpio.
    let informe =
        lint::analiza(&conn, dir.path(), NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(informe.ok, "{:?}", informe.hallazgos);
}

#[test]
fn el_separador_de_windows_del_indice_no_inventa_deriva() {
    // `notas.ruta` viene con separador nativo; si no se normaliza, en W11 toda
    // nota en subdirectorio parecería no indexada y `index_stale` gritaría
    // sobre una KB perfectamente al día.
    let (dir, conn) = kb_y_conn(&["sub/a.md"], &[]);
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES ('kb/sub/a', 'sub\\a.md', 'a', 'note', 0.0, NULL)",
        [],
    )
    .unwrap();
    let h = lint::indice_rancio(&conn, dir.path(), &["sub/a.md".into()]).unwrap();
    assert!(
        h.is_empty(),
        "el `\\` del índice se leyó como nota distinta: {h:?}"
    );
}

#[test]
fn una_kb_vacia_con_indice_vacio_no_es_deriva() {
    // Sin notas en disco no hay nada que indexar: no es un índice apagado.
    let (dir, conn) = kb_y_conn(&[], &[]);
    let informe =
        lint::analiza(&conn, dir.path(), NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(informe.ok, "{:?}", informe.hallazgos);
}
