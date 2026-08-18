# M6-04 — kbx al índice del engine: diseño

> **Régimen de esta spec:** diseño dialogado con Paul (2026-08-18), sobre recon
> primario contra los dos índices vivos. Revisado por **consultor fable:
> FIRMA-CON-CAMBIOS** (`consultas/2026-08-18-m6-04-kbx/consultor-m6-04.md`).
> Los seis claims del recon sobrevivieron a su verificación independiente; dos
> llevaban un error que esta spec ya incorpora corregido, y el gate de §5 está
> reformulado por hallazgo suyo. Lo que sigue es la síntesis ejecutable.

**Goal:** que `kbx` deje de leer el índice de basic-memory y lea el de exo, para
que **M5b** (desinstalar basic-memory) pueda ejecutarse sin que seis comandos
mueran en silencio.

**Plan madre:** `plans/2026-08-17-cierre-exo-m2-a-m5b.md` (M6-04, tabla §M6) ·
**Spec de régimen:** `specs/2026-08-18-cierre-en-regimen-design.md` §orden (M6-04
en 2º lugar) · **Salió de C9** por estar mal dimensionado:
`plans/2026-08-18-c9-m6-completo.md` §"Qué NO entra en este plan".

---

## 1. El camino: portar las queries. Las vistas de compatibilidad están muertas

El plan de C9 dejó escrito que había "dos caminos válidos con coste distinto":
vistas de compatibilidad en el índice de exo, o portar las queries de kbx. **No
son dos.** FTS5 no expone `snippet()` ni `ORDER BY rank` a través de una `VIEW`,
y `targets` depende de los dos además de `MATCH` (`targets.go:82-89`).

Medido (fts5 en memoria + `CREATE VIEW v AS SELECT * FROM f`):

```
SELECT * FROM v WHERE v MATCH 'x'   →  OperationalError: no such column: v
snippet(v, …)                       →  idem
```

**Matiz del consultor, que no rescata el camino:** el *column-MATCH*
(`WHERE cuerpo MATCH 'x'`) **sí** atraviesa la view — el planner aplana y empuja
el constraint a la fts5 subyacente. Pero `snippet()` y `ORDER BY rank` siguen
muertos, y reescribir las queries para usar column-MATCH **ya es portarlas**.
Añadido: una view no puede fabricar los `id` numéricos del JOIN
`entity.id = search_index.id`.

Decisión: **se portan las queries**. No se crean vistas.

## 2. Qué cambia en exo

Dos cambios, ambos aditivos. Ninguno fuerza rebuild del índice (restricción de
`nota.rs:14`).

### 2.1 Tabla `meta` — procedencia, no config

```sql
CREATE TABLE IF NOT EXISTS meta (clave TEXT PRIMARY KEY, valor TEXT NOT NULL);
```

El indexer escribe `kb_root` = raíz absoluta con la que se construyó el índice.
Sustituye al `SELECT path FROM project ORDER BY id LIMIT 1` de
`internal/index/db.go:35`.

**Por qué no colisiona con la config propia de exo (C10):** responde a una
pregunta distinta. `meta.kb_root` es **procedencia** — "de qué KB salió este
índice", y lo escribe el único componente que lo sabe, el indexer. La config de
C10 dirá **qué KB usar**. Para kbx la buena es la procedencia: quiere la raíz que
casa con el índice que está leyendo, no la que esté configurada. Por eso M6-04 no
depende de C10 y el orden de campañas no se invierte.

Se conserva el kill-criterion K2 de `doctor`: si la ruta no resuelve en disco,
kbx exige `--kb` explícito. **No adivina.**

### 2.2 `journal_mode = WAL`

`PRAGMA journal_mode=WAL` persistente en `abre_db`. El índice de exo está hoy en
`delete`; basic-memory ya usaba WAL.

**La razón NO es la que parecía.** El brief afirmaba que kbx podía comerse un
`SQLITE_BUSY` desde `kb-precommit.sh` y tumbar un commit. **Falso**, y verificado
por el consultor: ese hook solo llama `ratchet --kb` y `budget --kb`, y por ese
camino la DB **no se abre nunca** (`budget.go:62-75`; `ratchet`/`rotate` no tienen
ni flag `--db`).

