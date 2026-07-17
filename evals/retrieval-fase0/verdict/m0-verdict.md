# Verdict M0 — Fase 0 retrieval (adjudicación Fable, régimen §8)

- **Fecha**: 2026-07-17
- **Adjudicador**: consultor Fable T8 (régimen §8 de la spec: decisiones pre-aprobadas por Paul; verdict final salvo línea roja). Sin participación en tareas previas de la campaña.
- **Gate**: `evals/retrieval-fase0/gate.md` (inmutable, no se tocó).
- **Método de verificación primaria** (obligatoria, ejecutada antes de adjudicar):
  1. Re-corrida completa de `harness/analyze.py` (baseline, jina-es vs baseline, minilm vs baseline, textfts). Los `metrics-*.md` regenerados son **byte-idénticos** a los commiteados, salvo la sección "pareada jina-es vs minilm" de `metrics-jina-es.md`, que estaba documentada como añadida a mano (working tree restaurado tras la comparación).
  2. Recomputación **independiente** del criterio 2 y de las pareadas desde los jsonl crudos (script propio, sin importar `analyze.py`): jina-es criterio2 n=26, minilm n=26, baseline n=20; pareadas 7/0 y 7/2 — coinciden con lo reportado.
  3. Conteos: 168 filas y 0 errores en los 4 jsonl; estratificación 47/55 observation-sensitive; config vigente verificada por lectura directa (`minilm`, 384, `default_search_type=null`, `min_similarity=0.55`).
  4. Muestreo ground-truth por filesystem (solo Read/grep sobre `~/Documentos/proyectos/kb-demo`, cero MCP, cero escrituras): 3 queries arregladas por jina-es y 2 misses. Las 3 arregladas tienen el expected real en su top-5 (ranks 2, 1, 1) y las notas existen y responden a la query (memoria-v2-design, ingesta-incremental, kbx). Los 2 misses (`fabrica campaña` → `projects/agent-develop`; `coste workflows multi-agente tokens lección` → `learnings/desarrollo-agentico`) apuntan a notas que existen y son pertinentes — misses reales, no artefactos de etiquetado. **Las métricas no mienten.**
  5. Latencia: mediana 4.4s / p90 4.55s por llamada CLI (`elapsed_s`, 168 filas de jina-es.jsonl) — evidencia primaria para la decisión 3.

---

## Decisión 1 — Gana **jina-es** (criterio 1)

> Gate: *"GANA un candidato si: arregla ≥5 queries y rompe ≤1 (pareada)."*

- **jina-es: ARREGLA 7 · ROMPE 0 → GANA** (7 ≥ 5, 0 ≤ 1). hit@5 hybrid 43/55 vs baseline 36/55.
- **minilm: ARREGLA 7 · ROMPE 2 → NO gana** (rompe 2 > 1). 41/55.
- Subgrupo observation-sensitive (ajuste pre-registrado del gate: *"el verdict final debe examinar los cambios pareados de ese subgrupo por separado"*): examinado — las 7 arregladas y 0 rotas de jina-es caen íntegras dentro del subgrupo (baseline 30/47 → jina-es 37/47). Sin divergencia entre subgrupo y total; el resultado no está inflado por queries fuera del estrato dominante.
- Confound residual reindex-vs-modelo (anotado en el ledger de T7): adjudicado bajo la cláusula 4 del gate (*"Empate o resultado ambiguo ⇒ el consultor Fable adjudica"*). No invalida: (a) el brazo se define como paquete config+reindex — no existe forma de aplicar el modelo sin reindexar, así que el paquete es exactamente lo que se despliega; (b) los dos brazos comparten higiene de índice y difieren en resultado (43 vs 41, pareada directa 4/2), luego el modelo es causalmente relevante por sí mismo.

## Decisión 2 — Semántica **LOAD-BEARING** ⇒ engine en **Rust** (criterio 2 + spec §4.5)

> Gate: *"SEMÁNTICA LOCAL LOAD-BEARING si: con el mejor brazo, ≥3 queries etiquetadas tienen HIT en vector u hybrid y MISS en text (la semántica aporta lo que FTS no puede). → decide lenguaje del engine (spec §4.5): load-bearing ⇒ Rust; si no ⇒ Go."*

