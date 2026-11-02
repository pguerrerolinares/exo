use exo::hook::evento::{Clase, Decision, Entorno, Evento, EventoKind, Harness, Tool};
use exo::hook::guards::zero_residuo::{deny_dirs, zero_residuo};
use exo::hook::reflex::{lee_cola, registra};
use serde_json::json;
use std::path::PathBuf;

const TEXTO_DENY: &str = "Reflejo zero-residuo: `git add -A`/`--all`/`.` sin pathspec arrastra residuo (ficheros ajenos o a medias). Usa rutas explícitas: `git add <ruta> ...`.";

fn entorno(log: PathBuf) -> Entorno {
    Entorno {
        state_dir: PathBuf::from("."),
        reflex_log: log,
        cdpath_definido: false,
        panic_guard: None,
    }
}

fn evento(cmd: &str, cwd: &str, agent: &str) -> Evento {
    Evento {
        harness: Harness::Claude,
        kind: EventoKind::PreToolUse,
        session_id: "s1".into(),
        cwd: cwd.into(),
        agent_id: agent.into(),
        agent_type: String::new(),
        crudo: json!({"tool_input": {"command": cmd}, "cwd": cwd}).to_string(),
        tool: Some(Tool {
            nombre: "Bash".into(),
            clase: Clase::Shell,
            input: json!({"command": cmd}),
            command: Some(cmd.into()),
            paths: vec![],
        }),
    }
}

fn reflexes(en: &Entorno) -> Vec<String> {
    lee_cola(en, 100)
        .iter()
        .map(|v| v["reflex"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn zero_residuo_deny_literal() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path().join("log.jsonl"));
    let r = zero_residuo(&evento("git add -A", "/home/x/repo", ""), &en);
    assert_eq!(r, Decision::Deny(TEXTO_DENY.into()));
    assert_eq!(reflexes(&en), ["zero-residuo-deny"]);
}

#[test]
fn zero_residuo_tmp_solo_loguea() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path().join("log.jsonl"));
    let r = zero_residuo(&evento("git add -A", "/tmp/x", ""), &en);
    assert_eq!(r, Decision::Allow);
    assert_eq!(reflexes(&en), ["zero-residuo"]);
}

#[test]
fn zero_residuo_prefiltro_y_tool_no_shell_no_loguean() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path().join("log.jsonl"));
    let mut ev = evento("git add -A", "/home/x", "");
    ev.crudo = "{}".into();
    assert_eq!(zero_residuo(&ev, &en), Decision::Allow);
    let mut ev = evento("git add -A", "/home/x", "");
    ev.tool.as_mut().unwrap().clase = Clase::Web;
    assert_eq!(zero_residuo(&ev, &en), Decision::Allow);
    assert_eq!(
        zero_residuo(&evento("git status", "/home/x", ""), &en),
        Decision::Allow
    );
    assert!(reflexes(&en).is_empty());
}

#[test]
fn zero_residuo_con_pathspec_solo_loguea() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path().join("log.jsonl"));
    let r = zero_residuo(&evento("git add -A -- src", "/home/x", ""), &en);
    assert_eq!(r, Decision::Allow);
    assert_eq!(reflexes(&en), ["zero-residuo"]);
}

#[test]
fn zero_residuo_backoff_tras_3() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path().join("log.jsonl"));
    let ev = evento("git add -A", "/home/x/repo", "a1");
    for _ in 0..3 {
        assert!(matches!(zero_residuo(&ev, &en), Decision::Deny(_)));
    }
    assert_eq!(zero_residuo(&ev, &en), Decision::Allow);
    assert_eq!(
        reflexes(&en),
        [
            "zero-residuo-deny",
            "zero-residuo-deny",
            "zero-residuo-deny",
            "zero-residuo-backoff"
        ]
    );
    // otro agent_id o otra sesión no heredan la racha
    let otro = evento("git add -A", "/home/x/repo", "a2");
    assert!(matches!(zero_residuo(&otro, &en), Decision::Deny(_)));
    let mut otra_sesion = evento("git add -A", "/home/x/repo", "a1");
    otra_sesion.session_id = "s2".into();
    assert!(matches!(zero_residuo(&otra_sesion, &en), Decision::Deny(_)));
}

