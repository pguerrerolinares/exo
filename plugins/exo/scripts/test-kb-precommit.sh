#!/usr/bin/env bash
# Test standalone para kb-precommit.sh. Repo git real en mktemp -d: el script
# llama `git rev-parse --show-toplevel` y `checkout-index`, que exigen un
# repo de verdad, no un fixture de texto.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/kb-precommit.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }
contains() { case "$1" in *"$2"*) return 0 ;; *) return 1 ;; esac; }

KB="$TMP/kb"
mkdir -p "$KB"
git -C "$KB" init -q
git -C "$KB" -c user.email=t@t.local -c user.name=t commit -q --allow-empty -m init

# ------------------- fail-closed: sin exo ejecutable ⇒ exit 1, BLOQUEADO ---
OUT="$(cd "$KB" && EXO_BIN="$TMP/no-existe-exo" "$HOOK" 2>&1)"; RC=$?
if [ "$RC" -eq 1 ] && contains "$OUT" "BLOQUEADO" && contains "$OUT" "--no-verify"; then
  pass "fail-closed: sin exo ejecutable ⇒ exit 1, BLOQUEADO, menciona --no-verify"
else
  fail "fail-closed: sin exo ejecutable ⇒ exit 1, BLOQUEADO, menciona --no-verify" "rc=$RC out=$OUT"
fi

# ------------------- camino feliz: exo stub que siempre pasa los gates -----
STUB="$TMP/exo-stub-ok"
cat > "$STUB" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
chmod +x "$STUB"
OUT2="$(cd "$KB" && EXO_BIN="$STUB" "$HOOK" 2>&1)"; RC2=$?
if [ "$RC2" -eq 0 ]; then
  pass "camino feliz: exo presente y los dos gates pasan ⇒ exit 0"
else
  fail "camino feliz: exo presente y los dos gates pasan ⇒ exit 0" "rc=$RC2 out=$OUT2"
fi

# ------------------- fallback al PATH: exo SOLO en PATH ⇒ pasa el gate ----
# Ni EXO_BIN ni $HOME/.local/bin/exo(.exe): el hook tiene que caer al mismo
# `command -v exo` que ya usan exo-recall.sh/recall-inject.sh (I2 de la
# review final de H). HOME aparte para no depender de si esta máquina tiene
# un exo real instalado en ~/.local/bin.
HOME_SIN_EXO="$TMP/home-sin-exo"
mkdir -p "$HOME_SIN_EXO"
PATH_CON_EXO="$TMP/path-con-exo"
mkdir -p "$PATH_CON_EXO"
cat > "$PATH_CON_EXO/exo" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
chmod +x "$PATH_CON_EXO/exo"
OUT4="$(cd "$KB" && env -u EXO_BIN HOME="$HOME_SIN_EXO" PATH="$PATH_CON_EXO:$PATH" "$HOOK" 2>&1)"; RC4=$?
if [ "$RC4" -eq 0 ]; then
  pass "fallback al PATH: exo solo en \$PATH (sin EXO_BIN ni ~/.local/bin) ⇒ pasa el gate"
else
  fail "fallback al PATH: exo solo en \$PATH (sin EXO_BIN ni ~/.local/bin) ⇒ pasa el gate" "rc=$RC4 out=$OUT4"
fi

# ------------------- precedencia: PATH antes que ~/.local/bin --------------
# Los dos sitios tienen un exo ejecutable a la vez (sin EXO_BIN): el hook
# debe resolver el mismo que exo-recall.sh/recall-inject.sh — `command -v
# exo` (PATH) — antes que el literal ~/.local/bin/exo. Cada stub marca en un
# fichero cuál se ejecutó para poder distinguirlos aunque ambos exit 0.
HOME_CON_EXO="$TMP/home-con-exo"
mkdir -p "$HOME_CON_EXO/.local/bin"
MARCA="$TMP/marca-precedencia"
cat > "$HOME_CON_EXO/.local/bin/exo" <<EOF
#!/usr/bin/env bash
echo "local-bin" > "$MARCA"
exit 0
EOF
chmod +x "$HOME_CON_EXO/.local/bin/exo"

PATH_CON_EXO2="$TMP/path-con-exo2"
mkdir -p "$PATH_CON_EXO2"
cat > "$PATH_CON_EXO2/exo" <<EOF
#!/usr/bin/env bash
echo "path" > "$MARCA"
exit 0
EOF
chmod +x "$PATH_CON_EXO2/exo"

rm -f "$MARCA"
OUT5="$(cd "$KB" && env -u EXO_BIN HOME="$HOME_CON_EXO" PATH="$PATH_CON_EXO2:$PATH" "$HOOK" 2>&1)"; RC5=$?
MARCADO="$(cat "$MARCA" 2>/dev/null || echo "<sin marca>")"
if [ "$RC5" -eq 0 ] && [ "$MARCADO" = "path" ]; then
  pass "precedencia: exo en \$PATH y en ~/.local/bin a la vez ⇒ gana \$PATH (mismo orden que los hooks)"
else
  fail "precedencia: exo en \$PATH y en ~/.local/bin a la vez ⇒ gana \$PATH (mismo orden que los hooks)" \
    "rc=$RC5 marcado=$MARCADO salida=$OUT5"
fi

# ------------------- gate real rechaza (ratchet) ⇒ exit 1, sin cambiar -----
STUB_FAIL="$TMP/exo-stub-fail"
cat > "$STUB_FAIL" <<'EOF'
#!/usr/bin/env bash
case "$1" in
  ratchet) echo "ratchet: techo subido" >&2; exit 3 ;;
  budget) exit 0 ;;
  *) exit 0 ;;
esac
EOF
chmod +x "$STUB_FAIL"
OUT3="$(cd "$KB" && EXO_BIN="$STUB_FAIL" "$HOOK" 2>&1)"; RC3=$?
if [ "$RC3" -eq 1 ] && contains "$OUT3" "QUÉ HACER"; then
  pass "gate real rechaza (ratchet) ⇒ exit 1, mensaje QUÉ HACER (sin cambios)"
else
  fail "gate real rechaza (ratchet) ⇒ exit 1, mensaje QUÉ HACER (sin cambios)" "rc=$RC3 out=$OUT3"
fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
