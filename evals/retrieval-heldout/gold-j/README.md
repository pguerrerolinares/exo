# Gold J — cómo se construyó (100 % agéntico, fiabilidad medida)

Contrato: `docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md`
§3 (estratos, jueces, suelo) y §11 (kill-criteria). El gold vive fuera del
repo (`~/.local/share/exo-evals/j-heldout/gold-j.jsonl`); aquí solo el método,
la plantilla y los recuentos. Decisión de Paul del 2026-09-19 (`propuesta.md`
§7): la KB la consumen agentes, no Paul; el patrón de relevancia es la
utilidad para el agente, y el gold lo etiquetan jueces LLM de dos familias.

## Pipeline

1. **Queries** por estrato: `prompt` (prompts reales que dispararon
   `recall-inject` desde el 2026-09-13, `pool.py`), `agent-search` (comandos
   `exo search` reales de agentes, minados de los transcripts con autorización
   de Paul y limpiados con `limpia_agent_search.py`: criterio en su docstring;
   **estrato pendiente en este kit, ver «Estado del kit» abajo**), `hard`
   (paráfrasis por un agente que no vio ningún gold), `archive` (hechos que
   hoy viven solo en `archive/log/`), `negativo` (temas ausentes; ausencia
   comprobada con `grep` de `topic_terms` sobre la KB entera y, además, por
   los jueces). Anti-fuga: nada con Jaccard ≥ 0,8 frente a las 55 ni a las
   147 de C.
2. **Candidatos** (≤ 5 por query) por un agente filesystem-only, con al menos
   2 por navegación temática y no por grep. Es el cuello léxico declarado.
   Se despachó en 4 lotes de ≤ 60 filas, cada uno un subagente fresco
   distinto del generador de queries, sin acceso a `author_expected` ni
   `topic_terms` (se le pasó una copia de `queries.jsonl` con solo
   `id`/`query`/`source`, para no depender solo de la disciplina del brief).
3. **Dos jueces ciegos e independientes** sobre el mismo paquete (query +
   candidatos con título y primeros 4.000 caracteres; sin etiquetas, sin
   rankings, sin el otro juez): **fable** (Claude) y **Kimi** (Moonshot,
   `juez.py`, modelo fijado en el pre-registro §10). Kimi recibe trozos de la
   KB: **envío autorizado explícitamente por Paul el 2026-09-19**.
4. **Acuerdo** (`acuerdo.py`): una fila entra solo si los dos jueces
   coinciden (estricto, o el expected de uno es aceptable para el otro). Los
   desacuerdos se descartan (sin tercer juez) y se cuentan. Suelo
   pre-registrado: κ de Cohen ≥ 0,60 y acuerdo ≥ 0,70 sobre todas las filas
   juzgadas; por debajo, J PARA (es un resultado).
5. **Auditoría del sesgo léxico**: solape query↔nota en acordadas frente a
   descartadas, y el subconjunto «léxicamente difícil» como guard de D-A.

## Estado del kit — qué falta para juzgar

Este kit **no incluye juicio** (Task 7 del plan). La fábrica paró el juicio
con Kimi porque el control de gasto del breaker de `juez.py` tiene un fallo
detectado en review adversarial (el libro de gasto puede mentir en un caso
concreto); el arreglo de ese breaker es de otro frente y no se ha tocado
aquí. Nadie llamó a ninguna API de pago para construir este kit.

**Además, el estrato `agent-search` no se pudo generar.** El Step 3 del plan
(re-extracción de comandos `exo search` reales desde
`~/.claude/projects/**/*.jsonl`, con autorización de Paul del 2026-09-19)
está bloqueado: el clasificador de permisos del harness deniega la lectura
de esa ruta por motivo «PII Data Handling» / «Sensitive-Source Provenance»,
tanto con el script del plan como con un `grep` directo. Es un bloqueo
distinto de la autorización de D-J11 — D-J11 autoriza el *diseño* del
experimento (minar esas queries), no es un permiso del harness sobre sus
propios transcripts, y el clasificador no lo reconoce como tal. No se usó el
fichero crudo preexistente de 221 líneas (`~/.exo/priv-j/agent-search-raw.txt`,
de la sesión de planificación): el propio plan lo describe como truncado por
el escape `\"` de `grep`, así que meterlo habría sido peor que no tenerlo —
parecería un estrato válido sin serlo.

