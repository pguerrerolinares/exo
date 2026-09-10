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
        // Los mensajes de git están localizados: sin esto, discriminar por
        // texto de stderr fallaría en una máquina en castellano. LC_ALL/LANG
        // a C fuerza el mensaje en inglés de forma determinista.
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .output()
        .with_context(|| format!("invocar git en {}", dir.display()))?;
    if !salida.status.success() {
        let stderr = String::from_utf8_lossy(&salida.stderr);
        // "fatal: not a git repository (or any of the parent directories): .git"
        // es el ÚNICO fallo que A2 viene a cubrir: una KB sin `.git`. Cualquier
        // otro fallo de git —el relevante en la práctica: "detected dubious
        // ownership" de git >= 2.35.2— tiene que llegar al usuario con el
        // mensaje real de git, que trae el remedio
        // (`git config --global --add safe.directory ...`). Colapsar todo a
        // Ok(false) tapaba ese mensaje detrás del genérico "corre git init",
        // que para dubious ownership es un consejo activamente equivocado.
        //
        // Se mantiene la firma Result<bool>: la tercera condición ("git falló
        // por algo que no es 'no soy un repo'") se codifica como Err con
        // contexto, no como una variante nueva de un enum — Result ya tiene
        // dos casillas (Ok/Err) y la semántica existente de este módulo
        // (fail-loud, ver el comentario de módulo) es justo "fallo real de
        // git = Err". Un enum de tres estados solo añadiría un tipo que
        // ningún llamador necesita: busca_objetivos ya propaga el Err con `?`.
        if stderr.contains("not a git repository") {
            return Ok(false);
        }
        bail!(
            "git rev-parse --show-toplevel en {}: {}",
            dir.display(),
            stderr.trim()
        );
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

/// ¿Es `dir` un clon truncado (`--depth`)? Un clon shallow puede no tener el
/// objeto de un commit anterior a la trunca, y leer un sello vacío de ahí
/// haría pasar en verde cualquier subida de techo — el trinquete necesita
/// saber que no puede confiar en lo que ve.
///
/// **Asimetría deliberada, la única del módulo**: un error de git aquí es
/// `Ok(true)`, no `Err`. Es la semántica del Go (`err != nil || stdout ==
/// "true"` → abstiene): si no se puede ni preguntar si el repo es shallow,
/// la postura segura es tratarlo como si lo fuera y abstenerse aguas arriba,
/// no propagar un `Err` que en este punto del pipeline sería fail-loud sobre
/// algo que el trinquete puede simplemente no juzgar. El resto de este
/// módulo es fail-loud (ver el comentario de cabecera); esta función es la
/// excepción con nombre y razón.
pub fn es_shallow(dir: &Path) -> Result<bool> {
    let salida = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--is-shallow-repository"])
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .output();
    let Ok(salida) = salida else {
        return Ok(true);
    };
    if !salida.status.success() {
        return Ok(true);
    }
    Ok(String::from_utf8_lossy(&salida.stdout).trim() == "true")
}

/// ¿Está `dir` dentro de un work tree de git (en cualquier profundidad, no
/// solo en la raíz)? Mismo patrón fail-loud que `es_repo_git` —distinguir
/// "no es un repo" de "git falló por otra razón"— pero **sin** la
/// comparación con `--show-toplevel`: aquí sí vale "dentro de un repo",
/// porque el trinquete siempre resuelve el sello con `HEAD:./<fichero>`
/// contra el `-C` (A8/A2), y esa resolución funciona igual desde cualquier
/// subdirectorio del work tree. No hace falta que `dir` sea la raíz.
pub fn es_work_tree(dir: &Path) -> Result<bool> {
    let salida = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--is-inside-work-tree"])
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .output()
        .with_context(|| format!("invocar git en {}", dir.display()))?;
    if !salida.status.success() {
        let stderr = String::from_utf8_lossy(&salida.stderr);
        if stderr.contains("not a git repository") {
            return Ok(false);
        }
        bail!(
            "git rev-parse --is-inside-work-tree en {}: {}",
            dir.display(),
            stderr.trim()
        );
    }
    Ok(String::from_utf8_lossy(&salida.stdout).trim() == "true")
}

/// `git show <objeto>`. Devuelve `Ok(None)` cuando git sale con código ≠ 0
/// —en este módulo esa salida es señal semántica ("ese objeto no está en el
/// árbol"), no un fallo— y `Err` solo si el proceso git no se pudo ni
/// invocar. Es la **única** función de `gitx` que trata un exit ≠ 0 como
/// `Ok`: el llamador (`carga_head`) necesita distinguir "no hay sello en
/// HEAD" de "no hay HEAD", y esa distinción se hace encadenando
/// `head_resuelve`, no mirando el stderr de este comando — el stderr de un
/// `git show` que falla ("path does not exist", "bad revision", etc.) es
/// ruido de depuración, no información que el llamador tenga que parsear.
pub fn muestra(dir: &Path, objeto: &str) -> Result<Option<String>> {
    let salida = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["show", objeto])
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .output()
        .with_context(|| format!("invocar git show {objeto} en {}", dir.display()))?;
    if !salida.status.success() {
        return Ok(None);
    }
    Ok(Some(String::from_utf8_lossy(&salida.stdout).to_string()))
}

