#!/usr/bin/env bash
# Tests de upstream-score.sh con fixtures sintéticos (la verdad real vive fuera del repo).
set -uo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCORE="$DIR/upstream-score.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0

ok()  { PASS=$((PASS+1)); echo "PASS: $1"; }
bad() { FAIL=$((FAIL+1)); echo "FAIL: $1"; }

# gold_row <pr> <skill> <etiqueta>
gold_head() { printf '# gold\n\n| PR | título | base | skill exo | etiqueta | evidencia | conf. |\n|---|---|---|---|---|---|---|\n'; }
gold_row()  { printf '| %s | titulo x | dev | %s | %s | ev | alta |\n' "$1" "$2" "$3"; }
led_head()  { printf 'upstream_tag: v9.9.9\n\n| PR | skill | triage | estado | motivo | hash |\n|---|---|---|---|---|---|\n'; }
led_row()   { printf '| %s | %s | %s | hecho | m | abc123 |\n' "$1" "$2" "$3"; }

# Verdad base de 20 filas: 10 aplica, 10 no aplica.
build20() {
  { gold_head
    for i in $(seq 1 10); do gold_row "#$((100+i))" tdd aplica; done
    for i in $(seq 11 20); do gold_row "#$((100+i))" debug "no aplica"; done
  } > "$TMP/g20.md"
  { led_head
    for i in $(seq 1 10); do led_row "#$((100+i))" tdd aplica; done
    for i in $(seq 11 20); do led_row "#$((100+i))" debug "no aplica"; done
  } > "$TMP/l20.md"
}

# todo_acierta
{ gold_head; gold_row '#1943' plan aplica; gold_row '#1998' orchestrate aplica
  gold_row '#2136' brainstorm parcial; gold_row '#2318' verify "ya cubierto"; gold_row '#1959' debug "no aplica"; } > "$TMP/g.md"
cp "$TMP/g.md" "$TMP/g_as_l.md"
{ led_head; led_row 1943 plan aplica; led_row '#1998' orchestrate aplica
  led_row '#2136' brainstorm parcial; led_row '#2318' verify "ya cubierto"; led_row '#1959' debug "no aplica"; } > "$TMP/l.md"
out="$(bash "$SCORE" "$TMP/g.md" "$TMP/l.md" 2>&1)"; rc=$?
if [ $rc -eq 0 ] && grep -q 'aciertos 5/5 (100%)' <<<"$out"; then ok todo_acierta; else bad "todo_acierta rc=$rc: $out"; fi

# etiqueta con mayúsculas/espacios se normaliza
{ led_head; led_row '#1943' plan '  APLICA '; led_row '#1998' orchestrate Aplica
  led_row '#2136' brainstorm 'Parcial'; led_row '#2318' verify 'Ya  Cubierto'; led_row '#1959' debug 'No aplica'; } > "$TMP/l.md"
out="$(bash "$SCORE" "$TMP/g.md" "$TMP/l.md" 2>&1)"; rc=$?
if [ $rc -eq 0 ] && grep -q 'aciertos 5/5' <<<"$out"; then ok normaliza_etiqueta; else bad "normaliza_etiqueta rc=$rc: $out"; fi

# ya_cubierto_falso_tumba
build20
sed -i '0,/| #101 | tdd | aplica |/s//| #101 | tdd | ya cubierto |/' "$TMP/l20.md"
out="$(bash "$SCORE" "$TMP/g20.md" "$TMP/l20.md" 2>&1)"; rc=$?
if [ $rc -eq 1 ] && grep -q 'ya-cubierto-falsos 1' <<<"$out" && grep -q 'aciertos 19/20 (95%)' <<<"$out" && grep -q '101' <<<"$out"; then
  ok ya_cubierto_falso_tumba; else bad "ya_cubierto_falso_tumba rc=$rc: $out"; fi

# bajo_umbral: 8/10 sin ya-cubierto falsos
{ gold_head; for i in $(seq 1 10); do gold_row "#$i" tdd aplica; done; } > "$TMP/g10.md"
{ led_head; for i in $(seq 1 8); do led_row "#$i" tdd aplica; done
  led_row '#9' tdd "no aplica"; led_row '#10' tdd duda; } > "$TMP/l10.md"
