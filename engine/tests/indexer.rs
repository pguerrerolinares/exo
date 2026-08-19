use exo::indexer::indexa;
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

fn arista(db: &Path, origen: &str, destino_texto: &str) -> (String, Option<String>) {
    let conn = exo::abre_db(db).unwrap();
    conn.query_row(
        "SELECT origen, destino_permalink FROM aristas WHERE origen = ?1 AND destino_texto = ?2",
        rusqlite::params![origen, destino_texto],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .unwrap()
}

fn cuenta_aristas(db: &Path) -> i64 {
    let conn = exo::abre_db(db).unwrap();
    conn.query_row("SELECT count(*) FROM aristas", [], |r| r.get(0)).unwrap()
}

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .status()
        .expect("ejecutar git");
    assert!(status.success(), "git {args:?} falló en {dir:?}");
}

fn crea_nota(dir: &Path, nombre: &str, permalink: &str, titulo: &str, cuerpo: &str) {
    let contenido = format!("---\npermalink: {permalink}\ntitle: {titulo}\n---\n{cuerpo}\n");
    std::fs::write(dir.join(nombre), contenido).unwrap();
}

/// KB fixture con 3 notas + git init (mandato del plan T5).
fn kb_fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["config", "user.email", "test@exo.local"]);
    git(dir.path(), &["config", "user.name", "exo-test"]);
    crea_nota(dir.path(), "a.md", "kb-demo/a", "Nota A", "contenido alfa buscable");
    crea_nota(dir.path(), "b.md", "kb-demo/b", "Nota B", "contenido beta buscable");
    crea_nota(dir.path(), "c.md", "kb-demo/c", "Nota C", "contenido gamma buscable");
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "3 notas iniciales"]);
    dir
}

fn db_temporal() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let ruta = dir.path().join("exo.db");
    (dir, ruta)
}

