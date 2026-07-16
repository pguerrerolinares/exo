# Informe — consultoría adversarial #2: diseño del engine (Sección 2)

Consultor independiente. Verificación primaria: código instalado de basic-memory 0.22.1 y fastembed 0.8.0 (venv de uv), repo kbx, config real, log de retrieval, mediciones de latencia en esta máquina, y docs externas (MCP SDKs, fastembed-rs, sqlite-vec, hugot). Cada afirmación clave lleva su evidencia.

**Veredicto global**: la arquitectura de la sección es razonable y el estrangulamiento E1→E3 está bien pensado, pero la sección se ratifica solo con 4 correcciones: (1) los dos candidatos de modelo de Fase 0 **no existen en el stack actual** — hay que cambiarlos; (2) el hueco de embeddings-desde-el-engine es real y la respuesta correcta al mandato actualizado es **Rust para el binario nuevo, sin migrar kbx**; (3) el shadow-mode de E2 está mal costurado — basic-memory es file-first y el filesystem ya es la API; (4) el alcance de "paridad de índice" hay que recortarlo con los datos de uso real, que ya existen y son contundentes.

---

## Eje 1 — Fase 0 (diagnóstico): DÉBIL tal como está escrita, direccón correcta

### 1.1 Los candidatos de modelo están rotos (verificado contra el paquete instalado)

Listado completo de `TextEmbedding._list_supported_models()` del fastembed 0.8.0 que usa basic-memory:

- **`multilingual-e5-small` NO está soportado.** Tampoco e5-base. El candidato primario del diseño no se puede configurar.
- **`bge-m3` NO está soportado** (ni denso ni en otra clase). La alternativa tampoco.
- Lo multilingüe realmente disponible: `intfloat/multilingual-e5-large` (1024 dims, 2.24 GB), `sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2` (384 dims, 0.22 GB), `paraphrase-multilingual-mpnet-base-v2` (768, 1 GB), `jinaai/jina-embeddings-v3` (1024, 2.29 GB), `jinaai/jina-embeddings-v2-base-es` (768, 0.64 GB, bilingüe español-inglés).

### 1.2 Prefijos E5: el stack entero los ignora (verificado en código)

- `fastembed_provider.py:323-325`: `embed_query()` llama a `embed_documents([text])` — mismo camino para query y documento, sin prefijo alguno.
- En fastembed mismo, `multilingual-e5-large` lo sirve la clase `PooledEmbedding`, cuyo `query_embed()` y `passage_embed()` son alias literales de `embed()` — **fastembed tampoco prefija**.
- Los autores de E5 documentan en el model card (FAQ de HF) que los prefijos `query:`/`passage:` son obligatorios porque así se entrenó el modelo, con degradación de rendimiento si faltan. (No pude extraer la cita literal por truncado de la página; el hecho decisivo — que nadie en el stack prefija — está verificado en código.)

Conclusión: usar cualquier E5 vía basic-memory+fastembed es correr el modelo off-label. **Descarta la familia E5 para el experimento de Fase 0** (no para el engine propio, donde tú controlas los prefijos — ver eje 2).

### 1.3 Candidatos reales para el experimento config-only

1. **`jinaai/jina-embeddings-v2-base-es`** — bilingüe español-inglés, entrenado para retrieval, sin requisito de prefijos, 0.64 GB. Es exactamente el perfil de la KB (castellano con términos técnicos en inglés). Candidato primario. Requiere `semantic_embedding_dimensions: 768` en config (ver gotcha abajo).
2. **`paraphrase-multilingual-MiniLM-L12-v2`** — 384 dims (drop-in: ni siquiera cambia el schema de la tabla vectorial), pequeño y rápido. Contra: es un modelo de paráfrasis (simétrico), más débil en retrieval asimétrico query→documento. Buen segundo brazo del experimento por lo barato que es.
3. **Provider `litellm` + modelo API multilingüe** (Cohere embed-multilingual / Voyage / OpenAI): basic-memory ya lo soporta, incluidos `semantic_embedding_{document,query}_input_type` (¡el único camino del stack actual con asimetría query/passage real!). Contra: red, API key, y la KB personal sale de la máquina. Como experimento de techo de calidad vale; como default personal, decide tú el trade-off privacidad.

### 1.4 Gotchas de config verificados en código (te ahorran una tarde)

