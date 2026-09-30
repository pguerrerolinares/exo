#!/usr/bin/env bash
# reconstruir.sh — rehace el entorno de la campaña K en $K_ROOT para el test de techo:
#   prep/ (preparar.sh), gold/ (del tarball de registro), fuentes/<id> (clon local en el commit de tarea.json).
# Escribe $K_ROOT/reconstruccion.tsv (id  reconstruible  motivo) y nunca escribe en los repos fuente.
set -uo pipefail
H="$(cd "$(dirname "$0")" && pwd)"
HARNESS="$H/../ablacion-k/harness"
CONGEL="$H/../ablacion-k/congelacion.txt"
K_ROOT="${K_ROOT:-$HOME/.cache/exo-ablacion-k}"; export K_ROOT
TARBALL="${K_TARBALL:-$HOME/.cache/exo-ablacion-k-registro.tar.gz}"
GOLD_SHA=c2d8835b74fdcd8e3ee6f9c3ff32efc3248caca5bc3aa5618d7ff9e3908c244c
KB_COMMIT=80cba57878d37583be1f2654ecb7d96c51dad8ab
CLAUDE_MD_SHA=6f3c6f382f90e794
OUT="$K_ROOT/reconstruccion.tsv"
# reconstruir_fuente <id> <tarea.json> <sucio_en_K>: clona el repo de la tarea en su commit en
# $K_ROOT/fuentes/<id>, imprime la fila de la tabla y añade la de $OUT. Sin tocar el repo fuente.
reconstruir_fuente() {
  local id=$1 tj=$2 sucio=${3:-0} repo commit F motivo head co rec mot
  repo=$(jq -r .repo "$tj"); commit=$(jq -r .commit "$tj")
  F="$K_ROOT/fuentes/$id"; rm -rf "$F"; mkdir -p "$K_ROOT/fuentes"
  motivo=""
  if [ ! -d "$repo/.git" ]; then motivo=repo-inexistente
  elif ! git -C "$repo" cat-file -e "$commit^{commit}" 2>/dev/null; then motivo=commit-inexistente
  elif ! git clone -q --no-checkout "$repo" "$F" 2>/dev/null; then motivo=clon-fallo
  elif ! git -C "$F" checkout -q --detach "$commit" 2>/dev/null; then motivo=checkout-fallo
  fi
  if [ -n "$motivo" ]; then
    rm -rf "$F"
    printf '%s\trepo\t-\tno\t%s\n' "$id" "$sucio"; printf '%s\tno\t%s\n' "$id" "$motivo" >> "$OUT"; return 0
  fi
  git -C "$F" remote remove origin
  jq -r '(.excluir // [])[]' "$tj" | while read -r e; do rm -rf "${F:?}/$e"; done
  head=$(git -C "$F" rev-parse HEAD)
  if [ "$head" = "$commit" ]; then co=coincide; rec=si; mot="sucio_en_K=$sucio no reconstruible"; else co=DIFIERE; rec=no; mot=head-distinto; fi
  printf '%s\trepo\t%s\t%s\t%s\n' "$id" "${head:0:12}" "$co" "$sucio"
  printf '%s\t%s\t%s\n' "$id" "$rec" "$mot" >> "$OUT"
}

