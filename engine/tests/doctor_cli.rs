//! La superficie de CLI de `exo doctor`: envelope v2, informe ANTES del gate,
//! exit 3 cuando algún check sale `fail`.
//!
//! Los tests NO asertan exit 0: en un runner limpio faltan la DB y el modelo,
//! y eso es precisamente lo que doctor tiene que reportar. Lo determinista
//! —y lo que se aserta aquí— es el caso config-ausente.
use std::fs;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_exo")
}

/// Config mínima válida en un tempdir. Se apunta con `$EXO_CONFIG` para no
/// tocar el `~/.exo/config.toml` de la máquina (gate hermético).
fn config_valida(dir: &std::path::Path) -> std::path::PathBuf {
    let kb = dir.join("kb");
    fs::create_dir_all(&kb).unwrap();
    let ruta = dir.join("config.toml");
    fs::write(
        &ruta,
        format!(
            "schema_version = 1\n\n[kb]\npath = \"{}\"\nname = \"kb-demo\"\n\n\
             [index]\ndb = \"{}\"\n\n[embeddings]\n\
             model = \"jinaai/jina-embeddings-v2-base-es\"\ndims = 768\n\
             min_similarity = 0.35\n",
            kb.display().to_string().replace('\\', "/"),
            dir.join("index.db")
                .display()
                .to_string()
                .replace('\\', "/"),
        ),
    )
    .unwrap();
    ruta
}

#[test]
fn doctor_emite_envelope_v2_con_el_comando_y_la_plataforma() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = config_valida(dir.path());
    let salida = Command::new(bin())
        .args(["doctor", "--json"])
        .env("EXO_CONFIG", &cfg)
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(v["schema_version"], 2);
    assert_eq!(v["command"], "doctor");
    assert!(v["data"]["checks"].is_array());
    assert!(
        v["data"]["platform"].is_string(),
        "el informe declara la plataforma que midió"
    );
    let checks = v["data"]["checks"].as_array().unwrap();
    let config = checks.iter().find(|c| c["id"] == "config").unwrap();
    assert_eq!(config["status"], "ok");
    assert_eq!(
        config["artifact"],
        cfg.display().to_string(),
        "el check reporta el fichero que miró, no un veredicto pelado"
    );
}

#[test]
fn sin_config_el_informe_sale_igual_y_luego_gatea_con_exit_tres() {
    let dir = tempfile::tempdir().unwrap();
    let salida = Command::new(bin())
        .args(["doctor", "--json"])
        .env("EXO_CONFIG", dir.path().join("no-existe.toml"))
        .output()
        .unwrap();
    assert_eq!(
        salida.status.code(),
        Some(3),
        "config ausente es gate de dominio, no error de sistema"
    );
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(
        v["data"]["ok"], false,
        "el informe entero se emite ANTES de gatear"
    );
    let checks = v["data"]["checks"].as_array().unwrap();
    let config = checks.iter().find(|c| c["id"] == "config").unwrap();
    assert_eq!(config["status"], "fail");
}

#[test]
fn la_salida_humana_lleva_estado_id_y_artefacto_en_cada_linea() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = config_valida(dir.path());
    let salida = Command::new(bin())
        .arg("doctor")
        .env("EXO_CONFIG", &cfg)
        .output()
        .unwrap();
    let texto = String::from_utf8_lossy(&salida.stdout);
    let linea = texto
        .lines()
        .find(|l| l.contains("\tconfig\t"))
        .expect("hay una línea del check config");
    let campos: Vec<&str> = linea.split('\t').collect();
    assert_eq!(campos.len(), 4, "estado\tid\tartefacto\tdetalle");
    assert_eq!(campos[0], "ok");
    assert_eq!(campos[1], "config");
    assert_eq!(campos[2], cfg.display().to_string());
}
