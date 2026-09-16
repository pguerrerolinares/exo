#!/usr/bin/env bash
# Gate: shellcheck sobre todo el bash versionado que se publica o que corre CI.
#
# Por qué: ~4.700 líneas de bash sin análisis estático. Este gate caza una
# clase concreta de rotura — comillas, globs, `$?` indirecto, variables sin
# usar — vía análisis ESTÁTICO de ficheros `.sh` versionados; no ejecuta nada,
# así que no cubre portabilidad GNU/BSD (shellcheck no sabe que `timeout`,
# `date -d`, `stat -c` o `sha256sum` son GNU-only). Tres roturas de shell
# reales, cada una cazada por un gate distinto (o por ninguno):
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
#   distinto según el comando). Bash inline de un `run:` de workflow: en su
#   momento NINGÚN gate de este repo lo cubría — se cazó en producción, dos
#   veces (PR #7 `360175c`, PR #8 `8a86832`; detalle en `docs/backlog.md`).
#   Desde F4 (2026-09-15) los bloques `run: |` de `.github/workflows/*.yml`
#   SÍ entran (extraídos a fichero temporal, ver más abajo); sigue sin cubrir
#   `run:` de una línea ni `run: >` — ver el comentario junto a la extracción.
#
# Qué entra: todo fichero versionado que termina en .sh, más los ejecutables
# sin extensión cuyo shebang es sh/bash (skills/orchestrate/scripts/*), más
# los bloques `run: |` de `.github/workflows/*.yml` (F4). Los .sh se
# descubren por el índice de git, no por lista: un script nuevo entra solo.
# Qué NO entra: evals/ (harness congelado de gates ya firmados: tocarlo
# invalida la corrida que certifica), docs/, y de los workflows: `run:` de
# una sola línea, `run: >` (folded scalar) y cualquier `run: |` bajo
# `shell: pwsh`/`powershell` (ver detalle junto a la extracción).
#
# Fix de review (orquestador, 2026-09-15): la extracción de más abajo es una
# regex concreta (un espacio exacto tras "run:", sin comentario final). Si un
# bloque real usa otra forma válida (más espacios, comentario tras el `|`,
# CRLF) la extracción lo pierde y el gate seguía en verde reportando MENOS
# bloques, sin avisar — un fallo real pasando en silencio. La guarda de
# coherencia (justo después de la extracción, antes de tocar shellcheck)
# cuenta con un grep deliberadamente más laxo qué líneas DECLARAN un bloque
# `run: |`/`|-`/`|+` y exige que la extracción haya visto cada una; si no,
# `exit 1` listando fichero:línea. Se evalúa ANTES del check de binario
# ausente para que sea comprobable en esta máquina sin shellcheck instalado.
#
# Cada aviso se arregla o se justifica en el sitio con
# `# shellcheck disable=SCxxxx # <por qué>`. No hay .shellcheckrc global a
# propósito: una exclusión global no dice dónde ni por qué.
#
# `-x -P SCRIPTDIR`: sigue los `. "$(dirname "$0")/_helper.sh"`, así que un
# helper roto o una función mal llamada también cuentan.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

. "$(dirname "$0")/_bash-versionado.sh" || { echo "test-shellcheck: no puedo cargar scripts/_bash-versionado.sh" >&2; exit 1; }
bash_versionado

if [ "${#ficheros[@]}" -eq 0 ]; then
  # Un recorrido que no encuentra nada daría verde sin haber mirado nada.
  echo "test-shellcheck: no se encontró ningún script — el recorrido está roto" >&2
  exit 1
fi

