#!/usr/bin/env bash
# Gate: los tres documentos declarativos de producto (README.md,
# docs/arquitectura.md, docs/instalacion.md) no afirman lo que el repo ya no
# hace. `docs/backlog.md` es el cuarto "core" (arquitectura.md, "Qué
# documentación es viva") pero es un LEDGER histórico: cita a propósito
# frases ya muertas como evidencia de ítems cerrados (p.ej. "Sin CI" en
# backlog.md:249, dentro de un ítem [x]) y versiones de repos ajenos citados
# en sus consultas (backlog.md:126, "v2.2.1" del clon de ECC). Meterlo en
# los checks de abajo lo pondría en rojo permanente por su propio diseño —
# sus afirmaciones frágiles las cierra un humano al cerrar el ítem
# (backlog.md:283-285), no este gate.
#
# Cinco comprobaciones sobre README.md / docs/arquitectura.md /
# docs/instalacion.md:
#   (a) ausencia de frases muertas conocidas
#   (b) todo `exo <subcomando>` citado existe en enum Comando de main.rs
#       (y `exo write <x>` en enum ComandoWrite) — excluye las secciones
#       "## ... NO ..." (con espacios), que citan a propósito lo que NO
#       existe (docs/instalacion.md §7, "Lo que NO hay todavía")
#   (c) toda versión `vX.Y.Z` citada es un tag, o coincide con
#       engine/Cargo.toml o con plugin.json
#   (d) todo enlace `` `docs/...` ``, `` `evals/...` ``, `` `scripts/...` ``,
#       `` `plugins/...` `` o `` `engine/...` `` entre backticks resuelve a
#       un fichero o directorio existente, resuelto contra la raíz del repo
#       (`git rev-parse --show-toplevel`, adonde este script ya hace `cd`
#       más abajo). Prefijos ampliados (hallazgo del orquestador,
#       2026-09-15): la Task 2 corrigió las rutas de hooks del README a
#       `plugins/exo/scripts/<x>.sh`, pero el patrón original solo cazaba
#       `docs|evals|scripts` — la clase exacta de deriva que motivó esta
#       tarea (rutas de hooks rotas) quedaba sin gate en cuanto empezaban
#       por `plugins/`. `engine/` se añade por la misma razón: es el otro
#       directorio de primer nivel citado por ruta en estos docs.
#   (e) la tabla de hooks (`| Reflejo | Evento | Fichero |...`) tiene tantas
#       filas como `hooks/hooks.json` cablea de verdad — en LOS DOS
#       ficheros que la llevan, `README.md` y `plugins/exo/README.md` (I1,
#       review final 2026-09-16: «nueve» en el README, diez en el JSON;
#       campaña L Task 3: `plugins/exo/README.md` tenía la misma tabla sin
#       gate — su cabecera lleva dos columnas más, `Qué hace` y
#       `Abstención`, así que el patrón ya no puede anclar con `$`).
set -uo pipefail
# Fix de la review final (2026-09-16): `cd "$(git rev-parse --show-toplevel)"`
# directo tenía un fallo silencioso — si la sustitución sale vacía, `cd ""`
# devuelve 0 sin moverse (medido), así que `|| exit 1` nunca dispara y el
# resto del gate corre contra el cwd del caller, no la raíz del repo.
# Captura y comprueba antes de moverse.
RAIZ="$(git rev-parse --show-toplevel)" || exit 1
if [ -z "$RAIZ" ]; then
  echo "test-docs-vivos: git rev-parse --show-toplevel no devolvió nada" >&2
  exit 1
fi
cd "$RAIZ" || exit 1

# `grep`/`awk` con `.` o clases sobre texto con multibyte (é, —, ↔, presentes
# en los tres docs) puede fallar en este Git Bash sin un locale UTF-8
# explícito. Se fija aquí en vez de confiar en el entorno del caller (CI o
# shell local pueden traer LANG/LC_ALL distintos o vacíos).
export LC_ALL=C.UTF-8

FALLOS=0
DOCS=(README.md docs/arquitectura.md docs/instalacion.md)

# Fix de la review final (2026-09-16): sin esto, un doc core borrado o
# renombrado (fusión de ramas, typo) hacía que TODOS los bucles de abajo
# leyeran cero líneas de ese fichero — el gate seguía en verde con el doc
# ausente, exactamente la clase de fallo silencioso que este gate existe
# para cazar en los otros tres.
for doc in "${DOCS[@]}"; do
  [ -f "$doc" ] || { echo "[FAIL] falta $doc" >&2; exit 1; }
done

# --- (a) Frases muertas conocidas ------------------------------------------
# Lista curada (docs/backlog.md, ítem "(revisión 2026-09-04) La
# documentación de referencia contradice el repo el mismo día en que se
# escribió" — cítalo por título, no por línea: el sync de backlog.md
# desplaza líneas), no heurística: un falso positivo nuevo se añade a mano.
FRASES_MUERTAS=(
  "Sin CI"
  "no hay ningún tag"
  "viven en una rama sin mergear"
)
for doc in "${DOCS[@]}"; do
  for frase in "${FRASES_MUERTAS[@]}"; do
    if grep -n -F "$frase" "$doc" >/dev/null 2>&1; then
      echo "[FAIL] $doc afirma una frase muerta: \"$frase\"" >&2
      grep -n -F "$frase" "$doc" | sed "s|^|  $doc:|" >&2
      FALLOS=1
    fi
  done
done

# Quita el contenido de cualquier sección `## ...` cuyo título contenga
# " NO " (con espacios) — convención de este repo para "lo que falta"
# (arquitectura.md §7, instalacion.md §7): ahí se cita a propósito lo que
# NO existe, y (b) no debe dispararse con eso.
sin_secciones_negativas() {
  awk '/^## / { neg = ($0 ~ / NO /) } !neg { print }' "$1"
}

