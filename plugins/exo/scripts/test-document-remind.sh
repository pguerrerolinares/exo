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

# Stub de `exo rules`: registra llamadas y devuelve STUB_OUT con exit STUB_RC.
# Por defecto falla (sin_verdad): ningún test toca el engine real.
STUB="$TMP/exo-stub"
STUB_CALLS="$TMP/stub-calls"
cat > "$STUB" <<'STUBEOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$STUB_CALLS"
[ -n "${STUB_ERR:-}" ] && printf '%s' "$STUB_ERR" >&2
printf '%s' "${STUB_OUT:-}"
exit "${STUB_RC:-1}"
STUBEOF
chmod +x "$STUB"
export EXO_BIN="$STUB" STUB_CALLS
stub() {  # $1 = envelope JSON, $2 = rc (default 0)
  export STUB_OUT="$1" STUB_RC="${2:-0}"
}
stub_reset() { unset STUB_OUT STUB_ERR; export STUB_RC=1; : > "$STUB_CALLS"; }
ENV_OK1='{"data":{"status":"ok","repo":"exo","rules":["r1"]}}'
ENV_SKIP='{"data":{"status":"skip","reason":"sin_seccion","repo":"exo","rules":[]}}'
stub_reset

SMALL="$TMP/small.jsonl"; seq 1 10 > "$SMALL"
BIG="$TMP/big.jsonl"; seq 1 80 > "$BIG"

