# Fusión hybrid + calibración de threshold — design spec (M2-07)

- **Fecha**: 2026-07-17 · **Estado**: propuesta — pendiente de verificación adversarial + gate consultor
- **Item**: M2-07 (spec M2 `2026-07-17-m2-e1-read-design.md` §3, lane diseño). Spec-first: esta spec se sella ANTES de la primera línea de código de fusión, mismo patrón que la spec del indexer (M2-02).
- **Clean-room**: TODO el diseño de la fusión deriva de UNA frase de la spec madre `2026-07-16-framework-unificado-design.md` §4.2 (línea 65) más datos empíricos de los arms ya corridos (`evals/retrieval-fase0/results/*.jsonl`, RO) y del código propio del engine. Cero inspección del código de basic-memory (veto AGPL, spec M2 §2: "de basic-memory se copia el **diseño** de la fusión […], jamás código ni vendorizado").

## 1. Objetivo y no-objetivos

**Objetivo**: diseño completo de `exo search --type hybrid` — fusión de los canales FTS (M2-05) y vector (M2-06) a nivel entidad — y el procedimiento de calibración (sweep) de sus parámetros, con el gate de retrieval pareado pre-registrado en `evals/e1-read/gate.md`.

**No-objetivos** (spec M2 §1 + brief): ranking por grafo (fuera de E1 — spec M2 §1: "ranking por grafo (solo indexado de aristas)"); write-path (M4); `exo recall` y golden envelope (M2-08); cambios al envelope v1 o al schema §2 (superficies gateadas selladas); generalizar `replay.py`; tocar `analyze.py` (spec M2 §4: "**Reuso, no reescritura**: `analyze.py` intacto").

## 2. Fuente normativa única del diseño

Spec madre §4.2, LITERAL (línea 65):

> "Fusión: copiar el **diseño** de basic-memory (fórmula `max(v,f)+bonus·min(v,f)`, clave `(type,id)`, gate FTS, normalización BM25, threshold configurable) — **jamás el código: basic-memory es AGPL-3.0**"

Esa frase da cinco componentes: (1) fórmula, (2) clave de fusión, (3) gate FTS, (4) normalización BM25, (5) threshold configurable. Los §4.x de abajo aterrizan cada uno a los scores reales del engine. Lo que la frase NO fija (valor del bonus, forma exacta de la normalización, semántica exacta del gate FTS, valor del threshold) queda declarado como **parámetro del sweep** (§5), no como constante inventada — mandato del brief: "marca lo que quede sin datos como parámetro del sweep".

## 3. Señal empírica verificada (re-medida en este worktree, no tomada de fe)

Todos los números re-computados desde `evals/retrieval-fase0/` (eval set M0: `eval.jsonl`, 56 filas, 55 con gold — spec M2 §3 fila M2-07: "gold = eval set M0"; NO se construye gold nuevo).

### 3.1 hit@5 por arm (verificado contra `results/metrics-*.md` y recomputado)

| Arm | hit@5 | Fuente |
|---|---|---|
| engine-fts (`--type fts`, M2-05) | **28/55** | `results/metrics-engine-fts.md` |
| engine-vector (`--type vector`, M2-06, thr 0.35) | **46/55** | `results/metrics-engine-vector.md` |
| bm-hybrid (arm `jina-es`, referencia a batir) | **43/55** | `results/metrics-jina-es.md` (su mejor thr del sweep: 0.35 → 43) |

### 3.2 Estructura de la unión (computada query a query, `norm()` de analyze.py)

- FTS ∪ vector = **50/55** (referencia empírica: unión de los hits **top-5** de cada canal por separado; NO es techo teórico estricto — el canal vector es exhaustivo (KNN `k = COUNT(*)`, §4.2), así que con `thr=None` el gold siempre está en el pool de candidatos y el techo teórico estricto es 55/55; 50 es "lo que una fusión que reordena top-5s puede juntar"); intersección 24; **solo-FTS 4**; solo-vector 22; both-miss 5.
- **15/56 queries devuelven 0 resultados FTS**; de ellas, **12 son vector-HIT**.
- **Pareada engine-vector vs bm-hybrid: arregla 6, rompe 3.** El vector puro NO pasa el gate pareado (rompe 3 > 2) pese a su 46. De las 3 rotas, exactamente 1 (`lighthouses bot amortización triángulos energía planner`) es FTS-HIT — la fusión tiene que rescatarla vía canal FTS para bajar a rompe ≤2. **La fusión no es opcional para pasar el gate; es la pieza que falta.**
- Subgrupo observation-sensitive (`results/stratification.jsonl`): **47/55** queries.

