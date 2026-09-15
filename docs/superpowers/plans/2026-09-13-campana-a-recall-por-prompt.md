# Campaña A — Recall por prompt: correcto, barato y medido. Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: usa `exo:orchestrate`
> para ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`)
> para tracking. Sesión-fábrica: lane y oráculo por tarea en §«Orden, lanes y
> oráculos».

**Goal:** que el recall inyectado en cada prompt no se rompa en silencio, que
diga lo que le cuesta y cuándo se degrada, que no pague trabajo O(N) evitable,
y que haya un número pre-registrado que diga cuándo reabrir el proceso
residente.

**Architecture:** primero se construye el instrumento: un generador de KB
sintética (`engine/examples/kb_sintetica.rs`) y un bench con criterios
pre-registrados (`evals/recall-coste/`). Con él se mide la línea base **antes
de tocar código**. Después vienen los arreglos, cada uno con TDD: el tope de
KNN de vec0 (H27), los avisos y tiempos en `exo recall` más su log en el hook
(H2/H3), `resuelve_destinos` sin escrituras inútiles (H4), la guarda «una DB =
una KB» (H1), el dedupe de `separa_frontmatter` (H19) y tres piezas que
dependen de decisiones de Paul y del bench (H17a, H5, H10). La campaña cierra
con el script del criterio de reapertura (H3) y la corrida «después», que se
juzga mecánicamente contra el pre-registro.

**Tech Stack:** Rust edition 2024 (crate `exo` 0.1.0 en `engine/`, MSRV 1.95) ·
`anyhow` 1.0.103 · `clap` 4.6.2 derive · `rusqlite` 0.40.1 (`bundled`) ·
`sqlite-vec` =0.1.9 · `serde` 1.0.228 / `serde_json` 1.0.150 · `yaml_serde`
0.10.4 · `tempfile` 3.14 (dev) · bash + jq (1.7 medido en Linux) · git · para
el bench, solo en Linux: hyperfine 1.20.0 y awk.

Pre-registro del bench (criterios, escenarios, predicciones y reapertura):
`docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md`. **Se
commitea antes que nada de este plan** (ver Task 1, Step 0).

---

## Re-verificación (2026-09-13) — lo que el código dice de cada hallazgo

Cada hallazgo se contrastó contra el árbol de `3c1918f` y, donde había algo
que medir, con una medición real: el binario `~/.local/bin/exo` 0.1.0 sobre
una **copia** del índice. `~/.exo/index.db` no se tocó. Las cifras literales y
sus comandos están en §«Lo ya medido» del pre-registro.

| ID | ¿Se reproduce? | Qué cambia respecto a la revisión |
|---|---|---|
| **H1** | **Sí, y es peor que lo reportado.** Dos `exo init` con configs distintas sobre la misma `EXO_DB` dan en la segunda `error: upsert de notas para kb-b/AGENTS: UNIQUE constraint failed: notas.ruta`, exit 1. Pero con una segunda KB **sin rutas en común**, `exo index` sobre la misma DB sale con **exit 0 y `"deleted":11`**: borra en silencio el índice de la primera, y al volver re-embebe todo (`chunks_embedded: 51`, caché perdida). El riesgo ya estaba escrito en `scripts/test-contrato-ci.sh:30-35` («indexa la semilla ahí… pisándole meta.kb_root») | El problema no es `ruta UNIQUE` (`schema.rs:22`). El invariante real es **una DB = una KB**: `meta.kb_root` guarda un único valor (`indexer.rs:115-126`), `indexa` borra toda ruta que no ve en su walk (`indexer.rs:241-259`) y kbx consulta por `ruta` sin KB (`kbx/cmd/kbx/diffsince.go:180`, `history.go:139`, `internal/targets/targets.go:89-97`). Namespacing obligaría a acotar indexer, search, recall, lint y targets, y rompería kbx en silencio. Opciones en **D1** |
| **H2** | **Sí.** `recall.rs:495-521` solo lee `resultado.results`. `main.rs:816-888` (`recall_cmd`) no imprime avisos, mientras que `busca_cmd` sí lo hace (`main.rs:905-911`). No hay un solo test | Además `elapsed_s` de `busca_hybrid` no cubre el `--refresh`. Para H3 hace falta también `refresh_s` |
| **H3** | **Sí.** Sobre la KB real, recall con los flags del hook tarda 987–1000 ms con `--refresh` y 982–1021 ms sin él. `elapsed_s` interno: 0,94–0,97 s. `exo search --type fts`: 5–8 ms | El término dominante es **la carga del modelo ONNX para embeber la query**, no el refresh (~15 ms) ni el shell. Confirma que la palanca es el proceso residente, que queda fuera de alcance |
| **H4** | **Sí en código** (`aristas.rs:52-88`, llamado sin condición en `indexer.rs:264`) | **Severidad sobrestimada hoy:** `exo index` sin cambios tarda 19–21 ms en total y `strace` da **0 fsync** y 8 `pwrite64`. Es coste de CPU O(aristas) por prompt, y se mide con 5.000 notas. **Se descarta el fix propuesto** «saltar si `indexadas == 0 && borradas == 0`»: si un abort ocurre entre los commits por nota y la resolución, el índice queda con aristas sin resolver **para siempre**. Fix alternativo en la Task 6 |
| **H5** | **Parcial.** Crece sin cota: 892.617 B y 2.174 líneas desde 2026-06-26, unos 11 KB/día | **Severidad sobrestimada:** `recall-inject.sh` solo añade líneas (O(1)). El único lector del camino caliente es `exo-recall.sh:111-112`, que solo corre en `SessionStart` con `source=compact` y hoy tarda 17–20 ms. `a1-gate.sh`, `reflex-baseline.sh` y `reflex-fp-review.sh` son análisis manuales que **necesitan** la historia entera, igual que el item de backlog del cruce de coste (`docs/backlog.md` §«El bucle de coste de la inyección»). Rotar tiene coste. Opciones en **D3** |
| **H10** | **Sí** (`recall-inject.sh:213`, `exo-recall.sh:69`, `compose-inject.sh:29`) | `exo config --json` tarda 5–6 ms en Linux, **~1% del hook**. Solo se justifica si W11 da otra cosa: puerta **C-H10** y **D5** |
| **H17a** | **Sí** (`recall.rs:233-235`: `tier_de` por fila). Hoy `recall --content --note` tarda 14–17 ms con 174 notas | Hay una variante **sin schema** (lectura perezosa en el camino `--note`, que es el del hook). Opciones en **D2**. Solo se ejecuta si lo pide el bench (**C-H17a**) |
| **H17b** | **Sí en código:** tres aperturas en hybrid (`buscador.rs:442`, `:450`, `:461`), una más en `recall.rs:498` y otra en el refresh. N+1 de `fila_notas` + `primer_trozo` con `--limit 4`, es decir, 8 lecturas por PK | Se **mide y no se arregla en A**: lo caro escala en `buscador.rs` (HashMap de todos los trozos en `:289-294`, KNN en `:286`), que es fichero de la campaña C. El N+1 de `recall.rs` son 8 lecturas por PK, despreciables frente a ~1 s de modelo |
| **H19** | **Sí, con matiz:** `escritor.rs:426-445` no es una copia exacta. Añade el fallback «sin frontmatter ⇒ todo es cuerpo» y conserva el `\n` final | El dedupe queda como envoltorio sobre `nota::separa_frontmatter` (`nota.rs:74-87`) |
| **H23** | **Sí** (`kb-precommit.sh:33-35`) | Solo se mide (escenario `s8`) |

**Hallazgos descartados enteros: ninguno.** Recortados: H4 (severidad y fix
propuesto), H5 (severidad), H10 (coste en Linux) y H1 (forma del fix).

### Hallazgos nuevos que salieron al re-verificar

- **H27 (NUEVO, alto, S) — el arm vector revienta al pasar de 4.096 trozos.**
  sqlite-vec 0.1.9 tiene `#define SQLITE_VEC_VEC0_K_MAX 4096`
  (`~/.cargo/registry/src/*/sqlite-vec-0.1.9/sqlite-vec.c:7111-7117`), y
  `busca_vector` pide `k = COUNT(*)` de `vectores` (`buscador.rs:272-287`).
  Lo reproduje con un programa aparte (rusqlite 0.40.1 + sqlite-vec =0.1.9, con
  4.100 vectores): `k=4096` va bien, y `k=4097` da `k value in knn query too
  large, provided 4097 and the limit is 4096`. **La KB real va por 3.290
  trozos, el 80% del tope.** Al ritmo de las últimas cuatro semanas (138 notas
  el 08-17, 174 hoy, unos 19 trozos por nota) quedan del orden de un mes.
  Cuando se cruce, `exo search --type vector|hybrid` y `exo recall --query`
  saldrán con exit 1 en cada prompt, el hook registrará `degraded
  reason=error` y la memoria por prompt se apagará. Se planifica en A
  (Task 3), porque además bloquea el bench a 5.000 notas. Toca
  `engine/src/vectores.rs` y no `buscador.rs`.
- **H28 (NUEVO, a derivar a C, no se planifica aquí) — la «similitud» del arm
  vector no es coseno.** `l2_sqr_float` de sqlite-vec devuelve `sqrt(res)`
  (`sqlite-vec.c:361-375`; también la ruta AVX en `:158`), así que la
  `distance` de vec0 es la L2 y no L2². Medido: `vec_distance_l2 = 0.962646`,
  con L2 = 0.962646 y L2² = 0.926687. `similitud_desde_l2_cuadrado`
  (`buscador.rs:226-228`) calcula `1 − d/2` con d = L2, que no es coseno. El
  ranking del arm vector puro no cambia (la transformación es monótona),
  **pero el umbral `0.40` de `recall-inject.sh:145` y la fusión hybrid sí
  dependen de esa escala.** Arreglarlo cambia el ranking y exige re-evaluar,
  así que es de C. La doc de `vectores.rs:74-77` y `buscador.rs:212-225`
  afirma L2².
- **H29 (NUEVO, bajo, no se planifica) — el walker entra en `.git/`.** De los
  314 `openat` de un `exo index` sin cambios, 276 caen dentro de
  `wisdom-paul/.git/`. `walker.rs:5` solo excluye `.claude`, `.omc` y
  `.superpowers`. El escenario `s3` lo cuantifica, y el dato va al veredicto.

### Sinergias revisadas

- **«Una sola migración de schema» (H1 + H17a): FALSA.** H1 no necesita
  cambiar el schema (ver D1, opción A). H17a tiene una variante sin schema (D2,
  opción L). Con las recomendaciones, la campaña hace **cero migraciones** y
  kbx no se ve afectado. Si Paul elige P en D2, la columna `tier` al final de
  `notas` **no dispara** el canario de kbx: `CheckSchema` solo mira que estén
  las columnas que consume (`kbx/internal/index/schema.go:52-57`, `:66-99`);
  las columnas de más las ignora.
- **«H19 y H17a tocan `nota.rs`»: DÉBIL.** Ninguna variante de H17a cambia
  código de `nota.rs`; P solo tocaría el comentario de `nota.rs:14`.
- **«H2 habilita el logging de H3 sin instrumentación extra»: A MEDIAS.** Hace
  falta añadir `refresh_s`, porque `elapsed_s` de `busca_hybrid` no cubre el
  refresh.
- **«Un solo bench»: CIERTA.** Pero a 5.000 notas el bench revienta en la
  query (H27) antes de medir nada. Queda como predicción P1.
- **«H5 y H10 recortan el coste shell del mismo hook»: FALSA para H5.** El
  hook por prompt solo añade líneas. H10 pesa ~1% en Linux.
- **Sinergia real que no estaba listada: H1 (opción A2) + H10.** Los tres
  hooks cablean `~/.exo/index.db` (`recall-inject.sh:24`, `exo-recall.sh:34`,
  `exo-index.sh:28`) en vez de leerlo de la config. Un `exo init --db` sin
  cambiar eso es una trampa: los hooks seguirían mirando la DB de otra KB.
  Leer la DB de la config sale de la misma llamada a `exo config --json` que
  ya se hace para el nombre.

---

## Global Constraints

- **El crate vive en `engine/`, no en la raíz.** No hay workspace de Cargo.
  Los comandos `cargo` se ejecutan con cwd `engine/` o con `--manifest-path
  engine/Cargo.toml`.
- **MSRV: `rust-version = "1.95"`** (`engine/Cargo.toml:7`, con la nota
  «1.94 falla (libsqlite3-sys usa cfg_select, estable desde 1.95), 1.95
  compila»). El job `msrv` de CI corre `cargo check --all-targets --locked`,
  que **compila también `engine/examples/`**. No se añaden dependencias, ni
  `[dependencies]` ni `[dev-dependencies]`.
- **Gate hermético, verbatim de `engine/scripts/test-hermetico.sh:19`:**
  `EXO_CONFIG="$TMP/no-existe.toml" cargo test --release --no-fail-fast`.
  **Ningún test puede depender de `~/.exo/config.toml` ni del HOME real.**
  Los tests de librería usan `common::con_config` (`engine/tests/common/mod.rs`),
  y los subprocesos llevan `.env("EXO_CONFIG", …)` explícito, porque heredan
  el `EXO_CONFIG` inexistente del gate. El gate **no** cubre la caché del
  modelo ONNX: las suites que indexan cuerpos no vacíos lo bajan o lo leen.
- **Envelope v2, verbatim de `engine/src/envelope.rs:7` y `:17-24`:**
  `pub const SCHEMA_VERSION: u32 = 2;` y `{"schema_version":2,"command":<command>,"data":<data>}`
  en una línea a stdout vía `envelope::emite`. «Cambio breaking en la forma de
  `data` ⇒ bump; campos aditivos no lo suben». Las claves de `data` van **en
  inglés** (D8). Los avisos y el progreso van **siempre a stderr**. Los
  consumidores gatean por exit code, **nunca** por campos de `data`.
- **Códigos de salida, verbatim de `main.rs:375-413`:** `0` éxito · `3` gate
  de dominio (`escritor::Rechazo` o `gate::GateFallido` downcasteados) · `1`
  cualquier otro error (`eprintln!("error: {e:#}")`) · `2` lo pone clap ante
  un error de línea de comandos. Esta campaña **no crea códigos nuevos**: la
  guarda de H1 es exit 1 (error operativo, no hay informe que emitir).
- **Tests de CLI:** `std::process::Command::new(env!("CARGO_BIN_EXE_exo"))`,
  `tempfile` para fixtures y `serde_json::from_slice` para el envelope. No hay
  `assert_cmd` ni `predicates`, y este plan no los añade.
- **TDD en toda tarea de engine:** primero el test, verlo fallar por la razón
  esperada, luego el mínimo. Las dos excepciones van declaradas en su tarea:
  refactor sin cambio de comportamiento (Task 9) y cambio solo de rendimiento
  (Task 10). En ambas se sustituye el rojo por caracterización más una
  mutación que demuestra que el test muerde.
- **Hooks del plugin (`plugins/exo/scripts/`):**
  - `recall-inject.sh` **nunca** sale con ≠0 (`recall-inject.sh:6-9`: «un
    exit 2 no degrada, BORRA el prompt de Paul») y **solo** escribe el JSON
    final a stdout (`:11-14`).
  - Portables a macOS (commit `ec74f43`): nada de `timeout` (se usa
    `con_timeout` de `_timeout.sh`), `date -d`, `stat -c` ni `touch -d`.
  - Todo `plugins/*.sh` va en **100755** (`scripts/test-exec-bit.sh`, job
    `exec-bit`).
  - Los tests `plugins/exo/scripts/test-*.sh` se descubren por glob
    (`scripts/test-plugin.sh`) y corren en ubuntu, macos y windows (Git
    Bash). **Nunca** tocan el log, el índice ni el HOME reales: `REFLEX_LOG_FILE`
    y `HOME` apuntan a `mktemp -d`.
- **Cada spawn de proceso cuesta** «decenas de milisegundos» en Git Bash
  (`docs/backlog.md` §«El coste del hook completo en Windows no está
  medido»). Ningún cambio de hook añade invocaciones de `jq` o `exo` en el
  camino común si puede fusionarlas con una que ya exista.
- **Líneas rojas:** nada de push, merge a `main`, `gh` ni tocar
  `~/.exo/index.db` o la KB real. El bench trabaja solo sobre corpus
  sintético en `mktemp -d`.
- **Fuera de alcance, declarado:** el proceso residente (solo se instrumenta,
  se mide y se escribe el criterio de reapertura) · backup y seguridad · la
  calidad de retrieval y la escala de similitud (H28, campaña C) · el walker y
  `.git` (H29) · los arreglos de coste en `buscador.rs` (H17b, se miden y se
  derivan).

---

## Decisiones abiertas (PENDIENTE-PAUL)

> Régimen del config (`.superpowers/fabrica/config.md` §«Ejecución de
> gates»): cada una la intenta primero el consultor fable delegado. **D0, D1 y
> D5 no tienen fuente escrita citable**, así que se espera que escalen a Paul.

**D0 — ¿Esta campaña puede pre-registrar un bench?** *Bloquea: todas las tareas.*
El config vigente, en su bloque «ACTUALIZACIÓN 2026-08-17 — fase de cierre
(manda sobre todo lo de abajo)», dice literal: «**Sin métricas nuevas, sin
pre-registros nuevos, sin ventanas de observación.** Ningún item se selecciona
ni se bloquea por producir un número». Esta campaña es exactamente eso: un
pre-registro, dos puertas numéricas (C-H17a y C-H10) y una ventana de 14 días
(la reapertura de H3). Opciones:
(a) Paul actualiza el config, cerrando la fase de cierre ahora que M5b ya pasó;
(b) `OVERRIDE` puntual en el ledger;
(c) quitar las puertas numéricas y ejecutar H17a/H10 sin medir.
**Recomendación: (a).** La fase de cierre terminó con la release v0.1.0, y (c)
contradice la regla de validar contra datos.
**RESUELTA 2026-09-13 — opción (b):** `OVERRIDE` de Paul en sesión («lanza…
otro que empiece con la campaña A»). Vale para esta campaña; la actualización
del config (a) sigue pendiente. D1–D5 siguen abiertas.

**D1 — Forma del arreglo de H1.** *Bloquea: Task 7 (salvo con B o C) y Task 8
(solo con A2).*

| Opción | Qué es | A favor | En contra |
|---|---|---|---|
| **A1** guarda sola | `indexa` e `init` rechazan (exit 1, sin tocar disco) una DB cuyo `meta.kb_root` es **otra KB que sigue existiendo**. Una KB movida de sitio pasa. Queda documentado que exo es una KB por DB | S. Sin schema. kbx intacto. Convierte el borrado silencioso y el UNIQUE críptico en un error que dice el remedio | La segunda KB en la misma máquina solo es posible a mano (`EXO_CONFIG` + `EXO_DB` + `EXO_INDEX` en los hooks) |
| **A2** guarda + `--db` + hooks | A1 más `exo init --db`, y los tres hooks leen la DB de `exo config --json` en vez de `~/.exo/index.db` | M. N KBs en una máquina, una por config. Sin schema. kbx intacto | Toca tres hooks. El Stop hook deja de pasar `--db` (lo resuelve el engine) |
| **B** multi-KB en una DB | Columna `kb` en `notas` (y en `trozos`/`aristas`), `UNIQUE(kb, ruta)`, `meta.kb_root` por KB, y búsqueda/recall/lint/targets acotados | Una sola DB | L. **Migración:** SQLite no altera un UNIQUE, así que hay que reconstruir la tabla o hacer `exo rebuild`. **kbx se rompe en silencio:** consulta por `ruta` sin KB (`diffsince.go:180`, `history.go:139`, `targets.go:89-97`) y no conoce la columna. Es superficie irreversible (schema), así que va al régimen de gates. **No planificado aquí:** exige un plan propio |
| **C** `ruta` prefijada | `ruta = "<kb>/AGENTS.md"` | Sin columna nueva | Rompe kbx (abre `kb_root/ruta`), `lint` y `targets` de exo (rutas relativas a la KB). Migración igual que en B |

**Recomendación: A1 ahora.** A2 si Paul tiene o prevé una segunda KB real en
la misma máquina: en `~/Documentos/proyectos/` existe `wisdom-ai-news`, y no
verifiqué si es una KB de exo. B y C no: pagan migración y kbx por un caso de
uso que A2 ya cubre.

**D2 — `tier` (H17a).** *Bloquea: Task 10, que además está sujeta a la puerta C-H17a.*
- **L (lazy, sin schema):** en `recall --content --note` (el camino del hook)
  se lee de disco el `tier` solo de las candidatas a reciente, O(limite +
  cores recientes). Sin `--note` sigue siendo O(N). S, sin migración.
- **P-inplace:** `ALTER TABLE notas ADD COLUMN tier TEXT` en `crea_schema`,
  más `UPDATE notas SET mtime = -1` para forzar el reparse (los embeddings se
  reutilizan por texto), más una lectura tolerante mientras la columna esté a
  NULL. M. Riesgo: el primer `--refresh` tras actualizar reparsea toda la KB
  **dentro del timeout de 5 s del hook**. En Linux da unos segundos; en W11,
  con un `git log` por nota, puede ser más, y la convergencia sería a trozos,
  por prompts.
- **P-rebuild** (lo que propone `docs/backlog.md` §«`tier` no se persiste»):
  columna más guarda de versión que exige `exo rebuild`. M. El rebuild borra la
  DB y **re-embebe todo** (la caché de embeddings vive en la DB), y los hooks
  caen al fallback hasta que alguien lo corra.
- **Nada:** si el bench no cruza la puerta.

**Recomendación: L**, si la puerta se abre. P es superficie de schema: si Paul
la elige, hay que enmendar el plan con una tarea propia.

**D3 — Logs de reflejos (H5).** *Bloquea: Task 11 (se ejecuta una sola variante).*
- **T (tail):** `exo-recall.sh` lee solo las últimas `N` líneas (propuesta:
  2.000, unos 74 días al ritmo actual) al reforzar tras compactación. No se
  rota nada: el disco crece ~4 MB/año y el análisis conserva toda la historia.
- **R (rotación):** en `SessionStart`, si el log supera `X` bytes se mueve a
  `.1` (una generación; se pierde lo anterior a `.1`). Los tres scripts de
  análisis leen `.1` y el actual. Hay que fijar `X` (propuesta: 5 MB) y el
  número de generaciones. Coste: pierde historia que el item «bucle de coste
  de la inyección» quiere cruzar, y hay carrera entre sesiones concurrentes
  (una puede mover el `.1` de otra).
- **Nada:** hoy son 17–20 ms en un evento raro.

**Recomendación: T.** Datos para decidir: `s10` del bench.

**D4 — Umbrales de reapertura del proceso residente (H3).** *Bloquea: Task 13,
que tiene defaults.* La propuesta del pre-registro es REABRIR si, con ≥200
disparos en 14 días y por máquina, el p95 de `elapsed_ms + refresh_ms` supera
1.500 ms **o** los timeouts pasan del 2%. Alternativa: fijarlo solo sobre W11,
donde duele más. **Recomendación:** los defaults, evaluados por separado en
cada máquina.

**D5 — Medición en W11.** *Bloquea: Task 15 y la mitad W11 de la puerta C-H10
(Task 12).* ¿La hace Paul a mano con el bloque §W11 del pre-registro? ¿Cuándo?
Sin ella, la puerta C-H10 se decide solo con Linux, donde casi seguro queda
cerrada. **Recomendación:** que Paul la corra una vez antes de la Task 12. Son
dos bucles de 20 iteraciones, unos 2 minutos.

**D6 — Dueño de H27 (A o C).** *No bloquea.* Se planifica en A porque el bench
lo necesita y porque toca `vectores.rs`, no `buscador.rs`. Si C lo reclama, la
Task 3 sale de A y la predicción P1 se mantiene.

