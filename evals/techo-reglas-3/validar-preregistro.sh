#!/usr/bin/env bash
# Valida el pre-registro de techo-reglas-3: composición, bloques (sha, gold una vez, 1/10/20 reglas, anidados,
# regenerables), cláusulas, ORDEN recalculado, pins del gold y prep restaurado. No escribe nada fuera de un temporal.
# TECHO_RAIZ (def. raíz del repo) y K_ROOT son overrides para el test.
set -u
D="$(cd "$(dirname "$0")" && pwd)"
RAIZ="${TECHO_RAIZ:-$(cd "$D/../.." && pwd)}"
K="${K_ROOT:-$HOME/.cache/exo-ablacion-k}"
T="$D/tareas.tsv"; P="$D/preregistro.md"
SUELO="g0-122 g0-149 g0-33 g1-131 g1-139 g1-140 g1-157 g2-154 g2-170 g2-97"
fail=0
err() { echo "FALLA: $*" >&2; fail=1; }

[ -f "$T" ] || { echo "FALLA: falta $T" >&2; exit 1; }
[ -f "$P" ] || { echo "FALLA: falta preregistro.md" >&2; exit 1; }

# tareas.tsv: exactamente las 10 del suelo, un bloque por brazo
esperado=$(for i in $SUELO; do printf '%s\tsuelo\tevals/techo-reglas-3/bloques/%s/{brazo}.txt\tarp1:2,arp10:2,arp20:2\n' "$i" "$i"; done | sort)
[ "$(sort "$T")" = "$esperado" ] || err "tareas.tsv no tiene exactamente las 10 del suelo con arp1/arp10/arp20 k=2"

# bloques: sha sellado, y regenerarlos en un temporal da los mismos bytes
(cd "$D" && sha256sum -c --quiet bloques.sha256 >/dev/null 2>&1) || err "bloques/ no casa con bloques.sha256"
[ "$(wc -l < "$D/bloques.sha256")" -eq 30 ] || err "bloques.sha256 debe tener 30 líneas"
G=$(mktemp -d); trap 'rm -rf "$G"' EXIT
cp "$D/generar-bloques.py" "$D/pool.sha256" "$D/descartes.tsv" "$G/"
mkdir -p "$G/r/evals/techo-reglas-3" && mv "$G"/generar-bloques.py "$G"/pool.sha256 "$G"/descartes.tsv "$G/r/evals/techo-reglas-3/"
ln -s "$RAIZ/evals/techo-reglas" "$G/r/evals/techo-reglas"
python3 -I "$G/r/evals/techo-reglas-3/generar-bloques.py" >/dev/null 2>&1 || err "generar-bloques.py falla al regenerar"
cmp -s "$G/r/evals/techo-reglas-3/bloques.sha256" "$D/bloques.sha256" || err "regenerar los bloques no da bloques.sha256"
cmp -s "$G/r/evals/techo-reglas-3/seleccion.tsv" "$D/seleccion.tsv" || err "regenerar los bloques no da seleccion.tsv"

reglas() { grep -E '^- ' "$1"; }
for i in $SUELO; do
  gold="- $(cat "$RAIZ/evals/techo-reglas/reglas/$i.txt")"
  for n in 1 10 20; do
    b="$D/bloques/$i/arp$n.txt"
    [ -s "$b" ] || { err "$i: falta arp$n.txt"; continue; }
    [ "$(reglas "$b" | grep -cxF -- "$gold")" = 1 ] || err "$i/arp$n: la gold no aparece exactamente una vez"
    [ "$(reglas "$b" | wc -l)" = "$n" ] || err "$i/arp$n: no tiene $n reglas"
  done
  [ -s "$D/bloques/$i/arp10.txt" ] && [ -s "$D/bloques/$i/arp20.txt" ] || continue
  [ -z "$(comm -23 <(reglas "$D/bloques/$i/arp10.txt" | sort) <(reglas "$D/bloques/$i/arp20.txt" | sort))" ] || err "$i: arp10 no está contenido en arp20"
  head -1 "$D/bloques/$i/arp20.txt" | grep -qx '## Reglas duras del dueño' || err "$i/arp20: no empieza por «del dueño»"
