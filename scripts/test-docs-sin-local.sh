#!/usr/bin/env bash
# Gate: docs/ (repo público) no arrastra contexto local: rutas del home del
# autor, el nombre de la KB privada (wisdom-*) ni wikilinks a ella (#67, #80).
#
# Dos niveles:
#   - docs/backlog.md (histórico congelado): patrón estricto, cualquier
#     /home/, wisdom-* o [[...]].
#   - todos los .md versionados de docs/ (backlog incluido): patrón de
#     identidad: el home del autor (Linux y Windows) o wisdom-*.
#
# El resto de docs/ no usa el patrón estricto porque cita legítimamente
# /home/ y [[: el regex del gate de rutas, fixtures de usuarios ficticios,
# la sintaxis de wikilinks que indexa el engine y [[ ]] de bash, JSON y Rust.
#
# Uso: test-docs-sin-local.sh [directorio]   (por defecto docs/; con
# argumento se escanean los .md de ese directorio, y su backlog.md recibe el
# nivel estricto, para ejercitar el gate en rojo con fixtures).
set -uo pipefail
RAIZ="$(git rev-parse --show-toplevel)" || exit 1
if [ -z "$RAIZ" ]; then
  echo "test-docs-sin-local: git rev-parse --show-toplevel no devolvió nada" >&2
  exit 1
fi
cd "$RAIZ" || exit 1

DIR="${1:-docs}"
DIR="${DIR%/}"
# grep de MSYS sobre la lista: git.exe reescribiría en Windows un patrón que
# empieza por / como ruta.
if [ $# -ge 1 ]; then
  mapfile -t FICHEROS < <(find "$DIR" -name '*.md' | sort)
else
  mapfile -t FICHEROS < <(git ls-files 'docs/*.md')
fi
if [ "${#FICHEROS[@]}" -eq 0 ]; then
  echo "test-docs-sin-local: ningún .md en $DIR" >&2
  exit 1
fi

rc=0
# escanear <etiqueta> <patrón> <fichero>...
escanear() {
  local etiqueta="$1" patron="$2" match ec
  shift 2
  # exit de grep: 0 = hallazgo, 1 = limpio, >=2 = error (fichero ilegible);
  # tratar 1 y >=2 igual dejaría pasar en verde un fichero que no se pudo leer.
  match="$(grep -nE "$patron" "$@" 2>&1)"
  ec=$?
  case "$ec" in
    0)
      echo "test-docs-sin-local: contexto local ($etiqueta):" >&2
      printf '%s\n' "$match" >&2
      rc=1
      ;;
    1) : ;;
    *)
      echo "test-docs-sin-local: error leyendo ficheros ($etiqueta, grep exit=$ec): $match" >&2
      rc=1
      ;;
  esac
}

escanear "backlog, estricto" '/home/|wisdom-|\[\[' "$DIR/backlog.md"
escanear "docs/, identidad" '/home/paul|wisdom-|C:/Users/paul' "${FICHEROS[@]}"

if [ "$rc" -ne 0 ]; then
  echo "Arreglo: ~ o «el HOME del runner» en vez de una ruta bajo el home, el concepto en prosa en vez del nombre de la KB privada o de un wikilink." >&2
  exit 1
fi
echo "test-docs-sin-local: OK — $DIR sin rutas del home, KB privada ni wikilinks (backlog estricto)"
