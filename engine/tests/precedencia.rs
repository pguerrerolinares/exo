//! Precedencia `flag > env > config`, comprobada por el binario real. Se
//! prueba por CLI y no por unidad porque la precedencia VIVE en el CLI: un
//! test de unidad sobre `resuelve_db` no demostraría que el flag gana.

use std::io::Write;
use std::process::Command;

fn bin() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("target");
    p.push(if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    });
    p.push(if cfg!(windows) { "exo.exe" } else { "exo" });
    p
}

fn cfg_temporal(dir: &std::path::Path, db: &str) -> std::path::PathBuf {
    let ruta = dir.join("config.toml");
    let mut f = std::fs::File::create(&ruta).expect("crear");
    write!(
        f,
        r#"schema_version = 1
[kb]
path = "{}"
name = "fixture"
[index]
db = "{db}"
[embeddings]
model = "m"
dims = 768
min_similarity = 0.35
"#,
        dir.display().to_string().replace('\\', "/")
    )
    .expect("escribir");
    ruta
}

#[test]
fn el_flag_gana_a_la_env_y_la_env_gana_a_la_config() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = cfg_temporal(dir.path(), "C:/db-de-config.db");

    // Sin flag ni env: sale la de config. Se comprueba por el MENSAJE DE
    // ERROR, que nombra la ruta que intentó abrir — evidencia del artefacto,
    // no del exit code.
    let out = Command::new(bin())
        .args(["search", "--json", "loquesea"])
        .env("EXO_CONFIG", &cfg)
        .env_remove("EXO_DB")
        .output()
        .expect("correr");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("db-de-config"), "no cayó a la config: {err}");

    // Con env: gana la env.
    let out = Command::new(bin())
        .args(["search", "--json", "loquesea"])
        .env("EXO_CONFIG", &cfg)
        .env("EXO_DB", "C:/db-de-env.db")
        .output()
        .expect("correr");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("db-de-env"),
        "la env no ganó a la config: {err}"
    );

    // Con flag: gana el flag, aunque la env esté puesta.
    let out = Command::new(bin())
        .args(["search", "--db", "C:/db-de-flag.db", "--json", "loquesea"])
        .env("EXO_CONFIG", &cfg)
        .env("EXO_DB", "C:/db-de-env.db")
        .output()
        .expect("correr");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("db-de-flag"),
        "el flag no ganó a la env: {err}"
    );
}

/// Config con `kb`/`db` explícitos (a diferencia de `cfg_temporal`, que fija
/// la KB al propio `dir`). `db` debe ser una ruta VÁLIDA y escribible dentro
/// de un tempdir: `index` la abre (y la crea) antes de canonicalizar la KB,
/// así que una DB rota haría fallar el comando por el motivo equivocado y el
/// test aseveraría sobre un error que no es el de la cascada de `--kb`.
fn cfg_temporal_kb(
    dir: &std::path::Path,
    kb: &std::path::Path,
    db: &std::path::Path,
) -> std::path::PathBuf {
    let ruta = dir.join("config.toml");
    let mut f = std::fs::File::create(&ruta).expect("crear");
    write!(
        f,
        r#"schema_version = 1
[kb]
path = "{}"
name = "fixture"
[index]
db = "{}"
[embeddings]
model = "m"
dims = 768
min_similarity = 0.35
"#,
        kb.display().to_string().replace('\\', "/"),
        db.display().to_string().replace('\\', "/"),
    )
    .expect("escribir");
    ruta
}

#[test]
fn el_flag_gana_a_la_env_y_la_env_gana_a_la_config_para_kb() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Válida y escribible: `index` la crea. Las tres rutas de KB de abajo,
    // en cambio, son inexistentes A PROPÓSITO — así el comando falla al
    // canonicalizar la KB (que es el escalón que se quiere comprobar) y
    // nunca llega a indexar nada de verdad.
    let db = dir.path().join("indice-de-prueba.db");
    let kb_config = dir.path().join("kb-de-config-inexistente");
    let cfg = cfg_temporal_kb(dir.path(), &kb_config, &db);

    // Sin flag ni env: sale la de config. Se comprueba por el MENSAJE DE
    // ERROR, que nombra la ruta que `index` intentó canonicalizar.
    let out = Command::new(bin())
        .args(["index", "--json"])
        .env("EXO_CONFIG", &cfg)
        .env_remove("EXO_KB")
        .output()
        .expect("correr");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("kb-de-config-inexistente"),
        "no cayó a la config: {err}"
    );

    // Con env: gana la env.
    let kb_env = dir.path().join("kb-de-env-inexistente");
    let out = Command::new(bin())
        .args(["index", "--json"])
        .env("EXO_CONFIG", &cfg)
        .env("EXO_KB", &kb_env)
        .output()
        .expect("correr");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("kb-de-env-inexistente"),
        "la env no ganó a la config: {err}"
    );

    // Con flag: gana el flag, aunque la env esté puesta.
    let kb_flag = dir.path().join("kb-de-flag-inexistente");
    let out = Command::new(bin())
        .args(["index", "--json"])
        .arg("--kb")
        .arg(&kb_flag)
        .env("EXO_CONFIG", &cfg)
        .env("EXO_KB", &kb_env)
        .output()
        .expect("correr");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("kb-de-flag-inexistente"),
        "el flag no ganó a la env: {err}"
    );
}

