#!/usr/bin/env bash
# A1 (spec transporte §5.1): compone el bloque inyectable por perfil de agent_type.
# stdout = texto plano (el adaptador lo envuelve en JSON); exit !=0 => sin bloque.
# Sin cache en v1: solo lecturas locales (<5ms). Se cacheara cuando el backend sea exo recall.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TYPE=""; KB=""
PROFILES="${REFLEX_INJECT_PROFILES:-$SCRIPT_DIR/inject-profiles.json}"
EXECUTOR_MD="${REFLEX_EXECUTOR_MD:-$SCRIPT_DIR/../agents/executor.md}"
CANARY_FILE="${REFLEX_CANARY_FILE:-$HOME/.claude/reflex-inject-canary}"
while [ $# -gt 0 ]; do case "$1" in
  --type) [ $# -ge 2 ] || exit 1; TYPE="$2"; shift 2 ;;
  --kb)   [ $# -ge 2 ] || exit 1; KB="$2";   shift 2 ;;
  *) shift ;;
esac; done
[ -n "$TYPE" ] || exit 1
PERFIL="$(jq -r --arg t "$TYPE" '.[$t] // ._default // empty' "$PROFILES" 2>/dev/null)" || exit 1
[ -n "$PERFIL" ] || exit 1
if [ -z "$KB" ]; then
  KB="$(jq -r '.projects["kb-demo"].path // .projects["kb-demo"] // empty' \
        "$HOME/.basic-memory/config.json" 2>/dev/null)" || KB=""
fi

blen() { printf '%s' "$1" | wc -c; }  # longitud en BYTES (no caracteres; UTF-8 seguro)

cap_lines() {  # cap_lines <presupuesto_bytes> — trunca por LINEAS ENTERAS: imprime
  # cada linea completa mientras quepa en el presupuesto; al llegar a una linea
  # que no cabe entera la DESCARTA (no la corta a mitad) y para ahi. LC_ALL=C
  # fuerza a length() de awk a contar bytes, no caracteres (UTF-8 exacto).
  LC_ALL=C awk -v budget="$1" '
    { n = length($0) + 1
      if (used + n > budget) exit
      print
      used += n
    }'
}

doctrina() {  # fuente unica: cuerpo de executor.md (sin frontmatter), cap 800B por linea
  awk 'BEGIN{fm=0} /^---$/{fm++; next} fm>=2{print}' "$EXECUTOR_MD" 2>/dev/null | cap_lines 800
}
rutas() {     # rutas reales de la KB con linea de indice (titulo)
  [ -n "$KB" ] && [ -d "$KB" ] || return 0
  echo "Notas canonicas (rutas legibles con Read/Grep; hay mas en la KB):"
  { ls "$KB"/core/*.md 2>/dev/null; ls -t "$KB"/projects/*.md 2>/dev/null | head -2; } \
    | while IFS= read -r f; do
        t="$(grep -m1 '^# ' "$f" 2>/dev/null | head -c 60)"
        printf -- '- %s %s\n' "$f" "${t:+— $t}"
      done
}
doctrina_compacta() {  # seccion "## Doctrina compacta" del core-index, cap 550B por linea
  local ci="$KB/core/core-index.md"
  [ -n "$KB" ] && [ -f "$ci" ] || return 0
  awk '/^## Doctrina compacta/{p=1} /^## Cores/{p=0} p' "$ci" | cap_lines 550
}
estado() {    # seccion "## Cores" del core-index + rutas (perfil reducido: sin doctrina)
  local ci="$KB/core/core-index.md"
  if [ -n "$KB" ] && [ -f "$ci" ]; then
    awk '/^## Cores/{p=1} /^## /{if(p&&!/^## Cores/)p=0} p' "$ci"
  fi
  rutas
}
compone_contenido() {
  echo "=== Contexto inyectado (reflex, PARCIAL — no sustituye tu brief) ==="
  case "$PERFIL" in
    reducido)   estado ;;
    # echo entre CADA seccion: sin el separador tras doctrina_compacta, un corte
    # por presupuesto sin salto de linea final concatenaba con "Notas canonicas"
    # de rutas() en la misma linea (bug "brancNotas", I4).
    ejecucion)  doctrina; echo; doctrina_compacta; echo; rutas ;;
    divergente) doctrina; echo; rutas ;;
    doctrina|*) doctrina ;;
  esac
}
CANARIO=""
if [ -f "$CANARY_FILE" ]; then
  tok="$(head -1 "$CANARY_FILE" 2>/dev/null | tr -d '[:space:]')"
  [ -n "$tok" ] && CANARIO="$(printf '\nMarca de medicion: incluye el token %s literal en tu mensaje final.\n' "$tok")"
fi
# F3.1: este cap final (a diferencia de los internos de doctrina()/doctrina_compacta(),
# ya acotados) es el unico que puede recortar rutas() sin que nadie se entere — trunca
# lineas enteras en silencio. Medimos lineas antes/despues por fichero (no por variable:
# $(...) pela el newline final y falsearia el conteo) y dejamos un evento si el cap
# actuo de verdad, con cuantas lineas se perdieron — el dato que permite reaccionar.
BUDGET=$((2048 - $(blen "$CANARIO")))
CONTENT_FULL="$(mktemp)"
CONTENT_CUT="$(mktemp)"
compone_contenido > "$CONTENT_FULL"
cap_lines "$BUDGET" < "$CONTENT_FULL" > "$CONTENT_CUT"
LINEAS_TOTAL="$(wc -l < "$CONTENT_FULL")"
LINEAS_SALIDA="$(wc -l < "$CONTENT_CUT")"
if [ "$LINEAS_SALIDA" -lt "$LINEAS_TOTAL" ]; then
  CORTADAS=$((LINEAS_TOTAL - LINEAS_SALIDA))
  INPUT_JSON="$(jq -cn --arg t "$TYPE" '{agent_type:$t}' 2>/dev/null)" || INPUT_JSON='{}'
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && \
    reflex_log "inject-truncated" "$INPUT_JSON" "lines_cut=$CORTADAS budget=$BUDGET" || true
fi
cat "$CONTENT_CUT"
printf '%s' "$CANARIO"
rm -f "$CONTENT_FULL" "$CONTENT_CUT"
exit 0
