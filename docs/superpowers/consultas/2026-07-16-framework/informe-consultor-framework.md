# Informe — consultoría adversarial: framework unificado

Consultor independiente. Verificación primaria hecha sobre: los 4 repos (`kb-demo`, `agent-develop`, `kbx`, `workflow-lint`), `~/.claude` (plugins, commands, hooks), config viva de basic-memory (`~/.basic-memory/config.json`), y verificación externa de OKF y del guidance oficial de Agent Skills. No he leído el brainstorm original — trabajo contra el brief.

**Veredicto global: la arquitectura de 3 capas es sana, pero el plan de ejecución está mal secuenciado. El diseño ordena reemplazar el motor de memoria ANTES de haber aislado el diagnóstico que lo justifica — y encontré evidencia primaria de que el diagnóstico probablemente es un fix de config, no una reescritura.**

---

## 1. Veredicto por eje

### Eje 1 — Reemplazo de basic-memory en fase 1: **DÉBIL** (el eje más importante, y el que reabre parcialmente la decisión cerrada 3)

**El hallazgo que lo tumba:** `~/.basic-memory/config.json` tiene `semantic_search_enabled: true` con `semantic_embedding_model: bge-small-en-v1.5`. Ese modelo es **solo-inglés** (BAAI lo documenta así; existe variante multilingüe, y BGE-M3 es el estándar multilingüe actual). La KB de Paul está mayoritariamente en castellano. El síntoma "retrieval poco fiable" es exactamente lo que produce un embedding inglés sobre corpus castellano. **Nadie ha probado el tuning antes de firmar la reescritura.** Cambiar el modelo + re-index es una tarde; si el retrieval mejora, la mitad de la justificación del reemplazo se evapora.

**El síntoma "lento" también está ya diagnosticado en otra dirección:** el propio backlog (Frente 2) tiene pendiente "issue upstream a basic-memory por el coste de imports del CLI" — la latencia dominante es el arranque del CLI Python (~6s en frío, ya mitigado en `basic-memory-recall.sh` con binario directo + cache con TTL + refresh en background). El MCP server residente no paga ese coste por llamada. Ninguno de los dos síntomas apunta inequívocamente a "el producto está roto".

**Qué pierde Paul de verdad** (verificado en la instalación 0.22.1 viva):
- **Watcher + sync en vivo** (`watch-status.json` corriendo ahora mismo, pid activo). El engine propio necesita su propio watcher o re-index bajo demanda.
- **Semantic search**: fastembed es Python/ONNX. En Go eso es cgo con onnxruntime (doloroso) o un servicio aparte o renunciar a semántica y quedarse en FTS. Ninguna opción es gratis.
- **`edit_note` estructurado con semántica de concurrencia**: `/documenta` depende explícitamente de `append` porque "tolera mejor una edición concurrente desde otra sesión" (documenta.md, paso 3). El write-path propio tiene que replicar esa semántica, no solo "escribir ficheros".
- **build_context, multi-proyecto, evolución upstream** (0.22.1, desarrollo activo).

**Qué NO pierde (contra lo que insinúa el brief):** la migración del histórico es casi nula. El markdown ES la fuente de verdad; el índice SQLite (55MB) es derivado y regenerable. Ese riesgo del brief está sobrevalorado.

**El riesgo real del write-path:** kbx hoy son ~2.948 LOC de producción (+4.017 de test) estrictamente read-only, y no por accidente — el propio código lo declara doctrina: *"Strictly read-only: every connection opens with mode=ro (config hard veto — no write path)"* (`kbx/internal/index/db.go:1-2`). Evolucionar a read-write revierte una decisión de seguridad deliberada sobre **el único canon que Paul tiene** (17MB, 136 notas, sin backup fuera de git). Un bug de write en el engine joven corrompe la KB. basic-memory tiene ese path curtido upstream.

**Estimación honesta:** kbx read-write + indexer propio + watcher + search + MCP server es 5-10× el scope del kbx actual. Sí es el 70% del esfuerzo del framework; el valor que compra depende de un diagnóstico que aún no está aislado.

### Eje 2 — Monorepo políglota: **SÓLIDO**, con un recorte

