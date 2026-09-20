#!/usr/bin/env bash
# Golden de equivalencia para recall-inject.sh (campaña I, Task 2): congela
# la salida COMPLETA (el bloque inyectado, byte a byte) de seis escenarios ya
# existentes en test-recall-inject.sh, para que ninguna task que reescriba el
# parseo (jq/sed/tr -> bash) cambie ni un byte de lo que se inyecta.
#
# LIMITACIÓN CONOCIDA: los seis fixtures (CUATRO/GORDO/RECORTE/TITREP/UNICO/
# AVISA) cubren la COMPOSICIÓN del bloque con hits reales -- cero hits (gate
# que calla, DB/binario ausente, "recall vacío") y envelope inválido no están
# entre ellos, a propósito: el plan de la Task 2 nombra estos seis fixtures
# uno por uno y añadir otros se saldría de su contrato. Por eso este golden
# NO es por sí solo una red de regresión completa -- esos otros casos (P1,
# P2, P4, P5, el gate léxico, `norm_token`) solo los cubre
# `test-recall-inject.sh`, y este script se corre SIEMPRE junto a esa suite,
# nunca en su lugar.
#
# RECAPTURA=1 (re)escribe los goldens. Sin ella, compara y FALLA si faltan o
# difieren -- nunca aprueba en silencio un golden ausente.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/recall-inject.sh"
GOLD_DIR="${SCRIPT_DIR}/testdata/golden-recall-inject"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
export REFLEX_LOG_FILE="$TMP/reflex-log.jsonl"
export EXO_INJECT_CAP="${EXO_INJECT_CAP:-1024}"
export EXO_KB_NAME="kb-demo"
FAKE_DB="$TMP/index.db"; : > "$FAKE_DB"; export EXO_INDEX="$FAKE_DB"

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

run_hook() {  # $1=prompt $2=exo_bin
  printf '%s' "$1" | jq -Rs '{prompt:., session_id:"golden-sess"}' \
    | EXO_BIN="$2" "$HOOK" 2>/dev/null
}

# --- Fixtures: copiadas literalmente de test-recall-inject.sh (misma fuente) ---
CUATRO="$TMP/exo-cuatro"
cat > "$CUATRO" <<'PYEOF'
#!/usr/bin/env bash
cat <<'JSON'
{"command":"recall","data":{"cap_bytes":1400,"mode":"consulta","notes":[
{"permalink":"kb-demo/core/core-index","path":"/kb/core/core-index.md","score":0.6,"snippet":"mapa de memoria","tier":null,"title":"core-index"},
{"permalink":"kb-demo/log/kbx-bitacora","path":"/kb/log/kbx-bitacora.md","score":0.5,"snippet":"bitacora de kbx","tier":null,"title":"kbx-bitacora"},
{"permalink":"kb-demo/projects/kbx","path":"/kb/projects/kbx.md","score":0.47,"snippet":"destilado de kbx","tier":null,"title":"kbx"},
{"permalink":"kb-demo/log/exo-bitacora","path":"/kb/log/exo-bitacora.md","score":0.44,"snippet":"bitacora de exo","tier":null,"title":"exo-bitacora"}
],"query":"kbx","truncated":false},"schema_version":2}
JSON
PYEOF
chmod +x "$CUATRO"

GORDO="$TMP/exo-gordo"
jq -n '{data:{notes:[range(1;4) as $i | {
  permalink:("kb-demo/log/n"+($i|tostring)),
  path:("/kb/log/nota-larga-numero-"+($i|tostring)+".md"),
  score:0.5, tier:null,
  title:("nota larga numero "+($i|tostring)),
  snippet:(("palabra "*25)+"fin")}]}}' > "$TMP/gordo.json"
printf '#!/usr/bin/env bash\ncat "%s"\n' "$TMP/gordo.json" > "$GORDO"
chmod +x "$GORDO"

