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
        kb: None,
        db: None,
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
        kb: None,
        db: None,
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

/// El entregable de la Task 13a (G5b): antes de este arreglo, `check_kb`
/// leía la KB SOLO desde `cfg`, ignorando `$EXO_KB` — mientras el resto de
/// verbos (`index`, `search`...) sí lo honraban. Con `entorno.kb` puesto (lo
/// que rellena `doctor_cmd` resolviendo `flag > $EXO_KB > config`), el check
/// tiene que reportar ESA ruta, no la de la config: la config de este test
/// apunta a una KB que ni existe (`dir/kb`, nunca creada aquí), así que si el
/// bug reapareciera este test fallaría con `kb_readable` en `fail` en vez de
/// `ok`.
#[test]
fn con_kb_en_el_entorno_el_check_reporta_esa_kb_no_la_de_la_config() {
    let dir = tempfile::tempdir().unwrap();
    let kb_entorno = dir.path().join("kb-del-entorno");
    fs::create_dir_all(&kb_entorno).unwrap();
    let mut env = entorno_con_config(dir.path());
    env.kb = Some(kb_entorno.clone());
    let informe = analiza(&env);
    let c = check(&informe, "kb_readable");
    assert_eq!(
        c.estado,
        Estado::Ok,
        "la KB de la config ({}) no existe; solo sale ok si miró la del entorno",
        dir.path().join("kb").display()
    );
    assert_eq!(c.artefacto, kb_entorno.display().to_string());
}

/// Mismo contrato que el test de arriba, para `index_db`: `entorno.db` tiene
/// que ganarle a `cfg.index.db`.
#[test]
fn con_db_en_el_entorno_el_check_reporta_esa_db_no_la_de_la_config() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    let db_entorno = dir.path().join("db-del-entorno.sqlite");
    let conn = exo::abre_db(&db_entorno).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    drop(conn);
    let mut env = entorno_con_config(dir.path());
    env.db = Some(db_entorno.clone());
    let informe = analiza(&env);
    let c = check(&informe, "index_db");
    assert!(
        c.artefacto.starts_with(&db_entorno.display().to_string()),
        "reportó la DB de config, no la del entorno: {}",
        c.artefacto
    );
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
    // Un `exit 0` a secas ya no basta: el hook tiene que mencionar
    // `kb-precommit.sh` y el shim tiene que resolver a un script real, o el
    // veredicto no es un `ok` con artefacto. Esto es justo lo que esta tarea
    // vino a corregir en este mismo test.
    let dir = tempfile::tempdir().unwrap();
    shim_precommit(&dir.path().join("kb"), SHIM_DE_LA_KB);
    plugin_con_script(&dir.path().join("home"), "exo", "1.0.0");
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "kb_precommit_hook");
    assert_eq!(c.estado, Estado::Ok);
    assert!(
        c.artefacto.contains("pre-commit"),
        "artefacto: {}",
        c.artefacto
    );
}

/// Escribe un `pre-commit` que es un shim (fichero regular, no symlink) — el
/// caso real de una máquina con `core.symlinks=false`.
fn shim_precommit(kb: &Path, cuerpo: &str) {
    let hooks = kb.join(".git").join("hooks");
    fs::create_dir_all(&hooks).unwrap();
    fs::write(hooks.join("pre-commit"), cuerpo).unwrap();
}

/// Instala un `kb-precommit.sh` falso en el layout del plugin bajo `home`.
fn plugin_con_script(home: &Path, familia: &str, version: &str) -> PathBuf {
    let dir = home
        .join(".claude")
        .join("plugins")
        .join("cache")
        .join("exo")
        .join(familia)
        .join(version)
        .join("scripts");
    fs::create_dir_all(&dir).unwrap();
    let ruta = dir.join("kb-precommit.sh");
    fs::write(&ruta, b"#!/usr/bin/env bash\nexit 0\n").unwrap();
    ruta
}

const SHIM_DE_LA_KB: &str = "#!/usr/bin/env bash\nexec bash \"$script\" # kb-precommit.sh\n";

#[test]
fn un_shim_que_no_resuelve_a_ningun_script_es_fail() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    shim_precommit(&dir.path().join("kb"), SHIM_DE_LA_KB);
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "kb_precommit_hook");
    assert_eq!(
        c.estado,
        Estado::Fail,
        "el hook existe, pero el script al que llama no está en ningún sitio"
    );
    assert!(
        c.detalle.contains("no resuelve"),
        "dice que el problema es la resolución, no la ausencia: {}",
        c.detalle
    );
}

#[test]
fn un_shim_que_resuelve_al_plugin_exo_es_ok_y_reporta_el_script() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    shim_precommit(&dir.path().join("kb"), SHIM_DE_LA_KB);
    let script = plugin_con_script(&dir.path().join("home"), "exo", "1.1.1");
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "kb_precommit_hook");
    assert_eq!(c.estado, Estado::Ok);
    assert!(
        c.artefacto.contains(&script.display().to_string()),
        "reporta el script al que resuelve, no solo el hook: {}",
        c.artefacto
    );
}

