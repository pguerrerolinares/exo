#!/usr/bin/env bash
# Mutantes nocturnos: cargo-mutants solo sobre las líneas de engine/src que
# entraron en HEAD desde la última corrida verde. Rojo = entró código cuyos
# tests no cazan un mutante (o el baseline ya estaba rojo).
#
# Uso: mutantes-nocturnos.sh base <ultimo_verde|-> <corte_iso>
#        Imprime la base del diff. <ultimo_verde>: headSha de la última corrida
#        verde; si falta o no es ancestro de HEAD (historia reescrita), la base
#        es el último commit anterior a <corte_iso>.
#      mutantes-nocturnos.sh veredicto <exit_de_cargo_mutants>
#        Traduce el exit de cargo-mutants (src/exit_code.rs de 27.1.0) a
#        verde (0) o rojo (1). Timeout (3) cuenta como cazado.
#      mutantes-nocturnos.sh run <ultimo_verde|-> <corte_iso> [<jobs>]
#        base + diff de engine/src + cargo mutants --in-diff + veredicto.
set -uo pipefail

base() {
  local verde="$1" corte="$2" b
  if [ "$verde" != "-" ] && [ -n "$verde" ] && git merge-base --is-ancestor "$verde" HEAD 2>/dev/null; then
    echo "$verde"; return 0
  fi
  [ "$verde" != "-" ] && [ -n "$verde" ] && echo "aviso: $verde no es ancestro de HEAD; base por fecha ($corte)" >&2
  b="$(git rev-list -1 --before="$corte" HEAD)"
  [ -n "$b" ] || { echo "sin commit anterior a $corte: no hay base" >&2; return 1; }
  echo "$b"
}

veredicto() {
  case "$1" in
    0) echo "verde: todos los mutantes cazados"; return 0 ;;
    3) echo "verde: todos cazados; alguno por timeout (cuenta como cazado)"; return 0 ;;
    2) echo "rojo: hay mutantes sin cazar (ver mutants.out/missed.txt)" ;;
    4) echo "rojo: los tests ya fallan sin mutar (baseline)" ;;
    5|6) echo "rojo: el diff no casa con el árbol o no se puede parsear (exit $1)" ;;
    *) echo "rojo: cargo-mutants salió con $1" ;;
  esac
  return 1
}

run() {
  local b raiz diff rc
  b="$(base "$1" "$2")" || return 1
  raiz="$(git rev-parse --show-toplevel)" || return 1
  diff="$(mktemp)"
  git -C "$raiz" diff --relative=engine "$b" HEAD -- engine/src > "$diff"
  if [ ! -s "$diff" ]; then
    echo "verde: sin cambios en engine/src desde $b"; rm -f "$diff"; return 0
  fi
  echo "base $b; diff de $(grep -c '^+[^+]' "$diff") líneas añadidas en engine/src"
  (cd "$raiz/engine" && cargo mutants --in-diff "$diff" -j "${3:-2}"); rc=$?
  rm -f "$diff"
  veredicto "$rc"
}

case "${1:-}" in
  base) [ $# -eq 3 ] || { echo "uso: $0 base <ultimo_verde|-> <corte_iso>" >&2; exit 2; }; base "$2" "$3" ;;
  veredicto) [ $# -eq 2 ] || { echo "uso: $0 veredicto <exit>" >&2; exit 2; }; veredicto "$2" ;;
  run) [ $# -ge 3 ] || { echo "uso: $0 run <ultimo_verde|-> <corte_iso> [<jobs>]" >&2; exit 2; }; run "$2" "$3" "${4:-2}" ;;
  *) echo "uso: $0 base|veredicto|run ..." >&2; exit 2 ;;
esac
