# Pre-registro — Campaña C: retrieval fuera de muestra (H7, H7b, H14, H24)

> **Estado: BORRADOR hasta el commit de congelación (Task 5 del plan
> `2026-09-13-campana-c-retrieval-held-out.md`).** Lo que hace valer este
> documento es `git`, no la prosa. Primero, Paul fija las decisiones marcadas
> `PENDIENTE-PAUL` en §9. Después se aprueba el gold y la Task 5 escribe su
> sha256 en §10 y commitea. Ese commit es el pre-registro. Desde ese momento
> el fichero no se edita. Una errata descubierta después se anota en el
> verdict y no aquí.
>
> **Qué se ha observado al redactarlo (2026-09-13), y qué no.** Visto: los
> resultados in-sample de M2-07 y M2-09 sobre las 55 queries
> (`evals/retrieval-fase0/results/metrics-engine-hybrid-*.md`,
> `evals/e1-read/verdict/m2-09-corrida.md`). Visto también el recon de código
> citado abajo. **No visto:** el held-out, que todavía no existe; ningún brazo
> corrido sobre él; RRF, solape y late chunking corridos sobre ningún conjunto,
> ni siquiera las 55. El criterio de §6 se fija con esa mitad a ciegas.
>
> **Observado después, antes de fijar D1–D3 (2026-09-13, recon de diseño sin
> motor):** solo recuentos del pool filtrado (agent-search 39, prompt 72, de
> ellos 23 operativos por heurística y 46 de 49 temáticos en frase natural),
> 36 notas añadidas a la KB desde 2026-08-17 y la distribución de tamaño de
> las 174 notas (mediana 7.180 B; 71 superan ~2048 tokens). Ningún texto de
> query leído por el orquestador, ningún brazo corrido sobre el pool.

## 1. Preguntas

- **H7.** ¿Generaliza el hybrid sellado (`bonus=0.0`, `β=0.6`, umbral
  `0.40`) fuera de las 55 queries sobre las que se eligió?
- **H7b.** ¿Iguala o supera Reciprocal Rank Fusion (Cormack, Clarke &
  Büttcher, SIGIR 2009, `k=60`) a la fusión actual, con menos hiperparámetros?
- **H14.** ¿Mejora el retrieval respecto al corte duro de 900 caracteres sin
  solape (a) con solape en los cortes duros o (b) con late chunking
  (Günther et al., 2024, arXiv:2409.04701)? No son la misma intervención:
  (a) cambia los trozos, (b) mantiene los mismos trozos y cambia solo el
  contexto con el que se embeben.
- **H24.** El gold admite `acceptable_permalinks` secundarios desde el
  diseño. Queda por decidir si se aplica como overlay a la fila 13 de las 55,
  sin tocar `evals/retrieval-fase0/gate.md`.

## 2. Hechos de partida (re-verificados el 2026-09-13; mandan sobre el backlog)

1. **La fusión sellada es CombMAX ponderado.** `fusiona`
   (`engine/src/buscador.rs:380-414`) calcula
   `score = max(v,f) + bonus·min(v,f)`. Con `BONUS_SELLADO = 0.0`
   (`engine/src/main.rs:24`) queda en `score = max(v, 0.6·f_raw/f_max)`
   (`normaliza_fts`, `buscador.rs:358-372`; `ESCALA_FTS_SELLADA = 0.6`,
   `main.rs:25`). Es el CombMAX de Fox & Shaw (TREC-2, 1994) con la escala FTS
   como único peso. En la práctica los hiperparámetros efectivos son dos, β y
   el umbral, más la constante `K_C = 50` (`buscador.rs:441`).
2. **El umbral 0.40 no está en el binario ni en la config.** Lo pasa el hook
   (`plugins/exo/scripts/recall-inject.sh:145`, `--min-similarity 0.40`). La
   config de la máquina dice `min_similarity = 0.35`, y ese es también el
   default de `exo init` (`main.rs:588`). `exo search --type hybrid` sin el
   flag usa 0.35. En producción conviven dos umbrales.
