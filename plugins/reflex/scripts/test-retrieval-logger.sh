#!/usr/bin/env bash
# Test standalone para retrieval-logger.sh (PreToolUse, instrumentacion silenciosa).
# Casos:
#  1. read_note -> appendea 1 linea JSONL con tool + target(identifier), exit 0, stdout VACIO.
#  2. search_notes -> target = query.
#  3. build_context -> target = url.
#  4. recent_activity -> target vacio, pero linea escrita (evento de lectura).
#  5. subagente (agent_id no vacio) -> tambien loguea (no filtra).
#  6. input malformado / sin jq-fields -> no crashea, exit 0.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/retrieval-logger.sh"

PASS=0; FAIL=0
TMPLOG="$(mktemp)"; export REFLEX_RETRIEVAL_LOG="$TMPLOG"

echo "=== test-retrieval-logger.sh ==="

run() { printf '%s' "$1" | bash "$HOOK"; return $?; }
last() { tail -1 "$TMPLOG"; }
ck() { # ck "<desc>" <cond-exit>
  if [ "$2" -eq 0 ]; then echo "  PASS: $1"; PASS=$((PASS+1)); else echo "  FAIL: $1"; FAIL=$((FAIL+1)); fi
}

# 1. read_note
: > "$TMPLOG"
OUT="$(run '{"session_id":"s1","tool_name":"mcp__basic-memory__read_note","tool_input":{"identifier":"kb-demo/core/doctrina-agentes"}}')"; EC=$?
ck "read_note exit 0" "$EC"
[ -z "$OUT" ]; ck "read_note stdout VACIO (silencioso)" $?
[ "$(last | jq -r '.tool')" = "mcp__basic-memory__read_note" ]; ck "tool correcto" $?
[ "$(last | jq -r '.target')" = "kb-demo/core/doctrina-agentes" ]; ck "target = identifier" $?
[ "$(last | jq -r '.event')" = "read" ]; ck "event=read" $?

# 2. search_notes -> target = query
: > "$TMPLOG"
run '{"session_id":"s1","tool_name":"mcp__basic-memory__search_notes","tool_input":{"query":"RAG vs curacion"}}' >/dev/null
[ "$(last | jq -r '.target')" = "RAG vs curacion" ]; ck "search_notes target = query" $?

# 3. build_context -> target = url
: > "$TMPLOG"
run '{"session_id":"s1","tool_name":"mcp__basic-memory__build_context","tool_input":{"url":"memory://kb-demo/backlog"}}' >/dev/null
[ "$(last | jq -r '.target')" = "memory://kb-demo/backlog" ]; ck "build_context target = url" $?

# 4. recent_activity -> target vacio pero linea escrita
: > "$TMPLOG"
run '{"session_id":"s1","tool_name":"mcp__basic-memory__recent_activity","tool_input":{}}' >/dev/null
[ "$(wc -l < "$TMPLOG")" -eq 1 ]; ck "recent_activity escribe linea" $?
[ "$(last | jq -r '.target')" = "" ]; ck "recent_activity target vacio" $?

# 5. subagente (agent_id) -> tambien loguea
: > "$TMPLOG"
run '{"session_id":"s1","agent_id":"a1","agent_type":"Explore","tool_name":"mcp__basic-memory__read_note","tool_input":{"identifier":"x"}}' >/dev/null
[ "$(last | jq -r '.agent_id')" = "a1" ] && [ "$(last | jq -r '.agent_type')" = "Explore" ]; ck "subagente logueado con agent_id/type" $?

# 6. input malformado -> no crashea, exit 0
: > "$TMPLOG"
run 'no soy json' >/dev/null; EC=$?
ck "input malformado exit 0" "$EC"

rm -f "$TMPLOG"
echo ""
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
