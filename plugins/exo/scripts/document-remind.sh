#!/usr/bin/env bash
# Stop hook: (1) testigo de entrega de reglas de proyecto, (2) recordatorio no bloqueante para documentar la sesion via /document
# (engine exo). Solo recuerda una vez por sesion (sentinel) y solo si hubo
# trabajo real (umbral de transcript).
set -uo pipefail

INPUT="$(cat)"

SESSION_ID="$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)"
TRANSCRIPT="$(printf '%s' "$INPUT" | jq -r '.transcript_path // empty' 2>/dev/null)"

# Sin session_id no podemos deduplicar; salimos sin molestar.
[ -z "$SESSION_ID" ] && exit 0

SENTINEL_DIR="${REMIND_SENTINEL_DIR:-/tmp}"
SENTINEL="${SENTINEL_DIR}/claude-document-reminded-${SESSION_ID}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Testigo: compara lo que SessionStart decidio (ss) con lo que el mod entrego (hb).
# Va ANTES del umbral del recordatorio y con sentinel propio: ninguno silencia al otro.
WITNESS_MSG=""
WITNESS_LOG=""
RULES_DIR="$HOME/.claude/exo-rules"
WSENT="${SENTINEL_DIR}/claude-rules-witness-${SESSION_ID}"
DEGRADADO_PREFIJO="ℹ"
DSENT="${SENTINEL_DIR}/claude-rules-degradado-$(date +%Y%m%d)"
SS="$RULES_DIR/ss-${SESSION_ID}"
HB="$RULES_DIR/hb-${SESSION_ID}"
if [ ! -f "$WSENT" ]; then
  WREASON=""; WTOUCH=1
  if [ ! -f "$SS" ]; then
    WREASON="sin_ss"
  elif [ ! -f "$HB" ]; then
    WREASON="sin_latido"
    WITNESS_MSG="⚠ el mod de reglas de proyecto no cargó (sin latido): esta sesión no lleva reglas en el system prompt"
  else
    SS_ST="$(jq -r '.status // ""' "$SS" 2>/dev/null)"
    SS_N="$(jq -r '.n // 0' "$SS" 2>/dev/null)"
    if HB_ROW="$(jq -r '[(.status // ""), (.n // 0)] | @tsv' "$HB" 2>/dev/null)" && [ -n "$HB_ROW" ]; then
      HB_ST="${HB_ROW%%$'\t'*}"; HB_N="${HB_ROW##*$'\t'}"
    else
      HB_ST="corrupto"; HB_N="?"
    fi
    [ -z "$HB_ST" ] && { HB_ST="corrupto"; HB_N="?"; }
    # hb de 1.6.1 sin via = compose; corrupto no se lee.
    HB_VIA=""; HB_ORG=""
    if [ "$HB_ST" != "corrupto" ]; then
      HB_VIA="$(jq -r '.via // ""' "$HB" 2>/dev/null)" || HB_VIA=""
      HB_ORG="$(jq -r '.org // ""' "$HB" 2>/dev/null)" || HB_ORG=""
    fi
    if [ "$HB_VIA" = "none" ] && [ "$SS_ST" = "ok" ]; then
      WREASON="sin_canal"
      WITNESS_MSG="⚠ el mod de reglas de proyecto cargó pero ningún canal entregó (via=none): SessionStart vio ok n=${SS_N}; ¿política de la org nueva o exo rules falla?"
    elif [ "$HB_ST" = "corrupto" ] || { [ "$HB_ST" = "error" ] && [ "$SS_ST" = "ok" ]; }; then
      WREASON="no_entrego"
      WITNESS_MSG="⚠ el mod de reglas de proyecto no entregó: SessionStart vio ${SS_ST} n=${SS_N}, el latido dice ${HB_ST} n=${HB_N}"
    elif [ "$HB_ST" = "error" ]; then
      # El mod falló pero no había reglas que entregar: sin ruido en pantalla, con rastro.
      WREASON="hb_error"
    elif [ "$HB_ST" != "$SS_ST" ] || [ "$HB_N" != "$SS_N" ]; then
      # ss es una foto de SessionStart y compose corre en cada prompt: la KB pudo
      # cambiar a mitad de sesión. Solo el engine adjudica quién tiene razón.
      . "$SCRIPT_DIR/_timeout.sh" 2>/dev/null
      EXO_BIN="${EXO_BIN:-$(command -v exo 2>/dev/null || echo "$HOME/.local/bin/exo")}"
      WCWD="$(printf '%s' "$INPUT" | jq -r '.cwd // empty' 2>/dev/null)"
      WCWD="${WCWD:-$PWD}"
      WERR_TMP="$(mktemp)"
      WOUT="$(con_timeout "${EXO_RULES_TIMEOUT:-3}" "$EXO_BIN" rules --cwd "$WCWD" --json 2>"$WERR_TMP")" && WRC=0 || WRC=$?
      WERR="$(head -c 200 "$WERR_TMP" 2>/dev/null | tr '\n' ' ')"
      rm -f "$WERR_TMP"
      TR_ST=""; TR_REASON=""; TR_REPO=""; TR_N=""
      if [ "$WRC" -eq 0 ]; then
        IFS=$'\x1f' read -r TR_ST TR_REASON TR_REPO TR_N <<< "$(printf '%s' "$WOUT" | jq -r '[(.data.status // ""), (.data.reason // ""), (.data.repo // ""), ((.data.rules // []) | length | tostring)] | join("\u001f")' 2>/dev/null)"
      fi
      case "$TR_ST" in
        ok) ;;
        skip) TR_N=0 ;;
        *) TR_ST="" ;;
      esac
      case "${TR_N:-}" in ''|*[!0-9]*) TR_ST="" ;; esac
      if [ -z "$TR_ST" ]; then
        WREASON="sin_verdad"; WTOUCH=0
        WITNESS_LOG="${WERR:+ err=$WERR}"
      elif [ "$TR_ST" = "$HB_ST" ] && [ "$TR_N" = "$HB_N" ]; then
        WREASON="kb_cambio"; WTOUCH=0
        if [ -n "$SESSION_ID" ] && [ "${SESSION_ID#*/}" = "$SESSION_ID" ]; then
          jq -cn --arg st "$TR_ST" --arg rs "$TR_REASON" --arg repo "$TR_REPO" --argjson n "$TR_N" \
            '{status:$st} + (if $rs=="" then {} else {reason:$rs} end) + {n:$n, repo:(if $repo=="" then null else $repo end)}' \
            > "$SS.tmp" 2>/dev/null && mv -f "$SS.tmp" "$SS" 2>/dev/null
        fi
      else
        WREASON="no_entrego"
        WITNESS_MSG="⚠ el mod de reglas de proyecto no entregó: el engine dice ${TR_ST} n=${TR_N}, el latido dice ${HB_ST} n=${HB_N}"
      fi
    elif [ "$HB_VIA" = "submit" ] && [ "$HB_ST" = "ok" ] && [ "$HB_N" -gt 0 ] 2>/dev/null; then
      # Entrega por prompt.submit (la org salta compose): log siempre, aviso una vez al día.
      WREASON="entrega_degradada"
      WITNESS_LOG="${HB_ORG:+ org=$HB_ORG}"
      if [ ! -f "$DSENT" ]; then
        WITNESS_MSG="${DEGRADADO_PREFIJO} reglas de proyecto entregadas por canal degradado (política de la org): n=${HB_N}"
        touch "$DSENT" 2>/dev/null
      fi
    fi
  fi
  if [ -n "$WREASON" ]; then
    # kb_cambio y sin_verdad callan: sin sentinel, para poder gritar una divergencia real después.
    [ "$WTOUCH" = 1 ] && touch "$WSENT" 2>/dev/null
    . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null \
      && reflex_log "project-rules-witness" "$INPUT" "reason=${WREASON}${WITNESS_MSG:+ msg=$WITNESS_MSG}${WITNESS_LOG}" || true
  fi
