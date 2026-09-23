#!/usr/bin/env bash
# Test standalone para search-first.sh (reflejo "search-first", PreToolUse
# matcher ^(Agent|Task|Edit|Write|NotebookEdit)$ -- ver hooks/hooks.json).
# Warn-only: exit 0 SIEMPRE. Avisa como mucho 1x/sesion, solo en el padre, y
# calla cuando no sabe (sin jq, sin transcript_path, transcript ilegible, o
# si el jq de deteccion falla).
#
# Los 10 casos siguen la numeracion de
# docs/superpowers/specs/2026-09-23-search-first-design.md Seccion 6, en el
# mismo orden, para que sean trazables uno a uno contra la spec. El caso 7
# se parte en 7a/7b (dos motivos distintos de skip).
#
# Nota sobre el caso 4 (ver docs/superpowers/plans/2026-09-23-search-first.md,
# seccion "Contradiccion encontrada..."): la spec Seccion 6 ilustra el falso
# negativo con `echo "exo search"` (con comillas); la regex EXACTA de la
# Seccion 4.2 no matchea esa forma (el caracter antes de "exo" es una
# comilla, fuera de la clase [;&|[:space:]]). Este test usa `echo exo
# search` (sin comillas), que SI matchea y demuestra el mismo punto.
#
# Fixtures en mktemp -d; SEARCH_FIRST_SENTINEL_DIR y REFLEX_LOG_FILE
# apuntan siempre a ese directorio -- ningun caso toca /tmp real ni
# ~/.claude/reflex-log.jsonl.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/search-first.sh"

TMP="$(mktemp -d)"
mkdir -p "$TMP/sentinels"
LOG="$TMP/log.jsonl"
: > "$LOG"   # existe desde ya: sin esto, el primer "wc -l < $LOG" de
             # lineas_log() (antes de que corre() escriba nada) falla la
             # apertura del redirect y ensucia stderr con "No such file or
             # directory" (verificado corriendo la suite real antes de
             # este fix)
trap 'rm -rf "$TMP"' EXIT

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

# corre <session> <transcript_path|""> [agent_id] -> stdout del hook
corre() {
  local sesion="$1" transcript="$2" agente="${3:-}"
  jq -nc --arg s "sid-$sesion" --arg t "$transcript" --arg a "$agente" \
    '{session_id:$s, tool_name:"Edit", tool_input:{}, hook_event_name:"PreToolUse"}
     + (if $t == "" then {} else {transcript_path:$t} end)
     + (if $a == "" then {} else {agent_id:$a} end)' \
    | REFLEX_LOG_FILE="$LOG" SEARCH_FIRST_SENTINEL_DIR="$TMP/sentinels" "$HOOK"
}

avisa() { printf '%s' "$1" | jq -e '.hookSpecificOutput.additionalContext | length > 0' >/dev/null 2>&1; }
lineas_log() { wc -l < "$LOG" 2>/dev/null || echo 0; }
ultima_reflex() { tail -1 "$LOG" 2>/dev/null | jq -r '.reflex // empty' 2>/dev/null; }

linea_tool_use_bash() {  # <command>
  jq -nc --arg cmd "$1" '{type:"assistant", message:{content:[{type:"tool_use", id:"t1", name:"Bash", input:{command:$cmd}}]}}'
}
linea_texto_asistente() {  # <texto>
  jq -nc --arg txt "$1" '{type:"assistant", message:{content:[{type:"text", text:$txt}]}}'
}
linea_texto_usuario() {  # <texto>
  jq -nc --arg txt "$1" '{type:"user", message:{content:[{type:"text", text:$txt}]}}'
}

# --- Caso 1: tool_use Bash con `exo search --type hybrid "x"` -> ok --------
T1="$TMP/t1.jsonl"
linea_tool_use_bash 'exo search --type hybrid "x"' > "$T1"
ANTES=$(lineas_log)
OUT1="$(corre c1 "$T1")"
if [ -z "$OUT1" ] && [ "$(lineas_log)" -gt "$ANTES" ] && [ "$(ultima_reflex)" = "search-first-ok" ]; then
  pass "caso1: tool_use Bash con exo search --type hybrid -> search-first-ok, stdout vacío"
else
  fail "caso1: tool_use Bash con exo search --type hybrid -> search-first-ok, stdout vacío" "out=$OUT1 reflex=$(ultima_reflex)"
