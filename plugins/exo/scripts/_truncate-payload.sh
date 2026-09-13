#!/usr/bin/env bash
# Helper COMPARTIDO: el payload que un reflejo de PreToolUse:Bash persiste en
# el log cuando dispara. Lo usan git-add-all-guard.sh y verify-before-commit.sh,
# que llevaban este bloque copiado línea a línea.
#
# Uso:
#   PAYLOAD="${CMD:0:120}"   # fallback si el helper no se puede cargar
#   . "$(dirname "$0")/_truncate-payload.sh" 2>/dev/null && payload_truncado "$CMD" "$PATRON"
#
# Asigna la variable global PAYLOAD en vez de imprimir: una sustitución
# `$(...)` se comería los saltos de línea finales, y el bloque que sustituye
# asignaba sin subshell.
#
# Contrato: comando corto (<=120 chars) -> el comando entero, sin marcador.
# Comando largo -> prefijo de 120 chars + " … ⟨match⟩ " + TODAS las
# ocurrencias de PATRON (hasta MATCH_MAX, deduplicadas preservando orden,
# unidas con " | "), cada una truncada POR SEPARADO a cabeza+cola. Sin
# ocurrencias (no debería pasar: el reflejo ya comprobó que PATRON casa) ->
# solo el prefijo.
#
# OJO: el prefijo se saca con expansión de parámetro (${cmd:0:120}), NO con
# `cut -c`. `cut -c` trunca POR LÍNEA, no el string completo: con un comando
# de muchas líneas cortas cada una sobrevive intacta y el "prefijo" real acaba
# siendo líneas*120 caracteres, reventando el cap de _reflex-log.sh y
# comiéndose el match.
# OJO 2: PATRON no tiene techo (un path absurdo tras `-C`, espacios sin fin
# tras `git`), así que un match gigante puede por sí solo topar el cap de 2000
# de _reflex-log.sh y comerse la sentencia que disparó. Por eso CADA ocurrencia
# se trunca por separado, cabeza Y cola: el fragmento que informa vive al
# FINAL del match.
# OJO 3: TODAS las ocurrencias y no solo la primera (`head -1`). Con 2+
# ocurrencias en el mismo comando -- una mención en prosa dentro de un heredoc
# seguida de la invocación real -- quedarse con la primera loguea la mención
# inocua y esconde la real: el falso positivo benigno que el instrumento
# existe para medir, entrando por otra puerta.
# Nota: estos cortes cuentan caracteres en locale UTF-8 pero bytes en
# LC_ALL=C; el corte puede caer a mitad de un carácter multibyte. jq lo tolera
# (carácter de reemplazo, exit 0) y el contrato best-effort aguanta.

# shellcheck disable=SC2034 # PAYLOAD es la salida: la lee el script que hace source
payload_truncado() {
  local cmd="$1" patron="$2"
  local match_head=80 match_tail=60 match_max=5
  if [ "${#cmd}" -le 120 ]; then
    PAYLOAD="$cmd"
    return 0
  fi
  local prefijo="${cmd:0:120}"
  local matches match="" m
  matches="$(printf '%s' "$cmd" | grep -Eo "$patron" | head -n "$match_max" | awk '!seen[$0]++')"
  while IFS= read -r m; do
    [ -z "$m" ] && continue
    if [ "${#m}" -gt $((match_head + match_tail)) ]; then
      m="${m:0:match_head}…${m: -match_tail}"
    fi
    if [ -z "$match" ]; then
      match="$m"
    else
      match="${match} | ${m}"
    fi
  done <<< "$matches"
  if [ -n "$match" ]; then
    PAYLOAD="${prefijo} … ⟨match⟩ ${match}"
  else
    PAYLOAD="$prefijo"
  fi
}
