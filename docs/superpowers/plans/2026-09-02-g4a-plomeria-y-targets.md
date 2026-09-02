# G4a — plomería del port y `exo targets` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: usa `exo:orchestrate`
> para ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`)
> para tracking.

**Goal:** que `exo targets <topic>` exista en el engine Rust con la semántica
exacta de `kbx targets`, y que la plomería que los otros tres comandos de G4
van a necesitar —frontmatter con contrato kbx y un idioma git *fail-loud*—
quede construida y probada aquí.

**Architecture:** el port es puro de lenguaje, no de datos: kbx ya consulta el
índice SQLite de exo (`notas`, `notas_fts`, `aristas` — M6-04 lo migró en
agosto), así que el SQL se traslada literal y los dos binarios pueden correrse
contra el **mismo fichero** `.db`. Se añaden tres módulos nuevos
(`frontmatter.rs`, `gitx.rs`, `objetivos.rs`) y un subcomando en `main.rs`.
Ningún módulo existente cambia de comportamiento. El orden de tareas va de
dentro afuera: primero las dos piezas de plomería con sus invariantes, luego
las funciones puras de `targets`, luego la que toca DB y disco, y por último
el cableado del CLI. Cierra un gate de paridad pre-registrado contra el
binario Go.

**Tech Stack:** Rust edition 2024, MSRV 1.95 · `rusqlite` 0.40.1 (bundled) ·
`regex` 1.13.1 · `serde`/`serde_json` · `anyhow` · `clap` 4.6.2 derive ·
`tempfile` 3.14 (dev) · git como subproceso.

## Global Constraints

- **El crate vive en `engine/`, no en la raíz del repo.** No hay workspace de
  Cargo. Todo comando de cargo se ejecuta con cwd `engine/`.
- **Fuente del port: `origin/main` de kbx, commit `fe46443`.** Medido el
  2026-09-02: el checkout local de kbx (`main` = `f0d0564`) **diverge** —
  1 commit por delante, **18 por detrás**. `fe46443` es el árbol que tiene la
  guarda de aire (`6332c85`) y el fix del sello huérfano (`0ae126d`);
  `f0d0564` no tiene ninguno de los dos y aporta solo `no_air`. **Ninguno de
  los dos árboles es superconjunto del otro** y la reconciliación no es limpia
  (conflicto en `cmd/kbx/budget.go` e `internal/budget/budget.go`), y en W11 no
  hay toolchain Go para compilar ni testear esa resolución. Por eso: se porta
  desde `fe46443`, y `no_air` se porta aparte desde el diff de `f0d0564` (eso
  cae en **G4b**, no aquí). Nada de este plan toca el repo kbx.
- **Hallazgos ⇒ exit 3, errores ⇒ exit 1** (decisión de Paul, 2026-09-02). En
  kbx, `1` = hay hallazgos y `2` = error; en exo, `1` = error genérico y `3` =
  gate rechazado (`escritor::Rechazo`, `main.rs`). Los comandos de G4 que
  gatean (`budget`, `lint`, `ratchet`) usarán **3** para "la KB está mal" y
  dejarán **1** para "el binario falló". **`targets` no gatea nada**: solo
  emite 0 o error, así que en este plan la decisión no se ejerce — se declara
  porque G4b y G4c la heredan.
- **`SCHEMA_VERSION` sigue en 2 y no se toca** (`engine/src/envelope.rs:4-6`).
  `targets` es superficie **nueva**: añadir un `command` nuevo no es un cambio
  breaking del envelope. Claves de `data` en **inglés** (D8), campos Rust en
  castellano con `#[serde(rename)]`.
- **stdout es exclusivo del envelope.** Todo lo humano y todo aviso va por
  `eprintln!`, con o sin `--json`. Regla ya gateada por tests existentes.
- **Naming**: identificadores Rust en castellano (`objetivos`, `busca_objetivos`,
  `ultimo_commit`), claves JSON y verbos del CLI en inglés (D7).
  Comentarios en castellano, explicando el **porqué**.
- **Errores con `anyhow`.** No se añade `thiserror`. `.context(...)` /
  `.with_context(...)` con mensaje accionable en cada IO/parse falible.
- **Clippy es gate duro**: `cargo clippy --all-targets --locked -- -D warnings`
  debe salir limpio desde el primer commit, igual que `cargo fmt --check`.
- **Los tres tiers de dominio son `core`, `stable`, `log`.** En kbx los
  nominales (8500 / 12500 / 0) están definidos **tres veces** por separado
  (`cmd/kbx/budget.go` flags, `doctor.DefaultBudgetOptions`, y recompuestos en
  `cmd/kbx/ratchet.go`). En el port habrá **una sola fuente de verdad** — pero
  esa constante nace en **G4b**, no aquí: `targets` lee el `tier` como string
  opaco y nunca lo valida.
- **Fixture: no existe `kb-demo` en disco.** Cada test construye su KB en un
  `tempfile::tempdir()` con `git init` + notas `.md` reales. El helper de
  config compartido es `engine/tests/common/mod.rs::con_config`.
- Fuera de scope, y se declara en vez de disimularse: `budget`, `lint`,
  `ratchet` (G4b y G4c), el rewiring de `kb-precommit.sh` y de las skills
  `distill`/`document` a verbos `exo` (G4c), y `rotate`/`stale`/`diff-since`/
  `history`, que por D5 no se portan nunca.

---

## Task 1: `frontmatter.rs` — el parser con contrato kbx

**Files:**
- Create: `engine/src/frontmatter.rs`
- Modify: `engine/src/lib.rs` (añadir `pub mod frontmatter;` en orden alfabético, entre `escritor` e `indexer`)
- Test: inline, `#[cfg(test)] mod tests` al final de `engine/src/frontmatter.rs`

**Interfaces:**
- Consumes: nada. Es la hoja del árbol de dependencias.
- Produces:
  - `pub fn tier(contenido: &str) -> String` — `""` si no hay clave `tier`.
  - `pub fn valor(contenido: &str, clave: &str) -> Option<String>`
  - `pub fn budget_max(contenido: &str) -> Option<i64>` — `Some` **solo** con entero positivo.
  - `pub fn orphan_ok(contenido: &str) -> bool`

**Por qué un módulo nuevo y no `nota::parsea_nota`:** son dos contratos
distintos y conviven a propósito. `nota.rs` parsea YAML con `yaml_serde` y
devuelve `None` si falta `permalink` — perfecto para indexar, inservible para
un gate: descartaría justo las notas rotas que el gate existe para encontrar.
Este módulo replica `kbx/internal/frontmatter`, que deliberadamente **no usa
librería YAML** ("verdict v5: no YAML library"): escanea líneas, nunca falla,
y degrada todo a ausente. Esa diferencia es load-bearing, no duplicación.

- [ ] **Step 1: Escribir los tests que fallan**

