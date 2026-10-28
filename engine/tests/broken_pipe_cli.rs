//! Regresión de #210: con la salida cerrada por el lector (`exo … | head`),
//! el binario muere en silencio como `cat`, sin pánico en stderr.
//!
//! No depende de la KB ni del modelo: `exo config` solo imprime la config, y
//! el lector de la tubería se cierra ANTES de lanzar el proceso, así que el
//! primer `println!` recibe EPIPE de forma determinista (sin carrera con `head`).

#![cfg(unix)]

use std::process::{Command, Stdio};

#[test]
fn salida_cerrada_no_entra_en_panico() {
    let (lector, escritor) = std::io::pipe().unwrap();
    drop(lector);
    let salida = Command::new(env!("CARGO_BIN_EXE_exo"))
        .arg("config")
        .stdout(Stdio::from(escritor))
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&salida.stderr);
    assert!(!stderr.contains("panicked"), "pánico en stderr: {stderr}");
    assert_ne!(salida.status.code(), Some(101), "salió por pánico");
}
