//! Aviso de LECTURA (agravante silencioso del guard H1 de escritura,
//! `comprueba_kb_root` en `engine/src/indexer.rs`): `search`/`recall` avisan
//! por stderr —y, en `recall --json`, también en `warnings`— cuando la DB
//! resuelta trae `meta.kb_root` de OTRA KB que sigue en disco. Nunca abortan
//! (exit 0, resultados intactos): mismo criterio indulgente que el camino de
//! escritura, solo que aquí no hay `bail!`.
//!
//! No usa `tests/common/mod.rs` para los tests de `recall`: pasan `--db` y
//! `--kb` explícitos, así que `resuelve_db`/`resuelve_kb` cortan en el flag y
//! nunca llegan a cargar config (mismo motivo que documenta
//! `tests/targets_cli.rs::kb_con_indice`). Los de `search` sí necesitan un
//! `config.toml` propio vía `EXO_CONFIG` puesto en el propio `Command`
//! (nunca en el proceso del test): `search` no tiene `--kb`, así que la KB
//! esperada solo puede venir de la config.

mod common;

use std::path::Path;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_exo")
}

/// DB hand-craft (mismo patrón que `targets_cli.rs::kb_con_indice`): schema
/// vacío + una nota mínima —para que `recall` en modo arranque no bail-ee
/// "recall vacío" antes de llegar al aviso— y, opcionalmente, `meta.kb_root`
/// apuntando a `kb_root`. Sin cargar el modelo de embeddings: ni `exo index`
/// ni `con_embedder_de_proceso` intervienen, así que montar esta fixture es
/// obra de milisegundos, no de la carga de fastembed.
fn db_con_kb_root(dir: &Path, kb_root: Option<&Path>) -> std::path::PathBuf {
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
    if let Some(kb_root) = kb_root {
        let abs = std::fs::canonicalize(kb_root).unwrap();
        conn.execute(
            "INSERT INTO meta (clave, valor) VALUES ('kb_root', ?1)",
            [abs.to_string_lossy().to_string()],
        )
        .unwrap();
    }
    drop(conn);
    db
}

fn tiene_aviso(stderr: &str) -> bool {
    stderr.lines().any(|l| l.starts_with("aviso: "))
}

// ---------------------------------------------------------------------
// recall
// ---------------------------------------------------------------------

#[test]
fn recall_avisa_si_la_db_es_de_otra_kb_que_sigue_en_disco() {
    let kb_vieja = tempfile::tempdir().unwrap();
    let kb_pedida = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let db = db_con_kb_root(dir.path(), Some(kb_vieja.path()));

    let out = Command::new(bin())
        .args(["recall", "--kb"])
        .arg(kb_pedida.path())
        .arg("--db")
        .arg(&db)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "exit code intacto: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let err = String::from_utf8_lossy(&out.stderr);
    let kb_vieja_abs = std::fs::canonicalize(kb_vieja.path()).unwrap();
    let kb_pedida_abs = std::fs::canonicalize(kb_pedida.path()).unwrap();
    assert!(
        err.lines().any(|l| {
            l.starts_with("aviso: ")
                && l.contains(&kb_vieja_abs.display().to_string())
                && l.contains(&kb_pedida_abs.display().to_string())
        }),
        "esperaba un aviso nombrando las dos rutas: {err}"
    );
}

#[test]
fn recall_json_publica_el_aviso_en_warnings() {
    let kb_vieja = tempfile::tempdir().unwrap();
    let kb_pedida = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let db = db_con_kb_root(dir.path(), Some(kb_vieja.path()));

    let out = Command::new(bin())
        .args(["recall", "--kb"])
        .arg(kb_pedida.path())
        .arg("--db")
        .arg(&db)
        .arg("--json")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "exit code intacto: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let kb_vieja_abs = std::fs::canonicalize(kb_vieja.path()).unwrap();
    assert!(
        v["data"]["warnings"]
            .as_array()
            .is_some_and(|ws| ws.iter().any(|w| w
                .as_str()
                .is_some_and(|s| s.contains(&kb_vieja_abs.display().to_string())))),
        "el aviso debía estar en `warnings`: {v}"
    );
    // Resultados intactos: la nota de la DB sigue sirviéndose pese al aviso.
    assert!(
        !v["data"]["notes"].as_array().unwrap().is_empty(),
        "el aviso no debía vaciar los resultados: {v}"
    );
}

