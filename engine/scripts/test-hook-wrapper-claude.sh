#!/usr/bin/env bash
# Autotest del wrapper del oráculo: los eventos reinyectados deben llevar
# session_id, agent_id y cwd del payload exterior (spec §6.3), con el estado
# aislado por transcript. Ninguna suite del oráculo lo cubre (sus fixtures no
# tienen subagentes), así que se comprueba aquí con un EXO_BIN que vuelca lo
# que recibe y, para el journal real, con el binario de verdad.
set -uo pipefail

AQUI="$(cd "$(dirname "$0")" && pwd)"
WRAPPER="$AQUI/hook-wrapper-claude.sh"
REAL_BIN="${EXO_BIN:-$AQUI/../target/debug/exo}"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
FALLOS=0
check() { if [ "$2" = 0 ]; then echo "[PASS] $1"; else echo "[FAIL] $1 — $3"; FALLOS=$((FALLOS+1)); fi; }

cat > "$TMP/fake-exo" <<'FAKE'
#!/usr/bin/env bash
jq -nc --arg a "$*" --arg d "${EXO_STATE_DIR:-}" --arg i "$(cat)" '{args:$a, state:$d, in:($i|fromjson? // $i)}' >> "$FAKE_LOG"
FAKE
chmod +x "$TMP/fake-exo"

T="$TMP/t.jsonl"
{
  echo '{"type":"assistant","message":{"content":[{"type":"tool_use","id":"b1","name":"Bash","input":{"command":"pytest"}}]}}'
  echo '{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"b1","is_error":true}]}}'
  echo '{"type":"assistant","message":{"content":[{"type":"tool_use","id":"e1","name":"Edit","input":{"file_path":"x.py"}}]}}'
} > "$T"
PAYLOAD="$(jq -nc --arg t "$T" '{session_id:"S1",cwd:"/w/dir",agent_id:"sub1",transcript_path:$t,tool_name:"Edit",tool_input:{},hook_event_name:"PreToolUse"}')"

export FAKE_LOG="$TMP/fake.jsonl" EXO_PARIDAD_BASE="$TMP/base"
printf '%s' "$PAYLOAD" | EXO_BIN="$TMP/fake-exo" "$WRAPPER"
q() { jq -c "$1" "$FAKE_LOG" 2>/dev/null | tr '\n' ' '; }
ok() { jq -e "$1" "$FAKE_LOG" >/dev/null 2>&1 && echo 0 || echo 1; }

check "3 invocaciones: Failure, PostToolUse, PreToolUse" \
  "$(jq -es '[.[].args] == ["hook claude PostToolUseFailure","hook claude PostToolUse","hook claude PreToolUse"]' "$FAKE_LOG" >/dev/null 2>&1 && echo 0 || echo 1)" "$(q .args)"
check "reinyectados llevan session_id del exterior" \
  "$(jq -es '[.[0:2][].in.session_id] == ["S1","S1"]' "$FAKE_LOG" >/dev/null 2>&1 && echo 0 || echo 1)" "$(q .in.session_id)"
check "reinyectados llevan agent_id del exterior" \
  "$(jq -es '[.[0:2][].in.agent_id] == ["sub1","sub1"]' "$FAKE_LOG" >/dev/null 2>&1 && echo 0 || echo 1)" "$(q .in.agent_id)"
check "reinyectados llevan cwd del exterior" \
  "$(jq -es '[.[0:2][].in.cwd] == ["/w/dir","/w/dir"]' "$FAKE_LOG" >/dev/null 2>&1 && echo 0 || echo 1)" "$(q .in.cwd)"
check "EXO_STATE_DIR aislado por transcript (mismo en las 3, bajo la base)" \
  "$(jq -es --arg b "$TMP/base/" '([.[].state]|unique|length)==1 and (.[0].state|startswith($b))' "$FAKE_LOG" >/dev/null 2>&1 && echo 0 || echo 1)" "$(q .state)"

# Sin agent_id exterior no se inventa uno.
: > "$FAKE_LOG"
jq -c 'del(.agent_id) | .session_id="S2"' <<<"$PAYLOAD" | EXO_BIN="$TMP/fake-exo" "$WRAPPER"
check "sin agent_id exterior, los reinyectados no llevan agent_id" \
  "$(jq -es '[.[0:2][].in | has("agent_id")] == [false,false]' "$FAKE_LOG" >/dev/null 2>&1 && echo 0 || echo 1)" "$(q .in)"

# Journal real: el fichero es el de la sesión y el registro lleva el agent_id.
if [ -x "$REAL_BIN" ]; then
  export EXO_PARIDAD_BASE="$TMP/base-real"
  printf '%s' "$PAYLOAD" | EXO_BIN="$REAL_BIN" "$WRAPPER" >/dev/null
  J="$(ls "$EXO_PARIDAD_BASE"/*/sessions/claude-S1.jsonl 2>/dev/null | head -1)"
  check "journal real: claude-S1.jsonl con registros agent_id=sub1 (ok=false y ok=true)" \
    "$([ -n "$J" ] && jq -es '[.[]|select(.kind=="tool" and .agent_id=="sub1")|.ok] == [false,true]' "$J" >/dev/null 2>&1 && echo 0 || echo 1)" "journal=${J:-ausente}"
fi

[ "$FALLOS" -eq 0 ] && echo "wrapper: OK" || echo "wrapper: FALLA"
exit "$FALLOS"