/// El entregable de la Task 13a (G5b): `doctor` resolvía la KB y la DB SOLO
/// desde la config (`config::carga_desde`), ignorando `$EXO_KB`/`$EXO_DB` —
/// mientras el resto de verbos sí los honraban (`resuelve_db`/`resuelve_kb`,
/// arriba en este mismo fichero). Con `$EXO_KB` puesta, `exo index`/`exo
/// search` trabajaban sobre una KB y `exo doctor` dictaminaba sobre otra:
/// un informe impecable sobre un artefacto que nadie estaba tocando. Se
/// prueba por CLI, no por unidad, por la misma razón que el resto del
/// fichero: la precedencia vive en `main.rs`, no en `doctor::analiza`.
#[test]
fn doctor_honra_kb_por_flag_env_y_config_en_ese_orden() {
    // Normaliza separadores: la KB que viene de la config pasó por TOML con
    // `/` literales (`cfg_temporal_kb`), y en Windows `PathBuf::display()`
    // no los reescribe a `\`, así que comparar el string crudo contra el de
    // `dir.path().join(...)` (que sí lleva `\` nativos) daría un falso
    // negativo que no tiene nada que ver con la precedencia que se prueba.
    fn norm(p: &std::path::Path) -> String {
        p.display().to_string().replace('\\', "/")
    }

    let dir = tempfile::tempdir().expect("tempdir");
    let kb_config = dir.path().join("kb-de-config");
    std::fs::create_dir_all(&kb_config).unwrap();
    let db = dir.path().join("indice-de-prueba.db");
    let cfg = cfg_temporal_kb(dir.path(), &kb_config, &db);

    let kb_readable = |stdout: &[u8]| -> String {
        let v: serde_json::Value = serde_json::from_slice(stdout).expect("json");
        v["data"]["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == "kb_readable")
            .expect("el informe lleva kb_readable")["artifact"]
            .as_str()
            .unwrap()
            .replace('\\', "/")
    };

    // Sin flag ni env: `kb_readable` reporta la KB de la config.
    let out = Command::new(bin())
        .args(["doctor", "--json"])
        .env("EXO_CONFIG", &cfg)
        .env_remove("EXO_KB")
        .output()
        .expect("correr");
    assert_eq!(kb_readable(&out.stdout), norm(&kb_config));

    // Con $EXO_KB: gana la env, no la config. Este es el caso que estaba
    // roto: antes del arreglo, `doctor` seguía reportando `kb_config` aquí.
    let kb_env = dir.path().join("kb-de-env");
    std::fs::create_dir_all(&kb_env).unwrap();
    let out = Command::new(bin())
        .args(["doctor", "--json"])
        .env("EXO_CONFIG", &cfg)
        .env("EXO_KB", &kb_env)
        .output()
        .expect("correr");
    assert_eq!(
        kb_readable(&out.stdout),
        norm(&kb_env),
        "doctor tiene que honrar $EXO_KB como el resto de verbos"
    );

    // Con --kb: gana el flag, aunque la env esté puesta.
    let kb_flag = dir.path().join("kb-de-flag");
    std::fs::create_dir_all(&kb_flag).unwrap();
    let out = Command::new(bin())
        .args(["doctor", "--json", "--kb"])
        .arg(&kb_flag)
        .env("EXO_CONFIG", &cfg)
        .env("EXO_KB", &kb_env)
        .output()
        .expect("correr");
    assert_eq!(kb_readable(&out.stdout), norm(&kb_flag));
}

/// Mismo contrato que el test de arriba, para `index_db`.
#[test]
fn doctor_honra_db_por_flag_env_y_config_en_ese_orden() {
    let dir = tempfile::tempdir().expect("tempdir");
    let kb = dir.path().join("kb-de-config");
    std::fs::create_dir_all(&kb).unwrap();
    let db_config = dir.path().join("db-de-config.sqlite");
    let cfg = cfg_temporal_kb(dir.path(), &kb, &db_config);

    let index_db_artefacto = |stdout: &[u8]| -> String {
        let v: serde_json::Value = serde_json::from_slice(stdout).expect("json");
        v["data"]["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == "index_db")
            .expect("el informe lleva index_db")["artifact"]
            .as_str()
            .unwrap()
            .to_string()
    };

    // Con $EXO_DB: gana la env, no la config.
    let db_env = dir.path().join("db-de-env.sqlite");
    let out = Command::new(bin())
        .args(["doctor", "--json"])
        .env("EXO_CONFIG", &cfg)
        .env("EXO_DB", &db_env)
        .output()
        .expect("correr");
    assert!(
        index_db_artefacto(&out.stdout).starts_with(&db_env.display().to_string()),
        "doctor tiene que honrar $EXO_DB como el resto de verbos: {}",
        index_db_artefacto(&out.stdout)
    );

    // Con --db: gana el flag, aunque la env esté puesta.
    let db_flag = dir.path().join("db-de-flag.sqlite");
    let out = Command::new(bin())
        .args(["doctor", "--json", "--db"])
        .arg(&db_flag)
        .env("EXO_CONFIG", &cfg)
        .env("EXO_DB", &db_env)
        .output()
        .expect("correr");
    assert!(index_db_artefacto(&out.stdout).starts_with(&db_flag.display().to_string()));
}

#[test]
fn sin_config_ni_flag_el_error_dice_que_hacer() {
    let dir = tempfile::tempdir().expect("tempdir");
    let inexistente = dir.path().join("no-hay.toml");
    let out = Command::new(bin())
        .args(["search", "--json", "loquesea"])
        .env("EXO_CONFIG", &inexistente)
        .env_remove("EXO_DB")
        .output()
        .expect("correr");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("exo init"),
        "error sin salida accionable: {err}"
    );
}
