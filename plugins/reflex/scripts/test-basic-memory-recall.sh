#!/usr/bin/env bash
# Test standalone para basic-memory-recall.sh (SessionStart hook).
# Casos:
#  1. Ejecución normal → JSON válido, additionalContext trae el contrato de
#     memoria (KB real) o el fallback embebido, ≤8000 bytes, y si vino de la
#     KB es el .content real (markdown con newlines de verdad, header
#     "## Contrato de memoria"), NO el JSON crudo de read-note escapado.
#  2. BM_RECALL_UVX=/nonexistent → fuerza fallo del uvx real → JSON válido,
#     additionalContext = fallback.
#  3. Exit code siempre 0 en ambos casos.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/basic-memory-recall.sh"

# Aislamiento global (F3.1): compose_base ahora loguea sus fallbacks vía
# _reflex-log.sh, que por defecto escribe en $HOME/.claude/reflex-log.jsonl.
# Sin este default, cualquier caso que dispare un fallback (la mayoría, al no
# tener KB real accesible) ensuciaría el log real. Los casos que necesitan su
# propio REFLEX_LOG_FILE (p.ej. caso14) lo prefijan inline y ganan por ámbito.
TESTLOGDIR="$(mktemp -d)"
trap 'rm -rf "$TESTLOGDIR"' EXIT
export REFLEX_LOG_FILE="$TESTLOGDIR/reflex-log.jsonl"

PASS=0
FAIL=0

echo "=== test-basic-memory-recall.sh ==="
echo ""

# ---------------------------------------------------------------------------
# CASO 1: ejecución normal (input JSON vacío) → JSON válido, contrato de memoria o fallback,
# ≤8000 bytes, exit 0
# ---------------------------------------------------------------------------
{
  OUTPUT="$(printf '{}' | bash "$HOOK" 2>/dev/null)"
  EC=$?

  if ! printf '%s' "$OUTPUT" | jq . >/dev/null 2>&1; then
    printf '[FAIL] caso1: stdout no es JSON válido\n'
    printf '       output: %s\n' "$OUTPUT"
    FAIL=$((FAIL+1))
  else
    CTX="$(printf '%s' "$OUTPUT" | jq -r '.hookSpecificOutput.additionalContext')"
    BYTES="${#CTX}"

    if [ "$BYTES" -le 8000 ]; then
      printf '[PASS] caso1: %d bytes ≤ 8000\n' "$BYTES"
      PASS=$((PASS+1))
    else
      printf '[FAIL] caso1: additionalContext excede 8000 bytes (%d)\n' "$BYTES"
      FAIL=$((FAIL+1))
    fi

    if printf '%s' "$CTX" | grep -q 'Contrato de memoria'; then
      # Vino de la KB: debe ser el .content real (markdown con newlines de
      # verdad), no el JSON crudo de read-note escapado.
      if printf '%s' "$CTX" | grep -qF '\n'; then
        printf '[FAIL] caso1: additionalContext contiene la secuencia literal backslash-n — parece JSON escapado, no el .content extraído\n'
        FAIL=$((FAIL+1))
      else
        printf '[PASS] caso1: sin escaping JSON literal (\\n) en additionalContext\n'
        PASS=$((PASS+1))
      fi

      if printf '%s' "$CTX" | grep -q '^## Contrato de memoria$'; then
        printf '[PASS] caso1: header markdown real "## Contrato de memoria" presente (newline real antes/después)\n'
        PASS=$((PASS+1))
      else
        printf '[FAIL] caso1: no se encontró el header markdown "## Contrato de memoria" en su propia línea\n'
        printf '       output: %s\n' "$CTX"
        FAIL=$((FAIL+1))
      fi
    elif printf '%s' "$CTX" | grep -q 'memoria persistente es el MCP basic-memory'; then
      printf '[PASS] caso1: fallback embebido presente (KB no respondió, aceptable)\n'
      PASS=$((PASS+1))
    else
      printf '[FAIL] caso1: additionalContext no contiene ni contrato de memoria ni fallback\n'
      printf '       output: %s\n' "$CTX"
      FAIL=$((FAIL+1))
    fi
  fi

  if [ "$EC" -eq 0 ]; then
    printf '[PASS] caso1: exit code 0\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso1: exit code %d (esperaba 0)\n' "$EC"
    FAIL=$((FAIL+1))
  fi
}

