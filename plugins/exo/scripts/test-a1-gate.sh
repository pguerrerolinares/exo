#!/usr/bin/env bash
# Test standalone para a1-gate.sh (Task 6, gate doc docs/superpowers/evals/2026-08-02-a1-gate.md).
# Fixtures en mktemp -d/-p; nunca toca ~/.claude/reflex-log.jsonl ni ~/.claude/projects reales
# (REFLEX_LOG_FILE / REFLEX_PROJECTS_DIR siempre apuntan a fixtures).
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
GATE="${SCRIPT_DIR}/a1-gate.sh"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

get() { # get <clave> <output>
  printf '%s\n' "$2" | grep -m1 "^$1=" | cut -d= -f2-
}

# =============================================================================
# CASO 1 — fixture principal (plan Task 6 Step 1): log sintético + árbol
# REFLEX_PROJECTS_DIR con meta.json variados + 3 transcripts de hijo.
# Ventana: 2026-08-01..2026-08-03.
# =============================================================================
LOG1="$TMP/log1.jsonl"
PROJ1="$TMP/projects1"
mkdir -p "$PROJ1"

# --- Log: 7 campos por línea (ts, reflex, session_id, agent_id, agent_type, tool, payload) ---
cat > "$LOG1" <<'EOF'
{"ts":"2026-08-02T09:00:00Z","reflex":"inject-emitted","session_id":"sidA","agent_id":"a1","agent_type":"general-purpose","tool":"","payload":"type=general-purpose perfil=ejecucion bytes=500"}
{"ts":"2026-08-02T09:05:00Z","reflex":"inject-emitted","session_id":"sidA","agent_id":"a2","agent_type":"general-purpose","tool":"","payload":"type=general-purpose perfil=ejecucion bytes=480"}
{"ts":"2026-08-02T09:10:00Z","reflex":"inject-emitted","session_id":"sidB","agent_id":"a3","agent_type":"general-purpose","tool":"","payload":"type=general-purpose perfil=ejecucion bytes=490"}
{"ts":"2026-08-02T09:15:00Z","reflex":"inject-emitted","session_id":"sidC","agent_id":"a4","agent_type":"Explore","tool":"","payload":"type=Explore perfil=divergente bytes=300"}
{"ts":"2026-08-02T09:20:00Z","reflex":"inject-emitted","session_id":"sidD","agent_id":"a5","agent_type":"Explore","tool":"","payload":"type=Explore perfil=divergente bytes=310"}
{"ts":"2026-08-02T09:25:00Z","reflex":"inject-failed","session_id":"sidE","agent_id":"a8","agent_type":"claude","tool":"","payload":"type=claude perfil=ejecucion"}
{"ts":"2026-08-02T09:30:00Z","reflex":"inject-skipped-depth","session_id":"sidA","agent_id":"a6","agent_type":"general-purpose","tool":"","payload":"type=general-purpose depth=2"}
{"ts":"2026-08-02T09:35:00Z","reflex":"inject-skipped-depth","session_id":"sidF","agent_id":"a9","agent_type":"exo:executor","tool":"","payload":"type=exo:executor depth=2"}
{"ts":"2026-08-02T09:40:00Z","reflex":"git-c","session_id":"sidA","agent_id":"a1","agent_type":"general-purpose","tool":"Bash","payload":"cd x && git status"}
{"ts":"2026-08-02T09:45:00Z","reflex":"git-c-rewrite","session_id":"sidB","agent_id":"a3","agent_type":"general-purpose","tool":"Bash","payload":"cd y && git filter-branch"}
{"ts":"2026-08-02T09:50:00Z","reflex":"zero-residuo","session_id":"sidC","agent_id":"a4","agent_type":"Explore","tool":"Bash","payload":"git add -A"}
{"ts":"2026-08-02T10:00:00Z","reflex":"compact","session_id":"sidG","agent_id":"","agent_type":"","tool":"","payload":"compact"}
{"ts":"2026-08-02T10:05:00Z","reflex":"compact","session_id":"sidH","agent_id":"","agent_type":"","tool":"","payload":"compact"}
{"ts":"2026-08-02T10:10:00Z","reflex":"compact","session_id":"sidG","agent_id":"","agent_type":"","tool":"","payload":"compact"}
{"ts":"2026-07-01T09:00:00Z","reflex":"inject-emitted","session_id":"sidZ","agent_id":"az","agent_type":"general-purpose","tool":"","payload":"type=general-purpose perfil=ejecucion bytes=1"}
{"ts":"2026-08-02T09:12:00Z","reflex":"inject-emitted","session_id":"test-should-be-excluded","agent_id":"at","agent_type":"general-purpose","tool":"","payload":"type=general-purpose perfil=ejecucion bytes=1"}
{"ts":"2026-08-02T09:13:00Z","reflex":"inject-emitted","session_id":"sidLT","agent_id":"alt","agent_type":"general-purpose","tool":"","payload":"type=general-purpose LIVE-TEST"}
EOF

# --- meta.json: 6 cuentan para dispatches_fs (5 con spawnDepth=1 + 1 con
# spawnDepth=2, mtime en ventana — M2: ya NO se filtra por profundidad, la
# politica de la spec no existe en produccion, se cuenta TODO meta.json en
# ventana), 1 excluido por mtime fuera de ventana, 1 excluido por ser sesion
# de test (mismo filtro test* que el resto del gate, aplicado tambien a
# dispatches_fs). ---
mkdir -p "$PROJ1/proj1/sidA/subagents" "$PROJ1/proj1/sidB/subagents" "$PROJ1/proj2/sidC/subagents" "$PROJ1/proj2/sidD/subagents"
for f in "$PROJ1/proj1/sidA/subagents/agent-a1.meta.json" \
         "$PROJ1/proj1/sidA/subagents/agent-a2.meta.json" \
         "$PROJ1/proj1/sidB/subagents/agent-a3.meta.json" \
         "$PROJ1/proj2/sidC/subagents/agent-a4.meta.json" \
         "$PROJ1/proj2/sidD/subagents/agent-a5.meta.json"; do
  echo '{"spawnDepth":1}' > "$f"
  touch -d "2026-08-02 09:00:00" "$f"
