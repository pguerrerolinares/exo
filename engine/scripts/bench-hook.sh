#!/usr/bin/env bash
# Condición de invalidación 3 (spec 2026-11-02-exo-hook-multi-harness §7):
# `exo hook claude PreToolUse` sobre un journal de 2 MiB debe dar p99 < 300 ms.
# Caso medido: Edit de la sesión padre con search-first leyendo el journal
# entero (la lectura está acotada a 2 MiB) sin ningún `exo search` dentro.
# Imprime p99_ms=<n> y sale 0 si y solo si n < 300.
set -uo pipefail

AQUI="$(cd "$(dirname "$0")" && pwd)"
export EXO_BIN="${EXO_BIN:-$AQUI/../target/debug/exo}"
RUNS="${BENCH_RUNS:-200}"
LIMITE_MS=300

[ -x "$EXO_BIN" ] || { echo "bench: no existe el binario $EXO_BIN" >&2; exit 1; }
command -v hyperfine >/dev/null || { echo "bench: falta hyperfine" >&2; exit 1; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
export EXO_STATE_DIR="$TMP/estado" REFLEX_LOG_FILE="$TMP/reflex.jsonl"
mkdir -p "$EXO_STATE_DIR/sessions"

# Registros con la forma real del journal, de ~150 B cada uno, hasta 2 MiB.
JOURNAL="$EXO_STATE_DIR/sessions/claude-bench.jsonl"
awk 'BEGIN { for (i = 0; i < 20000; i++)
  printf "{\"ts\":\"2026-11-03T10:00:00Z\",\"kind\":\"tool\",\"tool\":\"Bash\",\"clase\":\"Shell\",\"cmd\":\"cargo test --workspace -p crate%d\",\"paths\":[],\"ok\":true,\"agent_id\":\"\",\"code\":\"\"}\n", i }' \
  | head -c $((2 * 1024 * 1024)) > "$JOURNAL"
printf '\n' >> "$JOURNAL"   # cierra la última línea cortada por head

printf '%s' '{"session_id":"bench","cwd":"/tmp","tool_name":"Edit","tool_input":{"file_path":"/tmp/x.rs"},"hook_event_name":"PreToolUse"}' > "$TMP/payload.json"

hyperfine -N --warmup 5 --runs "$RUNS" --input "$TMP/payload.json" \
  --export-json "$TMP/res.json" "$EXO_BIN hook claude PreToolUse" >/dev/null 2>"$TMP/err" \
  || { cat "$TMP/err" >&2; exit 1; }

P99="$(jq -r '.results[0].times | sort | .[((length * 0.99) | ceil) - 1] * 1000 | . * 10 | round / 10' "$TMP/res.json")"
[ -n "$P99" ] && [ "$P99" != null ] || { echo "bench: no se midió p99" >&2; exit 1; }
echo "journal_bytes=$(wc -c < "$JOURNAL") runs=$RUNS"
echo "p99_ms=$P99"
awk -v p="$P99" -v l="$LIMITE_MS" 'BEGIN { exit !(p < l) }'
