#!/usr/bin/env bash
# Tests de mutantes-nocturnos.sh: `base` sobre un repo git de juguete con fechas
# fijas, `veredicto` sobre cada exit de cargo-mutants. `run` no: necesita
# cargo-mutants y minutos de CPU; lo ejerce el propio workflow.
set -uo pipefail
RAIZ="$(git rev-parse --show-toplevel)" || exit 1
M="$RAIZ/scripts/mutantes-nocturnos.sh"
PASS=0; FAIL=0
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT

check() { # nombre, esperado_exit, patrón, args...
  local n="$1" ex="$2" pat="$3" out rc
  shift 3
  out="$(cd "$T/repo" && bash "$M" "$@" 2>&1)"; rc=$?
  if [ "$rc" = "$ex" ] && printf '%s' "$out" | grep -qF -- "$pat"; then
    echo "PASS: $n"; PASS=$((PASS+1))
  else
    echo "FAIL: $n (exit=$rc, esperado=$ex, salida='$out')"; FAIL=$((FAIL+1))
  fi
}

commit() { # mensaje, fecha_iso
  echo "$1" > "$T/repo/f"
  git -C "$T/repo" add f
  GIT_COMMITTER_DATE="$2" GIT_AUTHOR_DATE="$2" \
    git -C "$T/repo" -c user.email=t@t -c user.name=t commit -qm "$1"
  git -C "$T/repo" rev-parse HEAD
}

mkdir -p "$T/repo" && git -C "$T/repo" init -q
c1="$(commit uno 2026-10-01T00:00:00Z)"
c2="$(commit dos 2026-10-02T00:00:00Z)"
c3="$(commit tres 2026-10-03T00:00:00Z)"
git -C "$T/repo" checkout -q -b otra "$c1"
huerfano="$(commit fuera 2026-10-02T12:00:00Z)"
git -C "$T/repo" checkout -q -
git -C "$T/repo" branch -q -D otra

check verde_ancestro_manda 0 "$c2" base "$c2" 2026-10-01T12:00:00Z
check head_como_verde 0 "$c3" base "$c3" 2026-10-01T12:00:00Z
check sin_verde_cae_a_fecha 0 "$c1" base - 2026-10-01T12:00:00Z
check verde_vacio_cae_a_fecha 0 "$c1" base "" 2026-10-01T12:00:00Z
check no_ancestro_cae_a_fecha 0 "no es ancestro" base "$huerfano" 2026-10-02T06:00:00Z
check no_ancestro_base_por_fecha 0 "$c2" base "$huerfano" 2026-10-02T06:00:00Z
check corte_antes_de_todo_es_rojo 1 "no hay base" base - 2026-09-01T00:00:00Z
check base_sin_args 2 "uso:" base -

check exit0_verde 0 "verde" veredicto 0
check exit3_timeout_verde 0 "timeout" veredicto 3
check exit2_rojo 1 "sin cazar" veredicto 2
check exit4_baseline_rojo 1 "baseline" veredicto 4
check exit5_diff_rojo 1 "diff no casa" veredicto 5
check exit6_diff_rojo 1 "diff no casa" veredicto 6
check exit70_rojo 1 "salió con 70" veredicto 70
check subcomando_desconocido 2 "uso:" foo

echo "--- $PASS PASS, $FAIL FAIL"
[ "$FAIL" -eq 0 ]
