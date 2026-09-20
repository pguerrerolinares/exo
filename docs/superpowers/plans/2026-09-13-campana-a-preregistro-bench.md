# Pre-registro del bench de la campaña A: lo que cuesta el recall por prompt

> **Esto es un pre-registro, y lo que lo respalda está en `git`, no en lo que
> diga el texto.** Se escribe el 2026-09-13, antes de que exista el generador
> de KB sintética (`engine/examples/kb_sintetica.rs`, Task 1 del plan
> `2026-09-13-campana-a-recall-por-prompt.md`) y antes de haber corrido una
> sola medición sobre corpus sintético. Para que valga, **este fichero tiene
> que estar commiteado antes de que exista `evals/recall-coste/results/`**.
> Si el orquestador lo commitea después de la Task 2, deja de ser un
> pre-registro y hay que titularlo como en G4a: *«Registro del bench — NO es
> un pre-registro»*.
>
> Lo que sí se ha medido ya está en §«Lo ya medido»: son medidas del terreno,
> hechas sobre la KB real y una copia de su índice. Explican por qué la
> campaña existe, pero no deciden ningún criterio.

## Referencia

- **Base de código:** `main` en `3c1918f`. El binario de la línea base sale de
  `cargo build --release` en la rama de la campaña, **en un commit donde
  `git diff 3c1918f -- engine/src plugins/exo/scripts` salga vacío**. Las
  Tasks 1 y 2 solo añaden `engine/examples/` y `evals/recall-coste/`, así que
  se cumple. `bench.sh` deja apuntado ese diff en `entorno.txt`.
- **Máquina:** el Linux de Paul (12 hilos, NVMe con ext4 tanto en `/tmp`
  como en `~`, kernel 7.0.0-31, hyperfine 1.20.0, jq 1.7, git 2.43.0).
  `/tmp` y `~/.exo` están en el mismo disco, así que los fsync que se midan
  son de verdad.
- **Condiciones:** ninguna otra sesión de fábrica ni ningún `cargo` corriendo
  a la vez. Con `uptime` por encima de 1,0 en la carga del primer minuto al
  empezar, la corrida no se juzga y se repite. Motivo: el 2026-09-13, con
  otras dos sesiones de planificación en marcha, `exo search --type hybrid`
  midió 1,8 s en una tanda y 0,96 s en la siguiente.
- **Índice real:** no se toca. Esta campaña no usa `~/.exo/index.db` para
  nada, ni siquiera en solo lectura. El bench trabaja únicamente sobre corpus
  sintético.

## El corpus

Lo genera `kb_sintetica <N> <DIR> 42`. La forma de cada nota copia la de la
KB real `wisdom-paul`, medida el 2026-09-13 sobre una copia de su índice:

| Medida real (2026-09-13) | Valor | Parámetro del generador |
|---|---|---|
| notas | 174 | `N` |
| trozos | 3.290, 617 caracteres de media → 18,9 por nota | 19 párrafos de ≥580 caracteres. Dos juntos pasan de 900, así que sale un trozo por párrafo |
| aristas | 727 → 4,18 por nota | 4 wikilinks por nota |
| aristas sin resolver | 24/727 ≈ 1/30 | 1 de cada 30 enlaces apunta a `inexistente-*` |
| tiers (primer `tier:` de cada `.md`) | 6 core · 60 stable · 105 log | `i % 29`: 0 → core, 1–10 → stable, resto → log (3,4% / 34,5% / 62,1%) |

- **Tamaños:** `N ∈ {174, 1000, 5000}`.
- **Vectores:** unitarios, pseudoaleatorios (xorshift64 con la semilla) y sin
  pasar por el modelo. Este bench mide **coste, no calidad**. Con vectores
  aleatorios la similitud con la query ronda cero, y ninguna fila pasa el
  umbral 0,40. Por eso cada escenario de query tiene su variante con
  `--min-similarity 0.0`, en la que todo trozo entra en la agregación: es la
  cota superior de coste.
- **Git:** un único commit con fecha `1780000000 +0000`, y ese mismo valor en
  `notas.git_epoch`.
- **Fidelidad, comprobada antes de medir:** `exo index` sobre el corpus
  recién generado tiene que dar `indexed == 0 && skipped == N`. Si no sale
  así, el mtime o el cuerpo sintético no coinciden con lo que calcula el
  indexer. En ese caso el bench aborta, porque estaría midiendo un reindexado.
