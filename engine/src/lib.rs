use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Mutex, Once};

pub mod aristas;
pub mod buscador;
pub mod envelope;
pub mod indexer;
pub mod nota;
pub mod recall;
pub mod schema;
pub mod trozos;
pub mod vectores;
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
    let conn =
        Connection::open(ruta).with_context(|| format!("abrir sqlite en {}", ruta.display()))?;
    // Espera si otra invocación tiene la DB tomada en vez de fallar en el
    // acto (hallazgo del gate M6): con el indexado en el hook de cierre y el
    // recall en el de arranque, dos procesos pueden solaparse. Sin esto, el
    // recall devuelve SQLITE_BUSY, el hook cae al fallback y Paul pierde el
    // mapa de la KB esa sesión por una carrera de milisegundos. 5 s es de
    // sobra para un indexado incremental y sigue muy por debajo del timeout
    // que el harness da a un hook.
    conn.busy_timeout(std::time::Duration::from_secs(5))
        .context("fijar busy_timeout")?;
    Ok(conn)
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

/// Refresco del índice ANTES de servir un recall (M6-01, "índice fresco sin
/// daemon"). basic-memory mantenía el índice al día con un watch en segundo
/// plano; exo indexa **al invocar** (spec §4.2: "incremental por mtime/git al
/// invocar, sin daemon salvo que duela"), así que sin esto el hook de recall
/// de M6 serviría un bloque de una KB rancia — el fallo silencioso que este
/// milestone viene a evitar.
///
/// Es `indexer::indexa` sin adornos: existe como función propia para que el
/// contrato quede nombrado y testeado por separado del CLI. Coste real: si
/// nada cambió, un `stat` por fichero y ninguna carga del modelo ONNX (el
/// embedder es perezoso, `con_embedder_de_proceso` solo se inicializa cuando
/// hay texto nuevo que embeber). Si la DB no existe, la construye —
/// bootstrap de máquina limpia.
pub fn refresca_indice(kb: &Path, db: &Path) -> Result<indexer::Resumen> {
    indexer::indexa(kb, db)
}

/// Config de embeddings leída de `~/.basic-memory/config.json` (RO, D6):
/// modelo fastembed + dims declaradas. Separada de `Embedder` porque el
/// indexer necesita `dims` (p.ej. para decidir si hay algo que embeber)
/// sin pagar la carga del modelo.
pub struct ConfigEmbeddings {
    pub modelo: String,
    pub dims: usize,
}

/// Lee `semantic_embedding_model`/`semantic_embedding_dimensions` de la
/// config RO de basic-memory (D6, precedencia flags > config la resuelve el
/// llamador).
pub fn config_embeddings() -> Result<ConfigEmbeddings> {
    let ruta = dirs::home_dir()
        .context("sin HOME")?
        .join(".basic-memory/config.json");
    let cfg: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&ruta).context("leer config bm")?)?;
    let modelo = cfg["semantic_embedding_model"]
        .as_str()
        .context("semantic_embedding_model ausente")?
        .to_string();
    let dims = cfg["semantic_embedding_dimensions"]
        .as_u64()
        .context("semantic_embedding_dimensions ausente")? as usize;
    Ok(ConfigEmbeddings { modelo, dims })
}

/// Lee `semantic_min_similarity` de la config RO de basic-memory (D6; hoy
/// 0.35). Umbral por defecto del arm vector — precedencia flags > config la
/// resuelve el llamador (`buscador::busca_vector`, flag `--min-similitud`).
pub fn min_similitud_de_config() -> Result<f64> {
    let ruta = dirs::home_dir()
        .context("sin HOME")?
        .join(".basic-memory/config.json");
    let cfg: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&ruta).context("leer config bm")?)?;
    cfg["semantic_min_similarity"]
        .as_f64()
        .context("semantic_min_similarity ausente")
}

/// Handle reutilizable de fastembed (m2-06: refactor de `embebe_frase` de
/// m2-01 para soportar embed batch de los trozos de una nota sin
/// reinicializar el modelo por cada nota — el modelo se carga UNA vez por
/// proceso, coherente con "un `exo index` sin cambios no debe pagar la carga
/// del modelo", spec M2-06 Task 2). Layout interno: clase pre-autorizada.
pub struct Embedder {
    te: fastembed::TextEmbedding,
}

