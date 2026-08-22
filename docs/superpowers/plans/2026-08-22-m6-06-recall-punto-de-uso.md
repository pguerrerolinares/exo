# M6-06 — Recall en el punto de uso: Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: usa `process:orchestrate` para
> ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`) para
> tracking.

**Goal:** Un hook `UserPromptSubmit` que busca el prompt de Paul en la KB e inyecta
hasta 3 punteros a notas relevantes, sin que el modelo decida nada.

**Architecture:** Un único script bash (`recall-inject.sh`) en el plugin `reflex`,
registrado en `hooks.json`. Cuatro etapas en secuencia, cada una capaz de abortar
hacia "no inyectar nada" con exit 0: gate léxico → búsqueda vía `exo recall --query`
→ composición del bloque desde JSON con `jq` → emisión por
`hookSpecificOutput.additionalContext`. **Cero cambios en el engine** (Rust): el
binario `exo` ya expone todo lo necesario desde M2-08.

**Tech Stack:** bash 4+ (`${var,,}` para lowercase UTF-8), `jq`, `timeout`
(coreutils), el binario `exo` ya instalado, `_reflex-log.sh` para logging.

## Global Constraints

Copiados verbatim de la spec (`specs/2026-08-22-m6-06-recall-punto-de-uso-design.md`).
**Toda tarea los hereda.**

- **Cero cambios en el engine.** Si una tarea parece exigir tocar Rust, para y
  escala: cambia el coste del item.
- **El hook JAMÁS destruye el prompt** (P1). En `UserPromptSubmit` un **exit 2 borra
  el prompt de Paul**. Prohibido `set -e`; toda tubería con `|| true`; `exit 0`
  incondicional al final. Nunca `exit 2`, en ninguna rama.
- **Nada se escapa por stdout** (P6). En este evento el stdout plano de un hook que
  sale con 0 **se inyecta como contexto**. Todo lo que no sea el JSON final va a
  stderr o a `/dev/null`. Ni un `echo` de debug.
- **`--min-similitud 0.40` explícito, siempre.** Sin el flag, `exo recall` cae al
  0.35 de la config RO. Es el sellado de M2-07 y viaja por flag hasta M5a.
- **`--refresca` solo si la DB ya existe** (P5). Nunca bootstrap desde el hook.
- **exit 1 del engine NO es abstención por sí solo** (P2). Es el código de
  *cualquier* error (`main.rs:246`). Distinguir por stderr: contiene `recall vacío`
  ⇒ `empty`; no lo contiene ⇒ `error`.
- **Timeout propio de 5 s** (P4), no el del harness.
- **≤3 punteros, cap 1024 bytes** para el bloque entero, cabecera y línea final
  incluidas.
- **Excluir siempre** `kb-demo/core/core-index` (su cuerpo ya lo inyecta el
  hook de arranque).
- **La lista de stopwords se traduce, no se reescribe**: el artefacto normativo es
  `docs/superpowers/consultas/2026-08-22-m6-06/gate-artefacto.py` (127 entradas
  únicas). Todos los números del gate son propiedades de esa lista exacta.
- **Seams por entorno** en todo (`EXO_BIN`, `EXO_INDEX`, `REFLEX_LOG_FILE`): los
  tests jamás tocan la instalación real, la KB real ni el log real.

---

## File structure

| Fichero | Responsabilidad |
|---|---|
| `plugins/reflex/scripts/recall-inject.sh` | **Create.** El hook entero: gate, búsqueda, composición, emisión. |
| `plugins/reflex/scripts/test-recall-inject.sh` | **Create.** Suite standalone, fixtures en `mktemp -d`, binario `exo` falso. |
| `plugins/reflex/hooks/hooks.json` | **Modify.** Añade el evento `UserPromptSubmit`. |

Moldes a imitar (leerlos antes de escribir código): `scripts/exo-recall.sh` (el hook
hermano: guards, fallback, logging, seams) y `scripts/test-compose-inject.sh` (el
patrón de suite: `pass`/`fail`, `contains`/`not_contains`, `TMP` con `trap`).

---

## Task 1: El gate y el esqueleto a prueba de balas

Al terminar esta tarea el hook existe, decide correctamente si un prompt merece
búsqueda, y **nunca inyecta nada todavía**. Es inofensivo por construcción y ya
protege el prompt de Paul.

**Files:**
- Create: `plugins/reflex/scripts/recall-inject.sh`
- Create: `plugins/reflex/scripts/test-recall-inject.sh`

**Interfaces:**
- Consumes: el JSON de `UserPromptSubmit` por stdin, del que solo usa `.prompt`
  (string) y `.session_id` (string).
- Produces: para tareas siguientes, dentro del script —
  - `gate_skip "$PROMPT"` → exit status `0` = saltar (no buscar), `1` = disparar.
  - `norm_token "$tok"` → escribe a stdout el token normalizado.
  - variables globales ya resueltas: `INPUT`, `PROMPT`, `EXO_BIN`, `EXO_INDEX`.

- [ ] **Step 1: Escribir los tests que fallan**

Crear `plugins/reflex/scripts/test-recall-inject.sh`:

```bash
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

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
```

- [ ] **Step 2: Correr los tests y verificar que fallan**

```bash
chmod +x plugins/reflex/scripts/test-recall-inject.sh
plugins/reflex/scripts/test-recall-inject.sh
```

Expected: FAIL en todos los casos — `recall-inject.sh: No such file or directory`.

- [ ] **Step 3: Implementación mínima**

Crear `plugins/reflex/scripts/recall-inject.sh`:

```bash
#!/usr/bin/env bash
# UserPromptSubmit hook: busca el prompt de Paul en la KB e inyecta punteros a
# lo relevante (M6-06, "recall en el punto de uso"). Transporte mecánico: el
# modelo no decide si buscar.
#
# HAZARD PROPIO DE ESTE EVENTO, y la razón de que aquí no haya `set -e`: un
# exit 2 no degrada, BORRA el prompt de Paul. Es el único hook del harness
# donde un bug destruye input del usuario. Por eso: cero `set -e`, tuberías
# con `|| true`, y `exit 0` incondicional al final.
#
# Segundo hazard: en UserPromptSubmit el stdout plano de un hook que sale con 0
# SE INYECTA COMO CONTEXTO. Un `echo` de debug olvidado no ensucia un log:
# entra en el turno como si fuera material de la KB. Todo lo que no sea el
# JSON final va a stderr o a /dev/null.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

