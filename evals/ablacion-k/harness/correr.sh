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
#   a2s    a2 + reflejo search-first.sh en PreToolUse (matcher de producción) · sentinel por corrida en $O (campaña M)
#   ar     a0 exacto + SessionStart que inyecta la regla literal de $K_REGLA_FILE (techo de reglas)
#   arp    a0 exacto + sysprompt.md (claude-md.md + framing con la regla) por --append-system-prompt-file;
#          requiere K_REGLA_FILE y K_FRAMING_FILE (techo-reglas-2)
#   arp<N> alias de arp con directorio propio (techo-reglas-3: K_REGLA_FILE trae el bloque entero)
#   n      a0 exacto (sin exo, sin snapshot) + memoria nativa de Claude Code activa sobre $O/memoria, copia fresca
#          de $K_MEMORIA_DIR (default $P/memoria) en cada corrida (campaña M)
#   ars    a0 exacto (claude-md.md sin framing) + solo el mod register.ts por --plugin-dir con EXO_RULES_FORZAR_SUBMIT=1:
#          entrega la regla por prompt.submit (canal degradado); un shim `exo` le sirve la regla de K_REGLA_FILE
# $O/inyectado.txt = lo que el brazo inyecta al arrancar (a2: solo en corridas reales). $O/kb-sucia.txt = estado de
# $P/kb que dejó el agente; la KB se restaura al final (I2 de K).
# cmdline.txt (deny, PATH, append) se escribe siempre. K_ENSAYO=1: sale tras generarlo, sin lanzar claude.
set -uo pipefail
tarea=$1; brazo=$2; rep=$3
# arp<N> (techo-reglas-3): mismo brazo que arp con su propio directorio de corrida; el N solo distingue el bloque de reglas.
modo=$brazo; [[ $brazo =~ ^arp[0-9]+$ ]] && modo=arp
H="$(cd "$(dirname "$0")" && pwd)"
K_ROOT="${K_ROOT:-$HOME/.cache/exo-ablacion-k}"; P="$K_ROOT/prep"
PLUG="$(cd "$H/../../../plugins/exo/scripts" && pwd)"
MODELO="${K_MODELO:-claude-sonnet-5-5}"
[[ "${K_CORRIDAS:-corridas}" =~ ^corridas[A-Za-z0-9_-]*$ ]] || { echo "K_CORRIDAS inválida: ${K_CORRIDAS}" >&2; exit 2; }
if [ "$brazo" = ar ] && [ ! -s "${K_REGLA_FILE:-}" ]; then
  echo "ar requiere K_REGLA_FILE no vacío" >&2; exit 2
fi
if [ "$brazo" = ars ] && [ ! -s "${K_REGLA_FILE:-}" ]; then
  echo "ars requiere K_REGLA_FILE no vacío" >&2; exit 2
fi
if [ "$modo" = arp ] && { [ ! -s "${K_REGLA_FILE:-}" ] || [ ! -s "${K_FRAMING_FILE:-}" ]; }; then
  echo "arp requiere K_REGLA_FILE y K_FRAMING_FILE no vacíos" >&2; exit 2
fi
if [ "$modo" = arp ] && [ "$(grep -o '{{REGLA}}' "$K_FRAMING_FILE" | wc -l)" != 1 ]; then
  echo "arp: K_FRAMING_FILE debe contener {{REGLA}} exactamente una vez" >&2; exit 2
