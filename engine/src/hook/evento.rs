//! Tipos neutrales del hook: evento, herramienta, decisión y entorno.

use crate::state::{Rutas, Seams};
use serde_json::Value;
use std::path::PathBuf;

/// `desde_env` es la única función de `hook/` que lee el entorno; el resto recibe un `Entorno`.
/// `state_dir`/`reflex_log` son alias de `rutas.*` (los guards ya los leen así).
#[derive(Debug, Clone)]
pub struct Entorno {
    pub state_dir: PathBuf,
    pub reflex_log: PathBuf,
    pub rutas: Rutas,
    pub seams: Seams,
    pub cdpath_definido: bool,
    pub panic_guard: Option<String>,
}

impl Entorno {
    pub fn desde_env() -> Entorno {
        let rutas = Rutas::desde_env();
        Entorno {
            state_dir: rutas.state_dir.clone(),
            reflex_log: rutas.reflex_log.clone(),
            rutas,
            seams: Seams::desde_env(),
            cdpath_definido: std::env::var_os("CDPATH").is_some_and(|v| !v.is_empty()),
            // Solo en debug: en release un env no puede inyectar panics en los guards.
            #[cfg(debug_assertions)]
            panic_guard: std::env::var("EXO_HOOK_PANIC").ok(),
            #[cfg(not(debug_assertions))]
            panic_guard: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Harness {
    Claude,
}

impl Harness {
    pub fn nombre(&self) -> &'static str {
        match self {
            Harness::Claude => "claude",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventoKind {
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    Otro(String),
}

impl EventoKind {
    pub fn nombre(&self) -> &str {
        match self {
            EventoKind::PreToolUse => "PreToolUse",
            EventoKind::PostToolUse => "PostToolUse",
            EventoKind::PostToolUseFailure => "PostToolUseFailure",
            EventoKind::Otro(s) => s,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clase {
    Shell,
    FileEdit,
    Agent,
    Web,
    Mcp,
    Other,
}

impl Clase {
    /// Forma serializada en el journal; `verify` la compara por este nombre.
    pub fn nombre(&self) -> &'static str {
        match self {
            Clase::Shell => "Shell",
            Clase::FileEdit => "FileEdit",
            Clase::Agent => "Agent",
            Clase::Web => "Web",
            Clase::Mcp => "Mcp",
            Clase::Other => "Other",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tool {
    pub nombre: String,
    pub clase: Clase,
    pub input: Value,
    pub command: Option<String>,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Evento {
    pub harness: Harness,
    pub kind: EventoKind,
    pub session_id: String,
    pub cwd: String,
    pub agent_id: String,
    pub agent_type: String,
    /// stdin tal cual: el pre-filtro de git lo necesita sin parsear.
    pub crudo: String,
    pub tool: Option<Tool>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    Allow,
    Deny(String),
    Rewrite { input: Value, razon: String },
    Steer(String),
    Block(String),
}

pub fn clasifica(nombre: &str, input: &Value) -> (Clase, Option<String>, Vec<String>) {
    let campo = |k: &str| input.get(k).and_then(Value::as_str).map(str::to_string);
    match nombre {
        "Bash" | "PowerShell" => (Clase::Shell, campo("command"), vec![]),
        "Edit" | "Write" | "NotebookEdit" => {
            let clave = if nombre == "NotebookEdit" {
                "notebook_path"
            } else {
                "file_path"
            };
            match campo(clave) {
                Some(p) => (Clase::FileEdit, None, vec![p]),
                // Sin ruta no hay nada que vigilar: se abstiene.
                None => (Clase::Other, None, vec![]),
            }
        }
        "Agent" | "Task" => (Clase::Agent, None, vec![]),
        "WebFetch" | "WebSearch" => (Clase::Web, None, vec![]),
        n if n.starts_with("mcp__") => (Clase::Mcp, None, vec![]),
        _ => (Clase::Other, None, vec![]),
    }
}

/// Precedencia Block > Deny > Rewrite > Steer > Allow; las razones del mismo
/// rango se concatenan con línea en blanco.
pub fn combinar(ds: Vec<Decision>) -> Decision {
    let mut block = vec![];
    let mut deny = vec![];
    let mut steer = vec![];
    let mut rewrite = None;
    for d in ds {
        match d {
            Decision::Block(r) => block.push(r),
            Decision::Deny(r) => deny.push(r),
            Decision::Steer(r) => steer.push(r),
            Decision::Rewrite { .. } if rewrite.is_none() => rewrite = Some(d),
            Decision::Rewrite { .. } | Decision::Allow => {}
        }
    }
    if !block.is_empty() {
        Decision::Block(block.join("\n\n"))
    } else if !deny.is_empty() {
        Decision::Deny(deny.join("\n\n"))
    } else if let Some(r) = rewrite {
        r
    } else if !steer.is_empty() {
        Decision::Steer(steer.join("\n\n"))
    } else {
        Decision::Allow
    }
}
