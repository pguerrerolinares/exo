mod common;

use exo::hook::guards::recall::FALLBACK;
use exo::hook::guards::reglas::AUTORIDAD;
use serde_json::{Value, json};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NOMBRE_KB: &str = "kb-test";
const AVISO_GATE: &str =
    "AVISO exo: gate pendiente en core/x.md, sin auto-distill que lo resuelva (corre /distill)";
const REAFIRMA_GIT: &str = "\n\n--- Reglas reforzadas tras compactacion (dispararon esta sesion) ---\n- nunca `cd X && git ...`: usa `git -C X`.";

struct Entorno {
    dir: tempfile::TempDir,
}

impl Entorno {
    fn nuevo() -> Self {
        Entorno {
            dir: tempfile::tempdir().unwrap(),
        }
    }
    fn log(&self) -> std::path::PathBuf {
        self.dir.path().join("reflex.jsonl")
    }
    fn state(&self) -> std::path::PathBuf {
        self.dir.path().join("state")
    }
    fn home(&self) -> std::path::PathBuf {
        self.dir.path().join("home")
    }
    /// `HOME`, config y seams van aislados: el hook nunca ve el `~` ni el entorno reales.
    fn comando(&self, args: &[&str], extra: &[(String, String)]) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_exo"));
        cmd.arg("hook").args(args);
        for k in [
            "EXO_KB",
            "EXO_INDEX",
            "EXO_KB_NAME",
            "EXO_DB",
            "EXO_RECALL_NOTA",
            "EXO_RECALL_CAP",
            "RECALL_BLOQUE_CAP",
            "EXO_RECALL_LIMITE",
            "EXO_RECALL_COMPACT_LINEAS",
            "EXO_GIT_TIMEOUT",
            "EXO_BIN",
            "EXO_INDEX_LOG",
            "REFLEX_CANARY_FILE",
            "CLAUDE_PROJECT_DIR",
            "CODEX_HOME",
            "CDPATH",
            "EXO_HOOK_PANIC",
        ] {
            cmd.env_remove(k);
        }
        cmd.env("HOME", self.home())
            .env("USERPROFILE", self.home())
            .env("EXO_CONFIG", self.dir.path().join("sin-config.toml"))
            .env("EXO_STATE_DIR", self.state())
            .env("REFLEX_LOG_FILE", self.log())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (k, v) in extra {
            cmd.env(k, v);
        }
        cmd
    }
    fn corre_env(&self, args: &[&str], stdin: &[u8], extra: &[(String, String)]) -> Output {
        let mut hijo = self.comando(args, extra).spawn().unwrap();
        hijo.stdin.take().unwrap().write_all(stdin).unwrap();
        hijo.wait_with_output().unwrap()
    }
    fn corre(&self, args: &[&str], stdin: &[u8], extra: &[(&str, &str)]) -> Output {
        let extra: Vec<(String, String)> = extra
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.corre_env(args, stdin, &extra)
    }
    fn reflex(&self) -> Vec<Value> {
        std::fs::read_to_string(self.log())
            .unwrap_or_default()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }
    fn journal(&self) -> Vec<Value> {
        let dir = self.state().join("sessions");
        let Ok(rd) = std::fs::read_dir(dir) else {
            return vec![];
        };
        rd.flat_map(|e| {
            std::fs::read_to_string(e.unwrap().path())
                .unwrap()
                .lines()
                .map(|l| serde_json::from_str::<Value>(l).unwrap())
                .collect::<Vec<_>>()
        })
        .collect()
    }
}

fn fixture(nombre: &str) -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/hook/claude")
            .join(nombre),
    )
    .unwrap()
}

fn bash(evento: &str, cmd: &str) -> Vec<u8> {
    json!({
        "session_id": "sess-1", "cwd": "/home/u/proj", "hook_event_name": evento,
        "tool_name": "Bash", "tool_input": {"command": cmd}
    })
    .to_string()
    .into_bytes()
}

#[test]
fn cli_harness_desconocido_sale_0_sin_stdout() {
    let e = Entorno::nuevo();
    // Una entrada que claude denegaría: el vacío viene del harness, no de que no haya nada que denegar.
    let o = e.corre(
        &["foo", "PreToolUse"],
        &bash("PreToolUse", "git add -A"),
        &[],
    );
    assert_eq!(o.status.code(), Some(0));
    assert!(o.stdout.is_empty());
    assert!(o.stderr.is_empty());
    assert!(e.reflex().is_empty());
}

#[test]
fn cli_stdin_ilegible_sale_0() {
    let e = Entorno::nuevo();
    let o = e.corre(&["claude", "PreToolUse"], b"x", &[]);
    assert_eq!(o.status.code(), Some(0));
    assert!(o.stdout.is_empty());
    assert!(o.stderr.is_empty());
    let r = e.reflex();
    assert_eq!(r.len(), 1);
    assert_eq!(r[0]["reflex"], "hook-error");
    assert_eq!(r[0]["payload"], "code=parse");
}

#[test]
fn cli_post_tool_use_registra_ok_true() {
    let e = Entorno::nuevo();
    let o = e.corre(
        &["claude", "PostToolUse"],
        &fixture("post_tool_use_bash.json"),
        &[],
    );
    assert_eq!(o.status.code(), Some(0));
    assert!(o.stdout.is_empty());
    let j = e.journal();
    assert_eq!(j.len(), 1, "{j:?}");
    assert_eq!(j[0]["kind"], "tool");
    assert_eq!(j[0]["ok"], true);
}

#[test]
fn cli_post_tool_use_failure_registra_ok_false() {
    let e = Entorno::nuevo();
    let o = e.corre(
        &["claude", "PostToolUseFailure"],
        &fixture("post_tool_use_failure_bash.json"),
        &[],
    );
    assert_eq!(o.status.code(), Some(0));
    let j = e.journal();
    assert_eq!(j.len(), 1, "{j:?}");
    assert_eq!(j[0]["ok"], false);
}

