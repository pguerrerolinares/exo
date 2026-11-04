#!/usr/bin/env bash
# Gate: plugins/exo/hooks/hooks.json no referencia eventos desconocidos,
# hooks de tipo distinto de "command", ni scripts que no existen o no están
# en 100755 — la clase de bug que ya cazó test-exec-bit.sh para el resto del
# plugin, aplicada a lo que hooks.json declara.
#
# Sin dependencia nueva: en vez de JSON Schema + Ajv (precedente de ECC,
# docs/backlog.md, ítem "Rutas personales y `hooks.json` sin validar en CI —
# las dos sub-propuestas vivas del item de los `test-*.sh` del plugin" —
# cítalo por título, no por línea: el sync de backlog.md desplaza líneas —
# que exige Node/npm, ausentes de este repo), las mismas comprobaciones
# concretas con jq, que ya es una dependencia del plugin.
#
# jq en Windows/Git Bash emite CRLF: cada salida que se compara o se lee
# línea a línea pasa por `tr -d '\r'` — sin eso, "PreToolUse\r" no es igual
# a "PreToolUse" y el gate falla en todo, no en nada.
set -uo pipefail
# `cd "$(git rev-parse --show-toplevel)"` directo tiene un fallo silencioso: si la
# sustitución sale vacía, `cd ""` devuelve 0 sin moverse y el `|| exit 1` nunca
# dispara (mismo fix que en test-docs-vivos.sh, review final de F 2026-09-16).
RAIZ="$(git rev-parse --show-toplevel)" || exit 1
if [ -z "$RAIZ" ]; then
  echo "test-hooks-json: git rev-parse --show-toplevel no devolvió nada" >&2
  exit 1
fi
cd "$RAIZ" || exit 1

HOOKS=plugins/exo/hooks/hooks.json
command -v jq >/dev/null 2>&1 || { echo "test-hooks-json: jq requerido" >&2; exit 1; }
[ -f "$HOOKS" ] || { echo "test-hooks-json: no existe $HOOKS" >&2; exit 1; }
jq -e . "$HOOKS" >/dev/null 2>&1 || { echo "test-hooks-json: $HOOKS no es JSON válido" >&2; exit 1; }

# I5 (review final, 2026-09-16): con `.hooks` ausente o `{}`, los tres bucles
# de abajo leen cero líneas cada uno — cero hallazgos no es lo mismo que cero
# problemas, y el gate seguía en verde sin haber mirado ni un hook. Mismo
# estilo de guarda que la de "no se encontró ningún script" en
# scripts/test-shellcheck.sh.
N_HOOKS="$(jq -r '[.hooks[]?[]?.hooks[]?] | length' "$HOOKS" | tr -d '\r')"
if [ "$N_HOOKS" -eq 0 ]; then
  echo "test-hooks-json: cero hooks en $HOOKS — el recorrido está roto, o .hooks está vacío/ausente" >&2
  exit 1
fi

FALLOS=0
EVENTOS_VALIDOS="PreToolUse PostToolUse PostToolUseFailure SessionStart Stop SubagentStart SubagentStop"

for evento in $(jq -r '.hooks | keys[]' "$HOOKS" | tr -d '\r'); do
  if ! printf '%s\n' "$EVENTOS_VALIDOS" | tr ' ' '\n' | grep -qx "$evento"; then
    echo "[FAIL] $HOOKS: evento desconocido '$evento'" >&2
    FALLOS=1
  fi
done

# El recall por prompt se borró (campaña K): ningún hook debe volver a colgarse
# de UserPromptSubmit sin decisión explícita.
if [ "$(jq '.hooks | has("UserPromptSubmit")' "$HOOKS" | tr -d '\r')" != "false" ]; then
  echo "[FAIL] $HOOKS: sobrevive la clave UserPromptSubmit (hook de recall por prompt borrado)" >&2
  FALLOS=1
fi

while IFS= read -r tipo; do
  if [ "$tipo" != "command" ]; then
    echo "[FAIL] $HOOKS: un hook con type='$tipo' (solo se soporta 'command')" >&2
    FALLOS=1
  fi