# F4 (docs/backlog.md, ítem "(NUEVO, revisión final campaña B, 2026-09-13)
# El bash inline de `run:` en `.github/workflows/*.yml` no pasa por ningún
# gate" — cítalo por título, no por línea: el sync de backlog.md desplaza
# líneas): el bash inline de `run: |` en .github/workflows/*.yml no es un
# fichero .sh — bash_versionado() no lo ve.
# Cada bloque se extrae a un fichero temporal con la MISMA dedentación que
# aplica GitHub Actions (recorta hasta la columna de "run:" + 2) y se suma a
# la lista que shellcheck revisa. Sin `yq` ni dependencia nueva: lectura
# línea a línea en bash puro, igual de espíritu que _bash-versionado.sh.
#
# Qué SÍ entra: bloques `run: |` (literal, con o sin `+`/`-` de chomping)
# tal y como aparecen hoy en ci.yml y release.yml — todos bajo `shell: bash`
# (explícito o por default de runner Linux/macOS).
# Qué NO entra (gaps conocidos, no silenciosos):
# - `run:` de una sola línea: ya es bash inline sin analizar, pero extraerlo
#   fichero a fichero no aporta gran cosa sobre un one-liner y complica la
#   detección de dónde empieza/acaba; no se cubre en esta tarea.
# - `run: >` (folded scalar): semántica distinta a `|` (las líneas se unen
#   con espacios) — tratarlo como `|` daría un script sintácticamente
#   distinto al que GitHub Actions ejecuta de verdad. No se extrae.
# - `shell: pwsh` / `shell: powershell`: hoy NINGÚN `run: |` de este repo
#   corre bajo pwsh (verificado 2026-09-15: los tres runners de la matriz de
#   `release.yml` fijan `shell: bash` explícito). Este extractor no mira la
#   clave `shell:` del step — si algún día se añade un `run: |` bajo pwsh,
#   se colaría aquí como si fuera bash y shellcheck lo marcaría en falso.
#   Gap documentado, no implementado por ausencia de caso real hoy.
#
# Fix crítico (review final, 2026-09-16): `${{ matrix.target }}` / `${{
# matrix.bin }}` (release.yml, bloque "Empaquetar y calcular el SHA256") NO
# son bash — GitHub Actions los sustituye por texto ANTES de que el runner
# vea el script; `${{` no es una expansión válida de shell. `bash -n` los
# tolera (no es un error de sintaxis Bourne), pero el parser de ShellCheck
# 0.11.0 sí revienta con ellos (medido vía Docker: SC2296 "Parameter
# expansions can't start with {", y aborta el resto del fichero). Cada
# ocurrencia se sustituye por `${GHA_EXPR:-}` antes de escribirse al fichero
# temporal: sigue siendo una expansión de parámetro bash válida (no cambia
# la sintaxis del bloque — `case "${{ matrix.bin }}" in *.exe)` sigue siendo
# un `case` válido con `${GHA_EXPR:-}` de sujeto) y, al parecer una variable
# con valor por defecto en vez de una palabra suelta, no dispara SC2194
# ("¿olvidaste el $ de una variable?", medido vía Docker con un literal
# `GHA_EXPR` sin `$`). Perder de vista el valor real que GitHub inyectará es
# aceptable: ShellCheck analiza sintaxis y patrones de shell, no el valor
# concreto de `matrix.target`/`matrix.bin`.
sustituye_expr_gha() {
  # shellcheck disable=SC2016 # comillas simples deliberadas: es el programa de sed, no bash quien debe ver el $
  sed -E 's/\$\{\{[^}]*\}\}/${GHA_EXPR:-}/g'
}

WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

