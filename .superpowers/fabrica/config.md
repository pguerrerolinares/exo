# Config de fábrica — exo

> Seam por proyecto (spec `agent-develop/docs/superpowers/specs/2026-07-09-fabrica-campaign-harness-design.md`
> §4.6 — cada § citada abajo apunta a ese contrato salvo que se diga lo contrario).
> VERSIONADO. Redactado 2026-07-17 como bootstrap (no hay aún sesión pre-campaña
> síncrona con Paul para exo) — Paul lo gatea como cualquier rama, igual que hizo
> con kbx y cge en su día. Mientras no haya pre-campaña, las clases pre-autorizadas
> quedan deliberadamente escasas (patrón kbx, no cge).

## Fuentes de criterio escrito (para la regla de la cita)
- `docs/superpowers/specs/2026-07-16-framework-unificado-design.md` — contrato
  raíz del framework (roadmap §7, régimen de gates §8, riesgos §9, decisiones
  abiertas §10). Toda decisión de exo se cita contra ESTE documento, no contra
  configs de otros repos.
- `docs/superpowers/consultas/2026-07-16-framework/informe-consultor-{framework,engine,thin,thick,roadmap}.md`
  — audit trail de las 5 consultorías adversariales que ratificaron la spec.
- `evals/retrieval-fase0/gate.md` — gate numérico pre-registrado de M0 (inmutable).
- `evals/retrieval-fase0/verdict/m0-verdict.md` y `.../verdict/labels.md` —
  verdicts ya firmados de M0 (jina-es gana 7/0; semántica load-bearing 26/55 ⇒
  Rust firmado en spec §10 decisión 1; ground-truth 17/17 aprobado).
- Notas de la KB `wisdom-paul` `[[desarrollo-agentico]]` y `[[doctrina-agentes]]`
  — doctrina transversal (pirámide de coste,
  medir antes de confiar, agente independiente corrige al coordinador).
- Precedente operativo de régimen de gates: `e33-scripts/lighthouses_aicontest/.superpowers/fabrica/config.md`
  §"Ejecución de gates" (auditor opus delegado, override de Paul 2026-07-12) y
  los verdicts ya escritos en exo bajo el mismo régimen (`m0-t8`, `consultor-gate`
  sobre el eval de M0) — precedente citado explícitamente en spec §8.


## Roadmap / backlog
Fuente: spec §7 (grafo de milestones original, hoy cerrado: M0-M4 y M6 hechos, M5a decidido no construir, M5b cerrada por D3) + §8 (ejecución con fábrica). **El roadmap vivo es la ola 3**; lo que queda fuera de fábrica está debajo. El diagrama M0..M7 original está en `docs/superpowers/fabrica-historico.md`, Apéndice A, y en el spec.

| Campaña | Qué | Lane | Estado 2026-10-05 |
|---|---|---|---|
| **P** | `upstream-sync`: deuda del bot + primera pasada puntuada | mecánica + PAUL-STEP | **MERGEADA** en `main` `beecead` (`6a9ffcb`, `1e20dd4`, `5dc8382`, `190a756`). Pasada 1: 81 % (30/37), no pasa el gate; Paul activó la routine de todas formas |
| **O** | Deuda menor viva (engine, evals, CI, restos de macOS) | mecánica | rama `o-deuda`, **gate MERGED por consultor fable con condición «vía PR»** (verdict `0c0d2a4`); merge pendiente del PR de Paul |
| **Q** | Shipeo a W11: `install.ps1`, `doctor`, aviso de superpowers, `hook_ms` | mecánica, oráculo = CI Windows | rama `q-w11`, **gate MERGED por consultor fable con condición «vía PR, mergear solo con `install gate (windows-latest)` verde»**; diseño del disable por verdict B' (`verdicts/ola3-q-disable-superpowers.md`). Post-merge: release de engine + prueba real en W11 (PAUL-STEP) |
| **N** | Sincronía de `docs/backlog.md`, `pendiente-paul.md` y este config | mecánica (docs) | rama `n-sync`, esta |
| R | `retirar-limites` | — | **ENTERRADA** por D1: tag local `archivo/retirar-limites` (`3226cfa`), sin pushear. Los techos por tier y el ratchet siguen vivos |

