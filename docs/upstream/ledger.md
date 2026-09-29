# upstream-sync — ledger

Estado versionado del bot que porta a exo lo que cambia en obra/superpowers. Diseño: `docs/superpowers/specs/2026-09-29-upstream-sync-design.md`. Prompt del bot: `docs/upstream/sync-prompt.md`.

upstream_tag: v6.1.1

## Mapeo fichero → fichero

Rutas relativas a `skills/` de superpowers y a `plugins/exo/skills/` de exo. Las skills SKILL.md de exo son destilación, no copia: un cambio upstream en el SKILL.md se traduce a la prosa exo, no se pega.

| upstream | exo | nota |
|---|---|---|
| brainstorming/SKILL.md | brainstorm/SKILL.md | destilado |
| writing-plans/SKILL.md | plan/SKILL.md | destilado |
| writing-plans/SKILL.md | plan/plan-template.md | cabecera de atribución (superpowers 6.4.2) |
| subagent-driven-development/SKILL.md | orchestrate/SKILL.md | destilado; parte vive en orchestrate/olas.md |
| subagent-driven-development/SKILL.md | orchestrate/olas.md | destilado |
| subagent-driven-development/implementer-prompt.md | orchestrate/implementer-prompt.md | cabecera de atribución |
| subagent-driven-development/task-reviewer-prompt.md | orchestrate/reviewer-prompt.md | cabecera de atribución |
| subagent-driven-development/scripts/review-package | orchestrate/scripts/review-package | la sección MUTACIÓN es propia (D9) |
| subagent-driven-development/scripts/sdd-workspace | orchestrate/scripts/sdd-workspace | cabecera de atribución |
| subagent-driven-development/scripts/task-brief | orchestrate/scripts/task-brief | cabecera de atribución |
| requesting-code-review/code-reviewer.md | orchestrate/reviewer-prompt.md | template único de exo para la review por tarea y la final whole-branch |
| requesting-code-review/SKILL.md | orchestrate/SKILL.md | cuándo y cómo pedir review vive en orchestrate (review por tarea + final); el prompt, en reviewer-prompt.md |
| test-driven-development/SKILL.md | tdd/SKILL.md | destilado |
| test-driven-development/writing-good-tests.md | tdd/anti-patterns.md | cabecera de atribución (superpowers 6.4.2) |
| systematic-debugging/SKILL.md | debug/SKILL.md | destilado; referenciar sí, depender no (D10) |
| systematic-debugging/root-cause-tracing.md | debug/techniques.md | destilado; resumen en debug/SKILL.md Puerta 1 |
| systematic-debugging/defense-in-depth.md | debug/techniques.md | destilado |
| systematic-debugging/condition-based-waiting.md | debug/techniques.md | cabecera de atribución |
| systematic-debugging/condition-based-waiting-example.ts | debug/techniques.md | solo la técnica; el ejemplo .ts no se porta |
| verification-before-completion/SKILL.md | verify/SKILL.md | destilado |

## Excluidos

Ficheros upstream que no se portan nunca, ni se triagean.

- `systematic-debugging/CREATION-LOG.md`
- `systematic-debugging/test-pressure-1.md`, `test-pressure-2.md`, `test-pressure-3.md` (patrón `test-pressure-*.md`)
- `systematic-debugging/find-polluter.sh`

## Sin equivalente en exo

Ficheros upstream sin contraparte en exo. Un cambio ahí se triagea como `no aplica`, citando esta sección, salvo las excepciones que se indican en cada línea.

