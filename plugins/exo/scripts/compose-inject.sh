#!/usr/bin/env bash
# A1 (spec transporte §5.1): compone el bloque inyectable por perfil de agent_type.
# stdout = texto plano (el adaptador lo envuelve en JSON); exit !=0 => sin bloque.
# Coste: cada proceso lanzado cuesta ~50 ms bajo Git Bash, así que el bloque se
# compone con builtins (arrays, ${#x}, read, expansión de parámetros) y solo
# se lanzan `jq` y `exo`. Sin `mapfile`/`${x,,}`: tiene que correr en bash 3.2 (macOS).
set -uo pipefail
_d=${0%/*}; [ "$_d" = "$0" ] && _d=.
SCRIPT_DIR="$(cd "$_d" && pwd)"
TYPE=""; KB=""; CWD=""
PROFILES="${REFLEX_INJECT_PROFILES:-$SCRIPT_DIR/inject-profiles.json}"
EXECUTOR_MD="${REFLEX_EXECUTOR_MD:-$SCRIPT_DIR/../agents/executor.md}"
# shellcheck source=_state-dir.sh
. "$SCRIPT_DIR/_state-dir.sh"
CANARY_FILE="$REFLEX_CANARY_FILE"
while [ $# -gt 0 ]; do case "$1" in
  --type) [ $# -ge 2 ] || exit 1; TYPE="$2"; shift 2 ;;
  --kb)   [ $# -ge 2 ] || exit 1; KB="$2";   shift 2 ;;
  --cwd)  [ $# -ge 2 ] || exit 1; CWD="$2";  shift 2 ;;
  *) shift ;;
esac; done
[ -n "$TYPE" ] || exit 1
# jq.exe emite CRLF: se quita con ${v%$'\r'} (builtin) en vez de `| tr -d '\r'`.
PERFIL="$(jq -r --arg t "$TYPE" '.[$t] // ._default // empty' "$PROFILES" 2>/dev/null)" || exit 1
PERFIL=${PERFIL%$'\r'}
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
  KB="$(exo config --json 2>/dev/null | jq -r '.data.kb.path // empty')" || KB=""
  KB=${KB%$'\r'}
fi

# Las funciones que antes imprimían y se capturaban con $(...) (un subshell
# cada una) dejan su resultado en una variable global.
CR=$'\r'
blen() { local LC_ALL=C; BL=${#1}; }  # longitud en BYTES (no caracteres; UTF-8 seguro) -> $BL

# El bloque se acumula línea a línea (LINEAS/LLEN: texto y bytes+1 de cada una;
# LB: total) en vez de en un fichero temporal: el tope trabaja sobre el array.
LINEAS=(); LLEN=(); NL=0; LB=0
emit() {
  local LC_ALL=C
  local n=$((${#1} + 1))
  LINEAS[NL]=$1; LLEN[NL]=$n; NL=$((NL + 1)); LB=$((LB + n))
}
# Tope por LINEAS ENTERAS: emite cada linea completa mientras quepa en el
# presupuesto; al llegar a una que no cabe entera la DESCARTA (no la corta a
# mitad) y para ahi. Los bytes se cuentan con LC_ALL=C (UTF-8 exacto).
cap_ini() { CAP_USED=0; CAP_MAX=$1; }
cap_emit() {  # cap_emit <linea>: emite si cabe; devuelve 1 (y no emite) si no
  local LC_ALL=C
  local n=$((${#1} + 1))
  [ $((CAP_USED + n)) -gt "$CAP_MAX" ] && return 1
  CAP_USED=$((CAP_USED + n))
  emit "$1"
}

doctrina() {  # fuente unica: cuerpo de executor.md (sin frontmatter), presupuesto TOTAL de 800B (por lineas enteras)
  local l fm=0
  [ -r "$EXECUTOR_MD" ] || return 0
  cap_ini 800
  while IFS= read -r l || [ -n "$l" ]; do
    l=${l%"$CR"}  # awk de Git Bash descartaba el CR de un .md con CRLF
    if [ "$l" = "---" ]; then fm=$((fm + 1)); continue; fi
    [ "$fm" -ge 2 ] || continue
    cap_emit "$l" || break
  done < "$EXECUTOR_MD"
}
ruta_linea() {  # ruta_linea <fichero>: "- ruta — titulo" (primer "# ", 60 bytes)
  local LC_ALL=C
  local l t=""
  while IFS= read -r l || [ -n "$l" ]; do
    case "$l" in "# "*) t=$l; break ;; esac
  done 2>/dev/null < "$1"
  t=${t%$'\r'}  # el grep de Git Bash descartaba el CR de un .md con CRLF
  t=${t:0:60}
  emit "- $1 ${t:+— $t}"
}
rutas() {     # rutas reales de la KB con linea de indice (titulo)
  [ -n "$KB" ] && [ -d "$KB" ] || return 0
  emit "Notas canonicas (rutas legibles con Read/Grep; hay mas en la KB):"
  local f n=0 recientes
  for f in "$KB"/core/*.md; do
    [ -e "$f" ] || continue
    ruta_linea "$f"
  done
  # shellcheck disable=SC2012 # `ls -t` = orden por mtime; find no lo da portable (macOS/Git Bash)
  recientes="$(ls -t "$KB"/projects/*.md 2>/dev/null)"
  if [ -n "$recientes" ]; then
    while [ "$n" -lt 2 ] && IFS= read -r f; do
      ruta_linea "$f"
      n=$((n + 1))
    done <<<"$recientes"
  fi
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
  local ci="$KB/core/core-index.md" l p=0
  [ -n "$KB" ] && [ -f "$ci" ] || return 0
  cap_ini 550
  while IFS= read -r l || [ -n "$l" ]; do
    l=${l%"$CR"}
    case "$l" in "## Doctrina compacta"*) p=1 ;; esac
    case "$l" in "## Cores"*) p=0 ;; esac
    [ "$p" = 1 ] || continue
    cap_emit "$l" || break
  done < "$ci"
}
cores() {     # seccion "## Cores" del core-index
  local ci="$KB/core/core-index.md" l p=0
  [ -n "$KB" ] && [ -f "$ci" ] || return 0
  while IFS= read -r l || [ -n "$l" ]; do
    l=${l%"$CR"}
    case "$l" in
      "## Cores"*) p=1 ;;
      "## "*) p=0 ;;
    esac
    [ "$p" = 1 ] && emit "$l"
  done < "$ci"
  return 0
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
G_ARR=(); P_ARR=(); TG=0; TP=0
cargar_reglas() {  # rellena G_ARR/P_ARR (una regla por elemento, aplanada) y TG/TP
  [ -n "$CWD" ] || return 0
  . "$SCRIPT_DIR/_timeout.sh" 2>/dev/null || return 0
  local bin out l kbarg=()
  bin="${EXO_BIN:-}"
  if [ -z "$bin" ]; then
    # hash + BASH_CMDS = `command -v exo` sin subshell
    if hash exo 2>/dev/null; then bin=${BASH_CMDS[exo]}; else bin="$HOME/.local/bin/exo"; fi
  fi
  [ -n "$KB" ] && kbarg=(--kb "$KB")
  out="$(con_timeout "${EXO_RULES_TIMEOUT:-3}" "$bin" rules ${kbarg[@]+"${kbarg[@]}"} --cwd "$CWD" --json 2>/dev/null)" || return 0
  # Un solo jq: prefijo G/P por línea para repartir las dos listas.
  while IFS= read -r l; do
    l=${l//$'\r'/}
    case "$l" in
      G*) G_ARR[TG]=${l#G}; TG=$((TG + 1)) ;;
      P*) P_ARR[TP]=${l#P}; TP=$((TP + 1)) ;;
    esac
  done < <(jq -r '
    (.data.global[]? | "G- " + (tostring | gsub("\n"; " "))),
    (select(.data.status == "ok") | .data.rules[]? | "P- " + (tostring | gsub("\n"; " ")))' <<<"$out" 2>/dev/null)
}
render_reglas() {  # render_reglas <g> <p> <cortadas> -> $R: primeras g globales y p de proyecto
  local i msg
  R=""
  if [ "$1" -gt 0 ]; then
    R+="## Reglas duras del dueño"$'\n'"$AUTORIDAD"$'\n'
    for ((i = 0; i < $1; i++)); do R+="${G_ARR[i]}"$'\n'; done
  fi
  if [ "$2" -gt 0 ]; then
    [ "$1" -gt 0 ] && R+=$'\n'
    R+="## Reglas duras del proyecto"$'\n'"$AUTORIDAD"$'\n'
    for ((i = 0; i < $2; i++)); do R+="${P_ARR[i]}"$'\n'; done
  fi
  if [ "$3" -gt 0 ]; then
    printf -v msg -- '- (%d reglas más no caben aquí: corre «exo rules» en el repo)\n' "$3"
    R+=$msg
  fi
  # Como $(...): sin saltos de linea finales.
  while [[ "$R" == *$'\n' ]]; do R=${R%$'\n'}; done
}
REGLAS=""
bloque_reglas() {  # bloque_reglas <presupuesto_bytes>; deja el bloque (o nada) en $REGLAS
  local g p cortadas
  REGLAS=""
  [ $((TG + TP)) -gt 0 ] || return 0
  g=$TG; p=$TP
  while :; do
    cortadas=$((TG + TP - g - p))
    render_reglas "$g" "$p" "$cortadas"; blen "$R"
    [ "$BL" -le "$1" ] && break
    if [ "$g" -gt 0 ]; then g=$((g - 1)); elif [ "$p" -gt 0 ]; then p=$((p - 1)); else break; fi
  done
  cortadas=$((TG + TP - g - p))
  render_reglas 0 0 "$cortadas"; blen "$R"
  if [ $((g + p)) -gt 0 ] || [ "$BL" -le "$1" ]; then
    render_reglas "$g" "$p" "$cortadas"
    REGLAS=$R
  fi
  if [ "$cortadas" -gt 0 ]; then
    INPUT_JSON="$(input_evento)"
    . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && \
      reflex_log "inject-rules-truncated" "$INPUT_JSON" "rules_cut=$cortadas total=$((TG + TP))" || true
  fi
}
# Cabecera compartida por dos usos: se IMPRIME en compone_contenido() (abajo)
# y se usa para medir si el bloque final no trajo más que ella — literal
# único, para que las dos lecturas nunca diverjan (el bug "brancNotas" de
# I4, arriba, fue justo dos copias del mismo texto separándose).
CABECERA="=== Contexto inyectado (reflex, PARCIAL — no sustituye tu brief) ==="
HAY_REGLAS=0
compone_contenido() {
  emit "$CABECERA"
  case "$PERFIL" in
    reducido)   if [ "$HAY_REGLAS" = 1 ]; then cores; else estado; fi ;;
    # emit "" entre CADA seccion: sin el separador tras doctrina_compacta, un corte
    # por presupuesto sin salto de linea final concatenaba con "Notas canonicas"
    # de rutas() en la misma linea (bug "brancNotas", I4).
    ejecucion)  if [ "$HAY_REGLAS" = 1 ]; then doctrina; else doctrina; emit ""; doctrina_compacta; emit ""; rutas; fi ;;
    divergente) doctrina; emit ""; rutas ;;
    doctrina|*) doctrina ;;
  esac
}
CANARIO=""
if [ -f "$CANARY_FILE" ]; then
  tok=""
  IFS= read -r tok 2>/dev/null < "$CANARY_FILE" || true
  tok=${tok//[[:space:]]/}
  if [ -n "$tok" ]; then
    printf -v CANARIO '\nMarca de medicion: incluye el token %s literal en tu mensaje final.\n' "$tok"
    CANARIO=${CANARIO%$'\n'}
  fi
fi
case "$PERFIL" in reducido|ejecucion) cargar_reglas ;; esac
[ $((TG + TP)) -gt 0 ] && HAY_REGLAS=1
compone_contenido
blen "$CANARIO"; CANARIO_B=$BL
# +2: el separador en blanco entre el bloque de reglas y el resto.
REGLAS_PRESUPUESTO=$((TOPE - CANARIO_B - LB - 2))
[ "$REGLAS_PRESUPUESTO" -lt 0 ] && REGLAS_PRESUPUESTO=0
[ "$HAY_REGLAS" = 1 ] && bloque_reglas "$REGLAS_PRESUPUESTO"
# F3.1: el cap final trunca por lineas enteras; contamos lineas antes/despues
# y dejamos un evento si actuo de verdad.
BUDGET=$((TOPE - CANARIO_B))
if [ -n "$REGLAS" ]; then
  blen "$REGLAS"
  BUDGET=$((BUDGET - BL - 2))
fi
cap_ini "$BUDGET"
NC=0
while [ "$NC" -lt "$NL" ]; do
  [ $((CAP_USED + LLEN[NC])) -gt "$CAP_MAX" ] && break
  CAP_USED=$((CAP_USED + LLEN[NC])); NC=$((NC + 1))
done
CONTENIDO_BYTES=$CAP_USED
if [ "$NC" -lt "$NL" ]; then
  CORTADAS=$((NL - NC))
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
blen "$CABECERA"; CABECERA_BYTES=$((BL + 1))
if [ "$CONTENIDO_BYTES" -le "$CABECERA_BYTES" ] && [ -z "$REGLAS" ]; then
  echo "sin-contenido" >&2
fi
# Reglas justo tras la cabecera: prevalecen sobre el resto del bloque.
OUT=""
[ "$NC" -ge 1 ] && OUT+="${LINEAS[0]}"$'\n'
[ -n "$REGLAS" ] && OUT+="$REGLAS"$'\n\n'
for ((i = 1; i < NC; i++)); do OUT+="${LINEAS[i]}"$'\n'; done
OUT+=$CANARIO
printf '%s' "$OUT"
exit 0
