#!/usr/bin/env bash
# Tests bash (sin claude) de correr-techo.sh: ensayo real de correr.sh y breakers con un correr.sh falso.
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"; CT="$HERE/correr-techo.sh"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
fail=0; ok(){ echo "ok   $1"; }; ko(){ echo "FAIL $1"; fail=1; }
ids=$(cut -f1 "$HERE/tareas.tsv")
mk() {  # $1 = K_ROOT nuevo
  mkdir -p "$1/prep/stub" "$1/fuentes"; : > "$1/prep/manifiesto.txt"; : > "$1/reconstruccion.tsv"
  for i in $ids; do
    mkdir -p "$1/gold/s1/$i" "$1/fuentes/$i"; echo '{"prompt":"x"}' > "$1/gold/s1/$i/tarea.json"
    git -C "$1/fuentes/$i" init -q; printf '%s\tsi\tok\n' "$i" >> "$1/reconstruccion.tsv"
  done
}
export TECHO_TARBALL="$T/reg.tar.gz"

# ensayo: 28 corridas, cada una con K_REGLA_FILE = su regla (regla-ctx.json la lleva), sin claude; 'no' se omite
mk "$T/k1"; sed -i 's/^g0-122\tsi/g0-122\tno/' "$T/k1/reconstruccion.tsv"
out=$(K_ROOT="$T/k1" K_ENSAYO=1 bash "$CT" 4 2>&1); rc=$?
n=$(ls -d "$T"/k1/corridas/*/ar-r* 2>/dev/null | wc -l)
bien=1; for d in "$T"/k1/corridas/*/ar-r*; do id=$(basename "$(dirname "$d")")
  [ "$(jq -j .hookSpecificOutput.additionalContext "$d/regla-ctx.json"; echo x)" = "$(cat "$HERE/reglas/$id.txt"; echo x)" ] || bien=0; done
[ $rc = 0 ] && [ "$n" = 26 ] && [ $bien = 1 ] && [[ $out == *"omitida (no reconstruible): g0-122 r1"* ]] && ok ensayo_26_corridas_y_reglas_por_tarea || ko "ensayo rc=$rc n=$n bien=$bien"

# correr falso: escribe meta/fugas; g2-97 r1 fuga
cat > "$T/falso.sh" <<'F'
#!/usr/bin/env bash
id=$(basename "$1"); O="$K_ROOT/corridas/$id/$2-r$3"; mkdir -p "$O"; echo "$id $3" >> "$K_ROOT/lanzadas.log"
if [ -n "${FALSO_SIN:-}" ]; then echo '{"error":"sin result"}' > "$O/meta.json"; else echo "{\"fin\":\"completed\",\"usd\":${FALSO_USD:-0.1}}" > "$O/meta.json"; fi
if [ "$id $3" = "${FALSO_FUGA:-}" ]; then echo '{"fuga":true}' > "$O/fugas.json"; else echo '{"fuga":false}' > "$O/fugas.json"; fi
F
chmod +x "$T/falso.sh"; export TECHO_CORRER="$T/falso.sh"

mk "$T/k2"; out=$(K_ROOT="$T/k2" bash "$CT" 1 2>&1); rc=$?
[ $rc = 0 ] && [ "$(wc -l < "$T/k2/lanzadas.log")" = 28 ] && [ -s "$TECHO_TARBALL" ] && ok completa_28_sin_breakers || ko "completa rc=$rc"

mk "$T/k3"; out=$(K_ROOT="$T/k3" FALSO_FUGA="g2-97 2" bash "$CT" 1 2>&1); rc=$?
[ $rc = 1 ] && [ "$(wc -l < "$T/k3/lanzadas.log")" = 8 ] && [[ $out == *"PARAR: fuga: g2-97/ar-r2"* ]] && ok fuga_para_y_no_lanza_mas || ko "fuga rc=$rc n=$(wc -l < "$T/k3/lanzadas.log")"

mk "$T/k4"; out=$(K_ROOT="$T/k4" FALSO_USD=1 bash "$CT" 1 2>&1); rc=$?
[ $rc = 1 ] && [ "$(wc -l < "$T/k4/lanzadas.log")" = 16 ] && [[ $out == *"gasto:"* ]] && ok tope_de_gasto_para || ko "gasto rc=$rc n=$(wc -l < "$T/k4/lanzadas.log")"

# cero re-intentos: sin K_REANUDAR aborta; con K_REANUDAR solo lanza las que faltan
out=$(K_ROOT="$T/k3" bash "$CT" 1 2>&1); rc=$?
[ $rc = 2 ] && [ "$(wc -l < "$T/k3/lanzadas.log")" = 8 ] && ok sin_reanudar_aborta || ko "reanudar-abort rc=$rc"
K_ROOT="$T/k3" K_REANUDAR=1 bash "$CT" 1 >/dev/null 2>&1; rc=$?
[ $rc = 0 ] && [ "$(wc -l < "$T/k3/lanzadas.log")" = 28 ] && [ "$(sort "$T/k3/lanzadas.log" | uniq -d | wc -l)" = 0 ] && ok reanudar_no_repite || ko "reanudar rc=$rc"

mk "$T/k5"; out=$(K_ROOT="$T/k5" FALSO_SIN=1 bash "$CT" 1 2>&1); rc=$?
[ $rc = 1 ] && [[ $out == *"infra: 28/28 sin result"* ]] && ok infra_sin_result_para || ko "infra rc=$rc"
exit $fail
