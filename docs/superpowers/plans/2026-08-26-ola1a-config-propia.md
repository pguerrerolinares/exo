# Ola 1A — Config propia y contrato del envelope · Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: usa `process:orchestrate`
> para ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`)
> para tracking.

**Goal:** cortar la dependencia de arranque con basic-memory —el engine pasa a
leer su propia `~/.exo/config.toml`— y dejar el contrato JSON público
coherente en inglés antes del 1.0.

**Architecture:** un módulo `config` nuevo centraliza la carga TOML con
precedencia `flag > env > config > error accionable`, y los tres puntos que hoy
leen `~/.basic-memory/config.json` (`kb_desde_config`, `config_embeddings`,
`min_similitud_de_config`) pasan a consumirlo. El renombrado de claves del
envelope se hace con `#[serde(rename)]` sobre los structs ya existentes, sin
tocar identificadores internos de Rust —que están explícitamente fuera de
scope—, y sube `SCHEMA_VERSION` a 2.

**Tech Stack:** Rust 2024 · clap 4.6 · serde/serde_json · toml (dep nueva) ·
rusqlite 0.40 bundled · SQLite FTS5 + sqlite-vec 0.1.9 · Git Bash en Windows 11.

## Global Constraints

- Spec fuente: `docs/superpowers/specs/2026-08-26-exo-generico-design.md` (v2).
  Ante conflicto entre este plan y la spec, manda la spec.
- **Precedencia de config, verbatim de la spec:** `flag CLI > env (EXO_CONFIG,
  EXO_KB, EXO_DB) > ~/.exo/config.toml > error accionable`. Sin defaults
  inventados.
- **Sin fallback a basic-memory.** La única lectura de `~/.basic-memory/config.json`
  que sobrevive es la de `exo init --from-basic-memory`, explícita y de una vez.
- **Error handling, verbatim:** «Fichero ausente ⇒ el mensaje nombra `exo init` o
  `exo init --from-basic-memory`. Clave ausente ⇒ nombra la clave y la ruta.
  Nunca un default silencioso.»
- **D8:** las claves de `data` del envelope van al inglés, `SCHEMA_VERSION` 1→2.
- **Fuera de scope, no lo hagas:** traducir identificadores internos de Rust
  (`buscador.rs`, `busca_hybrid`, `escritor.rs`…). Por eso el renombrado usa
  `#[serde(rename)]`, patrón que ya existe en la casa
  (`buscador.rs`: `#[serde(rename = "type")] pub tipo: String`).
- **Working dir:** `C:\proyectos\homework\exo`. Shell: Git Bash. El engine vive
  en `engine/`; todos los `cargo` se lanzan desde ahí.
- **Nunca metas `cargo` en una tubería**: el exit code de una tubería es el del
  último comando. Usa `${PIPESTATUS[0]}` o no la uses. (Lección documentada:
  `cargo test … | tail -40` devolvió 0 con la suite rota y cargo saliendo 101.)
- Un test se ve fallar **antes** de escribir la implementación. Commit por tarea.
- **Orden respecto a la ola 1B.** La spec declara G1 y G2 paralelos, y
  lógicamente lo son, pero **comparten el directorio
  `plugins/reflex/scripts/`**: este plan lo edita (Tasks 7 y 8) y el de 1B lo
  mueve entero con `git mv`. **Este plan va primero y completo.** Ejecutarlos
  en paralelo produce un conflicto de rename que git resuelve mal y en
  silencio: los cambios caen en la ruta vieja y el `git mv` se lleva la
  versión sin migrar.

---

### Task 1: Pasada de `cargo fmt` (precondición, sin lógica)

Va primera para que todo el código posterior nazca formateado y el gate
`fmt --check` de la ola G5 no herede 90 diffs.

**Files:**
- Modify: todos los `engine/src/*.rs` y `engine/tests/*.rs` que `fmt` toque.

**Interfaces:**
- Consumes: nada.
- Produces: árbol `engine/` con `cargo fmt --check` limpio. Ninguna tarea
  posterior depende de esto funcionalmente, pero todas parten de él.

- [ ] **Step 1: Medir el estado de partida**

```bash
cd /c/proyectos/homework/exo/engine
cargo fmt --check > /tmp/fmt-antes.txt 2>&1; echo "EXIT=$?"
grep -c '^Diff in' /tmp/fmt-antes.txt
```

Expected: `EXIT=1` y un conteo de **90**. Si sale otro número, anótalo en el
commit; no abortes.

- [ ] **Step 2: Verificar que la suite está verde ANTES de tocar nada**

```bash
cd /c/proyectos/homework/exo/engine
cargo test --release --no-fail-fast > /tmp/test-antes.txt 2>&1
echo "CARGO_EXIT=$?"
grep -E '^test result' /tmp/test-antes.txt
```

Expected: `CARGO_EXIT=0` y todas las líneas `test result: ok`. Si algo está
rojo aquí, **para y repórtalo**: no se formatea sobre una suite rota.

- [ ] **Step 3: Aplicar el formato**

```bash
cd /c/proyectos/homework/exo/engine
cargo fmt
cargo fmt --check; echo "EXIT=$?"
```

Expected: `EXIT=0`.

- [ ] **Step 4: Verificar que el formato no cambió comportamiento**

```bash
cd /c/proyectos/homework/exo/engine
cargo test --release --no-fail-fast > /tmp/test-despues.txt 2>&1
echo "CARGO_EXIT=$?"
diff <(grep -E '^test result' /tmp/test-antes.txt) <(grep -E '^test result' /tmp/test-despues.txt)
echo "DIFF_EXIT=$?"
```

Expected: `CARGO_EXIT=0` y `DIFF_EXIT=0` (mismos conteos exactos antes y
después). `cargo fmt` no debe mover un solo test.

- [ ] **Step 5: Commit**

```bash
cd /c/proyectos/homework/exo
git add engine/
git commit -m "style(engine): cargo fmt — 90 diffs preexistentes, cero cambio de comportamiento"
```

---

### Task 2: Módulo `config` — carga TOML con precedencia y errores accionables

**Files:**
- Create: `engine/src/config.rs`
- Modify: `engine/src/lib.rs:6-16` (declaración de módulos)
- Modify: `engine/Cargo.toml` (dep `toml`)
- Test: `engine/tests/config.rs`

**Interfaces:**
- Consumes: nada.
- Produces, para las tareas 3, 4 y 8:
  - `exo::config::Config { schema_version: u32, kb: Kb, index: Index, embeddings: Embeddings }`
  - `exo::config::Kb { path: PathBuf, name: String }`
  - `exo::config::Index { db: PathBuf }`
  - `exo::config::Embeddings { model: String, dims: usize, min_similarity: f64 }`
  - `pub fn ruta_config() -> Result<PathBuf>` — `$EXO_CONFIG` si está, si no `~/.exo/config.toml`
  - `pub fn carga() -> Result<Config>`
  - `pub fn expande_tilde(p: &Path) -> PathBuf`

- [ ] **Step 1: Añadir la dependencia**

En `engine/Cargo.toml`, en `[dependencies]`, en orden alfabético (entre
`sqlite-vec` y `yaml_serde`):

```toml
toml = "0.8"
```

- [ ] **Step 2: Escribir los tests que fallan**

Crear `engine/tests/config.rs`:

```rust
//! Contrato de `exo::config`: precedencia, errores accionables y expansión
//! de `~`. Todos los tests fijan `EXO_CONFIG` a un fichero temporal, así
//! que ninguno toca el `~/.exo/config.toml` de la máquina.

use std::io::Write;

/// Escribe un config temporal y devuelve su ruta. El `TempDir` se devuelve
/// también: si se dropea, el directorio desaparece bajo los pies del test.
fn config_temporal(contenido: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let ruta = dir.path().join("config.toml");
    let mut f = std::fs::File::create(&ruta).expect("crear config");
    f.write_all(contenido.as_bytes()).expect("escribir config");
    (dir, ruta)
}

const COMPLETO: &str = r#"
schema_version = 1

[kb]
path = "C:/kb/demo"
name = "demo"

[index]
db = "~/.exo/index.db"

[embeddings]
model = "jinaai/jina-embeddings-v2-base-es"
dims = 768
min_similarity = 0.35
"#;

#[test]
fn carga_un_config_completo() {
    let (_dir, ruta) = config_temporal(COMPLETO);
    unsafe { std::env::set_var("EXO_CONFIG", &ruta) };
    let cfg = exo::config::carga().expect("carga");
    assert_eq!(cfg.schema_version, 1);
    assert_eq!(cfg.kb.name, "demo");
    assert_eq!(cfg.kb.path, std::path::PathBuf::from("C:/kb/demo"));
    assert_eq!(cfg.embeddings.dims, 768);
    assert_eq!(cfg.embeddings.min_similarity, 0.35);
    unsafe { std::env::remove_var("EXO_CONFIG") };
}

#[test]
fn fichero_ausente_nombra_el_comando_de_arranque() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ruta = dir.path().join("no-existe.toml");
    unsafe { std::env::set_var("EXO_CONFIG", &ruta) };
    let err = exo::config::carga().expect_err("debe fallar");
    let msg = format!("{err:#}");
    // El mensaje tiene que decirle al usuario qué ejecutar, no solo que falta
    // un fichero: es la diferencia entre un error y un callejón sin salida.
    assert!(msg.contains("exo init"), "mensaje sin salida accionable: {msg}");
    assert!(msg.contains("no-existe.toml"), "mensaje sin la ruta: {msg}");
    unsafe { std::env::remove_var("EXO_CONFIG") };
}

#[test]
fn clave_ausente_nombra_la_clave_y_la_ruta() {
    let sin_name = r#"
schema_version = 1

[kb]
path = "C:/kb/demo"

[index]
db = "~/.exo/index.db"

[embeddings]
model = "m"
dims = 768
min_similarity = 0.35
"#;
    let (_dir, ruta) = config_temporal(sin_name);
    unsafe { std::env::set_var("EXO_CONFIG", &ruta) };
    let err = exo::config::carga().expect_err("debe fallar");
    let msg = format!("{err:#}");
    assert!(msg.contains("name"), "no nombra la clave: {msg}");
    assert!(msg.contains("config.toml"), "no nombra la ruta: {msg}");
    unsafe { std::env::remove_var("EXO_CONFIG") };
}

#[test]
fn expande_tilde_en_rutas() {
    let home = dirs::home_dir().expect("home");
    let expandida = exo::config::expande_tilde(std::path::Path::new("~/.exo/index.db"));
    assert_eq!(expandida, home.join(".exo/index.db"));
    // Una ruta que NO empieza por `~` se devuelve intacta, incluida una
    // absoluta de Windows con dos puntos.
    let absoluta = std::path::Path::new("C:/proyectos/kb");
    assert_eq!(exo::config::expande_tilde(absoluta), absoluta.to_path_buf());
}

#[test]
fn acepta_barras_de_windows_en_el_path_de_la_kb() {
    let con_backslash = r#"
schema_version = 1

[kb]
path = 'C:\proyectos\homework\kb-demo'
name = "kb-demo"

[index]
db = "~/.exo/index.db"

[embeddings]
model = "m"
dims = 768
min_similarity = 0.35
"#;
    let (_dir, ruta) = config_temporal(con_backslash);
    unsafe { std::env::set_var("EXO_CONFIG", &ruta) };
    let cfg = exo::config::carga().expect("carga con backslashes");
    assert_eq!(
        cfg.kb.path,
        std::path::PathBuf::from(r"C:\proyectos\homework\kb-demo")
    );
    unsafe { std::env::remove_var("EXO_CONFIG") };
}

#[test]
fn toml_invalido_no_se_disfraza_de_clave_ausente() {
    let (_dir, ruta) = config_temporal("esto no es toml [[[");
    unsafe { std::env::set_var("EXO_CONFIG", &ruta) };
    let err = exo::config::carga().expect_err("debe fallar");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("no es TOML válido"),
        "un parse error debe decir que es un parse error: {msg}"
    );
    unsafe { std::env::remove_var("EXO_CONFIG") };
}
```

