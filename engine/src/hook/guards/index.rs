//! Guard Index: refresca el índice de la KB al parar (Stop), migrado de exo-index.sh.

use crate::hook::evento::{Decision, Entorno, Evento};
use crate::hook::reflex::registra;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Lanza `bin index --db <db> --json` sin esperarlo y fuera del process group
/// del hook: el harness mata ese group al terminar y el indexado (de ~1,5 s a
/// minutos tras un clone) debe sobrevivir. stdout y stderr van en append a `log`.
///
/// La rama Windows (`DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP`) solo la
/// ejercita el job `test-windows`, post-merge (spec §7.3); en Linux no compila.
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
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }
    // El Child se descarta sin wait(): el hook sale enseguida y el hijo se reparenta.
    cmd.spawn().map(drop)
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
