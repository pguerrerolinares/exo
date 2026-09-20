#!/usr/bin/env bash
# Gate: si `plugins/exo/**` cambia respecto a BASE, `version` tiene que cambiar
# con él.
#
# El marketplace de Claude Code sirve el plugin por NÚMERO de versión, no por
# contenido: publicar contenido nuevo bajo un número ya instalado es
# indistinguible de no publicar nada. `/plugin` contesta «already at the latest
# version» y quien pregunta se queda con el plugin viejo creyendo lo contrario
# — un check que no es falsable respecto de lo que le importa.
#
# Medido: `1.2.0` se declaró el 2026-09-15 (commit 5353038) y hasta el
# 2026-09-20 recibió 22 commits más bajo el mismo número. La caché de la
# máquina de Paul siguió sirviendo el contenido del 16 de septiembre —
# campaña I entera sin ejecutarse — y nada lo dijo.
#
# BASE: la base del PR, o el commit anterior en un push a main. Sin ella el
# gate se abstiene, y lo dice: un gate que calla cuando no puede mirar es
# indistinguible de uno que miró y no vio nada.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1

BASE="${BASE:-}"
PLUGIN_JSON="plugins/exo/.claude-plugin/plugin.json"

if [ -z "$BASE" ]; then
  echo "plugin-bump-gate: sin BASE, no hay con qué comparar — me abstengo"
  exit 0
fi
if ! git rev-parse --verify -q "$BASE^{commit}" >/dev/null 2>&1; then
  echo "plugin-bump-gate: BASE '$BASE' no resuelve a un commit — me abstengo"
  exit 0
fi

cambios="$(git diff --name-only "$BASE" -- plugins/exo/)"
if [ -z "$cambios" ]; then
  echo "plugin-bump-gate: OK — plugins/exo/ no cambia respecto a $BASE"
  exit 0
fi

v_base="$(git show "$BASE:$PLUGIN_JSON" 2>/dev/null | jq -r '.version // empty')"
v_head="$(jq -r '.version // empty' < "$PLUGIN_JSON")"

if [ -z "$v_head" ]; then
  echo "plugin-bump-gate: $PLUGIN_JSON no declara version" >&2
  exit 1
fi
if [ -z "$v_base" ]; then
  echo "plugin-bump-gate: OK — el plugin no existía en $BASE (version $v_head)"
  exit 0
fi
if [ "$v_base" != "$v_head" ]; then
  echo "plugin-bump-gate: OK — plugins/exo/ cambia y version también: $v_base → $v_head"
  exit 0
fi

n="$(printf '%s\n' "$cambios" | wc -l | tr -d ' ')"
{
  echo "plugin-bump-gate: $n fichero(s) de plugins/exo/ cambian y version sigue en $v_head."
  echo
  printf '%s\n' "$cambios" | sed 's/^/  /'
  echo
  echo "  El marketplace sirve por número: bajo un $v_head ya publicado, esto no le"
  echo "  llega a nadie — \`/plugin\` dirá «already at the latest version» y seguirá"
  echo "  sirviendo el contenido viejo, sin avisar."
  echo
  echo "  Sube \"version\" en $PLUGIN_JSON y en .claude-plugin/marketplace.json"
  echo "  (scripts/test-versiones.sh exige que coincidan)."
} >&2
exit 1
