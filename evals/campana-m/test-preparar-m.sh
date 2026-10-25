#!/usr/bin/env bash
# Tests bash (sin claude ni exo reales) de preparar-m.sh, congelar-m.sh y las variables nuevas de tanda.sh.
# Nada toca ~/.cache: K_ROOT en mktemp -d, exo/claude/preparar.sh/correr.sh son stubs.
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
HARNESS="$HERE/../ablacion-k/harness"
T=$(mktemp -d); trap 'chmod -R u+w "$T" 2>/dev/null; rm -rf "$T"' EXIT
fail=0
ok(){ echo "ok   $1"; }; ko(){ echo "FAIL $1"; fail=1; }
unset K_ROOT K_ORDEN K_TANDA K_CORRIDAS K_TOPE_USD K_CONGELACION K_REANUDAR K_TARBALL K_TARBALL_SHA

# ---- tanda.sh: copia junto a un correr.sh stub ----
mkdir -p "$T/h"; cp "$HARNESS/tanda.sh" "$T/h/tanda.sh"
cat > "$T/h/correr.sh" <<'STUB'
#!/usr/bin/env bash
id=$(basename "$1"); O="$K_ROOT/${K_CORRIDAS:-corridas}/$id/$2-r$3"; mkdir -p "$O"
echo "$id $2 $3" >> "$K_ROOT/invocaciones.log"
echo "$id $2 ${K_REGLA_FILE-<sin>}" >> "$K_ROOT/reglas-recibidas.log"
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
if [ $rc = 0 ] && [ "$lista" = "t3 t4 " ] && [ -f "$K/corridas-criba/t3/a0-r1/meta.json" ] && [ ! -e "$K/corridas" ] \
   && [ "$(sed -n 1,4p "$K/solape.log" | tr '\n' ' ')" = "start end start end " ]; then
  ok tanda_orden_propio; else ko tanda_orden_propio; fi

# tanda_pasa_regla: el brazo ar recibe K_REGLA_FILE=$K_ROOT/reglas/<tarea>.txt; los demás no
K=$(nuevo_k kar); echo f > "$K/congelacion.txt"
printf 't3\tar\t1\nt4\ta0\t1\n' > "$K/orden-etapa1.txt"
K_REGLA_FILE=ajeno K_ROOT=$K bash "$T/h/tanda.sh" 1 >/dev/null 2>&1
if grep -qx "t3 ar $K/reglas/t3.txt" "$K/reglas-recibidas.log" && grep -qx "t4 a0 <sin>" "$K/reglas-recibidas.log"; then
  ok tanda_pasa_regla; else ko "tanda_pasa_regla ($(tr '\n' ';' < "$K/reglas-recibidas.log"))"; fi

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
  echo "{\"fin\":\"ok\",\"tokens_in\":1,\"usd\":${3:-0}}" > "$k/corridas/otro/a0-r1/meta.json"   # otra carpeta de la campaña: cuenta
  STUB_USD=$2 K_ROOT=$k K_ORDEN="$k/o.txt" K_CORRIDAS=corridas-c K_TOPE_USD=1 bash "$T/h/tanda.sh" 1 > "$k/out.txt" 2>&1
  echo "$? $(grep -c 'PARAR: coste' "$k/out.txt") $(grep -c '^SEGUIR' "$k/out.txt")"
  LASTK=$k
}
r_sup=$(tope 0.7 0.4); r_inf=$(tope 0.1 0.2)   # 0,7 + 2×0,4 = 1,5 > 1 ; 0,1 + 2×0,2 = 0,5
r_cru=$(tope 0.3 0 0.6)   # 0,3 en corridas-c + 0,6 en corridas = 0,9 -> SEGUIR ; con 0,8 -> 1,1 -> PARAR
r_cru2=$(tope 0.3 0 0.8)
if [ "$r_sup" = "1 1 0" ] && [ "$r_inf" = "0 0 1" ] && [ "$r_cru" = "0 0 1" ] && [ "$r_cru2" = "1 1 0" ]; then ok tope_usd; else ko "tope_usd ($r_sup | $r_inf | $r_cru | $r_cru2)"; fi

