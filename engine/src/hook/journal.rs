//! Journal de eventos del hook: un JSONL por sesión bajo `<state_dir>/sessions/`.
//!
//! Cada registro es una única `write()` sobre un fichero `O_APPEND`: así varios
//! procesos del hook (uno por evento) no entrelazan líneas. No hay lock.

use super::evento::{Entorno, Evento, Harness};
use crate::obsolescencia::formatea_rfc3339_utc;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const CMD_MAX_BYTES: usize = 200;
const LECTURA_MAX_BYTES: u64 = 2 * 1024 * 1024;
const GC_EDAD: Duration = Duration::from_secs(14 * 86_400);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Registro {
    pub ts: String,
    /// `"tool"`, `"warn"`, `"event"` o `"mark"`.
    pub kind: String,
    pub tool: String,
    pub clase: String,
    pub cmd: String,
    pub paths: Vec<String>,
    pub ok: Option<bool>,
    pub agent_id: String,
    pub code: String,
}

pub struct Lectura {
    pub registros: Vec<Registro>,
    pub truncado: bool,
}

pub fn ruta(en: &Entorno, h: Harness, session_id: &str) -> PathBuf {
    // El session_id viene del harness: sin sanitizar permitiría `../`.
    let limpio: String = session_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    en.state_dir
        .join("sessions")
        .join(format!("{}-{limpio}.jsonl", h.nombre()))
}

pub fn registra_tool(en: &Entorno, ev: &Evento, ok: Option<bool>) -> std::io::Result<()> {
    escribe(en, ev, "tool", ok, "")
}

pub fn registra_warn(en: &Entorno, ev: &Evento, code: &str) -> std::io::Result<()> {
    escribe(en, ev, "warn", None, code)
}

/// Registra un evento sin herramienta (`UserPromptSubmit`, `PostCompact`); `code` lleva el nombre del evento.
pub fn registra_evento(en: &Entorno, ev: &Evento) -> std::io::Result<()> {
    escribe(en, ev, "event", None, ev.kind.nombre())
}

/// Marca de sesión (`post-compact`, `post-compact-consumed`).
pub fn registra_marca(en: &Entorno, ev: &Evento, code: &str) -> std::io::Result<()> {
    escribe(en, ev, "mark", None, code)
}

/// Hay un `post-compact` posterior al último `post-compact-consumed` del mismo `agent_id`.
pub fn marca_pendiente(en: &Entorno, ev: &Evento) -> bool {
    let Ok(Some(l)) = lee(en, ev.harness, &ev.session_id, &ev.agent_id) else {
        return false;
    };
    l.registros
        .iter()
        .rev()
        .find(|r| {
            r.kind == "mark" && (r.code == "post-compact" || r.code == "post-compact-consumed")
        })
        .is_some_and(|r| r.code == "post-compact")
}

fn escribe(
    en: &Entorno,
    ev: &Evento,
    kind: &str,
    ok: Option<bool>,
    code: &str,
) -> std::io::Result<()> {
    let epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    let reg = Registro {
        ts: formatea_rfc3339_utc(epoch),
        kind: kind.into(),
        tool: ev
            .tool
            .as_ref()
            .map(|t| t.nombre.clone())
            .unwrap_or_default(),
        clase: ev
            .tool
            .as_ref()
            .map(|t| t.clase.nombre().to_string())
            .unwrap_or_default(),
        cmd: ev
            .tool
            .as_ref()
            .and_then(|t| t.command.as_deref())
            .map(trunca_utf8)
            .unwrap_or_default(),
        paths: ev
            .tool
            .as_ref()
            .map(|t| t.paths.clone())
            .unwrap_or_default(),
        ok,
        agent_id: ev.agent_id.clone(),
        code: code.into(),
    };
    let mut linea = serde_json::to_vec(&reg).map_err(std::io::Error::other)?;
    linea.push(b'\n');

    let p = ruta(en, ev.harness, &ev.session_id);
    let dir = p.parent().expect("ruta() siempre cuelga de sessions/");
    fs::create_dir_all(dir)?;
    // El GC va solo al crear la sesión: el coste no se paga en cada evento.
    if !p.exists() {
        gc(dir);
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&p)?
        .write_all(&linea)
}

fn trunca_utf8(s: &str) -> String {
    let mut fin = s.len().min(CMD_MAX_BYTES);
    while !s.is_char_boundary(fin) {
        fin -= 1;
    }
    s[..fin].to_string()
}

fn gc(dir: &std::path::Path) {
    let Ok(entradas) = fs::read_dir(dir) else {
        return;
    };
    let ahora = SystemTime::now();
    for e in entradas.flatten() {
        let vieja = e
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| ahora.duration_since(t).ok())
            .is_some_and(|edad| edad > GC_EDAD);
        if vieja {
            let _ = fs::remove_file(e.path());
        }
    }
}

pub fn lee(
    en: &Entorno,
    h: Harness,
    session_id: &str,
    agent_id: &str,
) -> std::io::Result<Option<Lectura>> {
    let mut f = match File::open(ruta(en, h, session_id)) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let len = f.metadata()?.len();
    let truncado = len > LECTURA_MAX_BYTES;
    if truncado {
        f.seek(SeekFrom::Start(len - LECTURA_MAX_BYTES))?;
    }
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    // La primera línea tras el corte queda partida y no parsea: se salta como cualquier corrupta.
    let registros = buf
        .split(|&b| b == b'\n')
        .filter_map(|l| serde_json::from_slice::<Registro>(l).ok())
        .filter(|r| r.agent_id == agent_id)
        .collect();
    Ok(Some(Lectura {
        registros,
        truncado,
    }))
}
