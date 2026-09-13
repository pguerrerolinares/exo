#!/usr/bin/env bash
# Test standalone para clean-orchestrator-research.sh (PreToolUse, reflejo
# "orquestador limpio") y para el matcher que lo cablea en hooks/hooks.json.
# Warn-only: exit 0 SIEMPRE. Avisa 1x/sesión, solo en el padre, y no ante la
# app local.
# Fixtures en mktemp -d; nunca toca ~/.claude/reflex-log.jsonl real. Los
# sentinels del script viven en /tmp/claude-clean-orch-<session_id>: cada caso
# usa un session_id propio y el trap los borra.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/clean-orchestrator-research.sh"
HOOKS_JSON="${SCRIPT_DIR}/../hooks/hooks.json"

TMP="$(mktemp -d)"
SID="test-clean-orch-$$"
trap 'rm -rf "$TMP"; rm -f /tmp/claude-clean-orch-"$SID"-*' EXIT

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

# corre <session> <tool> <tool_input JSON> [agent_id] -> stdout del hook
corre() {
  local sesion="$1" tool="$2" input="$3" agente="${4:-}"
  jq -nc --arg s "$SID-$sesion" --arg t "$tool" --arg a "$agente" --argjson i "$input" \
    '{session_id:$s, tool_name:$t, tool_input:$i, hook_event_name:"PreToolUse"}
     + (if $a == "" then {} else {agent_id:$a} end)' \
    | REFLEX_LOG_FILE="$TMP/log.jsonl" "$HOOK"
}

avisa() { printf '%s' "$1" | jq -e '.hookSpecificOutput.additionalContext | length > 0' >/dev/null 2>&1; }

# Caso 1: WebSearch en el padre avisa; la segunda vez en la misma sesión calla.
OUT1="$(corre c1 WebSearch '{"query":"late chunking"}')"
OUT1B="$(corre c1 WebFetch '{"url":"https://arxiv.org/abs/2409.04701"}')"
if avisa "$OUT1" && [ -z "$OUT1B" ]; then
  pass "caso1: WebSearch en el padre avisa una vez por sesión"
else
  fail "caso1: WebSearch en el padre avisa una vez por sesión" "out1=$OUT1 out1b=$OUT1B"
fi

# Caso 2: dentro de un subagente calla y NO consume el sentinel.
OUT2="$(corre c2 WebSearch '{"query":"x"}' agente-1)"
OUT2B="$(corre c2 WebSearch '{"query":"x"}')"
if [ -z "$OUT2" ] && avisa "$OUT2B"; then
  pass "caso2: subagente calla sin gastar el aviso del padre"
else
  fail "caso2: subagente calla sin gastar el aviso del padre" "out2=$OUT2 out2b=$OUT2B"
fi

# Caso 3: navegar a la web con el MCP de chrome avisa.
OUT3="$(corre c3 mcp__claude-in-chrome__navigate '{"url":"https://example.com","tabId":1}')"
if avisa "$OUT3"; then
  pass "caso3: navigate de claude-in-chrome a una web avisa"
else
  fail "caso3: navigate de claude-in-chrome a una web avisa" "out3=$OUT3"
fi

# Caso 4: navegar a la app local calla y NO consume el sentinel.
OUT4="$(corre c4 mcp__plugin_playwright_playwright__browser_navigate '{"url":"http://localhost:3000/login"}')"
OUT4B="$(corre c4 mcp__claude-in-chrome__navigate '{"url":"http://127.0.0.1:8080"}')"
OUT4D="$(corre c4 mcp__claude-in-chrome__navigate '{"url":"https://0.0.0.0:8443"}')"
OUT4C="$(corre c4 WebFetch '{"url":"https://docs.rs/clap"}')"
if [ -z "$OUT4" ] && [ -z "$OUT4B" ] && [ -z "$OUT4D" ] && avisa "$OUT4C"; then
  pass "caso4: localhost/127.0.0.1/0.0.0.0 calla sin gastar el aviso"