#[test]
fn zero_residuo_plano_corta_la_racha() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path().join("log.jsonl"));
    let ev = evento("git add -A", "/home/x/repo", "");
    registra(&en, "zero-residuo-deny", &ev, "x");
    registra(&en, "zero-residuo-deny", &ev, "x");
    registra(&en, "zero-residuo", &ev, "x");
    registra(&en, "zero-residuo-deny", &ev, "x");
    registra(&en, "zero-residuo-deny", &ev, "x");
    assert!(matches!(zero_residuo(&ev, &en), Decision::Deny(_)));
}

#[test]
fn zero_residuo_variable_y_tilde_cuentan_como_cwd() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path().join("log.jsonl"));
    let r = zero_residuo(&evento("git -C $R add -A", "/tmp/x", ""), &en);
    assert_eq!(r, Decision::Allow);
    let r = zero_residuo(&evento("git -C ~/p add -A", "/tmp/x", ""), &en);
    assert_eq!(r, Decision::Allow);
    let r = zero_residuo(&evento("git -C ~/p add -A", "/home/x", ""), &en);
    assert!(matches!(r, Decision::Deny(_)));
}

#[test]
fn deny_dirs_cd_y_variable() {
    assert_eq!(deny_dirs("S=/tmp/x; cd $S/g && git add -A"), ["/tmp/x/g"]);
}

#[test]
fn deny_dirs_heredoc_y_comentario() {
    // positivo suelto: exactamente un dir (vacío = cwd)
    assert_eq!(deny_dirs("git add -A"), [""]);
    assert!(deny_dirs("cat <<EOF\ngit add -A\nEOF").is_empty());
    assert!(deny_dirs("echo hola # git add -A").is_empty());
    assert!(deny_dirs("git add -A -- ruta").is_empty());
    assert!(deny_dirs("git add -A src").is_empty());
    assert_eq!(deny_dirs("git add -A # nota"), [""]);
    assert_eq!(deny_dirs("cat <<<x\ngit add -A"), [""]);
    assert_eq!(deny_dirs("S='/q'; git -C \"$S\" add -A 2>&1"), ["/q"]);
    // tras el heredoc el comando real sí cuenta
    assert_eq!(deny_dirs("cat <<'EOF'\nx\nEOF\ngit add ."), [""]);
    assert_eq!(deny_dirs("git -C /a/b add --all"), ["/a/b"]);
    assert_eq!(deny_dirs("cd /a && cd b && git add -A"), ["/a/b"]);
    assert!(deny_dirs("echo 'x' && echo git add -A").is_empty());
}

#[test]
fn zero_residuo_patron_casa_por_linea_como_grep() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path().join("log.jsonl"));
    zero_residuo(&evento("git\nadd -A", "/tmp/x", ""), &en);
    assert!(reflexes(&en).is_empty());
    zero_residuo(&evento("git add -A\nls", "/tmp/x", ""), &en);
    assert_eq!(reflexes(&en), ["zero-residuo"]);
}

#[test]
fn zero_residuo_deniega_si_cualquier_dir_cae_fuera_de_tmp() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path().join("log.jsonl"));
    let cmd = "cd /tmp/x && git add -A; cd /home/r && git add -A";
    assert!(matches!(
        zero_residuo(&evento(cmd, "/tmp/x", ""), &en),
        Decision::Deny(_)
    ));
    assert_eq!(reflexes(&en), ["zero-residuo-deny"]);
}

#[test]
fn zero_residuo_frontera_tmp_exige_barra() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path().join("log.jsonl"));
    assert!(matches!(
        zero_residuo(&evento("git add -A", "/tmpfoo", ""), &en),
        Decision::Deny(_)
    ));
    assert_eq!(
        zero_residuo(&evento("git add -A", "/tmp", ""), &en),
        Decision::Allow
    );
}
