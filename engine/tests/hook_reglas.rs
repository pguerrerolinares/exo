use exo::hook::evento::{Entorno, Evento, EventoKind, Harness};
use exo::hook::guards::reglas::{
    AUTORIDAD, CABECERA_GLOBAL, CABECERA_PROYECTO, ResultadoReglas, avisos, bloque_reglas,
    engine_stale, escribe_testigo, reglas_testigo, resuelve_reglas,
};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

struct Mundo {
    dir: tempfile::TempDir,
}

impl Mundo {
    fn nuevo() -> Self {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("kb/projects")).unwrap();
        Mundo { dir }
    }
    fn kb(&self) -> PathBuf {
        self.dir.path().join("kb")
    }
    fn state(&self) -> PathBuf {
        self.dir.path().join("state")
    }
    fn rules_dir(&self) -> PathBuf {
        self.state().join("exo-rules")
    }
    fn log(&self) -> String {
        std::fs::read_to_string(self.state().join("reflex-log.jsonl")).unwrap_or_default()
    }
    fn nota(&self, rel: &str, contenido: &str) {
        let p = self.kb().join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, contenido).unwrap();
    }
    fn repo(&self, nombre: &str) -> PathBuf {
        let r = self.dir.path().join(nombre);
        std::fs::create_dir_all(&r).unwrap();
        let s = Command::new("git")
            .arg("-C")
            .arg(&r)
            .args(["init", "-q"])
            .status()
            .unwrap();
        assert!(s.success());
        r
    }
    fn entorno(&self) -> Entorno {
        let state = self.state();
        let rutas = exo::state::Rutas::resuelve(self.dir.path(), |k| {
            (k == "EXO_STATE_DIR").then(|| state.clone().into_os_string())
        });
        Entorno {
            state_dir: rutas.state_dir.clone(),
            reflex_log: rutas.reflex_log.clone(),
            rutas,
            seams: exo::state::Seams::resuelve(|_| None),
            cdpath_definido: false,
            panic_guard: None,
        }
    }
}

fn evento(sid: &str, cwd: &Path, source: &str) -> Evento {
    Evento {
        harness: Harness::Claude,
        kind: EventoKind::SessionStart,
        session_id: sid.into(),
        cwd: cwd.to_string_lossy().into_owned(),
        agent_id: String::new(),
        agent_type: String::new(),
        source: source.into(),
        prompt: String::new(),
        crudo: "{}".into(),
        tool: None,
    }
}

fn reglas_md(n: usize) -> String {
    let items: String = (1..=n).map(|i| format!("- regla {i}\n")).collect();
    format!("## Reglas duras\n{items}")
}

fn resultado(
    status: &str,
    reason: &str,
    repo: Option<&str>,
    n: usize,
    ng: usize,
) -> ResultadoReglas {
    ResultadoReglas {
        bloque: String::new(),
        avisos: vec![],
        status: status.into(),
        reason: reason.into(),
        repo: repo.map(Into::into),
        n,
        n_global: ng,
        error: String::new(),
    }
}

fn ss(m: &Mundo, sid: &str) -> String {
    std::fs::read_to_string(m.rules_dir().join(format!("ss-{sid}"))).unwrap()
}

#[test]
fn testigo_forma() {
    let m = Mundo::nuevo();
    escribe_testigo(
        &m.rules_dir(),
        "s1",
        &resultado("ok", "", Some("exo"), 3, 1),
    );
    assert_eq!(
        ss(&m, "s1"),
        r#"{"status":"ok","n":3,"n_global":1,"repo":"exo"}"#
    );
    escribe_testigo(
        &m.rules_dir(),
        "s2",
        &resultado("skip", "sin_nota", None, 0, 0),
    );
    assert_eq!(
        ss(&m, "s2"),
        r#"{"status":"skip","reason":"sin_nota","n":0,"n_global":0,"repo":null}"#
    );
    assert!(!m.rules_dir().join("ss-s1.tmp").exists());
}

