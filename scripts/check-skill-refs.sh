#!/usr/bin/env bash
# Toda referencia `exo:<nombre>` en los .md de <plugin_dir>/skills y
# <plugin_dir>/agents debe resolver a skills/<nombre>/SKILL.md o agents/<nombre>.md.
# Uso: check-skill-refs.sh [<plugin_dir>]   (default plugins/exo)
set -uo pipefail
DIR="${1:-plugins/exo}"
[ -d "$DIR" ] || { echo "check-skill-refs: no existe $DIR" >&2; exit 1; }

# Cero ficheros escaneados sería un verde sin haber mirado nada.
mapfile -t FICHEROS < <(find "$DIR/skills" "$DIR/agents" -type f -name '*.md' 2>/dev/null | sort)
if [ "${#FICHEROS[@]}" -eq 0 ]; then
  echo "check-skill-refs: ningún .md bajo $DIR/skills ni $DIR/agents" >&2
  exit 1
fi

ROTAS=0
# El nombre no termina en guion: `exo:plan-` o `exo:plan.` reconocen `plan`.
while IFS=: read -r f n ref; do
  nombre="${ref#exo:}"
  if [ ! -f "$DIR/skills/$nombre/SKILL.md" ] && [ ! -f "$DIR/agents/$nombre.md" ]; then
    echo "$f:$n: exo:$nombre no existe"
    ROTAS=$((ROTAS+1))
  fi
done < <(grep -noE 'exo:[a-z0-9]+(-[a-z0-9]+)*' "${FICHEROS[@]}" /dev/null)
[ "$ROTAS" -eq 0 ]
