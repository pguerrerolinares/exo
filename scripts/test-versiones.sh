#!/usr/bin/env bash
# Gate: los números de versión publicados no se contradicen entre sí.
#
# exo publica DOS artefactos con ciclo propio (decisión D3 de la campaña B):
#   - el engine: `engine/Cargo.toml` [package] version == tag de release `vX.Y.Z`
#     (lo que imprime `exo --version`);
#   - el plugin: `plugins/exo/.claude-plugin/plugin.json` .version ==
#     `.claude-plugin/marketplace.json` .plugins[0].version (lo que Claude Code
#     usa para detectar actualizaciones).
# Sin gate, el mismo número vivía en dos ficheros a mano y la metadata del
# marketplace llegó a decir 1.0.0 con el plugin en 1.1.2.
#
# Uso: scripts/test-versiones.sh            # coherencia del árbol
#      scripts/test-versiones.sh v0.2.0     # además, el tag casa con Cargo.toml
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

fallos=0
plugin="$(jq -r '.version // empty' plugins/exo/.claude-plugin/plugin.json)"
market="$(jq -r '.plugins[] | select(.name == "exo") | .version // empty' .claude-plugin/marketplace.json)"
engine="$(sed -n 's/^version = "\(.*\)"$/\1/p' engine/Cargo.toml | head -n 1)"

if [ -z "$plugin" ] || [ "$plugin" != "$market" ]; then
  echo "[FAIL] plugin.json dice '$plugin' y marketplace.json dice '$market'" >&2
  fallos=1
fi
if jq -e '.metadata.version' .claude-plugin/marketplace.json >/dev/null; then
  echo "[FAIL] marketplace.json lleva metadata.version: un tercer número sin dueño. Quítalo." >&2
  fallos=1
fi
if [ -z "$engine" ]; then
  echo "[FAIL] no leo la versión de engine/Cargo.toml" >&2
  fallos=1
fi
if [ "$#" -ge 1 ] && [ "$1" != "v$engine" ]; then
  echo "[FAIL] el tag '$1' no casa con engine/Cargo.toml ($engine): el binario diría otra versión" >&2
  fallos=1
fi

[ "$fallos" -eq 0 ] && echo "[OK] engine $engine · plugin $plugin"
exit "$fallos"
