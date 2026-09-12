#!/usr/bin/env bash
# Gate: todo *.sh versionado bajo plugins/ debe estar en el índice como 100755.
#
# Los hooks de plugins/exo/hooks/hooks.json invocan los scripts directamente
# ("${CLAUDE_PLUGIN_ROOT}"/scripts/x.sh), sin `bash` delante. Un script
# commiteado en 100644 llega así al cache del plugin y el hook muere con
# `Permission denied` (exit 126) — non-blocking, así que la sesión arranca
# igual y nadie lo ve. Pasó con estilo-directo.sh (4e14edc) en exo 1.1.1.
#
# Mira el índice de git, no el working tree: lo que se publica es el modo
# commiteado, y un `chmod +x` local sin stagear lo taparía.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

malos="$(git ls-files -s -- 'plugins/*.sh' | awk '$1 != "100755" { print $1, $4 }')"

if [[ -n "$malos" ]]; then
  echo "[FAIL] scripts sin bit de ejecución en el índice (esperado 100755):" >&2
  echo "$malos" >&2
  echo "Arreglo: git update-index --chmod=+x <fichero>" >&2
  exit 1
fi

total="$(git ls-files -- 'plugins/*.sh' | wc -l)"
if [[ "$total" -eq 0 ]]; then
  # Un glob que no matchea nada daría verde sin significado.
  echo "[FAIL] no se encontró ningún plugins/*.sh — el glob está roto" >&2
  exit 1
fi

echo "[OK] ${total} scripts bajo plugins/ en 100755"
