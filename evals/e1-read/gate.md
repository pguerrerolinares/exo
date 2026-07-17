# Gate pre-registrado — E1 read (corrida final M2-09)

Redactado y commiteado en M2-07, ANTES de implementar la fusión y ANTES de la
corrida final (spec M2 `2026-07-17-m2-e1-read-design.md` §5: "pre-registrar en
`evals/e1-read/gate.md` y commitear ANTES de la corrida final"). Tres patas
obligatorias. "Ambigüedad o empate ⇒ consultor fable adjudica con este texto
delante; los números no se renegocian post-hoc" (spec M2 §5, literal).

## Pata 1 — Paridad de corpus (spec M2 §5.1, literal)

"diff de permalinks a nivel entidad = **∅**, cero tolerancia; exclusiones §6.2
verificadas explícitamente (dotdirs fuera, `archive/` dentro, 5 entidades no-md
fuera, 0 permalinks regenerados)."

## Pata 2 — Retrieval pareado (criterio de fusión fijado en M2-07)

**GATE oficial, NO negociable** (spec M2 §5.2, literal): "ambos arms
re-corridos el mismo día sobre el mismo estado de la KB (commit de kb-demo
pineado en el verdict); prohibido comparar contra `results/` de julio. Gate:
engine-hybrid **rompe ≤2 y arregla ≥ las que rompe** vs bm-hybrid (referencia
hoy 43/55). Subgrupo observation-sensitive examinado aparte. `cge bitácora`
(fusion-miss conocido) = diagnóstico informativo, no exigible."

Procedimiento vinculante (fijado por la spec de fusión
`docs/superpowers/specs/2026-07-17-fusion-design.md` §5-§6):

- Comparación **pareada, no test de proporciones** (spec madre §4 punto 4);
  la pareada la computa `analyze.py <arm-engine> <arm-bm>` (ARREGLA/ROMPE),
  intacto.
- La config de fusión (bonus, β, threshold, gate FTS) llega **sellada por el
  sweep de M2-07** (selección pre-registrada en la spec de fusión §5.2.4,
  confirmación nativa §5.2.5) ANTES de la corrida de este gate; no se retoca
  contra el resultado.
- bm-hybrid compite con su mejor threshold del mismo procedimiento de sweep
  (precedente pre-registrado del gate M0: "cada brazo en su mejor threshold
  del sweep (mismo procedimiento para todos los brazos)").
- La referencia 43/55 es la de hoy; el número de referencia real es el de la
  corrida bm-hybrid del mismo día (el gate es pareado, no contra 43 congelado).
- **Atribución de cada miss obligatoria** (spec madre §4 punto 4:
  FTS-miss / vector-miss / threshold-miss, vía `search_type` explícito, con
  los arms fts/vector del mismo día como testigos).
- **Subgrupo observation-sensitive** (47/55 queries,
  `evals/retrieval-fase0/results/stratification.jsonl`): cambios pareados del
  subgrupo listados aparte en el verdict; informativo salvo degradación que el
  consultor adjudique con este texto delante.

**Sanity-check de ingeniería — informativo, NO es el gate**: engine-vector
puro ya da 46/55 (m2-06). Un engine-hybrid < 46/55 señala fusión mal calibrada
aunque pase el gate pareado vs bm; el verdict debe registrarlo con atribución
de qué hits vectoriales perdió la fusión. No bloquea por sí solo; no sustituye
ni endurece el gate oficial.

## Pata 3 — Recall demostrado (spec M2 §5.3, literal)

"(a) `exo recall --json` valida contra golden envelope; (b) latencia:
arranque+consulta FTS-only **p95 < 100 ms**; hybrid en frío con carga de
modelo **p95 < 2.0 s** (referencia bm hoy: mediana 4.4 s)."
