//! H2/H3 en la superficie de CLI: `exo recall --query` avisa por stderr y
//! publica `warnings`, `elapsed_s` y `refresh_s` en el envelope.
mod common;

use std::process::Command;

#[test]
fn recall_query_con_arm_vector_inerte_avisa_y_publica_tiempos() {
    let kb = tempfile::tempdir().unwrap();
    std::fs::write(
        kb.path().join("a.md"),
        "---\npermalink: kb-test/a\ntitle: A\n---\ncontenido buscable de la nota a\n",
    )
    .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("i.db");
    let cfg = dir.path().join("config.toml");
    std::fs::write(&cfg, common::render_config(kb.path(), "kb-test", &db)).unwrap();

    let idx = Command::new(env!("CARGO_BIN_EXE_exo"))
        .args(["index", "--kb"])
        .arg(kb.path())
        .arg("--db")
        .arg(&db)
        .env("EXO_CONFIG", &cfg)
        .output()
        .unwrap();
    assert!(
        idx.status.success(),
        "{}",
        String::from_utf8_lossy(&idx.stderr)
    );
    exo::abre_db(&db)
        .unwrap()
        .execute("DELETE FROM vectores", [])
        .unwrap();

    let recall = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_exo"))
            .args(["recall", "--kb"])
            .arg(kb.path())
            .arg("--db")
            .arg(&db)
            .args(["--query=buscable", "--min-similarity", "0.0", "--json"])
            .args(extra)
            .env("EXO_CONFIG", &cfg)
            .output()
            .unwrap()
    };

    let sin = recall(&[]);
    assert!(
        sin.status.success(),
        "{}",
        String::from_utf8_lossy(&sin.stderr)
    );
    let err = String::from_utf8_lossy(&sin.stderr);
    assert!(
        err.lines()
            .any(|l| l.starts_with("aviso: ") && l.contains("INERTE")),
        "el aviso sale por stderr: {err}"
    );
    let v: serde_json::Value = serde_json::from_slice(&sin.stdout).unwrap();
    assert!(
        v["data"]["warnings"][0]
            .as_str()
            .is_some_and(|w| w.contains("INERTE")),
        "{v}"
    );
    assert!(v["data"]["elapsed_s"].is_number(), "{v}");
    assert!(v["data"]["refresh_s"].is_null(), "sin --refresh: {v}");

    let con = recall(&["--refresh"]);
    assert!(
        con.status.success(),
        "{}",
        String::from_utf8_lossy(&con.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&con.stdout).unwrap();
    assert!(v["data"]["refresh_s"].is_number(), "con --refresh: {v}");
}