Crea `engine/src/frontmatter.rs` con **solo** el bloque de tests (sin las
funciones todavía):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_lee_el_valor_del_frontmatter() {
        let c = "---\ntier: core\ntitle: x\n---\ncuerpo\n";
        assert_eq!(tier(c), "core");
    }

    // El invariante más letal del port (kbx commit 5c7eb3d): en un checkout
    // CRLF el delimitador llega como "---\r". Sin tolerarlo, el frontmatter
    // entero se lee como AUSENTE y todos los comandos siguen saliendo 0.
    // Medido en kbx sobre la KB real: 88 notas NOTIER, waivers ignorados,
    // `budget` respondiendo ok con notes=0 en los tres tiers.
    #[test]
    fn tier_sobrevive_a_un_fichero_con_crlf() {
        let c = "---\r\ntier: core\r\ntitle: x\r\n---\r\ncuerpo\r\n";
        assert_eq!(tier(c), "core");
    }

    // Go hace stripWhitespace (tr -d '[:space:]'), no un trim: replica el awk
    // original de kb-budget-check.sh. Un tier con espacios internos se
    // normaliza en vez de fallar.
    #[test]
    fn tier_quita_todo_el_whitespace_incluido_el_interno() {
        let c = "---\ntier:   co re \n---\n";
        assert_eq!(tier(c), "core");
    }

    // La legalidad NO es responsabilidad de este módulo ("Tier() only
    // extracts; legality is the caller's job").
    #[test]
    fn tier_deja_pasar_un_valor_ilegal_tal_cual() {
        let c = "---\ntier: banana\n---\n";
        assert_eq!(tier(c), "banana");
    }

    #[test]
    fn sin_frontmatter_no_hay_nada() {
        assert_eq!(tier("cuerpo suelto\ntier: core\n"), "");
        assert_eq!(valor("cuerpo suelto\ntier: core\n", "tier"), None);
    }

    // El fichero DEBE empezar por la línea delimitadora. Un BOM UTF-8 delante
    // hace que no se detecte y el frontmatter se lea como ausente. Es el
    // comportamiento de kbx y se replica a propósito: degrada hacia rojo (la
    // nota cae en NOTIER y el gate grita), no hacia verde.
    #[test]
    fn un_bom_delante_del_delimitador_anula_el_bloque() {
        let c = "\u{feff}---\ntier: core\n---\n";
        assert_eq!(tier(c), "");
    }

    #[test]
    fn el_bloque_se_cierra_y_lo_de_despues_no_cuenta() {
        let c = "---\ntitle: x\n---\ntier: log\n";
        assert_eq!(tier(c), "");
    }

    // Refleja el awk original: no exige el delimitador de cierre para
    // encontrar una clave que ya ha visto.
    #[test]
    fn encuentra_la_clave_aunque_el_bloque_no_se_cierre() {
        let c = "---\ntier: stable\n";
        assert_eq!(tier(c), "stable");
    }

    #[test]
    fn valor_recorta_solo_los_extremos_y_preserva_el_interior() {
        let c = "---\ntitle:   hola   mundo  \n---\n";
        assert_eq!(valor(c, "title").as_deref(), Some("hola   mundo"));
    }

    #[test]
    fn valor_corta_por_el_primer_dos_puntos() {
        let c = "---\npermalink: kb/a:b\n---\n";
        assert_eq!(valor(c, "permalink").as_deref(), Some("kb/a:b"));
    }

    // Solo entero positivo activa el techo — fail-toward-red. Ojo: 0 NO es
    // "sin límite", es "ignora el override y usa el nominal del tier".
    #[test]
    fn budget_max_solo_acepta_entero_positivo() {
        assert_eq!(budget_max("---\nkbx_budget_max: 19000\n---\n"), Some(19000));
        assert_eq!(budget_max("---\nkbx_budget_max: 0\n---\n"), None);
        assert_eq!(budget_max("---\nkbx_budget_max: -5\n---\n"), None);
        assert_eq!(budget_max("---\nkbx_budget_max: basura\n---\n"), None);
        assert_eq!(budget_max("---\nkbx_budget_max:\n---\n"), None);
        assert_eq!(budget_max("---\ntier: core\n---\n"), None);
    }

    // Solo el literal exacto "true" waiva. "True", "yes", "1" no.
    #[test]
    fn orphan_ok_solo_con_el_literal_true() {
        assert!(orphan_ok("---\nkbx_orphan_ok: true\n---\n"));
        assert!(!orphan_ok("---\nkbx_orphan_ok: True\n---\n"));
        assert!(!orphan_ok("---\nkbx_orphan_ok: yes\n---\n"));
        assert!(!orphan_ok("---\nkbx_orphan_ok: 1\n---\n"));
        assert!(!orphan_ok("---\ntier: core\n---\n"));
    }
}
```

- [ ] **Step 2: Correr los tests y verlos fallar**

Run: `cd engine && cargo test --lib frontmatter`
Expected: FAIL de compilación — `cannot find function 'tier' in this scope`
(y las otras tres). Que falle por *no existir*, no por assert.

- [ ] **Step 3: Implementación**

Añade, **encima** del bloque de tests, en `engine/src/frontmatter.rs`:

```rust
//! Frontmatter con el contrato exacto de `kbx/internal/frontmatter`, que NO es
//! el de `nota.rs`.
//!
//! `nota::parsea_nota` parsea YAML de verdad y devuelve `None` si falta
//! `permalink`: correcto para indexar, inservible para un gate, porque
//! descartaría justo las notas rotas que el gate existe para encontrar. kbx
//! renunció a la librería YAML a propósito ("verdict v5: no YAML library"):
//! escanea líneas, nunca devuelve error y degrada todo a ausente. Los
//! comandos portados en G4 dependen de esa semántica, así que se replica aquí
//! en vez de reusar la de casa.
//!
//! Los marcadores conservan el prefijo histórico `kbx_` (`kbx_budget_max`,
//! `kbx_orphan_ok`): están escritos en 11 notas vivas y renombrarlos sería una
//! migración de datos a cambio de nada (spec G4, "Nombres que NO se tocan").

/// Recorre el bloque de frontmatter llamando a `f(clave, valor_crudo)` hasta
/// que `f` devuelve `true` o el bloque se cierra.
///
/// El fichero **debe empezar** por la línea delimitadora: un BOM o una línea
/// en blanco delante y el bloque se considera ausente. Es el comportamiento de
/// kbx y degrada hacia rojo (la nota cae en NOTIER), nunca hacia verde.
fn escanea(contenido: &str, mut f: impl FnMut(&str, &str) -> bool) {
    let mut lineas = contenido.lines();
    match lineas.next() {
        Some(primera) if es_delimitador(primera) => {}
        _ => return,
    }
    for linea in lineas {
        if es_delimitador(linea) {
            return;
        }
        let Some((clave, crudo)) = linea.split_once(':') else {
            continue;
        };
        if f(clave.trim(), crudo.trim_start_matches([' ', '\t'])) {
            return;
        }
    }
}

/// Una línea que es `---` seguida solo de whitespace, igual que el
/// `/^---[[:space:]]*$/` del awk original.
///
/// El `\r` es parte de esa clase y hay que recortarlo: `str::lines()` ya quita
/// el `\r` de un `\r\n`, pero no el de un `---\r\r` ni el de una línea que
/// llegue por otra vía. Sin esto, en un checkout CRLF el delimitador de
/// apertura no matchea nunca, el escaneo devuelve **nada** —ni tier, ni
/// waivers— y todos los comandos siguen saliendo con exit 0. Es el fallo
/// silencioso canónico del port (kbx `5c7eb3d`).
fn es_delimitador(linea: &str) -> bool {
    linea.trim_end_matches([' ', '\t', '\r']) == "---"
}

/// El `tier` declarado, o `""` si no hay clave.
///
/// Quita **todo** el whitespace, no solo los extremos: mimetiza el
/// `tr -d '[:space:]'` del `kb-budget-check.sh` retirado, así que `co re` se
/// normaliza a `core` en vez de fallar. La legalidad del valor no se juzga
/// aquí — eso es trabajo del llamante.
pub fn tier(contenido: &str) -> String {
    let mut salida = String::new();
    escanea(contenido, |clave, crudo| {
        if clave == "tier" {
            salida = crudo.chars().filter(|c| !c.is_whitespace()).collect();
            return true;
        }
        false
    });
    salida
}

/// El valor crudo de `clave`, recortado solo por los extremos.
pub fn valor(contenido: &str, clave: &str) -> Option<String> {
    let mut salida = None;
    escanea(contenido, |k, crudo| {
        if k == clave {
            salida = Some(crudo.trim_end_matches([' ', '\t', '\r']).to_string());
            return true;
        }
        false
    });
    salida
}

/// El techo declarado en `kbx_budget_max`.
///
/// `Some` **solo** con un entero positivo: `0`, negativo, vacío o no numérico
/// se ignoran en silencio y manda el nominal del tier. Ojo con la trampa: `0`
/// no significa "sin límite" (esa es la semántica del nominal de tier `log`),
/// significa "no hay declaración".
pub fn budget_max(contenido: &str) -> Option<i64> {
    let crudo = valor(contenido, "kbx_budget_max")?;
    match crudo.trim().parse::<i64>() {
        Ok(n) if n > 0 => Some(n),
        _ => None,
    }
}

