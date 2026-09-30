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
UTILS="bash sh true seq python3 env git jq sed grep awk tr mktemp sleep cat head tail sort dirname basename rm mkdir cp mv wc date id timeout perl uname chmod"
stubs_para() {
  local d="$TMP/path-$1" u p; shift
  mkdir -p "$d"
  for u in $UTILS; do
    p=$(command -v "$u" 2>/dev/null) || continue
    # `true` puede resolverse como builtin (sin ruta): mutmut lo lanza como binario (--runner true)
    case "$p" in /*) ;; *) p=""; for c in "/usr/bin/$u" "/bin/$u"; do [ -x "$c" ] && { p=$c; break; }; done ;; esac
    # En MSYS (Windows) `ln -s` COPIA el .exe lejos de sus DLLs (msys-2.0.dll...), y con el
    # PATH hermético el loader no las encuentra: 127. Un wrapper ejecuta el binario desde su
    # dir real, donde el loader busca las DLLs primero.
    case "$p" in /*)
      case "$(uname -s)" in
        MINGW*|MSYS*|CYGWIN*) printf '#!/bin/sh\nexec %s "$@"\n' "'$p'" > "$d/$u"; chmod +x "$d/$u" ;;
        *) ln -sf "$p" "$d/$u" ;;
      esac ;;
    esac
  done
  for u in "$@"; do cp "$STUBS/$u" "$d/$u"; done
  echo "$d"
}

# sedi ARGS... FICHERO -> `sed -i` portable: el de BSD (macOS) toma el script como
# sufijo de backup. El último argumento es el fichero; se reescribe vía temporal.
sedi() { local f=${!#}; sed "${@:1:$#-1}" "$f" > "$f.sedi" && mv "$f.sedi" "$f"; }

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
   && contains "$a" '--mutate src/a.js:1-1,src/b.js:1-1' && contains "$a" '--reporters json' \
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

# ---- helpers para repos con contenido controlado (base con líneas numeradas, head con cambios)
# mkrepo_cnt NOMBRE MANIFIESTO -> repo con base: MANIFIESTO + README; el test añade ficheros y llama a commitea.
lineas() { local i n=$1 pre=$2; for i in $(seq 1 "$n"); do echo "${pre}${i}"; done; }

# ---- mutmut_acotado_al_diff: subproyecto py/, 3 funciones, el diff toca 1 (más un test)
r="$TMP/repo-mmdiff"; mkdir -p "$r/py/src" "$r/py/tests"; git -C "$r" init -q; git -C "$r" config user.email t@t; git -C "$r" config user.name t
echo x > "$r/py/setup.py"
printf 'def a(x):\n    return x + 1\n\ndef b(x):\n    return x - 1\n\ndef c(x):\n    return x * 2\n' > "$r/py/src/m.py"
echo "def test_a(): pass" > "$r/py/tests/test_m.py"
git -C "$r" add -A >/dev/null; git -C "$r" commit -qm base; BASE=$(git -C "$r" rev-parse HEAD)
sedi 's/return x - 1/return x - 2/' "$r/py/src/m.py"; echo "def test_b(): pass" >> "$r/py/tests/test_m.py"
git -C "$r" commit -qam head; HEAD=$(git -C "$r" rev-parse HEAD)
corre "$r" "$(stubs_para mmdiff mutmut)" STUB_MUTMUT=2
a=$(cat "$LOG")
if contains "$a" "--use-patch-file" && contains "$a" "patch: +++ b/src/m.py" && contains "$a" "patch: +    return x - 2" \
   && not_contains "$a" "return x + 1" && not_contains "$a" "return x * 2" && not_contains "$a" "test_m" \
   && not_contains "$a" "py/src" && grep -q '^cwd=.*/py$' <<<"$a" && contains "$a" "--paths-to-mutate src/m.py"; then pass mutmut_acotado_al_diff; else fail mutmut_acotado_al_diff "log=$a"; fi

# ---- stryker_rangos: rangos del lado NUEVO por hunk; varios hunks -> varios rangos en un solo --mutate
r="$TMP/repo-strange"; mkdir -p "$r/src"; git -C "$r" init -q; git -C "$r" config user.email t@t; git -C "$r" config user.name t
echo '{"devDependencies":{"@stryker-mutator/core":"9"}}' > "$r/package.json"
lineas 30 L > "$r/src/a.js"; lineas 8 M > "$r/src/b.js"
git -C "$r" add -A >/dev/null; git -C "$r" commit -qm base; BASE=$(git -C "$r" rev-parse HEAD)
# a.js: cambia L3-L4, borra L10 (los siguientes suben 1), cambia L20 (queda en la 19 nueva)
sedi -e 's/^L3$/X3/' -e 's/^L4$/X4/' -e '/^L10$/d' -e 's/^L20$/X20/' "$r/src/a.js"
sedi 's/^M5$/Y5/' "$r/src/b.js"
git -C "$r" commit -qam head; HEAD=$(git -C "$r" rev-parse HEAD)
corre "$r" "$(stubs_para strange npx)"
a=$(cat "$LOG")
if contains "$a" '--mutate src/a.js:3-4,src/a.js:19-19,src/b.js:5-5' \
   && [ "$(grep -o -- '--mutate' <<<"$a" | wc -l | tr -d ' ')" = 1 ]; then pass stryker_rangos; else fail stryker_rangos "log=$a"; fi

# ---- hunk_solo_borrado: un diff que solo borra líneas no genera rango ni llama a la herramienta
r="$TMP/repo-del"; mkdir -p "$r/src"; git -C "$r" init -q; git -C "$r" config user.email t@t; git -C "$r" config user.name t
echo '{"devDependencies":{"@stryker-mutator/core":"9"}}' > "$r/package.json"
lineas 10 L > "$r/src/c.js"; printf 'def a():\n    return 1\n\ndef b():\n    return 2\n' > "$r/src/m.py"; echo x > "$r/setup.py"
git -C "$r" add -A >/dev/null; git -C "$r" commit -qm base; BASE=$(git -C "$r" rev-parse HEAD)
sedi '/^L5$/d;/^L6$/d' "$r/src/c.js"
git -C "$r" commit -qam head; HEAD=$(git -C "$r" rev-parse HEAD)
corre "$r" "$(stubs_para del npx mutmut)" STUB_MUTMUT=2
if contains "$OUT" "MUTACIÓN: no disponible (diff sin líneas añadidas ni modificadas)" && [ ! -s "$LOG" ]; then pass hunk_solo_borrado; else fail hunk_solo_borrado "$(seccion) log=$(cat "$LOG")"; fi
# ... y lo mismo en python
r="$TMP/repo-del2"; mkdir -p "$r/src"; git -C "$r" init -q; git -C "$r" config user.email t@t; git -C "$r" config user.name t
echo x > "$r/setup.py"; printf 'def a():\n    return 1\n\ndef b():\n    return 2\n' > "$r/src/m.py"
git -C "$r" add -A >/dev/null; git -C "$r" commit -qm base; BASE=$(git -C "$r" rev-parse HEAD)
sedi '4,5d' "$r/src/m.py"
git -C "$r" commit -qam head; HEAD=$(git -C "$r" rev-parse HEAD)
corre "$r" "$(stubs_para del2 mutmut)" STUB_MUTMUT=2
if contains "$OUT" "MUTACIÓN: no disponible (diff sin líneas añadidas ni modificadas)" && not_contains "$(cat "$LOG")" "run --paths"; then pass hunk_solo_borrado_python; else fail hunk_solo_borrado_python "$(seccion) log=$(cat "$LOG")"; fi

# ---- parcial_conserva_progreso: el timeout global no descarta el conteo.
# EXO_MUTATION_SAMPLE=0: la enumeración del muestreo gasta presupuesto medido con $SECONDS
# (grano de 1 s); con TIMEOUT=1 un tic de reloj deja resta=0 y el progreso sale "no disponible".
mkrepo parpy setup.py src/m.py; r=$REPO
corre "$r" "$(stubs_para parpy mutmut)" STUB_MUTMUT=2 STUB_MODE=progresshang EXO_MUTATION_TIMEOUT=1 EXO_MUTATION_SAMPLE=0
if pgrep -f "sleep 31339" >/dev/null 2>&1; then pkill -f "sleep 31339" 2>/dev/null; fi
if contains "$OUT" "MUTACIÓN: parcial (timeout 1s) — progreso: 3/7 evaluados, 2 caught, 1 supervivientes"; then pass parcial_progreso_mutmut; else fail parcial_progreso_mutmut "$(seccion)"; fi
mkrepo parjs package.json src/a.js; r=$REPO
echo '{"devDependencies":{"@stryker-mutator/core":"9"}}' > "$r/package.json"; git -C "$r" commit -qam pkg; HEAD=$(git -C "$r" rev-parse HEAD)
corre "$r" "$(stubs_para parjs npx)" STUB_MODE=progresshang EXO_MUTATION_TIMEOUT=1
if pgrep -f "sleep 31340" >/dev/null 2>&1; then pkill -f "sleep 31340" 2>/dev/null; fi
if contains "$OUT" "MUTACIÓN: parcial (timeout 1s) — progreso: 4/7 evaluados, 3 caught, 1 supervivientes"; then pass parcial_progreso_stryker; else fail parcial_progreso_stryker "$(seccion)"; fi
corre "$r" "$(stubs_para parjs2 npx)" STUB_MODE=silenciohang EXO_MUTATION_TIMEOUT=1
if pgrep -f "sleep 31341" >/dev/null 2>&1; then pkill -f "sleep 31341" 2>/dev/null; fi
if contains "$OUT" "MUTACIÓN: parcial (timeout 1s) — progreso: no disponible"; then pass parcial_sin_progreso; else fail parcial_sin_progreso "$(seccion)"; fi

# ---- timeout_por_mutante: EXO_MUTATION_MUTANT_TIMEOUT aplica SOLO a mutmut (envuelve --runner);
# stryker conserva su timeout por defecto (no se pasa --timeoutMS) y mutmut ya no recibe -b.
mkrepo tmpy setup.py src/m.py; r=$REPO
corre "$r" "$(stubs_para tmpy mutmut)" STUB_MUTMUT=2 EXO_MUTATION_MUTANT_TIMEOUT=45
a=$(cat "$LOG")
if contains "$a" "--runner " && grep -Eq -- '--runner [^ ]+ 45 python -m pytest -x --assert=plain' <<<"$a" \
   && not_contains "$a" " -b " && contains "$OUT" "MUTACIÓN: 8/10"; then pass timeout_por_mutante_mutmut; else fail timeout_por_mutante_mutmut "$a"; fi
corre "$r" "$(stubs_para tmpy2 mutmut)" STUB_MUTMUT=2
if grep -Eq -- '--runner [^ ]+ 60 python -m pytest -x --assert=plain' "$LOG"; then pass timeout_por_mutante_defecto_60; else fail timeout_por_mutante_defecto_60 "$(cat "$LOG")"; fi
corre "$r" "$(stubs_para tmpy3 mutmut)" STUB_MUTMUT=2 EXO_MUTATION_MUTANT_TIMEOUT=xx
if grep -Eq -- '--runner [^ ]+ 60 python' "$LOG" && contains "$(seccion)" "EXO_MUTATION_MUTANT_TIMEOUT inválido"; then pass timeout_por_mutante_invalido; else fail timeout_por_mutante_invalido "$(seccion) $(cat "$LOG")"; fi
# runner efectivo = el de la config de mutmut (pyproject [tool.mutmut] antes que setup.cfg [mutmut])
mkrepo rnpy pyproject.toml src/m.py; r=$REPO
printf '[tool.mutmut]\nrunner = "python -m pytest -x -q tests/unit"\n' > "$r/pyproject.toml"; git -C "$r" commit -qam cfg; HEAD=$(git -C "$r" rev-parse HEAD)
corre "$r" "$(stubs_para rnpy mutmut)" STUB_MUTMUT=2
if grep -Eq -- '--runner [^ ]+ 60 python -m pytest -x -q tests/unit( |$)' "$LOG"; then pass runner_config_pyproject; else fail runner_config_pyproject "$(cat "$LOG")"; fi
mkrepo rncfg setup.py src/m.py; r=$REPO
printf '[mutmut]\nrunner=python -m pytest -x tests/rapidos\n' > "$r/setup.cfg"; git -C "$r" add setup.cfg; git -C "$r" commit -qm cfg; HEAD=$(git -C "$r" rev-parse HEAD)
corre "$r" "$(stubs_para rncfg mutmut)" STUB_MUTMUT=2
if grep -Eq -- '--runner [^ ]+ 60 python -m pytest -x tests/rapidos( |$)' "$LOG"; then pass runner_config_setup_cfg; else fail runner_config_setup_cfg "$(cat "$LOG")"; fi
mkrepo js1 package.json src/a.js; r=$REPO
echo '{"devDependencies":{"@stryker-mutator/core":"9"}}' > "$r/package.json"; git -C "$r" commit -qam pkg; HEAD=$(git -C "$r" rev-parse HEAD)
corre "$r" "$(stubs_para tmjs npx)" EXO_MUTATION_MUTANT_TIMEOUT=45
if not_contains "$(cat "$LOG")" "timeoutMS" && contains "$(cat "$LOG")" "stryker run"; then pass stryker_sin_timeoutMS; else fail stryker_sin_timeoutMS "$(cat "$LOG")"; fi

# ---- mutmut_sospechoso_cuenta_caught (stub): 🤔 es un mutante MATADO lento -> cuenta en det y en el progreso
mkrepo susps setup.py src/m.py; r=$REPO
corre "$r" "$(stubs_para susps mutmut)" STUB_MUTMUT=2 STUB_MODE=suspicious
if contains "$OUT" "MUTACIÓN: 8/10 (80%) — 2 superviviente(s)"; then pass mutmut_sospechoso_cuenta_stub; else fail mutmut_sospechoso_cuenta_stub "$(seccion)"; fi
corre "$r" "$(stubs_para susps2 mutmut)" STUB_MUTMUT=2 STUB_MODE=progresshang2 EXO_MUTATION_TIMEOUT=1 EXO_MUTATION_SAMPLE=0
if pgrep -f "sleep 31342" >/dev/null 2>&1; then pkill -f "sleep 31342" 2>/dev/null; fi
if contains "$OUT" "progreso: 4/7 evaluados, 3 caught, 1 supervivientes"; then pass parcial_progreso_sospechoso; else fail parcial_progreso_sospechoso "$(seccion)"; fi

# ---- gitconfig_hostil: el git del usuario no rompe rangos ni patches
cat > "$TMP/hostil.gitconfig" <<'CFG'
[diff]
	noprefix = true
	mnemonicPrefix = true
	renames = copies
[color]
	ui = always
CFG
r="$TMP/repo-hostil"; mkdir -p "$r/src"; git -C "$r" init -q; git -C "$r" config user.email t@t; git -C "$r" config user.name t
echo '{"devDependencies":{"@stryker-mutator/core":"9"}}' > "$r/package.json"
lineas 30 L > "$r/src/a.js"; lineas 8 M > "$r/src/b.js"
git -C "$r" add -A >/dev/null; git -C "$r" commit -qm base; BASE=$(git -C "$r" rev-parse HEAD)
sedi -e 's/^L3$/X3/' -e 's/^L4$/X4/' -e '/^L10$/d' -e 's/^L20$/X20/' "$r/src/a.js"; sedi 's/^M5$/Y5/' "$r/src/b.js"
git -C "$r" commit -qam head; HEAD=$(git -C "$r" rev-parse HEAD)
corre "$r" "$(stubs_para hostil npx)" GIT_CONFIG_GLOBAL="$TMP/hostil.gitconfig"
if contains "$(cat "$LOG")" '--mutate src/a.js:3-4,src/a.js:19-19,src/b.js:5-5'; then pass gitconfig_hostil_rangos; else fail gitconfig_hostil_rangos "log=$(cat "$LOG")"; fi
mkrepo hostilc Cargo.toml src/lib.rs; r=$REPO
corre "$r" "$(stubs_para hostilc cargo-mutants)" GIT_CONFIG_GLOBAL="$TMP/hostil.gitconfig"
if contains "$(cat "$LOG")" "+++ b/src/lib.rs" && contains "$OUT" "MUTACIÓN: 8/10"; then pass gitconfig_hostil_cargo; else fail gitconfig_hostil_cargo "log=$(cat "$LOG")"; fi
mkrepo hostilp setup.py src/m.py; r=$REPO
corre "$r" "$(stubs_para hostilp mutmut)" STUB_MUTMUT=2 GIT_CONFIG_GLOBAL="$TMP/hostil.gitconfig"
if contains "$(cat "$LOG")" "patch: +++ b/src/m.py" && contains "$(cat "$LOG")" "--paths-to-mutate src/m.py"; then pass gitconfig_hostil_mutmut; else fail gitconfig_hostil_mutmut "log=$(cat "$LOG")"; fi

# ---- exclude_pathspec: EXO_MUTATION_EXCLUDE (pathspecs separados por comas) quita ficheros antes de agrupar
mkrepo excl setup.py src/m.py alembic/versions/001_x.py src/api_pb2.py src/gen/z.py; r=$REPO
corre "$r" "$(stubs_para excl mutmut)" STUB_MUTMUT=2 EXO_MUTATION_EXCLUDE='alembic/versions/**,*_pb2.py, src/gen'
a=$(cat "$LOG")
if contains "$a" "--paths-to-mutate src/m.py" && not_contains "$a" "001_x" && not_contains "$a" "_pb2" && not_contains "$a" "gen/z"; then pass exclude_pathspec; else fail exclude_pathspec "log=$a"; fi
corre "$r" "$(stubs_para excl2 mutmut)" STUB_MUTMUT=2
if contains "$(cat "$LOG")" "001_x.py" && contains "$(cat "$LOG")" "_pb2.py"; then pass exclude_vacio_por_defecto; else fail exclude_vacio_por_defecto "log=$(cat "$LOG")"; fi
corre "$r" "$(stubs_para excl3 mutmut)" STUB_MUTMUT=2 EXO_MUTATION_EXCLUDE='src,alembic'
if contains "$OUT" "MUTACIÓN: no disponible (diff sin código de producción)" && [ ! -s "$LOG" ]; then pass exclude_todo; else fail exclude_todo "$(seccion) log=$(cat "$LOG")"; fi

# ---- muestreo por adelantado (mutmut 2.x): enumerar sin tests, elegir n ids, correr solo esos
runids() { grep '^runid=' "$LOG" | sed 's/^runid=//'; }
# Los casos con la muestra de 150 cuestan ~150 procesos de mutmut cada uno: en Windows (fork
# caro) ~17 min en total, fuera del presupuesto del job. Allí se saltan, con [SKIP] visible:
# la lógica es bash sin nada del SO, y lo propio de Windows (CRLF del python) lo cubre
# muestra_python_crlf con n=5. Todos pasaron en windows-latest (run 36671923207).
case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) N150=0 ;; *) N150=1 ;; esac
salta_n150() { for t in "$@"; do printf '[SKIP] %s — muestra de 150: cara en Windows, solo en Unix\n' "$t"; done; }
mkrepo mues setup.py src/m.py; r=$REPO
if [ "$N150" = 1 ]; then
corre "$r" "$(stubs_para mues1 mutmut)" STUB_MUTMUT=2 EXO_MUTATION_TIMEOUT=120 STUB_MUTANTS=400
ids1=$(runids); s=$(seccion)
if [ "$(wc -l <<<"$ids1" | tr -d ' ')" = 150 ] && [ "$(sort -u <<<"$ids1" | wc -l | tr -d ' ')" = 150 ] \
   && [ "$(sort -n <<<"$ids1" | tail -n 1)" -le 400 ] && [ "$(sort -n <<<"$ids1" | head -n 1)" -ge 1 ] \
   && contains "$s" "muestra de 150/400 (semilla 20260929)" && contains "$s" "IC95 [" \
   && contains "$(cat "$LOG")" "--runner true" && contains "$(cat "$LOG")" "--tests-dir"; then pass muestra_por_encima_de_n; else fail muestra_por_encima_de_n "n=$(wc -l <<<"$ids1") $s"; fi
