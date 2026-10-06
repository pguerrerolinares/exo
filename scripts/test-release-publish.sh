#!/usr/bin/env bash
# Gate de scripts/release-publish.sh, sin red y sin tocar ningún repo: se
# fabrica un `gh` falso que registra cada invocación y responde según el
# estado que el caso quiera simular.
#
# El caso que motiva el script es el tercero de la lista de abajo, y es el que
# rompió la publicación de v0.2.0 (run 35521897798): `gh release create` no es
# idempotente, así que el `workflow_dispatch` de "re-publicar un tag ya
# existente" —que existe justo para eso— moría con «a release with the same
# tag name already exists». Un camino de recuperación que nunca se ejerce no
# es un camino de recuperación.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
fallos=0

ok()  { echo "test-release-publish: OK — $1"; }
mal() { echo "test-release-publish: FALLO — $1" >&2; fallos=1; }

# --- El `gh` falso. Solo `release view` necesita responder algo: el script
# lee de ahí si la release existe (exit code) y cuántos assets tiene (stdout).
mkdir -p "$TMP/bin"
cat > "$TMP/bin/gh" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$STUB_LOG"
if [ "${1:-}" = release ] && [ "${2:-}" = view ]; then
  if [ "$STUB_MODO" = inexistente ]; then
    echo "release not found" >&2
    exit 1
  fi
  printf '%s\n' "$STUB_N"
fi
exit 0
STUB
chmod +x "$TMP/bin/gh"

# El test del mod de reglas, falso y en verde: lo real necesita `claude`, que
# no está en CI. Su contrato propio se ejerce en release_exige_mod_validado.
printf '#!/usr/bin/env bash\nexit 0\n' > "$TMP/mod-ok.sh"
chmod +x "$TMP/mod-ok.sh"

# --- Un dist con los 4 ficheros que publica una release real.
DIST="$TMP/dist"
mkdir -p "$DIST"
printf 'binario-falso' > "$DIST/exo-x86_64-unknown-linux-gnu"
printf 'hash  exo-x86_64-unknown-linux-gnu\n' > "$DIST/exo-x86_64-unknown-linux-gnu.sha256"
printf 'binario-falso' > "$DIST/exo-x86_64-pc-windows-msvc.exe"
printf 'hash  exo-x86_64-pc-windows-msvc.exe\n' > "$DIST/exo-x86_64-pc-windows-msvc.exe.sha256"

# corre <modo> <n_assets> [FORCE] -> deja el exit en $rc, el log en $TMP/gh.log
corre() {
  : > "$TMP/gh.log"
  STUB_LOG="$TMP/gh.log" STUB_MODO="$1" STUB_N="$2" FORCE="${3:-}" \
  GH_BIN="$TMP/bin/gh" DIST="$DIST" GITHUB_REPOSITORY=owner/repo \
  MOD_TEST="$TMP/mod-ok.sh" \
    bash ./scripts/release-publish.sh v9.9.9 > "$TMP/out.log" 2>&1
  rc=$?
}

invocado() { grep -q -- "$1" "$TMP/gh.log"; }

# --- Caso 1: la release no existe -> se crea, con notas.
corre inexistente 0
if [ "$rc" -ne 0 ]; then
  mal "release inexistente: esperaba exit 0, hubo $rc"; cat "$TMP/out.log" >&2
elif ! invocado "release create"; then
  mal "release inexistente: no se invocó 'gh release create'"; cat "$TMP/gh.log" >&2
elif invocado "release upload"; then
  mal "release inexistente: no debería subir assets a una release que aún no existe"
elif ! invocado "--notes"; then
  mal "release inexistente: se creó sin notas"
else
  ok "release inexistente ⇒ gh release create, con notas"
fi

# --- Caso 2: existe y está vacía (el caso de v0.2.0) -> upload, sin tocar notas.
corre vacia 0
if [ "$rc" -ne 0 ]; then
  mal "release vacía: esperaba exit 0, hubo $rc"; cat "$TMP/out.log" >&2
elif ! invocado "release upload"; then
  mal "release vacía: no se invocó 'gh release upload'"; cat "$TMP/gh.log" >&2
elif invocado "release create"; then
  mal "release vacía: intentó crear una release que ya existe (el bug de v0.2.0)"
elif invocado "--notes"; then
  mal "release vacía: tocó las notas de una release ya publicada"
elif invocado "--clobber"; then
  mal "release vacía: --clobber sin necesidad (no hay nada que reemplazar)"
else
  ok "release existente y vacía ⇒ gh release upload, sin crear ni tocar notas"
fi

# --- Caso 3: existe y YA tiene assets, sin force -> para, y no sube nada.
corre con-assets 6
if [ "$rc" -eq 0 ]; then
  mal "release con assets: salió 0 en vez de parar"