Este kit se generó **sin `agent-search`**: `queries.jsonl`, `candidatos.jsonl`
y `paquetes.jsonl` tienen 229 filas (`prompt` 139 · `hard` 30 · `archive` 20 ·
`negativo` 40), no 289-349 como habría sido con `agent-search` (60-120 filas
más, cota del plan). **Riesgo real sobre el suelo de no nulas (60, §11)**
(actualizado con el recall real del kit medido en «Recall del kit» abajo —
`archive` 14/20, no el 20/20 que el §3 asumía en el mejor caso):

- **Recuento estimado de no nulas con el kit tal cual: ≈ 52-73, mediana ≈ 64**
  (`prompt` 20-30 + `hard` 20-25 + `archive` recortado por el recall real a
  ≈ 8-14 en vez de 12-16 + `negativo` 0). **Probabilidad de quedar por
  debajo del suelo de 60: ~35-45 %** — no es una formalidad, es una apuesta
  real con el kit en su estado actual.
- **Si se repite el Step 7 del bloque `archive` (`q170`-`q189`, ver «Recall
  del kit» arriba) antes de juzgar:** el rango sube a **≈ 55-76** y el
  riesgo de no llegar al suelo baja a **~30 %**. Barato (un lote de 20
  candidatos con un agente fresco) frente al coste de fallar (abajo).
- **El otro suelo que nadie citaba explícitamente: `archive` ≥ 8 filas no
  nulas** (§11, `no_nulas_por_estrato.archive`) para que D-C (la decisión
  sobre penalizar `archive/`) pueda tomarse. Lo esperado tras acuerdo sobre
  las 14 candidatas reales son **10-12 filas**: pasa, pero **sin margen** —
  un acuerdo algo peor de lo esperado en ese estrato concreto (no en el
  total) deja a D-C sin decidir, aunque el gold global sí llegue al suelo
  de 60.
- **Lo que se pierde si el suelo global falla NO es el dinero:** el kit tal
  cual son ≈ 0,45-0,6 M tokens de entrada para Kimi (ver «Recuentos» abajo)
  ⇒ **≈ $2 con kimi-k3, no los ≈ $6** que cita la Task 7 Step 3 del plan
  (esa cifra es para el job completo de ≈ 280 filas CON `agent-search`, que
  este kit no tiene). Lo que se pierde de verdad: **las 229 queries quedan
  consumidas** (§11, «Prohibido: … re-juzgar filas», y el gold entero queda
  «consumido» tras el verdict) — no se pueden re-juzgar ni reciclar en un
  segundo intento, y **`agent-search` quedaría fuera de este gold para
  siempre** (el estrato con las únicas queries reales de agentes, D-J11):
  cualquier necesidad futura de retrieval-para-agentes exigiría un gold
  nuevo desde cero, no una ampliación de este.
- **Recomendación del review (no del kit — la decisión es de Paul):**
  esperar a que se resuelva el permiso de lectura de `agent-search` (punto 1
  de «Para completar» abajo) antes de juzgar. Es el único estrato con
  queries reales lanzadas por agentes en producción; sin él, el gold
  responde una pregunta distinta de la que D-J11 firmó (relevancia para
  `prompt` + generadas, no para el patrón de uso real de `exo search` por
  agentes). Con `agent-search` (40-70 no nulas esperadas) el objetivo total
  92-141 deja margen mucho más cómodo sobre el suelo, y el riesgo de arriba
  desaparece. Pero esto es una recomendación, no un bloqueo: si Paul decide
  juzgar con el kit tal cual (o tras repetir el Step 7 de `archive`),
  asumiendo el ~30-45 % de riesgo y la pérdida irreversible de las 229
  queries si falla, esa es una decisión válida y suya.

**Para completar, el día que Paul decida:**

1. Resolver el permiso de lectura de `~/.claude/projects` (o decidir
   explícitamente juzgar sin `agent-search`, asumiendo el riesgo del párrafo
   anterior).
2. Si se resuelve (1): correr el Step 3 del plan (Task 6) para producir
   `agent-search.jsonl`, añadir esas filas a `queries.jsonl` (continuando la
   numeración `qNNN` a partir de `q230` para no reordenar ni tener que
   regenerar los candidatos/paquetes ya hechos — desviación deliberada del
   orden `prompt → agent-search → hard → archive → negativo` de §3, anotada
   aquí), generar sus candidatos (Step 7) y añadirlos a `candidatos.jsonl`, y
   regenerar `paquetes.jsonl` completo con `juez.py paquetes`.
3. Que el arreglo del breaker de `juez.py` esté cerrado y mergeado (otro
   frente de la fábrica).