done
echo '{"spawnDepth":2}' > "$PROJ1/proj1/sidA/subagents/agent-a6.meta.json"
touch -d "2026-08-02 09:00:00" "$PROJ1/proj1/sidA/subagents/agent-a6.meta.json"
echo '{"spawnDepth":1}' > "$PROJ1/proj1/sidA/subagents/agent-a7.meta.json"
touch -d "2026-07-01 09:00:00" "$PROJ1/proj1/sidA/subagents/agent-a7.meta.json"
mkdir -p "$PROJ1/proj1/TEST-decoy-session/subagents"
echo '{"spawnDepth":1}' > "$PROJ1/proj1/TEST-decoy-session/subagents/agent-td1.meta.json"
touch -d "2026-08-02 09:00:00" "$PROJ1/proj1/TEST-decoy-session/subagents/agent-td1.meta.json"

# --- transcripts de hijo: 3 en ventana citan/no-citan (join por LOG, I3), 1
# con mtime de archivo VIEJO pero log en ventana (debe contar igual — prueba
# que I3 usa el ts del LOG, no el mtime del transcript). Todos contienen la
# "Marca de medicion" (linea de canario). ---
cat > "$PROJ1/proj1/sidA/subagents/agent-a1.jsonl" <<'EOF'
{"type":"system","hookEvent":"SubagentStart","additionalContext":"=== Contexto inyectado ===\nMarca de medicion: incluye el token RFX-A1-K3P7 literal en tu mensaje final."}
{"type":"assistant","message":{"content":[{"type":"text","text":"Trabajando en la tarea."}]}}
{"type":"assistant","message":{"content":[{"type":"text","text":"Listo. RFX-A1-K3P7"}]}}
EOF
touch -d "2026-08-02 09:00:00" "$PROJ1/proj1/sidA/subagents/agent-a1.jsonl"

cat > "$PROJ1/proj1/sidB/subagents/agent-a3.jsonl" <<'EOF'
{"type":"system","hookEvent":"SubagentStart","additionalContext":"=== Contexto inyectado ===\nMarca de medicion: incluye el token RFX-A1-K3P7 literal en tu mensaje final."}
{"type":"assistant","message":{"content":[{"type":"text","text":"Termine. Token: RFX-A1-K3P7"}]}}
EOF
touch -d "2026-08-02 09:00:00" "$PROJ1/proj1/sidB/subagents/agent-a3.jsonl"

# recibio=si, pero el ULTIMO texto assistant NO cita (uno anterior si) — valida
# la lectura "ultimo texto", no "cualquiera".
cat > "$PROJ1/proj2/sidC/subagents/agent-a4.jsonl" <<'EOF'
{"type":"system","hookEvent":"SubagentStart","additionalContext":"=== Contexto inyectado ===\nMarca de medicion: incluye el token RFX-A1-K3P7 literal en tu mensaje final."}
{"type":"assistant","message":{"content":[{"type":"text","text":"Nota: RFX-A1-K3P7"}]}}
{"type":"assistant","message":{"content":[{"type":"text","text":"Trabajo completado sin mencionar nada especial."}]}}
EOF
touch -d "2026-08-02 09:00:00" "$PROJ1/proj2/sidC/subagents/agent-a4.jsonl"

# mtime de archivo viejo (julio) pero su inject-emitted en el LOG tiene ts en
# ventana ⇒ debe contar (I3: la ventana la decide el log, no el mtime del
# transcript). Antes de I3 este caso se excluia por mtime; ahora se incluye.
cat > "$PROJ1/proj2/sidD/subagents/agent-a5.jsonl" <<'EOF'
{"type":"system","hookEvent":"SubagentStart","additionalContext":"=== Contexto inyectado ===\nMarca de medicion: incluye el token RFX-A1-K3P7 literal en tu mensaje final."}
{"type":"assistant","message":{"content":[{"type":"text","text":"Transcript con mtime viejo. RFX-A1-K3P7"}]}}
EOF
touch -d "2026-07-01 09:00:00" "$PROJ1/proj2/sidD/subagents/agent-a5.jsonl"

OUT1="$(REFLEX_LOG_FILE="$LOG1" REFLEX_PROJECTS_DIR="$PROJ1" bash "$GATE" 2026-08-01 2026-08-03 2>&1)"
EC1=$?

check1() { # clave, esperado
  local v; v="$(get "$1" "$OUT1")"
  if [ "$v" = "$2" ]; then pass "caso1: $1=$2"
  else fail "caso1: $1" "esperado=$2 obtenido=$v"; fi
}

if [ $EC1 -ne 0 ]; then
  fail "caso1: exit code" "esperaba 0, obtuve $EC1 — output: $OUT1"
else
  pass "caso1: exit code 0"
fi

