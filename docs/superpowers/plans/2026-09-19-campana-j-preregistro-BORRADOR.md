# Pre-registro — Campaña J: retrieval con held-out nuevo (N1, fusión, `archive/`, abstención)

> **Estado: BORRADOR (2026-09-19).** No es contrato hasta el commit de
> congelación (plan fase 1, Task 8), que solo existe cuando el gold agéntico
> existe, `acuerdo.py` ha superado el suelo (§3, §11), `valida_gold.py` pasa y
> su sha256 está en §10. Hasta entonces:
> nadie toca `engine/src` por J, nadie computa hit@k de ningún brazo sobre
> ninguna query del gold nuevo, y este fichero puede cambiar **solo** en tres
> sitios: (1) el tope df de F1 en §4, marcado «T0 puede sustituirlo una vez,
> con evidencia in-sample anotada» (cláusula que Paul dejó intacta al firmar
> D-J8); (2) una línea de §9 **solo** si el review adversarial (plan, Task 5)
> objeta una firma con cita y Paul responde `CAMBIA A`; (3) los campos de
> §10. Las decisiones de §9 **ya están firmadas por Paul el 2026-09-19**
> (`docs/superpowers/consultas/2026-09-15-campanas/propuesta.md` §7),
> incluida la revocación de D-J1/D-J2 por D-J11: **el gold es 100 %
> agéntico** (dos jueces ciegos de dos familias, fable y Kimi/Moonshot, con
> suelo de acuerdo medido; 0 h de Paul; envío de trozos de la KB a Moonshot
> autorizado explícitamente, §8). La congelación la dispara el pipeline cuando
> el acuerdo supera el suelo (§11), no una firma humana. Cualquier otro cambio
> exige reabrir el borrador por escrito antes de que exista el gold.
>
> **Punto abierto (review, I-4):** el T0 está pendiente. La cláusula de D-J8
> se ejerce así: el tope solo cambia si la cobertura (no nulas de las 55 con
> ≥ 1 token raro) a 0,25 es < 80 %; entonces pasa al menor de {0,30; 0,35}
> que la alcance; en cualquier otro caso se anota «cláusula no ejercida». La
> enmienda se commitea antes de generar `paquetes.jsonl`.
>
> **Enmienda 2026-09-20 (T0, cláusula D-J8): cláusula no ejercida.** Cobertura
> de no nulas con ≥ 1 token raro a df ≤ ⌈0,25·174⌉ = 44 es 52/55 ≈ 94,5 %
> ≥ 80 %; por la regla anterior el tope de F1 se queda en 0,25 (evidencia:
> `evals/retrieval-heldout/verdict/diagnostico-55.md` línea «F1 (FTS OR sobre
> tokens raros, tope df 25 %): 52/55 queries tendrían canal léxico, 11 de los
> misses. Valor para D-J8: 0,25 se mantiene — cobertura 52/55 ≈ 94,5 % ≥ 80 %,
> cláusula no ejercida.»). §4 no cambia.
>
> **Enmienda 2026-09-20 (F5, honestidad del pre-registro sobre la regla del
> tope `df`): la regla se fijó con el dry-run in-sample ya visible; el T0
> solo añadió la captura de hoy.** Verificado: `DF_RARO = 0,25` es una
> constante del código (`diagnostico.py`), sin barrido — el T0 no la afinó
> buscando pasar el umbral, y ⌈0,25·174⌉ = 44 con 52/55 coincide con el
> fichero de la captura de hoy, así que el efecto numérico de esto es nulo.
> Pero la secuencia temporal importa para la honestidad del proceso: el §2.6
> (hecho de partida 6) ya citaba el dry-run del 2026-09-19 con «52/55 queries
> tienen ≥1 token "raro"» **antes** de que la regla del «< 80 % ⇒ sube al
> menor de {0,30; 0,35}» se redactara el 2026-09-20 — la cláusula se escribió
> ya sabiendo el número que iba a evaluar. La anotación «cláusula no
> ejercida» de arriba estaba, en ese sentido, pre-decidida; el T0 no la
> decidió, la confirmó. Se declara aquí para que quede escrito.
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
   de ≤ 200). `agent-search`: el `reflex-retrieval-log.jsonl` termina el
   2026-08-17, pero Paul autorizó el 2026-09-19 minar
   `~/.claude/projects/**/*.jsonl`: la extracción cruda
   de la sesión de planificación (221 líneas, `grep` sobre el JSONL) salió
   **truncada** —cortaba en la comilla escapada; 44 líneas sin texto de
   query— y se re-extrae parseando el JSON (Task 6 Step 3, solo `tool_use`
   Bash); tras limpieza y anti-fuga se esperan 60–120 (la Task 6 cuenta; el
   número que se congela es el de la re-extracción). 8 notas añadidas a la KB desde el 2026-09-13.
8. **int8 fuera.** `model_quantized.onnx` cambia el embedding (top-4 solapa
   2–4/4 con fp32, KB-exo «Olas 2-4»): sería un segundo índice completo
   (rebuild ≈ 1,5 h en C, `condiciones.md`) para una pregunta de **no
   inferioridad** («no es peor»), que la regla GANA de §6 no puede responder
   (NO GANA ≠ igual; un margen de 3 pp exige N ≫ 100), y cuyo beneficio
   (~590 ms de carga) canibaliza K si K se hace. Si K = no daemon, int8 tiene
   campaña propia con diseño de no inferioridad y gold propio.
9. **Repo público desde el 2026-09-02.** Ningún texto de query ni permalink
   por fila entra en git. Al repo: harness, kit, agregados, sha256.

## 3. Unidad, gold, jueces y suelo de acuerdo

- **Unidad de análisis:** una query del gold con `expected_permalink` no
  nulo. Las nulas forman dos corpus negativos distintos (abajo).
- **Quién etiqueta (D-J11, firmado 2026-09-19; revoca D-J1/D-J2):** nadie
  humano. Paul ya no lee la KB: la consumen agentes, y el patrón de
  relevancia que importa es la utilidad para el agente que lanzó la consulta.
  El gold lo etiquetan **dos jueces LLM ciegos e independientes, de dos
  familias distintas**, y una fila entra solo si coinciden. La fiabilidad del
  gold no se supone: se mide (κ) y tiene suelo.
