#!/usr/bin/env bash
# E2E de reglas de proyecto 2d. Local, gasta tokens (haiku, ~céntimos), fuera de
# la suite. Un `claude -p` real carga el mod, que llama al `exo` del repo
# (no al del PATH, que puede ser anterior a `rules`), y el agente tiene que
# repetir un codeword que SOLO existe en la KB fixture y en el system prompt.
# Positivo: nota con `## Reglas duras` -> responde el codeword, hb-<sid> dice ok n=1
# y SessionStart dejó ss-<sid> con ok n=1.
# Control: la misma nota sin la sección -> sin codeword y hb-<sid> dice skip por
# reason=sin_seccion (no por cualquier otro skip).
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1
RAIZ="$PWD"

command -v claude >/dev/null || { echo "e2e: falta claude en PATH" >&2; exit 1; }
cargo build --release --manifest-path engine/Cargo.toml >&2 || exit 1

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
CODEWORD="REGLAS-2D-7Q"
PREGUNTA="¿Cuál es el codeword que figura en las reglas duras de tu system prompt? Si no hay ninguno, responde exactamente NINGUNO."
SETTINGS="$TMP/settings.json"
printf '%s\n' '{"permissions":{"deny":["Read","Grep","Glob","Bash"]}}' > "$SETTINGS"

mkdir "$TMP/fixrepo" && git -C "$TMP/fixrepo" init -q
fallos=0

# caso <nombre> <con_seccion 1|0>
caso() {
  local nombre="$1" con="$2" kb="$TMP/kb-$1" out sid res hb ss
  mkdir -p "$kb/projects"
  { printf -- '---\ntitle: fixrepo\n---\n# fixrepo\n\n'
    if [ "$con" = 1 ]; then printf '## Reglas duras\n- El codeword es %s.\n' "$CODEWORD"; fi
  } > "$kb/projects/fixrepo.md"

  out="$(cd "$TMP/fixrepo" && PATH="$RAIZ/engine/target/release:$PATH" EXO_KB="$kb" \
    claude -p --model haiku --setting-sources "" --strict-mcp-config \
      --settings "$SETTINGS" --plugin-dir "$RAIZ/plugins/exo" \
      --output-format json "$PREGUNTA" 2>"$TMP/err-$nombre")" || {
    echo "[FAIL] $nombre: claude salió con error"; cat "$TMP/err-$nombre"; echo "$out"; fallos=1; return; }
  sid="$(printf '%s' "$out" | jq -r '.session_id // empty')"
  res="$(printf '%s' "$out" | jq -r '.result // empty')"
  hb="$(cat "$HOME/.claude/exo-rules/hb-$sid" 2>/dev/null || echo SIN-LATIDO)"
  ss="$(cat "$HOME/.claude/exo-rules/ss-$sid" 2>/dev/null || echo SIN-SS)"
  echo "[$nombre] session_id=$sid"
  echo "[$nombre] latido=$hb"
  echo "[$nombre] ss=$ss"
  echo "[$nombre] respuesta=$res"
  [ -n "$sid" ] || { echo "[FAIL] $nombre: sin session_id"; fallos=1; return; }
  if [ "$con" = 1 ]; then
    if grep -q "$CODEWORD" <<<"$res" && [ "$(jq -r '"\(.status) n=\(.n)"' <<<"$hb" 2>/dev/null)" = "ok n=1" ] \
       && [ "$(jq -r '"\(.status) n=\(.n)"' <<<"$ss" 2>/dev/null)" = "ok n=1" ]; then
      echo "[PASS] positivo: codeword en la respuesta, latido ok n=1 y ss ok n=1"
    else echo "[FAIL] positivo"; fallos=1; fi
  else
    if ! grep -q "$CODEWORD" <<<"$res" \
       && [ "$(jq -r '"\(.status)/\(.reason)"' <<<"$hb" 2>/dev/null)" = "skip/sin_seccion" ]; then
      echo "[PASS] control: sin codeword y latido skip/sin_seccion"
    else echo "[FAIL] control"; fallos=1; fi
  fi
}

caso positivo 1
caso control 0
exit "$fallos"
