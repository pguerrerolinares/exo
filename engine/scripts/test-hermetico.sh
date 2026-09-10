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
  # Los NOMBRES de los tests que fallaron, no solo el binario. El grep
  # anterior emitía `test result: FAILED` y `--test <binario>` y se dejaba
  # fuera lo único accionable: qué test cayó y con qué aserción. Eso convertía
  # un fallo del CI en "algo de targets_cli falla" y obligaba a reproducir a
  # ciegas en una plataforma que quizá no tienes — medido el 2026-09-10, con
  # el CI de main llevando 7 corridas en rojo sin que el log dijera el nombre.
  # Un gate que no dice QUÉ falló delega el diagnóstico en quien lo lea.
  echo "--- tests que fallaron ---" >&2
  grep -E '^test .* \.\.\. FAILED$' "$TMP/out.txt" >&2 || true
  # El bloque `failures:` de cargo trae la aserción y el panic de cada uno.
  sed -n '/^failures:$/,/^test result: FAILED/p' "$TMP/out.txt" >&2 || true
  echo "--- resumen ---" >&2
  grep -E '^test result: FAILED|--test ' "$TMP/out.txt" >&2 || true
  exit 1
fi
echo "test-hermetico: OK — la suite corre sin ~/.exo/config.toml; NO cubre la caché del modelo ONNX (~0,6 GB), que las suites de indexado siguen exigiendo."
