#!/usr/bin/env bash
# Test standalone para `sdd-workspace close` (skills/orchestrate/scripts/sdd-workspace).
# Repos en mktemp -d; los worktrees llevan espacios en la ruta a propósito.
# shellcheck disable=SC2016 # los backticks de los ledgers de prueba son literales
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SW="${SCRIPT_DIR}/../skills/orchestrate/scripts/sdd-workspace"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# Stub de gh: STUB_CI = estados separados por espacios (por defecto verde); "nopr" = sin PR (exit 1).
mkdir -p "$TMP/bin"
cat > "$TMP/bin/gh" <<'STUB'
#!/usr/bin/env bash
[ "${STUB_CI-SUCCESS}" != nopr ] || { echo "no pull requests found" >&2; exit 1; }
for s in ${STUB_CI-SUCCESS}; do echo "$s"; done
STUB
chmod +x "$TMP/bin/gh"
export PATH="$TMP/bin:$PATH"

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }
contains() { case "$1" in *"$2"*) return 0 ;; *) return 1 ;; esac; }

# mkrepo NOMBRE -> $REPO con un commit y docs/plan.md; $WS = workspace del plan con ledger
mkrepo() {
  RN=$1; REPO="$TMP/$1"; mkdir -p "$REPO/docs"
  git -C "$REPO" init -q -b main
  git -C "$REPO" config user.email t@t; git -C "$REPO" config user.name t
  echo base > "$REPO/README.md"; echo plan > "$REPO/docs/plan.md"
  git -C "$REPO" add README.md docs/plan.md; git -C "$REPO" commit -qm base
  ARCH="$TMP/$1-arch"
  mkws
}
mkws() {
  WS=$(cd "$REPO" && "$SW" docs/plan.md)
  echo "ledger" > "$WS/progress.md"; echo "final" > "$WS/final-review.md"
}
# close [ARGS...] -> $OUT, $RC (cwd = repo)
close() { OUT=$(cd "$REPO" && "$SW" close docs/plan.md --archive "$ARCH" "$@" 2>&1); RC=$?; }
# rama NOMBRE [integrada|nueva] -> worktree "$TMP/$RN wt NOMBRE" en rama task/NOMBRE
wt() {
  git -C "$REPO" worktree add -q -b "task/$1" "$TMP/$RN wt $1"
  git -C "$REPO" -C "$TMP/$RN wt $1" config user.email t@t
  echo "x $1" > "$TMP/$RN wt $1/f-$1.txt"; git -C "$TMP/$RN wt $1" add "f-$1.txt"; git -C "$TMP/$RN wt $1" commit -qm "t $1"
  [ "${2:-}" = integrada ] && git -C "$REPO" merge -q --no-edit "task/$1"
  return 0
}

# 1. archiva y borra
mkrepo c1; close
if [ "$RC" = 0 ] && [ -f "$ARCH/plan-$(date +%F)-progress.md" ] && [ -f "$ARCH/plan-$(date +%F)-final-review.md" ] \
   && [ ! -e "$WS" ] && contains "$OUT" "archivado: $ARCH/plan-" && contains "$OUT" "workspace borrado: $WS"; then
  pass close_archiva_y_borra; else fail close_archiva_y_borra "rc=$RC $OUT"; fi

# 2. no sobrescribe
mkws; close
if [ "$RC" = 0 ] && [ -f "$ARCH/plan-$(date +%F)-progress-2.md" ] && [ -f "$ARCH/plan-$(date +%F)-progress.md" ]; then
  pass close_no_sobrescribe; else fail close_no_sobrescribe "rc=$RC $(ls "$ARCH")"; fi

# 3. worktree integrado
mkrepo c3; wt t1 integrada; close
if [ "$RC" = 0 ] && ! git -C "$REPO" worktree list | grep -q "wt t1" && [ -z "$(git -C "$REPO" branch --list 'task/*')" ] \
   && contains "$OUT" "worktree eliminado: $TMP/$RN wt t1 (task/t1)" && contains "$OUT" "rama borrada: task/t1"; then
  pass close_worktree_integrado; else fail close_worktree_integrado "rc=$RC $OUT"; fi