- **Query fija:** `trinquete techos indice memoria`. Todas sus palabras están
  en el vocabulario del generador. FTS es un AND implícito: con una palabra de
  fuera del corpus, como «como» o «de», la query daba «recall vacío» (exit 1) y
  confundía P1. Medido en el smoke del plan el 2026-09-13 con N=58.

## Escenarios

Todos corren con `hyperfine --warmup 3 --runs 20 --ignore-failure`, con el
shell por defecto y `EXO_CONFIG` apuntando a la config sintética. Cada uno
tiene además una corrida directa que guarda `rc` y `stderr`. El comando
literal vive en `evals/recall-coste/harness/bench.sh`, y lo que sigue es su
contenido:

| id | Comando (abreviado: `$B`=binario, `$DB`, `$KB`) | Hallazgo |
|---|---|---|
| `s1-query-refresh` | `$B recall --db $DB --kb $KB --query=$Q --min-similarity 0.40 --limit 4 --cap-bytes 4000 --refresh --json`, con los flags exactos de `recall-inject.sh:143-146` | H3, H2 |
| `s1b-query-refresh-sim0` | lo mismo con `--min-similarity 0.0` | H17b (cota) |
| `s2-query` | `s1` sin `--refresh` | H3 (separa el refresh) |
| `s3-index-sin-cambios` | `$B index --db $DB --kb $KB --json` | H4, H29 |
| `s4-arranque-content` | `$B recall --db $DB --kb $KB --content --note sint/core/core-index --limit 10 --cap-bytes 6144`, los flags de `exo-recall.sh:86-87` | H17a |
| `s5-search-fts` | `$B search --db $DB --type fts --limit 5 --json "trinquete techos"` | suelo: arranque de proceso más apertura de DB |
| `s6-hook-entero` | `recall-inject.sh < prompt.json`, sin `EXO_KB_NAME`, así que llama a `exo config` de verdad | H3, H10 |
| `s7-config-jq` | `$B config --json \| jq -r .data.kb.name` | H10 |
| `s8-kb-precommit` | `kb-precommit.sh` con cwd en `$KB` y una nota staged | H23 |
| `s9-git-log-una-nota` | `git -C $KB log -1 --format=%ct -- log/nota-00011.md` | H17 (`git_epoch_de` por nota) |
| `s10-jq-log-l<L>` | el filtro de `exo-recall.sh:111-112` sobre un log sintético de `L ∈ {2174, 20000, 200000}` líneas de ~410 B, que es la media real (892.617 B / 2.174 líneas) | H5 |

- **p50** = mediana de `results[0].times`.
- **p95** = elemento `floor(0,95·(n−1))` de `times` ordenado.
- **Corridas fallidas** = cuántos `exit_codes` distintos de 0 hay.

## Predicciones declaradas antes de medir

Son falsables. Si una no se cumple, se anota como fallida y **no se reescribe**.

- **P1 (H27):** en la línea base, `s1`, `s1b` y `s2` salen con exit 1 las 20
  veces para `N=1000` y `N=5000`, y su `stderr` contiene `k value in knn query
  too large`. Con `N=174`, en cambio, salen con exit 0. El motivo es que
  174·19 = 3.306 trozos quedan por debajo del tope de 4.096 y 1000·19 = 19.000
  quedan por encima.
- **P2 (H3):** en la línea base, el p50 de `s2` con `N=174` cae entre 800 y
  1.300 ms. Sobre la KB real hoy da entre 960 y 1.020 ms, y la carga del
  modelo ONNX domina.
- **P3:** en la línea base, el p50 de `s1` menos el de `s2` con `N=174` es
  menor de 60 ms. Sobre la KB real el refresh sin cambios cuesta unos 15 ms.
- **P4 (H10):** el p50 de `s7` en Linux es menor de 20 ms. Sobre el binario
  real, `exo config --json` sin jq tarda de 5 a 6 ms.
- Para `s3`, `s4`, `s8`, `s9` y `s10` con `N=5000` **no hay predicción**: se
  miden. Declararlo evita escribir una predicción a posteriori.

## Criterios de decisión (fijados ahora)

