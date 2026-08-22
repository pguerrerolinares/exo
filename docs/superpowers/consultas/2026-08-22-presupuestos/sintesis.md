# Síntesis — auditoría del sistema de presupuestos de la KB

Coordinador (Fable), 2026-08-22. Construida haciendo debatir a los cuatro
autores: cinco rondas de confrontación, una ratificación (v1), un rechazo de
Paul con datos nuevos, dos retractaciones del propio orquestador, y una segunda
propuesta (v2) firmada por los cuatro. Todo lo afirmado es trazable a un
informe, a una respuesta de ronda o a una medición con script; donde dos
mediciones chocaron, se dice quién tenía razón y por qué.

## Veredicto en tres líneas

**El mecanismo no es un apaño; la calibración, el punto de mordida y dos
continentes concretos sí lo son.** Y la respuesta honesta a la queja de Paul
("consolida ya lo hemos hecho varias veces y siempre muerde") es: **tiene razón,
y ninguna propuesta cambia eso** — la tasa de reposición es su propia producción
de doctrina. Lo que sí se puede cambiar, y la v2 cambia: dónde aterriza esa
producción (factorizar los 2 cajones), cuánto cuesta cada ciclo (aire
garantizado + evicción con juicio) y cuándo muerde (el compactador, no el cierre
de sesión).

---

## 1. Contradicciones factuales y cómo se resolvieron

### 1.1 ¿Cuánto cuesta hoy el tamaño de una nota? (abogado vs diagnóstico)

Resuelta con medición del coordinador sobre los 79 transcripts reales de Claude
Code (07-23→hoy): **~50 aperturas de nota entera en 30 días** frente a 97
invocaciones de `exo search/recall` y ~82 lecturas parciales. Concentradas en
julio pre-split (`agent-solve-it` abierta ~10 veces a 29-84 KB) y en la campaña
`/consolida` del 08-03. Post-split: ~5-8 aperturas en 19 días, todas ≤27,8 KB.
El abogado verificó el conteo por su cuenta y lo confirmó.

Adjudicación, concedida por ambos: el **abogado acierta en el mecanismo** (la
unidad de consumo es la nota entera; el coste fue real en julio y el split lo
mató) y el **diagnóstico en la magnitud actual** (~0,3 aperturas/día, ≤28 KB) —
pero esa marginalidad **es un logro del dique, no del retrieval** (concesión del
diagnóstico). "M6-06 multiplica las aperturas" era pronóstico vestido de dato
(concedido): el 86% es tasa de inyección de punteros (233/272 prompts, trazable
a la spec), no de aperturas.

### 1.2 ¿Qué inyecta el SessionStart? (abogado vs diagnóstico)

Resuelta con el código (`exo-recall.sh`): inyecta **solo** `core/core-index`
(5.232 B, cap 6.144, 61% de su nominal, cero fricción histórica). Las otras
notas core no se inyectan nunca. El abogado concedió: el nominal de 8.500 B no
tiene justificación de transporte — y nunca la tuvo (la spec fundacional capó
por coste de pull, coherente con la corrección del contexto).

### 1.3 El A/B del abogado: tres regímenes, y un factor que bajó tres veces

La cronología, verificada en primario por arqueología y aceptada por el abogado:

- `kbx budget` nace el 11-jul (`5cc755c`); su único consumidor era `/consolida`,
  que **no corrió ni una vez en 22 días** (12-jul→3-ago). El +77,7 KB de
  agent-solve-it cae entero en ese hueco.
- El trinquete nace el 3-ago (`41b3959`). Entre el 3 y el 17-ago **nada hacía
  cumplir los techos**: tres notas (doctrina +1.703 B, perfil +1.428, Backlog
  +584) vivieron 13 días en brecha (`9238ade`), intactas hasta el triaje.
- El pre-commit bloqueante se creó el **17-ago 20:49:40** (`3aebd5d`) — 35
  segundos después del commit del triaje. La activación tardía fue deliberada
  (spec `f64502c`).

