# Informe — consultoría adversarial #5: Milestones, dependencias y riesgos (Sección 5)

Consultor independiente. Verificación primaria: los 4 informes previos del scratchpad (framework, engine, thin, thick), reflex plugin real (hooks.json, reflex-baseline.sh, basic-memory-recall.sh, consolida SKILL.md), logs vivos (`~/.claude/reflex-log.jsonl` — 656 git-c + 77 zero-residuo a hoy; `reflex-retrieval-log.jsonl`), el documento de pre-registro (`agent-develop/docs/superpowers/evals/2026-07-09-reflex-v2-baseline.md`), Backlog completo de la KB, core-index/doctrina-agentes, crontab, known_marketplaces.json, enabledPlugins de settings.json, remotes de kb-demo/kbx/agent-develop y el binario kbx instalado.

**Veredicto global: el grafo de milestones es correcto en topología pero está mal cosido contra el reloj. La Sección 5 gatea SOLO M6 con la métrica D, cuando el pre-registro del experimento exige literalmente que el rol `executor` sea "el ÚNICO cambio en la ventana" — y M0, M1 y M3, tal como están calendarizados, caen todos dentro de esa ventana. Además hay dos contradicciones internas de secuencia: M2 promete un cutover (recall) que pertenece a M6, y M5 desinstala basic-memory antes de que M6 haya migrado los scripts que dependen de él. Las tres cosas se arreglan con re-redacción y una espera de 7 días, no con rediseño.**

---

## Eje 1 — Interferencia M0 ↔ métrica D: LA MÉTRICA ES CASI INDEPENDIENTE DE M0, PERO EL PRE-REGISTRO NO PERDONA, Y EL REPLAY CONTAMINA OTRO ACTIVO

**Qué mide exactamente la métrica D (verificado en el doc de pre-registro y en reflex-baseline.sh):** reincidencia de violaciones git-c/zero-residuo por sesión, media semanal, sobre `~/.claude/reflex-log.jsonl` (disparos de los sensores PreToolUse:Bash), ventana 2026-07-09 → medir ≥2026-07-23, contra baseline pre-registrado (git-c 295, máx 142/sesión; zero-residuo 16). Criterio de éxito literal: *"caída medible con el rol `executor` como único cambio en la ventana"*.

**¿Contamina el cambio de embeddings?** Por el canal directo, NO: los sensores miden conducta Bash del agente, cuya doctrina viaja por (a) system prompt del rol `executor` y (b) el core-index que `basic-memory-recall.sh` inyecta vía `read-note core/core-index` — un lookup determinista por permalink que el modelo de embeddings no toca. El embedding solo afecta al ranking de `search_notes` (48 usos en 11 días, ninguno en el camino causal de git-c/zero-residuo). La métrica es de facto independiente de M0.

**Pero hay dos contaminaciones reales que el diseño no ve:**

1. **El replay de M0 contamina el retrieval-log, que es la materia prima del eval set.** Si las 46 queries se re-lanzan vía tools MCP en una sesión normal, el hook `retrieval-logger.sh` (matcher `mcp__basic-memory__search_notes`) las appendea a `reflex-retrieval-log.jsonl` — el mismo log del que se extrajo el eval set. Queries sintéticas × N modelos = el log deja de reflejar uso real y cualquier re-extracción futura queda envenenada. **Fix barato**: snapshot del log antes del replay + replay vía CLI (`basic-memory tool search-notes`), que no pasa por hooks MCP. De paso, las sesiones de trabajo de M0 conviene nombrarlas `test-*` (el FILTER de reflex-baseline.sh las excluye por prefijo, verificado en el script).

2. **El pre-registro es más estricto que la independencia causal.** "Único cambio en la ventana" es la frase firmada. M0 (config de retrieval), M1 (mover el repo que sirve el marketplace de reflex) y M3 (cutover del framework de skills entero) dentro de la ventana rompen esa cláusula aunque cada uno sea plausiblemente ortogonal. Con un experimento pre-registrado no negocias post-hoc qué confound "no cuenta" — esa es exactamente la disciplina que Paul practica en cge (criterios pre-registrados, erratum cuando su eval se equivocó). La salida no es esperar semanas: **la ventana cierra midiendo, y se puede medir el 2026-07-23 — dentro de 7 días.** Recomendación: M0 se hace ya con la higiene del punto 1 (defendible: no toca el canal causal y se documenta como desviación menor pre-declarada), y TODO lo demás que altera el entorno del agente (M1-marketplace, M3, el flip de recall) espera al 23. Es un delay de una semana que compra la validez del experimento que lleva armándose desde junio.

