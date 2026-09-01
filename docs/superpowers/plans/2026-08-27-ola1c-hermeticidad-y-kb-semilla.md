# Ola 1C — Hermeticidad de tests + KB semilla (G3) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: usa `exo:orchestrate`
> para ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`)
> para tracking.

**Goal:** dejar la suite de tests corriendo en verde sin `~/.exo/config.toml`
(runner limpio) y, sobre esa base, entregar la KB semilla clean-room de 12
ficheros que `exo init` vuelca para un tercero.

**Architecture:** dos pistas **independientes** que convergen en un único
punto. La Pista A introduce `engine/tests/common/mod.rs`, un helper que monta
una config temporal y apunta `EXO_CONFIG` a ella bajo un candado de proceso;
las 9 suites que hoy dependen del `$HOME` de Paul pasan a usarlo. La Pista B
añade `engine/kb-template/` (contenido) y `engine/src/plantilla.rs`
(`include_str!` fichero a fichero), y extiende `init_cmd` para volcar la
plantilla, versionarla e indexarla.

> **Corregido tras el gate del consultor (2026-08-27).** La v1 de este plan
> afirmaba que el gate de bytes de la Task 6 «se escribe con el helper de la
> Pista A, y por eso las fases van en serie». **Era falso**: ese test usa
> `include_str!` y no toca el helper — la propia sección Interfaces de la
> Task 6 lo admitía. El único acoplamiento real es el **gate de cierre**
> (Task 8, Step 7), que corre `scripts/test-hermetico.sh` y por tanto exige la
> Pista A terminada. Todo lo demás es paralelo.

**Tech Stack:** Rust (edición 2024), `anyhow`, `clap`, `tempfile`, `toml`,
`serde_json`, SQLite vía el binding ya presente, `git2` o `std::process::Command`
para el `git init` (ver Task 8).

## Global Constraints

Valores copiados verbatim de `docs/superpowers/specs/2026-08-26-exo-generico-design.md`
(§G3, D3, D4, D6, D7). Toda tarea los hereda:

- **Directorios en inglés** — son el contrato de la KB. **Nombres de nota en
  español** — son contenido, y se convierten en permalink y en título que el
  usuario lee.
- **`{{KB_NAME}}`** es el único placeholder, y es identificador (D7).
- Las 5 notas doctrinales van **reescritas, no copiadas de `kb-demo`**:
  sin nombres, sin proyectos, sin fechas de la historia de Paul. Método
  **clean-room (whitelist)**: se escribe desde cero mirando la instancia solo
  como referencia de forma. Nunca "destilar quitando lo personal".
- Frontmatter **`semilla: true`** en las notas de la plantilla, para que un
  usuario pueda barrerlas con un `grep`.
- **Gate de presupuesto:** el `core-index.md` de la semilla debe caber bajo
  6.144 B **con el 15% de aire**, es decir **≤ 5.222 B**.
- **Distribución de la plantilla:** `include_str!` fichero a fichero en un
  módulo `plantilla.rs`. Explícito, sin macro-crate, binario autosuficiente
  (requisito directo de D4).
- **`exo init [--kb <ruta>] [--name <n>] [--from-basic-memory] [--force]`** —
  manda el **flag**, no el posicional.
- `init` falla si `<ruta>` existe y **no está vacía**, salvo `--force`.
- `init` **no pisa** `~/.exo/config.toml` si existe; pide `--force`.
- Contenido y prosa **en español** (D6); identificadores, verbos de CLI y
  claves de config **en inglés** (D7).

## Orquestación

Dos pistas en paralelo, un solo punto de encuentro:

```
Pista A (hermeticidad):  T1 → T2a → T2b → T3 → T4 ─┐
                                                    ├─→ T8 Step 7 (gate de cierre)
Pista B (G3):            T5 → T6 → T7 → T8 (1-6) ──┘
```

- **Dentro de cada pista, el orden es estricto**: T2a/T2b/T3 consumen el helper
  de T1; T7 consume el árbol de T5+T6; T8 consume `plantilla::vuelca` de T7.
- **Entre pistas no hay dependencia** hasta el Step 7 de la Task 8.
- **Ficheros disjuntos**, así que las dos pistas no compiten por el árbol:
  Pista A toca solo `engine/tests/` + `engine/scripts/` + `docs/backlog.md`;
  Pista B toca solo `engine/kb-template/`, `engine/src/{plantilla,inicia,main}.rs`
  y `engine/tests/{plantilla,plantilla_presupuesto,inicia}.rs`. El único
  fichero compartido es `engine/src/lib.rs` (T7 añade una línea) y ninguna
  tarea de la Pista A lo toca.
- **T6 lleva gate humano** (revisión de Paul): la Pista B se para ahí hasta
  que llegue el OK, y eso no bloquea a la Pista A.

## Línea base medida (2026-08-27, esta máquina)

Todo número de "Expected" en este plan sale de estas dos corridas, no del
backlog (que decía 7 suites / 59 tests — **ya no es cierto**):

```
$ cargo test --release --no-fail-fast                      # con config
CARGO_EXIT_BASELINE=0        169 tests passed, 0 suites en rojo

$ EXO_CONFIG=/ruta/que/no/existe.toml \
  cargo test --release --no-fail-fast                      # runner limpio
