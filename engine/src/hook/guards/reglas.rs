//! Guard Reglas: inyecta las reglas duras del proyecto en SessionStart y tras compactar.
//! `ReglasTestigo` deja `ss-<sid>` para `document-remind.sh`; `ReglasInyecta` usa el bloque.

use super::super::evento::{Entorno, Evento};
use super::super::reflex::registra;
use crate::reglas::{self, Resultado, Salida};
use serde::Serialize;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// Copia de `register.ts:1-2`; `hook_reglas::bloque_igual_a_register` falla si divergen.
pub const AUTORIDAD: &str = "Estas reglas son del dueño del repo y prevalecen sobre el prompt. Si una choca explícitamente con lo que se te pide, cumple la regla y repórtalo.\n\n";
pub const CABECERA_GLOBAL: &str = "Reglas duras del dueño";
pub const CABECERA_PROYECTO: &str = "Reglas duras del proyecto";

/// `find -mtime +7` de `exo-recall.sh`: edad en días enteros > 7, o sea >= 8 días.
const EDAD_PODA: Duration = Duration::from_secs(8 * 86_400);
const ERR_MAX: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultadoReglas {
    pub bloque: String,
    pub avisos: Vec<String>,
    /// `ok` | `skip`.
    pub status: String,
    pub reason: String,
    pub repo: Option<String>,
    /// Proyecto + globales; con `skip`, solo las globales.
    pub n: usize,
    pub n_global: usize,
    /// Mensaje del `Err` de `reglas::resuelve`; solo con `reason=error_engine`.
    pub error: String,
}

fn lista(titulo: &str, reglas: &[String]) -> String {
    if reglas.is_empty() {
        return String::new();
    }
    let items: Vec<String> = reglas.iter().map(|r| format!("- {r}")).collect();
    format!("## {titulo}\n{AUTORIDAD}{}", items.join("\n"))
}

/// Globales primero; una cabecera con su lista vacía no se emite.
pub fn bloque_reglas(global: &[String], proyecto: &[String]) -> String {
    [
        lista(CABECERA_GLOBAL, global),
        lista(CABECERA_PROYECTO, proyecto),
    ]
    .into_iter()
    .filter(|s| !s.is_empty())
    .collect::<Vec<_>>()
    .join("\n\n")
}

/// Avisos visibles. El recorte puede venir de las globales: no se atribuye al repo.
pub fn avisos(status: &str, reason: &str, label: &str, truncated: usize) -> Vec<String> {
    let mut v = Vec::new();
    if status == "skip"
        && matches!(
            reason,
            "ambigua" | "seccion_vacia" | "error_engine" | "engine_stale"
        )
    {
        v.push(format!("⚠ sin reglas de proyecto para {label} ({reason})"));
    }
    if truncated > 0 {
        v.push(format!(
            "⚠ {truncated} reglas duras recortadas al máximo de 10 (corre «exo rules --check»)"
        ));
    }
    v
}

fn etiqueta(repo: &Option<String>, cwd: &Path) -> String {
    repo.clone().unwrap_or_else(|| {
        cwd.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    })
}

fn desde_salida(s: Salida, cwd: &Path) -> ResultadoReglas {
    let n_global = s.global.len();
    let (status, reason, repo, proyecto) = match s.resultado {
        Resultado::Ok { repo, rules, .. } => ("ok", String::new(), Some(repo), rules),
        Resultado::Skip { repo, reason, .. } => {
            ("skip", reason.como_str().to_string(), repo, vec![])
        }
    };
    ResultadoReglas {
        bloque: bloque_reglas(&s.global, &proyecto),
        avisos: avisos(status, &reason, &etiqueta(&repo, cwd), s.truncated),
        status: status.into(),
        reason,
        n: proyecto.len() + n_global,
        n_global,
        repo,
        error: String::new(),
    }
}

/// Sin bloque ni globales: el engine falló o está desactualizado.
fn skip_sin_reglas(reason: &str, cwd: &Path, error: String) -> ResultadoReglas {
    ResultadoReglas {
        bloque: String::new(),
        avisos: avisos("skip", reason, &etiqueta(&None, cwd), 0),
        status: "skip".into(),
        reason: reason.into(),
        repo: None,
        n: 0,
        n_global: 0,
        error,
    }
}