El argumento de cohesión es real y lo verifiqué: el contrato del recall (hook shell en agent-develop + kbx + `core/core-index.md` en kb-demo) cruza HOY 3 repos — un cambio de contrato son 3 PRs sin atomicidad ni test conjunto. El monorepo arregla eso de verdad; Go+shell+skills-md no es "políglota" problemático, es un binario + su pegamento de harness + prompts, cohesionados alrededor de la KB.

**El recorte: workflow-lint no pinta nada ahí.** No toca la KB, no toca el engine, no comparte contrato con nada — es Node autocontenido con su propio ciclo. Absorberlo responde a la fricción "dispersión física" (estética), no a cohesión (ingeniería). Absorber por cohesión, no por propiedad: workflow-lint se queda como plugin independiente del marketplace. Si en 6 meses molesta, se mete — mover un plugin es barato; el criterio no.

### Eje 3 — Absorber superpowers: **mitad y mitad — absorción selectiva DÉBIL-aceptable; formato "router de 20 líneas" ROTO tal como está descrito**

**Sobre el mecanismo (verificado contra el guidance oficial de Anthropic):** el patrón canónico de progressive disclosure es thin SKILL.md + **reference files empaquetados en el directorio de la skill**, cargados bajo demanda. No es "SKILL.md que apunta a un almacén de memoria externo vivo". La diferencia importa por tres costes reales del router-a-KB:
1. Tool call + latencia extra en cada activación.
2. **Riesgo de no-lectura**: el modelo puede seguir sin leer la nota. Este riesgo no es teórico — el propio `/documenta` de Paul lleva un fallback explícito para "si kbx no está o falla". Una skill de proceso cuyo contenido depende de que un motor esté vivo en runtime es más frágil que una skill autocontenida.
3. **Conflación de lifecycles**: la doctrina de proceso (cómo hacer TDD, cómo orquestar) se versiona con el framework; la memoria personal se versiona con la KB. Si el contenido de las skills absorbidas vive en notas de KB, el framework queda cáscara y la instancia carga la doctrina — **rompe la separación framework/instancia que el propio diseño predica** (un usuario N recibiría skills vacías).

**El patrón correcto ya existe en casa:** `orchestrate-personal/SKILL.md` duplica el contenido operativo (cost pyramid, división del trabajo) y lleva puntero de precedencia: *"Fuente canónica: [[doctrina-agentes]]; si divergen, manda la nota; actualiza la skill"*. Duplicación controlada con regla de precedencia > indirección en runtime. Reservar el router-a-KB solo para doctrina genuinamente personal y viva.

**Sobre el NIH:** las skills son prompts; forkear 5-6 es barato y Paul ya tiene opiniones formadas — el coste de construcción no me preocupa. Me preocupan dos cosas: (a) congelar = perder la evolución de superpowers 6.1.1 (instalado, activo); (b) **incoherencia con su propia doctrina de empresa**: `COMPANY-INTEGRATION-SPEC.md` manda literalmente *"do not reinvent what exists (reuse-first)… Building a new orchestration engine would be duplication"* y ancla el plugin de empresa a `superpowers:subagent-driven-development` como engine. Paul va a vivir con superpowers en el trabajo y con sus forks en casa: dos versiones de la misma disciplina divergiendo. Asumible, pero que lo firme sabiéndolo.

### Eje 4 — OKF: **SÓLIDO** tal como está enmarcado

Verificado: OKF v0.1 es real (Google Cloud, publicado 2026-06-12, repo `GoogleCloudPlatform/knowledge-catalog`), spec de una página, **exige exactamente un campo (`type`)**, todo lo demás libre; markdown + YAML frontmatter + links normales. La KB de Paul ya cumple ~90% por construcción. Coste de alineación ≈ añadir `type` donde falte. Es v0.1 con un mes de vida — señal débil — pero con coste casi cero es una apuesta asimétrica correcta **exactamente porque el diseño la enmarca como convención, no dependencia**. Único aviso: los campos propios (`tier`, `kbx_budget_max`, `kbx_orphan_ok`) se mantienen como extensiones sin culpa; si OKF v0.2 gira, no se persigue.