fn permalinks(db: &Path) -> BTreeSet<String> {
    let conn = exo::abre_db(db).unwrap();
    let mut stmt = conn.prepare("SELECT permalink FROM notas ORDER BY permalink").unwrap();
    stmt.query_map([], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}

fn cuenta_trozos(db: &Path, permalink: &str) -> i64 {
    let conn = exo::abre_db(db).unwrap();
    conn.query_row(
        "SELECT count(*) FROM trozos WHERE permalink = ?1",
        rusqlite::params![permalink],
        |r| r.get(0),
    )
    .unwrap()
}

fn cuenta_vectores(db: &Path) -> i64 {
    let conn = exo::abre_db(db).unwrap();
    conn.query_row("SELECT count(*) FROM vectores", [], |r| r.get(0)).unwrap()
}

fn ids_de_trozos(db: &Path, permalink: &str) -> BTreeSet<i64> {
    let conn = exo::abre_db(db).unwrap();
    let mut stmt = conn
        .prepare("SELECT id FROM trozos WHERE permalink = ?1 ORDER BY id")
        .unwrap();
    stmt.query_map(rusqlite::params![permalink], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}

fn rowids_de_vectores(db: &Path) -> BTreeSet<i64> {
    let conn = exo::abre_db(db).unwrap();
    let mut stmt = conn.prepare("SELECT rowid FROM vectores ORDER BY rowid").unwrap();
    stmt.query_map([], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}

#[test]
fn index_puebla_notas_y_fts() {
    let kb = kb_fixture();
    let (_db_dir, db) = db_temporal();

    let resumen = indexa(kb.path(), &db).unwrap();
    assert_eq!(resumen.indexadas, 3);
    assert_eq!(resumen.saltadas, 0);
    assert_eq!(resumen.borradas, 0);

    let conn = exo::abre_db(&db).unwrap();
    let n: i64 = conn.query_row("SELECT count(*) FROM notas", [], |r| r.get(0)).unwrap();
    assert_eq!(n, 3);
    let n_fts: i64 = conn
        .query_row("SELECT count(*) FROM notas_fts", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n_fts, 3);
    let hit: i64 = conn
        .query_row(
            "SELECT count(*) FROM notas_fts WHERE notas_fts MATCH 'alfa'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(hit, 1);
}

#[test]
fn index_incremental_salta_sin_cambios() {
    let kb = kb_fixture();
    let (_db_dir, db) = db_temporal();

    indexa(kb.path(), &db).unwrap();
    let resumen = indexa(kb.path(), &db).unwrap();

    assert_eq!(resumen.indexadas, 0);
    assert_eq!(resumen.saltadas, 3);
    assert_eq!(resumen.borradas, 0);
}

#[test]
fn index_borra_notas_ausentes() {
    let kb = kb_fixture();
    let (_db_dir, db) = db_temporal();

    indexa(kb.path(), &db).unwrap();
    std::fs::remove_file(kb.path().join("b.md")).unwrap();

    let resumen = indexa(kb.path(), &db).unwrap();
    assert_eq!(resumen.borradas, 1);

    let conn = exo::abre_db(&db).unwrap();
    let n: i64 = conn.query_row("SELECT count(*) FROM notas", [], |r| r.get(0)).unwrap();
    assert_eq!(n, 2);
    let n_fts: i64 = conn
        .query_row("SELECT count(*) FROM notas_fts", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n_fts, 2);
    assert!(!permalinks(&db).contains("kb-demo/b"));
}

#[test]
fn rebuild_es_idempotente() {
    let kb = kb_fixture();
    let (_db_dir, db) = db_temporal();

    indexa(kb.path(), &db).unwrap();
    let primera = permalinks(&db);

    std::fs::remove_file(&db).unwrap(); // simula `exo rebuild`: borra + reconstruye
    indexa(kb.path(), &db).unwrap();
    let segunda = permalinks(&db);

    assert_eq!(primera, segunda);
    assert_eq!(primera.len(), 3);
}

/// Test contractual (spec §1.1 fila 6, nombre fijado — intocable): un
/// `[[link roto]]` a una nota inexistente termina exit 0 (aquí: `indexa`
/// devuelve `Ok`) y deja la arista con `destino_permalink NULL` (§6.2 regla 6).
#[test]
fn link_roto_no_es_error() {
    let kb = kb_fixture();
    crea_nota(kb.path(), "a.md", "kb-demo/a", "Nota A", "contenido alfa buscable con [[link roto]]");
    let (_db_dir, db) = db_temporal();

    let resumen = indexa(kb.path(), &db);
    assert!(resumen.is_ok(), "indexar un link roto debe ser exit 0, no error");

    let (origen, destino_permalink) = arista(&db, "kb-demo/a", "link roto");
    assert_eq!(origen, "kb-demo/a");
    assert_eq!(destino_permalink, None);
}

#[test]
fn wikilink_a_nota_existente_resuelve_destino_permalink() {
    let kb = kb_fixture();
    // b.md tiene título "Nota B": un link por título debe resolver a su permalink.
    crea_nota(kb.path(), "a.md", "kb-demo/a", "Nota A", "ver [[Nota B]] para más");
    let (_db_dir, db) = db_temporal();

    indexa(kb.path(), &db).unwrap();

    let (_, destino_permalink) = arista(&db, "kb-demo/a", "Nota B");
    assert_eq!(destino_permalink, Some("kb-demo/b".to_string()));
}

#[test]
fn wikilink_con_alias_se_guarda_entero_y_resuelve_por_parte_antes_del_pipe() {
    let kb = kb_fixture();
    crea_nota(kb.path(), "a.md", "kb-demo/a", "Nota A", "ver [[Nota B|texto alias]] aquí");
    let (_db_dir, db) = db_temporal();

    indexa(kb.path(), &db).unwrap();

    let (_, destino_permalink) = arista(&db, "kb-demo/a", "Nota B|texto alias");
    assert_eq!(destino_permalink, Some("kb-demo/b".to_string()));
}

#[test]
fn wikilink_roto_se_cura_solo_cuando_aparece_la_nota_destino() {
    let kb = kb_fixture();
    crea_nota(kb.path(), "a.md", "kb-demo/a", "Nota A", "ver [[Nota Nueva]] aquí");
    let (_db_dir, db) = db_temporal();

    indexa(kb.path(), &db).unwrap();
    let (_, antes) = arista(&db, "kb-demo/a", "Nota Nueva");
    assert_eq!(antes, None);

    crea_nota(kb.path(), "d.md", "kb-demo/d", "Nota Nueva", "contenido delta");
    git(kb.path(), &["add", "."]);
    git(kb.path(), &["commit", "-q", "-m", "añade Nota Nueva"]);

    indexa(kb.path(), &db).unwrap();
    let (_, despues) = arista(&db, "kb-demo/a", "Nota Nueva");
    assert_eq!(despues, Some("kb-demo/d".to_string()));
}

/// Anclaje del brief: el ciclo de reindex (no el de borrado) no limpiaba
/// aristas antes de M2-04 — una nota que pierde un link dejaba aristas
/// huérfanas. Verifica que reparsear una nota tras quitarle un link borra la
/// arista vieja.
#[test]
fn reindexar_una_nota_que_pierde_un_link_borra_la_arista_vieja() {
    let kb = kb_fixture();
    crea_nota(kb.path(), "a.md", "kb-demo/a", "Nota A", "ver [[Nota B]] aquí");
    let (_db_dir, db) = db_temporal();

    indexa(kb.path(), &db).unwrap();
    assert_eq!(cuenta_aristas(&db), 1);

    // Reescribe a.md sin el link, forzando mtime a cambiar (git commit no es
    // necesario: la detección de cambio del indexer es por mtime, §3).
    std::thread::sleep(std::time::Duration::from_millis(10));
    crea_nota(kb.path(), "a.md", "kb-demo/a", "Nota A", "contenido sin ningún link ya");

    indexa(kb.path(), &db).unwrap();
    assert_eq!(cuenta_aristas(&db), 0, "la arista vieja debe desaparecer, no quedar huérfana");
}

#[test]
fn wikilink_duplicado_en_la_misma_nota_no_duplica_fila() {
    let kb = kb_fixture();
    crea_nota(kb.path(), "a.md", "kb-demo/a", "Nota A", "[[Nota B]] y otra vez [[Nota B]]");
    let (_db_dir, db) = db_temporal();

    indexa(kb.path(), &db).unwrap();
    assert_eq!(cuenta_aristas(&db), 1);
}

#[test]
fn rebuild_doble_da_el_mismo_conteo_de_aristas() {
    let kb = kb_fixture();
    crea_nota(kb.path(), "a.md", "kb-demo/a", "Nota A", "ver [[Nota B]] y [[link roto]]");
    let (_db_dir, db) = db_temporal();

    indexa(kb.path(), &db).unwrap();
    let primera = cuenta_aristas(&db);

    std::fs::remove_file(&db).unwrap(); // simula `exo rebuild`
    indexa(kb.path(), &db).unwrap();
    let segunda = cuenta_aristas(&db);

    assert_eq!(primera, segunda);
    assert_eq!(primera, 2);
}

/// M2-06 Task 2: indexar puebla `trozos` (uno por nota corta de la fixture,
/// cuerpos < 900 chars) y `vectores` con `rowid = trozos.id` exactamente
/// (§2, no negociable).
#[test]
fn index_puebla_trozos_y_vectores_con_rowid_igual_a_trozo_id() {
    let kb = kb_fixture();
    let (_db_dir, db) = db_temporal();

    indexa(kb.path(), &db).unwrap();

    assert_eq!(cuenta_trozos(&db, "kb-demo/a"), 1);
    assert_eq!(cuenta_trozos(&db, "kb-demo/b"), 1);
    assert_eq!(cuenta_trozos(&db, "kb-demo/c"), 1);
    assert_eq!(cuenta_vectores(&db), 3);

    let ids_a = ids_de_trozos(&db, "kb-demo/a");
    let rowids = rowids_de_vectores(&db);
    assert!(
        ids_a.is_subset(&rowids),
        "el id de cada trozo de A debe existir como rowid en vectores: {ids_a:?} ⊄ {rowids:?}"
    );
}

/// M2-06 Task 2: reindexar una nota cambiada reemplaza sus trozos/vectores
/// (reparse ⇒ reindex completo, spec §3), no los acumula.
#[test]
fn reindexar_nota_cambiada_reemplaza_trozos_y_vectores() {
    let kb = kb_fixture();
    let (_db_dir, db) = db_temporal();

    indexa(kb.path(), &db).unwrap();
    let ids_antes = ids_de_trozos(&db, "kb-demo/a");
    assert_eq!(ids_antes.len(), 1);

    std::thread::sleep(std::time::Duration::from_millis(10));
    crea_nota(kb.path(), "a.md", "kb-demo/a", "Nota A", "contenido alfa completamente distinto ahora");
    indexa(kb.path(), &db).unwrap();

    let ids_despues = ids_de_trozos(&db, "kb-demo/a");
    assert_eq!(ids_despues.len(), 1, "sigue siendo 1 trozo (cuerpo corto), no 2 acumulados");
    assert_ne!(ids_antes, ids_despues, "el reindex debe generar filas nuevas, no reusar las viejas");

    // ningún vector huérfano del id viejo
    let rowids = rowids_de_vectores(&db);
    for id_viejo in &ids_antes {
        assert!(!rowids.contains(id_viejo), "vector huérfano del trozo viejo id={id_viejo}");
    }
}

/// M2-06 Task 2 (deferred del gate m2-03): borrar una nota borra también
/// sus `vectores`, no solo sus `trozos` — la cascada de m2-03 se quedaba
/// corta porque `vectores` estaba vacía entonces.
#[test]
fn borrar_nota_borra_tambien_sus_vectores() {
    let kb = kb_fixture();
    let (_db_dir, db) = db_temporal();

    indexa(kb.path(), &db).unwrap();
    let ids_b = ids_de_trozos(&db, "kb-demo/b");
    assert_eq!(ids_b.len(), 1);
    assert!(rowids_de_vectores(&db).is_superset(&ids_b));

    std::fs::remove_file(kb.path().join("b.md")).unwrap();
    let resumen = indexa(kb.path(), &db).unwrap();
    assert_eq!(resumen.borradas, 1);

    assert_eq!(cuenta_trozos(&db, "kb-demo/b"), 0);
    let rowids_despues = rowids_de_vectores(&db);
    for id_b in &ids_b {
        assert!(!rowids_despues.contains(id_b), "vector de la nota borrada debe desaparecer, id={id_b}");
    }
    assert_eq!(cuenta_vectores(&db), 2, "quedan solo los vectores de a y c");
}

/// Nota con cuerpo vacío: 0 trozos, 0 vectores, y el indexado no debe
/// romperse (chunker de trozos.rs ya cubre "nota vacía → 0 trozos" a nivel
/// unitario; este test es el enganche end-to-end vía el indexer).
#[test]
fn nota_con_cuerpo_vacio_no_genera_trozos() {
    let kb = kb_fixture();
    crea_nota(kb.path(), "vacia.md", "kb-demo/vacia", "Vacía", "");
    let (_db_dir, db) = db_temporal();

    let resumen = indexa(kb.path(), &db).unwrap();
    assert_eq!(resumen.indexadas, 4);
    assert_eq!(cuenta_trozos(&db, "kb-demo/vacia"), 0);
}

fn mtime_de_nota(db: &Path, permalink: &str) -> f64 {
    let conn = exo::abre_db(db).unwrap();
    conn.query_row(
        "SELECT mtime FROM notas WHERE permalink = ?1",
        rusqlite::params![permalink],
        |r| r.get(0),
    )
    .unwrap()
}

#[test]
fn un_fallo_a_mitad_no_deja_la_nota_fuera_del_indice() {
    let kb = kb_fixture();
    let (_tmp, db) = db_temporal();
    indexa(kb.path(), &db).unwrap();

    let permalink = permalinks(&db).iter().next().unwrap().clone();
    let ruta = {
        let conn = exo::abre_db(&db).unwrap();
        conn.query_row(
            "SELECT ruta FROM notas WHERE permalink = ?1",
            rusqlite::params![permalink],
            |r| r.get::<_, String>(0),
        )
        .unwrap()
    };
    let mtime_antes = mtime_de_nota(&db, &permalink);

    // el contenido cambia -> la siguiente corrida DEBE reindexar esta nota
    std::fs::write(
        kb.path().join(&ruta),
        format!("---\npermalink: {permalink}\ntitle: T\n---\ncuerpo distinto\n"),
    )
    .unwrap();

    // fallo inyectado en el paso posterior al upsert de `notas`
    {
        let conn = exo::abre_db(&db).unwrap();
        conn.execute_batch(
            "CREATE TRIGGER falla_trozos BEFORE INSERT ON trozos
             BEGIN SELECT RAISE(ABORT, 'fallo inyectado'); END;",
        )
        .unwrap();
    }

    assert!(indexa(kb.path(), &db).is_err(), "se esperaba fallo al embeber");

    // Sin transaccion por nota, el mtime nuevo ya esta commiteado y la nota
    // queda fuera del indice PARA SIEMPRE: la corrida siguiente la salta.
    assert_eq!(
        mtime_de_nota(&db, &permalink),
        mtime_antes,
        "el mtime avanzo pese al fallo: la nota queda fuera del indice para siempre"
    );
}

/// M2-06 oráculo #2: rebuild real es idempotente también para trozos/vectores
/// (mismos counts en dos rebuilds seguidos), no solo para notas/aristas.
#[test]
fn rebuild_doble_da_el_mismo_conteo_de_trozos_y_vectores() {
    let kb = kb_fixture();
    let (_db_dir, db) = db_temporal();

    indexa(kb.path(), &db).unwrap();
    let primera = cuenta_vectores(&db);

    std::fs::remove_file(&db).unwrap(); // simula `exo rebuild`
    indexa(kb.path(), &db).unwrap();
    let segunda = cuenta_vectores(&db);

    assert_eq!(primera, segunda);
    assert_eq!(primera, 3);
}

#[test]
fn indexa_escribe_kb_root_en_meta() {
    let kb = tempfile::tempdir().expect("tempdir kb");
    std::fs::write(
        kb.path().join("nota.md"),
        "---\ntitle: nota\npermalink: kb/nota\n---\n\n# nota\n",
    )
    .expect("escribir nota");

    let dbdir = tempfile::tempdir().expect("tempdir db");
    let db = dbdir.path().join("indice.db");

    exo::indexer::indexa(kb.path(), &db).expect("indexa");

    let conn = exo::abre_db(&db).expect("abrir db");
    let valor: String = conn
        .query_row("SELECT valor FROM meta WHERE clave='kb_root'", [], |r| r.get(0))
        .expect("leer meta.kb_root");

    let esperado = std::fs::canonicalize(kb.path()).expect("canonicalizar kb");
    assert_eq!(valor, esperado.to_string_lossy());
}

#[test]
fn indexa_dos_veces_no_duplica_kb_root() {
    let kb = tempfile::tempdir().expect("tempdir kb");
    std::fs::write(
        kb.path().join("nota.md"),
        "---\ntitle: nota\npermalink: kb/nota\n---\n\n# nota\n",
    )
    .expect("escribir nota");

    let dbdir = tempfile::tempdir().expect("tempdir db");
    let db = dbdir.path().join("indice.db");

    exo::indexer::indexa(kb.path(), &db).expect("primera corrida");
    exo::indexer::indexa(kb.path(), &db).expect("segunda corrida");

    let conn = exo::abre_db(&db).expect("abrir db");
    let filas: i64 = conn
        .query_row("SELECT COUNT(*) FROM meta WHERE clave='kb_root'", [], |r| r.get(0))
        .expect("contar");
    assert_eq!(filas, 1, "kb_root debe ser upsert, no insert repetido");
}
