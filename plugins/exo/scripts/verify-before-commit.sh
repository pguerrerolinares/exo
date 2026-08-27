#!/usr/bin/env bash
# PreToolUse (matcher: Bash): reflejo "verify-before-done".
# Avisa (warn-only) cuando el agente intenta hacer un git commit de codigo sin
# haber ejecutado un test verde reciente en la sesion actual.
#
# LOGICA:
#   1. El comando debe contener `git commit` (como invocacion, no como dato).
#   2. Si lleva --no-verify -> abstension (escape hatch respetado).
#   3. Filtro codigo-vs-docs: si no hay ficheros de codigo en el staging area -> silencio.
#   4. Detecta "test verde reciente" escaneando la cola del transcript JSONL
#      (tail -n 400; misma tecnica que stuck-loop-pretool.sh). Busca el ULTIMO
#      tool_use Bash que sea un test-runner y mira el is_error de su tool_result.
#      Dispara si: no hay transcript, ningun test-runner en la cola, o el mas
#      reciente salio is_error:true / sin resultado correlacionable.
#   5. Si el test-runner mas reciente salio verde (is_error:false) -> silencio.
#
# POR QUE TRANSCRIPT-SCAN (no fichero de estado): el payload real de PostToolUse
# NO trae exit_code, asi que el viejo tracker nunca escribia estado y esta rama
# estaba muerta (siempre avisaba). El bound tail -n 400 actua como ventana de recencia.
#
# Por-ocurrencia (sin sentinel): cada commit de codigo sin verificar es un evento real.
# Aplica en padre Y subagentes (sin guarda agent_id).
#
# LOG: el payload persistido es contexto (primeros ~120 chars) + TODAS las
# invocaciones "git commit" (hasta un techo, dedup) que de verdad
# dispararon, extraidas con el MISMO patron que la deteccion (via
# grep -Eo). No solo la primera: con 2+ ocurrencias en el mismo comando la
# que ejecuta de verdad puede vivir al final (p.ej. una mencion en prosa
# dentro de un heredoc seguida del "git commit" real), y quedarse con
# head -1 le esconde esa segunda ocurrencia al log. Comando corto que cabe
# entero en esos ~120 chars se loguea tal cual, sin marcador.
set -uo pipefail

INPUT="$(cat)"
command -v jq >/dev/null 2>&1 || exit 0

CMD="$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)" || CMD=""
[ -z "$CMD" ] && exit 0

# 1. Solo actuar en git commit.
PATRON='git[[:space:]]+commit([[:space:]]|$)'
printf '%s' "$CMD" | grep -Eq "$PATRON" || exit 0

# 2. Escape hatch: --no-verify -> abstension.
printf '%s' "$CMD" | grep -q -- '--no-verify' && exit 0

# 3. Filtro codigo-vs-docs: ejecuta git diff --cached para ver que hay staged.
SID="$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)" || SID=""
SID="${SID:-nosession}"

# Intentar obtener el cwd del INPUT para correr git en el directorio correcto.
CWD_INPUT="$(printf '%s' "$INPUT" | jq -r '.cwd // empty' 2>/dev/null)" || CWD_INPUT=""
if [ -n "$CWD_INPUT" ]; then
  STAGED="$(git -C "$CWD_INPUT" diff --cached --name-only --diff-filter=ACM 2>/dev/null)" || STAGED=""
else
  STAGED="$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null)" || STAGED=""
fi

# Si no hay nada staged -> silencio (commit vacio o todo ya committed).
[ -z "$STAGED" ] && exit 0

# Filtrar por extensiones de codigo.
CODE_FILES="$(printf '%s' "$STAGED" | grep -E '\.(ts|tsx|js|jsx|py|go|rs|rb|java|kt|c|cc|cpp|h|hpp|cs|php|swift|scala|sh|bash)$')" || CODE_FILES=""

# Si no hay ficheros de codigo staged -> silencio (commit de docs/markdown).
[ -z "$CODE_FILES" ] && exit 0

# 4. Detectar "test verde reciente" escaneando la cola del transcript JSONL.
#    Default: avisa (no se puede confirmar verde -> warn), consistente con el
#    comportamiento previo de "sin estado -> avisa".
SHOULD_WARN=1

TRANSCRIPT="$(printf '%s' "$INPUT" | jq -r '.transcript_path // empty' 2>/dev/null)" || TRANSCRIPT=""