### Eje 5 — Genericidad / profile/: **aceptable como higiene, ROTO si se vuelve maquinaria**

"Nada personal en el repo del framework" es disciplina de escritura, cuesta cero, y de hecho ya la practica (reflex/paul-profile separados). Eso sí. Pero `profile/` con "hooks activos, paths, cost pyramid" configurable huele a config schema + parser + skills templetizadas — maquinaria cuyo único ejercitador en 12 meses sería un usuario hipotético. Con la doctrina anti-over-engineering de Paul: **límite duro = un fichero plano de paths/toggles + env vars. Cero abstracción que solo el segundo usuario ejercitaría.** Test simple: si un if o un campo del schema existe solo para "alguien que no es Paul", fuera.

### Eje 6 — Lo que el diseño no menciona (encontrado en verificación)

1. **Colisión con la ventana de medición de reflex v2.** La métrica D (reincidencia vs baseline **pre-registrado**) abre el 2026-07-23 — dentro de 7 días — sobre el deployment actual de reflex (backlog, Frente 1). Mover reflex al monorepo o cambiar el recall a engine propio durante la ventana **invalida la comparación** (y de paso el candidato a paper estrecho que depende de esos reflejos). "Mover plugins es mecánico después" es cierto en código y falso en metodología. O se mide primero, o se mata la medición — pero como decisión explícita de Paul, no como daño colateral.
2. **Coste de oportunidad no presupuestado.** El backlog tiene 10 frentes; cge P2 (ola ORM Django+Prisma) está marcado como "el foso vacío, el diferenciador real" y compite por las mismas noches de fábrica. El motor de memoria es infraestructura interna sin tesis de producto; cge la tiene. El diseño no dice cuántas campañas de fábrica come el engine ni qué se congela mientras.
3. **Higiene del write-path antes de producción**: el engine escribe sobre el único canon desde el día 1. Mínimo: shadow-mode contra una copia de la KB + diff vs basic-memory durante un periodo, antes de darle la pluma de verdad.
4. **Lifecycle del MCP server propio**: punto a favor honesto — stdio por sesión en Go arranca en milisegundos y mata el problema de latencia mejor que cualquier tuning de Python, y existe SDK oficial de Go para MCP. El coste de mantenimiento del protocolo es real pero acotado. Este es el argumento más fuerte A FAVOR del engine propio, y el diseño ni lo explota.
5. **Migración de matchers y doctrina**: los hooks matchean `mcp__basic-memory__write_note` por nombre de tool, y [[doctrina-agentes]]/core-index/documenta dicen "basic-memory = única fuente de verdad" en texto. Mecánico, pero fácil de olvidar y silencioso al fallar (un matcher que no matchea no avisa).

---

## 2. Los 3 riesgos más graves, en orden

1. **Reescribir el motor sin haber aislado el diagnóstico.** El embedding solo-inglés sobre KB castellana es un candidato a causa-raíz del "retrieval flojo" que se prueba en una tarde; la latencia ya está diagnosticada como coste de imports del CLI y mitigada con cache. Si el tuning funciona, la fase 1 del plan (el 70% del esfuerzo) queda construida sobre un diagnóstico equivocado. Y el write-path propio joven sobre el único canon añade riesgo de corrupción donde hoy hay código curtido.
2. **Coste de oportunidad + contaminación de la métrica D.** El engine come las noches de fábrica que cge P2 (prioridad estratégica declarada) necesita, y tocar reflex antes de cerrar la medición pre-registrada del 07-23 invalida un experimento que lleva semanas armándose.
3. **Router-skills de 20 líneas → doctrina de proceso dependiente de un motor vivo y fuera del framework.** Pierde la disciplina in-context probada de superpowers, añade un punto de fallo en runtime, y contradice tanto el guidance oficial de skills (reference files empaquetados) como la separación framework/instancia del propio diseño.

---

## 3. Cambios concretos que haría

