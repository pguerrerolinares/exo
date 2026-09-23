# Reflejo `search-first` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: usa `exo:orchestrate` para
> ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`) para
> tracking.

**Goal:** que el agente principal reciba un aviso (`additionalContext`,
never-block) cuando llega al primer `Agent`/`Task`/`Edit`/`Write`/
`NotebookEdit` de la sesión sin haber pasado antes por `exo search`/`exo
targets`, con el disparo registrado en `~/.claude/reflex-log.jsonl` para
poder medir la tasa `ok / (ok + search-first)` en el tiempo — corrige el
incidente de la Task 5 de la campaña I (el agente despachó sin buscar,
guiado solo por una regla en prosa que se trunca en silencio).

**Architecture:** cuatro tareas. La Task 1 crea el reflejo en TDD puro:
`plugins/exo/scripts/search-first.sh` (bash puro con camino rápido sin
`jq` desde la 2ª llamada de la sesión; `jq` solo en la 1ª llamada, para leer
`transcript_path` y detectar `tool_use` Bash que casen con `exo search`/`exo
targets`) y su suite `test-search-first.sh` (10 casos, numerados como en la
spec §6). La Task 2 cablea el hook en `plugins/exo/hooks/hooks.json`, sube
el plugin a 1.3.3 y sincroniza los tres documentos declarativos (`README.md`,
`plugins/exo/README.md`, `docs/arquitectura.md`) más los cinco gates de CI.
La Task 3 mide el coste en W11 (camino rápido y primera llamada con una
transcripción real) y publica las cifras en `docs/backlog.md`, sobre un ítem
existente que ya lleva la serie de mediciones de coste de hooks en Windows.
La Task 4 vive en el repo `wisdom-paul` (KB, no código): cambia la frase del
`core-index` a un disparador concreto y enlaza el reflejo desde el ítem
aparcado de «Tasa de re-explicación» — commit separado, en otro repo.

**Tech Stack:** bash puro (camino rápido) + `jq` (detección sobre la
transcripción, primera llamada de la sesión). Sin Rust: ninguna task toca
`engine/src`. Mismo patrón que `clean-orchestrator-research.sh` (sentinel
1×/sesión, abstención en subagentes) y que `_reflex-log.sh` (logging
best-effort, nunca rompe el warn-only).

## Global Constraints

- **Never-block: `exit 0` siempre.** Ningún camino del script puede
  bloquear ni devolver `deny`; como mucho `additionalContext` (warn) o
  silencio.
- **Solo cuenta como búsqueda `exo search`/`exo targets` por Bash** (no hay
  servidor MCP de exo, comprobado el 2026-09-23 — si algún día lo hay, se
  añade su nombre al patrón de detección). `Grep`/`Read` no cuentan.
- **Detección: leer la transcripción una vez por sesión**, en el primer
  trabajo sustantivo. Nunca en `bash-guards.sh` ni en `Stop`.
- **Ámbito: solo el agente principal.** Un subagente ejecutando una tarea
  concreta está exento por doctrina (`agent_id` no vacío → abstención total,
  sin tocar el sentinel).
- **Matcher del hook:** `^(Agent|Task|Edit|Write|NotebookEdit)$` en
  `PreToolUse`.
- **Regex de detección exacta (§4.2 de la spec):**
  `(^|[;&|[:space:]])exo(\.exe)?[[:space:]]+(search|targets)([[:space:]]|$)`
- **Mensaje de aviso (≤3 líneas, verbatim de la spec §4.2):** «Reflejo
  search-first: primer trabajo sustantivo de la sesión sin `exo search`/`exo
  targets` previo. Si el tema puede tener historia en la KB, busca antes
  (`exo search --type hybrid "<tema>"`); si no, dilo en una línea y sigue.»
- **Eventos de log:** `search-first-ok` (positivo), `search-first`
  (negativo, con aviso), `search-first-skip` (el sensor no sabe — sin
  `jq`, sin `transcript_path`, fichero ilegible, o si el `jq` de detección
  falla — nunca avisa, y el motivo va en el payload).
- **Sentinel:** como mucho un intento por sesión (ok o skip cuentan igual
  que "ya se avisó" a efectos de silencio); ruta con prefijo configurable
  por `SEARCH_FIRST_SENTINEL_DIR` (por defecto `/tmp`), solo para aislar
  tests — nombre `claude-search-first-<session_id>`.
- **Criterio de latencia W11 (spec §7):** si el camino rápido supera 150 ms
  p50, antes de mergear se estudia reducir el matcher a `^(Agent|Task)$`.
- **Versión:** plugin `exo` 1.3.2 → **1.3.3** (`plugin.json` y
  `marketplace.json`).
- **Git:** `git add <rutas explícitas>`, nunca `-A`/`.`; `git -C <path>`,
  nunca `cd <path> && git`. Mensajes: `feat(search-first, <área>): …` /
  `test(search-first, <área>): …` / `docs(search-first, <área>): …` para el
  repo `exo`; `docs(kb): …` para el commit en `wisdom-paul` (convención de
  ese repo, ver su `git log`).
- **Commits por separado** (spec §9): (1) script + test; (2) cableado +
  versión + docs; (3) `docs/backlog.md` con las cifras medidas; (4) en
  `wisdom-paul`, regla del core-index + enlace del ítem — repo distinto,
  commit distinto.

---

## Contradicción encontrada entre la spec y el código real (resuelta aquí, ver Task 1 Step 1)

La spec §6, caso 4, ilustra el falso negativo aceptado con
`` `echo "exo search"` `` (con comillas). Verificado con la regex EXACTA de
la spec §4.2 contra jq 1.8.2 (Oniguruma):

```bash
jq -n --arg re '(^|[;&|[:space:]])exo(\.exe)?[[:space:]]+(search|targets)([[:space:]]|$)' \
     --arg s 'echo "exo search"' '$s | test($re)'
