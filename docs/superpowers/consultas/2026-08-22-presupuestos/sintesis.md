# Síntesis — auditoría del sistema de presupuestos de la KB

Coordinador (Fable), 2026-08-22. Construida haciendo debatir a los cuatro autores
(tres rondas de confrontación + ronda de ratificación), con mediciones propias
donde los informes chocaban. Todo lo afirmado aquí es trazable a un informe, a
una respuesta de ronda, o a una medición con script en scratchpad.

## Veredicto en una línea

**El mecanismo no es un apaño — la calibración y el punto de mordida sí lo son.**
El techo por nota, el trinquete y `/consolida` sobreviven la auditoría con
evidencia a favor; lo que fabrica la fricción que motivó la queja es que los
techos se sellan sin aire (márgenes de 9-350 bytes) y que el mordisco cae sobre
el escritor al cierre de sesión, donde no hay juicio editorial disponible.

---

## 1. Contradicciones y cómo se resolvieron

### 1.1 ¿Cuánto cuesta hoy el tamaño de una nota? (abogado vs diagnóstico)

La contradicción central. Se resolvió con medición, no con argumento: escaneé
los 79 transcripts reales de Claude Code (2026-07-23 → hoy, 675 MB, todos los
proyectos) contando aperturas de nota entera de la KB por todas las vías
(`Read` sin limit, `cat`, MCP `read_note`).

**Resultado: ~50 aperturas de nota entera en 30 días**, frente a 97 invocaciones
de `exo search/recall` y ~82 lecturas parciales. Concentradas en dos sitios:
julio pre-split (`agent-solve-it` abierta ~10 veces cuando medía 29-84 KB) y la
campaña `/consolida` del 08-03 (12 aperturas: el compactador leyendo para
partir). **Post-split (08-04 → hoy): ~5-8 aperturas enteras en 19 días, todas
≤27,8 KB.** El abogado verificó el conteo por su cuenta y lo confirmó (41 de
las 44 lecturas MCP son de la ventana 24-jul→3-ago; 3 desde el 4-ago).

Adjudicación — **los dos tenían media razón, y ambos lo concedieron**:

- El **abogado acierta en el mecanismo**: `exo search` devuelve punteros a nivel
  nota (`type: "entity"`), la unidad de consumo en apertura deliberada es la
  nota entera, y el coste fue real en julio (la nota de 89 KB se abrió repetida
  a tamaño completo). El split lo mató — punto a favor del sistema.
- El **diagnóstico acierta en la magnitud actual**: ~0,3 aperturas/día, ≤28 KB.
  El delta que gobernaría una banda del 15% son ~750 tokens en un puñado de
  aperturas — ruido. Pero concedió que esa marginalidad **es un logro del
  dique, no del retrieval**: la cota ≤28 KB la ponen los techos.
- "M6-06 multiplica las aperturas" era **pronóstico vestido de dato** (concedido
  por el abogado): el 86% es la tasa de disparo del gate léxico, trazable y bien
  medida (233/272 prompts reales, spec `2026-08-22-m6-06-recall-punto-de-uso-design.md`),
  pero mide inyección de punteros, no aperturas. M6-06 se mergeó hoy: cero datos.

### 1.2 ¿Qué inyecta el SessionStart? (abogado vs diagnóstico)

Resuelta con el código delante (`plugins/reflex/scripts/exo-recall.sh`): inyecta
**solo** `kb-demo/core/core-index` (5.232 B, cap 6.144, al 61% de su
nominal, jamás dio fricción). Las otras 4 notas core no se inyectan nunca. El
abogado **concedió**: su "el presupuesto core es carga estructural hoy" queda
restringido a core-index; el nominal de 8.500 B de las demás no tiene
justificación de transporte — y nunca la tuvo: la spec fundacional las capó por
coste de pull, coherente con la corrección del contexto.

### 1.3 El A/B del abogado, re-fechado (abogado vs arqueología)