# ---------------------------------------------------------------------------
# CASO 2: BM_RECALL_UVX=/nonexistent → fuerza fallback, JSON válido, exit 0
# ---------------------------------------------------------------------------
{
  OUTPUT="$(printf '{}' | BM_RECALL_UVX=/nonexistent bash "$HOOK" 2>/dev/null)"
  EC=$?

  if ! printf '%s' "$OUTPUT" | jq . >/dev/null 2>&1; then
    printf '[FAIL] caso2: stdout no es JSON válido con BM_RECALL_UVX=/nonexistent\n'
    printf '       output: %s\n' "$OUTPUT"
    FAIL=$((FAIL+1))
  else
    CTX="$(printf '%s' "$OUTPUT" | jq -r '.hookSpecificOutput.additionalContext')"
    if printf '%s' "$CTX" | grep -q 'memoria persistente es el MCP basic-memory'; then
      # BM_RECALL_UVX roto debe usarse para AMBAS llamadas (core-index y
      # recent-activity) — si el hook honra el override, ninguna de las dos
      # puede haber tenido éxito, así que no debe aparecer digest de actividad.
      if printf '%s' "$CTX" | grep -q 'Actividad reciente'; then
        printf '[FAIL] caso2: additionalContext trae fallback PERO también un digest de actividad — BM_RECALL_UVX no se está honrando (se usó el uvx real, no el override roto)\n'
        FAIL=$((FAIL+1))
      else
        printf '[PASS] caso2: JSON válido, fallback presente, sin digest (BM_RECALL_UVX honrado)\n'
        PASS=$((PASS+1))
      fi
    else
      printf '[FAIL] caso2: additionalContext no contiene el texto de fallback\n'
      printf '       output: %s\n' "$CTX"
      FAIL=$((FAIL+1))
    fi
  fi

  if [ "$EC" -eq 0 ]; then
    printf '[PASS] caso2: exit code 0\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso2: exit code %d (esperaba 0)\n' "$EC"
    FAIL=$((FAIL+1))
  fi
}

# ---------------------------------------------------------------------------
# CASO 3: input source=="normal" (sin compact) → comportamiento actual intacto,
# no debe aparecer sección "Reglas reforzadas"
# ---------------------------------------------------------------------------
{
  INPUT='{"source":"startup","session_id":"test-sid-123"}'
  OUTPUT="$(printf '%s' "$INPUT" | BM_RECALL_UVX=/nonexistent bash "$HOOK" 2>/dev/null)"
  EC=$?

  if ! printf '%s' "$OUTPUT" | jq . >/dev/null 2>&1; then
    printf '[FAIL] caso3: stdout no es JSON válido\n'
    printf '       output: %s\n' "$OUTPUT"
    FAIL=$((FAIL+1))
  else
    CTX="$(printf '%s' "$OUTPUT" | jq -r '.hookSpecificOutput.additionalContext')"
    if printf '%s' "$CTX" | grep -q 'Reglas reforzadas tras compactacion'; then
      printf '[FAIL] caso3: apareció "Reglas reforzadas" en source non-compact (no debería)\n'
      FAIL=$((FAIL+1))
    else
      printf '[PASS] caso3: sin sección de reflejos en source=startup (correcto)\n'
      PASS=$((PASS+1))
    fi
  fi

  if [ "$EC" -eq 0 ]; then
    printf '[PASS] caso3: exit code 0\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso3: exit code %d (esperaba 0)\n' "$EC"
    FAIL=$((FAIL+1))
  fi
}

