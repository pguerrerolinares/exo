#!/usr/bin/env bash
# Stop hook: (1) testigo de entrega de reglas de proyecto, (2) recordatorio no bloqueante para documentar la sesion via /document
# (engine exo). Solo recuerda una vez por sesion (sentinel) y solo si hubo
# trabajo real (umbral de transcript).
set -uo pipefail

INPUT="$(cat)"

SESSION_ID="$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)"
TRANSCRIPT="$(printf '%s' "$INPUT" | jq -r '.transcript_path // empty' 2>/dev/null)"

# Sin session_id no podemos deduplicar; salimos sin molestar.
[ -z "$SESSION_ID" ] && exit 0

SENTINEL_DIR="${REMIND_SENTINEL_DIR:-/tmp}"
SENTINEL="${SENTINEL_DIR}/claude-document-reminded-${SESSION_ID}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Testigo: compara lo que SessionStart decidio (ss) con lo que el mod entrego (hb).
# Va ANTES del umbral del recordatorio y con sentinel propio: ninguno silencia al otro.
WITNESS_MSG=""
RULES_DIR="$HOME/.claude/exo-rules"
WSENT="${SENTINEL_DIR}/claude-rules-witness-${SESSION_ID}"
SS="$RULES_DIR/ss-${SESSION_ID}"
HB="$RULES_DIR/hb-${SESSION_ID}"
if [ ! -f "$WSENT" ]; then
  WREASON=""
  if [ ! -f "$SS" ]; then
    WREASON="sin_ss"
  elif [ ! -f "$HB" ]; then
    WREASON="sin_latido"
    WITNESS_MSG="⚠ el mod de reglas de proyecto no cargó (sin latido): esta sesión no lleva reglas en el system prompt"
  else
    SS_ST="$(jq -r '.status // ""' "$SS" 2>/dev/null)"
    SS_N="$(jq -r '.n // 0' "$SS" 2>/dev/null)"
    if HB_ROW="$(jq -r '[(.status // ""), (.n // 0)] | @tsv' "$HB" 2>/dev/null)" && [ -n "$HB_ROW" ]; then
      HB_ST="${HB_ROW%%$'\t'*}"; HB_N="${HB_ROW##*$'\t'}"
    else
      HB_ST="corrupto"; HB_N="?"
    fi
    if { [ "$SS_ST" = "ok" ] && { [ "$HB_ST" != "ok" ] || [ "$HB_N" != "$SS_N" ]; }; } \
       || [ "$HB_ST" = "corrupto" ] \
       || { [ "$SS_ST" = "skip" ] && [ "$HB_ST" = "ok" ]; }; then
      WREASON="no_entrego"
      WITNESS_MSG="⚠ el mod de reglas de proyecto no entregó: SessionStart vio ${SS_ST} n=${SS_N}, el latido dice ${HB_ST} n=${HB_N}"
    elif [ "$SS_ST" = "skip" ] && [ "$HB_ST" = "error" ]; then
      # El mod falló pero no había reglas que entregar: sin ruido en pantalla, con rastro.
      WREASON="hb_error"
    fi
  fi
  if [ -n "$WREASON" ]; then
    touch "$WSENT" 2>/dev/null
    . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null \
      && reflex_log "project-rules-witness" "$INPUT" "reason=${WREASON}${WITNESS_MSG:+ msg=$WITNESS_MSG}" || true
  fi
fi

emit() {  # une testigo y recordatorio en un unico objeto JSON
  local m="$WITNESS_MSG"
  [ -n "${1:-}" ] && m="${m:+$m$'\n'}$1"
  [ -n "$m" ] && jq -cn --arg m "$m" '{systemMessage:$m}'
  exit 0
}

# Ya recordado en esta sesion -> solo testigo.
[ -f "$SENTINEL" ] && emit

# Umbral de "sesion con trabajo real": transcript existe y supera ~50 lineas o ~20KB.
[ -z "$TRANSCRIPT" ] && emit
[ -f "$TRANSCRIPT" ] || emit

LINES="$(wc -l < "$TRANSCRIPT" 2>/dev/null | tr -d ' ')"
BYTES="$(wc -c < "$TRANSCRIPT" 2>/dev/null | tr -d ' ')"
LINES="${LINES:-0}"
BYTES="${BYTES:-0}"

# Si no llega al umbral, NO creamos sentinel todavia (puede crecer en futuros Stop).
if [ "$LINES" -lt 50 ] && [ "$BYTES" -lt 20480 ]; then
  emit
fi

# Supera umbral y no hay sentinel: marcamos y recordamos una sola vez.
touch "$SENTINEL" 2>/dev/null
emit "💾 ¿Cierras sesión? Usa /document para guardar decisiones y aprendizajes en la KB."
