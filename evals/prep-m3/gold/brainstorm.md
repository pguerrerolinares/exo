# Gold — process:brainstorm (paridad de movimientos)

Fuente: `~/.claude/plugins/cache/claude-plugins-official/superpowers/6.1.1/skills/brainstorming/SKILL.md` (159 líneas).
Uso: ver `evals/prep-m3/README.md`. Cada ítem = un movimiento que la skill
implementada DEBE conservar (presente/ausente, sin juicio estético).

## Movimientos

- [ ] Explorar el contexto del proyecto (ficheros, docs, commits recientes) antes de preguntar nada — SKILL.md §Checklist ítem 1 (línea 24) + §The Process (línea 67: "Check out the current project state first")
- [ ] Preguntas de una en una; si un tema necesita más, se parte en varias preguntas; el objetivo es entender purpose/constraints/success criteria — §Checklist ítem 3 (línea 26: "one at a time, understand purpose/constraints/success criteria") + línea 72 ("Only one question per message") + línea 73 ("Focus on understanding: purpose, constraints, success criteria") + §Key Principles (línea 135)
- [ ] Preferir multiple choice cuando sea posible — línea 71 + §Key Principles (línea 136)
- [ ] Detectar scope multi-subsistema ANTES de refinar detalles y descomponer en sub-proyectos, cada uno con su ciclo spec→plan→implementación — líneas 68-69
- [ ] Proponer 2-3 enfoques con trade-offs, liderando con la recomendación y su porqué — §Checklist ítem 4 (línea 27) + líneas 76-79
- [ ] Diseño antes de implementación: no escribir código ni invocar skills de implementación hasta que el usuario aprueba el diseño — HARD-GATE (líneas 12-14, destilado sin el grito)
- [ ] Anti-patrón "too simple to need a design": todo proyecto pasa por diseño; el diseño puede ser corto, pero se presenta y se aprueba — líneas 16-18
- [ ] Presentar el diseño por secciones escaladas a su complejidad, validando después de cada sección — línea 28 + líneas 82-87
- [ ] Cobertura del diseño: arquitectura, componentes, data flow, error handling, testing — línea 86
- [ ] Diseño para aislamiento: unidades con un propósito claro, interfaces bien definidas, comprensibles y testeables por separado — líneas 89-94
- [ ] En codebases existentes: seguir patrones actuales; mejoras targeted solo si afectan al trabajo; no proponer refactoring no relacionado — líneas 96-100
- [ ] Escribir la spec validada a `docs/superpowers/specs/YYYY-MM-DD-<topic>-design.md` (preferencia del usuario overridea el default) y commitearla — línea 29 + líneas 104-109
- [ ] Self-review de la spec: placeholders, consistencia interna, scope, ambigüedad — fix inline, sin re-review — líneas 111-119
- [ ] Gate de review del usuario: pedirle que revise la spec escrita y esperar su respuesta antes de seguir — líneas 121-126
- [ ] Estado terminal = invocar la skill de planning (en process: `process:plan`), ninguna otra — línea 61 ("The terminal state is invoking writing-plans") + líneas 128-131
- [ ] YAGNI: quitar features innecesarias de todos los diseños — §Key Principles (línea 137)

## DESCARTES (corpus negativo — presencia = fallo de paridad)

- Visual companion completo (§Visual Companion líneas 142-159 + `visual-companion.md` + `scripts/`): maquinaria opcional de presentación en browser, no un movimiento de diseño — framework §5.2 "extraer el movimiento esencial y reescribirlo thin".
- `spec-document-reviewer-prompt.md`: sin referencia viva desde el body 6.1.1 (el self-review es inline, línea 119 "No need to re-review — just fix and move on").
- Digraph dot del proceso (líneas 36-59): prosa redundante con el checklist — §5.2 "se tira la prosa".
- Gritos y gates dogmáticos: `<HARD-GATE>` en mayúsculas (líneas 12-14), "You MUST create a task for each of these items" (línea 22) — §5.2 "se tira … los gritos y los gates dogmáticos" (el movimiento diseño-antes-de-código se conserva como regla, arriba).