# ---------------------------------------------------------------------------
# CASO 4: input source=="compact" + log sintético con reflejos → debe aparecer
# sección "Reglas reforzadas" con las reglas mapeadas
# ---------------------------------------------------------------------------
{
  # Crea un log sintético en /tmp para este test
  TESTLOG="/tmp/test-reflex-$$.jsonl"
  printf '{"session_id":"test-sid-compact","reflex":"git-c","ts":1000}\n' > "$TESTLOG"
  printf '{"session_id":"test-sid-compact","reflex":"stuck-loop-pretool","ts":1001}\n' >> "$TESTLOG"
  printf '{"session_id":"other-sid","reflex":"git-c","ts":1002}\n' >> "$TESTLOG"

  INPUT='{"source":"compact","session_id":"test-sid-compact"}'
  OUTPUT="$(printf '%s' "$INPUT" | HOME=/tmp BM_RECALL_UVX=/nonexistent bash "$HOOK" 2>/dev/null <<< "$(cat "$TESTLOG" > /tmp/.claude/reflex-log.jsonl 2>/dev/null; :)" || printf '%s' "$INPUT" | HOME=/tmp BM_RECALL_UVX=/nonexistent bash "$HOOK" 2>/dev/null)"

  # Alternativa más simple: prepara el directorio HOME para el test
  mkdir -p /tmp/test-home-$$/.claude 2>/dev/null || true
  cat "$TESTLOG" > /tmp/test-home-$$/.claude/reflex-log.jsonl 2>/dev/null || true

  OUTPUT="$(printf '%s' "$INPUT" | HOME=/tmp/test-home-$$ BM_RECALL_UVX=/nonexistent bash "$HOOK" 2>/dev/null)"
  EC=$?

  if ! printf '%s' "$OUTPUT" | jq . >/dev/null 2>&1; then
    printf '[FAIL] caso4: stdout no es JSON válido\n'
    printf '       output: %s\n' "$OUTPUT"
    FAIL=$((FAIL+1))
  else
    CTX="$(printf '%s' "$OUTPUT" | jq -r '.hookSpecificOutput.additionalContext')"
    if printf '%s' "$CTX" | grep -q 'Reglas reforzadas tras compactacion'; then
      printf '[PASS] caso4: sección "Reglas reforzadas" presente\n'
      PASS=$((PASS+1))
    else
      printf '[FAIL] caso4: no encontró "Reglas reforzadas" en source=compact\n'
      FAIL=$((FAIL+1))
    fi

    if printf '%s' "$CTX" | grep -q 'git -C X'; then
      printf '[PASS] caso4: regla git-c mapeada correctamente\n'
      PASS=$((PASS+1))
    else
      printf '[FAIL] caso4: regla git-c no mapeada\n'
      FAIL=$((FAIL+1))
    fi

    if printf '%s' "$CTX" | grep -q 'recon-first'; then
      printf '[PASS] caso4: regla stuck-loop-pretool mapeada correctamente\n'
      PASS=$((PASS+1))
    else
      printf '[FAIL] caso4: regla stuck-loop-pretool no mapeada\n'
      FAIL=$((FAIL+1))
    fi
  fi

  if [ "$EC" -eq 0 ]; then
    printf '[PASS] caso4: exit code 0\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso4: exit code %d (esperaba 0)\n' "$EC"
    FAIL=$((FAIL+1))
  fi

  # Limpieza
  rm -rf "$TESTLOG" /tmp/test-home-$$ 2>/dev/null || true
}

# ---------------------------------------------------------------------------
# CASO 5: binario `basic-memory` directo en PATH (sin BM_RECALL_UVX) → el hook
# debe preferirlo sobre uvx. Fake binary con marker único; si el output trae el
# marker, el hook usó el binario directo.
# ---------------------------------------------------------------------------
{
  FAKEBIN="$(mktemp -d)"
  cat > "$FAKEBIN/basic-memory" <<'FAKE'
#!/usr/bin/env bash
[ -n "${FAKE_ARGLOG:-}" ] && echo "$*" >> "$FAKE_ARGLOG"
case "$*" in
  # printf '%s' con arg single-quoted: el \n queda como escape JSON literal
  # (valido); con \n en el FORMAT seria un newline real dentro del string
  # (JSON invalido y jq -re '.content' fallaria).
  *read-note*)       printf '%s' '{"content":"## Contrato de memoria\nMARKER-FAKE-BIN"}' ;;
  *recent-activity*) printf '%s' '[]' ;;
esac
exit 0
FAKE
  chmod +x "$FAKEBIN/basic-memory"

  OUTPUT="$(printf '{}' | PATH="$FAKEBIN:$PATH" bash "$HOOK" 2>/dev/null)"
  EC=$?

  CTX="$(printf '%s' "$OUTPUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"
  if printf '%s' "$CTX" | grep -q 'MARKER-FAKE-BIN'; then
    printf '[PASS] caso5: binario basic-memory directo preferido sobre uvx\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso5: el hook no usó el binario basic-memory del PATH (falta MARKER-FAKE-BIN)\n'
    printf '       output: %s\n' "$CTX"
    FAIL=$((FAIL+1))
  fi

  if [ "$EC" -eq 0 ]; then
    printf '[PASS] caso5: exit code 0\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso5: exit code %d (esperaba 0)\n' "$EC"
    FAIL=$((FAIL+1))
  fi

  rm -rf "$FAKEBIN"
}

# ---------------------------------------------------------------------------
# CASO 6: la llamada recent-activity debe pinear --project kb-demo (hoy
# depende del proyecto default de basic-memory — bug silencioso si cambia).
# Reutiliza un fake binary que loguea sus args.
# ---------------------------------------------------------------------------
{
  FAKEBIN="$(mktemp -d)"
  ARGLOG="$(mktemp)"
  cat > "$FAKEBIN/basic-memory" <<'FAKE'
#!/usr/bin/env bash
[ -n "${FAKE_ARGLOG:-}" ] && echo "$*" >> "$FAKE_ARGLOG"
case "$*" in
  *read-note*)       printf '%s' '{"content":"## Contrato de memoria\nMARKER-FAKE-BIN"}' ;;
  *recent-activity*) printf '%s' '[]' ;;
