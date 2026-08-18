#!/usr/bin/env bash
# Adaptador SubagentStart (A1, spec §5.2): stdin payload -> compose-inject -> additionalContext.
# Never-break: exit 0 SIEMPRE. Politica de profundidad v1 (spec §5.3): spawnDepth>1 => no inyectar;
# meta ausente/ilegible => inyectar (S5/S6: el meta puede no existir aun — el default es entregar).
set -uo pipefail

# Guard de stdin: evita colgarse si se invoca manualmente sin pipe (mismo patron
# defensivo que basic-memory-recall.sh). Never-break: sin input no hay nada que hacer.
[ -t 0 ] && exit 0
INPUT="$(cat)"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECTS_DIR="${REFLEX_PROJECTS_DIR:-$HOME/.claude/projects}"
TYPE="$(printf '%s' "$INPUT" | jq -r '.agent_type // empty' 2>/dev/null)" || TYPE=""
if [ -z "$TYPE" ]; then
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "inject-abstained" "$INPUT" "sin agent_type" || true
  exit 0
fi
PERFIL="$(jq -r --arg t "$TYPE" '.[$t] // ._default' "$SCRIPT_DIR/inject-profiles.json" 2>/dev/null)" || PERFIL=""
SID="$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)" || SID=""
AID="$(printf '%s' "$INPUT" | jq -r '.agent_id // empty' 2>/dev/null)" || AID=""
if [ -n "$SID" ] && [ -n "$AID" ]; then
  for m in "$PROJECTS_DIR"/*/"$SID"/subagents/"agent-${AID}.meta.json"; do
    [ -f "$m" ] || continue
    d="$(jq -r '.spawnDepth // 1' "$m" 2>/dev/null)" || d=1
    case "$d" in (*[!0-9]*) d=1 ;; esac
    if [ "$d" -gt 1 ]; then
      . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "inject-skipped-depth" "$INPUT" "type=$TYPE depth=$d" || true
      exit 0
    fi
  done
fi
KB_ARGS=()
[ -n "${REFLEX_INJECT_KB:-}" ] && KB_ARGS=(--kb "$REFLEX_INJECT_KB")
JSON=""
if BLOQUE="$("$SCRIPT_DIR/compose-inject.sh" --type "$TYPE" "${KB_ARGS[@]}" 2>/dev/null)" && [ -n "$BLOQUE" ]; then
  JSON="$(printf '%s' "$BLOQUE" | jq -Rs '{hookSpecificOutput:{hookEventName:"SubagentStart", additionalContext:.}}' 2>/dev/null)" || JSON=""
fi
if [ -n "$JSON" ]; then
  bytes="$(printf '%s' "$BLOQUE" | wc -c)"
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "inject-emitted" "$INPUT" "type=$TYPE perfil=$PERFIL bytes=$bytes" || true
  printf '%s' "$JSON"
else
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "inject-failed" "$INPUT" "type=$TYPE perfil=$PERFIL" || true
fi
exit 0
