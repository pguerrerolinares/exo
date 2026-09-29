#!/usr/bin/env bash
# Watchdog del bot upstream-sync: rojo si el último latido (comentario del issue
# de estado) es más viejo que <max_dias> o si no hay issue/latidos. Toda ausencia es rojo.
# Uso: upstream-watchdog.sh <comentarios.json|-> [<ahora_iso>] [<max_dias>]
set -uo pipefail

f="${1:-}"
ahora="${2:-$(date -u +%Y-%m-%dT%H:%M:%SZ)}"
max="${3:-8}"

[ -n "$f" ] || { echo "uso: $0 <comentarios.json|-> [<ahora_iso>] [<max_dias>]" >&2; exit 2; }
if [ "$f" = "-" ]; then
  echo "issue upstream-sync: estado no existe"; exit 1
fi

# max() sobre strings ISO-8601 en UTC: el orden léxico es el cronológico
ultimo="$(jq -r 'if type=="array" then ([.[].created_at] | max // "") else "" end' "$f" 2>/dev/null)" || ultimo=""
if [ -z "$ultimo" ] || [ "$ultimo" = "null" ]; then
  echo "sin latidos"; exit 1
fi

t_ultimo="$(date -u -d "$ultimo" +%s 2>/dev/null)" && t_ahora="$(date -u -d "$ahora" +%s 2>/dev/null)" || {
  echo "fecha ilegible: último '$ultimo', ahora '$ahora'"; exit 1; }

diff=$((t_ahora - t_ultimo))
dias=$((diff / 86400))
if [ "$diff" -le $((max * 86400)) ]; then
  echo "latido OK: $ultimo ($dias días)"; exit 0
fi
echo "latido caducado: último $ultimo ($dias días > $max)"; exit 1