**RESUELTAS 2026-09-13 — Paul acepta las recomendaciones del plan:**
D1 = **A1** (solo la guarda; Task 8 no se ejecuta) · D2 = **L** (moot: la
puerta C-H17a salió CERRADA en la línea base, Task 10 no se ejecuta) · D3 =
**T** (Task 11 variante T) · D4 = **defaults** (≥200 disparos/14 días por
máquina; p95 `elapsed_ms + refresh_ms` > 1.500 ms o timeouts > 2%) · D5 =
**Paul corre el bloque §W11 una vez antes de la Task 12** (12 y 15 esperan) ·
D6 = **A**, ejecutada como hotfix aparte (PR #11, mergeado).

---

## Dependencias y conflictos con las campañas B y C

| Con | Fichero o tema | Conflicto | Cómo se resuelve |
|---|---|---|---|
| **B** (`--help`, idioma CLI) | `engine/src/main.rs` | A toca `ArgsInit` (Task 8), `init_cmd` (Tasks 7 y 8) y `recall_cmd` (Task 4). B reescribe los doc-comments de clap y los metavars | Mergear primero las tareas de A que tocan `main.rs`, o que B rebase. Los mensajes nuevos de A (guarda de H1) siguen la decisión de idioma de H9: si B elige inglés para mensajes de usuario, B los traduce |
| **B** (versionado H16) | `plugins/exo/.claude-plugin/plugin.json` | A cambia hooks (Tasks 5, 8, 11 y 12) y eso exige subir la versión del plugin | A **no** sube la versión: la sube B según su decisión, al integrar |
| **B** (shellcheck H11, exec-bit H12) | scripts nuevos o tocados en A | Si B entra antes, los scripts de A tienen que pasar shellcheck | Los scripts de A nacen en 100755. Ojo con `${VAR:+--db "$VAR"}` sin comillas externas (Task 8): si shellcheck lo marca, `# shellcheck disable=SC2086` con motivo |
| **B** (backlog H13) | `docs/backlog.md` | Los dos lo editan. H13 corrige precisamente el item cerrado «Modo mudo» (`backlog.md:1186-1194`), que H2 contradice | B corrige el texto de H13. A (Task 14) solo **añade** una línea de estado al final de cuatro items, localizados por su título y no por número de línea |
| **C** (retrieval) | `engine/src/buscador.rs` | A no lo toca. Pero H2 depende de que `Busqueda` conserve `avisos` y `elapsed_s` | Restricción para C: si RRF reescribe `busca_hybrid`, `Busqueda.avisos` y `.elapsed_s` se mantienen |
| **C** (Task 12 de su plan, producción) | `engine/src/recall.rs` (`recall_consulta`), `engine/src/main.rs` (`recall_cmd`) | C le quita a `recall_consulta` los parámetros `bonus`/`escala_fts`; A (Task 4) cambia su cuerpo y `RecallBruto`. El plan de C ya declara que depende del merge de A | A entra primero. C rebasea conservando `avisos`/`elapsed_s` en `RecallBruto` |
| **C** | `engine/src/vectores.rs` | Task 3 cambia `knn`. Si C toca KNN, conflicto | D6 |
| **C** (chunking) | `engine/src/trozos.rs` | El generador usa `trozos::trocea`, así que otro troceado cambia el corpus y la línea base deja de ser comparable | La corrida «después» (Task 14) va **antes** de que entre en `main` un cambio de C a `trozos.rs` |
| **C** (H28) | umbral `0.40` en `recall-inject.sh:145` | Si C corrige la escala, el `0.40` del hook cambia de significado | Lo decide C, y avisa a quien mantenga el hook. El bench de A usa `0.40` y `0.0` |
| **C** (H17b) | coste de `buscador.rs` a escala | A entrega los números (C-H17b) | Derivación en el veredicto de la Task 14 |

---

## Mapa de ficheros

| Fichero | Tarea | Responsabilidad |
|---|---|---|
| `engine/examples/kb_sintetica.rs` (nuevo) | 1 | KB sintética con la forma de la real, e índice sin modelo |
| `evals/recall-coste/harness/bench.sh` (nuevo) | 1 | Escenarios `s1`–`s10` con hyperfine y `resumen.tsv` |
| `evals/recall-coste/harness/compara.sh` (nuevo) | 1 | Predicciones, puertas y criterios, aplicados de forma mecánica |
| `evals/recall-coste/results/{baseline,despues}/` | 2, 14 | Resultados crudos |
| `evals/recall-coste/verdict/2026-09-campana-a.md` (nuevo) | 14 | Veredicto |
| `engine/src/vectores.rs` | 3 | `knn` sin el tope de vec0 |
| `engine/src/recall.rs` | 4, 10 | `warnings`/`elapsed_s`/`refresh_s`; recientes perezosas |
| `engine/src/main.rs` | 4, 7, 8 | `recall_cmd` avisos y tiempos; `init_cmd` guarda y `--db` |
| `engine/src/aristas.rs` | 6 | `resuelve_destinos` sin escrituras nulas |
| `engine/src/indexer.rs` | 7 | `comprueba_kb_root` |
| `engine/src/inicia.rs` | 7 | `valida_db_para_kb` |
| `engine/src/nota.rs`, `engine/src/escritor.rs` | 9 | un único `separa_frontmatter` |
| `engine/tests/{recall.rs,recall_avisos_cli.rs,contrato_envelope.rs,aristas_resolucion.rs,indexer.rs,inicia.rs,recall_contenido.rs}` | 4, 6, 7, 8, 10 | tests |
| `plugins/exo/scripts/recall-inject.sh` | 5, 8, 12 | log de avisos y tiempos; DB desde config; memo de config |
| `plugins/exo/scripts/exo-recall.sh` | 8, 11, 12 | DB desde config; tail o rotación; memo |
| `plugins/exo/scripts/exo-index.sh` | 8 | sin `--db` cableado |
| `plugins/exo/scripts/_exo-config.sh` (nuevo) | 12 | `exo config --json` memoizado por sesión |
| `plugins/exo/scripts/subagent-inject.sh`, `compose-inject.sh` | 12 | usan el memo |
| `plugins/exo/scripts/recall-latencia.sh` (nuevo) | 13 | criterio de reapertura |
| `plugins/exo/scripts/{a1-gate,reflex-baseline,reflex-fp-review}.sh` | 11 (solo R) | leen `.1` y el actual |
| `plugins/exo/scripts/test-{recall-inject,exo-index,exo-recall,recall-latencia,contrato-engine,reflex-baseline,a1-gate}.sh` | 5, 8, 11, 12, 13 | tests |
| `docs/backlog.md` | 14 | líneas de estado |

## Orden, lanes y oráculos

Oráculos comunes, que se citan por nombre:
- **O-engine:** `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cd .. && ./engine/scripts/test-hermetico.sh`
- **O-plugin:** `./scripts/test-plugin.sh && bash scripts/test-exec-bit.sh`
- **O-contrato:** `./scripts/test-contrato-ci.sh`, que necesita el binario y el modelo

| # | Tarea | IDs | Lane | Oráculo específico (además de los comunes que aplique) | Bloqueada por |
|---|---|---|---|---|---|
| 1 | Generador, bench y compara | H3 H4 H5 H10 H17 H23 | **diseño** (construye el oráculo) | smoke del Step 4 de la Task 1 | D0 |
| 2 | Línea base | idem | mecánica | `evals/recall-coste/harness/bench.sh baseline 174 1000 5000 && evals/recall-coste/harness/compara.sh baseline` | 1 |
| 3 | KNN sin tope | H27 | mecánica | `cd engine && cargo test --release --lib vectores` + O-engine | 2 |
| 4 | Avisos y tiempos en `exo recall` | H2 H3 | mecánica (**envelope aditivo: gate de superficie**) | `cd engine && cargo test --release --test recall --test recall_avisos_cli --test contrato_envelope` + O-engine | 2 |
| 5 | El hook los loguea | H2 H3 | mecánica | `bash plugins/exo/scripts/test-recall-inject.sh` + O-plugin + O-contrato | 4 |
| 6 | `resuelve_destinos` | H4 | mecánica | `cd engine && cargo test --release --test aristas_resolucion --test indexer` + O-engine | 2 |
| 7 | Guarda una DB = una KB | H1 | mecánica | `cd engine && cargo test --release --test indexer --test inicia` + O-engine + O-contrato | 2, D1 ∈ {A1, A2} |
| 8 | `init --db` y hooks leen la DB de la config | H1 | mecánica | `cd engine && cargo test --release --test inicia` + `bash plugins/exo/scripts/test-recall-inject.sh && bash plugins/exo/scripts/test-exo-index.sh && bash plugins/exo/scripts/test-exo-recall.sh` + O-plugin + O-contrato | 7, D1 = A2 |
| 9 | Un solo `separa_frontmatter` | H19 | mecánica | `cd engine && cargo test --release --lib escritor && cargo test --release --test escritor --test nota` + O-engine | 2 |
| 10 | Recientes perezosas | H17a | mecánica | `cd engine && cargo test --release --test recall_contenido --test recall` + O-engine | 2, D2 = L, puerta C-H17a |
| 11 | Log de reflejos | H5 | mecánica | `bash plugins/exo/scripts/test-exo-recall.sh` (+ `test-reflex-baseline.sh` y `test-a1-gate.sh` si R) + O-plugin | 2, D3 ∈ {T, R} |
| 12 | Memo de `exo config` | H10 | mecánica | `bash plugins/exo/scripts/test-recall-inject.sh && bash plugins/exo/scripts/test-subagent-inject.sh` + O-plugin | 5, 8 si A2; puerta C-H10; D5 |
| 13 | Criterio de reapertura | H3 | mecánica | `bash plugins/exo/scripts/test-recall-latencia.sh` + O-plugin | 5, D4 |
| 14 | Después, veredicto y backlog | todos | mecánica | `evals/recall-coste/harness/bench.sh despues 174 1000 5000 && evals/recall-coste/harness/compara.sh baseline despues` | 3–13 que apliquen |
| 15 | Medición W11 | H3 H10 | **manual de Paul** | bloque §W11 del pre-registro | D5 |

Paralelizable, un worktree por item en `.worktrees/<item>`: {3, 6, 9} tras la
2; 4 → 5 → 13 en cadena; 7 → 8. La Task 8 choca con la 5 en
`recall-inject.sh`, así que la 8 va después de la 5.

---

### Task 1: generador de KB sintética, bench y comparador

**Lane:** diseño (construye el oráculo del resto) · **Hallazgos:** H3 H4 H5 H10 H17 H23

**Files:**
- Create: `engine/examples/kb_sintetica.rs`
- Create: `evals/recall-coste/harness/bench.sh` (100755)
- Create: `evals/recall-coste/harness/compara.sh` (100755)

**Interfaces:**
- Consumes (API pública existente, sin cambios): `exo::abre_db(&Path) -> Result<Connection>`;
  `exo::schema::crea_schema(&Connection) -> Result<()>`;
  `exo::nota::parsea_nota(&Path) -> Result<Option<Nota>>` (campos `permalink`, `titulo`, `tipo`, `cuerpo`);
  `exo::aristas::reindexa_aristas_de_nota(&Connection, &str, &str) -> Result<()>`;
  `exo::aristas::resuelve_destinos(&Connection) -> Result<()>`;
  `exo::trozos::trocea(&str) -> Vec<String>`;
  `exo::vectores::inserta(&Connection, i64, &[f32]) -> Result<()>`;
  `exo::MODELO_JINA_ES: &str`.
- Produces: el binario `engine/target/release/examples/kb_sintetica <N> <DIR> [SEMILLA]`, que deja
  `<DIR>/kb/` (repo git con un commit), `<DIR>/index.db` y `<DIR>/config.toml` (name `sint`);
  `bench.sh <etiqueta> [N...]` → `evals/recall-coste/results/<etiqueta>/resumen.tsv` con columnas
  `id p50_ms p95_ms mean_ms corridas_fallidas corridas rc_directa`, e ids `<escenario>-n<N>`;
  `compara.sh <base> [despues]` → líneas `<CRITERIO>\t<VEREDICTO>`.

- [ ] **Step 0: Commitear el pre-registro y el plan ANTES de nada**

```bash
git add docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md docs/superpowers/plans/2026-09-13-campana-a-recall-por-prompt.md
git commit -m "plan: campaña A — recall por prompt, con pre-registro del bench"
test ! -e evals/recall-coste/results && echo "OK: pre-registro antes que resultados"
```
Expected: `OK: pre-registro antes que resultados`.

- [ ] **Step 1: Escribir el smoke que falla**

Es el oráculo de esta tarea. Se corre tal cual, sin guardarlo en un fichero:

```bash
T="$(mktemp -d)" && cargo run --release --manifest-path engine/Cargo.toml --example kb_sintetica -- 58 "$T/n58" 42 \
 && EXO_CONFIG="$T/n58/config.toml" engine/target/release/exo index --kb "$T/n58/kb" --db "$T/n58/index.db" --json \
    | jq -e '.data.indexed == 0 and .data.skipped == 58' \
 && EXO_CONFIG="$T/n58/config.toml" engine/target/release/exo search --db "$T/n58/index.db" --type fts --json "trinquete techos" \
    | jq -e '.data.results | length > 0' \
 && EXO_CONFIG="$T/n58/config.toml" engine/target/release/exo recall --db "$T/n58/index.db" --kb "$T/n58/kb" --content --note sint/core/core-index \
    | grep -q 'Contrato de memoria' \
 && git -C "$T/n58/kb" log --oneline | grep -q 'kb sintetica' && echo SMOKE-OK; rm -rf "$T"
```

- [ ] **Step 2: Verificar que falla**

Expected: `error: no example target named `kb_sintetica`` y no aparece `SMOKE-OK`.

- [ ] **Step 3: Implementación**

`engine/examples/kb_sintetica.rs` (completo):

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
use anyhow::{Context, Result, bail};
use rusqlite::params;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

/// 19 párrafos de ≥580 caracteres: dos juntos pasan de 900 (`trozos::MAX_CHARS`),
/// así que `trocea` da un trozo por párrafo — 19 por nota (3.290/174 = 18,9).
const PARRAFOS_POR_NOTA: usize = 19;
const CHARS_POR_PARRAFO: usize = 580;
/// 727/174 = 4,18 wikilinks por nota.
const ENLACES_POR_NOTA: usize = 4;
/// 24/727 ≈ 1/30 aristas sin resolver.
const UNO_ROTO_CADA: usize = 30;
const EPOCH_COMMIT: i64 = 1_780_000_000;
const NOMBRE_KB: &str = "sint";
const VOCABULARIO: [&str; 24] = [
    "memoria",
    "indice",
    "trinquete",
    "techos",
    "bitacora",
    "canon",
    "recall",
    "prompt",
    "hook",
    "engine",
    "nota",
    "presupuesto",
    "sesion",
    "agente",
    "orquestador",
    "doctrina",
    "permalink",
    "wikilink",
    "trozo",
    "vector",
    "umbral",
    "gate",
    "campana",
    "fabrica",
];

struct Xorshift(u64);

impl Xorshift {
    fn siguiente(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn indice(&mut self, n: usize) -> usize {
        (self.siguiente() % n as u64) as usize
    }

    /// Uniforme en [-1, 1).
    fn unidad(&mut self) -> f32 {
        (self.siguiente() >> 40) as f32 / (1u64 << 23) as f32 - 1.0
    }
}

/// 1/29 core (3,4%), 10/29 stable (34,5%), 18/29 log (62,1%).
fn carpeta_y_tier(i: usize) -> (&'static str, &'static str) {
    match i % 29 {
        0 => ("core", "core"),
        1..=10 => ("projects", "stable"),
        _ => ("log", "log"),
    }
}

fn titulo(i: usize) -> String {
    if i == 0 {
        "core-index".to_string()
    } else {
        format!("nota-{i:05}")
    }
}

fn ruta_rel(i: usize) -> String {
    format!("{}/{}.md", carpeta_y_tier(i).0, titulo(i))
}

fn parrafo(rng: &mut Xorshift) -> String {
    let mut p = String::with_capacity(CHARS_POR_PARRAFO + 16);
    while p.len() < CHARS_POR_PARRAFO {
        if !p.is_empty() {
            p.push(' ');
        }
        p.push_str(VOCABULARIO[rng.indice(VOCABULARIO.len())]);
    }
    p
}

fn cuerpo(i: usize, n: usize, rng: &mut Xorshift) -> String {
    let enlaces: Vec<String> = (0..ENLACES_POR_NOTA)
        .map(|k| {
            if (i * ENLACES_POR_NOTA + k).is_multiple_of(UNO_ROTO_CADA) {
                format!("[[inexistente-{i}-{k}]]")
            } else {
                format!("[[{}]]", titulo(rng.indice(n)))
            }
        })
        .collect();
    let mut texto = format!("# {}\n\n", titulo(i));
    if i == 0 {
        // `exo-recall.sh` exige esta frase para aceptar el bloque de arranque.
        texto.push_str("Contrato de memoria: canon como delta, bitacora como append.\n\n");
    }
    for p in 0..PARRAFOS_POR_NOTA {
        if p > 0 {
            texto.push_str("\n\n");
        }
        texto.push_str(&parrafo(rng));
        if p == 0 {
            texto.push(' ');
            texto.push_str(&enlaces.join(" "));
        }
    }
    texto.push('\n');
    texto
}

fn nota_md(i: usize, n: usize, rng: &mut Xorshift) -> String {
    let (carpeta, tier) = carpeta_y_tier(i);
    let t = titulo(i);
    format!(
        "---\ntitle: {t}\ntype: note\npermalink: {NOMBRE_KB}/{carpeta}/{t}\ntier: {tier}\n---\n\n{}",
        cuerpo(i, n, rng)
    )
}

fn versiona(kb: &Path) -> Result<()> {
    let fecha = format!("{EPOCH_COMMIT} +0000");
    for args in [
        vec!["init", "-q"],
        vec!["add", "-A"],
        vec![
            "-c",
            "user.name=bench",
            "-c",
            "user.email=bench@exo.invalid",
            "commit",
            "-q",
            "-m",
            "kb sintetica",
        ],
    ] {
        let st = Command::new("git")
            .arg("-C")
            .arg(kb)
            .args(&args)
            .env("GIT_AUTHOR_DATE", &fecha)
            .env("GIT_COMMITTER_DATE", &fecha)
            .status()
            .with_context(|| format!("git {args:?}"))?;
        if !st.success() {
            bail!("git {args:?} falló en {}", kb.display());
        }
    }
    Ok(())
}

fn escribe_config(dir: &Path, kb: &Path, db: &Path) -> Result<()> {
    let barras = |p: &Path| p.display().to_string().replace('\\', "/");
    std::fs::write(
        dir.join("config.toml"),
        format!(
            "schema_version = 1\n\n[kb]\npath = \"{}\"\nname = \"{NOMBRE_KB}\"\n\n\
             [index]\ndb = \"{}\"\n\n[embeddings]\nmodel = \"{}\"\ndims = 768\n\
             min_similarity = 0.35\n",
            barras(kb),
            barras(db),
            exo::MODELO_JINA_ES
        ),
    )
    .context("escribir config.toml")
}

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
    let conn = exo::abre_db(db)?;
    exo::schema::crea_schema(&conn)?;
    let kb_abs = std::fs::canonicalize(kb).context("canonicalizar kb")?;
    for (clave, valor) in [
        ("kb_root", kb_abs.to_string_lossy().into_owned()),
        ("modelo_embeddings", exo::MODELO_JINA_ES.to_string()),
        ("dims_embeddings", "768".to_string()),
    ] {
        conn.execute(
            "INSERT INTO meta (clave, valor) VALUES (?1, ?2)
             ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
            params![clave, valor],
        )?;
    }
    let tx = conn.unchecked_transaction()?;
    for i in 0..n {
        let rel = ruta_rel(i);
        let abs = kb.join(&rel);
        // El MISMO parser que el indexer: título, tipo y cuerpo idénticos a `exo index`.
        let nota = exo::nota::parsea_nota(&abs)?.with_context(|| format!("{rel} sin permalink"))?;
        // Misma fórmula que `indexer::mtime_de`: si no cuadra al bit, `exo index`
        // reindexaría la nota y el bench mediría un indexado.
        let mtime = std::fs::metadata(&abs)?
            .modified()?
            .duration_since(UNIX_EPOCH)?
            .as_secs_f64();
        tx.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                nota.permalink,
                rel,
                nota.titulo,
                nota.tipo,
                mtime,
                EPOCH_COMMIT
            ],
        )?;
        tx.execute(
            "INSERT INTO notas_fts (titulo, cuerpo, permalink) VALUES (?1, ?2, ?3)",
            params![nota.titulo, nota.cuerpo, nota.permalink],
        )?;
        exo::aristas::reindexa_aristas_de_nota(&tx, &nota.permalink, &nota.cuerpo)?;
        for (orden, texto) in exo::trozos::trocea(&nota.cuerpo).iter().enumerate() {
            tx.execute(
                "INSERT INTO trozos (permalink, orden, texto) VALUES (?1, ?2, ?3)",
                params![nota.permalink, orden as i64, texto],
            )?;
            let id = tx.last_insert_rowid();
            exo::vectores::inserta(&tx, id, &vector_unitario(rng))?;
        }
    }
    tx.commit()?;
    exo::aristas::resuelve_destinos(&conn)?;
    let cuenta = |sql: &str| conn.query_row(sql, [], |r| r.get::<_, i64>(0));
    Ok((
        cuenta("SELECT count(*) FROM trozos")?,
        cuenta("SELECT count(*) FROM aristas")?,
        cuenta("SELECT count(*) FROM aristas WHERE destino_permalink IS NULL")?,
    ))
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        bail!("uso: kb_sintetica <N> <DIR> [SEMILLA]");
    }
    let n: usize = args[1].parse().context("N debe ser un entero")?;
    if n == 0 {
        bail!("N debe ser > 0");
    }
    let dir = PathBuf::from(&args[2]);
    let semilla: u64 = match args.get(3) {
        Some(s) => s.parse().context("SEMILLA debe ser un entero")?,
        None => 42,
    };
    if dir.exists() {
        bail!("{} ya existe: el generador no pisa nada", dir.display());
    }
    let kb = dir.join("kb");
    let db = dir.join("index.db");
    let mut rng = Xorshift(semilla.max(1));

    for carpeta in ["core", "projects", "log"] {
        std::fs::create_dir_all(kb.join(carpeta))?;
    }
    for i in 0..n {
        std::fs::write(kb.join(ruta_rel(i)), nota_md(i, n, &mut rng))?;
    }
    versiona(&kb)?;
    escribe_config(&dir, &kb, &db)?;
    let (trozos, aristas, rotas) = construye_indice(&kb, &db, n, &mut rng)?;
    println!(
        "kb_sintetica: N={n} semilla={semilla} trozos={trozos} aristas={aristas} sin_resolver={rotas}"
    );
    Ok(())
}
```

`evals/recall-coste/harness/bench.sh` (completo, `chmod +x`):

```bash
#!/usr/bin/env bash
# Bench de coste de la campaña A. Criterios, escenarios y predicciones:
# docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md
#
# Uso: evals/recall-coste/harness/bench.sh <etiqueta> [N ...]   (p.ej. baseline 174 1000 5000)
# Deja en evals/recall-coste/results/<etiqueta>/ un JSON de hyperfine, el rc y
# el stderr de una corrida directa por escenario, entorno.txt y resumen.tsv.
# Solo Linux. No toca ~/.exo ni la KB real: todo vive en BENCH_WORK.
set -uo pipefail
cd "$(dirname "$0")/../../.." || exit 1
REPO="$PWD"

ETIQUETA="${1:-}"
[ -n "$ETIQUETA" ] || { echo "uso: bench.sh <etiqueta> [N ...]" >&2; exit 1; }
shift
TAMANOS=("$@")
[ "${#TAMANOS[@]}" -gt 0 ] || TAMANOS=(174 1000 5000)
RUNS="${BENCH_RUNS:-20}"
# Solo palabras del vocabulario del generador: FTS es AND implícito y una
# palabra fuera del corpus («como», «de») daba «recall vacío» (exit 1) y
# confundía P1 (medido en el smoke del plan, 2026-09-13).
Q='trinquete techos indice memoria'
BIN="$REPO/engine/target/release/exo"
GEN="$REPO/engine/target/release/examples/kb_sintetica"
OUT="$REPO/evals/recall-coste/results/$ETIQUETA"

command -v hyperfine >/dev/null 2>&1 || { echo "bench: falta hyperfine" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "bench: falta jq" >&2; exit 1; }
[ ! -e "$OUT" ] || { echo "bench: $OUT ya existe — una etiqueta no se sobrescribe" >&2; exit 1; }