# 3b. rama integrada sin worktree
mkrepo c3b; git -C "$REPO" branch task/solo; close
if [ "$RC" = 0 ] && [ -z "$(git -C "$REPO" branch --list 'task/*')" ] && contains "$OUT" "rama borrada: task/solo"; then
  pass close_rama_integrada_sin_worktree; else fail close_rama_integrada_sin_worktree "rc=$RC $OUT"; fi

# 4. rama sin integrar
mkrepo c4; wt t2; close
if [ "$RC" = 1 ] && [ -n "$(git -C "$REPO" branch --list task/t2)" ] && [ -d "$TMP/$RN wt t2" ] \
   && contains "$OUT" "sin integrar (se queda): task/t2" && [ ! -e "$WS" ] && [ -f "$ARCH/plan-$(date +%F)-progress.md" ]; then
  pass close_rama_sin_integrar; else fail close_rama_sin_integrar "rc=$RC $OUT"; fi

# 5. worktree sucio
mkrepo c5; wt t3 integrada; echo sucio >> "$TMP/$RN wt t3/f-t3.txt"; close
if [ "$RC" = 1 ] && contains "$OUT" "no eliminado: $TMP/$RN wt t3" && grep -q sucio "$TMP/$RN wt t3/f-t3.txt" \
   && [ -n "$(git -C "$REPO" branch --list task/t3)" ]; then
  pass close_worktree_sucio; else fail close_worktree_sucio "rc=$RC $OUT"; fi

# 6. respeta otros prefijos y workspaces hermanos
mkrepo c6
git -C "$REPO" worktree add -q -b feature/x "$TMP/$RN wt fx"
mkdir -p "$REPO/.superpowers/sdd/otro"; echo keep > "$REPO/.superpowers/sdd/otro/progress.md"
close
if [ "$RC" = 0 ] && [ -d "$TMP/$RN wt fx" ] && [ -n "$(git -C "$REPO" branch --list feature/x)" ] \
   && [ -f "$REPO/.superpowers/sdd/otro/progress.md" ] && [ -f "$REPO/.superpowers/sdd/.gitignore" ] && [ ! -e "$WS" ]; then
  pass close_respeta_otros; else fail close_respeta_otros "rc=$RC $OUT"; fi

# 6b. --prefix
mkrepo c6b; wt t1 integrada; git -C "$REPO" branch lane/y; close --prefix lane/
if [ "$RC" = 0 ] && [ -z "$(git -C "$REPO" branch --list 'lane/*')" ] && [ -n "$(git -C "$REPO" branch --list task/t1)" ]; then
  pass close_prefix; else fail close_prefix "rc=$RC $OUT"; fi

# 6c. HEAD que no es main: integrada respecto de la rama de feature actual
mkrepo c6c; git -C "$REPO" switch -q -c feat/z; wt t1 integrada; wt t2; close
if [ "$RC" = 1 ] && [ -z "$(git -C "$REPO" branch --list task/t1)" ] && [ -n "$(git -C "$REPO" branch --list feat/z)" ] \
   && contains "$OUT" "sin integrar (se queda): task/t2"; then
  pass close_desde_rama_feature; else fail close_desde_rama_feature "rc=$RC $OUT"; fi

# 6d. sin workspace: exit 2 y no lo crea; sin ledger: aviso y borra
mkrepo c6d; rm -rf "$REPO/.superpowers"; close
if [ "$RC" = 2 ] && [ ! -e "$REPO/.superpowers/sdd/plan" ]; then pass close_sin_workspace_exit2; else fail close_sin_workspace_exit2 "rc=$RC $OUT"; fi
mkrepo c6e; rm "$WS/progress.md" "$WS/final-review.md"; close
if [ "$RC" = 0 ] && contains "$OUT" "sin ledger que archivar" && [ ! -e "$WS" ]; then pass close_sin_ledger; else fail close_sin_ledger "rc=$RC $OUT"; fi

