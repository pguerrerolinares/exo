# Gate pre-registrado — M0 Fase 0 (fijado antes de cambiar config)

Métrica primaria: hit@5 hybrid sobre queries etiquetadas, comparación PAREADA vs baseline,
cada brazo en su mejor threshold del sweep (mismo procedimiento para todos los brazos).

1. GANA un candidato si: arregla ≥5 queries y rompe ≤1 (pareada).
2. SEMÁNTICA LOCAL LOAD-BEARING si: con el mejor brazo, ≥3 queries etiquetadas
   tienen HIT en vector u hybrid y MISS en text (la semántica aporta lo que FTS no puede).
   → decide lenguaje del engine (spec §4.5): load-bearing ⇒ Rust; si no ⇒ Go.
3. URGENCIA DEL ENGINE: si ningún candidato GANA y la atribución muestra misses
   mayoritariamente fusion-miss/threshold (no del modelo), el fix pertenece al motor
   propio y M2 sube de prioridad; si un candidato GANA, M2 baja a "estrangulamiento
   tranquilo" (config-fix aplicado, dolor mitigado).
4. Empate o resultado ambiguo ⇒ el consultor Fable adjudica con este texto delante;
   no se re-negocian los números post-hoc.

## Ajustes de método pre-registrados (antes de los brazos candidatos)

- **Estratificación observation-hits**: el CLI de basic-memory agrega los resultados
  a entidad (verificado: 1680/1680 items del baseline son type=entity, mientras el
  índice interno tiene 1538 filas observation que SÍ rankean en FTS). La
  estratificación se mide por probe read-only contra search_index
  (harness/stratify.py → results/stratification.jsonl). Las queries
  observation-sensitive se listan ahí; el verdict final debe examinar los cambios
  pareados de ese subgrupo por separado.
- **Sweep de threshold**: se hace offline sobre el score fusionado capturado con
  min_similarity=0.0 (filtrar-luego-truncar top-5). Es una aproximación del efecto
  del threshold real de config (que aplica a similitud vectorial pre-fusión) —
  limitación documentada; la comparación entre brazos usa el mismo procedimiento,
  así que es interna-consistente.

## Baseline sellado (referencia)

hit@5 sobre 55 queries etiquetadas — hybrid 36/55 · text 36/55 · vector 34/55
(commit 6d74513, results/metrics-baseline.md). 19 misses: 18 both-miss + 1 fusion-miss.
