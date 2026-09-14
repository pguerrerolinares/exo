//! `exo rotate` contra el binario real.
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_exo")
}

fn kb_con_bitacora(cuerpo: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("log")).unwrap();
    std::fs::write(dir.path().join("log/p-bitacora.md"), cuerpo).unwrap();
    dir
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
    let salida = Command::new(bin())
        .args(["rotate", "--json", "--hot-bytes", "8000", "--apply"])
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(
        salida.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
    assert!(dir.path().join("archive/log").exists());
}

#[test]
fn ignora_notas_que_no_son_tier_log() {
    let dir = kb_con_bitacora("---\ntier: stable\n---\n\n# no rotar\n\n## 2026-01-01 -- a\nx\n");
    let salida = Command::new(bin())
        .args(["rotate", "--hot-bytes", "1"])
        .arg("--kb")
        .arg(dir.path())
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
        .output()
        .unwrap();
    assert!(!salida.status.success());
    assert!(salida.stdout.is_empty());
}

#[test]
fn sin_directorio_log_no_hay_nada_que_rotar() {
    let dir = tempfile::tempdir().unwrap();
    let salida = Command::new(bin())
        .args(["rotate", "--hot-bytes", "100"])
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(salida.status.success());
    assert_eq!(
        String::from_utf8_lossy(&salida.stdout).trim(),
        "rotate: nothing to rotate"
    );
}
