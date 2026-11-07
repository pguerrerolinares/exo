#!/usr/bin/env bash
# PreToolUse (matcher: ^SubagentHandback$): gate «RED como artefacto» (#136, #271).
# Un `exo:executor` que entrega sin ningún testigo `red exit=[1-9]` propio en el
# journal de sesión no corrió su RED con `red-witness` (o no hubo RED). Empieza
# LOG-ONLY: loguea `red-gate` (falta) o `red-gate-ok` (pasa), nunca bloquea; la
# promoción a deny (marker por agent_id, la 2ª llamada pasa) llega tras ≥10
# executors con FP ≤20 % medido en reflex-log.
#
# De dónde sale el testigo: `red-witness` imprime `exo-red-witness exit=N` como
# última línea de stderr; `exo hook claude PostToolUse[Failure]` (engine ≥ 0.17)
# la ve en la salida y anota en `<state>/sessions/claude-<sid>.jsonl` una marca
# `{"kind":"mark","code":"red exit=N","agent_id":…}`. El subagente no tiene su
# agent_id en el entorno; el hook sí. Antes se miraba `<toplevel>/.superpowers/sdd/red.log`,
# pero ese fichero solo crece, es del repo y no del agente, y el `cwd` del payload
# es el del padre, no el del worktree: un RED viejo aprobaba a todos (#271).
#
# Por qué PreToolUse y no SubagentStop: con SubagentHandback el report llega
# como `tool_input.message`; `last_assistant_message` de SubagentStop es el
# texto de cierre y dispara cuando el orquestador ya tiene el report
# (hooks.md, SubagentStop).
#
# Abstención: sin jq, `agent_type` ≠ exo:executor, sin `agent_id`, o mensaje con
# BLOCKED|NEEDS_CONTEXT|«RED: n/a».
set -uo pipefail
RED_TESTIGO_DESDE="0.17.0"   # primer engine que anota `red exit=N` en el journal
INPUT="$(cat)"
command -v jq >/dev/null 2>&1 || exit 0
ROW="$(printf '%s' "$INPUT" | jq -r '[(.agent_type // ""), (.agent_id // ""), (.session_id // ""), ((.tool_input.message // "") | gsub("[\n\r]"; " "))] | join("\u001f")' 2>/dev/null)" || exit 0
ROW="${ROW%$'\r'}"
IFS=$'\x1f' read -r ATYPE AID SID MSG <<< "$ROW"
[ "$ATYPE" = "exo:executor" ] || exit 0
[ -n "$AID" ] || exit 0
[[ "$MSG" =~ BLOCKED|NEEDS_CONTEXT|RED:[[:space:]]*n/a ]] && exit 0

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=_state-dir.sh
. "$SCRIPT_DIR/_state-dir.sh" 2>/dev/null || true
# Mismo saneado que el engine (journal::ruta): lo que no es [A-Za-z0-9_-] pasa a `_`.
SAFE_SID="$(printf '%s' "$SID" | tr -c 'A-Za-z0-9_-' '_')"
JOURNAL="${EXO_STATE_DIR:-$HOME/.exo/state}/sessions/claude-${SAFE_SID}.jsonl"

# Exits de los testigos de este agente, uno por línea (vacío si no hay ninguno).
# shellcheck disable=SC2016 # $a es variable de jq (--arg), no de bash
FILTRO='select(.kind == "mark" and .agent_id == $a) | .code | select(startswith("red exit=")) | ltrimstr("red exit=")'
EXITS="$(jq -r --arg a "$AID" "$FILTRO" "$JOURNAL" 2>/dev/null | tr -d '\r')"

# shellcheck source=_reflex-log.sh
. "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null || exit 0
if printf '%s\n' "$EXITS" | grep -qE '^[1-9][0-9]*$'; then
  reflex_log "red-gate-ok" "$INPUT" "testigos=$(printf '%s' "$EXITS" | grep -c .)" || true
elif [ -n "$EXITS" ]; then
  reflex_log "red-gate" "$INPUT" "solo-exit-0 journal=${JOURNAL}" || true
else
  # Un engine < 0.17 no anota testigos: sin esto cada entrega sería un falso
  # `sin-testigo`. Solo se mira aquí (un spawn por entrega sin testigo), y no
  # sube ENGINE_MIN: el gate es log-only y no merece degradar SessionStart.
  REASON="sin-testigo"
  # shellcheck source=_engine-version.sh
  if . "$SCRIPT_DIR/_engine-version.sh" 2>/dev/null; then
    V="$(exo_version_de exo)"
    [ -n "$V" ] && semver_lt "$V" "$RED_TESTIGO_DESDE" && REASON="engine-stale engine=${V}"
  fi
  reflex_log "red-gate" "$INPUT" "${REASON} journal=${JOURNAL}" || true
fi
exit 0
