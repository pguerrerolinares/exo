#!/usr/bin/env bash
# correr.sh <dir de tarea> <brazo a0|a1|a2|a3|ar> <réplica> — una corrida de la fase 1 de la campaña K.
# Requiere $K_ROOT/prep/ (preparar.sh). Deja en $K_ROOT/corridas/<tarea>/<brazo>-r<rep>/:
#   work/ (el repo tal como lo deja el agente), transcript.jsonl, reflex.jsonl, err.log, meta.json
# Brazos (§4 + erratas E1–E3):
#   todos  --setting-sources "" · autoMemory off · CLAUDE.md sin la sección de memoria (E1)
#          · lecturas de la KB de producción y de ~/.exo prohibidas · --max-budget-usd 10 (E3)
#   a0     sin bloque de arranque · exo = stub · lecturas del snapshot prohibidas (E1)
#   a1     bloque de arranque con búsqueda por grep · exo = stub
#   a2     bloque de arranque real (exo-recall.sh) · exo apunta al snapshot
#   a3     a2 + recall-inject.sh en cada prompt
#   ar     a0 exacto + SessionStart que inyecta la regla literal de $K_REGLA_FILE (techo de reglas)
#   arp    a0 exacto + sysprompt.md (claude-md.md + framing con la regla) por --append-system-prompt-file;
#          requiere K_REGLA_FILE y K_FRAMING_FILE (techo-reglas-2)
# K_ENSAYO=1: genera settings.json y cmdline.txt (deny, PATH) y sale sin lanzar claude.
set -uo pipefail
tarea=$1; brazo=$2; rep=$3
H="$(cd "$(dirname "$0")" && pwd)"
K_ROOT="${K_ROOT:-$HOME/.cache/exo-ablacion-k}"; P="$K_ROOT/prep"
PLUG="$(cd "$H/../../../plugins/exo/scripts" && pwd)"
MODELO="${K_MODELO:-claude-sonnet-5-5}"
if [ "$brazo" = ar ] && [ ! -s "${K_REGLA_FILE:-}" ]; then
  echo "ar requiere K_REGLA_FILE no vacío" >&2; exit 2
fi
if [ "$brazo" = arp ] && { [ ! -s "${K_REGLA_FILE:-}" ] || [ ! -s "${K_FRAMING_FILE:-}" ]; }; then
  echo "arp requiere K_REGLA_FILE y K_FRAMING_FILE no vacíos" >&2; exit 2
fi
if [ "$brazo" = arp ] && [ "$(grep -o '{{REGLA}}' "$K_FRAMING_FILE" | wc -l)" != 1 ]; then
  echo "arp: K_FRAMING_FILE debe contener {{REGLA}} exactamente una vez" >&2; exit 2
fi
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
hooks='{}'; append="$P/claude-md.md"
# a3 usa recall-inject.sh, borrado tras la campaña K; reproducir a3 desde el commit b94ed74.
case $brazo in
  a1) hooks=$(jq -n --arg c "cat $P/a1-inicio.json" '{SessionStart:[{hooks:[{type:"command",command:$c}]}]}') ;;
  a2) hooks=$(jq -n --arg c "$PLUG/exo-recall.sh" '{SessionStart:[{hooks:[{type:"command",command:$c}]}]}') ;;
  a3) hooks=$(jq -n --arg c "$PLUG/exo-recall.sh" --arg u "$PLUG/recall-inject.sh" \
        '{SessionStart:[{hooks:[{type:"command",command:$c}]}],UserPromptSubmit:[{hooks:[{type:"command",command:$u}]}]}') ;;
  a0) ;;
  ar) jq -Rs '{hookSpecificOutput:{hookEventName:"SessionStart",additionalContext:.}}' "$K_REGLA_FILE" > "$O/regla-ctx.json" || exit 2
      hooks=$(jq -n --arg c "cat $O/regla-ctx.json" '{SessionStart:[{hooks:[{type:"command",command:$c}]}]}') ;;
  arp) # python y no bash: $(...) recortaría los saltos de línea finales de la regla.
      python3 - "$P/claude-md.md" "$K_FRAMING_FILE" "$K_REGLA_FILE" > "$O/sysprompt.md" <<'PY' || exit 2
import sys
base, framing, regla = (open(p, "rb").read() for p in sys.argv[1:4])
sys.stdout.buffer.write(base + b"\n" + framing.replace(b"{{REGLA}}", regla))
PY
      append="$O/sysprompt.md" ;;
  *) echo "brazo desconocido: $brazo" >&2; exit 2 ;;
