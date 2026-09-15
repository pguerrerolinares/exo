# Campaña G — Engine: deuda diferida sin cambio de ranking — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: usa `exo:orchestrate` para
> ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`) para
> tracking. Gates adjudicados por un consultor Fable delegado (régimen
> firmado, `.superpowers/fabrica/config.md` §«Ejecución de gates»).

**Goal:** cerrar 9 items de deuda diferida del engine (`engine/src`) que
llevan meses en «cuando se toque por otra razón», con un único oráculo común
para todos — la salida de `search`/`recall` es byte-idéntica antes y después
sobre la KB de test, salvo dos cambios de comportamiento **declarados**
(unificación de `walk_kb`, Task 4; default de `exo search --type`, Task 11) —
sin tocar β, bonus, umbral ni fusión (eso es la campaña J, después).

**Architecture:** doce tareas. Task 1 arregla el bench sintético (vectores
correlacionados con contenido real en vez de ruido puro) porque es el
oráculo de coste que las Tasks 2-3 necesitan para medir de verdad. Tasks 2-3
atacan las dos patas vivas de «Techos de escala» (tres conexiones de DB por
`busca_hybrid`, un `git log` por nota). Tasks 4-9 son ítems de módulos
disjuntos (`walker`, `lint`, `escritor`+`main.rs`, `lib.rs`, `trinquete`) sin
dependencias cruzadas entre sí. Task 10 es un rename mecánico
(`kb-demo`→`kb-test`) transversal a los tests. Task 11 es el cambio de
superficie de D6 (decisión ya tomada por Paul), ampliado el mismo día para
que `exo init` escriba su propio default de `[embeddings] min_similarity`
desde la misma constante sellada, en vez de un segundo literal. Task 12
sincroniza
`docs/backlog.md`, cierra los ítems que la campaña resuelve y corrige tres
afirmaciones caducadas del propio backlog con evidencia de código.

**Tech Stack:** Rust 2024 (crate `exo` en `engine/`, MSRV 1.95) · `clap`
4.6.2 derive · `rusqlite` + `sqlite-vec` (vec0) · `fastembed` 5.17.3 (jina-es
768d) · `tempfile` 3.14 (dev) · bash (Git Bash en Windows) para
`evals/recall-coste/harness/*.sh` · GitHub Actions (`.github/workflows/ci.yml`,
jobs `lint`/`test`/`msrv`).

## Global Constraints

Todas verificadas hoy contra `origin/main` (worktree `campana-g`).

- **El crate vive en `engine/`, no en la raíz.** No hay workspace. Todo
  `cargo` con cwd `engine/`.
- **MSRV `rust-version = "1.95"`** (`engine/Cargo.toml:8`), edition 2024
  (let-chains e `if let ... && ...` ya son idioma de la casa, ver
  `doctor.rs`).
- **Lint (`ci.yml` job `lint`, verbatim):** `cargo fmt --check` y
  `cargo clippy --all-targets --locked -- -D warnings`, ambos con cwd
  `engine/`. `--all-targets` incluye `examples/kb_sintetica.rs` — un
  helper que deja de usarse ahí (p.ej. `vector_unitario` tras la Task 1)
  se BORRA, no se deja con `#[allow(dead_code)]`.
- **Tests:** `cd engine && cargo test --release --locked` (pesado, en
  background). Nunca `npm`/`tsc`, esto es Rust puro.
- **Envelope v2 sin cambios.** Ninguna task de este plan toca el schema de
  `data` de `search`/`recall`/`write`/`index` ni sube `SCHEMA_VERSION`.
- **Exit codes sin cambios** (`main.rs:387-412`): `3` `Rechazo`/`GateFallido`,
  `1` otro error, `0` éxito, `2` uso de clap. Las Tasks 6-7 SUMAN un caso de
  error nuevo (permalink de <3 segmentos), pero es el mismo exit `1` que ya
  usa `anyhow::bail!` en el resto del write-path — no un código nuevo.
- **Invariante de la campaña, sin cambio de ranking:** `exo search --type
  hybrid --json` y `exo recall --query ... --json` dan salida byte-idéntica
  sobre la KB semilla antes/después de cada task, **salvo** Task 4 (declara
  qué entra/deja de recorrer `walk_kb`) y Task 11 (declara el nuevo default
  de superficie, no de ranking). Ninguna task toca β, bonus, umbral de
  similitud ni el algoritmo de fusión (`normaliza_fts`/`fusiona`) — eso es
  la campaña J, después de esta.
- **No se consume held-out.** El de la campaña C está gastado
  (`evals/retrieval-heldout/verdict/c-verdict.md` §11). Ninguna task de
  este plan corre el harness de `evals/retrieval-heldout/`.
- **`main.rs` es zona compartida con H (retiro de aliases españoles,
  `:153-315`, y `--version`) en la misma ola.** G toca tres zonas disjuntas:
  `:11-26` y `:241` (Task 11, comentario/constantes sellados y default de
  `--type`), `:651-657` (Task 11, ampliado el 2026-09-15: default de
  `[embeddings] min_similarity` en `init_cmd`) y `:794-868` (Tasks 6-7,
  `write_new_cmd`/`write_append_cmd`). **Ninguna task de G usa ni añade un
  alias en español** — los tests nuevos de G usan siempre los nombres
  canónicos (`--min-similarity`, `--limit`, etc.), nunca `--min-similitud`
  ni `--limite`. **Orden de merge de la ola: H → F → G** — antes de
  arrancar las Tasks 6, 7 y 11 (las que tocan `main.rs`), rebasa
  `campana-g` sobre `main` ya con H y F mergeadas, y re-ancla cualquier
  `old_string` de `Edit` cuyo número de línea absoluto se haya movido (el
  *contenido* citado en este plan no cambia con el rebase salvo que H toque
  la MISMA línea, y H declara sus zonas como `:153-315`/`--version`,
  disjuntas de las de G).
- **kbx ya no es referencia** (cerrado en la campaña D, `backlog:181`).
  Task 9 no depende de kbx ni de la máquina Linux para implementarse — la
  verificación cruzada contra `internal/ratchet` de kbx es un checklist
  **opcional** de Paul (ver Task 9), nunca bloqueante.
- **Git:** `git add <rutas explícitas>`, nunca `-A`/`.`; `git -C <path>`,
  nunca `cd <path> && git`. Nada de push ni de tocar GitHub.
- **Commits:** `<tipo>(g, <slug>): <mensaje>` — `refactor`, `perf`, `fix`,
  `feat`, `test`, `docs`, `chore` según corresponda (convención de D/E).

---

### Task 1: `kb_sintetica` deja de ser ciega al umbral — vectores reales + ruido

**Lane:** mecánica. **Depende de:** nada (independiente; se hace primero
porque es el oráculo de coste que las Tasks 2-3 usan para medir sin
mentirse — con vectores puramente aleatorios en 768 dims, la similitud
coseno esperada con cualquier query es ~0, así que con el umbral de
producción (0.40) el arm vector nunca aportaba nada al bench: `s2-query`
media solo el canal FTS aunque diga `hybrid`).

**Files:**
- Modify: `engine/examples/kb_sintetica.rs`
- Modify: `evals/recall-coste/harness/bench.sh`

**Interfaces:**
- Consumes: `exo::con_embedder_de_proceso` (`engine/src/lib.rs:293`, firma
  `pub fn con_embedder_de_proceso<T>(f: impl FnOnce(&mut Embedder) -> Result<T>) -> Result<T>`),
  `exo::vectores::inserta` (ya usado), `exo::trozos::trocea` (ya usado).
- Produces: nada que otra task consuma directamente — Tasks 2 y 3 solo leen
  el `resumen.tsv` que `bench.sh`/`compara.sh` ya producían.

- [ ] **Step 1: Ver el bench ciego al umbral (estado ANTES, para tener el
  contraste)**

Run (Linux o Git Bash con `hyperfine`/`jq` instalados; si no están
disponibles en esta máquina, anota el resultado esperado y pasa al Step 2 —
no es bloqueante, es evidencia del estado previo):

```bash
cd engine && cargo build --release --locked --bin exo --example kb_sintetica
./target/release/examples/kb_sintetica 174 /tmp/kbg-antes 42
EXO_CONFIG=/tmp/kbg-antes/config.toml ./target/release/exo recall \
  --db /tmp/kbg-antes/index.db --kb /tmp/kbg-antes/kb \
  --query 'trinquete techos indice memoria' --min-similarity 0.40 \
  --limit 4 --cap-bytes 4000 --json | jq '.data.notes | length'
```

Expected: `0` (el arm vector no aporta nada; con vectores puramente
aleatorios en 768d, P(similitud coseno ≥ 0.40) ≈ 0 — la varianza de la
similitud entre dos vectores unitarios aleatorios en alta dimensión es
`1/√768 ≈ 0.036`, muy por debajo del umbral).

- [ ] **Step 2: Reescribir `kb_sintetica.rs` — pool de embeddings reales +
  ruido gaussiano**

Reemplaza el doc-comment del módulo (líneas 1-9):

old_string:
```rust
//! KB sintética para el bench de coste de la campaña A.
//!
//! Pre-registro: `docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md`.
//! La FORMA copia la de la KB real medida el 2026-09-13 (174 notas, 3.290
//! trozos de 617 caracteres de media, 727 aristas con 24 sin resolver, tiers
//! 6 core / 60 stable / 105 log). El índice se construye SIN el modelo: los
//! vectores son pseudoaleatorios y deterministas. Mide coste, no calidad.
//!
//! Uso: kb_sintetica <N> <DIR> [SEMILLA]
```

new_string:
```rust
//! KB sintética para el bench de coste de la campaña A (y el oráculo de
//! coste de la Ola 1 G, Task 1: el arm vector necesita contenido correlado
//! para no ser ciego al umbral, backlog:1046-1057).
//!
//! Pre-registro: `docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md`.
//! La FORMA copia la de la KB real medida el 2026-09-13 (174 notas, 3.290
//! trozos de 617 caracteres de media, 727 aristas con 24 sin resolver, tiers
//! 6 core / 60 stable / 105 log). El TEXTO sigue siendo sintético (24
//! palabras de vocabulario sin significado real) — mide coste, no calidad
//! de retrieval — pero los VECTORES ya no son pseudoaleatorios: cada trozo
//! usa el embedding REAL de su palabra de vocabulario dominante (pool de 24
//! embeddings, calculado UNA VEZ con el modelo real, nunca por trozo) más
//! ruido gaussiano `KB_SINTETICA_SIGMA` (default 0.5), renormalizado a
//! norma unidad. Antes, un vector puramente aleatorio en 768 dims tiene
//! similitud coseno esperada ~0 con cualquier query, así que con el umbral
//! de producción (0.40) el arm vector nunca aportaba nada al bench.
//!
//! Uso: kb_sintetica <N> <DIR> [SEMILLA]  (env `KB_SINTETICA_SIGMA` opcional)
```

Añade el import (junto a los `use` existentes):

old_string:
```rust
use anyhow::{Context, Result, bail};
use rusqlite::params;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;
```

new_string:
```rust
use anyhow::{Context, Result, bail};
use exo::con_embedder_de_proceso;
use rusqlite::params;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;
```

Borra `vector_unitario` entero (ya no lo usa nadie tras este Step — clippy
`-D warnings` marcaría dead-code si se quedara con `#[allow]`):

old_string:
```rust
fn vector_unitario(rng: &mut Xorshift) -> Vec<f32> {
    let mut v: Vec<f32> = (0..768).map(|_| rng.unidad()).collect();
    let norma = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    for x in &mut v {
        *x /= norma;
    }
    v
}

/// Construye el índice como lo dejaría `exo index`, salvo los vectores.
/// Devuelve (trozos, aristas, aristas sin resolver).
fn construye_indice(kb: &Path, db: &Path, n: usize, rng: &mut Xorshift) -> Result<(i64, i64, i64)> {
```

new_string:
```rust
/// Palabra del vocabulario más frecuente en `texto` — decide qué vector del
/// pool real usa este trozo. La query del bench («trinquete techos indice
/// memoria») también son solo palabras del vocabulario, así que compartir
/// este criterio es lo que hace que query y contenido caigan en la MISMA
/// región del espacio de embeddings real.
fn palabra_dominante(texto: &str) -> usize {
    let mut cuentas = [0usize; VOCABULARIO.len()];
    for palabra in texto.split_whitespace() {
        if let Some(i) = VOCABULARIO.iter().position(|v| *v == palabra) {
            cuentas[i] += 1;
        }
    }
    cuentas
        .iter()
        .enumerate()
        .max_by_key(|(_, c)| **c)
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Embeddings REALES de las 24 palabras del vocabulario — una sola pasada
/// de batch (24 textos, no miles). Se calcula UNA VEZ por generación, nunca
/// por trozo: la correlación de contenido que faltaba (backlog:1046-1057)
/// sin pagar el coste de embeber la KB sintética entera.
fn pool_de_vocabulario() -> Result<Vec<Vec<f32>>> {
    let textos: Vec<String> = VOCABULARIO.iter().map(|w| w.to_string()).collect();
    con_embedder_de_proceso(|embedder| embedder.embebe_batch(&textos))
        .context("embed del pool de vocabulario para kb_sintetica")
}

/// Vector "real + ruido": embedding real de la palabra dominante del trozo
/// más ruido gaussiano N(0, sigma²) por componente, renormalizado a norma
/// unidad. `sigma=0` es el embedding real sin ruido; `sigma` alto lo acerca
/// al `vector_unitario()` puramente aleatorio de antes de esta task.
fn vector_desde_pool(pool: &[Vec<f32>], idx: usize, rng: &mut Xorshift, sigma: f32) -> Vec<f32> {
    let base = &pool[idx];
    let mut v: Vec<f32> = base.iter().map(|x| x + sigma * rng.normal()).collect();
    let norma = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norma > 1e-9 {
        for x in &mut v {
            *x /= norma;
        }
    }
    v
}

/// Construye el índice como lo dejaría `exo index`, salvo el TEXTO (sigue
/// siendo sintético). Devuelve (trozos, aristas, aristas sin resolver).
fn construye_indice(
    kb: &Path,
    db: &Path,
    n: usize,
    rng: &mut Xorshift,
    pool: &[Vec<f32>],
    sigma: f32,
) -> Result<(i64, i64, i64)> {
```

Añade el método `normal()` al `impl Xorshift` (justo después de `unidad`):

old_string:
```rust
    /// Uniforme en [-1, 1).
    fn unidad(&mut self) -> f32 {
        (self.siguiente() >> 40) as f32 / (1u64 << 23) as f32 - 1.0
    }
}
```

new_string:
```rust
    /// Uniforme en [-1, 1).
    fn unidad(&mut self) -> f32 {
        (self.siguiente() >> 40) as f32 / (1u64 << 23) as f32 - 1.0
    }

    /// Normal estándar vía Box-Muller (forma polar simple), determinista con
    /// la misma semilla. `.max(1e-6)` evita `ln(0)` en el caso borde
    /// (probabilidad ~1/2^24 por muestra) sin añadir un segundo generador.
    fn normal(&mut self) -> f32 {
        let u1 = ((self.unidad() + 1.0) / 2.0).max(1e-6);
        let u2 = (self.unidad() + 1.0) / 2.0;
        (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos()
    }
}
```

Cambia la inserción del vector dentro de `construye_indice` (el bucle de
trozos):

old_string:
```rust
        for (orden, texto) in exo::trozos::trocea(&nota.cuerpo).iter().enumerate() {
            tx.execute(
                "INSERT INTO trozos (permalink, orden, texto) VALUES (?1, ?2, ?3)",
                params![nota.permalink, orden as i64, texto],
            )?;
            let id = tx.last_insert_rowid();
            exo::vectores::inserta(&tx, id, &vector_unitario(rng))?;
        }
```

new_string:
```rust
        for (orden, texto) in exo::trozos::trocea(&nota.cuerpo).iter().enumerate() {
            tx.execute(
                "INSERT INTO trozos (permalink, orden, texto) VALUES (?1, ?2, ?3)",
                params![nota.permalink, orden as i64, texto],
            )?;
            let id = tx.last_insert_rowid();
            let idx = palabra_dominante(texto);
            exo::vectores::inserta(&tx, id, &vector_desde_pool(pool, idx, rng, sigma))?;
        }
```

Y en `main()`, lee `sigma` de env y calcula el pool antes de construir el
índice:

old_string:
```rust
    let (trozos, aristas, rotas) = construye_indice(&kb, &db, n, &mut rng)?;
    println!(
        "kb_sintetica: N={n} semilla={semilla} trozos={trozos} aristas={aristas} sin_resolver={rotas}"
    );
    Ok(())
```

new_string:
```rust
    let sigma: f32 = std::env::var("KB_SINTETICA_SIGMA")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.5);
    let pool = pool_de_vocabulario().context("pool de embeddings reales del vocabulario")?;
    let (trozos, aristas, rotas) = construye_indice(&kb, &db, n, &mut rng, &pool, sigma)?;
    println!(
        "kb_sintetica: N={n} semilla={semilla} sigma={sigma} trozos={trozos} aristas={aristas} sin_resolver={rotas}"
    );
    Ok(())
```

- [ ] **Step 2: Compilar y correr el escenario del Step 1 otra vez**

Run:
```bash
cd engine && cargo build --release --locked --bin exo --example kb_sintetica
rm -rf /tmp/kbg-despues
./target/release/examples/kb_sintetica 174 /tmp/kbg-despues 42
EXO_CONFIG=/tmp/kbg-despues/config.toml ./target/release/exo recall \
  --db /tmp/kbg-despues/index.db --kb /tmp/kbg-despues/kb \
  --query 'trinquete techos indice memoria' --min-similarity 0.40 \
  --limit 4 --cap-bytes 4000 --json | jq '.data.notes | length'
```

Expected: un entero `> 0` (el arm vector ahora encuentra al menos una nota;
la query comparte palabras de vocabulario con el pool real, así que su
embedding cae en la misma región que los trozos que las usan). Si sale `0`,
sube `KB_SINTETICA_SIGMA` (p.ej. a 0.3) y repite — el valor exacto de sigma
NO es el oráculo, el gate es "`>0` con el umbral de producción 0.40".

- [ ] **Step 3: Extender el fidelity-check de `bench.sh` con una verificación
  de cobertura del arm vector**

Añade, justo después del bloque de fidelidad existente (que comprueba que el
índice sintético está fresco):

old_string:
```bash
  "$BIN" index --kb "$KB" --db "$DB" --json > "$OUT/fidelidad-n$N.json" || exit 1
  jq -e --argjson n "$N" '.data.indexed == 0 and .data.skipped == $n' "$OUT/fidelidad-n$N.json" >/dev/null || {
    echo "bench: índice sintético no fresco para N=$N: $(cat "$OUT/fidelidad-n$N.json")" >&2; exit 1; }

  jq -n --arg p "$Q" '{prompt:$p, session_id:"bench-a"}' > "$D/prompt.json"
```

new_string:
```bash
  "$BIN" index --kb "$KB" --db "$DB" --json > "$OUT/fidelidad-n$N.json" || exit 1
  jq -e --argjson n "$N" '.data.indexed == 0 and .data.skipped == $n' "$OUT/fidelidad-n$N.json" >/dev/null || {
    echo "bench: índice sintético no fresco para N=$N: $(cat "$OUT/fidelidad-n$N.json")" >&2; exit 1; }

  # Ola 1 G Task 1: el arm vector tiene que aportar algo con el umbral de
  # producción, o el bench mide solo el canal FTS aunque diga "hybrid".
  "$BIN" recall --db "$DB" --kb "$KB" "--query=$Q" --min-similarity 0.40 \
    --limit 4 --cap-bytes 4000 --json > "$OUT/cobertura-vector-n$N.json" || exit 1
  jq -e '.data.notes | length > 0' "$OUT/cobertura-vector-n$N.json" >/dev/null || {
    echo "bench: arm vector sin resultados (umbral 0.40) para N=$N — kb_sintetica sigue ciega al umbral" >&2
    exit 1; }

  jq -n --arg p "$Q" '{prompt:$p, session_id:"bench-a"}' > "$D/prompt.json"
```

- [ ] **Step 4: Commit**

```bash
git add engine/examples/kb_sintetica.rs evals/recall-coste/harness/bench.sh
git commit -m "perf(g, kb-sintetica): vectores = pool real de vocabulario + ruido gaussiano, ya no ciegos al umbral"
```

---

### Task 2: `busca_hybrid` con una sola conexión de DB

**Lane:** mecánica. **Depende de:** nada. **Bloquea:** ninguna otra task de
este plan (J la tocará después, en su propia campaña).

**Files:**
- Modify: `engine/src/buscador.rs`

**Interfaces:**
- Consumes: `crate::abre_db` (ya usado), `crate::indexer::aviso_kb_root_lectura`
  (firma sin cambios: `pub fn aviso_kb_root_lectura(conn: &Connection, kb: Option<&Path>) -> Option<String>`).
- Produces: `pub fn busca(db_ruta: &Path, query: &str, limite: usize, kb: Option<&Path>) -> Result<Busqueda>`,
  `pub fn busca_vector(db_ruta: &Path, query: &str, limite: usize, min_similitud: Option<f64>, kb: Option<&Path>) -> Result<Busqueda>`,
  `pub fn busca_hybrid(db_ruta: &Path, query: &str, limite: usize, min_similitud: Option<f64>, bonus: f64, escala_fts: f64, kb: Option<&Path>) -> Result<Busqueda>`
  — las tres firmas públicas quedan **exactamente iguales**; solo cambia su
  cuerpo interno. Ninguna otra task de este plan llama a estas funciones.

- [ ] **Step 1: Test que falla — `busca_hybrid` abre la DB una sola vez**

Añade al final de `engine/src/buscador.rs` (fuera de los `mod tests_*`
existentes, en un módulo nuevo):

```rust
#[cfg(test)]
mod tests_una_conexion {
    /// Grep sobre el propio código fuente: `busca_hybrid` debe abrir la DB
    /// UNA sola vez (backlog:524-566, "tres aperturas de la DB por
    /// búsqueda hybrid"). No es un test de comportamiento — el
    /// comportamiento ya lo cubren `tests_fusion` y los tests de
    /// `tests/buscador_cli.rs` — es un gate de que el refactor no vuelve a
    /// crecer un segundo `abre_db` dentro de la función.
    #[test]
    fn busca_hybrid_abre_una_sola_conexion() {
        let fuente = include_str!("buscador.rs");
        let inicio = fuente
            .find("pub fn busca_hybrid(")
            .expect("busca_hybrid debe existir en buscador.rs");
        let cuerpo = &fuente[inicio..];
        let fin_cuerpo = cuerpo
            .find("\n}\n")
            .expect("busca_hybrid debe cerrar con '\\n}\\n'");
        let cuerpo = &cuerpo[..fin_cuerpo];
        assert_eq!(
            cuerpo.matches("abre_db(").count(),
            1,
            "busca_hybrid debe abrir la DB una sola vez:\n{cuerpo}"
        );
    }
}
```

- [ ] **Step 2: Correr el test y verlo fallar**

Run: `cd engine && cargo test --release --locked busca_hybrid_abre_una_sola_conexion -- --nocapture`
Expected: FAIL — `assertion 'left == right' failed ... left: 3, right: 1`
(hoy `busca_hybrid` abre la DB en `busca()`, en `busca_vector()` y en su
propio `abre_db(db_ruta)?` para `enriquece_rutas`).

- [ ] **Step 3: Refactor — variantes `_con(&Connection)`**

Reemplaza `busca` y su cuerpo:

old_string:
```rust
pub fn busca(db_ruta: &Path, query: &str, limite: usize, kb: Option<&Path>) -> Result<Busqueda> {
    if !db_ruta.exists() {
        anyhow::bail!("DB no encontrada: {}", db_ruta.display());
    }

    let inicio = Instant::now();
    let conn = abre_db(db_ruta)?;
    // Sobre la conexión que YA está abierta para esta consulta — nunca una
    // propia (ver `indexer::aviso_kb_root_lectura`). `kb: None` (comando sin
    // KB resoluble) o cualquier fallo interno degradan a `None` solos.
    let aviso_kb_root = crate::indexer::aviso_kb_root_lectura(&conn, kb);
    let fts_query = prepara_query(query);

    // Query vacía tras normalizar (p.ej. solo whitespace): éxito con
    // resultados vacíos, sin invocar MATCH con cadena vacía (que revienta
    // FTS5 con "syntax error").
    let results = if fts_query.is_empty() {
        Vec::new()
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT permalink, -bm25(notas_fts) AS score
                 FROM notas_fts
                 WHERE notas_fts MATCH ?1
                 ORDER BY score DESC
                 LIMIT ?2",
            )
            .context("preparar consulta FTS5")?;

        stmt.query_map(params![fts_query, limite as i64], |r| {
            Ok(Resultado {
                permalink: r.get(0)?,
                tipo: "entity".to_string(),
                score: r.get(1)?,
                ruta: None,
            })
        })
        .with_context(|| format!("ejecutar MATCH FTS5 para query preparada: {fts_query}"))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("leer resultados FTS5")?
    };

    let mut results = results;
    enriquece_rutas(&conn, &mut results)?;

    Ok(Busqueda {
        query: query.to_string(),
        search_type: "fts".to_string(),
        elapsed_s: inicio.elapsed().as_secs_f64(),
        results,
        avisos: Vec::new(),
        aviso_kb_root,
    })
}
```

new_string:
```rust
pub fn busca(db_ruta: &Path, query: &str, limite: usize, kb: Option<&Path>) -> Result<Busqueda> {
    if !db_ruta.exists() {
        anyhow::bail!("DB no encontrada: {}", db_ruta.display());
    }
    let conn = abre_db(db_ruta)?;
    busca_con(&conn, query, limite, kb)
}

/// Cuerpo de `busca` sobre una conexión YA ABIERTA (Ola 1 G Task 2,
/// backlog:524-566: antes `busca_hybrid` pagaba tres `abre_db` — una por
/// `busca()`, una por `busca_vector()`, una propia para `enriquece_rutas` —
/// esta variante deja que el llamador decida la conexión). Mismo cuerpo que
/// antes tenía `busca`, palabra por palabra.
fn busca_con(conn: &rusqlite::Connection, query: &str, limite: usize, kb: Option<&Path>) -> Result<Busqueda> {
    let inicio = Instant::now();
    // Sobre la conexión que YA está abierta para esta consulta — nunca una
    // propia (ver `indexer::aviso_kb_root_lectura`). `kb: None` (comando sin
    // KB resoluble) o cualquier fallo interno degradan a `None` solos.
    let aviso_kb_root = crate::indexer::aviso_kb_root_lectura(conn, kb);
    let fts_query = prepara_query(query);

    // Query vacía tras normalizar (p.ej. solo whitespace): éxito con
    // resultados vacíos, sin invocar MATCH con cadena vacía (que revienta
    // FTS5 con "syntax error").
    let results = if fts_query.is_empty() {
        Vec::new()
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT permalink, -bm25(notas_fts) AS score
                 FROM notas_fts
                 WHERE notas_fts MATCH ?1
                 ORDER BY score DESC
                 LIMIT ?2",
            )
            .context("preparar consulta FTS5")?;

        stmt.query_map(params![fts_query, limite as i64], |r| {
            Ok(Resultado {
                permalink: r.get(0)?,
                tipo: "entity".to_string(),
                score: r.get(1)?,
                ruta: None,
            })
        })
        .with_context(|| format!("ejecutar MATCH FTS5 para query preparada: {fts_query}"))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("leer resultados FTS5")?
    };

    let mut results = results;
    enriquece_rutas(conn, &mut results)?;

    Ok(Busqueda {
        query: query.to_string(),
        search_type: "fts".to_string(),
        elapsed_s: inicio.elapsed().as_secs_f64(),
        results,
        avisos: Vec::new(),
        aviso_kb_root,
    })
}
```

Reemplaza `busca_vector` y su cuerpo:

old_string:
```rust
pub fn busca_vector(
    db_ruta: &Path,
    query: &str,
    limite: usize,
    min_similitud: Option<f64>,
    kb: Option<&Path>,
) -> Result<Busqueda> {
    if !db_ruta.exists() {
        anyhow::bail!("DB no encontrada: {}", db_ruta.display());
    }

    let inicio = Instant::now();
    let conn = abre_db(db_ruta)?;
    // Mismo aviso best-effort que `busca`, sobre esta misma conexión.
    let aviso_kb_root = crate::indexer::aviso_kb_root_lectura(&conn, kb);

    let total_vectores: i64 = conn
        .query_row("SELECT count(*) FROM vectores", [], |r| r.get(0))
        .context("contar filas de vectores")?;

    let results = if total_vectores == 0 || query.trim().is_empty() {
        Vec::new()
    } else {
        let umbral = min_similitud_efectivo(min_similitud)?;

        let mut embeddings =
            con_embedder_de_proceso(|embedder| embedder.embebe_batch(&[query.to_string()]))
                .context("embed de la query")?;
        let embedding = embeddings.pop().expect("un embedding de la query");

        busca_vector_con_embedding(&conn, &embedding, limite, umbral, total_vectores as usize)
            .context("KNN acotado por consulta")?
    };

    let mut results = results;
    enriquece_rutas(&conn, &mut results)?;
    let avisos = avisos_cobertura_vector(&conn)?;

    Ok(Busqueda {
        query: query.to_string(),
        search_type: "vector".to_string(),
        elapsed_s: inicio.elapsed().as_secs_f64(),
        results,
        avisos,
        aviso_kb_root,
    })
}
```

new_string:
```rust
pub fn busca_vector(
    db_ruta: &Path,
    query: &str,
    limite: usize,
    min_similitud: Option<f64>,
    kb: Option<&Path>,
) -> Result<Busqueda> {
    if !db_ruta.exists() {
        anyhow::bail!("DB no encontrada: {}", db_ruta.display());
    }
    let conn = abre_db(db_ruta)?;
    busca_vector_con(&conn, query, limite, min_similitud, kb)
}

/// Cuerpo de `busca_vector` sobre una conexión YA ABIERTA (Ola 1 G Task 2,
/// mismo motivo que `busca_con`). Mismo cuerpo que antes tenía
/// `busca_vector`, palabra por palabra.
fn busca_vector_con(
    conn: &rusqlite::Connection,
    query: &str,
    limite: usize,
    min_similitud: Option<f64>,
    kb: Option<&Path>,
) -> Result<Busqueda> {
    let inicio = Instant::now();
    // Mismo aviso best-effort que `busca`, sobre esta misma conexión.
    let aviso_kb_root = crate::indexer::aviso_kb_root_lectura(conn, kb);

    let total_vectores: i64 = conn
        .query_row("SELECT count(*) FROM vectores", [], |r| r.get(0))
        .context("contar filas de vectores")?;

    let results = if total_vectores == 0 || query.trim().is_empty() {
        Vec::new()
    } else {
        let umbral = min_similitud_efectivo(min_similitud)?;

        let mut embeddings =
            con_embedder_de_proceso(|embedder| embedder.embebe_batch(&[query.to_string()]))
                .context("embed de la query")?;
        let embedding = embeddings.pop().expect("un embedding de la query");

        busca_vector_con_embedding(conn, &embedding, limite, umbral, total_vectores as usize)
            .context("KNN acotado por consulta")?
    };

    let mut results = results;
    enriquece_rutas(conn, &mut results)?;
    let avisos = avisos_cobertura_vector(conn)?;

    Ok(Busqueda {
        query: query.to_string(),
        search_type: "vector".to_string(),
        elapsed_s: inicio.elapsed().as_secs_f64(),
        results,
        avisos,
        aviso_kb_root,
    })
}
```

Reemplaza el cuerpo de `busca_hybrid`:

old_string:
```rust
pub fn busca_hybrid(
    db_ruta: &Path,
    query: &str,
    limite: usize,
    min_similitud: Option<f64>,
    bonus: f64,
    escala_fts: f64,
    kb: Option<&Path>,
) -> Result<Busqueda> {
    if !db_ruta.exists() {
        anyhow::bail!("DB no encontrada: {}", db_ruta.display());
    }

    let inicio = Instant::now();

    const K_C: usize = 50;
    let fts = busca(db_ruta, query, K_C, kb)?;
    let aviso_fts = fts.aviso_kb_root;
    let candidatos_fts: Vec<(String, f64)> = fts
        .results
        .into_iter()
        .map(|r| (r.permalink, r.score))
        .collect();
    let f_por_entidad = normaliza_fts(&candidatos_fts, escala_fts);

    // Guarda explícita (H29, ver doc de la función): el atajo top-`limite`
    // solo es exacto con `bonus == 0.0`. Cualquier `bonus` distinto — hoy
    // solo alcanzable con `--bonus` explícito, `BONUS_SELLADO` es 0.0 —
    // vuelve al arm vector exhaustivo de siempre.
    let limite_vector = if bonus == 0.0 { limite } else { usize::MAX };
    let vector = busca_vector(db_ruta, query, limite_vector, min_similitud, kb)?;
    let avisos = vector.avisos;
    // El aviso de kb_root sale de la MISMA DB por los dos arms (`fts` y
    // `vector` abren conexiones distintas, pero contra el mismo fichero):
    // cualquiera de los dos vale, `or` evita duplicar el texto en el
    // resultado final.
    let aviso_kb_root = aviso_fts.or(vector.aviso_kb_root);
    let v_por_entidad: HashMap<String, f64> = vector
        .results
        .into_iter()
        .map(|r| (r.permalink, r.score))
        .collect();

    let mut results = fusiona(&v_por_entidad, &f_por_entidad, bonus, limite);
    // Conexión propia: los dos arms de arriba ya cerraron las suyas, y aquí
    // solo quedan `limite` filas que resolver.
    enriquece_rutas(&abre_db(db_ruta)?, &mut results)?;

    Ok(Busqueda {
        query: query.to_string(),
        search_type: "hybrid".to_string(),
        elapsed_s: inicio.elapsed().as_secs_f64(),
        results,
        avisos,
        aviso_kb_root,
    })
}
```

new_string:
```rust
pub fn busca_hybrid(
    db_ruta: &Path,
    query: &str,
    limite: usize,
    min_similitud: Option<f64>,
    bonus: f64,
    escala_fts: f64,
    kb: Option<&Path>,
) -> Result<Busqueda> {
    if !db_ruta.exists() {
        anyhow::bail!("DB no encontrada: {}", db_ruta.display());
    }

    let inicio = Instant::now();
    // Ola 1 G Task 2 (backlog:524-566): UNA conexión para los dos arms y el
    // enriquecido de rutas — antes eran tres `abre_db` distintos.
    let conn = abre_db(db_ruta)?;

    const K_C: usize = 50;
    let fts = busca_con(&conn, query, K_C, kb)?;
    let aviso_fts = fts.aviso_kb_root;
    let candidatos_fts: Vec<(String, f64)> = fts
        .results
        .into_iter()
        .map(|r| (r.permalink, r.score))
        .collect();
    let f_por_entidad = normaliza_fts(&candidatos_fts, escala_fts);

    // Guarda explícita (H29, ver doc de la función): el atajo top-`limite`
    // solo es exacto con `bonus == 0.0`. Cualquier `bonus` distinto — hoy
    // solo alcanzable con `--bonus` explícito, `BONUS_SELLADO` es 0.0 —
    // vuelve al arm vector exhaustivo de siempre.
    let limite_vector = if bonus == 0.0 { limite } else { usize::MAX };
    let vector = busca_vector_con(&conn, query, limite_vector, min_similitud, kb)?;
    let avisos = vector.avisos;
    // El aviso de kb_root sale de la MISMA conexión por los dos arms ahora:
    // cualquiera de los dos vale, `or` evita duplicar el texto.
    let aviso_kb_root = aviso_fts.or(vector.aviso_kb_root);
    let v_por_entidad: HashMap<String, f64> = vector
        .results
        .into_iter()
        .map(|r| (r.permalink, r.score))
        .collect();

    let mut results = fusiona(&v_por_entidad, &f_por_entidad, bonus, limite);
    enriquece_rutas(&conn, &mut results)?;

    Ok(Busqueda {
        query: query.to_string(),
        search_type: "hybrid".to_string(),
        elapsed_s: inicio.elapsed().as_secs_f64(),
        results,
        avisos,
        aviso_kb_root,
    })
}
```

- [ ] **Step 4: Correr el test y verlo pasar**

Run: `cd engine && cargo test --release --locked busca_hybrid_abre_una_sola_conexion -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Correr toda la suite de `buscador.rs` y la de CLI**

Run: `cd engine && cargo test --release --locked buscador -- --nocapture`
Expected: todos verdes (los tests `tests_fusion` y `tests_knn_por_consulta`
no cambian de comportamiento, solo de qué función interna los sirve).

Run: `cd engine && cargo test --release --locked --test buscador_cli --test search_no_results_cli -- --nocapture`
Expected: todos verdes (contrato de `exo search` idéntico).

- [ ] **Step 6: Verificación de no-regresión con la KB semilla**

Run (usa la KB generada en la Task 1, `/tmp/kbg-despues`, o genera una
nueva con el binario ya recompilado):
```bash
cd engine && cargo build --release --locked --bin exo
EXO_CONFIG=/tmp/kbg-despues/config.toml ./target/release/exo search \
  --db /tmp/kbg-despues/index.db --kb /tmp/kbg-despues/kb \
  --type hybrid --min-similarity 0.40 --limit 5 --json \
  'trinquete techos indice memoria' > /tmp/hybrid-despues.json
git stash
cargo build --release --locked --bin exo
EXO_CONFIG=/tmp/kbg-despues/config.toml ./target/release/exo search \
  --db /tmp/kbg-despues/index.db --kb /tmp/kbg-despues/kb \
  --type hybrid --min-similarity 0.40 --limit 5 --json \
  'trinquete techos indice memoria' > /tmp/hybrid-antes.json
git stash pop
diff <(jq -S 'del(.data.elapsed_s)' /tmp/hybrid-antes.json) \
     <(jq -S 'del(.data.elapsed_s)' /tmp/hybrid-despues.json)
```

Expected: diff vacío (`elapsed_s` es el único campo que puede variar entre
corridas — se excluye explícitamente).

- [ ] **Step 7: Commit**

```bash
git add engine/src/buscador.rs
git commit -m "refactor(g, buscador): busca_hybrid abre la DB una sola vez en vez de tres"
```

---

### Task 3: `git_epoch_de` en batch — un `git log` por `indexa`, no uno por nota

**Lane:** mecánica. **Depende de:** nada.

**Files:**
- Modify: `engine/src/gitx.rs`
- Modify: `engine/src/indexer.rs`
- Test: `engine/tests/indexer.rs`

**Interfaces:**
- Consumes: `Command` de `std::process` (ya usado en `gitx.rs`).
- Produces: `pub fn epochs_de_todo_el_historial(dir: &Path) -> Result<HashMap<String, i64>>`
  en `gitx.rs` — nueva función pública, fail-loud (mismo idioma que el resto
  del módulo). `indexer::git_epoch_de(kb: &Path, ruta_rel: &Path) -> Option<i64>`
  **no cambia de firma ni de comportamiento** — sigue siendo el fallback
  fail-silent per-nota.

- [ ] **Step 1: Test que falla — el lote coincide con el fallback per-nota**

Añade a `engine/tests/indexer.rs` (usa los helpers `git`/`crea_nota`/
`db_temporal` ya definidos en ese fichero):

```rust
#[test]
fn git_epoch_por_lote_coincide_con_el_fallback_por_nota() {
    let dir = tempfile::tempdir().unwrap();
    let kb = dir.path();
    git(kb, &["init", "-q"]);
    git(kb, &["config", "user.email", "test@exo.local"]);
    git(kb, &["config", "user.name", "exo-test"]);

    let commit_con_fecha = |archivo: &str, fecha: &str| {
        git(kb, &["add", archivo]);
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(kb)
            .args(["commit", "-q", "-m", archivo])
            .env("GIT_AUTHOR_DATE", fecha)
            .env("GIT_COMMITTER_DATE", fecha)
            .status()
            .expect("git commit");
        assert!(status.success());
    };

    crea_nota(kb, "a.md", "kb-test/a", "Nota A", "contenido alfa");
    commit_con_fecha("a.md", "2026-07-01T10:00:00+00:00");

    crea_nota(kb, "b.md", "kb-test/b", "Nota B", "contenido beta");
    commit_con_fecha("b.md", "2026-07-02T10:00:00+00:00");

    // c.md nunca se commitea: ejercita el fallback per-nota (fuera del mapa
    // de lote, `git_epoch_de` sobre un fichero sin commits da `None`).
    crea_nota(kb, "c.md", "kb-test/c", "Nota C", "contenido gamma");

    let (_guard, db) = db_temporal();
    indexa(kb, &db).unwrap();

    let conn = exo::abre_db(&db).unwrap();
    let leer = |ruta: &str| -> Option<i64> {
        conn.query_row(
            "SELECT git_epoch FROM notas WHERE ruta = ?1",
            rusqlite::params![ruta],
            |r| r.get(0),
        )
        .unwrap()
    };

    let esperado_a = exo::indexer::git_epoch_de(kb, Path::new("a.md"));
    let esperado_b = exo::indexer::git_epoch_de(kb, Path::new("b.md"));
    let esperado_c = exo::indexer::git_epoch_de(kb, Path::new("c.md"));

    assert_eq!(leer("a.md"), esperado_a, "a.md debe coincidir con el fallback per-nota");
    assert_eq!(leer("b.md"), esperado_b, "b.md debe coincidir con el fallback per-nota");
    assert_eq!(leer("c.md"), esperado_c, "c.md sin commits debe seguir siendo None");
    assert_ne!(esperado_a, esperado_b, "a y b deben tener epochs distintos (commits en fechas distintas)");
}
```

- [ ] **Step 2: Correr el test y verlo fallar**

Run: `cd engine && cargo test --release --locked git_epoch_por_lote_coincide_con_el_fallback_por_nota -- --nocapture`
Expected: PASS ya (el test compara el resultado ACTUAL de `indexa` contra
`git_epoch_de` recalculado — con el código de hoy, `indexa` YA llama a
`git_epoch_de` per-nota, así que este test pasa incluso antes del cambio).
Esto es correcto: el test es un **test de equivalencia**, no de regresión —
prueba que el Step 3 no cambia el resultado, no que hoy esté roto. Confirma
que pasa ANTES de tocar nada (control), y vuelve a correrlo en el Step 5
para confirmar que sigue pasando tras el refactor de rendimiento.

- [ ] **Step 3: `gitx::epochs_de_todo_el_historial` — un solo `git log` para
  toda la KB**

Añade a `engine/src/gitx.rs`, después de `md_staged` y antes de
`head_resuelve`:

```rust
use std::collections::HashMap;

/// Epoch (segundos unix, fecha de COMMITTER `%ct` — mismo campo que
/// `indexer::git_epoch_de`, nunca `%at` de autor, que puede diferir en un
/// rebase/amend) del último commit que tocó cada ruta bajo `dir`, en una
/// sola invocación de git en vez de una por nota indexada (Ola 1 G Task 3,
/// backlog:524-566: "un proceso `git log -1` por nota indexada, caro en
/// Windows"). Recorre el log COMPLETO una vez
/// (`git log --format=%x01%ct --name-only`, `\x01` como separador de
/// registro que no puede aparecer en una ruta) y se queda con el PRIMER
/// epoch visto para cada ruta: git emite los commits del más reciente al
/// más antiguo, así que el primero que toca una ruta es su último commit.
///
/// Fail-loud como el resto de `gitx` (comentario de módulo): un `dir` que
/// no es repo de git, o cualquier fallo de `git log`, propaga `Err` — el
/// llamador (`indexer::indexa`) es quien decide degradar a fallback
/// per-nota, nunca esta función.
pub fn epochs_de_todo_el_historial(dir: &Path) -> Result<HashMap<String, i64>> {
    let salida = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["log", "--format=%x01%ct", "--name-only"])
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .output()
        .with_context(|| format!("invocar git log --name-only en {}", dir.display()))?;
    if !salida.status.success() {
        bail!(
            "git log --format=%x01%ct --name-only en {}: {}",
            dir.display(),
            String::from_utf8_lossy(&salida.stderr).trim()
        );
    }
    let texto = String::from_utf8(salida.stdout)
        .with_context(|| format!("git log de {} devolvió stdout no-UTF8", dir.display()))?;

    let mut epochs: HashMap<String, i64> = HashMap::new();
    let mut epoch_actual: Option<i64> = None;
    for linea in texto.lines() {
        if let Some(cifra) = linea.strip_prefix('\u{1}') {
            epoch_actual = cifra.trim().parse::<i64>().ok();
            continue;
        }
        if linea.is_empty() {
            continue;
        }
        if let Some(epoch) = epoch_actual {
            // `entry().or_insert()`: el PRIMER epoch visto para esta ruta es
            // el más reciente (git recorre de nuevo a viejo) — una entrada
            // posterior en el mismo historial es más vieja y no debe pisarla.
            epochs.entry(linea.replace('\\', "/")).or_insert(epoch);
        }
    }
    Ok(epochs)
}
```

Añade tests unitarios al `mod tests` de `gitx.rs` (después de
`una_ruta_con_separador_nativo_encuentra_su_commit`):

```rust
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
        assert_eq!(mapa.get("log/a.md"), Some(&ultimo_commit_epoch(raiz, "log/a.md")));
        assert_eq!(mapa.get("log/b.md"), Some(&ultimo_commit_epoch(raiz, "log/b.md")));
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
        String::from_utf8_lossy(&salida.stdout).trim().parse().unwrap()
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
```

- [ ] **Step 4: Usar el lote en `indexer::indexa`, con fallback per-nota**

old_string:
```rust
    let rutas_absolutas = walk_kb(kb)?;

    // Antes de comparar nada: las filas escritas por una versión anterior
    // llevan el separador nativo, y la comparación es por cadena exacta.
    migra_rutas_portables(&conn)?;
```

new_string:
```rust
    let rutas_absolutas = walk_kb(kb)?;

    // Antes de comparar nada: las filas escritas por una versión anterior
    // llevan el separador nativo, y la comparación es por cadena exacta.
    migra_rutas_portables(&conn)?;

    // Ola 1 G Task 3 (backlog:524-566): un `git log` para TODA la KB en vez
    // de uno por nota. `unwrap_or_default()` degrada a mapa vacío si `kb` no
    // es repo git o `git log` falla — mismo resultado final que antes
    // (fallback per-nota para cada ruta, `git_epoch_de` ya es fail-silent).
    let epochs_batch = crate::gitx::epochs_de_todo_el_historial(kb).unwrap_or_default();
```

old_string:
```rust
        let git_epoch = git_epoch_de(kb, Path::new(&ruta_rel));
```

new_string:
```rust
        // Primero el lote (una consulta); solo si la ruta no aparece ahí
        // (nunca commiteada, o el lote degradó a vacío) se paga el spawn
        // per-nota de `git_epoch_de`.
        let git_epoch = epochs_batch
            .get(&ruta_rel)
            .copied()
            .or_else(|| git_epoch_de(kb, Path::new(&ruta_rel)));
```

- [ ] **Step 5: Correr los tests y verlos pasar**

Run: `cd engine && cargo test --release --locked --lib gitx:: -- --nocapture`
Expected: los 3 tests nuevos de `gitx.rs` PASS.

Run: `cd engine && cargo test --release --locked git_epoch -- --nocapture`
Expected: `git_epoch_por_lote_coincide_con_el_fallback_por_nota` (indexer.rs)
y toda la suite existente de `tests/git_epoch.rs` PASS.

Run: `cd engine && cargo test --release --locked --test indexer -- --nocapture`
Expected: toda la suite verde (el batch no cambia ningún otro comportamiento
de `indexa`).

- [ ] **Step 6: Medir la mejora (bench de Task 1)**

Run:
```bash
cd .. && ./evals/recall-coste/harness/bench.sh g-task3-despues 5000
./evals/recall-coste/harness/compara.sh <etiqueta-antes-de-esta-task> g-task3-despues
```

Expected: `s3-index-sin-cambios-n5000` (`exo rebuild` a 5.000 notas) baja
frente a la corrida "antes" — un `git log --name-only` sobre un historial
de 5.000 commits sigue siendo un solo proceso, frente a 5.000 spawns.

- [ ] **Step 7: Commit**

```bash
git add engine/src/gitx.rs engine/src/indexer.rs engine/tests/indexer.rs
git commit -m "perf(g, git-epoch): un git log por indexa en vez de uno por nota, con fallback fail-silent per-nota"
```

---

### Task 4: Unificar `walk_kb` sobre `walk_kb_excluyendo` (cambio de comportamiento declarado)

**Lane:** mecánica, pero con **cambio de comportamiento declarado**
(decisión 9 de Paul): `walk_kb` deja de ser case-sensitive con la extensión
y deja de recorrer dentro de `.git/` (276/314 `openat` medidos por H29 caían
ahí). `doctor.rs` usa la misma función, así que hereda el cambio.

**Files:**
- Modify: `engine/src/walker.rs`
- Test: `engine/tests/walker.rs`
- Modify: `docs/arquitectura.md` (declarar el cambio)

**Interfaces:**
- Consumes: `walk_kb_excluyendo(raiz: &Path, excluidos: &[&str]) -> Result<(Vec<(String,String)>, Vec<String>)>`
  (ya existe, sin cambios).
- Produces: `pub fn walk_kb(raiz: &Path) -> Result<Vec<PathBuf>>` — misma
  firma pública, comportamiento nuevo (case-insensitive, no entra en
  dotdirs). Llamadores sin cambios: `indexer::indexa` (`indexer.rs:159`) y
  `doctor.rs:312,451`.

- [ ] **Step 1: Tests que fallan — los dos comportamientos nuevos**

Añade a `engine/tests/walker.rs`, después de `walker_orden_determinista`:

```rust
#[test]
fn walk_kb_ahora_es_case_insensitive_como_walk_notas() {
    // Cambio de comportamiento declarado (Ola 1 G Task 4, backlog:821-845,
    // decisión 9 de Paul): antes de unificar, un NOTA.MD no se indexaba
    // (walk_kb comparaba extensión exacta `Some("md")`); ahora sí.
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("NOTA.MD"), "x").unwrap();
    let rutas = walk_kb(dir.path()).unwrap();
    assert!(nombres(&rutas).contains(&"NOTA.MD".to_string()));
}

#[test]
fn walk_kb_ya_no_recorre_dentro_de_punto_git() {
    // Cambio de comportamiento declarado: antes walk_kb caminaba dentro de
    // `.git/` (276/314 openat medidos por H29); ahora lo salta, como
    // walk_kb_excluyendo.
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    fs::write(dir.path().join(".git/dentro.md"), "x").unwrap();
    fs::write(dir.path().join("fuera.md"), "x").unwrap();
    let rutas = walk_kb(dir.path()).unwrap();
    let vistas = nombres(&rutas);
    assert!(!vistas.contains(&"dentro.md".to_string()));
    assert!(vistas.contains(&"fuera.md".to_string()));
}
```

- [ ] **Step 2: Correr los tests y verlos fallar**

Run: `cd engine && cargo test --release --locked --test walker walk_kb_ahora_es_case_insensitive walk_kb_ya_no_recorre_dentro_de_punto_git -- --nocapture`
Expected: los dos FAIL (`NOTA.MD` no aparece en el primero; `dentro.md` sí
aparece en el segundo).

- [ ] **Step 3: Unificar `walk_kb`**

old_string:
```rust
/// Directorios excluidos en cualquier nivel del árbol (§6.2 regla 3).
const DOTDIRS_EXCLUIDOS: [&str; 3] = [".claude", ".omc", ".superpowers"];

/// Recorre `raiz` recursivamente y devuelve las rutas absolutas de todos los
/// ficheros `.md`, en orden determinista (ordenado por ruta), excluyendo
/// `.claude/`, `.omc/` y `.superpowers/` en cualquier nivel. `archive/` SE
/// incluye (§6.2 regla 4).
pub fn walk_kb(raiz: &Path) -> Result<Vec<PathBuf>> {
    let mut encontradas = Vec::new();
    visita(raiz, &mut encontradas)?;
    encontradas.sort();
    Ok(encontradas)
}

fn visita(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entradas =
        std::fs::read_dir(dir).with_context(|| format!("leer directorio {}", dir.display()))?;
    for entrada in entradas {
        let entrada = entrada.with_context(|| format!("entrada de {}", dir.display()))?;
        let ruta = entrada.path();
        let tipo = entrada
            .file_type()
            .with_context(|| format!("file_type de {}", ruta.display()))?;

        if tipo.is_dir() {
            let nombre = entrada.file_name();
            let nombre = nombre.to_string_lossy();
            if DOTDIRS_EXCLUIDOS.contains(&nombre.as_ref()) {
                continue;
            }
            visita(&ruta, out)?;
        } else if tipo.is_file() && ruta.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(ruta);
        }
    }
    Ok(())
}
```

new_string:
```rust
/// Recorre `raiz` recursivamente y devuelve las rutas absolutas de todos los
/// ficheros `.md` (case-insensitive, `es_md`), en orden determinista
/// (ordenado por ruta). Excluye TODO directorio que empiece por `.` en
/// cualquier nivel — `.git/` incluido — igual que `walk_kb_excluyendo`.
/// `archive/` SE incluye (§6.2 regla 4).
///
/// Unificada sobre `walk_kb_excluyendo` (Ola 1 G Task 4, backlog:821-845,
/// decisión 9 de Paul: las dos funciones convivían con semánticas
/// distintas desde G4b — `walk_kb` no normalizaba mayúsculas y caminaba
/// dentro de `.git/`, 276/314 `openat` medidos por H29). **Cambio de
/// comportamiento declarado**: un `NOTA.MD` que antes no se indexaba ahora
/// sí; un `.git/x.md` que antes se recorría ahora no. Documentado en
/// `docs/arquitectura.md`; `exo rebuild` recomendado tras actualizar si la
/// KB tiene notas con extensión en mayúsculas.
pub fn walk_kb(raiz: &Path) -> Result<Vec<PathBuf>> {
    let (_, notas_rel) = walk_kb_excluyendo(raiz, &[])?;
    Ok(notas_rel.into_iter().map(|rel| raiz.join(rel)).collect())
}
```

- [ ] **Step 4: Correr los tests y verlos pasar**

Run: `cd engine && cargo test --release --locked --test walker -- --nocapture`
Expected: TODOS los tests de `walker.rs` verdes, incluidos los 4 anteriores
(`walker_excluye_dotdirs`, `walker_solo_markdown`, `walker_incluye_archive`,
`walker_orden_determinista` — ninguno cambia de comportamiento, `archive/`
no es un dotdir y `walk_kb_excluyendo(_, &[])` no filtra por primer
segmento) y los 2 nuevos.

- [ ] **Step 5: Correr la suite completa del engine (walk_kb lo usan
  `indexer` y `doctor`)**

Run: `cd engine && cargo test --release --locked --test indexer --test doctor --test doctor_cli -- --nocapture`
Expected: todos verdes — ninguna fixture existente de esos tres ficheros
usa un `.MD` en mayúsculas ni un `.md` dentro de `.git/`, así que el cambio
de comportamiento no dispara ninguna aserción existente.

- [ ] **Step 6: Declarar el cambio en `docs/arquitectura.md`**

Busca la sección que documenta `walk_kb`/exclusiones (probablemente §3 o
§6, cerca de la doctrina de la KB) y añade, si no existe ya una nota
equivalente:

```markdown
`walk_kb` (usada por `exo index`/`rebuild` y por `exo doctor`) y
`walk_kb_excluyendo` (usada por `exo lint`/`exo budget`) comparten una sola
implementación desde el 2026-09-15 (Ola 1, campaña G): excluyen cualquier
directorio que empiece por `.` en cualquier nivel (`.git/` incluido) y
reconocen `.md` sin distinguir mayúsculas. Antes de esa fecha, `walk_kb`
tenía una semántica distinta (case-sensitive, sin excluir `.git/`); si tu KB
tiene notas `.MD` en mayúsculas que antes no se indexaban, corre
`exo rebuild` tras actualizar.
```

(Si `docs/arquitectura.md` no tiene una sección natural para esto, añádela
al final de la sección que describe el pipeline de indexado.)

- [ ] **Step 7: Commit**

```bash
git add engine/src/walker.rs engine/tests/walker.rs docs/arquitectura.md
git commit -m "refactor(g, walker): unifica walk_kb sobre walk_kb_excluyendo — case-insensitive, ya no camina dentro de .git/ (decisión 9 de Paul)"
```

---

### Task 5: `budget_prose_drift` — la captura ya no trunca una cifra mal agrupada

**Lane:** mecánica. **Depende de:** nada. El bloqueador original («cuando
exista el gate de paridad con Go») **caducó**: los gates de paridad
corrieron en la campaña D y kbx dejó de ser referencia (`backlog:181`); el
fix es en Rust solo.

**Files:**
- Modify: `engine/src/lint.rs`
- Test: `engine/tests/lint_presupuesto.rs`

**Interfaces:**
- Consumes: nada nuevo.
- Produces: `pub fn deriva_de_prosa(kb: &Path, rutas: &[String], presupuestos: Presupuestos, excluidos: &[&str]) -> Result<Vec<Hallazgo>>`
  — firma pública sin cambios.

- [ ] **Step 1: Test que falla**

Añade a `engine/tests/lint_presupuesto.rs`, después de
`la_deriva_de_prosa_parsea_las_cuatro_grafias`:

```rust
#[test]
fn la_deriva_de_prosa_ignora_una_cifra_mal_agrupada() {
    // Task 5 (Ola 1 G, backlog:878-910): antes, "1.2345" (grupo de 4
    // dígitos tras el punto, no de 3) truncaba a "1.234" y el parse
    // producía el hallazgo falso "cita core 1234B" — una cifra que no está
    // en el texto. Ahora la captura consume el número ENTERO y, si no
    // agrupa en tríos, se ignora sin generar hallazgo (falsos positivos
    // pesan más que fallos, mismo criterio que la mención vaga).
    let dir = kb_con(&[(
        "c.md",
        "---\ntier: core\n---\n\nPresupuesto: core 1.2345 B.\n".to_string(),
    )]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let h =
        lint::deriva_de_prosa(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(h.is_empty(), "cifra mal agrupada no debe producir hallazgo: {h:?}");
}
```

- [ ] **Step 2: Correr el test y verlo fallar**

Run: `cd engine && cargo test --release --locked la_deriva_de_prosa_ignora_una_cifra_mal_agrupada -- --nocapture`
Expected: FAIL — `h` no está vacío, contiene `Hallazgo { detalle: "cita core
1234B, el tool aplica 8500B", .. }` (o el nominal que tenga `NOMINALES`).

- [ ] **Step 3: Regex que consume el número entero + validación de
  agrupación**

old_string:
```rust
/// Un tier seguido inmediatamente de una cifra: "core 8.500 B", "stable 12500".
/// El "." como separador de miles porque la KB está escrita en castellano.
/// Exigir que la cifra vaya pegada al tier es lo que evita que la frase
/// "el presupuesto y las 3 notas core" cuente como cita.
static TIER_Y_CIFRA: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"\b(core|stable|log)\b[:\s]+([0-9]+(?:\.[0-9]{3})*)\s*(?:B\b|bytes\b)?")
        .unwrap()
});
```

new_string:
```rust
/// Un tier seguido inmediatamente de una cifra: "core 8.500 B", "stable 12500".
/// El "." como separador de miles porque la KB está escrita en castellano.
/// Exigir que la cifra vaya pegada al tier es lo que evita que la frase
/// "el presupuesto y las 3 notas core" cuente como cita.
///
/// El grupo 2 consume TODA la racha de dígitos y puntos que sigue al tier
/// (`[0-9]+(?:\.[0-9]+)*`, sin exigir grupos de 3 aquí) — la validación de
/// que agrupa correctamente en tríos vive aparte, en `agrupacion_correcta`
/// (Task 5, Ola 1 G, backlog:878-910: la crate `regex` no tiene lookahead,
/// así que "consume el número entero y rechaza si no cuadra" no se puede
/// expresar en una sola pasada — antes, `(?:\.[0-9]{3})*` paraba en el
/// primer grupo mal formado y dejaba el resto suelto, truncando "1.2345" a
/// "1.234" en vez de reconocer que el número entero no agrupa).
static TIER_Y_CIFRA: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"\b(core|stable|log)\b[:\s]+([0-9]+(?:\.[0-9]+)*)\s*(?:B\b|bytes\b)?")
        .unwrap()
});

/// ¿Es `cifra` una agrupación de miles válida ("8.500", "12500", pero no
/// "1.2345")? Sin punto: siempre válida (número simple, "12500"). Con
/// punto: el primer grupo de 1 a 3 dígitos, cada grupo siguiente de
/// EXACTAMENTE 3 (el criterio que `TIER_Y_CIFRA` ya no puede aplicar en la
/// propia regex tras ensanchar la captura).
fn agrupacion_correcta(cifra: &str) -> bool {
    if !cifra.contains('.') {
        return true;
    }
    let grupos: Vec<&str> = cifra.split('.').collect();
    let Some((primero, resto)) = grupos.split_first() else {
        return false;
    };
    (1..=3).contains(&primero.len()) && resto.iter().all(|g| g.len() == 3)
}
```

Ahora usa `agrupacion_correcta` antes de parsear, dentro de `deriva_de_prosa`:

old_string:
```rust
            for c in TIER_Y_CIFRA.captures_iter(linea) {
                let tier_citado = &c[1];
                // Una cifra que no parsea se salta: inventar un hallazgo desde
                // una línea no parseada es cómo un gate empieza a gritar y
                // acaba ignorado.
                let Ok(citada) = c[2].replace('.', "").parse::<i64>() else {
                    continue;
                };
```

new_string:
```rust
            for c in TIER_Y_CIFRA.captures_iter(linea) {
                let tier_citado = &c[1];
                let cifra_cruda = &c[2];
                // Task 5: una captura que no agrupa en tríos es un número
                // mal formado en el texto — se ignora, no se inventa un
                // hallazgo con una cifra que no está escrita.
                if !agrupacion_correcta(cifra_cruda) {
                    continue;
                }
                // Una cifra que no parsea se salta: inventar un hallazgo desde
                // una línea no parseada es cómo un gate empieza a gritar y
                // acaba ignorado.
                let Ok(citada) = cifra_cruda.replace('.', "").parse::<i64>() else {
                    continue;
                };
```

- [ ] **Step 4: Correr el test y verlo pasar**

Run: `cd engine && cargo test --release --locked la_deriva_de_prosa_ignora_una_cifra_mal_agrupada -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Correr toda la suite de `lint_presupuesto.rs`**

Run: `cd engine && cargo test --release --locked --test lint_presupuesto -- --nocapture`
Expected: TODOS verdes, incluido
`la_deriva_de_prosa_parsea_las_cuatro_grafias` (las 4 grafías siguen
agrupando correctamente: "1.000", "2000" sin punto, "3.000", "4000" sin
punto) y `la_deriva_de_prosa_acepta_el_separador_de_miles_y_la_cifra_correcta`
("8.500", "12500").

- [ ] **Step 6: Commit**

```bash
git add engine/src/lint.rs engine/tests/lint_presupuesto.rs
git commit -m "fix(g, lint): budget_prose_drift ignora una cifra que no agrupa en tríos, en vez de truncarla y citar un número que no está en el texto"
```

---

### Task 6: `escribe_nueva(&NuevaNota{…})` — struct de parámetros, sin `allow`

**Lane:** mecánica. **Depende de:** nada. **Debe ejecutarse antes que la
Task 10** (ambas tocan `engine/tests/escritor.rs`; la Task 10 renombra
`kb-demo` en las mismas líneas que esta task reescribe).

**Files:**
- Modify: `engine/src/escritor.rs`
- Modify: `engine/src/main.rs:794-868` (`write_new_cmd`, `write_append_cmd`)
- Test: `engine/tests/escritor.rs`

**Interfaces:**
- Consumes: nada nuevo.
- Produces: `pub struct NuevaNota<'a> { pub kb: &'a Path, pub proyecto: &'a str, pub dir: &'a str, pub titulo: &'a str, pub cuerpo: &'a str, pub tier: Option<&'a str>, pub dup_candidatas: &'a [(String, f64)], pub forzado: bool }`
  y `pub fn escribe_nueva(n: &NuevaNota) -> Result<Escritura>` en
  `escritor.rs` — la Task 7 (misma rama, después) construye `NuevaNota`
  para su fix de M4 #5, así que hereda esta firma.

- [ ] **Step 1: Test que falla — la nueva firma compila**

Modifica el PRIMER test de `engine/tests/escritor.rs` (el resto se arregla
en el Step 3, este primero sirve de test que falla / guía):

old_string:
```rust
#[test]
fn nueva_genera_frontmatter_completo_y_ruta_correcta() {
    let kb = kb_falsa();
    let esc = escribe_nueva(
        kb.path(),
        "kb-demo",
        "projects",
        "Proyecto Nuevo — de prueba",
        "cuerpo de la nota\n",
        Some("stable"),
        &[],
        false,
    )
    .unwrap();
```

new_string:
```rust
#[test]
fn nueva_genera_frontmatter_completo_y_ruta_correcta() {
    let kb = kb_falsa();
    let esc = escribe_nueva(&NuevaNota {
        kb: kb.path(),
        proyecto: "kb-demo",
        dir: "projects",
        titulo: "Proyecto Nuevo — de prueba",
        cuerpo: "cuerpo de la nota\n",
        tier: Some("stable"),
        dup_candidatas: &[],
        forzado: false,
    })
    .unwrap();
```

Y el import, arriba del fichero:

old_string:
```rust
use exo::escritor::{Rechazo, escribe_append, escribe_nueva, slug};
```

new_string:
```rust
use exo::escritor::{NuevaNota, Rechazo, escribe_append, escribe_nueva, slug};
```

- [ ] **Step 2: Correr el test y verlo fallar (no compila)**

Run: `cd engine && cargo test --release --locked --test escritor 2>&1 | head -30`
Expected: FAIL en compilación — `error[E0308]: mismatched types` o
`error[E0061]: this function takes 1 argument but 8 arguments were supplied`
en los 7 call sites restantes de `escribe_nueva(...)` en el mismo fichero,
y en `main.rs`.

- [ ] **Step 3: `NuevaNota` en `escritor.rs`, quitar el `allow`**

old_string:
```rust
/// Crea una nota nueva. `cuerpo` puede traer frontmatter propio: se conserva
/// **literal** y solo se le añaden delante las claves que falten (M4-03).
/// Nunca sobrescribe un fichero existente — eso es error duro, no gate: lo
/// correcto ante una colisión es append o edit.
///
/// `dup_candidatas` lo calcula el llamador (el CLI, con `busca_hybrid`): este
/// módulo no conoce el índice, solo el filesystem. Si llega no vacío,
/// `Rechazo::Duplicada` sin tocar el disco.
// Ocho parámetros contra el umbral de 7 de clippy. Se declara en vez de
// refactorizar: agrupar en una struct de parámetros toca el camino de
// escritura y sus tests, y el cambio que trajo este `allow` era montar el CI
// (G5a, 2026-09-02). La deuda es la struct de parámetros, no el `allow`.
#[allow(clippy::too_many_arguments)]
pub fn escribe_nueva(
    kb: &Path,
    proyecto: &str,
    dir: &str,
    titulo: &str,
    cuerpo: &str,
    tier: Option<&str>,
    dup_candidatas: &[(String, f64)],
    forzado: bool,
) -> Result<Escritura> {
    if !dup_candidatas.is_empty() {
        return Err(Rechazo::Duplicada {
            candidatas: dup_candidatas
                .iter()
                .map(|(permalink, score)| Candidata {
                    permalink: permalink.clone(),
                    score: *score,
                })
                .collect(),
        }
        .into());
    }

    verifica_segmento(dir, "--dir")?;
    verifica_segmento(titulo, "--title")?;

    let permalink = format!("{proyecto}/{dir}/{}", slug(titulo));
    let ruta_rel = format!("{dir}/{}.md", nombre_fichero(titulo));
    let ruta_abs = kb.join(&ruta_rel);

    if ruta_abs.exists() {
        anyhow::bail!(
            "{} ya existe: una nota jamás se sobrescribe (usa append, o edita con Edit)",
            ruta_abs.display()
        );
    }

    let (yaml_previo, cuerpo_limpio) = separa_frontmatter(cuerpo);
    let (frontmatter, completado) = compone_frontmatter(&yaml_previo, titulo, &permalink, tier);

    let contenido = format!("---\n{frontmatter}---\n{cuerpo_limpio}");
    escribe_atomico(&ruta_abs, &contenido)?;

    Ok(Escritura {
        op: "new".into(),
        permalink,
        ruta_rel,
        ruta_abs: crate::walker::ruta_portable(&ruta_abs.display().to_string()),
        creada: true,
        frontmatter_completado: completado,
        forzado,
    })
}
```

new_string:
```rust
/// Parámetros de `escribe_nueva` (Ola 1 G Task 6, backlog:637-647): antes
/// eran 8 argumentos posicionales contra el umbral de 7 de clippy, con
/// `#[allow(clippy::too_many_arguments)]` declarado como deuda desde G5a
/// (2026-09-02). `dup_candidatas` lo calcula el llamador (el CLI, con
/// `busca_hybrid`): este módulo no conoce el índice, solo el filesystem.
pub struct NuevaNota<'a> {
    pub kb: &'a Path,
    pub proyecto: &'a str,
    pub dir: &'a str,
    pub titulo: &'a str,
    pub cuerpo: &'a str,
    pub tier: Option<&'a str>,
    pub dup_candidatas: &'a [(String, f64)],
    pub forzado: bool,
}

/// Crea una nota nueva. `cuerpo` puede traer frontmatter propio: se conserva
/// **literal** y solo se le añaden delante las claves que falten (M4-03).
/// Nunca sobrescribe un fichero existente — eso es error duro, no gate: lo
/// correcto ante una colisión es append o edit. Si `dup_candidatas` llega no
/// vacío, `Rechazo::Duplicada` sin tocar el disco.
pub fn escribe_nueva(n: &NuevaNota) -> Result<Escritura> {
    if !n.dup_candidatas.is_empty() {
        return Err(Rechazo::Duplicada {
            candidatas: n
                .dup_candidatas
                .iter()
                .map(|(permalink, score)| Candidata {
                    permalink: permalink.clone(),
                    score: *score,
                })
                .collect(),
        }
        .into());
    }

    verifica_segmento(n.dir, "--dir")?;
    verifica_segmento(n.titulo, "--title")?;

    let permalink = format!("{}/{}/{}", n.proyecto, n.dir, slug(n.titulo));
    let ruta_rel = format!("{}/{}.md", n.dir, nombre_fichero(n.titulo));
    let ruta_abs = n.kb.join(&ruta_rel);

    if ruta_abs.exists() {
        anyhow::bail!(
            "{} ya existe: una nota jamás se sobrescribe (usa append, o edita con Edit)",
            ruta_abs.display()
        );
    }

    let (yaml_previo, cuerpo_limpio) = separa_frontmatter(n.cuerpo);
    let (frontmatter, completado) = compone_frontmatter(&yaml_previo, n.titulo, &permalink, n.tier);

    let contenido = format!("---\n{frontmatter}---\n{cuerpo_limpio}");
    escribe_atomico(&ruta_abs, &contenido)?;

    Ok(Escritura {
        op: "new".into(),
        permalink,
        ruta_rel,
        ruta_abs: crate::walker::ruta_portable(&ruta_abs.display().to_string()),
        creada: true,
        frontmatter_completado: completado,
        forzado: n.forzado,
    })
}
```

- [ ] **Step 4: Actualizar los dos call sites de `main.rs`**

old_string:
```rust
    let esc = escribe_nueva(
        &kb,
        &proyecto,
        &args.dir,
        &args.titulo,
        &cuerpo,
        args.tier.as_deref(),
        &candidatas,
        args.force,
    )?;
```

new_string:
```rust
    let esc = escribe_nueva(&NuevaNota {
        kb: &kb,
        proyecto: &proyecto,
        dir: &args.dir,
        titulo: &args.titulo,
        cuerpo: &cuerpo,
        tier: args.tier.as_deref(),
        dup_candidatas: &candidatas,
        forzado: args.force,
    })?;
```

old_string:
```rust
                escribe_nueva(&kb, &proyecto, dir, slug_nota, "", Some("log"), &[], false)
                    .context("crear la bitácora con --create")?;
```

new_string:
```rust
                escribe_nueva(&NuevaNota {
                    kb: &kb,
                    proyecto: &proyecto,
                    dir,
                    titulo: slug_nota,
                    cuerpo: "",
                    tier: Some("log"),
                    dup_candidatas: &[],
                    forzado: false,
                })
                .context("crear la bitácora con --create")?;
```

Y el import agrupado de `main.rs`:

old_string:
```rust
    escritor::{escribe_append, escribe_nueva},
```

new_string:
```rust
    escritor::{NuevaNota, escribe_append, escribe_nueva},
```

- [ ] **Step 5: Actualizar los 7 call sites restantes de
  `engine/tests/escritor.rs`**

Cada uno sigue el mismo patrón del Step 1: posicional → struct literal.
Localízalos con `grep -n "escribe_nueva(" engine/tests/escritor.rs` (tras el
Step 1 quedan 7). Para cada uno:

- `nueva_respeta_el_frontmatter_que_ya_trae_el_cuerpo` (proyecto="kb-demo",
  dir="projects", titulo="Con Tags", cuerpo con frontmatter propio,
  tier=Some("log"), dup_candidatas=&[], forzado=false).
- `nueva_jamas_pisa_una_nota_existente` (titulo="Ya Existe", tier=None).
- `nueva_con_candidatas_duplicadas_rechaza_sin_escribir`
  (dup_candidatas=&[("kb-demo/projects/tema-repe".into(), 0.9)]).
- `titulo_con_barra_no_crea_subdirectorio`.
- `jamas_se_escribe_fuera_de_la_kb` (dentro del `for` — construye
  `NuevaNota` con `dir`/`titulo` del bucle, `forzado: true`).
- `new_forzado_queda_registrado_en_el_envelope` (`forzado: true`).
- `el_absolute_path_de_write_no_lleva_barra_invertida`
  (`exo::escritor::escribe_nueva(&exo::escritor::NuevaNota{...})` — este
  call site usa el path completo `exo::escritor::escribe_nueva`, no el
  importado; usa igualmente `exo::escritor::NuevaNota{...}`).

Cada conversión es mecánica: los 8 argumentos posicionales pasan a ser,
en el mismo orden, los 8 campos de `NuevaNota` (`kb`, `proyecto`, `dir`,
`titulo`, `cuerpo`, `tier`, `dup_candidatas`, `forzado`) — el valor de cada
uno NO cambia, solo su forma de paso.

- [ ] **Step 6: Correr la suite y verla pasar**

Run: `cd engine && cargo test --release --locked --test escritor -- --nocapture`
Expected: TODOS los tests de `escritor.rs` verdes.

- [ ] **Step 7: Verificar que el `allow` desapareció y clippy sigue verde**

Run: `cd engine && grep -rn "too_many_arguments" src/`
Expected: sin coincidencias.

Run: `cd engine && cargo clippy --all-targets --locked -- -D warnings`
Expected: exit 0.

- [ ] **Step 8: Correr toda la suite de escritura**

Run: `cd engine && cargo test --release --locked --test write_create_permalink --test rechazo_envelope -- --nocapture`
Expected: todos verdes (`NuevaNota` no cambia ningún comportamiento
observable de `write new`/`write append --create`).

- [ ] **Step 9: Commit**

```bash
git add engine/src/escritor.rs engine/src/main.rs engine/tests/escritor.rs
git commit -m "refactor(g, escritor): escribe_nueva toma &NuevaNota en vez de 8 posicionales, quita el allow(too_many_arguments)"
```

---

### Task 7: M4 #5 (walk de confirmación) y #6 (permalink de <3 segmentos)

**Lane:** mecánica. **Depende de:** Task 6 (usa `NuevaNota`).

**Files:**
- Modify: `engine/src/main.rs:832-868` (`write_append_cmd`)
- Test: `engine/tests/write_create_permalink.rs`

**Interfaces:**
- Consumes: `NuevaNota` (Task 6), `exo::nota::parsea_nota(ruta: &Path) -> Result<Option<exo::nota::Nota>>`
  (ya existe), `exo::walker::es_md(nombre: &str) -> bool` (ya existe).
- Produces: `fn busca_permalink_en_dir(kb: &Path, dir: &str, permalink: &str) -> Result<Option<PathBuf>>`
  — helper privado nuevo de `main.rs`, no consumido por otra task.

- [ ] **Step 1: Test que falla — permalink de 2 segmentos**

Añade a `engine/tests/write_create_permalink.rs`:

```rust
#[test]
fn write_append_create_con_permalink_de_dos_segmentos_falla_con_remedio() {
    // M4 #6 (Ola 1 G Task 7, backlog:708-730): antes, un permalink de solo
    // <proyecto>/<slug> colaba el primer segmento como si fuera <dir>, y
    // creaba `<primer-segmento>/<slug>.md` — un directorio espurio. Ahora
    // es error accionable, sin tocar el disco.
    let kb_tmp = tempfile::tempdir().expect("tempdir kb");
    let kb = kb_tmp.path().to_path_buf();

    let work = tempfile::tempdir().expect("tempdir work");
    let db = work.path().join("index.db");
    let cuerpo_path = work.path().join("cuerpo.txt");
    std::fs::write(&cuerpo_path, "cuerpo\n").expect("escribir cuerpo");

    common::con_config(&kb, "proyecto", &db, || {
        exo::indexer::indexa(&kb, &db).expect("bootstrap del índice");

        let out = std::process::Command::new(bin())
            .args(["write", "append", "--create", "--from"])
            .arg(&cuerpo_path)
            .args(["--kb"])
            .arg(&kb)
            .args(["--db"])
            .arg(&db)
            .args(["proyecto/slug-sin-dir"])
            .output()
            .expect("correr el binario");

        assert_ne!(
            out.status.code(),
            Some(0),
            "un permalink de 2 segmentos debe fallar, no crear un directorio espurio"
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("3 segmentos") || stderr.contains("<dir>"),
            "el error debe nombrar el remedio (forma <proyecto>/<dir>/<slug>): {stderr}"
        );
        assert!(
            !kb.join("proyecto").exists(),
            "no debe crearse el directorio espurio 'proyecto/': {stderr}"
        );
    });
}

#[test]
fn write_append_create_no_duplica_si_el_indice_esta_rancio() {
    // M4 #5 (Ola 1 G Task 7, backlog:708-730): el fichero real vive bajo un
    // nombre distinto del canónico (renombrado a mano tras un `exo index`
    // que no se volvió a correr) pero declara el MISMO permalink. Antes,
    // `--create` solo miraba la ruta canónica y creaba un segundo fichero
    // con el mismo permalink; el walk de confirmación tiene que atraparlo.
    let kb_tmp = tempfile::tempdir().expect("tempdir kb");
    let kb = kb_tmp.path().to_path_buf();
    std::fs::create_dir_all(kb.join("log")).expect("crear log/");

    let work = tempfile::tempdir().expect("tempdir work");
    let db = work.path().join("index.db");
    let cuerpo_path = work.path().join("cuerpo.txt");
    std::fs::write(&cuerpo_path, "cuerpo nuevo\n").expect("escribir cuerpo");

    let permalink = "proyecto/log/bitacora-vieja";
    std::fs::write(
        kb.join("log/Bitácora Renombrada A Mano.md"),
        format!(
            "---\npermalink: {permalink}\ntitle: Bitácora Renombrada A Mano\n---\ncontenido viejo\n"
        ),
    )
    .expect("escribir bitacora renombrada");

    common::con_config(&kb, "proyecto", &db, || {
        // El índice queda RANCIO a propósito: se corre sobre una KB que ya
        // tiene el fichero renombrado, así que ni siquiera entra bajo su
        // nombre canónico — el punto del test es que NINGÚN índice, ni
        // fresco ni rancio, evita el walk de confirmación de disco.
        exo::indexer::indexa(&kb, &db).expect("bootstrap del índice");

        let out = std::process::Command::new(bin())
            .args(["write", "append", "--create", "--from"])
            .arg(&cuerpo_path)
            .args(["--kb"])
            .arg(&kb)
            .args(["--db"])
            .arg(&db)
            .args([permalink])
            .output()
            .expect("correr el binario");

        assert_ne!(
            out.status.code(),
            Some(0),
            "no debe crear un segundo fichero con el mismo permalink"
        );
        assert!(
            !kb.join("log/bitacora-vieja.md").exists(),
            "no debe crearse el fichero canónico duplicado"
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("ya existe con este permalink"),
            "el error debe nombrar la causa: {stderr}"
        );
    });
}
```

- [ ] **Step 2: Correr los tests y verlos fallar**

Run: `cd engine && cargo build --release --locked --bin exo && cargo test --release --locked --test write_create_permalink -- --nocapture`
Expected: los 2 tests nuevos FAIL — el primero porque el binario sale `0` y
crea `proyecto/slug-sin-dir.md`; el segundo porque el binario sale `0` y
crea `log/bitacora-vieja.md` duplicado.

- [ ] **Step 3: Arreglar el parseo de segmentos y añadir el walk de
  confirmación**

old_string:
```rust
    let ruta_rel = match exo::buscador::ruta_de(&db, &args.permalink)? {
        Some(r) => r,
        None if args.crea => {
            // Sin fila en el índice: o la bitácora no existe, o el índice está
            // rancio. Derivar la ruta del permalink es correcto SOLO para
            // crearla (`log/<slug>.md`); para una nota ya existente el slug no
            // es invertible y por eso jamás se adivina.
            let (dir, slug_nota) = args
                .permalink
                .rsplit_once('/')
                .map(|(izq, slug)| (izq.rsplit_once('/').map_or(izq, |(_, d)| d), slug))
                .context("permalink sin forma <proyecto>/<dir>/<slug>")?;
            let rel = format!("{dir}/{slug_nota}.md");

            if !kb.join(&rel).exists() {
                let proyecto = exo::nombre_kb()?;
                escribe_nueva(&NuevaNota {
                    kb: &kb,
                    proyecto: &proyecto,
                    dir,
                    titulo: slug_nota,
                    cuerpo: "",
                    tier: Some("log"),
                    dup_candidatas: &[],
                    forzado: false,
                })
                .context("crear la bitácora con --create")?;
                eprintln!("write: bitácora creada en {rel}");
            }
            rel
        }
        None => anyhow::bail!(
            "{} no está en el índice: comprueba el permalink, o usa --create si la bitácora aún no existe",
            args.permalink
        ),
    };
```

new_string:
```rust
    let ruta_rel = match exo::buscador::ruta_de(&db, &args.permalink)? {
        Some(r) => r,
        None if args.crea => {
            // Sin fila en el índice: o la bitácora no existe, o el índice está
            // rancio. Derivar la ruta del permalink es correcto SOLO para
            // crearla (`log/<slug>.md`); para una nota ya existente el slug no
            // es invertible y por eso jamás se adivina.
            //
            // M4 #6 (Ola 1 G Task 7, backlog:708-730): exige EXACTAMENTE 3
            // segmentos. Antes, un permalink de 2 segmentos
            // (<proyecto>/<slug>) colaba el primer segmento como <dir> vía
            // `map_or(izq, ...)`, creando un directorio espurio
            // (`<proyecto>/<slug>.md`) que no es ni el proyecto real ni un
            // dir pedido por nadie.
            let (izq, slug_nota) = args
                .permalink
                .rsplit_once('/')
                .context("permalink sin forma <proyecto>/<dir>/<slug>")?;
            let (_, dir) = izq.rsplit_once('/').with_context(|| {
                format!(
                    "{:?} tiene menos de 3 segmentos (<proyecto>/<dir>/<slug>): \
                     --create no puede inferir un directorio",
                    args.permalink
                )
            })?;
            let rel = format!("{dir}/{slug_nota}.md");

            if !kb.join(&rel).exists() {
                // M4 #5 (Ola 1 G Task 7, backlog:708-730): walk de
                // confirmación antes de crear. El check de arriba solo mira
                // la ruta CANÓNICA; con índice rancio, la bitácora real
                // puede vivir bajo OTRO nombre de fichero con el mismo
                // permalink (renombrada a mano tras `exo index`). Crear
                // otra encima dejaría DOS ficheros con el mismo permalink.
                if let Some(existente) = busca_permalink_en_dir(&kb, dir, &args.permalink)? {
                    anyhow::bail!(
                        "{} ya existe con este permalink bajo otro nombre: {} \
                         (índice rancio: corre `exo rebuild` o usa ese fichero directamente)",
                        args.permalink,
                        existente.display()
                    );
                }
                let proyecto = exo::nombre_kb()?;
                escribe_nueva(&NuevaNota {
                    kb: &kb,
                    proyecto: &proyecto,
                    dir,
                    titulo: slug_nota,
                    cuerpo: "",
                    tier: Some("log"),
                    dup_candidatas: &[],
                    forzado: false,
                })
                .context("crear la bitácora con --create")?;
                eprintln!("write: bitácora creada en {rel}");
            }
            rel
        }
        None => anyhow::bail!(
            "{} no está en el índice: comprueba el permalink, o usa --create si la bitácora aún no existe",
            args.permalink
        ),
    };
```

Añade el helper, justo antes de `write_append_cmd` (o después — cualquier
sitio de nivel de módulo en `main.rs` vale, clippy no exige orden):

```rust
/// Walk de confirmación del gate M4 #5 (Ola 1 G Task 7): busca, DENTRO de
/// `dir` (no recursivo — las bitácoras de `--create` viven a un nivel), un
/// `.md` cuyo frontmatter declare exactamente `permalink`. `None` si `dir`
/// no existe todavía (KB nueva) o si ningún fichero lo declara.
fn busca_permalink_en_dir(kb: &Path, dir: &str, permalink: &str) -> Result<Option<PathBuf>> {
    let carpeta = kb.join(dir);
    if !carpeta.is_dir() {
        return Ok(None);
    }
    for entrada in std::fs::read_dir(&carpeta)
        .with_context(|| format!("leer directorio {}", carpeta.display()))?
    {
        let entrada = entrada.with_context(|| format!("entrada de {}", carpeta.display()))?;
        let ruta = entrada.path();
        if !exo::walker::es_md(&entrada.file_name().to_string_lossy()) {
            continue;
        }
        if let Some(nota) = exo::nota::parsea_nota(&ruta)?
            && nota.permalink == permalink
        {
            return Ok(Some(ruta));
        }
    }
    Ok(None)
}
```

- [ ] **Step 4: Correr los tests y verlos pasar**

Run: `cd engine && cargo build --release --locked --bin exo && cargo test --release --locked --test write_create_permalink -- --nocapture`
Expected: los 3 tests del fichero (el existente
`write_append_create_usa_el_name_de_la_config_no_el_basename_del_dir_kb` +
los 2 nuevos) PASS.

- [ ] **Step 5: Correr toda la suite del write-path**

Run: `cd engine && cargo test --release --locked --test escritor --test rechazo_envelope --test write_create_permalink -- --nocapture`
Expected: todos verdes.

- [ ] **Step 6: Commit**

```bash
git add engine/src/main.rs engine/tests/write_create_permalink.rs
git commit -m "fix(g, write-create): permalink de menos de 3 segmentos es error accionable, y un walk de confirmación evita duplicar una bitácora con índice rancio"
```

---

### Task 8: Assert de dimensión y norma tras `embebe_batch`

**Lane:** mecánica. **Depende de:** nada.

**Files:**
- Modify: `engine/src/lib.rs`

**Interfaces:**
- Consumes: nada nuevo.
- Produces: `pub fn embebe_batch(&mut self, textos: &[String]) -> Result<Vec<Vec<f32>>>`
  — firma pública sin cambios; ahora puede devolver `Err` si un embedding
  sale con dimensión o norma inesperadas (antes nunca fallaba por esto).

- [ ] **Step 1: Test que falla — helper puro sin cargar el modelo**

Añade a `engine/src/lib.rs`, al final del fichero (o cerca del `impl
Embedder`):

```rust
#[cfg(test)]
mod tests_verifica_embedding {
    use super::verifica_embedding;

    #[test]
    fn rechaza_dimension_equivocada() {
        let v = vec![0.5_f32; 100];
        let err = verifica_embedding(&v, 0, 1).unwrap_err();
        assert!(err.to_string().contains("100 dims"), "{err}");
    }

    #[test]
    fn rechaza_norma_fuera_de_rango() {
        let mut v = vec![0.0_f32; 768];
        v[0] = 5.0; // norma 5.0, muy lejos de 1.0
        let err = verifica_embedding(&v, 0, 1).unwrap_err();
        assert!(err.to_string().contains("norma"), "{err}");
    }

    #[test]
    fn acepta_un_vector_unitario_real() {
        let mut v = vec![0.0_f32; 768];
        v[0] = 1.0; // norma exacta 1.0
        assert!(verifica_embedding(&v, 0, 1).is_ok());
    }
}
```

- [ ] **Step 2: Correr los tests y verlos fallar (no compila: `verifica_embedding`
  no existe)**

Run: `cd engine && cargo test --release --locked tests_verifica_embedding 2>&1 | head -20`
Expected: FAIL en compilación — `error[E0425]: cannot find function
'verifica_embedding' in this scope`.

- [ ] **Step 3: `verifica_embedding` + su uso en `embebe_batch`**

old_string:
```rust
    /// Embebe un batch de textos en una sola pasada (usado por el indexer
    /// para los trozos de una nota; también sirve para embeber la query de
    /// `exo search --type vector` como batch de 1).
    pub fn embebe_batch(&mut self, textos: &[String]) -> Result<Vec<Vec<f32>>> {
        self.te
            .embed(textos, None)
            .context("embed batch con fastembed")
    }
}
```

new_string:
```rust
    /// Embebe un batch de textos en una sola pasada (usado por el indexer
    /// para los trozos de una nota; también sirve para embeber la query de
    /// `exo search --type vector` como batch de 1). Cada vector se valida
    /// (Ola 1 G Task 8, `KB-exo:16`) antes de devolverse — la única guarda
    /// existente estaba en la LECTURA (`vectores::lee`, `BYTES_ESPERADOS`),
    /// nunca en la escritura: un embedding corrupto entraba al índice sin
    /// avisar y solo se notaba después, en un KNN con resultados raros.
    pub fn embebe_batch(&mut self, textos: &[String]) -> Result<Vec<Vec<f32>>> {
        let vectores = self
            .te
            .embed(textos, None)
            .context("embed batch con fastembed")?;
        let total = vectores.len();
        for (i, v) in vectores.iter().enumerate() {
            verifica_embedding(v, i, total)?;
        }
        Ok(vectores)
    }
}

/// Dimensión y norma esperadas de un embedding de jina-es: 768 componentes
/// (mismo contrato que `vectores::BYTES_ESPERADOS = 768 * 4`), norma ~1.0
/// porque fastembed normaliza siempre (ver doc de `Embedder::desde_config`).
/// Falla ALTO en vez de dejar pasar un vector corrupto al índice.
fn verifica_embedding(v: &[f32], idx: usize, total: usize) -> Result<()> {
    const DIMS_ESPERADAS: usize = 768;
    if v.len() != DIMS_ESPERADAS {
        anyhow::bail!(
            "embedding {idx}/{total} con {} dims, se esperaban {DIMS_ESPERADAS}",
            v.len()
        );
    }
    let norma: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if !(0.99..=1.01).contains(&norma) {
        anyhow::bail!(
            "embedding {idx}/{total} con norma {norma:.4}, se esperaba ~1.0 \
             (fastembed normaliza siempre: el modelo no está normalizando o produjo NaN/Inf)"
        );
    }
    Ok(())
}
```

- [ ] **Step 4: Correr los tests y verlos pasar**

Run: `cd engine && cargo test --release --locked tests_verifica_embedding -- --nocapture`
Expected: los 3 PASS.

- [ ] **Step 5: Correr la suite completa (`embebe_batch` lo usa el indexer y
  `search --type vector`)**

Run: `cd engine && cargo test --release --locked --test indexer --test buscador -- --nocapture`
Expected: todos verdes — los embeddings reales de fastembed siempre pasan
la validación (norma 1.0, 768 dims por contrato del modelo), así que ningún
test existente que use el embedder real puede empezar a fallar por esto.

- [ ] **Step 6: Commit**

```bash
git add engine/src/lib.rs
git commit -m "fix(g, lib): embebe_batch valida dimension y norma de cada vector, fail-loud ante un embedding corrupto (KB-exo:16)"
```

---

### Task 9: `trinquete --staged` lee el tier del índice de git, no del disco

**Lane:** mecánica. **Depende de:** nada.

**Checklist opcional, NO bloqueante** (Global Constraints: kbx ya no es
referencia): si Paul quiere confirmar que la divergencia con
`internal/ratchet/staged.go` de kbx (`fe46443`) era accidental y no
deliberada, puede correr `git -C ~/Documentos/proyectos/kbx show
fe46443:internal/ratchet/staged.go` en la máquina Linux y comparar. **No
hace falta para implementar ni para mergear esta task**: el propio código
Rust ya documenta el gap como heredado, no como decisión deliberada (ver
comentario citado abajo).

**Files:**
- Modify: `engine/src/trinquete.rs`

**Interfaces:**
- Consumes: `gitx::muestra(dir: &Path, objeto: &str) -> Result<Option<String>>`
  (ya existe).
- Produces: `sellos_escapados_de_tier` cambia de firma (privada, sin
  consumidores fuera de este fichero): pasa de
  `fn sellos_escapados_de_tier(kb: &Path, actual: &Sellos, declaradas: &[Declarada], presupuestos: Presupuestos) -> Vec<Hallazgo>`
  a `fn sellos_escapados_de_tier(actual: &Sellos, declaradas: &[Declarada], presupuestos: Presupuestos, contenido_de: impl Fn(&str) -> Option<String>) -> Vec<Hallazgo>`.

- [ ] **Step 1: Test que falla — staged usa el índice, no el disco**

Añade al `mod tests` de `engine/src/trinquete.rs`, después de
`escapados_ignora_los_sellos_con_declaracion`:

```rust
    #[test]
    fn escapados_en_staged_lee_el_tier_del_indice_no_del_disco() {
        // Task 9 (Ola 1 G, backlog:1086-1099): antes, `sellos_escapados_de_tier`
        // leía SIEMPRE del disco, también en `--staged` — un `git add` con
        // `tier: log` (sin presupuesto) seguido de un edit sin re-stage a
        // `tier: core` (con presupuesto) escapaba el gate `--staged` en
        // silencio: el índice dice `log`, el disco dice `core`, y el
        // trinquete miraba el disco.
        let dir = tempfile::tempdir().unwrap();
        let kb = dir.path();
        let cfg = kb.join("gitconfig-vacio");
        std::fs::write(&cfg, "").unwrap();
        let git = |args: &[&str]| {
            let st = std::process::Command::new("git")
                .arg("-C")
                .arg(kb)
                .args(args)
                .env("GIT_CONFIG_GLOBAL", &cfg)
                .env("GIT_CONFIG_SYSTEM", &cfg)
                .env("GIT_AUTHOR_NAME", "f")
                .env("GIT_AUTHOR_EMAIL", "f@k.local")
                .env("GIT_COMMITTER_NAME", "f")
                .env("GIT_COMMITTER_EMAIL", "f@k.local")
                .status()
                .unwrap();
            assert!(st.success(), "git {args:?}");
        };
        git(&["init", "-q"]);
        std::fs::write(kb.join("a.md"), "---\ntier: log\n---\nx\n").unwrap();
        git(&["add", "a.md"]);
        // Tras el `add`, se edita el DISCO a un tier con presupuesto — sin
        // volver a stagear.
        std::fs::write(kb.join("a.md"), "---\ntier: core\n---\nx\n").unwrap();

        let actual = sellos(&[("a.md", 9000)]);

        let h_indice = sellos_escapados_de_tier(&actual, &[], crate::presupuesto::NOMINALES, |ruta| {
            crate::gitx::muestra(kb, &format!(":./{ruta}")).ok().flatten()
        });
        assert_eq!(
            h_indice.len(),
            1,
            "el índice sigue diciendo tier: log (sin presupuesto): {h_indice:?}"
        );
        assert_eq!(h_indice[0].ruta, "a.md");

        let h_disco = sellos_escapados_de_tier(&actual, &[], crate::presupuesto::NOMINALES, |ruta| {
            std::fs::read_to_string(kb.join(ruta)).ok()
        });
        assert!(
            h_disco.is_empty(),
            "el disco dice tier: core (con presupuesto), no debe escapar: {h_disco:?}"
        );
    }
```

- [ ] **Step 2: Correr el test y verlo fallar (no compila: la firma actual
  toma `kb: &Path`, no un closure)**

Run: `cd engine && cargo test --release --locked escapados_en_staged_lee_el_tier_del_indice_no_del_disco 2>&1 | head -20`
Expected: FAIL en compilación — `error[E0308]` (tipos no coinciden: la
firma actual espera `&Path` en la primera posición, el test pasa `&actual`).

- [ ] **Step 3: Parametrizar `sellos_escapados_de_tier` por la fuente de
  contenido**

old_string:
```rust
/// Una nota sellada que ya no declara waiver y cuyo tier ACTUAL no tiene
/// presupuesto se reclasificó a `log` para escapar del gate: el sello es la
/// prueba de que tuvo techo. Solo mira los sellos SIN `Declarada` — con
/// `Declarada` ya pasó por `checks_de_declaracion`.
///
/// Lee el tier del **disco** también en `--staged`: es el comportamiento
/// heredado de antes de partir `comprueba_contra`, y este refactor no lo
/// cambia.
fn sellos_escapados_de_tier(
    kb: &Path,
    actual: &Sellos,
    declaradas: &[Declarada],
    presupuestos: crate::presupuesto::Presupuestos,
) -> Vec<Hallazgo> {
    let rutas_declaradas: BTreeSet<&str> = declaradas.iter().map(|d| d.ruta.as_str()).collect();
    let mut hallazgos = Vec::new();
    for (ruta, &sello) in actual {
        if rutas_declaradas.contains(ruta.as_str()) {
            continue;
        }
        let Ok(contenido) = std::fs::read_to_string(kb.join(ruta)) else {
            continue; // nota borrada: el sello huérfano se queda, nada que mirar.
        };
        let tier = crate::frontmatter::tier(&contenido);
        if !tier.is_empty() && presupuestos.para_tier(&tier).unwrap_or(0) <= 0 {
            hallazgos.push(Hallazgo {
                ruta: ruta.clone(),
                tipo: Tipo::SelladaEscapadaDeTier,
                era: sello,
                ahora: 0,
                limite: 0,
            });
        }
    }
    hallazgos
}
```

new_string:
```rust
/// Una nota sellada que ya no declara waiver y cuyo tier ACTUAL no tiene
/// presupuesto se reclasificó a `log` para escapar del gate: el sello es la
/// prueba de que tuvo techo. Solo mira los sellos SIN `Declarada` — con
/// `Declarada` ya pasó por `checks_de_declaracion`.
///
/// `contenido_de` decide DE DÓNDE sale el contenido de cada nota (Ola 1 G
/// Task 9, backlog:1086-1099): antes leía SIEMPRE del disco, también en
/// `--staged` — comportamiento heredado, no una decisión deliberada. Ahora
/// `comprueba_contra` pasa `contenido_de_disco` o `contenido_de_indice`
/// según el modo, igual que ya hacía con `tamano_de`.
fn sellos_escapados_de_tier(
    actual: &Sellos,
    declaradas: &[Declarada],
    presupuestos: crate::presupuesto::Presupuestos,
    contenido_de: impl Fn(&str) -> Option<String>,
) -> Vec<Hallazgo> {
    let rutas_declaradas: BTreeSet<&str> = declaradas.iter().map(|d| d.ruta.as_str()).collect();
    let mut hallazgos = Vec::new();
    for (ruta, &sello) in actual {
        if rutas_declaradas.contains(ruta.as_str()) {
            continue;
        }
        let Some(contenido) = contenido_de(ruta) else {
            continue; // nota borrada (o sin stage): el sello huérfano se queda.
        };
        let tier = crate::frontmatter::tier(&contenido);
        if !tier.is_empty() && presupuestos.para_tier(&tier).unwrap_or(0) <= 0 {
            hallazgos.push(Hallazgo {
                ruta: ruta.clone(),
                tipo: Tipo::SelladaEscapadaDeTier,
                era: sello,
                ahora: 0,
                limite: 0,
            });
        }
    }
    hallazgos
}

/// Contenido de una nota en el árbol de trabajo (Ola 1 G Task 9). `None` si
/// no se puede leer (nota borrada, permisos).
fn contenido_de_disco(kb: &Path, ruta: &str) -> Option<String> {
    std::fs::read_to_string(kb.join(ruta)).ok()
}

/// Contenido de una nota en el índice de git, stage 0 (Ola 1 G Task 9,
/// backlog:1086-1099): mismo idioma que `tamano_de_indice` (`gitx::muestra`,
/// objeto `:./<ruta>`). `None` si el objeto no está en el índice (fichero
/// no staged, staged como borrado).
fn contenido_de_indice(kb: &Path, ruta: &str) -> Option<String> {
    gitx::muestra(kb, &format!(":./{ruta}")).ok().flatten()
}
```

Actualiza la llamada dentro de `comprueba_contra`:

old_string:
```rust
    hallazgos.extend(sellos_escapados_de_tier(
        kb,
        &actual,
        declaradas,
        presupuestos,
    ));
```

new_string:
```rust
    hallazgos.extend(sellos_escapados_de_tier(
        &actual,
        declaradas,
        presupuestos,
        |ruta| lee_contenido(kb, ruta),
    ));
```

Y añade el parámetro `lee_contenido` a `comprueba_contra` y a sus dos
llamadores (`comprueba`/`comprueba_staged`), mismo patrón que `tamano_de`:

old_string:
```rust
fn comprueba_contra(
    kb: &Path,
    declaradas: &[Declarada],
    presupuestos: crate::presupuesto::Presupuestos,
    carga_actual: impl Fn(&Path) -> Result<Sellos>,
    tamano_de: impl Fn(&Path, &str) -> Option<i64>,
) -> Result<Informe> {
```

new_string:
```rust
fn comprueba_contra(
    kb: &Path,
    declaradas: &[Declarada],
    presupuestos: crate::presupuesto::Presupuestos,
    carga_actual: impl Fn(&Path) -> Result<Sellos>,
    tamano_de: impl Fn(&Path, &str) -> Option<i64>,
    lee_contenido: impl Fn(&Path, &str) -> Option<String>,
) -> Result<Informe> {
```

old_string:
```rust
pub fn comprueba(
    kb: &Path,
    declaradas: &[Declarada],
    presupuestos: crate::presupuesto::Presupuestos,
) -> Result<Informe> {
    comprueba_contra(kb, declaradas, presupuestos, carga, tamano_de_disco)
}
```

new_string:
```rust
pub fn comprueba(
    kb: &Path,
    declaradas: &[Declarada],
    presupuestos: crate::presupuesto::Presupuestos,
) -> Result<Informe> {
    comprueba_contra(kb, declaradas, presupuestos, carga, tamano_de_disco, contenido_de_disco)
}
```

old_string:
```rust
pub fn comprueba_staged(
    kb: &Path,
    declaradas: &[Declarada],
    presupuestos: crate::presupuesto::Presupuestos,
) -> Result<Informe> {
    comprueba_contra(kb, declaradas, presupuestos, carga_staged, tamano_de_indice)
}
```

new_string:
```rust
pub fn comprueba_staged(
    kb: &Path,
    declaradas: &[Declarada],
    presupuestos: crate::presupuesto::Presupuestos,
) -> Result<Informe> {
    comprueba_contra(kb, declaradas, presupuestos, carga_staged, tamano_de_indice, contenido_de_indice)
}
```

- [ ] **Step 4: Actualizar los 2 tests unitarios existentes que llamaban a
  la firma vieja**

old_string:
```rust
        let h = sellos_escapados_de_tier(kb.path(), &actual, &[], crate::presupuesto::NOMINALES);
        assert_eq!(
            h.len(),
            1,
            "solo a.md: b.md tiene presupuesto, borrada.md no se lee"
        );
```

new_string:
```rust
        let h = sellos_escapados_de_tier(&actual, &[], crate::presupuesto::NOMINALES, |ruta| {
            std::fs::read_to_string(kb.path().join(ruta)).ok()
        });
        assert_eq!(
            h.len(),
            1,
            "solo a.md: b.md tiene presupuesto, borrada.md no se lee"
        );
```

old_string:
```rust
        let h = sellos_escapados_de_tier(kb.path(), &actual, &decl, crate::presupuesto::NOMINALES);
        assert!(h.is_empty());
```

new_string:
```rust
        let h = sellos_escapados_de_tier(&actual, &decl, crate::presupuesto::NOMINALES, |ruta| {
            std::fs::read_to_string(kb.path().join(ruta)).ok()
        });
        assert!(h.is_empty());
```

- [ ] **Step 5: Correr los tests y verlos pasar**

Run: `cd engine && cargo test --release --locked --lib trinquete:: -- --nocapture`
Expected: TODOS verdes, incluido
`escapados_en_staged_lee_el_tier_del_indice_no_del_disco`.

- [ ] **Step 6: Correr la suite CLI de trinquete**

Run: `cd engine && cargo test --release --locked --test ratchet_cli -- --nocapture`
Expected: todos verdes (el comportamiento del árbol de trabajo,
`comprueba`, no cambia — solo `comprueba_staged`).

- [ ] **Step 7: Commit**

```bash
git add engine/src/trinquete.rs
git commit -m "fix(g, trinquete): --staged lee el tier de una nota escapada desde el indice de git, no del disco"
```

---

### Task 10: `kb-demo` → `kb-test` en los 11 ficheros de test + comentarios de `src`

**Lane:** mecánica. **Depende de:** Task 6 (toca `engine/tests/escritor.rs`
en las mismas líneas — ejecutar DESPUÉS de la Task 6, sobre el fichero ya
convertido a struct literal).

**Excepciones explícitas, NO tocar** (verificadas antes de escribir esta
task):
- `engine/tests/help_producto.rs:154` — `(r"kb-demo", "nombre de la KB del
  autor")` es un patrón del gate anti-jerga que comprueba que `--help`
  JAMÁS mencione el nombre real de la KB de Paul. Es la razón de ser de esa
  línea, no una convención de fixture: renombrarla apagaría el gate.
- `engine/tests/escritor.rs:29` — `// Pares extraídos de
  /home/paul/Documentos/proyectos/kb-demo.` documenta de dónde salieron los
  pares `title`/`permalink` reales de `slug_replica_los_permalinks_reales_de_la_kb`.
  Es procedencia histórica, no un placeholder: los datos vinieron de esa
  KB con ese nombre.
- `engine/tests/recall_contenido.rs:108` — comentario que documenta una
  medición real (`` `kb-demo/`, `kb/` `` como ejemplos de prefijos que
  costaban bytes). Misma razón que arriba.
- `engine/src/lib.rs:103` — `Antes leía \`projects["kb-demo"].path\`` es un
  hecho histórico sobre qué literal leía una versión anterior del código
  (el config.json real de basic-memory). No se toca.

**Files:**
- Modify: `engine/tests/{buscador,config,doctor,doctor_cli,escritor,help_producto,indexer,inicia,nota,recall,recall_contenido}.rs`
- Modify: `engine/src/{buscador,inicia}.rs`

**Interfaces:**
- Consumes/Produces: ninguna — rename de literales, cero cambio de
  comportamiento ni de firma.

- [ ] **Step 1: Medir el estado ANTES**

Run: `cd engine && git grep -c kb-demo -- . | grep -v ':0'`
Expected: una lista con las 11 rutas de `tests/` + `src/buscador.rs`,
`src/inicia.rs`, `src/lib.rs` (14 ficheros en total, contando comentarios).

- [ ] **Step 2: Replace global en los 13 ficheros que SÍ cambian**

Run (Git Bash / sed GNU; `sed -i` en Windows con Git for Windows soporta
`-i` sin sufijo tal cual):
```bash
cd engine
sed -i 's/kb-demo/kb-test/g' \
  tests/buscador.rs tests/config.rs tests/doctor.rs tests/doctor_cli.rs \
  tests/escritor.rs tests/help_producto.rs tests/indexer.rs tests/inicia.rs \
  tests/nota.rs tests/recall.rs tests/recall_contenido.rs \
  src/buscador.rs src/inicia.rs
```

- [ ] **Step 3: Revertir las 3 excepciones que el `sed` global sí tocó**

`tests/help_producto.rs` (revierte la línea del gate anti-jerga):

old_string:
```rust
    (r"kb-test", "nombre de la KB del autor"),
```

new_string:
```rust
    (r"kb-demo", "nombre de la KB del autor"),
```

`tests/escritor.rs` (revierte el comentario de procedencia):

old_string:
```rust
    // Pares extraídos de /home/paul/Documentos/proyectos/kb-test.
```

new_string:
```rust
    // Pares extraídos de /home/paul/Documentos/proyectos/kb-demo.
```

`tests/recall_contenido.rs` (revierte el comentario de medición real):

old_string:
```rust
/// (`kb-test/`, `kb/`) delante de CADA línea: sobre la KB real son 12
```

new_string:
```rust
/// (`kb-demo/`, `kb/`) delante de CADA línea: sobre la KB real son 12
```

(`src/lib.rs` no entró en el Step 2 — no está en la lista de `sed` —, así
que no hace falta revertir nada ahí.)

- [ ] **Step 4: Verificar el estado DESPUÉS**

Run: `cd engine && git grep -c kb-demo -- . | grep -v ':0'`
Expected: exactamente 3 líneas:
```
src/lib.rs:1
tests/escritor.rs:1
tests/help_producto.rs:1
tests/recall_contenido.rs:1
```
(4 ficheros, 1 ocurrencia cada uno — las 4 excepciones documentadas arriba;
`src/inicia.rs` puede seguir apareciendo si el `sed` no tocó alguna
variante con mayúsculas — revisa manualmente si la lista no coincide
exactamente).

- [ ] **Step 5: Compilar y correr toda la suite**

Run: `cd engine && cargo build --release --locked --bin exo --example kb_sintetica`
Expected: exit 0 (el rename es de literales de test, no de nada que
`src/main.rs`, `src/lib.rs` en producción consuma por valor).

Run: `cd engine && cargo test --release --locked -- --nocapture 2>&1 | tail -60`
Expected: sin regresiones — la suite entera verde (correr en background,
es la suite completa del engine).

- [ ] **Step 6: Commit**

```bash
git add engine/tests/buscador.rs engine/tests/config.rs engine/tests/doctor.rs \
  engine/tests/doctor_cli.rs engine/tests/escritor.rs engine/tests/help_producto.rs \
  engine/tests/indexer.rs engine/tests/inicia.rs engine/tests/nota.rs \
  engine/tests/recall.rs engine/tests/recall_contenido.rs \
  engine/src/buscador.rs engine/src/inicia.rs
git commit -m "test(g, kb-test): kb-demo -> kb-test como fixture generico en los tests del engine (backlog:1310-1322)"
```

---

### Task 11: D6 — default de `exo search --type` pasa a `hybrid` + `--min-similarity 0.40`, y `exo init` alinea su propio default a la misma constante

**Lane:** mecánica, superficie (decisión 1 de Paul, ya tomada; ampliada el
2026-09-15: el default que `exo init` escribe en `[embeddings]
min_similarity` sube de 0.35 a **0.40**, usando la MISMA constante que sella
el default de `--type hybrid`, no un segundo literal — ver Step 5).
**Depende de:** ninguna task de este plan; **requiere que la ola haya
mergeado H y F primero** (zona compartida de `main.rs`, ver Global
Constraints).

**Files:**
- Modify: `engine/src/main.rs:11-26` (comentario + constantes), `:241`
  (`ArgsSearch`), `:651-657` (`init_cmd`, default de `[embeddings]
  min_similarity`), `:982-1009` (`busca_cmd`)
- Modify: `engine/src/buscador.rs:238-241` (comentario, "hoy 0.35" pasa a
  "hoy 0.40" — no toca código de la Task 2, ninguna línea en común)
- Modify: `engine/src/escritor.rs:162-168` (comentario, el rango "0.35-0.40"
  citado como calibración del dup-gate pasa a reflejar que ambos extremos
  son ya la misma constante)
- Modify: `engine/examples/kb_sintetica.rs:176-184` (`escribe_config`: el
  config sintético escrito a mano pasa su literal de 0.35 a 0.40 — ningún
  comando de `bench.sh` depende de este valor, porque todos pasan
  `--min-similarity` explícito (Task 1), pero el fichero deja de describir
  un `exo init` que ya no existe)
- Modify: `docs/arquitectura.md` (§3.4, §3.5 y §3.8 — cuatro citas del
  0.35/`fts` como default, incluida la tabla de comandos)
- Test: `engine/tests/flags.rs`, `engine/tests/help_producto.rs`,
  `engine/tests/inicia.rs`

**Interfaces:**
- Consumes: `TipoBusqueda` (enum ya existente, variantes `Fts`/`Vector`/
  `Hybrid`), `busca_hybrid` (Task 2, firma sin cambios),
  `exo::config::Embeddings`/`exo::config::carga_desde` (ya existentes, sin
  cambio de firma).
- Produces: nueva constante `MIN_SIMILARITY_SELLADO: f64 = 0.40` en
  `main.rs`, junto a `BONUS_SELLADO`/`ESCALA_FTS_SELLADA` — **fuente única**
  para dos consumidores: el default de `exo search --type hybrid` (Step 5) Y
  el default que `exo init` escribe en `[embeddings] min_similarity` de una
  config nueva (Step 5, mismo cuerpo). Antes de esta task existían dos
  literales sin relación en el código (0.40 solo documentado en comentario;
  0.35 hardcodeado en `init_cmd`) que coincidían con el sweep y con la
  config heredada de basic-memory por pura casualidad histórica, nunca por
  una fuente común.

- [ ] **Step 1: Test que falla — el default es `hybrid`**

Añade a `engine/tests/flags.rs`:

```rust
#[test]
fn el_default_de_search_type_es_hybrid_no_fts() {
    // D6 (decisión 1 de Paul, 2026-09-15): el held-out ya decidió — A0
    // (hybrid) gana a FTS en 41/55 filas, FTS a A0 en 0
    // (evals/retrieval-heldout/verdict/c-verdict.md). Este test comprueba
    // el `--help`: el default declarado por clap aparece ahí literal.
    let help_search = help_de(&["search", "--help"]);
    assert!(
        help_search.contains("[default: hybrid]"),
        "el --help de search debe declarar default hybrid, no fts:\n{help_search}"
    );
}
```

(Usa el helper `help_de` ya definido en `flags.rs` — el mismo que usan los
tests de aliases de ese fichero.)

- [ ] **Step 2: Correr el test y verlo fallar**

Run: `cd engine && cargo build --release --locked --bin exo && cargo test --release --locked el_default_de_search_type_es_hybrid_no_fts -- --nocapture`
Expected: FAIL — `--help` muestra `[default: fts]`.

- [ ] **Step 3: Test que falla — `exo init` (modo creación) sigue
  escribiendo `min_similarity = 0.35`**

Añade al final de `engine/tests/inicia.rs` (después de
`valida_db_para_kb_rechaza_otra_kb_sin_mencionar_un_flag_que_init_no_tiene`,
el último test del fichero):

old_string:
```rust
    assert!(
        msg.contains("otra KB"),
        "sigue siendo el mismo guard: {msg}"
    );
}
```

new_string:
```rust
    assert!(
        msg.contains("otra KB"),
        "sigue siendo el mismo guard: {msg}"
    );
}

/// D6 ampliado (decisión de Paul, 2026-09-15, Ola 1 G Task 11): el default
/// que `exo init` escribe en `[embeddings] min_similarity` sube de 0.35 a
/// 0.40 — la MISMA constante `MIN_SIMILARITY_SELLADO` que ya sella el
/// default de `exo search --type hybrid` (`flags.rs`,
/// `el_default_de_search_type_es_hybrid_no_fts`), no un segundo literal que
/// solo coincidía con el primero por casualidad. Cubre solo el modo
/// CREACIÓN (`init_cmd`, rama `else` sin `--from-basic-memory`) —
/// `--from-basic-memory` sigue leyendo `semantic_min_similarity` del JSON
/// de origen tal cual, sin cambios (ver
/// `migra_desde_basic_memory_leyendo_el_proyecto_por_defecto` arriba, que
/// sigue fijando 0.35 en su fixture a propósito: es el valor que trae ESA
/// KB de origen, no un default de `exo init`).
#[test]
fn init_en_modo_creacion_escribe_min_similarity_0_40_por_defecto() {
    let tmp = tempfile::TempDir::new().unwrap();
    let kb = tmp.path().join("kb-nueva");
    let config = tmp.path().join("config.toml");
    let db = tmp.path().join("index.db");

    let salida = std::process::Command::new(env!("CARGO_BIN_EXE_exo"))
        .args(["init", "--kb"])
        .arg(&kb)
        .args(["--name", "kb-nueva", "--json"])
        .env("EXO_CONFIG", &config)
        .env("EXO_DB", &db)
        .output()
        .expect("ejecutar exo init");
    assert!(
        salida.status.success(),
        "init falló: {}",
        String::from_utf8_lossy(&salida.stderr)
    );

    let cfg = exo::config::carga_desde(&config)
        .expect("releer la config que init acaba de escribir");
    assert_eq!(
        cfg.embeddings.min_similarity, 0.40,
        "exo init en modo creación debe escribir min_similarity = 0.40, no \
         0.35 — MIN_SIMILARITY_SELLADO en main.rs"
    );
}
```

- [ ] **Step 4: Correr el test y verlo fallar**

Run: `cd engine && cargo build --release --locked --bin exo && cargo test --release --locked init_en_modo_creacion_escribe_min_similarity_0_40_por_defecto -- --nocapture`
Expected: FAIL — `` assertion `left == right` failed: exo init en modo
creación debe escribir min_similarity = 0.40, no 0.35 — MIN_SIMILARITY_SELLADO
en main.rs
  left: 0.35
 right: 0.4 ``.

- [ ] **Step 5: Cambiar el default en `ArgsSearch`, sellar el umbral del
  camino Hybrid como constante, y alinear `exo init` a esa misma constante**

Reescribe el comentario de cabecera de los defaults sellados — el motivo por
el que el threshold NO se sellaba ("config es RO hasta M5a") caducó cuando
M5a-02 cerró:

old_string:
```rust
/// Defaults SELLADOS del arm hybrid (M2-07, §5.2.6 de la spec de fusión):
/// ganadores del sweep 15+1 corridas (grid bonus{0,0.1,0.2,0.3,0.5}×
/// β{0.6,0.8,1.0} + diagnóstica A, `evals/e1-read/reports/m2-07-impl-report.md`) —
/// selección pre-registrada §5.2.4 (max hit@5=49/55 → 4 celdas empatadas en
/// β=0.6 → menor bonus=0.0), confirmada nativa (§5.2.5, `--min-similarity
/// 0.40` da 49/55 idéntico al post-hoc). Cubren SOLO el uso de `exo search
/// --type hybrid` sin `--bonus`/`--fts-scale` explícitos; el sweep siempre
/// pasó ambos flags, así que estos valores no afectaron su resultado. El
/// threshold ganador (0.40) NO se sella aquí como constante — D-f3/§4.6: el
/// valor difiere del 0.35 de config y config es RO hasta M5a, así que se
/// pasa por `--min-similarity 0.40` explícito en corridas/consumidores hasta
/// entonces (documentado en el verdict, no hardcodeado en el binario).
```

new_string:
```rust
/// Defaults SELLADOS del arm hybrid (M2-07, §5.2.6 de la spec de fusión):
/// ganadores del sweep 15+1 corridas (grid bonus{0,0.1,0.2,0.3,0.5}×
/// β{0.6,0.8,1.0} + diagnóstica A, `evals/e1-read/reports/m2-07-impl-report.md`) —
/// selección pre-registrada §5.2.4 (max hit@5=49/55 → 4 celdas empatadas en
/// β=0.6 → menor bonus=0.0), confirmada nativa (§5.2.5, `--min-similarity
/// 0.40` da 49/55 idéntico al post-hoc). Cubren SOLO el uso de `exo search
/// --type hybrid` sin `--bonus`/`--fts-scale` explícitos; el sweep siempre
/// pasó ambos flags, así que estos valores no afectaron su resultado. El
/// threshold ganador (0.40) SÍ se sella aquí como constante desde el
/// 2026-09-15 (D6, decisión 1 de Paul, Ola 1 G Task 11): el motivo original
/// para no sellarlo — D-f3/§4.6, "el valor difiere del 0.35 de config y
/// config es RO hasta M5a" — caducó cuando M5a-02 (config propia) cerró el
/// 2026-08-26 (`docs/backlog.md:1945`, "M5a-02 config propia: cerrado el
/// 2026-08-26"). Ver `MIN_SIMILARITY_SELLADO` más abajo.
```

Añade la constante nueva junto a las dos existentes:

old_string:
```rust
const BONUS_SELLADO: f64 = 0.0;
const ESCALA_FTS_SELLADA: f64 = 0.6;
```

new_string:
```rust
const BONUS_SELLADO: f64 = 0.0;
const ESCALA_FTS_SELLADA: f64 = 0.6;
/// D6 (decisión 1 de Paul, 2026-09-15): umbral de similitud coseno para el
/// default nuevo de `exo search --type` (hybrid) — y, desde el mismo día,
/// el default que `exo init` escribe en `[embeddings] min_similarity` de
/// una config nueva (`init_cmd`, rama de creación, más abajo): una sola
/// constante para los dos usos en vez de dos literales que antes solo
/// coincidían en intención, nunca en código. En el camino de search se usa
/// cuando `--min-similarity` se omite Y `--type` resolvió a Hybrid — un
/// `--type vector` explícito sigue cayendo a `[embeddings] min_similarity`
/// de la config (comportamiento sin cambios, `min_similitud_efectivo` en
/// `buscador.rs`). Valor validado por el held-out de la campaña C
/// (`evals/retrieval-heldout/verdict/c-verdict.md`).
const MIN_SIMILARITY_SELLADO: f64 = 0.40;
```

Cambia el default del tipo de búsqueda:

old_string:
```rust
    /// Tipo de búsqueda.
    #[arg(long, value_enum, default_value_t = TipoBusqueda::Fts)]
    r#type: TipoBusqueda,
```

new_string:
```rust
    /// Tipo de búsqueda.
    #[arg(long, value_enum, default_value_t = TipoBusqueda::Hybrid)]
    r#type: TipoBusqueda,
```

Y usa el nuevo umbral sellado en la rama Hybrid de `busca_cmd`:

old_string:
```rust
        TipoBusqueda::Hybrid => busca_hybrid(
            &db,
            &args.query,
            args.limite,
            args.min_similitud,
            args.bonus.unwrap_or(BONUS_SELLADO),
            args.escala_fts.unwrap_or(ESCALA_FTS_SELLADA),
            kb.as_deref(),
        )?,
```

new_string:
```rust
        TipoBusqueda::Hybrid => busca_hybrid(
            &db,
            &args.query,
            args.limite,
            Some(args.min_similitud.unwrap_or(MIN_SIMILARITY_SELLADO)),
            args.bonus.unwrap_or(BONUS_SELLADO),
            args.escala_fts.unwrap_or(ESCALA_FTS_SELLADA),
            kb.as_deref(),
        )?,
```

Y alinea el default de `exo init` (modo creación) a la misma constante:

old_string:
```rust
        let emb = exo::config::Embeddings {
            model: exo::MODELO_JINA_ES.to_string(),
            // 768 es la dimensionalidad DE ESTE modelo (MODELO_JINA_ES): si
            // se cambia uno, el otro tiene que cambiar con él.
            dims: 768,
            min_similarity: 0.35,
        };
```

new_string:
```rust
        let emb = exo::config::Embeddings {
            model: exo::MODELO_JINA_ES.to_string(),
            // 768 es la dimensionalidad DE ESTE modelo (MODELO_JINA_ES): si
            // se cambia uno, el otro tiene que cambiar con él.
            dims: 768,
            // D6 ampliado (decisión de Paul, 2026-09-15, Ola 1 G Task 11):
            // MISMA constante que sella el default de `exo search --type
            // hybrid` (cabecera de este fichero) — antes era un literal
            // 0.35 propio, sin relación con el sweep de calibración ni con
            // el umbral que el propio `exo search` usa por defecto.
            min_similarity: MIN_SIMILARITY_SELLADO,
        };
```

- [ ] **Step 6: Correr los dos tests y verlos pasar**

Run: `cd engine && cargo build --release --locked --bin exo && cargo test --release --locked el_default_de_search_type_es_hybrid_no_fts -- --nocapture`
Expected: PASS.

Run: `cd engine && cargo test --release --locked init_en_modo_creacion_escribe_min_similarity_0_40_por_defecto -- --nocapture`
Expected: PASS.

- [ ] **Step 7: Correr la suite de flags, ayuda e inicia al completo**

Run: `cd engine && cargo test --release --locked --test flags --test help_producto --test inicia -- --nocapture`
Expected: todos verdes. `los_flags_espanoles_siguen_parseando_como_alias` y
`los_flags_ya_ingleses_no_se_han_movido` no dependen del default de
`--type`, solo de qué flags existen — sin cambios ahí (H retira los
aliases españoles en su propia rama, disjunta de esta).
`migra_desde_basic_memory_leyendo_el_proyecto_por_defecto` y las pruebas de
`escribe_config`/`carga_desde` en `inicia.rs` siguen usando 0.35 como valor
de FIXTURE arbitrario (prueban round-trip de cualquier valor, no un
default) — no se tocan y siguen en verde sin cambios.

- [ ] **Step 8: Verificar los consumidores del plugin — ninguno depende de
  `fts` implícito**

Run: `cd . && grep -rn "exo search" plugins/exo/ engine/tests/ 2>/dev/null | grep -v -- "--type"`
Expected: cero líneas que invoquen `exo search` sin `--type` explícito
salvo los propios ficheros de test de `search`/`buscador_cli.rs` y
`search_no_results_cli.rs` — verificado manualmente en el diseño de esta
task: esos dos ficheros solo comprueban presencia de claves del envelope
(nunca el valor de `search_type`) y su DB de fixture no tiene filas en
`vectores`, así que el arm vector del nuevo default `hybrid` sale vacío sin
tocar el embedder ni la config — comportamiento observable idéntico al de
`fts` puro para esos tests. `plugins/exo/agents/executor.md`,
`plugins/exo/skills/document/SKILL.md` y `plugins/exo/scripts/exo-recall.sh`
ya llaman `exo search --type hybrid` explícito.

Run: `cd engine && cargo test --release --locked --test buscador_cli --test search_no_results_cli -- --nocapture`
Expected: todos verdes SIN modificar esos dos ficheros (confirma la
verificación del párrafo anterior).

- [ ] **Step 9: Actualizar comentarios y `docs/arquitectura.md` que citaban
  0.35 como default — ahora coincide con `MIN_SIMILARITY_SELLADO`**

`engine/src/buscador.rs` (comentario de `similitud_desde_l2_cuadrado`, cita
el default vigente):

old_string:
```rust
/// dos vectores unitarios, `||a-b||² = 2 - 2·cos(a,b)`, luego
/// `cos(a,b) = 1 - ||a-b||²/2` — la conversión que usa esta función para
/// comparar contra `[embeddings] min_similarity` (threshold pensado en escala
/// coseno, config propia de `~/.exo/config.toml`, hoy 0.35).
```

new_string:
```rust
/// dos vectores unitarios, `||a-b||² = 2 - 2·cos(a,b)`, luego
/// `cos(a,b) = 1 - ||a-b||²/2` — la conversión que usa esta función para
/// comparar contra `[embeddings] min_similarity` (threshold pensado en escala
/// coseno, config propia de `~/.exo/config.toml`, hoy 0.40 por defecto desde
/// D6 — `MIN_SIMILARITY_SELLADO` en `main.rs`, Ola 1 G Task 11).
```

`engine/src/escritor.rs` (comentario de `solape_slug`, el rango citado
como calibración del dup-gate — los dos extremos son ya la misma
constante):

old_string:
```rust
/// Solape de tokens entre dos slugs. **Deliberadamente NO usa retrieval
/// semántico**: el umbral de `busca_hybrid` (0.35-0.40) está calibrado para
/// "tráeme contexto relevante", que es otra pregunta que "esto ya existe" —
/// usarlo como dup-gate produce falsos rojos (verificado: un título sin
/// relación alguna puntuaba 0.36 contra una bitácora cualquiera). Un guard que
/// rebota al cierre de sesión acaba desactivado, que es como murió el primer
/// guard de kbx (spec M4 §7.3).
```

new_string:
```rust
/// Solape de tokens entre dos slugs. **Deliberadamente NO usa retrieval
/// semántico**: el umbral de `busca_hybrid` (0.40 desde D6, Ola 1 G Task 11
/// — antes 0.35-0.40 según viniera de config o del sellado del sweep, hoy
/// los dos extremos son la misma constante `MIN_SIMILARITY_SELLADO`) está
/// calibrado para "tráeme contexto relevante", que es otra pregunta que
/// "esto ya existe" — usarlo como dup-gate produce falsos rojos (verificado:
/// un título sin relación alguna puntuaba 0.36 contra una bitácora
/// cualquiera). Un guard que rebota al cierre de sesión acaba desactivado,
/// que es como murió el primer guard de kbx (spec M4 §7.3).
```

`engine/examples/kb_sintetica.rs` (`escribe_config`, el config sintético
escrito a mano deja de describir un `exo init` que ya no existe):

old_string:
```rust
            "schema_version = 1\n\n[kb]\npath = \"{}\"\nname = \"{NOMBRE_KB}\"\n\n\
             [index]\ndb = \"{}\"\n\n[embeddings]\nmodel = \"{}\"\ndims = 768\n\
             min_similarity = 0.35\n",
```

new_string:
```rust
            "schema_version = 1\n\n[kb]\npath = \"{}\"\nname = \"{NOMBRE_KB}\"\n\n\
             [index]\ndb = \"{}\"\n\n[embeddings]\nmodel = \"{}\"\ndims = 768\n\
             min_similarity = 0.40\n",
```

`docs/arquitectura.md` §3.4 (umbral en escala coseno):

old_string:
```
`vectores` usa la métrica por defecto de vec0 (L2 al cuadrado), y para
vectores unitarios `cos = 1 − L2²/2` — así el umbral `min_similarity` de la
config (0.35 por defecto) se compara en escala coseno.
```

new_string:
```
`vectores` usa la métrica por defecto de vec0 (L2 al cuadrado), y para
vectores unitarios `cos = 1 − L2²/2` — así el umbral `min_similarity` de la
config (0.40 por defecto desde el 2026-09-15, D6 — antes 0.35, ver §3.5) se
compara en escala coseno.
```

`docs/arquitectura.md` §3.5 (párrafo de apertura del pipeline de búsqueda):

old_string:
```
### 3.5 Pipeline de búsqueda

`exo search` tiene tres modos (`--type fts|vector|hybrid`, default `fts`),
implementados en `engine/src/buscador.rs`. Todos devuelven resultados
**a nivel de nota** (`type: "entity"`), nunca de trozo. Ojo con el default:
el modo calibrado y medido (48/55 hit@5, §6) es `--type hybrid` **con el
umbral pasado explícito** (`--min-similarity 0.40`); `fts` a secas es el modo
léxico barato, no el medido. `exo recall --query` sí usa hybrid con los
parámetros sellados de serie.
```

new_string:
```
### 3.5 Pipeline de búsqueda

`exo search` tiene tres modos (`--type fts|vector|hybrid`, **default
`hybrid` desde el 2026-09-15** — D6, decisión 1 de Paul, campaña G Task 11;
antes `fts`), implementados en `engine/src/buscador.rs`. Todos devuelven
resultados **a nivel de nota** (`type: "entity"`), nunca de trozo. El modo
calibrado y medido (48/55 hit@5 in-sample, §6; held-out 41/55 gana a FTS,
FTS a él en 0, `evals/retrieval-heldout/verdict/c-verdict.md`) es ahora
justo el default: hybrid con `min_similarity = MIN_SIMILARITY_SELLADO =
0.40` cuando no se pasa `--min-similarity` explícito — ya no hace falta
pasarlo a mano. `fts` a secas sigue disponible con `--type fts`, es el modo
léxico barato, no el medido. `exo recall --query` usa hybrid con los mismos
parámetros sellados de serie.
```

`docs/arquitectura.md` §3.5 (bullet del modo hybrid):

old_string:
```
- **hybrid**: los dos canales fusionados por unión. Los parámetros de fusión
  van **sellados** en `main.rs` tras el sweep de calibración de M2-07:
  `bonus = 0.0` y `β = 0.6` (`BONUS_SELLADO`, `ESCALA_FTS_SELLADA`),
  sobreescribibles con `--bonus`/`--fts-scale`. El umbral ganador del sweep
  (0.40) **no** está sellado como constante: difiere del 0.35 de config y los
  consumidores lo pasan explícito con `--min-similarity 0.40` (así lo hace el
  hook `recall-inject.sh`).
```

new_string:
```
- **hybrid**: los dos canales fusionados por unión. Los parámetros de fusión
  van **sellados** en `main.rs` tras el sweep de calibración de M2-07:
  `bonus = 0.0` y `β = 0.6` (`BONUS_SELLADO`, `ESCALA_FTS_SELLADA`),
  sobreescribibles con `--bonus`/`--fts-scale`. El umbral ganador del sweep
  (0.40) **sí** está sellado como constante desde el 2026-09-15
  (`MIN_SIMILARITY_SELLADO`, D6 — decisión 1 de Paul, campaña G Task 11): el
  motivo original para no sellarlo — D-f3/§4.6, "difiere del 0.35 de config
  y config es RO hasta M5a" — caducó cuando M5a-02 (config propia) cerró el
  2026-08-26 (`docs/backlog.md:1945`). `exo init` escribe ahora ese mismo
  0.40 como default de `[embeddings] min_similarity` en una config nueva —
  una sola constante, no dos literales que antes solo coincidían por
  casualidad. El hook `recall-inject.sh` sigue pasando `--min-similarity
  0.40` explícito (documenta su propio contrato de todos modos, no depende
  del default).
```

`docs/arquitectura.md` §3.8 (tabla de comandos, fila de `exo search`):

old_string:
```
| `exo search <query>` | Búsqueda FTS / vector / hybrid | `--type` (default `fts`), `--limit` (10), `--min-similarity`, `--bonus`, `--fts-scale`, `--db`, `--kb`, `--json` |
```

new_string:
```
| `exo search <query>` | Búsqueda FTS / vector / hybrid | `--type` (default `hybrid` desde D6, 2026-09-15), `--limit` (10), `--min-similarity` (default 0.40, `MIN_SIMILARITY_SELLADO`), `--bonus`, `--fts-scale`, `--db`, `--kb`, `--json` |
```

Fuera de alcance, NO tocar (verificado antes de escribir este Step):
`.superpowers/fabrica/config.md:148`, `docs/superpowers/runbooks/2026-08-24-integracion-equipo-trabajo-windows.md:47`
y todos los `evals/**/*.md`/`evals/**/*.py` que citan 0.35 — son registros
históricos de mediciones y runbooks fechados (el umbral vigente en el
momento de esa medición, o un ejemplo de `~/.basic-memory/config.json` de
antes de M5a-02); reescribirlos sería falsificar un registro, no corregir
un default. `engine/src/buscador.rs:384` (comentario de `K_FACTOR_INICIAL`,
"a 95k trozos, `limite=10` con `sim≥0.35`...") es la misma clase: documenta
la calibración medida por H29 en su momento, no el default actual — no se
toca por la misma razón que las excepciones históricas de `kb-demo` en la
Task 10.

- [ ] **Step 10: Commit**

```bash
git add engine/src/main.rs engine/src/buscador.rs engine/src/escritor.rs \
  engine/examples/kb_sintetica.rs engine/tests/flags.rs engine/tests/inicia.rs \
  docs/arquitectura.md