| Régimen | Periodo | Comportamiento medido |
|---|---|---|
| R0: nada | jul | +1,8 KB/día (+3,1 KB/día **activo**) sostenido |
| R1: instrumento+techos+triaje ex-post | 3→17-ago | una sesión metió +4,6 KB en un día atravesando el techo sellado; 13 días en brecha |
| R2: + pre-commit | 17-ago→hoy | dos mordiscos, cero brechas; el coste es la fricción de la queja |

El A/B quedó como "sin instrumento vs régimen completo", no "warning ignorado
vs bloqueo" — y **su factor de contención bajó tres veces, siempre a la baja:
180× → 78× → ~9-21×** (el 78× del addendum era un error de anclaje del
orquestador: su `--before` cogía el commit posterior al rebote del 3-ago; el
anclaje correcto es el mínimo post-poda, y es de arqueología). Matices finales
del propio abogado: el neto "en el muro" compensa appends con podas — el bruto
fue ~1,9 KB/día activo, así que **el muro no disuade la escritura: lo que
contiene el stock es la poda; la contención es el ciclo completo, no el dique**.
Y su barra de confianza sobre R2: "casi nula como estadística — la vuelta de
Paul ES el experimento".

Lo que queda en pie, indiscutido: la presión debe existir en alguna cota (señal
sin bloqueo es R0, medido), y el write-path aporta detección que corre sin que
un humano decida lanzarla, con latencia de reparación ~13 días → 0.

### 1.4 ¿Notas coherentes o notas-área? (landscape vs diagnóstico)

La contradicción se cerró en dos pasos. Primero ambos convergieron en "fallan
2 de 11" (doctrina-agentes y desarrollo-agentico — exactamente las dos que
mordieron el día de la auditoría). Después, el diff por headings de arqueología
(11-jul→hoy) les dio el instrumento que faltaba — **% del crecimiento en
headings nuevos**:

| Nota | % headings nuevos | Dictamen |
|---|---|---|
| Paul - perfil | **0%** (8 cabeceras idénticas 6 semanas) | converge — pata ciclo |
| pragmatismo-y-pivots | ~12% (un bolo del 3-ago, plana 19 días) | digirió su campaña — watchlist |
| desarrollo-agentico | ~65%, **afines al título** | tema amplio: partir por **género** |
| doctrina-agentes | ~80%, **sin relación entre sí** | cajón: partir por tema, queda como índice |

Sin zona gris. El test del título queda operacionalizado con criterio numérico
barato (git, cero métricas nuevas): ~0% → engorde, evicción; >50% → candidata a
partir; entremedias → juicio del LLM (umbral revisable — 4 puntos de datos,
cautela del landscape). Y el caso **perfil** obligó a una corrección simétrica
del diagnóstico: una nota puede converger por estructura Y por título y aun así
clavar el techo por puro engorde (+9.853 B hasta quedar a 9 B del muro) —
"convergente ⇒ sin fricción" es falso; su remedio es el ciclo, no partir.

### 1.5 Las dos cifras del addendum, retractadas por su autor

Paul rechazó la v1 con "consolida ya lo hemos realizado varias veces y siempre
muerde", y corrigió el A/B (un mes de vacaciones: parte de la calma era ausencia
de uso — quedó cuantificado: ~4 días activos en agosto; 16 commits que tocan
canon en la ventana actual frente a 54 del hueco natural entre consolidas
históricas). El addendum del orquestador midió el rechazo, y de él:

- **Sobrevive lo esencial**: cuatro podas de doctrina-agentes, cuatro
  recuperaciones inmediatas — la del 3-ago repuso +2.709 B **en horas** (serie
  del día: 24.131→26.113→26.840) y la de hoy +184 B la misma mañana.
  `/consolida` desahoga, no estabiliza. Paul tiene razón.
