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
ultima_payload() { tail -1 "$LOG" 2>/dev/null | jq -r '.payload // empty' 2>/dev/null; }

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
# (review final: no basta con el reflex == skip, hay que distinguir el MOTIVO
# via el payload exacto de la ultima linea del log -- si no, un skip por la
# rama equivocada pasaria igual el test).
OUT7A="$(corre c7a "")"
if [ -z "$OUT7A" ] && [ "$(ultima_reflex)" = "search-first-skip" ] && [ "$(ultima_payload)" = "motivo=sin-transcript_path" ]; then
  pass "caso7a: sin transcript_path -> search-first-skip, sin aviso, payload motivo=sin-transcript_path"
else
  fail "caso7a: sin transcript_path -> search-first-skip, sin aviso, payload motivo=sin-transcript_path" \
    "out=$OUT7A reflex=$(ultima_reflex) payload=$(ultima_payload)"
fi

OUT7B="$(corre c7b "$TMP/no-existe-$$.jsonl")"
if [ -z "$OUT7B" ] && [ "$(ultima_reflex)" = "search-first-skip" ] && [ "$(ultima_payload)" = "motivo=transcript-ilegible" ]; then
  pass "caso7b: transcript_path a fichero inexistente -> search-first-skip, sin aviso, payload motivo=transcript-ilegible"
else
  fail "caso7b: transcript_path a fichero inexistente -> search-first-skip, sin aviso, payload motivo=transcript-ilegible" \
    "out=$OUT7B reflex=$(ultima_reflex) payload=$(ultima_payload)"
fi

# --- Caso 8: sin jq en el PATH -> exit 0, search-first-skip -----------------
# No basta con un jq falso que falle (eso simula "jq roto", no "jq ausente"
# -- command -v lo seguiría encontrando). Tampoco basta con filtrar del PATH
# el directorio que contiene jq: en Ubuntu 24.04 (usr-merge) ese directorio
# es /usr/bin, donde TAMBIEN viven bash/cat/date/touch/dirname -- excluirlo
# deja sin encontrar al propio `bash` del "PATH=... bash $HOOK" (una
# asignacion de variable delante de un comando simple usa el PATH YA
# modificado para buscar ese comando, no el PATH del shell que lo lanza:
# `PATH=/vacio ls` falla igual que fallaria aqui). Eso se vio en CI como
# ec=127 y el log mostrando el motivo del caso ANTERIOR (la invocacion ni
# llego a arrancar, no escribio nada).
#
# En vez de filtrar el PATH heredado, se construye un PATH minimo desde
# cero: un directorio en mktemp con wrappers SOLO para las herramientas que
# usa esta rama del hook (cat/dirname/date/touch -- ver search-first.sh:
# INPUT="$(cat)", log_skip_sin_jq() usa date+touch; dirname no se llega a
# invocar en esta rama pero se incluye igual por si el hook cambia). Cada
# wrapper tiene shebang a la ruta ABSOLUTA de bash y hace exec a la ruta
# ABSOLUTA de la herramienta real -- nada se copia (evita el problema de
# DLLs de copiar binarios en Git Bash/Windows). El hook se invoca con la
# ruta absoluta de bash (no "bash" a secas), asi que su propio arranque no
# depende en absoluto del PATH minimo.
BASH_ABS="$(command -v bash)"
SHIM="$(mktemp -d)"
for herramienta in cat dirname date touch; do
  real="$(command -v "$herramienta" 2>/dev/null)" || continue
  printf '#!%s\nexec %s "$@"\n' "$BASH_ABS" "$real" > "$SHIM/$herramienta"
  chmod +x "$SHIM/$herramienta"
done
PAYLOAD_C8="$(jq -nc --arg s "sid-c8" '{session_id:$s, tool_name:"Edit", tool_input:{}, hook_event_name:"PreToolUse"}')"
OUT8="$(printf '%s' "$PAYLOAD_C8" | REFLEX_LOG_FILE="$LOG" SEARCH_FIRST_SENTINEL_DIR="$TMP/sentinels" PATH="$SHIM" "$BASH_ABS" "$HOOK" 2>/dev/null)"
EC8=$?
rm -rf "$SHIM"
if [ -z "$OUT8" ] && [ "$EC8" -eq 0 ] && [ "$(ultima_reflex)" = "search-first-skip" ] && [ "$(ultima_payload)" = "motivo=sin-jq" ]; then
  pass "caso8: sin jq en el PATH -> exit 0, search-first-skip, payload motivo=sin-jq"
else
  fail "caso8: sin jq en el PATH -> exit 0, search-first-skip, payload motivo=sin-jq" \
    "out=$OUT8 ec=$EC8 reflex=$(ultima_reflex) payload=$(ultima_payload)"
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

