#!/usr/bin/env bash
# Tests bash (sin claude real) del brazo n, el aislamiento por corrida y la restauración de la KB en correr.sh.
# GOLDENS_GENERAR=1 reescribe goldens/ con el correr.sh actual (se generaron con la base 7382b18, antes del brazo n).
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
CORRER="${CORRER:-$HERE/../ablacion-k/harness/correr.sh}"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
fail=0
ok(){ echo "ok   $1"; }; ko(){ echo "FAIL $1"; fail=1; }

export K_ROOT="$T/k"; P="$K_ROOT/prep"
mkdir -p "$P/stub" "$K_ROOT/fuentes/tt" "$T/bin"; : > "$P/manifiesto.txt"
printf 'base\n' > "$P/claude-md.md"
git -C "$K_ROOT/fuentes/tt" init -q
mkdir -p "$P/kb"; git -C "$P/kb" init -q; echo base > "$P/kb/k.md"
git -C "$P/kb" add k.md; git -C "$P/kb" -c user.name=t -c user.email=t@t commit -qm base
KB_BASE=$(git -C "$P/kb" rev-parse HEAD); echo "kb_commit $KB_BASE" > "$P/manifiesto.txt"
tarea="$T/tareas/tt"; mkdir -p "$tarea"; echo '{"prompt":"x"}' > "$tarea/tarea.json"
export K_MEMORIA_DIR="$T/memoria-orig"; mkdir -p "$K_MEMORIA_DIR/sub"
printf -- '- [uno](uno.md) — primera\n' > "$K_MEMORIA_DIR/MEMORY.md"
printf 'uno\n' > "$K_MEMORIA_DIR/uno.md"; printf 'hondo\n' > "$K_MEMORIA_DIR/sub/dos.md"
printf 'Regla "uno" con \\ barra y $VAR\n' > "$T/regla.txt"; export K_REGLA_FILE="$T/regla.txt"
export K_FRAMING_FILE="$HERE/../techo-reglas-2/framing.txt"
export K_ENSAYO=1

ensayo(){ bash "$CORRER" "$tarea" "$1" "${2:-1}" >/dev/null 2>&1; }
dir(){ echo "$K_ROOT/corridas/tt/$1-r${2:-1}"; }

# n_settings_activa_memoria
ensayo n; O=$(dir n)
if [ "$(jq -r .autoMemoryEnabled "$O/settings.json" 2>/dev/null)" = true ] && [ "$(jq -r .autoMemoryDirectory "$O/settings.json" 2>/dev/null)" = "$O/memoria" ]; then
  ok n_settings_activa_memoria; else ko n_settings_activa_memoria; fi

# otros_brazos_memoria_off: n y los demás en el mismo test; solo n tiene la memoria
for b in a0 ar a2; do ensayo $b; done
solo_n=1
for b in a0 ar a2; do
  s="$(dir $b)/settings.json"
  [ "$(jq -r .autoMemoryEnabled "$s")" = false ] && [ "$(jq 'has("autoMemoryDirectory")' "$s")" = false ] || solo_n=0
done
[ "$(jq -r .autoMemoryEnabled "$O/settings.json" 2>/dev/null)" = true ] && [ "$(jq 'has("autoMemoryDirectory")' "$O/settings.json" 2>/dev/null)" = true ] || solo_n=0
[ $solo_n = 1 ] && ok otros_brazos_memoria_off || ko otros_brazos_memoria_off

# n_copia_fresca: copia real (no symlink), una por réplica; tocar una no toca el origen ni la otra
ensayo n 2; O2=$(dir n 2)
orig_sha=$(sha256sum "$K_MEMORIA_DIR/MEMORY.md" | cut -d' ' -f1)
c=1
for d in "$O/memoria" "$O2/memoria"; do [ -d "$d" ] && [ ! -L "$d" ] && diff -r "$d" "$K_MEMORIA_DIR" >/dev/null || c=0; done
[ "$O/memoria" != "$O2/memoria" ] || c=0
echo tocado > "$O/memoria/nuevo.md"; echo tocado >> "$O/memoria/MEMORY.md"
[ ! -e "$K_MEMORIA_DIR/nuevo.md" ] && [ ! -e "$O2/memoria/nuevo.md" ] || c=0
[ "$(sha256sum "$K_MEMORIA_DIR/MEMORY.md" | cut -d' ' -f1)" = "$orig_sha" ] && [ "$(sha256sum "$O2/memoria/MEMORY.md" | cut -d' ' -f1)" = "$orig_sha" ] || c=0
cmp -s "$O/inyectado.txt" <(cat "$K_MEMORIA_DIR/MEMORY.md") && cmp -s "$O2/inyectado.txt" "$K_MEMORIA_DIR/MEMORY.md" || c=0
[ $c = 1 ] && ok n_copia_fresca || ko n_copia_fresca

