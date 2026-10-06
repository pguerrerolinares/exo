#!/usr/bin/env bash
# Valida el pre-registro de techo-reglas-2: composicion, clausulas, ORDEN recalculado, gold sin campana-k.
# TECHO_RAIZ (def. raiz del repo) y K_ROOT son overrides para el test.
set -u
D="$(cd "$(dirname "$0")" && pwd)"
RAIZ="${TECHO_RAIZ:-$(cd "$D/../.." && pwd)}"
K="${K_ROOT:-$HOME/.cache/exo-ablacion-k}"
T="$D/tareas.tsv"; P="$D/preregistro.md"
SUELO="g0-122 g0-149 g0-33 g1-131 g1-139 g1-140 g1-157 g2-154 g2-170 g2-97"
CONTROL="g0-159 g1-144 g1-16 g1-25 g1-34 g2-35"
fail=0
err() { echo "FALLA: $*" >&2; fail=1; }

[ -f "$T" ] || { echo "FALLA: falta $T" >&2; exit 1; }
[ -f "$P" ] || { echo "FALLA: falta preregistro.md" >&2; exit 1; }

# tareas.tsv: exactamente las 10 + 6, cada una con su regla_file y sus brazos
esperado=$( { for i in $SUELO; do printf '%s\tsuelo\tevals/techo-reglas/reglas/%s.txt\tarp:2,a0:2\n' "$i" "$i"; done
              for i in $CONTROL; do printf '%s\tcontrol\tevals/techo-reglas/reglas/%s.txt\tarp:2\n' "$i" "$i"; done; } | sort)
[ "$(sort "$T")" = "$esperado" ] || err "tareas.tsv no tiene exactamente las 10 + 6 con sus brazos"
while IFS=$'\t' read -r id _ rf _; do
  [ -s "$RAIZ/$rf" ] || err "$id: $rf ausente o vacio"
done < "$T"

# clausulas: una linea de lista anclada por clausula, con su contenido clave
clausula() { # etiqueta, regex de contenido
  grep -E "^- \*\*$1[^*]*:\*\*" "$P" | grep -qE -- "$2" || err "clausula '$1' ausente o alterada"
}
clausula Canal 'append-system-prompt-file.*sysprompt\.md'
FSHA=$(sha256sum "$RAIZ/evals/techo-reglas-2/framing.txt" 2>/dev/null | cut -d' ' -f1)
clausula Framing "framing\.txt.*sha256.*${FSHA:-sin-framing}"
clausula 'Brazos' 'arp.*a0.*k=2.*52'
clausula Cumple 'check\.rc == 0.*>=1 de sus 2'
clausula Caida 'check\.rc != 0.*2/2'
clausula Gate '>= 6/10.*margen >= 3.*0/6'
clausula 'No reconstruible' 'no cumple.*caida'
clausula Modelo 'claude-sonnet-5-5'
clausula 'Version de' 'claude --version.*[0-9]+\.[0-9]+\.[0-9]+'
clausula 'Re-intentos' 'cero.*exactamente una vez'
clausula Breaker 'tanda es invalida.*no se adjudica'
clausula 'Clases' 'conflicto regla-tarea.*check roto.*regla mal escrita.*no reconstruible.*incumplimiento del agente'
clausula Cierre 'sin tercera bala'
clausula Gold 'tarball.*erratas-gold\.md'
clausula Potencia 'infrapotenciado'
clausula 'Sin placebo' 'placebo.*autoridad'
grep -q 'semilla 20261006' "$P" || err "semilla 20261006 no fijada"
[ -s "$D/claude-version.txt" ] && grep -qF -- "$(cat "$D/claude-version.txt")" "$P" || err "claude-version.txt vacio o no citado en el pre-registro"

# ORDEN: el publicado debe ser el recalculado desde tareas.tsv
ORD="$(sed -n '/^<!-- ORDEN-BEGIN -->$/,/^<!-- ORDEN-END -->$/p' "$P" | grep -E '^[a-z0-9]+ [a-z0-9-]+ [0-9]+$')"
CALC="$(python3 -c 'import random,csv,sys; L=[]
for i,g,_,b in csv.reader(open(sys.argv[1]),delimiter="\t"):
    for x in b.split(","):
        a,k=x.split(":"); L+=[(a,i,r) for r in range(1,int(k)+1)]
L.sort(); random.Random(20261006).shuffle(L)
print("\n".join(f"{a} {i} {r}" for a,i,r in L))' "$T")"
[ "$(echo "$ORD" | grep -c .)" -eq 52 ] || err "ORDEN debe tener 52 lineas 'brazo id rep'"
grep -q 'g2-171' "$T" && err "g2-171 aparece en tareas.tsv (debe estar fuera)"
echo "$ORD" | grep -q 'g2-171' && err "g2-171 aparece en el ORDEN (debe estar fuera)"
[ "$CALC" = "$ORD" ] || err "el ORDEN no coincide con el barajado de la semilla 20261006 sobre tareas.tsv"

# gold instalado: ningun check de las 16 con la ruta del worktree borrado
for id in $(cut -f1 "$T"); do
  c="$K/gold/s1/$id/check.sh"
  [ -f "$c" ] || { err "$id: falta $c"; continue; }
  ! grep -q 'worktrees/campana-k' "$c" || err "$id: check.sh contiene worktrees/campana-k"
done

[ $fail -eq 0 ] && echo "preregistro OK (10+6)" || exit 1
