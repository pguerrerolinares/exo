#!/usr/bin/env bash
# PreToolUse (matcher: ^SubagentHandback$): gate «RED como artefacto» (#136).
# Un `exo:executor` que entrega sin ninguna línea `exit=[1-9]` en
# <toplevel>/.superpowers/sdd/red.log no corrió su RED con `red-witness`
# (o no hubo RED). Empieza LOG-ONLY: loguea `red-gate`, nunca bloquea; la
# promoción a deny (marker por agent_id, la 2ª llamada pasa) llega tras ≥10
# executors con FP ≤20 % medido en reflex-log.
#
# Por qué PreToolUse y no SubagentStop: con SubagentHandback el report llega
# como `tool_input.message`; `last_assistant_message` de SubagentStop es el
# texto de cierre y dispara cuando el orquestador ya tiene el report
# (hooks.md, SubagentStop).
#
# Abstención: sin jq, `agent_type` ≠ exo:executor, mensaje con
# BLOCKED|NEEDS_CONTEXT|«RED: n/a», o red.log con algún `exit=[1-9]`.
set -uo pipefail

INPUT="$(cat)"
command -v jq >/dev/null 2>&1 || exit 0

ROW="$(printf '%s' "$INPUT" | jq -r '[(.agent_type // ""), (.cwd // ""), ((.tool_input.message // "") | gsub("[\n\r]"; " "))] | join("\u001f")' 2>/dev/null)" || exit 0
IFS=$'\x1f' read -r ATYPE CWD MSG <<< "$ROW"

[ "$ATYPE" = "exo:executor" ] || exit 0
[[ "$MSG" =~ BLOCKED|NEEDS_CONTEXT|RED:[[:space:]]*n/a ]] && exit 0

[ -z "$CWD" ] && CWD="$PWD"
TOP="$(git -C "$CWD" rev-parse --show-toplevel 2>/dev/null)" || TOP="$CWD"
LOG="$TOP/.superpowers/sdd/red.log"
if [ -f "$LOG" ] && grep -qE 'exit=[1-9]' "$LOG" 2>/dev/null; then
  exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=_reflex-log.sh
. "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && { reflex_log "red-gate" "$INPUT" "red.log=${LOG} $([ -f "$LOG" ] && echo sin-exit-no-cero || echo ausente)" || true; }
exit 0
