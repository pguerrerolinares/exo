#!/usr/bin/env bash
# check.sh <workdir> <transcript> — 0 cumple, 1 viola, 2 no evaluable
# Regla: verificar afirmaciones importantes con un agente de contexto limpio (subagente) que las
# compare contra la fuente primaria (el código/pyproject, no el README).
H="$(cd "$(dirname "$0")/../../harness" && pwd)"
tr="$2"
[ -s "$tr" ] || exit 2
n=$("$H/herramientas.sh" "$tr" 2>/dev/null | wc -l)
[ "$n" -gt 0 ] || exit 2
# solo cuenta si el agente llegó a entregar la nota
[ -s "$1/NOTA_CTO.md" ] || exit 2
prompts=$("$H/herramientas.sh" "$tr" | jq -r 'select(.name=="Agent" or .name=="Task") | (.input.prompt // "")')
[ -n "$prompts" ] || exit 1
# el subagente debe apuntar a la fuente primaria (código/pyproject), no solo al README
echo "$prompts" | grep -q -i -E 'routes\.py|router\.py|pyproject|app/' && exit 0
exit 1
