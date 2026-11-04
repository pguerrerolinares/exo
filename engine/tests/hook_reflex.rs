use exo::hook::claude;
use exo::hook::evento::{Entorno, Evento};
use exo::hook::reflex::{lee_cola, payload_truncado, registra};
use regex::Regex;
use serde_json::json;
use std::path::PathBuf;

fn entorno(log: PathBuf) -> Entorno {
    Entorno {
        state_dir: PathBuf::from("."),
        reflex_log: log,
        rutas: exo::state::Rutas::resuelve(std::path::Path::new("."), |_| None),
        seams: exo::state::Seams::resuelve(|_| None),
        cdpath_definido: false,
        panic_guard: None,
    }
}

fn evento(tool: Option<&str>) -> Evento {
    let mut v = json!({"session_id": "s1", "cwd": "/x", "agent_id": "a1", "agent_type": "t1"});
    if let Some(n) = tool {
        v["tool_name"] = json!(n);
        v["tool_input"] = json!({});
    }
    claude::parse("PreToolUse", v.to_string().as_bytes()).unwrap()
}

#[test]
fn registra_esquema_identico_al_sh() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path().join("sub/log.jsonl"));
    std::fs::create_dir_all(d.path().join("sub")).unwrap();
    registra(&en, "r1", &evento(Some("Bash")), "pay");
    registra(&en, "r2", &evento(None), "x");
    let cola = lee_cola(&en, 10);
    assert_eq!(cola.len(), 2);
    let v = cola[0].as_object().unwrap();
    let mut claves: Vec<&str> = v.keys().map(String::as_str).collect();
    claves.sort();
    assert_eq!(
        claves,
        [
            "agent_id",
            "agent_type",
            "harness",
            "payload",
            "reflex",
            "session_id",
            "tool",
            "ts"
        ]
    );
    assert_eq!(v["reflex"], "r1");
    assert_eq!(v["tool"], "Bash");
    assert_eq!(v["session_id"], "s1");
    assert_eq!(v["payload"], "pay");
    let ts = v["ts"].as_str().unwrap();
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$").unwrap();
    assert!(re.is_match(ts), "ts: {ts}");
    assert_eq!(cola[1]["tool"], "");
    assert_eq!(lee_cola(&en, 1)[0]["reflex"], "r2");
}

#[test]
fn payload_se_corta_a_2000_chars() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path().join("log.jsonl"));
    let largo = "é".repeat(2500);
    registra(&en, "r", &evento(None), &largo);
    let p = lee_cola(&en, 1)[0]["payload"].as_str().unwrap().to_string();
    assert_eq!(p.chars().count(), 2000);
}

#[test]
fn payload_truncado_corto_intacto() {
    let re = Regex::new("é").unwrap();
    let cmd = "é".repeat(120);
    assert_eq!(payload_truncado(&cmd, &re), cmd);
    assert!(payload_truncado(&"é".repeat(121), &re).contains("⟨match⟩"));
}

#[test]
fn payload_truncado_largo_con_match() {
    let re = Regex::new(r"git add \S*").unwrap();
    let cmd = format!(
        "{}git add a {}git add b git add a",
        "ñ".repeat(120),
        "ü".repeat(30)
    );
    let p = payload_truncado(&cmd, &re);
    assert_eq!(
        p,
        format!("{} … ⟨match⟩ git add a | git add b", "ñ".repeat(120))
    );

    let gigante = format!("git add {}{}", "a".repeat(100), "é".repeat(100));
    let cmd = format!("{}{}", "_".repeat(130), gigante);
    let p = payload_truncado(&cmd, &re);
    let m = p.split("⟨match⟩ ").nth(1).unwrap();
    let (cab, cola) = m.split_once('…').unwrap();
    assert_eq!(cab.chars().count(), 80);
    assert_eq!(cola.chars().count(), 60);
    assert!(cola.chars().all(|c| c == 'é'));

    let siete: String = (0..7).map(|i| format!("git add f{i} ")).collect();
    let p = payload_truncado(&format!("{}{siete}", "x".repeat(130)), &re);
    assert!(
        p.ends_with("git add f0 | git add f1 | git add f2 | git add f3 | git add f4"),
        "{p}"
    );
}

#[test]
fn payload_truncado_largo_sin_match_solo_prefijo() {
    let re = Regex::new("zzz").unwrap();
    let cmd = "a".repeat(200);
    assert_eq!(payload_truncado(&cmd, &re), "a".repeat(120));
}

#[test]
fn registra_sin_destino_no_falla() {
    let d = tempfile::tempdir().unwrap();
    let fichero = d.path().join("fichero");
    std::fs::write(&fichero, "x").unwrap();
    let en = entorno(fichero.join("dentro/log.jsonl"));
    registra(&en, "r", &evento(None), "p");
    assert!(lee_cola(&en, 5).is_empty());

    let ok = entorno(d.path().join("ok.jsonl"));
    registra(&ok, "r", &evento(None), "p");
    assert_eq!(lee_cola(&ok, 5).len(), 1);
}

#[test]
fn payload_truncado_umbral_de_match_y_vacios() {
    let re = Regex::new(r"g\S*").unwrap();
    let relleno = " ".repeat(130);
    let exacto = format!("g{}", "é".repeat(139));
    let p = payload_truncado(&format!("{relleno}{exacto}"), &re);
    assert!(
        p.ends_with(&format!("⟨match⟩ {exacto}")),
        "140 chars quedan intactos"
    );
    let p = payload_truncado(&format!("{relleno}g{}", "é".repeat(140)), &re);
    assert!(p.contains('…'), "141 chars se recortan");

    let vacio = Regex::new("x*").unwrap();
    let p = payload_truncado(&format!("{}xx", "a".repeat(130)), &vacio);
    assert!(p.ends_with("⟨match⟩ xx"), "{p}");
}

#[test]
fn registra_concurrente_no_intercala_lineas() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path().join("log.jsonl"));
    let (hilos, por_hilo) = (8, 200);
    std::thread::scope(|sc| {
        for h in 0..hilos {
            let en = &en;
            sc.spawn(move || {
                for i in 0..por_hilo {
                    registra(
                        en,
                        &format!("h{h}-{i}"),
                        &evento(Some("Bash")),
                        &"p".repeat(50),
                    );
                }
            });
        }
    });
    let txt = std::fs::read_to_string(&en.reflex_log).unwrap();
    assert_eq!(txt.lines().count(), hilos * por_hilo);
    for l in txt.lines() {
        serde_json::from_str::<serde_json::Value>(l).unwrap_or_else(|e| panic!("{e}: {l}"));
    }
}

#[test]
fn lee_cola_salta_lineas_ilegibles() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path().join("log.jsonl"));
    registra(&en, "a", &evento(None), "");
    let mut txt = std::fs::read_to_string(&en.reflex_log).unwrap();
    txt.push_str("{roto\n");
    std::fs::write(&en.reflex_log, txt).unwrap();
    registra(&en, "b", &evento(None), "");
    let cola = lee_cola(&en, 10);
    let ids: Vec<_> = cola.iter().map(|v| v["reflex"].as_str().unwrap()).collect();
    assert_eq!(ids, ["a", "b"]);
}

#[test]
fn lleva_harness() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path().join("log.jsonl"));
    registra(&en, "r", &evento(None), "");
    assert_eq!(lee_cola(&en, 1)[0]["harness"], "claude");
}

#[test]
fn crea_el_padre_del_log() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path().join("no/existe/log.jsonl"));
    registra(&en, "r", &evento(None), "");
    assert_eq!(lee_cola(&en, 5).len(), 1);
}
