#!/usr/bin/env bash
# Test de CONTRATO: confronta las expresiones jq de las que depende
# recall-inject.sh contra un envelope PRODUCIDO POR EL BINARIO REAL — no
# fixtures escritos a mano (Task 7, ola 1A "exo genérico").
#
# Por qué existe: `test-recall-inject.sh` stubea el binario (FAKE_EXO, DB
# falsa, envelopes hechos con printf/jq -n). Cuando D8 renombró las claves
# de `data` al inglés y subió SCHEMA_VERSION a 2, esa suite siguió en verde
# porque script y fixtures se migran juntos y ambos pueden acordar una forma
# que el binario real jamás emite. Ese gate no puede detectar la regresión
# que existe para detectar. Este test cierra el lazo: pide un envelope de
# verdad y comprueba sobre él los predicados exactos de los que vive
# recall-inject.sh (has data.notes, data.truncated booleano, notes[0] con
# path/title/permalink de tipo string no vacío y snippet null-o-string-no-vacío
# —porque recall-inject.sh hace `sane`/`ltrimstr` sobre esos valores, no solo
# lee sus claves—, schema_version==2).
#
# ABSTENCIÓN, no PASS falso: si falta el binario, el índice o la KB de esta
# máquina, este test SALE CON EXIT != 0 y dice en voz alta que no pudo
# verificar nada. Un test que aprueba sin haber ejercido el binario es
# exactamente el fallo silencioso que esta tarea persigue cerrar (ver
# kb-demo: "Fallo silencioso — el instrumento que no grita").
#
# En CI lo corre scripts/test-contrato-ci.sh, que monta un fixture propio
# (KB semilla de `exo init` + índice) con EXO_CONFIG aislado. En local, sin
# ese wrapper, resuelve índice y KB de la config de la máquina.
#
# Solo lee: `exo recall` no escribe nada, así que este test no necesita
# aislamiento de KB/índice como el resto de la suite de scripts.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Mismo seam EXO_BIN que el resto de scripts, pero el DEFAULT es el binario
# CONSTRUIDO DEL REPO (engine/target/release/exo[.exe]), no
# `$(command -v exo)` / `~/.local/bin/exo`: el instalado puede ir por detrás
# del repo, y un default que cayera ahí daría un resultado que no dice nada
# del cambio en curso. El `.exe` solo existe en Windows: con el literal
# anterior, en Linux/macOS este test se abstenía siempre.
BIN_REPO="$REPO_ROOT/engine/target/release/exo"
[ -e "$BIN_REPO.exe" ] && BIN_REPO="$BIN_REPO.exe"
EXO_BIN="${EXO_BIN:-$BIN_REPO}"

# Rutas estilo Windows: el binario es nativo y no entiende las que monta Git
# Bash (letra de unidad + "Users" + nombre de perfil).
# Índice y KB salen de `exo config --json` (Task 8), no de un literal — pero
# SIEMPRE del binario recién compilado del repo, nunca de $EXO_BIN: cuando
# este test apunta $EXO_BIN a un binario viejo para probar el estado "rojo",
# ese binario es de antes de Task 8 y no conoce `config`; resolver ahí
# dejaría índice/KB vacíos y el test abstendría en vez de fallar en rojo por
# la causa real (el contrato de `recall`). El seam de entorno (EXO_INDEX,
# EXO_KB) sigue mandando si algo los define, igual que antes.
CONFIG_BIN="$BIN_REPO"
CONFIG_JSON="$("$CONFIG_BIN" config --json 2>/dev/null)" || CONFIG_JSON=""
EXO_INDEX="${EXO_INDEX:-$(printf '%s' "$CONFIG_JSON" | jq -r '.data.index.db // empty' 2>/dev/null)}"
EXO_KB="${EXO_KB:-$(printf '%s' "$CONFIG_JSON" | jq -r '.data.kb.path // empty' 2>/dev/null)}"

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

abstenerse() {  # $1 = motivo
  printf '[ABSTENCION] no se pudo verificar el contrato del engine: %s\n' "$1" >&2
  exit 2
}

[ -x "$EXO_BIN" ] || abstenerse "binario ausente o no ejecutable ($EXO_BIN)"

# Dos causas distintas para un EXO_INDEX/EXO_KB vacíos o inválidos: que
# `exo config --json` (via $CONFIG_BIN) no haya podido resolver nada —y
# entonces el motivo NO es el índice/la KB, es la config— o que sí resolvió
# algo pero esa ruta no existe. Un solo mensaje interpolando una variable
# vacía culpa a la pieza equivocada y deja paréntesis en blanco.
if [ -z "$EXO_INDEX" ]; then
  abstenerse "no se pudo resolver el índice: \`exo config --json\` (via $CONFIG_BIN) no devolvió nada, y \$EXO_INDEX no está definida"
