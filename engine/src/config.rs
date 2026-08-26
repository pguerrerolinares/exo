//! Config propia de exo (`~/.exo/config.toml`).
//!
//! Sustituye la lectura RO de `~/.basic-memory/config.json` que hacían
//! `kb_desde_config`, `config_embeddings` y `min_similitud_de_config`. Ese
//! acoplamiento era el bloqueante de M5b: el sustituto no puede depender del
//! sustituido para arrancar.
//!
//! Precedencia (spec §G1): `flag CLI > env > este fichero > error accionable`.
//! La parte `flag > env` la resuelve el llamador en `main.rs`; aquí vive el
//! último escalón. **Sin defaults inventados**: un default silencioso es la
//! clase de fallo que este proyecto existe para no volver a tener.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    pub schema_version: u32,
    pub kb: Kb,
    pub index: Index,
    pub embeddings: Embeddings,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Kb {
    pub path: PathBuf,
    /// Prefijo de permalink. **Explícito, no derivado de `path.file_name()`**:
    /// cierra el disenso abierto del gate M4, donde la spec §3.1 afirmaba que
    /// salía de la config y el código lo sacaba del nombre del directorio.
    /// Hoy coinciden; el día que no, reventaba en silencio.
    pub name: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Index {
    pub db: PathBuf,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Embeddings {
    pub model: String,
    pub dims: usize,
    pub min_similarity: f64,
}

/// Ruta del fichero de config: `$EXO_CONFIG` si está definida, si no
/// `~/.exo/config.toml`.
pub fn ruta_config() -> Result<PathBuf> {
    if let Ok(v) = std::env::var("EXO_CONFIG") {
        if !v.is_empty() {
            return Ok(PathBuf::from(v));
        }
    }
    Ok(dirs::home_dir()
        .context("sin HOME: no se puede localizar ~/.exo/config.toml")?
        .join(".exo/config.toml"))
}

/// Expande un `~` inicial a `$HOME`. Cualquier otra ruta se devuelve intacta
/// —incluidas las absolutas de Windows, que llevan dos puntos y no tilde.
pub fn expande_tilde(p: &Path) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(resto) = s.strip_prefix("~/").or_else(|| s.strip_prefix("~\\")) {
        if let Some(home) = dirs::home_dir() {
            return home.join(resto);
        }
    }
    p.to_path_buf()
}

/// Carga y valida la config. Errores accionables por contrato: el de fichero
/// ausente nombra el comando que lo crea, el de clave ausente nombra la clave
/// y la ruta.
pub fn carga() -> Result<Config> {
    let ruta = ruta_config()?;
    let contenido = std::fs::read_to_string(&ruta).map_err(|e| {
        anyhow::anyhow!(
            "no encuentro la config de exo en {} ({e}).\n\
             Créala con `exo init --from-basic-memory` si vienes de basic-memory, \
             o con `exo init --kb <ruta> --name <nombre>`.",
            ruta.display()
        )
    })?;
    // Dos errores distintos con dos mensajes distintos: un TOML corrupto no
    // debe disfrazarse de "te falta una clave", que manda al usuario a buscar
    // donde no es.
    let valor: toml::Value = toml::from_str(&contenido)
        .with_context(|| format!("{} no es TOML válido", ruta.display()))?;
    let cfg: Config = valor.try_into().map_err(|e| {
        anyhow::anyhow!("config incompleta o mal tipada en {}: {e}", ruta.display())
    })?;
    Ok(cfg)
}
