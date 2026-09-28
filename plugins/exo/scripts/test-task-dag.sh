#!/usr/bin/env bash
# Test standalone para task-dag (olas en JSON desde un plan). Fixtures en testdata/task-dag.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DAG="${SCRIPT_DIR}/../skills/orchestrate/scripts/task-dag"
FX="${SCRIPT_DIR}/testdata/task-dag"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

check_olas() { # nombre fixture esperado
  local got
  got=$("$DAG" "$FX/$2.md" 2>/dev/null | jq -c '.olas' 2>/dev/null)
  if [ "$got" = "$3" ]; then pass "$1"; else fail "$1" "esperaba $3, obtuve '$got'"; fi
}

check_olas independientes independientes '[[1,2]]'
check_olas consume consume '[[1],[2]]'
check_olas transitivo transitivo '[[1],[2],[3]]'
check_olas fichero_compartido fichero_compartido '[[1],[2]]'
check_olas mixto mixto '[[1,2],[3]]'
# T2 y T3 comparten fichero (con espacios, formato "## Task N —"); T4 consume T1 pero comparte con T3
check_olas paths_con_espacios_y_formato_antiguo espacios_antiguo '[[1,2],[3],[4]]'
check_olas fences fences '[[1,3]]'

# sin_formato: secuencial + aviso
out=$("$DAG" "$FX/sin_formato.md" 2>/dev/null)
if [ "$(jq -c .olas <<<"$out")" = "[[1],[2],[3]]" ] && jq -e '.avisos | any(startswith("DAG: secuencial ("))' <<<"$out" >/dev/null; then
  pass sin_formato
else fail sin_formato "salida: $out"; fi

# consume_inexistente
out=$("$DAG" "$FX/consume_inexistente.md" 2>/dev/null)
if [ "$(jq -c .olas <<<"$out")" = "[[1],[2],[3]]" ] && jq -e '.avisos | any(startswith("DAG: secuencial ("))' <<<"$out" >/dev/null; then
  pass consume_inexistente
else fail consume_inexistente "salida: $out"; fi

check_seq() { # nombre fixture fragmento_aviso
  local out
  out=$("$DAG" "$FX/$2.md" 2>/dev/null)
  if jq -e --arg f "$3" '.avisos | any(startswith("DAG: secuencial (") and contains($f))' <<<"$out" >/dev/null 2>&1 \
     && [ "$(jq -c '.olas | map(length) | all(. == 1)' <<<"$out")" = true ]; then
    pass "$1"
  else fail "$1" "salida: $out"; fi
}
check_seq files_sin_backticks files_sin_backticks "Files sin paths legibles"
check_olas multi_path_bullet multi_path_bullet '[[1],[2]]'
check_seq ids_duplicados ids_duplicados "duplicad"
check_olas ruta_punto_barra ruta_punto_barra '[[1],[2]]'

# json_valido en todas las fixtures + plan sin tareas
ok=1
printf '# nada\n' > "$TMP/vacio.md"
for f in "$FX"/*.md "$TMP/vacio.md"; do
  "$DAG" "$f" 2>/dev/null | jq -e . >/dev/null 2>&1 || { ok=0; fail json_valido "$f"; }
done
[ "$ok" = 1 ] && pass json_valido

"$DAG" >/dev/null 2>&1; rc=$?
if [ "$rc" = 2 ]; then pass uso_sin_argumento; else fail uso_sin_argumento "rc=$rc"; fi
"$DAG" "$FX/no-existe.md" >/dev/null 2>&1; rc=$?
if [ "$rc" = 2 ]; then pass fichero_inexistente; else fail fichero_inexistente "rc=$rc"; fi

printf '\n%s pass, %s fail\n' "$PASS" "$FAIL"
[ "$FAIL" = 0 ]