esac
exit 0
FAKE
  chmod +x "$FAKEBIN/basic-memory"

  printf '{}' | FAKE_ARGLOG="$ARGLOG" PATH="$FAKEBIN:$PATH" bash "$HOOK" >/dev/null 2>&1

  if grep -q 'recent-activity' "$ARGLOG" && grep 'recent-activity' "$ARGLOG" | grep -q -- '--project kb-demo'; then
    printf '[PASS] caso6: recent-activity lleva --project kb-demo\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso6: recent-activity sin --project kb-demo\n'
    printf '       args logueados: %s\n' "$(cat "$ARGLOG")"
    FAIL=$((FAIL+1))
  fi

  rm -rf "$FAKEBIN" "$ARGLOG"
}

# ---------------------------------------------------------------------------
# CASO 7: las dos llamadas corren en paralelo. Fake binary que duerme 2s y
# falla: secuencial ≥4s, paralelo ~2s. Umbral: ≤3s.
# ---------------------------------------------------------------------------
{
  SLOWBIN="$(mktemp -d)"
  cat > "$SLOWBIN/basic-memory" <<'FAKE'
#!/usr/bin/env bash
sleep 2
exit 1
FAKE
  chmod +x "$SLOWBIN/basic-memory"

  T0="$(date +%s)"
  OUTPUT="$(printf '{}' | PATH="$SLOWBIN:$PATH" bash "$HOOK" 2>/dev/null)"
  EC=$?
  T1="$(date +%s)"
  ELAPSED=$((T1-T0))

  if [ "$ELAPSED" -le 3 ]; then
    printf '[PASS] caso7: %ds de wall con dos llamadas de 2s (paralelo)\n' "$ELAPSED"
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso7: %ds de wall — las llamadas parecen secuenciales\n' "$ELAPSED"
    FAIL=$((FAIL+1))
  fi

  CTX="$(printf '%s' "$OUTPUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"
  if printf '%s' "$CTX" | grep -q 'memoria persistente es el MCP basic-memory'; then
    printf '[PASS] caso7: fallback presente tras fallo de ambas llamadas\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso7: sin fallback tras fallo de ambas llamadas\n'
    FAIL=$((FAIL+1))
  fi

  if [ "$EC" -eq 0 ]; then
    printf '[PASS] caso7: exit code 0\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso7: exit code %d (esperaba 0)\n' "$EC"
    FAIL=$((FAIL+1))
  fi

  rm -rf "$SLOWBIN"
}

# ---------------------------------------------------------------------------
# CASO 8: --cached con cache fresco válido → sirve el cache (no el binario),
# rápido (<=1s aunque el binario sea lento), y refresca en background.
# ---------------------------------------------------------------------------
{
  SLOWBIN="$(mktemp -d)"
  cat > "$SLOWBIN/basic-memory" <<'FAKE'
#!/usr/bin/env bash
sleep 2
case "$*" in
  *read-note*)       printf '%s' '{"content":"## Contrato de memoria\nMARKER-FAKE-BIN"}' ;;
  *recent-activity*) printf '%s' '[]' ;;
esac
exit 0
FAKE
  chmod +x "$SLOWBIN/basic-memory"
  CACHEFILE="$(mktemp -u)"
  printf '## Contrato de memoria (cache)\nMARKER-CACHE\n' > "$CACHEFILE"

  T0="$(date +%s)"
  OUTPUT="$(printf '{}' | BM_RECALL_CACHE_FILE="$CACHEFILE" PATH="$SLOWBIN:$PATH" bash "$HOOK" --cached 2>/dev/null)"
  T1="$(date +%s)"
  ELAPSED=$((T1-T0))

  CTX="$(printf '%s' "$OUTPUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"
  if printf '%s' "$CTX" | grep -q 'MARKER-CACHE' && ! printf '%s' "$CTX" | grep -q 'MARKER-FAKE-BIN'; then
    printf '[PASS] caso8: sirvió el cache, no el binario\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso8: no sirvió del cache (esperaba MARKER-CACHE sin MARKER-FAKE-BIN)\n'
    FAIL=$((FAIL+1))
  fi

  if [ "$ELAPSED" -le 1 ]; then
    printf '[PASS] caso8: servido en %ds (no bloqueó en el refresh)\n' "$ELAPSED"
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso8: tardó %ds sirviendo de cache (esperaba <=1s)\n' "$ELAPSED"
    FAIL=$((FAIL+1))
  fi

  # El refresh en background debe actualizar el cache (fake duerme 2s; poll hasta 8s)
  REFRESHED=0
  for _ in $(seq 1 16); do
    if grep -q 'MARKER-FAKE-BIN' "$CACHEFILE" 2>/dev/null; then REFRESHED=1; break; fi
    sleep 0.5
  done
  if [ "$REFRESHED" -eq 1 ]; then
    printf '[PASS] caso8: refresh en background actualizó el cache\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso8: el cache no se refrescó en background tras 8s\n'
    FAIL=$((FAIL+1))
  fi

  rm -rf "$SLOWBIN" "$CACHEFILE"
}

