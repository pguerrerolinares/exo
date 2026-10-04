#!/usr/bin/env bash
# Test del aviso «superpowers sigue habilitado» de exo-recall.sh (SessionStart).
# HOME y proyecto en mktemp -d; sin engine (EXO_BIN inexistente): el aviso va
# en el contexto aunque el hook caiga a fallback.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/exo-recall.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
export HOME="$TMP/home"
PROY="$TMP/proy"
mkdir -p "$HOME/.claude" "$PROY/.claude"
export REFLEX_LOG_FILE="$HOME/.claude/reflex-log.jsonl"

PASS=0; FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }
contains() { case "$1" in *"$2"*) return 0 ;; *) return 1 ;; esac; }

ctx() {  # contexto inyectado con el estado actual de los settings
  printf '{"session_id":"s"}' \
    | env EXO_BIN="$TMP/no-existe" CLAUDE_PROJECT_DIR="$PROY" "$HOOK" 2>/dev/null \
    | jq -r '.hookSpecificOutput.additionalContext'
}
limpia() { rm -f "$HOME/.claude/settings.json" "$PROY/.claude/settings.json" "$PROY/.claude/settings.local.json"; }

limpia
if ! contains "$(ctx)" "superpowers"; then pass "sin settings: no avisa"
else fail "sin settings: no avisa" "avisó"; fi

printf '{\n  "enabledPlugins": {\n    "superpowers@claude-plugins-official": true\n  }\n}\n' > "$HOME/.claude/settings.json"
C="$(ctx)"
if contains "$C" "claude plugin disable superpowers@claude-plugins-official"; then pass "user scope habilitado (JSON multilínea): avisa con el comando"
else fail "user scope habilitado: avisa" "ctx='$C'"; fi

printf '{"enabledPlugins":{"superpowers@claude-plugins-official":false}}' > "$HOME/.claude/settings.json"
if ! contains "$(ctx)" "superpowers"; then pass "user scope false: no avisa"
else fail "user scope false: no avisa" "avisó"; fi

printf '{"enabledPlugins":{"superpowers@claude-plugins-official":true}}' > "$PROY/.claude/settings.json"
if contains "$(ctx)" "claude plugin disable superpowers@"; then pass "project scope habilitado: avisa"
else fail "project scope habilitado: avisa" "no avisó"; fi

printf '{"enabledPlugins":{"superpowers@claude-plugins-official":false}}' > "$PROY/.claude/settings.local.json"
if ! contains "$(ctx)" "superpowers"; then pass "local false pisa a project true: no avisa"
else fail "local false pisa a project true: no avisa" "avisó"; fi

limpia
printf '{"enabledPlugins":{"superpowers@claude-plugins-official":true}}' > "$HOME/.claude/settings.json"
printf '{"enabledPlugins":{"superpowers@claude-plugins-official":false}}' > "$PROY/.claude/settings.json"
if ! contains "$(ctx)" "superpowers"; then pass "project false pisa a user true: no avisa"
else fail "project false pisa a user true: no avisa" "avisó"; fi

printf '\n%d pass, %d fail\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
