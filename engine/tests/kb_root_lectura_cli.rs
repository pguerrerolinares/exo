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
//! esperada sale de `resuelve_kb(None)` (precedencia `$EXO_KB` > config).
//!
//! ## Nota de diseño (review 2026-09-13, degradación best-effort)
//!
//! El chequeo corre SIEMPRE sobre la conexión que `busca`/`busca_vector`/
//! `recall_arranque`/`recall_consulta` ya tienen abierta para su propia
//! consulta (nunca una conexión aparte) — por construcción ya no hay
//! apertura de fichero que pueda fallar por WAL/permisos/`busy_timeout`
//! específicamente para este aviso: si esa conexión no abre, la búsqueda
//! real tampoco lo haría, con o sin este feature. Por eso no hay aquí un
//! test de "WAL con -wal/-shm ausentes + directorio de solo lectura" (la
//! primera implementación SÍ abría su propia conexión de solo lectura y
//! ese caso la rompía; con el diseño actual es estructuralmente imposible
//! de reproducir vía este feature). Lo que sí sigue siendo un fallo barato
//! de montar — y que si no se blindara SÍ tumbaría el comando— es la query
//! en sí sobre una DB sin tabla `meta` (schema anterior a M6-04): cubierto
//! abajo.

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

/// DB de schema ANTERIOR a M6-04: `notas`/`notas_fts` existen (basta para
/// que `recall` en modo arranque sirva resultados) pero `meta` no — nunca
/// se creó porque nada la escribió (viene de antes de que `crea_schema`
/// existiera, o de un `CREATE TABLE` a mano como este). `SELECT valor FROM
/// meta ...` revienta con «no such table: meta», un error real de SQLite
/// que `.optional()` NO absorbe (solo absorbe "sin filas", no "sin
/// tabla"). Regla de diseño (CRITICAL del review): un fallo AQUÍ jamás
/// puede tumbar `recall` — exit 0, sin aviso, resultados intactos.
fn db_sin_tabla_meta(dir: &Path) -> std::path::PathBuf {
    let db = dir.join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    conn.execute_batch(
        "
        CREATE TABLE notas (
          permalink  TEXT PRIMARY KEY,
          ruta       TEXT NOT NULL UNIQUE,
          titulo     TEXT NOT NULL,
          tipo       TEXT,
          mtime      REAL NOT NULL,
          git_epoch  INTEGER
        );
        CREATE VIRTUAL TABLE notas_fts USING fts5(
          titulo, cuerpo,
          permalink UNINDEXED,
          tokenize='unicode61 tokenchars 0x2F'
        );
        ",
    )
    .unwrap();
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
fn recall_no_avisa_ni_falla_si_la_db_no_tiene_tabla_meta() {
    let kb_pedida = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let db = db_sin_tabla_meta(dir.path());

    let out = Command::new(bin())
        .args(["recall", "--kb"])
        .arg(kb_pedida.path())
        .arg("--db")
        .arg(&db)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "DB sin tabla meta no debía tumbar recall: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !tiene_aviso(&err),
        "sin tabla meta no hay con qué comparar: {err}"
    );
}

/// La KB PEDIDA (no la previa) no existe en disco: `canonicalize(kb)`
/// falla dentro de `aviso_kb_root_lectura`, que lo trata igual que
/// cualquier otro fallo interno — `None`, sin aviso. La DB SÍ tiene un
/// `kb_root` que sería un conflicto real contra cualquier KB existente,
/// para que quede claro que el "no aviso" es por el fallo de
/// `canonicalize`, no por falta de conflicto.
#[test]
fn recall_no_avisa_si_la_kb_pedida_no_existe_en_disco() {
    let kb_vieja = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let db = db_con_kb_root(dir.path(), Some(kb_vieja.path()));
    let kb_pedida_inexistente = dir.path().join("no-existe-en-absoluto");

    let out = Command::new(bin())
        .args(["recall", "--kb"])
        .arg(&kb_pedida_inexistente)
        .arg("--db")
        .arg(&db)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "KB pedida inexistente no debía tumbar recall: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !tiene_aviso(&err),
        "KB pedida sin canonicalizar, no hay con qué comparar: {err}"
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

/// Misma degradación que `recall_no_avisa_ni_falla_si_la_db_no_tiene_tabla_meta`,
/// para `search` (camino independiente: `busca`/`busca_vector` hacen su
/// propia llamada a `aviso_kb_root_lectura` sobre su propia conexión).
/// `--type fts` explícito (D6, Ola 1 G Task 11): la fixture no crea
/// `vectores` en absoluto (`db_sin_tabla_meta`, ni siquiera vacía), así que
/// el arm vector del nuevo default `hybrid` fallaría con "no such table:
/// vectores" — un error real, distinto de la degradación "0 vectores" que sí
/// tolera `aviso_kb_root_lectura`. Este test prueba el camino FTS puro
/// contra una DB con schema mínimo, no el default de `exo search`.
#[test]
fn search_no_avisa_ni_falla_si_la_db_no_tiene_tabla_meta() {
    let kb_pedida = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let db = db_sin_tabla_meta(dir.path());
    let cfg = config_con_kb(dir.path(), kb_pedida.path());

    let out = Command::new(bin())
        .args(["search", "--type", "fts", "--db"])
        .arg(&db)
        .arg("buscable")
        .env("EXO_CONFIG", &cfg)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "DB sin tabla meta no debía tumbar search: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !tiene_aviso(&err),
        "sin tabla meta no hay con qué comparar: {err}"
    );
    // Resultados intactos: el FTS de siempre, sin degradar.
    let salida = String::from_utf8_lossy(&out.stdout);
    assert!(
        salida.contains("kb/a"),
        "la búsqueda debía seguir encontrando la nota: {salida}"
    );
}

