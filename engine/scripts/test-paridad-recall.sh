#!/usr/bin/env bash
# Autotest de paridad-recall.sh (spec 2026-11-03-exo-hook-codex §7.2, §8.4).
# Fixtures en mktemp -d; el gate se apunta a ellas con EXO_PARIDAD_DIR y
# EXO_HOOK_TESTS_DIR. Solo el test del inventario lee ficheros reales del repo.
set -uo pipefail

AQUI="$(cd "$(dirname "$0")" && pwd)"
GATE="$AQUI/paridad-recall.sh"
REAL="$AQUI/../tests/paridad"
SUITES="$AQUI/../../plugins/exo/scripts"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
FALLOS=0
check() { if [ "$2" = 0 ]; then echo "[PASS] $1"; else echo "[FAIL] $1 — $3"; FALLOS=$((FALLOS+1)); fi; }

N=0
# nuevo <n>: fixture con n casos c1..cn (asignados a reglas), sin divergencias ni tests.
nuevo() {
  N=$((N+1)); D="$TMP/f$N"; mkdir -p "$D/p" "$D/t"
  for i in $(seq 1 "$1"); do
    echo "c$i" >> "$D/p/baseline-recall.txt"
    printf 'reglas\tc%s\n' "$i" >> "$D/p/asignacion-recall.txt"
  done
  : > "$D/p/divergencias-recall-x.txt"
}
div() { printf '%s\t%s\n' "$1" "${2-motivo}" >> "$D/p/divergencias-recall-${3-x}.txt"; }
tag() { printf '/// paridad-recall: %s\n#[test]\nfn t_%s() {}\n' "$1" "$RANDOM" >> "$D/t/hook_x.rs"; }
corre() { EXO_PARIDAD_DIR="$D/p" EXO_HOOK_TESTS_DIR="$D/t" bash "$GATE" 2>&1; echo "rc=$?"; }
cubre_todo() { for i in $(seq 1 "$1"); do tag "c$i"; done; }

# --- inventario real: contenido, no solo conteo --------------------------------
if [ -f "$SUITES/test-exo-recall.sh" ]; then
  grep -ho 'pass "[^"]*"' "$SUITES"/test-exo-recall.sh "$SUITES"/test-exo-recall-golden.sh "$SUITES"/test-exo-recall-superpowers.sh \
    | sed 's/^pass "//;s/"$//' | LC_ALL=C sort -u > "$TMP/regenerado.txt"
  if cmp -s "$TMP/regenerado.txt" "$REAL/baseline-recall.txt"; then check "baseline completo: igual al grep de las 3 suites" 0
  else check "baseline completo: igual al grep de las 3 suites" 1 "$(diff "$TMP/regenerado.txt" "$REAL/baseline-recall.txt" | head -5)"; fi
else
  echo "[SKIP] baseline contra suites: las suites bash ya no existen (inventario congelado)"
fi
check "baseline completo: 61 líneas" "$([ "$(wc -l < "$REAL/baseline-recall.txt")" = 61 ] && echo 0 || echo 1)" "wc=$(wc -l < "$REAL/baseline-recall.txt")"
check "baseline ordenado LC_ALL=C y sin duplicados" \
  "$(LC_ALL=C sort -c -u "$REAL/baseline-recall.txt" 2>/dev/null && echo 0 || echo 1)" "no está en orden C único"

# --- asignación real ------------------------------------------------------------
check "asignación real: cada línea del baseline con área válida" \
  "$(cut -f2- "$REAL/asignacion-recall.txt" | LC_ALL=C sort | cmp -s - "$REAL/baseline-recall.txt" \
     && ! cut -f1 "$REAL/asignacion-recall.txt" | grep -qvxE 'contrato|compact|avisos|reglas|fontaneria|wiring' && echo 0 || echo 1)" "asignación desalineada del baseline"
# toda línea fontaneria real está en las divergencias reales, con motivo
FONT="$(awk -F'\t' '$1=="fontaneria"{print $2}' "$REAL/asignacion-recall.txt" | LC_ALL=C sort)"
DIVF="$(cut -f1 "$REAL"/divergencias-recall-fontaneria.txt | LC_ALL=C sort)"
check "fontaneria real: todas en divergencias-recall-fontaneria.txt" "$([ "$FONT" = "$DIVF" ] && echo 0 || echo 1)" "difieren"
check "fontaneria real: motivo no vacío" "$(awk -F'\t' 'NF<2||$2==""{e=1} END{exit e}' "$REAL"/divergencias-recall-fontaneria.txt && echo 0 || echo 1)" "hay motivo vacío"

# --- OK ---------------------------------------------------------------------------
nuevo 5; div c1; div c2; cubre_todo 0; tag c3; tag c4; tag c5
O="$(corre)"
check "ok: baseline repartido entre divergencias y tests" "$(grep -q '^paridad-recall: OK' <<<"$O" && grep -q 'rc=0' <<<"$O" && echo 0 || echo 1)" "$O"

# --- falta ------------------------------------------------------------------------
nuevo 5; div c1; tag c3; tag c4; tag c5
O="$(corre)"
check "falta: c2 sin test ni divergencia ⇒ ≠0 y nombra c2" "$(grep -q 'falta.*c2' <<<"$O" && ! grep -q 'rc=0' <<<"$O" && echo 0 || echo 1)" "$O"

