#!/usr/bin/env bash
# Test standalone para recall-latencia.sh. Logs sintéticos en mktemp -d.
# shellcheck disable=SC2015 # `[ … ] && pass || fail` es el idioma de aserción de esta suite: pass nunca falla
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="${SCRIPT_DIR}/recall-latencia.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

# emite <n> <base_ms> <refresh_ms> <session> <fecha>
# `hook_ms` del payload sintético = base_ms + refresh_ms (fix de la review
# adversarial 2026-09-19): las CINCO llamadas a `emite` de este fichero ya
# existían antes de la campaña I con estos mismos números pensados como
# "elapsed_ms + refresh_ms suman la latencia total del caso" (ver los
# comentarios de cada caso, p. ej. "p95 = 910" = 900+10). Si `hook_ms`
# fuera solo `base_ms` sin sumar `refresh_ms`, el caso 1 daría p95=900 en vez
# de los 910 que su propia aserción espera — desalineación real, detectada
# corriendo el test, no solo leyéndolo. Sumar aquí preserva las CINCO
# aserciones existentes sin tocarlas: no es una fórmula real de producción
# (hook_ms de verdad es un número medido, no una suma), es una elección de
# este fixture sintético para no reescribir comentarios y aserciones que ya
# estaban bien.
emite() {
  awk -v n="$1" -v b="$2" -v r="$3" -v s="$4" -v d="$5" 'BEGIN {
    for (i = 0; i < n; i++)
      printf "{\"ts\":\"%sT10:00:00Z\",\"reflex\":\"recall-inject-emitted\",\"session_id\":\"%s\",\"agent_id\":\"\",\"agent_type\":\"\",\"tool\":\"\",\"payload\":\"n_hits=3 bytes=900 elapsed_ms=999 refresh_ms=%d hook_ms=%d permalinks=kb/a,kb/b\"}\n", d, s, r, (b+r)
  }'
}
timeouts() {  # timeouts <n> <fecha>
  awk -v n="$1" -v d="$2" 'BEGIN { for (i = 0; i < n; i++)
    printf "{\"ts\":\"%sT10:00:00Z\",\"reflex\":\"recall-inject-degraded\",\"session_id\":\"s\",\"agent_id\":\"\",\"agent_type\":\"\",\"tool\":\"\",\"payload\":\"reason=timeout-guard t=5s\"}\n", d }'
}
# viejos <n> <fecha>: `emitted` sin hook_ms (anteriores al despliegue de la
# campaña I, como el caso 6, pero repetible en cantidad).
viejos() {
  awk -v n="$1" -v d="$2" 'BEGIN { for (i = 0; i < n; i++)
    printf "{\"ts\":\"%sT10:00:00Z\",\"reflex\":\"recall-inject-emitted\",\"session_id\":\"s\",\"agent_id\":\"\",\"agent_type\":\"\",\"tool\":\"\",\"payload\":\"n_hits=3 bytes=900 permalinks=kb/a\"}\n", d }'
}
veredicto() { printf '%s\n' "$1" | awk -F'\t' '$1 == "veredicto" { print $2 }'; }
campo() { printf '%s\n' "$2" | awk -F'\t' -v k="$1" '$1 == k { print $2 }'; }

# 1. Sano: 200 a 900+10 ms y 10 a 1990+10 ms → p95 = 910 → NO-REABRIR.
{ emite 200 900 10 s 2026-10-01; emite 10 1990 10 s 2026-10-01; } > "$TMP/sano.jsonl"
OUT="$(REFLEX_LOG_FILE="$TMP/sano.jsonl" bash "$SCRIPT")"
[ "$(veredicto "$OUT")" = "NO-REABRIR" ] && [ "$(campo p95_ms "$OUT")" = "910" ] \
  && pass "sano: NO-REABRIR con p95=910" || fail "sano" "$OUT"

# 2. Lento: 100 a 900 y 110 a 2000 → p95 = 2010 > 1500 → REABRIR.
{ emite 100 900 10 s 2026-10-01; emite 110 2000 10 s 2026-10-01; } > "$TMP/lento.jsonl"
OUT="$(REFLEX_LOG_FILE="$TMP/lento.jsonl" bash "$SCRIPT")"
[ "$(veredicto "$OUT")" = "REABRIR" ] && pass "lento: REABRIR por p95" || fail "lento" "$OUT"

