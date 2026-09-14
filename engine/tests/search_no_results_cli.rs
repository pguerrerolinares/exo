//! `exo search` sin resultados: contrato análogo a `exo targets` (que ya
//! imprime `no candidates`, `engine/src/main.rs:1004`). Antes de esta
//! tarea, `busca_cmd` volvía `Ok(())` sin imprimir nada — una terminal en
//! blanco indistinguible de "no filtré la salida" (docs/backlog.md).
//!
//! No usa `tests/common/mod.rs`: pasa `--db` explícito, así que
//! `resuelve_db` corta en el flag antes de cargar config. `search` no tiene
//! `--kb`, así que la KB esperada sale de `resuelve_kb(None)` — sin
//! `$EXO_KB` ni config resoluble, resuelve a `None` (aviso `Option`, nunca
//! error). Hermético: `EXO_CONFIG` apunta a un fichero inexistente (mismo
//! patrón que `help_producto.rs` y
//! `kb_root_lectura_cli.rs::search_sin_kb_resoluble_no_avisa_y_no_falla`),
//! nunca `env_remove` — sin esto el test cae en `~/.exo/config.toml` real
//! si esta máquina tiene uno, y deja de ser hermético.

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
        .env("EXO_CONFIG", "C:/no-existe-jamas/config.toml")
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
        .env("EXO_CONFIG", "C:/no-existe-jamas/config.toml")
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
        .env("EXO_CONFIG", "C:/no-existe-jamas/config.toml")
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
    // Conjunto EXACTO de claves de `data` (forma de `Busqueda`,
    // `src/buscador.rs`): `query`/`search_type`/`elapsed_s`/`results`
    // siempre; `warnings` solo si `avisos` no está vacío
    // (`skip_serializing_if = "Vec::is_empty"`) — sin KB resoluble en este
    // caso no hay aviso que emitir, así que no aparece. Comparar el
    // conjunto entero, no solo excluir dos nombres inventados: así una
    // clave nueva cualquiera (no solo "no_results"/"empty") hace fallar el
    // test.
    let claves: std::collections::BTreeSet<&str> = v["data"]
        .as_object()
        .unwrap()
        .keys()
        .map(|s| s.as_str())
        .collect();
    let esperadas: std::collections::BTreeSet<&str> =
        ["query", "search_type", "elapsed_s", "results"]
            .into_iter()
            .collect();
    assert_eq!(
        claves, esperadas,
        "el envelope no debe ganar (ni perder) una clave en `data` para este caso"
    );
}