**Detalle operativo que el backlog ya exige y el roadmap debe heredar:** antes de calcular D, fix del `jq 2>/dev/null` de reflex-baseline.sh (una línea corrupta trunca el análisis en silencio). Tocar ese script no viola el freeze — es read-only de análisis, no conducta.

**Veredicto eje 1: M0 puede ir esta semana** (con replay vía CLI + snapshot del log + sesiones `test-*`); **la que NO puede ir esta semana es cualquier pieza que toque marketplace, skills o recall**. El roadmap solo gatea M6 con la métrica D — insuficiente; ver ejes 3 y 4.

---

## Eje 2 — M3 "no depende del engine": CIERTO EN LO SUSTANTIVO, CON DOS MINAS QUE NADIE HA LISTADO

**Lo verificado a favor:**
- `/documenta` escribe hoy vía MCP basic-memory (`edit_note` append; verificado en informes previos y en el comando). Post-M3 y pre-M4 seguiría igual: el matcher `mcp__basic-memory__write_note` de search-before-write (hooks.json de reflex) sigue disparando. Coherente.
- `consolida` consume kbx por **path absoluto** (`/home/paul/.local/bin/kbx`, verificado en su SKILL.md, binario instalado el 07-12). El binario instalado es independiente del repo: M1/M3 no lo rompen. Sus escrituras son ediciones de fichero + `git mv` — file-first ya, ni siquiera pasa por el write-path de basic-memory.
- El overlay pre-M2 funciona sin engine: el patrón probe→defaults ya está probado en producción (documenta: "kbx unavailable → fallback visible"), y el informe thin ya fijó que "vía engine" = "vía kbx/filesystem hasta E1". La KB es markdown local: leer el perfil es un Read de fichero; no hay dependencia técnica del engine en M3.

**Mina 1 — consolida vive DENTRO del plugin reflex** (`agent-develop/plugins/reflex/skills/consolida/SKILL.md`, verificado). Si M3 mete "documenta/consolida en process", mover consolida = editar el plugin reflex + bump de versión = violar el freeze si ocurre antes del cierre de la ventana, y en cualquier caso es un cambio de reflex que el grafo asigna a M6, no a M3. Opciones: (a) consolida se queda en reflex hasta M6 y process nace con documenta pero sin consolida; (b) el traslado de consolida se hace el día del cutover M3 SI ese día ya es post-medición. La Sección 5 no dice cuál — debe decirlo.

**Mina 2 — el dispatch del rol `executor`.** reflex v2 ancla su mecanismo (doctrina en system prompt) en que `orchestrate-personal` (paul-profile 0.3.0) despacha `subagent_type: reflex:executor` sin `model`. M3 absorbe orchestrate-personal en `process:orchestrate`. Si la skill nueva no hereda ese dispatch exacto, **el mecanismo entero de reflex v2 se desenchufa en silencio** — la doctrina deja de llegar a los ejecutores, la reincidencia vuelve, y parecerá que reflex v2 fracasó. La checklist de cutover del informe thin lista fabrica, workflow-lint, settings y orchestrate-personal, pero NO este dispatch. Es el ítem más peligroso de la checklist porque su fallo es invisible hasta que alguien mire el log. Añadir: "process:orchestrate despacha reflex:executor sin model (paridad con paul-profile 0.3.0)" + un probe post-cutover (mismo smoke que el despliegue del 07-09: verificar `agent_type` en reflex-log).

**Veredicto eje 2: el claim "M3 no depende del engine" es CIERTO; lo que M3 sí tiene son dos dependencias de reflex no declaradas** (ubicación de consolida, dispatch del executor). Con ambas en la checklist, ratificado.

---

## Eje 3 — Orden M1 antes de M2: EL ORDEN ES CORRECTO; EL RIESGO NO ES REWORK, ES EL MARKETPLACE EN MITAD DE LA VENTANA