fi
[ -f "$P/manifiesto.txt" ] || { echo "falta $P (preparar.sh)" >&2; exit 2; }
id=$(basename "$tarea"); O="$K_ROOT/${K_CORRIDAS:-corridas}/$id/$brazo-r$rep"; rm -rf "$O"; mkdir -p "$O"
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
case $modo in
  a1) hooks=$(jq -n --arg c "cat $P/a1-inicio.json" '{SessionStart:[{hooks:[{type:"command",command:$c}]}]}') ;;
  a2) hooks=$(jq -n --arg c "$PLUG/exo-recall.sh" '{SessionStart:[{hooks:[{type:"command",command:$c}]}]}') ;;
  a2s) hooks=$(jq -n --arg c "$PLUG/exo-recall.sh" --arg s "$PLUG/search-first.sh" \
        '{SessionStart:[{hooks:[{type:"command",command:$c}]}],PreToolUse:[{matcher:"^(Agent|Task|Edit|Write|NotebookEdit)$",hooks:[{type:"command",command:$s}]}]}') ;;
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
  ars) # Sin scripts/ ni hooks de comando: el plugin completo metería recall de KB y contaminaría la medición.
      mkdir -p "$O/plugin-ars/hooks" "$O/plugin-ars/.claude-plugin" "$O/stub-ars" || exit 2
      cp "$H/../../../plugins/exo/hooks/register.ts" "$O/plugin-ars/hooks/register.ts" || exit 2
      echo '{"modules":["./register.ts"]}' > "$O/plugin-ars/hooks/hooks.json"
      echo '{"name":"exo-ars","description":"Mod de reglas de proyecto, solo para el eval ars","version":"0.0.0"}' > "$O/plugin-ars/.claude-plugin/plugin.json"
      jq -Rs '{schema_version:2,command:"rules",data:{status:"ok",repo:"x",note:"x.md",rules:[rtrimstr("\n")],ignored_lines:[]}}' \
        "$K_REGLA_FILE" > "$O/stub-ars/rules.json" || exit 2
      printf '#!/usr/bin/env bash\nif [ "${1:-}" = rules ]; then cat "%s/rules.json"; exit 0; fi\nexec "%s/stub/exo" "$@"\n' \
        "$O/stub-ars" "$P" > "$O/stub-ars/exo"
      chmod +x "$O/stub-ars/exo" ;;
  n) K_MEMORIA_DIR="${K_MEMORIA_DIR:-$P/memoria}"
      [ -f "$K_MEMORIA_DIR/MEMORY.md" ] || { echo "n requiere $K_MEMORIA_DIR/MEMORY.md" >&2; exit 2; }
      cp -a "$K_MEMORIA_DIR" "$O/memoria" || exit 2 ;;
  *) echo "brazo desconocido: $brazo" >&2; exit 2 ;;
esac
if [ "$brazo" = n ]; then
  jq -n --argjson h "$hooks" --arg d "$O/memoria" '{autoMemoryEnabled:true, autoMemoryDirectory:$d, hooks:$h}' > "$O/settings.json"
else
  jq -n --argjson h "$hooks" '{autoMemoryEnabled:false, hooks:$h}' > "$O/settings.json"
fi
case $brazo in
  ar) cp "$O/regla-ctx.json" "$O/inyectado.txt" ;;
  a1) cp "$P/a1-inicio.json" "$O/inyectado.txt" 2>/dev/null || : > "$O/inyectado.txt" ;;
  n) cp "$O/memoria/MEMORY.md" "$O/inyectado.txt" ;;
  *) : > "$O/inyectado.txt" ;;
esac

deny=("Read(//home/paul/Documentos/proyectos/wisdom-paul/**)" "Read(//home/paul/.exo/**)"
      "Grep(//home/paul/Documentos/proyectos/wisdom-paul/**)" "Grep(//home/paul/.exo/**)")
{ [ "$brazo" = a0 ] || [ "$brazo" = n ] || [ "$brazo" = ar ] || [ "$modo" = arp ] || [ "$brazo" = ars ]; } && deny+=("Read(/$P/kb/**)" "Grep(/$P/kb/**)")
ruta="$PATH"; case $modo in a0|a1|n|ar|arp) ruta="$P/stub:$PATH" ;; ars) ruta="$O/stub-ars:$P/stub:$PATH" ;; esac

# S2: el agente usa el venv del repo (solo lectura) y el código de su workdir.
extra_env=(); plugin_args=(); persist=(--no-session-persistence)
# a2s persiste la sesión: es el único rastro del aviso que vio el modelo (y, antes de #182, sin ella search-first callaba).
[ "$brazo" = a2s ] && { extra_env+=(SEARCH_FIRST_SENTINEL_DIR="$O"); persist=(); }
[ "$brazo" = ars ] && { extra_env+=(EXO_RULES_FORZAR_SUBMIT=1); plugin_args=(--plugin-dir "$O/plugin-ars"); }
if [ -f "$tarea/meta.json" ]; then
  s2repo=$(jq -r .repo "$tarea/meta.json"); venv="$K_ROOT/s2/venv-$s2repo"
  ruta="$venv/bin:$ruta"
  extra_env+=(VIRTUAL_ENV="$venv" PYTHONPATH="$O/work/src:$O/work")
  [ "$s2repo" = django-oscar ] && extra_env+=(DATABASE_ENGINE=django.db.backends.sqlite3 DATABASE_NAME=:memory:)
