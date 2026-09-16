//! El idioma git **fail-loud** de los comandos portados de kbx.
//!
//! No confundir con `indexer::git_epoch_de`, que es fail-silent y devuelve
//! `Option`: allí, una nota sin `git_epoch` no es un error de indexado y
//! tragarse el fallo es lo correcto. Aquí no. `kbx targets` documenta
//! explícitamente que `last_commit` "never degrades silently to ''", y un port
//! que reutilizara el idioma de casa convertiría un git roto en un campo vacío
//! plausible. Son dos funciones distintas a propósito, y esta es la razón.

use anyhow::{Context, Result, bail};
use std::collections::HashMap;
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

/// `git show <objeto>`, en bytes crudos. Mismo contrato que `muestra`
/// (`Ok(None)` si el objeto no está, `Err` solo si git no se pudo invocar),
/// pero sin pasar por `from_utf8_lossy`.
///
/// Existe porque `muestra` no vale para medir tamaño: `String::len()` sobre
/// un `String` construido con `from_utf8_lossy` NO cuenta los bytes reales —
/// cada secuencia UTF-8 inválida se sustituye por U+FFFD, que ocupa 3 bytes
/// en UTF-8, así que un blob con un solo byte inválido mediría un tamaño
/// distinto del real. El trinquete en modo `--staged` mide el tamaño de una
/// nota con `git show :./<ruta>` (Task 10), y el Go mide `len(res.Stdout)`
/// sobre bytes crudos (`sizeFromIndex`, `internal/ratchet/staged.go`) — esta
/// función es el equivalente exacto. El contenido sigue leyéndose con
/// `muestra` (String) donde solo hace falta parsear, nunca medir.
pub fn muestra_bytes(dir: &Path, objeto: &str) -> Result<Option<Vec<u8>>> {
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
    Ok(Some(salida.stdout))
}

