# Verdict — gate M0 sobre ground truth eval.jsonl (retrieval-fase0)

- **Fecha**: 2026-07-17
- **Gate**: consultor-gate (régimen §8, adjudicación en lugar de Paul)
- **Objeto**: `~/Documentos/proyectos/exo/evals/retrieval-fase0/eval.jsonl` (56 filas)
- **Método**: solo filesystem sobre `~/Documentos/proyectos/kb-demo/` (Read/Grep). Cero tools MCP de basic-memory, cero escrituras en kb-demo. Permalinks verificados contra `grep "^permalink:"` en frontmatter de toda la KB.
- **Muestra**: 17 filas = fila null (9) + 5 hard (47, 51, 53, 54, 56) + 11 log (4, 5, 10, 12, 13, 19, 21, 24, 29, 36, 41), incluyendo todas las ambiguas señaladas (pares cge/cgeo, canon-vs-bitácora, portfolio-game).

## Verificación de existencia de permalinks

Los 16 permalinks no-null muestreados existen EXACTOS en el frontmatter de la KB (campo `permalink:`). Sin desviaciones slug/título.

Nota lateral (no afecta al eval): `docs/superpowers/plans/2026-07-03-memoria-v2.md` contiene líneas `permalink:` duplicadas embebidas en su cuerpo (líneas 44 y 84, ejemplos del plan) que replican `kb-demo/core/core-index` y `kb-demo/core/doctrina-agentes`. Un indexador ingenuo que grepee frontmatter sin delimitar `---` podría duplicar. Ninguna fila muestreada depende de ello.

## Adjudicación por fila

### Fila 9 (null) — "benchmark-methodology" → `null`
**CORRECTO.** Grep exhaustivo (`benchmark.?methodolog|metodología de benchmark`, case-insensitive) sobre toda la KB: 0 ficheros. El listado completo de permalinks confirma que no existe nota ni slug equivalente. `kb-demo/metodologia` es explícitamente "cómo se generó esta knowledge base" (pipeline multi-agente de generación de la KB), no metodología de benchmarking — el razonamiento de las notes es exacto. La metodología de benchmarks vive fragmentada en bitácoras (cge harness v2/v3, lighthouses CRN), ninguna es "la" nota. Null defendible y bien argumentado.

### Fila 4 — "ai-news plataforma noticias" → `projects/ai-news-pipeline`
**CORRECTO.** Título literal "AI News Pipeline — Del script Telegram a plataforma con MCP público"; la nota ES la plataforma (FastAPI + pgvector + React + MCP público). Verificado el descarte de la alternativa: `projects/wisdom-ai-news` se autodefine como "POC de la máquina genérica wisdom-X aplicado al proyecto [[ai-news-pipeline]]" — cerebro de dominio SOBRE la plataforma, no la plataforma.

### Fila 5 — "ai-news-bitacora" → `log/ai-news-pipeline-bitacora`
**CORRECTO.** El listado de permalinks confirma que es la única bitácora de la línea ai-news (no existe `ai-news-bitacora` ni bitácora de wisdom-ai-news). Frontmatter: `title: ai-news-pipeline-bitacora`, tags `ai-news-pipeline, bitacora`.

### Fila 10 — "blog notas publicar contenido web pguerrero divulgación posts" → `projects/pguerrero.me-hub-personal-portfolio-con-lab-explorable-de-llms`
**CORRECTO (con la tensión que las propias notes declaran).** La nota dice literal: "hub personal / portfolio, no blog ni landing de producto" y "La divulgación de IA es leitmotiv (tono), no estructura". La query asume blog/posts que la nota niega, pero es la única nota sobre la web personal de Paul y un usuario razonable que busque "web pguerrero divulgación" quiere exactamente esta. La honestidad de las notes sobre la fricción query↔nota es un punto a favor del ground truth.

