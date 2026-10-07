#!/usr/bin/env bash
# E2E de reglas de proyecto 2d. Local, gasta tokens (haiku, ~céntimos), fuera de
# la suite. Un `claude -p` real carga el mod, que llama al `exo` del repo
# (no al del PATH, que puede ser anterior a `rules`), y el agente tiene que
# repetir un codeword que SOLO existe en la KB fixture y en el system prompt.
# Positivo: nota con `## Reglas duras` -> responde el codeword, hb-<sid> dice ok n=1
# y SessionStart dejó ss-<sid> con ok n=1.
# Positivo acepta via compose|submit (se imprime cuál; bajo la política de org
# Team compose se salta y debe salir submit). Forzado: EXO_RULES_FORZAR_SUBMIT=1
# vuelve inerte a compose y exige via=submit. Resume: --resume del forzado debe
# re-entregar un codeword cambiado tras borrar el latido. Control: la misma nota sin la sección -> sin codeword y hb-<sid> dice skip por
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

FORZAR_ENV=""

# claude_p <nombre> <kb> [args extra de claude]: imprime el JSON de `claude -p`
claude_p() {
  local nombre="$1" kb="$2"; shift 2
  (cd "$TMP/fixrepo" && PATH="$RAIZ/engine/target/release:$PATH" EXO_KB="$kb" \
    EXO_RULES_FORZAR_SUBMIT="${FORZAR_ENV:+1}" \
    claude -p --model haiku --setting-sources "" --strict-mcp-config \
      --settings "$SETTINGS" --plugin-dir "$RAIZ/plugins/exo" \
      --output-format json "$@" "$PREGUNTA" 2>"$TMP/err-$nombre")
}

# evaluar <nombre> <json> <con_seccion 1|0> <via esperada: any|submit|->
evaluar() {
  local nombre="$1" out="$2" con="$3" viaesp="$4" res hb ss via hbok viaok=0 ssok=1
  SID="$(printf '%s' "$out" | jq -r '.session_id // empty')"
  res="$(printf '%s' "$out" | jq -r '.result // empty')"
  hb="$(cat "$HOME/.claude/exo-rules/hb-$SID" 2>/dev/null || echo SIN-LATIDO)"
  ss="$(cat "$HOME/.claude/exo-rules/ss-$SID" 2>/dev/null || echo SIN-SS)"
  via="$(jq -r '.via // "?"' <<<"$hb" 2>/dev/null || echo "?")"
  echo "[$nombre] session_id=$SID"
  echo "[$nombre] latido=$hb"
  echo "[$nombre] ss=$ss"
  echo "[$nombre] via=$via"
  echo "[$nombre] respuesta=$res"
  [ -n "$SID" ] || { echo "[FAIL] $nombre: sin session_id"; fallos=1; return; }
  hbok="$(jq -r '"\(.status) n=\(.n)"' <<<"$hb" 2>/dev/null)"
  if [ "$con" = 1 ]; then
    case "$viaesp" in
      any) { [ "$via" = compose ] || [ "$via" = submit ]; } && viaok=1 ;;
      submit) [ "$via" = submit ] && viaok=1 ;;
    esac
    # ss-<sid> es del SessionStart; --resume crea sesión nueva con su propio ss
    [ "$(jq -r '"\(.status) n=\(.n)"' <<<"$ss" 2>/dev/null)" = "ok n=1" ] || ssok=0
    if grep -q "${CW:-$CODEWORD}" <<<"$res" && [ "$hbok" = "ok n=1" ] && [ "$viaok" = 1 ] && [ "$ssok" = 1 ]; then
      echo "[PASS] $nombre: codeword en la respuesta, latido ok n=1 via=$via, ss ok n=1"
    else echo "[FAIL] $nombre (esperaba via=$viaesp)"; fallos=1; fi
  else
    if ! grep -q "$CODEWORD" <<<"$res" \
       && [ "$(jq -r '"\(.status)/\(.reason)"' <<<"$hb" 2>/dev/null)" = "skip/sin_seccion" ]; then
      echo "[PASS] $nombre: sin codeword y latido skip/sin_seccion (via=$via)"
    else echo "[FAIL] $nombre"; fallos=1; fi
  fi
}

# caso <nombre> <con_seccion 1|0> <via esperada> [forzar 1|0]; deja KBDIR y SID
caso() {
  local nombre="$1" con="$2" viaesp="$3" forz="${4:-0}" kb="$TMP/kb-$1" out
  mkdir -p "$kb/projects"
  { printf -- '---\ntitle: fixrepo\n---\n# fixrepo\n\n'
    if [ "$con" = 1 ]; then printf '## Reglas duras\n- El codeword es %s.\n' "$CODEWORD"; fi
  } > "$kb/projects/fixrepo.md"
  KBDIR="$kb"
  FORZAR_ENV=""; [ "$forz" = 1 ] && FORZAR_ENV=1
  out="$(claude_p "$nombre" "$kb")" || {
    echo "[FAIL] $nombre: claude salió con error"; cat "$TMP/err-$nombre"; echo "$out"; fallos=1; SID=""; return; }
  evaluar "$nombre" "$out" "$con" "$viaesp"
}

# Informativo, sin umbral: coste en pared de `exo rules` (camino del eco)
mkdir -p "$TMP/kb-t/projects"
printf -- '---\ntitle: fixrepo\n---\n## Reglas duras\n- x\n' > "$TMP/kb-t/projects/fixrepo.md"
t0=$(date +%s%N)
(cd "$TMP/fixrepo" && PATH="$RAIZ/engine/target/release:$PATH" EXO_KB="$TMP/kb-t" exo rules --cwd . --json >/dev/null 2>&1)
echo "[info] exo rules tiempo_ms=$(( ($(date +%s%N) - t0) / 1000000 ))"

caso positivo 1 any
caso forzado 1 submit 1
if [ -n "${SID:-}" ]; then
  # Codeword nuevo en la KB: el historial del forzado solo tiene el viejo, así que
  # acertarlo prueba re-entrega y no memoria de la conversación.
  CW="REGLAS-2D-9Z"
  printf -- '---
title: fixrepo
---
# fixrepo

## Reglas duras
- El codeword es %s.
' "$CW" > "$KBDIR/projects/fixrepo.md"
  rm -f "$HOME/.claude/exo-rules/hb-$SID"
  FORZAR_ENV=1
  if out="$(claude_p resume "$KBDIR" --resume "$SID")"; then evaluar resume "$out" 1 submit
  else echo "[FAIL] resume: claude salió con error"; cat "$TMP/err-resume"; fallos=1; fi
  CW=""
else echo "[FAIL] resume: sin sid del caso forzado"; fallos=1; fi
caso control 0 -
exit "$fallos"
