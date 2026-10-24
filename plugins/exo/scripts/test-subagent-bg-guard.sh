#!/usr/bin/env bash
# Test standalone para subagent-bg-guard.sh (SubagentStop, #170): bloquea solo si
# el subagente dejó tareas de fondo PROPIAS vivas. Transcripts sintéticos con los
# formatos reales de tool_result; HOME y ficheros en mktemp -d.
# shellcheck disable=SC2015 # `A && pass || fail`: pass siempre sale 0, no hay doble disparo
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/subagent-bg-guard.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
export HOME="$TMP/home"
mkdir -p "$HOME/.claude"
export REFLEX_LOG_FILE="$HOME/.claude/reflex-log.jsonl"

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

OUT_VIVO="$TMP/vivo.output";   printf 'buscando...\n' > "$OUT_VIVO"
OUT_FIN="$TMP/fin.output";     printf 'hecho\n[exited with code 0]\n' > "$OUT_FIN"

# $1 = id, $2 = ruta .output (formato «movida a background»)
moved() { printf '{"type":"user","message":{"content":[{"type":"tool_result","content":"Command did not complete within its 120s timeout and was moved to the background (ID: %s). Output is being written to: %s. You will be notified when it completes."}]}}\n' "$1" "$2"; }
started() { printf '{"type":"user","message":{"content":[{"type":"tool_result","content":"Command running in background with ID: %s. Output is being written to: %s. Keep working."}]}}\n' "$1" "$2"; }
monitor() { printf '{"type":"user","message":{"content":[{"type":"tool_result","content":"Monitor started (task %s, expires in 30m unless the source ends first)."}]}}\n' "$1"; }
stop() { printf '{"type":"assistant","message":{"content":[{"type":"tool_use","id":"t","name":"TaskStop","input":{"task_id":"%s"}}]}}\n' "$1"; }

run() {  # $1 = transcript, $2 = extra JSON (",k:v")
  OUT="$(printf '{"session_id":"s","agent_id":"a1","agent_type":"exo:executor","agent_transcript_path":"%s"%s}' "$1" "${2:-}" | "$HOOK" 2>/dev/null)"
}
decision() { printf '%s' "$OUT" | jq -r '.decision // empty' 2>/dev/null; }

T="$TMP/t-vivo.jsonl";  moved b6ia30yek "$OUT_VIVO" > "$T"
run "$T"
[ "$(decision)" = block ] && pass "bloquea: tarea movida a background, .output sin marca (fallback)" \
  || fail "bloquea: tarea movida a background, .output sin marca" "out=$OUT"
R="$(printf '%s' "$OUT" | jq -r '.reason')"
case "$R" in *b6ia30yek*"$OUT_VIVO"*TaskStop*BLOCKED*) pass "reason: id, ruta del log, TaskStop y BLOCKED" ;;
  *) fail "reason: id, ruta del log, TaskStop y BLOCKED" "reason=$R" ;; esac
jq -e 'select(.reflex=="subagent-bg-guard")' "$REFLEX_LOG_FILE" >/dev/null 2>&1 \
  && pass "bloqueo logueado en reflex-log" || fail "bloqueo logueado en reflex-log" "$(cat "$REFLEX_LOG_FILE" 2>/dev/null)"

T="$TMP/t-run.jsonl"; started bi1d3rqpi "$OUT_VIVO" > "$T"
run "$T"
[ "$(decision)" = block ] && pass "bloquea: run_in_background propio vivo" || fail "bloquea: run_in_background propio vivo" "out=$OUT"

run "$TMP/t-vivo.jsonl" ',"background_tasks":[{"id":"b6ia30yek","type":"shell","status":"running"}]'
[ "$(decision)" = block ] && pass "bloquea: propia y running en background_tasks" || fail "bloquea: propia y running en background_tasks" "out=$OUT"

run "$TMP/t-vivo.jsonl" ',"background_tasks":[{"id":"b6ia30yek","type":"shell","status":"completed"}]'
[ -z "$OUT" ] && pass "calla: background_tasks la da por terminada" || fail "calla: background_tasks la da por terminada" "out=$OUT"

T="$TMP/t-mon.jsonl"; monitor bmon1 > "$T"
run "$T" ',"background_tasks":[{"id":"bmon1","type":"monitor","status":"running"}]'
[ "$(decision)" = block ] && pass "bloquea: Monitor propio running" || fail "bloquea: Monitor propio running" "out=$OUT"
run "$T"
[ -z "$OUT" ] && pass "calla: Monitor sin background_tasks (no cubierto)" || fail "calla: Monitor sin background_tasks" "out=$OUT"