corre "$r" "$(stubs_para mues2 mutmut)" STUB_MUTMUT=2 EXO_MUTATION_TIMEOUT=120 STUB_MUTANTS=400; ids2=$(runids)
corre "$r" "$(stubs_para mues3 mutmut)" STUB_MUTMUT=2 EXO_MUTATION_TIMEOUT=120 STUB_MUTANTS=400 EXO_MUTATION_SEED=7; ids3=$(runids)
if [ -n "$ids1" ] && [ "$ids1" = "$ids2" ] && [ "$ids1" != "$ids3" ] && [ "$(wc -l <<<"$ids3" | tr -d ' ')" = 150 ]; then pass muestra_reproducible; else fail muestra_reproducible "misma semilla iguales=$([ "$ids1" = "$ids2" ] && echo si || echo no) otra semilla iguales=$([ "$ids1" = "$ids3" ] && echo si || echo no)"; fi
else salta_n150 muestra_por_encima_de_n muestra_reproducible; fi
# python con CRLF (el de Windows): `5\r` no es un id; sin quitar el \r cada id caía en la
# corrida completa y el agregado salía 1193/150.
p=$(stubs_para crlf mutmut); py=$(command -v python3); rm -f "$p/python3"   # es un symlink: no escribir a través
printf '#!/bin/sh\n%s "$@" | sed "s/\\$/\\r/"\n' "'$py'" > "$p/python3"; chmod +x "$p/python3"
corre "$r" "$p" STUB_MUTMUT=2 EXO_MUTATION_TIMEOUT=120 STUB_MUTANTS=400 EXO_MUTATION_SAMPLE=5
if [ "$(runids | grep -cE '^[0-9]+$')" = 5 ] && contains "$(seccion)" "MUTACIÓN: 5/5 (100,0%)"; then pass muestra_python_crlf; else fail muestra_python_crlf "runids=$(runids | od -c | head -n 2) $(seccion)"; fi
# por debajo de N: corrida completa de siempre (sin IC, sin ids sueltos)
corre "$r" "$(stubs_para mues4 mutmut)" STUB_MUTMUT=2 EXO_MUTATION_TIMEOUT=120 STUB_MUTANTS=100
if contains "$OUT" "MUTACIÓN: 8/10 (80%) — 2 superviviente(s)" && not_contains "$OUT" "IC95" && [ -z "$(runids)" ] \
   && contains "$(cat "$LOG")" "--use-patch-file"; then pass por_debajo_de_n; else fail por_debajo_de_n "$(seccion)"; fi