fi
# cmdline.txt también en corridas reales: es el único rastro del canal (append) que usó la corrida.
{ printf 'PATH=%s\n' "$ruta"; printf 'deny=%s\n' "${deny[@]}"; printf 'append=%s\n' "$append"
  # plugin-dir y env se derivan de los arrays que de verdad se pasan a claude
  [ ${#plugin_args[@]} -gt 0 ] && printf 'plugin-dir=%s\n' "${plugin_args[1]}"
  [ ${#extra_env[@]} -gt 0 ] && printf 'env=%s\n' "${extra_env[@]}"; } > "$O/cmdline.txt"
if [ "${K_ENSAYO:-0}" = 1 ]; then
  echo "correr.sh: modo ensayo, no se lanza claude" >&2; exit 0
fi
kb_base=$(awk '$1=="kb_commit"{print $2}' "$P/manifiesto.txt")
{ [ -e "$P/kb/.git" ] && [ -n "$kb_base" ] && git -C "$P/kb" cat-file -e "$kb_base^{commit}" 2>/dev/null; } \
  || { echo "no se puede restaurar $P/kb (sin .git, o kb_commit ausente o inexistente): no se lanza claude" >&2; exit 3; }
[ -z "$(git -C "$P/kb" status --porcelain)" ] && [ "$(git -C "$P/kb" rev-parse HEAD)" = "$(git -C "$P/kb" rev-parse "$kb_base^{commit}")" ] \
  || { echo "$P/kb no está limpia o su HEAD no es kb_commit (restauración previa fallida?): no se lanza claude" >&2; exit 3; }
prompt=$(jq -r .prompt "$t_json")
{ [ "$brazo" = a2 ] || [ "$brazo" = a2s ]; } && PATH="$ruta" EXO_CONFIG="$P/config.toml" EXO_KB="$P/kb" EXO_DB="$P/index.db" EXO_INDEX="$P/index.db" \
  EXO_BIN="$(command -v exo)" REFLEX_LOG_FILE="$O/reflex-captura.jsonl" "$PLUG/exo-recall.sh" < /dev/null > "$O/inyectado.txt" 2>/dev/null
cd "$O/work" || exit 3; t0=$(date +%s)
env "${extra_env[@]}" PATH="$ruta" EXO_CONFIG="$P/config.toml" EXO_KB="$P/kb" EXO_DB="$P/index.db" EXO_INDEX="$P/index.db" \
EXO_BIN="$(command -v exo)" REFLEX_LOG_FILE="$O/reflex.jsonl" \
DISABLE_AUTOUPDATER=1 timeout 1800 claude -p --model "$MODELO" --setting-sources "" --strict-mcp-config \
  --settings "$O/settings.json" "${plugin_args[@]}" --append-system-prompt-file "$append" \
  --disallowedTools "${deny[@]}" \
  --permission-mode bypassPermissions --max-turns 40 --max-budget-usd 10 "${persist[@]}" \
  --output-format stream-json --verbose "$prompt" < /dev/null > "$O/transcript.jsonl" 2> "$O/err.log"
rc=$?; t1=$(date +%s)
claude --version > "$O/claude-version.txt" 2>/dev/null
# a2s: la sesión persistida (único rastro del aviso de search-first que vio el modelo) se mueve a $O/sesion.jsonl.
if [ "$brazo" = a2s ]; then
  sdir="$HOME/.claude/projects/$(printf '%s' "$O/work" | sed 's#[^A-Za-z0-9]#-#g')"
  cat "$sdir"/*.jsonl > "$O/sesion.jsonl" 2>/dev/null && rm -f "$sdir"/*.jsonl && rmdir "$sdir" 2>/dev/null
fi
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
python3 "$H/fugas.py" "$O" "$modo" > "$O/fugas.json" 2>/dev/null
{ git -C "$P/kb" status --porcelain; git -C "$P/kb" log --format='C %h %s' "$kb_base..HEAD"; } > "$O/kb-sucia.txt" 2>/dev/null
restaura=0
git -C "$P/kb" reset -q --hard "$kb_base" || restaura=1
git -C "$P/kb" clean -fdq || restaura=1
[ $restaura = 0 ] || echo "restauracion: fallo" >> "$O/kb-sucia.txt"
git -C "$O/work" diff "$(cat "$O/inicio.txt" 2>/dev/null)" > "$O/diff.patch" 2>/dev/null
git -C "$O/work" status --porcelain > "$O/status.txt" 2>/dev/null
[ "${K_CONSERVAR_WORK:-0}" = 1 ] || rm -rf "$O/work"
cat "$O/meta.json"
[ $restaura = 0 ] || exit 3
