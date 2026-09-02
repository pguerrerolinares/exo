# Verdict del gate — m2-06 (embeddings + tabla vectorial)

- **Veredicto: MERGED**
- **Adjudica**: consultor Fable delegado (dispatch fresco, sin participación en ninguna fase de m2-06), régimen de gates de `.superpowers/fabrica/config.md` §"Ejecución de gates" (4 condiciones, todas cumplidas abajo).
- **Fecha**: 2026-07-17 (~22:20 local)
- **Rama juzgada**: `m2-06` (HEAD `7f96709`), diff vs `main`: 12 ficheros, +1353/−78.
- **Criterio citado**: spec M2 `docs/superpowers/specs/2026-07-17-m2-e1-read-design.md` §3 (fila M2-06: "hit@5 vector vía harness; referencia bm 43/55"), §4, §7.1; spec indexer `docs/superpowers/specs/2026-07-17-indexer-design.md` §2, §2.1, §3, §4.1.

## Oráculos re-corridos (verificación primaria propia, no del reporte)

### 1. `cargo test` — 61 passed / 0 failed / 1 ignored

Re-corrido completo por mí (dos veces; outputs literales de la segunda, suite por suite):

```
lib.rs        18 passed   (trozos 9 + vectores 4 + resto)
main.rs        0
buscador.rs   10 passed   (3 nuevos del arm vector, con el modelo jina real)
git_epoch.rs   2 passed
indexer.rs    16 passed   (5 nuevos: población, reemplazo, cascada, nota vacía, idempotencia)
nota.rs        7 passed
schema.rs      2 passed
smoke.rs       2 passed, 1 ignored
walker.rs      4 passed
```

Total 61/0/1 — coincide con lo declarado (baseline post-m2-05: 40/1; +21 de este item).
Smoke ignorado corrido aparte: `jina_es_embebe_a_768 ... ok` (1 passed, 1.13s con modelo en cache).

### 2. Rebuild real propio + integridad + paridad ∅

Rebuild ejecutado por mí desde cero sobre DB nueva (`gate-m206.db`, scratchpad):

```
{"command":"rebuild","data":{"borradas":0,"indexadas":115,"saltadas":0},"schema_version":1}
Elapsed (wall clock): 13:40.79   User: 7618.66s   System: 20.17s
Maximum resident set size: 7403276 kB (~7.1 GB)
DB final: 14200832 bytes  (md5 039c8c3078e60086a137efb6e91c841d)
```

- Counts verificados por mí en la DB: **notas=115, trozos=2194, vectores=2194, aristas=473** — idénticos a los declarados. `vectores` contado vía shadow table `vectores_rowids` (RO).
- **1:1 trozos↔vectores**: set de `trozos.id` == set de `vectores.rowid` exacto (0 huérfanos, 0 trozos sin vector). Además 0 trozos con permalink sin nota.
- **Idempotencia**: mi rebuild independiente reproduce byte-count idéntico (14200832) y counts idénticos a las dos corridas del executor — tres rebuilds independientes convergentes. El test `rebuild_doble_da_el_mismo_conteo_de_trozos_y_vectores` lo cubre además a nivel de suite.
- **Paridad**: `corpus-parity.py --diff gate-m206.db` → `gold=115 engine=115 faltan=0 sobran=0`, exit 0. **∅**.
- Estado de kb-demo al medir: HEAD `d7b8a5e` + 1 fichero modificado sin commitear ("Backlog — frentes abiertos.md") — el mismo estado que declaró el executor; drift sin efecto en paridad (spec M2 §7 riesgo 2 aplica al gate M2-09, no aquí).

### 3. Arm vector: hit@5 = 46/55, verificado dos veces

- `results/engine-vector.jsonl`: 56 filas, 0 con `"error"`.
- `analyze.py engine-vector` re-corrido por mí: exit 0, regenera `metrics-engine-vector.md` byte-idéntico (worktree limpio tras la corrida). **vector: 46/55**.
- **Recomputo independiente** (patrón m0-verdict): script propio desde el jsonl crudo + `eval.jsonl`, usando solo el `norm()` de analyze.py — **46/55**, y los 9 misses coinciden uno a uno con los `both-miss`/`vector=miss` del metrics. 46/55 > referencia bm 43/55 (criterio §3 fila M2-06) y muy por encima del umbral de alarma (<15/55).

### 4. Medición §7.1 (riesgo del modelo en frío) — números propios

- **Primer resultado de `exo search --type vector` en proceso frío** (modelo en cache de disco, 4 corridas, proceso nuevo cada una, DB de 2194 vectores): wall **0.95–1.02 s** (elapsed_s interno 0.928–0.985). Top-1 correcto y estable (`kb-demo/core/doctrina-agentes`, score 0.5309, determinista).
- **Rebuild completo con embeddings**: **13m40.79s** wall (mi corrida; executor: 13m39.86s y 13m02.12s — consistente), ~9.3 cores medios, pico de RAM ~7.1 GB.
- **Lectura**: el frío de search (~1 s) queda por debajo del presupuesto futuro de M2-08 (§5 pata 3: hybrid frío p95 < 2.0 s; referencia bm mediana 4.4 s), así que hoy NO revienta el presupuesto y no exige aún la salida de arquitectura. El coste real está en el rebuild completo (~13.7 min), que es el techo operativo (el `exo index` incremental solo re-embebe notas cambiadas, con embedder perezoso que ni se carga si no hay nada que embeber). Esto RESPALDA tener la arquitectura FTS-first/cache como plan si M2-08 midiera peor, exactamente como manda §7.1: "la salida es arquitectura, NUNCA relajar el gate" — nada en este item relaja gate alguno.

