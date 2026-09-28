#!/usr/bin/env bash
# Test standalone para skills/orchestrate/scripts/review-package (sección MUTACIÓN).
# Repos en mktemp -d con git init; las herramientas de mutación son stubs de
# testdata/review-package/bin en PATH. Nunca se llama a una herramienta real.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RP="${SCRIPT_DIR}/../skills/orchestrate/scripts/review-package"
STUBS="${SCRIPT_DIR}/testdata/review-package/bin"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }
contains()     { case "$1" in *"$2"*) return 0 ;; *) return 1 ;; esac; }
not_contains() { case "$1" in *"$2"*) return 1 ;; *) return 0 ;; esac; }

# Un dir de PATH por test con SOLO los stubs pedidos (hermético: un cargo-mutants
# real instalado en la máquina no contamina herramienta_ausente).
stubs_para() {
  local d="$TMP/path-$1"; shift
  mkdir -p "$d"
  local s
  for s in "$@"; do cp "$STUBS/$s" "$d/$s"; done
  echo "$d"
}

# mkrepo NOMBRE MANIFIESTO FICHERO... -> repo con base (manifiesto + README) y
# head (FICHEROs añadidos). Deja la ruta en $REPO (sin subshell). BASE/HEAD quedan en $BASE/$HEAD.
mkrepo() {
  local r="$TMP/repo-$1" manifiesto=$2; shift 2
  mkdir -p "$r"
  git -C "$r" init -q
  git -C "$r" config user.email t@t
  git -C "$r" config user.name t
  [ -n "$manifiesto" ] && echo x > "$r/$manifiesto"
  echo base > "$r/README.md"
  git -C "$r" add -A >/dev/null
  git -C "$r" commit -qm base
  BASE=$(git -C "$r" rev-parse HEAD)
  local f
  for f in "$@"; do
    mkdir -p "$r/$(dirname "$f")"
    echo "cambio $f" > "$r/$f"
  done
  git -C "$r" add -A >/dev/null
  git -C "$r" commit -qm head
  HEAD=$(git -C "$r" rev-parse HEAD)
  REPO=$r
}

# corre REPO PATHDIR [ENV=..]... -> $OUT (contenido del package), $LOG (argv del stub)
corre() {
  local r=$1 pdir=$2; shift 2
  LOG="$TMP/stub-$$-$RANDOM.log"; : > "$LOG"
  ( cd "$r" && env PATH="$pdir:$PATH" STUB_LOG="$LOG" "$@" "$RP" "$BASE" "$HEAD" "$TMP/out.diff" >/dev/null 2>"$TMP/err" )
  RC=$?
  OUT=$(cat "$TMP/out.diff")
}

# ---- sin_herramienta
mkrepo sinher "" src/foo.txt; r=$REPO
corre "$r" "$(stubs_para vacio)"
if [ "$RC" = 0 ] && contains "$OUT" "## MUTACIÓN" && contains "$OUT" "MUTACIÓN: no disponible (sin herramienta para este repo)"; then pass sin_herramienta; else fail sin_herramienta "rc=$RC"; fi

# ---- herramienta_ausente
mkrepo ausente Cargo.toml src/lib.rs; r=$REPO
corre "$r" "$(stubs_para vacio)"
if contains "$OUT" "MUTACIÓN: no disponible (cargo-mutants no instalado)"; then pass herramienta_ausente; else fail herramienta_ausente "$(tail -3 <<<"$OUT")"; fi

# ---- timeout (y sin huérfanos)
mkrepo tmo Cargo.toml src/lib.rs; r=$REPO
t0=$SECONDS
corre "$r" "$(stubs_para tmo cargo-mutants)" EXO_MUTATION_TIMEOUT=1 STUB_SLEEP=30
dt=$((SECONDS - t0))
if contains "$OUT" "MUTACIÓN: parcial (timeout 1s)" && [ "$dt" -lt 5 ]; then pass timeout; else fail timeout "dt=${dt}s: $(tail -3 <<<"$OUT")"; fi
if pgrep -f "sleep 30" >/dev/null 2>&1; then fail timeout_huerfanos "queda un sleep 30 vivo"; pkill -f "sleep 30" 2>/dev/null; else pass timeout_huerfanos; fi

# ---- rust_ok
mkrepo rust Cargo.toml src/lib.rs; r=$REPO
corre "$r" "$(stubs_para rust cargo-mutants)"
if contains "$OUT" "MUTACIÓN: 8/10" && contains "$OUT" "replace add -> i32 with 0" && contains "$OUT" "replace == with != in cmp"; then pass rust_ok; else fail rust_ok "$(sed -n '/## MUTACIÓN/,$p' <<<"$OUT")"; fi