fi
[ -f "$EXO_INDEX" ] || abstenerse "el índice resuelto no existe ($EXO_INDEX)"

if [ -z "$EXO_KB" ]; then
  abstenerse "no se pudo resolver la KB: \`exo config --json\` (via $CONFIG_BIN) no devolvió nada, y \$EXO_KB no está definida"
fi
[ -d "$EXO_KB" ] || abstenerse "la KB resuelta no existe ($EXO_KB)"

# Modo arranque (sin --query): no depende del modelo de embeddings y basta
# para ejercer la forma del envelope que consume recall-inject.sh.
ERR_TMP="$(mktemp)" || ERR_TMP=""
. "$SCRIPT_DIR/_timeout.sh"
SALIDA="$(con_timeout "${EXO_CONTRATO_TIMEOUT:-15}" "$EXO_BIN" recall --json \
            --db "$EXO_INDEX" --kb "$EXO_KB" 2>"${ERR_TMP:-/dev/null}")"
RC=$?
ERR=""
[ -n "$ERR_TMP" ] && ERR="$(cat "$ERR_TMP" 2>/dev/null)"; rm -f "$ERR_TMP"

if [ "$RC" -ne 0 ]; then
  abstenerse "el binario salió con rc=$RC (err: $(printf '%s' "$ERR" | tr -d '\n' | cut -c1-200))"
fi
[ -n "$SALIDA" ] || abstenerse "el binario no devolvió salida"

# --- Los predicados exactos de los que vive recall-inject.sh ----------------

if printf '%s' "$SALIDA" | jq -e 'has("data")' >/dev/null 2>&1; then
  pass "contrato: el envelope tiene .data"
else fail "contrato: el envelope tiene .data" "$(printf '%s' "$SALIDA" | head -c 200)"; fi

if printf '%s' "$SALIDA" | jq -e '.data | has("notes")' >/dev/null 2>&1; then
  pass "contrato: .data tiene notes"
else fail "contrato: .data tiene notes" "$(printf '%s' "$SALIDA" | jq -c '.data | keys' 2>/dev/null)"; fi

if printf '%s' "$SALIDA" | jq -e '.data.truncated | type == "boolean"' >/dev/null 2>&1; then
  pass "contrato: .data.truncated es booleano"
else fail "contrato: .data.truncated es booleano" "$(printf '%s' "$SALIDA" | jq -c '.data.truncated' 2>/dev/null)"; fi

# Los dos checks siguientes (no-vacío y forma de la primera nota) dependen
# de que .data.notes exista: si el check de arriba ya falló, no tiene sentido
# repetir el rojo por la misma causa raíz con un "n=" vacío y confuso — se
# omiten en vez de fallar en cascada.
if printf '%s' "$SALIDA" | jq -e '.data | has("notes")' >/dev/null 2>&1; then
  N_NOTES="$(printf '%s' "$SALIDA" | jq '.data.notes | length' 2>/dev/null)"
  if [ "${N_NOTES:-0}" -gt 0 ] 2>/dev/null; then
    pass "contrato: .data.notes no está vacío ($N_NOTES notas)"
    # No basta con has(...): un engine que emitiera {"path":null,...} tendría
    # las cuatro claves y pasaría en falso. recall-inject.sh hace `sane` y
    # `ltrimstr` sobre estos valores (operaciones de cadena), así que el gate
    # exige tipo string no vacío para path/title/permalink. `snippet` SÍ es
    # nullable a propósito (Option<String> en el engine; modo arranque lo deja
    # en null): se admite null o cadena no vacía, nunca cadena vacía.
    if printf '%s' "$SALIDA" | jq -e '
          .data.notes[0] as $n
          | ($n.path|type) == "string" and ($n.path|length) > 0
          and ($n.title|type) == "string" and ($n.title|length) > 0
          and ($n.permalink|type) == "string" and ($n.permalink|length) > 0
          and ( ($n.snippet == null)
                or (($n.snippet|type) == "string" and ($n.snippet|length) > 0) )
        ' >/dev/null 2>&1; then
      pass "contrato: la primera nota trae path/title/permalink no vacíos (snippet null o no vacío)"
    else
      fail "contrato: la primera nota trae path/title/permalink no vacíos (snippet null o no vacío)" \
        "$(printf '%s' "$SALIDA" | jq -c '.data.notes[0]' 2>/dev/null)"
    fi
  else
    fail "contrato: .data.notes no está vacío" "n=$N_NOTES — sin una nota real no se puede comprobar sus claves"
  fi
fi

