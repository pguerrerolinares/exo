#!/usr/bin/env bash
# Test standalone para cobertura-reglas.sh (#207): stub de `exo rules
# --coverage` y reflex-log falso en mktemp -d; nunca toca los reales.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="${SCRIPT_DIR}/cobertura-reglas.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
export REFLEX_LOG_FILE="$TMP/reflex-log.jsonl"
unset EXO_KB EXO_COBERTURA_DIAS

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }
contains() { case "$1" in *"$2"*) return 0 ;; *) return 1 ;; esac; }

STUB="$TMP/exo-stub"
cat > "$STUB" <<'STUBEOF'
#!/usr/bin/env bash
printf '%s' "${STUB_OUT:-}"
exit "${STUB_RC:-0}"
STUBEOF
chmod +x "$STUB"
export EXO_BIN="$STUB"

export STUB_OUT='{"command":"rules-coverage","data":{"cap":10,"notas":[
 {"nota":"core/reglas-duras.md","reglas":5,"estado":"ok"},
 {"nota":"projects/llena.md","reglas":10,"estado":"al_cap"},
 {"nota":"projects/sobra.md","reglas":11,"estado":"excede"},
 {"nota":"projects/vacia.md","reglas":0,"estado":"seccion_vacia"},
 {"nota":"projects/hermana.md","reglas":0,"estado":"sin_seccion"}]},"schema_version":2}'

hoy="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
skip() { printf '{"ts":"%s","reflex":"project-rules-skip","payload":"%s"}\n' "$1" "$2"; }
{
  skip "$hoy" "reason=sin_nota repo=foo"
  skip "2000-01-01T00:00:00Z" "reason=sin_nota repo=foo"
  skip "$hoy" "reason=sin_nota repo=foo"
  skip "$hoy" "reason=sin_seccion repo=bar"
  skip "$hoy" "reason=sin_git repo=-"
  skip "$hoy" "reason=error_engine repo=- err=boom"
  skip "$hoy" "reason=engine_stale repo=baz"
  printf '{"ts":"%s","reflex":"inject-emitted","payload":"reason=sin_nota repo=nope"}\n' "$hoy"
  printf 'no-json\n'
} > "$REFLEX_LOG_FILE"

OUT="$("$SCRIPT")"; RC=$?
if [ "$RC" -eq 0 ]; then pass "sale 0"; else fail "sale 0" "rc=$RC"; fi

if contains "$OUT" "al_cap	10/10	projects/llena.md" && contains "$OUT" "excede	11/10	projects/sobra.md" \
   && contains "$OUT" "seccion_vacia	0/10	projects/vacia.md"; then pass "notas al CAP, excedidas y vacías"
else fail "notas al CAP, excedidas y vacías" "$OUT"; fi

if ! contains "$OUT" "projects/hermana.md" && contains "$OUT" "resumen: 3 con reglas, 1 sin sección"; then
  pass "sin_seccion solo se cuenta"
else fail "sin_seccion solo se cuenta" "$OUT"; fi

if contains "$OUT" "   2 arranques	sin_nota	foo	último $hoy"; then pass "skips agrupados por repo, fuera de ventana excluidos"
else fail "skips agrupados por repo, fuera de ventana excluidos" "$OUT"; fi

if contains "$OUT" "sin_seccion	bar"; then pass "sin_seccion del log cuenta"
else fail "sin_seccion del log cuenta" "$OUT"; fi

if ! contains "$OUT" "sin_git" && ! contains "$OUT" "error_engine" && ! contains "$OUT" "engine_stale" \
   && ! contains "$OUT" "nope"; then pass "sin_git, errores y otros reflejos no son huecos"
else fail "sin_git, errores y otros reflejos no son huecos" "$OUT"; fi

OUT="$(STUB_RC=2 STUB_OUT='' "$SCRIPT")"; RC=$?
if [ "$RC" -eq 0 ] && contains "$OUT" "no conoce «rules --coverage»" && contains "$OUT" "sin_nota	foo"; then
  pass "engine viejo: avisa y sigue con el log"
else fail "engine viejo: avisa y sigue con el log" "rc=$RC $OUT"; fi

OUT="$(REFLEX_LOG_FILE="$TMP/no-existe" "$SCRIPT")"; RC=$?
if [ "$RC" -eq 0 ] && contains "$OUT" "(sin reflex-log"; then pass "sin reflex-log: sale 0"
else fail "sin reflex-log: sale 0" "rc=$RC $OUT"; fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
