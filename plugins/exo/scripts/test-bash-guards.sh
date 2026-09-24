#!/usr/bin/env bash
# Test standalone para bash-guards.sh (Task 5, campaña I: fusión de los
# tres guards de PreToolUse:Bash en un solo script).
#
# Porta TODOS los casos de test-git-c-bash.sh, test-git-add-all-guard.sh y
# test-verify-before-commit.sh contra bash-guards.sh (mismo veredicto y
# mismo log que los tres originales -- que se preservan intactos como red
# de regresión), más casos nuevos de combinación (Sección D/E) que solo
# tienen sentido con el script fundido: el pre-filtro/extracción
# compartidos, y comandos que disparan 2-3 ramas a la vez.
#
# REFLEX_LOG_FILE apunta a un tmpfile: los tests NUNCA ensucian
# ~/.claude/reflex-log.jsonl. Todas las aserciones de log son
# DIFERENCIALES (before/after wc -l), así que un único log compartido para
# todo el fichero es seguro pese a estar interseccionado entre secciones
# portadas de las tres suites distintas.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/bash-guards.sh"

REFLEX_LOG_FILE="$(mktemp)"
export REFLEX_LOG_FILE
trap 'rm -f "$REFLEX_LOG_FILE"' EXIT

PASS=0
FAIL=0

# ---------------------------------------------------------------------------
# Helpers compartidos
# ---------------------------------------------------------------------------

# Payload PreToolUse con command (+ description/timeout opcionales para el
# caso de preservación de git-c). Saltos de línea reales -> \n literal.
make_payload() {
  local cmd="$1"
  local cmd_escaped
  cmd_escaped="$(printf '%s' "$cmd" | sed 's/\\/\\\\/g; s/"/\\"/g' | awk 'NR>1{printf "\\n"} {printf "%s", $0}')"
  printf '{"session_id":"test-sid","tool_name":"Bash","tool_input":{"command":"%s","description":"desc-original","timeout":5000},"hook_event_name":"PreToolUse"}' \
    "$cmd_escaped"
}

# Payload PreToolUse con cwd y transcript_path opcionales (para verify-before-done).
make_payload_full() {
  local cwd="$1" transcript="$2" cmd="$3"
  local cmd_escaped
  cmd_escaped="$(printf '%s' "$cmd" | sed 's/\\/\\\\/g; s/"/\\"/g' | awk 'NR>1{printf "\\n"} {printf "%s", $0}')"
  if [ -n "$transcript" ]; then
    printf '{"session_id":"test-sid","cwd":"%s","transcript_path":"%s","tool_name":"Bash","tool_input":{"command":"%s"},"hook_event_name":"PreToolUse"}' \
      "$cwd" "$transcript" "$cmd_escaped"
  else
    printf '{"session_id":"test-sid","cwd":"%s","tool_name":"Bash","tool_input":{"command":"%s"},"hook_event_name":"PreToolUse"}' \
      "$cwd" "$cmd_escaped"
  fi
}

assert_rewrite() {
  local name="$1" cmd="$2" expected_new="$3"
  local output new decision
  output="$(make_payload "$cmd" | bash "$HOOK" 2>/dev/null)"
  new="$(printf '%s' "$output" | jq -r '.hookSpecificOutput.updatedInput.command // empty' 2>/dev/null)"
  decision="$(printf '%s' "$output" | jq -r '.hookSpecificOutput.permissionDecision // empty' 2>/dev/null)"
  if [ "$new" = "$expected_new" ] && [ "$decision" = "allow" ]; then
    printf '[PASS] %s\n' "$name"; PASS=$((PASS+1))
  else
    printf '[FAIL] %s — esperaba rewrite a «%s»/allow, obtuve command=«%s» decision=«%s»\n' \
      "$name" "$expected_new" "$new" "$decision"
    FAIL=$((FAIL+1))
  fi
}

# log-only genérico: stdout vacío + al menos una línea nueva de log con el
# reflex dado ENTRE las líneas añadidas por esta invocación. No exige que
# sea la ÚNICA línea añadida ni la última: algunos comandos portados de
# git-c-bash.sh (p.ej. "git add -A") también casan con el PATRON de otra
# rama (zero-residuo) -- eso es fusión correcta, no un bug; lo que esta
# aserción verifica es que LA rama bajo prueba sí disparó.
assert_log_only() {
  local name="$1" cmd="$2" reflex="$3"
  local before after output nuevas
  before="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  output="$(make_payload "$cmd" | bash "$HOOK" 2>/dev/null)"
  after="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  # "N,$p" con la direccion final como bloque `{p}` -- en vez del `N,$p`
  # sin llaves -- solo por casualidad imprime lo mismo en GNU sed (trata
  # `{p}` como un grupo de un solo comando tras la direccion `N,$`); no es
  # el idioma portable estandar y en macOS (bash-guards, CI 35977909222)
  # dejaba `nuevas` vacio en las 18 aserciones que pasan por aqui, sin
  # tocar sed en absoluto: `tail -n +N` es POSIX, identico en GNU
  # coreutils y en el tail de macOS, y no depende de como cada sed
  # interprete una direccion de rango seguida de un bloque.
  nuevas="$(tail -n "+$((before+1))" "$REFLEX_LOG_FILE" 2>/dev/null)"
  if [ -z "$output" ] && [ "$after" -gt "$before" ] \
     && printf '%s\n' "$nuevas" | jq -e --arg r "$reflex" 'select(.reflex==$r)' >/dev/null 2>&1; then
    printf '[PASS] %s\n' "$name"; PASS=$((PASS+1))
  else
    printf '[FAIL] %s — esperaba log-only (reflex=%s, línea log + stdout vacío). output=%s nuevas=%s\n' "$name" "$reflex" "$output" "$nuevas"
    FAIL=$((FAIL+1))
  fi
}

