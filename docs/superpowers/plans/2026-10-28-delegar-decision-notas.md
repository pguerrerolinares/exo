# delegar-decision — notas de la prueba manual del doble bloqueo (Task 5)

Estado: **concluyente (2026-10-28, sesión interactiva): no se tapan.** Ver «Resultado interactivo» al final.
La prueba headless de abajo no lo era; se conserva como histórico.

Versión probada: Claude Code 2.1.312, plugin exo 1.26.0 cargado con `--plugin-dir` (inline).
Nota: el exo 1.25.0 instalado en global también se cargó en paralelo; sus Stop hooks no incluyen
`delegar-decision`, así que no afecta a lo observado.

## Comando

```
cd <scratch>/w   # git init + .superpowers/sdd/x/progress.md (ledger fresco, para sdd-wake-guard)
echo "<prompt>" | REFLEX_LOG_FILE=<scratch>/reflex.jsonl claude -p \
  --plugin-dir <worktree>/plugins/exo --debug-file <scratch>/debug1.log \
  --output-format stream-json --verbose --allowedTools="Bash(sleep 240)"
```

(El prompt va por stdin: `--allowedTools` es variádico y se traga el argumento posicional.
`--permission-mode bypassPermissions` lo denegó el clasificador de auto mode y no se usó.)

## Variantes

1. Prompt: «Ejecuta con Bash, run_in_background=true, `sleep 240`. No lo esperes. Responde únicamente:
   Mi recomendación es usar la opción A para el cache. ¿Te parece?»
   - Dispara `sdd-wake-guard` (tarea de fondo viva + ledger fresco + sin crons).
   - `delegar-decision.sh` terminó con status 0 y **sin salida**; sin línea en el reflex-log.
2. Igual, por `--input-format stream-json` sin tarea de fondo: mismo resultado (`delegar-decision` no dispara).

## Observado

- **sdd-wake-guard bloquea solo, una vez**. Debug log:
  `Hook Stop (Stop) success: {"decision":"block","reason":"Hay tareas de fondo vivas (bs7lvhb1d) y ningún cron armado..."}`
  El agente recibió ese único reason, armó el `CronCreate` one-shot, lo canceló después y cerró. El segundo Stop
  (`stop_hook_active`) no bloqueó. Una petición, satisfecha en la continuación.
- **document-remind**: no bloquea nunca; emite `systemMessage` («💾 ¿Cierras sesión?...»). No sirve para el doble bloqueo.
  El único par bloqueante real es `sdd-wake-guard` + `delegar-decision`.
- **delegar-decision no disparó en -p (corregido en la ronda final, R23/R25)**. La conclusión original («inerte en
  -p por `promptSource: "sdk"`») era un artefacto: se diagnosticó sobre un transcript sin la línea assistant final.
  Experimento stop-race (volcado del input y del transcript en el Stop): durante el Stop el transcript todavía no
  tiene la línea assistant final, mientras que `last_assistant_message` del input sí la trae (`"MARCADOR-XYZ-42"`
  en las dos corridas; evidencia en el scratchpad de la sesión, `stop-race/dump.txt`). `promptSource: "sdk"` solo
  quita el uuid (no hay tope); el segmento abierto se evalúa igual. Fix: el hook evalúa `last_assistant_message`.
  Sin mensaje humano el gate bloquea en cada Stop no activo con P2/P3.
- Qué reason llega con doble bloqueo (uno / ambos concatenados / solo el primero): **no observado**.

## Qué falta

Sesión interactiva (promptSource `typed`). Se intentó vía tmux, pero el diálogo de confianza de carpeta
escribiría en `~/.claude.json` (config de usuario, fuera de los límites de la tarea), así que se detuvo. Para
cerrarlo: en una carpeta ya confiada, con el plugin cargado, ledger SDD fresco en `.superpowers/sdd/*/progress.md`,
lanzar `sleep 240` en background y pedir el cierre «Mi recomendación es X. ¿Te parece?», y leer en
`--debug-file` los `Hook Stop ... success` de ambos y el reason que recibe el agente.

## Riesgo para el kill (condicional)

Si en interactivo llegara un solo reason, `sdd-wake-guard` (los Stop corren en paralelo, sin orden) podría tapar a
`delegar-decision` en ese segmento; el sentinel de `delegar-decision` ya estaría consumido y el bloqueo se
perdería. Hay que medirlo antes de declarar el kill. Con R26 el tope suprimido queda en el log como `kind=tope`.

## Resultado interactivo (2026-10-28)

Sesión `typed`, plugin exo 1.26.0 instalado (no inline), carpeta con ledger fresco en
`.superpowers/sdd/prueba/progress.md`, `sleep 240` en background y el cierre «Mi recomendación es
usar la opción A para el cache. ¿Te parece?». Repetido en dos sesiones, con el mismo resultado.

- `--debug-file`: los dos Stop bloquean en el mismo ciclo (`sdd-wake-guard` 21:57:27.888 y
  `delegar-decision` 21:57:27.922, ambos `permissionDecision: deny`). `document-remind` solo emite
  `systemMessage`.
- El agente recibe **los dos reasons, por separado** («Stop hook error» ×2 en la UI) y cumple ambos
  en una sola continuación: `CronCreate` one-shot y reescritura con `[reserva: pedido]`.
- El Stop siguiente (21:57:44) no bloquea (`stop_hook_active`). Reflex-log: una línea
  `sdd-wake-guard` y una `delegar-decision kind=disparo patron=P2` por sesión.
- Consecuencia para el kill: el riesgo condicional de la sección anterior no se da; no se pierden bloqueos.
  Lo que sí se vio es el escape por `[reserva]` (aquí justificado: el prompt pedía responder
  «únicamente» esa frase); lo mide el kill 1.
- Falso positivo observado en la sesión que guiaba la prueba: un cierre con instrucciones que
  citaba el prompt de prueba dentro de un bloque ``` disparó P2. El detector no excluye bloques de
  código; candidato a fix si se repite en el re-etiquetado del kill 2.
