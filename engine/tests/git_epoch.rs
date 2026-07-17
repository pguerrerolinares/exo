use exo::indexer::git_epoch_de;
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

#[test]
fn recencia_viene_de_git() {
    // nombre FIJADO §1.1 regla 2
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["config", "user.email", "test@exo.local"]);
    git(dir.path(), &["config", "user.name", "exo-test"]);

    std::fs::write(dir.path().join("nota.md"), "contenido de prueba").unwrap();
    git(dir.path(), &["add", "nota.md"]);

    let epoch_esperado: i64 = 1_700_000_000; // fecha conocida arbitraria
    let fecha = format!("{epoch_esperado} +0000");
    let status = Command::new("git")
        .arg("-C")
        .arg(dir.path())
        .args(["commit", "-q", "-m", "nota inicial"])
        .env("GIT_AUTHOR_DATE", &fecha)
        .env("GIT_COMMITTER_DATE", &fecha)
        .status()
        .expect("git commit");
    assert!(status.success());

    let epoch = git_epoch_de(dir.path(), Path::new("nota.md"));
    assert_eq!(epoch, Some(epoch_esperado));
}

#[test]
fn fichero_sin_commit_devuelve_none() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    std::fs::write(dir.path().join("nueva.md"), "todavía no commiteada").unwrap();

    let epoch = git_epoch_de(dir.path(), Path::new("nueva.md"));
    assert_eq!(epoch, None);
}
