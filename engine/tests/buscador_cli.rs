//! `exo search` contra el binario real, modo humano y `--json`.
//!
//! No usa `tests/common/mod.rs`: estos tests pasan `--db` y `--kb` explícitos,
//! así que `resuelve_db`/`resuelve_kb` cortan en el flag y nunca cargan config.

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_exo")
}

/// KB con una nota en un subdirectorio (el subdirectorio importa: el defecto
/// original era un separador INTERIOR) y su índice poblado a mano.
fn kb_con_indice() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let kb = dir.path().to_path_buf();
    std::fs::create_dir_all(kb.join("log")).unwrap();
    std::fs::write(
        kb.join("log/alpha.md"),
        "---\ntier: stable\n---\n# alpha\ncuerpo de alpha\n",
    )
    .unwrap();

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

fn busca(kb: &std::path::Path, db: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(["search"])
        .arg("--db")
        .arg(db)
        .arg("--kb")
        .arg(kb)
        .args(args)
        .arg("alpha")
        .output()
        .unwrap()
}

#[test]
fn la_salida_humana_lleva_cuatro_columnas_y_la_cuarta_es_la_ruta() {
    let (dir, db) = kb_con_indice();
    let salida = busca(dir.path(), &db, &[]);
    assert!(salida.status.success());
    let texto = String::from_utf8_lossy(&salida.stdout);
    let linea = texto.lines().next().expect("al menos un resultado");
    let cols: Vec<&str> = linea.split('\t').collect();
    assert_eq!(cols.len(), 4, "linea: {linea:?}");
    assert_eq!(cols[0], "kb/log/alpha");
    assert!(
        std::path::Path::new(cols[3]).is_file(),
        "la 4a columna debe existir en disco: {}",
        cols[3]
    );
}

#[test]
fn la_ruta_humana_no_lleva_barra_invertida() {
    let (dir, db) = kb_con_indice();
    let salida = busca(dir.path(), &db, &[]);
    let texto = String::from_utf8_lossy(&salida.stdout);
    let linea = texto.lines().next().expect("al menos un resultado");
    let ruta = linea.split('\t').nth(3).unwrap();
    // La cadena ENTERA, no el prefijo: el defecto era un separador interior.
    assert!(!ruta.contains('\\'), "ruta: {ruta}");
}

#[test]
fn sin_ruta_la_columna_lo_dice_y_stderr_avisa() {
    // Índice inconsistente a propósito: el permalink está en notas_fts pero no
    // en notas, que es el único caso en que `path` es None.
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    conn.execute("DELETE FROM notas WHERE permalink = 'kb/log/alpha'", [])
        .unwrap();
    drop(conn);

    let salida = busca(dir.path(), &db, &[]);
    assert!(salida.status.success());
    let texto = String::from_utf8_lossy(&salida.stdout);
    let linea = texto.lines().next().expect("al menos un resultado");
    assert_eq!(linea.split('\t').nth(3).unwrap(), "(sin-ruta:rebuild)");

    // Media señal es la que no grita: marcador Y aviso, en el mismo test.
    let err = String::from_utf8_lossy(&salida.stderr);
    assert!(
        err.contains("sin ruta (índice inconsistente)"),
        "stderr: {err}"
    );
    assert!(err.contains("exo rebuild"), "stderr: {err}");
}

#[test]
fn el_marcador_no_lleva_whitespace() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    conn.execute("DELETE FROM notas WHERE permalink = 'kb/log/alpha'", [])
        .unwrap();
    drop(conn);

    let salida = busca(dir.path(), &db, &[]);
    let texto = String::from_utf8_lossy(&salida.stdout);
    let marcador = texto.lines().next().unwrap().split('\t').nth(3).unwrap();
    // Sin esto, `awk '{print $4}'` (sin -F, el habito mas comun) imprime
    // solo el primer trozo del marcador.
    assert!(
        !marcador.chars().any(char::is_whitespace),
        "marcador: {marcador:?}"
    );
}

#[test]
fn el_aviso_no_entra_en_el_envelope() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    conn.execute("DELETE FROM notas WHERE permalink = 'kb/log/alpha'", [])
        .unwrap();
    drop(conn);

    let salida = busca(dir.path(), &db, &["--json"]);
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    // El aviso de ruta ausente va SOLO a stderr: `path: null` ya es inferible
    // desde `results`, y `warnings` es para lo que NO se puede inferir.
    let avisos = v["data"]["warnings"].as_array().cloned().unwrap_or_default();
    assert!(
        !avisos.iter().any(|a| a.as_str().unwrap_or("").contains("sin ruta")),
        "warnings: {avisos:?}"
    );
}

#[test]
fn el_json_no_cambia() {
    // Ejerce el BINARIO, no el struct: `contrato_envelope.rs` ya serializa el
    // struct a mano y por eso no habria atrapado nada de este incidente.
    let (dir, db) = kb_con_indice();
    let salida = busca(dir.path(), &db, &["--json"]);
    assert!(salida.status.success());
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(v["schema_version"], 2);
    assert_eq!(v["command"], "search");
    for clave in ["elapsed_s", "query", "results", "search_type"] {
        assert!(v["data"].get(clave).is_some(), "falta .data.{clave}: {v}");
    }
    let r0 = &v["data"]["results"][0];
    assert_eq!(r0["permalink"], "kb/log/alpha");
    assert_eq!(r0["type"], "entity");
    // El JSON sigue dando la ruta RELATIVA (la absoluta es solo de la humana).
    assert_eq!(r0["path"], "log/alpha.md");
}

#[test]
fn sin_kb_resoluble_el_modo_humano_falla_con_remedio() {
    let (dir, db) = kb_con_indice();
    let salida = Command::new(bin())
        .args(["search"])
        .arg("--db")
        .arg(&db)
        .arg("alpha")
        .env("EXO_CONFIG", dir.path().join("no-existe.toml"))
        .env_remove("EXO_KB")
        .output()
        .unwrap();
    assert!(!salida.status.success(), "debe fallar, no dar ruta relativa");
    let err = String::from_utf8_lossy(&salida.stderr);
    assert!(err.contains("--kb"), "el error debe nombrar el remedio: {err}");
}