/// Rutas (relativas, con `/`, nunca separador nativo) de los ficheros `.md`
/// staged en el índice: `git diff --cached --no-renames --name-only
/// --diff-filter=ACMR` (A3). `--no-renames` es flag de `diff`, no de `git` —
/// sin él, qué ficheros salen (una entrada `R` vs. un `A`+`D`) depende de la
/// config `diff.renames` de la máquina; con él, el resultado es el mismo en
/// cualquier entorno. El emparejamiento de renames del trinquete es
/// heurístico sobre los sellos (`empareja_renames`, Task 7) y no usa la
/// detección de renames de git, así que quitarla no le quita información a
/// nadie (adjudicación A3 del plan).
///
/// El filtro de extensión (`.md`, case-insensitive) es aquí y no en
/// `trinquete`: es una propiedad del nombre del fichero, no del dominio de la
/// KB — igual que `walker::es_md`. La exclusión por directorio (`archive`,
/// `.superpowers`, ...) sigue siendo cosa del llamador, que sí conoce
/// `excluidos`.
pub fn md_staged(dir: &Path) -> Result<Vec<String>> {
    let salida = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "diff",
            "--cached",
            "--no-renames",
            "--name-only",
            "--diff-filter=ACMR",
        ])
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .output()
        .with_context(|| format!("invocar git diff --cached en {}", dir.display()))?;
    if !salida.status.success() {
        bail!(
            "git diff --cached --name-only en {}: {}",
            dir.display(),
            String::from_utf8_lossy(&salida.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&salida.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .filter(|l| l.to_ascii_lowercase().ends_with(".md"))
        .map(str::to_string)
        .collect())
}

/// Epoch (segundos unix, fecha de COMMITTER `%ct` — mismo campo que
/// `indexer::git_epoch_de`, nunca `%at` de autor, que puede diferir en un
/// rebase/amend) del último commit que tocó cada ruta bajo `dir`, en una
/// sola invocación de git en vez de una por nota indexada (Ola 1 G Task 3,
/// backlog:524-566: "un proceso `git log -1` por nota indexada, caro en
/// Windows").
///
/// **Diseño distinto del brief original de esta task, medido contra el
/// fallback per-nota (`indexer::git_epoch_de` / `git log -1 -- ruta`) el
/// 2026-09-15 con tres repos de prueba real (conflicto de merge, merge
/// limpio, ruta no-ASCII + rename):**
///
/// 1. **`-c core.quotepath=false`** (global, antes de `log`): sin esto, git
///    entrecomilla y escapa en octal cualquier byte no-ASCII del nombre de
///    fichero (`"notas/caf\303\251 \342\200\224 x.md"`), y esa cadena nunca
///    casaría con la clave portable (`notas/café — x.md`) que
///    `indexer::ruta_relativa` calcula. La comparación es de OUTPUT, no de
///    pathspec: `git log -1 -- <ruta-no-ascii>` (el fallback) no tiene este
///    problema porque ahí la ruta es un argumento de entrada, nunca una
///    línea de salida a parsear — medido: el fallback resuelve igual con y
///    sin `quotepath`. Con `-z` en vez de `quotepath=false` el problema
///    también se resolvería, pero cambiaría el formato de todo el parseo
///    (registros separados por NUL en vez de líneas) para una ganancia que
///    `quotepath=false` ya da sin tocar el resto del parser — se prefiere
///    la opción de menor blast radius.
/// 2. **`--no-renames`**: mismo argumento que `md_staged` ("por
///    determinismo": qué ruta(s) lista un commit de rename no dependa de
///    `diff.renames` de la máquina). Medido: con o sin el flag, la ruta que
///    HOY existe en disco (la nueva tras el rename) obtiene el mismo epoch
///    en este repo de prueba — pero sin el flag, si algún día una
///    detección de rename por similitud emparejara mal dos ficheros no
///    relacionados, la ruta nueva heredaría el epoch de un fichero ajeno.
/// 3. **`-m` + agrupación por `%H`**: sin `-m`, un commit de MERGE no
///    aparece en absoluto en `--name-only` (ninguna línea de fichero), así
///    que una ruta cuyo último toque real fue la resolución de un
///    CONFLICTO dentro del propio merge (la nota se edita a mano al
///    resolver) recibe en el lote el epoch de la rama que git visita
///    primero tras el merge — más viejo que el epoch real, DIVERGE del
///    fallback (medido: repo con conflicto real, diverge sin `-m`). Con
///    `-m` puro (sin agrupar) pasa lo contrario: un merge "limpio" donde
///    cada rama tocó ficheros distintos hace que CADA fichero aparezca en
///    el bloque de UN solo padre, y sin agrupar, `entry().or_insert()`
///    igual lo atribuiría al merge — un FALSO positivo (medido: repo con
///    merge limpio, diverge con `-m` sin agrupar). La regla que reconcilia
///    ambos casos, medida contra el fallback en los dos repos: `-m` hace
///    que `git log` repita la cabecera del commit (mismo `%H`, mismo `%ct`)
///    una vez POR PADRE del merge, cada una seguida del listado de ficheros
///    que cambian respecto a ESE padre. Agrupando las cabeceras
///    consecutivas de igual `%H` (`nbloques` = cuántas veces se repite) y
///    contando en cuántos de esos bloques aparece cada ruta, una ruta solo
///    se atribuye al merge si aparece en **todos** sus bloques — "no es
///    igual a NINGÚN padre", que es exactamente la condición bajo la que
///    `git log -1 -- ruta` (sin `-m`, con simplificación de historia)
///    considera un merge "interesante" para esa ruta. Una ruta que solo
///    difiere de UN padre (trivial, la rama que no la tocó) no cuenta, y
///    sigue el trámite normal: se recoge más abajo en el historial, en el
///    commit no-merge que de verdad la tocó. Para un commit normal
///    (`nbloques == 1`) la regla es un no-op: toda ruta listada aparece en
///    su único bloque, igual que en el diseño original del brief.
/// 4. **`--relative`** (fix de review, medido el 2026-09-15): cuando `dir`
///    (la KB) es un SUBDIRECTORIO de un repo git más grande —caso real: `exo
///    init --from-basic-memory` adopta una KB existente que ya vivía dentro
///    de un repo con más contenido—, `git -C <dir> log --name-only` sin este
///    flag imprime rutas relativas a la RAÍZ del repo (`sub/kb/a.md`), no a
///    `dir` (`a.md`). El llamador (`indexer::indexa`) siempre busca con
///    `ruta_rel = ruta_relativa(kb, ruta_abs)`, relativa a la KB — el lookup
///    nunca casaba, y CADA nota degradaba en silencio al fallback per-nota en
///    CADA corrida (la optimización de esta task quedaba anulada sin error ni
///    log). `--relative` hace que git emita rutas relativas al directorio de
///    `-C` y, de propina, filtra del todo los ficheros fuera de ese
///    subdirectorio — no hace falta un filtro aparte para el fichero
///    `fuera.md` del test de este caso. El fallback per-nota
///    (`indexer::git_epoch_de`) no tiene este problema: ahí la ruta es un
///    pathspec de ENTRADA relativo al `-C`, no una línea de salida a
///    parsear, y git ya la resuelve así sin flags adicionales (confirmado con
///    el mismo repo de prueba).
/// 5. **El divisor de "aparece en todos los bloques" sale de `%P` (número de
///    padres), no de cuántos bloques `-m` llegó a IMPRIMIR** (fix de
///    verificación dirigida, medido el 2026-09-15). Cuando el diff de un
///    merge contra UNO de sus padres queda completamente vacío bajo
///    `--relative` (ese padre solo tocaba rutas fuera de la KB) o vacío sin
///    más (`git merge -s ours`, o un merge no-ff cuyo primer padre no
///    avanzó), git **omite el bloque de ese padre por completo** en la
///    salida de `-m --name-only` — no imprime ni siquiera la cabecera.
///    Medido con los tres repos de prueba: en los tres, el merge de DOS
///    padres imprime UNA sola cabecera para su `%H` en vez de dos. Contar
///    cabeceras impresas (`nbloques`, diseño anterior) da entonces `1` en
///    vez de `2`, y una ruta que solo aparece en el bloque impreso (la que
///    SÍ difiere de ese padre, pero es TREESAME al padre omitido) alcanza
///    `cuenta == nbloques` y se atribuye al merge — divergencia real y
///    silenciosa frente al fallback, que sigue la simplificación de
///    historia real y resuelve esa ruta al commit de la rama, no al merge.
///    `%P` no depende de qué bloques `-m` decidió imprimir: siempre lista el
///    número real de padres, así que un bloque omitido cuenta como "ruta
///    ausente en ese padre" (no aparece en `cuenta`) y la ruta ya no llega a
///    `cuenta == num_padres` — dinámica coherente con la Task 3 original.
///    Commit raíz (`%P` vacío, 0 padres): sin padre trivial con el que ser
///    TREESAME, toda ruta listada se atribuye siempre — de ahí el
///    `.max(1)` en el conteo de padres, que también deja intacto el caso de
///    un commit normal (1 padre, jamás omitido: sin `-m` de por medio no
///    aplica la omisión de bloque, solo afecta a merges).
/// 6. **Contaminación: una ruta que pasa por un bloque omitido se excluye
///    del mapa entero, no solo del merge que la omitió** (fix del
///    orquestador, medido el 2026-09-15 contra `git merge -s ours`). El
///    punto 5 evita que el MERGE se lleve una ruta TREESAME a un padre
///    omitido, pero deja un hueco: si esa ruta también fue tocada por un
///    commit MÁS ANTIGUO alcanzable solo por el padre omitido, ese commit
///    antiguo tiene 1 solo padre real (`num_padres == nbloques_impresos ==
///    1`, ninguna omisión) y supera el chequeo del punto 5 sin más — el
///    lote le atribuiría a la ruta el epoch de un commit cuyo efecto la
///    reescritura de padres de git (parent rewriting, parte de la
///    simplificación de historia que usa `git log -1 -- ruta`) DESCARTA
///    por completo: ese commit solo es alcanzable por el subgrafo
///    no-TREESAME que el padre superviviente del merge tira, y el fallback
///    per-nota nunca lo ve. Medido con `git merge -s ours`: el merge
///    (2 padres, 1 solo bloque impreso porque el diff contra el padre
///    superviviente queda vacío) no atribuye la ruta a sí mismo (punto 5),
///    pero el commit de la rama descartada (1 padre, 1 bloque, sin omisión
///    ninguna) sí "gana" el chequeo normal y le da un epoch que el
///    fallback real nunca produciría — divergencia silenciosa que el punto
///    5 por sí solo no cierra (ver el doc-comment del test
///    `epochs_de_todo_el_historial_en_un_merge_ours_coincide_con_el_fallback`
///    para la salida cruda medida). El stream plano de esta función no
///    tiene grafo: no puede saber, al procesar ese commit antiguo, que un
///    merge posterior en el tiempo (pero procesado ANTES, log en orden
///    newest-first) va a descartar su resultado para esa ruta.
///
///    La regla (ampliada el 2026-09-16, ver más abajo): en cualquier commit
///    donde `-m` OMITE al menos un bloque (`nbloques_impresos < num_padres`,
///    leído de `%P`, no de cuántas cabeceras se llegaron a imprimir), toda
///    ruta que aparezca en CUALQUIERA de los bloques SÍ impresos de ese
///    commit se marca «contaminada» y se excluye del mapa **final entero**
///    — sin importar en qué otro commit del stream (anterior o posterior en
///    el recorrido) se le hubiera asignado un epoch. El coste queda acotado
///    a las rutas tocadas por un commit con AL MENOS un bloque omitido, raro
///    en una KB. Efecto colateral medido y aceptado: los casos (a) y (c)
///    del punto 5 —que sí tienen un bloque omitido, pero cuya ruta
///    contaminada tenía además un commit de rama limpio más abajo en el
///    stream que le daba el epoch correcto por otra vía— ahora también
///    caen al fallback per-nota en vez de resolverse desde el lote: sin
///    grafo, esta función no puede distinguir "esta contaminada tiene un
///    commit de rama limpio de sobra" de "esta contaminada solo es
///    recuperable vía reescritura de padres", y tratar ambos casos igual
///    (fallback) es la única postura que nunca da un dato incorrecto.
///
///    **Ampliación 2026-09-16 (review final de la campaña G):** la regla de
///    arriba solo miraba bloques OMITIDOS por completo, pero un commit puede
///    imprimir TODOS sus bloques (`nbloques_impresos == num_padres`, sin
///    omisión) y aun así descarrilar: si una ruta aparece en SOLO ALGUNOS de
///    esos bloques (`0 < n < num_padres`), es TREESAME a por lo menos un
///    padre para esa ruta concreta — mismo patrón de fondo que un bloque
///    omitido (git sigue el subgrafo del padre no-TREESAME y tira el otro
///    para esa ruta), solo que sin necesitar que el bloque entero quede
///    vacío. Reproducido con git real: un merge de 2 padres que revierte
///    `a.md` a la versión de un padre (conflicto resuelto a favor de ese
///    lado) Y acepta `y.md` del otro sin conflicto — 2 bloques impresos, sin
///    omisión, pero cada ruta aparece en uno solo (`n == 1`); el lote viejo
///    ni atribuía ni contaminaba ninguna de las dos en este commit, y el
///    stream plano se quedaba con el epoch del commit de la rama DESCARTADA
///    para `a.md` (más reciente que el fallback real, nunca al revés) — ver
///    el test `epochs_de_todo_el_historial_en_un_merge_que_revierte_una_ruta_y_acepta_otra_coincide_con_el_fallback`.
///    La regla ampliada: en `cierra_grupo`, cuando `n < num_padres` para una
///    ruta y `num_padres > 1`, esa ruta también se marca «contaminada» (el
///    caso `num_padres == 1` no aplica: sin `-m` de por medio no hay
///    TREESAME posible, es un commit normal). Con esto la garantía original
///    vuelve a sostenerse de verdad: abstenerse nunca da un epoch incorrecto,
///    como mucho una nota de más cayendo al fallback — el coste medido sobre
///    la KB real de Paul (174 notas) sube de 15 a 21 caídas al fallback (ver
///    report de la review final de G).
///
/// Recorre el log COMPLETO una vez y se queda con el PRIMER epoch válido
/// visto para cada ruta: git emite los commits del más reciente al más
/// antiguo, así que el primero que la reclama (por la regla de arriba) es
/// su último commit real.
///
/// Fail-loud como el resto de `gitx` (comentario de módulo): un `dir` que
/// no es repo de git, o cualquier fallo de `git log`, propaga `Err` — el
/// llamador (`indexer::indexa`) es quien decide degradar a fallback
/// per-nota, nunca esta función.
pub fn epochs_de_todo_el_historial(dir: &Path) -> Result<HashMap<String, i64>> {
    let salida = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "core.quotepath=false",
            "log",
            "-m",
            "--no-renames",
            "--relative",
            "--format=%x01%H %ct %P",
            "--name-only",
        ])
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .output()
        .with_context(|| format!("invocar git log --name-only en {}", dir.display()))?;
    if !salida.status.success() {
        bail!(
            "git log --name-only en {}: {}",
            dir.display(),
            String::from_utf8_lossy(&salida.stderr).trim()
        );
    }
    let texto = String::from_utf8(salida.stdout)
        .with_context(|| format!("git log de {} devolvió stdout no-UTF8", dir.display()))?;

    let mut epochs: HashMap<String, i64> = HashMap::new();
    // Rutas «contaminadas» (punto 6 del comentario de la función): pasaron
    // por un commit con al menos un bloque `-m` omitido, así que ningún
    // epoch que el stream plano les asigne es de fiar. Se excluyen del mapa
    // final SIN IMPORTAR en qué commit del stream (antes o después) se
    // intentó insertar su epoch — de ahí que sea un set aparte, filtrado al
    // final, y no una comprobación dentro de `cierra_grupo`.
    let mut contaminadas: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Estado del grupo de bloques en curso (ver el comentario de la función,
    // puntos 3, 5 y 6): `hash_actual`/`epoch_actual` identifican el commit;
    // `num_padres` es el número REAL de padres del commit, leído de `%P` (no
    // de cuántas cabeceras `-m` llegó a imprimir); `nbloques_impresos` es,
    // en cambio, exactamente eso — cuántas cabeceras `-m` imprimió de verdad
    // para este commit (punto 6: la comparación entre ambos es lo que
    // detecta un bloque omitido); `cuenta` cuántos de los bloques IMPRESOS
    // mencionan cada ruta; `vistas_en_bloque` evita contar una ruta dos
    // veces si git la repitiera dentro del MISMO bloque (defensivo, no
    // observado).
    let mut hash_actual: Option<String> = None;
    let mut epoch_actual: Option<i64> = None;
    let mut num_padres: usize = 0;
    let mut nbloques_impresos: usize = 0;
    let mut cuenta: HashMap<String, u32> = HashMap::new();
    let mut vistas_en_bloque: std::collections::HashSet<String> = std::collections::HashSet::new();

    for linea in texto.lines() {
        if let Some(resto) = linea.strip_prefix('\u{1}') {
            let mut partes = resto.splitn(3, ' ');
            let hash = partes.next().unwrap_or("").to_string();
            let epoch = partes.next().and_then(|c| c.trim().parse::<i64>().ok());
            // `%P` es una lista de hashes separados por espacio, vacía en un
            // commit raíz. Un commit raíz (0 padres) no tiene "padre trivial"
            // con el que ser TREESAME — toda ruta listada es nueva y se
            // atribuye siempre, igual que un commit normal de 1 padre — de
            // ahí el `.max(1)`.
            let padres_de_esta_cabecera = partes
                .next()
                .unwrap_or("")
                .split_whitespace()
                .count()
                .max(1);
            if hash_actual.as_deref() != Some(hash.as_str()) {
                cierra_grupo(
                    &mut epochs,
                    &mut contaminadas,
                    epoch_actual,
                    num_padres,
                    nbloques_impresos,
                    &cuenta,
                );
                hash_actual = Some(hash);
                epoch_actual = epoch;
                num_padres = padres_de_esta_cabecera;
                nbloques_impresos = 0;
                cuenta.clear();
            }
            // Cada cabecera `\x01` vista, sea la primera del grupo o una
            // repetición de `-m` para otro padre del mismo commit, es UN
            // bloque impreso — de ahí fuera del `if` de arriba.
            nbloques_impresos += 1;
            vistas_en_bloque.clear();
            continue;
        }
        if linea.is_empty() {
            continue;
        }
        let ruta = linea.replace('\\', "/");
        if vistas_en_bloque.insert(ruta.clone()) {
            *cuenta.entry(ruta).or_insert(0) += 1;
        }
    }
    cierra_grupo(
        &mut epochs,
        &mut contaminadas,
        epoch_actual,
        num_padres,
        nbloques_impresos,
        &cuenta,
    );

    // Filtro final de contaminación (punto 6): se aplica DESPUÉS de recorrer
    // el historial completo porque una ruta puede contaminarse en un commit
    // reciente (procesado primero, stream newest-first) y haber recibido ya
    // una entrada válida-en-apariencia de un commit más antiguo (procesado
    // después) — o al revés. El orden de inserción no importa: contaminada
    // es contaminada, se excluye siempre.
    for ruta in &contaminadas {
        epochs.remove(ruta);
    }

    Ok(epochs)
}