4. Correr el juicio (Task 7 del plan), con `$PRIV_J = ~/.local/share/exo-evals/j-heldout`:

   ```bash
   # fable: subagente fresco por lotes, ve solo paquetes.jsonl (brief exacto en plan, Task 7)
   # kimi:
   python3 evals/retrieval-heldout/harness/juez.py modelos --env-keys /home/paul/Documentos/proyectos/wisdom-ai-news/.env-keys
   python3 evals/retrieval-heldout/harness/juez.py kimi \
     --paquetes "$PRIV_J/paquetes.jsonl" \
     --env-keys /home/paul/Documentos/proyectos/wisdom-ai-news/.env-keys \
     --model <id fijado en el paso anterior, kimi-k3> \
     --out "$PRIV_J/kimi.jsonl" \
     --precio-entrada 3.00 --precio-salida 15.00 --tope-usd 10
   # acuerdo:
   python3 evals/retrieval-heldout/harness/acuerdo.py \
     --candidatos "$PRIV_J/candidatos.jsonl" \
     --fable "$PRIV_J/fable.jsonl" --kimi "$PRIV_J/kimi.jsonl" \
     --gold-out "$PRIV_J/gold-j.jsonl" \
     --descartes-out "$PRIV_J/descartes.jsonl" \
     --informe-out "$PRIV_J/informe-acuerdo.md" \
     --snap "$PRIV_J/kb-snap" --kappa-min 0.60 --po-min 0.70
   ```

   Precondición de `--tope-usd 10`: el presupuesto autorizado (plan, Global
   Constraints) es tope $10, gasto esperado ≈ $6 con kimi-k3.

## Recall del kit (generador de candidatos) — F7, review de rama 2026-09-20

Métrica del kit: fracción de queries cuyo `author_expected` (la nota que el
generador de queries de `hard`/`archive` tenía en mente al escribirla, no
visto por el agente de candidatos) aparece entre las ≤5 candidatas que ese
agente propuso. Es el techo de no-nulas que el paso de juicio puede alcanzar
para esos dos estratos: si `author_expected` no está en `candidatos`, ningún
juez -por bueno que sea- puede recuperarlo como `expected`.

- **`hard`: 29/30.** Sano: muy por encima del rango 20–25 no nulas esperado
  tras acuerdo (§3 del pre-registro).
- **`archive`: 14/20** (bloque `q170`–`q189` completo). Bajo el rango 12–16
  esperado en el mejor caso, y a un solo miss adicional del suelo de 8 filas
  archive del §11 (`no_nulas_por_estrato.archive`) por debajo del cual D-C no
  se decide. Con acuerdo lenient imperfecto (< 100 %) sobre esas 14, el
  estrato `archive` real del gold puede terminar más cerca de 10–11 filas
  útiles que de las 12–16 que el §3 espera.

**Antes de juzgar** (no ejecutado aquí, documentado como paso previo): repetir
el Step 7 del plan (generación de candidatos) **solo para el bloque
`q170`-`q189`** con un agente de candidatos fresco, verificando mecánicamente
que aplicó la regla de §3 «la rotación archivada **y** la bitácora viva
cuando la query menciona un hecho fechado» (el motivo más probable del
recall bajo en `archive` es no incluir ambas). **Lo que NO vale**: inyectar
`author_expected` en la lista de candidatos para forzar el recall a 20/20 —
eso rompería la ceguera del generador de candidatos (§3: «sin ver
`author_expected` ni `topic_terms`») y invalidaría el estrato entero, no lo
arreglaría.

## Recuentos (kit sin `agent-search`; sin texto de queries)

- snapshot `S_J`: `6bf57d513dc2ad53e815a4debafe6982f6998151` (detached en
  `$PRIV_J/kb-snap`, 32 ficheros en `archive/log`)
- `queries.jsonl`: 229 filas · por estrato `prompt 139 · hard 30 · archive 20
  · negativo 40 · agent-search 0 (PENDIENTE)`
- `candidatos.jsonl`: 229 filas (1:1 con `queries.jsonl`) · con lista de
  candidatos vacía: 66 (40 del estrato `negativo`, donde vacía es el
  resultado esperado; 26 del estrato `prompt`, sin candidata plausible
  encontrada por el agente de candidatos — `hard` y `archive` no tuvieron
  ninguna vacía, esperable porque su `author_expected` viene de una nota
  real del snapshot)
- `paquetes.jsonl`: 229 paquetes · chars por paquete: p50 8.296 · p90 16.463
  · máx 21.287 · total 1.782.082 (≈ 0,45-0,6 M tokens estimados para Kimi;
  bajo porque falta `agent-search`)
- acuerdo / gold: no calculado — Task 7 no se ejecutó (ver «Estado del kit»)
- coste Kimi: $0 — ninguna llamada a la API en este kit