### 3.3 Rangos de score reales (base de la normalización — medidos, blindspot nota 3)

- **FTS** (`buscador::busca`, `score = -bm25(notas_fts)`, "mayor = mejor"; ≥0 garantizado porque el `bm25()` nativo de SQLite es ≤0): sobre los 122 resultados de `results/engine-fts.jsonl`: rango global **[0.5623, 31.1188]**, mediana 5.41; top-1 por query **[0.6366, 31.1188]**, mediana 9.07. El spread del top-1 es **×49**: la escala BM25 depende del nº de términos de la query y de sus estadísticas de corpus → una normalización global (dividir por un máximo fijo) es inviable; tiene que ser por-query (§4.3).
- **Vector** (`buscador::busca_vector`, similitud coseno `1 − L2²/2` sobre embeddings unitarios de fastembed, agregación chunk→entidad por máximo): sobre los 280 resultados de `results/engine-vector.jsonl` (corrida con umbral runtime 0.35): rango **[0.3846, 0.6764]**, top-1 por query [0.4285, 0.6764], mediana top-1 0.548. Cota teórica [−1, 1]; **techo empírico observado ≈ 0.68** — con este modelo ni los matches perfectos del eval llegan a 0.7. Este techo es el dato que motiva el parámetro de anclaje β (§4.3).

## 4. Diseño de la fusión

### 4.1 Clave de fusión `(type, id)`

Cita: §4.2 "clave `(type,id)`". Aterrizaje (brief punto 2): en exo `type = "entity"` siempre en v1 (contrato §4.1 de la spec del indexer: resultados SIEMPRE a nivel entidad) e `id = permalink` (`notas.permalink`, tal cual — la normalización para comparar arms vive solo en `norm()` de `analyze.py`, spec M2 §4). El canal vector ya llega agregado a entidad (m2-06: mejor trozo por permalink), así que ambos canales producen candidatos con la misma clave. **La fusión combina el candidato FTS y el candidato vector de la MISMA entidad en una sola fila**; una entidad aparece a lo sumo una vez en `results`.

### 4.2 Canales de entrada

- `f_raw(e)` = score FTS de la entidad `e` = `-bm25(notas_fts)` (así lo produce hoy `busca()`; no se cambia).
- `v(e)` = similitud coseno del mejor trozo de `e` (así lo produce hoy `busca_vector()`; no se cambia), sujeta al threshold de admisión (§4.5).
- Pool de candidatos FTS interno: la fusión pide a FTS hasta **K_c = 50** candidatos (constante de implementación, NO parámetro de sweep: FTS cuesta ms y el resultado es insensible a K_c mientras K_c ≫ `limite`; con K_c = `limite` una entidad FTS-rank-6 que el vector también puntúa no podría fusionarse). El canal vector ya es exhaustivo (KNN con `k = COUNT(*)`, decisión m2-06).

### 4.3 Normalización BM25 (componente 4 de §4.2)

Cita: §4.2 "normalización BM25". La fórmula exige que `v` y `f` vivan en escala común; los datos de §3.3 fijan las restricciones: BM25 con spread ×49 entre queries, coseno con techo empírico ≈0.68.

**Decisión D-f1 — normalización por-query con anclaje β**:

```
f(e) = β · f_raw(e) / f_max(q)        con f_max(q) = max f_raw sobre los candidatos FTS de la query
```

- **Por-query** (dividir por `f_max` de la propia query): única forma compatible con el spread ×49 medido — cualquier constante global deja unas queries saturadas y otras invisibles. Monótona (preserva el orden FTS) y acotada a `(0, β]`.
- **β (anclaje del mejor hit FTS)**: sin β, el top-1 FTS valdría siempre 1.0 > 0.6764 (techo vectorial observado) — el canal FTS dominaría el `max(v,f)` por construcción en toda query con resultados FTS, aunque FTS solo acierta 28/55. No hay dato a priori que fije cuánto debe "valer" el mejor hit FTS frente al mejor hit vectorial → **β es parámetro del sweep** (§5), grid {0.6, 0.8, 1.0} (0.6 ≈ paridad con el techo vectorial; 1.0 = normalización pura sin anclaje).
- Degenerados: query con un solo candidato FTS → `f = β` (el max de un elemento es él mismo); `f_max = 0` (teóricamente posible si bm25 devuelve 0) → canal FTS se descarta entero para esa query (todo 0, sin división).
- Alternativas descartadas: min-max por-query (degenerada con 1 resultado: 0/0; y colapsa el peor candidato real a 0), saturante `f/(f+k)` (introduce una constante k de escala sin dato que la fije y sin la propiedad "el mejor FTS de cada query llega al ancla").
- El canal vector NO se re-normaliza: la similitud coseno ya está en escala absoluta comparable entre queries (mismo embedder, vectores unitarios).

