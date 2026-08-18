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
set -uo pipefail

INPUT="$(cat)"
command -v jq >/dev/null 2>&1 || exit 0

CMD="$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)" || CMD=""
[ -z "$CMD" ] && exit 0

# Patron: git add seguido de -A, --all, o . (con espacio o fin de string tras el argumento).
# No cruza separadores de comando para limitar FP en datos embebidos.
printf '%s' "$CMD" | grep -Eq 'git[[:space:]]+(-C[[:space:]]+[^[:space:]]+[[:space:]]+)?add[[:space:]]+(-A|--all|\.)([[:space:]]|$)' || exit 0

# log del disparo (best-effort, nunca rompe el warn-only)
. "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "zero-residuo" "$INPUT" "$(printf '%s' "$CMD" | cut -c1-200)" || true

exit 0