# -> false
```

El carácter justo antes de `exo` en `echo "exo search"` es una comilla
doble, que no está en la clase `[;&|[:space:]]` ni es inicio de cadena — la
regex NO matchea esa forma concreta. La misma regex SÍ matchea la forma sin
comillas (`echo exo search`, con un espacio antes de `exo`):

```bash
jq -n --arg re '(^|[;&|[:space:]])exo(\.exe)?[[:space:]]+(search|targets)([[:space:]]|$)' \
     --arg s 'echo exo search' '$s | test($re)'
# -> true
```

Decisión: la regex de §4.2 es la parte más cargada de justificación de la
spec (motivo explícito, casos 1 y 3 de §6 dependen literalmente de ella) —
se usa **verbatim**. El fixture del caso 4 en `test-search-first.sh` usa
`echo exo search` (sin comillas) en vez de la forma exacta de §6, con un
comentario que documenta esta discrepancia. Sigue demostrando el mismo
punto de §4.3 («el patrón mira el comando, no si se ejecutó») sin necesitar
tocar la regex.

---

### Task 1: `search-first.sh` + `test-search-first.sh` (TDD)

**Files:**
- Create: `plugins/exo/scripts/search-first.sh`
- Create: `plugins/exo/scripts/test-search-first.sh`

**Interfaces:**
- Consumes: `reflex_log "<reflex>" "$INPUT" "<payload>"` de
  `plugins/exo/scripts/_reflex-log.sh` (helper compartido ya existente, sin
  cambios).
- Produces: script ejecutable `plugins/exo/scripts/search-first.sh` que lee
  JSON de `PreToolUse` por stdin y opcionalmente escribe a stdout
  `{"hookSpecificOutput":{"hookEventName":"PreToolUse","additionalContext":"<aviso>"}}`;
  variable de entorno `SEARCH_FIRST_SENTINEL_DIR` (default `/tmp`); eventos
  `search-first-ok`/`search-first`/`search-first-skip` en
  `$REFLEX_LOG_FILE`. Los consume la Task 2 (cableado en `hooks.json`, sin
  tocar el script) y la Task 3 (medición, invocándolo directamente).

- [ ] **Step 1: Escribir el test que falla — `test-search-first.sh`**

Crea `plugins/exo/scripts/test-search-first.sh`:

```bash
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
```

Marca ejecutable y corre:

```bash
chmod +x plugins/exo/scripts/test-search-first.sh
bash plugins/exo/scripts/test-search-first.sh
```

Expected: falla de inmediato en el caso 1 y en cascada en el resto —
`search-first.sh: No such file or directory` (el script todavía no existe:
es el rojo de esta tarea). `=== Resultado: 0/11 pasaron ===` (11
aserciones: casos 1-6, 7a, 7b, 8-10).

- [ ] **Step 2: Implementación — `search-first.sh`**

Crea `plugins/exo/scripts/search-first.sh`:

```bash
#!/usr/bin/env bash
# PreToolUse (matcher: ^(Agent|Task|Edit|Write|NotebookEdit)$ -- regex
# exacta en hooks/hooks.json): reflejo "search-first". Warn-only, NUNCA
# bloquea (exit 0 siempre). Recuerda buscar en la KB (`exo search`/`exo
# targets`) antes del primer trabajo sustantivo de la sesion -- incidente de
# origen: Task 5 de la campana I (el agente principal despacho sin buscar,
# guiado solo por una regla en prosa que se trunca en silencio). Diseno
# completo: docs/superpowers/specs/2026-09-23-search-first-design.md.
#
# Abstencion:
#  (a) SOLO dispara en el PADRE -> dentro de un subagente (`agent_id` con
#      VALOR, no vacio) exit 0 SIN tocar el sentinel -- es por-sesion,
#      compartido con el padre (mismo patron que
#      clean-orchestrator-research.sh).
#  (b) Como mucho UNA vez por sesion (sentinel en
#      "${SEARCH_FIRST_SENTINEL_DIR:-/tmp}/claude-search-first-<session_id>";
#      la variable de entorno solo existe para aislar los tests de /tmp
#      real).
#  (c) Si el sensor no sabe -- sin `jq`, sin `transcript_path`, con el
#      fichero ausente/ilegible, o si el `jq` de deteccion falla -- calla
#      (ni avisa ni cuenta como "ok"): `reflex_log "search-first-skip"` con
#      el motivo, y CREA igualmente el sentinel (como mucho un intento por
#      sesion, ok o skip).
#
# Camino rapido (desde la 2a llamada de la sesion en adelante, y para
# distinguir subagente): bash puro, SIN jq -- session_id/agent_id se leen
# con regex de bash sobre el JSON crudo de entrada, tolerante a espacios
# tras ":" segun el serializador (el harness real usa JSON.stringify, que
# no los mete -- mismo supuesto documentado en git-c-bash.sh).
#
# Capa TRIGGER / clase event-watching del proyecto cerebro+reflejos.
set -uo pipefail

