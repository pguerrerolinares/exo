#!/usr/bin/env bash
# SessionStart hook: inyecta el core-index de la KB (contrato de memoria + doctrina compacta
# + mapa de cores; fuente unica: kb-demo/core/core-index) + digest de actividad 3d.
# Fallback al texto embebido si la KB no responde. Nunca bloquea el arranque (exit 0).
# Si SessionStart dispara con source=="compact", re-inyecta también los reflejos que
# dispararon en LA sesión (constraint pinning post-compactación).
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# stdin siempre trae el JSON del hook en harness; el guard -t evita colgarse en debug manual
if [ -t 0 ]; then INPUT=""; else INPUT="$(cat)"; fi

FALLBACK='Tu memoria persistente es el MCP basic-memory, proyecto kb-demo. Antes de empezar trabajo sustantivo, busca ahi contexto relevante (search_notes/recent_activity). Al cerrar una sesion con decisiones/aprendizajes, documentalos con el comando /documenta (search-before-write, edita la nota canonica en vez de duplicar).

MODO ORQUESTADOR LIMPIO (por defecto): delega investigacion, ejecucion multi-paso y lecturas voluminosas a subagentes (Explore para busquedas/lecturas que solo necesitas resumidas; un research-agent con modelo barato para investigar). Quedate con la CONCLUSION, no con el material crudo. Context-rot validado: mas contexto en el padre = peor rendimiento y menos eficiencia. Tu contexto es para sintetizar y decidir, no para acumular fuentes. Hacer el trabajo voluminoso inline en el padre es el anti-patron por defecto a evitar.

RECON-FIRST (look before you leap) en tareas DURAS/desconocidas/time-boxed: antes de grindear en solitario, recoge informacion (busca el error/los docs, verifica supuestos). Retrieve > compute para lo que no esta en tus pesos; reintentar lo mismo a ciegas no es progreso. Gate de dificultad: solo si el terreno es desconocido o llevas varios intentos sin avanzar — con priors solidos y progreso, sigue. (skill: recon-first.)'

# Launcher: seam de test (BM_RECALL_UVX) > binario directo (uv tool install
# basic-memory; evita la resolución de entorno de uvx, ~6s en frío) > uvx.
if [ -n "${BM_RECALL_UVX:-}" ]; then
  BM=("$BM_RECALL_UVX" basic-memory)
elif command -v basic-memory >/dev/null 2>&1; then
  BM=("$(command -v basic-memory)")
elif [ -x "$HOME/.local/bin/basic-memory" ]; then
  BM=("$HOME/.local/bin/basic-memory")
else
  BM=("$(command -v uvx 2>/dev/null || echo /home/paul/.local/bin/uvx)" basic-memory)
fi

MODE="${1:-}"
CACHE="${BM_RECALL_CACHE_FILE:-$HOME/.claude/reflex-recall-cache.md}"
TTL="${BM_RECALL_CACHE_TTL:-1800}"

# Fetch paralelo a la KB; deja CORE y DIGEST en globals (el coste por llamada
# es el arranque del launcher, no la KB; en paralelo el wall es max(a,b) en
# vez de a+b).
fetch_kb() {
  CORE_TMP="$(mktemp)" DIGEST_TMP="$(mktemp)"
  timeout 8 "${BM[@]}" tool read-note core/core-index --project kb-demo >"$CORE_TMP" 2>/dev/null </dev/null &
  PID_CORE=$!
  timeout 8 "${BM[@]}" tool recent-activity --timeframe 3d --project kb-demo >"$DIGEST_TMP" 2>/dev/null </dev/null &
  PID_DIGEST=$!
  wait "$PID_CORE"; CORE_EC=$?
  wait "$PID_DIGEST"; DIGEST_EC=$?
  CORE="$(cat "$CORE_TMP" 2>/dev/null)" || CORE=""
  [ "$CORE_EC" -eq 0 ] || CORE=""
  DIGEST_RAW="$(cat "$DIGEST_TMP" 2>/dev/null)" || DIGEST_RAW=""
  [ "$DIGEST_EC" -eq 0 ] || DIGEST_RAW=""
  rm -f "$CORE_TMP" "$DIGEST_TMP"
  # read-note devuelve el note envuelto en JSON ({title,permalink,content,...}); si es
  # parseable, nos quedamos solo con el contenido markdown real (sin el envoltorio ni el
  # escaping de JSON). Si no es JSON (p.ej. una version que ya devuelva markdown plano) o
  # no tiene .content, seguimos con el output tal cual.
  CORE_CONTENT="$(printf '%s' "$CORE" | jq -re '.content' 2>/dev/null)" && CORE="$CORE_CONTENT"
  DIGEST="$(printf '%s' "$DIGEST_RAW" | jq -r '.[].permalink' 2>/dev/null | sort -u | head -15)" || DIGEST=""
}

