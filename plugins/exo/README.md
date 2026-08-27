# plugin `exo`

Framework de trabajo agéntico con memoria persistente. Fusiona los dos
plugins previos (`process` y `reflex`) en uno solo: el proceso de trabajo
completo (diseño, plan, ejecución, TDD, debug, verificación, cierre de
sesión, consolidación) más la capa de reflejos que activa ese conocimiento
procedural **en el punto de acción**, no como prosa pasiva esperando recall.

Formato de skill: frontmatter de disparo (`name` + `description`) + body
router ~30-50 líneas + reference files empaquetados en el directorio de cada
skill (progressive disclosure — la carne va ahí, no en la KB). Overlay
personal con patrón único de degradación: probe del CLI → si no hay
engine/KB, defaults + aviso visible en el output final → nunca bloquear la
tarea.

Diseño: `docs/superpowers/specs/2026-07-17-prep-m3-process-skills-design.md`
y la nota *"Cerebro portable + capa de reflejos — design spec"* (proyecto
`kb-demo`, engine `exo`). Gold de paridad de movimientos:
`evals/prep-m3/gold/`.

## Los nueve skills

| Skill | Qué hace |
|---|---|
| brainstorm | Explora intención, requisitos y diseño en diálogo antes de implementar; termina en spec escrita y aprobada. |
| plan | De spec/requisitos a plan de tareas bite-sized con paths, código y comandos exactos para un ejecutor sin contexto. |
| orchestrate | Ejecuta planes multi-tarea: despacha un ejecutor fresco por tarea, review en dos etapas por tarea y review final whole-branch. |
| tdd | Test primero, verlo fallar por la razón esperada, código mínimo, verde, refactor. |
| debug | Dos puertas: bug/test que falla, o atasco (mismo error ≥3 veces, terreno desconocido). Root cause y recon antes de computar. |
| verify | Evidencia fresca del comando antes de cualquier claim de "completo" o "arreglado" — antes de commitear o aceptar el trabajo de un subagente. |
| document | Extrae decisiones, aprendizajes y patrones de la sesión y los guarda en la KB siguiendo el contrato de routing; commit scoped al cerrar. |
| distill | Consolidación offline de la KB (sleep-time compute manual): colapsa bitácoras en destilados, chequea presupuestos, promueve doctrina a core. |
| recon-first | Recupera y verifica asunciones antes de computar — la puerta "mira antes de saltar" cuando estás atascado o en terreno desconocido. |

## El agente `executor`

`agents/executor.md` — ejecutor de tareas de implementación acotadas bajo
doctrina de buena ingeniería. Despachado por `orchestrate` (subagent-driven
development): trae modelo y disciplina de serie, no hay que recordarle
verificar ni cómo commitear.

## Invariantes de todo reflejo

- **Never-block**: `exit 0` siempre; nunca `deny`. Dos mecanismos:
  `additionalContext` (warn = telemetría) y, desde v0.6.0, **rewrite
  silencioso** (`updatedInput`) SOLO sobre hechos parseados de alta
  confianza — nunca sobre juicio.
- **Abstención por defecto**: un falso positivo cuesta más que el silencio.
  Solo disparan con trigger de alta confianza (sentinel 1×/sesión, umbral, o
  match estricto).
- **El logging nunca rompe el warn-only**: best-effort
  (`>> log 2>/dev/null || true`).

## Hooks

Tabla exacta al cableado vivo de `hooks/hooks.json` (nueve comandos):