El riesgo real va en la dirección contraria y ya se materializó una vez:
`targets` y `doctor` mantienen el cursor `rows` abierto mientras hacen `ReadFile`
y shellean git **por fila** (`targets.go:118-146`, `doctor.go:170-190`). Eso es un
lock de lectura sostenido; en journal `delete` bloquea al escritor, y el
`busy_timeout` de 5 s del indexer (`lib.rs:51`) no cubre a un lector largo. El
indexer del hook de Stop ya registró un `database is locked` real.

Un `busy_timeout` en kbx **no** habría protegido esa dirección. WAL elimina la
clase entera en ambos sentidos.

**Trade-off aceptado:** aparecen ficheros `-wal`/`-shm` junto a `~/.exo/index.db`.
Hoy nada asume fichero único en ese path (el índice es derivado y regenerable; el
rollback del runbook C9 copia el binario, no el índice). Es un supuesto a
re-chequear si algún día se hace backup del índice.

**Descartado:** tratar `SQLITE_BUSY` como skip-soft. Es tolerar fallo silencioso,
que es la clase de bug que ha costado caro dos veces en este proyecto.

## 3. Qué cambia en kbx

### 3.1 Mapa de queries

Seis sitios abren la DB. Tres queries sustantivas, dos lookups 1:1, una de
arranque.

| Hoy (basic-memory) | Después (exo) |
|---|---|
| `SELECT path FROM project ORDER BY id LIMIT 1` (`db.go:35`) | `SELECT valor FROM meta WHERE clave='kb_root'` |
| `search_index JOIN entity ON entity.id = search_index.id` (`targets.go:87`) | `notas_fts JOIN notas ON notas.permalink = notas_fts.permalink` |
| `snippet(search_index, 3, …)` (`targets.go:85`) | `snippet(notas_fts, 1, …)` — `cuerpo` es la columna 1 |
| `COUNT(relation WHERE from_id/to_id = entity.id)` (`stale.go:143-144`) | `COUNT(aristas WHERE origen/destino_permalink = n.permalink)` |
| `entity.file_path` · `entity.permalink` | `notas.ruta` · `notas.permalink` |
| `SELECT COALESCE(permalink,'') FROM entity WHERE file_path = ?` (`diffsince.go:180`) | `SELECT permalink FROM notas WHERE ruta = ?` |
| `SELECT file_path FROM entity WHERE permalink = ?` (`history.go:139`) | `SELECT ruta FROM notas WHERE permalink = ?` |
| default `~/.basic-memory/memory.db` (`db.go:44-58`, `ResolvePath`) | default `~/.exo/index.db` |

**Ese último default es el interruptor real del cutover.**

Se caen por sí solos, sin equivalente necesario:

- **`project_id`** — no hay multi-KB en exo.
- **`observation`** — declarada en `consumed` y **no la consulta nadie** (grep
  confirmado: solo la tocan los fixtures).
- **`entity.size`** y **`tier`** — no necesitan columna en el índice. `targets` ya
  abre cada candidata (`targets.go:136`, `ExtractHeadings`), así que salen del
  mismo `open`+`stat`. Precedente interno más fuerte de lo que parecía: `budget`
  con `--kb` explícito **ni siquiera abre la DB** y lee tier de
  `internal/frontmatter`.

**Se conserva la dedup por `ruta` de `targets`** aunque hoy sobre: `notas_fts` es
1:1 con `notas` (138/138), pero cuesta cero y sigue siendo correcta el día que el
FTS pase a indexar por `trozos`.

### 3.2 Gotcha duro: `NOT IN` + NULL apaga el check de huérfanas

La query de `doctor` usa `NOT IN`. Con aristas sin resolver
(`destino_permalink IS NULL`) la subconsulta evalúa a NULL y `NOT IN` **no
devuelve TRUE para ninguna fila**. Medido contra el índice vivo, que tiene **23 de
573 aristas sin resolver**:

```
sin guardia IS NOT NULL  →  0 huérfanas   (verde, silencioso, check muerto)
con guardia              →  7 huérfanas   (correcto)
```

El código actual ya se protege (`doctor.go:166`, `WHERE to_id IS NOT NULL`). **El
port debe llevarse la guardia.** Es el mismo shape que el bug del indexer de C9:
no falla, se apaga.

