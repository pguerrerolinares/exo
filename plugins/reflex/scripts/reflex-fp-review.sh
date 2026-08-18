#!/usr/bin/env bash
# Digiere reflex-log.jsonl para el review de FP (paso 3). Agrupa por reflejo: cuenta
# disparos y vuelca cada uno (ts | reflejo | contexto padre/sub | payload) para
# adjudicar TP/FP. La rubrica + instrucciones de adjudicacion estan en el prompt
# canonico: reflex-fp-adjudicate.prompt.md (pasaselo a un agente junto con este volcado).
set -uo pipefail

LOG="${REFLEX_LOG_FILE:-$HOME/.claude/reflex-log.jsonl}"
if [ ! -f "$LOG" ]; then
  echo "No hay log de reflejos todavia: $LOG"
  echo "(Se crea solo cuando algun reflejo dispara por primera vez.)"
  exit 0
fi

echo "=============================================="
echo " REFLEX FP REVIEW — $(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null)"
echo " Log: $LOG"
echo "=============================================="
echo
echo "== Disparos por reflejo (volumen) =="
jq -r '.reflex' "$LOG" | sort | uniq -c | sort -rn
echo
echo "== Disparos por reflejo x contexto (padre vs subagente) =="
jq -r '.reflex + "\t" + (if .agent_id=="" then "padre" else "sub" end)' "$LOG" \
  | sort | uniq -c | sort -rn
echo
echo "== Detalle para adjudicar (ts | reflejo | ctx | payload) =="
jq -r '[.ts, .reflex, (if .agent_id=="" then "padre" else "sub:"+.agent_type end), .payload] | @tsv' "$LOG"
echo
echo "----------------------------------------------"
echo "Gate de escalado (umbrales de partida):"
echo "  review cuando un reflejo llega a >=10 disparos o pasan ~2 semanas."
echo "  FP-rate  <20% = sano (considerar escalar #6 a 'ask')"
echo "           20-50% = afinar abstencion"
echo "           >50%  = retirar / reworkear"
echo "Para el FP-rate: pasa este volcado + reflex-fp-adjudicate.prompt.md a un agente,"
echo "y haz spot-check de su clasificacion TP/FP."
