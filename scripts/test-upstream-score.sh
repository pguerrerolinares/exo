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

gold_head() { printf '# gold\n\n| PR | título | base | skill exo | etiqueta | evidencia | conf. |\n|---|---|---|---|---|---|---|\n'; }
gold_row()  { printf '| %s | titulo x | dev | %s | %s | ev | alta |\n' "$1" "$2" "$3"; }
# led_head <verdad.md>: el sello es el sha256 de los bytes de esa verdad.
led_head()  { printf 'upstream_tag: v9.9.9\nverdad_sello: %s v0..v1\n\n| PR | skill | triage | estado | motivo | hash |\n|---|---|---|---|---|---|\n' "$(sha256sum "$1" | cut -d' ' -f1)"; }
led_row()   { printf '| %s | %s | %s | portado | m | abc123 |\n' "$1" "$2" "$3"; }

# Verdad base de 20 filas: 10 aplica, 10 no aplica.
build20() {
  { gold_head
    for i in $(seq 1 10); do gold_row "#$((100+i))" tdd aplica; done
    for i in $(seq 11 20); do gold_row "#$((100+i))" debug "no aplica"; done
  } > "$TMP/g20.md"
  { led_head "$TMP/g20.md"
    for i in $(seq 1 10); do led_row "#$((100+i))" tdd aplica; done
    for i in $(seq 11 20); do led_row "#$((100+i))" debug "no aplica"; done
  } > "$TMP/l20.md"
}

# todo_acierta
{ gold_head; gold_row '#9001' plan aplica; gold_row '#9002' orchestrate aplica
  gold_row '#9003' brainstorm parcial; gold_row '#9004' verify "ya cubierto"; gold_row '#9005' debug "no aplica"; } > "$TMP/g.md"
{ led_head "$TMP/g.md"; led_row 9001 plan aplica; led_row '#9002' orchestrate aplica
  led_row '#9003' brainstorm parcial; led_row '#9004' verify "ya cubierto"; led_row '#9005' debug "no aplica"; } > "$TMP/l.md"
out="$(bash "$SCORE" "$TMP/g.md" "$TMP/l.md" 2>&1)"; rc=$?
if [ $rc -eq 0 ] && grep -q 'aciertos 5/5 (100%)' <<<"$out"; then ok todo_acierta; else bad "todo_acierta rc=$rc: $out"; fi

# etiqueta con mayúsculas/espacios se normaliza
{ led_head "$TMP/g.md"; led_row '#9001' plan '  APLICA '; led_row '#9002' orchestrate Aplica
  led_row '#9003' brainstorm 'Parcial'; led_row '#9004' verify 'Ya  Cubierto'; led_row '#9005' debug 'No aplica'; } > "$TMP/l.md"
out="$(bash "$SCORE" "$TMP/g.md" "$TMP/l.md" 2>&1)"; rc=$?
if [ $rc -eq 0 ] && grep -q 'aciertos 5/5' <<<"$out"; then ok normaliza_etiqueta; else bad "normaliza_etiqueta rc=$rc: $out"; fi

# ya_cubierto_falso_tumba
build20
sed -i '0,/| #101 | tdd | aplica |/s//| #101 | tdd | ya cubierto |/' "$TMP/l20.md"
out="$(bash "$SCORE" "$TMP/g20.md" "$TMP/l20.md" 2>&1)"; rc=$?
if [ $rc -eq 1 ] && grep -q 'no-portar-falsos 1' <<<"$out" && grep -q 'aciertos 19/20 (95%)' <<<"$out" && grep -q '101' <<<"$out"; then
  ok ya_cubierto_falso_tumba; else bad "ya_cubierto_falso_tumba rc=$rc: $out"; fi

# bajo_umbral: 8/10 sin no-portar-falsos
{ gold_head; for i in $(seq 1 10); do gold_row "#$i" tdd aplica; done; } > "$TMP/g10.md"
{ led_head "$TMP/g10.md"; for i in $(seq 1 8); do led_row "#$i" tdd aplica; done
  led_row '#9' tdd duda; led_row '#10' tdd duda; } > "$TMP/l10.md"
