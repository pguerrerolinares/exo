#!/usr/bin/env bash
# PreToolUse (matcher: Bash): script UNICO que funde los tres guards de
# PreToolUse:Bash -- git-c-bash.sh (reflejo "git -C"), git-add-all-guard.sh
# (reflejo "zero-residuo") y verify-before-commit.sh (reflejo
# "verify-before-done"). Motivo: triple PreToolUse:Bash medido en W11 tras
# la Task 4 de la campaña I = 587 ms > 200 ms, el umbral de fusion de
# docs/backlog.md:736-761 (evals/recall-coste/results/w11-2026-09-23-campana-i.txt).
#
# Cada rama conserva su propio PATRON/REWRITE_RE, su propia allowlist, su
# propio id de log y su propio payload de log EXACTAMENTE como en el script
# original del que viene -- lo unico que se comparte es: un solo `cat` de
# stdin, un solo pre-filtro `case *git*`, y una sola extraccion jq de
# tool_input.command/session_id/cwd/transcript_path. Los tres scripts
# originales NO se borran (quedan huerfanos de hooks.json pero siguen
# corriendo en scripts/test-plugin.sh como red de regresion).
#
# REGLA DE FUSION DE SALIDA (por que hace falta una regla y por que hoy es
# trivial de aplicar):
#   Antes, tres hooks == tres procesos == Claude Code combinaba sus tres
#   salidas por su cuenta. Un script unico tiene que producir ESE MISMO
#   efecto combinado en UN SOLO objeto JSON (o nada) en stdout.
#
#   Inventario real de lo que cada rama emite HOY (2026-09-23, verificado
#   leyendo el codigo, no los comentarios historicos -- el comentario de
#   cabecera de git-c-bash.sh todavia dice "WARN (additionalContext)" pero
#   ese codepath ya no imprime nada, solo loguea; los tests de las tres
#   suites lo confirman exigiendo stdout VACIO en el caso "log-only"):
#     - Rama 1a (git-c, rewrite): unica rama que imprime JSON a stdout --
#       {hookSpecificOutput:{permissionDecision:"allow", ...,
#       updatedInput:...}} -- solo si el comando ENTERO parsea como
#       `cd PATH && git <subcomando-de-solo-lectura>` sin metacaracteres.
#     - Rama 1b (git-c, warn), rama 2 (zero-residuo) y rama 3
#       (verify-before-done): SOLO loguean a REFLEX_LOG_FILE (best-effort,
#       nunca rompen) y terminan en silencio. Ninguna emite
#       `additionalContext` ni ningun otro JSON.
#   Con ese inventario, fusionar "hasta tres silencios-que-loguean" con "un
#   rewrite ocasional" no tiene ningun conflicto de verdad que resolver: se
#   emite el JSON del rewrite si lo hay: si no, no se emite nada. Las tres
#   ramas corren SIEMPRE en secuencia (nunca se saltan porque otra ya
#   disparo), exactamente como corrian como tres procesos independientes --
#   la UNICA rama que se salta condicionalmente es 1b, que en el codigo
#   original tampoco se alcanza si 1a ya reescribio y salio con exit 0.
#
#   Regla general (documentada por si una rama futura vuelve a emitir
#   `additionalContext`, aunque hoy sea codigo muerto): varios
#   `additionalContext` no vacios se concatenan (separados por una linea en
#   blanco) en un unico `hookSpecificOutput.additionalContext`, que convive
#   en el MISMO objeto con el `permissionDecision:"allow"` + `updatedInput`
#   del rewrite si lo hay -- son campos independientes de un solo
#   `hookSpecificOutput`. Si alguna rama decidiera `deny`/`ask` (hoy
#   ninguna lo hace: las tres son warn-only, "NUNCA bloquea"), esa decision
#   primaria sobre el `allow` del rewrite y el rewrite NO se aplicaria.
#
#   Nota estructural (por que no hay caso real de "rewrite + log de otra
#   rama" en el mismo comando, verificado por construccion): el rewrite de
#   1a exige que el comando ENTERO sea `cd PATH && git REST` con REST SIN
#   metacaracteres -- eso implica que "git" aparece UNA sola vez en todo el
#   comando. Para que zero-residuo (exige "git" seguido de "add") o
#   verify-before-done (exige "git" seguido de "commit") disparen sobre ESE
#   MISMO "git", el subcomando tendria que ser "add" o "commit" -- pero
#   ninguno de los dos esta en REWRITE_ALLOWLIST (solo lectura), asi que
#   esa misma condicion que habilita el rewrite excluye a las otras dos
#   ramas, y viceversa. Rama 1a y (rama2 O rama3) son mutuamente
#   excluyentes por construccion, no por accidente de los tests. Rama 2 y
#   rama 3 SI pueden dispararse juntas (`git add -A && git commit -m x`) --
#   cubierto en test-bash-guards.sh.
#
#   Warn-only, nunca rompe: si jq falla o el INPUT no es JSON valido, exit
#   0 sin salida -- igual que los tres scripts originales.
set -uo pipefail