- **Se retractaron dos cifras** (error de método: filtro por prefijos de carpeta
  sobre una KB reorganizada — comparaba conjuntos distintos de ficheros): "el
  canon total no crece" es **falso** (KB entera 570.910 → 1.498.996 B, +163%) y
  la concentración "99,8% en 6 notas, +77 B las otras 44" era **artefacto** —
  el dato firme es **21 de 27 notas con techo crecen; el top-3 concentra el
  53%**. La retractación la inició arqueología (no reproducible) y la confirmó
  el orquestador enseñando su script.
- Cautela unánime sobre el +163%: incluye `log/` y `archive/`, que crecen **por
  diseño** (régimen extensivo). Mide producción total, no descontrol; la serie
  que decide política de presupuestos es el canon con techo (21/27, top-3 53%).

Consecuencia: la reordenación "factorización como LA acción principal" — que
landscape y diagnóstico habían firmado apoyándose en parte en la concentración
falsa — fue revisada por ambos y **retirada**: con crecimiento distribuido real,
la jerarquía única no se sostiene. De ahí la v2.

---

## 2. La propuesta v2 — dos patas, sin jerarquía única

**Qué se queda (intocable)**: techo por nota, trinquete solo-baja, pre-commit
como tripwire, doctrina "pártela, no la mutiles", `/consolida` con humano en el
gatillo, canon+bitácora+archive.

**El principio** (formulado por el diagnóstico, firmado por el abogado): *bytes
como tripwire sí — es lo único mecánicamente chequeable en un pre-commit; bytes
como veredicto, no.* El mismo disparo produjo destilación real en `/consolida`
(`5da6c59`) y desplazamiento puro al cierre de sesión (`09a75eb`).

**La venta honesta** (condición explícita de landscape y abogado): nada de esto
cambia la tasa de reposición — esa tasa es la producción de doctrina de Paul,
exógena, y nadie debe tocarla. El bucle es un GC generacional: la alternativa al
bucle es heap infinito (R0: notas de 89 KB) o programa parado (las vacaciones).
La v2 decide si la pausa es stop-the-world en mitad del cierre (hoy: cinco notas
a <350 B del techo) o compactación programada. Las hijas calientes de la
factorización volverán a su techo — más despacio, más barato, y mordiendo al
compactador en vez de al escritor.

### Pata A — el ciclo (opera el crecimiento distribuido: 21/27 notas)

1. **Guarda de sellado en kbx** (`internal/ratchet`, repo kbx): sellar o bajar
   un techo exige **≥15% de aire** sobre el tamaño actual (techo ≥ 1,15·S).
   Pocas líneas en un gate que ya existe; propuesta del abogado, mecanizada
   (sin guarda es disciplina humana: el sello de la víspera con 1,1% de aire
   causó la mordida del día siguiente). Dimensionado: cores activas ~80-180
   B/día → 3-4 KB ≈ 2,4-7 semanas; la cadencia real de consolida tiene mediana
   de 4,5 días entre pasadas (los huecos de 19-22 días son el outlier).
2. **Evicción editorial nombrada en `/consolida`** (SKILL.md de reflex): "en
   cada pasada, qué párrafo del canon ya no paga su sitio → baja a bitácora".
   Va junto con 1: podar-para-dejar-aire sin criterio de valoración sería
   rotación por orden de llegada en frío.
3. **Semántica del mordisco, escrita en la doctrina**: cuando el gate muerda, la
   salida es partir/rotar/consolidar — **recortar el delta entrante es el
   anti-patrón** (precedentes verificados en primario: RocksDB — el stop espera
   al compactador, saltarse la banda suave es el bug, issue #9423; Letta — sí
   bloquea en write path pero la salida prescrita nunca es mutilar). Nota de
   registro: el día de la auditoría hubo una excepción real (el delta se recortó
   dos veces para caber) — esta regla la prohíbe hacia delante.
4. **Sincerar la doctrina en `core-index`**: el sistema real es "techo por nota
   + trinquete"; los nominales de tier quedan como default para notas nuevas
   (4 de 5 cores viven de waiver — describirlos como sistema es ficción).
