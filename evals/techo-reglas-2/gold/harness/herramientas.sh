#!/usr/bin/env bash
# herramientas.sh <transcript stream-json> — una línea JSON por tool_use: {name, input}.
jq -c 'select(.type=="assistant") | .message.content[]? | select(.type=="tool_use") | {name, input}' "$1"
