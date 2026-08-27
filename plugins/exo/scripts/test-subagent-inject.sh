#!/usr/bin/env bash
# Test standalone para subagent-inject.sh (adaptador SubagentStart, A1 spec §5.2).
# Never-break: exit 0 SIEMPRE. Politica de profundidad v1: spawnDepth>1 => no inyecta;
# meta ausente/sin match => inyecta (default never-break).
# Fixtures en mktemp -d; nunca toca ~/.claude/reflex-log.jsonl ni ~/.claude/projects reales.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ADAPTER="${SCRIPT_DIR}/subagent-inject.sh"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PASS=0
FAIL=0

pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

contains()     { case "$1" in *"$2"*) return 0 ;; *) return 1 ;; esac; }

# --- KB fixture mínima (compose-inject real la usará) ---
KB="$TMP/kb"
mkdir -p "$KB/core" "$KB/projects"
cat > "$KB/core/core-index.md" <<'EOF'
# Core Index (fixture)

## Doctrina compacta
- bullet de doctrina compacta fixture.

## Cores
- core-a: nota core A fixture.
EOF
cat > "$KB/projects/proj.md" <<'EOF'
# Título Proyecto Fixture
Contenido de relleno.
EOF

PAYLOAD1='{"session_id":"test-sid","agent_id":"a1b2","agent_type":"general-purpose","hook_event_name":"SubagentStart","cwd":"/tmp"}'
NO_PROJECTS="$TMP/no-existe-projects"

# =========================================================================
# Caso 1: payload SubagentStart fixture + compose-inject real + KB fixture
# (vía REFLEX_INJECT_KB) ⇒ stdout JSON válido, additionalContext no vacío
# con el header, log gana 1 línea inject-emitted con type=general-purpose
# y bytes=.
# =========================================================================
{
  LOG1="$TMP/log1.jsonl"
  : > "$LOG1"
  OUT1="$(printf '%s' "$PAYLOAD1" | REFLEX_LOG_FILE="$LOG1" REFLEX_INJECT_KB="$KB" REFLEX_PROJECTS_DIR="$NO_PROJECTS" "$ADAPTER")"
  EC1=$?
  CTX1="$(printf '%s' "$OUT1" | jq -r '.hookSpecificOutput.additionalContext // empty' 2>/dev/null)"
  LOGLINES1="$(wc -l < "$LOG1" | tr -d ' ')"
  LOGLINE1="$(tail -1 "$LOG1")"
  PAYLOADFIELD1="$(printf '%s' "$LOGLINE1" | jq -r '.payload // empty' 2>/dev/null)"
  if [ $EC1 -eq 0 ] \
     && printf '%s' "$OUT1" | jq . >/dev/null 2>&1 \
     && [ -n "$CTX1" ] \
     && contains "$CTX1" "=== Contexto inyectado" \
     && [ "$LOGLINES1" -eq 1 ] \
     && [ "$(printf '%s' "$LOGLINE1" | jq -r '.reflex')" = "inject-emitted" ] \
     && contains "$PAYLOADFIELD1" "type=general-purpose" \
     && contains "$PAYLOADFIELD1" "bytes="; then
    pass "caso1: payload general-purpose ⇒ JSON+additionalContext+log inject-emitted"
  else
    fail "caso1: payload general-purpose ⇒ JSON+additionalContext+log inject-emitted" \
      "ec=$EC1 out=$OUT1 loglines=$LOGLINES1 logline=$LOGLINE1"
  fi
}

# =========================================================================
# Caso 2: profundidad — fixture .meta.json con spawnDepth:2 en
# <projects>/x/test-sid/subagents/agent-a1b2.meta.json (REFLEX_PROJECTS_DIR)
# ⇒ stdout VACÍO, exit 0, línea inject-skipped-depth en el log.
# =========================================================================
{
  LOG2="$TMP/log2.jsonl"
  : > "$LOG2"
  PROJ2="$TMP/projects2"
  mkdir -p "$PROJ2/x/test-sid/subagents"
  cat > "$PROJ2/x/test-sid/subagents/agent-a1b2.meta.json" <<'EOF'
{"spawnDepth":2}
EOF
  OUT2="$(printf '%s' "$PAYLOAD1" | REFLEX_LOG_FILE="$LOG2" REFLEX_INJECT_KB="$KB" REFLEX_PROJECTS_DIR="$PROJ2" "$ADAPTER")"
  EC2=$?
  LOGLINES2="$(wc -l < "$LOG2" | tr -d ' ')"
  LOGLINE2="$(tail -1 "$LOG2")"
  if [ $EC2 -eq 0 ] && [ -z "$OUT2" ] \
     && [ "$LOGLINES2" -eq 1 ] \
     && [ "$(printf '%s' "$LOGLINE2" | jq -r '.reflex')" = "inject-skipped-depth" ]; then
    pass "caso2: spawnDepth:2 ⇒ stdout vacío, exit0, log inject-skipped-depth"
  else
    fail "caso2: spawnDepth:2 ⇒ stdout vacío, exit0, log inject-skipped-depth" \
      "ec=$EC2 out=$OUT2 loglines=$LOGLINES2 logline=$LOGLINE2"
  fi
}

