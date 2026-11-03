use exo::state::{Migracion, Rutas, migra, migra_todo};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

const TS: &str = "20261103T100000";

fn migrado(viejo: &Path) -> std::path::PathBuf {
    let mut s = viejo.as_os_str().to_owned();
    s.push(format!(".migrado-{TS}"));
    s.into()
}

#[test]
fn solo_viejo_renombra() {
    let d = tempfile::tempdir().unwrap();
    let (v, n) = (d.path().join("v"), d.path().join("n"));
    fs::write(&v, b"A\nB\n").unwrap();
    #[cfg(unix)]
    let ino = std::os::unix::fs::MetadataExt::ino(&fs::metadata(&v).unwrap());
    assert_eq!(migra(&v, &n, TS), Migracion::Renombrado);
    assert!(!v.exists());
    assert_eq!(fs::read(&n).unwrap(), b"A\nB\n");
    #[cfg(unix)]
    assert_eq!(
        std::os::unix::fs::MetadataExt::ino(&fs::metadata(&n).unwrap()),
        ino,
        "rename, no copia"
    );
}

#[test]
fn ambos_fusiona() {
    let d = tempfile::tempdir().unwrap();
    let (v, n) = (d.path().join("v"), d.path().join("n"));
    fs::write(&v, "A\n").unwrap();
    fs::write(&n, "B\n").unwrap();
    let r = migra(&v, &n, TS);
    assert_eq!(
        r,
        Migracion::Fusionado {
            viejo_a: migrado(&v)
        }
    );
    assert_eq!(fs::read_to_string(&n).unwrap(), "B\nA\n");
    assert!(!v.exists());
    assert_eq!(fs::read_to_string(migrado(&v)).unwrap(), "A\n");
}

#[test]
fn solo_nuevo_nada() {
    let d = tempfile::tempdir().unwrap();
    let (v, n) = (d.path().join("claude/v"), d.path().join("n"));
    fs::write(&n, "B\n").unwrap();
    assert_eq!(migra(&v, &n, TS), Migracion::Nada);
    assert!(!d.path().join("claude").exists());
    assert_eq!(fs::read_to_string(&n).unwrap(), "B\n");
}

#[test]
fn viejo_sin_salto_final() {
    let d = tempfile::tempdir().unwrap();
    let (v, n) = (d.path().join("v"), d.path().join("n"));
    fs::write(&v, "A").unwrap();
    fs::write(&n, "B\n").unwrap();
    migra(&v, &n, TS);
    assert_eq!(fs::read_to_string(&n).unwrap(), "B\nA\n");
}

#[test]
fn crea_destino() {
    let d = tempfile::tempdir().unwrap();
    let (v, n) = (d.path().join("v"), d.path().join("no/existe/aun/n"));
    fs::write(&v, "A\n").unwrap();
    assert_eq!(migra(&v, &n, TS), Migracion::Renombrado);
    assert_eq!(fs::read_to_string(&n).unwrap(), "A\n");
}

#[test]
fn migrado_no_pisa() {
    let d = tempfile::tempdir().unwrap();
    let (v, n) = (d.path().join("v"), d.path().join("n"));
    fs::write(&n, "").unwrap();
    fs::write(migrado(&v), "previo\n").unwrap();
    fs::write(&v, "A\n").unwrap();
    let mut suf = migrado(&v).into_os_string();
    suf.push("-1");
    let r = migra(&v, &n, TS);
    assert_eq!(
        r,
        Migracion::Fusionado {
            viejo_a: suf.clone().into()
        }
    );
    assert_eq!(fs::read_to_string(migrado(&v)).unwrap(), "previo\n");
    assert_eq!(fs::read_to_string(Path::new(&suf)).unwrap(), "A\n");
}

