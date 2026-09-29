#!/usr/bin/env bash
# Tests de scripts/upstream-reconcile.sh con repos git temporales.
set -uo pipefail

RAIZ="$(git rev-parse --show-toplevel)" || exit 1
SCRIPT="$RAIZ/scripts/upstream-reconcile.sh"
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "PASS: $1"; }
bad() { FAIL=$((FAIL+1)); echo "FAIL: $1"; }

TMPS=()
trap 'rm -rf "${TMPS[@]}"' EXIT

# Repo temporal con identidad local, inmune a la config global.
nuevo_repo() {
  local d; d="$(mktemp -d)"; TMPS+=("$d")
  git -C "$d" init -q -b main
  git -C "$d" config user.name t; git -C "$d" config user.email t@t
  git -C "$d" config commit.gpgsign false
  git -C "$d" commit -q --allow-empty -m "init"
  echo "$d"
}
commit() { git -C "$1" commit -q --allow-empty -m "$2"; }
cabecera='| PR | skill | triage | estado | motivo | hash |
|----|-------|--------|--------|--------|------|'
ledger() { # $1 dir, resto: filas
  local d="$1"; shift
  { printf 'upstream_tag: v1.0.0\n\n%s\n' "$cabecera"; printf '%s\n' "$@"; printf '\n## Divergencias\n\n| id | skill exo | qué diverge | motivo | fuente |\n|--|--|--|--|--|\n| D1 | x | y | z | w |\n'; } > "$d/ledger.md"
}

# propuesto_a_portado
d="$(nuevo_repo)"; commit "$d" "port(upstream#1943): ledger con scope de plan"
ledger "$d" '| #1943 | orchestrate | aplica | propuesto | m | abc123 |'
h="$(git -C "$d" rev-parse --short HEAD)"
"$SCRIPT" "$d/ledger.md" >/dev/null
if grep -qF "| portado |" "$d/ledger.md" && grep -qF "$h" "$d/ledger.md" && ! grep -q propuesto "$d/ledger.md"; then ok propuesto_a_portado; else bad propuesto_a_portado; fi

# sin_evidencia
d="$(nuevo_repo)"
ledger "$d" '| #1943 | orchestrate | aplica | propuesto | m | abc123 |'
out="$("$SCRIPT" "$d/ledger.md")"
if grep -qF "| propuesto |" "$d/ledger.md" && [ "$out" = "propuesto-sin-evidencia #1943 orchestrate" ]; then ok sin_evidencia; else bad sin_evidencia; fi

# solo_su_pr
d="$(nuevo_repo)"; commit "$d" "port(upstream#19): otro"
ledger "$d" '| #1943 | orchestrate | aplica | propuesto | m | abc123 |'
"$SCRIPT" "$d/ledger.md" >/dev/null
a=0; grep -qF "| propuesto |" "$d/ledger.md" && a=1
d2="$(nuevo_repo)"; commit "$d2" "port(upstream#1943): otro"
ledger "$d2" '| #19 | orchestrate | aplica | propuesto | m | abc123 |'
"$SCRIPT" "$d2/ledger.md" >/dev/null
b=0; grep -qF "| propuesto |" "$d2/ledger.md" && b=1
if [ $a = 1 ] && [ $b = 1 ]; then ok solo_su_pr; else bad solo_su_pr; fi

# otras_filas_intactas
d="$(nuevo_repo)"; commit "$d" "port(upstream#1): x"
ledger "$d" '| #2 | a | no aplica | rechazado | m | h1 |' '|#3|b|duda|pendiente|m|h2|' '|  #4 |  c | ya cubierto |  —  | m |  |' '| #1 | d | aplica | propuesto | m | zz |'
cp "$d/ledger.md" "$d/antes.md"
"$SCRIPT" "$d/ledger.md" >/dev/null
if diff <(grep -vF '#1 ' "$d/antes.md") <(grep -vF '#1 ' "$d/ledger.md") >/dev/null \
   && grep -qF "| portado |" "$d/ledger.md"; then ok otras_filas_intactas; else bad otras_filas_intactas; fi

# ledger_roto
d="$(nuevo_repo)"; printf '| PR | skill | triage | estado | motivo | hash |\n|--|--|--|--|--|--|\n' > "$d/l.md"
"$SCRIPT" "$d/l.md" >/dev/null 2>&1; r1=$?
printf 'upstream_tag: v1\n\nsin tabla\n' > "$d/l2.md"
"$SCRIPT" "$d/l2.md" >/dev/null 2>&1; r2=$?
if [ $r1 = 2 ] && [ $r2 = 2 ]; then ok ledger_roto; else bad "ledger_roto ($r1,$r2)"; fi

# varias_filas_un_commit + rama no mergeada + espacios irregulares
d="$(nuevo_repo)"; commit "$d" "port(upstream#7): x"
git -C "$d" checkout -q -b lateral; commit "$d" "port(upstream#8): y"; git -C "$d" checkout -q main
ledger "$d" '|#7|a|aplica|propuesto|m|h|' '| #7 |   b | parcial |   propuesto   | m | h |' '| #8 | c | aplica | propuesto | m | h |'
out="$("$SCRIPT" "$d/ledger.md")"
n="$(grep -cF portado "$d/ledger.md")"
if [ "$n" = 2 ] && [ "$out" = "propuesto-sin-evidencia #8 c" ]; then ok multi_fila_y_rama; else bad "multi_fila_y_rama ($n,$out)"; fi

echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" = 0 ]
