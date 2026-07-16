# Informe consultor #4 — Sección 4: KB y formato (capa thick)

Consultor independiente. Verificación primaria realizada sobre: la KB real (136 .md, frontmatter muestreado), `~/.basic-memory/memory.db` (schemas + queries directas), `~/.basic-memory/config.json`, código de kbx (`internal/frontmatter`, `internal/index/schema.go`, `internal/doctor`, `internal/stale`, `internal/targets`), `basic-memory-recall.sh`, `retrieval-logger.sh` + su log real (318 eventos, 5–16 jul), `/documenta` (commands/documenta.md), SKILL.md de consolida, y la spec OKF v0.1 (fetch del repo GoogleCloudPlatform/knowledge-catalog). Cada afirmación clave lleva su evidencia.

**Veredicto global**: la sección es la más sólida de las cuatro que he revisado — la tesis central ("no se migra nada, se re-indexa") está bien fundada y la degradación de la gramática está justificada por los datos de uso reales, que son contundentes. Se ratifica con 3 endurecimientos obligatorios (permalinks, gate E1, canary de kbx) y 2 correcciones de método (validación OKF como doctor-check no write-gate; template clean-room no destilado). Ninguno cambia la arquitectura; todos cambian redacción de la spec.

---

## Eje 1 — "Solo se regenera el índice": CIERTO con 3 matices, uno crítico

**Lo verificado a favor**: el índice es casi 100% derivado. Los permalinks NO son estado exclusivo del DB: `ensure_frontmatter_on_sync: true` en config hace que basic-memory los persista en el frontmatter de cada nota — verificado 112/112 notas markdown con `permalink:` en frontmatter, exactamente las 112 entidades md del índice. Checksums, tamaños y el grafo se reconstruyen de los ficheros. La tesis "migración de thick = re-index" se sostiene.

**Matiz crítico — permalinks**: la slugificación de basic-memory no es trivial (verificado en DB: "Paul - perfil de trabajo.md" → `paul-perfil-de-trabajo`; em-dashes colapsados, acentos transliterados: "racionalización" → "racionalizacion"; y `permalinks_include_project: true` mete el prefijo `kb-demo/` en el valor). Si el indexer propio REGENERA permalinks con su propia slugificación en vez de HONRAR el frontmatter, rompe el contrato de lectura más usado del sistema: `read_note` es el 83% del tráfico real (265 de 318 eventos en el retrieval-log), y recall.sh lo llama con `core/core-index` — un identifier SIN prefijo de proyecto que basic-memory resuelve con fuzzy match contra `kb-demo/core/core-index`. **Regla para la spec**: el indexer jamás regenera un `permalink:` existente; genera solo para notas nuevas; el read-path acepta identifier con/sin prefijo de proyecto y resolución por título.

**Matiz 2 — created_at no es derivable**: ninguna nota tiene `created:` en frontmatter (grep completo: 0 hits). Un re-index pierde las fechas de creación del DB. Consumo real: cero — kbx solo referencia `created_at` en fixtures y en el canary, ninguna query de producción lo usa (stale usa edad de commit git; targets usa `last_commit`). El único consumidor era `recent-activity` de basic-memory (usa `updated_at`), que se estrangula de todos modos. **Acción**: declarar en la spec "recencia = git, no mtime ni created_at del índice" — kbx targets ya lo hace así; el digest del recall post-engine debe hacer lo mismo (un clone fresco resetea mtimes y contaminaría un digest basado en mtime durante días).

**Matiz 3 — el corpus indexado no es "toda la KB"**: 136 .md en disco vs 112 entidades md. Los 24 restantes viven en dotdirs (`.claude/`, `.omc/`, `.superpowers/`) que basic-memory excluye. El indexer propio debe replicar la exclusión de dotdirs o el side-by-side de E1 comparará corpus distintos y los deltas serán falsos. Además hay 5 entidades no-markdown (`.tex`, `.json`, `.cls`, `.pdf` — las únicas con permalink NULL); decidir si el indexer v1 las indexa (recomiendo que no: son las únicas filas anómalas y dos son basura en la raíz, ver eje 6).

**Veredicto eje 1: RATIFICAR** con la regla de permalinks como texto obligatorio de spec, no como detalle de implementación.

---

## Eje 2 — Degradar observations en escritura: DIRECCIÓN CORRECTA, GATE COJO