# m4 (review final, 2026-09-16): sin nullglob, un glob que no casa nada deja
# el patrón literal `.github/workflows/*.yml` como único "fichero" del
# bucle — ni la extracción ni la guarda de coherencia de abajo lo detectan
# (ambas leen de una redirección que falla en silencio sin `set -e`), y el
# gate reportaba "+ 0 bloques" en verde como si de verdad no hubiera ningún
# `run: |` que analizar. Misma clase de guarda que la de `ficheros[]` de
# arriba.
shopt -s nullglob
WORKFLOWS=(.github/workflows/*.yml)
shopt -u nullglob
if [ "${#WORKFLOWS[@]}" -eq 0 ]; then
  echo "test-shellcheck: no se encontró ningún workflow en .github/workflows/*.yml — el glob está roto" >&2
  exit 1
fi

extraidos=()
detectadas=()  # fichero:línea de cada "run: |" que la extracción reconoció — para la guarda de coherencia
for wf in "${WORKFLOWS[@]}"; do
  base="$(basename "$wf" .yml)"
  n=0
  fichero=""
  indent=0
  lineno=0
  while IFS= read -r linea; do
    lineno=$((lineno + 1))
    linea="${linea%$'\r'}"  # normaliza CRLF si el fichero llegara con \r (defensivo: eol=lf ya lo impide)
    if [ -n "$fichero" ]; then
      if [[ "$linea" =~ ^[[:space:]]*$ ]]; then
        printf '%s\n' "" >> "$fichero"
        continue
      fi
      cur="${linea%%[! ]*}"
      if [ "${#cur}" -lt "$indent" ]; then
        fichero=""
      else
        printf '%s\n' "${linea:$indent}" | sustituye_expr_gha >> "$fichero"
        continue
      fi
    fi
    if [[ "$linea" =~ ^([[:space:]]*)run:\ \|[+-]?[[:space:]]*$ ]]; then
      n=$((n + 1))
      lead="${BASH_REMATCH[1]}"
      indent=$((${#lead} + 2))
      fichero="$WORKDIR/${base}-run-${n}.sh"
      printf '#!/usr/bin/env bash\n' > "$fichero"
      extraidos+=("$fichero")
      detectadas+=("$wf:$lineno")
    fi
  done < "$wf"
done

# Guarda de coherencia (fix de review): grep deliberadamente más laxo que la
# regex de extracción de arriba — cero o más espacios entre "run:" y "|" (en
# vez de exactamente uno) y comentario final opcional — para no perderse
# ninguna forma válida de declarar el bloque. Cada línea que este grep marca
# como declaración de un bloque `run: |`/`|-`/`|+` DEBE aparecer en
# `detectadas[]`; si no, la extracción se quedó corta y el gate lo dice antes
# de fallar en silencio con menos bloques de los que hay de verdad.
declaradas=()
for wf in "${WORKFLOWS[@]}"; do
  while IFS=: read -r ln _resto; do
    [ -n "$ln" ] && declaradas+=("$wf:$ln")
  done < <(tr -d '\r' < "$wf" | grep -n -E '^[[:space:]]*run:[[:space:]]*\|[+-]?[[:space:]]*(#.*)?$')
done

faltantes=()
for d in "${declaradas[@]}"; do
  hallado=0
  for e in "${detectadas[@]}"; do
    if [ "$d" = "$e" ]; then
      hallado=1
      break
    fi
  done
  [ "$hallado" -eq 0 ] && faltantes+=("$d")
done

if [ "${#faltantes[@]}" -gt 0 ]; then
  echo "test-shellcheck: la extracción de run: | dejó bloques declarados sin ver (silencioso hasta ahora):" >&2
  for f in "${faltantes[@]}"; do
    echo "  - $f" >&2
  done
  echo "test-shellcheck: soporta la variante en la extracción de arriba, o justifica por qué se excluye a propósito — nunca la ignores en silencio." >&2
  exit 1
fi

SC="${SHELLCHECK:-shellcheck}"
if ! command -v "$SC" >/dev/null 2>&1; then
  echo "test-shellcheck: no encuentro shellcheck ('$SC'). Instálalo o pasa SHELLCHECK=<ruta>." >&2
  exit 1
fi
"$SC" --version | sed -n '2p'

if "$SC" -x -P SCRIPTDIR "${ficheros[@]}" "${extraidos[@]}"; then
  echo "test-shellcheck: OK — ${#ficheros[@]} scripts + ${#extraidos[@]} bloques run: | de .github/workflows/ sin avisos"
else
  echo "test-shellcheck: avisos arriba. Arregla, o justifica en el sitio con '# shellcheck disable=SCxxxx # <por qué>'." >&2
  exit 1
fi
