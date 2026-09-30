#!/usr/bin/env bash
# Tests de scripts/check-skill-refs.sh sobre fixtures en tmpdir.
set -uo pipefail
RAIZ="$(git rev-parse --show-toplevel)" || exit 1
if [ -z "$RAIZ" ]; then echo "test-check-skill-refs: sin toplevel" >&2; exit 1; fi
cd "$RAIZ" || exit 1
CHECK="$RAIZ/scripts/check-skill-refs.sh"

T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
PASS=0; FAIL=0
ok()  { echo "PASS: $1"; PASS=$((PASS+1)); }
bad() { echo "FAIL: $1"; FAIL=$((FAIL+1)); }

# refs_validas
P="$T/validas"; mkdir -p "$P/skills/plan" "$P/skills/orchestrate" "$P/skills/recon-first"
printf 'Pasa a `exo:orchestrate`.\nVer exo:recon-first, y exo:plan.\n' > "$P/skills/plan/SKILL.md"
printf '# orchestrate\n' > "$P/skills/orchestrate/SKILL.md"
printf '# recon\n' > "$P/skills/recon-first/SKILL.md"
out="$(bash "$CHECK" "$P" 2>&1)"; rc=$?
if [ $rc -eq 0 ] && [ -z "$out" ]; then ok refs_validas; else bad "refs_validas (rc=$rc out=$out)"; fi

# ref_rota
P="$T/rota"; mkdir -p "$P/skills/plan"
printf 'linea 1\nUsa `exo:planificar`.\n' > "$P/skills/plan/SKILL.md"
out="$(bash "$CHECK" "$P" 2>&1)"; rc=$?
if [ $rc -eq 1 ] && printf '%s' "$out" | grep -q 'SKILL.md:2: exo:planificar no existe'; then ok ref_rota; else bad "ref_rota (rc=$rc out=$out)"; fi

# ref_agente
P="$T/agente"; mkdir -p "$P/skills/plan" "$P/agents"
printf 'subagent_type: exo:executor\n' > "$P/skills/plan/SKILL.md"
printf '# executor\n' > "$P/agents/executor.md"
out="$(bash "$CHECK" "$P" 2>&1)"; rc=$?
if [ $rc -eq 0 ] && [ -z "$out" ]; then ok ref_agente; else bad "ref_agente (rc=$rc out=$out)"; fi

# repo_real
out="$(bash "$CHECK" plugins/exo 2>&1)"; rc=$?
if [ $rc -eq 0 ]; then ok repo_real; else bad "repo_real (rc=$rc out=$out)"; fi

echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
