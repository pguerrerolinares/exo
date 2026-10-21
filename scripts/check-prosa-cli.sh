#!/usr/bin/env bash
# Toda invocación `exo <sub> [<sub2>] [--flag…]` citada en los .md de
# <plugin_dir> existe en `exo --help` / `exo <sub> --help` del binario real, y
# todo subcomando de `exo --help` está citado en algún .md (salvo EXCLUIDOS,
# con motivo). Un renombre del engine deja la prosa de los skills mandando a
# los agentes a comandos que ya no existen y nada lo ve. Los envelopes JSON los
# cubre plugins/exo/scripts/test-contrato-engine.sh; aquí solo la superficie CLI.
# Uso: check-prosa-cli.sh [<plugin_dir>]   (default plugins/exo)
# Binario: $EXO_BIN, o engine/target/debug/exo.
set -uo pipefail
RAIZ="$(git rev-parse --show-toplevel)" || exit 1
if [ -z "$RAIZ" ]; then echo "check-prosa-cli: sin toplevel" >&2; exit 1; fi
cd "$RAIZ" || exit 1
export LC_ALL=C.UTF-8

DIR="${1:-plugins/exo}"
BIN="${EXO_BIN:-engine/target/debug/exo}"
[ -d "$DIR" ] || { echo "check-prosa-cli: no existe $DIR" >&2; exit 1; }
[ -x "$BIN" ] || { echo "check-prosa-cli: no hay binario ejecutable en $BIN (compila el engine o fija EXO_BIN)" >&2; exit 1; }

# Subcomandos del engine que ninguna prosa del plugin cita, con motivo.
# Formato: nombre|motivo
EXCLUIDOS=(
  "init|setup de la máquina que hace el humano (docs/instalacion.md), no un skill"
  "index|lo dispara el hook post-commit, no un skill"
  "rebuild|remedio manual ante índice corrupto; lo guía `exo doctor`"
  "ratchet|gate de la KB que corre su propio pre-commit, no un skill"
  "rotate|mantenimiento de bitácoras que hoy hace el humano; /distill aún no lo cita"
  "stale|consulta de diagnóstico que ningún skill usa"
)

# Subcomandos listados bajo "Commands:" de `--help`, sin `help`.
subcomandos() {
  "$BIN" "$@" --help 2>/dev/null \
    | awk '/^Commands:/ {en=1; next} en && /^$/ {exit} en && /^  [a-z]/ {print $1}' \
    | grep -vx help
}
# Flags largos que anuncia `--help` (tolera `-x, --flag`).
flags() { "$BIN" "$@" --help 2>/dev/null | grep -oE -- '--[a-z][a-z0-9-]*' | sort -u; }

RAIZ_SUBS="$(subcomandos)"
if [ -z "$RAIZ_SUBS" ]; then
  echo "check-prosa-cli: '$BIN --help' no lista subcomandos (¿binario equivocado?)" >&2
  exit 1
fi

mapfile -t FICHEROS < <(find "$DIR" -type f -name '*.md' | sort)
if [ "${#FICHEROS[@]}" -eq 0 ]; then
  echo "check-prosa-cli: ningún .md bajo $DIR" >&2
  exit 1
fi

# Una invocación por línea: `fichero:línea<TAB>exo …`. Sale de spans inline
# `exo …` y de líneas de bloques ``` (con las continuaciones `\` unidas).
extrae() {
  awk '
    function emite(s) { if (s ~ /^exo [a-z]/) print FILENAME ":" ini "\t" s }
    FNR == 1 { fence = 0; pend = "" }
    /^[[:space:]]*```/ { fence = !fence; pend = ""; next }
    fence {
      l = $0; sub(/^[[:space:]]+/, "", l)
      if (pend == "") ini = FNR
      if (l ~ /\\$/) { sub(/\\$/, "", l); pend = pend l " "; next }
      emite(pend l); pend = ""; next
    }
    {
      ini = FNR; resto = $0
      while (match(resto, /`exo [^`]*`/)) {
        emite(substr(resto, RSTART + 1, RLENGTH - 2))
        resto = substr(resto, RSTART + RLENGTH)
      }
    }
  ' "${FICHEROS[@]}"
}

declare -A HELP_FLAGS CITADOS
FALLOS=0
fallo() { echo "$1"; FALLOS=$((FALLOS+1)); }

while IFS=$'\t' read -r donde cmd; do
  # shellcheck disable=SC2206 # troceo por espacios a propósito
  pal=($cmd)
  sub="${pal[1]}"
  [[ "$sub" =~ ^[a-z][a-z-]*$ ]] || continue
  if ! printf '%s\n' "$RAIZ_SUBS" | grep -qx "$sub"; then
    fallo "$donde: \`exo $sub\` no existe en exo --help"; continue
  fi
  CITADOS[$sub]=1
  ruta=("$sub"); desde=2
  hijos="$(subcomandos "$sub")"
  if [ -n "$hijos" ] && [[ "${pal[2]:-}" =~ ^[a-z][a-z-]*$ ]]; then
    if ! printf '%s\n' "$hijos" | grep -qx "${pal[2]}"; then
      fallo "$donde: \`exo $sub ${pal[2]}\` no existe en exo $sub --help"; continue
    fi
    ruta+=("${pal[2]}"); desde=3
  fi
  clave="${ruta[*]}"
  [ -n "${HELP_FLAGS[$clave]+x}" ] || HELP_FLAGS[$clave]="$(flags "${ruta[@]}")"
  for tok in "${pal[@]:desde}"; do
    [[ "$tok" =~ ^(--[a-z][a-z0-9-]*) ]] || continue
    fl="${BASH_REMATCH[1]}"
    printf '%s\n' "${HELP_FLAGS[$clave]}" | grep -qx -- "$fl" \
      || fallo "$donde: \`exo $clave\` no tiene $fl"
  done
done < <(extrae)

# Viceversa: subcomando del engine sin cita ni exclusión motivada.
for sub in $RAIZ_SUBS; do
  [ -n "${CITADOS[$sub]+x}" ] && continue
  excl=0
  for e in "${EXCLUIDOS[@]}"; do [ "${e%%|*}" = "$sub" ] && excl=1; done
  [ "$excl" -eq 1 ] || fallo "exo $sub existe en --help pero ningún .md de $DIR lo cita (cítalo o añádelo a EXCLUIDOS con motivo)"
done

[ "$FALLOS" -eq 0 ]