CARGO_EXIT=101               9 suites, 61 tests en rojo
```

Reparto exacto de los 61, y causa única (`no encuentro la config de exo`,
58 veces; **57** de ellas por la cadena `leer config de embeddings` y 1 por
`embed de la query`):

| Suite | Rojos | Verdes |
|---|---|---|
| `indexer` | 19 | 0 |
| `buscador` | 16 | 2 |
| `recall_contenido` | 7 | 0 |
| `guarda_modelo` | 5 | 0 |
| `recall` | 5 | 0 |
| `refresca` | 4 | 0 |
| `cache_embeddings` | 3 | 0 |
| `rechazo_envelope` | 1 | 3 |
| `write_create_permalink` | 1 | 0 |
| **Total** | **61** | |

El cuello es de producción, no de los tests: solo cuatro sitios leen config
global — `src/indexer.rs:99`, `src/lib.rs:200`, `src/lib.rs:286`
(`config_embeddings()`) y `src/buscador.rs:236`
(`min_similitud_de_config()`). Ninguna de las 9 suites llama a config
directamente; la heredan por esos cuatro. **Este plan no cambia esas firmas**
—inyectar la config por parámetro es un refactor de producción con su propio
blast radius y va en su propio bloque—; hermetiza por el entorno, que es la
acción que el backlog ya adjudicó y el patrón que
`engine/tests/config_cableado.rs` ya estableció.

---

# Pista A — Hermeticidad

### Task 1: Helper compartido `tests/common/mod.rs` + las dos suites de 1 test

**Files:**
- Create: `engine/tests/common/mod.rs`
- Modify: `engine/tests/write_create_permalink.rs`
- Modify: `engine/tests/rechazo_envelope.rs`

**Interfaces:**
- Consumes: `exo::config::Embeddings`, `exo::MODELO_JINA_ES` (ya públicos).
- Produces, para las Tasks 2, 3 y 7:
  - `pub fn render_config(kb: &Path, nombre: &str, db: &Path) -> String`
  - `pub fn con_config<T>(kb: &Path, nombre: &str, db: &Path, f: impl FnOnce() -> T) -> T`
  - `pub const MODELO: &str` — el mismo valor que `exo::MODELO_JINA_ES`
  - `pub const DIMS: usize = 768`

- [x] **Step 1: Escribir el helper**

`engine/tests/common/mod.rs`:

```rust
//! Config temporal para los tests que dependen de `config_embeddings()` o de
//! `min_similitud_de_config()`. Sin esto, 61 tests en 9 suites leen el
//! `~/.exo/config.toml` de la máquina que los corre y mueren en un runner
//! limpio (medido 2026-08-27: `CARGO_EXIT=101`).
//!
//! Cada binario de integración es un PROCESO propio, así que una instancia de
//! `ENTORNO` por binario es exactamente el alcance correcto: serializa los
//! tests de ese fichero y no toca a los demás.

#![allow(dead_code)] // cada suite usa un subconjunto; sin esto, warnings por fichero

use std::io::Write;
use std::path::Path;

/// El entorno es global al proceso y cargo corre los tests de un binario en
/// hilos paralelos. Todo test que toque `EXO_CONFIG` toma este candado durante
/// toda su vida. `unwrap_or_else(|e| e.into_inner())` porque un test que panica
/// con el candado tomado envenena el mutex, y eso convertiría un fallo en una
/// cascada que tapa al original.
static ENTORNO: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Mismo modelo y dimensionalidad que produce `exo init` en producción: las
/// suites que indexan comparan contra `meta`, y un modelo distinto dispararía
/// la guarda de `guarda_modelo` y convertiría un test de otra cosa en un
/// fallo de guarda.
/// **No dupliques el literal**: `repo_hf` en `lib.rs` solo pinea la revisión
/// del modelo si la cadena coincide EXACTAMENTE con esta constante, y un
/// literal que divergiera en silencio destinaría a `main` (móvil). Mismo
/// criterio que `init_cmd` documenta en `src/main.rs:393-397`.
pub const MODELO: &str = exo::MODELO_JINA_ES;
pub const DIMS: usize = 768;
pub const MIN_SIMILARITY: f64 = 0.35;

/// Renderiza un `config.toml` válido. Barras normales: el TOML las acepta en
/// Windows y evita el escapado de `\` que rompería el fichero.
pub fn render_config(kb: &Path, nombre: &str, db: &Path) -> String {
    format!(
        "schema_version = 1\n\n\
         [kb]\npath = \"{}\"\nname = \"{}\"\n\n\
         [index]\ndb = \"{}\"\n\n\
         [embeddings]\nmodel = \"{MODELO}\"\ndims = {DIMS}\nmin_similarity = {MIN_SIMILARITY}\n",
        kb.display().to_string().replace('\\', "/"),
        nombre,
        db.display().to_string().replace('\\', "/"),
    )
}

/// Corre `f` con `EXO_CONFIG` apuntando a una config temporal que describe
/// `kb`/`nombre`/`db`, y **restaura el valor previo** al salir.
///
/// Restaurar, no borrar: en el runner hermético `EXO_CONFIG` viene puesto
/// desde FUERA para todo el proceso (`scripts/test-hermetico.sh`). Un
/// `remove_var` dejaría al resto del binario sin ese centinela, y cualquier
/// test posterior que leyera config caería al `~/.exo/config.toml` de la
/// máquina: verde aquí, rojo en el CI limpio. Es el modo de fallo exacto que
/// esta fase existe para matar. (`tests/config_cableado.rs` ya tiene el
/// defecto; este helper no lo hereda.)
pub fn con_config<T>(kb: &Path, nombre: &str, db: &Path, f: impl FnOnce() -> T) -> T {
    let _guarda = ENTORNO.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().expect("tempdir de config");
    let ruta = dir.path().join("config.toml");
    let mut fh = std::fs::File::create(&ruta).expect("crear config.toml");
    fh.write_all(render_config(kb, nombre, db).as_bytes())
        .expect("escribir config.toml");
    let previo = std::env::var_os("EXO_CONFIG");
    unsafe { std::env::set_var("EXO_CONFIG", &ruta) };
    let r = f();
    unsafe {
        match previo {
            Some(v) => std::env::set_var("EXO_CONFIG", v),
            None => std::env::remove_var("EXO_CONFIG"),
        }
    }
    r
}
```

- [x] **Step 2: Verificar que las dos suites fallan HOY sin config**

```bash
cd engine
EXO_CONFIG=/tmp/no-existe.toml cargo test --release --no-fail-fast \
  --test write_create_permalink --test rechazo_envelope
```

Expected: FAILED. `write_create_permalink` 0 passed / 1 failed,
`rechazo_envelope` 3 passed / 1 failed. El mensaje debe citar
`no encuentro la config de exo`. Si cita otra cosa, **para**: la causa no es
la que este plan asume.

- [x] **Step 3: Cablear las dos suites**

En cada fichero, añadir arriba del todo:

```rust
mod common;
```

Envolver el cuerpo del test rojo en `common::con_config(...)`. En
`write_create_permalink.rs`, el test rojo es
`write_append_create_usa_el_name_de_la_config_no_el_basename_del_dir_kb`
(panica en `tests/write_create_permalink.rs:76`): su KB temporal y su DB ya
existen en el test — pásalas al helper, y usa como `nombre` el valor que el
test ya afirma (**no** el basename del directorio: eso es justo lo que el test
existe para desmentir).

- [x] **Step 4: Correr y verificar verde SIN config**

```bash
cd engine
EXO_CONFIG=/tmp/no-existe.toml cargo test --release --no-fail-fast \
  --test write_create_permalink --test rechazo_envelope