### 4.4 Fórmula de fusión (componente 1 de §4.2)

Cita: §4.2 "fórmula `max(v,f)+bonus·min(v,f)`". Para cada entidad del conjunto de candidatos (§4.5), con canal ausente = 0:

```
score(e) = max(v(e), f(e)) + bonus · min(v(e), f(e))
```

- **Canal ausente = 0**: si `e` solo tiene candidato vector, `score = max(v,0) + bonus·0 = v`; si solo FTS, `score = f`. La fórmula degenera limpiamente a "conservar el candidato de un solo canal con su score intacto" — es lo que hace viable la admisión por unión (§4.5) sin casos especiales.
- **bonus**: peso de la confirmación del canal débil. Rango [0, 1] (`bonus = 0` ⇒ fusión = mejor canal, puro max; `bonus > 1` haría dominar al canal débil sobre el fuerte, contradiciendo la semántica de "bonus"). Sin dato que lo fije → **parámetro del sweep** (§5), grid {0.0, 0.1, 0.2, 0.3, 0.5}.
- Codominio: `(0, (1+bonus)·max(β,1)]` — el score fusionado puede superar 1.0 cuando ambos canales confirman. Consistente con "su escala es informativa, no contractual" (contrato §4.1, sellado).
- Orden final: `results` por score fusionado descendente, truncado a `limite` DESPUÉS de fusionar (mismo contrato que los otros tipos).

### 4.5 Gate FTS y conjunto de candidatos (componente 3 de §4.2)

Cita: §4.2 "gate FTS". La prosa no fija su semántica; el brief la señala como pregunta de diseño ("¿candidatos solo si pasan FTS? ¿o unión?"). Dos lecturas posibles:

- **Lectura A — gate de admisión**: solo entra al ranking una entidad presente en los resultados FTS; el vector solo re-puntúa. **Descartada como default por aritmética, no por gusto**: el **techo duro de A es ≤ 41/55** — 15 queries quedan sin NINGÚN resultado FTS (FTS vacío) y **14 de ellas tienen gold** (§3.2), que bajo A nunca pueden acertar (55−14 = 41). El 28/55 (hit@5 FTS puro truncado a 5) es solo la **cota inferior**: con el pool `K_c = 50` (§4.2) y re-rank vectorial, la corrida diagnóstica de A puede subir dentro del rango **28–41**. En ambos extremos A queda bajo el sanity floor de 46 (§6); el argumento de las 14 FTS-vacías-con-gold basta por sí solo para descartarla.
- **Lectura B — gate del bonus (DEFAULT)**: la admisión es la **UNIÓN** de candidatos FTS y vector; el "gate FTS" queda realizado en la propia fórmula: **el término `bonus·min(v,f)` solo es distinto de 0 cuando el candidato pasa FTS** (si no pasa, `f = 0` ⇒ `min = 0`). Es decir: FTS no filtra la entrada, "gatea" el refuerzo.

**Decisión D-f2**: admisión = unión (lectura B). La lectura A se ejecuta UNA vez como **corrida diagnóstica** del sweep (§5) para dejar su hit@5 medido en la atribución — predicción pre-registrada: **28–41/55** (28 cota inferior = FTS puro; 41 techo duro = 55 − 14 FTS-vacías-con-gold) — no como candidata real.

### 4.6 Threshold configurable (componente 5 de §4.2)

Cita: §4.2 "threshold configurable" + spec del indexer §5 (config): "`semantic_min_similarity` | threshold del arm vector/hybrid (hoy 0.35; calibración en M2-07)".

