#!/usr/bin/env bash
# Puntúa el triage del ledger contra la verdad pre-registrada.
# Uso: upstream-score.sh <verdad.md> <ledger.md> [<umbral_pct>]   (umbral default 90)
# Empareja por (PR, skill). Exit 0 si ya-cubierto-falsos = 0 y pct >= umbral; 1 si no; 2 si falta tabla.
set -uo pipefail

[ $# -ge 2 ] || { echo "uso: $0 <verdad.md> <ledger.md> [umbral_pct]" >&2; exit 2; }
GOLD="$1"; LEDGER="$2"; UMBRAL="${3-90}"
[[ "$UMBRAL" =~ ^[0-9]+$ ]] && [ "$UMBRAL" -le 100 ] || { echo "upstream-score: umbral inválido: $UMBRAL" >&2; exit 2; }

# Emite "pr<TAB>skill<TAB>etiqueta" normalizado de la primera tabla cuya cabecera
# contiene las tres columnas pedidas. Sin tabla -> exit 3.
extraer() {
  awk -F'|' -v cp="pr" -v cs="$2" -v ce="$3" '
    function norm(s) { gsub(/`/, "", s); gsub(/^[ \t]+|[ \t]+$/, "", s); s = tolower(s); gsub(/[ \t]+/, " ", s); return s }
    /^[ \t]*\|/ {
      if (!fin && !hdr) {
        for (i = 1; i <= NF; i++) { c = norm($i); if (c == cp) ip = i; else if (c == cs) is = i; else if (c == ce) ie = i }
        if (ip && is && ie) { hdr = 1; next }
        ip = is = ie = 0; next
      }
      if (hdr && !fin) {
        if ($0 ~ /^[ \t]*\|[ \t:|-]+\|[ \t]*$/) next
        pr = norm($ip); sub(/^#/, "", pr); gsub(/ /, "", pr)
        sk = norm($is); et = norm($ie)
        if (pr == "" || sk == "" || et == "") { print "upstream-score: fila malformada en " FILENAME ": " $0 > "/dev/stderr"; bad = 1; exit 4 }
        print pr "\t" sk "\t" et
      }
      next
    }
    { if (hdr) fin = 1 }
    END { exit (bad ? 4 : hdr ? 0 : 3) }
  ' "$1"
}

G="$(extraer "$GOLD" "skill exo" "etiqueta")" || { [ $? -eq 4 ] && exit 2; echo "upstream-score: no hay tabla (PR | skill exo | etiqueta) en $GOLD" >&2; exit 2; }
L="$(extraer "$LEDGER" "skill" "triage")" || { [ $? -eq 4 ] && exit 2; echo "upstream-score: no hay tabla (PR | skill | triage) en $LEDGER" >&2; exit 2; }

RES="$(awk -F'\t' -v umbral="$UMBRAL" '
  FNR == NR { if (NF && !(($1 SUBSEP $2) in led)) { led[$1, $2] = $3; lkeys[++nl] = $1 SUBSEP $2 } ; next }
  NF {
    k = $1 SUBSEP $2; total++; seen[k] = 1
    if (k in led) {
      if (led[k] == $3) ok++
      else if (led[k] == "ya cubierto" && ($3 == "aplica" || $3 == "parcial")) falsos[++nf] = $1 " " $2 " (verdad: " $3 ")"
    } else if ($3 == "revertido") ok++
    else aus++
  }
  END {
    for (i = 1; i <= nl; i++) if (!(lkeys[i] in seen)) ext++
    pct = total ? int(ok * 100 / total) : 0
    printf "aciertos %d/%d (%d%%)\n", ok, total, pct
    printf "ya-cubierto-falsos %d\n", nf
    for (i = 1; i <= nf; i++) printf "  %s\n", falsos[i]
    printf "ausentes %d\n", aus
    printf "extra %d\n", ext
    exit (nf == 0 && total > 0 && ok * 100 >= umbral * total) ? 0 : 1
  }
' <(printf '%s\n' "$L") <(printf '%s\n' "$G"))"
rc=$?
printf '%s\n' "$RES"
exit $rc
