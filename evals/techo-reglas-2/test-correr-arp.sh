#!/usr/bin/env bash
# Tests bash (sin claude) del brazo arp en correr.sh y fugas.py.
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
CORRER="$HERE/../ablacion-k/harness/correr.sh"
FUG="$HERE/../ablacion-k/harness/fugas.py"
FRAMING="$HERE/framing.txt"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
fail=0
ok(){ echo "ok   $1"; }; ko(){ echo "FAIL $1"; fail=1; }

export K_ROOT="$T/k"; P="$K_ROOT/prep"
mkdir -p "$P/stub" "$K_ROOT/fuentes/tt"; : > "$P/manifiesto.txt"
printf 'base de claude-md\n\n' > "$P/claude-md.md"
git -C "$K_ROOT/fuentes/tt" init -q
tarea="$T/tareas/tt"; mkdir -p "$tarea"; echo '{"prompt":"x"}' > "$tarea/tarea.json"
export K_ENSAYO=1
: > "$T/vacio.txt"; printf 'r' > "$T/r.txt"

# arp_sin_regla_o_framing_falla_ruidoso
msg_ok="arp requiere K_REGLA_FILE y K_FRAMING_FILE no vacíos"
for caso in sin-regla sin-framing regla-vacia framing-vacio; do
  unset K_REGLA_FILE K_FRAMING_FILE
  case $caso in
    sin-regla) export K_FRAMING_FILE="$FRAMING" ;;
    sin-framing) export K_REGLA_FILE="$T/r.txt" ;;
    regla-vacia) export K_REGLA_FILE="$T/vacio.txt" K_FRAMING_FILE="$FRAMING" ;;
    framing-vacio) export K_REGLA_FILE="$T/r.txt" K_FRAMING_FILE="$T/vacio.txt" ;;
  esac
  msg=$(bash "$CORRER" "$tarea" arp 1 2>&1); rc=$?
  if [ $rc = 2 ] && [ "$msg" = "$msg_ok" ] && [ ! -d "$K_ROOT/corridas" ]; then ok "arp_sin_regla_o_framing_falla_ruidoso ($caso)"
  else ko "arp_sin_regla_o_framing_falla_ruidoso ($caso) rc=$rc msg=$msg"; fi
done

# arp_sysprompt_es_exacto
printf 'Regla "uno" con \\ barra\ny salto $VAR `cmd` {{REGLA}}\n' > "$T/regla.txt"
export K_REGLA_FILE="$T/regla.txt" K_FRAMING_FILE="$FRAMING"
bash "$CORRER" "$tarea" arp 1 >/dev/null 2>&1; rc=$?
O="$K_ROOT/corridas/tt/arp-r1"
# esperado construido sin python, con centinela x para conservar los saltos finales
fr="$(cat "$FRAMING"; echo x)"; fr=${fr%x}; rg="$(cat "$T/regla.txt"; echo x)"; rg=${rg%x}
{ cat "$P/claude-md.md"; printf '\n'; printf '%s' "${fr/'{{REGLA}}'/$rg}"; } > "$T/esperado.md"
if [ $rc = 0 ] && cmp -s "$O/sysprompt.md" "$T/esperado.md" && grep -qF 'Regla "uno" con \ barra' "$O/sysprompt.md" \
   && [ "$(grep -c '{{REGLA}}' "$O/sysprompt.md")" = 1 ]; then
  ok arp_sysprompt_es_exacto; else ko "arp_sysprompt_es_exacto rc=$rc"; fi
# el {{REGLA}} que viene dentro de la regla debe sobrevivir (1 aparición, la de la regla); el del framing no
grep -q '^- Regla "uno"' "$O/sysprompt.md" || ko "arp_sysprompt_es_exacto: marcador sin sustituir"

# arp_hereda_restricciones_de_a0
bash "$CORRER" "$tarea" a0 1 >/dev/null 2>&1; A0="$K_ROOT/corridas/tt/a0-r1"
if grep -qxF "deny=Read(/$P/kb/**)" "$O/cmdline.txt" && grep -qxF "deny=Grep(/$P/kb/**)" "$O/cmdline.txt" \
   && grep -q "^PATH=$P/stub:" "$O/cmdline.txt" && cmp -s "$O/settings.json" "$A0/settings.json" \
   && diff <(grep -v '^append=' "$O/cmdline.txt") <(grep -v '^append=' "$A0/cmdline.txt") >/dev/null; then
  ok arp_hereda_restricciones_de_a0; else ko arp_hereda_restricciones_de_a0; fi

# arp_cmdline_usa_sysprompt
if grep -qxF "append=$O/sysprompt.md" "$O/cmdline.txt" && grep -qxF "append=$P/claude-md.md" "$A0/cmdline.txt"; then
  ok arp_cmdline_usa_sysprompt; else ko arp_cmdline_usa_sysprompt; fi

# fugas_arp_sin_hooks
F="$T/fug"; mkdir -p "$F"
hook='{"type":"system","subtype":"hook_started","hook_event":"SessionStart"}'
tool="{\"type\":\"assistant\",\"message\":{\"content\":[{\"type\":\"tool_use\",\"id\":\"t1\",\"name\":\"Read\",\"input\":{\"file_path\":\"$K_ROOT/prep/kb/x.md\"}}]}}"
ok_msg='{"type":"assistant","message":{"content":[{"type":"text","text":"hola"}]}}'
echo "$ok_msg" > "$F/transcript.jsonl"
out=$(python3 "$FUG" "$F" arp 2>&1); rc=$?
[ $rc = 0 ] && [ "$(jq -r .fuga <<<"$out")" = false ] && ok fugas_arp_limpio || ko "fugas_arp_limpio rc=$rc out=$out"
printf '%s\n%s\n' "$hook" "$ok_msg" > "$F/transcript.jsonl"
out=$(python3 "$FUG" "$F" arp 2>&1); rc=$?
[ $rc = 1 ] && [ "$(jq -r .fuga <<<"$out")" = true ] && ok fugas_arp_hook_es_fuga || ko "fugas_arp_hook_es_fuga rc=$rc out=$out"
printf '%s\n' "$tool" > "$F/transcript.jsonl"
out=$(python3 "$FUG" "$F" arp 2>&1); rc=$?
[ $rc = 1 ] && [ "$(jq -r .fuga <<<"$out")" = true ] && ok fugas_arp_snapshot_es_fuga || ko "fugas_arp_snapshot_es_fuga rc=$rc out=$out"
exit $fail
