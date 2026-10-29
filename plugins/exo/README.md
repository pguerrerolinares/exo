# plugin `exo`

Framework de trabajo agéntico con memoria persistente: el proceso de trabajo
completo (diseño, plan, ejecución, TDD, debug, verificación, cierre de sesión,
consolidación) más una capa de reflejos que activa ese conocimiento
procedural **en el punto de acción**, no como prosa pasiva esperando recall.

Cada skill es un `SKILL.md` de disparo (`name` + `description`) con reference
files en su directorio (progressive disclosure). Patrón único de degradación:
si no hay engine o KB, defaults y un aviso visible; nunca se bloquea la tarea.

Diseño: `docs/superpowers/specs/2026-07-17-prep-m3-process-skills-design.md`.
Gold de paridad de movimientos: `evals/prep-m3/gold/`. El detalle de cada
script vive en su cabecera, que es lo que se mantiene al día; este README
solo da el mapa.

## Skills (8)

| Skill | Qué hace |
|---|---|
| brainstorm | Explora intención, requisitos y diseño en diálogo antes de implementar; termina en spec escrita y aprobada. Trae la plantilla del consultor de campo (`consultor-brief.md`), que usan también plan y la review adversarial de spec. |
| plan | De spec a plan-contrato: tareas con ficheros, interfaces y tests con aserciones, **sin cuerpos de código**, para un ingeniero capaz que conoce la interfaz y el test. |
| orchestrate | Ejecuta planes multi-tarea: un ejecutor fresco por tarea, review en dos etapas por tarea y review final whole-branch, con ledger durable. Scripts en `skills/orchestrate/scripts/`: `task-dag`, `task-brief`, `sdd-workspace`, `red-witness`, `review-package`, `rulings-collect`. Referencias: `olas.md`, `despertar.md`. |
| tdd | Test primero, verlo fallar por la razón esperada, código mínimo, verde, refactor. |
| debug | Dos puertas: bug o test que falla, y atasco o pre-grind (mismo error ≥3 veces, terreno desconocido). Root cause y recon antes de computar. |
| verify | Evidencia fresca del comando antes de cualquier claim de «completo» o «arreglado», antes de commitear o aceptar el trabajo de un subagente. |
| document | Extrae decisiones, aprendizajes y patrones de la sesión y los guarda en la KB según el contrato de routing; commit scoped al cerrar. Si el gate de presupuesto muerde, lanza el subagente de `auto-distill.md`. |
| distill | Consolidación offline de la KB (sleep-time compute manual): colapsa bitácoras en destilados, chequea presupuestos, promueve doctrina a core. |

Solo `document` y `distill` escriben en la KB; el resto no la tocan.

## Agentes

- **`exo:executor`** (`agents/executor.md`): ejecutor de tareas de
  implementación acotadas, con la doctrina de ingeniería en su system prompt
  (verificar antes de declarar hecho, commits limpios, escalera
  anti over-engineering). Lo despacha `orchestrate`.
- **`exo:consultor`** (`agents/consultor.md`): consultor de **solo lectura**
  (`disallowedTools: Edit, Write, NotebookEdit, Agent`) para decisiones de
  diseño. Su brief sale de `skills/brainstorm/consultor-brief.md`; lo usan
  brainstorm, plan y el reflejo `delegar-decision`.

## Capa de reflejos

Doce comandos cableados en `hooks/hooks.json`, más el mod `hooks/register.ts`
(ver «Canal de reglas duras»). Agrupados por efecto:

- **Solo log**: miden, no tocan al agente.
- **Avisan**: inyectan `additionalContext` (recordatorio o contexto).
- **Reescriben**: `updatedInput` silencioso, solo sobre hechos parseados de
  alta confianza, nunca sobre juicio.
- **Deniegan o bloquean**: `permissionDecision: deny` o `decision: block`.

Invariantes:

- **Abstención por defecto**: un falso positivo cuesta más que el silencio.
  Un reflejo dispara solo con un trigger de alta confianza (sentinel
  1×/sesión, umbral o match estricto) y sale con `exit 0` y sin salida ante
  cualquier duda (sin `jq`, transcript ilegible, subagente…).
- **Bloquear solo en los hooks nombrados** (`bash-guards`, `sdd-wake-guard`,
  `delegar-decision`, `subagent-bg-guard`), y siempre con tope y escape: un
  bloqueo por mensaje humano o por parada, o backoff tras denies repetidos,
  más un escape declarado (`[reserva: …]`, `--no-verify`, `/tmp`,
  `stop_hook_active`).
- **El logging nunca rompe**: best-effort (`>> log 2>/dev/null || true`).

El input de un hook `PreToolUse` trae `agent_id`/`agent_type` no vacíos solo
dentro de un subagente (`session_id`/`transcript_path` se comparten con el
padre), y es lo que permite declarar un reflejo parent-only o
subagent-aware.