# tope_meta_ilegible: un meta.json ilegible para (falla cerrado); un usd ausente se cuenta aparte y no para
tope 0.1 0 >/dev/null; echo '{rota' > "$LASTK/corridas-c/viejo/n-r3/meta.json"
K_ROOT=$LASTK K_ORDEN="$LASTK/o.txt" K_CORRIDAS=corridas-c K_TOPE_USD=1 bash "$T/h/tanda.sh" 1 > "$LASTK/out2.txt" 2>&1; rc=$?
if [ $rc = 1 ] && grep -q "PARAR: .*meta ilegible: $LASTK/corridas-c/viejo/n-r3/meta.json" "$LASTK/out2.txt"; then ok tope_meta_ilegible; else ko "tope_meta_ilegible ($rc)"; fi
tope 0.1 0 >/dev/null; echo '{"fin":"ok","tokens_in":1}' > "$LASTK/corridas-c/viejo/n-r3/meta.json"
K_ROOT=$LASTK K_ORDEN="$LASTK/o.txt" K_CORRIDAS=corridas-c K_TOPE_USD=1 bash "$T/h/tanda.sh" 1 > "$LASTK/out2.txt" 2>&1; rc=$?
if [ $rc = 0 ] && grep -q 'usd desconocido: 1' "$LASTK/out2.txt" && grep -q '^SEGUIR' "$LASTK/out2.txt"; then ok tope_usd_desconocido; else ko "tope_usd_desconocido ($rc)"; fi

# k_corridas_invalida (tanda): fuera de ^corridas[A-Za-z0-9_-]*$ => exit 2 sin lanzar nada
K=$(nuevo_k kv); echo f > "$K/congelacion.txt"; orden "$K/o.txt" 2; v=1
for bad in ../x otro corridas/sub; do
  K_ROOT=$K K_ORDEN="$K/o.txt" K_CORRIDAS=$bad bash "$T/h/tanda.sh" 1 >/dev/null 2>&1; [ $? = 2 ] || v=0
done
[ ! -e "$K/invocaciones.log" ] || v=0
K_ROOT=$K K_ORDEN="$K/o.txt" K_CORRIDAS=corridas-criba bash "$T/h/tanda.sh" 1 >/dev/null 2>&1 || v=0
[ -s "$K/invocaciones.log" ] || v=0
[ $v = 1 ] && ok k_corridas_invalida || ko k_corridas_invalida

# usd_invalido: no numérico y NaN paran con `usd inválido: <ruta>` (sin tumbar el breaker)
for val in '"abc"' NaN; do
  tope 0.1 0 >/dev/null; echo "{\"fin\":\"ok\",\"tokens_in\":1,\"usd\":$val}" > "$LASTK/corridas-c/viejo/n-r3/meta.json"
  K_ROOT=$LASTK K_ORDEN="$LASTK/o.txt" K_CORRIDAS=corridas-c K_TOPE_USD=1 bash "$T/h/tanda.sh" 1 > "$LASTK/out2.txt" 2>&1; rc=$?
  if [ $rc = 1 ] && grep -q "PARAR: .*usd inválido: $LASTK/corridas-c/viejo/n-r3/meta.json" "$LASTK/out2.txt"; then ok "usd_invalido_$val"; else ko "usd_invalido_$val ($rc)"; fi
done

# reanudar_con_k_corridas: una corrida completa (fin + fugas.json) en K_CORRIDAS no se relanza; la incompleta sí
K=$(nuevo_k kr); echo f > "$K/congelacion.txt"; orden "$K/o.txt" 2
mkdir -p "$K/corridas-criba/t1/a0-r1" "$K/corridas-criba/t2/a0-r1"
echo '{"fin":"ok","tokens_in":1,"usd":0}' > "$K/corridas-criba/t1/a0-r1/meta.json"; echo '{"fuga":false}' > "$K/corridas-criba/t1/a0-r1/fugas.json"
echo '{"tokens_in":1,"usd":0}' > "$K/corridas-criba/t2/a0-r1/meta.json"
K_ROOT=$K K_ORDEN="$K/o.txt" K_CORRIDAS=corridas-criba K_REANUDAR=1 bash "$T/h/tanda.sh" 1 >/dev/null 2>&1
if [ "$(cat "$K/invocaciones.log" 2>/dev/null)" = "t2 a0 1" ]; then ok reanudar_con_k_corridas; else ko "reanudar_con_k_corridas ($(tr '\n' ';' < "$K/invocaciones.log" 2>/dev/null))"; fi

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
[ "$1" = --forzar ] && shift
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
mkdir -p "$T/home" "$T/ktar/pool" "$T/kgold/s1/g0-1" "$T/kgold/s1/g0-2"
printf '{"id":"g0-1","nota":"core/a.md","regla":"Primera regla.\\nSegunda."}\n{"id":"g0-2","nota":"x","regla":"Sin salto final"}\n' > "$T/ktar/pool/reglas-g0.jsonl"
printf '{"id":"g1-1","nota":"x","regla":"De g1"}\n' > "$T/ktar/pool/reglas-g1.jsonl"; : > "$T/ktar/pool/reglas-g2.jsonl"
tar czf "$T/k-reg.tar.gz" -C "$T/ktar" pool
for g in g0-1 g0-2; do
  printf '#!/usr/bin/env bash\nH=/home/paul/Documentos/proyectos/exo/.worktrees/campana-k/evals/ablacion-k/harness\n"$H/herramientas.sh" "$2"\n' > "$T/kgold/s1/$g/check.sh"
  echo '{}' > "$T/kgold/s1/$g/tarea.json"