# EXO_MUTATION_SAMPLE=0: 1.5.1 tal cual (ni enumeración ni ids)
corre "$r" "$(stubs_para mues5 mutmut)" STUB_MUTMUT=2 EXO_MUTATION_TIMEOUT=120 STUB_MUTANTS=400 EXO_MUTATION_SAMPLE=0
if contains "$OUT" "MUTACIÓN: 8/10 (80%) — 2 superviviente(s)" && not_contains "$OUT" "IC95" && [ -z "$(runids)" ] \
   && not_contains "$(cat "$LOG")" "--runner true"; then pass muestreo_desactivado; else fail muestreo_desactivado "$(seccion) log=$(cat "$LOG")"; fi
# Wilson: 33/150 = [16,1%, 29,3%] (el intervalo normal daría [15,2%, 28,8%]); bordes c=0 y c=n sin NaN
if [ "$N150" = 1 ]; then
corre "$r" "$(stubs_para wil1 mutmut)" STUB_MUTMUT=2 EXO_MUTATION_TIMEOUT=120 STUB_MUTANTS=400 STUB_KILL_COUNT=33
w1=$(seccion)
corre "$r" "$(stubs_para wil2 mutmut)" STUB_MUTMUT=2 EXO_MUTATION_TIMEOUT=120 STUB_MUTANTS=400 STUB_KILL_COUNT=0
w2=$(seccion)
corre "$r" "$(stubs_para wil3 mutmut)" STUB_MUTMUT=2 EXO_MUTATION_TIMEOUT=120 STUB_MUTANTS=400 STUB_KILL_COUNT=150
w3=$(seccion)
if contains "$w1" "33/150 (22,0%) IC95 [16,1%, 29,3%] — muestra de 150/400" \
   && contains "$w2" "0/150 (0,0%) IC95 [0,0%, 2,5%]" && contains "$w3" "150/150 (100,0%) IC95 [97,5%, 100,0%]" \
   && not_contains "$w2$w3" "nan" && not_contains "$w2$w3" "inf"; then pass wilson_correcto; else fail wilson_correcto "$w1 | $w2 | $w3"; fi
