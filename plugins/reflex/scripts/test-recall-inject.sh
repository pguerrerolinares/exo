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

# Mismo seam que usa el hook para el cap de inyección. Se fija aquí explícito
# (en vez de dejar que cada assert repita "1024" en paralelo al default del
# script) para que un cambio de cap solo se toque en un sitio.
export EXO_INJECT_CAP="${EXO_INJECT_CAP:-1024}"

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
# $3 (opcional) = override de EXO_KB_NAME para ESTA llamada, sin contaminar
# las demás. Usa `${3-...}` (sin `:`) para que pasar "" explícito cuente como
# "quiero vacío", no "no me importa" — así los dos casos de resolución real
# de `exo config --json` pueden anular el seam global sin tocarlo.
run_hook() {  # $1 = prompt  [$2 = exo_bin]  [$3 = EXO_KB_NAME override]
  local bin="${2:-$EXO_BIN}"
  local kbname="${3-$EXO_KB_NAME}"
  printf '%s' "$1" | jq -Rs '{prompt:., session_id:"test-sess"}' \
    | EXO_BIN="$bin" EXO_KB_NAME="$kbname" "$HOOK" > "$TMP/hook-out.txt" 2>/dev/null
  HOOK_RC=$?
  HOOK_OUT="$(cat "$TMP/hook-out.txt" 2>/dev/null)"
}

# --- Un índice falso que existe (para que los guards no aborten por no-index) ---
FAKE_DB="$TMP/index.db"
: > "$FAKE_DB"
export EXO_INDEX="$FAKE_DB"

# Task 8: EXO_EXCLUIR ya no lleva "kb-demo" hardcodeado — sale de
# `exo config --json`. El FAKE_EXO de abajo siempre sale con rc=1 (no
# entiende subcomandos), así que sin este seam el exclude quedaría sin
# prefijo y las fixtures de esta suite (permalinks "kb-demo/...") no
# calzarían con el filtro. Fijar EXO_KB_NAME es el mismo seam que usa el
# script real, no un atajo de test — es el default para todos los casos de
# esta suite salvo los que anulan el 3er arg de `run_hook` para ejercer de
# verdad la extracción vía `exo config --json` (ver T-config al final).
export EXO_KB_NAME="kb-demo"

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
  if [ "$HOOK_RC" -eq 0 ] && [ -z "$HOOK_OUT" ]; then pass "P1: exit 0 sin bloque con binario '$modo'"
  else fail "P1: exit 0 sin bloque con binario '$modo'" "rc=$HOOK_RC out='$HOOK_OUT'"; fi
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

# ------------------------------------------------ T2: P2 (envelope-ilegible) ---
# rc=0 con una salida que NO es el envelope esperado no es una abstención: es el
# engine hablando otro idioma (cambio de schema, salida corrupta). Etiquetarlo
# `empty` lo haría invisible, porque `empty` es el caso normal — el mismo disfraz
# que P2 ya prohíbe en la rama de exit 1.
RARO="$TMP/exo-raro"
printf '#!/usr/bin/env bash\nprintf %s "{\\"schema_version\\":2,\\"data\\":{\\"resultados\\":[]}}"\n' "" > "$RARO"
chmod +x "$RARO"
: > "$REFLEX_LOG_FILE"
run_hook "kbx trinquete" "$RARO"
if grep -q 'envelope-ilegible' "$REFLEX_LOG_FILE" 2>/dev/null && [ "$HOOK_RC" -eq 0 ]; then
  pass "P2: envelope ilegible con rc=0 loguea error, no empty"
else fail "P2: envelope ilegible con rc=0 loguea error" "$(cat "$REFLEX_LOG_FILE" 2>/dev/null)"; fi

# ------------------------------------------- T3: composición del bloque ---
# Binario falso que devuelve 4 notas, una de ellas core-index (que YA se
# inyecta entera en el arranque y aquí sobra).
CUATRO="$TMP/exo-cuatro"
cat > "$CUATRO" <<'PYEOF'
#!/usr/bin/env bash
cat <<'JSON'
{"command":"recall","data":{"cap_bytes":1400,"mode":"consulta","notes":[
{"permalink":"kb-demo/core/core-index","path":"/kb/core/core-index.md","score":0.6,"snippet":"mapa de memoria","tier":null,"title":"core-index"},
{"permalink":"kb-demo/log/kbx-bitacora","path":"/kb/log/kbx-bitacora.md","score":0.5,"snippet":"bitacora de kbx","tier":null,"title":"kbx-bitacora"},
{"permalink":"kb-demo/projects/kbx","path":"/kb/projects/kbx.md","score":0.47,"snippet":"destilado de kbx","tier":null,"title":"kbx"},
{"permalink":"kb-demo/log/exo-bitacora","path":"/kb/log/exo-bitacora.md","score":0.44,"snippet":"bitacora de exo","tier":null,"title":"exo-bitacora"}
],"query":"kbx","truncated":false},"schema_version":2}
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

