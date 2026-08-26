//! Contrato de `exo::config`: precedencia, errores accionables y expansión
//! de `~`. Todos los tests fijan `EXO_CONFIG` a un fichero temporal, así
//! que ninguno toca el `~/.exo/config.toml` de la máquina.

use std::io::Write;

/// Escribe un config temporal y devuelve su ruta. El `TempDir` se devuelve
/// también: si se dropea, el directorio desaparece bajo los pies del test.
fn config_temporal(contenido: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let ruta = dir.path().join("config.toml");
    let mut f = std::fs::File::create(&ruta).expect("crear config");
    f.write_all(contenido.as_bytes()).expect("escribir config");
    (dir, ruta)
}

const COMPLETO: &str = r#"
schema_version = 1

[kb]
path = "C:/kb/demo"
name = "demo"

[index]
db = "~/.exo/index.db"

[embeddings]
model = "jinaai/jina-embeddings-v2-base-es"
dims = 768
min_similarity = 0.35
"#;

#[test]
fn carga_un_config_completo() {
    let (_dir, ruta) = config_temporal(COMPLETO);
    unsafe { std::env::set_var("EXO_CONFIG", &ruta) };
    let cfg = exo::config::carga().expect("carga");
    assert_eq!(cfg.schema_version, 1);
    assert_eq!(cfg.kb.name, "demo");
    assert_eq!(cfg.kb.path, std::path::PathBuf::from("C:/kb/demo"));
    assert_eq!(cfg.embeddings.dims, 768);
    assert_eq!(cfg.embeddings.min_similarity, 0.35);
    unsafe { std::env::remove_var("EXO_CONFIG") };
}

#[test]
fn fichero_ausente_nombra_el_comando_de_arranque() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ruta = dir.path().join("no-existe.toml");
    unsafe { std::env::set_var("EXO_CONFIG", &ruta) };
    let err = exo::config::carga().expect_err("debe fallar");
    let msg = format!("{err:#}");
    // El mensaje tiene que decirle al usuario qué ejecutar, no solo que falta
    // un fichero: es la diferencia entre un error y un callejón sin salida.
    assert!(
        msg.contains("exo init"),
        "mensaje sin salida accionable: {msg}"
    );
    assert!(msg.contains("no-existe.toml"), "mensaje sin la ruta: {msg}");
    unsafe { std::env::remove_var("EXO_CONFIG") };
}

#[test]
fn clave_ausente_nombra_la_clave_y_la_ruta() {
    let sin_name = r#"
schema_version = 1

[kb]
path = "C:/kb/demo"

[index]
db = "~/.exo/index.db"

[embeddings]
model = "m"
dims = 768
min_similarity = 0.35
"#;
    let (_dir, ruta) = config_temporal(sin_name);
    unsafe { std::env::set_var("EXO_CONFIG", &ruta) };
    let err = exo::config::carga().expect_err("debe fallar");
    let msg = format!("{err:#}");
    assert!(msg.contains("name"), "no nombra la clave: {msg}");
    assert!(msg.contains("config.toml"), "no nombra la ruta: {msg}");
    unsafe { std::env::remove_var("EXO_CONFIG") };
}

#[test]
fn expande_tilde_en_rutas() {
    let home = dirs::home_dir().expect("home");
    let expandida = exo::config::expande_tilde(std::path::Path::new("~/.exo/index.db"));
    assert_eq!(expandida, home.join(".exo/index.db"));
    // Una ruta que NO empieza por `~` se devuelve intacta, incluida una
    // absoluta de Windows con dos puntos.
    let absoluta = std::path::Path::new("C:/proyectos/kb");
    assert_eq!(exo::config::expande_tilde(absoluta), absoluta.to_path_buf());
}

#[test]
fn acepta_barras_de_windows_en_el_path_de_la_kb() {
    let con_backslash = r#"
schema_version = 1

[kb]
path = 'C:\proyectos\homework\kb-demo'
name = "kb-demo"

[index]
db = "~/.exo/index.db"

[embeddings]
model = "m"
dims = 768
min_similarity = 0.35
"#;
    let (_dir, ruta) = config_temporal(con_backslash);
    unsafe { std::env::set_var("EXO_CONFIG", &ruta) };
    let cfg = exo::config::carga().expect("carga con backslashes");
    assert_eq!(
        cfg.kb.path,
        std::path::PathBuf::from(r"C:\proyectos\homework\kb-demo")
    );
    unsafe { std::env::remove_var("EXO_CONFIG") };
}

#[test]
fn toml_invalido_no_se_disfraza_de_clave_ausente() {
    let (_dir, ruta) = config_temporal("esto no es toml [[[");
    unsafe { std::env::set_var("EXO_CONFIG", &ruta) };
    let err = exo::config::carga().expect_err("debe fallar");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("no es TOML válido"),
        "un parse error debe decir que es un parse error: {msg}"
    );
    unsafe { std::env::remove_var("EXO_CONFIG") };
}
