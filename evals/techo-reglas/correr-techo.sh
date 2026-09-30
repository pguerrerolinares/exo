#!/usr/bin/env bash
# correr-techo.sh [paralelo=4] — las 28 corridas del test de techo, en el orden barajado de preregistro.md.
# Política sellada: cero re-intentos. Cada corrida se ejecuta una vez; correr.sh hace rm -rf de su dir,
# así que nunca se lanza una corrida que ya tenga meta.json. K_REANUDAR=1 tras un corte solo SALTA las
# que ya tienen meta.json (completas o no); sin K_REANUDAR, si ya hay alguna, aborta (exit 2).
# Una tarea `no` (o ausente) en $K_ROOT/reconstruccion.tsv no se corre: evaluar.py la cuenta no cumple/caída.
# Breakers (tras cada corrida): fuga=true, gasto acumulado (meta.json .usd) > K_TOPE_USD (15), y
# > 10 % de las planificadas sin `result`. Todo falla cerrado. Si salta alguno: no se lanzan más, exit 1, y hay que registrarlo
# en evals/techo-reglas/erratas.md antes de seguir. Exit 0 = tanda completa sin breakers; 2 = precondición.
# K_ENSAYO=1 pasa a correr.sh (no lanza claude; sin breakers ni tarball).
# Overrides de test: TECHO_CORRER, TECHO_TAREAS, TECHO_PREREG, TECHO_TARBALL.
set -uo pipefail
par=${1:-4}
H="$(cd "$(dirname "$0")" && pwd)"; ROOT="$(cd "$H/../.." && pwd)"
K_ROOT="${K_ROOT:-$HOME/.cache/exo-ablacion-k}"; export K_ROOT
TAREAS="${TECHO_TAREAS:-$H/tareas.tsv}"; PREREG="${TECHO_PREREG:-$H/preregistro.md}"
TARBALL="${TECHO_TARBALL:-$HOME/.cache/exo-techo-registro.tar.gz}"
export K_TOPE_USD="${K_TOPE_USD:-15}" K_ENSAYO="${K_ENSAYO:-0}" K_REANUDAR="${K_REANUDAR:-0}"
CORRER="${TECHO_CORRER:-$H/../ablacion-k/harness/correr.sh}"
STOP="$K_ROOT/techo-STOP"; Q="$K_ROOT/techo.cola"
die() { echo "correr-techo: $*" >&2; exit 2; }
[ -f "$K_ROOT/reconstruccion.tsv" ] || die "falta $K_ROOT/reconstruccion.tsv"

orden=$(sed -n '/^<!-- ORDEN-BEGIN -->$/,/^<!-- ORDEN-END -->$/p' "$PREREG" | grep -E '^[a-z0-9-]+ [12]$')
esperado=$(awk -F'\t' '$2=="suelo"{print $1" 1"; print $1" 2"} $2=="control"{print $1" 1"}' "$TAREAS" | sort)
[ -n "$esperado" ] && [ "$(printf '%s\n' "$orden" | sort)" = "$esperado" ] || die "el bloque ORDEN no es exactamente el conjunto de pares (id, rep) de $TAREAS"

mkdir -p "$K_ROOT"; : > "$Q"; rm -f "$STOP"; printf '%s\n' "$orden" > "$K_ROOT/techo.orden"; export PLAN; PLAN=$(printf '%s\n' "$orden" | grep -c .)
lanzadas=0; omitidas=0; previas=0
while read -r id rep; do
  regla=$(awk -F'\t' -v i="$id" '$1==i{print $3}' "$TAREAS")
  [ -n "$regla" ] || die "$id no está en $TAREAS"
  [ -s "$ROOT/$regla" ] || die "regla vacía o ausente: $ROOT/$regla"
  [ -d "$K_ROOT/gold/s1/$id" ] || die "sin $K_ROOT/gold/s1/$id"
  if [ "$(awk -F'\t' -v i="$id" '$1==i{print $2}' "$K_ROOT/reconstruccion.tsv")" != si ]; then
    echo "omitida (no reconstruible): $id r$rep"; omitidas=$((omitidas+1)); continue
  fi
  if [ -e "$K_ROOT/corridas/$id/ar-r$rep/meta.json" ]; then
    previas=$((previas+1))
    [ "$K_REANUDAR" = 1 ] && continue
  fi
  printf '%s %s %s\n' "$id" "$rep" "$ROOT/$regla" >> "$Q"; lanzadas=$((lanzadas+1))