check1 "emitidos" "5"
check1 "fallidos" "1"
check1 "skipped_depth" "2"
check1 "abstained" "0"
check1 "dispatches_fs" "6"
check1 "entrega_pct" "83.33"
check1 "violaciones_subagente" "3"
check1 "u2" "0.3750"
check1 "u2_umbral" "0.3345"
check1 "u1_recibieron" "4"
check1 "u1_citaron" "3"
check1 "u1_pct" "75"
check1 "u1_jq_fallidos" "0"
check1 "u1_sin_transcript" "1"
check1 "sesiones" "6"
check1 "sesiones_con_compact" "2"
check1 "tipos" "Explore:2,general-purpose:3"
check1 "veredicto_entrega" "FAIL"
check1 "veredicto_u1" "INSUFICIENTE-N"
check1 "veredicto_u2" "INSUFICIENTE-N"

# =============================================================================
# CASO 2 — logica de umbral U2 en el limite (PASS vs FAIL) con n suficiente
# (>=30 inject-emitted, >=5 sesiones). Generado por loop: 30 emitidos en 6
# sesiones (5 c/u); ventana SEP con 10 violaciones (10/30=0.3333 <= 0.3345 =>
# PASS) y ventana OCT con 11 violaciones (11/30=0.3667 > 0.3345 => FAIL).
# =============================================================================
LOG2="$TMP/log2.jsonl"
: > "$LOG2"
for s in 1 2 3 4 5 6; do
  for k in 1 2 3 4 5; do
    printf '{"ts":"2026-09-01T09:%02d:00Z","reflex":"inject-emitted","session_id":"sepS%s","agent_id":"sep%s%s","agent_type":"general-purpose","tool":"","payload":"type=general-purpose perfil=ejecucion bytes=100"}\n' \
      "$((s*5+k))" "$s" "$s" "$k" >> "$LOG2"
    printf '{"ts":"2026-10-01T09:%02d:00Z","reflex":"inject-emitted","session_id":"octS%s","agent_id":"oct%s%s","agent_type":"general-purpose","tool":"","payload":"type=general-purpose perfil=ejecucion bytes=100"}\n' \
      "$((s*5+k))" "$s" "$s" "$k" >> "$LOG2"
  done
done
for i in $(seq 1 10); do
  printf '{"ts":"2026-09-01T10:%02d:00Z","reflex":"git-c","session_id":"sepS%s","agent_id":"sepv%s","agent_type":"general-purpose","tool":"Bash","payload":"cd x && git status"}\n' \
    "$i" "$(( (i % 6) + 1 ))" "$i" >> "$LOG2"
done
for i in $(seq 1 11); do
  printf '{"ts":"2026-10-01T10:%02d:00Z","reflex":"git-c","session_id":"octS%s","agent_id":"octv%s","agent_type":"general-purpose","tool":"Bash","payload":"cd x && git status"}\n' \
    "$i" "$(( (i % 6) + 1 ))" "$i" >> "$LOG2"
done

OUT2SEP="$(REFLEX_LOG_FILE="$LOG2" REFLEX_PROJECTS_DIR="$TMP/no-existe" bash "$GATE" 2026-09-01 2026-09-01 2>&1)"
EC2SEP=$?
OUT2OCT="$(REFLEX_LOG_FILE="$LOG2" REFLEX_PROJECTS_DIR="$TMP/no-existe" bash "$GATE" 2026-10-01 2026-10-01 2>&1)"
EC2OCT=$?

if [ $EC2SEP -eq 0 ]; then pass "caso2 (sep): exit code 0"; else fail "caso2 (sep): exit code" "obtuve $EC2SEP — $OUT2SEP"; fi
if [ $EC2OCT -eq 0 ]; then pass "caso2 (oct): exit code 0"; else fail "caso2 (oct): exit code" "obtuve $EC2OCT — $OUT2OCT"; fi

v="$(get emitidos "$OUT2SEP")";        [ "$v" = "30" ]     && pass "caso2 (sep): emitidos=30"       || fail "caso2 (sep): emitidos" "obtuve $v"
v="$(get sesiones "$OUT2SEP")";        [ "$v" = "6" ]      && pass "caso2 (sep): sesiones=6"         || fail "caso2 (sep): sesiones" "obtuve $v"
v="$(get u2 "$OUT2SEP")";              [ "$v" = "0.3333" ] && pass "caso2 (sep): u2=0.3333"          || fail "caso2 (sep): u2" "obtuve $v"
v="$(get veredicto_u2 "$OUT2SEP")";    [ "$v" = "PASS" ]   && pass "caso2 (sep): veredicto_u2=PASS"  || fail "caso2 (sep): veredicto_u2" "obtuve $v"

v="$(get emitidos "$OUT2OCT")";        [ "$v" = "30" ]     && pass "caso2 (oct): emitidos=30"        || fail "caso2 (oct): emitidos" "obtuve $v"
v="$(get u2 "$OUT2OCT")";              [ "$v" = "0.3667" ] && pass "caso2 (oct): u2=0.3667"          || fail "caso2 (oct): u2" "obtuve $v"
v="$(get veredicto_u2 "$OUT2OCT")";    [ "$v" = "FAIL" ]   && pass "caso2 (oct): veredicto_u2=FAIL"  || fail "caso2 (oct): veredicto_u2" "obtuve $v"