# 6f. archivo por defecto: EXO_SDD_ARCHIVE/<repo>/
mkrepo c6f; OUT=$(cd "$REPO" && EXO_SDD_ARCHIVE="$TMP/def" "$SW" close docs/plan.md 2>&1); RC=$?
if [ "$RC" = 0 ] && [ -f "$TMP/def/c6f/plan-$(date +%F)-progress.md" ]; then pass close_archivo_por_defecto; else fail close_archivo_por_defecto "rc=$RC $OUT"; fi

# 7. uso clásico
mkrepo c7; got=$(cd "$REPO" && "$SW" docs/plan.md)
if [ "$got" = "$WS" ] && [ "$(basename "$got")" = plan ]; then pass uso_clasico_intacto; else fail uso_clasico_intacto "$got"; fi

# 8. --prefix vacío o con glob: exit 2 sin tocar nada (main integrado en HEAD no puede caer)
mkrepo c8; git -C "$REPO" switch -q -c feat/z; wt t1 integrada
for pf in "" "*" m ma mai main task; do
  rm -rf "$ARCH"; [ -d "$WS" ] || mkws
  close --prefix "$pf"
  if [ "$RC" = 2 ] && [ -n "$(git -C "$REPO" branch --list main)" ] && [ -n "$(git -C "$REPO" branch --list task/t1)" ] \
     && [ -d "$TMP/$RN wt t1" ] && [ -d "$WS" ] && [ ! -e "$ARCH" ]; then
    pass "close_prefix_peligroso_rechazado[$pf]"; else fail "close_prefix_peligroso_rechazado[$pf]" "rc=$RC $OUT"; fi
done

# 9. final-review.md sin progress.md: se archiva igual y el workspace se borra
mkrepo c9; rm "$WS/progress.md"; close
if [ "$RC" = 0 ] && [ -f "$ARCH/plan-$(date +%F)-final-review.md" ] && [ ! -e "$WS" ] && contains "$OUT" "sin ledger que archivar"; then
  pass close_final_review_sin_progress; else fail close_final_review_sin_progress "rc=$RC $OUT"; fi

# 10. gate de CI: rojo/pendiente/sin PR/sin checks => exit 1 y NADA se toca; verde => cierra
for caso in "rojo:SUCCESS FAILURE" "pendiente:SUCCESS PENDING" "en_curso:IN_PROGRESS" "sin_pr:nopr" "sin_checks:"; do
  n=${caso%%:*}; mkrepo "c10$n"
  OUT=$(cd "$REPO" && STUB_CI="${caso#*:}" "$SW" close docs/plan.md --archive "$ARCH" 2>&1); RC=$?
  if [ "$RC" = 1 ] && contains "$OUT" "CI no verde: fix dispatch con el ledger vivo" && [ -f "$WS/progress.md" ] && [ ! -e "$ARCH" ]; then
    pass "close_ci_bloquea[$n]"; else fail "close_ci_bloquea[$n]" "rc=$RC $OUT"; fi
done
mkrepo c10v; OUT=$(cd "$REPO" && STUB_CI="SUCCESS SKIPPED" "$SW" close docs/plan.md --archive "$ARCH" 2>&1); RC=$?
if [ "$RC" = 0 ] && [ ! -e "$WS" ]; then pass close_ci_verde_cierra; else fail close_ci_verde_cierra "rc=$RC $OUT"; fi

# 10b. --no-ci MOTIVO: salta el gate y deja la línea en el ledger archivado
mkrepo c10n; OUT=$(cd "$REPO" && STUB_CI=FAILURE "$SW" close docs/plan.md --archive "$ARCH" --no-ci "repo sin CI" 2>&1); RC=$?
if [ "$RC" = 0 ] && [ ! -e "$WS" ] && grep -qx 'close: --no-ci (repo sin CI)' "$ARCH/plan-$(date +%F)-progress.md"; then
  pass close_no_ci_registra_en_ledger; else fail close_no_ci_registra_en_ledger "rc=$RC $OUT"; fi
mkrepo c10m; OUT=$(cd "$REPO" && "$SW" close docs/plan.md --archive "$ARCH" --no-ci 2>&1); RC=$?
if [ "$RC" = 2 ] && [ -d "$WS" ]; then pass close_no_ci_sin_motivo_exit2; else fail close_no_ci_sin_motivo_exit2 "rc=$RC $OUT"; fi