echo "EXIT=$?"
```

Expected: `EXIT=0`, ambas suites `ok`, 1 + 4 tests passed.

> **No canalices esto por una tubería** (`| tail`, `| grep`): el exit code que
> lee la shell es el del último comando de la tubería, no el de cargo. Ese
> error se cometió midiendo esta misma deuda el 2026-08-27 y dio un falso
> verde.

- [x] **Step 5: Verificar que no rompe CON config**

```bash
cd engine && cargo test --release --test write_create_permalink --test rechazo_envelope
echo "EXIT=$?"
```

Expected: `EXIT=0`. La hermeticidad no puede costar los verdes que ya había.

- [x] **Step 6: Commit**

```bash
git add engine/tests/common/mod.rs engine/tests/write_create_permalink.rs engine/tests/rechazo_envelope.rs
git commit -m "test(engine): helper de config temporal y hermetizadas las dos suites de un test"
```

---

### Task 2a: Hermetizar `indexer` (19 de los 61)

**Files:**
- Modify: `engine/tests/indexer.rs`

**Interfaces:**
- Consumes: `common::con_config`, `common::MODELO`, `common::DIMS` (Task 1).
- Produces: nada nuevo.

- [x] **Step 1: Confirmar el rojo de partida**

```bash
cd engine
EXO_CONFIG=/tmp/no-existe.toml cargo test --release --test indexer
```

Expected: `indexer` 0 passed / **19 failed**.

- [x] **Step 2: Cablear la suite**

`mod common;` en cabecera. Cada test que construya una KB temporal y una DB
temporal envuelve su cuerpo en
`common::con_config(kb.path(), "kb-test", &db, || { ... })`.

- [x] **Step 3: Verde sin config, y verde con config**

```bash
cd engine
EXO_CONFIG=/tmp/no-existe.toml cargo test --release --test indexer; echo "SIN=$?"
cargo test --release --test indexer; echo "CON=$?"
```

Expected: `SIN=0` y `CON=0`, 19 passed en ambas.

> **No canalices esto por una tubería** (`| tail`, `| grep`): el exit code que
> lee la shell es el del último comando de la tubería, no el de cargo. Ese
> error se cometió midiendo esta misma deuda el 2026-08-27 y dio un falso
> verde.

- [x] **Step 4: Commit**

```bash
git add engine/tests/indexer.rs
git commit -m "test(engine): hermetizada indexer contra la config global"
```

---

### Task 2b: Hermetizar `buscador` (16 de los 61)

**Files:**
- Modify: `engine/tests/buscador.rs`

**Interfaces:**
- Consumes: `common::con_config`, `common::MIN_SIMILARITY` (Task 1).
- Produces: nada nuevo.

- [x] **Step 1: Confirmar el rojo de partida**

```bash
cd engine
EXO_CONFIG=/tmp/no-existe.toml cargo test --release --test buscador
```

Expected: `buscador` 2 passed / **16 failed**.

- [x] **Step 2: Cablear la suite**

`mod common;` en cabecera, mismo envoltorio que la Task 2a.

`buscador` tiene además un matiz semántico: los 16 rojos vienen de
`src/buscador.rs:236`, que cae a `min_similitud_de_config()` cuando el
parámetro es `None`. El helper sirve `MIN_SIMILARITY = 0.35`, **idéntico al
valor de la config real de esta máquina**, así que el envoltorio no cambia
ningún valor efectivo hoy. Si algún test dependía del valor de la máquina,
**fíjalo explícito en el test** en vez de heredarlo — es la diferencia entre
un test que pasa y un test que dice algo.

- [x] **Step 3: Verde sin config, y verde con config**

```bash
cd engine
EXO_CONFIG=/tmp/no-existe.toml cargo test --release --test buscador; echo "SIN=$?"
cargo test --release --test buscador; echo "CON=$?"
```

Expected: `SIN=0` y `CON=0`, 18 passed en ambas.

- [x] **Step 4: Commit**

```bash
git add engine/tests/buscador.rs
git commit -m "test(engine): hermetizada buscador contra la config global"
```

---

### Task 3: Hermetizar las cinco suites restantes (24 de los 61)

**Files:**
- Modify: `engine/tests/recall.rs`
- Modify: `engine/tests/recall_contenido.rs`
- Modify: `engine/tests/guarda_modelo.rs`
- Modify: `engine/tests/refresca.rs`
- Modify: `engine/tests/cache_embeddings.rs`

**Interfaces:**
- Consumes: `common::con_config` (Task 1).
- Produces: nada nuevo.

- [x] **Step 1: Confirmar el rojo de partida**

```bash
cd engine
EXO_CONFIG=/tmp/no-existe.toml cargo test --release --no-fail-fast \
  --test recall --test recall_contenido --test guarda_modelo --test refresca --test cache_embeddings
```

Expected: 5 + 7 + 5 + 4 + 3 = 24 failed, 0 passed en las cinco.

> **`--no-fail-fast` es obligatorio aquí, no cosmético.** Sin él, cargo aborta
> tras el PRIMER binario que falla y las otras cuatro suites no llegan a
> ejecutarse: verías un rojo parcial y creerías que el reparto del plan está
> mal. Lo detectó el ejecutor de la Task 1 midiendo su propio Step 2, donde el
> comando tenía el mismo defecto.

> El total del plan (61) sale de 1+1 (Task 1) + 19 (Task 2a) + 16 (Task 2b)
> + 24 (Task 3).

- [x] **Step 2: Cablear las cinco**

Mismo patrón. Dos avisos concretos:

- `guarda_modelo.rs` documenta en su cabecera (`:15`) que "`config_embeddings()`
  lee `$HOME` global y **no es inyectable**; cambiar `$HOME` en el test
  redescargaría el modelo". Con `EXO_CONFIG` eso deja de ser cierto: **actualiza
  ese comentario**, o quedará mintiendo sobre el código de al lado.
- `guarda_modelo` valida que un modelo distinto aborta. Sus tests que esperan
  el camino de rechazo necesitan una config con **otro** modelo: usa
  `render_config` y sustituye el modelo a mano en la cadena, o siembra `meta`
  como ya hace hoy. No cambies `common::MODELO`.

- [x] **Step 3: Verde sin config**

```bash
cd engine
EXO_CONFIG=/tmp/no-existe.toml cargo test --release --no-fail-fast \
  --test recall --test recall_contenido --test guarda_modelo --test refresca --test cache_embeddings