**Fuera de fábrica — pre-campaña con Paul** (`docs/superpowers/consultas/2026-10-05-campanas/propuesta.md` §4; ninguno tiene criterio de cierre escrito, todos piden `exo:brainstorm` antes de poder ser campaña):
- `techo-reglas-2`, «última bala» (KB-exo:38-42): el gate está escrito, el pre-registro nuevo no existe; la nota del 10-05 (`prompt.compose`) cambia el diseño del brazo. Candidata a lane diseño **cuando haya pre-registro firmado**.
- platform.claude.com como fuente, mods de Claude Code y `exo serve` (proceso residente para SessionStart, prior art tgrep): un brainstorm o tres, lo decide Paul.
- Serie de medición A+ (KB-exo:34-37): sin campaña; las ramas de cada ola pueden alimentarla registrando sus métricas en el ledger.
- Tasa de re-explicación: aparcada, necesita que Paul etiquete 436 prompts (~40 min).
- `search-first`: leer la tasa `ok/(ok+aviso)` en `~/.claude/reflex-log.jsonl`. Es lectura, no campaña.

## ACTUALIZACIÓN 2026-10-05 — ola 3: cerrar frentes de exo (manda sobre todo lo de abajo)

Propuesta: `docs/superpowers/consultas/2026-10-05-campanas/propuesta.md` (recon de
vigencia contra el código, no contra el backlog, base `origin/main` `a3fcb5f`).
**Mecánica entera**: la instrumentación de la ola 2 dijo que la fábrica rinde en
lane mecánica y deja de ser fábrica en lane de diseño experimental; lo de diseño
quedó en el roadmap, fuera de fábrica. Estado de las campañas: ver §Roadmap.

**Decisiones de Paul, 2026-10-05 (citas literales)**
- **D0**: "D0 lanza P ya" — P se lanzó antes de que el watchdog de `upstream-sync`
  caducara (2026-10-08).
- **D1**: "D1 entiérrala con tag" — `retirar-limites` → tag local
  `archivo/retirar-limites`; los techos por tier y el ratchet **siguen vivos**.
- **D2**: "D2 cerrar" — H28, conversión L2→L2²: se cierra declarando 0,40 en la
  escala L2 sellada (≈ coseno 0,28), sin convertir.
- **D3**: "d3, hecho" — M5b: basic-memory y kbx desinstalados.
- **D4**: "d4, ok" — `archive/` se queda indexado, sin downrank.
- **D5**: "d5 retíralo" — ítem de la KB, no del repo; no aplica a este config.
- **D6**: "D6 borra ramas" — 30 ramas locales mergeadas y el worktree `campana-k`.
- **D7**: "d7 ok" — gate asíncrono. Después: "completa las campañas en sesiones
  nuevas. las decisiones que salgan las solventa consultor fable, yo me piro a
  dormir".

**Régimen**: el de §Ejecución de gates (consultor fable fresco, las 4 condiciones,
verdict commiteado en path versionado ANTES del `GATE-EXEC`). Líneas rojas
intactas: push, PR (abrirlo exige push), tag y release son de Paul. Por eso O
queda con el gate concedido y el merge pendiente de PR, y Q va por PR (lección 1
de la ola 2). `docs/backlog.md` lo toca solo N, y va la última; el bump de
`plugin.json` se re-bumpea en orden de merge.

