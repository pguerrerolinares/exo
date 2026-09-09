#!/usr/bin/env bash
# Analiza reflex-log.jsonl EXCLUYENDO sesiones de test y emite el baseline de
# métricas de reflex v2. READ-ONLY: nunca muta el log.
# Filtro de test: session_id que empieza por "test" (case-insensitive) o payload
# que contiene "LIVE-TEST".
set -uo pipefail
LOG="${REFLEX_LOG_FILE:-$HOME/.claude/reflex-log.jsonl}"
command -v jq >/dev/null 2>&1 || { echo "jq requerido" >&2; exit 1; }
[ -f "$LOG" ] || { echo "no existe $LOG" >&2; exit 1; }

# Integridad: un log con lineas corruptas invalida cualquier metrica (fix spec 2026-08-02 §6).
if ! jq -e . "$LOG" >/dev/null 2>&1; then
  echo "ERROR: $LOG contiene JSON invalido; metricas no fiables. Linea(s) ofensora(s):" >&2
  awk 'NF' "$LOG" | while IFS= read -r l; do printf '%s' "$l" | jq -e . >/dev/null 2>&1 || echo "  $l" >&2; done
  exit 1
fi

FILTER='select(((.session_id // "") | ascii_downcase | startswith("test") | not) and (((.payload // "") | contains("LIVE-TEST")) | not))'
ANY_FAIL=0

echo "== Disparos por reflejo (sin sesiones de test) =="
jq -c "$FILTER" "$LOG" \
  | jq -rs 'group_by(.reflex)[] | "\(.[0].reflex)\t\(length)"' \
  | sort -t "$(printf '\t')" -k2 -nr
RC=$?
if [ "$RC" -ne 0 ]; then
  echo "ERROR: fallo calculando 'Disparos por reflejo' (jq/sort exit $RC)" >&2
  ANY_FAIL=1
fi

echo
echo "== Padre vs subagente por reflejo =="
jq -c "$FILTER" "$LOG" \
  | jq -rs 'group_by(.reflex)[]
      | "\(.[0].reflex)\tpadre=\([.[]|select(.agent_id=="")]|length)\tsubag=\([.[]|select(.agent_id!="")]|length)"'
RC=$?
if [ "$RC" -ne 0 ]; then
  echo "ERROR: fallo calculando 'Padre vs subagente por reflejo' (jq exit $RC)" >&2
  ANY_FAIL=1
fi

echo
echo "== Reincidencia: máx disparos de un reflejo en una sola sesión =="
jq -c "$FILTER" "$LOG" \
  | jq -rs 'group_by(.reflex)[]
      | "\(.[0].reflex)\t\([group_by(.session_id)[] | length] | max)"'
RC=$?
if [ "$RC" -ne 0 ]; then
  echo "ERROR: fallo calculando 'Reincidencia' (jq exit $RC)" >&2
  ANY_FAIL=1
fi

[ "$ANY_FAIL" -ne 0 ] && exit 1
exit 0
