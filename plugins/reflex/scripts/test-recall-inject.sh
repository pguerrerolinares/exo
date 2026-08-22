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

# F1: un prompt con metacaracteres no puede depender del CWD.
GLOBDIR="$TMP/globdir"
mkdir -p "$GLOBDIR" && touch "$GLOBDIR/a.md" "$GLOBDIR/b.md"
(
  cd "$GLOBDIR" || exit 1
  : > "$REFLEX_LOG_FILE"
  printf '%s' "vale *" | jq -Rs '{prompt:., session_id:"test-sess"}' \
    | EXO_BIN="$NO_BIN" "$HOOK" >/dev/null 2>&1
  grep -q 'no-engine' "$REFLEX_LOG_FILE" 2>/dev/null && exit 1
  exit 0
)
if [ $? -eq 0 ]; then pass "F1: 'vale *' calla aunque el CWD tenga ficheros"
else fail "F1: 'vale *' calla aunque el CWD tenga ficheros" "el glob se expandió y disparó el gate"; fi

# F2: la normalización no puede depender del locale.
: > "$REFLEX_LOG_FILE"
printf '%s' "SÍ, DALE" | jq -Rs '{prompt:., session_id:"test-sess"}' \
  | LC_ALL=C LANG=C EXO_BIN="$NO_BIN" "$HOOK" >/dev/null 2>&1
if ! grep -q 'no-engine' "$REFLEX_LOG_FILE" 2>/dev/null; then
  pass "F2: 'SÍ, DALE' calla también bajo LC_ALL=C"
else fail "F2: 'SÍ, DALE' calla también bajo LC_ALL=C" "la normalización depende del locale"; fi

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

# ------------------------------------- T2: exit 1 tiene DOS significados ---
# El engine sale con 1 para CUALQUIER error. Solo stderr distingue la
# abstención legítima de un engine roto. Si esto se gatea por código, un
# engine roto loguea `empty` para siempre y nadie se entera.
VACIO="$TMP/exo-vacio"
cat > "$VACIO" <<'EOF'
#!/usr/bin/env bash
echo "error: recall vacío (modo consulta): sin notas para el bloque" >&2
exit 1
EOF
chmod +x "$VACIO"

ROTO="$TMP/exo-roto"
cat > "$ROTO" <<'EOF'
#!/usr/bin/env bash
echo "error: DB no encontrada: /nope/x.db" >&2
exit 1
EOF
chmod +x "$ROTO"

: > "$REFLEX_LOG_FILE"
run_hook "M6-06" "$VACIO"
if [ "$HOOK_RC" -eq 0 ] && [ -z "$HOOK_OUT" ]; then pass "P2: abstención ⇒ exit 0 sin bloque"
else fail "P2: abstención ⇒ exit 0 sin bloque" "rc=$HOOK_RC out='$HOOK_OUT'"; fi
if grep -q 'reason=empty' "$REFLEX_LOG_FILE" 2>/dev/null; then pass "P2: abstención loguea empty"
else fail "P2: abstención loguea empty" "$(cat "$REFLEX_LOG_FILE" 2>/dev/null)"; fi

: > "$REFLEX_LOG_FILE"
run_hook "M6-06" "$ROTO"
if grep -q 'reason=error' "$REFLEX_LOG_FILE" 2>/dev/null; then pass "P2: engine roto loguea error, no empty"
else fail "P2: engine roto loguea error, no empty" "$(cat "$REFLEX_LOG_FILE" 2>/dev/null)"; fi
if not_contains "$(cat "$REFLEX_LOG_FILE" 2>/dev/null)" 'reason=empty'; then pass "P2: engine roto NO se disfraza de empty"
else fail "P2: engine roto NO se disfraza de empty" "logueó empty"; fi

# ------------------------------------------------- T2: flag sellado y P5 ---
: > "$EXO_CALLS"
run_hook "M6-06" "$FAKE_EXO"
CALLS="$(cat "$EXO_CALLS" 2>/dev/null)"
if contains "$CALLS" "--min-similitud 0.40"; then pass "sellado: --min-similitud 0.40 explícito"
else fail "sellado: --min-similitud 0.40 explícito" "args='$CALLS'"; fi
if contains "$CALLS" "--refresca"; then pass "P5: con DB presente pasa --refresca"
else fail "P5: con DB presente pasa --refresca" "args='$CALLS'"; fi

