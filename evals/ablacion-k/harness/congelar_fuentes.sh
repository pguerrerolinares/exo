#!/usr/bin/env bash
# congelar_fuentes.sh <dir gold> — copia exacta de la fuente de cada tarea con repo real
# (working tree + .git + sin commitear + node_modules/binarios) en $K_ROOT/fuentes/<id>/.
# Las corridas copian de ahí: la fuente queda fija aunque el repo original siga cambiando.
# Imprime por tarea: id, HEAD, si difiere del commit de tarea.json, si está sucio, y tamaño.
set -uo pipefail
gold=$1
K_ROOT="${K_ROOT:-$HOME/.cache/exo-ablacion-k}"; F="$K_ROOT/fuentes"; mkdir -p "$F"
for t in "$gold"/*/; do
  id=$(basename "$t"); tj="$t/tarea.json"
  jq -e '.no_convertible // false' "$tj" >/dev/null 2>&1 && [ "$(jq -r '.no_convertible // false' "$tj")" = true ] && continue
  [ "$(jq -r '.setup // false' "$tj")" = true ] && continue
  repo=$(jq -r .repo "$tj"); commit=$(jq -r .commit "$tj")
  rm -rf "$F/$id"
  cp -a --reflink=auto "$repo" "$F/$id" || { echo "$id ERROR copia"; continue; }
  git -C "$F/$id" remote 2>/dev/null | while read -r r; do git -C "$F/$id" remote remove "$r"; done
  head=$(git -C "$F/$id" rev-parse HEAD 2>/dev/null)
  sucio=$(git -C "$F/$id" status --porcelain 2>/dev/null | wc -l)
  printf '%s\tHEAD=%s\t%s\tsucio=%s\t%s\n' "$id" "${head:0:12}" \
    "$([ "$head" = "$commit" ] && echo coincide || echo "DIFIERE(tarea=${commit:0:12})")" "$sucio" "$(du -sh "$F/$id" | cut -f1)"
done
