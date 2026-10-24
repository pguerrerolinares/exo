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
# `command -v exo` que ya usa exo-recall.sh (I2 de la
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
# debe resolver el mismo que exo-recall.sh — `command -v
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

# ------------------- frontmatter staged: tier (siempre) y tags (solo en A) --
# Repo aparte por caso; el stub ok deja pasar ratchet/budget, así que lo único
# que puede dar rojo es el chequeo de frontmatter.
caso_fm() { # $1=nombre $2=rc esperado $3=fragmento esperado en salida ("" = ninguno) $4=prep (función)
  local kb="$TMP/fm-$1"
  mkdir -p "$kb"
  git -C "$kb" init -q
  git -C "$kb" -c user.email=t@t.local -c user.name=t commit -q --allow-empty -m init
  "$4" "$kb"
  local out rc
  out="$(cd "$kb" && EXO_BIN="$STUB" "$HOOK" 2>&1)"; rc=$?
  if [ "$rc" -eq "$2" ] && { [ -z "$3" ] || contains "$out" "$3"; }; then
    pass "frontmatter: $1"
  else
    fail "frontmatter: $1" "rc=$rc out=$out"
  fi
}
nota() { printf -- '---\n%s\n---\ncuerpo\n' "$2" > "$1"; }
prep_a_sin_tier() { nota "$1/n.md" 'tags: [x]'; git -C "$1" add n.md; }
prep_a_sin_tags() { nota "$1/n.md" 'tier: stable'; git -C "$1" add n.md; }
prep_a_tags_vacio() { nota "$1/n.md" $'tier: stable\ntags: []'; git -C "$1" add n.md; }
prep_a_ok() { nota "$1/n.md" $'tier: stable\ntags: [x]'; git -C "$1" add n.md; }
prep_a_docs() { mkdir -p "$1/docs"; printf 'sin frontmatter\n' > "$1/docs/x.md"; printf 'x\n' > "$1/README.md"; git -C "$1" add docs/x.md README.md; }
prep_a_sidecar_distill() { mkdir -p "$1/.distill"; printf '## 2026-10-24 — movido del canon: x\nbloque\n' > "$1/.distill/n-moved.md"; git -C "$1" add .distill/n-moved.md; }
prep_a_tier_bloque() { nota "$1/n.md" $'tier: log\ntags:\n  - x'; git -C "$1" add n.md; }
prep_m_sin_tags() {
  nota "$1/n.md" 'tier: stable'
  git -C "$1" add n.md
  git -C "$1" -c user.email=t@t.local -c user.name=t commit -q -m base --no-verify
  printf 'más\n' >> "$1/n.md"; git -C "$1" add n.md
}
prep_m_sin_tier() {
  nota "$1/n.md" 'tags: [x]'
  git -C "$1" add n.md
  git -C "$1" -c user.email=t@t.local -c user.name=t commit -q -m base --no-verify
  printf 'más\n' >> "$1/n.md"; git -C "$1" add n.md
}
caso_fm "nota nueva sin tier ⇒ exit 1, 'falta tier'" 1 "n.md: falta tier" prep_a_sin_tier
caso_fm "nota nueva sin tags ⇒ exit 1, 'falta tags'" 1 "n.md: falta tags" prep_a_sin_tags
caso_fm "nota nueva con tags vacío ⇒ exit 1" 1 "n.md: falta tags" prep_a_tags_vacio
caso_fm "nota modificada sin tier ⇒ exit 1" 1 "n.md: falta tier" prep_m_sin_tier
caso_fm "nota modificada sin tags ⇒ exit 0 + aviso" 0 "aviso" prep_m_sin_tags
caso_fm "nota correcta ⇒ exit 0" 0 "" prep_a_ok
caso_fm "tags en lista YAML de bloque ⇒ exit 0" 0 "" prep_a_tier_bloque
caso_fm "docs/ y README.md exentos ⇒ exit 0" 0 "" prep_a_docs
caso_fm "sidecar .distill/ de auto-distill exento ⇒ exit 0" 0 "" prep_a_sidecar_distill

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