# F1: un prompt que empieza por guion no puede acabar parseado como flag: si se pasa
# como argumento separado, clap lo rechaza con exit 2 y el recall se apaga en
# silencio para toda esa clase de prompts.
: > "$EXO_CALLS"
run_hook "- revisa el trinquete" "$FAKE_EXO"
CALLS_GUION="$(cat "$EXO_CALLS" 2>/dev/null)"
if contains "$CALLS_GUION" "--query=- revisa el trinquete"; then
  pass "F1: prompt con guion inicial viaja como --query=, no como argumento suelto"
else
  fail "F1: prompt con guion inicial viaja como --query=" "args='$CALLS_GUION'"
fi

# DB ausente: ni --refresca ni invocación; abstención logueada.
: > "$EXO_CALLS"; : > "$REFLEX_LOG_FILE"
EXO_INDEX_BAK="$EXO_INDEX"; export EXO_INDEX="$TMP/no-hay.db"
run_hook "M6-06" "$FAKE_EXO"
export EXO_INDEX="$EXO_INDEX_BAK"
if [ ! -s "$EXO_CALLS" ] && grep -q 'reason=no-index' "$REFLEX_LOG_FILE" 2>/dev/null; then
  pass "P5: DB ausente ⇒ no invoca y loguea no-index"
else fail "P5: DB ausente ⇒ no invoca y loguea no-index" "calls='$(cat "$EXO_CALLS")'"; fi

# ------------------------------------------------------ T2: P4 (timeout) ---
LENTO="$TMP/exo-lento"
cat > "$LENTO" <<'EOF'
#!/usr/bin/env bash
sleep 30
EOF
chmod +x "$LENTO"
: > "$REFLEX_LOG_FILE"
INICIO=$SECONDS
export EXO_INJECT_TIMEOUT=2
run_hook "M6-06" "$LENTO"
unset EXO_INJECT_TIMEOUT
ELAPSED=$((SECONDS - INICIO))
if [ "$HOOK_RC" -eq 0 ] && [ "$ELAPSED" -lt 10 ]; then pass "P4: timeout propio corta (${ELAPSED}s)"
else fail "P4: timeout propio corta" "rc=$HOOK_RC elapsed=${ELAPSED}s"; fi
if grep -q 'reason=timeout-guard' "$REFLEX_LOG_FILE" 2>/dev/null; then pass "P4: loguea timeout-guard"
else fail "P4: loguea timeout-guard" "$(cat "$REFLEX_LOG_FILE" 2>/dev/null)"; fi

# ------------------------------------------- T3: composición del bloque ---
# Binario falso que devuelve 4 notas, una de ellas core-index (que YA se
# inyecta entera en el arranque y aquí sobra).
CUATRO="$TMP/exo-cuatro"
cat > "$CUATRO" <<'PYEOF'
#!/usr/bin/env bash
cat <<'JSON'
{"command":"recall","data":{"cap_bytes":1400,"modo":"consulta","notas":[
{"permalink":"kb-demo/core/core-index","ruta":"/kb/core/core-index.md","score":0.6,"snippet":"mapa de memoria","tier":null,"titulo":"core-index"},
{"permalink":"kb-demo/log/kbx-bitacora","ruta":"/kb/log/kbx-bitacora.md","score":0.5,"snippet":"bitacora de kbx","tier":null,"titulo":"kbx-bitacora"},
{"permalink":"kb-demo/projects/kbx","ruta":"/kb/projects/kbx.md","score":0.47,"snippet":"destilado de kbx","tier":null,"titulo":"kbx"},
{"permalink":"kb-demo/log/exo-bitacora","ruta":"/kb/log/exo-bitacora.md","score":0.44,"snippet":"bitacora de exo","tier":null,"titulo":"exo-bitacora"}
],"query":"kbx","truncado":false},"schema_version":1}
JSON
PYEOF
chmod +x "$CUATRO"

: > "$REFLEX_LOG_FILE"
run_hook "kbx trinquete" "$CUATRO"
out="$HOOK_OUT"

