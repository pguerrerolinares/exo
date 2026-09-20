#!/usr/bin/env bash
# Test standalone para _hook-ms.sh: `hook_ms_calcula` es PURA (sin guard de
# versión) a propósito, para poder probar la aritmética en CUALQUIER bash,
# incluido 3.2 — el guard (`hook_ms_soportado`/`hook_ms_de`) se prueba aparte,
# sin mezclar los dos motivos de fallo posibles en el mismo caso.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

. "$SCRIPT_DIR/_hook-ms.sh"

verifica() {  # $1=inicio $2=fin $3=esperado_ms
  hook_ms_calcula "$1" "$2"
  if [ "$HOOK_MS" = "$3" ]; then
    pass "hook_ms_calcula $1 $2 -> $HOOK_MS"
  else
    fail "hook_ms_calcula $1 $2 -> $3" "obtuve '$HOOK_MS'"
  fi
}

verifica "100.900000" "101.100000" 200
verifica "100.500000" "101.100000" 600
# Componente con cero a la izquierda: interpretado como octal sin el `10#`
# de _hook-ms.sh, "007811" revienta la aritmética ("value too great for base").
verifica "1758300000.007811" "1758300000.500000" 492
# Cruce de segundo con el microsegundo del fin MENOR que el del inicio: el
# bug que tenía la primera versión de esta función (dividir el delta de
# microsegundos por separado del de segundos perdía precisión al cruzar el
# borde). 11.000100 - 10.999900 = 200 microsegundos = 0 ms enteros.
verifica "10.999900" "11.000100" 0
verifica "5.000000" "5.000000" 0
verifica "5.000000" "8.000000" 3000
# Coma decimal: $EPOCHREALTIME bajo un locale con LC_NUMERIC que la use (p.
# ej. es_ES.utf8, verificado real en la máquina de Paul y en la de Git Bash
# en Windows, que hereda el locale regional). Sin normalizar la coma, esto
# devolvía un HOOK_MS del orden de 8x10^8 sin ni siquiera fallar.
verifica "10,999900" "11,000100" 0

if hook_ms_soportado; then
  pass "hook_ms_soportado: verdadero en bash ${BASH_VERSINFO[0]} (>=5 en esta máquina)"
else
  # Esta rama solo se ejerce de verdad en CI macOS (bash 3.2 real) o en
  # `docker run --rm -v "$PWD":/w -w /w bash:3.2 ./test-hook-ms.sh` (Global
  # Constraints). Aquí documentamos el contrato sin fingir la versión:
  # BASH_VERSINFO es de solo lectura, no se puede simular.
  pass "hook_ms_soportado: falso (bash <5 real de esta máquina)"
fi

# hook_ms_de aplica el guard ENCIMA de la aritmética pura: sin soporte,
# HOOK_MS queda vacío SIN que la aritmética llegue a correr (a diferencia de
# la primera versión de este helper, donde el guard vivía dentro de la misma
# función que la aritmética y un test que llamaba a esa función directamente
# en bash <5 nunca ejercía la cuenta). Esto se prueba de verdad —no
# simulado— en `docker run --rm -v "$PWD":/w -w /w bash:3.2 ./test-hook-ms.sh`.
hook_ms_de "100.0" "101.0"
if hook_ms_soportado; then
  [ "$HOOK_MS" = "1000" ] && pass "hook_ms_de con soporte: delega en hook_ms_calcula" \
    || fail "hook_ms_de con soporte: delega en hook_ms_calcula" "obtuve '$HOOK_MS'"
else
  [ -z "$HOOK_MS" ] && pass "hook_ms_de sin soporte (bash <5 real): HOOK_MS vacío, sin correr la aritmética" \
    || fail "hook_ms_de sin soporte: HOOK_MS vacío" "obtuve '$HOOK_MS'"
fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
