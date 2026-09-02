# G5a — CI mínimo Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: usa `exo:orchestrate`
> para ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`)
> para tracking.

**Goal:** que exo tenga un gate automático que corra su suite completa en
ubuntu / windows / macos más `fmt --check`, `clippy -D warnings` y un check de
la MSRV declarada — y que ese gate se haya visto **rojo** antes de darse por
bueno.

**Architecture:** tres commits en orden no negociable. Primero se pone verde lo
que el gate va a medir (`cargo fmt`, luego los 12 avisos de clippy), y solo
después entra `.github/workflows/ci.yml`. El workflow no inventa comandos de
test: consume `engine/scripts/test-hermetico.sh`, que ya existe, ya está
demostrado falsable (2026-08-27) y corre la suite entera con `EXO_CONFIG`
apuntando a un fichero inexistente. La única pieza nueva de infraestructura es
la caché del modelo ONNX de embeddings entre runs.

**Tech Stack:** GitHub Actions · Rust estable + toolchain 1.95 para el check de
MSRV · `actions/checkout@v4`, `dtolnay/rust-toolchain`, `Swatinem/rust-cache@v2`,
`actions/cache@v4` · bash (Git Bash en el runner de Windows).

## Global Constraints

- **El crate vive en `engine/`, no en la raíz del repo.** No hay workspace de
  Cargo. Todo comando de cargo se ejecuta con cwd `engine/`.
- **MSRV declarada: `rust-version = "1.95"`** en `engine/Cargo.toml`, con nota
  empírica en el propio fichero (1.94 falla porque `libsqlite3-sys` usa
  `cfg_select`). Hoy nada la comprueba; este plan la hace falsable.
- **`.gitattributes` de la raíz fuerza `* text=auto eol=lf`.** El checkout del
  runner de Windows es LF en el árbol de trabajo. No añadir pasos de
  normalización de finales de línea.
- **Modelo de embeddings pineado**: `jinaai/jina-embeddings-v2-base-es`,
  revisión `8e2d780d8fd38f81ca9123ee28e4c5a968aaf21e`
  (`engine/src/lib.rs:149-150`). La caché de `hf-hub` 0.5.0 es `$HF_HOME/hub`
  si la variable está puesta, y si no `~/.cache/huggingface/hub`
  (`Cache::from_env` / `Cache::default`, verificado en la fuente de la crate).
  Layout: `models--jinaai--jina-embeddings-v2-base-es/{blobs,refs,snapshots}`.
  **615 MB medidos** en la caché local el 2026-09-02.
- **Decisión de Paul (2026-09-02): las 9 suites que indexan NO se marcan
  `#[ignore]`.** Se cachea el modelo. Un CI que no ejerce el indexer ni el
  buscador es verde sin significado — el fallo silencioso que este proyecto
  existe para no repetir. La convención `#[ignore]` de
  `engine/tests/smoke.rs:32` se queda donde está y no se extiende.
- **`engine/scripts/test-hermetico.sh` NO se modifica en este plan.** Es un
  gate ya demostrado falsable con un ciclo rojo-verde real; tocarlo invalida esa
  evidencia. El CI lo consume tal cual.
- **Orden no negociable: fmt y clippy verdes ANTES de que entre el workflow.**
  Si el workflow entra primero, el CI nace rojo — exactamente el fallo que el
  ítem 2 de la ola 0 de la spec existe para evitar.
- **Un runner rojo por razón de entorno NO se arregla borrando el job, ni
  añadiendo `continue-on-error`, ni sacando el SO de la matriz.** Se reporta al
  orquestador con el log. Aflojar el gate ante su primer rojo es el
  anti-patrón que este plan tiene prohibido.
- **Cifras de partida, medidas el 2026-09-02 con rustc 1.98.0:**
  `cargo fmt --check` = **11 diffs en 5 ficheros**;
  `cargo clippy --all-targets` = **12 avisos**.
- Fuera de scope, y se declara en vez de disimularse: release workflow,
  `install.sh` / `install.ps1`, `exo doctor` (todo eso es G5b y depende de G4);
  el rename del fixture `kb-demo` en 8 ficheros de test; y meter en CI los 9
  scripts `test-*.sh` de `plugins/exo/scripts/` — **5 de esos 9 referencian
  rutas de esta máquina** (medido: `test-a1-gate.sh`, `test-compose-inject.sh`,
  `test-contrato-engine.sh`, `test-git-c-bash.sh`, `test-subagent-inject.sh`),
  así que llevarlos a CI es campaña propia, no un paso de esta.

