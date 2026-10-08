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

- **`PostToolUse` de Bash no trae `exit_code`.** Un hook que dependa de él está muerto en producción. El fallo de una herramienta tiene **su propio evento** (`PostToolUseFailure`); no lo deduzcas del evento de éxito.
- **`PreToolUse` evalúa antes de ejecutar.** Un guard que mira el comando completo ve `rm FLAG && merge` entero y lo deniega aunque el `rm` fuera primero. Gate y ejecución van en **comandos separados**: baja el flag en una llamada propia y ejecuta en la siguiente.
- **Los hooks se cargan al arrancar la sesión.** Cambiar o actualizar un hook no basta: hay que abrir sesión nueva para que cuente. Comprueba siempre sobre una sesión fresca.
- **Un guard que casa por texto del comando casa también menciones.** Un heredoc o un `echo` que contenga `git push` puede tumbar la llamada entera. Y resuelve el repo por el `cwd` del payload: `cd dir && git commit` puede denegarse donde `git -C dir commit` pasa.
- **Un "Edit operation failed" de un hook `PostToolUse` puede ser ruido.** Si el `tool_result` del propio edit dice "updated successfully", el aviso es falso: no reverifiques el fichero ni reintentes. Verifica al final del bloque de trabajo, no tras cada edit.
- **Hooks en background a mano:** con `cmd &`, un hijo que hereda stdout retiene el hook hasta que termina. Redirige los tres descriptores, o usa la opción nativa asíncrona si tu versión la ofrece.
- **La telemetría no debe romper lo que instrumenta.** Si el log falla (disco lleno, ruta ausente), eso no debe tumbar el hook ni la herramienta: `>> log 2>/dev/null || true`.

## Concurrencia y estado compartido

- **`>>` concurrente a un JSONL entrelaza las líneas grandes** (por encima de ~4 KB, el tamaño de escritura atómica de una tubería). En una prueba con líneas de varios KB las filas se entrelazaron y `wc -l` seguía dando el total esperado. Usa lock (`flock`) y **valida parseando cada línea, nunca contando**.
- **`git add -A` bajo concurrencia stagea el árbol entero**, incluido lo ajeno a medias, no "lo que tocaste". Arreglo estructural: un worktree por agente y `git -C <ruta>`; add con rutas explícitas.
- **Los subagentes a veces editan por error el checkout principal.** Corre `git status` antes de cada merge: caza el fichero perdido.
- **Ningún agente muta ni revierte estado compartido para limpiar, comparar o falsar.** `git stash` alcanza al árbol de todos; `git checkout -- <fichero>` para quitar un bug inyectado se lleva también edits sin commitear; restaurar un log para borrar tus propias trazas destruye un dato (*un reflejo disparado por tu propia actividad es dato, no suciedad*). Alternativas: `git show <sha>:<ruta>` volcado a un temporal, parche inverso, copia desechable; y falsa solo sobre estado commiteado.
- **No inferir de un estado intermedio.** Leer el working tree de un agente vivo y deducir que abandonó lleva a escribir encima de él. Si dudas, pregúntale con `SendMessage` y espera ([[El padre coordina y valida; el ejecutor implementa|learnings/orquestador-limpio]]).
- **Artefactos temporales de un dispatch, fuera del worktree.** Un directorio temporal del job: así no ensucian el árbol ni el diff.

## Plugins

- **El marketplace de plugins sirve por número de versión, no por contenido.** Si `version` no cambia, el contenido nuevo del repo no llega a ninguna máquina y la respuesta ("ya estás en la última versión") es indistinguible de estar al día. La caché no se refresca sin bump. La única comprobación falsable es un `diff -r` entre la caché y el repo. Remedio estructural: un gate de CI que exija versión distinta cuando cambia el directorio del plugin. Y recuerda que los hooks se cargan al arrancar: tras actualizar, sesión nueva.

## Windows

Verificados en una máquina con Windows 11 y PowerShell 5.1 los tres primeros; el de `cygpath` se enuncia sin reverificar.

- **PowerShell 5.1 lee `.ps1` en el codepage del sistema (CP-1252), no en UTF-8.** Un carácter no-ASCII **fuera de strings** (una raya larga, un acento en un identificador) rompe el parseo, y el error aparece lejos del culpable (cadena sin terminar, comando no encontrado). Regla: ASCII en el código; UTF-8 solo dentro de strings. *Verificado:* un `.ps1` UTF-8 sin BOM con una raya larga en un argumento sin comillas falló con "falta la cadena en el terminador".
- **`Get-Content -Raw` no lee UTF-8** en 5.1. Usa `[System.IO.File]::ReadAllText($ruta, [System.Text.Encoding]::UTF8)`; para escribir sin BOM, `New-Object System.Text.UTF8Encoding $false` con `WriteAllText`. *Verificado:* el mismo fichero de 4 caracteres dio 7 con `Get-Content -Raw` y 4 con `ReadAllText`.
- **Desde el bash de Git for Windows, `mktemp -d` devuelve `/tmp/...`**, pero un `python` lanzado desde ahí espera una ruta de Windows: pásala con `cygpath -w "$ruta"`.
- **No iteres `$(find ...)` sin comillas sobre nombres con espacios:** el word-splitting los parte; usa `find -print0 | xargs -0` o `pathlib.rglob`.

## Cuándo no aplica

- Si tu versión del harness ya documenta o corrige el comportamiento, manda lo medido hoy: borra o ajusta la regla en vez de conservarla por inercia.
- Estos hechos justifican **comprobar**, no desconfiar de todo: una prueba de un minuto sobre una sesión fresca basta para cada uno.
