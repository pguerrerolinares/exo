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