#[test]
fn un_shim_que_solo_encuentra_el_plugin_viejo_avisa() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    shim_precommit(&dir.path().join("kb"), SHIM_DE_LA_KB);
    plugin_con_script(&dir.path().join("home"), "reflex", "0.17.0");
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "kb_precommit_hook");
    assert_eq!(c.estado, Estado::Warn);
    assert!(
        c.detalle.contains("reflex"),
        "nombra el plugin viejo: {}",
        c.detalle
    );
}

#[test]
fn un_pre_commit_ajeno_no_se_hace_pasar_por_el_gate_de_la_kb() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    shim_precommit(&dir.path().join("kb"), "#!/usr/bin/env bash\nnpm test\n");
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "kb_precommit_hook");
    assert_eq!(
        c.estado,
        Estado::Warn,
        "hay un pre-commit, pero no es el de la KB: decirlo es el trabajo"
    );
    assert!(
        c.detalle.contains("no es el gate de la KB"),
        "detalle: {}",
        c.detalle
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

/// La rama symlink del check del pre-commit no tenía NINGÚN test que cubriera
/// su `ok` — lo señaló el review final de rama, y es la misma clase de hueco
/// que la Task 6b arregló en la rama de al lado del mismo `if`.
///
/// `#[cfg(unix)]` por lo mismo que el test del shim colgante: el `ln -sf` de
/// Git Bash copia el fichero por defecto y crear un symlink real en Windows
/// exige privilegios.
#[cfg(unix)]
#[test]
fn un_symlink_al_kb_precommit_del_plugin_es_ok() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb").join(".git").join("hooks")).unwrap();
    let script = plugin_con_script(&dir.path().join("home"), "exo", "1.1.1");
    std::os::unix::fs::symlink(
        &script,
        dir.path()
            .join("kb")
            .join(".git")
            .join("hooks")
            .join("pre-commit"),
    )
    .unwrap();
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "kb_precommit_hook");
    assert_eq!(c.estado, Estado::Ok);
    assert!(
        c.artefacto.contains("kb-precommit.sh"),
        "reporta a dónde apunta: {}",
        c.artefacto
    );
}

/// Un symlink a OTRO script no es el gate de la KB. Antes de este arreglo
/// devolvía `ok` afirmando que el gate corre en cada commit — mientras la
/// rama de shim, ante el mismo estado de máquina, decía `warn`. Dos veredictos
/// opuestos según `core.symlinks`.
#[cfg(unix)]
#[test]
fn un_symlink_a_otro_script_no_se_hace_pasar_por_el_gate() {
    let dir = tempfile::tempdir().unwrap();
    let hooks = dir.path().join("kb").join(".git").join("hooks");
    fs::create_dir_all(&hooks).unwrap();
    let ajeno = dir.path().join("prettier-hook.sh");
    fs::write(&ajeno, b"#!/usr/bin/env bash\nexit 0\n").unwrap();
    std::os::unix::fs::symlink(&ajeno, hooks.join("pre-commit")).unwrap();
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "kb_precommit_hook");
    assert_eq!(c.estado, Estado::Warn);
    assert!(
        c.detalle.contains("no es el gate de la KB"),
        "detalle: {}",
        c.detalle
    );
}

/// El hook evalúa `[ -x ]`, no «existe». En unix un binario sin bit de
/// ejecución dejaba el check en `ok` mientras el gate estaba apagado.
#[cfg(unix)]
#[test]
fn un_exo_sin_bit_de_ejecucion_no_cuenta_como_instalado() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let bindir = dir.path().join("home").join(".local").join("bin");
    fs::create_dir_all(&bindir).unwrap();
    let ruta = bindir.join("exo");
    fs::write(&ruta, b"#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&ruta, fs::Permissions::from_mode(0o644)).unwrap();
    let informe = analiza(&entorno(dir.path()));
    let c = check(&informe, "hook_fallback_binary");
    assert_eq!(
        c.estado,
        Estado::Fail,
        "el fichero está, pero `[ -x ]` del hook diría que no"
    );
}

#[test]
fn el_check_de_rutas_portables_avisa_cuando_el_indice_trae_backslash() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES ('kb/log/alpha', 'log\\alpha.md', 'alpha', 'note', 0.0, NULL)",
        [],
    )
    .unwrap();
    drop(conn);

    let check = exo::doctor::check_rutas_portables_de(&db);
    assert_eq!(check.id, "index_paths_portable");
    assert_eq!(check.estado, exo::doctor::Estado::Warn);
    assert!(
        check.detalle.contains("exo index"),
        "el detalle debe nombrar el remedio: {}",
        check.detalle
    );
}