RECORTE="$TMP/exo-recorte"
jq -n '{data:{notes:[range(1;4) as $i | {
  permalink:("kb-demo/log/r"+($i|tostring)),
  path:("/kb/log/recorte-"+($i|tostring)+".md"), score:0.5, tier:null,
  title:("recorte "+($i|tostring)),
  snippet:(("análisis técnico — decisión sellada según medición práctica; "*8))}]}}' \
  > "$TMP/recorte.json"
printf '#!/usr/bin/env bash\ncat "%s"\n' "$TMP/recorte.json" > "$RECORTE"
chmod +x "$RECORTE"

TITREP="$TMP/exo-titrep"
jq -n '{data:{notes:[
 {permalink:"kb-demo/log/kbx-bitacora",path:"/kb/log/kbx-bitacora.md",score:0.5,tier:null,
  title:"kbx-bitacora",snippet:"# kbx-bitacora  cuerpo real de la bitacora"},
 {permalink:"kb-demo/log/otra",path:"/kb/log/otra.md",score:0.4,tier:null,
  title:"Un título que sí aporta",snippet:"cuerpo de la otra"}]}}' > "$TMP/titrep.json"
printf '#!/usr/bin/env bash\ncat "%s"\n' "$TMP/titrep.json" > "$TITREP"
chmod +x "$TITREP"

UNICO="$TMP/exo-unico"
jq -n '{data:{notes:[{permalink:"kb-demo/log/solo",path:"/kb/log/solo.md",score:0.5,
  tier:null,title:"nota solitaria",snippet:"cuerpo de la unica nota"}]}}' > "$TMP/unico.json"
printf '#!/usr/bin/env bash\ncat "%s"\n' "$TMP/unico.json" > "$UNICO"
chmod +x "$UNICO"

AVISA="$TMP/exo-avisa"
cat > "$AVISA" <<'EOF'
#!/usr/bin/env bash
echo "aviso: arm vector INERTE: 0 vectores para 9 trozos." >&2
cat <<'JSON'
{"command":"recall","data":{"cap_bytes":4000,"mode":"consulta","elapsed_s":0.9876,"refresh_s":0.0123,"warnings":["arm vector INERTE: 0 vectores para 9 trozos."],"notes":[
{"permalink":"kb-demo/log/kbx-bitacora","path":"/kb/log/kbx-bitacora.md","score":0.5,"snippet":"bitacora de kbx","tier":null,"title":"kbx-bitacora"}
],"query":"kbx","truncated":false},"schema_version":2}
JSON
EOF
chmod +x "$AVISA"

# --- Comparación: cada escenario emite EXACTAMENTE el stdout completo del hook ---
# `tr -d '\r'` en AMBOS lados (review adversarial 2026-09-19): jq en
# Windows/Git Bash emite CRLF (mismo hallazgo ya documentado en
# `scripts/test-hooks-json.sh:14-16` — "jq en Windows/Git Bash emite CRLF:
# cada salida que se compara o se lee línea a línea pasa por tr -d '\r'").
# Sin esto, el golden capturado en Linux (sin \r) nunca compararía en verde
# contra la salida real de windows-latest (con \r) aunque el CONTENIDO sea
# idéntico -- un falso rojo que no dice nada sobre el bloque inyectado.
comprueba() {  # $1=nombre $2=prompt $3=bin
  local nombre="$1" out golden="$GOLD_DIR/$1.txt"
  out="$(run_hook "$2" "$3" | tr -d '\r')"
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

comprueba cuatro-hits "kbx trinquete" "$CUATRO"
comprueba gordo-cap-exacto "kbx trinquete" "$GORDO"
comprueba recorte-snippet "kbx trinquete" "$RECORTE"
comprueba titulo-repetido "kbx trinquete" "$TITREP"
comprueba unico-hit "kbx trinquete" "$UNICO"
comprueba con-avisos "kbx trinquete" "$AVISA"

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