else salta_n150 wilson_correcto; fi
# sin caché previa: dos corridas seguidas comparten el estado de mutmut (STUB_CACHE, el equivalente a
# un .mutmut-cache que sobreviviera); la segunda no puede leer el baseline ~0 s de la enumeración
# (⏰ inventados) ni acabar en <1 s. 5 ids x 0,3 s = 1,5 s reales cada corrida.
t0=$SECONDS; corre "$r" "$(stubs_para cache1 mutmut)" STUB_MUTMUT=2 EXO_MUTATION_TIMEOUT=120 STUB_MUTANTS=20 EXO_MUTATION_SAMPLE=5 STUB_ID_SLEEP=0.3 STUB_KILL_COUNT=3 STUB_CACHE="$TMP/cache-compartida"; d1=$((SECONDS - t0)); c1=$(seccion)
t0=$SECONDS; corre "$r" "$(stubs_para cache2 mutmut)" STUB_MUTMUT=2 EXO_MUTATION_TIMEOUT=120 STUB_MUTANTS=20 EXO_MUTATION_SAMPLE=5 STUB_ID_SLEEP=0.3 STUB_KILL_COUNT=3 STUB_CACHE="$TMP/cache-compartida"; d2=$((SECONDS - t0)); c2=$(seccion)
if [ "$d1" -ge 1 ] && [ "$d2" -ge 1 ] && contains "$c1" "3/5 (60,0%)" && contains "$c2" "3/5 (60,0%)"; then pass sin_cache_previa; else fail sin_cache_previa "d1=$d1 d2=$d2 $c1 | $c2"; fi
# timeout global dentro de la muestra: parcial con el progreso de la muestra
corre "$r" "$(stubs_para mues6 mutmut)" STUB_MUTMUT=2 STUB_MUTANTS=400 EXO_MUTATION_SAMPLE=5 STUB_HANG_AFTER=2 EXO_MUTATION_TIMEOUT=3
if pgrep -f "sleep 31344" >/dev/null 2>&1; then pkill -f "sleep 31344" 2>/dev/null; fi
if contains "$OUT" "MUTACIÓN: parcial (timeout 3s) — progreso: 2/5 evaluados, 2 caught, 0 supervivientes (muestra de 5/400, semilla 20260929)"; then pass timeout_en_muestra; else fail timeout_en_muestra "$(seccion)"; fi
# valores inválidos: aviso visible y se usan los de siempre
if [ "$N150" = 1 ]; then
corre "$r" "$(stubs_para mues7 mutmut)" STUB_MUTMUT=2 EXO_MUTATION_TIMEOUT=120 STUB_MUTANTS=400 EXO_MUTATION_SAMPLE=abc EXO_MUTATION_SEED=x1
if contains "$(seccion)" "EXO_MUTATION_SAMPLE inválido" && contains "$(seccion)" "EXO_MUTATION_SEED inválido" \
   && contains "$(seccion)" "muestra de 150/400 (semilla 20260929)"; then pass muestreo_env_invalido; else fail muestreo_env_invalido "$(seccion)"; fi
