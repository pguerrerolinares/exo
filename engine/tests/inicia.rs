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

#[test]
fn init_rechaza_un_directorio_no_vacio_sin_force() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(dir.path().join("algo.md"), "x").unwrap();
    let e = exo::inicia::prepara_kb(dir.path(), false).unwrap_err();
    assert!(
        e.to_string().contains("no está vacía"),
        "mensaje inesperado: {e}"
    );
}

#[test]
fn init_acepta_un_directorio_vacio() {
    let dir = tempfile::TempDir::new().unwrap();
    exo::inicia::prepara_kb(dir.path(), false).expect("dir vacío debe pasar");
}

#[test]
fn init_acepta_un_directorio_inexistente() {
    let dir = tempfile::TempDir::new().unwrap();
    let nueva = dir.path().join("kb-nueva");
    exo::inicia::prepara_kb(&nueva, false).expect("dir inexistente debe pasar");
}

/// `--from-basic-memory` adopta una KB existente: NO puede escribir ni un
/// byte dentro de ella. Con el cableado de la v1 de este plan, este test
/// fallaba de las dos formas posibles: sin `--force` abortaba, y con `--force`
/// machacaba `core/core-index.md` con la semilla.
#[test]
fn adopcion_no_toca_ni_un_fichero_de_la_kb_existente() {
    let tmp = tempfile::TempDir::new().unwrap();
    let kb = tmp.path().join("kb-poblada");
    std::fs::create_dir_all(kb.join("core")).unwrap();
    let canon = kb.join("core/core-index.md");
    std::fs::write(
        &canon,
        "---\npermalink: real/core/core-index\n---\nCONTENIDO REAL\n",
    )
    .unwrap();
    let antes = std::fs::read_to_string(&canon).unwrap();

    let bm = tmp.path().join("bm.json");
    let kb_json = kb.display().to_string().replace('\\', "/");
    std::fs::write(
        &bm,
        format!(
            r#"{{"projects":{{"real":{{"path":"{}"}}}},"default_project":"real","semantic_embedding_model":"{}","semantic_embedding_dimensions":768,"semantic_min_similarity":0.35}}"#,
            kb_json,
            exo::MODELO_JINA_ES
        ),
    )
    .unwrap();

    // Ejecuta el binario en modo adopción con config y db aisladas.
    let salida = std::process::Command::new(env!("CARGO_BIN_EXE_exo"))
        .args(["init", "--from-basic-memory", "--json"])
        .env("EXO_CONFIG", tmp.path().join("config.toml"))
        .env("EXO_DB", tmp.path().join("index.db"))
        .env("EXO_BASIC_MEMORY_JSON", &bm)
        .output()
        .expect("ejecutar exo init");

    assert!(
        salida.status.success(),
        "init falló: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(&canon).unwrap(),
        antes,
        "la adopción escribió en la KB"
    );
    assert!(
        !kb.join("AGENTS.md").exists(),
        "la adopción volcó la plantilla"
    );
    assert!(!kb.join(".git").exists(), "la adopción hizo git init");
}