echo "EXIT=$?"
```

Expected: `EXIT=0`, 24 passed, 0 failed.

- [x] **Step 4: La suite ENTERA, sin config — el gate de la fase**

```bash
cd engine
EXO_CONFIG=/tmp/no-existe.toml cargo test --release --no-fail-fast
echo "CARGO_EXIT=$?"
```

Expected: `CARGO_EXIT=0`, **169 passed, 0 failed** — el mismo número que la
línea base con config. Cualquier cifra menor que 169 significa que un test se
perdió por el camino, no que "ya pasa".

- [x] **Step 5: Commit**

```bash
git add engine/tests/recall.rs engine/tests/recall_contenido.rs engine/tests/guarda_modelo.rs engine/tests/refresca.rs engine/tests/cache_embeddings.rs
git commit -m "test(engine): la suite entera corre sin ~/.exo/config.toml"
```

---

### Task 4: Gate falsable contra la regresión

**Files:**
- Create: `engine/scripts/test-hermetico.sh`
- Modify: `docs/backlog.md` (cerrar el item Alta con evidencia)

**Interfaces:**
- Consumes: nada de tareas previas (invoca `cargo` directamente).
- Produces: `engine/scripts/test-hermetico.sh` — exit 0 si la suite corre sin
  config; lo consumirá el CI de G5.

- [x] **Step 1: Escribir el gate**

`engine/scripts/test-hermetico.sh`:

```bash
#!/usr/bin/env bash
# Gate: la suite tiene que correr sin `~/.exo/config.toml`. Sin esto, el CI de
# G5 en un runner limpio nace rojo y nadie se entera hasta que el runner existe.
#
# Apunta EXO_CONFIG a un fichero inexistente en vez de mover el config real:
# mover el de la máquina es destructivo y compite con el hook `Stop` que indexa.
set -uo pipefail
cd "$(dirname "$0")/.."

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# Sin tubería: el exit code de una tubería es el del ÚLTIMO comando, no el de
# cargo. Ese error dio un falso verde midiendo esta misma deuda (2026-08-27).
EXO_CONFIG="$TMP/no-existe.toml" cargo test --release --no-fail-fast > "$TMP/out.txt" 2>&1
EC=$?

if [ "$EC" -ne 0 ]; then
  echo "test-hermetico: la suite NO corre sin ~/.exo/config.toml (exit $EC)." >&2
  grep -E '^test result: FAILED|targets failed|--test ' "$TMP/out.txt" >&2
  exit 1
fi
echo "test-hermetico: OK — la suite corre sin config global."
```

- [x] **Step 2: Verificar que el gate DETECTA el fallo (ciclo red-green)**

No basta con verlo pasar. Revierte una suite de verdad y exige que grite.

> **`git stash push` NO sirve aquí.** En este punto el árbol está limpio: la
> hermetización de `indexer.rs` se commiteó en la Task 2a. `git stash push`
> con pathspec sobre un fichero sin cambios locales es un **no-op** («No local
> changes to save»), la suite sigue hermética y el gate devolvería `0` contra
> un Expected de `1`. El paso que existe para demostrar falsabilidad sería el
> que da el falso verde. Hay que revertir el estado **commiteado**.

```bash
cd engine
SHA_PRE="$(git log --format=%H -1 --skip=1 -- tests/indexer.rs)"   # commit anterior al de Task 2a
git show "$SHA_PRE:engine/tests/indexer.rs" > tests/indexer.rs
git diff --stat -- tests/indexer.rs        # DEBE mostrar cambios; si sale vacío, para
bash scripts/test-hermetico.sh; echo "EXIT_ROJO=$?"

git checkout -- tests/indexer.rs
git diff --quiet -- tests/indexer.rs && echo "restaurado OK"
bash scripts/test-hermetico.sh; echo "EXIT_VERDE=$?"
```

Expected: el `git diff --stat` **no vacío** (prueba de que la reversión mordió),
`EXIT_ROJO=1` citando `--test indexer`, `restaurado OK`, y `EXIT_VERDE=0`. Sin
el rojo verificado no hay gate, hay un script que siempre dice que sí.

- [x] **Step 3: Cerrar el item del backlog con evidencia**

En `docs/backlog.md`, mover el item Alta "La suite de tests no es hermética" a
"Cerrado con evidencia", citando: cifra de partida medida el 2026-08-27
(**9 suites / 61 tests / `CARGO_EXIT=101`** — corregida respecto a las 7/59
que decía el item), cifra final (169 passed, `CARGO_EXIT=0` sin config), y el
ciclo red-green del gate.

- [x] **Step 4: Commit**

```bash
git add engine/scripts/test-hermetico.sh docs/backlog.md
git commit -m "test(engine): gate falsable de hermeticidad y cierre del item del backlog"
```

---

# Pista B — G3: KB semilla + `exo init`

### Task 5: Estructura de `kb-template/` (los 6 ficheros no doctrinales)

**Files:**
- Create: `engine/kb-template/AGENTS.md`
- Create: `engine/kb-template/README.md`
- Create: `engine/kb-template/learnings/_template.md`
- Create: `engine/kb-template/projects/_template.md`
- Create: `engine/kb-template/log/_template.md`
- Create: `engine/kb-template/archive/log/.gitkeep`
- Create: `engine/scripts/fugas-semilla.sh`

**Interfaces:**
- Consumes: nada.
- Produces, para la Task 7: el árbol `engine/kb-template/` con esos 6 ficheros
  y `{{KB_NAME}}` como único placeholder.
- Produces, para la Task 6: `engine/scripts/fugas-semilla.sh` — sale **0 si
  encuentra** una fuga (0 es malo), 1 si el árbol está limpio.

- [x] **Step 1: Escribir los tres `_template.md`**

Cada uno con frontmatter mínimo válido para el indexer (`permalink`, `title`,
`tags`, `tier`) y `semilla: true`. `tier` por carpeta: `stable` en
`projects/`, `log` en `log/`, `stable` en `learnings/`. Los permalinks llevan
el prefijo `{{KB_NAME}}/`.

- [x] **Step 2: Escribir `AGENTS.md` y `README.md`**

`AGENTS.md`: el contrato de la KB para un agente — qué significa cada carpeta,
qué es `tier`, la regla de oro de routing (canon como delta, bitácora como
append, nota nueva casi nunca). `README.md`: para el humano — qué es esta KB,
cómo se indexa, cómo se busca. Ambos en español, sin una sola referencia a
Paul, a `kb-demo` ni a proyectos concretos.

- [x] **Step 3: Verificar que el placeholder es el único, y que no hay fugas**

```bash
cd engine
grep -rho '{{[A-Z_]*}}' kb-template/ | sort -u
bash scripts/fugas-semilla.sh ; echo "FUGAS_EXIT=$?"
```

Expected: la primera imprime exactamente `{{KB_NAME}}` y nada más.
La segunda **no imprime nada** y da `FUGAS_EXIT=1`.
Un `FUGAS_EXIT=0` es una fuga: párate y reescribe.

Crea `engine/scripts/fugas-semilla.sh` en esta tarea — un grep de 8 tokens no
es un gate, y el consultor lo marcó como débil:

```bash
#!/usr/bin/env bash
# Barrido de fugas de la KB semilla. NO es el gate real —ese es la revisión
# humana— pero sube el suelo. Sale 0 SI ENCUENTRA algo (o sea: 0 es malo).
set -uo pipefail
cd "$(dirname "$0")/.."
PATRON='paul|wisdom|empresa-x|cliente-a|equipo-x|cliente-c|cliente-b|redmine|universidad|lighthouse|spark|cge|solve-it|openwisdom|basic-memory|20[0-9]{2}-[0-9]{2}'
grep -rniE "$PATRON" kb-template/
```

`paul` da falso positivo con «paulatino»: falla hacia el lado seguro, que es el
correcto aquí. El patrón de fechas (`20NN-MM`) existe porque §G3 prohíbe
explícitamente «fechas de la historia de Paul» y ningún token las cazaba.

- [x] **Step 4: Commit**

```bash
git add engine/kb-template/
git commit -m "feat(kb-template): estructura y plantillas de la KB semilla"
```

---

### Task 6: Las 5 notas doctrinales clean-room + gate de bytes

> **Esta tarea es de juicio, no mecánica, y toca la superficie que D1 hace
> pública.** El texto se escribe **desde cero**; `kb-demo` se mira solo
> como referencia de forma. Antes de commitear, gate de revisión de Paul: es
> contenido que va a un repo para terceros y el coste de un miss no es un
> test rojo.

**Files:**
- Create: `engine/kb-template/core/core-index.md`
- Create: `engine/kb-template/core/doctrina.md`
- Create: `engine/kb-template/learnings/orquestador-limpio.md`
- Create: `engine/kb-template/learnings/recon-first.md`
- Create: `engine/kb-template/learnings/fallo-silencioso.md`
- Create: `engine/kb-template/learnings/el-brief-es-el-cuello-de-botella.md`
- Test: `engine/tests/plantilla_presupuesto.rs`

**Interfaces:**
- Consumes: `common::con_config` no hace falta aquí (el test lee ficheros, no
  config), pero el fichero de test vive junto a los demás y sigue sus reglas.
- Produces, para la Task 7: los 5 ficheros `.md` que `plantilla.rs` embebe.

- [x] **Step 1: Escribir el test del gate de bytes ANTES que las notas**

`engine/tests/plantilla_presupuesto.rs`:

```rust
//! El `core-index.md` de la semilla debe caber bajo el cap de 6.144 B CON EL
//! 15% DE AIRE que exige la propia doctrina que la nota predica (≤ 5.222 B).
//! Sin este gate la semilla nace mordiendo su presupuesto el primer día — que
//! es exactamente el estado de la KB de la que sale la doctrina: su
//! `core-index.md` mide **5.355 B**, ya por encima del límite semilla de
//! 5.222 B él solo. (El bloque de arranque completo —core-index más los
//! punteros de actividad git— son 5.921 B sobre un cap de 6.144; medido el
//! 2026-08-27. Son dos cifras distintas: no las confundas.)

