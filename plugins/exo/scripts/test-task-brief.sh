#!/usr/bin/env bash
# U1 (upstream #1943): task-brief escribe en un workspace por plan, no en la
# raíz plana .superpowers/sdd/. Repo git temporal, dos planes con `## Task 1`.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TB="${SCRIPT_DIR}/../skills/orchestrate/scripts/task-brief"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
git -C "$TMP" init -q
printf '# Plan A\n\n## Task 1: alfa\n\ncuerpo alfa\n' > "$TMP/plan-a.md"
printf '# Plan B\n\n## Task 1: beta\n\ncuerpo beta\n' > "$TMP/plan-b.md"

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

cd "$TMP" || exit 1
outa="$("$TB" plan-a.md 1)"; ra=$?
outb="$("$TB" plan-b.md 1)"; rb=$?
if [ "$ra" = 0 ] && [ "$rb" = 0 ]; then
  pass "ambos briefs generados"
else
  fail "ambos briefs generados" "rc=$ra,$rb: $outa / $outb"
fi

a="$TMP/.superpowers/sdd/plan-a/task-1-brief.md"
b="$TMP/.superpowers/sdd/plan-b/task-1-brief.md"
if [ -s "$a" ] && [ -s "$b" ] && [ "$(dirname "$a")" != "$(dirname "$b")" ]; then
  pass "dos planes, dos directorios"
else
  fail "dos planes, dos directorios" "$(find "$TMP/.superpowers" -type f 2>/dev/null | tr '\n' ' ')"
fi

if grep -q 'alfa' "$a" 2>/dev/null && grep -q 'beta' "$b" 2>/dev/null && ! grep -q 'beta' "$a" 2>/dev/null; then
  pass "cada brief tiene el texto de su plan"
else
  fail "cada brief tiene el texto de su plan" "contenido cruzado o ausente"
fi

if [ ! -e "$TMP/.superpowers/sdd/task-1-brief.md" ]; then
  pass "nada en la raíz plana"
else
  fail "nada en la raíz plana" ".superpowers/sdd/task-1-brief.md existe"
fi

printf 'PASS=%d FAIL=%d\n' "$PASS" "$FAIL"
[ "$FAIL" = 0 ]
