//! M6-01: índice fresco sin daemon. `exo recall --refresca` corre el
//! indexado incremental ANTES de servir, para que el consumidor (el hook de
//! recall de M6) no sirva un bloque de una KB rancia. basic-memory tenía un
//! watch en segundo plano; exo indexa al invocar (spec §4.2: "incremental por
//! mtime/git al invocar, sin daemon salvo que duela").

use exo::indexer::indexa;
use exo::recall::recall_arranque;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// KB mínima con una nota `tier: core` y su repo git (la recencia sale de
/// git, §6.2 regla 2, así que sin commits `git_epoch` queda NULL).
fn kb_con_una_nota(dir: &std::path::Path) {
    fs::write(
        dir.join("core-uno.md"),
        "---\npermalink: kb/uno\ntitle: Uno\ntier: core\n---\n\nCuerpo de uno.\n",
    )
    .unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["add", "-A"],
        vec!["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-qm", "uno"],
    ] {
        Command::new("git").args(&args).current_dir(dir).output().unwrap();
    }
}

/// Añade una nota nueva a la KB ya inicializada y la commitea.
fn anade_nota(dir: &std::path::Path) {
    fs::write(
        dir.join("core-dos.md"),
        "---\npermalink: kb/dos\ntitle: Dos\ntier: core\n---\n\nCuerpo de dos.\n",
    )
    .unwrap();
    for args in [
        vec!["add", "-A"],
        vec!["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-qm", "dos"],
    ] {
        Command::new("git").args(&args).current_dir(dir).output().unwrap();
    }
}

#[test]
fn recall_sin_refrescar_sirve_indice_rancio() {
    let kb = TempDir::new().unwrap();
    let db = TempDir::new().unwrap();
    let db = db.path().join("i.db");
    kb_con_una_nota(kb.path());
    indexa(kb.path(), &db).unwrap();

    anade_nota(kb.path()); // la KB cambia DESPUÉS de indexar

    // Sin refresco, el bloque sigue mostrando solo la nota vieja: ese es
    // justamente el fallo que M6-01 viene a evitar en el hook de arranque.
    let bruto = recall_arranque(&db, kb.path(), 5).unwrap();
    let permalinks: Vec<_> = bruto.notas.iter().map(|n| n.permalink.as_str()).collect();
    assert_eq!(permalinks, vec!["kb/uno"]);
}

#[test]
fn refresca_indice_antes_de_servir_incluye_la_nota_nueva() {
    let kb = TempDir::new().unwrap();
    let db = TempDir::new().unwrap();
    let db = db.path().join("i.db");
    kb_con_una_nota(kb.path());
    indexa(kb.path(), &db).unwrap();

    anade_nota(kb.path());

    let resumen = exo::refresca_indice(kb.path(), &db).unwrap();
    assert_eq!(resumen.indexadas, 1, "solo la nota nueva se reindexa");
    assert_eq!(resumen.saltadas, 1, "la que no cambió se salta (incremental)");

    let bruto = recall_arranque(&db, kb.path(), 5).unwrap();
    let mut permalinks: Vec<_> = bruto.notas.iter().map(|n| n.permalink.clone()).collect();
    permalinks.sort();
    assert_eq!(permalinks, vec!["kb/dos", "kb/uno"]);
}

#[test]
fn refresca_sin_cambios_no_reindexa_nada() {
    let kb = TempDir::new().unwrap();
    let db = TempDir::new().unwrap();
    let db = db.path().join("i.db");
    kb_con_una_nota(kb.path());
    indexa(kb.path(), &db).unwrap();

    // Caso del hook en el 99% de los arranques: la KB no ha cambiado desde
    // el último recall. Debe ser barato — nada que reindexar, nada que
    // embeber (el coste de cargar el modelo ONNX solo se paga si hay texto
    // nuevo que embeber).
    let resumen = exo::refresca_indice(kb.path(), &db).unwrap();
    assert_eq!(resumen.indexadas, 0);
    assert_eq!(resumen.saltadas, 1);
    assert_eq!(resumen.borradas, 0);
}

#[test]
fn refresca_crea_el_indice_si_no_existe() {
    let kb = TempDir::new().unwrap();
    let db = TempDir::new().unwrap();
    let db = db.path().join("no-existe-aun.db");
    kb_con_una_nota(kb.path());

    // Bootstrap: primera invocación en una máquina limpia. `recall` solo
    // fallaría con "DB no encontrada"; con refresco, se construye.
    let resumen = exo::refresca_indice(kb.path(), &db).unwrap();
    assert_eq!(resumen.indexadas, 1);
    assert!(db.exists());
}
