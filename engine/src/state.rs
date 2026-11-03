//! Rutas de estado de exo y seams de entorno del hook. `resuelve` y
//! `Seams::resuelve` son puras (reciben `var`): los tests no tocan el entorno
//! del proceso; `desde_env` es el único punto que lo lee.

use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rutas {
    pub state_dir: PathBuf,
    pub reflex_log: PathBuf,
    pub rules_dir: PathBuf,
    pub index_log: PathBuf,
    pub canary: PathBuf,
}

impl Rutas {
    /// Precedencia por fichero: variable propia > `$EXO_STATE_DIR/<x>` > `<home>/.exo/state/<x>`.
    pub fn resuelve(home: &Path, var: impl Fn(&str) -> Option<OsString>) -> Rutas {
        let var = |k: &str| var(k).filter(|v| !v.is_empty()).map(PathBuf::from);
        let state_dir = var("EXO_STATE_DIR").unwrap_or_else(|| home.join(".exo/state"));
        let fichero =
            |propia: &str, nombre: &str| var(propia).unwrap_or_else(|| state_dir.join(nombre));
        Rutas {
            reflex_log: fichero("REFLEX_LOG_FILE", "reflex-log.jsonl"),
            index_log: fichero("EXO_INDEX_LOG", "exo-index.log"),
            canary: fichero("REFLEX_CANARY_FILE", "reflex-inject-canary"),
            rules_dir: state_dir.join("exo-rules"),
            state_dir,
        }
    }

    pub fn desde_env() -> Rutas {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Rutas::resuelve(&home, |k| std::env::var_os(k))
    }
}

/// Variables de entorno que sobreviven como seams, con los defaults de `exo-recall.sh`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seams {
    pub exo_bin: Option<PathBuf>,
    pub exo_index: Option<PathBuf>,
    pub exo_kb: Option<PathBuf>,
    pub exo_kb_name: Option<String>,
    pub recall_nota: Option<String>,
    pub recall_cap: usize,
    pub bloque_cap: usize,
    pub recall_limite: usize,
    pub compact_lineas: usize,
    pub git_timeout_s: u64,
    pub claude_project_dir: Option<PathBuf>,
    pub codex_home: Option<PathBuf>,
}

impl Seams {
    /// Vacío = ausente; un número que no parsea = default.
    pub fn resuelve(var: impl Fn(&str) -> Option<OsString>) -> Seams {
        let var = |k: &str| var(k).filter(|v| !v.is_empty());
        let ruta = |k: &str| var(k).map(PathBuf::from);
        let texto = |k: &str| var(k).map(|v| v.to_string_lossy().into_owned());
        fn num<T: std::str::FromStr>(v: Option<String>, def: T) -> T {
            v.and_then(|s| s.trim().parse().ok()).unwrap_or(def)
        }
        Seams {
            exo_bin: ruta("EXO_BIN"),
            exo_index: ruta("EXO_INDEX"),
            exo_kb: ruta("EXO_KB"),
            exo_kb_name: texto("EXO_KB_NAME"),
            recall_nota: texto("EXO_RECALL_NOTA"),
            recall_cap: num(texto("EXO_RECALL_CAP"), 6144),
            bloque_cap: num(texto("RECALL_BLOQUE_CAP"), 1500),
            recall_limite: num(texto("EXO_RECALL_LIMITE"), 10),
            compact_lineas: num(texto("EXO_RECALL_COMPACT_LINEAS"), 2000),
            git_timeout_s: num(texto("EXO_GIT_TIMEOUT"), 3),
            claude_project_dir: ruta("CLAUDE_PROJECT_DIR"),
            codex_home: ruta("CODEX_HOME"),
        }
    }

