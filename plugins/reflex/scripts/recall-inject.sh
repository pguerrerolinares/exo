#!/usr/bin/env bash
# UserPromptSubmit hook: busca el prompt de Paul en la KB e inyecta punteros a
# lo relevante (M6-06, "recall en el punto de uso"). Transporte mecánico: el
# modelo no decide si buscar.
#
# HAZARD PROPIO DE ESTE EVENTO, y la razón de que aquí no haya `set -e`: un
# exit 2 no degrada, BORRA el prompt de Paul. Es el único hook del harness
# donde un bug destruye input del usuario. Por eso: cero `set -e`, tuberías
# con `|| true`, y `exit 0` incondicional al final.
#
# Segundo hazard: en UserPromptSubmit el stdout plano de un hook que sale con 0
# SE INYECTA COMO CONTEXTO. Un `echo` de debug olvidado no ensucia un log:
# entra en el turno como si fuera material de la KB. Todo lo que no sea el
# JSON final va a stderr o a /dev/null.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

if [ -t 0 ]; then INPUT=""; else INPUT="$(cat)"; fi
[ -n "$INPUT" ] || exit 0

# Seams por entorno: permiten probar el hook sin tocar la instalación real y,
# para otra persona, apuntar a SU KB sin editar el script.
EXO_BIN="${EXO_BIN:-$(command -v exo 2>/dev/null || echo "$HOME/.local/bin/exo")}"
EXO_INDEX="${EXO_INDEX:-$HOME/.exo/index.db}"

log_ri() {  # $1=sufijo de evento  $2=payload
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && \
    reflex_log "recall-inject-$1" "$INPUT" "${2:-}" || true
}

PROMPT="$(printf '%s' "$INPUT" | jq -r '.prompt // empty' 2>/dev/null)" || PROMPT=""
[ -n "$PROMPT" ] || exit 0

# --- Gate léxico -------------------------------------------------------------
# Traducción literal de `norm()` y `STOP` de
# docs/superpowers/consultas/2026-08-22-m6-06/gate-artefacto.py, que es el
# artefacto NORMATIVO: los números del gate (86% de disparo, cero falsos
# negativos topicales) son propiedades de esta lista exacta, así que se traduce,
# no se reescribe. 127 entradas únicas.
STOP='el la los las un una unos unas de del a al en con por para y o u e que se
lo le les mi tu su es son era esta este esto esa ese eso estas estos ya no ni si
tambien tampoco pero aunque como cuando donde cual cuales muy mas menos bien mal
solo ahora luego antes despues aqui ahi alla hoy
ok okey vale dale va venga listo perfe perfecto genial claro exacto correcto
gracias adelante
di haz hazlo corre lanza lanzalo revisa arregla borra quita usa guarda sube baja
sigue continua para espera prueba mira pon dime
pushea push commitea commit mergea mergealo merge rama ramas repo repos master
main pr
uno dos tres cuatro cinco 1 2 3 4 5'
# La lista se escribe en varias líneas por legibilidad, pero la pertenencia se
# comprueba con `case " $STOP " in *" $tok "*`: sin colapsar los saltos de línea
# a espacios, todo token a final de línea fallaría el match y el gate no callaría
# casi nunca.
STOP=" $(printf '%s' "$STOP" | tr '\n' ' ') "

# NFD + minúsculas + strip de acentos, conservando `/`, `.` y `-`. Que conserve
# esos tres NO es descuido: es lo que hace la normalización medida, y quitar
# toda la puntuación construiría un gate distinto del que produjo los números.
norm_token() {
  local t="${1,,}"
  t="$(printf '%s' "$t" \
       | sed -e 's/á/a/g' -e 's/é/e/g' -e 's/í/i/g' -e 's/ó/o/g' -e 's/ú/u/g' \
             -e 's/ü/u/g' -e 's/ñ/n/g' 2>/dev/null)" || t="${1,,}"
  printf '%s' "$t" | sed 's/[^a-z0-9/.-]//g' 2>/dev/null || true
}

gate_skip() {  # 0 = saltar, 1 = disparar
  local p="$1"
  case "$p" in
    '<'*) return 0 ;;          # turnos user no humanos (teammate, notificaciones)
    '/'*|'!'*) return 0 ;;     # comandos al harness, no prompts
  esac
  local tok n hay=0
  for tok in $p; do
    n="$(norm_token "$tok")"
    [ -n "$n" ] || continue
    case "$STOP" in
      *" $n "*) continue ;;
    esac
    case "$n" in
      [0-9]*) [ -z "${n//[0-9]/}" ] && continue ;;
    esac
    hay=1
    break
  done
  [ "$hay" -eq 1 ] && return 1
  return 0
}

# La abstención del gate es el caso NORMAL (~14% de los turnos): no se loguea,
# o el log diría más sobre el silencio que sobre los fallos.
gate_skip "$PROMPT" && exit 0

# --- Guards ------------------------------------------------------------------
if [ ! -x "$EXO_BIN" ]; then
  log_ri "degraded" "reason=no-engine bin=$EXO_BIN"
  exit 0
fi
if [ ! -f "$EXO_INDEX" ]; then
  # Sin índice NO se pasa `--refresca`: dispararía un bootstrap de minutos bajo
  # el timeout del evento. Se abstiene y deja rastro.
  log_ri "degraded" "reason=no-index db=$EXO_INDEX"
  exit 0
fi

exit 0
