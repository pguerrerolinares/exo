#!/usr/bin/env bash
# smoke-codex.sh — smoke e2e de hooks de Codex instalados como plugin (spec
# 2026-11-02-exo-hook-multi-harness §6.2.5). Fuera de CI: necesita la cuenta de
# Codex del dueño. Se corre al empezar B y en cada actualización de Codex.
#
# Mide las condiciones de invalidación de la spec §7:
#   #1 PostToolUse Bash trae tool_input.command y se dispara con exit≠0.
#   #4 Con el hook instalado como PLUGIN (no como hooks.json suelto), un deny de
#      PreToolUse impide de verdad el Bash.
# Además comprueba los aliases de matcher (Write|Edit→apply_patch) y regenera el
# golden saneado en engine/tests/fixtures/hook/codex/codex-<versión>-probe.jsonl.
#
# Aislamiento: CODEX_HOME temporal con auth.json enlazado (no copiado); no toca
# ~/.codex/config.toml. Lanza `codex exec --dangerously-bypass-hook-trust`: los
# hooks que confía son solo los de volcado que genera este script.
#
# Uso: scripts/smoke-codex.sh [--keep]   (--keep conserva el directorio temporal)
set -uo pipefail

KEEP=0
[ "${1:-}" = "--keep" ] && KEEP=1

RAIZ="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)" || exit 1
command -v codex >/dev/null || { echo "smoke-codex: falta codex en PATH" >&2; exit 1; }
command -v jq >/dev/null || { echo "smoke-codex: falta jq en PATH" >&2; exit 1; }
AUTH="${CODEX_HOME:-$HOME/.codex}/auth.json"
[ -f "$AUTH" ] || { echo "smoke-codex: no hay $AUTH (haz codex login)" >&2; exit 1; }

VER="$(codex --version | awk '{print $NF}')"
T="$(mktemp -d)"
if [ "$KEEP" -eq 0 ]; then trap 'rm -rf "$T"' EXIT; else echo "smoke-codex: dir temporal $T"; fi

export CODEX_HOME="$T/home"
MKT="$T/mkt"
PLUG="$MKT/plugins/exo-smoke"
WORK="$T/work"
EV="$T/ev.jsonl"
ALIAS="$T/alias.log"
mkdir -p "$CODEX_HOME" "$PLUG/.codex-plugin" "$PLUG/hooks" "$MKT/.claude-plugin" "$WORK"
ln -s "$AUTH" "$CODEX_HOME/auth.json"
: > "$EV"; : > "$ALIAS"

# --- plugin de volcado ---------------------------------------------------------
cat > "$PLUG/dump.sh" <<EOF
#!/bin/sh
# Una línea por evento: el payload crudo (ya es una línea JSON).
cat >> "$EV"; printf '\n' >> "$EV"
EOF
cat > "$PLUG/alias.sh" <<EOF
#!/bin/sh
cat > /dev/null; printf '%s\n' "\$1" >> "$ALIAS"
EOF
cat > "$PLUG/deny.sh" <<'EOF'
#!/bin/sh
# Deniega cualquier Bash cuyo comando contenga el centinela.
if grep -q EXO_SMOKE_DENY; then
  printf '%s\n' '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"exo-smoke: denegado por el hook"}}'
fi
exit 0
EOF
chmod +x "$PLUG/dump.sh" "$PLUG/alias.sh" "$PLUG/deny.sh"

