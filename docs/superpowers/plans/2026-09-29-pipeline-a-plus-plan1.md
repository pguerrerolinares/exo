# Plan 1 — pipeline A+: prosa de skills (contrato)

> For agentic workers: ejecución con `exo:orchestrate`. Este plan ya sigue el formato A+:
> contrato sin cuerpos de código. Escribes el texto tú, con criterio, dentro del contrato.

**Goal:** reescribir las skills del pipeline de desarrollo de exo según la spec A+. El plan pasa a ser un contrato, el executor gana libertad con una escalera anti over-engineering y reglas de comentarios y de qué no testear, y orchestrate ejecuta en olas.

**Architecture:** solo prosa Markdown en `plugins/exo/`. No se crean scripts; eso es el plan 2. Mientras `task-dag` no exista, orchestrate describe las olas y el orquestador las calcula a mano con la misma regla.

**Spec:** `docs/superpowers/specs/2026-09-29-pipeline-a-plus-design.md`. Es la fuente de verdad: §3 arquitectura, §4 tabla "Plan 1", §5 errores.

**Fuentes upstream** (MIT, clonadas en el scratchpad de la sesión):
- `SP=/tmp/claude-1000/-home-paul-Documentos-proyectos-exo/1ba28911-00d5-417b-b399-e72e68644522/scratchpad/sp` (superpowers v6.4.2)
- `PT=/tmp/claude-1000/-home-paul-Documentos-proyectos-exo/1ba28911-00d5-417b-b399-e72e68644522/scratchpad/ponytail/ponytail` (ponytail)

**Global Constraints:**
- Castellano con términos técnicos en inglés, en el mismo tono y densidad que las skills actuales. Se destila, no se traduce en bloque: las skills de exo son cortas.
- `SKILL.md` de cada skill ≤ ~4 KB (hoy rondan 2,5-3,8 KB). El detalle va a ficheros auxiliares, que se leen bajo demanda.
- Vocabulario compartido entre tareas, que debe usarse literal:
  - campos de tarea del plan: `Files`, `Interfaces` (`Consumes`/`Produces`), `Tests`, `Review Focus`, `Notas`;
  - desvío del executor: línea `Ruling: T<n> — <qué cambia> — <por qué>` en el ledger `.superpowers/sdd/progress.md`;
  - sección de mutación en el package: `MUTACIÓN:` (valores `<score> + supervivientes` | `no disponible (<motivo>)` | `parcial (<motivo>)`);
  - tags de over-engineering: `delete|stdlib|native|yagni|shrink`, con cierre `net: -N lines possible`.
- Regla de olas: dos tareas comparten ola si sus `Files` son disjuntos y ninguna consume, directa o transitivamente, algo que produce la otra.
- No se tocan `brainstorm`, `verify`, `debug`, `document`, `distill` ni `recon-first`.
- Sin tests de strings sobre skills (spec §6).

## Olas

- Ola 1, en paralelo: T1, T2, T3, T4. Sus `Files` son disjuntos y todas consumen solo el vocabulario de arriba.
- Ola 2: T5 (wiring: README, LICENSES, plugin.json), que consume T1-T4.

## Task 1 — skill plan como contrato

- **Files:** Modify `plugins/exo/skills/plan/SKILL.md` y `plugins/exo/skills/plan/plan-template.md`.
- **Interfaces:** Mantén el encabezado de tarea `### Task N: <nombre>` (lo parsea `orchestrate/scripts/task-brief`: `^#+ Task N`). Produces el formato de tarea, con los campos del vocabulario, en formato parseable. Cada campo es una línea `**Files:**` / `**Interfaces:**` con sub-bullets `Create|Modify|Test: <path>` y `Consumes|Produces: <firma>`, para que `task-dag` (plan 2) los lea.
- **Contenido exigido** (spec §4, fila plan): "No-placeholders" pasa a "Qué contiene una tarea".
  - Test = nombre + aserción con los valores de la spec. La aserción declara el fallo que caza.
  - Código = firma, fichero y valores. El executor escribe el cuerpo. `Notas` solo lleva un algoritmo que la firma y los tests no determinan.
  - El lector es un ingeniero capaz, no uno con cero contexto.
  - Tarea = unidad verificable, diff esperado ≲ 400 líneas. Las triviales se pliegan en la que las necesita. El wiring va en una tarea por ola.
  - Review Focus: ≤5 inputs que la spec implica y nadie nombra.
  - Sección "Olas" con la regla de olas.
  - Self-review: cobertura, placeholders, tipos y **proporción** ("un plan más largo que el código que describe ya es el código").
  - Handoff único a `exo:orchestrate`.
- **Referencia:** `$SP/skills/writing-plans/SKILL.md` (What a Step Contains, Review Focus, Self-Review/Proportion).
- **Aceptación:** la plantilla no obliga a ningún bloque de código. Este mismo fichero de plan cumple el formato nuevo, y sirve de ejemplo vivo.

## Task 2 — orchestrate: olas, Ruling, mutación en review, pase de over-engineering