# 3. Timeouts: 200 sanos y 10 timeouts → 4,76% > 2% → REABRIR.
{ emite 200 900 10 s 2026-10-01; timeouts 10 2026-10-01; } > "$TMP/to.jsonl"
OUT="$(REFLEX_LOG_FILE="$TMP/to.jsonl" bash "$SCRIPT")"
[ "$(veredicto "$OUT")" = "REABRIR" ] && [ "$(campo timeouts "$OUT")" = "10" ] \
  && pass "timeouts: REABRIR por >2%" || fail "timeouts" "$OUT"

# 4. Pocos datos: 50 disparos → INSUFICIENTE, aunque sean lentos.
emite 50 3000 10 s 2026-10-01 > "$TMP/poco.jsonl"
OUT="$(REFLEX_LOG_FILE="$TMP/poco.jsonl" bash "$SCRIPT")"
case "$(veredicto "$OUT")" in INSUFICIENTE*) pass "pocos datos: INSUFICIENTE" ;; *) fail "pocos datos" "$OUT" ;; esac

# 5. Las sesiones test* no cuentan, y la ventana de fechas filtra.
{ emite 250 3000 10 test-sess 2026-10-01; emite 250 900 10 s 2026-09-01; emite 210 900 10 s 2026-10-05; } > "$TMP/filtro.jsonl"
OUT="$(REFLEX_LOG_FILE="$TMP/filtro.jsonl" bash "$SCRIPT" 2026-10-01 2026-10-14)"
[ "$(campo disparos_medidos "$OUT")" = "210" ] && [ "$(veredicto "$OUT")" = "NO-REABRIR" ] \
  && pass "filtro: excluye test* y fuera de ventana" || fail "filtro" "$OUT"

# 6. Payloads sin tiempos (anteriores a la campaña A) no rompen ni cuentan.
{ printf '{"ts":"2026-10-01T10:00:00Z","reflex":"recall-inject-emitted","session_id":"s","agent_id":"","agent_type":"","tool":"","payload":"n_hits=3 bytes=900 permalinks=kb/a"}\n'; emite 200 900 10 s 2026-10-01; } > "$TMP/viejo.jsonl"
OUT="$(REFLEX_LOG_FILE="$TMP/viejo.jsonl" bash "$SCRIPT")"
[ "$(campo disparos_medidos "$OUT")" = "200" ] && pass "viejos: se ignoran los payloads sin elapsed_ms" || fail "viejos" "$OUT"

# 7. Log inexistente → exit 1.
REFLEX_LOG_FILE="$TMP/no-existe.jsonl" bash "$SCRIPT" >/dev/null 2>&1
[ $? -eq 1 ] && pass "sin log: exit 1" || fail "sin log" "exit distinto de 1"

# 8. Timeouts previos al despliegue de hook_ms no cuentan (review final de
# rama, 2026-09-20): 300 emitted viejos sin hook_ms + 5 timeout-guard viejos
# (mismo rango de fechas) + 200 emitted nuevos sanos con hook_ms. Antes del
# fix, $to contaba los 5 timeouts viejos contra un $n que solo veía los 200
# nuevos → timeout_pct = 5/205 = 2,44% > 2% → REABRIR (falso: sin DESDE, el
# script sin arreglar da justo eso). Con el fix, los timeouts anteriores a
# t0 (el primer `ts` con hook_ms) no cuentan → 0% → NO-REABRIR.
{ viejos 300 2026-09-01; timeouts 5 2026-09-10; emite 200 900 10 s 2026-09-21; } > "$TMP/despliegue.jsonl"
OUT="$(REFLEX_LOG_FILE="$TMP/despliegue.jsonl" bash "$SCRIPT")"
[ "$(veredicto "$OUT")" = "NO-REABRIR" ] && [ "$(campo timeouts "$OUT")" = "0" ] \
  && pass "despliegue: timeouts previos al primer hook_ms no cuentan (no REABRIR falso)" \
  || fail "despliegue: timeouts previos al primer hook_ms no cuentan" "$OUT"

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