3. **La rejilla in-sample era plana.** Las 15 celdas
   bonus{0,0.1,0.2,0.3,0.5} × β{0.6,0.8,1.0} dan entre **47/55 y 49/55**. Con
   la celda ganadora, los umbrales 0.35 y 0.40 empatan a 49/55 y 0.45 cae a
   46/55 (`metrics-engine-hybrid-b*-e*.md`). El optimismo atribuible a haber
   elegido bonus y β sobre esas 55 es, como mucho, de 2 queries. El riesgo real
   de H7 no es el ajuste fino: es la generalización del diseño completo
   (fórmula, `K_C`, umbral, troceado) a otras queries y a una KB que ha pasado
   de 138 a 174 notas.
4. **`exo recall --query=<prompt>` funciona en la práctica como vector puro.**
   `prepara_query` (`buscador.rs:147-153`) une los tokens con AND implícito de
   FTS5, así que una frase natural no encuentra nada por FTS. Reproducido en
   solo lectura contra el índice real: `exo search --type fts --limit 50`
   devuelve **29** candidatos para `fabrica campaña` y **0** para
   `cómo decidimos el umbral de similitud del recall y por qué quedó en 0.40`.
   Consecuencia de diseño: **la fusión solo puede cambiar el ranking en las
   queries con al menos un candidato FTS**. El hook de memoria depende casi
   solo del arm vector y del troceado. `exo search` con palabras clave, que es
   lo que usan los agentes, sí ejercita la fusión.
5. **Troceado.** `MAX_CHARS = 900` (`engine/src/trozos.rs:9`); `corta_duro`
   (`trozos.rs:100-106`) no solapa. Estimación con un script de recon que
   replica `bloques_markdown` sobre la KB actual (174 notas, no es el
   binario): 4.565 bloques, de los que 656 (14,4%) superan 900 caracteres, y
   **~50% de los ~3.290 trozos salen de un corte duro**. El modelo admite 8192
   posiciones (`config.json`: `max_position_embeddings: 8192`), pero el
   pipeline trunca a **512 tokens**: `tokenizer_config.json` del snapshot
   pineado declara `model_max_length: 512`, y fastembed 5.17.3 aplica
   `max_length.min(model_max_length)` (`fastembed-5.17.3/src/common.rs:97`).
   Hoy no muerde, porque un trozo de 900 caracteres ronda los 225–300 tokens.
   Un prompt largo como query sí se trunca.
6. **Viabilidad de late chunking: VIABLE con la API pública de fastembed
   5.17.3, sin `ort` directo.** Evidencia en la crate
   (`~/.cargo/registry/src/index.crates.io-*/`):
   - `TextEmbedding::transform` es `pub` y no hace pooling
     (`fastembed-5.17.3/src/text_embedding/impl.rs:322`).
   - `SingleBatchOutput::select_output` devuelve el tensor
     `[batch, seq, 768]` (`src/output/embedding_output.rs:22`), y
     `EmbeddingOutput::into_raw` es público (`:95`).
   - `pub tokenizer: Tokenizer` (`src/text_embedding/init.rs:142`) da acceso a
     `encode` con offsets (`tokenizers-0.22.2/src/tokenizer/encoding.rs:159`)
     y a `get_truncation_mut` (`tokenizer/mod.rs:642`) para abrir la ventana
     por encima de 512 sin tocar la config.
   - El ONNX cacheado expone `last_hidden_state` y no lleva pooling horneado.
   - Límites reales: la atención crece con L². A 8192 tokens, una matriz
     float32 de 12 cabezas ocupa ~3,2 GB (estimación), en una máquina de 15 GB.
     Por eso la ventana se fija en **2048 tokens** (§4, brazo `late`), que es
     "long late chunking" por macro-ventanas y no el documento entero.
   - La caché de embeddings del indexer va por texto de trozo
     (`engine/src/indexer.rs:318-347`), algo incorrecto bajo late chunking.
     Para el experimento no importa, porque se reconstruye desde cero. En
     producción obligaría a rediseñar la caché.
7. **El conjunto in-sample no está en el árbol de `main`.** `eval.jsonl` (56
   filas, 55 etiquetadas, 32 permalinks únicos; campos `query`, `source`
   `log|hard`, `expected_permalink`, `notes`) solo existe en la rama local
   `archivo/main-pre-reescritura` (commit `f847ce1`). Los 32 permalinks siguen
   existiendo en la KB a 2026-09-13 (frontmatter de
   `~/Documentos/proyectos/wisdom-paul`, HEAD `7f4aa22`).