- **Files:** Modify `plugins/exo/skills/orchestrate/SKILL.md`, `.../implementer-prompt.md` y `.../reviewer-prompt.md`.
- **Interfaces:** Consumes el vocabulario (Ruling, MUTACIÓN, tags) y el formato de tarea de T1 (solo por nombres de campo).
- **Contenido exigido** (spec §3 y §5; fila orchestrate, implementer y reviewer):
  - **SKILL.md**:
    - Olas por la regla. Mientras no exista `scripts/task-dag`, el orquestador las calcula; si es ambiguo, secuencial con `DAG: secuencial (<motivo>)` en el ledger.
    - Ola de ≥2 tareas: un worktree por tarea y todos los dispatches en un mensaje.
    - Merge de ola y después suite completa. Si queda roja, no se abre la siguiente ola y va un fix dispatch.
    - Conflicto de merge: executor de integración y, si no se resuelve, BLOCKED. Nunca `-X ours/theirs` a ciegas.
    - Pipeline de reviews dentro de la ola.
    - `Ruling:`.
    - Registro de métricas de la serie (spec §6) en el ledger al cerrar la rama.
    - Se conservan **intactos** la PARIDAD CRÍTICA, los estados, las red lines y el ledger.
  - **implementer-prompt**: el executor escribe el cuerpo de tests y código; si se desvía, `Ruling:`.
  - **reviewer-prompt**:
    - lee la sección `MUTACIÓN:` si existe; si no hay, "cannot verify";
    - pase de over-engineering separado del de corrección, con los tags;
    - check de comentarios: la proporción del diff no supera la del fichero, y cada comentario contiene un porqué; si no, finding Minor;
    - "reasonable user" para comportamientos que la spec no menciona.
- **Referencia:** `$SP/skills/executing-plans/SKILL.md` (Rulings), `$SP/skills/requesting-code-review/code-reviewer.md` y `$PT/skills/ponytail-review/SKILL.md`.
- **Review Focus:** que no se pierda la línea de PARIDAD CRÍTICA (`subagent_type: exo:executor`, sin `model`). Hay un gold que la exige: `evals/prep-m3/gold/orchestrate.md`.

## Task 3 — executor: escalera, comentarios, qué no testear

- **Files:** Modify `plugins/exo/agents/executor.md`.
- **Interfaces:** ninguna.
- **Restricción dura:** `plugins/exo/scripts/compose-inject.sh` inyecta en **todos los demás subagentes** solo los primeros ~800 B del cuerpo de este fichero (`cap_lines 800`, un presupuesto total). **No cambies ni reordenes los bullets existentes.** Añade el contenido nuevo DESPUÉS, en sección propia. El frontmatter (`model: sonnet`) no se toca.
- **Contenido exigido** (spec §4, fila executor):
  - Escalera: ¿hace falta? → ¿existe en el codebase? → stdlib/plataforma → dependencia instalada → mínimo código. Con la salvedad literal: la escalera acorta la solución, nunca la lectura; el cambio mínimo en el sitio equivocado es otro bug.
  - Sin abstracciones especulativas.
  - Regla de comentarios con el test del borrado.
  - Qué no testear: one-liners, glue, constantes, texto.
  - "Si puedes elegir un default razonable, elígelo y dilo; no te pares."
  - Salida: el resultado primero y como mucho 3 líneas de qué se omitió y cuándo añadirlo.
- **Referencia:** `$PT/skills/ponytail/SKILL.md`, reescrito como reglas operativas y sin la persona "lazy".
- **Aceptación:** `bash plugins/exo/scripts/test-compose-inject.sh` sigue verde. Enseña el output.

## Task 4 — tdd: writing-good-tests y suite = verde

- **Files:** Modify `plugins/exo/skills/tdd/anti-patterns.md` y `plugins/exo/skills/tdd/SKILL.md`.
- **Interfaces:** ninguna.
- **Contenido exigido:**
  - `anti-patterns.md` pasa a ser un port destilado de `$SP/skills/test-driven-development/writing-good-tests.md`: el test nombra el fallo que caza; expectativas derivadas de forma independiente; sin change-detector ni string-presence; mutation check. Se conservan las reglas de mocks actuales. El nombre del fichero no cambia, porque otras skills lo referencian.
  - `SKILL.md`: la suite del proyecto define qué es "verde" (no el fichero del test), y se añade un puntero a qué no testear.
  - Se mantiene la atribución MIT existente, con la versión actualizada a 6.4.2.

## Task 5 — wiring: README, licencias, versión (ola 2)

- **Files:** Modify `plugins/exo/README.md`, `plugins/exo/.claude-plugin/plugin.json` y, si procede, `plugins/exo/LICENSES/`. Create `plugins/exo/LICENSES/ponytail.LICENSE`.
- **Interfaces:** Consumes los cambios de T1-T4 (solo para describirlos).
- **Contenido exigido:**
  - README: la base pasa a "superpowers 6.1.1 + portes de 6.2.0–6.4.2 (writing-plans, writing-good-tests, Review Focus, Rulings)", y se añade la atribución a ponytail.
  - `plugin.json`: bump minor a 1.4.0.
  - LICENSE de ponytail: MIT, © 2026 DietrichGebert, texto copiado de `$PT/LICENSE`.
- **Aceptación:** `bash scripts/test-exec-bit.sh` y `bash scripts/test-shellcheck.sh` siguen verdes, si aplican.
