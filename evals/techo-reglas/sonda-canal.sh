#!/usr/bin/env bash
# Sonda: ¿llega additionalContext de un hook SessionStart en el modo headless de correr.sh?
# exit 0 si el codeword vuelve, 1 si no. Lanza un claude real (coste ~0,0x USD).
set -uo pipefail
MODELO="${K_MODELO:-claude-sonnet-5-5}"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
git -C "$T" init -q
jq -n '{hookSpecificOutput:{hookEventName:"SessionStart",additionalContext:"El codeword es TECHO-7Q"}}' > "$T/ctx.json"
jq -n --arg c "cat $T/ctx.json" '{autoMemoryEnabled:false, hooks:{SessionStart:[{hooks:[{type:"command",command:$c}]}]}}' > "$T/settings.json"
cd "$T" || exit 1
out=$(DISABLE_AUTOUPDATER=1 timeout 300 claude -p --model "$MODELO" --setting-sources "" --strict-mcp-config \
  --settings "$T/settings.json" --permission-mode bypassPermissions --max-turns 3 --max-budget-usd 0.10 \
  --no-session-persistence --output-format json "¿Cuál es el codeword? Responde solo el codeword." < /dev/null 2> "$T/err.log")
resp=$(printf '%s' "$out" | jq -r '.result // empty')
usd=$(printf '%s' "$out" | jq -r '.total_cost_usd // "?"')
if [[ $resp == *TECHO-7Q* ]]; then echo "sonda: OK TECHO-7Q (usd=$usd)"; exit 0; fi
echo "sonda: FALLA, respuesta='$resp' (usd=$usd)" >&2; head -5 "$T/err.log" >&2; exit 1
