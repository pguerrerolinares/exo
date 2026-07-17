# Gold — process:orchestrate (paridad de movimientos)

Fuentes (tabla framework §5.2):
- SDD = `~/.claude/plugins/cache/claude-plugins-official/superpowers/6.1.1/skills/subagent-driven-development/SKILL.md` (418 líneas)
- EP = `.../skills/executing-plans/SKILL.md` (70 líneas)
- DPA = `.../skills/dispatching-parallel-agents/SKILL.md` (185 líneas)
- OP = `~/.claude/plugins/cache/agent-develop/paul-profile/0.5.0/skills/orchestrate-personal/SKILL.md` (135 líneas) — de aquí, como mínimo, lo nombrado en framework §5.2 (cost pyramid, memory packet, blindspot pass, reviewer-dispatch escalado al riesgo del diff); además se absorben los movimientos de OP que morirían con ella en el cutover (ver §Movimientos — orchestrate-personal abajo)

Uso: ver `evals/prep-m3/README.md`.

## PARIDAD CRÍTICA (framework §5.3.2 — si falta, la implementación es inválida)

- [ ] El dispatch de ejecutores usa `subagent_type: reflex:executor` SIN parámetro `model` — OP líneas 50-51 ("Despacha los ejecutores como `subagent_type: reflex:executor`, nunca `general-purpose`") + OP líneas 27-28 ("Salvo roles con `model` fijo en su definición — p.ej. `reflex:executor` — donde NO debes pasar `model` en el dispatch (lo pisarías)") + framework §5.3.2 ("si se pierde, reflex v2 se desenchufa sin síntoma")

## Movimientos — núcleo SDD

- [ ] Subagente fresco por tarea con contexto aislado: nunca hereda la historia de la sesión; el orquestador construye exactamente lo que necesita — SDD líneas 8-10
- [ ] Ejecución continua: sin check-ins entre tareas; solo se para por BLOCKED irresoluble, ambigüedad que impide avanzar, o todas las tareas completas — SDD línea 17
- [ ] Narración mínima entre tool calls (una línea corta; el ledger y los resultados llevan el registro) — SDD líneas 14-15
- [ ] Pre-flight plan review antes de Task 1: buscar contradicciones internas y mandatos que el rubric trataría como defecto; presentarlos como UNA pregunta batcheada, no un interrupt por hallazgo — SDD líneas 85-97
- [ ] Review por tarea con DOS verdictos (spec compliance + code quality) y review final whole-branch al acabar todas — SDD línea 8 + líneas 355-359
- [ ] Manejo de estados del implementer: DONE → review; DONE_WITH_CONCERNS → leer concerns antes de seguir; NEEDS_CONTEXT → aportar y re-despachar; BLOCKED → contexto/modelo mayor/partir la tarea/escalar al humano; nunca forzar retry sin cambiar nada — SDD líneas 133-148
- [ ] Los ⚠️ "Cannot verify from diff" los resuelve el orquestador (tiene el plan y el contexto cross-task); si es gap real ⇒ vuelta al implementer y re-review — SDD líneas 150-157
- [ ] No pre-juzgar findings al reviewer: prohibido "do not flag", "at most Minor", etc.; el falso positivo se adjudica en el review loop — SDD líneas 168-173
- [ ] El bloque de global constraints del reviewer se copia verbatim del plan/spec (valores y formatos exactos) — SDD líneas 174-180
- [ ] File handoffs: brief, report y review package viajan como FICHEROS (paths en el prompt), nunca pegados en el dispatch — SDD líneas 219-244
- [ ] Review package generado con el BASE registrado antes de despachar al implementer — nunca `HEAD~1` (trunca tareas multi-commit) — SDD línea 136 + líneas 186-188
- [ ] Un dispatch describe UNA tarea (encaje en una línea + brief + interfaces previas + resolución de ambigüedades), no la historia acumulada de la sesión — SDD líneas 189-193 + 229-235
- [ ] Fix subagents para Critical/Important; los Minor se registran en el ledger y el review final los triaja (un roll-up que nadie lee = descarte silencioso) — SDD líneas 194-197
- [ ] Finding plan-mandated o en conflicto con el plan ⇒ decisión del humano: presentar finding + texto del plan, no obedecer ni descartar en silencio — SDD líneas 198-202
- [ ] Todo fix dispatch re-corre los tests que cubren su cambio y lo reporta; el re-review se despacha solo con tests+comando+output presentes — SDD líneas 208-213
- [ ] Findings del review final ⇒ UN solo fix subagent con la lista completa, no un fixer por finding — SDD líneas 214-217
- [ ] Al construir el prompt del reviewer: no añadir directivas open-ended ("check all uses", "run race tests if useful") sin razón concreta task-specific — SDD líneas 164-165
- [ ] Al construir el prompt del reviewer: no pedirle re-correr tests que el implementer ya corrió sobre el mismo código (su reporte ya lleva la evidencia) — SDD líneas 166-167
- [ ] El review final whole-branch también recibe package, generado con `MERGE_BASE` (el commit de arranque de la rama, vía `git merge-base main HEAD`), para que el reviewer final lea un fichero en vez de re-derivar el diff con git — SDD líneas 203-207
- [ ] Ledger durable en fichero (no solo todos en memoria): al cerrar cada tarea, línea `Task N: complete (commits …)`; tras compaction/resume, el ledger y `git log` mandan sobre el recuerdo; jamás re-despachar una tarea que el ledger marca completa — SDD líneas 246-264 + 388-389
- [ ] Nunca empezar implementación en main/master sin consentimiento explícito del usuario — SDD línea 370
- [ ] Nunca despachar múltiples implementers en paralelo sobre el mismo estado (conflictos) — SDD línea 373
- [ ] Selección de modelo explícita en cada dispatch de subagente genérico (un `model` omitido hereda el de la sesión, el más caro) — SDD líneas 115-117; matiz de paridad crítica: NO aplica al rol `reflex:executor`, que lleva modelo fijo (arriba)
- [ ] Turn-count beats token-price: modelo barato solo cuando el texto del plan ES el código (transcripción); tier medio como suelo para reviewers y trabajo desde prosa — SDD líneas 119-125 + OP líneas 41-44