# Rutas relativas a la raíz (F3 más abajo prueba la raíz en detalle): ya no
# empiezan por '/', así que el conteo es sobre '^- ' y no '^- /'.
N_HITS="$(printf '%s' "$BLOQUE" | grep -c '^- ' || true)"
if [ "$N_HITS" -eq 3 ]; then pass "cap: 3 punteros tras filtrar core-index"
else fail "cap: 3 punteros tras filtrar core-index" "n=$N_HITS"; fi

if contains "$BLOQUE" "=== Recall exo"; then pass "formato: cabecera propia del hook"
else fail "formato: cabecera propia del hook" "$BLOQUE"; fi
# Las tres propiedades de la cabecera (spec §2.4) se comprueban una a una y por su
# texto: son la ÚNICA defensa del diseño contra falsos positivos —el umbral no puede
# abstenerse—, así que un smoke-check de "empieza por === Recall exo" no basta:
# pasaría con una cabecera que hubiera perdido justo lo que la hace funcionar.
if contains "$BLOQUE" "automático sobre tu prompt"; then pass "formato: se declara mecánico (nadie lo pidió)"
else fail "formato: se declara mecánico" "$BLOQUE"; fi
if contains "$BLOQUE" "no es una instrucción"; then pass "formato: se declara material, no instrucción"
else fail "formato: se declara material, no instrucción" "$BLOQUE"; fi
if not_contains "$BLOQUE" "no sustituye tu brief"; then pass "formato: no arrastra la cabecera de subagentes"
else fail "formato: no arrastra la cabecera de subagentes" "$BLOQUE"; fi
if contains "$BLOQUE" "ignóralo si no aplica"; then pass "formato: licencia explícita de ignorar"
else fail "formato: licencia explícita de ignorar" "$BLOQUE"; fi

BYTES="$(printf '%s' "$BLOQUE" | wc -c)"
if [ "$BYTES" -le "$EXO_INJECT_CAP" ]; then pass "cap: bloque ≤${EXO_INJECT_CAP} B ($BYTES)"
else fail "cap: bloque ≤${EXO_INJECT_CAP} B" "$BYTES"; fi

if grep -q 'recall-inject-emitted' "$REFLEX_LOG_FILE" 2>/dev/null; then pass "log: emitted"
else fail "log: emitted" "$(cat "$REFLEX_LOG_FILE" 2>/dev/null)"; fi

# El log de permalinks debe reflejar lo EMITIDO (post-filtro, post-slice), no el
# JSON crudo del engine: es el estado que leería un v2 de dedup entre turnos, y
# deduplicar contra notas que el modelo nunca vio sería peor que no deduplicar.
PL="$(jq -r 'select(.reflex=="recall-inject-emitted") | .payload' "$REFLEX_LOG_FILE" 2>/dev/null | tail -1)"
if not_contains "$PL" "core-index" && [ "$(printf '%s' "$PL" | tr ',' '\n' | grep -c 'kb-demo')" -eq 3 ]; then
  pass "log: permalinks registra los 3 emitidos, no los 4 del engine"
else fail "log: permalinks registra los emitidos" "payload='$PL'"; fi

# Fixture con snippets del tamaño que devuelve el engine de verdad (~200 B).
# El anterior usaba 900 B y hacía que no cupiera NI UN hit: el bloque salía vacío
# y el test medía 0 <= 1024, pasando sin ejercer nunca "cabe entero, no se corta".
GORDO="$TMP/exo-gordo"
jq -n '{data:{notes:[range(1;4) as $i | {
  permalink:("kb-demo/log/n"+($i|tostring)),
  path:("/kb/log/nota-larga-numero-"+($i|tostring)+".md"),
  score:0.5, tier:null,
  title:("nota larga numero "+($i|tostring)),
  snippet:(("palabra "*25)+"fin")}]}}' > "$TMP/gordo.json"