**¿Rework por no saber el lenguaje?** No. M0 (una tarde, esta semana) emite la decisión de lenguaje ANTES de que M1 arranque de verdad; y aunque no lo hiciera, añadir un crate Rust o un módulo Go a un monorepo después es coste cero — la estructura del repo no se casa con el lenguaje del engine (kbx entra tal cual en Go, firmado). El contenido real de M1 que SÍ debe preceder a M2 es la higiene pre-baseline (backfill de las 11 notas sin `type`, root files, decisión dotdirs — verificadas en el informe thick): sin eso el side-by-side de E1 nace comparando corpus sucio. El orden M0→M1→M2 es correcto.

**¿Absorber agent-develop es reversible?** Verificado el acoplamiento real: el marketplace está registrado como GitHub `pguerrerolinares/agent-develop` con `autoUpdate: true` (known_marketplaces.json), y settings.json tiene 3 plugins `@agent-develop` activos (reflex, paul-profile, workflow-lint) — **reflex, el plugin bajo medición, se sirve desde ese marketplace**. Escenarios:
- *Rename del repo GitHub*: los fetch siguen funcionando por redirect de GitHub; la identidad `@agent-develop` de los plugins instalados no cambia. Reversible y de bajo riesgo, pero deja el nombre viejo fosilizado en settings/known_marketplaces hasta una re-registración.
- *Repo nuevo + archivo del viejo*: exige re-registrar marketplace + flip de TODAS las entradas enabledPlugins — exactamente la clase de operación que puede dejar reflex desactivado un rato o servido desde un clone stale (los gotchas de cache-por-versión y clone shallow ya mordieron en el despliegue del 07-09, documentados en la bitácora de reflex).

La conclusión no es "no absorbas": es **separar M1 en dos**: M1a (crear monorepo, mover historia, higiene KB) — puede ir ya, no toca nada instalado; M1b (re-registrar/renombrar el marketplace que sirve reflex) — va pegado al día del cutover M3, post-medición. La Sección 5 trata M1 como atómico y no menciona el marketplace; el informe thin ya pedía esa decisión ("¿repo nuevo o agent-develop renombrado?") y la integración la ha dejado caer.

**Veredicto eje 3: orden ratificado; M1 debe partirse en M1a/M1b con el marketplace explícito y M1b gated por el cierre de la métrica D.**

---

## Eje 4 — "Cada milestone deja el sistema funcionando": DOS AGUJEROS REALES, UNO DE ELLOS ROMPE EL CLAIM

**Agujero 1 (contradicción interna de la Sección 5) — M2 promete "Recall de SessionStart servido por engine", pero el recall es `basic-memory-recall.sh`, un script DEL PLUGIN REFLEX.** Cambiarlo es tocar reflex (bump de versión incluido), cosa que el propio grafo asigna a M6 ("scripts consultan engine") y gatea por métrica D. Tal como está escrito, M2 viola el freeze de reflex y duplica el scope de M6. Además el flip del recall cambia el canal de inyección de doctrina en plena ventana — contaminación de libro. **Fix de redacción**: el output de M2 es "engine CAPAZ de servir el recall, demostrado en side-by-side (mismo contenido que el hook actual, arranque en ms)"; el cutover del hook es y se queda en M6. Con eso, post-M2 el sistema queda: recall por basic-memory (como hoy), search del agente por basic-memory MCP (como hoy), engine corriendo en paralelo con eval — estado intermedio genuinamente parable.

**Sobre los dos índices vivos (post-M2 → M5):** el riesgo de divergencia es bajo y está bien contenido por diseño — el markdown es canónico, ambos índices son derivados, y el side-by-side con el eval set ES el detector de divergencia. Lo que falta es acotar la DURACIÓN: dos stacks de embeddings + dos DBs mantenidos "en noches sueltas" puede eternizarse. Añadir a la spec un punto de revisión: si E1 lleva >N semanas en side-by-side sin decisión, se decide (cutover o retirada del engine), no se cohabita indefinidamente.

**Post-M3 pre-M4 (documenta como skill nueva escribiendo vía MCP basic-memory viejo):** coherente, verificado — el matcher de search-before-write sigue matcheando y edit_note append sigue siendo el write-path. Sin objeción.

