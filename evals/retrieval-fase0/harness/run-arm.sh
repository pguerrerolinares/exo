#!/usr/bin/env bash
# run-arm.sh <arm-name> <hf-model> <dims>
#
# Pipeline detached de un brazo candidato de embeddings (M0 Fase 0, Task 7).
# Se lanza con setsid+nohup para sobrevivir al idle del agente (aprendizaje T4):
#
#   setsid nohup bash harness/run-arm.sh <arm> <model> <dims> \
#       > /tmp/m0-arm.log 2>&1 < /dev/null &
#
# Pasos: (1) edita ~/.basic-memory/config.json (modelo+dims) de forma ATÓMICA
# (tmp + os.replace, nunca open("w") directo — resolución (a) del controller,
# mismo patrón que harness/replay.py usa ya para el threshold); (2) `basic-memory
# reindex` y espera a que termine; (3) smoke test de búsqueda vector (≥1
# resultado con score) — si falla, escribe ARM_FAILED y para; (4)
# harness/replay.py <arm> (ya es resumible); (5) ARM_DONE <arm>.
set -uo pipefail

ARM_NAME="${1:?uso: run-arm.sh <arm-name> <hf-model> <dims>}"
HF_MODEL="${2:?uso: run-arm.sh <arm-name> <hf-model> <dims>}"
DIMS="${3:?uso: run-arm.sh <arm-name> <hf-model> <dims>}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
CONFIG_PATH="$HOME/.basic-memory/config.json"

fail() {
    echo "ARM_FAILED ${ARM_NAME}: $1"
    exit 1
}

echo "[$(date -Iseconds)] arranca brazo ${ARM_NAME} (${HF_MODEL}, dims=${DIMS})"

# Paso 1/4: config -> modelo/dims del brazo, escritura ATÓMICA.
echo "[paso 1/4] editando config.json -> ${HF_MODEL} (${DIMS})"
python3 - "$CONFIG_PATH" "$HF_MODEL" "$DIMS" <<'EOF' || fail "config-edit"
import json, os, sys, pathlib
config_path, model, dims = sys.argv[1], sys.argv[2], int(sys.argv[3])
p = pathlib.Path(config_path)
c = json.loads(p.read_text())
c["semantic_embedding_model"] = model
c["semantic_embedding_dimensions"] = dims
tmp = p.with_name(p.name + ".tmp")
tmp.write_text(json.dumps(c, indent=2) + "\n")
os.replace(tmp, p)
print(c["semantic_embedding_model"], c["semantic_embedding_dimensions"])
EOF

# Paso 2/4: re-index. El chunk-tracking de basic-memory guarda el modelo de
# embedding por chunk y detecta el cambio (compara contra la config vigente),
# así que el `reindex` incremental por defecto ya re-embeddea todo lo
# afectado sin necesitar --full; la tabla vectorial se recrea sola al
# detectar el mismatch de dimensiones (verificado en fuente:
# sqlite_search_repository._ensure_vector_tables). Primera vez descarga el
# modelo (jina ~0.64 GB) — puede tardar.
echo "[paso 2/4] basic-memory reindex (puede tardar; descarga de modelo la 1a vez)"
basic-memory reindex 2>&1 || fail "reindex"

# Paso 3/4: smoke test — búsqueda vector debe devolver >=1 resultado con score.
echo "[paso 3/4] smoke test: búsqueda vector"
SMOKE_JSON="$(basic-memory tool search-notes --project kb-demo --vector --page-size 3 "doctrina de agentes" 2>&1)"
SMOKE_RC=$?
if [ $SMOKE_RC -ne 0 ]; then
    fail "smoke search-notes exit ${SMOKE_RC}: ${SMOKE_JSON:0:300}"
fi
SMOKE_COUNT="$(printf '%s' "$SMOKE_JSON" | python3 -c '
import json, sys
try:
    data = json.load(sys.stdin)
except Exception:
    print(0)
    sys.exit(0)
results = data.get("results", data if isinstance(data, list) else [])
print(sum(1 for r in results if r.get("score") is not None))
')"
if ! [ "${SMOKE_COUNT:-0}" -ge 1 ] 2>/dev/null; then
    fail "smoke vector: 0 resultados con score (raw: ${SMOKE_JSON:0:300})"
fi
echo "smoke OK: ${SMOKE_COUNT} resultado(s) con score"

# Paso 4/4: replay + escribe results/<arm>.jsonl (resumible: reintentar este
# script tras un crash retoma el jsonl parcial).
echo "[paso 4/4] replay.py ${ARM_NAME}"
python3 "$BASE_DIR/harness/replay.py" "$ARM_NAME" || fail "replay"

echo "ARM_DONE ${ARM_NAME}"
