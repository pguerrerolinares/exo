#!/usr/bin/env bash
# Test standalone para exo-recall.sh (hook SessionStart). Fixtures en mktemp -d;
# nunca toca el HOME, el índice ni el log reales.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/exo-recall.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
export HOME="$TMP/home"
mkdir -p "$HOME/.claude"
export REFLEX_LOG_FILE="$HOME/.claude/reflex-log.jsonl"

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }
contains() { case "$1" in *"$2"*) return 0 ;; *) return 1 ;; esac; }

run_hook() {  # $1 = JSON de entrada; resto = argumentos de env (VAR=valor, -u VAR)
  local input="$1"; shift
  printf '%s' "$input" | env "$@" "$HOOK" > "$TMP/out.txt" 2>/dev/null
  HOOK_RC=$?
  HOOK_OUT="$(cat "$TMP/out.txt" 2>/dev/null)"
}

# ------------------- H5 (T): la reafirmación tras compactar mira solo la cola ---
# 2.500 líneas: un git-c de sess-x FUERA de la ventana de 2.000 y un
# verify-before-commit de sess-x DENTRO. Solo el segundo tiene que reforzarse.
LOGC="$HOME/.claude/reflex-log.jsonl"
{
  printf '{"ts":"t","reflex":"git-c","session_id":"sess-x","agent_id":"","agent_type":"","tool":"Bash","payload":"viejo"}\n'
  awk 'BEGIN { for (i = 0; i < 2498; i++) printf "{\"ts\":\"t\",\"reflex\":\"zero-residuo\",\"session_id\":\"otra-%d\",\"agent_id\":\"\",\"agent_type\":\"\",\"tool\":\"Bash\",\"payload\":\"x\"}\n", i }'
  printf '{"ts":"t","reflex":"verify-before-commit","session_id":"sess-x","agent_id":"","agent_type":"","tool":"Bash","payload":"nuevo"}\n'
} > "$LOGC"
run_hook '{"session_id":"sess-x","source":"compact"}' EXO_BIN="$TMP/no-existe"
CTX="$(printf '%s' "$HOOK_OUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"
if contains "$CTX" "verifica (corre el cambio)"; then pass "H5: refuerza lo disparado dentro de la ventana"
else fail "H5: refuerza lo disparado dentro de la ventana" "ctx='$CTX'"; fi
if ! contains "$CTX" "git -C X"; then pass "H5: no escanea más allá de la ventana de 2000 líneas"
else fail "H5: no escanea más allá de la ventana de 2000 líneas" "ctx='$CTX'"; fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
