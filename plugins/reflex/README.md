# reflex

Capa de **reflejos** de Paul: guardrails deterministas que activan conocimiento
procedural **en el punto de acción**, no como prosa pasiva esperando recall. Es la
mitad **TRIGGER/enforcement** de `paul-profile/orchestrate-personal` (que es la mitad
PUSH/prosa). Diseño completo en la KB: nota *"Cerebro portable + capa de reflejos — design spec"* (proyecto `kb-demo`, engine `exo`).

## Invariantes de todo reflejo

- **Never-block**: `exit 0` siempre; nunca `deny`. Dos mecanismos: `additionalContext`
  (warn = telemetría) y, desde v0.6.0, **rewrite silencioso** (`updatedInput`) SOLO
  sobre hechos parseados de alta confianza — nunca sobre juicio (investigación
  2026-07-05: el warn llega post-ejecución y en tool-tier; no cambia comportamiento).
- **Abstención por defecto**: un falso positivo cuesta más que el silencio. Solo
  disparan con trigger de alta confianza (sentinel 1×/sesión, umbral, o match estricto).
- **El logging nunca rompe el warn-only**: best-effort (`>> log 2>/dev/null || true`).

## Hooks

| Reflejo | Evento | Qué hace | Abstención |
|---|---|---|---|
| cost-pyramid (#1) | `PreToolUse:Workflow` | avisa si un workflow lanza fan-out ≥2 sin `model:` por fase | por-ocurrencia; calla si hay `model:` o fan-out <2 |
| clean-orchestrator (#6) | `PreToolUse:WebSearch\|WebFetch` | recuerda delegar research a subagentes | **parent-only** (guarda `agent_id`) + 1×/sesión |
| git-c (#2) | `PreToolUse:Bash` | **reescribe** `cd <path> && git <read-only>` → `git -C <path> …` (`updatedInput`+allow, log `git-c-rewrite`); el resto de matches conserva el warn | rewrite solo si: comando entero = `cd PATH && git REST`, PATH literal sin quotes/vars/`..` (cd lógico vs chdir físico divergen con symlinks) ni relativo bajo `CDPATH`, REST sin metacaracteres ni globs, subcomando en allowlist read-only (status/log/diff/show/…) |
| stuck-loop (#7) | `PreToolUse:Bash` | escanea el transcript JSONL de la sesión buscando el mismo comando con `is_error:true`; si ya falló ≥2× nudgea ANTES de malgastar otro intento — más robusto que PostToolUse, que no dispara ante tool-errors | `transcript_path` presente + ≥2 priors fallidos del mismo comando normalizado, 1× por comando/sesión (sentinel) |
| test-run-tracker | `PostToolUse:Bash` | tracker silencioso: guarda `{last_test_exit, last_test_ts}` en `/tmp/claude-reflex-testrun-<sid>.json` cuando el comando es un test-runner | no dispara, solo actualiza estado; silencio total si no hay exit_code |
| verify-before-done | `PreToolUse:Bash` | avisa antes de `git commit` si no hay test verde reciente (< 30 min) en la sesión; filtra commits de solo docs | escape hatch `--no-verify`; calla en commits solo-docs; calla si test verde reciente |
| zero-residuo | `PreToolUse:Bash` | avisa ante `git add -A`/`--all`/`.` (arrastra residuo); sugiere añadir ficheros explícitamente | calla en `git add <ficheros>` explícito; por-ocurrencia |
| exo-recall | `SessionStart` | inyecta instrucción de memoria + digest 7d + modo orquestador limpio, servido por el engine `exo` (SQLite, ~10ms) | — (PUSH) |
| documenta-remind | `Stop` | recuerda `/documenta` al cerrar | 1×/sesión + umbral de transcript |

## Medición de falsos positivos

Cada **disparo** se loguea (best-effort) a `~/.claude/reflex-log.jsonl` vía
`scripts/_reflex-log.sh`. Para revisar:

```bash
~/.claude/plugins/.../reflex/scripts/reflex-fp-review.sh   # digest por reflejo
```

y se pasa el volcado + `scripts/reflex-fp-adjudicate.prompt.md` a un agente que
clasifica TP/FP → **FP-rate por reflejo**. Gate de escalado: review a ≥10 disparos
o ~2 semanas; FP <20% sano · 20–50% afinar · >50% retirar.

## Primitivo útil

El input de un hook `PreToolUse` trae **`agent_id`/`agent_type` no vacíos sii corre
dentro de un subagente** (`session_id`/`transcript_path` son compartidos con el padre
y no discriminan). Un reflejo puede así declararse parent-only, subagent-aware o
indiferente.

**Estado por sesión (stuck-loop v2 — PreToolUse+transcript-scan)**: el reflejo `stuck-loop`
corre ahora en **PreToolUse:Bash**, ANTES de ejecutar el comando. Lee `transcript_path`
del payload (JSONL de la sesión) y escanea la cola (`tail -n 400`) buscando pares
`tool_use(Bash, mismo command) → tool_result(is_error:true)`. Si hay ≥2 priors fallidos
del mismo comando normalizado → nudge ANTES de malgastar otro intento. Sentinel por
`/tmp/claude-reflex-stuckpre-<sid>-<fp>` para no repetir el aviso del mismo comando.

Por qué se abandonó PostToolUse: (1) el payload real nunca incluye `exit_code`; (2) más
grave, `PostToolUse:Bash` **no dispara** cuando el comando sale como tool-error (exit≠0) —
justo los fallos que #7 quería cazar nunca llegaban. Los tests viejos pasaban (15/15)
porque inyectaban `exit_code` en payloads crafteados que la realidad nunca genera.
