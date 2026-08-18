# Brief — Consultor Fable: diseño de M6-04 (kbx al índice del engine exo)

## Rol

Eres el consultor Fable del régimen de gates delegado del proyecto exo. Paul ha
abierto el brainstorm de **M6-04**, el item que la campaña C9 dejó fuera por estar
mal dimensionado en la spec, y que **bloquea M5b** (desinstalar basic-memory).

Tu deliverable es un **veredicto escrito con adjudicación FIRMADA por decisión**, no
un menú de opciones: elige, razona corto, deja el trade-off explícito. Y antes de
adjudicar, **verifica por tu cuenta los 6 claims de §Claims**: están sacados de un
recon hecho hoy contra la KB viva, y si alguno cae, se lleva por delante decisiones
que Paul ya ha tomado encima de él. Verificación primaria propia — no me creas.

No edites la spec, no commitees, no toques código. Solo el veredicto.

## Contexto obligatorio (léelo, no lo asumas)

Repo exo: `/home/paul/Documentos/proyectos/exo` (branch `main`, limpio, C9 mergeada).
Repo kbx: `/home/paul/Documentos/proyectos/kbx` (Go, ~10.5k LOC).
KB viva: `/home/paul/Documentos/proyectos/kb-demo` (138 notas markdown).

- `docs/superpowers/plans/2026-08-18-c9-m6-completo.md` §"Qué NO entra en este plan"
  — por qué salió M6-04 y qué recon ya se había hecho.
- `docs/superpowers/specs/2026-08-18-cierre-en-regimen-design.md` §3.2, §"orden"
  (líneas ~205-223) — dónde encaja M6-04 en la cola C9→C10→C11.
- `docs/superpowers/runbooks/2026-08-18-c9-despliegue.md` — estado de despliegue,
  deuda anotada, y el gotcha del binario desincronizado (relevante para D3).
- `docs/superpowers/consultas/2026-08-18-cierre-regimen/consultor-cierre.md` — el
  verdict anterior de tu mismo rol. Mismo listón.
- `engine/src/schema.rs` — DDL completo del índice de exo (45 líneas, léelo entero).
- `engine/src/nota.rs:11-18` — por qué `tier` NO se persiste en el índice.
- `engine/src/indexer.rs:127-132` — el INSERT de `notas`.
- kbx: `internal/index/db.go`, `internal/index/schema.go` (el canary y su lista
  `consumed`), `internal/targets/targets.go:60-120`, `internal/stale/stale.go:120-150`,
  `internal/doctor/doctor.go:150-185`, `internal/fixtures/index.go`.

Índices en disco, ambos abribles en `mode=ro`:
- exo: `~/.exo/index.db` (`notas`, `aristas`, `notas_fts`, `trozos`, `vectores`)
- basic-memory: `~/.basic-memory/memory.db` (`entity`, `relation`, `search_index`, `project`)

`sqlite3` CLI **no está instalado**; usa `python3 -c` con el módulo `sqlite3`.

## Restricciones firmadas (NO adjudicables)

1. **M5b sigue gated** por "M6 completo y probado". No se relaja.
2. **Veto AGPL**: ni una línea de código de basic-memory. Forma de schema sí.
3. **El índice de exo no se rebuildea** por este item: los cambios de schema deben ser
   aditivos (`CREATE TABLE IF NOT EXISTS`). Razón en `nota.rs:14`.
4. **Propiedad protegida**: cada paso deja el sistema funcionando. kbx no puede quedar
   en un estado donde no se sepa contra qué DB corre.
5. Régimen §0 vigente: proyecto personal, cerrar ya, **sin métricas nuevas**.

## Claims a verificar (recon de hoy; si uno cae, dilo y di qué se lleva por delante)

- **C1** — El camino "vistas de compatibilidad" está **muerto**, no es un trade-off:
  FTS5 no soporta `MATCH` ni `snippet()` a través de una `VIEW`, y `targets` depende de
  ambos. Medido: `CREATE VIEW v AS SELECT * FROM <fts5>; SELECT * FROM v WHERE v MATCH 'x'`
  → `OperationalError: no such column: v`.
- **C2** — El filtro `note_type='note'` de kbx oculta **57 de 138 notas** markdown.
  Reparto de `notas.tipo` en el índice de exo: note 81 · report 39 · project 10 ·
  research 3 · guide 3 · spec 1 · person 1. Los 10 `projects/*.md` son los destilados
  que `core-index` lista como activos, y `Paul - perfil de trabajo.md` es `type: person`.
  Cuadre: basic-memory tiene 143 entities = 138 notas de exo + 5 assets `file`.
  Los tres comentarios del código de kbx justifican el filtro como "excluir assets
  pdf/tex/cls/json", pero el walker de exo solo indexa `.md` con permalink.