#[test]
fn cli_post_tool_use_no_dispara_guards() {
    let e = Entorno::nuevo();
    let o = e.corre(
        &["claude", "PostToolUse"],
        &bash("PostToolUse", "git add -A"),
        &[],
    );
    assert_eq!(o.status.code(), Some(0));
    assert!(
        o.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&o.stdout)
    );
    assert!(e.reflex().is_empty());
    // La misma entrada en PreToolUse sí es deny: el vacío anterior no es un guard roto.
    let o = e.corre(
        &["claude", "PreToolUse"],
        &bash("PreToolUse", "git add -A"),
        &[],
    );
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["hookSpecificOutput"]["permissionDecision"], "deny");
}

#[test]
fn cli_pre_tool_use_rewrite_end_to_end() {
    let e = Entorno::nuevo();
    let o = e.corre(
        &["claude", "PreToolUse"],
        &bash("PreToolUse", "cd /home/u/repo && git status"),
        &[],
    );
    assert_eq!(o.status.code(), Some(0));
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    let h = &v["hookSpecificOutput"];
    assert_eq!(h["permissionDecision"], "allow");
    assert_eq!(h["updatedInput"]["command"], "git -C /home/u/repo status");
    assert!(e.reflex().iter().any(|r| r["reflex"] == "git-c-rewrite"));
}

#[cfg(debug_assertions)]
#[test]
fn cli_panic_en_guard_no_rompe() {
    let e = Entorno::nuevo();
    let o = e.corre(
        &["claude", "PreToolUse"],
        &bash("PreToolUse", "git add -A"),
        &[("EXO_HOOK_PANIC", "git-c")],
    );
    assert_eq!(o.status.code(), Some(0));
    assert!(
        o.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    // zero-residuo sigue corriendo pese al panic de git-c.
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["hookSpecificOutput"]["permissionDecision"], "deny");
    let r = e.reflex();
    assert!(
        r.iter()
            .any(|x| x["reflex"] == "hook-panic" && x["payload"] == "git-c"),
        "{r:?}"
    );
}

#[test]
fn cli_cdpath_definido_no_reescribe_relativo() {
    let cmd = bash("PreToolUse", "cd repo && git status");
    let e = Entorno::nuevo();
    let o = e.corre(&["claude", "PreToolUse"], &cmd, &[("CDPATH", "/x")]);
    assert_eq!(o.status.code(), Some(0));
    assert!(
        o.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&o.stdout)
    );
    // Sin CDPATH la misma entrada sí se reescribe.
    let o = e.corre(&["claude", "PreToolUse"], &cmd, &[]);
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(
        v["hookSpecificOutput"]["updatedInput"]["command"],
        "git -C repo status"
    );
}

#[test]
fn cli_sin_argumentos_sale_0_sin_stdout() {
    let e = Entorno::nuevo();
    for args in [&[][..], &["claude"][..]] {
        let o = e.corre(args, b"{}", &[]);
        assert_eq!(o.status.code(), Some(0), "{args:?}");
        assert!(o.stdout.is_empty());
        assert!(o.stderr.is_empty());
    }
}

#[cfg(debug_assertions)]
#[test]
fn cli_panic_fuera_de_guards_sale_0() {
    let e = Entorno::nuevo();
    let o = e.corre(
        &["claude", "PreToolUse"],
        &bash("PreToolUse", "git add -A"),
        &[("EXO_HOOK_PANIC", "runner")],
    );
    assert_eq!(o.status.code(), Some(0));
    assert!(o.stdout.is_empty());
    assert!(
        o.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    let r = e.reflex();
    assert!(
        r.iter()
            .any(|x| x["reflex"] == "hook-panic" && x["payload"] == "runner"),
        "{r:?}"
    );
}

#[test]
fn cli_premisa_4_2_false_real_registra_ok_false() {
    let e = Entorno::nuevo();
    let probe = fixture("claude-2.1.317-probe.jsonl");
    let linea = String::from_utf8(probe)
        .unwrap()
        .lines()
        .find(|l| l.contains("PostToolUseFailure") && l.contains(r#""command":"false""#))
        .unwrap()
        .to_string();
    let o = e.corre(&["claude", "PostToolUseFailure"], linea.as_bytes(), &[]);
    assert_eq!(o.status.code(), Some(0));
    let j = e.journal();
    assert_eq!(j.len(), 1, "{j:?}");
    assert_eq!(j[0]["cmd"], "false");
    assert_eq!(j[0]["ok"], false);
}

// ---------- SessionStart / UPS / Stop de extremo a extremo ----------

fn git(dir: &Path, args: &[&str]) {
    let st = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.email=t@exo.local", "-c", "user.name=exo-test"])
        .args(args)
        .status()
        .unwrap();
    assert!(st.success(), "git {args:?}");
}

fn copia(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src).unwrap() {
        let e = e.unwrap();
        let destino = dst.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copia(&e.path(), &destino);
        } else {
            std::fs::copy(e.path(), destino).unwrap();
        }
    }
}

struct Plantilla {
    raiz: PathBuf,
}

/// KB + índice construidos una vez por ejecución con el propio binario; cada test copia los suyos.
fn plantilla() -> &'static Plantilla {
    static P: OnceLock<Plantilla> = OnceLock::new();
    P.get_or_init(|| {
        let raiz = Path::new(env!("CARGO_TARGET_TMPDIR")).join("hook_cli_plantilla");
        let _ = std::fs::remove_dir_all(&raiz);
        let (kb, db) = (raiz.join("kb"), raiz.join("exo.db"));
        std::fs::create_dir_all(kb.join("core")).unwrap();
        std::fs::create_dir_all(kb.join("projects")).unwrap();
        git(&kb, &["init", "-q"]);
        let nota = |f: &str, perm: &str, tit: &str, cuerpo: &str| {
            std::fs::write(
                kb.join("core").join(f),
                format!("---\npermalink: {NOMBRE_KB}/core/{perm}\ntitle: {tit}\ntier: core\n---\n{cuerpo}\n"),
            )
            .unwrap();
        };
        let relleno: String = (1..=30).map(|i| format!("- relleno-{i}\n")).collect();
        nota(
            "core-index.md",
            "core-index",
            "Core Index",
            &format!("intro\n\n## Contrato de memoria\nCONTRATO-LINEA-1\nCONTRATO-LINEA-2\n\n## Cores\n{relleno}"),
        );
        // El '€' (3 B) empieza en el byte 1499 del contrato: el cap de 1500 B lo parte.
        let largo = format!(
            "## Contrato de memoria\n{}€{}\n\n## Cores\n- x",
            "a".repeat(1476),
            "b".repeat(100)
        );
        nota("largo.md", "largo", "Largo", &largo);
        nota("sin-contrato.md", "sin-contrato", "Sin contrato", "nada que ver");
        git(&kb, &["add", "core"]);
        git(&kb, &["commit", "-q", "-m", "notas"]);
        let config = raiz.join("config.toml");
        std::fs::write(&config, common::render_config(&kb, NOMBRE_KB, &db)).unwrap();
        let o = Command::new(env!("CARGO_BIN_EXE_exo"))
            .args(["index", "--db"])
            .arg(&db)
            .arg("--kb")
            .arg(&kb)
            .env("EXO_CONFIG", &config)
            .output()
            .unwrap();
        assert!(o.status.success(), "exo index de la plantilla: {o:?}");
        Plantilla { raiz }
    })
}

