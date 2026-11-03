// Fixtures capturados con Claude Code 2.1.317 (claude -p + hook de volcado), rutas normalizadas.
// Excepción: `agent_id`/`agent_type` no salen en la captura (sin subagente) y se prueban con un payload inline.
use exo::hook::claude::{parse, render};
use exo::hook::evento::{Clase, Decision, EventoKind};
use serde_json::{Value, json};

const PRE_BASH: &str = include_str!("fixtures/hook/claude/pre_tool_use_bash.json");
const PRE_EDIT: &str = include_str!("fixtures/hook/claude/pre_tool_use_edit.json");
const POST_BASH: &str = include_str!("fixtures/hook/claude/post_tool_use_bash.json");
const POST_FAIL: &str = include_str!("fixtures/hook/claude/post_tool_use_failure_bash.json");
const PROBE: &str = include_str!("fixtures/hook/claude/claude-2.1.317-probe.jsonl");

fn json_de(s: &str) -> Value {
    serde_json::from_str(s).unwrap()
}

#[test]
fn parse_golden_pre_tool_use_bash() {
    let ev = parse("PreToolUse", PRE_BASH.as_bytes()).unwrap();
    assert_eq!(ev.kind, EventoKind::PreToolUse);
    let tool = ev.tool.as_ref().unwrap();
    assert_eq!(tool.clase, Clase::Shell);
    assert_eq!(tool.command.as_deref(), Some("exo search --help | head -1"));
    assert_eq!(ev.session_id, "562b11e3-c60d-4294-a45e-301b855415a9");
    assert_eq!(ev.cwd, "/tmp/probe");
    assert_eq!(ev.agent_id, "");
    assert_eq!(ev.agent_type, "");
    assert_eq!(ev.crudo, PRE_BASH);
}

#[test]
fn parse_agent_id_y_agent_type_de_subagente() {
    let stdin = json!({
        "session_id":"s","cwd":"/c","hook_event_name":"PreToolUse","tool_name":"Bash",
        "tool_input":{"command":"ls"},"agent_id":"agent-9","agent_type":"general-purpose"
    })
    .to_string();
    let ev = parse("PreToolUse", stdin.as_bytes()).unwrap();
    assert_eq!(ev.agent_id, "agent-9");
    assert_eq!(ev.agent_type, "general-purpose");
}

#[test]
fn parse_campos_ausentes_son_vacios_y_edit_es_fileedit() {
    let ev = parse("PreToolUse", PRE_EDIT.as_bytes()).unwrap();
    let tool = ev.tool.unwrap();
    assert_eq!(tool.clase, Clase::FileEdit);
    assert_eq!(tool.paths, vec!["/tmp/probe/x.txt".to_string()]);
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
        parse("Notification", b"{}").unwrap().kind,
        EventoKind::Otro("Notification".into())
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

#[test]
fn parse_captura_real_acepta_todas_las_lineas() {
    let lineas: Vec<&str> = PROBE.lines().collect();
    assert_eq!(lineas.len(), 10);
    for l in lineas {
        let nombre = json_de(l)["hook_event_name"].as_str().unwrap().to_string();
        let ev = parse(&nombre, l.as_bytes()).unwrap();
        assert_eq!(ev.session_id, "562b11e3-c60d-4294-a45e-301b855415a9");
        assert!(ev.tool.is_some(), "{nombre}");
    }
}

// Premisa de spec §4.2: un comando que falla llega como PostToolUseFailure (sin tool_response).
#[test]
fn premisa_4_2_false_real_es_post_tool_use_failure_shell() {
    let l = PROBE
        .lines()
        .find(|l| l.contains("PostToolUseFailure") && l.contains(r#""command":"false""#))
        .unwrap();
    let ev = parse("PostToolUseFailure", l.as_bytes()).unwrap();
    assert_eq!(ev.kind, EventoKind::PostToolUseFailure);
    let tool = ev.tool.unwrap();
    assert_eq!(tool.clase, Clase::Shell);
    assert_eq!(tool.command.as_deref(), Some("false"));
}

#[test]
fn parse_session_start_source() {
    let ev = parse(
        "SessionStart",
        br#"{"hook_event_name":"SessionStart","session_id":"s","cwd":"/x","source":"compact"}"#,
    )
    .unwrap();
    assert_eq!(ev.kind, EventoKind::SessionStart);
    assert_eq!(ev.source, "compact");
    assert_eq!(ev.prompt, "");
}

#[test]
fn source_desconocido() {
    let ev = parse("SessionStart", br#"{"session_id":"s","source":"fork"}"#).unwrap();
    assert_eq!(ev.kind, EventoKind::SessionStart);
    assert_eq!(ev.source, "fork");
}

#[test]
fn parse_kinds_nuevos_y_prompt() {
    for (n, k) in [
        ("Stop", EventoKind::Stop),
        ("PostCompact", EventoKind::PostCompact),
        ("UserPromptSubmit", EventoKind::UserPromptSubmit),
    ] {
        let ev = parse(n, br#"{"prompt":"hola","session_id":"s"}"#).unwrap();
        assert_eq!(ev.kind, k);
        assert_eq!(ev.prompt, "hola");
        assert_eq!(ev.source, "");
    }
}

#[test]
fn render_avisa() {
    let ev = parse("SessionStart", br#"{"session_id":"s"}"#).unwrap();
    assert_eq!(
        json_de(&render(&Decision::Avisa("⚠ x".into()), &ev)),
        json!({"systemMessage":"⚠ x"})
    );
}

#[test]
fn kinds_nuevos_no_rompen_pre() {
    let ev = parse("PreToolUse", PRE_BASH.as_bytes()).unwrap();
    assert_eq!(ev.kind, EventoKind::PreToolUse);
    assert_eq!(ev.source, "");
    assert_eq!(ev.prompt, "");
    assert!(ev.tool.is_some());
}
