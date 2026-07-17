# M2 Campaña 1 — Scaffold engine + spec/gold del indexer: Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Dejar el crate `engine/` compilando con las 3 dependencias nativas des-riesgadas (M2-01) y la spec del indexer + gold de paridad de corpus + envelope JSON sellados antes de escribir el indexer (M2-02).

**Architecture:** Crate Rust único en `engine/` (lib+bin), binario `exo`. M2-02 es lane diseño: produce documentos y un probe read-only contra el índice de basic-memory que se convierte en el oráculo de M2-03; no escribe código del indexer.

**Tech Stack:** Rust (edition 2021+), rusqlite (SQLite bundled con FTS5), sqlite-vec (pineado exacto), fastembed-rs (jina-embeddings-v2-base-es, 768 dims), Python 3 solo para scripts de harness (patrón M0).

**Alcance**: este plan cubre SOLO la campaña 1 firmada (spec M2 §6): M2-01 + M2-02. Los planes de M2-03..09 se escriben cuando el gold de M2-02 esté sellado (orden spec-first adjudicado en D2) — no es un hueco del plan, es el diseño.

## Global Constraints

- Lenguaje Rust; binario se llama `exo`; crate único en `engine/` con `src/lib.rs` + `src/main.rs` (spec M2 §2, D1).
- **Veto AGPL**: prohibido abrir, leer o vendorizar código del repo de basic-memory. El diseño de la fusión viene descrito en la spec madre §4.2 y no se necesita en esta campaña. Leer `~/.basic-memory/config.json` (config del usuario) y su `memory.db` con probe read-only SÍ está permitido (spec M2 §2, D6).
- **sqlite-vec pineado con versión exacta `=x.y.z`** en `Cargo.toml` — jamás `^`. La versión resuelta se anota en la spec del indexer (spec M2 §7 riesgo 4).
- Config del engine: se lee de `~/.basic-memory/config.json` read-only (claves `semantic_embedding_model`, `semantic_embedding_dimensions`, `semantic_min_similarity`, `projects.kb-demo.path`); precedencia flags CLI > config. Sin config propia de exo en esta fase (D6).
- Corpus (reglas duras spec madre §6.2, van a la spec del indexer): permalinks del frontmatter se honran y JAMÁS se regeneran; dotdirs (`.claude/`, `.omc/`, `.superpowers/`) excluidos; `archive/` SE indexa; entidades no-markdown no se indexan; links a notas inexistentes se toleran; recencia = git.
- E1 no toca nada instalado: sin cutover, sin escritura en la KB, probes siempre `mode=ro`.
- Commits en castellano, prefijo convencional (`feat(engine):`, `docs(m2):` …), desde el working dir de exo (nunca `cd &&`).

---

### Task 1: M2-01 — Scaffold del crate `engine/` con smoke tests de las 3 deps nativas

**Files:**
- Create: `engine/Cargo.toml`, `engine/src/main.rs`, `engine/src/lib.rs`
- Create: `engine/tests/smoke.rs`
- Modify: `.gitignore` (añadir `engine/target/`)

**Interfaces:**
- Consumes: nada (primer código del engine).
- Produces: crate `exo` compilable; `exo_engine::abre_db_en_memoria() -> rusqlite::Connection` (helper de lib con sqlite-vec registrado, que M2-03 reutilizará); evidencia de que fastembed-rs sirve jina-es a 768 dims.

**Lane**: mecánica. **Oráculo**: `cargo build --release && cargo test` verdes + smoke de embedding (ignored) verde, outputs citados en el reporte del executor.

- [ ] **Step 1: Crear el crate**

```bash
mkdir -p engine && cd engine && cargo init --name exo
```

(Única excepción permitida al no-`cd`: cargo init necesita el cwd; el resto de comandos usan `cargo ... --manifest-path engine/Cargo.toml` o `-C`.)

Añadir a `.gitignore` del repo raíz la línea `engine/target/`.

- [ ] **Step 2: Declarar dependencias**

```bash
cargo add rusqlite --features bundled --manifest-path engine/Cargo.toml
cargo add fastembed --manifest-path engine/Cargo.toml
cargo add anyhow serde serde_json --manifest-path engine/Cargo.toml
cargo add clap --features derive --manifest-path engine/Cargo.toml
```

Para sqlite-vec: resolver la última versión publicada (`cargo search sqlite-vec` o crates.io) y añadirla PINEADA:

```bash
cargo add sqlite-vec@=X.Y.Z --manifest-path engine/Cargo.toml   # sustituir X.Y.Z por la resuelta
```

Verificar en `engine/Cargo.toml` que la línea quedó `sqlite-vec = "=X.Y.Z"` (con el `=`). Anotar la versión elegida en el mensaje de commit.