if [ -n "$TRANSCRIPT" ] && [ -f "$TRANSCRIPT" ]; then
  # Regexes test-runner (copiados de test-run-tracker.sh).
  TR_RE1='(^|[&|;[:space:]]+)(pytest|jest|vitest|tox|rspec|phpunit)([[:space:]]|$)'
  TR_RE2='(^|[&|;[:space:]]+)(npm[[:space:]]+(test|run[[:space:]]+test)|yarn[[:space:]]+test|pnpm[[:space:]]+test|bun[[:space:]]+test|cargo[[:space:]]+test|go[[:space:]]+test|make[[:space:]]+test|mvn[[:space:]]+test|gradle[[:space:]]+test|dotnet[[:space:]]+test)([[:space:]]|$)'

  # Una pasada de jq (estilo stuck-loop-pretool.sh):
  #   $uses   = lista ordenada de tool_use Bash {id, cmd}
  #   $errmap = tool_use_id -> is_error (ausente == false, robusto a exitos sin el campo)
  #   $tr     = el ULTIMO (mas reciente) tool_use cuyo cmd matchea un test-runner
  # Emite "green" solo si ese tiene resultado correlacionado is_error:false; si no, "warn".
  # Los 2>/dev/null son redirects de shell, NO codigo jq.
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

# 5. Si todo ok -> silencio.
[ "$SHOULD_WARN" -eq 0 ] && exit 0

# payload del log: prefijo de contexto + TODAS las ocurrencias del PATRON
# (mismo PATRON de arriba via grep -Eo), hasta MATCH_MAX, deduplicadas
# preservando orden. Comando corto que cabe entero en el prefijo -> se
# loguea entero, sin marcador. Extraccion sin match (no deberia pasar) ->
# degrada al prefijo solo.
# OJO: el prefijo se saca con expansion de parametro (${CMD:0:120}), NO con
# `cut -c`. `cut -c` trunca POR LINEA, no el string completo -- con un
# comando de muchas lineas cortas cada una sobrevive intacta y el "prefijo"
# real acaba siendo lineas*120 caracteres, reventando el cap del helper y
# comiendose el match otra vez. Es el mismo bug que este fix vino a arreglar.
# OJO 2: el propio PATRON no tiene techo (`git[[:space:]]+commit` acepta
# cualquier cantidad de espacios), asi que ningun MATCH extraido lo tiene
# tampoco. Un match gigante puede por si solo topar el cap de 2000 del
# helper y comerse el "commit" -- la misma brecha, otra puerta. Por eso
# CADA ocurrencia se trunca POR SEPARADO (cabeza+cola, NO solo el
# principio: el fragmento que informa vive al FINAL del match, truncar
# solo por delante lo tiraria) antes de unirla a las demas: una sola
# ocurrencia larga no puede comerse el cap combinado.
# OJO 3: por que TODAS y no solo la primera (`head -1`, el bug de esta
# ronda). Con 2+ ocurrencias del patron en el mismo comando -- p.ej. una
# mencion en prosa dentro de un heredoc ("recuerda hacer git commit al
# final") seguida del "git commit" real -- quedarse con la primera loguea
# la mencion inocua y esconde la invocacion real: exactamente el falso
# positivo benigno que este instrumento existe para medir, entrando por
# otra puerta. Ocurrencias identicas se deduplican preservando orden:
# repetir el mismo texto no informa mas que mostrarlo una vez.
# Nota aparte (no ataja nada, solo lo documenta): estos cortes por indice
# (aqui, PREFIJO y el cap de _reflex-log.sh) cuentan caracteres en locale
# UTF-8 pero bytes en LC_ALL=C -- bajo esa locale el corte puede caer a
# mitad de un caracter multibyte. jq lo tolera (sustituye por el caracter
# de reemplazo, exit 0) y el contrato best-effort aguanta, asi que no hace
# falta blindarlo.
MATCH_HEAD=80
MATCH_TAIL=60
MATCH_MAX=5
if [ "${#CMD}" -le 120 ]; then
  PAYLOAD="$CMD"
else
  PREFIJO="${CMD:0:120}"
  MATCHES="$(printf '%s' "$CMD" | grep -Eo "$PATRON" | head -n "$MATCH_MAX" | awk '!seen[$0]++')"
  MATCH=""
  while IFS= read -r M; do
    [ -z "$M" ] && continue
    if [ "${#M}" -gt $((MATCH_HEAD + MATCH_TAIL)) ]; then
      M="${M:0:MATCH_HEAD}…${M: -MATCH_TAIL}"
    fi
    if [ -z "$MATCH" ]; then
      MATCH="$M"
    else
      MATCH="${MATCH} | ${M}"
    fi
  done <<< "$MATCHES"
  if [ -n "$MATCH" ]; then
    PAYLOAD="${PREFIJO} … ⟨match⟩ ${MATCH}"
  else
    PAYLOAD="$PREFIJO"
  fi
fi

# log del disparo (best-effort, nunca rompe el warn-only)
. "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "verify-before-done" "$INPUT" "$PAYLOAD" || true

exit 0
