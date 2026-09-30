#!/usr/bin/env bash
# Test de CONTRATO: confronta las expresiones jq de las que depende la prosa
# de los skills (document/SKILL.md, arquitectura.md: receta
# `exo search --json` -> `.data.results[] | .permalink, .path`) contra un
# envelope PRODUCIDO POR EL BINARIO REAL — no fixtures escritos a mano.
#
# Por qué existe: un envelope que deriva (p. ej. `ruta` vs `path`) deja la
# prosa mintiendo en silencio, sin error de jq. Los scripts vivos del plugin
# (exo-recall.sh, subagent-inject.sh) leen `exo recall` como texto, no su
# envelope JSON, así que el contrato `recall --json` ya no se asevera aquí.
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
# Solo lee: `exo search` no escribe nada, así que este test no necesita
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

. "$SCRIPT_DIR/_timeout.sh"

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
