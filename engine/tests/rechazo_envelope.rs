//! El rechazo del dup-gate emite envelope cuando se pide `--json`.
//!
//! Contrato: exit 3 (que es por donde gatea el consumidor, jamás por campos
//! de `data`) Y envelope en stdout con `data.candidates`. Las dos cosas,
//! no una: el exit code sigue siendo el gate y el envelope es el detalle.
//!
//! Nombre del `command` del envelope de rechazo: `"write"` (no `"write.new"`
//! ni `"write.rechazo"`). Se reutiliza el mismo nombre que ya usa
//! `emite_escritura` para el envelope de éxito de `write new` y `write
//! append` (main.rs) — ambos subcomandos comparten un único `command` desde
//! antes de esta tarea, distinguidos por `data.op`/`data.reason`, no por el
//! nombre del comando. Inventar `write.new`/`write.rechazo` aquí habría sido
//! un segundo esquema de nombrado conviviendo con el que ya existe.

mod common;

#[test]
fn el_data_del_rechazo_duplicada_lleva_las_candidatas() {
    let r = exo::escritor::Rechazo::Duplicada {
        candidatas: vec![exo::escritor::Candidata {
            permalink: "kb/projects/x".into(),
            score: 0.87,
        }],
    };
    let v = r.data();
    assert_eq!(v["reason"], "duplicate");
    let c = &v["candidates"][0];
    assert_eq!(c["permalink"], "kb/projects/x");
    assert_eq!(c["score"], 0.87);
}

#[test]
fn el_data_del_rechazo_append_a_canon_lleva_el_tier() {
    let r = exo::escritor::Rechazo::AppendACanon {
        tier: Some("core".into()),
    };
    let v = r.data();
    assert_eq!(v["reason"], "append_to_canon");
    assert_eq!(v["tier"], "core");
    // Un rechazo que no es por duplicado NO inventa una lista vacía de
    // candidatas: ausencia de campo, no campo vacío.
    assert!(v.get("candidates").is_none());
}

#[test]
fn el_data_del_rechazo_append_a_canon_sin_tier_es_null_no_el_centinela_de_prosa() {
    // `"(sin tier)"` es la frase para el `Display` humano (stderr); en el
    // contrato JSON, ausencia de tier es `null`, no un string disfrazado de
    // ausencia — un consumidor que lea `data.tier` debe poder distinguir por
    // TIPO entre "core" (tier real) y ausencia.
    let r = exo::escritor::Rechazo::AppendACanon { tier: None };
    let v = r.data();
    assert_eq!(v["reason"], "append_to_canon");
    assert!(
        v["tier"].is_null(),
        "tier debe ser JSON null, no {:?}",
        v["tier"]
    );
    assert_eq!(
        format!("{r}"),
        "append a nota tier '(sin tier)': el canon se edita como delta, no se anexa. Usa la bitácora del frente, o --force si es una excepción consciente"
    );
}

/// Título compartido por la nota-fixture (ya indexada) y por el `write new`
/// del test: mismo título ⇒ mismo slug ⇒ Jaccard 1.0 contra el umbral 0.6 de
/// `escritor::UMBRAL_DUP`. Determinista por construcción, no por suerte de
/// contenido real.
const TITULO_DUP: &str = "Prueba determinista del dup-gate de write new";

/// Escribe, sin indexar todavía, una nota-fixture cuyo permalink termina en
/// el slug de `TITULO_DUP`. Separado de la indexación a propósito: `indexa`
/// necesita `EXO_CONFIG` ya apuntando a una config válida (lee el modelo de
/// embeddings incondicionalmente), así que la llamada a `indexa` vive DENTRO
/// del closure de `common::con_config` en el llamador — no aquí, donde
/// `EXO_CONFIG` todavía no está puesto.
fn escribe_nota_fixture(kb_fixture: &std::path::Path) {
    let dir_notas = kb_fixture.join("projects");
    std::fs::create_dir_all(&dir_notas).expect("crear projects/");
    std::fs::write(
        dir_notas.join("existente.md"),
        format!(
            "---\npermalink: \"rechazo-envelope-kb/projects/{}\"\ntitle: \"{TITULO_DUP}\"\n---\n",
            exo::escritor::slug(TITULO_DUP)
        ),
    )
    .expect("escribir nota-fixture");
}

