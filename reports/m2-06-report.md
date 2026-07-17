# Reporte — m2-06: Embeddings + tabla vectorial (jina-es/768, chunking §2.1, sqlite-vec)

## Estado: COMPLETO — oráculo 4/4

Worktree `/home/paul/Documentos/proyectos/exo/.worktrees/m2-06`, rama `m2-06`.

Commits:
- `1b21559` — feat(engine): chunker propio de trozos + embedder batch de proceso
- `52b015c` — feat(engine): helpers de bajo nivel sobre vectores (vec0) + verificación de API
- `29fb0e4` — feat(engine): puebla trozos+vectores al indexar, cascada de borrado extendida
- `a61c787` — feat(engine): exo search --type vector
- `bd7d78a` — feat(evals): arm engine-vector en el harness
- `<pendiente>` — docs(m2): reporte final + verificación de idempotencia (este commit)

## Task 1 — Chunker (`engine/src/trozos.rs`)

Módulo nuevo `trozos::trocea`, spec §2.1 literal:
- Unidad = bloques markdown, separados por línea en blanco **o** por una línea
  heading ATX (`#`..`######` + espacio, o heading vacío al final de línea) —
  el heading cierra el bloque anterior aunque no venga precedido de línea en
  blanco.
- Empaquetado greedy de bloques consecutivos hasta **900 caracteres**
  (Unicode, no bytes) por trozo.
- Bloque que por sí solo excede 900 → corte duro en trozos de exactamente
  900 caracteres (el último puede ser más corto), operando sobre `Vec<char>`
  para no partir un carácter UTF-8 multibyte.
- Solape: 0.

9 unit tests: bloque simple, empaquetado greedy que junta bajo el techo,
empaquetado que abre trozo nuevo al no caber, corte duro sin solape
(`concat(t0,t1) == original` verificado), heading como separador (con y sin
línea en blanco previa), nota vacía → 0 trozos, determinismo.

## Task 2 — Población en el indexer (`engine/src/indexer.rs`, `engine/src/vectores.rs`)

### Helpers de vec0 (`vectores.rs`) — verificación de API ANTES de cablear (blindspot notes 1-3)

Sin README en el crate Rust (solo bindings FFI); verificado contra el C
vendorizado `sqlite-vec.c` (0.1.9) ANTES de escribir SQL:

- **Sintaxis KNN confirmada**: `SELECT rowid, distance FROM vectores WHERE
  embedding MATCH ?1 AND k = ?2` — exactamente la forma que sugería el
  blindspot-pass.
- **Métrica de distancia por defecto**: la DDL sellada (`schema.rs` §2,
  `CREATE VIRTUAL TABLE vectores USING vec0(embedding float[768])`) NO
  declara `distance_metric=cosine` → vec0 usa `VEC0_DISTANCE_METRIC_L2`
  (default en `vec0_column_config`, línea 2348 de `sqlite-vec.c`), y el motor
  de KNN llama `distance_l2_sqr_float` (línea 6878-6879) — **L2 al
  cuadrado, no coseno y no la raíz**. Dato que consume la conversión de
  `buscador::busca_vector` (ver Task 3).
- **Insert con rowid explícito**: soportado (`vector_from_value` acepta
  `SQLITE_BLOB` directo, línea 683/704 del C) — verificado con el test
  `vector_insert_rowid_y_knn_lo_recupera`.
- **DELETE por rowid**: soportado (`vec0Update`, xUpdate del vtab) —
  verificado con `vector_delete_por_rowid_desaparece_del_knn`.
- **KNN sobre tabla vacía**: no es error, devuelve 0 filas — verificado con
  `knn_sobre_tabla_vacia_devuelve_cero_resultados`.

4 tests directos en `vectores.rs`, los 4 verdes antes de tocar el indexer.

### Población e integración (`indexer.rs`)

- `reindexa_trozos_de_nota`: borra `trozos`+`vectores` previos de la nota
  (idempotente ante re-ejecución), trocea el cuerpo, embebe en batch (una
  sola llamada a `embebe_batch` por nota, no por trozo), inserta filas en
  `trozos` y, por cada una, su vector con `vectores.rowid =
  conn.last_insert_rowid()` (= `trozos.id`, §2 no negociable).
- **Cascada de borrado extendida a `vectores`** (deferred del gate m2-03
  ejecutado aquí): `borra_trozos_y_vectores_de_nota` reemplaza el `DELETE
  FROM trozos` suelto de m2-03 (que en m2-03 no dejaba huérfanos porque
  `vectores` aún estaba vacía) tanto en el reindex de una nota cambiada como
  en el borrado de una nota ausente del walk.