- **C3** — Paridad exacta de huérfanas hoy: **7 = 7, los mismos 7 ficheros**,
  intersección completa, cero en cada diferencia.
- **C4** — La paridad de *ranking* de `targets` es **inalcanzable**, y prometerla sería
  vender un gate que no puede fallar por las razones buenas. Causas: tokenizers distintos
  (`unicode61 tokenchars 0x2F` vs el de basic-memory), columnas FTS distintas
  (`titulo,cuerpo` vs `title,content_stems,content_snippet` — stemmed), y basic-memory
  duplica filas (160 filas `type='entity'` para 143 entities; exo es 1:1, 138/138).
- **C5** — `tier` y `size` no necesitan columna nueva: `targets` ya abre cada fichero
  candidato para extraer headings, así que salen del mismo `open`+`stat`. Precedente
  interno: `budget` y `ratchet` ya leen tier de `internal/frontmatter`, no de SQL.
- **C6** — La trampa `NOT IN` + NULL está **activa**: hay 23 aristas con
  `destino_permalink IS NULL` (de 573). Sin la guardia `IS NOT NULL`, la query de
  huérfanas devuelve **0 en vez de 7** — verde y silenciosa.

## Decisiones ya tomadas por Paul en el brainstorm (encima de este recon)

No las readjudico salvo que tu verificación las tumbe. **Si un claim cae, di
explícitamente qué decisión cae con él.**

- **P1** — Alcance: **port completo** de kbx. Las 6 queries que abren la DB. `budget`,
  `ratchet` y `rotate` no se tocan (no leen tablas, solo el fallback de kbRoot).