8. **El repo es público** desde el 2026-09-02. El held-out contiene prompts
   de Paul y consultas sobre su KB: **nada por query entra en el repo**. Solo
   se commitean agregados, el sha256 del gold y el harness (§8).

## 3. Unidad, gold y H24

- **Unidad de análisis:** una query del held-out con `expected_permalink` no
  nulo. Las filas `null` forman el corpus negativo: se conservan, se reportan
  aparte y no entran en ninguna métrica de decisión.
- **Schema del gold** (una línea JSON por query):
  `{"id": "c001", "query": str, "source": "prompt"|"agent-search"|"hard",
  "expected_permalink": str|null, "acceptable_permalinks": [str],
  "notes": str}`.
- **`acceptable_permalinks` (H24):**
  - Como mucho 2 por fila, todos distintos de `expected_permalink`, todos
    existentes en el snapshot. Vacío si la fila es `null`.
  - Cada uno exige en `notes` una frase que empiece por `aceptable:` y diga
    por qué un usuario razonable quedaría servido. Es el criterio de
    `evals/retrieval-fase0/verdict/labels.md:33`, fila 13: "un retriever que
    devuelva cge-bitacora sería castigado siendo razonablemente correcto".
  - Canon frente a bitácora, criterio de `labels.md:70`: "query
    genérica/estado → canon; query histórica/detalle fechado → bitácora".
- **Relevantes de una fila:** en modo *lenient*,
  `{expected} ∪ acceptable_permalinks`; en modo *strict*, solo `{expected}`.
  El modo de decisión lo fija D5 (§9).
- **Overlay de la fila 13 (in-sample, solo para reportar):** un fichero
  privado de una línea,
  `{"query": "cge evaluación head-to-head cgeo benchmark harness metodología",
  "acceptable_permalinks": ["wisdom-paul/log/cge-bitacora"]}`. Las 55 se
  reportan con y sin overlay. `evals/retrieval-fase0/gate.md` y los números
  históricos de M0/M2 no se tocan.

## 4. Brazos (parámetros fijos; ninguno se elige sobre el held-out)

Todas las capturas usan el mismo binario (post-campaña A), el mismo snapshot
de KB (commit `S`, §10) y el mismo modelo pineado (revisión
`8e2d780d8fd38f81ca9123ee28e4c5a968aaf21e`). Por cada query e índice se
captura: FTS `--limit 50` (= `K_C`), vector `--limit 1000
--min-similarity 0.0` y hybrid `--limit 10 --min-similarity 0.40 --bonus 0.0
--fts-scale 0.6`. Los rankings se calculan offline a partir de esas listas
(`evals/retrieval-heldout/harness/metricas.py`).

| Brazo | Índice | Ranking |
|---|---|---|
| `base:sellado` (**A0**, referencia) | base | `max(v, 0.6·f/f_max) + 0.0·min`, v filtrado a ≥0.40, orden (−score, permalink) |
| `base:vector` (A1, testigo) | base | vector filtrado a ≥0.40 |
| `base:fts` (A2, testigo) | base | FTS top-k tal cual |
| `base:rrf` (**A3**, H7b) | base | `Σ 1/(60 + rango)` sobre la lista FTS (≤50) y la vector filtrada a ≥0.40; orden (−score, permalink) |
| `base:sellado-035` (descriptivo) | base | A0 con umbral 0.35 (el default de config, §2.2) |
| `solape:sellado` (**A4**, H14a) | solape | A0 sobre el índice con solape |
| `solape:rrf` (descriptivo) | solape | A3 sobre el índice con solape |
| `late:sellado` (**A5**, H14b, si D3 = sí) | late | A0 sobre el índice late |
| `late:rrf` (descriptivo) | late | A3 sobre el índice late |

- **Índice `solape`:** `corta_duro` con ventana de 900 caracteres y **paso
  de 720** (solape de 180 caracteres, el 20%) solo dentro de los bloques que
  superan 900. El empaquetado greedy de bloques no cambia. El 20% no se ha
  afinado: es un valor único declarado aquí, sin literatura que lo imponga.
  Al ser un solo valor, no aporta optimismo de selección.
