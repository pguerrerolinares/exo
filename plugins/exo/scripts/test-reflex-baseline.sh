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

OUT="$(REFLEX_LOG_FILE="$TMPLOG" bash "$SCRIPT" 2>&1)"

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

# --- Mutación: si una tubería de métricas se rompe, el script debe fallar
# (exit != 0) y avisar por algún canal visible. Regresión permanente para el
# bug de fallo silencioso (jq roto pero exit 0). ---
MUTSCRIPT="$(mktemp)"
trap 'rm -f "$TMPLOG" "$MUTSCRIPT"' EXIT
# Rompe únicamente la 2ª jq del primer bloque de métricas ("Disparos por
# reflejo"): group_by -> BROKEN_group_by. El patrón buscado sólo aparece en
# esa línea (las otras dos tuberías parten group_by(.reflex)[] en varias
# líneas), así que el sed es específico a propósito.
sed "s#'group_by(\.reflex)\[\] | \"#'BROKEN_group_by(.reflex)[] | \"#" "$SCRIPT" > "$MUTSCRIPT"
if ! grep -q 'BROKEN_group_by' "$MUTSCRIPT"; then
  printf '[FAIL] mutación no se aplicó (patrón no encontrado en %s) — el sed de este test ya no matchea el script real\n' "$SCRIPT"
  FAIL=$((FAIL+1))
else
  MUTOUT="$(REFLEX_LOG_FILE="$TMPLOG" bash "$MUTSCRIPT" 2>&1)"; MUTRC=$?
  if [ "$MUTRC" -ne 0 ] && printf '%s' "$MUTOUT" | grep -qi 'error'; then
    printf '[PASS] tubería rota detectada: exit %d + mensaje de error visible\n' "$MUTRC"
    PASS=$((PASS+1))
  else
    printf '[FAIL] tubería rota NO detectada (exit=%d, se esperaba != 0 y un mensaje de error)\n' "$MUTRC"
    FAIL=$((FAIL+1))
  fi
fi

TOTAL=$((PASS+FAIL)); echo "=== ${PASS}/${TOTAL} ==="
[ $FAIL -eq 0 ] && exit 0 || exit 1
