#!/usr/bin/env bash
# Tests de guardar_sesion (sesion.sh), la limpieza de la sesión persistida del brazo a2s. Todo bajo mktemp -d; no toca ~/.claude.
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
fail=0
ok(){ echo "ok   $1"; }; ko(){ echo "FAIL $1"; fail=1; }
. "$HERE/sesion.sh"

sdir="$T/home/.claude/projects/-x-work"; O="$T/corrida"
mkdir -p "$sdir/u1/subagents" "$sdir/otra" "$O" "$T/home/.claude/projects/-y-otro-work/u9"
echo '{"a":1}' > "$sdir/u1.jsonl"; echo sub > "$sdir/u1/subagents/agent-1.jsonl"; echo ajeno > "$sdir/otra/f"
echo '{"b":2}' > "$T/home/.claude/projects/-y-otro-work/u9.jsonl"

guardar_sesion "$sdir" "$O"
# sesion_con_subagentes: jsonl a sesion.jsonl, árbol de la sesión a sesion-sub/<uuid>, nada de ella queda en sdir
if [ "$(cat "$O/sesion.jsonl")" = '{"a":1}' ] && [ "$(cat "$O/sesion-sub/u1/subagents/agent-1.jsonl")" = sub ] \
   && [ ! -e "$sdir/u1.jsonl" ] && [ ! -e "$sdir/u1" ]; then ok sesion_con_subagentes; else ko sesion_con_subagentes; fi
# solo_esa_sesion: lo que no es de la sesión (otra carpeta en sdir, otro proyecto) queda intacto
if [ -f "$sdir/otra/f" ] && [ -f "$T/home/.claude/projects/-y-otro-work/u9.jsonl" ] && [ -d "$T/home/.claude/projects/-y-otro-work/u9" ]; then ok solo_esa_sesion; else ko solo_esa_sesion; fi
# guardas_de_vacio: con argumentos vacíos no hace nada
guardar_sesion "" "$O" 2>/dev/null; guardar_sesion "$sdir" "" 2>/dev/null
[ -f "$sdir/otra/f" ] && ok guardas_de_vacio || ko guardas_de_vacio
# sesion_sin_subagentes: el sdir vacío se elimina (comportamiento previo)
s2="$T/home/.claude/projects/-z-work"; O2="$T/c2"; mkdir -p "$s2" "$O2"; echo '{"c":3}' > "$s2/u2.jsonl"
guardar_sesion "$s2" "$O2"
[ "$(cat "$O2/sesion.jsonl")" = '{"c":3}' ] && [ ! -e "$s2" ] && [ ! -e "$O2/sesion-sub" ] && ok sesion_sin_subagentes || ko sesion_sin_subagentes
exit $fail
