//! Adapter del harness Codex: stdin JSON -> `Evento`, `Decision` -> stdout JSON.

use super::evento::{Clase, Decision, Evento, EventoKind, Harness, Tool};
use serde_json::{Map, Value, json};

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
        harness: Harness::Codex,
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
        "SessionStart" => EventoKind::SessionStart,
        "Stop" => EventoKind::Stop,
        "UserPromptSubmit" => EventoKind::UserPromptSubmit,
        "PostCompact" => EventoKind::PostCompact,
        otro => EventoKind::Otro(otro.to_string()),
    }
}

pub fn clasifica(nombre: &str, input: &Value) -> (Clase, Option<String>, Vec<String>) {
    let comando = input.get("command").and_then(Value::as_str);
    match nombre {
        "Bash" => (Clase::Shell, comando.map(str::to_string), vec![]),
        "apply_patch" => {
            let paths = comando.map(rutas_patch).unwrap_or_default();
            // Sin rutas no hay nada que vigilar: se abstiene.
            let clase = if paths.is_empty() {
                Clase::Other
            } else {
                Clase::FileEdit
            };
            (clase, None, paths)
        }
        // `collaborationspawn_agent` en 0.166.0, `spawn_agent` en HEAD.
        n if n.ends_with("spawn_agent") => (Clase::Agent, None, vec![]),
        _ => (Clase::Other, None, vec![]),
    }
}

/// Rutas de las cabeceras `*** Add|Update|Delete File:` de un patch de `apply_patch`.
pub fn rutas_patch(patch: &str) -> Vec<String> {
    patch
        .lines()
        .filter_map(|l| {
            ["*** Add File: ", "*** Update File: ", "*** Delete File: "]
                .iter()
                .find_map(|p| l.strip_prefix(p))
        })
        .map(str::to_string)
        .collect()
}

/// Eventos cuyo schema de salida admite `hookSpecificOutput`.
fn admite_contexto(k: &EventoKind) -> bool {
    matches!(
        k,
        EventoKind::PreToolUse
            | EventoKind::PostToolUse
            | EventoKind::SessionStart
            | EventoKind::UserPromptSubmit
    )
}

/// Eventos cuyo schema de salida admite `decision: block`.
fn admite_block(k: &EventoKind) -> bool {
    matches!(
        k,
        EventoKind::PreToolUse
            | EventoKind::PostToolUse
            | EventoKind::UserPromptSubmit
            | EventoKind::Stop
    )
}

pub fn render(d: &Decision, ev: &Evento) -> String {
    let nombre = ev.kind.nombre();
    let contexto = |texto: &str| json!({"hookSpecificOutput":{"hookEventName": nombre, "additionalContext": texto}});
    let block = |r: &str| {
        if admite_block(&ev.kind) {
            json!({"decision":"block","reason":r})
        } else {
            json!({})
        }
    };
    let salida = match d {
        Decision::Allow => json!({}),
        Decision::Deny(r) if ev.kind == EventoKind::PreToolUse => json!({"hookSpecificOutput":{
            "hookEventName": nombre,
            "permissionDecision": "deny",
            "permissionDecisionReason": r,
        }}),
        Decision::Deny(r) | Decision::Block(r) => block(r),
        Decision::Rewrite { input, razon } if ev.kind == EventoKind::PreToolUse => {
            let mut actualizado: Map<String, Value> = ev
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
        }
        Decision::Rewrite { .. } => json!({}),
        Decision::Steer(m) if admite_contexto(&ev.kind) => contexto(m),
        Decision::Steer(_) => json!({}),
        // `codex exec` no pinta `systemMessage`: la línea va también al contexto.
        Decision::Avisa(m) if admite_contexto(&ev.kind) => {
            let mut o = contexto(m);
            o["systemMessage"] = json!(m);
            o
        }
        Decision::Avisa(m) => json!({"systemMessage": m}),
    };
    salida.to_string()
}
