# Campaña K — Task 0: recon del harness (2026-09-28)

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

## Coste medido (3 sondas fuera del pool, 2026-09-28)

Tareas de juguete sobre un clon de exo (`evals/ablacion-k/harness/tareas-sonda.tsv`):
una pregunta de código, un test Rust con commit y una edición de README con
commit. Una corrida por brazo. Precio de lista (`costBasis: list`).

| sonda | A0 USD | A3 USD | A0 turnos | A3 turnos | A0 tokens in | A3 tokens in |
|---|---|---|---|---|---|---|
| p1 | 0,057 | 0,067 | 4 | 6 | 75.156 | 123.865 |
| p2 | 0,077 | 0,106 | 8 | 8 | 147.271 | 184.041 |
| p3 | 0,040 | 0,063 | 4 | 5 | 68.110 | 102.120 |

- Las 6 corridas terminaron `completed`, con rc 0 y stderr vacío. Wall-clock
  entre 8 y 54 s.
- A3 cuesta entre +17 % y +58 % más que A0 en estas sondas: es el contexto
  inyectado. Con n = 3 es descriptivo, no una estimación.
- **Son tareas pequeñas.** Las del pool (§5) serán más largas. Estimación
  para fijar el tope, **no medida**: 3–10× por corrida, es decir
  $0,2–1 por corrida.
- **Aviso sobre la unidad de D2:** la autenticación es OAuth (suscripción),
  así que el USD reportado es precio de lista y no facturación. El límite
  real es la cuota de uso de la suscripción.

## Errata operativa propuesta para §9 (fuga de brazo)

El canario literal de §9 («el transcript de A0/A1 contiene
`=== Recall exo`») da **falsos positivos** cuando la tarea toca el repo de
exo. En `p1-a0` el agente hizo `grep` en el clon y encontró la cadena en
`plugins/exo/scripts/testdata/golden-recall-inject/*.txt`, dentro de un
`tool_result`. No hubo hook: el stream no tiene eventos `hook_*` en A0.

Propuesta: la fuga se detecta sobre **el canal de inyección**, no sobre el
texto. Cuenta como fuga cualquiera de estos casos en un brazo que no debería
tenerlos:

- eventos `system` con subtype `hook_*`, o `additionalContext` en mensajes
  que no son `tool_result`;
- un `tool_use` de `Bash` que invoca `exo` (A0/A1);
- `recall-inject-emitted` en `reflex-*.jsonl` (A0/A1/A2).

Las apariciones de la cadena dentro de un `tool_result` no cuentan. Se
declara como errata en el verdict, según el paso 1 de §12.

## Pendiente

- **A1 sin `exo`:** `exo` y `claude` comparten `~/.local/bin`, así que no
  basta con quitar el directorio del PATH. Propuesta: un stub `exo` que sale
  con 127, antepuesto en el PATH del agente; `EXO_BIN` absoluto solo en el
  hook; `--disallowedTools` sobre la ruta absoluta; y detección de fuga como
  arriba.
- **Snapshot de KB por tarea:** `EXO_INDEX` aislado + checkout de
  `wisdom-paul` en T−. Sin probar.
- **Harness:** `evals/ablacion-k/harness/` (`run.sh`, `sondas.sh`,
  `a0.json`, `a3.json`). Las sondas de esta Task 0 corrieron con una copia
  idéntica en scratchpad; la única diferencia es el directorio de salida.
