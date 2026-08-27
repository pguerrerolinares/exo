#!/usr/bin/env bash
# Barrido de fugas de la KB semilla. NO es el gate real —ese es la revisión
# humana— pero sube el suelo. Sale 0 SI ENCUENTRA algo (o sea: 0 es malo).
set -uo pipefail
cd "$(dirname "$0")/.."
PATRON='paul|wisdom|empresa-x|cliente-a|equipo-x|cliente-c|cliente-b|redmine|universidad|lighthouse|spark|cge|solve-it|openwisdom|basic-memory|20[0-9]{2}-[0-9]{2}'
grep -rniE "$PATRON" kb-template/