- **Índice `late`:** mismos trozos que `base`. Cada nota se agrupa en
  macro-ventanas greedy de trozos consecutivos hasta 2048 tokens (menos 16 de
  margen), sin solape entre ventanas. Se hace una pasada del modelo por
  ventana, mean pooling del `last_hidden_state` sobre los tokens de cada trozo
  (se excluyen los especiales, con offset vacío) y normalización L2. La query
  se embebe igual que en `base` (truncado a 512).
- **Oráculo de fidelidad (condición de validez de toda la medición):** en el
  índice `base`, el ranking offline de `sellado` tiene que reproducir
  **exactamente** la lista `hybrid` del binario en el 100% de las queries
  (in-sample y held-out): mismos permalinks, mismo orden y |Δscore| ≤ 1e-12.
  Si falla una sola query, no se computa ninguna decisión.

## 5. Métricas

- **Primaria: hit@5** (≥1 relevante en el top-5).
  - Da continuidad directa con los gates de M0 y E1 (`gate.md`,
    `e1-read/gate.md`) y con el 48/55 que se discute.
  - Con un único relevante por query (más como mucho 2 aceptables), es la
    pregunta que se hace el consumidor: "¿está la nota en lo que me inyectan?".
    El hook pide 4 (`recall-inject.sh:145`) y filtra el core-index, así que
    k=5 es la cota de continuidad y no la k exacta de producción.
  - Al ser binaria, admite comparación pareada exacta (ARREGLA/ROMPE).
- **Secundaria: MRR@10** (Voorhees, TREC-8 QA Track Report, 1999): premia
  subir la nota dentro del top. Es el término que distingue dos fusiones con
  el mismo hit@5 pero distinto orden. Sirve de veto (§6) y no de criterio de
  adopción.
- **Por qué no nDCG** (Järvelin & Kekäläinen, ACM TOIS 2002): su ventaja es
  la relevancia graduada. Aquí la relevancia es binaria con un solo relevante
  primario. nDCG@k se reduciría a `1/log2(rango+1)`, un MRR con otro
  descuento, y graduar los aceptables (p. ej. 0,5) inventaría una escala sin
  base.
- **Descriptivas, sin peso en la decisión:**
  - hit@5 en el modo que D5 no elija;
  - hit@1;
  - hit@5 por estrato (`source`);
  - número de queries con fusión activa (≥1 candidato FTS en `base`);
  - en el corpus negativo, la fracción con ≥1 resultado en el top-5 de cada
    brazo;
  - intervalo de Wilson al 95% (Wilson, JASA 1927) del hit@5 de cada brazo;
  - p exacto de McNemar (McNemar, Psychometrika 1947, forma binomial exacta);
  - IC95 bootstrap pareado de ΔMRR@10 (Efron & Tibshirani, 1993; es el test
    que recomiendan Smucker, Allan & Carterette, CIKM 2007, frente a sign test
    y Wilcoxon en evaluación de IR). 10.000 remuestreos, semilla `20260913`.

## 6. Reglas de decisión (fijadas antes de correr)

Notación: en la pareada de un candidato X contra A0 sobre las queries no
nulas del held-out, en el modo de relevancia de D5, **ARREGLA** = X acierta y
A0 falla; **ROMPE** = A0 acierta y X falla. **NETO** = ARREGLA − ROMPE.

**Regla GANA (una sola, para todos los candidatos):**
`NETO ≥ 3` **y** `ARREGLA ≥ 2·ROMPE` **y** el límite superior del IC95 de
ΔMRR@10 (X − A0) es `≥ 0`, es decir, que MRR no empeore con claridad. Los
valores 3 y 2 son la recomendación de D4 (§9) y se sustituyen aquí, en la
congelación, si Paul elige otros.

