mod common;

use exo::hook::evento::{Entorno, Evento, EventoKind, Harness};
use exo::hook::guards::recall::{
    FALLBACK, Fragmentos, compone_claude, reafirmacion, recall_contrato,
};
use exo::indexer::indexa;
use exo::state::{Rutas, Seams};
use std::path::{Path, PathBuf};
use std::process::Command;

const CABECERA_REAFIRMA: &str =
    "--- Reglas reforzadas tras compactacion (dispararon esta sesion) ---";
const PIN_GIT: &str = "- nunca `cd X && git ...`: usa `git -C X`.";
const PIN_VERIFY: &str = "- verifica (corre el cambio) antes de afirmar exito.";

fn git(dir: &Path, args: &[&str]) {
    let st = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(st.success(), "git {args:?}");
}

/// KB con una sola nota `<nombre>/core/core-index` (más una `core/otra`) cuyo cuerpo se pasa.
fn kb_con_core_index(nombre: &str, cuerpo: &str) -> tempfile::TempDir {
    let kb = tempfile::tempdir().unwrap();
    git(kb.path(), &["init", "-q"]);
    git(kb.path(), &["config", "user.email", "t@exo.local"]);
    git(kb.path(), &["config", "user.name", "exo-test"]);
    std::fs::create_dir_all(kb.path().join("core")).unwrap();
    let nota = |perm: &str, tit: &str, cuerpo: &str| {
        format!("---\npermalink: {perm}\ntitle: {tit}\ntier: core\n---\n{cuerpo}\n")
    };
    std::fs::write(
        kb.path().join("core/core-index.md"),
        nota(&format!("{nombre}/core/core-index"), "Core Index", cuerpo),
    )
    .unwrap();
    std::fs::write(
        kb.path().join("core/otra.md"),
        nota(
            &format!("{nombre}/core/otra"),
            "Otra",
            "## Contrato de memoria\nCONTRATO-DE-OTRA",
        ),
    )
    .unwrap();
    git(kb.path(), &["add", "core"]);
    git(kb.path(), &["commit", "-q", "-m", "nota"]);
    kb
}

const MULTI: &str = "intro\n\n## Doctrina compacta\n- doctrina-x\n\n## Contrato de memoria\nCONTRATO-LINEA-1\nCONTRATO-LINEA-2\n\n## Cores\n- core-x\n\n## Actividad reciente\n- act-x";

struct Montaje {
    kb: tempfile::TempDir,
    db_dir: tempfile::TempDir,
    _estado: tempfile::TempDir,
    nombre: String,
    en: Entorno,
}

impl Montaje {
    fn db(&self) -> PathBuf {
        self.db_dir.path().join("exo.db")
    }
}

fn entorno(estado: &Path, seams: Seams) -> Entorno {
    let rutas = Rutas::resuelve(estado, |k| match k {
        "EXO_STATE_DIR" => Some(estado.into()),
        _ => None,
    });
    Entorno {
        state_dir: rutas.state_dir.clone(),
        reflex_log: rutas.reflex_log.clone(),
        rutas,
        seams,
        cdpath_definido: false,
        panic_guard: None,
    }
}

/// Monta KB + índice (indexar necesita config, que se suelta al volver) y devuelve un
/// `Entorno` de seams explícitos: el hook no debe necesitar config ni env para estos casos.
fn con_montaje<T>(nombre: &str, cuerpo: &str, f: impl FnOnce(&mut Montaje) -> T) -> T {
    let kb = kb_con_core_index(nombre, cuerpo);
    let db_dir = tempfile::tempdir().unwrap();
    let db = db_dir.path().join("exo.db");
    let estado = tempfile::tempdir().unwrap();
    common::con_config(kb.path(), nombre, &db, || indexa(kb.path(), &db).unwrap());
    let mut seams = Seams::resuelve(|_| None);
    seams.exo_index = Some(db);
    seams.exo_kb = Some(kb.path().to_path_buf());
    seams.exo_kb_name = Some(nombre.to_string());
    let en = entorno(estado.path(), seams);
    let mut m = Montaje {
        kb,
        db_dir,
        _estado: estado,
        nombre: nombre.into(),
        en,
    };
    f(&mut m)
}