# monorepo: el proyecto muestreado se reporta aparte y NO entra en el agregado
mkrepo mues8 "" engine/Cargo.toml engine/src/lib.rs py/setup.py py/src/m.py; r2=$REPO
corre "$r2" "$(stubs_para mues8 cargo-mutants mutmut)" STUB_MUTMUT=2 EXO_MUTATION_TIMEOUT=120 STUB_MUTANTS=400
s=$(seccion); primera=$(sed -n '2p' <<<"$s")
if contains "$primera" "MUTACIÓN: 8/10 (80%) — 2 superviviente(s); 1 proyecto(s) sin score (1 muestreado(s), aparte)" \
   && contains "$s" "MUTACIÓN [engine]: 8/10" && contains "$s" "MUTACIÓN [py]: " && contains "$s" "IC95 [" && contains "$s" "muestra de 150/400"; then pass muestra_fuera_del_agregado; else fail muestra_fuera_del_agregado "$s"; fi
# selección EXACTA del protocolo del verdict 2: random.Random(20260929).sample(sorted(ids), 150) sobre 1..285
# (bizkaia-muestra.txt: un id por línea, en el orden del sample)
mkrepo prot setup.py src/m.py; r=$REPO
MUESTRA_V2="${SCRIPT_DIR}/../../../evals/pipeline-a-plus/linea-base-2/bizkaia-muestra.txt"
if [ -f "$MUESTRA_V2" ]; then
  corre "$r" "$(stubs_para proto mutmut)" STUB_MUTMUT=2 EXO_MUTATION_TIMEOUT=120 STUB_MUTANTS=285
  if [ "$(runids | sort -n)" = "$(sort -n "$MUESTRA_V2")" ] && [ "$(runids | wc -l | tr -d ' ')" = 150 ]; then pass muestra_igual_protocolo; else fail muestra_igual_protocolo "la selección difiere de bizkaia-muestra.txt"; fi