impl Embedder {
    /// Inicializa fastembed con el modelo de `~/.basic-memory/config.json`
    /// (RO, D6). Mapeo string de config -> camino fastembed (verificado
    /// contra docs.rs de fastembed 5.17.3, la versión pineada en
    /// Cargo.toml):
    ///
    /// `EmbeddingModel` (fastembed 5.17.3) NO tiene variante para
    /// "jinaai/jina-embeddings-v2-base-es" — solo existen
    /// `JinaEmbeddingsV2BaseEN` y `JinaEmbeddingsV2BaseCode` (confirmado en
    /// el enum de la crate). Camino tomado: `UserDefinedEmbeddingModel`,
    /// descargando del mismo repo HF que sirve basic-memory (vía `hf-hub`
    /// 0.5.0, ya dependencia transitiva de fastembed, aquí declarada
    /// explícita para poder importarla) los 5 ficheros que ese repo
    /// publica: `onnx/model.onnx` (~0.6 GB) + `tokenizer.json` +
    /// `config.json` + `special_tokens_map.json` + `tokenizer_config.json`.
    /// Pooling explícito a `Mean`: el repo declara en
    /// `1_Pooling/config.json` `pooling_mode_mean_tokens=true`
    /// (word_embedding_dimension=768), y el ONNX exportado no trae el
    /// pooling horneado. Los embeddings salen L2-normalizados (fastembed
    /// aplica `normalize()` en el transformer por defecto de
    /// `TextEmbedding` — verificado en `common.rs`/`text_embedding/output.rs`
    /// de la crate 5.17.3): propiedad que `buscador::busca_vector` explota
    /// para convertir distancia L2² de vec0 en similitud coseno.
    pub fn desde_config() -> Result<Self> {
        let cfg = config_embeddings()?;
        Self::con_modelo(&cfg.modelo)
    }

    fn con_modelo(modelo: &str) -> Result<Self> {
        use fastembed::{
            InitOptionsUserDefined, Pooling, TextEmbedding, TokenizerFiles,
            UserDefinedEmbeddingModel,
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

        let te =
            TextEmbedding::try_new_from_user_defined(modelo_custom, InitOptionsUserDefined::new())
                .context("inicializar fastembed con modelo custom jina-es")?;
        Ok(Self { te })
    }

    /// Embebe un batch de textos en una sola pasada (usado por el indexer
    /// para los trozos de una nota; también sirve para embeber la query de
    /// `exo search --type vector` como batch de 1).
    pub fn embebe_batch(&mut self, textos: &[String]) -> Result<Vec<Vec<f32>>> {
        self.te
            .embed(textos.to_vec(), None)
            .context("embed batch con fastembed")
    }
}

/// Cache del `Embedder` a nivel de PROCESO (no por llamada): `exo
/// index`/`rebuild`/`search --type vector` son procesos de vida corta, así
/// que "el embedder se inicializa UNA vez por proceso" (Task 2, brief
/// M2-06) se implementa como este lazy-static compartido en vez de una
/// variable local por llamada — evita recargar el modelo ONNX (~0.6 GB) si
/// el mismo proceso llama a `indexa` más de una vez, y es lo que de verdad
/// impone "una vez por proceso" al pie de la letra. Efecto colateral
/// deseado: dentro del test suite del crate, N tests que indexan notas con
/// cuerpo no vacío corren en threads concurrentes del mismo binario de
/// test (mismo proceso) — sin este cache cada uno crea su propio
/// `Embedder`, cargando N copias del modelo a la vez (~0.6 GB × N),
/// verificado empíricamente que revienta la memoria (SIGKILL) en esta
/// sesión. Con el cache, la carga ocurre una sola vez y las llamadas
/// concurrentes se serializan en el Mutex.
static EMBEDDER_PROCESO: Mutex<Option<Embedder>> = Mutex::new(None);

/// Ejecuta `f` con el `Embedder` cacheado del proceso, inicializándolo
/// (carga del modelo) la primera vez que se necesita.
pub fn con_embedder_de_proceso<T>(f: impl FnOnce(&mut Embedder) -> Result<T>) -> Result<T> {
    let mut guard = EMBEDDER_PROCESO
        .lock()
        .expect("lock del embedder de proceso envenenado (panic previo en otro hilo)");
    if guard.is_none() {
        *guard = Some(Embedder::desde_config()?);
    }
    f(guard.as_mut().expect("embedder inicializado arriba"))
}

/// Lee la config RO, inicializa fastembed y devuelve (embedding de la frase
/// de prueba, dims declaradas). Conserva la firma de m2-01 para el smoke
/// test; internamente delega en `Embedder`.
pub fn embedder_desde_config() -> Result<(Vec<f32>, usize)> {
    let cfg = config_embeddings()?;
    let mut embedder = Embedder::con_modelo(&cfg.modelo)?;
    let mut out = embedder.embebe_batch(&["el exocortex recuerda por ti".to_string()])?;
    Ok((out.pop().expect("un embedding"), cfg.dims))
}
