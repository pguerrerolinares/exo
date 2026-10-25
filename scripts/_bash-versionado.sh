#!/usr/bin/env bash
# Helper COMPARTIDO: descubre el bash versionado (índice de git: *.sh más
# ejecutables sin extensión con shebang sh/bash; excluye evals/ y docs/).
# Lo usan test-shellcheck.sh y test-rutas-personales.sh, que llevaban este
# bucle copiado línea a línea — si divergen, los dos gates dejan de mirar
# lo mismo sin avisar. Mismo patrón que plugins/exo/scripts/_truncate-payload.sh.
#
# Uso:  . "$(dirname "$0")/_bash-versionado.sh" && bash_versionado [--con-nuevos]
# --con-nuevos suma los no trackeados y no ignorados (`git ls-files --others
# --exclude-standard`): el .sh recién creado que el executor revisa antes de
# `git add`. En CI (checkout limpio) no hay ninguno y el resultado no cambia.
# Asigna el array global `ficheros` en vez de imprimir (un array no pasa
# por $(...)). Exige cwd = raíz del repo: los llamadores ya hacen el cd.
# shellcheck disable=SC2034 # ficheros es la salida: la lee el script que hace source
bash_versionado() {
  ficheros=()
  local con_nuevos=0
  [ "${1:-}" = "--con-nuevos" ] && con_nuevos=1
  while read -r modo blob _etapa ruta; do
    case "$ruta" in evals/*|docs/*) continue ;; esac
    case "$ruta" in
      *.sh) ficheros+=("$ruta") ;;
      *.*) : ;;
      *)
        [ "$modo" = "100755" ] || continue
        # sed -n 1p, no head -n 1: head cierra la tubería, cat-file muere
        # por SIGPIPE y pipefail omite el script sin avisar (#65).
        if git cat-file -p "$blob" | sed -n 1p | grep -Eq '^#!.*[/ ](ba)?sh([[:space:]]|$)'; then
          ficheros+=("$ruta")
        fi
        ;;
    esac
  done < <(git ls-files -s)
  [ "$con_nuevos" = 1 ] || return 0
  local f ruta
  # Se deduplica contra lo ya listado: el índice puede cambiar entre las dos llamadas a git.
  while IFS= read -r -d '' ruta; do
    case "$ruta" in evals/*|docs/*) continue ;; esac
    [ -f "$ruta" ] || continue
    for f in "${ficheros[@]}"; do [ "$f" = "$ruta" ] && continue 2; done
    case "$ruta" in
      *.sh) ficheros+=("$ruta") ;;
      *.*) : ;;
      *)
        [ -x "$ruta" ] || continue
        if sed -n 1p "$ruta" | grep -Eq '^#!.*[/ ](ba)?sh([[:space:]]|$)'; then
          ficheros+=("$ruta")
        fi
        ;;
    esac
  done < <(git ls-files --others --exclude-standard -z)
}
