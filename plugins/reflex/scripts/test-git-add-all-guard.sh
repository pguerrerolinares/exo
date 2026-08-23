#!/usr/bin/env bash
# Test standalone para git-add-all-guard.sh (PreToolUse, reflejo zero-residuo).
# Payloads PreToolUse mínimos por stdin; assert nudge = sale additionalContext,
# assert no-nudge = vacío.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/git-add-all-guard.sh"

PASS=0
FAIL=0

TMPLOG="$(mktemp)"; trap 'rm -f "$TMPLOG"' EXIT

# Genera un payload PreToolUse mínimo. El command se pasa ya JSON-escapado.
# Los saltos de línea reales (heredocs) se pasan a \n literal para que el JSON
# resultante sea válido (mismo patrón que test-git-c-bash.sh).
make_payload() {
  local cmd="$1"
  local cmd_escaped
  cmd_escaped="$(printf '%s' "$cmd" | sed 's/\\/\\\\/g; s/"/\\"/g' | awk 'NR>1{printf "\\n"} {printf "%s", $0}')"
  printf '{"session_id":"test-sid","tool_name":"Bash","tool_input":{"command":"%s"},"hook_event_name":"PreToolUse"}' \
    "$cmd_escaped"
}

assert_logged() {  # antes: nudge; ahora: log-only
  local name="$1" cmd="$2" before after output
  before="$(wc -l < "$TMPLOG" 2>/dev/null || echo 0)"
  output="$(make_payload "$cmd" | REFLEX_LOG_FILE="$TMPLOG" bash "$HOOK" 2>/dev/null)"
  after="$(wc -l < "$TMPLOG" 2>/dev/null || echo 0)"
  if [ -z "$output" ] && [ "$after" -gt "$before" ] \
     && tail -1 "$TMPLOG" | jq -e 'select(.reflex=="zero-residuo")' >/dev/null 2>&1; then
    printf '[PASS] %s\n' "$name"; PASS=$((PASS+1))
  else
    printf '[FAIL] %s — esperaba log-only (línea log + stdout vacío). output=%s\n' "$name" "$output"
    FAIL=$((FAIL+1))
  fi
}

assert_no_nudge() {
  local name="$1" cmd="$2"
  local before after output
  before="$(wc -l < "$TMPLOG" 2>/dev/null || echo 0)"
  output="$(make_payload "$cmd" | REFLEX_LOG_FILE="$TMPLOG" bash "$HOOK" 2>/dev/null)"
  after="$(wc -l < "$TMPLOG" 2>/dev/null || echo 0)"
  if [ -z "$output" ] && [ "$after" -eq "$before" ]; then
    printf '[PASS] %s\n' "$name"
    PASS=$((PASS+1))
  elif printf '%s' "$output" | jq -e '.hookSpecificOutput.additionalContext' >/dev/null 2>&1; then
    printf '[FAIL] %s — NO esperaba nudge, pero salió additionalContext\n' "$name"
    printf '       cmd: %s | output: %s\n' "$cmd" "$output"
    FAIL=$((FAIL+1))
  else
    printf '[FAIL] %s — NO esperaba log, pero el log creció (before=%s after=%s)\n' "$name" "$before" "$after"
    printf '       cmd: %s | output: %s\n' "$cmd" "$output"
    FAIL=$((FAIL+1))
  fi
}

echo "=== test-git-add-all-guard.sh ==="
echo ""

assert_logged   "git add -A → nudge"                "git add -A"
assert_logged   "git add --all → nudge"             "git add --all"
assert_logged   "git add . → nudge"                 "git add ."
assert_logged   "git -C /tmp/x add -A → nudge (fix)" "git -C /tmp/x add -A"
assert_no_nudge "git add foo.txt bar.py → no nudge" "git add foo.txt bar.py"
assert_no_nudge "git commit -m x → no nudge"        "git commit -m x"

# command vacío → exit 0 sin output
{
  PAYLOAD='{"session_id":"test-sid","tool_name":"Bash","tool_input":{"command":""},"hook_event_name":"PreToolUse"}'
  OUTPUT="$(printf '%s' "$PAYLOAD" | bash "$HOOK" 2>/dev/null)"
  EC=$?
  if [ $EC -eq 0 ] && [ -z "$OUTPUT" ]; then
    printf '[PASS] command vacío → exit 0, sin output\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] command vacío → ec=%d output=%s\n' "$EC" "$OUTPUT"
    FAIL=$((FAIL+1))
  fi
}

# caso heredoc (T2): un comando con un heredoc multilínea (>400 chars en total,
# como una spec o un mensaje de commit largo) delante del git add -A real.
# `cut -c1-200` del llamador corta cada LINEA a 200 (no el conjunto), pero el
# segundo cap de `_reflex-log.sh` (antes 500) sí corta la cadena ya compuesta
# de punta a punta: con varias líneas el acumulado supera 500 y el
# "git add -A", que va al final, cae fuera de la ventana capturada.
{
  LINE="$(head -c 130 < /dev/zero | tr '\0' 'x')"
  HEREDOC_BODY="$(printf '%s\n%s\n%s\n%s\n%s' "$LINE" "$LINE" "$LINE" "$LINE" "$LINE")"
  CMD="$(printf 'cat <<EOF\n%s\nEOF\ngit add -A' "$HEREDOC_BODY")"
  : > "$TMPLOG"
  make_payload "$CMD" | REFLEX_LOG_FILE="$TMPLOG" bash "$HOOK" >/dev/null 2>&1
  PAYLOAD="$(tail -1 "$TMPLOG" | jq -r '.payload' 2>/dev/null)"
  if printf '%s' "$PAYLOAD" | grep -q 'git add -A'; then
    printf '[PASS] heredoc >400 chars + git add -A → el payload conserva el match\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] heredoc >400 chars + git add -A → el match no aparece en el payload. payload=%s\n' "$PAYLOAD"
    FAIL=$((FAIL+1))
  fi
}

# caso comando corto (T2): cabe entero dentro de la ventana de prefijo →
# se loguea entero, sin marcador ⟨match⟩ (duplicarlo no informa de nada).
{
  CMD="git add -A"
  : > "$TMPLOG"
  make_payload "$CMD" | REFLEX_LOG_FILE="$TMPLOG" bash "$HOOK" >/dev/null 2>&1
  PAYLOAD="$(tail -1 "$TMPLOG" | jq -r '.payload' 2>/dev/null)"
  if [ "$PAYLOAD" = "$CMD" ]; then
    printf '[PASS] comando corto → payload = comando entero, sin marcador\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] comando corto → esperaba payload="%s", obtuve "%s"\n' "$CMD" "$PAYLOAD"
    FAIL=$((FAIL+1))
  fi
}

echo ""
TOTAL=$((PASS+FAIL))
echo "=== Resultado: ${PASS}/${TOTAL} pasaron ==="
[ $FAIL -eq 0 ] && exit 0 || exit 1
