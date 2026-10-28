#!/usr/bin/env bash
# Test standalone para delegar-decision.sh (Stop) y el modo CLI de delegar-decision.py.
# HOME y REFLEX_LOG_FILE en mktemp -d: nunca toca el sentinel ni el log reales.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/delegar-decision.sh"
PY="${SCRIPT_DIR}/delegar-decision.py"
DATA="${SCRIPT_DIR}/testdata/delegar-decision"
BASH_BIN="$(command -v bash)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
export HOME="$TMP/home"
mkdir -p "$HOME/.claude"
export REFLEX_LOG_FILE="$HOME/.claude/reflex-log.jsonl"
SENT="$HOME/.cache/exo/delegar-decision"

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }
ok_if() {  # $1 = nombre, $2 = condición (0 = ok), $3 = detalle
  if [ "$2" -eq 0 ]; then pass "$1"; else fail "$1" "$3"; fi
}

REASON='Cerraste con propuesta + petición de visto bueno sin delegar. Lanza ahora el consultor (skills/brainstorm/consultor-brief.md) o el ejecutor y cierra con su resultado; o, si la pregunta es de Paul, reescribe el cierre empezando por [reserva: externo|irreversible|re-scope|hecho-de-Paul|pedido].'

hook() {  # $1 = transcript, $2 = session_id, $3 = extra (",k:v")
  OUT="$(printf '{"session_id":"%s","transcript_path":"%s"%s}' "${2:-s1}" "$1" "${3:-}" | "$HOOK" 2>/dev/null)"
  RC=$?
}
decision() { printf '%s' "$OUT" | jq -r '.decision // empty' 2>/dev/null; }
reset() { rm -rf "$HOME/.cache" "$REFLEX_LOG_FILE"; }
# mk <fichero> <uuid humano> <texto del cierre>: un turno humano + un cierre de asistente.
mk() {
  jq -nc --arg u "$2" '{type:"user",uuid:$u,timestamp:"2026-10-10T10:00:00Z",message:{role:"user",content:"haz la cosa"}}' > "$1"
  jq -nc --arg t "$3" '{type:"assistant",uuid:"a",timestamp:"2026-10-10T10:01:00Z",message:{role:"assistant",content:[{type:"text",text:$t}]}}' >> "$1"
}
# mk_bloques <fichero> <uuid humano> <bloques JSON>
mk_bloques() {
  jq -nc --arg u "$2" '{type:"user",uuid:$u,timestamp:"2026-10-10T10:00:00Z",message:{role:"user",content:"haz la cosa"}}' > "$1"
  jq -nc --argjson b "$3" '{type:"assistant",uuid:"a",timestamp:"2026-10-10T10:01:00Z",message:{role:"assistant",content:$b}}' >> "$1"
}

reset
hook "$DATA/p2.jsonl"
if [ "$(decision)" = "block" ]; then pass "bloquea: p2 sin agent"; else fail "bloquea: p2 sin agent" "out=$OUT rc=$RC"; fi
R="$(printf '%s' "$OUT" | jq -r '.reason' 2>/dev/null)"
if [ "$R" = "$REASON" ]; then pass "reason: idéntico al literal"; else fail "reason: idéntico al literal" "reason=$R"; fi
ok_if "bloquea: sale 0" "$RC" "rc=$RC"

reset
hook "$DATA/agent.jsonl"
if [ -z "$OUT" ] && [ "$RC" -eq 0 ]; then pass "calla: agent en el segmento"; else fail "calla: agent en el segmento" "out=$OUT rc=$RC"; fi
hook "$DATA/reserva.jsonl"
if [ -z "$OUT" ] && [ "$RC" -eq 0 ]; then pass "calla: reserva al inicio"; else fail "calla: reserva al inicio" "out=$OUT rc=$RC"; fi
mk "$TMP/t-excl.jsonl" hx "Mi recomendación: seguir. ¿Hago push?"
hook "$TMP/t-excl.jsonl"
if [ -z "$OUT" ] && [ "$RC" -eq 0 ]; then pass "calla: término de reserva en la pregunta"; else fail "calla: término de reserva en la pregunta" "out=$OUT rc=$RC"; fi