- [ ] **Step 3: Correr los tests y verlos fallar**

```bash
cd /c/proyectos/homework/exo/engine
cargo test --test config 2>&1 | tail -20
```

Expected: FAIL en compilación — `could not find 'config' in 'exo'`.

- [ ] **Step 4: Implementación mínima**

Crear `engine/src/config.rs`:

```rust
//! Config propia de exo (`~/.exo/config.toml`).
//!
//! Sustituye la lectura RO de `~/.basic-memory/config.json` que hacían
//! `kb_desde_config`, `config_embeddings` y `min_similitud_de_config`. Ese
//! acoplamiento era el bloqueante de M5b: el sustituto no puede depender del
//! sustituido para arrancar.
//!
//! Precedencia (spec §G1): `flag CLI > env > este fichero > error accionable`.
//! La parte `flag > env` la resuelve el llamador en `main.rs`; aquí vive el
//! último escalón. **Sin defaults inventados**: un default silencioso es la
//! clase de fallo que este proyecto existe para no volver a tener.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    pub schema_version: u32,
    pub kb: Kb,
    pub index: Index,
    pub embeddings: Embeddings,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Kb {
    pub path: PathBuf,
    /// Prefijo de permalink. **Explícito, no derivado de `path.file_name()`**:
    /// cierra el disenso abierto del gate M4, donde la spec §3.1 afirmaba que
    /// salía de la config y el código lo sacaba del nombre del directorio.
    /// Hoy coinciden; el día que no, reventaba en silencio.
    pub name: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Index {
    pub db: PathBuf,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Embeddings {
    pub model: String,
    pub dims: usize,
    pub min_similarity: f64,
}

/// Ruta del fichero de config: `$EXO_CONFIG` si está definida, si no
/// `~/.exo/config.toml`.
pub fn ruta_config() -> Result<PathBuf> {
    if let Ok(v) = std::env::var("EXO_CONFIG") {
        if !v.is_empty() {
            return Ok(PathBuf::from(v));
        }
    }
    Ok(dirs::home_dir()
        .context("sin HOME: no se puede localizar ~/.exo/config.toml")?
        .join(".exo/config.toml"))
}

/// Expande un `~` inicial a `$HOME`. Cualquier otra ruta se devuelve intacta
/// —incluidas las absolutas de Windows, que llevan dos puntos y no tilde.
pub fn expande_tilde(p: &Path) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(resto) = s.strip_prefix("~/").or_else(|| s.strip_prefix("~\\")) {
        if let Some(home) = dirs::home_dir() {
            return home.join(resto);
        }
    }
    p.to_path_buf()
}

/// Carga y valida la config. Errores accionables por contrato: el de fichero
/// ausente nombra el comando que lo crea, el de clave ausente nombra la clave
/// y la ruta.
pub fn carga() -> Result<Config> {
    let ruta = ruta_config()?;
    let contenido = std::fs::read_to_string(&ruta).map_err(|e| {
        anyhow::anyhow!(
            "no encuentro la config de exo en {} ({e}).\n\
             Créala con `exo init --from-basic-memory` si vienes de basic-memory, \
             o con `exo init --kb <ruta> --name <nombre>`.",
            ruta.display()
        )
    })?;
    // Dos errores distintos con dos mensajes distintos: un TOML corrupto no
    // debe disfrazarse de "te falta una clave", que manda al usuario a buscar
    // donde no es.
    let valor: toml::Value = toml::from_str(&contenido)
        .with_context(|| format!("{} no es TOML válido", ruta.display()))?;
    let cfg: Config = valor.try_into().map_err(|e| {
        anyhow::anyhow!("config incompleta o mal tipada en {}: {e}", ruta.display())
    })?;
    Ok(cfg)
}
```

En `engine/src/lib.rs`, añadir la declaración del módulo en orden alfabético
(entre `pub mod buscador;` y `pub mod envelope;`):

```rust
pub mod config;
```

- [ ] **Step 5: Correr los tests y verlos pasar**

```bash
cd /c/proyectos/homework/exo/engine
cargo test --test config 2>&1 | grep -E '^test result|^test '
```

Expected: `test result: ok. 6 passed; 0 failed`.

- [ ] **Step 6: Verificar que no se rompió nada más**

```bash
cd /c/proyectos/homework/exo/engine
cargo test --no-fail-fast > /tmp/t2.txt 2>&1; echo "CARGO_EXIT=$?"
grep -E '^test result' /tmp/t2.txt
```

Expected: `CARGO_EXIT=0`, ninguna suite en rojo.

- [ ] **Step 7: Commit**

```bash
cd /c/proyectos/homework/exo
git add engine/Cargo.toml engine/Cargo.lock engine/src/config.rs engine/src/lib.rs engine/tests/config.rs
git commit -m "feat(config): módulo de config propia con precedencia y errores accionables"
```

---

### Task 3: Cablear la config y borrar las tres lecturas de basic-memory

**Files:**
- Modify: `engine/src/lib.rs` — `kb_desde_config`, `config_embeddings`, `min_similitud_de_config`
- Test: `engine/tests/config_cableado.rs`

**Interfaces:**
- Consumes: `exo::config::{carga, expande_tilde, Config}` de la Task 2.
- Produces, con la MISMA firma que hoy para no tocar los llamadores:
  - `pub fn kb_desde_config() -> Result<PathBuf>`
  - `pub fn config_embeddings() -> Result<ConfigEmbeddings>` (struct `ConfigEmbeddings { modelo: String, dims: usize }` **sin cambios**: es interno, no se serializa)
  - `pub fn min_similitud_de_config() -> Result<f64>`

- [ ] **Step 1: Escribir el test que falla**

Crear `engine/tests/config_cableado.rs`:

```rust
//! Las tres funciones que leían `~/.basic-memory/config.json` ahora leen la
//! config propia. El test lo comprueba de la única forma falsable que hay:
//! apunta `EXO_CONFIG` a un fichero con valores IMPOSIBLES de encontrar en
//! ninguna config de basic-memory de esta máquina.

use std::io::Write;

fn con_config<T>(contenido: &str, f: impl FnOnce() -> T) -> T {
    let dir = tempfile::tempdir().expect("tempdir");
    let ruta = dir.path().join("config.toml");
    let mut fh = std::fs::File::create(&ruta).expect("crear");
    fh.write_all(contenido.as_bytes()).expect("escribir");
    unsafe { std::env::set_var("EXO_CONFIG", &ruta) };
    let r = f();
    unsafe { std::env::remove_var("EXO_CONFIG") };
    r
}

const CFG: &str = r#"
schema_version = 1

[kb]
path = "C:/kb/valor-imposible-de-basic-memory"
name = "valor-imposible"

[index]
db = "~/.exo/index.db"

[embeddings]
model = "modelo/imposible-v9"
dims = 1234
min_similarity = 0.99
"#;

#[test]
fn kb_desde_config_lee_la_config_propia() {
    let kb = con_config(CFG, || exo::kb_desde_config().expect("kb"));
    assert_eq!(
        kb,
        std::path::PathBuf::from("C:/kb/valor-imposible-de-basic-memory")
    );
}

#[test]
fn config_embeddings_lee_la_config_propia() {
    let c = con_config(CFG, || exo::config_embeddings().expect("emb"));
    assert_eq!(c.modelo, "modelo/imposible-v9");
    assert_eq!(c.dims, 1234);
}

#[test]
fn min_similitud_lee_la_config_propia() {
    let m = con_config(CFG, || exo::min_similitud_de_config().expect("min"));
    assert_eq!(m, 0.99);
}

#[test]
fn el_db_de_la_config_expande_tilde() {
    let db = con_config(CFG, || {
        exo::config::expande_tilde(&exo::config::carga().expect("cfg").index.db)
    });
    let home = dirs::home_dir().expect("home");
    assert_eq!(db, home.join(".exo/index.db"));
}
```

- [ ] **Step 2: Correr el test y verlo fallar**

```bash
cd /c/proyectos/homework/exo/engine
cargo test --test config_cableado 2>&1 | grep -E '^test |panicked' | head
```

Expected: FAIL — las tres funciones siguen leyendo `~/.basic-memory/config.json`,
así que devuelven la ruta de `kb-demo` (o error) en vez del valor imposible.

- [ ] **Step 3: Reemplazar los tres cuerpos**

