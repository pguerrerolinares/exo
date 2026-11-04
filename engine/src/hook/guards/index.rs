//! Guard Index: refresca el índice de la KB al parar (Stop), migrado de exo-index.sh.

use crate::hook::evento::{Decision, Entorno, Evento};
use crate::hook::reflex::registra;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Lanza `bin index --db <db> --json` sin esperarlo y fuera del process group
/// del hook: el harness mata ese group al terminar y el indexado (de ~1,5 s a
/// minutos tras un clone) debe sobrevivir. stdout y stderr van en append a `log`.
///
/// En Windows el hijo se crea con `CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP`
/// (no `DETACHED_PROCESS`: sin consola, cada `git` del indexer abriría una
/// visible; con consola oculta la heredan los nietos) y con
/// `CREATE_BREAKAWAY_FROM_JOB` para salir del job del harness (libuv lo crea con
/// KILL_ON_JOB_CLOSE); si el job no permite breakaway `CreateProcess` da
/// ACCESS_DENIED y se reintenta sin el flag (el hijo funciona, pero muere con el
/// job). Además se quita la herencia a los handles std del hook: `CreateProcess`
/// va con `bInheritHandles=TRUE` y el nieto sujetaría el pipe de stdout del
/// harness, que espera su EOF.
pub fn lanza_desacoplado(bin: &Path, db: &Path, log: &Path) -> std::io::Result<()> {
    if let Some(p) = log.parent() {
        std::fs::create_dir_all(p)?;
    }
    let salida = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log)?;
    let mut cmd = Command::new(bin);
    cmd.args(["index", "--db"])
        .arg(db)
        .arg("--json")
        .stdin(Stdio::null())
        .stdout(salida.try_clone()?)
        .stderr(salida);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;
        const ERROR_ACCESS_DENIED: i32 = 5;
        no_hereda_std();
        let base = CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP;
        cmd.creation_flags(base | CREATE_BREAKAWAY_FROM_JOB);
        match cmd.spawn() {
            Err(e) if e.raw_os_error() == Some(ERROR_ACCESS_DENIED) => {
                cmd.creation_flags(base);
            }
            r => return r.map(drop),
        }
    }
    // El Child se descarta sin wait(): el hook sale enseguida y el hijo se reparenta.
    cmd.spawn().map(drop)
}

/// Best-effort: std duplica como heredable los handles que pasa con `Stdio::inherit`,
/// así que quitar el flag a los originales no afecta a spawns posteriores.
#[cfg(windows)]
fn no_hereda_std() {
    use std::ffi::c_void;
    unsafe extern "system" {
        fn GetStdHandle(n: u32) -> *mut c_void;
        fn SetHandleInformation(h: *mut c_void, mask: u32, flags: u32) -> i32;
    }
    const HANDLE_FLAG_INHERIT: u32 = 1;
    const INVALID_HANDLE_VALUE: isize = -1;
    // STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE: -10, -11, -12 como DWORD.
    for n in [-10_i32, -11, -12] {
        // SAFETY: llamadas FFI de kernel32 sin precondiciones de memoria; los handles
        // nulos o inválidos se descartan y los errores se ignoran a propósito.
        unsafe {
            let h = GetStdHandle(n as u32);
            if !h.is_null() && h as isize != INVALID_HANDLE_VALUE {
                SetHandleInformation(h, HANDLE_FLAG_INHERIT, 0);
            }
        }
    }
}

fn db_indice(en: &Entorno) -> Option<PathBuf> {
    if let Some(db) = &en.seams.exo_index {
        return Some(db.clone());
    }
    crate::config_db().ok()
}

/// Siempre `Allow`: nunca bloquea el cierre. Los fallos dejan `index-fallback`.
pub fn index(ev: &Evento, en: &Entorno) -> Decision {
    let Some(db) = db_indice(en) else {
        registra(en, "index-fallback", ev, "reason=no-db");
        return Decision::Allow;
    };
    let bin = match &en.seams.exo_bin {
        Some(b) => b.clone(),
        None => match std::env::current_exe() {
            Ok(b) => b,
            Err(e) => {
                registra(
                    en,
                    "index-fallback",
                    ev,
                    &format!("reason=detach-failed err={e}"),
                );
                return Decision::Allow;
            }
        },
    };
    if let Err(e) = lanza_desacoplado(&bin, &db, &en.rutas.index_log) {
        registra(
            en,
            "index-fallback",
            ev,
            &format!("reason=detach-failed bin={} err={e}", bin.display()),
        );
    }
    Decision::Allow
}
