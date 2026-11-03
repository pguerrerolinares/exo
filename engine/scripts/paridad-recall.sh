#!/usr/bin/env bash
# Gate de paridad del recall (spec 2026-11-03-exo-hook-codex §7.2, §8.4):
# cada `pass "…"` de las suites bash de recall (tests/paridad/baseline-recall.txt)
# debe estar o en un divergencias-recall-*.txt (con motivo) o en un
# `/// paridad-recall: <texto>` de engine/tests/hook_*.rs, nunca en los dos.
# Áreas: contrato|compact|avisos|reglas|fontaneria|wiring (wiring: engine-stale vía --min, en hook_cli).
# Valida también la asignación por área y el tope de 18 divergencias.
# Sale 0 e imprime "paridad-recall: OK" si y solo si todo cuadra.
set -uo pipefail

AQUI="$(cd "$(dirname "$0")" && pwd)"
PARIDAD="${EXO_PARIDAD_DIR:-$AQUI/../tests/paridad}"
TESTS="${EXO_HOOK_TESTS_DIR:-$AQUI/../tests}"
TOPE=18

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
FALLO=0
falla() { printf 'paridad-recall: %s\n' "$1"; FALLO=1; }
C() { LC_ALL=C "$@"; }

C sort "$PARIDAD/baseline-recall.txt" > "$TMP/baseline"

# Asignación: una línea <área>\t<texto> por cada línea del baseline.
cut -f2- "$PARIDAD/asignacion-recall.txt" | C sort > "$TMP/asignados"
while IFS= read -r l; do falla "ajena al baseline (asignación): $l"; done < <(C comm -13 "$TMP/baseline" "$TMP/asignados")
while IFS= read -r l; do falla "sin asignación: $l"; done < <(C comm -23 "$TMP/baseline" "$TMP/asignados")
while IFS=$'\t' read -r area texto; do
  case "$area" in contrato|compact|avisos|reglas|fontaneria|wiring) ;; *) falla "área inválida '$area': $texto" ;; esac
done < "$PARIDAD/asignacion-recall.txt"

# Divergencias: <texto>\t<motivo> en todos los ficheros divergencias-recall-*.txt.
: > "$TMP/div-raw"
for f in "$PARIDAD"/divergencias-recall-*.txt; do
  [ -f "$f" ] || continue
  while IFS=$'\t' read -r texto motivo; do
    [ -n "$texto" ] || continue
    [ -n "$motivo" ] || falla "divergencia sin motivo: $texto"
    printf '%s\n' "$texto" >> "$TMP/div-raw"
  done < "$f"
done
C sort "$TMP/div-raw" > "$TMP/div-sorted"
C sort -u "$TMP/div-raw" > "$TMP/div"
while IFS= read -r l; do falla "divergencia duplicada: $l"; done < <(C comm -23 "$TMP/div-sorted" "$TMP/div")
while IFS= read -r l; do falla "divergencia ajena al baseline: $l"; done < <(C comm -13 "$TMP/baseline" "$TMP/div")
N_DIV="$(wc -l < "$TMP/div" | tr -d ' ')"
if [ "$N_DIV" -gt "$TOPE" ]; then
  falla "$N_DIV divergencias (tope $TOPE): invalidación §8.4 — el port no conserva el comportamiento; replantear antes de seguir"
fi

# Lo asignado a fontaneria no lo porta nadie: debe constar como divergencia.
while IFS= read -r l; do
  grep -qxF -- "$l" "$TMP/div" || falla "fontaneria sin divergencia: $l"
done < <(awk -F'\t' '$1=="fontaneria"{print $2}' "$PARIDAD/asignacion-recall.txt")

# Tests Rust: `/// paridad-recall: <texto>`.
: > "$TMP/tags-raw"
for f in "$TESTS"/hook_*.rs; do
  [ -f "$f" ] || continue
  sed -n 's|^[[:space:]]*///[[:space:]]*paridad-recall:[[:space:]]*||p' "$f" | sed 's/[[:space:]]*$//' >> "$TMP/tags-raw"
done
C sort -u "$TMP/tags-raw" > "$TMP/tags"
while IFS= read -r l; do falla "test ajeno al baseline: $l"; done < <(C comm -13 "$TMP/baseline" "$TMP/tags")
while IFS= read -r l; do falla "duplicada (divergencia y test): $l"; done < <(C comm -12 "$TMP/div" "$TMP/tags")

C sort -u "$TMP/div" "$TMP/tags" > "$TMP/cubierto"
while IFS= read -r l; do falla "falta (ni test ni divergencia): $l"; done < <(C comm -23 "$TMP/baseline" "$TMP/cubierto")

if [ "$FALLO" -eq 0 ]; then echo "paridad-recall: OK"; else echo "paridad-recall: FALLA"; fi
exit "$FALLO"
