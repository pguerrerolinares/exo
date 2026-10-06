#!/usr/bin/env bash
# correr-techo.sh [paralelo=4] — las corridas del test de techo, en el orden barajado de $TECHO_EXP/preregistro.md.
# TECHO_EXP (def. evals/techo-reglas; relativa = contra la raíz del repo) trae tareas.tsv (4.ª col. opcional
# `brazos` = brazo:k,... ; def. ar:2 suelo / ar:1 control), preregistro.md y, si existen, pins.sha256 (se verifica
# antes de lanzar nada, rutas relativas a $K_ROOT) y framing.txt (K_FRAMING_FILE). ORDEN: `brazo id rep` o, v1, `id rep` (= brazo ar).
# Política sellada: cero re-intentos. Cada corrida se ejecuta una vez; correr.sh hace rm -rf de su dir,
# así que nunca se lanza una corrida que ya tenga meta.json. K_REANUDAR=1 tras un corte solo SALTA las
# que ya tienen meta.json (completas o no); sin K_REANUDAR, si ya hay alguna, aborta (exit 2).
# Una tarea `no` (o ausente) en $K_ROOT/reconstruccion.tsv no se corre: evaluar.py la cuenta no cumple/caída.
# Breakers (tras cada corrida): fuga=true, gasto acumulado (meta.json .usd) > K_TOPE_USD (15), y
# > 10 % de las planificadas sin `result`. Todo falla cerrado. Si salta alguno: no se lanzan más, exit 1, y hay que registrarlo
# en el erratas.md del experimento antes de seguir (v2: la tanda es inválida y no se adjudica). Exit 0 = tanda completa sin breakers; 2 = precondición.
# K_ENSAYO=1 pasa a correr.sh (no lanza claude; sin breakers ni tarball); exit 0 solo si cada par lanzado dejó cmdline.txt.
# Overrides de test: TECHO_CORRER, TECHO_TAREAS, TECHO_PREREG, TECHO_TARBALL.
set -uo pipefail
par=${1:-4}
H="$(cd "$(dirname "$0")" && pwd)"; ROOT="$(cd "$H/../.." && pwd)"
EXP="${TECHO_EXP:-$H}"; case "$EXP" in /*) ;; *) EXP="$ROOT/$EXP";; esac
EXP="$(cd "$EXP" 2>/dev/null && pwd)" || { echo "correr-techo: no existe TECHO_EXP=${TECHO_EXP:-$H}" >&2; exit 2; }
K_ROOT="${K_ROOT:-$HOME/.cache/exo-ablacion-k}"; export K_ROOT
TAREAS="${TECHO_TAREAS:-$EXP/tareas.tsv}"; PREREG="${TECHO_PREREG:-$EXP/preregistro.md}"
if [ "$EXP" = "$H" ]; then P=techo; else P=$(basename "$EXP"); fi  # v1 conserva sus nombres
TARBALL="${TECHO_TARBALL:-$HOME/.cache/exo-$P-registro.tar.gz}"
export K_TOPE_USD="${K_TOPE_USD:-15}" K_ENSAYO="${K_ENSAYO:-0}" K_REANUDAR="${K_REANUDAR:-0}"
CORRER="${TECHO_CORRER:-$H/../ablacion-k/harness/correr.sh}"
if [ "$P" = techo ]; then STOP="$K_ROOT/techo-STOP"; Q="$K_ROOT/techo.cola"; ORDEN_F="$K_ROOT/techo.orden"; RESUMEN="$K_ROOT/techo-resumen.txt"
else STOP="$K_ROOT/$P-STOP"; Q="$K_ROOT/$P-cola"; ORDEN_F="$K_ROOT/$P-orden"; RESUMEN="$K_ROOT/$P-resumen.txt"; fi
die() { echo "correr-techo: $*" >&2; exit 2; }
[ -f "$K_ROOT/reconstruccion.tsv" ] || die "falta $K_ROOT/reconstruccion.tsv"

if [ -f "$EXP/pins.sha256" ]; then
  pins=$(cd "$K_ROOT" && sha256sum -c "$EXP/pins.sha256" 2>&1) || die "pins no coinciden: $(printf '%s\n' "$pins" | grep -v ': OK$' | sed 's/: FAILED.*//; s/^sha256sum: //' | paste -sd' ')"
