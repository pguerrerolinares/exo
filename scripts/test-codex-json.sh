#!/usr/bin/env bash
# Gate: plugins/exo/hooks/codex.json. Codex lo parsea con deny_unknown_fields: una clave de más
# tumba todos los hooks del plugin, y un evento que no es de Codex no se dispara nunca.
set -uo pipefail
RAIZ="$(git rev-parse --show-toplevel)" || exit 1
if [ -z "$RAIZ" ]; then
  echo "test-codex-json: git rev-parse --show-toplevel no devolvió nada" >&2
  exit 1
fi
cd "$RAIZ" || exit 1

F=plugins/exo/hooks/codex.json
command -v jq >/dev/null 2>&1 || { echo "test-codex-json: jq requerido" >&2; exit 1; }
[ -f "$F" ] || { echo "test-codex-json: no existe $F" >&2; exit 1; }
jq -e . "$F" >/dev/null 2>&1 || { echo "test-codex-json: $F no es JSON válido" >&2; exit 1; }

# jq en Windows/Git Bash emite CRLF: cada salida leída por líneas pasa por tr -d '\r'.
n="$(jq -r '[.hooks[]?[]?.hooks[]?] | length' "$F" | tr -d '\r')"
if [ "$n" -eq 0 ]; then
  echo "test-codex-json: cero hooks en $F: el recorrido está roto" >&2
  exit 1
fi

FALLOS=()
EVENTOS="SessionStart UserPromptSubmit PreToolUse PostToolUse PostCompact Stop"
CLAVES="type command timeout additionalContextLimit statusMessage commandWindows async"

while IFS= read -r ev; do
  case " $EVENTOS " in *" $ev "*) ;; *) FALLOS+=("evento que no es de Codex: $ev") ;; esac
done < <(jq -r '.hooks | keys[]' "$F" | tr -d '\r')

while IFS=$'\t' read -r ev clave; do
  case " $CLAVES " in *" $clave "*) ;; *) FALLOS+=("$ev: clave desconocida en el handler: $clave") ;; esac
  if [ "$clave" = "additionalContextLimit" ] && [ "$ev" != "SessionStart" ] && [ "$ev" != "UserPromptSubmit" ]; then
    FALLOS+=("$ev: additionalContextLimit solo vale en SessionStart y UserPromptSubmit")
  fi
done < <(jq -r '.hooks | to_entries[] | .key as $e | .value[].hooks[] | keys[] | [$e, .] | @tsv' "$F" | tr -d '\r')

while IFS=$'\t' read -r ev tipo cmd; do
  [ "$tipo" = "command" ] || FALLOS+=("$ev: type='$tipo' (solo 'command')")
  case "$ev" in
    SessionStart)
      case "$cmd" in *"|| exit 0"*) FALLOS+=("SessionStart no lleva '|| exit 0': tiene que fallar en voz alta ($cmd)") ;; esac ;;
    *)
      case "$cmd" in *" || exit 0") ;; *) FALLOS+=("$ev: el comando no acaba en '|| exit 0': $cmd") ;; esac ;;
  esac
done < <(jq -r '.hooks | to_entries[] | .key as $e | .value[].hooks[] | [$e, .type, .command] | @tsv' "$F" | tr -d '\r')

if [ "${#FALLOS[@]}" -eq 0 ]; then
  echo "OK"
  exit 0
fi
for f in "${FALLOS[@]}"; do echo "[FAIL] $F: $f" >&2; done
exit 1
