//! `exo stale` contra el binario real, y contra el golden de kbx (oráculo
//! de valores, no comparación binario-contra-binario — ver el pre-registro
//! de la campaña D, sección "Stale").
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_exo")
}

fn kb_con_indice_y_git() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let kb = dir.path().to_path_buf();
    let cfg = kb.join("gitconfig-vacio");
    std::fs::write(&cfg, "").unwrap();
    let git = |args: &[&str]| {
        let s = std::process::Command::new("git")
            .arg("-C")
            .arg(&kb)
            .args(args)
            .env("GIT_CONFIG_GLOBAL", &cfg)
            .env("GIT_CONFIG_SYSTEM", &cfg)
            .env("GIT_AUTHOR_NAME", "f")
            .env("GIT_AUTHOR_EMAIL", "f@k.local")
            .env("GIT_COMMITTER_NAME", "f")
            .env("GIT_COMMITTER_EMAIL", "f@k.local")
            .env("GIT_AUTHOR_DATE", "2026-06-01T10:00:00+02:00")
            .env("GIT_COMMITTER_DATE", "2026-06-01T10:00:00+02:00")
            .output()
            .unwrap();
        assert!(s.status.success(), "git {args:?}");
    };
    std::fs::write(kb.join("a.md"), "---\ntier: core\n---\n\n# a\n").unwrap();
    git(&["init", "-q"]);
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "inicial"]);

    let db = dir.path().join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch) VALUES ('kb/a', 'a.md', 'a', 'note', 0.0, NULL)",
        [],
    ).unwrap();
    drop(conn);
    (dir, db)
}

#[test]
fn el_envelope_lleva_command_stale_y_schema_version_2() {
    let (dir, db) = kb_con_indice_y_git();
    let salida = Command::new(bin())
        .args(["stale", "--json", "--now", "2026-09-14T00:00:00Z"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(
        salida.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(v["schema_version"], 2);
    assert_eq!(v["command"], "stale");
    assert_eq!(v["data"]["now"], "2026-09-14T00:00:00Z");
    assert_eq!(v["data"]["notes"][0]["path"], "a.md");
    assert_eq!(v["data"]["notes"][0]["tier"], "core");
    // El score ya no se fuerza a 2 decimales fijos en el texto (ver
    // Global Constraints y divergencia 5 del pre-registro): el valor es lo
    // que importa, no el formato. Se compara por
    // VALOR, vía el `Value` ya parseado, no con un patrón sobre el texto
    // crudo.
    let score = v["data"]["notes"][0]["score"].as_f64().unwrap();
    assert!(
        score > 0.0,
        "a.md tiene commit y tier=core: el score debe ser positivo, no {score}"
    );
}

#[test]
fn sin_now_usa_el_reloj_de_pared_y_no_falla() {
    let (dir, db) = kb_con_indice_y_git();
    let salida = Command::new(bin())
        .args(["stale", "--json"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(
        salida.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
}

#[test]
fn now_invalido_falla_como_error_de_uso() {
    let (dir, db) = kb_con_indice_y_git();
    let salida = Command::new(bin())
        .args(["stale", "--now", "no-es-una-fecha"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(!salida.status.success());
}

#[test]
fn la_salida_humana_nombra_el_path_y_el_score() {
    let (dir, db) = kb_con_indice_y_git();
    let salida = Command::new(bin())
        .args(["stale", "--now", "2026-09-14T00:00:00Z"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(salida.status.success());
    let texto = String::from_utf8_lossy(&salida.stdout);
    assert!(texto.contains("a.md"), "salida: {texto}");
    assert!(texto.contains("score="), "salida: {texto}");
}
