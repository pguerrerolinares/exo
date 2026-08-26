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

#[test]
fn write_new_rechazado_con_json_emite_envelope_y_sale_3() {
    // Se apoya en la KB real y su índice: es el único sitio donde hay un
    // duplicado que el gate reconozca. Si no existen, el test se salta
    // ruidosamente en vez de dar un verde falso.
    let db = dirs::home_dir().expect("home").join(".exo/index.db");
    if !db.exists() {
        eprintln!(
            "SKIP: no hay índice en {} — este test necesita la KB real",
            db.display()
        );
        return;
    }
    let mut bin = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    bin.push("target");
    bin.push(if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    });
    bin.push(if cfg!(windows) { "exo.exe" } else { "exo" });

    let cuerpo = tempfile::NamedTempFile::new().expect("tmp");
    let out = std::process::Command::new(&bin)
        .args([
            "write",
            "new",
            "--db",
            db.to_str().unwrap(),
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

    assert_eq!(out.status.code(), Some(3), "el gate debe salir 3");
    let stdout = String::from_utf8_lossy(&out.stdout);
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
}
