#!/usr/bin/env bash
# Tests bash (sin claude) del brazo ars en correr.sh, fugas.py y el pre-registro de ars.
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
CORRER="$HERE/../ablacion-k/harness/correr.sh"
FUG="$HERE/../ablacion-k/harness/fugas.py"
REPO="$(cd "$HERE/../.." && pwd)"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
fail=0
ok(){ echo "ok   $1"; }; ko(){ echo "FAIL $1"; fail=1; }

export K_ROOT="$T/k"; P="$K_ROOT/prep"
mkdir -p "$P/stub" "$K_ROOT/fuentes/tt"; : > "$P/manifiesto.txt"
printf 'base de claude-md\n\n' > "$P/claude-md.md"
git -C "$K_ROOT/fuentes/tt" init -q
tarea="$T/tareas/tt"; mkdir -p "$tarea"; echo '{"prompt":"x"}' > "$tarea/tarea.json"
export K_ENSAYO=1
: > "$T/vacio.txt"; printf 'Regla "uno" con \ barra y $VAR\n' > "$T/r.txt"

# ars_sin_regla_falla_ruidoso
for caso in sin-regla regla-vacia; do
  unset K_REGLA_FILE; [ $caso = regla-vacia ] && export K_REGLA_FILE="$T/vacio.txt"
  msg=$(bash "$CORRER" "$tarea" ars 1 2>&1); rc=$?
  if [ $rc = 2 ] && [ "$msg" = "ars requiere K_REGLA_FILE no vacío" ] && [ ! -d "$K_ROOT/corridas" ]; then ok "ars_sin_regla_falla_ruidoso ($caso)"
  else ko "ars_sin_regla_falla_ruidoso ($caso) rc=$rc msg=$msg"; fi
done

export K_REGLA_FILE="$T/r.txt"
bash "$CORRER" "$tarea" ars 1 >/dev/null 2>&1; rcs=$?
O="$K_ROOT/corridas/tt/ars-r1"
bash "$CORRER" "$tarea" a0 1 >/dev/null 2>&1; A0="$K_ROOT/corridas/tt/a0-r1"
export K_FRAMING_FILE="$HERE/framing.txt"
bash "$CORRER" "$tarea" arp 1 >/dev/null 2>&1; ARP="$K_ROOT/corridas/tt/arp-r1"
[ $rcs = 0 ] || ko "ars corre en ensayo rc=$rcs"

# ars_cmdline_carga_solo_el_mod
if grep -qxF "plugin-dir=$O/plugin-ars" "$O/cmdline.txt" && [ -f "$O/plugin-ars/hooks/register.ts" ] \
   && [ ! -e "$O/plugin-ars/scripts" ] && [ "$(jq -c '.modules' "$O/plugin-ars/hooks/hooks.json")" = '["./register.ts"]' ] \
   && [ "$(jq '.hooks' "$O/plugin-ars/hooks/hooks.json")" = null ] \
   && [ -n "$(jq -r '.name // empty' "$O/plugin-ars/.claude-plugin/plugin.json")" ] \
   && cmp -s "$O/plugin-ars/hooks/register.ts" "$REPO/plugins/exo/hooks/register.ts" \
   && ! grep -q '^plugin-dir=' "$A0/cmdline.txt" "$ARP/cmdline.txt"; then
  ok ars_cmdline_carga_solo_el_mod; else ko ars_cmdline_carga_solo_el_mod; fi

# ars_env_forzar
if grep -qxF 'env=EXO_RULES_FORZAR_SUBMIT=1' "$O/cmdline.txt" && ! grep -q 'FORZAR' "$A0/cmdline.txt" "$ARP/cmdline.txt"; then
  ok ars_env_forzar; else ko ars_env_forzar; fi

# ars_shim_devuelve_la_regla (la regla sin el salto final; el shim va antes en PATH que el stub de a0)
env_json=$("$O/stub-ars/exo" rules --json 2>&1); rc=$?
esperada='Regla "uno" con \ barra y $VAR'
if [ $rc = 0 ] && [ "$(jq -r '.data.status' <<<"$env_json")" = ok ] && [ "$(jq '.data.rules|length' <<<"$env_json")" = 1 ] \
   && [ "$(jq -r '.data.rules[0]' <<<"$env_json")" = "$esperada" ] && [ "$(jq -r '.schema_version' <<<"$env_json")" = 2 ] \
   && [ "$(jq -r '.command' <<<"$env_json")" = rules ] && grep -q "^PATH=$O/stub-ars:$P/stub:" "$O/cmdline.txt"; then
  ok ars_shim_devuelve_la_regla; else ko "ars_shim_devuelve_la_regla rc=$rc"; fi