else
  printf '[SKIP] muestra_igual_protocolo — no está %s\n' "$MUESTRA_V2"
fi
else salta_n150 muestreo_env_invalido muestra_fuera_del_agregado muestra_igual_protocolo; mkrepo prot setup.py src/m.py; r=$REPO; fi
# degradaciones nunca mudas
corre "$r" "$(stubs_para dg1 mutmut)" STUB_MUTMUT=2 EXO_MUTATION_TIMEOUT=120 STUB_MUTANTS=0
if contains "$(seccion)" "MUTACIÓN: 8/10 (80%)" && contains "$(seccion)" "(muestreo no disponible: sin ids de mutmut; corrida completa)"; then pass degrada_sin_ids; else fail degrada_sin_ids "$(seccion)"; fi
corre "$r" "$(stubs_para dg2 mutmut)" STUB_MUTMUT=2 EXO_MUTATION_TIMEOUT=120 STUB_MUTANTS=400 STUB_IDS_RAW="ids: a b c"
if contains "$(seccion)" "MUTACIÓN: 8/10 (80%)" && contains "$(seccion)" "(muestreo no disponible: sin ids de mutmut; corrida completa)" && [ -z "$(runids)" ]; then pass degrada_formato_distinto; else fail degrada_formato_distinto "$(seccion)"; fi
pdg=$(stubs_para dg3 mutmut); rm -f "$pdg/python3"
corre "$r" "$pdg" STUB_MUTMUT=2 EXO_MUTATION_TIMEOUT=120 STUB_MUTANTS=400
if contains "$(seccion)" "MUTACIÓN: 8/10 (80%)" && contains "$(seccion)" "(muestreo no disponible: sin python de mutmut; corrida completa)" && [ -z "$(runids)" ]; then pass degrada_sin_python; else fail degrada_sin_python "$(seccion)"; fi
# cargo y stryker: con más de N mutantes, "no aplica"; por debajo de N, nada
mkrepo dg4 Cargo.toml src/lib.rs; r4=$REPO
corre "$r4" "$(stubs_para dg4 cargo-mutants)" EXO_MUTATION_SAMPLE=5
if contains "$(seccion)" "MUTACIÓN: 8/10 (80%) — 2 superviviente(s) (muestreo no aplica: cargo-mutants)"; then pass cargo_no_aplica_sobre_n; else fail cargo_no_aplica_sobre_n "$(seccion)"; fi
corre "$r4" "$(stubs_para dg5 cargo-mutants)"
if contains "$(seccion)" "MUTACIÓN: 8/10" && not_contains "$(seccion)" "muestreo"; then pass cargo_sin_nota_bajo_n; else fail cargo_sin_nota_bajo_n "$(seccion)"; fi
mkrepo dg6 package.json src/a.js; r5=$REPO
echo '{"devDependencies":{"@stryker-mutator/core":"9"}}' > "$r5/package.json"; git -C "$r5" commit -qam pkg; HEAD=$(git -C "$r5" rev-parse HEAD)
corre "$r5" "$(stubs_para dg6 npx)" EXO_MUTATION_SAMPLE=2
if contains "$(seccion)" "MUTACIÓN: 3/4" && contains "$(seccion)" "(muestreo no aplica: stryker)"; then pass stryker_no_aplica_sobre_n; else fail stryker_no_aplica_sobre_n "$(seccion)"; fi
corre "$r5" "$(stubs_para dg7 npx)"
if contains "$(seccion)" "MUTACIÓN: 3/4" && not_contains "$(seccion)" "muestreo"; then pass stryker_sin_nota_bajo_n; else fail stryker_sin_nota_bajo_n "$(seccion)"; fi