5. **Backlog**: sin cambio de tier ni exclusión (cambiarlo pide código que no
   compensa — diagnóstico); conserva su techo 30.000. Lo que cambia es el
   remedio prescrito al morder: **cerrar/archivar frentes, no destilar**.
   Registro honesto: arqueología votó NO a llamarlo "dashboard acotado" (su
   serie muestra acreción-hasta-techo con re-pegado el mismo día, la más rápida
   de la KB); el remedio propuesto es compatible con ambas lecturas.

### Pata B — la factorización (opera los 2 imanes-área)

6. **`doctrina-agentes`** (80% del crecimiento en headings sin relación): partir
   **por tema** — cada capítulo-concepto a nota propia con título-afirmación; la
   nota queda como **índice corto, puerta única de routing** (mitigación
   obligatoria del riesgo de routing nombrado por el abogado: sin índice, la
   partición convierte fricción-de-espacio en fricción-de-routing). La partición
   ya está escrita en sus propios headings, y su sección "capítulos que viven en
   nota propia" prueba que el método ya funcionó una vez.
7. **`desarrollo-agentico`** (65% afines): partir **por género** (narrativa de
   la meta-habilidad vs referencia técnica del harness vs epistemología de
   benchmarks). No es cajón: es tema amplio subdividiéndose. El IOU de la regla
   de los caps ("cuando se partan, esto sube") es demanda suprimida y **se salda
   en esta operación**.
8. **`pragmatismo-y-pivots`**: watchlist, no se toca (12% headings nuevos, bolo
   digerido, 19 días plana, 677 B de aire sin usar). Al próximo mordisco, split
   con la partición que ya tiene lista. **Perfil**: no se toca por tema (0%
   nuevos) — pata A pura: poda por valor de secciones infladas + resellado con
   aire (~20.700). Es el primer caso de prueba de la pata A: si evicción+aire no
   lo maneja sin mutilar, esa pata cojea (landscape).

**Asimetría de rol, firmada por el abogado y reforzada por la Ronda 6**: el
ciclo es la **pata permanente** (entre picos consecutivos el canon cayó −31,4%:
la contención la hace el ciclo); la factorización es un **alivio puntual** de
los imanes — compra mejor forma y mordiscos más baratos, no tiempo (el rebote
medido es de horas: +2.709 B la misma tarde de una poda de 21,7 KB). "Sin
jerarquía única" significa eso: roles distintos, ninguna subordinada.

**Ejecución: no hay orden conceptual, hay UNA pasada** — la próxima `/consolida`
parte doctrina (por tema) y desarrollo (por género), salda el IOU, poda y
resella perfil con aire, y resella lo que esté a ras. Todos los techos nuevos
nacen bajo la guarda (≥15% de aire) y solo bajan: trinquete intacto. Urgencia
honesta tras la Ronda 6: es la pasada que ya tocaba (dos notas a <50 B del
techo y un IOU pendiente), no una emergencia de volumen.

### Fase 2 — la banda, condicional por eventos (no calendario)

