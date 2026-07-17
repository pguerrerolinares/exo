# Verdict del gate — m2-07 (design spec de la fusión hybrid + gate pre-registrado)

- **Veredicto: MERGED**
- **Adjudica**: consultor Fable delegado (dispatch fresco, sin participación en la redacción de la spec ni en su verificación adversarial), régimen de gates de `.superpowers/fabrica/config.md` §"Ejecución de gates" (4 condiciones, cumplimiento al final).
- **Fecha**: 2026-07-17 (noche)
- **Rama juzgada**: `m2-07` (HEAD `714500b`), diff vs `main` (d93d362): 3 ficheros, +282/−0 tras fixes — `docs/superpowers/specs/2026-07-17-fusion-design.md` (spec), `evals/e1-read/gate.md` (gate pre-registrado), `reports/m2-07-report.md` (review-package). **Cero código**: `git diff main..HEAD -- engine/` = ∅, verificado.
- **Criterio citado**: spec madre `2026-07-16-framework-unificado-design.md` §4.2 línea 65 ("Fusión: copiar el **diseño** de basic-memory (fórmula max(v,f)+bonus·min(v,f), clave (type,id), gate FTS, normalización BM25, threshold configurable) — **jamás el código: basic-memory es AGPL-3.0**" — cotejada literal contra la cita de la spec §2) + §4 punto 4 ("comparación pareada, no test de proporciones"; "Re-sweep del threshold por modelo"; "Atribución de cada miss"). Spec M2 `2026-07-17-m2-e1-read-design.md` §1 (no-objetivos), §4 ("analyze.py intacto"), §5 (3 patas; pata 2 literal).

## Verificación primaria propia (no del reporte, no del verdict previo)

Script independiente propio (re-implementa norm/hit desde cero, lee SOLO los jsonl crudos de `evals/retrieval-fase0/results/`, fuera del worktree) + cross-check con `analyze.py` INTACTO (diff vs main = 0 líneas) re-corrido y `results/` restaurado después con `git checkout`.

| Afirmación (spec §3) | Re-derivado por mí | ¿Cuadra? |
|---|---|---|
| eval set 56 filas / 55 con gold | 56 / 55 | ✓ |
| hit@5 engine-fts | **28/55** | ✓ |
| hit@5 engine-vector | **46/55** | ✓ |
| hit@5 bm-hybrid jina-es | **43/55** (thr=None **y** thr=0.35 — empate, confirma el "moot hoy" de la nota M4) | ✓ |
| FTS ∪ vector / inter / solo-FTS / solo-vector / both-miss | 50 / 24 / 4 / 22 / 5 | ✓ |
| queries 0-resultados-FTS | 15/56, **14 con gold**, 12 vector-HIT (aritmética del techo 41 = 55−14 del fix M1: verificada) | ✓ |
| **pareada vector-vs-bm** | **ARREGLA 6 / ROMPE 3**; de las 3 rotas exactamente 1 FTS-HIT (`lighthouses bot amortización…`) — la justificación de que la fusión exista, reproducida query a query | ✓ |
| rangos FTS | n=122, [0.5623, 31.1188], mediana 5.41; top-1 [0.6366, 31.1188] mediana 9.07 spread ×48.9 | ✓ |
| rangos vector | n=280, [0.3846, 0.6764]; top-1 [0.4285, 0.6764] mediana 0.548 | ✓ |
| observation-sensitive | 47/55 (`stratification.jsonl`) | ✓ |
| orden desc por score en jsonl (validez de "top-1 = primer elemento") | True en todas las filas de ambos arms engine | ✓ |

Cross-check `analyze.py` (segundo oráculo): regenera `metrics-engine-fts.md` y `metrics-engine-vector.md` **byte-idénticos** a lo commiteado; en `metrics-jina-es.md` solo desaparece la sección "vs minilm" añadida a mano (comportamiento documentado en el propio fichero). Worktree restaurado, status limpio.

## Clean-room / AGPL — LIMPIO

- Diff completo main..HEAD: exactamente 3 ficheros de docs, cero código, cero cambios en `engine/`, harness, `analyze.py`, `replay*.py`.
- Grep de firmas de código y de internals de bm sobre el diff (`def |fn |class |import |SELECT|search_repository|permalink_match|src/basic|search.py|repository.py|vendoriz`): 0 hits reales. Las 5 menciones a basic-memory en el diff son: la cita literal de la spec madre §4.2, la prosa del veto clean-room (×2), y la ref RO a `~/.basic-memory/config.json` (D6, pre-existente) (×2). Sin contaminación.

## gate.md fiel a spec M2 §5 — SÍ, y congelado de verdad

- **Pata 2 cotejada palabra a palabra** contra spec M2 §5.2: "engine-hybrid **rompe ≤2 y arregla ≥ las que rompe** vs bm-hybrid (referencia hoy 43/55)…" — transcripción LITERAL, incluida la cláusula "prohibido comparar contra `results/` de julio" y el cierre "los números no se renegocian post-hoc". Patas 1 y 3 ídem, literales.
- **Separación GATE oficial vs sanity-check 46**: limpia. El sanity ("engine-hybrid < 46/55 señala fusión mal calibrada") queda marcado "informativo, NO es el gate", "no bloquea por sí solo; no sustituye ni endurece el gate oficial". Busqué endurecimiento encubierto (sanity→criterio, subgrupo→gate) y relajación encubierta ("referencia real = mismo día" NO es relajación: es exactamente lo que exige el pareado mismo-día de spec M2 §5): nada.
- **Congelado**: `git diff c100ce8..HEAD -- evals/e1-read/gate.md` = ∅ — el commit de fixes 714500b NO lo tocó (verificado por diff, no por el mensaje del commit).

