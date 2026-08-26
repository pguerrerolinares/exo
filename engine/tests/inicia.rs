//! `exo init`: escribe la config propia, y sabe migrarla desde el JSON de
//! basic-memory una sola vez. Es la ÚNICA lectura de basic-memory que
//! sobrevive en el engine, y es explícita.

const BM_JSON: &str = r#"{
  "projects": { "kb-demo": { "path": "C:/proyectos/homework/kb-demo" } },
  "default_project": "kb-demo",
  "semantic_embedding_model": "jinaai/jina-embeddings-v2-base-es",
  "semantic_embedding_dimensions": 768,
  "semantic_min_similarity": 0.35
}"#;

#[test]
fn migra_desde_basic_memory_leyendo_el_proyecto_por_defecto() {
    let (kb, nombre, emb) = exo::inicia::desde_basic_memory(BM_JSON).expect("migrar");
    assert_eq!(
        kb,
        std::path::PathBuf::from("C:/proyectos/homework/kb-demo")
    );
    // El nombre sale de `default_project`, NO de un literal "kb-demo"
    // hardcodeado: ese literal era justo el acoplamiento que se viene a matar.
    assert_eq!(nombre, "kb-demo");
    assert_eq!(emb.model, "jinaai/jina-embeddings-v2-base-es");
    assert_eq!(emb.dims, 768);
    assert_eq!(emb.min_similarity, 0.35);
}

#[test]
fn migrar_sin_default_project_falla_nombrando_la_clave() {
    let sin_default = r#"{ "projects": { "x": { "path": "/tmp/x" } } }"#;
    let err = exo::inicia::desde_basic_memory(sin_default).expect_err("debe fallar");
    let msg = format!("{err:#}");
    assert!(msg.contains("default_project"), "no nombra la clave: {msg}");
}

#[test]
fn escribe_una_config_que_se_puede_releer() {
    let dir = tempfile::tempdir().expect("tempdir");
    let destino = dir.path().join("config.toml");
    let emb = exo::config::Embeddings {
        model: "m/x".into(),
        dims: 768,
        min_similarity: 0.35,
    };
    exo::inicia::escribe_config(
        &destino,
        std::path::Path::new("C:/kb/demo"),
        "demo",
        &emb,
        std::path::Path::new("~/.exo/index.db"),
        false,
    )
    .expect("escribir");

    // `carga_desde` existe justo para esto: releer una ruta concreta sin
    // tocar `EXO_CONFIG`, que es entorno global compartido entre los tests
    // que cargo corre en paralelo dentro del mismo proceso.
    let cfg = exo::config::carga_desde(&destino).expect("releer lo que acabo de escribir");
    assert_eq!(cfg.kb.name, "demo");
    assert_eq!(cfg.embeddings.dims, 768);
}

#[test]
fn un_nombre_con_comillas_sigue_produciendo_toml_releible() {
    let dir = tempfile::tempdir().expect("tempdir");
    let destino = dir.path().join("config.toml");
    let emb = exo::config::Embeddings {
        model: "m/x".into(),
        dims: 768,
        min_similarity: 0.35,
    };
    let raro = r#"kb "de comillas""#;
    exo::inicia::escribe_config(
        &destino,
        std::path::Path::new("C:/kb/demo"),
        raro,
        &emb,
        std::path::Path::new("~/.exo/index.db"),
        false,
    )
    .expect("escribir");
    // El round-trip es el punto: no basta con que se escriba, tiene que
    // volver a leerse valiendo exactamente lo mismo.
    let cfg = exo::config::carga_desde(&destino).expect("releer");
    assert_eq!(cfg.kb.name, raro);
}

#[test]
fn crea_el_directorio_padre_en_el_primer_arranque() {
    let dir = tempfile::tempdir().expect("tempdir");
    let destino = dir.path().join("sub/dir/nuevo/config.toml");
    let emb = exo::config::Embeddings {
        model: "m/x".into(),
        dims: 768,
        min_similarity: 0.35,
    };
    exo::inicia::escribe_config(
        &destino,
        std::path::Path::new("C:/kb/demo"),
        "demo",
        &emb,
        std::path::Path::new("~/.exo/index.db"),
        false,
    )
    .expect("escribir creando el padre");
    assert!(destino.exists());
    let cfg = exo::config::carga_desde(&destino).expect("releer");
    assert_eq!(cfg.kb.name, "demo");
}

#[test]
fn no_pisa_una_config_existente_sin_force() {
    let dir = tempfile::tempdir().expect("tempdir");
    let destino = dir.path().join("config.toml");
    std::fs::write(&destino, "# la config del usuario\n").expect("sembrar");
    let emb = exo::config::Embeddings {
        model: "m/x".into(),
        dims: 768,
        min_similarity: 0.35,
    };
    let err = exo::inicia::escribe_config(
        &destino,
        std::path::Path::new("C:/kb/demo"),
        "demo",
        &emb,
        std::path::Path::new("~/.exo/index.db"),
        false,
    )
    .expect_err("debe negarse");
    assert!(format!("{err:#}").contains("--force"));
    // Y el fichero original sigue intacto: negarse no es medio-escribir.
    let contenido = std::fs::read_to_string(&destino).expect("releer");
    assert_eq!(contenido, "# la config del usuario\n");
}