done <<< "$orden"
[ "$previas" = 0 ] || [ "$K_REANUDAR" = 1 ] || die "$previas corridas ya tienen meta.json; cero re-intentos (K_REANUDAR=1 para saltarlas)"

# Estado de los pares del ORDEN (no de todo $K_ROOT): "<sin_result> <usd>" o vacío si no se puede calcular.
estado() {
  local f=() id rep
  while read -r id rep; do [ -e "$K_ROOT/corridas/$id/ar-r$rep/meta.json" ] && f+=("$K_ROOT/corridas/$id/ar-r$rep/meta.json"); done < "$K_ROOT/techo.orden"
  [ ${#f[@]} -gt 0 ] || return 0
  jq -s -r 'map(.usd // 0) as $u | if ($u | all(type=="number")) then "\(map(select((.fin // null) == null)) | length) \($u | add)" else error("usd no numérico") end' "${f[@]}" 2>/dev/null
}
export -f estado

# Tras cada corrida, fallando cerrado: fuga (cualquier cosa salvo un JSON válido con .fuga == false), gasto > tope
# (suma ilegible = STOP) o infra (sin_result*10 > corridas planificadas) -> techo-STOP; no se lanzan más.
corre() {  # $1=id $2=rep $3=regla
  [ -e "$STOP" ] && return 0
  K_REGLA_FILE="$3" "$CORRER" "$K_ROOT/gold/s1/$1" ar "$2" > /dev/null 2>&1
  [ "$K_ENSAYO" = 1 ] && return 0
  local d="$K_ROOT/corridas/$1/ar-r$2" st sr usd
  if [ ! -e "$d/fugas.json" ]; then echo "sin fugas.json: $1/ar-r$2 (setup o prep fallido; no es una fuga comprobada)" >> "$STOP"
  elif ! jq -e '.fuga == false' "$d/fugas.json" >/dev/null 2>&1; then echo "fuga: $1/ar-r$2 (fugas.json ilegible o fuga!=false)" >> "$STOP"
  fi
  st=$(estado); read -r sr usd <<< "$st"
  if [[ ! $sr =~ ^[0-9]+$ || ! $usd =~ ^[0-9.eE+-]+$ ]]; then echo "gasto: suma ilegible tras $1/ar-r$2" >> "$STOP"
  else
    awk -v u="$usd" -v t="$K_TOPE_USD" 'BEGIN{exit !(u>t)}' && echo "gasto: $usd USD > $K_TOPE_USD" >> "$STOP"
    [ $((sr*10)) -gt "$PLAN" ] && echo "infra: $sr sin result de $PLAN planificadas" >> "$STOP"
  fi
  return 0
}
export -f corre; export CORRER STOP K_ROOT K_TOPE_USD
xargs -P "$par" -L 1 bash -c 'corre "$0" "$1" "$2"' < "$Q"

[ "$K_ENSAYO" = 1 ] && { echo "ensayo: $lanzadas lanzadas, $omitidas omitidas"; exit 0; }

# Todas las corridas del orden con meta (también las de una sesión previa): resumen, breaker de infra y tarball de registro.
sin_result=0; total=0; usd=0; dirs=()
while read -r id rep _; do
  d="$K_ROOT/corridas/$id/ar-r$rep"; [ -e "$d/meta.json" ] || continue
  total=$((total+1)); dirs+=("corridas/$id/ar-r$rep")
  jq -e '.fin' "$d/meta.json" >/dev/null 2>&1 || sin_result=$((sin_result+1))
done <<< "$orden"
st=$(estado); read -r _ usd <<< "$st"; [ -n "$usd" ] || usd="ilegible"
parar=()
[ -e "$STOP" ] && parar+=("$(paste -sd';' "$STOP")")
[ $((sin_result*10)) -gt "$PLAN" ] && parar+=("infra: $sin_result sin result de $PLAN planificadas")
[ "$usd" = ilegible ] && parar+=("gasto: suma ilegible")
{ echo "techo: $total corridas con meta, $omitidas omitidas, previas saltadas $previas, $sin_result sin result, $usd USD"
  if [ ${#parar[@]} -gt 0 ]; then echo "PARAR: ${parar[*]}"; else echo SEGUIR; fi; } | tee "$K_ROOT/techo-resumen.txt"
[ ${#dirs[@]} -gt 0 ] && tar -czf "$TARBALL" -C "$K_ROOT" reconstruccion.tsv techo.orden techo-resumen.txt "${dirs[@]}" && echo "registro: $TARBALL"
[ ${#parar[@]} -eq 0 ]