**Decisión D-f3**: el threshold aplica a la **similitud vectorial `v` PRE-fusión** (admisión del canal vector): `v < umbral` ⇒ el candidato vector de esa entidad no existe (`v = 0`); la entidad puede seguir entrando por FTS. Misma semántica que `busca_vector` hoy (`min_similitud_efectivo`): valor de `--min-similitud` si se pasó; si no, `semantic_min_similarity` de `~/.basic-memory/config.json` RO (precedencia flags > config, D6).

- **Re-sweep obligatorio** (spec madre §4 punto 4: "**Re-sweep del threshold** por modelo (el 0.55 no sobrevive al cambio)"): ni se hereda el 0.55, ni se asume el 0.35 del verdict M0 sin re-barrer con la señal del engine — el 0.35 se calibró sobre el pipeline de bm, no sobre esta fusión. Procedimiento en §5.
- Si el valor elegido difiere del 0.35 de config: E1 NO escribe nada (ni en `config.json` — spec indexer §5), así que el valor elegido se pasa por `--min-similitud` en corridas y consumidores, y queda documentado en el verdict del sweep. Config propia de exo = M5a.

### 4.7 CLI y envelope

- `exo search --type hybrid` completa el enum previsto (`TipoBusqueda`); envelope v1 y contrato §4.1 **intactos** (superficie gateada): `search_type: "hybrid"` literal, `results` nivel entidad orden score desc, `score` = fusionado ("el score del `search_type` usado (fusionado en hybrid); su escala es informativa, no contractual" — §4.1).
- Flags nuevos (solo para el sweep y el override; D6 flags > config): `--bonus <f64>` y `--escala-fts <f64>` (β). `--min-similitud` ya existe. Tras el sweep, los valores ganadores se fijan como **defaults constantes en el binario** (documentados con el verdict); no entran en config hasta M5a.
- `--limite` conserva su semántica (truncado post-fusión).

### 4.8 Latencia

hybrid = embed de query + KNN exhaustivo (ya pagados por el canal vector) + un MATCH FTS (~ms) + fusión O(candidatos). Sin presupuesto nuevo: aplica el ya pre-registrado de spec M2 §5 pata 3 ("hybrid en frío con carga de modelo **p95 < 2.0 s**"), que se mide en M2-08/M2-09, no aquí.

## 5. Procedimiento de calibración (sweep)

Mandato: spec M2 §3 fila M2-07 "sweep con procedimiento `analyze.py`"; spec M2 §4 "`analyze.py` intacto"; precedente metodológico pre-registrado en el gate M0 (`evals/retrieval-fase0/gate.md`, "Ajustes de método"): "Sweep de threshold: se hace offline sobre el score fusionado capturado con min_similarity=0.0 (filtrar-luego-truncar top-5). Es una aproximación del efecto del threshold real de config (que aplica a similitud vectorial pre-fusión) — limitación documentada; la comparación entre brazos usa el mismo procedimiento, así que es interna-consistente." Se adopta el MISMO método.

### 5.1 Parámetros del sweep (lo-que-se-calibra, declarado)

| Parámetro | Qué es | Grid | Por qué no hay valor a priori |
|---|---|---|---|
| `bonus` | peso del canal débil en la fórmula (§4.4) | {0.0, 0.1, 0.2, 0.3, 0.5} | §4.2 no da valor; sin dato empírico previo del engine |
| `β` (`--escala-fts`) | anclaje del mejor hit FTS (§4.3) | {0.6, 0.8, 1.0} | el equilibrio FTS-top vs techo vectorial (0.68) no tiene dato que lo fije |
| threshold | admisión del canal vector (§4.6) | post-hoc: {None, 0.35 … 0.65} (los `THRESHOLDS` de `analyze.py`, intacto) | re-sweep por modelo obligatorio (spec madre §4.4) |
| gate FTS | lectura A vs B (§4.5) | B en todo el grid + **1 corrida diagnóstica** A | semántica no fijada por la prosa; A pre-predicha **28–41/55** (28 cota inf., 41 techo duro) |

Total: 15 corridas (grid B) + 1 diagnóstica (A con bonus/β centrales). El sweep es sobre el eval set completo (56 queries); cada corrida son ~minutos (precedente m2-06).

### 5.2 Mecánica

