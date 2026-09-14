//! `exo search` sin resultados: contrato análogo a `exo targets` (que ya
//! imprime `no candidates`, `engine/src/main.rs:1004`). Antes de esta
//! tarea, `busca_cmd` volvía `Ok(())` sin imprimir nada — una terminal en
//! blanco indistinguible de "no filtré la salida" (docs/backlog.md).
//!
//! No usa `tests/common/mod.rs`: pasa `--db` explícito, así que
//! `resuelve_db` corta en el flag antes de cargar config. `search` no tiene
//! `--kb`, así que la KB esperada sale de `resuelve_kb(None)` — sin
//! `$EXO_KB` ni config, resuelve a `None` (aviso `Option`, nunca error), lo
//! mismo que otros tests de `search` sin `EXO_CONFIG` puesto
//! (`kb_root_lectura_cli.rs::search_sin_kb_resoluble_no_avisa_y_no_falla`).

use std::path::Path;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_exo")
}

/// DB con UNA nota indexada por FTS, para poder buscar algo que SÍ matchea
/// (test de no-regresión) y algo que no matchea nada (el caso nuevo).
fn db_con_una_nota(dir: &Path) -> std::path::PathBuf {
    let db = dir.join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES ('kb/a', 'a.md', 'a', 'note', 0.0, NULL)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO notas_fts (titulo, cuerpo, permalink)
         VALUES ('a', 'contenido buscable de a', 'kb/a')",
        [],
    )
    .unwrap();
    drop(conn);
    db
}

#[test]
fn sin_resultados_imprime_no_results_y_sale_0() {
    let dir = tempfile::tempdir().unwrap();
    let db = db_con_una_nota(dir.path());

    let out = Command::new(bin())
        .args(["search", "--db"])
        .arg(&db)
        .arg("zzz-query-que-no-matchea-nada")
        .env_remove("EXO_CONFIG")
        .env_remove("EXO_KB")
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "sin resultados sigue siendo exit 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout, "no results\n",
        "stdout debe ser exactamente 'no results', igual que 'no candidates' en targets: {stdout:?}"
    );
}

#[test]
fn con_resultados_no_imprime_no_results() {
    let dir = tempfile::tempdir().unwrap();
    let db = db_con_una_nota(dir.path());

    let out = Command::new(bin())
        .args(["search", "--db"])
        .arg(&db)
        .arg("buscable")
        .env_remove("EXO_CONFIG")
        .env_remove("EXO_KB")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("no results"),
        "con resultados reales no debe aparecer 'no results': {stdout:?}"
    );
    assert!(
        stdout.contains("kb/a"),
        "la búsqueda debía seguir encontrando la nota: {stdout:?}"
    );
}

#[test]
fn json_sin_resultados_no_gana_ninguna_clave_nueva() {
    let dir = tempfile::tempdir().unwrap();
    let db = db_con_una_nota(dir.path());

    let out = Command::new(bin())
        .args(["search", "--db"])
        .arg(&db)
        .arg("--json")
        .arg("zzz-query-que-no-matchea-nada")
        .env_remove("EXO_CONFIG")
        .env_remove("EXO_KB")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("no results"),
        "el texto 'no results' es SOLO del modo texto plano, nunca del envelope JSON: {stdout}"
    );
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(v["schema_version"], 2);
    assert_eq!(v["command"], "search");
    assert!(v["data"]["results"].as_array().unwrap().is_empty());
    // Ninguna clave nueva en `data`: mismo conjunto de claves que antes de
    // esta tarea (results, avisos-si-los-hay — nada de un flag "empty").
    let claves: std::collections::BTreeSet<&str> = v["data"]
        .as_object()
        .unwrap()
        .keys()
        .map(|s| s.as_str())
        .collect();
    assert!(
        !claves.contains("no_results") && !claves.contains("empty"),
        "el envelope no debe ganar una clave nueva para este caso: {claves:?}"
    );
}