# =========================================================================
# Caso 3: meta ausente (sin match para session_id/agent_id) ⇒ inyecta —
# el default never-break.
# =========================================================================
{
  LOG3="$TMP/log3.jsonl"
  : > "$LOG3"
  PROJ3="$TMP/projects3"
  mkdir -p "$PROJ3/x/other-sid/subagents"
  OUT3="$(printf '%s' "$PAYLOAD1" | REFLEX_LOG_FILE="$LOG3" REFLEX_INJECT_KB="$KB" REFLEX_PROJECTS_DIR="$PROJ3" "$ADAPTER")"
  EC3=$?
  CTX3="$(printf '%s' "$OUT3" | jq -r '.hookSpecificOutput.additionalContext // empty' 2>/dev/null)"
  if [ $EC3 -eq 0 ] && [ -n "$CTX3" ] && contains "$CTX3" "=== Contexto inyectado"; then
    pass "caso3: meta ausente/sin match ⇒ inyecta por defecto (never-break)"
  else
    fail "caso3: meta ausente/sin match ⇒ inyecta por defecto (never-break)" "ec=$EC3 out=$OUT3"
  fi
}

# =========================================================================
# Caso 4: componedor roto (REFLEX_INJECT_PROFILES=/dev/null) ⇒ stdout
# vacío, exit 0, línea inject-failed en el log.
# =========================================================================
{
  LOG4="$TMP/log4.jsonl"
  : > "$LOG4"
  OUT4="$(printf '%s' "$PAYLOAD1" | REFLEX_LOG_FILE="$LOG4" REFLEX_INJECT_KB="$KB" REFLEX_PROJECTS_DIR="$NO_PROJECTS" REFLEX_INJECT_PROFILES="/dev/null" "$ADAPTER")"
  EC4=$?
  LOGLINES4="$(wc -l < "$LOG4" | tr -d ' ')"
  LOGLINE4="$(tail -1 "$LOG4")"
  if [ $EC4 -eq 0 ] && [ -z "$OUT4" ] \
     && [ "$LOGLINES4" -eq 1 ] \
     && [ "$(printf '%s' "$LOGLINE4" | jq -r '.reflex')" = "inject-failed" ]; then
    pass "caso4: REFLEX_INJECT_PROFILES=/dev/null ⇒ stdout vacío, exit0, log inject-failed"
  else
    fail "caso4: REFLEX_INJECT_PROFILES=/dev/null ⇒ stdout vacío, exit0, log inject-failed" \
      "ec=$EC4 out=$OUT4 loglines=$LOGLINES4 logline=$LOGLINE4"
  fi
}

# =========================================================================
# Caso 5: payload sin agent_type ⇒ stdout vacío, exit 0, línea inject-abstained
# en el log (I8: never-break no es SILENCIOSO — queda rastro para el gate/consolida).
# =========================================================================
{
  LOG5="$TMP/log5.jsonl"
  : > "$LOG5"
  PAYLOAD5='{"session_id":"test-sid","agent_id":"a1b2","hook_event_name":"SubagentStart","cwd":"/tmp"}'
  OUT5="$(printf '%s' "$PAYLOAD5" | REFLEX_LOG_FILE="$LOG5" REFLEX_INJECT_KB="$KB" REFLEX_PROJECTS_DIR="$NO_PROJECTS" "$ADAPTER")"
  EC5=$?
  LOGLINES5="$(wc -l < "$LOG5" | tr -d ' ')"
  LOGLINE5="$(tail -1 "$LOG5")"
  if [ $EC5 -eq 0 ] && [ -z "$OUT5" ] \
     && [ "$LOGLINES5" -eq 1 ] \
     && [ "$(printf '%s' "$LOGLINE5" | jq -r '.reflex')" = "inject-abstained" ]; then
    pass "caso5: sin agent_type ⇒ stdout vacío, exit0, log inject-abstained"
  else
    fail "caso5: sin agent_type ⇒ stdout vacío, exit0, log inject-abstained" \
      "ec=$EC5 out=$OUT5 loglines=$LOGLINES5 logline=$LOGLINE5"
  fi
}

# =========================================================================
# Caso 6: el JSON de salida del caso 1 pasa
# jq -e '.hookSpecificOutput.hookEventName=="SubagentStart"'.
# =========================================================================
{
  if printf '%s' "$OUT1" | jq -e '.hookSpecificOutput.hookEventName=="SubagentStart"' >/dev/null 2>&1; then
    pass "caso6: hookEventName==SubagentStart en el JSON del caso1"
  else
    fail "caso6: hookEventName==SubagentStart en el JSON del caso1" "out=$OUT1"
  fi
}

echo ""
TOTAL=$((PASS+FAIL))
echo "=== Resultado: ${PASS}/${TOTAL} pasaron ==="
[ $FAIL -eq 0 ] && exit 0 || exit 1