# n insuficiente: ventana sin ninguna linea (nov vacio) => INSUFICIENTE-N en los tres.
OUT2NOV="$(REFLEX_LOG_FILE="$LOG2" REFLEX_PROJECTS_DIR="$TMP/no-existe" bash "$GATE" 2026-11-01 2026-11-01 2>&1)"
v="$(get veredicto_u1 "$OUT2NOV")";    [ "$v" = "INSUFICIENTE-N" ] && pass "caso2 (nov, vacio): veredicto_u1=INSUFICIENTE-N" || fail "caso2 (nov): veredicto_u1" "obtuve $v"
v="$(get veredicto_u2 "$OUT2NOV")";    [ "$v" = "INSUFICIENTE-N" ] && pass "caso2 (nov, vacio): veredicto_u2=INSUFICIENTE-N" || fail "caso2 (nov): veredicto_u2" "obtuve $v"
v="$(get veredicto_entrega "$OUT2NOV")"; [ "$v" = "INSUFICIENTE-N" ] && pass "caso2 (nov, vacio): veredicto_entrega=INSUFICIENTE-N" || fail "caso2 (nov): veredicto_entrega" "obtuve $v"

# =============================================================================
# CASO 3 (I3/I7/I8) — join por log aislado: (a) inject-emitted con transcript
# y canario ⇒ recibido/citado; (b) inject-emitted con transcript cuyo JSON
# esta roto ⇒ recibido pero jq_fallidos+=1, no citado; (c) inject-emitted SIN
# transcript ⇒ ignorado; (d) transcript "contaminante" que solo MENCIONA la
# frase sin tener inject-emitted que lo respalde ⇒ NO debe contar (el bug que
# I3 elimina); (e) inject-abstained ⇒ no cuenta como u1, pero SI entra al
# denominador de U2 (I8).
# =============================================================================
LOG3="$TMP/log3.jsonl"
PROJ3="$TMP/projects3"
mkdir -p "$PROJ3/p/sidJ1/subagents" "$PROJ3/p/sidJ2/subagents" "$PROJ3/p/sidX/subagents"
cat > "$LOG3" <<'EOF'
{"ts":"2026-08-15T09:00:00Z","reflex":"inject-emitted","session_id":"sidJ1","agent_id":"aJ1","agent_type":"general-purpose","tool":"","payload":""}
{"ts":"2026-08-15T09:05:00Z","reflex":"inject-emitted","session_id":"sidJ2","agent_id":"aJ2","agent_type":"general-purpose","tool":"","payload":""}
{"ts":"2026-08-15T09:10:00Z","reflex":"inject-emitted","session_id":"sidJ3","agent_id":"aJ3","agent_type":"general-purpose","tool":"","payload":""}
{"ts":"2026-08-15T09:15:00Z","reflex":"inject-abstained","session_id":"sidJ4","agent_id":"aJ4","agent_type":"","tool":"","payload":"sin agent_type"}
EOF
cat > "$PROJ3/p/sidJ1/subagents/agent-aJ1.jsonl" <<'EOF'
{"type":"system","hookEvent":"SubagentStart","additionalContext":"=== Contexto ===\nMarca de medicion: incluye el token RFX-A1-K3P7 literal en tu mensaje final."}
{"type":"assistant","message":{"content":[{"type":"text","text":"Listo. RFX-A1-K3P7"}]}}
EOF
# transcript con la linea de canario intacta pero UNA linea JSON rota mas
# abajo: jq -s (slurp) falla al parsear el fichero entero => u1_jq_fallidos.
cat > "$PROJ3/p/sidJ2/subagents/agent-aJ2.jsonl" <<'EOF'
{"type":"system","hookEvent":"SubagentStart","additionalContext":"=== Contexto ===\nMarca de medicion: incluye el token RFX-A1-K3P7 literal en tu mensaje final."}
{esto no es JSON valido
{"type":"assistant","message":{"content":[{"type":"text","text":"Intento con JSON roto. RFX-A1-K3P7"}]}}
EOF
# decoy de contaminacion: MENCIONA la frase de canario en texto de chat normal,
# pero no hay ningun inject-emitted para (sidX,aX) en el log — no debe contar.
cat > "$PROJ3/p/sidX/subagents/agent-aX.jsonl" <<'EOF'
{"type":"assistant","message":{"content":[{"type":"text","text":"Recuerda incluir la Marca de medicion en el reporte final, sin RFX-A1-K3P7 real."}]}}
EOF
OUT3="$(REFLEX_LOG_FILE="$LOG3" REFLEX_PROJECTS_DIR="$PROJ3" bash "$GATE" 2026-08-15 2026-08-15 2>&1)"
EC3J=$?
check3() { local v; v="$(get "$1" "$OUT3")"; if [ "$v" = "$2" ]; then pass "caso3: $1=$2"; else fail "caso3: $1" "esperado=$2 obtenido=$v"; fi; }
if [ $EC3J -eq 0 ]; then pass "caso3: exit code 0"; else fail "caso3: exit code" "obtuve $EC3J — $OUT3"; fi
check3 "emitidos" "3"
check3 "abstained" "1"
check3 "u1_recibieron" "2"
check3 "u1_citaron" "1"
check3 "u1_jq_fallidos" "1"
check3 "u1_pct" "50"
check3 "sesiones" "3"
check3 "u2" "0.0000"

# =============================================================================
# CASO 4 (I9) — ENTREGA al 90%: 10 dispatches elegibles, 9 inject-emitted =>
# entrega_pct=90.00, veredicto_entrega=FAIL (umbral >=95%).
# =============================================================================
LOG4="$TMP/log4.jsonl"; : > "$LOG4"
PROJ4="$TMP/projects4"; mkdir -p "$PROJ4"
i=1
while [ "$i" -le 10 ]; do
  sid="sidE$i"; aid="aE$i"
  mkdir -p "$PROJ4/p/$sid/subagents"
  echo '{"spawnDepth":1}' > "$PROJ4/p/$sid/subagents/agent-${aid}.meta.json"
  touch -d "2026-08-20 09:00:00" "$PROJ4/p/$sid/subagents/agent-${aid}.meta.json"
  if [ "$i" -le 9 ]; then
    printf '{"ts":"2026-08-20T09:%02d:00Z","reflex":"inject-emitted","session_id":"%s","agent_id":"%s","agent_type":"general-purpose","tool":"","payload":""}\n' \
      "$i" "$sid" "$aid" >> "$LOG4"
  fi
  i=$((i+1))
done
OUT4="$(REFLEX_LOG_FILE="$LOG4" REFLEX_PROJECTS_DIR="$PROJ4" bash "$GATE" 2026-08-20 2026-08-20 2>&1)"
v="$(get dispatches_fs "$OUT4")";     [ "$v" = "10" ]     && pass "caso4: dispatches_fs=10"     || fail "caso4: dispatches_fs" "obtuve $v"
v="$(get emitidos "$OUT4")";          [ "$v" = "9" ]      && pass "caso4: emitidos=9"           || fail "caso4: emitidos" "obtuve $v"
v="$(get entrega_pct "$OUT4")";       [ "$v" = "90.00" ]  && pass "caso4: entrega_pct=90.00"    || fail "caso4: entrega_pct" "obtuve $v"
v="$(get veredicto_entrega "$OUT4")"; [ "$v" = "FAIL" ]   && pass "caso4: veredicto_entrega=FAIL" || fail "caso4: veredicto_entrega" "obtuve $v"

# =============================================================================
# CASO 5 (I9) — umbral U1 en el limite, n=20 exactos (join por log, I3):
# 12/20 citan => 60% => PASS; 11/20 citan => 55% => FAIL.
# =============================================================================
gen_u1_fixture() {  # gen_u1_fixture <tmpdir> <n_total> <n_citan> <fecha YYYY-MM-DD>
  local dir="$1" ntot="$2" ncit="$3" tsdate="$4" i sid aid last
  mkdir -p "$dir/proj"
  : > "$dir/log.jsonl"
  i=1
  while [ "$i" -le "$ntot" ]; do
    sid="u1s$i"; aid="u1a$i"
    printf '{"ts":"%sT09:%02d:00Z","reflex":"inject-emitted","session_id":"%s","agent_id":"%s","agent_type":"general-purpose","tool":"","payload":""}\n' \
      "$tsdate" "$i" "$sid" "$aid" >> "$dir/log.jsonl"
    mkdir -p "$dir/proj/p/$sid/subagents"
    if [ "$i" -le "$ncit" ]; then last="Listo. RFX-A1-K3P7"; else last="Listo, sin mencionar nada especial."; fi
    printf '{"type":"system","hookEvent":"SubagentStart","additionalContext":"=== Contexto ===\\nMarca de medicion: incluye el token RFX-A1-K3P7 literal en tu mensaje final."}\n{"type":"assistant","message":{"content":[{"type":"text","text":"%s"}]}}\n' \
      "$last" > "$dir/proj/p/$sid/subagents/agent-${aid}.jsonl"
    i=$((i+1))
  done
}

D5A="$TMP/u1_12de20"; gen_u1_fixture "$D5A" 20 12 "2026-08-10"
OUT5A="$(REFLEX_LOG_FILE="$D5A/log.jsonl" REFLEX_PROJECTS_DIR="$D5A/proj" bash "$GATE" 2026-08-10 2026-08-10 2>&1)"
v="$(get u1_recibieron "$OUT5A")"; [ "$v" = "20" ]  && pass "caso5: 12/20 u1_recibieron=20" || fail "caso5: 12/20 u1_recibieron" "obtuve $v"
v="$(get u1_citaron "$OUT5A")";    [ "$v" = "12" ]  && pass "caso5: 12/20 u1_citaron=12"    || fail "caso5: 12/20 u1_citaron" "obtuve $v"
v="$(get u1_pct "$OUT5A")";        [ "$v" = "60" ]  && pass "caso5: 12/20 u1_pct=60"        || fail "caso5: 12/20 u1_pct" "obtuve $v"
v="$(get veredicto_u1 "$OUT5A")";  [ "$v" = "PASS" ] && pass "caso5: 12/20 veredicto_u1=PASS" || fail "caso5: 12/20 veredicto_u1" "obtuve $v"

D5B="$TMP/u1_11de20"; gen_u1_fixture "$D5B" 20 11 "2026-08-11"
OUT5B="$(REFLEX_LOG_FILE="$D5B/log.jsonl" REFLEX_PROJECTS_DIR="$D5B/proj" bash "$GATE" 2026-08-11 2026-08-11 2>&1)"
v="$(get u1_pct "$OUT5B")";       [ "$v" = "55" ]   && pass "caso5: 11/20 u1_pct=55"        || fail "caso5: 11/20 u1_pct" "obtuve $v"
v="$(get veredicto_u1 "$OUT5B")"; [ "$v" = "FAIL" ] && pass "caso5: 11/20 veredicto_u1=FAIL" || fail "caso5: 11/20 veredicto_u1" "obtuve $v"

# =============================================================================
# CASO 6 (C1/I9) — elegibilidad spawnDepth<=1: 6 metas depth-0 (dispatch de
# equipo) + 4 depth-1 (tool Agent) = 10 dispatches elegibles, 10 emitidos =>
# dispatches_fs=10, entrega_pct=100.00. Corrida sintetica del reviewer.
# =============================================================================
LOG6="$TMP/log6.jsonl"; : > "$LOG6"
PROJ6="$TMP/projects6"; mkdir -p "$PROJ6"
i=1
while [ "$i" -le 10 ]; do
  sid="sidM$i"; aid="aM$i"
  mkdir -p "$PROJ6/p/$sid/subagents"
  if [ "$i" -le 6 ]; then depth=0; else depth=1; fi
  printf '{"spawnDepth":%s}\n' "$depth" > "$PROJ6/p/$sid/subagents/agent-${aid}.meta.json"
  touch -d "2026-08-25 09:00:00" "$PROJ6/p/$sid/subagents/agent-${aid}.meta.json"
  printf '{"ts":"2026-08-25T09:%02d:00Z","reflex":"inject-emitted","session_id":"%s","agent_id":"%s","agent_type":"general-purpose","tool":"","payload":""}\n' \
    "$i" "$sid" "$aid" >> "$LOG6"
  i=$((i+1))
done
OUT6="$(REFLEX_LOG_FILE="$LOG6" REFLEX_PROJECTS_DIR="$PROJ6" bash "$GATE" 2026-08-25 2026-08-25 2>&1)"
v="$(get dispatches_fs "$OUT6")"; [ "$v" = "10" ]    && pass "caso6: 6 depth-0+4 depth-1 => dispatches_fs=10" || fail "caso6: dispatches_fs" "obtuve $v"
v="$(get entrega_pct "$OUT6")";   [ "$v" = "100.00" ] && pass "caso6: entrega_pct=100.00"                    || fail "caso6: entrega_pct" "obtuve $v"

# =============================================================================
# CASO 7 (M3) — log vacio (0 bytes): mensaje propio "log vacío", distinto del
# de JSON invalido, exit != 0. Nombre de fixture SIN "vac" a proposito: si el
# nombre de archivo colara "vacio" al mensaje via interpolacion, el grep de
# abajo daria falso-positivo incluso con el mensaje generico de JSON invalido
# (asi se descubrio el problema al validar este mismo test).
# =============================================================================
LOGVACIO="$TMP/log-caso7-m3.jsonl"; : > "$LOGVACIO"
OUT7="$(REFLEX_LOG_FILE="$LOGVACIO" REFLEX_PROJECTS_DIR="$TMP/no-existe" bash "$GATE" 2026-08-01 2026-08-01 2>&1)"
EC7=$?
if [ $EC7 -ne 0 ] && printf '%s' "$OUT7" | grep -qi "vac" && ! printf '%s' "$OUT7" | grep -qi "JSON invalido"; then
  pass "caso7: log vacio => exit!=0, mensaje propio 'vacío' (no el de JSON invalido)"
else
  fail "caso7: log vacio => exit!=0, mensaje propio 'vacío' (no el de JSON invalido)" "ec=$EC7 out=$OUT7"
fi

# =============================================================================
# CASO 8 — argumentos ausentes => exit != 0 (uso incorrecto).
# =============================================================================
bash "$GATE" >/dev/null 2>&1
EC8="$?"
if [ $EC8 -ne 0 ]; then pass "caso8: sin args => exit != 0"; else fail "caso8: sin args" "exit 0 (esperaba error de uso)"; fi

# =============================================================================
# CASO 9 (M3) — union de candidatos de cierre (ultimo text O ultimo
# SendMessage). 5 sub-casos, uno por transcript:
#  (a) token SOLO en el SendMessage (el texto de cierre no lo trae)     => citado
#  (b) token SOLO en el texto de cierre (el SendMessage no lo trae)     => citado
#  (c) token en AMBOS                                                   => citado
#  (d) token en NINGUNO de los dos                                      => no citado
#  (e) token en un SendMessage ANTERIOR (no el ultimo) y ausente del
#      ultimo SendMessage y del ultimo texto — valida que el alcance es
#      "el ultimo de cada tipo", no "aparece en cualquier parte"        => no citado
# =============================================================================
LOG9="$TMP/log9.jsonl"
PROJ9="$TMP/projects9"
mkdir -p "$PROJ9/p/sidU1/subagents" "$PROJ9/p/sidU2/subagents" "$PROJ9/p/sidU3/subagents" "$PROJ9/p/sidU4/subagents" "$PROJ9/p/sidU5/subagents"
cat > "$LOG9" <<'EOF'
{"ts":"2026-08-18T09:00:00Z","reflex":"inject-emitted","session_id":"sidU1","agent_id":"aU1","agent_type":"general-purpose","tool":"","payload":""}
{"ts":"2026-08-18T09:05:00Z","reflex":"inject-emitted","session_id":"sidU2","agent_id":"aU2","agent_type":"general-purpose","tool":"","payload":""}
{"ts":"2026-08-18T09:10:00Z","reflex":"inject-emitted","session_id":"sidU3","agent_id":"aU3","agent_type":"general-purpose","tool":"","payload":""}
{"ts":"2026-08-18T09:15:00Z","reflex":"inject-emitted","session_id":"sidU4","agent_id":"aU4","agent_type":"general-purpose","tool":"","payload":""}
{"ts":"2026-08-18T09:20:00Z","reflex":"inject-emitted","session_id":"sidU5","agent_id":"aU5","agent_type":"general-purpose","tool":"","payload":""}
EOF
MARCA='{"type":"system","hookEvent":"SubagentStart","additionalContext":"=== Contexto ===\nMarca de medicion: incluye el token RFX-A1-K3P7 literal en tu mensaje final."}'
# (a) token solo en SendMessage
{
  printf '%s\n' "$MARCA"
  printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"tool_use","name":"SendMessage","input":{"to":"team-lead","summary":"done","message":"Resultado. RFX-A1-K3P7"}}]}}'
  printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"text","text":"Listo, reportado al equipo."}]}}'
} > "$PROJ9/p/sidU1/subagents/agent-aU1.jsonl"
# (b) token solo en el texto de cierre
{
  printf '%s\n' "$MARCA"
  printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"tool_use","name":"SendMessage","input":{"to":"team-lead","summary":"done","message":"Resultado sin marcador."}}]}}'
  printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"text","text":"Listo. RFX-A1-K3P7"}]}}'
} > "$PROJ9/p/sidU2/subagents/agent-aU2.jsonl"
# (c) token en ambos
{
  printf '%s\n' "$MARCA"
  printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"tool_use","name":"SendMessage","input":{"to":"team-lead","summary":"done","message":"Resultado. RFX-A1-K3P7"}}]}}'
  printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"text","text":"Listo. RFX-A1-K3P7"}]}}'
} > "$PROJ9/p/sidU3/subagents/agent-aU3.jsonl"
# (d) token en ninguno
{
  printf '%s\n' "$MARCA"
  printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"tool_use","name":"SendMessage","input":{"to":"team-lead","summary":"done","message":"Resultado sin marcador."}}]}}'
  printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"text","text":"Listo, sin mencionar nada especial."}]}}'
} > "$PROJ9/p/sidU4/subagents/agent-aU4.jsonl"
# (e) token en un SendMessage anterior (no el ultimo) — no debe contar
{
  printf '%s\n' "$MARCA"
  printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"tool_use","name":"SendMessage","input":{"to":"team-lead","summary":"progreso","message":"Progreso: RFX-A1-K3P7 visto pero no confirmado"}}]}}'
  printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"tool_use","name":"SendMessage","input":{"to":"team-lead","summary":"done","message":"Resultado final sin marcador."}}]}}'
  printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"text","text":"Listo, sin mencionar nada especial."}]}}'
} > "$PROJ9/p/sidU5/subagents/agent-aU5.jsonl"
OUT9="$(REFLEX_LOG_FILE="$LOG9" REFLEX_PROJECTS_DIR="$PROJ9" bash "$GATE" 2026-08-18 2026-08-18 2>&1)"
EC9=$?
if [ $EC9 -eq 0 ]; then pass "caso9: exit code 0"; else fail "caso9: exit code" "obtuve $EC9 — $OUT9"; fi
v="$(get u1_recibieron "$OUT9")"; [ "$v" = "5" ] && pass "caso9: u1_recibieron=5" || fail "caso9: u1_recibieron" "obtuve $v"
v="$(get u1_citaron "$OUT9")";    [ "$v" = "3" ] && pass "caso9: u1_citaron=3 (a,b,c citan; d,e no)" || fail "caso9: u1_citaron" "obtuve $v"
v="$(get u1_jq_fallidos "$OUT9")"; [ "$v" = "0" ] && pass "caso9: u1_jq_fallidos=0" || fail "caso9: u1_jq_fallidos" "obtuve $v"

