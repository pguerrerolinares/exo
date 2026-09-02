//! `exo targets` contra el binario real.
//!
//! No usa `tests/common/mod.rs`: estos tests pasan `--db` y `--kb` explícitos,
//! así que `resuelve_db`/`resuelve_kb` cortan en el flag y nunca llegan a
//! cargar config. Declarar el módulo sin usarlo sería un warning, y clippy es
//! gate duro.

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_exo")
}

/// Reusa el montaje de `tests/objetivos.rs` a través de un helper local: KB
/// con git y su índice poblado a mano.
fn kb_con_indice() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let kb = dir.path().to_path_buf();
    let cfg = kb.join("gitconfig-vacio");
    std::fs::write(&cfg, "").unwrap();
    let git = |args: &[&str]| {
        let s = Command::new("git")
            .arg("-C")
            .arg(&kb)
            .args(args)
            .env("GIT_CONFIG_GLOBAL", &cfg)
            .env("GIT_CONFIG_SYSTEM", &cfg)
            .env("GIT_AUTHOR_NAME", "f")
            .env("GIT_AUTHOR_EMAIL", "f@k.local")
            .env("GIT_COMMITTER_NAME", "f")
            .env("GIT_COMMITTER_EMAIL", "f@k.local")
            .env("GIT_AUTHOR_DATE", "2026-07-01T10:00:00+02:00")
            .env("GIT_COMMITTER_DATE", "2026-07-01T10:00:00+02:00")
            .output()
            .unwrap();
        assert!(s.status.success(), "git {args:?}");
    };
    std::fs::create_dir_all(kb.join("log")).unwrap();
    std::fs::write(
        kb.join("log/alpha.md"),
        "---\ntier: stable\n---\n# alpha\ncuerpo de alpha\n",
    )
    .unwrap();
    git(&["init", "-q"]);
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "inicial"]);

    let db = dir.path().join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES ('kb/log/alpha', 'log/alpha.md', 'alpha', 'note', 0.0, NULL)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO notas_fts (titulo, cuerpo, permalink)
         VALUES ('alpha', 'cuerpo de alpha', 'kb/log/alpha')",
        [],
    )
    .unwrap();
    drop(conn);
    (dir, db)
}

#[test]
fn el_envelope_lleva_command_targets_y_schema_version_2() {
    let (dir, db) = kb_con_indice();
    let salida = Command::new(bin())
        .args(["targets", "--json", "--limit", "5"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("alpha")
        .output()
        .unwrap();

    assert!(
        salida.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(v["schema_version"], 2);
    assert_eq!(v["command"], "targets");
    assert_eq!(v["data"]["topic"], "alpha");
    let c = &v["data"]["candidates"][0];
    assert_eq!(c["permalink"], "kb/log/alpha");
    assert_eq!(c["tier"], "stable");
    assert_eq!(c["last_commit"], "2026-07-01T10:00:00+02:00");
    assert!(c["size_bytes"].as_i64().unwrap() > 0);
}

// Las claves de data van en inglés (D8) y las colecciones vacías serializan
// como `[]`, nunca como `null`: un consumidor que haga `.candidates[]` con jq
// no puede encontrarse un null.
#[test]
fn sin_candidatas_el_array_es_vacio_no_nulo() {
    let (dir, db) = kb_con_indice();
    let salida = Command::new(bin())
        .args(["targets", "--json"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("zzz-inexistente")
        .output()
        .unwrap();
    assert!(salida.status.success());
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert!(v["data"]["candidates"].is_array());
    assert_eq!(v["data"]["candidates"].as_array().unwrap().len(), 0);
}

#[test]
fn un_limite_cero_falla_sin_ensuciar_stdout() {
    let (dir, db) = kb_con_indice();
    let salida = Command::new(bin())
        .args(["targets", "--json", "--limit", "0"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("alpha")
        .output()
        .unwrap();
    assert!(!salida.status.success());
    assert!(
        salida.stdout.is_empty(),
        "stdout tiene que quedar limpio ante error"
    );
}

#[test]
fn un_tema_vacio_falla() {
    let (dir, db) = kb_con_indice();
    let salida = Command::new(bin())
        .args(["targets", "--json"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("   ")
        .output()
        .unwrap();
    assert!(!salida.status.success());
}

#[test]
fn la_salida_humana_nombra_el_permalink() {
    let (dir, db) = kb_con_indice();
    let salida = Command::new(bin())
        .args(["targets"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("alpha")
        .output()
        .unwrap();
    assert!(salida.status.success());
    let texto = String::from_utf8_lossy(&salida.stdout);
    assert!(texto.contains("kb/log/alpha"), "salida: {texto}");
}

// Una DB inexistente no debe crearse como efecto colateral de un typo en
// --db: los comandos de solo lectura comprueban antes de abrir.
#[test]
fn una_db_inexistente_falla_y_no_se_crea() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("no-existe.db");
    let salida = Command::new(bin())
        .args(["targets", "--json"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("alpha")
        .output()
        .unwrap();
    assert!(!salida.status.success());
    assert!(!db.exists(), "un typo en --db no puede crear el fichero");
}