#[test]
fn recall_no_avisa_si_la_kb_pedida_es_la_misma_del_indice() {
    let kb = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let db = db_con_kb_root(dir.path(), Some(kb.path()));

    let out = Command::new(bin())
        .args(["recall", "--kb"])
        .arg(kb.path())
        .arg("--db")
        .arg(&db)
        .output()
        .unwrap();
    assert!(out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!tiene_aviso(&err), "misma KB, no debía avisar: {err}");
}

#[test]
fn recall_no_avisa_si_la_kb_previa_ya_no_existe_en_disco() {
    let kb_vieja = tempfile::tempdir().unwrap();
    let kb_vieja_abs = std::fs::canonicalize(kb_vieja.path()).unwrap();
    let kb_pedida = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();

    let db = dir.path().join("index.db");
    {
        let conn = exo::abre_db(&db).unwrap();
        exo::schema::crea_schema(&conn).unwrap();
        conn.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
             VALUES ('kb/a', 'a.md', 'a', 'note', 0.0, NULL)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO meta (clave, valor) VALUES ('kb_root', ?1)",
            [kb_vieja_abs.to_string_lossy().to_string()],
        )
        .unwrap();
    }
    // La KB "vieja" se borra DESPUÉS de grabar meta.kb_root: exactamente el
    // caso "KB movida/renombrada" que ni el guard de escritura ni este aviso
    // deben tratar como conflicto.
    kb_vieja.close().unwrap();

    let out = Command::new(bin())
        .args(["recall", "--kb"])
        .arg(kb_pedida.path())
        .arg("--db")
        .arg(&db)
        .output()
        .unwrap();
    assert!(out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !tiene_aviso(&err),
        "KB previa borrada, no debía avisar: {err}"
    );
}

#[test]
fn recall_no_avisa_si_la_db_no_tiene_kb_root() {
    let kb_pedida = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let db = db_con_kb_root(dir.path(), None);

    let out = Command::new(bin())
        .args(["recall", "--kb"])
        .arg(kb_pedida.path())
        .arg("--db")
        .arg(&db)
        .output()
        .unwrap();
    assert!(out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !tiene_aviso(&err),
        "sin meta.kb_root, no hay con qué comparar: {err}"
    );
}

// ---------------------------------------------------------------------
// search
// ---------------------------------------------------------------------

/// `search` no tiene `--kb`: la KB esperada sale SOLO de `[kb] path` en la
/// config, así que estos tests fijan `EXO_CONFIG` en el propio `Command`
/// (nunca en el proceso del test — cada `Command` es un binario nuevo, no
/// hace falta el candado de `tests/common/mod.rs`, solo su `render_config`
/// para no repetir un TOML con `[embeddings]` válido a mano). El `db` del
/// TOML no se usa: cada test pasa su propio `--db` explícito.
fn config_con_kb(dir: &Path, kb: &Path) -> std::path::PathBuf {
    let cfg = dir.join("config.toml");
    std::fs::write(
        &cfg,
        common::render_config(kb, "kb-test", &dir.join("no-se-usa.db")),
    )
    .unwrap();
    cfg
}