#[test]
fn fallo_si_el_destino_no_es_creable() {
    let d = tempfile::tempdir().unwrap();
    let (v, f) = (d.path().join("v"), d.path().join("fichero"));
    fs::write(&v, "A\n").unwrap();
    fs::write(&f, "x").unwrap();
    assert!(matches!(
        migra(&v, &f.join("dentro/n"), TS),
        Migracion::Fallo(_)
    ));
    assert!(v.exists(), "ante un fallo el viejo no se toca");
}

#[test]
fn viejo_igual_a_nuevo_es_nada() {
    let d = tempfile::tempdir().unwrap();
    let v = d.path().join("v");
    fs::write(&v, "A\n").unwrap();
    assert_eq!(migra(&v, &v, TS), Migracion::Nada);
    assert_eq!(fs::read_to_string(&v).unwrap(), "A\n");
}

#[test]
fn migra_todo_cubre_los_tres_ficheros() {
    let d = tempfile::tempdir().unwrap();
    let home = d.path();
    let r = Rutas::resuelve(home, |_| None);
    let claude = home.join(".claude");
    fs::create_dir_all(&claude).unwrap();
    for f in ["reflex-log.jsonl", "exo-index.log", "reflex-inject-canary"] {
        fs::write(claude.join(f), format!("{f}\n")).unwrap();
    }
    let res = migra_todo(home, &r, TS);
    let nombres: Vec<_> = res.iter().map(|(n, _)| *n).collect();
    assert_eq!(
        nombres,
        ["reflex-log.jsonl", "exo-index.log", "reflex-inject-canary"]
    );
    assert!(res.iter().all(|(_, m)| *m == Migracion::Renombrado));
    assert_eq!(
        fs::read_to_string(&r.reflex_log).unwrap(),
        "reflex-log.jsonl\n"
    );
    assert_eq!(fs::read_to_string(&r.index_log).unwrap(), "exo-index.log\n");
    assert_eq!(
        fs::read_to_string(&r.canary).unwrap(),
        "reflex-inject-canary\n"
    );
}

#[test]
fn conservacion_concurrente() {
    let d = tempfile::tempdir().unwrap();
    let (v, n) = (d.path().join("v"), d.path().join("n"));
    fs::write(&n, "").unwrap();
    fs::write(&v, "").unwrap();
    let escritas = AtomicUsize::new(0);
    let migrada = AtomicBool::new(false);
    let (hilos, por_hilo) = (4usize, 200usize);
    std::thread::scope(|sc| {
        for h in 0..hilos {
            let (v, escritas, migrada) = (&v, &escritas, &migrada);
            sc.spawn(move || {
                for i in 0..por_hilo {
                    let mut f = fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(v)
                        .unwrap();
                    f.write_all(format!("{{\"h\":{h},\"i\":{i}}}\n").as_bytes())
                        .unwrap();
                    escritas.fetch_add(1, Ordering::SeqCst);
                    // Los escritores siguen hasta que `migra` vuelve: fuerza el interleaving.
                    if i == por_hilo - 1 {
                        while !migrada.load(Ordering::SeqCst) {
                            std::thread::yield_now();
                        }
                    } else if i % 20 == 0 {
                        std::thread::yield_now();
                    }
                }
            });
        }
        while escritas.load(Ordering::SeqCst) < 100 {
            std::thread::yield_now();
        }
        let r = migra(&v, &n, TS);
        migrada.store(true, Ordering::SeqCst);
        assert!(matches!(r, Migracion::Fusionado { .. }), "{r:?}");
    });
    // Nada se pierde: la unión de lo que quedó en nuevo, .migrado y un viejo
    // recreado por escritores tardíos son las 800 líneas distintas, todas JSON.
    let mut vistas = std::collections::HashSet::new();
    for f in [n.clone(), migrado(&v), v.clone()] {
        let Ok(txt) = fs::read_to_string(&f) else {
            continue;
        };
        for l in txt.lines() {
            serde_json::from_str::<serde_json::Value>(l).unwrap_or_else(|e| panic!("{e}: {l}"));
            vistas.insert(l.to_string());
        }
    }
    assert_eq!(vistas.len(), hilos * por_hilo);
}
