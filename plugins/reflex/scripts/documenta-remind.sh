#!/usr/bin/env bash
# Stop hook: recordatorio no bloqueante para documentar la sesion via /documenta
# (engine exo). Solo recuerda una vez por sesion (sentinel) y solo si hubo
# trabajo real (umbral de transcript).
set -uo pipefail

INPUT="$(cat)"

SESSION_ID="$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)"
TRANSCRIPT="$(printf '%s' "$INPUT" | jq -r '.transcript_path // empty' 2>/dev/null)"

# Sin session_id no podemos deduplicar; salimos sin molestar.
[ -z "$SESSION_ID" ] && exit 0

SENTINEL="/tmp/claude-documenta-reminded-${SESSION_ID}"

# Ya recordado en esta sesion -> nada.
[ -f "$SENTINEL" ] && exit 0

# Umbral de "sesion con trabajo real": transcript existe y supera ~50 lineas o ~20KB.
[ -z "$TRANSCRIPT" ] && exit 0
[ -f "$TRANSCRIPT" ] || exit 0

LINES="$(wc -l < "$TRANSCRIPT" 2>/dev/null | tr -d ' ')"
BYTES="$(wc -c < "$TRANSCRIPT" 2>/dev/null | tr -d ' ')"
LINES="${LINES:-0}"
BYTES="${BYTES:-0}"

# Si no llega al umbral, NO creamos sentinel todavia (puede crecer en futuros Stop).
if [ "$LINES" -lt 50 ] && [ "$BYTES" -lt 20480 ]; then
  exit 0
fi

# Supera umbral y no hay sentinel: marcamos y recordamos una sola vez.
touch "$SENTINEL" 2>/dev/null
printf '%s\n' '{"systemMessage":"💾 ¿Cierras sesión? Usa /documenta para guardar decisiones y aprendizajes en la KB."}'

exit 0
