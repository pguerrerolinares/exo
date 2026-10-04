#!/usr/bin/env bash
# sondas.sh — Task 0: coste por corrida con 3 tareas fuera del pool, brazos A0 y A3.
set -u
H="$(cd "$(dirname "$0")" && pwd)"
OUT="${K_OUT:-$HOME/.cache/exo-ablacion-k/sondas}"; mkdir -p "$OUT"; : > "$OUT/costes.jsonl"
while IFS=$'\t' read -r id q; do
  for a in a0 a3; do "$H/run.sh" "$a" "$id" "$q" >> "$OUT/costes.jsonl" & done; wait
done < "$H/tareas-sonda.tsv"
cat "$OUT/costes.jsonl"
