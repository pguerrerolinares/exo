//! `exo config --json`: la superficie mínima que le da a un script en shell
//! los valores de una config TOML, que jq no sabe leer. Existe para que
//! ningún consumidor tenga que volver a hardcodear el nombre de la KB.

use std::io::Write;
use std::process::Command;

fn bin() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("target");
    p.push(if cfg!(debug_assertions) { "debug" } else { "release" });
    p.push(if cfg!(windows) { "exo.exe" } else { "exo" });
    p
}

#[test]
fn config_json_emite_la_config_con_rutas_expandidas() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ruta = dir.path().join("config.toml");
    let mut f = std::fs::File::create(&ruta).expect("crear");
    write!(
        f,
        r#"schema_version = 1
[kb]
path = "C:/kb/demo"
name = "demo"
[index]
db = "~/.exo/index.db"
[embeddings]
model = "m/x"
dims = 768
min_similarity = 0.35
"#
    )
    .expect("escribir");

    let out = Command::new(bin())
        .args(["config", "--json"])
        .env("EXO_CONFIG", &ruta)
        .output()
        .expect("correr");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));

    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("stdout debe ser envelope");
    assert_eq!(v["command"], "config");
    assert_eq!(v["data"]["kb"]["name"], "demo");
    assert_eq!(v["data"]["embeddings"]["dims"], 768);

    // La ruta de la DB sale EXPANDIDA: un `~` sin resolver en manos de un
    // script de shell es un fichero que no existe, y el fallo sería silencioso.
    let db = v["data"]["index"]["db"].as_str().expect("db");
    assert!(!db.starts_with('~'), "la ruta llegó sin expandir: {db}");
    assert!(db.contains(".exo"), "ruta inesperada: {db}");
}
