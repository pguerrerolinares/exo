#!/usr/bin/env bash
# PreToolUse (matcher: Bash): reflejo "zero-residuo".
# Warn-only, NUNCA bloquea (exit 0 siempre). Detecta `git add -A`, `git add --all`
# y `git add .` y recuerda anadir explicitamente solo los ficheros tocados.
# Regla de Paul (CLAUDE.md / /documenta): NUNCA `git add -A` — arrastra cambios
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
# contexto (primeros ~120 chars) + la sentencia que de verdad disparo el
# reflejo, extraida con el MISMO patron de deteccion (via grep -Eo). Un
# comando corto que cabe entero en esos ~120 chars se loguea tal cual, sin
# marcador (duplicar el mismo texto dos veces no informa de nada). Motivo:
# con un heredoc/spec largo delante, un prefijo ciego se come el "git add -A"
# real y el log miente sobre por que disparo.
set -uo pipefail

INPUT="$(cat)"
command -v jq >/dev/null 2>&1 || exit 0

CMD="$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)" || CMD=""
[ -z "$CMD" ] && exit 0

# Patron: git add seguido de -A, --all, o . (con espacio o fin de string tras el argumento).
# No cruza separadores de comando para limitar FP en datos embebidos.
PATRON='git[[:space:]]+(-C[[:space:]]+[^[:space:]]+[[:space:]]+)?add[[:space:]]+(-A|--all|\.)([[:space:]]|$)'
printf '%s' "$CMD" | grep -Eq "$PATRON" || exit 0

# payload del log: prefijo de contexto + la sentencia que disparo (mismo
# PATRON de arriba via grep -Eo -- si diverge del de deteccion el log deja
# de decir la verdad de por que disparo). Comando corto que cabe entero en
# el prefijo -> se loguea entero, sin marcador. Si la extraccion no
# encuentra nada (no deberia pasar), degrada al prefijo solo.
# OJO: el prefijo se saca con expansion de parametro (${CMD:0:120}), NO con
# `cut -c`. `cut -c` trunca POR LINEA, no el string completo -- con un
# comando de muchas lineas cortas cada una sobrevive intacta y el "prefijo"
# real acaba siendo lineas*120 caracteres, reventando el cap del helper y
# comiendose el match otra vez. Es el mismo bug que este fix vino a arreglar.
# OJO 2: el propio PATRON no tiene techo (`-C[[:space:]]+[^[:space:]]+`
# acepta un path de cualquier longitud), asi que el MATCH extraido tampoco
# lo tiene. Un match gigante (path absurdo tras -C) puede por si solo topar
# el cap de 2000 del helper y comerse el "add -A" -- la misma brecha, otra
# puerta. Si el match excede la ventana, se conserva cabeza+cola (NO solo
# el principio): el fragmento que informa vive al FINAL del match (el
# "add -A"/"--all"/"."), truncar solo por delante lo tiraria.
# Nota aparte (no ataja nada, solo lo documenta): estos cortes por indice
# (aqui, PREFIJO y el cap de _reflex-log.sh) cuentan caracteres en locale
# UTF-8 pero bytes en LC_ALL=C -- bajo esa locale el corte puede caer a
# mitad de un caracter multibyte. jq lo tolera (sustituye por el caracter
# de reemplazo, exit 0) y el contrato best-effort aguanta, asi que no hace
# falta blindarlo.
MATCH_HEAD=80
MATCH_TAIL=60
if [ "${#CMD}" -le 120 ]; then
  PAYLOAD="$CMD"
else
  PREFIJO="${CMD:0:120}"
  MATCH="$(printf '%s' "$CMD" | grep -Eo "$PATRON" | head -1)"
  if [ "${#MATCH}" -gt $((MATCH_HEAD + MATCH_TAIL)) ]; then
    MATCH="${MATCH:0:MATCH_HEAD}…${MATCH: -MATCH_TAIL}"
  fi
  if [ -n "$MATCH" ]; then
    PAYLOAD="${PREFIJO} … ⟨match⟩ ${MATCH}"
  else
    PAYLOAD="$PREFIJO"
  fi
fi

# log del disparo (best-effort, nunca rompe el warn-only)
. "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "zero-residuo" "$INPUT" "$PAYLOAD" || true

exit 0
