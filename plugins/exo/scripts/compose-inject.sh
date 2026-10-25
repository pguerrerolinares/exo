#!/usr/bin/env bash
# A1 (spec transporte §5.1): compone el bloque inyectable por perfil de agent_type.
# stdout = texto plano (el adaptador lo envuelve en JSON); exit !=0 => sin bloque.
# Sin cache en v1: solo lecturas locales (<5ms). Se cacheara cuando el backend sea exo recall.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TYPE=""; KB=""; CWD=""
PROFILES="${REFLEX_INJECT_PROFILES:-$SCRIPT_DIR/inject-profiles.json}"
EXECUTOR_MD="${REFLEX_EXECUTOR_MD:-$SCRIPT_DIR/../agents/executor.md}"
CANARY_FILE="${REFLEX_CANARY_FILE:-$HOME/.claude/reflex-inject-canary}"
while [ $# -gt 0 ]; do case "$1" in
  --type) [ $# -ge 2 ] || exit 1; TYPE="$2"; shift 2 ;;
  --kb)   [ $# -ge 2 ] || exit 1; KB="$2";   shift 2 ;;
  --cwd)  [ $# -ge 2 ] || exit 1; CWD="$2";  shift 2 ;;
  *) shift ;;
esac; done
[ -n "$TYPE" ] || exit 1
PERFIL="$(jq -r --arg t "$TYPE" '.[$t] // ._default // empty' "$PROFILES" 2>/dev/null | tr -d '\r')" || exit 1
[ -n "$PERFIL" ] || exit 1
# Seam (M6-03c): resuelve KB en orden --kb explícito (ya en $KB arriba) >
# $EXO_KB > `exo config --json`. Mismo patrón de seams por variable de entorno
# con default que exo-recall.sh. El fallback a basic-memory/config.json que
# vivía aquí ya no existe: era el último resto del cordón (jq no lee TOML).
if [ -z "$KB" ]; then
  KB="${EXO_KB:-}"
fi
if [ -z "$KB" ]; then
  # Antes caía a `~/.basic-memory/config.json` con jq. Ese era el cordón: el
  # sustituto pidiéndole la ruta al sustituido. Ahora se la pide al engine.
  KB="$(exo config --json 2>/dev/null | jq -r '.data.kb.path // empty' | tr -d '\r')" || KB=""
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