- **P2** — kbRoot: **tabla `meta(clave,valor)` en el índice de exo**, poblada por el
  indexer con `kb_root`. Argumento aceptado: es **procedencia** ("de qué KB es este
  índice"), no config ("qué KB debo usar"), y por eso no colisiona con la config propia
  de C10 ni invierte el orden de campañas.
- **P3** — El filtro `tipo='note'` se porta **conservándolo**, y se quita en un
  **segundo commit propio**. Razón: separa "el port es correcto" (gate = paridad exacta
  contra basic-memory) de "el alcance cambia" (delta esperado 81→138, inspeccionable).

## Decisiones a adjudicar

- **D1 — El gate asimétrico.** La propuesta es: `doctor` huérfanas → paridad exacta
  (7=7, mismos ficheros); `stale` → solo paridad del grado-0, **no** del ranking (exo
  extrae 573 aristas, basic-memory 674: la extracción de links difiere); `targets` →
  solo overlap de conjunto en top-N sobre 3-5 topics, **a ojo**. ¿Firmas el gate
  asimétrico, o exiges hacer `targets` comparable de verdad (corpus congelado + overlap
  medido en vez de inspección manual)? Considera el régimen §0 ("sin métricas nuevas")
  contra el hecho de que `targets` es el comando caliente y el único que Paul invoca a
  mano. Si exiges medición, define el umbral **antes** de que nadie lo corra.
- **D2 — `journal_mode`.** El índice de exo está en `delete`; basic-memory en `wal`.
  kbx abre `mode=ro` y corre desde `kb-precommit.sh`, un **pre-commit hook**. Si
  `exo index` escribe a la vez (arranque de sesión), el lector puede comerse
  `SQLITE_BUSY` y **tumbar un commit**. Hoy no pasa porque son DB distintas; después del
  port, sí. Dos salidas: (a) pasar exo a WAL — arregla la clase entera, pero aparecen
  ficheros `-wal`/`-shm` junto al índice y hay que comprobar que no rompe nada que asuma
  un fichero único; (b) `busy_timeout` en kbx — local, no toca exo, pero solo tapa a
  este consumidor. Adjudica, y di si hay una tercera que no he visto.
- **D3 — ¿Sobrevive el canary, y con qué contenido?** Hoy `CheckSchema` vigila que
  basic-memory no derive. Mi argumento para conservarlo repuntado: cambia de propósito,
  de "schema externo deriva" a "**kbx y exo son dos binarios con ciclos de release
  independientes**" — y ese riesgo está demostrado, no es teórico: el runbook de C9
  documenta que `~/.local/bin/exo` estuvo 20 h por detrás del fix de la campaña sin que
  nada avisara. Nueva lista `consumed` propuesta: `notas`, `aristas`, `notas_fts`, `meta`.
  Se cae `observation` (nadie la consulta nunca). `trozos`/`vectores` no entran (kbx no
  los toca). Verificado que `PRAGMA table_info` funciona sobre tablas fts5. ¿Firmas? ¿O
  el canary sobre schema propio es ceremonia, y basta un `meta.schema_version`?
- **D4 — Fixtures de test.** `internal/fixtures/index.go` (302 LOC) construye una DB con
  DDL "verbatim from the live index" de basic-memory. Tras el port debe reflejar el DDL
  de `engine/src/schema.rs`. ¿DDL copiado a mano (hermético, rápido, riesgo de
  divergencia silenciosa) o generado invocando el binario `exo` real en los tests
  (honesto, pero acopla la suite Go a un binario Rust y a que esté instalado)? Si eliges
  copiado a mano, di qué mecanismo detecta la divergencia y por qué basta.
- **D5 — Orden y cutover.** Propuesta: T1 `meta` en exo · T2 port de queries+canary+
  fixtures conservando el filtro · T3 quitar el filtro · T4 call sites y default de
  `KBX_DB` (hoy `~/.basic-memory/memory.db` → `~/.exo/index.db`; **ese default es el
  interruptor real del cutover**). Rollback = binario kbx anterior, sin modo dual: para
  el gate de paridad basta correr binario-viejo-sobre-basic-memory contra
  binario-nuevo-sobre-exo. ¿Firmas el orden y el rollback-por-binario, o hay una razón
  para que kbx hable los dos dialectos durante una ventana?
- **D6 — Hallazgos nuevos.** Igual que en tu verdict anterior: busca la familia de fallo
  que este diseño no lista. Sitios donde mirar — el resto de `consumed` que nadie ha
  auditado; qué pasa con `diff-since`/`history` si `notas.ruta` y `entity.file_path` no
  usan la misma convención de path; si `exo index` borra y reinserta filas de forma que
  un lector concurrente vea un estado intermedio; y si alguna skill o hook depende del
  shape JSON de kbx de un modo que el port cambie.

## Delta de estándares tácitos de Paul (aplícalos como criterio de adjudicación)

- **YAGNI despiadado**; odia el over-engineering. Solución que funciona hoy >
  arquitectura perfecta. Si dudas entre simple y preparado-para-el-futuro, **simple**.
- **Gates falsables pre-registrados**: un check que no puede distinguir éxito de no-op
  no es un check. Es literalmente el aprendizaje que dejó C9 escrito en su runbook.
- **Evidencia > vibes**: cada claim con dato o cita (`ruta:línea` o `§X`). Verificación
  primaria propia; "el brief dice" no es evidencia.
- **Trade-offs explícitos**: en cada adjudicación, una línea de "qué pierdo eligiendo
  esto". Prefiere una respuesta con opinión a una "safe".
- **Cuestiona y espera ser cuestionado**: si una decisión ya tomada (P1-P3) no se
  sostiene, dilo. Paul prefiere que se lo tumben ahora a descubrirlo en ejecución.
- **Fallo silencioso es el enemigo**: el bug caro de C9 fue un indexer no transaccional,
  y el segundo susto un `pre-commit` roto que git trataba como ausente y dejaba commitear
  con exit 0. Prioriza detectar sobre tolerar.
- **Castellano**, técnico en inglés sin traducir, directo, sin corporate speak.

## Punteros de memoria (rutas de fichero — no URIs MCP)

La doctrina y el digest de memoria te los inyecta reflex al arrancar. Si necesitas más:
- `/home/paul/Documentos/proyectos/kb-demo/projects/exo-framework-unificado-de-trabajo-agentico.md`
- `/home/paul/Documentos/proyectos/kb-demo/projects/kbx — explorador determinista de la KB (Go).md`
- `/home/paul/Documentos/proyectos/kb-demo/core/doctrina-agentes.md`
- `/home/paul/Documentos/proyectos/kb-demo/log/kbx-bitacora.md`
- `/home/paul/Documentos/proyectos/kb-demo/log/exo-bitacora.md`

## Formato del veredicto (tu output final, markdown)

1. **Veredicto global**: `FIRMA` · `FIRMA-CON-CAMBIOS` · `NO-FIRMA`, con 3-5 líneas de por qué.
2. **Verificación de C1-C6**: por claim, `SOBREVIVE` / `SOBREVIVE CON MATIZ` / `NO SOBREVIVE`,
   con tu método y tu dato. Si uno no sobrevive, di qué decisión P1-P3 o D1-D5 se lleva.
3. **Adjudicación de D1-D5**: por decisión — **Adjudicación** (una frase imperativa) ·
   **Racional** (2-4 frases con citas) · **Trade-off aceptado** (1 frase).
4. **D6 — Hallazgos nuevos**: máximo 6, ordenados por coste de descubrirlos tarde.
5. **Riesgos que el orquestador debe vigilar** (máx 5 bullets).
6. **Disenso**: qué buscaste para objetar y qué refutaste. Si no encontraste nada que
   objetar en alguna sección, dilo explícitamente — no rellenes.
7. **Preguntas que SÍ requieren a Paul**: solo si tocan línea roja (destructivo/externo/
   agenda). Si no hay, dilo.

Escribe tu veredicto en
`docs/superpowers/consultas/2026-08-18-m6-04-kbx/consultor-m6-04.md`.