/// ¿Resuelve `HEAD` a un commit? Falso en un repo recién iniciado sin
/// commits (o fuera de un repo git). Devuelve `bool`, no `Result`: no hay
/// distinción de fallo útil más allá de sí/no para el llamador — es
/// `carga_head` quien decide, encadenando `es_work_tree` antes, si "HEAD no
/// resuelve" significa "no hay repo" o "hay repo pero sin commits todavía".
pub fn head_resuelve(dir: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--verify", "HEAD"])
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .output()
        .map(|salida| salida.status.success())
        .unwrap_or(false)
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

    // `es_shallow` necesita un origen con AL MENOS dos commits: medido el
    // 2026-09-09, un origen de un solo commit clonado con --depth 1 no deja
    // `.git/shallow` (no hay historia que truncar), y el test pasaría en
    // verde sin ejercitar nada. Se añade un segundo commit al fixture
    // exclusivamente para este test.
    #[test]
    fn es_shallow_detecta_un_clone_truncado() {
        let origen = repo("log/a.md", "primero\n");
        let raiz = origen.path();
        let cfg = raiz.join("gitconfig-vacio");
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
                .env("GIT_AUTHOR_DATE", "2026-07-01T10:01:00+02:00")
                .env("GIT_COMMITTER_DATE", "2026-07-01T10:01:00+02:00")
                .output()
                .unwrap();
            assert!(salida.status.success(), "git {args:?} falló");
        };
        std::fs::write(raiz.join("log/a.md"), "segundo\n").unwrap();
        corre(&["add", "."]);
        corre(&["commit", "-q", "-m", "segundo"]);

        // file:// con el separador nativo convertido a `/`: en Windows la
        // ruta llega con `\` y un drive letter, y git solo entiende `/` en
        // la URL.
        let url = format!("file:///{}", raiz.display().to_string().replace('\\', "/"));
        let clon = tempfile::tempdir().unwrap();
        let destino = clon.path().join("clon");
        let salida = Command::new("git")
            .args(["clone", "--depth", "1", "-q", &url])
            .arg(&destino)
            .env("GIT_CONFIG_GLOBAL", &cfg)
            .env("GIT_CONFIG_SYSTEM", &cfg)
            .output()
            .unwrap();
        if !salida.status.success() {
            // Igual que el t.Skipf del test Go: no siempre se puede clonar
            // por file:// en esta máquina (política de seguridad de git,
            // permisos). Se anota y no se falla — pero tampoco se finge
            // haber ejercitado la rama.
            eprintln!(
                "es_shallow_detecta_un_clone_truncado: clone por file:// falló, \
                 test no ejercitado en esta máquina: {}",
                String::from_utf8_lossy(&salida.stderr)
            );
            return;
        }

        assert!(es_shallow(&destino).unwrap());
        assert!(!es_shallow(raiz).unwrap());
    }

    // `es_work_tree` difiere de `es_repo_git` justo en el punto que A2/A8
    // documentan: no compara con `--show-toplevel`, así que un subdirectorio
    // dentro del repo también cuenta. Es la propiedad que necesita el
    // trinquete, que siempre resuelve el sello con `HEAD:./<fichero>` contra
    // el `-C` — no le hace falta que `dir` sea la raíz.
    #[test]
    fn es_work_tree_es_falso_fuera_de_cualquier_repo() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!es_work_tree(dir.path()).unwrap());
    }

    #[test]
    fn es_work_tree_es_verdadero_en_la_raiz_de_un_repo() {
        let dir = repo("log/a.md", "cuerpo\n");
        assert!(es_work_tree(dir.path()).unwrap());
    }

    #[test]
    fn es_work_tree_es_verdadero_en_un_subdirectorio_anidado() {
        // Contraste deliberado con `es_repo_git_es_falso_en_un_subdirectorio_anidado`:
        // ahí es `false` porque compara con `--show-toplevel`; aquí es `true`
        // porque "dentro de un repo" ya basta (A2/A8).
        let dir = repo("log/a.md", "cuerpo\n");
        let anidado = dir.path().join("log");
        assert!(es_work_tree(&anidado).unwrap());
    }

    #[test]
    fn muestra_devuelve_none_cuando_el_objeto_no_esta() {
        let dir = repo("fichero.json", "{}\n");
        assert_eq!(muestra(dir.path(), "HEAD:./no-existe.json").unwrap(), None);
    }

    #[test]
    fn muestra_devuelve_el_contenido_committeado() {
        let dir = repo("fichero.json", "original\n");
        // Modificar el working tree DESPUÉS del commit: si `muestra` leyera
        // disco en vez de HEAD, este test no falsaría nada.
        std::fs::write(dir.path().join("fichero.json"), "modificado\n").unwrap();
        assert_eq!(
            muestra(dir.path(), "HEAD:./fichero.json").unwrap(),
            Some("original\n".to_string())
        );
    }

    #[test]
    fn head_resuelve_es_falso_en_un_repo_recien_iniciado_sin_commits() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = dir.path().join("gitconfig-vacio");
        std::fs::write(&cfg, "").unwrap();
        let salida = Command::new("git")
            .arg("-C")
            .arg(dir.path())
            .args(["init", "-q"])
            .env("GIT_CONFIG_GLOBAL", &cfg)
            .env("GIT_CONFIG_SYSTEM", &cfg)
            .output()
            .unwrap();
        assert!(salida.status.success());
        assert!(!head_resuelve(dir.path()));
    }

    #[test]
    fn head_resuelve_es_verdadero_tras_un_commit() {
        let dir = repo("log/a.md", "cuerpo\n");
        assert!(head_resuelve(dir.path()));
    }
}