Tabla exacta al cableado vivo (una fila por comando de `hooks.json`):

| Reflejo | Evento | Fichero | Efecto | Qué hace |
|---|---|---|---|---|
| red-gate | `PreToolUse:SubagentHandback` | `scripts/red-gate.sh` | solo log | Registra si un `exo:executor` entrega sin línea `exit=[1-9]` en `red.log`. Promoción a deny pendiente de medir. |
| clean-orchestrator | `PreToolUse:WebSearch\|WebFetch\|navegación MCP` | `scripts/clean-orchestrator-research.sh` | avisa | Recuerda delegar el research a subagentes (parent-only, 1×/sesión, no en apps locales). |
| search-first | `PreToolUse:Agent\|Task\|Edit\|Write\|NotebookEdit` | `scripts/search-first.sh` | avisa | Avisa si el primer `Agent`/`Task`/`Edit`/`Write` de la sesión no fue precedido de `exo search`/`exo targets` (1×/sesión, no en subagentes). |
| exo-recall | `SessionStart` | `scripts/exo-recall.sh` | avisa (inyecta) | Inyecta el `core-index` y las notas recientes servidos por el engine; reafirma reflejos tras `compact`; avisa de un gate de `/document` pendiente y de superpowers habilitado; escribe el testigo `ss-<sid>`. Cae a un fallback embebido si el engine es < `ENGINE_MIN`. |
| estilo-directo | `SessionStart` | `scripts/estilo-directo.sh` | avisa (inyecta) | Inyecta la directiva estática de estilo de `estilo-directo.md`. |
| document-remind | `Stop` | `scripts/document-remind.sh` | avisa | Recuerda `/document` al cerrar (1×/sesión, con umbral de transcript) y hace de testigo de entrega de reglas (`ss-` vs `hb-`). |
| subagent-inject | `SubagentStart` | `scripts/subagent-inject.sh` | avisa (inyecta) | Inyecta doctrina por perfil y, según el perfil, las reglas duras (ver abajo). |
| exo-index | `Stop` | `scripts/exo-index.sh` | mantenimiento | Reindexa la KB al cerrar la sesión; best-effort. |
| bash-guards | `PreToolUse:Bash` | `scripts/bash-guards.sh` | reescribe / deniega | Reescribe `cd <path> && git <lectura>` a `git -C`; **deniega** `git add -A`/`--all`/`.` sin pathspec; registra `git commit` sin test verde reciente (sin avisar). |
| sdd-wake-guard | `Stop` | `scripts/sdd-wake-guard.sh` | bloquea | Con un ledger SDD activo y tareas de fondo vivas sin cron, pide un `CronCreate` que siga `skills/orchestrate/despertar.md`. |
| delegar-decision | `Stop` | `scripts/delegar-decision.sh` | bloquea | Bloquea el cierre del padre si cierra con propuesta + petición de visto bueno sin lanzar `Agent`/`Task`. Escape `[reserva: externo\|irreversible\|re-scope\|hecho-de-Paul\|pedido]`; tope de un bloqueo por mensaje humano; requiere `python3`. |
| subagent-bg-guard | `SubagentStop` | `scripts/subagent-bg-guard.sh` | bloquea | Bloquea una vez el cierre de un subagente con tareas de fondo propias vivas y pide `TaskStop` o esperar sobre el log. |

Hay además scripts huérfanos (`git-c-bash.sh`, `git-add-all-guard.sh`,
`verify-before-commit.sh`) que ya no están cableados: sobreviven como red de
regresión de `bash-guards.sh`.

## Canal de reglas duras

Las reglas duras son las del dueño (globales, en `core/reglas-duras.md` de la
KB) y las del repo (sección `## Reglas duras` de su nota de proyecto), con un
máximo de 10 entre ambas listas. Llegan al agente por tres vías:

- **El mod `hooks/register.ts`** (`"modules"` de `hooks.json`): registra
  `session.start`, `prompt.submit`, `turn.complete` y `prompt.compose`, y en
  cada prompt inyecta «Reglas duras del dueño» y «Reglas duras del proyecto»
  (`exo rules --cwd`). Deja un latido `hb-<sid>` en `~/.claude/exo-rules/`.
- **`subagent-inject.sh` + `compose-inject.sh`**: componen el bloque por
  `agent_type` según `scripts/inject-profiles.json`. Las reglas duras entran
  en los perfiles `reducido` (`exo:executor`, `exo:consultor`) y `ejecucion`
  (`general-purpose`, `claude`); `Explore` y `Plan` reciben `divergente`, y el
  resto `doctrina`.
- **Testigo ss/hb**: `exo-recall` escribe `ss-<sid>` (lo que SessionStart
  decidió entregar) y `document-remind` lo compara con `hb-<sid>` (lo que el
  mod entregó). Si no coinciden, avisa de la degradación.

