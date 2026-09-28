# Campaña K — Task 0: recon del harness (2026-09-28, parcial)

Claude Code `2.1.284`. Todo lo de abajo se comprobó con corridas reales en
scratchpad. No se corrió ninguna tarea del pool.

## Comprobado

- **Modelo de los brazos:** `claude-sonnet-5-5` existe y responde. Rellena el
  hueco de D1 en el paso 2 de §12.
- **`--bare` descartado.** Solo autentica con `ANTHROPIC_API_KEY`, y en esta
  máquina no está definida (se usa OAuth).
- **Aislamiento base que funciona:**

  ```
  claude -p --model claude-sonnet-5-5 --setting-sources "" --strict-mcp-config \
    --settings <brazo>.json --append-system-prompt-file ~/.claude/CLAUDE.md \
    --no-session-persistence --output-format stream-json --verbose "<prompt>" < /dev/null
  ```

  - El `init` resultante solo lista los plugins builtin (`agents-md`,
    `telemetry`): no carga plugins del usuario, ni MCP, ni hooks.
  - `{"autoMemoryEnabled": false}` en `--settings` deja
    `memory_paths: null`. **Sin eso, la auto-memory de Claude Code queda
    activa en todos los brazos**, lo que sería un confusor.
  - **Con `--setting-sources ""` el CLAUDE.md global no se carga.** D3 lo
    exige en todos los brazos, así que se inyecta con
    `--append-system-prompt-file`. Canario: SÍ con el flag, NO sin él.
    Diferencia con producción, constante en los brazos: va como system
    prompt, no como contexto de usuario.
  - Los hooks de memoria se cablean en `--settings` con rutas absolutas
    (`exo-recall.sh` en SessionStart, `recall-inject.sh` en
    UserPromptSubmit). Canario A3: ve los dos bloques `=== Recall exo`.
    Canario A0: no ve ninguno.
  - Con `REFLEX_LOG_FILE` apuntando al scratchpad, el
    `~/.claude/reflex-log.jsonl` real queda intacto (0 líneas con el
    `session_id` del canario). Esto protege los datos de la fase 0.
- **Coste fijo por corrida** (prompt trivial, list price): $0,018–0,030 con
  el aislamiento, frente a $0,069 con la config del usuario.

## Pendiente

- **Coste de tareas reales** (3 sondas fuera del pool, A0 y A3). Bloqueado:
  lanzar agentes headless con `--permission-mode bypassPermissions` requiere
  permiso explícito de Paul.
- **A1 sin `exo`:** `exo` y `claude` comparten `~/.local/bin`, así que no
  basta con quitar el directorio del PATH. Propuesta: un stub `exo` que sale
  con 127, antepuesto en el PATH del agente; `EXO_BIN` absoluto solo en el
  hook; `--disallowedTools` sobre la ruta absoluta; y detección de fuga en el
  transcript (§9).
- **Snapshot de KB por tarea:** `EXO_INDEX` aislado + checkout de
  `wisdom-paul` en T−. Sin probar.