# F3.1: las tres ramas de guard de compose_base caían al FALLBACK en silencio
# — Paul pierde el mapa de la KB en TODAS las sesiones sin enterarse. Un evento
# por rama (reason distinguible) deja rastro greppable en reflex-log.jsonl sin
# tocar el contrato "nunca romper el arranque" (best-effort, siempre `|| true`).
log_recall_fallback() {  # $1=reason(empty|oversize|no-contract) $2=payload extra opcional
  local input="$INPUT"
  [ -n "$input" ] || input='{}'
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && \
    reflex_log "recall-fallback" "$input" "reason=$1${2:+ $2}" || true
}

# Compone core+digest a stdout SOLO si CORE pasa los guards; si no, return 1
# (el caller decide fallback). El fallback nunca sale de aquí — así jamás se cachea.
compose_base() {
  if [ -z "$CORE" ]; then
    log_recall_fallback "empty"
    return 1
  fi
  if [ "${#CORE}" -gt 6144 ]; then
    log_recall_fallback "oversize" "size=${#CORE}"
    return 1
  fi
  if ! printf '%s' "$CORE" | grep -q 'Contrato de memoria'; then
    log_recall_fallback "no-contract"
    return 1
  fi
  if [ -n "$DIGEST" ]; then
    printf '%s\n\n--- Actividad reciente (3d, permalinks; read_note para el detalle) ---\n%s' "$CORE" "$DIGEST"
  else
    printf '%s' "$CORE"
  fi
}

write_cache_atomic() {
  mkdir -p "$(dirname "$CACHE")" 2>/dev/null || true
  TMP="$(mktemp -p "$(dirname "$CACHE")" .reflex-recall-cache.XXXXXX)"
  printf '%s' "$1" > "$TMP" && mv "$TMP" "$CACHE"
}

if [ "$MODE" = "--refresh-cache" ]; then
  fetch_kb
  BASE="$(compose_base)" && write_cache_atomic "$BASE"
  exit 0
fi

BASE=""
if [ "$MODE" = "--cached" ] && [ -f "$CACHE" ]; then
  AGE=$(( $(date +%s) - $(stat -c %Y "$CACHE" 2>/dev/null || echo 0) ))
  if [ "$AGE" -lt "$TTL" ] && grep -q 'Contrato de memoria' "$CACHE" 2>/dev/null; then
    BASE="$(cat "$CACHE")"
    # Mantiene el cache caliente sin bloquear el arranque; hereda seams por env.
    # setsid -f: sesión/grupo nuevo — el harness mata el process group del hook
    # al terminar y un simple nohup+& muere con él (visto en vivo 2026-07-10).
    setsid -f nohup bash "$0" --refresh-cache >/dev/null 2>&1 </dev/null
  fi
fi
if [ -z "$BASE" ]; then
  fetch_kb
  if BASE="$(compose_base)"; then
    [ "$MODE" = "--cached" ] && write_cache_atomic "$BASE"
  else
    BASE="$FALLBACK"
  fi
fi
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
        verify-before-done) PIN="${PIN}- verifica (corre el cambio) antes de afirmar exito.\n" ;;
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
