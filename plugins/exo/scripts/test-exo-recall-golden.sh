#!/usr/bin/env bash
# Golden de equivalencia para exo-recall.sh (campaña I, Task 3).
# RECAPTURA=1 (re)escribe los goldens; sin ella, compara y FALLA si faltan o
# difieren.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/exo-recall.sh"
GOLD_DIR="${SCRIPT_DIR}/testdata/golden-exo-recall"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
export HOME="$TMP/home"
mkdir -p "$HOME/.claude"
LOGC="$HOME/.claude/reflex-log.jsonl"
export REFLEX_LOG_FILE="$LOGC"

PASS=0; FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

run_hook() {  # $1=input_json  resto=env
  local input="$1"; shift
  printf '%s' "$input" | env "$@" "$HOOK" 2>/dev/null
}

# `tr -d '\r'` en AMBOS lados (review adversarial 2026-09-19): jq en
# Windows/Git Bash emite CRLF (mismo hallazgo que `scripts/test-hooks-json.sh:14-16`).
# Sin esto, un golden capturado en Linux nunca compararía en verde contra la
# salida real de windows-latest aunque el contenido sea idéntico.
guarda_o_compara() {  # $1=nombre $2=salida
  local nombre="$1" out golden="$GOLD_DIR/$1.txt"
  out="$(printf '%s' "$2" | tr -d '\r')"
  if [ "${RECAPTURA:-0}" = "1" ]; then
    mkdir -p "$GOLD_DIR"
    printf '%s' "$out" > "$golden"
    pass "golden: $nombre capturado en $golden"
    return
  fi
  if [ ! -f "$golden" ]; then
    fail "golden: $nombre" "no existe $golden — corre con RECAPTURA=1 antes de comparar"
    return
  fi
  if [ "$out" = "$(tr -d '\r' < "$golden")" ]; then
    pass "golden: $nombre sin cambios"
  else
    fail "golden: $nombre" "difiere de $golden"
  fi
}

# --- Escenario 1: camino feliz (STUB_FELIZ de test-exo-recall.sh) ---
STUB_FELIZ="$TMP/exo-stub-feliz"
cat > "$STUB_FELIZ" <<'EOF'
#!/usr/bin/env bash
case "$1" in
  --version) echo "exo 9.0.0" ;;
  config) echo '{"schema_version":2,"command":"config","data":{"kb":{"name":"kb-test","path":"/tmp/kb-test"}}}' ;;
  recall) echo "Contrato de memoria: bloque de prueba camino feliz." ;;
  *) exit 1 ;;
esac
EOF
chmod +x "$STUB_FELIZ"
touch "$TMP/index-feliz.db"
: > "$LOGC"
OUT1="$(run_hook '{"session_id":"sess-ok"}' EXO_BIN="$STUB_FELIZ" EXO_INDEX="$TMP/index-feliz.db" ENGINE_MIN=0.1.0)"
guarda_o_compara camino-feliz "$OUT1"

# --- Escenario 2: reafirmación tras compact (ejercita SOURCE/SID, lo que ---
# --- toca esta task) — mismo fixture de log que test-exo-recall.sh:25-39 ---
{
  printf '{"ts":"t","reflex":"git-c","session_id":"sess-x","agent_id":"","agent_type":"","tool":"Bash","payload":"viejo"}\n'
  awk 'BEGIN { for (i = 0; i < 2498; i++) printf "{\"ts\":\"t\",\"reflex\":\"zero-residuo\",\"session_id\":\"otra-%d\",\"agent_id\":\"\",\"agent_type\":\"\",\"tool\":\"Bash\",\"payload\":\"x\"}\n", i }'
  printf '{"ts":"t","reflex":"verify-before-commit","session_id":"sess-x","agent_id":"","agent_type":"","tool":"Bash","payload":"nuevo"}\n'
} > "$LOGC"
OUT2="$(run_hook '{"session_id":"sess-x","source":"compact"}' EXO_BIN="$TMP/no-existe")"
guarda_o_compara reafirma-compact "$OUT2"

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