## Superficies gateadas — verificadas INTACTAS

- `git diff main...HEAD --name-only` sobre `engine/src/schema.rs`, `engine/src/envelope.rs`, `evals/retrieval-fase0/harness/replay.py`, `evals/retrieval-fase0/harness/analyze.py`, `engine/Cargo.toml`: **vacío** — ni un byte tocado.
- DDL sellada §2: `CREATE VIRTUAL TABLE vectores USING vec0(embedding float[768])` verbatim en schema.rs. La implicación (métrica default = L2², no coseno) está manejada en `busca_vector` con conversión exacta `cos = 1 − d²/2` (válida porque fastembed normaliza a norma unidad — verificado por el executor contra el código de fastembed 5.17.3 y coherente con los scores observados ≤1).
- Envelope v1 §4.1: `Busqueda`/`Resultado` sin cambios de forma; `search_type: "vector"`, `type: "entity"` siempre (verificado en código y en las 56 filas del jsonl). `--type` con default `fts` deja el comportamiento previo intacto.
- Pin `sqlite-vec = "=0.1.9"` intacto (Cargo.toml sin cambios).
- `replay.py` NO generalizado (spec M2 §4): el flag `--tipo` vive solo en `replay-engine.py`.
- Ningún fichero del diff fuera de `engine/`, `evals/`, `reports/`.

## Veto AGPL — verificado

- Grep de `entity|relation|search_index|search_vector_chunks|basic_memory|basicmachines|AGPL` sobre `engine/src/` y el harness: los únicos matches son (a) el literal `"entity"` del envelope, que es mandato del contrato §4.1, y (b) el comentario del propio `schema.rs` (sellado en m2-02, sin cambios aquí) que documenta el veto.
- `find` de clones/vendorizados de basic-memory en el worktree: ninguno.
- Accesos RO permitidos y solo esos: `~/.basic-memory/config.json` (D6 — verifiqué que los valores que el código asume coinciden con el fichero vivo: jina-es, 768, 0.35) y `memory.db` vía `corpus-parity.py` (autorizado desde m2-02/03).
- **Sin fusión hybrid**: `busca_vector` no contiene gate FTS, ni fórmula de fusión, ni bonus — M2-07 sigue clean-room por delante.

## Qué busqué para objetar (mandato de disenso)

1. **DDL sellada / envelope tocados** — lo busqué por diff nominal fichero a fichero y leyendo los structs serializados. Nada. La tentación de añadir `distance_metric=cosine` a la DDL habría sido el fallo típico; el executor en cambio resolvió la métrica en la capa de conversión, respetando el sello.
2. **Chunking §2.1 no literal** — leí `trozos.rs` completo contra la letra de §2.1: greedy ≤900 chars Unicode, solape 0, corte duro en 900 exactos sobre `Vec<char>`, heading como separador. Los 9 unit tests cubren los bordes (incluido `concat(t0,t1)==original` para el corte duro). Objeción menor encontrada y descartada como no-contractual: `es_heading` trataría un `# comentario` dentro de un code fence como separador (la spec §2.1 no menciona fences; imprecisión de chunking a nivel de nota, sin efecto en el contrato ni en el oráculo — si molesta, es parámetro del sweep M2-07).
3. **Vectores huérfanos en la cascada** — busqué el path que borrara `trozos` sin `vectores`: no existe; ambos paths (reindex de nota cambiada y nota ausente del walk) pasan por `borra_trozos_y_vectores_de_nota`, y lo verifiqué empíricamente en la DB real (0 huérfanos, 1:1 exacto). Nota no bloqueante: no hay transacción por nota, así que un crash a mitad de inserción podría dejar estado parcial — cubierto por diseño porque `rebuild` es la recuperación de primera clase (spec indexer §3: "corrupción de índice = borrar y rebuild").
4. **Rastro AGPL** — greps y find de arriba: nada. Además el diseño del search vector (KNN exhaustivo + max por permalink) no se parece al pipeline de bm (que fusiona con gate FTS), señal de diseño propio.
5. **Agregación chunk→entidad dudosa** — max por permalink con threshold pre-agregación: es la agregación correcta para ground truth a nivel de nota (§4: "results nivel entidad"); promedio/suma penalizaría notas largas. El orden desc y el truncado a limite están testeados. El 46/55 empírico (por encima de bm) confirma que no es una agregación rota.
6. **KNN con k=total como riesgo de rendimiento** — objeción considerada: es O(n) por query, pero sqlite-vec 0.1.9 sin partición ya escanea linealmente, y el wall medido (~1 s en frío, con la carga del modelo dentro) lo confirma como no-problema a esta escala. Decisión declarada, no accidente.
7. **Números del reporte inflados** — re-derivé cada número gateable por mi cuenta: 61 tests, 46/55 (dos métodos), 115/2194/2194/473, paridad ∅, 13m40s, ~1 s frío. Todos reproducen lo declarado; ninguna discrepancia.

## Cumplimiento de las 4 condiciones del régimen

1. **Fresco**: sí — dispatch nuevo, brifeado solo con deliverable + criterio.
2. **Verificación primaria**: sí — todos los oráculos re-corridos arriba, con recomputo independiente del hit@5.
3. **Disenso**: sección anterior.
4. **Verdict-artifact versionado**: este fichero, `evals/e1-read/verdict/gate-m2-06.md`, commiteado en la rama `m2-06` antes de cualquier `GATE-EXEC`.

**MERGED.** El merge a main lo ejecuta el orquestador (GATE-EXEC); este verdict no mergea nada.