## Fixes 714500b — auditados contra el diff exacto (`git show 714500b`)

Un solo fichero (la spec de fusión, +8/−6), los 4 fixes hacen lo que dicen y nada más:

- **M1** (§4.5, D-f2, tabla §5.1): "techo de A = 28" corregido a techo duro ≤41 (55−14 FTS-vacías-con-gold — aritmética re-verificada por mí: 14 exactas) con 28 como cota inferior; predicción diagnóstica reformulada a 28–41. ✓
- **M2** (§3.2): "techo teórico" → "referencia empírica de unión de top-5s", con el techo estricto 55/55 explicado (vector exhaustivo). ✓
- **M3** (§5.2.5 + §5.3): las 2 clases de divergencia que faltaban (candidatos solo-FTS threshold-izados de facto por el post-hoc; re-ranking que el post-hoc no hace) añadidas coherentemente en ambos sitios. ✓
- **M4** (§5.2): nota operacional del runner M2-09 (bm capturado nativo a su mejor thr si ≠ sin-filtro) — anti-sesgo A FAVOR del engine, no relaja nada; su afirmación "hoy es moot" re-verificada (43=43). Colocada en la spec, NO en gate.md. ✓

Ninguna decisión (D-f1/D-f2/D-f3) cambió; ningún contenido nuevo fuera de los 4 fixes.

## Qué busqué para objetar (mandato de disenso)

1. **Sweep gameable** — selección pre-registrada determinista (max hit@5 → menos ROTAS pareadas → menor bonus → mayor β → mayor thr): sin grados de libertad post-hoc; confirmación nativa obligatoria y "el número que se sella es el nativo"; config sellada antes del gate M2-09 y "no se retoca". El tie-break pareado contra `results/` de julio está declarado "diagnóstico de selección, no el gate" — compatible con la prohibición de spec M2 §5 (que aplica al gate). Grid cubre los extremos semánticos (bonus 0 = puro max; β 0.6 ≈ paridad con techo vectorial 0.6764 medido; 1.0 = sin anclaje). Sin objeción.
2. **Diseño incoherente** — normalización D-f1: monótona, acotada (0, β], degenerados cubiertos (1 candidato → β; f_max=0 → canal descartado); única compatible con el spread ×49 que yo mismo re-medí (×48.9). Fórmula §4.4 degenera limpiamente con canal ausente = 0 (lo que viabiliza la unión sin casos especiales). D-f3 pre-fusión = misma semántica que `busca_vector` hoy. Sin objeción.
3. **Viabilidad del gate M2-09 vendida de más** — comprobé la aritmética del argumento: vector puro rompe 3 (>2); de esas, las 2 no-FTS-HIT son both-miss a top-5, solo rescatables por efectos de pool profundo. La spec NO promete pasar el gate; documenta que la fusión debe rescatar `lighthouses…` sin romper hits vectoriales (riesgo §8.1 declarado, con la atribución como red). Honesto. Sin objeción.
4. **Sobreajuste al eval set** — declarado en §5.3 con la mitigación del precedente M0 (mismo trato a bm, gate pareado, sin gold nuevo — prohibido por spec M2 §3). Riesgo residual real pero pre-registrado. Sin objeción.
5. **Deferencia al verdict previo** — no deferí: TODOS los números de la tabla de arriba salen de mi propio script contra los jsonl crudos; los cotejos de gate.md y del diff 714500b son míos. El verdict previo (`m2-07-verificacion-adversarial.md`, APROBADA-CON-OBJECIONES-MENORES, 0 bloqueantes) resulta consistente con lo que yo mismo medí — coincidencia por verificación, no por confianza.

**Observación menor registrada (no bloqueante, no exige acción pre-merge)**: `reports/m2-07-report.md` es el snapshot del spec-writer PRE-fixes: conserva "predicción pre-registrada ≤28/55" y "Estado: … lista para verificación adversarial", desactualizados tras 714500b. La fuente normativa (la spec) está corregida; el reporte es artefacto histórico del review-package. Que nadie cite la predicción desde el reporte: la vigente es 28–41 (spec §4.5/§5.1).

## Cumplimiento de las 4 condiciones del régimen

1. **Fresco**: sí — dispatch nuevo, sin participación en spec ni verificación previa.
2. **Verificación primaria**: sí — script independiente propio desde jsonl crudos + analyze.py intacto re-corrido; tabla completa arriba.
3. **Disenso**: sección anterior, incluye lo que salió limpio y por qué.
4. **Verdict-artifact versionado**: este fichero, `evals/e1-read/verdict/gate-m2-07-spec.md`, commiteado en la rama `m2-07` ANTES de cualquier GATE-EXEC.

**MERGED.** El merge a main lo ejecuta el orquestador (GATE-EXEC); este verdict no mergea nada.
