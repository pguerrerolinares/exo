# Gate de paridad del port de `kbx` — informe (M6-04 Task 9)

Fecha de ejecución: 2026-08-19.

Contexto: novena tarea de la campaña M6-04 (port de `kbx` del índice SQLite
de basic-memory al índice del engine `exo`). Las ocho tareas anteriores
dejaron el port completo y `make check` verde (12/12 paquetes). Esta tarea
retira el fixture de basic-memory (sin consumidores) y corre los tres gates
que contrastan el binario nuevo contra el viejo sobre los índices reales.

El binario "viejo" sale de la rama `main` de kbx (no existe `master` en este
repo). El "nuevo" sale de la rama `m6-04` de kbx, sobre el índice de exo
producido por la rama `m6-04` del engine (con WAL + tabla `meta`, Tasks 1-3).
Ningún índice vivo se tocó: `~/.exo/index.db` y `~/.basic-memory/memory.db`
se copiaron a `/tmp/m6-04/` y todo el trabajo se hizo sobre las copias.

**Nota posterior — esta evidencia es de una configuración superada.** Los
tres gates de este informe (`doctor`, `stale`, `targets`) se corrieron con el
filtro `note_type='note'`/`tipo='note'` todavía activo en las tres queries de
kbx, tal como lo fija el pre-registro de Task 1. La Task 10, que corrió
**después** de este informe, retiró ese filtro de las tres queries
(`internal/targets/targets.go`, `internal/stale/stale.go`,
`internal/doctor/doctor.go`) para que kbx vea la KB entera. Eso cambia lo que
`doctor`/`stale` ven: **23 findings más** de los que este informe midió (57
notas más para `targets`, de las cuales 23 caen fuera de los directorios que
`doctor`/`stale` recorrían) — cifras exactas en
`docs/superpowers/runbooks/2026-08-18-m6-04-cutover.md` (Paso 4) y en el
detalle completo de la medición en `.superpowers/sdd/m604-task-10-report.md`.
Un lector que solo lea este informe concluiría "kbx doctor nuevo == kbx
doctor viejo, 7 huérfanos en los dos"; eso ya no es cierto tras Task 10 — es
falso por esos 23 findings. Los tres veredictos "PASA" de más abajo siguen
siendo válidos como gate de paridad del port (compararon exactamente lo que
el pre-registro pedía comparar en el momento en que se pre-registró), pero no
son evidencia de paridad de comportamiento post-Task 10.

---

## Paso 1-3: retiro del fixture de basic-memory

**Step 1 — consumidores del fixture viejo:**

```
$ grep -rn "BuildIndex(\|BuildIndexExtraChunk(\|BuildIndexMissingColumn(" --include='*.go' /home/paul/Documentos/proyectos/kbx
internal/fixtures/index_test.go:21:	db := openRW(t, BuildIndex(t))
internal/fixtures/index_test.go:139:	db := openRW(t, BuildIndexMissingColumn(t))
internal/fixtures/index.go:158:func BuildIndex(t *testing.T) string {
internal/fixtures/index.go:270:func BuildIndexExtraChunk(t *testing.T) string {
internal/fixtures/index.go:272:	path := BuildIndex(t)
internal/fixtures/index.go:290:func BuildIndexMissingColumn(t *testing.T) string {
internal/fixtures/index.go:292:	path := BuildIndex(t)
```

Solo hits en las propias definiciones (`index.go`) y en sus tests
(`index_test.go`). Ningún otro paquete consume el fixture viejo — no hizo
falta migrar nada antes de borrar.

**Step 2 — borrado.** De `internal/fixtures/index.go` se borraron: las
constantes `ddlProject`, `ddlEntity`, `ddlObservation`, `ddlRelation`,
`ddlSearchIndex`; el tipo `entityMetadata`; y las funciones `BuildIndex`,
`BuildIndexExtraChunk`, `BuildIndexMissingColumn`. Los imports `encoding/json`
y `fmt` quedaron sin uso tras el borrado y se quitaron. Se conservó `Notes`,
`fixtureRels`, el tipo `rel`, `KBPath` y todo `buildExo`/`BuildExo*`.