`exo rules` es el comando del engine: sin flags resuelve las reglas del
cwd; `--check` es el gate del máximo (exit 3 si una nota lo supera) y lo corre
el pre-commit de la KB; `--coverage` informa de cuántas reglas declara cada
nota. `scripts/cobertura-reglas.sh` suma a ese informe los repos con sesiones
donde `exo rules` no entregó nada.

**Cuentas Team/Enterprise**: el guard integrado `cc-plugin-sec-default`
intercepta `prompt.compose` antes que el mod y Claude Code avisa en cada turno
(`its prompt.compose hook was skipped`). No se pierde función: las reglas
llegan por `prompt.submit`. Para quitar el aviso, pon la opción `composeHook`
del plugin a `false` (fila «Registrar el hook prompt.compose» de `/config`,
que guarda `pluginConfigs["exo@exo"].options.composeHook` en
`~/.claude/settings.json`). La org no se puede autodetectar en el registro
(`register()` no tiene `$`).

## Pre-commit de la KB y auto-distill

`scripts/kb-precommit.sh` es el pre-commit de la KB (se instala con
`ln -sf <script> <kb>/.git/hooks/pre-commit`). Juzga el index, no el working
tree, y corre `exo budget`, `exo ratchet --staged`, `exo rules --check` y las
comprobaciones de `tier`, `tags` (obligatorio en notas nuevas, aviso en las
modificadas) y wikilinks. Se salta con `git commit --no-verify`.

Cuando `/document` choca con el gate de presupuesto de una nota,
`skills/document/auto-distill.md` es el brief del subagente que la baja al
≤60 % del techo moviendo bloques verbatim en un worktree aparte y commitea
como `docs(kb): auto-distill`.

## Sustituir superpowers

exo absorbe los skills de superpowers (ver Atribución); con los dos
habilitados el agente recibe skills y hooks duplicados. Antes de usar exo en
una máquina que tenía superpowers, desactívalo (desactivar, no desinstalar:
se puede revertir):

```bash
claude plugin disable superpowers
```

`install.ps1` lo ejecuta por ti (`claude plugin disable <clave> --scope user`)
si superpowers está habilitado en el scope de usuario; opt-out con
`-KeepSuperpowers` o `EXO_DISABLE_SUPERPOWERS=0`. Si se te olvida,
`exo doctor` marca `superpowers_disabled` como `warn` (scope de usuario) y
SessionStart añade una línea de aviso (scopes usuario, proyecto y local; el
scope managed no se mira).

## Medición de falsos positivos

Cada disparo se loguea (best-effort) a `~/.claude/reflex-log.jsonl` vía
`scripts/_reflex-log.sh`. Para revisar:

```bash
plugins/exo/scripts/reflex-fp-review.sh   # digest por reflejo
```

Se pasa el volcado con `scripts/reflex-fp-adjudicate.prompt.md` a un agente
que clasifica TP/FP y da el **FP-rate por reflejo**. Gate de escalado: review
a ≥10 disparos o ~2 semanas; FP <20 % sano · 20–50 % afinar · >50 % retirar.

La latencia de `SessionStart` se midió el 2026-09-24 (p95 de `hook_ms`: 1671 ms
en W11, 1035-1173 ms en Linux) y no se ha vuelto a medir desde que `exo-recall`
resuelve reglas y avisos.

## Atribución

`exo` absorbe por destilación selectiva el catálogo de
[`obra/superpowers`](https://github.com/obra/superpowers) (MIT, © 2025 Jesse
Vincent — copia literal del LICENSE en `LICENSES/superpowers.LICENSE`) más
doctrina propia. brainstorm, plan, orchestrate, tdd, debug y verify son obras
derivadas por destilación de sus fuentes superpowers 6.1.1 + portes de
6.2.0–6.4.2 (writing-plans, writing-good-tests, Review Focus, Rulings);
document, distill y la capa de reflejos son fuente propia.

La escalera anti over-engineering del executor y el pase de over-engineering
del review (tags `delete|stdlib|native|yagni|shrink`) derivan de
[`ponytail`](https://github.com/DietrichGebert/ponytail) (MIT, © 2026
DietrichGebert — copia literal en `LICENSES/ponytail.LICENSE`).

| Skill | Absorbe de superpowers (MIT) | Absorbe propio |
|---|---|---|
| brainstorm | brainstorming | — |
| plan | writing-plans | — |
| orchestrate | subagent-driven-development, executing-plans, dispatching-parallel-agents | blindspot pass (de `orchestrate-personal`), reviewer-dispatch escalado al riesgo del diff, ponytail (escalera anti over-engineering, pase de review) |
| tdd | test-driven-development + writing-good-tests | — |
| debug | systematic-debugging | dos puertas: bug y stuck/pre-grind |
| verify | verification-before-completion | gate de validación del padre de `orchestrate-personal` |
| document | — | propio |
| distill | — | sleep-time compute manual, propio |