INPUT="$(cat)"

# --- Camino rapido: session_id y agent_id sin jq ----------------------------
SID=""
if [[ "$INPUT" =~ \"session_id\"[[:space:]]*:[[:space:]]*\"([^\"]*)\" ]]; then
  SID="${BASH_REMATCH[1]}"
fi

# agent_id CON VALOR (no vacio) -> subagente: abstencion total. El campo
# viene AUSENTE en el padre (convencion del harness y de los demas
# reflejos: `.agent_id // empty`), asi que exigir `[^\"]+` (uno o mas
# caracteres) es exactamente "presente y no vacio".
if [[ "$INPUT" =~ \"agent_id\"[[:space:]]*:[[:space:]]*\"[^\"]+\" ]]; then
  exit 0
fi

SENTINEL_DIR="${SEARCH_FIRST_SENTINEL_DIR:-/tmp}"
SENTINEL="${SENTINEL_DIR}/claude-search-first-${SID:-nosession}"
[ -f "$SENTINEL" ] && exit 0

# --- Primera vez en la sesion (para el padre) -------------------------------
LOG_FILE="${REFLEX_LOG_FILE:-$HOME/.claude/reflex-log.jsonl}"

# Unica escritura de log que NO pasa por _reflex-log.sh/jq: si jq no esta en
# el PATH, _reflex-log.sh tampoco puede correr (el mismo pipe a `jq -c` que
# usa para el resto de reflejos). El payload es fijo, sin contenido de
# usuario -> seguro construirlo a mano, sin escapado.
log_skip_sin_jq() {
  local ts
  ts="$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null)" || ts=""
  printf '{"ts":"%s","reflex":"search-first-skip","session_id":"%s","agent_id":"","agent_type":"","tool":"","payload":"motivo=sin-jq"}\n' \
    "$ts" "$SID" >> "$LOG_FILE" 2>/dev/null || true
}

if ! command -v jq >/dev/null 2>&1; then
  log_skip_sin_jq
  touch "$SENTINEL" 2>/dev/null
  exit 0
fi

# A partir de aqui jq esta disponible: un solo intento por sesion, ok o
# skip -> el sentinel se crea YA, antes de saber el veredicto.
touch "$SENTINEL" 2>/dev/null

TRANSCRIPT="$(printf '%s' "$INPUT" | jq -r '.transcript_path // empty' 2>/dev/null)"

if [ -z "$TRANSCRIPT" ]; then
  . "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "search-first-skip" "$INPUT" "motivo=sin-transcript_path" || true
  exit 0
fi
if [ ! -f "$TRANSCRIPT" ] || [ ! -r "$TRANSCRIPT" ]; then
  . "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "search-first-skip" "$INPUT" "motivo=transcript-ilegible" || true
  exit 0
fi

# --- jq de deteccion sobre la transcripcion ---------------------------------
# `-R` (raw input: una linea = una cadena) + `fromjson?` tolera lineas
# no-JSON SIN abortar el resto del fichero -- una transcripcion JSONL de
# ~2 MB puede traer alguna linea truncada/corrupta; alimentar el fichero
# ENTERO a `jq -c .` sin `-R` revienta con "Invalid JSON text" ante el
# primer token invalido y pierde TODAS las lineas buenas detras. `-n` +
# `inputs` (en vez de una `.` de nivel superior implicita) deja usar `$re`
# una sola vez y devolver un UNICO booleano agregado -- asi `-e` decide el
# exit code sobre ESE booleano, no sobre "el ultimo valor de la ultima
# linea leida".
#
# Filtro: solo bloques `tool_use` de mensajes del asistente
# (`select(.type=="assistant") | .message.content[]? | select(.type==
# "tool_use")`), positivo si `name=="Bash"` y `input.command` casa con la
# regex de la spec. El pie de `recall-inject.sh` escribe "exo search --type
# hybrid" como TEXTO en cada prompt (`FOOTER=`, recall-inject.sh:313) -- un
# grep a pelo sobre la transcripcion daria siempre positivo; mirar solo
# `tool_use` de tipo Bash lo evita.
RE='(^|[;&|[:space:]])exo(\.exe)?[[:space:]]+(search|targets)([[:space:]]|$)'
jq -R -n -e --arg re "$RE" '
  [inputs
   | fromjson?
   | select(.type=="assistant")
   | .message.content[]?
   | select(.type=="tool_use" and .name=="Bash")
   | (.input.command // "")
   | select(test($re))
  ] | length > 0
' "$TRANSCRIPT" >/dev/null 2>&1
EC=$?

case "$EC" in
  0)
    . "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "search-first-ok" "$INPUT" "" || true
    ;;
  1)
    . "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "search-first" "$INPUT" "" || true
    MSG='Reflejo search-first: primer trabajo sustantivo de la sesión sin `exo search`/`exo targets` previo. Si el tema puede tener historia en la KB, busca antes (`exo search --type hybrid "<tema>"`); si no, dilo en una línea y sigue.'
    printf '%s' "$MSG" | jq -Rs '{hookSpecificOutput:{hookEventName:"PreToolUse",additionalContext:.}}'
    ;;
  *)
    . "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "search-first-skip" "$INPUT" "motivo=jq-deteccion-fallo-ec${EC}" || true
    ;;
esac

exit 0
```

Marca ejecutable: `chmod +x plugins/exo/scripts/search-first.sh`.

- [ ] **Step 3: Verlo verde**

Run: `bash plugins/exo/scripts/test-search-first.sh`
Expected: `=== Resultado: 11/11 pasaron ===`, exit 0.

- [ ] **Step 4: shellcheck**

```bash
command -v shellcheck >/dev/null 2>&1 || {
  curl -fsSL -o /tmp/sc.tar.xz https://github.com/koalaman/shellcheck/releases/download/v0.11.0/shellcheck-v0.11.0.linux.x86_64.tar.xz
  tar -xJf /tmp/sc.tar.xz -C /tmp
}
SC="$(command -v shellcheck || echo /tmp/shellcheck-v0.11.0/shellcheck)"
"$SC" -x plugins/exo/scripts/search-first.sh plugins/exo/scripts/test-search-first.sh
```

Expected: sin avisos. Si `SCxxxx` aparece y es un falso positivo deliberado
(p. ej. `SC2016` por comillas simples con `$` literal), se justifica en el
sitio con `# shellcheck disable=SCxxxx # <por qué>`, nunca con exclusión
global.

- [ ] **Step 5: Commit**

```bash
git add plugins/exo/scripts/search-first.sh plugins/exo/scripts/test-search-first.sh
git commit -m "feat(search-first, script): reflejo que avisa si el primer trabajo sustantivo de la sesión no vino precedido de exo search/exo targets -- TDD, 10 casos de la spec (11 aserciones)"
```

---

### Task 2: Cableado — `hooks.json`, versión 1.3.3, `README.md` / `plugins/exo/README.md` / `docs/arquitectura.md`, gates

**Depende de:** Task 1 (el script y su bit de ejecución ya están
commiteados — `test-hooks-json.sh` exige 100755 en el índice de git).

**Files:**
- Modify: `plugins/exo/hooks/hooks.json`
- Modify: `plugins/exo/.claude-plugin/plugin.json`
- Modify: `.claude-plugin/marketplace.json`
- Modify: `README.md`
- Modify: `plugins/exo/README.md`
- Modify: `docs/arquitectura.md`

**Interfaces:**
- Consumes: `plugins/exo/scripts/search-first.sh` (Task 1), referenciado
  por ruta en `hooks.json`.
- Produces: nada que otra task consuma — es la task de cableado final del
  plugin.

- [ ] **Step 1: Cablear el matcher en `hooks.json`**

`Edit` sobre `plugins/exo/hooks/hooks.json`:

old_string:
```
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "\"${CLAUDE_PLUGIN_ROOT}\"/scripts/bash-guards.sh"
          }
        ]
      }
    ],
```

new_string:
```
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "\"${CLAUDE_PLUGIN_ROOT}\"/scripts/bash-guards.sh"
          }
        ]
      },
      {
        "matcher": "^(Agent|Task|Edit|Write|NotebookEdit)$",
        "hooks": [
          {
            "type": "command",
            "command": "\"${CLAUDE_PLUGIN_ROOT}\"/scripts/search-first.sh"
          }
        ]
      }
    ],
```

Verificación rápida (no permanente, solo sanity check de este Step — el
matcher regex por sí no lo cubre `test-hooks-json.sh`, que solo valida
JSON/eventos/`type`/scripts existentes+100755):

```bash
MATCHER='^(Agent|Task|Edit|Write|NotebookEdit)$'
for t in Agent Task Edit Write NotebookEdit; do
  printf '%s' "$t" | grep -Eq -- "$MATCHER" || echo "FALTA: $t"
done
for t in Bash Read Glob Grep TaskX; do
  printf '%s' "$t" | grep -Eq -- "$MATCHER" && echo "SOBRA: $t"
done
```

Expected: sin salida (ningún `FALTA:` ni `SOBRA:`).

- [ ] **Step 2: Verlo verde — `test-hooks-json.sh`**

Run: `bash scripts/test-hooks-json.sh`
Expected: `[OK] test-hooks-json: eventos, type=command y scripts
referenciados, todos 100755` — exit 0. (El script ya es 100755 en el índice
desde la Task 1.)

- [ ] **Step 3: Versión 1.3.2 → 1.3.3**

`Edit` sobre `plugins/exo/.claude-plugin/plugin.json`:

old_string: `"version": "1.3.2",`
new_string: `"version": "1.3.3",`

`Edit` sobre `.claude-plugin/marketplace.json`:

old_string: `      "version": "1.3.2",`
new_string: `      "version": "1.3.3",`

Run: `bash scripts/test-versiones.sh`
Expected: `[OK] engine <ver> · plugin 1.3.3 · ENGINE_MIN <ver>` — exit 0.

- [ ] **Step 4: Tabla de hooks + fila en `README.md`**

`Edit` sobre `README.md`:

old_string:
```
| git-c + zero-residuo + verify-before-done | `PreToolUse:Bash` | `plugins/exo/scripts/bash-guards.sh` |
| exo-recall | `SessionStart` | `plugins/exo/scripts/exo-recall.sh` |
```

new_string:
```
| git-c + zero-residuo + verify-before-done | `PreToolUse:Bash` | `plugins/exo/scripts/bash-guards.sh` |
| search-first | `PreToolUse:Agent\|Task\|Edit\|Write\|NotebookEdit` | `plugins/exo/scripts/search-first.sh` |
| exo-recall | `SessionStart` | `plugins/exo/scripts/exo-recall.sh` |
```

- [ ] **Step 5: Tabla de hooks + fila + recuento en `plugins/exo/README.md`**

`Edit` sobre `plugins/exo/README.md`:

old_string:
```
Tabla exacta al cableado vivo de `hooks/hooks.json` (ocho comandos):
```

new_string:
```
Tabla exacta al cableado vivo de `hooks/hooks.json` (nueve comandos):
```

`Edit` sobre `plugins/exo/README.md`:

old_string:
```
| git-c + zero-residuo + verify-before-done | `PreToolUse:Bash` | `scripts/bash-guards.sh` | fusiona los tres guards de Bash (Task 5, campaña I): reescribe `cd <path> && git <read-only>` → `git -C <path> …`; avisa ante `git add -A`/`--all`/`.`; avisa antes de `git commit` si no hay test verde reciente | rewrite solo si patrón estricto; calla en `git add <ficheros>` explícito; escape hatch `--no-verify` y commits solo-docs (ver comentarios del script) |
| exo-recall | `SessionStart` | `scripts/exo-recall.sh` | inyecta instrucción de memoria + digest 7d, servido por el engine `exo` (SQLite) | — (PUSH); degrada al fallback embebido si el engine instalado es < `ENGINE_MIN` |
```

new_string:
```
| git-c + zero-residuo + verify-before-done | `PreToolUse:Bash` | `scripts/bash-guards.sh` | fusiona los tres guards de Bash (Task 5, campaña I): reescribe `cd <path> && git <read-only>` → `git -C <path> …`; avisa ante `git add -A`/`--all`/`.`; avisa antes de `git commit` si no hay test verde reciente | rewrite solo si patrón estricto; calla en `git add <ficheros>` explícito; escape hatch `--no-verify` y commits solo-docs (ver comentarios del script) |
| search-first | `PreToolUse:Agent\|Task\|Edit\|Write\|NotebookEdit` | `scripts/search-first.sh` | avisa si el primer `Agent`/`Task`/`Edit`/`Write`/`NotebookEdit` de la sesión no fue precedido de `exo search`/`exo targets` en la transcripción | 1×/sesión (sentinel); exenta en subagentes (`agent_id`); sin `jq`, sin `transcript_path`/legible, o si el `jq` de detección falla → skip silencioso (igual crea el sentinel) |
| exo-recall | `SessionStart` | `scripts/exo-recall.sh` | inyecta instrucción de memoria + digest 7d, servido por el engine `exo` (SQLite) | — (PUSH); degrada al fallback embebido si el engine instalado es < `ENGINE_MIN` |
```

- [ ] **Step 6: Verlo verde — `test-docs-vivos.sh`**

Run: `bash scripts/test-docs-vivos.sh`
Expected: `[OK] test-docs-vivos: README.md/docs/{arquitectura,instalacion}.md
sin frases muertas, subcomandos inventados, versiones huérfanas, enlaces
rotos ni tablas de hooks desfasadas` — exit 0 (la comprobación (e) exige
que las dos tablas tengan tantas filas como `hooks.json` cablea: 9).

- [ ] **Step 7: Mención en `docs/arquitectura.md`**

`Edit` sobre `docs/arquitectura.md`:

old_string:
```
**Hooks** (10 comandos cableados en `plugins/exo/hooks/hooks.json`). Son los
```

new_string:
```
**Hooks** (9 comandos cableados en `plugins/exo/hooks/hooks.json`). Son los
```

`Edit` sobre `docs/arquitectura.md`:

old_string:
```
- Los otros dos hooks son guardrails de disciplina, no de memoria:
  `clean-orchestrator-research.sh` (recuerda delegar la investigación web a
  subagentes; solo en el padre, 1 vez por sesión) y `bash-guards.sh` (Task 5,
  campaña I: funde los tres guards de `PreToolUse:Bash` en un solo script —
  reescribe `cd X && git <read-only>` a `git -C X …`, warn en el resto;
  avisa ante `git add -A|--all|.`; avisa ante `git commit` de código sin un
  test verde reciente en el transcript. `git-c-bash.sh`,
  `git-add-all-guard.sh` y `verify-before-commit.sh` se preservan como red
  de regresión, huérfanos de `hooks.json`).
```

new_string:
```
- Los otros tres hooks son guardrails de disciplina, no de memoria:
  `clean-orchestrator-research.sh` (recuerda delegar la investigación web a
  subagentes; solo en el padre, 1 vez por sesión), `bash-guards.sh` (Task 5,
  campaña I: funde los tres guards de `PreToolUse:Bash` en un solo script —
  reescribe `cd X && git <read-only>` a `git -C X …`, warn en el resto;
  avisa ante `git add -A|--all|.`; avisa ante `git commit` de código sin un
  test verde reciente en el transcript. `git-c-bash.sh`,
  `git-add-all-guard.sh` y `verify-before-commit.sh` se preservan como red
  de regresión, huérfanos de `hooks.json`) y `search-first.sh` (recuerda
  `exo search`/`exo targets` en el primer `Agent`/`Task`/`Edit`/`Write`/
  `NotebookEdit` de la sesión si la transcripción no muestra ya una llamada
  — 1 vez por sesión, exento en subagentes; incidente de origen y diseño en
  `docs/superpowers/specs/2026-09-23-search-first-design.md`).
```

Run: `bash scripts/test-docs-vivos.sh` de nuevo (por si el segundo `Edit`
introdujo algún enlace o versión rota).
Expected: mismo `[OK]` que en el Step 6.

- [ ] **Step 8: Verlo verde — `test-versiones.sh`, `test-exec-bit.sh`, `test-plugin.sh`**

```bash
bash scripts/test-versiones.sh
bash scripts/test-exec-bit.sh
./scripts/test-plugin.sh
```

Expected: los tres en verde. `test-plugin.sh` descubre
`test-search-first.sh` por glob (sin lista a mano) e imprime `[PASS]
test-search-first.sh — === Resultado: 11/11 pasaron ===` entre las demás
suites; línea final `test-plugin: OK — <N>/<N> suites del plugin en verde`.

- [ ] **Step 9: Commit**

```bash
git add plugins/exo/hooks/hooks.json plugins/exo/.claude-plugin/plugin.json \
        .claude-plugin/marketplace.json README.md plugins/exo/README.md \
        docs/arquitectura.md
git commit -m "feat(search-first, cableado): matcher PreToolUse en hooks.json, plugin 1.3.2 -> 1.3.3, sync de docs/tablas de hooks (nueve comandos)"
```

---

### Task 3: Medición W11 + `docs/backlog.md`

**Depende de:** Task 2 (el script tiene que estar cableado y con versión
final para que la medición sea representativa de lo que se publica —
aunque el binario invocado es el mismo desde la Task 1).

**Files:**
- Modify: `docs/backlog.md`

**Interfaces:**
- Consumes: `plugins/exo/scripts/search-first.sh` (Task 1), invocado
  directamente (sin pasar por el harness de Claude Code).
- Produces: nada que otra task consuma — cierre documental.

- [ ] **Step 1: Medir el camino rápido (sentinel ya creado), 20 repeticiones**

```bash
TMP="$(mktemp -d)"
SID="bench-search-first-fastpath"
mkdir -p "$TMP/sentinels"
touch "$TMP/sentinels/claude-search-first-$SID"   # pre-crea el sentinel: fuerza el camino rápido
PAYLOAD="$(jq -nc --arg s "$SID" '{session_id:$s, tool_name:"Edit", tool_input:{}, hook_event_name:"PreToolUse"}')"

for i in $(seq 20); do
  s=$(date +%s%N)
  printf '%s' "$PAYLOAD" | SEARCH_FIRST_SENTINEL_DIR="$TMP/sentinels" REFLEX_LOG_FILE="$TMP/log.jsonl" \
    plugins/exo/scripts/search-first.sh > /dev/null
  e=$(date +%s%N); echo $(( (e - s) / 1000000 ))
done | sort -n | awk '{a[NR]=$1} END{print "search_first_fastpath_p50_ms", a[int((NR-1)*0.5)+1], "search_first_fastpath_p95_ms", a[int((NR-1)*0.95)+1]}'

rm -rf "$TMP"
```

Expected: una línea `search_first_fastpath_p50_ms <N> search_first_fastpath_p95_ms
<M>`. Anota `<N>`/`<M>` — se citan en el Step 3. Criterio (spec §7): si
`<N>` (p50) supera 150, antes de continuar aplica el Step 2b (contingente)
más abajo.

- [ ] **Step 2: Medir la primera llamada con una transcripción real, 20 repeticiones**

```bash
TMP="$(mktemp -d)"
mkdir -p "$TMP/sentinels"
TRANSCRIPT="$(ls -t ~/.claude/projects/*/*.jsonl 2>/dev/null | head -1)"
if [ -z "$TRANSCRIPT" ]; then
  echo "sin transcripciones reales en ~/.claude/projects/*/*.jsonl — anota 'sin dato real disponible' en el Step 3" >&2
else
  ls -la "$TRANSCRIPT"   # confirma que ronda ~2 MB, como pide la spec §7
  for i in $(seq 20); do
    PAYLOAD="$(jq -nc --arg s "bench-search-first-first-$i" --arg t "$TRANSCRIPT" \
      '{session_id:$s, transcript_path:$t, tool_name:"Edit", tool_input:{}, hook_event_name:"PreToolUse"}')"
    s=$(date +%s%N)
    printf '%s' "$PAYLOAD" | SEARCH_FIRST_SENTINEL_DIR="$TMP/sentinels" REFLEX_LOG_FILE="$TMP/log.jsonl" \
      plugins/exo/scripts/search-first.sh > /dev/null
    e=$(date +%s%N); echo $(( (e - s) / 1000000 ))
  done | sort -n | awk '{a[NR]=$1} END{print "search_first_first_call_p50_ms", a[int((NR-1)*0.5)+1], "search_first_first_call_p95_ms", a[int((NR-1)*0.95)+1]}'
fi
rm -rf "$TMP"
```

Cada repetición usa un `session_id` distinto (`bench-search-first-first-$i`)
para que el sentinel no exista todavía y cada llamada pague el camino
frío (lectura + `jq` sobre la transcripción entera) de verdad, no el
camino rápido de la Task anterior.

Expected: una línea `search_first_first_call_p50_ms <P> search_first_first_call_p95_ms
<Q>`. Anota `<P>`/`<Q>`.

- [ ] **Step 2b (contingente, solo si el Step 1 dio p50 > 150 ms):
  reducir el matcher**

`Edit` sobre `plugins/exo/hooks/hooks.json`:

old_string:
```
      {
        "matcher": "^(Agent|Task|Edit|Write|NotebookEdit)$",
```

new_string:
```
      {
        "matcher": "^(Agent|Task)$",
```

Repite el Step 1 tras el cambio y usa esas cifras en el Step 3; documenta
en el Step 3 que se aplicó esta reducción y por qué (se pierde el caso de
`Edit`/`Write`/`NotebookEdit` directos sin delegar, spec §7). Si el Step 1
dio p50 ≤ 150 ms, este Step **no se ejecuta**.

- [ ] **Step 3: Publicar las cifras en `docs/backlog.md`**

El ítem que ya lleva la serie de mediciones de coste de hooks en Windows es
`(revisión 2026-09-04) El coste del hook completo en Windows no está
medido` (bajo `## Media`; localízalo por título, no por línea — el propio
fichero advierte que el sync desplaza líneas: `grep -n "El coste del hook
completo en Windows" docs/backlog.md`). Es el mismo ítem que ya acumula las
mediciones W11 de `recall-inject.sh` y del triple `PreToolUse:Bash`
(campañas A e I) — search-first es un hook nuevo del mismo tipo (`W11`,
`PreToolUse`, medido con el mismo patrón `date +%s%N` / `sort -n` / `awk`
percentil), así que se apila ahí en vez de abrir un ítem nuevo.

Antes de editar, releer el final del ítem para confirmar que el
`old_string` sigue siendo literal:

```bash
grep -n "bc4896a. (Task 4)" docs/backlog.md
```

`Edit` sobre `docs/backlog.md`, con `old_string` el final real del ítem
(verificado contra `HEAD` de este plan, `docs/backlog.md:951-954`):

old_string:
```
  Commits: `c462506`, `17afb9b` (Task 1); `d2b069d`, `c00e55e` (Task 2);
  `ee44a5a`, `2588384` (Task 3, el segundo es el fix del review adversarial
  que sustituye la reproducción standalone de SOURCE/SID por extracción del
  código real); `bc4896a` (Task 4).
```

new_string (añade el cierre de search-first, sin borrar el histórico de
arriba — sustituye `<N>`/`<M>`/`<P>`/`<Q>` por las cifras reales de los
Steps 1-2, y la última frase por si aplicó o no el Step 2b):

```
  Commits: `c462506`, `17afb9b` (Task 1); `d2b069d`, `c00e55e` (Task 2);
  `ee44a5a`, `2588384` (Task 3, el segundo es el fix del review adversarial
  que sustituye la reproducción standalone de SOURCE/SID por extracción del
  código real); `bc4896a` (Task 4).
  **search-first (2026-09-23, plan
  `docs/superpowers/plans/2026-09-23-search-first.md`, Task 3):** nuevo
  reflejo `PreToolUse:Agent|Task|Edit|Write|NotebookEdit`
  (`plugins/exo/scripts/search-first.sh`, plugin 1.3.3) que avisa si el
  primer trabajo sustantivo de la sesión no fue precedido de `exo
  search`/`exo targets`. Camino rápido (sentinel ya creado, 20
  repeticiones, W11): p50 = **[RELLENAR search_first_fastpath_p50_ms]** ms,
  p95 = **[RELLENAR search_first_fastpath_p95_ms]** ms — comando exacto en
  el Step 1 de la Task 3 del plan. Primera llamada de la sesión con una
  transcripción real de `~/.claude/projects/*/*.jsonl` (20 repeticiones,
  cada una con `session_id` distinto para forzar el camino frío): p50 =
  **[RELLENAR search_first_first_call_p50_ms]** ms, p95 = **[RELLENAR
  search_first_first_call_p95_ms]** ms — comando exacto en el Step 2 de la
  Task 3 del plan. Criterio (spec §7, umbral 150 ms p50 del camino rápido):
  **[RELLENAR: cumplido, matcher sin cambios / superado, matcher reducido a
  ^(Agent|Task)$ en el Step 2b]**.
```

Al aplicar este `Edit`, sustituye los cuatro `[RELLENAR ...]` por las
cifras reales obtenidas en los Steps 1-2 y por el veredicto real del
criterio **antes de commitear** — el placeholder no llega al commit.

- [ ] **Step 4: Commit**

```bash
git add docs/backlog.md
git commit -m "docs(search-first, backlog): cifras W11 del reflejo -- camino rápido y primera llamada, criterio de 150 ms p50"
```

---

### Task 4 (repo `wisdom-paul`, commit separado): regla del `core-index` + enlace desde el ítem de re-explicación

**Depende de:** Tasks 1-3 mergeadas (el reflejo tiene que existir de verdad
para que la frase del `core-index` y el enlace del backlog no citen algo
que no está en el árbol).

**Files:**
- Modify: `C:/proyectos/homework/wisdom-paul/core/core-index.md`
- Modify: `C:/proyectos/homework/wisdom-paul/backlog/Backlog — exo.md`

**Interfaces:**
- Consumes: nada de las tasks anteriores a nivel de código — cita al
  reflejo `search-first` por nombre/ruta como texto de doctrina.
- Produces: nada — es la última task del plan.

- [ ] **Step 1: Cambiar la frase del `core-index` (línea 17 real)**

`Edit` sobre `C:/proyectos/homework/wisdom-paul/core/core-index.md`:

old_string:
```
Antes de trabajo sustantivo, busca contexto (exo search --type hybrid, exo targets)
```

new_string:
```
Antes del primer Agent/Edit/Write de la sesión, busca contexto (`exo search --type hybrid "<tema>"` o `exo targets`), o di en una línea por qué no aplica (el reflejo `search-first` avisa si no)
```

- [ ] **Step 2: Comprobar el trinquete**

```bash
exo ratchet --kb "C:/proyectos/homework/wisdom-paul"
```

Expected: exit 0 (`core-index.md` no lleva techo sellado en
`.kbx-ratchet.json` — es un índice, exento de presupuesto por la doctrina
del propio `core-index`: «Índices... no se destilan»). Confirmado en este
worktree antes de tocar el fichero: `exo ratchet --kb
"C:/proyectos/homework/wisdom-paul"` ya da exit 0 hoy (una única línea de
deuda informativa sobre `Paul - perfil de trabajo.md`, que no bloquea).

- [ ] **Step 3: Enlazar desde el ítem aparcado de «Tasa de re-explicación»**

`Edit` sobre `C:/proyectos/homework/wisdom-paul/backlog/Backlog — exo.md`:

old_string:
```
  sondas con y sin plugin. Sin umbral pre-registrado por la directiva de método.
```

new_string:
```
  sondas con y sin plugin. Sin umbral pre-registrado por la directiva de método.
  **Instrumento relacionado** (2026-09-23): el reflejo `search-first`
  (plugin `exo` 1.3.3, `plugins/exo/scripts/search-first.sh`) mide la junta
  recuperada→usada en el punto de disparo — log `search-first-ok`/
  `search-first` en `~/.claude/reflex-log.jsonl`; no sustituye este
  embudo, es una sonda más barata en el mismo sitio.
```

- [ ] **Step 4: Commit (repo `wisdom-paul`, no `exo`)**

```bash
git -C "C:/proyectos/homework/wisdom-paul" add "core/core-index.md" "backlog/Backlog — exo.md"
git -C "C:/proyectos/homework/wisdom-paul" commit -m "docs(kb): search-first -- dispara concreto en core-index + enlace desde el ítem de re-explicación"
```

---

## Self-review contra la spec

- **Cobertura de cada sección de la spec:** §1-3 (contexto/diagnóstico/
  decisiones) no piden código, son las Global Constraints y el Goal. §4.1
  (componente + matcher) → Task 2 Step 1. §4.2 (flujo completo: camino
  rápido sin jq, primera vez, jq de detección, resultado, log) → Task 1
  Step 2. §4.3 (limitaciones aceptadas) → documentadas en el script y
  ejercitadas en los casos 2/4/10 del test. §5 (regla del core-index) →
  Task 4 Step 1-2. §6 (los 10 casos) → Task 1 Step 1, uno a uno, mismo
  orden. §7 (medición W11 + criterio 150 ms) → Task 3 Steps 1-2-2b. §8
  (comprobación futura + enlace desde el backlog de `wisdom-paul`, "no se
  abre un ítem nuevo") → Task 4 Step 3, respetado literalmente (no se creó
  ítem nuevo en `wisdom-paul`). §9 (entrega: versión, hooks.json, tablas +
  recuento, mención en arquitectura, los 5 gates, los 4 commits) → Task 2
  completa + Task 3/4 para los commits restantes. §10 (fuera de alcance) →
  ninguna task toca `recall-inject.sh` ni `subagent-inject.sh` ni añade un
  `exit 2`.
- **Placeholders:** los únicos `[RELLENAR ...]` son en `docs/backlog.md`
  (Task 3 Step 3), explícitamente para cifras que solo existen tras
  ejecutar los Steps 1-2 de esa misma task — mismo patrón ya usado en
  `docs/superpowers/plans/2026-09-19-campana-i-latencia-hook-w11.md:1928`.
  El propio Step 3 instruye sustituirlos antes de commitear. Ningún otro
  paso del plan deja código, comando o test a medias.
- **Consistencia de nombres entre tareas:** `SEARCH_FIRST_SENTINEL_DIR`,
  `search-first-ok`/`search-first`/`search-first-skip`, el matcher
  `^(Agent|Task|Edit|Write|NotebookEdit)$` y la ruta
  `plugins/exo/scripts/search-first.sh` se citan idénticos en las cuatro
  tasks (script, hooks.json, tablas de docs, comandos de medición).
- **Contradicción encontrada y resuelta:** spec §6 caso 4 (`echo "exo
  search"` con comillas) no matchea la regex exacta de §4.2, verificado
  con `jq -n --arg re '...' --arg s '...' '$s|test($re)'` contra jq 1.8.2.
  Documentado en la sección dedicada arriba y en el propio
  `test-search-first.sh` (comentario del caso 4): se mantiene la regex
  verbatim (más cargada de justificación) y se ajusta el fixture del test
  a `echo exo search` (sin comillas), que demuestra el mismo punto de
  §4.3.
- **Drift pre-existente en `docs/arquitectura.md` (no introducido por este
  plan, corregido de paso porque la Task 2 ya edita el mismo párrafo):**
  antes de esta task, la línea 39 («9 skills... y 9 hooks») ya
  sobrecontaba en 1 los 8 comandos reales de `hooks.json`, y la línea 381
  («10 comandos cableados») sobrecontaba en 2. Tras la Task 2 (8→9
  comandos reales), la línea 39 queda correcta por coincidencia (no se
  toca); la línea 381 si se corrige (10→9) porque cae en el mismo `Edit`
  que ya toca ese párrafo para la mención del hook nuevo.
- **`docs/instalacion.md`:** no se toca — no lleva tabla de hooks ni cita
  a `search-first`; confirmado que `test-docs-vivos.sh` solo exige la
  tabla en `README.md` y `plugins/exo/README.md`.
