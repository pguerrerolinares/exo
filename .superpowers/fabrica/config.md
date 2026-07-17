# Config de fábrica — exo

> Seam por proyecto (spec `agent-develop/docs/superpowers/specs/2026-07-09-fabrica-campaign-harness-design.md`
> §4.6 — cada § citada abajo apunta a ese contrato salvo que se diga lo contrario).
> VERSIONADO. Redactado 2026-07-17 como bootstrap (no hay aún sesión pre-campaña
> síncrona con Paul para exo) — Paul lo gatea como cualquier rama, igual que hizo
> con kbx y cge en su día. Mientras no haya pre-campaña, las clases pre-autorizadas
> quedan deliberadamente escasas (patrón kbx, no cge).

**ALERTA para quien mergee esto**: `.gitignore` de exo ignora `.superpowers/`
en bloque (línea `.superpowers/` — pensada para el scratch de SDD). Sin una
excepción `!.superpowers/fabrica/config.md` (y presumiblemente
`!.superpowers/fabrica/config.md` con `git add -f` si hace falta), este fichero
NO quedará versionado pese a que el contrato lo exige VERSIONADO. Arreglar el
`.gitignore` es prerequisito de que esta rama sea gateable de verdad.

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
- Nota basic-memory `kb-demo/learnings/desarrollo-agentico` y
  `kb-demo/log/doctrina-agentes` — doctrina transversal (pirámide de coste,
  medir antes de confiar, agente independiente corrige al coordinador).
- Precedente operativo de régimen de gates: `e33-scripts/lighthouses_aicontest/.superpowers/fabrica/config.md`
  §"Ejecución de gates" (auditor opus delegado, override de Paul 2026-07-12) y
  los verdicts ya escritos en exo bajo el mismo régimen (`m0-t8`, `consultor-gate`
  sobre `eval.jsonl`) — precedente citado explícitamente en spec §8.

## Roadmap / backlog
Fuente única: spec §7 (grafo de milestones) + §8 (ejecución con fábrica).

```
M0 Fase 0 ──→ M1a repo ──→ M2 E1-read ──→ M4 E2-write ──→ M5a MCP ──→ M5b desinstalar
                 │                                            ▲ gated por M6 completo
                 └──→ M3 cutover skills + M1b marketplace     │
                        (gated: métrica D ≥07-23)             │
                      M6 guardrails ←── (métrica D + M2) ─────┘
                      M7 templates (diferible)
```

**Estado a fecha de este config (2026-07-17)**:
- **M0 — CERRADO.** Verdict firmado (`evals/retrieval-fase0/verdict/m0-verdict.md`,
  commit `f80393a`/`dc74d26` en rama `m0-fase0`): jina-es/768/threshold-0.35 en
  producción, Rust firmado como lenguaje del engine (spec §10 decisión 1).
  **Pendiente solo el merge de `m0-fase0` → `master`** (en curso al redactar este
  config); al reconciliar (§0.2 del skill), git es la verdad — si el merge ya
  aterrizó, marcar `mergeada` sin re-adjudicar nada.