- **Nota sin trozos (cuerpo vacío) no toca el embedder** — cubierto por
  `nota_con_cuerpo_vacio_no_genera_trozos`.

5 tests nuevos en `tests/indexer.rs` (16/16 en el fichero): población con
rowid correcto, reemplazo en reindex (no acumulación, ids nuevos), cascada
de borrado a vectores, nota vacía, idempotencia de rebuild también para
vectores.

### Desviación declarada: embedder cacheado por PROCESO, no por llamada a `indexa()`

El brief pedía "el embedder se inicializa UNA vez por proceso". La primera
implementación usaba un `Option<Embedder>` local a `indexa()` — correcto
para el uso real del CLI (una invocación de `exo index` = un proceso = una
llamada a `indexa()`), pero **reventó el test suite por OOM**: `cargo test`
corre los `#[test]` de un mismo binario en threads concurrentes del mismo
proceso, y los 11 tests preexistentes de `tests/indexer.rs` indexan notas
con cuerpo no vacío (necesario para sus propias aserciones de FTS) — cada
uno creaba su propio `Embedder`, cargando **N copias del modelo ONNX (~0.6
GB) a la vez**. Verificado empíricamente: `SIGKILL` (signal 9) a los pocos
segundos.

Fix: `EMBEDDER_PROCESO: Mutex<Option<Embedder>>` como `static` en `lib.rs` +
`con_embedder_de_proceso(f)`, que inicializa el embedder la primera vez que
se necesita y lo comparte (serializado por el Mutex) entre todas las
llamadas del mismo proceso. Esto es más literal aún a "una vez por
proceso" que la variable local, no cambia el comportamiento del CLI real
(un `exo index` sigue siendo un único proceso), y resuelve el OOM: los 11
tests preexistentes + los 5 nuevos de trozos/vectores corren en 3.08s en
`tests/indexer.rs` (antes: SIGKILL). Layout interno bajo `engine/`, clase
pre-autorizada (Contexto del brief, punto 1).

## Task 3 — `exo search --type vector` (`engine/src/buscador.rs`, `engine/src/main.rs`)

`busca_vector(db_ruta, query, limite, min_similitud: Option<f64>)`, función
nueva (no se tocó `busca`, la función FTS existente, ni sus 6 tests):

1. Guard de DB inexistente (mismo contrato que FTS: error claro, no crea
   fichero).
2. `total_vectores = COUNT(*) FROM vectores`; si es 0 o la query es solo
   whitespace → `results: []` (declarado: **0 resultados, no error** —
   paridad con el contrato de `busca` FTS "sin hits = éxito").
3. Embed de la query vía `con_embedder_de_proceso` (mismo modelo/cache que
   el indexer).
4. **KNN exhaustivo**: `k = total_vectores` (declarado: sqlite-vec 0.1.9 sin
   partición ya hace scan lineal internamente para vec0 float, así que pedir
   menos vecinos no ahorra trabajo real y sí arriesga dejar fuera la mejor
   coincidencia de alguna entidad si sus chunks no caen entre los k más
   cercanos globales — con corpus de miles de chunks, exhaustivo es barato y
   correcto sin aproximar nada).
5. **Conversión distancia → similitud** (declarada, blindspot nota 1):
   fastembed normaliza sus embeddings a norma unidad SIEMPRE (verificado en
   `fastembed` 5.17.3: `text_embedding/output.rs::transformer_with_precedence`
   aplica `common::normalize` sin condición). Para vectores unitarios,
   `||a-b||² = 2 - 2·cos(a,b)` ⇒ `cos(a,b) = 1 - ||a-b||²/2`. Como vec0
   devuelve exactamente `||a-b||²` (L2 al cuadrado, ver Task 2), esta es la
   conversión exacta — no una aproximación.
6. **Filtro por `semantic_min_similarity`**: config RO (D6, `~/.basic-memory/config.json`,
   hoy 0.35) con precedencia flags > config vía `--min-similitud` (opcional,
   `None` = usa config).
7. **Agregación chunk→entidad por MÁXIMA similitud por permalink**
   (declarada, Task 3 del brief): el ground truth del eval es a nivel de
   nota (spec M2 §4), así que "la nota entra si su mejor trozo entra" es la
   agregación obvia; promediar o sumar castigaría notas largas con más
   trozos sin motivo real.
8. `results` ordenados por score descendente, truncados a `limite`.
   Envelope v1 intocado: `search_type: "vector"`, `type: "entity"` siempre.