**Quién consume la gramática HOY (verificación exhaustiva)**:
- `basic-memory-recall.sh` (SessionStart): solo `read-note core/core-index` + `recent-activity` del que extrae únicamente permalinks. No toca observations ni relations. Verificado línea a línea.
- kbx: **no ejecuta ni una query sobre la tabla observation** (grep de queries en doctor/stale/targets/budget: cero). Solo la declara en el schema-canary (ver abajo).
- Ningún script, hook ni skill pasa `entity_types`/`categories` a search_notes (grep sobre commands/, skills/, scripts/: cero).
- El único productor sistemático es `/documenta` Paso 3 ("observations en formato `- [categoria] contenido`").

**El dato que justifica degradar**: las categorías son un vocabulario sin control — `decisión` (140) + `decision` (136), `patrón` (138) + `pattern` (43), `aprendizaje` (238) + `learning` (9), más `note`, `insight`, `hallazgo`... Como estructura consultable ya es ruido bilingüe duplicado. Nadie filtra por categoría y nadie podría hacerlo de forma fiable.

**El dato que el diseño subestima**: en `search_index` hay 1.537 filas de observation vs 137 de entity — las observations son ~65% del corpus FTS y los hits a nivel de observación son granularidad real de la búsqueda actual. Degradarlas a bullets no borra el texto (queda en el body, indexable como chunk), pero cambia el ranking y la granularidad de los resultados. Eso es exactamente lo que E1 debe medir — y aquí está el problema:

**"Si E1 lo detecta se reabre" es un gate cojo tal como está escrito.** El eval set actual (46 queries únicas del retrieval-log, según el informe engine) está etiquetado a nivel de nota y sesgado a codenames donde FTS gana. Si ninguna query etiquetada tiene su señal en una fila de observation, los misses serán invisibles por construcción y el gate dará verde vacío. **Fix concreto y barato (~1h)**: antes del cutover, replay de las 46 queries reales contra basic-memory actual, marcar cuáles devuelven hits de fila observation en el top-k, y garantizar que ese subgrupo está representado y etiquetado en el eval set. Con eso el gate pasa de intención a mecanismo.

**Coupling duro encontrado — el canary de kbx**: `internal/index/schema.go` declara como schema consumido las tablas `observation` (id, entity_id, content, category) y `relation` completas. Si el índice del engine no crea la tabla observation ("sin fila propia"), `kbx doctor` dispara `schema_drift`, y la SKILL de consolida tiene instrucción explícita de **falla-fuerte** ante schema_drift ("para con mensaje accionable"). Es decir: el punto 4 de esta sección, tal cual, rompe consolida el día del cutover. Fix trivial pero DEBE estar en la spec: el cutover de thick actualiza la lista `consumed` del canary de kbx en el mismo commit (o el engine mantiene tablas schema-compatibles). Que no sea una sorpresa de E1.

**El histórico (1.417 obs) no se reescribe**: correcto y verificado viable — el texto vive en los .md como bullets `- [categoria] contenido`, un indexer de "notas planas" los indexa como contenido normal. Sin objeción.

**Veredicto eje 2: RATIFICAR la degradación**, con el eval set estratificado por observation-hits como condición del gate y la actualización del canary kbx nombrada en la spec.

---

## Eje 3 — Relations tipadas vs wikilinks: EL TIPO NO ES LOAD-BEARING (verificado)

- Distribución real: 452/540 relations son genéricas (`relates_to` 227 + `links_to` 225 — estas últimas son las que basic-memory genera de wikilinks planos). La cola "tipada" incluye basura de parsing (`Relacionado:`, `**Dominio:**`, `Detalle:`) — la gramática tipada ya está degradada en la práctica.
- Consumidores del grafo: `kbx doctor` (orphan: `WHERE id NOT IN (SELECT from_id...)`) y `kbx stale` (degree: `COUNT(*) FROM relation WHERE from_id/to_id`) usan solo EXISTENCIA de aristas, jamás `relation_type`. Verificado en las queries.
- `build_context` (el único consumidor de traversal): **1 uso en 11 días** de instrumentación. No es load-bearing; es decorativo.
- El único tipo con intención sistemática es `bitacora` (canon→su log, 20 filas, presente en doctrina-agentes, perfil y backlog), pero /documenta y consolida navegan por **convención de nombre** (`log/<slug>-bitacora.md`), no por el tipo de relación. Perder el tipo no rompe ningún flujo verificable.

Mantener wikilinks como contrato es correcto y suficiente: las aristas sobreviven (un wikilink plano genera relación igual), doctor/stale siguen funcionando. **Un matiz para la spec**: fijar la resolución de `[[Título]]` → entidad (hoy: match por título, `to_name`; los links a notas inexistentes quedan con `to_id NULL` y el sistema los tolera). El indexer propio debe replicar esa tolerancia — un link roto no puede ser error de indexado.

**Veredicto eje 3: RATIFICAR sin reservas.** Los datos no podrían ser más claros.

