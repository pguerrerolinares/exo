#!/usr/bin/env bash
# Tests de la guarda de preparar.sh (#192). Nada toca ~/.cache: K_ROOT en mktemp -d y git es un stub que falla
# (el test acaba justo después de la guarda; no clona la KB ni corre exo).
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
fail=0
ok(){ echo "ok   $1"; }; ko(){ echo "FAIL $1"; fail=1; }

mkdir -p "$T/bin"; printf '#!/bin/sh\nexit 1\n' > "$T/bin/git"; chmod +x "$T/bin/git"
export K_ROOT="$T/k"; P="$K_ROOT/prep"
nuevo_prep() { rm -rf "$K_ROOT"; mkdir -p "$P/kb"; echo "kb_commit viejo" > "$P/manifiesto.txt"; echo nota > "$P/kb/n.md"; }
prep() { PATH="$T/bin:$PATH" bash "$HERE/preparar.sh" "$@" 2>&1; }

# preparar_se_niega_sin_forzar: con manifiesto.txt sale 2, el prep queda intacto y no se mueve nada
nuevo_prep
out=$(prep abc123); rc=$?
if [ $rc = 2 ] && grep -q -- '--forzar' <<<"$out" && [ "$(cat "$P/manifiesto.txt")" = "kb_commit viejo" ] && [ -f "$P/kb/n.md" ] \
   && ! compgen -G "$K_ROOT/prep.pisado-*" >/dev/null; then ok preparar_se_niega_sin_forzar; else ko "preparar_se_niega_sin_forzar rc=$rc out=$out"; fi

# preparar_forzar_mueve_no_borra: el prep viejo queda entero en prep.pisado-<fecha>
nuevo_prep
out=$(prep --forzar abc123); rc=$?
mov=$(compgen -G "$K_ROOT/prep.pisado-*" | head -1)
if [ $rc != 0 ] && [ -n "$mov" ] && [ "$(cat "$mov/manifiesto.txt")" = "kb_commit viejo" ] && [ -f "$mov/kb/n.md" ] && [ ! -f "$P/manifiesto.txt" ]; then
  ok preparar_forzar_mueve_no_borra; else ko "preparar_forzar_mueve_no_borra rc=$rc mov=$mov out=$out"; fi

# preparar_sin_manifiesto_no_se_guarda: un prep a medias (sin manifiesto) se rehace como siempre
nuevo_prep; rm "$P/manifiesto.txt"
out=$(prep abc123); rc=$?
if ! grep -q -- '--forzar' <<<"$out" && [ ! -f "$P/kb/n.md" ] && ! compgen -G "$K_ROOT/prep.pisado-*" >/dev/null; then ok preparar_sin_manifiesto_no_se_guarda; else ko "preparar_sin_manifiesto_no_se_guarda rc=$rc out=$out"; fi

exit $fail