const CAP: usize = 6_144;
const AIRE: f64 = 0.15;

fn limite() -> usize {
    (CAP as f64 * (1.0 - AIRE)) as usize // 5_222
}

#[test]
fn el_core_index_semilla_cabe_con_15_por_ciento_de_aire() {
    let bytes = include_str!("../kb-template/core/core-index.md").len();
    assert!(
        bytes <= limite(),
        "core-index semilla: {bytes} B > límite {} B (cap {CAP} con {}% de aire). \
         Retira entradas del índice; NO subas el cap ni comprimas las entradas vivas.",
        limite(),
        (AIRE * 100.0) as u32
    );
}

#[test]
fn el_limite_es_el_declarado_en_la_spec() {
    assert_eq!(limite(), 5_222, "el límite de G3 es 5.222 B literal");
}
```

- [x] **Step 2: Verlo fallar por la razón correcta**

```bash
cd engine && cargo test --release --test plantilla_presupuesto
```

Expected: FAIL de compilación — `include_str!` no encuentra
`../kb-template/core/core-index.md`. Ese es el rojo esperado: el fichero aún
no existe.

- [x] **Step 3: Escribir las 5 notas clean-room**

> **Instrucción vinculante para quien ejecute este paso: NO abras las notas de
> `kb-demo`.** Escribe cada nota **desde el nombre del principio**, con tu
> propio conocimiento del tema. El método de §153 de la spec madre es
> *whitelist*: se escribe desde cero y la instancia se mira solo como
> referencia de **forma**. Un ejecutor que relea las notas origen y les quite
> lo personal está haciendo *blacklist-destilado* sin saberlo — que es
> exactamente lo prohibido, y el modo en que un miss llega al repo público.
>
> Agravante detectado por el consultor: el hook de recall **te habrá inyectado
> ya** punteros y resúmenes de `kb-demo` en el contexto de arranque. No son
> material de partida: ignóralos para esta tarea.

`core/core-index.md` — el mapa: contrato de memoria, presupuestos por tier,
regla de índices, y punteros a las 4 de `learnings/`. Es el que compite con el
límite de bytes.
`core/doctrina.md` — doctrina de trabajo con agentes, genérica.
`learnings/orquestador-limpio.md`, `recon-first.md`, `fallo-silencioso.md`,
`el-brief-es-el-cuello-de-botella.md` — un principio cada una.

Todas con `semilla: true` en frontmatter y permalink `{{KB_NAME}}/...`.

- [x] **Step 4: Gate de bytes en verde + barrido de fugas**

```bash
cd engine
cargo test --release --test plantilla_presupuesto
echo "EXIT=$?"
wc -c kb-template/core/core-index.md
bash scripts/fugas-semilla.sh ; echo "FUGAS_EXIT=$?"
```

Expected: `EXIT=0`, los 2 tests passed, `core-index.md` ≤ 5222 B, y
`FUGAS_EXIT=1` (sin match).

- [x] **Step 5: Gate de revisión de Paul**

Presenta las 5 notas. **No commitees sin su OK**: es la superficie pública.

- [x] **Step 6: Commit**

```bash
git add engine/kb-template/core/ engine/kb-template/learnings/ engine/tests/plantilla_presupuesto.rs
git commit -m "feat(kb-template): núcleo doctrinal clean-room con gate de presupuesto"
```

---

### Task 7: `plantilla.rs` — embeber y volcar

**Files:**
- Create: `engine/src/plantilla.rs`
- Modify: `engine/src/lib.rs` (añadir `pub mod plantilla;`)
- Test: `engine/tests/plantilla.rs`

**Interfaces:**
- Consumes: el árbol `engine/kb-template/` (Tasks 5 y 6).
- Produces, para la Task 8:
  - `pub const FICHEROS: &[(&str, &str)]` — `(ruta_relativa, contenido)`, 12 entradas
  - `pub fn render(contenido: &str, kb_name: &str) -> String`
  - `pub fn vuelca(destino: &Path, kb_name: &str) -> anyhow::Result<Vec<PathBuf>>`

- [x] **Step 1: Test que falla**

`engine/tests/plantilla.rs`:

```rust
use tempfile::TempDir;