**Agujero 2 (el que rompe el claim) — M5 desinstala basic-memory ANTES de que M6 migre lo que depende de él.** El grafo es M4→M5, con M6 gated por (métrica D + M2) pero SIN orden respecto a M5. Verificado lo que muere si basic-memory se desinstala con reflex sin migrar:
- `basic-memory-recall.sh` (SessionStart): su launcher es el CLI `basic-memory` → cae al FALLBACK embebido en el script — que además dice "Tu memoria persistente es el MCP basic-memory… /documenta". Recall degradado a texto stale, EN SILENCIO (el hook "nunca bloquea", exit 0 por diseño).
- Los matchers `mcp__basic-memory__*` (search-before-write, retrieval-logger, remind del Stop): dejan de matchear porque las tools ya no existen. Un matcher que no matchea no avisa — los guardrails de escritura y la telemetría de retrieval mueren sin ruido.
- kbx lee el índice de basic-memory (`memory.db`, canary de schema verificado en thick): desinstalado basic-memory, `memory.db` se congela y `kbx budget/stale/doctor` — es decir, consolida — reportan sobre una foto vieja. Staleness silenciosa, no error.

**Fix estructural**: M5 se parte — M5a = MCP propio activo + basic-memory MCP apagado pero CLI/watcher instalados (periodo sin divergencias); la desinstalación física es el ÚLTIMO acto y queda gated por "M6 hecho" (recall, matchers, remind y el repunte de kbx al índice del engine, todos migrados y probados). Con ese gate, el claim "cada milestone deja el sistema funcionando" pasa a ser verdad.

**Veredicto eje 4: el claim es defendible SOLO con las dos correcciones de secuencia (M2-recall→M6; M5-desinstalación gated por M6). Tal como está escrito hoy, es falso en M2 y en M5.**

---

## Eje 5 — Realismo de esfuerzo y coste de oportunidad: "NOCHES SUELTAS" ES HONESTO PARA TODO MENOS M2; Y M0+M3 CAPTURAN LA MAYOR PARTE DEL VALOR DE ESTE TRIMESTRE

**Dimensionamiento honesto de M2** (indexer que honra permalinks + exclusión dotdirs + archive/, FTS5 + vector + fusión calibrada + re-sweep de threshold + harness side-by-side + atribución de misses), en un lenguaje probablemente nuevo (Rust) con pipeline de embeddings ONNX: **8-15 noches efectivas, es decir 3-5 semanas a ritmo real de noches sueltas, o 3-4 campañas de fábrica**. La referencia interna existe: kbx v1 (Go, read-only, sin embeddings, lenguaje ya dominado) costó ~3 sesiones de fábrica; M2 es estrictamente más grande y con más incertidumbre (calibración de fusión y threshold es iterativo por naturaleza). Si M0 concluye que la semántica local NO es load-bearing y el engine nace Go+FTS+grafo, la estimación baja a la mitad. En ningún escenario M2 es "3 noches" — la Sección 5 no da número, pero "avanza en noches sueltas" sin dimensionar invita a subestimarlo.

**Coste de oportunidad (leído del backlog real):** compiten por las mismas noches: (a) **una LAN party es ya** — la submission de lighthouses está sellada (07-16) y quedan tareas de día-del-torneo esta/la próxima semana; (b) **cge P2 (ola ORM Django+Prisma)** está marcado como "el foso vacío, el diferenciador real", con campaña 3 enrutada y pre-campaña lista; (c) OpenWisdom Fase B está gated por el probe repowise (30-60 min). Contra eso, el dolor que el engine resuelve HOY es: retrieval flojo (lo ataca M0, una tarde), sprawl de skills (lo ataca M3, un día + checklist) y latencia de search en caliente (~3.5s × ~4 búsquedas/día — molestia, no sangría).

**Conclusión con opinión:** M0 + M3 (+ M1a como higiene) capturan la mayor parte del valor inmediato del roadmap. M2-M5 son infraestructura estratégica (independencia, AGPL-free, hooks en ms) cuya urgencia la decide M0 — que para eso se diseñó. La secuencia que protege todo: **esta semana M0 + preparación de M3 (skills se escriben con superpowers aún activo, como ya prevé el plan); el 23-25 se mide D; cutover M3 + M1b después; M2 arranca cuando cge P2/universidad lo permitan y con su coste dicho en voz alta.** Si M0 sale "config-fix suficiente", M2 baja de prioridad sin drama — el roadmap ya lo permite, solo hay que resistir la tentación de arrancarlo por inercia.

---

## Eje 6 — Lo que falta: CUATRO HUECOS, DOS CON ACCIÓN REAL

