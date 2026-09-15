#!/usr/bin/env bash
# Bench de coste de la campaña A. Criterios, escenarios y predicciones:
# docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md
#
# Uso: evals/recall-coste/harness/bench.sh <etiqueta> [N ...]   (p.ej. baseline 174 1000 5000)
# Deja en evals/recall-coste/results/<etiqueta>/ un JSON de hyperfine, el rc y
# el stderr de una corrida directa por escenario, entorno.txt y resumen.tsv.
# Solo Linux. No toca ~/.exo ni la KB real: todo vive en BENCH_WORK.
set -uo pipefail
cd "$(dirname "$0")/../../.." || exit 1
REPO="$PWD"

ETIQUETA="${1:-}"
[ -n "$ETIQUETA" ] || { echo "uso: bench.sh <etiqueta> [N ...]" >&2; exit 1; }
shift
TAMANOS=("$@")
[ "${#TAMANOS[@]}" -gt 0 ] || TAMANOS=(174 1000 5000)
RUNS="${BENCH_RUNS:-20}"
# Solo palabras del vocabulario del generador: FTS es AND implícito y una
# palabra fuera del corpus («como», «de») daba «recall vacío» (exit 1) y
# confundía P1 (medido en el smoke del plan, 2026-09-13).
Q='trinquete techos indice memoria'
BIN="$REPO/engine/target/release/exo"
GEN="$REPO/engine/target/release/examples/kb_sintetica"
OUT="$REPO/evals/recall-coste/results/$ETIQUETA"

command -v hyperfine >/dev/null 2>&1 || { echo "bench: falta hyperfine" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "bench: falta jq" >&2; exit 1; }
[ ! -e "$OUT" ] || { echo "bench: $OUT ya existe — una etiqueta no se sobrescribe" >&2; exit 1; }

cargo build --release --locked --manifest-path engine/Cargo.toml --bin exo --example kb_sintetica || exit 1

WORK="${BENCH_WORK:-$(mktemp -d)}"
[ -n "${BENCH_KEEP:-}" ] || trap 'rm -rf "$WORK"' EXIT
mkdir -p "$OUT"

{
  echo "etiqueta $ETIQUETA"
  echo "fecha $(date -Iseconds)"
  echo "commit $(git rev-parse HEAD)"
  echo "src_y_scripts_vs_3c1918f $(git diff --quiet 3c1918f -- engine/src plugins/exo/scripts && echo vacio || echo CAMBIOS)"
  echo "exo $("$BIN" --version)"
  uname -a
  echo "nproc $(nproc)"
  echo "uptime $(uptime)"
  hyperfine --version
  jq --version
  git --version
} > "$OUT/entorno.txt"

mide() {  # $1=id  $2=comando para sh -c
  local id="$1" cmd="$2"
  sh -c "$cmd" > /dev/null 2> "$OUT/$id.stderr"
  echo $? > "$OUT/$id.rc"
  hyperfine --warmup 3 --runs "$RUNS" --ignore-failure \
    --export-json "$OUT/$id.json" "$cmd" > /dev/null 2>> "$OUT/$id.stderr"
}

