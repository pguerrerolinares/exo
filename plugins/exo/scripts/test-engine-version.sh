#!/usr/bin/env bash
# Test standalone para _engine-version.sh: semver_lt y exo_version_de.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

. "$SCRIPT_DIR/_engine-version.sh"

# ------------------- semver_lt: seis pares, sin spawns --------------------
verifica_lt() {  # $1=a $2=b $3=esperado(0 o 1, como exit code de semver_lt)
  semver_lt "$1" "$2"
  local rc=$?
  if [ "$rc" -eq "$3" ]; then
    pass "semver_lt $1 $2 -> $rc"
  else
    fail "semver_lt $1 $2 -> $rc" "esperaba $3"
  fi
}
verifica_lt "0.1.0" "0.2.0" 0   # menor: cierto
verifica_lt "0.2.0" "0.1.0" 1   # mayor: falso
verifica_lt "0.2.0" "0.2.0" 1   # igual: falso (no es "menor que")
verifica_lt "1.9.0" "1.10.0" 0  # el caso que rompe la comparación como texto
verifica_lt "0.2.0" "0.2.1" 0   # parche decide
verifica_lt "1.0.0" "0.99.99" 1 # el mayor gana aunque los otros campos sean grandes

# ------------------- exo_version_de: stub que imita `exo --version` -------
STUB="$TMP/exo-stub"
cat > "$STUB" <<'EOF'
#!/usr/bin/env bash
echo "exo 0.1.0"
EOF
chmod +x "$STUB"
V="$(exo_version_de "$STUB")"
if [ "$V" = "0.1.0" ]; then
  pass "exo_version_de lee 'exo 0.1.0' -> 0.1.0"
else
  fail "exo_version_de lee 'exo 0.1.0' -> 0.1.0" "obtuve '$V'"
fi

V2="$(exo_version_de "$TMP/no-existe")"
if [ -z "$V2" ]; then
  pass "exo_version_de con binario inexistente -> cadena vacía"
else
  fail "exo_version_de con binario inexistente -> cadena vacía" "obtuve '$V2'"
fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
