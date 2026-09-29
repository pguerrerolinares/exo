#!/usr/bin/env bash
# tanda.sh <n_tanda> [paralelo=4] — corre la tanda n (40 corridas) del orden congelado de la etapa 1.
# Tras la tanda aplica, SIN mirar ningún resultado de éxito (E3, §9):
#   - consumo: media de tokens de entrada por corrida > 2M → PARAR
#   - infra: > 10 % de corridas de un brazo sin `result` → PARAR
#   - fuga: cualquier fugas.json con fuga=true → PARAR
# Escribe $K_ROOT/tandas/<n>.txt con el resumen y sale con 1 si hay que parar.
set -uo pipefail
n=$1; par=${2:-4}
H="$(cd "$(dirname "$0")" && pwd)"
K_ROOT="${K_ROOT:-$HOME/.cache/exo-ablacion-k}"; export K_ROOT
mkdir -p "$K_ROOT/tandas"
[ -f "$K_ROOT/congelacion.txt" ] || { echo "sin congelación" >&2; exit 2; }
desde=$(( (n-1)*40 + 1 )); hasta=$(( n*40 ))
sed -n "${desde},${hasta}p" "$K_ROOT/orden-etapa1.txt" > "$K_ROOT/tandas/$n.lista"
[ -s "$K_ROOT/tandas/$n.lista" ] || { echo "tanda $n vacía" >&2; exit 2; }

corre() {  # $1=tarea $2=brazo $3=rep
  local d="$K_ROOT/gold/s1/$1"; [ -d "$d" ] || d="$K_ROOT/gold/s2/$1"
  # K_REANUDAR=1: salta las corridas ya completas (con `fin` en meta.json). Solo para cortes
  # de infraestructura; se repiten todas las incompletas, de cualquier brazo.
  if [ "${K_REANUDAR:-0}" = 1 ] && jq -e '.fin' "$K_ROOT/corridas/$1/$2-r$3/meta.json" >/dev/null 2>&1; then return 0; fi
  "$H/correr.sh" "$d" "$2" "$3" > /dev/null 2>&1
}
export -f corre; export H K_REANUDAR
xargs -P "$par" -L 1 bash -c 'corre "$0" "$1" "$2"' < "$K_ROOT/tandas/$n.lista"

python3 - "$K_ROOT" "$n" <<'PY'
import json, os, sys, collections
root, n = sys.argv[1], sys.argv[2]
filas = [l.split() for l in open(f"{root}/tandas/{n}.lista")]
toks, sin_result, fugas, por_brazo = [], collections.Counter(), [], collections.Counter()
for t, b, r in filas:
    d = f"{root}/corridas/{t}/{b}-r{r}"
    por_brazo[b] += 1
    try:
        m = json.load(open(f"{d}/meta.json"))
    except Exception:
        m = {"error": "sin meta"}
    if "tokens_in" in m:
        toks.append(m["tokens_in"])
    else:
        sin_result[b] += 1
    try:
        if json.load(open(f"{d}/fugas.json"))["fuga"]:
            fugas.append(f"{t}/{b}-r{r}")
    except Exception:
        fugas.append(f"{t}/{b}-r{r} (sin fugas.json)")
media = sum(toks) / len(toks) if toks else 0
parar = []
if media > 2_000_000: parar.append(f"consumo: media {media:,.0f} > 2M")
for b in por_brazo:
    if sin_result[b] / por_brazo[b] > 0.10: parar.append(f"infra: {b} {sin_result[b]}/{por_brazo[b]} sin result")
if fugas: parar.append(f"fuga: {fugas}")
res = [f"tanda {n}: {len(filas)} corridas · tokens_in medio {media:,.0f} · total {sum(toks):,}",
       f"sin result por brazo: {dict(sin_result)}", "PARAR: " + "; ".join(parar) if parar else "SEGUIR"]
open(f"{root}/tandas/{n}.txt", "w").write("\n".join(res) + "\n")
print("\n".join(res))
sys.exit(1 if parar else 0)
PY
