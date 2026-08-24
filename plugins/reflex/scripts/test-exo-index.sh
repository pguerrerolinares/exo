#!/usr/bin/env bash
# Test standalone para exo-index.sh (hook Stop de M6-01, hecho portable).
# Fixtures en mktemp -d; nunca toca el índice real, el log real ni el HOME real.
#
# Las ramas de detach se fuerzan con los seams EXO_INDEX_SETSID/EXO_INDEX_CMD
# (mismo patrón que EXO_BIN): en Linux setsid existe siempre y en Git Bash no
# existe nunca, así que sin seam cada máquina solo podría probar su mitad.
# La rama cmd se prueba dos veces: con un cmd falso que registra argumentos
# (contrato de invocación, corre en ambas plataformas) y, solo donde hay cmd
# de verdad (Windows), end-to-end contra el cmd real midiendo que el hook
# vuelve sin esperar al indexado.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/exo-index.sh"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# Aislamiento global: ni el log de reflex real, ni el HOME real (los defaults
# del hook derivan de HOME; si un seam se rompiera, mejor escribir en el TMP
# que en la instalación de verdad).
export REFLEX_LOG_FILE="$TMP/reflex-log.jsonl"
export HOME="$TMP/home"
mkdir -p "$HOME"

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }
contains() { case "$1" in *"$2"*) return 0 ;; *) return 1 ;; esac; }

# Invoca el hook con el stdin JSON del Stop hook y deja HOOK_RC / HOOK_OUT.
# NO devuelve por stdout a propósito (mismo motivo que en test-recall-inject:
# un subshell se comería el exit code). $@ = VAR=valor extra para ESTA llamada.
run_hook() {
  printf '{"session_id":"test-sess","hook_event_name":"Stop"}' \
    | env "$@" "$HOOK" > "$TMP/hook-out.txt" 2>/dev/null
  HOOK_RC=$?
  HOOK_OUT="$(cat "$TMP/hook-out.txt" 2>/dev/null)"
}

# Espera activa a que el log del index contenga $1: el detach es asíncrono por
# contrato, así que un assert inmediato sería una race, no un test.
espera_log() {  # $1=texto  $2=segundos máx (default 15)
  local i=0 max=$(( ${2:-15} * 10 ))
  while [ "$i" -lt "$max" ]; do
    grep -q "$1" "$INDEX_LOG" 2>/dev/null && return 0
    sleep 0.1
    i=$((i+1))
  done
  return 1
}

# --- Fixtures compartidas ---------------------------------------------------
# Índice falso + log falso del index (seams EXO_INDEX / EXO_INDEX_LOG).
FAKE_DB="$TMP/index.db"
: > "$FAKE_DB"
export EXO_INDEX="$FAKE_DB"
INDEX_LOG="$TMP/exo-index.log"
export EXO_INDEX_LOG="$INDEX_LOG"

# Binario exo falso: escribe marcadores de inicio/fin por stdout (que el hook
# debe redirigir al log) y duerme STUB_SLEEP segundos entre medias para poder
# medir que el hook NO espera.
FAKE_EXO="$TMP/exo-stub.sh"
cat > "$FAKE_EXO" <<'EOF'
#!/usr/bin/env bash
printf 'stub-exo-inicio %s\n' "$*"
sleep "${STUB_SLEEP:-0}"
printf 'stub-exo-fin\n'
EOF
chmod +x "$FAKE_EXO"
export EXO_BIN="$FAKE_EXO"

# setsid falso: registra la invocación, pela el -f y ejecuta el resto para que
# el pipeline llegue de verdad al binario (nohup incluido, que existe en ambas
# plataformas).
FAKE_SETSID="$TMP/setsid-fake"
cat > "$FAKE_SETSID" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$SETSID_CALLS"
[ "${1:-}" = "-f" ] && shift
exec "$@"
EOF
chmod +x "$FAKE_SETSID"
export SETSID_CALLS="$TMP/setsid-calls.txt"

