//! Las tres funciones que leían `~/.basic-memory/config.json` ahora leen la
//! config propia. El test lo comprueba de la única forma falsable que hay:
//! apunta `EXO_CONFIG` a un fichero con valores IMPOSIBLES de encontrar en
//! ninguna config de basic-memory de esta máquina.

mod common;

use common::con_config_texto as con_config;

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
