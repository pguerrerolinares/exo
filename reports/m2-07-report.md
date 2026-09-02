# Reporte — m2-07: Design spec de la fusión hybrid + calibración (spec-writer, lane diseño)

## Estado: SPEC COMPLETA — lista para verificación adversarial + gate consultor

Worktree `/home/paul/Documentos/proyectos/exo/.worktrees/m2-07`, rama `m2-07`.

Deliverables:
- `docs/superpowers/specs/2026-07-17-fusion-design.md` — design spec de la fusión (NO hay implementación: cero Rust de fusión escrito, cumplido).
- `evals/e1-read/gate.md` — gate pre-registrado de E1 (3 patas; la pata 2 fija el criterio de fusión, patas 1 y 3 transcripción literal de spec M2 §5).

## Decisiones de diseño (cada una con su cita en la spec)

| # | Decisión | Fuente citada |
|---|---|---|
| D-f1 | Normalización BM25 **por-query** `f = β·f_raw/f_max(q)`, con anclaje β como parámetro del sweep | §4.2 "normalización BM25" + rangos medidos (spread top-1 ×49 → global inviable) |
| D-f2 | Admisión = **UNIÓN** de candidatos; el "gate FTS" se realiza como gate del bonus (`min=0` si no pasa FTS). Lectura "solo-si-pasa-FTS" descartada por aritmética (techo 28/55; 15 queries FTS-vacías, 12 vector-HIT) y relegada a 1 corrida diagnóstica | §4.2 "gate FTS"; pregunta abierta del brief resuelta con datos, no con el código de bm |
| D-f3 | Threshold = `semantic_min_similarity` aplicado a `v` **pre-fusión**, config RO + flags>config (D6); re-sweep obligatorio, ni 0.55 ni 0.35 heredados | spec indexer §5 ("hoy 0.35; calibración en M2-07") + spec madre §4.4 ("Re-sweep del threshold por modelo") |
| — | Fórmula `max(v,f)+bonus·min(v,f)` con canal ausente = 0 (degenera a identidad → viabiliza la unión) | spec madre §4.2 línea 65, literal |
| — | Clave `(type,id)` = `("entity", permalink)`, una fila por entidad | §4.2 + contrato §4.1 sellado |
| — | Envelope v1 y contrato §4.1 INTACTOS; `search_type:"hybrid"` literal; score fusionado "informativo, no contractual" | spec indexer §4.1 (superficie gateada, no tocada) |

## Parámetros del sweep (declarados, no hardcodeados)

- `bonus` ∈ {0.0, 0.1, 0.2, 0.3, 0.5} — §4.2 no da valor.
- `β` (anclaje FTS, flag `--escala-fts`) ∈ {0.6, 0.8, 1.0} — el equilibrio top-FTS vs techo vectorial (0.6764 medido) no tiene dato a priori.
- threshold: post-hoc {None, 0.35…0.65} con `analyze.py` INTACTO, corridas a `--min-similitud 0.0` (método pre-registrado del gate M0, citado); confirmación nativa obligatoria con el valor elegido.
- gate FTS: lectura B en todo el grid + 1 corrida diagnóstica de la lectura A (predicción pre-registrada ≤28/55).
- Selección pre-registrada (anti-renegociación): max hit@5 → menos rotas pareadas → config más simple. Total 16 corridas.

## Señal empírica re-verificada (no tomada de fe del brief)

- hit@5: engine-fts **28/55**, engine-vector **46/55**, bm-hybrid (jina-es) **43/55** — recomputados desde los jsonl con la `norm()` de analyze.py; coinciden con metrics-*.md. Ojo: el `baseline` 36/55 es el arm pre-M0 (bge-small-en); la referencia del gate es el arm jina-es.
- Rangos de score medidos: FTS `-bm25` ∈ [0.5623, 31.1188] (top-1 mediana 9.07, spread ×49); vector coseno ∈ [0.3846, 0.6764].
- Hallazgo clave para el gate: **el vector puro NO pasa el gate pareado** (vs bm-hybrid: arregla 6, ROMPE 3 > 2). De las 3 rotas, exactamente 1 es FTS-HIT (`lighthouses bot amortización…`) — la fusión existe para rescatarla. Unión FTS∪vector = techo 50/55.
- Subgrupo observation-sensitive: 47/55 queries (stratification.jsonl).
- Config viva verificada: `semantic_min_similarity: 0.35` en `~/.basic-memory/config.json` (RO).

## Scope-questions

Ninguna que exigiera escalar (0 > umbral de ~5). La única ambigüedad real del contrato — la semántica de "gate FTS" — la resuelve la propia spec con datos empíricos del engine y queda además cubierta como parámetro del sweep (ambas lecturas medidas).

## Confirmación clean-room

- El diseño deriva EXCLUSIVAMENTE de la prosa de la spec madre §4.2 (línea 65, citada literal en la spec §2), de los contratos de las specs propias (M2 §4/§5, indexer §4.1/§5, gate M0) y de datos medidos en este repo (results/*.jsonl RO, engine/src/buscador.rs propio).
- NO se abrió el repo ni el código de basic-memory en ningún momento. NO se inspeccionó su DB (no hizo falta ni para formas de dato). La única exposición a bm son sus OUTPUTS de arm ya commiteados en `evals/retrieval-fase0/results/jina-es.jsonl` (RO, permitidos y necesarios: son la referencia del gate).
- Cero implementación: no se escribió ni una línea de Rust de fusión.

## Commits

- `docs(m2-07): design spec de la fusión hybrid — clean-room desde §4.2, sweep declarado`
- `docs(m2-07): gate pre-registrado de E1 en evals/e1-read/gate.md`
- `docs(m2-07): reporte del spec-writer`

GATE: MERGED (consultor fable, 2026-07-17T23:49:21+02:00, verdict=evals/e1-read/verdict/gate-m2-07-spec.md@8cbfae7)
