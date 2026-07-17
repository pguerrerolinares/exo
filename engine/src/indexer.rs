use crate::abre_db;
use crate::nota::parsea_nota;
use crate::schema::crea_schema;
use crate::walker::walk_kb;
use anyhow::{Context, Result};
use rusqlite::params;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;
use std::time::UNIX_EPOCH;

/// Recencia de `ruta_rel` (relativa a `kb`) según el último commit de git
/// que la tocó — JAMÁS mtime (§6.2 regla 2: ningún `created:` en frontmatter,
/// un clone fresco resetea mtimes). `None` si el fichero no tiene commits
/// (nuevo, aún no versionado) o si `git` falla por cualquier motivo — la
/// columna `notas.git_epoch` admite NULL para este caso, no es un error de
/// indexado.
pub fn git_epoch_de(kb: &Path, ruta_rel: &Path) -> Option<i64> {
    let salida = Command::new("git")
        .arg("-C")
        .arg(kb)
        .arg("log")
        .arg("-1")
        .arg("--format=%ct")
        .arg("--")
        .arg(ruta_rel)
        .output()
        .ok()?;

    if !salida.status.success() {
        return None;
    }

    let texto = String::from_utf8(salida.stdout).ok()?;
    let texto = texto.trim();
    if texto.is_empty() {
        return None;
    }
    texto.parse::<i64>().ok()
}

/// Resultado de una corrida de `indexa` (base del `data` del envelope de
/// `index`/`rebuild`, spec §4).
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Resumen {
    pub indexadas: usize,
    pub saltadas: usize,
    pub borradas: usize,
}

/// Pipeline de `exo index`/`exo rebuild` (spec §3): walk de `kb` → por nota,
/// skip si `mtime` no cambió o reparse+reindex completo si cambió → borrado
/// de las notas cuya `ruta` ya no aparece en el walk. `git_epoch` se refresca
/// en cada reindex (regla 2: la recencia consumida por ranking/recall es
/// `git_epoch`, `mtime` es SOLO detección de cambio).
///
/// `aristas`/`trozos`/`vectores` se crean por `crea_schema` pero NO se
/// pueblan aquí — llegan en M2-04 (aristas) y M2-06 (chunks+vectores).
pub fn indexa(kb: &Path, db_ruta: &Path) -> Result<Resumen> {
    let conn = abre_db(db_ruta)?;
    crea_schema(&conn)?;

    let rutas_absolutas = walk_kb(kb)?;

    let existentes: HashMap<String, f64> = {
        let mut stmt = conn.prepare("SELECT ruta, mtime FROM notas")?;
        stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?)))?
            .collect::<rusqlite::Result<_>>()?
    };

    let mut indexadas = 0usize;
    let mut saltadas = 0usize;
    let mut vistas: HashSet<String> = HashSet::new();

    for ruta_abs in &rutas_absolutas {
        let ruta_rel = ruta_relativa(kb, ruta_abs)?;
        vistas.insert(ruta_rel.clone());

        let mtime = mtime_de(ruta_abs)?;
        if existentes.get(&ruta_rel) == Some(&mtime) {
            saltadas += 1;
            continue;
        }

        let Some(n) = parsea_nota(ruta_abs)? else {
            eprintln!(
                "aviso: {} sin permalink en frontmatter, se salta (§6.2 regla 1)",
                ruta_abs.display()
            );
            continue;
        };

        let git_epoch = git_epoch_de(kb, Path::new(&ruta_rel));

        conn.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(permalink) DO UPDATE SET
               ruta = excluded.ruta, titulo = excluded.titulo, tipo = excluded.tipo,
               mtime = excluded.mtime, git_epoch = excluded.git_epoch",
            params![n.permalink, ruta_rel, n.titulo, n.tipo, mtime, git_epoch],
        )
        .with_context(|| format!("upsert de notas para {}", n.permalink))?;

        conn.execute(
            "DELETE FROM notas_fts WHERE permalink = ?1",
            params![n.permalink],
        )
        .with_context(|| format!("limpiar notas_fts previo de {}", n.permalink))?;
        conn.execute(
            "INSERT INTO notas_fts (titulo, cuerpo, permalink) VALUES (?1, ?2, ?3)",
            params![n.titulo, n.cuerpo, n.permalink],
        )
        .with_context(|| format!("insertar notas_fts de {}", n.permalink))?;

        indexadas += 1;
    }

    let mut borradas = 0usize;
    for ruta_rel in existentes.keys().filter(|r| !vistas.contains(*r)) {
        let permalinks: Vec<String> = {
            let mut stmt = conn.prepare("SELECT permalink FROM notas WHERE ruta = ?1")?;
            stmt.query_map(params![ruta_rel], |r| r.get(0))?
                .collect::<rusqlite::Result<_>>()?
        };
        for permalink in permalinks {
            conn.execute(
                "DELETE FROM notas_fts WHERE permalink = ?1",
                params![permalink],
            )?;
            conn.execute("DELETE FROM aristas WHERE origen = ?1", params![permalink])?;
            conn.execute("DELETE FROM trozos WHERE permalink = ?1", params![permalink])?;
            conn.execute("DELETE FROM notas WHERE permalink = ?1", params![permalink])?;
            borradas += 1;
        }
    }

    Ok(Resumen {
        indexadas,
        saltadas,
        borradas,
    })
}

fn ruta_relativa(kb: &Path, ruta_abs: &Path) -> Result<String> {
    Ok(ruta_abs
        .strip_prefix(kb)
        .with_context(|| format!("{} no está bajo la raíz {}", ruta_abs.display(), kb.display()))?
        .to_string_lossy()
        .into_owned())
}

fn mtime_de(ruta: &Path) -> Result<f64> {
    let meta =
        std::fs::metadata(ruta).with_context(|| format!("metadata de {}", ruta.display()))?;
    let modificado = meta
        .modified()
        .with_context(|| format!("mtime de {}", ruta.display()))?;
    let duracion = modificado
        .duration_since(UNIX_EPOCH)
        .context("mtime anterior a unix epoch")?;
    Ok(duracion.as_secs_f64())
}