#[test]
fn son_doce_ficheros() {
    assert_eq!(exo::plantilla::FICHEROS.len(), 12);
}

#[test]
fn render_sustituye_el_placeholder() {
    let out = exo::plantilla::render("permalink: {{KB_NAME}}/core/core-index", "mi-kb");
    assert_eq!(out, "permalink: mi-kb/core/core-index");
    assert!(!out.contains("{{KB_NAME}}"));
}

#[test]
fn vuelca_escribe_los_doce_y_no_deja_placeholders() {
    let dir = TempDir::new().unwrap();
    let escritos = exo::plantilla::vuelca(dir.path(), "mi-kb").expect("volcar");
    assert_eq!(escritos.len(), 12);
    for f in &escritos {
        assert!(f.exists(), "no existe {}", f.display());
        if f.extension().is_some_and(|e| e == "md") {
            let c = std::fs::read_to_string(f).unwrap();
            assert!(!c.contains("{{"), "placeholder vivo en {}", f.display());
        }
    }
    assert!(dir.path().join("archive/log/.gitkeep").exists());
}
```

- [x] **Step 2: Verlo fallar**

```bash
cd engine && cargo test --release --test plantilla
```

Expected: FAIL de compilación — no existe `exo::plantilla`.

- [x] **Step 3: Implementar `plantilla.rs`**

`include_str!` **fichero a fichero**, sin macro-crate (D4):

```rust
//! La plantilla de la KB semilla, embebida en el binario. `include_str!`
//! explícito doce veces en vez de un macro-crate de embedding: D4 exige un
//! binario autosuficiente, y doce líneas legibles valen más que una dependencia
//! que hay que auditar para publicar.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub const PLACEHOLDER: &str = "{{KB_NAME}}";

pub const FICHEROS: &[(&str, &str)] = &[
    ("README.md", include_str!("../kb-template/README.md")),
    ("core/core-index.md", include_str!("../kb-template/core/core-index.md")),
    ("core/doctrina.md", include_str!("../kb-template/core/doctrina.md")),
    ("learnings/_template.md", include_str!("../kb-template/learnings/_template.md")),
    ("learnings/orquestador-limpio.md", include_str!("../kb-template/learnings/orquestador-limpio.md")),
    ("learnings/recon-first.md", include_str!("../kb-template/learnings/recon-first.md")),
    ("learnings/fallo-silencioso.md", include_str!("../kb-template/learnings/fallo-silencioso.md")),
    ("learnings/el-brief-es-el-cuello-de-botella.md", include_str!("../kb-template/learnings/el-brief-es-el-cuello-de-botella.md")),
    ("projects/_template.md", include_str!("../kb-template/projects/_template.md")),
    ("log/_template.md", include_str!("../kb-template/log/_template.md")),
    ("archive/log/.gitkeep", include_str!("../kb-template/archive/log/.gitkeep")),
    ("AGENTS.md", include_str!("../kb-template/AGENTS.md")),
];

pub fn render(contenido: &str, kb_name: &str) -> String {
    contenido.replace(PLACEHOLDER, kb_name)
}

/// Vuelca la plantilla en `destino`. Devuelve las rutas escritas, en el orden
/// de `FICHEROS`, para que el llamante pueda decir qué hizo.
pub fn vuelca(destino: &Path, kb_name: &str) -> Result<Vec<PathBuf>> {
    let mut escritos = Vec::with_capacity(FICHEROS.len());
    for (rel, contenido) in FICHEROS {
        let ruta = destino.join(rel);
        if let Some(padre) = ruta.parent() {
            std::fs::create_dir_all(padre)
                .with_context(|| format!("crear {}", padre.display()))?;
        }
        std::fs::write(&ruta, render(contenido, kb_name))
            .with_context(|| format!("escribir {}", ruta.display()))?;
        escritos.push(ruta);
    }
    Ok(escritos)
}
```

> **Son 12, no 11.** El bloque de §G3 dice "~11 ficheros" y esa cifra es una
> aproximación en prosa, no un conteo: la lista literal de la spec tiene 12
> entradas, y las Tasks 5 y 6 de este plan crean 6 + 6. El test
> `son_doce_ficheros` es el que manda; si al implementar salen 11, falta uno.

Registrar en `engine/src/lib.rs`, en orden alfabético entre `nota` y `recall`:

```rust
pub mod plantilla;
```

- [x] **Step 4: Verde**

```bash
cd engine && cargo test --release --test plantilla
echo "EXIT=$?"
```

Expected: `EXIT=0`, 3 tests passed.

- [x] **Step 5: Commit**

```bash
git add engine/src/plantilla.rs engine/src/lib.rs engine/tests/plantilla.rs
git commit -m "feat(engine): plantilla de la KB semilla embebida en el binario"
```

---

### Task 8: `exo init` vuelca la KB, la versiona y la indexa

**Files:**
- Modify: `engine/src/main.rs:374-425` (`init_cmd`) — dos modos + `EXO_DB` en el index inicial
- Modify: `engine/src/inicia.rs` — `prepara_kb`, y `ruta_basic_memory()` con override `EXO_BASIC_MEMORY_JSON`
- Test: `engine/tests/inicia.rs`

**Interfaces:**
- Consumes: `exo::plantilla::vuelca(&Path, &str) -> Result<Vec<PathBuf>>` (Task 7);
  `exo::inicia::escribe_config(...)` (ya existe, sin cambios).
- Produces:
  - `pub fn prepara_kb(kb: &Path, force: bool) -> anyhow::Result<()>`
  - comportamiento de CLI en dos modos (`create` / `adopt`)
  - dos seams de test: `EXO_DB` honrado por el index de `init`, y
    `EXO_BASIC_MEMORY_JSON` como override de `ruta_basic_memory()`

- [x] **Step 1: Tests que fallan**

Añadir a `engine/tests/inicia.rs`:

```rust
#[test]
fn init_rechaza_un_directorio_no_vacio_sin_force() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(dir.path().join("algo.md"), "x").unwrap();
    let e = exo::inicia::prepara_kb(dir.path(), false).unwrap_err();
    assert!(
        e.to_string().contains("no está vacía"),
        "mensaje inesperado: {e}"
    );
}