| Reflejo | Evento | Fichero | Qué hace | Abstención |
|---|---|---|---|---|
| clean-orchestrator | `PreToolUse:WebSearch\|WebFetch` | `scripts/clean-orchestrator-research.sh` | recuerda delegar research a subagentes | parent-only + 1×/sesión |
| git-c | `PreToolUse:Bash` | `scripts/git-c-bash.sh` | reescribe `cd <path> && git <read-only>` → `git -C <path> …` | rewrite solo si patrón estricto (ver comentarios del script) |
| zero-residuo | `PreToolUse:Bash` | `scripts/git-add-all-guard.sh` | avisa ante `git add -A`/`--all`/`.` | calla en `git add <ficheros>` explícito |
| verify-before-done | `PreToolUse:Bash` | `scripts/verify-before-commit.sh` | avisa antes de `git commit` si no hay test verde reciente | escape hatch `--no-verify`; calla en commits solo-docs |
| exo-recall | `SessionStart` | `scripts/exo-recall.sh` | inyecta instrucción de memoria + digest 7d, servido por el engine `exo` (SQLite) | — (PUSH) |
| document-remind | `Stop` | `scripts/document-remind.sh` | recuerda `/document` al cerrar | 1×/sesión + umbral de transcript |
| exo-index | `Stop` | `scripts/exo-index.sh` | reindexa la KB al cierre de sesión | best-effort, fallback logueado |
| subagent-inject | `SubagentStart` | `scripts/subagent-inject.sh` | inyecta doctrina/contexto al arrancar un subagente | — (PUSH) |
| recall-inject | `UserPromptSubmit` | `scripts/recall-inject.sh` | recall dirigido por el prompt del usuario | fallback logueado (`recall-fallback`) |

Nota: el README anterior de `reflex` documentaba también `cost-pyramid` y
`stuck-loop` como reflejos independientes. Ninguno de los dos está cableado
hoy en `hooks.json` como hook propio: `cost-pyramid` no tiene script en
`scripts/`, y la técnica de `stuck-loop` vive absorbida como comentario/lógica
dentro de `verify-before-commit.sh`. Se documenta la tabla real, no la
aspiracional.

## Medición de falsos positivos

Cada **disparo** se loguea (best-effort) a `~/.claude/reflex-log.jsonl` vía
`scripts/_reflex-log.sh`. Para revisar:

```bash
plugins/exo/scripts/reflex-fp-review.sh   # digest por reflejo
```

y se pasa el volcado + `scripts/reflex-fp-adjudicate.prompt.md` a un agente
que clasifica TP/FP → **FP-rate por reflejo**. Gate de escalado: review a
≥10 disparos o ~2 semanas; FP <20% sano · 20–50% afinar · >50% retirar.

## Primitivo útil

El input de un hook `PreToolUse` trae **`agent_id`/`agent_type` no vacíos
sii corre dentro de un subagente** (`session_id`/`transcript_path` son
compartidos con el padre y no discriminan). Un reflejo puede así declararse
parent-only, subagent-aware o indiferente.

## Atribución

`exo` absorbe por destilación selectiva el catálogo de
[`obra/superpowers`](https://github.com/obra/superpowers) (MIT, © 2025 Jesse
Vincent — copia literal del LICENSE en `LICENSES/superpowers.LICENSE`) más
doctrina propia. brainstorm, plan, orchestrate, tdd, debug y verify son obras
derivadas por destilación de sus fuentes superpowers 6.1.1; document,
distill, recon-first y la capa de reflejos son fuente propia.

| Skill | Absorbe de superpowers 6.1.1 (MIT) | Absorbe propio |
|---|---|---|
| brainstorm | brainstorming | — |
| plan | writing-plans | — |
| orchestrate | subagent-driven-development, executing-plans, dispatching-parallel-agents | orchestrate-personal (cost pyramid, memory packet, blindspot pass) + reviewer-dispatch escalado al riesgo del diff |
| tdd | test-driven-development | — |
| debug | systematic-debugging | recon-first (dos puertas: bug + stuck/pre-grind) |
| verify | verification-before-completion | gate de validación del padre de orchestrate-personal |
| document | — | `~/.claude/commands/documenta.md` |
| distill | — | sleep-time compute manual, propio |
| recon-first | — | doctrina propia (mitad TRIGGER de `paul-profile/orchestrate-personal`) |