# --- abstenciones ---
reset
hook "$DATA/p2.jsonl" s1 ',"agent_id":"a1"'
if [ -z "$OUT" ] && [ "$RC" -eq 0 ]; then pass "calla: subagente (agent_id)"; else fail "calla: subagente" "out=$OUT rc=$RC"; fi
hook "$DATA/p2.jsonl" s1 ',"stop_hook_active":true'
if [ -z "$OUT" ] && [ "$RC" -eq 0 ]; then pass "calla: stop_hook_active"; else fail "calla: stop_hook_active" "out=$OUT rc=$RC"; fi
hook "$TMP/no-existe.jsonl"
if [ -z "$OUT" ] && [ "$RC" -eq 0 ]; then pass "calla: transcript inexistente"; else fail "calla: transcript inexistente" "out=$OUT rc=$RC"; fi
printf '{"session_id":"s1"}' | "$HOOK" >/dev/null 2>&1
ok_if "calla: sin transcript_path (exit 0)" "$?" "rc!=0"

BIN="$TMP/bin"; mkdir -p "$BIN"
for c in cat dirname mkdir tr date head; do ln -s "$(command -v $c)" "$BIN/$c"; done
sin_path() {  # $1 = herramienta a incluir además de las básicas
  rm -f "$BIN/jq" "$BIN/python3"
  for t in "$@"; do [ -n "$t" ] && ln -s "$(command -v "$t")" "$BIN/$t"; done
  OUT="$(printf '{"session_id":"s1","transcript_path":"%s"}' "$DATA/p2.jsonl" | PATH="$BIN" "$BASH_BIN" "$HOOK" 2>/dev/null)"
  RC=$?
}
sin_path jq python3
if [ "$(decision)" = "block" ]; then pass "PATH recortado con jq y python3: control positivo, bloquea"; else fail "PATH recortado: control positivo" "out=$OUT rc=$RC"; fi
sin_path jq
if [ -z "$OUT" ] && [ "$RC" -eq 0 ]; then pass "calla: PATH sin python3"; else fail "calla: PATH sin python3" "out=$OUT rc=$RC"; fi
sin_path python3
if [ -z "$OUT" ] && [ "$RC" -eq 0 ]; then pass "calla: PATH sin jq"; else fail "calla: PATH sin jq" "out=$OUT rc=$RC"; fi
sin_path ""
if [ -z "$OUT" ] && [ "$RC" -eq 0 ]; then pass "calla: PATH sin python3 ni jq"; else fail "calla: PATH sin python3 ni jq" "out=$OUT rc=$RC"; fi

# --- tope por mensaje humano ---
reset
hook "$DATA/p2.jsonl" s1
PRIMERO="$(decision)"
hook "$DATA/p2.jsonl" s1
if [ "$PRIMERO" = "block" ] && [ -z "$OUT" ] && [ "$RC" -eq 0 ]; then pass "tope: segunda invocación calla"; else fail "tope: segunda invocación calla" "primero=$PRIMERO out=$OUT rc=$RC"; fi
if [ "$(cat "$SENT/s1" 2>/dev/null)" = "h1" ]; then pass "tope: sentinel = uuid del último humano"; else fail "tope: sentinel = uuid del último humano" "sentinel=$(cat "$SENT/s1" 2>/dev/null)"; fi
hook "$DATA/p2.jsonl" otra
if [ "$(decision)" = "block" ]; then pass "tope: otra sesión no comparte sentinel"; else fail "tope: otra sesión no comparte sentinel" "out=$OUT"; fi

