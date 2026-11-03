use exo::hook::evento::{Entorno, Evento, EventoKind, Harness, Tool, clasifica};
use exo::hook::journal::{lee, registra_tool, registra_warn, ruta};
use serde_json::json;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

fn entorno(dir: &std::path::Path) -> Entorno {
    Entorno {
        state_dir: dir.to_path_buf(),
        reflex_log: dir.join("reflex.jsonl"),
        rutas: exo::state::Rutas::resuelve(std::path::Path::new("."), |_| None),
        seams: exo::state::Seams::resuelve(|_| None),
        cdpath_definido: false,
        panic_guard: None,
    }
}

fn evento(sid: &str, agent: &str, nombre: &str, input: serde_json::Value) -> Evento {
    let (clase, command, paths) = clasifica(nombre, &input);
    Evento {
        harness: Harness::Claude,
        kind: EventoKind::PostToolUse,
        session_id: sid.into(),
        cwd: "/x".into(),
        agent_id: agent.into(),
        agent_type: String::new(),
        source: String::new(),
        prompt: String::new(),
        crudo: String::new(),
        tool: Some(Tool {
            nombre: nombre.into(),
            clase,
            input,
            command,
            paths,
        }),
    }
}

fn bash(sid: &str, agent: &str, cmd: &str) -> Evento {
    evento(sid, agent, "Bash", json!({ "command": cmd }))
}

#[test]
fn registra_y_lee_en_orden() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path());
    registra_tool(&en, &bash("s", "", "uno"), Some(true)).unwrap();
    registra_tool(
        &en,
        &evento("s", "", "Edit", json!({"file_path":"a.rs"})),
        None,
    )
    .unwrap();
    registra_warn(&en, &bash("s", "", "tres"), "W001").unwrap();
    let l = lee(&en, Harness::Claude, "s", "").unwrap().unwrap();
    assert!(!l.truncado);
    let r = &l.registros;
    assert_eq!(r.len(), 3);
    assert_eq!(
        (r[0].kind.as_str(), r[0].cmd.as_str(), r[0].ok),
        ("tool", "uno", Some(true))
    );
    assert_eq!((r[0].tool.as_str(), r[0].clase.as_str()), ("Bash", "Shell"));
    assert_eq!(
        (r[1].paths.clone(), r[1].ok),
        (vec!["a.rs".to_string()], None)
    );
    assert_eq!(r[1].clase, "FileEdit");
    assert_eq!((r[2].kind.as_str(), r[2].code.as_str()), ("warn", "W001"));
    assert_eq!(r[0].ts.len(), "2026-01-01T00:00:00Z".len());
    assert!(r[0].ts.ends_with('Z'));
}

#[test]
fn cmd_se_trunca_a_200_bytes_en_frontera_utf8() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path());
    // 'ñ' ocupa los bytes 199..201: cortar en 200 la partiría.
    let cmd = format!("{}ñ{}", "a".repeat(199), "b".repeat(99));
    assert_eq!(cmd.len(), 300);
    registra_tool(&en, &bash("s", "", &cmd), Some(true)).unwrap();
    let l = lee(&en, Harness::Claude, "s", "").unwrap().unwrap();
    let c = &l.registros[0].cmd;
    assert!(c.len() <= 200, "len {}", c.len());
    assert_eq!(c, &"a".repeat(199));
}

#[test]
fn una_linea_un_write_concurrente() {
    let d = tempfile::tempdir().unwrap();
    let en = Arc::new(entorno(d.path()));
    let hs: Vec<_> = (0..8)
        .map(|h| {
            let en = en.clone();
            std::thread::spawn(move || {
                for i in 0..300 {
                    let cmd = format!("h{h}-{i}-{}", "x".repeat(150));
                    registra_tool(&en, &bash("s", "", &cmd), Some(true)).unwrap();
                }
            })
        })
        .collect();
    for h in hs {
        h.join().unwrap();
    }
    let txt = std::fs::read_to_string(ruta(&en, Harness::Claude, "s")).unwrap();
    let lineas: Vec<_> = txt.lines().collect();
    assert_eq!(lineas.len(), 2400);
    for l in lineas {
        serde_json::from_str::<serde_json::Value>(l).unwrap();
    }
}

