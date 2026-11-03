use exo::hook::claude::render;
use exo::hook::evento::{Decision, Evento, EventoKind, Harness};
use exo::hook::matriz::{GUARDS, Guard, Soporte, soporte};
use serde_json::json;

fn decision(nombre: &str) -> Decision {
    match nombre {
        "Allow" => Decision::Allow,
        "Deny" => Decision::Deny("r".into()),
        "Rewrite" => Decision::Rewrite {
            input: json!({"command": "x"}),
            razon: "r".into(),
        },
        "Steer" => Decision::Steer("r".into()),
        "Block" => Decision::Block("r".into()),
        "Avisa" => Decision::Avisa("r".into()),
        otro => panic!("decisión desconocida en la matriz: {otro}"),
    }
}

fn evento(kind: EventoKind) -> Evento {
    Evento {
        harness: Harness::Claude,
        kind,
        session_id: String::new(),
        cwd: String::new(),
        agent_id: String::new(),
        agent_type: String::new(),
        crudo: String::new(),
        source: String::new(),
        prompt: String::new(),
        tool: None,
    }
}

fn fixtures_claude() -> Vec<serde_json::Value> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hook/claude");
    std::fs::read_dir(dir)
        .unwrap()
        .flat_map(|e| {
            let texto = std::fs::read_to_string(e.unwrap().path()).unwrap();
            texto
                .lines()
                .map(|l| serde_json::from_str(l).unwrap())
                .collect::<Vec<serde_json::Value>>()
        })
        .collect()
}

#[test]
fn matriz_capacidad_claude() {
    let golden = fixtures_claude();
    let declarados: Vec<_> = GUARDS
        .iter()
        .filter(|g| {
            matches!(
                soporte(**g, Harness::Claude),
                Soporte::Soportado | Soporte::Degradado(_)
            )
        })
        .collect();
    assert!(
        !declarados.is_empty(),
        "ningún guard declarado Soportado/Degradado en claude"
    );
    for g in declarados {
        let kind = g.evento();
        let nombre = kind.nombre();
        assert!(
            golden.iter().any(|v| v["hook_event_name"] == nombre),
            "{} declarado Soportado en claude pero el golden de claude no contiene {nombre}: decláralo NoSoportado o cambia el evento",
            g.nombre()
        );
        for d in g.decisiones() {
            let salida = render(&decision(d), &evento(kind.clone()));
            assert!(
                *d == "Allow" || !salida.is_empty(),
                "{}: la decisión {d} renderiza vacío en claude",
                g.nombre()
            );
        }
    }
}

#[test]
fn codex_existentes() {
    assert!(matches!(
        soporte(Guard::Verify, Harness::Codex),
        Soporte::Degradado(_)
    ));
    for g in [Guard::GitC, Guard::ZeroResiduo, Guard::SearchFirst] {
        assert_eq!(
            soporte(g, Harness::Codex),
            Soporte::Soportado,
            "{}",
            g.nombre()
        );
    }
}

#[test]
fn requiere_compact() {
    assert_eq!(Guard::RecallCompact.requiere(), [("source", "compact")]);
    for g in GUARDS.iter().filter(|g| **g != Guard::RecallCompact) {
        assert!(g.requiere().is_empty(), "{}", g.nombre());
    }
}

#[test]
fn matriz_tabla_6_2() {
    use Harness::{Claude, Codex};
    let no = |g, h| matches!(soporte(g, h), Soporte::NoSoportado(_));
    for g in [
        Guard::Index,
        Guard::RecallContrato,
        Guard::RecallGatePendiente,
    ] {
        assert_eq!(soporte(g, Claude), Soporte::Soportado, "{}", g.nombre());
        assert_eq!(soporte(g, Codex), Soporte::Soportado, "{}", g.nombre());
    }
    // Ruling (condición §8.6): el golden de 0.166.0 trae SessionStart con source=compact.
    assert_eq!(soporte(Guard::RecallCompact, Claude), Soporte::Soportado);
    assert_eq!(soporte(Guard::RecallCompact, Codex), Soporte::Soportado);
    assert_eq!(
        soporte(Guard::RecallSuperpowers, Claude),
        Soporte::Soportado
    );
    assert!(no(Guard::RecallSuperpowers, Codex));
    assert_eq!(soporte(Guard::ReglasTestigo, Claude), Soporte::Soportado);
    assert!(no(Guard::ReglasTestigo, Codex));
    assert!(no(Guard::ReglasInyecta, Claude));
    assert_eq!(soporte(Guard::ReglasInyecta, Codex), Soporte::Soportado);
    assert!(no(Guard::MarcaCompact, Claude));
    assert_eq!(soporte(Guard::MarcaCompact, Codex), Soporte::Soportado);
}