- **M1a — PARCIAL.** Hecho: repo creado, spec/plan/audit-trail commiteados.
  **Siguiente, en este orden** (selector simple: primero lo no-gateado):
  1. **Higiene pre-baseline de la KB** (spec §6.5): backfill `type:` en 11 notas
     de `kb-demo` sin ese campo, limpieza de root files ya flaggeados por
     `kbx doctor` (`developercv.cls`, `fontawesome.pdf`), decisión dotdirs ya
     escrita (se quedan, documentados fuera del bundle — no acción, solo
     verificar que sigue así), borrar comentario huérfano del cron reflex-fp +
     actualizar línea stale del backlog. **Lane mecánica** (oráculo: `kbx doctor`,
     ver §Oráculos) — NO gateada por calendario, puede correr ya.
     **Nota de worktree cruzada**: este ítem toca `kb-demo` (repo git
     separado de exo), no `exo/`. El worktree va bajo
     `~/Documentos/proyectos/kb-demo/.worktrees/<item>` (spec: "un git
     worktree por item, sin excepción"), NO bajo `exo/.worktrees/`. El guard
     PreToolUse de exo (flag `exo/.superpowers/fabrica/ACTIVE`) **no cubre
     `kb-demo`** — hasta que ese repo tenga su propio guard, el perímetro
     de "nunca push/merge a main sin gate" para ESTE ítem se sostiene solo por
     disciplina de la sesión, no por hook. Tratar como línea roja reforzada:
     cualquier `git push`/`merge` en `kb-demo` durante esta sesión exige el
     mismo `GATE-EXEC` que un merge en exo, aunque el hook no lo fuerce.
  2. **Prep-M3: skills de `process` escritas y revisadas, SIN instalar**
     (checklist spec §5.3 paso 1 — solo eso, NUNCA el paso 2 "mismo día
     superpowers disabled + process enabled", que es cutover real y está
     gateado por calendario, ver más abajo). **Lane diseño, secuencial**: el
     "gold" de cada skill absorbida es su checklist de paridad de movimientos
     vs la skill superpowers correspondiente (tabla spec §5.2). Formato de
     skill es superficie irreversible interna nombrada en spec §8 → pasa
     SIEMPRE por el régimen de gates (§Ejecución de gates), nunca por clase
     pre-autorizada.
  3. **M2 — E1 read**: bloqueado por `GATE-HUECO-M2` (ver más abajo). No se
     empieza aunque el selector lo alcance en orden.
- **M1b, M3 (cutover real), M6**: bloqueados por `GATE-CALENDARIO-D` (ver
  más abajo). No se seleccionan bajo ninguna circunstancia antes de esa fecha
  Y de que la métrica D esté efectivamente cerrada.
- **M4, M5a, M5b, M7**: no adjudicables aún (dependen transitivamente de M2/M3/M6
  no iniciados); quedan `encolado` sin acción.

### GATE-CALENDARIO-D (pre-registro de la métrica D, spec §7 + §2 decisión 6)
- **Bloquea**: M1b (rename/registro del marketplace), M3 cutover real (paso 2
  del checklist §5.3 en adelante), M6 (guardrails + cutover del hook de recall),
  y **cualquier cambio que altere marketplace/skills/recall del agente**
  (formulación literal spec §2.6: "nada que altere el entorno del agente...
  se ejecuta dentro de la ventana").
- **Condición de apertura**: fecha ≥ 2026-07-23 **Y** la métrica D efectivamente
  medida y cerrada (no basta con que pase la fecha sola — alguien tiene que
  correr `reflex-baseline.sh` post-fix del `jq 2>/dev/null` — spec §7 último
  párrafo — y cerrar el análisis). Ese cierre debe quedar citado en el ledger
  antes de que cualquier item de M1b/M3/M6 pase a `en_curso`.
- **Mecánica**: el orquestador, al reconciliar (§0.2 del skill), verifica esta
  condición ANTES de tocar el selector. Si no se cumple, esos items quedan
  `encolado` con nota "gate de calendario D, ver config" y el selector pasa al
  siguiente item adjudicable. Ningún consultor-gate ni override informal salta
  esto — es un `OVERRIDE` explícito de Paul (§Overrides de Paul) si alguna vez
  se decide adelantar, y dado que invalidaría un experimento pre-registrado,
  se espera que NO se pida nunca.

### GATE-HUECO-M2 (ventana de calendario de Paul, spec línea 171 — no es una fecha)
- **Bloquea**: M2 (E1 read) completo.
- **Condición de apertura**: NO es calculable por la fábrica — es una decisión
  de agenda de Paul ("hueco real entre una LAN party y cge P2"). Flag físico
  en este config: `hueco_m2_abierto: sí, 2026-07-17` (abierto por Paul en
  sesión interactiva 2026-07-17). Mientras diga `no`, la fábrica no
  empieza M2 aunque M0 esté cerrado (ya lo está) y no haya gate de calendario D
  activo para M2 (M2 no está en la lista de items D-gateados).
- Paul abre la ventana editando esta línea a `hueco_m2_abierto: sí, <fecha>` en
  una rama de config gateada como cualquier otra, o vía `OVERRIDE` puntual si
  quiere una sola noche de adelanto sin tocar el config.

## Lanes (routing, spec §8 + skill §1)
- **Higiene KB (§6.5)**: mecánica — oráculo `kbx doctor` ya existe y corrió en
  seco (ver §Oráculos), última corrida verde-parcial 2026-07-17 (5
  `budget_exceeded` preexistentes fuera de scope de este ítem, 2 `waived
  orphan` esperados).
  Correr en paralelo con el ítem 2 si hay executors libres.
- **Prep-M3 skills de process**: diseño, secuencial, fable en cabeza (redacción
  + review adversarial de cada skill contra su checklist de paridad).
- **M2 (cuando se abra)**: el eval set de M0 rutea gran parte a mecánica
  (side-by-side medible por comando, spec §8). Piezas de diseño dentro de M2:
  indexer (gold = paridad de permalinks/corpus), fusión/calibración de search
  (gold = eval set de M0). No detallar más hasta que `hueco_m2_abierto: sí`.

## Oráculos (comando literal + qué prueba)
- **KB doctor** (higiene M1a): `kbx doctor --kb ~/Documentos/proyectos/kb-demo`
  (o sin `--kb` si el default ya apunta ahí — verificado 2026-07-17). Corrida de
  referencia a esa fecha: 5 `budget_exceeded` (notas grandes preexistentes,
  fuera de scope de la higiene pre-baseline) + 2 `waived orphan` esperados
  (README.md, metodologia.md — `kbx_orphan_ok`). El ítem de higiene se
  considera cerrado cuando el backfill de `type:` y la limpieza de root files
  NO introducen findings nuevos (comparar corrida antes/después, no solo "sale
  verde").
- **M0 — retrieval eval** (ya cerrado, referencia para M2): `evals/retrieval-fase0/harness/analyze.py`
  + `harness/replay.py` + `harness/stratify.py` sobre los `.jsonl` de
  `results/`. Última corrida verde: commit `dc74d26` (verdict aplicado). Reusar
  este harness como base del side-by-side de E1 (spec §4.4-E1) cuando se abra
  M2 — no reinventar el arnés.
- **kbx suite** (si M1a toca código de kbx al absorberlo "tal cual"):
  `cd ~/Documentos/proyectos/kbx && make check` (build + vet + test, tag
  `sqlite_fts5`). Nada debe cambiar de comportamiento en la absorción — un
  diff de comportamiento post-absorción es bloqueante, no ajuste cosmético.
- **Skills de process (prep-M3)**: sin oráculo mecánico — el "oráculo" es el
  checklist de paridad de movimientos vs la skill superpowers absorbida (spec
  §5.2, tabla), verificado por el consultor-gate (§Ejecución de gates), no por
  comando.

## Corpus negativos
- **Higiene KB**: no aplica en el sentido clásico (no hay extractor); el
  equivalente es la lista de findings de `kbx doctor` — cualquier finding NO
  cerrado por la higiene y cualquier finding NUEVO introducido cuenta como
  regresión.
- **M2 (indexer/fusión, cuando se abra)**: el propio eval set de M0
  (`evals/retrieval-fase0/eval.jsonl`, 56 filas, incluye filas `null` como
  negativos explícitos de permalink) es el corpus negativo/gold de partida —
  no se construye uno nuevo desde cero (spec §8: "el eval set de M0 es el
  oráculo").
- **M4 write-path (futuro)**: corpus de casos search-before-write — sin
  construir aún (spec §4.2, decisión explícita de no adelantarse).

## Presupuesto (unidades spec §7: dispatches por modelo + horas de reloj)
- **reserva fable**: ≤ 6 dispatches/noche; ≤ 20% del cap semanal de Paul.
  **Ajuste explícito por el régimen de gates nuevo**: cada gate (merge o
  superficie irreversible) ahora consume UN dispatch fable adicional (el
  consultor delegado, §Ejecución de gates) que antes no existía como dispatch
  de modelo (era el ojo de Paul, gratis en presupuesto de rate limit). Esto
  puede agotar la reserva más rápido que en cge/kbx — si el ledger muestra
  ratio gates/reserva alto, es señal para subir la reserva en la siguiente
  rama de config, NO para saltarse la regla de la cita del gate.
  Valores estimados a ojo (patrón cge) — Paul los ajusta al gatear esta rama.
- **cap at-risk por item**: ≤ 8 dispatches sonnet o 3h de reloj.
- **cap retries por eval**: 2 (default spec §8).

## Clases de decisión pre-autorizadas (spec §5.1)
> Deliberadamente mínimas (patrón kbx, no cge): no hubo sesión pre-campaña
> síncrona todavía. Ampliar SOLO en una pre-campaña real con Paul — la
> "Prohibición de clases nuevas mid-campaña" del precedente cge aplica aquí
> desde el día 1.
- Backfill mecánico de `type:` en frontmatter cuando el valor lo determina sin
  ambigüedad el contenido/carpeta de la nota (p.ej. `projects/*.md` → `type:
  project`) y `kbx doctor` ya lo señala como finding — decide el executor sin
  verdict.
- Limpieza de root files ya flaggeados por `kbx doctor` como fuera de lugar
  (mover/borrar `developercv.cls`, `fontawesome.pdf` según ya decidido en spec
  §6.5) — decide el executor sin verdict.
- Layout interno de directorios bajo `engine/`, `plugins/`, `templates/`
  (cuando arranquen) siguiendo el patrón ya usado por kbx (`cmd/`, `internal/`)
  y por agent-develop (estructura de plugin) — decide el executor sin verdict,
  **salvo que toque el envelope JSON o el formato de skill**, que son
  superficies irreversibles (van al régimen de gates).
- (resto vacío hasta sesión pre-campaña)

## Ejecución de gates

**Régimen firmado por Paul 2026-07-16 (spec §8, párrafo "Régimen de gates")** —
sustituye, para exo, el default del skill ("Paul registra su veredicto...").
Ya en uso en exo (M0: verdict `m0-t8`/`consultor-gate` sobre `eval.jsonl`).

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
  - `git push` a cualquier remoto (origin de exo, de kb-demo, de
    agent-develop) — SOLO Paul, igual que en cge/lighthouses.
  - Cualquier acción sobre `kb-demo` que borre o sobrescriba notas sin
    pasar por `doctor`/search-before-write.
  - Cambios a `.claude/settings.json` o a guards PreToolUse (perímetro de
    permisos del propio harness).
  - El `GATE-CALENDARIO-D` y el `GATE-HUECO-M2` de este config — un consultor
    NO puede autorizar adelantarlos; eso es un `OVERRIDE` de Paul o nada.

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