cp "$DATA/p2.jsonl" "$TMP/t2.jsonl"
jq -nc '{type:"user",uuid:"h2",timestamp:"2026-10-10T10:02:00Z",message:{role:"user",content:"otra cosa"}}' >> "$TMP/t2.jsonl"
jq -nc '{type:"assistant",uuid:"a2",timestamp:"2026-10-10T10:03:00Z",message:{role:"assistant",content:[{type:"text",text:"Te propongo cambiar el nombre. ¿Lo hago?"}]}}' >> "$TMP/t2.jsonl"
hook "$TMP/t2.jsonl" s1
if [ "$(decision)" = "block" ]; then pass "tope: mensaje humano nuevo reabre el bloqueo"; else fail "tope: mensaje humano nuevo reabre el bloqueo" "out=$OUT"; fi

# Un hand-back posterior (no humano) no cambia el uuid del sentinel: no reabre el tope.
reset
hook "$DATA/p2.jsonl" s1
cp "$DATA/p2.jsonl" "$TMP/t3.jsonl"
jq -nc '{type:"user",uuid:"n1",timestamp:"2026-10-10T10:02:00Z",isMeta:true,promptSource:"system",message:{role:"user",content:"<task-notification>fin</task-notification>"}}' >> "$TMP/t3.jsonl"
jq -nc '{type:"assistant",uuid:"a3",timestamp:"2026-10-10T10:03:00Z",message:{role:"assistant",content:[{type:"text",text:"Mi recomendación: seguir. ¿Te parece?"}]}}' >> "$TMP/t3.jsonl"
hook "$TMP/t3.jsonl" s1
if [ -z "$OUT" ] && [ "$RC" -eq 0 ]; then pass "tope: un hand-back posterior no reabre el tope"; else fail "tope: un hand-back posterior no reabre el tope" "out=$OUT rc=$RC"; fi

# Sin ningún mensaje humano no hay uuid con el que topar: bloquea y no escribe sentinel (stop_hook_active evita el bucle).
reset
jq -nc '{type:"assistant",uuid:"a",timestamp:"2026-10-10T10:01:00Z",message:{role:"assistant",content:[{type:"text",text:"Mi recomendación: seguir. ¿Te parece?"}]}}' > "$TMP/t-sh.jsonl"
hook "$TMP/t-sh.jsonl" s1
D1="$(decision)"
hook "$TMP/t-sh.jsonl" s1
if [ "$D1" = "block" ] && [ "$(decision)" = "block" ] && [ ! -e "$SENT/s1" ]; then pass "uuid vacío: bloquea y no escribe sentinel"
else fail "uuid vacío: bloquea y no escribe sentinel" "d1=$D1 out=$OUT sent=$(find "$SENT" 2>&1 | tr '\n' ' ')"; fi

# --- session_id saneado ---
reset
hook "$DATA/p2.jsonl" "../x"
if [ "$(decision)" = "block" ] && [ ! -e "$HOME/.cache/exo/x" ] && [ ! -e "$HOME/.cache/x" ] && [ -f "$SENT/___x" ]; then pass "session_id=../x: se sanea, sin path traversal"
else fail "session_id=../x: se sanea, sin path traversal" "out=$OUT ls=$(find "$HOME/.cache" 2>&1 | tr '\n' ' ')"; fi
reset
hook "$DATA/p2.jsonl" ""
if [ "$(decision)" = "block" ] && [ "$RC" -eq 0 ]; then pass "session_id vacío: bloquea sin romper"; else fail "session_id vacío: bloquea sin romper" "out=$OUT rc=$RC"; fi

