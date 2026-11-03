use exo::hook::guards::recall_avisos::{
    GateTimeout, corre_con_deadline, gate_pendiente, superpowers,
};
use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use tempfile::TempDir;

const NOTA: &str = "projects/a — b (c).md";
const TRAILER: &str = "Gate: projects/a — b (c).md bytes=11000 limit=12500 routed=bitacora";
const AVISO: &str = "AVISO exo: gate pendiente en projects/a — b (c).md, sin auto-distill que lo resuelva (corre /distill)";

// ---------- superpowers ----------

struct Sp {
    _d: TempDir,
    home: PathBuf,
    proy: PathBuf,
}

fn sp() -> Sp {
    let d = tempfile::tempdir().unwrap();
    let home = d.path().join("home");
    let proy = d.path().join("proy");
    std::fs::create_dir_all(home.join(".claude")).unwrap();
    std::fs::create_dir_all(proy.join(".claude")).unwrap();
    Sp { _d: d, home, proy }
}

impl Sp {
    fn user(&self, s: &str) {
        std::fs::write(self.home.join(".claude/settings.json"), s).unwrap();
    }
    fn proyecto(&self, s: &str) {
        std::fs::write(self.proy.join(".claude/settings.json"), s).unwrap();
    }
    fn local(&self, s: &str) {
        std::fs::write(self.proy.join(".claude/settings.local.json"), s).unwrap();
    }
    fn aviso(&self) -> Option<String> {
        superpowers(&self.home, &self.proy)
    }
}

/// paridad-recall: sin settings: no avisa
#[test]
fn superpowers_sin_settings() {
    assert_eq!(sp().aviso(), None);
}

/// paridad-recall: user scope habilitado (JSON multilínea): avisa con el comando
#[test]
fn superpowers_user_multilinea() {
    let s = sp();
    s.user(
        "{\n  \"enabledPlugins\": {\n    \"superpowers@claude-plugins-official\": true\n  }\n}\n",
    );
    assert_eq!(
        s.aviso().unwrap(),
        "AVISO exo: superpowers@claude-plugins-official sigue habilitado junto a exo (skills y hooks duplicados). Desactívalo: claude plugin disable superpowers@claude-plugins-official"
    );
}

