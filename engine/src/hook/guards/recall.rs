//! Guard Recall: contrato de recall en SessionStart, migrado de exo-recall.sh.

use crate::hook::evento::{Entorno, Evento};
use crate::recall::recall_arranque_contenido;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::PathBuf;

// El texto es literal de exo-recall.sh:26-30 (sin tildes a propósito: el golden lo fija así).
pub const FALLBACK: &str = "Tu memoria persistente es una KB de notas markdown servida por el engine `exo` (`exo recall`, `exo search --type hybrid`). Antes de empezar trabajo sustantivo, busca ahi contexto relevante. Al cerrar una sesion con decisiones/aprendizajes, documentalos con /document (busca antes de escribir; edita la nota canonica en vez de duplicar).

MODO ORQUESTADOR LIMPIO (por defecto): delega investigacion, ejecucion multi-paso y lecturas voluminosas a subagentes. Quedate con la CONCLUSION, no con el material crudo. Context-rot validado: mas contexto en el padre = peor rendimiento. Tu contexto es para sintetizar y decidir, no para acumular fuentes.

RECON-FIRST (look before you leap) en tareas DURAS/desconocidas/time-boxed: antes de grindear en solitario, recoge informacion (busca el error/los docs, verifica supuestos). Retrieve > compute para lo que no esta en tus pesos; reintentar lo mismo a ciegas no es progreso. (skill: exo:debug, puerta 2.)";

const ENCABEZADO: &str = "## Contrato de memoria";

/// Piezas del `additionalContext` de SessionStart, antes de aplicar el cap.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Fragmentos {
    pub contrato: String,
    pub avisos: Vec<String>,
    pub reafirmacion: String,
    pub reglas: String,
}

/// Texto del contrato y las `reason` de `recall-fallback` que el llamador registra.
/// Seams primero y luego config: no lee el entorno del proceso (`EXO_DB` es de la CLI).
pub fn recall_contrato(_ev: &Evento, en: &Entorno) -> (String, Vec<&'static str>) {
    let mut razones = vec![];
    let s = &en.seams;
    let Some(db) = s.exo_index.clone().or_else(|| crate::config_db().ok()) else {
        return (FALLBACK.into(), vec!["no-index"]);
    };
    if !db.is_file() {
        return (FALLBACK.into(), vec!["no-index"]);
    }
    let nota = match &s.recall_nota {
        Some(n) => n.clone(),
        None => match s.exo_kb_name.clone().or_else(|| crate::nombre_kb().ok()) {
            Some(nombre) => format!("{nombre}/core/core-index"),
            None => {
                razones.push("no-config");
                "core/core-index".to_string()
            }
        },
    };
    let kb: Option<PathBuf> = s.exo_kb.clone().or_else(|| crate::kb_desde_config().ok());
    let base = kb.and_then(|kb| {
        let b =
            recall_arranque_contenido(&db, &kb, s.recall_limite, s.recall_cap, Some(&nota)).ok()?;
        // El engine solo avisa por stderr de que cortó; sin cap la salida es el bloque entero.
        let entero =
            recall_arranque_contenido(&db, &kb, s.recall_limite, usize::MAX, Some(&nota)).ok();
        if entero.is_some_and(|e| e.len() > b.len()) {
            razones.push("truncated");
        }
        Some(b)
    });
    // `$(...)` de la shell descartaba los saltos de línea finales.
    let base = base
        .map(|b| b.trim_end_matches('\n').to_string())
        .unwrap_or_default();
    if base.is_empty() {
        razones.push("empty");
        return (FALLBACK.into(), razones);
    }
    if !base.contains("Contrato de memoria") {
        razones.push("no-contract");
        return (FALLBACK.into(), razones);
    }
    // Como `$(awk …)`: sin saltos de línea finales.
    let seccion = extrae_seccion(&base).trim_end_matches('\n').to_string();
    (if seccion.is_empty() { base } else { seccion }, razones)
}

/// Del `## Contrato de memoria` al siguiente `## `; vacío si no hay encabezado.
fn extrae_seccion(base: &str) -> String {
    let mut dentro = false;
    let mut out = String::new();
    for l in base.lines() {
        if l.starts_with(ENCABEZADO) {
            dentro = true;
        } else if dentro && l.starts_with("## ") {
            break;
        } else if !dentro {
            continue;
        }
        out.push_str(l);
        out.push('\n');
    }
    out
}

/// Pin por reflejo; git-c y git-c-rewrite comparten línea.
fn pin(id: &str) -> Option<&'static str> {
    match id {
        "git-c" | "git-c-rewrite" => Some("- nunca `cd X && git ...`: usa `git -C X`."),
        "verify-before-done" | "verify-before-commit" => {
            Some("- verifica (corre el cambio) antes de afirmar exito.")
        }
        _ => None,
    }
}

/// Reafirmación de los reflejos que dispararon en `sid`, en las últimas
/// `seams.compact_lineas` líneas de `rutas.reflex_log`. Vacía si no hay pins;
/// si no, empieza por `\n\n` para concatenarse al bloque.
pub fn reafirmacion(sid: &str, en: &Entorno) -> String {
    if sid.is_empty() {
        return String::new();
    }
    let Ok(txt) = std::fs::read_to_string(&en.rutas.reflex_log) else {
        return String::new();
    };
    let disparados: BTreeSet<String> = txt
        .lines()
        .rev()
        .take(en.seams.compact_lineas)
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|v| v["session_id"].as_str() == Some(sid))
        .filter_map(|v| v["reflex"].as_str().map(str::to_string))
        .collect();
    let mut pins: Vec<&str> = vec![];
    for p in disparados.iter().filter_map(|id| pin(id)) {
        if !pins.contains(&p) {
            pins.push(p);
        }
    }
    if pins.is_empty() {
        return String::new();
    }
    format!(
        "\n\n--- Reglas reforzadas tras compactacion (dispararon esta sesion) ---\n{}",
        pins.join("\n")
    )
}

/// Bloque de Claude: contrato + avisos recortados a `cap` bytes por el final (sin partir
/// UTF-8) y la reafirmación detrás, fuera del cap. Devuelve además los bytes previos al
/// recorte si hubo (el `recall-truncated` que registrar).
pub fn compone_claude(fr: &Fragmentos, cap: usize) -> (String, Option<usize>) {
    let mut texto = fr.contrato.clone();
    for a in &fr.avisos {
        texto.push_str("\n\n");
        texto.push_str(a);
    }
    let antes = texto.len();
    let recortado = antes > cap;
    if recortado {
        let mut fin = cap;
        while !texto.is_char_boundary(fin) {
            fin -= 1;
        }
        texto.truncate(fin);
        texto.truncate(texto.trim_end_matches('\n').len());
    }
    texto.push_str(&fr.reafirmacion);
    (texto, recortado.then_some(antes))
}