# =============================================================================
# CASO 10 (M4) — u1_sin_transcript dedicado: 2 inject-emitted en ventana, uno
# con transcript real (cuenta u1_recibieron) y otro SIN ningun fichero de
# transcript (poda/retencion simulada) — debe incrementar u1_sin_transcript
# sin afectar u1_recibieron/u1_citaron.
# =============================================================================
LOG10="$TMP/log10.jsonl"
PROJ10="$TMP/projects10"
mkdir -p "$PROJ10/p/sidT1/subagents"
cat > "$LOG10" <<'EOF'
{"ts":"2026-08-19T09:00:00Z","reflex":"inject-emitted","session_id":"sidT1","agent_id":"aT1","agent_type":"general-purpose","tool":"","payload":""}
{"ts":"2026-08-19T09:05:00Z","reflex":"inject-emitted","session_id":"sidT2","agent_id":"aT2","agent_type":"general-purpose","tool":"","payload":""}
EOF
cat > "$PROJ10/p/sidT1/subagents/agent-aT1.jsonl" <<'EOF'
{"type":"system","hookEvent":"SubagentStart","additionalContext":"=== Contexto ===\nMarca de medicion: incluye el token RFX-A1-K3P7 literal en tu mensaje final."}
{"type":"assistant","message":{"content":[{"type":"text","text":"Listo. RFX-A1-K3P7"}]}}
EOF
# sidT2/aT2: sin fichero de transcript alguno (ni siquiera el directorio subagents existe).
OUT10="$(REFLEX_LOG_FILE="$LOG10" REFLEX_PROJECTS_DIR="$PROJ10" bash "$GATE" 2026-08-19 2026-08-19 2>&1)"
EC10=$?
if [ $EC10 -eq 0 ]; then pass "caso10: exit code 0"; else fail "caso10: exit code" "obtuve $EC10 — $OUT10"; fi
v="$(get u1_recibieron "$OUT10")";     [ "$v" = "1" ] && pass "caso10: u1_recibieron=1" || fail "caso10: u1_recibieron" "obtuve $v"
v="$(get u1_citaron "$OUT10")";        [ "$v" = "1" ] && pass "caso10: u1_citaron=1" || fail "caso10: u1_citaron" "obtuve $v"
v="$(get u1_sin_transcript "$OUT10")"; [ "$v" = "1" ] && pass "caso10: u1_sin_transcript=1" || fail "caso10: u1_sin_transcript" "obtuve $v"