out="$(bash "$SCORE" "$TMP/g10.md" "$TMP/l10.md" 2>&1)"; rc=$?
if [ $rc -eq 1 ] && grep -q 'aciertos 8/10 (80%)' <<<"$out" && grep -q 'no-portar-falsos 0' <<<"$out"; then ok bajo_umbral; else bad "bajo_umbral rc=$rc: $out"; fi
# ... y el umbral es parametrizable
if bash "$SCORE" "$TMP/g10.md" "$TMP/l10.md" 80 >/dev/null 2>&1; then ok umbral_parametrizable; else bad umbral_parametrizable; fi

# ausente_cuenta_como_fallo
build20
grep -v '^| #101 |' "$TMP/l20.md" > "$TMP/l20b.md"
out="$(bash "$SCORE" "$TMP/g20.md" "$TMP/l20b.md" 90 2>&1)"; rc=$?
if grep -q 'ausentes 1' <<<"$out" && grep -q 'aciertos 19/20' <<<"$out"; then ok ausente_cuenta_como_fallo; else bad "ausente_cuenta_como_fallo rc=$rc: $out"; fi

# revertido ausente en el ledger = acierto, y no cuenta como ausente fallido
{ gold_head; gold_row '#500' tdd aplica; gold_row '#501' tdd revertido; } > "$TMP/gr.md"
{ led_head "$TMP/gr.md"; led_row '#500' tdd aplica; } > "$TMP/lr.md"
out="$(bash "$SCORE" "$TMP/gr.md" "$TMP/lr.md" 2>&1)"; rc=$?
if [ $rc -eq 0 ] && grep -q 'aciertos 2/2' <<<"$out"; then ok revertido_ausente_acierta; else bad "revertido_ausente_acierta rc=$rc: $out"; fi

# aplica y parcial son una sola clase (misma acción: portar)
{ gold_head; gold_row '#9101' plan aplica; gold_row '#9102' orchestrate parcial; } > "$TMP/ge.md"
{ led_head "$TMP/ge.md"; led_row '#9101' plan parcial; led_row '#9102' orchestrate aplica; } > "$TMP/le.md"
out="$(bash "$SCORE" "$TMP/ge.md" "$TMP/le.md" 2>&1)"; rc=$?
if [ $rc -eq 0 ] && grep -q 'aciertos 2/2 (100%)' <<<"$out"; then ok aplica_parcial_equivalen; else bad "aplica_parcial_equivalen rc=$rc: $out"; fi

# ya cubierto sigue siendo clase propia: verdad parcial + ledger ya cubierto tumba
{ gold_head; gold_row '#9103' plan parcial; } > "$TMP/gy.md"
{ led_head "$TMP/gy.md"; led_row '#9103' plan "ya cubierto"; } > "$TMP/ly.md"
out="$(bash "$SCORE" "$TMP/gy.md" "$TMP/ly.md" 2>&1)"; rc=$?
if [ $rc -eq 1 ] && grep -q 'no-portar-falsos 1' <<<"$out" && grep -q 'aciertos 0/1' <<<"$out"; then ok ya_cubierto_sigue_estricto; else bad "ya_cubierto_sigue_estricto rc=$rc: $out"; fi

# extra_no_puntua + release #9006 no empareja con miembros
build20
led_row '#9006' plan aplica >> "$TMP/l20.md"
out="$(bash "$SCORE" "$TMP/g20.md" "$TMP/l20.md" 2>&1)"; rc=$?
if [ $rc -eq 0 ] && grep -q 'extra 1' <<<"$out" && grep -q 'aciertos 20/20 (100%)' <<<"$out"; then ok extra_no_puntua; else bad "extra_no_puntua rc=$rc: $out"; fi