T="$TMP/t-stop.jsonl"; { moved b6ia30yek "$OUT_VIVO"; stop b6ia30yek; } > "$T"
run "$T"
[ -z "$OUT" ] && pass "calla: TaskStop hecho" || fail "calla: TaskStop hecho" "out=$OUT"

T="$TMP/t-fin.jsonl"; moved b6ia30yek "$OUT_FIN" > "$T"
run "$T"
[ -z "$OUT" ] && pass "calla: .output con [exited with code 0]" || fail "calla: .output con [exited with code 0]" "out=$OUT"

T="$TMP/t-parcial.jsonl"; { moved b6ia30yek "$OUT_FIN"; started bi1d3rqpi "$OUT_VIVO"; stop b6ia30yek; } > "$T"
run "$T"
case "$(printf '%s' "$OUT" | jq -r '.reason // empty')" in *bi1d3rqpi*) pass "lista solo la viva entre varias" ;; *) fail "lista solo la viva entre varias" "out=$OUT" ;; esac

printf '{"type":"user","message":{"content":"nada de fondo"}}\n' > "$TMP/t-nada.jsonl"
run "$TMP/t-nada.jsonl" ',"background_tasks":[{"id":"srv-del-padre","type":"shell","status":"running"}]'
[ -z "$OUT" ] && pass "calla: tarea viva del PADRE (no está en el transcript del subagente)" || fail "calla: tarea del padre" "out=$OUT"

run "$TMP/t-vivo.jsonl" ',"stop_hook_active":true'
[ -z "$OUT" ] && pass "calla: stop_hook_active (sin bucle)" || fail "calla: stop_hook_active" "out=$OUT"

OUT="$(printf '{"session_id":"s","agent_transcript_path":"%s"}' "$TMP/t-vivo.jsonl" | "$HOOK" 2>/dev/null)"
[ -z "$OUT" ] && pass "calla: sin agent_id (padre)" || fail "calla: sin agent_id" "out=$OUT"

run "$TMP/no-existe.jsonl"
[ -z "$OUT" ] && pass "calla: transcript ilegible" || fail "calla: transcript ilegible" "out=$OUT"

printf '{"agent_id":"a1"}' | "$HOOK" >/dev/null 2>&1
[ $? -eq 0 ] && pass "calla: sin agent_transcript_path" || fail "calla: sin agent_transcript_path" "rc=$?"

# Fixture saneado del caso real (#170); `@TASKS@` = directorio de los .output.
FIX="${SCRIPT_DIR}/testdata/subagent-bg-guard/transcript.jsonl"
mkdir -p "$TMP/tasks"
for id in b6ia30yek bi1d3rqpi bquoted00; do printf 'x\n' > "$TMP/tasks/$id.output"; done
T="$TMP/t-fix.jsonl"; sed "s#@TASKS@#$TMP/tasks#g" "$FIX" > "$T"
run "$T"
R="$(printf '%s' "$OUT" | jq -r '.reason // empty')"
case "$R" in *b6ia30yek*bi1d3rqpi*) pass "fixture real: bloquea con las dos propias vivas" ;;
  *) fail "fixture real: bloquea con las dos propias vivas" "out=$OUT" ;; esac
case "$R" in *bquoted00*) fail "calla: resultado citado dentro de otro tool_result" "reason=$R" ;;
  *) pass "calla: resultado citado dentro de otro tool_result (bquoted00)" ;; esac
run "$T" ',"background_tasks":[{"id":"bquoted00","type":"shell","status":"running"}]'
[ -z "$OUT" ] && pass "calla: citado aunque esté running en background_tasks" || fail "calla: citado running" "out=$OUT"

# Sin jq: PATH mínimo con solo `cat` (patrón de test-search-first.sh).
BASH_ABS="$(command -v bash)"
mkdir -p "$TMP/shim"
printf '#!%s\nexec %s "$@"\n' "$BASH_ABS" "$(command -v cat)" > "$TMP/shim/cat"
chmod +x "$TMP/shim/cat"
OUT="$(printf '{"agent_id":"a1","agent_transcript_path":"%s"}' "$TMP/t-vivo.jsonl" | PATH="$TMP/shim" "$BASH_ABS" "$HOOK" 2>/dev/null)"; RC=$?
[ "$RC" = 0 ] && [ -z "$OUT" ] && pass "calla: sin jq en el PATH" || fail "calla: sin jq" "rc=$RC out=$OUT"

printf '\n%d pass, %d fail\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