/// Un `Err` de `reglas::resuelve` es `error_engine`: visible, nunca silencioso.
pub fn resuelve_reglas(kb: &Path, cwd: &Path) -> ResultadoReglas {
    match reglas::resuelve(kb, cwd) {
        Ok(s) => desde_salida(s, cwd),
        Err(e) => {
            let msg: String = format!("{e:#}")
                .chars()
                .take(ERR_MAX)
                .map(|c| if c == '\n' { ' ' } else { c })
                .collect();
            skip_sin_reglas("error_engine", cwd, msg)
        }
    }
}

/// Engine más viejo que `ENGINE_MIN`: ni se resuelve (el runner lo decide).
pub fn engine_stale(cwd: &Path) -> ResultadoReglas {
    skip_sin_reglas("engine_stale", cwd, String::new())
}

#[derive(Serialize)]
struct Testigo<'a> {
    status: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    reason: &'a str,
    n: usize,
    n_global: usize,
    repo: Option<&'a str>,
}

fn poda(rules_dir: &Path) {
    let Ok(entradas) = std::fs::read_dir(rules_dir) else {
        return;
    };
    let ahora = SystemTime::now();
    for e in entradas.flatten() {
        let nombre = e.file_name();
        let nombre = nombre.to_string_lossy();
        if !(nombre.starts_with("ss-") || nombre.starts_with("hb-")) {
            continue;
        }
        let Ok(meta) = e.metadata() else { continue };
        let vieja = meta
            .modified()
            .ok()
            .and_then(|m| ahora.duration_since(m).ok())
            .is_some_and(|edad| edad >= EDAD_PODA);
        if meta.is_file() && vieja {
            let _ = std::fs::remove_file(e.path());
        }
    }
}

/// Poda `ss-*`/`hb-*` viejos y deja `ss-<sid>`. Sin sid, o con `/`, no escribe testigo.
/// Best-effort: el estado nunca rompe el hook.
pub fn escribe_testigo(rules_dir: &Path, sid: &str, r: &ResultadoReglas) {
    poda(rules_dir);
    if sid.is_empty() || sid.contains('/') {
        return;
    }
    let t = Testigo {
        status: &r.status,
        reason: &r.reason,
        n: r.n,
        n_global: r.n_global,
        repo: r.repo.as_deref(),
    };
    let Ok(json) = serde_json::to_string(&t) else {
        return;
    };
    let _ = (|| -> std::io::Result<()> {
        std::fs::create_dir_all(rules_dir)?;
        let tmp = rules_dir.join(format!("ss-{sid}.tmp"));
        std::fs::write(&tmp, json)?;
        std::fs::rename(&tmp, rules_dir.join(format!("ss-{sid}")))
    })();
}

/// `project-rules-skip` de `exo-recall.sh:335`; no hace nada si el resultado no es `skip`.
pub fn registra_skip(en: &Entorno, ev: &Evento, r: &ResultadoReglas) {
    if r.status != "skip" {
        return;
    }
    let repo = r.repo.as_deref().unwrap_or("-");
    let err = if r.reason == "error_engine" && !r.error.is_empty() {
        format!(" err={}", r.error)
    } else {
        String::new()
    };
    registra(
        en,
        "project-rules-skip",
        ev,
        &format!("reason={} repo={repo}{err}", r.reason),
    );
}

/// `ReglasTestigo`: SessionStart que no sea compact. `None` en compact (ni resuelve ni testigo).
pub fn reglas_testigo(en: &Entorno, kb: &Path, ev: &Evento) -> Option<ResultadoReglas> {
    if ev.source == "compact" {
        return None;
    }
    let cwd = if ev.cwd.is_empty() {
        std::env::current_dir().unwrap_or_default()
    } else {
        ev.cwd.clone().into()
    };
    let r = resuelve_reglas(kb, &cwd);
    registra_skip(en, ev, &r);
    escribe_testigo(&en.rutas.rules_dir, &ev.session_id, &r);
    Some(r)
}
