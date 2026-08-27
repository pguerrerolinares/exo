//! `exo init`: creación de la config propia y migración de una sola vez desde
//! basic-memory.
//!
//! Es la ÚNICA lectura de `~/.basic-memory/config.json` que sobrevive en el
//! engine, y es explícita y borrable: una migración se puede eliminar en tres
//! meses, un fallback permanente no lo quita nadie nunca.

use crate::config::Embeddings;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Lee el JSON de basic-memory y devuelve `(ruta_kb, nombre_kb, embeddings)`.
///
/// El nombre sale de `default_project`, **no** de un literal `"kb-demo"`:
/// ese literal (`lib.rs:93` en la versión anterior) era el acoplamiento que
/// esta ola viene a matar; reintroducirlo aquí sería cambiar de sitio el bug.
pub fn desde_basic_memory(json: &str) -> Result<(PathBuf, String, Embeddings)> {
    let v: serde_json::Value =
        serde_json::from_str(json).context("la config de basic-memory no es JSON válido")?;
    let nombre = v
        .get("default_project")
        .and_then(|x| x.as_str())
        .context("default_project ausente en la config de basic-memory")?
        .to_string();
    let path = v
        .get("projects")
        .and_then(|p| p.get(&nombre))
        .and_then(|p| p.get("path"))
        .and_then(|p| p.as_str())
        .with_context(|| format!("projects.{nombre}.path ausente en la config de basic-memory"))?;
    let emb = Embeddings {
        model: v["semantic_embedding_model"]
            .as_str()
            .context("semantic_embedding_model ausente")?
            .to_string(),
        dims: v["semantic_embedding_dimensions"]
            .as_u64()
            .context("semantic_embedding_dimensions ausente")? as usize,
        min_similarity: v["semantic_min_similarity"]
            .as_f64()
            .context("semantic_min_similarity ausente")?,
    };
    Ok((PathBuf::from(path), nombre, emb))
}

/// Renderiza un valor de cadena como literal TOML **escapado**. La estructura
/// del fichero se sigue escribiendo a mano —los comentarios son la mitad del
/// valor de un TOML editable, y el serializador los pierde—, pero los valores
/// no: una comilla en un nombre o en una ruta (legal en Linux y macOS)
/// produciría un fichero que no se puede releer, y el fallo aparecería lejos
/// de aquí.
fn cadena_toml(s: &str) -> String {
    toml::Value::String(s.to_string()).to_string()
}

/// Escribe `config.toml`. Se niega si el destino existe y no hay `--force`:
/// pisar la config de alguien sin avisar es exactamente el tipo de efecto
/// silencioso que este proyecto persigue.
pub fn escribe_config(
    destino: &Path,
    kb: &Path,
    nombre: &str,
    emb: &Embeddings,
    db: &Path,
    force: bool,
) -> Result<()> {
    if destino.exists() && !force {
        anyhow::bail!(
            "ya existe una config en {} — repite con --force si de verdad quieres pisarla",
            destino.display()
        );
    }
    if let Some(padre) = destino.parent() {
        std::fs::create_dir_all(padre).with_context(|| format!("crear {}", padre.display()))?;
    }
    // Se serializa a mano en vez de con `toml::to_string`: los comentarios son
    // la mitad del valor de un TOML editable a mano, y el serializador los
    // pierde. El runbook de W11 documenta que este fichero se edita a mano.
    let contenido = format!(
        r#"schema_version = 1

[kb]
# Raíz de la KB markdown. Barras normales funcionan también en Windows.
path = {}
# Prefijo de permalink. Explícito: NO se deriva del nombre del directorio.
name = {}

[index]
db = {}

[embeddings]
model = {}
dims = {}
min_similarity = {}
"#,
        cadena_toml(&kb.display().to_string().replace('\\', "/")),
        cadena_toml(nombre),
        cadena_toml(&db.display().to_string().replace('\\', "/")),
        cadena_toml(&emb.model),
        emb.dims,
        emb.min_similarity,
    );
    std::fs::write(destino, contenido)
        .with_context(|| format!("escribir {}", destino.display()))?;
    Ok(())
}

/// Ruta por defecto del JSON de basic-memory, para `--from-basic-memory`.
///
/// `EXO_BASIC_MEMORY_JSON` la overridea cuando está puesta (y no vacía) —
/// mismo patrón que `resuelve_db`/`resuelve_kb` en `main.rs`: sin este seam,
/// los tests de adopción leerían el `~/.basic-memory/config.json` real de la
/// máquina y dejarían de ser herméticos.
pub fn ruta_basic_memory() -> Result<PathBuf> {
    if let Ok(v) = std::env::var("EXO_BASIC_MEMORY_JSON") {
        if !v.is_empty() {
            return Ok(PathBuf::from(v));
        }
    }
    Ok(dirs::home_dir()
        .context("sin HOME")?
        .join(".basic-memory/config.json"))
}

/// Comprueba que `kb` es un destino legítimo: inexistente, o existente y
/// vacío. Con `force`, cualquiera. Volcar sobre una KB con contenido pisaría
/// notas de alguien sin avisar — el efecto silencioso que este proyecto
/// persigue.
pub fn prepara_kb(kb: &Path, force: bool) -> Result<()> {
    if force || !kb.exists() {
        return Ok(());
    }
    let vacia = std::fs::read_dir(kb)
        .with_context(|| format!("leer {}", kb.display()))?
        .next()
        .is_none();
    if !vacia {
        anyhow::bail!(
            "{} existe y no está vacía — repite con --force si de verdad quieres volcar encima",
            kb.display()
        );
    }
    Ok(())
}
