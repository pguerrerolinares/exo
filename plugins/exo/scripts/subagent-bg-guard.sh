#!/usr/bin/env bash
# SubagentStop hook (#170): un subagente no termina con tareas de fondo propias vivas.
#
# Por qué: un `exo:executor` lanzó `find /` en foreground; a los 120 s el harness
# lo movió a background, el executor entregó su report sin `TaskStop` y quedó
# «waiting on its own background work» ~3 h hasta que se mató a mano. En otro caso
# un `until ! pgrep -f ...` huérfano vivió ~21 h. El harness no mata el fondo de
# un subagente al acabar su turno.
#
# Qué hace: lee el transcript del propio subagente (`agent_transcript_path`),
# saca los ids de sus tareas de fondo (tool_result «was moved to the background
# (ID: x)», «Command running in background with ID: x», «Monitor started (task x»),
# descuenta las que tienen `TaskStop` y, si alguna sigue viva ⇒ `decision: block`
# con la lista, para que haga TaskStop o reporte BLOCKED. `background_tasks` es de
# la sesión padre (incluye lo que el padre dejó a propósito), así que solo se usa
# para confirmar `running` de ids que ya son propios; sin él, la tarea vive si su
# `.output` existe y no tiene la marca `[exited with code`.
#
# Abstención: sin jq, sin `agent_id`, `stop_hook_active`, sin transcript legible,
# sin tareas propias vivas. Si SubagentStop disparase al acabar el fondo y no al
# acabar el turno (no documentado), ya no hay nada vivo y calla. No cubierto: un
# Monitor sin `background_tasks` (su resultado no trae ruta de log).
set -uo pipefail

command -v jq >/dev/null 2>&1 || exit 0
INPUT="$(cat)"

ROW="$(printf '%s' "$INPUT" | jq -r '[
    (.stop_hook_active // false),
    (.agent_id // ""),
    (.agent_transcript_path // ""),
    (if has("background_tasks") then "1" else "0" end),
    ([(.background_tasks // [])[] | select(.status == "running") | (.id // "?")] | join(" "))
  ] | map(tostring) | join("\u001f")' 2>/dev/null)" || exit 0
ROW="${ROW%$'\r'}"
IFS=$'\x1f' read -r ACTIVE AGENT TRANSCRIPT HAS_BG RUNNING <<< "$ROW"

[ "$ACTIVE" = "true" ] && exit 0
[ -z "$AGENT" ] && exit 0
[ -z "$TRANSCRIPT" ] && exit 0

# Builtins y un solo grep: en Git Bash cada proceso cuesta ~30-40 ms y el hook
# corre en cada parada de subagente. Sin `declare -A` ni `${x,,}`: bash 3.2.
#
# `\` -> `/` (en el JSONL la ruta Windows sale escapada, `C:\\x`); Git Bash lee
# `C:/x` tal cual. No-op en POSIX.
slash() { PX="${1//\\//}"; while [[ "$PX" == *//* ]]; do PX="${PX//\/\//\/}"; done; }

slash "$TRANSCRIPT"; TRANSCRIPT="$PX"
[ -r "$TRANSCRIPT" ] || exit 0

# El ancla `"tool_result","content":"` sin escapar descarta las mismas frases
# citadas dentro de otro contenido (un Read o un grep de otro transcript).
P='"tool_result","content":"'
BG='Command did not complete within its [0-9]+s timeout and was moved to the background \(ID: [a-z0-9]+\)|Command running in background with ID: [a-z0-9]+'
HITS="$(grep -oE "${P}(${BG})\. Output is being written to: [^ ]+\.output|${P}Monitor started \(task [a-z0-9]+,|\"name\":\"TaskStop\",\"input\":\{\"task_id\":\"[a-z0-9]+\"" "$TRANSCRIPT" 2>/dev/null)"
[ -z "$HITS" ] && exit 0

STOPPED=" " SEEN=" " OWN=()
while IFS= read -r H; do
  if [[ "$H" =~ \"task_id\":\"([a-z0-9]+)\" ]]; then
    STOPPED+="${BASH_REMATCH[1]} "
  elif [[ "$H" =~ ID:\ ([a-z0-9]+)\)?\.\ Output\ is\ being\ written\ to:\ (.+)$ ]] \
    || [[ "$H" =~ Monitor\ started\ \(task\ ([a-z0-9]+), ]]; then
    case "$SEEN" in *" ${BASH_REMATCH[1]} "*) continue ;; esac
    SEEN+="${BASH_REMATCH[1]} "
    OWN+=("${BASH_REMATCH[1]}"$'\t'"${BASH_REMATCH[2]:-}")
  fi
done <<< "$HITS"

LIVE=""
for E in ${OWN[@]+"${OWN[@]}"}; do
  ID="${E%%$'\t'*}" OUT="${E#*$'\t'}"
  case "$STOPPED" in *" $ID "*) continue ;; esac
  [ -n "$OUT" ] && { slash "$OUT"; OUT="$PX"; }
  if [ "$HAS_BG" = "1" ]; then
    case " $RUNNING " in *" $ID "*) ;; *) continue ;; esac
  else
    [ -f "$OUT" ] || continue
    [[ "$(tail -c 4096 "$OUT" 2>/dev/null)" == *"[exited with code"* ]] && continue
  fi
  LIVE="${LIVE:+$LIVE, }${ID}${OUT:+ (log: $OUT)}"
done
[ -z "$LIVE" ] && exit 0

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=_reflex-log.sh
. "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && { reflex_log "subagent-bg-guard" "$INPUT" "live=${LIVE}" || true; }

REASON="Tienes tareas de fondo propias aún vivas: ${LIVE}. Haz TaskStop (task_id) de las que ya no necesites. Si necesitas el resultado, espera sobre su log con timeout (until grep -q '<marca de fin>' LOG) o reporta BLOCKED con el comando y la ruta del log. Después termina."
jq -n --arg r "$REASON" '{decision: "block", reason: $r}'
