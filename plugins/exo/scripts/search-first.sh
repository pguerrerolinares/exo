#!/usr/bin/env bash
# PreToolUse (matcher: ^(Agent|Task|Edit|Write|NotebookEdit)$ -- regex
# exacta en hooks/hooks.json): reflejo "search-first". Warn-only, NUNCA
# bloquea (exit 0 siempre). Recuerda buscar en la KB (`exo search`/`exo
# targets`) antes del primer trabajo sustantivo de la sesion -- incidente de
# origen: Task 5 de la campana I (el agente principal despacho sin buscar,
# guiado solo por una regla en prosa que se trunca en silencio). Diseno
# completo: docs/superpowers/specs/2026-09-23-search-first-design.md.
#
# Abstencion:
#  (a) SOLO dispara en el PADRE -> dentro de un subagente (`agent_id` con
#      VALOR, no vacio) exit 0 SIN tocar el sentinel -- es por-sesion,
#      compartido con el padre (mismo patron que
#      clean-orchestrator-research.sh).
#  (b) Como mucho UNA vez por sesion (sentinel en
#      "${SEARCH_FIRST_SENTINEL_DIR:-/tmp}/claude-search-first-<session_id>";
#      la variable de entorno solo existe para aislar los tests de /tmp
#      real).
#  (c) Si el sensor no sabe -- sin `jq`, sin `transcript_path`, con el
#      fichero ausente/ilegible, o si el `jq` de deteccion falla -- calla
#      (ni avisa ni cuenta como "ok"): `reflex_log "search-first-skip"` con
#      el motivo, y CREA igualmente el sentinel (como mucho un intento por
#      sesion, ok o skip).
#
# Camino rapido (desde la 2a llamada de la sesion en adelante, y para
# distinguir subagente): bash puro, SIN jq -- session_id/agent_id se leen
# con regex de bash sobre el JSON crudo de entrada, tolerante a espacios
# tras ":" segun el serializador (el harness real usa JSON.stringify, que
# no los mete -- mismo supuesto documentado en git-c-bash.sh).
#
# Capa TRIGGER / clase event-watching del proyecto cerebro+reflejos.
set -uo pipefail

INPUT="$(cat)"

# --- Camino rapido: session_id y agent_id sin jq ----------------------------
SID=""
if [[ "$INPUT" =~ \"session_id\"[[:space:]]*:[[:space:]]*\"([^\"]*)\" ]]; then
  SID="${BASH_REMATCH[1]}"
fi

# agent_id CON VALOR (no vacio) -> subagente: abstencion total. El campo
# viene AUSENTE en el padre (convencion del harness y de los demas
# reflejos: `.agent_id // empty`), asi que exigir `[^\"]+` (uno o mas
# caracteres) es exactamente "presente y no vacio".
if [[ "$INPUT" =~ \"agent_id\"[[:space:]]*:[[:space:]]*\"[^\"]+\" ]]; then
  exit 0
fi

SENTINEL_DIR="${SEARCH_FIRST_SENTINEL_DIR:-/tmp}"
SENTINEL="${SENTINEL_DIR}/claude-search-first-${SID:-nosession}"
[ -f "$SENTINEL" ] && exit 0

# --- Primera vez en la sesion (para el padre) -------------------------------
LOG_FILE="${REFLEX_LOG_FILE:-$HOME/.claude/reflex-log.jsonl}"

# Unica escritura de log que NO pasa por _reflex-log.sh/jq: si jq no esta en
# el PATH, _reflex-log.sh tampoco puede correr (el mismo pipe a `jq -c` que
# usa para el resto de reflejos). El payload es fijo, sin contenido de
# usuario -> seguro construirlo a mano, sin escapado.
log_skip_sin_jq() {
  local ts
  ts="$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null)" || ts=""
  printf '{"ts":"%s","reflex":"search-first-skip","session_id":"%s","agent_id":"","agent_type":"","tool":"","payload":"motivo=sin-jq"}\n' \
    "$ts" "$SID" >> "$LOG_FILE" 2>/dev/null || true
}

if ! command -v jq >/dev/null 2>&1; then
  log_skip_sin_jq
  touch "$SENTINEL" 2>/dev/null
  exit 0
fi

# A partir de aqui jq esta disponible: un solo intento por sesion, ok o
# skip -> el sentinel se crea YA, antes de saber el veredicto.
touch "$SENTINEL" 2>/dev/null

TRANSCRIPT="$(printf '%s' "$INPUT" | jq -r '.transcript_path // empty' 2>/dev/null)"

if [ -z "$TRANSCRIPT" ]; then
  . "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "search-first-skip" "$INPUT" "motivo=sin-transcript_path" || true
  exit 0
fi
if [ ! -f "$TRANSCRIPT" ] || [ ! -r "$TRANSCRIPT" ]; then
  . "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "search-first-skip" "$INPUT" "motivo=transcript-ilegible" || true
  exit 0
fi

# --- jq de deteccion sobre la transcripcion ---------------------------------
# `-R` (raw input: una linea = una cadena) + `fromjson?` tolera lineas
# no-JSON SIN abortar el resto del fichero -- una transcripcion JSONL de
# ~2 MB puede traer alguna linea truncada/corrupta; alimentar el fichero
# ENTERO a `jq -c .` sin `-R` revienta con "Invalid JSON text" ante el
# primer token invalido y pierde TODAS las lineas buenas detras. `-n` +
# `inputs` (en vez de una `.` de nivel superior implicita) deja usar `$re`
# una sola vez y devolver un UNICO booleano agregado -- asi `-e` decide el
# exit code sobre ESE booleano, no sobre "el ultimo valor de la ultima
# linea leida".
#
# Filtro: solo bloques `tool_use` de mensajes del asistente
# (`select(.type=="assistant") | .message.content[]? | select(.type==
# "tool_use")`), positivo si `name=="Bash"` y `input.command` casa con la
# regex de la spec. El pie de `recall-inject.sh` escribe "exo search --type
# hybrid" como TEXTO en cada prompt (`FOOTER=`, recall-inject.sh:313) -- un
# grep a pelo sobre la transcripcion daria siempre positivo; mirar solo
# `tool_use` de tipo Bash lo evita.
RE='(^|[;&|[:space:]])exo(\.exe)?[[:space:]]+(search|targets)([[:space:]]|$)'
jq -R -n -e --arg re "$RE" '
  [inputs
   | fromjson?
   | select(.type=="assistant")
   | .message.content[]?
   | select(.type=="tool_use" and .name=="Bash")
   | (.input.command // "")
   | select(test($re))
  ] | length > 0
' "$TRANSCRIPT" >/dev/null 2>&1
EC=$?

case "$EC" in
  0)
    . "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "search-first-ok" "$INPUT" "" || true
    ;;
  1)
    . "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "search-first" "$INPUT" "" || true
    MSG='Reflejo search-first: primer trabajo sustantivo de la sesión sin `exo search`/`exo targets` previo. Si el tema puede tener historia en la KB, busca antes (`exo search --type hybrid "<tema>"`); si no, dilo en una línea y sigue.'
    printf '%s' "$MSG" | jq -Rs '{hookSpecificOutput:{hookEventName:"PreToolUse",additionalContext:.}}'
    ;;
  *)
    . "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "search-first-skip" "$INPUT" "motivo=jq-deteccion-fallo-ec${EC}" || true
    ;;
esac

exit 0
