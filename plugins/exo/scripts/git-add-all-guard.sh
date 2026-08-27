#!/usr/bin/env bash
# PreToolUse (matcher: Bash): reflejo "zero-residuo".
# Warn-only, NUNCA bloquea (exit 0 siempre). Detecta `git add -A`, `git add --all`
# y `git add .` y recuerda anadir explicitamente solo los ficheros tocados.
# Regla de Paul (CLAUDE.md / /document): NUNCA `git add -A` — arrastra cambios
# no relacionados y residuo.
#
# POR QUE IMPORTA MAS BAJO CONCURRENCIA: `git add -A` no anade "lo que tocaste",
# anade el ESTADO COMPLETO del working tree. Si otro agente/proceso escribe en el
# mismo repo a la vez, te llevas su trabajo a-medias en tu commit (codigo ajeno/roto,
# historial corrupto). El index es uno solo por working tree -> `git add -A` amplia la
# superficie del race a todo el arbol. `git add <rutas>` da scoping; el fix estructural
# para paralelismo real son git worktrees (cada agente su working tree + index propios).
#
# Por-ocurrencia (sin sentinel): cada ocurrencia es un evento real y corregible.
# Aplica en padre Y subagentes (la regla es universal).
# FP conocido y aceptable (warn-only): el patron puede aparecer como dato
# dentro del comando (echo, grep, comentario). El coste es una advertencia ignorable.
#
# LOG: el payload que se persiste no es un prefijo ciego del comando. Es
# contexto (primeros ~120 chars) + TODAS las ocurrencias (hasta un techo,
# dedup) de la sentencia que de verdad disparo el reflejo, extraidas con el
# MISMO patron de deteccion (via grep -Eo). No solo la primera: con 2+
# ocurrencias en el mismo comando la que ejecuta de verdad puede vivir al
# final (p.ej. una mencion en prosa dentro de un heredoc seguida del "git
# add -A" real), y quedarse con head -1 le esconde esa segunda ocurrencia
# al log. Un comando corto que cabe entero en esos ~120 chars se loguea tal
# cual, sin marcador (duplicar el mismo texto dos veces no informa de
# nada). Motivo: con un heredoc/spec largo delante, un prefijo ciego se
# come el "git add -A" real y el log miente sobre por que disparo.
set -uo pipefail

INPUT="$(cat)"
command -v jq >/dev/null 2>&1 || exit 0

CMD="$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)" || CMD=""
[ -z "$CMD" ] && exit 0

# Patron: git add seguido de -A, --all, o . (con espacio o fin de string tras el argumento).
# No cruza separadores de comando para limitar FP en datos embebidos.
PATRON='git[[:space:]]+(-C[[:space:]]+[^[:space:]]+[[:space:]]+)?add[[:space:]]+(-A|--all|\.)([[:space:]]|$)'
printf '%s' "$CMD" | grep -Eq "$PATRON" || exit 0

# payload del log: prefijo de contexto + TODAS las ocurrencias del PATRON
# (mismo PATRON de arriba via grep -Eo -- si diverge del de deteccion el
# log deja de decir la verdad de por que disparo), hasta MATCH_MAX,
# deduplicadas preservando orden. Comando corto que cabe entero en el
# prefijo -> se loguea entero, sin marcador. Si la extraccion no encuentra
# nada (no deberia pasar), degrada al prefijo solo.
# OJO: el prefijo se saca con expansion de parametro (${CMD:0:120}), NO con
# `cut -c`. `cut -c` trunca POR LINEA, no el string completo -- con un
# comando de muchas lineas cortas cada una sobrevive intacta y el "prefijo"
# real acaba siendo lineas*120 caracteres, reventando el cap del helper y
# comiendose el match otra vez. Es el mismo bug que este fix vino a arreglar.
# OJO 2: el propio PATRON no tiene techo (`-C[[:space:]]+[^[:space:]]+`
# acepta un path de cualquier longitud), asi que ningun MATCH extraido lo
# tiene tampoco. Un match gigante (path absurdo tras -C) puede por si solo
# topar el cap de 2000 del helper y comerse el "add -A" -- la misma brecha,
# otra puerta. Por eso CADA ocurrencia se trunca POR SEPARADO (cabeza+cola,
# NO solo el principio: el fragmento que informa vive al FINAL del match --
# "add -A"/"--all"/".", truncar solo por delante lo tiraria) antes de
# unirla a las demas: una sola ocurrencia larga no puede comerse el cap
# combinado.
# OJO 3: por que TODAS y no solo la primera (`head -1`, el bug de esta
# ronda). Con 2+ ocurrencias del patron en el mismo comando -- p.ej. una
# mencion en prosa dentro de un heredoc ("nunca uses git add . en este
# repo") seguida del "git add -A" real al final -- quedarse con la primera
# loguea la mencion inocua y esconde la invocacion real: exactamente el
# falso positivo benigno que este instrumento existe para medir, entrando
# por otra puerta. Ocurrencias identicas (p.ej. "git add ." repetido tres
# veces) se deduplican preservando orden: repetir el mismo texto no informa
# mas que mostrarlo una vez.
# Nota aparte (no ataja nada, solo lo documenta): estos cortes por indice
# (aqui, PREFIJO y el cap de _reflex-log.sh) cuentan caracteres en locale
# UTF-8 pero bytes en LC_ALL=C -- bajo esa locale el corte puede caer a
# mitad de un caracter multibyte. jq lo tolera (sustituye por el caracter
# de reemplazo, exit 0) y el contrato best-effort aguanta, asi que no hace
# falta blindarlo.
MATCH_HEAD=80
MATCH_TAIL=60
MATCH_MAX=5
if [ "${#CMD}" -le 120 ]; then
  PAYLOAD="$CMD"
else
  PREFIJO="${CMD:0:120}"
  MATCHES="$(printf '%s' "$CMD" | grep -Eo "$PATRON" | head -n "$MATCH_MAX" | awk '!seen[$0]++')"
  MATCH=""
  while IFS= read -r M; do
    [ -z "$M" ] && continue
    if [ "${#M}" -gt $((MATCH_HEAD + MATCH_TAIL)) ]; then
      M="${M:0:MATCH_HEAD}…${M: -MATCH_TAIL}"
    fi
    if [ -z "$MATCH" ]; then
      MATCH="$M"
    else
      MATCH="${MATCH} | ${M}"
    fi
  done <<< "$MATCHES"
  if [ -n "$MATCH" ]; then
    PAYLOAD="${PREFIJO} … ⟨match⟩ ${MATCH}"
  else
    PAYLOAD="$PREFIJO"
  fi
fi

# log del disparo (best-effort, nunca rompe el warn-only)
. "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "zero-residuo" "$INPUT" "$PAYLOAD" || true

exit 0