fn evento(sid: &str, source: &str) -> Evento {
    Evento {
        harness: Harness::Claude,
        kind: EventoKind::SessionStart,
        session_id: sid.into(),
        cwd: String::new(),
        agent_id: String::new(),
        agent_type: String::new(),
        crudo: "{}".into(),
        tool: None,
        source: source.into(),
        prompt: String::new(),
    }
}

fn escribe_log(en: &Entorno, lineas: &[(&str, &str)]) {
    let mut s = String::new();
    for (reflex, sid) in lineas {
        s.push_str(&format!(
            "{{\"ts\":\"t\",\"reflex\":\"{reflex}\",\"session_id\":\"{sid}\",\"agent_id\":\"\",\"agent_type\":\"\",\"tool\":\"Bash\",\"payload\":\"x\"}}\n"
        ));
    }
    std::fs::create_dir_all(en.reflex_log.parent().unwrap()).unwrap();
    std::fs::write(&en.reflex_log, s).unwrap();
}

fn entorno_vacio() -> (tempfile::TempDir, Entorno) {
    let d = tempfile::tempdir().unwrap();
    let en = entorno(d.path(), Seams::resuelve(|_| None));
    (d, en)
}

/// Contrato de ~2 KB (40 líneas de 49 B) bajo su encabezado.
fn contrato_grande() -> String {
    let mut s = String::from("## Contrato de memoria\n");
    for i in 0..40 {
        s.push_str(&format!("{i:049}\n"));
    }
    s.push_str("\n## Cores\n- core-x");
    s
}

/// paridad-recall: camino feliz: bloque real inyectado, sin fallback ni log de degradación
/// paridad-recall: bloque_empieza_por_contrato: additionalContext arranca en la sección de contrato
/// paridad-recall: bloque_solo_contrato: sin doctrina, cores ni actividad reciente
#[test]
fn contrato_extrae_seccion() {
    con_montaje("kb-test", MULTI, |m| {
        let (txt, razones) = recall_contrato(&evento("s", ""), &m.en);
        assert!(txt.starts_with("## Contrato de memoria"), "{txt}");
        assert!(txt.contains("CONTRATO-LINEA-1\nCONTRATO-LINEA-2"), "{txt}");
        for ajeno in [
            "## Cores",
            "## Doctrina compacta",
            "Actividad reciente",
            "doctrina-x",
            "core-x",
            "act-x",
            "Recall exo",
        ] {
            assert!(!txt.contains(ajeno), "{ajeno} no debía estar: {txt}");
        }
        assert_eq!(
            txt,
            "## Contrato de memoria\nCONTRATO-LINEA-1\nCONTRATO-LINEA-2"
        );
        assert!(razones.is_empty(), "{razones:?}");
    });
}

/// paridad-recall: bloque_sin_encabezado: sin '## Contrato de memoria' el bloque servido va entero
#[test]
fn sin_encabezado_sirve_bloque_entero() {
    con_montaje(
        "kb-test",
        "Contrato de memoria: una sola linea sin encabezado.",
        |m| {
            let (txt, razones) = recall_contrato(&evento("s", ""), &m.en);
            assert!(txt.contains("una sola linea sin encabezado"), "{txt}");
            assert!(
                txt.starts_with("=== Recall exo"),
                "bloque entero, cabecera incluida: {txt}"
            );
            assert!(razones.is_empty(), "{razones:?}");
        },
    );
}

#[test]
fn sin_contrato_fallback() {
    con_montaje("kb-test", "una nota sin la sección esperada", |m| {
        let (txt, razones) = recall_contrato(&evento("s", ""), &m.en);
        assert_eq!(txt, FALLBACK);
        assert_eq!(razones, ["no-contract"]);
    });
}

/// paridad-recall: no-index: EXO_INDEX ausente ⇒ recall-fallback reason=no-index
#[test]
fn sin_indice() {
    con_montaje("kb-test", MULTI, |m| {
        m.en.seams.exo_index = Some(m.db_dir.path().join("no-existe.db"));
        let (txt, razones) = recall_contrato(&evento("s", ""), &m.en);
        assert_eq!(txt, FALLBACK);
        assert_eq!(razones, ["no-index"]);
    });
}