El A/B más citado del expediente ("+77,7 KB en 16 días sin enforcement; 0 B en
19 días con él") quedó tocado por la cronología que la arqueología verificó en
primario:

- El pre-commit se creó el **17-ago a las 20:49:40** (`3aebd5d`) — **35 segundos
  después** del commit del triaje (`27f897e`): primero se cerraron las brechas,
  luego nació el hook que las habría bloqueado. Instalado entre el 17 y el
  18-ago (discrepancia de un día entre auditores, inmaterial). Antes no hubo
  **ningún** hook de git en la KB. La activación se pospuso a propósito (spec
  `f64502c`, verificada también por el abogado) para no dejar doctrina
  contradictoria.
- No hay ni un commit ni bitácora que documente una corrida manual de
  `kbx budget` con offenders visibles e ignorados entre el 07-11 y el 08-17.
- El diagnóstico añadió el dato complementario: el único consumidor de
  `kbx budget` era `/consolida`, y entre el 07-12 y el 08-03 hubo **22 días sin
  que el disparador corriera ni una vez**. El +77,7 KB cae entero en ese hueco.
  Y cuando el warning por fin corrió (08-03), **funcionó**: split de 9 notas.
- Los "0 B de deriva en 19 días": `desarrollo-agentico` y `agent-solve-it`
  están planas por **inactividad** (0 B de cambio), no por enforcement.

Conclusión adjudicada — **y aceptada por el abogado por escrito**: el A/B
compara "**sin instrumento** ni canal de entrega" contra "régimen completo", no
"warning ignorado" contra "bloqueo". Refuta el régimen de julio (nadie lo
defiende); **no** refuta "warning entregado en cada commit + bloqueo duro en
alguna cota". La objeción derivada del abogado ("hubo dos consolidas manuales en
la ventana del crecimiento a 45.788 B, luego el destilado sin muro no se
sostiene ni a mano") cae por la misma cronología: esas consolidas (3, 5, 9,
11-jul) son anteriores o simultáneas al nacimiento de la propia herramienta de
medición.

El re-fechado destapó, además, la tabla de **tres regímenes** (abogado, §8 de su
informe) que es la foto más precisa del expediente:

| Régimen | Periodo | Comportamiento medido |
|---|---|---|
| R0: nada | jul | +1,8 KB/día sostenido |
| R1: instrumento + techos + triaje ex-post | 3→17-ago | la única sesión que escribió en doctrina metió +4,6 KB en un día, atravesó el techo sellado sin que nada la parase, y la brecha tardó 13 días en repararse |
| R2: + pre-commit | 17-ago→hoy (4-5 días) | dos mordiscos, cero brechas; el coste es la fricción de la queja |

Lo que queda en pie de la tesis del abogado — y nadie lo disputa: **la presión
debe existir en alguna cota** (señal sin bloqueo es R0, medido), y el write-path
aporta dos cosas que el triaje manual no da: **detección que corre sin que un
humano decida lanzarla** y latencia de reparación acotada (~13 días → 0).

### 1.4 ¿Notas coherentes o notas-área? (landscape vs diagnóstico)

El landscape sostenía que las notas del trinquete son **notas-área**: títulos
que admiten cualquier contenido futuro y por eso no convergen nunca (test del
título de Matuschak). El diagnóstico dictaminó 9-de-11 coherentes. Confrontados,
el landscape **retiró su formulación gruesa** y la sustituyó por una
adjudicación nota a nota (§6.6 de su informe):

- **Fallan 2**: `doctrina-agentes` (sus 10 headings ya son títulos-API de
  conceptos independientes; es un volumen de ~10 notas-concepto encuadernadas, y
  su sección "capítulos que viven en nota propia" prueba que el split por
  concepto ya empezó y funcionó) y `desarrollo-agentico` (mezcla método SDD +
  hechos del harness + epistemología, y solapa territorio con doctrina-agentes —
  por eso el aprendizaje de hoy no cabía en ninguna: ambas lo reclamaban).
- **Falla de formato, no de título**: `Backlog` (dashboard; remedio: frente =
  3-5 líneas + puntero a projects/, no split).
- **Pasan 7**: perfil (nota-entidad), pragmatismo-y-pivots (concedida al
  diagnóstico) y las 5 de projects/ (entidades acotadas por mortalidad:
  convergen al cerrar el proyecto — lighthouses ya convergió).

Lo que valida el test como criterio: **las que pasan viven sin morder; las 2 que
fallan son exactamente las 2 que mordieron hoy** (a 33 y 49 B del techo). La
discrepancia con el diagnóstico queda reducida a una sola nota
(`doctrina-agentes`) y la resolución operativa es la versión mínima compatible
con §0 — **lazy splitting**: el test lo aplica el LLM de `/consolida` solo
cuando una nota dispara el tripwire, Paul aprueba o rechaza la partición. Sin
campaña de refactor.

### 1.5 Hallazgos laterales verificados