- **Schema** (idéntico a C, `source` ampliado):
  `{"id": "j001", "query": str, "source":
  "prompt"|"agent-search"|"hard"|"archive"|"negativo", "expected_permalink":
  str|null, "acceptable_permalinks": [str], "notes": str}`.
  `valida_gold.FUENTES` admite además `keyword` (alineado con este schema,
  M-6 del review): admitido-sin-uso, ningún generador de §3 lo emite en esta
  fábrica; se reserva para una fase futura de queries por palabras clave.
  `acceptable_permalinks`: máx. 2, distintos de `expected`, existentes en el
  snapshot, y **solo los admitidos por ambos jueces** (`acuerdo.fusiona`);
  `notes` lleva las razones de los dos jueces y el `solape_lexico` de la fila.
- **Estratos** (tamaños en filas **juzgadas**; el suelo en §11):

  | `source` | qué es | quién escribe la query | juzgadas | no nulas esperadas tras acuerdo |
  |---|---|---|---|---|
  | `prompt` | prompts reales que dispararon `recall-inject` desde el 2026-09-13 (pool entero; 113 el 2026-09-19; orden barajado con semilla `20260913:prompt`) | ya escritas (Paul, en producción) | ≈ 113 | 20–30 (C: 31 % no nulas × acuerdo ≈ 0,75); sus nulas = corpus negativo débil |
  | `agent-search` | comandos `exo search` reales lanzados por agentes, minados de `~/.claude/projects/**/*.jsonl` con autorización de Paul y limpiados con criterio escrito (`limpia_agent_search.py`: parsea, sin marcadores de doc/plan/test, anti-fuga vs 55 + 147 de C, dedupe) | ya escritas (agentes, en producción) | 60–120 (cota; la Task 6 cuenta) | 40–70 |
  | `hard` | paráfrasis sin palabras literales, mitad frase natural / mitad palabras clave, sobre 30 notas al azar del snapshot (semilla `20260919`, sin `archive/`) | subagente fresco que no ve las 55, ni C, ni `evals/` | 30 | 20–25 |
  | `archive` | preguntas por un hecho fechado que vive **hoy** solo en `archive/log/` (comprobado por el generador con grep en la bitácora viva), sobre 20 rotaciones al azar (semilla `20260919`) | subagente fresco | 20 | 12–16 (exige expected bajo `archive/`) |
  | `negativo` | temas **ausentes** de la KB; `expected` null por definición; ausencia comprobada **mecánicamente** (`grep -ril` de 3–5 `topic_terms` sobre el snapshot entero = 0 ficheros) **y** por los dos jueces (ambos null sobre las candidatas que un agente intentó encontrar) | subagente fresco | 40 | 0 (30–36 nulas verdaderas) |

  Objetivo total: **92–141 no nulas** (suma de los rangos «no nulas
  esperadas tras acuerdo» de la tabla; corrección del review, M-5 — este
  fichero decía 100-130 aquí, 80-95 más abajo y la tabla sumaba 92-141: se
  fija 92-141 en los tres sitios); suelo 60 (§11). Nulas de `prompt`
  (por inferibilidad) = corpus negativo **débil**, solo descriptivo.
  `negativo` = corpus negativo **verdadero**: ids `jNNN` pares → calibración
  de G1, ids `jNNN` impares → evaluación de G1 (§4) — `jNNN` es el id que
  `acuerdo.py` renumera al escribir el gold (§10), no `qNNN` (corrección del
  review, M-3). **Orden = ids `qNNN` ascendentes (F6, review de rama
  2026-09-20):** el split pares/impares es sobre `jNNN`, y `jNNN` lo asigna
  `acuerdo.py` numerando secuencialmente las filas de `cands` en el orden en
  que aparecen en `candidatos.jsonl` (§7 de este documento, `construye`) —
  ese orden no está fijado por ningún criterio salvo el del fichero de
  entrada. Se fija aquí: dentro del estrato `negativo`, `queries.jsonl` (y
  por tanto `candidatos.jsonl` y `paquetes.jsonl`) mantiene el orden de
  `qNNN` ascendente sin barajar (a diferencia de `prompt`, que sí se baraja
  con semilla propia, arriba) — así el split pares/impares de calibración/
  evaluación de G1 es reproducible y no depende de un accidente de
  generación.
- **`prompt` incluye consultas del propio día de diseño (F6, declarado):**
  del pool de 139 prompts reales en este kit, **35 son del 2026-09-19**, el
  mismo día en que se diseñó este experimento (`pool.jsonl`, campo `ts`). No
  es fuga del gold (son prompts de producción anteriores a la redacción de
  este borrador, no queries fabricadas para el diseño), pero se declara
  porque el generador de `hard`/`archive`/`negativo` y parte del propio
  pre-registro se escribieron el mismo día.
- **`hard` es "sin palabras literales" solo a medias (F6, declarado):** la
  mediana de `solape_lexico` (§3, auditoría del sesgo léxico) del estrato
  `hard` en el kit es **p50 = 0,50** — la mitad de las paráfrasis conserva
  al menos la mitad de los tokens ≥4 letras de la nota original, pese a la
  instrucción «sin palabras literales» al generador. Descriptivo: no cambia
  el diseño ni el guard léxico de D-A (§3, §6), que ya usa `solape_lexico`
  medido, no la instrucción al generador, como criterio.
- **Candidatos.** Un agente filesystem-only (sin `exo`, sin `kbx`, sin ver
  `author_expected` ni `topic_terms`) propone ≤ 5 notas por query, con al
  menos 2 por navegación temática (core-index, títulos) y no por grep, y con
  la rotación archivada **y** la bitácora viva cuando la query menciona un
  hecho fechado. Es el cuello léxico del diseño y se declara como tal.
