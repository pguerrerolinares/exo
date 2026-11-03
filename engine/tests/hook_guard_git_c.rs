use exo::hook::evento::{Clase, Decision, Entorno, Evento, EventoKind, Harness, Tool};
use exo::hook::guards::git_c::git_c;
use exo::hook::reflex::lee_cola;
use serde_json::{Value, json};

fn entorno(d: &tempfile::TempDir) -> Entorno {
    Entorno {
        state_dir: d.path().to_path_buf(),
        reflex_log: d.path().join("log.jsonl"),
        rutas: exo::state::Rutas::resuelve(std::path::Path::new("."), |_| None),
        seams: exo::state::Seams::resuelve(|_| None),
        cdpath_definido: false,
        panic_guard: None,
    }
}

fn evento_crudo(nombre: &str, cmd: &str, crudo: Option<String>) -> Evento {
    let input = json!({"command": cmd, "description": "d"});
    Evento {
        harness: Harness::Claude,
        kind: EventoKind::PreToolUse,
        session_id: "s".into(),
        cwd: "/tmp/x".into(),
        agent_id: String::new(),
        agent_type: String::new(),
        crudo: crudo
            .unwrap_or_else(|| json!({"tool_name": nombre, "tool_input": input}).to_string()),
        tool: Some(Tool {
            nombre: nombre.into(),
            clase: Clase::Shell,
            input,
            command: Some(cmd.into()),
            paths: vec![],
        }),
    }
}

fn ev(cmd: &str) -> Evento {
    evento_crudo("Bash", cmd, None)
}

fn log(d: &tempfile::TempDir) -> Vec<Value> {
    lee_cola(&entorno(d), 50)
}

fn reflexes(d: &tempfile::TempDir) -> Vec<String> {
    log(d)
        .iter()
        .map(|v| v["reflex"].as_str().unwrap().to_string())
        .collect()
}

fn rewrite_cmd(d: &Decision) -> Option<String> {
    match d {
        Decision::Rewrite { input, .. } => input["command"].as_str().map(str::to_string),
        _ => None,
    }
}

#[test]
fn git_c_rewrite_literal() {
    let d = tempfile::tempdir().unwrap();
    let dec = git_c(&ev("cd repo && git status"), &entorno(&d), false);
    assert_eq!(
        dec,
        Decision::Rewrite {
            input: json!({"command": "git -C repo status"}),
            razon: "Reflejo git-c: reescrito a `git -C repo status` (equivalente; evita el cd encadenado)".into(),
        }
    );
    let l = log(&d);
    assert_eq!(l.len(), 1);
    assert_eq!(l[0]["reflex"], "git-c-rewrite");
    assert_eq!(
        l[0]["payload"],
        "cd repo && git status -> git -C repo status"
    );
    for (c, esperado) in [
        (
            "cd /opt/p && git log --oneline -5",
            "git -C /opt/p log --oneline -5",
        ),
        (
            "cd ~/p/x.y && git diff --stat",
            "git -C ~/p/x.y diff --stat",
        ),
        ("cd /r&&git status", "git -C /r status"),
        ("cd /r && git status", "git -C /r status"),
    ] {
        assert_eq!(
            rewrite_cmd(&git_c(&ev(c), &entorno(&d), false)).as_deref(),
            Some(esperado),
            "{c}"
        );
    }
}

#[test]
fn git_c_fuera_de_allowlist_o_con_metacaracter_loguea() {
    for (i, c) in [
        "cd a && git commit",
        "cd a && git log | head",
        "cd /r && git status\necho hi",
        "cd /r && git push",
        "cd /r && git branch foo",
        "cd /r && git status && echo done",
        "cd /r && git status 2>/dev/null",
        "cd /r && git ls-files *.md",
        "cd /r; git status",
        "cd - && git status",
        "cd /r/../o && git log",
        "cd \"$DIR\" && git status",
        "echo 'cd /x && git status'",
    ]
    .iter()
    .enumerate()
    {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(
            git_c(&ev(c), &entorno(&d), false),
            Decision::Allow,
            "{i}: {c}"
        );
        assert_eq!(reflexes(&d), ["git-c"], "{i}: {c}");
    }
    let d = tempfile::tempdir().unwrap();
    assert_eq!(
        git_c(&ev("cd /r && make && git status"), &entorno(&d), false),
        Decision::Allow
    );
    assert!(log(&d).is_empty());
}