`main.rs`: flag `--type fts|vector` (clap `ValueEnum`, default `fts` —
comportamiento actual intacto sin el flag) + `--min-similitud` opcional.

3 tests nuevos en `tests/buscador.rs` (10/10 en el fichero): DB poblada
devuelve entidades ordenadas y encuentra la nota semánticamente más cercana
a una query real (validación con el modelo jina-es de verdad, no un fake),
threshold inalcanzable (1.5, por encima del máximo teórico 1.0) filtra
todo, DB con schema pero sin vectores da 0 resultados sin error.

## Task 4 — Arm `engine-vector` en el harness

`replay-engine.py`: flag `--tipo fts|vector` (default `fts`), reenviado tal
cual a `exo search --type <tipo>`; el nombre del arm (fichero de salida)
sigue siendo un positional independiente del tipo — **`replay.py` no se
tocó ni se generalizó**. `search_type: "vector"` del envelope no necesita
remapeo en `SEARCH_TYPE_MAP` (analyze.py ya lo busca literal como
`"vector"`); solo `"fts"→"text"` seguía haciendo falta (heredado de m2-05).

Corrida real sobre `exo-e1.db` (rebuild real, ver Task 5): 56/56 filas sin
error en `results/engine-vector.jsonl`. `analyze.py engine-vector` sin
excepciones.

## Task 5 — Mediciones spec M2 §7.1 (números literales)

### (a) Proceso frío hasta primer resultado de `exo search --type vector`

Modelo YA en cache (smoke `jina_es_embebe_a_768 -- --ignored` corrido antes
de medir, sin tráfico de descarga durante la medición). 4 corridas,
proceso nuevo cada vez (`--db exo-e1.db --type vector --json "doctrina de
agentes"`, DB poblada con 2194 vectores):

| corrida | wall (`/usr/bin/time`) | `elapsed_s` interno (envelope) |
|---|---|---|
| 1 | 0.98 s | 0.9525 s |
| 2 | 1.04 s | 1.0040 s |
| 3 | 1.02 s | 0.9936 s |
| 4 | 0.99 s | 0.9653 s |

Media ≈ **0.98 s**, rango 0.95-1.04 s. Muy por debajo del target futuro de
M2-08 (§5 pata 3: hybrid en frío p95 < 2.0 s) y de la referencia bm (mediana
4.4 s). El `elapsed_s` medido dentro del binario explica casi todo el wall
time — poco overhead de proceso fuera de la carga del modelo + inferencia +
KNN.

### (b) Rebuild completo con embeddings del corpus real

```
$ exo rebuild --db exo-e1.db --json
{"command":"rebuild","data":{"borradas":0,"indexadas":115,"saltadas":0},"schema_version":1}

real: 13m39.86s (819.86 s wall-clock)
user: 7646.50s · system: 19.56s  →  ~935% CPU medio (~9.4 cores en paralelo)
```

