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

/// Entorno con config válida en el tempdir, apuntando a `kb/` e `index.db`.
fn entorno_con_config(dir: &Path) -> Entorno {
    let home = dir.join("home");
    let kb = dir.join("kb");
    let db = dir.join("index.db");
    fs::create_dir_all(&home).unwrap();
    let cfg = home.join("config.toml");
    fs::write(
        &cfg,
        format!(
            "schema_version = 1\n\n[kb]\npath = \"{}\"\nname = \"kb-demo\"\n\n\
             [index]\ndb = \"{}\"\n\n[embeddings]\n\
             model = \"jinaai/jina-embeddings-v2-base-es\"\ndims = 768\n\
             min_similarity = 0.35\n",
            kb.display().to_string().replace('\\', "/"),
            db.display().to_string().replace('\\', "/"),
        ),
    )
    .unwrap();
    Entorno {
        config: cfg,
        cache_hf: home.join(".cache").join("huggingface").join("hub"),
        home,
        path: String::new(),
    }
}

#[test]
fn una_kb_que_no_existe_es_fail_y_el_check_dice_que_ruta_miro() {
    let dir = tempfile::tempdir().unwrap();
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "kb_readable");
    assert_eq!(c.estado, Estado::Fail);
    assert!(c.artefacto.ends_with("kb"), "artefacto: {}", c.artefacto);
}

#[test]
fn una_kb_con_notas_es_ok_y_reporta_cuantas_conto() {
    let dir = tempfile::tempdir().unwrap();
    let kb = dir.path().join("kb");
    fs::create_dir_all(kb.join("core")).unwrap();
    fs::write(kb.join("core").join("a.md"), "---\ntier: core\n---\nx\n").unwrap();
    fs::write(kb.join("core").join("b.md"), "---\ntier: core\n---\ny\n").unwrap();
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "kb_readable");
    assert_eq!(c.estado, Estado::Ok);
    assert!(c.detalle.contains('2'), "cuenta las notas: {}", c.detalle);
}

#[test]
fn sin_db_el_check_avisa_pero_no_gatea() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "index_db");
    assert_eq!(
        c.estado,
        Estado::Warn,
        "una DB ausente se arregla con `exo index`; no es una máquina rota"
    );
    assert!(
        c.detalle.contains("exo index"),
        "dice el remedio: {}",
        c.detalle
    );
}

#[test]
fn con_db_vacia_el_check_reporta_los_bytes_del_fichero_que_miro() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    let db = dir.path().join("index.db");
    // Una DB de verdad, con su schema: el check tiene que leerla, no
    // conformarse con que exista un fichero con ese nombre.
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    drop(conn);
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "index_db");
    assert_eq!(
        c.estado,
        Estado::Warn,
        "0 notas indexadas sigue siendo deuda"
    );
    assert!(
        c.artefacto.contains("bytes"),
        "reporta el tamaño del fichero que miró: {}",
        c.artefacto
    );
}

/// «DB no rancia» del primer bullet de G5. El caso que muerde de verdad está
/// medido: en W11 `setsid` no existía, `exo-index.sh` fallaba, el `|| true` se
/// lo tragaba y hubo **meses de sesiones sin refrescar el índice, sin un solo
/// rastro**. Esta fila es ese rastro.
#[test]
fn una_kb_mas_nueva_que_el_indice_sale_como_rancia() {
    let dir = tempfile::tempdir().unwrap();
    let kb = dir.path().join("kb");
    fs::create_dir_all(kb.join("core")).unwrap();
    fs::write(kb.join("core").join("a.md"), "---\ntier: core\n---\nx\n").unwrap();
    let db = dir.path().join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    drop(conn);
    // La nota se marca en el futuro en vez de dormir: con granularidad de
    // mtime de un segundo, un test que escribe seguido compara iguales y sale
    // verde por accidente.
    let f = fs::File::options()
        .write(true)
        .open(kb.join("core").join("a.md"))
        .unwrap();
    f.set_modified(std::time::SystemTime::now() + std::time::Duration::from_secs(3600))
        .unwrap();
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "index_db");
    assert_eq!(c.estado, Estado::Warn);
    assert!(
        c.detalle.contains("más nuevos"),
        "dice que la KB va por delante del índice: {}",
        c.detalle
    );
}

#[test]
fn sin_modelo_en_cache_avisa_con_el_tamano_de_la_descarga_que_viene() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "embeddings_model");
    assert_eq!(
        c.estado,
        Estado::Warn,
        "el modelo se baja solo en la primera indexación; no es una máquina rota"
    );
    assert!(
        c.artefacto
            .contains("models--jinaai--jina-embeddings-v2-base-es"),
        "reporta el directorio de caché que miró: {}",
        c.artefacto
    );
}

#[test]
fn sin_jq_es_fail_porque_los_hooks_del_plugin_lo_exigen() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "jq");
    assert_eq!(c.estado, Estado::Fail);
    assert!(
        c.detalle.contains("recall-inject"),
        "dice QUÉ se rompe sin jq: {}",
        c.detalle
    );
}