1. **Fase 0 nueva, antes de todo (una tarde):** cambiar `semantic_embedding_model` a la variante multilingüe (o BGE-M3), re-index, y medir el retrieval sobre 10-15 queries reales que hoy fallan. Con ese dato, ratificar o degradar el "engine-first". Es la aplicación directa de la doctrina de Paul (recon-first, validar contra evidencia antes de construir).
2. **Estrangulamiento en vez de big-bang:** el write-path de kbx nace acotado a los dos consumidores reales (`/documenta` y `/consolida`), con la semántica append-first que documenta ya exige; basic-memory sigue de watcher + MCP de lectura mientras tanto. El MCP server propio es la ÚLTIMA milestone (cuando el engine ya demostró no romper nada), no la primera. Shadow-mode contra copia de la KB antes de darle la pluma.
3. **Formato de las skills absorbidas:** contenido operativo como reference files dentro del directorio de cada skill (progressive disclosure canónico); puntero de precedencia a la nota de KB solo para doctrina personal viva (patrón orchestrate-personal actual). Nada de routers puros cuya carne viva en la KB.
4. **workflow-lint fuera del monorepo v1.** Criterio: se absorbe por contrato compartido, no por orden en el escritorio.
5. **profile/ minimalista:** un fichero plano + paths. Sin schema, sin templating, sin capa de configuración de cost pyramid — eso vive en las skills/doctrina como hoy.
6. **Secuenciar contra la métrica D:** reflex no se mueve al monorepo hasta cerrar (o matar explícitamente) la medición del 07-23.

## 4. Qué ratificaría tal cual

- **Personal-primero, genericidad como propiedad** (decisión 1) — correcta, y consistente con su historial (kbx, cge nacieron así).
- **Las 4 fricciones** (decisión 2) — reales; la del contrato cross-repo la verifiqué yo mismo.
- **La arquitectura de 3 capas con dependencia thin→thick** — sana; es la formalización de lo que ya funciona (recall hook → kbx → KB).
- **OKF como convención de portabilidad** — coste ~cero, enmarcado correcto.
- **Filosofía thin prompts / thick artifacts** — bien respaldada (context-rot, progressive disclosure); mi objeción del eje 3 es a UNA materialización (router-a-KB), no a la filosofía.
- **Jubilar superpowers como dependencia instalada** — aceptable con el formato del cambio 3 y sabiendo el coste de divergencia con el contexto de empresa.
- **kbx como semilla del engine** — el código que leí es sólido, testeado y con doctrina explícita en los comentarios; es la base correcta. Mi objeción es de secuencia y de scope del write-path, no de vehículo.

---

### Fuentes externas verificadas
- OKF: [Google Cloud Blog](https://cloud.google.com/blog/products/data-analytics/how-the-open-knowledge-format-can-improve-data-sharing/) · [SPEC.md en GitHub](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md) · [MarkTechPost 2026-06-16](https://www.marktechpost.com/2026/06/16/google-cloud-introduces-open-knowledge-format-okf-a-vendor-neutral-markdown-spec-for-giving-ai-agents-curated-context/)
- Agent Skills: [Equipping agents for the real world (Anthropic)](https://www.anthropic.com/engineering/equipping-agents-for-the-real-world-with-agent-skills) · [Skill authoring best practices](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices)
- Embeddings: [BAAI/bge-small-en-v1.5 (English)](https://huggingface.co/BAAI/bge-small-en-v1.5) · [Semantic Search — Basic Memory docs](https://docs.basicmemory.com/concepts/semantic-search)

### Evidencia primaria local (paths citables)
- `~/.basic-memory/config.json` — `semantic_embedding_model: bge-small-en-v1.5` sobre KB en castellano.
- `kbx/internal/index/db.go:1-2` — read-only como "config hard veto" deliberado.
- `~/.claude/commands/documenta.md` — dependencia de `edit_note` append + fallback "si kbx falla" ya existente.
- `agent-develop/plugins/paul-profile/skills/orchestrate-personal/SKILL.md:8-13` — patrón duplicación+precedencia vivo.
- `agent-develop/COMPANY-INTEGRATION-SPEC.md` — mandato reuse-first sobre superpowers en el contexto empresa.
- `kb-demo/Backlog — frentes abiertos.md` Frente 1 — ventana métrica D abre 2026-07-23 con baseline pre-registrado; Frente 2 — issue de latencia de imports del CLI ya diagnosticado.
