# Campaña L — Sueltos mecánicos pre-v0.2.0 y preparación de M5b — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: usa `exo:orchestrate` para
> ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`) para
> tracking.

**Goal:** arreglar la regresión que la campaña G metió en `exo search`
(default `hybrid` contra una DB sin tabla `vectores`), trocear el `IN (...)`
de `permalinks_de_rowids` bajo el límite de placeholders de SQLite, cerrar
el gate de hooks del README que falta en `plugins/exo/README.md`, blindar
tres gates de CI contra `cd ""` silencioso, dejar listo (sin ejecutarlo) el
runbook de desinstalación de basic-memory (checklist C10), sincronizar
`docs/backlog.md` con lo que el código y las decisiones de Paul del
2026-09-19 ya cerraron, y verificar la rama entera — **para que Paul pueda
tagear `v0.2.0` inmediatamente después de mergear esta rama** (decisión de
Paul, 2026-09-19: el tag espera a L porque L arregla la regresión que G
introdujo).

**Architecture:** siete tareas sobre módulos disjuntos. Tasks 1-2 tocan
`engine/src/buscador.rs` (funciones distintas, sin overlap de líneas) y sus
tests; Task 3 toca `scripts/test-docs-vivos.sh`; Task 4 toca tres scripts de
`scripts/` que ninguna otra task de este plan toca; Task 5 crea un fichero
nuevo (runbook); Task 6 toca `docs/backlog.md` y una línea de
`docs/superpowers/runbooks/2026-09-15-release-v0.2.0.md`; Task 7 es
verificación pura, sin cambios de fichero. Ninguna task cambia ranking,
schema del envelope, ni escribe en `~/.local/bin`.

**Tech Stack:** Rust 2024 (crate `exo` en `engine/`, MSRV 1.95) · `clap`
4.6.2 derive · `rusqlite` 0.40.1 + `sqlite-vec` 0.1.9 (bundled, SQLite con
`SQLITE_MAX_VARIABLE_NUMBER` = 32.766, medido empíricamente en la Task 2 —
NO los 999 de sqlite3 &lt; 3.32.0) · `serde_json` · bash (gates de
`scripts/`) · `jq` · `awk` · GitHub Actions (`.github/workflows/ci.yml`).

## Global Constraints

Todas verificadas hoy (2026-09-19) contra `HEAD` de `plan-campanas-i-l-j`
(que ya incluye G y H mergeadas a `main`).

- **El crate vive en `engine/`, no en la raíz.** No hay workspace. Todo
  `cargo` con cwd `engine/`.
- **MSRV `rust-version = "1.95"`**, edition 2024.
- **Lint:** `cd engine && cargo fmt --check` y
  `cargo clippy --all-targets --locked -- -D warnings`.
- **Tests:** `cd engine && cargo test --release --locked`.
- **Envelope v2 sin cambios.** Ninguna task de este plan toca el schema de
  `data` de `search`/`recall`/`write`/`index` ni sube `SCHEMA_VERSION`. La
  Task 1 usa un campo YA aditivo y ya serializado (`Busqueda.avisos` →
  `data.warnings`, desde el cierre de "Modo mudo de `busca_hybrid`",
  2026-08-22) — no añade ninguna clave nueva.
- **Sin cambio de ranking.** Ninguna task de este plan toca β, bonus, umbral
  de similitud, troceado ni el algoritmo de fusión
  (`normaliza_fts`/`fusiona`). La Task 1 solo cambia qué pasa cuando la
  tabla `vectores` NO EXISTE (antes: `Err` duro; ahora: se trata como "0
  vectores", que YA es un caso existente y ya probado — no inventa
  comportamiento de ranking nuevo). La Task 2 no cambia qué filas devuelve
  `permalinks_de_rowids`, solo CÓMO las pide (en lotes en vez de un único
  `IN`) — cubierto por un test de equivalencia exacta.
- **Nada escribe en `~/.local/bin`.** Ninguna task instala, compila-e-instala
  ni toca el binario en uso de Paul. La Task 5 (runbook) es lectura y
  comprobaciones en seco; **no desinstala nada** — desinstalar basic-memory
  es acción de Paul, línea roja no delegable (mismo régimen que
  `docs/superpowers/plans/2026-08-17-cierre-exo-m2-a-m5b.md` §Campaña 10).
- **`v0.2.0` en `Cargo.toml` ya está puesto** (campaña H). Esta campaña NO
  sube versión — es la campaña que Paul decidió mergear ANTES de poner el
  tag `v0.2.0` en GitHub, no una que lo dispare.
- **Git:** `git add <rutas explícitas>`, nunca `-A`/`.`; `git -C <path>`,
  nunca `cd <path> && git`. Nada de push ni de tocar GitHub (salvo la
  lectura de solo-consulta `gh api .../branches/main/protection` de la
  Task 6, que no escribe nada).
- **Commits:** `<tipo>(l, <slug>): <mensaje>` — `fix`, `perf`, `test`,
  `docs`, `chore` según corresponda.
- **Ejecución en paralelo con la campaña I**: I toca los SCRIPTS de los
  hooks bajo `plugins/exo/scripts/` (p.ej. `recall-inject.sh`,
  `exo-recall.sh` — la implementación bash de los reflejos, corrección del
  self-review: `plugins/exo/hooks/` NO contiene ningún `.sh`, solo
  `hooks.json`, verificado con `ls plugins/exo/hooks/`), sus tests
  (`plugins/exo/scripts/test-*.sh` de los hooks que toca),
  `plugins/exo/hooks/hooks.json` (la wiring, puede que I la edite si
  fusiona scripts), `plugins/exo/scripts/recall-latencia.sh` +
  `plugins/exo/scripts/test-recall-latencia.sh`, y **`docs/backlog.md`**
  (compartido). **Ninguna task de este plan ESCRIBE en `plugins/exo/**`**
  — la Task 3 lee `plugins/exo/README.md` (nunca lo edita) y la Task 3/5
  leen `plugins/exo/hooks/hooks.json` con `jq`/`grep` de solo consulta
  (nunca lo editan) para contar/verificar hooks; si I cambia el número de
  hooks o su wiring mientras L corre en paralelo, la Task 3 sigue midiendo
  correctamente porque lee `hooks.json` en el momento de ejecutarse, no un
  número congelado en este plan. Cero colisión de ESCRITURA salvo
  `docs/backlog.md`. `docs/backlog.md` se **re-ancla por texto, no por
  número de línea**, al mergear: todos los `old_string` de la Task 6 citan
  una frase única del párrafo, no un rango de líneas — si I ya movió líneas
  al mergear antes, el contenido citado sigue siendo `grep`-able igual
  (mismo patrón que usaron D/E/G entre sí). El orden de merge entre I y L
  no importa para esta regla; el que merge segundo re-ancla sus
  `old_string` contra el árbol ya actualizado.

### Tabla de ficheros por task (para que quede explícito el no-solape con I)

| Task | Ficheros que TOCA | Ficheros que NO toca (aunque los mencione) |
|---|---|---|
| 1 | `engine/src/buscador.rs`, `engine/tests/kb_root_lectura_cli.rs` | — |
| 2 | `engine/src/buscador.rs` | — |
| 3 | `scripts/test-docs-vivos.sh` | `README.md`, `plugins/exo/README.md`, `plugins/exo/hooks/hooks.json` (se LEEN con `jq`/`awk`, no se editan) |
| 4 | `scripts/test-rutas-personales.sh`, `scripts/test-exec-bit.sh`, `scripts/test-versiones.sh` | `scripts/test-hooks-json.sh`, `scripts/test-shellcheck.sh` (ya arreglados en `960a319`) |
| 5 | `docs/superpowers/runbooks/2026-09-19-m5b-desinstalar-basic-memory.md` (nuevo) | `plugins/exo/**` (se lee `hooks.json`/`SKILL.md` con `grep`, nunca se edita), cualquier fichero de basic-memory, `~/.claude.json` |
| 6 | `docs/backlog.md`, `docs/superpowers/runbooks/2026-09-15-release-v0.2.0.md` | `.superpowers/fabrica/config.md`, `docs/superpowers/consultas/2026-09-15-campanas/propuesta.md` (los lee como fuente, no los edita — los está escribiendo el orquestador en paralelo) |
| 7 | ninguno (solo verificación) | — |

---

### Task 1: `exo search` (default `hybrid`) degrada con aviso si falta la tabla `vectores`, en vez de reventar

**Lane:** mecánica. **Depende de:** nada.

**Decisión de esta task (declarada, con alternativa):** el ítem del backlog
que documenta esta regresión (`docs/backlog.md`, ítem «(NUEVO, review final
campaña G, 2026-09-16, M5) `exo search` sin `--type` contra una DB sin
tabla `vectores` falla duro donde antes daba FTS» — la cita del brief como
`~backlog:1712` está desactualizada, la posición real en HEAD es
`~backlog:838-856`, corregido en el self-review) **ya fija la acción**:
*"si se toca `busca_hybrid`/`busca_vector_con` por otra razón, degradar
'tabla `vectores` ausente' igual que '0 filas en `vectores`' (mismo aviso
de cobertura, no un error duro)"*. Esta task sigue esa acción: **fallback a
FTS puro, con aviso visible en stderr (`aviso: ...`) Y en el envelope
(`data.warnings`)** — nunca un fallback mudo (doctrina de fallo silencioso:
un instrumento degradado que no grita es el modo de fallo caro). El
mecanismo para el aviso visible **ya existe y ya está probado**:
`avisos_cobertura_vector` (que alimenta `Busqueda.avisos`, serializado como
`data.warnings`, aditivo desde el cierre de "Modo mudo de `busca_hybrid`",
2026-08-22) — este fix solo hace que ese mismo camino se alcance también
cuando la tabla `vectores` no existe, no solo cuando existe vacía.
**Alternativa considerada y descartada:** error accionable ("tabla
`vectores` ausente: corre `exo index`/`exo rebuild`, o pasa `--type fts`").
Se descarta porque (a) el ítem del backlog ya adjudicó la otra opción, (b)
sería inconsistente con el resto de `busca_hybrid`, que ya trata "0
vectores" (tabla vacía) como degradación silenciosa-pero-avisada, no como
error — una tabla AUSENTE es el mismo caso límite, no uno cualitativamente
distinto, y (c) el riesgo real ya está adjudicado como bajo (`schema.rs`
crea `vectores` SIEMPRE al indexar; el caso solo se da con una DB que nunca
pasó por `exo index`/`rebuild`).

**Files:**
- Modify: `engine/src/buscador.rs`
- Modify: `engine/tests/kb_root_lectura_cli.rs`

**Interfaces:**
- Consumes: `rusqlite::Connection`, `rusqlite::OptionalExtension` (ya
  importado en el fichero, `use rusqlite::{OptionalExtension, params};`).
- Produces: `fn tabla_existe(conn: &rusqlite::Connection, tabla: &str) -> Result<bool>`
  (privada, nueva) — ninguna otra task de este plan la consume.
  `busca_vector_con`/`avisos_cobertura_vector` cambian de cuerpo interno,
  NO de firma pública (`busca`, `busca_vector`, `busca_hybrid` quedan
  exactamente igual).

**Fix del self-review adversarial (2026-09-19, verificado en worktree
temporal — ver más abajo):** el diseño original de esta task
(`cuenta_o_ausente`, que hacía `trozos = 0 si la tabla no existe` y
`vectores = 0 si la tabla no existe`, cada uno de forma independiente) deja
un **fallback MUDO**: la fixture `db_sin_tabla_meta`
(`engine/tests/kb_root_lectura_cli.rs:244`) no crea `trozos` EN ABSOLUTO,
así que `avisos_cobertura_vector` hacía `trozos == 0` → `return
Ok(Vec::new())` **antes** de siquiera mirar `vectores` — cero avisos, exit
0, stderr vacío. El test del Step 1 quedaba en rojo para SIEMPRE, no solo
en el Step 2 (confirmado corriendo esa versión en un worktree: panic
`el default hybrid degradado debía avisar por stderr:` con stderr vacío).
Diseño corregido: `tabla_existe("vectores")` se comprueba **antes** del
`trozos == 0` early-return, y si `vectores` NO EXISTE se avisa siempre
(con un mensaje NUEVO y distinto: "la tabla `vectores` no existe", nunca el
texto "0 vectores para N trozos" que ya usan otros tests) — una DB con
schema incompleto no es "vacía todavía" (que no avisa, doctrina ya
existente), es "nunca pasó por el indexer" (sí avisa, aunque `trozos`
también sea 0). El mensaje ORIGINAL de "0 vectores para {trozos} trozos."
(tabla `vectores` existente pero con 0 filas) **no se toca** — sigue
palabra por palabra igual que en HEAD.

- [ ] **Step 1: Test que falla — el default `hybrid` no debe tumbar el
  comando**

Añade en `engine/tests/kb_root_lectura_cli.rs`, INMEDIATAMENTE DESPUÉS del
cierre de `search_no_avisa_ni_falla_si_la_db_no_tiene_tabla_meta` (justo
antes del doc-comment de
`search_no_avisa_si_la_kb_de_config_no_existe_en_disco`):

old_string:
```rust
    // Resultados intactos: el FTS de siempre, sin degradar.
    let salida = String::from_utf8_lossy(&out.stdout);
    assert!(
        salida.contains("kb/a"),
        "la búsqueda debía seguir encontrando la nota: {salida}"
    );
}

/// La KB de la CONFIG (la que `search` resolvería sin `$EXO_KB`) no existe
```

new_string:
```rust
    // Resultados intactos: el FTS de siempre, sin degradar.
    let salida = String::from_utf8_lossy(&out.stdout);
    assert!(
        salida.contains("kb/a"),
        "la búsqueda debía seguir encontrando la nota: {salida}"
    );
}

/// Campaña L Task 1: el default `hybrid` (D6) contra la fixture de arriba
/// (`vectores` AUSENTE, ni siquiera vacía) degrada con aviso en vez de
/// tumbar el comando — la regresión que documentaba el test de arriba antes
/// de este fix. `avisos_cobertura_vector`/`tabla_existe`
/// (`engine/src/buscador.rs`) tratan "tabla ausente" como su propio caso,
/// comprobado ANTES que "0 filas": siempre avisa "arm vector INERTE",
/// nunca un `Err` duro.
#[test]
fn search_default_hybrid_degrada_con_aviso_si_falta_tabla_vectores() {
    let kb_pedida = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let db = db_sin_tabla_meta(dir.path());
    let cfg = config_con_kb(dir.path(), kb_pedida.path());

    let out = Command::new(bin())
        .args(["search", "--db"])
        .arg(&db)
        .arg("--json")
        .arg("buscable")
        .env("EXO_CONFIG", &cfg)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "DB sin tabla vectores no debía tumbar el default hybrid: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        tiene_aviso(&err),
        "el default hybrid degradado debía avisar por stderr: {err}"
    );

    let salida: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(salida["data"]["search_type"], "hybrid");
    let warnings = salida["data"]["warnings"]
        .as_array()
        .expect("data.warnings debe existir en el envelope degradado");
    assert!(
        warnings
            .iter()
            .any(|w| w.as_str().unwrap_or("").contains("INERTE")),
        "el envelope debía llevar el aviso de arm INERTE en data.warnings: {salida}"
    );
    let permalinks: Vec<&str> = salida["data"]["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["permalink"].as_str().unwrap())
        .collect();
    assert!(
        permalinks.contains(&"kb/a"),
        "el canal FTS debía seguir encontrando la nota: {salida}"
    );
}

/// La KB de la CONFIG (la que `search` resolvería sin `$EXO_KB`) no existe
```

- [ ] **Step 2: Correr el test y verlo fallar por la razón esperada**

Run: `cd engine && cargo test --release --locked search_default_hybrid_degrada_con_aviso_si_falta_tabla_vectores -- --nocapture`

Expected: FAIL. `out.status.success()` es `false` (el proceso sale con
exit 1). El mensaje de aserción muestra el stderr real:
```
DB sin tabla vectores no debía tumbar el default hybrid: error: contar filas de vectores

Caused by:
    no such table: vectores
```
(exactamente el mensaje que produce hoy `busca_vector_con` al hacer
`conn.query_row("SELECT count(*) FROM vectores", ...)` sobre una DB que
nunca creó esa tabla — verificado en `main.rs::main()`, rama `Err(e) => { ...
eprintln!("error: {e:#}"); std::process::exit(1); }`).

- [ ] **Step 3: Implementación — `tabla_existe` comprobado ANTES del
  early-return de `trozos == 0`**

En `engine/src/buscador.rs`, reemplaza `avisos_cobertura_vector`:

old_string:
```rust
/// Avisos de cobertura del arm vector: compara filas de `vectores` contra
/// filas de `trozos`, que es la relación 1:1 que mantiene el indexer.
///
/// Corpus vacío (`trozos == 0`) no avisa: una DB recién creada no está
/// degradada, está vacía. Avisar ahí sería el falso rojo simétrico.
fn avisos_cobertura_vector(conn: &rusqlite::Connection) -> Result<Vec<String>> {
    let trozos: i64 = conn
        .query_row("SELECT count(*) FROM trozos", [], |f| f.get(0))
        .context("contar filas de trozos")?;
    if trozos == 0 {
        return Ok(Vec::new());
    }
    let vectores: i64 = conn
        .query_row("SELECT count(*) FROM vectores", [], |f| f.get(0))
        .context("contar filas de vectores")?;

    Ok(if vectores == 0 {
        vec![format!(
            "arm vector INERTE: 0 vectores para {trozos} trozos.              El resultado sale de FTS puro aunque se etiquete hybrid;              reindexa con `exo index` antes de fiarte del ranking."
        )]
    } else if vectores < trozos {
        vec![format!(
            "cobertura vectorial PARCIAL: {vectores} de {trozos} trozos embebidos.              El arm vector no puede aportar los que faltan."
        )]
    } else {
        Vec::new()
    })
}
```

new_string:
```rust
/// Existe `tabla` en el schema de `conn` — vía `sqlite_master`, que también
/// registra las tablas virtuales (`vectores` es `USING vec0(...)`) con
/// `type='table'`.
fn tabla_existe(conn: &rusqlite::Connection, tabla: &str) -> Result<bool> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1",
            params![tabla],
            |_| Ok(()),
        )
        .optional()
        .with_context(|| format!("comprobar existencia de la tabla {tabla}"))?
        .is_some())
}

/// Avisos de cobertura del arm vector: compara filas de `vectores` contra
/// filas de `trozos`, que es la relación 1:1 que mantiene el indexer.
///
/// Corpus vacío (`trozos == 0`) no avisa: una DB recién creada no está
/// degradada, está vacía. Avisar ahí sería el falso rojo simétrico. Eso
/// asume que el schema SÍ existe (`exo index`/`rebuild` ya corrieron) — una
/// tabla `vectores` AUSENTE (Campaña L Task 1: «exo search sin --type
/// contra una DB sin tabla vectores falla duro donde antes daba FTS»,
/// review final de G, 2026-09-16, M5) es un caso distinto: la DB no es
/// "vacía todavía", es "nunca pasó por el indexer" — por eso este chequeo
/// va ANTES del early-return de `trozos == 0`, y avisa aunque `trozos`
/// también sea 0 (fixture real: `db_sin_tabla_meta` en
/// `engine/tests/kb_root_lectura_cli.rs` no crea NI `trozos` NI `vectores`
/// — sin este orden, el early-return de `trozos == 0` se comía el aviso
/// antes de llegar a mirar `vectores`, dejando un fallback MUDO).
fn avisos_cobertura_vector(conn: &rusqlite::Connection) -> Result<Vec<String>> {
    if !tabla_existe(conn, "vectores")? {
        return Ok(vec![
            "arm vector INERTE: la tabla `vectores` no existe (la DB nunca \
             pasó por `exo index`/`exo rebuild`).              El resultado \
             sale de FTS puro aunque se etiquete hybrid;              corre \
             `exo index` antes de fiarte del ranking."
                .to_string(),
        ]);
    }

    let trozos: i64 = conn
        .query_row("SELECT count(*) FROM trozos", [], |f| f.get(0))
        .context("contar filas de trozos")?;
    if trozos == 0 {
        return Ok(Vec::new());
    }
    let vectores: i64 = conn
        .query_row("SELECT count(*) FROM vectores", [], |f| f.get(0))
        .context("contar filas de vectores")?;

    Ok(if vectores == 0 {
        vec![format!(
            "arm vector INERTE: 0 vectores para {trozos} trozos.              El resultado sale de FTS puro aunque se etiquete hybrid;              reindexa con `exo index` antes de fiarte del ranking."
        )]
    } else if vectores < trozos {
        vec![format!(
            "cobertura vectorial PARCIAL: {vectores} de {trozos} trozos embebidos.              El arm vector no puede aportar los que faltan."
        )]
    } else {
        Vec::new()
    })
}
```

**Nota deliberada:** el mensaje "arm vector INERTE: 0 vectores para
{trozos} trozos." (tabla `vectores` EXISTENTE con 0 filas) queda **palabra
por palabra igual que en HEAD** — no se toca, aunque parezca tentador
unificarlo con el mensaje nuevo. Motivo: `engine/tests/recall_avisos_cli.rs`
y `plugins/exo/scripts/test-recall-inject.sh` ya encajan contra ese texto
(por `contains("INERTE")`, no exacto, pero no hace falta arriesgar nada
tocándolo sin necesidad — YAGNI).

Y en `busca_vector_con`, reemplaza el cómputo de `total_vectores`:

old_string:
```rust
    let inicio = Instant::now();
    // Mismo aviso best-effort que `busca`, sobre esta misma conexión.
    let aviso_kb_root = crate::indexer::aviso_kb_root_lectura(conn, kb);

    let total_vectores: i64 = conn
        .query_row("SELECT count(*) FROM vectores", [], |r| r.get(0))
        .context("contar filas de vectores")?;

    let results = if total_vectores == 0 || query.trim().is_empty() {
```

new_string:
```rust
    let inicio = Instant::now();
    // Mismo aviso best-effort que `busca`, sobre esta misma conexión.
    let aviso_kb_root = crate::indexer::aviso_kb_root_lectura(conn, kb);

    // Campaña L Task 1: tabla `vectores` AUSENTE cuenta como 0 vectores, en
    // vez de un `Err` duro (ver doc de `tabla_existe`/`avisos_cobertura_vector`).
    let total_vectores: i64 = if tabla_existe(conn, "vectores")? {
        conn.query_row("SELECT count(*) FROM vectores", [], |r| r.get(0))
            .context("contar filas de vectores")?
    } else {
        0
    };

    let results = if total_vectores == 0 || query.trim().is_empty() {
```

- [ ] **Step 4: Correr el test y verlo pasar**

Run: `cd engine && cargo test --release --locked --test kb_root_lectura_cli search_default_hybrid_degrada_con_aviso_si_falta_tabla_vectores -- --nocapture`
Expected: PASS. **Verificado de verdad hoy en un worktree temporal**
(`git worktree add --detach <scratch>/l-verify-wt HEAD`, aplicado este
mismo Step 3 + este test, limpiado después con `git worktree remove
--force` — no queda residuo en el repo principal):
```
running 1 test
test search_default_hybrid_degrada_con_aviso_si_falta_tabla_vectores ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.02s
```
(Nota: usa `--test kb_root_lectura_cli <nombre>`, no solo `<nombre>` suelto
— con un filtro de nombre solo, `cargo test` reparte la búsqueda entre
TODOS los binarios de test y en máquinas con muchos binarios el output es
más ruidoso de leer; con `--test` fijas el binario y vas directo al grano.)

- [ ] **Step 5: No regresión — toda la suite de `kb_root_lectura_cli.rs`, de
  `buscador` (unit) y de `buscador_cli` (integración)**

Run: `cd engine && cargo test --release --locked --test kb_root_lectura_cli -- --nocapture`
Expected: **17 tests, todos verdes** (verificado hoy en el mismo worktree),
incluido `search_no_avisa_ni_falla_si_la_db_no_tiene_tabla_meta` (que pasa
`--type fts` explícito y sigue ejerciendo el camino que NO usa el arm
vector — no debe cambiar de comportamiento).

Run: `cd engine && cargo test --release --locked --lib buscador -- --nocapture`
Expected: **12 tests, todos verdes** (`tests_fusion`, `tests_knn_por_consulta`
—incluida `equivalencia_exacta_por_encima_del_tope_vec0`—,
`tests_una_conexion`; verificado hoy). El filtro `buscador` a secas (sin
`--lib`) NO alcanza `engine/tests/buscador_cli.rs`: ninguno de sus nombres
de test contiene la subcadena `buscador` (comprobado hoy: con
`cargo test --release --locked buscador` esa suite sale con "0 tests...
filtered out"), así que hace falta el `--test` explícito de abajo para no
darla por cubierta sin estarlo.

Run: `cd engine && cargo test --release --locked --test buscador_cli --test contrato_envelope --test recall_avisos_cli -- --nocapture`
Expected: **9 + 9 + 2 tests, todos verdes** (verificado hoy), incluidos
`search_sin_type_con_vectores_poblada_resuelve_hybrid`
(`buscador_cli.rs` — el camino NORMAL del default hybrid, con `vectores`
poblada, sigue igual), `las_claves_de_search_estan_en_ingles`
(`contrato_envelope.rs` — contrato de forma, construido a mano, no toca
`avisos_cobertura_vector` en absoluto) y
`recall_query_con_arm_vector_inerte_avisa_y_publica_tiempos`
(`recall_avisos_cli.rs` — el aviso original "0 vectores para N trozos"
sigue intacto, camino de `exo recall`, no de `exo search`).

- [ ] **Step 5b: Suite completa (evidencia final antes de dar la task por
  cerrada)**

Run: `cd engine && cargo test --release --locked > /tmp/l-task1-full.log 2>&1; echo "EXIT=$?"; grep -c 'test result: ok' /tmp/l-task1-full.log; grep -c 'FAILED' /tmp/l-task1-full.log`
Expected: `EXIT=0`, un conteo de `test result: ok` igual al número de
binarios de test del crate (verificado hoy: **55**, cero `FAILED`). Este es
el gate real, no un muestreo — los Steps de arriba son para localizar rápido
si algo falla, este es el que decide si la task está verde de verdad.

- [ ] **Step 6: Commit**

```bash
git add engine/src/buscador.rs engine/tests/kb_root_lectura_cli.rs
git commit -m "fix(l, search-vectores-ausente): exo search (default hybrid) degrada con aviso si falta la tabla vectores, en vez de reventar"
```

---

### Task 2: `permalinks_de_rowids` trocea el `IN (...)` bajo el límite de placeholders de SQLite

**Lane:** mecánica. **Depende de:** nada. **No colisiona con Task 1**: Task
1 toca `busca_vector_con`/`avisos_cobertura_vector` (líneas ~186-336 según
HEAD); esta task toca `permalinks_de_rowids` (líneas ~350-372), sin
overlap.

**Files:**
- Modify: `engine/src/buscador.rs`

**Interfaces:**
- Consumes: nada nuevo.
- Produces: `permalinks_de_rowids` **mantiene su firma exacta**
  (`fn permalinks_de_rowids(conn: &rusqlite::Connection, rowids: &[i64]) -> Result<HashMap<i64, String>>`)
  — `tests_knn_por_consulta::referencia_exhaustiva` (que ya la llama) no
  cambia. Nueva función privada
  `fn permalinks_de_rowids_lote(conn: &rusqlite::Connection, rowids: &[i64]) -> Result<HashMap<i64, String>>`
  y constante `const LOTE_PERMALINKS: usize = 500`.

**Medición previa (evidencia, no parte del ciclo rojo-verde):** el límite
real de `SQLITE_MAX_VARIABLE_NUMBER` en el SQLite bundled que usa este
crate (rusqlite 0.40.1 / libsqlite3-sys 0.38.1) es **32.766**, NO los 999 de
versiones de sqlite3 anteriores a 3.32.0. Medido hoy con un proyecto
standalone (`rusqlite = { version = "0.40.1", features = ["bundled"] }`,
mismo major.minor que `engine/Cargo.lock`) preparando
`SELECT count(*) FROM (SELECT 1 WHERE 1 IN (?,?,...))` con N crecientes:
`n=32766` → `OK`; `n=32767` → `ERR too many SQL variables`. La producción
real del bug (`exo search --type vector --limit 5000`) necesita, además,
que `total_vectores` de la KB supere ese umbral: `busca_vector_con_embedding`
pide `k = min(total_vectores, max(K_MIN, limite·K_FACTOR_INICIAL))`
(`K_FACTOR_INICIAL = 8`), así que con `--limit 5000` eso es
`min(total_vectores, 40000)` — con la KB sintética de 5.000 notas
(`engine/examples/kb_sintetica.rs`, ~18,9 trozos/nota de media, ~94.500
vectores totales) `k` llega a 40.000 > 32.766 y `permalinks_de_rowids`
revienta.

- [ ] **Step 1: Test que falla — trocear un `IN(...)` de 40.000 rowids**

Añade al final de `engine/src/buscador.rs`, después del cierre de
`tests_una_conexion` (línea final del fichero):

old_string:
```rust
        assert!(
            !contiene_llamada(cuerpo, "busca_vector"),
            "busca_hybrid no debe llamar a la función pública \
             busca_vector(db_ruta, ...) — cada llamada abre su propia conexión y \
             rompe la garantía de una sola apertura; debe usar \
             busca_vector_con(&conn, ...) sobre la conexión ya abierta:\n{cuerpo}"
        );
    }
}
```

new_string:
```rust
        assert!(
            !contiene_llamada(cuerpo, "busca_vector"),
            "busca_hybrid no debe llamar a la función pública \
             busca_vector(db_ruta, ...) — cada llamada abre su propia conexión y \
             rompe la garantía de una sola apertura; debe usar \
             busca_vector_con(&conn, ...) sobre la conexión ya abierta:\n{cuerpo}"
        );
    }
}

#[cfg(test)]
mod tests_permalinks_lote {
    use super::*;
    use crate::abre_db_en_memoria;
    use crate::schema::crea_schema;

    /// TDD rojo: sin trocear, un `IN (...)` de 40.000 placeholders revienta
    /// el límite de variables de SQLite (32.766, medido empíricamente contra
    /// rusqlite 0.40.1/libsqlite3-sys 0.38.1 — bundled, no el 999 de
    /// versiones de sqlite3 anteriores a 3.32.0). No hace falta poblar
    /// `trozos`: SQLite rechaza la sentencia al PREPARARLA, antes de mirar
    /// una sola fila — por eso `crea_schema` sin insertar nada ya basta para
    /// reproducir el reventón.
    #[test]
    fn trocea_por_encima_del_limite_de_placeholders_de_sqlite() {
        let conn = abre_db_en_memoria().expect("db en memoria");
        crea_schema(&conn).expect("crea_schema");
        let rowids: Vec<i64> = (1..=40_000).collect();
        let resultado = permalinks_de_rowids(&conn, &rowids);
        assert!(
            resultado.is_ok(),
            "permalinks_de_rowids debe trocear el IN(...) bajo el límite de \
             placeholders de SQLite: {:?}",
            resultado.err()
        );
        assert!(
            resultado.unwrap().is_empty(),
            "ningún id de los 40.000 existe de verdad en trozos (vacía): el \
             mapa debe salir vacío, no un error"
        );
    }

    /// Equivalencia: trocear en lotes de `LOTE_PERMALINKS` no puede perder ni
    /// inventar filas frente a una única consulta sin trocear. 650 filas
    /// reales (por encima del lote de 500 con el default de esta task, así
    /// que `permalinks_de_rowids` internamente ejecuta DOS lotes) contra la
    /// misma consulta hecha de una sola vez con `permalinks_de_rowids_lote`
    /// (650 está muy por debajo del límite real de 32.766, así que la
    /// versión sin trocear es aquí la referencia válida, no el bug).
    #[test]
    fn union_de_lotes_coincide_con_una_sola_consulta_sin_trocear() {
        let mut conn = abre_db_en_memoria().expect("db en memoria");
        crea_schema(&conn).expect("crea_schema");
        let tx = conn.transaction().expect("abrir transacción");
        let mut ids: Vec<i64> = Vec::new();
        for i in 0..650i64 {
            let permalink = format!("nota-{i}");
            tx.execute(
                "INSERT INTO notas (permalink, ruta, titulo, mtime) VALUES (?1, ?2, ?3, 0.0)",
                rusqlite::params![permalink, format!("{permalink}.md"), permalink],
            )
            .expect("insertar nota");
            tx.execute(
                "INSERT INTO trozos (id, permalink, orden, texto) VALUES (?1, ?2, 0, 'x')",
                rusqlite::params![i, permalink],
            )
            .expect("insertar trozo");
            ids.push(i);
        }
        tx.commit().expect("commit del corpus");

        let sin_trocear = permalinks_de_rowids_lote(&conn, &ids).expect("consulta sin trocear");
        let troceado = permalinks_de_rowids(&conn, &ids).expect("consulta troceada");
        assert_eq!(sin_trocear.len(), 650);
        assert_eq!(sin_trocear, troceado);
    }
}
```

- [ ] **Step 2: Correr el test y verlo fallar por la razón esperada**

Run: `cd engine && cargo test --release --locked trocea_por_encima_del_limite_de_placeholders_de_sqlite -- --nocapture`
Expected: FAIL con un panic que contiene `too many SQL variables` (la
sentencia `SELECT id, permalink FROM trozos WHERE id IN (?,?,...,?)` con
40.000 placeholders falla al `prepare()`, antes de bindear nada).

- [ ] **Step 3: Implementación — trocear en lotes de 500**

Reemplaza `permalinks_de_rowids`:

old_string:
```rust
/// Resuelve `permalink` para un conjunto concreto de rowids de `trozos`
/// (H29): reemplaza el `SELECT id, permalink FROM trozos` completo que
/// pagaba una tabla entera por consulta cuando el KNN solo necesitaba unas
/// decenas de filas. `rowids` vacío ⇒ mapa vacío sin tocar la DB.
fn permalinks_de_rowids(
    conn: &rusqlite::Connection,
    rowids: &[i64],
) -> Result<HashMap<i64, String>> {
    if rowids.is_empty() {
        return Ok(HashMap::new());
    }
    let placeholders = std::iter::repeat_n("?", rowids.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!("SELECT id, permalink FROM trozos WHERE id IN ({placeholders})");
    let mut stmt = conn
        .prepare(&sql)
        .context("preparar permalinks por rowid")?;
    let params_dyn: Vec<&dyn rusqlite::ToSql> =
        rowids.iter().map(|r| r as &dyn rusqlite::ToSql).collect();
    stmt.query_map(params_dyn.as_slice(), |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
    })?
    .collect::<rusqlite::Result<_>>()
    .context("leer permalinks de trozos por rowid")
}
```

new_string:
```rust
/// Como máximo `LOTE_PERMALINKS` placeholders por consulta — el límite real
/// de variables de SQLite bundled es 32.766 (medido empíricamente contra
/// rusqlite 0.40.1/libsqlite3-sys 0.38.1; NO son los 999 de versiones de
/// sqlite3 anteriores a 3.32.0). 500 deja margen amplio frente a los dos
/// límites conocidos (32.766 de SQLite, y `K_MIN`/`limite·K_FACTOR_INICIAL`
/// de `busca_vector_con_embedding`) sin volver a tocar esta constante si el
/// corpus crece.
const LOTE_PERMALINKS: usize = 500;

/// Resuelve `permalink` para un conjunto concreto de rowids de `trozos`
/// (H29): reemplaza el `SELECT id, permalink FROM trozos` completo que
/// pagaba una tabla entera por consulta cuando el KNN solo necesitaba unas
/// decenas de filas. `rowids` vacío ⇒ mapa vacío sin tocar la DB.
///
/// Campaña L Task 2 (backlog «Techos de escala»): trocea `rowids` en lotes
/// de `LOTE_PERMALINKS` — sin trocear, `exo search --type vector --limit
/// 5000` sobre una KB con más de 32.766 vectores totales revienta "too many
/// SQL variables" (el bucle de `busca_vector_con_embedding` pide
/// `k = min(total_vectores, limite·K_FACTOR_INICIAL)`, y con `--limit 5000`
/// eso es `min(total_vectores, 40000)`).
fn permalinks_de_rowids(
    conn: &rusqlite::Connection,
    rowids: &[i64],
) -> Result<HashMap<i64, String>> {
    if rowids.is_empty() {
        return Ok(HashMap::new());
    }
    let mut mapa = HashMap::with_capacity(rowids.len());
    for lote in rowids.chunks(LOTE_PERMALINKS) {
        mapa.extend(permalinks_de_rowids_lote(conn, lote)?);
    }
    Ok(mapa)
}

/// Un solo `IN (...)` — asume que el llamador (`permalinks_de_rowids`) ya
/// troceó `rowids` bajo el límite de placeholders de SQLite.
fn permalinks_de_rowids_lote(
    conn: &rusqlite::Connection,
    rowids: &[i64],
) -> Result<HashMap<i64, String>> {
    let placeholders = std::iter::repeat_n("?", rowids.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!("SELECT id, permalink FROM trozos WHERE id IN ({placeholders})");
    let mut stmt = conn
        .prepare(&sql)
        .context("preparar permalinks por rowid")?;
    let params_dyn: Vec<&dyn rusqlite::ToSql> =
        rowids.iter().map(|r| r as &dyn rusqlite::ToSql).collect();
    stmt.query_map(params_dyn.as_slice(), |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
    })?
    .collect::<rusqlite::Result<_>>()
    .context("leer permalinks de trozos por rowid")
}
```

- [ ] **Step 4: Correr los dos tests nuevos y verlos pasar**

Run: `cd engine && cargo test --release --locked tests_permalinks_lote -- --nocapture`
Expected: los dos, PASS
(`trocea_por_encima_del_limite_de_placeholders_de_sqlite` y
`union_de_lotes_coincide_con_una_sola_consulta_sin_trocear`).

- [ ] **Step 5: No regresión — la suite de KNN por consulta (equivalencia exacta)**

Run: `cd engine && cargo test --release --locked tests_knn_por_consulta -- --nocapture`
Expected: todos verdes, incluido
`equivalencia_exacta_por_encima_del_tope_vec0` (que ya ejercita
`permalinks_de_rowids` con corpus > 4096 — con el troceo de 500 sigue dando
exactamente el mismo resultado que la referencia exhaustiva, cubierto por
el test de equivalencia existente).

Run: `cd engine && cargo test --release --locked buscador -- --nocapture`
Expected: toda la suite de `buscador.rs` verde.

- [ ] **Step 6: Commit**

```bash
git add engine/src/buscador.rs
git commit -m "fix(l, permalinks-lote): permalinks_de_rowids trocea el IN(...) bajo el límite de placeholders de SQLite"
```

---

### Task 3: `test-docs-vivos.sh` check (e) gatea también `plugins/exo/README.md`

**Lane:** mecánica. **Depende de:** nada.

**Files:**
- Modify: `scripts/test-docs-vivos.sh`

**Interfaces:** ninguna (script bash, sin API Rust).

**Contexto verificado en HEAD:** `plugins/exo/README.md:58` lleva su propia
tabla `| Reflejo | Evento | Fichero | Qué hace | Abstención |` (cinco
columnas) con las mismas diez filas que `README.md` (tres columnas). El
check (e) actual solo mira `README.md` porque su patrón `awk` ancla con `$`
justo después de `Fichero |` (`/^\| Reflejo \| Evento \| Fichero \|$/`),
así que NUNCA hace match contra la cabecera de cinco columnas de
`plugins/exo/README.md` — ese fichero queda sin gate.

- [ ] **Step 1: RED — demostrar el hueco actual sobre un fixture en `/tmp`,
  sin tocar el README real**

```bash
FIXTURE="$(mktemp -d)"
git -C "$FIXTURE" init -q
mkdir -p "$FIXTURE/docs" "$FIXTURE/plugins/exo/hooks" "$FIXTURE/plugins/exo/.claude-plugin" "$FIXTURE/engine/src"
: > "$FIXTURE/docs/arquitectura.md"
: > "$FIXTURE/docs/instalacion.md"
: > "$FIXTURE/engine/src/main.rs"
echo 'version = "0.0.0"' > "$FIXTURE/engine/Cargo.toml"
echo '{"version":"0.0.0"}' > "$FIXTURE/plugins/exo/.claude-plugin/plugin.json"
cat > "$FIXTURE/plugins/exo/hooks/hooks.json" <<'JSON'
{"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"a"},{"type":"command","command":"b"},{"type":"command","command":"c"}]}]}}
JSON
cat > "$FIXTURE/README.md" <<'MD'
# fixture

## Hooks

| Reflejo | Evento | Fichero |
|---|---|---|
| a | Evt | scripts/a.sh |
| b | Evt | scripts/b.sh |
| c | Evt | scripts/c.sh |
MD
# A PROPÓSITO solo dos filas (faltan una respecto a hooks.json = 3):
cat > "$FIXTURE/plugins/exo/README.md" <<'MD'
# fixture plugin

## Hooks

| Reflejo | Evento | Fichero | Qué hace | Abstención |
|---|---|---|---|---|
| a | Evt | scripts/a.sh | hace a | — |
| b | Evt | scripts/b.sh | hace b | — |
MD

(cd "$FIXTURE" && bash /home/paul/Documentos/proyectos/exo/scripts/test-docs-vivos.sh; echo "exit=$?")
```

Expected: `exit=0` con el mensaje `[OK] test-docs-vivos: ...` — **falso
verde**: el script de HEAD nunca mira `plugins/exo/README.md`, así que la
fila que falta (2 de 3) pasa sin que nada lo detecte. Este es el hueco que
esta task cierra. Guarda `$FIXTURE` (no lo borres): el Step 3 reutiliza el
mismo fixture roto para demostrar que el script arreglado SÍ lo caza.

- [ ] **Step 2: Extender el check (e) a los dos ficheros**

Actualiza el comentario de cabecera del script:

old_string:
```bash
#   (e) la tabla de hooks de README.md (`| Reflejo | Evento | Fichero |`)
#       tiene tantas filas como `hooks/hooks.json` cablea de verdad (I1,
#       review final 2026-09-16: «nueve» en el README, diez en el JSON).
```

new_string:
```bash
#   (e) la tabla de hooks (`| Reflejo | Evento | Fichero |...`) tiene tantas
#       filas como `hooks/hooks.json` cablea de verdad — en LOS DOS
#       ficheros que la llevan, `README.md` y `plugins/exo/README.md` (I1,
#       review final 2026-09-16: «nueve» en el README, diez en el JSON;
#       campaña L Task 3: `plugins/exo/README.md` tenía la misma tabla sin
#       gate — su cabecera lleva dos columnas más, `Qué hace` y
#       `Abstención`, así que el patrón ya no puede anclar con `$`).
```

Reemplaza el bloque del check (e):

old_string:
```bash
# --- (e) La tabla de hooks del README tiene tantas filas como hooks reales -
# I1 (review final, 2026-09-16): el README llegó a decir «nueve hooks» con
# `hooks.json` cableando diez — nadie lo comprobó hasta la review. Cuenta
# filas de la tabla `| Reflejo | Evento | Fichero |` de README.md (hasta la
# primera línea que ya no empieza por `|`) y la compara contra el cableado
# vivo.
HOOKS_REAL="$(jq -r '[.hooks[]?[]?.hooks[]?] | length' plugins/exo/hooks/hooks.json | tr -d '\r')"
FILAS_TABLA="$(awk '
  /^\| Reflejo \| Evento \| Fichero \|$/ { en_tabla = 1; next }
  en_tabla && /^\|---/ { next }
  en_tabla && /^\|/ { n++; next }
  en_tabla { exit }
  END { print n + 0 }
' README.md)"
if [ "$FILAS_TABLA" -ne "$HOOKS_REAL" ]; then
  echo "[FAIL] README.md: la tabla de hooks tiene $FILAS_TABLA fila(s) pero plugins/exo/hooks/hooks.json cablea $HOOKS_REAL" >&2
  FALLOS=1
fi

if [ "$FALLOS" -eq 0 ]; then
  echo "[OK] test-docs-vivos: README.md/docs/{arquitectura,instalacion}.md sin frases muertas, subcomandos inventados, versiones huérfanas, enlaces rotos ni tabla de hooks desfasada"
fi
exit "$FALLOS"
```

new_string:
```bash
# --- (e) Las tablas de hooks (README.md Y plugins/exo/README.md) tienen ---
#         tantas filas como hooks reales ---------------------------------
# I1 (review final, 2026-09-16): el README llegó a decir «nueve hooks» con
# `hooks.json` cableando diez — nadie lo comprobó hasta la review. Campaña L
# Task 3: `plugins/exo/README.md` lleva la MISMA tabla (con dos columnas
# extra, `Qué hace` y `Abstención`) y no tenía gate — el patrón ya no ancla
# con `$` al final de la cabecera a propósito, para que haga match con las
# dos formas. Cuenta filas (hasta la primera línea que ya no empieza por
# `|`) en cada fichero y la compara contra el cableado vivo.
HOOKS_REAL="$(jq -r '[.hooks[]?[]?.hooks[]?] | length' plugins/exo/hooks/hooks.json | tr -d '\r')"
for TABLA_DOC in README.md plugins/exo/README.md; do
  FILAS_TABLA="$(awk '
    /^\| Reflejo \| Evento \| Fichero \|/ { en_tabla = 1; next }
    en_tabla && /^\|---/ { next }
    en_tabla && /^\|/ { n++; next }
    en_tabla { exit }
    END { print n + 0 }
  ' "$TABLA_DOC")"
  if [ "$FILAS_TABLA" -ne "$HOOKS_REAL" ]; then
    echo "[FAIL] $TABLA_DOC: la tabla de hooks tiene $FILAS_TABLA fila(s) pero plugins/exo/hooks/hooks.json cablea $HOOKS_REAL" >&2
    FALLOS=1
  fi
done

if [ "$FALLOS" -eq 0 ]; then
  echo "[OK] test-docs-vivos: README.md/docs/{arquitectura,instalacion}.md sin frases muertas, subcomandos inventados, versiones huérfanas, enlaces rotos ni tablas de hooks desfasadas"
fi
exit "$FALLOS"
```

- [ ] **Step 3: GREEN — el mismo fixture roto del Step 1, ahora sí lo caza**

```bash
(cd "$FIXTURE" && bash /home/paul/Documentos/proyectos/exo/scripts/test-docs-vivos.sh; echo "exit=$?")
```

Expected: `exit=1` con
`[FAIL] plugins/exo/README.md: la tabla de hooks tiene 2 fila(s) pero plugins/exo/hooks/hooks.json cablea 3`.

- [ ] **Step 4: Confirmar que arreglar el fixture también lo deja verde (no
  falso positivo)**

```bash
cat >> "$FIXTURE/plugins/exo/README.md" <<'MD'
| c | Evt | scripts/c.sh | hace c | — |
MD
(cd "$FIXTURE" && bash /home/paul/Documentos/proyectos/exo/scripts/test-docs-vivos.sh; echo "exit=$?")
rm -rf "$FIXTURE"
```

Expected: `exit=0` con `[OK] test-docs-vivos: ...`.

- [ ] **Step 5: Correr el gate arreglado contra el repo real**

Run: `bash scripts/test-docs-vivos.sh`
Expected: `[OK] test-docs-vivos: README.md/docs/{arquitectura,instalacion}.md sin frases muertas, subcomandos inventados, versiones huérfanas, enlaces rotos ni tablas de hooks desfasadas`,
exit 0 (README.md y plugins/exo/README.md ya tienen las diez filas
correctas en HEAD — esta task solo añade el gate que faltaba, no corrige
ninguna tabla).

- [ ] **Step 6: Commit**

```bash
git add scripts/test-docs-vivos.sh
git commit -m "fix(l, docs-vivos): el check (e) gatea también la tabla de hooks de plugins/exo/README.md"
```

---

### Task 4: `cd "$(git rev-parse --show-toplevel)"` sin guarda contra sustitución vacía en tres gates

**Lane:** mecánica. **Depende de:** nada. Mismo patrón que la campaña F
aplicó en `test-hooks-json.sh`/`test-shellcheck.sh` (commit `960a319`,
"`fix(f, gates): test-hooks-json y test-shellcheck tampoco pueden quedarse
en un cd "" silencioso`") — esta task cierra los tres gates restantes que
ese commit no tocó (`docs/backlog.md`, ítem "(NUEVO, campaña G, 2026-09-16)
El `cd "$(git rev-parse --show-toplevel)"` sin guarda contra una
sustitución vacía sigue en tres gates de CI", `~backlog:1691` — cita
correcta, coincide con HEAD).

**Files:**
- Modify: `scripts/test-rutas-personales.sh`
- Modify: `scripts/test-exec-bit.sh`
- Modify: `scripts/test-versiones.sh`

**Interfaces:** ninguna (scripts bash).

**Medición previa (evidencia real, hecha hoy sobre HEAD, con un `git` falso
que simula `rev-parse --show-toplevel` devolviendo cadena vacía):**

```bash
REAL_GIT="$(command -v git)"
FAKE="$(mktemp -d)"
cat > "$FAKE/git" <<EOF
#!/usr/bin/env bash
if [ "\$1" = "rev-parse" ] && [ "\$2" = "--show-toplevel" ]; then
  echo ""
  exit 0
fi
exec "$REAL_GIT" "\$@"
EOF
chmod +x "$FAKE/git"
mkdir -p /tmp/no-git-here && cd /tmp/no-git-here
PATH="$FAKE:$PATH" bash /home/paul/Documentos/proyectos/exo/scripts/test-rutas-personales.sh; echo "exit=$?"
PATH="$FAKE:$PATH" bash /home/paul/Documentos/proyectos/exo/scripts/test-exec-bit.sh; echo "exit=$?"
PATH="$FAKE:$PATH" bash /home/paul/Documentos/proyectos/exo/scripts/test-versiones.sh; echo "exit=$?"
```

Salida real (ANTES del fix, HEAD de hoy):
```
fatal: no es un repositorio git (ni ninguno de los directorios superiores): .git
test-rutas-personales: no se encontró ningún script — el recorrido está roto
exit=1
fatal: no es un repositorio git (ni ninguno de los directorios superiores): .git
[FAIL] no se encontró ningún script bajo plugins/ — el recorrido está roto
exit=1
jq: error: Could not open file plugins/exo/.claude-plugin/plugin.json: No existe el archivo o el directorio
exit=2
```

Los tres SALEN distintos de 0 hoy — no es un "pasa en silencio, exit 0".
Pero el diagnóstico es **engañoso**: "el recorrido está roto" y el error de
`jq` sobre un fichero que falta no dicen NADA de que la causa real es
`git rev-parse --show-toplevel` devolviendo vacío — un desarrollador que vea
ese mensaje va a sospechar del propio gate o de una `git`
desconfigurada, no de un `cd ""` silencioso. Fix: capturar la raíz,
comprobar que no está vacía, dar el diagnóstico correcto y SOLO entonces
moverse — igual que `960a319`.

- [ ] **Step 1: `test-rutas-personales.sh`**

old_string:
```bash
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

PATRON='(/home/[A-Za-z0-9_.-]+|/Users/[A-Za-z0-9_.-]+|C:[\\/]Users[\\/][A-Za-z0-9_.-]+)'
```

new_string:
```bash
set -uo pipefail
# Campaña L Task 4 (backlog:1691, mismo fix que 960a319 en test-hooks-json.sh
# y test-shellcheck.sh): `cd "$(git rev-parse --show-toplevel)" || exit 1`
# directo tiene un fallo silencioso — si la sustitución sale vacía, `cd ""`
# devuelve 0 sin moverse y el `|| exit 1` nunca dispara. Captura la raíz, la
# comprueba y entonces se mueve.
RAIZ="$(git rev-parse --show-toplevel)" || exit 1
if [ -z "$RAIZ" ]; then
  echo "test-rutas-personales: git rev-parse --show-toplevel no devolvió nada" >&2
  exit 1
fi
cd "$RAIZ" || exit 1

PATRON='(/home/[A-Za-z0-9_.-]+|/Users/[A-Za-z0-9_.-]+|C:[\\/]Users[\\/][A-Za-z0-9_.-]+)'
```

- [ ] **Step 2: `test-exec-bit.sh`**

old_string:
```bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

malos=""
```

new_string:
```bash
set -euo pipefail

# Campaña L Task 4 (backlog:1691, mismo fix que 960a319): `cd
# "$(git rev-parse --show-toplevel)"` directo tiene un fallo silencioso — si
# la sustitución sale vacía, `cd ""` devuelve 0 sin moverse, y bajo
# `set -e` eso NO aborta (el exit code de `cd ""` es 0). Captura la raíz, la
# comprueba y entonces se mueve.
RAIZ="$(git rev-parse --show-toplevel)" || exit 1
if [ -z "$RAIZ" ]; then
  echo "test-exec-bit: git rev-parse --show-toplevel no devolvió nada" >&2
  exit 1
fi
cd "$RAIZ" || exit 1

malos=""
```

- [ ] **Step 3: `test-versiones.sh`**

old_string:
```bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

fallos=0
```

new_string:
```bash
set -euo pipefail
# Campaña L Task 4 (backlog:1691, mismo fix que 960a319): `cd
# "$(git rev-parse --show-toplevel)"` directo tiene un fallo silencioso — si
# la sustitución sale vacía, `cd ""` devuelve 0 sin moverse y bajo
# `set -e` eso NO aborta. Captura la raíz, la comprueba y entonces se mueve.
RAIZ="$(git rev-parse --show-toplevel)" || exit 1
if [ -z "$RAIZ" ]; then
  echo "test-versiones: git rev-parse --show-toplevel no devolvió nada" >&2
  exit 1
fi
cd "$RAIZ" || exit 1

fallos=0
```

- [ ] **Step 4: Verificar el diagnóstico correcto con el mismo `git` falso
  del Step previo**

```bash
mkdir -p /tmp/no-git-here && cd /tmp/no-git-here
PATH="$FAKE:$PATH" bash /home/paul/Documentos/proyectos/exo/scripts/test-rutas-personales.sh; echo "exit=$?"
PATH="$FAKE:$PATH" bash /home/paul/Documentos/proyectos/exo/scripts/test-exec-bit.sh; echo "exit=$?"
PATH="$FAKE:$PATH" bash /home/paul/Documentos/proyectos/exo/scripts/test-versiones.sh; echo "exit=$?"
```

Expected (verificado hoy con la implementación de este Step, aplicada
sobre una copia en `/tmp`):
```
test-rutas-personales: git rev-parse --show-toplevel no devolvió nada
exit=1
test-exec-bit: git rev-parse --show-toplevel no devolvió nada
exit=1
test-versiones: git rev-parse --show-toplevel no devolvió nada
exit=1
```
Los tres siguen saliendo `1`, pero ahora el mensaje señala la causa real.

- [ ] **Step 5: Confirmar que los tres siguen verdes contra el repo real**

```bash
git -C /home/paul/Documentos/proyectos/exo rev-parse --show-toplevel
bash scripts/test-rutas-personales.sh
bash scripts/test-exec-bit.sh
bash scripts/test-versiones.sh
```

Expected: los tres `[OK]` con exit 0 (`test-rutas-personales: OK — N
scripts sin rutas personales`; `[OK] N scripts bajo plugins/ en 100755`;
`[OK] engine 0.2.0 · plugin 1.2.0 · ENGINE_MIN 0.1.0`).

- [ ] **Step 6: Commit**

```bash
git add scripts/test-rutas-personales.sh scripts/test-exec-bit.sh scripts/test-versiones.sh
git commit -m "fix(l, cd-toplevel-guard): tres gates más no se quedan en un cd \"\" silencioso"
```

---

### Task 5: Runbook de M5b — desinstalar basic-memory (checklist C10), en seco

**Lane:** mecánica (documental) + comprobaciones read-only. **Depende de:**
nada. **La fábrica NO desinstala nada** — línea roja, acción de Paul.

**Files:**
- Create: `docs/superpowers/runbooks/2026-09-19-m5b-desinstalar-basic-memory.md`

**Interfaces:** ninguna (documento).

**Contexto verificado hoy:**
- El MCP `basic-memory` está registrado en `~/.claude.json` →
  `mcpServers.basic-memory`, comando `/home/paul/.local/bin/uvx basic-memory mcp`,
  y ya sale como **"Disabled for this project"** en `claude mcp list` para
  este repo.
- `~/.basic-memory/config.json` → `default_project: "wisdom-paul"`, `path:
  "/home/paul/Documentos/proyectos/wisdom-paul"` — la KB real de Paul, la
  misma que resuelve `exo`.
- `~/.basic-memory/` ocupa **1007 MB** en disco (su propio índice/caché de
  embeddings, independiente de la KB); `~/basic-memory/` (sin el punto) está
  **vacío** (residuo).
- `wisdom-paul` (la KB) está limpia y sin commits sin pushear ahora mismo
  (`git -C /home/paul/Documentos/proyectos/wisdom-paul status --porcelain`
  vacío; `git -C ... log @{u}..HEAD` vacío) — **snapshot de hoy, Paul debe
  re-verificarlo justo antes de desinstalar**, no asumir que sigue igual.

- [ ] **Step 1: Escribir el runbook**

```markdown
# 2026-09-19 — M5b: desinstalar basic-memory (checklist C10)

> Checklist de cierre de `docs/superpowers/plans/2026-08-17-cierre-exo-m2-a-m5b.md`
> §Campaña 10 (M5b). Decisión de Paul del 2026-09-19 (#6,
> `docs/superpowers/consultas/2026-09-15-campanas/propuesta.md` §7): *"sí,
> tras el checklist C10. La fábrica prepara y corre en seco las
> comprobaciones automatizables (este runbook, campaña L); la desinstalación
> la ejecuta Paul (línea roja)."* Incluye también la decisión #15 de la
> misma sesión (slug canónico), que cierra el punto #8 del gate M4
> (`docs/backlog.md`, ítem «Barrer los hallazgos vivos del gate M4»,
> ~`backlog:857-861`).
>
> **Qué NO hace este runbook:** no desinstala basic-memory, no borra
> `~/.basic-memory/`, no toca `~/.claude.json`. Todo eso es de Paul,
> ejecutado a mano, después de leer las comprobaciones en seco de abajo y
> decidir que el checklist está satisfecho.

## Checklist C10 (del plan de cierre, con el estado real de hoy)

1. El hook de recall no llama a basic-memory por ninguna vía (ni en el
   FALLBACK). → **Comprobación A** abajo.
2. Ningún hook conserva matchers `mcp__basic-memory__*` vivos. →
   **Comprobación D** abajo.
3. ~~kbx apunta al índice del engine y `consumed` está actualizado.~~
   **CADUCADO**: kbx dejó de ser dependencia de nada desde la campaña D
   (`docs/backlog.md`, "cutover kbx→exo", commits `0051638`/`aa82f95`;
   ver también `docs/backlog.md:1143`, "kbx dejó de ser dependencia de
   nada"). No hay nada que verificar aquí.
4. `/document` y `/distill` corren end-to-end sin el MCP (nombres
   actuales de las skills que el plan de cierre llamaba `/documenta` y
   `/consolida`). → **Comprobación C** abajo.
5. exo tiene config propia (M5a-02). → **YA CERRADO** (`docs/backlog.md`,
   ítem "M5a-02 config propia: cerrado el 2026-08-26" — `~/.exo/config.toml`,
   precedencia `flag > env > config > error accionable`, sin fallback a
   basic-memory salvo `exo init --from-basic-memory`, explícito).
6. La KB está commiteada y pusheada (el plan de cierre dice "en kb-demo";
   hoy el nombre real es `wisdom-paul`). → **Comprobación E** abajo.

## Comprobación A — el hook de recall no llama a basic-memory (ni en fallback)

```bash
cd /home/paul/Documentos/proyectos/exo
git grep -in "mcp__basic" -- plugins/ engine/
echo "exit=$?"
```

Output real de hoy: sin líneas, `exit=1` (grep sin matches). Ninguna
mención viva de un tool MCP de basic-memory en plugin ni engine.

## Comprobación B — la única lectura de basic-memory que sobrevive es la explícita

```bash
cd /home/paul/Documentos/proyectos/exo
grep -rn "basic-memory/config.json" engine/src/ | grep -v inicia.rs
```

Output real de hoy (dos líneas, ambas COMENTARIOS de diseño, no código que
se ejecute):
```
engine/src/config.rs:3://! Sustituye la lectura RO de `~/.basic-memory/config.json` que hacían
engine/src/main.rs:129:    /// Toma raíz, nombre y embeddings de `~/.basic-memory/config.json`.
```
Las dos son doc-comments (`//!`/`///`) que EXPLICAN el diseño (uno describe
qué sustituyó `config.rs`, el otro documenta el flag `--from-basic-memory`
de `exo init`) — ninguna es una llamada real. La única lectura de código que
toca esa ruta sigue siendo `engine/src/inicia.rs`, exclusiva de
`exo init --from-basic-memory` (explícita, opt-in, nunca automática).

## Comprobación C — `/document` y `/distill` no invocan tools MCP de basic-memory

```bash
cd /home/paul/Documentos/proyectos/exo
grep -in "mcp__basic\|basic-memory" \
  plugins/exo/skills/document/SKILL.md \
  plugins/exo/skills/distill/SKILL.md \
  plugins/exo/skills/distill/chequeos.md
echo "exit=$?"
```

Output real de hoy: sin líneas, `exit=1`. Ninguna de las tres skills
menciona basic-memory ni sus tools MCP — las dos ya migraron por completo
al engine `exo` (`exo write`/`exo search`/`exo recall`, vía el binario, no
vía MCP).

## Comprobación D — `hooks.json` sin matchers `mcp__basic-memory__*`

```bash
cd /home/paul/Documentos/proyectos/exo
grep -n "basic-memory\|mcp__basic" plugins/exo/hooks/hooks.json
echo "exit=$?"
```

Output real de hoy: sin líneas, `exit=1`. Los diez hooks de producción
(`jq '[.hooks[]?[]?.hooks[]?] | length'` → `10`, verificado hoy) citan solo
scripts de `${CLAUDE_PLUGIN_ROOT}/scripts/`, ninguno un matcher de tool MCP.

## Comprobación E — la KB está commiteada y pusheada (snapshot de hoy)

```bash
git -C /home/paul/Documentos/proyectos/wisdom-paul status --porcelain
git -C /home/paul/Documentos/proyectos/wisdom-paul rev-parse --abbrev-ref --symbolic-full-name @{u}
git -C /home/paul/Documentos/proyectos/wisdom-paul log @{u}..HEAD --oneline
```

Output real de hoy: `status --porcelain` vacío (árbol limpio), upstream
`origin/main`, `log @{u}..HEAD` vacío (nada sin pushear). **Paul: repite
esta comprobación justo antes de desinstalar** — es un snapshot del momento
de escribir este runbook, no una garantía permanente.

## Decisión #15 (slug canónico) — cierra el #8 del gate M4

`docs/backlog.md`, ítem «Barrer los hallazgos vivos del gate M4»,
sub-ítem **#8 [baja]**: divergencia de slug medida **19/127** entre el
generador de permalinks de `exo` y el de basic-memory (`_` conservado en
bitácoras rotadas, CamelCase separado, `§`→`ss`). Decisión de Paul,
2026-09-19 (#15, `propuesta.md` §7): **el slug de exo es canónico** — la
divergencia 19/127 con basic-memory queda **aceptada por escrito**, no se
persigue paridad. Con esto el #8 del gate M4 queda cerrado (ver Task 6,
que marca el ítem correspondiente en `docs/backlog.md`).

## Pasos manuales de Paul — desinstalación

Ejecutar SOLO cuando las cinco comprobaciones de arriba estén en el estado
esperado el día de la desinstalación (repetirlas, no fiarse de este
runbook si pasó tiempo):

1. **Quitar el servidor MCP** de la config de Claude Code:
   `claude mcp remove basic-memory` (o edición manual de
   `~/.claude.json`, clave `mcpServers.basic-memory` — hoy ya está
   `"disabled": true` para el proyecto `exo`, pero sigue registrado
   globalmente).
2. **Confirmar que nada más lo referencia**: `claude mcp list` no debe
   listar `basic-memory` tras el paso 1.
3. **Liberar el caché/índice de basic-memory** (~1 GB medido hoy en
   `~/.basic-memory/`, independiente de la KB en sí):
   `rm -rf ~/.basic-memory` (revisar antes con `du -sh ~/.basic-memory` que
   la cifra sigue siendo del orden de lo esperado, no un false-negative de
   este runbook).
4. **Limpiar el residuo vacío**: `rmdir ~/basic-memory` si sigue vacío
   (`ls -la ~/basic-memory` antes de borrar, por si alguna vez se pobló).
5. **NO tocar** `/home/paul/Documentos/proyectos/wisdom-paul` — esa es la
   KB, vive fuera de basic-memory y la sigue usando `exo`.
6. **Verificación post-desinstalación**: `claude mcp list` sin
   `basic-memory`; sesión nueva de Claude Code en el repo `exo` arranca
   igual (el hook `exo-recall.sh` no lo usa, ya lo confirmó la
   Comprobación A); `exo doctor` sigue en verde (no depende de
   basic-memory).

## Rollback

Si algo falla tras el paso 3 (p. ej. se necesita releer una nota vieja que
solo basic-memory tenía indexada — no debería pasar, la KB en markdown
sigue intacta en `wisdom-paul/`, pero por si el índice de basic-memory
guardaba algo que el markdown no):

1. Los datos NUNCA vivieron solo en `~/.basic-memory/` — es un índice
   derivado de `wisdom-paul/` (markdown, en git). Nada se pierde al borrar
   el índice.
2. Para volver a tener el MCP disponible: `uvx basic-memory mcp` sigue
   funcionando sin reinstalar nada persistente (`uvx` descarga/cachea bajo
   `uv` en caliente); solo hace falta re-añadirlo con
   `claude mcp add basic-memory -- uvx basic-memory mcp` (o restaurar la
   entrada en `~/.claude.json`) y dejar que regenere su índice desde
   `wisdom-paul/` (`basic-memory sync` o equivalente, fuera de alcance de
   `exo`).
3. Si se borró `~/.basic-memory/config.json` por error: recrearlo con el
   `default_project`/`path` de hoy (documentado arriba, en "Contexto
   verificado") es suficiente para que vuelva a apuntar a `wisdom-paul/`.
```

- [ ] **Step 2: Verificar que las cinco comprobaciones del runbook dan
  exactamente el output documentado**

Run cada bloque de comando de las Comprobaciones A-E tal cual está en el
runbook. Expected: el output de cada una coincide byte a byte con el
"Output real de hoy" citado (si no coincide, corrige el runbook para que
cite el output REAL medido en el momento de ejecutar esta task, no el de
este plan — el plan es de hoy 2026-09-19, la task puede ejecutarse después).

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/runbooks/2026-09-19-m5b-desinstalar-basic-memory.md
git commit -m "docs(l, runbook-m5b): checklist C10 en seco — basic-memory NO se desinstala desde la fábrica"
```

---

### Task 6: `docs/backlog.md` — barrido de bookkeeping y decisiones de Paul del 2026-09-19

**Lane:** documental. **Depende de:** nada (independiente del código de las
Tasks 1-5; se hace en paralelo). **Comparte fichero con la campaña I** — ver
Global Constraints: cada `old_string` de esta task cita una frase única del
párrafo, re-anclable por texto si I ya movió líneas al mergear antes.

**Files:**
- Modify: `docs/backlog.md`
- Modify: `docs/superpowers/runbooks/2026-09-15-release-v0.2.0.md`

**Interfaces:** ninguna (documento).

**Verificación previa de los 5 ítems del barrido** (todos confirmados
cerrados en código, con `git grep`/lectura directa contra HEAD — ninguno se
excluye):

| Ítem (línea real en HEAD) | Verificación hecha hoy | Cierra con |
|---|---|---|
| `:456` «exo genérico» | `git grep -c Paul -- plugins/exo/` → solo `plugin.json:1` (`author.name`); `git grep -n kb-demo -- engine/` → 4 hits, todos comentarios/guard, ningún fixture hardcodeado | F Task 8 (`7aba784`) + G Task 10 (`5a71911`) |
| `:1438` bash inline `run:` sin gate | `scripts/test-shellcheck.sh` ya extrae y analiza los bloques `run: \|` de `.github/workflows/*.yml` (leído en HEAD) | F Task 4 (`9753441`+`2cf62e2`) |
| `:1476` job `lint` mal nombrado | `.github/workflows/ci.yml:34-35`: `static-checks` / "checks estáticos" (leído en HEAD) | F Task 7 (`8942969`) |
| `:1536` idioma mezclado (identificadores) | `docs/arquitectura.md:345-350` lleva el párrafo "Idioma de los identificadores de código..." (leído en HEAD) | F Task 8 (`7aba784`) |
| `:1600` nombres y ubicaciones | `reports/` ya no existe en la raíz (`ls reports/` → no existe); `docs/superpowers/` resuelto "se queda" desde `2294349` | B H26 (`7ee58ba`) + B D5=b (`2294349`), verificado por F Task 9 |

Las líneas citadas por el brief (`:456`, `:1438`, `:1476`, `:1536`, `:1600`)
**coinciden con HEAD** — no hicieron falta correcciones de línea para estos
cinco.

- [ ] **Step 1: `:456` «exo genérico» → `[x]`, con la mitad que cerró G**

old_string:
```
- [ ] **(revisión 2026-09-04) «exo genérico» sigue siendo el plugin de Paul
```

new_string:
```
- [x] **(revisión 2026-09-04) «exo genérico» sigue siendo el plugin de Paul
```

Y, al final del mismo ítem:

old_string:
```
  **Sigue abierta** la mitad `kb-demo` en
  `engine/tests/` (11 ficheros, no 8 — el conteo del backlog está caducado,
  ver más abajo) — es alcance de G, no de F.
```

new_string:
```
  **Sigue abierta** la mitad `kb-demo` en
  `engine/tests/` (11 ficheros, no 8 — el conteo del backlog está caducado,
  ver más abajo) — es alcance de G, no de F.
  **(campaña G, 2026-09-16, Task 10, commit `5a71911`): cerrada la mitad que
  quedaba.** `kb-demo` → `kb-test` en los 11 ficheros de test del engine (y
  comentarios de `src`). Re-verificado hoy (campaña L, barrido de
  bookkeeping): `git grep -l kb-demo -- engine/` da solo 4 ficheros
  (`lib.rs:103`, `tests/escritor.rs:29`, `tests/help_producto.rs:154`,
  `tests/recall_contenido.rs:108`), todos comentarios históricos o el propio
  guard de `help_producto.rs` que EXIGE que "kb-demo" no aparezca en
  `--help` — ninguno es un fixture hardcodeado. **Ítem cerrado por
  completo**: las cuatro acciones (a-Paul, a-kb-demo, b-kbx, c-release)
  están hechas.
```

- [ ] **Step 2: `:1438` bash inline de `run:` → `[x]`**

old_string:
```
- [ ] **(NUEVO, revisión final campaña B, 2026-09-13) El bash inline de
  `run:` en `.github/workflows/*.yml` no pasa por ningún gate.**
```

new_string:
```
- [x] **(NUEVO, revisión final campaña B, 2026-09-13) El bash inline de
  `run:` en `.github/workflows/*.yml` no pasa por ningún gate.**
```

- [ ] **Step 3: `:1476` job `lint` → `[x]`, con cita a la activación de
  branch protection de hoy**

old_string:
```
- [ ] **(NUEVO, revisión final campaña B, 2026-09-13) El job `lint` de
  `ci.yml` se llama «fmt + clippy» pero ya corre shellcheck y el gate de
  versiones.**
```

new_string:
```
- [x] **(NUEVO, revisión final campaña B, 2026-09-13) El job `lint` de
  `ci.yml` se llama «fmt + clippy» pero ya corre shellcheck y el gate de
  versiones.**
```

Y al final del mismo ítem:

old_string:
```
  alguien actualice la config del repo (fuera de este árbol de código) —
  verificar `required_status_checks` del repo antes de tocarlo. **Acción:**
  o renombrar `lint` a algo que cubra las cuatro cosas («checks estáticos»,
  «lint + gates estáticos») coordinando el cambio de required checks, o
  separar shellcheck y versiones a su propio job con nombre propio.
```

new_string:
```
  alguien actualice la config del repo (fuera de este árbol de código) —
  verificar `required_status_checks` del repo antes de tocarlo. **Acción:**
  o renombrar `lint` a algo que cubra las cuatro cosas («checks estáticos»,
  «lint + gates estáticos») coordinando el cambio de required checks, o
  separar shellcheck y versiones a su propio job con nombre propio.
  **(campaña F, 2026-09-15, Task 7, commit `8942969`): cerrado.** El job se
  renombró a `static-checks` / "checks estáticos"
  (`.github/workflows/ci.yml:34-35`, verificado en HEAD), con los 12
  required checks documentados para cuando Paul activara branch protection.
  **Branch protection ACTIVADA el 2026-09-19** (decisión #8,
  `docs/superpowers/consultas/2026-09-15-campanas/propuesta.md` §7):
  verificado hoy con
  `gh api repos/pguerrerolinares/exo/branches/main/protection` — los 12
  checks exactos de este item en `required_status_checks.contexts`,
  `strict: false`, sin revisión obligatoria (autor único), `enforce_admins:
  false`, force-push y borrado de `main` prohibidos.
```

- [ ] **Step 4: `:1536` idioma mezclado → `[x]`**

old_string:
```
- [ ] **(revisión 2026-09-04) Idioma mezclado sin criterio único.** Medido
```

new_string:
```
- [x] **(revisión 2026-09-04) Idioma mezclado sin criterio único.** Medido
```

- [ ] **Step 5: `:1600` nombres y ubicaciones → `[x]`**

old_string:
```
- [ ] **Nombres y ubicaciones.** `docs/superpowers/` como carpeta de docs del
```

new_string:
```
- [x] **Nombres y ubicaciones.** `docs/superpowers/` como carpeta de docs del
```

- [ ] **Step 6: Decisión #7 (M5a-01 MCP propio) → «no se construye»**

old_string:
```
- [ ] **(pasada de coste 2026-09-09) M5a (MCP propio) se diseñó contra un MCP
  con estado; la revisión 2026-07-28 del spec lo abarató y dejó el diseño sin
  releer.**
```

new_string:
```
- [x] **(pasada de coste 2026-09-09) M5a (MCP propio) se diseñó contra un MCP
  con estado; la revisión 2026-07-28 del spec lo abarató y dejó el diseño sin
  releer.**
```

Y al final del ítem — ancla verificada hoy contra HEAD (el ítem termina en
`backlog:1235`, "modelo los busca.", el siguiente ítem empieza en `:1237`):

old_string:
```
  muerto; (b) `ttlMs`/`cacheScope` sobre `list` toca directamente el item de
  Alta: las respuestas de `list` **son prefijo**, así que cachearlas es la
  misma palanca medida allí; (c) evaluar `defer_loading` para los tools poco
  usados, que los mantiene fuera del prefijo cacheado y solo entran cuando el
  modelo los busca.
```

new_string:
```
  muerto; (b) `ttlMs`/`cacheScope` sobre `list` toca directamente el item de
  Alta: las respuestas de `list` **son prefijo**, así que cachearlas es la
  misma palanca medida allí; (c) evaluar `defer_loading` para los tools poco
  usados, que los mantiene fuera del prefijo cacheado y solo entran cuando el
  modelo los busca.
  **Decisión de Paul, 2026-09-19 (#7, `propuesta.md` §7): no se construye.**
  Se reabre solo si alguien lo echa de menos con un caso concreto. Cierra el
  ítem: la pregunta que planteaba (releer M5a contra el spec sin estado y
  decidir si vale la pena) queda respondida — no.
```

- [ ] **Step 7: Decisión #3 (`budget` vs `cost`) → cierre final**

old_string:
```
  **(campaña F, 2026-09-15): caducado — parcialmente resuelto de hecho.**
  `grep budget docs/arquitectura.md` → vacío (ya no lo lista como
  "planeado"); `docs/instalacion.md:34` lo lista como verbo existente;
  `Comando::Budget` vive en `engine/src/main.rs:68` desde G4b. La colisión
  de nombre se resolvió de hecho: `budget` = bytes de KB. Queda solo el
  nombre de un verbo de coste de tokens hipotético, decisión #3 del paquete
  de Paul del 2026-09-15 (b/c: vive en `evals/` o no se construye hasta que
  haga falta) — fuera de alcance de F.
```

new_string:
```
  **(campaña F, 2026-09-15): caducado — parcialmente resuelto de hecho.**
  `grep budget docs/arquitectura.md` → vacío (ya no lo lista como
  "planeado"); `docs/instalacion.md:34` lo lista como verbo existente;
  `Comando::Budget` vive en `engine/src/main.rs:68` desde G4b. La colisión
  de nombre se resolvió de hecho: `budget` = bytes de KB. Queda solo el
  nombre de un verbo de coste de tokens hipotético, decisión #3 del paquete
  de Paul del 2026-09-15 (b/c: vive en `evals/` o no se construye hasta que
  haga falta) — fuera de alcance de F.
  **Decisión de Paul, 2026-09-19 (#3, `propuesta.md` §7): nada hasta que
  exista `cost`.** Con la decisión #17 (no medir el coste de la inyección
  por ahora), no hay nada que nombrar todavía — `budget` = bytes de KB queda
  sellado. Ítem cerrado.
```

Y flip del checkbox de este mismo ítem — busca su inicio:

old_string:
```
- [ ] **(pasada de coste 2026-09-09) `exo budget` va a colisionar de nombre:
```

new_string:
```
- [x] **(pasada de coste 2026-09-09) `exo budget` va a colisionar de nombre:
```

- [ ] **Step 8: Decisión #17 (coste de la inyección) → «no, por ahora»**

old_string:
```
- [ ] **(pasada de coste 2026-09-09) El bucle de coste de la inyección está
  a un `join` de distancia: el emisor ya loguea los bytes que emite y nadie los
  ha cruzado con lo que cuestan.**
```

new_string:
```
- [x] **(pasada de coste 2026-09-09) El bucle de coste de la inyección está
  a un `join` de distancia: el emisor ya loguea los bytes que emite y nadie los
  ha cruzado con lo que cuestan.**
```

Y al final del ítem (después del párrafo "Aviso de medición: ..."):

old_string:
```
  Y ojo con el diseño best-effort del sink (`|| true`, `2>/dev/null`,
  `_reflex-log.sh:4`): un log ausente es indistinguible de cero disparos, así
  que el harness debe **exigir** el fichero, no tolerar su falta.
```

new_string:
```
  Y ojo con el diseño best-effort del sink (`|| true`, `2>/dev/null`,
  `_reflex-log.sh:4`): un log ausente es indistinguible de cero disparos, así
  que el harness debe **exigir** el fichero, no tolerar su falta.
  **Decisión de Paul, 2026-09-19 (#17, `propuesta.md` §7): no, por ahora.**
  El harness de cruce (a/b/c de la acción original) no se construye hasta
  que haga falta. Ítem cerrado por decisión explícita, no por trabajo hecho.
```

- [ ] **Step 9: Decisión #8 (branch protection) → «ACTIVADA»**

old_string:
```
- [ ] **Hoy el CI no bloquea nada.** Paul decidió explícitamente no proteger
  `main` por ahora — no hay branch protection ni required status checks.
```

new_string:
```
- [x] **Hoy el CI no bloquea nada.** Paul decidió explícitamente no proteger
  `main` por ahora — no hay branch protection ni required status checks.
```

Y al final del ítem:

old_string:
```
  **Lo subsume G5 si adopta esa cadena.** **(2026-09-11: G5b cerró sin adoptarlo — release `v0.1.0` publicada, ver `## Cerrado con evidencia`. La marca queda huérfana: necesita dueño o campaña propia.)**

- [x] **Dos endurecimientos del CI que se decidieron NO aplicar en G5a, y por
```

new_string:
```
  **Lo subsume G5 si adopta esa cadena.** **(2026-09-11: G5b cerró sin adoptarlo — release `v0.1.0` publicada, ver `## Cerrado con evidencia`. La marca queda huérfana: necesita dueño o campaña propia.)**
  **Decisión de Paul, 2026-09-19 (#8, `propuesta.md` §7): ACTIVADA.**
  Ejecutada el mismo día por el orquestador con autorización explícita de
  Paul en sesión — verificado hoy con
  `gh api repos/pguerrerolinares/exo/branches/main/protection`: los 12
  required checks de F Task 7 (verificados contra los check-runs de
  `5efe812`), `strict: false`, sin revisión obligatoria (autor único),
  `enforce_admins: false`, force-push y borrado de `main` prohibidos. Ítem
  cerrado.

- [x] **Dos endurecimientos del CI que se decidieron NO aplicar en G5a, y por
```

- [ ] **Step 10: Decisión #16 (cap de 6.144 B) → «se mantiene»**

old_string:
```
- [ ] **El bloque de arranque va al 96% de su cap, y desborda en silencio.**
```

new_string:
```
- [x] **El bloque de arranque va al 96% de su cap, y desborda en silencio.**
```

El párrafo "(b)/(c)" de este ítem es MUCHO más largo de lo que parece a
primera vista (sigue con dos "Cruce (2026-09-09...)" adicionales antes de
terminar) — ancla verificada hoy contra HEAD, el ítem entero termina en
`backlog:617`, "(a).", el siguiente ítem («El bucle de coste de la
inyección...») empieza en `:619`:

old_string:
```
  Los nueve hooks son never-block por política global
  (`arquitectura.md:320-323`), no por una decisión razonada hook a hook, y la
  asimetría no apunta igual en todos: en la inyección a subagentes abstenerse es
  barato —`DietrichGebert/ponytail` elige ahí fail-open a propósito y lo
  comenta, `hooks/ponytail-subagent.js:31-38`— pero en el bloque de arranque lo
  barato es gritar. No es item nuevo: es el criterio que le falta a la acción
  (a).
```

new_string:
```
  Los nueve hooks son never-block por política global
  (`arquitectura.md:320-323`), no por una decisión razonada hook a hook, y la
  asimetría no apunta igual en todos: en la inyección a subagentes abstenerse es
  barato —`DietrichGebert/ponytail` elige ahí fail-open a propósito y lo
  comenta, `hooks/ponytail-subagent.js:31-38`— pero en el bloque de arranque lo
  barato es gritar. No es item nuevo: es el criterio que le falta a la acción
  (a).
  **Decisión de Paul, 2026-09-19 (#16, `propuesta.md` §7): se mantiene.**
  El cap de 6.144 B no sube; la presión se resuelve con evicción editorial
  de entradas muertas de `core-index` (es índice: no se comprime) hasta
  ≥15% de aire. Trabajo de Paul en la KB `wisdom-paul`, explícitamente NO de
  la fábrica. Cierra las acciones (b) y (c) por decisión, no por código.
```

- [ ] **Step 11: D-4 (`nombre_kb()`) → cierre en la tabla de Estado**

old_string:
```
Abierto: D-4 (permalink `nombre_kb()` vs literal fijo, recomendación ya aplicada en el código, formalmente pendiente de que Paul la zanje) y la acción (a) de «exo genérico» (`Paul`/`kb-demo` por nombres resueltos), fuera de alcance de D |
```

new_string:
```
D-4 (permalink `nombre_kb()` vs literal fijo) **cerrado por decisión de
Paul, 2026-09-19 (#2, `propuesta.md` §7): opción b, `nombre_kb()`** — lo que
ya había en código queda formalizado; y la acción (a) de «exo genérico»
(`Paul`/`kb-demo` por nombres resueltos) cerrada por G (ver Alta, ítem
«exo genérico») |
```

- [ ] **Step 12: Actualizar el runbook de release con la precondición del
  tag `v0.2.0`**

En `docs/superpowers/runbooks/2026-09-15-release-v0.2.0.md`, en el
checklist de máquina Linux:

old_string:
```
- [ ] Poner el tag `v0.2.0` en GitHub (acción de release, línea roja de la
  fábrica: nunca la ejecuta un consultor ni un executor).
```

new_string:
```
- [ ] Poner el tag `v0.2.0` en GitHub (acción de release, línea roja de la
  fábrica: nunca la ejecuta un consultor ni un executor). **Precondición
  añadida el 2026-09-19**: esperar a que la campaña L
  (`docs/superpowers/plans/2026-09-19-campana-l-sueltos-pre-v020.md`) esté
  mergeada a `main` — L arregla la regresión de `exo search` (default
  `hybrid`, de G) contra una DB sin tabla `vectores`. La campaña I no
  bloquea esta release (puede ir en `v0.2.1`).
```

- [ ] **Step 13: Añadir el bloque "Última revisión" de esta campaña**

En la cabecera de `docs/backlog.md`, justo antes del bloque `> Anterior:
**2026-09-16**` (el que hoy encabeza el histórico), inserta un nuevo bloque
`> Última revisión: **2026-09-19**` que resuma esta campaña, y renombra el
bloque que hoy dice `> Última revisión: **2026-09-16**` a `> Anterior:`.
Sigue la MISMA estructura que ya usa el fichero (ver el bloque de G como
plantilla): fecha, campaña, qué cierra con evidencia (los 5 ítems del
barrido + las 6 decisiones de Paul del 2026-09-19 + Tasks 1-4 de este
mismo plan), y qué queda corrigiendo-sin-cerrar si aplica. Redacta este
bloque de forma proporcional al patrón ya usado — no lo dejes en placeholder;
usa los commits reales de las Tasks 1-4 de este plan (los que resulten de
los Steps de commit de arriba) y las citas de los Steps 1-11 de esta misma
Task.

- [ ] **Step 14: Commit**

```bash
git add docs/backlog.md docs/superpowers/runbooks/2026-09-15-release-v0.2.0.md
git commit -m "docs(l, backlog-sync): barrido de bookkeeping (5 ítems) + decisiones de Paul del 2026-09-19 (#2,#3,#7,#8,#16,#17)"
```

---

### Task 7: Verificación final de la rama

**Lane:** verificación. **Depende de:** Tasks 1-6. No modifica ficheros —
si algún gate falla, la corrección vuelve a la task correspondiente, no a
esta.

**Files:** ninguno.

- [ ] **Step 1: Rust — fmt, clippy, test**

```bash
cd engine
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --release --locked
```

Expected: los tres comandos exit 0. `cargo test` termina con
`test result: ok. N passed; 0 failed` en cada uno de los binarios de test
(incluye los nuevos de las Tasks 1-2:
`search_default_hybrid_degrada_con_aviso_si_falta_tabla_vectores`,
`trocea_por_encima_del_limite_de_placeholders_de_sqlite`,
`union_de_lotes_coincide_con_una_sola_consulta_sin_trocear`).

- [ ] **Step 2: Gates de `scripts/`**

```bash
cd /home/paul/Documentos/proyectos/exo
bash scripts/test-docs-vivos.sh
bash scripts/test-rutas-personales.sh
bash scripts/test-exec-bit.sh
bash scripts/test-versiones.sh
bash scripts/test-hooks-json.sh
bash scripts/test-contrato-ci.sh
bash scripts/test-plugin.sh
```

Expected: los siete, `[OK] ...` y exit 0. (`scripts/test-shellcheck.sh`
necesita el binario `shellcheck` 0.11.0 — si no está instalado en esta
máquina, sáltalo aquí y confía en el job `static-checks` de CI, que lo
descarga pineado por sha256; no bloquea esta task si el binario no está
disponible localmente. `scripts/test-install.sh` es un gate de instalación
end-to-end, pesado — correrlo aquí es opcional, no bloqueante para esta
campaña, que no toca `install.sh`/`install.ps1`.)

- [ ] **Step 3: Confirmar que la regresión original queda cerrada de punta a
  punta (repro manual del bug de la Task 1, con el binario recién
  compilado)**

```bash
cd engine
cargo build --release --locked --bin exo
DIR="$(mktemp -d)"
cat > "$DIR/index.db.sql" <<'SQL'
CREATE TABLE notas (permalink TEXT PRIMARY KEY, ruta TEXT NOT NULL UNIQUE, titulo TEXT NOT NULL, tipo TEXT, mtime REAL NOT NULL, git_epoch INTEGER);
CREATE VIRTUAL TABLE notas_fts USING fts5(titulo, cuerpo, permalink UNINDEXED, tokenize='unicode61 tokenchars 0x2F');
INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch) VALUES ('kb/a', 'a.md', 'a', 'note', 0.0, NULL);
INSERT INTO notas_fts (titulo, cuerpo, permalink) VALUES ('a', 'contenido buscable de a', 'kb/a');
SQL
sqlite3 "$DIR/index.db" < "$DIR/index.db.sql" 2>/dev/null || python3 -c "
import sqlite3
conn = sqlite3.connect('$DIR/index.db')
conn.executescript(open('$DIR/index.db.sql').read())
conn.commit()
"
./target/release/exo search --db "$DIR/index.db" --json buscable; echo "exit=$?"
```

Expected: `exit=0`, stdout con el envelope `{"schema_version":2,"command":"search","data":{...}}`
con `data.search_type == "hybrid"`, `data.warnings` conteniendo un aviso
"arm vector INERTE", y `data.results` con `kb/a`. Si `sqlite3` CLI no está
instalado, el fallback de `python3` monta la misma DB (el `sqlite3` de la
librería estándar de Python siempre está disponible).

- [ ] **Step 4: No hay nada que commitear en este task**

```bash
git status --porcelain -- engine/ scripts/ docs/superpowers/
```

Expected: vacío (todo lo tocado por Tasks 1-6 ya se commiteó en su propio
Step). Si algo aparece, es residuo de un Step anterior sin commitear —
vuelve a esa task, no cierres Task 7 con cambios sin commit.

---

## Self-review (cobertura, placeholders, consistencia)

- **Cobertura de los 7 ítems del brief**: Task 1 = ítem 1 (search default
  vectores ausente, decisión declarada + alternativa). Task 2 = ítem 2
  (permalinks_de_rowids, troceo con test que reproduce el reventón antes de
  arreglarlo). Task 3 = ítem 3 (test-docs-vivos check e, ciclo rojo-verde
  sobre fixture en `/tmp`, README real intacto). Task 4 = ítem 4 (tres
  scripts, mismo patrón que `960a319`, verificado con `git` falso). Task 5
  = ítem 5 (runbook M5b, checklist C10, decisión #15, línea roja
  respetada — cero comandos de desinstalación real). Task 6 = ítem 6
  (barrido de 5 + 6 decisiones + D-4 + runbook de release). Task 7 = ítem 7
  (fmt/clippy/test + todos los `scripts/test-*.sh` relevantes).
- **Placeholder scan**: sin "TBD"/"TODO"/"similar a Task N". Los `old_string`
  de la Task 6 (incluidos los de los Steps 6 y 10, corregidos en la review
  adversarial del 2026-09-19 — ver más abajo) son anclas literales
  verificadas hoy contra HEAD, no descripciones aproximadas.
- **Consistencia de firmas**: `busca`, `busca_vector`, `busca_hybrid`
  (públicas) no cambian de firma en ninguna task. `tabla_existe`,
  `permalinks_de_rowids_lote`, `LOTE_PERMALINKS` son nuevas y ninguna otra
  task de este plan las consume — no hay riesgo de una task usando un
  nombre que otra no ha creado todavía (Tasks 1 y 2 son independientes
  entre sí, cada una autocontenida).
- **Fixes aplicados tras review adversarial (2026-09-19, verificados con
  ejecución real, no solo lectura):**
  1. [Bloq] Task 1 dejaba un fallback MUDO: la fixture `db_sin_tabla_meta`
     no crea `trozos`, así que el early-return de `trozos == 0` se comía el
     aviso de `vectores` ausente antes de que el código llegara a mirarlo —
     confirmado con un panic real (`el default hybrid degradado debía
     avisar por stderr:` con stderr vacío). Rediseñado con `tabla_existe`
     (vía `sqlite_master`) comprobado ANTES del early-return; verificado en
     worktree temporal (`git worktree add --detach`, limpiado con `git
     worktree remove --force` después): el test nuevo PASA, las 17 de
     `kb_root_lectura_cli`, las 12 de `buscador` (unit), las 9+9+2 de
     `buscador_cli`/`contrato_envelope`/`recall_avisos_cli`, y la suite
     completa (`cargo test --release --locked`, 55 binarios, 0 `FAILED`)
     quedan verdes.
  2. [Imp] Task 6 Steps 6 y 10 citaban texto que no existe en HEAD ("que
     exista un cliente MCP con soporte completo" nunca apareció; el ítem del
     cap es mucho más largo de lo que parecía, con dos "Cruce" adicionales).
     Corregido con los `old_string` reales: `backlog:1235` ("modelo los
     busca.") para M5a, `backlog:611-617` ("Los nueve hooks son
     never-block... (a).") para el cap.
  3. [Menor] El runbook de la Task 5 decía "Los nueve hooks" — son diez
     (`jq '[.hooks[]?[]?.hooks[]?]|length'` = 10). Corregido.
  4. [Menor] Global Constraints y la tabla de no-solape decían
     `plugins/exo/hooks/*.sh` — ese directorio NO tiene ningún `.sh`, solo
     `hooks.json` (verificado con `ls`). Los scripts de los hooks que toca
     la campaña I viven en `plugins/exo/scripts/`. Corregido, y aclarado que
     la Task 3/5 SOLO leen `hooks.json` (con `jq`/`grep`), nunca lo editan.
- **Citas corregidas frente al brief**: ítem 1, el brief cita
  `~backlog:1712`; la posición real en HEAD es `~backlog:838-856`. Ítem 2,
  el brief cita `~backlog:720`; la posición real de «Techos de escala» es
  `~backlog:659-707`. Los ítems 4 (`~backlog:1691`) y los cinco del barrido
  del ítem 6 (`:456`, `:1438`, `:1476`, `:1536`, `:1600`) coinciden con
  HEAD sin corrección.
- **Ítems del barrido NO confirmados cerrados**: ninguno — los cinco se
  verificaron con `git grep`/lectura directa contra HEAD antes de escribir
  el Step correspondiente (tabla en la cabecera de la Task 6).
- **Decisión que no pude verificar del todo**: el texto exacto de la
  cabecera de `docs/backlog.md` (Step 13 de la Task 6, "Última revisión")
  depende de los hashes de commit reales que resulten de ejecutar las Tasks
  1-4 — no existen todavía al escribir este plan, así que el Step da la
  estructura y el contenido a resumir, no el texto final byte a byte (es la
  única excepción deliberada a "cero placeholders": un hash de commit que
  no existe aún no se puede citar de antemano).
- **`## Cerrado con evidencia` no existe como heading real** (hallazgo del
  self-review, no una tarea): `docs/backlog.md` cita `` `## Cerrado con
  evidencia` `` quince veces como destino de los ítems cerrados, pero
  `grep -n "^##" docs/backlog.md` en HEAD solo devuelve `Estado`, `Alta`,
  `Media`, `Baja` — ese heading no existe en el fichero. La convención REAL
  observada (47 ítems `[x]` verificados, repartidos entre Alta/Media/Baja)
  es marcar `[x]` **en el sitio**, con el párrafo de cierre pegado al ítem
  — nunca reubicar. Este plan sigue esa convención real (marca en el sitio)
  y NO crea el heading ni migra los 47 ítems existentes: sería una
  reestructuración de medio fichero, fuera del alcance que pidió esta
  campaña, y con alto riesgo de colisión con la campaña I sobre el mismo
  fichero. Decisión mía, alternativa considerada: crear
  `## Cerrado con evidencia` y mover los 47 ítems — descartada por alcance
  y riesgo; queda para quien decida resolver esa referencia rota (afecta a
  quince citas, no a esta campaña).
- **Handoff**: ejecución con `exo:orchestrate`, un executor fresco por
  task. Ninguna task depende de contexto no escrito en su propia sección —
  cada `old_string` cita texto verificado hoy contra HEAD.
