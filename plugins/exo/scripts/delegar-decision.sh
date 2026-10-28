#!/usr/bin/env bash
# Stop hook: el padre no cierra turno con propuesta + petición de visto bueno sin delegar.
#
# Por qué: medido sobre 99 sesiones, ~1 de cada 4 cierres del padre pide permiso a Paul
# («¿te parece?») para algo que un consultor o un ejecutor podía resolver antes. Spec:
# docs/superpowers/specs/2026-10-28-delegar-decision-design.md.
#
# Qué hace: delega la decisión en delegar-decision.py (detector sobre el último segmento del
# transcript; el último texto es last_assistant_message, porque en el Stop el transcript aún no
# tiene la línea final: ver R23). Si dispara y no se
# ha bloqueado ya para ese mensaje humano, emite `decision: block`.
#
# Abstención (exit 0, sin salida): subagente (`agent_id`), `stop_hook_active`, sin jq o
# python3, transcript ilegible, excepción del detector (R16), sentinel ya consumido.
# El sentinel es best-effort (R7): si no se puede escribir, el siguiente Stop vuelve a bloquear.
set -uo pipefail

INPUT="$(cat)"
[[ "$INPUT" =~ \"agent_id\"[[:space:]]*:[[:space:]]*\"[^\"]+\" ]] && exit 0
command -v jq >/dev/null 2>&1 || exit 0
command -v python3 >/dev/null 2>&1 || exit 0

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

ROW="$(printf '%s' "$INPUT" | jq -r '[(.stop_hook_active // false), (.transcript_path // ""), (.session_id // "")]
  | map(tostring) | join("\u001f")' 2>/dev/null)" || exit 0
ROW="${ROW%$'\r'}"
IFS=$'\x1f' read -r ACTIVE TRANSCRIPT SID <<< "$ROW"

[ "$ACTIVE" = "true" ] && exit 0
[ -n "$TRANSCRIPT" ] && [ -r "$TRANSCRIPT" ] || exit 0

RES="$(printf '%s' "$INPUT" | python3 -I "$SCRIPT_DIR/delegar-decision.py" "$TRANSCRIPT" --stdin 2>/dev/null)" || exit 0
IFS=$'\x1f' read -r DISPARA KIND PATRON UUID FUERA TAIL < <(printf '%s' "$RES" | jq -r '
  [(.dispara // false), (.kind // ""), (.patron // ""), (.uuid // ""), (if .reserva_fuera then 1 else 0 end),
   ((.tail // "")[0:200] | gsub("[\n\r\u001f]"; " "))] | map(tostring) | join("\u001f")' 2>/dev/null) || exit 0

# shellcheck source=_reflex-log.sh
. "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null || true
log() { command -v reflex_log >/dev/null 2>&1 && { reflex_log "$1" "$INPUT" "$2" || true; }; }

if [ "$DISPARA" != "true" ]; then
  case "$KIND" in
    banda) log "delegar-decision:banda" "kind=banda" ;;
    p1_log|error) log "delegar-decision" "kind=${KIND}" ;;
  esac
  exit 0
fi

SAFE_SID="$(printf '%s' "$SID" | tr -c 'A-Za-z0-9_-' '_')"
SENTINEL="$HOME/.cache/exo/delegar-decision/${SAFE_SID:-sin-sesion}"
if [ -n "$UUID" ]; then
  if [ "$(cat "$SENTINEL" 2>/dev/null)" = "$UUID" ]; then
    log "delegar-decision" "kind=tope patron=${PATRON}"
    exit 0
  fi
  { mkdir -p "$(dirname "$SENTINEL")" && printf '%s' "$UUID" > "$SENTINEL"; } 2>/dev/null || true
fi

log "delegar-decision" "kind=${KIND} patron=${PATRON} reserva_fuera=${FUERA} tail=${TAIL}"

REASON='Cerraste con propuesta + petición de visto bueno sin delegar. Lanza ahora el consultor (skills/brainstorm/consultor-brief.md) o el ejecutor y cierra con su resultado; o, si la pregunta es de Paul, reescribe el cierre empezando por [reserva: externo|irreversible|re-scope|hecho-de-Paul|pedido].'
jq -n --arg r "$REASON" '{decision: "block", reason: $r}'