printf '#!/usr/bin/env bash\ncat "%s"\n' "$TMP/gordo.json" > "$GORDO"
chmod +x "$GORDO"
run_hook "kbx trinquete" "$GORDO"
BLOQUE2="$(printf '%s' "$HOOK_OUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"
B2="$(printf '%s' "$BLOQUE2" | wc -c)"
N2="$(printf '%s' "$BLOQUE2" | grep -c '^- ' || true)"
if [ "$B2" -le "$EXO_INJECT_CAP" ] && [ "$N2" -eq 3 ]; then
  pass "cap: 3 hits grandes caben enteros y el bloque respeta ${EXO_INJECT_CAP} B ($B2)"
else
  fail "cap: 3 hits grandes caben enteros bajo ${EXO_INJECT_CAP} B" "bytes=$B2 punteros=$N2"
fi
# Y que el recorte no parta palabras por la mitad.
if not_contains "$BLOQUE2" "palabr…" ; then pass "cap: recorta a frontera de palabra"
else fail "cap: recorta a frontera de palabra" "cortó dentro de una palabra"; fi

# F2: mutation testing encontró que dos invariantes de la spec se pueden borrar
# del código y la suite entera sigue en verde (`recorta` → identidad, y el
# `.[0:3]` del cap de punteros). Los fixtures de arriba no los ejercen: GORDO
# usa snippets que caben enteros y CUATRO nunca supera el presupuesto por hit.
# Estos dos SÍ muerden.

# Recorte por hit: fixture con snippets MÁS GRANDES que el presupuesto derivado
# (~240 B). Con snippets realistas de 200 B no hay nada que recortar y el test no
# ejerce nada — que es justo lo que pasaba antes.
RECORTE="$TMP/exo-recorte"
jq -n '{data:{notes:[range(1;4) as $i | {
  permalink:("kb-demo/log/r"+($i|tostring)),
  path:("/kb/log/recorte-"+($i|tostring)+".md"), score:0.5, tier:null,
  title:("recorte "+($i|tostring)),
  snippet:(("análisis técnico — decisión sellada según medición práctica; "*8))}]}}' \
  > "$TMP/recorte.json"
printf '#!/usr/bin/env bash\ncat "%s"\n' "$TMP/recorte.json" > "$RECORTE"
chmod +x "$RECORTE"
run_hook "kbx trinquete" "$RECORTE"
BL_R="$(printf '%s' "$HOOK_OUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"
BR="$(printf '%s' "$BL_R" | wc -c)"
if [ "$BR" -le "$EXO_INJECT_CAP" ] && [ "$BR" -gt 0 ]; then pass "recorte: snippets largos respetan el cap en BYTES ($BR)"
else fail "recorte: snippets largos respetan el cap en BYTES" "bytes=$BR"; fi
if contains "$BL_R" "…"; then pass "recorte: el recorte por hit se activa de verdad"
else fail "recorte: el recorte por hit se activa de verdad" "sin elipsis: no recortó nada"; fi

# ≤3 punteros: cuatro notas y NINGUNA es core-index, así que el slice es lo único
# que impide que salgan cuatro. El fixture CUATRO no lo prueba (4 − 1 core-index = 3
# con slice y sin él).
CINCO="$TMP/exo-cuatro-sin-core"
jq -n '{data:{notes:[range(1;5) as $i | {
  permalink:("kb-demo/log/n"+($i|tostring)),
  path:("/kb/log/nota-"+($i|tostring)+".md"), score:0.5, tier:null,
  title:("nota "+($i|tostring)), snippet:("cuerpo de la nota "+($i|tostring))}]}}' \
  > "$TMP/cuatro-sin-core.json"
printf '#!/usr/bin/env bash\ncat "%s"\n' "$TMP/cuatro-sin-core.json" > "$CINCO"
chmod +x "$CINCO"
run_hook "kbx trinquete" "$CINCO"
N_C="$(printf '%s' "$HOOK_OUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null | grep -c '^- ' || true)"
if [ "$N_C" -eq 3 ]; then pass "cap: 4 notas sin core-index se cortan a 3 punteros"
else fail "cap: 4 notas sin core-index se cortan a 3 punteros" "n=$N_C"; fi

# --------------------------------------------- T3: composición sin grasa ---
# La raíz de la KB se declara UNA vez en la cabecera y los hits van relativos.
run_hook "kbx trinquete" "$CUATRO"
BL="$(printf '%s' "$HOOK_OUT" | jq -r '.hookSpecificOutput.additionalContext')"
if contains "$BL" "material de la KB en /kb"; then pass "raíz: declarada una vez en la cabecera"
else fail "raíz: declarada una vez en la cabecera" "$BL"; fi
if not_contains "$BL" "- /kb/"; then pass "raíz: los hits llevan ruta relativa"
else fail "raíz: los hits llevan ruta relativa" "$BL"; fi

