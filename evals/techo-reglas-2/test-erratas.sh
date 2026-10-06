#!/usr/bin/env bash
# Tests de las erratas del gold (T2). Todo en mktemp -d; $K_ROOT y el tarball son solo lectura.
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
K_ROOT="${K_ROOT:-$HOME/.cache/exo-ablacion-k}"
TARBALL="${TARBALL:-$HOME/.cache/exo-ablacion-k-registro.tar.gz}"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
pass=0; fail=0
ok()  { echo "PASS $1"; pass=$((pass+1)); }
bad() { echo "FAIL $1${2:+ — $2}"; fail=$((fail+1)); }
chk() { if [ "$2" = "$3" ]; then ok "$1"; else bad "$1" "got=$2 want=$3"; fi; }

# originales desde el tarball (no desde $K_ROOT, que podría corromperse)
mkdir -p "$T/reg" && tar -xzf "$TARBALL" -C "$T/reg" gold-activo.tar \
  && mkdir -p "$T/orig" && tar -xf "$T/reg/gold-activo.tar" -C "$T/orig" || { echo "FAIL tarball"; exit 1; }
tar -xzf "$TARBALL" -C "$T/reg" corridas/g2-154/a0-r1/diff.patch corridas/g2-154/a0-r2/diff.patch
orig() { echo "$T/orig/s1/$1/check.sh"; }
[ -f "$(orig g2-154)" ] || { echo "FAIL layout gold-activo.tar (s1/<id>)"; exit 1; }

nuevo() { # nuevo <dir>: workdir de g2-154; imprime commit base
  bash "$K_ROOT/gold/s1/g2-154/setup.sh" "$1" >/dev/null 2>&1; git -C "$1" rev-parse HEAD; }
correr() { # correr <check> <wd> <base>
  bash "$1" "$2" /dev/null "$3" >/dev/null 2>&1; echo $?; }

# 1) clave arbitraria
wd="$T/w1"; base="$(nuevo "$wd")"
python3 -I - "$wd/kb.py" <<'PY'
import sys
p = sys.argv[1]; s = open(p).read()
s = s.replace('d.add_argument("--root", default="kb")',
  'd.add_argument("--detailed", action="store_true")\n    d.add_argument("--root", default="kb")')
s = s.replace('        out = check_schema(a.root)\n',
  '        out = check_schema(a.root)\n        if a.detailed:\n            out["issues"] = [{"type": "missing_file", "path": m, "detail": "x"} for m in out["missing"]]\n')
open(p, "w").write(s)
PY
chk g2154_acepta_cualquier_clave "$(correr "$HERE/gold/g2-154/check.sh" "$wd" "$base")" 0
chk g2154_acepta_cualquier_clave_orig_rechaza "$(correr "$(orig g2-154)" "$wd" "$base")" 1

# 2) v1 ar-r1/r2
for r in r1 r2; do
  wd="$T/w2$r"; base="$(nuevo "$wd")"; git -C "$wd" apply "$K_ROOT/corridas/g2-154/ar-$r/diff.patch" || bad g2154_v1_ar_pasa "apply $r"
  chk "g2154_v1_ar_pasa_$r(corregido)" "$(correr "$HERE/gold/g2-154/check.sh" "$wd" "$base")" 0
  chk "g2154_v1_ar_pasa_$r(original)" "$(correr "$(orig g2-154)" "$wd" "$base")" 1
done

# 3) migración en sitio de K
for r in r1 r2; do
  wd="$T/w3$r"; base="$(nuevo "$wd")"; git -C "$wd" apply "$T/reg/corridas/g2-154/a0-$r/diff.patch" || bad g2154_rechaza_migracion_a0_K "apply $r"
  rc="$(correr "$HERE/gold/g2-154/check.sh" "$wd" "$base")"
  [ "$rc" != 0 ] && ok "g2154_rechaza_migracion_a0_K_$r (rc=$rc)" || bad "g2154_rechaza_migracion_a0_K_$r" "rc=0"
done

# 4) helpers reproducen v1, en un K_ROOT temporal
KT="$T/k"; mkdir -p "$KT/gold/harness" "$T/wd"; echo nota > "$T/wd/NOTA_CTO.md"  # g0-33 solo exige NOTA_CTO.md no vacío; g0-122 solo lee scripts del wd
install -m755 "$HERE"/gold/harness/*.sh "$KT/gold/harness/"
for i in g0-122 g0-33 g2-35; do
  mkdir -p "$KT/gold/s1/$i"; install -m755 "$HERE/gold/$i/check.sh" "$KT/gold/s1/$i/"
  for c in "$K_ROOT"/corridas/$i/ar-r*; do
    want="$(cat "$c/check.rc")"
    chk "helpers_reproducen_v1_$i/$(basename "$c")" "$(bash "$KT/gold/s1/$i/check.sh" "$T/wd" "$c/transcript.jsonl" >/dev/null 2>&1; echo $?)" "$want"
    o="$(bash "$(orig "$i")" "$T/wd" "$c/transcript.jsonl" >/dev/null 2>&1; echo $?)"
    chk "helpers_original_roto_$i/$(basename "$c")" "$o" 2
  done
done

# 5) diff mínimo
d="$(diff "$(orig g2-154)" "$HERE/gold/g2-154/check.sh" | grep -E '^[<>]' | wc -l)"
chk diff_minimo_g2-154_lineas "$d" 3
diff "$(orig g2-154)" "$HERE/gold/g2-154/check.sh" | grep -qE '^[<>].*(f = g.get|ok = isinstance)' && ok diff_minimo_g2-154_solo_valid || bad diff_minimo_g2-154_solo_valid
for i in g0-122 g0-33 g2-35; do
  d="$(diff "$(orig $i)" "$HERE/gold/$i/check.sh" | grep -E '^[<>]')"
  [ "$(echo "$d" | wc -l)" = 2 ] && echo "$d" | grep -q '^< H=.*campana-k' && echo "$d" | grep -q '^> H="\$(cd' && ok "diff_minimo_$i" || bad "diff_minimo_$i" "$d"
done
for i in g2-154 g0-122 g0-33 g2-35; do [ "$(grep -c 'worktrees/campana-k' "$HERE/gold/$i/check.sh")" = 0 ] && ok "sin_campana-k_$i" || bad "sin_campana-k_$i"; done
for f in herramientas comandos; do git -C "$HERE" show "main:evals/ablacion-k/harness/$f.sh" | cmp -s - "$HERE/gold/harness/$f.sh" && ok "harness_copia_main_$f" || bad "harness_copia_main_$f"; done

echo "== $pass PASS, $fail FAIL"; [ "$fail" = 0 ]
