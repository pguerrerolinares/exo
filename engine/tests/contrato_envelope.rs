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
        elapsed_s: Some(0.5),
        refresh_s: None,
        avisos: vec!["arm vector INERTE".into()],
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
    assert_eq!(v["elapsed_s"], 0.5);
    assert!(v["refresh_s"].is_null());
    assert_eq!(v["warnings"][0], "arm vector INERTE");
    assert!(v.get("avisos").is_none(), "sobrevive `avisos`");
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
        aviso_kb_root: Some("otra kb".into()),
    };
    let v = serde_json::to_value(&b).expect("serializar");
    assert!(v.get("warnings").is_some(), "falta `warnings`: {v}");
    assert!(v.get("avisos").is_none(), "sobrevive `avisos`");
    assert!(
        v.as_object().unwrap().len() == 5,
        "aviso_kb_root no debe aparecer como clave en el envelope de search: {v}"
    );
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

#[test]
fn las_claves_de_budget_estan_en_ingles() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("core")).unwrap();
    std::fs::write(
        dir.path().join("core/big.md"),
        format!("---\ntier: core\n---\n{}", "x".repeat(9000)),
    )
    .unwrap();
    let informe = exo::presupuesto::analiza(
        dir.path(),
        exo::presupuesto::NOMINALES,
        &exo::presupuesto::EXCLUIDOS,
    )
    .unwrap();
    let v = serde_json::to_value(&informe).unwrap();

    for k in ["tiers", "offenders", "waived", "no_air", "notier"] {
        assert!(v.get(k).is_some(), "falta {k} en el informe: {v}");
    }
    for k in ["infractoras", "sin_aire"] {
        assert!(v.get(k).is_none(), "sobrevive la clave española {k}");
    }
    let o = &v["offenders"][0];
    assert_eq!(o["path"], "core/big.md");
    assert_eq!(o["tier"], "core");
    assert_eq!(o["budget"], 8500);
    // El swap que la presencia sola no vería: size_bytes y budget son ambos
    // enteros, así que se comprueba cuál es cuál.
    assert!(o["size_bytes"].as_i64().unwrap() > o["budget"].as_i64().unwrap());
    let f = &v["tiers"][0];
    assert_eq!(f["tier"], "core");
    assert_eq!(f["budget"], 8500);
    for k in ["notes", "bytes", "delta", "exceeded"] {
        assert!(f.get(k).is_some(), "falta {k} en tiers[]: {f}");
    }
    for k in ["notas", "presupuesto", "excedido"] {
        assert!(f.get(k).is_none(), "sobrevive la clave española {k}");
    }
}

#[test]
fn las_claves_de_lint_estan_en_ingles() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("suelto.txt"), "x").unwrap();
    let conn = exo::abre_db_en_memoria().unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    let informe = exo::lint::analiza(
        &conn,
        dir.path(),
        exo::presupuesto::NOMINALES,
        &exo::presupuesto::EXCLUIDOS,
    )
    .unwrap();
    let v = serde_json::to_value(&informe).unwrap();

    for k in ["ok", "findings", "waived"] {
        assert!(v.get(k).is_some(), "falta {k}: {v}");
    }
    assert!(
        v.get("hallazgos").is_none(),
        "sobrevive la clave española hallazgos"
    );
    assert_eq!(v["ok"], false);
    let h = &v["findings"][0];
    for k in ["type", "path", "detail"] {
        assert!(h.get(k).is_some(), "falta {k}: {h}");
    }
    for k in ["tipo", "ruta", "detalle"] {
        assert!(h.get(k).is_none(), "sobrevive la clave española {k}");
    }
    // type y path son ambos strings: sin comprobar el valor, un swap pasaría.
    assert_eq!(h["type"], "root_file");
    assert_eq!(h["path"], "suelto.txt");
}

#[test]
fn las_claves_de_doctor_estan_en_ingles() {
    let informe = exo::doctor::InformeDoctor {
        ok: false,
        plataforma: "linux",
        checks: vec![exo::doctor::Check {
            id: "config",
            estado: exo::doctor::Estado::Fail,
            artefacto: "/home/x/.exo/config.toml".into(),
            detalle: "no existe".into(),
        }],
    };
    let v = serde_json::to_value(&informe).expect("serializar");
    let obj = v.as_object().expect("objeto");

    let mut claves: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
    claves.sort_unstable();
    assert_eq!(
        claves,
        vec!["checks", "ok", "platform"],
        "renombrar una clave de data exige subir SCHEMA_VERSION"
    );
    assert!(!obj.contains_key("plataforma"), "sobrevive `plataforma`");

    let check = v["checks"][0].as_object().expect("objeto");
    let mut claves: Vec<&str> = check.keys().map(|k| k.as_str()).collect();
    claves.sort_unstable();
    assert_eq!(claves, vec!["artifact", "detail", "id", "status"]);
    for k in ["estado", "artefacto", "detalle"] {
        assert!(!check.contains_key(k), "sobrevive la clave española {k}");
    }

    // No basta con que las claves existan: `id`, `artifact` y `detail` son los
    // tres de tipo texto, así que dos `#[serde(rename)]` intercambiados
    // dejarían las cuatro claves presentes, ninguna española viva, y este test
    // verde con el contrato invertido. Por eso se aserta el VALOR de cada una
    // —el mismo patrón que ya siguen `recall`, `index` y `budget` en este
    // fichero—.
    assert_eq!(check["id"], "config");
    assert_eq!(check["artifact"], "/home/x/.exo/config.toml");
    assert_eq!(check["detail"], "no existe");
    assert_eq!(v["ok"], false);
    assert_eq!(v["platform"], "linux");

    // El vocabulario de cuatro estados es contrato: los consumidores filtran
    // por estas cadenas, y `Na` serializando como "Na" en vez de "na" las
    // rompería sin que ningún test de forma lo notara.
    assert_eq!(check["status"], "fail");
    assert_eq!(serde_json::to_value(exo::doctor::Estado::Na).unwrap(), "na");
}
