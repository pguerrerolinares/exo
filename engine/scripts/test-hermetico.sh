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

# EXO_HERMETICO_LOG (opt-in, campaña E, docs/backlog.md "Un rojo del job test
# no se puede diagnosticar desde el CI"): por defecto el log vive en $TMP y
# muere con el trap de arriba — sirve para diagnosticar EN el momento, no
# después de que el proceso termine. El CI lo fija a una ruta FUERA de $TMP
# para poder subirlo con actions/upload-artifact tras un fallo.
LOG="${EXO_HERMETICO_LOG:-$TMP/out.txt}"

# --locked: `lint`/`msrv` ya lo tenían; el job que de verdad EJECUTA la
# suite (y release.yml, que llama a este mismo script) no. Sin esto, el job
# más importante podía resolver un árbol de dependencias distinto del
# Cargo.lock commiteado sin que nada lo dijera.
#
# `tee`, no una redirección silenciosa: el log queda en disco EN VIVO (para
# `upload-artifact` si el runner muere a mitad) y en pantalla/stdout de quien
# corre esto a mano. `PIPESTATUS[0]`: el exit code de `cargo test`, no el de
# `tee` — explícito, no depende de que a alguien se le ocurra quitar
# `set -o pipefail` de la línea de arriba en un cambio futuro.
EXO_CONFIG="$TMP/no-existe.toml" cargo test --release --locked --no-fail-fast 2>&1 | tee "$LOG"
EC=${PIPESTATUS[0]}

if [ "$EC" -ne 0 ]; then
  echo "test-hermetico: la suite NO corre sin ~/.exo/config.toml (exit $EC)." >&2
  # Los NOMBRES de los tests que fallaron, no solo el binario — un gate que
  # no dice QUÉ falló delega el diagnóstico en quien lo lea (medido
  # 2026-09-10, con el CI de main llevando 7 corridas en rojo sin que el log
  # dijera el nombre).
  echo "--- tests que fallaron ---" >&2
  grep -E '^test .* \.\.\. FAILED$' "$LOG" >&2 || true
  sed -n '/^failures:$/,/^test result: FAILED/p' "$LOG" >&2 || true
  echo "--- resumen ---" >&2
  grep -E '^test result: FAILED|--test ' "$LOG" >&2 || true
  # F5 (docs/backlog.md:732-765, "Sigue abierto: un error de COMPILACIÓN de
  # la suite sigue sin casar ningún patrón de grep"): distingue un error de
  # rustc/cargo de un fallo de test normal. `error[EXXXX]:`/`-->` son la
  # forma de un diagnóstico de rustc; `error: could not compile` es el
  # resumen final de cargo cuando la compilación no llega a producir el
  # binario de test — ninguno de los dos aparece en un log de solo tests
  # que fallan (verificado con una muestra sintética de cada caso).
  if grep -qE '^error(\[E[0-9]+\])?:|error: could not compile' "$LOG"; then
    echo "--- error de compilación ---" >&2
    grep -E '^error(\[E[0-9]+\])?:|-->|error: could not compile' "$LOG" >&2 || true
  fi
  exit 1
fi
echo "test-hermetico: OK — la suite corre sin ~/.exo/config.toml, con --locked; NO cubre la caché del modelo ONNX (~0,6 GB), que las suites de indexado siguen exigiendo."
