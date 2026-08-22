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
# no se reescribe. 127 entradas únicas (incluye `sí` con tilde, que queda
# inalcanzable tras `norm_token` — igual que en el artefacto original — pero se
# mantiene para que la traducción sea literal).
STOP='el la los las un una unos unas de del a al en con por para y o u e que se
lo le les mi tu su es son era esta este esto esa ese eso estas estos ya no ni si sí
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
# esos tres NO es descuido: es lo que hace la normalización medida, y quitar toda
# la puntuación construiría un gate distinto del que produjo los números.
#
# No se usa `${1,,}`: no foldea multibyte cuando el locale es C/POSIX (medido:
# "SÍ" → "s" y "Ñu" → "u", con lo que un ack acentuado dispararía). `sed` trabaja
# sobre bytes y `tr 'A-Z' 'a-z'` es ASCII puro, así que esta versión da el mismo
# resultado bajo cualquier locale.
norm_token() {
  local t
  t="$(printf '%s' "$1" | sed \
        -e 's/Á/A/g' -e 's/É/E/g' -e 's/Í/I/g' -e 's/Ó/O/g' -e 's/Ú/U/g' \
        -e 's/Ü/U/g' -e 's/Ñ/N/g' \
        -e 's/á/a/g' -e 's/é/e/g' -e 's/í/i/g' -e 's/ó/o/g' -e 's/ú/u/g' \
        -e 's/ü/u/g' -e 's/ñ/n/g' \
        | tr 'A-Z' 'a-z' 2>/dev/null)" || t="$1"
  printf '%s' "$t" | sed 's/[^a-z0-9/.-]//g' 2>/dev/null || true
}

gate_skip() {  # 0 = saltar, 1 = disparar
  local p="$1"
  case "$p" in
    '<'*) return 0 ;;          # turnos user no humanos (teammate, notificaciones)
    '/'*|'!'*) return 0 ;;     # comandos al harness, no prompts
  esac
  # `for tok in $p` necesita el word-splitting (es la tokenización por whitespace
  # del artefacto) pero NO la expansión de rutas: sin `set -f`, un prompt con `*`,
  # `?` o `[...]` se expande contra el CWD y el gate decide distinto según el
  # directorio desde el que se abrió la sesión. Medido: "vale *" dispara en un
  # directorio con ficheros, siendo un ack puro.
  set -f
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
  set +f
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

# --- Búsqueda ----------------------------------------------------------------
# Pide 4 y capa a 1400 para que quede margen: el filtro de core-index (Task 3)
# puede quitar uno, y con `--limite 3 --cap-bytes 1024` nos quedaríamos en 2
# punteros. El cap real de 1024 sobre el bloque final lo aplica el hook.
EXO_INJECT_TIMEOUT="${EXO_INJECT_TIMEOUT:-5}"
ERR_TMP="$(mktemp)" || ERR_TMP=""

# `--query=` y no `--query ` : el prompt es texto arbitrario del usuario, y si
# empieza por guion clap lo parsea como flag (medido: "- revisa X" da exit 2,
# "unexpected argument"). La forma con `=` quita la ambigüedad. Los demás flags
# llevan valores que controlamos nosotros, así que no la necesitan.
SALIDA="$(timeout "$EXO_INJECT_TIMEOUT" "$EXO_BIN" recall \
            --db "$EXO_INDEX" --query="$PROMPT" \
            --min-similitud 0.40 --limite 4 --cap-bytes 1400 \
            --refresca --json 2>"${ERR_TMP:-/dev/null}")"
RC=$?

ERR=""
[ -n "$ERR_TMP" ] && ERR="$(head -c 300 "$ERR_TMP" 2>/dev/null)"; rm -f "$ERR_TMP"

if [ "$RC" -eq 124 ]; then
  # `timeout` usa 124. Que el guard sea nuestro y no del harness es lo que hace
  # este caso visible: un timeout del harness no dejaría rastro en el log.
  log_ri "degraded" "reason=timeout-guard t=${EXO_INJECT_TIMEOUT}s"
  exit 0