# cmd falso: registra la invocación y emula `start //b` lanzando el comando
# final (el script del bash interior) en background.
FAKE_CMD="$TMP/cmd-fake"
cat > "$FAKE_CMD" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$CMD_CALLS"
bash -c "${@: -1}" >/dev/null 2>&1 &
exit 0
EOF
chmod +x "$FAKE_CMD"
export CMD_CALLS="$TMP/cmd-calls.txt"

NO_EXISTE="$TMP/no-existe"  # ni setsid, ni cmd, ni exo: un path que no está

# ------------------------------------------------ T1: rama con setsid -------
: > "$INDEX_LOG"; : > "$SETSID_CALLS"; : > "$REFLEX_LOG_FILE"
run_hook EXO_INDEX_SETSID="$FAKE_SETSID"
if [ "$HOOK_RC" -eq 0 ]; then pass "setsid: exit 0"
else fail "setsid: exit 0" "rc=$HOOK_RC"; fi
CALLS="$(cat "$SETSID_CALLS" 2>/dev/null)"
if contains "$CALLS" "-f nohup" && contains "$CALLS" "index --db"; then
  pass "setsid: invocado con -f nohup y los args del index"
else fail "setsid: invocado con -f nohup y los args del index" "calls='$CALLS'"; fi
if espera_log "stub-exo-fin" 5 && grep -q -- "--json" "$INDEX_LOG"; then
  pass "setsid: el indexado corre y su salida acaba en el log del index"
else fail "setsid: el indexado corre y su salida acaba en el log del index" "$(cat "$INDEX_LOG" 2>/dev/null)"; fi
if [ ! -s "$REFLEX_LOG_FILE" ]; then pass "setsid: sin eventos de fallback"
else fail "setsid: sin eventos de fallback" "$(cat "$REFLEX_LOG_FILE")"; fi

# --------------------------- T2: rama sin setsid → cmd (contrato, fake) -----
: > "$INDEX_LOG"; : > "$CMD_CALLS"; : > "$REFLEX_LOG_FILE"
run_hook EXO_INDEX_SETSID="$NO_EXISTE" EXO_INDEX_CMD="$FAKE_CMD"
if [ "$HOOK_RC" -eq 0 ]; then pass "cmd: exit 0"
else fail "cmd: exit 0" "rc=$HOOK_RC"; fi
CALLS="$(cat "$CMD_CALLS" 2>/dev/null)"
# msys solo convierte //c→/c al cruzar a un exe nativo; el fake es un script,
# así que acepta ambas grafías para que el assert valga en las dos plataformas.
if { contains "$CALLS" "//c start" || contains "$CALLS" "/c start"; } && contains "$CALLS" "-c"; then
  pass "cmd: invocado como start + bash -c"
else fail "cmd: invocado como start + bash -c" "calls='$CALLS'"; fi
if espera_log "stub-exo-fin" 10; then
  pass "cmd: el indexado corre vía env (EXO_BIN/EXO_INDEX/LOG) y loguea"
else fail "cmd: el indexado corre vía env y loguea" "$(cat "$INDEX_LOG" 2>/dev/null)"; fi
if [ ! -s "$REFLEX_LOG_FILE" ]; then pass "cmd: sin eventos de fallback"
else fail "cmd: sin eventos de fallback" "$(cat "$REFLEX_LOG_FILE")"; fi

# ------------------- T2b: cmd REAL end-to-end (solo donde existe: Windows) --
case "$(uname -s)" in
  MINGW*|MSYS*|CYGWIN*)
    : > "$INDEX_LOG"; : > "$REFLEX_LOG_FILE"
    INICIO=$SECONDS
    run_hook EXO_INDEX_SETSID="$NO_EXISTE" STUB_SLEEP=4
    ELAPSED=$((SECONDS - INICIO))
    if [ "$HOOK_RC" -eq 0 ] && [ "$ELAPSED" -lt 3 ]; then
      pass "cmd real: el hook vuelve sin esperar al indexado (${ELAPSED}s)"
    else fail "cmd real: el hook vuelve sin esperar al indexado" "rc=$HOOK_RC elapsed=${ELAPSED}s"; fi
    # El stub duerme 4s DESPUÉS de que el hook ya salió: si el fin llega al
    # log es que el proceso sobrevivió a la muerte del hook, que es todo el
    # punto del detach.
    if espera_log "stub-exo-fin" 15; then
      pass "cmd real: el indexado sobrevive al hook y termina"
    else fail "cmd real: el indexado sobrevive al hook y termina" "$(cat "$INDEX_LOG" 2>/dev/null)"; fi
    if [ ! -s "$REFLEX_LOG_FILE" ]; then pass "cmd real: sin eventos de fallback"
    else fail "cmd real: sin eventos de fallback" "$(cat "$REFLEX_LOG_FILE")"; fi
    ;;
  *)
    printf '[SKIP] cmd real end-to-end: no hay cmd.exe en esta plataforma\n'
    ;;
