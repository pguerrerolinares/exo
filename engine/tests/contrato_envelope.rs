//! El contrato JSON público v2. Este test es el gate: si alguien renombra una
//! clave sin subir `SCHEMA_VERSION`, aquí se pone rojo.
//!
//! Se comprueba sobre `serde_json::to_value` de structs construidos a mano,
//! no sobre una corrida real: el contrato es de FORMA, y una corrida real lo
//! ataría además a tener índice y modelo en la máquina.

#[test]
fn schema_version_es_2() {
    assert_eq!(exo::envelope::SCHEMA_VERSION, 2);
}

#[test]
fn las_claves_de_recall_estan_en_ingles() {
    let r = exo::recall::Recall {
        modo: "arranque".into(),
        query: None,
        cap_bytes: 2048,
        truncado: false,
        notas: vec![exo::recall::NotaRecall {
            permalink: "kb/core/x".into(),
            ruta: "core/x.md".into(),
            titulo: "X".into(),
            tier: Some("core".into()),
            score: None,
            snippet: None,
        }],
    };
    let v = serde_json::to_value(&r).expect("serializar");
    let obj = v.as_object().expect("objeto");
    for k in ["mode", "query", "cap_bytes", "truncated", "notes"] {
        assert!(obj.contains_key(k), "falta la clave {k} en {v}");
    }
    for k in ["modo", "truncado", "notas"] {
        assert!(!obj.contains_key(k), "sobrevive la clave española {k}");
    }
    let nota = &v["notes"][0];
    for k in ["permalink", "path", "title", "tier", "score", "snippet"] {
        assert!(nota.get(k).is_some(), "falta {k} en la nota: {nota}");
    }
    assert!(nota.get("ruta").is_none(), "sobrevive `ruta`");
    assert!(nota.get("titulo").is_none(), "sobrevive `titulo`");

    // No basta con que las claves existan: si dos renames estuvieran
    // intercambiados, todas estarían presentes y ninguna española, y el test
    // pasaría con el contrato invertido. Se asevera el VALOR.
    assert_eq!(nota["path"], "core/x.md");
    assert_eq!(nota["title"], "X");
    assert_eq!(v["mode"], "arranque");
    assert_eq!(v["truncated"], false);
    assert_eq!(v["cap_bytes"], 2048);
}

#[test]
fn las_claves_de_index_estan_en_ingles() {
    let r = exo::indexer::Resumen {
        indexadas: 1,
        saltadas: 2,
        sin_permalink: 6,
        borradas: 3,
        trozos_embebidos: 4,
        trozos_reusados: 5,
    };
    let v = serde_json::to_value(&r).expect("serializar");
    let obj = v.as_object().expect("objeto");
    for k in [
        "indexed",
        "skipped",
        "unreadable",
        "deleted",
        "chunks_embedded",
        "chunks_reused",
    ] {
        assert!(obj.contains_key(k), "falta {k} en {v}");
    }
    assert_eq!(v["indexed"], 1);
    assert_eq!(v["unreadable"], 6);
    assert_eq!(v["chunks_embedded"], 4);
    for k in [
        "indexadas",
        "saltadas",
        "sin_permalink",
        "borradas",
        "trozos_embebidos",
        "trozos_reusados",
    ] {
        assert!(!obj.contains_key(k), "sobrevive la clave española {k}");
    }
}

#[test]
fn las_claves_de_search_estan_en_ingles() {
    let b = exo::buscador::Busqueda {
        query: "q".into(),
        search_type: "fts".into(),
        elapsed_s: 0.1,
        results: vec![exo::buscador::Resultado {
            permalink: "kb/x".into(),
            tipo: "entity".into(),
            score: 1.0,
            ruta: Some("x.md".into()),
        }],
        avisos: vec!["algo".into()],
    };
    let v = serde_json::to_value(&b).expect("serializar");
    assert!(v.get("warnings").is_some(), "falta `warnings`: {v}");
    assert!(v.get("avisos").is_none(), "sobrevive `avisos`");
    assert!(
        v["results"][0].get("path").is_some(),
        "falta `path` en el resultado"
    );
    assert!(v["results"][0].get("ruta").is_none(), "sobrevive `ruta`");

    // Mismo motivo que en recall: aseverar solo presencia de clave no
    // distingue un swap de renames. Se asevera el VALOR.
    assert_eq!(v["results"][0]["path"], "x.md");
    assert_eq!(v["results"][0]["permalink"], "kb/x");
    assert_eq!(v["warnings"][0], "algo");
}