/// paridad-recall: no-config: EXO_BIN sin subcomando config ⇒ recall-fallback reason=no-config
#[test]
fn sin_config_ni_nombre() {
    con_montaje("kb-test", MULTI, |m| {
        m.en.seams.exo_kb_name = None;
        // Config ilegible: ni seam de nombre ni `nombre_kb()`; la nota cae a `core/core-index` pelado.
        common::con_config_texto("esto no es toml [", || {
            let (txt, razones) = recall_contrato(&evento("s", ""), &m.en);
            assert_eq!(txt, FALLBACK);
            assert_eq!(razones, ["no-config", "empty"]);
        });
    });
}

#[test]
fn seam_nota() {
    con_montaje("kb-test", MULTI, |m| {
        // EXO_RECALL_NOTA pisa la nota por defecto.
        m.en.seams.recall_nota = Some("kb-test/core/otra".into());
        let (txt, razones) = recall_contrato(&evento("s", ""), &m.en);
        assert_eq!(txt, "## Contrato de memoria\nCONTRATO-DE-OTRA");
        assert!(razones.is_empty(), "{razones:?}");
    });
    // EXO_KB_NAME cambia el prefijo de `<kb>/core/core-index`.
    con_montaje("otro-proyecto", MULTI, |m| {
        let (txt, razones) = recall_contrato(&evento("s", ""), &m.en);
        assert!(txt.contains("CONTRATO-LINEA-2"), "{txt}");
        assert!(razones.is_empty(), "{razones:?}");
        m.en.seams.exo_kb_name = Some("kb-test".into());
        let (txt, razones) = recall_contrato(&evento("s", ""), &m.en);
        assert_eq!(txt, FALLBACK, "con otro prefijo la nota no existe");
        assert_eq!(razones, ["empty"]);
    });
}

/// paridad-recall: recall_cap_al_engine: EXO_RECALL_CAP llega como --cap-bytes
#[test]
fn truncado() {
    con_montaje("kb-test", &contrato_grande(), |m| {
        m.en.seams.recall_cap = 600;
        let (txt, razones) = recall_contrato(&evento("s", ""), &m.en);
        assert!(txt.starts_with("## Contrato de memoria\n"), "{txt}");
        assert!(txt.contains(&format!("{:049}", 0)), "{txt}");
        assert!(
            !txt.contains(&format!("{:049}", 39)),
            "el cap tenía que cortar: {txt}"
        );
        assert_eq!(razones, ["truncated"]);
        // Con cap holgado (el de verdad llega al engine) no hay truncado.
        m.en.seams.recall_cap = 6144;
        let (txt, razones) = recall_contrato(&evento("s", ""), &m.en);
        assert!(txt.contains(&format!("{:049}", 39)));
        assert!(razones.is_empty(), "{razones:?}");
    });
}

#[test]
fn cap_y_limite_cero_no_panican() {
    con_montaje("kb-test", MULTI, |m| {
        m.en.seams.recall_cap = 0;
        let (txt, razones) = recall_contrato(&evento("s", ""), &m.en);
        assert_eq!(txt, FALLBACK);
        assert_eq!(razones, ["truncated", "empty"]);
        m.en.seams.recall_cap = 6144;
        m.en.seams.recall_limite = 0;
        let (txt, _) = recall_contrato(&evento("s", ""), &m.en);
        assert!(txt.contains("CONTRATO-LINEA-1"), "{txt}");
    });
}

#[test]
fn contrato_ignora_el_entorno_del_proceso() {
    // `EXO_DB` es de la CLI: el hook usa el seam `exo_index` y luego la config.
    con_montaje("kb-test", MULTI, |m| {
        let (_, razones) = recall_contrato(&evento("s", ""), &m.en);
        assert!(razones.is_empty());
        m.en.seams.exo_index = None;
        m.en.seams.exo_kb = None;
        m.en.seams.exo_kb_name = None;
        // Sin seams: rama de config (`con_config` apunta `[index] db` al índice real).
        let (kb, db, nombre) = (m.kb.path().to_path_buf(), m.db(), m.nombre.clone());
        common::con_config(&kb, &nombre, &db, || {
            let (txt, razones) = recall_contrato(&evento("s", ""), &m.en);
            assert!(txt.contains("CONTRATO-LINEA-1"), "{txt}");
            assert!(razones.is_empty(), "{razones:?}");
        });
    });
}

#[test]
fn reafirma_git_c() {
    let (_d, en) = entorno_vacio();
    escribe_log(&en, &[("git-c-rewrite", "s1")]);
    assert_eq!(
        reafirmacion("s1", &en),
        format!("\n\n{CABECERA_REAFIRMA}\n{PIN_GIT}")
    );
}

