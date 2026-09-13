#!/usr/bin/env bash
# Gate: shellcheck sobre todo el bash versionado que se publica o que corre CI.
#
# Por qué: ~4.700 líneas de bash sin análisis estático. Este gate caza una
# clase concreta de rotura — comillas, globs, `$?` indirecto, variables sin
# usar — vía análisis ESTÁTICO de ficheros `.sh` versionados; no ejecuta nada,
# así que no cubre portabilidad GNU/BSD (shellcheck no sabe que `timeout`,
# `date -d`, `stat -c` o `sha256sum` son GNU-only) ni el bash inline de
# `run:` en `.github/workflows/*.yml` (no son ficheros `.sh`, ver «Qué NO
# entra» abajo). Tres roturas de shell reales, cada una cazada por un gate
# distinto (o por ninguno):
# - `estilo-directo.sh` sin bit de ejecución llegó a release (exo 1.1.1):
#   la caza `test-exec-bit.sh`, no este gate.
# - GNU-ismos portando mal a macOS (`timeout`, `date -d`, `stat -c`,
#   `touch -d` en `recall-inject.sh` y otros hooks del plugin) rompían el
#   plugin en `macos-latest`, degradando en cada prompt sin avisar: los cazó
#   la matriz de `plugin-tests` en macOS EJECUTANDO la suite, no shellcheck
#   (fix `ec74f43`).
# - `sha256sum` (GNU) vs `shasum` (macOS/BSD) en `.github/workflows/
#   release.yml` rompió la publicación del tag `v0.1.0` en el runner de
#   Windows (`shasum: command not found`, exit 127) y, ya arreglado hacia
#   macOS, rompió `install.ps1` en la dirección contraria (formato de firma
#   distinto según el comando). Bash inline de un `run:` de workflow: NINGÚN
#   gate de este repo lo cubre — ni shellcheck (no son `.sh`), ni
#   `plugin-tests` (no toca `.github/workflows/`) — se cazó en producción,
#   dos veces (PR #7 `360175c`, PR #8 `8a86832`; detalle en
#   `docs/backlog.md`).
#
# Qué entra: todo fichero versionado que termina en .sh, más los ejecutables
# sin extensión cuyo shebang es sh/bash (skills/orchestrate/scripts/*). Descubre
# por el índice de git, no por lista: un script nuevo entra solo.
# Qué NO entra: evals/ (harness congelado de gates ya firmados: tocarlo
# invalida la corrida que certifica) y docs/.
#
# Cada aviso se arregla o se justifica en el sitio con
# `# shellcheck disable=SCxxxx # <por qué>`. No hay .shellcheckrc global a
# propósito: una exclusión global no dice dónde ni por qué.
#
# `-x -P SCRIPTDIR`: sigue los `. "$(dirname "$0")/_helper.sh"`, así que un
# helper roto o una función mal llamada también cuentan.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

SC="${SHELLCHECK:-shellcheck}"
if ! command -v "$SC" >/dev/null 2>&1; then
  echo "test-shellcheck: no encuentro shellcheck ('$SC'). Instálalo o pasa SHELLCHECK=<ruta>." >&2
  exit 1
fi
"$SC" --version | sed -n '2p'

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
  # Un recorrido que no encuentra nada daría verde sin haber mirado nada.
  echo "test-shellcheck: no se encontró ningún script — el recorrido está roto" >&2
  exit 1
fi

if "$SC" -x -P SCRIPTDIR "${ficheros[@]}"; then
  echo "test-shellcheck: OK — ${#ficheros[@]} scripts sin avisos"
else
  echo "test-shellcheck: avisos arriba. Arregla, o justifica en el sitio con '# shellcheck disable=SCxxxx # <por qué>'." >&2
  exit 1
fi