# --- Casos 11+: añadidos en el review final (2026-09-23), fuera de la
# numeracion 1-10 de la spec Seccion 6 (esa sigue intacta arriba) --------

# --- Caso 11: `exo` invocado por ruta relativa (`./engine/target/release/exo.exe search ...`) -> ok
T11="$TMP/t11.jsonl"
linea_tool_use_bash './engine/target/release/exo.exe search "x"' > "$T11"
OUT11="$(corre c11 "$T11")"
if [ -z "$OUT11" ] && [ "$(ultima_reflex)" = "search-first-ok" ]; then
  pass "caso11: exo invocado por ruta relativa (./engine/.../exo.exe search) -> ok"
else
  fail "caso11: exo invocado por ruta relativa (./engine/.../exo.exe search) -> ok" "out=$OUT11 reflex=$(ultima_reflex)"
fi

# --- Caso 12: `exo` invocado por ruta con `~` (`~/.local/bin/exo search`) -> ok
T12="$TMP/t12.jsonl"
# shellcheck disable=SC2088 # literal deliberado: es el comando tal cual queda
# en el JSON de la transcripcion (input.command), no se ejecuta ni se espera
# que la tilde expanda aqui -- solo se casa contra la regex del hook.
linea_tool_use_bash '~/.local/bin/exo search "x"' > "$T12"
OUT12="$(corre c12 "$T12")"
if [ -z "$OUT12" ] && [ "$(ultima_reflex)" = "search-first-ok" ]; then
  pass "caso12: exo invocado por ruta con ~ (~/.local/bin/exo search) -> ok"
else
  fail "caso12: exo invocado por ruta con ~ (~/.local/bin/exo search) -> ok" "out=$OUT12 reflex=$(ultima_reflex)"
fi

# --- Caso 13: `notexo search` NO debe casar (el prefijo "exo" no está tras
# uno de los separadores de la clase) -> aviso, igual que sin búsqueda
T13="$TMP/t13.jsonl"
linea_tool_use_bash 'notexo search "x"' > "$T13"
OUT13="$(corre c13 "$T13")"
if avisa "$OUT13" && [ "$(ultima_reflex)" = "search-first" ]; then
  pass "caso13: notexo search (no debe casar) -> aviso"
else
  fail "caso13: notexo search (no debe casar) -> aviso" "out=$OUT13 reflex=$(ultima_reflex)"
fi

# --- Caso 14: JSON con espacios alrededor de ":" y "agent_id": "" (vacío,
# valor de tipo string pero SIN caracteres) -> se trata como PADRE: crea
# sentinel y evalúa la transcripción (no es "presente y no vacío")
T14="$TMP/t14.jsonl"
linea_tool_use_bash 'exo search --type hybrid "x"' > "$T14"
PAYLOAD14=$(printf '{"session_id" : "sid-c14", "agent_id" : "", "transcript_path" : "%s", "tool_name" : "Edit", "tool_input" : {}, "hook_event_name" : "PreToolUse"}' "$T14")
SENTINEL14="$TMP/sentinels/claude-search-first-sid-c14"
OUT14="$(printf '%s' "$PAYLOAD14" | REFLEX_LOG_FILE="$LOG" SEARCH_FIRST_SENTINEL_DIR="$TMP/sentinels" "$HOOK")"
if [ -z "$OUT14" ] && [ -f "$SENTINEL14" ] && [ "$(ultima_reflex)" = "search-first-ok" ]; then
  pass "caso14: JSON con espacios en \":\" y agent_id vacío -> tratado como padre (sentinel creado, evalúa)"
else
  fail "caso14: JSON con espacios en \":\" y agent_id vacío -> tratado como padre (sentinel creado, evalúa)" \
    "out=$OUT14 sentinel_existe=$([ -f "$SENTINEL14" ] && echo si || echo no) reflex=$(ultima_reflex)"
fi

# --- Caso 15: transcripción con una línea JSON truncada/corrupta seguida de
# un tool_use Bash positivo -> search-first-ok (el `fromjson?` descarta la
# línea rota sin abortar el resto del fichero)
T15="$TMP/t15.jsonl"
printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"tool_use","id":"trunc' > "$T15"
linea_tool_use_bash 'exo search --type hybrid "y"' >> "$T15"
OUT15="$(corre c15 "$T15")"
if [ -z "$OUT15" ] && [ "$(ultima_reflex)" = "search-first-ok" ]; then
  pass "caso15: línea JSON truncada + tool_use Bash positivo -> search-first-ok"
else
  fail "caso15: línea JSON truncada + tool_use Bash positivo -> search-first-ok" "out=$OUT15 reflex=$(ultima_reflex)"
fi

echo ""
TOTAL=$((PASS+FAIL))
echo "=== Resultado: ${PASS}/${TOTAL} pasaron ==="
[ $FAIL -eq 0 ] && exit 0 || exit 1
