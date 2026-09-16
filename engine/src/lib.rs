use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Mutex, Once};

pub mod aristas;
pub mod buscador;
pub mod config;
pub mod doctor;
pub mod envelope;
pub mod escritor;
pub mod frontmatter;
pub mod gate;
pub mod gitx;
pub mod indexer;
pub mod inicia;
pub mod lint;
pub mod nota;
pub mod objetivos;
pub mod obsolescencia;
pub mod plantilla;
pub mod presupuesto;
pub mod recall;
pub mod rotacion;
pub mod schema;
pub mod trinquete;
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
        // Anotación explícita del tipo destino (clippy::missing_transmute_annotations):
        // en una línea `unsafe`, dejar que el compilador lo infiera esconde
        // exactamente el dato que hay que poder auditar.
        rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute::<
            *const (),
            unsafe extern "C" fn(
                *mut rusqlite::ffi::sqlite3,
                *mut *mut std::os::raw::c_char,
                *const rusqlite::ffi::sqlite3_api_routines,
            ) -> std::os::raw::c_int,
        >(
            sqlite_vec::sqlite3_vec_init as *const ()
        )));
    });
}

/// Conexión en memoria con sqlite-vec registrado como auto-extension.
pub fn abre_db_en_memoria() -> Result<Connection> {
    registra_vec();
    Connection::open_in_memory().context("abrir sqlite en memoria")
}

/// Conexión a un fichero de DB en disco (mismo registro de sqlite-vec que
/// `abre_db_en_memoria`). Usada tanto por los caminos de escritura (`exo
/// index`/`exo rebuild`, M2-03) como por todos los de lectura (`exo search`,
/// `exo recall`): el `bail!` de más abajo si la conversión a WAL no consigue
/// el lock es, por tanto, un fallo duro que también puede darse al leer, no
/// solo al indexar (M6-04).
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
    // journal_mode=WAL (M6-04 §2.2). Persistente en el fichero: basta con
    // fijarlo, no hay que repetirlo por conexión, pero fijarlo en cada
    // apertura es idempotente y cubre el bootstrap de una DB nueva.
    //
    // La razón no es el pre-commit de la KB (ese camino no abre la DB): es que
    // kbx mantiene cursores de lectura abiertos mientras lee ficheros y
    // shellea git por fila (`targets.go:118-146`, `doctor.go:170-190`). En
    // journal `delete` un lector así bloquea al escritor, y el busy_timeout de
    // arriba protege al lector, no al indexer. WAL deja convivir a ambos.
    //
    // `PRAGMA journal_mode` devuelve fila, así que va por query_row: un
    // `execute` fallaría con "Execute returned results".
    let modo: String = conn
        .query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))
        .context("fijar journal_mode=WAL")?;
    if modo != "wal" {
        anyhow::bail!("journal_mode quedó en {modo}, se esperaba wal");
    }
    Ok(conn)
}

/// Raíz de la KB desde `[kb] path` de la config propia (`~/.exo/config.toml`).
///
/// Antes leía `projects["kb-demo"].path` del `config.json` de basic-memory
/// (`~/.basic-memory/`): el sustituto dependía del sustituido para arrancar,
/// y era el bloqueante duro de M5b. La precedencia `flags > env > config` la
/// resuelve el llamador.
pub fn kb_desde_config() -> Result<std::path::PathBuf> {
    let cfg = config::carga()?;
    Ok(config::expande_tilde(&cfg.kb.path))
}

