#!/usr/bin/env bash
# Test standalone para git-c-bash.sh (PreToolUse, reflejo git -C).
# Tres modos de salida a asertar: rewrite (updatedInput+allow), warn (additionalContext), silencio.
# REFLEX_LOG_FILE apunta a un tmpfile: los tests NUNCA ensucian ~/.claude/reflex-log.jsonl.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/git-c-bash.sh"

export REFLEX_LOG_FILE="$(mktemp)"
trap 'rm -f "$REFLEX_LOG_FILE"' EXIT

PASS=0
FAIL=0

# Payload PreToolUse mínimo con description y timeout (para asertar preservación).
make_payload() {
  local cmd="$1"
  local cmd_escaped
  cmd_escaped="$(printf '%s' "$cmd" | sed 's/\\/\\\\/g; s/"/\\"/g' | awk 'NR>1{printf "\\n"} {printf "%s", $0}')"
  printf '{"session_id":"test-sid","tool_name":"Bash","tool_input":{"command":"%s","description":"desc-original","timeout":5000},"hook_event_name":"PreToolUse"}' \
    "$cmd_escaped"
}

assert_rewrite() {
  local name="$1" cmd="$2" expected_new="$3"
  local output new decision
  output="$(make_payload "$cmd" | bash "$HOOK" 2>/dev/null)"
  new="$(printf '%s' "$output" | jq -r '.hookSpecificOutput.updatedInput.command // empty' 2>/dev/null)"
  decision="$(printf '%s' "$output" | jq -r '.hookSpecificOutput.permissionDecision // empty' 2>/dev/null)"
  if [ "$new" = "$expected_new" ] && [ "$decision" = "allow" ]; then
    printf '[PASS] %s\n' "$name"; PASS=$((PASS+1))
  else
    printf '[FAIL] %s — esperaba rewrite a «%s»/allow, obtuve command=«%s» decision=«%s»\n' \
      "$name" "$expected_new" "$new" "$decision"
    FAIL=$((FAIL+1))
  fi
}

assert_logged() {  # antes: warn (additionalContext); ahora: log-only
  local name="$1" cmd="$2" before after output
  before="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  output="$(make_payload "$cmd" | bash "$HOOK" 2>/dev/null)"
  after="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  if [ -z "$output" ] && [ "$after" -gt "$before" ] \
     && tail -1 "$REFLEX_LOG_FILE" | jq -e 'select(.reflex=="git-c")' >/dev/null 2>&1; then
    printf '[PASS] %s\n' "$name"; PASS=$((PASS+1))
  else
    printf '[FAIL] %s — esperaba log-only (línea log + stdout vacío). output=%s\n' "$name" "$output"
    FAIL=$((FAIL+1))
  fi
}

assert_silent() {
  local name="$1" cmd="$2"
  local output
  output="$(make_payload "$cmd" | bash "$HOOK" 2>/dev/null)"
  if [ -z "$output" ]; then
    printf '[PASS] %s\n' "$name"; PASS=$((PASS+1))
  else
    printf '[FAIL] %s — esperaba silencio, output: %s\n' "$name" "$output"
    FAIL=$((FAIL+1))
  fi
}

echo "=== test-git-c-bash.sh ==="
echo ""

# --- rewrites (HECHO parseado: cd PATH && git <read-only>) ---
assert_rewrite "cd && git status → rewrite" \
  "cd /repo && git status" \
  "git -C /repo status"
assert_rewrite "cd && git log con flags → rewrite" \
  "cd /home/paul/Documentos/proyectos/code-graph-go && git log --oneline -5" \
  "git -C /home/paul/Documentos/proyectos/code-graph-go log --oneline -5"
assert_rewrite "path con ~ y . → rewrite" \
  "cd ~/proyectos/x.y && git diff --stat" \
  "git -C ~/proyectos/x.y diff --stat"
assert_rewrite "path relativo simple (sin ..) → rewrite" \
  "cd subdir && git status" \
  "git -C subdir status"

# --- preservación del resto de tool_input ---
{
  OUTPUT="$(make_payload "cd /repo && git status" | bash "$HOOK" 2>/dev/null)"
  DESC="$(printf '%s' "$OUTPUT" | jq -r '.hookSpecificOutput.updatedInput.description // empty')"
  TMO="$(printf '%s' "$OUTPUT" | jq -r '.hookSpecificOutput.updatedInput.timeout // empty')"
  if [ "$DESC" = "desc-original" ] && [ "$TMO" = "5000" ]; then
    printf '[PASS] updatedInput preserva description y timeout\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] updatedInput perdió campos: description=«%s» timeout=«%s»\n' "$DESC" "$TMO"
    FAIL=$((FAIL+1))
  fi
}

