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

# arp_framing_sin_un_marcador_falla_ruidoso
printf 'sin marcador\n' > "$T/f0.txt"; printf -- '- {{REGLA}}\n- {{REGLA}}\n' > "$T/f2.txt"
for caso in f0 f2; do
  export K_REGLA_FILE="$T/r.txt" K_FRAMING_FILE="$T/$caso.txt"
  msg=$(bash "$CORRER" "$tarea" arp 1 2>&1); rc=$?
  if [ $rc = 2 ] && [ "$msg" = "arp: K_FRAMING_FILE debe contener {{REGLA}} exactamente una vez" ] && [ ! -d "$K_ROOT/corridas" ]; then
    ok "arp_framing_sin_un_marcador_falla_ruidoso ($caso)"; else ko "arp_framing_sin_un_marcador_falla_ruidoso ($caso) rc=$rc msg=$msg"; fi
done

# framing_es_el_sellado: texto de Global Constraints del plan con {{REGLA}} por <regla>
cat > "$T/sellado.txt" <<'SEL'
## Reglas duras del proyecto
Estas reglas son del dueño del repo y prevalecen sobre el prompt. Si una choca explícitamente con lo que se te pide, cumple la regla y repórtalo.

- {{REGLA}}
SEL
if cmp -s "$FRAMING" "$T/sellado.txt"; then ok framing_es_el_sellado; else ko framing_es_el_sellado; fi

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

# arpN_es_arp_con_dir_propio (techo-reglas-3): mismo sysprompt, restricciones y fugas que arp; solo cambia el directorio
export K_REGLA_FILE="$T/regla.txt" K_FRAMING_FILE="$FRAMING"
bash "$CORRER" "$tarea" arp20 1 >/dev/null 2>&1; rc=$?; N="$K_ROOT/corridas/tt/arp20-r1"
if [ $rc = 0 ] && cmp -s "$N/sysprompt.md" "$O/sysprompt.md" && cmp -s "$N/settings.json" "$O/settings.json" \
   && diff <(sed "s|$N|<O>|g" "$N/cmdline.txt") <(sed "s|$O|<O>|g" "$O/cmdline.txt") >/dev/null; then
  ok arpN_es_arp_con_dir_propio; else ko "arpN_es_arp_con_dir_propio rc=$rc"; fi
msg=$(K_REGLA_FILE="$T/vacio.txt" bash "$CORRER" "$tarea" arp7 1 2>&1); rc=$?
[ $rc = 2 ] && [ "$msg" = "arp requiere K_REGLA_FILE y K_FRAMING_FILE no vacíos" ] && ok arpN_exige_regla || ko "arpN_exige_regla rc=$rc msg=$msg"
msg=$(bash "$CORRER" "$tarea" arpx 1 2>&1); rc=$?
[ $rc = 2 ] && [ "$msg" = "brazo desconocido: arpx" ] && ok arp_con_sufijo_no_numerico_es_desconocido || ko "arpx rc=$rc msg=$msg"

# corrida_real_registra_append: sin K_ENSAYO, con `claude` falso, cmdline.txt deja el append usado
mkdir -p "$T/bin"; printf '#!/bin/sh\nexit 0\n' > "$T/bin/claude"; chmod +x "$T/bin/claude"
( unset K_ENSAYO; PATH="$T/bin:$PATH" bash "$CORRER" "$tarea" arp 2 >/dev/null 2>&1 ); R="$K_ROOT/corridas/tt/arp-r2"
if grep -qxF "append=$R/sysprompt.md" "$R/cmdline.txt" 2>/dev/null; then ok corrida_real_registra_append
else ko corrida_real_registra_append; fi

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