# el shim delega el resto de órdenes en el stub de a0
printf '#!/usr/bin/env bash\necho "exo: orden no encontrada"; exit 127\n' > "$P/stub/exo"; chmod +x "$P/stub/exo"
bash "$CORRER" "$tarea" ars 1 >/dev/null 2>&1
out=$("$O/stub-ars/exo" search x 2>&1); rc=$?
[ $rc = 127 ] && [ "$out" = "exo: orden no encontrada" ] && ok ars_shim_delega_en_el_stub || ko "ars_shim_delega_en_el_stub rc=$rc out=$out"

# ars_sin_claude_md_distinto
if grep -qxF "append=$P/claude-md.md" "$O/cmdline.txt" && [ ! -e "$O/sysprompt.md" ] && cmp -s "$O/settings.json" "$A0/settings.json" \
   && diff <(grep -vE '^(append|plugin-dir|env)=|^PATH=' "$O/cmdline.txt") <(grep -vE '^(append|plugin-dir|env)=|^PATH=' "$A0/cmdline.txt") >/dev/null; then
  ok ars_sin_claude_md_distinto; else ko ars_sin_claude_md_distinto; fi

# fugas_ars_conocido: sin hooks de eventos de comando; un hook cableado sería fuga
F="$T/fug"; mkdir -p "$F"
ok_msg='{"type":"assistant","message":{"content":[{"type":"text","text":"hola"}]}}'
hook='{"type":"system","subtype":"hook_started","hook_event":"SessionStart"}'
echo "$ok_msg" > "$F/transcript.jsonl"
out=$(python3 "$FUG" "$F" ars 2>&1); rc=$?
[ $rc = 0 ] && [ "$(jq -r .fuga <<<"$out")" = false ] && ok fugas_ars_limpio || ko "fugas_ars_limpio rc=$rc out=$out"
printf '%s\n%s\n' "$hook" "$ok_msg" > "$F/transcript.jsonl"
out=$(python3 "$FUG" "$F" ars 2>&1); rc=$?
[ $rc = 1 ] && [ "$(jq -r .fuga <<<"$out")" = true ] && ok fugas_ars_hook_es_fuga || ko "fugas_ars_hook_es_fuga rc=$rc out=$out"

# preregistro_ars_valido
PR="$HERE/preregistro-ars.md"; TS="$HERE/tareas-ars.tsv"
SUELO="g0-122 g0-149 g0-33 g1-131 g1-139 g1-140 g1-157 g2-154 g2-170 g2-97"
esperado=$(for i in $SUELO; do printf '%s\tsuelo\tevals/techo-reglas/reglas/%s.txt\tars:2\n' "$i" "$i"; done | sort)
calc=$(python3 -c 'import random,csv,sys; L=[]
for i,g,_,b in csv.reader(open(sys.argv[1]),delimiter="\t"):
    for x in b.split(","):
        a,k=x.split(":"); L+=[(a,i,r) for r in range(1,int(k)+1)]
L.sort(); random.Random(20261006).shuffle(L)
print("\n".join(f"{a} {i} {r}" for a,i,r in L))' "$TS" 2>/dev/null | tr -d '\r')
pub=$(sed -n '/^<!-- ORDEN-BEGIN -->$/,/^<!-- ORDEN-END -->$/p' "$PR" 2>/dev/null | grep -E '^[a-z0-9]+ [a-z0-9-]+ [0-9]+$')
v=$(cat "$HERE/claude-version.txt" 2>/dev/null)
if [ -f "$PR" ] && [ "$(sort "$TS")" = "$esperado" ] && grep -qxF '>=4/10 cumplen = util; <4 = placebo' "$PR" \
   && grep -qE '^- \*\*Brazos y k:\*\*.*ars.*k=2.*20' "$PR" && grep -qF -- "$v" "$PR" && grep -q 'semilla 20261006' "$PR" \
   && grep -qE '^- \*\*Re-intentos:\*\*.*cero' "$PR" && grep -qE '^- \*\*Breaker:\*\*.*no se adjudica' "$PR" \
   && [ "$(grep -c . <<<"$pub")" = 20 ] && [ "$calc" = "$pub" ]; then
  ok preregistro_ars_valido; else ko preregistro_ars_valido; fi
exit $fail
