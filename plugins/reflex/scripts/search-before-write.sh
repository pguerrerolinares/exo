#!/usr/bin/env bash
# PreToolUse (matcher: mcp__basic-memory__write_note): reflejo "search-before-write".
# Warn-only, NUNCA bloquea (exit 0 siempre). Antes de crear una nota NUEVA conviene
# buscar (search_notes) para no duplicar: el contrato (/documenta) es editar la nota
# canonica, no crear una variante.
# Abstencion: como mucho UNA vez por sesion (sentinel). Aplica en padre y subagentes
# (la disciplina es universal); el sentinel por-sesion basta para no hacer nag.
# Capa TRIGGER / clase event-watching del proyecto cerebro+reflejos.
set -uo pipefail

INPUT="$(cat)"
command -v jq >/dev/null 2>&1 || exit 0

SESSION_ID="$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)" || SESSION_ID=""

SENTINEL="/tmp/claude-search-before-write-${SESSION_ID:-nosession}"
# Ya avisado en esta sesion -> calla (abstencion).
[ -f "$SENTINEL" ] && exit 0
touch "$SENTINEL" 2>/dev/null

MSG="⚠️ Reflejo search-before-write: vas a escribir una nota en basic-memory. Si es NUEVA, busca antes (search_notes / search) para no duplicar — el contrato (/documenta) es editar la nota canonica, no crear una variante. Si ya buscaste o es claramente nueva, ignora. (Aviso 1x/sesion.)"

# log del disparo (best-effort, nunca rompe el warn-only)
. "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "search-before-write" "$INPUT" "$(printf '%s' "$INPUT" | jq -r '.tool_input.title // empty' 2>/dev/null)" || true

printf '%s' "$MSG" | jq -Rs '{hookSpecificOutput:{hookEventName:"PreToolUse",additionalContext:.}}'

exit 0
