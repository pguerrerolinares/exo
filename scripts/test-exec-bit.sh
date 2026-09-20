#!/usr/bin/env bash
# Gate: todo script versionado bajo plugins/ debe estar en el índice como 100755.
# "Script" = termina en .sh O su contenido empieza por `#!` (shebang).
#
# Los hooks de plugins/exo/hooks/hooks.json invocan los scripts directamente
# ("${CLAUDE_PLUGIN_ROOT}"/scripts/x.sh), sin `bash` delante. Un script
# commiteado en 100644 llega así al cache del plugin y el hook muere con
# `Permission denied` (exit 126) — non-blocking, así que la sesión arranca
# igual y nadie lo ve. Pasó con estilo-directo.sh (4e14edc) en exo 1.1.1.
#
# Por qué también el shebang y no solo `*.sh`: los scripts de
# skills/orchestrate/scripts/ (review-package, sdd-workspace, task-brief) no
# llevan extensión y la skill los invoca por ruta. Con el filtro `*.sh` de la
# primera versión de este gate quedaban fuera sin que nada lo dijera.
#
# Mira el índice de git, no el working tree: lo que se publica es el modo
# commiteado, y un `chmod +x` local sin stagear lo taparía. El contenido
# también sale del índice (`git cat-file`), por la misma razón.
set -euo pipefail

# Campaña L Task 4 (backlog:1691, mismo fix que 960a319): `cd
# "$(git rev-parse --show-toplevel)"` directo tiene un fallo silencioso — si
# la sustitución sale vacía, `cd ""` devuelve 0 sin moverse, y bajo
# `set -e` eso NO aborta (el exit code de `cd ""` es 0). Captura la raíz, la
# comprueba y entonces se mueve.
RAIZ="$(git rev-parse --show-toplevel)" || exit 1
if [ -z "$RAIZ" ]; then
  echo "test-exec-bit: git rev-parse --show-toplevel no devolvió nada" >&2
  exit 1
fi
cd "$RAIZ" || exit 1

malos=""
total=0
while read -r modo blob _etapa ruta; do
  es_script=0
  case "$ruta" in
    *.sh) es_script=1 ;;
    *) [ "$(git cat-file -p "$blob" | head -c 2)" = "#!" ] && es_script=1 ;;
  esac
  [ "$es_script" -eq 1 ] || continue
  total=$((total + 1))
  if [ "$modo" != "100755" ]; then
    malos="${malos}${modo} ${ruta}"$'\n'
  fi
done < <(git ls-files -s -- plugins)

if [[ -n "$malos" ]]; then
  echo "[FAIL] scripts sin bit de ejecución en el índice (esperado 100755):" >&2
  printf '%s' "$malos" >&2
  echo "Arreglo: git update-index --chmod=+x <fichero>" >&2
  exit 1
fi

if [[ "$total" -eq 0 ]]; then
  # Un recorrido que no encuentra nada daría verde sin significado.
  echo "[FAIL] no se encontró ningún script bajo plugins/ — el recorrido está roto" >&2
  exit 1
fi

echo "[OK] ${total} scripts bajo plugins/ en 100755"