- **Jueces (los dos ven exactamente el mismo paquete: query + candidatas con
  permalink, título y los primeros 12.000 caracteres; sin etiquetas, sin
  rankings de ningún brazo, sin el otro juez, sin acceso al snapshot ni a
  `evals/`):**
  - **fable** (Claude): subagente fresco por lote de 25 paquetes, instrucción
    literal `juez.SISTEMA`, salida JSONL validada por `juez.py valida`.
  - **Kimi** (Moonshot, otra familia de modelo, para romper la correlación de
    errores Claude–Claude): `juez.py kimi`, API OpenAI-compatible en
    `https://api.moonshot.ai/v1`, `response_format: json_schema` estricto,
    `temperature 1`, misma instrucción `juez.SISTEMA`, 3 reintentos con
    backoff. Modelo: `kimi-k3` (elegido por Paul 2026-09-19 tras ver la lista real de `/v1/models`: k2.6, k2.7-code, k2.7-code-highspeed, k3); si no lo lista, el `kimi-k*`
    de mayor versión; fijado en §10 en runtime.
  - **sonnet** solo genera (queries `hard`/`archive`/`negativo`, candidatos);
    nunca juzga.
- **`temperature 1` del juez Kimi, no 0 (F6, declarado, 2026-09-20):** se
  pidió `temperature 0` por determinismo; la API de Moonshot lo rechaza con
  400 en los dos modelos probados (`kimi-k3` y `kimi-k2.6`): `{"error":
  {"message":"invalid temperature: only 1 is allowed for this model",
  "type":"invalid_request_error"}}`. Se usa `1`, el único valor que la API
  acepta. Se pierde la reproducibilidad bit a bit de una corrida del juez;
  queda en pie la defensa del diseño: dos jueces independientes (fable y
  Kimi) y su acuerdo medido contra el suelo pre-registrado (κ ≥ 0,60 ∧
  p_o ≥ 0,70, arriba), no la determinación de una sola llamada.
