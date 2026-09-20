#!/usr/bin/env bash
# Gate de scripts/plugin-bump-gate.sh, contra repos git de juguete.
#
# Lo que vigila el gate: `plugins/exo/**` cambió y `version` no. El
# marketplace sirve por NÚMERO, así que un contenido nuevo bajo un número ya
# publicado no le llega a nadie — `/plugin` responde «already at the latest
# version» y el que lo pregunta se queda con el plugin viejo creyendo que
# está al día. Pasó de verdad: 22 commits de la campaña I bajo `1.2.0`, cuatro
# días sirviéndose el contenido del 15 de septiembre.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1

GATE="$PWD/scripts/plugin-bump-gate.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
fallos=0

ok()  { echo "test-plugin-bump: OK — $1"; }
mal() { echo "test-plugin-bump: FALLO — $1" >&2; fallos=1; }

# fixture <nombre> -> repo con un commit base: plugin 1.0.0 y un script.
# Deja $REPO y $BASE (el sha del commit base).
fixture() {
  REPO="$TMP/$1"
  mkdir -p "$REPO/plugins/exo/.claude-plugin" "$REPO/plugins/exo/scripts" "$REPO/scripts"
  cp "$GATE" "$REPO/scripts/"
  printf '{\n  "name": "exo",\n  "version": "1.0.0"\n}\n' > "$REPO/plugins/exo/.claude-plugin/plugin.json"
  printf 'echo viejo\n' > "$REPO/plugins/exo/scripts/algo.sh"
  printf 'nada que ver\n' > "$REPO/otro.txt"
  git -C "$REPO" init -q
  git -C "$REPO" config user.email test@exo.invalid
  git -C "$REPO" config user.name test
  git -C "$REPO" add plugins otro.txt scripts
  git -C "$REPO" commit -qm base
  BASE="$(git -C "$REPO" rev-parse HEAD)"
}

# corre <repo> <base> -> $rc y $TMP/out.log
corre() {
  ( cd "$1" && BASE="$2" bash scripts/plugin-bump-gate.sh ) > "$TMP/out.log" 2>&1
  rc=$?
}

# --- Caso 1: el plugin cambia y la versión se queda quieta -> rojo.
fixture cambio-sin-bump
printf 'echo nuevo\n' > "$REPO/plugins/exo/scripts/algo.sh"
git -C "$REPO" add plugins/exo/scripts/algo.sh
git -C "$REPO" commit -qm "cambia un script del plugin"
corre "$REPO" "$BASE"
if [ "$rc" -eq 0 ]; then
  mal "cambio sin bump: salió 0 — es justo el caso que dejó 1.2.0 con dos contenidos"
elif ! grep -q "algo.sh" "$TMP/out.log"; then
  mal "cambio sin bump: el error no dice qué fichero cambió"; cat "$TMP/out.log" >&2
elif ! grep -q "1.0.0" "$TMP/out.log"; then
  mal "cambio sin bump: el error no dice en qué versión se quedó"; cat "$TMP/out.log" >&2
else
  ok "plugins/exo/ cambia y version no ⇒ rojo, diciendo fichero y versión"
fi

# --- Caso 2: el plugin cambia y la versión sube -> verde.
fixture cambio-con-bump
printf 'echo nuevo\n' > "$REPO/plugins/exo/scripts/algo.sh"
printf '{\n  "name": "exo",\n  "version": "1.1.0"\n}\n' > "$REPO/plugins/exo/.claude-plugin/plugin.json"
git -C "$REPO" add plugins/exo/scripts/algo.sh plugins/exo/.claude-plugin/plugin.json
git -C "$REPO" commit -qm "cambia el plugin y sube la versión"
corre "$REPO" "$BASE"
if [ "$rc" -ne 0 ]; then
  mal "cambio con bump: esperaba 0, hubo $rc"; cat "$TMP/out.log" >&2
else
  ok "plugins/exo/ cambia y version sube ⇒ verde"
fi

# --- Caso 3: cambia algo de fuera del plugin -> el gate no opina.
fixture cambio-fuera
printf 'otra cosa\n' > "$REPO/otro.txt"
git -C "$REPO" add otro.txt
git -C "$REPO" commit -qm "cambia algo que no es el plugin"
corre "$REPO" "$BASE"
if [ "$rc" -ne 0 ]; then
  mal "cambio fuera del plugin: esperaba 0, hubo $rc"; cat "$TMP/out.log" >&2
else
  ok "cambios fuera de plugins/exo/ ⇒ el gate no opina"
fi

# --- Caso 4: sin BASE -> se abstiene, pero LO DICE. Un gate que calla cuando
# no puede mirar es indistinguible de un gate que miró y no vio nada.
fixture sin-base
corre "$REPO" ""
if [ "$rc" -ne 0 ]; then
  mal "sin BASE: esperaba abstención con 0, hubo $rc"; cat "$TMP/out.log" >&2
elif ! grep -qi "abstien\|abstención\|sin base" "$TMP/out.log"; then
  mal "sin BASE: se abstuvo en silencio"; cat "$TMP/out.log" >&2
else
  ok "sin BASE ⇒ abstención explícita, exit 0"
fi

# --- Caso 5: BASE que no es un commit (el 000000… del primer push de una
# rama) -> abstención, no un revent'on que tiña de rojo un push legítimo.
fixture base-invalida
corre "$REPO" 0000000000000000000000000000000000000000
if [ "$rc" -ne 0 ]; then
  mal "BASE inválida: esperaba abstención con 0, hubo $rc"; cat "$TMP/out.log" >&2
elif ! grep -qi "abstien\|abstención\|no existe\|no resuelve" "$TMP/out.log"; then
  mal "BASE inválida: se abstuvo en silencio"; cat "$TMP/out.log" >&2
else
  ok "BASE que no resuelve a un commit ⇒ abstención explícita, exit 0"
fi

[ "$fallos" -eq 0 ] || { echo "test-plugin-bump: hay fallos" >&2; exit 1; }
echo "test-plugin-bump: OK — los cinco casos"