/// Si la nota declara el marcador `kbx_orphan_ok: true`.
///
/// Solo el literal exacto `true` waiva: `True`, `yes` y `1` no.
pub fn orphan_ok(contenido: &str) -> bool {
    valor(contenido, "kbx_orphan_ok").is_some_and(|v| v.trim() == "true")
}
```

Y registra el módulo en `engine/src/lib.rs`, entre `pub mod escritor;` y
`pub mod indexer;`:

```rust
pub mod frontmatter;
```

- [ ] **Step 4: Correr los tests y verlos pasar**

Run: `cd engine && cargo test --lib frontmatter`
Expected: PASS, 13 tests.

Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
Expected: sin salida, exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/frontmatter.rs engine/src/lib.rs
git commit -m "feat(frontmatter): el contrato kbx, con el CRLF que apaga el gate en silencio"
```

---

## Task 2: `gitx.rs` — el idioma git *fail-loud*

**Files:**
- Create: `engine/src/gitx.rs`
- Modify: `engine/src/lib.rs` (añadir `pub mod gitx;` entre `frontmatter` e `indexer`)
- Test: inline, `#[cfg(test)] mod tests` al final de `engine/src/gitx.rs`

**Interfaces:**
- Consumes: nada.
- Produces:
  - `pub fn ultimo_commit(kb: &Path, ruta_rel: &str) -> anyhow::Result<String>` —
    fecha ISO-8601 del último commit que tocó el fichero, `""` si no tiene
    ninguno, `Err` si git falla.

**La trampa concreta de este port** (invariante 6 de la spec): el idioma git
que ya hay en casa es **fail-silent** — `indexer::git_epoch_de` devuelve
`Option` y traga el binario ausente, el exit no-cero, el stdout no-UTF8 y el
texto no parseable, todos como `None`. Es correcto ahí: una nota sin
`git_epoch` no es un error de indexado. Pero `targets.LastCommit` en kbx es
**fail-loud** por diseño explícito ("it never degrades silently to ''"), y un
port que reutilice el idioma de casa degrada `last_commit` en silencio. Son
dos funciones distintas a propósito.

`ruta_rel` es `&str`, no `&Path`, y **siempre con `/`**: es un *pathspec* de
git, no una ruta de disco. `notas.ruta` se guarda con el separador nativo
(`indexer::ruta_relativa` no normaliza), así que en Windows llega con `\` y se
convierte antes de dárselo a git.

> **Corregido el 2026-09-02, al ejecutar la tarea.** Este plan afirmaba que
> sin la conversión el pathspec **no matchearía** y `last_commit` degradaría a
> `""` en silencio. **Medido, es falso**: quitando el `replace` en Git for
> Windows los cuatro tests siguen verdes, porque git acepta el `\` en el
> pathspec. La conversión se queda —hace el pathspec determinista frente a
> versiones y configs de git que no tienen por qué compartir esa tolerancia, y
> cuesta una línea— pero es una **guarda defensiva sin test que la ejercite**,
> no un invariante demostrado, y el test se renombró para no prometer lo que
> no prueba. La lección es la de siempre en este repo: un hecho sobre el
> entorno que nadie vuelve a medir sobrevive a las revisiones porque suena
> plausible.

- [ ] **Step 1: Escribir los tests que fallan**

Crea `engine/src/gitx.rs` con **solo** el bloque de tests:

```rust
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
    // normaliza), así que en Windows llega con `\`. Este test comprueba el
    // camino end-to-end con separador nativo; NO falsa el replace() — medido
    // el 2026-09-02, Git for Windows acepta el backslash en el pathspec y sin
    // la conversión los cuatro tests siguen verdes. La conversión se queda por
    // determinismo entre versiones de git, declarada como guarda defensiva.
    #[test]
    fn una_ruta_con_separador_nativo_encuentra_su_commit() {
        let dir = repo("log/a.md", "cuerpo\n");
        let nativa = format!("log{}a.md", std::path::MAIN_SEPARATOR);
        assert_eq!(
            ultimo_commit(dir.path(), &nativa).unwrap(),
            "2026-07-01T10:00:00+02:00"
        );
    }
}
```

- [ ] **Step 2: Correr los tests y verlos fallar**

Run: `cd engine && cargo test --lib gitx`
Expected: FAIL de compilación — `cannot find function 'ultimo_commit' in this scope`.

- [ ] **Step 3: Implementación**

Añade, encima del bloque de tests, en `engine/src/gitx.rs`:

```rust
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
```

Y registra el módulo en `engine/src/lib.rs`, entre `pub mod frontmatter;` y
`pub mod indexer;`:

```rust
pub mod gitx;
```

- [ ] **Step 4: Correr los tests y verlos pasar**

Run: `cd engine && cargo test --lib gitx`
Expected: PASS, 4 tests.

Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
Expected: sin salida, exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/gitx.rs engine/src/lib.rs
git commit -m "feat(gitx): last_commit fail-loud, el idioma que no es el de casa"
```

---

## Task 3: `objetivos.rs` — las dos funciones puras

**Files:**
- Create: `engine/src/objetivos.rs`
- Modify: `engine/src/lib.rs` (añadir `pub mod objetivos;` entre `nota` y `plantilla`)
- Test: inline, `#[cfg(test)] mod tests` al final de `engine/src/objetivos.rs`

**Interfaces:**
- Consumes: nada (funciones puras; la Task 4 añade el resto del módulo).
- Produces:
  - `pub fn construye_match_query(tema: &str) -> anyhow::Result<String>`
  - `pub fn extrae_headings(ruta: &std::path::Path) -> Vec<String>`

- [ ] **Step 1: Escribir los tests que fallan**

Crea `engine/src/objetivos.rs` con **solo** el bloque de tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cita_cada_termino_como_literal_fts5() {
        assert_eq!(construye_match_query("metodologia").unwrap(), "\"metodologia\"");
        assert_eq!(
            construye_match_query("estructura headings").unwrap(),
            "\"estructura\" \"headings\""
        );
    }

    #[test]
    fn duplica_las_comillas_internas() {
        assert_eq!(
            construye_match_query("say \"hi\"").unwrap(),
            "\"say\" \"\"\"hi\"\"\""
        );
    }

    #[test]
    fn un_tema_vacio_es_error() {
        assert!(construye_match_query("").is_err());
        assert!(construye_match_query("   ").is_err());
        assert!(construye_match_query("\t\n").is_err());
    }

    // Ningún operador FTS5 debe sobrevivir al quoting: ni prefijo, ni OR, ni
    // NEAR, ni filtro por columna. Todo es texto literal.
    #[test]
    fn ningun_operador_fts5_sobrevive_al_quoting() {
        for tema in ["metodolog*", "foo OR bar\"", "NEAR(alpha bitacora)", "title:alpha"] {
            let q = construye_match_query(tema).unwrap();
            assert!(q.starts_with('"'), "{tema} -> {q}");
            assert!(q.ends_with('"'), "{tema} -> {q}");
        }
    }

    fn escribe(dir: &std::path::Path, nombre: &str, contenido: &str) -> std::path::PathBuf {
        let p = dir.join(nombre);
        std::fs::write(&p, contenido).unwrap();
        p
    }

    #[test]
    fn extrae_headings_de_nivel_1_a_3_en_orden() {
        let dir = tempfile::tempdir().unwrap();
        let p = escribe(dir.path(), "a.md", "# uno\ntexto\n## dos\n### tres\n");
        assert_eq!(extrae_headings(&p), vec!["uno", "dos", "tres"]);
    }

    // Nivel 4+ no matchea porque tras tres `#` el patrón exige un espacio
    // literal, y un cuarto `#` no lo es. Y lo que hay dentro de una valla de
    // código no es un heading.
    #[test]
    fn ignora_el_nivel_4_y_lo_que_hay_dentro_de_una_valla() {
        let dir = tempfile::tempdir().unwrap();
        let p = escribe(
            dir.path(),
            "a.md",
            "# real\n#### profundo\n```sh\n# falso\n```\n## otro\n",
        );
        assert_eq!(extrae_headings(&p), vec!["real", "otro"]);
    }

    #[test]
    fn un_fichero_ilegible_da_lista_vacia_nunca_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(extrae_headings(&dir.path().join("no-existe.md")).is_empty());
    }

    #[test]
    fn los_headings_sobreviven_a_crlf() {
        let dir = tempfile::tempdir().unwrap();
        let p = escribe(dir.path(), "a.md", "# uno\r\n## dos\r\n");
        assert_eq!(extrae_headings(&p), vec!["uno", "dos"]);
    }
}
```

- [ ] **Step 2: Correr los tests y verlos fallar**

Run: `cd engine && cargo test --lib objetivos`
Expected: FAIL de compilación — `cannot find function 'construye_match_query'`
y `cannot find function 'extrae_headings'`.

- [ ] **Step 3: Implementación**

Añade, encima del bloque de tests, en `engine/src/objetivos.rs`:

```rust
//! `exo targets` — candidatas de la KB para un tema, portado de
//! `kbx/internal/targets`.

