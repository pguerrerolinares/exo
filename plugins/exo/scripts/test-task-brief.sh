#!/usr/bin/env bash
# U1 (upstream #1943): task-brief escribe en un workspace por plan, no en la
# raíz plana .superpowers/sdd/. Repo git temporal, dos planes con `## Task 1`.
# shellcheck disable=SC2016 # los backticks de los planes de prueba son literales
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

# Gate de ola (#129): la tarea de la ola K>1 exige `Ola K-1: cierre — verde` en el ledger.
printf '# Plan C\n\n## Task 1: uno\n\n**Files:**\n- Create: `a`\n\n## Task 2: dos\n\n**Files:**\n- Create: `b`\n\n**Interfaces:**\n- Consumes: x @Task 1\n' > "$TMP/plan-c.md"
"$TB" plan-c.md 1 >/dev/null 2>&1; r1=$?
out="$("$TB" plan-c.md 2 2>&1)"; r2=$?
if [ "$r1" = 0 ] && [ "$r2" = 4 ] && [ ! -e "$TMP/.superpowers/sdd/plan-c/task-2-brief.md" ]; then
  pass "ola 2 sin cierre de la 1: exit 4"
else
  fail "ola 2 sin cierre de la 1: exit 4" "r1=$r1 r2=$r2 $out"
fi

ledger="$TMP/.superpowers/sdd/plan-c/progress.md"
printf 'Ola 1: cierre — verde (1/1) @abc\nOla 1: cierre — ROJO (1/2) @abd\n' > "$ledger"
"$TB" plan-c.md 2 >/dev/null 2>&1; r=$?
if [ "$r" = 4 ]; then pass "el último cierre de la ola 1 es rojo: exit 4"; else fail "el último cierre de la ola 1 es rojo: exit 4" "r=$r"; fi

printf 'Ola 1: cierre — ROJO (1/2) @abd\nOla 1: cierre — verde (2/2) @abe\n' > "$ledger"
"$TB" plan-c.md 2 >/dev/null 2>&1; r=$?
if [ "$r" = 0 ] && [ -s "$TMP/.superpowers/sdd/plan-c/task-2-brief.md" ]; then pass "con cierre verde de la ola 1: brief emitido"; else fail "con cierre verde de la ola 1: brief emitido" "r=$r"; fi

printf '# Plan D\n\n## Task 1: a\n\n**Files:**\n- `x`\n\n## Task 2: b\n\n**Files:**\n- `x`\n\n**Interfaces:**\n- Consumes: y @Task 2\n' > "$TMP/plan-d.md"
out="$("$TB" plan-d.md 1 2>&1)"; r=$?
if [ "$r" = 4 ]; then pass "DAG error: exit 4"; else fail "DAG error: exit 4" "r=$r $out"; fi

printf 'PASS=%d FAIL=%d\n' "$PASS" "$FAIL"
[ "$FAIL" = 0 ]
