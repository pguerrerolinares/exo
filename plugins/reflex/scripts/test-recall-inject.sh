#!/usr/bin/env bash
# Test standalone para recall-inject.sh (hook UserPromptSubmit de M6-06).
# Fixtures en mktemp -d; nunca toca la KB real, el índice real ni el log real.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/recall-inject.sh"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# Aislamiento global: ningún test escribe en el log real de reflex.
export REFLEX_LOG_FILE="$TMP/reflex-log.jsonl"

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }
contains()     { case "$1" in *"$2"*) return 0 ;; *) return 1 ;; esac; }
not_contains() { case "$1" in *"$2"*) return 1 ;; *) return 0 ;; esac; }

# Invoca el hook y deja el resultado en HOOK_OUT / HOOK_RC.
# NO devuelve por stdout a propósito: `out="$(run_hook …)"` correría la función
# en un subshell y el exit code se perdería por el camino.
# $2 (opcional) = binario exo a usar en ESTA llamada, sin contaminar las demás.
run_hook() {  # $1 = prompt  [$2 = exo_bin]
  local bin="${2:-$EXO_BIN}"
  printf '%s' "$1" | jq -Rs '{prompt:., session_id:"test-sess"}' \
    | EXO_BIN="$bin" "$HOOK" > "$TMP/hook-out.txt" 2>/dev/null
  HOOK_RC=$?
  HOOK_OUT="$(cat "$TMP/hook-out.txt" 2>/dev/null)"
}

# --- Un índice falso que existe (para que los guards no aborten por no-index) ---
FAKE_DB="$TMP/index.db"
: > "$FAKE_DB"
export EXO_INDEX="$FAKE_DB"

# --- Binario exo falso: registra su invocación y no devuelve nada ---
FAKE_EXO="$TMP/exo-silent"
cat > "$FAKE_EXO" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$EXO_CALLS"
exit 1
EOF
chmod +x "$FAKE_EXO"
export EXO_CALLS="$TMP/calls.txt"
export EXO_BIN="$FAKE_EXO"

# ---------------------------------------------------------------- T1: gate ---
# Observable: un binario INEXISTENTE. Si el gate salta, el hook sale antes de
# los guards y no hay evento; si dispara, llega al guard y loguea `no-engine`.
# Así la Task 1 se verifica entera sin depender de la búsqueda (Task 2).
NO_BIN="$TMP/no-existe"

gate_dispara() {  # $1 = prompt
  : > "$REFLEX_LOG_FILE"
  run_hook "$1" "$NO_BIN"
  grep -q 'no-engine' "$REFLEX_LOG_FILE" 2>/dev/null
}

for p in "M6-06" "vamos con brainstorm de M6-06" "el trinquete"; do
  if gate_dispara "$p"; then pass "gate: '$p' dispara"
  else fail "gate: '$p' dispara" "no llegó a los guards"; fi
done

# Calla: todos los tokens son stopword/ack/git/numérico.
for p in "dale" "sí, hazlo" "ok gracias" "pushea los dos repos" "mergea a master" "1 2 3"; do
  if ! gate_dispara "$p"; then pass "gate: '$p' calla"
  else fail "gate: '$p' calla" "llegó a los guards"; fi
done

# Calla: turnos no humanos y comandos al harness.
for p in "<teammate-message>algo largo aquí</teammate-message>" "/compact" "!ls -la"; do
  if ! gate_dispara "$p"; then pass "gate: '${p:0:20}' calla"
  else fail "gate: '${p:0:20}' calla" "llegó a los guards"; fi
done

# Normalización: acentos y mayúsculas no crean tokens con contenido.
if ! gate_dispara "SÍ, DALE"; then pass "gate: normaliza acentos y mayúsculas"
else fail "gate: normaliza acentos y mayúsculas" "llegó a los guards"; fi

# --------------------------------------------------- T1: P1 (nunca rompe) ---
# Binario que sale con 2, que revienta, que escupe basura: exit 0 y sin bloque.
for modo in "exit 2" "kill -TERM \$\$" "printf 'basura no-json'"; do
  BAD="$TMP/exo-bad"
  printf '#!/usr/bin/env bash\n%s\n' "$modo" > "$BAD"
  chmod +x "$BAD"
  run_hook "M6-06" "$BAD"
  if [ "$HOOK_RC" -eq 0 ]; then pass "P1: exit 0 con binario '$modo'"
  else fail "P1: exit 0 con binario '$modo'" "rc=$HOOK_RC"; fi
done

# Binario ausente: exit 0, sin bloque, y degradación logueada.
: > "$REFLEX_LOG_FILE"
run_hook "M6-06" "$NO_BIN"
if [ "$HOOK_RC" -eq 0 ] && [ -z "$HOOK_OUT" ]; then pass "P1: binario ausente ⇒ exit 0 sin bloque"
else fail "P1: binario ausente ⇒ exit 0 sin bloque" "rc=$HOOK_RC out='$HOOK_OUT'"; fi
if grep -q 'no-engine' "$REFLEX_LOG_FILE" 2>/dev/null; then pass "log: no-engine"
else fail "log: no-engine" "sin evento en $REFLEX_LOG_FILE"; fi

# ------------------------------------------------- T1: P6 (stdout limpio) ---
# Cuando el gate salta, el stdout es EXACTAMENTE vacío.
run_hook "dale"
if [ -z "$HOOK_OUT" ]; then pass "P6: gate que salta no escribe nada en stdout"
else fail "P6: gate que salta no escribe nada en stdout" "out='$HOOK_OUT'"; fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
