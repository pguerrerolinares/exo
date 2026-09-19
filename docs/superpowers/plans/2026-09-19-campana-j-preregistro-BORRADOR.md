# Pre-registro — Campaña J: retrieval con held-out nuevo (N1, fusión, `archive/`, abstención)

> **Estado: BORRADOR (2026-09-19).** No es contrato hasta el commit de
> congelación (plan fase 1, Task 7), que solo existe cuando el gold de Paul
> existe, ha pasado `valida_gold.py` y su sha256 está en §10. Hasta entonces:
> nadie toca `engine/src` por J, nadie computa hit@k de ningún brazo sobre
> ninguna query del gold nuevo, y este fichero puede cambiar **solo** en tres
> sitios: (1) el tope df de F1 en §4, marcado «T0 puede sustituirlo una vez,
> con evidencia in-sample anotada» (cláusula que Paul dejó intacta al firmar
> D-J8); (2) una línea de §9 **solo** si el review adversarial (plan, Task 5)
> objeta una firma con cita y Paul responde `CAMBIA A`; (3) los campos de
> §10. Las diez decisiones `D-J1..D-J10` de §9 **ya están firmadas por Paul
> el 2026-09-19** (`docs/superpowers/consultas/2026-09-15-campanas/propuesta.md`
> §7). Cualquier otro cambio exige reabrir el borrador por escrito antes de
> que exista el gold.
>
> **Qué se ha observado al redactarlo, y qué no.** Visto: el verdict público
> de C (`evals/retrieval-heldout/verdict/c-verdict.md`, `agregados*.md`), las
> capturas privadas de C **solo en agregado** (distribución de scores, plazas
> de `archive/`, gap top1–top2; ningún texto de query), el in-sample de C
> (43/55 con binario `4f4d2a8` y snapshot `885246d`; 12 misses por id), el
> dry-run del diagnóstico T0 sobre esas capturas (in-sample, descriptivo) y el
> recon de código citado en §2. **No visto:** ninguna query del gold de J
> (no existe), ningún brazo corrido sobre ninguna query fuera de las 55 y del
> held-out consumido de C. El criterio de §6 se fija con esa mitad a ciegas.

## 1. Preguntas

- **Q1 — N1, FTS selectivo.** `prepara_query` (`engine/src/buscador.rs:158-164`)
  une los tokens con AND implícito de FTS5; una frase natural da 0 candidatos
  y el hook corre como vector puro (C: fusión activa en 47/92 held-out, 6/22
  en `prompt`, 0/18 en `hard-larga`). ¿Un canal léxico que **no** exige todos
  los tokens mejora hit@5 sin romper lo que el AND acierta? El OR simple ya
  está descartado con la fusión sellada (`docs/backlog.md`, item N1:
  ARREGLA 1 · ROMPE 4 sobre las 54 del 09-04).
- **Q2 — Operador de fusión.** La familia sellada `max(v, β·f) + bonus·min`
  nunca expresa un CombSUM (`v + β·f`), «la única vía por la que un canal
  léxico arreglado podría sumar en vez de desplazar»; y el barrido que eligió
  CombMAX se corrió **con el FTS conjuntivo** (KB `Backlog — exo`, «El
  operador de fusión, no sus parámetros»). ¿CombSUM gana a la fusión sellada
  sobre el modo FTS que sobreviva a Q1?