# --- sentinel no escribible (R7: el tope es cortesía, no garantía) ---
reset
mkdir -p "$HOME/.cache"; : > "$HOME/.cache/exo"
hook "$DATA/p2.jsonl"
D1="$(decision)"; RC1=$RC
hook "$DATA/p2.jsonl"
D2="$(decision)"
if [ "$D1" = "block" ] && [ "$RC1" -eq 0 ] && [ "$RC" -eq 0 ]; then pass "sentinel no escribible: bloquea y sale 0"; else fail "sentinel no escribible: bloquea y sale 0" "d1=$D1 rc1=$RC1 rc=$RC"; fi
if [ "$D2" = "block" ]; then pass "sentinel no escribible: bloquea de nuevo (best-effort, R7)"; else fail "sentinel no escribible: bloquea de nuevo (R7)" "d2=$D2"; fi
rm -f "$HOME/.cache/exo"

# --- log ---
reset
hook "$DATA/p2.jsonl"
L="$(jq -c 'select(.reflex=="delegar-decision")' "$REFLEX_LOG_FILE" 2>/dev/null)"
P="$(printf '%s' "$L" | jq -r '.payload' 2>/dev/null)"
case "$P" in *"kind=disparo"*"patron=P2"*"reserva_fuera=0"*"tail=Mi recomendación: aplicar el cambio en el módulo. ¿Te parece?"*) pass "log: disparo con patron=P2, reserva_fuera=0 y tail real" ;;
  *) fail "log: disparo con patron=P2, reserva_fuera=0 y tail real" "payload=$P" ;; esac
hook "$DATA/p2.jsonl"
N="$(jq -c 'select(.reflex=="delegar-decision")' "$REFLEX_LOG_FILE" | wc -l)"
if [ "$N" -eq 1 ]; then pass "log: el disparo suprimido por el tope no se loguea"; else fail "log: el disparo suprimido por el tope no se loguea" "lineas=$N"; fi

reset
mk "$TMP/t-rf.jsonl" hr "Mi recomendación: seguir. ¿Te parece? Antes hice push de la rama."
hook "$TMP/t-rf.jsonl"
P="$(jq -r 'select(.reflex=="delegar-decision") | .payload' "$REFLEX_LOG_FILE" 2>/dev/null)"
case "$P" in *"reserva_fuera=1"*) pass "log: reserva_fuera=1 cuando el término está fuera de la pregunta" ;;
  *) fail "log: reserva_fuera=1 cuando el término está fuera de la pregunta" "payload=$P" ;; esac

reset
hook "$DATA/banda.jsonl"
if [ -z "$OUT" ] && jq -e 'select(.reflex=="delegar-decision:banda") | select(.payload|test("kind=banda"))' "$REFLEX_LOG_FILE" >/dev/null 2>&1; then pass "log: banda → delegar-decision:banda"
else fail "log: banda → delegar-decision:banda" "out=$OUT log=$(cat "$REFLEX_LOG_FILE" 2>/dev/null)"; fi

reset
mk "$TMP/t-p1.jsonl" hp "Hay dos opciones, A o B. ¿Cuál prefieres?"
hook "$TMP/t-p1.jsonl"
if [ -z "$OUT" ] && jq -e 'select(.reflex=="delegar-decision") | select(.payload|test("kind=p1_log"))' "$REFLEX_LOG_FILE" >/dev/null 2>&1; then pass "log: p1_log → delegar-decision"
else fail "log: p1_log → delegar-decision" "out=$OUT log=$(cat "$REFLEX_LOG_FILE" 2>/dev/null)"; fi

reset
export REFLEX_LOG_FILE="$TMP/no-existe/dir/log.jsonl"
hook "$DATA/p2.jsonl"
D1="$(decision)"; RC1=$RC
hook "$DATA/banda.jsonl"
if [ "$D1" = "block" ] && [ "$RC1" -eq 0 ] && [ "$RC" -eq 0 ]; then pass "log: REFLEX_LOG_FILE inválido no rompe el hook"; else fail "log: REFLEX_LOG_FILE inválido no rompe el hook" "d1=$D1 rc1=$RC1 rc=$RC"; fi
export REFLEX_LOG_FILE="$HOME/.claude/reflex-log.jsonl"