- [ ] **Step 3: Escribir los smoke tests (fallan: lib vacía)**

`engine/tests/smoke.rs`:

```rust
use exo::abre_db_en_memoria;

#[test]
fn fts5_disponible() {
    let db = abre_db_en_memoria().expect("db en memoria");
    db.execute_batch(
        "CREATE VIRTUAL TABLE t USING fts5(cuerpo);
         INSERT INTO t(cuerpo) VALUES ('el engine indexa la kb');",
    )
    .expect("FTS5 compilado en el bundle");
    let n: i64 = db
        .query_row("SELECT count(*) FROM t WHERE t MATCH 'engine'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 1);
}

#[test]
fn sqlite_vec_disponible() {
    let db = abre_db_en_memoria().expect("db en memoria");
    let version: String = db
        .query_row("SELECT vec_version()", [], |r| r.get(0))
        .expect("extensión sqlite-vec registrada");
    assert!(!version.is_empty());
    db.execute_batch("CREATE VIRTUAL TABLE v USING vec0(embedding float[768]);")
        .expect("tabla vec0 de 768 dims");
}

// Descarga el modelo (~0.6 GB) la primera vez: se corre explícito, no en CI de cada merge.
#[test]
#[ignore]
fn jina_es_embebe_a_768() {
    let (modelo, dims) = exo::embedder_desde_config().expect("fastembed con jina-es");
    let vecs = modelo; // ver Step 4: embedder_desde_config devuelve ya el embedding de prueba
    assert_eq!(dims, 768);
    assert_eq!(vecs.len(), 768);
}
```

- [ ] **Step 4: Implementación mínima de la lib**

`engine/src/lib.rs`:

```rust
use anyhow::{Context, Result};
use rusqlite::Connection;

/// Conexión en memoria con sqlite-vec registrado como auto-extension.
/// M2-03 reutiliza este mismo camino para la DB del índice.
pub fn abre_db_en_memoria() -> Result<Connection> {
    unsafe {
        // Registro estático de sqlite-vec (patrón documentado del crate sqlite-vec).
        rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
            sqlite_vec::sqlite3_vec_init as *const (),
        )));
    }
    Connection::open_in_memory().context("abrir sqlite en memoria")
}

/// Lee ~/.basic-memory/config.json (RO, D6), inicializa fastembed con el modelo
/// configurado y devuelve (embedding de la frase de prueba, dims declaradas).
pub fn embedder_desde_config() -> Result<(Vec<f32>, usize)> {
    let ruta = dirs::home_dir()
        .context("sin HOME")?
        .join(".basic-memory/config.json");
    let cfg: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&ruta).context("leer config bm")?)?;
    let modelo = cfg["semantic_embedding_model"]
        .as_str()
        .context("semantic_embedding_model ausente")?;
    let dims = cfg["semantic_embedding_dimensions"]
        .as_u64()
        .context("semantic_embedding_dimensions ausente")? as usize;
    let vector = embebe_frase(modelo, "el exocortex recuerda por ti")?;
    Ok((vector, dims))
}

fn embebe_frase(modelo: &str, frase: &str) -> Result<Vec<f32>> {
    use fastembed::{InitOptions, TextEmbedding};
    // NOTA para el executor: comprobar en docs.rs de la versión instalada de
    // fastembed si `jinaai/jina-embeddings-v2-base-es` existe como variante del
    // enum EmbeddingModel. Si SÍ: usar la variante. Si NO: usar el camino
    // UserDefinedEmbeddingModel de fastembed cargando el ONNX del repo HF
    // jinaai/jina-embeddings-v2-base-es (mismo modelo que sirve basic-memory).
    // Si ninguno de los dos caminos funciona, PARAR y escalar al orquestador:
    // es exactamente el riesgo que M2-01 existe para destapar.
    let mut te = TextEmbedding::try_new(
        InitOptions::new(/* variante o modelo custom según lo anterior */)
    )?;
    let mut out = te.embed(vec![frase.to_string()], None)?;
    Ok(out.pop().expect("un embedding"))
}
```

Añadir `cargo add dirs --manifest-path engine/Cargo.toml`. El bloque de `embebe_frase` se ajusta a la API real de la versión instalada (docs vía context7/docs.rs — no de memoria); lo invariante es el contrato: frase → `Vec<f32>` de 768.

`engine/src/main.rs`:

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "exo", version, about = "engine del framework exo (E1: read)")]
struct Cli {}

