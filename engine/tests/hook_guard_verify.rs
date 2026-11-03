use exo::hook::evento::{Clase, Decision, Entorno, Evento, EventoKind, Harness, Tool};
use exo::hook::guards::verify::{Veredicto, veredicto, verify};
use exo::hook::journal::{Registro, registra_tool};
use exo::hook::reflex::lee_cola;
use serde_json::json;
use std::path::Path;
use std::process::Command;

fn reg(clase: &str, tool: &str, cmd: &str, ok: Option<bool>) -> Registro {
    Registro {
        kind: "tool".into(),
        tool: tool.into(),
        clase: clase.into(),
        cmd: cmd.into(),
        ok,
        ..Default::default()
    }
}

fn test_ok() -> Registro {
    reg("Shell", "Bash", "cargo test", Some(true))
}

fn edit() -> Registro {
    reg("FileEdit", "Edit", "", Some(true))
}

#[test]
fn veredicto_sin_test_es_warn() {
    assert_eq!(veredicto(&[]), Veredicto::Warn);
    assert_eq!(
        veredicto(&[edit(), reg("Shell", "Bash", "ls", Some(true))]),
        Veredicto::Warn
    );
}

#[test]
fn veredicto_ultimo_test_ok_es_verde() {
    assert_eq!(veredicto(&[edit(), test_ok()]), Veredicto::Verde);
}

#[test]
fn veredicto_verde_luego_rojo_es_warn() {
    let rojo = reg("Shell", "Bash", "pytest -x", Some(false));
    assert_eq!(veredicto(&[test_ok(), rojo.clone()]), Veredicto::Warn);
    // Control: el orden inverso (rojo y luego verde) sí es verde.
    assert_eq!(veredicto(&[rojo, test_ok()]), Veredicto::Verde);
}

#[test]
fn veredicto_ok_none_es_warn() {
    let desconocido = reg("Shell", "Bash", "cargo test", None);
    assert_eq!(veredicto(&[desconocido]), Veredicto::Warn);
    assert_eq!(veredicto(&[test_ok()]), Veredicto::Verde);
}

#[test]
fn veredicto_edit_tras_verde_es_stale() {
    assert_eq!(veredicto(&[test_ok(), edit()]), Veredicto::Stale);
    // La clase FileEdit basta aunque el nombre de la herramienta no sea Edit/Write.
    let otra = reg("FileEdit", "ApplyPatch", "", Some(true));
    assert_eq!(veredicto(&[test_ok(), otra]), Veredicto::Stale);
    // Edit anterior al test: no es stale.
    assert_eq!(veredicto(&[edit(), test_ok()]), Veredicto::Verde);
    // Solo cuenta la clase Shell como test: un FileEdit con "pytest" en cmd no.
    let falso = reg("FileEdit", "Edit", "pytest", Some(true));
    assert_eq!(veredicto(&[falso]), Veredicto::Warn);
    // Los warn del journal se ignoran.
    let mut w = reg("FileEdit", "Edit", "", None);
    w.kind = "warn".into();
    assert_eq!(veredicto(&[test_ok(), w]), Veredicto::Verde);
}

fn git(dir: &Path, args: &[&str]) {
    let st = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(st.success(), "git {args:?}");
}

fn repo_con_staged(fichero: &str) -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    git(d.path(), &["init", "-q"]);
    std::fs::write(d.path().join(fichero), "x").unwrap();
    git(d.path(), &["add", fichero]);
    d
}

fn entorno(d: &Path) -> Entorno {
    Entorno {
        state_dir: d.join("state"),
        reflex_log: d.join("reflex.jsonl"),
        rutas: exo::state::Rutas::resuelve(std::path::Path::new("."), |_| None),
        seams: exo::state::Seams::resuelve(|_| None),
        cdpath_definido: false,
        panic_guard: None,
    }
}

fn evento(cwd: &Path, cmd: &str, clase: Clase, nombre: &str) -> Evento {
    Evento {
        harness: Harness::Claude,
        kind: EventoKind::PreToolUse,
        session_id: "s1".into(),
        cwd: cwd.to_string_lossy().into(),
        agent_id: String::new(),
        agent_type: String::new(),
        crudo: String::new(),
        tool: Some(Tool {
            nombre: nombre.into(),
            clase,
            input: json!({}),
            command: Some(cmd.into()),
            paths: vec![],
        }),
    }
}

fn commit(cwd: &Path, cmd: &str) -> Evento {
    evento(cwd, cmd, Clase::Shell, "Bash")
}

#[test]
fn verify_solo_con_codigo_staged() {
    let md = repo_con_staged("a.md");
    let en = entorno(md.path());
    assert_eq!(
        verify(&commit(md.path(), "git commit -m x"), &en),
        Decision::Allow
    );
    assert!(lee_cola(&en, 10).is_empty());

    let rs = repo_con_staged("a.rs");
    let en = entorno(rs.path());
    assert_eq!(
        verify(&commit(rs.path(), "git commit -m x"), &en),
        Decision::Allow
    );
    let cola = lee_cola(&en, 10);
    assert_eq!(cola.len(), 1);
    assert_eq!(cola[0]["reflex"], "verify-before-done");
}

#[test]
fn verify_git_y_commit_en_lineas_distintas_no_dispara() {
    let rs = repo_con_staged("a.rs");
    let en = entorno(rs.path());
    verify(&commit(rs.path(), "git\ncommit -m x"), &en);
    assert!(lee_cola(&en, 10).is_empty());
    // Control: en una sola línea sí loguea.
    verify(&commit(rs.path(), "git commit -m x"), &en);
    assert_eq!(lee_cola(&en, 10).len(), 1);
}

#[test]
fn veredicto_test_partido_en_lineas_no_cuenta() {
    for roto in ["cargo\ntest", "npm\ntest"] {
        assert_eq!(
            veredicto(&[reg("Shell", "Bash", roto, Some(true))]),
            Veredicto::Warn,
            "{roto:?}"
        );
    }
    // Control: un test en su propia línea sí cuenta.
    let ok = reg("Shell", "Bash", "echo a\ncargo test", Some(true));
    assert_eq!(veredicto(&[ok]), Veredicto::Verde);
}

#[test]
fn verify_no_verify_no_dispara() {
    let rs = repo_con_staged("a.rs");
    let en = entorno(rs.path());
    assert_eq!(
        verify(&commit(rs.path(), "git commit --no-verify -m x"), &en),
        Decision::Allow
    );
    assert!(lee_cola(&en, 10).is_empty());
    // Control: el mismo repo sin --no-verify sí loguea.
    verify(&commit(rs.path(), "git commit -m x"), &en);
    assert_eq!(lee_cola(&en, 10).len(), 1);
}

#[test]
fn verify_stale_prefijo() {
    let rs = repo_con_staged("a.rs");
    let en = entorno(rs.path());
    let test = evento(rs.path(), "cargo test", Clase::Shell, "Bash");
    registra_tool(&en, &test, Some(true)).unwrap();
    verify(&commit(rs.path(), "git commit -m x"), &en);
    assert!(lee_cola(&en, 10).is_empty(), "verde: no loguea");

    let ed = evento(rs.path(), "", Clase::FileEdit, "Edit");
    registra_tool(&en, &ed, Some(true)).unwrap();
    verify(&commit(rs.path(), "git commit -m x"), &en);
    let cola = lee_cola(&en, 10);
    assert_eq!(cola.len(), 1);
    assert!(cola[0]["payload"].as_str().unwrap().starts_with("stale "));
}
