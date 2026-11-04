//! Rutas de estado de exo y seams de entorno del hook. `resuelve` y
//! `Seams::resuelve` son puras (reciben `var`): los tests no tocan el entorno
//! del proceso; `desde_env` es el único punto que lo lee.

use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rutas {
    pub home: PathBuf,
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
            home: home.to_path_buf(),
            reflex_log: fichero("REFLEX_LOG_FILE", "reflex-log.jsonl"),
            index_log: fichero("EXO_INDEX_LOG", "exo-index.log"),
            canary: fichero("REFLEX_CANARY_FILE", "reflex-inject-canary"),
            rules_dir: state_dir.join("exo-rules"),
            state_dir,
        }
    }

    pub fn desde_env() -> Rutas {
        let home = crate::home_dir().unwrap_or_else(|| PathBuf::from("."));
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
    migra_con(viejo, nuevo, ts, || {}, || {})
}

/// `migra` con ganchos para tests deterministas: `antes_del_link` corre tras el reclamo y
/// antes de crear el nuevo (un appender puede crearlo ahí); `antes_del_append` corre antes
/// de anexar al nuevo (un hook viejo puede recrear el viejo ahí).
#[doc(hidden)]
pub fn migra_con(
    viejo: &Path,
    nuevo: &Path,
    ts: &str,
    antes_del_link: impl FnOnce(),
    antes_del_append: impl FnOnce(),
) -> Migracion {
    if viejo == nuevo || !viejo.exists() {
        return Migracion::Nada;
    }
    match migra_io(viejo, nuevo, ts, antes_del_link, antes_del_append) {
        Ok(m) => m,
        Err(e) => Migracion::Fallo(format!("{}: {e}", viejo.display())),
    }
}

static RECLAMOS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn migra_io(
    viejo: &Path,
    nuevo: &Path,
    ts: &str,
    antes_del_link: impl FnOnce(),
    antes_del_append: impl FnOnce(),
) -> std::io::Result<Migracion> {
    use std::io::ErrorKind::{AlreadyExists, NotFound};
    if let Some(padre) = nuevo.parent() {
        std::fs::create_dir_all(padre)?;
    }
    // Se suelta al salir de `migra_io`.
    #[cfg(windows)]
    let _exclusion = exclusion_windows(viejo)?;
    #[cfg(windows)]
    if !viejo.exists() {
        return Ok(Migracion::Nada);
    }
    // Reclamo: reserva con O_EXCL un nombre `.migrando-<pid>-<n>` y `rename` del viejo sobre
    // esa reserva vacía. El pid se reutiliza y puede haber dos pid namespaces sobre el mismo
    // home, así que un reclamo sobrante de otro (única copia de su histórico) puede ocupar el
    // nombre: `rename` reemplaza el destino, la reserva garantiza que solo pisa un fichero
    // nuestro y vacío. Solo un migrador se lleva el viejo (el otro ve ENOENT). A partir de aquí
    // nadie vuelve a tocar `viejo` por nombre, así que un hook que lo recree no pierde líneas.
    let reclamado = loop {
        let candidato = PathBuf::from(format!(
            "{}.migrando-{}-{}",
            viejo.display(),
            std::process::id(),
            RECLAMOS.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        ));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidato)
        {
            Ok(_) => break candidato,
            Err(e) if e.kind() == AlreadyExists => {}
            Err(e) => return Err(e),
        }
    };
    match std::fs::rename(viejo, &reclamado) {
        Ok(()) => {}
        Err(e) => {
            let _ = std::fs::remove_file(&reclamado);
            return if e.kind() == NotFound {
                Ok(Migracion::Nada)
            } else {
                Err(e)
            };
        }
    }
    antes_del_link();
    // `hard_link` no reemplaza un nuevo creado por un appender concurrente; `rename` sí.
    match std::fs::hard_link(&reclamado, nuevo) {
        Ok(()) => {
            return match std::fs::remove_file(&reclamado) {
                Ok(()) => Ok(Migracion::Renombrado),
                Err(e) => Err(e),
            };
        }
        Err(e) if e.kind() == AlreadyExists => {}
        // FS sin hard links: rename (que sí reemplaza) si el nuevo sigue sin existir.
        Err(_) if !nuevo.exists() => match std::fs::rename(&reclamado, nuevo) {
            Ok(()) => return Ok(Migracion::Renombrado),
            Err(e) => return Err(devuelve(&reclamado, viejo, e)),
        },
        Err(_) => {}
    }
    antes_del_append();
    let anexado = (|| -> std::io::Result<()> {
        let mut bytes = std::fs::read(&reclamado)?;
        if !bytes.is_empty() && bytes.last() != Some(&b'\n') {
            bytes.push(b'\n');
        }
        // Un solo write() con O_APPEND: no se intercala con los hooks que escriben en el nuevo.
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(nuevo)?
            .write_all(&bytes)
    })();
    if let Err(e) = anexado {
        return Err(devuelve(&reclamado, viejo, e));
    }
    let base = format!("{}.migrado-{ts}", viejo.display());
    let mut destino = PathBuf::from(&base);
    for n in 1.. {
        match mueve_sin_reemplazar(&reclamado, &destino) {
            Ok(()) => break,
            Err(e) if e.kind() == AlreadyExists => {
                destino = PathBuf::from(format!("{base}-{n}"));
            }
            Err(e) => {
                return Err(std::io::Error::new(
                    e.kind(),
                    format!("{e}; ya anexado, el viejo quedó en {}", reclamado.display()),
                ));
            }
        }
    }
    Ok(Migracion::Fusionado { viejo_a: destino })
}

