#!/usr/bin/env bash
# Falsabilidad de scripts/check-jq-crlf.sh: rojo contra scripts de prueba con
# `jq | while read` y verde contra el plugin real.
set -uo pipefail
RAIZ="$(git rev-parse --show-toplevel)" || exit 1
if [ -z "$RAIZ" ]; then echo "test-check-jq-crlf: sin toplevel" >&2; exit 1; fi
cd "$RAIZ" || exit 1
CHECK="$RAIZ/scripts/check-jq-crlf.sh"

T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
PASS=0; FAIL=0
ok()  { echo "PASS: $1"; PASS=$((PASS+1)); }
bad() { echo "FAIL: $1"; FAIL=$((FAIL+1)); }

corre() { # $1 = contenido del script; deja rc y out
  local P="$T/p"; rm -rf "$P"; mkdir -p "$P"
  printf '%s\n' "$1" > "$P/x.sh"
  out="$(bash "$CHECK" "$P" 2>&1)"; rc=$?
}

# shellcheck disable=SC2016 # el contenido es texto literal del fixture
{
corre 'echo "$J" | jq -r ".[]" | while read -r l; do echo "$l"; done'
if [ $rc -eq 1 ] && printf '%s' "$out" | grep -q 'x.sh:1:'; then ok tuberia_while_read; else bad "tuberia_while_read (rc=$rc out=$out)"; fi

corre 'while read -r l; do echo "$l"; done < <(jq -r ".[]" <<< "$J")'
if [ $rc -eq 1 ]; then ok process_substitution; else bad "process_substitution (rc=$rc out=$out)"; fi

corre 'echo "$J" | jq -r ".[]" \
  | while read -r l; do echo "$l"; done'
if [ $rc -eq 1 ] && printf '%s' "$out" | grep -q 'x.sh:1:'; then ok continuacion_de_linea; else bad "continuacion_de_linea (rc=$rc out=$out)"; fi

corre 'echo "$J" | jq -j ".[]" | read -r -d "" a'
if [ $rc -eq 0 ]; then ok jq_j_pasa; else bad "jq_j_pasa (rc=$rc out=$out)"; fi

corre 'echo "$J" | jq -r ".[]" | tr -d "\r" | while read -r l; do echo "$l"; done'
if [ $rc -eq 0 ]; then ok tr_d_cr_pasa; else bad "tr_d_cr_pasa (rc=$rc out=$out)"; fi

corre 'v="$(jq -r .a <<< "$J")"; echo "$v"'
if [ $rc -eq 0 ]; then ok jq_sin_read_pasa; else bad "jq_sin_read_pasa (rc=$rc out=$out)"; fi
}

out="$(bash "$CHECK" 2>&1)"; rc=$?
if [ $rc -eq 0 ]; then ok plugin_real_verde; else bad "plugin_real_verde (rc=$rc out=$out)"; fi

echo "$PASS pass, $FAIL fail"
[ "$FAIL" -eq 0 ]