- **R1 — H7 (¿generaliza el sellado?).**
  - **GENERALIZA** si A0 no pierde contra ninguno de sus componentes: en las
    pareadas A0 frente a A1 y A0 frente a A2, `ARREGLA(A0) ≥ ROMPE(A0)`.
  - **NO GENERALIZA** si A1 o A2 cumplen la regla GANA contra A0: un
    componente sin fusión es claramente mejor fuera de muestra.
  - Cualquier otro caso es **INDETERMINADO**.
  - El nivel absoluto del hit@5 de A0 en el held-out se reporta con su Wilson
    y **no se compara con 48/55**: la fuente de las queries cambia (§7) y la
    diferencia mezclaría sobreajuste con cambio de distribución.
  - Descriptivo: el NETO de A0 frente a A3 in-sample y held-out. Si la
    ventaja de A0 sobre una fusión sin afinar existe solo in-sample, esa es la
    firma del sobreajuste.
  - Un NO GENERALIZA **no cambia producción** en esta campaña. Genera un item
    de backlog y una entrada PENDIENTE-PAUL.
- **R2 — H7b (fusión).** Se adopta RRF **solo si A3 cumple GANA contra A0**
  en el held-out. Si no, se queda la fusión actual. RRF y A0 comparten la
  misma admisión de candidatos (el umbral se aplica antes de fusionar): solo
  difieren en el orden, y solo en las queries con fusión activa (§2.4). Las
  55 in-sample se reportan como pareada **sesgada en contra de RRF**, porque
  A0 se eligió sobre ellas, y no deciden.
- **R3 — H14a (solape).** El troceado con solape se adopta si A4 cumple GANA
  contra A0 **y** el p95 de latencia de `exo search --type hybrid` sobre su
  índice es ≤ 1,25× el de `base` (§7, protocolo de latencia). Motivo del
  guard: más trozos alargan el KNN exhaustivo (`buscador.rs:286`) y el hook
  ya paga ~1 s por turno.
- **R4 — H14b (late chunking, si D3 = sí).** A5 cumple GANA contra A0 **y**
  el rebuild completo del índice `late` termina sin OOM en ≤ 3× el tiempo del
  rebuild `base`. Aunque R4 pase, **late chunking no se implementa en
  producción en esta campaña**: exige otra clave de caché y otra forma de
  `trocea` (§2.6). R4 GANA abre una spec de producción aparte.
- **Orden jerárquico:** primero R2 sobre `base`. R3 y R4 comparan siempre la
  fusión sellada (A4 y A5 frente a A0). Si R2 adopta RRF, las celdas
  `solape:rrf` y `late:rrf` se reportan como descriptivas **sin** decidir
  sobre ellas: elegir la mejor de las cuatro celdas sería seleccionar sobre el
  held-out.
- **Si nada gana, el resultado válido es "se queda como está".** No se toca
  `buscador.rs` ni `trozos.rs`. La campaña cierra con el verdict y el backlog
  actualizado. NO GANA no significa "son iguales" (§7, potencia).
- **Empate o ambigüedad** (por ejemplo, un umbral de la regla que cae justo
  en el borde por una query con error de captura): lo adjudica fable con este
  texto delante y cita textual. Los números no se renegocian.

## 7. Tamaño, fuentes y lo que el tamaño permite ver

**Fuentes y estratos.** Las fija D1 (§9); cada estrato se reporta aparte.

- `prompt`: prompts de Paul que dispararon `recall-inject`, de 2026-08-22 a
  2026-09-12. Se cruzan los eventos `recall-inject-emitted` y
  `recall-inject-degraded` de `~/.claude/reflex-log.jsonl` con el mensaje de
  usuario del transcript de la misma sesión. Es la distribución real del hook.
- `agent-search`: queries de búsqueda emitidas por agentes, de 2026-07-19 (el
  día siguiente al sellado del sweep, commit `ee839ac` del 2026-07-18) a
  2026-09-12. Salen de `search_notes` en `~/.claude/reflex-retrieval-log.jsonl`
  y de comandos `exo|kbx search|targets` en los transcripts. Misma
  distribución que las 55 y la única que ejercita la fusión (§2.4).
- `hard`: paráfrasis sin palabras clave literales, escritas por un subagente
  fresco a partir de notas **añadidas a la KB después de 2026-08-17** (38
  ficheros según `git log --diff-filter=A`). Ese autor no ve las 55, ni
  `evals/`, ni `reports/`.