#[test]
fn reafirma_todos_los_reflejos_y_dedup() {
    let (_d, en) = entorno_vacio();
    // git-c y git-c-rewrite comparten pin (una sola línea); los ids ajenos no pintan nada.
    escribe_log(
        &en,
        &[
            ("verify-before-done", "s1"),
            ("git-c", "s1"),
            ("zero-residuo", "s1"),
            ("git-c-rewrite", "s1"),
        ],
    );
    assert_eq!(
        reafirmacion("s1", &en),
        format!("\n\n{CABECERA_REAFIRMA}\n{PIN_GIT}\n{PIN_VERIFY}")
    );
    escribe_log(&en, &[("verify-before-commit", "s1")]);
    assert_eq!(
        reafirmacion("s1", &en),
        format!("\n\n{CABECERA_REAFIRMA}\n{PIN_VERIFY}")
    );
}

#[test]
fn reafirma_sin_disparos_vacia() {
    let (_d, en) = entorno_vacio();
    // Sin log, sin reflejos con pin, y sid vacío: nada.
    assert_eq!(reafirmacion("s1", &en), "");
    escribe_log(
        &en,
        &[("zero-residuo", "s1"), ("compact", "s1"), ("git-c", "otra")],
    );
    assert_eq!(reafirmacion("s1", &en), "");
    escribe_log(&en, &[("git-c", "")]);
    assert_eq!(reafirmacion("", &en), "");
}

#[test]
fn reafirma_respeta_reflex_log_file() {
    let (d, mut en) = entorno_vacio();
    // `REFLEX_LOG_FILE` fuera de `state_dir`: el bug de exo-recall.sh:161 leía siempre ~/.claude.
    let aparte: PathBuf = d.path().join("otro-sitio/log.jsonl");
    en.reflex_log = aparte;
    escribe_log(&en, &[("git-c", "s1")]);
    assert!(!en.rutas.state_dir.join("reflex-log.jsonl").exists());
    assert!(reafirmacion("s1", &en).contains(PIN_GIT));
}

#[test]
fn reafirma_otra_sesion() {
    let (_d, en) = entorno_vacio();
    escribe_log(&en, &[("git-c", "otra"), ("verify-before-commit", "s1")]);
    let r = reafirmacion("s1", &en);
    assert!(r.contains(PIN_VERIFY) && !r.contains("git -C"), "{r}");
}

/// Log de 2.500 líneas: un git-c de `sess-x` FUERA de la ventana y un verify DENTRO.
fn log_h5(en: &Entorno) {
    let mut v: Vec<(String, String)> = vec![("git-c".into(), "sess-x".into())];
    v.extend((0..2498).map(|i| ("zero-residuo".to_string(), format!("otra-{i}"))));
    v.push(("verify-before-commit".into(), "sess-x".into()));
    let refs: Vec<(&str, &str)> = v.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    escribe_log(en, &refs);
}

/// paridad-recall: H5: refuerza lo disparado dentro de la ventana
#[test]
fn h5_refuerza_dentro_de_la_ventana() {
    let (_d, en) = entorno_vacio();
    log_h5(&en);
    assert!(reafirmacion("sess-x", &en).contains(PIN_VERIFY));
}

/// paridad-recall: H5: no escanea más allá de la ventana de 2000 líneas
#[test]
fn h5_no_escanea_mas_alla() {
    let (_d, mut en) = entorno_vacio();
    log_h5(&en);
    assert!(!reafirmacion("sess-x", &en).contains("git -C X"));
    // La ventana es la seam: con 3000 el git-c vuelve a entrar; con 0 no entra nada.
    en.seams.compact_lineas = 3000;
    assert!(reafirmacion("sess-x", &en).contains("git -C X"));
    en.seams.compact_lineas = 0;
    assert_eq!(reafirmacion("sess-x", &en), "");
}

/// paridad-recall: golden: $nombre sin cambios
#[test]
fn golden_reafirma_compact() {
    let (_d, en) = entorno_vacio();
    log_h5(&en);
    let golden = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../plugins/exo/scripts/testdata/golden-exo-recall/reafirma-compact.txt"),
    )
    .unwrap();
    let v: serde_json::Value = serde_json::from_str(&golden).unwrap();
    let esperado = v["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    let fr = Fragmentos {
        contrato: FALLBACK.into(),
        reafirmacion: reafirmacion("sess-x", &en),
        ..Fragmentos::default()
    };
    assert_eq!(compone_claude(&fr, 1500).0, esperado);
}

