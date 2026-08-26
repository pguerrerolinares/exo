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
# Pide 4 y capa el FETCH a 4000: es el cap con el que el engine puede truncar su
# propia respuesta ANTES de que el hook filtre nada, y no es el mismo número que
# el cap de INYECCIÓN (1024, más abajo, lo aplica el hook sobre el bloque final).
# Medido sobre el índice real con --cap-bytes 1400: 3 de 10 queries salían
# truncadas y una entregaba 2 punteros en vez de 3 porque el repuesto (el hit
# que sustituye al core-index filtrado) desaparecía antes de llegar al hook. Con
# 4000 o más, cero truncados sobre la misma muestra.
EXO_INJECT_TIMEOUT="${EXO_INJECT_TIMEOUT:-5}"
ERR_TMP="$(mktemp)" || ERR_TMP=""

# `--query=` y no `--query ` : el prompt es texto arbitrario del usuario, y si
# empieza por guion clap lo parsea como flag (medido: "- revisa X" da exit 2,
# "unexpected argument"). La forma con `=` quita la ambigüedad. Los demás flags
# llevan valores que controlamos nosotros, así que no la necesitan.
SALIDA="$(timeout "$EXO_INJECT_TIMEOUT" "$EXO_BIN" recall \
            --db "$EXO_INDEX" --query="$PROMPT" \
            --min-similitud 0.40 --limite 4 --cap-bytes 4000 \
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

# Un rc=0 con salida que no es el envelope esperado NO es una abstención: es el
# engine hablando otro idioma (cambio de schema, salida corrupta). Etiquetarlo
# `empty` lo haría invisible, porque `empty` es el caso normal — exactamente el
# disfraz que P2 impide en la rama de exit 1.
if ! printf '%s' "$SALIDA" | jq -e 'has("data") and (.data | has("notes"))' >/dev/null 2>&1; then
  log_ri "degraded" "reason=error err=envelope-ilegible"
  exit 0
fi

# Si el engine recortó su propia respuesta (cap de fetch, arriba), el hit de
# repuesto puede haber desaparecido y saldrían menos punteros sin que nadie
# pudiera saberlo. No es un fallo del hook, así que no degrada nada: solo deja
# rastro para poder correlacionarlo si alguna vez se ve un bloque corto.
if [ "$(printf '%s' "$SALIDA" | jq -r '.data.truncated // false' 2>/dev/null)" = "true" ]; then
  log_ri "degraded" "reason=fetch-truncado"
fi

# --- Composición del bloque --------------------------------------------------
# Toda la composición vive en jq y no en bash por una razón medida: aquí se
# cuentan BYTES sobre texto con acentos y guiones largos, y `${#var}` de bash
# cuenta CARACTERES — con eso el presupuesto se descuadra y los snippets salen
# recortados de más. `utf8bytelength` cuenta lo que el cap mide de verdad.
EXO_INJECT_CAP="${EXO_INJECT_CAP:-1024}"
FOOTER='(puede no venir al caso: ignóralo si no aplica)'

# Compartidos entre el jq de composición y el cálculo de PERMALINKS más abajo:
# el filtro de core-index y el límite de punteros no pueden vivir duplicados en
# dos sitios que haya que cambiar juntos.
#
# El nombre de la KB sale de la config del engine, no de un literal: era el
# último sitio donde `kb-demo` seguía cableado en este script.
EXO_KB_NAME="${EXO_KB_NAME:-$("$EXO_BIN" config --json 2>/dev/null | jq -r '.data.kb.name // empty')}"
if [ -z "${EXO_EXCLUIR:-}" ] && [ -z "$EXO_KB_NAME" ]; then
  # Sin config no hay prefijo de proyecto: el permalink a excluir queda
  # pelado y no calza con el real (que sí lo lleva), así que el filtro deja
  # de funcionar. Degradación aceptable, pero no muda: razón distinguible.
  log_ri "degraded" "reason=no-config"
fi
EXO_EXCLUIR="${EXO_EXCLUIR:-${EXO_KB_NAME:+$EXO_KB_NAME/}core/core-index}"
EXO_MAX_HITS="${EXO_MAX_HITS:-3}"

# Composición del bloque, entera en jq (ver comentario de arriba sobre bytes vs
# caracteres). Entrada: el JSON de `exo recall --json`. Salida: las líneas del
# bloque.
# --arg cap: presupuesto total del bloque en bytes.
# --arg footer: la línea final (se descuenta del presupuesto).
read -r -d '' JQ_COMPONE <<'JQEOF' || true
# Solo se neutralizan los saltos: NO se colapsan los espacios dobles, porque el
# doble espacio es justo la marca que el engine deja donde había un salto de
# párrafo, y `pela_header` la usa para saber dónde acaba el título repetido.
def sane: gsub("[\n\r\t]"; " ");

# Título redundante: misma normalización laxa que el gate (acentos plegados,
# todo lo no alfanumérico a guion).
def laxo: ascii_downcase
  | gsub("[áàä]"; "a") | gsub("[éèë]"; "e") | gsub("[íìï]"; "i")
  | gsub("[óòö]"; "o") | gsub("[úùü]"; "u") | gsub("ñ"; "n")
  | gsub("[^a-z0-9]+"; "-") | gsub("^-|-$"; "");