#[test]
fn el_check_de_rutas_portables_pasa_con_el_indice_limpio() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES ('kb/log/alpha', 'log/alpha.md', 'alpha', 'note', 0.0, NULL)",
        [],
    )
    .unwrap();
    drop(conn);

    let check = exo::doctor::check_rutas_portables_de(&db);
    assert_eq!(check.estado, exo::doctor::Estado::Ok);
}

/// Instala un `ENGINE_MIN` falso en el layout del plugin bajo `home`, en la
/// familia y versión dadas. Devuelve el directorio de esa versión.
fn plugin_con_engine_min(home: &Path, familia: &str, version: &str, engine_min: &str) -> PathBuf {
    let dir = home
        .join(".claude")
        .join("plugins")
        .join("cache")
        .join("exo")
        .join(familia)
        .join(version);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("ENGINE_MIN"), engine_min).unwrap();
    dir
}

#[test]
fn sin_plugin_instalado_plugin_compat_es_warn_no_fail() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "plugin_compat");
    assert_eq!(
        c.estado,
        Estado::Warn,
        "sin plugin no hay hooks que degradar, pero tampoco memoria"
    );
}

#[test]
fn plugin_con_engine_min_ya_satisfecho_es_ok() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    // "0.0.0" es <= a cualquier versión real del binario, sea cual sea hoy
    // engine/Cargo.toml: el test no depende de ese número.
    plugin_con_engine_min(&dir.path().join("home"), "exo", "1.0.0", "0.0.0");
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "plugin_compat");
    assert_eq!(c.estado, Estado::Ok);
}

#[test]
fn plugin_con_engine_min_futuro_es_fail() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    // "99.0.0" es mayor que cualquier versión real que este repo vaya a
    // publicar: garantiza el caso "binario viejo" sin acoplar el test al
    // valor actual de engine/Cargo.toml.
    plugin_con_engine_min(&dir.path().join("home"), "exo", "1.0.0", "99.0.0");
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "plugin_compat");
    assert_eq!(c.estado, Estado::Fail);
    assert!(
        c.detalle.contains("99.0.0"),
        "dice qué ENGINE_MIN exige el plugin: {}",
        c.detalle
    );
}

#[test]
fn ruta_de_wsl_no_sugiere_git_bash() {
    assert!(!exo::doctor::ruta_sugiere_git_bash(Path::new(
        r"C:\Windows\System32\bash.exe"
    )));
}

#[test]
fn ruta_bajo_git_for_windows_sugiere_git_bash_sin_ejecutar_nada() {
    assert!(exo::doctor::ruta_sugiere_git_bash(Path::new(
        r"C:\Program Files\Git\bin\bash.exe"
    )));
    assert!(exo::doctor::ruta_sugiere_git_bash(Path::new(
        r"C:\Program Files\Git\usr\bin\bash.exe"
    )));
}

#[test]
fn version_de_msys_se_reconoce_como_git_bash() {
    assert!(exo::doctor::salida_indica_git_bash(
        "GNU bash, version 5.2.26(1)-release (x86_64-pc-msys)"
    ));
}

#[test]
fn version_de_wsl_no_se_reconoce_como_git_bash() {
    assert!(!exo::doctor::salida_indica_git_bash(
        "GNU bash, version 5.1.16(1)-release (x86_64-pc-linux-gnu)"
    ));
}

#[cfg(windows)]
#[test]
fn bash_resuelto_bajo_system32_es_warn_no_ok() {
    let dir = tempfile::tempdir().unwrap();
    let bindir = dir.path().join("System32");
    fs::create_dir_all(&bindir).unwrap();
    // No hace falta un bash.exe real: la ruta ya lo descarta (System32) y el
    // intento de ejecutarlo fallará (no es un ejecutable válido), lo que
    // `es_git_bash` trata igual que "no dijo msys/mingw".
    fs::write(bindir.join("bash.exe"), b"no es un binario de verdad").unwrap();
    let mut env = entorno(dir.path());
    env.path = bindir.display().to_string();
    let informe = analiza(&env);
    let c = check(&informe, "git_bash");
    assert_eq!(c.estado, Estado::Warn);
    assert!(
        c.detalle.contains("WSL"),
        "dice que lo que resolvió es WSL, no Git Bash: {}",
        c.detalle
    );
}

#[test]
fn con_1_9_0_y_1_10_0_en_cache_elige_la_1_10_0() {
    let dir = tempfile::tempdir().unwrap();
    shim_precommit(&dir.path().join("kb"), SHIM_DE_LA_KB);
    plugin_con_script(&dir.path().join("home"), "exo", "1.9.0");
    let script_alto = plugin_con_script(&dir.path().join("home"), "exo", "1.10.0");
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "kb_precommit_hook");
    assert_eq!(c.estado, Estado::Ok);
    assert!(
        c.artefacto.contains(&script_alto.display().to_string()),
        "debía resolver a 1.10.0 (la más alta), no a 1.9.0 por orden de \
         texto: {}",
        c.artefacto
    );
}
