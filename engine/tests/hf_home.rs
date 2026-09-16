//! Task 13 (G): el engine tiene que respetar `HF_HOME`.
//!
//! Antes de esta tarea `Embedder::con_modelo` construía su cliente hf-hub
//! con `Api::new()`, que baja a `ApiBuilder::new()` → `Cache::default()`
//! (hf-hub 0.5.0) e ignora `HF_HOME` por completo — mientras
//! `doctor::Entorno::del_proceso` reimplementaba a mano la lógica de
//! `HF_HOME` asumiendo (falsamente) que el descargador la seguía. Estos
//! tests prueban el helper puro que ahora es la fuente única de la ruta
//! (`exo::cache_hf_del_entorno`), sin tocar red ni el modelo real: probar el
//! descargador de verdad exige la descarga de ~0.6 GB (ver
//! `tests/smoke.rs::jina_es_embebe_a_768`, `#[ignore]`).
//!
//! `HF_HOME` es del PROCESO: los tests de este fichero mutan la env, así que
//! se serializan con un candado propio (mismo patrón que
//! `tests/common/mod.rs` usa para `EXO_CONFIG`) para no correr en paralelo
//! con otro hilo del mismo binario.
use std::path::PathBuf;

static ENTORNO: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Restaura `HF_HOME` al salir de scope (incluido un panic a mitad del
/// test), igual que `RestauraEnv` de `tests/common/mod.rs` hace con
/// `EXO_CONFIG`.
struct RestauraHfHome {
    previo: Option<std::ffi::OsString>,
}

impl Drop for RestauraHfHome {
    fn drop(&mut self) {
        unsafe {
            match &self.previo {
                Some(v) => std::env::set_var("HF_HOME", v),
                None => std::env::remove_var("HF_HOME"),
            }
        }
    }
}

#[test]
fn con_hf_home_definida_la_cache_cae_bajo_esa_ruta() {
    let _guard = ENTORNO.lock().unwrap_or_else(|e| e.into_inner());
    let previo = std::env::var_os("HF_HOME");
    let _restaura = RestauraHfHome { previo };

    let tmp = tempfile::tempdir().unwrap();
    unsafe {
        std::env::set_var("HF_HOME", tmp.path());
    }

    let cache = exo::cache_hf_del_entorno();
    assert_eq!(
        cache,
        tmp.path().join("hub"),
        "con HF_HOME puesta, la caché debe caer bajo $HF_HOME/hub, no bajo ~/.cache/huggingface"
    );
}

#[test]
fn sin_hf_home_la_cache_cae_bajo_el_home_del_usuario() {
    let _guard = ENTORNO.lock().unwrap_or_else(|e| e.into_inner());
    let previo = std::env::var_os("HF_HOME");
    let _restaura = RestauraHfHome { previo };
    unsafe {
        std::env::remove_var("HF_HOME");
    }

    let cache = exo::cache_hf_del_entorno();
    let esperado: PathBuf = dirs::home_dir()
        .unwrap()
        .join(".cache")
        .join("huggingface")
        .join("hub");
    assert_eq!(
        cache, esperado,
        "sin HF_HOME, comportamiento actual sin cambios: ~/.cache/huggingface/hub"
    );
}

/// El check de `doctor` y el descargador no pueden volver a divergir: los
/// dos tienen que resolver la MISMA ruta a partir del MISMO entorno. Antes
/// de la Task 13 esto era cierto por coincidencia (dos implementaciones
/// manuales que resultaban dar el mismo resultado); ahora es cierto porque
/// `Entorno::del_proceso` llama al mismo helper.
#[test]
fn el_cache_hf_de_doctor_coincide_con_el_del_descargador() {
    let _guard = ENTORNO.lock().unwrap_or_else(|e| e.into_inner());
    let previo = std::env::var_os("HF_HOME");
    let _restaura = RestauraHfHome { previo };

    let tmp = tempfile::tempdir().unwrap();
    unsafe {
        std::env::set_var("HF_HOME", tmp.path());
    }

    let entorno = exo::doctor::Entorno::del_proceso();
    assert_eq!(
        entorno.cache_hf,
        exo::cache_hf_del_entorno(),
        "doctor y el descargador deben resolver la misma ruta de caché"
    );
}
