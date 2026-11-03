#!/usr/bin/env bash
# smoke-claude.sh — recaptura el golden de hooks de Claude Code (spec
# 2026-11-03-exo-hook-codex §7.12/§10.1.13). Fuera de CI: gasta la cuenta del
# dueño. Se corre en cada actualización de Claude Code.
#
# Lanza `claude -p` con --settings apuntando a hooks de volcado (cat >> ev.jsonl),
# un Bash que falla (PostToolUseFailure) y, en un segundo `claude -p --resume`,
# `/compact` (PreCompact + SessionStart source=compact). Escribe el golden saneado
# en engine/tests/fixtures/hook/claude/claude-<versión>-probe.jsonl.
#
# Uso: scripts/smoke-claude.sh [--keep|--raiz]   (--keep conserva el directorio temporal)
#      scripts/smoke-claude.sh --sanear <work> <tmp> <jsonl>   (solo saneado a stdout, sin claude)
set -uo pipefail

# Rutas absolutas del entorno → fijas, para que el golden no lleve nada personal.
sanear() { # sanear <work> <tmp> <fichero>
  A_HOME="$HOME" A_WORK="$1" A_T="$2" perl -pe \
    's/\Q$ENV{A_WORK}\E/\/tmp\/probe/g; s/\Q$ENV{A_T}\E/\/tmp\/smoke/g; s/\Q$ENV{A_HOME}\E/\/home\/u/g' "$3"
}
if [ "${1:-}" = "--sanear" ]; then
  [ $# -eq 4 ] || { echo "uso: smoke-claude.sh --sanear <work> <tmp> <jsonl>" >&2; exit 1; }
  sanear "$2" "$3" "$4"; exit
fi

KEEP=0
[ "${1:-}" = "--keep" ] && KEEP=1

RAIZ="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)" || exit 1
[ "${1:-}" = "--raiz" ] && { echo "$RAIZ"; exit 0; }
command -v claude >/dev/null || { echo "smoke-claude: falta claude en PATH" >&2; exit 1; }
command -v jq >/dev/null || { echo "smoke-claude: falta jq en PATH" >&2; exit 1; }
command -v uuidgen >/dev/null || { echo "smoke-claude: falta uuidgen en PATH" >&2; exit 1; }

VER="$(claude --version | awk '{print $1}')"
T="$(mktemp -d)"
if [ "$KEEP" -eq 0 ]; then trap 'rm -rf "$T"' EXIT; else echo "smoke-claude: dir temporal $T"; fi

WORK="$T/work"
EV="$T/ev.jsonl"
mkdir -p "$WORK"
: > "$EV"

cat > "$T/dump.sh" <<EOF2
#!/bin/sh
# Una línea por evento: el payload crudo (ya es una línea JSON).
cat >> "$EV"; printf '\n' >> "$EV"
EOF2
chmod +x "$T/dump.sh"

dump() { printf '[{"hooks":[{"type":"command","command":"sh \\"%s/dump.sh\\"","timeout":5}]}]' "$T"; }
cat > "$T/settings.json" <<EOF2
{"hooks":{
  "SessionStart":$(dump),
  "UserPromptSubmit":$(dump),
  "PreToolUse":$(dump),
  "PostToolUse":$(dump),
  "PostToolUseFailure":$(dump),
  "PreCompact":$(dump),
  "Stop":$(dump)
}}
EOF2
jq -e . "$T/settings.json" >/dev/null || { echo "smoke-claude: settings.json generado no es JSON" >&2; exit 1; }

SID="$(uuidgen)"
PROMPT="Ejecuta estos pasos en orden, cada uno con una llamada de shell separada: \
1) echo hola  2) false . Luego responde 'listo'."

( cd "$WORK" && claude -p --dangerously-skip-permissions --settings "$T/settings.json" \
    --session-id "$SID" "$PROMPT" ) > "$T/exec.log" 2>&1
RC=$?
( cd "$WORK" && claude -p --dangerously-skip-permissions --settings "$T/settings.json" \
    --resume "$SID" "/compact" ) > "$T/compact.log" 2>&1
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

chk mal "claude -p rc=0" "claude -p rc=$RC (ver $T/exec.log con --keep)" test "$RC" -eq 0
chk mal "claude -p --resume /compact rc=0" "resume rc=$RC2 (ver $T/compact.log con --keep)" test "$RC2" -eq 0
sed -i '/^$/d' "$EV"
n="$(wc -l < "$EV")"
chk mal "hooks cargados: $n eventos" "cero eventos: Claude no cargó los hooks de --settings" test "$n" -gt 0
chk mal "SessionStart con source=startup" "falta SessionStart source=startup" \
  has 'any(.[]; .hook_event_name=="SessionStart" and .source=="startup")'
chk mal "evento Stop" "falta evento Stop" has 'any(.[]; .hook_event_name=="Stop")'
chk mal "evento PostToolUseFailure" "falta PostToolUseFailure (¿el Bash 'false' no se ejecutó?)" \
  has 'any(.[]; .hook_event_name=="PostToolUseFailure")'
chk info "SessionStart con source=compact" "ningún SessionStart con source=compact (¿/compact no corrió en -p?)" \
  has 'any(.[]; .hook_event_name=="SessionStart" and .source=="compact")'
chk info "evento PreCompact" "falta PreCompact" has 'any(.[]; .hook_event_name=="PreCompact")'
info "eventos vistos: $(jq -r -s '[.[].hook_event_name] | unique | join(",")' "$EV")"

# --- golden saneado ------------------------------------------------------------
DEST="$RAIZ/engine/tests/fixtures/hook/claude/claude-$VER-probe.jsonl"
mkdir -p "$(dirname "$DEST")"
sanear "$WORK" "$T" "$EV" > "$DEST"
info "golden: ${DEST#"$RAIZ"/} ($(wc -l < "$DEST") líneas)"

if [ "$fallos" -eq 0 ]; then echo "smoke-claude $VER: verde"; exit 0; fi
echo "smoke-claude $VER: $fallos fallos"; exit 1
