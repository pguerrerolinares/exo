#!/usr/bin/env bash
# PreToolUse (matcher: WebSearch, WebFetch y la navegación de los MCP de
# navegador — claude-in-chrome y playwright; regex exacta en hooks/hooks.json):
# reflejo "orquestador limpio".
# Warn-only, NUNCA bloquea (exit 0 siempre). Recuerda delegar la investigacion
# web a un subagente (Explore / research con modelo barato) para no ensuciar el
# contexto del PADRE (context-rot: mas contexto = peor rendimiento).
#
# Abstencion: (a) SOLO dispara en el PADRE -> dentro de un subagente la web-research
# YA es el patron deseado, avisar ahi es falso positivo (verificado 2026-06-26: el
# input trae `agent_id` no vacio sii corre en un subagente; session_id/transcript_path
# son COMPARTIDos con el padre y no discriminan). (b) Como mucho UNA vez por sesion
# (sentinel). Reframea al primer fallo y calla el resto -> sin nag, sin auto-ensuciar.
# Capa TRIGGER / clase event-watching del proyecto cerebro+reflejos.
set -uo pipefail

INPUT="$(cat)"
command -v jq >/dev/null 2>&1 || exit 0

SESSION_ID="$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)" || SESSION_ID=""
TOOL="$(printf '%s' "$INPUT" | jq -r '.tool_name // empty' 2>/dev/null)" || TOOL=""
AGENT_ID="$(printf '%s' "$INPUT" | jq -r '.agent_id // empty' 2>/dev/null)" || AGENT_ID=""

# Dentro de un subagente (agent_id no vacio): la investigacion web YA esta delegada,
# que es el patron deseado. Abstencion total: ni avisa (seria FP) ni consume el sentinel
# (que es por-sesion y por tanto compartido padre<->hijos).
[ -n "$AGENT_ID" ] && exit 0

# Navegar a la app local que estás probando (dev server, fichero) no es
# investigación web: abstención, y sin consumir el sentinel, para que la
# primera búsqueda web real de la sesión siga avisando. Solo mira `url`:
# WebSearch trae `query` y get_page_text no trae url, así que esos siguen
# el camino normal.
URL="$(printf '%s' "$INPUT" | jq -r '.tool_input.url // empty' 2>/dev/null)" || URL=""

# `mcp__claude-in-chrome__navigate` con url:"back"/"forward" es navegación
# por historial del propio navegador, no una URL: no es investigación web.
case "$URL" in
  back|forward) exit 0 ;;
esac

# Normaliza antes de comparar: minúsculas (con `tr`, no `${var,,}` — bash 3.2
# de macOS no lo tiene) y esquema http(s) opcional fuera, porque
# `mcp__claude-in-chrome__navigate` admite URL sin protocolo
# (p.ej. "localhost:3000", "127.0.0.1:8080").
URL_NORM="$(printf '%s' "$URL" | tr '[:upper:]' '[:lower:]')"
case "$URL_NORM" in
  http://*) URL_NORM="${URL_NORM#http://}" ;;
  https://*) URL_NORM="${URL_NORM#https://}" ;;
esac
case "$URL_NORM" in
  localhost|localhost[:/]*) exit 0 ;;
  127.0.0.1|127.0.0.1[:/]*) exit 0 ;;
  \[::1\]*|0.0.0.0*|file:*) exit 0 ;;
esac

SENTINEL="/tmp/claude-clean-orch-${SESSION_ID:-nosession}"

# Ya avisado en esta sesion -> calla (abstencion).
[ -f "$SENTINEL" ] && exit 0
touch "$SENTINEL" 2>/dev/null

MSG="⚠️ Reflejo orquestador limpio: estas investigando en web (${TOOL:-WebSearch/WebFetch}) desde el contexto del PADRE. Salvo consulta puntual de una sola llamada, delega a un subagente (Explore para busquedas/lecturas; research-agent con modelo barato) y quedate con la CONCLUSION, no con las fuentes. Context-rot: mas contexto en el padre = peor rendimiento. (Aviso 1x/sesion.)"

# log del disparo (best-effort, nunca rompe el warn-only)
. "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "clean-orchestrator" "$INPUT" "${TOOL}: $(printf '%s' "$INPUT" | jq -r '.tool_input.query // .tool_input.url // empty' 2>/dev/null)" || true

printf '%s' "$MSG" | jq -Rs '{hookSpecificOutput:{hookEventName:"PreToolUse",additionalContext:.}}'

exit 0
