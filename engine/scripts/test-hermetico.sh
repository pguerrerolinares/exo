#!/usr/bin/env bash
# Gate: la suite tiene que correr sin `~/.exo/config.toml`. Sin esto, el CI de
# G5 en un runner limpio nace rojo y nadie se entera hasta que el runner existe.
#
# Apunta EXO_CONFIG a un fichero inexistente en vez de mover el config real:
# mover el de la máquina es destructivo y compite con el hook `Stop` que indexa.
set -uo pipefail
# `|| exit 1` no es ceremonia: sin el, un cd fallido dejaria a cargo corriendo
# en el cwd de invocacion, y si ese directorio tuviera su propio Cargo.toml el
# gate daria verde sin haber probado nada. Hoy no lo tiene — pero eso es una
# garantia del layout, no del script.
cd "$(dirname "$0")/.." || exit 1

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# Sin tubería: el exit code de una tubería es el del ÚLTIMO comando, no el de
# cargo. Ese error dio un falso verde midiendo esta misma deuda (2026-08-27).
EXO_CONFIG="$TMP/no-existe.toml" cargo test --release --no-fail-fast > "$TMP/out.txt" 2>&1
EC=$?

if [ "$EC" -ne 0 ]; then
  echo "test-hermetico: la suite NO corre sin ~/.exo/config.toml (exit $EC)." >&2
  grep -E '^test result: FAILED|targets failed|--test ' "$TMP/out.txt" >&2
  exit 1
fi
echo "test-hermetico: OK — la suite corre sin config global."