# --- (b) Subcomandos citados existen en el binario -------------------------
# Fuente de verdad: enum Comando / enum ComandoWrite de engine/src/main.rs
# (parseo estático — este gate corre en el job `static-checks`, que no compila el
# release; sin binario no hay `exo --help` que leer).
COMANDOS="$(sed -n '/^enum Comando {/,/^}/p' engine/src/main.rs \
  | grep -oE '^    [A-Z][A-Za-z]+\(' | tr -d ' (' | tr '[:upper:]' '[:lower:]' | sort -u)"
SUBCOMANDOS_WRITE="$(sed -n '/^enum ComandoWrite {/,/^}/p' engine/src/main.rs \
  | grep -oE '^    [A-Z][A-Za-z]+\(' | tr -d ' (' | tr '[:upper:]' '[:lower:]' | sort -u)"

for doc in "${DOCS[@]}"; do
  while IFS= read -r cmd; do
    [ -n "$cmd" ] || continue
    if ! printf '%s\n' "$COMANDOS" | grep -qx "$cmd"; then
      echo "[FAIL] $doc cita \`exo $cmd\`, que no existe en enum Comando de main.rs" >&2
      FALLOS=1
    fi
  done < <(sin_secciones_negativas "$doc" | grep -oE '`exo [a-z][a-z-]*' | sed 's/`exo //' | sort -u)
  while IFS= read -r sub; do
    [ -n "$sub" ] || continue
    if ! printf '%s\n' "$SUBCOMANDOS_WRITE" | grep -qx "$sub"; then
      echo "[FAIL] $doc cita \`exo write $sub\`, que no existe en enum ComandoWrite" >&2
      FALLOS=1
    fi
  done < <(sin_secciones_negativas "$doc" | grep -oE '`exo write [a-z]+' | sed 's/`exo write //' | sort -u)
done

# --- (c) Versiones citadas existen -----------------------------------------
ENGINE_VER="$(sed -n 's/^version = "\(.*\)"$/\1/p' engine/Cargo.toml | head -n1)"
PLUGIN_VER="$(jq -r '.version' plugins/exo/.claude-plugin/plugin.json | tr -d '\r')"
TAGS="$(git tag -l)"
for doc in "${DOCS[@]}"; do
  while IFS= read -r v; do
    [ -n "$v" ] || continue
    if [ "$v" = "v$ENGINE_VER" ] || [ "$v" = "v$PLUGIN_VER" ] || printf '%s\n' "$TAGS" | grep -qx "$v"; then
      continue
    fi
    echo "[FAIL] $doc cita $v, que no es un tag ni coincide con engine ($ENGINE_VER) ni plugin ($PLUGIN_VER)" >&2
    FALLOS=1
  done < <(grep -oE '\bv[0-9]+\.[0-9]+\.[0-9]+\b' "$doc" | sort -u)
done

# --- (d) Enlaces relativos a docs/, evals/, scripts/, plugins/, engine/ ----
# resuelven ------------------------------------------------------------------
for doc in "${DOCS[@]}"; do
  # shellcheck disable=SC2016 # backticks literales de markdown en el patrón de grep, no sustitución de comandos
  while IFS= read -r ruta; do
    [ -n "$ruta" ] || continue
    destino="${ruta%/}"
    if [ ! -e "$destino" ]; then
      echo "[FAIL] $doc cita \`$ruta\`, que no existe" >&2
      FALLOS=1
    fi
  done < <(grep -oE '`(docs|evals|scripts|plugins|engine)/[A-Za-z0-9_./-]*`' "$doc" | tr -d '`' | sort -u)
done

# --- (e) Las tablas de hooks (README.md Y plugins/exo/README.md) tienen ---
#         tantas filas como hooks reales ---------------------------------
# I1 (review final, 2026-09-16): el README llegó a decir «nueve hooks» con
# `hooks.json` cableando diez — nadie lo comprobó hasta la review. Campaña L
# Task 3: `plugins/exo/README.md` lleva la MISMA tabla (con dos columnas
# extra, `Qué hace` y `Abstención`) y no tenía gate — el patrón ya no ancla
# con `$` al final de la cabecera a propósito, para que haga match con las
# dos formas. Cuenta filas (hasta la primera línea que ya no empieza por
# `|`) en cada fichero y la compara contra el cableado vivo.
HOOKS_REAL="$(jq -r '[.hooks[]?[]?.hooks[]?] | length' plugins/exo/hooks/hooks.json | tr -d '\r')"
for TABLA_DOC in README.md plugins/exo/README.md; do
  FILAS_TABLA="$(awk '
    /^\| Reflejo \| Evento \| Fichero \|/ { en_tabla = 1; next }
    en_tabla && /^\|---/ { next }
    en_tabla && /^\|/ { n++; next }
    en_tabla { exit }
    END { print n + 0 }
  ' "$TABLA_DOC")"
  if [ "$FILAS_TABLA" -ne "$HOOKS_REAL" ]; then
    echo "[FAIL] $TABLA_DOC: la tabla de hooks tiene $FILAS_TABLA fila(s) pero plugins/exo/hooks/hooks.json cablea $HOOKS_REAL" >&2
    FALLOS=1
  fi
done

if [ "$FALLOS" -eq 0 ]; then
  echo "[OK] test-docs-vivos: README.md/docs/{arquitectura,instalacion}.md sin frases muertas, subcomandos inventados, versiones huérfanas, enlaces rotos ni tablas de hooks desfasadas"
fi
exit "$FALLOS"