fn main() {
    let _cli = Cli::parse();
    // Subcomandos (index/search/recall/rebuild) entran con sus items (M2-03+).
}
```

- [ ] **Step 5: Verificar smokes**

```bash
cargo test --manifest-path engine/Cargo.toml
```
Expected: `fts5_disponible` y `sqlite_vec_disponible` PASS; `jina_es_embebe_a_768` ignored.

```bash
cargo test --manifest-path engine/Cargo.toml -- --ignored
```
Expected: `jina_es_embebe_a_768` PASS (primera corrida descarga el modelo; anotar duración de la descarga y del primer embed en el reporte — dato para el riesgo "modelo en frío", spec M2 §7.1).

```bash
cargo build --release --manifest-path engine/Cargo.toml && ./engine/target/release/exo --version
```
Expected: compila y emite versión.

- [ ] **Step 6: Commit**

```bash
git add engine/ .gitignore
git commit -m "feat(engine): scaffold del crate exo — FTS5, sqlite-vec =X.Y.Z pineado, fastembed jina-es 768 (M2-01)"
```

---

### Task 2: M2-02 — Spec del indexer + gold de paridad de corpus + envelope fijado (lane diseño)

**Files:**
- Create: `docs/superpowers/specs/2026-07-XX-indexer-design.md` (XX = fecha real de redacción)
- Create: `evals/e1-read/harness/corpus-parity.py`
- Create: `evals/e1-read/gold/corpus-bm.json` (sellado del lado basic-memory)
- Test: la corrida del probe ES el test (script idempotente, read-only)

**Interfaces:**
- Consumes: índice vivo de basic-memory (`~/.basic-memory/memory.db`, probe `mode=ro`); envelope de kbx (`~/Documentos/proyectos/kbx/internal/envelope/envelope.go`: `{schema_version:int, command:string, data:any}`, v1) como forma a adoptar.
- Produces: (a) spec del indexer que M2-03 implementa; (b) `corpus-parity.py --capture-bm` y `--diff <engine.db>` que M2-03/M2-09 usan como oráculo; (c) contrato de envelope y de `data` de search que M2-05 (`replay-engine.py`) consume: `{"schema_version":1,"command":"search","data":{"query":str,"search_type":str,"elapsed_s":float,"results":[{"permalink":str,"type":str,"score":float}]}}`.

**Lane**: diseño — fable en cabeza, review adversarial, y UN gate fable que cubre el par 02+03 (el de 02 al sellar spec+gold; D2). El gold se sella ANTES de que exista una línea del indexer.

- [ ] **Step 1: Redactar la spec del indexer**

`docs/superpowers/specs/2026-07-XX-indexer-design.md` con estas secciones obligatorias (contenido, no placeholder — lo redacta la cabeza fable de la campaña con la spec madre §6.2 delante):

1. **Contrato de corpus**: las 6 reglas duras de §6.2 copiadas literales + su verificación (cada regla mapeada a un check del probe o del indexer).
2. **Schema SQLite del engine**: tablas propias (nombres propios, NO calcados de basic-memory: p.ej. `notas(permalink PK, ruta, titulo, tipo, mtime, git_epoch)`, `notas_fts` (FTS5 sobre titulo+cuerpo), `aristas(origen, destino_permalink, destino_id NULL)`, `vectores` (vec0 float[768] + rowid→chunk)); chunking propio documentado (tamaño/solape y por qué).
3. **Incrementalidad**: mtime/git al invocar, sin daemon; `rebuild` = borrar DB + reconstruir (primera clase).
4. **Envelope**: adopción de la forma kbx v1 `{schema_version, command, data}` con `schema_version` propio de exo arrancando en 1; contrato de `data` para `search` (el de Interfaces, arriba) y regla de gating para consumidores (gate en exit code, no en campos informativos — doctrina del envelope de kbx).
5. **Config**: lectura RO de `~/.basic-memory/config.json`, precedencia flags > config (D6).
6. **Versión pineada de sqlite-vec** anotada (la que fijó M2-01).

- [ ] **Step 2: Escribir el probe de paridad de corpus**

`evals/e1-read/harness/corpus-parity.py`:

```python
#!/usr/bin/env python3
"""Oráculo de paridad de corpus (spec M2 §4-§5 pata 1).
--capture-bm: extrae del índice de basic-memory (RO) el set de permalinks a
nivel ENTIDAD (gotcha M0: jamás contra el output del CLI) y lo sella en gold/.
--diff ENGINE_DB: compara el índice del engine contra el gold sellado.
Exit 0 = diff vacío; exit 1 = divergencia (lista completa en stdout)."""
import argparse, json, sqlite3, sys
from pathlib import Path

BASE = Path(__file__).resolve().parent.parent
GOLD = BASE / "gold" / "corpus-bm.json"

