# Diagnóstico estructural — régimen de presupuestos de la KB kb-demo

Consultor: diagnóstico estructural (Fable). Fecha: 2026-08-22 (segunda pasada,
tras corrección de premisa). Solo lectura; toda cifra verificada contra la KB,
los hooks de reflex, el índice de exo y la spec fundacional.

> **Corrección de premisa a mitad de auditoría**: la primera versión de este
> informe asumía (siguiendo el contexto compartido) que el presupuesto nació para
> proteger el coste de arranque. Es falso, y lo verifica la propia spec
> fundacional (`kb-demo/docs/superpowers/specs/2026-07-03-memoria-v2-design.md`,
> §1): *"El arranque es barato […] ≈1.4k tok. No es el problema."* El presupuesto
> nació contra el coste de **pull** de cores hipertrofiados — "el destilado está
> sepultado dentro del propio fichero" — como **forzador de destilado editorial**.
> Este informe está reescrito sobre la premisa corregida; el veredicto cambia de
> matiz, no de dirección, y se explica por qué.

## Veredicto firmado

**El límite de bytes nació como proxy de una enfermedad concreta — la mezcla de
géneros (principios + 25 bitácoras en un mismo fichero) — y el contrato
canon/bitácora curó esa enfermedad. El límite ha sobrevivido a su correlato: hoy
muerde sobre canon ya destilado, donde no queda destilación disponible al margen,
y por eso su efecto cotidiano ya no es destilar sino racionar — evicción de
conocimiento por orden de llegada, ejecutada en el peor momento del ciclo (pre-commit
al cierre de sesión).** La causa raíz sigue siendo la misma que en la primera
pasada: la KB no tiene mecanismo que decida *qué deja de merecer su sitio en el
canon*, y el límite de tamaño es el sustituto de esa decisión. Como los bytes no
saben de valor, el desalojado es siempre el aprendizaje más nuevo.