# cargo no muestrea: sale igual, sin IC
mkrepo mues9 Cargo.toml src/lib.rs; r3=$REPO
corre "$r3" "$(stubs_para mues9 cargo-mutants)"
if contains "$OUT" "MUTACIÓN: 8/10" && not_contains "$OUT" "IC95"; then pass cargo_no_muestrea; else fail cargo_no_muestrea "$(seccion)"; fi

# ---- tests con el mutmut 2.x REAL (se saltan si no hay uno con whatthepatch y pytest en el PATH del llamador)
REAL_DIR=""
if _mm=$(command -v mutmut 2>/dev/null) && [ -x "$_mm" ]; then
  _d=$(dirname "$_mm")
  if "$_mm" run --help 2>&1 | grep -q -- '--paths-to-mutate' && "$_d/python" -c 'import whatthepatch, pytest' >/dev/null 2>&1; then REAL_DIR=$_d; fi
fi
if [ -z "$REAL_DIR" ]; then
  printf '[SKIP] mutmut_sospechoso_cuenta_caught, mutmut_timeout_real, mutmut_muestreo_real — sin mutmut 2.x real (con whatthepatch+pytest) en PATH\n'
else
  mkreal() { # NOMBRE  (el contenido de m.py y del test lo escribe el llamador antes de commitear con `realcommit`)
    REPO="$TMP/repo-$1"; mkdir -p "$REPO/pkg/tests"; git -C "$REPO" init -q; git -C "$REPO" config user.email t@t; git -C "$REPO" config user.name t
    printf '[project]\nname="p"\n' > "$REPO/pkg/pyproject.toml"
  }
  # sospechoso: el mutante se mata pero tarda > 2x el baseline (🤔). Es un mutante MATADO.
  mkreal susp
  printf 'V = 0\n' > "$REPO/pkg/m.py"
  printf 'import sys, os\nsys.path.insert(0, os.path.dirname(os.path.dirname(__file__)))\nimport m\ndef test_f():\n    assert m.V == 1\n' > "$REPO/pkg/tests/test_m.py"
  git -C "$REPO" add -A >/dev/null; git -C "$REPO" commit -qm base; BASE=$(git -C "$REPO" rev-parse HEAD)
  printf 'V = 1\n' > "$REPO/pkg/m.py"
  printf 'import sys, os, time\nsys.path.insert(0, os.path.dirname(os.path.dirname(__file__)))\nimport m\ndef test_f():\n    time.sleep(0.3)\n    if m.V != 1:\n        time.sleep(4)\n    assert m.V == 1\n' > "$REPO/pkg/tests/test_m.py"
  git -C "$REPO" add -A >/dev/null; git -C "$REPO" commit -qm head; HEAD=$(git -C "$REPO" rev-parse HEAD)
  corre "$REPO" "$(stubs_para real1):$REAL_DIR"
  if contains "$OUT" "MUTACIÓN: 2/2 (100%) — 0 superviviente(s)"; then pass mutmut_sospechoso_cuenta_caught; else fail mutmut_sospechoso_cuenta_caught "$(seccion)"; fi

  # timeout real por mutante: baseline lento (~2.5 s => corte propio de mutmut ~25 s por mutante);
  # con EXO_MUTATION_MUTANT_TIMEOUT=5 cada mutante colgado se corta en ~5 s y cuenta como caught.
  mkreal slow
  printf 'def f(x):\n    return x + 1\n' > "$REPO/pkg/m.py"
  printf 'import sys, os\nsys.path.insert(0, os.path.dirname(os.path.dirname(__file__)))\nfrom m import f\ndef test_f():\n    assert f(1) == 3\n' > "$REPO/pkg/tests/test_m.py"
  git -C "$REPO" add -A >/dev/null; git -C "$REPO" commit -qm base; BASE=$(git -C "$REPO" rev-parse HEAD)
  printf 'def f(x):\n    return x + 2\n' > "$REPO/pkg/m.py"
  printf 'import sys, os, time\nsys.path.insert(0, os.path.dirname(os.path.dirname(__file__)))\nfrom m import f\ndef test_f():\n    time.sleep(2.5)\n    if f(1) != 3:\n        time.sleep(31343)\n    assert f(1) == 3\n' > "$REPO/pkg/tests/test_m.py"
  git -C "$REPO" add -A >/dev/null; git -C "$REPO" commit -qm head; HEAD=$(git -C "$REPO" rev-parse HEAD)
  t0=$SECONDS
  corre "$REPO" "$(stubs_para real2):$REAL_DIR" EXO_MUTATION_MUTANT_TIMEOUT=5 EXO_MUTATION_TIMEOUT=120
  dt=$((SECONDS - t0))
  if contains "$OUT" "(100%) — 0 superviviente(s)" && [ "$dt" -lt 60 ]; then pass mutmut_timeout_real; else fail mutmut_timeout_real "dt=${dt}s $(seccion)"; fi

  # muestreo real: enumerar sin tests + un id con el runner real (2+ mutantes, muestra de 1)
  mkreal samp
  printf 'def f(x):\n    return x + 1\n' > "$REPO/pkg/m.py"
  printf 'import sys, os\nsys.path.insert(0, os.path.dirname(os.path.dirname(__file__)))\nfrom m import f\ndef test_f():\n    assert f(1) == 1\n' > "$REPO/pkg/tests/test_m.py"
  git -C "$REPO" add -A >/dev/null; git -C "$REPO" commit -qm base; BASE=$(git -C "$REPO" rev-parse HEAD)
  printf 'def f(x):\n    return x + 2\n' > "$REPO/pkg/m.py"
  printf 'import sys, os\nsys.path.insert(0, os.path.dirname(os.path.dirname(__file__)))\nfrom m import f\ndef test_f():\n    assert f(1) == 3\n' > "$REPO/pkg/tests/test_m.py"
  git -C "$REPO" add -A >/dev/null; git -C "$REPO" commit -qm head; HEAD=$(git -C "$REPO" rev-parse HEAD)
  corre "$REPO" "$(stubs_para real3):$REAL_DIR" EXO_MUTATION_SAMPLE=1 EXO_MUTATION_TIMEOUT=120
  if contains "$OUT" "IC95 [" && contains "$OUT" "muestra de 1/" && contains "$OUT" "(semilla 20260929)"; then pass mutmut_muestreo_real; else fail mutmut_muestreo_real "$(seccion)"; fi
fi

# ---- mutmut sin whatthepatch: --use-patch-file falla con ImportError -> no disponible visible, no "salida no reconocida"
mkrepo nowtp setup.py src/m.py; r=$REPO
corre "$r" "$(stubs_para nowtp mutmut)" STUB_MUTMUT=2 STUB_MODE=nowhatthepatch
if contains "$OUT" "MUTACIÓN: no disponible (mutmut sin whatthepatch: pip install 'mutmut[patch]')"; then pass mutmut_sin_whatthepatch; else fail mutmut_sin_whatthepatch "$(seccion)"; fi

printf '\n%d pass, %d fail\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