# 11. wave-close: ejecuta cada comando del ledger, escribe las líneas y sale con el nº de rojos
PLAN2=$'# Plan\n\n## Task 1: a\n\n**Files:**\n- Create: `a.txt`\n\n## Task 2: b\n\n**Files:**\n- Create: `b.txt`\n\n**Interfaces:**\n- Consumes: x @Task 1\n'
# mkwave NOMBRE LEDGER_CIERRE -> repo con plan de 2 olas y ledger con la sección dada
mkwave() {
  mkrepo "$1"; printf '%s' "$PLAN2" > "$REPO/docs/plan.md"
  git -C "$REPO" add docs/plan.md; git -C "$REPO" commit -qm plan2
  printf '# SDD ledger — plan: docs/plan.md\n\n## Cierre de ola\n%s\n' "$2" > "$WS/progress.md"
}
wclose() { OUT=$(cd "$REPO" && "$SW" wave-close docs/plan.md "$1" 2>&1); RC=$?; }
mkwave w1 $'- `echo uno && echo hecho` → verde = hecho\n- `test -d .` → verde = ok'
wclose 1
sha=$(git -C "$REPO" rev-parse --short HEAD)
if [ "$RC" = 0 ] && grep -qxF 'Ola 1: cierre — echo uno && echo hecho: exit=0 (hecho)' "$WS/progress.md" \
   && grep -qxF "Ola 1: cierre — verde (2/2) @$sha" "$WS/progress.md"; then
  pass wave_close_verde; else fail wave_close_verde "rc=$RC $OUT / $(tail -3 "$WS/progress.md")"; fi

mkwave w2 $'- `true` → ok\n- `echo roto; exit 3` → ok\n- `false` → ok'
wclose 1
if [ "$RC" = 2 ] && grep -qxF 'Ola 1: cierre — echo roto; exit 3: exit=3 (roto)' "$WS/progress.md" \
   && grep -q '^Ola 1: cierre — ROJO (2/3)' "$WS/progress.md" && ! grep -q 'cierre — verde' "$WS/progress.md"; then
  pass wave_close_rojo_exit_nrojos; else fail wave_close_rojo_exit_nrojos "rc=$RC $(tail -4 "$WS/progress.md")"; fi

mkrepo w3; printf '# SDD ledger\n' > "$WS/progress.md"; wclose 1
if [ "$RC" = 1 ] && ! grep -q 'cierre' "$WS/progress.md"; then pass wave_close_sin_seccion_exit1; else fail wave_close_sin_seccion_exit1 "rc=$RC $OUT"; fi
mkwave w3b '- `true`'; wclose x
if [ "$RC" = 2 ]; then pass wave_close_k_invalida_exit2; else fail wave_close_k_invalida_exit2 "rc=$RC $OUT"; fi

# 12. close exige la última ola en verde (plan de 2 olas), sumado al gate de CI
mkwave w4 '- `true`'; wclose 1
close --no-ci t
if [ "$RC" = 1 ] && contains "$OUT" "Ola 2 sin cierre verde" && [ -f "$WS/progress.md" ] && [ ! -e "$ARCH" ]; then
  pass close_ultima_ola_sin_cierre; else fail close_ultima_ola_sin_cierre "rc=$RC $OUT"; fi
wclose 2
OUT=$(cd "$REPO" && STUB_CI=FAILURE "$SW" close docs/plan.md --archive "$ARCH" 2>&1); RC=$?
if [ "$RC" = 1 ] && contains "$OUT" "CI no verde" && [ -f "$WS/progress.md" ]; then
  pass close_ola_verde_pero_ci_rojo; else fail close_ola_verde_pero_ci_rojo "rc=$RC $OUT"; fi
close
if [ "$RC" = 0 ] && [ ! -e "$WS" ]; then pass close_ola_verde_y_ci_verde; else fail close_ola_verde_y_ci_verde "rc=$RC $OUT"; fi
mkwave w5 '- `false`'; wclose 1; wclose 2; close --no-ci t
if [ "$RC" = 1 ] && contains "$OUT" "Ola 2 sin cierre verde"; then pass close_ultima_ola_roja; else fail close_ultima_ola_roja "rc=$RC $OUT"; fi