#[test]
fn con_jq_en_el_path_el_check_reporta_la_ruta_resuelta() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    let nombre = if cfg!(windows) { "jq.exe" } else { "jq" };
    let bindir = binario_falso(&dir.path().join("bin"), nombre);
    let mut env = entorno_con_config(dir.path());
    env.path = bindir.display().to_string();
    let informe = analiza(&env);
    let c = check(&informe, "jq");
    assert_ne!(
        c.estado,
        Estado::Fail,
        "está presente; que no se pueda ejecutar el falso es warn, no fail"
    );
    assert_eq!(c.artefacto, bindir.join(nombre).display().to_string());
}

#[test]
fn un_jq_de_windowsapps_es_fail_porque_es_el_alias_de_la_store() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    let nombre = if cfg!(windows) { "jq.exe" } else { "jq" };
    let bindir = binario_falso(
        &dir.path()
            .join("AppData")
            .join("Local")
            .join("Microsoft")
            .join("WindowsApps"),
        nombre,
    );
    let mut env = entorno_con_config(dir.path());
    env.path = bindir.display().to_string();
    let informe = analiza(&env);
    let c = check(&informe, "jq");
    assert_eq!(c.estado, Estado::Fail);
    assert!(
        c.detalle.contains("WindowsApps"),
        "nombra la trampa: {}",
        c.detalle
    );
}

#[test]
fn git_bash_sale_na_fuera_de_windows_y_no_desaparece_del_informe() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "git_bash");
    if cfg!(windows) {
        assert_eq!(c.estado, Estado::Fail, "sin bash en el PATH de un Windows");
    } else {
        assert_eq!(
            c.estado,
            Estado::Na,
            "no aplica, pero SALE: una fila ausente no se distingue de un \
             check que nunca existió"
        );
        assert!(!c.artefacto.is_empty(), "hasta el `na` dice qué miró");
    }
}

#[test]
fn sin_via_de_detach_el_check_nombra_el_evento_que_deja_el_hook() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "detach");
    assert_ne!(c.estado, Estado::Ok, "PATH vacío: no hay setsid ni cmd");
    assert!(
        c.detalle.contains("no-detach"),
        "nombra el evento que deja exo-index.sh: {}",
        c.detalle
    );
}

#[test]
fn con_el_onnx_en_cache_reporta_la_ruta_y_los_bytes_del_fichero() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    let snap = dir
        .path()
        .join("home")
        .join(".cache")
        .join("huggingface")
        .join("hub")
        .join("models--jinaai--jina-embeddings-v2-base-es")
        .join("snapshots")
        .join("8e2d780d8fd38f81ca9123ee28e4c5a968aaf21e")
        .join("onnx");
    fs::create_dir_all(&snap).unwrap();
    fs::write(snap.join("model.onnx"), vec![0u8; 4096]).unwrap();
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "embeddings_model");
    assert_eq!(c.estado, Estado::Ok);
    assert!(
        c.artefacto.contains("model.onnx"),
        "reporta el fichero, no el directorio: {}",
        c.artefacto
    );
    assert!(
        c.detalle.contains("4096"),
        "reporta los bytes que midió: {}",
        c.detalle
    );
}

#[test]
fn una_kb_sin_hook_instalado_avisa_con_el_comando_para_instalarlo() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb").join(".git").join("hooks")).unwrap();
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "kb_precommit_hook");
    assert_eq!(c.estado, Estado::Warn, "no instalado es deuda, no rotura");
    assert!(
        c.detalle.contains("ln -sf"),
        "dice cómo instalarlo: {}",
        c.detalle
    );
}

#[test]
fn con_el_hook_instalado_el_check_es_ok_y_reporta_la_ruta() {
    let dir = tempfile::tempdir().unwrap();
    let hooks = dir.path().join("kb").join(".git").join("hooks");
    fs::create_dir_all(&hooks).unwrap();
    fs::write(hooks.join("pre-commit"), b"#!/usr/bin/env bash\nexit 0\n").unwrap();
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "kb_precommit_hook");
    assert_eq!(c.estado, Estado::Ok);
    assert!(
        c.artefacto.contains("pre-commit"),
        "artefacto: {}",
        c.artefacto
    );
}

/// El fallo de V6 convertido en check permanente: el shim existe, git lo
/// ejecuta, y apunta a un script que no está. Solo se prueba en unix porque
/// el `ln -sf` de Git Bash **copia** el fichero por defecto (winsymlinks), y
/// crear un symlink real en Windows exige privilegios: el caso colgante no se
/// puede fabricar ahí sin mentir sobre lo que se está midiendo.
#[cfg(unix)]
#[test]
fn un_shim_que_apunta_a_un_script_inexistente_es_fail() {
    let dir = tempfile::tempdir().unwrap();
    let hooks = dir.path().join("kb").join(".git").join("hooks");
    fs::create_dir_all(&hooks).unwrap();
    std::os::unix::fs::symlink(dir.path().join("no-existe.sh"), hooks.join("pre-commit")).unwrap();
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "kb_precommit_hook");
    assert_eq!(c.estado, Estado::Fail);
    assert!(
        c.artefacto.contains("no-existe.sh"),
        "reporta a DÓNDE apunta el shim roto: {}",
        c.artefacto
    );
}
