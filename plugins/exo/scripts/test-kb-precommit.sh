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

# ------------------- reglas duras: rules --check sobre el snapshot staged ---
# `exo` falso: ante `rules --check --kb <dir>` deja marca con el --kb recibido,
# cuenta las líneas `- ` bajo `## Reglas duras` y sale 3 si alguna nota pasa de
# 10. EXO_FAKE_SIN_CHECK=1 emula un engine anterior (clap, exit 2);
# EXO_FAKE_RC_CHECK=<n> fuerza ese exit.
EXO_REGLAS="$TMP/exo-reglas"
cat > "$EXO_REGLAS" <<'EOF'
#!/usr/bin/env bash
[ "$1" = rules ] && [ "$2" = "--check" ] && [ "$3" = "--kb" ] || exit 0
dir="$4"
[ -n "${EXO_FAKE_MARCA:-}" ] && echo "$dir" > "$EXO_FAKE_MARCA"
[ "${EXO_FAKE_SIN_CHECK:-}" = 1 ] && exit 2
[ -n "${EXO_FAKE_RC_CHECK:-}" ] && exit "$EXO_FAKE_RC_CHECK"
rc=0
for f in "$dir"/projects/*.md "$dir"/projects/*/*.md "$dir"/core/reglas-duras.md; do
  [ -f "$f" ] || continue
  n="$(awk '/^## /{en=($0=="## Reglas duras");next} en&&/^- /{c++} END{print c+0}' "$f")"
  if [ "$n" -gt 10 ]; then
    echo "exo rules --check: ${f#$dir/} tiene $n reglas duras (máximo 10): desplaza la última"
    rc=3
  fi
done
exit $rc
EOF
chmod +x "$EXO_REGLAS"

_nota_reglas() { # $1=kb $2=nº reglas
  local i
  { printf -- '---\ntier: stable\ntags: [x]\n---\n## Reglas duras\n'
    for ((i=1;i<=$2;i++)); do printf -- '- regla %d\n' "$i"; done; } > "$1/projects/p.md"
}
prep_reglas() { # $1=kb $2=nº reglas staged $3=nº reglas en disco (default = staged)
  mkdir -p "$1/projects"
  git -C "$1" init -q
  git -C "$1" -c user.email=t@t.local -c user.name=t commit -q --allow-empty -m init
  _nota_reglas "$1" "$2"; git -C "$1" add projects/p.md
  _nota_reglas "$1" "${3:-$2}"
}
corre_reglas() { # $1=kb; deja OUT/RC/MARCA_VISTA
  rm -f "$TMP/marca-reglas"
  OUT="$(cd "$1" && EXO_BIN="$EXO_REGLAS" EXO_FAKE_MARCA="$TMP/marca-reglas" "$HOOK" 2>&1)"; RC=$?
  MARCA_VISTA="$(cat "$TMP/marca-reglas" 2>/dev/null || echo "<sin marca>")"
}

K="$TMP/r-exceso"; prep_reglas "$K" 11; corre_reglas "$K"
if [ "$RC" -eq 1 ] && contains "$OUT" "tiene 11 reglas duras (máximo 10)"; then
  pass "precommit_rechaza_exceso: 11 reglas staged ⇒ exit 1 y mensaje"
else
  fail "precommit_rechaza_exceso: 11 reglas staged ⇒ exit 1 y mensaje" "rc=$RC out=$OUT"
fi

K="$TMP/r-cap"; prep_reglas "$K" 10; corre_reglas "$K"
if [ "$RC" -eq 0 ] && [ "$MARCA_VISTA" != "<sin marca>" ]; then
  pass "precommit_pasa_con_cap: 10 reglas ⇒ exit 0 y rules --check invocado"
else
  fail "precommit_pasa_con_cap: 10 reglas ⇒ exit 0 y rules --check invocado" "rc=$RC marca=$MARCA_VISTA out=$OUT"
fi

K="$TMP/r-viejo"; prep_reglas "$K" 11
OUT="$(cd "$K" && EXO_BIN="$EXO_REGLAS" EXO_FAKE_SIN_CHECK=1 "$HOOK" 2>&1)"; RC=$?
if [ "$RC" -eq 0 ] && contains "$OUT" "kb-precommit: el engine no conoce «rules --check»; reglas duras sin verificar"; then
  pass "precommit_engine_viejo: exit 2 ⇒ aviso y no bloquea"
else
  fail "precommit_engine_viejo: exit 2 ⇒ aviso y no bloquea" "rc=$RC out=$OUT"
fi

K="$TMP/r-fallo"; prep_reglas "$K" 10
OUT="$(cd "$K" && EXO_BIN="$EXO_REGLAS" EXO_FAKE_RC_CHECK=5 "$HOOK" 2>&1)"; RC=$?
if [ "$RC" -eq 1 ]; then
  pass "precommit_fallo_engine: exit 5 en --check ⇒ bloquea (fail-closed)"
else
  fail "precommit_fallo_engine: exit 5 en --check ⇒ bloquea (fail-closed)" "rc=$RC out=$OUT"
fi

K1="$TMP/r-idx1"; prep_reglas "$K1" 11 10; corre_reglas "$K1"
R1=$RC; O1=$OUT; M1=$MARCA_VISTA
K2="$TMP/r-idx2"; prep_reglas "$K2" 10 11; corre_reglas "$K2"
if [ "$R1" -eq 1 ] && contains "$O1" "tiene 11 reglas" && [ "$RC" -eq 0 ] \
   && [ "$M1" != "<sin marca>" ] && [ "$M1" != "$K1" ] \
   && [ "$MARCA_VISTA" != "<sin marca>" ] && [ "$MARCA_VISTA" != "$K2" ]; then
  pass "precommit_mira_el_index: staged 11/disco 10 bloquea; staged 10/disco 11 no; --kb es el snapshot"
else
  fail "precommit_mira_el_index: staged 11/disco 10 bloquea; staged 10/disco 11 no; --kb es el snapshot" \
    "r1=$R1 o1=$O1 m1=$M1 rc2=$RC m2=$MARCA_VISTA"
fi

# Sin snapshot (checkout-index falla): mismo aviso que antes, no bloquea y no
# llama a `rules --check` (no hay --kb que pasarle).
mkdir -p "$TMP/git-shim"
printf '#!/usr/bin/env bash\ncase " $* " in *" checkout-index "*) exit 1 ;; esac\nexec "%s" "$@"\n' "$(command -v git)" > "$TMP/git-shim/git"
chmod +x "$TMP/git-shim/git"
K="$TMP/r-sin-snap"; prep_reglas "$K" 11
rm -f "$TMP/marca-reglas"
OUT="$(cd "$K" && PATH="$TMP/git-shim:$PATH" EXO_BIN="$EXO_REGLAS" EXO_FAKE_MARCA="$TMP/marca-reglas" "$HOOK" 2>&1)"; RC=$?
if [ "$RC" -eq 0 ] && contains "$OUT" "kb-precommit: no pude materializar el index; presupuestos no verificados" && [ ! -e "$TMP/marca-reglas" ]; then
  pass "precommit_sin_snapshot: checkout-index falla ⇒ aviso, exit 0 y sin rules --check"
else
  fail "precommit_sin_snapshot: checkout-index falla ⇒ aviso, exit 0 y sin rules --check" "rc=$RC out=$OUT marca=$(cat "$TMP/marca-reglas" 2>/dev/null)"
fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