| id | Criterio | Qué decide |
|---|---|---|
| **C-H27** | Tras la Task 3, `s1`, `s1b` y `s2` salen con exit 0 en 20/20 corridas para `N ∈ {1000, 5000}` | PASA/FALLA de la Task 3 |
| **C-H4** | Tras la Task 6, el p50 de `s3` con `N=5000` es ≤ 0,6 × el de la línea base. Si la línea base ya daba ≤ 50 ms, se declara «sin margen» y el criterio queda vacío | PASA/FALLA/VACÍO de la Task 6 |
| **C-H17a** | La Task 10 **solo se ejecuta** si el p95 de `s4` con `N=5000` en la línea base es > 250 ms. Si se ejecuta, pasa cuando ese p95 queda ≤ 0,5 × el de la línea base | puerta y PASA de la Task 10 |
| **C-H17b** | Tras la Task 3, si el p50 de `s2` con `N=5000` menos el p50 de `s2` con `N=174` es > 250 ms, el número se entrega a la campaña C o al backlog (KNN o índice particionado). En A **no se arregla** | derivación |
| **C-H23** | Si el p50 de `s8` con `N=5000` es > 2.000 ms, se abre un item en el backlog. En A no se arregla | derivación |
| **C-H10** | La Task 12 **solo se ejecuta** si el p50 de `s7` en Linux es > 30 ms, o si la medición manual en W11 (§W11) da un p50 > 100 ms | puerta de la Task 12 |
| **C-H5** | `s10` no decide nada por sí solo: sus números van a la decisión D3 de Paul | informe |
| **C-noregresión** | Tras la campaña, el p50 de `s5` y de `s3` para cada `N` queda ≤ 1,2 × el de la línea base + 2 ms. Los 2 ms absolutos cubren la resolución: `resumen.tsv` trunca a milisegundos enteros y `s5` ronda los 5 ms | FALLA de la campaña si se incumple |

Los criterios los aplica `evals/recall-coste/harness/compara.sh`, que se
escribe en la Task 1, y no se redactan a mano. La campaña **no inventa
umbrales a la vista del resultado**. Una diferencia
que ningún criterio cubra se anota como observación y no decide nada.

## Criterio de reapertura del proceso residente (H3)

La spec `2026-08-22-m6-06-recall-punto-de-uso-design.md` (§2.2, líneas
169-173) aceptó «el segundo por turno» con este criterio de reapertura: *«si el
segundo por turno duele tras semanas de uso, ESA es la palanca»*. Faltaba el
mecanismo que detecte ese dolor, y lo pone la Task 13
(`plugins/exo/scripts/recall-latencia.sh`), que lee los eventos de producción
y no el bench.

**REABRIR** cuando, en una ventana de 14 días y con al menos **200** disparos
(eventos `recall-inject-emitted` más `recall-inject-degraded
reason=timeout-guard`) fuera de sesiones `test*`, se cumpla una de estas dos:

- el p95 de `elapsed_ms + refresh_ms` sobre los `emitted` es **> 1.500 ms**;
- los timeouts suponen **> 2%** de los disparos.

Los valores 1.500 ms, 2% y 200 son una **propuesta** que queda pendiente de
Paul (D4 del plan). Si los cambia al gatear, se sustituyen aquí y en las tres
constantes de la Task 13 antes de ejecutarla. La ventana se evalúa **por
máquina**: el log vive en el `$HOME` de cada una, y W11 no se mezcla con Linux.

