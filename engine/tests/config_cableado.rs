//! Las tres funciones que leían `~/.basic-memory/config.json` ahora leen la
//! config propia. El test lo comprueba de la única forma falsable que hay:
//! apunta `EXO_CONFIG` a un fichero con valores IMPOSIBLES de encontrar en
//! ninguna config de basic-memory de esta máquina.

use std::io::Write;

/// El entorno es global al proceso y cargo corre los tests de un fichero en
/// hilos paralelos. Todo test que toque `EXO_CONFIG` toma este candado durante
/// toda su vida. `unwrap_or_else(|e| e.into_inner())` porque un test que panica
/// con el candado tomado envenena el mutex, y eso convertiría un fallo en una
/// cascada de fallos que tapan al original.
static ENTORNO: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn con_config<T>(contenido: &str, f: impl FnOnce() -> T) -> T {
    let _guarda = ENTORNO.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().expect("tempdir");
    let ruta = dir.path().join("config.toml");
    let mut fh = std::fs::File::create(&ruta).expect("crear");
    fh.write_all(contenido.as_bytes()).expect("escribir");
    unsafe { std::env::set_var("EXO_CONFIG", &ruta) };
    let r = f();
    unsafe { std::env::remove_var("EXO_CONFIG") };
    r
}

const CFG: &str = r#"
schema_version = 1

[kb]
path = "C:/kb/valor-imposible-de-basic-memory"
name = "valor-imposible"

[index]
db = "~/.exo/index.db"

[embeddings]
model = "modelo/imposible-v9"
dims = 1234
min_similarity = 0.99
"#;

#[test]
fn kb_desde_config_lee_la_config_propia() {
    let kb = con_config(CFG, || exo::kb_desde_config().expect("kb"));
    assert_eq!(
        kb,
        std::path::PathBuf::from("C:/kb/valor-imposible-de-basic-memory")
    );
}

#[test]
fn config_embeddings_lee_la_config_propia() {
    let c = con_config(CFG, || exo::config_embeddings().expect("emb"));
    assert_eq!(c.modelo, "modelo/imposible-v9");
    assert_eq!(c.dims, 1234);
}

#[test]
fn nombre_kb_lee_la_config_propia() {
    let n = con_config(CFG, || exo::nombre_kb().expect("nombre"));
    assert_eq!(n, "valor-imposible");
}

#[test]
fn min_similitud_lee_la_config_propia() {
    let m = con_config(CFG, || exo::min_similitud_de_config().expect("min"));
    assert_eq!(m, 0.99);
}

#[test]
fn el_db_de_la_config_expande_tilde() {
    let db = con_config(CFG, || {
        exo::config::expande_tilde(&exo::config::carga().expect("cfg").index.db)
    });
    let home = dirs::home_dir().expect("home");
    assert_eq!(db, home.join(".exo/index.db"));
}
