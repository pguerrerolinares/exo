# Gold — process:plan (paridad de movimientos)

Fuente: `~/.claude/plugins/cache/claude-plugins-official/superpowers/6.1.1/skills/writing-plans/SKILL.md` (174 líneas).
Uso: ver `evals/prep-m3/README.md`.

## Movimientos

- [ ] Premisa del lector: el plan asume un ingeniero hábil pero con CERO contexto del codebase y el dominio — documentar ficheros a tocar, código, testing, docs — SKILL.md §Overview (líneas 10-12)
- [ ] Guardar planes en `docs/superpowers/plans/YYYY-MM-DD-<feature-name>.md` (preferencia del usuario overridea) — líneas 18-19
- [ ] Scope check: spec con múltiples subsistemas independientes ⇒ planes separados, cada uno produce software funcionando y testeable por sí solo — líneas 21-23
- [ ] File structure antes de las tareas: mapear qué ficheros se crean/modifican y la responsabilidad de cada uno; unidades con límites claros; ficheros que cambian juntos viven juntos — líneas 25-34
- [ ] Task right-sizing: la tarea es la unidad mínima con ciclo de test propio y digna del gate de un reviewer fresco; setup/scaffolding se pliegan a la tarea que los necesita — líneas 36-43
- [ ] Pasos bite-sized de una acción (2-5 min): failing test → verlo fallar → implementación mínima → verlo pasar → commit — líneas 45-52
- [ ] Header obligatorio del plan: goal (1 frase), architecture (2-3), tech stack, y Global Constraints con valores exactos copiados verbatim de la spec — líneas 54-77
- [ ] Pointer "For agentic workers" en el header: indica la skill de ejecución (delta: `process:orchestrate`) y que los pasos usan checkbox (`- [ ]`) como tracking — línea 61
- [ ] Estructura de tarea: Files con paths exactos (Create/Modify/Test) + bloque Interfaces (Consumes/Produces con firmas exactas — es como un implementer que solo ve su tarea aprende los nombres y tipos vecinos) — líneas 79-93
- [ ] Regla no-placeholders: nada de "TBD/TODO", "add appropriate error handling", "write tests for the above" sin código, "similar to Task N" (se repite el código), pasos sin el cómo, ni referencias a tipos/funciones no definidos en ninguna tarea — líneas 128-136
- [ ] Recordatorio operativo: paths exactos siempre, código completo en cada paso que cambia código, comandos exactos con output esperado, DRY/YAGNI/TDD/frequent commits — línea 10 + líneas 138-142
- [ ] Self-review contra la spec con ojos frescos (checklist propio, no dispatch): cobertura de spec (¿cada requisito tiene tarea?), placeholder scan, consistencia de tipos/firmas entre tareas; fix inline, y si falta tarea se añade — líneas 144-154
- [ ] Handoff de ejecución al terminar el plan — líneas 156-175, con el delta de diseño de la spec prep-M3 §5.2: destino único `process:orchestrate` (la bifurcación subagent-driven vs executing-plans muere con la fusión de la tabla framework §5.2)

## DESCARTES (corpus negativo)

- "Announce at start: 'I'm using the writing-plans skill…'" (línea 14): ritual de anuncio — framework §5.2 "se tira … los gritos".
- Referencia a `superpowers:using-git-worktrees` (línea 16): skill no migrada — framework §5.2 "No se migra lo no usado … using-git-worktrees … 0 referencias vivas".
- Elección de dos modos de ejecución en el handoff (líneas 158-175): en process hay UN orchestrate — tabla framework §5.2 (orchestrate absorbe subagent-driven-development Y executing-plans).
- `plan-document-reviewer-prompt.md`: sin referencia viva desde el body 6.1.1 (el self-review es checklist propio, línea 146 "This is a checklist you run yourself — not a subagent dispatch").