De `internal/fixtures/index_test.go` se borraron `TestBuildIndexIsCoherentWithNotes`
y `TestBuildIndexMissingColumnDropsConsumedColumn` (referenciaban las
funciones borradas). El helper `openRW`, que solo servía a esos dos tests, se
borró también por quedar muerto — no lo pedía el brief explícitamente pero no
tenía ningún otro consumidor. Los cuatro tests de `BuildExoIndex*` quedaron
intactos.

**Step 3 — suite:**

```
$ cd /home/paul/Documentos/proyectos/kbx && make check
/home/paul/.local/go/bin/go build -tags sqlite_fts5 ./...
/home/paul/.local/go/bin/go vet -tags sqlite_fts5 ./...
/home/paul/.local/go/bin/go test -tags sqlite_fts5 ./...
ok  	github.com/pguerrerolinares/kbx/cmd/kbx	10.422s
ok  	github.com/pguerrerolinares/kbx/internal/budget	(cached)
ok  	github.com/pguerrerolinares/kbx/internal/doctor	5.196s
ok  	github.com/pguerrerolinares/kbx/internal/envelope	(cached)
ok  	github.com/pguerrerolinares/kbx/internal/fixtures	1.100s
ok  	github.com/pguerrerolinares/kbx/internal/frontmatter	(cached)
ok  	github.com/pguerrerolinares/kbx/internal/gitx	0.301s
ok  	github.com/pguerrerolinares/kbx/internal/index	1.555s
ok  	github.com/pguerrerolinares/kbx/internal/ratchet	(cached)
ok  	github.com/pguerrerolinares/kbx/internal/rotate	(cached)
ok  	github.com/pguerrerolinares/kbx/internal/stale	2.047s
ok  	github.com/pguerrerolinares/kbx/internal/targets	3.499s
```

VERDE, 12/12 paquetes.

---

## Paso 4-5: preparación de la comparación

**Step 4 — dos binarios:**

```
$ git -C /home/paul/Documentos/proyectos/kbx worktree add /tmp/m6-04/kbx-viejo main
Preparando árbol de trabajo (haciendo checkout a 'main')
HEAD está ahora en e307612 merge: kbx doctor — check budget_prose_drift (F3.4)

$ cd /tmp/m6-04/kbx-viejo && go build -tags sqlite_fts5 -o /tmp/m6-04/kbx-viejo-bin ./cmd/kbx
$ cd /home/paul/Documentos/proyectos/kbx && go build -tags sqlite_fts5 -o /tmp/m6-04/kbx-nuevo-bin ./cmd/kbx
$ ls -l /tmp/m6-04/kbx-viejo-bin /tmp/m6-04/kbx-nuevo-bin
-rwxrwxr-x 1 paul paul 9167928 ago 19 01:19 /tmp/m6-04/kbx-nuevo-bin
-rwxrwxr-x 1 paul paul 9163232 ago 19 01:19 /tmp/m6-04/kbx-viejo-bin
```

Los dos binarios existen.

**Step 5 — copias RO y reindexado de la copia de exo:**

```
$ cp ~/.basic-memory/memory.db /tmp/m6-04/bm.db
$ cp ~/.exo/index.db /tmp/m6-04/exo.db
$ cd /home/paul/Documentos/proyectos/exo/engine && cargo run --release -- index \
  --kb /home/paul/Documentos/proyectos/kb-demo --db /tmp/m6-04/exo.db --json
    Finished `release` profile [optimized] target(s) in 0.11s
     Running `target/release/exo index --kb /home/paul/Documentos/proyectos/kb-demo --db /tmp/m6-04/exo.db --json`
{"command":"index","data":{"borradas":0,"indexadas":0,"saltadas":138,"trozos_embebidos":0,"trozos_reusados":0},"schema_version":1}
```

Envelope JSON válido (0 indexadas/borradas, 138 saltadas: la copia ya estaba
al día respecto al índice vivo). Verificación de `meta` y WAL:

```
$ python3 -c "
import sqlite3
c = sqlite3.connect('file:/tmp/m6-04/exo.db?mode=ro', uri=True)
print(list(c.execute('SELECT clave, valor FROM meta')))
print('journal_mode:', c.execute('PRAGMA journal_mode').fetchone())
"
[('kb_root', '/home/paul/Documentos/proyectos/kb-demo')]
journal_mode: ('wal',)
```