fi

emit() {  # une testigo y recordatorio en un unico objeto JSON
  local m="$WITNESS_MSG"
  [ -n "${1:-}" ] && m="${m:+$m$'\n'}$1"
  [ -n "$m" ] && jq -cn --arg m "$m" '{systemMessage:$m}'
  exit 0
}

# Ya recordado en esta sesion -> solo testigo.
[ -f "$SENTINEL" ] && emit

# Umbral de "sesion con trabajo real": transcript existe y supera ~50 lineas o ~20KB.
[ -z "$TRANSCRIPT" ] && emit
[ -f "$TRANSCRIPT" ] || emit

LINES="$(wc -l < "$TRANSCRIPT" 2>/dev/null | tr -d ' ')"
BYTES="$(wc -c < "$TRANSCRIPT" 2>/dev/null | tr -d ' ')"
LINES="${LINES:-0}"
BYTES="${BYTES:-0}"

# Si no llega al umbral, NO creamos sentinel todavia (puede crecer en futuros Stop).
if [ "$LINES" -lt 50 ] && [ "$BYTES" -lt 20480 ]; then
  emit
fi

# Supera umbral y no hay sentinel: marcamos y recordamos una sola vez.
touch "$SENTINEL" 2>/dev/null
emit "💾 ¿Cierras sesión? Usa /document para guardar decisiones y aprendizajes en la KB."