# =============================================================================
# CASO 11 (M5) — DEDUP POR AGENTE: un agente continuado (SendMessage) redispara
# SubagentStart y produce VARIOS inject-emitted con el MISMO agent_id, mientras
# el filesystem sigue teniendo UN solo meta.json. Contar eventos cruza eventos
# (numerador) contra agentes (denominador) => entrega_pct >100%, denominador de
# U2 inflado y transcripts ponderados N veces en U1. Las tres metricas deben
# contar AGENTES DISTINTOS, no eventos.
#   aC1: 3 inject-emitted (2 continuaciones), 1 meta.json, transcript que cita.
#   aC2: 1 inject-emitted + 1 inject-failed (mismo agente en ambos eventos:
#        prueba que denom_u2 no lo cuenta dos veces), 1 meta.json, no cita.
# Esperado: emitidos=2, dispatches_fs=2 => entrega_pct=100.00 (no 200.00);
#   fallidos=1; denom_u2=2 (agentes con algun evento, no 2+1=3);
#   violaciones=1 => u2=0.5000 (no 0.2500); u1_recibieron=2 (no 4);
#   u1_citaron=1 (no 3).
# =============================================================================
LOG11="$TMP/log11.jsonl"
PROJ11="$TMP/projects11"
mkdir -p "$PROJ11/p/sidK/subagents"
cat > "$LOG11" <<'EOF'
{"ts":"2026-08-20T09:00:00Z","reflex":"inject-emitted","session_id":"sidK","agent_id":"aC1","agent_type":"exo:executor","tool":"","payload":"type=exo:executor perfil=reducido bytes=1026"}
{"ts":"2026-08-20T09:06:00Z","reflex":"inject-emitted","session_id":"sidK","agent_id":"aC1","agent_type":"exo:executor","tool":"","payload":"type=exo:executor perfil=reducido bytes=1026"}
{"ts":"2026-08-20T09:12:00Z","reflex":"inject-emitted","session_id":"sidK","agent_id":"aC1","agent_type":"exo:executor","tool":"","payload":"type=exo:executor perfil=reducido bytes=1026"}
{"ts":"2026-08-20T09:20:00Z","reflex":"inject-emitted","session_id":"sidK","agent_id":"aC2","agent_type":"general-purpose","tool":"","payload":"type=general-purpose perfil=ejecucion bytes=900"}
{"ts":"2026-08-20T09:21:00Z","reflex":"inject-failed","session_id":"sidK","agent_id":"aC2","agent_type":"general-purpose","tool":"","payload":"type=general-purpose perfil=ejecucion"}
{"ts":"2026-08-20T09:30:00Z","reflex":"git-c","session_id":"sidK","agent_id":"aC1","agent_type":"exo:executor","tool":"Bash","payload":"cd x && git status"}
EOF
for f in "$PROJ11/p/sidK/subagents/agent-aC1.meta.json" "$PROJ11/p/sidK/subagents/agent-aC2.meta.json"; do
  echo '{"spawnDepth":1}' > "$f"
  touch -d "2026-08-20 09:00:00" "$f"
