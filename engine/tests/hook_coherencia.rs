//! Coherencia matriz <-> hooks.json / codex.json (spec 2026-11-02-exo-hook-multi-harness §4.3, §6.1;
//! 2026-11-03-exo-hook-codex §6.3).

use exo::hook::evento::Harness;
use exo::hook::matriz::{GUARDS, Guard, Soporte, soporte};
use regex::Regex;
use serde_json::Value;

/// `(evento, matcher, command)` de un hook declarado.
type Entrada = (String, String, String);

fn json_plugin(fichero: &str) -> Value {
    let ruta = format!(
        "{}/../plugins/exo/hooks/{fichero}",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(&ruta).expect(fichero)).expect("JSON")
}

fn entradas_de(raiz: &Value) -> Vec<Entrada> {
    let mut v = vec![];
    for (evento, grupos) in raiz["hooks"].as_object().expect("hooks") {
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

fn entradas() -> Vec<Entrada> {
    entradas_de(&json_plugin("hooks.json"))
}

fn entradas_codex() -> Vec<Entrada> {
    entradas_de(&json_plugin("codex.json"))
}

/// Claude: el matcher es una regex sin anclar.
fn cubre_claude(matcher: &str, tool: &str) -> bool {
    matcher.is_empty() || Regex::new(matcher).expect("matcher regex").is_match(tool)
}

/// Codex: exacto (alternativas con `|`) si solo trae `[A-Za-z0-9_|]`; si no, regex sin anclar.
fn cubre_codex(matcher: &str, tool: &str) -> bool {
    if matcher.is_empty() {
        return true;
    }
    if matcher
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '|')
    {
        return matcher.split('|').any(|alt| alt == tool);
    }
    Regex::new(matcher).expect("matcher regex").is_match(tool)
}

fn cubre(h: Harness, matcher: &str, tool: &str) -> bool {
    match h {
        Harness::Claude => cubre_claude(matcher, tool),
        Harness::Codex => cubre_codex(matcher, tool),
    }
}

fn comando_exo(c: &str) -> Option<(&str, &str)> {
    let mut it = c.strip_prefix("exo hook ")?.split_whitespace();
    Some((it.next()?, it.next()?))
}

fn activo(g: Guard, h: Harness) -> bool {
    !matches!(soporte(g, h), Soporte::NoSoportado(_))
}

/// Guards que atienden `(h, ev)`. Dos eventos no son el `evento()` de nadie y los atiende un guard
/// con otro `evento()`: `UserPromptSubmit` en Codex lo atiende `ReglasInyecta` (inyecta la primera vez
/// tras una marca `post-compact`, spec §4.6) y los `PostToolUse*` los atiende `Verify` (journal con
/// `ok`, spec §5.3).
fn atienden(h: Harness, ev: &str) -> Vec<Guard> {
    GUARDS
        .iter()
        .copied()
        .filter(|g| {
            g.evento().nombre() == ev
                || (h == Harness::Codex && ev == "UserPromptSubmit" && *g == Guard::ReglasInyecta)
                || (ev.starts_with("PostToolUse") && *g == Guard::Verify)
        })
        .collect()
}

fn harness_de(nombre: &str) -> Harness {
    [Harness::Claude, Harness::Codex]
        .into_iter()
        .find(|h| h.nombre() == nombre)
        .unwrap_or_else(|| panic!("harness desconocido: {nombre}"))
}

/// Informe de lo no declarado (spec §6.3): NoSoportado que cada entrada ignora, entradas `.sh`
/// y tools del golden que ningún matcher cubre.
fn informe(ents: &[(Harness, Entrada)], golden_tools: &[String]) -> String {
    let mut s = String::new();
    s.push_str("NoSoportado que cada entrada ignora:\n");
    for (_, (e, _, c)) in ents {
        if let Some((hc, ev)) = comando_exo(c) {
            let ign: Vec<_> = atienden(harness_de(hc), ev)
                .into_iter()
                .filter(|g| !activo(*g, harness_de(hc)))
                .map(|g| g.nombre())
                .collect();
            if !ign.is_empty() {
                s.push_str(&format!("  {c} [{e}] ignora {}\n", ign.join(", ")));
            }
        }
    }
    s.push_str("entradas .sh fuera de la matriz:\n");
    for (_, (e, m, c)) in ents
        .iter()
        .filter(|(_, (_, _, c))| comando_exo(c).is_none())
    {
        s.push_str(&format!("  {e} [{m}] {c}\n"));
    }
    s.push_str("tools del golden sin matcher que las cubra:\n");
    for t in golden_tools {
        let cubierta = ents.iter().any(|(h, (e, m, c))| {
            e.ends_with("ToolUse") && comando_exo(c).is_some() && cubre(*h, m, t)
        });
        if !cubierta {
            s.push_str(&format!("  {t}\n"));
        }
    }
    s
}

fn tools_golden_codex() -> Vec<String> {
    let mut v: Vec<String> = include_str!("fixtures/hook/codex/codex-0.166.0-probe.jsonl")
        .lines()
        .filter_map(|l| {
            serde_json::from_str::<Value>(l).unwrap()["tool_name"]
                .as_str()
                .map(String::from)
        })
        .collect();
    v.sort();
    v.dedup();
    v
}

#[test]
fn todo_guard_soportado_tiene_entrada() {
    let ents = entradas();
    for g in GUARDS {
        if !activo(g, Harness::Claude) {
            continue;
        }
        let kind = g.evento();
        let ev = kind.nombre();
        for t in g.tools(Harness::Claude) {
            assert!(
                ents.iter().any(|(e, m, c)| e == ev
                    && cubre_claude(m, t)
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
                    && cubre_claude(m, t)
                    && comando_exo(c) == Some(("claude", ev))),
                "sin entrada {ev} que cubra {t}"
            );
        }
    }
}

#[test]
fn codex_cubre_tools() {
    let ents = entradas_codex();
    for g in GUARDS {
        if !activo(g, Harness::Codex) {
            continue;
        }
        let kind = g.evento();
        let ev = kind.nombre();
        let tools = g.tools(Harness::Codex);
        if tools.is_empty() {
            assert!(
                ents.iter()
                    .any(|(e, _, c)| e == ev && comando_exo(c) == Some(("codex", ev))),
                "guard {} sin entrada exo hook codex {ev} en codex.json",
                g.nombre()
            );
        }
        for t in tools {
            assert!(
                ents.iter().any(|(e, m, c)| e == ev
                    && cubre_codex(m, t)
                    && comando_exo(c) == Some(("codex", ev))),
                "guard {} sin entrada exo hook codex {ev} que cubra {t} en codex.json",
                g.nombre()
            );
        }
    }
}

#[test]
fn matcher_por_harness() {
    // Codex: `Bash` solo trae [A-Za-z0-9_|] -> exacto.
    assert!(cubre_codex("Bash", "Bash"));
    assert!(!cubre_codex("Bash", "BashX"));
    assert!(cubre_codex("Bash|apply_patch", "apply_patch"));
    assert!(!cubre_codex("Bash|apply_patch", "apply_patch2"));
    // Con metacaracteres es regex sin anclar.
    assert!(cubre_codex("^(Bash)", "BashX"));
    assert!(!cubre_codex("^(Bash)$", "BashX"));
    // Claude: siempre regex sin anclar.
    assert!(cubre_claude("Bash", "BashX"));
    assert!(!cubre_claude("^Bash$", "BashX"));
}

#[test]
fn al_menos_uno() {
    let todas = entradas()
        .into_iter()
        .chain(entradas_codex())
        .collect::<Vec<_>>();
    let mut vistas = 0;
    for (e, _, c) in todas {
        let Some((hc, ev)) = comando_exo(&c) else {
            continue;
        };
        vistas += 1;
        let h = harness_de(hc);
        assert_eq!(ev, e, "el evento del comando no es el de su clave: {c}");
        assert!(
            atienden(h, ev).into_iter().any(|g| activo(g, h)),
            "{c}: ningún guard Soportado/Degradado en ({hc}, {ev})"
        );
    }
    assert!(
        vistas > 0,
        "ninguna entrada exo hook: el recorrido está roto"
    );
}

#[test]
fn codex_json_solo_apunta_a_codex_y_hooks_json_a_claude() {
    for (_, _, c) in entradas_codex() {
        if let Some((h, _)) = comando_exo(&c) {
            assert_eq!(h, "codex", "{c}");
        }
    }
    for (_, _, c) in entradas() {
        if let Some((h, _)) = comando_exo(&c) {
            assert_eq!(h, "claude", "{c}");
        }
    }
}

#[test]
fn informe_texto() {
    let ents = vec![
        (
            Harness::Codex,
            (
                "SessionStart".to_string(),
                String::new(),
                "exo hook codex SessionStart".to_string(),
            ),
        ),
        (
            Harness::Codex,
            (
                "PreToolUse".to_string(),
                "^Bash$".to_string(),
                "exo hook codex PreToolUse".to_string(),
            ),
        ),
        (
            Harness::Claude,
            (
                "Stop".to_string(),
                String::new(),
                "\"${CLAUDE_PLUGIN_ROOT}\"/scripts/x.sh".to_string(),
            ),
        ),
    ];
    let s = informe(&ents, &["Bash".to_string(), "web_search".to_string()]);
    assert!(s.contains("recall-superpowers"), "{s}");
    assert!(s.contains("reglas-testigo"), "{s}");
    assert!(s.contains("scripts/x.sh"), "{s}");
    assert!(s.contains("  web_search\n"), "{s}");
    assert!(!s.contains("  Bash\n"), "{s}");
}

#[test]
fn informe_no_declarado() {
    let ents: Vec<(Harness, Entrada)> = entradas()
        .into_iter()
        .map(|e| (Harness::Claude, e))
        .chain(entradas_codex().into_iter().map(|e| (Harness::Codex, e)))
        .collect();
    let s = informe(&ents, &tools_golden_codex());
    eprintln!("hook_coherencia (informe, no fallo):\n{s}");
    assert!(s.contains("tools del golden"), "{s}");
}