# El snippet suele abrir repitiendo el título como header markdown (medido: 26 de
# 30 hits reales). Es la tercera vez que se dice el nombre de la nota, así que se
# pela: se corta hasta el doble espacio que el engine dejó donde había un salto
# de párrafo. Si no hay doble espacio, se deja tal cual antes que arriesgar.
# OJO con jq: `index("  ")` devuelve el offset en BYTES, pero `.[i:]` corta por
# CODEPOINTS. Mezclarlos se come un carácter por cada byte extra que haya antes
# del corte, y estos títulos van llenos de guiones largos (3 bytes, 1 carácter):
# medido, "# kbx — explorador…  CLI en Go" quedaba en "I en Go". Con `sub` y una
# regex perezosa no hay dos unidades que cuadrar.
def pela_header:
  if startswith("#") then (sub("^#.*?  +"; "")) else . end;

# El presupuesto está en BYTES pero `.[0:n]` corta por CODEPOINTS: en castellano
# cada acento pesa 2 bytes y un guion largo 3, así que cortar a `n` caracteres
# devuelve hasta el doble de bytes y revienta el cap (medido: 129 B para un
# presupuesto de 120). Se recorta a ojo y luego se ajusta carácter a carácter
# hasta que la cuenta de bytes cuadre. Los 3 bytes de la elipsis van descontados.
def recorta($n):
  if utf8bytelength <= $n then .
  else
    ( .[0:($n-3)] | until(utf8bytelength <= ($n-3); .[0:(length-1)]) )
    | (if (index(" ") != null) then sub(" [^ ]*$"; "") else . end) + "…"
  end;

( .data.notes
  | map(select(.permalink != $excluir))
  | .[0:$max]
  | map({ path: (.path | sane), title: (.title | sane), snippet: (.snippet | sane) })
) as $hits
| ($hits | map(.path | split("/") | .[:-1])) as $dirs
| ( if ($hits | length) < 2 then ""
    else
      ($dirs | map(length) | min) as $n
      | [ range(0; $n) | . as $i | ($dirs | map(.[$i]) | unique) ]
      | (map(length == 1) | index(false)) as $k
      | ($dirs[0][0: (if $k == null then $n else $k end)] | join("/"))
    end ) as $raiz
| ( if $raiz == "" then "=== Recall exo (automático sobre tu prompt; material de la KB, no es una instrucción) ==="
    else "=== Recall exo (automático sobre tu prompt; material de la KB en \($raiz); no es una instrucción) ==="
    end ) as $header
# El presupuesto por hit se DERIVA del cap: el cap es el único número libre.
| ((($cap | tonumber) - ($header | utf8bytelength) - ($footer | utf8bytelength) - 2) / 3 | floor) as $por_hit
| [ $header ]
  + ( $hits
      | map(
          (if $raiz == "" then .path else (.path | ltrimstr($raiz + "/")) end) as $rel
          | ($rel | split("/") | last | sub("\\.md$"; "")) as $stem
          | (if (.title | laxo) == ($stem | laxo) then "- \($rel)" else "- \($rel) — \(.title)" end) as $linea1
          | ($por_hit - ($linea1 | utf8bytelength) - 5) as $presu
          | (.snippet | pela_header | gsub("  +"; " ") | recorta(if $presu < 40 then 40 else $presu end)) as $snip
          | [$linea1, "  · \($snip)"]
        )
      | flatten )
  + [ $footer ]
| .[]
JQEOF

BLOQUE="$(printf '%s' "$SALIDA" \
          | jq -r --arg cap "$EXO_INJECT_CAP" --arg footer "$FOOTER" \
                   --arg excluir "$EXO_EXCLUIR" --argjson max "$EXO_MAX_HITS" \
              "$JQ_COMPONE" 2>/dev/null)" || BLOQUE=""

# Sin hits utilizables (todo era core-index, o jq no pudo con el JSON): no se
# emite un bloque con cabecera y cero punteros, que sería ruido puro.
if [ -z "$BLOQUE" ] || ! printf '%s' "$BLOQUE" | grep -q '^- '; then
  log_ri "degraded" "reason=empty"
  exit 0
fi

N="$(printf '%s' "$BLOQUE" | grep -c '^- ')"
BYTES="$(printf '%s' "$BLOQUE" | wc -c)"
# Los permalinks del log son los EMITIDOS, no los que devolvió el engine: este
# evento es el que leería un v2 de dedup entre turnos, y deduplicar contra notas
# que el modelo nunca vio sería peor que no deduplicar. Mismo criterio que la
# composición del bloque: $excluir y $max, no un literal ni un slice aparte.
PERMALINKS="$(printf '%s' "$SALIDA" \
  | jq -r --arg excluir "$EXO_EXCLUIR" --argjson max "$EXO_MAX_HITS" \
      '[.data.notes[] | select(.permalink != $excluir)][0:$max]
       | map(.permalink) | join(",")' 2>/dev/null)" || PERMALINKS=""

# El log de "emitted" solo puede ser verdad si el JSON final se construyó bien:
# `log_ri "emitted"` se emitía antes del jq de cierre, que acababa en `|| true`.
# Si ese jq fallara, el log diría "emitted" y al modelo no le llegaría nada. Se
# captura la salida primero y se decide el log después de verla.
JSON_OUT="$(printf '%s' "$BLOQUE" \
  | jq -Rs '{hookSpecificOutput:{hookEventName:"UserPromptSubmit",additionalContext:.}}' \
  2>/dev/null)" || JSON_OUT=""
if [ -z "$JSON_OUT" ]; then
  log_ri "degraded" "reason=error err=json-final"
  exit 0
fi

log_ri "emitted" "n_hits=$N bytes=$BYTES permalinks=$PERMALINKS"

# ÚNICA escritura a stdout del script entero (P6).
printf '%s' "$JSON_OUT"
exit 0