### 3.3 El canary sobrevive, y cambia de propósito

Nueva lista `consumed` en `internal/index/schema.go`:

```go
{"notas",     []string{"permalink","ruta","titulo","tipo","mtime","git_epoch"}},
{"aristas",   []string{"origen","destino_texto","destino_permalink"}},
{"notas_fts", []string{"titulo","cuerpo","permalink"}},
{"meta",      []string{"clave","valor"}},
```

`trozos` y `vectores` **no entran**: kbx no los consulta, y el principio "canary
sobre el subconjunto consumido, no sobre el schema entero" se mantiene.
Verificado que `PRAGMA table_info` funciona sobre tablas fts5 (devuelve
`titulo, cuerpo, permalink`).

Hoy el canary vigila que un schema **externo** no derive. Después vigila que
**kbx y exo no se desincronicen** — dos binarios, dos repos, ciclos de release
independientes, instalados por separado en `~/.local/bin/`. Esa clase de fallo
está demostrada, no es teórica: el runbook de C9 documenta `~/.local/bin/exo`
corriendo 20 h por detrás del fix de su propia campaña sin que nada avisara.

Su caso de disparo inmediato es concreto: **kbx nuevo contra un índice generado
por un binario exo viejo, sin `meta`** → canary rojo accionable en el primer
`/consolida`, en vez de un error críptico de kbRoot.

**Descartado `meta.schema_version`.** Un `PRAGMA` mira la realidad; un número de
versión es una declaración que alguien tiene que acordarse de bumpear. Es
sustituir una comprobación por un acto de fe.

### 3.4 Fixtures: DDL a mano, suite hermética

`internal/fixtures/index.go` construye hoy una DB con DDL "verbatim from the live
index" de basic-memory. Pasa a reflejar el DDL de `engine/src/schema.rs`, copiado a
mano y marcado con su fuente.

**No** se invoca el binario `exo` desde los tests de Go. El mecanismo que detecta
la divergencia fixture-vs-real **ya existe y corre en producción**: el canary de
`doctor`, que `/consolida` ejecuta y que falla-fuerte ante `schema_drift` por
diseño. Acoplar `go test` a un binario Rust instalado rompería la hermeticidad de
la suite por un riesgo que el runtime ya cubre.

**Trade-off aceptado:** una divergencia se detecta en el primer `/consolida`
post-release (runtime, ruidoso) y no en CI (temprano). Días de retraso a cambio de
una suite que corre sola.

## 4. El filtro `tipo='note'`: el hallazgo que cambia el peso del item

`targets`, `stale` y `doctor` filtran por `note_type='note'`. Los tres comentarios
que lo justifican (`targets.go:70`, `stale.go:136`, `doctor.go:156`) dicen lo
mismo: *"excluye non-note entities (assets pdf/tex/cls/json que el índice real
también trackea)"*.

**El filtro no hace eso.** Reparto real de `notas.tipo` en el índice de exo:

```
note 81 · report 39 · project 10 · research 3 · guide 3 · spec 1 · person 1  =  138
```

Y cuadra al dígito con basic-memory: 143 entities = 138 notas + 5 assets `file`.
Los tres comandos ven **81 de 138**. Lo que queda fuera no son assets:

- los **10 destilados de proyecto** (`projects/*.md`, `type: project`) — justo los
  que `core-index` lista como activos;
- **`Paul - perfil de trabajo.md`** (`type: person`);
- los **39 `report`** (`archive/sesiones/*`).

`kbx targets <topic> --json`, que `core-index` nombra como herramienta de búsqueda
de memoria, no puede devolver ni los destilados de proyecto ni el perfil. El
consultor añade el refuerzo que lo cierra: **`note_type` en basic-memory ES el
`type:` del frontmatter**, con reparto idéntico — o sea, el filtro ya oculta esas
57 notas **hoy**, contra basic-memory. No es una regresión que introduzca el port.

En el índice de exo el propósito original del filtro ni siquiera aplica: el walker
solo indexa `.md` (`walker.rs:35`), así que los assets no existen. Portarlo literal
sería portar el efecto colateral sin la causa.

**Decisión: se porta conservando el filtro, y se quita en un commit propio (T3).**
Separa dos preguntas que merecen dos gates: *¿el port es correcto?* (paridad contra
basic-memory) y *¿el alcance cambia?* (delta inspeccionable).

