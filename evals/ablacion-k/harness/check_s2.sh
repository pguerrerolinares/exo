#!/usr/bin/env bash
# check_s2.sh <dir tarea gold> <workdir> — check común de S2 (errata E5.4).
# Copia sobre el workdir los tests que añadió el commit de referencia (tests_ref/) y los
# ejecuta con el venv del repo. 0 = pasan (tarea resuelta) · 1 = fallan.
set -uo pipefail
g=$1; w=$2
K_ROOT="${K_ROOT:-$HOME/.cache/exo-ablacion-k}"
repo=$(jq -r .repo "$g/meta.json"); py="$K_ROOT/s2/venv-$repo/bin/python"
mapfile -t tests < <(jq -r '.tests[]' "$g/meta.json")
cp -a "$g/tests_ref/." "$w/"
cd "$w" || exit 1
if [ "$repo" = wagtail ]; then
  labels=(); for t in "${tests[@]}"; do labels+=("$(echo "${t%.py}" | tr / .)"); done
  timeout 900 "$py" runtests.py "${labels[@]}" --parallel 1 > /dev/null 2>&1
else
  DATABASE_ENGINE=django.db.backends.sqlite3 DATABASE_NAME=:memory: \
    timeout 900 "$py" -m pytest -q -p no:cacheprovider "${tests[@]}" > /dev/null 2>&1
fi
[ $? -eq 0 ] && exit 0 || exit 1
