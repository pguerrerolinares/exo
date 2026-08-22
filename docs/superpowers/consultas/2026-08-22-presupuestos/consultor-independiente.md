# Consultor independiente — ¿hay algo mejor que la v2?

Consultor Fable externo a la auditoría, 2026-08-23. Encargo: investigar cómo
resuelve esto la comunidad y la literatura, con permiso explícito para concluir
que la v2 está mal enfocada o que el problema no es de ingeniería. Método:
lectura completa del expediente, verificación primaria propia de los números
decisivos, e investigación externa que **construye sobre** el trabajo del
consultor landscape sin repetirlo. Toda fuente citada está verificada hoy;
donde algo es opinión mía, está marcado.

## 0. Verificación primaria — los números decisivos reproducen

Antes de opinar, medí (script en scratchpad, `git show | wc -c` sobre
`kb-demo`, mismas exclusiones que `kbx budget`):

| Afirmación del expediente | Mi medición | Veredicto |
|---|---|---|
| Pico-a-pico del canon: 453.261 → 310.994 B (−31,4%) | **453.261 B (27 notas) → 310.994 B (27 notas)** — exacto al byte | Reproduce |
| Serie de doctrina-agentes (4 podas, 4 reposiciones) | 8.493 (11-jul) → 45.788 (2-ago) → 24.131 (split 3-ago) → 28.703 (4-ago) → 26.999 (22-ago) → 19.783 (split) → 19.967 la misma mañana | Reproduce, incluida la reposición en horas |
| Banda 92-100%: las 11 del ratchet pegadas al techo | 8 de 11 ≥95%; perfil 100,0%, doctrina 99,8%, desarrollo 99,7% | Reproduce |
| El ratchet tiene 11 techos, 10.000-30.000 | Confirmado leyendo `.kbx-ratchet.json` | Reproduce |

No encontré ninguna discrepancia material con los números canónicos de la
síntesis. La base factual de la v2 es sólida; mi trabajo es sobre el enfoque.

## 1. Veredicto sobre la v2

**La v2 es lo mejor disponible dentro de su clase de soluciones, y su clase es
la correcta.** No la ratifico por inercia: la sometí a los tres ataques que el
brief me autoriza (mal enfocada / problema mal planteado / no hacer nada) y
sobrevive a los tres, con matices que importan:

1. **"No hacer nada" pierde contra el dato.** El −31,4% pico-a-pico y la serie
   R0 (45.788 B en tres semanas sin gate) muestran un sistema que funciona y un
   contrafactual medido de qué pasa sin él. No hacer nada = conservar la
   fricción que motivó la queja. Descartado.
2. **"Problema mal planteado" es verdad a medias, y la v2 ya lo incorpora.**
   El problema real no es volumen (el stock cae ciclo a ciclo) sino (a) el
   precio del ciclo y (b) dos continentes mal factorizados. La v2 ataca
   exactamente eso. Lo que la v2 no dice en voz alta — y debería — está en §4
   y §5 de este informe.
3. **"Mal enfocada": no.** Cada pieza de la v2 tiene análogo externo verificado,
   incluido uno importante que el landscape no encontró y que valida la
   estructura casi punto por punto (§2.1).

Lo que añado no es una alternativa a la v2: son dos piezas de fuera que la
refuerzan, una advertencia que nadie ha contabilizado (§2.2), la respuesta a la
pregunta radical que la auditoría asumió sin contestar (§3), y una opción de
fase 3 que nadie puso sobre la mesa (§6).

## 2. Lo que el exterior añade (no cubierto por el consultor previo)

### 2.1 Wikipedia:Article size — el precedente que faltaba, y ratifica la v2

El landscape afirmó: *"nadie limita el tamaño de una memoria individual del
almacén recuperable"*. En sistemas de memoria de agentes es cierto. Pero el
sistema de conocimiento con décadas de rodaje más grande del mundo **sí lo
hace**, y su régimen es casi idéntico a la v2:

**Wikipedia:Article size** (guideline vigente, verificada hoy): un artículo con
más de **8.000 palabras** de prosa legible *"may need to be divided"*, más de
9.000 *"probably should be divided"*, más de 15.000 *"almost certainly should
be divided"*. Cuatro paralelos exactos con la v2:

- **Umbrales graduados, no un muro único** — la escalera "may / probably /
  almost certainly" es la banda suave + stop de la Fase 2 (y de RocksDB),
  reinventada por una comunidad editorial hace ~20 años.
- **El remedio prescrito es partir con summary style**: las secciones salen a
  artículo propio y el padre conserva **un resumen + enlace** — literalmente
  "pártela y deja índice", la Pata B con la mitigación de routing del abogado.
  Recortar contenido para caber no es remedio aceptado.
- **Se aplica en el momento editorial, no como bloqueo al guardar.** Wikipedia
  no tiene pre-commit de tamaño; el umbral dispara trabajo editorial diferido.
  Es "bytes como tripwire, no como veredicto", que es el principio firmado de
  la v2.
- **La prosa legible excluye listas y tablas del cómputo.** Las notas-índice
  (MOCs, listas, dashboards) se gobiernan aparte — ver §4, porque esto es una
  laguna concreta de la v2.

Dos lecturas honestas de este precedente: (a) valida la **estructura** de la v2
mejor que ningún paper — un sistema editorial humano que lleva dos décadas
creciendo llegó al mismo diseño; (b) los números **no** viajan: 12.500 B de
castellano son ~1.900 palabras — los techos de la KB son ~4× más agresivos que
el umbral más suave de Wikipedia. No es un argumento para subirlos (la función
es distinta: coste de pull por agente vs estamina del lector), pero sí confirma
lo que ya dijo el landscape: ningún sistema externo da soporte a los números
concretos, solo a la mecánica. El 15% de aire y los techos actuales son números
de esta casa, y está bien dicho así en la síntesis (§5.3).

### 2.2 ACE — el coste del ciclo que nadie ha contabilizado

**Agentic Context Engineering** (Zhang et al., Stanford, arXiv:2510.04618,
ICLR 2026) estudia contextos destilados que un LLM reescribe iterativamente —
que es exactamente lo que `/consolida` hace con el canon — y nombra dos modos
de fallo medidos:

- **Brevity bias**: la reescritura bajo presión de concisión tira insights de
  dominio para quedarse con el resumen.
- **Context collapse**: la reescritura monolítica iterativa erosiona detalle
  con cada pasada.

Su remedio: contexto como **bullets estructurados con updates incrementales**
(delta, no reescritura entera) y "grow-and-refine" — crecer con deduplicación
en vez de comprimir.

Traducción a esta auditoría, y es la advertencia central de mi informe: **toda
la auditoría midió bytes; nadie midió pérdida de información por pasada de
consolida.** Cuatro podas con reposición inmediata se leyeron como "el ciclo
funciona" (cierto en volumen), pero ACE predice que cada reescritura del canon
bajo presión de bytes tiene un coste de erosión que no deja rastro en `wc -c`.
La queja de Paul ("consolida siempre muerde") puede tener un componente que
ningún informe recogió: no solo la frecuencia del mordisco, sino la sospecha de
que cada mordisco *degrada* — y eso la v2 no lo mide ni lo mitiga
explícitamente.

Mitigación barata, coherente con lo ya firmado (opinión mía, marcada): la
semántica del mordisco (punto 3 de la v2) debería preferir **mover bloques
enteros** (rotar secciones íntegras a bitácora, partir por heading) sobre
**re-resumir prosa**. La doctrina "pártela, no la mutiles" ya apunta ahí; ACE
le da la razón empírica y el vocabulario. Cero código: una frase más en el
SKILL.md de consolida. La estructura por headings-concepto de las notas de la
KB ya es el formato "itemized bullets" que ACE recomienda — otro punto donde el
diseño local coincide con el estado del arte sin haberlo leído.

### 2.3 La fricción editorial sobre el humano: sí está estudiada

El brief pregunta si alguien ha estudiado el coste de la fricción editorial.
Sí — la literatura de Personal Information Management, y su resultado grande es
incómodo para los dos bandos:

**Whittaker, Matthews, Cerruti, Badenes & Tang, "Am I wasting my time
organizing email?" (CHI 2011)**: 345 usuarios instrumentados, >85.000
operaciones de re-búsqueda reales. Resultado: los que invertían esfuerzo en
organización preparatoria (carpetas al momento de guardar) **no re-encontraban
mejor** (tasa de éxito idéntica, 0.88 vs 0.88) y accedían más lento (mediana
58s por navegación vs 17s por búsqueda). Conclusión del paper: la organización
preparatoria per-ítem no paga cuando hay búsqueda buena; el esfuerzo editorial
en el write path es mayormente desperdicio. (La síntesis de esa línea de
investigación está en Bergman & Whittaker, *The Science of Managing Our Digital
Stuff*, MIT Press 2016, con matices: para ficheros la gente prefiere navegar, y
algo de estructura sí paga.)

Leído deprisa, esto parece munición contra el canon entero: "deja de curar,
busca sobre el log crudo". **Pero no traslada**, y la razón exacta es la
respuesta a la pregunta radical (§3): re-encontrar un email no exige saber si
sigue siendo verdad. La función del canon no es findability — para eso el
retrieval ya basta y Whittaker aplicaría. Es otra cosa, y hay que nombrarla.

Lo que sí traslada de esta literatura, y apoya la v2: **la decisión editorial
per-ítem en el momento de guardar es el patrón que el campo entero identificó
como caro y de bajo retorno.** Mover la mordida del cierre de sesión al
compactador no es solo ergonomía: es la corrección que PIM lleva 15 años
recomendando. Y el foro de Zettelkasten añade el matiz simétrico (hilo "The
Complete Guide to Atomic Note-Taking", verificado): partir notas largas *a
posteriori* es caro — "splitting requires too much decision overhead" — porque
una nota larga es amalgama sin líneas de corte limpias. Consecuencia operativa:
la factorización de la Pata B se paga UNA vez (las particiones ya están
escritas en los headings, el caso barato), y el régimen sostenible después es
**atomicidad de nacimiento** (título-afirmación al crear la nota), no splits
recurrentes. La v2 lo tiene implícito; conviene decirlo explícito.

### 2.4 La práctica PKM que la gente sostiene años: MOCs, no límites

Confirmo con las comunidades lo que el landscape encontró en los autores:
**nadie en Obsidian/Zettelkasten/Roam impone límites de tamaño**; la heurística
emergente más citada es "si la nota necesita scroll, pregúntate si es una
idea". Lo que la comunidad sí institucionalizó — y lleva años sosteniéndose —
es el **MOC (Map of Content)** de Nick Milo: cuando un tema desborda, el hub
deja de contener y pasa a **enlazar**. Es exactamente la "nota-índice como
puerta única de routing" del punto 6 de la v2. La Pata B no es un invento de
esta auditoría: es la práctica estándar de la comunidad con otro nombre. Y
Forte (progressive summarization) añade el *cuándo* comunitario: se destila
**al tocar la nota**, oportunista, no en campaña — coherente con el lazy
splitting ya firmado.

## 3. La pregunta radical: ¿para qué sirve el canon? Respuesta con nombre

La auditoría asumió el canon; el brief pide nombrarlo con precisión. Mi
respuesta, construida sobre el §3.3 del abogado y el modelo bi-temporal de Zep:

**El canon es el bit de vigencia implementado como ubicación.** La KB necesita
distinguir "lo que pasó" (log/archive) de "lo que está en vigor" (canon) —
derecho vigente vs jurisprudencia histórica. El retrieval híbrido encuentra,
pero **no puede adjudicar vigencia**: ante dos chunks que se contradicen (la
doctrina de julio y su corrección de agosto), FTS5 y los embeddings los
puntúan por similitud, no por vigencia. Zep resuelve esto con metadata
(`invalid_at` por arista); la KB lo resuelve **posicionalmente**: si está en
canon, es vigente. Son las dos únicas soluciones conocidas, y la posicional es
la barata — la de metadata exige grafo de afirmaciones con matching LLM en
ingestión (maquinaria, §0).

Consecuencias de nombrar la función:

1. **El canon se queda.** "Disolver el canon en retrieval" no disuelve el
   problema: lo transforma en "retrieval sirviendo pasado con cara de
   presente", que es el peor fallo posible para esta KB. La pregunta radical
   tiene respuesta no-radical.
2. **El techo es el precio del bit de vigencia, no su esencia.** Lo que exige
   la función es que el canon sea *curado* (todo lo que hay es vigente), no que
   sea *pequeño*. El tamaño entra solo como coste de pull y señal/ruido — los
   argumentos ya adjudicados en 1.1 del expediente, reales pero débiles a estas
   escalas (~5k tokens por apertura, ~0,3 aperturas/día).
3. Corolario (opinión mía): el sistema podría relajar techos sin perder la
   función de vigencia — pero el contrafactual R0 muestra que sin presión el
   canon deja de curarse (acumula sin destilar, y la vigencia se degrada por
   dilución). O sea: **la presión de bytes es el proxy mecánico de la
   disciplina de vigencia**, y por eso no hay número correcto — solo un
   trade-off entre frecuencia de mordisco y dilución. La v2 elige bien: presión
   sí, mordisco en el compactador.

## 4. La laguna concreta de la v2: el régimen de las notas-índice

La v2 convierte `doctrina-agentes` en nota-índice y deja el Backlog como
dashboard con techo 30k. Wikipedia y la práctica MOC señalan lo mismo: **una
nota-índice no es una nota de conocimiento y su tamaño es función del número de
hijas, no de disciplina editorial** (Wikipedia excluye listas del cómputo de
prosa; los MOCs no se miden). El expediente ya tiene el dato de alarma:
Backlog es la nota más rápida en re-pegarse al techo de toda la KB
(~1.785 B/día activo, re-pegado el mismo día, dos veces). Ese es el
comportamiento natural de un índice/dashboard, no una patología editorial.

Predicción falsable (mía): tras la factorización, `doctrina-agentes`-como-índice
y `core-index` heredarán dinámica tipo Backlog — crecimiento por conteo de
conceptos, mordiscos que ninguna evicción editorial puede resolver (¿qué
párrafo "no paga su sitio" en una lista de punteros?). La v2 no dice qué pasa
cuando un índice muerde. Propuesta de una línea, cero código, coherente con el
punto 5 de la v2 (Backlog): **doctrina para índices: cuando una nota-índice
muerde, el remedio es compactar formato (línea por hija, sin resúmenes largos)
o partir el índice por sub-tema — nunca evicción por valor, que no aplica.**
Si no se escribe ahora, el primer mordisco de la nota-índice nueva reabrirá
esta conversación entera.

## 5. La hipótesis incómoda, adjudicada

*¿La insatisfacción de Paul viene de que el sistema le exige decisiones
editoriales que no quiere tomar, y ninguna arquitectura lo elimina?*

**Parcialmente cierta, con una precisión que cambia su implicación práctica.**

La parte cierta, con la aritmética delante: canon acotado + producción exógena
⇒ decisiones de evicción recurrentes, por construcción. Eso no lo elimina
ninguna arquitectura — solo se elige **quién** decide (Paul / LLM / nadie),
**cuándo** (write path / offline / nunca) y **con qué criterio** (valor /
orden de llegada / decay). "Nadie + nunca" existe y está medido: es R0. La v2
elige LLM-con-Paul-supervisando + offline + valor, que es la mejor celda de esa
matriz. Hasta aquí, la hipótesis se confirma: **el residuo es irreducible y la
conversación pendiente de la síntesis (§5.1) es real** — si el bucle en sí no
es aceptable, el ajuste es de producción o de expectativa, no de mecanismo.

La precisión que falta: **la mayoría de las decisiones editoriales ya no las
toma Paul — las toman agentes.** `/documenta` y `/consolida` ejecutan; Paul
dispara, supervisa y paga tres costes propios: (a) la interrupción al cierre
(la v2 la mata — es su mejor pieza), (b) la revisión del trabajo del
compactador (irreducible mientras el gatillo y el gate sean suyos — ver §6), y
(c) **el coste meta**: el sistema de presupuestos lleva semanas exigiendo
atención — la queja gemela del cap del hook, la doctrina sobre caps, esta
auditoría de un día con cinco agentes. Parte del "estoy hasta la polla" es
(c), y (c) no se arregla con arquitectura: se arregla con **cierre**. La v2
trae un criterio de reapertura por eventos (Fase 2); lo que le falta es decir
en voz alta su contrapartida: **entre evento y evento, el tema está cerrado y
no se relitiga por incidente.** Si Paul firma la v2, lo más valioso que compra
no es la guarda del 15% — es permiso para no volver a pensar en presupuestos
hasta que una mordida en caso normal lo despierte.

La frase honesta que yo le diría a Paul, y que ningún informe dice así de
crudo: *el sistema funciona (−31,4% pico-a-pico), la fricción que odias tenía
causa mecánica identificada (techos sellados sin aire) y fix mecánico (la
guarda), y lo que quede después de eso no es un bug — es el precio de tener un
canon vigente con producción viva. Ese precio se puede seguir bajando por un
solo camino más (§6), pero no llega a cero salvo que dejes de producir o dejes
de curar.*

## 6. La opción que nadie puso sobre la mesa: consolida programada con gate asíncrono

Todos los informes asumen que `/consolida` la dispara Paul (el SKILL.md lo
dice: "Paul lo invoca al cerrar un frente o semanalmente"). El abogado defendió
el write-path precisamente porque *"es el único punto de detección que corre
sin que un humano decida lanzarlo"* — la cadencia de consolida depende de la
disciplina de Paul, y el supuesto operativo 8 de la cláusula del diagnóstico lo
admite: si la cadencia se rompe, la banda se llena y el bloqueo reaparece. El
landscape descartó automatizarla como over-engineering, pero con el argumento
de la ronda 1 ("hoy no duele eso") — **anterior al rechazo de Paul**, que
demostró que lo que duele es exactamente el ciclo manual.

La pieza que falta tiene precedente triple, todo ya citado en el expediente
sin juntar las piezas: LSM compacta en **background sin que nadie lo pida**;
Letta vende **sleep-time agents** que reorganizan memoria offline de forma
autónoma; y este mismo harness ya opera el patrón "Paul fuera del critical
path con gate de merge asíncrono" (la fábrica). La forma mínima:

- Un **scheduled run semanal** de `/consolida` (el harness ya tiene scheduled
  agents; cero código nuevo en kbx/exo) que trabaja **en rama** de kb-demo
  y deja el resultado como diff para revisión.
- **Paul conserva el gate**: revisa el diff y mergea cuando quiera — misma
  revisión que hoy hace en caliente, movida a asíncrono. El pre-commit y el
  trinquete siguen vigilando igual (la rama pasa por el mismo gate al commitear).
- La objeción del abogado ("un consolidador autónomo corrompe canon en
  silencio") queda respondida por el gate: nada llega a main sin ojos de Paul,
  y la skill ya trae checks de conservación (conteo de headings, regla de oro
  nada-se-borra, dry-run primero).
- **Qué se retira a cambio** (obligación del brief): el supuesto operativo 8
  de la cláusula (la cadencia como disciplina de Paul) desaparece, y con él
  gran parte de la razón de ser de la banda de Fase 2 — si el compactador
  corre solo cada semana, el hueco que la banda cubre casi no existe. Menos
  maquinaria futura, no más.

Condición de disparo, coherente con el estilo por-eventos del expediente
(opinión mía): **no ahora**. La v2 se ejecuta tal cual ("una pasada") y se
observa la vuelta de Paul. Si el sistema vuelve a morder *porque consolida no
corrió a tiempo* — mordida con la banda llena y sin pasada reciente — esa es la
evidencia de que el eslabón débil es el gatillo humano, y entonces el cron
gana su lugar en vez de la banda. Si Paul sostiene la cadencia sin dolor, no se
construye. Es la misma lógica "gana su código con evidencia" que la Fase 2,
aplicada al gatillo en vez de al gate.

Refinamiento opcional que dejo anotado sin recomendar aún (rozaría "métricas
nuevas" si se formaliza): la evicción por valor de la v2 decide "qué párrafo no
paga su sitio" sin ningún dato de uso. Existe señal gratis en artefactos que ya
se generan (transcripts de Claude Code, `reflex-log.jsonl` con los permalinks
inyectados) — el coordinador ya los minó una vez a mano. Un paso de consolida
que los grepee convertiría la evicción de juicio-en-frío a juicio-con-uso, que
es como evictan todos los sistemas de §3 del landscape (LRU/LFU). Barato, pero
es una pieza más: solo si la evicción a ciegas demuestra fallar.

## 7. Qué firmaría yo

1. **La v2, tal cual está, se ejecuta.** Es lo mejor disponible y ninguna
   fuente externa que haya encontrado sugiere una clase de solución superior
   para una KB de un autor bajo §0. La estructura tiene ahora un precedente
   humano de dos décadas (Wikipedia) además de los de sistemas.
2. **Tres líneas de doctrina que añadiría** (cero código las tres): (a)
   preferir mover bloques enteros a re-resumir prosa en cada poda — ACE; (b)
   régimen propio para notas-índice cuando muerdan — compactar formato o
   partir índice, nunca evicción por valor (§4); (c) atomicidad de nacimiento:
   toda nota nueva de learnings/core nace con título-afirmación — el test del
   título aplicado en el único momento en que es gratis.
3. **Cláusula de cierre explícita**: firmada la v2, el tema presupuestos queda
   cerrado hasta el evento de Fase 2. Sin relitigación por incidente. Es la
   pieza que ataca el coste (c) de §5, y nadie la ha escrito.
4. **Fase 3 condicional** (§6): si la mordida vuelve por cadencia rota de
   consolida, scheduled run semanal en rama con merge de Paul — y la banda de
   Fase 2 probablemente no llegue a construirse. Registrado hoy para no
   rediseñarlo en caliente.
5. **A la pregunta de Paul** ("¿apaño o solución?"), mi respuesta de fuera:
   el mecanismo es solución con pedigrí doble (sistemas + editorial humano);
   el bucle no es un defecto sino el precio del bit de vigencia (§3); la parte
   de apaño era real y localizada (sellado sin aire, mordisco al cierre, dos
   continentes) y la v2 la retira. Lo que quede de insatisfacción después de
   la v2 ya no tendrá remedio técnico — tendrá el remedio de §5: cerrar el
   tema, o recalibrar cuánta doctrina quiere sostener en canon.

## Fuentes

Nuevas de este informe (verificadas 2026-08-23):

- Wikipedia, *Wikipedia:Article size* (guideline; umbrales 8k/9k/15k palabras
  de prosa legible, summary style, exclusión de listas/tablas).
  https://en.wikipedia.org/wiki/Wikipedia:Article_size
- Zhang, Q. et al., *Agentic Context Engineering: Evolving Contexts for
  Self-Improving Language Models*, arXiv:2510.04618 (ICLR 2026). Brevity bias,
  context collapse, incremental delta updates.
  https://arxiv.org/abs/2510.04618
- Whittaker, S., Matthews, T., Cerruti, J., Badenes, H., Tang, J., *Am I
  wasting my time organizing email? A study of email refinding*, CHI 2011.
  https://dl.acm.org/doi/10.1145/1978942.1979457
- Bergman, O. & Whittaker, S., *The Science of Managing Our Digital Stuff*,
  MIT Press, 2016 (síntesis PIM; citada con alcance general, sin números).
- Zettelkasten Forum, *The Complete Guide to Atomic Note-Taking* (heurística de
  pantalla, decision overhead del split tardío).
  https://forum.zettelkasten.de/discussion/3335/the-complete-guide-to-atomic-note-taking
- Nick Milo / comunidad LYT, patrón MOC (Map of Content) como hub que enlaza
  en vez de contener (múltiples fuentes comunitarias; concepto, no cifras).
- Forte Labs, *Progressive Summarization* (destilado oportunista al tocar la
  nota). https://fortelabs.com/blog/progressive-summarization-a-practical-technique-for-designing-discoverable-notes/
- Anthropic, *Effective context engineering for AI agents* (compaction y
  structured note-taking como técnicas estándar; contexto general).
  https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents
- MemOS (arXiv:2507.03724) — revisado y **descartado** como over-engineering
  para este caso, consistente con el §5 del landscape; lo cito solo como
  registro de que la familia "memoria como OS" no aporta nada nuevo aquí.

Heredadas del expediente sobre las que construyo: Zep/Graphiti
(arXiv:2501.13956) para §3; sleep-time compute (arXiv:2504.13171) y Letta
sleep-time agents para §6; RocksDB Write Stalls para §2.1 y §6.

Verificación primaria propia: script `verify.py` en mi scratchpad
(`git show | wc -c` sobre kb-demo; pico-a-pico, serie doctrina-agentes,
banda del ratchet). Solo lectura en todo momento; ni la KB, ni los repos, ni
`~/.exo/index.db` fueron modificados (el índice no hizo falta abrirlo).