/// Nombre de proyecto de la KB (prefijo de permalink), EXPLÍCITO en config.
///
/// Cierra el disenso del gate M4: `write new` lo derivaba de
/// `kb.file_name()`, contra lo que decía la spec §3.1. Coincidían por suerte.
pub fn nombre_kb() -> Result<String> {
    Ok(config::carga()?.kb.name)
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

/// Config de embeddings leída de `[embeddings]` de la config propia
/// (`~/.exo/config.toml`): modelo fastembed + dims declaradas. Separada de
/// `Embedder` porque el indexer necesita `dims` (p.ej. para decidir si hay
/// algo que embeber) sin pagar la carga del modelo.
pub struct ConfigEmbeddings {
    pub modelo: String,
    pub dims: usize,
}

/// Modelo y dims de embeddings desde `[embeddings]` de la config propia.
pub fn config_embeddings() -> Result<ConfigEmbeddings> {
    let cfg = config::carga()?;
    Ok(ConfigEmbeddings {
        modelo: cfg.embeddings.model,
        dims: cfg.embeddings.dims,
    })
}

/// Umbral por defecto del arm vector desde `[embeddings] min_similarity`.
pub fn min_similitud_de_config() -> Result<f64> {
    Ok(config::carga()?.embeddings.min_similarity)
}

/// Modelo de producción y su revisión CONGELADA en HuggingFace.
///
/// `Api::model()` resuelve `main`, que es una referencia móvil: si Jina
/// re-sube `jina-embeddings-v2-base-es`, los pesos cambian, los embeddings
/// cambian con ellos y se pierde en silencio la comparabilidad del índice y
/// de la línea base de las 55 queries de `evals/retrieval-fase0/`. El sha es
/// el del snapshot que generó esos números (cache local del 2026-07-17,
/// `refs/main` en esa fecha). Cambiarlo obliga a re-correr el eval, igual que
/// cambiar de modelo — por eso va pineado exacto, como `sqlite-vec = "=0.1.9"`.
pub const MODELO_JINA_ES: &str = "jinaai/jina-embeddings-v2-base-es";
const REVISION_JINA_ES: &str = "8e2d780d8fd38f81ca9123ee28e4c5a968aaf21e";

/// Ruta de caché de `hf_hub` para el proceso actual: `$HF_HOME/hub` si la
/// variable está definida, si no `~/.cache/huggingface/hub` — exactamente lo
/// que resuelve `hf_hub::Cache::from_env()` (hf-hub 0.5.0), delegado aquí en
/// vez de reimplementado.
///
/// Fuente ÚNICA de esa ruta para el crate: `Embedder::con_modelo` construye
/// su cliente hf-hub con `ApiBuilder::from_env()` (misma llamada, mismo
/// resultado) y `doctor::Entorno::del_proceso` llama a esta función para su
/// campo `cache_hf`. Antes de la Task 13 (G, 2026-09-16) el descargador
/// ignoraba `HF_HOME` (`Api::new()` → `Cache::default()`) mientras `doctor`
/// reimplementaba el cálculo a mano asumiendo que sí lo respetaba: dos
/// caminos que podían divergir en silencio. **No dupliques este cálculo en
/// otro sitio** — si `doctor` o un futuro llamador necesitan la ruta de
/// caché, que llamen a esta función.
pub fn cache_hf_del_entorno() -> std::path::PathBuf {
    hf_hub::Cache::from_env().path().clone()
}

/// Repo de HF para un modelo, con revisión fija si es uno de los nuestros.
/// Un modelo ajeno sigue resolviendo `main` (no rompemos a quien cambie el
/// modelo en su config), pero `con_modelo` lo dice por stderr en vez de
/// dejar creer que está congelado.
fn repo_hf(modelo: &str) -> hf_hub::Repo {
    use hf_hub::{Repo, RepoType};
    match modelo {
        MODELO_JINA_ES => Repo::with_revision(
            modelo.to_string(),
            RepoType::Model,
            REVISION_JINA_ES.to_string(),
        ),
        _ => Repo::model(modelo.to_string()),
    }
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
    /// Inicializa fastembed con el modelo de `[embeddings] model` de
    /// `~/.exo/config.toml`. Mapeo string de config -> camino fastembed (verificado
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
        use hf_hub::api::sync::ApiBuilder;

        let repo_id = repo_hf(modelo);
        if repo_id.revision() == "main" {
            eprintln!(
                "aviso: modelo `{modelo}` sin revisión pineada. Si su repo de HuggingFace \
                 se re-sube, los embeddings cambian en silencio y el índice deja de ser \
                 comparable con la línea base del eval."
            );
        }
        // `from_env()` (no `Api::new()`, que cae a `Cache::default()` y ROMPE
        // `HF_HOME`): Task 13, ver doc de `cache_hf_del_entorno` — misma ruta
        // que usa `doctor::Entorno::del_proceso` para el check `embeddings_model`.
        let repo = ApiBuilder::from_env()
            .build()
            .context("crear cliente hf-hub")?
            .repo(repo_id);
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
    /// `exo search --type vector` como batch de 1). Cada vector se valida
    /// (Ola 1 G Task 8, `KB-exo:16`) antes de devolverse — la única guarda
    /// existente estaba en la LECTURA (`vectores::lee`, `BYTES_ESPERADOS`),
    /// nunca en la escritura: un embedding corrupto entraba al índice sin
    /// avisar y solo se notaba después, en un KNN con resultados raros.
    pub fn embebe_batch(&mut self, textos: &[String]) -> Result<Vec<Vec<f32>>> {
        let vectores = self
            .te
            .embed(textos, None)
            .context("embed batch con fastembed")?;
        let total = vectores.len();
        for (i, v) in vectores.iter().enumerate() {
            verifica_embedding(v, i, total)?;
        }
        Ok(vectores)
    }
}

/// Dimensión y norma esperadas de un embedding de jina-es: 768 componentes
/// (mismo contrato que `vectores::BYTES_ESPERADOS = 768 * 4`), norma ~1.0
/// porque fastembed normaliza siempre (ver doc de `Embedder::desde_config`).
///
/// Tolerancia `0.99..=1.01` medida contra el modelo real (cache local,
/// jina-embeddings-v2-base-es, sha pineado `REVISION_JINA_ES`) el
/// 2026-09-15, sobre los casos de riesgo — texto normal, `""`, texto muy
/// largo (~5000 palabras, se trunca), solo espacios, y solo emoji/unicode
/// (`🦀🔥✨` + `漢字`): las 5 normas cayeron en `[0.999999702, 1.000000238]`,
/// ruido de redondeo de f32 (~3e-7) alrededor de 1.0, ninguna NaN/Inf. El
/// margen `±0.01` es ~30000x ese ruido — sin falsos positivos en salida
/// legítima, pero sigue atrapando una corrupción real (vector cero, NaN
/// propagado a 0.0, o un modelo futuro que no normalice).
///
/// Zona de la futura campaña J (cuantización int8 de jina): si J cambia el
/// tipo de salida de `embebe_batch` a algo que no sea `f32` unitario, este
/// assert (dimensión Y norma ~1.0) hay que revisarlo entero, no solo la
/// tolerancia.
///
/// Falla ALTO en vez de dejar pasar un vector corrupto al índice.
fn verifica_embedding(v: &[f32], idx: usize, total: usize) -> Result<()> {
    const DIMS_ESPERADAS: usize = 768;
    if v.len() != DIMS_ESPERADAS {
        anyhow::bail!(
            "embedding {idx}/{total} con {} dims, se esperaban {DIMS_ESPERADAS}",
            v.len()
        );
    }
    let norma: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if !(0.99..=1.01).contains(&norma) {
        anyhow::bail!(
            "embedding {idx}/{total} con norma {norma:.4}, se esperaba ~1.0 \
             (fastembed normaliza siempre: el modelo no está normalizando o produjo NaN/Inf)"
        );
    }
    Ok(())
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

#[cfg(test)]
mod tests_verifica_embedding {
    use super::verifica_embedding;

    #[test]
    fn rechaza_dimension_equivocada() {
        let v = vec![0.5_f32; 100];
        let err = verifica_embedding(&v, 0, 1).unwrap_err();
        assert!(err.to_string().contains("100 dims"), "{err}");
    }

    #[test]
    fn rechaza_norma_fuera_de_rango() {
        let mut v = vec![0.0_f32; 768];
        v[0] = 5.0; // norma 5.0, muy lejos de 1.0
        let err = verifica_embedding(&v, 0, 1).unwrap_err();
        assert!(err.to_string().contains("norma"), "{err}");
    }

    #[test]
    fn acepta_un_vector_unitario_real() {
        let mut v = vec![0.0_f32; 768];
        v[0] = 1.0; // norma exacta 1.0
        assert!(verifica_embedding(&v, 0, 1).is_ok());
    }
}

#[cfg(test)]
mod tests_revision_hf {
    use super::repo_hf;

    /// El modelo de producción tiene que llegar a HF con revisión FIJA: si
    /// `jinaai/jina-embeddings-v2-base-es` se re-sube, `main` serviría pesos
    /// distintos, los embeddings cambiarían en silencio y con ellos el índice
    /// **y la línea base de las 55 queries del eval**.
    #[test]
    fn el_modelo_de_produccion_va_pineado_a_un_sha() {
        let repo = repo_hf("jinaai/jina-embeddings-v2-base-es");
        let rev = repo.revision();
        assert_ne!(rev, "main", "el modelo de producción no puede ir a `main`");
        assert_eq!(rev.len(), 40, "la revisión debe ser un sha completo: {rev}");
        assert!(
            rev.chars().all(|c| c.is_ascii_hexdigit()),
            "la revisión debe ser hex: {rev}"
        );
    }

    /// Un modelo que nadie ha pineado sigue funcionando (no rompemos a quien
    /// cambie el modelo en su config); lo que no puede es fingir que está
    /// congelado.
    #[test]
    fn un_modelo_sin_pin_cae_a_main() {
        assert_eq!(repo_hf("otro/modelo-cualquiera").revision(), "main");
    }
}
