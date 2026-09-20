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
más, cota del plan). **Riesgo real sobre el suelo de no nulas (60, §11):**
sumando los rangos «no nulas esperadas tras acuerdo» de §3 del pre-registro
para los estratos presentes — `prompt` 20-30, `hard` 20-25, `archive` 12-16,
`negativo` 0 — el rango sin `agent-search` es **52-71 no nulas**, que
**se solapa con el suelo de 60 sin garantizarlo**: en el peor caso del rango
(52) el gold no llegaría al suelo y J PARA; en el mejor caso (71) sí. Con
`agent-search` (que aporta 40-70 no nulas esperadas) el objetivo total
92-141 deja margen mucho más cómodo sobre el suelo. Juzgar sin `agent-search`
es una apuesta real, no una formalidad.

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