fi
[ -f "$EXP/framing.txt" ] && export K_FRAMING_FILE="$EXP/framing.txt"

orden=$(sed -n '/^<!-- ORDEN-BEGIN -->$/,/^<!-- ORDEN-END -->$/p' "$PREREG" | grep -E '^([a-z0-9]+ )?[a-z0-9-]+ [0-9]+$' | awk 'NF==2{print "ar "$0; next} {print}')
esperado=$(awk -F'\t' '{b=($4!=""?$4:($2=="suelo"?"ar:2":"ar:1")); n=split(b,a,","); for(i=1;i<=n;i++){split(a[i],c,":"); for(r=1;r<=c[2];r++) print c[1]" "$1" "r}}' "$TAREAS" | sort)
[ -n "$esperado" ] && [ "$(printf '%s\n' "$orden" | sort)" = "$esperado" ] || die "el bloque ORDEN no es exactamente el conjunto de pares (brazo, id, rep) de $TAREAS"

mkdir -p "$K_ROOT"; : > "$Q"; rm -f "$STOP"; printf '%s\n' "$orden" > "$ORDEN_F"; export PLAN; PLAN=$(printf '%s\n' "$orden" | grep -c .)
lanzadas=0; omitidas=0; previas=0
while read -r brazo id rep; do
  regla=$(awk -F'\t' -v i="$id" '$1==i{print $3}' "$TAREAS")
  [ -n "$regla" ] || die "$id no está en $TAREAS"
  [ -s "$ROOT/$regla" ] || die "regla vacía o ausente: $ROOT/$regla"
  [ -d "$K_ROOT/gold/s1/$id" ] || die "sin $K_ROOT/gold/s1/$id"
  if [ "$(awk -F'\t' -v i="$id" '$1==i{print $2}' "$K_ROOT/reconstruccion.tsv")" != si ]; then
    echo "omitida (no reconstruible): $id r$rep${brazo:+ ($brazo)}"; omitidas=$((omitidas+1)); continue
  fi
  if [ -e "$K_ROOT/corridas/$id/$brazo-r$rep/meta.json" ]; then
    previas=$((previas+1))
    [ "$K_REANUDAR" = 1 ] && continue
  fi
  printf '%s %s %s %s\n' "$brazo" "$id" "$rep" "$ROOT/$regla" >> "$Q"; lanzadas=$((lanzadas+1))
done <<< "$orden"
[ "$previas" = 0 ] || [ "$K_REANUDAR" = 1 ] || die "$previas corridas ya tienen meta.json; cero re-intentos (K_REANUDAR=1 para saltarlas)"

