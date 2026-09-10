//! Task 4 — abstención y ancla de activación: `carga_head` y `anclado_en_head`.
//!
//! Mismo patrón de aislamiento de git que el `#[cfg(test)]` de
//! `engine/src/gitx.rs`: `GIT_CONFIG_GLOBAL`/`GIT_CONFIG_SYSTEM` apuntan a un
//! fichero vacío real (no `/dev/null`: en Windows esa ruta no vale para esta
//! variable) y la identidad de autor/committer va fija por env var, para que
//! el test no dependa del git del desarrollador ni del runner de CI
//! (ubuntu/windows/macos).

use exo::trinquete::{self, Sellos};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Invoca git con el mismo aislamiento que el helper `repo()` de `gitx.rs`.
fn git_aislado(raiz: &Path, cfg: &Path, args: &[&str]) {
    let salida = Command::new("git")
        .arg("-C")
        .arg(raiz)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", cfg)
        .env("GIT_CONFIG_SYSTEM", cfg)
        .env("GIT_AUTHOR_NAME", "f")
        .env("GIT_AUTHOR_EMAIL", "f@k.local")
        .env("GIT_COMMITTER_NAME", "f")
        .env("GIT_COMMITTER_EMAIL", "f@k.local")
        .env("GIT_AUTHOR_DATE", "2026-07-01T10:00:00+02:00")
        .env("GIT_COMMITTER_DATE", "2026-07-01T10:00:00+02:00")
        .output()
        .unwrap();
    assert!(
        salida.status.success(),
        "git {args:?} falló: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
}

/// Inicializa un repo git vacío y aislado en `dir`. Devuelve la ruta del
/// fichero de config vacío, a reutilizar en cualquier comando git posterior
/// sobre el mismo repo.
fn init_repo(dir: &Path) -> PathBuf {
    let cfg = dir.join("gitconfig-vacio");
    std::fs::write(&cfg, "").unwrap();
    git_aislado(dir, &cfg, &["init", "-q"]);
    cfg
}

fn commit_todo(dir: &Path, cfg: &Path, mensaje: &str) {
    git_aislado(dir, cfg, &["add", "."]);
    git_aislado(dir, cfg, &["commit", "-q", "-m", mensaje]);
}

#[test]
fn sin_git_se_abstiene() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(trinquete::carga_head(dir.path()).unwrap(), None);
}

// Mismo montaje que `es_shallow_detecta_un_clone_truncado` en `gitx.rs`: el
// origen necesita al menos dos commits, si no `.git/shallow` no se escribe y
// el test pasaría en verde sin ejercitar nada.
#[test]
fn un_repo_shallow_se_abstiene() {
    let origen = tempfile::tempdir().unwrap();
    let cfg = init_repo(origen.path());
    std::fs::write(origen.path().join("a.md"), "primero\n").unwrap();
    commit_todo(origen.path(), &cfg, "primero");
    std::fs::write(origen.path().join("a.md"), "segundo\n").unwrap();
    commit_todo(origen.path(), &cfg, "segundo");

    let url = format!(
        "file:///{}",
        origen.path().display().to_string().replace('\\', "/")
    );
    let clon_padre = tempfile::tempdir().unwrap();
    let destino = clon_padre.path().join("clon");
    let salida = Command::new("git")
        .args(["clone", "--depth", "1", "-q", &url])
        .arg(&destino)
        .env("GIT_CONFIG_GLOBAL", &cfg)
        .env("GIT_CONFIG_SYSTEM", &cfg)
        .output()
        .unwrap();
    if !salida.status.success() {
        // Igual que el t.Skipf del test Go equivalente: el clone por
        // file:// no siempre se puede en esta máquina (política de
        // seguridad de git, permisos). Se anota, no se falla, y tampoco se
        // finge haber ejercitado la rama.
        eprintln!(
            "un_repo_shallow_se_abstiene: clone por file:// falló, test no \
             ejercitado en esta máquina: {}",
            String::from_utf8_lossy(&salida.stderr)
        );
        return;
    }

    assert_eq!(trinquete::carga_head(&destino).unwrap(), None);
}

// La distinción que es el bug si se confunde: "no hay sello commiteado" NO
// es "no hay historia". Abstenerse aquí desactivaría el trinquete en toda KB
// nueva y nadie se enteraría.
#[test]
fn sin_fichero_committeado_aplica_con_sellos_vacios() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    std::fs::write(dir.path().join("otro.md"), "contenido\n").unwrap();
    commit_todo(dir.path(), &cfg, "inicial, sin sello");

    let sellos = trinquete::carga_head(dir.path()).unwrap();
    assert_eq!(sellos, Some(Sellos::new()));
}

// A8: sin el `./` explícito, git resuelve `HEAD:<ruta>` contra la raíz del
// repo, no contra el `-C`, y este test devolvería vacío en vez del techo —
// pasando en verde el resto de la suite.
#[test]
fn el_sello_se_resuelve_contra_la_kb_no_contra_la_raiz_del_repo() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    let kb = dir.path().join("kb");
    std::fs::create_dir_all(&kb).unwrap();
    std::fs::write(
        kb.join(trinquete::FICHERO_SELLO),
        "{\n  \"ceilings\": {\n    \"a.md\": 100\n  }\n}\n",
    )
    .unwrap();
    commit_todo(dir.path(), &cfg, "sello en kb/, commiteado desde la raiz");

    let sellos = trinquete::carga_head(&kb)
        .unwrap()
        .expect("HEAD resuelve, no es abstencion");
    assert_eq!(sellos.get("a.md"), Some(&100));
}

#[test]
fn un_sello_corrupto_en_head_es_error_no_abstencion() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    std::fs::write(dir.path().join(trinquete::FICHERO_SELLO), "{\"ceilings\":{").unwrap();
    commit_todo(dir.path(), &cfg, "sello corrupto");

    assert!(trinquete::carga_head(dir.path()).is_err());
}

#[test]
fn anclado_en_head_es_falso_antes_de_commitear_el_sello_y_verdadero_despues() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    std::fs::write(dir.path().join("otro.md"), "contenido\n").unwrap();
    commit_todo(dir.path(), &cfg, "sin sello aun");
    assert!(!trinquete::anclado_en_head(dir.path()));

    std::fs::write(
        dir.path().join(trinquete::FICHERO_SELLO),
        trinquete::serializa_sellos(&Sellos::new()),
    )
    .unwrap();
    commit_todo(dir.path(), &cfg, "instala el trinquete");
    assert!(trinquete::anclado_en_head(dir.path()));
}
