#!/usr/bin/env bash
# Adaptador SubagentStart (A1, spec §5.2): stdin payload -> compose-inject -> additionalContext.
# Never-break: exit 0 SIEMPRE. Politica de profundidad v1 (spec §5.3): spawnDepth>1 => no inyectar;
# meta ausente/ilegible => inyectar (S5/S6: el meta puede no existir aun — el default es entregar).
set -uo pipefail

# Guard de stdin: evita colgarse si se invoca manualmente sin pipe (mismo patron
# defensivo que basic-memory-recall.sh). Never-break: sin input no hay nada que hacer.
[ -t 0 ] && exit 0
INPUT="$(cat)"
_d=${0%/*}; [ "$_d" = "$0" ] && _d=.
SCRIPT_DIR="$(cd "$_d" && pwd)"
PROJECTS_DIR="${REFLEX_PROJECTS_DIR:-$HOME/.claude/projects}"
# Cada proceso cuesta ~50 ms bajo Git Bash: un solo jq extrae los campos del
# payload y el perfil (NUL como separador), y el resto es builtin. Medido:
# ~2,1 s -> ~0,7 s por disparo. jq.exe emite CRLF: se quita con ${v%$'\r'}.
PROFILES_JSON="$SCRIPT_DIR/inject-profiles.json"
PROF_ARGS=(--argjson prof '[{}]')
[ -r "$PROFILES_JSON" ] && PROF_ARGS=(--slurpfile prof "$PROFILES_JSON")
TYPE=""; PERFIL=""; SID=""; AID=""; CWD=""
{
  IFS= read -r -d '' TYPE; IFS= read -r -d '' PERFIL; IFS= read -r -d '' SID
  IFS= read -r -d '' AID;  IFS= read -r -d '' CWD
} < <(jq -j "${PROF_ARGS[@]}" '
  (.agent_type // "" | tostring) as $t
  | $t, ($prof[0][$t] // $prof[0]._default // ""), .session_id, .agent_id, .cwd
  | (. // "" | tostring) + "\u0000"' <<<"$INPUT" 2>/dev/null)
TYPE=${TYPE%$'\r'}; PERFIL=${PERFIL%$'\r'}; SID=${SID%$'\r'}; AID=${AID%$'\r'}; CWD=${CWD%$'\r'}
if [ -z "$TYPE" ]; then
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "inject-abstained" "$INPUT" "sin agent_type" || true
  exit 0
fi
if [ -n "$SID" ] && [ -n "$AID" ]; then
  # shellcheck disable=SC2140 # glob `*` entre tramos entrecomillados: intencionado
  for m in "$PROJECTS_DIR"/*/"$SID"/subagents/"agent-${AID}.meta.json"; do
    [ -f "$m" ] || continue
    d="$(jq -r '.spawnDepth // 1' "$m" 2>/dev/null | tr -d '\r')" || d=1
    case "$d" in (*[!0-9]*) d=1 ;; esac
    if [ "$d" -gt 1 ]; then
      . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "inject-skipped-depth" "$INPUT" "type=$TYPE depth=$d" || true
      exit 0
    fi
  done
fi
KB_ARGS=()
[ -n "${REFLEX_INJECT_KB:-}" ] && KB_ARGS=(--kb "$REFLEX_INJECT_KB")
# #116: el cwd del payload resuelve el repo cuyas reglas duras se inyectan.
KB_ARGS+=(--cwd "${CWD:-$PWD}")
JSON=""
# Si mktemp falla (disco lleno, /tmp no escribible, etc.) el hook sigue
# inyectando igual que antes de F3.2: stderr de compose-inject.sh se pierde
# (mismo `2>/dev/null` que ya se usaba), pero nunca se cuelga ni deja de
# responder — never-break.
COMPOSE_ERR="$(mktemp)" || COMPOSE_ERR=""
# `${KB_ARGS[@]+"${KB_ARGS[@]}"}`, no `"${KB_ARGS[@]}"` a secas: con `set -u`,
# bash < 4.4 (el /bin/bash 3.2 de macOS) trata un array vacío como variable
# no definida y aborta la sustitución ⇒ el hook logueaba inject-failed siempre
# que REFLEX_INJECT_KB no estaba fijada, que es el caso normal en producción.
if BLOQUE="$(REFLEX_INJECT_PAYLOAD="$INPUT" "$SCRIPT_DIR/compose-inject.sh" --type "$TYPE" ${KB_ARGS[@]+"${KB_ARGS[@]}"} 2>"${COMPOSE_ERR:-/dev/null}")" && [ -n "$BLOQUE" ]; then
  # El here-string añade un único "\n" tras BLOQUE (que $(...) dejó sin saltos
  # finales): rtrimstr lo quita, el additionalContext queda igual que antes.
  JSON="$(jq -Rs '{hookSpecificOutput:{hookEventName:"SubagentStart", additionalContext:rtrimstr("\n")}}' <<<"$BLOQUE" 2>/dev/null)" || JSON=""
fi
if [ -n "$JSON" ]; then
  bytes_de() { local LC_ALL=C; bytes=${#1}; }  # BYTES, no caracteres (UTF-8 seguro)
  bytes_de "$BLOQUE"
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "inject-emitted" "$INPUT" "type=$TYPE perfil=$PERFIL bytes=$bytes" || true
  # F3.2: compose-inject.sh avisa "sin-contenido" por stderr cuando el
  # bloque no supera el tamaño de su propia cabecera — el caso medido de
  # `reducido` sin KB resoluble. inject-emitted YA se logueó arriba (el
  # contrato "el hook siempre entrega algo" no cambia); esto es aditivo.
  # Sin COMPOSE_ERR (mktemp falló arriba) no hay dónde haber capturado el
  # aviso: se omite el chequeo, no se inventa un inject-empty sin evidencia.
  if [ -n "$COMPOSE_ERR" ] && [[ "$(<"$COMPOSE_ERR")" == *sin-contenido* ]]; then
    . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "inject-empty" "$INPUT" "type=$TYPE perfil=$PERFIL bytes=$bytes" || true
  fi
  printf '%s' "$JSON"
else
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "inject-failed" "$INPUT" "type=$TYPE perfil=$PERFIL" || true
fi
rm -f "$COMPOSE_ERR"
exit 0