En `engine/src/lib.rs`, sustituir **completos** los tres bloques
(`kb_desde_config` en la zona de la línea 78, `config_embeddings` y
`min_similitud_de_config` en la zona de la 131) por:

```rust
/// Raíz de la KB desde `[kb] path` de la config propia (`~/.exo/config.toml`).
///
/// Antes leía `projects["kb-demo"].path` de `~/.basic-memory/config.json`:
/// el sustituto dependía del sustituido para arrancar, y era el bloqueante
/// duro de M5b. La precedencia `flags > env > config` la resuelve el llamador.
pub fn kb_desde_config() -> Result<std::path::PathBuf> {
    let cfg = config::carga()?;
    Ok(config::expande_tilde(&cfg.kb.path))
}

/// Nombre de proyecto de la KB (prefijo de permalink), EXPLÍCITO en config.
///
/// Cierra el disenso del gate M4: `write new` lo derivaba de
/// `kb.file_name()`, contra lo que decía la spec §3.1. Coincidían por suerte.
pub fn nombre_kb() -> Result<String> {
    Ok(config::carga()?.kb.name)
}

/// Modelo y dims de embeddings desde `[embeddings]` de la config propia.
pub fn config_embeddings() -> Result<ConfigEmbeddings> {
    let cfg = config::carga()?;
    Ok(ConfigEmbeddings {
        modelo: cfg.embeddings.model,
        dims: cfg.embeddings.dims,
    })
}

/// Umbral por defecto del arm vector desde `[embeddings] min_similarity`.
pub fn min_similitud_de_config() -> Result<f64> {
    Ok(config::carga()?.embeddings.min_similarity)
}
```

Actualizar además el doc-comment de `Embedder::desde_config` (`lib.rs`, zona de
la línea 205): donde dice «Inicializa fastembed con el modelo de
`~/.basic-memory/config.json` (RO, D6)», poner «Inicializa fastembed con el
modelo de `[embeddings] model` de `~/.exo/config.toml`».

- [ ] **Step 4: Correr el test y verlo pasar**

```bash
cd /c/proyectos/homework/exo/engine
cargo test --test config_cableado 2>&1 | grep -E '^test result'
```

Expected: `test result: ok. 4 passed; 0 failed`.

- [ ] **Step 5: Verificar que basic-memory ya no se lee en ningún camino de arranque**

```bash
cd /c/proyectos/homework/exo/engine
grep -rn "basic-memory/config.json" src/
```

Expected: **cero líneas** (la de `exo init --from-basic-memory` aún no existe;
llega en la Task 4). Si sale alguna, no está cortado el cordón.

- [ ] **Step 6: Commit**

```bash
cd /c/proyectos/homework/exo
git add engine/src/lib.rs engine/tests/config_cableado.rs
git commit -m "feat(config): el engine arranca con su propia config, no con la de basic-memory"
```

---

### Task 4: `exo init` — crear la config y migrar desde basic-memory

Aquí `exo init` **solo escribe config**. El volcado de la KB semilla es G3 y
se le añade después sin cambiar esta firma.

**Files:**
- Modify: `engine/src/main.rs` — `enum Comando` (zona línea 36) y `fn ejecuta` (zona 251)
- Create: `engine/src/inicia.rs`
- Modify: `engine/src/lib.rs` (declarar `pub mod inicia;`)
- Test: `engine/tests/inicia.rs`

**Interfaces:**
- Consumes: `exo::config::{ruta_config, Config}` (Task 2).
- Produces:
  - `pub fn escribe_config(destino: &Path, kb: &Path, nombre: &str, emb: &Embeddings, db: &Path, force: bool) -> Result<()>`
  - `pub fn desde_basic_memory(json: &str) -> Result<(PathBuf, String, Embeddings, )>` — devuelve `(kb_path, kb_name, embeddings)` leídos del JSON de basic-memory
  - CLI: `exo init [--kb <ruta>] [--name <n>] [--from-basic-memory] [--force] [--json]`

- [ ] **Step 1: Escribir los tests que fallan**

Crear `engine/tests/inicia.rs`:

```rust
//! `exo init`: escribe la config propia, y sabe migrarla desde el JSON de
//! basic-memory una sola vez. Es la ÚNICA lectura de basic-memory que
//! sobrevive en el engine, y es explícita.

const BM_JSON: &str = r#"{
  "projects": { "kb-demo": { "path": "C:/proyectos/homework/kb-demo" } },
  "default_project": "kb-demo",
  "semantic_embedding_model": "jinaai/jina-embeddings-v2-base-es",
  "semantic_embedding_dimensions": 768,
  "semantic_min_similarity": 0.35
}"#;

#[test]
fn migra_desde_basic_memory_leyendo_el_proyecto_por_defecto() {
    let (kb, nombre, emb) = exo::inicia::desde_basic_memory(BM_JSON).expect("migrar");
    assert_eq!(kb, std::path::PathBuf::from("C:/proyectos/homework/kb-demo"));
    // El nombre sale de `default_project`, NO de un literal "kb-demo"
    // hardcodeado: ese literal era justo el acoplamiento que se viene a matar.
    assert_eq!(nombre, "kb-demo");
    assert_eq!(emb.model, "jinaai/jina-embeddings-v2-base-es");
    assert_eq!(emb.dims, 768);
    assert_eq!(emb.min_similarity, 0.35);
}

#[test]
fn migrar_sin_default_project_falla_nombrando_la_clave() {
    let sin_default = r#"{ "projects": { "x": { "path": "/tmp/x" } } }"#;
    let err = exo::inicia::desde_basic_memory(sin_default).expect_err("debe fallar");
    let msg = format!("{err:#}");
    assert!(msg.contains("default_project"), "no nombra la clave: {msg}");
}

#[test]
fn escribe_una_config_que_se_puede_releer() {
    let dir = tempfile::tempdir().expect("tempdir");
    let destino = dir.path().join("config.toml");
    let emb = exo::config::Embeddings {
        model: "m/x".into(),
        dims: 768,
        min_similarity: 0.35,
    };
    exo::inicia::escribe_config(
        &destino,
        std::path::Path::new("C:/kb/demo"),
        "demo",
        &emb,
        std::path::Path::new("~/.exo/index.db"),
        false,
    )
    .expect("escribir");

    unsafe { std::env::set_var("EXO_CONFIG", &destino) };
    let cfg = exo::config::carga().expect("releer lo que acabo de escribir");
    unsafe { std::env::remove_var("EXO_CONFIG") };
    assert_eq!(cfg.kb.name, "demo");
    assert_eq!(cfg.embeddings.dims, 768);
}

#[test]
fn no_pisa_una_config_existente_sin_force() {
    let dir = tempfile::tempdir().expect("tempdir");
    let destino = dir.path().join("config.toml");
    std::fs::write(&destino, "# la config del usuario\n").expect("sembrar");
    let emb = exo::config::Embeddings {
        model: "m/x".into(),
        dims: 768,
        min_similarity: 0.35,
    };
    let err = exo::inicia::escribe_config(
        &destino,
        std::path::Path::new("C:/kb/demo"),
        "demo",
        &emb,
        std::path::Path::new("~/.exo/index.db"),
        false,
    )
    .expect_err("debe negarse");
    assert!(format!("{err:#}").contains("--force"));
    // Y el fichero original sigue intacto: negarse no es medio-escribir.
    let contenido = std::fs::read_to_string(&destino).expect("releer");
    assert_eq!(contenido, "# la config del usuario\n");
}
```

- [ ] **Step 2: Correr los tests y verlos fallar**

```bash
cd /c/proyectos/homework/exo/engine
cargo test --test inicia 2>&1 | tail -15
```

Expected: FAIL en compilación — `could not find 'inicia' in 'exo'`.

- [ ] **Step 3: Implementación**

En `engine/src/config.rs`, añadir `Serialize` y `Clone` al derive de
`Embeddings` (los otros structs no lo necesitan):

```rust
#[derive(Debug, Deserialize, serde::Serialize, PartialEq, Clone)]
pub struct Embeddings {
    pub model: String,
    pub dims: usize,
    pub min_similarity: f64,
}
```

Crear `engine/src/inicia.rs`:

```rust
//! `exo init`: creación de la config propia y migración de una sola vez desde
//! basic-memory.
//!
//! Es la ÚNICA lectura de `~/.basic-memory/config.json` que sobrevive en el
//! engine, y es explícita y borrable: una migración se puede eliminar en tres
//! meses, un fallback permanente no lo quita nadie nunca.

use crate::config::Embeddings;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Lee el JSON de basic-memory y devuelve `(ruta_kb, nombre_kb, embeddings)`.
///
/// El nombre sale de `default_project`, **no** de un literal `"kb-demo"`:
/// ese literal (`lib.rs:93` en la versión anterior) era el acoplamiento que
/// esta ola viene a matar; reintroducirlo aquí sería cambiar de sitio el bug.
pub fn desde_basic_memory(json: &str) -> Result<(PathBuf, String, Embeddings)> {
    let v: serde_json::Value =
        serde_json::from_str(json).context("la config de basic-memory no es JSON válido")?;
    let nombre = v
        .get("default_project")
        .and_then(|x| x.as_str())
        .context("default_project ausente en la config de basic-memory")?
        .to_string();
    let path = v
        .get("projects")
        .and_then(|p| p.get(&nombre))
        .and_then(|p| p.get("path"))
        .and_then(|p| p.as_str())
        .with_context(|| format!("projects.{nombre}.path ausente en la config de basic-memory"))?;
    let emb = Embeddings {
        model: v["semantic_embedding_model"]
            .as_str()
            .context("semantic_embedding_model ausente")?
            .to_string(),
        dims: v["semantic_embedding_dimensions"]
            .as_u64()
            .context("semantic_embedding_dimensions ausente")? as usize,
        min_similarity: v["semantic_min_similarity"]
            .as_f64()
            .context("semantic_min_similarity ausente")?,
    };
    Ok((PathBuf::from(path), nombre, emb))
}

/// Escribe `config.toml`. Se niega si el destino existe y no hay `--force`:
/// pisar la config de alguien sin avisar es exactamente el tipo de efecto
/// silencioso que este proyecto persigue.
pub fn escribe_config(
    destino: &Path,
    kb: &Path,
    nombre: &str,
    emb: &Embeddings,
    db: &Path,
    force: bool,
) -> Result<()> {
    if destino.exists() && !force {
        anyhow::bail!(
            "ya existe una config en {} — repite con --force si de verdad quieres pisarla",
            destino.display()
        );
    }
    if let Some(padre) = destino.parent() {
        std::fs::create_dir_all(padre)
            .with_context(|| format!("crear {}", padre.display()))?;
    }
    // Se serializa a mano en vez de con `toml::to_string`: los comentarios son
    // la mitad del valor de un TOML editable a mano, y el serializador los
    // pierde. El runbook de W11 documenta que este fichero se edita a mano.
    let contenido = format!(
        r#"schema_version = 1

[kb]
# Raíz de la KB markdown. Barras normales funcionan también en Windows.
path = "{}"
# Prefijo de permalink. Explícito: NO se deriva del nombre del directorio.
name = "{}"

[index]
db = "{}"

[embeddings]
model = "{}"
dims = {}
min_similarity = {}
"#,
        kb.display().to_string().replace('\\', "/"),
        nombre,
        db.display().to_string().replace('\\', "/"),
        emb.model,
        emb.dims,
        emb.min_similarity,
    );
    std::fs::write(destino, contenido)
        .with_context(|| format!("escribir {}", destino.display()))?;
    Ok(())
}

/// Ruta por defecto del JSON de basic-memory, para `--from-basic-memory`.
pub fn ruta_basic_memory() -> Result<PathBuf> {
    Ok(dirs::home_dir()
        .context("sin HOME")?
        .join(".basic-memory/config.json"))
}
```

