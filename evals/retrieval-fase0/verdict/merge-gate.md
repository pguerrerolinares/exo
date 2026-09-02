# Merge gate — rama m0-fase0 → master (adjudicación Fable, régimen §8)

- **Fecha**: 2026-07-17
- **Adjudicador**: consultor Fable de merge (régimen §8; consultor fresco, sin participación en tareas de la campaña; verificación primaria ejecutada, no delegada).
- **Objeto**: rama `m0-fase0` (11 commits, `0a403ad..2a8c034`) para merge a `master`. El verdict experimental (51e104b) está fuera de alcance por mandato — este gate adjudica solo la calidad de la rama.

## GATE: APROBADA

Fast-forward puro (merge-base de master y m0-fase0 == master == `0a403ad`; sin conflictos posibles). Los tres Minors del ledger van como deuda documentada, ninguno bloquea (triage abajo).

## Verificación primaria ejecutada

1. **Privacidad del log crudo**: `git log --all --oneline -- evals/retrieval-fase0/snapshot/reflex-retrieval-log.jsonl` → **vacío**: el snapshot crudo jamás entró en la historia, en ninguna rama. `git check-ignore -v` confirma que `.gitignore:3` lo cubre; el fichero existe solo en el working tree local (74 KB).
2. **Contenido trackeado sin datos sensibles**: muestreados `queries.jsonl` (56 filas) y `eval.jsonl` (56 filas + notes) completos — solo queries de trabajo (proyectos, bitácoras, conceptos técnicos). Los 4 `results/*.jsonl` (168 filas c/u, verificado por conteo; 0 filas de error en los 4) contienen solo `permalink/type/score`, sin contenido de notas. Scan de secretos sobre el árbol commiteado: único hit `snapshot/config-baseline.json` con `cloud_api_key: null` y un client_id público de OAuth — no es secreto. Cero emails.
3. **Sin residuo en el árbol**: `git ls-tree -r 2a8c034` no contiene temporales, logs, `.tmp`, sidecar ni pycache. Fuera de `evals/` y `docs/` solo cambia `.gitignore` (+5 líneas, correctas). El sidecar `.min_similarity_backup.json` no existe en disco (restauración de replay.py funcionó). Único residuo: `harness/__pycache__/` **untracked** en el working tree — no entra al merge; housekeeping opcional añadir `__pycache__/` al .gitignore.
4. **Audit trail íntegro y trazable**: orden de commits correcto — baseline medido (8d84cd6) → gate sellado (23865e7) → brazos (e0899ce) → fix de medición (c261b46) → verdict (51e104b) → aplicación (2a8c034). `git log --follow -- gate.md` muestra **un solo commit** (23865e7): el gate no se tocó después de sellado. Los números del verdict (43/55, 41/55, 7/0, 7/2, 26/55, 18/55) cuadran con `metrics-*.md`, que cuadran con los jsonl.
5. **Reproducibilidad**: re-corrida de `analyze.py baseline` y `analyze.py minilm baseline` → los metrics regenerados son **byte-idénticos** a los commiteados (git diff vacío; working tree restaurado).
6. **Fix de medición T7 verificado en datos**: en `textfts.jsonl` las filas `search_type=text` tienen scores BM25 negativos y ranking distinto de hybrid — el control FTS es real, ya no "hybrid disfrazado".
7. **Estado final documentado == realidad**: `~/.basic-memory/config.json` vigente tiene `semantic_embedding_model=jinaai/jina-embeddings-v2-base-es`, `semantic_embedding_dimensions=768`, `semantic_min_similarity=0.35`, `default_search_type=null` — exactamente lo que firma la Decisión 4 del verdict y el commit 2a8c034 (incluida la actualización de spec §10: lenguaje FIRMADO Rust con cita al verdict).

## Triage de los 3 Minors del ledger → los tres como DEUDA DOCUMENTADA (post-merge)

1. **Sidecar write no atómico** (`replay.py:459`, `MIN_SIMILARITY_BACKUP.write_text`): un crash a mitad de escritura deja sidecar corrupto, pero el orden sidecar-antes-de-config garantiza que config.json sigue intacto en ese instante, y el resume falla RUIDOSO (JSONDecodeError) en vez de restaurar un valor malo. Código de harness de eval, no de producción. No bloquea.
2. **`norm()` docstring overclaims** (`analyze.py`): cosmético; la versión actual ya documenta la verificación manual de Task 5 y el invariante de punto-único-de-normalización. No bloquea.
3. **`observation_hit` impreso como lista, no campo estructurado** (`analyze.py` → metrics md): cosmético; además la métrica fue 0 en todos los brazos, así que no hay dato perdido. No bloquea.

Ninguno afecta a la validez de los datos commiteados ni al estado de producción; si el harness se reutiliza para el eval set permanente (§4.1), arreglar 1 y 3 entonces.
