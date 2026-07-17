# Framework unificado de trabajo agéntico — design spec

- **Fecha**: 2026-07-16
- **Estado**: borrador — pendiente de review de Paul
- **Proceso**: brainstorm + 5 consultorías adversariales Fable con verificación primaria (informes en `docs/superpowers/consultas/2026-07-16-framework/`). Toda decisión de abajo está ratificada por convergencia orquestador+consultor o firmada por Paul explícitamente.
- **Ejecución prevista**: campañas fábrica (skill `paul-profile:fabrica`) sobre el monorepo — ver §8.

## 1. Objetivo y alcance

Consolidar el proto-sistema personal (KB kb-demo + basic-memory, plugins reflex/paul-profile, CLI kbx, workflow-lint, skills superpowers, /documenta) en **un framework coherente de trabajo agéntico con memoria persistente**, atacando las 4 fricciones diagnosticadas: dispersión física, duplicación de lógica, falta de modelo mental unificado y mantenimiento costoso.

- **Personal-primero**: el usuario del framework es Paul. La genericidad es una propiedad de diseño (separación framework/instancia desde el día 1), no un producto: no se construye nada cuyo único consumidor sería un tercero hipotético (excepción acotada: `templates/`, §6.6).
- **Se jubilan**: basic-memory (MCP de memoria; retrieval poco fiable + latencia CLI 3.0-3.9s/llamada fría, medida) y superpowers (absorción selectiva MIT, §5.2). Ambas por **estrangulamiento**, nunca big-bang.
- **Se absorben al monorepo**: agent-develop (que ES el marketplace), kbx (tal cual, sin migrar), reflex (post-métrica-D), paul-profile (menos fabrica), /documenta.
- **Quedan fuera**: workflow-lint (repo propio, referenciado por el marketplace como hoy), fabrica (plugin de instancia: workflow personal + hook propio), la KB kb-demo (repo de datos propio).

## 2. Decisiones raíz (firmadas por Paul)

1. Consolidar lo personal primero; genérico de base, no como producto.
2. Reemplazar basic-memory por motor propio — **gated por Fase 0** (§4.1): el diagnóstico verificado apunta a config (embedding solo-inglés `bge-small-en-v1.5` sobre KB en castellano); la Fase 0 decide urgencia y lenguaje.
3. superpowers se jubila por absorción selectiva (licencia MIT verificada, © 2025 Jesse Vincent; atribución en el repo desde el día 1).
4. Filosofía: **thin prompts / thin skills / thick artifacts+context** (respaldo: literatura context-rot; guidance Anthropic de Agent Skills/progressive disclosure). Sin números mágicos: la heurística operativa es "¿tiene que estar SIEMPRE en contexto o solo cuando dispara?".
5. Lenguaje del engine: decisión abierta hasta el veredicto de Fase 0 (§4.5). kbx (Go, ~2.9k líneas prod + 4k test) **no se migra** en ningún escenario.
6. La ventana pre-registrada de la métrica D de reflex (2026-07-09 → medir ≥2026-07-23) se respeta **como experimento, no solo como plugin**: nada que altere el entorno del agente (marketplace, skills, recall) se ejecuta dentro de la ventana (§7, gate de calendario).

## 3. Arquitectura: tres capas, dependencia en un solo sentido

```
THIN   superficie de agente: skills-router (~30-50 líneas + reference files),
       hooks (guardrails), comandos. Leen doctrina / invocan engine.
  ↓
ENGINE motor de memoria y contratos: index, search, recall, write, budget,
       doctor, stale. Expuesto como CLI (hooks/humanos) y MCP (agente, E3).
  ↓
THICK  la KB: markdown + YAML frontmatter (≈OKF), doctrina canónica, perfiles,
       destilados, bitácoras. Única fuente de verdad.
```