/// Exclusión entre migradores en Windows. Ahí `rename(viejo, X)` no es un reclamo exclusivo:
/// abre `viejo` por ruta y renombra por handle, así que dos migradores que lo abren a la vez
/// lo renombran los dos con éxito (medido: ~95% de las carreras de dos hilos), cada uno a su
/// `.migrando-*`, y el fichero acaba bajo un solo nombre. Si ambos leen el histórico antes de
/// ese segundo rename, lo anexan dos veces. Un lock con `share_mode(0)` lo serializa: lo suelta
/// el SO si el proceso muere (sin locks huérfanos) y `DELETE_ON_CLOSE` no deja fichero.
#[cfg(windows)]
fn exclusion_windows(viejo: &Path) -> std::io::Result<std::fs::File> {
    use std::os::windows::fs::OpenOptionsExt;
    const GENERIC_WRITE_Y_DELETE: u32 = 0x4000_0000 | 0x0001_0000;
    const FILE_FLAG_DELETE_ON_CLOSE: u32 = 0x0400_0000;
    // ERROR_ACCESS_DENIED: el lock ajeno está en delete-pending; ERROR_SHARING_VIOLATION: lo tiene abierto.
    const OCUPADO: [i32; 3] = [5, 32, 33];
    let ruta = format!("{}.migrando.lock", viejo.display());
    let limite = std::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        match std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .access_mode(GENERIC_WRITE_Y_DELETE)
            .share_mode(0)
            .custom_flags(FILE_FLAG_DELETE_ON_CLOSE)
            .open(&ruta)
        {
            Ok(f) => return Ok(f),
            Err(e)
                if e.raw_os_error().is_some_and(|c| OCUPADO.contains(&c))
                    && std::time::Instant::now() < limite =>
            {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            Err(e) => return Err(e),
        }
    }
}

/// Falló antes de terminar con el nuevo: el histórico vuelve al viejo con `hard_link` (nunca
/// `rename`, que pisaría un viejo recreado) o, si no se puede, el error nombra el reclamado.
fn devuelve(reclamado: &Path, viejo: &Path, e: std::io::Error) -> std::io::Error {
    match mueve_sin_reemplazar(reclamado, viejo) {
        Ok(()) => e,
        Err(_) => std::io::Error::new(
            e.kind(),
            format!("{e}; el contenido quedó en {}", reclamado.display()),
        ),
    }
}

/// `hard_link` + `remove_file`: mueve sin reemplazar un destino existente (`AlreadyExists`).
/// En un FS sin hard links cae a `rename`, que sí reemplaza: ventana residual aceptada.
fn mueve_sin_reemplazar(origen: &Path, destino: &Path) -> std::io::Result<()> {
    use std::io::ErrorKind::AlreadyExists;
    match std::fs::hard_link(origen, destino) {
        Ok(()) => std::fs::remove_file(origen),
        Err(e) if e.kind() == AlreadyExists => Err(e),
        Err(_) if !destino.exists() => std::fs::rename(origen, destino),
        Err(e) => Err(e),
    }
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