`meta` poblada con `kb_root`, journal en `wal`. Los índices vivos
(`~/.exo/index.db`, `~/.basic-memory/memory.db`) nunca se tocaron; todo el
trabajo posterior lee las copias en `/tmp/m6-04/`.

---

## Step 6: Gate de `doctor` — paridad exacta sobre el conjunto crudo

Medido sobre la query cruda (huérfanos `note_type='note'`/`tipo='note'`), no
sobre el JSON final de `doctor`, porque el report neto (`0 findings ·
2 waived`) pasaría igual con un port roto que devolviera conjuntos vacíos.

```
$ python3 - <<'PY'
import sqlite3
bm = sqlite3.connect('file:/tmp/m6-04/bm.db?mode=ro', uri=True)
exo = sqlite3.connect('file:/tmp/m6-04/exo.db?mode=ro', uri=True)
b = set(r[0] for r in bm.execute(
  "SELECT file_path FROM entity WHERE id NOT IN (SELECT from_id FROM relation) "
  "AND id NOT IN (SELECT to_id FROM relation WHERE to_id IS NOT NULL) AND note_type='note'"))
e = set(r[0] for r in exo.execute(
  "SELECT ruta FROM notas WHERE permalink NOT IN (SELECT origen FROM aristas) "
  "AND permalink NOT IN (SELECT destino_permalink FROM aristas WHERE destino_permalink IS NOT NULL) "
  "AND tipo='note'"))
print("basic-memory:", len(b), "| exo:", len(e))
print("solo bm :", sorted(b - e))
print("solo exo:", sorted(e - b))
print("GATE:", "PASA" if b == e else "FALLA")
PY
basic-memory: 7 | exo: 7
solo bm : []
solo exo: []
GATE: PASA
```

**Veredicto: PASA.** Conjuntos idénticos (7 = 7), sin diferencias en ninguna
dirección. Coincide exactamente con el `Expected` del brief.

---

## Step 7: Gate de `stale` — paridad del grado-0

```
$ /tmp/m6-04/kbx-viejo-bin stale --db /tmp/m6-04/bm.db --kb /home/paul/Documentos/proyectos/kb-demo --json > /tmp/m6-04/stale-viejo.json
$ /tmp/m6-04/kbx-nuevo-bin stale --db /tmp/m6-04/exo.db --kb /home/paul/Documentos/proyectos/kb-demo --json > /tmp/m6-04/stale-nuevo.json
```

Ambos comandos salieron con exit 0 sin stderr.

**Desviación del script literal del brief:** el brief asume `json.load(open(p))["notes"]`
en el nivel raíz. Los dos binarios (viejo y nuevo, verificado en ambos)
envuelven la salida en el envelope estándar `{schema_version, command, data}`
— `notes` vive en `data.notes`, simétrico en ambos lados. Es un ajuste
mecánico de acceso al JSON, no un cambio de semántica de la comparación.

```
$ python3 - <<'PY'
import json
def grado0(p):
    return set(n["path"] for n in json.load(open(p))["data"]["notes"] if n["degree"] == 0)
v, n = grado0('/tmp/m6-04/stale-viejo.json'), grado0('/tmp/m6-04/stale-nuevo.json')
print("grado-0 viejo:", len(v), "| nuevo:", len(n))
print("solo viejo:", sorted(v - n))
print("solo nuevo:", sorted(n - v))
print("GATE:", "PASA" if v == n else "FALLA")
PY
grado-0 viejo: 2 | nuevo: 2
solo viejo: []
solo nuevo: []
GATE: PASA
```

**Veredicto: PASA.** Conjuntos de grado-0 idénticos (2 = 2), sin diferencias.
El ranking por grado no se comparó (por diseño: exo extrae 573 aristas,
basic-memory 674). Esa diferencia de extracción, ~15%, se decidió aceptar sin
cerrarla en este port: el spec (§5, `2026-08-18-m6-04-kbx-al-indice-design.md`)
la documentó **antes** de escribir código, porque el criterio de paridad de
M6-04 es estructural (que `doctor`/`stale` vean el mismo conjunto de
huérfanos y de grado-0), no un conteo de aristas byte a byte — cerrar esa
brecha exigiría reimplementar la semántica de extracción de enlaces de
basic-memory, que está fuera del alcance de portar `kbx` a un índice que ya
existe. No es un bug tapado; es un límite de alcance escrito antes de medir.