**Reglas estructurales** (matan la duplicación de raíz):
- Ningún procedimiento vive en una skill: la skill es router + reference files; el porqué y los deltas personales viven en la KB.
- Ningún cálculo vive en shell/skill si el engine lo expone (budget, search, doctor).
- El contrato entre capas es el **envelope JSON versionado** de kbx, promovido a contrato del sistema. Es agnóstico al lenguaje → el lenguaje es decisión por-binario, no por-sistema.
- **Framework sin nada personal.** Instancia de Paul = kb-demo (repo aparte) + `profile.md` plano (sin schema) + plugin fabrica.

## 4. Engine

### 4.1 Fase 0 — diagnóstico de retrieval (M0; una tarde; SIN código de engine)

1. **Eval set**: las 46 queries únicas reales de `~/.claude/reflex-retrieval-log.jsonl` + 5-10 casos duros de memoria de Paul ("busqué X y no salió Y"). Etiquetar TODAS (ground truth a nivel de nota). **Estratificación obligatoria**: replay previo contra basic-memory actual marcando qué queries devuelven hits de fila observation en el top-k; ese subgrupo debe estar representado (sin él, el gate de degradación de la gramática §6.3 sería ciego: las observations son ~65% del corpus FTS).
2. **Higiene de replay** (protege el pre-registro de la métrica D y el retrieval-log): snapshot de `reflex-retrieval-log.jsonl` antes; replay **vía CLI** (`basic-memory tool search-notes`), nunca vía tools MCP (el hook retrieval-logger appendearía queries sintéticas al log del que salió el eval set); sesiones de trabajo nombradas `test-*` (el FILTER de reflex-baseline.sh las excluye); desviación documentada en el doc de pre-registro (`docs/superpowers/evals/2026-07-09-reflex-v2-baseline.md`).
3. **Candidatos** (verificados contra fastembed 0.8.0 instalado — NO usar familia E5 ni bge-m3 aquí: E5 exige prefijos query:/passage: que ni basic-memory ni fastembed ponen; bge-m3 no está soportado):
   - Primario: `jinaai/jina-embeddings-v2-base-es` (bilingüe es-en, retrieval, sin prefijos; requiere `semantic_embedding_dimensions: 768` — el provider hardcodea 384 y sin ese campo revienta con RuntimeError).
   - Barato: `sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2` (384 dims, drop-in; modelo de paráfrasis, más débil en retrieval asimétrico).
   - Techo opcional: provider `litellm` + modelo API multilingüe con input_type asimétrico (trade-off: la KB sale de la máquina).
   - Nombre HF completo en config (el alias corto solo existe para bge-small-en-v1.5). El cambio de dims recrea la tabla vectorial y el re-embed es automático (117 entidades / 5.154 chunks / ~1.7 MB: minutos).
4. **Gate numérico pre-registrado antes de mirar** (comparación pareada, no test de proporciones): p.ej. "≥5 queries arregladas y ≤1 rota". **Re-sweep del threshold** por modelo (el 0.55 no sobrevive al cambio). **Atribución de cada miss** (FTS-miss / vector-miss / threshold-miss, vía search_type explícito) — sin atribución el gate no distingue "modelo malo" de "fusión mal calibrada".
5. **Outputs**: (a) urgencia del engine RW; (b) **decisión de lenguaje** (§4.5); (c) el eval set etiquetado queda como test de regresión permanente del engine (side-by-side de E1 y sucesivos).

### 4.2 Componentes

**Se quedan tal cual (kbx, Go)**: index reader, doctor, budget, stale, targets, envelope JSON, gitx. Comandos se portan al binario nuevo oportunistamente o nunca.