/// KB con git y su índice poblado a mano, igual que
/// `tests/targets_cli.rs::kb_con_indice`. No se importa de allí porque cada
/// fichero de `tests/` es un binario de test distinto; se replica el montaje.
fn kb_con_indice() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let kb = dir.path().to_path_buf();
    let cfg = kb.join("gitconfig-vacio");
    std::fs::write(&cfg, "").unwrap();
    let git = |args: &[&str]| {
        let s = std::process::Command::new("git")
            .arg("-C")
            .arg(&kb)
            .args(args)
            .env("GIT_CONFIG_GLOBAL", &cfg)
            .env("GIT_CONFIG_SYSTEM", &cfg)
            .env("GIT_AUTHOR_NAME", "f")
            .env("GIT_AUTHOR_EMAIL", "f@k.local")
            .env("GIT_COMMITTER_NAME", "f")
            .env("GIT_COMMITTER_EMAIL", "f@k.local")
            .env("GIT_AUTHOR_DATE", "2026-07-01T10:00:00+02:00")
            .env("GIT_COMMITTER_DATE", "2026-07-01T10:00:00+02:00")
            .output()
            .unwrap();
        assert!(s.status.success(), "git {args:?}");
    };
    std::fs::create_dir_all(kb.join("log")).unwrap();
    std::fs::write(
        kb.join("log/alpha.md"),
        "---\ntier: stable\n---\n# alpha\ncuerpo de alpha\n",
    )
    .unwrap();
    git(&["init", "-q"]);
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "inicial"]);

    let db = dir.path().join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES ('kb/log/alpha', 'log/alpha.md', 'alpha', 'note', 0.0, NULL)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO notas_fts (titulo, cuerpo, permalink)
         VALUES ('alpha', 'cuerpo de alpha', 'kb/log/alpha')",
        [],
    )
    .unwrap();
    drop(conn);
    (dir, db)
}

#[test]
fn las_claves_de_targets_estan_en_ingles() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    let r = exo::objetivos::busca_objetivos(&conn, dir.path(), "alpha", 10).unwrap();
    let v = serde_json::to_value(&r).expect("serializar");

    for k in ["topic", "candidates"] {
        assert!(v.get(k).is_some(), "falta la clave {k} en {v}");
    }
    for k in ["tema", "candidatos"] {
        assert!(v.get(k).is_none(), "sobrevive la clave española {k}");
    }

    let c = &v["candidates"][0];
    for k in [
        "permalink",
        "tier",
        "size_bytes",
        "headings",
        "last_commit",
        "snippet",
    ] {
        assert!(c.get(k).is_some(), "falta {k} en la candidata: {c}");
    }
    for k in ["tamano_bytes", "ultimo_commit"] {
        assert!(c.get(k).is_none(), "sobrevive la clave española {k}");
    }

    // Mismo motivo que en recall/search/write: no basta con presencia de
    // clave, un swap de renames pasaría igual. Se asevera el VALOR.
    assert_eq!(v["topic"], "alpha");
    assert_eq!(c["permalink"], "kb/log/alpha");
    assert_eq!(c["tier"], "stable");
    assert!(c["size_bytes"].as_i64().unwrap() > 0);
    assert_eq!(c["headings"][0], "alpha");
    assert_eq!(c["last_commit"], "2026-07-01T10:00:00+02:00");
    assert_eq!(c["snippet"], "cuerpo de alpha");
}

#[test]
fn las_claves_de_write_estan_en_ingles() {
    let e = exo::escritor::Escritura {
        op: "new".into(),
        permalink: "kb/projects/x".into(),
        ruta_rel: "projects/x.md".into(),
        ruta_abs: "/kb/projects/x.md".into(),
        creada: true,
        frontmatter_completado: vec!["tier".into()],
        forzado: false,
    };
    let v = serde_json::to_value(&e).expect("serializar");
    let obj = v.as_object().expect("objeto");
    for k in [
        "op",
        "permalink",
        "relative_path",
        "absolute_path",
        "created",
        "frontmatter_filled",
        "forced",
    ] {
        assert!(obj.contains_key(k), "falta la clave {k} en {v}");
    }
    for k in [
        "ruta_rel",
        "ruta_abs",
        "creada",
        "frontmatter_completado",
        "forzado",
    ] {
        assert!(!obj.contains_key(k), "sobrevive la clave española {k}");
    }

    // Mismo motivo que en recall/search: aseverar solo presencia de clave no
    // distingue un swap de renames. Se asevera el VALOR.
    assert_eq!(v["op"], "new");
    assert_eq!(v["permalink"], "kb/projects/x");
    assert_eq!(v["relative_path"], "projects/x.md");
    assert_eq!(v["absolute_path"], "/kb/projects/x.md");
    assert_eq!(v["created"], true);
    assert_eq!(v["frontmatter_filled"][0], "tier");
    assert_eq!(v["forced"], false);
}