# ---------------------------------------------------------------------------
# CASO 9: --cached sin cache → fetch inline del binario, sirve su contenido
# y escribe el cache.
# ---------------------------------------------------------------------------
{
  FAKEBIN="$(mktemp -d)"
  cat > "$FAKEBIN/basic-memory" <<'FAKE'
#!/usr/bin/env bash
case "$*" in
  *read-note*)       printf '%s' '{"content":"## Contrato de memoria\nMARKER-FAKE-BIN"}' ;;
  *recent-activity*) printf '%s' '[]' ;;
esac
exit 0
FAKE
  chmod +x "$FAKEBIN/basic-memory"
  CACHEFILE="$(mktemp -u)"

  OUTPUT="$(printf '{}' | BM_RECALL_CACHE_FILE="$CACHEFILE" PATH="$FAKEBIN:$PATH" bash "$HOOK" --cached 2>/dev/null)"
  CTX="$(printf '%s' "$OUTPUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"

  if printf '%s' "$CTX" | grep -q 'MARKER-FAKE-BIN'; then
    printf '[PASS] caso9: miss → sirvió el fetch inline\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso9: miss no sirvió el contenido del binario\n'
    FAIL=$((FAIL+1))
  fi

  if grep -q 'MARKER-FAKE-BIN' "$CACHEFILE" 2>/dev/null; then
    printf '[PASS] caso9: el cache quedó escrito tras el miss\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso9: el cache no se escribió tras el miss\n'
    FAIL=$((FAIL+1))
  fi

  rm -rf "$FAKEBIN" "$CACHEFILE"
}

# ---------------------------------------------------------------------------
# CASO 10: --cached con cache STALE (mtime > TTL) → NO sirve el cache viejo;
# fetch inline y cache actualizado.
# ---------------------------------------------------------------------------
{
  FAKEBIN="$(mktemp -d)"
  cat > "$FAKEBIN/basic-memory" <<'FAKE'
#!/usr/bin/env bash
case "$*" in
  *read-note*)       printf '%s' '{"content":"## Contrato de memoria\nMARKER-FAKE-BIN"}' ;;
  *recent-activity*) printf '%s' '[]' ;;
esac
exit 0
FAKE
  chmod +x "$FAKEBIN/basic-memory"
  CACHEFILE="$(mktemp -u)"
  printf '## Contrato de memoria (cache)\nMARKER-CACHE\n' > "$CACHEFILE"
  touch -d '2 hours ago' "$CACHEFILE"

  OUTPUT="$(printf '{}' | BM_RECALL_CACHE_FILE="$CACHEFILE" PATH="$FAKEBIN:$PATH" bash "$HOOK" --cached 2>/dev/null)"
  CTX="$(printf '%s' "$OUTPUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"

  if printf '%s' "$CTX" | grep -q 'MARKER-FAKE-BIN' && ! printf '%s' "$CTX" | grep -q 'MARKER-CACHE'; then
    printf '[PASS] caso10: cache stale ignorado, fetch inline servido\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso10: sirvió el cache stale (o no sirvió el fetch)\n'
    FAIL=$((FAIL+1))
  fi

  if grep -q 'MARKER-FAKE-BIN' "$CACHEFILE" 2>/dev/null; then
    printf '[PASS] caso10: cache actualizado tras el miss por stale\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso10: cache no actualizado tras el miss por stale\n'
    FAIL=$((FAIL+1))
  fi

  rm -rf "$FAKEBIN" "$CACHEFILE"
}