#[test]
fn lee_filtra_por_agent_id() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path());
    registra_tool(&en, &bash("s", "", "padre"), None).unwrap();
    registra_tool(&en, &bash("s", "ag1", "hijo"), None).unwrap();
    let p = lee(&en, Harness::Claude, "s", "").unwrap().unwrap();
    assert_eq!(p.registros.len(), 1);
    assert_eq!(p.registros[0].cmd, "padre");
    let h = lee(&en, Harness::Claude, "s", "ag1").unwrap().unwrap();
    assert_eq!(h.registros.len(), 1);
    assert_eq!(h.registros[0].cmd, "hijo");
}

#[test]
fn lee_salta_linea_corrupta() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path());
    registra_tool(&en, &bash("s", "", "uno"), None).unwrap();
    let mut f = OpenOptions::new()
        .append(true)
        .open(ruta(&en, Harness::Claude, "s"))
        .unwrap();
    f.write_all(b"{esto no es json\n").unwrap();
    drop(f);
    registra_tool(&en, &bash("s", "", "dos"), None).unwrap();
    let l = lee(&en, Harness::Claude, "s", "").unwrap().unwrap();
    let cmds: Vec<_> = l.registros.iter().map(|r| r.cmd.as_str()).collect();
    assert_eq!(cmds, ["uno", "dos"]);
}

#[test]
fn lee_cota_2_mib_marca_truncado() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path());
    let p = ruta(&en, Harness::Claude, "s");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    let mut f = File::create(&p).unwrap();
    let mut total = 0u64;
    let mut n = 0;
    while total <= 3 * 1024 * 1024 {
        let l = format!(
            "{{\"ts\":\"t\",\"kind\":\"tool\",\"tool\":\"Bash\",\"clase\":\"Shell\",\"cmd\":\"n{n}\",\"paths\":[],\"ok\":null,\"agent_id\":\"\",\"code\":\"\"}}\n"
        );
        f.write_all(l.as_bytes()).unwrap();
        total += l.len() as u64;
        n += 1;
    }
    drop(f);
    let l = lee(&en, Harness::Claude, "s", "").unwrap().unwrap();
    assert!(l.truncado);
    assert!(!l.registros.is_empty() && l.registros.len() < n);
    assert_eq!(l.registros.last().unwrap().cmd, format!("n{}", n - 1));
    // Solo registros completos y contiguos hasta el final.
    let primero: usize = l.registros[0].cmd[1..].parse().unwrap();
    assert_eq!(primero + l.registros.len(), n);
}

#[test]
fn lee_inexistente_es_none() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path());
    assert!(lee(&en, Harness::Claude, "nada", "").unwrap().is_none());
}

fn envejece(p: &std::path::Path, dias: u64) {
    let t = SystemTime::now() - Duration::from_secs(dias * 86_400);
    File::options()
        .write(true)
        .open(p)
        .unwrap()
        .set_modified(t)
        .unwrap();
}

#[test]
fn gc_borra_mayores_de_14_dias_al_crear_sesion() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path());
    let dir = d.path().join("sessions");
    std::fs::create_dir_all(&dir).unwrap();
    let viejo = dir.join("claude-viejo.jsonl");
    let reciente = dir.join("claude-reciente.jsonl");
    std::fs::write(&viejo, "").unwrap();
    std::fs::write(&reciente, "").unwrap();
    envejece(&viejo, 15);
    envejece(&reciente, 13);
    registra_tool(&en, &bash("nueva", "", "x"), None).unwrap();
    assert!(!viejo.exists());
    assert!(reciente.exists());

    // Sesión ya existente: no hay GC.
    let viejo2 = dir.join("claude-viejo2.jsonl");
    std::fs::write(&viejo2, "").unwrap();
    envejece(&viejo2, 15);
    registra_tool(&en, &bash("nueva", "", "y"), None).unwrap();
    assert!(viejo2.exists());
}

#[test]
fn session_id_no_escapa_del_dir() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path());
    let p = ruta(&en, Harness::Claude, "../../x");
    assert_eq!(p.parent().unwrap(), d.path().join("sessions"));
    let nombre = |sid: &str| {
        ruta(&en, Harness::Claude, sid)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    };
    assert_eq!(nombre("a-B_9"), "claude-a-B_9.jsonl");
    assert_eq!(nombre("a.b/c"), "claude-a_b_c.jsonl");
    assert_eq!(nombre("ñ-1"), "claude-_-1.jsonl");
    registra_tool(&en, &bash("../../x", "", "z"), None).unwrap();
    assert!(p.exists());
}