# --- log del rewrite con id propio ---
{
  : > "$REFLEX_LOG_FILE"
  make_payload "cd /repo && git status" | bash "$HOOK" >/dev/null 2>&1
  if jq -e 'select(.reflex=="git-c-rewrite")' "$REFLEX_LOG_FILE" >/dev/null 2>&1; then
    printf '[PASS] rewrite loguea como git-c-rewrite\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] no hay entrada git-c-rewrite en el log de test\n'; FAIL=$((FAIL+1))
  fi
}

# --- fallback a warn (no elegibles para rewrite, status quo: ahora log-only) ---
assert_logged "git push (mutante, fuera de allowlist) → log-only" \
  "cd /repo && git push"
assert_logged "git add (mutante) → log-only" \
  "cd /repo && git add -A"
assert_logged "git branch (bare puede crear) → log-only" \
  "cd /repo && git branch foo"
assert_logged "pipe en REST → log-only" \
  "cd /repo && git log --oneline | head -5"
assert_logged "chain extra tras git → log-only" \
  "cd /repo && git status && echo done"
assert_logged "redirect en REST → log-only" \
  "cd /repo && git status 2>/dev/null"
assert_logged "path con variable → log-only" \
  'cd "$DIR" && git status'
assert_logged "separador ; → log-only (v1 solo &&)" \
  "cd /repo; git status"
assert_logged "patrón como dato en echo → log-only (FP conocido del warn, NO rewrite)" \
  "echo 'cd /x && git status'"
assert_logged "glob en REST → log-only (se expandiría en el cwd original, no en PATH)" \
  "cd /repo && git ls-files *.md"
assert_logged "cd - (OLDPWD, arg especial de cd) → log-only" \
  "cd - && git status"
assert_logged "componente .. relativo → log-only (cd lógico vs chdir físico divergen con symlinks)" \
  "cd ../sub && git status"
assert_logged "componente .. embebido → log-only" \
  "cd /repo/../otro && git log --oneline"
assert_logged "subcomando fuera del allowlist read-only → log-only" \
  "cd /tmp && git branch tmpbranch"

# --- CDPATH seteado + path relativo → log-only (cd resolvería vía CDPATH, git -C no) ---
{
  BEFORE="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  OUTPUT="$(make_payload "cd subdir && git status" | CDPATH="/tmp" bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  if [ -z "$OUTPUT" ] && [ "$AFTER" -gt "$BEFORE" ] \
     && tail -1 "$REFLEX_LOG_FILE" | jq -e 'select(.reflex=="git-c")' >/dev/null 2>&1; then
    printf '[PASS] CDPATH + path relativo → log-only, sin rewrite\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] CDPATH + path relativo — esperaba log-only sin rewrite, output: %s\n' "$OUTPUT"
    FAIL=$((FAIL+1))
  fi
}

# --- silencios (status quo) ---
assert_silent "git -C ya correcto → silencio" "git -C /repo status"
assert_silent "cd && make && git → silencio (regex no cruza separadores)" "cd /repo && make && git status"
assert_silent "sin git → silencio" "ls -la /repo"

# --- multiline → nunca rewrite (warn o silencio según el regex viejo, pero sin updatedInput) ---
{
  OUTPUT="$(make_payload "$(printf 'cd /repo &&\ngit status')" | bash "$HOOK" 2>/dev/null)"
  if printf '%s' "$OUTPUT" | jq -e '.hookSpecificOutput.updatedInput' >/dev/null 2>&1; then
    printf '[FAIL] multiline produjo rewrite (prohibido)\n'; FAIL=$((FAIL+1))
  else
    printf '[PASS] multiline → sin rewrite\n'; PASS=$((PASS+1))
  fi
}

# --- command vacío → exit 0 sin output ---
{
  PAYLOAD='{"session_id":"test-sid","tool_name":"Bash","tool_input":{"command":""},"hook_event_name":"PreToolUse"}'
  OUTPUT="$(printf '%s' "$PAYLOAD" | bash "$HOOK" 2>/dev/null)"
  EC=$?
  if [ $EC -eq 0 ] && [ -z "$OUTPUT" ]; then
    printf '[PASS] command vacío → exit 0, sin output\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] command vacío → ec=%d output=%s\n' "$EC" "$OUTPUT"; FAIL=$((FAIL+1))
  fi
}

echo ""
TOTAL=$((PASS+FAIL))
echo "=== Resultado: ${PASS}/${TOTAL} pasaron ==="
[ $FAIL -eq 0 ] && exit 0 || exit 1