---

### Task 1: Pasada de `cargo fmt`

**Files:**
- Modify: `engine/src/plantilla.rs` (diffs en :10, :34)
- Modify: `engine/tests/buscador.rs` (:391)
- Modify: `engine/tests/escritor.rs` (:76)
- Modify: `engine/tests/inicia.rs` (:200, :232)
- Modify: `engine/tests/recall.rs` (:58, :70, :81, :92, :103)

**Interfaces:**
- Consumes: nada.
- Produces: `cargo fmt --check` sale 0. Task 3 depende de ello: el job `lint`
  del workflow corre ese comando exacto.

Ninguno de los ficheros que toca esta tarea contiene un aviso de clippy de la
Task 2 (los de clippy viven en `src/{config,inicia,lib,main,vectores,escritor}.rs`
y `tests/recall_contenido.rs`), así que **no hay deriva de números de línea**
entre las dos tareas.

- [ ] **Step 1: Ver el check en rojo y anotar la cifra de partida**

```bash
cd engine
cargo fmt --check 2>&1 | grep -c "^Diff in"
```

Expected: `11`

- [ ] **Step 2: Aplicar el formateo**

```bash
cd engine
cargo fmt
```

- [ ] **Step 3: Ver el check en verde**

```bash
cd engine
cargo fmt --check; echo "EXIT=$?"
```

Expected: sin salida y `EXIT=0`

- [ ] **Step 4: Confirmar que el cambio es solo de formato y sigue compilando**

```bash
cd engine
git diff --stat
cargo check --all-targets 2>&1 | tail -3
```

Expected: los 5 ficheros de arriba y ninguno más; `cargo check` termina sin
errores (los 12 avisos de clippy NO salen aquí — `cargo check` no corre clippy).

- [ ] **Step 5: Commit**

```bash
git add engine/src/plantilla.rs engine/tests/buscador.rs engine/tests/escritor.rs engine/tests/inicia.rs engine/tests/recall.rs
git commit -m "style: cargo fmt en commit propio, para que el gate no nazca rojo"
```

---

### Task 2: Los 12 avisos de clippy a cero

**Files:**
- Modify: `engine/src/config.rs:66`
- Modify: `engine/src/inicia.rs:166`
- Modify: `engine/src/main.rs:334`, `:348`, `:410-414`
- Modify: `engine/src/lib.rs:29`, `:250`
- Modify: `engine/src/vectores.rs:54`
- Modify: `engine/src/escritor.rs:248`
- Modify: `engine/tests/recall_contenido.rs:4-8`

**Interfaces:**
- Consumes: el árbol formateado de la Task 1.
- Produces: `cargo clippy --all-targets -- -D warnings` sale 0. Task 3 corre ese
  comando exacto en el job `lint`.

- [ ] **Step 1: Ver el gate en rojo con la cifra exacta**

```bash
cd engine
cargo clippy --all-targets -- -D warnings 2>&1 | grep -c "^error: "
```

Expected: `12` (con `-D warnings` cada aviso se emite como `error:`)

- [ ] **Step 2: Los cuatro `collapsible_if`**

`src/config.rs:65-70` — de:

```rust
    if let Some(resto) = s.strip_prefix("~/").or_else(|| s.strip_prefix("~\\")) {
        if let Some(home) = dirs::home_dir() {
            return home.join(resto);
        }
    }
```

a:

```rust
    if let Some(resto) = s.strip_prefix("~/").or_else(|| s.strip_prefix("~\\"))
        && let Some(home) = dirs::home_dir()
    {
        return home.join(resto);
    }
```

`src/inicia.rs:166-170` — de:

```rust
    if let Ok(v) = std::env::var("EXO_BASIC_MEMORY_JSON") {
        if !v.is_empty() {
            return Ok(PathBuf::from(v));
        }
    }
```

a:

```rust
    if let Ok(v) = std::env::var("EXO_BASIC_MEMORY_JSON")
        && !v.is_empty()
    {
        return Ok(PathBuf::from(v));
    }
```

`src/main.rs:334-338` (dentro de `resuelve_db`) — de:

```rust
    if let Ok(v) = std::env::var("EXO_DB") {
        if !v.is_empty() {
            return Ok(exo::config::expande_tilde(Path::new(&v)));
        }
    }
```

a:

```rust
    if let Ok(v) = std::env::var("EXO_DB")
        && !v.is_empty()
    {
        return Ok(exo::config::expande_tilde(Path::new(&v)));
    }
```

