# plugin `process`

Skills del framework agéntico de Paul: el proceso de trabajo completo
(diseño, plan, ejecución, TDD, debug, verificación, cierre de sesión)
destilado de `superpowers` (MIT, Jesse Vincent) más doctrina propia
(paul-profile, reflex, `/documenta`).

Formato de skill: frontmatter de disparo (`name` + `description`) + body
router ~30-50 líneas + reference files empaquetados en el directorio de cada
skill (progressive disclosure — la carne va ahí, no en la KB). Overlay
personal con patrón único de degradación: probe del CLI → si no hay
engine/KB, defaults + aviso visible en el output final → nunca bloquear la
tarea.

Diseño: `docs/superpowers/specs/2026-07-17-prep-m3-process-skills-design.md`.
Gold de paridad de movimientos: `evals/prep-m3/gold/`.

## Atribución

`process` absorbe por destilación selectiva el catálogo de
[`obra/superpowers`](https://github.com/obra/superpowers) (MIT, © 2025 Jesse
Vincent — copia literal del LICENSE en `LICENSES/superpowers.LICENSE`) más
doctrina propia. brainstorm, plan, orchestrate, tdd, debug y verify son obras
derivadas por destilación de sus fuentes superpowers 6.1.1; documenta es
fuente propia.

| Skill | Absorbe de superpowers 6.1.1 (MIT) | Absorbe propio |
|---|---|---|
| brainstorm | brainstorming | — |
| plan | writing-plans | — |
| orchestrate | subagent-driven-development, executing-plans, dispatching-parallel-agents | orchestrate-personal (cost pyramid, memory packet, blindspot pass) + reviewer-dispatch escalado al riesgo del diff |
| tdd | test-driven-development | — |
| debug | systematic-debugging | recon-first (dos puertas: bug + stuck/pre-grind) |
| verify | verification-before-completion | gate de validación del padre de orchestrate-personal |
| documenta | — | `~/.claude/commands/documenta.md` |