run() {  # $1 = sid, $2 = transcript, $3 = cwd (opcional)
  OUT="$(printf '{"session_id":"%s","transcript_path":"%s","cwd":"%s"}' "$1" "$2" "${3:-$TMP}" | "$HOOK" 2>/dev/null)"
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
stub '{"data":{"status":"ok","rules":["a","b","c"]}}'
run c "$SMALL"
stub_reset
if contains "$(msg)" "no entregó" && contains "$(msg)" "el engine dice ok n=3, el latido dice ok n=2"; then pass "n_distinto: engine adjudica, grita no entregó"
else fail "n_distinto: engine adjudica, grita no entregó" "out=$OUT"; fi

# hb corrupto
echo '{"status":"ok","n":3,"repo":"exo"}' > "$RULES/ss-d"
echo 'no es json{' > "$RULES/hb-d"
run d "$SMALL"
if contains "$(msg)" "no entregó"; then pass "hb corrupto: cuenta como no entregó"
else fail "hb corrupto: cuenta como no entregó" "out=$OUT"; fi

# ss skip con hb ok: no entregó
echo '{"status":"skip","reason":"sin_nota","n":0,"repo":"exo"}' > "$RULES/ss-e"
echo '{"status":"ok","n":2}' > "$RULES/hb-e"
stub '{"data":{"status":"ok","rules":["a"]}}'
run e "$SMALL"
stub_reset
if contains "$(msg)" "no entregó" && contains "$(msg)" "el engine dice ok n=1, el latido dice ok n=2"; then pass "ss skip + hb ok n distinto de la verdad: grita no entregó"
else fail "ss skip + hb ok n distinto de la verdad: grita no entregó" "out=$OUT"; fi

# coherente_calla
echo '{"status":"ok","n":3,"repo":"exo"}' > "$RULES/ss-f"; echo '{"status":"ok","n":3}' > "$RULES/hb-f"
run f "$SMALL"; r1="$OUT"
echo '{"status":"skip","reason":"sin_nota","n":0,"repo":"exo"}' > "$RULES/ss-g"; echo '{"status":"skip","reason":"sin_nota","n":0}' > "$RULES/hb-g"
run g "$SMALL"; r2="$OUT"
echo '{"status":"error","n":0}' > "$RULES/hb-h"; cp "$RULES/ss-g" "$RULES/ss-h"
run h "$SMALL"; r3="$OUT"
if [ -z "$r1$r2$r3" ]; then pass "coherente_calla: ok=ok, skip=skip, skip/error"
else fail "coherente_calla: ok=ok, skip=skip, skip/error" "r1=$r1 r2=$r2 r3=$r3"; fi

# M1: hb vacío (jq sin salida) cuenta como corrupto
echo '{"status":"skip","reason":"sin_nota","n":0,"repo":"exo"}' > "$RULES/ss-hv"
: > "$RULES/hb-hv"
run hv "$SMALL"
if contains "$(msg)" "no entregó"; then pass "hb vacío: cuenta como corrupto"
else fail "hb vacío: cuenta como corrupto" "out=$OUT"; fi

# M2: ss skip + hb error: callado en pantalla, pero con rastro en el log
: > "$REFLEX_LOG_FILE"
echo '{"status":"skip","reason":"sin_nota","n":0,"repo":"exo"}' > "$RULES/ss-he"
echo '{"status":"error","n":0,"error":"x"}' > "$RULES/hb-he"
run he "$SMALL"
if [ -z "$OUT" ] && contains "$(witness_reasons)" "reason=hb_error"; then pass "ss skip + hb error: calla en pantalla, log hb_error"
else fail "ss skip + hb error: calla en pantalla, log hb_error" "out=$OUT log=$(witness_reasons)"; fi

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

# --- fila 5: divergencia sin error, adjudica `exo rules` ---
ss_json() { printf '{"status":"%s","n":%s,"repo":"exo"}' "$2" "$3" > "$RULES/ss-$1"; }
hb_json() { printf '{"status":"%s","n":%s}' "$2" "$3" > "$RULES/hb-$1"; }

# kb_cambio_calla_y_refresca_ss (la regresión real: regla añadida a mitad de sesión)
: > "$REFLEX_LOG_FILE"; stub_reset
ss_json kc skip 0; hb_json kc ok 1; stub "$ENV_OK1"
run kc "$SMALL"
if [ -z "$OUT" ] && contains "$(witness_reasons)" "reason=kb_cambio" \
   && [ "$(jq -r '[.status,.n]|@tsv' "$RULES/ss-kc")" = "$(printf 'ok\t1')" ] && [ ! -f "$RULES/ss-kc.tmp" ]; then pass "kb_cambio_calla_y_refresca_ss"
else fail "kb_cambio_calla_y_refresca_ss" "out=$OUT log=$(witness_reasons) ss=$(cat "$RULES/ss-kc")"; fi
# sin sentinel: el siguiente Stop ya es la fila 4 y no llama al engine
: > "$STUB_CALLS"; run kc "$SMALL"
if [ -z "$OUT" ] && [ ! -s "$STUB_CALLS" ]; then pass "kb_cambio: siguiente Stop coherente sin spawn"
else fail "kb_cambio: siguiente Stop coherente sin spawn" "out=$OUT calls=$(cat "$STUB_CALLS")"; fi

# regla_quitada_calla
: > "$REFLEX_LOG_FILE"; stub_reset
ss_json rq ok 1; hb_json rq skip 0; stub "$ENV_SKIP"
run rq "$SMALL"
if [ -z "$OUT" ] && contains "$(witness_reasons)" "reason=kb_cambio"; then pass "regla_quitada_calla"
else fail "regla_quitada_calla" "out=$OUT log=$(witness_reasons)"; fi

# divergencia_real_grita (+ no sentinel en kb_cambio previo)
: > "$REFLEX_LOG_FILE"; stub_reset
ss_json dr ok 1; hb_json dr skip 0; stub "$ENV_OK1"
run dr "$SMALL"
EXP='⚠ el mod de reglas de proyecto no entregó: el engine dice ok n=1, el latido dice skip n=0'
if [ "$(msg)" = "$EXP" ] && contains "$(witness_reasons)" "reason=no_entrego"; then pass "divergencia_real_grita: copy exacto"
else fail "divergencia_real_grita: copy exacto" "out=$OUT log=$(witness_reasons)"; fi

# sin_verdad_calla (exit 1) y JSON inválido; no crea sentinel
: > "$REFLEX_LOG_FILE"; stub_reset
ss_json sv ok 1; hb_json sv skip 0; export STUB_RC=1 STUB_ERR="boom del engine"
run sv "$SMALL"
if [ -z "$OUT" ] && contains "$(witness_reasons)" "reason=sin_verdad" && contains "$(witness_reasons)" "boom del engine" \
   && [ ! -f "$REMIND_SENTINEL_DIR/claude-rules-witness-sv" ]; then pass "sin_verdad_calla: exit 1, stderr en log, sin sentinel"
else fail "sin_verdad_calla: exit 1, stderr en log, sin sentinel" "out=$OUT log=$(witness_reasons)"; fi
: > "$REFLEX_LOG_FILE"; stub_reset
ss_json sv2 ok 1; hb_json sv2 skip 0; stub 'no json{' 0
run sv2 "$SMALL"
if [ -z "$OUT" ] && contains "$(witness_reasons)" "reason=sin_verdad"; then pass "sin_verdad_calla: JSON inválido"
else fail "sin_verdad_calla: JSON inválido" "out=$OUT log=$(witness_reasons)"; fi
stub_reset

# coherente_no_llama_al_engine (todas las filas que no son divergencia)
: > "$STUB_CALLS"
ss_json nc1 ok 3; hb_json nc1 ok 3; run nc1 "$SMALL"
ss_json nc2 skip 0; hb_json nc2 skip 0; run nc2 "$SMALL"
ss_json nc3 skip 0; hb_json nc3 error 0; run nc3 "$SMALL"
ss_json nc4 ok 3; hb_json nc4 error 0; run nc4 "$SMALL"
if [ ! -s "$STUB_CALLS" ]; then pass "coherente_no_llama_al_engine"
else fail "coherente_no_llama_al_engine" "calls=$(cat "$STUB_CALLS")"; fi

# cwd_del_input
stub_reset; ss_json cw ok 1; hb_json cw skip 0; stub "$ENV_OK1"
run cw "$SMALL" "/ruta/del/input"
if [ "$(cat "$STUB_CALLS")" = "rules --cwd /ruta/del/input --json" ]; then pass "cwd_del_input"
else fail "cwd_del_input" "calls=$(cat "$STUB_CALLS")"; fi
stub_reset

# hb_sin_status_es_corrupto: `{}` grita y no consulta al engine
stub_reset; : > "$STUB_CALLS"
ss_json hs ok 1; echo '{}' > "$RULES/hb-hs"; stub "$ENV_OK1"
run hs "$SMALL"
if contains "$(msg)" "no entregó" && [ ! -s "$STUB_CALLS" ]; then pass "hb_sin_status_es_corrupto"
else fail "hb_sin_status_es_corrupto" "out=$OUT calls=$(cat "$STUB_CALLS")"; fi
stub_reset

# recordatorio_intacto
run rem "$BIG"
if [ "$OUT" = "{\"systemMessage\":\"$REMIND\"}" ]; then pass "recordatorio_intacto"
else fail "recordatorio_intacto" "out=$OUT"; fi

# ambos_unidos
echo '{"status":"ok","n":3,"repo":"exo"}' > "$RULES/ss-both"
run both "$BIG"
if [ "$(printf '%s\n' "$OUT" | wc -l)" -eq 1 ] && [ "$(msg)" = "$NOCARGO"$'\n'"$REMIND" ]; then pass "ambos_unidos: una línea, dos textos"
else fail "ambos_unidos: una línea, dos textos" "out=$OUT"; fi

# independencia de sentinels (a): el testigo ya gritó; el recordatorio sale igual
echo '{"status":"ok","n":3,"repo":"exo"}' > "$RULES/ss-ind1"
run ind1 "$SMALL"
r1="$(msg)"
run ind1 "$BIG"
if [ "$r1" = "$NOCARGO" ] && [ "$(msg)" = "$REMIND" ]; then pass "sentinels independientes: testigo no silencia recordatorio"
else fail "sentinels independientes: testigo no silencia recordatorio" "r1=$r1 out=$OUT"; fi

# (b): el recordatorio ya salió; el testigo grita igual
echo '{"status":"ok","n":3,"repo":"exo"}' > "$RULES/ss-ind2"
touch "$REMIND_SENTINEL_DIR/claude-document-reminded-ind2"
run ind2 "$BIG"
if [ "$(msg)" = "$NOCARGO" ]; then pass "sentinels independientes: recordatorio no silencia testigo"
else fail "sentinels independientes: recordatorio no silencia testigo" "out=$OUT"; fi

# --- tres estados: via del latido (none / submit / legacy) ---
hbraw() { printf '%s' "$2" > "$RULES/hb-$1"; }
SINCANAL='⚠ el mod de reglas de proyecto cargó pero ningún canal entregó (via=none): SessionStart vio ok n=1; ¿política de la org nueva o exo rules falla?'
DEGRADADO='ℹ reglas de proyecto entregadas por canal degradado (política de la org): n=1'
HOY="$(date +%Y%m%d)"
rm_degradado() { rm -f "$REMIND_SENTINEL_DIR"/claude-rules-degradado-*; }

# via_none_con_ss_ok_grita
: > "$REFLEX_LOG_FILE"; stub_reset
ss_json vn ok 1; hbraw vn '{"status":"error","n":0,"via":"none","error":"sin entrega"}'
run vn "$SMALL"
if [ "$(msg)" = "$SINCANAL" ] && contains "$(witness_reasons)" "reason=sin_canal"; then pass "via_none_con_ss_ok_grita: copy exacto"
else fail "via_none_con_ss_ok_grita: copy exacto" "out=$OUT log=$(witness_reasons)"; fi
run vn "$SMALL"
if [ -z "$OUT" ]; then pass "via_none_con_ss_ok_grita: segundo Stop calla"
else fail "via_none_con_ss_ok_grita: segundo Stop calla" "out=$OUT"; fi

# via_none_con_ss_skip_calla
: > "$REFLEX_LOG_FILE"
echo '{"status":"skip","reason":"sin_nota","n":0,"repo":"exo"}' > "$RULES/ss-vs"
hbraw vs '{"status":"error","n":0,"via":"none","error":"sin entrega"}'
run vs "$SMALL"
if [ -z "$OUT" ] && contains "$(witness_reasons)" "reason=hb_error"; then pass "via_none_con_ss_skip_calla"
else fail "via_none_con_ss_skip_calla" "out=$OUT log=$(witness_reasons)"; fi

# via_submit_log_y_aviso_una_vez (sentinel diario compartido entre sesiones)
rm_degradado; : > "$REFLEX_LOG_FILE"
ss_json sb1 ok 1; hbraw sb1 '{"status":"ok","n":1,"via":"submit"}'
ss_json sb2 ok 1; hbraw sb2 '{"status":"ok","n":1,"via":"submit"}'
run sb1 "$SMALL"
if [ "$(msg)" = "$DEGRADADO" ] && contains "$(witness_reasons)" "reason=entrega_degradada"; then pass "via_submit: primer aviso copy exacto"
else fail "via_submit: primer aviso copy exacto" "out=$OUT log=$(witness_reasons)"; fi
: > "$REFLEX_LOG_FILE"
run sb2 "$SMALL"
if [ -z "$OUT" ] && contains "$(witness_reasons)" "reason=entrega_degradada"; then pass "via_submit: otra sesión el mismo día, log sí, mensaje no"
else fail "via_submit: otra sesión el mismo día, log sí, mensaje no" "out=$OUT log=$(witness_reasons)"; fi
if [ -f "$REMIND_SENTINEL_DIR/claude-rules-degradado-$HOY" ]; then pass "via_submit: sentinel diario"
else fail "via_submit: sentinel diario" "falta claude-rules-degradado-$HOY"; fi

# el sentinel diario no silencia el recordatorio /document
rm_degradado
ss_json sb3 ok 1; hbraw sb3 '{"status":"ok","n":1,"via":"submit"}'
run sb3 "$BIG"
if [ "$(msg | tr -d '\r')" = "$DEGRADADO"$'\n'"$REMIND" ]; then pass "via_submit: convive con el recordatorio"
else fail "via_submit: convive con el recordatorio" "out=$OUT"; fi

# via_submit_org_en_log
rm_degradado; : > "$REFLEX_LOG_FILE"
ss_json so ok 1; hbraw so '{"status":"ok","n":1,"via":"submit","org":"claude_team"}'
run so "$SMALL"
if contains "$(witness_reasons)" "org=claude_team"; then pass "via_submit_org_en_log"
else fail "via_submit_org_en_log" "log=$(witness_reasons)"; fi

# via_submit_skip_no_avisa
rm_degradado; : > "$REFLEX_LOG_FILE"
echo '{"status":"skip","reason":"sin_nota","n":0,"repo":"exo"}' > "$RULES/ss-sk"
hbraw sk '{"status":"skip","reason":"sin_seccion","n":0,"via":"submit"}'
run sk "$SMALL"
if [ -z "$OUT" ] && ! contains "$(witness_reasons)" "entrega_degradada"; then pass "via_submit_skip_no_avisa"
else fail "via_submit_skip_no_avisa" "out=$OUT log=$(witness_reasons)"; fi

# via_submit_n_distinto_adjudica
rm_degradado; : > "$REFLEX_LOG_FILE"; stub_reset
ss_json vk ok 3; hbraw vk '{"status":"ok","n":2,"via":"submit"}'
stub '{"data":{"status":"ok","rules":["a","b"]}}'
run vk "$SMALL"
if [ -z "$OUT" ] && contains "$(witness_reasons)" "reason=kb_cambio" \
   && [ "$(jq -r '[.status,.n]|@tsv' "$RULES/ss-vk")" = "$(printf 'ok\t2')" ]; then pass "via_submit_n_distinto_adjudica: kb_cambio"
else fail "via_submit_n_distinto_adjudica: kb_cambio" "out=$OUT log=$(witness_reasons)"; fi
: > "$REFLEX_LOG_FILE"; stub_reset
ss_json vk2 ok 3; hbraw vk2 '{"status":"ok","n":2,"via":"submit"}'
stub '{"data":{"status":"ok","rules":["a","b","c"]}}'
run vk2 "$SMALL"; stub_reset
if contains "$(msg)" "el engine dice ok n=3, el latido dice ok n=2" && contains "$(witness_reasons)" "reason=no_entrego"; then pass "via_submit_n_distinto_adjudica: no_entrego"
else fail "via_submit_n_distinto_adjudica: no_entrego" "out=$OUT log=$(witness_reasons)"; fi

# sin_hb_sigue_siendo_no_cargo
: > "$REFLEX_LOG_FILE"
ss_json nh ok 1
run nh "$SMALL"
if [ "$(msg)" = "$NOCARGO" ] && contains "$(witness_reasons)" "reason=sin_latido"; then pass "sin_hb_sigue_siendo_no_cargo"
else fail "sin_hb_sigue_siendo_no_cargo" "out=$OUT log=$(witness_reasons)"; fi

# hb_sin_via_legacy (1.6.1): se trata como compose
rm_degradado; : > "$REFLEX_LOG_FILE"
ss_json lg ok 2; hbraw lg '{"status":"ok","n":2}'
run lg "$SMALL"
if [ -z "$OUT" ] && ! contains "$(witness_reasons)" "entrega_degradada"; then pass "hb_sin_via_legacy"
else fail "hb_sin_via_legacy" "out=$OUT log=$(witness_reasons)"; fi

# via_none_hb_corrupto: sigue siendo no_entrego
: > "$REFLEX_LOG_FILE"
ss_json vc ok 1; hbraw vc 'no es json{'
run vc "$SMALL"
if contains "$(msg)" "no entregó" && contains "$(witness_reasons)" "reason=no_entrego"; then pass "via_none_hb_corrupto"
else fail "via_none_hb_corrupto" "out=$OUT log=$(witness_reasons)"; fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