# ---- solo_diff
mkrepo solo Cargo.toml src/lib.rs src/otro.rs tests/it.rs docs/x.md src/foo_test.rs; r=$REPO
corre "$r" "$(stubs_para solo cargo-mutants)"
args=$(cat "$LOG")
if contains "$args" "src/lib.rs" && contains "$args" "src/otro.rs" \
   && not_contains "$args" "tests/it.rs" && not_contains "$args" ".md" && not_contains "$args" "foo_test.rs" \
   && not_contains "$args" "Cargo.toml"; then pass solo_diff; else fail solo_diff "argv: $args"; fi

# ---- diff_sin_produccion
mkrepo sinprod Cargo.toml docs/x.md tests/it.rs; r=$REPO
corre "$r" "$(stubs_para sinprod cargo-mutants)"
if contains "$OUT" "MUTACIÓN: no disponible (diff sin código de producción)" && [ ! -s "$LOG" ]; then pass diff_sin_produccion; else fail diff_sin_produccion "log=$(cat "$LOG")"; fi

# ---- desactivada
mkrepo off Cargo.toml src/lib.rs; r=$REPO
corre "$r" "$(stubs_para off cargo-mutants)" EXO_MUTATION=0
if contains "$OUT" "MUTACIÓN: no disponible (desactivada)" && [ ! -s "$LOG" ]; then pass desactivada; else fail desactivada "$(tail -3 <<<"$OUT")"; fi

# ---- regresion_package
mkrepo reg Cargo.toml src/lib.rs; r=$REPO
corre "$r" "$(stubs_para reg cargo-mutants)" EXO_MUTATION=0
esperado=$(
  echo "# Review package: ${BASE}..${HEAD}"; echo; echo "## Commits"
  git -C "$r" log --oneline "${BASE}..${HEAD}"; echo; echo "## Files changed"
  git -C "$r" diff --stat "${BASE}..${HEAD}"; echo; echo "## Diff"
  git -C "$r" diff -U10 "${BASE}..${HEAD}"
)
previo=$(sed '/^## MUTACIÓN$/,$d' "$TMP/out.diff")
# comparamos sin la línea en blanco final que separa la sección nueva
if [ "$(printf '%s' "$previo")" = "$(printf '%s' "$esperado")" ]; then pass regresion_package; else fail regresion_package "las secciones previas cambiaron"; fi

# ---- arbol_sucio
mkrepo sucio Cargo.toml src/lib.rs; r=$REPO
echo sucio >> "$r/src/lib.rs"
corre "$r" "$(stubs_para sucio cargo-mutants)"
if contains "$OUT" "MUTACIÓN: no disponible (árbol sucio)" && [ ! -s "$LOG" ]; then pass arbol_sucio; else fail arbol_sucio "$(tail -3 <<<"$OUT")"; fi

# ---- salida_no_reconocida
mkrepo garb Cargo.toml src/lib.rs; r=$REPO
corre "$r" "$(stubs_para garb cargo-mutants)" STUB_MODE=garbage
if contains "$OUT" "MUTACIÓN: parcial (salida no reconocida)" && contains "$OUT" "esto no es json"; then pass salida_no_reconocida; else fail salida_no_reconocida "$(tail -4 <<<"$OUT")"; fi

# ---- python_3x_no_acota
mkrepo py3 pyproject.toml src/m.py; r=$REPO
corre "$r" "$(stubs_para py3 mutmut)" STUB_MUTMUT=3
if contains "$OUT" "MUTACIÓN: no disponible (mutmut 3.x no admite acotar por CLI)" && not_contains "$(cat "$LOG")" "run --paths"; then pass python_3x_no_acota; else fail python_3x_no_acota "$(tail -3 <<<"$OUT") log=$(cat "$LOG")"; fi

# ---- python_2x
mkrepo py2 setup.py src/m.py tests/test_m.py; r=$REPO
corre "$r" "$(stubs_para py2 mutmut)" STUB_MUTMUT=2
if contains "$OUT" "MUTACIÓN: 8/10" && contains "$(cat "$LOG")" "--paths-to-mutate src/m.py" && not_contains "$(cat "$LOG")" "test_m"; then pass python_2x; else fail python_2x "$(tail -3 <<<"$OUT") log=$(cat "$LOG")"; fi

# ---- js_stryker
mkrepo js package.json src/a.js src/a.test.js; r=$REPO
echo '{"devDependencies":{"@stryker-mutator/core":"9"}}' > "$r/package.json"
git -C "$r" commit -qam pkg
HEAD=$(git -C "$r" rev-parse HEAD)
corre "$r" "$(stubs_para js npx)"
if contains "$OUT" "MUTACIÓN: 3/4" && contains "$OUT" "src/a.js:7" && contains "$(cat "$LOG")" "--mutate src/a.js" && not_contains "$(cat "$LOG")" "a.test.js"; then pass js_stryker; else fail js_stryker "$(sed -n '/## MUTACIÓN/,$p' <<<"$OUT") log=$(cat "$LOG")"; fi

printf '\n%d pass, %d fail\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
