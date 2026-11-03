use exo::hook::evento::{Decision, Entorno, Evento, EventoKind, Harness};
use exo::hook::reflex::lee_cola;
use std::path::{Path, PathBuf};

fn entorno(d: &Path, exo_bin: Option<PathBuf>, exo_index: Option<PathBuf>) -> Entorno {
    let mut seams = exo::state::Seams::resuelve(|_| None);
    seams.exo_bin = exo_bin;
    seams.exo_index = exo_index;
    let mut rutas = exo::state::Rutas::resuelve(Path::new("."), |_| None);
    rutas.index_log = d.join("estado/exo-index.log");
    Entorno {
        state_dir: d.to_path_buf(),
        reflex_log: d.join("log.jsonl"),
        rutas,
        seams,
        cdpath_definido: false,
        panic_guard: None,
    }
}

fn stop() -> Evento {
    Evento {
        harness: Harness::Claude,
        kind: EventoKind::Stop,
        session_id: "s".into(),
        cwd: "/tmp/x".into(),
        agent_id: String::new(),
        agent_type: String::new(),
        source: String::new(),
        prompt: String::new(),
        crudo: "{}".into(),
        tool: None,
    }
}

fn payloads(en: &Entorno) -> Vec<String> {
    lee_cola(en, 50)
        .iter()
        .filter(|v| v["reflex"] == "index-fallback")
        .map(|v| v["payload"].as_str().unwrap().to_string())
        .collect()
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::io::{Read, Write};
    use std::os::unix::fs::PermissionsExt;

    /// Stub: se bloquea leyendo la fifo `go`, vuelca sus args a stdout (= el log)
    /// y avisa por la fifo `done`. Ambas las controla el test, no hay reloj.
    fn stub(d: &Path) -> (PathBuf, PathBuf, PathBuf) {
        let (go, done, bin) = (d.join("go"), d.join("done"), d.join("exo-stub"));
        for f in [&go, &done] {
            assert!(
                std::process::Command::new("mkfifo")
                    .arg(f)
                    .status()
                    .unwrap()
                    .success()
            );
        }
        let script = format!(
            "#!/bin/sh\nread _ < '{}'\necho $$\nps -o pgid= -p $$ | tr -d ' '\nfor a in \"$@\"; do echo \"$a\"; done\necho fin > '{}'\n",
            go.display(),
            done.display()
        );
        std::fs::write(&bin, script).unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        (bin, go, done)
    }

    fn suelta_y_espera(go: &Path, done: &Path) {
        std::fs::OpenOptions::new()
            .write(true)
            .open(go)
            .unwrap()
            .write_all(b"x\n")
            .unwrap();
        let mut s = String::new();
        std::fs::File::open(done)
            .unwrap()
            .read_to_string(&mut s)
            .unwrap();
    }

    #[test]
    fn no_espera() {
        let d = tempfile::tempdir().unwrap();
        let (bin, go, done) = stub(d.path());
        let en = entorno(d.path(), Some(bin), Some(d.path().join("i.db")));
        // Si el guard esperase al hijo no volvería: el stub sigue bloqueado en
        // `go` hasta que el test lo suelta. El hilo + recv_timeout convierte el
        // cuelgue en fallo.
        let (tx, rx) = std::sync::mpsc::channel();
        let en2 = en.clone();
        std::thread::spawn(move || {
            let _ = tx.send(exo::hook::guards::index::index(&stop(), &en2));
        });
        let dec = rx
            .recv_timeout(std::time::Duration::from_secs(20))
            .expect("index() esperó al hijo");
        assert!(matches!(dec, Decision::Allow));
        assert!(
            !en.rutas.index_log.exists()
                || std::fs::read_to_string(&en.rutas.index_log)
                    .unwrap()
                    .is_empty()
        );
        suelta_y_espera(&go, &done);
        assert!(
            std::fs::read_to_string(&en.rutas.index_log)
                .unwrap()
                .contains("--json")
        );
    }

    #[test]
    fn argumentos() {
        let d = tempfile::tempdir().unwrap();
        let (bin, go, done) = stub(d.path());
        let db = d.path().join("i.db");
        let en = entorno(d.path(), Some(bin), Some(db.clone()));
        exo::hook::guards::index::index(&stop(), &en);
        suelta_y_espera(&go, &done);
        let log = std::fs::read_to_string(&en.rutas.index_log).unwrap();
        let args: Vec<&str> = log.lines().skip(2).collect();
        assert_eq!(args, ["index", "--db", db.to_str().unwrap(), "--json"]);
    }

    #[test]
    fn grupo_propio() {
        let d = tempfile::tempdir().unwrap();
        let (bin, go, done) = stub(d.path());
        let en = entorno(d.path(), Some(bin), Some(d.path().join("i.db")));
        exo::hook::guards::index::index(&stop(), &en);
        suelta_y_espera(&go, &done);
        let log = std::fs::read_to_string(&en.rutas.index_log).unwrap();
        let mut l = log.lines();
        let pid: i32 = l.next().unwrap().trim().parse().unwrap();
        let pgid: i32 = l.next().unwrap().trim().parse().unwrap();
        assert_eq!(pgid, pid, "el hijo debe liderar su propio grupo");
        assert_ne!(
            pgid,
            unsafe { libc::getpgid(0) },
            "no debe heredar el grupo del hook"
        );
    }
}

#[test]
fn fallo_detach() {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(
        d.path(),
        Some(d.path().join("no-existe")),
        Some(d.path().join("i.db")),
    );
    assert!(matches!(
        exo::hook::guards::index::index(&stop(), &en),
        Decision::Allow
    ));
    let p = payloads(&en);
    assert_eq!(p.len(), 1, "{p:?}");
    assert!(p[0].starts_with("reason=detach-failed"), "{p:?}");
}

/// Sin `exo_index` la db sale de la config; `carga()` lee `EXO_CONFIG`, así que
/// el caso corre en un hijo con el entorno acotado (los tests no tocan `set_var`).
#[test]
fn sin_db() {
    if std::env::var_os("EXO_T11_HIJO").is_some() {
        let d = std::path::PathBuf::from(std::env::var("EXO_T11_DIR").unwrap());
        let en = entorno(&d, None, None);
        assert!(matches!(
            exo::hook::guards::index::index(&stop(), &en),
            Decision::Allow
        ));
        return;
    }
    let d = tempfile::tempdir().unwrap();
    let st = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "sin_db", "--test-threads=1"])
        .env("EXO_T11_HIJO", "1")
        .env("EXO_T11_DIR", d.path())
        .env("EXO_CONFIG", d.path().join("no-existe.toml"))
        .status()
        .unwrap();
    assert!(st.success());
    let en = entorno(d.path(), None, None);
    assert_eq!(payloads(&en), vec!["reason=no-db".to_string()]);
    assert!(!en.rutas.index_log.exists(), "no debe lanzar nada sin db");
}
