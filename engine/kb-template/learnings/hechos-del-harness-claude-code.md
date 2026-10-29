---
permalink: "{{KB_NAME}}/learnings/hechos-del-harness-claude-code"
title: "El harness tiene hechos que muerden en producción: verificarlos contra la versión viva"
tags: [harness, claude-code, hooks, windows, agentes]
tier: stable
semilla: true
---

# El harness tiene hechos que muerden en producción: verificarlos contra la versión viva

**Aviso: esta nota recoge observaciones de una versión del harness (Claude Code, git, PowerShell), no contratos.** Pueden haber cambiado. Antes de apoyarte en una, compruébala contra la versión que usas: un hook de prueba, un comando de dos líneas. Lo que sí es estable es el método: estos hechos se encontraron rompiéndose en producción, no leyendo documentación, y la única defensa fiable es medir el comportamiento real. Un reflejo o guardarraíl que dependa de un hecho no verificado puede estar muerto aunque sus tests pasen (ver [[El fallo más caro es el que no avisa, y cada forma tiene su remedio|learnings/fallo-silencioso]] y [[La lectura verifica coherencia; solo la ejecución verifica verdad|learnings/verificar-ejecutando-no-leyendo]]).

## Hooks

- **`PostToolUse` de Bash no trae `exit_code`.** Un hook que dependa de él está muerto en producción. El fallo de una herramienta tiene **su propio evento** (`PostToolUseFailure`, con el código como primera línea de `error`: `Exit code N`); no lo deduzcas del evento de éxito.
- **`PreToolUse` evalúa antes de ejecutar.** Un guard que mira el comando completo ve `rm FLAG && merge` entero y lo deniega aunque el `rm` fuera primero. Gate y ejecución van en **comandos separados**: baja el flag en una llamada propia y ejecuta en la siguiente.
- **No des por cargado un hook que acabas de cambiar.** Los cambios en settings suelen recogerse en caliente y los de un plugin con `/reload-plugins`, pero no siempre: si el hook no se dispara, reinicia la sesión. Comprueba el cambio provocando el evento, no leyendo el fichero.
- **Un guard que casa por texto del comando casa también menciones.** Un heredoc o un `echo` que contenga `git push` puede tumbar la llamada entera. Y resuelve el repo por el `cwd` del payload: `cd dir && git commit` puede denegarse donde `git -C dir commit` pasa.
- **Un "Edit operation failed" de un hook `PostToolUse` puede ser ruido.** Si el `tool_result` del propio edit dice "updated successfully", el aviso es falso: no reverifiques el fichero ni reintentes. Verifica al final del bloque de trabajo, no tras cada edit.
- **Hooks en background a mano:** con `cmd &`, un hijo que hereda stdout retiene el hook hasta que termina. Redirige los tres descriptores, o declara el hook con `"async": true` (ojo: a un hook asíncrono no se le aplica el `timeout`).
- **La telemetría no debe romper lo que instrumenta.** Si el log falla (disco lleno, ruta ausente), eso no debe tumbar el hook ni la herramienta: `>> log 2>/dev/null || true`.

## Concurrencia y estado compartido

- **`>>` concurrente a un JSONL entrelaza las líneas grandes** (por encima de ~4 KB, el tamaño de escritura atómica de una tubería). En una prueba con líneas de varios KB las filas se entrelazaron y `wc -l` seguía dando el total esperado. Usa lock (`flock`) y **valida parseando cada línea, nunca contando**.
- **`git add -A` bajo concurrencia stagea el árbol entero**, incluido lo ajeno a medias, no "lo que tocaste". Arreglo estructural: un worktree por agente y `git -C <ruta>`; add con rutas explícitas.
- **Los subagentes a veces editan por error el checkout principal.** Corre `git status` antes de cada merge: caza el fichero perdido.
- **Ningún agente muta ni revierte estado compartido para limpiar, comparar o falsar.** `git stash` alcanza al árbol de todos; `git checkout -- <fichero>` para quitar un bug inyectado se lleva también edits sin commitear; restaurar un log para borrar tus propias trazas destruye un dato (*un reflejo disparado por tu propia actividad es dato, no suciedad*). Alternativas: `git show <sha>:<ruta>` volcado a un temporal, parche inverso, copia desechable; y falsa solo sobre estado commiteado.
- **No inferir de un estado intermedio.** Leer el working tree de un agente vivo y deducir que abandonó lleva a escribir encima de él. Si dudas, pregúntale con `SendMessage` y espera ([[El padre coordina y valida; el ejecutor implementa|learnings/orquestador-limpio]]).
- **Artefactos temporales de un dispatch, fuera del worktree.** Un directorio temporal del job: así no ensucian el árbol ni el diff.

## Subagentes y procesos largos