- Con el brazo ganador (jina-es) y el control FTS real (`textfts`, 18/55 en modo text): **n = 26/55** queries con HIT en hybrid/vector y MISS en FTS. Recomputado independientemente desde los jsonl. 26 ≥ 3 con margen ~9x.
- **Ponderación de la desviación post-gate** (control textfts capturado tras el sellado): **SOSTIENE la medición, no la invalida.** Razones:
  1. El "text" de todas las capturas pre-sellado era hybrid disfrazado (`_default_search_type()` auto-resuelve hybrid con `semantic_search_enabled=true` y `default_search_type=null`; verificado en fuente por el reviewer de T7). El criterio 2 tal como está escrito era **literalmente incalculable** con los datos sellados — la desviación no cambia la regla, la hace medible.
  2. El fix no tocó queries, labels, scoring ni el gate: solo forzó `default_search_type="text"` (atómico, restaurado y verificado), mismo harness y misma `norm()`.
  3. El control es válido para ambos brazos: FTS no depende del modelo de embeddings (verificado en fuente); las filas hybrid/vector de textfts son idénticas a las de minilm, confirmando que nada más se movió.
  4. Robustez: incluso el baseline da n=20 ≥ 3 contra el mismo control. La conclusión no depende del brazo ni de deducciones finas.
- La dirección de la desviación es además conservadora respecto al riesgo de sesgo: sin el fix, text≡hybrid habría dado n≈0 y decidido Go por artefacto de medición.
- Per spec §4.5 (*"Si la semántica local es load-bearing (Fase 0 lo dice): Rust para el binario nuevo"*): **Rust**.

## Decisión 3 — Urgencia de M2: **BAJA a "estrangulamiento tranquilo"** (criterio 3)

> Gate: *"si un candidato GANA, M2 baja a 'estrangulamiento tranquilo' (config-fix aplicado, dolor mitigado)."*

- jina-es GANA ⇒ por texto literal del gate, M2 baja. Per spec §7: *"Si M0 sale 'config-fix suficiente', M2 baja de prioridad sin drama"* — M2 arranca en el hueco real entre universidad y cge P2, no antes ni desplazándolos.
- Lo que el config-fix **no** resuelve (motiva M2 como dirección, no como urgencia):
  1. **Latencia**: mediana 4.4s por búsqueda CLI (medida, 168 filas). Los hooks y el recall pagan ese arranque en cada llamada — exactamente el motivo "arranque en ms" de §4.5.
  2. **FTS real débil**: 18/55. El hybrid actual vive casi enteramente de la semántica; un motor propio con FTS+fusión decentes tiene margen real (el diseño de fusión a copiar está identificado en §4.2).
  3. **12 misses restantes** de jina-es: 11 both-miss (techo del stack actual completo, ni FTS ni semántica los ven) + 1 fusion-miss (`cge bitácora`, HIT en vector que la fusión pierde — bug de fusión que el motor propio puede arreglar).
  4. **Threshold inútil**: el sweep no encuentra ningún punto que mejore sobre "sin filtro" — el mecanismo de threshold actual no aporta.

## Decisión 4 — Config final a aplicar (el sistema está en minilm: **requiere cambio + reindex**)

> Gate: *"cada brazo en su mejor threshold del sweep (mismo procedimiento para todos los brazos)."*

- `semantic_embedding_model`: **`jinaai/jina-embeddings-v2-base-es`**
- `semantic_embedding_dimensions`: **768**
- `default_search_type`: **`null`** (auto-resuelve hybrid, el modo medido y ganador)
- `semantic_min_similarity`: **0.35** (mejor threshold del sweep de jina-es: 0.35→43/55, empata con sin-filtro; **dejar el 0.55 vigente sería activamente dañino**: el sweep da 29/55 en 0.55 — peor que baseline. Limitación conocida y pre-registrada: el sweep es offline sobre score fusionado y el threshold de config aplica pre-fusión — *"la comparación entre brazos usa el mismo procedimiento, así que es interna-consistente"*; 0.35 es la mejor aproximación disponible bajo el procedimiento sellado, y el riesgo de sobre-filtrado con 0.55 está medido).
- **Acción para el controller**: aplicar config (escritura atómica), `basic-memory reindex` (el incremental re-embeddea todo al detectar cambio de modelo — verificado en T7), smoke test vector ≥1 resultado con score.

## Decisión 5 — Ambigüedades adjudicadas (cláusula 4)

> Gate: *"Empate o resultado ambiguo ⇒ el consultor Fable adjudica con este texto delante; no se re-negocian los números post-hoc."*

1. Desviación textfts post-sellado → **sostiene** el criterio 2 (razonado en Decisión 2). Los números no se re-negociaron: 43/55, 41/55, 7/0, 7/2, 26/55 y 18/55 quedan tal como se midieron y fueron re-verificados aquí.
2. Confound reindex/purge sobre el criterio 1 → **no invalida** (razonado en Decisión 1).
3. Threshold de config vs sweep offline → se aplica **0.35** como mejor lectura del procedimiento pre-registrado; si en uso real el 0.35 pre-fusión se comportara distinto del offline, eso es medición nueva para el eval set permanente (output (c) de §4.1), no re-negociación de este gate.

**Sin línea roja detectada.** Verdict: FINAL.
