//! C2 (review de pre-merge de `ola1a-config-propia`, 2026-08-26): el camino
//! `--create` de `write append` (`write_append_cmd`, `engine/src/main.rs`)
//! seguía derivando el prefijo de permalink de `kb.file_name()` en vez de
//! `exo::nombre_kb()` — el mismo disenso del gate M4 que `write_new_cmd` ya
//! había cerrado. Este test lo reproduce con un montaje donde el `[kb] name`
//! de la config y el basename del directorio de `--kb` son deliberadamente
//! distintos: es el ÚNICO montaje donde el defecto es visible (si coinciden,
//! el bug queda enmascarado por casualidad, que es justo como sobrevivió).
//!
//! Invoca el binario compilado como subproceso porque `write_append_cmd` es
//! privada del binario, no de la librería `exo` — mismo patrón de localizar
//! `target/{debug,release}/exo{.exe}` que `rechazo_envelope.rs`.

use std::io::Write;

fn bin() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("target");
    p.push(if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    });
    p.push(if cfg!(windows) { "exo.exe" } else { "exo" });
    p
}

/// Barra hacia delante para que la ruta sea TOML-válida también en Windows
/// (mismo tratamiento que `precedencia.rs`/`config.rs`).
fn forward(p: &std::path::Path) -> String {
    p.display().to_string().replace('\\', "/")
}

#[test]
fn write_append_create_usa_el_name_de_la_config_no_el_basename_del_dir_kb() {
    // KB real: basename ALEATORIO (tmpXXXXXX de tempfile), deliberadamente
    // distinto del `name` de la config de abajo.
    let kb_tmp = tempfile::tempdir().expect("tempdir kb");
    let kb = kb_tmp.path().to_path_buf();
    let kb_basename = kb.file_name().unwrap().to_string_lossy().into_owned();

    // Directorio de trabajo para config/db/cuerpo — NO es la KB.
    let work = tempfile::tempdir().expect("tempdir work");
    let db = work.path().join("index.db");
    let config_path = work.path().join("config.toml");
    let cuerpo_path = work.path().join("cuerpo.txt");

    const NAME_CONFIG: &str = "mi-proyecto-config";
    assert_ne!(
        kb_basename, NAME_CONFIG,
        "el montaje exige que difieran — si no, el defecto queda enmascarado"
    );

    let mut f = std::fs::File::create(&config_path).expect("crear config");
    write!(
        f,
        r#"schema_version = 1
[kb]
path = "{}"
name = "{NAME_CONFIG}"
[index]
db = "{}"
[embeddings]
model = "m"
dims = 768
min_similarity = 0.35
"#,
        forward(&kb),
        forward(&db),
    )
    .expect("escribir config");

    // Bootstrapea el esquema de `notas` sobre la KB temporal (vacía): así
    // `ruta_de` devuelve `None` para el permalink de prueba y el CLI entra en
    // la rama `--create`. Se hace vía librería, no como segundo subproceso.
    exo::indexer::indexa(&kb, &db).expect("bootstrap del índice");

    std::fs::write(&cuerpo_path, "contenido de prueba para el append\n").expect("escribir cuerpo");

    // Prefijo del permalink pasado por CLI, distinto TANTO del basename de la
    // KB como del `name` de la config: el código actual lo descarta (solo
    // usa los dos últimos segmentos para la ruta), así que si el test pasara
    // "por casualidad" con ese prefijo, sería la prueba de que el fix no está
    // realmente comprobando lo que dice comprobar.
    let slug = "write-create-permalink-c2";
    let permalink_arg = format!("otro-prefijo-cualquiera/log/{slug}");

    let out = std::process::Command::new(bin())
        .args(["write", "append", "--create", "--from"])
        .arg(&cuerpo_path)
        .args(["--kb"])
        .arg(&kb)
        .args(["--db"])
        .arg(&db)
        .args(["--json", &permalink_arg])
        .env("EXO_CONFIG", &config_path)
        .output()
        .expect("correr el binario");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "write append --create debe salir 0 (stdout={stdout}, stderr={stderr})"
    );

    let v: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("stdout no es envelope ({e}): {stdout}"));
    let permalink = v["data"]["permalink"]
        .as_str()
        .unwrap_or_else(|| panic!("sin data.permalink en el envelope: {v}"));

    assert!(
        permalink.starts_with(&format!("{NAME_CONFIG}/")),
        "el permalink debe llevar el `name` de la config como prefijo \
         ({NAME_CONFIG}/…), salió {permalink:?} — envelope completo: {v}"
    );
    assert!(
        !permalink.starts_with(&format!("{kb_basename}/")),
        "el permalink llevó el basename del directorio de --kb ({kb_basename}) \
         en vez del `name` de la config: regresión del bug C2. Envelope: {v}"
    );

    // Evidencia física, no solo el envelope: el frontmatter del fichero
    // creado en disco tiene que llevar el mismo permalink.
    let ruta_fichero = kb.join("log").join(format!("{slug}.md"));
    let frontmatter = std::fs::read_to_string(&ruta_fichero).unwrap_or_else(|e| {
        panic!(
            "no se pudo leer el fichero creado en {} ({e})",
            ruta_fichero.display()
        )
    });
    let linea_esperada = format!("permalink: {NAME_CONFIG}/log/{slug}");
    assert!(
        frontmatter.contains(&linea_esperada),
        "el frontmatter en disco no lleva `{linea_esperada}` — contenido:\n{frontmatter}"
    );
}
