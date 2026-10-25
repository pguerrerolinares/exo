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
        self.rules_con(cwd, &self.kb(), None)
    }

    fn rules_con(&self, cwd: &Path, kb: &Path, techo: Option<&Path>) -> Value {
        let o = self.ejecuta(cwd, kb, techo);
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        let linea = String::from_utf8(o.stdout).unwrap();
        assert_eq!(linea.trim().lines().count(), 1);
        serde_json::from_str(linea.trim()).unwrap()
    }

    fn ejecuta(&self, cwd: &Path, kb: &Path, techo: Option<&Path>) -> std::process::Output {
        let mut c = Command::new(bin());
        c.args(["rules", "--json", "--kb"])
            .arg(kb)
            .arg("--cwd")
            .arg(cwd)
            .env("GIT_CONFIG_GLOBAL", &self.cfg)
            .env("GIT_CONFIG_SYSTEM", &self.cfg);
        if let Some(t) = techo {
            c.env("GIT_CEILING_DIRECTORIES", t);
        }
        c.output().unwrap()
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
    let v = m.rules_con(&d, &m.kb(), Some(m.dir.path()));
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
fn slug_desempata_frente_a_stem() {
    // Nota canon con slug + nota hermana partida que solo casa por stem.
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota(
        "projects/foo — canon.md",
        "---\nslug: foo\n---\n## Reglas duras\n- a\n",
    );
    m.nota("projects/foo — reglas android.md", "## Reglas duras\n- b\n");
    let v = m.rules(&r);
    assert_eq!(data(&v)["status"], "ok");
    assert_eq!(data(&v)["note"], "projects/foo — canon.md");
    assert_eq!(data(&v)["rules"], serde_json::json!(["a"]));
}

#[test]
fn ambigua_por_stem() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota("projects/foo — x.md", "## Reglas duras\n- a\n");
    m.nota("projects/foo.md", "## Reglas duras\n- b\n");
    let v = m.rules(&r);
    assert_eq!(data(&v)["reason"], "ambigua");
    assert_eq!(
        data(&v)["candidates"],
        serde_json::json!(["projects/foo — x.md", "projects/foo.md"])
    );
}

#[test]
fn ambigua_por_slug() {
    // Dos slugs iguales empatan; el stem de una tercera no entra en la lista.
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota(
        "projects/x.md",
        "---\nslug: foo\n---\n## Reglas duras\n- a\n",
    );
    m.nota(
        "projects/y.md",
        "---\nslug: foo\n---\n## Reglas duras\n- b\n",
    );
    m.nota("projects/foo.md", "## Reglas duras\n- c\n");
    let v = m.rules(&r);
    assert_eq!(data(&v)["reason"], "ambigua");
    assert_eq!(
        data(&v)["candidates"],
        serde_json::json!(["projects/x.md", "projects/y.md"])
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

fn con_reglas(prefijo: &str, n: usize) -> String {
    let mut s = String::from("## Reglas duras\n");
    for i in 1..=n {
        s.push_str(&format!("- {prefijo}{i}\n"));
    }
    s
}

fn lista(prefijo: &str, n: usize) -> Value {
    (1..=n).map(|i| format!("{prefijo}{i}")).collect()
}

#[test]
fn excede_cap_entrega_las_diez_primeras() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota("projects/foo.md", &con_reglas("r", 12));
    let v = m.rules(&r);
    assert_eq!(data(&v)["status"], "ok");
    assert_eq!(data(&v)["rules"], lista("r", 10));
    assert_eq!(data(&v)["truncated"], 2);
    m.nota("projects/foo.md", &con_reglas("r", 10));
    assert_eq!(data(&m.rules(&r))["truncated"], 0);
}

#[test]
fn nota_en_subdirectorio_resuelve_por_slug() {
    let m = Mundo::nuevo();
    let r = m.repo("code-graph-engine");
    m.nota(
        "projects/cge/cge — motor.md",
        "---\nslug: code-graph-engine\n---\n## Reglas duras\n- a\n",
    );
    let v = m.rules(&r);
    assert_eq!(data(&v)["status"], "ok");
    assert_eq!(data(&v)["note"], "projects/cge/cge — motor.md");
}

#[test]
fn subdirectorio_por_stem() {
    let m = Mundo::nuevo();
    let r = m.repo("pm");
    m.nota("projects/pm/pm — guía.md", "## Reglas duras\n- a\n");
    let v = m.rules(&r);
    assert_eq!(data(&v)["status"], "ok");
    assert_eq!(data(&v)["note"], "projects/pm/pm — guía.md");
}

#[test]
fn note_relativa_con_kb_relativa() {
    let m = Mundo::nuevo();
    let r = m.repo("pm");
    m.nota("projects/pm/pm — guía.md", "## Reglas duras\n- a\n");
    let o = Command::new(bin())
        .args(["rules", "--json", "--kb", "kb", "--cwd"])
        .arg(&r)
        .current_dir(m.dir.path())
        .env("GIT_CONFIG_GLOBAL", &m.cfg)
        .env("GIT_CONFIG_SYSTEM", &m.cfg)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(data(&v)["note"], "projects/pm/pm — guía.md");
}

#[test]
fn ambigua_entre_raiz_y_subdirectorio() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota("projects/foo.md", "## Reglas duras\n- a\n");
    m.nota("projects/x/foo — b.md", "## Reglas duras\n- b\n");
    let v = m.rules(&r);
    assert_eq!(data(&v)["status"], "skip");
    assert_eq!(data(&v)["reason"], "ambigua");
    assert_eq!(
        data(&v)["candidates"],
        serde_json::json!(["projects/foo.md", "projects/x/foo — b.md"])
    );
}