9. Criterio compuesto (base del diagnóstico + observable del abogado, que cedió
   su umbral ≥2-de-10): **la primera mordida en caso normal sobre una nota
   sellada-con-aire dispara la banda** (warning en cada commit entre techo y
   techo+15%, bloqueo duro en techo+15% — dual-trigger de RocksDB; coste: una
   comparación + exit code, spec ya escrita en §9 del informe del diagnóstico,
   incluido el fix de la línea 33 del pre-commit que hoy descarta el output en
   éxito). Precisiones que cierran la ambigüedad señalada por arqueología:
   "mordida" = rotación forzada o IOU visibles en commit que toca canon (cero
   métricas nuevas); una mordida sobre techo **pre-guarda** no dispara la banda
   — dispara el resellado de esa nota; los cutovers de arquitectura (tipo M6-04)
   no cuentan como caso normal. Reloj de eventos: sin uso no corre; con uso, el
   primer fallo real dispara. Refinamiento heredado del abogado para cuando la
   banda gane su código: warning **acumulativo** ("N notas en banda desde hace M
   días"), porque la latencia de reparación medida fue ~13 días.

### Recomendación separada (decisión independiente de todo lo anterior)

10. El retrieval **penaliza `archive/` por defecto** (downrank, no exclusión —
    matiz del abogado y del diagnóstico: lo superado marcado sigue valiendo como
    precedente), con opt-in para búsqueda histórica. Magnitud correcta del
    argumento: el archivo **gasta plazas escasas** del bloque de inyección (33%
    de los punteros del primer día de M6-06, 4/12 verificado sobre reflex-log;
    muestra mínima), no suplanta al canon (el vivo gana en score en los pares
    directos medidos: 0,65 vs 0,58).

### Restricciones, comprobadas

§0: se modifica maquinaria existente (guarda en gate existente, líneas de skill,
doctrina), no se construye nueva; la banda solo si gana su código con evidencia.
Trinquete: intacto y reforzado. Cero métricas nuevas: gradiente de headings y
criterio de Fase 2 se leen de git. Disciplina humana nueva: ninguna fuera de
flujos existentes. El coste de la factorización es una pasada de `/consolida`
que de todos modos está pendiente (dos notas a <50 B del techo).

---

## 3. Ratificación

### v1 (histórica — Paul la rechazó después)

Los cuatro firmaron la v1 (abogado, landscape y diagnóstico SÍ a las dos fases y
a la separada; arqueología SÍ factual salvo NO al punto del Backlog-como-
dashboard). El voto más significativo fue el del abogado: *"empecé defendiendo
el statu quo entero y termino firmando una propuesta que conserva sus mecanismos
y jubila su fricción."* La v1 sigue viva dentro de la v2 como Pata A + Fase 2;
lo que el rechazo de Paul añadió fue la Pata B, la honestidad sobre la tasa de
reposición y el criterio por eventos.

### v2 — votos finales

| Voto | v2 dos patas | Fase 2 por eventos | Separada |
|---|---|---|---|
| cons-abogado | **SÍ** ("es mi tesis del GC generacional, con mis dos riesgos nombrados") | **SÍ** (cede su umbral: "una demostración limpia basta") | **SÍ** (downrank) |
| cons-diagnostico | **SÍ** ("retiro la jerarquía; perfil es el caso que me obliga y lo reconozco") | **SÍ** (el trigger es suyo) | **SÍ** (downrank) |
| cons-landscape | **SÍ** ("no como compromiso: es lo que mi marco decía si se le quita el dato falso") | **SÍ** | **SÍ** |
| aud-arqueologia | chequeo factual: sostiene el gradiente, las tasas y la cronología; bloque de números canónicos en §4 | — | 33% confirmado en primario |

Cambios de posición registrados en esta segunda vuelta, todos por el dato:
landscape y diagnóstico retiraron "factorización primero" (se apoyaba en la
concentración retractada); el abogado aceptó tres rebajas sucesivas de su A/B
(180×→78×→~9-21×) y cedió su umbral de Fase 2; el orquestador retractó dos
cifras propias y el anclaje del 78×; el diagnóstico corrigió su
"convergente ⇒ sin fricción" ante el caso perfil.

---

## 4. Números canónicos del expediente

Bloque final de arqueología (Adenda Ronda 6), con convención declarada en cada
número. Cada cifra sobrevivió a la confrontación entre al menos dos mediciones
independientes:

- **Crecimiento del canon, según la pregunta**. "¿Cuánto trabajo de contención
  hay dentro de un ciclo?": +40,4 KB desde el pico del 11-jul (la divergencia
  +40.960 vs +78.899 era 100% fase de la sierra en la baseline — pico vs valle
  del mismo día; confirmada la hipótesis del abogado con precisión del 1,3%).
  "¿El stock está acotado?": la prueba correcta es **cima-contra-cima de ciclos
  consecutivos**, y da **453.261 → 310.994 B: −142.267 B (−31,4%)** entre el
  pico pre-consolida del 3-ago y el pre-consolida de hoy. **Es el dato más
  favorable a "el sistema funciona" de toda la auditoría** — con su cautela:
  n=2 ciclos, y el primer pico arrastra deuda histórica de julio sin gate; hace
  falta un tercer ciclo para saber si es amplitud estable o sigue bajando.