1. **Backup/recuperación del índice: no-problema, pero que la spec lo diga.** El índice es 100% derivado (tesis de thick, verificada); el canon markdown está en git con remote GitHub y commits diarios (verificado). La recuperación es `rebuild` — que el engine lo tenga como comando de primera clase desde v1 (kbx ya tiene el patrón "schema drift → rebuild"). Una línea en la spec: "corrupción de índice = borrar y rebuild; jamás cirugía sobre la DB".
2. **Daemon/cron de reflex-fp: ya no existe.** El cron del 07-10 era one-shot auto-eliminable y ya corrió (crontab solo conserva el comentario; el backlog aún dice "sigue armado" — stale). Ninguna migración lo afecta. Limpieza trivial: borrar el comentario huérfano del crontab y actualizar la línea del backlog.
3. **El hook de recall en el cambio de servidor: es el agujero 2 del eje 4.** Sobrevive M2-M4 sin tocarse (su launcher es el CLI, no el MCP). Muere en silencio si M5 desinstala antes de M6. Cubierto con el gate propuesto. Detalle extra: el texto FALLBACK embebido en el script también hay que reescribirlo en M6 (hoy predica basic-memory + /documenta viejo).
4. **No hay milestone de doctrina, y las referencias por nombre están en el tier core (verificado):** core-index línea 17 ("Memoria persistente = MCP basic-memory…"), doctrina-agentes líneas 17 y 20 ("basic-memory = única fuente de verdad"), más el CLAUDE.md global de Paul (sección "Memoria de sesiones" entera) y el FALLBACK del recall. Post-M5, un agente que arranque leerá doctrina que apunta a un MCP desinstalado. **Añadir a la checklist de M5/M6 un ítem "cutover de doctrina"**: core-index, doctrina-agentes, CLAUDE.md global, FALLBACK del recall, y de paso la description stale de recon-first (ya señalada en thin). Es media hora; olvidarlo produce la clase de deriva doctrina↔realidad que este framework dice existir para matar.

---

## Contradicciones de integración cazadas (Sección 5 vs lo firmado en los 4 informes)

1. **M2 "Recall servido por engine" vs M6 "scripts consultan engine" + freeze de reflex** — la misma pieza en dos milestones, uno de ellos sin gate. (Eje 4, agujero 1.)
2. **M5 "basic-memory desinstalado" sin orden respecto a M6** — rompe recall, matchers y kbx/consolida en silencio; viola "cada milestone deja el sistema funcionando". (Eje 4, agujero 2.)
3. **"documenta/consolida en process" (M3) vs consolida dentro del plugin reflex** — mover consolida es tocar reflex; el grafo dice que reflex no se toca hasta M6. (Eje 2, mina 1.)
4. **El grafo gatea M6 por métrica D pero deja M1 (marketplace de reflex) y M3 (entorno de skills entero) sueltos dentro de la ventana** — contradice el pre-registro firmado ("executor como único cambio") que el propio roadmap dice respetar con "reflex intocable". Intocable-el-plugin no es intocable-el-experimento. (Ejes 1 y 3.)
5. Menor: la checklist de cutover heredada de thin no incluye el dispatch `reflex:executor` de orchestrate — el único ítem cuyo olvido desmonta reflex v2 sin síntoma visible. (Eje 2, mina 2.)

## Top 3 riesgos (en orden)

1. **Ejecutar M3/M1b dentro de la ventana de la métrica D** — invalida un experimento pre-registrado que lleva armándose desde junio y del que depende el cierre del Frente 1 (y lo que quede del candidato a paper). Coste de evitarlo: esperar al 23-25 de julio. Asimetría total.
2. **M5 antes de M6** — recall degradado a doctrina stale + guardrails de escritura muertos + consolida sobre índice congelado, todo sin un solo error visible. Es el único fallo del roadmap cuyo modo de fallo es 100% silencioso.
3. **Arrancar M2 sin dimensionarlo y sin el veredicto de M0** — 8-15 noches que salen del mismo presupuesto que cge P2 (prioridad estratégica declarada) y el pre-torneo de universidad; si M0 demuestra que el config-fix basta, esas noches compran independencia, no capacidad.

## Cambios concretos que haría a la Sección 5

