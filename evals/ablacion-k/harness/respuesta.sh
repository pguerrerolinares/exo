#!/usr/bin/env bash
# respuesta.sh <transcript stream-json> — imprime la respuesta final del agente.
jq -r 'select(.type=="result") | .result // empty' "$1" | tail -n +1