### Fila 12 — "cge bitácora" → `log/cge-bitacora` (par cge/cgeo)
**CORRECTO.** `log/cge-bitacora.md` se autodefine "Bitácora de [[cge — motor code-graph en Go]]" — el motor activo post-independencia. La alternativa `log/code-graph-engine-bitacora` es la bitácora del motor Node (cgeo, jubilado). "cge" hoy nombra al Go; la elección es la correcta, no solo defendible.

### Fila 13 — "cge evaluación head-to-head cgeo benchmark harness metodología" → `log/code-graph-engine-bitacora` (ambigua declarada)
**DEFENDIBLE, la más floja de la muestra.** Evidencia a favor: la bitácora de cgeo acumula 7+ entradas fechadas de head-to-head (spike GO, M3 gates vs cmm, reviews ampliados 2/6 repos, benchmark utilidad-agente v1) — es la fuente histórica de esa metodología. Evidencia en contra: el término "harness" solo aparece en `log/cge-bitacora` (harness v2/v3, benchmark A/B pre-registrado, $61,63/$116,58), y la query nombra "cge" primero. Un retriever que devuelva cge-bitacora sería castigado siendo razonablemente correcto. Las notes documentan las 3 alternativas con criterio explícito → lo doy por defendible como single-label, pero si el harness de scoring admitiera aceptables-secundarios, esta fila debería llevar `log/cge-bitacora` como alternativa aceptada. No bloquea.

### Fila 19 — "codebase-memory-mcp cmm bugs" → `log/code-graph-engine-bitacora` (canon-vs-bitácora)
**CORRECTO.** La bitácora cataloga bugs de ambos lados con fechas: "seam HTTP de cmm roto (~16% precisión)" (entrada 2026-07-02), bug de layout `src/` (2026-06-29), poison `tsBuildFieldEnv` (2026-07-01). El canon resume estado, no el catálogo histórico — la elección bitácora para una query de "bugs" (plural, histórico) es la correcta.

### Fila 21 — "coste workflows multi-agente tokens lección" → `learnings/desarrollo-agentico` (ambigua declarada)
**CORRECTO.** La nota contiene la pirámide de coste como doctrina destilada: "La pirámide de coste no es 'usa el modelo más capaz': es distribuir el gasto donde hay juicio irreducible, no donde hay transcripción". La query pide "lección" → learnings gana sobre `metodologia` (que documenta costes de un pipeline concreto). Routing correcto por tipo de contenido.

### Fila 24 — "fabrica campaña" → `projects/agent-develop` (canon-vs-bitácora)
**DEFENDIBLE.** El canon aloja y define el skill `fabrica` ("protocolo de sesión-fábrica — Paul fuera del critical path, gate de merge asíncrono vía línea GATE:", estructura `skills/fabrica/SKILL.md`, guard hook). Query genérica sin ancla temporal → canon sobre bitácora es coherente con el contrato memoria v2 (canon = estado vigente; bitácora = histórico). La bitácora sería igual de razonable, pero la regla "query genérica → canon" es un criterio consistente y las notes lo explicitan.

### Fila 29 — "Frente 9 lighthouses Fase 4 divergencia core split thin-core" → `log/lighthouses-bot-bitacora` (ambigua declarada, 3 candidatos)
**DEFENDIBLE.** La bitácora tiene los encabezados dedicados: "2026-07-12 — fábrica campaña 3: fundación experimental + core-split decidido (thin-core)" y "2026-07-13 — fábrica Campaña 5: Fase 4 divergencia MERGEADA". El Backlog tiene "Frente 9 — lighthouses-bot" literal pero como una línea de estado, no el detalle que la query enumera (Fase 4, divergencia, core split, thin-core — todos términos de las entradas de bitácora). La mayoría de los términos de la query resuelven a la bitácora → elección defendible y probablemente óptima.

