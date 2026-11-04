use exo::hook::evento::{Decision, Entorno, Evento, EventoKind, Harness, Tool, clasifica};
use exo::hook::guards::search_first::search_first;
use exo::hook::journal::{registra_tool, ruta};
use exo::hook::reflex::lee_cola;
use serde_json::json;

const AVISO: &str = "Reflejo search-first: primer trabajo sustantivo de la sesión sin `exo search`/`exo targets` previo. Si el tema puede tener historia en la KB, busca antes (`exo search --type hybrid \"<tema>\"`); si no, dilo en una línea y sigue.";

fn entorno(dir: &std::path::Path) -> Entorno {
    Entorno {
        state_dir: dir.to_path_buf(),
        reflex_log: dir.join("reflex.jsonl"),
        rutas: exo::state::Rutas::resuelve(std::path::Path::new("."), |_| None),
        seams: exo::state::Seams::resuelve(|_| None),
        cdpath_definido: false,
        panic_guard: None,
    }
}

fn evento(agent: &str, nombre: &str, input: serde_json::Value) -> Evento {
    let (clase, command, paths) = clasifica(nombre, &input);
    Evento {
        harness: Harness::Claude,
        kind: EventoKind::PreToolUse,
        session_id: "s".into(),
        cwd: "/x".into(),
        agent_id: agent.into(),
        agent_type: String::new(),
        source: String::new(),
        prompt: String::new(),
        crudo: String::new(),
        tool: Some(Tool {
            nombre: nombre.into(),
            clase,
            input,
            command,
            paths,
        }),
    }
}

fn edit(agent: &str) -> Evento {
    evento(agent, "Edit", json!({}))
}

fn bash(cmd: &str) -> Evento {
    evento("", "Bash", json!({ "command": cmd }))
}

fn log(en: &Entorno) -> Vec<(String, String)> {
    lee_cola(en, 50)
        .iter()
        .map(|v| {
            (
                v["reflex"].as_str().unwrap().to_string(),
                v["payload"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

#[test]
fn search_first_edit_sin_path_tambien_avisa() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path());
    assert!(matches!(search_first(&edit(""), &en), Decision::Steer(_)));
}

#[test]
fn search_first_avisa_sin_busqueda() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path());
    assert_eq!(search_first(&edit(""), &en), Decision::Steer(AVISO.into()));
    assert_eq!(log(&en), vec![("search-first".into(), String::new())]);
}

#[test]
fn search_first_ok_con_busqueda_previa() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path());
    registra_tool(&en, &bash("exo search --type hybrid x"), Some(true)).unwrap();
    assert_eq!(search_first(&edit(""), &en), Decision::Allow);
    assert_eq!(log(&en), vec![("search-first-ok".into(), String::new())]);
}

#[test]
fn search_first_una_vez_por_sesion() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path());
    assert!(matches!(search_first(&edit(""), &en), Decision::Steer(_)));
    assert_eq!(search_first(&edit(""), &en), Decision::Allow);
    assert_eq!(log(&en).len(), 1);
}

#[test]
fn search_first_en_subagente_calla() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path());
    assert_eq!(search_first(&edit("sub1"), &en), Decision::Allow);
    assert!(log(&en).is_empty());
    assert!(!ruta(&en, Harness::Claude, "s").exists());
    // El subagente no consume el aviso del padre.
    assert!(matches!(search_first(&edit(""), &en), Decision::Steer(_)));
}

#[test]
fn search_first_journal_ilegible() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path());
    // Un directorio donde va el journal: `lee` da Err y `registra_warn` también.
    std::fs::create_dir_all(ruta(&en, Harness::Claude, "s")).unwrap();
    assert_eq!(search_first(&edit(""), &en), Decision::Steer(AVISO.into()));
    assert_eq!(
        log(&en),
        vec![("search-first".into(), "motivo=journal-ilegible".into())]
    );
    // Sin journal legible no hay forma de ver el warn: la 2ª llamada vuelve a avisar.
    assert!(matches!(search_first(&edit(""), &en), Decision::Steer(_)));
}

#[test]
fn search_first_no_aplica_a_bash() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path());
    assert_eq!(search_first(&bash("ls"), &en), Decision::Allow);
    assert!(log(&en).is_empty());
    assert!(!ruta(&en, Harness::Claude, "s").exists());
    assert!(matches!(search_first(&edit(""), &en), Decision::Steer(_)));
}

#[test]
fn search_first_regex_literal() {
    let casa = |cmd: &str| {
        let d = tempfile::tempdir().unwrap();
        let en = entorno(d.path());
        registra_tool(&en, &bash(cmd), Some(true)).unwrap();
        search_first(&edit(""), &en) == Decision::Allow
    };
    assert!(casa("exo.exe search x"));
    assert!(casa("/usr/bin/exo targets"));
    assert!(casa("echo exo search"));
    assert!(!casa("exo searchx"));
    assert!(!casa("noexo search"));
}

#[test]
fn search_first_solo_cuenta_bash_tool() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path());
    let ps = evento("", "PowerShell", json!({ "command": "exo search x" }));
    registra_tool(&en, &ps, Some(true)).unwrap();
    exo::hook::journal::registra_warn(&en, &bash("exo search x"), "otro").unwrap();
    assert!(matches!(search_first(&edit(""), &en), Decision::Steer(_)));
}