- **La amnistía mancha el "solo baja"**: el techo de doctrina-agentes subió de
  8.500 nominal a 27.000 sellado el día de la amnistía (+218%), en bloque, con
  nombre ceremonial. Desde el ancla es verdad que solo baja — pero el historial
  "solo baja" son 19 días con una sola bajada (concedido por el propio abogado).
- **La brecha del 04-ago fueron TRES notas, no una** (`9238ade`): doctrina-agentes
  +1.703 B, perfil +1.428 B y Backlog +584 B sobre sus techos sellados —
  exactamente las tres que el triaje del 17-ago dice arreglar. Cero commits
  sobre ellas en los 13 días intermedios. Explicada por la cronología: el gate
  no existía aún; lo que reparó fue el triaje.
- **La "sierra" del abogado no sostiene su tasa**: su ~10 B/día "en el muro" es
  en realidad 80-240 B/día según la serie (diferencial real frente a R0: ~7-22×,
  no 180×, incluso normalizando el 5× de menos actividad de agosto). El patrón
  real es más tosco: crecimiento libre → meseta plana de 13 días en brecha (nada
  vigilaba) → corrección de golpe + crecimiento lento bajo enforcement.
- **Los "atractores" del diagnóstico, matizados por la serie**: de las 4 notas
  sin waiver al 98-99,9% del nominal, solo `kbx.md` se aproximó al techo de
  forma orgánica y continua; las otras 3 nacieron por encima, se recortaron UNA
  vez en la consolida masiva del 05-jul (a ras del número, como recorta
  cualquiera con un número delante) y llevan 36-48 días congeladas. El atractor
  recurrente real, con ciclos repetidos de crecer→tocar→recortar, son las waiver
  activas: Backlog, doctrina-agentes, perfil.
- **El relleno post-triaje (1.863 B en 48 h) no era reacreción**: era doctrina
  nueva legítima (cutover M6-04 + una viñeta nueva). Pesa a favor del abogado en
  ese caso: el techo ajustado al límite no absorbe ni el flujo legítimo.
- **Tasas de crecimiento heterogéneas** (arqueología): Backlog ~1.785 B/día
  activo; cores activas ~80-180 B/día; desarrollo-agentico y solve-it 0 B en 19
  días. No hay una tasa media de la KB: el dimensionado es por nota.
- **Archive en el retrieval** (orquestador, sobre reflex-log de hoy): 33% de los
  punteros servidos por el hook M6-06 fueron a `archive/` — pero en pares
  canon/archivo del mismo tema el vivo gana en score (0,65 vs 0,58). El archivo
  **no suplanta al canon: le quita plazas** (un tercio de un bloque de 3).
