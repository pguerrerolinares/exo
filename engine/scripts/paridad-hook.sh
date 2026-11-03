#!/usr/bin/env bash
# Oráculo de paridad (spec 2026-11-02-exo-hook-multi-harness §6.3, §7.2):
# corre test-bash-guards.sh y test-search-first.sh con HOOK apuntando al
# wrapper de `exo hook claude` y compara sus líneas [PASS] normalizadas con
# tests/paridad/baseline.txt menos tests/paridad/divergencias.txt.
# Sale 0 e imprime "paridad: OK" si y solo si los conjuntos coinciden.
set -uo pipefail

AQUI="$(cd "$(dirname "$0")" && pwd)"
RAIZ="$(cd "$AQUI/../.." && pwd)"
PARIDAD="$AQUI/../tests/paridad"
SUITES="$RAIZ/plugins/exo/scripts"
export EXO_BIN="${EXO_BIN:-$AQUI/../target/debug/exo}"

if [ ! -x "$EXO_BIN" ]; then
  echo "paridad: FALLA — no existe el binario $EXO_BIN (cargo build --manifest-path engine/Cargo.toml)" >&2
  exit 1
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
export EXO_PARIDAD_BASE="$TMP/estado"
mkdir -p "$EXO_PARIDAD_BASE"
export HOOK="$AQUI/hook-wrapper-claude.sh"

norm() { sed -E 's/\([0-9]+ bytes\)/(N bytes)/'; }

for suite in test-bash-guards.sh test-search-first.sh; do
  bash "$SUITES/$suite" 2>/dev/null | grep '^\[PASS\]' | norm >> "$TMP/obtenido.raw" || true
done
LC_ALL=C sort "$TMP/obtenido.raw" > "$TMP/obtenido"
LC_ALL=C sort "$PARIDAD/baseline.txt" > "$TMP/baseline"
cut -f1 "$PARIDAD/divergencias.txt" | LC_ALL=C sort > "$TMP/divergencias"

# Una divergencia que no está en el baseline es un typo: no puede ocultar nada.
FALLO=0
while IFS= read -r l; do
  printf 'paridad: divergencia ajena al baseline: %s\n' "$l"; FALLO=1
done < <(LC_ALL=C comm -13 "$TMP/baseline" "$TMP/divergencias")

LC_ALL=C comm -23 "$TMP/baseline" "$TMP/divergencias" > "$TMP/esperado"
while IFS= read -r l; do
  printf 'paridad: falta (y no es divergencia): %s\n' "$l"; FALLO=1
done < <(LC_ALL=C comm -23 "$TMP/esperado" "$TMP/obtenido")
while IFS= read -r l; do
  printf 'paridad: sobra (pasa y está declarada divergente, o es un caso nuevo): %s\n' "$l"; FALLO=1
done < <(LC_ALL=C comm -13 "$TMP/esperado" "$TMP/obtenido")

if [ "$FALLO" -eq 0 ]; then
  echo "paridad: OK"
else
  echo "paridad: FALLA"
fi
exit "$FALLO"