for N in "${TAMANOS[@]}"; do
  D="$WORK/n$N"
  "$GEN" "$N" "$D" 42 > "$OUT/generador-n$N.txt" || { echo "bench: el generador falló para N=$N" >&2; exit 1; }
  export EXO_CONFIG="$D/config.toml"
  KB="$D/kb"
  DB="$D/index.db"

  # Fidelidad: el índice sintético tiene que verse FRESCO; si `exo index`
  # reindexa algo, el bench mediría un indexado y no un recall.
  "$BIN" index --kb "$KB" --db "$DB" --json > "$OUT/fidelidad-n$N.json" || exit 1
  jq -e --argjson n "$N" '.data.indexed == 0 and .data.skipped == $n' "$OUT/fidelidad-n$N.json" >/dev/null || {
    echo "bench: índice sintético no fresco para N=$N: $(cat "$OUT/fidelidad-n$N.json")" >&2; exit 1; }

  # Ola 1 G Task 1 (fix del orquestador, 2): el arm vector AISLADO
  # (`search --type vector`, no `recall`/`hybrid`) tiene que aportar algo
  # con el umbral de producción, o el bench mide solo el canal FTS aunque
  # diga "hybrid" — `busca_hybrid` fusiona por UNIÓN, así que un `recall`
  # que devuelve notas no prueba que el arm vector participó.
  "$BIN" search --db "$DB" --kb "$KB" --type vector --min-similarity 0.40 \
    --limit 4 --json "$Q" > "$OUT/cobertura-vector-n$N.json" || exit 1
  jq -e '.data.results | length > 0' "$OUT/cobertura-vector-n$N.json" >/dev/null || {
    echo "bench: KB sintética ciega al umbral: el brazo vector no devuelve nada con 0.40 (N=$N)" >&2
    exit 1; }

  jq -n --arg p "$Q" '{prompt:$p, session_id:"bench-a"}' > "$D/prompt.json"

  mide "s3-index-sin-cambios-n$N" "\"$BIN\" index --db \"$DB\" --kb \"$KB\" --json"
  mide "s5-search-fts-n$N" "\"$BIN\" search --db \"$DB\" --type fts --limit 5 --json 'trinquete techos'"
  mide "s2-query-n$N" "\"$BIN\" recall --db \"$DB\" --kb \"$KB\" '--query=$Q' --min-similarity 0.40 --limit 4 --cap-bytes 4000 --json"
  mide "s1-query-refresh-n$N" "\"$BIN\" recall --db \"$DB\" --kb \"$KB\" '--query=$Q' --min-similarity 0.40 --limit 4 --cap-bytes 4000 --refresh --json"
  mide "s1b-query-refresh-sim0-n$N" "\"$BIN\" recall --db \"$DB\" --kb \"$KB\" '--query=$Q' --min-similarity 0.0 --limit 4 --cap-bytes 4000 --refresh --json"
  mide "s4-arranque-content-n$N" "\"$BIN\" recall --db \"$DB\" --kb \"$KB\" --content --note sint/core/core-index --limit 10 --cap-bytes 6144"
  mide "s6-hook-entero-n$N" "EXO_BIN=\"$BIN\" EXO_INDEX=\"$DB\" REFLEX_LOG_FILE=\"$D/reflex-log.jsonl\" \"$REPO/plugins/exo/scripts/recall-inject.sh\" < \"$D/prompt.json\""
  mide "s7-config-jq-n$N" "\"$BIN\" config --json | jq -r .data.kb.name"
  mide "s9-git-log-una-nota-n$N" "git -C \"$KB\" log -1 --format=%ct -- log/nota-00011.md"

  # s8 va el último: deja una nota modificada y staged.
  printf '\nlinea extra del bench\n' >> "$KB/log/nota-00011.md"
  git -C "$KB" add log/nota-00011.md
  mide "s8-kb-precommit-n$N" "cd \"$KB\" && EXO_BIN=\"$BIN\" \"$REPO/plugins/exo/scripts/kb-precommit.sh\""

  [ -n "${BENCH_KEEP:-}" ] || rm -rf "$D"
done

# s10, independiente de N: el filtro de exo-recall.sh:111-112 sobre logs de ~410 B/línea.
for L in 2174 20000 200000; do
  F="$WORK/reflex-log-$L.jsonl"
  awk -v n="$L" 'BEGIN {
    pad = sprintf("%260s", ""); gsub(/ /, "x", pad)
    for (i = 0; i < n; i++)
      printf "{\"ts\":\"2026-09-13T00:00:00Z\",\"reflex\":\"recall-inject-emitted\",\"session_id\":\"s%d\",\"agent_id\":\"\",\"agent_type\":\"\",\"tool\":\"\",\"payload\":\"n_hits=3 bytes=900 permalinks=%s\"}\n", i % 500, pad
  }' > "$F"
  mide "s10-jq-log-l$L" "jq -r --arg sid s7 'select(.session_id==\$sid) | .reflex' \"$F\" | sort -u"
done

{
  printf 'id\tp50_ms\tp95_ms\tmean_ms\tcorridas_fallidas\tcorridas\trc_directa\n'
  for f in "$OUT"/s*.json; do
    id="$(basename "$f" .json)"
    rc="$(cat "$OUT/$id.rc" 2>/dev/null)"
    jq -r --arg id "$id" --arg rc "$rc" '
      .results[0] as $r | ($r.times | sort) as $t | ($t | length) as $n
      | [ $id,
          ($t[(($n - 1) * 0.5 | floor)] * 1000 | floor),
          ($t[(($n - 1) * 0.95 | floor)] * 1000 | floor),
          ($r.mean * 1000 | floor),
          ([ ($r.exit_codes // [])[] | select(. != 0) ] | length),
          $n, $rc ] | map(tostring) | @tsv' "$f"
  done | sort
} > "$OUT/resumen.tsv"

column -t -s "$(printf '\t')" "$OUT/resumen.tsv"