- **`MAX_CHARS` 4.000 → 12.000, parada y rehecho del kit (F7, declarado,
  2026-09-20):** el kit se juzgó primero con `MAX_CHARS = 4.000`
  (`juez.py:paquete`/`nota`). Medido sobre ese kit: **573 de 619 cuerpos
  servidos (93%) llegaban truncados al tope**, y **216 de 217 filas con
  candidatas (100%) tenían al menos una candidata truncada** — con
  p50 = 10.093 y p90 = 20.464 caracteres de las notas servidas, en la nota
  mediana el juez veía solo el **40%** del texto. Varios jueces lo
  detectaron solos e independientemente ("ningún extracto lo muestra", "el
  paquete corta justo antes") y etiquetaron por inferencia de tema en vez de
  por haber leído el hecho: un gold etiquetado sobre media nota sesga hacia
  `null`. Se paró la corrida y se rehizo el kit con `MAX_CHARS = 12.000`,
  elegido porque coincide con el techo que el contrato de memoria de la KB
  `wisdom-paul` fija para una nota `stable` (12.500 B) — deja de ser un
  número arbitrario y hace que la nota mediana entre entera. Con el tope
  nuevo: 175/619 cuerpos (28%) y 118/217 filas (54%) siguen con truncamiento
  — de una nota grande de verdad, no ya de la mayoría del kit por defecto.
  **Se conserva** la corrida truncada como brazo de comparación, intacta, en
  `~/.local/share/exo-evals/j-heldout/trunc4000/` (no se toca, no se usa
  como gold). El gold final sale del kit rehecho a 12.000; los `candidatos`
  de cada fila (permalinks y orden) no cambiaron, solo el texto servido.
- **Entrada al gold y desempate (D-J11):** una fila entra si el acuerdo es
  *estricto* (mismo `expected`, null incluido) o *lenient* (el `expected` de
  un juez está en los `acceptable` del otro; entonces `expected` = el que
  ambos admiten, y el `expected` descartado del otro juez pasa a
  `acceptable` **solo si el primer juez también lo admite** en su propio
  `acceptable`: «entra solo con acuerdo», D-J11 — corregido en `acuerdo.fusiona`
  el 2026-09-20 tras el review, I-2; antes de la corrección una nota vista
  por un solo juez podía colarse en `acceptable` y contar hit). **Tie-break
  asimétrico, declarado (I-2):** si en una fila ambas direcciones lenient
  valen a la vez (el `expected` de fable está en lo aceptable de Kimi **y**
  viceversa), hoy gana **fable** por el orden de los `if` en `fusiona`; no se
  corrige (no hay criterio no arbitrario para preferir uno u otro en ese
  empate exacto), solo se declara. **Los desacuerdos se
  descartan**, se listan en privado y se cuentan en público. No hay tercer
  juez: un opus de desempate convertiría cada desacuerdo en 2 Claude contra
  1 Kimi y el gold quedaría etiquetado de hecho por una familia; el pool sobra
  para permitírselo; y el sesgo de descartar (hacia filas «claras») se mide
  (abajo) en vez de esconderse en un voto.
- **Suelo de acuerdo pre-registrado (kill-criterion, §11):** sobre **todas**
  las filas juzgadas por ambos, `κ ≥ 0,60` **y** `p_o ≥ 0,70`, donde `p_o` es
  la fracción con acuerdo estricto o lenient y κ es la **κ de Cohen** (Cohen,
  Educational and Psychological Measurement 20(1), 1960) sobre el `expected`
  estricto, con cada permalink y el null como categorías nominales. Por qué
  κ y no solo `p_o`: en `prompt` ≈ 70 % de las filas son null y dos jueces
  que dijeran null siempre acordarían el 49 % por azar; κ descuenta ese azar.
  **Con ≈ 57 % de nulas, `p_e` ≈ 0,32 y κ ≥ 0,60 equivale a `p_o` estricto ≥
  0,73: κ es el criterio operativo y `p_o` ≥ 0,70 (lenient) el redundante. El
  estrato `negativo` (null-null por construcción) infla κ y `p_o` pooled; se
  publica también κ y `p_o` excluyendo `negativo` como descriptivo (Feinstein
  & Cicchetti, J Clin Epidemiol 43(6):543-549, 1990, sobre la dependencia de
  κ de los marginales; corrección del review, I-3, 2026-09-20 — la
  formulación anterior, «κ ≈ p_o con muchas categorías», no aplicaba aquí: el
  null es una única categoría mayoritaria, no muchas equiprobables).** 0,60
  es la **frontera moderate/substantial** de Landis & Koch (Biometrics 33(1),
  1977: 0,41-0,60 es *moderate*, 0,61-0,80 es *substantial* — 0,60 es el
  techo de *moderate*, no el suelo de *substantial*; corrección del review,
  I-6). 0,70 de acuerdo bruto es el
  suelo que, con ≈ 280 filas juzgadas, deja ≥ 190 filas y coincide con el
  objetivo total de 92–141 no nulas (§3, M-5 del review). Se
  calcula también por estrato: un estrato con `p_o < 0,60` se marca «flojo»
  y se reporta, sin decidir por sí solo (la decisión es sobre el total).
  **κ / `p_o` sin `negativo`** se computa siempre y se reporta como línea
  adicional del informe (`acuerdo.construye`, sobre todas las filas con
  `source != "negativo"`); no cambia el exit ni el suelo firmado (0,60 ∧
  0,70), es descriptivo. **Si
  el suelo global (con `negativo` incluido, tal como firmó D-J11) no pasa, J
  PARA: es un resultado** («dos familias de modelo
  no coinciden en qué nota sirve al agente»), se publica el informe de
  acuerdo y no se congela nada.
- **Sesgo léxico de los jueces LLM, declarado.** Un juez LLM tiende a llamar
  relevante a la nota que repite las palabras de la consulta (Alaofi, Thomas,
  Scholer & Sanderson, **SIGIR-AP 2024** (no SIGIR 2024; corrección del
  review, M-8), DOI 10.1145/3673791.3698431;
  Clarke & Dietz 2024 y Soboroff 2024 sobre por qué los juicios LLM no
  sustituyen a los humanos; Thomas, Spielman, Craswell & Mitra, SIGIR 2024, y
  Faggioli et al., ICTIR 2023, sobre el acuerdo LLM–humano; verificadas Task
  5, §7). Aquí no hay humano al que compararse, así que el
  sesgo se **mide; el guard solo acota el daño en filas no léxicas**, no se
  contiene y no se niega — corrección del review (I-1, 2026-09-20): el guard
  de (iii) exige `NETO ≥ 0` solo sobre el subconjunto de solape **bajo**
  (`< 0,5`), pero el sesgo real produce falsos ARREGLA de F1 en filas de
  solape **alto**, que ese subconjunto no mira; dentro del subconjunto que sí
  mira, F1 apenas difiere del baseline, así que el guard pasa por
  construcción con frecuencia (cuantificado por el review: veta un F1 «igual
  al baseline» solo el 37 % de las veces). El guard **acota el daño que se
  ve**, no lo impide: (i) `juez.SISTEMA` pide
  explícitamente no premiar la repetición literal; (ii) `acuerdo.py` calcula
  por fila `solape_lexico` (fracción de tokens ≥ 4 letras de la query
  presentes en la nota esperada, **por subcadena, no por token** — «nota»
  casa con «notación»; declarado, M-4 del review) y publica la distribución
  en filas
  acordadas frente a descartadas: si los desacuerdos se concentran en solape
  bajo, el gold está sesgado hacia lo léxico y se dice (para las filas
  descartadas, `solape_lexico` usa `fab.expected or kim.expected` — el
  `expected` del primer juez que lo tenga, lo cual es **asimétrico**: si
  fable no propuso `expected` pero Kimi sí, se audita el de Kimi, y
  viceversa; declarado, M-4); (iii) el sesgo
  favorece a F1 (FTS sobre raros) y a A2, así que **D-A lleva un guard**: F1
  solo GANA si además `NETO ≥ 0` en el subconjunto «léxicamente difícil»
  (`solape_lexico < 0,5`) de las no nulas (§6); **además, como descriptivo
  obligatorio del verdict de D-A (pre-registrado, I-1 del review), se
  publica el reparto de los ARREGLA de F1 por `solape_lexico` ≥/< 0,5, con
  esta lectura literal: si ≥ 80 % de los ARREGLA de F1 caen en solape ≥ 0,5 y
  la mediana de solape de las filas descartadas es ≥ 0,2 inferior a la de
  las acordadas, el GANA de D-A se publica con la etiqueta *compatible con
  sesgo léxico del gold*. Es una etiqueta, no un veto: no toca D-J4 ni
  D-J11**; (iv) los candidatos incluyen
  ≥ 2 notas por navegación temática para que el juez pueda elegir una nota
  que responde sin repetir.
- **Modo de decisión:** lenient (`{expected} ∪ acceptable`), strict como
  descriptivo (D-J10).
- **Anti-fuga:** se excluye toda query con forma normalizada igual o Jaccard
  ≥ 0,8 frente a las 55 **y** frente a las 147 de C (`valida_gold.py
  --in-sample` repetible, `limpia_agent_search.py`). Generadores, autores de
  candidatos y jueces no leen las 55, ni el gold de C, ni `evals/`, ni
  `reports/`. Otros sesgos declarados: citas literales de prompts en
  bitácoras (S2 de C, `i`); `hard`, `archive` y `negativo` los escribe un
  modelo de la misma familia que uno de los jueces (sonnet/fable): mitigado
  porque el segundo juez es de otra familia y porque el generador no juzga.

## 4. Brazos (parámetros fijos; ninguno se elige sobre el held-out)

**Un solo índice `base`** (snapshot `S_J`, binario post-G/L, modelo pineado
`8e2d780d…`). Por query se captura, como en C: FTS-AND `--limit 50`, vector
`--limit 1000 --min-similarity 0.0`, hybrid `--limit 10 --min-similarity 0.40
--bonus 0.0 --fts-scale 0.6`; y además, **offline** con `sqlite3` sobre una
copia de `idx-base.db`: la réplica FTS-AND (oráculo) y la lista FTS de F1.
Todos los rankings se calculan offline (`metricas.py` + módulo `brazos.py`
de la fase 2). Scores vector en la escala `s` del binario (§2.2).
**Nota para la fase 2 (M-9 del review):** hoy `metricas.rankings` solo conoce
`sellado|sellado-035|rrf|vector|fts`; F1, S1, P1 y G1 de la tabla de abajo
**no existen en código todavía** — son prosa de este borrador, no celdas
computables. Congelar hoy sería congelar prosa, no un `brazos.py` verificado;
la Task 8 (congelación) no se ejecuta en esta fábrica, así que esto queda
como nota abierta para cuando `brazos.py` se escriba.

| brazo | definición exacta | papel |
|---|---|---|
| **A0** `sellado` | `max(v, 0.6·f/f_max)`, v filtrado a ≥0,40, orden (−score, permalink), top-10 | referencia |
| A1 `vector` · A2 `fts` | vector ≥0,40 · FTS-AND top-k | testigos (D6 se confirma con A0 vs A2) |
| A3 `rrf` | `Σ 1/(60+rango)` sobre FTS-AND y vector ≥0,40 | testigo (C ya decidió) |
| **F1** `fts-raro` | tokens = `query.split()` como `prepara_query`; `df(tok)` = nº de notas con `MATCH '"tok"'`; se conservan los tokens con **1 ≤ df ≤ ⌈0,25·N_notas⌉** (T0 puede sustituir 0,25 una vez, con evidencia in-sample anotada aquí; dry-run: 52/55 queries con ≥1 raro); `MATCH` = raros unidos con `OR`, `-bm25`, `LIMIT 50`; 0 raros ⇒ canal FTS vacío. La fusión es la sellada, idéntica a A0, sobre esa lista | candidato Q1 |
| **S1** `combsum` | `score = v_adm + 0.6·f/f_max`, `v_adm = v` si v ≥ 0,40, si no 0; admisión = unión; orden (−score, permalink); top-10. β = 0,6 heredado, sin afinar | candidato Q2 |
| **P1** `archive-0.90` | score × **0,90** para permalinks que contienen `/archive/`, antes de ordenar (valor único declarado; D-J7) | candidato Q3 |
| **G1** `abstiene-τ` | top-5 vacío si `score(top1) < τ`; **τ = percentil 70 de los scores top-1 del brazo incumbente sobre los `negativo` de ids pares** (calibración): valor en la posición `⌈0,7·n⌉` de la lista **ascendente** de esos scores top-1 (n = nº de `negativo` pares con top-1 no vacío); abstiene si `score(top1) < τ` (estricto, no `≤`) (método fijado, corrección del review, M-2); se evalúa sobre `negativo` impares y sobre todas las no nulas | candidato Q4 |

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
  semilla `20260913` — corrección del review, M-1: este fichero decía
  `20260919`, `metricas.py` tiene `SEMILLA = 20260913`; gana el código.
  Método (`metricas.bootstrap_ic95`, declarado): percentil 2,5/97,5 sobre
  las 10.000 medias remuestreadas, `random.Random(semilla)`, remuestreo con
  reemplazo **por query** — cada remuestreo saca `m` deltas de MRR con
  reemplazo del vector de `m` deltas pareados, uno por query, y promedia).

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
  **Guard léxico (§3):** además de GANA, F1 exige `NETO ≥ 0` en el
  subconjunto de no nulas «léxicamente difíciles» (`solape_lexico < 0,5` en
  `notes`); si ese subconjunto tiene < 20 filas, el guard se reporta y no
  veta. Motivo: los jueces LLM favorecen la coincidencia léxica y F1 es el
  brazo que más se beneficia de ella.
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
- **No hay empates** (corrección del review, I-5: la formulación anterior
  —«lo adjudica un fable fresco con este texto delante»— abría una puerta
  post hoc justo donde el pre-registro no puede abrirla, un humano o un
  modelo interpretando el texto después de ver los números). `metricas.decide`
  congelado **es** la regla: la regla GANA (§6) y las reglas D-A/D-B/D-C/D-D
  son funciones puras de (ARREGLA, ROMPE, IC95, guards) sobre el gold
  congelado, sin remanente de ambigüedad que adjudicar. Si una lectura
  humana del texto de este documento discrepa de lo que computa el código
  congelado, **manda el código**, y la discrepancia se anota como errata en
  el verdict (no se recomputa nada, no se re-juzga nada: se corrige la prosa
  para la próxima campaña).