else
  fail "caso4: localhost/127.0.0.1/0.0.0.0 calla sin gastar el aviso" "out4=$OUT4 out4b=$OUT4B out4d=$OUT4D out4c=$OUT4C"
fi

# Caso 5: el matcher de hooks.json. Claude Code lo evalúa como regex JS sin
# anclar cuando lleva caracteres especiales; las anclas ^...$ van explícitas,
# así que ERE (grep -E) y JS dan lo mismo para este patrón.
MATCHER="$(jq -r '.hooks.PreToolUse[] | select(any(.hooks[]; .command | test("clean-orchestrator-research"))) | .matcher' "$HOOKS_JSON")"
malos=""
for t in WebSearch WebFetch mcp__claude-in-chrome__navigate mcp__claude-in-chrome__get_page_text \
         mcp__plugin_playwright_playwright__browser_navigate mcp__playwright__browser_navigate; do
  printf '%s' "$t" | grep -Eq -- "$MATCHER" || malos="$malos no-casa:$t"
done
for t in Bash WebSearchX mcp__claude-in-chrome__computer \
         mcp__plugin_playwright_playwright__browser_navigate_back \
         mcp__plugin_playwright_playwright__browser_click; do
  printf '%s' "$t" | grep -Eq -- "$MATCHER" && malos="$malos casa:$t"
done
if [ -n "$MATCHER" ] && [ -z "$malos" ]; then
  pass "caso5: el matcher cubre búsqueda y navegación, y nada más"
else
  fail "caso5: el matcher cubre búsqueda y navegación, y nada más" "matcher=$MATCHER;$malos"
fi

# Caso 6: URL sin esquema (mcp__claude-in-chrome__navigate la admite) y en
# mayúsculas — sigue siendo la app local, calla sin gastar el aviso.
OUT6="$(corre c6 mcp__claude-in-chrome__navigate '{"url":"localhost:3000"}')"
OUT6B="$(corre c6 mcp__claude-in-chrome__navigate '{"url":"127.0.0.1:8080/x"}')"
OUT6C="$(corre c6 mcp__claude-in-chrome__navigate '{"url":"LOCALHOST:3000"}')"
OUT6D="$(corre c6 mcp__claude-in-chrome__navigate '{"url":"[::1]:9000"}')"
OUT6E="$(corre c6 WebFetch '{"url":"https://docs.rs/clap"}')"
if [ -z "$OUT6" ] && [ -z "$OUT6B" ] && [ -z "$OUT6C" ] && [ -z "$OUT6D" ] && avisa "$OUT6E"; then
  pass "caso6: localhost/127.0.0.1/[::1] sin esquema (y en mayúsculas) calla sin gastar el aviso"
else
  fail "caso6: localhost/127.0.0.1/[::1] sin esquema (y en mayúsculas) calla sin gastar el aviso" \
    "out6=$OUT6 out6b=$OUT6B out6c=$OUT6C out6d=$OUT6D out6e=$OUT6E"
fi

# Caso 7: navigate con url:"back"/"forward" (navegación por historial del
# propio navegador, no una URL) no es investigación web: calla sin gastar
# el aviso.
OUT7="$(corre c7 mcp__claude-in-chrome__navigate '{"url":"back","tabId":1}')"
OUT7B="$(corre c7 mcp__claude-in-chrome__navigate '{"url":"forward","tabId":1}')"
OUT7C="$(corre c7 WebFetch '{"url":"https://docs.rs/clap"}')"
if [ -z "$OUT7" ] && [ -z "$OUT7B" ] && avisa "$OUT7C"; then
  pass "caso7: navigate back/forward calla sin gastar el aviso"
else
  fail "caso7: navigate back/forward calla sin gastar el aviso" "out7=$OUT7 out7b=$OUT7B out7c=$OUT7C"
fi

echo ""
TOTAL=$((PASS+FAIL))
echo "=== Resultado: ${PASS}/${TOTAL} pasaron ==="
[ $FAIL -eq 0 ] && exit 0 || exit 1
