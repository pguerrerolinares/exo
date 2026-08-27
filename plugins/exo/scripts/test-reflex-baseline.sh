#!/usr/bin/env bash
# Test standalone para reflex-baseline.sh. Genera un log sintético temporal,
# corre el script apuntando a él, y verifica los conteos (excluyendo test).
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="${SCRIPT_DIR}/reflex-baseline.sh"
PASS=0; FAIL=0
TMPLOG="$(mktemp)"
trap 'rm -f "$TMPLOG"' EXIT

# 3 disparos git-c en sesión s1 (reincidencia), 1 en s2; 1 zero-residuo subagente;
# 1 línea de sesión test (debe excluirse); 1 con payload LIVE-TEST (debe excluirse).
cat > "$TMPLOG" <<'EOF'
{"ts":"t","reflex":"git-c","session_id":"s1","agent_id":"","agent_type":"","tool":"Bash","payload":"cd a && git status"}
{"ts":"t","reflex":"git-c","session_id":"s1","agent_id":"sub1","agent_type":"general-purpose","tool":"Bash","payload":"cd b && git log"}
{"ts":"t","reflex":"git-c","session_id":"s1","agent_id":"sub1","agent_type":"general-purpose","tool":"Bash","payload":"cd c && git diff"}
{"ts":"t","reflex":"git-c","session_id":"s2","agent_id":"","agent_type":"","tool":"Bash","payload":"cd d && git show"}
{"ts":"t","reflex":"zero-residuo","session_id":"s2","agent_id":"sub9","agent_type":"general-purpose","tool":"Bash","payload":"git add -A"}
{"ts":"t","reflex":"git-c","session_id":"test-sid","agent_id":"","agent_type":"","tool":"Bash","payload":"cd x && git status"}
{"ts":"t","reflex":"stuck-loop","session_id":"s3","agent_id":"","agent_type":"","tool":"Bash","payload":"STUCK-LOOP-LIVE-TEST"}
EOF

OUT="$(REFLEX_LOG_FILE="$TMPLOG" bash "$SCRIPT" 2>/dev/null)"

check() { # name, pattern
  if printf '%s' "$OUT" | grep -Eq "$2"; then printf '[PASS] %s\n' "$1"; PASS=$((PASS+1))
  else printf '[FAIL] %s — no matcheó /%s/\n' "$1" "$2"; FAIL=$((FAIL+1)); fi
}

# git-c real = 4 (excluye la sesión test); zero-residuo = 1; reincidencia git-c en s1 = 3.
check "git-c cuenta 4"            'git-c[[:space:]]+4'
check "zero-residuo cuenta 1"     'zero-residuo[[:space:]]+1'

# Assert negativo real: ninguna sesión de test debe aparecer en el output.
if printf '%s' "$OUT" | grep -q 'test-sid'; then
  printf '[FAIL] excluye sesión test — test-sid apareció en el output\n'; FAIL=$((FAIL+1))
else printf '[PASS] excluye sesión test\n'; PASS=$((PASS+1)); fi

check "reincidencia git-c = 3"    'git-c.*3'

# stuck-loop NO debe aparecer (su única línea es LIVE-TEST).
if printf '%s' "$OUT" | grep -Eq 'stuck-loop[[:space:]]+[1-9]'; then
  printf '[FAIL] stuck-loop apareció pese a ser LIVE-TEST\n'; FAIL=$((FAIL+1))
else printf '[PASS] stuck-loop excluido\n'; PASS=$((PASS+1)); fi

TOTAL=$((PASS+FAIL)); echo "=== ${PASS}/${TOTAL} ==="
[ $FAIL -eq 0 ] && exit 0 || exit 1