done
cat > "$PROJ11/p/sidK/subagents/agent-aC1.jsonl" <<'EOF'
{"type":"system","hookEvent":"SubagentStart","additionalContext":"=== Contexto ===\nMarca de medicion: incluye el token RFX-A1-K3P7 literal en tu mensaje final."}
{"type":"assistant","message":{"content":[{"type":"text","text":"Hecho. RFX-A1-K3P7"}]}}
EOF
cat > "$PROJ11/p/sidK/subagents/agent-aC2.jsonl" <<'EOF'
{"type":"system","hookEvent":"SubagentStart","additionalContext":"=== Contexto ===\nMarca de medicion: incluye el token RFX-A1-K3P7 literal en tu mensaje final."}
{"type":"assistant","message":{"content":[{"type":"text","text":"Hecho, sin token."}]}}
EOF
OUT11="$(REFLEX_LOG_FILE="$LOG11" REFLEX_PROJECTS_DIR="$PROJ11" bash "$GATE" 2026-08-20 2026-08-20 2>&1)"
EC11=$?
if [ $EC11 -eq 0 ]; then pass "caso11: exit code 0"; else fail "caso11: exit code" "obtuve $EC11 — $OUT11"; fi
v="$(get emitidos "$OUT11")";       [ "$v" = "2" ] && pass "caso11: emitidos=2 (agentes, no 4 eventos)" || fail "caso11: emitidos" "obtuve $v"
v="$(get dispatches_fs "$OUT11")";  [ "$v" = "2" ] && pass "caso11: dispatches_fs=2" || fail "caso11: dispatches_fs" "obtuve $v"
v="$(get entrega_pct "$OUT11")";    [ "$v" = "100.00" ] && pass "caso11: entrega_pct=100.00 (no 200.00)" || fail "caso11: entrega_pct" "obtuve $v"
v="$(get fallidos "$OUT11")";       [ "$v" = "1" ] && pass "caso11: fallidos=1" || fail "caso11: fallidos" "obtuve $v"
v="$(get u2 "$OUT11")";             [ "$v" = "0.5000" ] && pass "caso11: u2=0.5000 (denom=2 agentes, no 4 ni 3)" || fail "caso11: u2" "obtuve $v"
v="$(get u1_recibieron "$OUT11")";  [ "$v" = "2" ] && pass "caso11: u1_recibieron=2 (no 4)" || fail "caso11: u1_recibieron" "obtuve $v"
v="$(get u1_citaron "$OUT11")";     [ "$v" = "1" ] && pass "caso11: u1_citaron=1 (no 3)" || fail "caso11: u1_citaron" "obtuve $v"
v="$(get u1_sin_transcript "$OUT11")"; [ "$v" = "0" ] && pass "caso11: u1_sin_transcript=0" || fail "caso11: u1_sin_transcript" "obtuve $v"

echo ""
TOTAL=$((PASS+FAIL))
echo "=== Resultado: ${PASS}/${TOTAL} pasaron ==="
[ $FAIL -eq 0 ] && exit 0 || exit 1
