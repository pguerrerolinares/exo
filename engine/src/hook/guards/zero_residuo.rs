//! Guard zero-residuo: port de la rama 2 de `bash-guards.sh` (retirado en plugin 1.32.0) y del awk de
//! `_zero-residuo-clasifica.sh`.

use crate::hook::evento::{Clase, Decision, Entorno, Evento};
use crate::hook::reflex::{lee_cola, payload_truncado, registra};
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

const TEXTO_DENY: &str = "Reflejo zero-residuo: `git add -A`/`--all`/`.` sin pathspec arrastra residuo (ficheros ajenos o a medias). Usa rutas explícitas: `git add <ruta> ...`.";
const VENTANA_RACHA: usize = 500;
const UMBRAL_RACHA: usize = 3;

// grep -E casa por línea: el espacio no cruza un `\n` (`[^\S\n]`) y `$` es fin de línea (`(?m)`).
static PATRON_ZR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)git[^\S\n]+(-C[^\S\n]+\S+[^\S\n]+)?add[^\S\n]+(-A|--all|\.)([^\S\n]|$)")
        .unwrap()
});
static HEREDOC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<<-?[ \t]*['"]?[A-Za-z_][A-Za-z0-9_]*['"]?"#).unwrap());
static SEPARADORES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"&&|\|\||[;|&]").unwrap());
static ESPACIOS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[ \t]+").unwrap());
static VAR_INICIO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\$\{?[A-Za-z_][A-Za-z0-9_]*\}?").unwrap());
static ASIGNACION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*=").unwrap());
static APERTURA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[({]+$").unwrap());
static REDIR_SOLA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9]*[<>]+$").unwrap());
static REDIR_PEGADA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9]*[<>]").unwrap());
static CIERRE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[})]+$").unwrap());

fn quita_comillas(x: &str) -> String {
    x.chars().filter(|c| *c != '"' && *c != '\'').collect()
}

fn expande(x: &str, vars: &HashMap<String, String>) -> String {
    let x = quita_comillas(x);
    if let Some(m) = VAR_INICIO.find(&x) {
        let nombre: String = m.as_str().chars().filter(|c| !"${}".contains(*c)).collect();
        if let Some(v) = vars.get(&nombre) {
            return format!("{v}{}", &x[m.end()..]);
        }
    }
    x
}

/// Directorio efectivo de cada `git [-C DIR] add (-A|--all|.)` sin pathspec en
/// posición de comando; vacío = ninguno (usa el cwd). Misma heurística que el awk.
pub fn deny_dirs(cmd: &str) -> Vec<String> {
    let mut salida = vec![];
    let mut vars: HashMap<String, String> = HashMap::new();
    let mut cur = String::new();
    let mut term = String::new();
    for linea in cmd.split('\n') {
        if !term.is_empty() {
            if linea.trim_matches([' ', '\t']) == term {
                term.clear();
            }
            continue;
        }
        if let Some(m) = HEREDOC.find(linea)
            && !(m.start() > 0 && linea.as_bytes()[m.start() - 1] == b'<')
        {
            let w = m.as_str().trim_start_matches('<').trim_start_matches('-');
            term = quita_comillas(w.trim_start_matches([' ', '\t']));
        }
        for seg in SEPARADORES.replace_all(linea, "\n").split('\n') {
            segmento(seg, &mut vars, &mut cur, &mut salida);
        }
    }
    salida
}

fn segmento(
    s: &str,
    vars: &mut HashMap<String, String>,
    cur: &mut String,
    salida: &mut Vec<String>,
) {
    let t: Vec<&str> = ESPACIOS.split(s).collect();
    let g = |i: usize| t.get(i).copied().unwrap_or("");
    let mut i = 0;
    while i < t.len() && (t[i].is_empty() || APERTURA.is_match(t[i]) || ASIGNACION.is_match(t[i])) {
        if ASIGNACION.is_match(t[i]) {
            let (nombre, v) = t[i].split_once('=').unwrap();
            vars.insert(nombre.to_string(), quita_comillas(v));
        }
        i += 1;
    }
    if g(i) == "cd" {
        let d = expande(g(i + 1), vars);
        *cur = if d.starts_with('/') || cur.is_empty() {
            d
        } else {
            format!("{cur}/{d}")
        };
        return;
    }
    if g(i) != "git" {
        return;
    }
    i += 1;
    let mut d = String::new();
    if g(i) == "-C" {
        d = expande(g(i + 1), vars);
        i += 2;
    }
    if g(i) != "add" || !matches!(g(i + 1), "-A" | "--all" | ".") {
        return;
    }
    let mut salta = false;
    for tok in t.iter().skip(i + 2) {
        if tok.is_empty() || tok.starts_with('#') {
            break;
        }
        if salta {
            salta = false;
            continue;
        }
        if REDIR_SOLA.is_match(tok) {
            salta = true;
            continue;
        }
        if REDIR_PEGADA.is_match(tok) || CIERRE.is_match(tok) {
            continue;
        }
        if tok.starts_with('-') && *tok != "--" {
            continue;
        }
        return;
    }
    salida.push(if d.is_empty() { cur.clone() } else { d });
}

/// Racha de entradas consecutivas (desde el final) distintas de `zero-residuo`
/// del mismo session_id y agent_id.
fn racha(en: &Entorno, ev: &Evento) -> usize {
    let cola = lee_cola(en, VENTANA_RACHA);
    let propias: Vec<&str> = cola
        .iter()
        .filter(|v| {
            v["session_id"].as_str() == Some(ev.session_id.as_str())
                && v["agent_id"].as_str() == Some(ev.agent_id.as_str())
        })
        .filter_map(|v| v["reflex"].as_str())
        .filter(|r| {
            matches!(
                *r,
                "zero-residuo" | "zero-residuo-deny" | "zero-residuo-backoff"
            )
        })
        .collect();
    propias
        .iter()
        .rev()
        .take_while(|r| **r != "zero-residuo")
        .count()
}

pub fn zero_residuo(ev: &Evento, en: &Entorno) -> Decision {
    let Some(tool) = ev.tool.as_ref().filter(|t| t.clase == Clase::Shell) else {
        return Decision::Allow;
    };
    let Some(cmd) = tool.command.as_deref().filter(|c| !c.is_empty()) else {
        return Decision::Allow;
    };
    if !ev.crudo.contains("git") || !PATRON_ZR.is_match(cmd) {
        return Decision::Allow;
    }
    let fuera_de_tmp = deny_dirs(cmd).iter().any(|d| {
        let d = if d.starts_with(['$', '~']) {
            ""
        } else {
            d.as_str()
        };
        let efectivo = if d.is_empty() { ev.cwd.as_str() } else { d };
        efectivo != "/tmp" && !efectivo.starts_with("/tmp/")
    });
    let mut id = "zero-residuo";
    let mut decision = Decision::Allow;
    if fuera_de_tmp {
        if racha(en, ev) >= UMBRAL_RACHA {
            id = "zero-residuo-backoff";
        } else {
            id = "zero-residuo-deny";
            decision = Decision::Deny(TEXTO_DENY.into());
        }
    }
    registra(en, id, ev, &payload_truncado(cmd, &PATRON_ZR));
    decision
}