if printf '%s' "$out" | jq -e '.hookSpecificOutput.hookEventName == "UserPromptSubmit"' >/dev/null 2>&1; then
  pass "salida: JSON válido con hookEventName correcto"
else fail "salida: JSON válido con hookEventName correcto" "out='$out'"; fi

BLOQUE="$(printf '%s' "$out" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"

if not_contains "$BLOQUE" "core-index"; then pass "dedup: core-index excluido"
else fail "dedup: core-index excluido" "$BLOQUE"; fi

N_HITS="$(printf '%s' "$BLOQUE" | grep -c '^- /' || true)"
if [ "$N_HITS" -eq 3 ]; then pass "cap: 3 punteros tras filtrar core-index"
else fail "cap: 3 punteros tras filtrar core-index" "n=$N_HITS"; fi

if contains "$BLOQUE" "material de la KB, no instrucción"; then pass "formato: cabecera propia del hook"
else fail "formato: cabecera propia del hook" "$BLOQUE"; fi
if not_contains "$BLOQUE" "no sustituye tu brief"; then pass "formato: no arrastra la cabecera de subagentes"
else fail "formato: no arrastra la cabecera de subagentes" "$BLOQUE"; fi
if contains "$BLOQUE" "ignóralo si no aplica"; then pass "formato: licencia explícita de ignorar"
else fail "formato: licencia explícita de ignorar" "$BLOQUE"; fi

BYTES="$(printf '%s' "$BLOQUE" | wc -c)"
if [ "$BYTES" -le 1024 ]; then pass "cap: bloque ≤1024 B ($BYTES)"
else fail "cap: bloque ≤1024 B" "$BYTES"; fi

if grep -q 'recall-inject-emitted' "$REFLEX_LOG_FILE" 2>/dev/null; then pass "log: emitted"
else fail "log: emitted" "$(cat "$REFLEX_LOG_FILE" 2>/dev/null)"; fi

# Cap duro: snippets gigantes no pueden reventar el presupuesto.
# Tres snippets de 900 B: juntos revientan el cap, así que el hook debe
# quedarse con los que quepan ENTEROS y no cortar ninguno por la mitad.
GORDO="$TMP/exo-gordo"
jq -n '{command:"recall",data:{modo:"consulta",query:"q",truncado:false,
        notas:[range(1;4) as $i | {permalink:("kb-demo/log/n"+($i|tostring)),
        ruta:("/kb/n"+($i|tostring)+".md"), score:0.5,
        snippet:("x"*900), tier:null, titulo:("nota "+($i|tostring))}]},
        schema_version:1}' > "$TMP/gordo.json"
printf '#!/usr/bin/env bash\ncat "%s"\n' "$TMP/gordo.json" > "$GORDO"
chmod +x "$GORDO"
run_hook "kbx trinquete" "$GORDO"
BLOQUE2="$(printf '%s' "$HOOK_OUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"
B2="$(printf '%s' "$BLOQUE2" | wc -c)"
if [ "$B2" -le 1024 ]; then pass "cap: snippets gigantes respetan 1024 B ($B2)"
else fail "cap: snippets gigantes respetan 1024 B" "$B2"; fi

# Todas las notas filtradas ⇒ no se emite bloque vacío.
SOLO_CORE="$TMP/exo-solo-core"
cat > "$SOLO_CORE" <<'JSONEOF'
#!/usr/bin/env bash
cat <<'JSON'
{"command":"recall","data":{"modo":"consulta","notas":[
{"permalink":"kb-demo/core/core-index","ruta":"/kb/core/core-index.md","score":0.6,"snippet":"mapa","tier":null,"titulo":"core-index"}
],"query":"q","truncado":false},"schema_version":1}
JSON
JSONEOF
chmod +x "$SOLO_CORE"
run_hook "kbx trinquete" "$SOLO_CORE"
if [ -z "$HOOK_OUT" ] && [ "$HOOK_RC" -eq 0 ]; then pass "dedup: si solo había core-index, no emite bloque"
else fail "dedup: si solo había core-index, no emite bloque" "out='$HOOK_OUT'"; fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