- `subagent-driven-development/re-review-prompt.md`: exo no tiene prompt de re-review.
- `using-git-worktrees/SKILL.md`: exo no tiene skill de worktrees (D1).
- `systematic-debugging/test-academic.md`: escenario de test de la skill upstream, sin contraparte en exo.
- `receiving-code-review/SKILL.md`: exo no tiene skill de recepción de review. Una regla de ahí sobre cómo el padre trata los hallazgos del reviewer se evalúa contra `orchestrate/SKILL.md` y `orchestrate/olas.md`; el resto, `no aplica`.
- `executing-plans/SKILL.md` y `executing-plans/scripts/*`: el modo de ejecución inline no se porta (D8). Una regla de review o de ledger que upstream ponga ahí y sea independiente del modo inline se evalúa contra `orchestrate/` (reviewer-prompt.md, SKILL.md u olas.md); no se descarta por el fichero.

Todo fichero upstream que no salga en ninguna de las tres secciones anteriores es `duda: mapeo`.

## Divergencias deliberadas

Un cambio upstream que choque con una de estas filas se triagea como `no aplica`, citando el id. D3 y D5 describen la forma que debe tener un porte, no el estado actual del fichero.

| id | skill exo | qué diverge | motivo | fuente |
|---|---|---|---|---|
| D1 | orchestrate | Sin skill de worktrees ni paso "asegura workspace aislado" al inicio; aislamiento por ola (worktree por tarea en olas de ≥2) + red line de main | Decisión de Paul 2026-09-30 | plugins/exo/skills/orchestrate/olas.md § Ejecutar una ola |
| D2 | debug | La verificación del fix va en prosa en la Fase 4; sin puntero explícito a exo:verify | La regla de verificar ya vive en verify, executor.md y exo-recall.sh | plugins/exo/skills/debug/SKILL.md (Fase 4) |
| D3 | orchestrate | Forma de porte (no estado actual): si se prohíbe despachar subagentes, el executor pierde la herramienta Agent (contrato mecánico en frontmatter); la prohibición de despachar subagentes va en prosa solo en el prompt del reviewer | Lo mecánico no depende de la prosa (Fallo silencioso, ley 3) | plugins/exo/agents/executor.md |
| D4 | orchestrate | Sin guía de esperas acotadas ni polling de subagentes | Específico de harnesses con wait por polling; Claude Code notifica al terminar | — |
| D5 | orchestrate | Forma de porte (no estado actual): si el pre-flight produce tabla, copia al ledger las filas de pares que da task-dag y añade solo una fila de coherencia interna por tarea; el texto vive en olas.md, no en SKILL.md | task-dag ya calcula Files/Interfaces compartidos; presupuesto de SKILL.md | plugins/exo/skills/orchestrate/scripts/task-dag |
| D6 | orchestrate | Una tarea por dispatch; el agrupado de trabajo trivial se hace al planificar, no al despachar | plan/SKILL.md pliega lo trivial en la tarea que lo necesita | plugins/exo/skills/plan/SKILL.md § Antes de las tareas |
| D7 | orchestrate | La prosa invoca los scripts directamente (sin `bash` delante) | exo solo se distribuye por git; git conserva modos; en Git Bash la ejecutabilidad la decide el shebang | scripts/test-exec-bit.sh |
| D8 | plan, orchestrate | Sin modo de ejecución inline; handoff único a exo:orchestrate | Orquestador limpio: el padre integra, no implementa | plugins/exo/skills/orchestrate/SKILL.md |
| D9 | orchestrate/scripts/review-package | La sección MUTACIÓN (acotada al diff, timeout por mutante, `EXO_MUTATION_EXCLUDE`, muestreo; exo 1.5.0-1.5.2) es propia; un porte nunca la pisa, solo el resto del script (p. ej. guardas de rango) | Instrumento propio del pipeline A+ | plugins/exo/skills/orchestrate/olas.md § Mutación |
| D10 | debug | Referenciar systematic-debugging OK, depender NO | Decisión de diseño previa | — |

## Filas

`triage` ∈ {aplica, parcial, ya cubierto, no aplica, duda}, siempre no vacío (una fila `pendiente` por tope lleva su triage real). `estado` ∈ {propuesto, portado, rechazado, pendiente, —}; `—` para no aplica y ya cubierto.

| PR | skill | triage | estado | motivo | hash |
|---|---|---|---|---|---|