#[test]
fn search_avisa_si_la_db_es_de_otra_kb_que_sigue_en_disco() {
    let kb_vieja = tempfile::tempdir().unwrap();
    let kb_pedida = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let db = db_con_kb_root(dir.path(), Some(kb_vieja.path()));
    let cfg = config_con_kb(dir.path(), kb_pedida.path());

    let out = Command::new(bin())
        .args(["search", "--db"])
        .arg(&db)
        .arg("--json")
        .arg("nada-que-encontrar")
        .env("EXO_CONFIG", &cfg)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "exit code intacto: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let err = String::from_utf8_lossy(&out.stderr);
    let kb_vieja_abs = std::fs::canonicalize(kb_vieja.path()).unwrap();
    let kb_pedida_abs = std::fs::canonicalize(kb_pedida.path()).unwrap();
    assert!(
        err.lines().any(|l| {
            l.starts_with("aviso: ")
                && l.contains(&kb_vieja_abs.display().to_string())
                && l.contains(&kb_pedida_abs.display().to_string())
        }),
        "esperaba un aviso nombrando las dos rutas: {err}"
    );

    // El envelope de `search` no gana claves nuevas: el aviso es SOLO
    // stderr. `warnings` puede existir por la cobertura del arm vector, pero
    // nunca debe traer el texto de este aviso ni una clave de kb_root.
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        v["data"].get("kb_root").is_none(),
        "no se añaden claves nuevas al envelope de search: {v}"
    );
    let en_warnings = v["data"]["warnings"].as_array().is_some_and(|ws| {
        ws.iter()
            .any(|w| w.as_str().is_some_and(|s| s.contains("otra KB")))
    });
    assert!(
        !en_warnings,
        "el aviso de kb_root no debe colarse en el envelope de search: {v}"
    );
}

#[test]
fn search_no_avisa_si_la_kb_de_config_es_la_misma_del_indice() {
    let kb = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let db = db_con_kb_root(dir.path(), Some(kb.path()));
    let cfg = config_con_kb(dir.path(), kb.path());

    let out = Command::new(bin())
        .args(["search", "--db"])
        .arg(&db)
        .arg("nada-que-encontrar")
        .env("EXO_CONFIG", &cfg)
        .output()
        .unwrap();
    assert!(out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!tiene_aviso(&err), "misma KB, no debía avisar: {err}");
}

#[test]
fn search_no_avisa_si_la_kb_previa_ya_no_existe_en_disco() {
    let kb_vieja = tempfile::tempdir().unwrap();
    let kb_vieja_abs = std::fs::canonicalize(kb_vieja.path()).unwrap();
    let kb_pedida = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();

    let db = dir.path().join("index.db");
    {
        let conn = exo::abre_db(&db).unwrap();
        exo::schema::crea_schema(&conn).unwrap();
        conn.execute(
            "INSERT INTO meta (clave, valor) VALUES ('kb_root', ?1)",
            [kb_vieja_abs.to_string_lossy().to_string()],
        )
        .unwrap();
    }
    kb_vieja.close().unwrap();
    let cfg = config_con_kb(dir.path(), kb_pedida.path());

    let out = Command::new(bin())
        .args(["search", "--db"])
        .arg(&db)
        .arg("nada-que-encontrar")
        .env("EXO_CONFIG", &cfg)
        .output()
        .unwrap();
    assert!(out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !tiene_aviso(&err),
        "KB previa borrada, no debía avisar: {err}"
    );
}

#[test]
fn search_sin_kb_resoluble_no_avisa_y_no_falla() {
    let kb_vieja = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let db = db_con_kb_root(dir.path(), Some(kb_vieja.path()));
    // `EXO_CONFIG` apunta a un fichero que no existe: `search` no tiene
    // `--kb`, así que sin config resoluble no hay KB esperada con la que
    // comparar — ni aviso ni fallo (contrato del brief, punto 3).
    let cfg_inexistente = dir.path().join("no-existe.toml");

    let out = Command::new(bin())
        .args(["search", "--db"])
        .arg(&db)
        .arg("nada-que-encontrar")
        .env("EXO_CONFIG", &cfg_inexistente)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "sin KB resoluble no debía fallar: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !tiene_aviso(&err),
        "sin KB resoluble no debía avisar: {err}"
    );
}