---

## Eje 4 — OKF: COHERENTE, pero validación en doctor, jamás en el write-path

Spec real leída (OKF v0.1): único campo **obligatorio** es `type` (string no vacío, valores libres, "consumers must tolerate unknown types" — `type: note` conforma trivialmente). `title`/`description`/`tags`/`timestamp` recomendados. **Campos de extensión explícitamente permitidos** ("producers may include additional keys; consumers must preserve unknown keys") — `tier`, `permalink`, `kbx_*` son exactamente eso. No hay conflicto de campos: OKF no define `permalink` ni `created`. Reserved filenames `index.md`/`log.md`: la KB no tiene ninguno (verificado con find). "OKF + extensiones propias" es coherente con la letra de la spec.

**Dos fricciones reales**:
1. **11 notas fallan la conformance HOY**: sin `type:` en frontmatter (10 destilados de `projects/` + `learnings/frontend-y-librerias.md`). Backfill one-shot trivial, pero hazlo antes de encender el check o nace en rojo.
2. **El riesgo del punto 2 es la palabra "valida"**: si validar = rechazar writes, /documenta puede fallar al cierre de sesión por un frontmatter incompleto — exactamente el tipo de fricción que el contrato de memoria no tolera ("nunca bloquees el cierre de sesión", texto literal de /documenta). **Regla para la spec**: la validación OKF vive en `doctor` como finding (offline, consolida); el write-path **auto-completa** campos faltantes (como hace `ensure_frontmatter_on_sync` hoy) y jamás rechaza.

**Matiz de honestidad narrativa**: el claim "directorio markdown que cumple spec abierta" debe acotarse al árbol indexado. Los dotdirs (`.omc/`, `.claude/`, `.superpowers/`) contienen 24 .md sin frontmatter que romperían una conformance literal del bundle. Con el alcance acotado ("el árbol de notas cumple OKF; el estado de tooling vive en dotdirs fuera del bundle"), el claim es defendible.

**Veredicto eje 4: RATIFICAR** como convención + doctor-check con las dos correcciones. El beneficio es narrativo y el coste es ~cero solo si se queda fuera del write-path.

---

## Eje 5 — templates/: NO ES YAGNI, pero el método propuesto es el equivocado

El coste es real pero pequeño (4-5 ficheros cortos). El problema es **"se destila DE la instancia de Paul quitando lo personal"**: eso es una blacklist, y un miss = dato personal en un repo pensado para terceros. La KB contiene finanzas (finanzas-empresa-x), perfil personal, timeline vital. Destilar-por-borrado exige auditar cada línea del resultado contra todo lo que NO debe estar — más caro y más peligroso que la alternativa.

**Método correcto: clean-room (whitelist).** El template se escribe desde cero — core-index vacío con la línea de routing, profile.md plano comentado, README, estructura de tiers — mirando la instancia solo como referencia de forma. Para 5 ficheros el clean-room es objetivamente más barato que el destilado auditado, y elimina la clase de riesgo entera en vez de mitigarla.

**Momento**: sin consumidor hasta que exista un tercero; no bloquea nada del estrangulamiento. Se puede diferir a "cuando el engine tenga historia de install". Si se hace ya por el beneficio narrativo del monorepo, que sea clean-room. RATIFICAR el qué, corregir el cómo.

---

## Eje 6 — Lo que la sección no menciona: 4 huecos, 2 con acción

1. **archive/ SÍ se indexa hoy** (38 de 117 entidades — el 32% del índice). El budget lo excluye, la búsqueda no. El indexer propio debe tomar esta decisión EXPLÍCITA. Recomiendo: indexar como hoy — excluirlo sería cambiar corpus y motor a la vez, y la atribución de deltas en E1 se vuelve imposible. Si algún día molesta en el ranking, se recorta después de E1 con datos.
2. **.omc/ dentro del repo KB** (project-memory.json, sessions, state, workflows — estado de orquestación versionado junto a las notas). No rompe el engine (dotdir excluido) pero contamina el claim OKF y cualquier template. Acción barata: una línea en la spec decidiendo si se queda (documentado como fuera-del-bundle) o se muda. Lo mismo aplica a `.superpowers/` y `.claude/`.
3. **Root files**: hay 2 ficheros no-nota en la raíz (`developercv.cls`, `fontawesome.pdf` — además las únicas entidades con permalink NULL junto a los assets de archive/cv-assets). `kbx doctor` ya los flaggea (`root_file`); limpieza trivial vía consolida antes del cutover para que el baseline de E1 nazca limpio. AGENTS.md/metodologia/timeline: indexados como entidades normales con tier — nada que hacer, el harness los sirve desde el filesystem.
4. **Sync/watch**: el watch daemon corre hoy (pid vivo, verificado). El plan mtime-incremental cubre las ediciones manuales de Paul en Obsidian sin pérdida: cualquier save toca mtime y la siguiente invocación del engine re-indexa; borrados/renames los detecta el listado completo (gratis a 136 ficheros). El único caso raro es el clone fresco (mtimes reseteados → todo parece recién editado), que se neutraliza con la regla "recencia = git" del eje 1. Ningún blocker; documentar la ventana de staleness aceptada (entre edición manual e invocación siguiente no hay índice fresco — hoy el watch da ~1s; con incremental-on-invoke es "hasta el próximo comando", lo cual para este uso es correcto).

