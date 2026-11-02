use exo::hook::claude::render;
use exo::hook::evento::{Decision, Evento, EventoKind, Harness};
use exo::hook::matriz::{GUARDS, Soporte, soporte};
use serde_json::json;

fn nombre_evento(k: &EventoKind) -> &str {
    match k {
        EventoKind::PreToolUse => "PreToolUse",
        EventoKind::PostToolUse => "PostToolUse",
        EventoKind::PostToolUseFailure => "PostToolUseFailure",
        EventoKind::Otro(s) => s,
    }
}

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
        tool: None,
    }
}

fn fixtures_claude() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hook/claude");
    std::fs::read_dir(dir)
        .unwrap()
        .map(|e| std::fs::read_to_string(e.unwrap().path()).unwrap())
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
        let nombre = nombre_evento(&kind);
        assert!(
            golden.contains(&format!("\"hook_event_name\":\"{nombre}\"")),
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