out="$(bash "$SCORE" "$TMP/g10.md" "$TMP/l10.md" 2>&1)"; rc=$?
if [ $rc -eq 1 ] && grep -q 'aciertos 8/10 (80%)' <<<"$out" && grep -q 'ya-cubierto-falsos 0' <<<"$out"; then ok bajo_umbral; else bad "bajo_umbral rc=$rc: $out"; fi
# ... y el umbral es parametrizable
bash "$SCORE" "$TMP/g10.md" "$TMP/l10.md" 80 >/dev/null 2>&1 && ok umbral_parametrizable || bad umbral_parametrizable

# ausente_cuenta_como_fallo
build20
grep -v '^| #101 |' "$TMP/l20.md" > "$TMP/l20b.md"
out="$(bash "$SCORE" "$TMP/g20.md" "$TMP/l20b.md" 90 2>&1)"; rc=$?
if grep -q 'ausentes 1' <<<"$out" && grep -q 'aciertos 19/20' <<<"$out"; then ok ausente_cuenta_como_fallo; else bad "ausente_cuenta_como_fallo rc=$rc: $out"; fi

# revertido ausente en el ledger = acierto, y no cuenta como ausente fallido
{ gold_head; gold_row '#500' tdd aplica; gold_row '#501' tdd revertido; } > "$TMP/gr.md"
{ led_head; led_row '#500' tdd aplica; } > "$TMP/lr.md"
out="$(bash "$SCORE" "$TMP/gr.md" "$TMP/lr.md" 2>&1)"; rc=$?
if [ $rc -eq 0 ] && grep -q 'aciertos 2/2' <<<"$out"; then ok revertido_ausente_acierta; else bad "revertido_ausente_acierta rc=$rc: $out"; fi

# extra_no_puntua + release #2028 no empareja con miembros
build20
led_row '#2028' plan aplica >> "$TMP/l20.md"
out="$(bash "$SCORE" "$TMP/g20.md" "$TMP/l20.md" 2>&1)"; rc=$?
if [ $rc -eq 0 ] && grep -q 'extra 1' <<<"$out" && grep -q 'aciertos 20/20 (100%)' <<<"$out"; then ok extra_no_puntua; else bad "extra_no_puntua rc=$rc: $out"; fi

# release en el ledger, miembros ausentes: fallo
{ gold_head; gold_row '#1943' plan aplica; gold_row '#1998' orchestrate aplica; } > "$TMP/gm.md"
{ led_head; led_row '#2028' plan aplica; } > "$TMP/lm.md"
out="$(bash "$SCORE" "$TMP/gm.md" "$TMP/lm.md" 2>&1)"; rc=$?
if [ $rc -eq 1 ] && grep -q 'ausentes 2' <<<"$out" && grep -q 'extra 1' <<<"$out" && grep -q 'aciertos 0/2' <<<"$out"; then ok release_no_empareja_miembros; else bad "release_no_empareja_miembros rc=$rc: $out"; fi

# solo se usa la tabla principal (otra tabla con otra cabecera se ignora)
{ gold_head; gold_row '#1' tdd aplica; printf '\n## Confianza media\n\n| PR | nota |\n|---|---|\n| #77 | x |\n'; } > "$TMP/go.md"
{ led_head; led_row '#1' tdd aplica; } > "$TMP/lo.md"
out="$(bash "$SCORE" "$TMP/go.md" "$TMP/lo.md" 2>&1)"; rc=$?
if [ $rc -eq 0 ] && grep -q 'aciertos 1/1' <<<"$out"; then ok solo_tabla_principal; else bad "solo_tabla_principal rc=$rc: $out"; fi

# sin tabla reconocible: falla ruidoso, no 0/0 verde
printf 'nada\n' > "$TMP/vacio.md"
bash "$SCORE" "$TMP/vacio.md" "$TMP/l.md" >/dev/null 2>&1; rc=$?
[ $rc -eq 2 ] && ok sin_tabla_falla_ruidoso || bad "sin_tabla_falla_ruidoso rc=$rc"

echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