use anyhow::{Result, bail};
use std::path::Path;
use std::sync::LazyLock;

/// Headings de nivel 1 a 3. El espacio tras las almohadillas es obligatorio,
/// que es lo que deja fuera a `####` sin necesidad de contarlas.
static PATRON_HEADING: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^#{1,3} (.+)$").unwrap());

/// Convierte un tema en una query FTS5 de literales citados.
///
/// Cada término separado por whitespace se envuelve en comillas dobles y las
/// comillas internas se duplican (el escape de string-literal de FTS5). El
/// efecto es que **ningún** operador sobrevive: ni `*` de prefijo, ni `OR`,
/// ni `NEAR(...)`, ni `col:term`. Todo lo que teclee el usuario es texto.
pub fn construye_match_query(tema: &str) -> Result<String> {
    let terminos: Vec<String> = tema
        .split_whitespace()
        .map(|t| format!("\"{}\"", t.replace('"', "\"\"")))
        .collect();
    if terminos.is_empty() {
        bail!("targets: el tema está vacío");
    }
    Ok(terminos.join(" "))
}

/// Headings de nivel 1-3 del fichero, en orden de aparición.
///
/// Best-effort a propósito: un fichero ilegible o inexistente devuelve lista
/// vacía, nunca error. La candidata sigue apareciendo aunque su fichero no se
/// pueda leer, porque el índice la conoce.
pub fn extrae_headings(ruta: &Path) -> Vec<String> {
    // `read` + `from_utf8_lossy` y no `read_to_string`: en Go esto es un
    // `bufio.Scanner` sobre bytes, que sigue produciendo headings en un
    // fichero con UTF-8 inválido. `read_to_string` fallaría y devolvería lista
    // vacía — una divergencia silenciosa con el binario contra el que se mide
    // la paridad.
    let Ok(bytes) = std::fs::read(ruta) else {
        return Vec::new();
    };
    let contenido = String::from_utf8_lossy(&bytes);
    let mut headings = Vec::new();
    let mut en_valla = false;
    for linea in contenido.lines() {
        if linea.trim_start().starts_with("```") {
            en_valla = !en_valla;
            continue;
        }
        if en_valla {
            continue;
        }
        if let Some(m) = PATRON_HEADING.captures(linea) {
            headings.push(m[1].to_string());
        }
    }
    headings
}
```

Y registra el módulo en `engine/src/lib.rs`, entre `pub mod nota;` y
`pub mod plantilla;`:

```rust
pub mod objetivos;
```

- [ ] **Step 4: Correr los tests y verlos pasar**

Run: `cd engine && cargo test --lib objetivos`
Expected: PASS, 8 tests.

Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
Expected: sin salida, exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/objetivos.rs engine/src/lib.rs
git commit -m "feat(objetivos): query FTS5 sin operadores y headings fuera de las vallas"
```

---

## Task 4: `busca_objetivos` — SQL, dedup y la asimetría de fallo

**Files:**
- Modify: `engine/src/objetivos.rs` (añadir structs y `busca_objetivos`)
- Test: `engine/tests/objetivos.rs` (nuevo, integración contra el crate)

**Interfaces:**
- Consumes:
  - `exo::frontmatter::tier(contenido: &str) -> String` (Task 1)
  - `exo::gitx::ultimo_commit(kb: &Path, ruta_rel: &str) -> Result<String>` (Task 2)
  - `exo::objetivos::construye_match_query(tema: &str) -> Result<String>` (Task 3)
  - `exo::objetivos::extrae_headings(ruta: &Path) -> Vec<String>` (Task 3)
  - `exo::abre_db(ruta: &Path) -> anyhow::Result<rusqlite::Connection>` (ya existe, `lib.rs`)
- Produces:
  - `pub struct Objetivos { pub tema: String, pub candidatos: Vec<Candidato> }`
    (JSON: `topic`, `candidates`)
  - `pub struct Candidato { pub permalink: String, pub tier: String, pub tamano_bytes: i64, pub headings: Vec<String>, pub ultimo_commit: String, pub snippet: String }`
    (JSON: `permalink`, `tier`, `size_bytes`, `headings`, `last_commit`, `snippet`)
  - `pub fn busca_objetivos(conn: &rusqlite::Connection, kb: &Path, tema: &str, limite: usize) -> anyhow::Result<Objetivos>`

- [ ] **Step 1: Escribir los tests que fallan**

Crea `engine/tests/objetivos.rs`:

