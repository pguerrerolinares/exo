#!/usr/bin/env bash
# Helper COMPARTIDO: hook_ms de reloj de pared para hooks bash (campaña I,
# decisión #12 de Paul, 2026-09-19). Sin spawn: usa $EPOCHREALTIME (bash >=5)
# y aritmética entera de bash. En bash <5 (macOS con /bin/bash 3.2 -- la
# campaña E ya pagó un bug de ese mismo bash con `${arr[@]}` tratado como
# variable no definida bajo `set -u` cuando el array está vacío,
# subagent-inject.sh:42) $EPOCHREALTIME no existe: se degrada a HOOK_MS
# vacío. NUNCA se llama a `date` como sustituto -- ese spawn es exactamente
# lo que esta campaña existe para quitar.
#
# Dos funciones, no una (review adversarial 2026-09-19): `hook_ms_calcula`
# es PURA -- sin mirar la versión de bash -- para que se pueda testear la
# aritmética en CUALQUIER bash, incluido 3.2. `hook_ms_de` es la que se usa
# de verdad en un hook: aplica el guard y delega. Con las dos fundidas en
# una sola función (como la primera versión de este helper), un test que
# llama a esa función directamente en bash <5 nunca llega a ejercer la
# aritmética -- el guard corta antes.
#
# Uso:
#   . "$SCRIPT_DIR/_hook-ms.sh"
#   HOOK_START=""; hook_ms_soportado && HOOK_START="$EPOCHREALTIME"
#   ...
#   hook_ms_de "$HOOK_START"   # setea HOOK_MS (global); vacío si no hay soporte

hook_ms_soportado() { [ "${BASH_VERSINFO[0]:-0}" -ge 5 ]; }

# hook_ms_calcula INICIO FIN -> setea HOOK_MS (global, entero de
# milisegundos). PURA: no mira `hook_ms_soportado`, así que es testeable en
# cualquier bash. INICIO/FIN en formato de $EPOCHREALTIME
# ("SEGUNDOS.MICROS", 6 dígitos de fracción, separador PUNTO en el caso
# normal).
#
# Todo se convierte a microsegundos ANTES de dividir por 1.000: dividir el
# delta de segundos y el de microsegundos por separado y sumar los dos
# resultados pierde precisión cuando el microsegundo del FIN es menor que el
# del INICIO (cruce de segundo) -- medido con "10.999900" -> "11.000100"
# (200 microsegundos reales): esa fórmula daba 1 ms en vez de 0.
#
# Normaliza coma a punto ANTES de partir por `.`: bajo un locale con
# LC_NUMERIC que use coma decimal (es_ES.utf8, verificado real en la máquina
# de Paul -- y Git Bash en Windows hereda el locale regional de Windows),
# $EPOCHREALTIME imprime "1789804285,193241". Sin esto, `${ini%.*}` no
# encuentra ningún punto, devuelve la cadena ENTERA con la coma dentro, y la
# aritmética de abajo da un número absurdo SIN fallar (medido: del orden de
# 8x10^8) -- el recall-inject.sh que llama a esto TAMBIÉN fuerza
# `export LC_NUMERIC=C` (mismo patrón que `recall-latencia.sh:13`), así que
# esto es cinturón y tirantes, no la única defensa.
#
# `10#` fuerza base 10 en la aritmética: un componente con cero a la
# izquierda ("007811") lo interpretaría bash como octal y "008" o "009"
# revientan con "value too great for base 8".
hook_ms_calcula() {
  local ini="$1" fin="$2"
  ini="${ini//,/.}"; fin="${fin//,/.}"
  local s0="${ini%.*}" u0="${ini#*.}" s1="${fin%.*}" u1="${fin#*.}"
  # shellcheck disable=SC2034 # HOOK_MS es la salida global: la lee el llamador
  HOOK_MS=$(( ((10#$s1 - 10#$s0) * 1000000 + (10#$u1 - 10#$u0)) / 1000 ))
}

# hook_ms_de INICIO [FIN] -> guard + hook_ms_calcula. FIN por defecto: ahora.
# HOOK_MS queda vacío (cadena "") si `hook_ms_soportado` es falso -- la
# aritmética de hook_ms_calcula NUNCA llega a correr en ese caso.
hook_ms_de() {
  # shellcheck disable=SC2034 # HOOK_MS es la salida global: la lee el llamador
  HOOK_MS=""
  hook_ms_soportado || return 0
  hook_ms_calcula "$1" "${2:-$EPOCHREALTIME}"
}