#[test]
fn write_new_rechazado_con_json_emite_envelope_y_sale_3() {
    // El duplicado no depende de la KB real de la máquina: se monta aquí, en
    // un tempdir, indexando una nota cuyo slug coincide con el título que se
    // va a escribir. Antes este test se apoyaba en `~/.exo/index.db` y se
    // saltaba silenciosamente (`eprintln!` + `return`, que cargo cuenta como
    // `ok`) si ese índice no existía — abstención permanente en el runner
    // hermético que `test-hermetico.sh` existe para proteger. Ahora afirma
    // sin condición.
    let kb_fixture = tempfile::tempdir().expect("tempdir kb-fixture");
    escribe_nota_fixture(kb_fixture.path());
    let dir_db = tempfile::tempdir().expect("tempdir db");
    let db = dir_db.path().join("index.db");

    let kb_write = tempfile::tempdir().expect("tempdir kb destino");
    let cuerpo = tempfile::NamedTempFile::new().expect("tmp");

    // Config temporal del proceso (helper de `common`): tanto la llamada a
    // `indexa` de aquí abajo como el subproceso de `write new` (que no recibe
    // `--kb`/`--db` como fuente de config — esos flags solo dirigen el propio
    // comando, así que hereda `EXO_CONFIG` del entorno de ESTE proceso de
    // test) necesitan `EXO_CONFIG` puesto. Sin esto cae al
    // `~/.exo/config.toml` de la máquina en un runner limpio y el gate nunca
    // llega a evaluarse.
    common::con_config(kb_write.path(), "rechazo-envelope-kb", &db, || {
        // Cuerpo vacío ⇒ `trocea("")` no produce trozos ⇒ `indexa` jamás toca
        // el embedder de proceso (indexer.rs: "Nota sin trozos... jamás toca
        // el embedder"). El fixture indexa en milisegundos, sin cargar el
        // modelo ONNX.
        exo::indexer::indexa(kb_fixture.path(), &db).expect("indexar la nota-fixture");

        let out = std::process::Command::new(env!("CARGO_BIN_EXE_exo"))
            .args([
                "write",
                "new",
                "--db",
                db.to_str().unwrap(),
                "--kb",
                kb_write.path().to_str().unwrap(),
                "--dir",
                "projects",
                "--title",
                TITULO_DUP,
                "--from",
                cuerpo.path().to_str().unwrap(),
                "--json",
            ])
            .output()
            .expect("correr");

        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(
            out.status.code(),
            Some(3),
            "el gate debe salir 3 (stdout={stdout}, stderr={stderr})"
        );
        let v: serde_json::Value = serde_json::from_str(stdout.trim())
            .unwrap_or_else(|e| panic!("stdout no es envelope ({e}): {stdout}"));
        assert_eq!(v["command"], "write");
        assert_eq!(v["data"]["reason"], "duplicate");
        assert!(
            v["data"]["candidates"]
                .as_array()
                .is_some_and(|a| !a.is_empty()),
            "sin candidatas en el envelope: {v}"
        );
    });
}

/// Variante honesta de la anterior contra la KB real de la máquina (la que
/// tenía el test original antes de hermetizarse): `#[ignore]` explícito, no
/// un skip silencioso — no corre por defecto y se ve en la lista de tests
/// ignorados que no corre, en vez de aparecer como `ok` sin haber probado
/// nada. Se lanza a mano con `cargo test -- --ignored`.
#[test]
#[ignore = "depende de ~/.exo/index.db real; correr a mano con --ignored"]
fn write_new_rechazado_contra_la_kb_real_emite_envelope_y_sale_3() {
    let db = dirs::home_dir().expect("home").join(".exo/index.db");
    if !db.exists() {
        panic!(
            "no hay índice en {} — este test necesita la KB real, no lo saltes: quítalo de \
             --ignored solo cuando exista",
            db.display()
        );
    }

    let kb_tmp = tempfile::tempdir().expect("tempdir kb");
    let cuerpo = tempfile::NamedTempFile::new().expect("tmp");

    common::con_config(kb_tmp.path(), "rechazo-envelope-kb", &db, || {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_exo"))
            .args([
                "write",
                "new",
                "--db",
                db.to_str().unwrap(),
                "--kb",
                kb_tmp.path().to_str().unwrap(),
                "--dir",
                "projects",
                "--title",
                "exo — framework unificado de trabajo agéntico",
                "--from",
                cuerpo.path().to_str().unwrap(),
                "--json",
            ])
            .output()
            .expect("correr");

        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(
            out.status.code(),
            Some(3),
            "el gate debe salir 3 (stdout={stdout}, stderr={stderr})"
        );
        let v: serde_json::Value = serde_json::from_str(stdout.trim())
            .unwrap_or_else(|e| panic!("stdout no es envelope ({e}): {stdout}"));
        assert_eq!(v["command"], "write");
        assert_eq!(v["data"]["reason"], "duplicate");
        assert!(
            v["data"]["candidates"]
                .as_array()
                .is_some_and(|a| !a.is_empty()),
            "sin candidatas en el envelope: {v}"
        );
    });
}