# ---------------------------------------------------------------------------
# CASO 11: --cached con KB rota y sin cache → sirve el fallback y NO cachea
# el fallback.
# ---------------------------------------------------------------------------
{
  CACHEFILE="$(mktemp -u)"
  OUTPUT="$(printf '{}' | BM_RECALL_CACHE_FILE="$CACHEFILE" BM_RECALL_UVX=/nonexistent bash "$HOOK" --cached 2>/dev/null)"
  EC=$?
  CTX="$(printf '%s' "$OUTPUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"

  if printf '%s' "$CTX" | grep -q 'memoria persistente es el MCP basic-memory'; then
    printf '[PASS] caso11: fallback servido con KB rota\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso11: sin fallback con KB rota\n'
    FAIL=$((FAIL+1))
  fi

  if [ ! -f "$CACHEFILE" ]; then
    printf '[PASS] caso11: el fallback no se cacheó\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso11: se escribió cache con KB rota\n'
    FAIL=$((FAIL+1))
  fi

  if [ "$EC" -eq 0 ]; then
    printf '[PASS] caso11: exit code 0\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso11: exit code %d (esperaba 0)\n' "$EC"
    FAIL=$((FAIL+1))
  fi

  rm -f "$CACHEFILE"
}

# ---------------------------------------------------------------------------
# CASO 12: --cached con basic-memory que imprime contenido válido pero sale
# non-zero (p.ej. timeout matándolo a mitad) → NO se sirve ni se cachea el
# contenido truncado; se sirve el fallback y no queda cache.
# ---------------------------------------------------------------------------
{
  BADEXITBIN="$(mktemp -d)"
  cat > "$BADEXITBIN/basic-memory" <<'FAKE'
#!/usr/bin/env bash
case "$*" in
  *read-note*)       printf '%s' '{"content":"## Contrato de memoria\nMARKER-BADEXIT"}' ;;
  *recent-activity*) printf '%s' '[]' ;;
esac
exit 3
FAKE
  chmod +x "$BADEXITBIN/basic-memory"
  CACHEFILE="$(mktemp -u)"

  OUTPUT="$(printf '{}' | BM_RECALL_CACHE_FILE="$CACHEFILE" PATH="$BADEXITBIN:$PATH" bash "$HOOK" --cached 2>/dev/null)"
  EC=$?
  CTX="$(printf '%s' "$OUTPUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"

  if printf '%s' "$CTX" | grep -q 'memoria persistente es el MCP basic-memory' && ! printf '%s' "$CTX" | grep -q 'MARKER-BADEXIT'; then
    printf '[PASS] caso12: exit non-zero → fallback servido, no el contenido truncado\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso12: sirvió contenido con exit non-zero (esperaba fallback, no MARKER-BADEXIT)\n'
    FAIL=$((FAIL+1))
  fi

  if [ ! -f "$CACHEFILE" ]; then
    printf '[PASS] caso12: no se cacheó el contenido con exit non-zero\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso12: se escribió cache pese a exit non-zero del binario\n'
    FAIL=$((FAIL+1))
  fi

  if [ "$EC" -eq 0 ]; then
    printf '[PASS] caso12: exit code 0\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso12: exit code %d (esperaba 0)\n' "$EC"
    FAIL=$((FAIL+1))
  fi

  rm -rf "$BADEXITBIN" "$CACHEFILE"
}

# ---------------------------------------------------------------------------
# CASO 13: el refresh en background sobrevive al group-kill del harness.
# Claude Code mata el process group del hook al terminar; si el refresh muere
# con él, el cache queda degradado a miss-por-TTL (visto en vivo 2026-07-10).
# Simulación: hook en su propio process group (setsid externo), kill al grupo
# al salir el hook, y el cache debe actualizarse igualmente.
# ---------------------------------------------------------------------------
{
  SLOWBIN="$(mktemp -d)"
  cat > "$SLOWBIN/basic-memory" <<'FAKE'
#!/usr/bin/env bash
sleep 2
case "$*" in
  *read-note*)       printf '%s' '{"content":"## Contrato de memoria\nMARKER-REFRESH"}' ;;
  *recent-activity*) printf '%s' '[]' ;;