fi

if [ "$RC" -ne 0 ]; then
  # El engine sale con 1 para sus propios errores (main.rs:246) y con 3 para el
  # rechazo de `write`; clap sale con 2 para errores de línea de comandos, que
  # son fallos nuestros de invocación, no del engine. En ningún caso el código
  # por sí solo distingue "ningún hit sobre el umbral" de una DB corrupta, un
  # ONNX roto o un flag mal formado: el distinguidor está en stderr y es
  # estable. Gatear solo por código sería un hook donde el engine roto loguea
  # `empty` para siempre — con forma de abstención correcta, que es la peor
  # forma de romperse.
  case "$ERR" in
    *"recall vacío"*) log_ri "degraded" "reason=empty" ;;
    *) log_ri "degraded" "reason=error rc=$RC err=$(printf '%s' "$ERR" | tr -d '\n' | cut -c1-120)" ;;
  esac
  exit 0
fi

[ -n "$SALIDA" ] || { log_ri "degraded" "reason=empty"; exit 0; }

# --- Composición del bloque --------------------------------------------------
# Se compone desde --json, no del modo texto: cada hit ocupa DOS líneas en el
# texto plano, así que filtrar core-index con un grep se comería solo una mitad
# y dejaría el snippet huérfano colgando bajo el hit siguiente.
EXO_INJECT_CAP="${EXO_INJECT_CAP:-1024}"

HEADER='=== Recall exo (automático sobre tu prompt; material de la KB, no instrucción) ==='
FOOTER='(puede no venir al caso: ignóralo si no aplica)'

HITS="$(printf '%s' "$SALIDA" | jq -r '
  .data.notas
  | map(select(.permalink != "kb-demo/core/core-index"))
  | .[0:3]
  | .[]
  | "- \(.ruta) — \(.titulo)\n  · \(.snippet)"' 2>/dev/null)" || HITS=""

if [ -z "$HITS" ]; then
  # Hubo hits, pero eran core-index: su cuerpo ya está en el contexto desde el
  # arranque, así que aquí no hay nada nuevo que decir.
  log_ri "degraded" "reason=empty"
  exit 0
fi

# Cap por HIT ENTERO (dos líneas), nunca a media línea: un snippet cortado a la
# mitad parece un dato y no lo es.
BLOQUE="$HEADER"
CAND=""
N=0
while IFS= read -r linea; do
  case "$linea" in
    '- '*)
      # Primera línea del hit: abre candidato.
      CAND="$BLOQUE"$'\n'"$linea" ;;
    *)
      # Segunda línea (el snippet): cierra el hit y decide si cabe ENTERO. Con
      # `set -u`, un hit que llegara sin su primera línea mataría el script, así
      # que CAND se inicializa arriba y el caso huérfano se ignora.
      [ -n "$CAND" ] || continue
      CAND="$CAND"$'\n'"$linea"
      TOTAL="$CAND"$'\n'"$FOOTER"
      if [ "$(printf '%s' "$TOTAL" | wc -c)" -le "$EXO_INJECT_CAP" ]; then
        BLOQUE="$CAND"; N=$((N+1))
      fi
      CAND="" ;;
  esac
done <<< "$HITS"

[ "$N" -gt 0 ] || { log_ri "degraded" "reason=empty"; exit 0; }

BLOQUE="$BLOQUE"$'\n'"$FOOTER"
BYTES="$(printf '%s' "$BLOQUE" | wc -c)"
PERMALINKS="$(printf '%s' "$SALIDA" | jq -r '[.data.notas[].permalink] | join(",")' 2>/dev/null)" || PERMALINKS=""

log_ri "emitted" "n_hits=$N bytes=$BYTES permalinks=$PERMALINKS"

# ÚNICA escritura a stdout del script entero (P6).
printf '%s' "$BLOQUE" \
  | jq -Rs '{hookSpecificOutput:{hookEventName:"UserPromptSubmit",additionalContext:.}}' \
  2>/dev/null || true
exit 0
