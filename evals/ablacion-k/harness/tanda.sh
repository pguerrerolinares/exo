#!/usr/bin/env bash
# tanda.sh <n_tanda> [paralelo=4] — corre la tanda n (K_TANDA=40 corridas) del orden congelado de la etapa 1.
# Variables (campaña M; sin ellas, comportamiento de K):
#   K_CONGELACION  fichero de congelación (default $K_ROOT/congelacion.txt)
#   K_ORDEN        orden a leer (default $K_ROOT/orden-etapa1.txt). Con K_ORDEN el paralelo por
#                  defecto es 1: $P/kb es compartido y la restauración de correr.sh no es concurrente.
#   K_TANDA        corridas por tanda (default 40)
#   K_CORRIDAS     carpeta de resultados bajo $K_ROOT (default corridas); correr.sh la honra
#   K_TOPE_USD     si la suma de `usd` de TODAS las corridas de la campaña ($K_ROOT/corridas*/) lo supera → PARAR
# Tras la tanda aplica, SIN mirar ningún resultado de éxito (E3, §9):
#   - consumo: media de tokens de entrada por corrida > 2M → PARAR
#   - infra: > 10 % de corridas de un brazo sin `result` → PARAR
#   - fuga: cualquier fugas.json con fuga=true → PARAR
# Escribe $K_ROOT/tandas/<n>.txt con el resumen y sale con 1 si hay que parar.
set -uo pipefail
n=$1
K_TANDA="${K_TANDA:-40}"; K_CORRIDAS="${K_CORRIDAS:-corridas}"
par=${2:-$([ -n "${K_ORDEN:-}" ] && echo 1 || echo 4)}
H="$(cd "$(dirname "$0")" && pwd)"
K_ROOT="${K_ROOT:-$HOME/.cache/exo-ablacion-k}"; export K_ROOT
[[ "$K_CORRIDAS" =~ ^corridas[A-Za-z0-9_-]*$ ]] || { echo "K_CORRIDAS inválida: $K_CORRIDAS" >&2; exit 2; }
mkdir -p "$K_ROOT/tandas"
[ -f "${K_CONGELACION:-$K_ROOT/congelacion.txt}" ] || { echo "sin congelación" >&2; exit 2; }
# Con K_ORDEN, las tandas de órdenes distintos (p. ej. criba a0 y ar) no se pisan.
tag=""; [ -n "${K_ORDEN:-}" ] && tag="$(basename "$K_ORDEN" .txt)-"
L="$K_ROOT/tandas/$tag$n"
desde=$(( (n-1)*K_TANDA + 1 )); hasta=$(( n*K_TANDA ))
sed -n "${desde},${hasta}p" "${K_ORDEN:-$K_ROOT/orden-etapa1.txt}" > "$L.lista"
[ -s "$L.lista" ] || { echo "tanda $n vacía" >&2; exit 2; }

corre() {  # $1=tarea $2=brazo $3=rep
  local d="$K_ROOT/gold/s1/$1"; [ -d "$d" ] || d="$K_ROOT/gold/s2/$1"
  # K_REANUDAR=1: salta las corridas ya completas (con `fin` en meta.json). Solo para cortes
  # de infraestructura; se repiten todas las incompletas, de cualquier brazo.
  if [ "${K_REANUDAR:-0}" = 1 ] && jq -e '.fin' "$K_ROOT/$K_CORRIDAS/$1/$2-r$3/meta.json" >/dev/null 2>&1 && [ -f "$K_ROOT/$K_CORRIDAS/$1/$2-r$3/fugas.json" ]; then return 0; fi
  # El oráculo (ar) recibe su regla; reglas/ lo puebla preparar-m.sh. Los demás brazos no la ven.
  if [ "$2" = ar ]; then export K_REGLA_FILE="$K_ROOT/reglas/$1.txt"; else unset K_REGLA_FILE; fi
  "$H/correr.sh" "$d" "$2" "$3" > /dev/null 2>&1
}
export -f corre; export H K_REANUDAR K_CORRIDAS
xargs -P "$par" -L 1 bash -c 'corre "$0" "$1" "$2"' < "$L.lista"

python3 - "$K_ROOT" "$n" "$K_CORRIDAS" "${K_TOPE_USD:-}" "$tag" <<'PY'
import glob, json, math, os, sys, collections
root, n, corridas, tope, tag = sys.argv[1:6]
filas = [l.split() for l in open(f"{root}/tandas/{tag}{n}.lista")]
toks, sin_result, fugas, por_brazo = [], collections.Counter(), [], collections.Counter()
for t, b, r in filas:
    d = f"{root}/{corridas}/{t}/{b}-r{r}"
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
parar = []; usd_desc = 0
if media > 2_000_000: parar.append(f"consumo: media {media:,.0f} > 2M")
for b in por_brazo:
    if sin_result[b] / por_brazo[b] > 0.10: parar.append(f"infra: {b} {sin_result[b]}/{por_brazo[b]} sin result")
if fugas: parar.append(f"fuga: {fugas}")
if tope:
    gastado = 0.0
    # Un symlink (corridas -> corridas-principal) llevaría al mismo meta.json dos veces.
    for f in sorted({os.path.realpath(f) for f in glob.glob(f"{root}/corridas*/*/*/meta.json")}):
        try:
            u = json.load(open(f)).get("usd")
        except Exception:
            parar.append(f"meta ilegible: {f}"); continue
        if u is None: usd_desc += 1
        elif isinstance(u, bool) or not isinstance(u, (int, float)) or not math.isfinite(u):
            parar.append(f"usd inválido: {f}")
        else: gastado += u
    if gastado > float(tope): parar.append(f"coste: {gastado:.2f} USD > tope {float(tope):.2f}")
res = [f"tanda {n}: {len(filas)} corridas · tokens_in medio {media:,.0f} · total {sum(toks):,}",
       f"sin result por brazo: {dict(sin_result)}" + (f" · usd desconocido: {usd_desc}" if tope else ""), "PARAR: " + "; ".join(parar) if parar else "SEGUIR"]
open(f"{root}/tandas/{tag}{n}.txt", "w").write("\n".join(res) + "\n")
print("\n".join(res))
sys.exit(1 if parar else 0)
PY
