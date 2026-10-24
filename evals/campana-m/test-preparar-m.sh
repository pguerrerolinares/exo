#!/usr/bin/env bash
# Tests bash (sin claude ni exo reales) de preparar-m.sh, congelar-m.sh y las variables nuevas de tanda.sh.
# Nada toca ~/.cache: K_ROOT en mktemp -d, exo/claude/preparar.sh/correr.sh son stubs.
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
HARNESS="$HERE/../ablacion-k/harness"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
fail=0
ok(){ echo "ok   $1"; }; ko(){ echo "FAIL $1"; fail=1; }
unset K_ROOT K_ORDEN K_TANDA K_CORRIDAS K_TOPE_USD K_CONGELACION K_REANUDAR K_TARBALL K_TARBALL_SHA

# ---- tanda.sh: copia junto a un correr.sh stub ----
mkdir -p "$T/h"; cp "$HARNESS/tanda.sh" "$T/h/tanda.sh"
cat > "$T/h/correr.sh" <<'STUB'
#!/usr/bin/env bash
id=$(basename "$1"); O="$K_ROOT/corridas/$id/$2-r$3"; mkdir -p "$O"
echo start >> "$K_ROOT/solape.log"; sleep 0.15; echo end >> "$K_ROOT/solape.log"
echo "{\"fin\":\"ok\",\"tokens_in\":100,\"usd\":${STUB_USD:-0}}" > "$O/meta.json"
echo '{"fuga":false}' > "$O/fugas.json"
STUB
chmod +x "$T/h/correr.sh"

nuevo_k() {  # $1 = nombre; crea K_ROOT con gold para t1..t50
  local k="$T/$1"; mkdir -p "$k/gold/s1"; for i in $(seq 1 50); do mkdir -p "$k/gold/s1/t$i"; done
  echo "$k"
}
orden() {  # $1 = fichero, $2 = n líneas
  : > "$1"; for i in $(seq 1 "$2"); do printf 't%s\ta0\t1\n' "$i" >> "$1"; done
}

# tanda_default_igual: sin variables nuevas, orden-etapa1.txt en bloques de 40, corridas/, paralelo 4
K=$(nuevo_k kd); echo f > "$K/congelacion.txt"; orden "$K/orden-etapa1.txt" 45
K_ROOT=$K bash "$T/h/tanda.sh" 2 >/dev/null 2>&1; rc2=$?
K_ROOT=$K bash "$T/h/tanda.sh" 1 >/dev/null 2>&1; rc1=$?
if [ $rc1 = 0 ] && [ $rc2 = 0 ] && [ "$(wc -l < "$K/tandas/1.lista")" = 40 ] && [ "$(wc -l < "$K/tandas/2.lista")" = 5 ] \
   && [ -f "$K/corridas/t45/a0-r1/meta.json" ] && [ -f "$K/tandas/1.txt" ] && grep -q '^SEGUIR' "$K/tandas/1.txt" \
   && [ "$(sed -n 1p "$K/solape.log")$(sed -n 2p "$K/solape.log")" = startstart ]; then
  ok tanda_default_igual; else ko tanda_default_igual; fi

# tanda_orden_propio: K_ORDEN + K_TANDA=2 lee bloques de 2 de ese fichero; K_CORRIDAS aparte; paralelo 1; K_CONGELACION
K=$(nuevo_k ko); orden "$K/orden-criba-a0.txt" 5; echo f > "$K/congelacion-criba.txt"
K_ROOT=$K K_ORDEN="$K/orden-criba-a0.txt" K_TANDA=2 K_CORRIDAS=corridas-criba K_CONGELACION="$K/congelacion-criba.txt" \
  bash "$T/h/tanda.sh" 2 >/dev/null 2>&1; rc=$?
lista=$(cat "$K/tandas/orden-criba-a0-2.lista" 2>/dev/null | cut -f1 | tr '\n' ' ')
if [ $rc = 0 ] && [ "$lista" = "t3 t4 " ] && [ -f "$K/corridas-criba/t3/a0-r1/meta.json" ] && [ ! -e "$K/corridas/t3/a0-r1" ] \
   && [ "$(sed -n 1,4p "$K/solape.log" | tr '\n' ' ')" = "start end start end " ]; then
  ok tanda_orden_propio; else ko tanda_orden_propio; fi