---

## Step 8: Gate de `targets` — pre-registro de Task 1

Pre-registro aplicado tal como está escrito
(`docs/superpowers/plans/2026-08-18-m6-04-preregistro-targets.md`): 5 topics,
top-5 candidatos por topic, PASA un topic si ≥3/5 permalinks del top-5 de
basic-memory aparecen en el top-5 de exo; el gate global pasa si pasan 4 de
los 5 topics.

**Misma desviación mecánica que en Step 7:** `candidates` vive en
`data.candidates`, no en la raíz; se ajustó el acceso del script del brief
sin tocar la semántica.

Output literal de los cinco topics (permalinks del top-5, orden tal como los
devuelve cada binario):

```
=== indexer ===
--- bm (viejo) ---
kb-demo/log/code-graph-engine-bitacora
kb-demo/log/exo-bitacora
kb-demo/archive/log/backlog-diario-2026-06-26_2026-07-11
kb-demo/projects/code-graph-engine/grafo-estructural-propio-proveedor-del-que-reemplazo-de-codebase-memory-mcp
kb-demo/projects/exo-framework-unificado-de-trabajo-agentico
--- exo (nuevo) ---
kb-demo/archive/log/backlog-diario-2026-06-26_2026-07-11
kb-demo/log/code-graph-engine-bitacora
kb-demo/log/exo-bitacora
kb-demo/projects/exo-framework-unificado-de-trabajo-agentico
kb-demo/projects/code-graph-engine/grafo-estructural-propio-proveedor-del-que-reemplazo-de-codebase-memory-mcp

=== reflex ===
--- bm (viejo) ---
kb-demo/archive/log/agent-develop-bitacora-2026-06-26_2026-07-11
kb-demo/log/reflex-bitacora
kb-demo/core/core-index
kb-demo/docs/superpowers/plans/2026-07-03-memoria-v2
kb-demo/projects/exo-framework-unificado-de-trabajo-agentico
--- exo (nuevo) ---
kb-demo/archive/log/agent-develop-bitacora-2026-06-26_2026-07-11
kb-demo/log/reflex-bitacora
kb-demo/docs/superpowers/plans/2026-07-03-memoria-v2
kb-demo/archive/research/2026-08-03-terreno-1-hooks-deterministas-horkos-vector
kb-demo/log/agent-develop-bitacora

=== memoria ===
--- bm (viejo) ---
kb-demo/docs/superpowers/plans/2026-07-03-memoria-v2-verificacion
kb-demo/docs/superpowers/plans/2026-07-03-memoria-v2
kb-demo/core/core-index
kb-demo/docs/superpowers/specs/2026-07-03-memoria-v2-design
kb-demo/log/ocr-ml-docs-bitacora
--- exo (nuevo) ---
kb-demo/docs/superpowers/plans/2026-07-03-memoria-v2-verificacion
kb-demo/docs/superpowers/plans/2026-07-03-memoria-v2
kb-demo/core/core-index
kb-demo/docs/superpowers/specs/2026-07-03-memoria-v2-design
kb-demo/log/ocr-ml-docs-bitacora

=== kbx ===
--- bm (viejo) ---
kb-demo/log/kbx-bitacora
kb-demo/projects/kbx-explorador-determinista-de-la-kb-go
kb-demo/core/core-index
kb-demo/log/backlog-diario
kb-demo/projects/exo-framework-unificado-de-trabajo-agentico
--- exo (nuevo) ---
kb-demo/log/kbx-bitacora
kb-demo/projects/kbx-explorador-determinista-de-la-kb-go
kb-demo/log/backlog-diario
kb-demo/core/core-index
kb-demo/projects/exo-framework-unificado-de-trabajo-agentico

=== recall en el punto de uso ===
--- bm (viejo) ---
kb-demo/backlog-frentes-abiertos
kb-demo/archive/log/desarrollo-agentico-bitacora-2026-06-23_2026-07-09
kb-demo/archive/log/cge-bitacora-2026-07-04_2026-07-11
kb-demo/log/exo-bitacora
kb-demo/archive/research/2026-08-03-terreno-3-replay-determinista-observabilidad
--- exo (nuevo) ---
kb-demo/backlog-frentes-abiertos
kb-demo/archive/log/desarrollo-agentico-bitacora-2026-06-23_2026-07-09
kb-demo/log/exo-bitacora
kb-demo/archive/log/cge-bitacora-2026-07-04_2026-07-11
kb-demo/archive/research/2026-08-03-terreno-3-replay-determinista-observabilidad
```

