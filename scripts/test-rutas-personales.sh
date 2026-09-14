#!/usr/bin/env bash
# Gate: ninguna ruta de una máquina concreta en el bash versionado.
#
# Decisión de Paul: SOLO rutas (/home/<user>, /Users/<user>,
# C:\Users\<user> o su forma C:/Users/<user>), nunca nombres propios — "Paul"
# o "kb-demo" sueltos NO cuentan, así que no hace falta tocar los fixtures de
# test-recall-inject.sh (EXO_KB_NAME="kb-demo", etc.).
#
# Precedente: affaan-m/ECC (clone 5064474d4d762dc9640234a41617cccb79185cec,
# scripts/ci/validate-no-personal-paths.js:41-42) cubre /Users/<nombre> y
# C:\Users\<nombre> pero NO /home/<user> — justo la única ofensora real de
# este repo (test-git-c-bash.sh). Copiarlo tal cual no bastaba.
#
# Mismo descubrimiento que test-shellcheck.sh: índice de git, .sh + shebang
# sh/bash, excluye evals/ (harness congelado — cita rutas de corridas
# pasadas como evidencia, no las produce) y docs/ (backlog.md documenta
# estas mismas rutas como HALLAZGOS, con cita; no son las que produce el
# hook).
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

PATRON='(/home/[A-Za-z0-9_.-]+|/Users/[A-Za-z0-9_.-]+|C:[\\/]Users[\\/][A-Za-z0-9_.-]+)'

ficheros=()
while read -r modo blob _etapa ruta; do
  case "$ruta" in evals/*|docs/*) continue ;; esac
  case "$ruta" in
    *.sh) ficheros+=("$ruta") ;;
    *.*) : ;;
    *)
      [ "$modo" = "100755" ] || continue
      if git cat-file -p "$blob" | head -n 1 | grep -Eq '^#!.*[/ ](ba)?sh([[:space:]]|$)'; then
        ficheros+=("$ruta")
      fi
      ;;
  esac
done < <(git ls-files -s)

if [ "${#ficheros[@]}" -eq 0 ]; then
  echo "test-rutas-personales: no se encontró ningún script — el recorrido está roto" >&2
  exit 1
fi

hallados=""
for f in "${ficheros[@]}"; do
  # `grep` distingue tres casos por exit code: 0 = hallazgo, 1 = limpio,
  # >=2 = error (fichero no legible, etc.). `if match=$(grep …)` trataba 1
  # y >=2 como el mismo "no hallazgo" — un error de lectura pasaba el gate
  # en silencio, justo el patrón de fallo silencioso que este gate existe
  # para cazar en otros.
  match="$(grep -EnH "$PATRON" "$f" 2>&1)"
  ec=$?
  case "$ec" in
    0) hallados="${hallados}${match}"$'\n' ;;
    1) : ;; # limpio: sin hallazgo, sin error
    *)
      echo "test-rutas-personales: error leyendo $f (grep exit=$ec): $match" >&2
      exit 1
      ;;
  esac
done

if [ -n "$hallados" ]; then
  echo "test-rutas-personales: rutas de una máquina concreta en bash versionado:" >&2
  printf '%s' "$hallados" >&2
  echo "Arreglo: sustituye por una ruta genérica fuera de /home, /Users o C:\\Users (p.ej. /opt/proyectos/...) o por una variable." >&2
  exit 1
fi

echo "test-rutas-personales: OK — ${#ficheros[@]} scripts sin rutas personales"
