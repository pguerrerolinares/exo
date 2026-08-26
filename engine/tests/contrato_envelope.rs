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
        borradas: 3,
        trozos_embebidos: 4,
        trozos_reusados: 5,
    };
    let v = serde_json::to_value(&r).expect("serializar");
    let obj = v.as_object().expect("objeto");
    for k in [
        "indexed",
        "skipped",
        "deleted",
        "chunks_embedded",
        "chunks_reused",
    ] {
        assert!(obj.contains_key(k), "falta {k} en {v}");
    }
    assert_eq!(v["indexed"], 1);
    assert_eq!(v["chunks_embedded"], 4);
    for k in [
        "indexadas",
        "saltadas",
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