#[test]
fn git_c_powershell() {
    let d = tempfile::tempdir().unwrap();
    let ps = |c: &str| evento_crudo("PowerShell", c, None);
    let dec = git_c(&ps("Set-Location a && git status"), &entorno(&d), false);
    assert_eq!(rewrite_cmd(&dec).as_deref(), Some("git -C a status"));
    let d2 = tempfile::tempdir().unwrap();
    assert_eq!(
        git_c(&ps("cd ~/a && git status"), &entorno(&d2), false),
        Decision::Allow
    );
    assert_eq!(reflexes(&d2), ["git-c"]);
    // Set-Location solo es cd en PowerShell.
    let d3 = tempfile::tempdir().unwrap();
    assert_eq!(
        git_c(&ev("Set-Location a && git status"), &entorno(&d3), false),
        Decision::Allow
    );
    assert!(log(&d3).is_empty());
    // Rama 1b exclusiva de PowerShell.
    let d4 = tempfile::tempdir().unwrap();
    assert_eq!(
        git_c(&ps("pushd a; git status"), &entorno(&d4), false),
        Decision::Allow
    );
    assert_eq!(reflexes(&d4), ["git-c"]);
    let d5 = tempfile::tempdir().unwrap();
    assert_eq!(
        git_c(&ev("pushd a; git status"), &entorno(&d5), false),
        Decision::Allow
    );
    assert!(log(&d5).is_empty());
}

#[test]
fn git_c_relativo_con_cdpath_no_reescribe() {
    let d = tempfile::tempdir().unwrap();
    assert_eq!(
        git_c(&ev("cd repo && git status"), &entorno(&d), true),
        Decision::Allow
    );
    assert_eq!(reflexes(&d), ["git-c"]);
    let d2 = tempfile::tempdir().unwrap();
    assert!(rewrite_cmd(&git_c(&ev("cd repo && git status"), &entorno(&d2), false)).is_some());
    // CDPATH solo afecta a rutas relativas.
    assert!(rewrite_cmd(&git_c(&ev("cd /abs && git status"), &entorno(&d2), true)).is_some());
}

#[test]
fn git_c_punto_punto_no_reescribe() {
    let d = tempfile::tempdir().unwrap();
    assert_eq!(
        git_c(&ev("cd ../x && git status"), &entorno(&d), false),
        Decision::Allow
    );
    assert_eq!(reflexes(&d), ["git-c"]);
    assert_eq!(
        git_c(&ev("cd x/.. && git status"), &entorno(&d), false),
        Decision::Allow
    );
    assert_eq!(
        rewrite_cmd(&git_c(&ev("cd x && git status"), &entorno(&d), false)).as_deref(),
        Some("git -C x status")
    );
}

#[test]
fn git_c_prefiltro_sin_git_no_loguea() {
    let d = tempfile::tempdir().unwrap();
    // El comando tiene el patrón de 1b salvo la palabra git: sin "git" en crudo no se mira.
    let e = evento_crudo("Bash", "cd a && ls", Some(json!({"cwd": "/x"}).to_string()));
    assert_eq!(git_c(&e, &entorno(&d), false), Decision::Allow);
    assert!(!d.path().join("log.jsonl").exists());
    // El crudo lleva "git" escapado: el pre-filtro del .sh lo descarta aunque el command ya parseado lo tenga.
    let e = evento_crudo(
        "Bash",
        "cd a && git commit",
        Some(r#"{"tool_input":{"command":"cd a && \u0067it commit"}}"#.into()),
    );
    assert_eq!(git_c(&e, &entorno(&d), false), Decision::Allow);
    assert!(!d.path().join("log.jsonl").exists());
    // "git" solo en otro campo del crudo: pasa el pre-filtro y, sin match, no loguea.
    let e = evento_crudo("Bash", "ls", Some(json!({"cwd": "/git/x"}).to_string()));
    assert_eq!(git_c(&e, &entorno(&d), false), Decision::Allow);
    assert!(log(&d).is_empty());
    // "git" en crudo con match de 1b sin rewrite: sí loguea.
    assert_eq!(
        git_c(&ev("cd a; git commit"), &entorno(&d), false),
        Decision::Allow
    );
    assert_eq!(reflexes(&d), ["git-c"]);
}

#[test]
fn git_c_multilinea_evalua_por_linea_como_grep() {
    let d = tempfile::tempdir().unwrap();
    assert_eq!(
        git_c(&ev("cd /r &&\ngit status"), &entorno(&d), false),
        Decision::Allow
    );
    assert!(log(&d).is_empty());
}

#[test]
fn git_c_solo_shell_y_punto_punto_exacto() {
    let d = tempfile::tempdir().unwrap();
    let mut e = ev("cd a && git status");
    e.tool.as_mut().unwrap().clase = Clase::Other;
    assert_eq!(git_c(&e, &entorno(&d), false), Decision::Allow);
    assert!(log(&d).is_empty());
    assert_eq!(
        git_c(&ev("cd .. && git status"), &entorno(&d), false),
        Decision::Allow
    );
    let l = log(&d);
    assert_eq!(l.len(), 1);
    assert_eq!(l[0]["reflex"], "git-c");
    assert_eq!(l[0]["payload"], "cd .. && git status");
}

#[test]
fn git_c_cada_metacaracter_bloquea_el_rewrite() {
    for m in ['&', ';', '|', '<', '>', '`', '$', '(', ')', '*', '?', '['] {
        let d = tempfile::tempdir().unwrap();
        let c = format!("cd r && git log a{m}b");
        assert_eq!(git_c(&ev(&c), &entorno(&d), false), Decision::Allow, "{m}");
        assert_eq!(reflexes(&d), ["git-c"], "{m}");
    }
}