**Nuevos (binario nuevo, lenguaje según §4.5)**:
- **Indexer propio**: markdown+frontmatter → FTS5 + vectores (sqlite-vec, **versión pineada** — pre-v1 con breaking changes anunciados) + grafo de links. Incremental por mtime/git al invocar, sin daemon salvo que duela. Reglas duras en §6.
- **Search/recall unificados**: una sola implementación de retrieval (FTS5 + semántica + fusión + grafo) para el recall de SessionStart y las búsquedas del agente. Fusión: copiar el **diseño** de basic-memory (fórmula max(v,f)+bonus·min(v,f), clave (type,id), gate FTS, normalización BM25, threshold configurable) — **jamás el código: basic-memory es AGPL-3.0** y copiarlo haría el framework derivado AGPL (veto también a vendorizar).
- **Write-path**: write frontmatter-aware con search-before-write **nativo** (busca duplicados, fuerza decisión merge/append/new). v1: new + append + replace_section. **SIN move** (un move sin actualización de links corrompe el grafo: no se ofrece hasta hacerlo con links; hasta entonces move = basic-memory o manual+doctor). Veto RO por defecto; write habilitado por comando explícito. La validación OKF **auto-completa y nunca rechaza** (§6.4).
- **MCP server** (stdio; E3, última milestone): hot-path real medido en 11 días — `read_note` (265), `search_notes` (48), `recent_activity` (4). **Sin build_context** (1 uso; si hace falta, composición search+read en capa thin). SDKs oficiales maduros en Go y Rust.

### 4.3 Recortes v1 (decisiones explícitas, no omisiones)

Sin gramática observations/relations en el índice (§6.3) · sin move · sin build_context · sin cloud/sync bidireccional (no se usa: mode local verificado) · sin daemon · `rebuild` como comando de primera clase ("corrupción de índice = borrar y rebuild; jamás cirugía sobre la DB").

### 4.4 Estrangulamiento

- **E1 (read)**: indexer + search propios corriendo en paralelo; side-by-side contra basic-memory con el eval set (mismo corpus: exclusión de dotdirs replicada, archive/ indexado). Output: "engine CAPAZ de servir el recall, demostrado" — **el cutover del hook de recall es de M6, no de E1** (el recall es un script del plugin reflex). **Punto de revisión de duración**: side-by-side >4 semanas sin decisión ⇒ se decide (cutover o retirada), no se cohabita.
- **E2 (write acotado)**: file-first — el engine escribe el markdown directamente (KB en git = rollback), el watch de basic-memory lo absorbe solo, y doctor verifica que el índice absorbió lo esperado. Solo /documenta y /consolida. Sin traductor engine→MCP (se tiraría en E3).
- **E3 (MCP + jubilación)**: M5a = MCP propio activo, basic-memory **apagado pero instalado** (periodo sin divergencias). La **desinstalación física es el último acto, gated por "M6 completo y probado"** — sin ese gate mueren en silencio: el recall (fallback stale embebido), los matchers `mcp__basic-memory__*` de guardrails (dejan de matchear sin avisar) y kbx/consolida (índice congelado).

### 4.5 Lenguaje del engine — decisión post-Fase 0 (criterio pre-acordado)

- **Si la semántica local es load-bearing** (Fase 0 lo dice): **Rust** para el binario nuevo — fastembed-rs trae multilingual-e5-* y bge-m3 nativos con prefijos bajo nuestro control, ort estático (sin drama de shared libs), rusqlite+fts5 bundled, crate sqlite-vec, rmcp oficial, arranque en ms (los hooks pagan startup en cada tool call). Riesgo aceptado: fastembed-rs es proyecto de un maintainer (órbita Qdrant).
- **Si FTS+grafo bastan**: **Go puro** — gana por reuso del ecosistema kbx; la semántica queda como provider pluggable (API o proceso aparte).
- Descartados: Go+ONNX (tres costuras nativas simultáneas), sidecar Python (la dependencia que venimos a jubilar), fork/vendor de basic-memory (AGPL + 0.3-3s de arranque), TypeScript (embeddings débiles, no gana en ningún eje).
- Duplicación aceptada y vigilada: parser de frontmatter en Go (kbx) y en el binario nuevo.

## 5. Capa thin

### 5.1 Formato de skill