- El provider fastembed **hardcodea 384 dims por defecto** y la factory solo pasa `dimensions` si `semantic_embedding_dimensions` está seteado (`embedding_provider_factory.py:142-143`). Con un modelo de 768/1024 dims sin ese campo: `RuntimeError` en el primer embed (`fastembed_provider.py:316-320`).
- El alias corto solo existe para `bge-small-en-v1.5`; los demás modelos van con **nombre HF completo** en `semantic_embedding_model`.
- El cambio de dims **recrea la tabla vectorial automáticamente** (`sqlite_search_repository.py:470-479`) y la columna `embedding_model` (formato `Provider:modelo:dims`) invalida los chunks del modelo anterior → el re-embed es automático vía sync.
- **Re-index barato: CONFIRMADO.** La KB son 117 entidades, 5.154 chunks, ~1.7 MB de texto. Minutos de CPU incluso con el modelo de 768.

### 1.5 El eval set: existe la materia prima, pero no como dice el diseño

- El `retrieval-logger.sh` de reflex loguea `search_notes` con la **query en el campo `target`**: hay **46 queries únicas reales** (5–16 jul) en `~/.claude/reflex-retrieval-log.jsonl`. La afirmación del diseño es cierta con matiz: son queries **del agente**, no de Paul, y están sesgadas a codenames/keywords ("kbx", "cmm", "cliente-c", "lighthouses bot…") donde FTS ya gana. Los casos donde bge-small-en duele — queries conceptuales en castellano — están infrarrepresentados. Añade a mano 5-10 casos duros que Paul recuerde ("busqué X y no salió la nota Y").
- No hay ground truth en el log: hay que **etiquetar** qué nota debía salir. Etiqueta las 46, no 15-20 — el coste marginal es cero y el set se reutiliza (ver eje 5).
- Sobre "¿15-20 dan señal o es teatro?": como test de hipótesis de proporciones, teatro. Como **comparación pareada** sobre las mismas queries (cuántas arregla el modelo nuevo, cuántas rompe), n=20-46 da señal decisoria de sobra para un sistema personal. Define el gate numérico antes de mirar: p.ej. "≥5 queries arregladas y ≤1 rota".
- **El 0.55 no sobrevive al cambio de modelo.** La similitud es coseno vía `1 - L2²/2` sobre vectores L2-normalizados (el provider normaliza explícitamente, `fastembed_provider.py:303-313`), pero cada modelo tiene su distribución de similitudes (los E5/jina comprimen hacia arriba; los paraphrase se dispersan). Re-sweep del threshold con el mismo eval set, es un bucle barato.
- Falta en el diseño: **atribución de fallos**. Si recall no mejora, el gate "engine-first ratificado" solo es válido si los fallos son semánticos y no de FTS/fusión/threshold. Clasifica cada miss (FTS-miss / vector-miss / threshold-miss usando `search_type` explícito por query) o el gate no distingue "modelo malo" de "fusión mal calibrada".

### 1.6 Latencia (medida en esta máquina)

- `basic-memory --version`: 0.3s. `tool read-note`: **~3.0s**. `tool search-notes` (híbrida, proceso frío): **~3.4-3.9s** (arranque Python + carga ONNX + query por proceso).
- El diagnóstico del diseño (coste de imports del CLI, mitigado con cache) es correcto y el hook de recall ya lo mitiga bien (cache 30min + refresh en background con `setsid`). Ojo: la mitigación solo cubre SessionStart; cualquier búsqueda semántica lanzada desde un hook pagará los ~3.5s siempre. Ese es el argumento real pro-engine-binario, y es sólido.

**Veredicto eje 1: DÉBIL en los detalles, SÓLIDO en la intención.** Con los candidatos corregidos y el gate con atribución, la Fase 0 es exactamente lo que hay que hacer antes de escribir código.

---

## Eje 2 — Indexer propio y el hueco de embeddings: HUECO CONFIRMADO; decisión de lenguaje

### 2.1 El hueco existe

Confirmado: la Sección 2 no dice cómo genera embeddings un binario Go. fastembed es Python (y su port Rust); no hay fastembed-go. Las opciones reales, evaluadas con el mandato actualizado (lenguaje abierto, coste de desarrollo no bloqueante):

### 2.2 Opciones de lenguaje

**(a) Go + ONNX (hugot + onnxruntime_go).** Viable — hugot tiene `FeatureExtractionPipeline` para embeddings. Coste: tres costuras nativas simultáneas: la shared library de ONNX Runtime (distribución/versionado manual), tokenizers vía FFI, y cgo para SQLite con FTS5 (`mattn` + build tag) más los bindings cgo de sqlite-vec. Cada una es "doable"; las tres juntas en un binario personal multi-máquina son fricción permanente. Reuso del kbx actual: 100%.

