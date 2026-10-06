#!/usr/bin/env bash
# Sonda: ¿llega el sysprompt.md del brazo arp al system prompt en headless con los flags de correr.sh?
# sonda-sysprompt.sh [codeword]. exit 0 e imprime "sonda: OK <codeword>" ⇔ la corrida con sysprompt.md
# devuelve el codeword y la de control (claude-md.md solo) no. Lanza 2 claude reales (~0,0x USD).
# Sin herramientas (--tools "") y sin ningún fichero con el codeword en el cwd: solo puede salir del prompt.
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
CODEWORD="${1:-ZORZAL-4417}"
K_ROOT="${K_ROOT:-$HOME/.cache/exo-ablacion-k}"; P="$K_ROOT/prep"
MODELO="${K_MODELO:-claude-sonnet-5-5}"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
printf 'El codeword es %s.' "$CODEWORD" > "$T/regla.txt"
# g1-57 está fuera del experimento: su arp-r1 es el único residuo que deja la sonda en $K_ROOT.
K_ENSAYO=1 K_REGLA_FILE="$T/regla.txt" K_FRAMING_FILE="$HERE/framing.txt" \
  bash "$HERE/../ablacion-k/harness/correr.sh" "$K_ROOT/gold/s1/g1-57" arp 1 > /dev/null 2> "$T/gen.log" \
  || { echo "sonda: FALLA, correr.sh arp en ensayo" >&2; cat "$T/gen.log" >&2; exit 1; }
SYS="$K_ROOT/corridas/g1-57/arp-r1/sysprompt.md"
grep -qF "$CODEWORD" "$SYS" || { echo "sonda: FALLA, sysprompt.md sin el codeword" >&2; exit 1; }

mkdir -p "$T/repo"; git -C "$T/repo" init -q
echo '{"autoMemoryEnabled":false,"hooks":{}}' > "$T/settings.json"
q="¿Cuál es el codeword que te dieron? Responde solo el codeword, o NINGUNO si no te dieron ninguno."
corre(){ # $1 = fichero de system prompt; imprime "<resultado>\t<usd>"
  local out
  out=$(cd "$T/repo" && printf '%s' "$q" | DISABLE_AUTOUPDATER=1 timeout 300 claude -p --model "$MODELO" \
    --setting-sources "" --strict-mcp-config --settings "$T/settings.json" --append-system-prompt-file "$1" \
    --tools "" --permission-mode bypassPermissions --max-turns 3 --max-budget-usd 0.10 --no-session-persistence \
    --output-format json 2> "$T/err.$2.log")
  printf '%s\t%s\n' "$(jq -r '.result // empty' <<<"$out" | tr '\n' ' ')" "$(jq -r '.total_cost_usd // "?"' <<<"$out")"
}
IFS=$'\t' read -r r1 u1 < <(corre "$SYS" sys)
IFS=$'\t' read -r r2 u2 < <(corre "$P/claude-md.md" ctl)
echo "sonda: con sysprompt  -> '$r1' (usd=$u1)"
echo "sonda: control        -> '$r2' (usd=$u2)"
if [[ $r1 == *"$CODEWORD"* && $r2 != *"$CODEWORD"* ]]; then echo "sonda: OK $CODEWORD"; exit 0; fi
echo "sonda: FALLA" >&2; head -5 "$T/err.sys.log" "$T/err.ctl.log" >&2; exit 1
