#!/usr/bin/env bash
# Test standalone para skills/orchestrate/scripts/rulings-collect.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RC_BIN="${SCRIPT_DIR}/../skills/orchestrate/scripts/rulings-collect"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

L="$TMP/progress.md"
cat > "$L" <<'EOT'
# Ledger
- Task 1: DONE
  - Ruling: T1 — usa glob — porque sí
- Task 2: DONE sin rulings
```
Ruling: dentro de fence, ignorado
```
- Task 4: Ruling: reabre el contrato — PR-pendiente
    Ruling: T5 — indentado
EOT

OUT=$("$RC_BIN" "$L"); rc=$?
want=$'Rulings: 3 (1 PR-pendiente)\n1. - Ruling: T1 — usa glob — porque sí\n2. ⚠ - Task 4: Ruling: reabre el contrato — PR-pendiente\n3. Ruling: T5 — indentado'
if [ "$rc" = 0 ] && [ "$OUT" = "$want" ]; then pass numeracion_orden_y_pr_pendiente; else fail numeracion_orden_y_pr_pendiente "rc=$rc: $OUT"; fi

if [[ "$OUT" != *"fence"* ]]; then pass fenced_ignorado; else fail fenced_ignorado "$OUT"; fi

printf '# Ledger\n- Task 1: DONE\n' > "$TMP/vacio.md"
OUT=$("$RC_BIN" "$TMP/vacio.md"); rc=$?
if [ "$rc" = 0 ] && [ "$OUT" = "Rulings: 0" ]; then pass sin_rulings; else fail sin_rulings "rc=$rc: $OUT"; fi

"$RC_BIN" "$TMP/no-existe.md" >/dev/null 2>&1; rc=$?
if [ "$rc" = 2 ]; then pass inexistente_exit2; else fail inexistente_exit2 "rc=$rc"; fi

printf '\n%d pass, %d fail\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
