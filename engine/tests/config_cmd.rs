//! `exo config --json`: la superficie mínima que le da a un script en shell
//! los valores de una config TOML, que jq no sabe leer. Existe para que
//! ningún consumidor tenga que volver a hardcodear el nombre de la KB.

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
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

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

    // Y sale con separadores normales: `expande_tilde` hace `home.join(resto)`
    // con `home` en formato nativo de Windows (`\`) y `resto` con el `/` del
    // TOML — sin normalizar, esta ruta llegaría mezclada (`C:\Users\x\.exo/index.db`),
    // y un script que le hiciera `dirname` o la comparase contra otra ruta
    // construida con `/` fallaría en silencio.
    assert!(
        !db.contains('\\'),
        "la ruta llegó con separadores mezclados: {db}"
    );
    let kb = v["data"]["kb"]["path"].as_str().expect("kb.path");
    assert!(
        !kb.contains('\\'),
        "kb.path llegó con separadores mezclados: {kb}"
    );
}

const CINCO: [(&str, &str); 5] = [
    ("dir", "state_dir"),
    ("reflex_log", "reflex_log"),
    ("rules_dir", "rules_dir"),
    ("index_log", "index_log"),
    ("canary", "canary"),
];

fn esperado(r: &exo::state::Rutas, campo: &str) -> String {
    let p = match campo {
        "state_dir" => &r.state_dir,
        "reflex_log" => &r.reflex_log,
        "rules_dir" => &r.rules_dir,
        "index_log" => &r.index_log,
        _ => &r.canary,
    };
    p.display().to_string().replace('\\', "/")
}

#[test]
fn json_expone_state() {
    let dir = tempfile::tempdir().expect("tempdir");
    let estado = dir.path().join("estado");
    let ruta = dir.path().join("config.toml");
    std::fs::write(
        &ruta,
        "schema_version = 1\n[kb]\npath = \"/kb/demo\"\nname = \"demo\"\n[index]\ndb = \"/i.db\"\n[embeddings]\nmodel = \"m/x\"\ndims = 768\nmin_similarity = 0.35\n",
    )
    .expect("escribir");
    let out = Command::new(bin())
        .args(["config", "--json"])
        .env("EXO_CONFIG", &ruta)
        .env("EXO_STATE_DIR", &estado)
        .env_remove("REFLEX_LOG_FILE")
        .env_remove("EXO_INDEX_LOG")
        .env_remove("REFLEX_CANARY_FILE")
        .output()
        .expect("correr");
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("envelope");
    let esp = exo::state::Rutas::resuelve(std::path::Path::new("/h"), |k| {
        (k == "EXO_STATE_DIR").then(|| estado.clone().into_os_string())
    });
    for (clave, campo) in CINCO {
        assert_eq!(v["data"]["state"][clave], esperado(&esp, campo), "{clave}");
    }
    assert_eq!(v["data"]["kb"]["name"], "demo");
    assert!(v["data"]["index"]["db"].is_string());
    assert_eq!(v["data"]["embeddings"]["dims"], 768);
}

#[test]
fn state_sin_kb() {
    let dir = tempfile::tempdir().expect("tempdir");
    let estado = dir.path().join("estado");
    let out = Command::new(bin())
        .args(["config", "--json"])
        .env("EXO_CONFIG", dir.path().join("no-existe.toml"))
        .env("EXO_STATE_DIR", &estado)
        .env_remove("REFLEX_LOG_FILE")
        .env_remove("EXO_INDEX_LOG")
        .env_remove("REFLEX_CANARY_FILE")
        .output()
        .expect("correr");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout sin envelope ({e}): {}",
            String::from_utf8_lossy(&out.stdout)
        )
    });
    let esp = exo::state::Rutas::resuelve(std::path::Path::new("/h"), |k| {
        (k == "EXO_STATE_DIR").then(|| estado.clone().into_os_string())
    });
    for (clave, campo) in CINCO {
        assert_eq!(v["data"]["state"][clave], esperado(&esp, campo), "{clave}");
    }
    assert!(v["data"]["kb"].is_null());
}
