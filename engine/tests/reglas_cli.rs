//! `exo rules` contra el binario real, con KB y repos git en tempfile.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_exo")
}

struct Mundo {
    dir: tempfile::TempDir,
    cfg: PathBuf,
}

impl Mundo {
    fn nuevo() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let cfg = dir.path().join("gitconfig-vacio");
        std::fs::write(&cfg, "").unwrap();
        std::fs::create_dir_all(dir.path().join("kb/projects")).unwrap();
        Mundo { dir, cfg }
    }

    fn kb(&self) -> PathBuf {
        self.dir.path().join("kb")
    }

    fn nota(&self, rel: &str, contenido: &str) {
        let p = self.kb().join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, contenido).unwrap();
    }

    fn git(&self, dir: &Path, args: &[&str]) {
        let s = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .env("GIT_CONFIG_GLOBAL", &self.cfg)
            .env("GIT_CONFIG_SYSTEM", &self.cfg)
            .env("GIT_AUTHOR_NAME", "f")
            .env("GIT_AUTHOR_EMAIL", "f@k.local")
            .env("GIT_COMMITTER_NAME", "f")
            .env("GIT_COMMITTER_EMAIL", "f@k.local")
            .output()
            .unwrap();
        assert!(s.status.success(), "git {args:?}");
    }

    fn repo(&self, nombre: &str) -> PathBuf {
        let r = self.dir.path().join(nombre);
        std::fs::create_dir_all(r.join("a/b")).unwrap();
        self.git(&r, &["init", "-q"]);
        std::fs::write(r.join("f"), "x").unwrap();
        self.git(&r, &["add", "f"]);
        self.git(&r, &["commit", "-q", "-m", "i"]);
        r
    }

    fn rules(&self, cwd: &Path) -> Value {
        let o = Command::new(bin())
            .args(["rules", "--json", "--kb"])
            .arg(self.kb())
            .arg("--cwd")
            .arg(cwd)
            .env("GIT_CONFIG_GLOBAL", &self.cfg)
            .env("GIT_CONFIG_SYSTEM", &self.cfg)
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        let linea = String::from_utf8(o.stdout).unwrap();
        assert_eq!(linea.trim().lines().count(), 1);
        serde_json::from_str(linea.trim()).unwrap()
    }
}

fn data(v: &Value) -> &Value {
    &v["data"]
}

#[test]
fn ok_por_slug() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota(
        "projects/Foo bar.md",
        "---\nslug: FOO\n---\n# x\n## Reglas duras\n- uno\n- dos\n",
    );
    let v = m.rules(&r);
    assert_eq!(data(&v)["status"], "ok");
    assert_eq!(data(&v)["repo"], "foo");
    assert_eq!(data(&v)["note"], "projects/Foo bar.md");
    assert_eq!(data(&v)["rules"], serde_json::json!(["uno", "dos"]));
}

#[test]
fn slug_con_comillas() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota(
        "projects/otro.md",
        "---\nslug: \"foo\"\n---\n## Reglas duras\n- uno\n",
    );
    assert_eq!(data(&m.rules(&r))["status"], "ok");
}

#[test]
fn ok_por_stem_con_raya() {
    let m = Mundo::nuevo();
    let r = m.repo("exo");
    m.nota(
        "projects/exo — framework.md",
        "# x\n## Reglas duras\n- uno\n",
    );
    assert_eq!(data(&m.rules(&r))["status"], "ok");
}

#[test]
fn desde_subdirectorio_y_worktree() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota("projects/foo.md", "## Reglas duras\n- uno\n");
    assert_eq!(data(&m.rules(&r.join("a/b")))["repo"], "foo");
    let wt = m.dir.path().join("lejos-wt");
    m.git(&r, &["worktree", "add", "-q", wt.to_str().unwrap()]);
    let v = m.rules(&wt);
    assert_eq!(data(&v)["repo"], "foo");
    assert_eq!(data(&v)["status"], "ok");
}

#[test]
fn sin_git() {
    let m = Mundo::nuevo();
    let d = m.dir.path().join("vacio");
    std::fs::create_dir_all(&d).unwrap();
    let v = m.rules(&d);
    assert_eq!(data(&v)["status"], "skip");
    assert_eq!(data(&v)["reason"], "sin_git");
    assert!(data(&v)["repo"].is_null());
}

