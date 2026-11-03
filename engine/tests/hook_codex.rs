//! Los schemas de `fixtures/hook/codex/schema/` son de openai/codex `6b2a275` (HEAD al
//! escribirse), no de 0.166.0 (spec §7.4): validan la forma que Codex acepta hoy, y el
//! golden de 0.166.0 solo prueba el parse.
use exo::hook::codex::{clasifica, parse, render, rutas_patch};
use exo::hook::evento::{Clase, Decision, Evento, EventoKind, Harness};
use jsonschema::Validator;
use serde_json::{Value, json};

const GOLDEN: &str = include_str!("fixtures/hook/codex/codex-0.166.0-probe.jsonl");

fn ev(kind: &str) -> Evento {
    let stdin = json!({"session_id":"s","cwd":"/c","hook_event_name":kind}).to_string();
    parse(kind, stdin.as_bytes()).unwrap()
}

fn schema(kind: &str) -> Validator {
    let f = match kind {
        "PreToolUse" => "pre-tool-use",
        "PostToolUse" => "post-tool-use",
        "SessionStart" => "session-start",
        "UserPromptSubmit" => "user-prompt-submit",
        "PostCompact" => "post-compact",
        "Stop" => "stop",
        otro => panic!("sin schema para {otro}"),
    };
    let ruta = format!(
        "{}/tests/fixtures/hook/codex/schema/{f}.command.output.schema.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let v: Value = serde_json::from_str(&std::fs::read_to_string(ruta).unwrap()).unwrap();
    jsonschema::validator_for(&v).unwrap()
}

const EVENTOS: [&str; 6] = [
    "PreToolUse",
    "PostToolUse",
    "SessionStart",
    "UserPromptSubmit",
    "PostCompact",
    "Stop",
];

fn decisiones() -> Vec<Decision> {
    vec![
        Decision::Allow,
        Decision::Deny("no".into()),
        Decision::Rewrite {
            input: json!({"command":"ls"}),
            razon: "r".into(),
        },
        Decision::Steer("s".into()),
        Decision::Avisa("a".into()),
        Decision::Block("b".into()),
    ]
}

fn salida(d: &Decision, kind: &str) -> Value {
    serde_json::from_str(&render(d, &ev(kind))).unwrap()
}

#[test]
fn golden_parsea_todo() {
    let mut n = 0;
    for l in GOLDEN.lines().filter(|l| !l.trim().is_empty()) {
        let v: Value = serde_json::from_str(l).unwrap();
        let nombre = v["hook_event_name"].as_str().unwrap();
        let e = parse(nombre, l.as_bytes()).unwrap_or_else(|e| panic!("{nombre}: {e}\n{l}"));
        assert_eq!(e.harness, Harness::Codex);
        assert_eq!(e.session_id, v["session_id"].as_str().unwrap());
        assert_eq!(e.crudo, l);
        n += 1;
    }
    assert_eq!(n, 51);
}

#[test]
fn golden_campos_claves() {
    let por = |nombre: &str, f: &dyn Fn(&Value) -> bool| -> Evento {
        let l = GOLDEN
            .lines()
            .find(|l| {
                let v: Value = serde_json::from_str(l).unwrap();
                v["hook_event_name"] == nombre && f(&v)
            })
            .unwrap();
        parse(nombre, l.as_bytes()).unwrap()
    };
    let e = por("SessionStart", &|v| v["source"] == "compact");
    assert_eq!(
        (e.kind, e.source.as_str()),
        (EventoKind::SessionStart, "compact")
    );
    let e = por("UserPromptSubmit", &|_| true);
    assert!(!e.prompt.is_empty());
    let e = por("PreToolUse", &|v| v["tool_name"] == "Bash");
    assert_eq!(e.tool.unwrap().clase, Clase::Shell);
    let e = por("PreToolUse", &|v| v["tool_name"] == "apply_patch");
    let t = e.tool.unwrap();
    assert_eq!(t.clase, Clase::FileEdit);
    assert_eq!(t.paths, vec!["/tmp/probe/x.txt".to_string()]);
    let e = por("PreToolUse", &|v| {
        v["tool_name"] == "collaborationspawn_agent"
    });
    assert_eq!(e.tool.unwrap().clase, Clase::Agent);
    let e = por("PreToolUse", &|v| v.get("agent_id").is_some());
    assert!(!e.agent_id.is_empty() && !e.agent_type.is_empty());
    assert_eq!(por("Stop", &|_| true).kind, EventoKind::Stop);
    assert_eq!(por("PostCompact", &|_| true).kind, EventoKind::PostCompact);
    assert_eq!(
        por("PreCompact", &|_| true).kind,
        EventoKind::Otro("PreCompact".into())
    );
}

#[test]
fn source_desconocido_no_es_error() {
    let e = parse("SessionStart", br#"{"source":"fork"}"#).unwrap();
    assert_eq!(e.source, "fork");
}

#[test]
fn apply_patch_rutas() {
    let p = "*** Begin Patch\n*** Add File: a.txt\n+x\n*** Update File: src/b.rs\n@@\n-a\n+b\n*** Delete File: c\n*** End Patch";
    assert_eq!(rutas_patch(p), vec!["a.txt", "src/b.rs", "c"]);
    let (clase, _, paths) = clasifica("apply_patch", &json!({"command": p}));
    assert_eq!(clase, Clase::FileEdit);
    assert_eq!(paths, vec!["a.txt", "src/b.rs", "c"]);
    let (clase, _, paths) = clasifica(
        "apply_patch",
        &json!({"command": "*** Begin Patch\n*** End Patch"}),
    );
    assert_eq!(clase, Clase::Other);
    assert!(paths.is_empty());
    let (clase, _, _) = clasifica("apply_patch", &json!({}));
    assert_eq!(clase, Clase::Other);
}

#[test]
fn patch_crlf() {
    let p = "*** Begin Patch\r\n*** Add File: a.txt\r\n+x\r\n*** Update File: src/b.rs\r\n*** End Patch\r\n";
    assert_eq!(rutas_patch(p), vec!["a.txt", "src/b.rs"]);
}

#[test]
fn spawn_por_sufijo() {
    let c = |n: &str| clasifica(n, &json!({})).0;
    assert_eq!(c("collaborationspawn_agent"), Clase::Agent);
    assert_eq!(c("spawn_agent"), Clase::Agent);
    assert_eq!(c("collaborationwait_agent"), Clase::Other);
    assert_eq!(c("collaborationlist_agents"), Clase::Other);
    assert_eq!(c("Bash"), Clase::Shell);
    let (_, cmd, _) = clasifica("Bash", &json!({"command":"ls -la"}));
    assert_eq!(cmd.as_deref(), Some("ls -la"));
}

#[test]
fn allow_es_objeto_vacio() {
    assert_eq!(render(&Decision::Allow, &ev("PreToolUse")), "{}");
    for k in EVENTOS {
        assert_eq!(salida(&Decision::Allow, k), json!({}), "{k}");
    }
}

#[test]
fn deny_y_rewrite_en_pre_tool_use() {
    let o = salida(&Decision::Deny("no".into()), "PreToolUse");
    assert_eq!(o["hookSpecificOutput"]["permissionDecision"], "deny");
    assert_eq!(o["hookSpecificOutput"]["permissionDecisionReason"], "no");
    let stdin = json!({"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"a","timeout":5}});
    let e = parse("PreToolUse", stdin.to_string().as_bytes()).unwrap();
    let d = Decision::Rewrite {
        input: json!({"command":"b"}),
        razon: "r".into(),
    };
    let o: Value = serde_json::from_str(&render(&d, &e)).unwrap();
    assert_eq!(o["hookSpecificOutput"]["permissionDecision"], "allow");
    assert_eq!(
        o["hookSpecificOutput"]["updatedInput"],
        json!({"command":"b","timeout":5})
    );
}

#[test]
fn avisa_doble() {
    let o = salida(&Decision::Avisa("⚠ x".into()), "SessionStart");
    assert_eq!(o["systemMessage"], "⚠ x");
    assert_eq!(o["hookSpecificOutput"]["hookEventName"], "SessionStart");
    assert!(
        o["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap()
            .lines()
            .any(|l| l == "⚠ x")
    );
}

#[test]
fn steer_va_a_additional_context() {
    for k in [
        "PreToolUse",
        "PostToolUse",
        "SessionStart",
        "UserPromptSubmit",
    ] {
        let o = salida(&Decision::Steer("s".into()), k);
        assert_eq!(o["hookSpecificOutput"]["additionalContext"], "s", "{k}");
        assert!(o.get("systemMessage").is_none(), "{k}");
    }
}

#[test]
fn post_compact_sin_contexto() {
    assert_eq!(
        salida(&Decision::Steer("s".into()), "PostCompact"),
        json!({})
    );
    assert_eq!(salida(&Decision::Steer("s".into()), "Stop"), json!({}));
    for k in ["PostCompact", "Stop"] {
        let o = salida(&Decision::Avisa("a".into()), k);
        assert_eq!(o, json!({"systemMessage":"a"}), "{k}");
    }
}

#[test]
fn schemas() {
    for k in EVENTOS {
        let s = schema(k);
        for d in decisiones() {
            let texto = render(&d, &ev(k));
            let v: Value = serde_json::from_str(&texto).unwrap();
            assert!(s.is_valid(&v), "{k} {d:?} -> {texto}");
        }
        assert!(
            !s.is_valid(&json!({"campoExtra": 1})),
            "{k} acepta campo extra"
        );
    }
    let s = schema("PreToolUse");
    assert!(!s.is_valid(&json!({"hookSpecificOutput":{"hookEventName":"PreToolUse","extra":1}})));
}
