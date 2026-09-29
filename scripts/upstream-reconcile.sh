#!/usr/bin/env bash
# Uso: upstream-reconcile.sh <ledger.md> [<rama>]   (rama default: main)
# Promueve filas `propuesto` a `portado` si <rama> contiene un commit cuyo
# subject empieza por `port(upstream#<PR>):`, y fija su hash al de ese commit.
# El hash de la rama del PR no sirve: tras un squash ya no existe.
# stdout: `propuesto-sin-evidencia #<PR> <skill>` por fila que sigue propuesta.
# exit 2: ledger sin `upstream_tag:` o sin la cabecera de la tabla de filas.
set -uo pipefail

ledger="${1:-}"; rama="${2:-main}"
[ -f "$ledger" ] || { echo "upstream-reconcile: no existe '$ledger'" >&2; exit 2; }
dir="$(dirname "$ledger")"

trim() { local s="$1"; s="${s#"${s%%[![:space:]]*}"}"; s="${s%"${s##*[![:space:]]}"}"; printf '%s' "$s"; }

git -C "$dir" rev-parse --verify --quiet "$rama^{commit}" >/dev/null \
  || { echo "upstream-reconcile: rama $rama no existe o no es un commit" >&2; exit 2; }
grep -q '^upstream_tag:' "$ledger" || { echo "upstream-reconcile: falta 'upstream_tag:'" >&2; exit 2; }

tmp="$(mktemp)"; trap 'rm -f "$tmp"' EXIT
en_tabla=0; visto_cabecera=0
while IFS= read -r linea || [ -n "$linea" ]; do
  if [ $en_tabla = 0 ]; then
    if [[ "$linea" == \|* ]]; then
      IFS='|' read -ra c <<< "$linea"
      if [ "${#c[@]}" -ge 7 ] \
         && [ "$(trim "${c[1]}")" = PR ] && [ "$(trim "${c[2]}")" = skill ] \
         && [ "$(trim "${c[3]}")" = triage ] && [ "$(trim "${c[4]}")" = estado ] \
         && [ "$(trim "${c[5]}")" = motivo ] && [ "$(trim "${c[6]}")" = hash ]; then
        en_tabla=1; visto_cabecera=1
      fi
    fi
    printf '%s\n' "$linea" >> "$tmp"; continue
  fi
  if [[ "$linea" != \|* ]]; then en_tabla=2; fi
  if [ $en_tabla = 2 ]; then printf '%s\n' "$linea" >> "$tmp"; continue; fi

  IFS='|' read -ra c <<< "$linea"
  pr="$(trim "${c[1]:-}")"; pr="${pr#\#}"; skill="$(trim "${c[2]:-}")"
  if [ "$(trim "${c[4]:-}")" = propuesto ] && ! [[ "$pr" =~ ^[0-9]+$ ]]; then
    echo "propuesto-malformado $(trim "${c[1]:-}") $skill"
  elif [ "$(trim "${c[4]:-}")" = propuesto ]; then
    hash=""
    while IFS=$'\t' read -r h subj; do
      case "$subj" in "port(upstream#$pr):"*) hash="$h"; break ;; esac
    done < <(git -C "$dir" log --format='%H%x09%s' "$rama" --)
    if [ -n "$hash" ]; then
      c[4]=" portado "
      c[6]=" $(git -C "$dir" rev-parse --short "$hash") "
      nuevo=""; for x in "${c[@]:1}"; do nuevo+="|$x"; done
      linea="$nuevo|"
    else
      echo "propuesto-sin-evidencia #$pr $skill"
    fi
  fi
  printf '%s\n' "$linea" >> "$tmp"
done < "$ledger"

if [ $visto_cabecera = 0 ]; then
  echo "upstream-reconcile: falta la cabecera de la tabla de filas" >&2; exit 2
fi
cat "$tmp" > "$ledger"
exit 0