#[test]
fn init_acepta_un_directorio_vacio() {
    let dir = tempfile::TempDir::new().unwrap();
    exo::inicia::prepara_kb(dir.path(), false).expect("dir vacío debe pasar");
}

#[test]
fn init_acepta_un_directorio_inexistente() {
    let dir = tempfile::TempDir::new().unwrap();
    let nueva = dir.path().join("kb-nueva");
    exo::inicia::prepara_kb(&nueva, false).expect("dir inexistente debe pasar");
}
```

Y el test que fija el modo ADOPCIÓN — el que impide volver a escribir el bug
que el consultor paró. Va sobre el binario, no sobre `prepara_kb`, porque lo
que se afirma es el comportamiento del comando:

```rust
/// `--from-basic-memory` adopta una KB existente: NO puede escribir ni un
/// byte dentro de ella. Con el cableado de la v1 de este plan, este test
/// fallaba de las dos formas posibles: sin `--force` abortaba, y con `--force`
/// machacaba `core/core-index.md` con la semilla.
#[test]
fn adopcion_no_toca_ni_un_fichero_de_la_kb_existente() {
    let tmp = tempfile::TempDir::new().unwrap();
    let kb = tmp.path().join("kb-poblada");
    std::fs::create_dir_all(kb.join("core")).unwrap();
    let canon = kb.join("core/core-index.md");
    std::fs::write(&canon, "---
permalink: real/core/core-index
---
CONTENIDO REAL
").unwrap();
    let antes = std::fs::read_to_string(&canon).unwrap();

    let bm = tmp.path().join("bm.json");
    std::fs::write(&bm, format!(
        r#"{{"projects":{{"real":{{"path":"{}"}}}},"default_project":"real",
            "semantic_embedding_model":"{}","semantic_embedding_dimensions":768,
            "semantic_min_similarity":0.35}}"#,
        kb.display().to_string().replace('\', "/"),
        exo::MODELO_JINA_ES)).unwrap();

    // Ejecuta el binario en modo adopción con config y db aisladas.
    let salida = std::process::Command::new(env!("CARGO_BIN_EXE_exo"))
        .args(["init", "--from-basic-memory", "--json"])
        .env("EXO_CONFIG", tmp.path().join("config.toml"))
        .env("EXO_DB", tmp.path().join("index.db"))
        .env("EXO_BASIC_MEMORY_JSON", &bm)
        .output()
        .expect("ejecutar exo init");

    assert!(salida.status.success(), "init falló: {}", String::from_utf8_lossy(&salida.stderr));
    assert_eq!(std::fs::read_to_string(&canon).unwrap(), antes, "la adopción escribió en la KB");
    assert!(!kb.join("AGENTS.md").exists(), "la adopción volcó la plantilla");
    assert!(!kb.join(".git").exists(), "la adopción hizo git init");
}
```

> Este test necesita dos seams que hoy no existen y que esta tarea añade:
> `EXO_DB` honrado por el index de `init` (ver Step 6) y `EXO_BASIC_MEMORY_JSON`
> como override de `inicia::ruta_basic_memory()`. Sin el segundo, el test leería
> el `~/.basic-memory/config.json` real y dejaría de ser hermético — justo lo
> que la Pista A existe para erradicar.

- [x] **Step 2: Verlo fallar**

```bash
cd engine && cargo test --release --test inicia
```

Expected: FAIL de compilación — `prepara_kb` no existe.

- [x] **Step 3: Implementar `prepara_kb` en `src/inicia.rs`**

```rust
/// Comprueba que `kb` es un destino legítimo: inexistente, o existente y
/// vacío. Con `force`, cualquiera. Volcar sobre una KB con contenido pisaría
/// notas de alguien sin avisar — el efecto silencioso que este proyecto
/// persigue.
pub fn prepara_kb(kb: &Path, force: bool) -> Result<()> {
    if force || !kb.exists() {
        return Ok(());
    }
    let vacia = std::fs::read_dir(kb)
        .with_context(|| format!("leer {}", kb.display()))?
        .next()
        .is_none();
    if !vacia {
        anyhow::bail!(
            "{} existe y no está vacía — repite con --force si de verdad quieres volcar encima",
            kb.display()
        );
    }
    Ok(())
}
```

- [x] **Step 4: Verde**

```bash
cd engine && cargo test --release --test inicia
echo "EXIT=$?"
```

Expected: `EXIT=0`.

- [x] **Step 5: Cablear `init_cmd` en el orden de §G3**

> **Dos modos, no uno.** `--from-basic-memory` resuelve `kb` a una KB
> **existente y poblada** (en esta máquina, `kb-demo`, 100+ notas). Cablear
> el volcado incondicionalmente rompe ese camino sin `--force` y, **con
> `--force`, sobreescribe `core/core-index.md`, `AGENTS.md` y `README.md` de la
> KB real con la semilla** — y el «primer commit» consagra el pisotón. El flag
> está en la firma de §G3 (`spec:492`): es requisito, no caso de borde.

En `src/main.rs`, dentro de `init_cmd`, tras resolver `(kb, nombre, emb)`:

**Modo ADOPCIÓN (`--from-basic-memory`)** — la KB ya existe y ya tiene
contenido. **No** se llama a `prepara_kb`, **no** se vuelca la plantilla,
**no** se hace `git init` ni commit. Solo:

1. `escribe_config(...)` — como hoy.
2. `exo index` inicial.
3. Salida, declarando que adoptó una KB existente y no escribió nada en ella.

**Modo CREACIÓN (`--kb` + `--name`)** — la KB nace aquí:

1. `exo::inicia::prepara_kb(&kb, args.force)?`
2. `std::fs::create_dir_all(&kb)`
3. `let escritos = exo::plantilla::vuelca(&kb, &nombre)?`
4. `git init` + primer commit vía `std::process::Command` (no `git2`: añadir
   una dependencia con toolchain C contradice D4, que existe para sacar el
   toolchain C del camino del usuario). **Cualquier fallo del paso git avisa
   por stderr y sigue** — no solo «git ausente del PATH»: en una máquina fresca
   de tercero `git init` funciona y `git commit` **falla** sin `user.name` /
   `user.email`, que es el caso más probable del público objetivo de G3. Una KB
   sin git funciona; abortar por eso sería peor.
5. `escribe_config(...)` — como hoy.
6. `exo index` inicial sobre la KB recién volcada.
7. Salida: qué hizo y cuál es el siguiente comando.

El envelope `--json` gana `"files": escritos.len()` (0 en adopción),
`"git": true|false` (false si cualquier paso git falló) y
`"mode": "create"|"adopt"`.

- [x] **Step 6: Prueba end-to-end real, no solo unitaria**

> **`EXO_CONFIG` aísla la config, NO la db.** `init_cmd` escribe la config con
> `db_default = ~/.exo/index.db` (`src/main.rs:376`; `ArgsInit` no tiene flag
> `--db`), así que el «`exo index` inicial» abriría el **índice de producción
> de esta máquina** y metería dentro los 12 ficheros de la semilla — y el
> indexer poda lo que no cuelga de la KB activa (clave `borradas` del
> envelope). El índice real reducido a la semilla, compitiendo con el hook
> `Stop`. Es el mismo efecto colateral que este plan prohíbe al vetar mover
> `~/.exo/config.toml`. **Antes de correr el e2e, haz que el paso de index de
> `init` honre `EXO_DB`**, como ya hace el resto de comandos
> (`src/main.rs:334`).

```bash
cd engine && cargo build --release
TMP=$(mktemp -d)
export EXO_CONFIG="$TMP/config.toml" EXO_DB="$TMP/index.db"
./target/release/exo init --kb "$TMP/kb" --name mi-kb --json
echo "EXIT=$?"