/// paridad-recall: session_id con '/': no se escribe ss (ni dentro de subdirectorios)
#[test]
fn sid_con_barra() {
    let m = Mundo::nuevo();
    std::fs::create_dir_all(m.rules_dir().join("ss-a")).unwrap();
    escribe_testigo(
        &m.rules_dir(),
        "a/b",
        &resultado("ok", "", Some("exo"), 1, 0),
    );
    assert_eq!(
        std::fs::read_dir(m.rules_dir().join("ss-a"))
            .unwrap()
            .count(),
        0
    );
    assert_eq!(std::fs::read_dir(m.rules_dir()).unwrap().count(), 1);
}

/// paridad-recall: sin session_id: se resuelve pero no se escribe ss-
#[test]
fn sin_session_id() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota("projects/foo.md", &reglas_md(2));
    let res = reglas_testigo(&m.entorno(), &m.kb(), &evento("", &r, "startup")).unwrap();
    assert_eq!((res.status.as_str(), res.n), ("ok", 2));
    assert!(!m.rules_dir().exists() || std::fs::read_dir(m.rules_dir()).unwrap().count() == 0);
}

#[test]
fn bloque_igual_a_register() {
    let ts = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../plugins/exo/hooks/register.ts"),
    )
    .unwrap();
    let re = regex::Regex::new(r#"const AUTORIDAD =\s*"((?:[^"\\]|\\.)*)""#).unwrap();
    let lit = &re.captures(&ts).expect("AUTORIDAD en register.ts")[1];
    assert_eq!(lit.replace("\\n", "\n"), AUTORIDAD);
    let re = regex::Regex::new(r#"lista\("([^"]+)""#).unwrap();
    let cabeceras: Vec<&str> = re
        .captures_iter(&ts)
        .map(|c| c.get(1).unwrap().as_str())
        .collect();
    assert_eq!(cabeceras, [CABECERA_GLOBAL, CABECERA_PROYECTO]);
}

#[test]
fn bloque_ensamblado() {
    let g = vec!["g1".to_string()];
    let p = vec!["p1".to_string(), "p2".to_string()];
    assert_eq!(
        bloque_reglas(&g, &p),
        format!(
            "## {CABECERA_GLOBAL}\n{AUTORIDAD}- g1\n\n## {CABECERA_PROYECTO}\n{AUTORIDAD}- p1\n- p2"
        )
    );
    assert_eq!(
        bloque_reglas(&[], &p),
        format!("## {CABECERA_PROYECTO}\n{AUTORIDAD}- p1\n- p2")
    );
    assert_eq!(bloque_reglas(&[], &[]), "");
}

/// paridad-recall: sin_excede_cap: truncated>0 ⇒ aviso de recorte sin atribuirlo al repo
#[test]
fn recorte_avisa() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota("projects/foo.md", &reglas_md(12));
    let res = resuelve_reglas(&m.kb(), &r, None);
    assert_eq!(res.n, 10);
    assert_eq!(
        res.bloque
            .lines()
            .filter(|l| l.starts_with("- regla"))
            .count(),
        10
    );
    assert_eq!(
        res.avisos,
        ["⚠ 2 reglas duras recortadas al máximo de 10 (corre «exo rules --check»)"]
    );
    assert!(!res.avisos[0].contains("foo"));
}

/// paridad-recall: sin_excede_cap: el engine ya no lo emite; sin aviso propio
#[test]
fn sin_excede_cap() {
    assert!(avisos("skip", "excede_cap", "foo", 0).is_empty());
    assert!(avisos("skip", "sin_nota", "foo", 0).is_empty());
    assert!(avisos("skip", "sin_seccion", "foo", 0).is_empty());
    assert!(avisos("ok", "", "foo", 0).is_empty());
}

