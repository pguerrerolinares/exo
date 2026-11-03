use exo::state::{Migracion, Rutas, migra, migra_todo};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
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
fn nuevo_creado_en_la_ventana_del_link_sobrevive() {
    let d = tempfile::tempdir().unwrap();
    let (v, n) = (d.path().join("v"), d.path().join("n"));
    fs::write(&v, "A\n").unwrap();
    let r = exo::state::migra_con(&v, &n, TS, || fs::write(&n, "B\n").unwrap(), || {});
    assert_eq!(
        r,
        Migracion::Fusionado {
            viejo_a: migrado(&v)
        }
    );
    assert_eq!(fs::read_to_string(&n).unwrap(), "B\nA\n");
}

#[test]
fn dos_migradores_no_duplican() {
    for _ in 0..2000 {
        let d = tempfile::tempdir().unwrap();
        let (v, n) = (d.path().join("v"), d.path().join("n"));
        let lineas = |p: &str| {
            (0..50)
                .map(|i| format!("{{\"{p}\":{i}}}\n"))
                .collect::<String>()
        };
        fs::write(&v, lineas("v")).unwrap();
        fs::write(&n, lineas("n")).unwrap();
        let barrera = std::sync::Barrier::new(2);
        std::thread::scope(|sc| {
            for _ in 0..2 {
                sc.spawn(|| {
                    barrera.wait();
                    migra(&v, &n, TS);
                });
            }
        });
        let txt = fs::read_to_string(&n).unwrap();
        assert_eq!(txt.lines().count(), 100, "cada línea una vez");
    }
}

#[test]
fn conservacion_concurrente() {
    let d = tempfile::tempdir().unwrap();
    let (v, n) = (d.path().join("v"), d.path().join("n"));
    let linea = |o: &str, h: usize, i: usize| format!("{{\"o\":\"{o}\",\"h\":{h},\"i\":{i}}}");
    let previo: Vec<String> = (0..3000).map(|i| linea("previo", 0, i)).collect();
    fs::write(&v, previo.join("\n") + "\n").unwrap();
    fs::write(&n, "").unwrap();

    let por_hilo = 1000usize;
    let hechas = Mutex::new(Vec::<(char, String)>::new());
    let tras_migra = AtomicUsize::new(0);
    let migrada = AtomicBool::new(false);
    let mut previas_viejo = vec![];
    std::thread::scope(|sc| {
        for (destino, c) in [(&v, 'v'), (&v, 'v'), (&n, 'n'), (&n, 'n')]
            .into_iter()
            .enumerate()
            .map(|(h, (p, c))| ((h, p), c))
        {
            let (h, ruta) = destino;
            let (hechas, tras_migra, migrada) = (&hechas, &tras_migra, &migrada);
            sc.spawn(move || {
                for i in 0..por_hilo {
                    // La última línea espera a que `migra` vuelva: garantiza solape real.
                    if i == por_hilo - 1 {
                        while !migrada.load(Ordering::SeqCst) {
                            std::thread::yield_now();
                        }
                    }
                    let l = linea(&c.to_string(), h, i);
                    let mut f = fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(ruta)
                        .unwrap();
                    f.write_all(format!("{l}\n").as_bytes()).unwrap();
                    if migrada.load(Ordering::SeqCst) {
                        tras_migra.fetch_add(1, Ordering::SeqCst);
                    }
                    hechas.lock().unwrap().push((c, l));
                }
            });
        }
        while hechas.lock().unwrap().len() < 300 {
            std::thread::yield_now();
        }
        // Todo lo del viejo terminado antes de arrancar `migra` tiene que acabar en el nuevo.
        previas_viejo = hechas
            .lock()
            .unwrap()
            .iter()
            .filter(|(c, _)| *c == 'v')
            .map(|(_, l)| l.clone())
            .collect();
        let r = migra(&v, &n, TS);
        migrada.store(true, Ordering::SeqCst);
        assert!(matches!(r, Migracion::Fusionado { .. }), "{r:?}");
    });
    assert!(
        tras_migra.load(Ordering::SeqCst) >= 4,
        "hubo escrituras después de migra"
    );

    let leer = |p: &Path| fs::read_to_string(p).unwrap_or_default();
    let en_nuevo: std::collections::HashSet<String> = leer(&n).lines().map(String::from).collect();
    for l in leer(&n)
        .lines()
        .chain(leer(&migrado(&v)).lines())
        .chain(leer(&v).lines())
    {
        serde_json::from_str::<serde_json::Value>(l).unwrap_or_else(|e| panic!("{e}: {l}"));
    }
    for l in previo.iter().chain(previas_viejo.iter()) {
        assert!(
            en_nuevo.contains(l),
            "línea previa a migra ausente del nuevo: {l}"
        );
    }
    for (c, l) in hechas.lock().unwrap().iter().filter(|(c, _)| *c == 'n') {
        assert!(
            en_nuevo.contains(l),
            "línea escrita al nuevo perdida ({c}): {l}"
        );
    }
    let todas: std::collections::HashSet<String> = leer(&n)
        .lines()
        .chain(leer(&migrado(&v)).lines())
        .chain(leer(&v).lines())
        .map(String::from)
        .collect();
    assert_eq!(todas.len(), previo.len() + 4 * por_hilo);
}

