//! Preflight de **entorno**: `lint` juzga la KB, `doctor` juzga la máquina.
//!
//! Contrato, y es el punto entero del comando: **cada check reporta el
//! artefacto que miró** —la ruta, la versión, los bytes— y **ninguno
//! desaparece del informe**. Lo que no aplica a esta plataforma sale como
//! `na` con lo que miró, porque una fila ausente no se distingue de un check
//! que nunca existió. Es la lección literal de los seis casos del runbook de
//! W11 (`docs/superpowers/runbooks/2026-08-24-integracion-equipo-trabajo-windows.md`):
//! «el código de salida no es evidencia; lo que valió fue mirar el artefacto
//! real».
use serde::Serialize;
use std::path::PathBuf;

/// Estado de un check. `Na` NO es `Ok`: es «aquí esto no se mide».
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Estado {
    Ok,
    Warn,
    Fail,
    Na,
}

impl std::fmt::Display for Estado {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Estado::Ok => "ok",
            Estado::Warn => "warn",
            Estado::Fail => "fail",
            Estado::Na => "na",
        })
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Check {
    pub id: &'static str,
    #[serde(rename = "status")]
    pub estado: Estado,
    /// Lo que se miró: ruta absoluta, comando resuelto, versión, tamaño.
    #[serde(rename = "artifact")]
    pub artefacto: String,
    #[serde(rename = "detail")]
    pub detalle: String,
}

impl Check {
    fn nuevo(
        id: &'static str,
        estado: Estado,
        artefacto: impl Into<String>,
        detalle: impl Into<String>,
    ) -> Self {
        Self {
            id,
            estado,
            artefacto: artefacto.into(),
            detalle: detalle.into(),
        }
    }
}

#[derive(Serialize)]
pub struct InformeDoctor {
    pub ok: bool,
    #[serde(rename = "platform")]
    pub plataforma: &'static str,
    pub checks: Vec<Check>,
}

impl InformeDoctor {
    fn nuevo(checks: Vec<Check>) -> Self {
        // `warn` informa y NO gatea: una DB rancia o el modelo sin cachear son
        // deuda con arreglo conocido, no una máquina rota.
        let ok = !checks.iter().any(|c| c.estado == Estado::Fail);
        Self {
            ok,
            plataforma: std::env::consts::OS,
            checks,
        }
    }

    pub fn fallidos(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| c.estado == Estado::Fail)
            .count()
    }
}

/// El entorno que doctor juzga, inyectable entero: un test no puede mover el
/// HOME ni el PATH del proceso, y sin inyección estos checks solo se podrían
/// probar en la máquina del que los escribió.
pub struct Entorno {
    pub home: PathBuf,
    pub config: PathBuf,
    pub cache_hf: PathBuf,
    pub path: String,
}

impl Entorno {
    /// El entorno real del proceso. La caché replica lo que hace `hf_hub`
    /// (`Cache::from_env`): `$HF_HOME/hub`, y si no `~/.cache/huggingface/hub`.
    pub fn del_proceso() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let cache_hf = match std::env::var_os("HF_HOME") {
            Some(h) => PathBuf::from(h).join("hub"),
            None => home.join(".cache").join("huggingface").join("hub"),
        };
        Self {
            config: crate::config::ruta_config()
                .unwrap_or_else(|_| home.join(".exo").join("config.toml")),
            home,
            cache_hf,
            path: std::env::var("PATH").unwrap_or_default(),
        }
    }
}

pub fn analiza(entorno: &Entorno) -> InformeDoctor {
    // La config se carga UNA vez y se pasa a los checks que dependen de ella:
    // releerla por check daría informes internamente incoherentes si alguien
    // la edita a mitad de corrida.
    let cfg = crate::config::carga_desde(&entorno.config).ok();
    InformeDoctor::nuevo(vec![
        check_config(entorno),
        check_binario_en_path(entorno),
        check_fallback_del_hook(entorno),
        check_kb(cfg.as_ref()),
        check_indice(cfg.as_ref()),
        check_modelo(entorno, cfg.as_ref()),
    ])
}