#[test]
fn avisa_las_cuatro_reasons() {
    for reason in ["ambigua", "seccion_vacia", "error_engine", "engine_stale"] {
        assert_eq!(
            avisos("skip", reason, "foo", 0),
            [format!("⚠ sin reglas de proyecto para foo ({reason})")]
        );
    }
    // El aviso de reason solo con skip; con recorte van los dos, el de reason primero.
    assert!(avisos("ok", "ambigua", "foo", 0).is_empty());
    assert_eq!(avisos("skip", "ambigua", "foo", 3).len(), 2);
    assert!(engine_stale(Path::new("/x/foo")).avisos[0].ends_with("(engine_stale)"));
}

/// paridad-recall: ambigua_visible: systemMessage con el copy fijado + reflex-log + ss
#[test]
fn ambigua_avisa() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota("projects/foo.md", &reglas_md(1));
    m.nota("projects/x/foo — b.md", &reglas_md(1));
    let res = reglas_testigo(&m.entorno(), &m.kb(), &evento("s1", &r, "startup")).unwrap();
    assert_eq!(res.avisos, ["⚠ sin reglas de proyecto para foo (ambigua)"]);
    assert!(m.log().contains("reason=ambigua repo=foo"), "{}", m.log());
    assert_eq!(
        ss(&m, "s1"),
        r#"{"status":"skip","reason":"ambigua","n":0,"n_global":0,"repo":"foo"}"#
    );
}

/// paridad-recall: compact_no_resuelve: sin llamada a exo rules ni ss
#[test]
fn compact_no_resuelve() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota("projects/foo.md", &reglas_md(1));
    assert!(reglas_testigo(&m.entorno(), &m.kb(), &evento("s1", &r, "compact")).is_none());
    assert!(!m.rules_dir().exists());
    assert_eq!(m.log(), "");
}

/// Un `core/reglas-duras.md` que es un directorio fuerza el `Err` de `reglas::resuelve`.
fn mundo_con_error() -> (Mundo, PathBuf) {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    std::fs::create_dir_all(m.kb().join("core/reglas-duras.md")).unwrap();
    (m, r)
}

/// paridad-recall: error_engine_visible ($modo): systemMessage y ss skip/error_engine, el hook sigue emitiendo
#[test]
fn error_engine_visible() {
    let (m, r) = mundo_con_error();
    let res = reglas_testigo(&m.entorno(), &m.kb(), &evento("s1", &r, "startup")).unwrap();
    assert_eq!(
        res.avisos,
        ["⚠ sin reglas de proyecto para foo (error_engine)"]
    );
    assert_eq!(
        ss(&m, "s1"),
        r#"{"status":"skip","reason":"error_engine","n":0,"n_global":0,"repo":null}"#
    );
}

/// paridad-recall: error_engine_stderr: el stderr del engine viaja en el payload de project-rules-skip
#[test]
fn error_engine_stderr() {
    let (m, r) = mundo_con_error();
    reglas_testigo(&m.entorno(), &m.kb(), &evento("s1", &r, "startup")).unwrap();
    let log = m.log();
    assert!(log.contains("project-rules-skip"), "{log}");
    assert!(log.contains("reason=error_engine repo=- err="), "{log}");
    assert!(log.contains("reglas-duras.md"), "{log}");
}

/// paridad-recall: poda_7_dias: ss de 8 días fuera, de 1 día se queda
#[test]
fn poda_7_dias() {
    let m = Mundo::nuevo();
    let rd = m.rules_dir();
    std::fs::create_dir_all(&rd).unwrap();
    let hace = |dias: u64, f: &str| {
        let p = rd.join(f);
        std::fs::write(&p, "{}").unwrap();
        let t = SystemTime::now() - Duration::from_secs(dias * 86_400);
        std::fs::File::options()
            .write(true)
            .open(&p)
            .unwrap()
            .set_modified(t)
            .unwrap();
    };
    hace(8, "ss-viejo");
    hace(8, "hb-viejo");
    hace(1, "ss-nuevo");
    hace(1, "hb-nuevo");
    hace(30, "otro-viejo");
    escribe_testigo(&rd, "s1", &resultado("ok", "", Some("exo"), 1, 0));
    assert!(!rd.join("ss-viejo").exists());
    assert!(!rd.join("hb-viejo").exists());
    assert!(rd.join("ss-nuevo").exists());
    assert!(rd.join("hb-nuevo").exists());
    assert!(rd.join("otro-viejo").exists());
    assert!(rd.join("ss-s1").exists());
}

