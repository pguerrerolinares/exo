#!/usr/bin/env bash
# correr.sh <dir de tarea> <brazo a0|a1|a2|a3> <réplica> — una corrida de la fase 1 de la campaña K.
# Requiere $K_ROOT/prep/ (preparar.sh). Deja en $K_ROOT/corridas/<tarea>/<brazo>-r<rep>/:
#   work/ (el repo tal como lo deja el agente), transcript.jsonl, reflex.jsonl, err.log, meta.json
# Brazos (§4 + erratas E1–E3):
#   todos  --setting-sources "" · autoMemory off · CLAUDE.md sin la sección de memoria (E1)
#          · lecturas de la KB de producción y de ~/.exo prohibidas · --max-budget-usd 10 (E3)
#   a0     sin bloque de arranque · exo = stub · lecturas del snapshot prohibidas (E1)
#   a1     bloque de arranque con búsqueda por grep · exo = stub
#   a2     bloque de arranque real (exo-recall.sh) · exo apunta al snapshot
#   a3     a2 + recall-inject.sh en cada prompt
set -uo pipefail
tarea=$1; brazo=$2; rep=$3
H="$(cd "$(dirname "$0")" && pwd)"
K_ROOT="${K_ROOT:-$HOME/.cache/exo-ablacion-k}"; P="$K_ROOT/prep"
PLUG="$(cd "$H/../../../plugins/exo/scripts" && pwd)"
MODELO="${K_MODELO:-claude-sonnet-5-5}"
[ -f "$P/manifiesto.txt" ] || { echo "falta $P (preparar.sh)" >&2; exit 2; }
id=$(basename "$tarea"); O="$K_ROOT/corridas/$id/$brazo-r$rep"; rm -rf "$O"; mkdir -p "$O"
t_json="$tarea/tarea.json"

# Workdir: setup sintético o copia congelada de la fuente (congelar_fuentes.sh).
if [ "$(jq -r '.setup // false' "$t_json")" = true ]; then
  bash "$tarea/setup.sh" "$O/work" > "$O/setup.log" 2>&1 || { echo '{"error":"setup"}' > "$O/meta.json"; exit 3; }
elif [ -d "$K_ROOT/fuentes/$id" ]; then
  # Copia exacta congelada (congelar_fuentes.sh): incluye lo que no está en git.
  cp -a --reflink=auto "$K_ROOT/fuentes/$id" "$O/work" || { echo '{"error":"copia"}' > "$O/meta.json"; exit 3; }
else
  echo '{"error":"sin fuente congelada"}' > "$O/meta.json"; exit 3
fi
git -C "$O/work" rev-parse HEAD > "$O/inicio.txt" 2>/dev/null || true

# Settings del brazo.
hooks='{}'
case $brazo in
  a1) hooks=$(jq -n --arg c "cat $P/a1-inicio.json" '{SessionStart:[{hooks:[{type:"command",command:$c}]}]}') ;;
  a2) hooks=$(jq -n --arg c "$PLUG/exo-recall.sh" '{SessionStart:[{hooks:[{type:"command",command:$c}]}]}') ;;
  a3) hooks=$(jq -n --arg c "$PLUG/exo-recall.sh" --arg u "$PLUG/recall-inject.sh" \
        '{SessionStart:[{hooks:[{type:"command",command:$c}]}],UserPromptSubmit:[{hooks:[{type:"command",command:$u}]}]}') ;;
  a0) ;;
  *) echo "brazo desconocido: $brazo" >&2; exit 2 ;;
esac
jq -n --argjson h "$hooks" '{autoMemoryEnabled:false, hooks:$h}' > "$O/settings.json"

deny=("Read(//home/paul/Documentos/proyectos/wisdom-paul/**)" "Read(//home/paul/.exo/**)"
      "Grep(//home/paul/Documentos/proyectos/wisdom-paul/**)" "Grep(//home/paul/.exo/**)")
[ "$brazo" = a0 ] && deny+=("Read(/$P/kb/**)" "Grep(/$P/kb/**)")
ruta="$PATH"; [ "$brazo" = a0 ] || [ "$brazo" = a1 ] && ruta="$P/stub:$PATH"

prompt=$(jq -r .prompt "$t_json")
cd "$O/work" || exit 3; t0=$(date +%s)
PATH="$ruta" EXO_CONFIG="$P/config.toml" EXO_KB="$P/kb" EXO_DB="$P/index.db" EXO_INDEX="$P/index.db" \
EXO_BIN="$(command -v exo)" REFLEX_LOG_FILE="$O/reflex.jsonl" \
timeout 1800 claude -p --model "$MODELO" --setting-sources "" --strict-mcp-config \
  --settings "$O/settings.json" --append-system-prompt-file "$P/claude-md.md" \
  --disallowedTools "${deny[@]}" \
  --permission-mode bypassPermissions --max-turns 40 --max-budget-usd 10 --no-session-persistence \
  --output-format stream-json --verbose "$prompt" < /dev/null > "$O/transcript.jsonl" 2> "$O/err.log"
rc=$?; t1=$(date +%s)
jq -c --arg id "$id" --arg b "$brazo" --arg r "$rep" --arg rc "$rc" --arg s "$((t1-t0))" \
  'select(.type=="result") | {tarea:$id, brazo:$b, rep:($r|tonumber), rc:($rc|tonumber), seg:($s|tonumber),
   fin:.terminal_reason, turnos:.num_turns, usd:.total_cost_usd,
   tokens_in:(.usage.input_tokens+.usage.cache_read_input_tokens+.usage.cache_creation_input_tokens),
   tokens_out:.usage.output_tokens}' "$O/transcript.jsonl" | tail -1 > "$O/meta.json"
[ -s "$O/meta.json" ] || echo "{\"tarea\":\"$id\",\"brazo\":\"$brazo\",\"rep\":$rep,\"rc\":$rc,\"error\":\"sin result\"}" > "$O/meta.json"
cat "$O/meta.json"