# H2/H3: recall-inject.sh lee .data.elapsed_s y .data.refresh_s (número o null)
# y .data.warnings (array o ausente). En modo arranque sin --refresh las dos
# claves de tiempo EXISTEN con null: se exige la clave, no solo el valor, para
# que un engine anterior a la campaña A dé rojo aquí.
if printf '%s' "$SALIDA" | jq -e '(.data | has("elapsed_s") and has("refresh_s"))
      and .data.elapsed_s == null and .data.refresh_s == null
      and ((.data.warnings // []) | type) == "array"' >/dev/null 2>&1; then
  pass "contrato: elapsed_s/refresh_s presentes (null en arranque) y warnings array o ausente"
else fail "contrato: elapsed_s/refresh_s presentes (null en arranque) y warnings array o ausente" \
  "$(printf '%s' "$SALIDA" | jq -c '.data | {elapsed_s, refresh_s, warnings}' 2>/dev/null)"; fi

if printf '%s' "$SALIDA" | jq -e '.schema_version == 2' >/dev/null 2>&1; then
  pass "contrato: schema_version == 2"
else fail "contrato: schema_version == 2" "$(printf '%s' "$SALIDA" | jq -c '.schema_version' 2>/dev/null)"; fi

# --- Los predicados de los que vive la prosa de `search --json` --------------
# La receta que documentan document/SKILL.md y arquitectura.md es
# `.data.results[] | .permalink, .path`. Si el envelope deriva, esa prosa pasa a
# mentir en silencio (un `.ruta` devolvía `null` sin error de jq durante meses).
SALIDA_S="$(con_timeout "${EXO_CONTRATO_TIMEOUT:-15}" "$EXO_BIN" search --type hybrid \
              --json --limit 3 --db "$EXO_INDEX" --kb "$EXO_KB" "doctrina" 2>/dev/null)"
RC_S=$?
if [ "$RC_S" -ne 0 ] || [ -z "$SALIDA_S" ]; then
  abstenerse "search --json salió con rc=$RC_S o sin salida"
fi

if printf '%s' "$SALIDA_S" | jq -e '.data.results | type == "array"' >/dev/null 2>&1; then
  pass "contrato search: .data.results es un array"
else fail "contrato search: .data.results es un array" "$(printf '%s' "$SALIDA_S" | jq -c '.data | keys' 2>/dev/null)"; fi

# Guard de vacuidad, hermano del de `.data.notes` de arriba: los tres predicados
# siguientes miran `.data.results[0]`, y sobre una lista vacía `jq` opera contra
# `null` — pasarían o fallarían por vacuidad, sin haber ejercido nada. Importa
# porque este gate corre en CI (`scripts/test-contrato-ci.sh`) contra la KB
# semilla de `exo init`, no contra una KB poblada: el día que la semilla deje de
# traer una nota que case con la query, el rojo debe decir ESO y no otra cosa.
N_RES="$(printf '%s' "$SALIDA_S" | jq '.data.results | length' 2>/dev/null)"
if [ "${N_RES:-0}" -gt 0 ] 2>/dev/null; then
  pass "contrato search: la query de prueba devolvió resultados ($N_RES)"
else
  fail "contrato search: la query de prueba devolvió resultados" \
    "n=$N_RES — sin un resultado real no se puede comprobar la forma de sus claves"
fi

if printf '%s' "$SALIDA_S" | jq -e '
      .data.results[0] as $r
      | ($r.permalink|type) == "string" and ($r.permalink|length) > 0
      and ($r.path|type) == "string" and ($r.path|length) > 0
    ' >/dev/null 2>&1; then
  pass "contrato search: el primer resultado trae permalink y path no vacíos"
else
  fail "contrato search: el primer resultado trae permalink y path no vacíos" \
    "$(printf '%s' "$SALIDA_S" | jq -c '.data.results[0]' 2>/dev/null)"
fi

# `ruta` es el nombre del campo RUST; el envelope emite `path`. Si algún día
# reaparece, la prosa que lo citaba vuelve a ser correcta y este gate debe caer.
if printf '%s' "$SALIDA_S" | jq -e '.data.results[0] | has("ruta") | not' >/dev/null 2>&1; then
  pass "contrato search: el envelope NO trae 'ruta' (es 'path')"
else fail "contrato search: el envelope NO trae 'ruta'" "$(printf '%s' "$SALIDA_S" | jq -c '.data.results[0]' 2>/dev/null)"; fi

# La ruta del envelope no lleva separador nativo.
if printf '%s' "$SALIDA_S" | jq -e '[.data.results[].path | select(. != null) | contains("\\")] | any | not' >/dev/null 2>&1; then
  pass "contrato search: ninguna path del envelope lleva barra invertida"
else fail "contrato search: ninguna path lleva barra invertida" "$(printf '%s' "$SALIDA_S" | jq -c '[.data.results[].path]' 2>/dev/null)"; fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
