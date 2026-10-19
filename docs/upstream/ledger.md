# upstream-sync — ledger

Estado versionado del bot que porta a exo lo que cambia en obra/superpowers. Diseño: `docs/superpowers/specs/2026-09-29-upstream-sync-design.md`. Prompt del bot: `docs/upstream/sync-prompt.md`.

upstream_tag: v7.0.0

verdad_sello: —

## Mapeo fichero → fichero

Rutas relativas a `skills/` de superpowers y a `plugins/exo/skills/` de exo. Las skills SKILL.md de exo son destilación, no copia: un cambio upstream en el SKILL.md se traduce a la prosa exo, no se pega.

| upstream | exo | nota |
|---|---|---|
| brainstorming/SKILL.md | brainstorm/SKILL.md | destilado |
| brainstorming/spec-document-reviewer-prompt.md | brainstorm/SKILL.md | eliminado upstream en 7.0.0 (#2463); sustituido por el builder check |
| brainstorming/builder-check-prompt.md | brainstorm/SKILL.md | destilado en el builder check |
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
| dispatching-parallel-agents/SKILL.md | orchestrate/SKILL.md | destilado (paralelismo por ola); detalle en orchestrate/olas.md |
| dispatching-parallel-agents/SKILL.md | orchestrate/olas.md | destilado |
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
- `writing-skills/*`, `using-superpowers/*`, `finishing-a-development-branch/*` y `diagnosing-superpowers/*`: skills sin contraparte en exo, fuera del catálogo destilado (decisión del mantenedor 2026-10-14)
- `writing-plans/plan-document-reviewer-prompt.md`: eliminado upstream

## Sin equivalente en exo

Ficheros upstream sin contraparte en exo. Un cambio ahí se triagea como `no aplica`, citando esta sección, salvo las excepciones que se indican en cada línea.

- `brainstorming/visual-companion.md` y `brainstorming/scripts/*`: exo descartó el visual companion (spec prep-M3).
- `subagent-driven-development/re-review-prompt.md`: exo no tiene prompt de re-review.
- `using-git-worktrees/SKILL.md`: exo no tiene skill de worktrees (D1).
- `systematic-debugging/test-academic.md`: escenario de test de la skill upstream, sin contraparte en exo.
- `receiving-code-review/SKILL.md`: exo no tiene skill de recepción de review. Una regla de ahí sobre cómo el padre trata los hallazgos del reviewer se evalúa contra `orchestrate/SKILL.md` y `orchestrate/olas.md`; el resto, `no aplica`.
- `executing-plans/SKILL.md` y `executing-plans/scripts/*`: el modo de ejecución inline no se porta (D8). Una regla de review o de ledger que upstream ponga ahí y sea independiente del modo inline se evalúa contra `orchestrate/` (reviewer-prompt.md, SKILL.md u olas.md); no se descarta por el fichero.

Todo fichero upstream que no salga en ninguna de las tres secciones anteriores es `triage=duda` con `motivo=mapeo: <fichero upstream>`.

## Divergencias deliberadas

Un cambio upstream que choque con una de estas filas se triagea como `no aplica`, citando el id. D3 y D5 describen la forma que debe tener un porte, no el estado actual del fichero.

| id | skill exo | qué diverge | motivo | fuente |
|---|---|---|---|---|
| D1 | orchestrate | Sin skill de worktrees ni paso "asegura workspace aislado" al inicio; aislamiento por ola (worktree por tarea en olas de ≥2) + red line de main | Decisión del mantenedor 2026-09-30 | plugins/exo/skills/orchestrate/olas.md § Ejecutar una ola |
| D2 | debug | La verificación del fix va en prosa en la Fase 4; sin puntero explícito a exo:verify | La regla de verificar ya vive en verify, executor.md y exo-recall.sh | plugins/exo/skills/debug/SKILL.md (Fase 4) |
| D3 | orchestrate | Forma de porte (no estado actual): si se prohíbe despachar subagentes, el executor pierde la herramienta Agent (contrato mecánico en frontmatter); la prohibición de despachar subagentes va en prosa solo en el prompt del reviewer | Lo mecánico no depende de la prosa (Fallo silencioso, ley 3) | plugins/exo/agents/executor.md |
| D4 | orchestrate | Sin guía de esperas acotadas ni polling de subagentes | Específico de harnesses con wait por polling; Claude Code notifica al terminar | — |
| D5 | orchestrate | Forma de porte (no estado actual): si el pre-flight produce tabla, copia al ledger las filas de pares que da task-dag y añade solo una fila de coherencia interna por tarea; el texto vive en olas.md, no en SKILL.md | task-dag ya calcula Files/Interfaces compartidos; presupuesto de SKILL.md | plugins/exo/skills/orchestrate/scripts/task-dag |
| D6 | orchestrate | Una tarea por dispatch; el agrupado de trabajo trivial se hace al planificar, no al despachar | plan/SKILL.md pliega lo trivial en la tarea que lo necesita | plugins/exo/skills/plan/SKILL.md § Antes de las tareas |
| D7 | orchestrate | La prosa invoca los scripts directamente (sin `bash` delante) | exo solo se distribuye por git; git conserva modos; en Git Bash la ejecutabilidad la decide el shebang | scripts/test-exec-bit.sh |
| D8 | plan, orchestrate | Sin modo de ejecución inline; handoff único a exo:orchestrate | Orquestador limpio: el padre integra, no implementa | plugins/exo/skills/orchestrate/SKILL.md |
| D9 | orchestrate/scripts/review-package | La sección MUTACIÓN (acotada al diff, timeout por mutante, `EXO_MUTATION_EXCLUDE`, muestreo; exo 1.5.0-1.5.2) es propia; un porte nunca la pisa, solo el resto del script | Instrumento propio del pipeline A+ | plugins/exo/skills/orchestrate/olas.md § Mutación |
| D10 | debug | Referenciar systematic-debugging OK, depender NO | Decisión de diseño previa | — |
| D11 | brainstorm | Sin salida «salta las preguntas» en el primer mensaje | brainstorm se invoca a propósito; lo trivial ya lo cubre «Tamaño del trabajo» (cambio pequeño en chat). Decisión del mantenedor 2026-10-14 | plugins/exo/skills/brainstorm/SKILL.md § Tamaño del trabajo |
| D12 | brainstorm | Se mantiene «prefiere multiple choice»; sin regla de pregunta abierta primero ni menú solo si el usuario se atasca | Decisión del mantenedor 2026-10-14 | plugins/exo/skills/brainstorm/SKILL.md § Proceso |

## Decisiones de doctrina (portar)

Decisiones del mantenedor (2026-09-30), distintas de las Divergencias: no son "no portar", son doctrina de exo que un cambio upstream puede implementar. Un cambio que las implementa es `aplica`/`parcial`, no `duda`.

- **Rule and continue.** Ante conflictos, ambigüedades o defectos del plan, el orquestador decide, registra `Ruling: <decisión> — <porqué> — <coste si está mal>` en el ledger y sigue. Solo para ante (1) una operación irreversible o destructiva, (2) una acción sensible de seguridad, (3) un efecto fuera del worktree que las normas mandan preguntar (merge, push a rama compartida, publicar), (4) un plan tan roto que todo camino es adivinar.
- **Lo que el reviewer no juzgó.** El reviewer lista lo que dejó sin juzgar ("Declined to judge") y el orquestador convierte cada línea en un Ruling.
- **tdd conserva las refutaciones de "tests después"** como filas de la tabla de rationalizations: upstream midió que quitarlas empeora el test-first.
- **Brainstorm por tamaño del trabajo.** Las mejoras del flujo que distinguen el tamaño (exploración / acotado / arquitectónico) son compatibles con exo y se portan.

## Filas

`triage` ∈ {aplica, parcial, ya cubierto, no aplica, duda}, siempre no vacío (una fila `pendiente` por tope lleva su triage real). `estado` ∈ {propuesto, portado, rechazado, pendiente, —}; `—` para no aplica y ya cubierto.

`motivo` en `pendiente`: `tope`, `gate: <causa>`, `respondida` (el mantenedor contestó una `duda`) o, en `duda`, la pregunta (para un fichero sin mapeo: `mapeo: <fichero upstream>`). Nunca `|` dentro de `motivo`: usa `/`.

| PR | skill | triage | estado | motivo | hash |
|---|---|---|---|---|---|
| #2136 | orchestrate | aplica | portado | — | f0cf912 |
| #2089 | orchestrate | aplica | portado | — | 8a4f832 |
| #2059 | orchestrate | aplica | portado | — | b833219 |
| #2110 | tdd | parcial | portado | — | 34e1fd9 |
| #2080 | orchestrate | aplica | portado | — | bfef11e |
| #1943 | orchestrate | aplica | portado | — | 5b4ca55 |
| #2138 | orchestrate | aplica | portado | — | f6d91e7 |
| #1998 | orchestrate | aplica | portado | — | 54bc199 |
| #2086 | orchestrate | parcial | portado | — | b78b279 |
| #2258 | brainstorm | aplica | portado | — | 8952629 |
| #2319 | orchestrate | parcial | portado | — | 001764f |
| #2077 | orchestrate | aplica | portado | — | a8d0c5b |
| #2063 | brainstorm | ya cubierto | — | cubierto por #2463: brainstorm/SKILL.md «Tamaño del trabajo» (decisión de Paul 2026-10-14) | — |
| #1934 | tdd | aplica | portado | — | 719a1fd |
| #1934 | writing-skills | no aplica | — | Excluidos: `writing-skills/*` (decisión de Paul 2026-10-14) | — |
| #1934 | orchestrate | no aplica | — | solo elimina: exo no tiene esas secciones (Advantages / Integration) | — |
| #1934 | debug | no aplica | — | solo elimina: exo no tiene social proof ni Real-World Impact (D2 aparte) | — |
| #1934 | verify | no aplica | — | solo elimina: exo no tiene esas secciones | — |
| #1934 | brainstorm | no aplica | — | solo elimina: exo no tiene Key Principles duplicados (brainstorm/SKILL.md:132-134) | — |
| #1934 | plan | no aplica | — | solo elimina: exo no tiene recap Remember | — |
| #1933 | finishing-a-development-branch | no aplica | — | Excluidos: `finishing-a-development-branch/*` (decisión de Paul 2026-10-14) | — |
| #2024 | finishing-a-development-branch | no aplica | — | Excluidos: `finishing-a-development-branch/*` (decisión de Paul 2026-10-14) | — |
| #1805 | writing-skills | no aplica | — | Excluidos: `writing-skills/*` (decisión de Paul 2026-10-14) | — |
| #1959 | writing-skills | no aplica | — | Excluidos: `writing-skills/*` (decisión de Paul 2026-10-14) | — |
| #2301 | writing-skills | no aplica | — | Excluidos: `writing-skills/*` (decisión de Paul 2026-10-14) | — |
| #2010 | using-superpowers | no aplica | — | Excluidos: `using-superpowers/*` (decisión de Paul 2026-10-14) | — |
| #2025 | using-superpowers | no aplica | — | Excluidos: `using-superpowers/*` (decisión de Paul 2026-10-14) | — |
| #2317 | using-superpowers | no aplica | — | Excluidos: `using-superpowers/*` (decisión de Paul 2026-10-14) | — |
| #2320 | using-superpowers | no aplica | — | Excluidos: `using-superpowers/*` (decisión de Paul 2026-10-14) | — |
| #2236 | diagnosing-superpowers | no aplica | — | Excluidos: `diagnosing-superpowers/*` (decisión de Paul 2026-10-14) | — |
| #2287 | diagnosing-superpowers | no aplica | — | Excluidos: `diagnosing-superpowers/*` (decisión de Paul 2026-10-14) | — |
| #2333 | writing-plans | no aplica | — | Excluidos: `writing-plans/plan-document-reviewer-prompt.md` (decisión de Paul 2026-10-14) | — |
| #1932 | orchestrate | no aplica | — | D1 | — |
| #1932 | debug | no aplica | — | D2 | — |
| #2134 | orchestrate | no aplica | — | D7 | — |
| #2301 | orchestrate | no aplica | — | D7 | — |
| #2301 | debug | no aplica | — | D7 | — |
| #2061 | orchestrate | no aplica | — | D4 | — |
| #2078 | orchestrate | no aplica | — | D6 | — |
| #2318 | orchestrate | no aplica | — | D8 | — |
| #2318 | plan | no aplica | — | D8 | — |
| #2258 | plan | aplica | propuesto | — | 8790cf3 |
| #2006 | brainstorm | no aplica | — | sin equivalente: brainstorming/visual-companion.md | — |
| #1959 | brainstorm | no aplica | — | sin equivalente: brainstorming/visual-companion.md | — |
| #2133 | orchestrate | ya cubierto | — | — | — |
| #1935 | tdd | ya cubierto | — | — | — |
| #2333 | plan | ya cubierto | — | — | — |
| #2319 | plan | parcial | portado | — | 001764f |
| #2086 | plan | ya cubierto | — | — | — |
| #2463 | brainstorm | parcial | portado | — | f74f7b4 |
| #2463 | brainstorm | parcial | portado | — | f74f7b4 |
| #2463 | brainstorm | ya cubierto | — | mapeo confirmado por Paul 2026-10-14: builder-check-prompt.md → brainstorm/SKILL.md | — |
| #2482 | brainstorm | aplica | portado | — | 987ac71 |
| #2483 | brainstorm | no aplica | — | D11: exo no tiene la salida «salta las preguntas»; el bullet que quita no existe en exo | — |
| #2399 | orchestrate | aplica | portado | — | db0eddd |
| #2489 | orchestrate | aplica | portado | — | 2331afa |
| #2364 | brainstorm | no aplica | — | sin equivalente: brainstorming/scripts/* | — |
| #2388 | orchestrate | no aplica | — | sin equivalente: executing-plans/scripts/task-done (D8) | — |
| #2361 | orchestrate | no aplica | — | solo cambia un ejemplo de BASE_SHA que exo no tiene | — |
| #2380 | debug | no aplica | — | solo cambia un ejemplo de diagnóstico que exo no tiene | — |