#[test]
fn migrado_previo_con_contenido_sobrevive_y_lo_nuevo_va_a_menos_1() {
    let d = tempfile::tempdir().unwrap();
    let (v, n) = (d.path().join("v"), d.path().join("n"));
    fs::write(&n, "B\n").unwrap();
    fs::write(&v, "V1\n").unwrap();
    fs::write(migrado(&v), "X\n").unwrap();
    migra(&v, &n, TS);
    assert_eq!(fs::read_to_string(migrado(&v)).unwrap(), "X\n");
    let menos1 = format!("{}-1", migrado(&v).display());
    assert_eq!(fs::read_to_string(menos1).unwrap(), "V1\n");
}

#[test]
fn fallo_del_append_devuelve_el_historico_al_viejo() {
    let d = tempfile::tempdir().unwrap();
    let (v, n) = (d.path().join("v"), d.path().join("n"));
    fs::write(&v, "A\n").unwrap();
    fs::create_dir(&n).unwrap(); // el append no puede abrir un directorio
    assert!(matches!(migra(&v, &n, TS), Migracion::Fallo(_)));
    assert_eq!(fs::read_to_string(&v).unwrap(), "A\n");
    assert!(!migrado(&v).exists());
}

#[test]
fn segundo_migrador_tras_el_reclamo_no_anexa() {
    let d = tempfile::tempdir().unwrap();
    let (v, n) = (d.path().join("v"), d.path().join("n"));
    fs::write(&v, "A\n").unwrap();
    fs::write(&n, "B\n").unwrap();
    let mut segundo = None;
    let r = exo::state::migra_con(&v, &n, TS, || segundo = Some(migra(&v, &n, TS)), || {});
    assert_eq!(segundo, Some(Migracion::Nada));
    assert!(matches!(r, Migracion::Fusionado { .. }));
    assert_eq!(fs::read_to_string(&n).unwrap(), "B\nA\n");
}

#[test]
fn viejo_recreado_durante_la_migracion_conserva_sus_lineas() {
    let d = tempfile::tempdir().unwrap();
    let (v, n) = (d.path().join("v"), d.path().join("n"));
    fs::write(&v, "A\n").unwrap();
    fs::write(&n, "B\n").unwrap();
    let r = exo::state::migra_con(&v, &n, TS, || fs::write(&v, "H\n").unwrap(), || {});
    assert!(matches!(r, Migracion::Fusionado { .. }));
    assert_eq!(
        fs::read_to_string(&v).unwrap(),
        "H\n",
        "el recreado queda intacto"
    );
    assert_eq!(fs::read_to_string(&n).unwrap(), "B\nA\n");
}

#[test]
fn viejo_recreado_en_solo_viejo_conserva_sus_lineas() {
    let d = tempfile::tempdir().unwrap();
    let (v, n) = (d.path().join("v"), d.path().join("n"));
    fs::write(&v, "A\n").unwrap();
    let r = exo::state::migra_con(&v, &n, TS, || fs::write(&v, "H\n").unwrap(), || {});
    assert_eq!(r, Migracion::Renombrado);
    assert_eq!(fs::read_to_string(&v).unwrap(), "H\n");
    assert_eq!(fs::read_to_string(&n).unwrap(), "A\n");
}

#[test]
fn fallo_del_append_con_viejo_recreado_no_lo_pisa_y_nombra_el_reclamado() {
    let d = tempfile::tempdir().unwrap();
    let (v, n) = (d.path().join("v"), d.path().join("n"));
    fs::write(&v, "A\n").unwrap();
    fs::create_dir(&n).unwrap();
    let r = exo::state::migra_con(&v, &n, TS, || {}, || fs::write(&v, "H\n").unwrap());
    let Migracion::Fallo(m) = r else {
        panic!("{r:?}")
    };
    assert!(m.contains(".migrando-"), "nombra el reclamado: {m}");
    assert_eq!(
        fs::read_to_string(&v).unwrap(),
        "H\n",
        "el recreado sigue intacto"
    );
    let reclamado = fs::read_dir(d.path())
        .unwrap()
        .flatten()
        .find(|e| e.file_name().to_string_lossy().contains(".migrando-"))
        .expect("el reclamado sigue en disco");
    assert_eq!(fs::read_to_string(reclamado.path()).unwrap(), "A\n");
}
