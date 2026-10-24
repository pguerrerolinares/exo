#!/usr/bin/env bash
# escritura.sh <sesion.json> <brazo x|n> <salida> — piloto de escritura (campaña M, descriptivo).
# sesion.json = {id, turnos:[prompt...], decision}. Corre los turnos encadenados con --resume y luego el cierre literal.
#   x  KB: copia de $K_ROOT/prep/kb (cwd de claude) + plugin exo del repo (/document vive ahí). Diff = git de la copia.
#   n  memoria nativa: copia de $K_ROOT/prep/memoria con autoMemoryDirectory apuntando a ella. Diff = diff -ruN vs original.
# Deja en <salida>: escrito.diff, cmdline.txt, transcript-<i>.jsonl, err-<i>.log. K_ENSAYO=1: sale tras cmdline.txt.
# Aislamiento como correr.sh (setting-sources vacío, strict-mcp, deny de KB de producción y ~/.exo, 10 USD, 40 turnos),
# salvo --no-session-persistence: --resume necesita la sesión, que vive en CLAUDE_CONFIG_DIR=<salida>/config.
set -uo pipefail
ses=$1; brazo=$2; O=$3
H="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$H/../.." && pwd)"
K_ROOT="${K_ROOT:-$HOME/.cache/exo-campana-m}"; P="$K_ROOT/prep"
MODELO="${K_MODELO:-claude-sonnet-5-5}"
CIERRE='cierra la sesión y persiste lo decidido'
case $brazo in x|n) ;; *) echo "brazo desconocido: $brazo" >&2; exit 2 ;; esac
[ -f "$ses" ] || { echo "falta $ses" >&2; exit 2; }
[ -f "$P/manifiesto.txt" ] || { echo "falta $P (preparar-m.sh)" >&2; exit 2; }
mapfile -d '' prompts < <(jq -j '.turnos[] + "\u0000"' "$ses") || exit 2
[ ${#prompts[@]} -gt 0 ] || { echo "sesion sin turnos" >&2; exit 2; }
prompts+=("$CIERRE")
rm -rf "$O"; mkdir -p "$O/config" "$O/work" || exit 2

args=(); ruta="$PATH"; extra_env=()
deny=("Read(//home/paul/Documentos/proyectos/wisdom-paul/**)" "Read(//home/paul/.exo/**)"
      "Grep(//home/paul/Documentos/proyectos/wisdom-paul/**)" "Grep(//home/paul/.exo/**)")
if [ "$brazo" = x ]; then
  cp -a "$P/kb" "$O/kb" && cp -a "$P/index.db" "$O/index.db" || exit 2
  base=$(git -C "$O/kb" rev-parse HEAD) || exit 2
  cwd="$O/kb"
  jq -n '{autoMemoryEnabled:false, hooks:{}}' > "$O/settings.json"
  args+=(--plugin-dir "$REPO/plugins/exo")
  extra_env+=(EXO_CONFIG="$P/config.toml" EXO_KB="$O/kb" EXO_DB="$O/index.db" EXO_INDEX="$O/index.db")
else
  [ -f "$P/memoria/MEMORY.md" ] || { echo "n requiere $P/memoria/MEMORY.md" >&2; exit 2; }
  cp -a "$P/memoria" "$O/memoria" || exit 2
  cwd="$O/work"
  jq -n --arg d "$O/memoria" '{autoMemoryEnabled:true, autoMemoryDirectory:$d, hooks:{}}' > "$O/settings.json"
  deny+=("Read(/$P/kb/**)" "Grep(/$P/kb/**)"); ruta="$P/stub:$PATH"
fi

{ printf 'PATH=%s\n' "$ruta"; printf 'deny=%s\n' "${deny[@]}"; printf 'append=%s\n' "$P/claude-md.md"
  [ ${#args[@]} -gt 0 ] && printf 'plugin-dir=%s\n' "${args[1]}"
  printf 'prompt=%s\n' "${prompts[@]}"; } > "$O/cmdline.txt"
if [ "${K_ENSAYO:-0}" = 1 ]; then echo "escritura.sh: modo ensayo, no se lanza claude" >&2; exit 0; fi

# Config propia para que la sesión no toque ~/.claude; sin credenciales copiadas no hay login.
[ -f "$HOME/.claude/.credentials.json" ] && install -m 600 "$HOME/.claude/.credentials.json" "$O/config/.credentials.json"
sid=$(cat /proc/sys/kernel/random/uuid)
cd "$cwd" || exit 3
rc=0
for i in "${!prompts[@]}"; do
  if [ "$i" = 0 ]; then sesion=(--session-id "$sid"); else sesion=(--resume "$sid"); fi
  env "${extra_env[@]}" PATH="$ruta" CLAUDE_CONFIG_DIR="$O/config" DISABLE_AUTOUPDATER=1 \
    timeout 1800 claude -p --model "$MODELO" --setting-sources "" --strict-mcp-config \
    --settings "$O/settings.json" "${args[@]}" --append-system-prompt-file "$P/claude-md.md" \
    --disallowedTools "${deny[@]}" \
    --permission-mode bypassPermissions --max-turns 40 --max-budget-usd 10 "${sesion[@]}" \
    --output-format stream-json --verbose "${prompts[$i]}" < /dev/null > "$O/transcript-$i.jsonl" 2> "$O/err-$i.log" \
    || { rc=$?; echo "turno $i: rc=$rc" >&2; break; }
done
if [ "$brazo" = x ]; then
  git -C "$O/kb" add -A && git -C "$O/kb" diff --cached "$base" > "$O/escrito.diff"
else
  diff -ruN "$P/memoria" "$O/memoria" > "$O/escrito.diff"
fi
exit $rc