cargo build --release --locked --manifest-path engine/Cargo.toml --bin exo --example kb_sintetica || exit 1

WORK="${BENCH_WORK:-$(mktemp -d)}"
[ -n "${BENCH_KEEP:-}" ] || trap 'rm -rf "$WORK"' EXIT
mkdir -p "$OUT"

{
  echo "etiqueta $ETIQUETA"
  echo "fecha $(date -Iseconds)"
  echo "commit $(git rev-parse HEAD)"
  echo "src_y_scripts_vs_3c1918f $(git diff --quiet 3c1918f -- engine/src plugins/exo/scripts && echo vacio || echo CAMBIOS)"
  echo "exo $("$BIN" --version)"
  uname -a
  echo "nproc $(nproc)"
  echo "uptime $(uptime)"
  hyperfine --version
  jq --version
  git --version
} > "$OUT/entorno.txt"

mide() {  # $1=id  $2=comando para sh -c
  local id="$1" cmd="$2"
  sh -c "$cmd" > /dev/null 2> "$OUT/$id.stderr"
  echo $? > "$OUT/$id.rc"
  hyperfine --warmup 3 --runs "$RUNS" --ignore-failure \
    --export-json "$OUT/$id.json" "$cmd" > /dev/null 2>> "$OUT/$id.stderr"
}

for N in "${TAMANOS[@]}"; do
  D="$WORK/n$N"
  "$GEN" "$N" "$D" 42 > "$OUT/generador-n$N.txt" || { echo "bench: el generador falló para N=$N" >&2; exit 1; }
  export EXO_CONFIG="$D/config.toml"
  KB="$D/kb"
  DB="$D/index.db"

  # Fidelidad: el índice sintético tiene que verse FRESCO; si `exo index`
  # reindexa algo, el bench mediría un indexado y no un recall.
  "$BIN" index --kb "$KB" --db "$DB" --json > "$OUT/fidelidad-n$N.json" || exit 1
  jq -e --argjson n "$N" '.data.indexed == 0 and .data.skipped == $n' "$OUT/fidelidad-n$N.json" >/dev/null || {
    echo "bench: índice sintético no fresco para N=$N: $(cat "$OUT/fidelidad-n$N.json")" >&2; exit 1; }

  jq -n --arg p "$Q" '{prompt:$p, session_id:"bench-a"}' > "$D/prompt.json"

  mide "s3-index-sin-cambios-n$N" "\"$BIN\" index --db \"$DB\" --kb \"$KB\" --json"
  mide "s5-search-fts-n$N" "\"$BIN\" search --db \"$DB\" --type fts --limit 5 --json 'trinquete techos'"
  mide "s2-query-n$N" "\"$BIN\" recall --db \"$DB\" --kb \"$KB\" '--query=$Q' --min-similarity 0.40 --limit 4 --cap-bytes 4000 --json"
  mide "s1-query-refresh-n$N" "\"$BIN\" recall --db \"$DB\" --kb \"$KB\" '--query=$Q' --min-similarity 0.40 --limit 4 --cap-bytes 4000 --refresh --json"
  mide "s1b-query-refresh-sim0-n$N" "\"$BIN\" recall --db \"$DB\" --kb \"$KB\" '--query=$Q' --min-similarity 0.0 --limit 4 --cap-bytes 4000 --refresh --json"
  mide "s4-arranque-content-n$N" "\"$BIN\" recall --db \"$DB\" --kb \"$KB\" --content --note sint/core/core-index --limit 10 --cap-bytes 6144"
  mide "s6-hook-entero-n$N" "EXO_BIN=\"$BIN\" EXO_INDEX=\"$DB\" REFLEX_LOG_FILE=\"$D/reflex-log.jsonl\" \"$REPO/plugins/exo/scripts/recall-inject.sh\" < \"$D/prompt.json\""
  mide "s7-config-jq-n$N" "\"$BIN\" config --json | jq -r .data.kb.name"
  mide "s9-git-log-una-nota-n$N" "git -C \"$KB\" log -1 --format=%ct -- log/nota-00011.md"

  # s8 va el último: deja una nota modificada y staged.
  printf '\nlinea extra del bench\n' >> "$KB/log/nota-00011.md"
  git -C "$KB" add log/nota-00011.md
  mide "s8-kb-precommit-n$N" "cd \"$KB\" && EXO_BIN=\"$BIN\" \"$REPO/plugins/exo/scripts/kb-precommit.sh\""

  [ -n "${BENCH_KEEP:-}" ] || rm -rf "$D"
done

# s10, independiente de N: el filtro de exo-recall.sh:111-112 sobre logs de ~410 B/línea.
for L in 2174 20000 200000; do
  F="$WORK/reflex-log-$L.jsonl"
  awk -v n="$L" 'BEGIN {
    pad = sprintf("%260s", ""); gsub(/ /, "x", pad)
    for (i = 0; i < n; i++)
      printf "{\"ts\":\"2026-09-13T00:00:00Z\",\"reflex\":\"recall-inject-emitted\",\"session_id\":\"s%d\",\"agent_id\":\"\",\"agent_type\":\"\",\"tool\":\"\",\"payload\":\"n_hits=3 bytes=900 permalinks=%s\"}\n", i % 500, pad
  }' > "$F"
  mide "s10-jq-log-l$L" "jq -r --arg sid s7 'select(.session_id==\$sid) | .reflex' \"$F\" | sort -u"
done