if [ -t 0 ]; then INPUT=""; else INPUT="$(cat)"; fi
[ -n "$INPUT" ] || exit 0

# Seams por entorno: permiten probar el hook sin tocar la instalación real y,
# para otra persona, apuntar a SU KB sin editar el script.
EXO_BIN="${EXO_BIN:-$(command -v exo 2>/dev/null || echo "$HOME/.local/bin/exo")}"
EXO_INDEX="${EXO_INDEX:-$HOME/.exo/index.db}"

log_ri() {  # $1=sufijo de evento  $2=payload
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && \
    reflex_log "recall-inject-$1" "$INPUT" "${2:-}" || true
}

PROMPT="$(printf '%s' "$INPUT" | jq -r '.prompt // empty' 2>/dev/null)" || PROMPT=""
[ -n "$PROMPT" ] || exit 0

# --- Gate léxico -------------------------------------------------------------
# Traducción literal de `norm()` y `STOP` de
# docs/superpowers/consultas/2026-08-22-m6-06/gate-artefacto.py, que es el
# artefacto NORMATIVO: los números del gate (86% de disparo, cero falsos
# negativos topicales) son propiedades de esta lista exacta, así que se traduce,
# no se reescribe. 127 entradas únicas.
STOP='el la los las un una unos unas de del a al en con por para y o u e que se
lo le les mi tu su es son era esta este esto esa ese eso estas estos ya no ni si
tambien tampoco pero aunque como cuando donde cual cuales muy mas menos bien mal
solo ahora luego antes despues aqui ahi alla hoy
ok okey vale dale va venga listo perfe perfecto genial claro exacto correcto
gracias adelante
di haz hazlo corre lanza lanzalo revisa arregla borra quita usa guarda sube baja
sigue continua para espera prueba mira pon dime
pushea push commitea commit mergea mergealo merge rama ramas repo repos master
main pr
uno dos tres cuatro cinco 1 2 3 4 5'
# La lista se escribe en varias líneas por legibilidad, pero la pertenencia se
# comprueba con `case " $STOP " in *" $tok "*`: sin colapsar los saltos de línea
# a espacios, todo token a final de línea fallaría el match y el gate no callaría
# casi nunca.
STOP=" $(printf '%s' "$STOP" | tr '\n' ' ') "