# release en el ledger, miembros ausentes: fallo
{ gold_head; gold_row '#9001' plan aplica; gold_row '#9002' orchestrate aplica; } > "$TMP/gm.md"
{ led_head "$TMP/gm.md"; led_row '#9006' plan aplica; } > "$TMP/lm.md"
out="$(bash "$SCORE" "$TMP/gm.md" "$TMP/lm.md" 2>&1)"; rc=$?
if [ $rc -eq 1 ] && grep -q 'ausentes 2' <<<"$out" && grep -q 'extra 1' <<<"$out" && grep -q 'aciertos 0/2' <<<"$out"; then ok release_no_empareja_miembros; else bad "release_no_empareja_miembros rc=$rc: $out"; fi

# solo se usa la tabla principal (otra tabla con otra cabecera se ignora)
{ gold_head; gold_row '#1' tdd aplica; printf '\n## Confianza media\n\n| PR | nota |\n|---|---|\n| #77 | x |\n'; } > "$TMP/go.md"
{ led_head "$TMP/go.md"; led_row '#1' tdd aplica; } > "$TMP/lo.md"
out="$(bash "$SCORE" "$TMP/go.md" "$TMP/lo.md" 2>&1)"; rc=$?
if [ $rc -eq 0 ] && grep -q 'aciertos 1/1' <<<"$out"; then ok solo_tabla_principal; else bad "solo_tabla_principal rc=$rc: $out"; fi

# sin tabla reconocible: falla ruidoso, no 0/0 verde
printf 'nada\n' > "$TMP/vacio.md"
bash "$SCORE" "$TMP/vacio.md" "$TMP/l.md" >/dev/null 2>&1; rc=$?
if [ $rc -eq 2 ]; then ok sin_tabla_falla_ruidoso; else bad "sin_tabla_falla_ruidoso rc=$rc"; fi

# umbral inválido: no numérico o fuera de rango -> exit 2
{ gold_head; gold_row '#9201' tdd aplica; } > "$TMP/gu.md"
{ led_head "$TMP/gu.md"; led_row '#9201' tdd aplica; } > "$TMP/lu.md"
for u in '90%' abc 101 ''; do
  bash "$SCORE" "$TMP/gu.md" "$TMP/lu.md" "$u" >"$TMP/o.txt" 2>&1; rc=$?
  if [ $rc -eq 2 ] && grep -q "umbral inválido: $u" "$TMP/o.txt"; then ok "umbral_invalido[$u]"; else bad "umbral_invalido[$u] rc=$rc: $(cat "$TMP/o.txt")"; fi
done

# fila malformada (celda vacía o menos columnas) en verdad o ledger -> exit 2
{ gold_head; gold_row '#1' tdd aplica; printf '| #2 | t | dev | | aplica | ev | alta |\n'; } > "$TMP/gmal.md"
{ led_head "$TMP/gmal.md"; led_row '#1' tdd aplica; } > "$TMP/lok.md"
out="$(bash "$SCORE" "$TMP/gmal.md" "$TMP/lok.md" 2>&1)"; rc=$?
if [ $rc -eq 2 ] && grep -q 'fila malformada en .*gmal.md' <<<"$out"; then ok fila_malformada_verdad; else bad "fila_malformada_verdad rc=$rc: $out"; fi
{ gold_head; gold_row '#1' tdd aplica; } > "$TMP/gok.md"
{ led_head "$TMP/gok.md"; led_row '#1' tdd aplica; printf '| #2 | tdd |\n'; } > "$TMP/lmal.md"
out="$(bash "$SCORE" "$TMP/gok.md" "$TMP/lmal.md" 2>&1)"; rc=$?
if [ $rc -eq 2 ] && grep -q 'fila malformada en .*lmal.md' <<<"$out"; then ok fila_malformada_ledger; else bad "fila_malformada_ledger rc=$rc: $out"; fi

# no aplica falso: 19/20 = 95% pasaría el umbral, pero la clase lo tumba
build20
sed -i '0,/| #101 | tdd | aplica |/s//| #101 | tdd | no aplica |/' "$TMP/l20.md"
out="$(bash "$SCORE" "$TMP/g20.md" "$TMP/l20.md" 2>&1)"; rc=$?
if [ $rc -eq 1 ] && grep -qF 'no-portar-falsos 1 (ya cubierto: 0, no aplica: 1)' <<<"$out" && grep -qF '101 tdd (verdad: aplica)' <<<"$out"; then
  ok no_aplica_falso_tumba; else bad "no_aplica_falso_tumba rc=$rc: $out"; fi