**Pool medido el 2026-09-13, en bruto y antes de filtros:** 35 queries
`search_notes` únicas posteriores al corte; unos 27 comandos
`exo|kbx search|targets` en transcripts; 97 eventos `recall-inject-emitted`
en 20 sesiones. Las cuotas de D2 pueden no caber. Si el pool filtrado no llega,
la Task 2 para y lo escala: **no se inventan queries para rellenar.**

**Sin fuga desde las 55 ni desde quien afinó:**

- Se excluye cualquier candidata cuya forma normalizada (minúsculas, sin
  acentos, espacios colapsados) coincida con una de las 55 o tenga Jaccard de
  tokens ≥ 0,8 con alguna.
- Se excluye todo lo emitido desde el 2026-09-13, porque las sesiones de esta
  campaña ya han visto las 55.
- Se excluyen las queries que empiezan por `-` (clap las toma por flag) y los
  prompts de más de 1.500 caracteres (contenido pegado, imposible de
  etiquetar). Ambas exclusiones se declaran como sesgo.
- El etiquetador trabaja **solo con el filesystem del snapshot**: tiene
  prohibidos `exo`, `kbx`, basic-memory, `evals/`, `reports/` y `~/.exo/`.
  Etiquetar con el motor haría el gold circular.
- **Sesgo residual declarado:** etiquetar con grep favorece la coincidencia
  léxica, es decir, a A2. El estrato `hard` y la verificación adversarial
  (Task 4) lo mitigan, pero no lo anulan.
- **Sin parada secuencial (D1/D2, §9):** se etiquetan **todas** las
  candidatas de los pools filtrados de `prompt` y `agent-search`, en el orden
  de la muestra barajada (semilla `20260913`). `hard` = una query por nota
  añadida desde 2026-08-17, la mitad en frase natural y la otra mitad en
  palabras clave. Las nulas no descartan una query: pasan al corpus negativo.
  **Suelo:** si el gold verificado tiene menos de 60 filas no nulas en total,
  STOP y PENDIENTE-PAUL; no se inventan queries para llegar.

**Qué diferencias se pueden detectar** (cálculo exacto, script de recon
2026-09-13):

