#!/usr/bin/env bash
# PreToolUse (matcher: mcp__basic-memory__read_note|search_notes|build_context|recent_activity):
# INSTRUMENTACION silenciosa de RETRIEVAL. NO es un reflejo: no avisa, no bloquea,
# no emite additionalContext (seria ruido en CADA lectura de la KB). Solo appendea
# una linea JSONL por lectura y sale 0.
#
# Por que existe: basic-memory NO registra accesos (el sqlite solo tiene created/updated).
# Sin esto, el ratio read/write por nota es un blind spot y "¿la curacion se paga?"
# es irresoluble con datos. Ventana de medicion ~2-3 semanas; retirar cuando haya senal.
#
# Log: $HOME/.claude/reflex-retrieval-log.jsonl (override: $REFLEX_RETRIEVAL_LOG).
# Captura padre Y subagente (agent_id/agent_type) sin filtrar: una lectura via memory
# packet de un ejecutor es dato tan valido como un read del padre.
set -uo pipefail

INPUT="$(cat)"
command -v jq >/dev/null 2>&1 || exit 0

LOG_FILE="${REFLEX_RETRIEVAL_LOG:-$HOME/.claude/reflex-retrieval-log.jsonl}"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null)" || TS=""

# target = lo que identifica QUE se leyo, segun la tool:
#   read_note -> identifier ; search_notes -> query ; build_context -> url ; recent_activity -> (vacio)
printf '%s' "$INPUT" | jq -c --arg ts "$TS" '
  {
    ts: $ts,
    event: "read",
    tool: (.tool_name // ""),
    target: (((.tool_input.identifier // .tool_input.query // .tool_input.url // "") | tostring)[0:300]),
    session_id: (.session_id // ""),
    agent_id: (.agent_id // ""),
    agent_type: (.agent_type // "")
  }' >> "$LOG_FILE" 2>/dev/null || true

exit 0
