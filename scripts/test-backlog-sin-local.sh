#!/usr/bin/env bash
# Gate: docs/backlog.md (histórico congelado, repo público) no arrastra
# contexto local: rutas bajo /home/, el nombre de la KB privada (wisdom-*) ni
# wikilinks [[...]] a ella (#67).
#
# Solo cubre backlog.md: el resto de docs/ (consultas, planes, runbooks) aún
# cita esos patrones y está fuera del alcance de #67.
#
# Uso: test-backlog-sin-local.sh [fichero]   (por defecto docs/backlog.md;
# el argumento permite ejercitar el gate en rojo con un fixture).
set -uo pipefail
RAIZ="$(git rev-parse --show-toplevel)" || exit 1
if [ -z "$RAIZ" ]; then
  echo "test-backlog-sin-local: git rev-parse --show-toplevel no devolvió nada" >&2
  exit 1
fi
cd "$RAIZ" || exit 1

FICHERO="${1:-docs/backlog.md}"
PATRON='/home/|wisdom-|\[\['

# exit de grep: 0 = hallazgo, 1 = limpio, >=2 = error (fichero ilegible);
# tratar 1 y >=2 igual dejaría pasar en verde un fichero que no se pudo leer.
match="$(grep -nE "$PATRON" "$FICHERO" 2>&1)"
ec=$?
case "$ec" in
  0)
    echo "test-backlog-sin-local: contexto local en $FICHERO:" >&2
    printf '%s\n' "$match" >&2
    echo "Arreglo: ~ o «el HOME del runner» en vez de una ruta bajo el home, el concepto en prosa en vez del nombre de la KB privada o de un wikilink." >&2
    exit 1
    ;;
  1) : ;;
  *)
    echo "test-backlog-sin-local: error leyendo $FICHERO (grep exit=$ec): $match" >&2
    exit 1
    ;;
esac

echo "test-backlog-sin-local: OK — $FICHERO sin rutas /home/, KB privada ni wikilinks"