elif invocado "release upload"; then
  mal "release con assets: subió encima sin force (cambia sha256 ya publicados)"
elif ! grep -q "6" "$TMP/out.log"; then
  mal "release con assets: el error no dice cuántos assets hay"
elif ! grep -qi "force" "$TMP/out.log"; then
  mal "release con assets: el error no dice cómo forzar"
else
  ok "release con assets y sin force ⇒ para, sin subir, y explica cómo forzar"
fi

# --- Caso 4: existe, ya tiene assets, y se fuerza -> upload --clobber.
corre con-assets 6 1
if [ "$rc" -ne 0 ]; then
  mal "force: esperaba exit 0, hubo $rc"; cat "$TMP/out.log" >&2
elif ! invocado "--clobber"; then
  mal "force: subió sin --clobber, así que los assets existentes no se reemplazan"
else
  ok "force=1 ⇒ gh release upload --clobber"
fi

# --- Caso 5: dist vacío -> para ANTES de llamar a gh. Sin esto, el glob sin
# expandir se le pasa a gh como el literal 'dist/*' y la release sale a cero
# binarios con el job en verde.
vacio="$TMP/dist-vacio"; mkdir -p "$vacio"
: > "$TMP/gh.log"
STUB_LOG="$TMP/gh.log" STUB_MODO=inexistente STUB_N=0 FORCE="" \
GH_BIN="$TMP/bin/gh" DIST="$vacio" GITHUB_REPOSITORY=owner/repo \
MOD_TEST="$TMP/mod-ok.sh" \
  bash ./scripts/release-publish.sh v9.9.9 > "$TMP/out.log" 2>&1
rc=$?
if [ "$rc" -eq 0 ]; then
  mal "dist vacío: salió 0"
elif [ "$rc" -eq 127 ]; then
  mal "dist vacío: el script ni siquiera se pudo ejecutar (exit 127)"
elif ! grep -qi "dist\\|fichero" "$TMP/out.log"; then
  mal "dist vacío: paró, pero el error no dice que el problema es el dist"
  cat "$TMP/out.log" >&2
elif [ -s "$TMP/gh.log" ]; then
  mal "dist vacío: llamó a gh con un dist sin ficheros"; cat "$TMP/gh.log" >&2
else
  ok "dist vacío ⇒ para antes de llamar a gh"
fi

# --- release_exige_mod_validado: con un PATH sin `claude` y el test REAL del
# mod, la release no puede salir: falla antes de llamar a gh.
mkdir -p "$TMP/pathsin"
for b in bash env dirname find sort basename; do
  ln -sf "$(command -v "$b")" "$TMP/pathsin/$b"
done
: > "$TMP/gh.log"
PATH="$TMP/pathsin" STUB_LOG="$TMP/gh.log" STUB_MODO=inexistente STUB_N=0 FORCE="" \
GH_BIN="$TMP/bin/gh" DIST="$DIST" GITHUB_REPOSITORY=owner/repo \
  "$TMP/pathsin/bash" ./scripts/release-publish.sh v9.9.9 > "$TMP/out.log" 2>&1
rc=$?
if [ "$rc" -eq 0 ]; then
  mal "release_exige_mod_validado: salió 0 sin claude, la release saldría con el mod sin validar"
elif [ -s "$TMP/gh.log" ]; then
  mal "release_exige_mod_validado: llamó a gh con el mod sin validar"; cat "$TMP/gh.log" >&2
elif ! grep -q "SKIP-GRITA" "$TMP/out.log"; then
  mal "release_exige_mod_validado: paró, pero no por el mod sin validar"; cat "$TMP/out.log" >&2
else
  ok "release_exige_mod_validado ⇒ sin claude falla antes de gh"
fi

# --- mod_sin_claude_se_salta_en_voz_alta: sin `claude` y sin EXO_REQUIRE_CLAUDE,
# test-reglas-mod.sh sale 0 pero lo grita por stderr (CI no tiene claude).
PATH="$TMP/pathsin" "$TMP/pathsin/bash" ./plugins/exo/scripts/test-reglas-mod.sh \
  > "$TMP/mod.out" 2> "$TMP/mod.err"
rc=$?
if [ "$rc" -ne 0 ]; then
  mal "mod_sin_claude: esperaba exit 0, hubo $rc"; cat "$TMP/mod.err" >&2
elif ! grep -q "\[SKIP-GRITA\]" "$TMP/mod.err"; then
  mal "mod_sin_claude: saltó en silencio, sin [SKIP-GRITA] en stderr"
else
  ok "mod_sin_claude_se_salta_en_voz_alta ⇒ exit 0 y [SKIP-GRITA] por stderr"
fi

[ "$fallos" -eq 0 ] || { echo "test-release-publish: hay fallos" >&2; exit 1; }
echo "test-release-publish: OK — los siete casos"
