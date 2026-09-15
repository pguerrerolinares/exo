#!/usr/bin/env bash
# Test standalone para exo-recall.sh (hook SessionStart). Fixtures en mktemp -d;
# nunca toca el HOME, el índice ni el log reales.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/exo-recall.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
export HOME="$TMP/home"
mkdir -p "$HOME/.claude"
export REFLEX_LOG_FILE="$HOME/.claude/reflex-log.jsonl"

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }
contains() { case "$1" in *"$2"*) return 0 ;; *) return 1 ;; esac; }

run_hook() {  # $1 = JSON de entrada; resto = argumentos de env (VAR=valor, -u VAR)
  local input="$1"; shift
  printf '%s' "$input" | env "$@" "$HOOK" > "$TMP/out.txt" 2>/dev/null
  HOOK_OUT="$(cat "$TMP/out.txt" 2>/dev/null)"
}

# ------------------- H5 (T): la reafirmación tras compactar mira solo la cola ---
# 2.500 líneas: un git-c de sess-x FUERA de la ventana de 2.000 y un
# verify-before-commit de sess-x DENTRO. Solo el segundo tiene que reforzarse.
LOGC="$HOME/.claude/reflex-log.jsonl"
{
  printf '{"ts":"t","reflex":"git-c","session_id":"sess-x","agent_id":"","agent_type":"","tool":"Bash","payload":"viejo"}\n'
  awk 'BEGIN { for (i = 0; i < 2498; i++) printf "{\"ts\":\"t\",\"reflex\":\"zero-residuo\",\"session_id\":\"otra-%d\",\"agent_id\":\"\",\"agent_type\":\"\",\"tool\":\"Bash\",\"payload\":\"x\"}\n", i }'
  printf '{"ts":"t","reflex":"verify-before-commit","session_id":"sess-x","agent_id":"","agent_type":"","tool":"Bash","payload":"nuevo"}\n'
} > "$LOGC"
run_hook '{"session_id":"sess-x","source":"compact"}' EXO_BIN="$TMP/no-existe"
CTX="$(printf '%s' "$HOOK_OUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"
if contains "$CTX" "verifica (corre el cambio)"; then pass "H5: refuerza lo disparado dentro de la ventana"
else fail "H5: refuerza lo disparado dentro de la ventana" "ctx='$CTX'"; fi
if ! contains "$CTX" "git -C X"; then pass "H5: no escanea más allá de la ventana de 2000 líneas"
else fail "H5: no escanea más allá de la ventana de 2000 líneas" "ctx='$CTX'"; fi

# ------------------- helpers para leer el log de fallback -----------------
ultimo_evento() { tail -1 "$LOGC" 2>/dev/null | jq -r '.reflex // empty' 2>/dev/null; }
ultimo_payload() { tail -1 "$LOGC" 2>/dev/null | jq -r '.payload // empty' 2>/dev/null; }

# ------------------- no-engine: EXO_BIN no ejecutable ----------------------
: > "$LOGC"
run_hook '{"session_id":"sess-ne"}' EXO_BIN="$TMP/no-existe-bin"
EV_NE="$(ultimo_evento)"; PL_NE="$(ultimo_payload)"
if [ "$EV_NE" = "recall-fallback" ] && contains "$PL_NE" "reason=no-engine"; then
  pass "no-engine: EXO_BIN no ejecutable ⇒ recall-fallback reason=no-engine"
else
  fail "no-engine: EXO_BIN no ejecutable ⇒ recall-fallback reason=no-engine" \
    "evento=$EV_NE payload=$PL_NE"
fi

# --- orden: sin engine, UNA sola causa (el bug histórico: no-config Y ------
# no-engine para la misma ausencia de binario, Task 8 de la ola 1B) ---------
LINEAS_NE="$(wc -l < "$LOGC" | tr -d ' ')"
if [ "$LINEAS_NE" -eq 1 ] && ! grep -q 'reason=no-config' "$LOGC"; then
  pass "orden: sin engine, se loguea SOLO no-engine (nunca no-config además)"
else
  fail "orden: sin engine, se loguea SOLO no-engine (nunca no-config además)" \
    "lineas=$LINEAS_NE log=$(cat "$LOGC")"
fi

# ------------------- no-index: EXO_BIN ejecutable, EXO_INDEX ausente ------
STUB_OK="$TMP/exo-stub-ok"
cat > "$STUB_OK" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
chmod +x "$STUB_OK"

: > "$LOGC"
run_hook '{"session_id":"sess-ni"}' EXO_BIN="$STUB_OK" EXO_INDEX="$TMP/no-existe.db"
EV_NI="$(ultimo_evento)"; PL_NI="$(ultimo_payload)"
if [ "$EV_NI" = "recall-fallback" ] && contains "$PL_NI" "reason=no-index"; then
  pass "no-index: EXO_INDEX ausente ⇒ recall-fallback reason=no-index"
else
  fail "no-index: EXO_INDEX ausente ⇒ recall-fallback reason=no-index" \
    "evento=$EV_NI payload=$PL_NI"
fi

# ------------------- no-config: exo config --json no resuelve nombre ------
: > "$LOGC"
touch "$TMP/index-vacio.db"
run_hook '{"session_id":"sess-nc"}' EXO_BIN="$STUB_OK" EXO_INDEX="$TMP/index-vacio.db"
if jq -e 'select(.reflex=="recall-fallback" and (.payload|test("reason=no-config")))' "$LOGC" >/dev/null 2>&1; then
  pass "no-config: EXO_BIN sin subcomando config ⇒ recall-fallback reason=no-config"
else
  fail "no-config: EXO_BIN sin subcomando config ⇒ recall-fallback reason=no-config" \
    "log=$(cat "$LOGC")"
fi

# ------------------- camino feliz: bloque real, sin ningún fallback -------
STUB_FELIZ="$TMP/exo-stub-feliz"
cat > "$STUB_FELIZ" <<'EOF'
#!/usr/bin/env bash
case "$1" in
  config) echo '{"schema_version":2,"command":"config","data":{"kb":{"name":"kb-test","path":"/tmp/kb-test"}}}' ;;
  recall) echo "Contrato de memoria: bloque de prueba camino feliz." ;;
  *) exit 1 ;;
esac
EOF
chmod +x "$STUB_FELIZ"

: > "$LOGC"
touch "$TMP/index-feliz.db"
run_hook '{"session_id":"sess-ok"}' EXO_BIN="$STUB_FELIZ" EXO_INDEX="$TMP/index-feliz.db"
CTX_OK="$(printf '%s' "$HOOK_OUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"
if contains "$CTX_OK" "Contrato de memoria" \
   && ! contains "$CTX_OK" "Tu memoria persistente es una KB" \
   && [ ! -s "$LOGC" ]; then
  pass "camino feliz: bloque real inyectado, sin fallback ni log de degradación"
else
  fail "camino feliz: bloque real inyectado, sin fallback ni log de degradación" \
    "ctx='$CTX_OK' log=$(cat "$LOGC" 2>/dev/null)"
fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