/// Cierra el grupo de bloques de un commit (ver `epochs_de_todo_el_historial`,
/// puntos 3, 5 y 6 del comentario): una ruta entra en `epochs` (primer epoch
/// visto gana, `or_insert`) solo si apareció en TANTOS bloques como PADRES
/// tiene el commit (`num_padres`, leído de `%P`) — no como bloques llegó a
/// IMPRIMIR `-m` (`nbloques_impresos`, contado cabecera a cabecera). Para un
/// commit normal (`num_padres == 1`) o un merge donde `-m` imprimió un
/// bloque por padre, `nbloques_impresos == num_padres` y la regla es la
/// misma de siempre.
///
/// **Contaminación (punto 6):** cuando `-m` OMITE por completo el bloque de
/// algún padre —porque, tras `--relative` (o sin pathspec alguno), el diff
/// contra ese padre quedó vacío: un `git merge -s ours`, un merge no-ff cuyo
/// primer padre no avanzó, o un merge donde uno de los padres solo tocaba
/// rutas fuera de la KB— entonces `nbloques_impresos < num_padres`, y NINGUNA
/// ruta de `cuenta` puede alcanzar `n == num_padres` (su cota máxima es
/// `nbloques_impresos`). Contar bloques impresos en vez de padres reales
/// (diseño anterior a esta task) infla el umbral y atribuye al merge una
/// ruta TREESAME al padre omitido — divergencia real. El punto 5 ya evita
/// ESA atribución equivocada; no basta: toda ruta que apareció en cualquiera
/// de los bloques SÍ impresos de un commit con bloques omitidos se marca
/// «contaminada» y se excluye del mapa final entero, sin importar si otro
/// commit del stream (más antiguo o más reciente) le habría dado un epoch
/// que por casualidad coincide con el fallback. La razón es la reescritura
/// de padres de git (parent rewriting): un commit alcanzable solo por el
/// padre omitido puede tocar esa misma ruta con un resultado que la
/// simplificación de historia real (la que usa el fallback per-nota)
/// DESCARTA por completo — el caso medido es `git merge -s ours`, ver el
/// doc-comment del test
/// `epochs_de_todo_el_historial_en_un_merge_ours_coincide_con_el_fallback`.
/// El stream plano de esta función no tiene grafo: no puede saber, al
/// procesar ese commit antiguo, que un merge (procesado antes, por venir
/// después en el tiempo) va a descartar su resultado para esa ruta.
///
/// **Ampliación 2026-09-16 (review final de G):** contaminar solo cuando un
/// bloque entero falta no basta. Un merge puede imprimir los `num_padres`
/// bloques completos y aun así una ruta puede aparecer en SOLO ALGUNOS de
/// ellos (`0 < n < num_padres`) — TREESAME a por lo menos un padre para esa
/// ruta, sin que ningún bloque quedara vacío. Es el mismo riesgo de
/// reescritura de padres del párrafo de arriba, solo que sin la señal de
/// "bloque omitido": si `num_padres > 1` y una ruta no llega a `n ==
/// num_padres`, también se marca contaminada (ver el test
/// `epochs_de_todo_el_historial_en_un_merge_que_revierte_una_ruta_y_acepta_otra_coincide_con_el_fallback`,
/// que reproduce un merge con conflicto donde ninguna ruta ve su bloque
/// omitido pero ambas son TREESAME a un padre distinto cada una). El caso
/// `num_padres == 1` queda fuera a propósito: sin `-m` de por medio no hay
/// TREESAME que detectar, es un commit normal y `n` solo puede ser 0 o 1.
/// Marcar contaminada la ruta y abstenerse (fallback per-nota) es, con esta
/// ampliación, la única postura que nunca da un epoch incorrecto — la
/// promesa original del comentario de módulo solo se sostiene con las dos
/// mitades de la regla juntas.
fn cierra_grupo(
    epochs: &mut HashMap<String, i64>,
    contaminadas: &mut std::collections::HashSet<String>,
    epoch: Option<i64>,
    num_padres: usize,
    nbloques_impresos: usize,
    cuenta: &HashMap<String, u32>,
) {
    let Some(epoch) = epoch else { return };
    if num_padres == 0 {
        return;
    }
    if nbloques_impresos < num_padres {
        // Bloque(s) omitido(s): ninguna ruta de este commit puede alcanzar
        // `n == num_padres` (ver doc de arriba), así que no hace falta ni
        // mirar `cuenta` para atribución — solo para contaminar.
        for ruta in cuenta.keys() {
            contaminadas.insert(ruta.clone());
        }
        return;
    }
    for (ruta, n) in cuenta {
        if *n as usize == num_padres {
            epochs.entry(ruta.clone()).or_insert(epoch);
        } else if num_padres > 1 {
            // Ningún bloque fue OMITIDO (si lo hubiera sido, ya habríamos
            // vuelto arriba), pero esta ruta apareció en solo ALGUNOS de los
            // bloques impresos: es TREESAME a por lo menos un padre para esta
            // ruta. Igual que el bloque omitido, esto es la marca de que git
            // sigue solo el subgrafo del padre no-TREESAME y descarta el
            // otro para esta ruta — mismo riesgo de reescritura de padres que
            // el punto 6 de arriba, solo que sin necesitar un bloque vacío
            // entero. Contamina y abstente (ver doc-comment de la función).
            contaminadas.insert(ruta.clone());
        }
    }
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

    #[test]
    fn epochs_de_todo_el_historial_da_el_ultimo_commit_por_ruta() {
        let dir = repo("log/a.md", "primero\n");
        let raiz = dir.path();
        let cfg = raiz.join("gitconfig-vacio");
        let corre = |args: &[&str], fecha: &str| {
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
                .env("GIT_AUTHOR_DATE", fecha)
                .env("GIT_COMMITTER_DATE", fecha)
                .output()
                .unwrap();
            assert!(salida.status.success(), "git {args:?} falló");
        };
        std::fs::write(raiz.join("log/b.md"), "b\n").unwrap();
        corre(&["add", "."], "2026-07-02T10:00:00+02:00");
        corre(&["commit", "-q", "-m", "b"], "2026-07-02T10:00:00+02:00");

        let mapa = epochs_de_todo_el_historial(raiz).unwrap();
        assert_eq!(
            mapa.get("log/a.md"),
            Some(&ultimo_commit_epoch(raiz, "log/a.md"))
        );
        assert_eq!(
            mapa.get("log/b.md"),
            Some(&ultimo_commit_epoch(raiz, "log/b.md"))
        );
        assert_ne!(mapa["log/a.md"], mapa["log/b.md"]);
    }

    /// Helper del test de arriba: epoch vía `%ct` per-nota, para comparar
    /// contra el mapa de lote sin duplicar la conversión de fecha a mano.
    fn ultimo_commit_epoch(dir: &Path, ruta_rel: &str) -> i64 {
        let salida = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["log", "-1", "--format=%ct", "--", ruta_rel])
            .output()
            .unwrap();
        String::from_utf8_lossy(&salida.stdout)
            .trim()
            .parse()
            .unwrap()
    }

    #[test]
    fn epochs_de_todo_el_historial_ignora_rutas_nunca_commiteadas() {
        let dir = repo("log/a.md", "cuerpo\n");
        std::fs::write(dir.path().join("log/sin-commit.md"), "x\n").unwrap();
        let mapa = epochs_de_todo_el_historial(dir.path()).unwrap();
        assert!(!mapa.contains_key("log/sin-commit.md"));
    }

    #[test]
    fn epochs_de_todo_el_historial_falla_fuera_de_un_repo() {
        let dir = tempfile::tempdir().unwrap();
        assert!(epochs_de_todo_el_historial(dir.path()).is_err());
    }

    /// Caso límite 1 del brief (global-constraints): ruta con `é`/`—`/espacio.
    /// `core.quotepath` (default `true`) entrecomilla y escapa en octal esos
    /// bytes en la salida de `--name-only`; sin `-c core.quotepath=false` la
    /// clave del mapa de lote no casaría con la ruta portable que calcula
    /// `indexer::ruta_relativa`. Medido el 2026-09-15: sin el flag, este test
    /// falla (la clave del mapa es la cadena octal-escapada).
    #[test]
    fn epochs_de_todo_el_historial_soporta_rutas_no_ascii() {
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        std::fs::create_dir_all(raiz.join("notas")).unwrap();
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
        std::fs::write(raiz.join("notas/café — x.md"), "contenido\n").unwrap();
        corre(&["add", "."]);
        corre(&["commit", "-q", "-m", "unicode"]);

        let mapa = epochs_de_todo_el_historial(raiz).unwrap();
        let esperado = ultimo_commit_epoch(raiz, "notas/café — x.md");
        assert_eq!(mapa.get("notas/café — x.md"), Some(&esperado));
    }

    /// Caso límite 2: un `git mv` no debe hacer que la ruta ACTUAL (la nueva)
    /// pierda su entrada ni herede el epoch equivocado — con `--no-renames` el
    /// commit de rename lista ambas rutas (vieja borrada, nueva añadida) y la
    /// nueva recibe el epoch del propio commit de rename, igual que el
    /// fallback per-nota.
    #[test]
    fn epochs_de_todo_el_historial_tras_un_git_mv_coincide_con_el_fallback() {
        let dir = repo("log/original.md", "cuerpo\n");
        let raiz = dir.path();
        let cfg = raiz.join("gitconfig-vacio");
        let corre = |args: &[&str], fecha: &str| {
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
                .env("GIT_AUTHOR_DATE", fecha)
                .env("GIT_COMMITTER_DATE", fecha)
                .output()
                .unwrap();
            assert!(salida.status.success(), "git {args:?} falló");
        };
        corre(
            &["mv", "log/original.md", "log/renombrada.md"],
            "2026-07-02T10:00:00+02:00",
        );
        corre(
            &["commit", "-q", "-m", "rename"],
            "2026-07-02T10:00:00+02:00",
        );

        let mapa = epochs_de_todo_el_historial(raiz).unwrap();
        let esperado = ultimo_commit_epoch(raiz, "log/renombrada.md");
        assert_eq!(mapa.get("log/renombrada.md"), Some(&esperado));
    }

    /// Caso límite 3a: merge que RESUELVE UN CONFLICTO editando el fichero en
    /// el propio commit de merge. Sin `-m`, `--name-only` no lista ficheros
    /// de un merge en absoluto, y el lote atribuiría la ruta al commit de la
    /// rama que git visita primero tras el merge — más viejo que el epoch
    /// real. Medido el 2026-09-15: sin `-m` (o con `-m` sin agrupar por
    /// `%H`), este test diverge del fallback.
    #[test]
    fn epochs_de_todo_el_historial_en_un_merge_con_conflicto_coincide_con_el_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        let cfg = raiz.join("gitconfig-vacio");
        let corre = |args: &[&str], fecha: &str| {
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
                .env("GIT_AUTHOR_DATE", fecha)
                .env("GIT_COMMITTER_DATE", fecha)
                .output()
                .unwrap();
            assert!(salida.status.success(), "git {args:?} falló");
        };
        std::fs::write(&cfg, "").unwrap();
        corre(&["init", "-q"], "2026-01-01T10:00:00+00:00");
        std::fs::write(raiz.join("conflict.md"), "base\n").unwrap();
        corre(&["add", "."], "2026-01-01T10:00:00+00:00");
        corre(&["commit", "-q", "-m", "base"], "2026-01-01T10:00:00+00:00");
        corre(
            &["checkout", "-q", "-b", "rama-a"],
            "2026-01-01T10:00:00+00:00",
        );
        std::fs::write(raiz.join("conflict.md"), "version-a\n").unwrap();
        corre(&["add", "."], "2026-01-02T10:00:00+00:00");
        corre(&["commit", "-q", "-m", "a"], "2026-01-02T10:00:00+00:00");
        corre(&["checkout", "-q", "master"], "2026-01-01T10:00:00+00:00");
        std::fs::write(raiz.join("conflict.md"), "version-b\n").unwrap();
        corre(&["add", "."], "2026-01-03T10:00:00+00:00");
        corre(&["commit", "-q", "-m", "b"], "2026-01-03T10:00:00+00:00");
        // El merge falla con conflicto (esperado, se ignora el status); se
        // resuelve a mano y se commitea aparte con su propia fecha.
        let _ = Command::new("git")
            .arg("-C")
            .arg(raiz)
            .args(["merge", "rama-a", "-q", "-m", "merge with conflict"])
            .env("GIT_CONFIG_GLOBAL", &cfg)
            .env("GIT_CONFIG_SYSTEM", &cfg)
            .output()
            .unwrap();
        std::fs::write(raiz.join("conflict.md"), "resuelto\n").unwrap();
        corre(&["add", "."], "2026-01-04T10:00:00+00:00");
        corre(
            &["commit", "-q", "-m", "merge with conflict"],
            "2026-01-04T10:00:00+00:00",
        );

        let mapa = epochs_de_todo_el_historial(raiz).unwrap();
        let esperado = ultimo_commit_epoch(raiz, "conflict.md");
        assert_eq!(mapa.get("conflict.md"), Some(&esperado));
    }

    /// Caso límite 3b, contraste del anterior: merge LIMPIO donde cada rama
    /// tocó un fichero distinto (sin conflicto). Aquí `-m` sin agrupar por
    /// `%H` produciría el falso positivo contrario: atribuiría AMBOS
    /// ficheros al merge, cuando en realidad cada uno pertenece a su commit
    /// de rama. Medido el 2026-09-15: con `-m` sin agrupar, este test
    /// diverge del fallback.
    /// Fix de review de esta task: la KB puede ser un SUBDIRECTORIO de un repo
    /// git más grande (caso real: `exo init --from-basic-memory` adopta una
    /// KB existente que ya vive dentro de un repo con más contenido). Sin
    /// `--relative`, `git -C <kb> log --name-only` imprime rutas relativas a
    /// la RAÍZ del repo (`sub/kb/a.md`), no a la KB (`a.md`) — el lookup de
    /// `indexer::indexa` (que compara contra `ruta_relativa(kb, ...)`, SIEMPRE
    /// relativa a la KB) nunca casa, y todas las notas degradan en silencio al
    /// fallback per-nota en cada corrida. Reproducido aquí: repo con la KB en
    /// `sub/kb/` y un fichero FUERA de la KB en el mismo repo.
    #[test]
    fn epochs_de_todo_el_historial_con_kb_subdirectorio_del_repo_coincide_con_el_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        let kb = raiz.join("sub").join("kb");
        std::fs::create_dir_all(&kb).unwrap();
        let cfg = raiz.join("gitconfig-vacio");
        std::fs::write(&cfg, "").unwrap();
        let corre = |args: &[&str], fecha: &str| {
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
                .env("GIT_AUTHOR_DATE", fecha)
                .env("GIT_COMMITTER_DATE", fecha)
                .output()
                .unwrap();
            assert!(salida.status.success(), "git {args:?} falló");
        };
        corre(&["init", "-q"], "2026-01-01T10:00:00+00:00");
        std::fs::write(kb.join("a.md"), "alfa\n").unwrap();
        corre(&["add", "."], "2026-01-01T10:00:00+00:00");
        corre(&["commit", "-q", "-m", "a"], "2026-01-01T10:00:00+00:00");

        std::fs::write(kb.join("b.md"), "beta\n").unwrap();
        corre(&["add", "."], "2026-01-02T10:00:00+00:00");
        corre(&["commit", "-q", "-m", "b"], "2026-01-02T10:00:00+00:00");

        // Fichero FUERA de la KB, hermano de `sub/kb`, en el mismo repo: debe
        // quedar invisible para el mapa (ni como clave con prefijo `sub/kb/`
        // de otra ruta, ni como clave propia).
        std::fs::write(raiz.join("fuera.md"), "fuera\n").unwrap();
        corre(&["add", "."], "2026-01-03T10:00:00+00:00");
        corre(
            &["commit", "-q", "-m", "fuera"],
            "2026-01-03T10:00:00+00:00",
        );

        let mapa = epochs_de_todo_el_historial(&kb).unwrap();
        let esperado_a = crate::indexer::git_epoch_de(&kb, Path::new("a.md"));
        let esperado_b = crate::indexer::git_epoch_de(&kb, Path::new("b.md"));

        assert_eq!(
            mapa.get("a.md").copied(),
            esperado_a,
            "a.md: la clave debe ser relativa a la KB, no al repo"
        );
        assert_eq!(
            mapa.get("b.md").copied(),
            esperado_b,
            "b.md: la clave debe ser relativa a la KB, no al repo"
        );
        assert!(
            !mapa.contains_key("fuera.md"),
            "un fichero fuera de la KB no debe aparecer en el mapa"
        );
        assert!(
            !mapa.keys().any(|k| k.contains("sub/kb")),
            "las claves no deben llevar el prefijo del repo: {mapa:?}"
        );
    }

    #[test]
    fn epochs_de_todo_el_historial_en_un_merge_limpio_coincide_con_el_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        let cfg = raiz.join("gitconfig-vacio");
        let corre = |args: &[&str], fecha: &str| {
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
                .env("GIT_AUTHOR_DATE", fecha)
                .env("GIT_COMMITTER_DATE", fecha)
                .output()
                .unwrap();
            assert!(salida.status.success(), "git {args:?} falló");
        };
        std::fs::write(&cfg, "").unwrap();
        corre(&["init", "-q"], "2026-01-01T10:00:00+00:00");
        std::fs::write(raiz.join("base.md"), "base\n").unwrap();
        corre(&["add", "."], "2026-01-01T10:00:00+00:00");
        corre(&["commit", "-q", "-m", "base"], "2026-01-01T10:00:00+00:00");
        corre(
            &["checkout", "-q", "-b", "rama-a"],
            "2026-01-01T10:00:00+00:00",
        );
        std::fs::write(raiz.join("y.md"), "contenido-y\n").unwrap();
        corre(&["add", "."], "2026-01-02T10:00:00+00:00");
        corre(
            &["commit", "-q", "-m", "add y on a"],
            "2026-01-02T10:00:00+00:00",
        );
        corre(&["checkout", "-q", "master"], "2026-01-01T10:00:00+00:00");
        std::fs::write(raiz.join("z.md"), "contenido-z\n").unwrap();
        corre(&["add", "."], "2026-01-03T10:00:00+00:00");
        corre(
            &["commit", "-q", "-m", "add z on master"],
            "2026-01-03T10:00:00+00:00",
        );
        corre(
            &["merge", "rama-a", "-q", "-m", "clean merge", "--no-edit"],
            "2026-01-04T10:00:00+00:00",
        );

        let mapa = epochs_de_todo_el_historial(raiz).unwrap();
        let esperado_y = ultimo_commit_epoch(raiz, "y.md");
        let esperado_z = ultimo_commit_epoch(raiz, "z.md");
        // Actualizado por la ampliación 2026-09-16 de la regla de
        // contaminación (ver doc-comment de `cierra_grupo`, punto 6
        // ampliado): en este merge de 2 padres SIN bloque omitido, `y.md`
        // solo aparece en el bloque de `master` (TREESAME a `rama-a`) y
        // `z.md` solo en el de `rama-a` (TREESAME a `master`) — ambas caen
        // en `0 < n < num_padres` y quedan contaminadas, aunque en este caso
        // concreto (adiciones disjuntas, sin conflicto real) el lote SIN la
        // ampliación también acertaba. El valor efectivo sigue coincidiendo
        // con el fallback; lo que cambia es que ya no se resuelve desde el
        // mapa — coste aceptado de la regla ampliada, documentado en I1.
        let efectivo_y = mapa.get("y.md").copied().unwrap_or(esperado_y);
        let efectivo_z = mapa.get("z.md").copied().unwrap_or(esperado_z);
        assert_eq!(
            efectivo_y, esperado_y,
            "y.md: el valor efectivo (mapa o fallback) debe coincidir con el per-nota"
        );
        assert_eq!(
            efectivo_z, esperado_z,
            "z.md: el valor efectivo (mapa o fallback) debe coincidir con el per-nota"
        );
        assert_ne!(efectivo_y, efectivo_z);
        assert!(
            !mapa.contains_key("y.md") && !mapa.contains_key("z.md"),
            "ambas caen al fallback per-nota bajo la regla de contaminación \
             ampliada (TREESAME a un solo padre, sin bloque omitido)"
        );
    }

    /// Caso (a) de la verificación dirigida (orquestador, 2026-09-15): KB en
    /// `sub/kb/`, rama A toca `sub/kb/a.md`, rama B toca SOLO `fuera.md`
    /// (fuera de la KB, hermano de `sub/kb`), merge no-ff de B en A.
    ///
    /// Diff del merge contra el padre A: dentro del repo completo solo
    /// cambia `fuera.md` (A no tocó nada más), y `--relative` lo filtra por
    /// completo — CERO rutas de la KB en ese bloque. Diff contra el padre B:
    /// `a.md` cambia (B no lo tocó, A sí). Salida cruda medida (git real,
    /// scratchpad, antes de escribir este test):
    ///
    /// ```text
    /// \x01<merge> <ct> <hashA> <hashB>
    ///
    /// a.md
    /// \x01<hashB> <ctB> <hashBase>
    /// \x01<hashA> <ctA> <hashBase>
    ///
    /// a.md
    /// \x01<hashBase> <ctBase>
    ///
    /// a.md
    /// ```
    ///
    /// El bloque del merge contra A NUNCA se imprime (ni cabecera): solo
    /// aparece UNA cabecera para el merge, seguida del bloque no vacío
    /// (contra B). Con el diseño anterior (`nbloques` = cabeceras impresas),
    /// `nbloques == 1` para el merge y `a.md` (que aparece en ese único
    /// bloque) se atribuía al merge — divergía del fallback, que resuelve
    /// `a.md` al commit de A (TREESAME al merge respecto a A).
    #[test]
    fn epochs_de_todo_el_historial_en_un_merge_con_padre_omitido_fuera_de_la_kb_coincide_con_el_fallback()
     {
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        let kb = raiz.join("sub").join("kb");
        std::fs::create_dir_all(&kb).unwrap();
        let cfg = raiz.join("gitconfig-vacio");
        std::fs::write(&cfg, "").unwrap();
        let corre = |args: &[&str], fecha: &str| {
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
                .env("GIT_AUTHOR_DATE", fecha)
                .env("GIT_COMMITTER_DATE", fecha)
                .output()
                .unwrap();
            assert!(salida.status.success(), "git {args:?} falló");
        };
        corre(&["init", "-q"], "2026-01-01T10:00:00+00:00");
        std::fs::write(kb.join("a.md"), "base\n").unwrap();
        std::fs::write(raiz.join("fuera.md"), "base\n").unwrap();
        corre(&["add", "."], "2026-01-01T10:00:00+00:00");
        corre(&["commit", "-q", "-m", "base"], "2026-01-01T10:00:00+00:00");
        corre(
            &["checkout", "-q", "-b", "rama-a"],
            "2026-01-01T10:00:00+00:00",
        );
        std::fs::write(kb.join("a.md"), "cambio-a\n").unwrap();
        corre(&["add", "."], "2026-01-02T10:00:00+00:00");
        corre(
            &["commit", "-q", "-m", "A toca a.md"],
            "2026-01-02T10:00:00+00:00",
        );
        corre(
            &["checkout", "-q", "-b", "rama-b", "master"],
            "2026-01-01T10:00:00+00:00",
        );
        std::fs::write(raiz.join("fuera.md"), "cambio-b\n").unwrap();
        corre(&["add", "."], "2026-01-02T12:00:00+00:00");
        corre(
            &["commit", "-q", "-m", "B toca fuera.md"],
            "2026-01-02T12:00:00+00:00",
        );
        corre(&["checkout", "-q", "rama-a"], "2026-01-02T10:00:00+00:00");
        corre(
            &["merge", "-q", "--no-edit", "rama-b", "-m", "merge B en A"],
            "2026-01-03T10:00:00+00:00",
        );

        let mapa = epochs_de_todo_el_historial(&kb).unwrap();
        let esperado_a = crate::indexer::git_epoch_de(&kb, Path::new("a.md"));
        // Valor EFECTIVO (mapa si existe, si no el per-nota), no presencia en
        // el mapa: con la regla de contaminación (punto 6 del doc-comment de
        // `epochs_de_todo_el_historial`) este merge tiene un bloque omitido
        // (el padre A, filtrado del todo por `--relative`), así que "a.md"
        // —que SÍ aparece en el único bloque impreso, el de B— se marca
        // contaminada y cae al fallback per-nota, aunque el commit "A toca
        // a.md" más abajo en el stream le habría dado el mismo epoch por otra
        // vía. Aceptado: nunca es un dato incorrecto, como mucho una nota más
        // en el fallback (ver el punto 6 del doc-comment).
        let efectivo = mapa.get("a.md").copied().or(esperado_a);
        assert_eq!(
            efectivo, esperado_a,
            "a.md es TREESAME al merge respecto al padre A (bloque omitido): \
             el valor efectivo debe coincidir con el fallback per-nota"
        );
        assert!(
            !mapa.contains_key("a.md"),
            "a.md queda contaminada (bloque omitido en este merge) y debe \
             caer al fallback, no resolverse desde el lote"
        );
    }

    /// Caso (b) de la verificación dirigida (orquestador, 2026-09-15): KB en
    /// la raíz, `git merge -s ours` de una rama que modificó `b.md` — el
    /// merge queda byte-idéntico al primer padre (estrategia `ours` descarta
    /// el contenido del otro padre).
    ///
    /// **M1 (triaje de la review final de G, 2026-09-16): este comentario
    /// documentaba un `NEEDS_CONTEXT` con el test en `#[ignore]` — ya no es
    /// cierto.** El fix de contaminación (punto 6 del comentario de
    /// `epochs_de_todo_el_historial`, commit `6d08961`) SÍ cierra este caso:
    /// el merge (`-s ours`, 2 padres, 1 solo bloque impreso porque el diff
    /// contra el padre superviviente queda vacío) tiene `nbloques_impresos
    /// (1) < num_padres (2)` — bloque omitido — así que TODA ruta de ese
    /// bloque (`b.md`) se marca contaminada y se excluye del mapa **final
    /// entero**, no solo del merge. El commit de la rama descartada ("rama
    /// toca b", el que antes "ganaba" el chequeo normal con un epoch que el
    /// fallback real nunca produciría) queda excluido igual, por la misma
    /// regla. El test está en la suite verde, sin `#[ignore]`, desde ese
    /// commit — ver el aserto `!mapa.contains_key("b.md")` más abajo, que
    /// falsifica justo lo que este comentario negaba.
    #[test]
    fn epochs_de_todo_el_historial_en_un_merge_ours_coincide_con_el_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        let cfg = raiz.join("gitconfig-vacio");
        std::fs::write(&cfg, "").unwrap();
        let corre = |args: &[&str], fecha: &str| {
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
                .env("GIT_AUTHOR_DATE", fecha)
                .env("GIT_COMMITTER_DATE", fecha)
                .output()
                .unwrap();
            assert!(salida.status.success(), "git {args:?} falló");
        };
        corre(&["init", "-q"], "2026-01-01T10:00:00+00:00");
        std::fs::write(raiz.join("b.md"), "base\n").unwrap();
        corre(&["add", "."], "2026-01-01T10:00:00+00:00");
        corre(&["commit", "-q", "-m", "base"], "2026-01-01T10:00:00+00:00");
        corre(
            &["checkout", "-q", "-b", "rama"],
            "2026-01-01T10:00:00+00:00",
        );
        std::fs::write(raiz.join("b.md"), "cambio\n").unwrap();
        corre(&["add", "."], "2026-01-02T10:00:00+00:00");
        corre(
            &["commit", "-q", "-m", "rama toca b"],
            "2026-01-02T10:00:00+00:00",
        );
        corre(&["checkout", "-q", "master"], "2026-01-01T10:00:00+00:00");
        corre(
            &[
                "merge",
                "-s",
                "ours",
                "-q",
                "--no-edit",
                "rama",
                "-m",
                "merge ours",
            ],
            "2026-01-03T10:00:00+00:00",
        );

        let mapa = epochs_de_todo_el_historial(raiz).unwrap();
        let esperado = ultimo_commit_epoch(raiz, "b.md");
        // Valor EFECTIVO: lo que `indexer::indexa` acaba usando de verdad
        // (entrada del mapa si existe, si no el per-nota) — nunca debe
        // divergir del fallback, esté o no la ruta en el mapa.
        let efectivo = mapa.get("b.md").copied().unwrap_or(esperado);
        assert_eq!(
            efectivo, esperado,
            "el valor efectivo (mapa o fallback) debe coincidir con el per-nota"
        );
        // La parte que de verdad falsifica la regla de contaminación: ANTES
        // del fix, `mapa` contenía "b.md" con el epoch equivocado
        // (el de "rama toca b", descartado por la reescritura de padres de
        // `-s ours`) y el assert de arriba habría fallado. Con la regla, la
        // ruta queda excluida del mapa y cae al fallback per-nota.
        assert!(
            !mapa.contains_key("b.md"),
            "b.md debe caer al fallback per-nota: el lote (stream plano, sin \
             grafo) no puede replicar la reescritura de padres de `-s ours`"
        );
    }

    /// Variante barata del caso (b): mismo `git merge -s ours`, pero con la
    /// KB en un SUBDIRECTORIO del repo (como el caso de subdirectorio y el
    /// caso (a)) — combina `--relative` con la regla de contaminación del
    /// punto 6. `fuera.md`, hermano de `sub/kb`, existe solo para confirmar
    /// que sigue invisible para el mapa (mismo chequeo que el test de
    /// subdirectorio de más arriba).
    #[test]
    fn epochs_de_todo_el_historial_en_un_merge_ours_con_kb_subdirectorio_coincide_con_el_fallback()
    {
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        let kb = raiz.join("sub").join("kb");
        std::fs::create_dir_all(&kb).unwrap();
        let cfg = raiz.join("gitconfig-vacio");
        std::fs::write(&cfg, "").unwrap();
        let corre = |args: &[&str], fecha: &str| {
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
                .env("GIT_AUTHOR_DATE", fecha)
                .env("GIT_COMMITTER_DATE", fecha)
                .output()
                .unwrap();
            assert!(salida.status.success(), "git {args:?} falló");
        };
        corre(&["init", "-q"], "2026-01-01T10:00:00+00:00");
        std::fs::write(kb.join("b.md"), "base\n").unwrap();
        std::fs::write(raiz.join("fuera.md"), "base\n").unwrap();
        corre(&["add", "."], "2026-01-01T10:00:00+00:00");
        corre(&["commit", "-q", "-m", "base"], "2026-01-01T10:00:00+00:00");
        corre(
            &["checkout", "-q", "-b", "rama"],
            "2026-01-01T10:00:00+00:00",
        );
        std::fs::write(kb.join("b.md"), "cambio\n").unwrap();
        corre(&["add", "."], "2026-01-02T10:00:00+00:00");
        corre(
            &["commit", "-q", "-m", "rama toca b"],
            "2026-01-02T10:00:00+00:00",
        );
        corre(&["checkout", "-q", "master"], "2026-01-01T10:00:00+00:00");
        corre(
            &[
                "merge",
                "-s",
                "ours",
                "-q",
                "--no-edit",
                "rama",
                "-m",
                "merge ours",
            ],
            "2026-01-03T10:00:00+00:00",
        );

        let mapa = epochs_de_todo_el_historial(&kb).unwrap();
        let esperado = crate::indexer::git_epoch_de(&kb, Path::new("b.md"));
        let efectivo = mapa.get("b.md").copied().or(esperado);
        assert_eq!(
            efectivo, esperado,
            "b.md (KB en subdirectorio, -s ours): el valor efectivo debe \
             coincidir con el fallback per-nota"
        );
        assert!(
            !mapa.contains_key("b.md"),
            "b.md debe caer al fallback per-nota, igual que en la raíz"
        );
        assert!(
            !mapa.contains_key("fuera.md") && !mapa.keys().any(|k| k.contains("sub/kb")),
            "fuera.md no debe aparecer, y ninguna clave debe llevar el prefijo del repo"
        );
    }

    /// Caso (c) de la verificación dirigida (orquestador, 2026-09-15): KB en
    /// la raíz, merge no-ff donde la rama B no tocó nada de lo que A tocó Y
    /// el primer padre (`master`) no avanzó desde el commit base — el
    /// fast-forward trivial se fuerza como merge explícito.
    ///
    /// Diff del merge contra el primer padre (`master` == commit base):
    /// `a.md` cambia (la rama sí lo tocó) — bloque NO vacío. Diff contra el
    /// segundo padre (la rama): vacío (TREESAME, el merge es idéntico a la
    /// rama). Salida cruda medida: UNA sola cabecera para el merge, con
    /// `a.md` en el bloque contra `master`. `nbloques == 1` (diseño
    /// anterior) atribuía `a.md` al merge; el fallback lo resuelve al commit
    /// de la rama (TREESAME al merge), no al merge — este es el patrón más
    /// común en la práctica: "mergear una rama de feature en una main que no
    /// se movió".
    #[test]
    fn epochs_de_todo_el_historial_en_un_merge_no_ff_sin_avance_del_primer_padre_coincide_con_el_fallback()
     {
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        let cfg = raiz.join("gitconfig-vacio");
        std::fs::write(&cfg, "").unwrap();
        let corre = |args: &[&str], fecha: &str| {
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
                .env("GIT_AUTHOR_DATE", fecha)
                .env("GIT_COMMITTER_DATE", fecha)
                .output()
                .unwrap();
            assert!(salida.status.success(), "git {args:?} falló");
        };
        corre(&["init", "-q"], "2026-01-01T10:00:00+00:00");
        std::fs::write(raiz.join("a.md"), "base\n").unwrap();
        corre(&["add", "."], "2026-01-01T10:00:00+00:00");
        corre(&["commit", "-q", "-m", "base"], "2026-01-01T10:00:00+00:00");
        corre(
            &["checkout", "-q", "-b", "feat"],
            "2026-01-01T10:00:00+00:00",
        );
        std::fs::write(raiz.join("a.md"), "cambio\n").unwrap();
        corre(&["add", "."], "2026-01-02T10:00:00+00:00");
        corre(
            &["commit", "-q", "-m", "feat toca a.md"],
            "2026-01-02T10:00:00+00:00",
        );
        corre(&["checkout", "-q", "master"], "2026-01-01T10:00:00+00:00");
        // master NO avanza: se queda en el commit base. Merge no-ff forzado
        // (sin --no-ff, este sería un fast-forward puro y no habría commit
        // de merge que ejercitar).
        corre(
            &[
                "merge",
                "--no-ff",
                "-q",
                "feat",
                "-m",
                "merge no-ff, main no avanzo",
            ],
            "2026-01-03T10:00:00+00:00",
        );

        let mapa = epochs_de_todo_el_historial(raiz).unwrap();
        let esperado = ultimo_commit_epoch(raiz, "a.md");
        // Valor EFECTIVO, no presencia en el mapa — mismo razonamiento que el
        // caso (a): este merge también tiene un bloque omitido (contra
        // 'feat', TREESAME), así que "a.md" queda contaminada y cae al
        // fallback per-nota aunque el commit "feat toca a.md" le habría dado
        // el mismo epoch por el camino normal. Aceptado (punto 6 del
        // doc-comment de `epochs_de_todo_el_historial`).
        let efectivo = mapa.get("a.md").copied().unwrap_or(esperado);
        assert_eq!(
            efectivo, esperado,
            "a.md es TREESAME al merge respecto al padre 'feat' (bloque \
             vacío omitido): el valor efectivo debe coincidir con el fallback"
        );
        assert!(
            !mapa.contains_key("a.md"),
            "a.md queda contaminada (bloque omitido en este merge) y debe \
             caer al fallback, no resolverse desde el lote"
        );
    }

    /// Caso reportado en la review final de G (2026-09-16): un merge con
    /// TODOS los bloques `-m` impresos (sin omisión, `nbloques_impresos ==
    /// num_padres == 2`) puede seguir descarrilando el lote si una ruta
    /// aparece en SOLO ALGUNOS de esos bloques. Aquí `master` y `rama` editan
    /// `a.md` de forma distinta (conflicto real); el merge se resuelve
    /// quedándose con la versión de `master` — a.md queda TREESAME al padre
    /// `master` (no aparece en su bloque) pero SÍ difiere de `rama` (aparece
    /// en su bloque): `n == 1 < num_padres == 2`. Con la regla vieja
    /// (`cierra_grupo` solo miraba `n == num_padres` para atribuir, y solo
    /// contaminaba cuando un bloque entero faltaba) esta ruta ni se atribuye
    /// ni se contamina en el merge — sigue "viva" para el resto del stream
    /// plano, que newest-first encuentra antes el commit de `rama` (más
    /// reciente, YA DESCARTADO por la reescritura de padres del merge) que el
    /// de `master` (más viejo, el que de verdad sobrevive) y el primer
    /// `or_insert` se queda con el epoch equivocado — MÁS RECIENTE que el
    /// fallback real, nunca al revés. `y.md`, aceptado de `rama` sin
    /// conflicto, es TREESAME a `rama` (no en su bloque) y nuevo respecto a
    /// `master` (en su bloque): mismo patrón `n == 1 < 2`, pero aquí el lote
    /// viejo SÍ coincidía con el fallback por casualidad (solo hay un commit
    /// que lo toca) — el fix lo manda igualmente a fallback, coste aceptado
    /// (ver doc-comment de `cierra_grupo`, punto 6 ampliado).
    #[test]
    fn epochs_de_todo_el_historial_en_un_merge_que_revierte_una_ruta_y_acepta_otra_coincide_con_el_fallback()
     {
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        let cfg = raiz.join("gitconfig-vacio");
        std::fs::write(&cfg, "").unwrap();
        let corre = |args: &[&str], fecha: &str| {
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
                .env("GIT_AUTHOR_DATE", fecha)
                .env("GIT_COMMITTER_DATE", fecha)
                .output()
                .unwrap();
            assert!(salida.status.success(), "git {args:?} falló");
        };
        corre(&["init", "-q"], "2026-01-01T10:00:00+00:00");
        std::fs::write(raiz.join("a.md"), "v1-base\n").unwrap();
        corre(&["add", "."], "2026-01-01T10:00:00+00:00");
        corre(&["commit", "-q", "-m", "base"], "2026-01-01T10:00:00+00:00");
        corre(
            &["checkout", "-q", "-b", "rama"],
            "2026-01-01T10:00:00+00:00",
        );
        corre(&["checkout", "-q", "master"], "2026-01-01T10:00:00+00:00");
        // master edita a.md ANTES que rama: esta es la versión que sobrevive
        // (fecha t1, más vieja que la de rama).
        std::fs::write(raiz.join("a.md"), "v1-master\n").unwrap();
        corre(&["add", "."], "2026-01-02T10:00:00+00:00");
        corre(
            &["commit", "-q", "-m", "master edita a"],
            "2026-01-02T10:00:00+00:00",
        );
        corre(&["checkout", "-q", "rama"], "2026-01-01T10:00:00+00:00");
        // rama edita a.md DESPUÉS de master (fecha t2) y además añade y.md:
        // esta versión de a.md queda descartada por el merge, pero es la más
        // reciente de las dos.
        std::fs::write(raiz.join("a.md"), "v2-rama\n").unwrap();
        std::fs::write(raiz.join("y.md"), "rama nuevo\n").unwrap();
        corre(&["add", "."], "2026-01-03T10:00:00+00:00");
        corre(
            &["commit", "-q", "-m", "rama edita a y agrega y"],
            "2026-01-03T10:00:00+00:00",
        );
        corre(&["checkout", "-q", "master"], "2026-01-02T10:00:00+00:00");
        // Conflicto real en a.md (master y rama lo editaron distinto); y.md
        // no tiene conflicto, se auto-acepta de rama. Se ignora el status:
        // el merge falla con conflicto, se resuelve a mano abajo.
        let _ = Command::new("git")
            .arg("-C")
            .arg(raiz)
            .args(["merge", "-q", "rama", "-m", "merge que revierte a"])
            .env("GIT_CONFIG_GLOBAL", &cfg)
            .env("GIT_CONFIG_SYSTEM", &cfg)
            .output()
            .unwrap();
        // Resuelve el conflicto quedándose con la versión de master (revierte
        // a.md "al lado viejo"); y.md ya quedó auto-aceptado de rama.
        std::fs::write(raiz.join("a.md"), "v1-master\n").unwrap();
        corre(&["add", "."], "2026-01-04T10:00:00+00:00");
        corre(
            &["commit", "-q", "-m", "merge que revierte a"],
            "2026-01-04T10:00:00+00:00",
        );

        let mapa = epochs_de_todo_el_historial(raiz).unwrap();
        let esperado_a = ultimo_commit_epoch(raiz, "a.md");
        let esperado_y = ultimo_commit_epoch(raiz, "y.md");
        let efectivo_a = mapa.get("a.md").copied().unwrap_or(esperado_a);
        let efectivo_y = mapa.get("y.md").copied().unwrap_or(esperado_y);
        assert_eq!(
            efectivo_a, esperado_a,
            "a.md: el valor efectivo (mapa o fallback) debe coincidir con el \
             per-nota; ANTES del fix el lote daba el epoch de 'rama edita a' \
             (descartado por el merge), más reciente que el real"
        );
        assert!(
            !mapa.contains_key("a.md"),
            "a.md debe caer al fallback per-nota: aparece en SOLO UNO de los \
             dos bloques impresos (TREESAME a master, distinto de rama)"
        );
        assert_eq!(
            efectivo_y, esperado_y,
            "y.md: el valor efectivo debe coincidir con el per-nota"
        );
        assert!(
            !mapa.contains_key("y.md"),
            "y.md también aparece en SOLO UNO de los dos bloques (TREESAME a \
             rama, nuevo respecto a master) y debe caer al fallback, aunque \
             coincida con el lote por casualidad"
        );
    }
}