## 7. Tamaño, potencia y multiplicidad

**Qué puede ver la regla** (cálculo exacto multinomial sobre (ARREGLA, ROMPE),
sin veto MRR, script de recon 2026-09-19; `a` = P(X acierta, I falla), `b` =
al revés; «igual, disc. d» = a = b = d/2). Con el gold agéntico el N
esperado es **92–141 no nulas** (§3, corrección del review M-5); se muestran N = 60–100 con las tres reglas
(para leer la firma D-J4) y N = 100–180 con NETO ≥ 4:

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

| escenario verdadero, **NETO ≥ 4 ∧ ARREGLA ≥ 2·ROMPE** (D-J4) | N=100 | N=120 | N=150 | N=180 |
|---|---|---|---|---|
| igual, disc. 5 % | 0,06 | 0,07 | 0,10 | 0,11 |
| igual, disc. 10 % | 0,12 | 0,13 | 0,11 | 0,10 |
| igual, disc. 16 % | 0,11 | 0,09 | 0,06 | 0,04 |
| peor 5 pp | 0,00 | 0,00 | 0,00 | 0,00 |
| mejor 3 pp | 0,42 | 0,48 | 0,53 | 0,54 |
| mejor 5 pp | 0,67 | 0,74 | 0,79 | 0,81 |
| mejor 8 pp | 0,91 | 0,94 | 0,96 | 0,97 |
| mejor 10 pp | 0,93 | 0,95 | 0,96 | 0,97 |
| familia de 3 decisiones GANA, cota `1−(1−p)^3` con «igual, disc. 10 %» | 0,32 | 0,33 | 0,31 | 0,26 |

