# Verdict del consultor — M6-04: kbx al índice del engine (2026-08-18)

> Consultor fable fresco. Verificación primaria propia sobre los dos índices
> (`~/.exo/index.db`, `~/.basic-memory/memory.db`, ambos abiertos `mode=ro` vía
> `python3`+sqlite3), el código de kbx (`~/Documentos/proyectos/kbx`), el engine
> y la KB en disco. No he editado nada: solo este fichero.

## 1. Veredicto global: **FIRMA-CON-CAMBIOS**

El diseño es correcto en lo grueso: port de queries (no vistas), `meta` como
procedencia, filtro en segundo commit, canary conservado, cutover por default
de `KBX_DB` con rollback por binario. Los 6 claims sobreviven — pero dos llevan
matices que corrigen premisas: la causa "tokenizers distintos" de C4 es falsa
(ambos FTS usan `unicode61 tokenchars 0x2F`, la conclusión aguanta por las otras
causas), y la premisa de D2 ("el pre-commit puede comerse un `SQLITE_BUSY` y
tumbar un commit") **cae**: el camino pre-commit no abre la DB en absoluto. La
adjudicación de D2 (WAL) sobrevive igualmente, por una razón mejor: proteger al
*indexer* de los lectores. Y el gate de D1 necesita una corrección para ser
falsable: la paridad de `doctor` medida sobre el report neto sería 0=0 — un
check que no distingue éxito de no-op, exactamente el aprendizaje que C9 dejó
escrito en su runbook.

## 2. Verificación de C1–C6

### C1 — Vistas de compatibilidad muertas → **SOBREVIVE CON MATIZ**

Método: fts5 en memoria + `CREATE VIEW v AS SELECT * FROM f`, cuatro queries.
Reproducido exacto: `SELECT * FROM v WHERE v MATCH 'x'` →
`OperationalError: no such column: v`, y `snippet(v,…)` idem. **Matiz**: el
column-MATCH (`WHERE cuerpo MATCH 'x'`) SÍ atraviesa la view — el planner
aplana y empuja el constraint a la fts5 subyacente. No rescata el camino:
`snippet()` y `ORDER BY rank` siguen muertos a través de la view, y kbx usa
los tres (`targets.go:82-89`: `search_index MATCH ?` + `snippet(search_index,3,…)`
+ `ORDER BY rank`). Además las views no pueden fabricar los `id` numéricos del
JOIN `entity.id = search_index.id`. Camino muerto: confirmado.

### C2 — El filtro oculta 57 de 138 → **SOBREVIVE** (y es peor de lo que dice)

Método: `GROUP BY tipo` sobre `notas` en exo y `note_type` sobre `entity` en bm.
Reparto exacto al claim: note 81 · report 39 · project 10 · research 3 · guide 3
· spec 1 · person 1 = 138; bm 143 = 138 + 5 `file`. Los 10 `projects/*.md` son
`type: project` y el perfil es `person`, confirmado. Walker de exo: solo `.md`
(`walker.rs:35`). Refuerzo que el claim no dice: **el reparto de `note_type` en
bm es idéntico al de exo** — `note_type` para markdown ES el type del
frontmatter, así que el filtro ya oculta esas 57 notas HOY contra basic-memory.
Los tres comentarios del código (`targets.go:70`, `stale.go:136`, `doctor.go:156`,
"excluye assets pdf/tex/cls/json") describen mal lo que el filtro hace incluso
en el mundo viejo. P3 sale reforzado: quitar el filtro es un cambio de alcance
real y deseable, no una regresión del port.

### C3 — Paridad 7=7, mismos ficheros → **SOBREVIVE** (con un matiz que corrige D1)

Método: query de `doctor.go:158-166` literal contra bm, y su equivalente con
guardia `IS NOT NULL` contra exo; comparación de conjuntos. **7 = 7, los mismos
7 ficheros, intersección completa** (README.md, metodologia.md, 2× archive/research,
2× docs/superpowers, 1× docs/superpowers/verdicts). Matiz material: eso es la
query **cruda**. Tras el pipeline de doctor (exclusión `archive/.superpowers/docs`
+ waiver `kbx_orphan_ok`), el report neto es **findings 0 · waived 2** en ambos
lados: 5 de las 7 caen por dir excluido y las otras 2 (README, metodologia)
están waived. Consecuencia para D1 abajo.

### C4 — Paridad de ranking de `targets` inalcanzable → **SOBREVIVE CON MATIZ**

La conclusión aguanta; una de las tres causas citadas es falsa. Verificado en el
DDL real de bm (`sqlite_master`): `tokenize='unicode61 tokenchars 0x2F'` — **el
mismo tokenizer que exo**, no distinto (el fixture de kbx ya lo decía verbatim,
`fixtures/index.go:218`). Las causas que sí sostienen el claim, medidas:
columnas FTS distintas (`title, content_stems` — stemming hecho por el pipeline
Python de bm — `content_snippet`, `permalink` indexado, más `prefix='1,2,3,4'`
vs `titulo, cuerpo` crudos), y duplicación de filas: **160 filas `type='entity'`
para 143 entities** en bm; exo es 1:1 (138 filas fts / 138 notas). bm25 sobre
corpus con columnas, stems y multiplicidad distintos no puede dar el mismo
orden. No se lleva ninguna decisión: D1 estaba construida sobre la conclusión,
no sobre la pata del tokenizer.

### C5 — `tier` y `size` sin columna nueva → **SOBREVIVE**

Verificado en código: `targets` ya abre cada candidata (`targets.go:137`
`ExtractHeadings(filepath.Join(kbRoot, filePath))`) y shellea git por fichero —
tier y `stat` salen del mismo open. Precedente confirmado y más fuerte de lo que
dice el claim: `budget` con `--kb` explícito **ni siquiera abre la DB**
(`budget.go:62-75`, `resolveBudgetKBRoot`) y lee tier de disco vía
`internal/frontmatter`; `ratchet` y `rotate` no tienen ni flag `--db` (grep en
`cmd/kbx/ratchet.go`, `rotate.go`: cero hits de `index.Open`). Bonus medido: el
`size` de stat será **más correcto** que `entity.size` — ver hallazgo D6-1.

### C6 — Trampa `NOT IN` + NULL activa → **SOBREVIVE**

Medido contra `~/.exo/index.db`: 573 aristas, **23 con `destino_permalink IS
NULL`**. Query de huérfanas sin la guardia: **0**; con `IS NOT NULL`: **7**.
Exacto al claim. El port de la query de doctor debe llevar la guardia igual que
la lleva hoy contra `relation.to_id` (`doctor.go:163`).

**Ningún claim cae**, así que P1–P3 quedan en pie. Los matices de C3 y C4
corrigen la formulación de D1 y una pata argumental, no las decisiones.

## 3. Adjudicación de D1–D5

### D1 — Gate asimétrico

**Adjudicación**: firma el gate asimétrico, pero define la paridad de `doctor`
sobre el conjunto crudo de la query (7=7, mismos ficheros — o como mínimo
findings+waived), y para `targets` pre-registra topics y criterio por escrito
ANTES de la primera corrida; nada de "a ojo" sin criterio.

**Racional**: el report neto de doctor es hoy **0 findings / 2 waived en ambos
lados** (medido, §C3): un gate sobre el JSON final pasaría igual con un port
roto que devolviera conjuntos vacíos — no distingue éxito de no-op, el
anti-patrón que el runbook de C9 documenta explícitamente (§"Paso 3 —
Verificación falsable"). Para `targets`, la paridad de ranking es inalcanzable
(C4) y un harness de overlap medido es una métrica nueva que §0 retiró — pero
"a ojo" sin pre-registro es un gate que no puede fallar. El punto medio cuesta
10 minutos: 3-5 topics escritos con criterio (p.ej. "≥3 del top-5 de bm en el
top-5 de exo; cada miss se explica por stems/duplicación o se investiga") antes
de correr nada. El corpus ya está congelado de facto: ambos índices existen
sobre la misma KB en el mismo instante — congelar más es teatro. `stale`
grado-0: paridad ya demostrada (los mismos 7 degree-0); grados exactos
incomparables (573 aristas vs 674 relations, medido).

**Trade-off aceptado**: un topic puede fallar el criterio por diferencia
legítima de stemming y obligar a una segunda pasada de explicación — es el
precio de que el gate pueda fallar.

### D2 — `journal_mode`

**Adjudicación**: pasa el índice de exo a WAL (a), y corrige la premisa del
brief: el pre-commit no está en riesgo — quien está en riesgo es el indexer.

**Racional**: la premisa "kbx corre desde kb-precommit.sh y puede tumbar un
commit" **cae con verificación**: `kb-precommit.sh` solo llama `ratchet --kb` y
`budget --kb` (líneas 23 y 33), y por ese camino **la DB no se abre nunca**
(`budget.go:62-75`; ratchet/rotate no tienen `--db`). El riesgo real es el
inverso y ya se materializó: `targets` y `doctor` mantienen `rows` abiertos
mientras hacen `ReadFile`/git-shell por fila (`targets.go:118-146`,
`doctor.go:170-190`) — lock de lectura sostenido en journal `delete` — y el
indexer del Stop hook ya se comió un `database is locked` real (runbook C9 /
verdict anterior, `exo-index.log`); su `busy_timeout` de 5 s (`lib.rs:51`) no
cubre un lector largo. `busy_timeout` en kbx (b) no protege esa dirección. WAL
elimina la clase entera en ambos sentidos y lo verifiqué empírico: lector
`mode=ro` funciona con escritor vivo y tras cierre limpio, y el escritor
committea con un cursor lector abierto. No hay tercera vía que valga: tratar
BUSY como skip-soft es tolerar fallo silencioso (anti-doctrina) y correr sobre
snapshot es over-engineering. Es un `PRAGMA journal_mode=WAL` persistente en
`abre_db` — aditivo, sin rebuild, compatible con la restricción 3.

**Trade-off aceptado**: aparecen `-wal`/`-shm` junto a `~/.exo/index.db` — hoy
nada copia ni asume fichero único ese path (el rollback del runbook copia el
binario; el índice es derivado y regenerable), pero es un supuesto a re-chequear
si algún día se hace backup del índice.

### D3 — Canary

**Adjudicación**: firma el canary repuntado con `consumed` = `notas`, `aristas`,
`notas_fts`, `meta`; descarta `meta.schema_version`.

**Racional**: el canary verifica la realidad (`PRAGMA table_info`, verificado
que funciona sobre fts5: devuelve `titulo, cuerpo, permalink`); un
`schema_version` es una declaración que alguien tiene que acordarse de bumpear —
fallo humano silencioso, justo el enemigo que la doctrina nombra. Y su caso de
disparo inmediato no es teórico: kbx nuevo contra un índice generado por un
binario exo viejo **sin `meta`** → canary rojo accionable en el primer
`/consolida` (que falla-fuerte a `schema_drift` por diseño, SKILL.md:86), en
vez de un error críptico de kbRoot. La clase "dos binarios con release
independiente" está demostrada: 20 h de binario desincronizado en el runbook.
Caída de `observation`: correcta — grep confirma que solo fixtures la tocan.
`trozos`/`vectores` fuera: correcto, kbx no los consulta.

**Trade-off aceptado**: mientras rija la restricción aditiva el canary casi
nunca disparará — son ~30 líneas ya escritas de cinturón barato, no un gate que
se espere ver rojo a menudo.

### D4 — Fixtures

**Adjudicación**: DDL copiado a mano desde `schema.rs`, hermético; sin binario
Rust en la suite Go.

**Racional**: el mecanismo que detecta la divergencia silenciosa ya existe y es
el canary **en runtime contra el índice real**: `doctor` corre en cada
`/consolida` y falla-fuerte a `schema_drift` — una divergencia fixture-vs-real
que los tests no vean la ataja producción en días, loud. El DDL de exo son 45
líneas selladas por la restricción aditiva y marcadas VERBATIM con su spec
(`schema.rs:4-8`) — exactamente el mismo contrato que el fixture actual ya
mantiene con bm (`fixtures/index.go:13-14`, "verbatim from the live index") y
que no ha fallado. Acoplar `go test` a un binario Rust instalado rompe la
hermeticidad de la suite por un riesgo que el runtime ya cubre.

**Trade-off aceptado**: una divergencia se detecta en el primer `/consolida`
post-release (runtime, loud), no en CI (early) — detección tardía en días a
cambio de suite hermética.

### D5 — Orden y cutover

**Adjudicación**: firma T1→T2→T3→T4 y el rollback por binario, sin modo dual;
añade a T1 el cierre del gotcha del binario y a T4 una copia del kbx viejo.

**Racional**: el modo dual es exactamente el estado que la restricción 4
prohíbe ("no saber contra qué DB corre") y el gate de paridad no lo necesita:
binario-viejo-sobre-bm vs binario-nuevo-sobre-exo compara los dos mundos sin
que ningún binario hable dos dialectos. Dos precisiones operativas: (1) `meta`
solo existe cuando el binario exo **nuevo** corre `exo index` — y el runbook
demuestra que el binario instalado puede ir 20 h por detrás; T1 debe cerrar con
rebuild+install+corrida verificada (`SELECT valor FROM meta WHERE
clave='kb_root'` no vacío), no con el merge. (2) `kbx --version` seguramente no
distingue builds (como `exo --version`, runbook §"Ya hecho"): antes de T4,
`cp ~/.local/bin/kbx /tmp/kbx-rollback` — la misma red que usó C9.

**Trade-off aceptado**: rollback por binario revierte el port entero (no hay
grano fino); en un proyecto personal con la KB en git y el índice regenerable,
aceptable.

## 4. D6 — Hallazgos nuevos (por coste de descubrirlos tarde)

1. **`entity.size` está rancio en 17 de 138 notas** (medido contra disco; p.ej.
   `research/2026-06-12-estado-del-arte…` db=5958 vs disco=7206). Tras el port,
   `size_bytes` sale de `stat` y será **más correcto que bm**. Si nadie lo deja
   escrito, cualquier comparación campo-a-campo de `targets` en el gate
   mostrará diffs y el port cargará con la culpa. Refuerza el gate de conjunto
   (D1); el plan debe anotar "size difiere por diseño: bm está desactualizado".
2. **El delta de T3 no es 81→138 en todos los comandos.** `doctor` y `stale`
   excluyen por dir (`archive/.superpowers/docs`): de las 57 notas que el
   filtro oculta, solo **23** son visibles tras exclusiones (11 en `projects/`,
   9 en `research/`, 2 en `learnings/`, 1 en raíz — medido). `targets` no
   excluye por dir y sí ve las 57. Si T3 pre-registra "delta 57" contra
   `doctor`, el inspector verá 23 y dudará del commit equivocado.
3. **La paridad neta de doctor es 0=0** — todas las huérfanas de hoy están
   excluidas (5) o waived (2). Gate sobre findings = gate que no puede fallar.
   (Detalle en C3/D1; lo listo aquí porque descubrirlo tarde convierte el gate
   en ceremonia sin que nadie lo note.)
4. **El segundo bucle del indexer sigue sin transacción** (`indexer.rs:161-177`,
   deuda ya anotada en el runbook): borra `notas_fts` → `aristas` → trozos →
   `notas` en autocommit por statement. Con kbx como lector concurrente nuevo,
   un `doctor` puede ver una nota borrada-a-medias (sin aristas de origen
   todavía con fila en `notas`) → finding `orphan` transitorio falso. WAL
   reduce la ventana (snapshot por query) pero doctor hace varias queries. Son
   las 4 líneas ya presupuestadas — este item les añade un consumidor.
5. **`history`/`diffsince`: verificado limpio, que nadie lo re-audite.** Las dos
   queries que faltaban en el recuento (`history.go:139`, `diffsince.go:180`)
   son lookups 1:1 y la convención de path/permalink es **idéntica** entre
   índices (set-equality medida: `notas.ruta` == `entity.file_path` y
   `notas.permalink` == `entity.permalink` para las 138, cero diferencias).
6. **La tabla `vectores` (vec0) no rompe a kbx**: toda mi verificación corrió
   con un sqlite3 sin vec0 registrado y `notas`/`aristas`/`notas_fts` se
   consultan sin error — el módulo solo falta si una query toca `vectores`, y
   ninguna de kbx lo hace. Guardia para el plan: que doctor no añada jamás un
   scan global (`integrity_check`, iterar `sqlite_master` ejecutando) o ese día
   kbx muere con `no such module: vec0`.

## 5. Riesgos que el orquestador debe vigilar

- **T1 no está cerrado hasta que `meta` esté poblada en `~/.exo/index.db`** con
  el binario reinstalado — el gotcha de las 20 h es la familia de fallo más
  probable de esta campaña, ya ocurrió una vez.
- **El pre-registro de `targets` caduca al primer vistazo**: una vez corrido el
  binario nuevo y visto el output, ya no hay pre-registro posible. Escribir
  topics y criterio antes de compilar siquiera.
- **WAL: primer arranque y supuestos de fichero único** — verificar tras el
  PRAGMA que el hook de Stop, `exo search/recall` y kbx ro conviven (el
  experimento dice que sí; confirmarlo con los binarios reales, no solo python).
- **Comunicar el doble delta de T3 (57 en targets, 23 en doctor/stale)** en el
  mensaje del commit, o la inspección de P3 concluirá lo contrario de lo que ve.
- **kbx y `schema.rs` viven en repos distintos** y el canary es el único puente.
  Opcional barato: un comentario en `schema.rs` ("kbx consume notas/aristas/
  notas_fts/meta — tocar esto mira kbx") para el yo-futuro.

## 6. Disenso: qué busqué para objetar y qué refuté

- **Intenté resucitar las vistas de compatibilidad** (tumbar C1) con el
  column-MATCH que sí atraviesa views — funciona, pero no rescata `snippet()`,
  `ORDER BY rank` ni los JOINs por id: reescribir las queries para
  column-MATCH ya ES portarlas. Muerto se queda.
- **Tumbé la premisa de D2**: el pre-commit no abre la DB (verificado en
  `kb-precommit.sh` + `budget.go:62` + ausencia de `--db` en ratchet/rotate).
  La adjudicación WAL sobrevive por la dirección opuesta (lector largo vs
  indexer), que está demostrada con un `database is locked` real.
- **Tumbé una pata de C4**: el tokenizer NO difiere (DDL real de bm leído). La
  conclusión aguanta por columnas/stems/duplicación, todas medidas.
- **Busqué una razón para el modo dual en D5** (¿y si el gate necesita los dos
  dialectos en un binario?): refutado — el gate compara procesos distintos, y
  la ventana bilingüe viola la restricción 4 sin comprar nada.
- **Consideré exigir el harness de overlap medido en D1**: lo rebajé a
  pre-registro manual barato. Un harness es una métrica nueva (§0) para un gate
  que corre una vez; pero "a ojo" sin criterio escrito es un no-gate. El punto
  medio no es tibieza: es la versión mínima falsable.
- **Consideré objetar P2** (¿`meta` invade la config de C10?): refutado — la
  distinción procedencia/config es real: `kb_root` lo escribe el indexer, que
  es el único que sabe de qué KB salió el índice; la config de C10 dirá qué KB
  usar, no de cuál vino. No colisionan.
- En P1 y D4 no encontré nada que objetar más allá de lo dicho: el recuento de
  6 queries es correcto (las localicé las 6 más el canary) y budget/ratchet/
  rotate están aún más desacoplados de la DB de lo que P1 afirma.

## 7. Preguntas que SÍ requieren a Paul

Ninguna. Nada de este item toca línea roja: no hay acción destructiva (el
cutover es un default con rollback por binario, basic-memory no se toca hasta
M5b, que sigue gated), no hay superficie externa, y las decisiones de agenda ya
las tomó Paul en el brainstorm (P1–P3). El cambio a WAL es la única pieza que
toca el engine además de kbx, y entra dentro del alcance que P1 ya asume para
`meta`; si Paul prefiere verlo como tarea propia (T0) es orden de plan, no
pregunta bloqueante.
