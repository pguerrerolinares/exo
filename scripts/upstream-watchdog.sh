#!/usr/bin/env bash
# Watchdog del bot upstream-sync: rojo si el último latido (comentario del issue
# de estado con línea `estado: ok|alerta`) está en alerta o es más viejo que
# <max_dias>, o si no hay issue/latidos. Toda ausencia es rojo.
# Uso: upstream-watchdog.sh <comentarios.json|-> [<ahora_iso>] [<max_dias>]
set -uo pipefail

f="${1:-}"
ahora="${2:-$(date -u +%Y-%m-%dT%H:%M:%SZ)}"
max="${3:-8}"

[ -n "$f" ] || { echo "uso: $0 <comentarios.json|-> [<ahora_iso>] [<max_dias>]" >&2; exit 2; }
if [ "$f" = "-" ]; then
  echo "issue upstream-sync: estado no existe"; exit 1
fi

command -v jq >/dev/null 2>&1 || { echo "jq no disponible"; exit 1; }

# Solo cuentan los comentarios con una línea `estado: ok|alerta` (los latidos): un
# comentario humano no renueva el reloj. max_by sobre ISO-8601 UTC: léxico = cronológico.
sel='if type=="array" then ([.[] | select((.body // "") | test("(^|\n)estado: (ok|alerta)[ \t\r]*(\n|$)"))
  | {t: .created_at, e: (.body | capture("(^|\n)estado: (?<e>ok|alerta)") | .e)}] | max_by(.t) // empty
  | "\(.t) \(.e)") else empty end'
lat="$(jq -r "$sel" "$f" 2>/dev/null)" || lat=""
ultimo="${lat%% *}"; estado="${lat##* }"
if [ -z "$ultimo" ] || [ "$ultimo" = "null" ]; then
  echo "sin latidos"; exit 1
fi
if [ "$estado" = alerta ]; then
  echo "latido en alerta: $ultimo"; exit 1
fi

t_ultimo="$(date -u -d "$ultimo" +%s 2>/dev/null)" && t_ahora="$(date -u -d "$ahora" +%s 2>/dev/null)" || {
  echo "fecha ilegible: último '$ultimo', ahora '$ahora'"; exit 1; }

diff=$((t_ahora - t_ultimo))
dias=$((diff / 86400))
if [ "$diff" -le $((max * 86400)) ]; then
  echo "latido OK: $ultimo ($dias días)"; exit 0
fi
echo "latido caducado: último $ultimo ($dias días > $max)"; exit 1