# NFD + minúsculas + strip de acentos, conservando `/`, `.` y `-`. Que conserve
# esos tres NO es descuido: es lo que hace la normalización medida, y quitar
# toda la puntuación construiría un gate distinto del que produjo los números.
norm_token() {
  local t="${1,,}"
  t="$(printf '%s' "$t" \
       | sed -e 's/á/a/g' -e 's/é/e/g' -e 's/í/i/g' -e 's/ó/o/g' -e 's/ú/u/g' \
             -e 's/ü/u/g' -e 's/ñ/n/g' 2>/dev/null)" || t="${1,,}"
  printf '%s' "$t" | sed 's/[^a-z0-9/.-]//g' 2>/dev/null || true
}

gate_skip() {  # 0 = saltar, 1 = disparar
  local p="$1"
  case "$p" in
    '<'*) return 0 ;;          # turnos user no humanos (teammate, notificaciones)
    '/'*|'!'*) return 0 ;;     # comandos al harness, no prompts
  esac
  local tok n hay=0
  for tok in $p; do
    n="$(norm_token "$tok")"
    [ -n "$n" ] || continue
    case "$STOP" in
      *" $n "*) continue ;;
    esac
    case "$n" in
      [0-9]*) [ -z "${n//[0-9]/}" ] && continue ;;
    esac
    hay=1
    break
  done
  [ "$hay" -eq 1 ] && return 1
  return 0
}

# La abstención del gate es el caso NORMAL (~14% de los turnos): no se loguea,
# o el log diría más sobre el silencio que sobre los fallos.
gate_skip "$PROMPT" && exit 0

# --- Guards ------------------------------------------------------------------
if [ ! -x "$EXO_BIN" ]; then
  log_ri "degraded" "reason=no-engine bin=$EXO_BIN"
  exit 0
fi
if [ ! -f "$EXO_INDEX" ]; then
  # Sin índice NO se pasa `--refresca`: dispararía un bootstrap de minutos bajo
  # el timeout del evento. Se abstiene y deja rastro.
  log_ri "degraded" "reason=no-index db=$EXO_INDEX"
  exit 0
fi

exit 0
```

- [ ] **Step 4: Correr los tests y verificar que pasan**

```bash
chmod +x plugins/reflex/scripts/recall-inject.sh
plugins/reflex/scripts/test-recall-inject.sh
```

Expected: `N passed, 0 failed`, **todo en verde**. La tarea es autocontenida: los
casos de "dispara" se verifican por el evento `no-engine` del guard, no por la
búsqueda, que aún no existe.

- [ ] **Step 5: Commit**

```bash
git add plugins/reflex/scripts/recall-inject.sh plugins/reflex/scripts/test-recall-inject.sh
git commit -m "feat(m6-06): el gate del recall en el punto de uso, y el blindaje del prompt

Primer trozo del hook UserPromptSubmit: decide si un prompt merece búsqueda y
no inyecta nada todavía. El gate es la traducción literal del artefacto
normativo (gate-artefacto.py, 127 entradas) — los números que lo justifican son
propiedades de esa lista exacta, así que se traduce y no se reescribe.

Blindaje desde el primer commit porque en este evento un exit 2 no degrada:
borra el prompt de Paul."
```

---

## Task 2: La búsqueda, y saber distinguir el silencio del fallo

**Files:**
- Modify: `plugins/reflex/scripts/recall-inject.sh` (sustituye el `exit 0` final)
- Modify: `plugins/reflex/scripts/test-recall-inject.sh` (añade casos al final,
  antes del bloque `printf '\n%d passed…'`)

**Interfaces:**
- Consumes: de la Task 1 — `gate_skip`, `log_ri`, `EXO_BIN`, `EXO_INDEX`, `PROMPT`.
- Produces: para la Task 3 — la variable `SALIDA` (string, el JSON crudo de
  `exo recall --json`), no vacía solo si la búsqueda tuvo éxito.

- [ ] **Step 1: Escribir los tests que fallan**

Añadir a `test-recall-inject.sh`, justo antes del `printf` final:

```bash
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
```

- [ ] **Step 2: Correr los tests y verificar que fallan**

```bash
plugins/reflex/scripts/test-recall-inject.sh
```

Expected: FAIL en los casos de P2, sellado, P5 y P4 — el script aún no invoca el
engine, así que `$EXO_CALLS` está vacío y no hay eventos de log.

- [ ] **Step 3: Implementación mínima**

En `recall-inject.sh`, **sustituir el `exit 0` final** por:

```bash
# --- Búsqueda ----------------------------------------------------------------
# Pide 4 y capa a 1400 para que quede margen: el filtro de core-index (Task 3)
# puede quitar uno, y con `--limite 3 --cap-bytes 1024` nos quedaríamos en 2
# punteros. El cap real de 1024 sobre el bloque final lo aplica el hook.
EXO_INJECT_TIMEOUT="${EXO_INJECT_TIMEOUT:-5}"
ERR_TMP="$(mktemp)" || ERR_TMP=""