1. **Implementación previa** (executor, fuera de esta spec): `busca_hybrid` en `buscador.rs` según §4; `--type hybrid` en `main.rs`; `replay-engine.py` extendido — `--tipo hybrid` (el `SEARCH_TYPE_MAP` ya lo deja pasar literal, su docstring lo prevé: "hybrid: M2-07") + reenvío de `--min-similitud`, `--bonus`, `--escala-fts` a `exo search`. `analyze.py` NO se toca.
2. **Corridas del sweep**: `replay-engine.py <arm> --tipo hybrid --min-similitud 0.0 --limite 10` por cada config del grid (umbral runtime 0.0 para capturar candidatos completos; el threshold se barre post-hoc con el filtro por score de `analyze.py`, método M0 §arriba; `--limite 10` = misma profundidad que las filas del arm bm, deja margen al filtrar-luego-truncar-a-5).
3. **Métricas**: `analyze.py <arm>` por corrida → headline (thr=None) + columnas thr=0.35…0.65 + atribución de misses. La atribución usa además los arms `engine-fts`/`engine-vector` existentes para clasificar cada miss del hybrid (spec madre §4.4: "**Atribución de cada miss** (FTS-miss / vector-miss / threshold-miss, vía search_type explícito)").
4. **Selección pre-registrada** (para que el sweep no degenere en renegociación): gana la celda (config × thr) con mayor hit@5; empate → menos ROTAS en `analyze.py <arm> jina-es` pareada; empate → config más simple: menor `bonus`, luego mayor `β`, luego mayor thr. La comparación pareada contra los `results/` actuales es **diagnóstico de selección**, no el gate: los números oficiales salen solo de la corrida mismo-día de M2-09 (spec M2 §5: "prohibido comparar contra `results/` de julio" para el gate).
5. **Confirmación nativa**: una corrida final con los valores ganadores pasados por flags reales (`--min-similitud <elegido>`, sin post-hoc). El hit@5 nativo puede diferir del post-hoc por la aproximación documentada (el filtro post-hoc actúa sobre el score fusionado; el umbral real actúa sobre `v` pre-fusión). Divergen en tres clases: (a) el término `bonus·min` de candidatos fusionados cuyo `v` cae bajo el umbral; (b) **candidatos solo-FTS** (`v = 0`): el umbral nativo (solo aplica a `v`, D-f3) NUNCA los toca, pero el filtro post-hoc sobre score fusionado los ELIMINA si `f < thr` (con `f ≤ β ≤ 1.0`, cualquier FTS de rango medio cae bajo thr 0.35–0.65 — el post-hoc threshold-iza de facto también el canal FTS); (c) **re-ranking**: el umbral nativo convierte un fusionado con `v < thr` en solo-FTS (cae a score `f` y se REORDENA), mientras el filtro post-hoc solo trunca, nunca reordena. El número que se sella es el **nativo** (por eso esta corrida es obligatoria, no opcional).
6. **Sellado**: config ganadora (bonus, β, threshold, gate) como defaults del binario + verdict del sweep commiteado. Después de sellar, la config no se retoca contra la corrida del gate M2-09.

**Nota operacional para el runner de M2-09** (el gate.md pre-registrado NO se toca — está congelado): la sección pareada de `analyze.py` (intacto) evalúa `hit()` sin argumento de threshold (`thr=None`) sobre las filas del arm. Por tanto, cuando gate.md exige que "bm-hybrid compite con su mejor threshold del mismo procedimiento de sweep", ese mejor threshold **solo es aplicable a la corrida pareada si se captura NATIVAMENTE** (bm ejecutado con su `min_similarity` en su config), o si el mejor threshold de bm coincide con el sin-filtro. Hoy es moot (bm da 43/55 tanto a `thr=None` como a `thr=0.35` — empate, verificado), pero en la corrida del mismo día de M2-09 puede no serlo: si el mejor thr de bm ≠ sin-filtro, su corrida del gate se captura nativa a ese threshold antes de la pareada. Sin esto, la pareada compararía el engine-hybrid a su mejor config contra un bm degradado a `thr=None` — un sesgo a favor del engine que el gate pareado NO debe tener.

### 5.3 Aproximaciones documentadas del método (heredadas del precedente M0, no defectos nuevos)