/// Campaña L Task 1: el default `hybrid` (D6) contra la fixture de arriba
/// (`vectores` AUSENTE, ni siquiera vacía) degrada con aviso en vez de
/// tumbar el comando — la regresión que documentaba el test de arriba antes
/// de este fix. `avisos_cobertura_vector`/`tabla_existe`
/// (`engine/src/buscador.rs`) tratan "tabla ausente" como su propio caso,
/// comprobado ANTES que "0 filas": siempre avisa "arm vector INERTE",
/// nunca un `Err` duro.
#[test]
fn search_default_hybrid_degrada_con_aviso_si_falta_tabla_vectores() {
    let kb_pedida = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let db = db_sin_tabla_meta(dir.path());
    let cfg = config_con_kb(dir.path(), kb_pedida.path());

    let out = Command::new(bin())
        .args(["search", "--db"])
        .arg(&db)
        .arg("--json")
        .arg("buscable")
        .env("EXO_CONFIG", &cfg)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "DB sin tabla vectores no debía tumbar el default hybrid: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        tiene_aviso(&err),
        "el default hybrid degradado debía avisar por stderr: {err}"
    );

    let salida: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(salida["data"]["search_type"], "hybrid");
    let warnings = salida["data"]["warnings"]
        .as_array()
        .expect("data.warnings debe existir en el envelope degradado");
    assert!(
        warnings
            .iter()
            .any(|w| w.as_str().unwrap_or("").contains("INERTE")),
        "el envelope debía llevar el aviso de arm INERTE en data.warnings: {salida}"
    );
    let permalinks: Vec<&str> = salida["data"]["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["permalink"].as_str().unwrap())
        .collect();
    assert!(
        permalinks.contains(&"kb/a"),
        "el canal FTS debía seguir encontrando la nota: {salida}"
    );
}

/// La KB de la CONFIG (la que `search` resolvería sin `$EXO_KB`) no existe
/// en disco: mismo criterio que el análogo de `recall` — `canonicalize`
/// falla dentro de `aviso_kb_root_lectura`, `None`, sin aviso.
#[test]
fn search_no_avisa_si_la_kb_de_config_no_existe_en_disco() {
    let kb_vieja = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let db = db_con_kb_root(dir.path(), Some(kb_vieja.path()));
    let kb_config_inexistente = dir.path().join("no-existe-en-absoluto");
    let cfg = config_con_kb(dir.path(), &kb_config_inexistente);

    let out = Command::new(bin())
        .args(["search", "--db"])
        .arg(&db)
        .arg("nada-que-encontrar")
        .env("EXO_CONFIG", &cfg)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "KB de config inexistente no debía tumbar search: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !tiene_aviso(&err),
        "KB de config sin canonicalizar, no hay con qué comparar: {err}"
    );
}

// ---------------------------------------------------------------------
// search: precedencia $EXO_KB > config (review punto 3)
// ---------------------------------------------------------------------
//
// `busca_cmd` resuelve la KB esperada con `resuelve_kb(None)`, la MISMA
// función que usa el resto del binario: `--kb` (search no tiene) > `$EXO_KB`
// > `[kb] path` de la config. Antes de este fix, `busca_cmd` llamaba
// directo a `exo::kb_desde_config()`, saltándose `$EXO_KB` por completo.

#[test]
fn search_exo_kb_sin_conflicto_gana_a_config_con_conflicto() {
    let kb_indexada = tempfile::tempdir().unwrap(); // == meta.kb_root, la que pide EXO_KB
    let kb_otra = tempfile::tempdir().unwrap(); // la que pondría la config, en conflicto real
    let dir = tempfile::tempdir().unwrap();
    let db = db_con_kb_root(dir.path(), Some(kb_indexada.path()));
    let cfg = config_con_kb(dir.path(), kb_otra.path());

    let out = Command::new(bin())
        .args(["search", "--db"])
        .arg(&db)
        .arg("nada-que-encontrar")
        .env("EXO_CONFIG", &cfg)
        .env("EXO_KB", kb_indexada.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !tiene_aviso(&err),
        "$EXO_KB coincide con el índice y debía ganar a la config (que sí estaría en conflicto): {err}"
    );
}

#[test]
fn search_exo_kb_con_conflicto_gana_a_config_sin_conflicto() {
    let kb_indexada = tempfile::tempdir().unwrap(); // == meta.kb_root, la que pondría la config
    let kb_otra = tempfile::tempdir().unwrap(); // la que pide EXO_KB, en conflicto real
    let dir = tempfile::tempdir().unwrap();
    let db = db_con_kb_root(dir.path(), Some(kb_indexada.path()));
    let cfg = config_con_kb(dir.path(), kb_indexada.path());

    let out = Command::new(bin())
        .args(["search", "--db"])
        .arg(&db)
        .arg("nada-que-encontrar")
        .env("EXO_CONFIG", &cfg)
        .env("EXO_KB", kb_otra.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    let kb_indexada_abs = std::fs::canonicalize(kb_indexada.path()).unwrap();
    let kb_otra_abs = std::fs::canonicalize(kb_otra.path()).unwrap();
    assert!(
        err.lines().any(|l| {
            l.starts_with("aviso: ")
                && l.contains(&kb_indexada_abs.display().to_string())
                && l.contains(&kb_otra_abs.display().to_string())
        }),
        "$EXO_KB en conflicto debía ganar a la config (que NO estaría en conflicto) y avisar: {err}"
    );
}