### Fila 36 — "lighthouses contest bot Horus MadeInHeaven Pegasus e33" → `projects/lighthouses-bot` (canon-vs-bitácora)
**CORRECTO.** El canon cubre los 4 términos: rename documentado ("submission RENOMBRADA a MadeInHeaven... el harness lo sigue etiquetando Horus"), Pegasus como rival real ("IronBot mk4 == Pegasus, el MISMO algoritmo con dos nombres"; monocultura Pegasus), y el path e33 en frontmatter (`e33-scripts/lighthouses_aicontest`). Query de entidades/estado → canon correcto.

### Fila 41 — "portfolio-game canvas físicas motor plataformas" → `projects/frontend-creativo` (portfolio-game)
**CORRECTO.** El canon documenta portfolio-game en detalle: "Angular 21 zoneless + física custom, estilo Jump King", `PhysicsService` (gravedad/fricción/AABB/wall jump), `GameLoopService`, torre de plataformas por secciones. El término "canvas" de la query es impreciso (el render de portfolio-game no es canvas; el canvas hand-rolled es foss-jam-kit, que tiene canon propio `projects/foss-jam-kit`), pero el proyecto nombrado es inequívoco y vive aquí. Elección correcta pese al ruido léxico.

### Fila 47 (hard) — "¿dónde compensa meter IA generativa y dónde reglas fijas baratas?" → `learnings/construir-con-llms`
**CORRECTO.** Árbol de decisión literal en la nota: "¿Necesito LLM aquí? → ¿Hay una regla determinista que cubra ≥80% de los casos?" + sección "When not to add LLM". Match exacto de intención.

### Fila 51 (hard) — "cambio tres líneas y se reprocesa la KB entera; solo recalcular lo afectado" → `learnings/ingesta-incremental-en-pipelines-llm-...`
**CORRECTO.** Título "Ingesta incremental en pipelines LLM — mantener una KB al día sin re-procesar todo" y motivación ("minar un repo entero tardó ~100 min en frío; ¿viable cuando las fuentes cambian?") coinciden exactamente con la query.

### Fila 53 (hard) — "torneo de juegos donde el jugador que aprendía solo ganaba menos que la versión de reglas sencillas" → `projects/lighthouses-bot`
**CORRECTO.** Literal en el canon: "Osiris (TD + Thompson + Softmax) dominaba en 2P pero caía a 6.7% win rate en 8P vs 36.7% de Anubis (solo 2 hard gates)"; MCTS también descartado. El rollback está documentado. Match exacto.

### Fila 54 (hard) — "herramienta que revisa usabilidad por discapacidad y evita repetir comprobaciones en páginas sin cambios" → `projects/accesibilidad-saga`
**CORRECTO.** Literal: "La última iteración (Differential Probe, v4.1+) aplica la lógica de git al testing de accesibilidad: no re-auditar lo que no cambió"; auditoría WCAG 2.2. Match exacto.

### Fila 56 (hard) — "app de inversión con pantalla distinta a novatos/medios/expertos, mismo código, sin ifs regados" → `projects/finanzas-empresa-x`
**CORRECTO.** Literal: "tres interfaces completamente distintas (principiante/intermedio/avanzado) sobre el mismo código, sin condicionales dispersos, usando un patrón viewConfig centralizado". Match exacto.

## Veredicto final

**APROBADO.** 17/17 filas muestreadas correctas o defendibles; 0 correcciones requeridas. Los 16 permalinks no-null existen exactos en frontmatter; el null (fila 9) está bien adjudicado. Las elecciones canon-vs-bitácora siguen un criterio consistente (query genérica/estado → canon; query histórica/detalle fechado → bitácora) y las notes documentan alternativas con honestidad — incluidas las fricciones query↔nota (filas 10, 41), lo que eleva la confianza en el resto del set no muestreado.

**Observación no bloqueante** (para el harness de scoring, no para el ground truth): la fila 13 es la única donde un retriever razonable podría devolver la alternativa (`log/cge-bitacora`) y ser castigado. Si en fases posteriores el eval admite `acceptable_permalinks` secundarios, empezar por ahí.