/// paridad-recall: user scope false: no avisa
#[test]
fn superpowers_user_false() {
    let s = sp();
    s.user(r#"{"enabledPlugins":{"superpowers@claude-plugins-official":false}}"#);
    assert_eq!(s.aviso(), None);
}

/// paridad-recall: project scope habilitado: avisa
/// Falla si mira solo la primera clave.
#[test]
fn superpowers_true_en_proyecto() {
    let s = sp();
    s.proyecto(r#"{"enabledPlugins":{"superpowers@a":false,"superpowers@b":true}}"#);
    let a = s.aviso().unwrap();
    assert!(a.contains("claude plugin disable superpowers@b"), "{a}");
    assert!(!a.contains("superpowers@a"), "{a}");
}

/// paridad-recall: local false pisa a project true: no avisa
/// paridad-recall: project false pisa a user true: no avisa
#[test]
fn superpowers_local_pisa() {
    let s = sp();
    s.user(r#"{"enabledPlugins":{"superpowers@x":true}}"#);
    s.proyecto(r#"{"enabledPlugins":{"superpowers@x":true}}"#);
    assert!(s.aviso().is_some());
    s.local(r#"{"enabledPlugins":{"superpowers@x":false}}"#);
    assert_eq!(s.aviso(), None, "local false pisa a project y user true");

    let s = sp();
    s.user(r#"{"enabledPlugins":{"superpowers@x":true}}"#);
    s.proyecto(r#"{"enabledPlugins":{"superpowers@x":false}}"#);
    assert_eq!(s.aviso(), None, "project false pisa a user true");
}

/// paridad-recall: false seguido de true en el mismo fichero: avisa con la clave true
#[test]
fn superpowers_false_y_luego_true() {
    let s = sp();
    s.user(r#"{"enabledPlugins":{"superpowers@a":false,"superpowers@b":true}}"#);
    let a = s.aviso().unwrap();
    assert!(a.contains("claude plugin disable superpowers@b"), "{a}");
}

// ---------- gate pendiente ----------

fn git(kb: &Path, args: &[&str]) {
    let st = Command::new("git")
        .arg("-C")
        .arg(kb)
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(args)
        .status()
        .unwrap();
    assert!(st.success(), "git {args:?}");
}

fn ahora() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

struct Kb {
    _d: TempDir,
    kb: PathBuf,
    t0: i64,
}

fn kb() -> Kb {
    let d = tempfile::tempdir().unwrap();
    let kb = d.path().join("kb");
    std::fs::create_dir_all(&kb).unwrap();
    git(&kb, &["init", "-q", "-b", "main"]);
    Kb {
        _d: d,
        kb,
        t0: ahora(),
    }
}

impl Kb {
    /// Commit vacío con fecha `atras` segundos en el pasado.
    fn commit(&self, atras: i64, msg: &str) {
        self.commit_con(atras, msg, &["commit", "-q", "--allow-empty", "-m", msg]);
    }
    fn commit_con(&self, atras: i64, _msg: &str, args: &[&str]) {
        let f = format!("{} +0000", self.t0 - atras);
        let st = Command::new("git")
            .arg("-C")
            .arg(&self.kb)
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args)
            .env("GIT_AUTHOR_DATE", &f)
            .env("GIT_COMMITTER_DATE", &f)
            .status()
            .unwrap();
        assert!(st.success());
    }
    fn gate(&self, atras: i64) {
        self.commit(atras, &format!("docs(kb): nota grande\n\n{TRAILER}"));
    }
    /// Auto-distill que toca la nota (commit con fichero), en la rama actual. La
    /// nota pesa el `limit` entero: que no la resuelva el umbral del 85 %.
    fn autodistill_toca(&self, atras: i64) {
        let p = self.kb.join(NOTA);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "x".repeat(12500)).unwrap();
        git(&self.kb, &["add", "--", NOTA]);
        self.commit_con(
            atras,
            "",
            &["commit", "-q", "-m", "docs(kb): auto-distill a — evicción"],
        );
    }
    fn avisos(&self) -> Vec<String> {
        gate_pendiente(&self.kb, Duration::from_secs(10), OsStr::new("git")).unwrap()
    }
}

/// paridad-recall: gate_pendiente_avisa: ruta con espacios, em dash y paréntesis
/// Cubre también `Review Focus`: ruta con espacios, ` — ` y paréntesis con quotePath=false.
#[test]
fn gate_pendiente_aviso() {
    let k = kb();
    k.commit(5000, "docs(kb): documenta algo");
    k.gate(4000);
    assert_eq!(k.avisos(), vec![AVISO.to_string()]);
}

/// Un Gate `notas/x.md` sin auto-distill: el aviso nombra la ruta.
#[test]
fn gate_pendiente_ruta_simple() {
    let k = kb();
    k.commit(
        4000,
        "docs(kb): n\n\nGate: notas/x.md bytes=9000 limit=8000 routed=bitacora",
    );
    let a = k.avisos();
    assert_eq!(a.len(), 1);
    assert!(a[0].contains("notas/x.md"), "{a:?}");
}

/// paridad-recall: gate_sin_duplicados: una línea por ruta aunque haya varios trailers
#[test]
fn gate_sin_duplicados() {
    let k = kb();
    k.gate(4000);
    k.commit(
        3000,
        &format!(
            "docs(kb): otra\n\n{TRAILER}\nGate: projects/z.md bytes=9000 limit=8000 routed=bitacora"
        ),
    );
    let a = k.avisos();
    assert_eq!(
        a.iter().filter(|l| l.contains("projects/a")).count(),
        1,
        "{a:?}"
    );
    assert!(
        a.iter()
            .any(|l| l.contains("gate pendiente en projects/z.md,")),
        "{a:?}"
    );
    assert_eq!(a.len(), 2);
}

/// paridad-recall: gate_resuelto_en_otra_rama_no_avisa: auto-distill en otra rama (--all) resuelve solo su ruta
/// paridad-recall: gate_autodistill_sin_aterrizar_vuelve_a_avisar
#[test]
fn gate_otra_rama_y_sin_aterrizar() {
    let k = kb();
    k.gate(4000);
    k.commit(
        3000,
        &format!(
            "docs(kb): otra\n\n{TRAILER}\nGate: projects/z.md bytes=9000 limit=8000 routed=bitacora"
        ),
    );
    git(&k.kb, &["checkout", "-q", "-b", "distill/auto/x"]);
    k.autodistill_toca(2000);
    git(&k.kb, &["checkout", "-q", "main"]);
    let a = k.avisos();
    assert!(
        !a.iter().any(|l| l.contains("gate pendiente en projects/a")),
        "{a:?}"
    );
    assert!(
        a.iter()
            .any(|l| l.contains("gate pendiente en projects/z.md,")),
        "{a:?}"
    );

    // La rama se borra sin aterrizar: el aviso vuelve.
    git(&k.kb, &["branch", "-q", "-D", "distill/auto/x"]);
    git(&k.kb, &["reflog", "expire", "--expire=now", "--all"]);
    git(&k.kb, &["gc", "-q", "--prune=now"]);
    assert!(k.avisos().contains(&AVISO.to_string()));
}

/// paridad-recall: gate_resuelto_por_commit_en_main_no_avisa
#[test]
fn gate_resuelto_por_commit_en_main() {
    let k = kb();
    k.gate(4000);
    k.autodistill_toca(2000);
    assert_eq!(k.avisos(), Vec::<String>::new());
}

/// Mismo segundo: el gate no es posterior al auto-distill (`ct > d` estricto, como el awk).
#[test]
fn gate_mismo_segundo_que_autodistill_resuelve() {
    let k = kb();
    k.gate(4000);
    k.autodistill_toca(4000);
    assert_eq!(k.avisos(), Vec::<String>::new());
}

/// paridad-recall: gate_con_atribucion_en_otro_parrafo_avisa
#[test]
fn gate_con_atribucion_en_otro_parrafo() {
    let k = kb();
    k.commit(
        4000,
        &format!("docs(kb): nota grande\n\n{TRAILER}\n\nCo-Authored-By: X <x@x>\nClaude-Session: https://claude.ai/code/s"),
    );
    assert_eq!(k.avisos(), vec![AVISO.to_string()]);
}

/// paridad-recall: gate_resuelto_por_nota_sin_name_only
#[test]
fn resuelto_por_nota() {
    let k = kb();
    k.gate(4000);
    git(&k.kb, &["checkout", "-q", "-b", "distill/auto/n"]);
    k.commit(
        2000,
        &format!("docs(kb): auto-distill a — propuesta de partición\n\nNota: {NOTA}"),
    );
    git(&k.kb, &["checkout", "-q", "main"]);
    assert_eq!(k.avisos(), Vec::<String>::new());
}

/// paridad-recall: gate_anterior_a_distill_last_no_avisa
#[test]
fn gate_anterior_a_distill_last() {
    let k = kb();
    k.gate(4000);
    k.commit(3000, "docs(kb): distill");
    git(&k.kb, &["tag", "distill/last"]);
    assert_eq!(k.avisos(), Vec::<String>::new());
}

/// paridad-recall: gate_sin_tag_ventana_7_dias
#[test]
fn gate_sin_tag_ventana_7_dias() {
    let k = kb();
    k.gate(10 * 86400);
    assert_eq!(k.avisos(), Vec::<String>::new());
}

/// paridad-recall: gate_posterior_a_autodistill_sigue_avisando
#[test]
fn gate_posterior_a_autodistill() {
    let k = kb();
    k.commit(6000, "docs(kb): base");
    git(&k.kb, &["checkout", "-q", "-b", "distill/auto/y"]);
    k.autodistill_toca(5000);
    git(&k.kb, &["checkout", "-q", "main"]);
    // El checkout deja la nota fuera del árbol de main: sin fichero, no hay tamaño.
    k.gate(4000);
    assert_eq!(k.avisos(), vec![AVISO.to_string()]);
}

/// paridad-recall: gate_resuelto_a_mano_no_avisa: nota bajo el 85 % del limit
/// paridad-recall: gate_nota_al_85_sigue_avisando
/// Falla si mira el 85 % al revés o con `<=`.
#[test]
fn gate_resuelto_por_tamano() {
    let k = kb();
    k.gate(4000);
    let p = k.kb.join(NOTA);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(&p, "x".repeat(10624)).unwrap(); // 85 % de 12500 = 10625
    assert_eq!(k.avisos(), Vec::<String>::new());
    std::fs::write(&p, "x".repeat(10625)).unwrap();
    assert_eq!(k.avisos(), vec![AVISO.to_string()]);
}

/// paridad-recall: kb_sin_git_no_rompe
#[test]
fn kb_sin_git() {
    let d = tempfile::tempdir().unwrap();
    assert_eq!(
        gate_pendiente(d.path(), Duration::from_secs(3), OsStr::new("git")),
        Ok(vec![])
    );
}

// ---------- deadline ----------

fn stub(d: &Path, cuerpo: &str) -> PathBuf {
    let p = d.join("git");
    std::fs::write(&p, format!("#!/bin/sh\n{cuerpo}\n")).unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    p
}

fn kb_con_git_dir() -> (TempDir, PathBuf) {
    let d = tempfile::tempdir().unwrap();
    let kb = d.path().join("kb");
    std::fs::create_dir_all(kb.join(".git")).unwrap();
    (d, kb)
}

/// paridad-recall: gate_git_colgado_corta_sin_aviso
#[test]
fn gate_deadline() {
    let (d, kb) = kb_con_git_dir();
    let g = stub(d.path(), "exec sleep 10");
    let t = Instant::now();
    let r = gate_pendiente(&kb, Duration::from_millis(200), g.as_os_str());
    assert_eq!(r, Err(GateTimeout));
    assert!(t.elapsed() < Duration::from_secs(2), "{:?}", t.elapsed());
}

/// 3 git de 150 ms con deadline 300 ms: por comando pasarían los tres; global vence.
#[test]
fn gate_deadline_es_global() {
    let (d, kb) = kb_con_git_dir();
    // rev-parse: éxito (hay tag); log: salida no vacía para que corran los 3.
    let g = stub(
        d.path(),
        "sleep 0.15\ncase \"$*\" in *rev-parse*) ;; *) echo '@@1';; esac",
    );
    let t = Instant::now();
    let r = gate_pendiente(&kb, Duration::from_millis(300), g.as_os_str());
    assert_eq!(r, Err(GateTimeout));
    assert!(
        t.elapsed() < Duration::from_millis(1500),
        "{:?}",
        t.elapsed()
    );
    // Control: con holgura, los 3 git corren y terminan sin aviso.
    let r = gate_pendiente(&kb, Duration::from_secs(5), g.as_os_str());
    assert_eq!(r, Ok(vec![]));
}

#[test]
fn corre_con_deadline_mata_y_recoge() {
    let t = Instant::now();
    let mut c = Command::new("sh");
    c.args(["-c", "exec sleep 10"]);
    assert!(corre_con_deadline(c, Instant::now() + Duration::from_millis(100)).is_none());
    assert!(t.elapsed() < Duration::from_secs(2));

    let mut c = Command::new("sh");
    c.args(["-c", "echo hola"]);
    let o = corre_con_deadline(c, Instant::now() + Duration::from_secs(5)).unwrap();
    assert!(o.status.success());
    assert_eq!(o.stdout, b"hola\n");
}