**Enmienda (2026-09-19, decisión de Paul #12, campaña I):** el criterio de
p95 pasa de `elapsed_ms + refresh_ms` (tiempo interno del engine, el que
reporta el propio `exo recall`) a **`hook_ms`** (reloj de pared del hook
`recall-inject.sh` entero, medido con `$EPOCHREALTIME`, sin spawn). Motivo:
en W11 el shell alrededor del binario cuesta tanto como el binario mismo
(`evals/recall-coste/results/w11-2026-09-15.txt`: hook p50 2.312 ms frente a
`elapsed_ms+refresh_ms` ~993-1.003 ms en las tres muestras registradas) —
con la métrica vieja, el criterio de 1.500 ms nunca se dispara en W11 aunque
cada prompt cueste el doble. El umbral (1.500 ms), el porcentaje de
timeouts (2%) y el mínimo de disparos (200) **no cambian**: la enmienda es
solo de qué mide el reloj, no de dónde está la barrera. Implementado en
`docs/superpowers/plans/2026-09-19-campana-i-latencia-hook-w11.md` (Task 1).
No se reescribe el texto de arriba: esto es un anexo fechado, como pide el
propio contrato de pre-registro de la cabecera de este fichero.

## W11 (manual, sujeto a D5)

El bench de arriba es solo de Linux. En la W11 de Paul, desde Git Bash, con
una copia del índice y de la config de esa máquina:

```bash
TMP="$(mktemp -d)"; cp ~/.exo/index.db "$TMP/index.db"
jq -n '{prompt:"como funciona el trinquete de techos", session_id:"bench-w11"}' > "$TMP/prompt.json"
H="${EXO_PLUGIN_SCRIPTS:?exporta EXO_PLUGIN_SCRIPTS con la ruta a plugins/exo/scripts del plugin instalado}"
for i in $(seq 20); do
  s=$(date +%s%N)
  EXO_INDEX="$TMP/index.db" REFLEX_LOG_FILE="$TMP/log.jsonl" "$H/recall-inject.sh" < "$TMP/prompt.json" > /dev/null
  e=$(date +%s%N); echo $(( (e - s) / 1000000 ))
done | sort -n | awk '{a[NR]=$1} END{print "p50_ms", a[int((NR-1)*0.5)+1], "p95_ms", a[int((NR-1)*0.95)+1]}'
for i in $(seq 20); do
  s=$(date +%s%N); exo config --json | jq -r .data.kb.name > /dev/null; e=$(date +%s%N)
  echo $(( (e - s) / 1000000 ))
done | sort -n | awk '{a[NR]=$1} END{print "config_jq_p50_ms", a[int((NR-1)*0.5)+1]}'
jq -r 'select(.reflex=="recall-inject-emitted") | .payload' "$TMP/log.jsonl" | tail -3
rm -rf "$TMP"
```

`date +%s%N` funciona en Git Bash (GNU date) y no en macOS, pero este bloque
es solo para W11. Hay que registrar la salida entera y la versión de
`exo --version` en el veredicto de la Task 14.

## Lo ya medido (terreno, 2026-09-13, no decide criterios)

Todo se midió sobre `~/.local/bin/exo` (0.1.0) y una **copia** del índice
real, en `scratchpad/bench/index.db`:

| Qué | Comando | Resultado |
|---|---|---|
| `exo index` sin cambios | `exo index --db <copia> --kb wisdom-paul --json` ×4 | 12 / 20 / 19 / 21 ms |
| syscalls de ese index | `strace -f -c -e trace=fsync,fdatasync,pwrite64,openat` | **0 fsync**, 8 `pwrite64`, 314 `openat` (276 dentro de `.git/`) |
| recall por prompt, flags del hook | `exo recall --query=… --min-similarity 0.40 --limit 4 --cap-bytes 4000 [--refresh] --json` | con refresh: 987–1000 ms; sin refresh: 982–1021 ms |
| hybrid, tiempo interno | `exo search --type hybrid … --json \| jq .data.elapsed_s` | 0,943–0,966 s sobre 0,970–0,994 s de reloj |
| suelo de proceso | `exo search --type fts … --json` | 5–8 ms (`elapsed_s` 0,004 s) |
| config | `exo config --json` | 5–6 ms |
| arranque del hook de SessionStart | `exo recall --content --note wisdom-paul/core/core-index --limit 10 --cap-bytes 6144` | 14–17 ms |
| log de reflejos | `wc`, `ls -la ~/.claude/reflex-log.jsonl` | 892.617 B, 2.174 líneas desde 2026-06-26 (~11 KB/día) |
| jq de compactación | `jq -r 'select(.session_id=="x") \| .reflex' ~/.claude/reflex-log.jsonl` | 17–20 ms |
| tope de KNN | programa aparte con rusqlite 0.40.1 y sqlite-vec =0.1.9, 4.100 vectores | `k=4096` ok; `k=4097` → `k value in knn query too large, provided 4097 and the limit is 4096`; barrido `vec_distance_l2` de 4.100 filas en 30 ms |
| escala de la distancia de vec0 | mismo programa, rowid 1 | `vec_distance_l2 = 0.962646` = L2 (L2² = 0.926687) |

## Lo que este pre-registro NO cubre

- **La calidad de retrieval:** hit@5, fusión, RRF, chunking y la escala de
  similitud (H28). Todo eso es de la campaña C. Si C cambia el troceado, el
  generador produce otro número de trozos, y la línea base de A deja de ser
  comparable con cualquier corrida posterior a ese merge. La corrida
  «después» de A (Task 14) **tiene que hacerse antes** de que entre en `main`
  un cambio de C a `trozos.rs`.
- **El coste de un `exo rebuild` real a 5.000 notas.** Exige embeber unos
  95.000 trozos, del orden de horas a 0,25 s por trozo. `s9` da el coste de
  `git log` por nota, y el total se extrapola por N en el veredicto, avisando
  de que es una extrapolación.
