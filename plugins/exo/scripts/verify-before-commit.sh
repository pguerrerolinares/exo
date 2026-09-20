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

# Pre-filtro bash puro (campaña I): el PATRON de este reflejo exige "git
# commit" literal. Mismo contrato que git-c-bash.sh: sin "git" en el JSON
# crudo, se ahorra el spawn de jq sin poder perder ningún disparo real.
case "$INPUT" in
  *git*) : ;;
  *) exit 0 ;;
esac

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

# payload del log: contexto + TODAS las ocurrencias del PATRON (mismo PATRON
# de arriba: si diverge del de deteccion, el log deja de decir por que
# disparo). Contrato y trampas en _truncate-payload.sh. Si el helper no se
# puede cargar, degrada al prefijo: el reflejo es warn-only y nunca rompe.
PAYLOAD="${CMD:0:120}"
. "$(dirname "$0")/_truncate-payload.sh" 2>/dev/null && payload_truncado "$CMD" "$PATRON"

# log del disparo (best-effort, nunca rompe el warn-only)
. "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "verify-before-done" "$INPUT" "$PAYLOAD" || true

exit 0
