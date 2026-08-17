# Reporte — M6-01: índice fresco sin daemon

Campaña C6, primer item. Implementado por el ORQUESTADOR (no por un executor:
en C5 dos de tres subagentes dejaron de responder a mitad y hubo que rematar
igual; item pequeño, se declara para que conste).

Rama `m6-01`, base `3d7f073`.

## El problema que resuelve

basic-memory mantenía su índice al día con un **watch** en segundo plano. exo
indexa **al invocar**, sin daemon (spec §4.2: "incremental por mtime/git al
invocar, sin daemon salvo que duela"). Sin nada que refresque, el hook de
recall de M6 serviría un bloque de una KB rancia — y lo haría en silencio, que
es la peor forma de fallar.

## Lo implementado

- `exo::refresca_indice(kb, db)` — contrato nombrado y testeado aparte del CLI.
  Es `indexer::indexa` sin adornos; existe como función propia para poder
  probar el comportamiento sin pasar por el binario.
- `exo recall --refresca` — refresca antes de servir. El resumen va a
  **stderr** (stdout es exclusivo del envelope/bloque, contrato §4) y solo se
  imprime si de verdad cambió algo.
- 4 tests nuevos (`engine/tests/refresca.rs`), TDD estricto rojo→verde:
  - `recall_sin_refrescar_sirve_indice_rancio` — documenta el fallo que
    justifica el item: sin refresco, una nota añadida tras indexar NO aparece.
  - `refresca_indice_antes_de_servir_incluye_la_nota_nueva` — y el incremental
    de verdad lo es: `indexadas=1, saltadas=1`.
  - `refresca_sin_cambios_no_reindexa_nada`.
  - `refresca_crea_el_indice_si_no_existe` — bootstrap de máquina limpia.

Suite completa: **90/90 verdes** (86 previos + 4).

## Medición en vivo (KB real, 138 notas)

| Caso | Tiempo | Comentario |
|---|---|---|
| `recall` sin `--refresca` | 14 ms (p95, M2-08) | referencia |
| `--refresca`, **nada que reindexar** | **21-25 ms** | el caso normal del hook |
| `--refresca`, **1 nota modificada** | **3,4 s** | carga del modelo ONNX + embed de sus trozos |
| `--refresca`, **todos los mtimes nuevos** | **>10 min** | reindexado completo (138 notas) |

## Decisión de diseño que estos números imponen a M6-02 (cutover del hook)

**El hook de SessionStart NO debe llamar a `--refresca` a ciegas.** Con el
presupuesto de arranque en <100 ms, una sola nota editada desde la sesión
anterior lo multiplica por 34 (3,4 s), y el peor caso —mtimes nuevos, p. ej.
tras un `git clone` fresco de la KB— bloquearía el arranque más de diez
minutos. Eso no es una latencia: es un cuelgue.

Reparto correcto, y así entra en el brief de M6-02:

- **SessionStart** → `exo recall` **sin** `--refresca`: 14 ms, sirve lo que
  haya en el índice.
- **Stop** (cierre de sesión) → `exo index`: ahí 3,4 s no molestan a nadie, y
  es justo cuando la KB acaba de cambiar (`/documenta` escribe al cerrar).
- `--refresca` queda como **red de seguridad manual y bootstrap**, no como
  camino por defecto del hot path.

Ese reparto tiene además la propiedad de que el índice está fresco *antes* del
siguiente arranque, que es cuando hace falta, en vez de pagarlo en el momento
en que el usuario espera.

## Cómo se descubrió el peor caso (declarado, porque fue un error)

Al medir el caso "1 nota modificada" copié la KB con `cp -r`, que **no
preserva mtimes**: las 138 notas parecieron modificadas y el refresco entró en
un reindexado completo que se comió un timeout de 10 minutos. El error de
método dio el dato del peor caso, que es el que motiva la decisión de arriba.
La medición buena se repitió con `cp -a`.

## M6-01b — Cache de embeddings por contenido del trozo

Añadido tras la investigación que pidió Paul (dos agentes en paralelo:
literatura académica e ingeniería real, 2026-08-17). Ambos frentes señalaron
la misma pieza como "hazla pase lo que pase".

### Por qué, con el dato que lo motiva

La medición inicial decía "3,4 s de coste fijo por cargar el ONNX". Era
incompleto. Midiendo con una nota mínima:

| Nota tocada | Trozos | Tiempo |
|---|---|---|
| Mínima | 1 | 1,12 s |
| `core-index.md` | 9 | 3,40 s |

O sea: **~1 s de carga del runtime + ~0,25 s por trozo**. El embebido no era
ruido, era la mayor parte. Y se estaba re-embebiendo la nota ENTERA aunque
solo cambiara una línea.

### Qué se implementó

Al reindexar una nota se leen sus vectores previos ANTES de borrarlos,
indexados **por el texto del trozo**. Un trozo cuyo texto no cambió reutiliza
su embedding; solo los trozos nuevos pasan por el modelo. Si no hay ninguno
nuevo, el modelo ni se inicializa.

La clave es el contenido, no la posición: insertar un párrafo al principio
desplaza todos los `orden` pero conserva los textos, así que el cache sigue
acertando donde uno indexado por `(permalink, orden)` fallaría entero.

Sin tocar el schema: los textos ya estaban en `trozos` y los vectores se leen
de `vectores` por `rowid` (verificado: 3072 bytes = 768 × f32). El `Resumen`
gana dos campos aditivos (`trozos_embebidos`, `trozos_reusados`), que no
suben `SCHEMA_VERSION` por contrato del envelope.

### Efecto medido (KB real, 138 notas)

| Caso | Antes | Después | Contabilidad |
|---|---|---|---|
| Editar una línea de una nota de 9 trozos | 3,40 s | **1,50 s** | 2 embebidos / 8 reusados |
| Tocar solo el frontmatter (cuerpo intacto) | 3,40 s | **0,38 s** | **0 embebidos** / 9 reusados |

En el segundo caso el modelo no llega a cargarse.

3 tests nuevos: contabilidad con cuerpo intacto, contabilidad editando un solo
trozo, y que el vector reutilizado sea idéntico byte a byte (reutilizar no
puede corromper ni desplazar lo guardado). Suite: **93/93 verdes**.

### Procedencia de la decisión

- **LlamaIndex `IngestionPipeline`** guarda un hash por nodo en su docstore y
  solo re-procesa lo cambiado (`DocstoreStrategy.UPSERTS`) — es el patrón
  estándar del ecosistema RAG. LanceDB documenta lo mismo para contextual
  retrieval.
- La literatura confirma el patrón (Regmi & Pun, "GPT Semantic Cache",
  arXiv:2411.05276) pero advierte de que **no ataca el coste fijo de carga**:
  ahí sigue el ~1 s. Cierto, y por eso el reparto arranque/cierre de M6-02
  sigue siendo necesario.

### Lo que se DESCARTA, con razón escrita

- **Daemon o servidor de embeddings** (TEI, infinity, ollama con
  `KEEP_ALIVE=-1`): eliminan el cold-start, pero se amortizan con tráfico
  sostenido. Aquí son ~2 invocaciones por sesión: es la herramienta
  equivocada, y reintroduce justo la complejidad operativa que el diseño
  evita. Nota incómoda que conviene tener presente: **ningún sistema del
  estudio evita el daemon de verdad** — Smart Connections y obsidian-copilot
  re-embeben por nota en <1 s porque viven dentro de Obsidian, un proceso YA
  persistente; y basic-memory, al que sustituimos, usa `sync --watch`. No
  tenemos ese lujo, así que la salida es no necesitar frescura síncrona, no
  conseguirla más barata.
- **Warm-pool con idle-timeout** (patrón ServerlessLLM, OSDI 2024): es la
  única idea de la literatura que ataca el coste fijo sin daemon permanente.
  Se descarta para este volumen por la misma razón que el daemon.
- **mmap / page cache**: medido y refutado empíricamente. Tres cargas
  consecutivas del modelo dieron 3,39 / 3,52 / 3,47 s — la page cache no
  amortiza nada, porque el coste es inicializar el runtime ONNX, no leer el
  fichero.
- **Cuantizar el modelo** (fastembed-rs sirve variantes `Q`): mejora barata y
  ortogonal, ~1-2 s estimados en vez de 3,4 s en el peor caso. **No se hace
  ahora** — con el cache y el reparto arranque/cierre el dolor ya no está en
  el camino crítico. Queda anotado por si algún día hace falta embeber
  síncrono.

### El dato que cierra el diseño

La pregunta que la investigación decía que solo Paul podía responder —¿se
editan notas fuera de sesión de agente?— se contestó midiendo su repo:
**232 de 244 commits de los últimos 60 días (95%) llevan marca de Claude**.
La KB se escribe dentro de sesión, casi siempre por `/documenta`.

Por tanto el reparto arranque/cierre cubre el 95% de los casos, y el 5%
restante queda con una obsolescencia máxima de una sesión — con `--refresca`
disponible como red manual para quien no quiera esperar.

## M6-02 — Cutover del hook de recall (hecho, pendiente de instalar)

Autorizado por Paul en sesión ("dale con el cutover").

### La regresión que se evitó por comprobar antes de tocar

El hook `basic-memory-recall.sh` NO inyecta rutas: inyecta el **cuerpo** de
`core/core-index` (contrato de memoria + doctrina compacta + mapa de cores) más
un digest de actividad reciente. `exo recall` servía **rutas**. Un cutover
directo habría dejado al agente sin doctrina en todas las sesiones a cambio de
una lista de ficheros — y sin ningún síntoma visible.

De ahí salió `exo recall --contenido` (y `--nota`, tras descubrir que "vuelca
todos los cores" agota el presupuesto con el backlog de 20 KB y deja fuera
justo el core-index).

### Qué cambia

| | Antes | Ahora |
|---|---|---|
| SessionStart | `basic-memory-recall.sh --cached` | `exo-recall.sh` |
| Latencia del bloque | 0,03 s con cache caliente · **6,6 s** en fallo | **~10 ms** |
| Indexado | watch de basic-memory | `exo-index.sh` en el hook **Stop** |
| Cache con TTL + refresco en background | 90 líneas | **eliminado** |

El cache existía para tapar los 6,6 s del arranque del CLI de Python. Con
SQLite a 10 ms sobra, y con él se van sus modos de fallo: cache rancio,
refresco que muere con el process group, escritura a medias.

### Lo que se conserva deliberadamente

- `exit 0` siempre: el arranque no se rompe jamás.
- Fallback embebido con **evento greppable por razón**
  (`no-engine` / `no-index` / `empty` / `no-contract`). La degradación
  silenciosa de este canal ya mordió una vez (F3.1) y no se repite.
- Guard del contrato de memoria: un bloque que no contiene "Contrato de
  memoria" no es el core-index, y vale más el fallback conocido que un bloque
  plausible pero falso.
- Reafirmación de reflejos tras compactación.
- Seams por entorno (`EXO_BIN`, `EXO_INDEX`, `EXO_RECALL_NOTA`, `EXO_RECALL_CAP`):
  permiten probar sin tocar la instalación y, para otra persona, apuntar a SU
  KB sin editar el script.

El texto del FALLBACK se reescribió: mandaba al agente a basic-memory, un MCP
en retirada.

De regalo, un bug conocido arreglado: el pin post-compactación ahora acepta
`verify-before-done` y `verify-before-commit`. La discrepancia entre el id que
se loguea y el que se matcheaba lo tenía muerto (anotado en la foto as-is del
2026-08-02, sin arreglar hasta hoy).

### Verificación en vivo (hecha, con el índice y la KB reales)

- Bloque servido: **4531 bytes**, con el core-index íntegro y la actividad
  reciente por permalink. `exit 0`.
- `EXO_BIN=/no/existe` → fallback activado, y el texto ya no menciona
  basic-memory.
- `EXO_INDEX=/no/existe.db` → fallback activado.
- Ambos casos dejaron su evento `recall-fallback` en `reflex-log.jsonl`.
- `exo-index.sh` → indexa detached y escribe su envelope en
  `~/.claude/exo-index.log`; el cierre no espera.

### Estado de instalación

Instalado en la máquina: binario `exo` en `~/.local/bin/exo` e índice de
producción en `~/.exo/index.db` (138 notas, al día en 10 ms).

**El plugin NO está actualizado**: vive la 0.12.0; el cutover es la 0.13.0 en
la rama `m6-cutover-recall` de agent-develop, sin mergear ni pushear. Hasta
que Paul haga merge + push + `/plugin update`, el arranque sigue usando el
camino viejo. Rollback = reinstalar 0.12.0; el script antiguo sigue en el repo
intacto.

## Lo que NO entra en este item

El cutover en sí (M6-02..05: reapuntar el hook de recall, reescribir el
FALLBACK embebido, mover reflex al monorepo, repuntar kbx, cutover de
doctrina) **espera OK explícito de Paul**: cambia el entorno vivo de sus
sesiones, y el config §Línea roja nombra guards y settings como no delegables.
