#!/usr/bin/env bash
# Test standalone para verify-before-commit.sh (PreToolUse + transcript-scan).
# Usa fixtures de transcript JSONL REALES (como test-stuck-loop-pretool.sh) y un
# repo git temporal con staging real para ejercitar el filtro codigo-vs-docs.
#
# El hook corre `git -C "$CWD_INPUT" diff --cached`, asi que cada caso crea un tmp
# repo, hace git add, y pone cwd=$tmp en el payload.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/verify-before-commit.sh"

PASS=0
FAIL=0
TMPDIR_BASE="/tmp/claude-test-verify-$$"
mkdir -p "$TMPDIR_BASE"
TMP_REPOS=()

TMPLOG="$(mktemp)"; trap 'rm -f "$TMPLOG"' EXIT

# Crea un repo git temporal con ficheros staged. Args: nombre, luego pares "fichero:contenido".
# Echo del path del repo.
make_repo() {
  local name="$1"; shift
  local repo="${TMPDIR_BASE}/${name}"
  mkdir -p "$repo"
  git -C "$repo" init -q
  local spec f content
  for spec in "$@"; do
    f="${spec%%:*}"
    content="${spec#*:}"
    printf '%s\n' "$content" > "${repo}/${f}"
    git -C "$repo" add "$f"
  done
  TMP_REPOS+=("$repo")
  printf '%s' "$repo"
}

# Lineas de transcript (forma real de Claude Code).
transcript_tool_use() {
  local id="$1" cmd="$2"
  local cmd_escaped
  cmd_escaped="$(printf '%s' "$cmd" | sed 's/\\/\\\\/g; s/"/\\"/g')"
  printf '{"type":"assistant","message":{"content":[{"type":"tool_use","id":"%s","name":"Bash","input":{"command":"%s"}}]}}\n' \
    "$id" "$cmd_escaped"
}
transcript_tool_result() {
  local tool_use_id="$1" is_error="$2"
  printf '{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"%s","is_error":%s}]}}\n' \
    "$tool_use_id" "$is_error"
}

# Payload PreToolUse. Args: cwd, transcript_path (puede ser vacio), command.
make_payload() {
  local cwd="$1" transcript="$2" cmd="$3"
  local cmd_escaped
  cmd_escaped="$(printf '%s' "$cmd" | sed 's/\\/\\\\/g; s/"/\\"/g')"
  if [ -n "$transcript" ]; then
    printf '{"session_id":"test-sid","cwd":"%s","transcript_path":"%s","tool_name":"Bash","tool_input":{"command":"%s"},"hook_event_name":"PreToolUse"}' \
      "$cwd" "$transcript" "$cmd_escaped"
  else
    printf '{"session_id":"test-sid","cwd":"%s","tool_name":"Bash","tool_input":{"command":"%s"},"hook_event_name":"PreToolUse"}' \
      "$cwd" "$cmd_escaped"
  fi
}

assert_logged() {  # antes: nudge; ahora: log-only
  local name="$1" before="$2" after="$3" output="$4"
  if [ -z "$output" ] && [ "$after" -gt "$before" ] \
     && tail -1 "$TMPLOG" | jq -e 'select(.reflex=="verify-before-done")' >/dev/null 2>&1; then
    printf '[PASS] %s\n' "$name"; PASS=$((PASS+1))
  else
    printf '[FAIL] %s — esperaba log-only (línea log + stdout vacío). output=%s\n' "$name" "$output"
    FAIL=$((FAIL+1))
  fi
}
assert_no_nudge() {
  local name="$1" before="$2" after="$3" output="$4"
  if [ -z "$output" ] && [ "$after" -eq "$before" ]; then
    printf '[PASS] %s\n' "$name"; PASS=$((PASS+1))
  elif printf '%s' "$output" | jq -e '.hookSpecificOutput.additionalContext' >/dev/null 2>&1; then
    printf '[FAIL] %s — NO esperaba nudge, pero salió additionalContext\n' "$name"
    printf '       output: %s\n' "$output"; FAIL=$((FAIL+1))
  else
    printf '[FAIL] %s — NO esperaba log, pero el log creció (before=%s after=%s)\n' "$name" "$before" "$after"
    printf '       output: %s\n' "$output"; FAIL=$((FAIL+1))
  fi
}

echo "=== test-verify-before-commit.sh ==="
echo ""

# ---------------------------------------------------------------------------
# CASO 1 (clave): codigo .py staged + test-runner verde reciente → NO nudge
# ---------------------------------------------------------------------------
{
  REPO="$(make_repo caso1 "foo.py:print('hi')")"
  T="${TMPDIR_BASE}/t1.jsonl"
  transcript_tool_use    "t1" "pytest tests/foo.py" > "$T"
  transcript_tool_result "t1" "false"               >> "$T"
  PAYLOAD="$(make_payload "$REPO" "$T" "git commit -m fix")"
  BEFORE="$(wc -l < "$TMPLOG" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD" | REFLEX_LOG_FILE="$TMPLOG" bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$TMPLOG" 2>/dev/null || echo 0)"
  assert_no_nudge "caso1: .py staged + test verde reciente → silencio" "$BEFORE" "$AFTER" "$OUTPUT"
}