# --- R16: cualquier excepción del detector ⇒ abstención ---
reset
mk_bloques "$TMP/t-exc.jsonl" he '[{"type":"text","text":5}]'
hook "$TMP/t-exc.jsonl"
if [ -z "$OUT" ] && [ "$RC" -eq 0 ]; then pass "excepción (text no-str): abstención exit 0"; else fail "excepción (text no-str): abstención exit 0" "out=$OUT rc=$RC"; fi
if jq -e 'select(.reflex=="delegar-decision") | select(.payload|test("kind=error"))' "$REFLEX_LOG_FILE" >/dev/null 2>&1; then pass "excepción: se registra kind=error"
else fail "excepción: se registra kind=error" "log=$(cat "$REFLEX_LOG_FILE" 2>/dev/null)"; fi
mk "$TMP/t-ok.jsonl" ht "¿Te parece?"
{ jq -nc '{type:"user",uuid:"hz",origin:"cadena",message:{role:"user",content:"x"}}'; cat "$TMP/t-ok.jsonl"; } > "$TMP/t-ts.jsonl"
hook "$TMP/t-ts.jsonl"
if [ -z "$OUT" ] && [ "$RC" -eq 0 ]; then pass "excepción (origin no-str): abstención exit 0"; else fail "excepción (origin no-str)" "out=$OUT rc=$RC"; fi

# --- CLI ---
cli() { CLI_OUT="$(python3 -I "$PY" "$1" 2>/dev/null)"; CLI_RC=$?; }
cli "$DATA/p2.jsonl"
J="$(printf '%s' "$CLI_OUT" | jq -c '[.uuid,.dispara,.kind,.patron,.reserva_fuera]' 2>/dev/null)"
if [ "$J" = '["h1",true,"disparo","P2",false]' ] || [ "$J" = '["h1",true,"disparo","P2",0]' ]; then pass "cli: p2 → uuid, dispara, kind, patron, reserva_fuera"
else fail "cli: p2 → uuid, dispara, kind, patron, reserva_fuera" "json=$CLI_OUT"; fi
TL="$(printf '%s' "$CLI_OUT" | jq -r '.tail')"
if [ "$TL" = "Mi recomendación: aplicar el cambio en el módulo. ¿Te parece?" ]; then pass "cli: tail real"; else fail "cli: tail real" "tail=$TL"; fi
cli "$TMP/no-existe.jsonl"
if [ "$CLI_RC" -eq 2 ]; then pass "cli: transcript ilegible → exit 2"; else fail "cli: transcript ilegible → exit 2" "rc=$CLI_RC"; fi
mk "$TMP/t-p3.jsonl" hq "Mi recomendación: dejarlo así, ¿no?"
cli "$TMP/t-p3.jsonl"
J="$(printf '%s' "$CLI_OUT" | jq -c '[.dispara,.patron]')"
if [ "$J" = '[true,"P3"]' ]; then pass "cli: patron=P3"; else fail "cli: patron=P3" "json=$CLI_OUT"; fi
cli "$TMP/t-p1.jsonl"
J="$(printf '%s' "$CLI_OUT" | jq -c '[.dispara,.kind,.patron]')"
if [ "$J" = '[false,"p1_log",null]' ]; then pass "cli: p1_log no dispara"; else fail "cli: p1_log no dispara" "json=$CLI_OUT"; fi
cli "$TMP/t-exc.jsonl"
if [ "$CLI_RC" -eq 0 ] && [ "$(printf '%s' "$CLI_OUT" | jq -r '.kind')" = "error" ] && [ "$(printf '%s' "$CLI_OUT" | jq -r '.dispara')" = "false" ]; then pass "cli: excepción → kind=error, dispara=false, exit 0"
else fail "cli: excepción → kind=error" "rc=$CLI_RC out=$CLI_OUT"; fi

printf '\n%s passed, %s failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
