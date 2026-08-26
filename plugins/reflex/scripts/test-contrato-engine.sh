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
# G5 (cuando haya CI): este test depende de estado de ESTA máquina
# (C:/Users/paul/.exo/index.db, C:/proyectos/homework/kb-demo). No hay
# fixture reproducible todavía — CI necesitará uno propio (índice + KB de
# ejemplo) para poder correr esto en el pipeline. Hasta entonces es un gate
# local, de máquina de desarrollo.
#
# Solo lee: `exo recall` no escribe nada, así que este test no necesita
# aislamiento de KB/índice como el resto de la suite de scripts.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Mismo seam EXO_BIN que el resto de scripts, pero el DEFAULT es el binario
# CONSTRUIDO DEL REPO (engine/target/release/exo.exe), no
# `$(command -v exo)` / `~/.local/bin/exo`: ese instalado en esta máquina
# sigue siendo v1 a propósito (decisión de pre-flight de esta ola), y un
# default que cayera ahí daría un resultado que no dice nada de este cambio.
EXO_BIN="${EXO_BIN:-$REPO_ROOT/engine/target/release/exo.exe}"

# Rutas estilo Windows: el binario es nativo y no entiende `/c/Users/...`.
# Índice y KB reales de esta máquina — no hay fixture (ver nota G5 arriba).
EXO_INDEX="${EXO_INDEX:-C:/Users/paul/.exo/index.db}"
EXO_KB="${EXO_KB:-C:/proyectos/homework/kb-demo}"

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

abstenerse() {  # $1 = motivo
  printf '[ABSTENCION] no se pudo verificar el contrato del engine: %s\n' "$1" >&2
  exit 2
}

[ -x "$EXO_BIN" ]  || abstenerse "binario ausente o no ejecutable ($EXO_BIN)"
[ -f "$EXO_INDEX" ] || abstenerse "falta el índice de esta máquina ($EXO_INDEX)"
[ -d "$EXO_KB" ]    || abstenerse "falta la KB de esta máquina ($EXO_KB)"

# Modo arranque (sin --query): no depende del modelo de embeddings y basta
# para ejercer la forma del envelope que consume recall-inject.sh.
ERR_TMP="$(mktemp)" || ERR_TMP=""
SALIDA="$(timeout "${EXO_CONTRATO_TIMEOUT:-15}" "$EXO_BIN" recall --json \
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

if printf '%s' "$SALIDA" | jq -e '.schema_version == 2' >/dev/null 2>&1; then
  pass "contrato: schema_version == 2"
else fail "contrato: schema_version == 2" "$(printf '%s' "$SALIDA" | jq -c '.schema_version' 2>/dev/null)"; fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