En `engine/src/lib.rs`, declarar el módulo en orden alfabético (entre
`pub mod indexer;` y `pub mod nota;`):

```rust
pub mod inicia;
```

En `engine/src/main.rs`, añadir la variante al `enum Comando` (zona línea 36),
**la primera de la lista** porque es el primer comando que corre un usuario
nuevo:

```rust
    /// Crea `~/.exo/config.toml`. Con `--from-basic-memory`, migra los valores
    /// de `~/.basic-memory/config.json` una sola vez.
    Init(ArgsInit),
```

Y el struct de args, junto a los demás `ArgsX`:

```rust
#[derive(clap::Args)]
struct ArgsInit {
    /// Raíz de la KB. Obligatorio salvo con `--from-basic-memory`.
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Nombre de la KB (prefijo de permalink). Obligatorio salvo con
    /// `--from-basic-memory`.
    #[arg(long)]
    name: Option<String>,
    /// Toma raíz, nombre y embeddings de `~/.basic-memory/config.json`.
    #[arg(long)]
    from_basic_memory: bool,
    /// Sobreescribe una config existente.
    #[arg(long)]
    force: bool,
    #[arg(long)]
    json: bool,
}
```

En `fn ejecuta` (zona 251), añadir el brazo como primero del `match`:

```rust
        Comando::Init(args) => init_cmd(args),
```

Y la función, junto a las demás `*_cmd`:

```rust
/// `exo init`: escribe la config propia. El volcado de la KB semilla llega en
/// G3 y se engancha aquí sin cambiar esta firma.
fn init_cmd(args: ArgsInit) -> Result<()> {
    let destino = exo::config::ruta_config()?;
    let db_default = dirs::home_dir()
        .context("sin HOME")?
        .join(".exo/index.db");

    let (kb, nombre, emb) = if args.from_basic_memory {
        let ruta = exo::inicia::ruta_basic_memory()?;
        let json = std::fs::read_to_string(&ruta)
            .with_context(|| format!("leer {}", ruta.display()))?;
        exo::inicia::desde_basic_memory(&json)?
    } else {
        let kb = args.kb.context("--kb es obligatorio sin --from-basic-memory")?;
        let nombre = args
            .name
            .context("--name es obligatorio sin --from-basic-memory")?;
        // Defaults del modelo de producción: los mismos que la línea base del
        // eval, declarados en la spec como posicionamiento (producto en español).
        let emb = exo::config::Embeddings {
            model: "jinaai/jina-embeddings-v2-base-es".to_string(),
            dims: 768,
            min_similarity: 0.35,
        };
        (kb, nombre, emb)
    };

    exo::inicia::escribe_config(&destino, &kb, &nombre, &emb, &db_default, args.force)?;

    if args.json {
        exo::envelope::emite(
            "init",
            serde_json::json!({
                "config": destino.display().to_string(),
                "kb": kb.display().to_string(),
                "name": nombre,
                "from_basic_memory": args.from_basic_memory,
            }),
        );
    } else {
        println!("config escrita en {}", destino.display());
        println!("KB: {} (name: {nombre})", kb.display());
        println!("siguiente: exo index --json");
    }
    Ok(())
}
```

- [ ] **Step 4: Correr los tests y verlos pasar**

```bash
cd /c/proyectos/homework/exo/engine
cargo test --test inicia 2>&1 | grep -E '^test result'
```

Expected: `test result: ok. 4 passed; 0 failed`.

- [ ] **Step 5: Verificación end-to-end contra la config real de esta máquina**

```bash
cd /c/proyectos/homework/exo/engine
cargo build --release
EXO_CONFIG=/tmp/exo-init-prueba.toml ./target/release/exo.exe init --from-basic-memory --json
cat /tmp/exo-init-prueba.toml
```

Expected: envelope con `"command":"init"`, y un TOML con
`name = "kb-demo"` y `path = "C:/proyectos/homework/kb-demo"`.
**Mira el fichero, no el exit code.**

- [ ] **Step 6: Commit**

```bash
cd /c/proyectos/homework/exo
git add engine/src/inicia.rs engine/src/config.rs engine/src/lib.rs engine/src/main.rs engine/tests/inicia.rs
git commit -m "feat(init): exo init escribe la config propia y migra desde basic-memory"
```

---

### Task 5: `--db` y `--kb` caen a la config (precedencia completa)

**Files:**
- Modify: `engine/src/main.rs` — `ArgsIndex`, `ArgsSearch`, `ArgsRecall`, `ArgsWriteNew`, `ArgsWriteAppend` y sus `*_cmd`
- Test: `engine/tests/precedencia.rs`

**Interfaces:**
- Consumes: `exo::config::{carga, expande_tilde}`, `exo::kb_desde_config` (Tasks 2 y 3).
- Produces, para uso interno de `main.rs`:
  - `fn resuelve_db(flag: Option<PathBuf>) -> Result<PathBuf>`
  - `fn resuelve_kb(flag: Option<PathBuf>) -> Result<PathBuf>`

- [ ] **Step 1: Escribir el test que falla**

Crear `engine/tests/precedencia.rs`:

```rust
//! Precedencia `flag > env > config`, comprobada por el binario real. Se
//! prueba por CLI y no por unidad porque la precedencia VIVE en el CLI: un
//! test de unidad sobre `resuelve_db` no demostraría que el flag gana.

use std::io::Write;
use std::process::Command;

fn bin() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("target");
    p.push(if cfg!(debug_assertions) { "debug" } else { "release" });
    p.push(if cfg!(windows) { "exo.exe" } else { "exo" });
    p
}

fn cfg_temporal(dir: &std::path::Path, db: &str) -> std::path::PathBuf {
    let ruta = dir.join("config.toml");
    let mut f = std::fs::File::create(&ruta).expect("crear");
    write!(
        f,
        r#"schema_version = 1
[kb]
path = "{}"
name = "fixture"
[index]
db = "{db}"
[embeddings]
model = "m"
dims = 768
min_similarity = 0.35
"#,
        dir.display().to_string().replace('\\', "/")
    )
    .expect("escribir");
    ruta
}

#[test]
fn el_flag_gana_a_la_env_y_la_env_gana_a_la_config() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = cfg_temporal(dir.path(), "C:/db-de-config.db");

    // Sin flag ni env: sale la de config. Se comprueba por el MENSAJE DE
    // ERROR, que nombra la ruta que intentó abrir — evidencia del artefacto,
    // no del exit code.
    let out = Command::new(bin())
        .args(["search", "--json", "loquesea"])
        .env("EXO_CONFIG", &cfg)
        .env_remove("EXO_DB")
        .output()
        .expect("correr");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("db-de-config"), "no cayó a la config: {err}");

    // Con env: gana la env.
    let out = Command::new(bin())
        .args(["search", "--json", "loquesea"])
        .env("EXO_CONFIG", &cfg)
        .env("EXO_DB", "C:/db-de-env.db")
        .output()
        .expect("correr");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("db-de-env"), "la env no ganó a la config: {err}");

    // Con flag: gana el flag, aunque la env esté puesta.
    let out = Command::new(bin())
        .args(["search", "--db", "C:/db-de-flag.db", "--json", "loquesea"])
        .env("EXO_CONFIG", &cfg)
        .env("EXO_DB", "C:/db-de-env.db")
        .output()
        .expect("correr");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("db-de-flag"), "el flag no ganó a la env: {err}");
}

#[test]
fn sin_config_ni_flag_el_error_dice_que_hacer() {
    let dir = tempfile::tempdir().expect("tempdir");
    let inexistente = dir.path().join("no-hay.toml");
    let out = Command::new(bin())
        .args(["search", "--json", "loquesea"])
        .env("EXO_CONFIG", &inexistente)
        .env_remove("EXO_DB")
        .output()
        .expect("correr");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("exo init"), "error sin salida accionable: {err}");
}
```

- [ ] **Step 2: Correr el test y verlo fallar**

```bash
cd /c/proyectos/homework/exo/engine
cargo build && cargo test --test precedencia 2>&1 | grep -E '^test |panicked' | head
```

Expected: FAIL — hoy `--db` es obligatorio, así que clap aborta antes de llegar
a leer nada y el stderr habla de un argumento requerido.

- [ ] **Step 3: Implementación**

En `engine/src/main.rs`, añadir las dos funciones auxiliares justo antes de
`fn ejecuta`:

```rust
/// Precedencia de la DB del índice: `--db` > `$EXO_DB` > `[index] db`.
///
/// `--db` era obligatorio «sin default (D6: un default sería config
/// encubierta)». Con config propia deja de serlo: un valor declarado en un
/// fichero del usuario no es config encubierta, es config.
fn resuelve_db(flag: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = flag {
        return Ok(p);
    }
    if let Ok(v) = std::env::var("EXO_DB") {
        if !v.is_empty() {
            return Ok(exo::config::expande_tilde(Path::new(&v)));
        }
    }
    let cfg = exo::config::carga()?;
    Ok(exo::config::expande_tilde(&cfg.index.db))
}

/// Precedencia de la raíz de la KB: `--kb` > `$EXO_KB` > `[kb] path`.
fn resuelve_kb(flag: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = flag {
        return Ok(p);
    }
    if let Ok(v) = std::env::var("EXO_KB") {
        if !v.is_empty() {
            return Ok(exo::config::expande_tilde(Path::new(&v)));
        }
    }
    exo::kb_desde_config()
}
```

Asegúrate de que `use std::path::Path;` está en los imports de `main.rs`;
añádelo si no.

En los cinco structs de args, cambiar el tipo de `db` de `PathBuf` a
`Option<PathBuf>` y actualizar su doc-comment. El patrón, idéntico en los cinco
(`ArgsWriteNew` zona 68, `ArgsWriteAppend` zona 100, `ArgsIndex` zona 119,
`ArgsSearch` zona 142, `ArgsRecall` zona 180):

```rust
    /// Fichero SQLite del índice. Default: `[index] db` de la config
    /// (`~/.exo/config.toml`). Precedencia: flag > $EXO_DB > config.
    #[arg(long)]
    db: Option<PathBuf>,
```

Y el doc-comment de los cinco `kb`:

```rust
    /// Raíz de la KB. Default: `[kb] path` de la config
    /// (`~/.exo/config.toml`). Precedencia: flag > $EXO_KB > config.
    #[arg(long)]
    kb: Option<PathBuf>,
```

En cada `*_cmd`, sustituir el uso directo por la resolución. Los sitios:

- `corre(...)`: donde use `args.db`, poner `let db = resuelve_db(args.db)?;`
  y usar `db`. Donde resuelva `kb`, sustituir el `match args.kb { … }` por
  `let kb = resuelve_kb(args.kb)?;`
- `busca_cmd`, `recall_cmd`, `write_new_cmd`, `write_append_cmd`: lo mismo.
  En `write_new_cmd` (zona 279) el bloque actual es:

```rust
    let kb = match args.kb {
        Some(p) => p,
        None => kb_desde_config().context("resolver raíz de la KB (--kb ausente)")?,
    };
```

  que pasa a ser:

```rust
    let kb = resuelve_kb(args.kb)?;
```

Además, en `write_new_cmd` sustituir la derivación del nombre de proyecto
(zona 285, el bloque `let proyecto = kb.file_name()…`) por la config, que es
lo que cierra el disenso del gate M4:

```rust
    // El nombre del proyecto sale de `[kb] name` de la config, EXPLÍCITO.
    // Antes se derivaba de `kb.file_name()` contra lo que decía la spec §3.1;
    // coincidían por suerte y el día que no, reventaba en silencio.
    let proyecto = exo::nombre_kb()?;
```

- [ ] **Step 4: Correr el test y verlo pasar**

```bash
cd /c/proyectos/homework/exo/engine
cargo build && cargo test --test precedencia 2>&1 | grep -E '^test result'
```

Expected: `test result: ok. 2 passed; 0 failed`.

- [ ] **Step 5: Verificar que los scripts que pasan `--db` explícito siguen funcionando**

```bash
cd /c/proyectos/homework/exo/engine
cargo build --release
./target/release/exo.exe search --db ~/.exo/index.db --type fts --limite 3 --json "memoria persistente" | head -c 400
```

Expected: envelope con 3 resultados. Los scripts de reflex pasan `--db`
explícito, así que el flag debe seguir ganando.

- [ ] **Step 6: Suite completa**

```bash
cd /c/proyectos/homework/exo/engine
cargo test --no-fail-fast > /tmp/t5.txt 2>&1; echo "CARGO_EXIT=$?"
grep -E '^test result' /tmp/t5.txt
```

Expected: `CARGO_EXIT=0`.

- [ ] **Step 7: Commit**

```bash
cd /c/proyectos/homework/exo
git add engine/src/main.rs engine/tests/precedencia.rs
git commit -m "feat(cli): --db y --kb caen a la config propia con precedencia flag > env > config"
```

---

### Task 6: D8 — claves del envelope al inglés, `SCHEMA_VERSION` 1→2

**Files:**
- Modify: `engine/src/envelope.rs:6`
- Modify: `engine/src/recall.rs:37-55` (`NotaRecall`, `Recall`)
- Modify: `engine/src/indexer.rs:50-60` (`Resumen`)
- Modify: `engine/src/buscador.rs:14-50` (`Resultado.ruta`, `Busqueda.avisos`)
- Test: `engine/tests/contrato_envelope.rs`

**Interfaces:**
- Consumes: nada de tareas anteriores.
- Produces: contrato JSON v2. Consumido por la Task 7 (scripts).

Los identificadores de Rust **no se tocan** —están fuera de scope por spec—:
el renombrado va con `#[serde(rename = "...")]`, patrón que ya existe en
`buscador.rs` (`#[serde(rename = "type")] pub tipo: String`).

Mapeo completo:

| Struct | Campo Rust | Clave JSON nueva |
|---|---|---|
| `NotaRecall` | `ruta` | `path` |
| `NotaRecall` | `titulo` | `title` |
| `Recall` | `modo` | `mode` |
| `Recall` | `truncado` | `truncated` |
| `Recall` | `notas` | `notes` |
| `Resumen` | `indexadas` | `indexed` |
| `Resumen` | `saltadas` | `skipped` |
| `Resumen` | `borradas` | `deleted` |
| `Resumen` | `trozos_embebidos` | `chunks_embedded` |
| `Resumen` | `trozos_reusados` | `chunks_reused` |
| `Resultado` | `ruta` | `path` |
| `Busqueda` | `avisos` | `warnings` |

`Recall.cap_bytes`, `Recall.query`, `NotaRecall.{permalink,tier,score,snippet}`,
`Busqueda.{query,search_type,elapsed_s,results}` y `Resultado.{permalink,type,score}`
**ya están en inglés y no se tocan**.

- [ ] **Step 1: Escribir el test que falla**

Crear `engine/tests/contrato_envelope.rs`:

```rust
//! El contrato JSON público v2. Este test es el gate: si alguien renombra una
//! clave sin subir `SCHEMA_VERSION`, aquí se pone rojo.
//!
//! Se comprueba sobre `serde_json::to_value` de structs construidos a mano,
//! no sobre una corrida real: el contrato es de FORMA, y una corrida real lo
//! ataría además a tener índice y modelo en la máquina.

#[test]
fn schema_version_es_2() {
    assert_eq!(exo::envelope::SCHEMA_VERSION, 2);
}

#[test]
fn las_claves_de_recall_estan_en_ingles() {
    let r = exo::recall::Recall {
        modo: "arranque".into(),
        query: None,
        cap_bytes: 2048,
        truncado: false,
        notas: vec![exo::recall::NotaRecall {
            permalink: "kb/core/x".into(),
            ruta: "core/x.md".into(),
            titulo: "X".into(),
            tier: Some("core".into()),
            score: None,
            snippet: None,
        }],
    };
    let v = serde_json::to_value(&r).expect("serializar");
    let obj = v.as_object().expect("objeto");
    for k in ["mode", "query", "cap_bytes", "truncated", "notes"] {
        assert!(obj.contains_key(k), "falta la clave {k} en {v}");
    }
    for k in ["modo", "truncado", "notas"] {
        assert!(!obj.contains_key(k), "sobrevive la clave española {k}");
    }
    let nota = &v["notes"][0];
    for k in ["permalink", "path", "title", "tier", "score", "snippet"] {
        assert!(nota.get(k).is_some(), "falta {k} en la nota: {nota}");
    }
    assert!(nota.get("ruta").is_none(), "sobrevive `ruta`");
    assert!(nota.get("titulo").is_none(), "sobrevive `titulo`");
}

#[test]
fn las_claves_de_index_estan_en_ingles() {
    let r = exo::indexer::Resumen {
        indexadas: 1,
        saltadas: 2,
        borradas: 3,
        trozos_embebidos: 4,
        trozos_reusados: 5,
    };
    let v = serde_json::to_value(&r).expect("serializar");
    let obj = v.as_object().expect("objeto");
    for k in ["indexed", "skipped", "deleted", "chunks_embedded", "chunks_reused"] {
        assert!(obj.contains_key(k), "falta {k} en {v}");
    }
    assert_eq!(v["indexed"], 1);
    assert_eq!(v["chunks_embedded"], 4);
    for k in ["indexadas", "saltadas", "borradas", "trozos_embebidos", "trozos_reusados"] {
        assert!(!obj.contains_key(k), "sobrevive la clave española {k}");
    }
}

#[test]
fn las_claves_de_search_estan_en_ingles() {
    let b = exo::buscador::Busqueda {
        query: "q".into(),
        search_type: "fts".into(),
        elapsed_s: 0.1,
        results: vec![exo::buscador::Resultado {
            permalink: "kb/x".into(),
            tipo: "entity".into(),
            score: 1.0,
            ruta: Some("x.md".into()),
        }],
        avisos: vec!["algo".into()],
    };
    let v = serde_json::to_value(&b).expect("serializar");
    assert!(v.get("warnings").is_some(), "falta `warnings`: {v}");
    assert!(v.get("avisos").is_none(), "sobrevive `avisos`");
    assert!(v["results"][0].get("path").is_some(), "falta `path` en el resultado");
    assert!(v["results"][0].get("ruta").is_none(), "sobrevive `ruta`");
}
```

- [ ] **Step 2: Correr el test y verlo fallar**

```bash
cd /c/proyectos/homework/exo/engine
cargo test --test contrato_envelope 2>&1 | grep -E '^test |panicked|assertion' | head -20
```

Expected: FAIL — `SCHEMA_VERSION` es 1 y las claves salen en español.

- [ ] **Step 3: Aplicar los renames**

En `engine/src/envelope.rs`, línea 6:

```rust
pub const SCHEMA_VERSION: u32 = 2;
```