# 13. Métricas: una línea `Métricas: ` en el ledger archivado, antes de archivar
metrica() { grep '^Métricas: ' "$ARCH/plan-$(date +%F)-progress.md" 2>/dev/null; }
mkwave m1 '- `true`'; wclose 1; wclose 2
printf '\n```\ncodigo\n```\n' >> "$REPO/docs/plan.md"
kb=$(awk -v b="$(wc -c < "$REPO/docs/plan.md")" 'BEGIN { printf "%.1f", b / 1024 }')
dagout=$("$SCRIPT_DIR/../skills/orchestrate/scripts/task-dag" "$REPO/docs/plan.md")
close; M=$(metrica)
if [ "$RC" = 0 ] && [ "$(printf '%s\n' "$M" | wc -l)" = 1 ] && contains "$M" 'olas=2 anchos=1,1 ' && contains "$dagout" '[[1], [2]]' \
   && contains "$M" "plan_kb=$kb " && contains "$M" 'codigo=0.1' && contains "$M" ' mutacion=sin-dato ' \
   && contains "$M" 'wallclock=manual basura=manual turnos=manual'; then
  pass close_metricas_basicas; else fail close_metricas_basicas "rc=$RC M=$M dag=$dagout"; fi
echo "# ejemplo: $M"

# 13b. mutación: score del package (single y por proyecto, peor caso entre packages, muestreados fuera)
mkwave m2 '- `true`'; wclose 1; wclose 2
printf 'diff\n## MUTACIÓN\nMUTACIÓN: 5/6 (83%%) — 1 superviviente(s)\n' > "$WS/review-aaa..bbb.diff"
printf 'diff\n## MUTACIÓN\nMUTACIÓN: 7/10 (70%%) — 3 superviviente(s)\nMUTACIÓN aviso: x (99%%)\n' > "$WS/review-ccc..ddd.diff"
close; M=$(metrica)
if [ "$RC" = 0 ] && contains "$M" ' mutacion=global:70% '; then pass close_metricas_mutacion_score; else fail close_metricas_mutacion_score "rc=$RC M=$M"; fi
mkwave m3 '- `true`'; wclose 1; wclose 2
printf '## MUTACIÓN\nMUTACIÓN: 9/10 (90%%) — 1 superviviente(s); 1 proyecto(s) sin score\nMUTACIÓN [api]: 9/10 (90%%) — 1 superviviente(s)\nMUTACIÓN [web]: parcial (timeout 5s)\nMUTACIÓN [py]: 10/20 (50%%) IC95 [30%%, 70%%] — muestra de 10/200 (semilla 1)\n' > "$WS/review-a..b.diff"
close; M=$(metrica)
if [ "$RC" = 0 ] && contains "$M" ' mutacion=api:90% '; then pass close_metricas_mutacion_por_proyecto; else fail close_metricas_mutacion_por_proyecto "rc=$RC M=$M"; fi
mkwave m4 '- `true`'; wclose 1; wclose 2
printf '## MUTACIÓN\nMUTACIÓN: no disponible (sin herramienta para este repo)\n' > "$WS/review-a..b.diff"
close; M=$(metrica)
if [ "$RC" = 0 ] && contains "$M" ' mutacion=sin-dato '; then pass close_metricas_mutacion_no_disponible; else fail close_metricas_mutacion_no_disponible "rc=$RC M=$M"; fi

# 13c. un package ilegible => mutacion=? y el close sigue igual (exit 0, archiva, borra)
mkwave m5 '- `true`'; wclose 1; wclose 2; mkdir "$WS/review-roto..x.diff"
close; M=$(metrica)
if [ "$RC" = 0 ] && contains "$M" ' mutacion=? ' && contains "$M" 'olas=2 ' && [ ! -e "$WS" ]; then
  pass close_metricas_fallo_no_rompe_close; else fail close_metricas_fallo_no_rompe_close "rc=$RC M=$M $OUT"; fi

printf '\n%d pass, %d fail\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
