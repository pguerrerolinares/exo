#!/usr/bin/env bash
# Gate: corre TODOS los tests del plugin (plugins/exo/scripts/test-*.sh).
#
# Descubre por glob, no por lista: test-estilo-directo.sh existía, fallaba
# 0/2 (exit 126) y aun así el hook llegó a release en exo 1.1.1, porque
# ningún CI corría los tests del plugin. Con una lista a mano, el siguiente
# test nuevo quedaría igual de huérfano.
#
# Excluido: test-contrato-engine.sh, que necesita el binario compilado, un
# índice y una KB. Lo corre scripts/test-contrato-ci.sh en el job `test` de
# CI, que ya compila el engine y cachea el modelo ONNX.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1

EXCLUIDOS=" test-contrato-engine.sh "

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

total=0
fallidos=()
for t in plugins/exo/scripts/test-*.sh; do
  [ -e "$t" ] || continue
  nombre="$(basename "$t")"
  case "$EXCLUIDOS" in *" $nombre "*) echo "[EXCLUIDO] $nombre"; continue ;; esac
  total=$((total + 1))
  # Invocación directa, no `bash "$t"`: así el test también ejerce el bit de
  # ejecución, que es como los invoca el harness de Claude Code.
  "./$t" > "$TMP/out.txt" 2>&1
  ec=$?
  if [ "$ec" -eq 0 ]; then
    echo "[PASS] $nombre — $(tail -n 1 "$TMP/out.txt")"
  else
    echo "[FAIL] $nombre (exit $ec) — salida completa:"
    sed 's/^/    /' "$TMP/out.txt"
    fallidos+=("$nombre")
  fi
done

if [ "$total" -eq 0 ]; then
  # Un glob que no matchea nada daría verde sin haber probado nada.
  echo "test-plugin: no se encontró ningún test — el glob está roto" >&2
  exit 1
fi

if [ "${#fallidos[@]}" -gt 0 ]; then
  echo "test-plugin: ${#fallidos[@]}/${total} suites fallaron: ${fallidos[*]}" >&2
  exit 1
fi
echo "test-plugin: OK — ${total}/${total} suites del plugin en verde"