# Estado de los pares del ORDEN (no de todo $K_ROOT): "<sin_result> <usd>" o vacío si no se puede calcular.
estado() {
  local f=() b id rep
  while read -r b id rep; do [ -e "$K_ROOT/corridas/$id/$b-r$rep/meta.json" ] && f+=("$K_ROOT/corridas/$id/$b-r$rep/meta.json"); done < "$ORDEN_F"
  [ ${#f[@]} -gt 0 ] || return 0
  jq -s -r 'map(.usd // 0) as $u | if ($u | all(type=="number")) then "\(map(select((.fin // null) == null)) | length) \($u | add)" else error("usd no numérico") end' "${f[@]}" 2>/dev/null
}
export -f estado

# Tras cada corrida, fallando cerrado: fuga (cualquier cosa salvo un JSON válido con .fuga == false), gasto > tope
# (suma ilegible = STOP) o infra (sin_result*10 > corridas planificadas) -> techo-STOP; no se lanzan más.
corre() {  # $1=brazo $2=id $3=rep $4=regla
  [ -e "$STOP" ] && return 0
  local salida crc d="$K_ROOT/corridas/$2/$1-r$3" st sr usd
  salida=$(K_REGLA_FILE="$4" "$CORRER" "$K_ROOT/gold/s1/$2" "$1" "$3" 2>&1); crc=$?
  [ $crc = 0 ] || echo "correr falló (rc=$crc): $2/$1-r$3: $(printf '%s' "$salida" | tail -n 3 | paste -sd'|')" >&2
  [ "$K_ENSAYO" = 1 ] && return 0
  if [ ! -e "$d/fugas.json" ]; then echo "sin fugas.json: $2/$1-r$3 (setup o prep fallido; no es una fuga comprobada)" >> "$STOP"
  elif ! jq -e '.fuga == false' "$d/fugas.json" >/dev/null 2>&1; then echo "fuga: $2/$1-r$3 (fugas.json ilegible o fuga!=false)" >> "$STOP"
  fi
  st=$(estado); read -r sr usd <<< "$st"
  if [[ ! $sr =~ ^[0-9]+$ || ! $usd =~ ^[0-9.eE+-]+$ ]]; then echo "gasto: suma ilegible tras $2/$1-r$3" >> "$STOP"
  else
    awk -v u="$usd" -v t="$K_TOPE_USD" 'BEGIN{exit !(u>t)}' && echo "gasto: $usd USD > $K_TOPE_USD" >> "$STOP"
    [ $((sr*10)) -gt "$PLAN" ] && echo "infra: $sr sin result de $PLAN planificadas" >> "$STOP"
  fi
  return 0
}
export -f corre; export CORRER STOP ORDEN_F K_ROOT K_TOPE_USD
xargs -P "$par" -L 1 bash -c 'corre "$0" "$1" "$2" "$3"' < "$Q"

if [ "$K_ENSAYO" = 1 ]; then
  sin=0; while read -r b id rep _; do [ -e "$K_ROOT/corridas/$id/$b-r$rep/cmdline.txt" ] || { echo "ensayo: sin cmdline.txt: $id/$b-r$rep" >&2; sin=$((sin+1)); }; done < "$Q"
  echo "ensayo: $lanzadas lanzadas, $omitidas omitidas, $sin sin cmdline.txt"; [ "$sin" = 0 ]; exit
fi

# Todas las corridas del orden con meta (también las de una sesión previa): resumen, breaker de infra y tarball de registro.
sin_result=0; total=0; usd=0; dirs=()
while read -r brazo id rep _; do
  d="$K_ROOT/corridas/$id/$brazo-r$rep"; [ -e "$d/meta.json" ] || continue
  total=$((total+1)); dirs+=("corridas/$id/$brazo-r$rep")
  jq -e '.fin' "$d/meta.json" >/dev/null 2>&1 || sin_result=$((sin_result+1))
done <<< "$orden"
st=$(estado); read -r _ usd <<< "$st"; [ -n "$usd" ] || usd="ilegible"
parar=()
[ -e "$STOP" ] && parar+=("$(paste -sd';' "$STOP")")
[ $((sin_result*10)) -gt "$PLAN" ] && parar+=("infra: $sin_result sin result de $PLAN planificadas")
[ "$usd" = ilegible ] && parar+=("gasto: suma ilegible")
{ echo "techo: $total corridas con meta, $omitidas omitidas, previas saltadas $previas, $sin_result sin result, $usd USD"
  if [ ${#parar[@]} -gt 0 ]; then echo "PARAR: ${parar[*]}"; else echo SEGUIR; fi; } | tee "$RESUMEN"
mkdir -p "$(dirname "$TARBALL")"
[ ${#dirs[@]} -gt 0 ] && tar -czf "$TARBALL" -C "$K_ROOT" reconstruccion.tsv "$(basename "$ORDEN_F")" "$(basename "$RESUMEN")" "${dirs[@]}" && echo "registro: $TARBALL"
[ ${#parar[@]} -eq 0 ]