- **Precedente externo verificado** (landscape, fuente primaria): RocksDB usa
  dos umbrales (slowdown en el nominal, stop total a 1,8-4×; saltarse la banda
  suave es un bug reconocido, issue #9423), y su stop significa "espera al
  compactador", nunca "recorta el payload". Letta **sí bloquea en write path**
  (`ValueError: Exceeds 2000 character limit`) — pero solo en el tier inyectado,
  con el propio LLM como bloqueado y con salida de reorganizar, jamás de
  mutilar. Ningún precedente cubre el evento de hoy: recorte por 33 B en el
  nominal, pagado por el escritor al cierre.

---

## 2. La propuesta ratificada

**Qué se queda (intocable)**: techo por nota, trinquete solo-baja, pre-commit
como tripwire, doctrina "pártela, no la mutiles", `/consolida` con humano en el
gatillo, canon+bitácora+archive.

**El principio que la organiza** (formulación del diagnóstico, aceptada por el
abogado): *bytes como tripwire, sí — es lo único mecánicamente chequeable en un
pre-commit; bytes como veredicto, no.* El mismo disparo produjo destilación real
en `/consolida` (`5da6c59`) y desplazamiento puro en el cierre de sesión
(`09a75eb`).

### Fase 1 — ya, sin esperar evidencia nueva

1. **Guarda de sellado en kbx** (`internal/ratchet`, repo kbx): sellar o bajar
   un techo exige **≥15% de aire** sobre el tamaño actual de la nota. Pocas
   líneas en un gate que ya existe. Propuesta original del abogado, mecanizada a
   petición del coordinador (sin guarda era disciplina humana: el split de hoy
   dejó 217 B de aire el mismo día en que se escribió la prescripción del 15%).
   Dimensionado: cores activas ~80-180 B/día → 3-4 KB ≈ 2-5 semanas ≈ cadencia
   real de `/consolida` (huecos de 19-22 días).
2. **Evicción editorial nombrada en `/consolida`** (SKILL.md de reflex): "en
   cada pasada, qué párrafo del canon ya no paga su sitio → baja a bitácora".
   Condición del diagnóstico: **1 y 2 van juntos** — podar-para-dejar-aire sin
   criterio de valoración sería rotación por orden de llegada en frío.
3. **Test del título al partir** (misma skill): cuando toque split, si el título
   admite cualquier contenido futuro (nota-área), partir **por tema**, no solo
   canon/bitácora. Solo en el momento del split; sin campaña de refactor.
4. **El Backlog sale del régimen de nota de conocimiento**: es un dashboard de
   estado (su tamaño es función de frentes abiertos; ~1.785 B/día activo —
   ningún headroom le sirve). Conserva su techo 30.000 como cota de dashboard;
   cuando muerda, la acción es cerrar/archivar frentes, no destilar. Cero código.
5. **Sincerar la doctrina en `core-index`**: el sistema real es "techo por nota
   + trinquete"; los nominales de tier (8.500/12.500) quedan como default para
   notas nuevas, no como descripción del sistema (4 de 5 cores viven de waiver).
6. **Semántica del mordisco, escrita en la doctrina**: cuando el gate muerda, la
   salida es partir/rotar con juicio o correr `/consolida` — **recortar el delta
   entrante para caber es el anti-patrón** (precedentes RocksDB y Letta, §1.5).
7. **Primera aplicación**: la próxima `/consolida` salda las notas **activas** a
   <350 B de techo (perfil 9 B, doctrina-agentes 33 B), partiendo más hondo y
   resellando con aire — techos solo bajan, trinquete intacto. Las inactivas
   (desarrollo-agentico 49 B, solve-it 152 B: 0 B de cambio en 19 días) no urgen.

### Fase 2 — condicional, con criterio falsable

8. Si tras **dos pasadas de `/consolida`** con la guarda activa `/documenta`
   vuelve a morder en el caso normal (rotación forzada o IOU para que quepa un
   delta — observable en commits, cero métricas nuevas), la banda gana su
   código: **warning en cada commit entre techo y techo+15%, bloqueo duro en
   techo+15%**. Coste: una comparación + exit code en `internal/budget` /
   pre-commit. Es la semántica exacta del dual-trigger de RocksDB, y respeta la
   línea roja del abogado (el bloqueo existe siempre en alguna cota).

### Recomendación separada (decisión independiente)

El retrieval **penaliza o excluye `archive/` por defecto**, con opt-in para
búsqueda histórica — un `WHERE`/penalización de score, no un subsistema.
Argumento con la magnitud correcta: el archivo gasta plazas escasas del bloque
de inyección (33% de los punteros de hoy), no suplanta al canon (el vivo gana en
score). No toca el sistema de presupuestos; Paul puede aprobarla o rechazarla
con independencia de la propuesta principal.

### Restricciones, comprobadas

§0: se modifica maquinaria existente (una guarda en un gate que ya existe, dos
líneas de skill, doctrina), no se construye nueva. Trinquete: intacto y
reforzado (la guarda lo hace más honesto al sellar). Cero métricas nuevas: el
criterio de Fase 2 se lee de commits que ya se producen. Disciplina humana
nueva: ninguna fuera de flujos ya existentes.

---

## 3. Ratificación

| Voto | Fase 1 | Fase 2 | Separada (archive/) |
|---|---|---|---|
| cons-abogado | **SÍ** | **SÍ** | **SÍ** (matiz) |
| cons-landscape | **SÍ** | **SÍ** | **SÍ** |
| cons-diagnostico | **SÍ** | **SÍ** | **SÍ** (dos matices de acta) |
| aud-arqueologia (chequeo factual) | **SÍ salvo el punto 4** | **SÍ** | **SÍ** |

> **Nota del orquestador sobre estas dos filas.** El coordinador agotó su límite
> de sesión justo después de recibir estos dos votos y antes de transcribirlos,
> así que los registro yo desde sus mensajes de ratificación. Lo que llegó es el
> veredicto, no su desarrollo: del diagnóstico, "SÍ, SÍ, SÍ — con dos matices de
> acta"; del arqueólogo, "7 SÍ, 1 NO (Backlog no es dashboard), 1 matiz a
> cerrar". Los nueve ítems votados son los 7 puntos de Fase 1 + Fase 2 + la
> recomendación separada, así que el NO cae sobre el **punto 4**. **El contenido
> de los tres matices no llegó a escribirse en ningún sitio** y los cuatro
> agentes están caídos hasta las 19:00; si alguno resulta importante, hay que
> volver a preguntárselo, no reconstruirlo.

**Abogado — SÍ a todo, con firma explícita de "bytes como tripwire sí, veredicto
no"** y aceptación verificada por él mismo del re-fechado de su A/B (comprobó
`f64502c` en primario). Sus condiciones quedaron dentro de la propuesta: bloqueo
duro en alguna cota (Fase 1: techo; Fase 2: techo+15%), guarda mecánica de
sellado con aire, y el disparo en banda no exige resolución al cierre ("el
pre-commit deja de pedir cirugía al cierre y pasa a pedir cita con el
cirujano"). Matices no bloqueantes: (a) cuando la banda gane su código, el
warning debe ser **acumulativo** ("N notas en banda desde hace M días" — la
latencia de reparación medida es ~13 días y un warning puntual se normaliza como
ruido); (b) en la recomendación separada, **penalizar antes que excluir** —
el 33% es dato del primer día de M6-06, y la doctrina de la propia KB desconfía
de ajustes nacidos de una medición puntual. Cita de cierre: *"empecé defendiendo
el statu quo entero y termino firmando una propuesta que conserva sus mecanismos
y jubila su fricción — que es como debía terminar si el proceso funcionaba."*

**Landscape — SÍ a todo.** Votó deliberadamente a favor de mantener el stop en
el techo en Fase 1 (contra su propia semántica inicial): "si la guarda basta, mi
banda codificada era over-engineering — YAGNI aplicado a mi propia
recomendación; si no basta, gana su código con evidencia". Su condición no
negociable (la salida del mordisco es partir/rotar/consolidar, nunca recortar el
delta) queda satisfecha en el punto 6. Observación operativa no bloqueante: el
test del título debería correr también en la primera aplicación (punto 7) si la
salida elegida para doctrina-agentes es el split — es uno de los dos especímenes
donde el test ya tiene veredicto y la partición ya está escrita en sus headings.
Confirmó explícitamente el reencuadre de archive/ ("gasta plazas escasas").

---

## 4. Lo que queda abierto para Paul

**Unanimidad en 8 de los 9 ítems.** Fase 1 (salvo el punto 4), Fase 2 y la
recomendación separada salen ratificadas por los cuatro, incluido el voto
adversarial. Quedan tres decisiones:

1. **El único desacuerdo real: el punto 4, el Backlog.** La propuesta lo saca
   del régimen de nota de conocimiento y lo trata como dashboard de estado (su
   tamaño es función de cuántos frentes hay abiertos, ~1.785 B/día en activo, y
   ningún headroom le sirve). El arqueólogo vota **NO**: sostiene que el Backlog
   no es un dashboard. Es una discrepancia de categoría, no de mecanismo, y
   decide una sola cosa: cuando el Backlog muerda su techo, ¿la acción es
   **cerrar y archivar frentes** (propuesta) o **destilar como cualquier nota
   canónica** (arqueólogo)? Su argumento detallado no llegó a escribirse.

2. **La recomendación separada de `archive/`**, que es independiente del resto y
   se aprueba o rechaza por su cuenta. Si la apruebas, el matiz del abogado es
   bueno: **penalizar antes que excluir**, porque el 33% es dato del primer día
   de M6-06 y la doctrina de esta casa desconfía de los ajustes nacidos de una
   medición puntual — exactamente el error que originó esta auditoría.

3. **El número del aire: 15%.** Nadie lo disputó, pero tampoco lo defiende
   ninguna evidencia externa: sale de dimensionar la cadencia real de
   `/consolida` (huecos de 19-22 días × 80-180 B/día en cores activas ≈ 3-4 KB).
   Es un número de esta casa, no del estado del arte. Merece decidirse a
   sabiendas de eso, que es justo lo que no se hizo con el 1024 de esta mañana.

**Riesgo declarado del proceso**: los cuatro matices no escritos (dos del
diagnóstico, uno del arqueólogo, y el operativo del landscape sobre correr el
test del título en la primera aplicación) se perdieron al caer las sesiones. El
landscape sí dejó el suyo por escrito arriba; los otros tres no. Si Fase 1 se
implementa antes de las 19:00, se implementa sin ellos.