- **El informe de un subagente viaja como mensaje de entrega, no como resultado de una herramienta.** Si tu versión ofrece una herramienta de entrega (aquí `SubagentHandback`; confirma que existe en la tuya), el texto aparece en `tool_input.message` de un `PreToolUse` cuyo matcher sea ese nombre, y `SubagentStop` solo ve la frase de despedida en `last_assistant_message`. Además `agent_type` puede traer el nombre con que se lanzó el agente en vez de su tipo, y un gate que filtre por tipo se abstiene sin avisar. Imprime el payload de cada evento y míralo.
- **El `additionalContext` que devuelve un `PreToolUse` se entrega con el resultado de la herramienta, cuando ya corrió.** Influye en lo que el modelo haga después, no en la llamada que lo originó. Para intervenir antes de ejecutar solo valen `deny` (el modelo recibe el motivo y no se ejecuta nada) o reescribir la entrada con `updatedInput`. Compruébalo en el transcript: llamada, contexto del hook, resultado.
- **Ningún bloqueo trae límite de serie.** El harness obedece cada `decision: block` de `SubagentStop` aunque traiga `stop_hook_active: true`; el tope lo pone el hook, dejando pasar cuando el flag es `true`. Igual con un `deny` en `PreToolUse`: sin una marca por `agent_id` que deje pasar el reintento, el agente reintenta sin fin. Compruébalo con un hook que bloquee siempre y cuente las invocaciones.
- **Un proceso en background se espera por su artefacto o su notificación, no por polling.** En versiones anteriores se vio que, en un subagente, `run_in_background` no lo re-invocaba: cerrar el turno «esperando la notificación» lo dejaba parado con el trabajo hecho. Antes de fiarte, pruébalo en la tuya con un `sleep` corto en background y un cierre de turno, sin ningún hook que impida cerrarlo. Bloquea en el mismo turno (primer plano con `timeout` explícito, el default corta a 120 s y el máximo de Bash son 10 min). Lo que no cabe, lo lanza y vigila el padre. Y prohíbe en el brief los bucles `while ...; sleep` caseros.
- **Espera sobre el log, no sobre `pgrep -f`.** El patrón aparece en la línea de comandos del propio waiter y casa consigo mismo: nunca sale. `ps ... | grep -v grep` tampoco sirve para buscar al huérfano (`pgrep` contiene «grep»). Si hay que esperar una condición, usa la herramienta de monitorización del harness o que lo haga el padre: nunca `sleep` en primer plano ni dentro de un brief. `TaskStop` de lo que lanzaste antes de entregar: el agente puede seguir listado como `running` horas después.
- **Un agente en idle no ha progresado necesariamente.** Antes de volver a despachar, mira el artefacto (mtime, `git log`): lo normal es que el trabajo esté hecho. Idle tras una entrega limpia solo indica que está libre; idle con «sigo esperando» es un bloqueo. Para reactivarlo usa `SendMessage`: un informe en texto plano al cerrar el turno no llega a nadie.
- **Hay un solo `refs/stash` por repositorio, compartido entre worktrees.** Dos agentes que hagan `git stash` en worktrees distintos usan la misma pila, y uno puede recuperar con `pop` lo que guardó el otro. `git stash list` lo delata.
- **El scratchpad vive en el directorio temporal del sistema, no en la sesión.** Cerrar la sesión no lo borra, pero un reinicio o una limpieza de `/tmp` sí, y ninguna sesión nueva sabe dónde quedó. Lo que importe más allá de la sesión se guarda en la KB, en un issue o en el repo.
- **No des por hecho que el último mensaje de un subagente llega entero.** Según versión y canal se ha visto recortado a unos miles de caracteres o completo; pruébalo con un informe numerado de tamaño conocido. Un informe largo va a un fichero y se contesta con la ruta. La regla tiene que estar en el prompt del ejecutor y en la reacción del padre; en la prosa sola se queda en comentario ([[Una regla que se cita y se ignora es un comentario|learnings/la-prosa-no-es-enforcement]]).

## Plugins

- **El marketplace de plugins sirve por número de versión, no por contenido.** Si `version` no cambia, el contenido nuevo del repo no llega a ninguna máquina y la respuesta ("ya estás en la última versión") es indistinguible de estar al día. La caché no se refresca sin bump. La única comprobación falsable es un `diff -r` entre la caché y el repo. Remedio estructural: un gate de CI que exija versión distinta cuando cambia el directorio del plugin, o no declarar `version` (ni en `plugin.json` ni en el marketplace) para que se siga por commit. Un marketplace local cargado en sitio no depende de la versión: ahí el `diff -r` no aplica. Tras actualizar, `/reload-plugins` o sesión nueva.

## Windows

Verificados en una máquina con Windows 11 y PowerShell 5.1 los tres primeros; el de `cygpath` se enuncia sin reverificar.

- **PowerShell 5.1 lee `.ps1` en el codepage del sistema (CP-1252), no en UTF-8.** Un carácter no-ASCII **fuera de strings** (una raya larga, un acento en un identificador) rompe el parseo, y el error aparece lejos del culpable (cadena sin terminar, comando no encontrado). Regla: ASCII en el código; UTF-8 solo dentro de strings. *Verificado:* un `.ps1` UTF-8 sin BOM con una raya larga en un argumento sin comillas falló con "falta la cadena en el terminador".
- **`Get-Content -Raw` no lee UTF-8** en 5.1. Usa `[System.IO.File]::ReadAllText($ruta, [System.Text.Encoding]::UTF8)`; para escribir sin BOM, `New-Object System.Text.UTF8Encoding $false` con `WriteAllText`. *Verificado:* el mismo fichero de 4 caracteres dio 7 con `Get-Content -Raw` y 4 con `ReadAllText`.
- **Desde el bash de Git for Windows, `mktemp -d` devuelve `/tmp/...`**, pero un `python` lanzado desde ahí espera una ruta de Windows: pásala con `cygpath -w "$ruta"`.
- **No iteres `$(find ...)` sin comillas sobre nombres con espacios:** el word-splitting los parte; usa `find -print0 | xargs -0` o `pathlib.rglob`.

## Cuándo no aplica

- Si tu versión del harness ya documenta o corrige el comportamiento, manda lo medido hoy: borra o ajusta la regla en vez de conservarla por inercia.
- Estos hechos justifican **comprobar**, no desconfiar de todo: una prueba de un minuto sobre una sesión fresca basta para cada uno.
