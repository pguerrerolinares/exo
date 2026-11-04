//! Adapter del harness Claude Code: stdin JSON -> `Evento`, `Decision` -> stdout JSON.

use super::evento::{Decision, Evento, EventoKind, Harness, Tool, clasifica};
use serde_json::{Value, json};

pub fn parse(evento: &str, stdin: &[u8]) -> anyhow::Result<Evento> {
    let v: Value = serde_json::from_slice(stdin)?;
    let campo = |k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let tool = v.get("tool_name").and_then(Value::as_str).map(|nombre| {
        let input = v.get("tool_input").cloned().unwrap_or(Value::Null);
        let (clase, command, paths) = clasifica(nombre, &input);
        Tool {
            nombre: nombre.to_string(),
            clase,
            input,
            command,
            paths,
        }
    });
    Ok(Evento {
        harness: Harness::Claude,
        kind: kind(evento),
        session_id: campo("session_id"),
        cwd: campo("cwd"),
        agent_id: campo("agent_id"),
        agent_type: campo("agent_type"),
        crudo: String::from_utf8_lossy(stdin).into_owned(),
        tool,
        source: campo("source"),
        prompt: campo("prompt"),
    })
}

fn kind(evento: &str) -> EventoKind {
    match evento {
        "PreToolUse" => EventoKind::PreToolUse,
        "PostToolUse" => EventoKind::PostToolUse,
        "PostToolUseFailure" => EventoKind::PostToolUseFailure,
        "SessionStart" => EventoKind::SessionStart,
        "Stop" => EventoKind::Stop,
        "UserPromptSubmit" => EventoKind::UserPromptSubmit,
        "PostCompact" => EventoKind::PostCompact,
        otro => EventoKind::Otro(otro.to_string()),
    }
}

pub fn render(d: &Decision, ev: &Evento) -> String {
    let nombre = ev.kind.nombre();
    match d {
        Decision::Allow => String::new(),
        Decision::Deny(r) => json!({"hookSpecificOutput":{
            "hookEventName": nombre,
            "permissionDecision": "deny",
            "permissionDecisionReason": r,
        }})
        .to_string(),
        Decision::Rewrite { input, razon } => {
            let mut actualizado = ev
                .tool
                .as_ref()
                .and_then(|t| t.input.as_object().cloned())
                .unwrap_or_default();
            if let Some(cambios) = input.as_object() {
                actualizado.extend(cambios.clone());
            }
            json!({"hookSpecificOutput":{
                "hookEventName": nombre,
                "permissionDecision": "allow",
                "permissionDecisionReason": razon,
                "updatedInput": actualizado,
            }})
            .to_string()
        }
        Decision::Steer(m) => json!({"hookSpecificOutput":{
            "hookEventName": nombre,
            "additionalContext": m,
        }})
        .to_string(),
        Decision::Avisa(m) => json!({"systemMessage": m}).to_string(),
        Decision::Block(r) => json!({"decision":"block","reason":r}).to_string(),
    }
}
