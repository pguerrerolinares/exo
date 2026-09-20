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
# shellcheck disable=SC2016 # texto literal: los backticks son markdown, no sustitución
FALLBACK='Tu memoria persistente es una KB de notas markdown servida por el engine `exo` (`exo recall`, `exo search --type hybrid`). Antes de empezar trabajo sustantivo, busca ahi contexto relevante. Al cerrar una sesion con decisiones/aprendizajes, documentalos con /document (busca antes de escribir; edita la nota canonica en vez de duplicar).

MODO ORQUESTADOR LIMPIO (por defecto): delega investigacion, ejecucion multi-paso y lecturas voluminosas a subagentes. Quedate con la CONCLUSION, no con el material crudo. Context-rot validado: mas contexto en el padre = peor rendimiento. Tu contexto es para sintetizar y decidir, no para acumular fuentes.

RECON-FIRST (look before you leap) en tareas DURAS/desconocidas/time-boxed: antes de grindear en solitario, recoge informacion (busca el error/los docs, verifica supuestos). Retrieve > compute para lo que no esta en tus pesos; reintentar lo mismo a ciegas no es progreso. (skill: recon-first.)'

# Seams por entorno: permiten probar el hook sin tocar la instalación real y,
# para otra persona, apuntar a SU KB sin editar el script.
EXO_BIN="${EXO_BIN:-$(command -v exo 2>/dev/null || echo "$HOME/.local/bin/exo")}"
EXO_INDEX="${EXO_INDEX:-$HOME/.exo/index.db}"
EXO_CAP="${EXO_RECALL_CAP:-6144}"
# ENGINE_MIN (campaña H): el mínimo de engine que ESTE plugin declara
# necesitar, sobreescribible por test (`ENGINE_MIN=x.y.z`, seam igual que
# EXO_BIN/EXO_INDEX de arriba). Sin override, el fichero versionado junto al
# plugin. El check vive SOLO aquí (SessionStart) y no en recall-inject.sh
# (UserPromptSubmit, un spawn por prompt): SessionStart + `exo doctor` ya
# lo cubren.
. "$SCRIPT_DIR/_engine-version.sh" 2>/dev/null
ENGINE_MIN="${ENGINE_MIN:-$(cat "$SCRIPT_DIR/../ENGINE_MIN" 2>/dev/null)}"
ENGINE_MIN="${ENGINE_MIN:-0.0.0}"
# Recientes en el digest. El camino viejo listaba hasta 15 permalinks de los
# últimos 3 días; con 5 se perdían notas del mismo día (hallazgo del gate M6).
# 10 cabe de sobra por número de notas, pero el margen de BYTES ya no sobra:
# medido el 2026-08-27, el bloque real es 5.921 B sobre el cap de 6.144 — un
# 3,6% de aire, y el desbordamiento se trunca EN SILENCIO por el final. El
# comentario anterior decía «ronda los 4,5 KB»: llevaba rancio lo bastante
# como para tranquilizar a quien lo leyera. Ver docs/backlog.md.
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
ENGINE_VER=""
if [ ! -x "$EXO_BIN" ]; then
  log_recall_fallback "no-engine" "bin=$EXO_BIN"
elif [ ! -f "$EXO_INDEX" ]; then
  log_recall_fallback "no-index" "db=$EXO_INDEX"
