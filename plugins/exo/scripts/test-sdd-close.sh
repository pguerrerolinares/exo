#!/usr/bin/env bash
# Test standalone para `sdd-workspace close` (skills/orchestrate/scripts/sdd-workspace).
# Repos en mktemp -d; los worktrees llevan espacios en la ruta a propósito.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SW="${SCRIPT_DIR}/../skills/orchestrate/scripts/sdd-workspace"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

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

printf '\n%d pass, %d fail\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