`src/main.rs:348-352` (dentro de `resuelve_kb`) — de:

```rust
    if let Ok(v) = std::env::var("EXO_KB") {
        if !v.is_empty() {
            return Ok(exo::config::expande_tilde(Path::new(&v)));
        }
    }
```

a:

```rust
    if let Ok(v) = std::env::var("EXO_KB")
        && !v.is_empty()
    {
        return Ok(exo::config::expande_tilde(Path::new(&v)));
    }
```

Las cadenas `let` en `if` son estables desde Rust 1.88 y el crate está en
`edition = "2024"`: compilan bajo la MSRV declarada (1.95). Si alguna no
compilara, el fallback es anidar al revés (`if !v.is_empty()` fuera) o un
`#[allow(clippy::collapsible_if)]` con justificación escrita — **nunca** dejar
el aviso vivo.

- [ ] **Step 3: Los cuatro `doc_list_item_without_indentation`**

El disparador es el mismo en los dos sitios: una línea de doc-comment que
**empieza por `+ `**, que markdown lee como marca de lista, con las líneas
siguientes sin indentar. El arreglo es reflujo del texto para que el `+` no
caiga a principio de línea — no se cambia ni una palabra del contenido.

`src/main.rs:410-414` — de:

```rust
/// `exo init`: dos modos. ADOPCIÓN (`--from-basic-memory`) apunta a una KB
/// **ya existente y poblada** — no se toca ni un byte dentro de ella: nada
/// de `prepara_kb`, nada de plantilla, nada de `git init`. CREACIÓN (`--kb`
/// + `--name`) es la que nace aquí: valida el destino, vuelca la semilla, la
/// versiona con git (best-effort) y la indexa.
```

a:

```rust
/// `exo init`: dos modos. ADOPCIÓN (`--from-basic-memory`) apunta a una KB
/// **ya existente y poblada** — no se toca ni un byte dentro de ella: nada
/// de `prepara_kb`, nada de plantilla, nada de `git init`. CREACIÓN
/// (`--kb` + `--name`) es la que nace aquí: valida el destino, vuelca la
/// semilla, la versiona con git (best-effort) y la indexa.
```

`tests/recall_contenido.rs:4-8` — de:

```rust
//! inyecta el CUERPO del core-index (contrato de memoria + doctrina compacta
//! + mapa de cores) más un digest de actividad reciente. Servir solo rutas
//! sería una regresión funcional silenciosa — el agente perdería la doctrina
//! en todas las sesiones y nadie lo notaría hasta que empezara a comportarse
//! peor.
```

a:

```rust
//! inyecta el CUERPO del core-index (contrato de memoria + doctrina
//! compacta + mapa de cores) más un digest de actividad reciente. Servir
//! solo rutas sería una regresión funcional silenciosa — el agente perdería
//! la doctrina en todas las sesiones y nadie lo notaría hasta que empezara
//! a comportarse peor.
```

- [ ] **Step 4: `unnecessary_to_owned` en `src/lib.rs:248-252`**

La firma real de fastembed 5.17.3 es
`pub fn embed<S: AsRef<str> + Send + Sync>(&mut self, texts: impl AsRef<[S]>, batch_size: Option<usize>)`
(verificado en `text_embedding/impl.rs:432`), así que un `&[String]` entra
directo y el `to_vec()` clona el batch entero sin necesidad. De:

```rust
    pub fn embebe_batch(&mut self, textos: &[String]) -> Result<Vec<Vec<f32>>> {
        self.te
            .embed(textos.to_vec(), None)
            .context("embed batch con fastembed")
    }
```

a:

```rust
    pub fn embebe_batch(&mut self, textos: &[String]) -> Result<Vec<Vec<f32>>> {
        self.te
            .embed(textos, None)
            .context("embed batch con fastembed")
    }
```

- [ ] **Step 5: `missing_transmute_annotations` en `src/lib.rs:27-33`**

Anotar el tipo del `transmute` no cambia el comportamiento: hace explícito lo
que hoy infiere el compilador, que es justo lo que el lint pide en una línea
`unsafe`. De:

```rust
        rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
            sqlite_vec::sqlite3_vec_init as *const (),
        )));
```

a:

```rust
        // Anotación explícita del tipo destino (clippy::missing_transmute_annotations):
        // en una línea `unsafe`, dejar que el compilador lo infiera esconde
        // exactamente el dato que hay que poder auditar.
        rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute::<
            *const (),
            unsafe extern "C" fn(
                *mut rusqlite::ffi::sqlite3,
                *mut *mut std::os::raw::c_char,
                *const rusqlite::ffi::sqlite3_api_routines,
            ) -> std::os::raw::c_int,
        >(sqlite_vec::sqlite3_vec_init as *const ())));
```

Si el tipo exacto no cuadra al compilar, **cópialo literal del propio mensaje
de clippy**, que lo imprime entero en su `help:`.

- [ ] **Step 6: `chunks_exact` con tamaño constante en `src/vectores.rs:53-57`**

De:

```rust
        b.chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect()
```

a:

```rust
        b.as_chunks::<4>()
            .0
            .iter()
            .map(|c| f32::from_le_bytes(*c))
            .collect()
```

La guarda de longitud exacta de la línea de arriba (`b.len() == BYTES_ESPERADOS`)
**no se toca**: es un hallazgo del gate M6, y el resto que devuelve `as_chunks`
se descarta solo porque esa guarda ya garantiza que no lo hay. Si `as_chunks` no
está estabilizado bajo la MSRV 1.95, deja el `chunks_exact` y pon
`#[allow(clippy::manual_slice_size_calculation)]` sobre la función con una línea
de justificación citando la MSRV.

- [ ] **Step 7: `too_many_arguments` en `src/escritor.rs:248` (8/7)**

Se resuelve con `allow` justificado, **no** con refactor: meter una struct de
parámetros en `escribe_nueva` toca el camino de escritura y sus tests durante
una tarea cuyo objetivo es montar el CI, y ese es scope que no se ha pedido.
Se declara la deuda en vez de esconderla. Encima del doc-comment existente de
`pub fn escribe_nueva`:

```rust
// Ocho parámetros contra el umbral de 7 de clippy. Se declara en vez de
// refactorizar: agrupar en una struct de parámetros toca el camino de
// escritura y sus tests, y esto es una tarea de CI. Anotado como deuda en
// docs/backlog.md.
#[allow(clippy::too_many_arguments)]
```

- [ ] **Step 8: Ver el gate en verde**

```bash
cd engine
cargo clippy --all-targets -- -D warnings; echo "EXIT=$?"
```

Expected: `EXIT=0`, sin una sola línea `error:` ni `warning:` del crate `exo`.

- [ ] **Step 9: La suite sigue verde (los Steps 4, 5 y 6 tocan código, no solo comentarios)**

```bash
cd engine
./scripts/test-hermetico.sh; echo "EXIT=$?"
```

Expected: `EXIT=0` y la línea que empieza por `test-hermetico: OK — la suite corre sin ~/.exo/config.toml`.

- [ ] **Step 10: `fmt` sigue verde tras editar código**

```bash
cd engine
cargo fmt --check; echo "EXIT=$?"
```

Expected: `EXIT=0`. Si sale rojo, `cargo fmt` e incluir el resultado en este
mismo commit.

- [ ] **Step 11: Commit**

```bash
git add engine/src engine/tests/recall_contenido.rs
git commit -m "chore(lint): los 12 avisos de clippy a cero, para que -D warnings sea un gate y no un adorno"
```

---

### Task 3: El workflow — y verlo rojo antes de darlo por bueno

**Files:**
- Create: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: `cargo fmt --check` (exit 0 tras Task 1), `cargo clippy --all-targets -- -D warnings`
  (exit 0 tras Task 2), y `engine/scripts/test-hermetico.sh`, que ya existe y
  corre `EXO_CONFIG=<ruta inexistente> cargo test --release --no-fail-fast`
  desde `engine/`, midiendo el exit code de cargo directamente.
- Produces: un workflow `CI` con cinco jobs: `lint`, `msrv`, y `test` en tres SO.

- [ ] **Step 1: Crear el workflow**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
  workflow_dispatch:

permissions:
  contents: read

concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  lint:
    name: fmt + clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: engine
      - name: cargo fmt --check
        working-directory: engine
        run: cargo fmt --check
      - name: cargo clippy -D warnings
        working-directory: engine
        run: cargo clippy --all-targets -- -D warnings

  msrv:
    name: MSRV declarada (1.95)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      # La MSRV está declarada en engine/Cargo.toml con una nota empírica y
      # hasta hoy nada la comprobaba. Este job la hace falsable.
      - uses: dtolnay/rust-toolchain@1.95.0
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: engine
          key: msrv
      - name: cargo check con la MSRV
        working-directory: engine
        run: cargo check --all-targets --locked

  test:
    name: test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    timeout-minutes: 60
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: engine
      # Nueve suites indexan cuerpos no vacíos y eso descarga el ONNX de
      # jina-es (615 MB medidos) vía hf-hub. NO se marcan `#[ignore]`: un CI
      # que no ejerce indexer ni buscador es verde sin significado. La clave
      # lleva la revisión pineada del modelo, así que es inmutable: un acierto
      # de caché no vuelve a subir nada.
      - name: Caché del modelo jina-es (revisión pineada)
        uses: actions/cache@v4
        with:
          path: ~/.cache/huggingface/hub/models--jinaai--jina-embeddings-v2-base-es
          key: hf-jina-es-8e2d780d-${{ runner.os }}
      # El gate ya existe y ya se demostró falsable (2026-08-27). El CI lo
      # consume tal cual en vez de reinventar el comando de test.
      - name: Gate hermético — suite completa sin ~/.exo/config.toml
        shell: bash
        run: ./engine/scripts/test-hermetico.sh
```

- [ ] **Step 2: Commit y push a una rama**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: gate automatico en tres SO, consumiendo el gate hermetico que ya existia"
git push -u origin HEAD
```

- [ ] **Step 3: Ver la primera corrida**

```bash
RAMA="$(git branch --show-current)"
ID="$(gh run list --branch "$RAMA" --limit 1 --json databaseId --jq '.[0].databaseId')"
gh run watch "$ID" --exit-status; echo "EXIT=$?"
gh run view "$ID"
```

Expected: `EXIT=0` y los cinco jobs (`lint`, `msrv`, `test (ubuntu-latest)`,
`test (windows-latest)`, `test (macos-latest)`) en `success`.

Esta primera corrida paga la descarga del modelo en los tres SO (615 MB por
runner) y la de los binarios de ONNX Runtime que baja la feature
`ort-download-binaries-rustls-tls` de fastembed. Es lenta por diseño; la
segunda no lo será. Anota la duración del job `test` de cada SO.

**Si algún runner sale rojo:** NO lo borres de la matriz, NO le pongas
`continue-on-error` y NO marques suites como `#[ignore]`. Captura el log
(`gh run view <id> --log-failed`) y repórtalo al orquestador. Los dos rojos
plausibles y su lectura: `macos-latest` es arm64 y depende de que ort publique
binario para `aarch64-apple-darwin`; `windows-latest` compila `rusqlite`
bundled y `sqlite-vec`, que exigen toolchain C (MSVC viene de serie en el
runner, a diferencia del portátil de trabajo donde `cargo check` falla con
`cc-rs: failed to find tool "gcc.exe"`). Un rojo ahí es información, que es
justo para lo que se monta esto.

- [ ] **Step 4: Romper el gate a propósito**

Un gate que nunca se ha visto fallar no es evidencia de nada. Se rompen a la vez
las dos mitades del job `lint` con una sola función que viola el formato y
`collapsible_if`:

```bash
cd engine
printf '\nfn _roto_a_proposito( x : i32 )->i32{ if x > 0 { if x > 1 { return 2; } } x }\n' >> src/vectores.rs
cargo fmt --check | grep -c "^Diff in"
cd ..
git add engine/src/vectores.rs
git commit -m "temp: rompe fmt y clippy a proposito para ver el gate en rojo"
git push
```

Expected en local: al menos `1` diff de fmt.

- [ ] **Step 5: Ver la corrida en ROJO**

```bash
RAMA="$(git branch --show-current)"
ID="$(gh run list --branch "$RAMA" --limit 1 --json databaseId --jq '.[0].databaseId')"
gh run watch "$ID" --exit-status; echo "EXIT=$?"
gh run view "$ID" --log-failed | grep -E "Diff in|collapsible_if" | head -5
```

Expected: `EXIT` distinto de 0, el job `lint` en `failure`, y el log citando
tanto un `Diff in` de rustfmt como `clippy::collapsible_if`. **Anota el id de
esta corrida: es la evidencia de falsabilidad de este plan.**

- [ ] **Step 6: Retirar la rotura y ver el verde de vuelta**

```bash
git reset --hard HEAD~1
git push --force-with-lease
```

Force-push sobre la rama de trabajo, nunca sobre `main`. La corrida roja
sobrevive en el historial de Actions aunque el commit desaparezca — que es
donde vive la evidencia.

- [ ] **Step 7: Confirmar el verde final y medir la caché**

