#!/usr/bin/env bash
# run.sh <brazo> <id> <prompt> — una corrida aislada de la campaña K sobre un clon
# desechable de exo@main. Salida en $K_OUT (default ~/.cache/exo-ablacion-k/sondas).
# Aislamiento verificado en evals/ablacion-k/task0-recon.md.
set -u
H="$(cd "$(dirname "$0")" && pwd)"
a=$1; id=$2; q=$3
OUT="${K_OUT:-$HOME/.cache/exo-ablacion-k/sondas}"; mkdir -p "$OUT"
w="$OUT/w-$id-$a"
rm -rf "$w"; git clone -q --no-hardlinks "$H/../../.." "$w" && git -C "$w" remote remove origin
cd "$w" || exit 1; t0=$(date +%s)
REFLEX_LOG_FILE="$OUT/reflex-$id-$a.jsonl" timeout 900 claude -p --model claude-sonnet-5-5 \
  --setting-sources "" --strict-mcp-config --settings "$H/$a.json" \
  --append-system-prompt-file "$HOME/.claude/CLAUDE.md" \
  --permission-mode bypassPermissions --max-turns 40 --no-session-persistence \
  --output-format stream-json --verbose "$q" < /dev/null > "$OUT/out-$id-$a.jsonl" 2> "$OUT/err-$id-$a.log"
rc=$?; t1=$(date +%s)
grep '"type":"result"' "$OUT/out-$id-$a.jsonl" | tail -1 | jq -c --arg id "$id" --arg a "$a" --arg s "$((t1-t0))" --arg rc "$rc" \
 '{id:$id, brazo:$a, rc:$rc, seg:($s|tonumber), usd:.total_cost_usd, turnos:.num_turns, in:(.usage.input_tokens+.usage.cache_read_input_tokens+.usage.cache_creation_input_tokens), out:.usage.output_tokens, fin:.terminal_reason}' \
 || echo "{\"id\":\"$id\",\"brazo\":\"$a\",\"rc\":\"$rc\",\"error\":\"sin result\"}"