struct Mundo {
    e: Entorno,
    kb: PathBuf,
    db: PathBuf,
}

impl std::ops::Deref for Mundo {
    type Target = Entorno;
    fn deref(&self) -> &Entorno {
        &self.e
    }
}

impl Mundo {
    fn nuevo() -> Self {
        let e = Entorno::nuevo();
        let p = plantilla();
        let kb = e.dir.path().join("kb");
        copia(&p.raiz.join("kb"), &kb);
        let db = e.dir.path().join("exo.db");
        std::fs::copy(p.raiz.join("exo.db"), &db).unwrap();
        Mundo { e, kb, db }
    }
    /// Seams de recall apuntando a la KB copiada; `extra` los pisa (un valor vacío los quita).
    fn corre(&self, args: &[&str], stdin: &[u8], extra: &[(&str, &str)]) -> Output {
        let mut env: Vec<(String, String)> = vec![
            ("EXO_KB".into(), self.kb.display().to_string()),
            ("EXO_INDEX".into(), self.db.display().to_string()),
            ("EXO_KB_NAME".into(), NOMBRE_KB.into()),
        ];
        env.extend(extra.iter().map(|(k, v)| (k.to_string(), v.to_string())));
        self.e.corre_env(args, stdin, &env)
    }
    fn repo(&self, nombre: &str) -> PathBuf {
        let r = self.e.dir.path().join("repos").join(nombre);
        std::fs::create_dir_all(&r).unwrap();
        git(&r, &["init", "-q"]);
        r
    }
    fn nota(&self, rel: &str, contenido: &str) {
        let p = self.kb.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, contenido).unwrap();
    }
    fn reglas(&self, repo: &str, n: usize) {
        self.nota(&format!("projects/{repo}.md"), &reglas_md(n, 0));
    }
    /// Un commit con trailer `Gate:` sin auto-distill que lo resuelva.
    fn gate_pendiente(&self) {
        git(
            &self.kb,
            &[
                "commit",
                "-q",
                "--allow-empty",
                "-m",
                "docs(kb): x\n\nGate: core/x.md bytes=900 limit=500 routed=bitacora",
            ],
        );
    }
    fn ss(&self, sid: &str) -> Option<String> {
        std::fs::read_to_string(self.state().join("exo-rules").join(format!("ss-{sid}"))).ok()
    }
    fn siembra_reflex(&self, sid: &str, reflejos: &[&str]) {
        let l: String = reflejos
            .iter()
            .map(|r| {
                format!("{{\"ts\":\"t\",\"reflex\":\"{r}\",\"session_id\":\"{sid}\",\"agent_id\":\"\",\"agent_type\":\"\",\"tool\":\"Bash\",\"payload\":\"x\"}}\n")
            })
            .collect();
        std::fs::write(self.log(), l).unwrap();
    }
}

fn reglas_md(n: usize, relleno: usize) -> String {
    let items: String = (1..=n)
        .map(|i| format!("- regla {i}{}\n", "x".repeat(relleno)))
        .collect();
    format!("## Reglas duras\n{items}")
}

fn evt(nombre: &str, sid: &str, cwd: &Path, extra: Value) -> Vec<u8> {
    let mut v = json!({"session_id": sid, "cwd": cwd, "hook_event_name": nombre});
    for (k, x) in extra.as_object().unwrap() {
        v[k] = x.clone();
    }
    v.to_string().into_bytes()
}

fn inicio(sid: &str, cwd: &Path, source: &str) -> Vec<u8> {
    evt("SessionStart", sid, cwd, json!({"source": source}))
}

fn salida(o: &Output) -> Value {
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(
        o.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    serde_json::from_slice(&o.stdout)
        .unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&o.stdout)))
}

