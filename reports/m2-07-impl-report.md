# Reporte — m2-07-impl: fusión hybrid FTS+vector + sweep de calibración

Worktree `/home/paul/Documentos/proyectos/exo/.worktrees/m2-07-impl`, rama `m2-07-impl`
(base `main` en `8299add`, tras el merge de la spec de fusión). Implementación por TDD
según `docs/superpowers/specs/2026-07-17-fusion-design.md` (sellada, no rediseñada).

Commits:
- `73c1675` — feat(engine): fusión hybrid FTS+vector (TDD, 11 tests §7 + CLI + harness).
- `<pendiente>` — docs(m2-07): sellado del sweep + este reporte.

## 1. Implementación (`busca_hybrid`, spec §4)

`engine/src/buscador.rs` añade dos helpers puros (B2 del brief, testeables sin DB) más
la función pública:

- `normaliza_fts(candidatos_fts: &[(String, f64)], beta: f64) -> HashMap<String, f64>`
  — normalización BM25 por-query con anclaje β (D-f1): `f(e) = β·f_raw(e)/f_max(q)`.
  `f_max == 0` (o lista vacía) descarta el canal FTS entero (mapa vacío), sin dividir
  por cero.
- `fusiona(v_por_entidad, f_por_entidad, bonus, limite) -> Vec<Resultado>` — fusión por
  UNIÓN (D-f2), clave = permalink, canal ausente = 0: `score = max(v,f) + bonus·min(v,f)`.
  Orden desc, truncado a `limite` DESPUÉS de fusionar.
- `busca_hybrid(db, query, limite, min_similitud, bonus, escala_fts) -> Busqueda` —
  candidatos FTS hasta **K_c=50** (constante, vía `busca()`, sin refactor necesario
  porque `busca()` ya acepta un límite explícito); candidatos vector exhaustivos con
  threshold pre-fusión sobre `v` (vía `busca_vector()`, mismo threshold/precedencia
  flags>config que el arm vector puro, límite efectivamente sin techo); normaliza y
  fusiona. Envelope `search_type: "hybrid"`, forma del contrato §4.1 intacta.

CLI (`engine/src/main.rs`): `TipoBusqueda::Hybrid` añadido al enum (`--type hybrid`);
flags nuevos `--bonus`/`--escala-fts` (`Option<f64>`), con fallback a constantes del
binario cuando se omiten (`BONUS_SELLADO`/`ESCALA_FTS_SELLADA`, ver §5 sellado).
`busca_cmd` añade el brazo `Hybrid => busca_hybrid(...)`.

Harness: `evals/retrieval-fase0/harness/replay-engine.py` extendido con `--tipo hybrid`
y reenvío opcional de `--min-similitud`/`--bonus`/`--escala-fts` a `exo search` (se
añaden al comando SOLO si se pasaron; si no, `exo` cae a sus defaults). `analyze.py`
**intacto**, no se tocó ni una línea.

### Deviación declarada: dos scripts nuevos en `harness/` (no tocan `analyze.py`/`replay.py`)

