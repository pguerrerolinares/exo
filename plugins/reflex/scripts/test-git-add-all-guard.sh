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

# caso heredoc pequeño (650 chars, 5 líneas): regresión ligera, cabe
# holgado en el cap de 2000 aunque el prefijo se calculara mal.
{
  LINE="$(head -c 130 < /dev/zero | tr '\0' 'x')"
  HEREDOC_BODY="$(printf '%s\n%s\n%s\n%s\n%s' "$LINE" "$LINE" "$LINE" "$LINE" "$LINE")"
  CMD="$(printf 'cat <<EOF\n%s\nEOF\ngit add -A' "$HEREDOC_BODY")"
  : > "$TMPLOG"
  make_payload "$CMD" | REFLEX_LOG_FILE="$TMPLOG" bash "$HOOK" >/dev/null 2>&1
  PAYLOAD="$(tail -1 "$TMPLOG" | jq -r '.payload' 2>/dev/null)"
  if printf '%s' "$PAYLOAD" | grep -q 'git add -A'; then
    printf '[PASS] heredoc 650 chars + git add -A → el payload conserva el match\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] heredoc 650 chars + git add -A → el match no aparece en el payload. payload=%s\n' "$PAYLOAD"
    FAIL=$((FAIL+1))
  fi
}

# caso heredoc grande (T2, el que fija el contrato — el que cita el plan
# literalmente: "con un heredoc de 4 KB seguiria fallando"): ~4.5 KB
# repartidos en 42 líneas de 100 chars, como una spec o runbook real
# pegado con un heredoc, delante del git add -A real.
#
# Por qué hacen falta MUCHAS líneas y no una sola larga: `cut -c1-200` (el
# bug original) y `cut -c1-120` (un bug equivalente que este mismo fix
# introdujo y luego corrigió) truncan POR LINEA, no el string completo. Un
# heredoc de una sola línea gigante se trunca a 120/200 chars igual que uno
# corto -- no revienta nada. Lo que revienta el prefijo es que decenas de
# líneas, cada una intacta tras el cut-por-línea, se acumulen: 42 líneas *
# ~120 chars de "prefijo" = ~5000 chars, muy por encima del cap de 2000 del
# helper, y el "git add -A" final cae fuera de la ventana igual que antes.
{
  HEREDOC_BODY=""
  for i in $(seq -w 0 41); do
    LINEA="linea ${i}: $(head -c 100 < /dev/zero | tr '\0' 'x')"
    if [ -z "$HEREDOC_BODY" ]; then
      HEREDOC_BODY="$LINEA"
    else
      HEREDOC_BODY="$(printf '%s\n%s' "$HEREDOC_BODY" "$LINEA")"
    fi
  done
  CMD="$(printf "cat > spec.md <<'EOF'\n%s\nEOF\ngit add -A" "$HEREDOC_BODY")"
  : > "$TMPLOG"
  make_payload "$CMD" | REFLEX_LOG_FILE="$TMPLOG" bash "$HOOK" >/dev/null 2>&1
  PAYLOAD="$(tail -1 "$TMPLOG" | jq -r '.payload' 2>/dev/null)"
  PAYLOAD_LEN="$(printf '%s' "$PAYLOAD" | wc -c)"
  if [ "${#CMD}" -lt 4000 ]; then
    printf '[FAIL] heredoc grande → el comando de prueba mide %d chars, no llega a los 4 KB del plan\n' "${#CMD}"
    FAIL=$((FAIL+1))
  elif printf '%s' "$PAYLOAD" | grep -q 'git add -A'; then
    printf '[PASS] heredoc ~4.5KB/42 líneas + git add -A → el payload (cmd=%d chars, payload=%d bytes) conserva el match\n' "${#CMD}" "$PAYLOAD_LEN"
    PASS=$((PASS+1))
  else
    printf '[FAIL] heredoc ~4.5KB/42 líneas + git add -A → el match no aparece (cmd=%d chars, payload=%d bytes). payload=%s\n' "${#CMD}" "$PAYLOAD_LEN" "$PAYLOAD"
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
