#!/usr/bin/env bash
# PreToolUse (matcher: Bash): reflejo "zero-residuo".
# Warn-only, NUNCA bloquea (exit 0 siempre). Detecta `git add -A`, `git add --all`
# y `git add .` y recuerda anadir explicitamente solo los ficheros tocados.
# Regla del dueño de la KB (CLAUDE.md / /document): NUNCA `git add -A` — arrastra cambios
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

# Pre-filtro bash puro (campaña I): el PATRON de este reflejo exige "git"
# literal. Mismo contrato que git-c-bash.sh: sin "git" en el JSON crudo, se
# ahorra el spawn de jq sin poder perder ningún disparo real.
case "$INPUT" in
  *git*) : ;;
  *) exit 0 ;;
esac

command -v jq >/dev/null 2>&1 || exit 0

CMD="$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)" || CMD=""
[ -z "$CMD" ] && exit 0

# Patron: git add seguido de -A, --all, o . (con espacio o fin de string tras el argumento).
# No cruza separadores de comando para limitar FP en datos embebidos.
PATRON='git[[:space:]]+(-C[[:space:]]+[^[:space:]]+[[:space:]]+)?add[[:space:]]+(-A|--all|\.)([[:space:]]|$)'
printf '%s' "$CMD" | grep -Eq "$PATRON" || exit 0

# payload del log: contexto + TODAS las ocurrencias del PATRON (mismo PATRON
# de arriba: si diverge del de deteccion, el log deja de decir por que
# disparo). Contrato y trampas en _truncate-payload.sh. Si el helper no se
# puede cargar, degrada al prefijo: el reflejo es warn-only y nunca rompe.
PAYLOAD="${CMD:0:120}"
. "$(dirname "$0")/_truncate-payload.sh" 2>/dev/null && payload_truncado "$CMD" "$PATRON"

# log del disparo (best-effort, nunca rompe el warn-only)
. "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "zero-residuo" "$INPUT" "$PAYLOAD" || true

exit 0
