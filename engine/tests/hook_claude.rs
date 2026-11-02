// Fixtures construidos, no capturados: el classifier del entorno denegó lanzar
// `claude -p` con hooks de volcado. Campos tomados de los payloads de las suites
// bash y de la documentación de hooks de Claude Code (2.1.316 instalado).
use exo::hook::claude::{parse, render};
use exo::hook::evento::{Clase, Decision, EventoKind};
use serde_json::{Value, json};

const PRE_BASH: &str = include_str!("fixtures/hook/claude/pre_tool_use_bash.json");
const PRE_EDIT: &str = include_str!("fixtures/hook/claude/pre_tool_use_edit.json");
const POST_BASH: &str = include_str!("fixtures/hook/claude/post_tool_use_bash.json");
const POST_FAIL: &str = include_str!("fixtures/hook/claude/post_tool_use_failure_bash.json");

fn json_de(s: &str) -> Value {
    serde_json::from_str(s).unwrap()
}

#[test]
fn parse_golden_pre_tool_use_bash() {
    let ev = parse("PreToolUse", PRE_BASH.as_bytes()).unwrap();
    assert_eq!(ev.kind, EventoKind::PreToolUse);
    let tool = ev.tool.as_ref().unwrap();
    assert_eq!(tool.clase, Clase::Shell);
    assert_eq!(tool.command.as_deref(), Some("cd repo && git status"));
    assert_eq!(ev.session_id, "sess-123");
    assert_eq!(ev.cwd, "/home/u/proj");
    assert_eq!(ev.agent_id, "agent-9");
    assert_eq!(ev.agent_type, "general-purpose");
    assert_eq!(ev.crudo, PRE_BASH);
}

#[test]
fn parse_campos_ausentes_son_vacios_y_edit_es_fileedit() {
    let ev = parse("PreToolUse", PRE_EDIT.as_bytes()).unwrap();
    assert_eq!(ev.agent_id, "");
    assert_eq!(ev.agent_type, "");
    let tool = ev.tool.unwrap();
    assert_eq!(tool.clase, Clase::FileEdit);
    assert_eq!(tool.paths, vec!["/home/u/proj/src/a.rs".to_string()]);
    let ev = parse("PreToolUse", b"{}").unwrap();
    assert_eq!(ev.session_id, "");
    assert!(ev.tool.is_none());
}

#[test]
fn parse_kinds_post() {
    assert_eq!(
        parse("PostToolUse", POST_BASH.as_bytes()).unwrap().kind,
        EventoKind::PostToolUse
    );
    assert_eq!(
        parse("PostToolUseFailure", POST_FAIL.as_bytes())
            .unwrap()
            .kind,
        EventoKind::PostToolUseFailure
    );
    assert_eq!(
        parse("Stop", b"{}").unwrap().kind,
        EventoKind::Otro("Stop".into())
    );
}

#[test]
fn parse_json_invalido_es_err() {
    assert!(parse("PreToolUse", PRE_BASH.as_bytes()).is_ok());
    assert!(parse("PreToolUse", b"no json").is_err());
}

#[test]
fn render_allow_es_vacio() {
    let ev = parse("PreToolUse", PRE_BASH.as_bytes()).unwrap();
    assert_eq!(render(&Decision::Allow, &ev), "");
}

#[test]
fn render_deny_pre_tool_use() {
    let ev = parse("PreToolUse", PRE_BASH.as_bytes()).unwrap();
    let out = render(&Decision::Deny("r".into()), &ev);
    assert_eq!(
        out,
        r#"{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"r"}}"#
    );
}

#[test]
fn render_rewrite_preserva_otros_campos_de_tool_input() {
    let stdin = json!({
        "session_id":"s","cwd":"/c","hook_event_name":"PreToolUse","tool_name":"Bash",
        "tool_input":{"command":"cd a && git status","description":"d","timeout":5000}
    })
    .to_string();
    let ev = parse("PreToolUse", stdin.as_bytes()).unwrap();
    let d = Decision::Rewrite {
        input: json!({"command":"git -C a status"}),
        razon: "porque".into(),
    };
    let v = json_de(&render(&d, &ev));
    let h = &v["hookSpecificOutput"];
    assert_eq!(h["hookEventName"], "PreToolUse");
    assert_eq!(h["permissionDecision"], "allow");
    assert_eq!(h["permissionDecisionReason"], "porque");
    assert_eq!(
        h["updatedInput"],
        json!({"command":"git -C a status","description":"d","timeout":5000})
    );
}

#[test]
fn render_steer_usa_el_evento() {
    let ev = parse("PreToolUse", PRE_BASH.as_bytes()).unwrap();
    assert_eq!(
        json_de(&render(&Decision::Steer("m".into()), &ev)),
        json!({"hookSpecificOutput":{"hookEventName":"PreToolUse","additionalContext":"m"}})
    );
    let ev = parse("PostToolUse", POST_BASH.as_bytes()).unwrap();
    let v = json_de(&render(&Decision::Steer("m".into()), &ev));
    assert_eq!(v["hookSpecificOutput"]["hookEventName"], "PostToolUse");
}

#[test]
fn render_block_es_decision_de_nivel_superior() {
    let ev = parse("PostToolUse", POST_BASH.as_bytes()).unwrap();
    assert_eq!(
        render(&Decision::Block("b".into()), &ev),
        r#"{"decision":"block","reason":"b"}"#
    );
}