/// paridad-recall: bloque_cap_1500: contrato de ~2 KB ⇒ bloque <= 1500 B y evento recall-truncated
#[test]
fn bloque_cap_1500() {
    con_montaje("kb-test", &contrato_grande(), |m| {
        let (contrato, razones) = recall_contrato(&evento("s", ""), &m.en);
        assert!(razones.is_empty(), "{razones:?}");
        let fr = Fragmentos {
            contrato,
            ..Fragmentos::default()
        };
        let (txt, antes) = compone_claude(&fr, m.en.seams.bloque_cap);
        assert!(txt.len() <= 1500 && txt.len() > 1000, "{}", txt.len());
        assert!(antes.is_some_and(|b| b > 1500), "{antes:?}");
    });
}

/// paridad-recall: bloque_cap_configurable: RECALL_BLOQUE_CAP baja el cap
#[test]
fn bloque_cap_configurable() {
    let fr = Fragmentos {
        contrato: contrato_grande(),
        ..Fragmentos::default()
    };
    let (txt, antes) = compone_claude(&fr, 300);
    assert!(txt.len() <= 300 && txt.len() > 200, "{}", txt.len());
    assert!(antes.is_some());
    let (txt, antes) = compone_claude(&fr, 0);
    assert_eq!(txt, "");
    assert!(antes.is_some());
}

#[test]
fn cap_no_parte_utf8_ni_recorta_si_cabe() {
    let fr = Fragmentos {
        contrato: "ñ".repeat(10),
        ..Fragmentos::default()
    };
    // 20 B; cap 5 cae en mitad de una ñ: se descarta el byte partido.
    let (txt, antes) = compone_claude(&fr, 5);
    assert_eq!(txt, "ññ");
    assert_eq!(antes, Some(20));
    let (txt, antes) = compone_claude(&fr, 20);
    assert_eq!(txt.len(), 20);
    assert_eq!(antes, None);
}

#[test]
fn avisos_van_tras_el_contrato_y_caen_primero() {
    let fr = Fragmentos {
        contrato: "CONTRATO".into(),
        avisos: vec!["AVISO-1".into(), "AVISO-2".into()],
        ..Fragmentos::default()
    };
    assert_eq!(
        compone_claude(&fr, 1500).0,
        "CONTRATO\n\nAVISO-1\n\nAVISO-2"
    );
    assert_eq!(compone_claude(&fr, 12).0, "CONTRATO\n\nAV");
}

/// paridad-recall: compact_fuera_del_cap: cap actuó sobre el contrato y la reafirmación sigue presente
#[test]
fn compact_fuera_del_cap() {
    let (_d, en) = entorno_vacio();
    escribe_log(&en, &[("git-c", "c5")]);
    let fr = Fragmentos {
        contrato: contrato_grande(),
        reafirmacion: reafirmacion("c5", &en),
        ..Fragmentos::default()
    };
    let (txt, antes) = compone_claude(&fr, 1500);
    let (parte_contrato, _) = txt.split_once("\n\n--- Reglas reforzadas").unwrap();
    assert!(parte_contrato.len() <= 1500, "{}", parte_contrato.len());
    assert!(txt.len() > 1500);
    assert!(txt.contains(CABECERA_REAFIRMA) && txt.contains(PIN_GIT));
    assert!(antes.is_some());
}

/// Hijo de `truncado_no_escribe_stderr`: solo hace algo si lo lanza el padre.
#[test]
fn hijo_truncado_stderr() {
    if std::env::var_os("HOOK_RECALL_HIJO").is_none() {
        return;
    }
    con_montaje("kb-test", &contrato_grande(), |m| {
        m.en.seams.recall_cap = 600;
        let (_, razones) = recall_contrato(&evento("s", ""), &m.en);
        assert_eq!(razones, ["truncated"]);
    });
}

#[test]
fn truncado_no_escribe_stderr() {
    let out = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "hijo_truncado_stderr", "--nocapture"])
        .env("HOOK_RECALL_HIJO", "1")
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "el hijo falló: {err}");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("hijo_truncado_stderr ... ok"),
        "el hijo no corrió"
    );
    assert!(!err.contains("aviso"), "stderr del hook no vacío: {err}");
}
