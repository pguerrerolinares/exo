#!/usr/bin/env bash
# Stop hook: refresca el índice de exo al cerrar la sesión (M6-01).
#
# Por qué AQUÍ y no en el arranque. basic-memory mantenía su índice al día con
# un watch en segundo plano; exo indexa al invocar, sin daemon. Medido en esta
# máquina: reindexar sin cambios cuesta ~25 ms, pero con una nota editada
# cuesta ~1,5 s (cargar el runtime ONNX + embeber los trozos que cambiaron), y
# tras un clone fresco —todos los mtimes nuevos— más de diez minutos. Poner eso
# en SessionStart convierte un arranque de 14 ms en una espera, y en el peor
# caso en un cuelgue.
#
# Al cierre no molesta a nadie, y es justo cuando la KB acaba de cambiar
# (/documenta escribe al terminar), así que el índice llega fresco al arranque
# siguiente. Medido sobre el repo de la KB: el 95% de sus commits de los
# últimos 60 días se hacen dentro de una sesión de agente, así que este
# disparador cubre casi todo; lo que se edite fuera queda obsoleto como mucho
# una sesión, y `exo recall --refresh` está para forzarlo a mano.
#
# Contrato: nunca bloquea el cierre. Se lanza detached y sale de inmediato.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# stdin trae el JSON del hook; solo se usa para dejar rastro con reflex_log si
# ninguna vía de detach funciona. El guard -t evita colgarse en debug manual.
if [ -t 0 ]; then INPUT=""; else INPUT="$(cat)"; fi

EXO_BIN="${EXO_BIN:-$(command -v exo 2>/dev/null || echo "$HOME/.local/bin/exo")}"
EXO_INDEX="${EXO_INDEX:-$HOME/.exo/index.db}"
LOG="${EXO_INDEX_LOG:-$HOME/.claude/exo-index.log}"

# Seams de test (mismo patrón que EXO_BIN): permiten forzar cada rama de detach
# sin cirugía de PATH. Sin seam, la mitad de las ramas serían intesteables en
# cada máquina: en Linux setsid existe siempre y en Git Bash no existe nunca.
SETSID_BIN="${EXO_INDEX_SETSID:-setsid}"
CMD_BIN="${EXO_INDEX_CMD:-cmd}"

log_index_fallback() {  # $1=reason $2=payload extra opcional
  local input="$INPUT"
  [ -n "$input" ] || input='{}'
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && \
    reflex_log "index-fallback" "$input" "reason=$1${2:+ $2}" || true
}

# Sin binario no hay nada que indexar: máquina sin exo instalado, no es una
# degradación. (Distinto de "hay binario pero no sé lanzarlo detached", que sí
# se loguea más abajo.)
[ -x "$EXO_BIN" ] || exit 0

if command -v "$SETSID_BIN" >/dev/null 2>&1; then
  # setsid -f: el harness mata el process group del hook al terminar, y un
  # nohup+& moriría con él (visto en vivo con el refresco de cache del recall
  # viejo, 2026-07-10). Con sesión propia, el indexado sobrevive al cierre.
  "$SETSID_BIN" -f nohup "$EXO_BIN" index --db "$EXO_INDEX" --json >>"$LOG" 2>&1 </dev/null || true
elif command -v "$CMD_BIN" >/dev/null 2>&1; then
  # Git Bash/msys no trae setsid, y el `|| true` de arriba se tragaba el fallo:
  # el índice llevaba sin refrescarse NUNCA en Windows, en silencio — justo la
  # degradación que exo-recall se esfuerza en evitar. El equivalente aquí es
  # `cmd start`: crea un proceso fuera del árbol de bash, así que sobrevive al
  # kill del process group del hook (verificado en esta máquina matando el
  # group del lanzador con el indexado a medias: terminó igual, 2026-08-24).
  #
  # Se lanza un `bash -c` interior en vez del exe directo por tres razones:
  #   1. `start` no sabe redirigir la salida del proceso que lanza; el bash
  #      interior conserva el `>>$LOG 2>&1` del contrato.
  #   2. Los argumentos viajan como ENV exportado, no como argumentos de cmd:
  #      cero quoting msys→cmd→bash, y msys ya convierte las rutas de las vars
  #      al cruzar a cmd, así que no hay que cygpath-ear cada una a mano.
  #   3. Funciona igual con un exo.exe nativo que con un script (stubs de test).
  # Solo bash necesita ruta Windows para que cmd lo encuentre; la da cygpath,
  # y si falta se cae al nombre pelado confiando en el PATH heredado.
  BASH_BIN="$(command -v bash 2>/dev/null || echo bash)"
  if command -v cygpath >/dev/null 2>&1; then
    BASH_BIN="$(cygpath -w "$BASH_BIN" 2>/dev/null)" || BASH_BIN="bash"
  fi
  export EXO_BIN EXO_INDEX LOG
  # El '""' literal es el título (vacío) que `start` exige como primer
  # argumento cuando el comando va entre comillas; //b evita ventana nueva
  # (msys convierte //c→/c y //b→/b al cruzar a cmd).
  "$CMD_BIN" //c start '""' //b "$BASH_BIN" -c \
    'exec "$EXO_BIN" index --db "$EXO_INDEX" --json >>"$LOG" 2>&1 </dev/null' \
    >/dev/null 2>&1 || log_index_fallback "detach-failed" "via=cmd-start"
else
  # Ninguna vía de detach: el índice NO se va a refrescar esta sesión. Que se
  # note en el log en vez de morir en silencio, que es lo que hacía el
  # `|| true` a secas durante meses en Windows.
  log_index_fallback "no-detach" "bin=$EXO_BIN"
fi
exit 0