SCRIPT_DIR="$(dirname "$0")"

INPUT="$(cat)"

# Pre-filtro UNICO (antes: uno por script, con su propio comentario
# identico en los tres -- ver la version larga de esa nota en
# git-c-bash.sh/git-add-all-guard.sh/verify-before-commit.sh, que se
# preservan como red de regresion). Las tres ramas exigen "git" literal en
# alguna parte de tool_input.command; si "git" no aparece en NINGUN sitio
# del JSON crudo de entrada, ninguna de las tres puede disparar -- se
# ahorra el UNICO spawn de jq compartido. Igual que en los originales: si
# "git" aparece en OTRO campo del JSON (cwd, session_id...) el filtro no
# descarta (falso negativo de disparo, imposible; como mucho se pierde una
# oportunidad de ahorro).
case "$INPUT" in
  *git*) : ;;
  *) exit 0 ;;
esac

command -v jq >/dev/null 2>&1 || exit 0

# Extraccion UNICA (antes: CMD se extraia tres veces -- una por script -- y
# SID/CWD/TRANSCRIPT una cuarta vez, solo en verify-before-commit.sh).
# NUL como separador de campos, leido directamente desde el stream de la
# process-substitution (nunca a traves de una variable intermedia): una
# variable bash NO puede almacenar un NUL embebido, y tool_input.command
# puede traer heredocs multilinea (con tabs/newlines reales) que un
# separador de linea rompería. `read -r -d ''` consume hasta el NUL o EOF;
# como jq emite un NUL final tras CADA campo (incluido el ultimo), las
# cuatro lecturas siempre tienen delimitador explicito.
CMD="" SID="" CWD="" TRANSCRIPT=""
{
  IFS= read -r -d '' CMD
  IFS= read -r -d '' SID
  IFS= read -r -d '' CWD
  IFS= read -r -d '' TRANSCRIPT
} < <(printf '%s' "$INPUT" | jq -j '
    (.tool_input.command // ""), "\u0000",
    (.session_id // ""), "\u0000",
    (.cwd // ""), "\u0000",
    (.transcript_path // ""), "\u0000"
  ' 2>/dev/null)

[ -z "$CMD" ] && exit 0

REWRITE_JSON=""

# =============================================================
# Rama 1: git-c -- de git-c-bash.sh (rewrite silencioso / warn-log)
# =============================================================
# ---- 1a: REWRITE silencioso (HECHO parseado, alta confianza) ----
# Solo si el comando ENTERO es `cd PATH && git REST` con PATH literal (sin
# quotes/vars/subst), REST sin metacaracteres de shell y subcomando git de
# SOLO LECTURA. Comentarios completos de cada excepcion en git-c-bash.sh
# (preservado como red de regresion) -- aqui se porta la logica tal cual.
REWRITE_ALLOWLIST='status|log|diff|show|rev-parse|describe|ls-files|blame|shortlog|reflog|grep'
REWRITE_RE='^cd[[:space:]]+([A-Za-z0-9._/~-]+)[[:space:]]*&&[[:space:]]*git[[:space:]]+(.+)$'
NEW_CMD=""
case "$CMD" in
  *$'\n'*) : ;;  # multiline: nunca rewrite
  *)
    if [[ "$CMD" =~ $REWRITE_RE ]]; then
      CD_PATH="${BASH_REMATCH[1]}"
      GIT_REST="${BASH_REMATCH[2]}"
      REWRITE_OK=0
      case "$CD_PATH" in
        -*) : ;;
        ..|../*|*/..|*/../*) : ;;
        [!/~]*) [ -z "${CDPATH:-}" ] && REWRITE_OK=1 ;;
        *) REWRITE_OK=1 ;;
      esac
      # shellcheck disable=SC2016 # `$` literal dentro de la clase de caracteres de grep
      if [ "$REWRITE_OK" = "1" ] \
         && ! printf '%s' "$GIT_REST" | grep -q '[&;|<>`$()*?[]' \
         && printf '%s' "$GIT_REST" | grep -Eq "^(${REWRITE_ALLOWLIST})([[:space:]]|\$)"; then
        NEW_CMD="git -C ${CD_PATH} ${GIT_REST}"
      fi
    fi
    ;;
esac

GITC_REWROTE=0
if [ -n "$NEW_CMD" ]; then
  RJ="$(printf '%s' "$INPUT" | jq -c --arg new "$NEW_CMD" \
      '{hookSpecificOutput:{hookEventName:"PreToolUse",
        permissionDecision:"allow",
        permissionDecisionReason:("Reflejo git-c: reescrito a `" + $new + "` (equivalente; evita el cd encadenado)"),
        updatedInput:(.tool_input | .command = $new)}}' 2>/dev/null)" || RJ=""
  if [ -n "$RJ" ]; then
    REWRITE_JSON="$RJ"
    GITC_REWROTE=1
    . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "git-c-rewrite" "$INPUT" "${CMD} -> ${NEW_CMD}" || true
  fi
  # si jq fallo (RJ vacio), cae a 1b (warn) -- never-break, igual que el original
fi

# ---- 1b: WARN (log-only), solo si 1a no reescribio -- igual que el
# original: si 1a reescribe e imprime, el proceso termina ahi y esta rama
# nunca se alcanza. ----
if [ "$GITC_REWROTE" -eq 0 ]; then
  if printf '%s' "$CMD" | grep -Eq 'cd[[:space:]]+[^&;|]+(&&|;)[[:space:]]*git([[:space:]]|$)'; then
    . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "git-c" "$INPUT" "$CMD" || true
  fi
fi

# =============================================================
# Rama 2: zero-residuo -- de git-add-all-guard.sh (log-only)
# =============================================================
PATRON_ZR='git[[:space:]]+(-C[[:space:]]+[^[:space:]]+[[:space:]]+)?add[[:space:]]+(-A|--all|\.)([[:space:]]|$)'
if printf '%s' "$CMD" | grep -Eq "$PATRON_ZR"; then
  PAYLOAD="${CMD:0:120}"
  . "$SCRIPT_DIR/_truncate-payload.sh" 2>/dev/null && payload_truncado "$CMD" "$PATRON_ZR"
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "zero-residuo" "$INPUT" "$PAYLOAD" || true
fi

# =============================================================
# Rama 3: verify-before-done -- de verify-before-commit.sh (log-only)
# =============================================================
PATRON_VBD='git[[:space:]]+commit([[:space:]]|$)'
if printf '%s' "$CMD" | grep -Eq "$PATRON_VBD" \
   && ! printf '%s' "$CMD" | grep -q -- '--no-verify'; then

  SID="${SID:-nosession}"  # sin uso posterior; se extrae por paridad con el original (ya era una var muerta alli)

  if [ -n "$CWD" ]; then
    STAGED="$(git -C "$CWD" diff --cached --name-only --diff-filter=ACM 2>/dev/null)" || STAGED=""
  else
    STAGED="$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null)" || STAGED=""
  fi

  if [ -n "$STAGED" ]; then
    CODE_FILES="$(printf '%s' "$STAGED" | grep -E '\.(ts|tsx|js|jsx|py|go|rs|rb|java|kt|c|cc|cpp|h|hpp|cs|php|swift|scala|sh|bash)$')" || CODE_FILES=""

    if [ -n "$CODE_FILES" ]; then
      SHOULD_WARN=1
      if [ -n "$TRANSCRIPT" ] && [ -f "$TRANSCRIPT" ]; then
        TR_RE1='(^|[&|;[:space:]]+)(pytest|jest|vitest|tox|rspec|phpunit)([[:space:]]|$)'
        TR_RE2='(^|[&|;[:space:]]+)(npm[[:space:]]+(test|run[[:space:]]+test)|yarn[[:space:]]+test|pnpm[[:space:]]+test|bun[[:space:]]+test|cargo[[:space:]]+test|go[[:space:]]+test|make[[:space:]]+test|mvn[[:space:]]+test|gradle[[:space:]]+test|dotnet[[:space:]]+test)([[:space:]]|$)'

        RESULT="$(tail -n 400 "$TRANSCRIPT" 2>/dev/null \
          | jq -rs --arg re1 "$TR_RE1" --arg re2 "$TR_RE2" '
              [ .[] | select(type=="object")
                | .message.content[]?
                | select(.type=="tool_use" and .name=="Bash")
                | {id: .id, cmd: (.input.command // "")}
              ] as $uses
              | ( [ .[] | select(type=="object")
                    | .message.content[]?
                    | select(.type=="tool_result")
                    | {key: .tool_use_id, value: (.is_error // false)}
                  ] | from_entries ) as $errmap
              | ( [ $uses[] | select((.cmd | test($re1)) or (.cmd | test($re2))) ] | last ) as $tr
              | if $tr == null then "warn"
                else (if $errmap[$tr.id] == false then "green" else "warn" end)
                end
            ' 2>/dev/null)" || RESULT="warn"

        [ "$RESULT" = "green" ] && SHOULD_WARN=0
      fi

      if [ "$SHOULD_WARN" -eq 1 ]; then
        PAYLOAD="${CMD:0:120}"
        . "$SCRIPT_DIR/_truncate-payload.sh" 2>/dev/null && payload_truncado "$CMD" "$PATRON_VBD"
        . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "verify-before-done" "$INPUT" "$PAYLOAD" || true
      fi
    fi
  fi
fi

# ---- Emision fusionada: el rewrite (si lo hay) es lo UNICO que va a
# stdout -- ver REGLA DE FUSION arriba. ----
if [ -n "$REWRITE_JSON" ]; then
  printf '%s' "$REWRITE_JSON"
fi
exit 0