# El título se omite cuando no aporta sobre el nombre del fichero.
TITREP="$TMP/exo-titrep"
jq -n '{data:{notes:[
 {permalink:"kb-demo/log/kbx-bitacora",path:"/kb/log/kbx-bitacora.md",score:0.5,tier:null,
  title:"kbx-bitacora",snippet:"# kbx-bitacora  cuerpo real de la bitacora"},
 {permalink:"kb-demo/log/otra",path:"/kb/log/otra.md",score:0.4,tier:null,
  title:"Un título que sí aporta",snippet:"cuerpo de la otra"}]}}' > "$TMP/titrep.json"
printf '#!/usr/bin/env bash\ncat "%s"\n' "$TMP/titrep.json" > "$TITREP"
chmod +x "$TITREP"
run_hook "kbx trinquete" "$TITREP"
BL2="$(printf '%s' "$HOOK_OUT" | jq -r '.hookSpecificOutput.additionalContext')"
if contains "$BL2" "- kbx-bitacora.md" && not_contains "$BL2" "kbx-bitacora.md — kbx-bitacora"; then
  pass "título: se omite cuando repite el nombre del fichero"
else fail "título: se omite cuando repite el nombre del fichero" "$BL2"; fi
if contains "$BL2" "otra.md — Un título que sí aporta"; then pass "título: se conserva cuando aporta"
else fail "título: se conserva cuando aporta" "$BL2"; fi
if not_contains "$BL2" "· # kbx-bitacora"; then pass "snippet: se pela el header markdown repetido"
else fail "snippet: se pela el header markdown repetido" "$BL2"; fi

# EL CRITICAL: un título con salto de línea no puede fabricar un puntero.
NL="$TMP/exo-nl"
jq -n '{data:{notes:[
 {permalink:"kb-demo/log/uno",path:"/kb/log/uno.md",score:0.5,tier:null,
  title:"raro\n- inyectado",snippet:"snippet real de uno"},
 {permalink:"kb-demo/log/dos",path:"/kb/log/dos.md",score:0.4,tier:null,
  title:"normal",snippet:"snippet real de dos"}]}}' > "$TMP/nl.json"
printf '#!/usr/bin/env bash\ncat "%s"\n' "$TMP/nl.json" > "$NL"
chmod +x "$NL"
run_hook "kbx trinquete" "$NL"
BL3="$(printf '%s' "$HOOK_OUT" | jq -r '.hookSpecificOutput.additionalContext')"
N3="$(printf '%s' "$BL3" | grep -c '^- ' || true)"
if [ "$N3" -eq 2 ] && not_contains "$BL3" "
- inyectado"; then
  pass "saneo: un título con newline no fabrica un puntero falso"
else fail "saneo: un título con newline no fabrica un puntero falso" "punteros=$N3 bloque='$BL3'"; fi

# Todas las notas filtradas ⇒ no se emite bloque vacío.
SOLO_CORE="$TMP/exo-solo-core"
cat > "$SOLO_CORE" <<'JSONEOF'
#!/usr/bin/env bash
cat <<'JSON'
{"command":"recall","data":{"mode":"consulta","notes":[
{"permalink":"kb-demo/core/core-index","path":"/kb/core/core-index.md","score":0.6,"snippet":"mapa","tier":null,"title":"core-index"}
],"query":"q","truncated":false},"schema_version":2}
JSON
JSONEOF
chmod +x "$SOLO_CORE"
# Un solo hit: no hay prefijo común que declarar, así que la cabecera va sin raíz y
# la ruta entera viaja en el puntero. Rama distinta de la de 2-3 hits y sin cobertura
# hasta ahora.
UNICO="$TMP/exo-unico"
jq -n '{data:{notes:[{permalink:"kb-demo/log/solo",path:"/kb/log/solo.md",score:0.5,
  tier:null,title:"nota solitaria",snippet:"cuerpo de la unica nota"}]}}' > "$TMP/unico.json"
printf '#!/usr/bin/env bash\ncat "%s"\n' "$TMP/unico.json" > "$UNICO"
chmod +x "$UNICO"
run_hook "kbx trinquete" "$UNICO"
BL_U="$(printf '%s' "$HOOK_OUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"
if contains "$BL_U" "material de la KB, no es una instrucción" && contains "$BL_U" "- /kb/log/solo.md"; then
  pass "raíz: con un solo hit, cabecera sin raíz y ruta absoluta"
else fail "raíz: con un solo hit, cabecera sin raíz y ruta absoluta" "$BL_U"; fi