done < <(jq -r '.hooks[][].hooks[].type' "$HOOKS" | tr -d '\r')

# `exo hook claude <evento> || exit 0` es el cableado del núcleo: el `|| exit 0`
# es lo que impide que un `exo` ausente o viejo bloquee la herramienta (exit 127),
# y `timeout` 5 acota el coste de un hook colgado. SessionStart es la excepción:
# lleva `--min <ENGINE_MIN>` y NO `|| exit 0` (el propio `exo` avisa del engine
# viejo; con un `exo` ausente el hook falla visible), con timeout 10.
while IFS=$'\t' read -r cmd timeout; do
  if printf '%s' "$cmd" | grep -qE '^exo hook '; then
    if printf '%s' "$cmd" | grep -qE '^exo hook claude SessionStart'; then
      if ! printf '%s' "$cmd" | grep -qE '^exo hook claude SessionStart --min [0-9]+\.[0-9]+\.[0-9]+$'; then
        echo "[FAIL] $HOOKS: SessionStart debe ser 'exo hook claude SessionStart --min X.Y.Z' sin '|| exit 0': $cmd" >&2
        FALLOS=1
      fi
      if [ "$timeout" != "10" ]; then
        echo "[FAIL] $HOOKS: '$cmd' sin timeout 10 (timeout='$timeout')" >&2
        FALLOS=1
      fi
      continue
    fi
    if ! printf '%s' "$cmd" | grep -qE '^exo hook claude (PreToolUse|PostToolUse|PostToolUseFailure|Stop) \|\| exit 0$'; then
      echo "[FAIL] $HOOKS: command exo hook no casa con 'exo hook claude <evento> || exit 0': $cmd" >&2
      FALLOS=1
    fi
    if [ "$timeout" != "5" ]; then
      echo "[FAIL] $HOOKS: '$cmd' sin timeout 5 (timeout='$timeout')" >&2
      FALLOS=1
    fi
    continue
  fi
  script="$(printf '%s' "$cmd" | grep -oE '\$\{CLAUDE_PLUGIN_ROOT\}"?/scripts/[A-Za-z0-9_.-]+' | sed -E 's#.*/scripts/##')"
  if [ -z "$script" ]; then
    echo "[FAIL] $HOOKS: command sin \${CLAUDE_PLUGIN_ROOT}/scripts/<x> reconocible: $cmd" >&2
    FALLOS=1
    continue
  fi
  ruta="plugins/exo/scripts/$script"
  if [ ! -f "$ruta" ]; then
    echo "[FAIL] $HOOKS: $ruta no existe (citado por hooks.json)" >&2
    FALLOS=1
    continue
  fi
  modo="$(git ls-files -s -- "$ruta" | awk '{print $1}')"
  if [ "$modo" != "100755" ]; then
    echo "[FAIL] $HOOKS: $ruta no está en 100755 en el índice (modo=$modo)" >&2
    FALLOS=1
  fi
done < <(jq -r '.hooks[][].hooks[] | [.command, (.timeout // "" | tostring)] | @tsv' "$HOOKS" | tr -d '\r')

# Cutover de recall/index: SessionStart y Stop tienen que estar cableados al
# núcleo, no a los scripts borrados.
for ev in SessionStart Stop; do
  n="$(jq -r --arg e "$ev" '[.hooks[$e][]?.hooks[]? | select(.command | startswith("exo hook claude " + $e))] | length' "$HOOKS" | tr -d '\r')"
  if [ "$n" != "1" ]; then
    echo "[FAIL] $HOOKS: $ev debe llevar exactamente un 'exo hook claude $ev' (hay $n)" >&2
    FALLOS=1
  fi
done

if [ "$FALLOS" -eq 0 ]; then
  echo "[OK] test-hooks-json: eventos, sin UserPromptSubmit, type=command, exo hook (SessionStart --min sin || exit 0 y timeout 10; resto || exit 0 y timeout 5), scripts referenciados 100755"
fi
exit "$FALLOS"
