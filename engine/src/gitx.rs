//! El idioma git **fail-loud** de los comandos portados de kbx.
//!
//! No confundir con `indexer::git_epoch_de`, que es fail-silent y devuelve
//! `Option`: allí, una nota sin `git_epoch` no es un error de indexado y
//! tragarse el fallo es lo correcto. Aquí no. `kbx targets` documenta
//! explícitamente que `last_commit` "never degrades silently to ''", y un port
//! que reutilizara el idioma de casa convertiría un git roto en un campo vacío
//! plausible. Son dos funciones distintas a propósito, y esta es la razón.

use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

/// Fecha ISO-8601 del último commit que tocó `ruta_rel` dentro de la KB.
///
/// `ruta_rel` es un **pathspec de git**, no una ruta de disco: se normaliza a
/// `/` porque `notas.ruta` viaja con el separador nativo y en Windows llegaría
/// con `\`, que git no matchea (saldría 0 con stdout vacío y el campo
/// degradaría a "" sin que nadie se entere).
///
/// - stdout vacío con exit 0 ⇒ `Ok("")`: el fichero existe pero no tiene
///   commits. Es un caso legítimo, no un fallo.
/// - exit no-cero ⇒ `Err` con el stderr de git. Nunca `Ok("")`.
pub fn ultimo_commit(kb: &Path, ruta_rel: &str) -> Result<String> {
    let pathspec = ruta_rel.replace('\\', "/");
    let salida = Command::new("git")
        .arg("-C")
        .arg(kb)
        .args(["log", "-1", "--format=%aI", "--"])
        .arg(&pathspec)
        .output()
        .with_context(|| format!("invocar git log para {pathspec}"))?;

    if !salida.status.success() {
        bail!(
            "git log -- {pathspec}: {}",
            String::from_utf8_lossy(&salida.stderr).trim()
        );
    }

    Ok(String::from_utf8(salida.stdout)
        .with_context(|| format!("git log de {pathspec} devolvió stdout no-UTF8"))?
        .trim()
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    /// Repo git real en un tempdir, aislado de la config del desarrollador.
    ///
    /// `GIT_CONFIG_GLOBAL` apunta a un fichero vacío real y no a `/dev/null`:
    /// en Windows esa ruta no vale para esta variable, y los tests de kbx que
    /// la usan no son portables. Un fichero vacío en el tempdir sí lo es.
    fn repo(nombre_fichero: &str, contenido: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        let cfg = raiz.join("gitconfig-vacio");
        std::fs::write(&cfg, "").unwrap();
        let corre = |args: &[&str]| {
            let salida = Command::new("git")
                .arg("-C")
                .arg(raiz)
                .args(args)
                .env("GIT_CONFIG_GLOBAL", &cfg)
                .env("GIT_CONFIG_SYSTEM", &cfg)
                .env("GIT_AUTHOR_NAME", "f")
                .env("GIT_AUTHOR_EMAIL", "f@k.local")
                .env("GIT_COMMITTER_NAME", "f")
                .env("GIT_COMMITTER_EMAIL", "f@k.local")
                .env("GIT_AUTHOR_DATE", "2026-07-01T10:00:00+02:00")
                .env("GIT_COMMITTER_DATE", "2026-07-01T10:00:00+02:00")
                .output()
                .unwrap();
            assert!(salida.status.success(), "git {args:?} falló");
        };
        corre(&["init", "-q"]);
        std::fs::create_dir_all(raiz.join("log")).unwrap();
        std::fs::write(raiz.join(nombre_fichero), contenido).unwrap();
        corre(&["add", "."]);
        corre(&["commit", "-q", "-m", "inicial"]);
        dir
    }

    #[test]
    fn devuelve_la_fecha_iso_del_ultimo_commit() {
        let dir = repo("log/a.md", "cuerpo\n");
        let fecha = ultimo_commit(dir.path(), "log/a.md").unwrap();
        assert_eq!(fecha, "2026-07-01T10:00:00+02:00");
    }

    // Un fichero sin commits no es un error: git sale 0 con stdout vacío.
    #[test]
    fn fichero_sin_commits_da_cadena_vacia_sin_error() {
        let dir = repo("log/a.md", "cuerpo\n");
        std::fs::write(dir.path().join("log/b.md"), "nuevo\n").unwrap();
        assert_eq!(ultimo_commit(dir.path(), "log/b.md").unwrap(), "");
    }

    // El contraste con indexer::git_epoch_de, que devolvería None y seguiría.
    // Aquí un directorio que no es repo git tiene que GRITAR.
    #[test]
    fn fuera_de_un_repo_git_es_error_no_cadena_vacia() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.md"), "x\n").unwrap();
        assert!(ultimo_commit(dir.path(), "a.md").is_err());
    }

    // notas.ruta se guarda con el separador nativo (indexer::ruta_relativa no
    // normaliza), así que en Windows llega con `\`. git quiere un pathspec con
    // `/`: sin la conversión, el pathspec no matchea, git sale 0 con stdout
    // vacío y last_commit degrada a "" en silencio — justo en la función que
    // existe para no degradar en silencio.
    #[test]
    fn normaliza_el_separador_nativo_a_pathspec_de_git() {
        let dir = repo("log/a.md", "cuerpo\n");
        let nativa = format!("log{}a.md", std::path::MAIN_SEPARATOR);
        assert_eq!(
            ultimo_commit(dir.path(), &nativa).unwrap(),
            "2026-07-01T10:00:00+02:00"
        );
    }
}