# n_deniega_snapshot
ensayo n; O=$(dir n)
if grep -qxF "deny=Read(/$P/kb/**)" "$O/cmdline.txt" && grep -qxF "deny=Grep(/$P/kb/**)" "$O/cmdline.txt" && grep -q "^PATH=$P/stub:" "$O/cmdline.txt"; then
  ok n_deniega_snapshot; else ko n_deniega_snapshot; fi

# restaura_kb: sin ensayo; claude falso que ensucia la KB; con salida 0, con salida 1 y con n
cat > "$T/bin/claude" <<STUB
#!/usr/bin/env bash
[ "\${1:-}" = --version ] && { echo fake; exit 0; }
echo lanzado > "$T/testigo"; echo sucio > "$P/kb/nuevo-agente.md"; echo cambiado >> "$P/kb/k.md"
if [ -n "\${STUB_COMMIT:-}" ]; then git -C "$P/kb" add -A; git -C "$P/kb" -c user.name=t -c user.email=t@t commit -qm agente; fi
exit \${STUB_RC:-0}
STUB
chmod +x "$T/bin/claude"
unset K_ENSAYO
for caso in "a0 0" "a0 1" "n 1" "a0 124"; do
  set -- $caso; b=$1; export STUB_RC=$2
  PATH="$T/bin:$PATH" bash "$CORRER" "$tarea" $b 7 >/dev/null 2>&1; Ob=$(dir $b 7)
  if grep -q 'nuevo-agente.md' "$Ob/kb-sucia.txt" 2>/dev/null && [ "$(grep -c . "$Ob/kb-sucia.txt")" = 2 ] \
     && [ -z "$(git -C "$P/kb" status --porcelain)" ] && [ ! -e "$P/kb/nuevo-agente.md" ]; then
    ok "restaura_kb ($b, claude rc=$2)"; else ko "restaura_kb ($b, claude rc=$2) sucia=$(cat "$Ob/kb-sucia.txt" 2>/dev/null) ahora=$(git -C "$P/kb" status --porcelain)"; fi
done
unset STUB_RC

# restaura_kb_commit: el agente hace add + commit; HEAD vuelve a kb_commit y kb-sucia.txt lo refleja
export STUB_RC=0 STUB_COMMIT=1
PATH="$T/bin:$PATH" bash "$CORRER" "$tarea" a0 9 >/dev/null 2>&1; Oc=$(dir a0 9)
if [ "$(git -C "$P/kb" rev-parse HEAD)" = "$KB_BASE" ] && [ -z "$(git -C "$P/kb" status --porcelain)" ] && [ ! -e "$P/kb/nuevo-agente.md" ] \
   && grep -q 'nuevo-agente.md\|^C ' "$Oc/kb-sucia.txt" && grep -q '^C .* agente$' "$Oc/kb-sucia.txt"; then
  ok restaura_kb_commit; else ko "restaura_kb_commit head=$(git -C "$P/kb" rev-parse --short HEAD) sucia=$(cat "$Oc/kb-sucia.txt" 2>/dev/null)"; fi
unset STUB_RC STUB_COMMIT

