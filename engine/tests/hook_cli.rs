use serde_json::{Value, json};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

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
    fn corre(&self, args: &[&str], stdin: &[u8], extra: &[(&str, &str)]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_exo"));
        cmd.arg("hook")
            .args(args)
            .env("EXO_STATE_DIR", self.state())
            .env("REFLEX_LOG_FILE", self.log())
            .env_remove("CDPATH")
            .env_remove("EXO_HOOK_PANIC")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (k, v) in extra {
            cmd.env(k, v);
        }
        let mut hijo = cmd.spawn().unwrap();
        hijo.stdin.take().unwrap().write_all(stdin).unwrap();
        hijo.wait_with_output().unwrap()
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
