//! Reflex-log: espejo en Rust de `_reflex-log.sh` y `_truncate-payload.sh` (este retirado en plugin 1.32.0).

use super::evento::{Entorno, Evento};
use crate::obsolescencia::formatea_rfc3339_utc;
use regex::Regex;
use serde_json::{Value, json};
use std::io::Write;

const PAYLOAD_MAX: usize = 2000;
const PREFIJO: usize = 120;
const MATCH_HEAD: usize = 80;
const MATCH_TAIL: usize = 60;
const MATCH_MAX: usize = 5;

fn prefijo(s: &str, n: usize) -> &str {
    s.char_indices().nth(n).map_or(s, |(i, _)| &s[..i])
}

/// Best-effort: un fallo de I/O se traga, nunca rompe el hook que instrumenta.
pub fn registra(en: &Entorno, reflex: &str, ev: &Evento, payload: &str) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| formatea_rfc3339_utc(d.as_secs() as i64))
        .unwrap_or_default();
    let linea = json!({
        "ts": ts,
        "reflex": reflex,
        "session_id": ev.session_id,
        "agent_id": ev.agent_id,
        "agent_type": ev.agent_type,
        "tool": ev.tool.as_ref().map_or("", |t| t.nombre.as_str()),
        "payload": prefijo(payload, PAYLOAD_MAX),
    });
    let _ = (|| -> std::io::Result<()> {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&en.reflex_log)?;
        // Una sola write(): con O_APPEND es atómica y las líneas de hooks concurrentes no se intercalan.
        let mut bytes = linea.to_string().into_bytes();
        bytes.push(b'\n');
        f.write_all(&bytes)
    })();
}

pub fn payload_truncado(cmd: &str, patron: &Regex) -> String {
    if cmd.chars().count() <= PREFIJO {
        return cmd.to_string();
    }
    let pre = prefijo(cmd, PREFIJO);
    let mut vistos: Vec<String> = Vec::new();
    for m in patron
        .find_iter(cmd)
        .map(|m| m.as_str())
        .filter(|m| !m.is_empty())
        .take(MATCH_MAX)
    {
        let m = if m.chars().count() > MATCH_HEAD + MATCH_TAIL {
            let n = m.chars().count();
            let cola: String = m.chars().skip(n - MATCH_TAIL).collect();
            format!("{}…{cola}", prefijo(m, MATCH_HEAD))
        } else {
            m.to_string()
        };
        if !vistos.contains(&m) {
            vistos.push(m);
        }
    }
    if vistos.is_empty() {
        pre.to_string()
    } else {
        format!("{pre} … ⟨match⟩ {}", vistos.join(" | "))
    }
}

/// Últimas `n` líneas válidas del log; las ilegibles se saltan.
pub fn lee_cola(en: &Entorno, n: usize) -> Vec<Value> {
    let Ok(txt) = std::fs::read_to_string(&en.reflex_log) else {
        return vec![];
    };
    let mut v: Vec<Value> = txt
        .lines()
        .rev()
        .filter_map(|l| serde_json::from_str(l).ok())
        .take(n)
        .collect();
    v.reverse();
    v
}
