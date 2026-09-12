#!/usr/bin/env bash
# Test standalone para estilo-directo.sh (adaptador SessionStart, directiva de
# estilo de respuesta estática, texto fijo en estilo-directo.md).
# Never-break: exit 0 SIEMPRE. Sin .md legible => abstención silenciosa.
# Fixtures en mktemp -d; nunca toca ~/.claude/reflex-log.jsonl real.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ADAPTER="${SCRIPT_DIR}/estilo-directo.sh"
TEXTO="${SCRIPT_DIR}/estilo-directo.md"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PASS=0
FAIL=0

pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

contains()     { case "$1" in *"$2"*) return 0 ;; *) return 1 ;; esac; }

PAYLOAD1='{"session_id":"x","source":"startup"}'

# =========================================================================
# Caso 1: camino feliz — payload SessionStart típico ⇒ stdout JSON válido,
# hookEventName==SessionStart, additionalContext no vacío con el header,
# log gana 1 línea estilo-directo-emitted con bytes=.
# =========================================================================
{
  LOG1="$TMP/log1.jsonl"
  : > "$LOG1"
  OUT1="$(printf '%s' "$PAYLOAD1" | REFLEX_LOG_FILE="$LOG1" "$ADAPTER")"
  EC1=$?
  CTX1="$(printf '%s' "$OUT1" | jq -r '.hookSpecificOutput.additionalContext // empty' 2>/dev/null)"
  EVT1="$(printf '%s' "$OUT1" | jq -r '.hookSpecificOutput.hookEventName // empty' 2>/dev/null)"
  LOGLINES1="$(wc -l < "$LOG1" | tr -d ' ')"
  LOGLINE1="$(tail -1 "$LOG1")"
  PAYLOADFIELD1="$(printf '%s' "$LOGLINE1" | jq -r '.payload // empty' 2>/dev/null)"
  if [ $EC1 -eq 0 ] \
     && printf '%s' "$OUT1" | jq . >/dev/null 2>&1 \
     && [ "$EVT1" = "SessionStart" ] \
     && [ -n "$CTX1" ] \
     && contains "$CTX1" "## Estilo de respuesta: directo" \
     && [ "$LOGLINES1" -eq 1 ] \
     && [ "$(printf '%s' "$LOGLINE1" | jq -r '.reflex')" = "estilo-directo-emitted" ] \
     && contains "$PAYLOADFIELD1" "bytes="; then
    pass "caso1: payload SessionStart ⇒ JSON+additionalContext+log estilo-directo-emitted"
  else
    fail "caso1: payload SessionStart ⇒ JSON+additionalContext+log estilo-directo-emitted" \
      "ec=$EC1 out=$OUT1 loglines=$LOGLINES1 logline=$LOGLINE1"
  fi
}

# =========================================================================
# Caso 2: abstención — el .md real no existe (movido a backup temporal
# dentro del propio test, restauración garantizada por trap incluso si el
# test falla a medias) ⇒ stdout vacío, exit 0, log con estilo-directo-abstained.
# estilo-directo.sh no soporta ninguna var de entorno de override de ruta
# del .md (misma convención que subagent-inject.sh para su propio texto: sin
# override, solo REFLEX_* para logging/KB), así que se monta moviendo el
# fichero real.
# =========================================================================
{
  BACKUP="$TMP/estilo-directo.md.bak"
  mv "$TEXTO" "$BACKUP"
  trap 'mv -f "'"$BACKUP"'" "'"$TEXTO"'" 2>/dev/null; rm -rf "'"$TMP"'"' EXIT

  LOG2="$TMP/log2.jsonl"
  : > "$LOG2"
  OUT2="$(printf '%s' "$PAYLOAD1" | REFLEX_LOG_FILE="$LOG2" "$ADAPTER")"
  EC2=$?
  LOGLINES2="$(wc -l < "$LOG2" | tr -d ' ')"
  LOGLINE2="$(tail -1 "$LOG2")"

  mv -f "$BACKUP" "$TEXTO"
  trap 'rm -rf "$TMP"' EXIT

  if [ $EC2 -eq 0 ] && [ -z "$OUT2" ] \
     && [ "$LOGLINES2" -eq 1 ] \
     && [ "$(printf '%s' "$LOGLINE2" | jq -r '.reflex')" = "estilo-directo-abstained" ]; then
    pass "caso2: sin .md ⇒ stdout vacío, exit0, log estilo-directo-abstained"
  else
    fail "caso2: sin .md ⇒ stdout vacío, exit0, log estilo-directo-abstained" \
      "ec=$EC2 out=$OUT2 loglines=$LOGLINES2 logline=$LOGLINE2"
  fi
}

echo ""
TOTAL=$((PASS+FAIL))
echo "=== Resultado: ${PASS}/${TOTAL} pasaron ==="
[ $FAIL -eq 0 ] && exit 0 || exit 1