# ---------------------------------------------------------------------------
# CASO 2: codigo .py staged + test-runner rojo reciente → nudge
# ---------------------------------------------------------------------------
{
  REPO="$(make_repo caso2 "foo.py:print('hi')")"
  T="${TMPDIR_BASE}/t2.jsonl"
  transcript_tool_use    "t1" "pytest tests/foo.py" > "$T"
  transcript_tool_result "t1" "true"                >> "$T"
  PAYLOAD="$(make_payload "$REPO" "$T" "git commit -m fix")"
  BEFORE="$(wc -l < "$TMPLOG" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD" | REFLEX_LOG_FILE="$TMPLOG" bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$TMPLOG" 2>/dev/null || echo 0)"
  assert_logged "caso2: .py staged + test rojo reciente → nudge" "$BEFORE" "$AFTER" "$OUTPUT"
}

# ---------------------------------------------------------------------------
# CASO 3: codigo .py staged + sin test-runner en transcript → nudge
# ---------------------------------------------------------------------------
{
  REPO="$(make_repo caso3 "foo.py:print('hi')")"
  T="${TMPDIR_BASE}/t3.jsonl"
  transcript_tool_use    "t1" "ls -la"  > "$T"
  transcript_tool_result "t1" "false"   >> "$T"
  PAYLOAD="$(make_payload "$REPO" "$T" "git commit -m fix")"
  BEFORE="$(wc -l < "$TMPLOG" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD" | REFLEX_LOG_FILE="$TMPLOG" bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$TMPLOG" 2>/dev/null || echo 0)"
  assert_logged "caso3: .py staged + sin test-runner → nudge" "$BEFORE" "$AFTER" "$OUTPUT"
}

# ---------------------------------------------------------------------------
# CASO 4: codigo .py staged + transcript_path ausente → nudge
# ---------------------------------------------------------------------------
{
  REPO="$(make_repo caso4 "foo.py:print('hi')")"
  PAYLOAD="$(make_payload "$REPO" "" "git commit -m fix")"
  BEFORE="$(wc -l < "$TMPLOG" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD" | REFLEX_LOG_FILE="$TMPLOG" bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$TMPLOG" 2>/dev/null || echo 0)"
  assert_logged "caso4: .py staged + transcript ausente → nudge" "$BEFORE" "$AFTER" "$OUTPUT"
}

# ---------------------------------------------------------------------------
# CASO 5: solo docs (.md) staged + sin test → NO nudge (filtro codigo-vs-docs)
# ---------------------------------------------------------------------------
{
  REPO="$(make_repo caso5 "README.md:hola")"
  T="${TMPDIR_BASE}/t5.jsonl"
  transcript_tool_use    "t1" "ls -la"  > "$T"
  transcript_tool_result "t1" "false"   >> "$T"
  PAYLOAD="$(make_payload "$REPO" "$T" "git commit -m docs")"
  BEFORE="$(wc -l < "$TMPLOG" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD" | REFLEX_LOG_FILE="$TMPLOG" bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$TMPLOG" 2>/dev/null || echo 0)"
  assert_no_nudge "caso5: solo docs staged → silencio (filtro código-vs-docs)" "$BEFORE" "$AFTER" "$OUTPUT"
}

# ---------------------------------------------------------------------------
# CASO 6: git commit --no-verify con codigo staged → NO nudge (escape hatch)
# ---------------------------------------------------------------------------
{
  REPO="$(make_repo caso6 "foo.py:print('hi')")"
  T="${TMPDIR_BASE}/t6.jsonl"
  transcript_tool_use    "t1" "ls -la"  > "$T"
  transcript_tool_result "t1" "false"   >> "$T"
  PAYLOAD="$(make_payload "$REPO" "$T" "git commit --no-verify -m wip")"
  BEFORE="$(wc -l < "$TMPLOG" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD" | REFLEX_LOG_FILE="$TMPLOG" bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$TMPLOG" 2>/dev/null || echo 0)"
  assert_no_nudge "caso6: --no-verify → silencio (escape hatch)" "$BEFORE" "$AFTER" "$OUTPUT"
}

# ---------------------------------------------------------------------------
# CASO 7: nada staged → NO nudge
# ---------------------------------------------------------------------------
{
  REPO="${TMPDIR_BASE}/caso7"
  mkdir -p "$REPO"
  git -C "$REPO" init -q
  TMP_REPOS+=("$REPO")
  T="${TMPDIR_BASE}/t7.jsonl"
  transcript_tool_use    "t1" "ls -la"  > "$T"
  transcript_tool_result "t1" "false"   >> "$T"
  PAYLOAD="$(make_payload "$REPO" "$T" "git commit -m empty")"
  BEFORE="$(wc -l < "$TMPLOG" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD" | REFLEX_LOG_FILE="$TMPLOG" bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$TMPLOG" 2>/dev/null || echo 0)"
  assert_no_nudge "caso7: nada staged → silencio" "$BEFORE" "$AFTER" "$OUTPUT"
}

# ---------------------------------------------------------------------------
# CLEANUP
# ---------------------------------------------------------------------------
rm -rf "$TMPDIR_BASE" 2>/dev/null || true

echo ""
TOTAL=$((PASS+FAIL))
echo "=== Resultado: ${PASS}/${TOTAL} pasaron ==="
[ $FAIL -eq 0 ] && exit 0 || exit 1