fi

# --- Caso 2 (trampa): solo el pie del recall como TEXTO, sin tool_use -> aviso
T2="$TMP/t2.jsonl"
linea_texto_asistente 'Voy a mirar la KB. (ignóralo si no aplica) Es top-3 de UNA query: para más, exo search --type hybrid "<q>"' > "$T2"
OUT2="$(corre c2 "$T2")"
if avisa "$OUT2" && [ "$(ultima_reflex)" = "search-first" ]; then
  pass "caso2 (trampa): pie del recall como texto, sin tool_use -> aviso"
else
  fail "caso2 (trampa): pie del recall como texto, sin tool_use -> aviso" "out=$OUT2 reflex=$(ultima_reflex)"
fi

# --- Caso 3: `cd x && exo targets` dentro de un encadenado -> ok -----------
T3="$TMP/t3.jsonl"
linea_tool_use_bash 'cd x && exo targets' > "$T3"
OUT3="$(corre c3 "$T3")"
if [ -z "$OUT3" ] && [ "$(ultima_reflex)" = "search-first-ok" ]; then
  pass "caso3: cd x && exo targets (encadenado) -> ok"
else
  fail "caso3: cd x && exo targets (encadenado) -> ok" "out=$OUT3 reflex=$(ultima_reflex)"
fi

# --- Caso 4: `echo exo search` (falso negativo documentado, Seccion 4.3) -> ok
T4="$TMP/t4.jsonl"
linea_tool_use_bash 'echo exo search' > "$T4"
OUT4="$(corre c4 "$T4")"
if [ -z "$OUT4" ] && [ "$(ultima_reflex)" = "search-first-ok" ]; then
  pass "caso4: echo exo search (falso negativo aceptado) -> ok"
else
  fail "caso4: echo exo search (falso negativo aceptado) -> ok" "out=$OUT4 reflex=$(ultima_reflex)"
fi

# --- Caso 5: subagente (agent_id con valor) -> silencio, sin log, sin sentinel
ANTES5=$(lineas_log)
OUT5="$(corre c5 "$T1" agente-1)"
SENTINEL5="$TMP/sentinels/claude-search-first-sid-c5"
if [ -z "$OUT5" ] && [ "$(lineas_log)" -eq "$ANTES5" ] && [ ! -f "$SENTINEL5" ]; then
  pass "caso5: subagente (agent_id con valor) -> stdout vacío, sin log, sin sentinel"
else
  fail "caso5: subagente (agent_id con valor) -> stdout vacío, sin log, sin sentinel" \
    "out=$OUT5 lineas_antes=$ANTES5 lineas_despues=$(lineas_log) sentinel_existe=$([ -f "$SENTINEL5" ] && echo si || echo no)"
fi

# --- Caso 6: segunda llamada con el sentinel ya creado -> silencio, jq no se invoca
corre c6 "$T1" >/dev/null   # primera llamada (jq real): crea el sentinel

POISON_DIR="$(mktemp -d)"
MARCA="$TMP/marca-jq-c6"
cat > "$POISON_DIR/jq" <<EOF
#!/usr/bin/env bash
echo x >> "$MARCA"
exit 1
EOF
chmod +x "$POISON_DIR/jq"

PAYLOAD_C6="$(jq -nc --arg s "sid-c6" --arg t "$T1" '{session_id:$s, tool_name:"Edit", tool_input:{}, hook_event_name:"PreToolUse", transcript_path:$t}')"
OUT6B="$(printf '%s' "$PAYLOAD_C6" | REFLEX_LOG_FILE="$LOG" SEARCH_FIRST_SENTINEL_DIR="$TMP/sentinels" PATH="$POISON_DIR:$PATH" bash "$HOOK" 2>/dev/null)"
if [ -z "$OUT6B" ] && [ ! -f "$MARCA" ]; then
  pass "caso6: segunda llamada con sentinel -> silencio, jq no se invoca"
else
  fail "caso6: segunda llamada con sentinel -> silencio, jq no se invoca" \
    "out=$OUT6B marca_existe=$([ -f "$MARCA" ] && echo si || echo no)"
fi
rm -rf "$POISON_DIR"