{
  printf 'id\tp50_ms\tp95_ms\tmean_ms\tcorridas_fallidas\tcorridas\trc_directa\n'
  for f in "$OUT"/s*.json; do
    id="$(basename "$f" .json)"
    rc="$(cat "$OUT/$id.rc" 2>/dev/null)"
    jq -r --arg id "$id" --arg rc "$rc" '
      .results[0] as $r | ($r.times | sort) as $t | ($t | length) as $n
      | [ $id,
          ($t[(($n - 1) * 0.5 | floor)] * 1000 | floor),
          ($t[(($n - 1) * 0.95 | floor)] * 1000 | floor),
          ($r.mean * 1000 | floor),
          ([ ($r.exit_codes // [])[] | select(. != 0) ] | length),
          $n, $rc ] | map(tostring) | @tsv' "$f"
  done | sort
} > "$OUT/resumen.tsv"

column -t -s "$(printf '\t')" "$OUT/resumen.tsv"
```

`evals/recall-coste/harness/compara.sh` (completo, `chmod +x`):

```bash
#!/usr/bin/env bash
# Aplica los criterios del pre-registro
# (docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md). Es mecánico
# a propósito: el veredicto no se redacta a mano.
#   compara.sh <base>             predicciones P1–P4 y puertas de las Tasks 10 y 12
#   compara.sh <base> <despues>   tabla antes/después y criterios C-*
set -uo pipefail
cd "$(dirname "$0")/../../.." || exit 1
R="evals/recall-coste/results"
BASE="${1:-}"
DESP="${2:-}"
[ -n "$BASE" ] || { echo "uso: compara.sh <base> [despues]" >&2; exit 1; }
for e in "$BASE" ${DESP:+"$DESP"}; do
  [ -f "$R/$e/resumen.tsv" ] || { echo "compara: falta $R/$e/resumen.tsv" >&2; exit 1; }
done

# campo <etiqueta> <id> <col>: 2=p50 3=p95 5=fallidas 6=corridas 7=rc_directa; NA si no está
campo() { awk -F'\t' -v id="$2" -v c="$3" '$1 == id { print $c; f = 1 } END { if (!f) print "NA" }' "$R/$1/resumen.tsv"; }
# cumple "<expr awk>": 0 si la expresión es verdadera (bash no hace flotantes)
cumple() { case "$1" in *NA*) return 1 ;; esac; awk "BEGIN { exit !($1) }"; }
knn_tope() { grep -q 'k value in knn query too large' "$R/$1/$2.stderr" 2>/dev/null; }
linea() { printf '%s\t%s\n' "$1" "$2"; }

if [ -z "$DESP" ]; then
  p1=PASA
  for s in s1-query-refresh s1b-query-refresh-sim0 s2-query; do
    for n in 1000 5000; do
      { [ "$(campo "$BASE" "$s-n$n" 7)" = 1 ] && knn_tope "$BASE" "$s-n$n"; } || p1=FALLA
    done
    [ "$(campo "$BASE" "$s-n174" 7)" = 0 ] || p1=FALLA
  done
  linea "P1 (H27: tope KNN a N>=1000, ok a 174)" "$p1"
  s2=$(campo "$BASE" s2-query-n174 2); s1=$(campo "$BASE" s1-query-refresh-n174 2); s7=$(campo "$BASE" s7-config-jq-n174 2)
  cumple "$s2 >= 800 && $s2 <= 1300" && linea "P2 (s2 n174 p50=$s2)" CUMPLIDA || linea "P2 (s2 n174 p50=$s2)" FALLIDA
  cumple "$s1 - $s2 < 60" && linea "P3 (refresh n174 = $s1-$s2)" CUMPLIDA || linea "P3 (refresh n174 = $s1-$s2)" FALLIDA
  cumple "$s7 < 20" && linea "P4 (s7 n174 p50=$s7)" CUMPLIDA || linea "P4 (s7 n174 p50=$s7)" FALLIDA
  s4=$(campo "$BASE" s4-arranque-content-n5000 3)
  cumple "$s4 > 250" && linea "PUERTA C-H17a (s4 n5000 p95=$s4)" "ABIERTA: Task 10 procede" || linea "PUERTA C-H17a (s4 n5000 p95=$s4)" "CERRADA: Task 10 no se ejecuta"
  cumple "$s7 > 30" && linea "PUERTA C-H10 Linux (s7 n174 p50=$s7)" "ABIERTA" || linea "PUERTA C-H10 Linux (s7 n174 p50=$s7)" "CERRADA (falta W11, D5)"
  exit 0
fi

printf '| id | p50 antes | p50 después | p95 antes | p95 después | fallidas antes | fallidas después |\n|---|---|---|---|---|---|---|\n'
awk -F'\t' 'FNR == 1 { next }
  NR == FNR { a50[$1] = $2; a95[$1] = $3; af[$1] = $5; next }
  { x50 = ($1 in a50) ? a50[$1] : "NA"; x95 = ($1 in a95) ? a95[$1] : "NA"; xf = ($1 in af) ? af[$1] : "NA"
    printf "| %s | %s | %s | %s | %s | %s | %s |\n", $1, x50, $2, x95, $3, xf, $5 }' \
  "$R/$BASE/resumen.tsv" "$R/$DESP/resumen.tsv"
echo

c=PASA
for s in s1-query-refresh s1b-query-refresh-sim0 s2-query; do
  for n in 1000 5000; do
    [ "$(campo "$DESP" "$s-n$n" 5)" = 0 ] && [ "$(campo "$DESP" "$s-n$n" 7)" = 0 ] || c=FALLA
  done
done
linea C-H27 "$c"

a=$(campo "$BASE" s3-index-sin-cambios-n5000 2); b=$(campo "$DESP" s3-index-sin-cambios-n5000 2)
if cumple "$a <= 50"; then linea "C-H4 (s3 n5000 $a→$b)" VACÍO
elif cumple "$b <= 0.6 * $a"; then linea "C-H4 (s3 n5000 $a→$b)" PASA
else linea "C-H4 (s3 n5000 $a→$b)" FALLA; fi

a=$(campo "$BASE" s4-arranque-content-n5000 3); b=$(campo "$DESP" s4-arranque-content-n5000 3)
if ! cumple "$a > 250"; then linea "C-H17a (s4 n5000 p95 $a→$b)" "NO APLICA (puerta cerrada)"
elif cumple "$b <= 0.5 * $a"; then linea "C-H17a (s4 n5000 p95 $a→$b)" PASA
else linea "C-H17a (s4 n5000 p95 $a→$b)" FALLA; fi

g=$(campo "$DESP" s2-query-n5000 2); p=$(campo "$DESP" s2-query-n174 2)
cumple "$g - $p > 250" && linea "C-H17b (s2 n5000-n174 = $g-$p)" "DERIVAR a C/backlog" || linea "C-H17b (s2 n5000-n174 = $g-$p)" "NO DERIVAR"

k=$(campo "$DESP" s8-kb-precommit-n5000 2)
cumple "$k > 2000" && linea "C-H23 (s8 n5000 p50=$k)" "ABRIR ITEM backlog" || linea "C-H23 (s8 n5000 p50=$k)" "NO ABRIR"

c=PASA
for n in 174 1000 5000; do
  for s in s5-search-fts s3-index-sin-cambios; do
    a=$(campo "$BASE" "$s-n$n" 2); b=$(campo "$DESP" "$s-n$n" 2)
    cumple "$b <= 1.2 * $a + 2" || { c=FALLA; linea "  regresión $s-n$n" "$a→$b"; }
  done
done
linea C-noregresión "$c"
```

- [ ] **Step 4: Correr el smoke y verificar que pasa**

Run: el bloque del Step 1.
Expected: `SMOKE-OK`. Anotar además la línea que imprime el generador para
N=58. Debe decir `trozos=1102` (58×19) y `aristas` cerca de 58×4. Si
`trozos` ≠ 1102, el troceado no da un trozo por párrafo: **para** y revisa
`CHARS_POR_PARRAFO` antes de medir nada.

Run: `bash -n evals/recall-coste/harness/bench.sh && bash -n evals/recall-coste/harness/compara.sh && BENCH_RUNS=2 evals/recall-coste/harness/bench.sh smoke 58 && evals/recall-coste/harness/compara.sh smoke; rm -rf evals/recall-coste/results/smoke`
Expected: la tabla de `resumen.tsv` con ids `s1…s10` y `rc_directa` 0 en
`s1`, `s2`, `s3`, `s4`, `s5`, `s7` y `s9`. `s8` puede salir con 1 si `exo
budget` rechaza notas sintéticas grandes, y eso se mide igual. `compara.sh`
imprime P1 FALLA para este tamaño, y es lo esperado: no hay N=1000.

Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
Expected: sin salida, exit 0.

- [ ] **Step 5: Commit**

```bash
chmod +x evals/recall-coste/harness/bench.sh evals/recall-coste/harness/compara.sh
git add engine/examples/kb_sintetica.rs evals/recall-coste/harness/bench.sh evals/recall-coste/harness/compara.sh
git commit -m "bench(campaña A): KB sintética con la forma de la real, bench y comparador pre-registrados"
```

---

### Task 2: línea base, medida antes de tocar código

**Lane:** mecánica · **Oráculo:** `evals/recall-coste/harness/bench.sh baseline 174 1000 5000 && evals/recall-coste/harness/compara.sh baseline`

**Files:**
- Create: `evals/recall-coste/results/baseline/` (lo genera el bench)
- Create: `evals/recall-coste/results/baseline/predicciones.txt`

**Interfaces:**
- Consumes: `bench.sh` y `compara.sh` de la Task 1.
- Produces: `evals/recall-coste/results/baseline/resumen.tsv`, que usan las
  puertas de las Tasks 10 y 12 y la Task 14.

- [ ] **Step 1: Comprobar las condiciones pre-registradas**

Run: `git diff --quiet 3c1918f -- engine/src plugins/exo/scripts && echo SRC-BASE; uptime; pgrep -fa 'cargo|exo ' | grep -v pgrep`
Expected: `SRC-BASE`, la carga del primer minuto por debajo de 1,0 y ningún
`cargo` ni `exo` corriendo. Si algo no se cumple, **no mides**: espera y
repite el paso.

- [ ] **Step 2: Correr el bench**

Run: `evals/recall-coste/harness/bench.sh baseline 174 1000 5000`
Expected: exit 0 y la tabla. Esperado por P1: `rc_directa` = 1 y
`corridas_fallidas` = 20 en `s1`, `s1b` y `s2` para n1000 y n5000.

- [ ] **Step 3: Aplicar las predicciones y las puertas**

Run: `evals/recall-coste/harness/compara.sh baseline | tee evals/recall-coste/results/baseline/predicciones.txt`
Expected: siete líneas: P1–P4 con su veredicto y las dos PUERTAS. Una
predicción FALLIDA **no se corrige ni se repite**: queda así en el fichero y
se comenta en el veredicto de la Task 14. La excepción es P1 con
`src_y_scripts_vs_3c1918f CAMBIOS` en `entorno.txt`: en ese caso la línea
base no vale y se repite desde el Step 1.

- [ ] **Step 4: Commit**

```bash
git add evals/recall-coste/results/baseline
git commit -m "bench(campaña A): línea base antes de tocar código"
```

---

### Task 3: `knn` sin el tope de 4.096 de vec0

**Lane:** mecánica · **Hallazgo:** H27 · **Oráculo:** `cd engine && cargo test --release --lib vectores` + O-engine

**Files:**
- Modify: `engine/src/vectores.rs:83-105` (`knn`) y su `mod tests` (`:107-176`)

**Interfaces:**
- Consumes: `serializa(&[f32]) -> Vec<u8>` y `VecinoKnn { rowid: i64, distancia: f64 }`, ambos en `vectores.rs`.
- Produces: `pub fn knn(conn: &Connection, query: &[f32], k: usize) -> Result<Vec<VecinoKnn>>`,
  con la **misma firma**: para `k ≤ 4096` sigue el camino de vec0 sin cambios; por encima,
  un barrido completo. Además, `fn barrido_completo(conn: &Connection, query: &[f32], k: usize) -> Result<Vec<VecinoKnn>>`
  y `const K_MAX_VEC0: usize = 4096`. `buscador.rs` no se toca.

Por qué no hay que re-evaluar retrieval: con los 3.290 trozos de hoy, `k =
COUNT(*)` ≤ 4096, así que el camino ejecutado es **byte a byte el mismo** que
antes. Por encima del tope, `vec_distance_l2` es la misma función de distancia
que usa el KNN de vec0 para la métrica L2 (`sqlite-vec.c:1235` y `:6879`, las
dos llaman a `distance_l2_sqr_float`). Solo pueden cambiar los empates, y
`busca_vector` los desempata por permalink (`buscador.rs:319-323`).

- [ ] **Step 1: Escribir los tests que fallan**

Añadir dentro de `mod tests` de `engine/src/vectores.rs`, después de
`knn_sobre_tabla_vacia_devuelve_cero_resultados`:

```rust
    /// H27: vec0 0.1.9 rechaza `k > 4096` (`SQLITE_VEC_VEC0_K_MAX`) y
    /// `busca_vector` pide `k = COUNT(*)`. La KB real iba por 3.290 trozos el
    /// 2026-09-13: al cruzar el tope, vector/hybrid/recall --query salían con
    /// exit 1 en cada prompt.
    #[test]
    fn knn_por_encima_del_tope_de_vec0_devuelve_todos_los_vecinos() {
        let conn = db_con_schema();
        let n = K_MAX_VEC0 + 1;
        for i in 0..n {
            inserta(&conn, i as i64 + 1, &vector_768(i as f32)).unwrap();
        }
        let vecinos = knn(&conn, &vector_768(0.0), n).expect("k > 4096 no puede ser un error");
        assert_eq!(vecinos.len(), n);
        assert_eq!(vecinos[0].rowid, 1, "el idéntico a la query va primero");
        assert!(
            vecinos.windows(2).all(|w| w[0].distancia <= w[1].distancia),
            "orden ascendente por distancia"
        );
    }

    /// El barrido tiene que dar la MISMA distancia que el KNN de vec0 para cada
    /// rowid: si difiriera, cruzar el tope cambiaría el ranking y los umbrales.
    #[test]
    fn barrido_y_knn_de_vec0_dan_las_mismas_distancias() {
        let conn = db_con_schema();
        for i in 0..50 {
            inserta(&conn, i + 1, &vector_768(i as f32 * 0.1)).unwrap();
        }
        let q = vector_768(2.05);
        let de_vec0: std::collections::HashMap<i64, f64> = knn(&conn, &q, 50)
            .unwrap()
            .into_iter()
            .map(|v| (v.rowid, v.distancia))
            .collect();
        let barrido = barrido_completo(&conn, &q, 50).unwrap();
        assert_eq!(barrido.len(), 50);
        for v in &barrido {
            assert!(
                (de_vec0[&v.rowid] - v.distancia).abs() < 1e-9,
                "rowid {}: vec0={} barrido={}",
                v.rowid,
                de_vec0[&v.rowid],
                v.distancia
            );
        }
    }
```

- [ ] **Step 2: Verificar que falla**

Run: `cd engine && cargo test --release --lib vectores`
Expected: FAIL de compilación: `cannot find value `K_MAX_VEC0``, y ``cannot find function `barrido_completo``.
Para ver el rojo de comportamiento: añade temporalmente `const K_MAX_VEC0:
usize = 4096;` sin tocar nada más y el primer test falla con `k value in knn
query too large, provided 4097 and the limit is 4096`. Luego quita la
constante y sigue con el Step 3.

- [ ] **Step 3: Implementación mínima**

En `engine/src/vectores.rs`, sustituir `pub fn knn` (líneas 83-105, doc
incluida) por:

```rust
/// Tope duro de `k` en el KNN de vec0: `#define SQLITE_VEC_VEC0_K_MAX 4096`
/// (sqlite-vec 0.1.9, `sqlite-vec.c:7111`). Pedir más es `SQLITE_ERROR`.
const K_MAX_VEC0: usize = 4096;

/// KNN sobre `vectores`: los `k` vecinos más cercanos a `query`, ordenados
/// por la `distance` nativa de vec0. `vectores` vacía ⇒ `Ok(vec![])`, jamás
/// error. Hasta `K_MAX_VEC0` usa el KNN de vec0 (`embedding MATCH ?1 AND k =
/// ?2`); por encima, `barrido_completo` (H27): `busca_vector` pide
/// `k = COUNT(*)` y una KB de más de 4.096 trozos dejaba el arm vector
/// muerto con exit 1.
pub fn knn(conn: &Connection, query: &[f32], k: usize) -> Result<Vec<VecinoKnn>> {
    if k > K_MAX_VEC0 {
        return barrido_completo(conn, query, k);
    }
    let mut stmt = conn.prepare(
        "SELECT rowid, distance
         FROM vectores
         WHERE embedding MATCH ?1 AND k = ?2
         ORDER BY distance",
    )?;
    let filas = stmt
        .query_map(params![serializa(query), k as i64], |r| {
            Ok(VecinoKnn {
                rowid: r.get(0)?,
                distancia: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("leer resultados KNN de vectores")?;
    Ok(filas)
}

/// Mismo contrato que el KNN de vec0 pero sin su tope: la distancia se
/// calcula fila a fila con `vec_distance_l2`, la MISMA función que usa vec0
/// para la métrica L2 (`sqlite-vec.c:1235` y `:6879`, ambas vía
/// `distance_l2_sqr_float`), así que las distancias coinciden; solo el orden
/// entre empates puede variar. Coste lineal, igual que el KNN sin partición.
fn barrido_completo(conn: &Connection, query: &[f32], k: usize) -> Result<Vec<VecinoKnn>> {
    let mut stmt = conn.prepare(
        "SELECT rowid, vec_distance_l2(embedding, ?1) AS distancia
         FROM vectores
         ORDER BY distancia
         LIMIT ?2",
    )?;
    let limite = i64::try_from(k).unwrap_or(i64::MAX);
    let filas = stmt
        .query_map(params![serializa(query), limite], |r| {
            Ok(VecinoKnn {
                rowid: r.get(0)?,
                distancia: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("leer barrido completo de vectores")?;
    Ok(filas)
}
```

No toques la doc de `VecinoKnn` (`:73-77`) ni `buscador.rs:212-228`: su
afirmación «L2²» es H28 y la adjudica la campaña C.

- [ ] **Step 4: Verificar que pasa**

Run: `cd engine && cargo test --release --lib vectores`
Expected: PASS, 6 tests en `vectores::tests`.

Run, como spot-check sobre la KB sintética por encima del tope (300 × 19 = 5.700 trozos):
`T="$(mktemp -d)"; engine/target/release/examples/kb_sintetica 300 "$T/n" 42 >/dev/null && EXO_CONFIG="$T/n/config.toml" engine/target/release/exo recall --db "$T/n/index.db" --kb "$T/n/kb" --query=trinquete --min-similarity 0.0 --limit 4 --json | jq -e '.data.notes | length == 4' && echo H27-OK; rm -rf "$T"`
(antes de esto, `cargo build --release --manifest-path engine/Cargo.toml --bin exo --example kb_sintetica`)
Expected: `H27-OK`.

Run: O-engine. Expected: exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/vectores.rs
git commit -m "fix(vectores): knn por encima del tope de 4096 de vec0 con barrido de vec_distance_l2 (campaña A, H27)"
```

---

### Task 4: `exo recall` publica `warnings`, `elapsed_s` y `refresh_s`

**Lane:** mecánica, **con gate de superficie**: envelope aditivo, así que
según el config §«Ejecución de gates» lo adjudica el consultor · **Hallazgos:** H2 H3 ·
**Oráculo:** `cd engine && cargo test --release --test recall --test recall_avisos_cli --test contrato_envelope` + O-engine

**Files:**
- Modify: `engine/src/recall.rs` (`Recall` en `:49-60`, `RecallBruto` en `:65-69`, `aplica_cap` en `:173-183`, `renderiza` en `:187-189`, `recall_arranque` en `:274-278`, `recall_consulta` en `:495-520`)
- Modify: `engine/src/main.rs:816-888` (`recall_cmd`)
- Modify: `engine/tests/contrato_envelope.rs:14-52`
- Test: `engine/tests/recall.rs` (añadir), `engine/tests/recall_avisos_cli.rs` (nuevo)

**Interfaces:**
- Consumes: `exo::buscador::Busqueda { elapsed_s: f64, results: Vec<Resultado>, avisos: Vec<String>, .. }` (`buscador.rs:36-51`), sin cambios.
- Produces:
  - `exo::recall::Recall` gana `pub elapsed_s: Option<f64>`, `pub refresh_s: Option<f64>` y `pub avisos: Vec<String>`, serializados como `elapsed_s`, `refresh_s` (`null` si `None`) y `warnings` (omitido si está vacío).
  - `exo::recall::RecallBruto` gana `pub avisos: Vec<String>` y `pub elapsed_s: Option<f64>`.
  - CLI: `exo recall` escribe `aviso: <texto>` en stderr por cada aviso, **antes** del `bail!` de «recall vacío».
  - La Task 5 consume `.data.warnings`, `.data.elapsed_s` y `.data.refresh_s`.

- [ ] **Step 1: Escribir los tests que fallan**

Añadir a `engine/tests/recall.rs`, al final:

```rust
/// Vacía `vectores` sobre una DB indexada: el estado de un embed abortado a
/// medias (mismo helper que `tests/buscador.rs:66-74`).
fn vacia_vectores(db: &Path) {
    let conn = exo::abre_db(db).unwrap();
    conn.execute("DELETE FROM vectores", []).unwrap();
}

/// H2: `recall_consulta` tiraba `resultado.avisos`, así que el hook de cada
/// prompt servía FTS puro etiquetado hybrid sin que nadie se enterara.
#[test]
fn recall_consulta_propaga_los_avisos_y_el_tiempo_de_la_busqueda() {
    let kb = kb_arranque();
    let (_db_dir, db) = db_temporal();

    common::con_config(kb.path(), "kb-test", &db, || {
        indexa(kb.path(), &db).unwrap();
        vacia_vectores(&db);

        let bruto = recall_consulta(&db, "contenido", 5, Some(0.0), 0.0, 0.6).unwrap();
        assert!(!bruto.notas.is_empty(), "precondición: FTS encuentra 'contenido'");
        assert!(
            bruto.avisos.iter().any(|a| a.contains("INERTE")),
            "el aviso del arm vector tiene que llegar al recall: {:?}",
            bruto.avisos
        );
        assert!(bruto.elapsed_s.is_some_and(|s| s >= 0.0));

        let r = renderiza(bruto, 4000);
        let v = serde_json::to_value(&r.recall).unwrap();
        assert!(
            v["warnings"].as_array().is_some_and(|a| !a.is_empty()),
            "warnings en el envelope: {v}"
        );
        assert!(v["elapsed_s"].is_number(), "{v}");
        assert!(v["refresh_s"].is_null(), "refresh_s lo pone el CLI, no la librería: {v}");
    });
}

#[test]
fn recall_arranque_no_trae_avisos_ni_tiempo_de_busqueda() {
    let kb = kb_arranque();
    let (_db_dir, db) = db_temporal();

    common::con_config(kb.path(), "kb-test", &db, || {
        indexa(kb.path(), &db).unwrap();
        let bruto = recall_arranque(&db, kb.path(), 5).unwrap();
        let r = renderiza(bruto, 4000);
        let v = serde_json::to_value(&r.recall).unwrap();
        assert!(v["elapsed_s"].is_null(), "{v}");
        assert!(v.get("warnings").is_none(), "sin avisos la clave se omite: {v}");
    });
}
```

`engine/tests/recall_avisos_cli.rs` (nuevo):

```rust
//! H2/H3 en la superficie de CLI: `exo recall --query` avisa por stderr y
//! publica `warnings`, `elapsed_s` y `refresh_s` en el envelope.
mod common;

use std::process::Command;

#[test]
fn recall_query_con_arm_vector_inerte_avisa_y_publica_tiempos() {
    let kb = tempfile::tempdir().unwrap();
    std::fs::write(
        kb.path().join("a.md"),
        "---\npermalink: kb-test/a\ntitle: A\n---\ncontenido buscable de la nota a\n",
    )
    .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("i.db");
    let cfg = dir.path().join("config.toml");
    std::fs::write(&cfg, common::render_config(kb.path(), "kb-test", &db)).unwrap();

    let idx = Command::new(env!("CARGO_BIN_EXE_exo"))
        .args(["index", "--kb"])
        .arg(kb.path())
        .arg("--db")
        .arg(&db)
        .env("EXO_CONFIG", &cfg)
        .output()
        .unwrap();
    assert!(idx.status.success(), "{}", String::from_utf8_lossy(&idx.stderr));
    exo::abre_db(&db)
        .unwrap()
        .execute("DELETE FROM vectores", [])
        .unwrap();

    let recall = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_exo"))
            .args(["recall", "--kb"])
            .arg(kb.path())
            .arg("--db")
            .arg(&db)
            .args(["--query=buscable", "--min-similarity", "0.0", "--json"])
            .args(extra)
            .env("EXO_CONFIG", &cfg)
            .output()
            .unwrap()
    };

    let sin = recall(&[]);
    assert!(sin.status.success(), "{}", String::from_utf8_lossy(&sin.stderr));
    let err = String::from_utf8_lossy(&sin.stderr);
    assert!(
        err.lines().any(|l| l.starts_with("aviso: ") && l.contains("INERTE")),
        "el aviso sale por stderr: {err}"
    );
    let v: serde_json::Value = serde_json::from_slice(&sin.stdout).unwrap();
    assert!(
        v["data"]["warnings"][0].as_str().is_some_and(|w| w.contains("INERTE")),
        "{v}"
    );
    assert!(v["data"]["elapsed_s"].is_number(), "{v}");
    assert!(v["data"]["refresh_s"].is_null(), "sin --refresh: {v}");

    let con = recall(&["--refresh"]);
    assert!(con.status.success(), "{}", String::from_utf8_lossy(&con.stderr));
    let v: serde_json::Value = serde_json::from_slice(&con.stdout).unwrap();
    assert!(v["data"]["refresh_s"].is_number(), "con --refresh: {v}");
}
```

En `engine/tests/contrato_envelope.rs`, dentro de `las_claves_de_recall_estan_en_ingles`,
sustituir el literal `exo::recall::Recall { … }` (líneas 15-28) por:

```rust
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
        elapsed_s: Some(0.5),
        refresh_s: None,
        avisos: vec!["arm vector INERTE".into()],
    };
```

y añadir antes del cierre de la función:

```rust
    assert_eq!(v["elapsed_s"], 0.5);
    assert!(v["refresh_s"].is_null());
    assert_eq!(v["warnings"][0], "arm vector INERTE");
    assert!(v.get("avisos").is_none(), "sobrevive `avisos`");
```

- [ ] **Step 2: Verificar que falla**

Run: `cd engine && cargo test --release --test recall --test recall_avisos_cli --test contrato_envelope`
Expected: FAIL de compilación en `recall.rs` y `contrato_envelope.rs`
(`no field `avisos` on type `RecallBruto``, `struct `Recall` has no field
named `elapsed_s``). `recall_avisos_cli` compila y falla en la aserción
`el aviso sale por stderr`.

- [ ] **Step 3: Implementación mínima**

`engine/src/recall.rs`:

(a) En `pub struct Recall`, después de `pub notas: Vec<NotaRecall>,`:

```rust
    /// Segundos de la búsqueda hybrid (`Busqueda::elapsed_s`, carga del
    /// modelo incluida). `None` en modo arranque. Aditivo (H2): no sube
    /// `SCHEMA_VERSION`.
    pub elapsed_s: Option<f64>,
    /// Segundos del refresco previo (`--refresh`), `None` sin él. Lo rellena
    /// `main.rs::recall_cmd`, que es quien refresca. Aditivo (H3).
    pub refresh_s: Option<f64>,
    /// Degradaciones de la búsqueda (hoy: cobertura del arm vector), las
    /// mismas que `exo search` ya publica. Omitido si está vacío (H2).
    #[serde(rename = "warnings", skip_serializing_if = "Vec::is_empty")]
    pub avisos: Vec<String>,
```

(b) En `pub struct RecallBruto`, después de `pub notas: Vec<NotaRecall>,`:

```rust
    pub avisos: Vec<String>,
    pub elapsed_s: Option<f64>,
```

(c) En `aplica_cap`, en el literal `recall: Recall { … }` (`:175-181`), después de `notas: notas_finales,`:

```rust
            elapsed_s: None,
            refresh_s: None,
            avisos: Vec::new(),
```

(d) Sustituir `renderiza`:

```rust
/// Punto de entrada único: renderiza un `RecallBruto` aplicando el cap. Los
/// avisos y el tiempo no dependen del cap: pasan tal cual.
pub fn renderiza(bruto: RecallBruto, cap_bytes: usize) -> ResultadoCap {
    let mut r = aplica_cap(&bruto.modo, bruto.query, bruto.notas, cap_bytes);
    r.recall.elapsed_s = bruto.elapsed_s;
    r.recall.avisos = bruto.avisos;
    r
}
```

(e) En `recall_arranque`, en el `Ok(RecallBruto { … })` final, después de `notas,`:

```rust
        avisos: Vec::new(),
        elapsed_s: None,
```

(f) En `recall_consulta`, justo después de `let resultado = crate::buscador::busca_hybrid(…)?;`:

```rust
    // H2: los avisos y el tiempo se rescatan ANTES de consumir `results`.
    let avisos = resultado.avisos;
    let elapsed_s = Some(resultado.elapsed_s);
```

y en su `Ok(RecallBruto { … })`, después de `notas,`:

```rust
        avisos,
        elapsed_s,
```

`engine/src/main.rs`, en `recall_cmd`:

(g) Sustituir el bloque `if args.refresca { … }` por:

```rust
    let mut refresh_s = None;
    if args.refresca {
        // El resumen va a stderr: stdout es exclusivo del envelope/bloque
        // (contrato §4), y el hook consume stdout tal cual.
        let inicio = std::time::Instant::now();
        let resumen = exo::refresca_indice(&kb, &db)
            .context("refrescar el índice antes del recall (--refresh)")?;
        refresh_s = Some(inicio.elapsed().as_secs_f64());
        if resumen.indexadas > 0 || resumen.borradas > 0 {
            eprintln!(
                "refresca: indexadas={} borradas={} saltadas={}",
                resumen.indexadas, resumen.borradas, resumen.saltadas
            );
        }
    }
```

(h) Sustituir `let resultado = renderiza(bruto, args.cap_bytes);` por:

```rust
    let mut resultado = renderiza(bruto, args.cap_bytes);
    resultado.recall.refresh_s = refresh_s;

    // H2: los avisos van a stderr SIEMPRE, igual que en `busca_cmd`, y ANTES
    // del bail de «recall vacío»: un arm vector INERTE sin hits FTS es justo
    // el caso en que más importa verlo, y el que antes se perdía entero.
    for aviso in &resultado.recall.avisos {
        eprintln!("aviso: {aviso}");
    }
```

- [ ] **Step 4: Verificar que pasa**

Run: `cd engine && cargo test --release --test recall --test recall_avisos_cli --test contrato_envelope && cargo test --release --lib recall`
Expected: PASS (los tests unitarios de `recall.rs` siguen verdes; `recall_json_forma_del_contrato` no cambia).

Run: O-engine. Expected: exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/recall.rs engine/src/main.rs engine/tests/recall.rs engine/tests/recall_avisos_cli.rs engine/tests/contrato_envelope.rs
git commit -m "feat(recall): warnings, elapsed_s y refresh_s en el envelope y avisos por stderr (campaña A, H2/H3)"
```

---

### Task 5: `recall-inject.sh` registra avisos y tiempos

**Lane:** mecánica · **Hallazgos:** H2 H3 · **Oráculo:** `bash plugins/exo/scripts/test-recall-inject.sh` + O-plugin + O-contrato

**Files:**
- Modify: `plugins/exo/scripts/recall-inject.sh:149-150` (captura de stderr), `:159-173` (rama rc≠0), `:186-192` (truncado), `:339` (log `emitted`)
- Modify: `plugins/exo/scripts/test-contrato-engine.sh` (predicado nuevo antes del de `schema_version == 2`, `:146`)
- Test: `plugins/exo/scripts/test-recall-inject.sh` (añadir antes del resumen final `printf '\n%d passed, %d failed\n'`)

**Interfaces:**
- Consumes: de la Task 4, `.data.warnings` (array, puede faltar), `.data.elapsed_s` y `.data.refresh_s` (número o null), y líneas `aviso: …` en stderr.
- Produces, en el log de reflejos:
  - `recall-inject-degraded` con `reason=engine-warning w=<≤160 chars>` (el bloque se emite igual);
  - `reason=empty warn=…` / `reason=error … warn=…` cuando rc≠0 trae avisos;
  - `recall-inject-emitted` con payload `n_hits=N bytes=B elapsed_ms=E refresh_ms=R permalinks=…`, en ese orden. Lo parsea la Task 13.

- [ ] **Step 1: Escribir los tests que fallan**

Añadir a `plugins/exo/scripts/test-recall-inject.sh`, justo antes de `printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"`:

```bash
# ------------------------------------ H2/H3: avisos y tiempos del engine ---
AVISA="$TMP/exo-avisa"
cat > "$AVISA" <<'EOF'
#!/usr/bin/env bash
echo "aviso: arm vector INERTE: 0 vectores para 9 trozos." >&2
cat <<'JSON'
{"command":"recall","data":{"cap_bytes":4000,"mode":"consulta","elapsed_s":0.9876,"refresh_s":0.0123,"warnings":["arm vector INERTE: 0 vectores para 9 trozos."],"notes":[
{"permalink":"kb-demo/log/kbx-bitacora","path":"/kb/log/kbx-bitacora.md","score":0.5,"snippet":"bitacora de kbx","tier":null,"title":"kbx-bitacora"}
],"query":"kbx","truncated":false},"schema_version":2}
JSON
EOF
chmod +x "$AVISA"
: > "$REFLEX_LOG_FILE"
run_hook "kbx trinquete" "$AVISA"
BL_AV="$(printf '%s' "$HOOK_OUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"
if [ "$HOOK_RC" -eq 0 ] && contains "$BL_AV" "kbx-bitacora"; then pass "H2: con avisos el bloque se sigue emitiendo"
else fail "H2: con avisos el bloque se sigue emitiendo" "rc=$HOOK_RC out='$HOOK_OUT'"; fi
if grep 'recall-inject-degraded' "$REFLEX_LOG_FILE" 2>/dev/null | grep -q 'reason=engine-warning w=arm vector INERTE'; then
  pass "H2: el aviso del engine deja rastro engine-warning"
else fail "H2: el aviso del engine deja rastro engine-warning" "$(cat "$REFLEX_LOG_FILE" 2>/dev/null)"; fi
PL_AV="$(jq -r 'select(.reflex=="recall-inject-emitted") | .payload' "$REFLEX_LOG_FILE" 2>/dev/null | tail -1)"
if contains "$PL_AV" "elapsed_ms=987 refresh_ms=12 permalinks="; then pass "H3: emitted lleva elapsed_ms y refresh_ms antes de permalinks"
else fail "H3: emitted lleva elapsed_ms y refresh_ms antes de permalinks" "payload='$PL_AV'"; fi

: > "$REFLEX_LOG_FILE"
run_hook "kbx trinquete" "$CUATRO"
if grep -q 'engine-warning' "$REFLEX_LOG_FILE" 2>/dev/null; then fail "H2: sin avisos no hay engine-warning" "$(cat "$REFLEX_LOG_FILE")"
else pass "H2: sin avisos no hay engine-warning"; fi

# rc=1 con un aviso de más de 300 B delante de «recall vacío»: antes el
# `head -c 300` del stderr se quedaba solo con el aviso y lo logueaba como error.
VACIO_AVISA="$TMP/exo-vacio-avisa"
cat > "$VACIO_AVISA" <<'EOF'
#!/usr/bin/env bash
printf 'aviso: arm vector INERTE: 0 vectores para 3290 trozos.%0300d\n' 0 >&2
echo "error: recall vacío (modo consulta): sin notas para el bloque" >&2
exit 1
EOF
chmod +x "$VACIO_AVISA"
: > "$REFLEX_LOG_FILE"
run_hook "M6-06" "$VACIO_AVISA"
if grep -q 'reason=empty warn=aviso: arm vector INERTE' "$REFLEX_LOG_FILE" 2>/dev/null; then
  pass "H2: vacío con aviso largo sigue siendo empty y lleva el aviso"
else fail "H2: vacío con aviso largo sigue siendo empty y lleva el aviso" "$(cat "$REFLEX_LOG_FILE" 2>/dev/null)"; fi
```

En `plugins/exo/scripts/test-contrato-engine.sh`, justo antes de `if printf '%s' "$SALIDA" | jq -e '.schema_version == 2'`:

```bash
# H2/H3: recall-inject.sh lee .data.elapsed_s y .data.refresh_s (número o null)
# y .data.warnings (array o ausente). En modo arranque sin --refresh las dos
# claves de tiempo EXISTEN con null: se exige la clave, no solo el valor, para
# que un engine anterior a la campaña A dé rojo aquí.
if printf '%s' "$SALIDA" | jq -e '(.data | has("elapsed_s") and has("refresh_s"))
      and .data.elapsed_s == null and .data.refresh_s == null
      and ((.data.warnings // []) | type) == "array"' >/dev/null 2>&1; then
  pass "contrato: elapsed_s/refresh_s presentes (null en arranque) y warnings array o ausente"
else fail "contrato: elapsed_s/refresh_s presentes (null en arranque) y warnings array o ausente" \
  "$(printf '%s' "$SALIDA" | jq -c '.data | {elapsed_s, refresh_s, warnings}' 2>/dev/null)"; fi
```

- [ ] **Step 2: Verificar que falla**

Run: `bash plugins/exo/scripts/test-recall-inject.sh`
Expected: FAIL en `H2: el aviso del engine deja rastro engine-warning`,
`H3: emitted lleva elapsed_ms…` y `H2: vacío con aviso largo…` (este último
loguea `reason=error`). Los demás en PASS.

- [ ] **Step 3: Implementación mínima**

En `plugins/exo/scripts/recall-inject.sh`:

(a) Sustituir las líneas 149-150:
```bash
ERR=""
[ -n "$ERR_TMP" ] && ERR="$(head -c 300 "$ERR_TMP" 2>/dev/null)"; rm -f "$ERR_TMP"
```
por:
```bash
# Desde la campaña A (H2) el engine escribe `aviso: …` en stderr también
# cuando sale con 1. Se separan: el distinguidor de P2 («recall vacío» frente a
# engine roto) mira solo lo que NO es aviso, así un aviso largo ya no lo empuja
# fuera de los 300 bytes. Solo se parsea con rc≠0: el camino común no paga
# estos spawns.
ERR=""
AVISOS_ERR=""
if [ "$RC" -ne 0 ] && [ "$RC" -ne 124 ] && [ -n "$ERR_TMP" ]; then
  ERR="$(grep -v '^aviso: ' "$ERR_TMP" 2>/dev/null | head -c 300)"
  AVISOS_ERR="$(grep '^aviso: ' "$ERR_TMP" 2>/dev/null | tr '\n' ' ' | cut -c1-160)"
fi
[ -n "$ERR_TMP" ] && rm -f "$ERR_TMP"
```

(b) En la rama `if [ "$RC" -ne 0 ]; then`, sustituir el `case`:
```bash
  case "$ERR" in
    *"recall vacío"*) log_ri "degraded" "reason=empty${AVISOS_ERR:+ warn=$AVISOS_ERR}" ;;
    *) log_ri "degraded" "reason=error rc=$RC err=$(printf '%s' "$ERR" | tr -d '\n' | cut -c1-120)${AVISOS_ERR:+ warn=$AVISOS_ERR}" ;;
  esac
```

(c) Sustituir el bloque de truncado (líneas 186-192, desde el comentario `# Si el engine recortó su propia respuesta` hasta su `fi`) por:
```bash
# Metadatos del envelope en UNA pasada de jq (H2/H3): cada spawn cuesta decenas
# de ms en Git Bash. Si el engine recortó su propia respuesta (cap de fetch), el
# hit de repuesto puede haber desaparecido: no degrada nada, pero deja rastro.
# `@tsv` con los avisos AL FINAL, porque `read` colapsa un campo vacío en medio
# (el tab es whitespace de IFS).
META="$(printf '%s' "$SALIDA" | jq -r '[ (.data.truncated // false | tostring),
    ((.data.elapsed_s // 0) * 1000 | floor | tostring),
    ((.data.refresh_s // 0) * 1000 | floor | tostring),
    ((.data.warnings // []) | join(" | ")) ] | @tsv' 2>/dev/null)" || META=""
TRUNCADO=""; ELAPSED_MS=""; REFRESH_MS=""; AVISOS=""
[ -n "$META" ] && IFS=$'\t' read -r TRUNCADO ELAPSED_MS REFRESH_MS AVISOS <<< "$META"
if [ "$TRUNCADO" = "true" ]; then
  log_ri "degraded" "reason=fetch-truncado"
fi
if [ -n "$AVISOS" ]; then
  # Degradación de la búsqueda (arm vector INERTE/PARCIAL): el bloque se sirve
  # igual, pero ya no en silencio.
  log_ri "degraded" "reason=engine-warning w=$(printf '%s' "$AVISOS" | cut -c1-160)"
fi
```

(d) Sustituir la línea 339:
```bash
log_ri "emitted" "n_hits=$N bytes=$BYTES permalinks=$PERMALINKS"
```
por:
```bash
# Los tiempos van ANTES de permalinks: `_reflex-log.sh` corta el payload a
# 2000 chars y la lista de permalinks es lo único que puede crecer.
log_ri "emitted" "n_hits=$N bytes=$BYTES elapsed_ms=${ELAPSED_MS:-?} refresh_ms=${REFRESH_MS:-?} permalinks=$PERMALINKS"
```

- [ ] **Step 4: Verificar que pasa**

Run: `bash plugins/exo/scripts/test-recall-inject.sh`
Expected: `0 failed` en la última línea.

Run: O-plugin && O-contrato.
Expected: `test-plugin: OK — …`, exec-bit en verde y el contrato con todas
las líneas en `[PASS]`, incluida la nueva.

- [ ] **Step 5: Commit**

```bash
git add plugins/exo/scripts/recall-inject.sh plugins/exo/scripts/test-recall-inject.sh plugins/exo/scripts/test-contrato-engine.sh
git commit -m "feat(recall-inject): loguea avisos del engine y elapsed_ms/refresh_ms por prompt (campaña A, H2/H3)"
```

---

### Task 6: `resuelve_destinos` solo escribe lo que cambia, en una transacción

**Lane:** mecánica · **Hallazgo:** H4 · **Oráculo:** `cd engine && cargo test --release --test aristas_resolucion --test indexer` + O-engine

**Files:**
- Modify: `engine/src/aristas.rs:43-88` (`resuelve_destinos` y su doc)
- Test: `engine/tests/aristas_resolucion.rs` (nuevo)

**Interfaces:**
- Consumes: `exo::abre_db_en_memoria() -> Result<Connection>`, `exo::schema::crea_schema`.
- Produces: `pub fn resuelve_destinos(conn: &Connection) -> Result<()>`, con la misma firma. **Invariante nuevo:** una segunda pasada sin cambios hace `total_changes` = 0. Una arista desresuelta (NULL) por cualquier motivo se cura en la pasada siguiente, **aunque no se haya indexado nada**. Por eso NO se añade el salto «`indexadas == 0 && borradas == 0`».

- [ ] **Step 1: Escribir los tests que fallan**

`engine/tests/aristas_resolucion.rs`:

```rust
//! H4: `resuelve_destinos` corre en cada `--refresh` (≈86% de los prompts) y
//! hacía un UPDATE autocommit por arista aunque nada cambiara. Sin modelo: SQL puro.
use exo::aristas::resuelve_destinos;
use rusqlite::Connection;

fn db_con_aristas() -> Connection {
    let conn = exo::abre_db_en_memoria().unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    conn.execute_batch(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch) VALUES
           ('kb/a', 'a.md', 'A', NULL, 1.0, NULL),
           ('kb/b', 'b.md', 'B', NULL, 1.0, NULL);
         INSERT INTO aristas (origen, destino_texto) VALUES
           ('kb/a', 'B'), ('kb/b', 'kb/a'), ('kb/a', 'nadie'), ('kb/b', 'A|alias');",
    )
    .unwrap();
    conn
}

fn destino(conn: &Connection, origen: &str, texto: &str) -> Option<String> {
    conn.query_row(
        "SELECT destino_permalink FROM aristas WHERE origen = ?1 AND destino_texto = ?2",
        [origen, texto],
        |r| r.get(0),
    )
    .unwrap()
}

#[test]
fn resuelve_por_titulo_por_permalink_con_alias_y_deja_null_el_roto() {
    let conn = db_con_aristas();
    resuelve_destinos(&conn).unwrap();
    assert_eq!(destino(&conn, "kb/a", "B").as_deref(), Some("kb/b"));
    assert_eq!(destino(&conn, "kb/b", "kb/a").as_deref(), Some("kb/a"));
    assert_eq!(destino(&conn, "kb/b", "A|alias").as_deref(), Some("kb/a"));
    assert_eq!(destino(&conn, "kb/a", "nadie"), None);
}

#[test]
fn una_segunda_pasada_sin_cambios_no_escribe_ninguna_fila() {
    let conn = db_con_aristas();
    resuelve_destinos(&conn).unwrap();
    let antes = conn.total_changes();
    resuelve_destinos(&conn).unwrap();
    assert_eq!(
        conn.total_changes() - antes,
        0,
        "sin cambios en notas/aristas no hay nada que escribir"
    );
}

/// Por esto NO vale saltarse la pasada con `indexadas == 0 && borradas == 0`:
/// un abort entre los commits por nota y esta pasada dejaba aristas a NULL, y
/// la corrida siguiente, sin nada que indexar, no las curaría nunca.
#[test]
fn una_arista_desresuelta_se_cura_en_la_pasada_siguiente_escribiendo_solo_esa() {
    let conn = db_con_aristas();
    resuelve_destinos(&conn).unwrap();
    conn.execute(
        "UPDATE aristas SET destino_permalink = NULL WHERE destino_texto = 'B'",
        [],
    )
    .unwrap();
    let antes = conn.total_changes();
    resuelve_destinos(&conn).unwrap();
    assert_eq!(destino(&conn, "kb/a", "B").as_deref(), Some("kb/b"));
    assert_eq!(conn.total_changes() - antes, 1);
}

#[test]
fn una_nota_nueva_que_cura_un_link_roto_escribe_solo_esa_arista() {
    let conn = db_con_aristas();
    resuelve_destinos(&conn).unwrap();
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES ('kb/nadie', 'nadie.md', 'nadie', NULL, 1.0, NULL)",
        [],
    )
    .unwrap();
    let antes = conn.total_changes();
    resuelve_destinos(&conn).unwrap();
    assert_eq!(destino(&conn, "kb/a", "nadie").as_deref(), Some("kb/nadie"));
    assert_eq!(conn.total_changes() - antes, 1);
}
```

- [ ] **Step 2: Verificar que falla**

Run: `cd engine && cargo test --release --test aristas_resolucion`
Expected: FAIL. `una_segunda_pasada_sin_cambios_no_escribe_ninguna_fila`
da `left: 4, right: 0` y los dos de «solo esa» dan `left: 4, right: 1`.
`resuelve_por_titulo…` pasa: es caracterización.

- [ ] **Step 3: Implementación mínima**

En `engine/src/aristas.rs`, sustituir `resuelve_destinos` (desde su doc-comment, línea 43, hasta el cierre, línea 88) por:

```rust
/// Resuelve `destino_permalink` para TODAS las aristas de la DB (§diseño
/// punto 2): pase final sobre la tabla completa tras cada `index`/`rebuild`.
/// Para cada arista, la parte destino es el texto antes de `|` si hay alias;
/// se busca primero una nota cuyo `titulo` coincida EXACTO, si no una cuyo
/// `permalink` coincida EXACTO, si no queda NULL (§6.2 regla 6: un link a
/// nota inexistente se tolera, jamás error de indexado).
///
/// H4 (campaña A): corre en cada `exo recall --refresh`, es decir, en casi
/// cada prompt. Antes hacía un UPDATE autocommit por arista aunque el valor no
/// cambiara. Ahora solo escribe las aristas cuyo destino calculado difiere del
/// guardado, todas en UNA transacción, y si no hay ninguna, no abre
/// transacción. Se sigue recorriendo la tabla entera sin condición: saltarse el
/// pase cuando «no se indexó nada» dejaría sin curar para siempre las aristas
/// que un abort a mitad de corrida hubiera dejado a NULL.
pub fn resuelve_destinos(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("SELECT titulo, permalink FROM notas ORDER BY permalink")?;
    let filas: Vec<(String, String)> = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    drop(stmt);

    // ORDER BY permalink arriba hace determinista qué permalink gana si dos
    // notas comparten título exacto (última en orden alfabético de permalink).
    let por_titulo: HashMap<String, String> = filas.iter().cloned().collect();
    let por_permalink: HashSet<String> = filas.into_iter().map(|(_, p)| p).collect();

    let mut stmt =
        conn.prepare("SELECT rowid, destino_texto, destino_permalink FROM aristas")?;
    let aristas: Vec<(i64, String, Option<String>)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    drop(stmt);

    let pendientes: Vec<(i64, Option<&str>)> = aristas
        .iter()
        .filter_map(|(rowid, destino_texto, actual)| {
            let parte_destino = destino_texto
                .split_once('|')
                .map(|(antes, _)| antes)
                .unwrap_or(destino_texto);
            let resuelto: Option<&str> = por_titulo
                .get(parte_destino)
                .map(String::as_str)
                .or_else(|| por_permalink.get(parte_destino).map(String::as_str));
            (resuelto != actual.as_deref()).then_some((*rowid, resuelto))
        })
        .collect();

    if pendientes.is_empty() {
        return Ok(());
    }

    let tx = conn.unchecked_transaction()?;
    {
        let mut upd = tx.prepare("UPDATE aristas SET destino_permalink = ?1 WHERE rowid = ?2")?;
        for (rowid, resuelto) in &pendientes {
            upd.execute(params![resuelto, rowid]).with_context(|| {
                format!("resolver destino_permalink de la arista rowid={rowid}")
            })?;
        }
    }
    tx.commit()
        .context("commit de la resolución de destino_permalink")?;
    Ok(())
}
```

- [ ] **Step 4: Verificar que pasa**

Run: `cd engine && cargo test --release --test aristas_resolucion --test indexer && cargo test --release --lib aristas`
Expected: PASS, 4 tests en `aristas_resolucion`, y `indexer` sin cambios.

Run: O-engine. Expected: exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/aristas.rs engine/tests/aristas_resolucion.rs
git commit -m "perf(aristas): resuelve_destinos solo escribe lo que cambia, en una transacción (campaña A, H4)"
```

---

### Task 7: guarda «una DB = una KB»

**Lane:** mecánica · **Hallazgo:** H1 · **Bloqueada por:** D1 ∈ {A1, A2}. Con B o C, esta tarea se retira y hay que enmendar el plan.
**Oráculo:** `cd engine && cargo test --release --test indexer --test inicia` + O-engine + O-contrato

**Files:**
- Modify: `engine/src/indexer.rs` (nueva `pub fn comprueba_kb_root`; llamada en `indexa` justo después de `let kb_abs = std::fs::canonicalize(kb)…?;`, en `:119-120`)
- Modify: `engine/src/inicia.rs` (nueva `pub fn valida_db_para_kb`, junto a `valida_config_escribible`, en `:92-99`)
- Modify: `engine/src/main.rs:533-608` (`init_cmd`)
- Test: `engine/tests/indexer.rs` (añadir), `engine/tests/inicia.rs` (añadir)

**Interfaces:**
- Produces:
  - `pub fn exo::indexer::comprueba_kb_root(conn: &rusqlite::Connection, kb_abs: &Path) -> anyhow::Result<()>`: `Ok` si no hay `meta.kb_root`, si coincide con `kb_abs` o si la KB registrada ya no existe en disco (se ha movido). Si no, `Err` con un mensaje que contiene la ruta registrada, `otra KB`, `--db` y `exo rebuild`.
  - `pub fn exo::inicia::valida_db_para_kb(db: &Path, kb: &Path) -> anyhow::Result<()>`: `Ok` si `db` no existe. Si existe, aplica `comprueba_kb_root` con `kb` canonicalizada, o tal cual si todavía no existe.
  - La Task 8 consume ambas.

- [ ] **Step 1: Escribir los tests que fallan**

Añadir a `engine/tests/inicia.rs`:

```rust
/// H1: dos KBs de `exo init` con la MISMA plantilla sobre la MISMA DB. Antes:
/// la segunda moría con `UNIQUE constraint failed: notas.ruta` DESPUÉS de
/// volcar la plantilla y escribir su config, dejando residuo. Ahora falla
/// antes de tocar el disco, nombra la KB dueña y dice el remedio.
#[test]
fn dos_kbs_con_la_misma_plantilla_sobre_la_misma_db_la_segunda_falla_sin_residuo() {
    let tmp = tempfile::TempDir::new().unwrap();
    let db = tmp.path().join("compartida.db");
    let init = |kb: &std::path::Path, nombre: &str, config: &std::path::Path| {
        std::process::Command::new(env!("CARGO_BIN_EXE_exo"))
            .args(["init", "--kb"])
            .arg(kb)
            .args(["--name", nombre, "--json"])
            .env("EXO_CONFIG", config)
            .env("EXO_DB", &db)
            .output()
            .expect("ejecutar exo init")
    };
    let kb_a = tmp.path().join("kb-a");
    let kb_b = tmp.path().join("kb-b");
    let cfg_b = tmp.path().join("b.toml");

    let a = init(&kb_a, "kb-a", &tmp.path().join("a.toml"));
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));

    let b = init(&kb_b, "kb-b", &cfg_b);
    let err = String::from_utf8_lossy(&b.stderr);
    assert_eq!(b.status.code(), Some(1), "stderr: {err}");
    let dueña = std::fs::canonicalize(&kb_a).unwrap().display().to_string();
    assert!(err.contains(&dueña), "nombra la KB dueña ({dueña}): {err}");
    assert!(err.contains("--db"), "dice el remedio: {err}");
    assert!(!err.contains("UNIQUE constraint"), "ya no es el error críptico: {err}");
    assert!(!kb_b.exists(), "no deja la segunda KB a medio volcar");
    assert!(!cfg_b.exists(), "no escribe la config de la segunda");

    let conn = exo::abre_db(&db).unwrap();
    let n: i64 = conn
        .query_row("SELECT count(*) FROM notas WHERE permalink LIKE 'kb-a/%'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 11, "el índice de la primera KB sigue entero");
}
```

Añadir a `engine/tests/indexer.rs`, después de `indexa_dos_veces_no_duplica_kb_root`:

```rust
/// H1: medido el 2026-09-13, `exo index` de una segunda KB (sin rutas en común)
/// sobre la misma DB salía con exit 0 y `deleted: 11`: borraba en silencio el
/// índice de la primera. Notas sin cuerpo: sin trozos, sin modelo.
#[test]
fn indexar_otra_kb_existente_sobre_la_misma_db_falla_y_no_borra_la_primera() {
    let kb1 = tempfile::tempdir().unwrap();
    let kb2 = tempfile::tempdir().unwrap();
    std::fs::write(kb1.path().join("uno.md"), "---\ntitle: uno\npermalink: kb1/uno\n---\n").unwrap();
    std::fs::write(kb2.path().join("dos.md"), "---\ntitle: dos\npermalink: kb2/dos\n---\n").unwrap();
    let dbdir = tempfile::tempdir().unwrap();
    let db = dbdir.path().join("indice.db");

    common::con_config(kb1.path(), "kb-test", &db, || {
        exo::indexer::indexa(kb1.path(), &db).expect("primera KB");
        let err = exo::indexer::indexa(kb2.path(), &db).expect_err("la segunda KB debe rechazarse");
        let msg = format!("{err:#}");
        assert!(msg.contains("otra KB"), "{msg}");
        assert!(msg.contains("exo rebuild"), "{msg}");

        let conn = exo::abre_db(&db).unwrap();
        let permalinks: Vec<String> = conn
            .prepare("SELECT permalink FROM notas")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(permalinks, vec!["kb1/uno".to_string()]);
    });
}

/// Una KB MOVIDA (la ruta registrada ya no existe) no es otra KB: se reindexa
/// y `kb_root` pasa a la ruta nueva, como hasta ahora (`indexer.rs:115-116`).
#[test]
fn una_kb_movida_de_sitio_se_reindexa_sin_rechazo() {
    let raiz = tempfile::tempdir().unwrap();
    let vieja = raiz.path().join("vieja");
    let nueva = raiz.path().join("nueva");
    std::fs::create_dir_all(&vieja).unwrap();
    std::fs::write(vieja.join("uno.md"), "---\ntitle: uno\npermalink: kb/uno\n---\n").unwrap();
    let dbdir = tempfile::tempdir().unwrap();
    let db = dbdir.path().join("indice.db");

    common::con_config(&vieja, "kb-test", &db, || {
        exo::indexer::indexa(&vieja, &db).expect("antes de mover");
        std::fs::rename(&vieja, &nueva).unwrap();
        exo::indexer::indexa(&nueva, &db).expect("una KB movida no es otra KB");
        let conn = exo::abre_db(&db).unwrap();
        let valor: String = conn
            .query_row("SELECT valor FROM meta WHERE clave='kb_root'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(valor, std::fs::canonicalize(&nueva).unwrap().to_string_lossy());
    });
}
```

- [ ] **Step 2: Verificar que falla**

Run: `cd engine && cargo test --release --test indexer --test inicia -- kb`
Expected: FAIL. En `indexar_otra_kb_existente…`, el `expect_err` salta con
`la segunda KB debe rechazarse` (hoy devuelve Ok). En
`dos_kbs_con_la_misma_plantilla…` falla la aserción `nombra la KB dueña`
(stderr trae `UNIQUE constraint failed: notas.ruta`). `una_kb_movida…` pasa:
es caracterización.

- [ ] **Step 3: Implementación mínima**

`engine/src/indexer.rs`, añadir después de `fn verifica_modelo` (y antes de `fn ruta_relativa`):

```rust
/// H1 (campaña A): una DB sirve a UNA KB. `meta.kb_root` es de un solo valor
/// y el walk borra toda ruta que no ve, así que indexar otra KB sobre la
/// misma DB borraba en silencio el índice de la primera (exit 0, medido el
/// 2026-09-13), o reventaba con `UNIQUE constraint failed: notas.ruta` si las
/// dos compartían rutas (dos KBs de `exo init`).
///
/// Pasa si no hay `kb_root`, si coincide con `kb_abs` (ambas canónicas) o si
/// la KB registrada ya no existe en disco: eso es una KB movida, y seguir
/// actualizando `kb_root` es el contrato de siempre.
pub fn comprueba_kb_root(conn: &Connection, kb_abs: &Path) -> Result<()> {
    let previo: Option<String> = conn
        .query_row("SELECT valor FROM meta WHERE clave = 'kb_root'", [], |r| r.get(0))
        .optional()
        .context("leer meta.kb_root")?;
    let Some(previo) = previo else {
        return Ok(());
    };
    if previo == kb_abs.to_string_lossy() || !Path::new(&previo).is_dir() {
        return Ok(());
    }
    bail!(
        "este índice es de otra KB que sigue en disco: {previo} (pediste {}). \
         Una DB sirve a UNA KB: usa otra --db para esta, o `exo rebuild --kb {} --db <esta db>` \
         si de verdad quieres reemplazar el índice",
        kb_abs.display(),
        kb_abs.display()
    )
}
```

En `indexa`, justo después de:
```rust
    let kb_abs = std::fs::canonicalize(kb)
        .with_context(|| format!("canonicalizar raíz de KB {}", kb.display()))?;
```
insertar:
```rust
    comprueba_kb_root(&conn, &kb_abs)?;
```

`engine/src/inicia.rs`, añadir después de `valida_config_escribible`:

```rust
/// H1: llamada por `exo init` ANTES de tocar el disco, por la misma razón que
/// `valida_config_escribible` (I4): si la DB ya es de otra KB, el aborto tiene
/// que llegar antes de volcar la plantilla y escribir la config, no en el
/// indexado final. En modo creación la KB aún no existe y no se puede
/// canonicalizar; una ruta inexistente nunca es la KB registrada, que sí
/// existe, así que se compara tal cual.
pub fn valida_db_para_kb(db: &Path, kb: &Path) -> Result<()> {
    if !db.exists() {
        return Ok(());
    }
    let conn = crate::abre_db(db)?;
    crate::schema::crea_schema(&conn)?;
    let kb_abs = std::fs::canonicalize(kb).unwrap_or_else(|_| kb.to_path_buf());
    crate::indexer::comprueba_kb_root(&conn, &kb_abs)
}
```

`engine/src/main.rs`, en `init_cmd`:

(a) Justo después de `exo::inicia::valida_config_escribible(&destino, args.force)?;`:

```rust
    // H1: la DB que este `init` va a indexar, con la precedencia de
    // `resuelve_db` pero sin config (aún no existe): $EXO_DB > el default que
    // se graba en config.toml.
    let db_objetivo = match std::env::var("EXO_DB") {
        Ok(v) if !v.is_empty() => exo::config::expande_tilde(Path::new(&v)),
        _ => db_default.clone(),
    };
```

(b) En la rama de adopción, después del bloque `exo::inicia::valida_nombre(&nombre).with_context(…)?;` y antes de `(kb, nombre, emb, "adopt", Vec::new(), false)`:

```rust
        exo::inicia::valida_db_para_kb(&db_objetivo, &kb)?;
```

(c) En la rama de creación, sustituir:
```rust
        exo::inicia::valida_nombre(&nombre)?;
        exo::inicia::prepara_kb(&kb, args.force)?;
```
por:
```rust
        exo::inicia::valida_nombre(&nombre)?;
        exo::inicia::valida_db_para_kb(&db_objetivo, &kb)?;
        exo::inicia::prepara_kb(&kb, args.force)?;
```

- [ ] **Step 4: Verificar que pasa**

Run: `cd engine && cargo test --release --test indexer --test inicia`
Expected: PASS en todos. Si algún test existente de otra suite indexa dos
raíces distintas sobre la misma DB, **es H1 dentro del test**: separa las DBs
y anótalo en el commit. No relajes la guarda.

Run: O-engine && O-contrato. Expected: exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/indexer.rs engine/src/inicia.rs engine/src/main.rs engine/tests/indexer.rs engine/tests/inicia.rs
git commit -m "fix(indexer): una DB sirve a una KB — rechaza indexar otra KB viva sobre el mismo índice (campaña A, H1)"
```

---

### Task 8: `exo init --db` y hooks que leen la DB de la config

**Lane:** mecánica · **Hallazgo:** H1 · **Bloqueada por:** Task 7, Task 5 (ambas tocan `recall-inject.sh`) y **D1 = A2**. Con A1, esta tarea no se ejecuta.
**Oráculo:** `cd engine && cargo test --release --test inicia` + `bash plugins/exo/scripts/test-recall-inject.sh && bash plugins/exo/scripts/test-exo-index.sh && bash plugins/exo/scripts/test-exo-recall.sh` + O-plugin + O-contrato

**Files:**
- Modify: `engine/src/main.rs` (`struct ArgsInit`, `:93-113`; `init_cmd`: `db_objetivo` de la Task 7, `escribe_config` en `:600`, `let db = resuelve_db(None)?;` en `:607`)
- Modify: `plugins/exo/scripts/recall-inject.sh` (`:24`; guards `:112-122`; bloque de config `:202-226`)
- Modify: `plugins/exo/scripts/exo-recall.sh` (`:34`; bloque `:55-101`)
- Modify: `plugins/exo/scripts/exo-index.sh` (`:28`, `:53`, `:80`)
- Test: `engine/tests/inicia.rs`, `plugins/exo/scripts/test-recall-inject.sh`, `plugins/exo/scripts/test-exo-index.sh`, `plugins/exo/scripts/test-exo-recall.sh` (nuevo)

**Interfaces:**
- Consumes: `exo::inicia::valida_db_para_kb` (Task 7); `exo config --json` → `.data.index.db`, `.data.kb.name` (`main.rs:680-703`).
- Produces: `exo init --db <PATH>`, con precedencia flag > `$EXO_DB` > `~/.exo/index.db`. Sin `--db`, la config sigue grabando `~/.exo/index.db`, como hoy. En los hooks, `EXO_INDEX` pasa a ser un seam: sin él, `recall-inject.sh` y `exo-recall.sh` usan `.data.index.db`, y `exo-index.sh` no pasa `--db`, de modo que el engine resuelve flag > `$EXO_DB` > config.

- [ ] **Step 1: Escribir los tests que fallan**

Añadir a `engine/tests/inicia.rs`:

```rust
/// H1 (A2): con `--db` propia, una segunda KB de la misma plantilla convive con
/// la primera en la misma máquina. `--db` gana a `$EXO_DB` y queda grabada en
/// config.toml, que es de donde la leen los hooks.
#[test]
fn con_db_propia_una_segunda_kb_de_la_misma_plantilla_convive() {
    let tmp = tempfile::TempDir::new().unwrap();
    let db_a = tmp.path().join("a.db");
    let db_b = tmp.path().join("b.db");
    let init = |kb: &str, extra: &[&std::ffi::OsStr], config: &str| {
        std::process::Command::new(env!("CARGO_BIN_EXE_exo"))
            .args(["init", "--kb"])
            .arg(tmp.path().join(kb))
            .args(["--name", kb, "--json"])
            .args(extra)
            .env("EXO_CONFIG", tmp.path().join(config))
            .env("EXO_DB", &db_a)
            .output()
            .expect("ejecutar exo init")
    };
    let a = init("kb-a", &[], "a.toml");
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
    let b = init("kb-b", &["--db".as_ref(), db_b.as_os_str()], "b.toml");
    assert!(b.status.success(), "{}", String::from_utf8_lossy(&b.stderr));

    let cfg_b = exo::config::carga_desde(&tmp.path().join("b.toml")).unwrap();
    assert_eq!(cfg_b.index.db, db_b, "--db queda grabada en la config");
    for (db, prefijo) in [(&db_a, "kb-a/%"), (&db_b, "kb-b/%")] {
        let n: i64 = exo::abre_db(db)
            .unwrap()
            .query_row("SELECT count(*) FROM notas WHERE permalink LIKE ?1", [prefijo], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 11, "{prefijo} en {}", db.display());
    }
}
```

Añadir a `plugins/exo/scripts/test-recall-inject.sh`, antes del resumen final:

```bash
# --------------------------- H1 (A2): la DB sale de `exo config --json` ---
CFG_DB_EXO="$TMP/exo-config-db"
cat > "$CFG_DB_EXO" <<EOF
#!/usr/bin/env bash
if [ "\$1" = "config" ]; then
  printf '{"schema_version":2,"command":"config","data":{"kb":{"name":"kb-demo","path":"/kb"},"index":{"db":"%s"}}}\n' "$FAKE_DB"
  exit 0
fi
printf '%s\n' "\$*" >> "$EXO_CALLS"
exit 1
EOF
chmod +x "$CFG_DB_EXO"
: > "$EXO_CALLS"; : > "$REFLEX_LOG_FILE"
printf '%s' "M6-06" | jq -Rs '{prompt:., session_id:"test-sess"}' \
  | env -u EXO_INDEX EXO_BIN="$CFG_DB_EXO" EXO_KB_NAME="" "$HOOK" >/dev/null 2>&1
if contains "$(cat "$EXO_CALLS" 2>/dev/null)" "--db $FAKE_DB"; then pass "H1: sin EXO_INDEX usa la DB de la config"
else fail "H1: sin EXO_INDEX usa la DB de la config" "calls='$(cat "$EXO_CALLS")' log='$(cat "$REFLEX_LOG_FILE")'"; fi
```

Añadir a `plugins/exo/scripts/test-exo-index.sh`, antes de su resumen final:

```bash
# ---------------- H1 (A2): sin EXO_INDEX, el engine resuelve la DB él solo ---
: > "$INDEX_LOG"; : > "$SETSID_CALLS"
run_hook -u EXO_INDEX EXO_INDEX_SETSID="$FAKE_SETSID"
CALLS_SIN_DB="$(cat "$SETSID_CALLS" 2>/dev/null)"
if contains "$CALLS_SIN_DB" "index --json" && ! contains "$CALLS_SIN_DB" "--db"; then
  pass "H1: sin EXO_INDEX no se cablea --db (flag > \$EXO_DB > config en el engine)"
else fail "H1: sin EXO_INDEX no se cablea --db" "calls='$CALLS_SIN_DB'"; fi
```

`plugins/exo/scripts/test-exo-recall.sh` (nuevo, `chmod +x`):

```bash
#!/usr/bin/env bash
# Test standalone para exo-recall.sh (hook SessionStart). Fixtures en mktemp -d;
# nunca toca el HOME, el índice ni el log reales.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/exo-recall.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
export HOME="$TMP/home"
mkdir -p "$HOME/.claude"
export REFLEX_LOG_FILE="$HOME/.claude/reflex-log.jsonl"

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }
contains() { case "$1" in *"$2"*) return 0 ;; *) return 1 ;; esac; }

run_hook() {  # $1 = JSON de entrada; resto = argumentos de env (VAR=valor, -u VAR)
  local input="$1"; shift
  printf '%s' "$input" | env "$@" "$HOOK" > "$TMP/out.txt" 2>/dev/null
  HOOK_RC=$?
  HOOK_OUT="$(cat "$TMP/out.txt" 2>/dev/null)"
}

# ------------------------------ H1 (A2): la DB sale de `exo config --json` ---
FAKE_DB="$TMP/index.db"; : > "$FAKE_DB"
CALLS="$TMP/calls.txt"
FAKE_EXO="$TMP/exo-fake"
cat > "$FAKE_EXO" <<EOF
#!/usr/bin/env bash
if [ "\$1" = "config" ]; then
  printf '{"schema_version":2,"command":"config","data":{"kb":{"name":"kb-demo","path":"/kb"},"index":{"db":"%s"}}}\n' "$FAKE_DB"
  exit 0
fi
printf '%s\n' "\$*" >> "$CALLS"
printf '# Contrato de memoria\nbloque de prueba\n'
EOF
chmod +x "$FAKE_EXO"
: > "$CALLS"
run_hook '{"session_id":"test-sess","source":"startup"}' -u EXO_INDEX EXO_BIN="$FAKE_EXO"
if contains "$(cat "$CALLS")" "--db $FAKE_DB" && contains "$HOOK_OUT" "bloque de prueba"; then
  pass "H1: sin EXO_INDEX usa la DB de la config y sirve el bloque"
else fail "H1: sin EXO_INDEX usa la DB de la config" "calls='$(cat "$CALLS")' out='$HOOK_OUT'"; fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
```

- [ ] **Step 2: Verificar que falla**

Run: `cd engine && cargo test --release --test inicia -- con_db_propia; cd .. && bash plugins/exo/scripts/test-recall-inject.sh | grep -E 'H1|failed'; bash plugins/exo/scripts/test-exo-index.sh | grep -E 'H1|failed'; bash plugins/exo/scripts/test-exo-recall.sh`
Expected: el test de Rust falla con `error: unexpected argument '--db' found`
(exit 2 de clap). Los tres de shell dan `[FAIL] H1: …`: los hooks miran
`$HOME/.exo/index.db`, loguean `no-index` y no invocan `recall`, y
`exo-index.sh` sigue pasando `--db`.

- [ ] **Step 3: Implementación mínima**

`engine/src/main.rs`:

(a) En `struct ArgsInit`, antes de `#[arg(long)] json: bool,`:
```rust
    /// Fichero SQLite del índice, grabado en `[index] db`. Default: `$EXO_DB`
    /// si está puesta; si no, `~/.exo/index.db`. Una DB sirve a UNA KB: para
    /// una segunda KB en la misma máquina, dale su propia `--db`.
    #[arg(long)]
    db: Option<PathBuf>,
```

(b) Sustituir el `let db_objetivo = match std::env::var("EXO_DB") { … };` de la Task 7 por:
```rust
    // H1: la DB que este `init` va a indexar: --db > $EXO_DB > el default.
    let db_objetivo = match &args.db {
        Some(p) => exo::config::expande_tilde(p),
        None => match std::env::var("EXO_DB") {
            Ok(v) if !v.is_empty() => exo::config::expande_tilde(Path::new(&v)),
            _ => db_default.clone(),
        },
    };
    // Lo que se GRABA: la `--db` explícita, o el default de siempre. Sin flag,
    // `$EXO_DB` redirige el indexado de este proceso (tests, contrato-ci) pero
    // no se graba (comportamiento previo).
    let db_config = if args.db.is_some() {
        db_objetivo.clone()
    } else {
        db_default.clone()
    };
```

(c) Sustituir `exo::inicia::escribe_config(&destino, &kb, &nombre, &emb, &db_default, args.force)?;` por:
```rust
    exo::inicia::escribe_config(&destino, &kb, &nombre, &emb, &db_config, args.force)?;
```

(d) Sustituir `let db = resuelve_db(None)?;` por:
```rust
    let db = if args.db.is_some() {
        db_objetivo.clone()
    } else {
        resuelve_db(None)?
    };
```

`plugins/exo/scripts/recall-inject.sh`:

(e) Línea 24: `EXO_INDEX="${EXO_INDEX:-$HOME/.exo/index.db}"` → `EXO_INDEX="${EXO_INDEX:-}"`

(f) Sustituir el bloque `# --- Guards ---` (líneas 112-122) por:
```bash
# --- Guards ------------------------------------------------------------------
if [ ! -x "$EXO_BIN" ]; then
  log_ri "degraded" "reason=no-engine bin=$EXO_BIN"
  exit 0
fi

# Config del engine: índice (H1) y nombre de la KB (prefijo del exclude de
# core-index), en UNA llamada. Va DESPUÉS del guard del binario (sin binario
# fallaría por la misma causa y doblaría el evento con una razón que mentiría)
# y ANTES del guard del índice, porque ahora es la config la que dice qué
# índice mirar. Su posible fallo se loguea más abajo, pasado ese guard, por el
# mismo motivo.
CONFIG_ERR=""
CFG_DB=""
if [ -z "$EXO_INDEX" ] || [ -z "${EXO_KB_NAME:-}" ]; then
  CONFIG_ERR_TMP="$(mktemp)" || CONFIG_ERR_TMP=""
  CFG="$("$EXO_BIN" config --json 2>"${CONFIG_ERR_TMP:-/dev/null}" \
        | jq -r '(.data.index.db // "") + "\u0001" + (.data.kb.name // "")' 2>/dev/null)" || CFG=""
  CFG_DB="${CFG%%$'\001'*}"
  EXO_KB_NAME="${EXO_KB_NAME:-${CFG#*$'\001'}}"
  [ -n "$CONFIG_ERR_TMP" ] && CONFIG_ERR="$(head -1 "$CONFIG_ERR_TMP" 2>/dev/null | tr -d '\n' | cut -c1-120)"
  rm -f "$CONFIG_ERR_TMP"
fi
EXO_INDEX="${EXO_INDEX:-${CFG_DB:-$HOME/.exo/index.db}}"
if [ ! -f "$EXO_INDEX" ]; then
  # Sin índice NO se pasa `--refresh`: dispararía un bootstrap de minutos bajo
  # el timeout del evento. Se abstiene y deja rastro.
  log_ri "degraded" "reason=no-index db=$EXO_INDEX"
  exit 0
fi
```

(g) Sustituir el bloque de resolución de config (desde `# El nombre de la KB sale de la config del engine, no de un literal` hasta `rm -f "$CONFIG_ERR_TMP"`, líneas 206-225) por:
```bash
# El nombre de la KB ya se resolvió junto al índice (bloque de guards). Sin él
# no hay prefijo de proyecto y el exclude no calza con el permalink real:
# degradación aceptable, pero no muda, y con el motivo exacto de `exo config`.
if [ -z "${EXO_EXCLUIR:-}" ] && [ -z "$EXO_KB_NAME" ]; then
  log_ri "degraded" "reason=no-config err=$CONFIG_ERR"
fi
```
(las dos líneas siguientes, `EXO_EXCLUIR="${EXO_EXCLUIR:-…}"` y `EXO_MAX_HITS=…`, se quedan igual).

`plugins/exo/scripts/exo-recall.sh`:

(h) Línea 34: `EXO_INDEX="${EXO_INDEX:-$HOME/.exo/index.db}"` → `EXO_INDEX="${EXO_INDEX:-}"`

(i) Sustituir desde `BASE=""` (línea 55) hasta el `fi` que cierra el `if/elif/else` (línea 101) por:
```bash
BASE=""
if [ ! -x "$EXO_BIN" ]; then
  log_recall_fallback "no-engine" "bin=$EXO_BIN"
else
  # Índice (H1) y nombre de la KB en UNA llamada a la config, después del
  # guard del binario y antes del del índice (mismo razonamiento que
  # recall-inject.sh). El fallo de config se loguea pasado el guard del índice.
  CONFIG_ERR=""
  CFG_DB=""
  if [ -z "$EXO_INDEX" ] || [ -z "${EXO_KB_NAME:-}" ]; then
    CONFIG_ERR_TMP="$(mktemp)"
    CFG="$("$EXO_BIN" config --json 2>"$CONFIG_ERR_TMP" \
          | jq -r '(.data.index.db // "") + "\u0001" + (.data.kb.name // "")' 2>/dev/null)" || CFG=""
    CFG_DB="${CFG%%$'\001'*}"
    EXO_KB_NAME="${EXO_KB_NAME:-${CFG#*$'\001'}}"
    CONFIG_ERR="$(head -1 "$CONFIG_ERR_TMP" 2>/dev/null | tr -d '\n' | cut -c1-120)"
    rm -f "$CONFIG_ERR_TMP"
  fi
  EXO_INDEX="${EXO_INDEX:-${CFG_DB:-$HOME/.exo/index.db}}"
  if [ ! -f "$EXO_INDEX" ]; then
    log_recall_fallback "no-index" "db=$EXO_INDEX"
  else
    if [ -z "${EXO_RECALL_NOTA:-}" ] && [ -z "$EXO_KB_NAME" ]; then
      log_recall_fallback "no-config" "err=$CONFIG_ERR"
    fi
    EXO_NOTA="${EXO_RECALL_NOTA:-${EXO_KB_NAME:+$EXO_KB_NAME/}core/core-index}"

    # stderr se captura, no se tira: ahí avisa el engine de que el bloque no
    # cupo entero (F3.1).
    ERR_TMP="$(mktemp)"
    BASE="$("$EXO_BIN" recall --db "$EXO_INDEX" --content --note "$EXO_NOTA" \
            --limit "$EXO_LIMITE" --cap-bytes "$EXO_CAP" 2>"$ERR_TMP")" || BASE=""
    if grep -q 'truncado' "$ERR_TMP" 2>/dev/null; then
      log_recall_fallback "truncated" "$(head -1 "$ERR_TMP" | tr -d '\n' | cut -c1-120)"
    fi
    rm -f "$ERR_TMP"
    if [ -z "$BASE" ]; then
      log_recall_fallback "empty"
    elif ! printf '%s' "$BASE" | grep -q 'Contrato de memoria'; then
      # Un bloque sin el contrato de memoria no es el core-index: mejor el
      # fallback conocido que un bloque plausible pero falso.
      log_recall_fallback "no-contract"
      BASE=""
    fi
  fi
fi
```

`plugins/exo/scripts/exo-index.sh`:

(j) Línea 28: `EXO_INDEX="${EXO_INDEX:-$HOME/.exo/index.db}"` → `EXO_INDEX="${EXO_INDEX:-}"`

(k) Línea 53, la invocación con setsid, pasa a ser:
```bash
  # H1: sin EXO_INDEX explícito no se cablea `--db`: el engine resuelve flag >
  # $EXO_DB > `[index] db`, igual que la KB. Pasar `~/.exo/index.db` a pelo
  # indexaba la KB de la config sobre el índice de OTRA KB en cuanto
  # `exo init --db` grababa una DB distinta.
  "$SETSID_BIN" -f nohup "$EXO_BIN" index ${EXO_INDEX:+--db "$EXO_INDEX"} --json >>"$LOG" 2>&1 </dev/null || true
```

(l) Línea 80, el `bash -c` de la rama cmd:
```bash
    'exec "$EXO_BIN" index ${EXO_INDEX:+--db "$EXO_INDEX"} --json >>"$LOG" 2>&1 </dev/null' \
```

- [ ] **Step 4: Verificar que pasa**

Run: `cd engine && cargo test --release --test inicia && cd .. && bash plugins/exo/scripts/test-recall-inject.sh && bash plugins/exo/scripts/test-exo-index.sh && bash plugins/exo/scripts/test-exo-recall.sh`
Expected: PASS y `0 failed` en los tres scripts. Los tests previos de
`no-config`, `no-index` y `no-engine` siguen en verde.

Run: `chmod +x plugins/exo/scripts/test-exo-recall.sh && O-plugin && O-contrato`. Expected: exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/main.rs engine/tests/inicia.rs plugins/exo/scripts/recall-inject.sh plugins/exo/scripts/exo-recall.sh plugins/exo/scripts/exo-index.sh plugins/exo/scripts/test-recall-inject.sh plugins/exo/scripts/test-exo-index.sh plugins/exo/scripts/test-exo-recall.sh
git commit -m "feat(init): --db propia y hooks que leen el índice de la config (campaña A, H1-A2)"
```

---

### Task 9: un solo `separa_frontmatter`

**Lane:** mecánica · **Hallazgo:** H19 · **Oráculo:** `cd engine && cargo test --release --lib escritor && cargo test --release --test escritor --test nota` + O-engine

**Excepción TDD declarada:** es un refactor sin cambio de comportamiento. El
«rojo» sale de dos sitios: (1) un test que exige que el write-path use el
corte del indexer, y que hoy no compila porque `nota::separa_frontmatter` es
privada; (2) una mutación deliberada que demuestra que la caracterización
muerde.

**Files:**
- Modify: `engine/src/nota.rs:74` (visibilidad)
- Modify: `engine/src/escritor.rs:422-445` (`separa_frontmatter` y su doc)
- Test: `engine/src/escritor.rs` (nuevo `#[cfg(test)] mod tests_separa` al final del fichero)

**Interfaces:**
- Produces: `pub(crate) fn crate::nota::separa_frontmatter(contenido: &str) -> Option<(String, String)>`, con el mismo cuerpo. `escritor::separa_frontmatter(&str) -> (String, String)` conserva su firma privada y su comportamiento.
- No tocar `engine/src/frontmatter.rs`: su divergencia es deliberada, para compatibilidad con kbx.

- [ ] **Step 1: Escribir los tests**

Al final de `engine/src/escritor.rs`:

```rust
#[cfg(test)]
mod tests_separa {
    use super::separa_frontmatter;

    /// Caracterización del write-path ANTES del dedupe (H19): estos casos
    /// pasan con la copia vieja y tienen que seguir pasando con el envoltorio.
    #[test]
    fn separa_conserva_el_salto_final_del_cuerpo_y_normaliza_crlf() {
        assert_eq!(
            separa_frontmatter("---\ntitle: X\n---\ncuerpo\n"),
            ("title: X".to_string(), "cuerpo\n".to_string())
        );
        assert_eq!(
            separa_frontmatter("---\r\ntitle: X\r\n---\r\ncuerpo\r\n"),
            ("title: X".to_string(), "cuerpo\n".to_string())
        );
        assert_eq!(
            separa_frontmatter("---\n---\ncuerpo"),
            (String::new(), "cuerpo".to_string())
        );
        assert_eq!(
            separa_frontmatter("---\ntitle: X\n---\n"),
            ("title: X".to_string(), String::new())
        );
    }

    #[test]
    fn sin_frontmatter_o_sin_cierre_todo_es_cuerpo() {
        assert_eq!(
            separa_frontmatter("sin frontmatter\n"),
            (String::new(), "sin frontmatter\n".to_string())
        );
        assert_eq!(
            separa_frontmatter("---\ntitle: X\nsin cierre\n"),
            (String::new(), "---\ntitle: X\nsin cierre\n".to_string())
        );
    }

    /// El write-path y el indexer cortan IGUAL el YAML: si divergieran, una nota
    /// escrita por `exo write` podría indexarse con otro frontmatter.
    #[test]
    fn el_write_path_usa_el_corte_del_indexer() {
        for texto in ["---\ntitle: X\n---\ncuerpo\n", "---\r\na: 1\r\n---\r\nb\r\n"] {
            let (yaml, _) = separa_frontmatter(texto);
            let (yaml_indexer, _) = crate::nota::separa_frontmatter(texto).unwrap();
            assert_eq!(yaml, yaml_indexer);
        }
    }
}
```

- [ ] **Step 2: Verificar el rojo**

Run: `cd engine && cargo test --release --lib escritor`
Expected: FAIL de compilación, `function `separa_frontmatter` is private`
(E0603), en `el_write_path_usa_el_corte_del_indexer`.

- [ ] **Step 3: Implementación mínima**

`engine/src/nota.rs:74`: `fn separa_frontmatter(contenido: &str) -> Option<(String, String)> {` →
`pub(crate) fn separa_frontmatter(contenido: &str) -> Option<(String, String)> {`

`engine/src/escritor.rs`, sustituir el doc-comment y la función `separa_frontmatter` (desde `/// Divide en (yaml, cuerpo).` hasta su cierre) por:

```rust
/// Divide en (yaml, cuerpo). Sin frontmatter delimitado ⇒ yaml vacío y el
/// texto entero como cuerpo. El corte lo hace `nota::separa_frontmatter`, el
/// mismo que usa el indexer (H19: antes aquí vivía una copia del algoritmo).
/// Este envoltorio solo añade lo que el write-path necesita y el indexer no:
/// conservar el `\n` final del cuerpo.
fn separa_frontmatter(contenido: &str) -> (String, String) {
    match crate::nota::separa_frontmatter(contenido) {
        Some((yaml, mut cuerpo)) => {
            if contenido.ends_with('\n') && !cuerpo.is_empty() {
                cuerpo.push('\n');
            }
            (yaml, cuerpo)
        }
        None => (String::new(), contenido.to_string()),
    }
}
```

- [ ] **Step 4: Verificar que pasa y que la caracterización muerde**

Run: `cd engine && cargo test --release --lib escritor && cargo test --release --test escritor --test nota`
Expected: PASS, 3 en `tests_separa` y las suites `escritor` y `nota` sin cambios.

Mutación: comenta la línea `cuerpo.push('\n');` y corre `cargo test --release --lib escritor`.
Expected: FAIL en `separa_conserva_el_salto_final_del_cuerpo_y_normaliza_crlf`.
**Restaura la línea** y vuelve a correr hasta verde.

Run: O-engine. Expected: exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/nota.rs engine/src/escritor.rs
git commit -m "refactor(escritor): separa_frontmatter delega en el corte del indexer (campaña A, H19)"
```

---

### Task 10: recientes perezosas en `recall --content --note`

**Lane:** mecánica · **Hallazgo:** H17a · **Bloqueada por:** D2 = L **y** la puerta C-H17a ABIERTA en `evals/recall-coste/results/baseline/predicciones.txt`. Si la puerta está CERRADA, la tarea no se ejecuta y se anota en el ledger. Si D2 = P, se enmienda el plan.
**Oráculo:** `cd engine && cargo test --release --test recall_contenido --test recall` + O-engine

**Excepción TDD declarada:** es un cambio solo de rendimiento, con la misma
salida. Primero se añade caracterización del camino `--note`, que hoy no tiene
ningún test (todas las llamadas de `tests/recall_contenido.rs` pasan `None`).
El «pasa» de rendimiento lo da C-H17a en la Task 14.

**Files:**
- Modify: `engine/src/recall.rs:308-388` (`recall_arranque_contenido`) y una función nueva `recientes_no_core`
- Test: `engine/tests/recall_contenido.rs` (añadir)

**Interfaces:**
- Consumes: `fila_notas(&Connection, &str)`, `tier_de(&Path)`, `relativa`, `etiqueta` y `cuerpo_de`, todas en `recall.rs`; `recall_arranque` sin cambios.
- Produces: `fn recientes_no_core(conn: &rusqlite::Connection, kb: &Path, limite: usize) -> Result<Vec<NotaRecall>>`. `recall_arranque_contenido` mantiene su firma pública y su salida.

- [ ] **Step 1: Escribir la caracterización**

Añadir a `engine/tests/recall_contenido.rs`:

```rust
/// H17a: el camino del hook de SessionStart (`--note`). Recientes = no-core,
/// por git_epoch descendente y cortadas a `limite`; con `--note` solo va el
/// cuerpo de la nota pedida. Caracteriza ANTES de que la Task 10 deje de
/// parsear el frontmatter de toda la KB.
#[test]
fn con_note_las_recientes_excluyen_core_respetan_limite_y_van_por_git() {
    let kb = TempDir::new().unwrap();
    git(kb.path(), &["init", "-q"]);
    for (nombre, tier, epoch) in [
        ("indice.md", "core", 1_700_000_000i64),
        ("vieja.md", "log", 1_700_000_001),
        ("media.md", "stable", 1_700_000_002),
        ("nueva.md", "log", 1_700_000_003),
        ("core-reciente.md", "core", 1_700_000_004),
    ] {
        escribe(kb.path(), nombre, tier, &format!("cuerpo de {nombre}"));
        git(kb.path(), &["add", nombre]);
        let fecha = format!("{epoch} +0000");
        Command::new("git")
            .args(["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-qm", nombre])
            .env("GIT_AUTHOR_DATE", &fecha)
            .env("GIT_COMMITTER_DATE", &fecha)
            .current_dir(kb.path())
            .output()
            .unwrap();
    }
    let (_d, db) = db_temporal();

    common::con_config(kb.path(), "kb-test", &db, || {
        indexa(kb.path(), &db).unwrap();
        let bloque =
            recall_arranque_contenido(&db, kb.path(), 2, 8192, Some("kb/indice")).unwrap();
        let recientes: Vec<&str> = bloque
            .lines()
            .skip_while(|l| !l.starts_with("--- Actividad reciente"))
            .skip(1)
            .collect();
        assert_eq!(recientes, vec!["nueva.md", "media.md"], "bloque:\n{bloque}");
        assert!(bloque.contains("cuerpo de indice.md"), "{bloque}");
        assert!(!bloque.contains("cuerpo de core-reciente.md"), "{bloque}");
    });
}

#[test]
fn con_note_de_un_permalink_inexistente_sigue_fallando() {
    let kb = kb_de_prueba();
    let (_d, db) = db_temporal();
    common::con_config(kb.path(), "kb-test", &db, || {
        indexa(kb.path(), &db).unwrap();
        let err = recall_arranque_contenido(&db, kb.path(), 5, 8192, Some("kb/no-existe"))
            .expect_err("permalink inexistente");
        assert!(format!("{err:#}").contains("no está en el índice"), "{err:#}");
    });
}
```

- [ ] **Step 2: Verificar que la caracterización pasa con el código actual**

Run: `cd engine && cargo test --release --test recall_contenido -- con_note`
Expected: PASS (2). Si falla, el test está mal escrito respecto al
comportamiento vigente: corrígelo **a él**, nunca a `recall.rs`, y anota el
motivo en el commit.

- [ ] **Step 3: Implementación**

En `engine/src/recall.rs`, añadir antes de `pub fn recall_arranque_contenido`:

```rust
/// Las `limite` notas más recientes que NO son `tier: core`, en el mismo
/// orden que `recall_arranque` (git_epoch descendente, sin epoch al final,
/// empate por ruta en orden de bytes: la collation BINARY de SQLite, igual que
/// `String::cmp`), leyendo el `tier` de disco SOLO de las candidatas que
/// recorre (H17a). `recall_arranque` parsea el frontmatter de TODAS las notas
/// para separar los core. El camino `--note` del hook de SessionStart no los
/// necesita, y con la KB creciendo eran N lecturas por arranque.
fn recientes_no_core(
    conn: &rusqlite::Connection,
    kb: &Path,
    limite: usize,
) -> Result<Vec<NotaRecall>> {
    let mut stmt = conn.prepare(
        "SELECT permalink, ruta, titulo FROM notas
         ORDER BY git_epoch IS NULL, git_epoch DESC, ruta",
    )?;
    let filas = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    })?;
    let mut recientes = Vec::with_capacity(limite);
    for fila in filas {
        if recientes.len() >= limite {
            break;
        }
        let (permalink, ruta_rel, titulo) = fila.context("leer notas para recientes")?;
        let ruta_abs = kb.join(&ruta_rel);
        if tier_de(&ruta_abs).as_deref() == Some("core") {
            continue;
        }
        recientes.push(NotaRecall {
            permalink,
            ruta: ruta_abs.display().to_string(),
            titulo,
            tier: None,
            score: None,
            snippet: None,
        });
    }
    Ok(recientes)
}
```

En `recall_arranque_contenido`, sustituir desde `let bruto = recall_arranque(db_ruta, kb, limite)?;`
hasta el final de `let elegidas: Vec<&NotaRecall> = match &pedida { … };` (todo ese tramo) por:

```rust
    if !db_ruta.exists() {
        anyhow::bail!("DB no encontrada: {}", db_ruta.display());
    }

    // H17a: con `--note` solo hacen falta la nota pedida y las recientes
    // no-core, así que el `tier` se lee de disco solo de las candidatas
    // (`recientes_no_core`). Sin `--note` hay que encontrar TODAS las
    // `tier: core`, y ese camino sigue siendo O(N) vía `recall_arranque`.
    //
    // La nota pedida se busca en TODO el índice, no solo entre las
    // seleccionadas: si no, solo se podrían pedir cores o recientes.
    let (elegidas, recientes): (Vec<NotaRecall>, Vec<NotaRecall>) = match nota {
        Some(permalink) => {
            let conn = abre_db(db_ruta)?;
            let Some((ruta_rel, titulo)) = fila_notas(&conn, permalink)? else {
                anyhow::bail!(
                    "la nota pedida no está en el índice: {permalink} \
                     (¿permalink mal escrito, o índice sin refrescar?)"
                );
            };
            let pedida = NotaRecall {
                permalink: permalink.to_string(),
                ruta: kb.join(ruta_rel).display().to_string(),
                titulo,
                tier: None,
                score: None,
                snippet: None,
            };
            (vec![pedida], recientes_no_core(&conn, kb, limite)?)
        }
        None => recall_arranque(db_ruta, kb, limite)?
            .notas
            .into_iter()
            .partition(|n| n.tier.as_deref() == Some("core")),
    };
```

Luego, en el resto de la función:
- `for nota in elegidas {` → `for nota in &elegidas {`
- borrar el bloque
  ```rust
      let recientes: Vec<&NotaRecall> = bruto
          .notas
          .iter()
          .filter(|n| n.tier.as_deref() != Some("core"))
          .collect();
  ```
- `for nota in recientes {` → `for nota in &recientes {`

- [ ] **Step 4: Verificar que pasa**

Run: `cd engine && cargo test --release --test recall_contenido --test recall && cargo test --release --lib recall`
Expected: PASS, con los dos `con_note_*` y todas las pruebas previas en verde.

Run: O-engine. Expected: exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/recall.rs engine/tests/recall_contenido.rs
git commit -m "perf(recall): --content --note lee el tier solo de las recientes candidatas (campaña A, H17a)"
```

---

### Task 11: log de reflejos acotado. Se ejecuta **una** variante según D3

**Lane:** mecánica · **Hallazgo:** H5 · **Bloqueada por:** D3. Con «nada», no se ejecuta.
**Oráculo:** `bash plugins/exo/scripts/test-exo-recall.sh` (variante R: además `bash plugins/exo/scripts/test-reflex-baseline.sh && bash plugins/exo/scripts/test-a1-gate.sh`) + O-plugin

**Interfaces:**
- Consumes: el formato de `_reflex-log.sh` (una línea JSON por evento: `ts`, `reflex`, `session_id`, `agent_id`, `agent_type`, `tool`, `payload`).
- Produces (T): `exo-recall.sh` mira solo las últimas `${EXO_RECALL_COMPACT_LINEAS:-2000}` líneas.
  Produces (R): en SessionStart, rotación a `$REFLEX_LOG_FILE.1` por encima de `${REFLEX_LOG_MAX_BYTES:-5000000}` bytes. `a1-gate.sh`, `reflex-baseline.sh` y `reflex-fp-review.sh` leen `.1` y el actual.
  Los valores 2000 y 5000000 son la propuesta de D3: si Paul fija otros, se sustituyen esos literales antes del Step 1.

**Esqueleto de `test-exo-recall.sh`:** si la Task 8 no se ejecutó (D1 = A1),
este fichero no existe todavía. Créalo con este contenido (`chmod +x`) y
añade los casos de la variante justo antes de la línea `printf '\n%d passed,
%d failed\n'`:

```bash
#!/usr/bin/env bash
# Test standalone para exo-recall.sh (hook SessionStart). Fixtures en mktemp -d;
# nunca toca el HOME, el índice ni el log reales.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/exo-recall.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
export HOME="$TMP/home"
mkdir -p "$HOME/.claude"
export REFLEX_LOG_FILE="$HOME/.claude/reflex-log.jsonl"

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }
contains() { case "$1" in *"$2"*) return 0 ;; *) return 1 ;; esac; }

run_hook() {  # $1 = JSON de entrada; resto = argumentos de env (VAR=valor, -u VAR)
  local input="$1"; shift
  printf '%s' "$input" | env "$@" "$HOOK" > "$TMP/out.txt" 2>/dev/null
  HOOK_RC=$?
  HOOK_OUT="$(cat "$TMP/out.txt" 2>/dev/null)"
}

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
```

#### Variante T (tail)

**Files:** Modify `plugins/exo/scripts/exo-recall.sh:109-112` · Test `plugins/exo/scripts/test-exo-recall.sh`

- [ ] **Step T1: Escribir el test que falla**

Casos a añadir a `test-exo-recall.sh`:

```bash
# ------------------- H5 (T): la reafirmación tras compactar mira solo la cola ---
# 2.500 líneas: un git-c de sess-x FUERA de la ventana de 2.000 y un
# verify-before-commit de sess-x DENTRO. Solo el segundo tiene que reforzarse.
LOGC="$HOME/.claude/reflex-log.jsonl"
{
  printf '{"ts":"t","reflex":"git-c","session_id":"sess-x","agent_id":"","agent_type":"","tool":"Bash","payload":"viejo"}\n'
  awk 'BEGIN { for (i = 0; i < 2498; i++) printf "{\"ts\":\"t\",\"reflex\":\"zero-residuo\",\"session_id\":\"otra-%d\",\"agent_id\":\"\",\"agent_type\":\"\",\"tool\":\"Bash\",\"payload\":\"x\"}\n", i }'
  printf '{"ts":"t","reflex":"verify-before-commit","session_id":"sess-x","agent_id":"","agent_type":"","tool":"Bash","payload":"nuevo"}\n'
} > "$LOGC"
run_hook '{"session_id":"sess-x","source":"compact"}' EXO_BIN="$TMP/no-existe"
CTX="$(printf '%s' "$HOOK_OUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"
if contains "$CTX" "verifica (corre el cambio)"; then pass "H5: refuerza lo disparado dentro de la ventana"
else fail "H5: refuerza lo disparado dentro de la ventana" "ctx='$CTX'"; fi
if ! contains "$CTX" "git -C X"; then pass "H5: no escanea más allá de la ventana de 2000 líneas"
else fail "H5: no escanea más allá de la ventana de 2000 líneas" "ctx='$CTX'"; fi
```

- [ ] **Step T2: Verificar que falla**

Run: `bash plugins/exo/scripts/test-exo-recall.sh`
Expected: `[FAIL] H5: no escanea más allá de la ventana de 2000 líneas`: hoy
lee el fichero entero y refuerza también `git -C`.

- [ ] **Step T3: Implementación mínima**

En `plugins/exo/scripts/exo-recall.sh`, sustituir:
```bash
  FIRED="$(jq -r --arg sid "$SID" 'select(.session_id==$sid) | .reflex' \
            "$HOME/.claude/reflex-log.jsonl" 2>/dev/null | sort -u)"
```
por:
```bash
  # H5: el log crece sin cota (~11 KB/día medidos el 2026-09-13) y esto corre
  # en cada compactación. Los disparos de ESTA sesión están en la cola: 2.000
  # líneas son ~74 días al ritmo actual. No se rota nada, porque el análisis
  # (a1-gate, reflex-baseline, reflex-fp-review) necesita la historia entera.
  FIRED="$(tail -n "${EXO_RECALL_COMPACT_LINEAS:-2000}" "$HOME/.claude/reflex-log.jsonl" 2>/dev/null \
            | jq -r --arg sid "$SID" 'select(.session_id==$sid) | .reflex' 2>/dev/null | sort -u)"
```

- [ ] **Step T4: Verificar que pasa**

Run: `bash plugins/exo/scripts/test-exo-recall.sh && ./scripts/test-plugin.sh && bash scripts/test-exec-bit.sh`
Expected: `0 failed` y `test-plugin: OK`.

- [ ] **Step T5: Commit**

```bash
git add plugins/exo/scripts/exo-recall.sh plugins/exo/scripts/test-exo-recall.sh
git commit -m "perf(exo-recall): la reafirmación tras compactar lee solo la cola del log (campaña A, H5-T)"
```

#### Variante R (rotación)

**Files:** Modify `plugins/exo/scripts/exo-recall.sh` (antes del bloque `# --- Reafirmación de reflejos`, `:105`), `plugins/exo/scripts/reflex-baseline.sh:7`, `plugins/exo/scripts/a1-gate.sh:17` y `:46`, `plugins/exo/scripts/reflex-fp-review.sh:8` · Test `plugins/exo/scripts/test-exo-recall.sh`, `plugins/exo/scripts/test-reflex-baseline.sh`, `plugins/exo/scripts/test-a1-gate.sh`

- [ ] **Step R1: Escribir los tests que fallan**

Casos a añadir a `test-exo-recall.sh`:

```bash
# ------------------------- H5 (R): rotación a .1 en SessionStart por tamaño ---
LOGR="$HOME/.claude/reflex-log.jsonl"
rm -f "$LOGR" "$LOGR.1"
awk 'BEGIN { for (i = 0; i < 50; i++) printf "{\"ts\":\"t\",\"reflex\":\"git-c\",\"session_id\":\"s\",\"agent_id\":\"\",\"agent_type\":\"\",\"tool\":\"\",\"payload\":\"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\"}\n" }' > "$LOGR"
run_hook '{"session_id":"s2","source":"startup"}' EXO_BIN="$TMP/no-existe" REFLEX_LOG_MAX_BYTES=1000
if [ -f "$LOGR.1" ] && [ "$(wc -l < "$LOGR.1" | tr -d ' ')" -eq 50 ]; then pass "H5-R: por encima del tope rota a .1 entero"
else fail "H5-R: por encima del tope rota a .1 entero" "$(ls -la "$HOME/.claude")"; fi
if [ ! -f "$LOGR" ] || [ "$(wc -l < "$LOGR" | tr -d ' ')" -le 2 ]; then pass "H5-R: el log vivo arranca de nuevo"
else fail "H5-R: el log vivo arranca de nuevo" "$(wc -l < "$LOGR")"; fi
rm -f "$LOGR.1"
printf '{"ts":"t","reflex":"git-c","session_id":"s","agent_id":"","agent_type":"","tool":"","payload":"p"}\n' > "$LOGR"
run_hook '{"session_id":"s2","source":"startup"}' EXO_BIN="$TMP/no-existe" REFLEX_LOG_MAX_BYTES=1000
if [ ! -f "$LOGR.1" ]; then pass "H5-R: por debajo del tope no rota"
else fail "H5-R: por debajo del tope no rota" "rotó"; fi
```

En `test-reflex-baseline.sh`, antes de su resumen final, un caso de equivalencia: el log partido en `.1` y el actual da la misma salida que el entero.

```bash
# H5-R: con el log rotado, el baseline lee las dos generaciones.
SPLIT="$(mktemp -d)"
head -n 3 "$TMPLOG" > "$SPLIT/log.jsonl.1"
tail -n +4 "$TMPLOG" > "$SPLIT/log.jsonl"
OUT_SPLIT="$(REFLEX_LOG_FILE="$SPLIT/log.jsonl" bash "$SCRIPT" 2>&1)"
if [ "$OUT_SPLIT" = "$OUT" ]; then printf '[PASS] %s\n' "H5-R: .1 + actual == log entero"; PASS=$((PASS+1))
else printf '[FAIL] %s — %s\n' "H5-R: .1 + actual == log entero" "$OUT_SPLIT"; FAIL=$((FAIL+1)); fi
rm -rf "$SPLIT"
```

En `test-a1-gate.sh`, antes de `echo ""` / `TOTAL=$((PASS+FAIL))`:

```bash
# H5-R: el gate sobre el log rotado (.1 + actual) da lo mismo que sobre el entero.
LOG1R="$TMP/log1r.jsonl"
head -n 8 "$LOG1" > "$LOG1R.1"
tail -n +9 "$LOG1" > "$LOG1R"
OUT1R="$(REFLEX_LOG_FILE="$LOG1R" REFLEX_PROJECTS_DIR="$PROJ1" bash "$GATE" 2026-08-01 2026-08-03 2>&1)"
if [ "$OUT1R" = "$OUT1" ]; then pass "H5-R: a1-gate sobre .1 + actual == log entero"
else fail "H5-R: a1-gate sobre .1 + actual == log entero" "$OUT1R"; fi
```

- [ ] **Step R2: Verificar que falla**

Run: `bash plugins/exo/scripts/test-exo-recall.sh; bash plugins/exo/scripts/test-reflex-baseline.sh; bash plugins/exo/scripts/test-a1-gate.sh`
Expected: `[FAIL] H5-R: por encima del tope rota a .1 entero`, y `[FAIL]`
en los dos casos de equivalencia, porque hoy solo se lee el fichero actual.

- [ ] **Step R3: Implementación**

`exo-recall.sh`, insertar justo antes de `# --- Reafirmación de reflejos disparados si SessionStart(source=compact) ---`:
```bash
# H5 (R): rotación por tamaño, UNA generación, una vez por sesión (aquí y no en
# `_reflex-log.sh`, para no pagar un `wc` por evento). Best-effort: en Git Bash
# `mv` falla si otra sesión tiene el fichero abierto, y entonces rota en la
# siguiente. Carrera aceptada: dos arranques simultáneos pueden rotar dos veces
# y perder el `.1` anterior.
REFLEX_LOG="${REFLEX_LOG_FILE:-$HOME/.claude/reflex-log.jsonl}"
if [ -f "$REFLEX_LOG" ]; then
  BYTES_LOG="$(wc -c < "$REFLEX_LOG" 2>/dev/null | tr -d ' ')" || BYTES_LOG=0
  if [ "${BYTES_LOG:-0}" -gt "${REFLEX_LOG_MAX_BYTES:-5000000}" ] 2>/dev/null; then
    mv -f "$REFLEX_LOG" "$REFLEX_LOG.1" 2>/dev/null || true
  fi
fi
```

`reflex-baseline.sh`, `a1-gate.sh` y `reflex-fp-review.sh`: justo después de la línea `LOG="${REFLEX_LOG_FILE:-$HOME/.claude/reflex-log.jsonl}"` de cada uno, insertar:
```bash
# H5 (R): el log rota a `.1` en SessionStart; el análisis mira las dos generaciones.
LOG_TODO=""
if [ -f "$LOG.1" ]; then
  LOG_TODO="$(mktemp)"
  cat "$LOG.1" "$LOG" > "$LOG_TODO" 2>/dev/null
  LOG="$LOG_TODO"
fi
```
En `reflex-baseline.sh` y `reflex-fp-review.sh`, añadir después de ese bloque:
```bash
trap 'rm -f "$LOG_TODO"' EXIT
```
En `a1-gate.sh`, sustituir `trap 'rm -f "$FILTERED"' EXIT` por:
```bash
trap 'rm -f "$FILTERED" ${LOG_TODO:+"$LOG_TODO"}' EXIT
```

- [ ] **Step R4: Verificar que pasa**

Run: `bash plugins/exo/scripts/test-exo-recall.sh && bash plugins/exo/scripts/test-reflex-baseline.sh && bash plugins/exo/scripts/test-a1-gate.sh && ./scripts/test-plugin.sh && bash scripts/test-exec-bit.sh`
Expected: todo en verde.

- [ ] **Step R5: Commit**

```bash
git add plugins/exo/scripts/exo-recall.sh plugins/exo/scripts/reflex-baseline.sh plugins/exo/scripts/a1-gate.sh plugins/exo/scripts/reflex-fp-review.sh plugins/exo/scripts/test-exo-recall.sh plugins/exo/scripts/test-reflex-baseline.sh plugins/exo/scripts/test-a1-gate.sh
git commit -m "feat(reflex-log): rotación a .1 en SessionStart y análisis sobre las dos generaciones (campaña A, H5-R)"
```

---

### Task 12: `exo config --json` memoizado por sesión

**Lane:** mecánica · **Hallazgo:** H10 · **Bloqueada por:** la puerta C-H10 ABIERTA (Linux en `predicciones.txt`, o W11 > 100 ms en la Task 15), D5, la Task 5 y, si D1 = A2, la Task 8. Con la puerta cerrada, **no se ejecuta**, y en la Task 14 solo se corrige el comentario de `compose-inject.sh:4` con el número medido.
**Oráculo:** `bash plugins/exo/scripts/test-recall-inject.sh && bash plugins/exo/scripts/test-subagent-inject.sh` + O-plugin

**Files:**
- Create: `plugins/exo/scripts/_exo-config.sh` (100755, como el resto de helpers)
- Modify: `plugins/exo/scripts/recall-inject.sh` (extracción de prompt en `:31`; toda llamada `"$EXO_BIN" config --json`)
- Modify: `plugins/exo/scripts/exo-recall.sh` (la llamada a `config --json` y la extracción de `SID`, adelantada)
- Modify: `plugins/exo/scripts/subagent-inject.sh:27-30` (pasa `--kb` resuelta del memo)
- Modify: `plugins/exo/scripts/compose-inject.sh:4` (comentario)
- Test: `plugins/exo/scripts/test-recall-inject.sh`

**Interfaces:**
- Produces: `exo_config_json <exo_bin> <session_id>` → stdout = envelope de `exo config --json`. Con `session_id` vacío o con caracteres fuera de `[A-Za-z0-9_-]`, llama directo, sin caché. La caché vive en `${EXO_CONFIG_CACHE_DIR:-${TMPDIR:-/tmp}}/exo-config-<sid>.json` y **solo** se escribe con exit 0.

- [ ] **Step 1: Escribir el test que falla**

Añadir a `test-recall-inject.sh`, antes del resumen final:

```bash
# ------------------------------ H10: una sola `exo config` por sesión ---
CNT_EXO="$TMP/exo-cuenta-config"
cat > "$CNT_EXO" <<EOF
#!/usr/bin/env bash
if [ "\$1" = "config" ]; then
  echo x >> "$TMP/config-calls.txt"
  printf '{"schema_version":2,"command":"config","data":{"kb":{"name":"kb-demo","path":"/kb"},"index":{"db":"%s"}}}\n' "$FAKE_DB"
  exit 0
fi
exit 1
EOF
chmod +x "$CNT_EXO"
: > "$TMP/config-calls.txt"
mkdir -p "$TMP/cache-cfg"
for sid in sess-uno sess-uno sess-dos; do
  printf '%s' "M6-06" | jq -Rs --arg s "$sid" '{prompt:., session_id:$s}' \
    | EXO_BIN="$CNT_EXO" EXO_KB_NAME="" EXO_CONFIG_CACHE_DIR="$TMP/cache-cfg" "$HOOK" >/dev/null 2>&1
done
N_CFG="$(wc -l < "$TMP/config-calls.txt" | tr -d ' ')"
if [ "$N_CFG" -eq 2 ]; then pass "H10: config una vez por sesión (3 prompts, 2 sesiones → 2)"
else fail "H10: config una vez por sesión" "llamadas=$N_CFG"; fi
```

- [ ] **Step 2: Verificar que falla**

Run: `bash plugins/exo/scripts/test-recall-inject.sh | grep -E 'H10|failed'`
Expected: `[FAIL] H10: config una vez por sesión — llamadas=3`.

- [ ] **Step 3: Implementación**

`plugins/exo/scripts/_exo-config.sh` (nuevo):
```bash
#!/usr/bin/env bash
# Helper COMPARTIDO (H10): `exo config --json` memoizado por sesión.
#
# Uso:  . "$SCRIPT_DIR/_exo-config.sh"; exo_config_json <exo_bin> <session_id>
# stdout = el envelope de `exo config --json`; exit = el de exo, o 0 si sale de caché.
#
# Por qué: recall-inject.sh la pedía en cada prompt y subagent-inject en cada
# subagente, y bajo Git Bash cada spawn cuesta decenas de ms (puerta C-H10 del
# pre-registro de la campaña A). La config no cambia a mitad de sesión en el
# uso normal; si alguien la edita, la sesión siguiente la ve.
# Sin session_id no hay caché. Un error no se memoiza.
exo_config_json() {
  local bin="$1" sid="${2:-}" dir f tmp salida
  case "$sid" in
    ''|*[!A-Za-z0-9_-]*) "$bin" config --json; return ;;
  esac
  dir="${EXO_CONFIG_CACHE_DIR:-${TMPDIR:-/tmp}}"
  f="$dir/exo-config-$sid.json"
  if [ -s "$f" ]; then
    printf '%s\n' "$(< "$f")"
    return 0
  fi
  salida="$("$bin" config --json)" || return $?
  tmp="$(mktemp "$dir/exo-config.XXXXXX" 2>/dev/null)" \
    && printf '%s\n' "$salida" > "$tmp" && mv -f "$tmp" "$f" 2>/dev/null
  printf '%s\n' "$salida"
}
```

`recall-inject.sh`:

(a) Sustituir la línea 31:
```bash
PROMPT="$(printf '%s' "$INPUT" | jq -r '.prompt // empty' 2>/dev/null)" || PROMPT=""
```
por:
```bash
# session_id y prompt en UNA pasada de jq (H10), separados por \001: el prompt
# puede traer saltos de línea y tabs, y el session_id no trae \001.
SID_PROMPT="$(printf '%s' "$INPUT" | jq -r '(.session_id // "") + "\u0001" + (.prompt // "")' 2>/dev/null)" || SID_PROMPT=""
SID="${SID_PROMPT%%$'\001'*}"
PROMPT="${SID_PROMPT#*$'\001'}"
[ "$PROMPT" = "$SID_PROMPT" ] && PROMPT=""
```

(b) Justo después de `SCRIPT_DIR=…` (línea 16):
```bash
. "$SCRIPT_DIR/_exo-config.sh" 2>/dev/null || exo_config_json() { "$1" config --json; }
```

(c) Sustituir la invocación `"$EXO_BIN" config --json` (una sola en el script, en el bloque de config) por `exo_config_json "$EXO_BIN" "$SID"`, dejando igual la redirección `2>…` que la sigue.

`exo-recall.sh`:

(d) Justo después de `if [ -t 0 ]; then INPUT=""; else INPUT="$(cat)"; fi` (línea 21; el SID necesita `INPUT`):
```bash
. "$SCRIPT_DIR/_exo-config.sh" 2>/dev/null || exo_config_json() { "$1" config --json; }
# SID adelantado (H10): calienta el memo de config de esta sesión para el
# primer prompt. La reafirmación de abajo lo reutiliza.
SID="$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)" || SID=""
# Memos de sesiones viejas: se barren aquí, una vez por arranque.
find "${EXO_CONFIG_CACHE_DIR:-${TMPDIR:-/tmp}}" -maxdepth 1 -name 'exo-config-*.json' -mtime +2 -delete 2>/dev/null || true
```

(e) Sustituir `"$EXO_BIN" config --json` por `exo_config_json "$EXO_BIN" "$SID"`, y borrar la línea posterior
`SID="$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)" || SID=""` del bloque de reafirmación (ya está arriba).

`subagent-inject.sh`, sustituir:
```bash
KB_ARGS=()
[ -n "${REFLEX_INJECT_KB:-}" ] && KB_ARGS=(--kb "$REFLEX_INJECT_KB")
```
por:
```bash
KB_ARGS=()
if [ -n "${REFLEX_INJECT_KB:-}" ]; then
  KB_ARGS=(--kb "$REFLEX_INJECT_KB")
elif [ -z "${EXO_KB:-}" ] && command -v exo >/dev/null 2>&1; then
  # H10: la KB sale del memo de config de la sesión en vez de que
  # compose-inject.sh llame a `exo config` en cada subagente.
  . "$SCRIPT_DIR/_exo-config.sh" 2>/dev/null || exo_config_json() { "$1" config --json; }
  KB_MEMO="$(exo_config_json exo "$SID" 2>/dev/null | jq -r '.data.kb.path // empty' 2>/dev/null)" || KB_MEMO=""
  [ -n "$KB_MEMO" ] && KB_ARGS=(--kb "$KB_MEMO")
fi
```

`compose-inject.sh:4`, sustituir `# Sin cache en v1: solo lecturas locales (<5ms). Se cacheara cuando el backend sea exo recall.` por:
```bash
# Sin cache propia: la KB llega por --kb (subagent-inject.sh la resuelve del memo
# de sesión, _exo-config.sh). El fallback a `exo config --json` + jq de abajo
# medía ~5-6 ms + jq en Linux (2026-09-13); en Git Bash, bastante más.
```

- [ ] **Step 4: Verificar que pasa**

Run: `chmod +x plugins/exo/scripts/_exo-config.sh && bash plugins/exo/scripts/test-recall-inject.sh && bash plugins/exo/scripts/test-subagent-inject.sh && bash plugins/exo/scripts/test-compose-inject.sh && ./scripts/test-plugin.sh && bash scripts/test-exec-bit.sh`
Expected: todo en verde. Los tests de `no-config` siguen pasando porque un error no se memoiza.

- [ ] **Step 5: Commit**

```bash
git add plugins/exo/scripts/_exo-config.sh plugins/exo/scripts/recall-inject.sh plugins/exo/scripts/exo-recall.sh plugins/exo/scripts/subagent-inject.sh plugins/exo/scripts/compose-inject.sh plugins/exo/scripts/test-recall-inject.sh
git commit -m "perf(hooks): exo config --json memoizado por sesión (campaña A, H10)"
```

---

### Task 13: criterio de reapertura del proceso residente

**Lane:** mecánica · **Hallazgo:** H3 · **Bloqueada por:** la Task 5 (formato del payload) y D4 (umbrales). Los defaults son la propuesta del pre-registro: si Paul fija otros, sustituye `UMBRAL_P95_MS`, `UMBRAL_TIMEOUT_PCT` y `MIN_DISPAROS` **y** los números del test **y** §«Criterio de reapertura» del pre-registro, antes del Step 1.
**Oráculo:** `bash plugins/exo/scripts/test-recall-latencia.sh` + O-plugin

**Files:**
- Create: `plugins/exo/scripts/recall-latencia.sh` (100755)
- Test: `plugins/exo/scripts/test-recall-latencia.sh` (100755)

**Interfaces:**
- Consumes: eventos `recall-inject-emitted` con payload `… elapsed_ms=<int> refresh_ms=<int> …` (Task 5) y `recall-inject-degraded` con `reason=timeout-guard` (`recall-inject.sh:155`).
- Produces: `recall-latencia.sh [DESDE] [HASTA]` (YYYY-MM-DD, inclusivas). Imprime líneas TSV `disparos_medidos`, `timeouts`, `p50_ms`, `p95_ms`, `timeout_pct` y `veredicto` (`REABRIR`, `NO-REABRIR` o `INSUFICIENTE …`). Exit 0 si pudo calcular, 1 si hay error de entrada. El veredicto es información, no un gate.

- [ ] **Step 1: Escribir el test que falla**

`plugins/exo/scripts/test-recall-latencia.sh`:

```bash
#!/usr/bin/env bash
# Test standalone para recall-latencia.sh. Logs sintéticos en mktemp -d.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="${SCRIPT_DIR}/recall-latencia.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

# emite <n> <elapsed_ms> <refresh_ms> <session> <fecha>
emite() {
  awk -v n="$1" -v e="$2" -v r="$3" -v s="$4" -v d="$5" 'BEGIN {
    for (i = 0; i < n; i++)
      printf "{\"ts\":\"%sT10:00:00Z\",\"reflex\":\"recall-inject-emitted\",\"session_id\":\"%s\",\"agent_id\":\"\",\"agent_type\":\"\",\"tool\":\"\",\"payload\":\"n_hits=3 bytes=900 elapsed_ms=%d refresh_ms=%d permalinks=kb/a,kb/b\"}\n", d, s, e, r
  }'
}
timeouts() {  # timeouts <n> <fecha>
  awk -v n="$1" -v d="$2" 'BEGIN { for (i = 0; i < n; i++)
    printf "{\"ts\":\"%sT10:00:00Z\",\"reflex\":\"recall-inject-degraded\",\"session_id\":\"s\",\"agent_id\":\"\",\"agent_type\":\"\",\"tool\":\"\",\"payload\":\"reason=timeout-guard t=5s\"}\n", d }'
}
veredicto() { printf '%s\n' "$1" | awk -F'\t' '$1 == "veredicto" { print $2 }'; }
campo() { printf '%s\n' "$2" | awk -F'\t' -v k="$1" '$1 == k { print $2 }'; }

# 1. Sano: 200 a 900+10 ms y 10 a 1990+10 ms → p95 = 910 → NO-REABRIR.
{ emite 200 900 10 s 2026-10-01; emite 10 1990 10 s 2026-10-01; } > "$TMP/sano.jsonl"
OUT="$(REFLEX_LOG_FILE="$TMP/sano.jsonl" bash "$SCRIPT")"
[ "$(veredicto "$OUT")" = "NO-REABRIR" ] && [ "$(campo p95_ms "$OUT")" = "910" ] \
  && pass "sano: NO-REABRIR con p95=910" || fail "sano" "$OUT"

# 2. Lento: 100 a 900 y 110 a 2000 → p95 = 2010 > 1500 → REABRIR.
{ emite 100 900 10 s 2026-10-01; emite 110 2000 10 s 2026-10-01; } > "$TMP/lento.jsonl"
OUT="$(REFLEX_LOG_FILE="$TMP/lento.jsonl" bash "$SCRIPT")"
[ "$(veredicto "$OUT")" = "REABRIR" ] && pass "lento: REABRIR por p95" || fail "lento" "$OUT"

# 3. Timeouts: 200 sanos y 10 timeouts → 4,76% > 2% → REABRIR.
{ emite 200 900 10 s 2026-10-01; timeouts 10 2026-10-01; } > "$TMP/to.jsonl"
OUT="$(REFLEX_LOG_FILE="$TMP/to.jsonl" bash "$SCRIPT")"
[ "$(veredicto "$OUT")" = "REABRIR" ] && [ "$(campo timeouts "$OUT")" = "10" ] \
  && pass "timeouts: REABRIR por >2%" || fail "timeouts" "$OUT"

# 4. Pocos datos: 50 disparos → INSUFICIENTE, aunque sean lentos.
emite 50 3000 10 s 2026-10-01 > "$TMP/poco.jsonl"
OUT="$(REFLEX_LOG_FILE="$TMP/poco.jsonl" bash "$SCRIPT")"
case "$(veredicto "$OUT")" in INSUFICIENTE*) pass "pocos datos: INSUFICIENTE" ;; *) fail "pocos datos" "$OUT" ;; esac

# 5. Las sesiones test* no cuentan, y la ventana de fechas filtra.
{ emite 250 3000 10 test-sess 2026-10-01; emite 250 900 10 s 2026-09-01; emite 210 900 10 s 2026-10-05; } > "$TMP/filtro.jsonl"
OUT="$(REFLEX_LOG_FILE="$TMP/filtro.jsonl" bash "$SCRIPT" 2026-10-01 2026-10-14)"
[ "$(campo disparos_medidos "$OUT")" = "210" ] && [ "$(veredicto "$OUT")" = "NO-REABRIR" ] \
  && pass "filtro: excluye test* y fuera de ventana" || fail "filtro" "$OUT"

# 6. Payloads sin tiempos (anteriores a la campaña A) no rompen ni cuentan.
{ printf '{"ts":"2026-10-01T10:00:00Z","reflex":"recall-inject-emitted","session_id":"s","agent_id":"","agent_type":"","tool":"","payload":"n_hits=3 bytes=900 permalinks=kb/a"}\n'; emite 200 900 10 s 2026-10-01; } > "$TMP/viejo.jsonl"
OUT="$(REFLEX_LOG_FILE="$TMP/viejo.jsonl" bash "$SCRIPT")"
[ "$(campo disparos_medidos "$OUT")" = "200" ] && pass "viejos: se ignoran los payloads sin elapsed_ms" || fail "viejos" "$OUT"

# 7. Log inexistente → exit 1.
REFLEX_LOG_FILE="$TMP/no-existe.jsonl" bash "$SCRIPT" >/dev/null 2>&1
[ $? -eq 1 ] && pass "sin log: exit 1" || fail "sin log" "exit distinto de 1"

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
```

- [ ] **Step 2: Verificar que falla**

Run: `bash plugins/exo/scripts/test-recall-latencia.sh`
Expected: FAIL en los 7 casos: `recall-latencia.sh` no existe y `bash` sale con 127.

- [ ] **Step 3: Implementación**

`plugins/exo/scripts/recall-latencia.sh`:

```bash
#!/usr/bin/env bash
# Criterio de reapertura del proceso residente (H3, campaña A). READ-ONLY.
#
# La spec 2026-08-22-m6-06-recall-punto-de-uso-design.md §2.2 aceptó «el segundo
# por turno» con reapertura «si duele tras semanas de uso», sin mecanismo que
# detectara el dolor. Esto es ese mecanismo: lee los eventos de recall-inject.sh
# del log de ESTA máquina (W11 y Linux se evalúan por separado).
#
# Uso: recall-latencia.sh [DESDE] [HASTA]   (YYYY-MM-DD, inclusivas; por defecto todo)
# Umbrales HARDCODEADOS adrede: son pre-registro
# (docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md §«Criterio de reapertura»).
set -uo pipefail
export LC_NUMERIC=C

LOG="${REFLEX_LOG_FILE:-$HOME/.claude/reflex-log.jsonl}"
DESDE="${1:-0000-00-00}"
HASTA="${2:-9999-12-31}"
UMBRAL_P95_MS=1500
UMBRAL_TIMEOUT_PCT=2
MIN_DISPAROS=200

command -v jq >/dev/null 2>&1 || { echo "jq requerido" >&2; exit 1; }
[ -f "$LOG" ] || { echo "no existe $LOG" >&2; exit 1; }
jq -e . "$LOG" >/dev/null 2>&1 || { echo "ERROR: $LOG contiene JSON inválido" >&2; exit 1; }

jq -rs --arg d "$DESDE" --arg h "${HASTA}T23:59:59Z" \
  --argjson p95max "$UMBRAL_P95_MS" --argjson tmax "$UMBRAL_TIMEOUT_PCT" --argjson nmin "$MIN_DISPAROS" '
  def campo($k): ((capture("(^| )" + $k + "=(?<v>[0-9]+)") | .v | tonumber) // null);
  [ .[] | select(.ts >= $d and .ts <= $h)
        | select((.session_id // "") | ascii_downcase | startswith("test") | not) ] as $ev
  | [ $ev[] | select(.reflex == "recall-inject-emitted") | (.payload // "")
      | {e: campo("elapsed_ms"), r: campo("refresh_ms")} | select(.e != null)
      | .e + (.r // 0) ] | sort as $ms
  | ([ $ev[] | select(.reflex == "recall-inject-degraded"
                      and ((.payload // "") | contains("reason=timeout-guard"))) ] | length) as $to
  | ($ms | length) as $n
  | (if $n == 0 then null else $ms[(($n - 1) * 0.5 | floor)] end) as $p50
  | (if $n == 0 then null else $ms[(($n - 1) * 0.95 | floor)] end) as $p95
  | (if ($n + $to) == 0 then 0 else ($to * 100 / ($n + $to)) end) as $tpct
  | "disparos_medidos\t\($n)",
    "timeouts\t\($to)",
    "p50_ms\t\($p50)",
    "p95_ms\t\($p95)",
    "timeout_pct\t\($tpct)",
    ( if ($n + $to) < $nmin then "veredicto\tINSUFICIENTE (menos de \($nmin) disparos)"
      elif ($p95 != null and $p95 > $p95max) or $tpct > $tmax then "veredicto\tREABRIR"
      else "veredicto\tNO-REABRIR" end )
' "$LOG"
```

- [ ] **Step 4: Verificar que pasa**

Run: `chmod +x plugins/exo/scripts/recall-latencia.sh plugins/exo/scripts/test-recall-latencia.sh && bash plugins/exo/scripts/test-recall-latencia.sh && ./scripts/test-plugin.sh && bash scripts/test-exec-bit.sh`
Expected: `7 passed, 0 failed` y el plugin en verde en los tres SO (CI).

Run, en solo lectura sobre el log real y como dato para el veredicto:
`bash plugins/exo/scripts/recall-latencia.sh`
Expected: `INSUFICIENTE` hasta que la Task 5 esté instalada y acumule
disparos. Los payloads anteriores no traen `elapsed_ms`.

- [ ] **Step 5: Commit**

```bash
git add plugins/exo/scripts/recall-latencia.sh plugins/exo/scripts/test-recall-latencia.sh
git commit -m "feat(plugin): recall-latencia.sh — criterio pre-registrado de reapertura del proceso residente (campaña A, H3)"
```

---

### Task 14: bench «después», veredicto y líneas de estado en el backlog

**Lane:** mecánica · **Hallazgos:** todos · **Bloqueada por:** las Tasks 3–13 que apliquen, mergeadas o en la pila de la rama. **Antes** de que entre en `main` un cambio de la campaña C a `trozos.rs`.
**Oráculo:** `evals/recall-coste/harness/bench.sh despues 174 1000 5000 && evals/recall-coste/harness/compara.sh baseline despues`

**Files:**
- Create: `evals/recall-coste/results/despues/` (lo genera el bench)
- Create: `evals/recall-coste/verdict/2026-09-campana-a.md`
- Modify: `docs/backlog.md` (solo se AÑADEN líneas al final de cuatro items, localizados por título)

- [ ] **Step 1: Condiciones**

Run: `uptime; pgrep -fa 'cargo|exo ' | grep -v pgrep; git log --oneline -1 -- engine/src/trozos.rs`
Expected: la carga del primer minuto por debajo de 1,0, sin procesos, y el
último commit a `trozos.rs` anterior a `3c1918f`. Si C ya lo cambió, **no
compares**: escribe el veredicto con «línea base no comparable» y para.

- [ ] **Step 2: Correr y comparar**

Run: `evals/recall-coste/harness/bench.sh despues 174 1000 5000 && evals/recall-coste/harness/compara.sh baseline despues | tee evals/recall-coste/results/despues/criterios.txt`
Expected: la tabla y las líneas `C-H27`, `C-H4`, `C-H17a`, `C-H17b`, `C-H23` y
`C-noregresión`, cada una con su veredicto.

- [ ] **Step 3: Escribir el veredicto**

`evals/recall-coste/verdict/2026-09-campana-a.md`. La estructura es fija; las
cifras se **pegan** de los ficheros, no se reescriben a mano:

```markdown
# Veredicto del bench — campaña A (recall por prompt)

- Pre-registro: `docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md` (commit `<git log -1 --format=%h -- ese fichero>`)
- Línea base: `evals/recall-coste/results/baseline/` (`entorno.txt` pegado abajo)
- Después: `evals/recall-coste/results/despues/` (`entorno.txt` pegado abajo)

## Predicciones (línea base)
<contenido literal de results/baseline/predicciones.txt>

## Criterios
<contenido literal de results/despues/criterios.txt>

## Tabla antes/después
<salida literal de la tabla de compara.sh>

## Derivaciones
- C-H17b → <si DERIVAR: «item para la campaña C: s2 n5000−n174 = N ms (KNN/HashMap de trozos, buscador.rs:286-294)»; si no: «ninguna»>
- C-H23 → <si ABRIR ITEM: «item de backlog: kb-precommit a 5.000 notas p50 = N ms»; si no: «ninguna»>
- H29 (`.git` en el walker): s3 n5000 p50 = <de la tabla> ms. Solo es dato; sin criterio pre-registrado, no decide.
- `rebuild` extrapolado: s9 n5000 p50 × 5000 = <cálculo> ms en `git log` por nota (extrapolación, no medida).
- H5: s10 l2174 / l20000 / l200000 p50 = <tres cifras> ms → dato para D3.

## W11 (Task 15)
<salida literal del bloque §W11, o «no medido (D5)»>

## Lo que buscó este veredicto para objetar
<obligatorio: qué número se miró para intentar tumbar cada PASA, aunque no se encontrara nada>
```

Los `<…>` de este bloque no son placeholders del plan: son la instrucción de
pegar la salida de comandos ya ejecutados en los Steps 2 de las Tasks 2 y 14.
Un veredicto con un `<` sin sustituir no se commitea.

- [ ] **Step 4: Líneas de estado en `docs/backlog.md`**

Solo se AÑADE una línea al final de cada item. Los items se localizan por el
texto de su título, con `grep -n`:

1. `grep -n "tier\` no se persiste en el índice" docs/backlog.md` → al final de ese item:
   `  **(campaña A, <fecha>):** <si Task 10 ejecutada: «resuelto sin schema en el camino del hook: recientes perezosas en --content --note (D2=L); s4 n5000 p95 X→Y ms»; si no: «puerta C-H17a cerrada: s4 n5000 p95 = X ms ≤ 250; no se toca»>. Veredicto: evals/recall-coste/verdict/2026-09-campana-a.md.`
2. `grep -n "Techos de escala declarados, sin camino ni" docs/backlog.md` → al final:
   `  **(campaña A, <fecha>):** medido con KB sintética de 174/1000/5000 notas (evals/recall-coste/). Encontrado y arreglado un techo DURO no listado: KNN de vec0 limitado a k=4096 (H27). Números y derivaciones en el veredicto.`
3. `grep -n "El coste del hook completo en Windows no está" docs/backlog.md` → al final:
   `  **(campaña A, <fecha>):** Linux: s6 hook entero n174 p50 = X ms, s7 config+jq = Y ms. W11: <cifras de la Task 15 o «pendiente (D5)»>. El término dominante es la carga del modelo (≈0,95 s de ≈1 s), no el shell.`
4. `grep -n "Modo mudo de \`busca_hybrid\`: cerrado" docs/backlog.md` → al final:
   `  **(campaña A, <fecha>):** el cierre era parcial — \`exo recall\` descartaba los avisos (H2). Reabierto y cerrado en la campaña A: \`warnings\`/\`elapsed_s\`/\`refresh_s\` en el envelope de recall y \`engine-warning\` en el log del hook.`

(Si la campaña B ya reescribió el item 4 por H13, se añade la línea al texto
nuevo, sin tocar lo que escribió B.)

- [ ] **Step 5: Commit**

```bash
git add evals/recall-coste/results/despues evals/recall-coste/verdict/2026-09-campana-a.md docs/backlog.md
git commit -m "bench(campaña A): corrida después, veredicto contra el pre-registro y estado en backlog"
```

---

### Task 15: medición en W11 (manual, Paul)

**Lane:** fuera de la fábrica, **tarea manual de Paul** · **Hallazgos:** H3 H10 · **Bloqueada por:** D5.

- [x] **Step 1:** en la W11, desde Git Bash, con `exo --version` a mano, correr el bloque §W11 del pre-registro tal cual.
- [x] **Step 2:** pegar la salida completa en un comentario del ledger o en `evals/recall-coste/results/w11-<fecha>.txt`. Con eso, la Task 12 aplica C-H10 (config+jq p50 > 100 ms ⇒ puerta ABIERTA) y la Task 14 lo incorpora al veredicto.
- [ ] **Step 3:** si la Task 13 ya está instalada en esa máquina, correr `bash <plugin>/scripts/recall-latencia.sh` dentro de 14 días y anotar el veredicto. Así se evalúa por primera vez el criterio de reapertura sobre la máquina donde más duele.

> **Hecho el 2026-09-15** (`evals/recall-coste/results/w11-2026-09-15.txt`): Steps 1 y 2 ⇒ `config_jq_p50_ms 68`, C-H10 CERRADA en W11 y Linux, Task 12 no se ejecuta. Step 3 corrido una vez: `INSUFICIENTE` (3 disparos con `elapsed_ms`); queda abierto hasta tener 200 en la ventana. Ojo, hallazgo nuevo: ese criterio solo mide el tiempo interno del engine, ≈1,0 s de los ≈2,3 s que tarda el hook en W11 (ver `docs/backlog.md`). El hint del bloque (`plugins/exo/scripts`) no corresponde a la caché instalada, donde la ruta es `<installPath>/scripts`.

---

## Self-review (hecha contra el brief y los hallazgos)

- **Cobertura:** H1 → Tasks 7 y 8 (D1) · H2 → Tasks 4 y 5 · H3 → Tasks 1, 2, 5, 13, 14 y 15 (el proceso residente queda fuera; solo instrumentación, medida y criterio) · H4 → Task 6 · H5 → Task 11 (D3) y `s10` · H10 → Task 12 (puerta C-H10 y D5) · H17a → Task 10 (D2 y puerta C-H17a) · H17b → `s1b`, `s2`, `s5` y la derivación C-H17b · H19 → Task 9 · H23 → `s8` y C-H23 · H27 (nuevo) → Task 3 · H28 → derivado a C · H29 → dato en `s3`.
- **Placeholders:** los `<…>` solo aparecen en la plantilla de veredicto y en las líneas de backlog de la Task 14, que es donde se pega la salida de comandos que ya se ejecutaron. Van declarados como tal.
- **Consistencia de firmas:** `Recall.{elapsed_s, refresh_s, avisos}` (Task 4) = lo que leen la Task 5 (`.data.elapsed_s|refresh_s|warnings`) y el contrato. El payload `elapsed_ms=… refresh_ms=…` (Task 5) = la regex de la Task 13. `comprueba_kb_root` y `valida_db_para_kb` (Task 7) = lo que usa la Task 8. `knn` mantiene su firma (Task 3). Los ids de `bench.sh` coinciden con los de `compara.sh`.
- **Riesgos que quedan declarados:** la guarda de H1 puede destapar tests que ya compartían una DB entre dos KBs (Task 7, Step 4: se separan, no se relaja la guarda). El equivalente de la Task 11-R en `test-a1-gate.sh` compara salidas enteras: si `a1-gate.sh` imprime la ruta del log, esa línea diferirá, y hay que compararlo con `grep -v "$LOG1R"`. P1–P4 pueden fallar y eso no invalida la campaña: se reporta.