### Overlap por topic

| topic | overlap top-5 | veredicto | ausencias |
|---|---|---|---|
| indexer | 5/5 | PASA | ninguna |
| reflex | 3/5 | PASA | `core/core-index`, `projects/exo-framework-unificado-de-trabajo-agentico` |
| memoria | 5/5 | PASA | ninguna (mismo orden incluso) |
| kbx | 5/5 | PASA | ninguna |
| recall en el punto de uso | 5/5 | PASA | ninguna |

**Explicación de las dos ausencias (topic `reflex`):** no es un fallo del
port. Se investigó bajando a top-15 en ambos binarios:

```
$ (top-15 "reflex", bm vs exo — resumen de rangos)
                                                        bm   exo
core/core-index                                        #3   #8
projects/exo-framework-unificado-de-trabajo-agentico   #5   #9
archive/research/2026-08-03-terreno-1-hooks-...        #10  #4
log/agent-develop-bitacora                             #8   #5
```

Ambos documentos siguen presentes en el índice de exo (no desaparecieron:
están en el top-15, a un par de puestos del top-5), pero dos documentos que
en basic-memory rankeaban más abajo (`terreno-1-hooks-deterministas-horkos-vector`,
`agent-develop-bitacora` sin archivar) suben por delante de ellos en exo. La
causa es la señal de relevancia FTS, que el pre-registro nombra
explícitamente como fuente de diferencia esperada: `content_stems` en
basic-memory lleva stemming del pipeline Python, mientras que `cuerpo` en exo
es texto crudo (mismo tokenizer `unicode61 tokenchars 0x2F` en ambos, pero
distinta normalización previa), y la multiplicidad de filas también difiere
(160 filas `type='entity'` para 143 entities en basic-memory frente a 1:1 en
exo). Esa diferencia de bm25 basta para reordenar candidatos con relevancia
parecida y empujar dos de ellos justo fuera del corte top-5. `size_bytes` de
`core-index` difiere en 26 bytes (4747 en bm vs 4721 en exo, el real de
`stat`) pero esa diferencia es demasiado pequeña para ser la causa principal
del salto de 5 puestos — la causa dominante es la de stemming/multiplicidad
de filas ya prevista por el pre-registro. `targets` no compara orden ni
score (pre-registro, sección "Qué NO mide este gate"), así que esta
reordenación no es en sí misma una señal de bug; lo único que el gate exige
es que el conjunto top-5 mantenga ≥3/5 de solape, y lo mantiene (3/5).

### Veredicto global

**5 de 5 topics pasan** (≥4/5 requerido) → **GATE: PASA**.

---

## Veredicto global del informe

| Gate | Resultado |
|---|---|
| `doctor` (huérfanos crudos) | PASA — 7 = 7, sin diferencias |
| `stale` (grado-0) | PASA — 2 = 2, sin diferencias |
| `targets` (pre-registro Task 1) | PASA — 5/5 topics (umbral: 4/5) |

**PASA global.** El port de `kbx` al índice de exo reproduce, sobre los
índices reales, el conjunto de huérfanos de `doctor`, el conjunto de
grado-0 de `stale`, y el top-5 de `targets` con el margen de solape exigido
por el pre-registro en los cinco topics, incluyendo el único topic con
diferencias (`reflex`), cuyas dos ausencias quedan explicadas por causas que
el pre-registro admitía de antemano (relevancia FTS: stemming +
multiplicidad de filas), no por un defecto del port.