fn ctx(v: &Value) -> String {
    v["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

fn payloads(rs: &[Value], id: &str) -> Vec<String> {
    rs.iter()
        .filter(|r| r["reflex"] == id)
        .map(|r| r["payload"].as_str().unwrap().to_string())
        .collect()
}

fn valida_codex(evento: &str, v: &Value) {
    let f = match evento {
        "SessionStart" => "session-start",
        "UserPromptSubmit" => "user-prompt-submit",
        otro => panic!("{otro}"),
    };
    let ruta = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/hook/codex/schema")
        .join(format!("{f}.command.output.schema.json"));
    let schema: Value = serde_json::from_str(&std::fs::read_to_string(ruta).unwrap()).unwrap();
    let validador = jsonschema::validator_for(&schema).unwrap();
    assert!(
        validador.is_valid(v),
        "la salida no valida contra el schema de Codex: {v}"
    );
}

const CONTRATO: &str = "## Contrato de memoria\nCONTRATO-LINEA-1\nCONTRATO-LINEA-2";

#[test]
fn flag_desconocida_rc2() {
    let e = Entorno::nuevo();
    let o = e.corre(&["claude", "SessionStart", "--bogus"], b"{}", &[]);
    assert_eq!(o.status.code(), Some(2));
    assert!(o.stdout.is_empty());
}

/// paridad-recall: engine-stale: version por debajo de ENGINE_MIN ⇒ recall-fallback + aviso en el bloque
#[test]
fn min_desconocido_rc2() {
    let m = Mundo::nuevo();
    let aviso =
        format!("engine desactualizado ({VERSION} < 9.9.9) — actualiza el binario instalado");
    for h in ["claude", "codex"] {
        let o = m.corre(
            &[h, "SessionStart", "--min", "9.9.9"],
            &inicio("s1", &m.kb, "startup"),
            &[],
        );
        let v = salida(&o);
        let sys = v["systemMessage"].as_str().unwrap();
        assert!(sys.contains(&aviso), "{h}: {sys}");
        if h == "claude" {
            assert_eq!(ctx(&v), FALLBACK);
            assert!(!sys.contains("sin reglas duras"), "{sys}");
        } else {
            let c = ctx(&v);
            assert!(c.starts_with(FALLBACK) && c.contains(&aviso), "{c}");
            assert!(sys.contains("sin reglas duras"), "{sys}");
        }
        let rs = m.reflex();
        let propios: Vec<_> = rs.iter().filter(|r| r["harness"] == h).cloned().collect();
        assert_eq!(
            payloads(&propios, "recall-fallback"),
            [format!("reason=engine-stale engine={VERSION} min=9.9.9")],
            "{h}"
        );
    }
    // Con el mínimo igual a la versión propia no hay aviso: arranca normal.
    let o = m.corre(
        &["claude", "SessionStart", "--min", VERSION],
        &inicio("s2", &m.kb, "startup"),
        &[],
    );
    let v = salida(&o);
    assert!(ctx(&v).starts_with("## Contrato de memoria"), "{v}");
    assert!(v.get("systemMessage").is_none(), "{v}");
}

/// paridad-recall: engine_stale_sin_llamar: no se invoca exo rules, systemMessage engine_stale
#[test]
fn stale_no_migra() {
    let m = Mundo::nuevo();
    let repo = m.repo("foo");
    m.reglas("foo", 2);
    let viejo = m.home().join(".claude/reflex-log.jsonl");
    std::fs::create_dir_all(viejo.parent().unwrap()).unwrap();
    std::fs::write(&viejo, "{\"reflex\":\"viejo\"}\n").unwrap();
    let nuevo = m.state().join("reflex-log.jsonl");
    for h in ["claude", "codex"] {
        let o = m.corre(
            &[h, "SessionStart", "--min", "9.9.9"],
            &inicio("s1", &repo, "startup"),
            &[("REFLEX_LOG_FILE", nuevo.to_str().unwrap())],
        );
        let v = salida(&o);
        assert!(
            v["systemMessage"]
                .as_str()
                .unwrap()
                .contains("(engine_stale)"),
            "{v}"
        );
        assert!(!ctx(&v).contains("regla 1"), "{v}");
    }
    assert!(viejo.exists(), "con engine obsoleto no se migra");
    assert!(!std::fs::read_to_string(&nuevo).unwrap().contains("viejo"));
    assert_eq!(m.ss("s1"), None, "con engine obsoleto no hay testigo");
    let rs: Vec<Value> = std::fs::read_to_string(&nuevo)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(
        payloads(&rs, "project-rules-skip"),
        ["reason=engine_stale repo=-", "reason=engine_stale repo=-"]
    );
}

#[test]
fn session_start_claude() {
    let m = Mundo::nuevo();
    let repo = m.repo("foo");
    m.reglas("foo", 2);
    m.gate_pendiente();
    let o = m.corre(
        &["claude", "SessionStart"],
        &inicio("sc", &repo, "startup"),
        &[],
    );
    let v = salida(&o);
    let c = ctx(&v);
    assert!(c.starts_with("## Contrato de memoria"), "{c}");
    assert_eq!(c, format!("{CONTRATO}\n\n{AVISO_GATE}"));
    assert!(!c.contains("Reglas duras") && !c.contains("regla 1"), "{c}");
    assert!(
        v.get("systemMessage").is_none(),
        "reglas OK: silencioso: {v}"
    );
    assert_eq!(
        m.ss("sc").as_deref(),
        Some(r#"{"status":"ok","n":2,"n_global":0,"repo":"foo"}"#)
    );
    assert!(payloads(&m.reflex(), "project-rules-emitted").is_empty());
}

/// paridad-recall: avisos_dentro_del_cap: el gate pendiente entra tras el contrato
#[test]
fn avisos_dentro_del_cap() {
    let m = Mundo::nuevo();
    m.gate_pendiente();
    let o = m.corre(
        &["claude", "SessionStart"],
        &inicio("sc", &m.kb, "startup"),
        &[],
    );
    let c = ctx(&salida(&o));
    assert_eq!(c, format!("{CONTRATO}\n\n{AVISO_GATE}"));
    assert!(c.len() <= 1500, "{}", c.len());
}

#[test]
fn session_start_codex() {
    let m = Mundo::nuevo();
    let repo = m.repo("foo");
    m.reglas("foo", 2);
    m.gate_pendiente();
    let o = m.corre(
        &["codex", "SessionStart"],
        &inicio("sx", &repo, "startup"),
        &[],
    );
    let v = salida(&o);
    valida_codex("SessionStart", &v);
    let c = ctx(&v);
    let bloque = format!("## Reglas duras del proyecto\n{AUTORIDAD}- regla 1\n- regla 2");
    assert_eq!(c, format!("{CONTRATO}\n\n{bloque}\n\n{AVISO_GATE}"));
    assert!(
        v.get("systemMessage").is_none(),
        "reglas OK: silencioso: {v}"
    );
    assert_eq!(m.ss("sx"), None, "nadie lee ss- en Codex");
    let rs = m.reflex();
    let emitido: Vec<_> = rs
        .iter()
        .filter(|r| r["reflex"] == "project-rules-emitted")
        .collect();
    assert_eq!(emitido.len(), 1, "{rs:?}");
    assert_eq!(emitido[0]["payload"], format!("bytes={}", bloque.len()));
    assert_eq!(emitido[0]["harness"], "codex");
    assert!(payloads(&rs, "context-spill-risk").is_empty());
}

#[test]
fn reglas_ambigua_visible() {
    let m = Mundo::nuevo();
    let repo = m.repo("foo");
    m.reglas("foo", 1);
    m.nota("projects/x/foo — b.md", &reglas_md(1, 0));
    let aviso = "⚠ sin reglas de proyecto para foo (ambigua)";
    let o = m.corre(
        &["claude", "SessionStart"],
        &inicio("sc", &repo, "startup"),
        &[],
    );
    let v = salida(&o);
    assert_eq!(v["systemMessage"], aviso);
    assert_eq!(ctx(&v), CONTRATO, "el hook sigue emitiendo el contrato");
    assert_eq!(
        m.ss("sc").as_deref(),
        Some(r#"{"status":"skip","reason":"ambigua","n":0,"n_global":0,"repo":"foo"}"#)
    );
    // En Codex el aviso va además al contexto (`codex exec` no pinta el systemMessage).
    let o = m.corre(
        &["codex", "SessionStart"],
        &inicio("sx", &repo, "startup"),
        &[],
    );
    let v = salida(&o);
    valida_codex("SessionStart", &v);
    assert_eq!(v["systemMessage"], aviso);
    assert_eq!(ctx(&v), format!("{CONTRATO}\n\n{aviso}"));
    assert!(
        payloads(&m.reflex(), "project-rules-emitted").is_empty(),
        "sin bloque no hay emisión"
    );
}

#[test]
fn reglas_skip_rastro() {
    let m = Mundo::nuevo();
    let repo = m.repo("foo");
    m.reglas("foo", 1);
    m.nota("projects/x/foo — b.md", &reglas_md(1, 0));
    for h in ["claude", "codex"] {
        m.corre(&[h, "SessionStart"], &inicio("s1", &repo, "startup"), &[]);
    }
    let rs = m.reflex();
    assert_eq!(
        payloads(&rs, "project-rules-skip"),
        ["reason=ambigua repo=foo", "reason=ambigua repo=foo"]
    );
}

#[test]
fn reglas_error_engine_visible() {
    let m = Mundo::nuevo();
    let repo = m.repo("foo");
    std::fs::create_dir_all(m.kb.join("core/reglas-duras.md")).unwrap();
    let o = m.corre(
        &["claude", "SessionStart"],
        &inicio("sc", &repo, "startup"),
        &[],
    );
    let v = salida(&o);
    assert_eq!(
        v["systemMessage"],
        "⚠ sin reglas de proyecto para foo (error_engine)"
    );
    assert_eq!(ctx(&v), CONTRATO, "el hook sigue emitiendo el contrato");
    let p = payloads(&m.reflex(), "project-rules-skip");
    assert_eq!(p.len(), 1);
    assert!(
        p[0].starts_with("reason=error_engine repo=- err="),
        "{}",
        p[0]
    );
}

#[test]
fn reglas_codex_en_contexto_con_recorte() {
    let m = Mundo::nuevo();
    let repo = m.repo("foo");
    m.nota("projects/foo.md", &reglas_md(12, 0));
    let o = m.corre(
        &["codex", "SessionStart"],
        &inicio("sx", &repo, "startup"),
        &[],
    );
    let v = salida(&o);
    let aviso = "⚠ 2 reglas duras recortadas al máximo de 10 (corre «exo rules --check»)";
    assert_eq!(v["systemMessage"], aviso);
    let c = ctx(&v);
    assert!(c.contains("- regla 10") && !c.contains("- regla 11"), "{c}");
    assert!(c.ends_with(aviso), "{c}");
}

#[test]
fn cap_claude_utf8() {
    let m = Mundo::nuevo();
    let repo = m.repo("foo");
    m.siembra_reflex("sc", &["git-c"]);
    let o = m.corre(
        &["claude", "SessionStart"],
        &inicio("sc", &repo, "compact"),
        &[("EXO_RECALL_NOTA", "kb-test/core/largo")],
    );
    let c = ctx(&salida(&o));
    let (cabeza, cola) = c.split_at(c.find("\n\n--- Reglas reforzadas").unwrap());
    // El '€' empieza en el byte 1499: el cap lo descarta entero en vez de partirlo.
    assert_eq!(cabeza.len(), 1499, "{}", cabeza.len());
    assert!(cabeza.starts_with(&format!("## Contrato de memoria\n{}", "a".repeat(1476))));
    assert!(!cabeza.contains('€') && !cabeza.contains('\u{FFFD}'));
    assert_eq!(cola, REAFIRMA_GIT);
}

/// paridad-recall: avisos_recortan_despues: con contrato lleno el cap come el aviso, no el contrato
#[test]
fn avisos_recortan_despues() {
    let m = Mundo::nuevo();
    m.gate_pendiente();
    let o = m.corre(
        &["claude", "SessionStart"],
        &inicio("sc", &m.kb, "startup"),
        &[("EXO_RECALL_NOTA", "kb-test/core/largo")],
    );
    let c = ctx(&salida(&o));
    assert_eq!(c.len(), 1499, "{}", c.len());
    assert!(c.starts_with(&format!("## Contrato de memoria\n{}", "a".repeat(1476))));
    assert!(!c.contains("gate pendiente"));
}

#[test]
fn recall_truncated_rastro() {
    let m = Mundo::nuevo();
    m.corre(
        &["claude", "SessionStart"],
        &inicio("sc", &m.kb, "startup"),
        &[("EXO_RECALL_NOTA", "kb-test/core/largo")],
    );
    // 23 B de cabecera + 1476 'a' + '€' (3 B) + 100 'b'.
    assert_eq!(
        payloads(&m.reflex(), "recall-truncated"),
        ["bytes=1602 cap=1500"]
    );
    let m = Mundo::nuevo();
    m.corre(
        &["claude", "SessionStart"],
        &inicio("sc", &m.kb, "startup"),
        &[],
    );
    assert!(payloads(&m.reflex(), "recall-truncated").is_empty());
}

#[test]
fn compact_rastro() {
    let m = Mundo::nuevo();
    m.siembra_reflex("sc", &["git-c"]);
    m.corre(
        &["claude", "SessionStart"],
        &inicio("sc", &m.kb, "compact"),
        &[],
    );
    assert_eq!(payloads(&m.reflex(), "compact"), ["compact"]);
    let m = Mundo::nuevo();
    m.corre(
        &["claude", "SessionStart"],
        &inicio("sc", &m.kb, "startup"),
        &[],
    );
    assert!(payloads(&m.reflex(), "compact").is_empty());
}

fn fallback(extra: &[(&str, &str)]) -> (Vec<String>, Value) {
    let m = Mundo::nuevo();
    let o = m.corre(
        &["claude", "SessionStart"],
        &inicio("sc", &m.kb, "startup"),
        extra,
    );
    let v = salida(&o);
    (payloads(&m.reflex(), "recall-fallback"), v)
}

#[test]
fn recall_fallback_no_index() {
    let (p, v) = fallback(&[("EXO_INDEX", "/no/existe/exo.db")]);
    assert_eq!(p, ["reason=no-index"]);
    assert_eq!(ctx(&v), FALLBACK);
}

#[test]
fn recall_fallback_no_config() {
    let (p, v) = fallback(&[("EXO_KB_NAME", "")]);
    assert_eq!(p, ["reason=no-config", "reason=empty"]);
    assert_eq!(ctx(&v), FALLBACK);
}

#[test]
fn recall_fallback_empty() {
    let (p, v) = fallback(&[("EXO_RECALL_NOTA", "kb-test/core/no-existe")]);
    assert_eq!(p, ["reason=empty"]);
    assert_eq!(ctx(&v), FALLBACK);
}

#[test]
fn recall_fallback_no_contract() {
    let (p, v) = fallback(&[("EXO_RECALL_NOTA", "kb-test/core/sin-contrato")]);
    assert_eq!(p, ["reason=no-contract"]);
    assert_eq!(ctx(&v), FALLBACK);
}

#[test]
fn recall_fallback_truncated() {
    // Cap del engine que corta el relleno tras el contrato: se sirve igual.
    let (p, v) = fallback(&[("EXO_RECALL_CAP", "450")]);
    assert_eq!(p, ["reason=truncated"]);
    assert_eq!(ctx(&v), CONTRATO);
}

#[test]
fn recall_fallback_cap_deja_el_bloque_vacio() {
    let (p, v) = fallback(&[("EXO_RECALL_CAP", "10")]);
    assert_eq!(p, ["reason=truncated", "reason=empty"]);
    assert_eq!(ctx(&v), FALLBACK);
}

#[test]
fn codex_fallback_sin_reglas_avisa() {
    let m = Mundo::nuevo();
    let o = m.corre(
        &["codex", "SessionStart"],
        &inicio("sx", &m.kb, "startup"),
        &[("EXO_INDEX", "/no/existe/exo.db")],
    );
    let v = salida(&o);
    assert!(
        v["systemMessage"]
            .as_str()
            .unwrap()
            .contains("sin reglas duras"),
        "{v}"
    );
    assert!(ctx(&v).starts_with(FALLBACK));
}

/// paridad-recall: gate_compact_no_repite
#[test]
fn compact_claude_sin_gate() {
    let m = Mundo::nuevo();
    let repo = m.repo("foo");
    m.reglas("foo", 1);
    m.nota("projects/x/foo — b.md", &reglas_md(1, 0));
    m.gate_pendiente();
    let o = m.corre(
        &["claude", "SessionStart"],
        &inicio("sc", &repo, "compact"),
        &[],
    );
    let v = salida(&o);
    assert_eq!(ctx(&v), CONTRATO, "compact: sin aviso de gate");
    assert!(
        v.get("systemMessage").is_none(),
        "compact no resuelve reglas: {v}"
    );
    assert_eq!(m.ss("sc"), None);
    assert!(payloads(&m.reflex(), "project-rules-skip").is_empty());
    // Control: el mismo mundo en startup sí trae el gate y el testigo.
    let o = m.corre(
        &["claude", "SessionStart"],
        &inicio("sd", &repo, "startup"),
        &[],
    );
    assert!(ctx(&salida(&o)).contains(AVISO_GATE));
    assert!(m.ss("sd").is_some());
}

#[cfg(debug_assertions)]
#[test]
fn panic_subguard() {
    let m = Mundo::nuevo();
    m.gate_pendiente();
    let o = m.corre(
        &["claude", "SessionStart"],
        &inicio("sc", &m.kb, "startup"),
        &[("EXO_HOOK_PANIC", "recall-contrato")],
    );
    let v = salida(&o);
    // El contrato cae al fallback pero el resto del contexto sale igual.
    assert_eq!(ctx(&v), format!("{FALLBACK}\n\n{AVISO_GATE}"));
    assert_eq!(payloads(&m.reflex(), "hook-panic"), ["recall-contrato"]);
}

#[test]
fn post_compact_y_ups() {
    let m = Mundo::nuevo();
    let repo = m.repo("foo");
    m.reglas("foo", 2);
    m.siembra_reflex("sx", &["git-c"]);
    let ups = evt("UserPromptSubmit", "sx", &repo, json!({"prompt": "hola"}));
    // Sin marca, el UPS no inyecta.
    assert_eq!(
        salida(&m.corre(&["codex", "UserPromptSubmit"], &ups, &[])),
        json!({})
    );
    let o = m.corre(
        &["codex", "PostCompact"],
        &evt("PostCompact", "sx", &repo, json!({})),
        &[],
    );
    assert_eq!(salida(&o), json!({}));
    let v = salida(&m.corre(&["codex", "UserPromptSubmit"], &ups, &[]));
    valida_codex("UserPromptSubmit", &v);
    let c = ctx(&v);
    assert!(
        c.contains("## Reglas duras del proyecto") && c.contains("- regla 2"),
        "{c}"
    );
    assert!(c.ends_with(REAFIRMA_GIT.trim_start()), "{c}");
    assert_eq!(v["hookSpecificOutput"]["hookEventName"], "UserPromptSubmit");
    // El segundo UPS ya no inyecta.
    assert_eq!(
        salida(&m.corre(&["codex", "UserPromptSubmit"], &ups, &[])),
        json!({})
    );
    let marcas: Vec<_> = m
        .journal()
        .into_iter()
        .filter(|r| r["kind"] == "mark")
        .map(|r| r["code"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(marcas, ["post-compact", "post-compact-consumed"]);
}

#[test]
fn session_start_compact_consume_marca() {
    let m = Mundo::nuevo();
    let repo = m.repo("foo");
    m.reglas("foo", 2);
    m.siembra_reflex("sx", &["git-c"]);
    m.corre(
        &["codex", "PostCompact"],
        &evt("PostCompact", "sx", &repo, json!({})),
        &[],
    );
    let v = salida(&m.corre(
        &["codex", "SessionStart"],
        &inicio("sx", &repo, "compact"),
        &[],
    ));
    valida_codex("SessionStart", &v);
    let c = ctx(&v);
    assert!(
        c.contains("- regla 2") && c.contains(REAFIRMA_GIT.trim_start()),
        "{c}"
    );
    let ups = evt("UserPromptSubmit", "sx", &repo, json!({"prompt": "hola"}));
    assert_eq!(
        salida(&m.corre(&["codex", "UserPromptSubmit"], &ups, &[])),
        json!({})
    );
    let consumos = m
        .journal()
        .iter()
        .filter(|r| r["code"] == "post-compact-consumed")
        .count();
    assert_eq!(consumos, 1);
}

#[test]
fn spill_canario() {
    let m = Mundo::nuevo();
    let repo = m.repo("foo");
    m.nota("projects/foo.md", &reglas_md(10, 1100));
    let v = salida(&m.corre(
        &["codex", "SessionStart"],
        &inicio("sx", &repo, "startup"),
        &[],
    ));
    let n = ctx(&v).len();
    assert!(n > 10_000, "{n}");
    assert_eq!(
        payloads(&m.reflex(), "context-spill-risk"),
        [format!("bytes={n}")]
    );
    // Hasta el umbral no hay rastro.
    let m2 = Mundo::nuevo();
    let repo2 = m2.repo("foo");
    m2.nota("projects/foo.md", &reglas_md(3, 100));
    m2.corre(
        &["codex", "SessionStart"],
        &inicio("sx", &repo2, "startup"),
        &[],
    );
    assert!(payloads(&m2.reflex(), "context-spill-risk").is_empty());
}

#[test]
fn migra_en_arranque() {
    let m = Mundo::nuevo();
    let claude = m.home().join(".claude");
    std::fs::create_dir_all(&claude).unwrap();
    std::fs::write(claude.join("reflex-log.jsonl"), "{\"reflex\":\"viejo\"}\n").unwrap();
    std::fs::write(claude.join("exo-index.log"), "indexado\n").unwrap();
    let nuevo = m.state().join("reflex-log.jsonl");
    m.corre(
        &["claude", "SessionStart"],
        &inicio("sc", &m.kb, "startup"),
        &[("REFLEX_LOG_FILE", nuevo.to_str().unwrap())],
    );
    assert!(!claude.join("reflex-log.jsonl").exists());
    assert!(
        std::fs::read_to_string(&nuevo)
            .unwrap()
            .contains("\"viejo\"")
    );
    assert!(!claude.join("exo-index.log").exists());
    assert_eq!(
        std::fs::read_to_string(m.state().join("exo-index.log")).unwrap(),
        "indexado\n"
    );
}

#[cfg(unix)]
#[test]
fn migra_fallo_rastro() {
    use std::os::unix::fs::PermissionsExt;
    let m = Mundo::nuevo();
    let ro = m.dir.path().join("ro");
    std::fs::create_dir_all(&ro).unwrap();
    std::fs::set_permissions(&ro, std::fs::Permissions::from_mode(0o555)).unwrap();
    // Como root el modo 0o555 no se respeta y el fallo no se produce: no hay nada que probar.
    if std::fs::write(ro.join("sonda"), "").is_ok() {
        eprintln!(
            "migra_fallo_rastro: omitido, el usuario escribe en un dir de solo lectura (root)"
        );
        return;
    }
    let claude = m.home().join(".claude");
    std::fs::create_dir_all(&claude).unwrap();
    std::fs::write(claude.join("exo-index.log"), "indexado\n").unwrap();
    let o = m.corre(
        &["claude", "SessionStart"],
        &inicio("sc", &m.kb, "startup"),
        &[("EXO_STATE_DIR", ro.to_str().unwrap())],
    );
    std::fs::set_permissions(&ro, std::fs::Permissions::from_mode(0o755)).unwrap();
    let v = salida(&o);
    assert!(
        ctx(&v).starts_with("## Contrato de memoria"),
        "sale igual: {v}"
    );
    let p = payloads(&m.reflex(), "state-migrate-failed");
    assert_eq!(p.len(), 1, "{p:?}");
    assert!(p[0].starts_with("exo-index.log: "), "{}", p[0]);
    assert!(
        claude.join("exo-index.log").exists(),
        "el viejo queda intacto"
    );
}

#[cfg(unix)]
fn ejecutable(ruta: &Path, texto: &str) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::write(ruta, texto).unwrap();
    std::fs::set_permissions(ruta, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(unix)]
#[test]
fn stop_lanza_index() {
    let m = Mundo::nuevo();
    let stub = m.dir.path().join("exo-stub");
    let volcado = m.dir.path().join("args.txt");
    ejecutable(&stub, "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$VOLCADO\"\n");
    for (h, esperado) in [("claude", ""), ("codex", "{}")] {
        let _ = std::fs::remove_file(&volcado);
        let o = m.corre(
            &[h, "Stop"],
            &evt("Stop", "s", &m.kb, json!({})),
            &[
                ("EXO_BIN", stub.to_str().unwrap()),
                ("VOLCADO", volcado.to_str().unwrap()),
            ],
        );
        assert_eq!(o.status.code(), Some(0));
        assert_eq!(String::from_utf8_lossy(&o.stdout), esperado, "{h}");
        let t0 = Instant::now();
        while !volcado.exists() || std::fs::read_to_string(&volcado).unwrap().is_empty() {
            assert!(
                t0.elapsed() < Duration::from_secs(10),
                "{h}: el stub no recibió index"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(
            std::fs::read_to_string(&volcado).unwrap(),
            format!("index\n--db\n{}\n--json\n", m.db.display()),
            "{h}"
        );
    }
}

/// El hook no hereda el stdout al hijo desacoplado: vuelve sin esperar al indexado lento.
#[cfg(unix)]
#[test]
fn stop_no_espera_al_stub_lento() {
    let m = Mundo::nuevo();
    let stub = m.dir.path().join("exo-lento");
    let pidf = m.dir.path().join("pid");
    ejecutable(&stub, "#!/bin/sh\necho $$ > \"$PIDF\"\nexec sleep 20\n");
    let mut cmd = m.e.comando(
        &["claude", "Stop"],
        &[
            ("EXO_BIN".into(), stub.display().to_string()),
            ("EXO_INDEX".into(), m.db.display().to_string()),
            ("PIDF".into(), pidf.display().to_string()),
        ],
    );
    let mut hijo = cmd.spawn().unwrap();
    hijo.stdin
        .take()
        .unwrap()
        .write_all(&evt("Stop", "s", &m.kb, json!({})))
        .unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let t0 = Instant::now();
    std::thread::spawn(move || {
        let _ = tx.send(hijo.wait_with_output());
    });
    let resultado = rx.recv_timeout(Duration::from_secs(10));
    let tardo = t0.elapsed();
    // El stub quedó vivo (desacoplado): se mata antes de afirmar nada.
    let t1 = Instant::now();
    while !pidf.exists() && t1.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(20));
    }
    if let Ok(pid) = std::fs::read_to_string(&pidf) {
        let _ = Command::new("kill").arg(pid.trim()).status();
    }
    let o = resultado
        .expect("el hook no volvió en 10 s: sujeta el pipe del stub")
        .unwrap();
    assert_eq!(o.status.code(), Some(0));
    assert!(tardo < Duration::from_secs(8), "{tardo:?}");
    assert!(pidf.exists(), "el stub tenía que haberse lanzado");
}

#[cfg(unix)]
#[test]
fn gate_timeout_rastro() {
    let m = Mundo::nuevo();
    m.gate_pendiente();
    let inicio_ = inicio("sc", &m.kb, "startup");
    // Control: con git real el aviso sale y no hay rastro.
    let c = ctx(&salida(&m.corre(
        &["claude", "SessionStart"],
        &inicio_,
        &[],
    )));
    assert!(c.contains(AVISO_GATE), "{c}");
    assert!(payloads(&m.reflex(), "gate-timeout").is_empty());

    // `git` de PATH que se cuelga solo en la primera consulta del gate.
    let real = String::from_utf8(
        Command::new("sh")
            .args(["-c", "command -v git"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let dir = m.dir.path().join("bin");
    std::fs::create_dir_all(&dir).unwrap();
    ejecutable(
        &dir.join("git"),
        &format!(
            "#!/bin/sh\ncase \"$*\" in *distill/last*) exec sleep 30 ;; esac\nexec {} \"$@\"\n",
            real.trim()
        ),
    );
    let path = format!("{}:{}", dir.display(), std::env::var("PATH").unwrap());
    let t0 = Instant::now();
    let v = salida(&m.corre(
        &["claude", "SessionStart"],
        &inicio_,
        &[("PATH", &path), ("EXO_GIT_TIMEOUT", "1")],
    ));
    assert!(t0.elapsed() < Duration::from_secs(10));
    assert_eq!(ctx(&v), CONTRATO, "sin aviso de gate pero con contrato");
    assert_eq!(payloads(&m.reflex(), "gate-timeout"), ["deadline=1s"]);
}