- **Distribución**: dentro del ciclo, 21 de 27 notas con techo crecen y el top-3
  concentra el 53%. Entre picos consecutivos, **los seis "grandes crecedores"
  netean NEGATIVO** (solve-it −70.341, lighthouses −44.456, doctrina −18.789,
  pragmatismo −7.125, perfil −6.362, desarrollo −501) y 19 de ~27 notas están
  bit a bit idénticas. Lo que parecía crecimiento era la mitad ascendente de un
  ciclo que termina más abajo de donde empezó.
- **Factor de contención del muro**: **~9-21×** (anclaje en el mínimo post-poda;
  el 180× y el 78× fueron retirados por errores de anclaje sucesivos, el segundo
  por el propio orquestador enseñando su script). En neto; el bruto en el muro
  fue ~1,9 KB/día activo — el muro no disuade la escritura, **lo que contiene el
  stock es la poda: la contención es el ciclo completo, no el dique** (conclusión
  del abogado, firmada por arqueología; una cifra suya que la ilustraba, 1.863 B
  como "append del 4-ago", estaba mal etiquetada — era el tamaño de la poda del
  triaje — sin efecto en la conclusión).
- **"El régimen nuevo no está probado", cuantificado**: ~4 días activos en
  agosto (vacaciones); la ventana actual lleva **16 commits que tocan canon
  frente a 54** del hueco natural entre consolidas históricas — un 30% de una
  sola pasada. R2 (pre-commit) tiene ~2 días activos de muestra. La vuelta de
  Paul es el experimento.
- **Gradiente de headings** (% del crecimiento en headings nuevos, 11-jul→hoy):
  perfil 0% · pragmatismo ~12% · desarrollo ~65% (afines) · doctrina ~80% (sin
  relación). No lo afecta la convención de fase: sigue siendo el criterio de
  triaje al disparar el tripwire; lo que la Ronda 6 cambia es la **urgencia**
  ("partir doctrina YA" pasa de emergencia de volumen a cuestión de forma).
- **Perfil, doblemente cerrada**: 0% headings nuevos (forma) y −6.362 entre
  picos (magnitud). No se toca por tema; pata A pura.

---

## 5. Lo que queda abierto para Paul

0. **Un dato antes de decidir, que nadie tenía al empezar**: entre los dos
   últimos picos pre-consolida, el canon **cayó un 31,4%**. El sistema no solo
   acota — está reduciendo el stock ciclo a ciclo (n=2, cautela). Tu queja es
   real, pero es del **precio del ciclo**, no de su resultado: "siempre muerde"
   y a la vez está funcionando mejor de lo que ninguno de los cuatro informes
   afirmó. La v2 ataca el precio.
1. **Aprobar o no la v2.** Sabiendo exactamente qué compra: no cambia la tasa de
   reposición (imposible sin cambiar tu producción); cambia dónde aterriza
   (factorización de 2 notas), cuánto cuesta cada ciclo (aire + evicción con
   juicio) y quién paga el mordisco (el compactador, no tú al cierre). El bucle
   seguirá existiendo, más barato y menos frecuente. Si eso no te vale, la
   conversación pendiente no es de mecanismos: es cuánta doctrina nueva por
   semana quieres sostener en canon.
2. **La recomendación separada de archive/** (downrank en retrieval): decisión
   independiente; el dato a favor es de un solo día de M6-06.
3. **El número del aire (15%)**: nadie lo disputó, pero es un número de esta
   casa (dimensionado con la cadencia real de consolida), no del estado del
   arte. Igual que el umbral >50%/<25% del gradiente de headings: 4 puntos de
   datos, revisable.
4. **El experimento real**: tu vuelta al uso normal. R2 tiene ~2 días activos de
   muestra; el criterio de Fase 2 está diseñado para que tu propio uso decida si
   la banda gana su código, sin que nadie mida nada nuevo.