- Filtro post-hoc sobre score fusionado ≈ threshold pre-fusión: exacto para candidatos solo-vector; difiere para fusionados (término `bonus·min`), para solo-FTS (el post-hoc los threshold-iza vía `f < thr`, el nativo no toca `f`) y en el re-ranking (el nativo reordena un `v<thr` a solo-FTS, el post-hoc solo trunca) — las tres clases detalladas en §5.2.5; por eso la confirmación nativa de §5.2.5 es obligatoria y su número es el que se sella.
- El sweep selecciona sobre el mismo eval set en que se mide el gate (55 queries). Mitigación pre-registrada, misma que M0: bm-hybrid recibe idéntico trato ("cada brazo en su mejor threshold del sweep (mismo procedimiento para todos los brazos)" — gate M0) y el criterio del gate es pareado, no absoluto. El eval set es además el test de regresión permanente del engine (spec madre §4.1.5) — no hay un held-out disponible en E1 y crearlo sería construir gold nuevo, prohibido (brief contrato 4).

## 6. Gate pre-registrado (fijado en `evals/e1-read/gate.md`, commiteado con esta spec)

Distinción vinculante (blindspot nota 1 del brief):

- **GATE oficial (NO negociable)** — spec M2 §5 pata 2, LITERAL: "engine-hybrid **rompe ≤2 y arregla ≥ las que rompe** vs bm-hybrid (referencia hoy 43/55). Subgrupo observation-sensitive examinado aparte." Comparación pareada (spec madre §4 punto 4: "comparación pareada, no test de proporciones"), ambos arms el mismo día sobre el mismo estado de la KB, commit pineado (spec M2 §5 pata 2). "los números no se renegocian post-hoc" (spec M2 §5).
- **Sanity-check de ingeniería (informativo, NO gate)**: el vector puro ya da 46/55; un hybrid < 46 es una fusión mal calibrada aunque pasase el gate vs bm. Se registra como diagnóstico obligatorio del verdict (con atribución de qué hits vectoriales perdió la fusión), separado del gate.

El texto completo de las tres patas vive en `evals/e1-read/gate.md` (este item fija la pata 2; las patas 1 y 3 son transcripción literal de spec M2 §5 para que el fichero quede completo para M2-09).

## 7. Tests contractuales (a escribir en la implementación — contrato nombrado, no implementado)

1. `fusion_formula_ambos_canales` — con `v`, `f` y `bonus` conocidos, `score == max(v,f) + bonus*min(v,f)` exacto.
2. `fusion_conserva_candidato_solo_vector` — entidad sin fila FTS entra con `score == v` (canal ausente = 0, §4.4).
3. `fusion_conserva_candidato_solo_fts` — dual del anterior: `score == f`.
4. `fusion_gate_fts_no_pierde_hit_semantico` — admisión por unión (D-f2): un candidato vector jamás se descarta por no aparecer en FTS.
5. `fusion_clave_entidad_una_fila_por_permalink` — la misma entidad llegando por ambos canales produce UNA fila fusionada, no dos.
6. `normalizacion_bm25_monotona` — la normalización §4.3 preserva el orden FTS y acota a `(0, β]`; el top-1 FTS vale exactamente β.
7. `normalizacion_bm25_query_sin_fmax` — `f_max == 0` ⇒ canal FTS descartado sin división por cero.
8. `fusion_bonus_cero_es_max` — `bonus = 0` ⇒ `score == max(v,f)`.
9. `threshold_filtra_vector_pre_fusion` — `v < umbral` ⇒ la entidad pierde el canal vector pero conserva el FTS si lo tiene (D-f3).
10. `fusion_orden_desc_truncado_post_fusion` — orden por score fusionado desc y truncado a `limite` DESPUÉS de fusionar.
11. `busqueda_hybrid_envelope` — `search_type: "hybrid"` literal, forma del contrato §4.1 intacta (`{permalink, type: "entity", score}`).

## 8. Riesgos

1. **Dominancia FTS por normalización** (top-1 FTS anclado a β > similitudes vectoriales): es exactamente lo que β barre; si el sweep muestra hits vectoriales desplazados del top-5 por candidatos FTS erróneos, la atribución lo destapa (fusion-miss con vector=HIT).
2. **Sobreajuste del sweep al eval set**: mitigación §5.3 (mismo trato a ambos arms, gate pareado, sin gold nuevo).
3. **Contaminación AGPL**: el diseño usa solo la prosa §4.2 + datos propios; si en la implementación o el gate alguien necesita ver la lógica de fusión de bm, PARA y escala (spec M2 §7.3).
4. **Config divergente tras calibrar** (elegido ≠ 0.35 de config RO): cubierto en §4.6 — flag explícito hasta M5a, documentado en verdict.
