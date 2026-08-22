# Consultor landscape — cómo resuelve esto el resto del mundo

Ángulo: estado del arte externo. Literatura de memoria de agentes, sistemas de
notas humanos con décadas de rodaje, compactación automática en software, y
productos reales de memoria para LLMs. Todas las fuentes citadas están
verificadas hoy (2026-08-22) contra arXiv/editores; ninguna es de memoria.

> **Nota de revisión a mitad de encargo**: el orquestador corrigió la premisa
> del contexto compartido — el presupuesto NO nació para proteger el coste del
> arranque (la spec fundacional lo descarta explícitamente: "el arranque es
> barato, no es el problema"); nació como **forzador de destilado editorial**.
> Las secciones 1-5 se escribieron antes de esa corrección y siguen siendo
> válidas como mapa del terreno; la **sección 6** responde al reenfoque:
> cómo se consigue destilado sin que un humano decida cada vez.

## TL;DR

1. **La arquitectura de la KB es el estado del arte, no un apaño.** Core
   inyectado con límite + resto por retrieval + consolidación offline con LLM
   es exactamente lo que hacen MemGPT/Letta, y `/consolida` es lo que la
   literatura llama *reflection* (Generative Agents, 2023) y *sleep-time
   compute* (Letta, 2025). Tiene nombre, tiene papers, tiene producto comercial
   que lo hace igual.
2. **Lo que NO tiene análogo externo es el presupuesto duro por nota en el
   tier `stable`**, enforced en pre-commit. Nadie limita el tamaño de una
   memoria individual del almacén recuperable; todo el mundo limita lo que se
   inyecta siempre. Y bloquear al escritor en el write path es el anti-patrón
   que los LSM-trees existen para evitar.
3. **El tamaño es síntoma, no problema** — y la doctrina de la KB ya lo sabe
   ("pártela, no la mutiles"). El fallo no es la regla, es el *momento* del
   enforcement: morder al cierre de sesión convierte al detector de mala
   atomicidad en un mutilador, que es justo lo que la doctrina prohíbe.

---

## 1. Literatura académica sobre memoria de agentes

### 1.1 La jerarquía: límite donde se inyecta, no donde se almacena

**MemGPT: Towards LLMs as Operating Systems** (Packer, Wooders, Lin, Fang,
Patil, Stoica, Gonzalez — arXiv:2310.08560, oct 2023). El paper fundacional de
la memoria jerárquica para agentes. Propone *virtual context management*
inspirado en la jerarquía de memoria de un SO: un **main context** de tamaño
fijo (lo que entra en la ventana, siempre) y un **external context** ilimitado
(recall storage + archival storage), con el propio LLM moviendo datos entre
niveles mediante function calls. Evidencia: mantiene coherencia en
conversaciones y análisis de documentos que exceden con mucho la ventana nativa.

Lo relevante para esta auditoría: **el presupuesto vive en el main context**
(está acotado por construcción), **el almacén externo no tiene límite por
ítem**. El equivalente de la KB: el cap de 6.144 chars del `SessionStart` es
main context y limitar lo que entra ahí tiene todo el sentido; las notas
`stable` que llegan por `exo search` son external context, y MemGPT no las
limitaría por tamaño individual. (Matiz tras la corrección del orquestador:
esta es la justificación *arquitectural* del límite en lo inyectado, no la
motivación histórica del presupuesto de la KB — que fue forzar destilado.)

### 1.2 La consolidación offline: /consolida tiene papers

**Generative Agents: Interactive Simulacra of Human Behavior** (Park et al.,
arXiv:2304.03442, UIST 2023). Memoria = stream append-only de observaciones +
un proceso de **reflection**: periódicamente (disparado por un umbral de
"importancia acumulada", no por calendario ni por tamaño) el agente sintetiza
insights de alto nivel a partir de observaciones crudas y los guarda como
memorias de primera clase. Es estructuralmente idéntico a "promover doctrina
repetida a core". Evidencia: ablations del paper — quitar reflection degrada
el comportamiento de los agentes de forma medible.

**Sleep-time Compute: Beyond Inference Scaling at Test-time** (Lin, Snell,
Wang, Packer, Wooders, Stoica, Gonzalez — arXiv:2504.13171, 2025; el equipo de
Letta). Precomputar sobre el contexto *antes* de que lleguen las queries:
~5x menos compute en test-time a igual accuracy, y hasta +13/+18% de accuracy
en sus benchmarks (Stateful GSM-Symbolic, Stateful AIME); el beneficio se
amortiza cuando hay varias queries sobre el mismo contexto. Es la validación
cuantitativa de la apuesta de `/consolida`: destilar offline compensa **cuando
las queries futuras son predecibles** — que es el caso de una KB personal
(las queries de mañana se parecen a las de hoy). El paper también muestra el
límite: si la query es impredecible o única, el precompute se desperdicia.

**Mem0: Building Production-Ready AI Agents with Scalable Long-Term Memory**
(Chhikara et al., arXiv:2504.19413, 2025). Pipeline de dos fases: extracción
de hechos + fase de *update* donde un LLM decide por tool call entre
ADD / UPDATE / DELETE / NOOP contra las memorias similares existentes.
Evidencia: en LOCOMO, +26% relativo sobre la memoria nativa de OpenAI, p95 de
latencia 91% menor que full-context, >90% menos tokens. El punto: **la
deduplicación y la poda las decide un LLM, no una regla de tamaño** — porque
en memoria semántica no existe regla sintáctica que decida qué sobra.

### 1.3 Qué se pierde al compactar: episódica vs semántica

**Position: Episodic Memory is the Missing Piece for Long-Term LLM Agents**
(Pink, Wu, Vo, Turek, Mu, Huth, Toneva — arXiv:2502.06975, 2025). Position
paper (argumentativo, no empírico — lo marco): defiende que los agentes
necesitan memoria episódica con cinco propiedades, entre ellas memorias
*instance-specific* y *contextuales* (quién/cuándo/por qué, ligado al
contenido). El argumento central aplicable aquí: **la semantización (destilar
a conocimiento general) pierde exactamente el detalle de instancia**, y por
eso hay que conservar episodios además de destilados. La estructura
canon + bitácora de la KB es literalmente esa recomendación: el canon es
memoria semántica, la bitácora es episódica, y rotar a `archive/` en vez de
borrar es la política conservadora que el paper pediría.

**MemoryBank: Enhancing LLMs with Long-Term Memory** (Zhong et al.,
arXiv:2305.10250, AAAI 2024). Olvido gradual por curva de Ebbinghaus: las
memorias accedidas se refuerzan, las ignoradas se desvanecen. Lo cito como
representante de toda la familia "decay automático": **exige tracking de
accesos y scoring por memoria**. Para 10⁶ memorias multiusuario, razonable;
para ~109 notas activas, es maquinaria pura (ver §5).

**A-MEM: Agentic Memory for LLM Agents** (Xu et al., arXiv:2502.12110,
NeurIPS 2025). Memoria explícitamente inspirada en el Zettelkasten: notas
atómicas con atributos estructurados y enlaces que un LLM crea y actualiza
dinámicamente (*memory evolution*: una nota nueva puede disparar la
actualización de notas viejas). Evidencia: mejora sobre baselines (incl.
MemGPT) en seis modelos en benchmarks de diálogo largo. Dato sociológico
relevante: cuando la investigación punta de 2025 busca estructura para la
memoria de agentes, converge en... notas atómicas enlazadas. La KB ya está ahí.

### 1.4 Por qué el tamaño de nota importa algo incluso con retrieval

Tres fuentes empíricas sobre el coste de contexto largo/ruidoso en el punto
de lectura:

- **Lost in the Middle** (Liu et al., arXiv:2307.03172, TACL 2024): la
  accuracy sobre información situada en medio de un contexto largo cae de
  forma pronunciada; los extremos se leen bien, el medio mal.
- **LLMs Can Be Easily Distracted by Irrelevant Context** (Shi et al.,
  arXiv:2302.00093, ICML 2023): añadir información irrelevante al problema
  degrada el rendimiento drásticamente (benchmark GSM-IC).
- **Context Rot** (Hong, Troynikov, Huber — Chroma, technical report, jul
  2025; no peer-reviewed pero con toolkit replicable): 18 modelos frontier,
  todos degradan al crecer el input, de forma no uniforme; los distractores
  semánticamente parecidos son lo que más daño hace.

**Pero las magnitudes importan**: la degradación seria aparece a decenas de
miles de tokens. Una nota `stable` al presupuesto (12.500 B) son ~3-4K tokens;
la peor nota del ratchet (30.000 B) son ~8K. Un agente que abre 1-3 notas está
lejos de la zona roja de estos estudios. La justificación "coste de leer una
nota gorda" **existe pero es débil a estas escalas**. La justificación fuerte
es otra (opinión mía, marcada): **la unidad de retrieval es la nota**, y una
nota-cajón que mezcla temas empareja mal tanto en FTS como en embeddings — el
tamaño daña el *recall* antes que la lectura. Eso apunta a atomicidad, no a
bytes.

## 2. Sistemas de notas humanos con décadas de rodaje

### 2.1 Luhmann (el de verdad, vía Schmidt)

Fuente: **Johannes F.K. Schmidt, "Niklas Luhmann's Card Index: The Fabrication
of Serendipity"** (Sociologica 12(1), 2018). Schmidt coordina el proyecto de
investigación a largo plazo sobre el archivo real de Luhmann — es LA fuente,
no un blog.

Datos: ~90.000 fichas manuscritas en formato **A6 fijo**, dos colecciones
(1951-1962 y 1963-1996), numeración no jerárquica con ramificación
(*Folgezettel*), índice de palabras clave.

Lo que responde a esta auditoría:

1. **El límite de tamaño existía y era brutal: una cara de una ficha A6.**
   Pero no era una política, era el soporte físico. Y la respuesta al límite
   nunca fue recortar: era **ramificar** — ficha nueva numerada como
   continuación/derivación. El límite de tamaño funcionaba como **forzador
   mecánico de atomicidad**, no como control de volumen.
2. **Luhmann jamás podó.** El archivo es append-only: 90.000 fichas en 45
   años, cero consolidación, cero borrado. Le funcionó porque su modo de
   acceso era navegación por enlaces + índice, donde el volumen total no
   cuesta nada por consulta. Moraleja directa para una KB con retrieval:
   **el crecimiento del corpus no es el problema; el problema sería el
   crecimiento de la unidad de consulta.** Luhmann tenía la unidad acotada
   por el cartón.
3. Schmidt documenta que Luhmann veía la dispersión física de temas como un
   *positivo* (serendipia). El equivalente moderno: no hay que tener la KB
   "ordenada" para que el retrieval funcione.

### 2.2 Evergreen notes (Matuschak) y PARA (Forte)

**Andy Matuschak, "Evergreen notes"** (notas públicas en notes.andymatuschak.org,
~2019-presente; no es paper, es praxis documentada). Sus principios: notas
**atómicas** ("about one thing"), **orientadas a concepto** (no a fuente ni a
proyecto), **densamente enlazadas**, y — a diferencia de Luhmann — *escritas
para ser reescritas*: la nota evoluciona hacia una versión mejor del concepto.
Su regla ante la nota que crece no es un límite: es que **una nota que crece
está intentando ser dos conceptos**, y se parte por la línea conceptual. El
tamaño es diagnóstico, nunca el criterio de corte.

**PARA / Building a Second Brain (Tiago Forte, libro 2022).** No dice nada
sobre tamaño de nota. Su mecanismo anti-crecimiento es **tiering por
accionabilidad**: Projects → Areas → Resources → **Archives**, con la regla de
mover a Archive al cerrar el proyecto, no de podar contenido. Es exactamente
lo que la KB ya hace con `rotate` a `archive/` (534 KB de cola fría rotados,
59 notas archivadas). El dato del recon "93 notas log" incluye esas 59
archivadas, por cierto: el corpus activo es ~109 notas, y el tiering
frío/caliente **ya existe y ya funciona**.

Síntesis de los tres sistemas: **ninguno trata el tamaño como problema en sí.
Dos lo usan como síntoma de mala atomicidad (Luhmann por hardware, Matuschak
por doctrina) y el tercero ni lo mira.** La doctrina de la KB ("si no cabe,
pártela, no subas el techo ni la mutiles") es exactamente la regla
Luhmann/Matuschak. Lo que ninguno de ellos tiene es un pre-commit que te
bloquea el guardado si te pasas de bytes.

## 3. Software que compacta sin humanos: el criterio es lo interesante

- **LSM-trees** (RocksDB, Cassandra...): las escrituras van a un memtable y
  se vuelcan a L0 **sin bloquear jamás al escritor**; la compactación corre
  después, en background, con criterio formal: para claves duplicadas gana la
  versión más reciente, los tombstones eliminan. El *write stall* (bloquear
  escrituras porque la compactación no da abasto) es el modo de fallo más
  temido y todo el diseño existe para evitarlo.
- **Kafka log compaction**: retiene el último valor por clave. Criterio: la
  clave. Formal, decidible, cero juicio.
- **GC generacional**: recolecta por **alcanzabilidad** (formal) apoyándose
  en la hipótesis generacional (la mayoría de objetos mueren jóvenes) para
  decidir *cuándo* mirar dónde.
- **Caches (LRU/LFU/ARC)**: expulsan por patrón de acceso, y pueden
  permitírselo porque **hay backing store** — perder una entrada cuesta una
  recarga, no una pérdida.

El patrón común, y la respuesta a "¿por qué no exige humano?": **todos tienen
un criterio de equivalencia u obsolescencia formal** (misma clave, no
alcanzable, no accedido). Una KB semántica no lo tiene: "estas dos notas
dicen lo mismo" o "esto ya no es verdad" es un juicio semántico. Por eso la
literatura de 2024-2025 (§1.2) converge en que **el compactador de memoria
semántica es un LLM corriendo offline** — Mem0 en el write path, Generative
Agents por umbral, Letta en sleep-time. `/consolida` es exactamente eso. No
hay una técnica de sistemas que Paul se esté perdiendo: la técnica es la que
ya tiene.

Lo que sí se está saltando el diseño actual es la **otra** mitad del patrón
LSM: *el escritor nunca espera a la compactación*. El pre-commit que rechaza
un `/documenta` porque `doctrina-agentes` está a 33 B del techo es un write
stall de manual: el coste de la compactación pagado en el peor momento
(cierre de sesión, contenido en la mano, Paul queriendo irse) por el actor
equivocado (el escritor, no el compactador).

## 4. Productos de memoria para LLMs

- **Letta** (el MemGPT comercial; docs.letta.com). Memory blocks de core
  memory con campo **`limit` en caracteres** (default 2.000, ajustable por
  bloque) porque van pinned a la ventana en cada turno; archival memory
  **sin límite**, accesible por búsqueda. Su doctrina documentada ante un
  bloque que no cabe: subir el límite o **mover el detalle a archival y
  dejar el resumen en core**. Es la validación de producto más directa del
  diseño de la KB: presupuesto en lo inyectado, libertad en lo recuperable,
  y "split canon/bitácora" como respuesta al desborde. Además Letta vende
  *sleep-time agents* que reorganizan la memoria offline — `/consolida`
  como producto.
- **Mem0** (producto sobre el paper de §1.2): la poda es automática vía LLM
  (ADD/UPDATE/DELETE/NOOP); el usuario no poda a mano.
- **ChatGPT Memory** (OpenAI): extracción automática de memorias, gestión
  automática de qué se retiene; el usuario puede ver y borrar pero no está
  *obligado* a podar. Cuando la memoria se llena, el sistema pide limpiar —
  su versión del presupuesto también muerde al usuario, y es una de las
  quejas de UX más comunes del feature. (Esto último es apreciación mía de
  usuario, no tengo fuente medible.)

Patrón de mercado: **nadie le pide al usuario disciplina de poda**. El que la
pide (ChatGPT al llenarse) genera exactamente la queja que Paul tiene hoy.

## 5. Síntesis: qué es aplicable y qué es over-engineering

### Aplicable (y barato)

1. **Mantener el presupuesto donde hay coste fijo por sesión**: `core` y
   cualquier cosa que un hook inyecte incondicionalmente. Validado por
   MemGPT (main context), Letta (`limit` por block) y por la aritmética:
   eso se paga en cada arranque. Aquí el sistema actual está bien y el
   mundo lo hace igual.
2. **Reencuadrar el techo de `stable` como detector de atomicidad, no como
   muro.** Es lo que el tamaño significa en Luhmann y Matuschak, y la
   doctrina de la KB ya lo dice. Consecuencia operativa mínima (opinión mía,
   trade-off explícito): el pre-commit **avisa y registra** (la nota queda
   marcada como deuda de split), y quien *exige* es `/consolida` — el
   compactador, offline, con calma y con LLM, como en todos los sistemas de
   §1.2 y §3. Es mover un check de sitio, no construir nada. Trade-off: la
   deuda puede acumularse entre consolidaciones; el trinquete existente
   sirve tal cual de registro de esa deuda (los techos sellados ya son eso:
   deuda reconocida con ancla).
3. **Conservar canon + bitácora + archive.** Es la recomendación del position
   paper de memoria episódica (semántica + episódica, destilar sin destruir)
   y el tiering de PARA. La rotación a archive ya funciona (59 notas, 534 KB).
4. **`/consolida` no es el apaño — es la solución con literatura.** Sleep-time
   compute demuestra que destilar offline compensa cuando las queries futuras
   son predecibles (una KB personal lo es). Si algo, la conclusión externa es
   que el apaño es hacer que el *escritor* haga el trabajo del compactador.

### Over-engineering para 109 notas activas / 4,3 MB

- **Decay automático / curvas de olvido** (MemoryBank): exige tracking de
  accesos y scoring. Para millones de memorias multiusuario, sí; aquí es un
  subsistema nuevo para un corpus que cabe entero (~1,1 M tokens) en la
  ventana de un modelo grande. Además viola la restricción "nada de métricas
  nuevas".
- **Grafos de memoria / memory evolution automática** (A-MEM, HippoRAG y
  familia): resultados reales en benchmarks, pero atacan el problema de
  *recall a escala*, que esta KB no tiene. Los wikilinks manuales + retrieval
  híbrido ya dan la estructura que A-MEM construye con maquinaria.
- **Consolidación programada/automática** (cron de `/consolida`, umbrales de
  disparo tipo Generative Agents): tentador y con pedigrí académico, pero es
  un subsistema nuevo con modos de fallo nuevos, para ahorrar la decisión
  "toca consolidar" que hoy cuesta un comando. Si algún día duele, el umbral
  de importancia de Park et al. es el diseño a copiar; hoy no duele eso.
- **Afinar los números de los presupuestos** (¿12.500 o 15.000?): la
  literatura no da soporte para ningún número concreto a estas escalas
  (§1.4: la zona de degradación empieza un orden de magnitud más arriba).
  Discutir el número es discutir la parte que menos importa.

### La frase que resume el estado del arte

Todos los sistemas maduros — Luhmann, LSM, Letta — coinciden en tres cosas:
**el límite duro vive donde el coste es por-consulta fija** (la ficha A6, la
ventana de contexto, el memory block), **el almacén crece libre** (90.000
fichas, archival ilimitado), y **la compactación es trabajo del compactador,
offline, nunca del escritor en el momento de escribir**. La KB de Paul cumple
las dos primeras y viola la tercera exactamente en los dos puntos que hoy le
mordieron.

## 6. Reenfoque: destilado editorial sin humano en el bucle

El problema reformulado por el orquestador: el presupuesto es un forzador de
destilado disfrazado de límite de bytes. La pregunta pasa a ser: **¿cómo se
decide qué merece quedarse en la versión canónica y qué pasa a histórico, sin
que un humano delibere cada vez?** Tres respuestas del exterior, una por
pregunta del orquestador.

### 6.1 Atomicidad: ¿criterio operativo o solo principio?

Hay criterio operativo en los dos sistemas, y es el mismo con distinto
soporte:

- **Luhmann**: el criterio era el cartón. Una cara de una ficha A6; si no
  cabe, ficha nueva ramificada (*Folgezettel*). Cero deliberación — el
  hardware era el linter (Schmidt, Sociologica 2018).
- **Matuschak**: el test es el título. Sus notas "Evergreen notes should be
  atomic", "should be concept-oriented" y sobre todo **"Evergreen note titles
  are like APIs"** (notes.andymatuschak.org, verificadas): una nota bien
  factorizada admite un título que funciona como *handle* — una afirmación
  completa que cubre todo su contenido y se puede usar como referencia en
  otra frase. **Test aplicable sin deliberar: si el título honesto de la nota
  necesita un "y", o es un sustantivo-área en vez de una afirmación, la nota
  mezcla conceptos.** Ese test lo puede aplicar un LLM en `/consolida` sin
  juicio humano: titular cada bloque de la nota; si emergen dos o más
  títulos-afirmación independientes, hay split.

Diagnóstico directo sobre la KB (verificable en el ratchet): las notas que
viven de waiver se llaman `doctrina-agentes`, `desarrollo-agentico`,
`Backlog — frentes abiertos`. Son **notas-área, no notas-concepto**: su
título admite cualquier contenido futuro, así que crecen sin converger jamás.
Una nota cuyo título es una afirmación converge (cuando la afirmación está
bien dicha, la nota está terminada); una nota cuyo título es un área crece
para siempre. **El presupuesto lleva meses detectando esto correctamente —
notas-área que acumulan — pero el remedio aplicado (recortar/rotar para caber)
ataca el síntoma. El remedio de Matuschak/Luhmann es refactorizar por
concepto.** Eso explica el 22% de waivers: no son notas gordas, son notas con
el título mal factorizado.

Respaldo empírico del formato-afirmación: **LoCoMo** (Maharana et al.,
arXiv:2402.17753, ACL 2024 — benchmark de conversaciones de 300+ turnos)
encuentra que RAG rinde notablemente mejor cuando el historial se transforma
en una **base de afirmaciones atómicas** (observations) sobre cada hablante
que cuando se recupera diálogo crudo o resúmenes de sesión — y que incluso
así los modelos siguen fallando en razonamiento temporal/causal de largo
alcance. El destilado a afirmaciones no es estética: mejora el retrieval
medido.

### 6.2 El criterio de supervivencia: "última versión de cada clave gana", versión notas

Lo que hace posible que Kafka/LSM compacten sin preguntar a nadie no es el
mecanismo, es que **cada registro declara su clave**. Con clave + regla de
orden (más reciente gana), la compactación es clasificación, no deliberación.
GC generacional: la clave es la referencia, la regla es alcanzabilidad.
Caches: la clave es la entrada, la regla es recencia de acceso. En todos:
criterio = clave + orden total.

**El análogo para notas existe y está en producción: Zep/Graphiti** (*Zep: A
Temporal Knowledge Graph Architecture for Agent Memory*, arXiv:2501.13956,
2025; Graphiti es open source, ~20K stars). Cada hecho es una arista con
clave semántica (sujeto-relación-objeto) y modelo **bi-temporal** (cuándo fue
verdad en el mundo, cuándo entró/salió del sistema). Cuando llega un hecho
que contradice a uno existente, el viejo recibe `invalid_at` — **se invalida,
no se borra**, y queda recuperable como historia. Es exactamente "última
versión de cada clave gana" donde la clave es la afirmación, y el matching de
claves (¿este hecho nuevo habla de lo mismo que aquel?) lo hace un LLM en
ingestión. Mem0 (arXiv:2504.19413) es la misma idea con menú cerrado:
ADD/UPDATE/DELETE/NOOP decidido por LLM contra las memorias similares.

Traducción operativa a la KB, sin subsistema nuevo:

1. **La clave de una nota atómica es su título-afirmación** (§6.1). La clave
   de una entrada de bitácora es la afirmación que contiene.
2. **Regla de supervivencia enunciable sin juicio**: para cada afirmación, la
   versión más reciente vive en el canon; las anteriores y sus episodios
   pasan a bitácora/archive. Nadie decide "qué merece quedarse": **todo se
   queda, solo cambia de tier**. La única operación con juicio es el matching
   de afirmaciones, y eso es exactamente lo que Mem0/Zep delegan en un LLM
   con menú cerrado — clasificación, no deliberación.
3. **El gatillo también es formal**: en LSM la compactación de un nivel se
   dispara cuando el nivel excede su presupuesto de bytes. El presupuesto por
   nota YA es ese trigger. La diferencia entre LSM y la KB actual es qué pasa
   al dispararse: en LSM se encola una compactación asíncrona; en la KB se
   bloquea al escritor. El número está bien; el efecto del número es lo que
   no tiene análogo sano.

### 6.3 Qué se pierde al compactar, y el archivo que sigue apareciendo

**Sobre la pérdida** — tres resultados convergentes:

- **LoCoMo**: los resúmenes narrativos de sesión pierden detalle
  temporal/causal frente a la base de afirmaciones; el razonamiento temporal
  es lo primero que se rompe al comprimir.
- **Position paper de memoria episódica** (arXiv:2502.06975): la
  semantización pierde por construcción el detalle *instance-specific*
  (quién/cuándo/por qué); su recomendación es conservar episodios además de
  destilados.
- **Sleep-time compute** (arXiv:2504.13171): el destilado offline es una
  apuesta sobre qué se preguntará; cuando la query real no era predecible,
  el precompute se desperdicia — conservar el crudo permite recomputar.

Conclusión común: **compactar con red de seguridad = destilar sin destruir**.
La KB ya cumple (canon + bitácora + archive + git). Aquí no hay nada que
importar: el diseño local ya es la recomendación de la literatura.

**Sobre el archivo recuperable** — el dato nuevo del orquestador (40% del
índice de búsqueda son notas archivadas y siguen apareciendo en resultados)
tiene análogo directo, y el análogo revela el fallo:

- **Elasticsearch ILM** (docs de Elastic, data tiers): hot → warm → cold →
  frozen, transición por edad (criterio formal, sin humano), y **todo sigue
  siendo buscable menos lo borrado** — pero las tiers frías se consultan con
  menor prioridad y se pueden excluir por defecto de las búsquedas normales.
  Archivado-pero-buscable es un patrón industrial estándar; archivado
  compitiendo *de igual a igual* con lo vigente, no.
- **Zep**: los hechos invalidados se conservan y son recuperables, pero **no
  se sirven como hechos vigentes** — se sirven como historia cuando se
  pregunta por historia. La distinción vigente/histórico viaja con el hecho.
- **Generative Agents** (arXiv:2304.03442): el score de retrieval multiplica
  relevancia por un factor de **recencia con decay exponencial** — lo viejo
  no desaparece, pierde puntos.
- Y el coste de no hacerlo está medido: **Context Rot** (Chroma, 2025)
  identifica los *distractores semánticamente similares* como el daño mayor
  en contexto — y una versión archivada de la misma doctrina es el distractor
  semánticamente similar perfecto contra su versión canónica.

Fix mínimo coherente con todos los análogos: **el retrieval por defecto
excluye o penaliza `archive/`, con opt-in explícito para búsqueda histórica**.
Es un filtro o un boost en la query que ya existe — un WHERE, no un
subsistema. De todo lo que he mirado fuera, este es el hallazgo más barato
con respaldo más unánime.

### 6.4 El paquete completo, sin maquinaria nueva

El destilado sin humano, tal como lo hace el estado del arte, se compone de
cuatro piezas — y las cuatro caben en doctrina para el LLM que ya ejecuta
`/consolida` más un filtro de query:

| Pieza | Quién la hace fuera | Traducción local |
|---|---|---|
| Clave por unidad | Kafka (key), Zep (afirmación) | Título-afirmación (test API de Matuschak, aplicado por LLM) |
| Regla de supervivencia | LWW por clave; invalidar-no-borrar | Última versión al canon; lo anterior cambia de tier, nada se borra |
| Gatillo | Presupuesto de bytes por nivel (LSM) | El presupuesto actual, disparando compactación asíncrona en vez de bloquear el write |
| Ejecutor | LLM con menú cerrado (Mem0), offline (sleep-time) | `/consolida`, que ya existe |

Lo único genuinamente nuevo que sugiere el exterior: el filtro de archive en
retrieval (§6.3) y el test de título-afirmación como criterio de split
(§6.1). Ambos son cambios de doctrina/query, no subsistemas. Todo lo demás
que ofrece la literatura (decay, grafos, scoring de acceso) sigue siendo
over-engineering para este tamaño, como se argumentó en §5.

## 6.5 Verificaciones de ronda de síntesis: ¿el precedente permite bloquear?

El coordinador de la síntesis pidió precisión sobre dos precedentes que se
estaban usando en el arbitraje warning-vs-bloqueo. Ambos verificados contra
fuente primaria; ambos matizan lo que yo mismo había escrito en §3.

### RocksDB: el precedente completo son DOS umbrales, y el stop duro existe

Verificado contra la wiki oficial (facebook/rocksdb, "Write Stalls") y los
defaults declarados en `include/rocksdb/advanced_options.h` (rama main):

- Escalera completa: compactación en background → **slowdown** (las
  escrituras se ralentizan a `delayed_write_rate`) al llegar a
  `level0_slowdown_writes_trigger` = **20 ficheros** o
  `soft_pending_compaction_bytes_limit` = **64 GB** → **stop total** a
  `level0_stop_writes_trigger` = **36 ficheros** o
  `hard_pending_compaction_bytes_limit` = **256 GB**.
- O sea: mi §3 contaba media historia. El estado del arte NO es "el escritor
  nunca se entera": es **banda de degradación suave en el umbral nominal, y
  stop duro como última línea — a 1,8x (ficheros) o 4x (bytes) del umbral
  suave**. Saltarse la banda suave y parar en seco lo trata el propio
  proyecto como mal comportamiento (issue #9423: "too many write stalls
  because the write slowdown stage is frequently skipped").
- **Pero la semántica del stop importa tanto como su existencia**: cuando
  RocksDB para escrituras, el escritor *espera a que el compactador alcance*.
  Jamás se le pide que modifique su payload para caber. Un stop que fuerza a
  recortar el dato no existe en ningún sistema de almacenamiento — eso no es
  backpressure, es pérdida.

Traducción honesta: el precedente da para "aviso/deuda en el presupuesto
nominal + bloqueo solo en desbordamiento grosero (~2-4x)", no para "el
pre-commit nunca bloquea". Y el bloqueo legítimo significa "corre
`/consolida` antes de seguir", nunca "recorta el contenido". El sistema
actual hace las dos cosas que el precedente excluye: para en el umbral
nominal (a 33 B del techo) y la salida practicada hoy fue recortar.

### Letta: sí bloquea en write path — con tres acotaciones

Verificado (docs.letta.com y github.com/letta-ai/letta, issue #7): cuando un
agente escribe en un core memory block por encima de su `limit`, recibe un
**error inmediato y visible en el momento de escribir**:
`ValueError: Edit failed: Exceeds 2000 character limit (requested 7194)`.
Es un bloqueo en write path real — precedente utilizable por la defensa del
bloqueo. Con tres acotaciones que fijan su alcance:

1. **Solo en core memory** (bloques pinned al contexto en cada turno) — el
   equivalente del tier `core` de la KB. Archival memory no tiene límite por
   ítem: Letta no valida bloquear el equivalente de `stable`.
2. **El bloqueado es el propio LLM, dentro del mismo turno**, con
   herramientas para autorrepararse en el acto (mover a archival, resumir);
   el coste del stall son segundos de compute. El precedente traslada solo
   si quien recibe el error es el agente de `/documenta` con margen para
   reorganizar — no si el coste cae en el humano al cierre.
3. **La salida prescrita nunca es recortar**: la guía de Letta ante el error
   es resumir o mover a archival (o subir el `limit` deliberadamente). Es la
   doctrina "pártela, no la mutiles" — el precedente valida el error, no la
   mutilación.

(Nota: `core_memory_append/replace` figuran como funciones legacy frente a
`memory_insert/replace/rethink`; no he verificado si las nuevas cambian el
comportamiento del límite, así que no lo afirmo.)

### Veredicto para el arbitraje

Ninguna de las dos posturas hereda el precedente entero. Lo que el exterior
dibuja: **(a)** en el presupuesto nominal, aviso/registro de deuda — nunca
stop; **(b)** stop duro legítimo solo en desbordamiento grosero (~2-4x del
nominal), y con semántica "compacta antes de seguir", jamás "recorta para
caber"; **(c)** bloqueo en el nominal defendible únicamente en el tier
inyectado (core) y únicamente si quien lo recibe es un agente capaz de
reorganizar en el acto. El evento de hoy (recorte por 33 B en el umbral
nominal) viola (a) y la semántica de (b) a la vez.

Sobre la propuesta convergente "warning en banda + stop duro en techo+15%":
el precedente valida la **estructura** (banda suave, después stop), no el
número — los ratios de RocksDB (1,8x-4x) son dinámica de un storage engine y
no se trasladan numéricamente a una KB. +15% es una elección local legítima.
La condición que sí hereda del precedente y no es negociable: **la salida
prescrita del stop es partir/mover/consolidar — nunca recortar.** Con esa
semántica, ratifico la propuesta como consistente con el estado del arte.

## 6.6 El test del título aplicado a las 11 notas del ratchet

El coordinador pidió adjudicación operativa: qué notas fallan el test de
§6.1, y qué título/partición les correspondería. Aplicado sobre la
estructura real (título + headings) de las 11 notas con techo sellado.
Resultado honesto por delante: **el test no condena a las 11 — separa
limpiamente, y las dos que fallan son exactamente las dos que mordieron
hoy.** Mi formulación original ("las notas del ratchet son notas-área") era
demasiado gruesa; esta adjudicación la sustituye.

| Nota | Techo | Veredicto | Razón |
|---|---|---|---|
| `core/doctrina-agentes` | 20k | **FALLA** | Título-área declarada ("fuente única de la doctrina de agentes": cualquier doctrina futura cabe). Sus 10 headings son títulos-API perfectos de conceptos independientes (Recon-first, Cost pyramid, Orquestador limpio, Régimen de gates…). Es un volumen de ~10 notas-concepto encuadernadas — y su sección "Capítulos que viven en nota propia" demuestra que el split por concepto ya empezó y funciona. |
| `learnings/desarrollo-agentico` | 19k | **FALLA** | "La meta-habilidad" — área abierta total. Mezcla método (SDD), hechos técnicos del harness de Claude Code, y epistemología de benchmarks. Además **solapa territorio con doctrina-agentes** (delegación de gates, disciplina epistémica aparecen en ambas) — por eso el aprendizaje transversal de hoy no cabía en ninguna: las dos lo reclamaban. |
| `Backlog — frentes abiertos` | 30k | **Falla de formato, no de título** | Es un dashboard operativo (14 frentes), no una nota de conocimiento — el análogo de la "projects list" que Matuschak mantiene separada de las evergreen. Su presión de tamaño viene de retener detalle que ya vive en `projects/*`. Remedio: frente = 3-5 líneas de estado + puntero, no split por concepto. |
| `learnings/evidencia-y-divulgacion` | 10k | Falla latente, sin acción | El título lleva el "y" delator (dos conceptos declarados), pero a 8,9 KB con dos secciones limpias no hay patología. El test es tripwire: mientras no dispare, no se toca. |
| `Paul - perfil de trabajo` | 18k | Pasa | Nota-entidad (la persona), estructura coherente, crece por refinamiento y no por acumulación. Es el análogo del bloque "human/persona" de Letta — singular y acotada por diseño. |
| `learnings/pragmatismo-y-pivots` | 15k | Pasa | Concepto cohesionado (cómo Paul decide y descarta) con casos destilados. Coincido con el diagnóstico. |
| `projects/*` (5 notas) | 14-19k | Pasan | Notas-entidad acotadas por **mortalidad**: convergen cuando el proyecto cierra (lighthouses, "entrega sellada", ya convergió de facto). No son evergreen en el sentido de Matuschak — él separa lo project-oriented — pero en una KB de trabajo son unidad legítima. Señal de vigilancia: sus secciones "Patrones reutilizables"/"Aprendizajes" son conceptos transversales atrapados en la nota-proyecto; el remedio existente (promoción a `learnings/`) es el correcto, y una nota-proyecto que crece tras el cierre del proyecto sí es patología. |

Lo que valida el test: **las 9 que pasan viven sin morder; las 2 que fallan
son las 2 que mordieron hoy** (a 33 y 49 B de sus techos). La correlación
dolor↔fallo-del-test es exactamente lo que se le pide a un criterio
operativo. La discrepancia real con el dictamen del diagnóstico queda
reducida a una sola nota — `doctrina-agentes` — donde mantengo FALLA con
evidencia estructural: headings que ya son títulos de nota, y un historial
de split parcial que bajó su techo de 27k a 20k y funcionó.

**Respuesta a la objeción §0 (esto pide disciplina editorial de Paul):** no
en su versión mínima. El patrón externo aplicable es el *lazy splitting* —
análogo de la compactación lazy de LSM: nada se refactoriza por campaña; el
test del título lo aplica **el LLM de `/consolida` únicamente cuando una
nota dispara el tripwire**, proponiendo la partición (que en las dos que
fallan ya está escrita: sus propios headings). Paul aprueba o rechaza el
split propuesto — la misma aprobación que hoy ya da al recorte, pero con el
remedio que la doctrina prescribe. Cero disciplina nueva: trigger existente
(presupuesto), ejecutor existente (`/consolida`), solo cambia el remedio
prescrito al disparar.

## 6.7 Voto de ratificación (propuesta conjunta v1)

**Fase 1: SÍ. Fase 2: SÍ. Recomendación separada: SÍ.** Razonamiento:

- Mi punto (a) del veredicto de precedente pedía "aviso en el nominal, stop
  solo en desbordamiento"; la Fase 1 mantiene el stop en el techo pero
  garantiza por construcción ≥15% de aire al sellar — una banda implícita en
  el sellado en vez de en el gate. No es mi semántica exacta, y lo voto SÍ
  igualmente: la diferencia queda cubierta por la Fase 2 con criterio
  falsable y observable en commits. Si la guarda de sellado basta, mi banda
  codificada era over-engineering (YAGNI aplicado a mi propia
  recomendación); si no basta, la banda gana su código con evidencia. Es la
  resolución correcta de la discrepancia.
- Mi condición no negociable — la salida del mordisco es partir/rotar/
  consolidar, nunca recortar el delta — está escrita en el punto 6 con los
  precedentes citados (RocksDB #9423, Letta). Satisfecha.
- El punto 3 es mi tesis en versión mínima (lazy splitting) y **me basta**,
  con una observación operativa: el test del título debe correr también en
  la primera aplicación (punto 7) si la salida elegida para
  `doctrina-agentes` y `desarrollo-agentico` es el split — son los dos
  especímenes donde el test ya tiene veredicto (§6.6) y la partición ya está
  escrita en sus headings. No hace falta campaña: ambas están a <50 B de
  morder y el tripwire llegará solo.
- Punto 4 (Backlog como dashboard, morder = cerrar frentes) coincide con mi
  adjudicación de §6.6 (falla de formato, no de título).
- Separada de archive/: **confirmo el reencuadre** — el argumento que
  sostiene la evidencia es "gasta plazas escasas" (33% de punteros de hoy a
  material frío), no "suplanta al canon" (el vivo gana en score en pares
  directos). Context Rot queda como riesgo teórico secundario. Ambos
  argumentos prescriben el mismo fix.

## Fuentes

Papers (verificados en arXiv/editor hoy):
- Packer et al., *MemGPT: Towards LLMs as Operating Systems*, arXiv:2310.08560 (2023). https://arxiv.org/abs/2310.08560
- Park et al., *Generative Agents: Interactive Simulacra of Human Behavior*, arXiv:2304.03442 (UIST 2023). https://arxiv.org/abs/2304.03442
- Lin et al., *Sleep-time Compute: Beyond Inference Scaling at Test-time*, arXiv:2504.13171 (2025). https://arxiv.org/abs/2504.13171
- Chhikara et al., *Mem0: Building Production-Ready AI Agents with Scalable Long-Term Memory*, arXiv:2504.19413 (2025). https://arxiv.org/abs/2504.19413
- Pink et al., *Position: Episodic Memory is the Missing Piece for Long-Term LLM Agents*, arXiv:2502.06975 (2025). https://arxiv.org/abs/2502.06975
- Xu et al., *A-MEM: Agentic Memory for LLM Agents*, arXiv:2502.12110 (NeurIPS 2025). https://arxiv.org/abs/2502.12110
- Zhong et al., *MemoryBank: Enhancing LLMs with Long-Term Memory*, arXiv:2305.10250 (AAAI 2024). https://arxiv.org/abs/2305.10250
- Liu et al., *Lost in the Middle: How Language Models Use Long Contexts*, arXiv:2307.03172 (TACL 2024). https://arxiv.org/abs/2307.03172
- Shi et al., *Large Language Models Can Be Easily Distracted by Irrelevant Context*, arXiv:2302.00093 (ICML 2023). https://arxiv.org/abs/2302.00093
- Maharana et al., *Evaluating Very Long-Term Conversational Memory of LLM Agents* (LoCoMo), arXiv:2402.17753 (ACL 2024). https://arxiv.org/abs/2402.17753
- Rasmussen et al. (Zep), *Zep: A Temporal Knowledge Graph Architecture for Agent Memory*, arXiv:2501.13956 (2025). https://arxiv.org/abs/2501.13956

No-paper pero primarias:
- Schmidt, J.F.K., *Niklas Luhmann's Card Index: Thinking Tool, Communication Partner, Publication Machine* / *The Fabrication of Serendipity*, Sociologica 12(1), 2018.
- Hong, Troynikov, Huber, *Context Rot* (Chroma technical report, jul 2025, con toolkit replicable). https://www.trychroma.com/research/context-rot
- Letta docs, *Memory blocks (core memory)*. https://docs.letta.com/guides/core-concepts/memory/memory-blocks
- Matuschak, A., *Evergreen notes*; en particular *Evergreen notes should be atomic*, *should be concept-oriented* y *Evergreen note titles are like APIs* (notes.andymatuschak.org, verificadas hoy).
- Elastic docs, *Elasticsearch data tiers: hot, warm, cold, and frozen* e *Index lifecycle management*. https://www.elastic.co/docs/manage-data/lifecycle/data-tiers
- RocksDB wiki, *Write Stalls* (https://github.com/facebook/rocksdb/wiki/Write-Stalls) y defaults verificados en `include/rocksdb/advanced_options.h` (rama main); issue facebook/rocksdb#9423.
- Letta, comportamiento del `limit` en core memory: docs.letta.com (*Memory blocks*, *Core memory*) y github.com/letta-ai/letta issue #7 (error literal `Edit failed: Exceeds 2000 character limit`).
- Forte, T., *Building a Second Brain* (2022) — sistema PARA.

Verificación local propia: distribución de tamaños por tier medida sobre la KB
(core n=5 activas máx 27,8 KB; stable n=41 máx 18,8 KB; log activas n=36 máx
58 KB; archive n=59). El "93 log" del recon incluye las 59 archivadas.
