#!/usr/bin/env bash
# Test de _timeout.sh: con_timeout no debe usar el TIMEOUT.EXE de Windows.
# El fake imita a TIMEOUT.EXE (exit 1 sin ejecutar nada) y vive bajo una ruta
# .../Windows/System32/timeout, la forma que resuelve `command -v` en Git Bash
# (/c/WINDOWS/system32/timeout). Así el caso se reproduce también en Linux/macOS.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$TMP/c/Windows/System32"
printf '#!/bin/sh\necho "ERROR: Sintaxis no valida" >&2\nexit 1\n' >"$TMP/c/Windows/System32/timeout"
chmod +x "$TMP/c/Windows/System32/timeout"

# Subshell: el PATH falso no se filtra a las demás aserciones.
corre() { ( PATH="$TMP/c/Windows/System32:$PATH"; . "$SCRIPT_DIR/_timeout.sh"; "$@" ); }

PASS=0
FAIL=0
check() { # nombre esperado obtenido
  if [ "$2" = "$3" ]; then printf '[PASS] %s\n' "$1"; PASS=$((PASS+1))
  else printf '[FAIL] %s — esperaba «%s», obtuve «%s»\n' "$1" "$2" "$3"; FAIL=$((FAIL+1)); fi
}

corre con_timeout 1 sleep 3 >/dev/null 2>&1
check timeout_exe_windows_corta_con_124 124 "$?"

out="$(corre con_timeout 5 sh -c 'echo hecho')"
check timeout_exe_windows_ejecuta_el_comando hecho "$out"

corre con_timeout 5 sh -c 'exit 7' >/dev/null 2>&1
check timeout_exe_windows_propaga_exit 7 "$?"

# Camino normal, sin fake: no se rompe.
( . "$SCRIPT_DIR/_timeout.sh"; con_timeout 1 sleep 3 ) >/dev/null 2>&1
check timeout_normal_corta_con_124 124 "$?"

printf '\n%d pass, %d fail\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
