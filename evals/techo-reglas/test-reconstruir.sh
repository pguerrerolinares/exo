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
  grep -q "^$id	.*sucio_en_K=$sucio\b" "$T" || { echo "  $id sucio=$sucio no reportado"; bad=1; }
done < <(awk '/^  g[0-9]/ && $4!="sucio=0" {sub(/sucio=/,"",$4); print $1, $4}' "$H/../ablacion-k/congelacion.txt")
[ $bad = 0 ] && ok sucio_de_K_se_reporta || ko sucio_de_K_se_reporta

[ "$(wc -l < "$T")" = 41 ] && ok "tsv 41 filas" || ko "tsv filas $(wc -l < "$T")"

# Lógica de reconstrucción sobre repos sintéticos (sin tarball ni preparar.sh).
W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
git init -q "$W/repo"; git -C "$W/repo" -c user.email=t@t -c user.name=t commit -q --allow-empty -m uno
c1=$(git -C "$W/repo" rev-parse HEAD)
mk() { printf '{"setup":false,"repo":"%s","commit":"%s","excluir":[]}' "$2" "$3" > "$W/$1.json"; }
mk ok "$W/repo" "$c1"; mk nocommit "$W/repo" 0123456789012345678901234567890123456789; mk norepo "$W/no-existe" "$c1"
fila_sint() { ( export K_ROOT="$W/k"; RECONSTRUIR_LIB=1; . "$H/reconstruir.sh"; mkdir -p "$K_ROOT"; : > "$OUT"; reconstruir_fuente "$1" "$W/$1.json" 3 >/dev/null; cat "$OUT" ) 2>&1; }
[ "$(fila_sint ok)" = "$(printf 'ok\tsi\tsucio_en_K=3 no reconstruible')" ] && [ "$(git -C "$W/k/fuentes/ok" rev-parse HEAD)" = "$c1" ] \
  && ok fuente_sintetica_si || ko "fuente_sintetica_si: $(fila_sint ok)"
[ "$(fila_sint nocommit)" = "$(printf 'nocommit\tno\tcommit-inexistente')" ] && [ ! -e "$W/k/fuentes/nocommit" ] \
  && ok fuente_commit_inexistente || ko "fuente_commit_inexistente: $(fila_sint nocommit)"
[ "$(fila_sint norepo)" = "$(printf 'norepo\tno\trepo-inexistente')" ] && ok fuente_repo_inexistente || ko "fuente_repo_inexistente: $(fila_sint norepo)"
[ "$(git -C "$W/repo" status --porcelain | wc -l)" = 0 ] && ok repo_fuente_intacto || ko repo_fuente_intacto

# rederivar_claude_md sobre un prep sintético (nunca toca ~/.claude/CLAUDE.md).
printf '# Reglas\nuna\n## Memoria de sesiones\nfuera\n## Otra\ndos\n' > "$W/CLAUDE.md"
esperado=$("$H/../ablacion-k/harness/filtro-claude-md.sh" < "$W/CLAUDE.md" | sha256sum | cut -c1-16)
rd() { ( export K_ROOT="$W/kp"; RECONSTRUIR_LIB=1; . "$H/reconstruir.sh"; prep_nota=""; rederivar_claude_md "$W/CLAUDE.md"; echo "$prep_nota|$prep_res" ); }
mkdir -p "$W/kp/prep"; echo "viejo" > "$W/kp/prep/claude-md.md"; printf 'kb_commit x\nclaude-md.md 0000000000000000\nnotas 1\n' > "$W/kp/prep/manifiesto.txt"
r=$(rd)
{ [[ "$r" == *"estaba obsoleto"* ]] && [[ "$r" == *"DIFIERE sha=$esperado esperado=6f3c6f382f90e794"* ]] \
  && grep -qx "claude-md.md $esperado" "$W/kp/prep/manifiesto.txt" && grep -q 'notas 1' "$W/kp/prep/manifiesto.txt"; } \
  && ok claude_md_obsoleto_se_repara || ko "claude_md_obsoleto_se_repara: $r"
r=$(rd); [[ "$r" != *obsoleto* && "$r" != *faltaba* ]] && ok claude_md_al_dia_sin_nota || ko "claude_md_al_dia_sin_nota: $r"
rm "$W/kp/prep/claude-md.md"; r=$(rd)
[[ "$r" == *"faltaba"* && "$r" != *obsoleto* ]] && ok claude_md_ausente_dice_faltaba || ko "claude_md_ausente_dice_faltaba: $r"

exit $fail