done

# cláusulas: una línea de lista anclada por cláusula, con su contenido clave
clausula() { grep -E "^- \*\*$1[^*]*:\*\*" "$P" | grep -qE -- "$2" || err "cláusula '$1' ausente o alterada"; }
clausula Canal 'append-system-prompt-file.*sysprompt\.md'
FSHA=$(sha256sum "$D/framing.txt" 2>/dev/null | cut -d' ' -f1)
clausula Framing "framing\.txt.*sha256.*${FSHA:-sin-framing}"
clausula Bloques 'generar-bloques\.py.*20261025.*bloques\.sha256'
clausula 'Brazos' 'arp1.*arp10.*arp20.*k=2.*60'
clausula Cumple 'check\.rc == 0.*>=1 de sus 2'
clausula Replica 'arp1. >= 5/10'
clausula CAP '>= 5/10 da .CAP. 10; 4/10 da 7; <= 3/10 da 5'
clausula 'Invalidacion' 'arp10. <= 3/10.*7\.3'
clausula Modelo 'claude-sonnet-5-5'
clausula 'Version de' 'claude --version.*[0-9]+\.[0-9]+\.[0-9]+'
clausula 'Re-intentos' 'cero.*exactamente una vez'
clausula Breaker 'tanda es invalida.*no se adjudica'
clausula 'Gold y prep' 'pins\.sha256.*restaurar-prep\.sh'
clausula Potencia 'ruido'
clausula 'Distractoras' 'optimista'
grep -q 'semilla 20261025' "$P" || err "semilla 20261025 no fijada"
{ [ -s "$D/claude-version.txt" ] && grep -qF -- "$(cat "$D/claude-version.txt")" "$P"; } || err "claude-version.txt vacío o no citado en el pre-registro"

# ORDEN: el publicado debe ser el recalculado desde tareas.tsv
ORD="$(sed -n '/^<!-- ORDEN-BEGIN -->$/,/^<!-- ORDEN-END -->$/p' "$P" | grep -E '^[a-z0-9]+ [a-z0-9-]+ [0-9]+$')"
CALC="$(python3 -c 'import random,csv,sys; L=[]
for i,g,_,b in csv.reader(open(sys.argv[1]),delimiter="\t"):
    for x in b.split(","):
        a,k=x.split(":"); L+=[(a,i,r) for r in range(1,int(k)+1)]
L.sort(); random.Random(20261025).shuffle(L)
print("\n".join(f"{a} {i} {r}" for a,i,r in L))' "$T")"
[ "$(echo "$ORD" | grep -c .)" -eq 60 ] || err "ORDEN debe tener 60 líneas 'brazo id rep'"
[ "$CALC" = "$ORD" ] || err "el ORDEN no coincide con el barajado de la semilla 20261025 sobre tareas.tsv"

# gold y prep: pins de techo-reglas-2 y kb en el kb_commit de K
cmp -s "$D/pins.sha256" "$RAIZ/evals/techo-reglas-2/pins.sha256" || err "pins.sha256 no es el de techo-reglas-2"
(cd "$K" && sha256sum -c --quiet "$D/pins.sha256" >/dev/null 2>&1) || err "pins del gold/prep no coinciden en $K"
kc=$(awk '$1=="kb_commit"{print $2; exit}' "$RAIZ/evals/ablacion-k/congelacion.txt")
[ "$(git -C "$K/prep/kb" rev-parse HEAD 2>/dev/null)" = "$kc" ] || err "$K/prep/kb no está en $kc (corre restaurar-prep.sh)"
[ -x "$K/prep/stub/exo" ] || err "falta $K/prep/stub/exo"

[ $fail -eq 0 ] && echo "preregistro OK (10 × arp1/arp10/arp20 × 2 = 60)" || exit 1