esac
jq -n --argjson h "$hooks" '{autoMemoryEnabled:false, hooks:$h}' > "$O/settings.json"

deny=("Read(//home/paul/Documentos/proyectos/wisdom-paul/**)" "Read(//home/paul/.exo/**)"
      "Grep(//home/paul/Documentos/proyectos/wisdom-paul/**)" "Grep(//home/paul/.exo/**)")
{ [ "$brazo" = a0 ] || [ "$brazo" = ar ] || [ "$brazo" = arp ]; } && deny+=("Read(/$P/kb/**)" "Grep(/$P/kb/**)")
ruta="$PATH"; case $brazo in a0|a1|ar|arp) ruta="$P/stub:$PATH" ;; esac

# S2: el agente usa el venv del repo (solo lectura) y el código de su workdir.
extra_env=()
if [ -f "$tarea/meta.json" ]; then
  s2repo=$(jq -r .repo "$tarea/meta.json"); venv="$K_ROOT/s2/venv-$s2repo"
  ruta="$venv/bin:$ruta"
  extra_env=(VIRTUAL_ENV="$venv" PYTHONPATH="$O/work/src:$O/work")
  [ "$s2repo" = django-oscar ] && extra_env+=(DATABASE_ENGINE=django.db.backends.sqlite3 DATABASE_NAME=:memory:)
fi
if [ "${K_ENSAYO:-0}" = 1 ]; then
  echo "correr.sh: modo ensayo, no se lanza claude" >&2
  { printf 'PATH=%s\n' "$ruta"; printf 'deny=%s\n' "${deny[@]}"; printf 'append=%s\n' "$append"; } > "$O/cmdline.txt"; exit 0
fi
prompt=$(jq -r .prompt "$t_json")
cd "$O/work" || exit 3; t0=$(date +%s)
env "${extra_env[@]}" PATH="$ruta" EXO_CONFIG="$P/config.toml" EXO_KB="$P/kb" EXO_DB="$P/index.db" EXO_INDEX="$P/index.db" \
EXO_BIN="$(command -v exo)" REFLEX_LOG_FILE="$O/reflex.jsonl" \
DISABLE_AUTOUPDATER=1 timeout 1800 claude -p --model "$MODELO" --setting-sources "" --strict-mcp-config \
  --settings "$O/settings.json" --append-system-prompt-file "$append" \
  --disallowedTools "${deny[@]}" \
  --permission-mode bypassPermissions --max-turns 40 --max-budget-usd 10 --no-session-persistence \
  --output-format stream-json --verbose "$prompt" < /dev/null > "$O/transcript.jsonl" 2> "$O/err.log"
rc=$?; t1=$(date +%s)
claude --version > "$O/claude-version.txt" 2>/dev/null
jq -c --arg id "$id" --arg b "$brazo" --arg r "$rep" --arg rc "$rc" --arg s "$((t1-t0))" \
  'select(.type=="result") | {tarea:$id, brazo:$b, rep:($r|tonumber), rc:($rc|tonumber), seg:($s|tonumber),
   fin:.terminal_reason, turnos:.num_turns, usd:.total_cost_usd,
   tokens_in:(.usage.input_tokens+.usage.cache_read_input_tokens+.usage.cache_creation_input_tokens),
   tokens_out:.usage.output_tokens}' "$O/transcript.jsonl" | tail -1 > "$O/meta.json"
[ -s "$O/meta.json" ] || echo "{\"tarea\":\"$id\",\"brazo\":\"$brazo\",\"rep\":$rep,\"rc\":$rc,\"error\":\"sin result\"}" > "$O/meta.json"
# Check (contrato E8.3: workdir, transcript, commit inicial) y limpieza de disco.
if [ -x "$tarea/check.sh" ]; then
  bash "$tarea/check.sh" "$O/work" "$O/transcript.jsonl" "$(cat "$O/inicio.txt" 2>/dev/null)" > "$O/check.log" 2>&1
  echo $? > "$O/check.rc"
fi
python3 "$H/fugas.py" "$O" "$brazo" > "$O/fugas.json" 2>/dev/null
git -C "$O/work" diff "$(cat "$O/inicio.txt" 2>/dev/null)" > "$O/diff.patch" 2>/dev/null
git -C "$O/work" status --porcelain > "$O/status.txt" 2>/dev/null
[ "${K_CONSERVAR_WORK:-0}" = 1 ] || rm -rf "$O/work"
cat "$O/meta.json"
