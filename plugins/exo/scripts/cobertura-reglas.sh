#!/usr/bin/env bash
# Informe de cobertura de las reglas duras (#207). Solo lee; sale 0 salvo que
# falte jq. Dos caras:
#   1. Lo declarado: `exo rules --coverage` sobre la KB. Se listan las notas al
#      CAP (lo siguiente que entre desplaza a otra regla), las que lo exceden y
#      las de sección vacía. Las `sin_seccion` solo se cuentan: casi todas son
#      notas hermanas (decisiones, runbooks) que no casan con ningún repo.
#   2. Lo no declarado: repos donde hubo sesiones y `exo rules` no entregó
#      (`project-rules-skip` del reflex-log con sin_nota, sin_seccion, ambigua
#      o seccion_vacia), en los últimos EXO_COBERTURA_DIAS días (30). Hoy esos
#      skips no se le muestran a nadie en la sesión.
#
# Uso: cobertura-reglas.sh   (EXO_BIN, EXO_KB, REFLEX_LOG_FILE, EXO_COBERTURA_DIAS)
set -uo pipefail

command -v jq >/dev/null 2>&1 || { echo "cobertura-reglas: falta jq" >&2; exit 1; }
EXO_BIN="${EXO_BIN:-$(command -v exo)}"
LOG="${REFLEX_LOG_FILE:-$HOME/.claude/reflex-log.jsonl}"
DIAS="${EXO_COBERTURA_DIAS:-30}"
case "$DIAS" in ''|*[!0-9]*) DIAS=30 ;; esac

echo "== Notas de la KB (exo rules --coverage)"
if [ -z "$EXO_BIN" ]; then
  echo "  ⚠ sin binario exo: cobertura de notas sin verificar"
else
  out="$("$EXO_BIN" rules --coverage --json ${EXO_KB:+--kb "$EXO_KB"} 2>&1)"; rc=$?
  if [ "$rc" -eq 2 ]; then
    echo "  ⚠ el engine no conoce «rules --coverage» (anterior a 0.13.0): cobertura de notas sin verificar"
  elif [ "$rc" -ne 0 ]; then
    echo "  ⚠ exo rules --coverage falló (rc=$rc): $(printf '%s' "$out" | head -c 300)"
  else
    printf '%s' "$out" | jq -r '
      .data as $d
      | ($d.notas | map(select(.estado == "al_cap" or .estado == "excede" or .estado == "seccion_vacia"))
         | if length == 0 then "  (ninguna al CAP, excedida ni vacía)"
           else .[] | "  \(.estado)\t\(.reglas)/\($d.cap)\t\(.nota)" end),
        "  resumen: \($d.notas | map(select(.estado == "ok" or .estado == "al_cap" or .estado == "excede")) | length) con reglas, \($d.notas | map(select(.estado == "sin_seccion")) | length) sin sección"
    ' 2>/dev/null | tr -d '\r' || echo "  ⚠ salida de exo rules --coverage ilegible"
  fi
fi

echo "== Repos con sesiones sin reglas (reflex-log, últimos $DIAS días)"
if [ ! -f "$LOG" ]; then
  echo "  (sin reflex-log en $LOG)"
  exit 0
fi
# El payload es «reason=<r> repo=<repo>[ err=…]». sin_git y los errores del
# engine no son huecos de cobertura: no se cuentan aquí.
# shellcheck disable=SC2016  # programa jq: los $ son de jq
SKIPS='
  select(.reflex == "project-rules-skip")
  | .ts as $ts
  | select((.ts // "") >= ((now - $dias * 86400) | todate))
  | (.payload | capture("reason=(?<r>[a-z_]+) repo=(?<repo>[^ ]+)")?) // empty
  | select(.r == "sin_nota" or .r == "sin_seccion" or .r == "ambigua" or .r == "seccion_vacia")
  | select(.repo != "-")
  | "\(.r)\t\(.repo)\t\($ts)"'
filas="$(jq -r --argjson dias "$DIAS" "$SKIPS" "$LOG" 2>/dev/null | tr -d '\r')"
if [ -z "$filas" ]; then
  echo "  (ninguno)"
else
  # Con el último skip a la vista: un hueco ya tapado deja de crecer, y su
  # fecha queda anterior a la nota o sección que lo cerró.
  printf '%s\n' "$filas" | awk -F'\t' '
    { k = $1 "\t" $2; n[k]++; if ($3 > u[k]) u[k] = $3 }
    END { for (k in n) printf "%d\t%s\t%s\n", n[k], k, u[k] }' \
    | sort -t$'\t' -k1,1nr -k3,3 \
    | awk -F'\t' '{ printf "  %4d arranques\t%s\t%s\túltimo %s\n", $1, $2, $3, $4 }'
fi
exit 0