elif ENGINE_VER="$(exo_version_de "$EXO_BIN")" && [ -n "$ENGINE_VER" ] && ! semver_lt "$ENGINE_VER" "$ENGINE_MIN"; then
  # El nombre de la KB sale de la config del engine, no de un literal: era el
  # último sitio donde `kb-demo` seguía cableado en el camino de arranque.
  # Resuelto AQUÍ (binario ejecutable e índice ya confirmados arriba) y no
  # antes: moverlo antes de esos guards doblaba el log cuando la causa real
  # era `no-engine`/`no-index` — el mismo binario ausente que hace fallar
  # `exo recall` también hace fallar `exo config`, y `no-config` mentiría
  # sobre la causa.
  CONFIG_ERR_TMP="$(mktemp)"
  EXO_KB_NAME="${EXO_KB_NAME:-$("$EXO_BIN" config --json 2>"$CONFIG_ERR_TMP" | jq -r '.data.kb.name // empty')}"
  if [ -z "${EXO_RECALL_NOTA:-}" ] && [ -z "$EXO_KB_NAME" ]; then
    # Sin config no hay prefijo de proyecto: la nota cae a `core/core-index`
    # pelado, que no resuelve. Degradación aceptable, pero no muda: distinta
    # razón que `no-engine`/`no-index`, y el payload trae el motivo exacto de
    # `exo config` (config rota vs. binario transicional que aún no conoce
    # el subcomando) sin tener que reproducirlo a mano.
    log_recall_fallback "no-config" "err=$(head -1 "$CONFIG_ERR_TMP" 2>/dev/null | tr -d '\n' | cut -c1-120)"
  fi
  rm -f "$CONFIG_ERR_TMP"
  EXO_NOTA="${EXO_RECALL_NOTA:-${EXO_KB_NAME:+$EXO_KB_NAME/}core/core-index}"

  # stderr se captura, no se tira: ahí avisa el engine de que el bloque no
  # cupo entero. El script viejo caía a fallback con evento `oversize` en ese
  # caso; tirar el aviso dejaría llegar un bloque cortado sin rastro, que es
  # justo la degradación silenciosa que F3.1 arregló.
  ERR_TMP="$(mktemp)"
  BASE="$("$EXO_BIN" recall --db "$EXO_INDEX" --content --note "$EXO_NOTA" \
          --limit "$EXO_LIMITE" --cap-bytes "$EXO_CAP" 2>"$ERR_TMP")" || BASE=""
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
else
  ENGINE_VER="${ENGINE_VER:-desconocida}"
  log_recall_fallback "engine-stale" "engine=$ENGINE_VER min=$ENGINE_MIN"
  BASE="engine desactualizado ($ENGINE_VER < $ENGINE_MIN) — actualiza el binario instalado

$FALLBACK"
fi
[ -n "$BASE" ] || BASE="$FALLBACK"
TEXTO="$BASE"

# --- Reafirmación de reflejos disparados si SessionStart(source=compact) ---
# SOURCE y SID en una sola pasada de jq (campaña I; antes eran dos jq sobre
# el mismo $INPUT). Separador `\x1f` (unit separator), NO `@tsv`/tab: `read`
# trata el tab como whitespace de IFS y colapsa un campo vacío inicial (el
# caso normal de `source`, ausente en casi todo prompt) -- verificado que
# `@tsv` + `IFS=tab` desalinea SOURCE/SID en ese caso, `\x1f` no.
#
# Caveat conocido (review adversarial 2026-09-20, construido y comprobado):
# un byte `\x1f` LITERAL dentro de `source` sí desalinearía (ese campo no
# pasa por el mismo escapado de jq que evita el problema con tabs/newlines
# embebidos). Riesgo práctico bajo y se acepta: `source` es un enum que fija
# el harness de Claude Code ("startup"/"resume"/"clear"/"compact"/"vscode"),
# no texto libre que un agente o un tercero pueda inyectar.
SOURCE=""; SID=""
IFS=$'\x1f' read -r SOURCE SID <<< "$(printf '%s' "$INPUT" | jq -r '[(.source // ""), (.session_id // "")] | join("\u001f")' 2>/dev/null)"

if [ "$SOURCE" = "compact" ] && [ -n "$SID" ] && [ -f "$HOME/.claude/reflex-log.jsonl" ]; then
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "compact" "$INPUT" "compact" || true
  # H5: el log crece sin cota (~11 KB/día medidos el 2026-09-13) y esto corre
  # en cada compactación. Los disparos de ESTA sesión están en la cola: 2.000
  # líneas son ~74 días al ritmo actual. No se rota nada, porque el análisis
  # (a1-gate, reflex-baseline, reflex-fp-review) necesita la historia entera.
  FIRED="$(tail -n "${EXO_RECALL_COMPACT_LINEAS:-2000}" "$HOME/.claude/reflex-log.jsonl" 2>/dev/null \
            | jq -r --arg sid "$SID" 'select(.session_id==$sid) | .reflex' 2>/dev/null | sort -u)"
  if [ -n "$FIRED" ]; then
    PIN=""
    for id in $FIRED; do
      case "$id" in
        git-c|git-c-rewrite) PIN="${PIN}- nunca \`cd X && git ...\`: usa \`git -C X\`.\n" ;;
        verify-before-done|verify-before-commit) PIN="${PIN}- verifica (corre el cambio) antes de afirmar exito.\n" ;;
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