(Las celdas «≥k» son `NETO ≥ k ∧ ARREGLA ≥ 2·ROMPE`. Reproduce las cifras de C
para ≥3: 0,07/0,13 · 0,15/0,16 · 0,81/0,93; la errata de C en la fila «+5 pp»
está corregida aquí con el supuesto declarado. Con N ≥ 150 y disc. 5 % la
probabilidad de adoptar un igual sube, no baja: `NETO ≥ 4` es absoluto y una
discordancia rara con N grande produce NETO 4 por azar más a menudo. No se
corrige: N real ≈ 92–141 y la cota familiar queda ≈ 0,32.)

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
   Westfall, SIGIR 2013) reduce N a la mitad y la potencia ante +5 pp a ≈0,5:
   descartado. Se elige el **camino secuencial pre-declarado** (§6), que es la
   forma de «hipótesis a priori ordenadas» de Maurer, Hothorn & Lehmacher
   (1995) y Westfall & Krishen (JSPI, 2001) aplicada a reglas de decisión:
   cada paso compara con el incumbente y no hay selección entre celdas.
2. **NETO ≥ 4** (D-J4, firmado 2026-09-19): con k=3 la cota familiar baja de
   0,41 a 0,32 y la potencia ante +8 pp está en 0,91–0,94 (N=100–120). NETO
   ≥ 5 bajaría la cota a 0,21 pagando +5 pp → 0,55.
3. **N esperado 92–141** (§3): potencia ante +5 pp 0,67–0,74 con NETO ≥ 4.
   Una diferencia de 3 pp sigue siendo casi invisible (0,42–0,48): la regla
   es una **regla de decisión con tasas de error declaradas**, no una
   afirmación de significación (Webber, Moffat & Zobel, CIKM 2008; Smucker,
   Allan & Carterette, CIKM 2007, sobre por qué bootstrap pareado y no
   sign/Wilcoxon). Sobre tamaño de conjunto de topics: Voorhees & Buckley,
   SIGIR 2002; Sakai, Information Retrieval Journal 2016 («Topic set size
   design»).
4. **D-D (abstención)** no entra en la familia GANA: métrica distinta y guard
   asimétrico. Con 15–18 negativos de evaluación, 9/15 (60 %) da Wilson
   [0,36, 0,80]; el guard `ROMPE ≤ 2` sobre ~80 aciertos se cumple con
   probabilidad ≈ 0,95 si la pérdida real por fila es 1 %, ≈ 0,57 si es 3 %:
   G1 solo se adopta si casi no cuesta. **Declarado (corrección del review,
   M-7):** «abstiene en ≥ 60 % de evaluación» es casi tautológico con
   τ = p70 — por construcción la tasa esperada sobre el propio calibrado es
   ≈ 0,70, y P(≥9/15 con p verdadero 0,70) ≈ 0,87, así que esa mitad de la
   regla D-D casi siempre se cumple sola. El contenido informativo real de
   D-D está en el guard, no en el umbral de abstención: **`ROMPE ≤ 2`** es
   la condición que de verdad puede fallar y la que decide si G1 se adopta.
5. **Fiabilidad del gold como fuente de ruido adicional:** una etiqueta
   errónea compartida por ambos jueces castiga o premia a todos los brazos
   por igual (no fabrica un GANA por sí sola salvo que se concentre en filas
   discordantes); una etiqueta errónea de un solo juez descarta la fila. El
   riesgo direccional es el léxico (§3), y por eso tiene guard.

**Confianza en las referencias** (Paul valida con papers; se dice lo que se
sabe): Cormack, Clarke & Büttcher SIGIR 2009 (RRF); Fox & Shaw TREC-2 1994
(CombSUM/CombMAX); Lee SIGIR 1997; Voorhees & Buckley SIGIR 2002; Smucker et
al. CIKM 2007; Webber et al. CIKM 2008; McNemar 1947; Wilson 1927; Efron &
Tibshirani 1993; Cohen 1960 — **seguras**.

Las 12 siguientes están **verificadas 2026-09-20 (review adversarial, Task
5)**, con DOI/identificador comprobado — ya no son «de memoria»:

- Alaofi, Thomas, Scholer & Sanderson — SIGIR-AP 2024 — DOI
  10.1145/3673791.3698431 (corregida de «SIGIR 2024», M-8)
- Landis & Koch — Biometrics 33(1):159-174, 1977 — PMID 843571
- Feinstein & Cicchetti — J Clin Epidemiol 43(6):543-549, 1990 — PMID
  2348207
- Thomas, Spielman, Craswell & Mitra — DOI 10.1145/3626772.3657707
- Faggioli et al. — DOI 10.1145/3578337.3605136
- Clarke & Dietz — arXiv:2412.17156
- Soboroff — arXiv:2409.15133
- Carterette — DOI 10.1145/2094072.2094076
- Boytsov, Belova & Westfall — DOI 10.1145/2484028.2484034
- Sakai — DOI 10.1007/s10791-015-9273-z
- Maurer, Hothorn & Lehmacher, 1995 — capítulo de libro, sin DOI
- Westfall & Krishen — DOI 10.1016/S0378-3758(01)00077-5

Ninguna regla dependía de ellas (dependen de las tablas); la verificación
solo cambia el estado de las citas, no ninguna cifra.

**Protocolo de latencia:** no aplica en J (ningún brazo cambia el índice ni el
KNN; F1 añade una consulta FTS más por query, ≤ ms). Se mide en la fase 2 solo
si el ganador se implementa, como no-regresión (`hyperfine`, mismo protocolo
que C §7).

## 8. Privacidad, exposición de datos y artefactos

- **Envío a Moonshot (Kimi), autorizado explícitamente por Paul el
  2026-09-19 en sesión** (`docs/superpowers/consultas/2026-09-15-campanas/propuesta.md`
  §7; frase literal: «no me importa mandar a kimi, continua por ahí»). Por
  fila juzgada salen hacia `https://api.moonshot.ai/v1`: la query (un prompt
  de Paul o un comando de agente) y hasta 5 notas candidatas de la KB
  `wisdom-paul`, cada una recortada a 12.000 caracteres (F7, declarado
  arriba: era 4.000, rehecho el 2026-09-20). No salen etiquetas,
  rankings, nada de `evals/` ni del gold de C. La key vive en
  `wisdom-ai-news/.env-keys` (gitignored), se lee en runtime y nunca se
  imprime ni entra en este repo.