SALIDA="$(timeout "$EXO_INJECT_TIMEOUT" "$EXO_BIN" recall \
            --db "$EXO_INDEX" --query "$PROMPT" \
            --min-similitud 0.40 --limite 4 --cap-bytes 1400 \
            --refresca --json 2>"${ERR_TMP:-/dev/null}")"
RC=$?

ERR=""
[ -n "$ERR_TMP" ] && ERR="$(head -c 300 "$ERR_TMP" 2>/dev/null)" && rm -f "$ERR_TMP"

if [ "$RC" -eq 124 ]; then
  # `timeout` usa 124. Que el guard sea nuestro y no del harness es lo que hace
  # este caso visible: un timeout del harness no dejaría rastro en el log.
  log_ri "degraded" "reason=timeout-guard t=${EXO_INJECT_TIMEOUT}s"
  exit 0
fi

if [ "$RC" -ne 0 ]; then
  # El engine sale con 1 para CUALQUIER error (main.rs:246): la abstención por
  # "ningún hit sobre el umbral" es indistinguible por código de una DB
  # corrupta o un ONNX roto. El distinguidor está en stderr y es estable.
  # Gatear solo por código sería un hook donde el engine roto loguea `empty`
  # para siempre — con forma de abstención correcta, que es la peor forma de
  # romperse.
  case "$ERR" in
    *"recall vacío"*) log_ri "degraded" "reason=empty" ;;
    *) log_ri "degraded" "reason=error rc=$RC err=$(printf '%s' "$ERR" | tr -d '\n' | cut -c1-120)" ;;
  esac
  exit 0
fi

[ -n "$SALIDA" ] || { log_ri "degraded" "reason=empty"; exit 0; }

exit 0
```

- [ ] **Step 4: Correr los tests y verificar que pasan**

```bash
plugins/reflex/scripts/test-recall-inject.sh
```

Expected: `N passed, 0 failed`. Ahora también pasan los dos casos de gate de la
Task 1 que dependían de invocar el engine.

- [ ] **Step 5: Commit**

```bash
git add plugins/reflex/scripts/recall-inject.sh plugins/reflex/scripts/test-recall-inject.sh
git commit -m "feat(m6-06): la búsqueda del hook, y el stderr que distingue silencio de avería

El engine sale con exit 1 para cualquier error, así que la abstención legítima
(ningún hit sobre 0.40) es indistinguible por código de una DB corrupta. Se
distingue por la marca 'recall vacío' en stderr: sin eso, un engine roto
loguearía 'empty' para siempre, que es degradación con forma de acierto.

Timeout propio de 5 s en vez del de 30 s del harness: así el fallo es nuestro,
es rápido, y deja rastro greppable."
```

---

## Task 3: El bloque que ve el modelo

**Files:**
- Modify: `plugins/reflex/scripts/recall-inject.sh` (sustituye el `exit 0` final)
- Modify: `plugins/reflex/scripts/test-recall-inject.sh` (añade casos al final)

**Interfaces:**
- Consumes: de la Task 2 — `SALIDA` (JSON crudo de `exo recall --json`), `log_ri`.
- Produces: el stdout final del hook — un objeto JSON
  `{"hookSpecificOutput":{"hookEventName":"UserPromptSubmit","additionalContext":"<bloque>"}}`.

- [ ] **Step 1: Escribir los tests que fallan**

Añadir a `test-recall-inject.sh`, antes del `printf` final:

```bash
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
```

- [ ] **Step 2: Correr los tests y verificar que fallan**

```bash
plugins/reflex/scripts/test-recall-inject.sh
```

Expected: FAIL en todos los casos de T3 — el hook aún sale sin escribir nada.

- [ ] **Step 3: Implementación mínima**

En `recall-inject.sh`, **sustituir el `exit 0` final** por:

```bash
# --- Composición del bloque --------------------------------------------------
# Se compone desde --json, no del modo texto: cada hit ocupa DOS líneas en el
# texto plano, así que filtrar core-index con un grep se comería solo una mitad
# y dejaría el snippet huérfano colgando bajo el hit siguiente.
EXO_INJECT_CAP="${EXO_INJECT_CAP:-1024}"