### 4.1 El delta de T3 es doble, no uno

Hallazgo del consultor, y es el que más caro sale descubrir tarde. Las 57 notas no
aparecen por igual en todos los comandos:

| Comando | Delta esperado en T3 | Por qué |
|---|---|---|
| `targets` | **+57** (81 → 138) | no excluye por directorio |
| `doctor`, `stale` | **+23** | excluyen `archive`/`.superpowers`/`docs`; de las 57, solo 23 sobreviven a la exclusión (11 en `projects/`, 9 en `research/`, 2 en `learnings/`, 1 en raíz) |

El mensaje del commit de T3 **debe decir los dos números**. Si pre-registra "delta
57" y el inspector mide `doctor`, verá 23 y concluirá que el commit está mal.

## 5. Gates: qué es falsable y qué no

La paridad no aplica igual a los tres comandos, y venderla entera sería vender un
gate que no puede fallar por las razones buenas.

| Comando | Gate | Por qué no da para más |
|---|---|---|
| `doctor` huérfanas | **Paridad exacta sobre el conjunto CRUDO de la query**: 7 = 7, los mismos 7 ficheros | Ya medido: intersección completa, cero en cada diferencia |
| `stale` | **Solo el grado-0** (mismo conjunto que doctor). El ranking por grado, no | exo extrae 573 aristas, basic-memory 674: la extracción de links difiere |
| `targets` | **Overlap de conjunto en top-N sobre 3-5 topics, con criterio escrito ANTES de correr nada** | Paridad de ranking inalcanzable, ver abajo |

### 5.1 Por qué el gate de `doctor` se mide sobre la query cruda

Corrección del consultor a la formulación original, y es material. El 7=7 es de la
query cruda. Pasado por el pipeline de `doctor` —exclusión por directorio +
waiver `kbx_orphan_ok`— el report neto es **0 findings · 2 waived en ambos lados**:
5 de las 7 caen por dir excluido y las otras 2 (`README.md`, `metodologia.md`)
están waived.

Un gate sobre el JSON final pasaría **igual con un port roto que devolviera
conjuntos vacíos**. Es exactamente el anti-patrón que C9 dejó escrito en su
runbook (§"Paso 3 — Verificación falsable"): un check que no distingue éxito de
no-op no es un check. La paridad se mide sobre el conjunto crudo, o como mínimo
sobre `findings + waived`.

### 5.2 Por qué `targets` no puede tener paridad de ranking

Dos causas medidas, **no tres**: el brief afirmaba también "tokenizers distintos" y
es **falso** — el DDL real de basic-memory usa `unicode61 tokenchars 0x2F`, el
mismo que exo. Las que sí sostienen la conclusión:

- **Columnas FTS distintas**: `title, content_stems` (stemming hecho por el
  pipeline Python de basic-memory), `content_snippet`, `permalink` indexado, más
  `prefix='1,2,3,4'` — contra `titulo, cuerpo` crudos en exo.
- **Multiplicidad distinta**: basic-memory tiene **160 filas `type='entity'` para
  143 entities**; exo es 1:1 (138/138).

bm25 sobre corpus con columnas, stems y multiplicidad distintos no puede dar el
mismo orden.

### 5.3 El pre-registro de `targets`

Ni harness de overlap medido (es una métrica nueva, y §0 del régimen las retiró),
ni "a ojo" (un gate sin criterio no puede fallar). El punto medio cuesta diez
minutos: **3-5 topics y su criterio escritos antes de compilar**, del tipo *"≥3 del
top-5 de basic-memory aparecen en el top-5 de exo; cada miss se explica por
stems/duplicación o se investiga"*.

El corpus ya está congelado de facto — ambos índices existen sobre la misma KB en
el mismo instante. Congelar más es teatro.

**Riesgo operativo:** el pre-registro caduca al primer vistazo. Una vez visto el
output del binario nuevo ya no hay pre-registro posible. Se escribe **antes de
compilar**.

## 6. Orden de ejecución

