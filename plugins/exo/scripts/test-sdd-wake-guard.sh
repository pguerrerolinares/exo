#!/usr/bin/env bash
# Test standalone para sdd-wake-guard.sh (Stop): bloquea solo con ledger SDD
# fresco + tarea de fondo viva no-teammate + sin cron. HOME y cwd en mktemp -d.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/sdd-wake-guard.sh"
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
mkdir -p "$REPO/.superpowers/sdd/plan-a"
echo '# SDD ledger — plan: docs/plan-a.md' > "$REPO/.superpowers/sdd/plan-a/progress.md"

SHELL_VIVA='{"id":"bjegyaj4i","type":"shell","status":"running"}'
TEAMMATE='{"id":"t6","type":"teammate","status":"running"}'
CRON='{"id":"9e058e57","cron":"43 21 19 10 *","recurring":false}'

run() {  # $1 = background_tasks JSON, $2 = session_crons JSON, $3 = extra (",k:v"), $4 = cwd
  OUT="$(printf '{"session_id":"s","cwd":"%s","background_tasks":%s,"session_crons":%s%s}' \
    "${4:-$REPO}" "$1" "$2" "${3:-}" | "$HOOK" 2>/dev/null)"
}
decision() { printf '%s' "$OUT" | jq -r '.decision // empty' 2>/dev/null; }

run "[$SHELL_VIVA]" '[]'
if [ "$(decision)" = "block" ]; then pass "bloquea: shell viva, sin cron, ledger fresco"
else fail "bloquea: shell viva, sin cron, ledger fresco" "out=$OUT"; fi

R="$(printf '%s' "$OUT" | jq -r '.reason')"
case "$R" in *bjegyaj4i*CronCreate*skills/orchestrate/despertar.md*) pass "reason: cita la tarea, CronCreate y la ruta de despertar.md" ;;
  *) fail "reason: cita la tarea, CronCreate y la ruta de despertar.md" "reason=$R" ;; esac
D="${R##*sigue }"; D="${D%%»*}"
if [ -f "$D" ]; then pass "reason: la ruta de despertar.md existe"
else fail "reason: la ruta de despertar.md existe" "ruta=$D"; fi

if jq -e 'select(.reflex=="sdd-wake-guard")' "$REFLEX_LOG_FILE" >/dev/null 2>&1; then pass "bloqueo logueado en reflex-log"
else fail "bloqueo logueado en reflex-log" "$(cat "$REFLEX_LOG_FILE" 2>/dev/null)"; fi

run "[$SHELL_VIVA]" "[$CRON]"
if [ -z "$OUT" ]; then pass "calla: hay un cron armado"; else fail "calla: hay un cron armado" "out=$OUT"; fi

run "[$TEAMMATE]" '[]'
if [ -z "$OUT" ]; then pass "calla: solo teammates (salen running aunque estén idle)"; else fail "calla: solo teammates" "out=$OUT"; fi

run '[{"id":"x","type":"shell","status":"completed"}]' '[]'
if [ -z "$OUT" ]; then pass "calla: tarea ya terminada"; else fail "calla: tarea ya terminada" "out=$OUT"; fi

run "[$SHELL_VIVA]" '[]' ',"stop_hook_active":true'
if [ -z "$OUT" ]; then pass "calla: stop_hook_active (sin bucle)"; else fail "calla: stop_hook_active" "out=$OUT"; fi

run "[$SHELL_VIVA]" '[]' ',"agent_id":"a1"'
if [ -z "$OUT" ]; then pass "calla: subagente"; else fail "calla: subagente" "out=$OUT"; fi

mkdir -p "$TMP/sin-sdd"
run "[$SHELL_VIVA]" '[]' '' "$TMP/sin-sdd"
if [ -z "$OUT" ]; then pass "calla: sin .superpowers/sdd"; else fail "calla: sin .superpowers/sdd" "out=$OUT"; fi

touch -t 202001010000 "$REPO/.superpowers/sdd/plan-a/progress.md"
run "[$SHELL_VIVA]" '[]'
if [ -z "$OUT" ]; then pass "calla: ledger viejo (> 6 h)"; else fail "calla: ledger viejo" "out=$OUT"; fi
touch "$REPO/.superpowers/sdd/plan-a/progress.md"

OUT="$(printf '{"session_id":"s","cwd":"%s","background_tasks":[%s]}' "$REPO" "$SHELL_VIVA" | "$HOOK" 2>/dev/null)"
if [ "$(decision)" = "block" ]; then pass "bloquea: session_crons ausente cuenta como vacío"
else fail "bloquea: session_crons ausente" "out=$OUT"; fi

OUT="$(printf 'no es json' | "$HOOK" 2>/dev/null)"; EC=$?
if [ "$EC" -eq 0 ] && [ -z "$OUT" ]; then pass "never-block: stdin inválido"
else fail "never-block: stdin inválido" "ec=$EC out=$OUT"; fi

printf '%d/%d passing\n' "$PASS" "$((PASS+FAIL))"
[ "$FAIL" -eq 0 ]
