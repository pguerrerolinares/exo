#!/usr/bin/env bash
# Tests bash (sin claude) del brazo ar y de la invariancia de a0-a3 en correr.sh.
# Uso: bash test-correr-ar.sh   (REF_SETTINGS = settings.json de g1-57/a0-r1 del tarball)
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
CORRER="$HERE/../ablacion-k/harness/correr.sh"
TAR="${K_TARBALL:-$HOME/.cache/exo-ablacion-k-registro.tar.gz}"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
fail=0
ok(){ echo "ok   $1"; }; ko(){ echo "FAIL $1"; fail=1; }

export K_ROOT="$T/k"; P="$K_ROOT/prep"
mkdir -p "$P/stub" "$K_ROOT/fuentes/tt"; : > "$P/manifiesto.txt"
git -C "$K_ROOT/fuentes/tt" init -q; 
tarea="$T/tareas/tt"; mkdir -p "$tarea"; echo '{"prompt":"x"}' > "$tarea/tarea.json"
export K_ENSAYO=1
# claude falso: si correr.sh lo lanzara, deja marca
mkdir -p "$T/bin"; printf '#!/bin/sh\ntouch %s/claude-lanzado\n' "$T" > "$T/bin/claude"; chmod +x "$T/bin/claude"
export PATH="$T/bin:$PATH"

# ar_sin_regla_falla_ruidoso
for caso in sin vacio; do
  unset K_REGLA_FILE; [ $caso = vacio ] && { : > "$T/vacia.txt"; export K_REGLA_FILE="$T/vacia.txt"; }
  msg=$(bash "$CORRER" "$tarea" ar 1 2>&1); rc=$?
  if [ $rc = 2 ] && [[ $msg == *"ar requiere K_REGLA_FILE no vacío"* ]] && [ ! -e "$T/claude-lanzado" ] && [ ! -d "$K_ROOT/corridas" ]; then
    ok "ar_sin_regla_falla_ruidoso ($caso)"; else ko "ar_sin_regla_falla_ruidoso ($caso) rc=$rc msg=$msg"; fi
done

# ar_regla_con_comillas_es_json_valido
printf 'Regla "uno" con \\ barra\ny salto de línea\n\ttab y $VAR `cmd`\n' > "$T/regla.txt"
export K_REGLA_FILE="$T/regla.txt"
bash "$CORRER" "$tarea" ar 1 >/dev/null 2>&1; rc=$?
O="$K_ROOT/corridas/tt/ar-r1"
if [ $rc = 0 ] && jq -e .hookSpecificOutput.additionalContext "$O/regla-ctx.json" >/dev/null \
   && [ "$(jq -r .hookSpecificOutput.hookEventName "$O/regla-ctx.json")" = SessionStart ] \
   && [ "$(jq -j .hookSpecificOutput.additionalContext "$O/regla-ctx.json"; echo x)" = "$(cat "$T/regla.txt"; echo x)" ] \
   && [ "$(jq -r '.hooks.SessionStart[0].hooks[0].command' "$O/settings.json")" = "cat $O/regla-ctx.json" ]; then
  ok ar_regla_con_comillas_es_json_valido; else ko "ar_regla_con_comillas_es_json_valido rc=$rc"; fi

# ar_hereda_restricciones_de_a0
bash "$CORRER" "$tarea" a0 1 >/dev/null 2>&1
A0="$K_ROOT/corridas/tt/a0-r1"
if grep -qxF "deny=Read(/$P/kb/**)" "$O/cmdline.txt" && grep -qxF "deny=Grep(/$P/kb/**)" "$O/cmdline.txt" \
   && grep -q "^PATH=$P/stub:" "$O/cmdline.txt" && cmp -s <(jq 'del(.hooks)' "$O/settings.json") <(jq 'del(.hooks)' "$A0/settings.json") \
   && diff <(grep -v '^PATH=' "$O/cmdline.txt") <(grep -v '^PATH=' "$A0/cmdline.txt") >/dev/null; then
  ok ar_hereda_restricciones_de_a0; else ko ar_hereda_restricciones_de_a0; fi

# a0 byte a byte contra el tarball (g1-57/a0-r1)
mkdir -p "$T/ref"; tar -xzf "$TAR" -C "$T/ref" corridas/g1-57/a0-r1/settings.json 2>/dev/null
if cmp -s "$A0/settings.json" "$T/ref/corridas/g1-57/a0-r1/settings.json"; then ok a0_settings_identico_a_K; else ko a0_settings_identico_a_K; fi

# a1/a2/a3: misma construcción jq que el correr.sh original
PLUG="$(cd "$HERE/../../plugins/exo/scripts" && pwd)"
esp(){ jq -n --argjson h "$1" '{autoMemoryEnabled:false, hooks:$h}'; }
exp1=$(esp "$(jq -n --arg c "cat $P/a1-inicio.json" '{SessionStart:[{hooks:[{type:"command",command:$c}]}]}')")
exp2=$(esp "$(jq -n --arg c "$PLUG/exo-recall.sh" '{SessionStart:[{hooks:[{type:"command",command:$c}]}]}')")
exp3=$(esp "$(jq -n --arg c "$PLUG/exo-recall.sh" --arg u "$PLUG/recall-inject.sh" '{SessionStart:[{hooks:[{type:"command",command:$c}]}],UserPromptSubmit:[{hooks:[{type:"command",command:$u}]}]}')")
for b in a1 a2 a3; do
  bash "$CORRER" "$tarea" $b 1 >/dev/null 2>&1
  e=exp${b#a}
  if [ "$(cat "$K_ROOT/corridas/tt/$b-r1/settings.json")" = "${!e}" ]; then ok "${b}_settings_sin_cambio"; else ko "${b}_settings_sin_cambio"; fi
done
exit $fail
