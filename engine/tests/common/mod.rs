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

/// Restaura `EXO_CONFIG` a su valor previo cuando `_guarda` sale de scope —
/// vía `Drop`, no vía código que corre "después de `f()`".
///
/// Por qué `Drop` y no un restore manual tras `let r = f();`: si `f` entra en
/// pánico, un restore manual posterior a esa línea nunca se ejecuta, y
/// `EXO_CONFIG` queda apuntando a un tempdir que el `Drop` del `TempDir` ya
/// borró — contaminando TODOS los tests siguientes de ese binario con una
/// ruta muerta (el mutex `ENTORNO` sí sobrevive gracias a
/// `unwrap_or_else(|e| e.into_inner())`, pero el env no tiene ese mismo
/// salvavidas). `Drop` corre también al desenrollar la pila por panic —es el
/// único punto que garantiza la restauración pase lo que pase—, así que la
/// guarda vive en una estructura propia en vez de en código plano.
struct RestauraEnv {
    previo: Option<std::ffi::OsString>,
}

impl Drop for RestauraEnv {
    fn drop(&mut self) {
        unsafe {
            match &self.previo {
                Some(v) => std::env::set_var("EXO_CONFIG", v),
                None => std::env::remove_var("EXO_CONFIG"),
            }
        }
    }
}

/// Lógica compartida de `con_config`/`con_config_texto`, ya con el candado
/// `ENTORNO` tomado por quien llama: escribe `contenido` en un
/// `config.toml` temporal, apunta `EXO_CONFIG` a él, corre `f`, y devuelve su
/// resultado. La restauración de `EXO_CONFIG` la hace `RestauraEnv::drop`,
/// no esta función.
///
/// Sin candado propio: la toma `con_config_interna` (la envoltura pública)
/// para los casos normales, y el test de panic la llama directamente porque
/// necesita sostener el candado durante todo el `catch_unwind` sin
/// re-entrar en un `Mutex` no reentrante.
fn con_config_sin_candado<T>(contenido: &str, f: impl FnOnce() -> T) -> T {
    let dir = tempfile::tempdir().expect("tempdir de config");
    let ruta = dir.path().join("config.toml");
    let mut fh = std::fs::File::create(&ruta).expect("crear config.toml");
    fh.write_all(contenido.as_bytes())
        .expect("escribir config.toml");
    let previo = std::env::var_os("EXO_CONFIG");
    unsafe { std::env::set_var("EXO_CONFIG", &ruta) };
    let _restaura = RestauraEnv { previo };
    f()
}

/// Toma el candado `ENTORNO` y delega en `con_config_sin_candado`.
///
/// Restaurar, no borrar: en el runner hermético `EXO_CONFIG` viene puesto
/// desde FUERA para todo el proceso (`scripts/test-hermetico.sh`). Un
/// `remove_var` dejaría al resto del binario sin ese centinela, y cualquier
/// test posterior que leyera config caería al `~/.exo/config.toml` de la
/// máquina: verde aquí, rojo en el CI limpio. Es el modo de fallo exacto que
/// esta fase existe para matar.
fn con_config_interna<T>(contenido: &str, f: impl FnOnce() -> T) -> T {
    let _guarda = ENTORNO.lock().unwrap_or_else(|e| e.into_inner());
    con_config_sin_candado(contenido, f)
}

/// Corre `f` con `EXO_CONFIG` apuntando a una config temporal que describe
/// `kb`/`nombre`/`db`, y **restaura el valor previo** al salir (ver
/// `RestauraEnv`). Firma sin cambios: la usan nueve suites.
pub fn con_config<T>(kb: &Path, nombre: &str, db: &Path, f: impl FnOnce() -> T) -> T {
    con_config_interna(&render_config(kb, nombre, db), f)
}

/// Igual que `con_config`, pero para montajes que necesitan un TOML que
/// `render_config` no puede producir —p.ej. `config_cableado.rs`, que fija
/// valores TOTALMENTE imposibles (`model = "modelo/imposible-v9"`,
/// `dims = 1234`, …) para probar que el código lee la config PROPIA de exo,
/// no la de basic-memory, y no puede pasar por las constantes
/// `MODELO`/`DIMS`/`MIN_SIMILARITY` de este módulo. Recibe el TOML ya
/// armado tal cual.
pub fn con_config_texto<T>(contenido: &str, f: impl FnOnce() -> T) -> T {
    con_config_interna(contenido, f)
}

#[test]
fn con_config_restaura_env_aunque_f_panique() {
    // Candado tomado aquí, no dentro de `con_config_sin_candado` (que no lo
    // toma): `ENTORNO` no es reentrante, así que si este test también
    // llamara a `con_config`/`con_config_interna` (que sí lo toman)
    // reentraría en el mismo Mutex desde el mismo hilo y se bloquearía para
    // siempre. Tomarlo aquí y llamar a la variante "sin candado" evita el
    // problema y además aísla el test de otros tests del binario que toquen
    // `EXO_CONFIG` concurrentemente.
    let _guarda = ENTORNO.lock().unwrap_or_else(|e| e.into_inner());

    let previo_real = std::env::var_os("EXO_CONFIG");
    unsafe { std::env::set_var("EXO_CONFIG", "valor-de-partida-conocido") };

    let resultado = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        con_config_sin_candado("contenido irrelevante: f panica antes de leerlo", || {
            panic!("panic deliberado del test de RestauraEnv")
        })
    }));
    assert!(
        resultado.is_err(),
        "la clausura debía panicar; si no panicó el test no prueba nada"
    );

    assert_eq!(
        std::env::var("EXO_CONFIG").as_deref(),
        Ok("valor-de-partida-conocido"),
        "EXO_CONFIG debía quedar restaurado a su valor de partida tras el panic, \
         no colgado del tempdir ya borrado"
    );

    // Deja el entorno como lo encontró, para no ensuciar tests posteriores
    // del binario que corran tras este.
    unsafe {
        match previo_real {
            Some(v) => std::env::set_var("EXO_CONFIG", v),
            None => std::env::remove_var("EXO_CONFIG"),
        }
    }
}