Frontmatter de disparo + body ~30-50 líneas + **reference files empaquetados en el directorio de la skill** (progressive disclosure; la carne va ahí, NO en la KB — si la doctrina genérica viviera en la KB, el framework quedaría vacío y las skills dependerían del engine en runtime). **Overlay personal con patrón único de degradación**: probe del CLI → si no hay engine/KB, defaults + aviso visible (patrón ya probado en /documenta). "Vía engine" = kbx/basic-memory/filesystem hasta que E1 exista.

### 5.2 Plugin `process` — absorción de superpowers (destilar, no copiar)

| Skill | Absorbe de superpowers (MIT) | Absorbe propio |
|---|---|---|
| brainstorm | brainstorming | — |
| plan | writing-plans | — |
| orchestrate | subagent-driven-development, executing-plans, dispatching-parallel-agents | orchestrate-personal (cost pyramid, memory packet, blindspot pass) **+ reviewer-dispatch escalado al riesgo del diff** |
| tdd | test-driven-development | — |
| debug | systematic-debugging | recon-first (description con dos puertas: bug + stuck/pre-grind; el hook stuck-loop está muerto — commit 590d6ca — la description actual de recon-first está stale) |
| verify | verification-before-completion | gate de validación de orchestrate-personal. **Solo auto-verificación barata pre-commit**; el reviewer-dispatch (caro, pre-merge) vive en orchestrate — mezclarlos = spam de reviews o dilución del gate |

- Absorber = extraer el movimiento esencial y reescribirlo thin; se tira la prosa, los gritos y los gates dogmáticos. Atribución MIT en el repo día 1.
- No se migra lo no usado (finishing-a-development-branch, writing-skills, using-git-worktrees, …): 0 referencias vivas en la KB (verificado). Se añade si duele.
- `using-superpowers` (SessionStart) desaparece. Su función real es compliance anti-racionalización, no descubrimiento. Sustituto mínimo: **línea de routing en core-index** + **contador de "skill que debió disparar y no disparó"** durante la ventana de rollback, como criterio pre-registrado de desinstalación.
- `/documenta` entra a process. **`/consolida` se queda en el plugin reflex hasta M6** (vive dentro de reflex; moverla = tocar reflex = violar el freeze). Consume kbx por path absoluto (verificado): sobrevive M1/M3 sin cambios.

### 5.3 Cutover de superpowers (checklist, no evento)

Pre-condición: **post-medición de métrica D** (≥2026-07-23). Pasos:
1. Skills de process escritas y revisadas con superpowers aún activo (sin instalar process).
2. Mismo día: `superpowers` disabled + `process` enabled; **actualizar fabrica** (referencia `superpowers:subagent-driven-development` y `paul-profile:orchestrate-personal` — ambas mueren); **`process:orchestrate` conserva el dispatch `subagent_type: reflex:executor` sin `model`** (paridad paul-profile 0.3.0 — si se pierde, reflex v2 se desenchufa sin síntoma) + **probe post-cutover** verificando `agent_type` en reflex-log; decidir entrada de workflow-lint en el marketplace nuevo; atribución MIT.
3. superpowers instalado-pero-apagado ≥ un ciclo real de trabajo (rollback de un flag) + contador de no-disparos activo.
4. Desinstalar cuando el ciclo cierre sin carencias.

### 5.4 Plugin `guardrails` (reflex)

Migra al monorepo en M6 (gated: métrica D cerrada + M2). Cambio interno único: scripts dejan de calcular y consultan el engine (`engine search --json`, etc.). La infra de medición de FP se queda tal cual y el framework la hereda como patrón: *hooks warn-only con medición de falsos positivos incorporada*.

## 6. Capa thick (KB)

### 6.1 Principio

**La KB no se migra.** kb-demo queda como repo git propio, ficheros intactos. Lo único que muere con basic-memory es su índice SQLite — derivado, se regenera. Migración de thick = re-index.

