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

# verifica() corre hook_ms_calcula en un SUBSHELL (fix review adversarial
# 2026-09-19). Una aritmética inválida dentro de hook_ms_calcula (p. ej. el
# bug del octal sin `10#` que el caso #3 de abajo existe para cazar) es un
# error FATAL de bash -- no un `return 1` normal -- que aborta toda la pila
# de llamadas hasta el nivel superior, saltándose cualquier código posterior
# en la misma función, incluido este propio verifica() si llamara a
# hook_ms_calcula directo (medido: sin subshell, el caso desaparece SIN
# llamar ni a pass ni a fail -- el contador de casos baja de 9 a 8 mientras
# FAIL se queda en 0, y la suite sale en verde). Aislado en un subshell, el
# abort muere ahí dentro y $? se puede leer con normalidad aquí fuera.
verifica() {  # $1=inicio $2=fin $3=esperado_ms
  local ini="$1" fin="$2" esperado="$3" salida rc
  salida="$(hook_ms_calcula "$ini" "$fin"; rc=$?; printf '%s' "$HOOK_MS"; exit "$rc")"
  rc=$?
  if [ "$rc" -ne 0 ]; then
    fail "hook_ms_calcula $ini $fin -> $esperado" "hook_ms_calcula abortó (rc=$rc) -- aritmética inválida, ver stderr arriba"
    return
  fi
  if [ "$salida" = "$esperado" ]; then
    pass "hook_ms_calcula $ini $fin -> $salida"
  else
    fail "hook_ms_calcula $ini $fin -> $esperado" "obtuve '$salida'"
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

# hook_ms_soportado: contrato falsable contra el entorno REAL de esta máquina
# (fix review adversarial 2026-09-19). El esperado se calcula aquí de forma
# INDEPENDIENTE de la función bajo test -- comparando BASH_VERSINFO
# directamente -- para que mutar hook_ms_soportado a `{ true; }` constante
# haga fallar este caso cuando el bash real es <5. Antes, las dos ramas
# llamaban siempre a `pass`, así que esa mutación solo cambiaba el texto del
# mensaje y la suite seguía en verde (medido en
# `docker run --rm -v "$PWD":/w -w /w bash:3.2 ./test-hook-ms.sh`, donde
# hook_ms_soportado DEBE decir "no soportado" y la mutación decía "sí" --
# consecuencia real: recall-inject.sh muere con
# "EPOCHREALTIME: unbound variable" en bash 3.2).
if [ "${BASH_VERSINFO[0]:-0}" -ge 5 ]; then
  esperado_rc=0
else
  esperado_rc=1
fi
hook_ms_soportado
obtenido_rc=$?
if [ "$obtenido_rc" -eq "$esperado_rc" ]; then
  pass "hook_ms_soportado: coincide con bash ${BASH_VERSINFO[0]:-0} (rc=$obtenido_rc)"
else
  fail "hook_ms_soportado: contrato roto para bash ${BASH_VERSINFO[0]:-0}" \
    "esperaba rc=$esperado_rc, obtuve rc=$obtenido_rc"
fi

# hook_ms_de aplica el guard ENCIMA de la aritmética pura: sin soporte,
# HOOK_MS queda vacío SIN que la aritmética llegue a correr (a diferencia de
# la primera versión de este helper, donde el guard vivía dentro de la misma
# función que la aritmética y un test que llamaba a esa función directamente
# en bash <5 nunca ejercía la cuenta). Esto se prueba de verdad —no
# simulado— en `docker run --rm -v "$PWD":/w -w /w bash:3.2 ./test-hook-ms.sh`.
hook_ms_de "100.0" "101.0"
if hook_ms_soportado; then
  if [ "$HOOK_MS" = "1000" ]; then
    pass "hook_ms_de con soporte: delega en hook_ms_calcula"
  else
    fail "hook_ms_de con soporte: delega en hook_ms_calcula" "obtuve '$HOOK_MS'"
  fi
else
  if [ -z "$HOOK_MS" ]; then
    pass "hook_ms_de sin soporte (bash <5 real): HOOK_MS vacío, sin correr la aritmética"
  else
    fail "hook_ms_de sin soporte: HOOK_MS vacío" "obtuve '$HOOK_MS'"
  fi
fi

# Defensa general (fix review adversarial 2026-09-19): si un caso desaparece
# sin llamar ni a pass ni a fail -- el modo de fallo exacto del bug del
# octal antes de este fix, y potencialmente cualquier otro fallo fatal
# futuro no anticipado -- el conteo de PASS+FAIL baja por debajo de lo
# esperado mientras FAIL se queda en 0 y la suite sale en verde. Este
# backstop compara el conteo ejecutado contra el número de aserciones que
# este fichero declara arriba (7 verifica() + 2 checks de guard) y tiñe la
# suite de rojo si no coinciden, sin depender de conocer la causa concreta.
ESPERADOS=9
EJECUTADOS=$((PASS + FAIL))
if [ "$EJECUTADOS" -ne "$ESPERADOS" ]; then
  fail "conteo de casos ejecutados" "esperaba $ESPERADOS, ejecuté $EJECUTADOS -- algún caso abortó sin llamar a pass ni a fail"
fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
