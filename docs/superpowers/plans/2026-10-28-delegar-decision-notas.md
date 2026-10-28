# delegar-decision — notas de la prueba manual del doble bloqueo (Task 5)

Estado: **NO concluyente**. El doble bloqueo no se pudo reproducir en headless porque
`delegar-decision` no dispara en `claude -p`. Falta una sesión interactiva (ver "Qué falta").

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
- **delegar-decision no dispara en -p**. Causa, verificada ejecutando `delegar-decision.py` sobre el transcript:
  el prompt del usuario queda con `promptSource: "sdk"`, y `es_mensaje_humano` solo acepta `typed`/`queued`.
  Sin mensaje humano no hay segmento: `{"uuid": null, "dispara": false, "kind": null}`. Es la R19 de la spec,
  pero con consecuencia más fuerte: en headless/SDK el gate es inerte, no "un bloqueo por segmento".
- Qué reason llega con doble bloqueo (uno / ambos concatenados / solo el primero): **no observado**.

## Qué falta

Sesión interactiva (promptSource `typed`). Se intentó vía tmux, pero el diálogo de confianza de carpeta
escribiría en `~/.claude.json` (config de usuario, fuera de los límites de la tarea), así que se detuvo. Para
cerrarlo: en una carpeta ya confiada, con el plugin cargado, ledger SDD fresco en `.superpowers/sdd/*/progress.md`,
lanzar `sleep 240` en background y pedir el cierre «Mi recomendación es X. ¿Te parece?», y leer en
`--debug-file` los `Hook Stop ... success` de ambos y el reason que recibe el agente.

## Riesgo para el kill (condicional)

Si en interactivo llegara un solo reason, `sdd-wake-guard` (primero en el orden de hooks.json) taparía a
`delegar-decision` en ese segmento; el sentinel de `delegar-decision` ya estaría consumido y el bloqueo se
perdería. Hay que medirlo antes de declarar el kill. Adicional ya confirmado: en `claude -p`/SDK el gate nunca
actúa, así que la medición de bloqueos en esos modos saldrá a cero.