# --- tag ajeno --------------------------------------------------------------------
nuevo 2; cubre_todo 2; tag zzz
O="$(corre)"
check "ajena: tag de test que no está en el baseline ⇒ ≠0" "$(grep -q 'ajeno al baseline.*zzz' <<<"$O" && ! grep -q 'rc=0' <<<"$O" && echo 0 || echo 1)" "$O"

# --- sobra ------------------------------------------------------------------------
nuevo 2; cubre_todo 2; div zzz
O="$(corre)"
check "sobra: divergencia fuera del baseline ⇒ ≠0 «ajena al baseline»" "$(grep -q 'ajena al baseline.*zzz' <<<"$O" && ! grep -q 'rc=0' <<<"$O" && echo 0 || echo 1)" "$O"

# --- duplicada --------------------------------------------------------------------
nuevo 2; cubre_todo 2; div c1
O="$(corre)"
check "duplicada: c1 en divergencias y en un test ⇒ ≠0" "$(grep -q 'duplicada.*c1' <<<"$O" && ! grep -q 'rc=0' <<<"$O" && echo 0 || echo 1)" "$O"

nuevo 2; tag c2; div c1 m a; div c1 m b
O="$(corre)"
check "duplicada: c1 en dos ficheros de divergencias ⇒ ≠0" "$(grep -q 'duplicada.*c1' <<<"$O" && ! grep -q 'rc=0' <<<"$O" && echo 0 || echo 1)" "$O"

# --- motivo -----------------------------------------------------------------------
nuevo 2; tag c2; printf 'c1\n' >> "$D/p/divergencias-recall-x.txt"
O="$(corre)"
check "motivo: divergencia sin motivo ⇒ ≠0" "$(grep -q 'sin motivo.*c1' <<<"$O" && ! grep -q 'rc=0' <<<"$O" && echo 0 || echo 1)" "$O"

# --- tope 18 ----------------------------------------------------------------------
nuevo 30; for i in $(seq 1 18); do div "c$i"; done; for i in $(seq 19 30); do tag "c$i"; done
O="$(corre)"
check "tope: exactamente 18 divergencias ⇒ OK" "$(grep -q '^paridad-recall: OK' <<<"$O" && grep -q 'rc=0' <<<"$O" && echo 0 || echo 1)" "$O"
nuevo 30; for i in $(seq 1 19); do div "c$i"; done; for i in $(seq 20 30); do tag "c$i"; done
O="$(corre)"
check "tope: 19 divergencias ⇒ ≠0 con «invalidación §8.4»" "$(grep -q 'invalidación §8.4' <<<"$O" && ! grep -q 'rc=0' <<<"$O" && echo 0 || echo 1)" "$O"

# --- asignación ---------------------------------------------------------------------
nuevo 2; cubre_todo 2; sed -i '/^reglas\tc2$/d' "$D/p/asignacion-recall.txt"
O="$(corre)"
check "asignación: línea del baseline sin asignar ⇒ ≠0 y nombra c2" "$(grep -q 'sin asignación.*c2' <<<"$O" && ! grep -q 'rc=0' <<<"$O" && echo 0 || echo 1)" "$O"

nuevo 2; cubre_todo 2; sed -i 's/^reglas\tc2$/marte\tc2/' "$D/p/asignacion-recall.txt"
O="$(corre)"
check "asignación: área fuera del conjunto ⇒ ≠0 y nombra c2" "$(grep -q 'área inválida.*c2' <<<"$O" && ! grep -q 'rc=0' <<<"$O" && echo 0 || echo 1)" "$O"

nuevo 2; cubre_todo 2; printf 'reglas\tzzz\n' >> "$D/p/asignacion-recall.txt"
O="$(corre)"
check "asignación: línea que no está en el baseline ⇒ ≠0" "$(grep -q 'ajena al baseline.*zzz' <<<"$O" && ! grep -q 'rc=0' <<<"$O" && echo 0 || echo 1)" "$O"

nuevo 2; cubre_todo 2; sed -i 's/^reglas\tc2$/fontaneria\tc2/' "$D/p/asignacion-recall.txt"
O="$(corre)"
check "fontaneria: asignada pero sin divergencia ⇒ ≠0 y nombra c2" "$(grep -q 'fontaneria sin divergencia.*c2' <<<"$O" && ! grep -q 'rc=0' <<<"$O" && echo 0 || echo 1)" "$O"

nuevo 2; tag c1; sed -i 's/^reglas\tc2$/wiring\tc2/' "$D/p/asignacion-recall.txt"
O="$(corre)"
check "wiring: área válida, pero sin test ni divergencia ⇒ ≠0 y nombra c2" "$(grep -q 'falta.*c2' <<<"$O" && ! grep -q 'área inválida' <<<"$O" && ! grep -q 'rc=0' <<<"$O" && echo 0 || echo 1)" "$O"
nuevo 2; tag c1; sed -i 's/^reglas\tc2$/wiring\tc2/' "$D/p/asignacion-recall.txt"; div c2 pendiente w
O="$(corre)"
check "wiring: en divergencias ⇒ OK" "$(grep -q '^paridad-recall: OK' <<<"$O" && echo 0 || echo 1)" "$O"

echo
[ "$FALLOS" -eq 0 ] && echo "OK" || echo "FALLAN $FALLOS"
exit "$FALLOS"
