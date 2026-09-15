//! `exo rotate` contra el binario real.
//!
//! `rotate_cmd` lee `[kb] name` vía `exo::nombre_kb()` (`config::carga()` →
//! `EXO_CONFIG` o `~/.exo/config.toml` si no está definida). Sin fijar
//! `EXO_CONFIG` por test, estos tests leían la config REAL de la máquina que
//! los corre — no herméticos, mismo defecto que `tests/kb_root_lectura_cli.rs`
//! ya documenta y arregla para `search`. Cada `Command` es un binario nuevo,
//! así que `EXO_CONFIG` se fija en el propio `Command` (`.env(...)`), no en
//! el proceso del test — no hace falta el candado de `tests/common/mod.rs`.
use std::process::Command;

mod common;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_exo")
}

fn kb_con_bitacora(cuerpo: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("log")).unwrap();
    std::fs::write(dir.path().join("log/p-bitacora.md"), cuerpo).unwrap();
    dir
}

/// Ruta de config que NUNCA existe en disco, dentro del propio tempdir del
/// test — `exo::nombre_kb()` falla con ella igual que sin `[kb] name`
/// definido. El dry-run (`--apply` ausente) no necesita config y sigue
/// funcionando con esta ruta; `--apply` con ella es exactamente el montaje de
/// `apply_sin_kb_name_falla_antes_de_escribir` (Decisión D-4, verdict D+E
/// 2026-09-15: sin `[kb] name` resoluble, `--apply` aborta ANTES de tocar
/// disco — ya no hay fallback al prefijo `kb`).
fn cfg_inexistente(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("no-existe-config.toml")
}

fn nota_grande() -> String {
    let mut s = String::from("---\ntitle: p-bitacora\ntier: log\n---\n\n# p -- bitacora\n\n");
    for i in 0..10 {
        s.push_str(&format!(
            "## 2026-0{}-01 -- entrada\n{}\n\n",
            1 + i % 9,
            "x".repeat(3000)
        ));
    }
    s
}