/// Primera coincidencia de `nombre` (o `nombre.exe` en Windows) en un PATH
/// dado. No usa la crate `which`: el PATH es inyectable a propósito, y una
/// dependencia nueva para quince líneas no se paga.
fn busca_en_path(path: &str, nombre: &str) -> Option<PathBuf> {
    let candidatos: Vec<String> = if cfg!(windows) {
        vec![format!("{nombre}.exe"), nombre.to_string()]
    } else {
        vec![nombre.to_string()]
    };
    // `split_paths("")` devuelve UN componente vacío, no cero, y `"".join(x)`
    // es una ruta relativa: sin este filtro, un PATH vacío hace que el check
    // busque en el **cwd del proceso**. Medido el 2026-09-10: plantando un
    // `exo.exe` en `engine/`, el test del PATH vacío pasa de verde a rojo — es
    // decir, estaba midiendo el directorio actual, no el PATH.
    for dir in std::env::split_paths(path).filter(|d| !d.as_os_str().is_empty()) {
        for c in &candidatos {
            let ruta = dir.join(c);
            if ruta.is_file() {
                return Some(ruta);
            }
        }
    }
    None
}

fn check_binario_en_path(entorno: &Entorno) -> Check {
    match busca_en_path(&entorno.path, "exo") {
        Some(ruta) => Check::nuevo(
            "binary_on_path",
            Estado::Ok,
            ruta.display().to_string(),
            "los hooks lo resuelven con `command -v exo`",
        ),
        None => Check::nuevo(
            "binary_on_path",
            Estado::Warn,
            format!("PATH={}", entorno.path),
            "`command -v exo` no lo encuentra; los hooks caerán al fallback \
             $HOME/.local/bin/exo",
        ),
    }
}

/// El fallback literal de `plugins/exo/scripts/kb-precommit.sh:18`
/// (`EXO="${EXO_BIN:-$HOME/.local/bin/exo}"`). Si ese fichero no está, la
/// línea 20 del hook sale **0**: commit permitido, gate apagado, sin romper
/// nada. Por eso esto es `fail` y no `warn`.
///
/// Se reporta QUÉ fichero existe: medido el 2026-09-10 en el Git Bash de W11,
/// msys resuelve `exo` → `exo.exe` en `stat()` y el test `-x` sobre la ruta
/// sin extensión da verdadero, al revés de lo que afirma la spec. Un check
/// que diera por buena cualquiera de las dos versiones estaría adivinando;
/// este mira.
fn check_fallback_del_hook(entorno: &Entorno) -> Check {
    let base = entorno.home.join(".local").join("bin");
    let literal = base.join("exo");
    let con_exe = base.join("exo.exe");
    let existe_literal = literal.is_file();
    let existe_exe = con_exe.is_file();
    let artefacto = format!(
        "{} (existe={}) · {} (existe={})",
        literal.display(),
        existe_literal,
        con_exe.display(),
        existe_exe
    );
    if existe_literal || existe_exe {
        Check::nuevo(
            "hook_fallback_binary",
            Estado::Ok,
            artefacto,
            "kb-precommit.sh encuentra el binario y el gate de la KB muerde",
        )
    } else {
        Check::nuevo(
            "hook_fallback_binary",
            Estado::Fail,
            artefacto,
            "kb-precommit.sh:20 sale 0 sin gate — COMMIT PERMITIDO en silencio. \
             Instala con install.sh/install.ps1 o copia el binario ahí",
        )
    }
}

fn check_kb(cfg: Option<&crate::config::Config>) -> Check {
    let Some(cfg) = cfg else {
        return Check::nuevo(
            "kb_readable",
            Estado::Fail,
            "(sin config)",
            "no hay config legible, así que no se sabe qué KB mirar",
        );
    };
    let kb = crate::config::expande_tilde(&cfg.kb.path);
    let artefacto = kb.display().to_string();
    if !kb.is_dir() {
        return Check::nuevo(
            "kb_readable",
            Estado::Fail,
            artefacto,
            "la raíz de la KB no existe o no es un directorio",
        );
    }
    match crate::walker::walk_kb(&kb) {
        Ok(notas) => Check::nuevo(
            "kb_readable",
            Estado::Ok,
            artefacto,
            format!("{} nota(s) .md bajo la raíz", notas.len()),
        ),
        Err(e) => Check::nuevo("kb_readable", Estado::Fail, artefacto, format!("{e:#}")),
    }
}