esac
exit 0
FAKE
  chmod +x "$SLOWBIN/basic-memory"
  CACHEFILE="$(mktemp -u)"
  printf '## Contrato de memoria (cache)\nMARKER-CACHE\n' > "$CACHEFILE"

  # Hook en un process group propio; al salir, matamos el grupo entero como el harness
  setsid bash -c "printf '{}' | BM_RECALL_CACHE_FILE='$CACHEFILE' PATH='$SLOWBIN:$PATH' bash '$HOOK' --cached >/dev/null 2>&1" &
  GPID=$!
  wait "$GPID" 2>/dev/null || true
  kill -TERM -- -"$GPID" 2>/dev/null || true

  SURVIVED=0
  for _ in $(seq 1 16); do
    if grep -q 'MARKER-REFRESH' "$CACHEFILE" 2>/dev/null; then SURVIVED=1; break; fi
    sleep 0.5
  done
  if [ "$SURVIVED" -eq 1 ]; then
    printf '[PASS] caso13: el refresh sobrevivió al group-kill y actualizó el cache\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso13: el group-kill mató el refresh (cache sin actualizar tras 8s)\n'
    FAIL=$((FAIL+1))
  fi

  rm -rf "$SLOWBIN" "$CACHEFILE"
}

# ---------------------------------------------------------------------------
# CASO 14: input source=="compact" con session_id → debe quedar loggeada una
# línea reflex=="compact" en REFLEX_LOG_FILE (Task 6 A1: alimenta a1-gate.sh
# para sesiones_con_compact).
# ---------------------------------------------------------------------------
{
  TESTHOME14="/tmp/test-home-compactlog-$$"
  mkdir -p "$TESTHOME14/.claude" 2>/dev/null || true
  : > "$TESTHOME14/.claude/reflex-log.jsonl"
  REFLEXLOG14="$(mktemp)"

  INPUT='{"source":"compact","session_id":"test-sid-compactlog"}'
  OUTPUT="$(printf '%s' "$INPUT" | HOME="$TESTHOME14" REFLEX_LOG_FILE="$REFLEXLOG14" BM_RECALL_UVX=/nonexistent bash "$HOOK" 2>/dev/null)"
  EC=$?

  if grep -q '"reflex":"compact"' "$REFLEXLOG14" 2>/dev/null; then
    printf '[PASS] caso14: línea reflex=="compact" logueada en REFLEX_LOG_FILE\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso14: no se encontró línea reflex=="compact" en REFLEX_LOG_FILE\n'
    printf '       contenido: %s\n' "$(cat "$REFLEXLOG14" 2>/dev/null)"
    FAIL=$((FAIL+1))
  fi

  if printf '%s' "$OUTPUT" | jq . >/dev/null 2>&1; then
    printf '[PASS] caso14: stdout sigue siendo JSON válido (never-break)\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso14: stdout no es JSON válido tras loggear compact\n'
    FAIL=$((FAIL+1))
  fi

  if [ "$EC" -eq 0 ]; then
    printf '[PASS] caso14: exit code 0\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso14: exit code %d (esperaba 0)\n' "$EC"
    FAIL=$((FAIL+1))
  fi

  rm -rf "$TESTHOME14" "$REFLEXLOG14"
}

# ---------------------------------------------------------------------------
# CASO 15 (F3.1, rama "empty"): CORE vacío (KB inalcanzable, BM_RECALL_UVX
# roto) → compose_base cae al FALLBACK y debe quedar logueada una línea
# reflex=="recall-fallback" con reason=empty en REFLEX_LOG_FILE.
# ---------------------------------------------------------------------------
{
  REFLEXLOG15="$(mktemp)"
  OUTPUT="$(printf '{}' | REFLEX_LOG_FILE="$REFLEXLOG15" BM_RECALL_UVX=/nonexistent bash "$HOOK" 2>/dev/null)"
  EC=$?

  if grep -q '"reflex":"recall-fallback"' "$REFLEXLOG15" 2>/dev/null \
     && grep '"reflex":"recall-fallback"' "$REFLEXLOG15" | grep -q 'reason=empty'; then
    printf '[PASS] caso15: recall-fallback reason=empty logueado (CORE vacío)\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso15: no se logueó recall-fallback reason=empty\n'
    printf '       log: %s\n' "$(cat "$REFLEXLOG15" 2>/dev/null)"
    FAIL=$((FAIL+1))
  fi

  if printf '%s' "$OUTPUT" | jq . >/dev/null 2>&1 && [ "$EC" -eq 0 ]; then
    printf '[PASS] caso15: JSON válido y exit 0 (el log no rompe el arranque)\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso15: JSON inválido o exit≠0 (ec=%d)\n' "$EC"
    FAIL=$((FAIL+1))
  fi

  rm -f "$REFLEXLOG15"
}