#[test]
fn dry_run_no_toca_disco_y_reporta_json() {
    let dir = kb_con_bitacora(&nota_grande());
    let salida = Command::new(bin())
        .args(["rotate", "--json", "--hot-bytes", "8000"])
        .arg("--kb")
        .arg(dir.path())
        .env("EXO_CONFIG", cfg_inexistente(dir.path()))
        .output()
        .unwrap();
    assert!(
        salida.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(v["schema_version"], 2);
    assert_eq!(v["command"], "rotate");
    assert_eq!(v["data"]["applied"], false);
    assert_eq!(v["data"]["rotations"][0]["rotated"], true);
    assert!(!dir.path().join("archive").exists());
}

#[test]
fn apply_escribe_de_verdad() {
    let dir = kb_con_bitacora(&nota_grande());
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        common::render_config(dir.path(), "prueba", &dir.path().join("x.db")),
    )
    .unwrap();
    let salida = Command::new(bin())
        .args(["rotate", "--json", "--hot-bytes", "8000", "--apply"])
        .arg("--kb")
        .arg(dir.path())
        .env("EXO_CONFIG", &config_path)
        .output()
        .unwrap();
    assert!(
        salida.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
    assert!(dir.path().join("archive/log").exists());

    let archivo = std::fs::read_dir(dir.path().join("archive/log"))
        .unwrap()
        .find_map(|e| e.ok())
        .expect("debía haber al menos un archivo rotado");
    let contenido = std::fs::read_to_string(archivo.path()).unwrap();
    assert!(
        contenido.contains("permalink: 'prueba/archive/log/"),
        "el permalink del archivo debía usar el `[kb] name` de la config: {contenido}"
    );
}

#[test]
fn ignora_notas_que_no_son_tier_log() {
    let dir = kb_con_bitacora("---\ntier: stable\n---\n\n# no rotar\n\n## 2026-01-01 -- a\nx\n");
    let salida = Command::new(bin())
        .args(["rotate", "--hot-bytes", "1"])
        .arg("--kb")
        .arg(dir.path())
        .env("EXO_CONFIG", cfg_inexistente(dir.path()))
        .output()
        .unwrap();
    assert!(salida.status.success());
    assert_eq!(
        String::from_utf8_lossy(&salida.stdout).trim(),
        "rotate: nothing to rotate"
    );
}

#[test]
fn hot_bytes_no_positivo_falla_sin_ensuciar_stdout() {
    let dir = kb_con_bitacora(&nota_grande());
    let salida = Command::new(bin())
        .args(["rotate", "--hot-bytes", "0", "--json"])
        .arg("--kb")
        .arg(dir.path())
        .env("EXO_CONFIG", cfg_inexistente(dir.path()))
        .output()
        .unwrap();
    assert!(!salida.status.success());
    assert!(salida.stdout.is_empty());
}

// Pin del finding I2 de la review: un directorio `algo.md/` dentro de
// `log/` no es una nota — kbx lo salta con `if e.IsDir() || ... { continue }`
// antes incluso de mirar la extensión. Sin ese filtro, `exo rotate` intenta
// `std::fs::read()` sobre el directorio, falla con un error de I/O y ese
// fallo cuenta como nota fallida (exit 1) en vez de saltarse en silencio.
#[test]
fn un_directorio_con_extension_md_en_log_no_cuenta_como_fallo() {
    let dir = kb_con_bitacora(&nota_grande());
    std::fs::create_dir_all(dir.path().join("log/x.md")).unwrap();
    let salida = Command::new(bin())
        .args(["rotate", "--hot-bytes", "8000"])
        .arg("--kb")
        .arg(dir.path())
        .env("EXO_CONFIG", cfg_inexistente(dir.path()))
        .output()
        .unwrap();
    assert!(
        salida.status.success(),
        "un directorio .md en log/ no debe hacer fallar la barrida; stderr: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
}

#[test]
fn sin_directorio_log_no_hay_nada_que_rotar() {
    let dir = tempfile::tempdir().unwrap();
    let salida = Command::new(bin())
        .args(["rotate", "--hot-bytes", "100"])
        .arg("--kb")
        .arg(dir.path())
        .env("EXO_CONFIG", cfg_inexistente(dir.path()))
        .output()
        .unwrap();
    assert!(salida.status.success());
    assert_eq!(
        String::from_utf8_lossy(&salida.stdout).trim(),
        "rotate: nothing to rotate"
    );
}

// Decisión D-4 (verdict D+E 2026-09-15, decisión 3): el prefijo del
// `permalink` que escribe `--apply` es `[kb] name`. Sin config válida (o sin
// `name` en ella), `--apply` no inventa un prefijo `kb` — aborta ANTES de
// tocar disco, igual que `write new` (`main.rs:794-797`). El dry-run (sin
// `--apply`) no necesita config: eso lo cubren los tests de arriba con la
// misma `cfg_inexistente`.
#[test]
fn apply_sin_kb_name_falla_antes_de_escribir() {
    let dir = kb_con_bitacora(&nota_grande());
    let original = std::fs::read(dir.path().join("log/p-bitacora.md")).unwrap();
    let salida = Command::new(bin())
        .args(["rotate", "--hot-bytes", "8000", "--apply"])
        .arg("--kb")
        .arg(dir.path())
        .env("EXO_CONFIG", cfg_inexistente(dir.path()))
        .output()
        .unwrap();

    assert!(
        !salida.status.success(),
        "esperaba que --apply fallara sin [kb] name; stdout: {}",
        String::from_utf8_lossy(&salida.stdout)
    );
    assert_eq!(
        salida.status.code(),
        Some(1),
        "un [kb] name irresoluble no es un GateFallido — exit 1, no exit 3"
    );

    let err = String::from_utf8_lossy(&salida.stderr);
    assert!(
        err.contains("[kb] name"),
        "esperaba que el stderr nombrara `[kb] name`: {err}"
    );

    assert!(
        !dir.path().join("archive").exists(),
        "un --apply fallido no debe crear archive/"
    );
    assert_eq!(
        std::fs::read(dir.path().join("log/p-bitacora.md")).unwrap(),
        original,
        "un --apply fallido no debe tocar la nota original"
    );
}