```rust
//! `busca_objetivos` contra una KB y un índice reales en un tempdir.

use exo::objetivos::busca_objetivos;
use std::path::Path;
use std::process::Command;

/// KB con tres notas committeadas y su índice SQLite poblado a mano.
///
/// El índice se puebla con SQL directo en vez de con `exo index` porque
/// indexar de verdad descargaría el modelo ONNX de 615 MB, y estos tests no
/// ejercen embeddings.
fn kb_con_indice() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let kb = dir.path().to_path_buf();
    let cfg = kb.join("gitconfig-vacio");
    std::fs::write(&cfg, "").unwrap();
    let git = |args: &[&str]| {
        let s = Command::new("git")
            .arg("-C")
            .arg(&kb)
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
        assert!(s.status.success(), "git {args:?}");
    };

    std::fs::create_dir_all(kb.join("log")).unwrap();
    std::fs::write(
        kb.join("log/alpha.md"),
        "---\ntier: stable\n---\n# alpha\ncuerpo de alpha\n",
    )
    .unwrap();
    std::fs::write(
        kb.join("log/beta.md"),
        "---\ntier: core\n---\n# beta\ncuerpo de beta\n",
    )
    .unwrap();
    std::fs::write(kb.join("log/gamma.md"), "sin frontmatter\ncuerpo de gamma\n").unwrap();
    git(&["init", "-q"]);
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "inicial"]);

    let db = dir.path().join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    for (permalink, rel, titulo, cuerpo) in [
        ("kb/log/alpha", "log/alpha.md", "alpha", "cuerpo de alpha"),
        ("kb/log/beta", "log/beta.md", "beta", "cuerpo de beta"),
        ("kb/log/gamma", "log/gamma.md", "gamma", "cuerpo de gamma"),
    ] {
        conn.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
             VALUES (?1, ?2, ?3, 'note', 0.0, NULL)",
            rusqlite::params![permalink, rel, titulo],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO notas_fts (titulo, cuerpo, permalink) VALUES (?1, ?2, ?3)",
            rusqlite::params![titulo, cuerpo, permalink],
        )
        .unwrap();
    }
    drop(conn);
    (dir, db)
}

#[test]
fn encuentra_la_candidata_con_tier_size_y_last_commit_de_disco() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    let r = busca_objetivos(&conn, dir.path(), "alpha", 10).unwrap();

    assert_eq!(r.tema, "alpha");
    assert_eq!(r.candidatos.len(), 1);
    let c = &r.candidatos[0];
    assert_eq!(c.permalink, "kb/log/alpha");
    assert_eq!(c.tier, "stable");
    assert_eq!(c.headings, vec!["alpha"]);
    assert_eq!(c.ultimo_commit, "2026-07-01T10:00:00+02:00");
    let en_disco = std::fs::metadata(dir.path().join("log/alpha.md")).unwrap().len();
    assert_eq!(c.tamano_bytes as u64, en_disco);
}

#[test]
fn sin_coincidencias_la_lista_esta_vacia_y_no_hay_error() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    let r = busca_objetivos(&conn, dir.path(), "zzz-inexistente", 10).unwrap();
    assert!(r.candidatos.is_empty());
}

// El test de `construye_match_query` prueba la FORMA de la cadena; este
// prueba lo único que importa de verdad: que FTS5 la trate como texto. Un
// operador que se ejecutase daría error de sintaxis o traería resultados
// semánticos. Los cuatro casos son las cuatro clases de operador.
//
// Portado de `TestSearch_Injection` de kbx, que mi plan se había dejado.
#[test]
fn ningun_operador_fts5_se_ejecuta_como_operador() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    for tema in [
        "cuerp*",
        "alpha OR beta",
        "NEAR(alpha beta)",
        "titulo:alpha",
    ] {
        let r = busca_objetivos(&conn, dir.path(), tema, 10)
            .unwrap_or_else(|e| panic!("{tema} no debe ser error de sintaxis FTS5: {e:#}"));
        assert!(
            r.candidatos.is_empty(),
            "{tema} se ejecutó como operador y trajo {} candidatas",
            r.candidatos.len()
        );
    }
}

#[test]
fn el_limite_trunca_y_el_orden_es_estable() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    let amplio = busca_objetivos(&conn, dir.path(), "cuerpo", 100).unwrap();
    assert!(amplio.candidatos.len() > 2);

    let a = busca_objetivos(&conn, dir.path(), "cuerpo", 2).unwrap();
    let b = busca_objetivos(&conn, dir.path(), "cuerpo", 2).unwrap();
    assert_eq!(a.candidatos.len(), 2);
    let permalinks = |r: &exo::objetivos::Objetivos| {
        r.candidatos.iter().map(|c| c.permalink.clone()).collect::<Vec<_>>()
    };
    assert_eq!(permalinks(&a), permalinks(&b));
}

#[test]
fn un_limite_menor_que_uno_es_error() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    assert!(busca_objetivos(&conn, dir.path(), "alpha", 0).is_err());
}

// El dedup es por `notas.ruta`, no por permalink, y se queda con la PRIMERA
// fila — que por `ORDER BY rank` es la de mejor rank. Hoy `notas_fts` es 1:1
// con `notas` y el dedup es un no-op, pero se mantiene para el día en que el
// FTS indexe trozos.
#[test]
fn deduplica_por_ruta_conservando_la_fila_de_mejor_rank() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    conn.execute(
        "INSERT INTO notas_fts (titulo, cuerpo, permalink) VALUES ('alpha', 'relleno relleno alpha', 'kb/log/alpha')",
        [],
    )
    .unwrap();
    let r = busca_objetivos(&conn, dir.path(), "alpha", 10).unwrap();
    assert_eq!(r.candidatos.len(), 1, "la ruta duplicada debe colapsar");
}

// Asimetría deliberada: un fallo de DISCO degrada a valores vacíos y la
// candidata sigue apareciendo (el índice la conoce), mientras que un fallo de
// GIT aborta todo el resultado. Este test cubre la mitad best-effort.
#[test]
fn una_candidata_con_fichero_ilegible_sigue_apareciendo() {
    let (dir, db) = kb_con_indice();
    std::fs::remove_file(dir.path().join("log/alpha.md")).unwrap();
    let conn = exo::abre_db(&db).unwrap();
    let r = busca_objetivos(&conn, dir.path(), "alpha", 10).unwrap();
    assert_eq!(r.candidatos.len(), 1);
    assert_eq!(r.candidatos[0].tier, "");
    assert_eq!(r.candidatos[0].tamano_bytes, 0);
    assert!(r.candidatos[0].headings.is_empty());
}

// Y esta es la otra mitad: git roto NO degrada, aborta. Es el invariante 6 de
// la spec — la trampa concreta de este port.
#[test]
fn un_fallo_de_git_aborta_en_vez_de_dejar_last_commit_vacio() {
    let (dir, db) = kb_con_indice();
    std::fs::remove_dir_all(dir.path().join(".git")).unwrap();
    let conn = exo::abre_db(&db).unwrap();
    assert!(busca_objetivos(&conn, dir.path(), "alpha", 10).is_err());
}

// La nota sin frontmatter no se excluye: aparece con tier vacío. Filtrar por
// tipo o por tier aquí escondería notas reales (M6-04 T3 retiró el filtro
// `note_type='note'` justo por eso).
#[test]
fn una_nota_sin_frontmatter_aparece_con_tier_vacio() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    let r = busca_objetivos(&conn, dir.path(), "gamma", 10).unwrap();
    assert_eq!(r.candidatos.len(), 1);
    assert_eq!(r.candidatos[0].tier, "");
}
```

- [ ] **Step 2: Correr los tests y verlos fallar**

Run: `cd engine && cargo test --test objetivos`
Expected: FAIL de compilación — `cannot find function 'busca_objetivos' in
module 'exo::objetivos'`.

- [ ] **Step 3: Implementación**

Añade a `engine/src/objetivos.rs`, entre los `use` y `construye_match_query`
(los `use` de cabecera pasan a ser los de abajo):

```rust
use crate::{frontmatter, gitx};
use anyhow::{Context, Result, bail};
use rusqlite::Connection;
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;
use std::sync::LazyLock;

/// El SQL es el de `kbx/internal/targets` literal, con dos detalles que NO se
/// tocan:
///
/// - `snippet(notas_fts, 1, ...)`: el `1` es el índice ordinal de la columna
///   `cuerpo` dentro de `notas_fts(titulo, cuerpo, permalink UNINDEXED)`.
///   Insertar una columna antes de `cuerpo` haría que los snippets salieran de
///   la columna equivocada **en silencio** — ningún check de esquema mira el
///   orden ordinal, solo la presencia por nombre.
/// - **Sin `LIMIT`**: el truncado se aplica en Rust y DESPUÉS del dedup. Meter
///   `LIMIT ?` aquí es la optimización obvia y rompe la semántica el día que
///   `notas_fts` deje de ser 1:1 con `notas`: truncaría antes de deduplicar y
///   devolvería menos candidatas únicas de las que hay.
const CONSULTA_CANDIDATAS: &str = "SELECT notas.permalink,
       notas.ruta,
       COALESCE(snippet(notas_fts, 1, '', '', '…', 12), '') AS snip
FROM notas_fts
JOIN notas ON notas.permalink = notas_fts.permalink
WHERE notas_fts MATCH ?1
ORDER BY rank";

#[derive(Serialize)]
pub struct Objetivos {
    #[serde(rename = "topic")]
    pub tema: String,
    #[serde(rename = "candidates")]
    pub candidatos: Vec<Candidato>,
}

#[derive(Serialize)]
pub struct Candidato {
    pub permalink: String,
    pub tier: String,
    #[serde(rename = "size_bytes")]
    pub tamano_bytes: i64,
    pub headings: Vec<String>,
    #[serde(rename = "last_commit")]
    pub ultimo_commit: String,
    pub snippet: String,
}

/// Candidatas de la KB para `tema`, ordenadas por rank bm25.
///
/// La asimetría de fallo es deliberada y está cubierta por tests: leer el
/// fichero de disco (tier, tamaño, headings) es **best-effort** y degrada a
/// valores vacíos sin excluir a la candidata, porque el índice la conoce;
/// leer git es **fail-loud** y aborta el resultado entero. Unificar los dos
/// manejos rompe el contrato.
pub fn busca_objetivos(
    conn: &Connection,
    kb: &Path,
    tema: &str,
    limite: usize,
) -> Result<Objetivos> {
    if limite < 1 {
        bail!("targets: --limit tiene que ser >= 1, se recibió {limite}");
    }
    let match_query = construye_match_query(tema)?;

    let mut stmt = conn
        .prepare(CONSULTA_CANDIDATAS)
        .context("preparar la consulta de candidatas")?;
    let mut filas = stmt
        .query(rusqlite::params![match_query])
        .context("ejecutar la consulta de candidatas")?;

    let mut candidatos = Vec::new();
    let mut vistas: HashSet<String> = HashSet::new();

    while candidatos.len() < limite {
        let Some(fila) = filas.next().context("leer una candidata")? else {
            break;
        };
        let permalink: String = fila.get(0)?;
        let ruta_rel: String = fila.get(1)?;
        let snippet: String = fila.get(2)?;

        if !vistas.insert(ruta_rel.clone()) {
            continue;
        }

        let ultimo_commit = gitx::ultimo_commit(kb, &ruta_rel)?;

        // `read` y no `read_to_string`: `size_bytes` es el tamaño en BYTES y
        // el gate de paridad lo compara exacto. Un fichero con UTF-8 inválido
        // le da a Go su tamaño real y a `read_to_string` un Err — es decir, un
        // 0 silencioso justo en el campo que se está midiendo.
        let absoluta = kb.join(&ruta_rel);
        let (tier, tamano_bytes) = match std::fs::read(&absoluta) {
            Ok(bytes) => {
                let contenido = String::from_utf8_lossy(&bytes);
                (frontmatter::tier(&contenido), bytes.len() as i64)
            }
            Err(_) => (String::new(), 0),
        };

        candidatos.push(Candidato {
            permalink,
            tier,
            tamano_bytes,
            headings: extrae_headings(&absoluta),
            ultimo_commit,
            snippet,
        });
    }

    Ok(Objetivos {
        tema: tema.to_string(),
        candidatos,
    })
}
```