esac

# ------------------------------------------------ T3: exe ausente -----------
# Máquina sin exo instalado: no es una degradación. Exit 0, silencio total.
: > "$INDEX_LOG"; : > "$SETSID_CALLS"; : > "$CMD_CALLS"; : > "$REFLEX_LOG_FILE"
run_hook EXO_BIN="$NO_EXISTE" EXO_INDEX_SETSID="$FAKE_SETSID" EXO_INDEX_CMD="$FAKE_CMD"
if [ "$HOOK_RC" -eq 0 ] && [ -z "$HOOK_OUT" ]; then pass "exe ausente: exit 0 sin salida"
else fail "exe ausente: exit 0 sin salida" "rc=$HOOK_RC out='$HOOK_OUT'"; fi
if [ ! -s "$SETSID_CALLS" ] && [ ! -s "$CMD_CALLS" ] && [ ! -s "$INDEX_LOG" ]; then
  pass "exe ausente: no lanza nada"
else fail "exe ausente: no lanza nada" "setsid='$(cat "$SETSID_CALLS")' cmd='$(cat "$CMD_CALLS")'"; fi
if [ ! -s "$REFLEX_LOG_FILE" ]; then pass "exe ausente: sin evento (no-instalado ≠ degradación)"
else fail "exe ausente: sin evento" "$(cat "$REFLEX_LOG_FILE")"; fi

# ------------------------------- T4: ninguna vía de detach ⇒ evento, no cero -
# EL punto del arreglo: antes esto salía 0 en silencio y el índice no se
# refrescaba nunca en Windows sin que nadie se enterase.
: > "$INDEX_LOG"; : > "$REFLEX_LOG_FILE"
run_hook EXO_INDEX_SETSID="$NO_EXISTE" EXO_INDEX_CMD="$NO_EXISTE"
if [ "$HOOK_RC" -eq 0 ]; then pass "no-detach: exit 0 (nunca bloquea el cierre)"
else fail "no-detach: exit 0" "rc=$HOOK_RC"; fi
if grep -q 'index-fallback' "$REFLEX_LOG_FILE" 2>/dev/null && \
   grep -q 'reason=no-detach' "$REFLEX_LOG_FILE" 2>/dev/null; then
  pass "no-detach: deja evento greppable index-fallback/no-detach"
else fail "no-detach: deja evento greppable" "$(cat "$REFLEX_LOG_FILE" 2>/dev/null)"; fi

# ------------------------------- T5: cmd presente pero que revienta ---------
# Un cmd que falla tampoco puede tumbar el cierre, pero sí debe dejar rastro.
CMD_ROTO="$TMP/cmd-roto"
printf '#!/usr/bin/env bash\nexit 1\n' > "$CMD_ROTO"
chmod +x "$CMD_ROTO"
: > "$REFLEX_LOG_FILE"
run_hook EXO_INDEX_SETSID="$NO_EXISTE" EXO_INDEX_CMD="$CMD_ROTO"
if [ "$HOOK_RC" -eq 0 ]; then pass "cmd roto: exit 0"
else fail "cmd roto: exit 0" "rc=$HOOK_RC"; fi
if grep -q 'reason=detach-failed' "$REFLEX_LOG_FILE" 2>/dev/null; then
  pass "cmd roto: loguea detach-failed"
else fail "cmd roto: loguea detach-failed" "$(cat "$REFLEX_LOG_FILE" 2>/dev/null)"; fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