#[test]
fn dos_niveles_no_cuentan() {
    let m = Mundo::nuevo();
    m.nota("projects/pm/pm — guía.md", "## Reglas duras\n- a\n");
    m.nota("projects/a/b/foo.md", "## Reglas duras\n- b\n");
    let pm = m.repo("pm");
    assert_eq!(data(&m.rules(&pm))["status"], "ok");
    let foo = m.repo("foo");
    let v = m.rules(&foo);
    assert_eq!(data(&v)["status"], "skip");
    assert_eq!(data(&v)["reason"], "sin_nota");
}

#[test]
fn global_llega_con_ok() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota("projects/foo.md", "## Reglas duras\n- a\n");
    m.nota("core/reglas-duras.md", "## Reglas duras\n- g1\n- g2\n");
    let v = m.rules(&r);
    assert_eq!(data(&v)["status"], "ok");
    assert_eq!(data(&v)["global"], serde_json::json!(["g1", "g2"]));
}

#[test]
fn global_llega_con_skip() {
    let m = Mundo::nuevo();
    m.nota("core/reglas-duras.md", "## Reglas duras\n- g1\n");
    let r = m.repo("foo");
    let v = m.rules(&r);
    assert_eq!(data(&v)["status"], "skip");
    assert_eq!(data(&v)["reason"], "sin_nota");
    assert_eq!(data(&v)["global"], serde_json::json!(["g1"]));
    let d = m.dir.path().join("vacio");
    std::fs::create_dir_all(&d).unwrap();
    let v = m.rules_con(&d, &m.kb(), Some(m.dir.path()));
    assert_eq!(data(&v)["reason"], "sin_git");
    assert_eq!(data(&v)["global"], serde_json::json!(["g1"]));
}

#[test]
fn global_ausente_es_lista_vacia() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    let v = m.rules(&r);
    assert_eq!(data(&v)["reason"], "sin_nota");
    assert_eq!(data(&v)["global"], serde_json::json!([]));
    assert_eq!(data(&v)["truncated"], 0);
    m.nota("projects/foo.md", "## Reglas duras\n- a\n");
    let v = m.rules(&r);
    assert_eq!(data(&v)["status"], "ok");
    assert_eq!(data(&v)["global"], serde_json::json!([]));
    assert_eq!(data(&v)["truncated"], 0);
}

#[test]
fn global_ilegible_es_error_del_engine() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    std::fs::create_dir_all(m.kb().join("core/reglas-duras.md")).unwrap();
    let o = m.ejecuta(&r, &m.kb(), None);
    assert!(!o.status.success(), "global ilegible debe ser exit != 0");
}

#[test]
fn global_recorta_al_cap() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    m.nota("projects/foo.md", &con_reglas("r", 12));
    m.nota("core/reglas-duras.md", &con_reglas("g", 11));
    let v = m.rules(&r);
    assert_eq!(data(&v)["rules"], lista("r", 10));
    assert_eq!(data(&v)["global"], lista("g", 10));
    assert_eq!(data(&v)["truncated"], 3);
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

#[test]
fn kb_sin_projects_es_error_del_engine() {
    let m = Mundo::nuevo();
    let r = m.repo("foo");
    for kb in [
        m.dir.path().join("no-existe"),
        m.dir.path().join("vacio-kb"),
    ] {
        std::fs::create_dir_all(m.dir.path().join("vacio-kb")).unwrap();
        let o = m.ejecuta(&r, &kb, None);
        assert!(!o.status.success(), "KB sin projects/ debe ser exit != 0");
        assert!(!String::from_utf8_lossy(&o.stdout).contains("sin_nota"));
    }
}

#[test]
fn git_roto_dentro_de_un_repo_es_error_no_sin_git() {
    let m = Mundo::nuevo();
    let roto = m.dir.path().join("roto");
    std::fs::create_dir_all(roto.join("sub")).unwrap();
    std::fs::write(roto.join(".git"), "gitdir: /no/existe\n").unwrap();
    let o = m.ejecuta(&roto.join("sub"), &m.kb(), Some(m.dir.path()));
    assert!(
        !o.status.success(),
        "git roto bajo un .git debe ser exit != 0"
    );
    assert!(!String::from_utf8_lossy(&o.stdout).contains("sin_git"));
}