### 6.2 Reglas duras del indexer (texto de spec, no detalle de implementación)

- **El indexer honra el `permalink:` del frontmatter y JAMÁS lo regenera** (verificado: `ensure_frontmatter_on_sync` los persiste, 112/112 notas). Genera solo para notas nuevas. El read-path acepta identifier con/sin prefijo de proyecto y resolución por título (read_note = 83% del tráfico real).
- **Recencia = git, no mtime ni created_at del índice** (ninguna nota tiene `created:`; un clone fresco resetea mtimes). kbx targets ya lo hace así.
- **Exclusión de dotdirs replicada** (.claude/, .omc/, .superpowers/: 24 .md fuera del índice actual) — sin esto el side-by-side de E1 compara corpus distintos.
- **archive/ SE INDEXA** (como hoy; 32% del índice): cambiar corpus y motor a la vez impediría atribuir deltas en E1. Si molesta en ranking, se recorta post-E1 con datos.
- Entidades no-markdown (5, permalink NULL): no se indexan en v1.
- Links a notas inexistentes se toleran (to_id NULL), jamás error de indexado.

### 6.3 Gramática observations/relations → estilo opcional (también en escritura)

Justificación con datos: 0 consumidores de la gramática (ni una query por categoría en hooks/skills/kbx), taxonomía rota en la práctica (decisión 140 + decision 136; aprendizaje 238 + learning 9), 84% de relations genéricas, build_context 1 uso/11 días, y doctor/stale usan solo EXISTENCIA de aristas.
- **Wikilinks `[[...]]` = contrato load-bearing** (grafo, orphan, stale, degree): se mantienen e indexan.
- Observations pasan a bullets normales (texto indexable, sin fila propia). /documenta deja de generarlas como estructura. El histórico (1.417 obs / 540 rel) **no se reescribe** — queda buscable como contenido.
- **Gate real**: el eval set estratificado por observation-hits (§4.1.1) vigila la degradación en E1. Si retrieval real las echa de menos, se reabre con datos.
- **Coupling obligatorio**: el cutover de índice actualiza la lista `consumed` del schema-canary de kbx **en el mismo commit** (consolida falla-fuerte ante schema_drift por diseño; sin esto, el primer /consolida post-cutover muere).

### 6.4 OKF (Open Knowledge Format, Google Cloud, v0.1)

Alineación como convención, no dependencia: `type` obligatorio (ya se cumple en 101/112; **backfill one-shot de las 11 notas sin type ANTES de encender el check**), extensiones propias (`tier`, `permalink`, `kbx_*`) permitidas por la spec. **La validación vive en `doctor` como finding (offline); el write-path auto-completa y nunca rechaza** — /documenta no puede fallar al cierre de sesión. Claim de conformance acotado al árbol de notas (dotdirs documentados fuera del bundle). El check nuevo de doctor "un core jamás recibe appends fechados" entra aquí (regla en un solo dueño).

### 6.5 Higiene pre-baseline (M1a)

Backfill de `type:` (11 notas) · limpieza de root files (developercv.cls, fontawesome.pdf — doctor ya los flaggea) · decisión dotdirs escrita (se quedan, documentados fuera del bundle) · borrar comentario huérfano del cron reflex-fp + actualizar línea stale del backlog.

### 6.6 templates/ (M7, diferible)

KB esqueleto + profile.md comentado + README. **Método clean-room (whitelist): se escribe desde cero mirando la instancia solo como referencia de forma.** Nunca "destilar quitando lo personal" (blacklist: un miss = finanzas/timeline de Paul en un repo para terceros). Sin consumidor hasta que exista un tercero; no bloquea nada.

## 7. Roadmap

```
M0 Fase 0 ──→ M1a repo ──→ M2 E1-read ──→ M4 E2-write ──→ M5a MCP ──→ M5b desinstalar
                 │                                            ▲ gated por M6 completo
                 └──→ M3 cutover skills + M1b marketplace     │
                        (gated: métrica D medida ≥07-23)      │
                      M6 guardrails ←── (métrica D + M2) ─────┘
                      M7 templates (diferible)
```

