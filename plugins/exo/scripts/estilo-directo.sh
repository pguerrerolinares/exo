#!/usr/bin/env bash
# Adaptador SessionStart: inyecta una directiva de estilo de respuesta ESTATICA
# (texto fijo en estilo-directo.md) como additionalContext. Never-break: exit 0
# SIEMPRE. No depende del engine ni de la KB, a diferencia de exo-recall.sh.
set -uo pipefail

# Guard de stdin: evita colgarse si se invoca manualmente sin pipe (mismo patron
# defensivo que subagent-inject.sh). Never-break: sin input no hay nada que hacer.
[ -t 0 ] && exit 0
INPUT="$(cat)"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TEXTO="$SCRIPT_DIR/estilo-directo.md"

if [ ! -r "$TEXTO" ]; then
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "estilo-directo-abstained" "$INPUT" "sin fichero .md" || true
  exit 0
fi

BLOQUE="$(cat "$TEXTO")"
JSON="$(printf '%s' "$BLOQUE" | jq -Rs '{hookSpecificOutput:{hookEventName:"SessionStart", additionalContext:.}}' 2>/dev/null)" || JSON=""

if [ -n "$JSON" ]; then
  bytes="$(printf '%s' "$BLOQUE" | wc -c)"
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "estilo-directo-emitted" "$INPUT" "bytes=$bytes" || true
  printf '%s' "$JSON"
else
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "estilo-directo-failed" "$INPUT" "jq fallo o JSON vacio" || true
fi
exit 0
