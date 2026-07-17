use exo::indexer::indexa;
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

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
