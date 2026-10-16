#!/usr/bin/env bash
# Puntúa el triage del ledger contra la verdad pre-registrada.
# Uso: upstream-score.sh <verdad.md> <ledger.md> [<umbral_pct>]   (umbral default 90)
# Empareja por (PR, skill). aplica y parcial cuentan como una sola clase (misma acción: portar).
# Clase no-portar-falsos: fila que el ledger deja sin portar (ya cubierto o no aplica) y la verdad
# etiqueta aplica o parcial. Umbral pre-registrado: 0.
# Sello: el ledger debe traer, en línea propia, `verdad_sello: <sha256> <desde>..<hasta>`; se cuenta
# la primera. El sha256 es sobre los bytes de la verdad tal cual (sin normalizar CRLF) y se compara
# con `sha256sum <verdad.md>`. Sin sello válido no se puntúa.
# Exit 0 si no-portar-falsos = 0 y pct >= umbral; 1 si no;
# 2 si falta tabla, fila malformada, umbral inválido, uso incorrecto o verdad sin sellar.
set -uo pipefail

[ $# -ge 2 ] || { echo "uso: $0 <verdad.md> <ledger.md> [umbral_pct]" >&2; exit 2; }
GOLD="$1"; LEDGER="$2"; UMBRAL="${3-90}"
[[ "$UMBRAL" =~ ^[0-9]+$ ]] && [ "$UMBRAL" -le 100 ] || { echo "upstream-score: umbral inválido: $UMBRAL" >&2; exit 2; }

# Emite "pr<TAB>skill<TAB>etiqueta" normalizado de la primera tabla cuya cabecera
# contiene las tres columnas pedidas. Sin tabla -> exit 3; fila malformada -> exit 4
# (ambos internos: el llamador los traduce a exit 2).
extraer() {
  awk -F'|' -v cs="$2" -v ce="$3" '
    function norm(s) { gsub(/`/, "", s); gsub(/^[ \t]+|[ \t]+$/, "", s); s = tolower(s); gsub(/[ \t]+/, " ", s); return s }
    /^[ \t]*\|/ {
      if (!fin && !hdr) {
        for (i = 1; i <= NF; i++) { c = norm($i); if (c == "pr") ip = i; else if (c == cs) is = i; else if (c == ce) ie = i }
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

sello_ledger="$(tr -d '\r' < "$LEDGER" | awk '/^verdad_sello:/ { print $2; exit }')"
sello_real="$(sha256sum < "$GOLD" | cut -d' ' -f1)"
{ [ -n "$sello_ledger" ] && [ "$sello_ledger" = "$sello_real" ]; } || { echo "upstream-score: verdad sin sellar: $LEDGER no trae verdad_sello con el sha256 de $GOLD" >&2; exit 2; }

RES="$(awk -F'\t' -v umbral="$UMBRAL" '
  function cls(e) { return (e == "aplica" || e == "parcial") ? "portar" : e }
  FNR == NR { if (NF && !(($1 SUBSEP $2) in led)) { led[$1, $2] = $3; lkeys[++nl] = $1 SUBSEP $2 } ; next }
  NF {
    k = $1 SUBSEP $2; total++; seen[k] = 1
    if (k in led) {
      if (cls(led[k]) == cls($3)) ok++
      else if ((led[k] == "ya cubierto" || led[k] == "no aplica") && ($3 == "aplica" || $3 == "parcial")) { falsos[++nf] = $1 " " $2 " (verdad: " $3 ")"; if (led[k] == "ya cubierto") yc++; else na++ }
    } else if ($3 == "revertido") ok++
    else aus++
  }
  END {
    for (i = 1; i <= nl; i++) if (!(lkeys[i] in seen)) ext++
    pct = total ? int(ok * 100 / total) : 0
    printf "aciertos %d/%d (%d%%)\n", ok, total, pct
    printf "no-portar-falsos %d (ya cubierto: %d, no aplica: %d)\n", nf, yc, na
    for (i = 1; i <= nf; i++) printf "  %s\n", falsos[i]
    printf "ausentes %d\n", aus
    printf "extra %d\n", ext
    exit (nf == 0 && total > 0 && ok * 100 >= umbral * total) ? 0 : 1
  }
' <(printf '%s\n' "$L") <(printf '%s\n' "$G"))"
rc=$?
printf '%s\n' "$RES"
exit $rc
