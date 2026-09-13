#!/usr/bin/env bash
# Aplica los criterios del pre-registro
# (docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md). Es mecánico
# a propósito: el veredicto no se redacta a mano.
#   compara.sh <base>             predicciones P1–P4 y puertas de las Tasks 10 y 12
#   compara.sh <base> <despues>   tabla antes/después y criterios C-*
set -uo pipefail
cd "$(dirname "$0")/../../.." || exit 1
R="evals/recall-coste/results"
BASE="${1:-}"
DESP="${2:-}"
[ -n "$BASE" ] || { echo "uso: compara.sh <base> [despues]" >&2; exit 1; }
for e in "$BASE" ${DESP:+"$DESP"}; do
  [ -f "$R/$e/resumen.tsv" ] || { echo "compara: falta $R/$e/resumen.tsv" >&2; exit 1; }
done

# campo <etiqueta> <id> <col>: 2=p50 3=p95 5=fallidas 6=corridas 7=rc_directa; NA si no está
campo() { awk -F'\t' -v id="$2" -v c="$3" '$1 == id { print $c; f = 1 } END { if (!f) print "NA" }' "$R/$1/resumen.tsv"; }
# cumple "<expr awk>": 0 si la expresión es verdadera (bash no hace flotantes)
cumple() { case "$1" in *NA*) return 1 ;; esac; awk "BEGIN { exit !($1) }"; }
knn_tope() { grep -q 'k value in knn query too large' "$R/$1/$2.stderr" 2>/dev/null; }
linea() { printf '%s\t%s\n' "$1" "$2"; }

if [ -z "$DESP" ]; then
  p1=PASA
  for s in s1-query-refresh s1b-query-refresh-sim0 s2-query; do
    for n in 1000 5000; do
      { [ "$(campo "$BASE" "$s-n$n" 7)" = 1 ] && knn_tope "$BASE" "$s-n$n"; } || p1=FALLA
    done
    [ "$(campo "$BASE" "$s-n174" 7)" = 0 ] || p1=FALLA
  done
  linea "P1 (H27: tope KNN a N>=1000, ok a 174)" "$p1"
  s2=$(campo "$BASE" s2-query-n174 2); s1=$(campo "$BASE" s1-query-refresh-n174 2); s7=$(campo "$BASE" s7-config-jq-n174 2)
  cumple "$s2 >= 800 && $s2 <= 1300" && linea "P2 (s2 n174 p50=$s2)" CUMPLIDA || linea "P2 (s2 n174 p50=$s2)" FALLIDA
  cumple "$s1 - $s2 < 60" && linea "P3 (refresh n174 = $s1-$s2)" CUMPLIDA || linea "P3 (refresh n174 = $s1-$s2)" FALLIDA
  cumple "$s7 < 20" && linea "P4 (s7 n174 p50=$s7)" CUMPLIDA || linea "P4 (s7 n174 p50=$s7)" FALLIDA
  s4=$(campo "$BASE" s4-arranque-content-n5000 3)
  cumple "$s4 > 250" && linea "PUERTA C-H17a (s4 n5000 p95=$s4)" "ABIERTA: Task 10 procede" || linea "PUERTA C-H17a (s4 n5000 p95=$s4)" "CERRADA: Task 10 no se ejecuta"
  cumple "$s7 > 30" && linea "PUERTA C-H10 Linux (s7 n174 p50=$s7)" "ABIERTA" || linea "PUERTA C-H10 Linux (s7 n174 p50=$s7)" "CERRADA (falta W11, D5)"
  exit 0
fi

printf '| id | p50 antes | p50 después | p95 antes | p95 después | fallidas antes | fallidas después |\n|---|---|---|---|---|---|---|\n'
awk -F'\t' 'FNR == 1 { next }
  NR == FNR { a50[$1] = $2; a95[$1] = $3; af[$1] = $5; next }
  { x50 = ($1 in a50) ? a50[$1] : "NA"; x95 = ($1 in a95) ? a95[$1] : "NA"; xf = ($1 in af) ? af[$1] : "NA"
    printf "| %s | %s | %s | %s | %s | %s | %s |\n", $1, x50, $2, x95, $3, xf, $5 }' \
  "$R/$BASE/resumen.tsv" "$R/$DESP/resumen.tsv"
echo

c=PASA
for s in s1-query-refresh s1b-query-refresh-sim0 s2-query; do
  for n in 1000 5000; do
    [ "$(campo "$DESP" "$s-n$n" 5)" = 0 ] && [ "$(campo "$DESP" "$s-n$n" 7)" = 0 ] || c=FALLA
  done
done
linea C-H27 "$c"

a=$(campo "$BASE" s3-index-sin-cambios-n5000 2); b=$(campo "$DESP" s3-index-sin-cambios-n5000 2)
if cumple "$a <= 50"; then linea "C-H4 (s3 n5000 $a→$b)" VACÍO
elif cumple "$b <= 0.6 * $a"; then linea "C-H4 (s3 n5000 $a→$b)" PASA
else linea "C-H4 (s3 n5000 $a→$b)" FALLA; fi

a=$(campo "$BASE" s4-arranque-content-n5000 3); b=$(campo "$DESP" s4-arranque-content-n5000 3)
if ! cumple "$a > 250"; then linea "C-H17a (s4 n5000 p95 $a→$b)" "NO APLICA (puerta cerrada)"
elif cumple "$b <= 0.5 * $a"; then linea "C-H17a (s4 n5000 p95 $a→$b)" PASA
else linea "C-H17a (s4 n5000 p95 $a→$b)" FALLA; fi

g=$(campo "$DESP" s2-query-n5000 2); p=$(campo "$DESP" s2-query-n174 2)
cumple "$g - $p > 250" && linea "C-H17b (s2 n5000-n174 = $g-$p)" "DERIVAR a C/backlog" || linea "C-H17b (s2 n5000-n174 = $g-$p)" "NO DERIVAR"

k=$(campo "$DESP" s8-kb-precommit-n5000 2)
cumple "$k > 2000" && linea "C-H23 (s8 n5000 p50=$k)" "ABRIR ITEM backlog" || linea "C-H23 (s8 n5000 p50=$k)" "NO ABRIR"

c=PASA
for n in 174 1000 5000; do
  for s in s5-search-fts s3-index-sin-cambios; do
    a=$(campo "$BASE" "$s-n$n" 2); b=$(campo "$DESP" "$s-n$n" 2)
    cumple "$b <= 1.2 * $a + 2" || { c=FALLA; linea "  regresión $s-n$n" "$a→$b"; }
  done
done
linea C-noregresión "$c"