- **Q3 — `archive/` penalizado (decisión #4 = c, 2026-09-19).** `archive/`
  es 75/182 notas hoy (41 %); en C ocupó 274/723 plazas del top-5 (38 %) y
  **24/92 `expected_permalink` del gold de C vivían en `archive/`** (más 7
  aceptables). Penalizar, no excluir: ¿un factor único sobre el score de
  `archive/` arregla más de lo que rompe, con el estrato `archive` dentro
  del gold para que la decisión no sea a ciegas?
- **Q4 — Abstención ante corpus negativo.** 54/55 nulas de C devuelven top-5
  con 0,40 (55/55 con 0,35). Pero esas nulas son «por inferibilidad» (la nota
  existe, la query sola no la identifica), no «por ausencia» (§6b del
  verdict): C **no tiene negativos verdaderos**. ¿Un umbral de abstención
  calibrado sobre negativos verdaderos abstiene en ≥ 60 % de ellos sin
  perder más de 2 aciertos en positivos?
- **Fuera de las preguntas, por diseño (§2.2, §2.8, §9):** H28 no es un
  brazo; int8 no entra; D6 (default `hybrid`) se **confirma** con la pareada
  A0 frente a FTS puro como descriptivo, no se decide de nuevo; RRF se
  reporta como testigo (C ya decidió R2: «se queda la fusión actual»).

## 2. Hechos de partida (re-verificados el 2026-09-19; mandan sobre el backlog)

1. **Fusión sellada = CombMAX ponderado.** `score = max(v, 0.6·f/f_max)`:
   `BONUS_SELLADO = 0.0`, `ESCALA_FTS_SELLADA = 0.6`,
   `MIN_SIMILARITY_SELLADO = 0.40` (`engine/src/main.rs:26-39`, D6 de G
   Task 11: el umbral **sí** es constante del binario desde el 2026-09-15 y
   `--type` resuelve a `hybrid` por defecto, `main.rs:251`). `K_C = 50`
   (`buscador.rs:603`). `normaliza_fts` (`buscador.rs:507-522`) y `fusiona`
   (`buscador.rs:529-577`) sin cambios desde C. `recall --query` usa el mismo
   0,40 (`0229540`, G I2).
2. **H28, confirmado por código, no por el backlog.** `vec0` sin
   `distance_metric` usa `VEC0_DISTANCE_METRIC_L2` y el KNN llama a
   `distance_l2_sqr_float` (`sqlite-vec.c:6879`), cuya implementación devuelve
   **`sqrt(res)`** (`sqlite-vec.c:389`; rutas SIMD `:224`, `:263`) pese al
   nombre. `similitud_desde_l2_cuadrado(d) = 1 − d/2` (`buscador.rs:256`)
   asume L2², así que el score vector del binario es `s = 1 − √(2−2·cos)/2`:
   monótono en coseno (**el ranking vectorial no cambia**), acotado en
   `[1−√2/2, 1] = [0,293, 1]` para cos ≥ 0, y el umbral 0,40 equivale a
   **cos = 1 − 2·(1−0,40)² = 0,28** (0,35 ⇔ 0,155; 0,45 ⇔ 0,395). Consistente
   con las capturas de C: 25.578 scores vector con `--min-similarity 0.0`,
   mínimo 0,2906, máximo 0,689 (cos 0,81). **Consecuencia:** β = 0,6 y el
   0,40 están calibrados sobre la escala `s`; pasar la fusión a escala coseno
   cambia `max(v, f)` aunque no cambie el orden de `v`. Por eso H28 **no es
   un brazo**: todos los brazos de §4 se computan sobre la escala `s` que el
   binario emite hoy, y H28 se cierra como corrección de nombre y
   documentación (D-J5), sin cambio de comportamiento.
3. **FTS del motor, replicable offline.** `SELECT permalink, -bm25(notas_fts)
   AS score FROM notas_fts WHERE notas_fts MATCH ?1 ORDER BY score DESC LIMIT
   ?2` (`buscador.rs:~208-212`) sobre `fts5(titulo, cuerpo, permalink
   UNINDEXED, tokenize='unicode61 tokenchars 0x2F')`. El `sqlite3` de Python
   3.12 (SQLite 3.45.1) trae FTS5: `OR`, `NEAR` y `bm25()` verificados en solo
   lectura sobre la copia privada `idx-base.db` de C. **`vec0` no se carga en
   Python** (`no such module: vec0`): la lista vector sigue viniendo de la
   captura del binario, como en C. ⇒ Todo brazo de §4 se calcula **offline
   sobre las mismas capturas**; `engine/src` solo se toca para implementar al
   ganador, después del verdict (plan fase 2).
4. **`archive/` en cifras.** 75/182 notas hoy (32 en `archive/log/`); en las
   capturas de C: 274/723 plazas de top-5 (38 %), 134/147 queries con ≥1 nota
   de `archive/` en su top-5; 24/92 esperadas y 7 aceptables en `archive/`.
   En el in-sample (dry-run T0): 114/275 plazas, y **6 de los 12 misses tienen
   en su top-5 una rotación de la nota esperada** (`archive/log/<misma
   base>-<fechas>`), es decir, la respuesta está y se etiquetó el vivo. La
   auditoría del 2026-09-04 (KB, `archive/log/exo-bitacora-2026-07-17_2026-09-04`,
   entrada del 09-04) marcó 9 SUPERADA + 2 AMBIGUA en las 55 (filas 7, 13, 22,
   23, 24, 25, 27, 29, 30, 34, 38); **7 de esas 11 son miss en S**. Esas 9 no
   sirven como queries del gold nuevo (anti-fuga, son de las 55): el estrato
   `archive` se escribe nuevo.
5. **Abstención: el gap no separa.** Gap top1–top2 del hybrid en C: p10
   0,002 · p50 0,021 · p90 0,126. Una regla por gap abstendría casi siempre.
   La calibración por score del top-1 necesita negativos verdaderos, que no
   existen: por eso el estrato `negativo` y el split calibración/evaluación
   de §4 (G1).
6. **In-sample, tres estados del mismo conjunto (T0, dry-run 2026-09-19).**
   hist 49/55 (6 misses, KB 138 notas, binario del sweep) → S 43/55 (12
   misses, binario `4f4d2a8`, 174 notas): **hit→miss 7, miss→hit 1,
   miss→miss 5** (el «6 de 55» del backlog es neto). Causas en S: 11/12
   `vector-lejos` (la nota está admitida ≥0,40 pero fuera del top-5), 1/12
   bajo umbral, 0/12 `fusion-desplaza`, 2/12 con FTS vacío (AND), 6/12 con
   rotación de la esperada en el top-5, 7/12 marcadas re-verificar. Para F1:
   52/55 queries tienen ≥1 token «raro» (1 ≤ df ≤ ⌈0,25·174⌉ = 44), 11/12 de
   los misses. 17/55 queries tienen FTS vacío con AND. Todo esto es in-sample y
   **no decide nada**; informa §4 (valores únicos) y el kit de gold.
7. **Held-out de C consumido** (`c-verdict.md` §11). Pool nuevo medido el
   2026-09-19 con `pool.py` y ventana `2026-09-13 → 2026-09-20`, excluyendo
   las 55 y las 147 de C (Jaccard ≥ 0,8): **113 `prompt`** (p50 51 chars, 101
   de ≤ 200), **0 `agent-search`** (el `reflex-retrieval-log.jsonl` termina el
   2026-08-17 y los comandos `exo search` de transcripts caen fuera de
   ventana o duplican). 8 notas añadidas a la KB desde el 2026-09-13.
8. **int8 fuera.** `model_quantized.onnx` cambia el embedding (top-4 solapa
   2–4/4 con fp32, KB-exo «Olas 2-4»): sería un segundo índice completo
   (rebuild ≈ 1,5 h en C, `condiciones.md`) para una pregunta de **no
   inferioridad** («no es peor»), que la regla GANA de §6 no puede responder
   (NO GANA ≠ igual; un margen de 3 pp exige N ≫ 100), y cuyo beneficio
   (~590 ms de carga) canibaliza K si K se hace. Si K = no daemon, int8 tiene
   campaña propia con diseño de no inferioridad y gold propio.
9. **Repo público desde el 2026-09-02.** Ningún texto de query ni permalink
   por fila entra en git. Al repo: harness, kit, agregados, sha256.

## 3. Unidad, gold y estratos

- **Unidad de análisis:** una query del gold con `expected_permalink` no
  nulo. Las nulas forman dos corpus negativos distintos (abajo).
- **Schema** (idéntico a C, `source` ampliado):
  `{"id": "j001", "query": str, "source":
  "prompt"|"keyword"|"hard"|"archive"|"negativo", "expected_permalink":
  str|null, "acceptable_permalinks": [str], "notes": str}`.
  `acceptable_permalinks`: máx. 2, distintos de `expected`, existentes en el
  snapshot, cada uno con una frase `aceptable: …` en `notes`
  (`valida_gold.py`). Criterio canon/bitácora literal de
  `evals/retrieval-fase0/verdict/labels.md:70`.
- **Estratos y quién los produce** (tamaños objetivo; el suelo está en §11):

  | `source` | qué es | quién escribe la query | quién etiqueta | objetivo no nulas |
  |---|---|---|---|---|
  | `prompt` | prompts reales que dispararon `recall-inject` desde el 2026-09-13 (pool entero, D-J2 = 113 firmado; orden barajado con semilla `20260913:prompt` de `pool.muestrea`) | ya escritas | agente fresco pre-etiqueta + verificador adversarial; Paul revisa (D-J1 = b) | 20–35 (C: 31 % no nulas) |
  | `keyword` | 2–5 palabras clave, el estilo `agent-search` de las 55, que ya no tiene pool | **Paul** (15) | Paul | 15 |
  | `hard` | paráfrasis sin palabras literales, mitad frase natural / mitad palabras clave, sobre 30 notas al azar del snapshot (semilla `20260919`, sin `archive/`) | subagente fresco que no ve las 55, ni C, ni `evals/` | agente fresco pre-etiqueta + verificador; Paul revisa (D-J1 = b) | 25–30 |
  | `archive` | preguntas cuya respuesta vive **hoy** en `archive/log/` (hechos de episodios rotados: julio–agosto) | **Paul** (12–15) | Paul | 12–15 |
  | `negativo` | temas **ausentes** de la KB (no «difíciles»: ausentes); `expected` null por definición; `notes` dice por qué no hay nota | **Paul** (30) | — | 0 (30 nulas) |

  Objetivo total: **80–95 no nulas**; suelo 60 (§11). Nulas de `prompt` (por
  inferibilidad) = corpus negativo **débil**, solo descriptivo. `negativo` =
  corpus negativo **verdadero**: ids pares → calibración de G1, ids impares →
  evaluación de G1 (§4).
- **Etiquetado sin motor.** Quien etiquete (Paul incluido) trabaja con el
  filesystem del snapshot, Obsidian o grep; **prohibido** `exo search`,
  `kbx`, `~/.exo/`, `evals/`: etiquetar con el motor haría el gold circular a
  favor de A0. Sesgos declarados: Paul conoce el sistema y sus notas (favorece
  a todos los brazos por igual, no a uno); `keyword` y `archive` las escribe
  quien luego juzga (misma persona, distinta semana: se declara, no se
  corrige); citas literales de prompts en bitácoras (S2 de C, `i`).
- **Modo de decisión:** lenient (`{expected} ∪ acceptable`), strict como
  descriptivo (D5 de C heredado; si Paul quiere otro, D-J10).
- **Anti-fuga:** se excluye toda query con forma normalizada igual o Jaccard
  ≥ 0,8 frente a las 55 **y** frente a las 147 de C (`valida_gold.py
  --in-sample` repetible). Los subagentes de `hard` y de verificación no leen
  las 55, ni el gold de C, ni `evals/`, ni `reports/`.

## 4. Brazos (parámetros fijos; ninguno se elige sobre el held-out)

**Un solo índice `base`** (snapshot `S_J`, binario post-G/L, modelo pineado
`8e2d780d…`). Por query se captura, como en C: FTS-AND `--limit 50`, vector
`--limit 1000 --min-similarity 0.0`, hybrid `--limit 10 --min-similarity 0.40
--bonus 0.0 --fts-scale 0.6`; y además, **offline** con `sqlite3` sobre una
copia de `idx-base.db`: la réplica FTS-AND (oráculo) y la lista FTS de F1.
Todos los rankings se calculan offline (`metricas.py` + módulo `brazos.py`
de la fase 2). Scores vector en la escala `s` del binario (§2.2).

| brazo | definición exacta | papel |
|---|---|---|
| **A0** `sellado` | `max(v, 0.6·f/f_max)`, v filtrado a ≥0,40, orden (−score, permalink), top-10 | referencia |
| A1 `vector` · A2 `fts` | vector ≥0,40 · FTS-AND top-k | testigos (D6 se confirma con A0 vs A2) |
| A3 `rrf` | `Σ 1/(60+rango)` sobre FTS-AND y vector ≥0,40 | testigo (C ya decidió) |
| **F1** `fts-raro` | tokens = `query.split()` como `prepara_query`; `df(tok)` = nº de notas con `MATCH '"tok"'`; se conservan los tokens con **1 ≤ df ≤ ⌈0,25·N_notas⌉** (T0 puede sustituir 0,25 una vez, con evidencia in-sample anotada aquí; dry-run: 52/55 queries con ≥1 raro); `MATCH` = raros unidos con `OR`, `-bm25`, `LIMIT 50`; 0 raros ⇒ canal FTS vacío. La fusión es la sellada, idéntica a A0, sobre esa lista | candidato Q1 |
| **S1** `combsum` | `score = v_adm + 0.6·f/f_max`, `v_adm = v` si v ≥ 0,40, si no 0; admisión = unión; orden (−score, permalink); top-10. β = 0,6 heredado, sin afinar | candidato Q2 |
| **P1** `archive-0.90` | score × **0,90** para permalinks que contienen `/archive/`, antes de ordenar (valor único declarado; D-J7) | candidato Q3 |
| **G1** `abstiene-τ` | top-5 vacío si `score(top1) < τ`; **τ = percentil 70 de los scores top-1 del brazo incumbente sobre los `negativo` de ids pares** (calibración); se evalúa sobre `negativo` impares y sobre todas las no nulas | candidato Q4 |

- **Celdas que se calculan** (todas, para el informe): A0, A1, A2, A3, F1,
  S1∘A0, S1∘F1, P1∘{A0, F1, S1∘A0, S1∘F1}, G1∘(incumbente de D-C). Solo las
  cuatro comparaciones de §6 deciden.
- **Oráculos de fidelidad (condición de validez):** (i) la fusión sellada
  offline reproduce la lista `hybrid` del binario en el 100 % de las queries
  (permalinks, orden, |Δscore| ≤ 1e-12), como en C; (ii) la réplica FTS-AND
  offline con `sqlite3` reproduce la lista `fts` capturada del binario en el
  100 % (permalinks, orden, |Δscore| ≤ 1e-9). Si falla una query de cualquiera
  de los dos, no se computa ninguna decisión.
- **Qué NO es un brazo y por qué:** H28 (§2.2); `sellado-cos` (fusión en
  escala coseno con 0,28) es una recalibración de β disfrazada → celda
  descriptiva solo si D-J5 = b; NEAR (sin hipótesis para prosa); normalización
  FTS «por fuerza absoluta» (segunda fusión candidata = quinta comparación;
  entra solo si S1 pierde y hay gold nuevo); int8 (§2.8); solape/late (C).

## 5. Métricas

- **Primaria: hit@5** lenient sobre las no nulas (continuidad con M0/E1/C;
  binaria ⇒ pareada exacta ARREGLA/ROMPE).
- **Secundaria: MRR@10** (Voorhees, TREC-8, 1999) como **veto**, no como
  criterio de adopción.
- **Negativos verdaderos (Q4):** tasa de abstención = fracción de `negativo`
  impares con top-5 vacío, con Wilson 95 %; se reporta también sobre las
  nulas de `prompt` (descriptivo).
- **Descriptivas sin peso:** hit@5 strict; hit@1; por estrato; por
  sub-tipo `hard` corta/larga (S4 de C); queries con fusión activa por modo
  FTS (AND vs raro); plazas de `archive/` en el top-5 por brazo; Wilson;
  McNemar exacto; IC95 bootstrap pareado de ΔMRR@10 (10.000 remuestreos,
  semilla `20260919`).

## 6. Reglas de decisión (fijadas antes de correr)

Notación: en la pareada de un candidato X contra el incumbente I sobre las no
nulas, en modo lenient: **ARREGLA** = X acierta e I falla; **ROMPE** = I acierta
y X falla; **NETO** = ARREGLA − ROMPE.

**Regla GANA (una sola, para D-A, D-B y D-C):** `NETO ≥ 4` **y** `ARREGLA ≥
2·ROMPE` **y** límite superior del IC95 de ΔMRR@10 (X − I) `≥ 0`. El 4 lo
firmó Paul el 2026-09-19 (D-J4; §7: familia de tres decisiones; 3 y 5 fueron
las alternativas descartadas).

**Camino secuencial con incumbente (una comparación por decisión, orden
pre-declarado; la config que llega a producción es siempre una celda medida
directamente contra su predecesora, nunca una combinación no medida):**

- **D-A (Q1):** F1 vs A0. Si GANA, incumbente `I_A = F1`; si no, `I_A = A0`.
- **D-B (Q2):** S1∘I_A vs I_A. Si GANA, `I_B = S1∘I_A`; si no, `I_B = I_A`.
- **D-C (Q3):** P1∘I_B vs I_B, sobre **todas** las no nulas (el estrato
  `archive` dentro: ahí es donde P1 rompe; fuera es donde arregla). Se reporta
  por estrato, sin decidir por estrato. Si GANA, `I_C = P1∘I_B`; si no,
  `I_C = I_B`.
- **D-D (Q4):** G1∘I_C vs I_C. G1 solo quita resultados (ARREGLA = 0 por
  construcción), así que no usa GANA: **se adopta si ROMPE ≤ 2 sobre las no
  nulas y abstiene en ≥ 60 % de los `negativo` de evaluación** (impares). Si
  hay menos de 12 negativos de evaluación, D-D no se decide (§11).
- **Orden, justificado:** Q1 primero porque cambia el conjunto de candidatos
  y es la causa raíz identificada (N1); Q2 después porque el operador se
  evalúa sobre el canal léxico que sobreviva; Q3 es un filtro sobre el
  ranking resultante; Q4 solo resta y se mide con otra métrica. Cada paso
  compara una celda con su predecesora: cuatro decisiones, no ocho brazos
  contra A0. Las celdas «cruzadas» (p. ej. S1∘F1 si D-A no adoptó F1) se
  reportan descriptivas y **no** deciden: elegir la mejor celda de la tabla
  sería seleccionar sobre el held-out.
- **Si nada gana, el resultado válido es «A0 sellado se queda»**: D6 queda
  confirmado, `buscador.rs` no se toca, los ítems N1 / fusión / `archive/` /
  abstención del backlog se cierran como «medido con held-out J, no
  adoptado» con las cifras, y ninguno se reabre sin un gold nuevo. NO GANA no
  significa «son iguales» (§7).
- **Empate o ambigüedad:** lo adjudica un fable fresco con este texto delante
  y cita textual. Los números no se renegocian.

## 7. Tamaño, potencia y multiplicidad

**Qué puede ver la regla** (cálculo exacto multinomial sobre (ARREGLA, ROMPE),
sin veto MRR, script de recon 2026-09-19; `a` = P(X acierta, I falla), `b` =
al revés; «igual, disc. d» = a = b = d/2):

| escenario verdadero | N=60 ≥3 | N=60 ≥4 | N=60 ≥5 | N=80 ≥3 | N=80 ≥4 | N=80 ≥5 | N=100 ≥3 | N=100 ≥4 | N=100 ≥5 | McNemar α=0,05 N=100 |
|---|---|---|---|---|---|---|---|---|---|---|
| igual, disc. 5 % | 0,07 | 0,02 | 0,01 | 0,10 | 0,04 | 0,01 | 0,13 | 0,06 | 0,02 | 0,00 |
| igual, disc. 10 % | 0,15 | 0,07 | 0,03 | 0,17 | 0,10 | 0,05 | 0,16 | 0,12 | 0,07 | 0,01 |
| igual, disc. 16 % | 0,17 | 0,12 | 0,07 | 0,14 | 0,13 | 0,09 | 0,11 | 0,11 | 0,10 | 0,01 |
| peor 5 pp | 0,01 | 0,00 | 0,00 | 0,01 | 0,00 | 0,00 | 0,00 | 0,00 | 0,00 | 0,00 |
| mejor 3 pp (a=0,06, b=0,03) | 0,37 | 0,22 | 0,12 | 0,46 | 0,33 | 0,21 | 0,51 | 0,42 | 0,30 | 0,09 |
| mejor 5 pp (a=0,075, b=0,025) | 0,57 | 0,41 | 0,26 | 0,68 | 0,56 | 0,42 | 0,74 | 0,67 | 0,55 | 0,24 |
| mejor 8 pp (a=0,10, b=0,02) | 0,81 | 0,68 | 0,53 | 0,89 | 0,83 | 0,73 | 0,93 | 0,91 | 0,85 | 0,56 |
| mejor 10 pp (a=0,13, b=0,03) | 0,86 | 0,79 | 0,69 | 0,91 | 0,89 | 0,84 | 0,93 | 0,93 | 0,91 | 0,66 |

(Todas las celdas «≥k» son `NETO ≥ k ∧ ARREGLA ≥ 2·ROMPE`. Reproduce las
cifras de C para ≥3: 0,07/0,13 · 0,15/0,16 · 0,81/0,93; la errata de C en la
fila «+5 pp» está corregida aquí con el supuesto declarado.)

**Multiplicidad — el challenge al esbozo.** El esbozo proponía 8 brazos con
`NETO ≥ 3` sobre un solo gold. Con k decisiones independientes y un candidato
«igual» en cada una (disc. 10 %, N=100), la probabilidad de adoptar **al
menos un** cambio inútil es `1 − (1−p)^k`:

| regla | p por decisión | k=2 | k=3 | k=4 | k=8 |
|---|---|---|---|---|---|
| NETO ≥ 3 | 0,163 | 0,30 | 0,41 | 0,51 | **0,76** |
| NETO ≥ 4 | 0,121 | 0,23 | **0,32** | 0,40 | 0,64 |
| NETO ≥ 5 | 0,075 | 0,14 | 0,21 | 0,27 | 0,46 |

Ocho brazos con NETO ≥ 3 = tres de cada cuatro campañas adoptarían algo
inútil. Es la cota bajo independencia; los brazos comparten los misses de A0,
así que la tasa real es menor, pero no se sabe cuánto. Decisión de diseño:

1. **Menos decisiones, no corrección:** cuatro (§6), tres con GANA. Una
   corrección tipo Bonferroni sobre una regla de decisión (no un test) solo
   equivaldría a subir NETO; y partir el gold en cribado/confirmación
   (Carterette, TOIS 2012, sobre multiplicidad en IR; Boytsov, Belova &
   Westfall, SIGIR 2013) reduce N a la mitad y la potencia ante +5 pp a ≈0,4:
   descartado. Se elige el **camino secuencial pre-declarado** (§6), que es la
   forma de «hipótesis a priori ordenadas» de Maurer, Hothorn & Lehmacher
   (1995) y Westfall & Krishen (JSPI, 2001) aplicada a reglas de decisión:
   cada paso compara con el incumbente y no hay selección entre celdas.
2. **NETO ≥ 4** (D-J4, firmado 2026-09-19): con k=3 la cota familiar baja de
   0,41 a 0,32 y la potencia ante +8 pp sigue en 0,83–0,91 (N=80–100). NETO
   ≥ 5 bajaría la cota a 0,21 pagando +5 pp → 0,42–0,55. `ARREGLA ≥ 2·ROMPE` se
   mantiene (escala con N; C §7 explica por qué no un tope absoluto).
3. **N objetivo 80–95** (§3): potencia ante +5 pp 0,56–0,67 con NETO ≥ 4.
   Una diferencia de 3 pp es invisible con cualquier N etiquetable (0,33–0,42):
   la regla es una **regla de decisión con tasas de error declaradas**, no una
   afirmación de significación (Webber, Moffat & Zobel, CIKM 2008; Smucker,
   Allan & Carterette, CIKM 2007, sobre por qué bootstrap pareado y no
   sign/Wilcoxon). Sobre tamaño de conjunto de topics: Voorhees & Buckley,
   SIGIR 2002; Sakai, Information Retrieval Journal 2016 («Topic set size
   design»).
4. **D-D (abstención)** no entra en la familia GANA: métrica distinta y guard
   asimétrico. Con 15 negativos de evaluación, 9/15 (60 %) da Wilson
   [0,36, 0,80]; el guard `ROMPE ≤ 2` sobre ~65 aciertos se cumple con
   probabilidad 0,97 si la pérdida real por fila es 1 %, 0,69 si es 3 %, 0,36
   si es 5 %: G1 solo se adopta si casi no cuesta.

**Confianza en las referencias** (Paul valida con papers; se dice lo que se
sabe): Cormack, Clarke & Büttcher SIGIR 2009 (RRF); Fox & Shaw TREC-2 1994
(CombSUM/CombMAX); Lee SIGIR 1997 (CombSUM/CombMNZ > CombMAX); Voorhees &
Buckley SIGIR 2002; Smucker et al. CIKM 2007; Webber et al. CIKM 2008;
McNemar 1947; Wilson 1927; Efron & Tibshirani 1993 — **seguras** (las de C ya
fueron verificadas). Carterette ACM TOIS 30(1) 2012 «Multiple testing in
statistical analysis of systems-based IR experiments»; Boytsov, Belova &
Westfall SIGIR 2013 «Deciding on an adjustment for multiplicity in IR
experiments»; Sakai IRJ 19(3) 2016 «Topic set size design»; Maurer, Hothorn
& Lehmacher 1995; Westfall & Krishen JSPI 99 (2001) — **de memoria, sin DOI
comprobado en esta sesión**: verificar antes de congelar; si alguna no existe
tal cual, se retira sin que cambie ninguna regla (ninguna regla depende de
ellas: dependen de la tabla).

**Protocolo de latencia:** no aplica en J (ningún brazo cambia el índice ni el
KNN; F1 añade una consulta FTS más por query, ≤ ms). Se mide en la fase 2 solo
si el ganador se implementa, como no-regresión (`hyperfine`, mismo protocolo
que C §7).

## 8. Privacidad y artefactos

- **Directorio privado:** `PRIV_J=~/.local/share/exo-evals/j-heldout`
  (`chmod 700`). Contiene `gold-j.jsonl` (`chmod 444` tras congelar),
  `kb-snap/` (snapshot `S_J`), `muestra-prompt.jsonl`, `hard-candidatas.jsonl`,
  `cap-*.jsonl`, `idx-base.db`, `diagnostico-55-detalle.md`,
  `gold-verificacion.md`, `detalle.jsonl`. Se reutiliza en lectura
  `PRIV_C=~/.local/share/exo-evals/c-heldout` (in-sample, capturas de C).
- **Al repo** (`evals/retrieval-heldout/`): `harness/diagnostico.py`,
  extensiones de `valida_gold.py` y `pool.py`, `kit-gold-j/`,
  `verdict/diagnostico-55.md`, `verdict/gold-j-verificacion-resumen.md`,
  y en la fase 2 `verdict/j-agregados.md` y `verdict/j-verdict.md`. Ningún
  texto de query ni permalink por fila. El gold **no** va al repo ni
  gitignored: vive fuera del árbol, como en C (un `.gitignore` es una
  promesa; un directorio fuera del repo no necesita promesa).

## 9. Decisiones de Paul (firmadas el 2026-09-19)

Firmadas en sesión, una a una, cada una con la recomendación del
planificador. Fuente: `docs/superpowers/consultas/2026-09-15-campanas/propuesta.md`
§7. Opciones descartadas y trade-offs en el plan, §«Decisiones firmadas por
Paul 2026-09-19». Solo cambian si el review adversarial (plan, Task 5) objeta
con cita y Paul responde `CAMBIA A`.

- `D-J1` modo de etiquetado: `b — agente pre-etiqueta prompt+hard, verificador adversarial, Paul escribe keyword/archive/negativo y revisa todo (≈ 3,5–4 h)` (descartadas: a, c)
- `D-J2` tamaño del estrato `prompt`: `113 (pool entero)` (descartada: 60)
- `D-J3` verificación adversarial del gold antes de congelar: `sí`
- `D-J4` regla GANA: `NETO ≥ 4` (`ARREGLA ≥ 2·ROMPE` y veto ΔMRR) (descartadas: 3, 5)
- `D-J5` H28: `a — corrección de nombre y doc, escala s se queda, sin brazo` (descartada: b, celda sellado-cos)
- `D-J6` int8: `fuera` (descartada: dentro si K = no daemon)
- `D-J7` factor de P1: `0,90`
- `D-J8` tope df de F1: `0,25, con la cláusula de T0 intacta` (T0 puede sustituirlo una vez, con evidencia in-sample anotada como enmienda)
- `D-J9` abstención G1: `tal cual — percentil 70 de calibración; adopta si abstiene ≥ 60 % de evaluación y ROMPE ≤ 2`
- `D-J10` modo de relevancia: `lenient decide, strict descriptivo (D5 de C)` (descartada: strict decide)

## 10. Congelación

- Commit de la KB para el snapshot `S_J`: `<sha de $PRIV_J/kb-snap.commit>`
- Binario de medición: `<commit de main post-G/L; se anota en verdict/j-condiciones.md en la fase 2, no aquí>`, `cargo build --release --locked`
- `sha256(gold-j.jsonl)`: `<salida de valida_gold.py>`
- Filas: `<total>` · `<no nulas>` · por estrato `prompt <n> · keyword <n> · hard <n> · archive <n>` · nulas `<prompt n · negativo n>` · con acceptable `<n>`
- Aprobación de Paul del gold: línea `GATE: GOLD-J APROBADO <fecha>` en `.superpowers/fabrica/packages/j-gold.md` §Firma
- Commit de congelación: el que introduce este bloque relleno y cambia la cabecera a `CONGELADO`.

## 11. Circuit breakers y kill-criteria

- **Gold verificado con < 60 no nulas:** STOP y PENDIENTE-PAUL; no se
  inventan queries. Con < 12 `negativo` de evaluación (impares), D-D no se
  decide (se reporta). Con < 8 filas `archive`, D-C no decide (se reporta por
  estrato como descriptivo). Con < 15 filas `prompt` no nulas, el estrato se
  reporta sin que cambie la regla (la decisión es sobre el total).
- **Fidelidad ≠ 100 %** en cualquiera de los dos oráculos (§4): STOP, se
  diagnostica el harness o el binario; tope 2 reintentos; al tercero,
  PENDIENTE-PAUL. No se mide ni se decide.
- **Gold modificado tras congelar** (el sha256 no casa): medición inválida.
- **Error de captura** en una query: se reintenta la captura completa, tope
  2; error persistente = fallo en todos los brazos de esa query, declarado.
- **Implementación del ganador (fase 2):** antes de sellar, el binario con el
  ganador debe reproducir la celda offline ganadora en el 100 % de las queries
  del gold (mismo oráculo (i)). Si no, no se sella: se vuelve a la celda y se
  diagnostica.
- **Prohibido:** afinar cualquier parámetro sobre el held-out, añadir brazos
  o comparaciones después de congelar, re-etiquetar a la vista de resultados,
  decidir por estrato o por celda cruzada. Tras el verdict, el gold de J
  queda **consumido**: cualquier ajuste posterior (segunda fusión candidata,
  otro factor de `archive/`, otro τ) necesita un gold nuevo.
- **Si el ganador de D-A/D-B exige reescribir la parada del KNN** (H29,
  `busca_vector_con_embedding`: la cota `limite` solo vale con `bonus == 0`
  y `max`), la fase 2 lo trata como en C (fila PR #12 del plan de C): lista
  vectorial exhaustiva o cálculo de `v` para los ≤50 candidatos FTS por
  `permalink IN (…)`, con test de equivalencia exacta antes de mergear.
