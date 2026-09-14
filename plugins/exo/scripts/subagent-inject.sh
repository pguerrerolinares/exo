#!/usr/bin/env bash
# Adaptador SubagentStart (A1, spec §5.2): stdin payload -> compose-inject -> additionalContext.
# Never-break: exit 0 SIEMPRE. Politica de profundidad v1 (spec §5.3): spawnDepth>1 => no inyectar;
# meta ausente/ilegible => inyectar (S5/S6: el meta puede no existir aun — el default es entregar).
set -uo pipefail

# Guard de stdin: evita colgarse si se invoca manualmente sin pipe (mismo patron
# defensivo que basic-memory-recall.sh). Never-break: sin input no hay nada que hacer.
[ -t 0 ] && exit 0
INPUT="$(cat)"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECTS_DIR="${REFLEX_PROJECTS_DIR:-$HOME/.claude/projects}"
TYPE="$(printf '%s' "$INPUT" | jq -r '.agent_type // empty' 2>/dev/null)" || TYPE=""
if [ -z "$TYPE" ]; then
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "inject-abstained" "$INPUT" "sin agent_type" || true
  exit 0
fi
PERFIL="$(jq -r --arg t "$TYPE" '.[$t] // ._default' "$SCRIPT_DIR/inject-profiles.json" 2>/dev/null)" || PERFIL=""
SID="$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)" || SID=""
AID="$(printf '%s' "$INPUT" | jq -r '.agent_id // empty' 2>/dev/null)" || AID=""
if [ -n "$SID" ] && [ -n "$AID" ]; then
  # shellcheck disable=SC2140 # glob `*` entre tramos entrecomillados: intencionado
  for m in "$PROJECTS_DIR"/*/"$SID"/subagents/"agent-${AID}.meta.json"; do
    [ -f "$m" ] || continue
    d="$(jq -r '.spawnDepth // 1' "$m" 2>/dev/null)" || d=1
    case "$d" in (*[!0-9]*) d=1 ;; esac
    if [ "$d" -gt 1 ]; then
      . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "inject-skipped-depth" "$INPUT" "type=$TYPE depth=$d" || true
      exit 0
    fi
  done
fi
KB_ARGS=()
[ -n "${REFLEX_INJECT_KB:-}" ] && KB_ARGS=(--kb "$REFLEX_INJECT_KB")
JSON=""
# Si mktemp falla (disco lleno, /tmp no escribible, etc.) el hook sigue
# inyectando igual que antes de F3.2: stderr de compose-inject.sh se pierde
# (mismo `2>/dev/null` que ya se usaba), pero nunca se cuelga ni deja de
# responder — never-break. Patrón de `recall-inject.sh` (CONFIG_ERR_TMP).
COMPOSE_ERR="$(mktemp)" || COMPOSE_ERR=""
if BLOQUE="$("$SCRIPT_DIR/compose-inject.sh" --type "$TYPE" "${KB_ARGS[@]}" 2>"${COMPOSE_ERR:-/dev/null}")" && [ -n "$BLOQUE" ]; then
  JSON="$(printf '%s' "$BLOQUE" | jq -Rs '{hookSpecificOutput:{hookEventName:"SubagentStart", additionalContext:.}}' 2>/dev/null)" || JSON=""
fi
if [ -n "$JSON" ]; then
  bytes="$(printf '%s' "$BLOQUE" | wc -c)"
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "inject-emitted" "$INPUT" "type=$TYPE perfil=$PERFIL bytes=$bytes" || true
  # F3.2: compose-inject.sh avisa "sin-contenido" por stderr cuando el
  # bloque no supera el tamaño de su propia cabecera — el caso medido de
  # `reducido` sin KB resoluble. inject-emitted YA se logueó arriba (el
  # contrato "el hook siempre entrega algo" no cambia); esto es aditivo.
  # Sin COMPOSE_ERR (mktemp falló arriba) no hay dónde haber capturado el
  # aviso: se omite el chequeo, no se inventa un inject-empty sin evidencia.
  if [ -n "$COMPOSE_ERR" ] && grep -q 'sin-contenido' "$COMPOSE_ERR" 2>/dev/null; then
    . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "inject-empty" "$INPUT" "type=$TYPE perfil=$PERFIL bytes=$bytes" || true
  fi
  printf '%s' "$JSON"
else
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "inject-failed" "$INPUT" "type=$TYPE perfil=$PERFIL" || true
fi
rm -f "$COMPOSE_ERR"
exit 0
