#!/usr/bin/env bash
# Tests de scripts/check-prosa-cli.sh con un `exo` falso y fixtures en tmpdir.
set -uo pipefail
RAIZ="$(git rev-parse --show-toplevel)" || exit 1
if [ -z "$RAIZ" ]; then echo "test-check-prosa-cli: sin toplevel" >&2; exit 1; fi
cd "$RAIZ" || exit 1
CHECK="$RAIZ/scripts/check-prosa-cli.sh"

T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
PASS=0; FAIL=0
ok()  { echo "PASS: $1"; PASS=$((PASS+1)); }
bad() { echo "FAIL: $1"; FAIL=$((FAIL+1)); }

# Reproduce la forma de clap: `Commands:` y `Options:` en `--help`.
cat > "$T/exo" <<'STUB'
#!/usr/bin/env bash
args="$*"
case "$args" in
  "--help") printf 'Usage: exo <COMMAND>\n\nCommands:\n  search  Busca\n  write   Escribe\n  extra   Algo\n  help    Print this message\n\nOptions:\n  -h, --help  Print help\n' ;;
  "search --help") printf 'Usage: exo search\n\nOptions:\n      --limit <N>  Tope\n      --json       Salida JSON\n  -h, --help       Print help\n' ;;
  "write --help") printf 'Usage: exo write <COMMAND>\n\nCommands:\n  new     Crea\n  help    Print\n\nOptions:\n  -h, --help  Print help\n' ;;
  "write new --help") printf 'Usage: exo write new\n\nOptions:\n      --tier <T>  Tier\n  -h, --help      Print help\n' ;;
  "extra --help") printf 'Usage: exo extra\n\nOptions:\n  -h, --help  Print help\n' ;;
esac
STUB
chmod +x "$T/exo"
export EXO_BIN="$T/exo"

corre() { # $1 = contenido del .md; deja rc y out
  local P="$T/p"; rm -rf "$P"; mkdir -p "$P/skills/x"
  printf '%s\n' "$1" > "$P/skills/x/SKILL.md"
  out="$(bash "$CHECK" "$P" 2>&1)"; rc=$?
}

# shellcheck disable=SC2016 # los backticks son texto literal del fixture
corre 'Usa `exo search --limit 5 "q"` y `exo write new --tier core`, y `exo extra`.'
if [ $rc -eq 0 ] && [ -z "$out" ]; then ok prosa_valida; else bad "prosa_valida (rc=$rc out=$out)"; fi

# shellcheck disable=SC2016
corre 'Usa `exo buscar "q"` y `exo extra`, `exo search`, `exo write new`.'
if [ $rc -eq 1 ] && printf '%s' "$out" | grep -q 'SKILL.md:1: `exo buscar` no existe'; then ok subcomando_inexistente; else bad "subcomando_inexistente (rc=$rc out=$out)"; fi

# shellcheck disable=SC2016
corre 'Usa `exo search --limite 5` y `exo extra`, `exo write new`.'
if [ $rc -eq 1 ] && printf '%s' "$out" | grep -q 'exo search. no tiene --limite'; then ok flag_inexistente; else bad "flag_inexistente (rc=$rc out=$out)"; fi

# shellcheck disable=SC2016
corre 'Usa `exo write borrar` y `exo extra`, `exo search`.'
if [ $rc -eq 1 ] && printf '%s' "$out" | grep -q 'exo write borrar. no existe'; then ok subsubcomando_inexistente; else bad "subsubcomando_inexistente (rc=$rc out=$out)"; fi

corre 'Bloque:

```bash
exo search \
  --nope 3
exo write new --tier core
exo extra
```'
if [ $rc -eq 1 ] && printf '%s' "$out" | grep -q 'SKILL.md:4: .exo search. no tiene --nope'; then ok flag_en_bloque_con_continuacion; else bad "flag_en_bloque_con_continuacion (rc=$rc out=$out)"; fi

# shellcheck disable=SC2016
corre 'Solo `exo search` y `exo write new`.'
if [ $rc -eq 1 ] && printf '%s' "$out" | grep -q 'exo extra existe en --help pero ningún .md'; then ok subcomando_sin_cita; else bad "subcomando_sin_cita (rc=$rc out=$out)"; fi

# shellcheck disable=SC2016
corre 'El motor `exo` y `exo:plan`, "exo es" no son invocaciones; `exo search`, `exo write new`, `exo extra`.'
if [ $rc -eq 0 ]; then ok prosa_que_no_es_invocacion; else bad "prosa_que_no_es_invocacion (rc=$rc out=$out)"; fi

# repo_real, solo si hay binario compilado
unset EXO_BIN
if [ -x engine/target/debug/exo ]; then
  out="$(bash "$CHECK" 2>&1)"; rc=$?
  if [ $rc -eq 0 ]; then ok repo_real; else bad "repo_real (rc=$rc out=$out)"; fi
fi

echo "$PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