- [ ] **Step 4: Correr los tests y verlos pasar**

Run: `cd engine && cargo test --test objetivos`
Expected: PASS, 9 tests.

Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
Expected: sin salida, exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/objetivos.rs engine/tests/objetivos.rs
git commit -m "feat(objetivos): candidatas por rank, dedup por ruta y la asimetria disco/git"
```

---

## Task 5: cablear `exo targets` en el CLI

**Files:**
- Modify: `engine/src/main.rs` (enum `Comando`, struct `ArgsTargets`, `quiere_json`, `ejecuta`, función `targets_cmd`)
- Test: `engine/tests/targets_cli.rs` (nuevo, contra el binario compilado)

**Interfaces:**
- Consumes:
  - `exo::objetivos::busca_objetivos(conn, kb, tema, limite) -> Result<Objetivos>` (Task 4)
  - `exo::objetivos::Objetivos { tema, candidatos }` y `Candidato { permalink, tier, tamano_bytes, headings, ultimo_commit, snippet }` (Task 4)
  - `resuelve_db(flag: Option<PathBuf>) -> Result<PathBuf>` y
    `resuelve_kb(flag: Option<PathBuf>) -> Result<PathBuf>` (ya en `main.rs`)
  - `exo::abre_db(ruta: &Path) -> Result<Connection>` y
    `exo::envelope::emite(command: &str, data: serde_json::Value)` (ya existen)
- Produces: el subcomando `exo targets`. Nada consume esto desde Rust.

**Ojo con `quiere_json`:** es un `match` **exhaustivo sin comodín**, a
propósito ("el compilador debe obligar a decidir explícitamente si emite
JSON"). Añadir `Comando::Targets` sin tocarla es un error de compilación, no
un bug silencioso — pero hay que tocarla.

- [ ] **Step 1: Escribir los tests que fallan**

Crea `engine/tests/targets_cli.rs`:

```rust
//! `exo targets` contra el binario real.
//!
//! No usa `tests/common/mod.rs`: estos tests pasan `--db` y `--kb` explícitos,
//! así que `resuelve_db`/`resuelve_kb` cortan en el flag y nunca llegan a
//! cargar config. Declarar el módulo sin usarlo sería un warning, y clippy es
//! gate duro.

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_exo")
}

/// Reusa el montaje de `tests/objetivos.rs` a través de un helper local: KB
/// con git y su índice poblado a mano.
fn kb_con_indice() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let kb = dir.path().to_path_buf();
    let cfg = kb.join("gitconfig-vacio");
    std::fs::write(&cfg, "").unwrap();
    let git = |args: &[&str]| {
        let s = Command::new("git")
            .arg("-C")
            .arg(&kb)
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
        assert!(s.status.success(), "git {args:?}");
    };
    std::fs::create_dir_all(kb.join("log")).unwrap();
    std::fs::write(
        kb.join("log/alpha.md"),
        "---\ntier: stable\n---\n# alpha\ncuerpo de alpha\n",
    )
    .unwrap();
    git(&["init", "-q"]);
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "inicial"]);

    let db = dir.path().join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES ('kb/log/alpha', 'log/alpha.md', 'alpha', 'note', 0.0, NULL)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO notas_fts (titulo, cuerpo, permalink)
         VALUES ('alpha', 'cuerpo de alpha', 'kb/log/alpha')",
        [],
    )
    .unwrap();
    drop(conn);
    (dir, db)
}

