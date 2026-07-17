# Verdict del gate — m2-07-impl (implementación de la fusión hybrid + sweep de calibración)

- **Veredicto: MERGED**
- **Adjudica**: consultor Fable delegado (dispatch fresco: sin participación en la implementación, ni en su review, ni en la spec), régimen de gates de `.superpowers/fabrica/config.md` §"Ejecución de gates". Este gate juzga el MERGE de la implementación; NO es el gate oficial M2-09 (mismo-día, pineado), que sigue pendiente.
- **Fecha**: 2026-07-18 (madrugada)
- **Rama juzgada**: `m2-07-impl` (HEAD `ee839ac`, 2 commits sobre `main`@`8299add`): 44 ficheros, +2414/−25. Código: `engine/src/buscador.rs` (+209), `engine/src/main.rs` (+52/−…), `engine/tests/buscador.rs` (+79); harness: `replay-engine.py` extendido + 2 scripts nuevos; el resto son `results/*.jsonl`/`metrics-*.md` de las corridas del sweep + el reporte.
- **Contrato**: spec sellada `docs/superpowers/specs/2026-07-17-fusion-design.md` (§4 diseño, §5 sweep, §7 tests), mergeada con gate propio (`gate-m2-07-spec.md`). La implementación se juzga contra ella, no contra gusto propio.

## Verificación primaria propia (re-corrida, no tomada del reporte ni de la review)

### 1. Tests — `cargo test --manifest-path engine/Cargo.toml`

**72 passed, 0 failed, 1 ignored** (el ignored es `jina_es_embebe_a_768`, smoke opcional preexistente). Desglose: lib 26 (incluye `mod tests_fusion`), tests/buscador.rs 13 (10 preexistentes M2-05/06 + 3 nuevos DB), git_epoch 2, indexer 16, nota 7, schema 2, smoke 2, walker 4. **Los 11 tests contractuales de §7 presentes con sus nombres exactos** (8 puros en `tests_fusion`, 3 de integración: `fusion_gate_fts_no_pierde_hit_semantico`, `threshold_filtra_vector_pre_fusion`, `busqueda_hybrid_envelope`). Ningún test preexistente removido: M2-01..06 intactos.

### 2. Corrección de la fusión (`engine/src/buscador.rs`, leída línea a línea)

- `normaliza_fts` (:233) — D-f1 exacta: `f = β·f_raw/f_max`, `f_max` por fold de max; `f_max == 0` o lista vacía → mapa vacío (canal descartado, sin división por 0). Monótona, top-1 = β.
- `fusiona` (:255) — D-f2 exacta: unión de claves (`HashSet` sobre `keys().chain()`), canal ausente = 0 (`unwrap_or(&0.0)`), `score = v.max(f) + bonus * v.min(f)` (:272), `type: "entity"` hardcodeado, orden desc por `partial_cmp`, `truncate(limite)` DESPUÉS de fusionar.
- `busca_hybrid` (:293) — `K_C = 50` constante (§4.2); vector exhaustivo (`usize::MAX`) con `min_similitud` reenviado a `busca_vector` = threshold PRE-fusión sobre `v` (D-f3), misma precedencia flags>config que el arm vector; envelope `search_type: "hybrid"` literal.

Fiel a §4 en los cinco componentes. Sin lógica copiada de basic-memory: `fusiona` son ~25 líneas derivables de la fórmula de la spec; grep de internals de bm sobre el diff: nada.

### 3. Re-derivación del sweep — scorer INDEPENDIENTE propio

Script propio (`rescore.py`, fuera del worktree, replica norm/hit de `analyze.py` sin ejecutarlo — `results/` no se regeneró, worktree limpio al final). Sobre los jsonl crudos commiteados:

| Afirmación (reporte §3) | Re-derivado por mí | ¿Cuadra? |
|---|---|---|
| Grid 15 celdas thr=None | b{0.0..0.3}-e0.6 = **49/55**; b0.5-e0.6 = 48; e0.8/e1.0 ∈ {47,48} — celda a celda idéntico a la tabla §3.1 | ✓ |
| Celda ganadora por threshold | b0.0-e0.6: None/0.35/**0.40** = 49; 0.45 = 46; 0.50 = 41 → mayor thr que sostiene el máximo = **0.40** | ✓ |
| Desempate §5.2.4 paso 2 | las 4 celdas de 49 (b0.0/0.1/0.2/0.3 × e0.6) TODAS con ARREGLA 7 / ROMPE 1 vs jina-es — el paso no desempata; decide "menor bonus" → 0.0 | ✓ |
| **Confirmación nativa** (`engine-hybrid-nativo-final`, --min-similitud 0.40) | **49/55**, idéntico al post-hoc | ✓ |
| **Pareada nativa vs jina-es** | **ARREGLA 7 / ROMPE 1**; la rota es `esa utilidad de terminal…` (both-miss: ni FTS ni vector puro la aciertan). La FTS-HIT que el vector puro rompía (`lighthouses bot amortización…`, spec §3.2) ya NO está entre las rotas — la fusión la rescató, que era su razón de existir | ✓ |
| Diagnóstica lectura A | re-computada desde `engine-hybrid-b0.2-e0.8` ∩ pool `engine-fts-k50-m207`, top-5: **30/55** ∈ predicción pre-registrada [28, 41] | ✓ |
| Sanity §6 | 49 ≥ 46 (floor spec) y ≥ **47** (vector puro mismo-día `engine-vector-m207`, re-derivado) | ✓ |
| Referencias | jina-es 43/55; engine-fts-m207 29/55 (drift +1 vs los 28 de julio, consistente con KB crecida — irrelevante: el gate oficial es mismo-día M2-09) | ✓ |

La selección (bonus=0.0, β=0.6, thr=0.40) sale del procedimiento pre-registrado §5.2.4 aplicado mecánicamente — incluido el resultado incómodo bonus=0.0 (fusión = max puro en este eval set), reportado sin maquillar. Cero renegociación detectada.

### 4. Sellado (`engine/src/main.rs`)

`BONUS_SELLADO = 0.0` (:23) y `ESCALA_FTS_SELLADA = 0.6` (:24) como fallback de `--bonus`/`--escala-fts` (:132-133) — reemplazan los provisionales 0.2/0.8. El **0.40 NO está hardcodeado**: grep de `0.4` en main.rs solo lo encuentra en doc-comments; `min_similitud` sigue siendo `Option` con caída a config RO (D-f3/§4.6, config propia = M5a). Correcto según spec, con la consecuencia documentada en el reporte: `--type hybrid` sin flags usa threshold config (0.35), el punto sellado exige `--min-similitud 0.40` explícito.

### 5. Superficies gateadas — ∅ verificado

`git diff main..m2-07-impl -- analyze.py envelope.rs schema.rs gate.md replay.py` = **0 líneas** (los cinco). Los 2 scripts nuevos (`atribucion-cruzada.py`, `diagnostica-lectura-a.py`) hacen `from analyze import BASE, K, hit, load, norm` — importan, no modifican; no añaden code-path al binario (la diagnóstica A es post-procesado, como exige B4). El diff de `replay-engine.py` es solo reenvío condicional de 3 flags + docstring. Clean-room AGPL limpio.

## Qué busqué para objetar (mandato de disenso)

1. **¿El pass-through del threshold funciona de verdad, o solo lo parece?** (el test 9 no lo probaría — ver nits). Verifiqué empíricamente: el top-5 de `cliente-c` DIFIERE entre la corrida nativa (0.40) y la sin-threshold — el filtro pre-fusión tiene efecto observable en datos reales. Funciona.
2. **¿El desempate no-determinista de `fusiona` contaminó los números sellados?** Busqué scores duplicados en las corridas commiteadas: existen (2/56 queries en b0.0-e0.6), pero en posiciones 2-3 y 7-9 — **ninguno cruza la frontera del top-5**, así que el 49/55 y la pareada son deterministas para estos datos. No contaminó.
3. **¿La selección "mayor thr = 0.40" es renegociación?** No: es la lectura natural del criterio pre-registrado ("mayor thr" sobre el grid `{None, 0.35…0.65}` restringido a las celdas que sostienen el máximo).
4. **¿La pareada del sweep contra `jina-es.jsonl` de julio viola "prohibido comparar contra results/ de julio"?** No: esa prohibición es del gate oficial (spec M2 §5); aquí la pareada es diagnóstico de selección (§5.2.4), rol declarado explícitamente en el reporte. El gate oficial sigue siendo M2-09 mismo-día.
5. **¿El test 5 movido a unit puro reabre diseño?** No: partición del brief con un hueco; el test no necesita DB y verifica lo mismo.
6. **¿`usize::MAX` como límite del canal vector es una bomba?** Empíricamente no (56 queries corridas sin error, elapsed ~1s); el KNN ya es exhaustivo por diseño m2-06.
7. **¿Drift de KB (trozos 2213 vs 2194 de m2-06) invalida algo?** No: declarado en el reporte, y ninguna comparación absoluta contra julio se usa como gate.

## Postura sobre los 2 nits de la review (opinión propia, no deferida)

- **Nit 1 — test 9 asevera presencia, no `score == f`**: REAL, y de hecho algo más débil de lo que dice el enunciado: tal como está escrito, tampoco detectaría una implementación que ignore `min_similitud` por completo (con candidatos vector admitidos, el candidato FTS seguiría presente y el `any()` pasaría). **No-bloqueante** por tres razones: (a) el mecanismo es un reenvío de un argumento a `busca_vector`, que ya tiene sus propios tests de threshold en M2-06; (b) lo verifiqué empíricamente en datos reales (punto 1 de arriba); (c) un filtro post-fusión erróneo con umbral 1.5 SÍ haría fallar el test (vaciaría los resultados), así que la asersión discrimina el error más plausible. Recomendación de follow-up: endurecer a "cero candidatos solo-vector presentes y `score == f` exacto".
- **Nit 2 — desempate no-determinista en `fusiona`** (empates quedan en orden de iteración del `HashSet`): REAL — hay empates exactos en datos reales de esta KB (2/56 queries). Hoy no cruzan el top-5 (verificado, punto 2), así que ningún número sellado depende del azar. **No-bloqueante para este merge**, pero con recomendación CONCRETA: añadir clave secundaria determinista (permalink asc) **antes de la corrida oficial M2-09** — un empate que cruce la frontera del top-5 en esa corrida haría el número del gate no-reproducible, y ese régimen ("los números no se renegocian") no tolera flakiness. Es un cambio de una línea que no altera ninguna semántica sellada.

Ninguno de los dos invalida un número sellado ni viola la spec; ambos son endurecimientos de robustez. Coincido con la review en el carácter no-bloqueante, por verificación propia.

## Citas

- Spec fusión `docs/superpowers/specs/2026-07-17-fusion-design.md` §4.3 (D-f1), §4.4-4.5 (D-f2), §4.6 (D-f3), §5.1-5.2 (grid, selección pre-registrada, confirmación nativa obligatoria), §6 (gate oficial ≠ sanity 46), §7 (11 tests).
- `evals/e1-read/gate.md` — congelado, 0 diff, NO adjudicado aquí (es de M2-09).
- Reporte `reports/m2-07-impl-report.md` — cotejado contra mis re-derivaciones, sin discrepancias.
- Código: `engine/src/buscador.rs:233,255,293,307`; `engine/src/main.rs:23-24,127-133`.
- Datos: `evals/retrieval-fase0/results/engine-hybrid-*.jsonl`, `engine-fts{,-k50}-m207.jsonl`, `engine-vector-m207.jsonl`, `jina-es.jsonl`, `eval.jsonl`.
