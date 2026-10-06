#!/usr/bin/env bash
# Test standalone para document-remind.sh (Stop): recordatorio + testigo de
# entrega de reglas. HOME y sentinels en mktemp -d; nunca toca los reales.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/document-remind.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
export HOME="$TMP/home"
export REMIND_SENTINEL_DIR="$TMP/sentinels"
mkdir -p "$HOME/.claude/exo-rules" "$REMIND_SENTINEL_DIR"
export REFLEX_LOG_FILE="$HOME/.claude/reflex-log.jsonl"
RULES="$HOME/.claude/exo-rules"

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }
contains() { case "$1" in *"$2"*) return 0 ;; *) return 1 ;; esac; }

SMALL="$TMP/small.jsonl"; seq 1 10 > "$SMALL"
BIG="$TMP/big.jsonl"; seq 1 80 > "$BIG"

run() {  # $1 = sid, $2 = transcript
  OUT="$(printf '{"session_id":"%s","transcript_path":"%s"}' "$1" "$2" | "$HOOK" 2>/dev/null)"
}
msg() { printf '%s' "$OUT" | jq -r '.systemMessage // empty' 2>/dev/null; }
witness_reasons() { jq -r 'select(.reflex=="project-rules-witness") | .payload' "$REFLEX_LOG_FILE" 2>/dev/null; }

NOCARGO='⚠ el mod de reglas de proyecto no cargó (sin latido): esta sesión no lleva reglas en el system prompt'
REMIND='💾 ¿Cierras sesión? Usa /document para guardar decisiones y aprendizajes en la KB.'

# sin_latido_grita_una_vez
echo '{"status":"ok","n":3,"repo":"exo"}' > "$RULES/ss-a"
run a "$SMALL"
if [ "$(msg)" = "$NOCARGO" ] && contains "$(witness_reasons)" "reason=sin_latido"; then pass "sin_latido_grita_una_vez: primer Stop"
else fail "sin_latido_grita_una_vez: primer Stop" "out=$OUT log=$(witness_reasons)"; fi
run a "$SMALL"
if [ -z "$OUT" ]; then pass "sin_latido_grita_una_vez: segundo Stop calla"
else fail "sin_latido_grita_una_vez: segundo Stop calla" "out=$OUT"; fi

# ss con error_engine y hb ausente: grita igual
echo '{"status":"skip","reason":"error_engine","n":0,"repo":null}' > "$RULES/ss-ee"
run ee "$SMALL"
if [ "$(msg)" = "$NOCARGO" ]; then pass "sin_latido: ss skip error_engine sin hb grita"
else fail "sin_latido: ss skip error_engine sin hb grita" "out=$OUT"; fi

# no_entrego
echo '{"status":"ok","n":3,"repo":"exo"}' > "$RULES/ss-b"
echo '{"status":"error","n":0,"error":"x"}' > "$RULES/hb-b"
run b "$SMALL"
EXP='⚠ el mod de reglas de proyecto no entregó: SessionStart vio ok n=3, el latido dice error n=0'
if [ "$(msg)" = "$EXP" ] && contains "$(witness_reasons)" "reason=no_entrego"; then pass "no_entrego: copy exacto"
else fail "no_entrego: copy exacto" "out=$OUT"; fi

# n_distinto
echo '{"status":"ok","n":3,"repo":"exo"}' > "$RULES/ss-c"
echo '{"status":"ok","n":2}' > "$RULES/hb-c"
run c "$SMALL"
if contains "$(msg)" "no entregó" && contains "$(msg)" "el latido dice ok n=2"; then pass "n_distinto: grita no entregó"
else fail "n_distinto: grita no entregó" "out=$OUT"; fi

# hb corrupto
echo '{"status":"ok","n":3,"repo":"exo"}' > "$RULES/ss-d"
echo 'no es json{' > "$RULES/hb-d"
run d "$SMALL"
if contains "$(msg)" "no entregó"; then pass "hb corrupto: cuenta como no entregó"
else fail "hb corrupto: cuenta como no entregó" "out=$OUT"; fi

# ss skip con hb ok: no entregó
echo '{"status":"skip","reason":"sin_nota","n":0,"repo":"exo"}' > "$RULES/ss-e"
echo '{"status":"ok","n":2}' > "$RULES/hb-e"
run e "$SMALL"
if contains "$(msg)" "no entregó" && contains "$(msg)" "SessionStart vio skip n=0, el latido dice ok n=2"; then pass "ss skip + hb ok: grita no entregó"
else fail "ss skip + hb ok: grita no entregó" "out=$OUT"; fi

# coherente_calla
echo '{"status":"ok","n":3,"repo":"exo"}' > "$RULES/ss-f"; echo '{"status":"ok","n":3}' > "$RULES/hb-f"
run f "$SMALL"; r1="$OUT"
echo '{"status":"skip","reason":"sin_nota","n":0,"repo":"exo"}' > "$RULES/ss-g"; echo '{"status":"skip","reason":"sin_nota","n":0}' > "$RULES/hb-g"
run g "$SMALL"; r2="$OUT"
echo '{"status":"error","n":0}' > "$RULES/hb-h"; cp "$RULES/ss-g" "$RULES/ss-h"
run h "$SMALL"; r3="$OUT"
if [ -z "$r1$r2$r3" ]; then pass "coherente_calla: ok=ok, skip=skip, skip/error"
else fail "coherente_calla: ok=ok, skip=skip, skip/error" "r1=$r1 r2=$r2 r3=$r3"; fi

# sin_ss_solo_log
: > "$REFLEX_LOG_FILE"
run nss "$SMALL"
if [ -z "$OUT" ] && contains "$(witness_reasons)" "reason=sin_ss"; then pass "sin_ss_solo_log"
else fail "sin_ss_solo_log" "out=$OUT log=$(witness_reasons)"; fi

# testigo_antes_de_umbral: SMALL (10 líneas) ya lo cubre arriba; verificación explícita
echo '{"status":"ok","n":1,"repo":"exo"}' > "$RULES/ss-u"
run u "$SMALL"
if [ "$(msg)" = "$NOCARGO" ]; then pass "testigo_antes_de_umbral: 10 líneas gritan"
else fail "testigo_antes_de_umbral: 10 líneas gritan" "out=$OUT"; fi

# recordatorio_intacto
run rem "$BIG"
if [ "$OUT" = "{\"systemMessage\":\"$REMIND\"}" ]; then pass "recordatorio_intacto"
else fail "recordatorio_intacto" "out=$OUT"; fi

# ambos_unidos
echo '{"status":"ok","n":3,"repo":"exo"}' > "$RULES/ss-both"
run both "$BIG"
if [ "$(printf '%s\n' "$OUT" | wc -l)" -eq 1 ] && [ "$(msg)" = "$NOCARGO"$'\n'"$REMIND" ]; then pass "ambos_unidos: una línea, dos textos"
else fail "ambos_unidos: una línea, dos textos" "out=$OUT"; fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