    pub fn desde_env() -> Seams {
        Seams::resuelve(|k| std::env::var_os(k))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Migracion {
    Renombrado,
    Fusionado { viejo_a: PathBuf },
    Nada,
    Fallo(String),
}

/// `viejo` → `nuevo` (spec §4.2). Nunca borra el viejo: o pasa a ser el nuevo o acaba en `.migrado-<ts>`.
pub fn migra(viejo: &Path, nuevo: &Path, ts: &str) -> Migracion {
    migra_con(viejo, nuevo, ts, || {})
}

/// `migra` con un gancho que corre justo antes de crear el nuevo: deja a un test
/// simular un appender que lo crea en esa ventana sin depender del reloj.
#[doc(hidden)]
pub fn migra_con(viejo: &Path, nuevo: &Path, ts: &str, antes_del_link: impl FnOnce()) -> Migracion {
    if viejo == nuevo || !viejo.exists() {
        return Migracion::Nada;
    }
    match migra_io(viejo, nuevo, ts, antes_del_link) {
        Ok(m) => m,
        Err(e) => Migracion::Fallo(format!("{}: {e}", viejo.display())),
    }
}

fn migra_io(
    viejo: &Path,
    nuevo: &Path,
    ts: &str,
    antes_del_link: impl FnOnce(),
) -> std::io::Result<Migracion> {
    use std::io::ErrorKind::{AlreadyExists, NotFound};
    if let Some(padre) = nuevo.parent() {
        std::fs::create_dir_all(padre)?;
    }
    antes_del_link();
    // `hard_link` no reemplaza un destino existente; `rename(2)` sí, y pisaría las
    // líneas de un hook que creó el nuevo entre el `stat` y el rename. Los fds que
    // el viejo tenga abiertos siguen escribiendo en el mismo inodo, ya nuevo.
    match std::fs::hard_link(viejo, nuevo) {
        Ok(()) => {
            return match std::fs::remove_file(viejo) {
                Ok(()) => Ok(Migracion::Renombrado),
                Err(e) if e.kind() == NotFound => Ok(Migracion::Renombrado),
                Err(e) => Err(e),
            };
        }
        Err(e) if e.kind() == NotFound => return Ok(Migracion::Nada),
        Err(e) if e.kind() == AlreadyExists => {}
        // FS sin hard links: cae al rename (que sí reemplaza) si el nuevo sigue sin existir.
        Err(_) if !nuevo.exists() => {
            std::fs::rename(viejo, nuevo)?;
            return Ok(Migracion::Renombrado);
        }
        Err(e) => return Err(e),
    }
    // Otro migrador acaba de enlazar viejo y nuevo al mismo inodo y está por borrar el viejo.
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let (Ok(v), Ok(n)) = (std::fs::metadata(viejo), std::fs::metadata(nuevo))
            && v.ino() == n.ino()
            && v.dev() == n.dev()
        {
            return Ok(Migracion::Nada);
        }
    }
    // Rename primero: solo un migrador gana el viejo (el otro ve ENOENT) y lo que un
    // hook escriba después recrea el viejo en vez de caer en un `.migrado` que nadie lee.
    let base = format!("{}.migrado-{ts}", viejo.display());
    let mut destino = PathBuf::from(&base);
    for n in 1.. {
        if !destino.exists() {
            break;
        }
        destino = PathBuf::from(format!("{base}-{n}"));
    }
    match std::fs::rename(viejo, &destino) {
        Ok(()) => {}
        Err(e) if e.kind() == NotFound => return Ok(Migracion::Nada),
        Err(e) => return Err(e),
    }
    let mut bytes = std::fs::read(&destino)?;
    if !bytes.is_empty() && bytes.last() != Some(&b'\n') {
        bytes.push(b'\n');
    }
    // Un solo write() con O_APPEND: no se intercala con los hooks que escriben en el nuevo.
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(nuevo)?
        .write_all(&bytes)?;
    Ok(Migracion::Fusionado { viejo_a: destino })
}

/// Los tres ficheros de estado que vivían en `~/.claude/`.
pub fn migra_todo(home: &Path, r: &Rutas, ts: &str) -> Vec<(&'static str, Migracion)> {
    [
        ("reflex-log.jsonl", &r.reflex_log),
        ("exo-index.log", &r.index_log),
        ("reflex-inject-canary", &r.canary),
    ]
    .into_iter()
    .map(|(nombre, nuevo)| (nombre, migra(&home.join(".claude").join(nombre), nuevo, ts)))
    .collect()
}