- **Directorio privado:** `PRIV_J=~/.local/share/exo-evals/j-heldout`
  (`chmod 700`). Contiene `gold-j.jsonl` (`chmod 444` tras congelar),
  `kb-snap/` (snapshot `S_J`), `muestra-prompt.jsonl`, `agent-search.jsonl`,
  `generadas.jsonl`, `queries.jsonl`, `candidatos.jsonl`, `paquetes.jsonl`,
  `juicio-kimi.jsonl`, `juicio-fable.jsonl`, `descartes.jsonl`,
  `kimi-modelos.json`, `cap-*.jsonl`, `idx-base.db`, `diagnostico-55-detalle.md`,
  `review-preregistro.md`, `detalle.jsonl`. Se reutiliza en lectura
  `PRIV_C=~/.local/share/exo-evals/c-heldout` (in-sample, capturas de C).
- **Al repo** (`evals/retrieval-heldout/`): `harness/{diagnostico,limpia_agent_search,juez,acuerdo,test_gold_j}.py`,
  extensiones de `valida_gold.py` y `pool.py`, `gold-j/README.md` y
  `gold-j/plantilla.jsonl`, `verdict/diagnostico-55.md`,
  `verdict/gold-j-acuerdo.md` (solo recuentos, κ, auditoría léxica), y en la
  fase 2 `verdict/j-agregados.md` y `verdict/j-verdict.md`. Ningún texto de
  query ni permalink por fila. El gold **no** va al repo ni gitignored: vive
  fuera del árbol, como en C.

## 9. Decisiones de Paul (firmadas el 2026-09-19)

Firmadas en sesión, una a una. Fuente: `docs/superpowers/consultas/2026-09-15-campanas/propuesta.md`
§7. Opciones descartadas y trade-offs en el plan, §«Decisiones firmadas por
Paul 2026-09-19». Solo cambian si el review adversarial (plan, Task 5)
objeta con cita y Paul responde `CAMBIA A`.

- `D-J1` modo de etiquetado: `REVOCADA el 2026-09-19 por D-J11` (había firmado b; ninguna hora de Paul)
- `D-J2` tamaño del estrato `prompt`: `REVOCADA el 2026-09-19 por D-J11` (D-J11 fija el pool entero, 113 el 2026-09-19)
- `D-J3` verificación adversarial del gold antes de congelar: `sí` — realizada por el diseño de dos jueces y por el consultor-gate de la congelación
- `D-J4` regla GANA: `NETO ≥ 4` (`ARREGLA ≥ 2·ROMPE` y veto ΔMRR) (descartadas: 3, 5)
- `D-J5` H28: `a — corrección de nombre y doc, escala s se queda, sin brazo` (descartada: b, celda sellado-cos)
- `D-J6` int8: `fuera` (descartada: dentro si K = no daemon)
- `D-J7` factor de P1: `0,90`
- `D-J8` tope df de F1: `0,25, con la cláusula de T0 intacta` (T0 puede sustituirlo una vez, con evidencia in-sample anotada como enmienda)
- `D-J9` abstención G1: `tal cual — percentil 70 de calibración; adopta si abstiene ≥ 60 % de evaluación y ROMPE ≤ 2`
- `D-J10` modo de relevancia: `lenient decide, strict descriptivo (D5 de C)` (descartada: strict decide)
- `D-J11` gold agéntico: `queries reales de agentes (agent-search minado con autorización) + prompt + hard/archive/negativo generados; jueces ciegos fable y Kimi (Moonshot); entra solo con acuerdo; desacuerdos descartados, sin tercer juez; suelo κ ≥ 0,60 ∧ p_o ≥ 0,70, por debajo J PARA; envío de trozos de la KB y queries de agentes a Moonshot autorizado explícitamente` (descartadas: tercer juez opus; gold etiquetado por Paul)

## 10. Congelación

- Commit de la KB para el snapshot `S_J`: `<sha de $PRIV_J/kb-snap.commit>`
- Binario de medición: `<commit de main post-G/L; se anota en verdict/j-condiciones.md en la fase 2, no aquí>`, `cargo build --release --locked`
- Modelo de Kimi: `<id elegido por la regla de la Task 7 Step 1>` de entre `<lista de /v1/models en kimi-modelos.json>`; llamadas `<n>`, tokens `<prompt> / <completion>`, coste `<$>`
- Acuerdo: filas juzgadas por ambos `<n>` · `p_o <x>` · `κ <x>` · descartes `<n>` (desacuerdo `<n>`, negativo con nota `<n>`, archive fuera de archive/ `<n>`) · estratos flojos (`p_o < 0,60`): `<ninguno | lista>` · auditoría léxica: mediana de solape acordadas `<x>` / descartadas `<x>`
- κ / p_o sin `negativo` ni candidatos vacíos: `<x> / <x>` (descriptivo, I-3 del review + F3 del review de rama 2026-09-20; no cambia el exit ni el suelo firmado 0,60 ∧ 0,70)
- `sha256(gold-j.jsonl)`: `<salida de valida_gold.py>`
- Filas: `<total>` · `<no nulas>` · por estrato `prompt <n> · agent-search <n> · hard <n> · archive <n>` · nulas `<prompt n · negativo n>` · negativos de evaluación (impares) `<n>` · con acceptable `<n>` · no nulas «léxicamente difíciles» (solape < 0,5) `<n>`
- Aprobación del gold: `acuerdo.py exit 0 el <fecha>` (suelo superado; no hay línea de Paul) · gate del pre-registro: línea `GATE: PRE-REGISTRO J CONGELADO <fecha>` del consultor fable en `.superpowers/fabrica/verdicts/j-consultor-gate.md`
- **Precondición dura antes de la primera captura de la fase 2 (F4, review de
  rama 2026-09-20 — cierra el séptimo grado de libertad):** este §10 dice que
  «manda `metricas.decide` congelado» (§6), pero F1/S1/P1/G1 (§4) **no
  existen en código todavía** — son prosa de este borrador (nota M-9 del
  review original, §4), y `brazos.py` es explícitamente de la fase 2. Si
  `brazos.py` y la extensión de `metricas.decide` para el camino secuencial
  de §6 se escribieran **después** de ver los datos, las decisiones que ese
  código tiene que tomar (el `f_max` exacto de F1; si la admisión "unión" de
  S1 cuenta una nota que solo tiene score FTS con `v_adm = 0` o la descarta;
  si P1 multiplica el factor 0,90 antes o después de tomar el `max` de la
  fusión; el criterio de empate cuando dos brazos ordenan listas distintas
  con el mismo score) dejarían de ser reglas pre-registradas y pasarían a
  ser código escrito con los números delante. Por eso: `brazos.py` +
  `metricas.decide` (extendido) + **tests-oráculo por brazo** (filas fixture
  sintéticas con la salida exacta esperada, uno por cada decisión de arriba)
  deben existir **committeados** antes de la primera captura sobre el gold
  congelado. Cambiar cualquiera de los dos DESPUÉS de esa primera captura
  invalida la medición hecha con la versión anterior (no se recomputa con el
  código nuevo sobre los mismos resultados: se declara qué versión aplicó y,
  si hace falta el cambio, se repite la captura y la medición desde cero).
  Ver kill-criterion correspondiente en §11.