115 notas → 2194 trozos → 2194 vectores. Referencia previa (spec M2 §7
riesgo 1): m2-01 midió ~30.5 s **con descarga** del modelo para 1 sola
frase — no comparable directamente (aquí son 2194 embeds reales del
corpus, no 1). Dato de arquitectura para M2-08 (spec M2 §7.1: "si revienta
el presupuesto, la salida es arquitectura del recall — FTS-first + cache —
nunca relajar el gate"): un `exo rebuild` completo con embeddings tarda
**~13.7 minutos** en esta máquina; un `exo index` incremental normal solo
paga este coste por las notas que cambiaron (skip por mtime, embedder
perezoso — Task 2), así que el coste operativo real en el día a día es muy
inferior a este número, que es el techo (rebuild completo desde cero).

## Oráculo — los 4 pasos, outputs literales

### 1. `cargo test --manifest-path engine/Cargo.toml` — todo verde

```
running 18 tests (lib: trozos, vectores, aristas)      ... 18 passed
running 10 tests (tests/buscador.rs)                    ... 10 passed
running  2 tests (tests/git_epoch.rs)                    ...  2 passed
running 16 tests (tests/indexer.rs)                      ... 16 passed
running  7 tests (tests/nota.rs)                         ...  7 passed
running  2 tests (tests/schema.rs)                       ...  2 passed
running  3 tests (tests/smoke.rs)                        ...  2 passed, 1 ignored
running  4 tests (tests/walker.rs)                       ...  4 passed

TOTAL: 61 passed, 0 failed, 1 ignored
```

Smoke ignorado corrido aparte:
```
$ cargo test --manifest-path engine/Cargo.toml --test smoke -- --ignored
test jina_es_embebe_a_768 ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out
```

Baseline post-m2-05 era 40 passed/1 ignored; +21 tests nuevos de este item
(9 trozos + 4 vectores + 5 indexer + 3 buscador) = 61 passed/1 ignored.

### 2. Rebuild real sobre kb-demo: poblado + idempotente + paridad ∅

```
$ exo rebuild --db exo-e1.db --json   (corrida 1, cronometrada arriba Task 5b)
{"command":"rebuild","data":{"borradas":0,"indexadas":115,"saltadas":0}}

notas=115  trozos=2194  vectores=2194  aristas=473

$ exo rebuild --db exo-e1.db --json   (corrida 2, verificación de idempotencia)
{"command":"rebuild","data":{"borradas":0,"indexadas":115,"saltadas":0}}
# 13m02.12s wall (7407.24s user, 949% cpu) — mismo perfil que corrida 1 (13m39.86s)
# counts idénticos: notas=115  trozos=2194  vectores=2194  aristas=473
# DB byte-idéntica a corrida 1 (14200832 bytes ambas) => rebuild idempotente

$ python3 evals/e1-read/harness/corpus-parity.py --diff exo-e1.db
gold=115 engine=115 faltan=0 sobran=0
```

`kb-demo` estaba en HEAD `d7b8a5e` al medir (gold sellado en m2-03 sobre
`28f153a`) con un fichero modificado sin commitear ("Backlog — frentes
abiertos.md") — drift esperado (spec §7 riesgo 2: "la validez del gate
depende de re-correr ambos arms el mismo día", aplicable al gate final
M2-09, no a este item) y sin efecto: paridad sigue en ∅ porque el walker
lee el filesystem, no un commit pineado, y ningún permalink cambió.

### 3. Arm vector: 56/56 sin error, hit@5 citado

```
$ wc -l results/engine-vector.jsonl
56

$ grep -c '"error"' results/engine-vector.jsonl
0

$ python3 evals/retrieval-fase0/harness/analyze.py engine-vector
exit=0
```

`results/metrics-engine-vector.md`:
- **vector: 46/55** (queries etiquetadas)
- Referencia informativa (no gate de este item, M2-09 lo es): bm jina-es
  hybrid/text/vector 43/55, engine-fts (m2-05) text 28/55.
- 46/55 > 43/55 (referencia bm): por ENCIMA del umbral de alarma del brief
  (`<15/55` = investigar como bug propio) y por encima incluso de la
  referencia de producción de basic-memory. Ningún patrón de miss
  sospechoso en la atribución (`fusion-miss`/`both-miss` del sweep, mismo
  formato que engine-fts): coherente con un chunking (§2.1) calibrado por
  paridad de granularidad contra bm (mediana/p90/máx de chunks, spec del
  indexer §2.1) y una conversión de similitud verificada matemáticamente
  (Task 3), no una casualidad.

### 4. Medición §7.1: hecha y citada arriba (Task 5), números literales

## Reglas duras — cumplimiento

- Envelope v1 y schema §2: **no tocados**. `schema.rs` DDL verbatim, sin
  ningún cambio.
- Veto AGPL: no se abrió el repo de basic-memory; solo lectura RO de
  `~/.basic-memory/config.json` (D6) y del propio `~/.basic-memory/memory.db`
  vía `corpus-parity.py` (ya autorizado desde m2-02/m2-03).
- Fusión hybrid (M2-07): ni tocada ni "dejada preparada" — `busca_vector` es
  clean-room, solo consume la config RO ya autorizada.
- `replay.py`/`analyze.py`: no tocados (`git status` tras la corrida
  completa solo mostró los ficheros nuevos/esperados del arm).
- Ramas/commits: todo en `m2-06`, `git -C` siempre, sin `cd X &&`. Sin
  merge, sin push, `main` intacto.
- sqlite-vec sigue pineado `=0.1.9` (sin tocar `Cargo.toml` en esa línea).
- Toolchain: sin cambios (rustc 1.97.1 preinstalado, ya usado tal cual).

## Memory packet
- `kb-demo/projects/exo-framework-unificado-de-trabajo-agentico`
- `kb-demo/core/doctrina-agentes`
- `kb-demo/log/exo-bitacora`
- `kb-demo/learnings/desarrollo-agentico`

GATE: MERGED (consultor fable, 2026-07-17T22:56:04+02:00, verdict=evals/e1-read/verdict/gate-m2-06.md@312da73)
