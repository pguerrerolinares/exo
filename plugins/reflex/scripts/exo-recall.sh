#!/usr/bin/env bash
# SessionStart hook: inyecta el core-index de la KB + digest de actividad,
# servido por el engine `exo` (M6-02 del cierre de exo). Sustituye a
# basic-memory-recall.sh, que hacía lo mismo a través del CLI de basic-memory.
#
# Por qué desaparece el cache: el camino viejo tardaba ~6,6 s en frío (arranque
# del CLI de Python + MCP), así que necesitaba cache con TTL, refresco en
# background y escritura atómica — 90 líneas de máquina para tapar una latencia.
# `exo recall` tarda ~10 ms leyendo SQLite: el cache sobra, y con él se van sus
# modos de fallo (cache rancio, refresco que muere con el process group,
# escrituras a medias).
#
# Contrato que NO cambia: nunca bloquea el arranque (exit 0 siempre), cae a un
# fallback embebido si no hay bloque, y reafirma los reflejos disparados cuando
# SessionStart llega con source=compact.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# stdin siempre trae el JSON del hook en harness; el guard -t evita colgarse en
# debug manual.
if [ -t 0 ]; then INPUT=""; else INPUT="$(cat)"; fi

# Fallback embebido: se inyecta si el engine no puede servir. Reescrito en el
# cutover — el texto viejo mandaba al agente a un MCP que está en retirada.
FALLBACK='Tu memoria persistente es una KB de notas markdown servida por el engine `exo` (`exo recall`, `exo search --type hybrid`). Antes de empezar trabajo sustantivo, busca ahi contexto relevante. Al cerrar una sesion con decisiones/aprendizajes, documentalos con /documenta (busca antes de escribir; edita la nota canonica en vez de duplicar).

MODO ORQUESTADOR LIMPIO (por defecto): delega investigacion, ejecucion multi-paso y lecturas voluminosas a subagentes. Quedate con la CONCLUSION, no con el material crudo. Context-rot validado: mas contexto en el padre = peor rendimiento. Tu contexto es para sintetizar y decidir, no para acumular fuentes.

RECON-FIRST (look before you leap) en tareas DURAS/desconocidas/time-boxed: antes de grindear en solitario, recoge informacion (busca el error/los docs, verifica supuestos). Retrieve > compute para lo que no esta en tus pesos; reintentar lo mismo a ciegas no es progreso. (skill: recon-first.)'

# Seams por entorno: permiten probar el hook sin tocar la instalación real y,
# para otra persona, apuntar a SU KB sin editar el script.
EXO_BIN="${EXO_BIN:-$(command -v exo 2>/dev/null || echo "$HOME/.local/bin/exo")}"
EXO_INDEX="${EXO_INDEX:-$HOME/.exo/index.db}"
EXO_NOTA="${EXO_RECALL_NOTA:-kb-demo/core/core-index}"
EXO_CAP="${EXO_RECALL_CAP:-6144}"
# Recientes en el digest. El camino viejo listaba hasta 15 permalinks de los
# últimos 3 días; con 5 se perdían notas del mismo día (hallazgo del gate M6).
# 10 cabe de sobra: el bloque real ronda los 4,5 KB sobre un cap de 6144.
EXO_LIMITE="${EXO_RECALL_LIMITE:-10}"

log_recall_fallback() {  # $1=reason $2=payload extra opcional
  local input="$INPUT"
  [ -n "$input" ] || input='{}'
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && \
    reflex_log "recall-fallback" "$input" "reason=$1${2:+ $2}" || true
}

# Un fallback silencioso deja al agente sin mapa de la KB en TODAS las sesiones
# sin que nadie se entere (esto ya mordió una vez, F3.1): cada rama deja un
# evento greppable con su razón, siempre best-effort.
BASE=""
if [ ! -x "$EXO_BIN" ]; then
  log_recall_fallback "no-engine" "bin=$EXO_BIN"
elif [ ! -f "$EXO_INDEX" ]; then
  log_recall_fallback "no-index" "db=$EXO_INDEX"
else
  # stderr se captura, no se tira: ahí avisa el engine de que el bloque no
  # cupo entero. El script viejo caía a fallback con evento `oversize` en ese
  # caso; tirar el aviso dejaría llegar un bloque cortado sin rastro, que es
  # justo la degradación silenciosa que F3.1 arregló.
  ERR_TMP="$(mktemp)"
  BASE="$("$EXO_BIN" recall --db "$EXO_INDEX" --contenido --nota "$EXO_NOTA" \
          --limite "$EXO_LIMITE" --cap-bytes "$EXO_CAP" 2>"$ERR_TMP")" || BASE=""
  if grep -q 'truncado' "$ERR_TMP" 2>/dev/null; then
    log_recall_fallback "truncated" "$(head -1 "$ERR_TMP" | tr -d '\n' | cut -c1-120)"
  fi
  rm -f "$ERR_TMP"
  if [ -z "$BASE" ]; then
    log_recall_fallback "empty"
  elif ! printf '%s' "$BASE" | grep -q 'Contrato de memoria'; then
    # Mismo guard semántico que el script viejo: un bloque sin el contrato de
    # memoria no es el core-index (nota renombrada, índice apuntando a otra
    # cosa), y es mejor el fallback conocido que un bloque plausible pero falso.
    log_recall_fallback "no-contract"
    BASE=""
  fi
fi
[ -n "$BASE" ] || BASE="$FALLBACK"
TEXTO="$BASE"

# --- Reafirmación de reflejos disparados si SessionStart(source=compact) ---
SOURCE="$(printf '%s' "$INPUT" | jq -r '.source // empty' 2>/dev/null)" || SOURCE=""
SID="$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)" || SID=""

if [ "$SOURCE" = "compact" ] && [ -n "$SID" ] && [ -f "$HOME/.claude/reflex-log.jsonl" ]; then
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "compact" "$INPUT" "compact" || true
  FIRED="$(jq -r --arg sid "$SID" 'select(.session_id==$sid) | .reflex' \
            "$HOME/.claude/reflex-log.jsonl" 2>/dev/null | sort -u)"
  if [ -n "$FIRED" ]; then
    PIN=""
    for id in $FIRED; do
      case "$id" in
        git-c|git-c-rewrite) PIN="${PIN}- nunca \`cd X && git ...\`: usa \`git -C X\`.\n" ;;
        verify-before-done|verify-before-commit) PIN="${PIN}- verifica (corre el cambio) antes de afirmar exito.\n" ;;
        search-before-write) PIN="${PIN}- busca en la KB antes de escribir nota nueva.\n" ;;
      esac
    done
    if [ -n "$PIN" ]; then
      TEXTO="${TEXTO}

--- Reglas reforzadas tras compactacion (dispararon esta sesion) ---
$(printf '%b' "$PIN" | awk '!seen[$0]++')"
    fi
  fi
fi

printf '%s' "$TEXTO" | jq -Rs '{hookSpecificOutput:{hookEventName:"SessionStart",additionalContext:.}}'
exit 0
