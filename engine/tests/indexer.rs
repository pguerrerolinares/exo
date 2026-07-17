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
