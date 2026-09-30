#!/usr/bin/env bash
# Comprueba el entorno que dejó reconstruir.sh en $K_ROOT (correr reconstruir.sh antes).
set -uo pipefail
H="$(cd "$(dirname "$0")" && pwd)"
K_ROOT="${K_ROOT:-$HOME/.cache/exo-ablacion-k}"; T="$K_ROOT/reconstruccion.tsv"
[ -s "$T" ] || { echo "FAIL falta $T (correr reconstruir.sh)"; exit 1; }
fail=0; ok() { echo "ok   $1"; }; ko() { echo "FAIL $1"; fail=1; }

[ "$(sha256sum < "$K_ROOT/gold-activo.tar" | cut -d' ' -f1)" = c2d8835b74fdcd8e3ee6f9c3ff32efc3248caca5bc3aa5618d7ff9e3908c244c ] \
  && ok gold_activo_sha_coincide || ko gold_activo_sha_coincide

sha=$(sha256sum < "$K_ROOT/prep/claude-md.md" | cut -c1-16)
fila=$(awk -F'\t' '$1=="_prep"' "$T")
if [ "$sha" = 6f3c6f382f90e794 ]; then echo "$fila" | grep -q 'coincide' && ok prep_claude_md_sha || ko prep_claude_md_sha
else echo "$fila" | grep -q "DIFIERE sha=$sha" && ok "prep_claude_md_sha (difiere, reportado: $sha)" || ko "prep_claude_md_sha difiere sin reportar"; fi

bad=0
while read -r id; do
  tj="$K_ROOT/gold/s1/$id/tarea.json"
  [ "$(jq -r '.setup // false' "$tj")" = true ] && continue
  rec=$(awk -F'\t' -v i="$id" '$1==i{print $2}' "$T"); mot=$(awk -F'\t' -v i="$id" '$1==i{print $3}' "$T")
  if [ "$rec" = si ]; then
    [ "$(git -C "$K_ROOT/fuentes/$id" rev-parse HEAD 2>/dev/null)" = "$(jq -r .commit "$tj")" ] || { echo "  $id HEAD distinto marcado si"; bad=1; }
    jq -r '(.excluir // [])[]' "$tj" | while read -r e; do if [ -e "$K_ROOT/fuentes/$id/$e" ]; then exit 1; fi; done || { echo "  $id excluir no respetado"; bad=1; }
  else [ -n "$mot" ] || { echo "  $id no sin motivo"; bad=1; }; fi
done < <(ls "$K_ROOT/gold/s1")
[ $bad = 0 ] && ok fuente_en_commit_de_tarea || ko fuente_en_commit_de_tarea

bad=0
while read -r id sucio; do
  grep -q "^$id	.*sucio_en_K=$sucio" "$T" || { echo "  $id sucio=$sucio no reportado"; bad=1; }
done < <(awk '/^  g[0-9]/ && $4!="sucio=0" {sub(/sucio=/,"",$4); print $1, $4}' "$H/../ablacion-k/congelacion.txt")
[ $bad = 0 ] && ok sucio_de_K_se_reporta || ko sucio_de_K_se_reporta

[ "$(wc -l < "$T")" = 41 ] && ok "tsv 41 filas" || ko "tsv filas $(wc -l < "$T")"
exit $fail