```bash
RAMA="$(git branch --show-current)"
ID="$(gh run list --branch "$RAMA" --limit 1 --json databaseId --jq '.[0].databaseId')"
gh run watch "$ID" --exit-status; echo "EXIT=$?"
gh run view "$ID"
```

Expected: `EXIT=0`, cinco jobs en `success`. Compara la duración del job `test`
con la de la primera corrida (Step 3) y anota ambas: es la medida de si la
caché del modelo sirve para algo o solo lo parece.

---

### Task 4: Sincerar la documentación

**Files:**
- Modify: `docs/backlog.md` (ítem Media «CI mínimo — el gate que falta»; ítem
  Media «Caché del modelo de embeddings»; sección `## Cerrado con evidencia`;
  cabecera `Última revisión`)
- Modify: `README.md` (bloque de estado, la frase «sin CI que los corra
  todavía»)

**Interfaces:**
- Consumes: los ids de corrida y las duraciones anotadas en la Task 3.
- Produces: nada que consuma código.

- [ ] **Step 1: Comprobar qué afirma hoy el README**

```bash
grep -n "sin CI que los corra todavía" README.md
```

Expected: una coincidencia dentro del bloque de estado.

- [ ] **Step 2: Corregir el bloque de estado del README**

Sustituir «Suite: 200 tests verdes en 28 binarios — sin CI que los corra
todavía, y la hermeticidad cubre la config, no la caché del modelo de
embeddings (un runner limpio descarga ~0,6 GB la primera vez, o falla sin red)»
por una frase que diga la verdad de hoy: que la suite la corre
`.github/workflows/ci.yml` en ubuntu / windows / macos vía el gate hermético,
con la caché del modelo pineada por revisión, más `fmt --check`,
`clippy -D warnings` y un check de la MSRV declarada. Y quitar de la lista de
«Pendiente» lo que este plan cierra, dejando lo que sigue abierto (release,
instaladores, `exo doctor`, `exo budget`).

- [ ] **Step 3: Mover los dos ítems del backlog a `## Cerrado con evidencia`**

Con las cifras reales, no con adjetivos: los ids de las corridas verde y roja,
la duración del job `test` con caché fría y con caché caliente, y la mención
explícita de que la falsabilidad se demostró rompiendo el gate a propósito.
Dejar anotado en el ítem de la caché del modelo que la decisión tomada fue
cachear y **no** marcar `#[ignore]`, con el porqué.

- [ ] **Step 4: Anotar la deuda nueva que abre este plan**

Dos ítems nuevos en `## Media` de `docs/backlog.md`:
uno para el `#[allow(clippy::too_many_arguments)]` de `escritor.rs` (la struct
de parámetros que se decidió no hacer aquí), y otro para los 9 scripts
`test-*.sh` de `plugins/exo/scripts/`, de los que **5 referencian rutas de esta
máquina** y por eso no entran en CI: la capa thin sigue sin gate automático.

- [ ] **Step 5: Actualizar `Última revisión` de `docs/backlog.md` a 2026-09-02 y commitear**

```bash
git add docs/backlog.md README.md
git commit -m "docs: el CI existe, y el backlog lo dice con las cifras de la corrida"
```

---

## Self-review

- **Cobertura contra el alcance pedido:** pasada de fmt (Task 1) · 12 avisos de
  clippy a cero (Task 2) · `.github/workflows/ci.yml` con test + fmt + clippy
  en los tres SO consumiendo `test-hermetico.sh` (Task 3) · caché del modelo
  con clave por revisión pineada y sin `#[ignore]` (Task 3, Step 1). Añadido
  sobre lo pedido, y declarado aquí: el job `msrv`, porque el ítem de backlog
  que pide el CI se queja literalmente de que la MSRV se afirma y no se
  comprueba, y cuesta un job de diez líneas.
- **Placeholder scan:** cada paso que cambia código trae el código; cada paso de
  verificación trae el comando y su salida esperada. No hay «similar a la Task
  N» ni «manejar los errores apropiadamente».
- **Consistencia:** el comando de test del workflow (`./engine/scripts/test-hermetico.sh`)
  es el mismo que verifica la Task 2 en su Step 9; los comandos de fmt y clippy
  del job `lint` son literalmente los de los Steps 3 y 8 de las Tasks 1 y 2.
- **Orden:** las Tasks 1 y 2 tienen que estar commiteadas en la rama antes del
  push de la Task 3, o la primera corrida nace roja por algo que ya sabíamos.