#[test]
fn el_envelope_lleva_command_targets_y_schema_version_2() {
    let (dir, db) = kb_con_indice();
    let salida = Command::new(bin())
        .args(["targets", "--json", "--limit", "5"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("alpha")
        .output()
        .unwrap();

    assert!(salida.status.success(), "stderr: {}", String::from_utf8_lossy(&salida.stderr));
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(v["schema_version"], 2);
    assert_eq!(v["command"], "targets");
    assert_eq!(v["data"]["topic"], "alpha");
    let c = &v["data"]["candidates"][0];
    assert_eq!(c["permalink"], "kb/log/alpha");
    assert_eq!(c["tier"], "stable");
    assert_eq!(c["last_commit"], "2026-07-01T10:00:00+02:00");
    assert!(c["size_bytes"].as_i64().unwrap() > 0);
}

// Las claves de data van en inglés (D8) y las colecciones vacías serializan
// como `[]`, nunca como `null`: un consumidor que haga `.candidates[]` con jq
// no puede encontrarse un null.
#[test]
fn sin_candidatas_el_array_es_vacio_no_nulo() {
    let (dir, db) = kb_con_indice();
    let salida = Command::new(bin())
        .args(["targets", "--json"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("zzz-inexistente")
        .output()
        .unwrap();
    assert!(salida.status.success());
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert!(v["data"]["candidates"].is_array());
    assert_eq!(v["data"]["candidates"].as_array().unwrap().len(), 0);
}

#[test]
fn un_limite_cero_falla_sin_ensuciar_stdout() {
    let (dir, db) = kb_con_indice();
    let salida = Command::new(bin())
        .args(["targets", "--json", "--limit", "0"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("alpha")
        .output()
        .unwrap();
    assert!(!salida.status.success());
    assert!(salida.stdout.is_empty(), "stdout tiene que quedar limpio ante error");
}

#[test]
fn un_tema_vacio_falla() {
    let (dir, db) = kb_con_indice();
    let salida = Command::new(bin())
        .args(["targets", "--json"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("   ")
        .output()
        .unwrap();
    assert!(!salida.status.success());
}

#[test]
fn la_salida_humana_nombra_el_permalink() {
    let (dir, db) = kb_con_indice();
    let salida = Command::new(bin())
        .args(["targets"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("alpha")
        .output()
        .unwrap();
    assert!(salida.status.success());
    let texto = String::from_utf8_lossy(&salida.stdout);
    assert!(texto.contains("kb/log/alpha"), "salida: {texto}");
}

// Una DB inexistente no debe crearse como efecto colateral de un typo en
// --db: los comandos de solo lectura comprueban antes de abrir.
#[test]
fn una_db_inexistente_falla_y_no_se_crea() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("no-existe.db");
    let salida = Command::new(bin())
        .args(["targets", "--json"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("alpha")
        .output()
        .unwrap();
    assert!(!salida.status.success());
    assert!(!db.exists(), "un typo en --db no puede crear el fichero");
}
```

- [ ] **Step 2: Correr los tests y verlos fallar**

Run: `cd engine && cargo test --test targets_cli`
Expected: FAIL — clap sale con `unrecognized subcommand 'targets'`, así que
los asserts de `status.success()` revientan.

- [ ] **Step 3: Implementación**

En `engine/src/main.rs`:

**3a.** Añade la variante al enum `Comando` (tras `Recall(ArgsRecall),`):

```rust
    Targets(ArgsTargets),
```

**3b.** Añade el struct de args, junto a los demás `ArgsXxx`:

```rust
#[derive(clap::Args)]
struct ArgsTargets {
    /// Fichero SQLite del índice. Precedencia: flag > $EXO_DB > config.
    #[arg(long)]
    db: Option<PathBuf>,
    /// Raíz de la KB en disco. Precedencia: flag > $EXO_KB > config.
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Máximo de candidatas. Default 10, igual que `kbx targets`.
    #[arg(long = "limit", default_value_t = 10)]
    limite: usize,
    /// Emite el resultado como envelope JSON (spec §4) en stdout.
    #[arg(long)]
    json: bool,
    /// Tema a buscar.
    tema: String,
}
```

`targets` es superficie nueva: sus flags nacen en inglés y **no** llevan alias
español (la ventana de migración 1.0→1.1 es solo para los flags que ya tenían
nombre español).

**3c.** Añade el brazo a `quiere_json` (el `match` exhaustivo sin comodín):

```rust
        Comando::Targets(a) => a.json,
```

**3d.** Añade el brazo a `ejecuta`:

```rust
        Comando::Targets(args) => targets_cmd(args),
```

**3e.** Añade la función del comando, junto a `busca_cmd`:

```rust
fn targets_cmd(args: ArgsTargets) -> Result<()> {
    let db_ruta = resuelve_db(args.db)?;
    if !db_ruta.exists() {
        anyhow::bail!(
            "DB no encontrada: {} — corre `exo index` primero",
            db_ruta.display()
        );
    }
    let kb = resuelve_kb(args.kb)?;
    let conn = exo::abre_db(&db_ruta)?;
    let resultado = exo::objetivos::busca_objetivos(&conn, &kb, &args.tema, args.limite)?;

    if args.json {
        exo::envelope::emite("targets", serde_json::to_value(&resultado)?);
    } else if resultado.candidatos.is_empty() {
        println!("no candidates");
    } else {
        for c in &resultado.candidatos {
            println!(
                "{}\ttier={}\tsize={}\tlast_commit={}\theadings={:?}",
                c.permalink, c.tier, c.tamano_bytes, c.ultimo_commit, c.headings
            );
            println!("\t{}", c.snippet);
        }
    }
    Ok(())
}
```

- [ ] **Step 4: Correr los tests y verlos pasar**

Run: `cd engine && cargo test --test targets_cli`
Expected: PASS, 6 tests.

Run: `cd engine && ./scripts/test-hermetico.sh` (desde la raíz del repo:
`./engine/scripts/test-hermetico.sh`)
Expected: la suite entera en verde, incluidos los tests preexistentes.

Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
Expected: sin salida, exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/main.rs engine/tests/targets_cli.rs
git commit -m "feat(cli): exo targets, el primer verbo del nucleo de kbx en Rust"
```

---

## Task 6: gate de paridad pre-registrado contra el kbx Go

**Files:**
- Create: `docs/superpowers/plans/2026-09-02-g4a-preregistro-targets.md`
- Create: `docs/superpowers/runbooks/2026-09-02-g4a-paridad-targets.md` (el informe, se escribe al correrlo)

**Interfaces:**
- Consumes: el binario `exo` de la Task 5 y el binario `kbx` compilado de `fe46443`.
- Produces: un veredicto PASA/FALLA escrito, y el informe que lo sostiene.

**Esta tarea corre en la máquina Linux, no en W11.** Medido el 2026-09-02:
`go: command not found` en W11, así que kbx no compila donde más falta hace.
La spec ya lo declara en vez de fingir que es reproducible. Los tests de las
Tasks 1-5 sí corren en los tres SO; esto es la red privada de Paul.

- [ ] **Step 1: Escribir el pre-registro ANTES de compilar nada**

Crea `docs/superpowers/plans/2026-09-02-g4a-preregistro-targets.md`:

```markdown
# Pre-registro del gate de paridad de `targets` (G4a)

> Escrito ANTES de compilar el binario Rust y ANTES de ver ningún output suyo.
> Si esta nota se edita después de la primera corrida, deja de ser un
> pre-registro y hay que declararlo aquí.

## Referencia

- **kbx**: `origin/main`, commit `fe46443`, compilado fresco con `make install`
  en la máquina Linux antes de la corrida. Se añade `-ldflags` con el commit al
  Makefile de kbx para que el binario sepa decir qué es: hoy `kbx` no tiene
  flag de versión, así que "el kbx instalado" no es evidencia falsable.
- **exo**: el binario de esta rama, `cargo build --release`.
- **Índice**: una copia de solo lectura de `~/.exo/index.db` en `/tmp/g4a/`.
  Ningún índice vivo se toca.
- **KB**: `wisdom-paul`, el árbol real.

## Qué se compara y qué NO

`targets` **no es byte-comparable** y se compara **como conjunto, no como
secuencia**: ordena por rank bm25 **sin tie-break** (`ORDER BY rank`), y dos
bindings distintos de SQLite (mattn/go-sqlite3 vs rusqlite bundled) pueden
ordenar los empates de forma distinta, con `snippet()` divergiendo en los
bordes de tokenización.

No se comparan: el orden, el score, ni el texto del `snippet`.
Sí se comparan: el **conjunto de permalinks**, y por cada permalink presente en
ambos, los campos `tier`, `size_bytes` y `last_commit`, que salen de disco y de
git y **tienen que coincidir exactamente**.

## Los topics

Los mismos cinco del pre-registro de M6-04, para que las dos medidas sean
comparables entre sí: un término técnico, un nombre propio, una palabra de
dominio, un acrónimo y una consulta multipalabra.

1. `indexer`
2. `reflex`
3. `memoria`
4. `kbx`
5. `recall en el punto de uso`

## Criterio

Por topic, con `--limit 10` en los dos binarios:

- **PASA** si el conjunto de permalinks es **idéntico**, y si para cada
  permalink `tier`, `size_bytes` y `last_commit` coinciden exactamente.
- Una diferencia de conjunto se explica por empate de rank en la frontera del
  límite (un permalink en la posición 10 de uno y la 11 del otro). **Esa es la
  única explicación admisible.** Cualquier otra diferencia es un FALLO del
  port, no una diferencia aceptable.
- Una diferencia en `tier`, `size_bytes` o `last_commit` es **siempre** un
  fallo: esos tres campos no dependen del binding de SQLite.
- El gate global pasa si **pasan los cinco topics**. Con cuatro o menos, el
  port se investiga antes de seguir con G4b.
```

- [ ] **Step 2: Compilar las dos referencias**

```bash
cd ~/…/kbx && git fetch origin && git checkout fe46443 && make install
kbx --version   # debe imprimir fe46443 tras añadir -ldflags al Makefile
cd ~/…/exo/engine && cargo build --release
```

Expected: los dos binarios existen y `kbx --version` dice `fe46443`.

- [ ] **Step 3: Correr los cinco topics contra la misma DB**

```bash
mkdir -p /tmp/g4a && cp ~/.exo/index.db /tmp/g4a/index.db
KB=~/…/wisdom-paul
for t in "indexer" "reflex" "memoria" "kbx" "recall en el punto de uso"; do
  kbx targets --db /tmp/g4a/index.db --kb "$KB" --limit 10 --json "$t" \
    | jq -S '.data.candidates | map({permalink, tier, size_bytes, last_commit}) | sort_by(.permalink)' \
    > "/tmp/g4a/go-$(echo "$t" | tr ' ' '-').json"
  ./target/release/exo targets --db /tmp/g4a/index.db --kb "$KB" --limit 10 --json "$t" \
    | jq -S '.data.candidates | map({permalink, tier, size_bytes, last_commit}) | sort_by(.permalink)' \
    > "/tmp/g4a/rs-$(echo "$t" | tr ' ' '-').json"
  diff -u "/tmp/g4a/go-$(echo "$t" | tr ' ' '-').json" "/tmp/g4a/rs-$(echo "$t" | tr ' ' '-').json" \
    && echo "PASA: $t" || echo "REVISAR: $t"
done
```

Expected: cinco líneas `PASA:`. Cualquier `REVISAR:` se investiga contra el
criterio del pre-registro antes de tocar código.

- [ ] **Step 4: Escribir el informe**

Crea `docs/superpowers/runbooks/2026-09-02-g4a-paridad-targets.md` con: los
dos commits comparados, el comando exacto, el diff de cada topic (o su
ausencia), el veredicto por topic y el veredicto global. Si algún topic falla,
el informe dice **qué** difiere y **por qué**, no solo que difiere.

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/plans/2026-09-02-g4a-preregistro-targets.md \
        docs/superpowers/runbooks/2026-09-02-g4a-paridad-targets.md
git commit -m "docs(g4a): el gate de paridad de targets, pre-registrado y corrido"
```

---

## Los dos planes que siguen — alcance fijado, detalle pendiente

Se escriben cuando G4a haya enseñado el terreno Rust real, en vez de
inventarlos hoy contra un engine que todavía no tiene esta plomería.

### G4b — `budget` + `lint`

Entra: `exo budget` (bytes/tier vs presupuesto, `offenders`, `waived`,
`notier`, y `no_air` portado aparte del diff de `f0d0564`) y `exo lint` (**6**
checks: `duplicate_dir`, `orphan`, `bad_frontmatter`, `root_file`,
`budget_exceeded`, `budget_prose_drift`).

Lo que ya se sabe que hay que decidir:

- **`schema_drift` muere y se declara.** Existía porque kbx y exo eran dos
  binarios contra un schema compartido; con uno solo deja de tener objeto. Hay
  que declararlo porque el gate de paridad compara contra un doctor de 7 tipos.
- **Los nominales 8500/12500/0 se unifican en una sola constante.** En kbx
  están **triplicados** y nada los enlaza. Pero `doctor.budgetExceededFindings`
  es hoy una **reimplementación duplicada** del switch de `budget.Run`, con un
  test de paridad dedicado que la vigila: al unificarlas en Rust hay que portar
  los tests de paridad de **los dos lados**, no de uno.
- **La guarda NULL del orphan-check es load-bearing**:
  `WHERE destino_permalink IS NOT NULL`. Sin ella, `NOT IN` sobre un subquery
  con NULL devuelve **cero huérfanos en verde**. Medido en kbx: 0 sin guarda,
  7 con. El test tiene que auto-invalidarse si el fixture no contiene ninguna
  arista sin resolver.
- **`walker::walk_kb` no vale tal cual**: excluye `.claude/`, `.omc/` y
  `.superpowers/` e **incluye** `archive/`, mientras que kbx excluye
  `archive`, `.superpowers` y `docs`. Hace falta una exclusión parametrizable,
  y hay que decidir entre las dos semánticas de kbx, que tampoco coinciden
  entre sí (`budget` compara el basename en cada nivel del walk, `doctor` solo
  el primer segmento de la ruta relativa).
- Dos inconsistencias de kbx que el port tiene que resolver a conciencia en
  vez de heredar: `.md` case-insensitive en `budget` pero case-sensitive en
  `doctor`, y `root_file` como único check que ignora la lista de exclusión.
- `budget` y `lint` gatean, así que aquí es donde se ejerce por primera vez la
  decisión de **exit 3**.

### G4c — `ratchet` + cutover

Entra: `exo ratchet` (`--seal`, `--staged`, abstención por shallow-clone, ancla
de activación, absolución de renames, guarda de aire, y los nueve invariantes
de la spec) y el rewiring de los consumidores a verbos `exo`.

Lo que ya se sabe:

- **Se porta desde `fe46443`**, que es el único árbol con la guarda de aire
  (`6332c85`) y con el fix del sello huérfano (`0ae126d`). El checkout local de
  kbx **no tiene ninguno de los dos** y su bypass de rename sigue abierto: un
  port hecho contra `f0d0564` nacería con un agujero ya corregido aguas arriba.
- **Aritmética entera, nunca floats**: `hasAir(ceiling, size) := ceiling*100 >= size*115`
  y `minCeilingFor(size) := (size*115 + 99)/100`. Y ojo con una divergencia que
  Go no tiene: Rust **panica** en overflow de enteros en modo debug y Go hace
  wraparound silencioso — hay que decidir `checked_mul`/`saturating_mul`
  explícitamente en vez de heredar el silencio por accidente.
- **`git show HEAD:./<path>` con el `./` explícito.** Sin él, git resuelve la
  ruta contra la raíz del repo y no contra el `-C`: si la KB vive en un
  subdirectorio, los sellos de HEAD se leen vacíos y cualquier subida pasa en
  verde. Hay un test de kbx dedicado a este bug (A6).
- **`BTreeMap`, no `HashMap`**, para el emparejamiento de renames: en Go el
  orden de iteración del map no está garantizado y hay un test que corre el
  emparejamiento 20 veces para detectarlo. Aquí se puede subir el listón en vez
  de replicar la incertidumbre.
- **`--no-renames` explícito** en `git diff --cached`: hoy kbx depende de la
  config `diff.renames` del entorno, que no es determinista entre máquinas.
- **Las claves del sello son `String` con `/` literal**, y solo se convierten a
  `PathBuf` en el punto de tocar disco. En Go, `filepath.Join` acepta `/` en
  Windows y eso enmascara la cuestión; en Rust hay que decidirlo.
- **H13**: con `--seal` dentro, la doctrina de kbx «read-only salvo `rotate`»
  pasa a «salvo `rotate` y `--seal`». Escrito, no asumido.
- **El cutover** es lo que cierra la ventana H8: `kb-precommit.sh` (5
  referencias a `kbx`, y su degradación es «binario ausente ⇒ commit
  permitido»), `distill/SKILL.md` (11), `document/SKILL.md` (3),
  `document/routing.md` (1), `agents/executor.md` (1). Las **13 rutas absolutas
  `/home/paul/…`** que la spec asignaba a G4 **ya no existen**: las limpió el
  privacy-pass B1 el 09-01. Quedan 2 en `scripts/test-git-c-bash.sh`.
- `exo:distill` degrada de forma **anunciada** sin `rotate`: detecta la
  ausencia y lo dice en una línea visible. Aquí se relaja el patrón habitual
  (`distill` falla-fuerte) porque el fallo-fuerte deja la skill inservible en
  Windows, que es el estado que esta ola viene a arreglar.

---

## Residuo declarado

**El repo kbx queda divergido y esto no lo arregla.** `main` local (`f0d0564`,
el detector `no_air` del 09-02) está 1 commit por delante y 18 por detrás de
`origin/main`, con conflicto en `budget.go`. Reconciliarlo exige toolchain Go
para compilar y testear la resolución, y en W11 no lo hay. No bloquea G4 —el
port toma `fe46443` como fuente y trae `no_air` aparte— pero ese commit sigue
sin publicar, y conviene decidir en la máquina Linux si se reconcilia o si se
deja morir cuando `exo` sustituya a `kbx`.
