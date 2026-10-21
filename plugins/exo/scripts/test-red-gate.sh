#!/usr/bin/env bash
# Test standalone para red-gate.sh (PreToolUse SubagentHandback, log-only) y
# para skills/orchestrate/scripts/red-witness. HOME y repo en mktemp -d.
# shellcheck disable=SC2015 # `A && pass || fail`: pass siempre sale 0, no hay doble disparo
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/red-gate.sh"
WITNESS="${SCRIPT_DIR}/../skills/orchestrate/scripts/red-witness"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
export HOME="$TMP/home"
mkdir -p "$HOME/.claude"
export REFLEX_LOG_FILE="$HOME/.claude/reflex-log.jsonl"

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

REPO="$TMP/repo"
mkdir -p "$REPO"
git -C "$REPO" init -q
LOG="$REPO/.superpowers/sdd/red.log"

run() {  # $1 = agent_type, $2 = message
  : > "$REFLEX_LOG_FILE"
  jq -n --arg t "$1" --arg m "$2" --arg c "$REPO" \
    '{session_id:"s", agent_id:"a1", agent_type:$t, cwd:$c, tool_name:"SubagentHandback", tool_input:{message:$m}}' \
    | "$HOOK" >/dev/null 2>&1
  NLOG="$(grep -c '"reflex":"red-gate"' "$REFLEX_LOG_FILE" 2>/dev/null || true)"
}

run "exo:executor" "Status: DONE. RED: red.log"
[ "$NLOG" = 1 ] && pass "loguea red-gate: sin red.log" || fail "loguea red-gate: sin red.log" "n=$NLOG"

mkdir -p "$(dirname "$LOG")"
printf 'RED: t exit=0\nok\n' > "$LOG"
run "exo:executor" "Status: DONE. RED: red.log"
[ "$NLOG" = 1 ] && pass "loguea red-gate: red.log solo con exit=0" || fail "loguea red-gate: red.log solo con exit=0" "n=$NLOG"

printf 'RED: t exit=1\nfail\n' >> "$LOG"
run "exo:executor" "Status: DONE. RED: red.log"
[ "$NLOG" = 0 ] && pass "calla: red.log con exit=1" || fail "calla: red.log con exit=1" "n=$NLOG"

rm -f "$LOG"
for M in "Status: BLOCKED. sin entorno" "Status: NEEDS_CONTEXT" "Status: DONE. RED: n/a"; do
  run "exo:executor" "$M"
  [ "$NLOG" = 0 ] && pass "se abstiene: $M" || fail "se abstiene: $M" "n=$NLOG"
done

run "exo:reviewer" "Status: DONE. RED: red.log"
[ "$NLOG" = 0 ] && pass "se abstiene: agent_type distinto" || fail "se abstiene: agent_type distinto" "n=$NLOG"

run "" "Status: DONE. RED: red.log"
[ "$NLOG" = 0 ] && pass "se abstiene: sin agent_type (padre)" || fail "se abstiene: sin agent_type (padre)" "n=$NLOG"

OUT="$(printf '{"agent_type":"exo:executor","cwd":"%s","tool_input":{"message":"DONE"}}' "$REPO" | "$HOOK" 2>/dev/null)"
[ -z "$OUT" ] && pass "log-only: no emite decisión" || fail "log-only: no emite decisión" "out=$OUT"

# --- red-witness ---
rm -f "$LOG"
( cd "$REPO" && "$WITNESS" bash -c 'echo uno; echo dos; exit 3' >/dev/null 2>&1 ); RC=$?
[ "$RC" = 3 ] && pass "red-witness: preserva el exit real (3)" || fail "red-witness: preserva el exit real" "rc=$RC"
grep -q '^RED: bash -c .* exit=3$' "$LOG" && grep -q '^dos$' "$LOG" \
  && pass "red-witness: anexa cabecera con exit y cola" || fail "red-witness: anexa cabecera con exit y cola" "$(cat "$LOG" 2>/dev/null)"

( cd "$REPO" && "$WITNESS" true >/dev/null 2>&1 ); RC=$?
[ "$RC" = 0 ] && [ "$(grep -c '^RED: ' "$LOG")" = 2 ] \
  && pass "red-witness: anexa (no pisa) y sale 0" || fail "red-witness: anexa (no pisa) y sale 0" "rc=$RC $(cat "$LOG")"

( cd "$REPO" && "$WITNESS" seq 1 50 >/dev/null 2>&1 )
[ "$(grep -c '^[0-9][0-9]*$' "$LOG")" = 20 ] && grep -q '^50$' "$LOG" && ! grep -q '^30$' "$LOG" \
  && pass "red-witness: cola de 20 líneas" || fail "red-witness: cola de 20 líneas" "$(tail -3 "$LOG")"

# El gate lee lo que escribe el witness
( cd "$REPO" && "$WITNESS" false >/dev/null 2>&1 )
run "exo:executor" "Status: DONE. RED: red.log"
[ "$NLOG" = 0 ] && pass "integración: witness con exit≠0 silencia el gate" || fail "integración witness+gate" "n=$NLOG"

printf '\n%d pass, %d fail\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