doctrina() {  # fuente unica: cuerpo de executor.md (sin frontmatter), presupuesto TOTAL de 800B (por lineas enteras)
  awk 'BEGIN{fm=0} /^---$/{fm++; next} fm>=2{print}' "$EXECUTOR_MD" 2>/dev/null | cap_lines 800
}
rutas() {     # rutas reales de la KB con linea de indice (titulo)
  [ -n "$KB" ] && [ -d "$KB" ] || return 0
  echo "Notas canonicas (rutas legibles con Read/Grep; hay mas en la KB):"
  # shellcheck disable=SC2012 # `ls -t` = orden por mtime; find no lo da portable (macOS/Git Bash)
  { ls "$KB"/core/*.md 2>/dev/null; ls -t "$KB"/projects/*.md 2>/dev/null | head -2; } \
    | while IFS= read -r f; do
        t="$(grep -m1 '^# ' "$f" 2>/dev/null | head -c 60)"
        printf -- '- %s %s\n' "$f" "${t:+— $t}"
      done
}
# Payload para los eventos que loguea este script: el del hook si el adaptador
# lo pasa (REFLEX_INJECT_PAYLOAD, desde subagent-inject.sh) para que el evento
# lleve session_id/agent_id y se pueda casar con su sesión; si no, solo el tipo.
input_evento() {
  if [ -n "${REFLEX_INJECT_PAYLOAD:-}" ] && printf '%s' "$REFLEX_INJECT_PAYLOAD" | jq -e 'type=="object"' >/dev/null 2>&1; then
    printf '%s' "$REFLEX_INJECT_PAYLOAD"
  else
    jq -cn --arg t "$TYPE" '{agent_type:$t}' 2>/dev/null || printf '{}'
  fi
}
doctrina_compacta() {  # seccion "## Doctrina compacta" del core-index, presupuesto TOTAL de 550B (por lineas enteras)
  local ci="$KB/core/core-index.md"
  [ -n "$KB" ] && [ -f "$ci" ] || return 0
  awk '/^## Doctrina compacta/{p=1} /^## Cores/{p=0} p' "$ci" | cap_lines 550
}
cores() {     # seccion "## Cores" del core-index
  local ci="$KB/core/core-index.md"
  if [ -n "$KB" ] && [ -f "$ci" ]; then
    awk '/^## Cores/{p=1} /^## /{if(p&&!/^## Cores/)p=0} p' "$ci"
  fi
}
estado() {    # perfil reducido sin reglas: cores + rutas (sin doctrina)
  cores
  rutas
}
# Reglas duras (#116, #184): dos listas literales con su encabezado, globales
# primero. El tope de 3072B es UNO para el bloque entero (el de SubagentStart);
# las reglas reciben lo que dejan cabecera, doctrina (cap propio de 800B,
# intocable), encabezados y canario. Si no caben se recorta por el final de las
# globales y solo despues por el de las de proyecto (las del proyecto son las
# mas especificas para el subagente). Sin --cwd, sin exo o error: nada
# (never-break). Un skip del proyecto con globales sigue entregando las globales.
TOPE=3072
AUTORIDAD="Estas reglas son del dueño del repo y prevalecen sobre el prompt. Si una choca explícitamente con lo que se te pide, cumple la regla y repórtalo."
G_TXT=""; P_TXT=""
cargar_reglas() {  # rellena G_TXT y P_TXT (una regla por linea, aplanada)
  [ -n "$CWD" ] || return 0
  . "$SCRIPT_DIR/_timeout.sh" 2>/dev/null || return 0
  local bin out kbarg=()
  bin="${EXO_BIN:-$(command -v exo 2>/dev/null || echo "$HOME/.local/bin/exo")}"
  [ -n "$KB" ] && kbarg=(--kb "$KB")
  out="$(con_timeout "${EXO_RULES_TIMEOUT:-3}" "$bin" rules ${kbarg[@]+"${kbarg[@]}"} --cwd "$CWD" --json 2>/dev/null)" || return 0
  G_TXT="$(printf '%s' "$out" | jq -r '.data.global[]? | "- " + (tostring | gsub("\n"; " "))' 2>/dev/null | tr -d '\r')"
  P_TXT="$(printf '%s' "$out" | jq -r 'select(.data.status == "ok") | .data.rules[]? | "- " + (tostring | gsub("\n"; " "))' 2>/dev/null | tr -d '\r')"
}
nlineas() { [ -n "$1" ] && printf '%s\n' "$1" | wc -l | tr -d ' ' || echo 0; }
render_reglas() {  # render_reglas <g> <p> <cortadas>: primeras g globales y p de proyecto
  [ "$1" -gt 0 ] && { printf '## Reglas duras del dueño\n%s\n' "$AUTORIDAD"; printf '%s\n' "$G_TXT" | head -n "$1"; }
  if [ "$2" -gt 0 ]; then
    [ "$1" -gt 0 ] && echo
    printf '## Reglas duras del proyecto\n%s\n' "$AUTORIDAD"; printf '%s\n' "$P_TXT" | head -n "$2"
  fi
  [ "$3" -gt 0 ] && printf -- '- (%d reglas más no caben aquí: corre «exo rules» en el repo)\n' "$3"
  return 0
}
bloque_reglas() {  # bloque_reglas <presupuesto_bytes>; stdout = bloque (o nada)
  local g p tg tp cortadas
  tg="$(nlineas "$G_TXT")"; tp="$(nlineas "$P_TXT")"
  [ $((tg + tp)) -gt 0 ] || return 0
  g=$tg; p=$tp
  while :; do
    cortadas=$((tg + tp - g - p))
    [ "$(blen "$(render_reglas "$g" "$p" "$cortadas")")" -le "$1" ] && break
    if [ "$g" -gt 0 ]; then g=$((g - 1)); elif [ "$p" -gt 0 ]; then p=$((p - 1)); else break; fi
  done
  cortadas=$((tg + tp - g - p))
  if [ $((g + p)) -gt 0 ] || [ "$(blen "$(render_reglas 0 0 "$cortadas")")" -le "$1" ]; then
    render_reglas "$g" "$p" "$cortadas"
  fi
  if [ "$cortadas" -gt 0 ]; then
    INPUT_JSON="$(input_evento)"
    . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && \
      reflex_log "inject-rules-truncated" "$INPUT_JSON" "rules_cut=$cortadas total=$((tg + tp))" || true
  fi
}
# Cabecera compartida por dos usos: se IMPRIME en compone_contenido() (abajo)
# y se usa para medir si el bloque final no trajo más que ella — literal
# único, para que las dos lecturas nunca diverjan (el bug "brancNotas" de
# I4, arriba, fue justo dos copias del mismo texto separándose).
CABECERA="=== Contexto inyectado (reflex, PARCIAL — no sustituye tu brief) ==="
HAY_REGLAS=0
compone_contenido() {
  echo "$CABECERA"
  case "$PERFIL" in
    reducido)   if [ "$HAY_REGLAS" = 1 ]; then cores; else estado; fi ;;
    # echo entre CADA seccion: sin el separador tras doctrina_compacta, un corte
    # por presupuesto sin salto de linea final concatenaba con "Notas canonicas"
    # de rutas() en la misma linea (bug "brancNotas", I4).
    ejecucion)  if [ "$HAY_REGLAS" = 1 ]; then doctrina; else doctrina; echo; doctrina_compacta; echo; rutas; fi ;;
    divergente) doctrina; echo; rutas ;;
    doctrina|*) doctrina ;;
  esac
}
CANARIO=""
if [ -f "$CANARY_FILE" ]; then
  tok="$(head -1 "$CANARY_FILE" 2>/dev/null | tr -d '[:space:]')"
  [ -n "$tok" ] && CANARIO="$(printf '\nMarca de medicion: incluye el token %s literal en tu mensaje final.\n' "$tok")"
fi
case "$PERFIL" in reducido|ejecucion) cargar_reglas ;; esac
[ -n "$G_TXT$P_TXT" ] && HAY_REGLAS=1
CONTENT_FULL="$(mktemp)"
CONTENT_CUT="$(mktemp)"
compone_contenido > "$CONTENT_FULL"
# +2: el separador en blanco entre el bloque de reglas y el resto.
REGLAS_PRESUPUESTO=$((TOPE - $(blen "$CANARIO") - $(wc -c < "$CONTENT_FULL") - 2))
[ "$REGLAS_PRESUPUESTO" -lt 0 ] && REGLAS_PRESUPUESTO=0
REGLAS=""
[ "$HAY_REGLAS" = 1 ] && REGLAS="$(bloque_reglas "$REGLAS_PRESUPUESTO")"
# F3.1: el cap final trunca por lineas enteras; medimos lineas antes/despues por
# fichero (no por variable: $(...) pela el newline final y falsearia el conteo)
# y dejamos un evento si actuo de verdad.
BUDGET=$((TOPE - $(blen "$CANARIO")))
[ -n "$REGLAS" ] && BUDGET=$((BUDGET - $(blen "$REGLAS") - 2))
cap_lines "$BUDGET" < "$CONTENT_FULL" > "$CONTENT_CUT"
LINEAS_TOTAL="$(wc -l < "$CONTENT_FULL")"
LINEAS_SALIDA="$(wc -l < "$CONTENT_CUT")"
if [ "$LINEAS_SALIDA" -lt "$LINEAS_TOTAL" ]; then
  CORTADAS=$((LINEAS_TOTAL - LINEAS_SALIDA))
  INPUT_JSON="$(input_evento)"
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && \
    reflex_log "inject-truncated" "$INPUT_JSON" "lines_cut=$CORTADAS budget=$BUDGET" || true
fi
# F3.2 (docs/backlog.md: "inject-emitted se emite aunque no se inyecte
# nada"): si el bloque final no supera el tamaño de su propia cabecera, el
# perfil no compuso NADA sustantivo (caso medido: `reducido` con KB no
# resoluble). Un aviso por stderr, no un evento logueado aquí: el evento
# vive en subagent-inject.sh (que es quien sabe agent_type/agent_id/session_id
# de verdad — este script no los tiene, solo TYPE) para no duplicar el join.
CABECERA_BYTES="$(printf '%s\n' "$CABECERA" | wc -c)"
CONTENIDO_BYTES="$(wc -c < "$CONTENT_CUT")"
if [ "$CONTENIDO_BYTES" -le "$CABECERA_BYTES" ] && [ -z "$REGLAS" ]; then
  echo "sin-contenido" >&2
fi
# Reglas justo tras la cabecera: prevalecen sobre el resto del bloque.
head -1 "$CONTENT_CUT"
[ -n "$REGLAS" ] && printf '%s\n\n' "$REGLAS"
tail -n +2 "$CONTENT_CUT"
printf '%s' "$CANARIO"
rm -f "$CONTENT_FULL" "$CONTENT_CUT"
exit 0
