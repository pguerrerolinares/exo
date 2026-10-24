#!/usr/bin/env bash
# Tests bash (sin claude real: un stub en PATH) de escritura.sh, el piloto de escritura de la campaña M.
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ESC="${ESC:-$HERE/escritura.sh}"
REPO="$(cd "$HERE/../.." && pwd)"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
fail=0
ok(){ echo "ok   $1"; }; ko(){ echo "FAIL $1"; fail=1; }

export K_ROOT="$T/k"; P="$K_ROOT/prep"
mkdir -p "$P/stub" "$T/bin" "$T/home"; export HOME="$T/home"
printf 'base\n' > "$P/claude-md.md"
mkdir -p "$P/kb"; git -C "$P/kb" init -q; echo base > "$P/kb/k.md"
git -C "$P/kb" add k.md; git -C "$P/kb" -c user.name=t -c user.email=t@t commit -qm base
echo "kb_commit $(git -C "$P/kb" rev-parse HEAD)" > "$P/manifiesto.txt"
mkdir -p "$P/memoria"; printf -- '- [uno](uno.md) — primera\n' > "$P/memoria/MEMORY.md"; echo uno > "$P/memoria/uno.md"
: > "$P/index.db"; : > "$P/config.toml"
cat > "$T/sesion.json" <<'J'
{"id":"s1","turnos":["primer turno","segundo turno"],"decision":"usar X"}
J
# claude stub: registra los argumentos; escribe en su cwd y, si hay autoMemoryDirectory, en esa carpeta.
cat > "$T/bin/claude" <<'S'
#!/usr/bin/env bash
printf '%s\0' "$@" | tr '\0' '\n' | sed 's/^/ARG /' >> "$STUB_LOG"; echo "--" >> "$STUB_LOG"
echo escrito-por-stub > ./escrito-stub.md
while [ $# -gt 0 ]; do
  [ "$1" = --settings ] && { d=$(jq -r '.autoMemoryDirectory // empty' "$2"); [ -n "$d" ] && echo memoria-nueva > "$d/nueva.md"; }
  shift
done
echo '{"type":"result"}'
S
chmod +x "$T/bin/claude"; export PATH="$T/bin:$PATH" STUB_LOG="$T/stub.log"
CIERRE='cierra la sesión y persiste lo decidido'

# cierre_literal: el último prompt de cmdline.txt es el literal en los dos brazos; x usa plugins/exo del repo
c=1
for b in x n; do
  K_ENSAYO=1 bash "$ESC" "$T/sesion.json" $b "$T/ens-$b" >/dev/null 2>&1 || c=0
  [ "$(tail -1 "$T/ens-$b/entrada.jsonl" | jq -r 'select(.type=="user") | .message.content')" = "$CIERRE" ] || c=0
  [ "$(wc -l < "$T/ens-$b/entrada.jsonl")" = 3 ] || c=0
done
grep -qxF "plugin-dir=$REPO/plugins/exo" "$T/ens-x/cmdline.txt" || c=0
grep -q '^plugin-dir=' "$T/ens-n/cmdline.txt" && c=0
[ $c = 1 ] && ok cierre_literal || ko cierre_literal

# copias_aisladas: x escribe en la copia de la KB, n en la de memoria; los originales no cambian
sha_mem=$(cd "$P/memoria" && find . -type f | sort | xargs sha256sum | sha256sum)
c=1
: > "$STUB_LOG"
bash "$ESC" "$T/sesion.json" x "$T/out-x" >/dev/null 2>&1 || c=0
bash "$ESC" "$T/sesion.json" n "$T/out-n" >/dev/null 2>&1 || c=0
grep -q 'escrito-stub.md' "$T/out-x/escrito.diff" || c=0
[ -z "$(git -C "$P/kb" status --porcelain)" ] || c=0
grep -q 'nueva.md' "$T/out-n/escrito.diff" || c=0
[ "$(cd "$P/memoria" && find . -type f | sort | xargs sha256sum | sha256sum)" = "$sha_mem" ] || c=0
[ ! -e "$T/out-n/memoria/.git" ] && [ ! -L "$T/out-n/memoria" ] || c=0
# un solo proceso por sesión (stream-json por stdin), sin resume ni session-id
[ "$(grep -c '^--$' "$STUB_LOG")" = 2 ] && [ "$(grep -c '^ARG --no-session-persistence$' "$STUB_LOG")" = 2 ] || c=0
[ "$(grep -c '^ARG --input-format$' "$STUB_LOG")" = 2 ] || c=0
grep -qE '^ARG --(resume|session-id)$' "$STUB_LOG" && c=0
[ $c = 1 ] && ok copias_aisladas || ko copias_aisladas

# sin_credenciales: nada de tokens bajo la salida (acaba en el tarball de registro) ni CLAUDE_CONFIG_DIR propio
mkdir -p "$HOME/.claude"; echo '{"claudeAiOauth":{"accessToken":"SECRETO"}}' > "$HOME/.claude/.credentials.json"
c=1
for b in x n; do
  bash "$ESC" "$T/sesion.json" $b "$T/cred-$b" >/dev/null 2>&1 || c=0
  [ -z "$(find "$T/cred-$b" -name .credentials.json)" ] || c=0
  grep -rq accessToken "$T/cred-$b" && c=0
  grep -q CLAUDE_CONFIG_DIR "$T/cred-$b/cmdline.txt" && c=0
done
[ $c = 1 ] && ok sin_credenciales || ko sin_credenciales

# brazo_desconocido
bash "$ESC" "$T/sesion.json" a0 "$T/out-a0" >/dev/null 2>&1; [ $? = 2 ] && ok brazo_desconocido || ko brazo_desconocido

exit $fail