# ya cubierto falso conserva su subclase
build20
sed -i '0,/| #101 | tdd | aplica |/s//| #101 | tdd | ya cubierto |/' "$TMP/l20.md"
out="$(bash "$SCORE" "$TMP/g20.md" "$TMP/l20.md" 2>&1)"; rc=$?
if [ $rc -eq 1 ] && grep -qF 'no-portar-falsos 1 (ya cubierto: 1, no aplica: 0)' <<<"$out"; then
  ok ya_cubierto_falso_sigue_tumbando; else bad "ya_cubierto_falso_sigue_tumbando rc=$rc: $out"; fi

# no aplica con verdad no aplica es acierto (build20 ya lo cubre en 10 filas)
{ gold_head; gold_row '#1' debug "no aplica"; } > "$TMP/gn.md"
{ led_head "$TMP/gn.md"; led_row '#1' debug "no aplica"; } > "$TMP/ln.md"
out="$(bash "$SCORE" "$TMP/gn.md" "$TMP/ln.md" 2>&1)"; rc=$?
if [ $rc -eq 0 ] && grep -qF 'no-portar-falsos 0' <<<"$out" && grep -q 'aciertos 1/1' <<<"$out"; then ok no_aplica_con_verdad_no_aplica_acierta; else bad "no_aplica_con_verdad_no_aplica_acierta rc=$rc: $out"; fi

# verdad parcial + ledger no aplica
{ gold_head; gold_row '#1' plan parcial; } > "$TMP/gp.md"
{ led_head "$TMP/gp.md"; led_row '#1' plan "no aplica"; } > "$TMP/lp.md"
out="$(bash "$SCORE" "$TMP/gp.md" "$TMP/lp.md" 2>&1)"; rc=$?
if [ $rc -eq 1 ] && grep -qF 'no-portar-falsos 1' <<<"$out"; then ok parcial_en_verdad_tambien_cuenta; else bad "parcial_en_verdad_tambien_cuenta rc=$rc: $out"; fi

# sello: ausente, guion, de otra verdad -> exit 2
{ gold_head; gold_row '#1' tdd aplica; } > "$TMP/gs.md"
{ gold_head; gold_row '#2' tdd aplica; } > "$TMP/gs2.md"
sha_s="$(sha256sum "$TMP/gs.md" | cut -d' ' -f1)"
sha_o="$(sha256sum "$TMP/gs2.md" | cut -d' ' -f1)"
ledger_con() { # <línea de sello o vacío>
  { printf 'upstream_tag: v9.9.9\n'; [ -n "$1" ] && printf '%b\n' "$1"; printf '\n| PR | skill | triage | estado | motivo | hash |\n|---|---|---|---|---|---|\n'; led_row '#1' tdd aplica; } > "$TMP/lsel.md"
}
chk_sello() { # <nombre> <línea> <rc esperado> [<texto stderr>]
  ledger_con "$2"
  out="$(bash "$SCORE" "$TMP/gs.md" "$TMP/lsel.md" 2>&1)"; rc=$?
  if [ $rc -eq "$3" ] && { [ -z "${4-}" ] || grep -qF "$4" <<<"$out"; }; then ok "$1"; else bad "$1 rc=$rc: $out"; fi
}
chk_sello sello_ausente_exit2 '' 2 'verdad sin sellar'
chk_sello sello_guion_exit2 'verdad_sello: —' 2 'verdad sin sellar'
chk_sello sello_distinto_exit2 "verdad_sello: $sha_o v0..v1" 2 'verdad sin sellar'
chk_sello sello_con_espacio_final_vale "verdad_sello: $sha_s v0..v1   " 0
chk_sello sello_duplicado_usa_la_primera "verdad_sello: $sha_s v0..v1\nverdad_sello: basura" 0
chk_sello sello_duplicado_primera_mala "verdad_sello: $sha_o v0..v1\nverdad_sello: $sha_s v0..v1" 2 'verdad sin sellar'

echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