assert_silent() {
  local name="$1" cmd="$2"
  local before after output
  before="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  output="$(make_payload "$cmd" | bash "$HOOK" 2>/dev/null)"
  after="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  if [ -z "$output" ] && [ "$after" -eq "$before" ]; then
    printf '[PASS] %s\n' "$name"; PASS=$((PASS+1))
  else
    printf '[FAIL] %s — esperaba silencio total (sin log), output: %s (before=%s after=%s)\n' "$name" "$output" "$before" "$after"
    FAIL=$((FAIL+1))
  fi
}

echo "=== test-bash-guards.sh ==="
echo ""
echo "--- Sección A: casos de git-c-bash.sh (rama 1) ---"

assert_rewrite "cd && git status → rewrite" \
  "cd /repo && git status" \
  "git -C /repo status"
assert_rewrite "cd && git log con flags → rewrite" \
  "cd /opt/proyectos/code-graph-go && git log --oneline -5" \
  "git -C /opt/proyectos/code-graph-go log --oneline -5"
assert_rewrite "path con ~ y . → rewrite" \
  "cd ~/proyectos/x.y && git diff --stat" \
  "git -C ~/proyectos/x.y diff --stat"
assert_rewrite "path relativo simple (sin ..) → rewrite" \
  "cd subdir && git status" \
  "git -C subdir status"

{
  OUTPUT="$(make_payload "cd /repo && git status" | bash "$HOOK" 2>/dev/null)"
  DESC="$(printf '%s' "$OUTPUT" | jq -r '.hookSpecificOutput.updatedInput.description // empty')"
  TMO="$(printf '%s' "$OUTPUT" | jq -r '.hookSpecificOutput.updatedInput.timeout // empty')"
  if [ "$DESC" = "desc-original" ] && [ "$TMO" = "5000" ]; then
    printf '[PASS] updatedInput preserva description y timeout\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] updatedInput perdió campos: description=«%s» timeout=«%s»\n' "$DESC" "$TMO"
    FAIL=$((FAIL+1))
  fi
}

{
  : > "$REFLEX_LOG_FILE"
  make_payload "cd /repo && git status" | bash "$HOOK" >/dev/null 2>&1
  if jq -e 'select(.reflex=="git-c-rewrite")' "$REFLEX_LOG_FILE" >/dev/null 2>&1; then
    printf '[PASS] rewrite loguea como git-c-rewrite\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] no hay entrada git-c-rewrite en el log de test\n'; FAIL=$((FAIL+1))
  fi
}

assert_log_only "git push (mutante, fuera de allowlist) → log-only" "cd /repo && git push" "git-c"
assert_log_only "git add (mutante) → log-only" "cd /repo && git add -A" "git-c"
assert_log_only "git branch (bare puede crear) → log-only" "cd /repo && git branch foo" "git-c"
assert_log_only "pipe en REST → log-only" "cd /repo && git log --oneline | head -5" "git-c"
assert_log_only "chain extra tras git → log-only" "cd /repo && git status && echo done" "git-c"
assert_log_only "redirect en REST → log-only" "cd /repo && git status 2>/dev/null" "git-c"
# shellcheck disable=SC2016 # el test pasa el `$DIR` literal, sin expandir
assert_log_only "path con variable → log-only" 'cd "$DIR" && git status' "git-c"
assert_log_only "separador ; → log-only (v1 solo &&)" "cd /repo; git status" "git-c"
assert_log_only "patrón como dato en echo → log-only (FP conocido, NO rewrite)" "echo 'cd /x && git status'" "git-c"
assert_log_only "glob en REST → log-only" "cd /repo && git ls-files *.md" "git-c"
assert_log_only "cd - (OLDPWD) → log-only" "cd - && git status" "git-c"
assert_log_only "componente .. relativo → log-only" "cd ../sub && git status" "git-c"
assert_log_only "componente .. embebido → log-only" "cd /repo/../otro && git log --oneline" "git-c"
assert_log_only "subcomando fuera del allowlist read-only → log-only" "cd /tmp && git branch tmpbranch" "git-c"