**(b) Go + sidecar Python para embeddings.** Mantienes vivo un proceso/venv Python — exactamente la clase de dependencia que el diseño quiere jubilar. Descartada salvo como puente temporal.

**(c) Rust + fastembed-rs.** Verificado: fastembed-rs trae **`multilingual-e5-small`, `-base`, `-large` Y `bge-m3` nativos** — los dos candidatos que el diseño quería y que el stack Python no tiene. Los prefijos E5 los pones tú (es tu engine). `ort` enlaza ONNX Runtime con binarios descargados/estáticos → binario único sin drama de shared libs; `rusqlite` con SQLite bundled + feature `fts5`; sqlite-vec tiene crate (0.1.9). SDK MCP oficial (`rmcp`, modelcontextprotocol/rust-sdk) activo. Arranque ~ms. Coste: se pierden las ~2.948 líneas de producción Go de kbx… **si migraras kbx, que no hace falta** (ver 2.3). Riesgos: fluidez de Paul en Rust (factor abierto — aunque su propio case study en la KB sobre reescrituras agénticas a escala reduce esa barrera), y fastembed-rs es proyecto comunitario de un maintainer principal (Anush008, órbita Qdrant) — riesgo de mantenimiento comparable al que ya aceptas con sqlite-vec.

**(d) Python — fork/vendorizar basic-memory.** Dos vetos: (1) **basic-memory es AGPL-3.0-or-later** (verificado en METADATA del paquete). Vendorizar sus piezas convierte el framework genérico en derivado AGPL — si algún día quieres publicarlo permisivo (MIT/Apache), este camino lo cierra hoy. (2) Arrastra el problema medido: 0.3-3s de arranque CLI por invocación es lo contrario de la ergonomía para hooks. Máxima paridad, mínimo futuro. Corolario importante: **tampoco copies código de basic-memory al engine** en ningún lenguaje — copiar el *diseño* (schema, fórmula de fusión, chunking) es legítimo; copiar código te hace derivado AGPL.

**(e) TypeScript/Bun.** El SDK MCP más maduro del ecosistema y arranque decente (~30ms con Bun), pero el stack de embeddings es el eslabón débil (fastembed-js semi-abandonado; transformers.js funciona pero es la opción más lenta y menos controlable). No gana a Rust en ningún eje salvo familiaridad; no gana a Go en reuso. Descartada.

### 2.3 Recomendación (con la costura que disuelve el dilema)

**Rust para el binario nuevo (indexer + search + write-path + futuro MCP server). Y kbx NO se migra.** La clave está ya en tu propio diseño: el contrato entre capas es el **envelope JSON versionado + CLI**. Ese contrato es agnóstico al lenguaje — hace que el lenguaje sea una decisión *por binario*, no *por sistema*. `kbx doctor/budget/stale/targets` siguen en Go, tal cual, con sus tests; el binario Rust nuevo cubre lo que kbx no tiene (indexer, search, write). Se portan comandos de kbx a Rust oportunistamente o nunca. Es el mismo estrangulamiento que ya firmaste para basic-memory, aplicado a kbx. Así el "coste de migrar ~40 .go" es cero en v1 y las 4.017 líneas de tests Go quedan como spec de comportamiento si algún día portas.

Único punto de duplicación real a vigilar: el parser de frontmatter existirá en Go (kbx) y Rust (engine). Aceptable — es la pieza más pequeña y estable del sistema.

**Trade-off explícito de la recomendación**: pagas aprender/mantener Rust y un segundo binario en el repo-ecosistema, a cambio de: los modelos multilingües exactos que quieres con prefijos correctos, binario único estático, arranque en ms para hooks, y cero costuras cgo/sidecar. Si tras Fase 0 resulta que la semántica local NO es load-bearing (posible: mira el sesgo FTS de tus queries reales), la alternativa honesta es Go puro con FTS5+grafo y la semántica como provider pluggable (API o proceso aparte) — en ese mundo Go gana por reuso. **Por eso la decisión de lenguaje debe firmarse DESPUÉS de Fase 0, no antes.** La Fase 0 no solo decide urgencia del write-path: decide si el engine necesita embeddings dentro.

### 2.4 Paridad del índice: es más grande de lo que la sección sugiere