done
mkdir -p "$T/kgold/harness"; echo h > "$T/kgold/harness/herramientas.sh"
printf '#!/usr/bin/env bash\nH="$(cd "$(dirname "$0")/../../harness" && pwd)"\n' > "$T/kgold/s1/g0-1/check.sh"
cp -r "$T/kgold" "$T/kgold.orig"; chmod -R a-w "$T/kgold"   # el gold real de K es de solo lectura
prep() {  # prep cuda con tarball y gold falsos
  rm -rf "$T/kp"; mkdir -p "$T/kp"; : > "$T/prep.log"
  PATH="$T/bin:$PATH" STUB_LOG="$T/prep.log" STUB_DISP="$1" PREPARAR="$T/prep-stub/preparar.sh" K_ROOT="$T/kp" \
    K_TARBALL="$T/k-reg.tar.gz" K_GOLD_ORIGEN="$T/kgold/s1" HOME="$T/home" \
    bash "$HERE/preparar-m.sh" > "$T/prep.out" 2>&1
}
prep cuda; rc=$?
if [ $rc = 0 ] && [ -s "$T/kp/prep/memoria/MEMORY.md" ] && grep -q '^80cba57878d37583be1f2654ecb7d96c51dad8ab ' "$T/prep.log"; then
  ok preparar_cuda_ok; else ko "preparar_cuda_ok ($rc) $(cat "$T/prep.out")"; fi
prep cpu; rc=$?;  { [ $rc = 1 ] && grep -q 'índice no embebido en cuda' "$T/prep.out" && [ ! -e "$T/kp/prep/memoria" ]; } && ok preparar_exige_cuda_cpu || ko "preparar_exige_cuda_cpu ($rc)"
prep SIN; rc=$?;  { [ $rc = 1 ] && grep -q 'índice no embebido en cuda' "$T/prep.out"; } && ok preparar_exige_cuda_sin_clave || ko "preparar_exige_cuda_sin_clave ($rc)"

prep cuda
# preparar_reglas: reglas/<id>.txt byte a byte (sin salto añadido), de los tres g*
if [ "$(cat "$T/kp/reglas/g0-1.txt")" = $'Primera regla.\nSegunda.' ] && [ "$(wc -c < "$T/kp/reglas/g0-2.txt")" = "$(printf %s 'Sin salto final' | wc -c)" ] \
   && [ "$(cat "$T/kp/reglas/g1-1.txt")" = 'De g1' ]; then ok preparar_reglas; else ko "preparar_reglas"; fi

# preparar_gold: la COPIA apunta al harness del repo; el origen queda intacto
HR="$(cd "$HERE/../ablacion-k/harness" && pwd)"
if grep -qx "H=$HR" "$T/kp/gold/s1/g0-2/check.sh" && ! grep -rq 'campana-k' "$T/kp/gold/s1" && [ -f "$T/kp/gold/s1/g0-1/tarea.json" ] && [ -f "$T/kp/gold/harness/herramientas.sh" ] \
   && diff -r "$T/kgold" "$T/kgold.orig" >/dev/null; then ok preparar_gold_reescrito; else ko "preparar_gold_reescrito"; fi
# preparar_sin_k_root: sin K_ROOT explícito (o el de K) se niega y no llama a preparar.sh (rm -rf "$P")
: > "$T/prep.log"
o1=$(unset K_ROOT; PATH="$T/bin:$PATH" STUB_LOG="$T/prep.log" PREPARAR="$T/prep-stub/preparar.sh" bash "$HERE/preparar-m.sh" 2>&1); rc1=$?
o2=$(PATH="$T/bin:$PATH" STUB_LOG="$T/prep.log" PREPARAR="$T/prep-stub/preparar.sh" K_ROOT="$HOME/.cache/exo-ablacion-k" bash "$HERE/preparar-m.sh" 2>&1); rc2=$?
if [ $rc1 = 2 ] && [ $rc2 = 2 ] && grep -q 'K_ROOT' <<<"$o1" && grep -q 'K_ROOT' <<<"$o2" && [ ! -s "$T/prep.log" ]; then ok preparar_sin_k_root; else ko "preparar_sin_k_root ($rc1 $rc2)"; fi

exit $fail
