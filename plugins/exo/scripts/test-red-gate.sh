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
mkdir -p "$HOME/.claude" "$HOME/.exo/state"
export REFLEX_LOG_FILE="$HOME/.exo/state/reflex-log.jsonl"

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

REPO="$TMP/repo"
mkdir -p "$REPO"
git -C "$REPO" init -q
LOG="$REPO/.superpowers/sdd/red.log"
export EXO_STATE_DIR="$HOME/.exo/state"
JOURNAL="$EXO_STATE_DIR/sessions/claude-s_1.jsonl"   # session_id «s/1» saneado, como el engine
mkdir -p "$(dirname "$JOURNAL")"

marca() {  # $1 = agent_id, $2 = code
  printf '{"ts":"t","kind":"mark","tool":"Bash","clase":"Shell","cmd":"","paths":[],"ok":null,"agent_id":"%s","code":"%s"}\n' "$1" "$2" >> "$JOURNAL"
}

run() {  # $1 = agent_type, $2 = message, $3 = agent_id (default a1)
  : > "$REFLEX_LOG_FILE"
  jq -n --arg t "$1" --arg m "$2" --arg c "$REPO" --arg a "${3-a1}" \
    '{session_id:"s/1", agent_id:$a, agent_type:$t, cwd:$c, tool_name:"SubagentHandback", tool_input:{message:$m}}' \
    | "$HOOK" >/dev/null 2>&1
  NLOG="$(grep -c '"reflex":"red-gate"' "$REFLEX_LOG_FILE" 2>/dev/null || true)"
  NOK="$(grep -c '"reflex":"red-gate-ok"' "$REFLEX_LOG_FILE" 2>/dev/null || true)"
  PAYLOAD="$(jq -r 'select(.reflex=="red-gate") | .payload' "$REFLEX_LOG_FILE" 2>/dev/null)"
}

run "exo:executor" "Status: DONE. RED: red.log"
[ "$NLOG" = 1 ] && [[ "$PAYLOAD" == *sin-testigo* ]] && pass "loguea red-gate: sin journal" || fail "loguea red-gate: sin journal" "n=$NLOG p=$PAYLOAD"

marca a1 "red exit=0"
run "exo:executor" "Status: DONE. RED: red.log"
[ "$NLOG" = 1 ] && [[ "$PAYLOAD" == *solo-exit-0* ]] && pass "loguea red-gate: solo testigos con exit=0" || fail "loguea red-gate: solo exit=0" "n=$NLOG p=$PAYLOAD"

marca a1 "red exit=1"
run "exo:executor" "Status: DONE. RED: red.log"
[ "$NLOG" = 0 ] && [ "$NOK" = 1 ] && pass "pasa (red-gate-ok): testigo exit=1 del propio agente" || fail "pasa con testigo propio" "n=$NLOG ok=$NOK"

# El bug de #271: un RED viejo de otro agente o un red.log del repo no cuentan.
mkdir -p "$(dirname "$LOG")"
printf 'RED: t exit=1\nfail\n' > "$LOG"
run "exo:executor" "Status: DONE. RED: red.log" a2
[ "$NLOG" = 1 ] && [[ "$PAYLOAD" == *sin-testigo* ]] && pass "no cuenta: RED de otro agente ni red.log del repo" || fail "no cuenta RED ajeno" "n=$NLOG p=$PAYLOAD"

marca a22 "red exit=1"
run "exo:executor" "Status: DONE. RED: red.log" a2
[ "$NLOG" = 1 ] && pass "no cuenta: agent_id que solo comparte prefijo" || fail "prefijo de agent_id" "n=$NLOG"

printf '{"kind":"tool","agent_id":"a3","cmd":"echo \\"code\\":\\"red exit=1\\"","code":""}\n' >> "$JOURNAL"
run "exo:executor" "Status: DONE. RED: red.log" a3
[ "$NLOG" = 1 ] && pass "no cuenta: «red exit=1» dentro de un registro tool" || fail "registro tool con texto testigo" "n=$NLOG"

FAKE="$TMP/fakebin"; mkdir -p "$FAKE"
printf '#!/bin/sh\necho "exo 0.16.0"\n' > "$FAKE/exo"; chmod +x "$FAKE/exo"
PATH="$FAKE:$PATH" run "exo:executor" "Status: DONE. RED: red.log" a8
[ "$NLOG" = 1 ] && [[ "$PAYLOAD" == "engine-stale engine=0.16.0 "* ]] && pass "engine < 0.17 sin testigo: engine-stale, no sin-testigo" || fail "engine-stale" "n=$NLOG p=$PAYLOAD"

for M in "Status: BLOCKED. sin entorno" "Status: NEEDS_CONTEXT" "Status: DONE. RED: n/a"; do
  run "exo:executor" "$M" a9
  [ "$NLOG" = 0 ] && [ "$NOK" = 0 ] && pass "se abstiene: $M" || fail "se abstiene: $M" "n=$NLOG"
done

run "exo:reviewer" "Status: DONE. RED: red.log" a9
[ "$NLOG" = 0 ] && pass "se abstiene: agent_type distinto" || fail "se abstiene: agent_type distinto" "n=$NLOG"

run "" "Status: DONE. RED: red.log" ""
[ "$NLOG" = 0 ] && pass "se abstiene: sin agent_type (padre)" || fail "se abstiene: sin agent_type (padre)" "n=$NLOG"

run "exo:executor" "Status: DONE. RED: red.log" ""
[ "$NLOG" = 0 ] && [ "$NOK" = 0 ] && pass "se abstiene: executor sin agent_id" || fail "se abstiene: sin agent_id" "n=$NLOG"

OUT="$(printf '{"agent_type":"exo:executor","agent_id":"a1","session_id":"s/1","cwd":"%s","tool_input":{"message":"DONE"}}' "$REPO" | "$HOOK" 2>/dev/null)"
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

ERR="$( cd "$REPO" && "$WITNESS" bash -c 'echo fuera; exit 4' 2>&1 >/dev/null )"
[ "$(printf '%s\n' "$ERR" | tail -n 1)" = "exo-red-witness exit=4" ] \
  && pass "red-witness: línea testigo como última línea de stderr" || fail "red-witness: línea testigo" "err=$ERR"

# Integración: el testigo que imprime el witness, pasado por el engine, silencia el gate.
if command -v exo >/dev/null 2>&1 && exo hook --help >/dev/null 2>&1; then
  : > "$JOURNAL"
  SAL="$( cd "$REPO" && "$WITNESS" false 2>&1 )"
  jq -n --arg e "Exit code 1
$SAL" '{session_id:"s/1", agent_id:"a7", cwd:"/x", hook_event_name:"PostToolUseFailure", tool_name:"Bash", tool_input:{command:"red-witness false"}, error:$e}' \
    | exo hook claude PostToolUseFailure >/dev/null 2>&1
  run "exo:executor" "Status: DONE. RED: red.log" a7
  [ "$NLOG" = 0 ] && [ "$NOK" = 1 ] && pass "integración: witness → exo hook → gate" || fail "integración witness+engine+gate" "n=$NLOG ok=$NOK p=$PAYLOAD (¿exo en PATH < 0.17?)"
else
  printf '[SKIP] integración witness+engine+gate: sin exo en PATH\n'
fi

printf '\n%d pass, %d fail\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