# .git/ tiene decenas de ficheros: excluirlo o el conteo no puede dar 12 nunca.
find "$TMP/kb" -type f -not -path '*/.git/*' | sort | tee "$TMP/lista.txt" | wc -l
git -C "$TMP/kb" log --oneline
grep -rl '{{' "$TMP/kb" --include='*.md' ; echo "PLACEHOLDERS_EXIT=$?"

# El índice real NO se ha tocado:
ls -l "$TMP/index.db" && test ! -s "$HOME/.exo/index.db.lock" && echo "db aislada OK"
```

Expected: `EXIT=0`; el `wc -l` da **12**; un commit en el log;
`PLACEHOLDERS_EXIT=1` (`grep -l` sin match ⇒ ningún fichero con placeholders
vivos); `$TMP/index.db` existe. Y repetir el mismo `init` sobre el mismo
`--kb` debe fallar citando "no está vacía".

- [x] **Step 7: La suite entera, con y sin config**

```bash
cd engine
cargo test --release --no-fail-fast; echo "CON_CONFIG=$?"
bash scripts/test-hermetico.sh; echo "HERMETICO=$?"
```

Expected: `CON_CONFIG=0` con **≥ 178** tests passed (169 de base + 2 de
presupuesto + 3 de plantilla + 4 nuevos de `inicia`, incluido
`adopcion_no_toca_ni_un_fichero_de_la_kb_existente`), y `HERMETICO=0`.

> **Este es el punto de encuentro de las dos pistas.** `scripts/test-hermetico.sh`
> lo produce la Task 4 (Pista A): no ejecutes este step hasta que esa pista
> haya cerrado.

- [x] **Step 8: Commit**

```bash
git add engine/src/main.rs engine/src/inicia.rs engine/tests/inicia.rs
git commit -m "feat(engine): exo init vuelca la KB semilla, la versiona y la indexa"
```

---

## Self-review

- **Cobertura contra §G3:** árbol de 12 ficheros (Tasks 5, 6, 7) · placeholder
  `{{KB_NAME}}` (7) · 5 notas reescritas con `semilla: true` (6) · gate ≤5.222 B
  (6) · `include_str!` fichero a fichero (7) · los 6 pasos de `exo init` (8) ·
  **`--from-basic-memory` con modo de adopción propio y test (8)** · `--kb`
  flag y no posicional (ya en `ArgsInit`, sin cambio) · no pisar config
  existente (ya en `escribe_config`, sin cambio).
- **Cobertura del item Alta del backlog:** las 9 suites reales (Tasks 1, 2a,
  2b, 3), no las 7 que decía el item; gate anti-regresión (4); cierre con
  evidencia (4).
- **Firmas cruzadas:** `common::con_config` (T1) → T2a, T2b, T3.
  `plantilla::vuelca` (T7) → T8. `inicia::prepara_kb` (T8) → nadie.
  `plantilla::FICHEROS` 12 entradas ↔ `son_doce_ficheros`.
  `scripts/fugas-semilla.sh` (T5) → T5 Step 3 y T6 Step 4.
  `scripts/test-hermetico.sh` (T4) → T8 Step 7 (el único cruce de pistas).
- **Corregido en review:** la spec dice "~11 ficheros"; la lista literal de §G3
  tiene 12 y las Tasks 5+6 crean 6+6. El plan usa **12** en todas partes y el
  test `son_doce_ficheros` lo fija. La cifra en prosa de la spec queda como
  aproximación, no como contrato.
- **Tensión declarada, no resuelta aquí (consultor, MENOR-7):** la semilla
  planta `AGENTS.md` y `README.md` en la **raíz** de la KB (§G3), y G4 porta el
  check `root_file` del doctor de kbx (`spec:540-548`). El primer `exo lint` de
  un usuario fresco puede flaggear la propia semilla. Es un conflicto entre dos
  gates de la spec, no un defecto de este plan; se declara ahora para que G4 no
  lo descubra en rojo. Salida esperada: whitelist de la semilla en el check, o
  excepción documentada.
- **Fuera de scope, deliberado:** inyectar la config por parámetro en
  `config_embeddings()` / `min_similitud_de_config()` (refactor de producción,
  bloque propio; el consultor confirmó que además **no cubriría las dos suites
  que arrancan subprocesos**) · el privacy-pass y B1 (gate de publicación, una
  sola pasada de `filter-repo`, no lo toca este plan) · G4 y G5.

## Historial de gates

| Fecha | Gate | Resultado |
|---|---|---|
| 2026-08-27 | Consultor independiente (régimen de gates, spec `2026-07-16`) | **APROBADO CON CAMBIOS** — 3 BLOQUEANTES, 1 MAYOR, 8 MENORES. Verdict: `docs/superpowers/consultas/2026-08-27-ola1c/consultor-plan.md` |
| 2026-08-27 | Aplicación del verdict | Los 4 exigidos y los 8 menores aplicados; fases paralelizadas por decisión de Paul |
| 2026-08-27 | Auditor de la semilla — waiver `learnings/recon-first.md` (m-3) | **Waiver concedido**: se queda con el nombre en inglés, identificador de producto (alinea con el skill publicado `exo:recon-first`). Razonamiento completo en `docs/superpowers/consultas/2026-08-27-ola1c/auditor-semilla.md` (## Resolución del gate) |