# rederivar_claude_md <CLAUDE.md>: re-deriva SIEMPRE prep/claude-md.md del CLAUDE.md dado (barato), para que un
# prep reutilizado no dé un coincide falso. Si falta o difiere, lo escribe y actualiza el manifiesto.
# Fija prep_nota (se le añade la nota) y prep_res (coincide/DIFIERE frente a CLAUDE_MD_SHA).
rederivar_claude_md() {
  local src=$1 P="$K_ROOT/prep" nuevo sha
  nuevo=$(mktemp); "$HARNESS/filtro-claude-md.sh" < "$src" > "$nuevo"
  if [ ! -f "$P/claude-md.md" ]; then prep_nota="${prep_nota}prep/claude-md.md faltaba, derivado; "
  elif ! cmp -s "$nuevo" "$P/claude-md.md"; then prep_nota="${prep_nota}prep/claude-md.md estaba obsoleto, re-derivado; "
  fi
  if ! cmp -s "$nuevo" "$P/claude-md.md" 2>/dev/null; then
    cp "$nuevo" "$P/claude-md.md"
    if [ -f "$P/manifiesto.txt" ]; then sed -i "s/^claude-md.md .*/claude-md.md $(sha256sum < "$nuevo" | cut -c1-16)/" "$P/manifiesto.txt"; fi
  fi
  rm -f "$nuevo"
  sha=$(sha256sum < "$P/claude-md.md" | cut -c1-16)
  if [ "$sha" = "$CLAUDE_MD_SHA" ]; then prep_res="claude-md sha=$sha coincide"
  else prep_res="claude-md DIFIERE sha=$sha esperado=$CLAUDE_MD_SHA (~/.claude/CLAUDE.md cambio desde K)"; fi
}

# Con RECONSTRUIR_LIB=1 el script solo define funciones (para los tests).
[ -n "${RECONSTRUIR_LIB:-}" ] && return 0

[ -f "$TARBALL" ] || { echo "falta $TARBALL" >&2; exit 2; }

mkdir -p "$K_ROOT"; TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
tar -xzf "$TARBALL" -C "$TMP" gold-activo.tar || { echo "tarball sin gold-activo.tar" >&2; exit 2; }
got=$(sha256sum < "$TMP/gold-activo.tar" | cut -d' ' -f1)
[ "$got" = "$GOLD_SHA" ] || { echo "gold-activo.tar sha $got != $GOLD_SHA" >&2; exit 2; }
rm -rf "$K_ROOT/gold"; mkdir -p "$K_ROOT/gold"
tar -xf "$TMP/gold-activo.tar" -C "$K_ROOT/gold"
cp "$TMP/gold-activo.tar" "$K_ROOT/gold-activo.tar"

mkdir -p "$K_ROOT"; : > "$OUT"

# prep: si el índice de exo falla se reporta y se sigue (ar usa el stub).
prep_nota=""; prep_ok=si
# preparar.sh tarda ~30 min (exo rebuild): se reutiliza un prep ya hecho en el mismo commit de KB (K_REHACER_PREP=1 lo fuerza).
if [ -z "${K_REHACER_PREP:-}" ] && grep -qx "kb_commit $KB_COMMIT" "$K_ROOT/prep/manifiesto.txt" 2>/dev/null; then :
elif ! bash "$HARNESS/preparar.sh" "$KB_COMMIT" > "$K_ROOT/preparar.log" 2>&1; then
  prep_nota="preparar.sh fallo (ver preparar.log); "
fi
[ -f "$K_ROOT/prep/manifiesto.txt" ] || { prep_nota="${prep_nota}sin manifiesto.txt (correr.sh fallara); "; prep_ok=no; }
mkdir -p "$K_ROOT/prep"
rederivar_claude_md "$HOME/.claude/CLAUDE.md"
printf '_prep\t%s\t%s%s\n' "$prep_ok" "$prep_nota" "$prep_res" >> "$OUT"
echo "PREP: ${prep_nota}${prep_res}"

printf 'id\tfuente\tHEAD\tcoincide\tsucio_en_K\n'
while read -r id; do
  tj="$K_ROOT/gold/s1/$id/tarea.json"
  sucio=$(awk -v i="$id" '$1==i {sub(/.*sucio=/,"",$0); print $1}' "$CONGEL" | head -1)
  if [ "$(jq -r '.setup // false' "$tj")" = true ]; then
    printf '%s\tsetup\t-\t-\t-\n' "$id"; printf '%s\tsi\tsetup\n' "$id" >> "$OUT"; continue
  fi
  reconstruir_fuente "$id" "$tj" "$sucio"
done < <(ls "$K_ROOT/gold/s1")
echo "reconstruccion.tsv: $(($(wc -l < "$OUT"))) filas en $OUT"
