#!/usr/bin/env bash
# Ningún script de <dir> lee la salida de `jq` línea a línea sin protegerse
# del CRLF del jq nativo de Windows (winget): `while read` / `read` desde una
# tubería o una process substitution se lleva el `\r` pegado al último campo.
# `exo doctor` dejó de avisar de ese CRLF porque ningún script vivo lo sufre;
# este gate es lo que mantiene cierto ese "ningún script vivo".
#
# Se marca una línea lógica (continuaciones `\` unidas) con `jq` que alimenta
# un `read` (`jq … | … read`, `< <(… jq …)`) salvo que el jq lleve `-j` /
# `--join-output` (sin saltos de línea) o la línea filtre con `tr -d '\r'`.
# También `read … <<< "$(… jq …)"`: vale si el fichero limpia la última
# variable del read con `${VAR%$'\r'}`.
#
# Segunda forma, la asignación `VAR="$(… jq …)"` / `VAR=$(… jq …)` /
# `VAR="${VAR:-$(… jq …)}"`: la sustitución de comandos quita el `\n` final pero
# no el `\r`, y la variable acaba con `\r` (en `[ "$X" = ok ]` o dentro de una
# ruta). Excepciones: `-j`/`--join-output`, `tr -d '\r'` en la misma línea
# lógica, o `${VAR%$'\r'}` en el fichero (para el multilínea, donde el `tr`
# cae en otra línea). Criterio explícito para el JSON: un programa jq que
# empieza por `{` (`jq -c`/`-n`/`-Rs '{…}'`) emite un objeto JSON, y un `\r`
# final es whitespace para quien lo parsea (stdout del hook, `--argjson`);
# los `\r` internos van escapados. Un `jq -c`/`-n` cuyo programa NO es un
# objeto (número, cadena, array) no se exime: un número con `\r` rompe
# `[ "$N" -ge 3 ]`. La regla de asignación no mira los `test-*.sh`: son
# fixtures que corren en dev/CI, no hooks que ejecute el jq de un usuario.
# Heurística de línea: no sigue variables intermedias.
# Uso: check-jq-crlf.sh [<dir>]   (default plugins/exo/scripts)
set -uo pipefail
DIR="${1:-plugins/exo/scripts}"
[ -d "$DIR" ] || { echo "check-jq-crlf: no existe $DIR" >&2; exit 1; }

# Cero ficheros escaneados sería un verde sin haber mirado nada.
mapfile -t FICHEROS < <(find "$DIR" -maxdepth 1 -type f -name '*.sh' | sort)
if [ "${#FICHEROS[@]}" -eq 0 ]; then
  echo "check-jq-crlf: ningún .sh bajo $DIR" >&2
  exit 1
fi

awk '
  function limpia_ultima(l,   pre, w, k, v, txt, x) {
    pre = substr(l, 1, index(l, "<<<") - 1); sub(/[[:space:]]+$/, "", pre)
    k = split(pre, w, /[[:space:]]+/); v = w[k]
    while ((getline x < FILENAME) > 0) txt = txt x "\n"
    close(FILENAME)
    return index(txt, "${" v "%$\x27\\r\x27}") > 0
  }
  function limpia_var(v,   txt, x) {
    while ((getline x < FILENAME) > 0) txt = txt x "\n"
    close(FILENAME)
    return index(txt, "${" v "%$\x27\\r\x27}") > 0
  }
  function asignacion(l, n,   v) {
    if (FILENAME ~ /(^|\/)test-[^\/]*$/) return
    if (!match(l, /(^|[^[:alnum:]_])[A-Za-z_][A-Za-z0-9_]*=("?\$\(|"?\$\{[^}]*\$\()/)) return
    if (substr(l, RSTART + RLENGTH) !~ /(^|[^[:alnum:]_.-])jq[[:space:]]/) return
    v = substr(l, RSTART, RLENGTH); sub(/^[^[:alnum:]_]/, "", v); sub(/=.*/, "", v)
    if (l ~ /jq[[:space:]]+(-[[:alpha:]]*j[[:alpha:]]*|--join-output)([[:space:]]|$)/) return
    if (l ~ /tr[[:space:]]+-d[[:space:]]+.\\r./) return
    if (l ~ /jq[^\x27]*\x27[[:space:]]*\{/) return
    if (limpia_var(v)) return
    printf "%s:%d: jq asignado con $(…) sin -j, tr -d \x27\\r\x27 ni ${%s%%$\x27\\r\x27}: %s\n", FILENAME, n, v, substr(l, 1, 100)
    malas++
  }
  function revisa(l, n,   aqui) {
    if (l !~ /(^|[^[:alnum:]_.-])jq[[:space:]]/) return
    asignacion(l, n)
    aqui = (l ~ /(^|[^[:alnum:]_])read[[:space:]].*<<<[[:space:]]*"?\$\(.*jq[[:space:]]/)
    if (aqui && limpia_ultima(l)) return
    if (!aqui && l !~ /<[[:space:]]*<\(.*jq[[:space:]]/ && l !~ /jq[[:space:]].*\|[^|]*(^|[^[:alnum:]_-])read[[:space:]]/) return
    if (l ~ /jq[[:space:]]+(-[[:alpha:]]*j[[:alpha:]]*|--join-output)([[:space:]]|$)/) return
    if (l ~ /tr[[:space:]]+-d[[:space:]]+.\\r./) return
    printf "%s:%d: jq leído con read sin -j ni tr -d \x27\\r\x27: %s\n", FILENAME, n, substr(l, 1, 100)
    malas++
  }
  FNR == 1 { acum = ""; ini = 0 }
  {
    if (acum == "") ini = FNR
    if ($0 ~ /\\$/) { acum = acum substr($0, 1, length($0) - 1) " "; next }
    revisa(acum $0, ini); acum = ""
  }
  END { exit malas > 0 }
' "${FICHEROS[@]}"