dump() { printf '{"type":"command","command":"sh \\"%s/dump.sh\\"","timeout":5}' "$PLUG"; }
cat > "$PLUG/hooks/codex.json" <<EOF
{"hooks":{
  "SessionStart":[{"hooks":[$(dump)],"additionalContextLimit":2500}],
  "UserPromptSubmit":[{"hooks":[$(dump)]}],
  "PreToolUse":[
    {"hooks":[$(dump)]},
    {"matcher":"^Bash\$","hooks":[{"type":"command","command":"sh \\"$PLUG/deny.sh\\"","timeout":5}]},
    {"matcher":"^(Write|Edit)\$","hooks":[{"type":"command","command":"sh \\"$PLUG/alias.sh\\" pre-write-edit","timeout":5}]},
    {"matcher":"^Agent\$","hooks":[{"type":"command","command":"sh \\"$PLUG/alias.sh\\" pre-agent","timeout":5}]},
    {"matcher":"spawn_agent","hooks":[{"type":"command","command":"sh \\"$PLUG/alias.sh\\" pre-spawn","timeout":5}]}
  ],
  "PostToolUse":[{"hooks":[$(dump)]}],
  "PreCompact":[{"hooks":[$(dump)]}],
  "PostCompact":[{"hooks":[$(dump)]}],
  "Stop":[{"hooks":[$(dump)]}],
  "SubagentStart":[{"hooks":[$(dump)]}],
  "SubagentStop":[{"hooks":[$(dump)]}]
}}
EOF
jq -e . "$PLUG/hooks/codex.json" >/dev/null || { echo "smoke-codex: codex.json generado no es JSON" >&2; exit 1; }

cat > "$PLUG/.codex-plugin/plugin.json" <<'EOF'
{"name":"exo-smoke","version":"0.0.1","description":"smoke de hooks de exo","hooks":"./hooks/codex.json"}
EOF
cat > "$MKT/.claude-plugin/marketplace.json" <<'EOF'
{"name":"exo-smoke-mkt","owner":{"name":"exo"},"plugins":[{"name":"exo-smoke","source":"./plugins/exo-smoke","version":"0.0.1","description":"smoke"}]}
EOF

cat > "$CODEX_HOME/config.toml" <<EOF
[projects."$WORK"]
trust_level = "trusted"
EOF

# --- instalación real como plugin ---------------------------------------------
codex plugin marketplace add "$MKT" || { echo "smoke-codex: falló marketplace add" >&2; exit 1; }
codex plugin add exo-smoke@exo-smoke-mkt || { echo "smoke-codex: falló plugin add" >&2; exit 1; }

PROMPT="Ejecuta estos pasos en orden, cada uno con una llamada de shell separada y sin combinarlos: \
1) exo search --help | head -3  \
2) false  \
3) sh -c 'exit 3'  \
4) crea el fichero x.txt con el contenido hola usando apply_patch  \
5) touch denied.marker && echo EXO_SMOKE_DENY  (si se deniega, no lo reintentes de otra forma)  \
6) lanza un subagente que ejecute 'echo hijo' y espera su respuesta. \
Luego responde 'listo'."

# El límite bajo fuerza una compactación automática en el primer exec; el resume
# produce el segundo UserPromptSubmit y, con suerte, SessionStart source=compact.
( cd "$WORK" && codex exec --skip-git-repo-check --dangerously-bypass-approvals-and-sandbox \
    --dangerously-bypass-hook-trust -c model_auto_compact_token_limit=3000 "$PROMPT" ) > "$T/exec.log" 2>&1
RC=$?
( cd "$WORK" && codex exec resume --last --skip-git-repo-check --dangerously-bypass-approvals-and-sandbox \
    --dangerously-bypass-hook-trust "di ok" ) > "$T/resume.log" 2>&1
RC2=$?

# --- aserciones ----------------------------------------------------------------
fallos=0
ok()   { echo "OK    $1"; }
# shellcheck disable=SC2329 # invocada vía chk
mal()  { echo "FALLO $1"; fallos=$((fallos+1)); }
info() { echo "INFO  $1"; }
# chk <nivel-si-falla: mal|info> <texto-ok> <texto-falla> <cmd...>
chk() {
  local nivel="$1" si="$2" no="$3"; shift 3
  if "$@"; then ok "$si"; else "$nivel" "$no"; fi
}
# shellcheck disable=SC2329 # invocada vía chk
has() { jq -e -s "$1" "$EV" >/dev/null 2>&1; }
# shellcheck disable=SC2329 # invocada vía chk
no_has() { ! has "$1"; }

