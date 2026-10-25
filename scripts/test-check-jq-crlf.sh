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

corre 'IFS=, read -r a b <<< "$(jq -r ".x" <<< "$J")"'
if [ $rc -eq 1 ] && printf '%s' "$out" | grep -q 'x.sh:1:'; then ok read_aqui_string_sin_limpiar; else bad "read_aqui_string_sin_limpiar (rc=$rc out=$out)"; fi

corre 'IFS=, read -r a b <<< "$(jq -r ".x" <<< "$J")"
b="${b%$'"'"'\r'"'"'}"'
if [ $rc -eq 0 ]; then ok read_aqui_string_limpio_pasa; else bad "read_aqui_string_limpio_pasa (rc=$rc out=$out)"; fi

corre 'jq -r .a <<< "$J" | sort -u'
if [ $rc -eq 0 ]; then ok jq_sin_read_ni_asignacion_pasa; else bad "jq_sin_read_ni_asignacion_pasa (rc=$rc out=$out)"; fi
# Asignación VAR="$(… jq …)": la sustitución quita el \n pero no el \r.
corre 'v="$(printf %s "$J" | jq -r .a)"; echo "$v"'
if [ $rc -eq 1 ] && printf '%s' "$out" | grep -q 'x.sh:1:'; then ok asignacion_comillas_rojo; else bad "asignacion_comillas_rojo (rc=$rc out=$out)"; fi

corre 'v=$(printf %s "$J" | jq -r .a)'
if [ $rc -eq 1 ]; then ok asignacion_sin_comillas_rojo; else bad "asignacion_sin_comillas_rojo (rc=$rc out=$out)"; fi

corre 'v="${v:-$(printf %s "$J" | jq -r .a)}"'
if [ $rc -eq 1 ]; then ok asignacion_default_expansion_rojo; else bad "asignacion_default_expansion_rojo (rc=$rc out=$out)"; fi

corre 'v="$(printf %s "$J" \
  | jq -r .a)"'
if [ $rc -eq 1 ] && printf '%s' "$out" | grep -q 'x.sh:1:'; then ok asignacion_continuacion_rojo; else bad "asignacion_continuacion_rojo (rc=$rc out=$out)"; fi

corre 'n="$(printf %s "$J" | jq -nc ".a|length")"'
if [ $rc -eq 1 ]; then ok asignacion_jq_nc_no_objeto_rojo; else bad "asignacion_jq_nc_no_objeto_rojo (rc=$rc out=$out)"; fi

corre 'v="$(printf %s "$J" | jq -j .a)"'
if [ $rc -eq 0 ]; then ok asignacion_jq_j_verde; else bad "asignacion_jq_j_verde (rc=$rc out=$out)"; fi

corre 'v="$(printf %s "$J" | jq -r .a | tr -d "\r")"'
if [ $rc -eq 0 ]; then ok asignacion_tr_verde; else bad "asignacion_tr_verde (rc=$rc out=$out)"; fi

corre 'v="$(printf %s "$J" \
  | jq -r .a | tr -d "\r")"'
if [ $rc -eq 0 ]; then ok asignacion_continuacion_tr_verde; else bad "asignacion_continuacion_tr_verde (rc=$rc out=$out)"; fi

corre 'v="$(printf %s "$J" | jq -r .a)"
v="${v%$'"'"'\r'"'"'}"'
if [ $rc -eq 0 ]; then ok asignacion_limpieza_explicita_verde; else bad "asignacion_limpieza_explicita_verde (rc=$rc out=$out)"; fi

corre 'v="$(printf %s "$J" | jq -r .a)"
w="${w%$'"'"'\r'"'"'}"'
if [ $rc -eq 1 ]; then ok limpieza_de_otra_variable_rojo; else bad "limpieza_de_otra_variable_rojo (rc=$rc out=$out)"; fi

corre 'j="$(jq -nc --arg a "$A" '"'"'{x:$a}'"'"')"'
if [ $rc -eq 0 ]; then ok asignacion_objeto_json_verde; else bad "asignacion_objeto_json_verde (rc=$rc out=$out)"; fi

corre 'j="$(printf %s "$T" | jq -Rs '"'"'{x:.}'"'"')"'
if [ $rc -eq 0 ]; then ok asignacion_jq_Rs_objeto_verde; else bad "asignacion_jq_Rs_objeto_verde (rc=$rc out=$out)"; fi

# Los test-*.sh son fixtures, no hooks: la regla de asignación no los mira.
P="$T/p"; rm -rf "$P"; mkdir -p "$P"
printf '%s\n' 'v="$(printf %s "$J" | jq -r .a)"' > "$P/test-x.sh"
out="$(bash "$CHECK" "$P" 2>&1)"; rc=$?
if [ $rc -eq 0 ]; then ok test_fixture_exento_de_asignacion; else bad "test_fixture_exento_de_asignacion (rc=$rc out=$out)"; fi
printf '%s\n' 'echo "$J" | jq -r .a | while read -r l; do :; done' > "$P/test-x.sh"
out="$(bash "$CHECK" "$P" 2>&1)"; rc=$?
if [ $rc -eq 1 ]; then ok test_fixture_sigue_vigilado_en_read; else bad "test_fixture_sigue_vigilado_en_read (rc=$rc out=$out)"; fi
}

out="$(bash "$CHECK" 2>&1)"; rc=$?
if [ $rc -eq 0 ]; then ok plugin_real_verde; else bad "plugin_real_verde (rc=$rc out=$out)"; fi

echo "$PASS pass, $FAIL fail"
[ "$FAIL" -eq 0 ]
