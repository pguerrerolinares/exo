//! Coherencia matriz <-> hooks.json (spec 2026-11-02-exo-hook-multi-harness §4.3, §6.1).

use exo::hook::evento::Harness;
use exo::hook::matriz::{GUARDS, Soporte, soporte};
use regex::Regex;
use serde_json::Value;

fn hooks_json() -> Value {
    let ruta = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../plugins/exo/hooks/hooks.json"
    );
    serde_json::from_str(&std::fs::read_to_string(ruta).expect("hooks.json")).expect("JSON")
}

/// (evento, matcher, command) de cada hook declarado.
fn entradas() -> Vec<(String, String, String)> {
    let mut v = vec![];
    for (evento, grupos) in hooks_json()["hooks"].as_object().expect("hooks") {
        for g in grupos.as_array().expect("grupos") {
            let matcher = g["matcher"].as_str().unwrap_or("").to_string();
            for h in g["hooks"].as_array().expect("hooks[]") {
                let c = h["command"].as_str().expect("command").to_string();
                v.push((evento.clone(), matcher.clone(), c));
            }
        }
    }
    v
}

fn cubre(matcher: &str, tool: &str) -> bool {
    Regex::new(matcher).expect("matcher regex").is_match(tool)
}

fn comando_exo(c: &str) -> Option<(&str, &str)> {
    let mut it = c.strip_prefix("exo hook ")?.split_whitespace();
    Some((it.next()?, it.next()?))
}

#[test]
fn todo_guard_soportado_tiene_entrada() {
    let ents = entradas();
    // Informe (spec §6.1), no fallo: entradas que siguen en `.sh` y la matriz no cubre.
    let sh: Vec<_> = ents
        .iter()
        .filter(|(_, _, c)| comando_exo(c).is_none())
        .collect();
    eprintln!(
        "hook_coherencia: {} entradas .sh fuera de la matriz:",
        sh.len()
    );
    for (e, m, c) in sh {
        eprintln!("  {e} [{m}] {c}");
    }
    for g in GUARDS {
        if matches!(soporte(g, Harness::Claude), Soporte::NoSoportado(_)) {
            continue;
        }
        let kind = g.evento();
        let ev = kind.nombre();
        for t in g.tools() {
            assert!(
                ents.iter().any(|(e, m, c)| e == ev
                    && cubre(m, t)
                    && comando_exo(c) == Some(("claude", ev))),
                "guard {} sin entrada exo hook claude {ev} que cubra {t}",
                g.nombre()
            );
        }
    }
}

#[test]
fn el_journal_tiene_entradas_post() {
    let ents = entradas();
    for ev in ["PostToolUse", "PostToolUseFailure"] {
        for t in ["Bash", "PowerShell", "Edit", "Write", "NotebookEdit"] {
            assert!(
                ents.iter().any(|(e, m, c)| e == ev
                    && cubre(m, t)
                    && comando_exo(c) == Some(("claude", ev))),
                "sin entrada {ev} que cubra {t}"
            );
        }
    }
}

#[test]
fn ninguna_entrada_apunta_a_un_no_soportado() {
    for (e, _, c) in entradas() {
        let Some((harness, ev)) = comando_exo(&c) else {
            continue;
        };
        assert_eq!(harness, Harness::Claude.nombre(), "harness ajeno en {c}");
        assert_eq!(ev, e, "el evento del comando no es el de su clave: {c}");
        for g in GUARDS.iter().filter(|g| g.evento().nombre() == ev) {
            assert!(
                !matches!(soporte(*g, Harness::Claude), Soporte::NoSoportado(_)),
                "{c} cablea el guard no soportado {}",
                g.nombre()
            );
        }
    }
}