```
T0  exo: journal_mode=WAL
T1  exo: tabla meta + poblarla con kb_root
    └─ NO cierra con el merge: cierra con binario reconstruido, instalado,
       `exo index` corrido y `SELECT valor FROM meta WHERE clave='kb_root'`
       devolviendo algo. (El gotcha de las 20 h ya ocurrió una vez.)
T2  kbx: 6 queries + canary + fixtures, CONSERVANDO tipo='note'
    └─ gate: doctor crudo 7=7, mismos ficheros, contra basic-memory
T3  kbx: quitar el filtro           (commit propio)
    └─ gate: delta +57 en targets, +23 en doctor/stale — los DOS en el commit
T4  call sites + default de KBX_DB → ~/.exo/index.db
    └─ antes: cp ~/.local/bin/kbx /tmp/kbx-rollback
```

**Rollback: el binario `kbx` anterior.** No hace falta modo dual. Para el gate de
paridad basta correr **binario-viejo-sobre-basic-memory** contra
**binario-nuevo-sobre-exo**: compara los dos mundos sin que ningún binario hable
dos dialectos. Una ventana bilingüe violaría además la propiedad "no se sabe contra
qué DB corre" y no compra nada.

`kbx --version` probablemente no distingue builds, igual que `exo --version` está
fijo en `0.1.0` (runbook C9). La red es la copia del binario, no la versión.

**Trade-off aceptado:** el rollback revierte el port entero, sin grano fino. En un
proyecto personal con la KB en git y el índice regenerable, aceptable.

## 7. Riesgos anotados

- **`entity.size` está rancio en 17 de 138 notas** (medido; p.ej.
  `research/2026-06-12-estado-del-arte…`: db 5958 vs disco 7206). Tras el port,
  `size` sale de `stat` y será **más correcto que basic-memory**. Queda escrito
  aquí para que una comparación campo-a-campo en el gate no le cargue la culpa al
  port.
- **El segundo bucle del indexer gana un consumidor.** `indexer.rs:161-179` borra
  `notas_fts` → `aristas` → `trozos` → `notas` en autocommit por statement (deuda
  ya anotada en el runbook de C9, 4 líneas). Con kbx leyendo en concurrencia, un
  `doctor` puede ver una nota borrada a medias —fila en `notas`, aristas ya
  borradas— y reportar un **`orphan` transitorio falso**. WAL reduce la ventana
  (snapshot por query) pero `doctor` hace varias queries. Este item no arregla esa
  deuda; le añade una razón.
- **`vectores` (vec0) no rompe a kbx**, verificado: las tablas que kbx consulta se
  leen sin el módulo registrado. Guardia para el plan: que `doctor` **no añada
  jamás** un scan global (`integrity_check`, iterar `sqlite_master` ejecutando) o
  ese día kbx muere con `no such module: vec0`.
- **kbx y `schema.rs` viven en repos distintos** y el canary es el único puente.
  Barato: un comentario en `schema.rs` señalando que kbx consume
  `notas`/`aristas`/`notas_fts`/`meta`.
- **WAL, primer arranque**: confirmar con los binarios reales (no solo con python)
  que el hook de Stop, `exo search/recall` y kbx en `mode=ro` conviven.

## 8. Verificado limpio — que nadie lo re-audite

- **`history` y `diffsince`**: las convenciones de path y permalink son
  **idénticas** entre los dos índices. Set-equality medida sobre las 138 notas:
  `notas.ruta` == `entity.file_path` y `notas.permalink` == `entity.permalink`,
  cero diferencias. Son lookups 1:1 y el port es mecánico.
- **`budget`, `ratchet`, `rotate`**: fuera de alcance, y más desacoplados de lo que
  el recon inicial suponía. `ratchet` y `rotate` no tienen ni flag `--db`; `budget`
  con `--kb` explícito no abre la DB.

## 9. Lo que esta spec NO cierra

- **M6-06 (recall en el punto de uso)** — sigue con su propio ciclo de diseño,
  fuera desde `cierre-en-regimen-design.md` §3.2. **M5b sigue gated** hasta que
  M6-04 y M6-06 estén hechos; este item cierra uno de los dos.
- **La transaccionalidad del segundo bucle del indexer** — deuda del runbook de
  C9, no de este item. Aquí solo queda anotado que gana un consumidor (§7).
- **La config propia de exo (C10)** — `meta.kb_root` es procedencia y no la
  sustituye (§2.1).
