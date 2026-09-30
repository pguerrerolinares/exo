#!/usr/bin/env bash
# Valida el pre-registro del test de techo: composicion fija, reglas presentes, orden parseable.
set -u
D="$(cd "$(dirname "$0")" && pwd)"
T="$D/tareas.tsv"
SUELO="g0-122 g0-149 g0-33 g1-131 g1-139 g1-140 g1-157 g2-154 g2-170 g2-171 g2-97"
fail=0
err() { echo "FALLA: $*" >&2; fail=1; }

[ -f "$T" ] || { echo "FALLA: falta $T" >&2; exit 1; }
[ -f "$D/preregistro.md" ] || err "falta preregistro.md"
[ "$(wc -l < "$T")" -eq 17 ] || err "tareas.tsv debe tener 17 filas"
[ "$(awk -F'\t' '$2=="suelo"' "$T" | wc -l)" -eq 11 ] || err "deben ser 11 suelo"
[ "$(awk -F'\t' '$2=="control"' "$T" | wc -l)" -eq 6 ] || err "deben ser 6 control"
[ "$(awk -F'\t' '$2=="suelo"{print $1}' "$T" | sort | tr '\n' ' ')" = "$(echo $SUELO | tr ' ' '\n' | sort | tr '\n' ' ')" ] || err "ids del suelo distintos de Global Constraints"

while IFS=$'\t' read -r id grupo rf; do
  [ "$rf" = "evals/techo-reglas/reglas/$id.txt" ] || err "$id: regla_file inesperado ($rf)"
  [ -s "$D/reglas/$id.txt" ] || err "$id: reglas/$id.txt ausente o vacio"
  if [ "$grupo" = control ]; then
    case " $SUELO g1-147 g1-57 " in *" $id "*) err "control $id prohibido";; esac
  fi
done < "$T"
[ "$(ls "$D/reglas" 2>/dev/null | wc -l)" -eq 17 ] || err "reglas/ debe tener 17 ficheros"

# orden: bloque entre marcadores, 28 lineas 'id rep', k=2 suelo, k=1 control
ORD="$(sed -n '/^<!-- ORDEN-BEGIN -->$/,/^<!-- ORDEN-END -->$/p' "$D/preregistro.md" | grep -E '^[a-z0-9-]+ [12]$')"
[ "$(echo "$ORD" | grep -c .)" -eq 28 ] || err "orden debe tener 28 lineas 'id rep'"
for id in $(cut -f1 "$T"); do
  g=$(awk -F'\t' -v i="$id" '$1==i{print $2}' "$T")
  n=$(echo "$ORD" | awk -v i="$id" '$1==i' | wc -l)
  want=2; [ "$g" = control ] && want=1
  [ "$n" -eq "$want" ] || err "$id: $n corridas en el orden, esperadas $want"
done
[ "$(echo "$ORD" | awk '{print $1" "$2}' | sort -u | wc -l)" -eq 28 ] || err "orden con duplicados"
for k in 20260930 claude-sonnet-5-5 "claude --version" sucio_en_K; do grep -q -- "$k" "$D/preregistro.md" || err "preregistro.md no menciona '$k'"; done

[ $fail -eq 0 ] && echo "preregistro OK (11+6)" || exit 1