y actualizar su doc-comment añadiendo, tras la frase existente:

```rust
/// v2 (2026-08-26, D8): claves de `data` al inglés antes del 1.0 público.
```

En `engine/src/recall.rs`, en `NotaRecall`:

```rust
    #[serde(rename = "path")]
    pub ruta: String,
    #[serde(rename = "title")]
    pub titulo: String,
```

y en `Recall`:

```rust
    #[serde(rename = "mode")]
    pub modo: String,
    pub query: Option<String>,
    pub cap_bytes: usize,
    #[serde(rename = "truncated")]
    pub truncado: bool,
    #[serde(rename = "notes")]
    pub notas: Vec<NotaRecall>,
```

En `engine/src/indexer.rs`, en `Resumen`:

```rust
    #[serde(rename = "indexed")]
    pub indexadas: usize,
    #[serde(rename = "skipped")]
    pub saltadas: usize,
    #[serde(rename = "deleted")]
    pub borradas: usize,
    #[serde(rename = "chunks_embedded")]
    pub trozos_embebidos: usize,
    #[serde(rename = "chunks_reused")]
    pub trozos_reusados: usize,
```

(conservando los doc-comments que ya tienen `trozos_embebidos` y
`trozos_reusados`).

En `engine/src/buscador.rs`, en `Resultado` (conservando el doc-comment largo
que explica por qué el campo existe):

```rust
    #[serde(rename = "path")]
    pub ruta: Option<String>,
```

y en `Busqueda`:

```rust
    #[serde(rename = "warnings", skip_serializing_if = "Vec::is_empty")]
    pub avisos: Vec<String>,
```

- [ ] **Step 4: Correr el test y verlo pasar**

```bash
cd /c/proyectos/homework/exo/engine
cargo test --test contrato_envelope 2>&1 | grep -E '^test result'
```

Expected: `test result: ok. 4 passed; 0 failed`.

- [ ] **Step 5: Ver qué tests existentes rompe (es lo esperado, no un accidente)**

```bash
cd /c/proyectos/homework/exo/engine
cargo test --no-fail-fast > /tmp/t6.txt 2>&1; echo "CARGO_EXIT=$?"
grep -E '^test result|FAILED|^failures:' /tmp/t6.txt | head -20
```

Los tests de `tests/recall.rs`, `tests/recall_contenido.rs`, `tests/indexer.rs`
y `tests/buscador.rs` que aserten sobre el JSON con claves españolas van a
caer. **Es correcto**: son el contrato viejo. Actualízalos a las claves nuevas,
uno por uno, sin cambiar lo que miden.

- [ ] **Step 6: Suite verde**

```bash
cd /c/proyectos/homework/exo/engine
cargo test --no-fail-fast > /tmp/t6b.txt 2>&1; echo "CARGO_EXIT=$?"
grep -E '^test result' /tmp/t6b.txt
```

Expected: `CARGO_EXIT=0`.

- [ ] **Step 7: Commit**

```bash
cd /c/proyectos/homework/exo
git add engine/src/envelope.rs engine/src/recall.rs engine/src/indexer.rs engine/src/buscador.rs engine/tests/
git commit -m "feat(envelope)!: claves de data al inglés, SCHEMA_VERSION 1->2 (D8)

BREAKING CHANGE: notas->notes, ruta->path, titulo->title, truncado->truncated,
modo->mode, indexadas->indexed, saltadas->skipped, borradas->deleted,
trozos_embebidos->chunks_embedded, trozos_reusados->chunks_reused,
avisos->warnings. Consumidor a migrar: plugins/reflex/scripts/."
```

---

### Task 7: Migrar los consumidores del envelope

Blast radius medido hoy: **4 ficheros**. Verifícalo antes de empezar; si el
conteo no cuadra, hay un consumidor nuevo que este plan no vio.

**Files:**
- Modify: `plugins/reflex/scripts/recall-inject.sh` (12 ocurrencias)
- Modify: `plugins/reflex/scripts/test-recall-inject.sh` (42, casi todas fixtures)
- Modify: `plugins/reflex/scripts/exo-recall.sh` (3)
- Modify: `plugins/reflex/scripts/compose-inject.sh` (8)

**Interfaces:**
- Consumes: contrato v2 de la Task 6.
- Produces: scripts que hablan v2. Los consume la ola 1B (Plan 2).

- [ ] **Step 1: Confirmar el blast radius**

```bash
cd /c/proyectos/homework/exo
for f in plugins/reflex/scripts/recall-inject.sh plugins/reflex/scripts/test-recall-inject.sh plugins/reflex/scripts/exo-recall.sh plugins/reflex/scripts/compose-inject.sh; do
  printf "%s: %s\n" "$f" "$(grep -c 'notas\|\bruta\b\|titulo\|truncado\|indexadas\|trozos_\|avisos' "$f")"
done
grep -rln 'data\.notas\|has("notas")\|data\.truncado\|indexadas\|trozos_embebidos' plugins evals --include=*.sh --include=*.py 2>/dev/null
```

Expected: 12 / 42 / 3 / 8, y la segunda orden lista **solo** ficheros de esos
cuatro. Si aparece otro, añádelo a esta tarea antes de seguir.

- [ ] **Step 2: Ver los tests de script fallar contra el binario v2**

```bash
cd /c/proyectos/homework/exo/engine && cargo build --release && cd ..
bash plugins/reflex/scripts/test-recall-inject.sh; echo "EXIT=$?"
```

Expected: FAIL. Los fixtures del test hablan v1 y el script comprueba
`has("notas")`, así que el camino real degradaría con
`reason=error err=envelope-ilegible`.

- [ ] **Step 3: Migrar cada fichero**

Revisa **cada match a mano**: `ruta` aparece también en comentarios en prosa,
donde no hay que tocarla. Solo se cambian las que son claves JSON.

En `recall-inject.sh`, los cambios de fondo son:

```bash
# línea ~177 — el check de envelope legible
if ! printf '%s' "$SALIDA" | jq -e 'has("data") and (.data | has("notes"))' >/dev/null 2>&1; then

# línea ~186 — el aviso de truncado
if [ "$(printf '%s' "$SALIDA" | jq -r '.data.truncated // false' 2>/dev/null)" = "true" ]; then

# línea ~246 — composición del bloque
( .data.notes
  | map(select(.permalink != $excluir))
  | .[0:$max]
  | map({ path: (.path | sane), title: (.title | sane), snippet: (.snippet | sane) })
) as $hits

# línea ~299 — dedup de permalinks
      '[.data.notes[] | select(.permalink != $excluir)][0:$max]
       | map(.permalink) | join(",")'
```

Ojo con el `map({...})` de la línea ~246: las claves de salida (`path`,
`title`) también se renombran, así que hay que seguir el rastro de dónde se
consumen esas claves más abajo en el mismo script y actualizarlas.

En `test-recall-inject.sh`, los fixtures JSON: `"ruta"`→`"path"`,
`"titulo"`→`"title"`, `"notas"`→`"notes"`, y en las construcciones `jq -n`
las claves `ruta:`→`path:`, `titulo:`→`title:`, `notas:`→`notes:`.

- [ ] **Step 4: Correr las suites de script y verlas pasar**

```bash
cd /c/proyectos/homework/exo
for t in test-recall-inject test-compose-inject test-exo-index; do
  printf "%s: " "$t"
  bash "plugins/reflex/scripts/$t.sh" >/tmp/$t.log 2>&1 && echo OK || { echo "FAIL"; tail -20 /tmp/$t.log; }
done
```

Expected: los tres `OK`.

- [ ] **Step 5: Verificación end-to-end del camino real (el que importa)**

```bash
cd /c/proyectos/homework/exo
cp engine/target/release/exo.exe ~/.local/bin/exo.exe
echo '{"session_id":"t","source":"startup"}' \
  | bash plugins/reflex/scripts/exo-recall.sh \
  | jq -r '.hookSpecificOutput.additionalContext' | grep -c 'Contrato de memoria'
```

Expected: `1`. Un `0` significa que está sirviendo el fallback embebido —el
arranque mentiría en silencio, que es justo el fallo que este proyecto
persigue.

```bash
grep recall-fallback ~/.claude/reflex-log.jsonl | tail -3
```

Expected: ninguna línea nueva con `reason=no-engine` / `no-index` / `no-contract`.

- [ ] **Step 6: Commit**

```bash
cd /c/proyectos/homework/exo
git add plugins/reflex/scripts/
git commit -m "fix(reflex): migrar los scripts al contrato v2 del envelope"
```

---

### Task 8: `exo config --json` y des-hardcodear `kb-demo` de los scripts

La spec lista en la superficie de G1 tres scripts con el nombre de la KB
hardcodeado: `exo-recall.sh:35`, `recall-inject.sh:201` y `compose-inject.sh:27`.
Los dos primeros lo llevan como **default detrás de un seam de entorno** (ya
están medio parametrizados); el tercero además conserva un fallback que lee
`~/.basic-memory/config.json` con jq — el último resto del cordón, y jq no sabe
leer TOML, así que hace falta una superficie mínima que se lo dé.

**Files:**
- Modify: `engine/src/main.rs` (`enum Comando`, `fn ejecuta`, nueva `config_cmd`)
- Modify: `plugins/reflex/scripts/exo-recall.sh:35`
- Modify: `plugins/reflex/scripts/recall-inject.sh:201`
- Modify: `plugins/reflex/scripts/compose-inject.sh:20-30`
- Test: `engine/tests/config_cmd.rs`

**Interfaces:**
- Consumes: `exo::config::{carga, expande_tilde}` (Task 2), `exo::envelope::emite`.
- Produces:
  - CLI `exo config --json` → envelope con
    `data = {"kb":{"path":<absoluta>,"name":<str>},"index":{"db":<absoluta>},"embeddings":{...}}`.
    Las rutas salen **ya expandidas**: un `~` sin resolver en manos de un
    script es un fichero que no existe.

- [ ] **Step 1: Escribir el test que falla**

Crear `engine/tests/config_cmd.rs`:

```rust
//! `exo config --json`: la superficie mínima que le da a un script en shell
//! los valores de una config TOML, que jq no sabe leer. Existe para que
//! ningún consumidor tenga que volver a hardcodear el nombre de la KB.

use std::io::Write;
use std::process::Command;

fn bin() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("target");
    p.push(if cfg!(debug_assertions) { "debug" } else { "release" });
    p.push(if cfg!(windows) { "exo.exe" } else { "exo" });
    p
}

#[test]
fn config_json_emite_la_config_con_rutas_expandidas() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ruta = dir.path().join("config.toml");
    let mut f = std::fs::File::create(&ruta).expect("crear");
    write!(
        f,
        r#"schema_version = 1
[kb]
path = "C:/kb/demo"
name = "demo"
[index]
db = "~/.exo/index.db"
[embeddings]
model = "m/x"
dims = 768
min_similarity = 0.35
"#
    )
    .expect("escribir");

    let out = Command::new(bin())
        .args(["config", "--json"])
        .env("EXO_CONFIG", &ruta)
        .output()
        .expect("correr");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));

    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("stdout debe ser envelope");
    assert_eq!(v["command"], "config");
    assert_eq!(v["data"]["kb"]["name"], "demo");
    assert_eq!(v["data"]["embeddings"]["dims"], 768);

    // La ruta de la DB sale EXPANDIDA: un `~` sin resolver en manos de un
    // script de shell es un fichero que no existe, y el fallo sería silencioso.
    let db = v["data"]["index"]["db"].as_str().expect("db");
    assert!(!db.starts_with('~'), "la ruta llegó sin expandir: {db}");
    assert!(db.contains(".exo"), "ruta inesperada: {db}");
}
```

- [ ] **Step 2: Correr el test y verlo fallar**

```bash
cd /c/proyectos/homework/exo/engine
cargo build && cargo test --test config_cmd 2>&1 | grep -E '^test |panicked' | head
```

Expected: FAIL — clap no conoce el subcomando `config`.

- [ ] **Step 3: Implementar el subcomando**

En `engine/src/main.rs`, añadir al `enum Comando`, justo después de `Init`:

```rust
    /// Emite la config efectiva como envelope JSON, con las rutas ya
    /// expandidas. Existe para los consumidores en shell: jq no lee TOML.
    Config(ArgsConfig),
```

El struct de args:

```rust
#[derive(clap::Args)]
struct ArgsConfig {
    #[arg(long)]
    json: bool,
}
```

El brazo en `fn ejecuta`, tras el de `Init`:

```rust
        Comando::Config(args) => {
            let json = args.json;
            config_cmd(args).map(|_| json)
        }
```

Y la función:

```rust
/// `exo config`: la config efectiva, con rutas expandidas.
fn config_cmd(args: ArgsConfig) -> Result<()> {
    let cfg = exo::config::carga()?;
    let kb = exo::config::expande_tilde(&cfg.kb.path);
    let db = exo::config::expande_tilde(&cfg.index.db);
    let data = serde_json::json!({
        "kb": { "path": kb.display().to_string(), "name": cfg.kb.name },
        "index": { "db": db.display().to_string() },
        "embeddings": {
            "model": cfg.embeddings.model,
            "dims": cfg.embeddings.dims,
            "min_similarity": cfg.embeddings.min_similarity,
        },
    });
    if args.json {
        exo::envelope::emite("config", data);
    } else {
        println!("kb.path   {}", kb.display());
        println!("kb.name   {}", cfg.kb.name);
        println!("index.db  {}", db.display());
        println!("model     {}", cfg.embeddings.model);
    }
    Ok(())
}
```

- [ ] **Step 4: Correr el test y verlo pasar**

```bash
cd /c/proyectos/homework/exo/engine
cargo build && cargo test --test config_cmd 2>&1 | grep -E '^test result'
```

Expected: `test result: ok. 1 passed; 0 failed`.

- [ ] **Step 5: Des-hardcodear los tres scripts**

En `plugins/reflex/scripts/exo-recall.sh`, sustituir la línea 35:

```bash
# El nombre de la KB sale de la config del engine, no de un literal: era el
# último sitio donde `kb-demo` seguía cableado en el camino de arranque.
EXO_KB_NAME="${EXO_KB_NAME:-$("$EXO_BIN" config --json 2>/dev/null | jq -r '.data.kb.name // empty')}"
EXO_NOTA="${EXO_RECALL_NOTA:-${EXO_KB_NAME:+$EXO_KB_NAME/}core/core-index}"
```

Si `exo config` falla (no hay config todavía), `EXO_KB_NAME` queda vacío y
`EXO_NOTA` cae a `core/core-index` sin prefijo. El script ya tiene su camino de
degradación con log — **comprueba que ese caso deja evento en
`reflex-log.jsonl`** y no pasa mudo.

En `plugins/reflex/scripts/recall-inject.sh`, línea 201, el mismo patrón para
`EXO_EXCLUIR`.

En `plugins/reflex/scripts/compose-inject.sh`, sustituir el bloque de fallback
a basic-memory (líneas ~26-29) por:

```bash
if [ -z "$KB" ]; then
  # Antes caía a `~/.basic-memory/config.json` con jq. Ese era el cordón: el
  # sustituto pidiéndole la ruta al sustituido. Ahora se la pide al engine.
  KB="$(exo config --json 2>/dev/null | jq -r '.data.kb.path // empty')" || KB=""
fi
```

- [ ] **Step 6: Verificar que no queda ni un literal ni una lectura de basic-memory**

```bash
cd /c/proyectos/homework/exo
grep -rn 'kb-demo' plugins/reflex/scripts/*.sh | grep -v 'test-'
grep -rn 'basic-memory' plugins/reflex/scripts/*.sh | grep -v 'test-'
```

Expected: **cero líneas en ambos**. Los `test-*.sh` sí pueden nombrarlo: son
fixtures, y un fixture con nombre propio es legítimo.

- [ ] **Step 7: Verificación end-to-end del arranque**

```bash
cd /c/proyectos/homework/exo/engine && cargo build --release
cp target/release/exo.exe ~/.local/bin/exo.exe
cd /c/proyectos/homework/exo
echo '{"session_id":"t","source":"startup"}' | bash plugins/reflex/scripts/exo-recall.sh \
  | jq -r '.hookSpecificOutput.additionalContext' | grep -c 'Contrato de memoria'
```

Expected: `1`. Un `0` significa que la nota de arranque no se resolvió y está
sirviendo el fallback embebido.

- [ ] **Step 8: Suites de script y commit**

```bash
cd /c/proyectos/homework/exo
for t in plugins/reflex/scripts/test-*.sh; do
  printf "%s: " "$(basename "$t")"; bash "$t" >/tmp/h-$(basename "$t").log 2>&1 && echo OK || { echo FAIL; tail -15 /tmp/h-$(basename "$t").log; }
done
git add engine/src/main.rs engine/tests/config_cmd.rs plugins/reflex/scripts/
git commit -m "feat(config): exo config --json y fin del literal kb-demo en los scripts"
```

---

### Task 9: Hallazgo #3 del gate M4 — el rechazo exit 3 emite envelope con `--json`

Hoy `main()` intercepta `Rechazo`, escribe una línea humana en stderr y sale 3.
Con `--json` **no emite envelope**, pese a que la spec de write §3.3 promete
`data.dup_candidatas`. Es contrato por prosa en una superficie que un tercero
va a consumir.

**Files:**
- Modify: `engine/src/main.rs:236-248` (`fn main`) y `fn ejecuta`
- Modify: `engine/src/escritor.rs:20-24` (`enum Rechazo`, añadir accesor)
- Test: `engine/tests/rechazo_envelope.rs`

**Interfaces:**
- Consumes: `exo::envelope::emite` (existente), `exo::escritor::{Rechazo, Candidata}` (existentes).
- Produces:
  - `impl Rechazo { pub fn data(&self) -> serde_json::Value }`
  - `fn ejecuta() -> Result<bool>` — el `bool` es «se pidió `--json`», y es lo
    que permite a `main()` decidir el formato de salida del rechazo.

- [ ] **Step 1: Escribir el test que falla**

Crear `engine/tests/rechazo_envelope.rs`:

```rust
//! El rechazo del dup-gate emite envelope cuando se pide `--json`.
//!
//! Contrato: exit 3 (que es por donde gatea el consumidor, jamás por campos
//! de `data`) Y envelope en stdout con `data.dup_candidatas`. Las dos cosas,
//! no una: el exit code sigue siendo el gate y el envelope es el detalle.

#[test]
fn el_data_del_rechazo_duplicada_lleva_las_candidatas() {
    let r = exo::escritor::Rechazo::Duplicada {
        candidatas: vec![exo::escritor::Candidata {
            permalink: "kb/projects/x".into(),
            score: 0.87,
        }],
    };
    let v = r.data();
    assert_eq!(v["motivo"], "duplicada");
    let c = &v["dup_candidatas"][0];
    assert_eq!(c["permalink"], "kb/projects/x");
    assert_eq!(c["score"], 0.87);
}

#[test]
fn el_data_del_rechazo_append_a_canon_lleva_el_tier() {
    let r = exo::escritor::Rechazo::AppendACanon {
        tier: "core".into(),
    };
    let v = r.data();
    assert_eq!(v["motivo"], "append_a_canon");
    assert_eq!(v["tier"], "core");
    // Un rechazo que no es por duplicado NO inventa una lista vacía de
    // candidatas: ausencia de campo, no campo vacío.
    assert!(v.get("dup_candidatas").is_none());
}
```

Y el test end-to-end, en el mismo fichero:

```rust
#[test]
fn write_new_rechazado_con_json_emite_envelope_y_sale_3() {
    // Se apoya en la KB real y su índice: es el único sitio donde hay un
    // duplicado que el gate reconozca. Si no existen, el test se salta
    // ruidosamente en vez de dar un verde falso.
    let db = dirs::home_dir().expect("home").join(".exo/index.db");
    if !db.exists() {
        eprintln!("SKIP: no hay índice en {} — este test necesita la KB real", db.display());
        return;
    }
    let mut bin = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    bin.push("target");
    bin.push(if cfg!(debug_assertions) { "debug" } else { "release" });
    bin.push(if cfg!(windows) { "exo.exe" } else { "exo" });

    let cuerpo = tempfile::NamedTempFile::new().expect("tmp");
    let out = std::process::Command::new(&bin)
        .args([
            "write", "new",
            "--db", db.to_str().unwrap(),
            "--dir", "projects",
            "--titulo", "exo — framework unificado de trabajo agéntico",
            "--from", cuerpo.path().to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("correr");

    assert_eq!(out.status.code(), Some(3), "el gate debe salir 3");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value =
        serde_json::from_str(stdout.trim()).unwrap_or_else(|e| panic!("stdout no es envelope ({e}): {stdout}"));
    assert_eq!(v["command"], "write.new");
    assert_eq!(v["data"]["motivo"], "duplicada");
    assert!(
        v["data"]["dup_candidatas"].as_array().is_some_and(|a| !a.is_empty()),
        "sin candidatas en el envelope: {v}"
    );
}
```