# tanda_sin_congelacion: sin el fichero, exit 2
K=$(nuevo_k ks); orden "$K/orden-etapa1.txt" 3
K_ROOT=$K bash "$T/h/tanda.sh" 1 >/dev/null 2>&1; [ $? = 2 ] && ok tanda_sin_congelacion || ko tanda_sin_congelacion

# tope_usd: la suma abarca todas las corridas de K_CORRIDAS (no solo la tanda)
tope() {  # $1=usd previo, $2=usd de la tanda
  local k; k=$(nuevo_k "kt$RANDOM"); echo f > "$k/congelacion.txt"; orden "$k/o.txt" 2
  mkdir -p "$k/corridas-c/viejo/a0-r1" "$k/corridas/otro/a0-r1"
  mkdir -p "$k/corridas-c/viejo/n-r3"
  echo "{\"fin\":\"ok\",\"tokens_in\":1,\"usd\":0}" > "$k/corridas-c/viejo/a0-r1/meta.json"
  echo "{\"fin\":\"ok\",\"tokens_in\":1,\"usd\":$1}" > "$k/corridas-c/viejo/n-r3/meta.json"
  echo '{"fin":"ok","tokens_in":1,"usd":99}' > "$k/corridas/otro/a0-r1/meta.json"   # otra carpeta: no cuenta
  STUB_USD=$2 K_ROOT=$k K_ORDEN="$k/o.txt" K_CORRIDAS=corridas-c K_TOPE_USD=1 bash "$T/h/tanda.sh" 1 > "$k/out.txt" 2>&1
  echo "$? $(grep -c 'PARAR: coste' "$k/out.txt") $(grep -c '^SEGUIR' "$k/out.txt")"
}
r_sup=$(tope 0.7 0.4); r_inf=$(tope 0.1 0.2)   # 0,7 + 2×0,4 = 1,5 > 1 ; 0,1 + 2×0,2 = 0,5
if [ "$r_sup" = "1 1 0" ] && [ "$r_inf" = "0 0 1" ]; then ok tope_usd; else ko "tope_usd ($r_sup | $r_inf)"; fi

# ---- congelar-m.sh ----
mkdir -p "$T/bin"
printf '#!/bin/sh\necho "exo 0.9.0"\n' > "$T/bin/exo"
printf '#!/bin/sh\necho "2.1.0 (Claude Code)"\n' > "$T/bin/claude"
chmod +x "$T/bin/exo" "$T/bin/claude"
K=$T/kc; mkdir -p "$K/prep/memoria" "$K/gold/s1/t1"
echo "kb_commit abc123" > "$K/prep/manifiesto.txt"; echo "- [a](a.md) — x" > "$K/prep/memoria/MEMORY.md"
echo "s1/t1" > "$K/gold/activas.txt"; echo x > "$K/gold/s1/t1/tarea.json"
orden "$K/orden-criba-a0.txt" 3
echo registro > "$T/reg.tar.gz"; SHA=$(sha256sum "$T/reg.tar.gz" | cut -d' ' -f1)
cong() { PATH="$T/bin:$PATH" K_ROOT=$K K_TARBALL="$T/reg.tar.gz" "$@" bash "$HERE/congelar-m.sh" criba 2>&1; }
salida=$(cong env K_TARBALL_SHA=$SHA); rc=$?
F="$K/congelacion-criba.txt"
if [ $rc = 0 ] && [ -f "$F" ] && grep -q "^tarball_k_sha256 $SHA" "$F" && grep -q '^kb_commit abc123' "$F" \
   && grep -q '^exo exo 0.9.0' "$F" && grep -qE '^modelo claude-sonnet-5-5 · claude 2\.1\.0' "$F" \
   && grep -q "^memoria_sha256 $(sha256sum < "$K/prep/memoria/MEMORY.md" | cut -c1-16)" "$F" \
   && grep -q '^gold_activo_sha256 [0-9a-f]\{64\}' "$F" && grep -q '^orden_sha256 ' "$F" \
   && grep -q '^semilla 20261024' "$F" && grep -q '^paralelo 1$' "$F"; then
  ok congelar_sha_tarball_coincide; else ko "congelar_sha_tarball_coincide ($rc) $salida"; fi
rm -f "$F"
salida=$(cong env K_TARBALL_SHA=0000); rc=$?
if [ $rc = 1 ] && grep -q 'tarball de K no coincide' <<<"$salida" && [ ! -e "$F" ]; then
  ok congelar_sha_tarball_discrepa; else ko "congelar_sha_tarball_discrepa ($rc)"; fi