/// paridad-recall: reglas_ok_silencioso: ss ok n=3, sin systemMessage, additionalContext intacto, cwd del input
#[test]
fn reglas_ok_silencioso() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota("projects/foo.md", &reglas_md(3));
    // El cwd del proceso es otro (el del crate): el repo sale del cwd del evento.
    let res = reglas_testigo(&m.entorno(), &m.kb(), &evento("s1", &r, "startup")).unwrap();
    assert!(res.avisos.is_empty());
    assert_eq!(res.repo.as_deref(), Some("foo"));
    assert_eq!(
        ss(&m, "s1"),
        r#"{"status":"ok","n":3,"n_global":0,"repo":"foo"}"#
    );
    assert_eq!(m.log(), "");
}

/// paridad-recall: sin_indice_resuelve_reglas: engine OK e índice ausente ⇒ exo rules llamado y ss escrito
#[test]
fn sin_indice_resuelve_reglas() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota("projects/foo.md", &reglas_md(1));
    assert!(!m.state().join("index.db").exists());
    reglas_testigo(&m.entorno(), &m.kb(), &evento("s1", &r, "startup")).unwrap();
    assert_eq!(
        ss(&m, "s1"),
        r#"{"status":"ok","n":1,"n_global":0,"repo":"foo"}"#
    );
}

/// paridad-recall: sin_seccion_solo_log: sin systemMessage, project-rules-skip en reflex-log
#[test]
fn sin_seccion_solo_log() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota("projects/foo.md", "# foo\nsin sección\n");
    let res = reglas_testigo(&m.entorno(), &m.kb(), &evento("s1", &r, "startup")).unwrap();
    assert!(res.avisos.is_empty());
    assert!(
        m.log().contains("reason=sin_seccion repo=foo"),
        "{}",
        m.log()
    );
    assert!(!m.log().contains("err="));
}

/// paridad-recall: ss_cuenta_global: skip + 2 globales ⇒ n=2 n_global=2
#[test]
fn ss_cuenta_global() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota("core/reglas-duras.md", "## Reglas duras\n- g1\n- g2\n");
    let res = reglas_testigo(&m.entorno(), &m.kb(), &evento("s1", &r, "startup")).unwrap();
    assert_eq!((res.n, res.n_global), (2, 2));
    assert_eq!(
        ss(&m, "s1"),
        r#"{"status":"skip","reason":"sin_nota","n":2,"n_global":2,"repo":"foo"}"#
    );
    assert!(res.bloque.starts_with(&format!("## {CABECERA_GLOBAL}")));
}

#[test]
fn skip_n_cero() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    let res = resuelve_reglas(&m.kb(), &r, None);
    assert_eq!((res.status.as_str(), res.n, res.n_global), ("skip", 0, 0));
    assert_eq!(res.bloque, "");
}

/// paridad-recall: ss_suma_proyecto_y_global: ok con 3 de proyecto + 2 globales ⇒ n=5
#[test]
fn n_suma() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota("projects/foo.md", &reglas_md(3));
    m.nota("core/reglas-duras.md", "## Reglas duras\n- g1\n- g2\n");
    let res = reglas_testigo(&m.entorno(), &m.kb(), &evento("s1", &r, "startup")).unwrap();
    assert_eq!((res.n, res.n_global), (5, 2));
    assert_eq!(
        ss(&m, "s1"),
        r#"{"status":"ok","n":5,"n_global":2,"repo":"foo"}"#
    );
}