- [ ] **Step 2: Correr el test y verlo fallar**

```bash
cd /c/proyectos/homework/exo/engine
cargo test --test rechazo_envelope 2>&1 | tail -15
```

Expected: FAIL en compilación — `no method named 'data' found for enum 'Rechazo'`.

- [ ] **Step 3: Implementación**

En `engine/src/escritor.rs`, tras el `impl std::error::Error for Rechazo {}`:

```rust
impl Rechazo {
    /// `data` del envelope de un rechazo (spec de write §3.3). Existe para que
    /// el consumidor tenga el detalle en JSON, no solo una línea de prosa en
    /// stderr — un contrato prometido por escrito y servido por prosa es la
    /// definición de contrato no falsable.
    ///
    /// El exit code sigue siendo el gate: esto es el detalle, no la señal.
    pub fn data(&self) -> serde_json::Value {
        match self {
            Rechazo::Duplicada { candidatas } => serde_json::json!({
                "motivo": "duplicada",
                "dup_candidatas": candidatas,
            }),
            Rechazo::AppendACanon { tier } => serde_json::json!({
                "motivo": "append_a_canon",
                "tier": tier,
            }),
        }
    }
}
```

En `engine/src/main.rs`, `fn ejecuta` pasa a devolver el flag de json. Cada
brazo del `match` devuelve el `args.json` de su comando:

```rust
/// Devuelve `true` si el comando pidió `--json`, para que `main` sepa en qué
/// formato reportar un rechazo del gate.
fn ejecuta() -> Result<bool> {
    let cli = Cli::parse();
    match cli.comando {
        Comando::Init(args) => {
            let json = args.json;
            init_cmd(args).map(|_| json)
        }
        Comando::Index(args) => {
            let json = args.json;
            corre("index", args, false).map(|_| json)
        }
        Comando::Rebuild(args) => {
            let json = args.json;
            corre("rebuild", args, true).map(|_| json)
        }
        Comando::Search(args) => {
            let json = args.json;
            busca_cmd(args).map(|_| json)
        }
        Comando::Recall(args) => {
            let json = args.json;
            recall_cmd(args).map(|_| json)
        }
        Comando::Write(sub) => match sub {
            ComandoWrite::New(args) => {
                let json = args.json;
                write_new_cmd(args).map(|_| json)
            }
            ComandoWrite::Append(args) => {
                let json = args.json;
                write_append_cmd(args).map(|_| json)
            }
        },
    }
}
```

Pero `main()` necesita saber el `json` **aunque `ejecuta` haya fallado**, y en
la rama de error el flag se ha perdido con el `args` movido. La forma más
simple y sin estado global es leerlo del propio argv, que es la fuente de
verdad y no puede desincronizarse:

```rust
fn main() {
    // Se lee de argv y no del `args` parseado porque en la rama de error el
    // `args` ya se movió dentro de `ejecuta`. argv es la fuente de verdad y no
    // puede desincronizarse de lo que el usuario pidió.
    let quiere_json = std::env::args().any(|a| a == "--json");
    match ejecuta() {
        Ok(_) => {}
        Err(e) => {
            // Un gate rechazado NO es un error del sistema: es una decisión que
            // se le devuelve al llamador (nota duplicada, append al canon). Sale
            // con 3 para que el consumidor lo distinga de un fallo real por exit
            // code —jamás parseando `data`— y pueda reintentar con `--force`.
            if let Some(rechazo) = e.downcast_ref::<exo::escritor::Rechazo>() {
                eprintln!("rechazado: {rechazo}");
                if quiere_json {
                    exo::envelope::emite("write.rechazo", rechazo.data());
                }
                std::process::exit(3);
            }
            eprintln!("error: {e:#}");
            std::process::exit(1);
        }
    }
}
```

Ajusta el `command` del envelope al que espere el test (`"write.new"` si el
rechazo viene de `new`); si no es distinguible desde `main`, cambia el test
para esperar `"write.rechazo"` y anótalo en la spec de write como el nombre
del contrato. **Elige uno y déjalo escrito**: el fallo aquí sería dejar la
spec diciendo una cosa y el código otra, que es el hallazgo que esta tarea
viene a cerrar.

- [ ] **Step 4: Correr el test y verlo pasar**

```bash
cd /c/proyectos/homework/exo/engine
cargo build --release
cargo test --release --test rechazo_envelope 2>&1 | grep -E '^test result|SKIP'
```

Expected: `test result: ok. 3 passed`. Si el tercero dice `SKIP`, corre
`exo index` antes y repite —un skip no es un verde.

- [ ] **Step 5: Actualizar la spec de write si hizo falta**

```bash
cd /c/proyectos/homework/exo
grep -rn "dup_candidatas" docs/superpowers/specs/
```

Comprueba que lo que dice la spec y lo que emite el binario son la misma cosa.
Si divergen, corrige la spec en este mismo commit.

- [ ] **Step 6: Suite completa y commit**

```bash
cd /c/proyectos/homework/exo/engine
cargo test --release --no-fail-fast > /tmp/t8.txt 2>&1; echo "CARGO_EXIT=$?"
grep -E '^test result' /tmp/t8.txt
cd /c/proyectos/homework/exo
git add engine/src/escritor.rs engine/src/main.rs engine/tests/rechazo_envelope.rs docs/
git commit -m "fix(write): el rechazo del dup-gate emite envelope con --json (hallazgo #3 del gate M4)"
```

---

### Task 10: Cierre de la ola — actualizar backlog y README

**Files:**
- Modify: `docs/backlog.md` (item Alta y hallazgo #3 de la sección Media)
- Modify: `README.md` (bloque de estado)

**Interfaces:**
- Consumes: todo lo anterior.
- Produces: nada de código.

- [ ] **Step 1: Verificar de una vez todo lo que la ola prometía**

```bash
cd /c/proyectos/homework/exo/engine
echo "--- 1. cero lecturas de basic-memory fuera de la migración explícita"
grep -rn "basic-memory" src/ | grep -v "inicia.rs" | grep -v "^src/.*://"
echo "--- 2. cero literales kb-demo en el engine"
grep -rn "kb-demo" src/ | grep -v "///" | grep -v "//"
echo "--- 3. suite verde"
cargo test --release --no-fail-fast > /tmp/t9.txt 2>&1; echo "CARGO_EXIT=$?"
grep -E '^test result' /tmp/t9.txt
echo "--- 4. fmt limpio"
cargo fmt --check; echo "FMT_EXIT=$?"
```

Expected: 1 y 2 sin salida, `CARGO_EXIT=0`, `FMT_EXIT=0`.

- [ ] **Step 2: Marcar cerrado el item Alta del backlog**

En `docs/backlog.md`, mover el item **«Adelantar M5a-02: config propia y
des-hardcodear la KB»** de `## Alta` a `## Cerrado con evidencia (para no
re-proponer)`, reescrito como:

```markdown
- [x] **M5a-02 config propia: cerrado el 2026-08-26.** El engine arranca con
  `~/.exo/config.toml` (`src/config.rs`), con precedencia
  `flag > env > config > error accionable` y sin fallback a basic-memory: la
  única lectura que sobrevive es `exo init --from-basic-memory`, explícita y
  borrable. Cierra de paso el disenso del gate M4 — el prefijo de permalink
  sale de `[kb] name`, no de `kb.file_name()`. Verificado: `grep -rn
  "basic-memory" engine/src/ | grep -v inicia.rs` sin salida.
```

Y en la sección `## Media`, marcar el hallazgo **#3** como cerrado con la
evidencia del test `rechazo_envelope.rs`.

- [ ] **Step 3: Actualizar el bloque de estado del README**

En `README.md`, en la lista de estado, sustituir la línea que empieza por
`- Estado: M0, M1a y M2 …` por una que diga qué está cerrado hoy y apunte al
backlog y a la spec de exo genérico. El item «Rot documental» de la sección
Baja del backlog lleva dos campañas abierto; ciérralo aquí.

- [ ] **Step 4: Commit**

```bash
cd /c/proyectos/homework/exo
git add docs/backlog.md README.md
git commit -m "docs: cerrar M5a-02 y el hallazgo #3 en el backlog, actualizar estado del README"
```

---

## Decisión que este plan deja abierta

**Los flags del CLI siguen en español** (`--limite`, `--titulo`, `--cap-bytes`,
`--contenido`, `--refresca`, `--min-similitud`, `--escala-fts`, `--crea`,
`--nota`). D7 cubre «nombres de skills y verbos del CLI»; los subcomandos ya
son ingleses (`index`, `search`, `recall`, `write`, `init`), pero los flags no
están cubiertos por ninguna decisión.

Medido: **44 ocurrencias en 7 ficheros vivos** —`exo-index.sh`,
`exo-recall.sh`, `recall-inject.sh`, `test-recall-inject.sh`,
`consolida/SKILL.md`, `replay-engine.py`, `diagnostica-lectura-a.py`—. Cuatro
de los siete ya los toca la Task 7.

**Recomendación:** hacerlo, y hacerlo aquí. Un 1.0 público con `--limite` al
lado de `--json` es la misma incoherencia que D8 existe para matar, y la
ventana barata es esta —después rompe a terceros—. Se implementa como tarea
propia con `#[arg(long = "limit", alias = "limite")]`, que da migración sin
rotura instantánea.

**No está en el plan porque la spec no lo decide.** Si se aprueba, entra como
Task 10 antes del cierre; si no, se anota en la spec como decisión consciente
de dejarlos en español para que nadie la vuelva a abrir.