#[test]
fn sin_nota() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    let v = m.rules(&r);
    assert_eq!(data(&v)["reason"], "sin_nota");
    assert_eq!(data(&v)["repo"], "foo");
}

#[test]
fn cwd_dentro_de_la_kb_es_un_repo_mas() {
    let m = Mundo::nuevo();
    m.git(&m.kb(), &["init", "-q"]);
    let v = m.rules(&m.kb());
    assert_eq!(data(&v)["reason"], "sin_nota");
    assert_eq!(data(&v)["repo"], "kb");
}

#[test]
fn subcarpeta_no_compite() {
    let m = Mundo::nuevo();
    let r = m.repo("pm");
    m.nota(
        "projects/pm.md",
        "---\nslug: pm\n---\n## Reglas duras\n- a\n",
    );
    m.nota("projects/pm/pm — guía.md", "## Reglas duras\n- b\n");
    let v = m.rules(&r);
    assert_eq!(data(&v)["status"], "ok");
    assert_eq!(data(&v)["note"], "projects/pm.md");
}

#[test]
fn ambigua() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota(
        "projects/x.md",
        "---\nslug: foo\n---\n## Reglas duras\n- a\n",
    );
    m.nota("projects/foo.md", "## Reglas duras\n- b\n");
    let v = m.rules(&r);
    assert_eq!(data(&v)["reason"], "ambigua");
    assert_eq!(
        data(&v)["candidates"],
        serde_json::json!(["projects/foo.md", "projects/x.md"])
    );
}

#[test]
fn sin_seccion_y_seccion_vacia() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota("projects/foo.md", "# x\n- suelta\n");
    assert_eq!(data(&m.rules(&r))["reason"], "sin_seccion");
    m.nota("projects/foo.md", "## Reglas duras\nsolo prosa\n");
    let v = m.rules(&r);
    assert_eq!(data(&v)["reason"], "seccion_vacia");
    assert!(!v.to_string().contains("solo prosa"));
}

#[test]
fn seccion_hasta_siguiente_h2_y_crlf() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota(
        "projects/foo.md",
        "## Reglas duras\r\n- a\r\nprosa\r\n-x\r\n- b\r\n## Otra\r\n- x\r\n",
    );
    let v = m.rules(&r);
    assert_eq!(data(&v)["rules"], serde_json::json!(["a", "b"]));
    assert_eq!(
        data(&v)["ignored_lines"],
        serde_json::json!(["prosa", "-x"])
    );
}

#[test]
fn encabezado_exacto() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota("projects/foo.md", "## Reglas duras (borrador)\n- a\n");
    assert_eq!(data(&m.rules(&r))["reason"], "sin_seccion");
}

#[test]
fn excede_cap() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    let con = |n: usize| {
        let mut s = String::from("## Reglas duras\n");
        for i in 0..n {
            s.push_str(&format!("- r{i}\n"));
        }
        s
    };
    m.nota("projects/foo.md", &con(11));
    let v = m.rules(&r);
    assert_eq!(data(&v)["reason"], "excede_cap");
    assert!(data(&v).get("rules").is_none());
    m.nota("projects/foo.md", &con(10));
    assert_eq!(data(&m.rules(&r))["status"], "ok");
}

#[test]
fn envelope_rules() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    let v = m.rules(&r);
    assert_eq!(v["command"], "rules");
    assert_eq!(v["schema_version"], 2);
}

#[test]
fn texto_sin_json() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota("projects/foo.md", "## Reglas duras\n- a\n");
    let o = Command::new(bin())
        .args(["rules", "--kb"])
        .arg(m.kb())
        .arg("--cwd")
        .arg(&r)
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&o.stdout), "ok foo: 1 reglas\n");
}

#[test]
fn git_que_no_arranca_es_error_del_engine_no_sin_git() {
    let m = Mundo::nuevo();
    let vacio = m.dir.path().join("path-vacio");
    std::fs::create_dir_all(&vacio).unwrap();
    let o = Command::new(bin())
        .args(["rules", "--json", "--kb"])
        .arg(m.kb())
        .arg("--cwd")
        .arg(m.dir.path())
        .env("PATH", &vacio)
        .output()
        .unwrap();
    assert!(!o.status.success(), "git ausente debe ser exit != 0");
    assert!(!String::from_utf8_lossy(&o.stdout).contains("sin_git"));
}