**Gate de calendario (pre-registro de métrica D)**: M1b (re-registro/renombre del marketplace que sirve reflex), M3 (cutover) y cualquier cambio de recall **NO se ejecutan antes de medir la métrica D** (≥2026-07-23). M0 sí puede ir ya, con la higiene de §4.1.2. Antes de calcular D: fix del `jq 2>/dev/null` de reflex-baseline.sh (análisis read-only, no viola el freeze).

- **M0 — Fase 0** (una tarde, esta semana): §4.1. Outputs: urgencia + lenguaje + eval set permanente.
- **M1a — Monorepo** (puede ir ya): crear repo (nombre pendiente), absorber historia de agent-develop, estructura engine/plugins/templates, kbx tal cual, higiene pre-baseline (§6.5). No toca nada instalado.
- **M1b — Marketplace** (pegado a M3, post-medición): decisión repo-nuevo-vs-rename explícita (rename: redirects de GitHub mantienen fetch, identidad `@agent-develop` no cambia — preferido; repo nuevo: re-registrar + flip de enabledPlugins, riesgo de reflex servido stale).
- **M2 — E1 read**: §4.2 + §4.4. **Dimensionado honesto: 8-15 noches efectivas en Rust (~la mitad si Go+FTS)** — kbx v1 (más pequeño, lenguaje dominado) costó ~3 sesiones de fábrica. **Arranque condicionado a**: veredicto de M0 + hueco real entre universidad y cge P2. Si M0 sale "config-fix suficiente", M2 baja de prioridad sin drama.
- **M3 — Cutover skills** (post-medición): §5.3.
- **M4 — E2 write**: §4.4. Canary de kbx en el mismo commit.
- **M5a — MCP propio** / **M5b — desinstalación** (gated por M6): §4.4-E3.
- **M6 — guardrails**: §5.4 + cutover del hook de recall + reescritura del FALLBACK embebido + repunte de kbx al índice del engine + **cutover de doctrina**: core-index, doctrina-agentes, CLAUDE.md global de Paul y description de recon-first referencian basic-memory/superpowers por nombre — actualizarlos o quedará doctrina apuntando a un MCP muerto.
- **M7 — templates**: §6.6.

**Propiedad protegida**: cada milestone deja el sistema funcionando (verificada por consultoría para M0, M1a, M3, M2 re-redactado y M4; en M5 la garantiza el gate M5b←M6).

## 8. Ejecución con fábrica

El desarrollo se ejecuta con campañas del skill `paul-profile:fabrica` sobre el monorepo:

- **Prerequisito de M1a**: `.superpowers/fabrica/config.md` del monorepo (fuente del roadmap = esta spec §7; clases pre-autorizadas; reserva fable; protocolo de gates como en lighthouses/kbx).
- **Routing de lanes**: el eval set de M0 es el oráculo que permite rutear a lane mecánica gran parte de M2 (side-by-side medible por comando). Piezas de lane diseño con spec-first + gold: el indexer (paridad de permalinks/corpus como gold set), la fusión/calibración de search (eval set como gold), el write-path (corpus de casos search-before-write). Las skills de process (M3) son lane diseño con review adversarial (su "gold" es checklist de paridad de movimientos vs la skill superpowers absorbida).
- **Kill-criteria pre-registrados por pieza** en su spec (patrón ya practicado). Los gates numéricos de esta spec (Fase 0 §4.1.4, contador de no-disparos §5.2, punto de revisión de side-by-side §4.4) son kill-criteria de primera clase.
- **Bootstrap**: fabrica corre sobre el stack actual (superpowers + orchestrate-personal + reflex:executor) hasta el cutover M3; la actualización de fabrica es ítem de la checklist §5.3. Las campañas de M0-M2 usan el stack viejo; las de M4+ ya corren sobre process.
- **Régimen de gates (firmado por Paul 2026-07-16): consultor Fable como adjudicador delegado.** Todo review/gate que el protocolo derivaría a Paul (incluidos `GATE:` de merge y superficies irreversibles internas: envelope, schema del índice, formato de skill) lo adjudica un **consultor Fable independiente** — Paul pre-aprueba sus decisiones. Condiciones que hacen legítima la delegación (sin ellas, el verdict es inválido y escala a Paul): (a) consultor fresco por gate, sin haber participado en la pieza que juzga; (b) verificación primaria propia obligatoria; (c) mandato explícito de disenso (convergencia complaciente = fallo); (d) verdict-artifact con cita escrito al repo (audit trail). Precedente: régimen de auditor de la campaña lighthouses. **Línea roja que NO se delega**: acciones destructivas o externas al sistema (borrado de la KB o de repos, publicación fuera, cambios de permisos) — esas siempre a Paul.

