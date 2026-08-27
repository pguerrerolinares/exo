//! Config temporal para los tests que dependen de `config_embeddings()` o de
//! `min_similitud_de_config()`. Sin esto, 61 tests en 9 suites leen el
//! `~/.exo/config.toml` de la máquina que los corre y mueren en un runner
//! limpio (medido 2026-08-27: `CARGO_EXIT=101`).
//!
//! Cada binario de integración es un PROCESO propio, así que una instancia de
//! `ENTORNO` por binario es exactamente el alcance correcto: serializa los
//! tests de ese fichero y no toca a los demás.

#![allow(dead_code)] // cada suite usa un subconjunto; sin esto, warnings por fichero

use std::io::Write;
use std::path::Path;

/// El entorno es global al proceso y cargo corre los tests de un binario en
/// hilos paralelos. Todo test que toque `EXO_CONFIG` toma este candado durante
/// toda su vida. `unwrap_or_else(|e| e.into_inner())` porque un test que panica
/// con el candado tomado envenena el mutex, y eso convertiría un fallo en una
/// cascada que tapa al original.
static ENTORNO: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Mismo modelo y dimensionalidad que produce `exo init` en producción: las
/// suites que indexan comparan contra `meta`, y un modelo distinto dispararía
/// la guarda de `guarda_modelo` y convertiría un test de otra cosa en un
/// fallo de guarda.
/// **No dupliques el literal**: `repo_hf` en `lib.rs` solo pinea la revisión
/// del modelo si la cadena coincide EXACTAMENTE con esta constante, y un
/// literal que divergiera en silencio destinaría a `main` (móvil). Mismo
/// criterio que `init_cmd` documenta en `src/main.rs:393-397`.
pub const MODELO: &str = exo::MODELO_JINA_ES;
pub const DIMS: usize = 768;
pub const MIN_SIMILARITY: f64 = 0.35;

/// Renderiza un `config.toml` válido. Barras normales: el TOML las acepta en
/// Windows y evita el escapado de `\` que rompería el fichero.
pub fn render_config(kb: &Path, nombre: &str, db: &Path) -> String {
    format!(
        "schema_version = 1\n\n\
         [kb]\npath = \"{}\"\nname = \"{}\"\n\n\
         [index]\ndb = \"{}\"\n\n\
         [embeddings]\nmodel = \"{MODELO}\"\ndims = {DIMS}\nmin_similarity = {MIN_SIMILARITY}\n",
        kb.display().to_string().replace('\\', "/"),
        nombre,
        db.display().to_string().replace('\\', "/"),
    )
}

/// Corre `f` con `EXO_CONFIG` apuntando a una config temporal que describe
/// `kb`/`nombre`/`db`, y **restaura el valor previo** al salir.
///
/// Restaurar, no borrar: en el runner hermético `EXO_CONFIG` viene puesto
/// desde FUERA para todo el proceso (`scripts/test-hermetico.sh`). Un
/// `remove_var` dejaría al resto del binario sin ese centinela, y cualquier
/// test posterior que leyera config caería al `~/.exo/config.toml` de la
/// máquina: verde aquí, rojo en el CI limpio. Es el modo de fallo exacto que
/// esta fase existe para matar. (`tests/config_cableado.rs` ya tiene el
/// defecto; este helper no lo hereda.)
pub fn con_config<T>(kb: &Path, nombre: &str, db: &Path, f: impl FnOnce() -> T) -> T {
    let _guarda = ENTORNO.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().expect("tempdir de config");
    let ruta = dir.path().join("config.toml");
    let mut fh = std::fs::File::create(&ruta).expect("crear config.toml");
    fh.write_all(render_config(kb, nombre, db).as_bytes())
        .expect("escribir config.toml");
    let previo = std::env::var_os("EXO_CONFIG");
    unsafe { std::env::set_var("EXO_CONFIG", &ruta) };
    let r = f();
    unsafe {
        match previo {
            Some(v) => std::env::set_var("EXO_CONFIG", v),
            None => std::env::remove_var("EXO_CONFIG"),
        }
    }
    r
}
