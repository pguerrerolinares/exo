use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::Once;

pub mod aristas;
pub mod buscador;
pub mod envelope;
pub mod indexer;
pub mod nota;
pub mod schema;
pub mod walker;

static REG: Once = Once::new();

/// Registra sqlite-vec como auto-extension exactamente una vez por proceso
/// (deferred de campaña 1, review opus m2-01: `sqlite3_auto_extension` es
/// acumulativo — registrar dos veces duplica el extension point).
fn registra_vec() {
    REG.call_once(|| unsafe {
        // Registro estático de sqlite-vec (patrón documentado del crate sqlite-vec).
        rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
            sqlite_vec::sqlite3_vec_init as *const (),
        )));
    });
}

/// Conexión en memoria con sqlite-vec registrado como auto-extension.
pub fn abre_db_en_memoria() -> Result<Connection> {
    registra_vec();
    Connection::open_in_memory().context("abrir sqlite en memoria")
}

/// Conexión a un fichero de DB en disco (mismo registro de sqlite-vec que
/// `abre_db_en_memoria`). Usada por `exo index`/`exo rebuild` (M2-03).
pub fn abre_db(ruta: &Path) -> Result<Connection> {
    registra_vec();
    Connection::open(ruta).with_context(|| format!("abrir sqlite en {}", ruta.display()))
}

/// Raíz de la KB desde `projects.kb-demo.path` en `~/.basic-memory/config.json`
/// (RO, D6; precedencia flags > config la resuelve el llamador con `--kb`).
/// Sin fallback inventado: si el fichero no existe, no es legible, o le
/// falta la clave, error claro y `Result::Err` (exit ≠0 en el CLI) —
/// aclaración vinculante m2-03.
pub fn kb_desde_config() -> Result<std::path::PathBuf> {
    let ruta = dirs::home_dir()
        .context("sin HOME: no se puede localizar ~/.basic-memory/config.json")?
        .join(".basic-memory/config.json");
    let contenido = std::fs::read_to_string(&ruta)
        .with_context(|| format!("leer config RO de basic-memory en {}", ruta.display()))?;
    let cfg: serde_json::Value = serde_json::from_str(&contenido)
        .with_context(|| format!("{} no es JSON válido", ruta.display()))?;
    let path = cfg
        .get("projects")
        .and_then(|p| p.get("kb-demo"))
        .and_then(|p| p.get("path"))
        .and_then(|p| p.as_str())
        .with_context(|| {
            format!("projects.kb-demo.path ausente en {}", ruta.display())
        })?;
    Ok(std::path::PathBuf::from(path))
}

/// Lee ~/.basic-memory/config.json (RO, D6), inicializa fastembed con el modelo
/// configurado y devuelve (embedding de la frase de prueba, dims declaradas).
pub fn embedder_desde_config() -> Result<(Vec<f32>, usize)> {
    let ruta = dirs::home_dir()
        .context("sin HOME")?
        .join(".basic-memory/config.json");
    let cfg: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&ruta).context("leer config bm")?)?;
    let modelo = cfg["semantic_embedding_model"]
        .as_str()
        .context("semantic_embedding_model ausente")?;
    let dims = cfg["semantic_embedding_dimensions"]
        .as_u64()
        .context("semantic_embedding_dimensions ausente")? as usize;
    let vector = embebe_frase(modelo, "el exocortex recuerda por ti")?;
    Ok((vector, dims))
}

/// Mapeo string de config -> camino fastembed (verificado contra docs.rs de
/// fastembed 5.17.3, la versión pineada en Cargo.toml):
///
/// `EmbeddingModel` (fastembed 5.17.3) NO tiene variante para
/// "jinaai/jina-embeddings-v2-base-es" — solo existen `JinaEmbeddingsV2BaseEN`
/// y `JinaEmbeddingsV2BaseCode` (confirmado en el enum de la crate). Camino
/// tomado: `UserDefinedEmbeddingModel`, descargando del mismo repo HF que
/// sirve basic-memory (vía `hf-hub` 0.5.0, ya dependencia transitiva de
/// fastembed, aquí declarada explícita para poder importarla) los 5 ficheros
/// que ese repo publica: `onnx/model.onnx` (~0.6 GB) + `tokenizer.json` +
/// `config.json` + `special_tokens_map.json` + `tokenizer_config.json`.
/// Pooling explícito a `Mean`: el repo declara en `1_Pooling/config.json`
/// `pooling_mode_mean_tokens=true` (word_embedding_dimension=768), y el ONNX
/// exportado no trae el pooling horneado.
fn embebe_frase(modelo: &str, frase: &str) -> Result<Vec<f32>> {
    use fastembed::{
        InitOptionsUserDefined, Pooling, TextEmbedding, TokenizerFiles, UserDefinedEmbeddingModel,
    };
    use hf_hub::api::sync::Api;

    let repo = Api::new()
        .context("crear cliente hf-hub")?
        .model(modelo.to_string());
    let leer = |fichero: &str| -> Result<Vec<u8>> {
        let ruta = repo
            .get(fichero)
            .with_context(|| format!("descargar {fichero} de {modelo}"))?;
        std::fs::read(&ruta).with_context(|| format!("leer {fichero} descargado"))
    };

    let onnx_file = leer("onnx/model.onnx")?;
    let tokenizer_files = TokenizerFiles {
        tokenizer_file: leer("tokenizer.json")?,
        config_file: leer("config.json")?,
        special_tokens_map_file: leer("special_tokens_map.json")?,
        tokenizer_config_file: leer("tokenizer_config.json")?,
    };

    let mut modelo_custom = UserDefinedEmbeddingModel::new(onnx_file, tokenizer_files);
    modelo_custom.pooling = Some(Pooling::Mean);

    let mut te = TextEmbedding::try_new_from_user_defined(modelo_custom, InitOptionsUserDefined::new())
        .context("inicializar fastembed con modelo custom jina-es")?;
    let mut out = te.embed(vec![frase.to_string()], None)?;
    Ok(out.pop().expect("un embedding"))
}