run_hook "kbx trinquete" "$SOLO_CORE"
if [ -z "$HOOK_OUT" ] && [ "$HOOK_RC" -eq 0 ]; then pass "dedup: si solo había core-index, no emite bloque"
else fail "dedup: si solo había core-index, no emite bloque" "out='$HOOK_OUT'"; fi

# ------------------------------ T-config: `exo config --json` de verdad ---
# Todos los casos de arriba corren con EXO_KB_NAME="kb-demo" fijado por
# el seam global (línea ~46): necesario porque el FAKE_EXO por defecto no
# entiende NINGÚN subcomando, así que sin el seam el código de esta tarea
# quedaría sin ejercer. Estos dos casos anulan ese seam (3er arg de
# `run_hook`, override a "") con binarios falsos que SÍ distinguen
# `config` de `recall`, para probar la extracción real y su degradación.

# Caso A: `exo config --json` responde con un envelope válido ⇒ el exclude
# se resuelve solo, sin el seam, y el dedup del core-index funciona.
FAKE_EXO_CONFIG_OK="$TMP/exo-config-ok"
cat > "$FAKE_EXO_CONFIG_OK" <<'EOF'
#!/usr/bin/env bash
if [ "$1" = "config" ]; then
  printf '%s\n' '{"schema_version":2,"command":"config","data":{"kb":{"name":"kb-demo","path":"/kb"}}}'
  exit 0
fi
cat <<'JSON'
{"command":"recall","data":{"mode":"consulta","notes":[
{"permalink":"kb-demo/core/core-index","path":"/kb/core/core-index.md","score":0.6,"snippet":"mapa","tier":null,"title":"core-index"},
{"permalink":"kb-demo/log/kbx-bitacora","path":"/kb/log/kbx-bitacora.md","score":0.5,"snippet":"bitacora de kbx","tier":null,"title":"kbx-bitacora"}
],"query":"kbx","truncated":false},"schema_version":2}
JSON
EOF
chmod +x "$FAKE_EXO_CONFIG_OK"

: > "$REFLEX_LOG_FILE"
run_hook "kbx trinquete" "$FAKE_EXO_CONFIG_OK" ""
BL_CFG_OK="$(printf '%s' "$HOOK_OUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"
if [ "$HOOK_RC" -eq 0 ] && not_contains "$BL_CFG_OK" "core-index"; then
  pass "config real: sin seam, \`exo config --json\` resuelve el exclude y dedupa core-index"
else
  fail "config real: sin seam, \`exo config --json\` resuelve el exclude y dedupa core-index" \
    "rc=$HOOK_RC bloque='$BL_CFG_OK'"
fi
if grep -q 'reason=no-config' "$REFLEX_LOG_FILE" 2>/dev/null; then
  fail "config real: config OK no debe loguear no-config" "$(cat "$REFLEX_LOG_FILE" 2>/dev/null)"
else
  pass "config real: config OK no loguea no-config"
fi

# Caso B: `exo config` falla (subcomando desconocido / config rota) ⇒ sin
# el seam, el script se degrada (exclude sin prefijo) pero deja rastro
# `reason=no-config`, distinguible de no-engine/no-index/empty.
FAKE_EXO_CONFIG_FAIL="$TMP/exo-config-fail"
cat > "$FAKE_EXO_CONFIG_FAIL" <<'EOF'
#!/usr/bin/env bash
if [ "$1" = "config" ]; then
  echo "error: unrecognized subcommand 'config'" >&2
  exit 2
fi
cat <<'JSON'
{"command":"recall","data":{"mode":"consulta","notes":[
{"permalink":"kb-demo/core/core-index","path":"/kb/core/core-index.md","score":0.6,"snippet":"mapa","tier":null,"title":"core-index"}
],"query":"q","truncated":false},"schema_version":2}
JSON
EOF
chmod +x "$FAKE_EXO_CONFIG_FAIL"

: > "$REFLEX_LOG_FILE"
run_hook "kbx trinquete" "$FAKE_EXO_CONFIG_FAIL" ""
NOCFG="$(grep 'reason=no-config' "$REFLEX_LOG_FILE" 2>/dev/null)"
if [ -n "$NOCFG" ] && contains "$NOCFG" "unrecognized subcommand"; then
  pass "config real: \`exo config\` falla ⇒ loguea no-config con el motivo exacto"
else
  fail "config real: \`exo config\` falla ⇒ loguea no-config con el motivo exacto" \
    "$(cat "$REFLEX_LOG_FILE" 2>/dev/null)"
fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