PATH="$T/bin:$PATH" K_ROOT=$K K_TARBALL="$T/reg.tar.gz" K_TARBALL_SHA=$SHA bash "$HERE/congelar-m.sh" otra >/dev/null 2>&1
[ $? = 2 ] && ok congelar_fase_invalida || ko congelar_fase_invalida

# ---- preparar-m.sh ----
mkdir -p "$T/prep-stub"
cat > "$T/prep-stub/preparar.sh" <<'STUB'
#!/usr/bin/env bash
echo "$1 $K_ROOT" >> "$STUB_LOG"
P="$K_ROOT/prep"; rm -rf "$P"; mkdir -p "$P/kb/core"
git -C "$P/kb" init -q
printf -- '---\ntitle: A\ntier: core\n---\nTexto uno. Dos.\n' > "$P/kb/core/a.md"
git -C "$P/kb" add -A; git -C "$P/kb" -c user.name=t -c user.email=t@t commit -qm x
printf 'schema_version = 1\n' > "$P/config.toml"; echo "kb_commit $1" > "$P/manifiesto.txt"
STUB
chmod +x "$T/prep-stub/preparar.sh"
cat > "$T/bin/exo" <<'STUB'
#!/usr/bin/env bash
case "$1" in
  --version) echo "exo 0.9.0";;
  rebuild) python3 - "$EXO_DB" "${STUB_DISP-cuda}" <<'PY'
import sqlite3, sys
c = sqlite3.connect(sys.argv[1]); c.execute("CREATE TABLE meta (clave TEXT PRIMARY KEY, valor TEXT)")
if sys.argv[2] != "SIN": c.execute("INSERT INTO meta VALUES ('dispositivo_embeddings', ?)", (sys.argv[2],))
c.commit()
PY
  ;;
esac
STUB
chmod +x "$T/bin/exo"
prep() {  # $1 = valor de STUB_DISP ; devuelve rc y deja salida en $T/prep.out
  rm -rf "$T/kp"; mkdir -p "$T/kp"; : > "$T/prep.log"
  PATH="$T/bin:$PATH" STUB_LOG="$T/prep.log" STUB_DISP="$1" PREPARAR="$T/prep-stub/preparar.sh" K_ROOT="$T/kp" \
    bash "$HERE/preparar-m.sh" > "$T/prep.out" 2>&1
}
prep cuda; rc=$?
if [ $rc = 0 ] && [ -s "$T/kp/prep/memoria/MEMORY.md" ] && grep -q '^80cba57878d37583be1f2654ecb7d96c51dad8ab ' "$T/prep.log"; then
  ok preparar_cuda_ok; else ko "preparar_cuda_ok ($rc) $(cat "$T/prep.out")"; fi
prep cpu; rc=$?;  { [ $rc = 1 ] && grep -q 'índice no embebido en cuda' "$T/prep.out" && [ ! -e "$T/kp/prep/memoria" ]; } && ok preparar_exige_cuda_cpu || ko "preparar_exige_cuda_cpu ($rc)"
prep SIN; rc=$?;  { [ $rc = 1 ] && grep -q 'índice no embebido en cuda' "$T/prep.out"; } && ok preparar_exige_cuda_sin_clave || ko "preparar_exige_cuda_sin_clave ($rc)"

# preparar_sin_k_root: sin K_ROOT explícito (o el de K) se niega y no llama a preparar.sh (rm -rf "$P")
: > "$T/prep.log"
o1=$(unset K_ROOT; PATH="$T/bin:$PATH" STUB_LOG="$T/prep.log" PREPARAR="$T/prep-stub/preparar.sh" bash "$HERE/preparar-m.sh" 2>&1); rc1=$?
o2=$(PATH="$T/bin:$PATH" STUB_LOG="$T/prep.log" PREPARAR="$T/prep-stub/preparar.sh" K_ROOT="$HOME/.cache/exo-ablacion-k" bash "$HERE/preparar-m.sh" 2>&1); rc2=$?
if [ $rc1 = 2 ] && [ $rc2 = 2 ] && grep -q 'K_ROOT' <<<"$o1" && grep -q 'K_ROOT' <<<"$o2" && [ ! -s "$T/prep.log" ]; then ok preparar_sin_k_root; else ko "preparar_sin_k_root ($rc1 $rc2)"; fi

exit $fail