**Lecciones que esta ola incorpora, las tres con su caso**
1. **Una pasada puntuable que nadie puntúa no es un gate.** Caso: la pasada 1 de
   `upstream-sync` se mergeó (#33) «sin puntuar»; al puntuarla en la ola 3 daba
   `aciertos 30/37 (81%)` con rc=1 contra el gate ≥90 %. Y el gate, además, no
   mide los «no aplica» falsos: #2258/plan marcado `no aplica` por D8
   sobre-extendida es un error real que el 81 % no cuenta (backlog, Baja).
2. **El brief del orquestador fue la fuente de dos errores que se corrigieron
   aguas abajo; el executor o el consultor que va a la fuente gana al brief.**
   Casos: (a) el brief de O invirtió 0,28 y 0,393 en `kb_sintetica.rs` y el
   executor siguió el criterio literal del backlog; (b) el brief de Q dijo «no
   desactivar superpowers por defecto» contra la cita de Paul en KB-exo:45
   («`claude plugin disable superpowers` avisando, nunca uninstall»); el
   consultor lo adjudicó a B' (`verdicts/ola3-q-disable-superpowers.md`).
3. **El package no puede contradecir la propuesta que lo funda.** Caso: la
   propuesta decía de O «el job de CI va por PR» y el package de O prescribió
   merge local; lo cazó el consultor del gate (condición «vía PR», `0c0d2a4`).

**Se mantienen**: la línea roja, el régimen de gates por consultor fable con sus
4 condiciones, la regla PENDIENTE-PAUL, que ninguna task escribe en
`~/.local/bin`, y que la fábrica **no pushea la KB**.

---

## ACTUALIZACIÓN 2026-09-20 — ola 2: J (juicio del gold) + dos bugs vivos (histórico: ola cerrada; la sustituye el bloque 2026-10-05)

Ola 1 cerrada y publicada: `v0.2.0` → `6aa5257`, release en GitHub, CI 12/12.
L, I y J fase 1 mergeadas (`7e29f42`, `18ba004`, `aaf66c0`) y `main` = `origin/main`
= `b8e3117` tras el PR #28.

**El recon pre-flight de esta ola encontró que el roadmap estaba inflado**: de los
12 candidatos que `docs/backlog.md` presentaba como abiertos, **9 están cerrados**
en el código de hoy, con commit o test que lo demuestra, y el issue **#23 está
CLOSED** con su fix en `main` (`876264a`, PR #27). La ola 2 es por tanto mucho
más corta de lo que el backlog sugería, y eso es un resultado, no un problema.

| Lane | Campaña | Tipo | Estado |
|---|---|---|---|
| diseño (secuencial) | **A — juicio del gold de J, acuerdo y congelación del pre-registro** | evals + dispatches de juez; **no toca `engine/src`** | critical path: desbloquea la fase 2 entera |
| mecánica A | **B1 — los dos bugs de `evals/recall-coste/harness/bench.sh`** | bash | independiente |
| mecánica B | **B2 — H28: el doc-comment de `buscador.rs` es falso** | Rust, doc + test, **sin cambio de ranking** | independiente |
| mecánica C (última) | **C1 — sincronizar `docs/backlog.md` con lo que ya está cerrado** | docs | va al final, absorbe lo que cierren A/B1/B2 |
| PAUL-STEP | **`hook_ms` en W11** (Task 7 de I) | medición manual | no ejecutable por la fábrica: requiere la máquina de Paul |

**Decisiones de Paul, tomadas en sesión el 2026-09-20 (citas literales)**
- **Juzgar el gold de J: sí, ya** — "Sí, juzgar ya". 285 filas, $2,7-4,5, tope
  estructural $10,02, irreversible (consume las queries).
- **`temperature` del juez: 0 → 1, DECLARADO en el pre-registro** — "temperature 1
  declarado en el pre-registro". No omitido: el cuerpo de la petición sigue
  declarando qué temperatura se usó.
- **`--tolerancia-desconocidas 0`**, con la fábrica reanudando por id cada parada —
  "Tolerancia 0 y reanudo yo cada parada". El techo de gasto sigue siendo
  estructural.
- **Gate de merge asíncrono** (ramas + `GATE-EXEC` con su "ok, mergea") — "Asíncrono,
  como en la ola I∥L+J".
- **Tamaño: ~57 dispatches** — "Como la ola anterior". El recon lo dejó en mucho
  menos: el grueso serán los dispatches del segundo juez de J.

**Zonas de colisión**
- `docs/backlog.md`: **solo lo toca C1**, y va la última. A, B1 y B2 tienen prohibido
  editarlo — es el fichero que más conflictos dio en las olas anteriores.
- `engine/src/buscador.rs`: solo B2, y solo doc-comments y un test nuevo. Ninguna
  otra task de la ola toca `engine/src`.
- `evals/retrieval-heldout/`: solo A. `evals/recall-coste/`: solo B1. Disjuntas.

**Lecciones que esta ola incorpora al régimen, las tres con su caso**
1. **Un fix de portabilidad a un SO que no se puede probar en local se mergea por
   PR, nunca por push directo.** Caso: el CI rojo de `aaf66c0` (2026-09-20) salió
   de un `sed` "sin extensiones GNU" que el sed de BSD rechaza igual; las dos
   verificaciones disponibles en esta máquina (GNU sed y busybox) aceptan ambas
   formas, así que ninguna podía detectarlo. **El CI del SO que no se puede probar
   es parte del fix, no una comprobación posterior.**
2. **Un harness que llama a una API externa y nunca ha hecho una llamada real no
   está verificado**, por muchos tests y rondas adversariales que acumule. Caso: el
   breaker de gasto de `juez.py` pasó 3 rondas de fixes y 4 de review adversarial
   —todas sobre la contabilidad— y la primera llamada real murió con
   `HTTP 400: invalid temperature: only 1 is allowed for this model`. El oráculo
   barato existía y costaba una llamada: `--max 1`.
3. **Antes de planificar una ola, recon de vigencia contra el código, no contra el
   backlog.** Caso: 9 de 12 candidatos de esta ola ya estaban cerrados y el issue
   #23 llevaba un día cerrado con su fix mergeado. Un `docs/backlog.md` de 2.702
   líneas acumula, no caduca.

**Se mantienen**: la línea roja (`git push`, tag y release = SIEMPRE Paul), el
régimen de gates por consultor fable fresco con sus 4 condiciones, la regla
PENDIENTE-PAUL, que ninguna task escribe en `~/.local/bin`, y que la fábrica **no
pushea la KB**.


---

> Los bloques `ACTUALIZACIÓN` anteriores al 2026-09-20 (2026-09-19, 2026-09-15,
> 2026-09-14, 2026-09-13 y 2026-08-17), íntegros, están en
> [`docs/superpowers/fabrica-historico.md`](../../docs/superpowers/fabrica-historico.md).

## Lanes (routing, spec §8 + skill §1)
- **Mecánica**: el criterio de cierre está escrito y se verifica con un comando
  (`cargo test`, `scripts/test-*.sh`, CI). Es donde la fábrica rinde (ola 2, B1/B2/C1;
  ola 3 entera). Executor sonnet, review final opus por rama.
- **Diseño experimental** (pre-registro, gold, juez): secuencial, fable en cabeza,
  y **no es fábrica en sentido estricto**: la ola 2 tuvo 11 decisiones de Paul en
  sesión, casi todas ahí. Solo entra con pre-registro firmado.
- **Superficies irreversibles** (envelope JSON, schema del índice, formato de skill):
  pasan SIEMPRE por el régimen de gates, nunca por clase pre-autorizada.
- Un fix de portabilidad a un SO que no se puede probar en local (W11) va por PR,
  con el CI de ese SO como parte del fix.

## Oráculos (comando literal + qué prueba)
- **Engine**: `cd engine && cargo test --release --locked` (prueba el comportamiento;
  621 passed, 0 failed, 12 ignored el 2026-10-05, ~40 s) y
  `cargo clippy --all-targets --locked -- -D warnings` (lint sin avisos).
- **Gates de repo**: `scripts/test-*.sh` desde la raíz (docs vivos, rutas
  personales, contrato de CI, hooks.json, shellcheck, hermético, upstream-*, …).
- **Plugin**: `plugins/exo/scripts/test-*.sh`, lanzados con `scripts/test-plugin.sh`.
- **Suite Python del harness de J**: `python3 -m unittest discover -s
  evals/retrieval-heldout/harness -p "test_*.py"` (82 tests; sin red, los de red
  van mockeados). Corre en CI desde la rama `o-deuda` (pendiente de PR).
- **CI**: `.github/workflows/ci.yml` (job `static-checks`, jobs de test) y
  `release.yml`. Es el oráculo del SO que no se puede probar en local: **parte
  del fix, no una comprobación posterior**.
- **Skills de process**: sin oráculo mecánico — el checklist de paridad contra la
  skill absorbida, verificado por el consultor del gate.
- **Pasada de `upstream-sync`**: `scripts/upstream-score.sh <verdad> docs/upstream/ledger.md`
  (rc=1 si no pasa ≥90 % o hay ya-cubierto falsos).

## Corpus negativos
- **Retrieval**: el held-out de J (`evals/retrieval-heldout/`, 240 filas, 95 nulas)
  ya está consumido como orientativo en B1/B2; cualquier cambio de ranking exige
  un held-out nuevo, no se afina sobre éste.
- **`upstream-sync`**: `verdad-v6.1.1-v6.4.2.md` ya está vista; la próxima pasada
  puntuable necesita verdad nueva, con los «no aplica» falsos declarados antes
  de puntuar.
- **Write-path**: corpus de casos search-before-write, sin construir (spec §4.2).

## Presupuesto (unidades spec §7: dispatches por modelo + horas de reloj)
Recalibrado el 2026-10-05 con la evidencia de la ola 2. Se separan **dos tipos** de
reserva fable porque la ola 2 demostró que no son intercambiables:
- **Reserva fable de gates / adjudicación / reviews finales: ≤ 8 dispatches por
  ola.** Se mantiene: la ola 2 la usó a 4/8 (1 revisión adversarial del
  pre-registro de J + 3 reviews finales de rama) y no se agotó. Cada gate
  (merge o superficie irreversible) cuesta UN dispatch fable adicional (el
  consultor delegado, §Ejecución de gates). Si en una ola el ratio gates/reserva
  supera 50 %, se consolidan items, no se sube en caliente.
- **Juicio de gold / spec experimental: SIN reserva genérica.** La ola 2 gastó
  **30 fable reales contra 8 de reserva** y contra 14 del plan de J firmado
  (desviación D3 del informe de cierre: 29 lotes de juez + 1 gate), porque el
  kit se dobló (274 K → ~475 K caracteres por lote) y se relanzaron lotes. El
  brazo de un juez **no cabe en una reserva de 8**: se presupuesta por el plan
  específico y firmado de cada campaña de gold, con el coste del lote
  recalculado si cambia el kit, y se declara antes de despachar.
- **Contar por filas**: la ola 2 incumplió esto (18 dispatches sin fila,
  desviación D1: el ledger decía 13 fable, eran 30). El contador de §Instrumentación
  separa fable-en-gates de fable-en-gold y se recalcula contando filas.
- **cap at-risk por item**: ≤ 8 dispatches sonnet o 3h de reloj.
- **cap retries por eval**: 2 (default spec §8).
No hay cifras nuevas: la reserva de gates se queda donde estaba, con ola 2 como
base; la de gold se retira del config y pasa al plan de cada campaña.

## Clases de decisión pre-autorizadas (spec §5.1)
> Deliberadamente mínimas (patrón kbx, no cge): no hubo sesión pre-campaña
> síncrona todavía. Ampliar SOLO en una pre-campaña real con Paul — la
> "Prohibición de clases nuevas mid-campaña" del precedente cge aplica aquí
> desde el día 1.
- (Retiradas el 2026-10-05, agotadas: el backfill de `type:` y la limpieza de
  root files de la higiene M1a se hicieron y mergearon el 2026-07-17; su texto
  está en `docs/superpowers/fabrica-historico.md`, Apéndice A.)
- Layout interno de directorios bajo `engine/`, `plugins/`, `templates/`
  (cuando arranquen) siguiendo el patrón ya usado por agent-develop (estructura de plugin) — decide el executor sin verdict,
  **salvo que toque el envelope JSON o el formato de skill**, que son
  superficies irreversibles (van al régimen de gates).
- (resto vacío hasta sesión pre-campaña)


## Ejecución de gates

**Régimen firmado por Paul 2026-07-16 (spec §8, párrafo "Régimen de gates")** —
sustituye, para exo, el default del skill ("Paul registra su veredicto...").
Ya en uso en exo (M0: verdict `m0-t8`/`consultor-gate` sobre el eval de M0).

- **Todo gate que el protocolo estándar derivaría a Paul** (`GATE: MERGED/RECHAZADA`
  en un review-package, y toda decisión sobre las superficies irreversibles
  internas nombradas en spec §8: envelope JSON, schema del índice, formato de
  skill) **lo adjudica un consultor Fable delegado**, no Paul. Paul
  pre-aprueba el régimen, no cada veredicto.
- **Condiciones (las 4 del §8 — sin TODAS, el verdict es inválido y escala a
  Paul igual que un verdict sin cita per §6 del skill)**:
  1. **Fresco**: el consultor es un dispatch nuevo, sin haber participado en
     ninguna fase de la pieza que juzga (no revisa su propio trabajo). Se
     brifea solo con el deliverable + criterio, no con el razonamiento previo
     del orquestador.
  2. **Verificación primaria propia**: el consultor re-corre los oráculos
     citados (no se fía de un resumen) — patrón ya practicado en
     `m0-verdict.md` (re-corrida byte-idéntica de `analyze.py` + recomputación
     independiente desde los `.jsonl` crudos, sin importar el script del
     coordinador).
  3. **Mandato explícito de disenso**: el brief del consultor exige que
     declare qué buscó para objetar, incluso si no encontró nada (convergencia
     complaciente = fallo, no éxito). Un verdict sin esta sección es inválido.
  4. **Verdict-artifact commiteado**: a diferencia de los verdicts intra-rama
     del §6 del skill (esos viven en `.superpowers/fabrica/verdicts/`,
     GITIGNORED por contrato §4.4), el verdict de un GATE se escribe **fuera**
     de ese directorio gitignored — en un path versionado del repo (patrón ya
     usado: `evals/retrieval-fase0/verdict/*.md`, commiteado en la misma rama
     que gatea) — y se commitea ANTES de que el orquestador ejecute el merge.
     Sin commit del verdict, no hay `GATE-EXEC`.
- **Mecánica de despacho**: la fábrica despacha el consultor (`model: fable`),
  registra el dispatch en el ledger ANTES de despacharlo (cuenta contra la
  reserva de §Presupuesto), y el consultor appendea al review-package:
  `GATE: MERGED (consultor fable, <ts>, verdict=<path commiteado>)` o
  `GATE: RECHAZADA — <motivo> (consultor fable, <ts>, verdict=<path>)`.
- **Ejecución del merge = el orquestador**, nunca el consultor ni Paul: con
  `GATE: MERGED` registrado y el verdict commiteado, el orquestador ejecuta
  `GATE-EXEC` (borra `ACTIVE`, merge `--no-ff`, corre la suite/oráculo
  post-merge, re-arma `ACTIVE`) — mismo mecanismo que cge/lighthouses.
- **`PENDIENTE-PAUL` → `PENDIENTE-CONSULTOR`**: toda decisión que el skill
  derivaría a la cola `pendiente-paul.md` por falta de criterio citable la
  intenta primero el mismo consultor delegado (con las 4 condiciones de
  arriba) ANTES de escalar a Paul. Si el consultor tampoco encuentra fuente
  citable (ni en la spec ni en doctrina), SÍ escala — pero como excepción, no
  como default. El fichero sigue llamándose `pendiente-paul.md` (contrato del
  skill), pero su población normal ahora son residuos que ni el consultor pudo
  cerrar, no el flujo estándar.
- **Línea roja que NUNCA se delega** (spec §8, formulación literal): acciones
  **destructivas o externas al sistema** — borrado de la KB o de repos,
  publicación fuera del repo (push a origin, release, npm publish, etc.),
  cambios de permisos. Estas van SIEMPRE a Paul, sin excepción y sin que
  ningún consultor pueda auto-adjudicarlas. Concretamente para exo:
  - `git push` a cualquier remoto (origin de exo, de `wisdom-paul`, de
    agent-develop) — SOLO Paul, igual que en cge/lighthouses.
  - Cualquier acción sobre `wisdom-paul` que borre o sobrescriba notas sin
    pasar por `doctor`/search-before-write.
  - Cambios a `.claude/settings.json` o a guards PreToolUse (perímetro de
    permisos del propio harness).
  
## Overrides de Paul

Las prohibiciones y caps (incluidos los dos gates de calendario de arriba)
ceden SOLO ante pedido directo de Paul en sesión. Todo override se registra en
el ledger **ANTES** de ejecutarlo:

`OVERRIDE (Paul, <ts>, regla=<cuál>, cita="<palabras de Paul>")`

Acción fuera de letra sin línea OVERRIDE = desviación a declarar en el informe
de cierre (patrón cge, fix A1 del auditor).

## Reapertura post-cierre

Trabajo post-cierre pedido por Paul = REAPERTURA, protocolo de 3 líneas (igual
que el skill §5 y precedente cge):
1. Re-crear el flag `ACTIVE`.
2. Abrir sección `EXTENSIÓN <n>` en el ledger con presupuesto propio
   (remanente de la noche salvo `OVERRIDE`).
3. Registrar dispatches igual que en sesión normal, cerrar con mini-informe +
   borrado de flag.

## Instrumentación

- Todo ts de línea del ledger se escribe con `date -Iseconds` literal — nunca
  a mano, nunca con dígitos enmascarados.
- El contador de cap (fable/sonnet/haiku) se recalcula **contando filas del
  ledger**, nunca de memoria ni de un total declarado a mano.
- **Contador nuevo para el régimen de gates**: dispatches fable gastados en
  adjudicación de gates vs dispatches fable gastados en spec/gold (separar en
  el informe de cierre — es la instrumentación que dirá si la reserva de
  §Presupuesto necesita subir).
- Ratio adjudicado/encolado por noche, % presupuesto critical-path vs filler,
  scope-questions por candidato de gold (>~5 ⇒ señal de spec insuficiente): los
  4 contadores de spec §9, en cabecera del informe de cierre de cada noche —
  sin excepción por ser la primera campaña de exo.
- **Drill de reanudación en frío**: al ser la primera campaña real de exo con
  el harness de fábrica (M0 se corrió con orchestrate-personal plano, sin
  ledger/packages), repetir el drill de cge (§9.4 de la spec): Paul mata la
  sesión tras el segundo item despachado en la primera noche real, rearranca,
  y compara el informe post-reconciliación contra el estado real antes de
  confiar el harness a más noches sin supervisión.

---

`Nota de redacción: este config lo escribió un agente de investigación por
encargo de Paul (2026-07-17), sin sesión pre-campaña síncrona previa. Válido
como bootstrap (patrón kbx); Paul debe gatearlo como cualquier rama antes de
que una sesión-fábrica real lo use, y las clases pre-autorizadas/presupuestos
son punto de partida a ajustar, no cifras firmadas en pre-campaña.`
