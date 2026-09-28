#!/usr/bin/env bash
# Test standalone para skills/orchestrate/scripts/review-package (sección MUTACIÓN).
# Repos en mktemp -d con git init; las herramientas de mutación son stubs de
# testdata/review-package/bin. Nunca se llama a una herramienta real.
# Los stubs replican la invocación verificada contra los binarios reales
# (cargo-mutants 27.1.0, mutmut 2.5.1/3.8.0, stryker 9.6.1).
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

# PATH hermético por test: $pdir contiene SOLO symlinks a las utilidades que el
# script necesita (resueltas una vez con `command -v`) más los stubs pedidos, y
# se usa como PATH a secas. Así un cargo-mutants/mutmut/npx real instalado en la
# máquina no puede contaminar ningún caso.
UTILS="bash sh env git jq sed grep awk tr mktemp sleep cat head tail sort dirname basename rm mkdir cp mv wc date id timeout perl uname"
stubs_para() {
  local d="$TMP/path-$1" u p; shift
  mkdir -p "$d"
  for u in $UTILS; do
    p=$(command -v "$u" 2>/dev/null) || continue
    case "$p" in /*) ln -sf "$p" "$d/$u" ;; esac
  done
  for u in "$@"; do cp "$STUBS/$u" "$d/$u"; done
  echo "$d"
}

# mkrepo NOMBRE MANIFIESTO FICHERO... -> repo con base (manifiesto + README) y
# head (FICHEROs añadidos). Deja la ruta en $REPO y los commits en $BASE/$HEAD
# (sin subshell, para que las variables sobrevivan). El manifiesto puede ir en un subdir.
mkrepo() {
  local r="$TMP/repo-$1" manifiesto=$2; shift 2
  mkdir -p "$r"
  git -C "$r" init -q
  git -C "$r" config user.email t@t
  git -C "$r" config user.name t
  if [ -n "$manifiesto" ]; then mkdir -p "$r/$(dirname "$manifiesto")"; echo x > "$r/$manifiesto"; fi
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

# corre REPO PATHDIR [ENV=..]... -> $OUT (contenido del package), $LOG (log de los stubs)
corre() {
  local r=$1 pdir=$2; shift 2
  LOG="$TMP/stub-$$-$RANDOM.log"; : > "$LOG"
  ( cd "$r" && env PATH="$pdir" STUB_LOG="$LOG" "$@" "$RP" "$BASE" "$HEAD" "$TMP/out.diff" >/dev/null 2>"$TMP/err" )
  RC=$?
  OUT=$(cat "$TMP/out.diff")
}
seccion() { sed -n '/^## MUTACIÓN$/,$p' <<<"$OUT"; }

# ---- self-test del arnés: con PATH hermético no aparece ninguna herramienta real
p=$(stubs_para selftest)
if PATH="$p" command -v cargo-mutants mutmut npx stryker >/dev/null 2>&1; then fail selftest_hermetico "el PATH hermético ve una herramienta"; else pass selftest_hermetico; fi

# ---- sin_herramienta
mkrepo sinher "" src/foo.txt; r=$REPO
corre "$r" "$(stubs_para vacio)"
if [ "$RC" = 0 ] && contains "$OUT" "## MUTACIÓN" && contains "$OUT" "MUTACIÓN: no disponible (sin herramienta para este repo)"; then pass sin_herramienta; else fail sin_herramienta "rc=$RC"; fi

# ---- herramienta_ausente
mkrepo ausente Cargo.toml src/lib.rs; r=$REPO
corre "$r" "$(stubs_para vacio)"
if contains "$OUT" "MUTACIÓN: no disponible (cargo-mutants no instalado)"; then pass herramienta_ausente; else fail herramienta_ausente "$(seccion)"; fi

# ---- timeout (y sin huérfanos: sleep con duración única)
mkrepo tmo Cargo.toml src/lib.rs; r=$REPO
t0=$SECONDS
corre "$r" "$(stubs_para tmo cargo-mutants)" EXO_MUTATION_TIMEOUT=1 STUB_SLEEP=31337
dt=$((SECONDS - t0))
if contains "$OUT" "MUTACIÓN: parcial (timeout 1s)" && [ "$dt" -lt 5 ]; then pass timeout; else fail timeout "dt=${dt}s: $(seccion)"; fi
if pgrep -f "sleep 31337" >/dev/null 2>&1; then fail timeout_huerfanos "queda un sleep 31337 vivo"; pkill -f "sleep 31337" 2>/dev/null; else pass timeout_huerfanos; fi

# ---- rust_ok
mkrepo rust Cargo.toml src/lib.rs; r=$REPO
corre "$r" "$(stubs_para rust cargo-mutants)"
if contains "$OUT" "MUTACIÓN: 8/10" && contains "$OUT" "replace add -> i32 with 0" && contains "$OUT" "replace == with != in cmp"; then pass rust_ok; else fail rust_ok "$(seccion)"; fi

# ---- solo_diff: cargo recibe --in-diff con un patch SOLO de producción
mkrepo solo Cargo.toml src/lib.rs src/otro.rs tests/it.rs docs/x.md src/foo_test.rs; r=$REPO
corre "$r" "$(stubs_para solo cargo-mutants)"
args=$(cat "$LOG")
if contains "$args" "--in-diff" && not_contains "$args" " -f " \
   && contains "$args" "+++ b/src/lib.rs" && contains "$args" "+++ b/src/otro.rs" \
   && not_contains "$args" "tests/it.rs" && not_contains "$args" ".md" && not_contains "$args" "foo_test.rs" \
   && not_contains "$args" "Cargo.toml"; then pass solo_diff; else fail solo_diff "log: $args"; fi

# ---- diff_sin_produccion
mkrepo sinprod Cargo.toml docs/x.md tests/it.rs; r=$REPO
corre "$r" "$(stubs_para sinprod cargo-mutants)"
if contains "$OUT" "MUTACIÓN: no disponible (diff sin código de producción)" && [ ! -s "$LOG" ]; then pass diff_sin_produccion; else fail diff_sin_produccion "log=$(cat "$LOG")"; fi

# ---- desactivada
mkrepo off Cargo.toml src/lib.rs; r=$REPO
corre "$r" "$(stubs_para off cargo-mutants)" EXO_MUTATION=0
if contains "$OUT" "MUTACIÓN: no disponible (desactivada)" && [ ! -s "$LOG" ]; then pass desactivada; else fail desactivada "$(seccion)"; fi

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
if [ "$(printf '%s' "$previo")" = "$(printf '%s' "$esperado")" ]; then pass regresion_package; else fail regresion_package "las secciones previas cambiaron"; fi

# ---- arbol_sucio_no_bloquea: se muta un worktree temporal; el árbol del usuario ni se mira ni se toca
mkrepo sucio Cargo.toml src/lib.rs; r=$REPO
echo sucio >> "$r/src/lib.rs"
corre "$r" "$(stubs_para sucio cargo-mutants)"
if contains "$OUT" "MUTACIÓN: 8/10" && not_contains "$OUT" "árbol sucio" \
   && [ "$(tail -n 1 "$r/src/lib.rs")" = sucio ] && [ "$(git -C "$r" status --porcelain)" = " M src/lib.rs" ] \
   && [ "$(grep -c '^+++ b/src/lib.rs' "$LOG")" = 1 ] && not_contains "$(cat "$LOG")" "cwd=$r"; then pass arbol_sucio_no_bloquea; else fail arbol_sucio_no_bloquea "$(seccion) log=$(cat "$LOG")"; fi

# ---- mutacion_no_ensucia_arbol (C1): mutmut 2.x muta en sitio y se cuelga hasta el timeout
mkrepo nodirty setup.py src/m.py; r=$REPO
corre "$r" "$(stubs_para nodirty mutmut)" STUB_MUTMUT=2 STUB_MODE=dirty EXO_MUTATION_TIMEOUT=1
if pgrep -f "sleep 31338" >/dev/null 2>&1; then pkill -f "sleep 31338" 2>/dev/null; fi
if contains "$OUT" "MUTACIÓN: parcial (timeout 1s)" && [ -z "$(git -C "$r" status --porcelain)" ] \
   && [ "$(git -C "$r" worktree list | wc -l | tr -d ' ')" = 1 ] && [ -z "$(ls "$r/.git/worktrees" 2>/dev/null)" ]; then
  pass mutacion_no_ensucia_arbol
else fail mutacion_no_ensucia_arbol "status=[$(git -C "$r" status --porcelain)] wt=$(git -C "$r" worktree list) $(seccion)"; fi

# ---- head_no_actual (I3): BASE..HEAD~1 sí muta, parcheando lo de HEAD~1
mkrepo headprev Cargo.toml src/lib.rs; r=$REPO
echo "otro cambio" > "$r/src/otro.rs"; git -C "$r" add src/otro.rs; git -C "$r" commit -qm extra
HEAD=$(git -C "$r" rev-parse HEAD~1)
corre "$r" "$(stubs_para headprev cargo-mutants)"
if contains "$OUT" "MUTACIÓN: 8/10" && not_contains "$OUT" "HEAD no es el worktree" \
   && contains "$(cat "$LOG")" "+++ b/src/lib.rs" && not_contains "$(cat "$LOG")" "otro.rs"; then pass head_no_actual; else fail head_no_actual "$(seccion) log=$(cat "$LOG")"; fi

# ---- worktree_falla: si `git worktree add` falla => no disponible (<motivo>), sin mutar
mkrepo wtfalla Cargo.toml src/lib.rs; r=$REPO
p=$(stubs_para wtfalla cargo-mutants); real_git=$(command -v git)
rm -f "$p/git"
printf '#!/bin/sh\ncase " $* " in *" worktree add "*) echo "fatal: simulado" >&2; exit 128 ;; esac\nexec %s "$@"\n' "$real_git" > "$p/git"; chmod +x "$p/git"
corre "$r" "$p"
if contains "$OUT" "MUTACIÓN: no disponible (worktree:" && contains "$OUT" "simulado" && [ ! -s "$LOG" ]; then pass worktree_falla; else fail worktree_falla "$(seccion) log=$(cat "$LOG")"; fi

# ---- workspace_cargo (I1): el patch va relativo a la RAÍZ del workspace, no al crate miembro
mkrepo ws "" crates/a/Cargo.toml crates/a/src/lib.rs; r=$REPO
printf '[workspace]\nmembers = ["crates/a"]\n' > "$r/Cargo.toml"
git -C "$r" add Cargo.toml; git -C "$r" commit -qm ws
HEAD=$(git -C "$r" rev-parse HEAD)
corre "$r" "$(stubs_para ws cargo-mutants cargo)"
a=$(cat "$LOG")
if contains "$OUT" "MUTACIÓN [crates/a]: 8/10" && contains "$a" "+++ b/crates/a/src/lib.rs" \
   && grep -q '^cwd=.*/crates/a$' <<<"$a" && not_contains "$a" "+++ b/src/lib.rs"; then pass workspace_cargo; else fail workspace_cargo "$(seccion) log=$a"; fi

# ---- baseline_falla (M4): baseline rojo => parcial (baseline falló), no "0 mutantes"
mkrepo bl Cargo.toml src/lib.rs; r=$REPO
corre "$r" "$(stubs_para bl cargo-mutants)" STUB_MODE=baselinefail
if contains "$OUT" "MUTACIÓN: parcial (baseline falló)" && not_contains "$OUT" "0 mutantes"; then pass baseline_falla; else fail baseline_falla "$(seccion)"; fi

# ---- timeout_invalido (M5): aviso visible en la sección y se usa 600
mkrepo tinv Cargo.toml src/lib.rs; r=$REPO
corre "$r" "$(stubs_para tinv cargo-mutants)" EXO_MUTATION_TIMEOUT=abc
if contains "$OUT" "MUTACIÓN: 8/10" && contains "$(seccion)" "EXO_MUTATION_TIMEOUT inválido" && contains "$(seccion)" "600"; then pass timeout_invalido; else fail timeout_invalido "$(seccion)"; fi

# ---- salida_no_reconocida
mkrepo garb Cargo.toml src/lib.rs; r=$REPO
corre "$r" "$(stubs_para garb cargo-mutants)" STUB_MODE=garbage
if contains "$OUT" "MUTACIÓN: parcial (salida no reconocida)" && contains "$OUT" "esto no es json"; then pass salida_no_reconocida; else fail salida_no_reconocida "$(seccion)"; fi

# ---- python_3x_no_acota (mutmut --version declara 3.x)
mkrepo py3 pyproject.toml src/m.py; r=$REPO
corre "$r" "$(stubs_para py3 mutmut)" STUB_MUTMUT=3
if contains "$OUT" "MUTACIÓN: no disponible (mutmut 3.x no admite acotar por CLI)" && not_contains "$(cat "$LOG")" "paths-to-mutate src"; then pass python_3x_no_acota; else fail python_3x_no_acota "$(seccion) log=$(cat "$LOG")"; fi

# ---- python_2x (sin --version, run --help anuncia --paths-to-mutate; lista con comas)
mkrepo py2 setup.py src/m.py src/n.py tests/test_m.py; r=$REPO
corre "$r" "$(stubs_para py2 mutmut)" STUB_MUTMUT=2
if contains "$OUT" "MUTACIÓN: 8/10" && contains "$(cat "$LOG")" "--paths-to-mutate src/m.py,src/n.py" && not_contains "$(cat "$LOG")" "test_m"; then pass python_2x; else fail python_2x "$(seccion) log=$(cat "$LOG")"; fi

# ---- js_stryker: una sola lista --mutate separada por comas, --reporters json,
# sin --jsonReporter.* (no existe), informe en reports/mutation/mutation.json
mkrepo js package.json src/a.js src/b.js src/a.test.js; r=$REPO
echo '{"devDependencies":{"@stryker-mutator/core":"9"}}' > "$r/package.json"
git -C "$r" commit -qam pkg
HEAD=$(git -C "$r" rev-parse HEAD)
corre "$r" "$(stubs_para js npx)"
a=$(cat "$LOG")
if contains "$OUT" "MUTACIÓN: 3/4" && contains "$OUT" "src/a.js:7" \
   && contains "$a" '--mutate src/a.js,src/b.js' && contains "$a" '--reporters json' \
   && not_contains "$a" "jsonReporter" && not_contains "$a" "a.test.js" \
   && [ "$(grep -o -- '--mutate' <<<"$a" | wc -l | tr -d ' ')" = 1 ]; then pass js_stryker; else fail js_stryker "$(seccion) log=$a"; fi

# ---- monorepo_subdir: manifest en engine/ -> cargo corre EN engine/, patch relativo
mkrepo mono1 engine/Cargo.toml engine/src/lib.rs; r=$REPO
corre "$r" "$(stubs_para mono1 cargo-mutants)"
a=$(cat "$LOG")
if contains "$OUT" "MUTACIÓN [engine]: 8/10" && contains "$(seccion)" "MUTACIÓN: 8/10" \
   && grep -q '^cwd=.*/engine$' <<<"$a" && not_contains "$a" "cwd=$r" && contains "$a" "+++ b/src/lib.rs" && not_contains "$a" "engine/src"; then pass monorepo_subdir; else fail monorepo_subdir "$(seccion) log=$a"; fi

# ---- monorepo_dos_proyectos: rust en engine/ + js en web/, una línea por proyecto y agregado
mkrepo mono2 engine/Cargo.toml engine/src/lib.rs web/src/a.js web/src/a.test.js; r=$REPO
echo '{"devDependencies":{"@stryker-mutator/core":"9"}}' > "$r/web/package.json"
git -C "$r" add web/package.json; git -C "$r" commit -qm pkg
HEAD=$(git -C "$r" rev-parse HEAD)
corre "$r" "$(stubs_para mono2 cargo-mutants npx)"
a=$(cat "$LOG"); s=$(seccion)
primera=$(sed -n '2p' <<<"$s")
if contains "$primera" "MUTACIÓN: 11/14" && contains "$s" "MUTACIÓN [engine]: 8/10" && contains "$s" "MUTACIÓN [web]: 3/4" \
   && grep -q '^cwd=.*/engine$' <<<"$a" && grep -q '^cwd=.*/web$' <<<"$a" && not_contains "$a" "cwd=$r" && contains "$a" "--mutate src/a.js"; then pass monorepo_dos_proyectos; else fail monorepo_dos_proyectos "$s log=$a"; fi

printf '\n%d pass, %d fail\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