{
  BEFORE="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  OUTPUT="$(make_payload "cd subdir && git status" | CDPATH="/tmp" bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  if [ -z "$OUTPUT" ] && [ "$AFTER" -gt "$BEFORE" ] \
     && tail -1 "$REFLEX_LOG_FILE" | jq -e 'select(.reflex=="git-c")' >/dev/null 2>&1; then
    printf '[PASS] CDPATH + path relativo → log-only, sin rewrite\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] CDPATH + path relativo — esperaba log-only sin rewrite, output: %s\n' "$OUTPUT"
    FAIL=$((FAIL+1))
  fi
}

assert_silent "git -C ya correcto → silencio" "git -C /repo status"
assert_silent "cd && make && git → silencio (regex no cruza separadores)" "cd /repo && make && git status"
assert_silent "sin git → silencio" "ls -la /repo"

{
  OUTPUT="$(make_payload "$(printf 'cd /repo &&\ngit status')" | bash "$HOOK" 2>/dev/null)"
  if printf '%s' "$OUTPUT" | jq -e '.hookSpecificOutput.updatedInput' >/dev/null 2>&1; then
    printf '[FAIL] multiline produjo rewrite (prohibido)\n'; FAIL=$((FAIL+1))
  else
    printf '[PASS] multiline → sin rewrite\n'; PASS=$((PASS+1))
  fi
}

{
  PAYLOAD='{"session_id":"test-sid","tool_name":"Bash","tool_input":{"command":""},"hook_event_name":"PreToolUse"}'
  OUTPUT="$(printf '%s' "$PAYLOAD" | bash "$HOOK" 2>/dev/null)"
  EC=$?
  if [ $EC -eq 0 ] && [ -z "$OUTPUT" ]; then
    printf '[PASS] command vacío → exit 0, sin output\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] command vacío → ec=%d output=%s\n' "$EC" "$OUTPUT"; FAIL=$((FAIL+1))
  fi
}

echo ""
echo "--- Sección B: casos de git-add-all-guard.sh (rama 2) ---"

assert_log_only "git add -A → nudge"                "git add -A" "zero-residuo"
assert_log_only "git add --all → nudge"             "git add --all" "zero-residuo"
assert_log_only "git add . → nudge"                 "git add ." "zero-residuo"
assert_log_only "git -C /tmp/x add -A → nudge (fix)" "git -C /tmp/x add -A" "zero-residuo"
assert_silent   "git add foo.txt bar.py → no nudge" "git add foo.txt bar.py"

# "git commit -m x" también casa con el PATRON de verify-before-done (rama
# 3), así que sin aislar el cwd este caso queda a merced del staging area
# REAL del proceso que corre el test (falso positivo/negativo según qué
# haya staged en ese momento en el repo del propio exo). Se aísla con un
# repo temporal vacío (nada staged) para que rama 3 tampoco tenga con qué
# disparar, y así el caso sigue probando lo que probaba en el original:
# zero-residuo no dispara ante un commit sin "add -A/--all/.".
{
  REPO_AISLADO="$(mktemp -d)"
  git -C "$REPO_AISLADO" init -q
  PAYLOAD_JSON="$(make_payload_full "$REPO_AISLADO" "" "git commit -m x")"
  BEFORE="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD_JSON" | bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  rm -rf "$REPO_AISLADO"
  if [ -z "$OUTPUT" ] && [ "$AFTER" -eq "$BEFORE" ]; then
    printf '[PASS] git commit -m x (repo vacío, aislado) → no nudge de zero-residuo\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] git commit -m x (repo vacío) — esperaba silencio total, output=%s (before=%s after=%s)\n' "$OUTPUT" "$BEFORE" "$AFTER"
    FAIL=$((FAIL+1))
  fi
}

# caso heredoc pequeño (650 chars, 5 líneas)
{
  LINE="$(head -c 130 < /dev/zero | tr '\0' 'x')"
  HEREDOC_BODY="$(printf '%s\n%s\n%s\n%s\n%s' "$LINE" "$LINE" "$LINE" "$LINE" "$LINE")"
  CMD="$(printf 'cat <<EOF\n%s\nEOF\ngit add -A' "$HEREDOC_BODY")"
  : > "$REFLEX_LOG_FILE"
  make_payload "$CMD" | bash "$HOOK" >/dev/null 2>&1
  PAYLOAD="$(tail -1 "$REFLEX_LOG_FILE" | jq -r 'select(.reflex=="zero-residuo") | .payload' 2>/dev/null)"
  if printf '%s' "$PAYLOAD" | grep -q 'git add -A'; then
    printf '[PASS] heredoc 650 chars + git add -A → el payload conserva el match\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] heredoc 650 chars + git add -A → el match no aparece en el payload. payload=%s\n' "$PAYLOAD"
    FAIL=$((FAIL+1))
  fi
}

# caso heredoc grande (~4.5 KB, 42 líneas)
{
  HEREDOC_BODY=""
  for i in $(seq -w 0 41); do
    LINEA="linea ${i}: $(head -c 100 < /dev/zero | tr '\0' 'x')"
    if [ -z "$HEREDOC_BODY" ]; then HEREDOC_BODY="$LINEA"; else HEREDOC_BODY="$(printf '%s\n%s' "$HEREDOC_BODY" "$LINEA")"; fi
  done
  CMD="$(printf "cat > spec.md <<'EOF'\n%s\nEOF\ngit add -A" "$HEREDOC_BODY")"
  : > "$REFLEX_LOG_FILE"
  make_payload "$CMD" | bash "$HOOK" >/dev/null 2>&1
  PAYLOAD="$(tail -1 "$REFLEX_LOG_FILE" | jq -r 'select(.reflex=="zero-residuo") | .payload' 2>/dev/null)"
  PAYLOAD_LEN="$(printf '%s' "$PAYLOAD" | wc -c)"
  if [ "${#CMD}" -lt 4000 ]; then
    printf '[FAIL] heredoc grande → el comando de prueba mide %d chars, no llega a los 4 KB del plan\n' "${#CMD}"
    FAIL=$((FAIL+1))
  elif printf '%s' "$PAYLOAD" | grep -q 'git add -A'; then
    printf '[PASS] heredoc ~4.5KB/42 líneas + git add -A → payload (%d bytes) conserva el match\n' "$PAYLOAD_LEN"
    PASS=$((PASS+1))
  else
    printf '[FAIL] heredoc ~4.5KB/42 líneas + git add -A → el match no aparece. payload=%s\n' "$PAYLOAD"
    FAIL=$((FAIL+1))
  fi
}

# caso match sin techo (path de 3000 chars tras -C)
{
  PATH_LARGO="$(head -c 3000 < /dev/zero | tr '\0' 'x')"
  CMD="git -C ${PATH_LARGO} add -A"
  : > "$REFLEX_LOG_FILE"
  make_payload "$CMD" | bash "$HOOK" >/dev/null 2>&1
  PAYLOAD="$(tail -1 "$REFLEX_LOG_FILE" | jq -r 'select(.reflex=="zero-residuo") | .payload' 2>/dev/null)"
  PAYLOAD_LEN="$(printf '%s' "$PAYLOAD" | wc -c)"
  if [ "${#CMD}" -lt 2500 ]; then
    printf '[FAIL] match sin techo → comando de prueba mide %d chars, no llega a 2500\n' "${#CMD}"
    FAIL=$((FAIL+1))
  elif printf '%s' "$PAYLOAD" | grep -q 'add -A' && [ "$PAYLOAD_LEN" -lt 2000 ]; then
    printf '[PASS] match sin techo (path 3000 chars) → payload (%d bytes) conserva "add -A"\n' "$PAYLOAD_LEN"
    PASS=$((PASS+1))
  else
    printf '[FAIL] match sin techo → "add -A" no aparece o payload no cabe. payload=%s\n' "$PAYLOAD"
    FAIL=$((FAIL+1))
  fi
}

# comando corto → sin marcador
{
  CMD="git add -A"
  : > "$REFLEX_LOG_FILE"
  make_payload "$CMD" | bash "$HOOK" >/dev/null 2>&1
  PAYLOAD="$(tail -1 "$REFLEX_LOG_FILE" | jq -r 'select(.reflex=="zero-residuo") | .payload' 2>/dev/null)"
  if [ "$PAYLOAD" = "$CMD" ]; then
    printf '[PASS] comando corto → payload = comando entero, sin marcador\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] comando corto → esperaba "%s", obtuve "%s"\n' "$CMD" "$PAYLOAD"; FAIL=$((FAIL+1))
  fi
}

# dos ocurrencias distintas (prosa + real al final)
{
  CMD="$(printf "cat > docs/normas-del-repo.md <<'EOF'\nNorma 1: nunca uses git add . en este repositorio porque arrastra residuo.\nNorma 2: usa git add con rutas explicitas.\nEOF\ngit add -A")"
  : > "$REFLEX_LOG_FILE"
  make_payload "$CMD" | bash "$HOOK" >/dev/null 2>&1
  PAYLOAD="$(tail -1 "$REFLEX_LOG_FILE" | jq -r 'select(.reflex=="zero-residuo") | .payload' 2>/dev/null)"
  if printf '%s' "$PAYLOAD" | grep -q 'add -A'; then
    printf '[PASS] dos ocurrencias distintas (prosa + real al final) → el payload conserva "add -A"\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] dos ocurrencias distintas → "add -A" no aparece. payload=%s\n' "$PAYLOAD"; FAIL=$((FAIL+1))
  fi
}

# ocurrencias idénticas repetidas → dedup
{
  RELLENO="$(head -c 150 < /dev/zero | tr '\0' 'x')"
  CMD="echo ${RELLENO} ; git add . ; echo nope ; git add . ; true"
  : > "$REFLEX_LOG_FILE"
  make_payload "$CMD" | bash "$HOOK" >/dev/null 2>&1
  PAYLOAD="$(tail -1 "$REFLEX_LOG_FILE" | jq -r 'select(.reflex=="zero-residuo") | .payload' 2>/dev/null)"
  MATCH_SEGMENT="$(printf '%s' "$PAYLOAD" | sed 's/.*⟨match⟩ //')"
  OCURRENCIAS="$(printf '%s' "$MATCH_SEGMENT" | grep -o 'git add \.' | wc -l)"
  if [ "$OCURRENCIAS" -eq 1 ]; then
    printf '[PASS] ocurrencias idénticas repetidas → dedup (aparece una sola vez)\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] ocurrencias idénticas repetidas → esperaba 1, hubo %s. payload=%s\n' "$OCURRENCIAS" "$PAYLOAD"
    FAIL=$((FAIL+1))
  fi
}

echo ""
echo "--- Sección C: casos de verify-before-commit.sh (rama 3) ---"

TMPDIR_BASE="/tmp/claude-test-bashguards-$$"
mkdir -p "$TMPDIR_BASE"
TMP_REPOS=()

make_repo() {
  local name="$1"; shift
  local repo="${TMPDIR_BASE}/${name}"
  mkdir -p "$repo"
  git -C "$repo" init -q
  local spec f content
  for spec in "$@"; do
    f="${spec%%:*}"
    content="${spec#*:}"
    printf '%s\n' "$content" > "${repo}/${f}"
    git -C "$repo" add "$f"
  done
  TMP_REPOS+=("$repo")
  printf '%s' "$repo"
}

transcript_tool_use() {
  local id="$1" cmd="$2"
  local cmd_escaped
  cmd_escaped="$(printf '%s' "$cmd" | sed 's/\\/\\\\/g; s/"/\\"/g')"
  printf '{"type":"assistant","message":{"content":[{"type":"tool_use","id":"%s","name":"Bash","input":{"command":"%s"}}]}}\n' \
    "$id" "$cmd_escaped"
}
transcript_tool_result() {
  local tool_use_id="$1" is_error="$2"
  printf '{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"%s","is_error":%s}]}}\n' \
    "$tool_use_id" "$is_error"
}

assert_vbd_logged() {
  local name="$1" before="$2" after="$3" output="$4"
  if [ -z "$output" ] && [ "$after" -gt "$before" ] \
     && tail -1 "$REFLEX_LOG_FILE" | jq -e 'select(.reflex=="verify-before-done")' >/dev/null 2>&1; then
    printf '[PASS] %s\n' "$name"; PASS=$((PASS+1))
  else
    printf '[FAIL] %s — esperaba log-only (verify-before-done). output=%s\n' "$name" "$output"
    FAIL=$((FAIL+1))
  fi
}
assert_vbd_silent() {
  local name="$1" before="$2" after="$3" output="$4"
  if [ -z "$output" ] && [ "$after" -eq "$before" ]; then
    printf '[PASS] %s\n' "$name"; PASS=$((PASS+1))
  else
    printf '[FAIL] %s — esperaba silencio (before=%s after=%s). output=%s\n' "$name" "$before" "$after" "$output"
    FAIL=$((FAIL+1))
  fi
}

# CASO 1: .py staged + test verde reciente → silencio
{
  REPO="$(make_repo caso1 "foo.py:print('hi')")"
  T="${TMPDIR_BASE}/t1.jsonl"
  transcript_tool_use    "t1" "pytest tests/foo.py" > "$T"
  transcript_tool_result "t1" "false"               >> "$T"
  PAYLOAD="$(make_payload_full "$REPO" "$T" "git commit -m fix")"
  BEFORE="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD" | bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  assert_vbd_silent "caso1: .py staged + test verde reciente → silencio" "$BEFORE" "$AFTER" "$OUTPUT"
}

# CASO 2: .py staged + test rojo reciente → nudge
{
  REPO="$(make_repo caso2 "foo.py:print('hi')")"
  T="${TMPDIR_BASE}/t2.jsonl"
  transcript_tool_use    "t1" "pytest tests/foo.py" > "$T"
  transcript_tool_result "t1" "true"                >> "$T"
  PAYLOAD="$(make_payload_full "$REPO" "$T" "git commit -m fix")"
  BEFORE="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD" | bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  assert_vbd_logged "caso2: .py staged + test rojo reciente → nudge" "$BEFORE" "$AFTER" "$OUTPUT"
}

# CASO 3: .py staged + sin test-runner en transcript → nudge
{
  REPO="$(make_repo caso3 "foo.py:print('hi')")"
  T="${TMPDIR_BASE}/t3.jsonl"
  transcript_tool_use    "t1" "ls -la"  > "$T"
  transcript_tool_result "t1" "false"   >> "$T"
  PAYLOAD="$(make_payload_full "$REPO" "$T" "git commit -m fix")"
  BEFORE="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD" | bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  assert_vbd_logged "caso3: .py staged + sin test-runner → nudge" "$BEFORE" "$AFTER" "$OUTPUT"
}

# CASO 4: .py staged + transcript_path ausente → nudge
{
  REPO="$(make_repo caso4 "foo.py:print('hi')")"
  PAYLOAD="$(make_payload_full "$REPO" "" "git commit -m fix")"
  BEFORE="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD" | bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  assert_vbd_logged "caso4: .py staged + transcript ausente → nudge" "$BEFORE" "$AFTER" "$OUTPUT"
}

# CASO 5: solo docs (.md) staged + sin test → silencio (filtro código-vs-docs)
{
  REPO="$(make_repo caso5 "README.md:hola")"
  T="${TMPDIR_BASE}/t5.jsonl"
  transcript_tool_use    "t1" "ls -la"  > "$T"
  transcript_tool_result "t1" "false"   >> "$T"
  PAYLOAD="$(make_payload_full "$REPO" "$T" "git commit -m docs")"
  BEFORE="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD" | bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  assert_vbd_silent "caso5: solo docs staged → silencio (filtro código-vs-docs)" "$BEFORE" "$AFTER" "$OUTPUT"
}

# CASO 6: --no-verify con código staged → silencio (escape hatch)
{
  REPO="$(make_repo caso6 "foo.py:print('hi')")"
  T="${TMPDIR_BASE}/t6.jsonl"
  transcript_tool_use    "t1" "ls -la"  > "$T"
  transcript_tool_result "t1" "false"   >> "$T"
  PAYLOAD="$(make_payload_full "$REPO" "$T" "git commit --no-verify -m wip")"
  BEFORE="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD" | bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  assert_vbd_silent "caso6: --no-verify → silencio (escape hatch)" "$BEFORE" "$AFTER" "$OUTPUT"
}

# CASO 7: nada staged → silencio
{
  REPO="${TMPDIR_BASE}/caso7"
  mkdir -p "$REPO"; git -C "$REPO" init -q; TMP_REPOS+=("$REPO")
  T="${TMPDIR_BASE}/t7.jsonl"
  transcript_tool_use    "t1" "ls -la"  > "$T"
  transcript_tool_result "t1" "false"   >> "$T"
  PAYLOAD="$(make_payload_full "$REPO" "$T" "git commit -m empty")"
  BEFORE="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD" | bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  assert_vbd_silent "caso7: nada staged → silencio" "$BEFORE" "$AFTER" "$OUTPUT"
}

# CASO 8: heredoc pequeño delante del "git commit" real
{
  REPO="$(make_repo caso8 "foo.py:print('hi')")"
  LINE="$(head -c 130 < /dev/zero | tr '\0' 'x')"
  HEREDOC_BODY="$(printf '%s\n%s\n%s\n%s\n%s' "$LINE" "$LINE" "$LINE" "$LINE" "$LINE")"
  CMD="$(printf 'cat <<EOF\n%s\nEOF\ngit commit -m fix' "$HEREDOC_BODY")"
  PAYLOAD_JSON="$(make_payload_full "$REPO" "" "$CMD")"
  BEFORE="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD_JSON" | bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  LOGGED="$(tail -1 "$REFLEX_LOG_FILE" | jq -r 'select(.reflex=="verify-before-done") | .payload' 2>/dev/null)"
  if [ -z "$OUTPUT" ] && [ "$AFTER" -gt "$BEFORE" ] && printf '%s' "$LOGGED" | grep -q 'git commit'; then
    printf '[PASS] caso8: heredoc 650 chars + git commit → el payload conserva el match\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] caso8: heredoc 650 chars + git commit → no conserva el match (before=%s after=%s). payload=%s\n' "$BEFORE" "$AFTER" "$LOGGED"
    FAIL=$((FAIL+1))
  fi
}

# CASO 8b: heredoc grande, ~4.5 KB / 42 líneas
{
  REPO="$(make_repo caso8b "foo.py:print('hi')")"
  HEREDOC_BODY=""
  for i in $(seq -w 0 41); do
    LINEA="linea ${i}: $(head -c 100 < /dev/zero | tr '\0' 'x')"
    if [ -z "$HEREDOC_BODY" ]; then HEREDOC_BODY="$LINEA"; else HEREDOC_BODY="$(printf '%s\n%s' "$HEREDOC_BODY" "$LINEA")"; fi
  done
  CMD="$(printf "cat > spec.md <<'EOF'\n%s\nEOF\ngit commit -m fix" "$HEREDOC_BODY")"
  PAYLOAD_JSON="$(make_payload_full "$REPO" "" "$CMD")"
  BEFORE="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD_JSON" | bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  LOGGED="$(tail -1 "$REFLEX_LOG_FILE" | jq -r 'select(.reflex=="verify-before-done") | .payload' 2>/dev/null)"
  PAYLOAD_LEN="$(printf '%s' "$LOGGED" | wc -c)"
  if [ "${#CMD}" -lt 4000 ]; then
    printf '[FAIL] caso8b: comando de prueba mide %d chars, no llega a 4 KB\n' "${#CMD}"; FAIL=$((FAIL+1))
  elif [ -z "$OUTPUT" ] && [ "$AFTER" -gt "$BEFORE" ] && printf '%s' "$LOGGED" | grep -q 'git commit'; then
    printf '[PASS] caso8b: heredoc ~4.5KB/42 líneas + git commit → payload (%d bytes) conserva el match\n' "$PAYLOAD_LEN"
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso8b: no conserva el match (before=%s after=%s). payload=%s\n' "$BEFORE" "$AFTER" "$LOGGED"
    FAIL=$((FAIL+1))
  fi
}

# CASO 8c: match sin techo (3000 espacios entre git y commit)
{
  REPO="$(make_repo caso8c "foo.py:print('hi')")"
  ESPACIOS="$(head -c 3000 < /dev/zero | tr '\0' ' ')"
  CMD="git${ESPACIOS}commit -m fix"
  PAYLOAD_JSON="$(make_payload_full "$REPO" "" "$CMD")"
  BEFORE="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD_JSON" | bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  LOGGED="$(tail -1 "$REFLEX_LOG_FILE" | jq -r 'select(.reflex=="verify-before-done") | .payload' 2>/dev/null)"
  PAYLOAD_LEN="$(printf '%s' "$LOGGED" | wc -c)"
  if [ "${#CMD}" -lt 2500 ]; then
    printf '[FAIL] caso8c: comando de prueba mide %d chars, no llega a 2500\n' "${#CMD}"; FAIL=$((FAIL+1))
  elif [ -z "$OUTPUT" ] && [ "$AFTER" -gt "$BEFORE" ] && printf '%s' "$LOGGED" | grep -q 'commit' && [ "$PAYLOAD_LEN" -lt 2000 ]; then
    printf '[PASS] caso8c: match sin techo → payload (%d bytes) conserva "commit"\n' "$PAYLOAD_LEN"; PASS=$((PASS+1))
  else
    printf '[FAIL] caso8c: "commit" no aparece o payload no cabe. payload=%s\n' "$LOGGED"; FAIL=$((FAIL+1))
  fi
}

# CASO 9: comando corto → sin marcador
{
  REPO="$(make_repo caso9 "foo.py:print('hi')")"
  CMD="git commit -m fix"
  PAYLOAD_JSON="$(make_payload_full "$REPO" "" "$CMD")"
  BEFORE="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD_JSON" | bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  LOGGED="$(tail -1 "$REFLEX_LOG_FILE" | jq -r 'select(.reflex=="verify-before-done") | .payload' 2>/dev/null)"
  if [ -z "$OUTPUT" ] && [ "$AFTER" -gt "$BEFORE" ] && [ "$LOGGED" = "$CMD" ]; then
    printf '[PASS] caso9: comando corto → payload = comando entero, sin marcador\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] caso9: esperaba "%s", obtuve "%s"\n' "$CMD" "$LOGGED"; FAIL=$((FAIL+1))
  fi
}

# CASO 10: dos ocurrencias distintas (prosa + real al final)
{
  REPO="$(make_repo caso10 "foo.py:print('hi')")"
  CMD="$(printf "cat > docs/normas.md <<'EOF'\nrecuerda hacer git commit tras cada cambio importante siempre que sea posible en este repo\nEOF\ngit commit")"
  PAYLOAD_JSON="$(make_payload_full "$REPO" "" "$CMD")"
  BEFORE="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD_JSON" | bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  LOGGED="$(tail -1 "$REFLEX_LOG_FILE" | jq -r 'select(.reflex=="verify-before-done") | .payload' 2>/dev/null)"
  if [ -z "$OUTPUT" ] && [ "$AFTER" -gt "$BEFORE" ] && printf '%s' "$LOGGED" | grep -qE 'commit$'; then
    printf '[PASS] caso10: dos ocurrencias distintas → el payload conserva la real\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] caso10: la real no aparece al final. payload=%s\n' "$LOGGED"; FAIL=$((FAIL+1))
  fi
}

# CASO 11: ocurrencias idénticas repetidas → dedup
{
  REPO="$(make_repo caso11 "foo.py:print('hi')")"
  RELLENO="$(head -c 150 < /dev/zero | tr '\0' 'x')"
  CMD="echo ${RELLENO} ; git commit -m x ; git commit -m x"
  PAYLOAD_JSON="$(make_payload_full "$REPO" "" "$CMD")"
  BEFORE="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  OUTPUT="$(printf '%s' "$PAYLOAD_JSON" | bash "$HOOK" 2>/dev/null)"
  AFTER="$(wc -l < "$REFLEX_LOG_FILE" 2>/dev/null || echo 0)"
  LOGGED="$(tail -1 "$REFLEX_LOG_FILE" | jq -r 'select(.reflex=="verify-before-done") | .payload' 2>/dev/null)"
  MATCH_SEGMENT="$(printf '%s' "$LOGGED" | sed 's/.*⟨match⟩ //')"
  OCURRENCIAS="$(printf '%s' "$MATCH_SEGMENT" | grep -o 'git commit ' | wc -l)"
  if [ -z "$OUTPUT" ] && [ "$AFTER" -gt "$BEFORE" ] && [ "$OCURRENCIAS" -eq 1 ]; then
    printf '[PASS] caso11: ocurrencias idénticas repetidas → dedup\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] caso11: esperaba 1, hubo %s. payload=%s\n' "$OCURRENCIAS" "$LOGGED"; FAIL=$((FAIL+1))
  fi
}

rm -rf "$TMPDIR_BASE" 2>/dev/null || true

# ---------------------------------------------------------------------------
# Sección D: pre-filtro y extracción ÚNICOS (solo tiene sentido con el
# script fundido -- antes eran tres pre-filtros/extracciones idénticos,
# uno por script).
# ---------------------------------------------------------------------------
echo ""
echo "--- Sección D: pre-filtro/extracción compartidos ---"

poison_jq_counter() {
  # Crea un jq falso que cuenta invocaciones en $1 (fichero contador,
  # una línea por llamada) y devuelve el PATH del directorio poison.
  local counter="$1"
  local dir; dir="$(mktemp -d)"
  cat > "$dir/jq" <<EOF
#!/usr/bin/env bash
echo x >> "$counter"
exit 1
EOF
  chmod +x "$dir/jq"
  printf '%s' "$dir"
}

{
  COUNTER="$(mktemp -u)"; rm -f "$COUNTER"
  POISON_DIR="$(poison_jq_counter "$COUNTER")"
  PAYLOAD_SIN_GIT='{"session_id":"test-sid","tool_name":"Bash","tool_input":{"command":"ls -la /tmp","description":"desc-original","timeout":5000},"hook_event_name":"PreToolUse"}'
  OUTPUT="$(printf '%s' "$PAYLOAD_SIN_GIT" | PATH="$POISON_DIR:$PATH" bash "$HOOK" 2>/dev/null)"
  EC=$?
  if [ -z "$OUTPUT" ] && [ "$EC" -eq 0 ] && [ ! -f "$COUNTER" ]; then
    printf '[PASS] sin "git" en el JSON crudo → pre-filtro único ahorra el jq compartido (marca ausente)\n'; PASS=$((PASS+1))
  else
    N="$( [ -f "$COUNTER" ] && wc -l < "$COUNTER" || echo 0)"
    printf '[FAIL] sin "git" → esperaba 0 llamadas a jq, hubo %s. ec=%d out=%s\n' "$N" "$EC" "$OUTPUT"
    FAIL=$((FAIL+1))
  fi
  rm -rf "$POISON_DIR" "$COUNTER"
}

{
  COUNTER="$(mktemp -u)"; rm -f "$COUNTER"
  POISON_DIR="$(poison_jq_counter "$COUNTER")"
  PAYLOAD_MAYUS='{"session_id":"test-sid","tool_input":{"command":"GIT STATUS && echo done"},"hook_event_name":"PreToolUse"}'
  OUTPUT="$(printf '%s' "$PAYLOAD_MAYUS" | PATH="$POISON_DIR:$PATH" bash "$HOOK" 2>/dev/null)"
  EC=$?
  if [ -z "$OUTPUT" ] && [ "$EC" -eq 0 ] && [ ! -f "$COUNTER" ]; then
    printf '[PASS] "GIT STATUS" mayúsculas → pre-filtro también ahorra jq, sin regresión\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] "GIT STATUS" mayúsculas — ec=%d out=%s\n' "$EC" "$OUTPUT"; FAIL=$((FAIL+1))
  fi
  rm -rf "$POISON_DIR" "$COUNTER"
}

{
  COUNTER="$(mktemp -u)"; rm -f "$COUNTER"
  POISON_DIR="$(poison_jq_counter "$COUNTER")"
  PAYLOAD_GIT_OTRO_CAMPO='{"session_id":"test-sid-con-git-en-otro-lado","tool_input":{"command":"ls -la /tmp"},"hook_event_name":"PreToolUse"}'
  OUTPUT="$(printf '%s' "$PAYLOAD_GIT_OTRO_CAMPO" | PATH="$POISON_DIR:$PATH" bash "$HOOK" 2>/dev/null)"
  if [ -f "$COUNTER" ]; then
    printf '[PASS] "git" en session_id (no en command) → pre-filtro NO descarta, sigue a jq\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] "git" en session_id no forzó el camino lento\n'; FAIL=$((FAIL+1))
  fi
  rm -rf "$POISON_DIR" "$COUNTER"
}

{
  COUNTER="$(mktemp -u)"; rm -f "$COUNTER"
  POISON_DIR="$(poison_jq_counter "$COUNTER")"
  PAYLOAD_SUBCADENA='{"session_id":"test-sid","tool_input":{"command":"npm run digitize --verbose"},"hook_event_name":"PreToolUse"}'
  OUTPUT="$(printf '%s' "$PAYLOAD_SUBCADENA" | PATH="$POISON_DIR:$PATH" bash "$HOOK" 2>/dev/null)"
  if [ -f "$COUNTER" ]; then
    printf '[PASS] "digitize" (subcadena "git") → pre-filtro NO descarta, cae a jq\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] "digitize" no forzó el camino lento\n'; FAIL=$((FAIL+1))
  fi
  rm -rf "$POISON_DIR" "$COUNTER"
  OUTPUT_REAL="$(printf '%s' "$PAYLOAD_SUBCADENA" | bash "$HOOK" 2>/dev/null)"
  EC_REAL=$?
  if [ -z "$OUTPUT_REAL" ] && [ "$EC_REAL" -eq 0 ]; then
    printf '[PASS] "digitize" → silencio real (sin falso disparo)\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] "digitize" → esperaba silencio, ec=%d out=%s\n' "$EC_REAL" "$OUTPUT_REAL"; FAIL=$((FAIL+1))
  fi
}

# La extracción es UNA sola jq compartida (antes: hasta 4 jq repartidas
# entre los tres scripts para el mismo comando). Con "git status" (no
# dispara ni rewrite -- no empieza por "cd" -- ni zero-residuo ni
# verify-before-done), el ÚNICO jq que debe correr es el de extracción.
{
  COUNTER="$(mktemp -u)"; rm -f "$COUNTER"
  DIR="$(mktemp -d)"
  cat > "$DIR/jq" <<EOF
#!/usr/bin/env bash
echo x >> "$COUNTER"
exec "$(command -v jq)" "\$@"
EOF
  chmod +x "$DIR/jq"
  PAYLOAD='{"session_id":"test-sid","tool_input":{"command":"git status"},"hook_event_name":"PreToolUse","cwd":"/tmp"}'
  OUTPUT="$(printf '%s' "$PAYLOAD" | PATH="$DIR:$PATH" bash "$HOOK" 2>/dev/null)"
  N="$( [ -f "$COUNTER" ] && wc -l < "$COUNTER" || echo 0)"
  N="$(printf '%s' "$N" | tr -d ' ')"
  if [ -z "$OUTPUT" ] && [ "$N" -eq 1 ]; then
    printf '[PASS] "git status" → exactamente 1 invocación de jq (la extracción compartida), no 3-4\n'; PASS=$((PASS+1))
  else
    printf '[FAIL] "git status" → esperaba 1 invocación de jq, hubo %s. output=%s\n' "$N" "$OUTPUT"; FAIL=$((FAIL+1))
  fi
  rm -rf "$DIR" "$COUNTER"
}

# ---------------------------------------------------------------------------
# Sección E: combinaciones -- comandos que disparan 2-3 ramas a la vez.
# Solo tiene sentido con el script fundido: antes eran tres procesos
# independientes, cada uno ajeno a los demás.
# ---------------------------------------------------------------------------
echo ""
echo "--- Sección E: combinación de ramas ---"

TMPDIR_E="/tmp/claude-test-bashguards-combo-$$"
mkdir -p "$TMPDIR_E"

# E1: 3 ramas a la vez -- cd + git add -A + git commit, con código staged
# sin test verde reciente. Debe salir UN solo resultado (stdout vacío,
# porque ninguna de las tres ramas que aplican aquí emite JSON) y las TRES
# entradas de log, en orden.
{
  REPO="${TMPDIR_E}/e1"
  mkdir -p "$REPO"; git -C "$REPO" init -q
  printf 'print(1)\n' > "$REPO/foo.py"
  git -C "$REPO" add foo.py
  CMD="cd ${REPO} && git add -A && git commit -m y"
  PAYLOAD_JSON="$(make_payload_full "$REPO" "" "$CMD")"
  : > "$REFLEX_LOG_FILE"
  OUTPUT="$(printf '%s' "$PAYLOAD_JSON" | bash "$HOOK" 2>/dev/null)"
  EC=$?
  REFLEXES="$(jq -r '.reflex' "$REFLEX_LOG_FILE" 2>/dev/null | tr -d '\r' | tr '\n' ',')"
  if [ -z "$OUTPUT" ] && [ "$EC" -eq 0 ] \
     && [ "$REFLEXES" = "git-c,zero-residuo,verify-before-done," ]; then
    printf '[PASS] E1: cd+add-A+commit (3 ramas) → stdout vacío, log con las tres en orden (%s)\n' "$REFLEXES"
    PASS=$((PASS+1))
  else
    printf '[FAIL] E1: esperaba stdout vacío y log "git-c,zero-residuo,verify-before-done,", obtuve reflexes=%s ec=%d out=%s\n' \
      "$REFLEXES" "$EC" "$OUTPUT"
    FAIL=$((FAIL+1))
  fi
}

# E2: 2 ramas a la vez -- sin "cd X &&" delante, así que git-c (rama 1) no
# aplica; add -A + commit sí disparan zero-residuo Y verify-before-done
# juntas en la MISMA invocación.
{
  REPO="${TMPDIR_E}/e2"
  mkdir -p "$REPO"; git -C "$REPO" init -q
  printf 'print(1)\n' > "$REPO/foo.py"
  git -C "$REPO" add foo.py
  CMD="git add -A && git commit -m y"
  PAYLOAD_JSON="$(make_payload_full "$REPO" "" "$CMD")"
  : > "$REFLEX_LOG_FILE"
  OUTPUT="$(printf '%s' "$PAYLOAD_JSON" | bash "$HOOK" 2>/dev/null)"
  EC=$?
  REFLEXES="$(jq -r '.reflex' "$REFLEX_LOG_FILE" 2>/dev/null | tr -d '\r' | tr '\n' ',')"
  if [ -z "$OUTPUT" ] && [ "$EC" -eq 0 ] \
     && [ "$REFLEXES" = "zero-residuo,verify-before-done," ]; then
    printf '[PASS] E2: add-A+commit sin cd (2 ramas) → stdout vacío, log con las dos en orden (%s)\n' "$REFLEXES"
    PASS=$((PASS+1))
  else
    printf '[FAIL] E2: esperaba log "zero-residuo,verify-before-done,", obtuve reflexes=%s ec=%d out=%s\n' \
      "$REFLEXES" "$EC" "$OUTPUT"
    FAIL=$((FAIL+1))
  fi
}

# E3: rewrite aislado -- por construcción (ver cabecera de bash-guards.sh),
# un rewrite de git-c exige que TODO el comando sea `cd PATH && git
# <verbo-de-solo-lectura>`, así que "add"/"commit" no pueden aparecer en
# ese mismo "git" -- rewrite y (zero-residuo O verify-before-done) son
# mutuamente excluyentes en el MISMO comando. Esta prueba confirma la
# fusión de todos modos: cuando 1a reescribe, el JSON es el ÚNICO output
# (jq -e . valida un solo objeto bien formado) y NINGUNA otra rama deja
# entrada de log -- la salida fusionada no se contamina entre ramas.
{
  : > "$REFLEX_LOG_FILE"
  OUTPUT="$(make_payload "cd /repo && git status" | bash "$HOOK" 2>/dev/null)"
  N_JSON_OBJS="$(printf '%s' "$OUTPUT" | jq -s 'length' 2>/dev/null)"
  VALIDO="$(printf '%s' "$OUTPUT" | jq -e '.hookSpecificOutput.updatedInput.command' >/dev/null 2>&1 && echo si || echo no)"
  REFLEXES="$(jq -r '.reflex' "$REFLEX_LOG_FILE" 2>/dev/null | tr -d '\r' | tr '\n' ',')"
  if [ "$VALIDO" = "si" ] && [ "$N_JSON_OBJS" = "1" ] && [ "$REFLEXES" = "git-c-rewrite," ]; then
    printf '[PASS] E3: rewrite aislado → un solo JSON válido en stdout, log solo con git-c-rewrite (sin zero-residuo/verify-before-done)\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] E3: rewrite aislado → válido=%s n_objs=%s reflexes=%s output=%s\n' "$VALIDO" "$N_JSON_OBJS" "$REFLEXES" "$OUTPUT"
    FAIL=$((FAIL+1))
  fi
}

rm -rf "$TMPDIR_E" 2>/dev/null || true

echo ""
TOTAL=$((PASS+FAIL))
echo "=== Resultado: ${PASS}/${TOTAL} pasaron ==="
[ $FAIL -eq 0 ] && exit 0 || exit 1