**Puntos 3 y 6 de la sección** (tiers dueño-único; core-index): RATIFICAR sin cambios. Verificado que consolida ya opera como consumidor puro de kbx (budget/doctor/stale con falla-fuerte) y que recall.sh inyecta el core-index con guard de contenido y techo de 6.144B en el hook (coherente con el presupuesto de 3.6KB de la nota, holgura 1.7x). El check nuevo de doctor "un core jamás recibe appends fechados" es barato y va en la dirección correcta (regla en un solo dueño).

---

## Top 3 riesgos

1. **Regeneración de permalinks en el re-index.** Si el indexer no honra el frontmatter, rompe el 83% del tráfico real de lectura (read_note) y todos los memory packets. Es el único riesgo de esta sección capaz de romper el sistema en silencio.
2. **Gate E1 ciego a observations.** Sin eval set estratificado por observation-hits, "si E1 lo detecta se reabre" es un gate que no puede disparar. 65% del corpus FTS quedaría sin vigilancia.
3. **Schema-canary de kbx vs índice sin tabla observation.** Consolida falla-fuerte por diseño ante schema_drift; el cutover debe actualizar la lista `consumed` de kbx en el mismo movimiento o el primer /consolida post-cutover muere.

## Cambios concretos de redacción para la spec

1. Punto 1: "el indexer honra `permalink:` de frontmatter y jamás lo regenera; genera solo para notas nuevas; el read-path acepta identifier con/sin prefijo de proyecto y por título; recencia = git".
2. Punto 4: "el eval set de E1 incluye las queries reales cuyo top-k actual contiene hits de fila observation (replay contra basic-memory antes del cutover)".
3. Punto 4 (coupling): "el cutover actualiza la lista `consumed` del schema-canary de kbx en el mismo commit".
4. Punto 2: "la validación OKF vive en doctor como finding; el write-path auto-completa y nunca rechaza" + backfill one-shot de las 11 notas sin `type:` + claim de conformance acotado al árbol indexado.
5. Punto 5: "el template se escribe clean-room (whitelist), no se destila por borrado de lo personal".
6. Añadir tres decisiones explícitas: archive/ se indexa (como hoy); dotdirs fuera del bundle (documentado); limpieza de root files antes del baseline E1.

## Qué ratifico

- La tesis central: KB intacta, índice derivado, migración = re-index (con la regla de permalinks).
- La degradación de observations/relations a estilo opcional en escritura — los datos de uso (0 queries por categoría, 1 build_context en 11 días, 84% de relations genéricas, taxonomía bilingüe duplicada) la justifican sobradamente.
- Wikilinks como contrato load-bearing (grafo, orphan, stale, degree) — verificado que es exactamente lo que kbx consume.
- OKF como convención con extensiones (coherente con la spec real) — como doctor-check.
- Tiers/presupuestos dueño-único en el engine y core-index sin cambios.
- templates/ como objetivo — con método clean-room.

---
Evidencia local: memory.db (queries directas: 117 entidades, distribución de relation_type y categorías, search_index 137/1537/650, permalinks unicode), config.json (`ensure_frontmatter_on_sync`, `permalinks_include_project`), kbx `internal/index/schema.go` (canary `consumed`), `doctor.go:152-155,300` (orphan por existencia, root_file), `stale.go:143-144` (degree), `frontmatter/frontmatter.go` (Tier/Value/BudgetMax/OrphanOK), `basic-memory-recall.sh` (read-note + recent-activity permalinks-only), `retrieval-logger.sh` + `reflex-retrieval-log.jsonl` (265 read_note / 48 search / 4 recent / 1 build_context), `documenta.md` Paso 3 (productor de la gramática), consolida SKILL.md (falla-fuerte ante schema_drift), grep frontmatter KB (112/112 permalink, 101/112 type, 0 created), OKF SPEC.md v0.1 (fetch remoto).