1. **Gate de calendario explícito**: "M1b (marketplace), M3 (cutover) y cualquier cambio de recall NO se ejecutan antes de medir la métrica D (≥2026-07-23). M0 sí, con higiene: replay vía CLI (no MCP), snapshot previo de reflex-retrieval-log.jsonl, sesiones de trabajo `test-*`, y desviación documentada en el doc de pre-registro."
2. **Partir M1** en M1a (monorepo + historia + higiene pre-baseline — puede ir ya) y M1b (re-registro/renombre del marketplace — pegado al cutover M3, post-medición), nombrando la decisión repo-nuevo-vs-rename.
3. **Reescribir el output de M2**: "engine capaz de servir el recall (side-by-side)", cutover del hook explícitamente en M6. Añadir punto de revisión de duración del side-by-side (>N semanas sin decisión = decidir, no cohabitar).
4. **Partir M5**: M5a (MCP propio activo, basic-memory apagado pero instalado) y desinstalación final gated por "M6 completo" (recall + matchers + remind + FALLBACK + repunte de kbx al índice del engine, probados).
5. **Checklist de cutover M3, dos ítems nuevos**: (a) `process:orchestrate` conserva el dispatch `subagent_type: reflex:executor` sin `model` + probe post-cutover contra reflex-log; (b) decisión explícita sobre consolida (se queda en reflex hasta M6, o se mueve el día del cutover si ya es post-medición).
6. **Ítem "cutover de doctrina" en M5/M6**: core-index, doctrina-agentes, CLAUDE.md global, FALLBACK del recall, description de recon-first.
7. **Dimensionar M2 en la spec** (8-15 noches Rust / ~la mitad si Go+FTS) y condicionar su arranque al veredicto de M0 + hueco real entre cge P2 y universidad. Añadir la línea "rebuild como recuperación del índice" y las limpiezas triviales (comentario de cron huérfano, línea stale del backlog sobre el cron del 07-10).

## Qué ratifico

- **La topología del grafo**: M0→M1→M2→M4→M5 con M3 en paralelo, M6 gated, M7 diferible — correcta; todos mis cambios son de calendario y redacción, ninguno reordena la espina dorsal.
- **M0 tal como está definido** (candidatos jina-v2-base-es/MiniLM, gate pre-registrado, re-sweep, atribución de misses, estratificación por observation-hits): integra fielmente las correcciones de los informes engine y thick. Solo le añado la higiene de replay.
- **"M3 no depende del engine"** en lo sustantivo: documenta escribe vía basic-memory hasta M4, consolida consume kbx por path absoluto, overlay probe→defaults probado. Cierto, con las dos minas en checklist.
- **El claim "se puede parar tras cualquier milestone"** — verdadero para M0, M1a, M3 (post-medición), M2 (re-redactado) y M4 (file-first hace el estado intermedio genuinamente seguro, verificado en el diseño del engine); falso solo en M5 tal como está, y el fix es un gate.
- **Las mitigaciones cosidas** (recortes firmados, regla de permalinks, checklist, contador de no-disparos, pin sqlite-vec, candidatos corregidos): todas trazan a decisiones verificadas de los 4 informes previos — la integración es fiel en el contenido; donde falla es en el tiempo.
- **Las 3 decisiones abiertas a propósito** (lenguaje→M0, nombre, archive/ en ranking→post-E1): bien elegidas; ninguna bloquea.

---
Evidencia primaria citable: `agent-develop/docs/superpowers/evals/2026-07-09-reflex-v2-baseline.md` (pre-registro: "único cambio en la ventana"; ventana 07-09→≥07-23) · `plugins/reflex/scripts/reflex-baseline.sh` (FILTER de sesiones test) · `plugins/reflex/hooks/hooks.json` (matchers `mcp__basic-memory__*`, recall en SessionStart, remind en Stop) · `plugins/reflex/scripts/basic-memory-recall.sh` (launcher CLI, FALLBACK embebido stale, cache TTL 1800) · `plugins/reflex/skills/consolida/SKILL.md` (dentro de reflex; kbx por path absoluto; git mv file-first) · `~/.claude/plugins/known_marketplaces.json` (agent-develop GitHub + autoUpdate) · `~/.claude/settings.json` enabledPlugins (reflex/paul-profile/workflow-lint `@agent-develop`) · `~/.claude/reflex-log.jsonl` (656 git-c vivos a 07-16) · crontab (one-shot reflex-fp ya consumido) · `kb-demo/core/core-index.md:17` y `core/doctrina-agentes.md:17,20` (referencias por nombre) · Backlog Frentes 1/2/3/5/9 (métrica D, cge P2, OpenWisdom, universidad sellada 07-16) · remotes kb-demo/kbx en GitHub · `~/.local/bin/kbx` (binario instalado 07-12).
