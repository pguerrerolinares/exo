//! La plantilla de la KB semilla, embebida en el binario. `include_str!`
//! explícito diecisiete veces en vez de un macro-crate de embedding: D4 exige un
//! binario autosuficiente, y diecisiete líneas legibles valen más que una dependencia
//! que hay que auditar para publicar.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub const PLACEHOLDER: &str = "{{KB_NAME}}";

pub const FICHEROS: &[(&str, &str)] = &[
    ("README.md", include_str!("../kb-template/README.md")),
    (
        "core/core-index.md",
        include_str!("../kb-template/core/core-index.md"),
    ),
    (
        "core/doctrina.md",
        include_str!("../kb-template/core/doctrina.md"),
    ),
    (
        "learnings/_template.md",
        include_str!("../kb-template/learnings/_template.md"),
    ),
    (
        "learnings/orquestador-limpio.md",
        include_str!("../kb-template/learnings/orquestador-limpio.md"),
    ),
    (
        "learnings/recon-first.md",
        include_str!("../kb-template/learnings/recon-first.md"),
    ),
    (
        "learnings/fallo-silencioso.md",
        include_str!("../kb-template/learnings/fallo-silencioso.md"),
    ),
    (
        "learnings/el-brief-es-el-cuello-de-botella.md",
        include_str!("../kb-template/learnings/el-brief-es-el-cuello-de-botella.md"),
    ),
    (
        "learnings/verificar-ejecutando-no-leyendo.md",
        include_str!("../kb-template/learnings/verificar-ejecutando-no-leyendo.md"),
    ),
    (
        "learnings/instrumento-validado-antes-de-medir.md",
        include_str!("../kb-template/learnings/instrumento-validado-antes-de-medir.md"),
    ),
    (
        "learnings/la-prosa-no-es-enforcement.md",
        include_str!("../kb-template/learnings/la-prosa-no-es-enforcement.md"),
    ),
    (
        "learnings/hechos-del-harness-claude-code.md",
        include_str!("../kb-template/learnings/hechos-del-harness-claude-code.md"),
    ),
    (
        "learnings/construir-con-llms-y-descartar.md",
        include_str!("../kb-template/learnings/construir-con-llms-y-descartar.md"),
    ),
    (
        "projects/_template.md",
        include_str!("../kb-template/projects/_template.md"),
    ),
    (
        "log/_template.md",
        include_str!("../kb-template/log/_template.md"),
    ),
    (
        "archive/log/.gitkeep",
        include_str!("../kb-template/archive/log/.gitkeep"),
    ),
    ("AGENTS.md", include_str!("../kb-template/AGENTS.md")),
];

pub fn render(contenido: &str, kb_name: &str) -> String {
    contenido.replace(PLACEHOLDER, kb_name)
}

/// Vuelca la plantilla en `destino`. Devuelve las rutas escritas, en el orden
/// de `FICHEROS`, para que el llamante pueda decir qué hizo.
pub fn vuelca(destino: &Path, kb_name: &str) -> Result<Vec<PathBuf>> {
    let mut escritos = Vec::with_capacity(FICHEROS.len());
    for (rel, contenido) in FICHEROS {
        let ruta = destino.join(rel);
        if let Some(padre) = ruta.parent() {
            std::fs::create_dir_all(padre).with_context(|| format!("crear {}", padre.display()))?;
        }
        std::fs::write(&ruta, render(contenido, kb_name))
            .with_context(|| format!("escribir {}", ruta.display()))?;
        escritos.push(ruta);
    }
    Ok(escritos)
}
