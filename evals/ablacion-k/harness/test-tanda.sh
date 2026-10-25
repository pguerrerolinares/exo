#!/usr/bin/env bash
# Test del tope de coste de tanda.sh (#185): un symlink corridas -> corridas-principal no duplica el gasto.
# K_REANUDAR=1 con corridas ya completas: no se lanza claude. Todo bajo mktemp -d.
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
fail=0
ok(){ echo "ok   $1"; }; ko(){ echo "FAIL $1"; fail=1; }

export K_ROOT="$T/k"; mkdir -p "$K_ROOT/corridas-principal/t1/a0-r1" "$K_ROOT/corridas-principal/t1/a0-r2"
for r in 1 2; do
  d="$K_ROOT/corridas-principal/t1/a0-r$r"
  echo '{"fin":"completed","tokens_in":1000,"usd":30}' > "$d/meta.json"; echo '{"fuga":false}' > "$d/fugas.json"
done
ln -s corridas-principal "$K_ROOT/corridas"
printf 't1 a0 1\nt1 a0 2\n' > "$K_ROOT/orden.txt"; echo x > "$K_ROOT/congelacion.txt"
export K_ORDEN="$K_ROOT/orden.txt" K_REANUDAR=1

# gasto real 60 USD: con tope 100 hay que SEGUIR (contando el symlink dos veces serían 120 y pararía)
out=$(K_TOPE_USD=100 bash "$HERE/tanda.sh" 1 2>&1); rc=$?
if [ $rc = 0 ] && grep -q SEGUIR <<<"$out"; then ok symlink_no_duplica_gasto; else ko "symlink_no_duplica_gasto rc=$rc: $out"; fi
# y un tope por debajo del gasto real sigue parando
out=$(K_TOPE_USD=50 bash "$HERE/tanda.sh" 1 2>&1); rc=$?
if [ $rc = 1 ] && grep -q "coste: 60.00 USD" <<<"$out"; then ok tope_real_sigue_parando; else ko "tope_real_sigue_parando rc=$rc: $out"; fi
exit $fail
