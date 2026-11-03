#!/usr/bin/env bash
# Wrapper del oráculo de paridad (spec 2026-11-02-exo-hook-multi-harness §6.3):
# hace que `exo hook claude` se comporte, bajo test-bash-guards.sh y
# test-search-first.sh, como si Claude hubiera ido emitiendo PostToolUse /
# PostToolUseFailure por cada tool_use del transcript fixture.
#
# Entorno: EXO_BIN (binario; defecto engine/target/debug/exo) y
# EXO_PARIDAD_BASE (directorio base de estado, obligatorio: lo crea
# paridad-hook.sh con mktemp -d).
set -uo pipefail

AQUI="$(cd "$(dirname "$0")" && pwd)"
EXO_BIN="${EXO_BIN:-$AQUI/../target/debug/exo}"
if [ -z "${EXO_PARIDAD_BASE:-}" ]; then
  echo "hook-wrapper-claude: falta EXO_PARIDAD_BASE" >&2
  exit 2
fi

PAYLOAD="$(cat)"
campo() { printf '%s' "$PAYLOAD" | jq -r --arg k "$1" '.[$k] // "" | if type=="string" then . else "" end' 2>/dev/null || true; }
sha1() { { sha1sum 2>/dev/null || shasum 2>/dev/null; } | cut -d' ' -f1; }
limpio() { printf '%s' "$1" | tr -c 'A-Za-z0-9_-' '_'; }

SID="$(campo session_id)"
TRANSCRIPT="$(campo transcript_path)"
AGENT="$(campo agent_id)"
CWD="$(campo cwd)"
EVENTO="$(campo hook_event_name)"
[ -n "$EVENTO" ] || EVENTO=PreToolUse

# Los casos comparten session_id ("test-sid"): el aislamiento va por transcript.
if [ -n "$TRANSCRIPT" ]; then
  CLAVE="$(printf '%s' "$TRANSCRIPT" | sha1)"
else
  CLAVE="none-$(limpio "$SID")"
fi
export EXO_STATE_DIR="$EXO_PARIDAD_BASE/${CLAVE:-nosha}"
mkdir -p "$EXO_STATE_DIR" 2>/dev/null || true

# Reinyecta solo los tool_use aún no vistos (la marca cuenta los ya enviados
# por sesión), para que repetir la invocación no duplique el journal.
if [ -n "$TRANSCRIPT" ] && [ -r "$TRANSCRIPT" ]; then
  MARCA="$EXO_STATE_DIR/.inyectados-$(limpio "$SID")"
  HECHOS="$(cat "$MARCA" 2>/dev/null || echo 0)"
  TOTAL=0
  while IFS= read -r ev; do
    TOTAL=$((TOTAL+1))
    [ "$TOTAL" -gt "$HECHOS" ] || continue
    nombre="$(printf '%s' "$ev" | jq -r .hook_event_name)"
    printf '%s' "$ev" | "$EXO_BIN" hook claude "$nombre" >/dev/null 2>&1 || true
  done < <(jq -nRc --arg sid "$SID" --arg agent "$AGENT" --arg cwd "$CWD" '
    [inputs | fromjson? | select(type=="object")] as $l
    | ([$l[] | .message.content? | arrays | .[]
        | select(type=="object" and .type=="tool_result")
        | {key:(.tool_use_id // "" | tostring), value:(.is_error == true)}] | from_entries) as $err
    | $l[] | .message.content? | arrays | .[]
    | select(type=="object" and .type=="tool_use")
    | {session_id:$sid, cwd:$cwd,
       hook_event_name:(if $err[(.id // "" | tostring)] then "PostToolUseFailure" else "PostToolUse" end),
       tool_name:.name, tool_input:.input}
      + (if $agent == "" then {} else {agent_id:$agent} end)
  ' "$TRANSCRIPT" 2>/dev/null)
  printf '%s' "$TOTAL" > "$MARCA" 2>/dev/null || true
fi

printf '%s' "$PAYLOAD" | exec "$EXO_BIN" hook claude "$EVENTO"