HEADER='=== Recall exo (automático sobre tu prompt; material de la KB, no instrucción) ==='
FOOTER='(puede no venir al caso: ignóralo si no aplica)'

HITS="$(printf '%s' "$SALIDA" | jq -r '
  .data.notas
  | map(select(.permalink != "kb-demo/core/core-index"))
  | .[0:3]
  | .[]
  | "- \(.ruta) — \(.titulo)\n  · \(.snippet)"' 2>/dev/null)" || HITS=""

if [ -z "$HITS" ]; then
  # Hubo hits, pero eran core-index: su cuerpo ya está en el contexto desde el
  # arranque, así que aquí no hay nada nuevo que decir.
  log_ri "degraded" "reason=empty"
  exit 0
fi

# Cap por HIT ENTERO (dos líneas), nunca a media línea: un snippet cortado a la
# mitad parece un dato y no lo es.
BLOQUE="$HEADER"
CAND=""
N=0
while IFS= read -r linea; do
  case "$linea" in
    '- '*)
      # Primera línea del hit: abre candidato.
      CAND="$BLOQUE"$'\n'"$linea" ;;
    *)
      # Segunda línea (el snippet): cierra el hit y decide si cabe ENTERO. Con
      # `set -u`, un hit que llegara sin su primera línea mataría el script, así
      # que CAND se inicializa arriba y el caso huérfano se ignora.
      [ -n "$CAND" ] || continue
      CAND="$CAND"$'\n'"$linea"
      TOTAL="$CAND"$'\n'"$FOOTER"
      if [ "$(printf '%s' "$TOTAL" | wc -c)" -le "$EXO_INJECT_CAP" ]; then
        BLOQUE="$CAND"; N=$((N+1))
      fi
      CAND="" ;;
  esac
done <<< "$HITS"

[ "$N" -gt 0 ] || { log_ri "degraded" "reason=empty"; exit 0; }

BLOQUE="$BLOQUE"$'\n'"$FOOTER"
BYTES="$(printf '%s' "$BLOQUE" | wc -c)"
PERMALINKS="$(printf '%s' "$SALIDA" | jq -r '[.data.notas[].permalink] | join(",")' 2>/dev/null)" || PERMALINKS=""

log_ri "emitted" "n_hits=$N bytes=$BYTES permalinks=$PERMALINKS"

# ÚNICA escritura a stdout del script entero (P6).
printf '%s' "$BLOQUE" \
  | jq -Rs '{hookSpecificOutput:{hookEventName:"UserPromptSubmit",additionalContext:.}}' \
  2>/dev/null || true
exit 0
```

- [ ] **Step 4: Correr los tests y verificar que pasan**

```bash
plugins/reflex/scripts/test-recall-inject.sh
```

Expected: `N passed, 0 failed`, con los casos de composición, dedup, cap y formato
en verde.

- [ ] **Step 5: Commit**

```bash
git add plugins/reflex/scripts/recall-inject.sh plugins/reflex/scripts/test-recall-inject.sh
git commit -m "feat(m6-06): el bloque que ve el modelo, con su licencia de ignorarlo

Compone desde --json y no del modo texto: cada hit ocupa dos líneas, así que
filtrar core-index con grep dejaría el snippet huérfano bajo el hit siguiente.