El retrieval de basic-memory no es "FTS + coseno": chunking con reglas de merge por secciones (~700 líneas en `search_repository_base.py`), fusión score-based `max(v,f) + bonus·min(v,f)` con clave `(type,id)` (bug #982 aprendido), gate de FTS, normalización BM25, calibración distancia→similitud, threshold. Además el índice actual tiene **3 tipos de fila**: entidades (117), observations (1.417) y relations (540) — tu KB usa de verdad la gramática de observations/relations y las búsquedas indexan sobre ella. El diseño debe decidir **explícitamente** si el indexer propio replica esa gramática o solo notas+wikilinks (recomiendo lo segundo para v1 — ver eje 5 — pero que sea una decisión, no una omisión que descubras en E1 comparando side-by-side).

sqlite-vec: **activo de nuevo** (hiato terminado, financiado por Mozilla; releases 0.1.9/0.1.10-alpha en 2026) pero **pre-v1 con breaking changes anunciados de SQL API y formato de storage**. basic-memory ya te expone a ese riesgo; un indexer propio debe fijar versión y asumir posible migración de tabla vectorial. No es bloqueante, es un pin.

**Veredicto eje 2: el hueco era real; con Rust + estrangulamiento-también-para-kbx queda cerrado. La paridad del índice estaba infraestimada.**

---

## Eje 3 — Write-path: SHADOW-MODE MAL COSTURADO; hay camino más simple

Hallazgo que cambia el E2: **basic-memory es file-first con watch service**. El servidor MCP arranca `sync_and_watch` en background (`sync/background_sync.py`) y absorbe ediciones externas de ficheros al índice (tu config: `sync_changes: true`). Es el flujo soportado de fábrica (gente editando en Obsidian). Implicación:

- El shadow-mode del diseño ("engine propone diff, basic-memory ejecuta, divergencias logueadas") **invierte la costura natural**. No necesitas que basic-memory ejecute nada: el engine escribe el fichero markdown directamente (KB en git = rollback, como ya dice el diseño), el watch de basic-memory lo indexa solo, y la verificación de divergencia es "kbx/engine doctor compara lo que el índice absorbió vs lo esperado". Menos coreografía, misma red de seguridad, y ejercitas desde el día 1 el write-path real que quedará en E3.
- ¿Es implementable el shadow-mode original? Sí, pero es un handwave *innecesario*: te obliga a mantener un traductor engine-diff → llamadas MCP write_note/edit_note que tirarás en E3.

Qué pierde Paul de basic-memory write (verificado):
- `edit_note` incremental (append/prepend/find_replace/replace_section): el write-path propio debe cubrir al menos append y replace_section (los que usa /documenta).
- `move_note` con actualización de links: tu config tiene `update_permalinks_on_move: true` — está en uso. O el engine v1 cubre move+links, o **no ofrece move** y lo deja en basic-memory hasta E3. No lo dejes a medias: un move sin actualización de links corrompe el grafo silenciosamente.
- Sync cloud bidireccional: **no lo usas** (mode `local`, `bisync_initialized: false`, sin workspace). Pérdida cero hoy; si el framework genérico quiere ofrecer cloud algún día, es un problema de otro milestone.

**Veredicto eje 3: la sustancia (write frontmatter-aware + search-before-write nativo + veto RO por defecto) es SÓLIDA; el mecanismo de transición E2 hay que reescribirlo a "file-first + doctor-verify".**

---

## Eje 4 — MCP server: SÓLIDO

- SDK Go oficial: v1.0.0 estable desde 2025, hoy v1.7+, mantenido con Google. SDK Rust oficial (`rmcp`) activo. El riesgo "¿hay SDK maduro?" está muerto en ambos lenguajes.
- Coste de paridad de tools, medido con tu propio log (11 días): `read_note` 265, `search_notes` 48, `recent_activity` 4, `build_context` 1. **El hot-path son 3 tools.** `build_context` con 1 uso en 11 días es candidato a no existir en el MCP propio (o quedar como composición de search+read en la capa thin).
- Dejarlo como última milestone con basic-memory de MCP hasta entonces: correcto y coherente con el estrangulamiento.

**Veredicto eje 4: SÓLIDO. Ratificado con el recorte de build_context.**

---

## Eje 5 — Alcance: SÍ se estaba tragando un proyecto; recortes concretos

Si "indexer propio" significa paridad total con basic-memory (gramática observations/relations, chunking idéntico, edit_note completo, move con links, migraciones), kbx se traga una reimplementación de un producto con años de bug-fixes (#872, #982, corrupción de cache ONNX…). Recortes para una v1 alcanzable en noches/fines de semana:

1. **Indexer v1**: notas + frontmatter + wikilinks. SIN gramática de observations/relations (el grafo v1 = links entre notas). Es la mayor reducción de superficie y la más defendible: tus 46 queries reales van contra títulos/contenido, no contra observation-rows.
2. **Search v1**: FTS5 + vector + fusión. Copia el *diseño* de la fusión de basic-memory (fórmula, clave (type,id), threshold configurable) — no el código (AGPL).
3. **Write v1**: write nueva nota + append + replace_section, con search-before-write. SIN move (queda en basic-memory hasta E3 o hasta que el engine lo haga con links).
4. **SIN build_context, SIN cloud, SIN daemon** (mtime/git incremental como ya dice la sección).
5. **El eval set de Fase 0 se convierte en el test de regresión del engine**: mismo etiquetado, side-by-side E1 contra basic-memory. Un activo, dos usos — esto además convierte el "side-by-side" de E1 de intención en mecanismo medible.

Con esos recortes, la v1 del engine es un proyecto acotado (indexer simple + 2 comandos de search + 3 de write), no un cuarto proyecto. El coste de oportunidad contra cge/OpenWisdom queda contenido porque kbx-Go no se toca y la Fase 0 (una tarde, verificada barata) puede incluso desactivar la urgencia de todo lo demás.

---

## Top 3 riesgos

1. **Ejecutar Fase 0 con los candidatos escritos**: multilingual-e5-small/bge-m3 no existen en fastembed 0.8.0, y cualquier E5 iría sin prefijos. Tal cual está redactada, la Fase 0 moriría en el paso 1 o, peor, mediría un modelo off-label y "demostraría" que lo multilingüe no ayuda.
2. **Infraestimar la paridad del índice** (gramática observations/relations, chunking, fusión afinada): es donde "consolidación" muta en reimplementación de basic-memory. Mitigación: los recortes del eje 5 como decisiones explícitas en la spec.
3. **Decidir lenguaje antes de Fase 0**: si la semántica local no es load-bearing (plausible por el sesgo FTS de las queries reales), Rust pierde su ventaja decisiva y Go+FTS+grafo gana por reuso. La decisión de lenguaje es un output de Fase 0, no un input.

## Qué ratifico

- Fase 0 diagnóstica antes de código, con eval real y gate — la idea, corregidos candidatos, atribución de fallos y re-sweep del threshold.
- Engine como evolución del ecosistema kbx con envelope JSON versionado — es además la costura que permite Rust sin migrar Go.
- Estrangulamiento E1→E3, write acotado a /documenta y /consolida, veto RO por defecto, MCP propio al final, KB-en-git como rollback.
- El diagnóstico de latencia (CLI-por-invocación) — medido y confirmado: 3.0-3.9s por llamada fría.

## Fuentes externas

- [MCP Go SDK releases (v1.0.0, oficial con Google)](https://github.com/modelcontextprotocol/go-sdk/releases) · [blog MCP: SDKs beta spec 2026-07-28](https://blog.modelcontextprotocol.io/posts/sdk-betas-2026-07-28/)
- [fastembed-rs (modelos soportados: multilingual-e5-*, bge-m3)](https://github.com/Anush008/fastembed-rs) · [docs.rs EmbeddingModel](https://docs.rs/fastembed/latest/fastembed/enum.EmbeddingModel.html)
- [sqlite-vec releases (0.1.9/0.1.10-alpha, 2026)](https://github.com/asg017/sqlite-vec/releases) · [issue #226 mantenimiento](https://github.com/asg017/sqlite-vec/issues/226) · [alexgarcia.xyz/sqlite-vec](https://alexgarcia.xyz/sqlite-vec/)
- [hugot (pipelines ONNX en Go)](https://github.com/knights-analytics/hugot)
- [Model card multilingual-e5-small (idiomas, incl. español)](https://huggingface.co/intfloat/multilingual-e5-small)

Evidencia local: `fastembed_provider.py` (líneas 41-43, 297-325), `embedding_provider_factory.py` (118-166), `sqlite_search_repository.py` (418-523), `search_repository_base.py` (2150-2266), `sync/background_sync.py`, `basic_memory-0.22.1.dist-info/METADATA` (AGPL), `~/.claude/reflex-retrieval-log.jsonl` (318 eventos, 46 queries únicas), `kbx` (2.948 líneas prod / 4.017 test), `memory.db` (117 entidades / 5.154 chunks / 1.417 observations / 540 relations), timings CLI medidos 2026-07-16.
