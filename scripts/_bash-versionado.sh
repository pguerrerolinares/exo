#!/usr/bin/env bash
# Helper COMPARTIDO: descubre el bash versionado (índice de git: *.sh más
# ejecutables sin extensión con shebang sh/bash; excluye evals/ y docs/).
# Lo usan test-shellcheck.sh y test-rutas-personales.sh, que llevaban este
# bucle copiado línea a línea — si divergen, los dos gates dejan de mirar
# lo mismo sin avisar. Mismo patrón que plugins/exo/scripts/_truncate-payload.sh.
#
# Uso:  . "$(dirname "$0")/_bash-versionado.sh" && bash_versionado
# Asigna el array global `ficheros` en vez de imprimir (un array no pasa
# por $(...)). Exige cwd = raíz del repo: los llamadores ya hacen el cd.
# shellcheck disable=SC2034 # ficheros es la salida: la lee el script que hace source
bash_versionado() {
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
}