La cabecera declara el bloque mecánico y da licencia explícita de ignorarlo.
No es cortesía: con hybrid sin abstención posible (ningún umbral separa ruido
de acierto en esta KB), el formato ES la única defensa contra falsos positivos
que este diseño tiene."
```

---

## Task 4: Enchufarlo, y verlo funcionar de verdad

**Files:**
- Modify: `plugins/reflex/hooks/hooks.json`
- Modify: `plugins/reflex/.claude-plugin/plugin.json` (bump de versión)

**Interfaces:**
- Consumes: `plugins/reflex/scripts/recall-inject.sh` (ejecutable, contrato de la
  Task 3).
- Produces: nada para tareas posteriores. Es la última.

**Por qué el bump va en esta tarea y no "luego"** (decisión de Paul, pre-flight del
2026-08-22): el `reflex` que corre en las sesiones de Paul es una **copia cacheada**
en `~/.claude/plugins/cache/exo/reflex`, no un symlink al repo. Sin subir la versión,
`claude plugin update` no traería el hook nuevo y el Step 4 —el criterio de cierre de
la spec— no se podría ejecutar. El item quedaría "hecho en repo, invisible en la
máquina", que es la forma exacta de fallo silencioso que esta campaña vino a evitar.

- [ ] **Step 1: Verificar que hoy no hay nada registrado**

```bash
jq -r '.hooks | keys[]' plugins/reflex/hooks/hooks.json
```

Expected: `PreToolUse`, `SessionStart`, `Stop`, `SubagentStart`. **Sin**
`UserPromptSubmit`. Si aparece, para: hay otro consumidor del evento y este plan
asume que no.

- [ ] **Step 2: Registrar el hook**

En `plugins/reflex/hooks/hooks.json`, añadir esta clave dentro de `"hooks"`,
al mismo nivel que `"SessionStart"`:

```json
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "\"${CLAUDE_PLUGIN_ROOT}\"/scripts/recall-inject.sh"
          }
        ]
      }
    ],
```

- [ ] **Step 3: Bump de versión del plugin**

En `plugins/reflex/.claude-plugin/plugin.json`, subir `version` de `0.14.0` a
`0.15.0`. Sin esto el hook no llega a la instalación viva (ver arriba).

```bash
jq -r '.version' plugins/reflex/.claude-plugin/plugin.json
```

Expected: `0.15.0`.

- [ ] **Step 4: Verificar el JSON y la suite completa de reflex**

```bash
jq -e '.hooks.UserPromptSubmit[0].hooks[0].command' plugins/reflex/hooks/hooks.json
plugins/reflex/scripts/test-recall-inject.sh
plugins/reflex/scripts/test-reflex-baseline.sh
```

Expected: el `command` sale por pantalla; ambas suites en verde. (Verificado en el
pre-flight: `test-reflex-baseline.sh` **no** valida el conjunto de eventos de
`hooks.json`, así que no hay que tocarlo. No lo busques.)

- [ ] **Step 5: Verificación end-to-end en vivo (la que cierra el item)**

Esto lo ejecuta Paul, no el agente: requiere `claude plugin update` y una sesión
real de Claude Code.

```bash
# 0. Traer el plugin 0.15.0 a la instalación viva:
claude plugin update exo

# 1. Con el plugin recargado, en una sesión NUEVA, escribir literalmente:
M6-06

# 2. En otra terminal, comprobar que el hook emitió:
grep 'recall-inject-emitted' ~/.claude/reflex-log.jsonl | tail -1 | jq .

# 3. Comprobar que los acks no pagan nada (escribir "dale" en la sesión):
grep -c 'recall-inject' ~/.claude/reflex-log.jsonl
```

Expected: (1) el turno trae punteros a notas de M6-06 sin que nadie los pidiera —
**este es el criterio de cierre de la spec**; (2) un evento `emitted` con `n_hits`
entre 1 y 3 y `bytes` ≤1024; (3) el contador no sube tras el ack.

- [ ] **Step 6: Commit**

```bash
git add plugins/reflex/hooks/hooks.json plugins/reflex/.claude-plugin/plugin.json
git commit -m "feat(m6-06): enchufa el recall en el punto de uso

Registra recall-inject.sh en UserPromptSubmit y sube reflex a 0.15.0: el plugin
vivo es una copia cacheada, así que sin bump el hook se quedaría en el repo y el
criterio de cierre no se podría verificar. Con esto M6 cierra entero y M5b
—desinstalar basic-memory— deja de estar bloqueado.

Rollback: quitar esta clave de hooks.json. Nada más se ha tocado."
```

---

## Notas para el implementer

**Dependencias del entorno** (verificar antes de empezar): `jq`, `timeout`
(coreutils) y bash 4+ para `${var,,}`. Los tres ya los usa reflex hoy.

**Lo que este plan NO hace, y no es un olvido**: no toca el engine, no dedupe entre
turnos de la misma sesión, no trunca la query en prompts gigantes (medido: 8 KB
cuesta ~1,4 s y el embedder trunca solo), y no cubre subagentes. Cada una tiene su
párrafo en §7 de la spec. Si te parece que falta algo, léelo antes de añadirlo.

**El coste está aceptado, no es un bug a optimizar**: ~0,95 s por turno sustantivo y
hasta 3 punteros inyectados haya o no señal. Está firmado en §6 de la spec con los
números que lo justifican.