- McNemar exacto bilateral, α=0,05, favorables mínimas por número de
  discordantes: 6→6/0 · 8→8/0 · 10→9/1 · 12→10/2 · 16→13/3 · 20→15/5.
  Ejemplos de p: 3/0 → 0,25; 4/1 → 0,375; 5/1 → 0,219 (la regla "arregla ≥5,
  rompe ≤1" del gate M0 tenía p=0,22 en su borde); 6/1 → 0,125; 9/1 → 0,021;
  13/4 → 0,049.
- Potencia del McNemar exacto (α=0,05) ante una mejora real de 10 pp con
  discordancia del 16%: **0,19 con N=40 · 0,38 con N=60 · 0,66 con N=100 ·
  0,85 con N=150**. Ante +5 pp: ≤0,37 incluso con N=150.
- **Conclusión honesta:** con un N etiquetable (60–100) ningún test de
  significación va a ver diferencias de 5 pp entre dos fusiones sobre los
  mismos candidatos. Por eso la regla GANA de §6 es una **regla de decisión con
  tasas de error declaradas** y no una afirmación de significación (Webber,
  Moffat & Zobel, CIKM 2008, sobre potencia en experimentación de IR).
- Características operativas de la regla GANA (NETO ≥ 3 y ARREGLA ≥ 2·ROMPE),
  bajo supuestos de discordancia declarados como supuestos:

  | Escenario verdadero | N=60 | N=100 |
  |---|---|---|
  | X igual a A0, discordancia 5% | adopta 0,07 | 0,13 |
  | X igual a A0, discordancia 10% | 0,15 | 0,16 |
  | X peor 5 pp | 0,00 | 0,00 |
  | X mejor 5 pp | 0,58 | 0,83 |
  | X mejor 8 pp | 0,81 | 0,93 |
  | X mejor 10 pp, discordancia 16% | 0,86 | 0,93 |

  Adoptar un candidato **peor** es prácticamente imposible. Adoptar uno
  **igual** cuesta churn y no calidad, y R3 lo acota con el guard de latencia.
  Se prefirió `ARREGLA ≥ 2·ROMPE` a un tope absoluto `ROMPE ≤ 2` porque ese
  tope no escala con N: con N=100 y +10 pp la probabilidad de adoptar bajaba a
  0,42.
- Intervalo de Wilson del hit@5 absoluto: ±8,9 pp con 48/55; ±6,6 pp con
  87/100.

**Protocolo de latencia (R3):** `hyperfine --warmup 3 --runs 30 --export-json`
de `exo search --db <índice> --type hybrid --limit 5 --min-similarity 0.40
--json "memoria persistente de sesiones"`, en el mismo binario y la misma
máquina, con los índices corridos consecutivamente. El p95 se calcula sobre
`results[0].times`.

## 8. Privacidad y artefactos

- **Directorio privado, fuera del repo:** `PRIV=~/.local/share/exo-evals/c-heldout`
  (`chmod 700`). Contiene `in-sample-55.jsonl` (extraído con `git show` de la
  rama archivo), `overlay-fila13.jsonl`, `pool.jsonl`, `muestra.jsonl`,
  `gold.jsonl` (`chmod 444` tras congelar), el snapshot `kb-snap/`, los
  índices `idx-*.db`, las capturas `cap-*.jsonl`, `detalle.jsonl` y el verdict
  completo de la verificación adversarial.
- **Al repo** (`evals/retrieval-heldout/`): el harness, `verdict/agregados.md`,
  `verdict/latencia.md`, `verdict/gold-verificacion-resumen.md` (solo
  recuentos) y `verdict/c-verdict.md`. Ningún texto de query ni permalink por
  fila.

## 9. Decisiones de Paul que se fijan en la congelación

Cada línea se completa en la Task 5 con la opción elegida, literal. Opciones
y trade-offs en el plan, §Decisiones abiertas.

- `D0` régimen de cierre levantado para C: `a` (config de fábrica, bloque ACTUALIZACIÓN 2026-09-13: "Pre-registros y métricas permitidos de nuevo")
- `D1` estratos y proporción: `pools enteros de prompt y agent-search + hard 1 por nota nueva; proporción natural, cada estrato reportado aparte` (Paul, 2026-09-13: "Todo el pool, suelo 60")
- `D2` N de filas no nulas: `sin N fijo; suelo 60 no nulas totales` (Paul, 2026-09-13)
- `D3` brazo late chunking: `no — solo solape; A5, late:rrf y R4 no se miden` (Paul, 2026-09-13: "No, solo solape")
- `D4` umbrales de la regla GANA: `NETO ≥ ____`, `ARREGLA ≥ ____·ROMPE`
  (recomendado: 3 y 2)
- `D5` modo de relevancia para decidir: `____` (recomendado: lenient; strict
  como descriptivo) · overlay de la fila 13 en el reporte in-sample: `____`
  (recomendado: sí, reportando ambas)

## 10. Congelación

- Commit de la KB para el snapshot `S`: `____`
- Binario de medición: `main` en `____` (post-campaña A), `cargo build
  --release --locked`
- `sha256(gold.jsonl)`: `____`
- Filas: `____` totales · `____` no nulas · por estrato `____`
- Aprobación de Paul del gold: línea `GATE:` en `____`
- Commit de congelación: el que introduce este bloque relleno.

## 11. Circuit breakers

- **Fidelidad ≠ 100%:** STOP. Se diagnostica el harness o el binario. No se
  mide ni se decide. Tope de 2 reintentos (`retries` del ledger); al tercero,
  PENDIENTE-PAUL.
- **Gold modificado tras congelar** (el sha256 no casa): medición inválida.
- **Error de captura** en cualquier query de cualquier índice: se reintenta la
  captura completa de ese índice, con el mismo tope. Una query con error
  persistente cuenta como fallo en todos los brazos de ese índice y se declara
  en el verdict.
- **Rebuild `late` con OOM o por encima de 3× el tiempo de `base`:** el brazo
  se declara **no medido**, que no es lo mismo que perder. R4 no se evalúa.
- **Prohibido:** afinar cualquier parámetro sobre el held-out, añadir brazos
  después de congelar o re-etiquetar a la vista de los resultados. El
  held-out queda **consumido** por esta campaña: un ajuste futuro de β, umbral
  o troceado necesita un held-out nuevo.
