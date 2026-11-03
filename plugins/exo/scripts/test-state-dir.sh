#!/usr/bin/env bash
# Tests de _state-dir.sh: rutas de estado de los reflejos (.exo/state) y harness en reflex_log.
# shellcheck disable=SC2016 # los bash -c llevan $0 en comillas simples a propósito: lo expande el hijo
set -uo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
FAIL=0
ok()   { echo "PASS $1"; }
fail() { echo "FAIL $1: $2"; FAIL=1; }
eq()   { if [ "$2" = "$3" ]; then ok "$1"; else fail "$1" "esperado [$3] obtenido [$2]"; fi; }

# Carga _state-dir.sh en un subshell con entorno limpio y vuelca las cinco variables.
vars() { # $@ = asignaciones de entorno extra
  env -i PATH="$PATH" "$@" bash -c '. "$0"; printf "%s\n" "$EXO_STATE_DIR" "$REFLEX_LOG_FILE" "$EXO_RULES_DIR" "$EXO_INDEX_LOG" "$REFLEX_CANARY_FILE"' "$DIR/_state-dir.sh"
}

h="$T/h"; mkdir -p "$h"
mapfile -t v < <(vars HOME="$h")
eq "test-state-dir: default EXO_STATE_DIR"    "${v[0]}" "$h/.exo/state"
eq "test-state-dir: default REFLEX_LOG_FILE"  "${v[1]}" "$h/.exo/state/reflex-log.jsonl"
eq "test-state-dir: default EXO_RULES_DIR"    "${v[2]}" "$h/.exo/state/exo-rules"
eq "test-state-dir: default EXO_INDEX_LOG"    "${v[3]}" "$h/.exo/state/exo-index.log"
eq "test-state-dir: default REFLEX_CANARY_FILE" "${v[4]}" "$h/.exo/state/reflex-inject-canary"

mapfile -t v < <(vars HOME="$h" EXO_STATE_DIR="$T/s" REFLEX_LOG_FILE="$T/r")
eq "test-state-dir: propia gana REFLEX_LOG_FILE" "${v[1]}" "$T/r"
eq "test-state-dir: EXO_STATE_DIR rige el resto"  "${v[3]}" "$T/s/exo-index.log"
eq "test-state-dir: EXO_STATE_DIR rige rules"     "${v[2]}" "$T/s/exo-rules"

mapfile -t v < <(vars HOME="$h" EXO_STATE_DIR="" REFLEX_LOG_FILE="")
eq "test-state-dir: vacío = ausente" "${v[1]}" "$h/.exo/state/reflex-log.jsonl"

# reflex_log añade harness
log="$T/rl.jsonl"
env -i PATH="$PATH" HOME="$h" REFLEX_LOG_FILE="$log" bash -c '. "$0" && reflex_log x "{}" p' "$DIR/_reflex-log.sh"
eq "test-state-dir: reflex-log lleva harness" "$(jq -r '.harness' "$log" 2>/dev/null)" "claude"
eq "test-state-dir: reflex-log sigue siendo JSON de una línea" "$(wc -l < "$log" | tr -d ' ')/$(jq -r '.reflex' "$log" 2>/dev/null)" "1/x"

# _state-dir.sh se carga en cada reflejo: sin sustituciones de comando, backticks ni cd
# (el test con strace no existe en todas las máquinas; este es el estático siempre activo).
[ -f "$DIR/_state-dir.sh" ] || fail "test-state-dir: _state-dir.sh existe" "falta"
sin_codigo="$(grep -v '^[[:space:]]*#' "$DIR/_state-dir.sh")"
if printf '%s\n' "$sin_codigo" | grep -nE '\$\(|`|(^|[;&|[:space:]])cd[[:space:]]'; then
  fail "test-state-dir: _state-dir.sh sin forks" "hay \$(...), backticks o cd"
else
  ok "test-state-dir: _state-dir.sh sin forks (estático)"
fi
if command -v strace >/dev/null 2>&1; then
  n="$(strace -f -e trace=clone,fork,vfork,clone3 -o "$T/st" env -i PATH="$PATH" HOME="$h" bash -c '. "$0"' "$DIR/_state-dir.sh" >/dev/null 2>&1; grep -c 'clone\|fork' "$T/st")"
  eq "test-state-dir: 0 forks al cargar (strace)" "$n" "0"
else
  echo "SKIP strace no disponible"
fi

# Ninguna ruta vieja en los consumidores
cd "$DIR/../../.." || exit 1
mapfile -t ficheros < <(printf '%s\n' \
  plugins/exo/scripts/_state-dir.sh plugins/exo/scripts/_reflex-log.sh plugins/exo/scripts/cobertura-reglas.sh \
  plugins/exo/scripts/reflex-fp-review.sh plugins/exo/scripts/document-remind.sh plugins/exo/scripts/subagent-inject.sh \
  plugins/exo/scripts/compose-inject.sh plugins/exo/scripts/exo-recall.sh plugins/exo/scripts/exo-index.sh \
  plugins/exo/scripts/reflex-fp-adjudicate.prompt.md plugins/exo/hooks/register.ts scripts/e2e-reglas-proyecto.sh)
viejas="$(grep -n '\.claude/\(reflex-log\|exo-rules\|exo-index\|reflex-inject-canary\)' "${ficheros[@]}" 2>&1)"
if [ -z "$viejas" ]; then ok "test-state-dir: sin rutas .claude"; else fail "test-state-dir: sin rutas .claude" "$viejas"; fi

if [ "$FAIL" = 0 ]; then echo OK; else echo FALLOS; exit 1; fi