`analyze.py` clasifica misses mirando, DENTRO del mismo `results/<arm>.jsonl`, las
claves de `search_type` presentes en cada fila. Eso funciona para el arm bm (su API
real devuelve breakdown text/vector/hybrid en una sola respuesta) pero NO para los
arms del engine: `replay-engine.py` invoca `exo search --type <tipo>` una vez por
query, así que `results/engine-hybrid*.jsonl` solo trae la clave `"hybrid"` — nunca
`"text"`/`"vector"` a la vez. Analizado solo con `analyze.py`, la sección "atribución
de misses" de cualquier arm hybrid del engine clasifica todo como `both-miss` (las
claves ausentes nunca dan HIT). Para cumplir el mandato de atribución (spec §5.2.3,
gate.md pata 2: "atribución de cada miss obligatoria... con los arms fts/vector del
mismo día como testigos") y el sanity-check §6 ("registra qué hits vectoriales
perdió"), añadí dos scripts que **importan `analyze.py` como librería sin
modificarlo**:

- `harness/atribucion-cruzada.py <arm-hybrid> <arm-fts> <arm-vector>` — cruza los
  misses del arm hybrid contra los arms fts/vector del mismo día, query a query.
- `harness/diagnostica-lectura-a.py <arm-hybrid-centro> <arm-fts-k50>` — produce la
  corrida diagnóstica de la lectura A post-hoc (§2 abajo, B4).

Ninguno de los dos añade un code-path nuevo al binario (B4 lo prohíbe explícitamente
para la lectura A); son post-procesado en Python sobre corridas reales ya capturadas.

## 2. TDD — los 11 tests contractuales (spec §7)

Patrón B2 del brief: helpers puros primero, DB fixture solo donde hace falta.

**8 unit-tests puros** (`engine/src/buscador.rs`, `mod tests_fusion`, sin DB):

| # | test | qué verifica |
|---|---|---|
| 1 | `fusion_formula_ambos_canales` | `score == max(v,f)+bonus·min(v,f)` exacto |
| 2 | `fusion_conserva_candidato_solo_vector` | solo-`v` → `score == v` |
| 3 | `fusion_conserva_candidato_solo_fts` | solo-`f` → `score == f` |
| 5 | `fusion_clave_entidad_una_fila_por_permalink` | misma entidad en ambos canales → UNA fila |
| 6 | `normalizacion_bm25_monotona` | orden preservado, top-1 == β exacto, acotado a (0,β] |
| 7 | `normalizacion_bm25_query_sin_fmax` | `f_max==0` → mapa vacío, sin panic de división por 0 |
| 8 | `fusion_bonus_cero_es_max` | `bonus=0` → `score == max(v,f)` |
| 10 | `fusion_orden_desc_truncado_post_fusion` | orden desc + truncado a `limite` tras fusionar |

Nota: el brief (B2) solo listó 7 tests como puros (1/2/3/6/7/8/10) y clasificó los
otros 3 como DB (4/9/11), dejando el test 5 sin asignar explícitamente en esa
partición. El test 5 (clave única por permalink) no necesita DB — se construye igual
que 1/2/3 con dos `HashMap` a mano — así que lo implementé también como unit-test
puro. No reabre ninguna decisión de diseño, solo completa un hueco de esa lista.

**3 tests de integración sobre DB fixture** (`engine/tests/buscador.rs`, mismo patrón
`db_indexada()` que M2-05/M2-06; todos con `min_similitud: Some(x)` explícito — B3,
nunca `None`):

| # | test | qué verifica |
|---|---|---|
| 4 | `fusion_gate_fts_no_pierde_hit_semantico` | query con FTS vacío + `min_similitud: Some(0.0)` → el canal vector no se pierde (unión, D-f2) |
| 9 | `threshold_filtra_vector_pre_fusion` | `min_similitud: Some(1.5)` (inalcanzable) → vector filtrado entero, FTS sobrevive con `score==f` |
| 11 | `busqueda_hybrid_envelope` | `search_type:"hybrid"` literal, forma §4.1 (`permalink`/`type`/`score`) |

### Oráculo — `cargo test --manifest-path engine/Cargo.toml`

```
lib (buscador::tests_fusion + trozos + aristas + vectores): 26 passed
tests/buscador.rs:  13 passed   (10 preexistentes M2-05/M2-06 + 3 nuevos DB)
tests/git_epoch.rs:  2 passed
tests/indexer.rs:   16 passed
tests/nota.rs:       7 passed
tests/schema.rs:     2 passed
tests/smoke.rs:      2 passed, 1 ignored
tests/walker.rs:     4 passed

TOTAL: 72 passed, 0 failed, 1 ignored
```

Todo verde, M2-01..06 intactos (ningún test preexistente tocado ni removido).

## 3. Sweep (§5) — DB real, KB pineada

DB de la sesión: `exo-e1.db` (rebuild completo real, no fixture), commit de
`kb-demo` pineado en **`6578351`** ("docs(kb): documenta arco CEM T1.1 cerrado
NULL..."). Conteos verificados: notas=115, trozos=2213, vectores=2213 (1:1).

Nota de drift declarada: el conteo de trozos/vectores difiere del `2194` medido en
m2-06 (la KB de `kb-demo` ha seguido creciendo desde entonces). Es exactamente la
razón por la que el gate oficial (M2-09) es mismo-día — los números absolutos de este
sweep no se comparan contra los `results/` de julio (spec M2 §5), solo internamente
entre celdas del propio grid, y la pareada usa `jina-es` re-corrido... en rigor
`jina-es.jsonl` en `results/` es de la corrida original de M0 (no re-corrida hoy); la
comparación pareada de este sweep es explícitamente **diagnóstico de selección, no el
gate** (§5.2.4: "el gate oficial es M2-09 mismo-día"), así que usar el `jina-es.jsonl`
existente para la selección interna del sweep es consistente con ese rol.

### 3.1 Grid — 15 corridas (`replay-engine.py <arm> --tipo hybrid --min-similitud 0.0 --limite 10 --bonus B --escala-fts β`, `analyze.py <arm> jina-es`)

| bonus \ β | 0.6 | 0.8 | 1.0 |
|---|---|---|---|
| **0.0** | **49/55** (A7·R1) | 47/55 (A6·R2) | 47/55 (A6·R2) |
| **0.1** | **49/55** (A7·R1) | 48/55 (A6·R1) | 48/55 (A6·R1) |
| **0.2** | **49/55** (A7·R1) | 48/55 (A6·R1) | 48/55 (A6·R1) |
| **0.3** | **49/55** (A7·R1) | 48/55 (A6·R1) | 48/55 (A6·R1) |
| **0.5** | 48/55 (A6·R1) | 48/55 (A6·R1) | 48/55 (A6·R1) |

hit@5 = `thr=None` (headline de `analyze.py`); `A`/`R` = ARREGLA/ROMPE de la pareada
`engine-hybrid-b{bonus}-e{β} vs jina-es` (siempre a `thr=None`, es como la calcula
`analyze.py` — sin argumento de threshold).

Barrido completo de threshold post-hoc (`thr=0.35…0.65`) por celda: el máximo de cada
celda coincide con su headline en `thr=0.35`/`0.40` (verificado en las 15
`metrics-engine-hybrid-*.md`); ninguna celda supera 49/55 en ningún threshold. Ejemplo
(celda ganadora, `b0.0-e0.6`):

```
thr=0.35: 49/55   thr=0.40: 49/55   thr=0.45: 46/55
thr=0.50: 41/55   thr=0.55: 32/55   thr=0.60: 19/55   thr=0.65: 0/55
```

### 3.2 Diagnóstica de lectura A (§4.5/§5.1, B4) — 1 corrida

Bonus/β centrales (0.2/0.8), post-hoc sobre la corrida ya capturada
`engine-hybrid-b0.2-e0.8` (min_similitud 0.0, límite 10) cruzada contra una corrida
FTS fresca a K_c=50 (`engine-fts-k50-m207`, mismo DB) vía
`harness/diagnostica-lectura-a.py`: filtra la lista fusionada a los permalinks
presentes en el pool FTS-50 (admisión = gate FTS, lectura A) antes de truncar a 5.

**Resultado: 30/55**, dentro del rango pre-registrado **28–41/55** (predicción §4.5).
Confirma la descarte de la lectura A como default: incluso en su mejor caso posible
(re-rank vectorial sobre el pool FTS de 50), pierde 19 hits frente a la lectura B
(unión), consistente con las 14 queries FTS-vacías-con-gold que A nunca puede acertar.

### 3.3 Selección pre-registrada (§5.2.4)

1. **Max hit@5**: 49/55, empatado en 4 celdas — todas con β=0.6, bonus ∈ {0.0, 0.1, 0.2, 0.3}.
2. **Menos ROTAS pareadas vs jina-es**: las 4 celdas empatan también aquí (ARREGLA 7,
   ROMPE 1 idéntico en las cuatro) — no desempata.
3. **Menor bonus**: de {0.0, 0.1, 0.2, 0.3} → **bonus = 0.0**.
4. **Mayor β**: ya fijado en 0.6 por el paso 1 (las 4 celdas comparten β=0.6, no hay
   elección aquí).
5. **Mayor thr**: para (bonus=0.0, β=0.6), 49/55 se sostiene en `{None, 0.35, 0.40}`
   (cae a 46/55 en 0.45) → el mayor threshold numérico que sostiene el máximo es
   **thr = 0.40**.

**Config ganadora: bonus = 0.0, β (escala_fts) = 0.6, threshold = 0.40.**

Dato notable, reportado sin suavizar: el bonus ganador es **0.0** — la fórmula de
fusión colapsa a `score = max(v,f)` puro en el resto de este eval set; el término
`bonus·min(v,f)` (la "confirmación del canal débil") no aporta hits adicionales aquí.
Es el resultado que da el procedimiento pre-registrado, no una elección de diseño
posterior.

### 3.4 Confirmación nativa (§5.2.5)

`replay-engine.py engine-hybrid-nativo-final --db exo-e1.db --tipo hybrid --min-similitud 0.40 --limite 5 --bonus 0.0 --escala-fts 0.6` (sin post-hoc, threshold real
pre-fusión sobre `v`) + `analyze.py engine-hybrid-nativo-final jina-es`:

```
- **hybrid**: 49/55
## pareada engine-hybrid-nativo-final vs jina-es: ARREGLA 7 [...] · ROMPE 1 [...]
```

**Idéntico al post-hoc** (49/55, A7·R1) — las tres clases de divergencia documentadas
en §5.2.5 (bonus·min de fusionados bajo threshold, solo-FTS que el post-hoc filtra de
más, re-ranking) no se manifestaron en esta config porque `bonus=0.0` elimina la
primera clase de raíz y el resto no tocó al top-5 de estas 55 queries. El número
sellado es este nativo: **49/55**.

## 4. Sellado (§5.2.6)

`engine/src/main.rs`: constantes `BONUS_SELLADO = 0.0` y `ESCALA_FTS_SELLADA = 0.6`
(antes provisionales B1 = 0.2/0.8), usadas como fallback de `--bonus`/`--escala-fts`
cuando se omiten. El threshold ganador (**0.40**) **no** se hardcodea como constante
del binario — D-f3/§4.6: difiere del `0.35` de `semantic_min_similarity` (config RO de
basic-memory), y esa config no es propia de exo hasta M5a; se documenta aquí y se pasa
explícito (`--min-similitud 0.40`) en corridas/consumidores hasta entonces. `exo search
--type hybrid` sin flags hoy usa bonus=0.0/β=0.6/threshold=config(0.35) — el operador
que quiera el punto exacto sellado por este sweep debe pasar `--min-similitud 0.40`.

## 5. Sanity-check (§6, informativo, NO gate)

`engine-hybrid` nativo = **49/55 ≥ 46/55** (floor de la spec, `busca_vector` m2-06) y
**≥ 47/55** (vector puro re-corrido hoy mismo, `engine-vector-m207`, mismo DB/día,
threshold config 0.35) → pasa el sanity-check con margen.

Atribución (`atribucion-cruzada.py engine-hybrid-nativo-final engine-fts-m207
engine-vector-m207`, mismo día, mismo DB):

- **Hits vectoriales puros perdidos por la fusión: 2** — `cge bitácora`,
  `reflex capa de reflejos plugin destilado canónico`.
- **Hits FTS puros perdidos por la fusión: 0**.
- **both-miss** (ni FTS ni vector puro acertaban tampoco): 4 — `cge evaluación
  head-to-head cgeo benchmark harness metodología`, `coste workflows multi-agente
  tokens lección`, `esa utilidad de terminal de solo lectura que da a las sesiones un
  resumen estructural barato de mis notas para no gastar tokens leyéndolo todo`,
  `fabrica campaña`.

Subgrupo observation-sensitive: 0 resultados `type:"observation"` en el top-5 de
`engine-hybrid-nativo-final` (consistente con el contrato §4.1: resultados siempre a
nivel entidad) — sin degradación que examinar aparte.

## 6. Resumen de comandos ejecutados (reproducibilidad)

```
exo rebuild --db exo-e1.db --json                                   # DB base, kb-demo@6578351

# grid 15 celdas (bonus × β), cada una:
replay-engine.py engine-hybrid-b{B}-e{E} --db exo-e1.db --tipo hybrid \
    --min-similitud 0.0 --limite 10 --bonus {B} --escala-fts {E}
analyze.py engine-hybrid-b{B}-e{E} jina-es

# diagnóstica A
replay-engine.py engine-fts-k50-m207 --db exo-e1.db --tipo fts --limite 50
diagnostica-lectura-a.py engine-hybrid-b0.2-e0.8 engine-fts-k50-m207

# confirmación nativa
replay-engine.py engine-hybrid-nativo-final --db exo-e1.db --tipo hybrid \
    --min-similitud 0.40 --limite 5 --bonus 0.0 --escala-fts 0.6
analyze.py engine-hybrid-nativo-final jina-es

# sanity-check / atribución
replay-engine.py engine-fts-m207 --db exo-e1.db --tipo fts --limite 5
replay-engine.py engine-vector-m207 --db exo-e1.db --tipo vector --limite 5
atribucion-cruzada.py engine-hybrid-nativo-final engine-fts-m207 engine-vector-m207
```

Todos los `.jsonl`/`metrics-*.md` de las corridas (15 celdas del grid + diagnóstica A +
confirmación nativa + fts/vector same-day para la atribución del sanity-check) quedan
commiteados en `evals/retrieval-fase0/results/`, mismo patrón que M2-05/M2-06
(`engine-fts.jsonl`/`engine-vector.jsonl`/`metrics-*.md` ya estaban tracked).

## 7. Prohibiciones respetadas

`analyze.py`/`replay.py`/`gate.md` intactos (0 diffs). Envelope §4.1 y schema §2 no
tocados. Sin apertura de basic-memory. Sin renegociar los números del gate — la
selección siguió el orden pre-registrado §5.2.4 al pie de la letra, incluido el
resultado poco intuitivo `bonus=0.0`. Sin merge/push a `main`; todo en `m2-07-impl`.