#[test]
fn eventos_de_los_guards_nuevos() {
    let ev = |g: Guard| g.evento();
    assert_eq!(ev(Guard::Index), EventoKind::Stop);
    assert_eq!(ev(Guard::MarcaCompact), EventoKind::PostCompact);
    for g in [
        Guard::RecallContrato,
        Guard::RecallCompact,
        Guard::RecallSuperpowers,
        Guard::RecallGatePendiente,
        Guard::ReglasTestigo,
        Guard::ReglasInyecta,
    ] {
        assert_eq!(ev(g), EventoKind::SessionStart, "{}", g.nombre());
    }
    assert_eq!(Guard::GitC.tools(Harness::Codex), ["Bash"]);
    assert!(
        Guard::SearchFirst
            .tools(Harness::Codex)
            .contains(&"apply_patch")
    );
    assert!(Guard::Index.tools(Harness::Codex).is_empty());
}

/// Cada guard Soportado/Degradado en Codex tiene en el golden una línea que case el evento
/// y los campos de `requiere()`; y sus decisiones renderizan algo.
#[test]
fn capacidad_codex() {
    let golden: Vec<serde_json::Value> =
        include_str!("fixtures/hook/codex/codex-0.166.0-probe.jsonl")
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
    for g in GUARDS {
        if matches!(soporte(g, Harness::Codex), Soporte::NoSoportado(_)) {
            continue;
        }
        let kind = g.evento();
        assert!(
            golden.iter().any(|v| v["hook_event_name"] == kind.nombre()
                && g.requiere().iter().all(|(k, val)| v[*k] == *val)),
            "{} declarado Soportado en codex pero ninguna línea del golden casa {} con {:?}: decláralo NoSoportado o recaptura el golden",
            g.nombre(),
            kind.nombre(),
            g.requiere()
        );
        let mut e = evento(kind);
        e.harness = Harness::Codex;
        for d in g.decisiones() {
            let salida = exo::hook::codex::render(&decision(d), &e);
            assert!(
                !salida.is_empty(),
                "{}: {d} renderiza vacío en codex",
                g.nombre()
            );
        }
    }
}

/// Cada guard Soportado/Degradado, en cada harness, tiene en el golden de ese harness una línea
/// con su evento y todos los campos de `requiere()`.
#[test]
fn capacidad_con_campos() {
    let codex: Vec<serde_json::Value> =
        include_str!("fixtures/hook/codex/codex-0.166.0-probe.jsonl")
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
    let claude: Vec<serde_json::Value> =
        include_str!("fixtures/hook/claude/claude-2.1.317-probe.jsonl")
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
    for (h, golden) in [(Harness::Claude, &claude), (Harness::Codex, &codex)] {
        for g in GUARDS {
            if matches!(soporte(g, h), Soporte::NoSoportado(_)) {
                continue;
            }
            let kind = g.evento();
            let ev = kind.nombre();
            let en_evento: Vec<_> = golden
                .iter()
                .filter(|v| v["hook_event_name"] == ev)
                .collect();
            for (k, val) in g.requiere() {
                assert!(
                    en_evento.iter().any(|v| v[*k] == *val),
                    "{} declarado Soportado en {} pero ningún {ev} del golden trae {k}={val}: decláralo NoSoportado o recaptura el golden",
                    g.nombre(),
                    h.nombre()
                );
            }
            assert!(
                !en_evento.is_empty(),
                "{} declarado Soportado en {} pero ningún {ev} del golden trae evento={ev}: decláralo NoSoportado o recaptura el golden",
                g.nombre(),
                h.nombre()
            );
        }
    }
}
