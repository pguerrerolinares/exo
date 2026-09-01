#!/usr/bin/env bash
# Barrido de fugas de la KB semilla. NO es el gate real —ese es la revisión
# humana— pero sube el suelo. Sale 0 SI ENCUENTRA algo (o sea: 0 es malo).
#
# Los patrones NO viven en este repo: la propia lista de qué se busca (nombres
# de clientes y proyectos privados) sería ella misma una fuga si se publicara
# verbatim. Viven en un fichero fuera del repo, un patrón regex-ERE por
# línea, líneas en blanco ignoradas; se unen con `|` y se pasan a `grep -E`.
# Ejemplo de dos líneas del fichero (genérico a propósito: poner aquí un
# patrón real reintroduciría en el repo justo lo que este diseño saca de él):
#   nombre-de-cliente
#   20[0-9]{2}-[0-9]{2}
#
# Ruta por defecto: $HOME/.config/exo/fugas.patterns — override con
# EXO_FUGAS_PATTERNS (para tests y CI, con un fichero temporal inyectable).
# En la máquina Linux hay que crear el mismo fichero a mano (o inyectarlo vía
# EXO_FUGAS_PATTERNS); no se distribuye ni con el repo ni con el plugin.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1

PATTERNS_FILE="${EXO_FUGAS_PATTERNS:-$HOME/.config/exo/fugas.patterns}"
if [[ ! -f "$PATTERNS_FILE" ]]; then
  echo "fugas-semilla: no encuentro el fichero de patrones ($PATTERNS_FILE)." >&2
  echo "Formato esperado: un patrón regex-ERE por línea (se combinan con OR)." >&2
  echo "Créalo antes de correr este gate. Sin él NO se puede afirmar que el árbol está limpio — eso sería exactamente el fallo silencioso que este script existe para impedir." >&2
  exit 3
fi

PATRON="$(grep -v '^[[:space:]]*$' "$PATTERNS_FILE" | paste -sd '|' -)"
if [[ -z "$PATRON" ]]; then
  echo "fugas-semilla: $PATTERNS_FILE existe pero no contiene ningún patrón (vacío)." >&2
  exit 3
fi

grep -rniE "$PATRON" kb-template/