fn check_indice(cfg: Option<&crate::config::Config>) -> Check {
    let Some(cfg) = cfg else {
        return Check::nuevo(
            "index_db",
            Estado::Fail,
            "(sin config)",
            "no hay config legible, así que no se sabe qué DB mirar",
        );
    };
    let db = crate::config::expande_tilde(&cfg.index.db);
    if !db.is_file() {
        return Check::nuevo(
            "index_db",
            Estado::Warn,
            db.display().to_string(),
            "no hay índice todavía — córrelo con `exo index`",
        );
    }
    let bytes = std::fs::metadata(&db).map(|m| m.len()).unwrap_or(0);
    let artefacto = format!("{} ({bytes} bytes)", db.display());
    let notas: i64 = match crate::abre_db(&db)
        .and_then(|c| Ok(c.query_row("SELECT count(*) FROM notas", [], |r| r.get(0))?))
    {
        Ok(n) => n,
        Err(e) => return Check::nuevo("index_db", Estado::Fail, artefacto, format!("{e:#}")),
    };
    // «No rancia»: la DB es al menos tan nueva como la nota más reciente.
    // Heurística de mtime, la misma que usa el indexer incremental; no
    // pretende detectar un borrado, sino el caso medido en W11 —el hook de
    // reindexado muerto durante meses sin un solo rastro—.
    let kb = crate::config::expande_tilde(&cfg.kb.path);
    if let Some(nota) = mtime_mas_reciente(&kb)
        && let Ok(indice) = std::fs::metadata(&db).and_then(|m| m.modified())
        && nota > indice
    {
        return Check::nuevo(
            "index_db",
            Estado::Warn,
            artefacto,
            format!(
                "{notas} nota(s) indexadas, pero la KB tiene cambios más nuevos \
                 que el índice — corre `exo index`"
            ),
        );
    }
    if notas == 0 {
        Check::nuevo(
            "index_db",
            Estado::Warn,
            artefacto,
            "el índice existe pero está vacío — corre `exo index`",
        )
    } else {
        Check::nuevo(
            "index_db",
            Estado::Ok,
            artefacto,
            format!("{notas} nota(s) indexadas"),
        )
    }
}

/// El mtime de la nota más reciente de la KB. `None` si la KB no se puede
/// recorrer: la ranciedad no se puede afirmar, y afirmarla a ciegas sería
/// justo el tipo de veredicto sin artefacto que este comando evita.
fn mtime_mas_reciente(kb: &std::path::Path) -> Option<std::time::SystemTime> {
    let notas = crate::walker::walk_kb(kb).ok()?;
    notas
        .iter()
        .filter_map(|n| std::fs::metadata(n).ok())
        .filter_map(|m| m.modified().ok())
        .max()
}

/// El `model.onnx` bajo cualquier snapshot del modelo. `metadata` sigue el
/// symlink —en la caché de HF los ficheros del snapshot apuntan a `blobs/`—,
/// así que la longitud es la real, no la del enlace.
fn onnx_en_cache(dir_modelo: &std::path::Path) -> Option<(PathBuf, u64)> {
    let entradas = std::fs::read_dir(dir_modelo.join("snapshots")).ok()?;
    for e in entradas.flatten() {
        let cand = e.path().join("onnx").join("model.onnx");
        if let Ok(m) = std::fs::metadata(&cand)
            && m.len() > 0
        {
            return Some((cand, m.len()));
        }
    }
    None
}

/// Presencia del modelo de embeddings en la caché de `hf_hub`. `warn`, no
/// `fail`: sin él la primera indexación baja 615 MB y tarda unos minutos —es
/// deuda de tiempo, no una máquina rota—, pero decirlo AQUÍ es la diferencia
/// entre un `exo index` que parece colgado y uno que se sabe descargando.
fn check_modelo(entorno: &Entorno, cfg: Option<&crate::config::Config>) -> Check {
    let modelo = cfg
        .map(|c| c.embeddings.model.as_str())
        .unwrap_or(crate::MODELO_JINA_ES);
    let dir = entorno
        .cache_hf
        .join(format!("models--{}", modelo.replace('/', "--")));
    match onnx_en_cache(&dir) {
        Some((ruta, bytes)) => Check::nuevo(
            "embeddings_model",
            Estado::Ok,
            ruta.display().to_string(),
            format!("{modelo} · {bytes} bytes en caché"),
        ),
        None => Check::nuevo(
            "embeddings_model",
            Estado::Warn,
            dir.display().to_string(),
            format!(
                "{modelo} no está en la caché: la primera indexación se baja \
                 ~615 MB (unos minutos en frío)"
            ),
        ),
    }
}

fn check_config(entorno: &Entorno) -> Check {
    let ruta = entorno.config.display().to_string();
    match crate::config::carga_desde(&entorno.config) {
        Ok(c) => Check::nuevo(
            "config",
            Estado::Ok,
            ruta,
            format!(
                "schema_version={} kb={} db={}",
                c.schema_version,
                c.kb.path.display(),
                c.index.db.display()
            ),
        ),
        Err(e) => Check::nuevo("config", Estado::Fail, ruta, format!("{e:#}")),
    }
}
