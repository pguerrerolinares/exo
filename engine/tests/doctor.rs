//! Tests unitarios de los checks de `doctor` contra un `Entorno` fabricado.
//!
//! Van aquí y no en `doctor_cli.rs` porque un test NO puede mover el HOME ni
//! el PATH del proceso: sin `Entorno` inyectable, estos checks solo se
//! podrían probar en la máquina del que los escribió.
use std::fs;
use std::path::{Path, PathBuf};

use exo::doctor::{Check, Entorno, Estado, InformeDoctor, analiza};

/// Entorno sin nada: home vacío, PATH vacío, config inexistente.
fn entorno(dir: &Path) -> Entorno {
    let home = dir.join("home");
    Entorno {
        config: home.join(".exo").join("config.toml"),
        cache_hf: home.join(".cache").join("huggingface").join("hub"),
        home,
        path: String::new(),
    }
}

fn check<'a>(informe: &'a InformeDoctor, id: &str) -> &'a Check {
    informe
        .checks
        .iter()
        .find(|c| c.id == id)
        .unwrap_or_else(|| panic!("el informe tiene que llevar el check {id}"))
}

/// Crea un "binario" ejecutable y devuelve el directorio que lo contiene.
fn binario_falso(dir: &Path, nombre: &str) -> PathBuf {
    fs::create_dir_all(dir).unwrap();
    let ruta = dir.join(nombre);
    fs::write(&ruta, b"#!/bin/sh\nexit 0\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&ruta, fs::Permissions::from_mode(0o755)).unwrap();
    }
    dir.to_path_buf()
}

#[test]
fn sin_exo_en_el_path_el_check_lo_dice_y_reporta_el_path_que_miro() {
    let dir = tempfile::tempdir().unwrap();
    let informe = analiza(&entorno(dir.path()));
    let c = check(&informe, "binary_on_path");
    assert_eq!(
        c.estado,
        Estado::Warn,
        "sin PATH no es fatal: los hooks tienen fallback"
    );
    assert!(
        c.artefacto.contains("PATH="),
        "reporta el PATH que miró: {}",
        c.artefacto
    );
}

#[test]
fn con_exo_en_el_path_el_check_reporta_la_ruta_resuelta() {
    let dir = tempfile::tempdir().unwrap();
    let nombre = if cfg!(windows) { "exo.exe" } else { "exo" };
    let bindir = binario_falso(&dir.path().join("bin"), nombre);
    let mut env = entorno(dir.path());
    env.path = bindir.display().to_string();
    let informe = analiza(&env);
    let c = check(&informe, "binary_on_path");
    assert_eq!(c.estado, Estado::Ok);
    assert_eq!(c.artefacto, bindir.join(nombre).display().to_string());
}

#[test]
fn sin_binario_en_local_bin_el_gate_de_la_kb_queda_apagado_y_eso_es_fail() {
    let dir = tempfile::tempdir().unwrap();
    let informe = analiza(&entorno(dir.path()));
    let c = check(&informe, "hook_fallback_binary");
    assert_eq!(
        c.estado,
        Estado::Fail,
        "kb-precommit.sh sale 0 —commit permitido— si este fichero no está"
    );
    assert!(
        c.artefacto.contains(".local"),
        "reporta la ruta literal que mira el hook: {}",
        c.artefacto
    );
    assert!(
        c.detalle.contains("kb-precommit"),
        "el detalle dice QUÉ se apaga, no solo que falta un fichero: {}",
        c.detalle
    );
}

#[test]
fn con_el_binario_en_local_bin_el_check_reporta_el_fichero_real() {
    let dir = tempfile::tempdir().unwrap();
    let nombre = if cfg!(windows) { "exo.exe" } else { "exo" };
    binario_falso(&dir.path().join("home").join(".local").join("bin"), nombre);
    let informe = analiza(&entorno(dir.path()));
    let c = check(&informe, "hook_fallback_binary");
    assert_eq!(c.estado, Estado::Ok);
    assert!(
        c.artefacto.contains(nombre),
        "reporta QUÉ fichero existe, con extensión o sin ella: {}",
        c.artefacto
    );
}