chk mal "codex exec rc=0" "codex exec rc=$RC (ver $T/exec.log con --keep)" test "$RC" -eq 0
chk mal "codex exec resume rc=0" "codex exec resume rc=$RC2 (ver $T/resume.log con --keep)" test "$RC2" -eq 0
sed -i '/^$/d' "$EV"
n="$(wc -l < "$EV")"
# Codex parsea con deny_unknown_fields: si additionalContextLimit estuviera mal, no cargaría ningún hook.
chk mal "hooks cargados con additionalContextLimit presente: $n eventos" "cero eventos: Codex no cargó los hooks (¿additionalContextLimit rechazado?)" test "$n" -gt 0
for ev in SessionStart UserPromptSubmit PreToolUse PostToolUse Stop; do
  chk mal "evento $ev" "falta evento $ev" has "any(.[]; .hook_event_name==\"$ev\")"
done
chk mal "evento PostCompact" "falta PostCompact (¿no se compactó?)" has 'any(.[]; .hook_event_name=="PostCompact")'
chk mal "dos UserPromptSubmit (exec + resume)" "no hay dos UserPromptSubmit" has '[.[]|select(.hook_event_name=="UserPromptSubmit")]|length>=2'
chk info "SessionStart con source=compact" "ningún SessionStart con source=compact" has 'any(.[]; .hook_event_name=="SessionStart" and .source=="compact")'
# Condición #1
chk mal "#1 PostToolUse Bash con exit≠0 (false) trae command" "#1 sin PostToolUse para 'false'" \
  has 'any(.[]; .hook_event_name=="PostToolUse" and .tool_name=="Bash" and (.tool_input.command|test("^false")))'
chk mal "#1 PostToolUse Bash con exit 3 trae command" "#1 sin PostToolUse para 'exit 3'" \
  has 'any(.[]; .hook_event_name=="PostToolUse" and .tool_name=="Bash" and (.tool_input.command|test("exit 3")))'
# Condición #4
chk mal "#4 el modelo intentó el comando centinela" "#4 no se intentó el centinela: el deny no se ejercitó" \
  has 'any(.[]; .hook_event_name=="PreToolUse" and .tool_name=="Bash" and (.tool_input.command|test("EXO_SMOKE_DENY")))'
chk mal "#4 deny de PreToolUse impidió el Bash" "#4 denied.marker existe: el deny NO impidió el Bash" \
  test ! -e "$WORK/denied.marker"
chk mal "#4 sin PostToolUse del comando denegado" "#4 hubo PostToolUse del comando denegado" \
  no_has 'any(.[]; .hook_event_name=="PostToolUse" and ((.tool_input.command // "")|test("EXO_SMOKE_DENY")))'
# Aliases de matcher
chk mal "matcher Write|Edit casa apply_patch" "matcher Write|Edit no casó apply_patch" grep -qx pre-write-edit "$ALIAS"
# 0.166.0: la tool es `collaborationspawn_agent` y el alias Agent NO la casa.
chk info "matcher Agent casa el spawn de subagente" "matcher Agent no casó el spawn" grep -qx pre-agent "$ALIAS"
chk info "matcher spawn_agent casa el spawn" "matcher spawn_agent no casó (¿no lanzó subagente?)" grep -qx pre-spawn "$ALIAS"
# Subagentes (informativo: depende de que el modelo obedezca el paso 6)
chk info "hay eventos con agent_id" "ningún evento con agent_id" has 'any(.[]; .agent_id != null)'
info "tools vistas: $(jq -r -s '[.[].tool_name // empty] | unique | join(",")' "$EV")"
info "eventos vistos: $(jq -r -s '[.[].hook_event_name] | unique | join(",")' "$EV")"

# --- golden saneado ------------------------------------------------------------
DEST="$RAIZ/engine/tests/fixtures/hook/codex/codex-$VER-probe.jsonl"
mkdir -p "$(dirname "$DEST")"
A_HOME="$HOME" A_WORK="$WORK" A_T="$T" perl -pe \
  's/\Q$ENV{A_WORK}\E/\/tmp\/probe/g; s/\Q$ENV{A_T}\E/\/tmp\/smoke/g; s/\Q$ENV{A_HOME}\E/\/home\/u/g' \
  "$EV" > "$DEST"
info "golden: ${DEST#"$RAIZ"/} ($(wc -l < "$DEST") líneas)"

if [ "$fallos" -eq 0 ]; then echo "smoke-codex $VER: verde"; exit 0; fi
echo "smoke-codex $VER: $fallos fallos"; exit 1