# guarda_antes_de_lanzar: sin .git, sin kb_commit o con kb_commit inexistente => exit 3 y claude no se invoca
guarda(){ # $1 caso
  rm -f "$T/testigo"
  PATH="$T/bin:$PATH" bash "$CORRER" "$tarea" a0 10 >/dev/null 2>&1; rc=$?
  if [ $rc = 3 ] && [ ! -e "$T/testigo" ] && [ ! -e "$(dir a0 10)/transcript.jsonl" ]; then ok "guarda_antes_de_lanzar ($1)"; else ko "guarda_antes_de_lanzar ($1) rc=$rc testigo=$(ls "$T/testigo" 2>&1)"; fi
}
mv "$P/kb/.git" "$T/kb-git"; guarda sin-git; mv "$T/kb-git" "$P/kb/.git"
cp "$P/manifiesto.txt" "$T/manifiesto.bak"
echo "exo x" > "$P/manifiesto.txt"; guarda sin-kb_commit
echo "kb_commit 0123456789abcdef0123456789abcdef01234567" > "$P/manifiesto.txt"; guarda kb_commit-inexistente
cp "$T/manifiesto.bak" "$P/manifiesto.txt"

# guarda_kb_contaminada: fichero sin trackear o HEAD movido antes de empezar => exit 3 sin lanzar claude
echo previo > "$P/kb/previo.md"; guarda kb-sucia-al-arrancar; rm -f "$P/kb/previo.md"
echo mas >> "$P/kb/k.md"; git -C "$P/kb" -c user.name=t -c user.email=t@t commit -qam movido; guarda head-movido
git -C "$P/kb" reset -q --hard "$KB_BASE"

# restaura_kb_fallo_visible: reset/clean que falla => "restauracion: fallo" y exit 3
cat > "$T/bin/git" <<GIT
#!/usr/bin/env bash
case "\$*" in *"reset -q --hard"*) exit 1 ;; esac
exec $(command -v git) "\$@"
GIT
chmod +x "$T/bin/git"
PATH="$T/bin:$PATH" bash "$CORRER" "$tarea" a0 11 >/dev/null 2>&1; rc=$?
if [ $rc = 3 ] && grep -qxF 'restauracion: fallo' "$(dir a0 11)/kb-sucia.txt"; then ok restaura_kb_fallo_visible; else ko "restaura_kb_fallo_visible rc=$rc"; fi
rm -f "$T/bin/git"; git -C "$P/kb" reset -q --hard "$KB_BASE"; git -C "$P/kb" clean -fdq

# ensayo_no_restaura: K_ENSAYO=1 sale antes de tocar la KB
echo sucio > "$P/kb/nuevo-agente.md"
K_ENSAYO=1 bash "$CORRER" "$tarea" a0 8 >/dev/null 2>&1
[ -e "$P/kb/nuevo-agente.md" ] && [ ! -e "$(dir a0 8)/kb-sucia.txt" ] && ok ensayo_no_restaura || ko ensayo_no_restaura
rm -f "$P/kb/nuevo-agente.md"

# invariancia_brazos_viejos: contra goldens generados con la base 7382b18; rutas variables normalizadas
export K_ENSAYO=1
norm(){ sed -e "s|$1|<O>|g" -e "s|$K_ROOT|<K_ROOT>|g" -e "s|$T|<T>|g" -e "s|:$PATH\$|:<PATH>|"; }
inv=1
for b in a0 ar arp ars; do
  ensayo $b 3; Ob=$(dir $b 3)
  for f in settings.json cmdline.txt; do
    g="$HERE/goldens/$b-$f"
    if [ -n "${GOLDENS_GENERAR:-}" ]; then norm "$Ob" < "$Ob/$f" > "$g"
    else diff <(norm "$Ob" < "$Ob/$f") "$g" >/dev/null 2>&1 || { inv=0; echo "  difiere: $b/$f"; }; fi
  done
done
[ $inv = 1 ] && ok invariancia_brazos_viejos || ko invariancia_brazos_viejos

# k_corridas: correr.sh escribe en $K_CORRIDAS si está, y no en corridas/
K_CORRIDAS=corridas-criba bash "$CORRER" "$tarea" a0 41 >/dev/null 2>&1
[ -f "$K_ROOT/corridas-criba/tt/a0-r41/inicio.txt" ] && [ ! -e "$K_ROOT/corridas/tt/a0-r41" ] && ok k_corridas || ko k_corridas
exit $fail