# --- Caso 7: sin transcript_path, o apuntando a un fichero que no existe -> skip, sin aviso
OUT7A="$(corre c7a "")"
if [ -z "$OUT7A" ] && [ "$(ultima_reflex)" = "search-first-skip" ]; then
  pass "caso7a: sin transcript_path -> search-first-skip, sin aviso"
else
  fail "caso7a: sin transcript_path -> search-first-skip, sin aviso" "out=$OUT7A reflex=$(ultima_reflex)"
fi

OUT7B="$(corre c7b "$TMP/no-existe-$$.jsonl")"
if [ -z "$OUT7B" ] && [ "$(ultima_reflex)" = "search-first-skip" ]; then
  pass "caso7b: transcript_path a fichero inexistente -> search-first-skip, sin aviso"
else
  fail "caso7b: transcript_path a fichero inexistente -> search-first-skip, sin aviso" "out=$OUT7B reflex=$(ultima_reflex)"
fi

# --- Caso 8: sin jq en el PATH -> exit 0, search-first-skip -----------------
# No basta con un jq falso que falle (eso simula "jq roto", no "jq ausente"
# -- command -v lo seguiría encontrando). Se construye un PATH que excluye
# SOLO el directorio que contiene el jq real -- cat/date/touch/dirname/bash
# siguen resolviendo desde el resto del PATH, sin copiar binarios (evita
# problemas de DLLs en Git Bash/Windows).
path_sin_jq() {
  local jq_real jq_dir resultado="" d
  jq_real="$(command -v jq 2>/dev/null)" || { printf '%s' "$PATH"; return; }
  jq_dir="$(dirname "$jq_real")"
  local viejo_ifs="$IFS"
  IFS=':'
  for d in $PATH; do
    [ "$d" = "$jq_dir" ] && continue
    [ -e "$d/jq" ] && continue
    [ -e "$d/jq.exe" ] && continue
    resultado="${resultado:+$resultado:}$d"
  done
  IFS="$viejo_ifs"
  printf '%s' "$resultado"
}
PATH_SIN_JQ="$(path_sin_jq)"
PAYLOAD_C8="$(jq -nc --arg s "sid-c8" '{session_id:$s, tool_name:"Edit", tool_input:{}, hook_event_name:"PreToolUse"}')"
OUT8="$(printf '%s' "$PAYLOAD_C8" | REFLEX_LOG_FILE="$LOG" SEARCH_FIRST_SENTINEL_DIR="$TMP/sentinels" PATH="$PATH_SIN_JQ" bash "$HOOK" 2>/dev/null)"
EC8=$?
if [ -z "$OUT8" ] && [ "$EC8" -eq 0 ] && [ "$(ultima_reflex)" = "search-first-skip" ]; then
  pass "caso8: sin jq en el PATH -> exit 0, search-first-skip"
else
  fail "caso8: sin jq en el PATH -> exit 0, search-first-skip" "out=$OUT8 ec=$EC8 reflex=$(ultima_reflex)"
fi

# --- Caso 9: la salida del caso negativo es JSON válido ---------------------
T9="$TMP/t9.jsonl"
linea_tool_use_bash 'ls -la' > "$T9"
OUT9="$(corre c9 "$T9")"
if printf '%s' "$OUT9" | jq -e '.hookSpecificOutput.additionalContext | length > 0' >/dev/null 2>&1; then
  pass "caso9: salida del caso negativo es JSON válido con additionalContext no vacío"
else
  fail "caso9: salida del caso negativo es JSON válido con additionalContext no vacío" "out=$OUT9"
fi

# --- Caso 10: `exo search` en un mensaje del usuario (no tool_use) -> aviso -
T10="$TMP/t10.jsonl"
linea_texto_usuario 'voy a hacer exo search de esto en un rato' > "$T10"
OUT10="$(corre c10 "$T10")"
if avisa "$OUT10" && [ "$(ultima_reflex)" = "search-first" ]; then
  pass "caso10: exo search en mensaje de usuario (no tool_use) -> aviso"
else
  fail "caso10: exo search en mensaje de usuario (no tool_use) -> aviso" "out=$OUT10 reflex=$(ultima_reflex)"
fi

echo ""
TOTAL=$((PASS+FAIL))
echo "=== Resultado: ${PASS}/${TOTAL} pasaron ==="
[ $FAIL -eq 0 ] && exit 0 || exit 1
