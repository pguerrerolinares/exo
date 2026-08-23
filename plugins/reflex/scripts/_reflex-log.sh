#!/usr/bin/env bash
# Helper COMPARTIDO de los reflejos: loguea un DISPARO (no abstenciones) a un JSONL
# persistente, para poder medir FP-rate por reflejo (paso 3 del proyecto reflejos).
# Best-effort por diseño: NUNCA debe romper el warn-only del reflejo que lo llama.
#
# Uso (en un reflejo, justo antes de emitir el aviso):
#   . "$HOME/.claude/hooks/_reflex-log.sh" 2>/dev/null && reflex_log "<id>" "$INPUT" "<payload>" || true
#
# El helper extrae los campos estandar (session_id, agent_id, agent_type, tool) del
# propio INPUT crudo, asi que el reflejo solo pasa su id + el INPUT + un extracto.
# Requiere jq (los reflejos ya lo exigen antes de llegar aqui).

REFLEX_LOG_FILE="${REFLEX_LOG_FILE:-$HOME/.claude/reflex-log.jsonl}"

reflex_log() {
  local reflex="$1" input="$2" payload="${3:-}"
  local ts; ts="$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null)" || ts=""
  printf '%s' "$input" | jq -c \
    --arg ts "$ts" --arg reflex "$reflex" --arg payload "${payload:0:2000}" \
    '{ts:$ts, reflex:$reflex,
      session_id: (.session_id // ""),
      agent_id: (.agent_id // ""),
      agent_type: (.agent_type // ""),
      tool: (.tool_name // ""),
      payload: $payload}' \
    >> "$REFLEX_LOG_FILE" 2>/dev/null || true
}
