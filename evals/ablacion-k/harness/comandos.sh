#!/usr/bin/env bash
# comandos.sh <transcript stream-json> — imprime, uno por línea (JSON string), los comandos
# Bash que ejecutó el agente, en orden. Para checks de la campaña K.
jq -c 'select(.type=="assistant") | .message.content[]? | select(.type=="tool_use" and .name=="Bash") | .input.command' "$1"
