#!/usr/bin/env bash
# Watchdog del bot upstream-sync: rojo si el último latido (comentario del issue
# de estado con línea `estado: ok|alerta`) está en alerta o es más viejo que
# <max_dias> (default: 15 días), o si no hay issue/latidos. Toda ausencia es rojo.
# Frescura: si se pasa <tag_upstream>, rojo si difiere del `tag revisado:` del último
# latido y se publicó hace más de <max_dias_tag> días (default: 3). Sin <tag_upstream>
# (compat con llamadas antiguas) no se comprueba; vacío = rojo (el workflow no lo obtuvo).
# Uso: upstream-watchdog.sh <comentarios.json|-> [<ahora_iso>] [<max_dias>]
#                           [<tag_upstream> <fecha_tag_iso> [<max_dias_tag>]]
set -uo pipefail

f="${1:-}"
ahora="${2:-$(date -u +%Y-%m-%dT%H:%M:%SZ)}"
max="${3:-15}"
comprobar_tag=0; [ $# -ge 4 ] && comprobar_tag=1
tag_up="${4:-}"; fecha_tag="${5:-}"; max_tag="${6:-3}"

[ -n "$f" ] || { echo "uso: $0 <comentarios.json|-> [<ahora_iso>] [<max_dias>] [<tag_upstream> <fecha_tag_iso> [<max_dias_tag>]]" >&2; exit 2; }
if [ "$f" = "-" ]; then
  echo "issue upstream-sync: estado no existe"; exit 1
fi

command -v jq >/dev/null 2>&1 || { echo "jq no disponible"; exit 1; }

# Solo cuentan los comentarios con una línea `estado: ok|alerta` (los latidos): un
# comentario humano no renueva el reloj. max_by sobre ISO-8601 UTC: léxico = cronológico.
sel='if type=="array" then ([.[] | select((.body // "") | test("(^|\n)estado: (ok|alerta)[ \t\r]*(\n|$)"))
  | {t: .created_at, e: ([.body | capture("(^|\n)estado: (?<e>ok|alerta)"; "g") | .e] | last),
     g: ([.body | capture("(^|\n)tag revisado:[ \t]*(?<g>[^ \t\r\n]+)"; "gi") | .g] | last // "")}] | max_by(.t) // empty
  | "\(.t) \(.e) \(.g)") else empty end'
# tr: el jq nativo de Windows emite CRLF y el \r se pegaría al último campo.
lat="$(jq -r "$sel" "$f" 2>/dev/null | tr -d '\r')" || lat=""
read -r ultimo estado tag_rev <<< "$lat"
if [ -z "$ultimo" ] || [ "$ultimo" = "null" ]; then
  echo "sin latidos"; exit 1
fi
if [ "$estado" = alerta ]; then
  echo "latido en alerta: $ultimo"; exit 1
fi

if ! t_ultimo="$(date -u -d "$ultimo" +%s 2>/dev/null)" || ! t_ahora="$(date -u -d "$ahora" +%s 2>/dev/null)"; then
  echo "fecha ilegible: último '$ultimo', ahora '$ahora'"; exit 1
fi

diff=$((t_ahora - t_ultimo))
dias=$((diff / 86400))
if [ "$diff" -gt $((max * 86400)) ]; then
  echo "latido caducado: último $ultimo ($dias días > $max)"; exit 1
fi

if [ "$comprobar_tag" = 0 ]; then
  echo "latido OK: $ultimo ($dias días)"; exit 0
fi

if [ -z "$tag_up" ] || [ -z "$fecha_tag" ]; then
  echo "tag upstream no obtenido"; exit 1
fi
if [ -z "$tag_rev" ]; then
  echo "latido sin línea 'tag revisado': $ultimo"; exit 1
fi
if [ "${tag_rev#v}" = "${tag_up#v}" ]; then
  echo "latido OK: $ultimo ($dias días), tag al día: $tag_up"; exit 0
fi
if ! t_tag="$(date -u -d "$fecha_tag" +%s 2>/dev/null)"; then
  echo "fecha del tag ilegible: '$fecha_tag'"; exit 1
fi
edad=$((t_ahora - t_tag))
dias_tag=$((edad / 86400))
if [ "$edad" -le $((max_tag * 86400)) ]; then
  echo "latido OK: $ultimo ($dias días), tag nuevo $tag_up de hace $dias_tag días (revisado: $tag_rev)"; exit 0
fi
echo "tag sin revisar: upstream $tag_up de hace $dias_tag días (> $max_tag), el último latido revisó $tag_rev"; exit 1
