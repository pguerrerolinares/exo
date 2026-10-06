#!/usr/bin/env bash
# Valida el mod de reglas de proyecto (plugins/exo/hooks/register.ts) con el
# harness del propio claude. Sin `claude` en el PATH se salta en voz alta;
# EXO_REQUIRE_CLAUDE=1 (release) convierte ese salto en fallo.
set -uo pipefail
cd "$(dirname "$0")/../../.." || exit 1

if ! command -v claude >/dev/null 2>&1; then
  echo "[SKIP-GRITA] claude no está en PATH: el mod de reglas NO se ha validado" >&2
  [ "${EXO_REQUIRE_CLAUDE:-}" = 1 ] && exit 1
  exit 0
fi

claude plugin validate plugins/exo || { echo "test-reglas-mod: FALLO — plugin validate" >&2; exit 1; }
claude plugin test plugins/exo || { echo "test-reglas-mod: FALLO — plugin test" >&2; exit 1; }
echo "test-reglas-mod: OK — validate y test del mod en verde"