def ro(path):
    return sqlite3.connect(f"file:{path}?mode=ro", uri=True)

def capture_bm():
    db = ro(Path.home() / ".basic-memory/memory.db")
    filas = db.execute(
        "SELECT permalink, file_path FROM entity WHERE permalink IS NOT NULL"
    ).fetchall()
    permalinks = sorted(p for p, _ in filas)
    dotdirs = [f for _, f in filas if f and f.split("/")[0] in (".claude", ".omc", ".superpowers")]
    datos = {
        "n_entidades": len(permalinks),
        "n_dotdirs_dentro": len(dotdirs),   # DEBE ser 0 (exclusión §6.2)
        "n_archive": sum(1 for p in permalinks if p.startswith("archive/") or "/archive/" in p),
        "permalinks": permalinks,
    }
    GOLD.parent.mkdir(parents=True, exist_ok=True)
    GOLD.write_text(json.dumps(datos, ensure_ascii=False, indent=1))
    print(f"sellado: {datos['n_entidades']} entidades, archive={datos['n_archive']}, dotdirs_dentro={datos['n_dotdirs_dentro']}")

def diff(engine_db):
    gold = set(json.loads(GOLD.read_text())["permalinks"])
    eng = {r[0] for r in ro(engine_db).execute("SELECT permalink FROM notas").fetchall()}
    faltan, sobran = sorted(gold - eng), sorted(eng - gold)
    for p in faltan: print(f"FALTA en engine: {p}")
    for p in sobran: print(f"SOBRA en engine: {p}")
    print(f"gold={len(gold)} engine={len(eng)} faltan={len(faltan)} sobran={len(sobran)}")
    sys.exit(0 if not faltan and not sobran else 1)

if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--capture-bm", action="store_true")
    g.add_argument("--diff", metavar="ENGINE_DB")
    a = ap.parse_args()
    capture_bm() if a.capture_bm else diff(a.diff)
```

Nota al executor: verificar el nombre real de la tabla/columnas de entidades del schema de basic-memory con `sqlite3 ~/.basic-memory/memory.db '.schema'` en RO antes de correr (si no es `entity(permalink, file_path)`, ajustar el SELECT y anotarlo en la spec del indexer). Esto es inspección del schema de una DB local del usuario, no del código AGPL.

- [ ] **Step 3: Sellar el gold**

```bash
python3 evals/e1-read/harness/corpus-parity.py --capture-bm
```
Expected: `sellado: ~117 entidades, archive≈32%, dotdirs_dentro=0`. Verificar contra los números de referencia (spec madre §6.2: 24 .md de dotdirs fuera, 5 entidades no-md sin permalink fuera por el `WHERE permalink IS NOT NULL`, archive dentro). Si `dotdirs_dentro != 0`, PARAR: o el probe está mal o el supuesto de exclusión era falso — escalar antes de sellar. Anotar en el gold-commit el commit HEAD de kb-demo en el momento de la captura.

- [ ] **Step 4: Review adversarial + gate fable del item**

Dispatch de consultor fable fresco (mecánica de fábrica, reserva D5) con: spec del indexer, gold sellado, checklist = {las 6 reglas §6.2 mapeadas a checks, envelope versionado con contrato de search completo, schema sin calcos de basic-memory, chunking justificado, versión sqlite-vec anotada}. El item NO se mergea sin veredicto favorable; hallazgos se corrigen y se re-somete.

- [ ] **Step 5: Commit (tras gate favorable)**

```bash
git add docs/superpowers/specs/2026-07-XX-indexer-design.md evals/e1-read/
git commit -m "docs(m2): spec del indexer + gold de paridad sellado + envelope exo v1 (M2-02, gate fable OK)"
```

---

## Self-review (hecho al escribir el plan)

1. **Cobertura de spec**: campaña 1 = M2-01 (Task 1) + M2-02 (Task 2) ✔; M2-03 arranca solo si el gate de Task 2 cierra en la misma noche (decisión del orquestador de campaña, D5) y su plan se escribe contra la spec del indexer ya sellada.
2. **Placeholders**: `X.Y.Z` (versión sqlite-vec) y `2026-07-XX` son deliberados — se resuelven en el momento exacto que el plan indica, no son huecos. El bloque `embebe_frase` declara contrato fijo e instruye verificación de API contra docs de la versión instalada (riesgo real: la disponibilidad de jina-es en fastembed-rs es EXACTAMENTE lo que M2-01 des-riesga).
3. **Consistencia de tipos**: `abre_db_en_memoria`/`embedder_desde_config` (Task 1 lib) son los que consume el smoke; el contrato de search de Task 2 §Interfaces es el que consumirá `replay-engine.py` en M2-05.