El instrumento en sí no es el error. Un límite de bytes es el único disparador
mecánicamente chequeable que existe — la calidad editorial no se puede medir en
un pre-commit; los bytes sí. Como **tripwire** ("esta nota necesita pasada
editorial") es el instrumento correcto y barato. El error es usarlo como
**veredicto**: el bloqueo duro exige respuesta inmediata en el único momento del
ciclo sin capacidad editorial, y la respuesta posible ahí es mecánica —
rotar, aplazar, IOU — nunca destilar.

## 1. Qué vino a resolver el presupuesto, y qué queda de eso

La spec fundacional diagnostica tres males medidos: (1) cores hipertrofiados por
append de secciones fechadas — coste de pull y destilado sepultado; (2)
redundancia estructural (24 notas de sesión en 5 días, re-contextualización
×3,5); (3) doctrina triplicada con drift. El presupuesto + split canon/bitácora +
contratos de escritura atacaban el (1).

Estado actual de cada componente del mal (1):

| Componente | Estado | Evidencia |
|---|---|---|
| Mezcla de géneros (destilado sepultado en su propio log) | **CURADO por el split, no por el límite** | Las notas canónicas de hoy son destilados limpios: bitácoras separadas en `log/`, histórico frío en `archive/log/`. Las 11 waiver tienen estructura de destilado (ver §3). La mordida de hoy cayó sobre `doctrina-agentes` **el mismo día de su split** — canon recién limpiado, nada que destilar al margen. |
| Coste de transporte del pull | **MITIGADO por el engine, no por el límite** | La spec rechazó el RAG semántico como over-engineering ("el cuello de botella es cuánto se carga, no cómo se rankea")… y exo lo construyó después: retrieval por chunks (3.082 trozos), punteros con cap 1.024 en cada prompt, subagentes con cap 2 KB. La nota entera solo se paga en aperturas deliberadas (~250 tokens/KB). El mundo que la spec descartó por futuro llegó, y el razonamiento de coste de la spec quedó superado por su propio proyecto. |
| Densidad de señal dentro del canon | **VIVO — lo único que el límite sigue custodiando** | Sin presión alguna, el canon acrece sin freno (dato duro: `doctrina-agentes` 4.967 B → 45.788 B en tres semanas sin intervención). La presión es necesaria. La pregunta es dónde aplicarla (§5). |
| Contexto de arranque | Nunca fue el objetivo | La spec lo midió y lo descartó en su §1. Hoy sigue igual: `SessionStart` inyecta solo core-index (5.232 B, cap propio 6.144). |

## 2. ¿Es un límite de bytes el instrumento correcto para calidad editorial?

Adjudicación en dos cortes:

**Corte 1 — tripwire vs veredicto.** Como detector, sí: bytes es la única
variable que un gate mecánico puede evaluar, y correlaciona lo bastante con
"esta nota pide revisión" para valer como alarma. Como veredicto, no: la calidad
de la respuesta al disparo depende por completo de *cuándo* se responde. El
mismo disparo produce cosas opuestas según el momento:

- **En `/consolida`** (offline, nota entera sobre la mesa): produce destilación
  real — hoy mismo `5da6c59` partió doctrina-agentes, movió capítulos a nota
  propia y **bajó** el techo 27.000→20.000. El instrumento funcionando.
- **En pre-commit al cierre de sesión** (`/documenta`, prisa, juicio agotado):
  produce desplazamiento — hoy mismo `09a75eb`, dos mordidas: una rotación para
  hacer sitio y la regla de los caps desterrada a bitácora con IOU ("candidato a
  promoción… se queda aquí porque ambas están a 33 B y 49 B de su techo").
  Cero destilación. El instrumento midiendo el síntoma y castigando al contenido
  equivocado.

**Corte 2 — el proxy depende de si el scope de la nota es estacionario.** En
notas de scope fijo (perfil de trabajo: una persona, un tema), crecer más allá de
cierto tamaño sí aproxima dilución — el proxy es razonable. En notas de scope
creciente (doctrina que gana capítulos, proyectos que acumulan historia real),
el tamaño mide acumulación legítima y el proxy castiga el crecimiento sin
distinguirlo de la grasa. El régimen actual aplica el mismo instrumento a ambos
géneros, más a un tercero que no es conocimiento (el Backlog, dashboard de
estado cuyo tamaño es función de frentes abiertos: 27.831 B, 3,3× el nominal core).

**Respuesta firmada**: el sistema no lleva un año midiendo la variable
equivocada; lleva un año **respondiendo al disparo en el momento equivocado**.
Bytes como alarma: correcto. Bytes como bloqueo en caliente: es lo que convierte
la presión de destilado en racionamiento LIFO.

## 3. Granularidad de las 11 con waiver (sin cambios tras la corrección)

- **(a) Nota coherente sobre tema grande — 9 de 11**: las 7 de `projects/`
  (mono-tema por construcción), `Paul - perfil de trabajo`,
  `pragmatismo-y-pivots`. `evidencia-y-divulgacion` además con techo por debajo
  del nominal (10.000 < 12.500 — el trinquete usado bien).
- **(b) Mezcla real — 1 de 11**: `desarrollo-agentico` (narrativa de la
  meta-habilidad + referencia técnica del harness, dos géneros).
- **(c) Error de categoría — 1 de 11**: `Backlog — frentes abiertos` (estado, no
  canon).

El presupuesto no está detectando mala granularidad: pelea contra el tamaño
natural de temas coherentes ya destilados.

## 4. El dato de la banda 92–100%: ni Parkinson ni artefacto del sellado

Verificado y ampliado. Las 14 notas core/stable más grandes, contra su límite
efectivo (techo sellado o nominal):

```
 27831/30000  92,8%  Backlog (waiver)          15680/16000  98,0%  pguerrero-music (waiver)
 19967/20000  99,8%  doctrina-agentes (waiver) 14323/15000  95,5%  pragmatismo (waiver)
 18951/19000  99,7%  desarrollo-ag. (waiver)   13738/14000  98,1%  pguerrero.me (waiver)
 18848/19000  99,2%  agent-solve-it (waiver)   12490/12500  99,9%  kbx (NOMINAL)
 17991/18000 100,0%  perfil (waiver)           12374/12500  99,0%  finanzas (NOMINAL)
 15904/16000  99,4%  lighthouses (waiver)      12331/12500  98,6%  code-intel (NOMINAL)
 15696/17000  92,3%  agent-develop (waiver)    12326/12500  98,6%  ocr-ml-docs (NOMINAL)
```

Las dos hipótesis del orquestador tienen un discriminador limpio, y está en la
columna derecha:

- **"El techo se fijó donde estaba el contenido"** explica solo los sellos de
  amnistía (F1.a se ancló sobre tamaños existentes — banda al 100% por
  definición en t₀). Pero **no puede explicar las cuatro notas a nominal**:
  12.500 es un número genérico de tier fijado *antes* e independientemente de su
  contenido, y esas notas están al 98,6–99,9% igual. La banda re-emerge donde
  quiera que esté el límite, se haya fijado como se haya fijado.
- **Parkinson clásico** (el contenido se hincha hasta llenar el espacio) implica
  que sin límite el contenido sería menor o igual, solo esponjado. El
  contrafactual está medido y lo desmiente: sin enforcement, doctrina-agentes fue
  de 4.967 a 45.788 B — el contenido no crece *hasta* el límite, **presiona más
  allá** del límite y el dique lo recorta. Y lo recortado no es relleno: la regla
  de los caps desterrada hoy es doctrina de primera.

**Lectura firmada**: la banda 92–100% es el aspecto que tiene un dique visto
desde aguas arriba. Presión de crecimiento constante y exógena (la KB documenta
trabajo vivo; llega doctrina nueva cada semana) contra un límite que no reduce la
demanda sino que la desvía — a bitácoras, a IOUs, a Tetris. Tres consecuencias
prácticas:

1. Subir techos no produciría hinchazón (el contenido es real), pero tampoco
   equilibrio (45.788 B en tres semanas sin dique): **quitar la presión no es la
   respuesta**.
2. Mantener el dique donde está tampoco "mantiene tamaños razonables": mantiene
   racionamiento permanente, con cada edición del canon en régimen de suma cero.
3. Lo único que decide *qué* pasa el dique es el orden de llegada. Ese es el
   agujero: presión sin valoración.

Corolario que re-usa la doctrina más fresca de la propia KB (la desterrada hoy):
"si un cap muerde en el caso normal, el bug está en el formato o en el nacimiento
del cap — se arregla una vez". Este cap muerde en el caso normal — dos veces en
un solo `/documenta` — y lleva meses arreglándose "cada vez que duele" (11
waivers, 22% de las notas con techo). Por su propia regla, toca arreglarlo una vez.

## 5. La pregunta incómoda: el olvido (sin cambios)

El borrado no existe ("Regla de oro: nada se borra" — `/consolida`; 2–3 borrados
en toda la historia git, todos errores de categoría o tooling). Y archivar no es
olvidar: `archive/` está plenamente indexado (59 de 145 notas del índice de exo).
Para una KB personal con retrieval por chunks eso es defendible — lo superado se
marca, no hace falta borrarlo. **El hueco no es evicción de la KB: es evicción
del canon** — la operación "este párrafo ya no paga su sitio, baja a bitácora"
ejecutada como valoración con tiempo, no como hacer-sitio bajo presión. El
presupuesto es el sustituto pobre de esa operación.

## 6. Propuesta (reforzada por la premisa corregida)

Si el objetivo del presupuesto siempre fue calidad editorial — y la spec lo dice —
entonces debe morder donde hay juicio editorial disponible. La corrección de
premisa no debilita la propuesta original: la vuelve tautológica.

1. **Mover la mordida del write path al ciclo de `/consolida`.** Pre-commit:
   warning y deuda anotada dentro de una banda de tolerancia (p. ej. techo +15%);
   bloqueo duro solo para desbordamiento grosero. `/consolida` — que ya chequea
   presupuestos y ya demuestra que el disparo en frío produce destilación real
   (`5da6c59`) — salda la deuda decidiendo *qué* baja, no solo *cuánto*.
2. **El trinquete se queda tal cual.** Techos solo bajan, sellados, `--staged`.
   Es la parte que funciona.
3. **Backlog fuera del régimen de conocimiento.** Es estado; su tamaño lo
   gobiernan los frentes abiertos. Regla propia o ninguna.
4. **Sincerar los nominales por género, no por tier.** La banda 92–100% sobre
   nominales genéricos (§4) demuestra que 12.500 es tan dique como los sellos.
   Nacimiento correcto según la regla de los caps: peor caso por construcción
   del género (nota de proyecto maduro ≠ destilado de learning ≠ nota de
   identidad), no un número redondo por tier.
5. **Nombrar la evicción de canon en `/consolida`**: una línea — "en cada
   pasada, pregunta qué párrafo del canon ya no paga su sitio y bájalo a
   bitácora" — convierte la rotación de hacer-sitio en valoración.

**Trade-off explícito.** Aflojar el gate de escritura arriesga acreción entre
pasadas de consolida — y el contrafactual de 45.788 B demuestra que la acreción
sin presión es real y rápida. Por eso la propuesta no quita el dique: le añade un
aliviadero con deuda registrada. La banda de tolerancia acota el exceso (+15% de
un techo de 20.000 son 3.000 B ≈ 750 tokens en las aperturas deliberadas que
ocurran); el trinquete impide que la banda se convierta en subida encubierta; y
consolida corre de verdad. El riesgo contrario — dejar el bloqueo en caliente —
tiene coste demostrado hoy: doctrina fresca fuera del canon por 33 bytes.

## 7. Ronda de síntesis — respuesta al A/B del abogado y al flanco §0

**Concesión (con precisión).** Concedo que la marginalidad del coste de lectura
entera es un logro del dique, no una propiedad del retrieval. Mi §1 lo
sobreatribuía al engine: las vías de *transporte* sí son independientes del
tamaño (punteros cap 1.024, subagentes cap 2 KB — eso se sostiene), pero la
unidad de consumo en apertura deliberada es la nota entera (exo search devuelve
punteros a nivel nota), y que esa apertura cueste ≤27,8 KB hoy lo garantizan los
techos. La concesión no toca la propuesta: la banda +15% conserva la cota por
construcción (peor caso: doctrina 23.000 B, Backlog ~32 KB) — el logro se queda.

**El A/B de julio, desmontado con cronología.** El régimen de julio no fue
"warning ignorado": fue **señal sin canal de entrega en el camino de escritura**.
Verificado:

- `kbx budget` nace el 2026-07-11 (`5cc755c`, repo kbx). Su único consumidor era
  `/consolida` — `/documenta` no lo ejecutaba en julio (la regla de presupuesto
  entra en su SKILL.md con el régimen post-gate).
- Pasadas de consolida: 07-09, 07-11 (×2), 07-12… y **nada hasta el 08-03**:
  22 días sin que nadie ejecutara el disparador. El +77,7 KB de agent-solve-it
  cae dentro de ese hueco. `doctrina-agentes` en el mismo hueco: 18.256 B
  (07-18) → 45.788 B (08-02), picos de ~1.800 B/día.
- Y cuando el disparador por fin corrió (08-03), **el warning funcionó**:
  `b3df97c`, split de 9 notas obesas en una pasada.

El A/B del abogado compara "sin disparador en el write path + colector ausente
22 días" contra "bloqueo en cada commit". La banda es un tercer régimen distinto
de ambos: warning **entregado en cada commit** (el pre-commit lo imprime — canal
que julio no tenía) + stop duro a techo+15%. El modo de fallo de julio — deriva
ilimitada — es imposible por construcción: cota de 3.000 B por nota (15% de
20.000) contra los 77.700 B medidos. Ratio 26:1. Lo que no está medido, y lo
digo: que un warning entregado se atienda. Lo que sí está garantizado: la cota.

**Flanco §0 — acepto la síntesis, con orden y aritmética.** El fix del abogado
(al partir, podar hasta dejar 15–20% de aire) va primero: cero código. Su
aritmética, medida sobre la nota más caliente: doctrina recreció 24.131 → 26.999 B
en 16 días bajo bloqueo ≈ **180 B/día netos**; un headroom de 3–4 KB compra
**2–3 semanas de aire**, comparable a la cadencia real de consolida (huecos
medidos: 22 y 19 días). Suficiente en régimen normal, justo en semanas calientes.
Dos condiciones:

1. **El headroom presupone la valoración.** "Podar hasta dejar aire" ES decidir
   qué párrafo ya no paga su sitio — el fix del abogado necesita mi punto 5
   (evicción por valoración nombrada en la skill) o la poda será otra vez
   rotación por orden de llegada, ahora en frío. Van juntos.
2. **La banda queda en reserva tras criterio falsable** — el patrón de la propia
   spec fundacional (RAG: "criterio de reapertura"): si tras dos pasadas de
   consolida con headroom, `/documenta` vuelve a morder en el caso normal
   (rotación forzada o IOU para que quepa un delta — observable en commits, sin
   métricas nuevas, §0-safe), la banda gana su código. Que es poco: `budget` ya
   computa tamaño contra límite; la banda es una comparación y un exit code
   distinto, y la "deuda anotada" ni hace falta — el IOU "candidato a promoción"
   ya existe como convención y consolida puede grepearlo.

## 8. Ronda 3 — adjudicación del test del título (landscape vs mi 9-de-11)

La contradicción es parcialmente aparente: medíamos ejes distintos. Mi 9-de-11
era **cohesión transversal** (¿mezcla temas hoy?); el del landscape es
**dinámica de convergencia** (¿dejará de crecer alguna vez?). Una nota puede ser
cohesiva Y área. Adjudico con el criterio operativo del coordinador (converge si
el tamaño se estabiliza al madurar el tema), medido en git a cuatro fechas
(07-15 / 08-03 / 08-12 / 08-22):

| Nota | Dinámica medida | Dictamen |
|---|---|---|
| doctrina-agentes | 8.493→26.840→26.840→19.967; dos splits, recrece siempre que puede | **ÁREA genuina** — título-cajón, demanda perpetua |
| desarrollo-agentico | 12.916→18.951, luego clavada 19 días… con IOU pendiente (la regla de los caps, hoy) | **ÁREA genuina** — quieta porque está BLOQUEADA, no porque convergió; la demanda existe y está desviada |
| Backlog | 31.528→29.999→29.999→27.831 — **decrece** al cerrar frentes | **ESTADO** — ni área ni evergreen; el test del título no aplica; su regulador es el ciclo de frentes |
| Paul - perfil | 9.407→17.779→17.991 (+212 B en 19 días) | Converge — entidad acotada que edita en sitio, deriva lenta |
| evidencia-y-divulgacion | nace hoy del split, techo BAJO nominal | Converge — título de afirmación, el testigo Matuschak-compliant |
| pragmatismo-y-pivots | 10.815→14.323, quieta 19 días con 4,7% de aire | Converge |
| lighthouses-bot | 36.626→15.904, quieta total desde el cierre | **Converge — el caso testigo del criterio**: proyecto cerrado, 99,4% y quieta |
| agent-solve-it | 6.128→18.848, quieta (proyecto en pausa) | Área **acotada por vida del proyecto** |
| agent-develop | 12.359→15.696, quieta, 92,3% | Ídem |
| pguerrero-music | 12.279 quieta en julio, +3,4 KB en su ráfaga de agosto | Ídem — crece a ráfagas mientras vive |
| pguerrero.me | 12.344→13.738, cuasi-quieta | Ídem |

**Dictamen: el landscape tiene razón en 2 notas de 11** (doctrina-agentes y
desarrollo-agentico — cajones de demanda perpetua), más el Backlog por una razón
distinta de la suya (es estado, no un título mal factorizado). **En las 7 de
projects/ el área tiene condición de parada** — la vida del proyecto — y
lighthouses lo demuestra empíricamente: cerrada, pegada al techo y quieta. Su
tesis "el 22% de waivers son títulos mal factorizados" no sobrevive a la
medición: 7 de 11 son notas de proyecto con convergencia demostrada al cierre.

Para las 2 áreas genuinas, la banda + evicción no basta a largo plazo: piden
**factorización continua** — el capítulo maduro sale a nota propia con título de
afirmación (exactamente lo que consolida hizo ayer con [[Fallo silencioso]]), y
el cajón queda como spine/índice. El modelo correcto de una nota-área no es
"nota que converge" sino **bomba de dos tiempos**: lo nuevo aterriza en el spine,
madura, y consolida lo bombea a nota propia. La banda dimensiona la cámara; la
pasada de consolida es la carrera de vaciado. Ese criterio de corte (por
capítulo-afirmación, no por tamaño) entra en la cláusula, punto 4.

## 9. Cláusula firmable — régimen de presupuestos v2

Tres cambios de posición míos respecto a rondas anteriores, declarados: (i)
abandono el escalonado "banda solo tras criterio falsable" — el abogado convergió
("me vale cualquiera de las dos") y el código quedó minimizado, así que la banda
entra directa: mecánica > disciplina (restricción 3); (ii) el Backlog no sale
del régimen — un tier inventado falla como NOTIER en `budget.go` y una exclusión
per-fichero pide código que no compensa; (iii) los nominales no se suben — el
sello per-nota YA es la calibración por género.

> **CLÁUSULA (SÍ/NO):**
>
> 1. **Gate (pre-commit, mecánico).** Para cada nota core/stable con tamaño S y
>    límite efectivo L (sello de `.kbx-ratchet.json` vía `kbx_budget_max` si
>    existe; si no, nominal del tier — core 8.500, stable 12.500):
>    S ≤ L → pasa · L < S ≤ 1,15·L → pasa **con warning impreso** (nota, S, L,
>    exceso) · S > 1,15·L → **bloquea**, sin negociación, como hoy.
>    `kbx ratchet --staged` queda EXACTAMENTE como está: subir techo, borrar
>    sello o reclasificar para escapar sigue en rojo. **El trinquete no se toca.**
> 2. **Nacimiento de techos (guarda del abogado, mecánica).** kbx rechaza crear
>    o bajar un sello que quede a <15% de aire sobre el tamaño actual
>    (techo ≥ 1,15·S al sellar): poda primero, sella después. Prohíbe el caso de
>    anoche — sellar 20.000 con la nota a 19.783 (1,1% de aire), que es la causa
>    directa de la mordida de hoy.
> 3. **Deuda: sin registro nuevo.** La deuda es el warning de banda (impreso en
>    cada commit y listado por `kbx budget` como `banded`) más la convención IOU
>    existente ("candidato a promoción" en bitácoras), grepeable. Ni ficheros ni
>    métricas nuevas.
> 4. **/consolida, dos reglas de texto (cero código).** (a) Al abrir la pasada:
>    `kbx budget` + grep de IOUs; toda nota en banda se salda en esa pasada,
>    podando o partiendo hasta S ≤ L con ≥15% de aire (coherente con 2).
>    (b) Criterio de corte: se parte por **capítulo con título-afirmación**
>    (test del título), no por tamaño; lo que baja del canon baja por **valor**
>    ("¿qué párrafo ya no paga su sitio?"), no por orden de llegada. El Backlog
>    se poda cerrando/rotando frentes, no destilando.
> 5. **/documenta.** Si su commit sale con warning de banda, lo reporta en el
>    resumen final. El rechazo duro sigue sin negociarse. Nada más.
> 6. **Backlog y nominales: sin cambio de mecanismo.** Backlog conserva
>    tier core + sello 30.000, gobernado por ciclo de frentes (decrece solo:
>    31.528→27.831 este mes). Nominales de tier quedan como default de notas
>    nuevas; una nota que desborda su nominal entra en banda y consolida decide:
>    podar con aire o sellar techo honesto (con la guarda de 2).
> 7. **Código total que toca esta cláusula.**
>    `kbx/internal/budget/budget.go`: el switch del callback de WalkDir (hoy
>    `size > effective`, estricto) pasa a bloquear en `size > 1,15·effective` y
>    añade lista `Banded` al `Report` (`Failed()` no la mira); `cmd` imprime
>    banded como warning. `kbx/internal/ratchet` (collect.go): la guarda del
>    punto 2 al crear/bajar sellos. Pre-commit de la KB: imprimir el output de
>    `kbx budget` también en éxito cuando trae warnings (hoy lo descarta — línea
>    33). Estimación: 40–60 líneas de Go + tests + 1 línea de hook. **Nada más.**
> 8. **Supuesto operativo declarado, no obligación nueva:** consolida mantiene
>    su cadencia real (~2–3 semanas; huecos medidos: 22 y 19 días). La banda
>    dimensiona ese hueco: 15% de 20.000 = 3.000 B ≈ 16 días al ritmo medido de
>    la nota más caliente (180 B/día). Si la cadencia se rompe, la banda se
>    llena y el bloqueo duro reaparece: **el sistema degrada al statu quo, nunca
>    a julio.**

Un solo número libre (15%) usado en las tres cotas — banda del gate, aire al
sellar, aire al podar — deliberadamente: simétrico, memorizable, sin segunda
constante que calibrar. Yo la firmo: SÍ.

## 10. Ratificación — voto sobre la propuesta conjunta v1

**Fase 1: SÍ.** **Fase 2: SÍ.** **Recomendación separada: SÍ**, con preferencia
por penalizar sobre excluir.

Sobre la banda: la conjunta elige el escalonado de mi ronda 1 (banda condicional
tras criterio falsable) sobre la banda directa de mi ronda 3. Ambas posiciones
son mías y firmo el escalonado sin pataleo: la guarda mecánica de sellado ya
satisface la restricción "nada que pida disciplina" en el punto crítico (cómo
nacen los techos), la diferencia práctica entre ambas variantes es una pasada de
espera (~3 semanas), y bajo §0 gana la que difiere código. El criterio de
disparo de Fase 2 es literal al que propuse; me obliga.

**Adjudicación notas-área (alcance del punto 3)**: 2 de 11 son áreas genuinas
(doctrina-agentes, desarrollo-agentico — títulos-cajón con demanda perpetua);
las 7 de projects/ son áreas con condición de parada (lighthouses: cerrada,
99,4%, quieta — convergencia demostrada al cierre); Backlog es estado; perfil,
evidencia y pragmatismo convergen. El test del título *solo al partir* tiene el
alcance correcto: no toca las notas de proyecto coherentes y muerde exactamente
donde debe.

Dos matices no bloqueantes, para el acta:

1. **El punto 7 llama "inactiva" a desarrollo-agentico (0 B/19 días) y no lo
   es: está bloqueada.** Su quietud es demanda suprimida — el IOU de la regla de
   los caps dice literalmente "cuando se partan, esto sube" y apunta a ella o a
   doctrina-agentes. La primera aplicación debe **saldar ese IOU explícitamente**
   (es el caso testigo de todo el diagnóstico); si al partir doctrina el IOU
   quiere subir a desarrollo-agentico, entonces sí urge.
2. **El "~1.800 B/día" del Backlog es churn, no crecimiento neto**: en el mes
   medido el Backlog DECRECE (31.528→27.831). Que el acta no deje implícito que
   crece a ese ritmo — el dato correcto para su cota de dashboard es el neto,
   que es negativo cuando se cierran frentes.

Sobre la separada: el 33% de punteros a frío cuadra con la composición del
índice (59 de 145 notas son archive/, un 41%). Penalizar por defecto alinea el
retrieval con la semántica de tiers ya declarada; excluir del todo haría el
archivo invisible a la vía automática (inyección de punteros) cuando lo superado
marcado sigue valiendo como precedente. Downrank por defecto, exclusión no.

## 11. Ronda 4 — el rechazo de Paul, y las tres preguntas del addendum

(Sobre el mensaje cruzado: mi voto anterior ya respondía — firmé la v1 tal como
está, banda en Fase 2, sin discrepancia. Queda confirmado y ahora superado por
esta ronda.)

### P1 — ¿La v1 resuelve el bucle o lo retrasa? Respuesta literal

**Lo retrasa y lo abarata; no lo resuelve.** La v1 arregla el MOMENTO de la poda
(consolida, no cierre de sesión), el CRITERIO (valor, no orden de llegada) y el
NACIMIENTO de los techos (aire garantizado). No toca la tasa de reposición: la
cámara se rellena al ritmo de producción de doctrina de Paul, que es exógeno, y
ninguna guarda debe tocarlo — reducir la producción sería el único "arreglo" peor
que el problema. **El bucle en las notas-área es permanente mientras sigan siendo
área.** Mi propio modelo de bomba lo decía: una bomba se llena y se vacía; eso no
es avería, es funcionamiento. Lo que la v1 compra es que cada ciclo cueste menos
(sin Tetris de cierre, sin IOUs) y pode mejor. Dicho tal cual, sin venta.

La única palanca de toda la propuesta que toca DÓNDE aterriza la producción (no
cuánta hay) es la factorización: tras partir, lo nuevo cae en capítulos con aire
o crea capítulo nuevo (nota nueva = 0% de su presupuesto), y el cajón queda como
spine-índice. La reposición no desaparece: **se desconcentra**. El dato del
addendum lo explica al revés: el 99,8% del crecimiento cae en 6 notas porque los
cajones concentran — todo el flujo choca contra los mismos 2 techos al 99%.
Honestidad también aquí: el spine sigue recibiendo primero (doctrina recreció
184 B la misma tarde de su split), así que el bucle persiste post-factorización,
pero su amplitud baja de oscilaciones de 7–21 KB a 1–3 KB, y las mordidas de
write-path desaparecen mientras el spine conserve aire.

### P2 (y el punto 2 del coordinador) — pragmatismo-y-pivots, medido

Descompuse el +8.773 por secciones H2 en tres fechas (script en scratchpad):

- **07-11**: 7.205 B — For agents (3.509) + un "Delta 2026-07-11 (campaña
  lighthouses)" de 1.654 B.
- **08-03**: 14.323 B — el delta se convirtió en "Descartar con disciplina: lo
  que enseñó lighthouses (destilado)" (6.779 B) + "Cuándo parar de auditar"
  (919 B) + Observations ×2. **Todo el crecimiento es UN bolo: la pasada de
  consolida del 03-ago destilando una campaña.**
- **08-22**: 14.323 B — **idéntica al byte**. 19 días, cero cambios, con 677 B
  de aire disponible (no es supresión: doctrina tenía 33 B y mordió; esta tenía
  sitio y nadie lo usó). Y el test más fino: hoy hubo doctrina de decisión
  fresca buscando casa (la regla de los caps) y fue a doctrina-agentes /
  desarrollo-agentico, **no aquí**.

Mi criterio pre-registrado ("≥2 H2 nuevos ligados a campañas → área encubierta")
se cumple formalmente, así que concedo la mitad del dictamen: **el título es
cajón** (siempre cabrá una lección de pragmatismo más). Pero la mitad dinámica
falla: la demanda no es perpetua sino **episódica** — un bolo por campaña con
descartes, congelación entre bolos. Eso pide una tercera clase en la taxonomía:

- **Área activa** (doctrina-agentes, desarrollo-agentico): demanda continua,
  3.108 B/día activo sin muro, 4 podas / 4 recuperaciones. **Factorizar YA.**
- **Área latente** (pragmatismo-y-pivots): título-cajón + demanda episódica.
  **Watchlist con trigger**: no partirla hoy (sería campaña de refactor sin
  presión, contra el propio punto 3 de la v1); al próximo mordisco, el split es
  por tema — "Descartar con disciplina" ya es un capítulo-afirmación listo.
- **Convergentes** (el resto): sin cambio.

Las notas a partir ya son **2, no 3**. La contradicción con landscape queda
resuelta por el dato, no por reparto: tenía razón en el título, yo en la
dinámica, y la clase "latente" captura ambas mitades.

### P2 de fondo — ¿volumen o continente? Firmo la reordenación

**Continente.** El canon lleva 7 semanas plano (269.947→268.560 B) y el
crecimiento está concentrado en cajones — es un problema de dónde aterriza, no
de cuánto hay. Firmo la jerarquía invertida: **acción principal = factorizar
doctrina-agentes y desarrollo-agentico** (la partición de doctrina ya está
escrita en sus headings; el IOU desambigua desarrollo-agentico y la primera
aplicación lo salda); guarda + banda quedan como gestión de fricción mientras
tanto y para las notas que no son área. Es lo que mi §8 ya decía ("para un área
genuina, banda+evicción no basta: piden factorización continua") elevado de
nota al pie a titular. Condición heredada de la guarda: los capítulos nuevos
nacen con techo honesto (≥15% de aire al sellar).

### P3 — el criterio de Fase 2, reescrito sin calendario

Mi criterio original ("dos pasadas de consolida") era de tiempo de calendario y
las vacaciones lo rompen — concedido. Propuesta de reemplazo, más simple aún,
por eventos y sin ventana: **la primera mordida en caso normal (rotación forzada
o IOU para hacer caber un delta) ocurrida DESPUÉS de la primera /consolida
post-guarda dispara Fase 2.** Sin contar días ni pasadas: con aire ≥15%
garantizado por la guarda, cualquier mordida en caso normal es la demostración
empírica de que el headroom no basta — que es exactamente lo que la banda
necesita demostrar para ganar su código. Observable en commits (la mordida deja
rastro: rotación o IOU en el diff), cero métricas nuevas, y las vacaciones no lo
tocan: sin uso no hay mordidas ni disparo, con uso el primer fallo dispara.

## 12. Ronda 5 — voto revisado tras la retirada de las dos cifras

Mi firma de "factorización primero" se apoyaba en el dato retirado (99,8% del
crecimiento en 6 notas). Con el dato corregido — top-3 concentra 53%, 21 de 27
notas con techo crecen — el argumento de la desconcentración compra menos de lo
que afirmé: la factorización ataca ~2 de los 5 mayores imanes, y el crecimiento
distribuido que la corrección revela es real y solo lo opera el ciclo v1. Lo
digo sin rodeo: **retiro la jerarquía "factorización primero". Voto: dos patas
sin jerarquía única.**

El caso perfil es el que me obliga y lo reconozco como tal: 0% de headings
nuevos en 6 semanas (estructura convergente por título y por dinámica), y aun
así #4 en crecimiento (+9.853) hasta quedarse a 9 B del muro. Una nota puede
converger estructuralmente y clavar el techo por puro volumen de engorde. Su
remedio no es partir: es exactamente la pata v1 — poda por valor de las
secciones infladas + resellar con aire. Mi taxonomía de ronda 4 (área
activa/latente/convergente) clasificaba bien la ESTRUCTURA pero asumía que
"convergente" implicaba "sin fricción", y perfil lo refuta.

**Qué opera cada pata:**

- **Pata A — ciclo v1** (guarda de sellado, evicción editorial por valor,
  resellado con aire): el crecimiento distribuido (21/27) y las convergentes
  pegadas al techo — perfil como caso tipo, más las 4 notas a nominal al
  98–99%. También los proyectos activos sanos (solve-it +12.720: no se opera,
  converge al cierre; a lo sumo reseal con aire si molesta).
- **Pata B — factorización por tema**: los imanes-área. Doctrina-agentes es el
  caso puro (80% de headings sin relación al título: cajón). Desarrollo-agentico
  entra como caso más suave y con motivo distinto del que yo daba: el gradiente
  (65% nuevos pero AFINES al título) dice que no es cajón sino tema amplio
  subdividiéndose — su split correcto es por GÉNERO (meta-habilidad narrativa vs
  referencia técnica del harness, la mezcla que señalé en ronda 1), y el IOU
  pendiente lo mantiene en "ya". Pragmatismo (12%) queda en watchlist, como
  estaba.

**El gradiente de headings de arqueología es, de propina, el instrumento de
triaje que faltaba** — barato y sin métricas nuevas (se calcula con git al abrir
consolida): ~0% nuevos → pata A (engorde: podar por valor + resellar);
alto % nuevos y afines → mirar mezcla de géneros, split por género si la hay;
alto % sin relación → cajón, pata B. Con eso la elección de pata por nota deja
de ser juicio de gusto.

**Orden real de ejecución: no hay orden — hay UNA pasada.** La próxima
/consolida hace las dos patas juntas: parte doctrina (por tema) y desarrollo
(por género), salda el IOU de la regla de los caps, poda y resella perfil con
aire (≥1,15·S ≈ techo ~20.700), y resella con aire lo que esté en banda. La
jerarquía que descarto es la conceptual; la secuencia operativa es una sesión.

Sin cambios: el trigger de Fase 2 por eventos (§11-P3) no depende de las cifras
retiradas y se mantiene. Y una nota de método para el acta: la cifra retirada
"+163% la KB entera" tampoco debe sustituir a la retirada — KB entera incluye
log/ y archive/, que crecen por diseño (append); la serie relevante para este
expediente sigue siendo el canon con techo, donde el dato firme es 21/27
creciendo con top-3 al 53%.

## Datos de verificación

- Spec fundacional: `kb-demo/docs/superpowers/specs/2026-07-03-memoria-v2-design.md`
  §1 ("El arranque es barato… No es el problema"; el mal era pull + destilado
  sepultado), §3 (RAG rechazado con criterio de reapertura).
- Tiers reales: core 5/89.972 B · stable 43/406.038 B · log 93/1.422.136 B.
- Ratchet: 11 techos, 10.000–30.000 B. Banda: 10 de 11 waiver ≥92% del techo;
  4 notas a nominal ≥98,6% (kbx 12.490/12.500, finanzas, code-intel, ocr-ml-docs).
- Crecimiento sin dique: doctrina-agentes 4.967 → 45.788 B en ~3 semanas
  (verificado por arqueología, contrastado con historia git).
- Inyección: `exo-recall.sh` (solo core-index, cap 6.144), `recall-inject.sh`
  (punteros, cap 1.024), `subagent-inject.sh` (cap 2 KB). Índice: 145 notas,
  59 en archive/, 3.082 chunks.
- Mordidas de hoy: `09a75eb` (rotación + IOU de la regla de los caps);
  destilación en frío: `5da6c59` (split + bajada de techo 27.000→20.000).
- Pre-commit: bloqueo duro sobre snapshot staged (`.git/hooks/pre-commit`, F1.b).
