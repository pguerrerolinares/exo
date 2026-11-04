#!/usr/bin/env bash
# Rust (`exo config --json` -> data.state.*), bash (_state-dir.sh) y TS
# (register.ts hbPath) deben resolver las mismas rutas de estado (spec §8.1).
# Usa el binario del árbol, no el instalado: EXO_BIN o engine/target/debug/exo.
set -euo pipefail

repo=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)
[ -n "$repo" ] || { echo "test-state-acuerdo: sin raíz del repo" >&2; exit 1; }
estado_sh="$repo/plugins/exo/scripts/_state-dir.sh"
register_ts="${REGISTER_TS:-$repo/plugins/exo/hooks/register.ts}"
case "$(uname -s)" in
  MINGW* | MSYS*) exe=.exe ;;
  *) exe= ;;
esac
exo="${EXO_BIN:-$repo/engine/target/debug/exo$exe}"

[ -x "$exo" ] || { echo "test-state-acuerdo: no hay binario $exo (cargo build --manifest-path engine/Cargo.toml)" >&2; exit 1; }
esperada=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$repo/engine/Cargo.toml" | head -n1)
real=$("$exo" --version | awk '{print $2}')
if [ "$real" != "$esperada" ]; then
  echo "test-state-acuerdo: $exo es $real pero engine/Cargo.toml dice $esperada" >&2
  exit 1
fi

t=$(mktemp -d)
trap 'rm -rf "$t"' EXIT
mkdir -p "$t/home"

norm() {
  case "$(uname -s)" in
    MINGW* | MSYS*) cygpath -u "$1" ;;
    *) printf '%s' "$1" ;;
  esac
}

# clave json <-> variable de _state-dir.sh
claves=(dir reflex_log rules_dir index_log canary)
vars=(EXO_STATE_DIR REFLEX_LOG_FILE EXO_RULES_DIR EXO_INDEX_LOG REFLEX_CANARY_FILE)

# $1 = nombre del caso; el resto = asignaciones VAR=valor extra al entorno.
caso() {
  local nombre=$1
  shift
  local limpio=(env -u EXO_STATE_DIR -u REFLEX_LOG_FILE -u EXO_RULES_DIR -u EXO_INDEX_LOG -u REFLEX_CANARY_FILE
    HOME="$t/home" EXO_CONFIG="$t/sin-config.toml" "$@")
  local json
  # Sin config.toml el envelope sale igual pero con rc!=0: se lee stdout e ignora el rc.
  json=$("${limpio[@]}" "$exo" config --json 2>/dev/null || true)
  local i x a b
  for i in "${!claves[@]}"; do
    x=${claves[$i]}
    a=$(printf '%s' "$json" | jq -r ".data.state.$x // empty")
    # shellcheck disable=SC2016 # el script va entre comillas simples a propósito: lo expande el bash hijo
    b=$("${limpio[@]}" bash -c 'source "$1"; printf "%s" "${!2}"' _ "$estado_sh" "${vars[$i]}")
    [ -n "$a" ] || { echo "test-state-acuerdo: [$nombre] Rust no publica data.state.$x" >&2; exit 1; }
    a=$(norm "$a")
    b=$(norm "$b")
    if [ "$a" != "$b" ]; then
      echo "Rust y bash discrepan en $x: $a ≠ $b" >&2
      exit 1
    fi
  done
}

caso "sin variables"
caso "EXO_STATE_DIR" EXO_STATE_DIR="$t/s"
caso "REFLEX_LOG_FILE" REFLEX_LOG_FILE="$t/r"

# TS: líneas que no son comentario y llevan la lectura de EXO_STATE_DIR y el
# fallback a ~/.exo/state (un comentario con el literal no cuenta).
# shellcheck disable=SC2016 # los literales TS llevan ${...} y comillas invertidas que no deben expandirse
ts_ok() {
  local codigo
  codigo=$(grep -Ev '^[[:space:]]*(//|\*|/\*)' "$1") || return 1
  printf '%s\n' "$codigo" | grep -qF '$.env.get("EXO_STATE_DIR")' &&
    printf '%s\n' "$codigo" | grep -qF '`${estado || `${home}/.exo/state`}/exo-rules/hb-${sid}`'
}
ts_ok "$register_ts" || { echo "test-state-acuerdo: register.ts no resuelve EXO_STATE_DIR || home/.exo/state" >&2; exit 1; }

# Casos negativos: el propio test sabe ponerse rojo.
# shellcheck disable=SC2016 # idem: literal TS
printf '// $.env.get("EXO_STATE_DIR") `${estado || `${home}/.exo/state`}/exo-rules/hb-${sid}`\n' >"$t/comentario.ts"
! ts_ok "$t/comentario.ts" || { echo "test-state-acuerdo: ts_ok acepta un comentario" >&2; exit 1; }
printf '#!/bin/sh\necho "exo 0.0.0"\n' >"$t/exo-falso"
chmod +x "$t/exo-falso"
if salida=$(EXO_BIN="$t/exo-falso" bash "$0" 2>&1) || ! grep -q "es 0.0.0 pero" <<<"$salida"; then
  echo "test-state-acuerdo: la guardia de versión no falla con exo 0.0.0" >&2
  exit 1
fi

echo "test-state-acuerdo: OK"