# ---------------------------------------------------------------------------
# CASO 16 (F3.1, rama "oversize"): CORE con "Contrato de memoria" pero de más
# de 6144 caracteres → compose_base cae al FALLBACK y debe quedar logueada
# reason=oversize con el tamaño real medido (dato para reaccionar).
# ---------------------------------------------------------------------------
{
  FAKEBIN="$(mktemp -d)"
  BIGFILE="$(mktemp)"
  BIGBODY="$(printf 'x%.0s' $(seq 1 7000))"
  jq -n --arg c "## Contrato de memoria
$BIGBODY" '{content:$c}' > "$BIGFILE"
  cat > "$FAKEBIN/basic-memory" <<FAKE
#!/usr/bin/env bash
case "\$*" in
  *read-note*)       cat "$BIGFILE" ;;
  *recent-activity*) printf '%s' '[]' ;;
esac
exit 0
FAKE
  chmod +x "$FAKEBIN/basic-memory"
  REFLEXLOG16="$(mktemp)"

  OUTPUT="$(printf '{}' | REFLEX_LOG_FILE="$REFLEXLOG16" PATH="$FAKEBIN:$PATH" bash "$HOOK" 2>/dev/null)"
  EC=$?

  LINEA16="$(grep '"reflex":"recall-fallback"' "$REFLEXLOG16" 2>/dev/null | grep 'reason=oversize')"
  if [ -n "$LINEA16" ]; then
    printf '[PASS] caso16: recall-fallback reason=oversize logueado\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso16: no se logueó recall-fallback reason=oversize\n'
    printf '       log: %s\n' "$(cat "$REFLEXLOG16" 2>/dev/null)"
    FAIL=$((FAIL+1))
  fi

  SIZE16="$(printf '%s' "$LINEA16" | jq -r '.payload' 2>/dev/null | grep -o 'size=[0-9]*' | cut -d= -f2)"
  if [ -n "$SIZE16" ] && [ "$SIZE16" -gt 6144 ]; then
    printf '[PASS] caso16: size=%d medido y > 6144 (el dato real, no solo el hecho)\n' "$SIZE16"
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso16: falta size= o no es > 6144 (size16=%s)\n' "$SIZE16"
    FAIL=$((FAIL+1))
  fi

  if printf '%s' "$OUTPUT" | jq . >/dev/null 2>&1 && [ "$EC" -eq 0 ]; then
    printf '[PASS] caso16: JSON válido y exit 0\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso16: JSON inválido o exit≠0 (ec=%d)\n' "$EC"
    FAIL=$((FAIL+1))
  fi

  rm -rf "$FAKEBIN" "$BIGFILE" "$REFLEXLOG16"
}

# ---------------------------------------------------------------------------
# CASO 17 (F3.1, rama "no-contract"): CORE no vacío, dentro del límite, pero
# sin la cadena "Contrato de memoria" → compose_base cae al FALLBACK y debe
# quedar logueada reason=no-contract.
# ---------------------------------------------------------------------------
{
  FAKEBIN="$(mktemp -d)"
  cat > "$FAKEBIN/basic-memory" <<'FAKE'
#!/usr/bin/env bash
case "$*" in
  *read-note*)       printf '%s' '{"content":"# Otra nota cualquiera\nsin el header esperado."}' ;;
  *recent-activity*) printf '%s' '[]' ;;
esac
exit 0
FAKE
  chmod +x "$FAKEBIN/basic-memory"
  REFLEXLOG17="$(mktemp)"

  OUTPUT="$(printf '{}' | REFLEX_LOG_FILE="$REFLEXLOG17" PATH="$FAKEBIN:$PATH" bash "$HOOK" 2>/dev/null)"
  EC=$?

  if grep -q '"reflex":"recall-fallback"' "$REFLEXLOG17" 2>/dev/null \
     && grep '"reflex":"recall-fallback"' "$REFLEXLOG17" | grep -q 'reason=no-contract'; then
    printf '[PASS] caso17: recall-fallback reason=no-contract logueado\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso17: no se logueó recall-fallback reason=no-contract\n'
    printf '       log: %s\n' "$(cat "$REFLEXLOG17" 2>/dev/null)"
    FAIL=$((FAIL+1))
  fi

  if printf '%s' "$OUTPUT" | jq . >/dev/null 2>&1 && [ "$EC" -eq 0 ]; then
    printf '[PASS] caso17: JSON válido y exit 0\n'
    PASS=$((PASS+1))
  else
    printf '[FAIL] caso17: JSON inválido o exit≠0 (ec=%d)\n' "$EC"
    FAIL=$((FAIL+1))
  fi

  rm -rf "$FAKEBIN" "$REFLEXLOG17"
}

echo ""
TOTAL=$((PASS+FAIL))
echo "=== Resultado: ${PASS}/${TOTAL} pasaron ==="
[ $FAIL -eq 0 ] && exit 0 || exit 1