- Commit de congelación: el que introduce este bloque relleno y cambia la cabecera a `CONGELADO`.

## 11. Circuit breakers y kill-criteria

- **Acuerdo por debajo del suelo** (`κ < 0,60` o `p_o < 0,70` sobre todas las
  filas juzgadas por ambos): **J PARA**. Es un resultado: dos familias de
  modelo no coinciden en qué nota sirve al agente; se publica
  `verdict/gold-j-acuerdo.md` con `NO PASA`, se anota en el backlog y no se
  congela nada. Reabrir J exige otro diseño de gold, no repetir los jueces
  hasta que pasen.
- **Gold construido con < 60 no nulas:** STOP y PENDIENTE-PAUL; no se
  inventan filas ni se relaja el suelo. Con < 12 `negativo` de evaluación
  (impares), D-D no se decide (se reporta). Con < 8 filas `archive`, D-C no
  decide (se reporta por estrato como descriptivo). Un estrato con `p_o <
  0,60` se marca «flojo»: sigue en el total (la decisión es sobre el total)
  pero cualquier lectura por estrato lo dice. Con `agent-search` limpio < 30,
  el estrato se declara flojo de origen.
- **Guard léxico de D-A** (§3, §6): F1 no GANA si `NETO < 0` en el subconjunto
  «léxicamente difícil» (`solape_lexico < 0,5`); si ese subconjunto tiene
  < 20 filas, el guard se reporta pero no veta (se declara).
- **`brazos.py`/`metricas.decide` sin commitear y sin tests-oráculo por brazo
  antes de la primera captura sobre el gold** (F4, review de rama
  2026-09-20, §10): STOP, la fase 2 no captura nada. F1/S1/P1/G1 son prosa
  de §4 hasta que ese commit exista; congelar la medición con esas
  definiciones sin fijar en código sería congelar prosa, no una regla de
  decisión verificable. Si `brazos.py` o `metricas.decide` cambian
  **después** de la primera captura, esa medición queda invalidada (no se
  recomputa con el código nuevo): se declara qué versión produjo qué
  resultado y, si el cambio es necesario, se repite la captura entera.
- **Kimi:** si el humo de 3 paquetes falla por `response_format` o el modelo
  elegido por la regla no existe, STOP y PENDIENTE-PAUL (no se improvisa
  modo de salida ni modelo). Coste acumulado > $10: STOP. Cualquier fila sin
  juicio válido de **ambos** jueces tras 3 reintentos queda fuera del gold y
  se cuenta como descarte «sin juicio de ambos». **El tope de 3 reintentos es
  por corrida, no por fila a lo largo de toda la campaña** (F6, review de
  rama 2026-09-20): `procesa_lote` es reanudable por id (`juez.py`), y una
  reanudación vuelve a intentar los ids sin juicio válido con otros 3
  reintentos propios, sin memoria de los intentos de corridas anteriores —
  así que cuántas veces se reintentó de verdad una fila antes de que se
  cuente como descarte depende de cuántas veces el operador reanudó el
  pipeline, una decisión que hoy no queda registrada. Antes de la Task 7 (si
  se ejecuta): fijar un máximo de reanudaciones por el pipeline completo (no
  solo por llamada), o registrar explícitamente en el ledger cuántas
  reanudaciones tuvo cada corrida de Kimi.
- **Fuga de la key o de datos (F6, sincronizado con Global Constraints el
  2026-09-20 — el gate viejo de esta línea nacía en rojo):** si
  `LC_ALL=C git grep --untracked -cE 'sk-[A-Za-z0-9]{20,}' -- evals` da algo
  distinto de "sin salida" (exit 1), o si `grep -c wisdom-paul <fichero>` da
  distinto de 0 sobre cada `.md` que se vaya a commitear bajo `evals/`
  (`verdict/`, `gold-j/`) — **no** sobre este pre-registro ni sobre otros
  ficheros de `docs/superpowers/plans/`, que sí citan `wisdom-paul` por
  nombre y no son el fichero público cuya fuga se vigila —, el commit no se
  hace; si ya se hizo, se reescribe la rama antes de cualquier push. El
  patrón `'sk-'` a secas nacía en rojo el mismo día que se escribió (3 falsos
  positivos preexistentes en `evals/prep-m3/`, p.ej. «task-specific»
  contiene el substring `sk-`): un gate que nace en rojo se normaliza y deja
  de proteger, por eso el patrón exige forma de key de Moonshot.
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
  o comparaciones después de congelar, re-juzgar filas a la vista de
  resultados, decidir por estrato o por celda cruzada. Tras el verdict, el
  gold de J queda **consumido**: cualquier ajuste posterior (segunda fusión
  candidata, otro factor de `archive/`, otro τ) necesita un gold nuevo.
- **Si el ganador de D-A/D-B exige reescribir la parada del KNN** (H29,
  `busca_vector_con_embedding`: la cota `limite` solo vale con `bonus == 0`
  y `max`), la fase 2 lo trata como en C (fila PR #12 del plan de C): lista
  vectorial exhaustiva o cálculo de `v` para los ≤50 candidatos FTS por
  `permalink IN (…)`, con test de equivalencia exacta antes de mergear.