## 9. Riesgos top (consolidados de las 5 consultorías)

| # | Riesgo | Mitigación cosida |
|---|---|---|
| 1 | Ejecutar M3/M1b dentro de la ventana de métrica D → invalida experimento pre-registrado | Gate de calendario §7; M0 con higiene de replay §4.1.2 |
| 2 | M5 antes de M6 → recall/guardrails/consolida mueren en silencio | M5b gated por M6 completo y probado |
| 3 | Permalinks regenerados en re-index → rompe 83% del tráfico + memory packets | Regla dura §6.2, gold set de paridad en fábrica |
| 4 | Scope-creep hacia "reimplementar basic-memory" | Recortes v1 como decisiones firmadas §4.3; kbx no se migra |
| 5 | Perder el dispatch reflex:executor en el cutover → reflex v2 desenchufado sin síntoma | Ítem explícito + probe post-cutover §5.3 |
| 6 | M2 subestimado (8-15 noches) canibaliza cge P2/universidad | Dimensionado en spec; arranque condicionado a M0 + calendario §7 |
| 7 | Quitar el forcing de superpowers degrada disparo de skills | Contador de no-disparos pre-registrado §5.2 |
| 8 | sqlite-vec pre-v1 con breaking changes | Versión pineada §4.2 |

## 10. Decisiones abiertas (a firmar con datos, no hoy)

1. ~~Lenguaje del engine~~ — **FIRMADO 2026-07-17 por verdict M0 (`evals/retrieval-fase0/verdict/m0-verdict.md`): RUST** — semántica load-bearing (criterio 2 real: 26/55 queries solo recuperables por vía semántica; FTS puro 18/55). Corolario del mismo verdict: jina-es GANA el gate (7/0), config aplicada (jina-es/768/threshold 0.35 — el 0.55 heredado habría sido dañino con el modelo nuevo), y **M2 baja a estrangulamiento tranquilo** (arranca en hueco real post-universidad/cge P2).
2. ~~Nombre del framework/monorepo~~ — **FIRMADO 2026-07-16: `exo`** (exocortex).
3. **archive/ en el ranking** — post-E1, con datos del side-by-side.
4. **Marketplace: rename vs repo nuevo** — en M1b (preferencia actual: rename por los redirects).

## 11. Audit trail

- Informes de consultoría adversarial (verificación primaria citada en cada uno): `docs/superpowers/consultas/2026-07-16-framework/informe-consultor-{framework,engine,thin,thick,roadmap}.md`.
- Convergencias auto-adjudicadas per doctrina (consulta adversarial y ratificación); decisiones de Paul en sesión: objetivo personal-primero, reemplazo de basic-memory (→ suavizado a gated-por-Fase-0 tras disenso del consultor #1, re-firmado), jubilación de superpowers, lenguaje abierto ("migrar de lenguaje no es bloqueante si mejora"), Fase 0 + estrangulamiento, ejecución vía fabrica.