## Movimientos — executing-plans

- [ ] Revisión crítica del plan al cargarlo: concerns al humano ANTES de empezar a ejecutar — EP líneas 18-23
- [ ] Ante blocker, gap crítico del plan o instrucción incomprensible: PARAR y preguntar, no adivinar ni forzar — EP líneas 39-55

## Movimientos — dispatching-parallel-agents

- [ ] Paralelizar SOLO dominios independientes sin estado compartido; un agente por dominio; todos los dispatches en un mismo mensaje — DPA líneas 12-15 + 66-77
- [ ] No paralelizar cuando: fallos relacionados, se necesita el estado completo del sistema, debugging exploratorio, o estado compartido — DPA líneas 129-134
- [ ] Prompt de agente paralelo: scope específico, self-contained, constraints explícitos ("no toques X"), output esperado definido — DPA líneas 58-65 + 87-113
- [ ] Al volver los agentes: leer cada summary, verificar que los fixes no chocan, correr la suite completa, spot-check — DPA líneas 79-85 + 170-176

## Movimientos — orchestrate-personal (el "propio" de la tabla §5.2)

- [ ] Cost pyramid: haiku = transcripción (código literal en el plan, fixes mecánicos de un fichero); sonnet = juicio (integración multi-fichero, refactor, evals); modelo top = SOLO el review final whole-branch + la orquestación misma, una vez por rama — OP líneas 30-40 + 46-48
- [ ] Reviewer-dispatch escalado al riesgo del DIFF, elegido explícitamente: diff literal contenido → barato; wiring de integración → medio; concurrencia/seguridad sutil → top; nunca heredar el modelo del padre — OP líneas 38-40 + framework §5.2 ("reviewer-dispatch escalado al riesgo del diff")
- [ ] Memory packet en cada brief de hijo: 3-5 permalinks de notas canónicas + instrucción de leerlas solo si la tarea lo pide; punteros, no contenido pegado — OP línea 57
- [ ] Brief completeness (map≠territorio): aflorar estándares tácitos como delta en el brief + blindspot pass barato (lee SOLO el brief → devuelve ambigüedades/gaps) antes de despachar trabajo no trivial — OP líneas 67-74
- [ ] Delegate by default: inline solo lo genuinamente trivial (typo, flag de una línea); en la duda, delegar — OP líneas 76-80
- [ ] El hijo se auto-revisa antes de reportar, pero el padre valida SIEMPRE independientemente; nunca auto-aprobar inline — OP líneas 52-54 (el contenido del gate de validación vive en `process:verify`, framework §5.2)
- [ ] Pre-flight recon: verificar las refs del plan (líneas, firmas, símbolos) contra el código real antes de Task 1 — OP líneas 102-105
- [ ] El controller filtra el review y DECIDE el fix; el reviewer detecta, no manda; fixes doc/comment baratos se aplican inline; finding que contradice el plan se escala — OP líneas 113-116
- [ ] Autonomous runs: backlog secuencial; NUNCA push/deploy desatendido (commit local y batch a review); skip de items que requieren decisión del dueño, explicando por qué; documentar toda decisión que normalmente se habría preguntado — OP líneas 118-127
- [ ] Investigate, don't stop: cuando un número no cuadra, recon antes de racionalizarlo — así se encuentra el bug pre-existente en vez de culpar a la propia rama (y se evita convertir un miss real en un pass) — OP líneas 110-112

## DESCARTES (corpus negativo)

- Digraphs dot (SDD líneas 21-37 y 47-83; DPA líneas 18-34): prosa — framework §5.2 "se tira la prosa".
- Example Workflow transcript completo (SDD líneas 272-333) y Real Example/Real-World Impact (DPA líneas 136-161 + 163-168 + 178-185; deja fuera §Verification 170-176, que es movimiento conservado arriba): prosa de ejemplo.
- Secciones Advantages/Efficiency/Quality/Cost (SDD líneas 335-365): prosa justificativa.
- executing-plans como skill/modo separado con su announce (EP líneas 12-14) y su nota "Superpowers works much better with subagents": la bifurcación muere con la fusión — tabla framework §5.2.
- Referencias a `superpowers:using-git-worktrees`, `superpowers:finishing-a-development-branch`, `superpowers:requesting-code-review` como skills (SDD líneas 406-418; EP líneas 65-70): no migradas — framework §5.2 "0 referencias vivas"; el template del review final se absorbe como `orchestrate/reviewer-prompt.md` (spec prep-M3 §5.3).
- Dispatch default `general-purpose` con `model` obligatorio del implementer-prompt de SDD (implementer-prompt.md líneas 6-9) para el rol ejecutor: sustituido por `reflex:executor` sin model — PARIDAD CRÍTICA arriba.
- "Do NOT reinvent orchestration… The engine is superpowers:subagent-driven-development" (OP líneas 8-11): el layering sobre superpowers muere; process:orchestrate ES la fusión.
- Nota "Fuente canónica … si esta skill y la nota divergen, manda la nota" (OP línea 13): la doctrina genérica vive ahora en la skill/reference files, no en la KB — framework §5.1 ("la carne va ahí, NO en la KB"); los deltas personales siguen en la KB vía overlay (spec prep-M3 §3.4).
