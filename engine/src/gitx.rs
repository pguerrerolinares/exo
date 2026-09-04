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

/// ¿Es `dir` **la raíz** de un work tree de git? Es una **condición de la
/// KB**, no un fallo: `exo init --from-basic-memory` crea KBs sin versionar,
/// donde `search`, `recall` e `index` funcionan perfectamente. Distinguirla
/// de un fallo de git por fichero es lo que convierte un mensaje inútil en
/// uno accionable (A2 del plan de G4b).
///
/// **La raíz, no "dentro de algún repo".** `--is-inside-work-tree` devuelve
/// `true` para una KB anidada en un repo ajeno —un `$HOME` con los dotfiles
/// versionados—, y ahí `git log` sobre cada nota resuelve sin error y
/// devuelve cadena vacía: `last_commit` sale vacío para TODAS las notas, en
/// silencio. Comparar con `--show-toplevel` cierra esa puerta.
pub fn es_repo_git(dir: &Path) -> Result<bool> {
    let salida = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .with_context(|| format!("invocar git en {}", dir.display()))?;
    if !salida.status.success() {
        return Ok(false);
    }
    let raiz = String::from_utf8_lossy(&salida.stdout).trim().to_string();
    // Guarda defensiva sin test que la ejercite: probado el 2026-09-04 contra
    // un repo bare, que es el candidato obvio a "éxito con stdout vacío", y
    // `--show-toplevel` ahí falla con exit 128 en vez de salir vacío. No se
    // encontró ninguna forma de que git real dispare esta rama; se mantiene
    // igualmente porque parsear un stdout vacío como ruta y compararlo contra
    // `dir` sería, si algún día ocurre, un `Ok(true)` falso por accidente de
    // `canonicalize("")`.
    if raiz.is_empty() {
        return Ok(false);
    }
    // `canonicalize` en los dos lados: git devuelve la ruta con `/` y resuelve
    // symlinks, y en Windows además difiere en el prefijo de unidad.
    let (a, b) = (std::fs::canonicalize(dir), std::fs::canonicalize(&raiz));
    Ok(matches!((a, b), (Ok(a), Ok(b)) if a == b))
}

/// Fecha ISO-8601 del último commit que tocó `ruta_rel` dentro de la KB.
///
/// `ruta_rel` es un **pathspec de git**, no una ruta de disco: se normaliza a
/// `/` porque `notas.ruta` viaja con el separador nativo y en Windows llegaría
/// con `\`. La normalización es **defensiva**, no un invariante demostrado:
/// medido el 2026-09-02, Git for Windows acepta el `\` en el pathspec y sin
/// la conversión los tests siguen verdes. Se mantiene para que el pathspec sea
/// determinista frente a versiones y configuraciones de git que no tienen por
/// qué compartir esa tolerancia.
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

    // `notas.ruta` se guarda con el separador nativo (`indexer::ruta_relativa`
    // no normaliza), así que en Windows llega con `\`. Este test comprueba que
    // una ruta con separador nativo funciona de punta a punta.
    //
    // Lo que este test NO hace, y conviene decirlo en vez de dejarlo creer: no
    // falsa el `replace('\\', "/")` de `ultimo_commit`. Medido el 2026-09-02
    // en Git for Windows quitando la conversión — los 4 tests siguen verdes,
    // porque git acepta el `\` como separador en el pathspec. El plan de G4a
    // afirmaba que sin la conversión el pathspec no matchearía y `last_commit`
    // degradaría a "" en silencio; sobre esta máquina eso es **falso**.
    //
    // La conversión se queda igualmente, y no por inercia: hace el pathspec
    // determinista frente a versiones y configuraciones de git que no tienen
    // por qué compartir esa tolerancia, y cuesta una línea. Pero es una guarda
    // defensiva sin test que la ejercite, no un invariante demostrado.
    #[test]
    fn una_ruta_con_separador_nativo_encuentra_su_commit() {
        let dir = repo("log/a.md", "cuerpo\n");
        let nativa = format!("log{}a.md", std::path::MAIN_SEPARATOR);
        assert_eq!(
            ultimo_commit(dir.path(), &nativa).unwrap(),
            "2026-07-01T10:00:00+02:00"
        );
    }

    // Las tres ramas de `es_repo_git`: git falla (no es un repo en absoluto),
    // salida vacía, y toplevel distinto de la raíz (KB anidada en repo ajeno).
    #[test]
    fn es_repo_git_es_falso_fuera_de_cualquier_repo() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!es_repo_git(dir.path()).unwrap());
    }

    #[test]
    fn es_repo_git_es_verdadero_en_la_raiz_de_un_work_tree() {
        let dir = repo("log/a.md", "cuerpo\n");
        assert!(es_repo_git(dir.path()).unwrap());
    }

    #[test]
    fn es_repo_git_es_falso_en_un_subdirectorio_anidado() {
        let dir = repo("log/a.md", "cuerpo\n");
        let anidado = dir.path().join("log");
        assert!(!es_repo_git(&anidado).unwrap());
    }
}