git commit -m "feat(g, search-default): D6 - exo search sin --type usa hybrid + min-similarity 0.40, exo init alinea su default a la misma constante (decision de Paul 2026-09-15)"
```

---

### Task 12: Sync de `docs/backlog.md`

**Lane:** mecánica. **Depende de:** Tasks 1-11 (todas — cita sus commits).
Va **la última**. **Oráculo:** los `grep` de este Step.

**Files:**
- Modify: `docs/backlog.md`

- [ ] **Step 1: Cabecera — nuevo párrafo `> Última revisión`**

old_string (la primera línea del bloque de cabecera — cópiala verbatim del
fichero real al ejecutar, esta cita es de `origin/main` al momento de
escribir este plan):
```
> Última revisión: **2026-09-15** (bookkeeping de estado: entran por PR las
```

new_string:
```
> Última revisión: **<fecha de ejecución>** (campaña G — deuda diferida del
> engine sin cambio de ranking, `docs/superpowers/plans/2026-09-15-campana-g-engine-deuda-diferida.md`,
> mergeada vía <PR de G>. Cierra con evidencia: dos patas vivas de «Techos
> de escala» — tres conexiones de DB en `busca_hybrid` (Task 2, commit
> `<commit Task 2>`) y un `git log` por nota indexada (Task 3, commit
> `<commit Task 3>`), con el bench sintético corregido primero para medirlas
> de verdad (Task 1, commit `<commit Task 1>`) — `walk_kb` unificada sobre
> `walk_kb_excluyendo` (Task 4, commit `<commit Task 4>`, decisión 9 de
> Paul), los dos límites de `budget_prose_drift` que quedan (Task 5, commit
> `<commit Task 5>`), el `allow(too_many_arguments)` de `escribe_nueva`
> (Task 6, commit `<commit Task 6>`), M4 #5 y #6 del gate M4 (Task 7, commit
> `<commit Task 7>`), el assert de dimensión/norma tras `embebe_batch`
> (Task 8, commit `<commit Task 8>`, `KB-exo:16`), `trinquete --staged`
> leyendo el tier del disco (Task 9, commit `<commit Task 9>`),
> `kb-demo`→`kb-test` en los tests del engine (Task 10, commit
> `<commit Task 10>`) y D6 — default `hybrid` + `--min-similarity 0.40`,
> con `exo init` alineado a la misma constante `MIN_SIMILARITY_SELLADO`
> para su propio default de `[embeddings] min_similarity` (Task 11, commit
> `<commit Task 11>`, decisión 1 de Paul). **Corrige con
> evidencia, sin cerrar por completo**: el conteo de `kb-demo` en tests
> pasó de 8 a 11 ficheros (medido de nuevo hoy); el relato de campaña en
> los comentarios de `buscador.rs`/`escritor.rs`/`indexer.rs`
> (backlog:1250-1283) NO se mueve al verdict en esta campaña — habría
> exigido reescribir ~10 funciones solo por higiene de comentarios, sin
> oráculo de comportamiento, y viola «construir antes que medir» aplicado a
> churn documental; queda abierto para quien lo priorice. Fuera de esta
> campaña: la fusión de `buscador.rs` (campaña J, después) y el check de
> desfase binario↔plugin (campaña H, en paralelo).)
>
> Anterior: **2026-09-15** (bookkeeping de estado: entran por PR las
```

- [ ] **Step 2: Tabla `## Estado`** — añade una fila de campaña

old_string:
```
| **Ruta portable** | grafía única de ruta (`/`) en el binario y ruta visible en la salida humana de `exo search` — apila sobre la campaña D, mergeada a `main` el **2026-09-15** vía PR #21 (`ba4b75f`); plan en `docs/superpowers/plans/2026-09-11-ruta-portable-y-columna-humana.md`, spec en `docs/superpowers/specs/2026-09-11-ruta-portable-y-columna-humana-design.md` |
```

new_string:
```
| **Ruta portable** | grafía única de ruta (`/`) en el binario y ruta visible en la salida humana de `exo search` — apila sobre la campaña D, mergeada a `main` el **2026-09-15** vía PR #21 (`ba4b75f`); plan en `docs/superpowers/plans/2026-09-11-ruta-portable-y-columna-humana.md`, spec en `docs/superpowers/specs/2026-09-11-ruta-portable-y-columna-humana-design.md` |
| **Campaña G** | 11 tasks del engine (bench sintético con vectores reales, `busca_hybrid` a una sola conexión, `git_epoch_de` en batch, `walk_kb` unificada, `budget_prose_drift` sin truncar, `escribe_nueva` con struct de parámetros, M4 #5/#6, assert de embeddings, `trinquete --staged` sobre el índice de git, `kb-demo`→`kb-test`, D6 default hybrid+0.40 con `exo init` alineado a la misma constante) — sin cambio de ranking (β/bonus/umbral/fusión intactos) — mergeada a `main` el **<fecha>** vía <PR de G>; plan en `docs/superpowers/plans/2026-09-15-campana-g-engine-deuda-diferida.md` |
```

- [ ] **Step 3: Cerrar con evidencia — «Techos de escala» (`backlog:524-566`)**

Reemplaza el párrafo final de re-verificación (cópialo verbatim del fichero
real, esta cita es su forma en `origin/main` al escribir este plan):

old_string:
```
**(re-verificado el 2026-09-15): las dos primeras patas quedan CERRADAS,
```

new_string:
```
**(cerrado con evidencia el <fecha de ejecución>, campaña G): las cuatro
patas quedan cerradas.** `busca_hybrid` abre la DB una sola vez desde
`<commit Task 2>` (Task 2 de G) y `indexer::git_epoch_de` ya no se llama
una vez por nota — `gitx::epochs_de_todo_el_historial` hace un solo
`git log --name-only` por `indexa`, con fallback fail-silent per-nota para
lo que el lote no cubra, desde `<commit Task 3>` (Task 3 de G). Números del
bench (`evals/recall-coste/results/`): `<pegar aquí el resumen.tsv de la
Task 3, Step 6, del plan de G>`.

**(re-verificado el 2026-09-15): las dos primeras patas quedan CERRADAS,
```

- [ ] **Step 4: Cerrar con evidencia — `walk_kb` vs `walk_kb_excluyendo`
  (`backlog:821-845`)**

old_string:
```
- [ ] **`walker::walk_kb` frente a `walk_kb_excluyendo`: conviven con semánticas
  distintas desde G4b.**
```

new_string:
```
- [x] **`walker::walk_kb` frente a `walk_kb_excluyendo`: conviven con semánticas
  distintas desde G4b.** **Cerrado el <fecha de ejecución> (campaña G, Task
  4, commit `<commit Task 4>`, decisión 9 de Paul):** `walk_kb` delega en
  `walk_kb_excluyendo(raiz, &[])` — case-insensitive, no recorre `.git/`.
  Cambio de comportamiento declarado en `docs/arquitectura.md`.
```

- [ ] **Step 5: Cerrar con evidencia — `budget_prose_drift`
  (`backlog:878-910`)**

old_string:
```
  **Acción:** ampliar la regex, o rechazar explícitamente una captura que no
  consume toda la cifra, es trabajo para cuando una cita real mal formada
  haga daño de verdad, o para cuando exista el gate de paridad con Go y el
  fix se pueda decidir en los dos binarios a la vez.
```

new_string:
```
  **Cerrado el <fecha de ejecución> (campaña G, Task 5, commit `<commit
  Task 5>`): el bloqueador («cuando exista el gate de paridad con Go»)
  caducó — los gates de paridad corrieron en la campaña D y kbx dejó de
  ser referencia (`backlog:181`).** `TIER_Y_CIFRA` ahora consume el número
  ENTERO (`[0-9]+(?:\.[0-9]+)*`, sin exigir grupos de 3 en la propia regex)
  y `agrupacion_correcta` valida aparte que agrupe en tríos — una captura
  mal formada ("1.2345") se ignora en vez de truncarse a "1.234" y citar
  una cifra que no está en el texto. Test:
  `la_deriva_de_prosa_ignora_una_cifra_mal_agrupada`
  (`engine/tests/lint_presupuesto.rs`). El punto ciego por adyacencia
  (cifra no pegada al tier) sigue siendo un límite conocido y deliberado,
  sin acción — ver el `Falsos positivos son peores que fallos aquí` del
  propio test.
```

- [ ] **Step 6: Cerrar con evidencia — `#[allow(too_many_arguments)]` de
  `escritor.rs` (`backlog:637-647`)**

old_string (localiza el ítem exacto con
`grep -n "too_many_arguments" docs/backlog.md` y cita su texto real; el
patrón esperado es un ítem de una línea o un párrafo corto sobre
`escritor.rs:252`):
```
  el `allow(clippy::too_many_arguments)` — struct de parámetros para
  `escribe_nueva`.
```

new_string:
```
  **[x] Cerrado el <fecha de ejecución> (campaña G, Task 6, commit
  `<commit Task 6>`):** `escribe_nueva(&NuevaNota{...})` — struct de
  parámetros, `#[allow(clippy::too_many_arguments)]` retirado. `grep -rn
  too_many_arguments engine/src` vacío.
```

- [ ] **Step 7: Cerrar con evidencia — M4 #5 y #6 (`backlog:708-730`)**

old_string:
```
  - **#5 [media]** sin fallback walk+parse (la spec §3.2 lo afirma en presente):
    con índice rancio, un `--crea` puede dejar **dos ficheros con el mismo
    permalink**. Riesgo hoy bajo (las 26 bitácoras de `log/` son slug-clean).
    Mínimo: walk de confirmación antes de crear.
```

new_string:
```
  - [x] **#5 [media] CERRADO el <fecha de ejecución> (campaña G, Task 7,
    commit `<commit Task 7>`):** `busca_permalink_en_dir` hace el walk de
    confirmación antes de crear con `--create`; tests
    `write_append_create_no_duplica_si_el_indice_esta_rancio` en
    `engine/tests/write_create_permalink.rs`.
```

old_string:
```
  - **#6 [baja]** `--crea` con permalink de 2 segmentos crea directorio espurio;
    `write_append_cmd` asume 3.
```

new_string:
```
  - [x] **#6 [baja] CERRADO el <fecha de ejecución> (campaña G, Task 7,
    commit `<commit Task 7>`):** un permalink de <3 segmentos ahora es
    error accionable (`"tiene menos de 3 segmentos"`), no crea directorio.
    Test `write_append_create_con_permalink_de_dos_segmentos_falla_con_remedio`.
```

- [ ] **Step 8: Cerrar con evidencia — `KB-exo:16` (assert de embeddings)**

Este ítem vive en `wisdom-paul/backlog/Backlog — exo.md`, NO en
`docs/backlog.md` de este repo — **no lo toques aquí**. Anota en el
párrafo de cabecera (ya cubierto en el Step 1) que queda cerrado con el
commit de la Task 8; la actualización del fichero externo, si Paul la
quiere, es una acción suya fuera de esta fábrica (no está versionado en
`exo`).

- [ ] **Step 9: Cerrar con evidencia — `trinquete --staged` y tier
  (`backlog:1086-1099`)**

old_string (localiza el ítem con
`grep -n "sellos_escapados_de_tier\|lee el tier del disco" docs/backlog.md`
y cita su texto real completo):
```
- [ ] **`trinquete::sellos_escapados_de_tier` lee el tier del disco en
  `--staged`**
```
(ajusta el `old_string` exacto al texto real del fichero — este plan cita
la forma esperada por el título del ítem en la propuesta de campañas;
verifica con el grep antes de escribir el `Edit`.)

new_string (antepón, sin borrar el resto del ítem):
```
- [x] **CERRADO el <fecha de ejecución> (campaña G, Task 9, commit
  `<commit Task 9>`).** `sellos_escapados_de_tier` recibe un closure
  `contenido_de`; `comprueba_staged` le pasa `contenido_de_indice`
  (`gitx::muestra(kb, ":./<ruta>")`) en vez de leer disco. Test
  `escapados_en_staged_lee_el_tier_del_indice_no_del_disco`. Checklist
  opcional (no bloqueante, no corrida): comparar contra
  `internal/ratchet/staged.go` de kbx `fe46443` en la máquina Linux.

- [ ] **`trinquete::sellos_escapados_de_tier` lee el tier del disco en
  `--staged`**
```

- [ ] **Step 10: Corregir con cita — conteo de `kb-demo` (`backlog:1310-1322`)**

old_string (busca el ítem con
`grep -n "kb-demo.*8 ficheros\|8 ficheros de test" docs/backlog.md` y cita
su texto real):
```
- [ ] **`kb-demo` como fixture en 8 ficheros de test**
```

new_string:
```
- [x] **`kb-demo` como fixture, conteo CORREGIDO y CERRADO el <fecha de
  ejecución> (campaña G, Task 10, commit `<commit Task 10>`).** Eran 11
  ficheros de test (`git grep -l kb-demo -- engine/tests` a fecha de la
  propuesta de campañas, no 8) más 2 comentarios en `src/`. Renombrado a
  `kb-test` en los 11 tests + 2 comentarios de `src/{buscador,inicia}.rs`.
  **Excepciones deliberadas, no tocadas**: `help_producto.rs:154` (gate
  anti-jerga que comprueba que `--help` no mencione el nombre real de la
  KB de Paul), `escritor.rs:29` y `recall_contenido.rs:108` (procedencia
  histórica de datos reales), `src/lib.rs:103` (hecho histórico sobre una
  versión anterior del código). `git grep -c kb-demo -- engine` = 4 (las
  4 excepciones).

- [ ] **`kb-demo` como fixture en 8 ficheros de test**
```

- [ ] **Step 11: Cerrar D6 (`backlog:243-245`)**

old_string:
```
  - [ ] (c) decidir si el default de `exo search --type` (`main.rs:215`)
    pasa a ser el modo medido, o si el README deja de presentar el 48/55
    como «lo que hace exo» — PENDIENTE-PAUL, con A0 vs A2 held-out delante.
```

new_string:
```
  - [x] (c) **CERRADO el <fecha de ejecución> (campaña G, Task 11, commit
    `<commit Task 11>`, decisión 1 de Paul, 2026-09-15).** `exo search`
    sin `--type` usa `hybrid` con `--min-similarity 0.40`
    (`main.rs`, `ArgsSearch.r#type` default `TipoBusqueda::Hybrid`).
    Cita: `evals/retrieval-heldout/verdict/c-verdict.md` (A0 gana a FTS en
    41/55 filas, FTS a A0 en 0 filas). `docs/arquitectura.md` actualizado
    con la cita. **Ampliado el mismo día:** `exo init` (modo creación)
    escribe ahora ese mismo 0.40 en `[embeddings] min_similarity` de la
    config nueva, vía la misma constante `MIN_SIMILARITY_SELLADO` (antes
    0.35, literal propio sin relación con el sweep) — el motivo por el que
    no se compartía ("config es RO hasta M5a") caducó al cerrar M5a-02 el
    2026-08-26 (`backlog:1945`).
```

- [ ] **Step 12: Verificar que todos los `Edit` aplicaron**

Run: `grep -c "\[x\]" docs/backlog.md`
Expected: el número sube en exactamente 5 respecto a antes de esta task
(walk_kb, M4 #5, M4 #6, trinquete-staged, D6 — más el rename de kb-demo si
ese ítem usa el mismo marcador).

Run: `grep -n "<fecha de ejecución>\|<PR de G>\|<commit Task" docs/backlog.md`
Expected: **sin coincidencias** — sustituye cada placeholder por su valor
real (fecha del día, número de PR cuando exista, hash corto de cada commit
de este plan) ANTES de cerrar la task. Los hashes son resolubles sin
ambigüedad: son los commits que las Tasks 1-11 de este mismo plan acaban
de crear en esta misma rama.

- [ ] **Step 13: Commit**

```bash
git add docs/backlog.md
git commit -m "docs(g, backlog): cierra 6 items con evidencia, corrige el conteo de kb-demo y cita D6 — campaña G"
```

---

## Self-review (cobertura, placeholders, consistencia de firmas)

**Cobertura contra §2 de la propuesta** (`docs/superpowers/consultas/2026-09-15-campanas/propuesta.md`):
las 11 filas de la tabla «Esbozo de tasks» de G están cubiertas 1:1 por las
Tasks 1-11 de este plan, más la Task 12 de sync (patrón D/E). Los ítems
`#8` (divergencia de slug) y `#9` (SKILL.md sin `--db`, caducado) de la
tabla M4 del backlog **no entran**: `#8` es decisión de Paul (§5 de la
propuesta, fuera de fábrica) y `#9` ya está caducado (verificado: `--db`
existe en `ArgsWriteAppend`, `SKILL.md` está correcto) — el backlog ya lo
documenta así, esta campaña no necesita tocarlo de nuevo.

**Placeholders:** cero `TBD`/`TODO`. Los únicos marcadores entre `<...>`
son, deliberadamente, valores que solo existen DESPUÉS de ejecutar (fecha,
número de PR, hashes de commit) — el mismo patrón que usan los planes de
D y E citados como referencia, con instrucción explícita de sustituirlos
antes de cerrar la Task 12.

**Consistencia de firmas entre tasks:**
- Task 6 introduce `NuevaNota`; Task 7 la CONSUME (no la redefine) para su
  fix de M4 #5/#6 en el mismo `write_append_cmd` — verificado que el
  `old_string` del Step 3 de la Task 7 ya asume el `escribe_nueva(&NuevaNota{...})`
  que deja la Task 6.
- Task 2 mantiene las firmas públicas de `busca`/`busca_vector`/`busca_hybrid`
  exactamente iguales — Task 11 (Step 5) llama a `busca_hybrid` con la
  MISMA firma de 7 argumentos que ya tenía antes de la Task 2.
- Task 10 depende de que la Task 6 ya haya convertido
  `engine/tests/escritor.rs` a `NuevaNota{...}` — el rename de la Task 10
  opera sobre literales de string (`"kb-demo"`) dentro de esos structs, no
  sobre su forma posicional, así que el orden Task 6 → Task 10 es
  obligatorio y está declarado en el header de la Task 10.
- Task 9 cambia la firma PRIVADA de `sellos_escapados_de_tier` — verificado
  que ningún fichero fuera de `trinquete.rs` la llama
  (`grep -rn sellos_escapados_de_tier engine/` antes de escribir esta task
  dio solo coincidencias dentro de `trinquete.rs`).
- Ninguna task de este plan introduce una firma que otra task de una
  campaña distinta (F, H, J) necesite — verificado contra la matriz de
  colisión §3 de la propuesta: F no toca `engine/src`, H no toca los
  módulos de G salvo `doctor.rs` (que USA `walk_kb`, sin cambiar su propia
  firma) y `main.rs` en zonas disjuntas, J llega después y consumirá las
  firmas de `buscador.rs` tal como las deja la Task 2 (documentado en el
  Global Constraints de este plan).

**Fix inline aplicado durante el self-review:** ninguno — la investigación
de código (ver informe de cierre) se hizo ANTES de escribir cada task, así
que no quedó ninguna afirmación de la propuesta sin verificar contra
`origin/main` antes de convertirla en Step.

**Addendum 2026-09-15 (enmienda de Task 11 — decisión de Paul, alinear el
default de `exo init`):** `git grep -n "0\.35"` contra `campana-g` antes de
escribir la enmienda dio 4 ficheros de código con literales relevantes
(`engine/src/main.rs:656`, `engine/src/buscador.rs:241`,
`engine/src/escritor.rs:163`, `engine/examples/kb_sintetica.rs:183`) y 4
citas en `docs/arquitectura.md` (§3.4, §3.5×2, §3.8) — todos cubiertos ahora
por los Steps 3-9 de la Task 11. Quedan **deliberadamente fuera**, por ser
registros históricos y no defaults vigentes (misma clase que las
excepciones de `kb-demo` de la Task 10): `engine/src/buscador.rs:384`
(medición H29 a 95k trozos), `evals/retrieval-heldout/harness/metricas.py:125`
(harness del held-out ya gastado — Global Constraints prohíbe tocarlo),
todos los `evals/**/*.md`/`*.py`/`verdict/*.md` que citan el 0.35 del sweep
M0/M2 (medidas fechadas), `.superpowers/fabrica/config.md:148` y
`docs/superpowers/runbooks/2026-08-24-integracion-equipo-trabajo-windows.md:47`
(runbook fechado, describe `~/.basic-memory/config.json`, un sistema previo
a M5a-02). Los tests que fijan `0.35` como valor de FIXTURE arbitrario en
`engine/tests/{inicia,config,config_cmd,precedencia,doctor,doctor_cli,
write_create_permalink}.rs` y `engine/tests/common/mod.rs::MIN_SIMILARITY`
tampoco se tocan: prueban parseo/precedencia/round-trip de CUALQUIER valor
de config, no el default que escribe `exo init` — cambiarlos sería mover el
oráculo sin motivo. `MIN_SIMILARITY_SELLADO` queda como fuente única para
los dos consumidores (default de `--type hybrid` y default de `exo init`),
resolviendo la pregunta abierta del brief ("¿misma fuente que dos
literales?") a favor de una constante — ver Step 5 de la Task 11.
