# Defensa del statu quo — el presupuesto no es el apaño, es el dique

**Consultor adversarial (abogado del sistema actual). 2026-08-22.**

Encargo: construir el mejor caso posible a favor de dejar el régimen de
presupuestos como está, atacar las alternativas previsibles, y conceder
explícitamente si algo no se sostiene. Toda cifra de este informe está medida
sobre la KB real (`~/Documentos/proyectos/kb-demo`, historia git completa)
o sobre el índice del engine (`~/.exo/index.db`, solo lectura). Nada es
razonamiento a priori.

---

## 1. Veredicto en una línea

El mecanismo (techo por nota + trinquete + doctrina "pártela") **no es un
apaño: tiene el A/B más limpio de toda la KB a su favor**. Lo que sí es apaño
—y ahí concedo— es la calibración: los presupuestos nominales de tier son letra
muerta para el canon que importa, y los techos se sellan sin headroom, lo que
fabrica exactamente la fricción de la que Paul está harto.

---

## 2. El contrafactual está medido, no imaginado

La KB contiene su propio experimento natural, con grupo de control y todo.

### 2.1 El A/B del 3 de agosto

Antes del 2026-08-03 el presupuesto existía como *señal* (`kbx budget` avisaba)
pero **no bloqueaba**. Desde F0/F1 (3-ago: `rotate` + `ratchet` + pre-commit
que juzga el index staged), bloquea. Curva de `projects/agent-solve-it.md`,
medida commit a commit:

| fecha | tamaño | régimen |
|---|---|---|
| 2026-07-11 | 6.128 B | señal no bloqueante |
| 2026-07-17 | 6.128 B | " |
| 2026-07-25 | 29.407 B | " |
| 2026-08-02 | **83.856 B** | " |
| 2026-08-03 | 18.848 B | split de /consolida + techo sellado 19.000 |
| 2026-08-17 → hoy | **18.848 B, plano** | techo sellado + triaje (pre-commit solo desde el 18-ago, ver §8) |

**+77,7 KB en 16 días (~4,9 KB/día) sin régimen; 0 B de deriva en 19 días con
él.** *(Atribución corregida en §8: el agente causal de agosto es el régimen
instrumento+techo+triaje, no el pre-commit, que nace el 17-ago; y el
aplanamiento de solve-it tiene el confound de que su campaña terminó.)* El Backlog cuenta la misma historia: 14,3 KB (9-jul) → 49,9 KB
(2-ago) a ~1,5 KB/día; desde el sello a 30.000 oscila acotado (23,2 → 26,4 →
30,0 → 27,8 KB): crece, se poda, respira. Eso no es una nota estable por
virtud; es una constricción activa haciendo su trabajo.

### 2.2 El grupo de control vive en la misma KB

El tier `log` no tiene límite. Resultado: 93 notas, 1,42 MB, con máximos de
**98,6 / 94,8 / 91,1 KB**. Los tiers con techo: `stable` topa en 18,8 KB y
`core` en 27,8 (el Backlog, con waiver). La diferencia no es el tema ni el
autor ni la época — es la existencia del límite. Y no es que los logs "estén
mal": son el destino *diseñado* del histórico. Pero muestran, dentro de casa,
qué tamaño alcanza una nota de este autor con estos agentes cuando nada la
frena.

### 2.3 Cuánta basura evitó ya

- `rotate` F0 movió **534.247 B de cola fría** fuera de las bitácoras vivas en
  una sola pasada.
- El split de b3df97c (3-ago) sacó del canon, verbatim: agent-solve-it 89,2→18,8;
  lighthouses-bot 60,4→15,9; doctrina-agentes 45,8→24,1; Backlog 51,8→23,2.
- Huella total de agent-solve-it hoy: 18,8 KB de estado vivo + 98,6 KB de
  crónica archivada = **117 KB que, sin el sistema, serían UNA nota**. Abrirla
  costaría ~25-30k tokens *cada vez*.

Proyección del contrafactual: con las tasas medidas (1,5-5 KB/día por nota
activa) y 48 notas de canon, una KB sin límites desde el inicio tendría hoy su
canon en ~1,2 MB con las notas activas en 60-90 KB. No es especulación: es
exactamente donde estaban solve-it, lighthouses y el Backlog el 2 de agosto,
extrapolado.

---

## 3. Qué compra el presupuesto HOY, con el retrieval ya operativo

La pregunta fuerte del contexto: si el contenido llega por retrieval, ¿qué
resuelve limitar la nota? Respuesta verificada: **el retrieval encuentra, no
adelgaza. La unidad de consumo sigue siendo la nota entera.**

Y hay respaldo documental, no solo mecánico: la premisa del contexto compartido
("el presupuesto nació para proteger el coste del arranque") es **falsa**. La
spec fundacional (`docs/superpowers/specs/2026-07-03-memoria-v2-design.md`,
verificado en el fichero) ya midió el arranque y lo descartó: *"El arranque es
barato […] ≈1.4k tok. No es el problema. El coste real está en cada pull: los
cores están hipertrofiados por append […] Cargar un core = tragarse su log."*
El sistema nació contra el coste de pull y contra el destilado sepultado — dos
problemas que el retrieval **no toca** (encuentra la nota; no la hace legible).
"Ya hay retrieval, luego el tamaño da igual" ataca un objetivo que el sistema
nunca tuvo.

1. **`exo search` devuelve punteros, no contenido.** Verificado contra el
   engine real: el JSON trae `permalink`, `ruta`, `score`, `type: "entity"` —
   granularidad de nota. El hook M6-06 (`recall-inject.sh`) inyecta punteros con
   snippets de ~2 líneas bajo su propio cap. El paso siguiente del agente es
   **abrir la nota entera** (Read / `read_note`). Una nota de 89 KB cuesta ~23k
   tokens en ese momento; una de 18,8 KB, ~5k. El presupuesto es lo único que
   acota el precio del punto de uso. M6-06 no debilita este argumento: **lo
   multiplica** — ahora hay punteros en el 86% de los prompts sustantivos, o
   sea más aperturas de nota por sesión, no menos.

2. **La vía de inyección de arranque sigue viva y capada.** `exo-recall.sh`
   mete el `core-index` entero (5.232 B hoy) bajo un cap duro de 6.144 en cada
   sesión, con log de `truncated` porque pasarse = degradación silenciosa. El
   presupuesto core es carga estructural *hoy*, no una reliquia del mundo
   pre-retrieval.

3. **Señal/ruido dentro del canon.** Los embeddings van por chunks (3.082
   `trozos` para 145 notas), pero los chunks resuelven *encontrar*, no
   *vigencia*: una nota-crónica con 25 arcos fechados produce 25 chunks que
   matchean "agentes" con la misma confianza, y el retrieval no sabe cuál es el
   estado actual. El régimen canon-bajo-techo garantiza que todo chunk que
   venga de canon es estado vivo. Sin él, el retrieval devuelve pasado con cara
   de presente — el peor fallo posible para una KB cuya doctrina es
   evidencia-antes-de-afirmar.

4. **La disciplina editorial no es un subproducto: es el producto.** El split
   de b3df97c encontró el Frente 7 partido en 4 secciones, el 9 en 5, "recon
   del torneo" **tres veces**, el fix de ouija dos, la VM cem-runner dos. Eso
   es lo que produce el append sin presión consolidadora: duplicados que se
   contradicen entre sí a medida que el estado evoluciona. El mordisco del
   presupuesto en cada `/documenta` es el *forzador* de la decisión
   canon-vs-historia. Quitar el mordisco no elimina la decisión: la pospone
   hasta que son 300 notas obesas — y la KB ya midió que "posponer" significa
   3 semanas y 89 KB.

5. **La doctrina de esta misma mañana lo respalda.** De la auditoría del cap de
   M6-06, hoy, en `log/doctrina-agentes-bitacora.md`: *"todo cap nacido de una
   medición puntual optimista mordió en días y se parcheó; todo cap nacido como
   presupuesto con mecanismo aguantó o bajó: los 2 KB de A1 sobrevivieron dos
   incidentes sin moverse, el 6144 del arranque nunca se movió, el de kbx solo
   ha bajado."* El sistema de presupuestos de la KB está en la segunda
   categoría — la buena — por el propio criterio destilado de la queja gemela.

### La asimetría del coste

El coste del sistema es **visible, acotado y por sesión**: hoy mordió dos veces
y ambas costaron minutos (una rotación de histórico frío; un aprendizaje
aparcado con marcador de promoción — molesto, no perdido). El coste de
quitarlo es **invisible, compuesto y diferido**: +1,5-5 KB/día por nota activa,
sin ningún evento que avise, hasta que abrir el estado de un proyecto cueste
25k tokens y el retrieval sirva arcos muertos. Un sistema cuyo modo de fallo
es ruidoso e inmediato siempre gana a uno cuyo modo de fallo es silencioso y
acumulativo. Eso es exactamente [[Fallo silencioso]], que la KB destiló como
nota propia *ayer*.

---

## 4. Ataques preventivos a lo que van a proponer

**"El retrieval lo resuelve; los límites son del mundo pre-exo."**
Falso en las tres vías de acceso, verificado: (a) arranque = inyección entera
con cap; (b) search = punteros, consumo = nota entera; (c) recall M6-06 =
punteros, ídem. Para que el retrieval "resolviera" tamaño haría falta lectura
por-chunk en el punto de uso — que no existe, habría que construirla
(maquinaria, régimen §0), y aún así no resolvería vigencia (§3.3).

**"Automatizar la consolidación."**
Lo que hizo /consolida en b3df97c no es compresión, es criterio: distinguir
estado vivo de crónica, detectar que 4 items "perdidos" eran duplicados, decidir
qué es doctrina promovible. Un consolidador autónomo que se equivoca corrompe
**canon** — y lo hace en silencio, que es el único modo de fallo que esta KB ha
jurado no aceptar. Además viola dos restricciones del contexto de una vez:
maquinaria nueva (§0) y disciplina delegada a un proceso que nadie audita.
/consolida como *herramienta con humano en el gatillo* ya existe y ya funcionó
5 veces (5-jul, 11-jul, 12-jul, 3-ago, hoy).

**"Quitar límites y confiar en el olvido/decay."**
No hay decay en exo: habría que construirlo (§0 otra vez). Y decay automático
= mover la decisión editorial de un punto visible (rechazo en pre-commit, con
humano delante) a uno invisible (contenido que deja de aparecer, sin que nadie
decidiera). Es cambiar una molestia auditada por una pérdida silenciosa.

**"Una nota puede ser tan larga como su tema."**
El tema entero YA es ilimitado: tier log sin techo, archive sin techo. El
presupuesto no limita cuánto sabe la KB de agent-solve-it (117 KB y creciendo);
limita **cuánto hay que leer para tener su estado actual** (18,8 KB). Confundir
las dos cosas es exactamente el error que la nota de 89 KB materializaba: un
lector (humano o agente) que quería el estado se comía la crónica reto a reto.
El split no borró un byte — todo fue verbatim a bitácora.

---

## 5. Concesiones — lo que no defiendo

Un abogado que no concede nada es un generador de excusas. Tres cosas no se
sostienen, y las tres son **calibración**, no estructura:

1. **El presupuesto nominal es letra muerta para el canon que importa.**
   4 de las 5 notas core viven de waiver (8.500 nominal vs techos 18-30k);
   en stable, 6 de 43. El "modelo de tres tiers con presupuesto por tier" ya no
   describe el sistema real, que es **techo por nota + trinquete**. Mantener la
   ficción nominal tiene coste: cada conversación sobre "el presupuesto" habla
   de un número que no aplica a nada de lo que duele. Simplificar la doctrina a
   "cada nota canónica tiene su techo sellado, que solo baja" sería más honesto
   y no cambia ningún mecanismo.

2. **Los techos se sellan sin headroom, y eso fabrica la fricción.** Márgenes
   hoy: perfil **9 B**, doctrina-agentes **33 B**, desarrollo-agentico **49 B**,
   lighthouses **96 B**, solve-it **152 B**. Cinco notas a un delta de morder.
   El caso de hoy es de libro: /consolida partió doctrina-agentes por la mañana
   (techo 27.000→20.000, nota en 19.967) y **horas después** el mismo techo
   bloqueó la promoción de un aprendizaje. Sellar el techo al tamaño post-split
   garantiza que el siguiente delta muerde. El fix es interno al sistema: **al
   partir, partir más hondo — dejar ~15-20% de aire bajo el techo nuevo**. No
   se sube ningún techo (el trinquete queda intacto); se poda más al podar.
   Con eso, la fricción por sesión que motivó esta auditoría cae casi a cero
   sin tocar un solo mecanismo.

3. **El peor edge real: el presupuesto puede bloquear doctrina NUEVA, no solo
   histórico.** El aprendizaje transversal de hoy acabó "candidato a promoción"
   en una bitácora. Es el único caso donde el sistema frena señal en vez de
   ruido. Mitigado (el marcador existe, /consolida lo recoge, y de hecho el
   split de hoy era el paso previo), pero es coste real y hay que nombrarlo.
   El fix del punto 2 lo cubre en la práctica: con headroom, la promoción del
   día a día cabe y el mordisco queda para el crecimiento sostenido, que es a
   quien debe morder.

**Lo que se salva entero** (y defenderé contra cualquier propuesta de los
otros tres): el techo por nota, el trinquete solo-baja, el pre-commit sobre el
index staged, la doctrina "pártela, no subas el techo ni la mutiles", y
/consolida con humano en el gatillo. Cada pieza tiene evidencia de trabajo
real; ninguna tiene un sustituto que no sea o maquinaria nueva o degradación
silenciosa.

---

## 6. Segunda vuelta — el dato del 92-100% y qué lectura sostiene la evidencia

El orquestador trajo dos datos del arqueólogo contra este informe, y la
pregunta correcta: 11 de las 12 notas grandes de canon viven entre el 92% y el
100% de su techo (confirmado por mi propia tabla de márgenes: perfil 99,95%,
doctrina 99,8%, desarrollo 99,7%, lighthouses 99,4%, solve-it 99,2%…). ¿Eso es
"el techo es lo único que frena el crecimiento" o "el instrumento no destila,
solo tapa"?

Fui a la curva completa de `core/doctrina-agentes.md` para discriminarlo
(33 puntos, `git show` commit a commit):

- **Sin techo bloqueante, la consolida manual no basta.** La consolida del
  11-jul la bajó a 8.493 B. Tres semanas después estaba en 45.788 —
  **+1,8 KB/día con dos consolidas intermedias en la ventana**. El bucle
  "crece → se parte → vuelve a crecer" que documenta el arqueólogo es real,
  pero su primera vuelta demuestra justo que el destilado *sin muro* no se
  sostiene ni haciéndolo a mano.
- **En el muro, el crecimiento neto cae dos órdenes de magnitud**: 26.840 →
  26.999 en 16 días (~10 B/día neto), con contenido fluyendo por dentro —
  la rotación de hoy (12.398→11.982 en el destilado de exo) es un delta que
  entró *desplazando* histórico frío, no apilándose encima. (Cautela honesta:
  agosto tiene ~5x menos commits que julio, así que parte de la caída es
  actividad; pero ni un 5x explica pasar de 1.800 a 10 B/día.)
- **En el muro, el techo baja.** Hoy: split y 27.000→20.000. Un tapón puro no
  produce eso; un forzador de destilado sí.

Veredicto sobre las dos lecturas: **la evidencia sostiene una tercera, más
precisa que ambas.** El sistema no produce "notas razonables" que convergen a
un tamaño natural bajo el techo — eso no pasa y no va a pasar, y quien lo
prometa miente: el estado estacionario es una sierra (acreción hasta el muro,
desplazamiento en el muro, split que baja el muro). El 92-100% es la foto de
esa sierra tomada justo antes del diente, no un fallo: una nota clavada en su
techo con rotaciones fluyendo a través es exactamente lo que "canon = estado
vivo acotado" promete. Lo que el sistema garantiza es **coste de pull acotado y
triaje forzado** — que es lo que la spec fundacional pedía — no notas pequeñas.
Si el objetivo declarado fuera "notas destiladas por debajo de la presión",
concedería el punto entero; como el objetivo documentado es el pull y el
destilado sepultado, el instrumento hace lo que dice su partida de nacimiento.

Dos concesiones nuevas que esta segunda vuelta obliga a firmar:

4. **La amnistía mancha el relato del trinquete.** Los 9 waivers de agosto
   subieron los techos efectivos de golpe (doctrina: nominal 8.500 → sellado
   27.000, +218%) antes de anclar el "solo baja". Es verdad que el ancla se
   puso donde ya estaba el agua (el nominal nunca se aplicó bloqueante a esas
   notas), pero también es verdad que la doctrina "pártela, no subas el techo"
   se estrenó subiendo techos una vez. El invariante "solo baja" es real pero
   **joven**: 19 días y una sola bajada (la de hoy). Como evidencia, es una
   dirección, no un historial.
5. **El enforcement tiene agujeros — dato que encontré yo y que ningún brief
   me dio**: el 4-ago doctrina-agentes se commiteó en **28.703 B, 1,7 KB por
   encima de su techo sellado**, y estuvo 13 días en brecha hasta el triaje
   del 17-ago ("tres notas core vuelven bajo su techo sellado").
   *(Corrección de ronda 3: ese día el pre-commit bloqueante NO existía — nace
   el 17-ago, ver §8. La brecha no fue un salto del dique: fue el régimen
   sin dique operando. La lectura correcta está en §8.)*

Nada de esto toca el argumento central (la asimetría del coste y el A/B del
3 de agosto siguen en pie, y la curva de doctrina los refuerza), pero un
informe adversarial que esconda la brecha del 4-ago o la juventud del
trinquete sería propaganda, no contrapeso.

---

## 7. Ronda de síntesis — dos concesiones más y la forma final de la defensa

El coordinador trajo mediciones propias contra §3 y contra mi fix de headroom.
Verifiqué la que pude reproducir (aperturas de nota entera en los transcripts:
44 `read_note`, 41 entre 24-jul y 3-ago, **3 desde el 4-ago** — su dato
aguanta) y respondo:

1. **Concedo el transporte para las 4 notas core que no son core-index.**
   `exo-recall.sh` inyecta solo `core/core-index` (5.232 B, cap 6.144, 61% de
   uso, cero fricción histórica). Doctrina-agentes, perfil, Backlog y
   desarrollo-agentico no se inyectan nunca: su nominal de 8.500 B no tiene
   justificación de transporte, y nunca la tuvo — la spec fundacional los capó
   por coste de *pull*, no de arranque. La frase de §3.2 queda restringida a
   core-index.

2. **Concedo que "M6-06 multiplica las aperturas" es predicción, no dato.**
   El 86% viene del gate léxico de M6-06 (propiedad medida del artefacto
   normativo `gate-artefacto.py`, citada en `recall-inject.sh`): mide frecuencia
   de *inyección de punteros*, no de aperturas. M6-06 se mergeó hoy; si los
   punteros producen más aperturas enteras es hipótesis a verificar, no
   evidencia. Y a las magnitudes medidas post-split (~0,3 aperturas/día, todas
   ≤27,8 KB), **el coste marginal de lectura de una banda del 15% sobre el
   techo es ruido** (~750 tok × 0,3/día). Lo concedo sin pelea porque no toca
   la defensa: el valor del muro nunca fue el 15% marginal de lectura, sino que
   exista un muro *en alguna parte* que dispare la sierra de §6. Lo único que
   no concedo es la vuelta a la señal sin bloqueo en ninguna cota: eso es el
   régimen pre-3-ago, medido en +1,8 KB/día. Bloqueo duro en techo+15% con
   warning en banda ≡ mi headroom por el otro lado; ambas conservan todo lo
   que defiendo.

3. **Acepto que mi fix de headroom, tal como lo escribí, es disciplina humana
   — y murió el mismo día que se escribió** (el split de hoy dejó 217 B de
   aire; el triaje del 17-ago liberó 1.863 B y se rellenó en 48 h). Sin check,
   es exactamente el tipo de solución que las restricciones prohíben. La forma
   con mecanismo: **`kbx` rechaza sellar un techo con <15% de aire sobre el
   tamaño actual de la nota** — una guarda en `internal/ratchet`, dentro de un
   gate que ya existe; modifica maquinaria, no la construye. Expectativa
   honesta, acotada por el dato del relleno en 48 h: 15% de 20.000 son ~3.000 B
   ≈ días-a-semanas de escritura caliente. El headroom no elimina la sierra
   (ni debe): alinea la frecuencia del mordisco con la cadencia de /consolida
   en vez de con cada sesión — que es exactamente la fricción que motivó esta
   auditoría.

**Forma final de la defensa tras tres rondas.** Sobrevive, con evidencia: el
muro en alguna cota (dispara la sierra: desplazamiento + splits que bajan
techos), el trinquete (joven, pero con dirección correcta), el pre-commit
staged (con su triaje como red para las brechas), /consolida con humano en el
gatillo, y la asimetría del coste como argumento de fondo. Concedido, con
evidencia: el nominal por tier (letra muerta), el transporte como
justificación para todo lo que no sea core-index, el sellado sin aire (fabrica
la fricción y requiere mecanismo, no prescripción), la magnitud actual del
coste de pull (pequeña post-split — el split la mató, lo cual es punto a favor
del sistema y a la vez achica mi argumento de coste), y que "M6-06 multiplica"
era un pronóstico vestido de dato.

---

## 8. Ronda 3 — la cronología re-fecha el A/B, y la firma de la cláusula

El coordinador trajo una cronología que verifiqué en primario y **acepto
entera** (con un día de matiz a mi favor que no cambia nada):

- `kbx budget` (la herramienta que mide): **11-jul** (`5cc755c` en kbx). ✓
- Trinquete: **3-ago** (`41b3959`). ✓
- Pre-commit bloqueante (F1.b): **nace el 17-ago** en agent-develop
  (`3aebd5d`, "pre-commit hook del gate — juzga el index, no el disco"),
  entra al monorepo el 18-ago (`765711e`), symlink instalado en la KB el
  18-ago 22:39. ✓

Consecuencias, sin anestesia:

**a) Mi objeción de §6 ("consolidar a mano no basta") queda re-fechada y
reducida.** Las cuatro consolidas de julio corrían sin instrumento de medición
(kbx budget llega el 11-jul), sin techos sellados y sin muro. Lo que mi dato
prueba es "consolidar **sin presión ni instrumento** no sostuvo el destilado" —
no "consolidar no basta". Contra la propuesta convergente (que conserva
presión, banda y trinquete) esa objeción no dispara. Concedido.

**b) Mi A/B del 3-ago, tal como lo escribí, está mal atribuido.** "0 B de
deriva en 19 días con pre-commit bloqueante" — 15 de esos 19 días el
pre-commit no existía. Lo que mantuvo el orden del 3 al 17-ago fue el régimen
instrumento + techos sellados como referencia + triaje ex-post (+ el confound
de actividad: solve-it se aplanó también porque su campaña terminó). Lo que
queda al re-fechar es una tabla de **tres regímenes**, más honesta y aún útil:

| régimen | ventana | deriva medida (doctrina-agentes) |
|---|---|---|
| R0 — nada | 11-jul→2-ago | **+1,8 KB/día sostenido** (8.493→45.788, con 2 consolidas pre-instrumento en la ventana) |
| R1 — instrumento + techo sellado + triaje ex-post | 3-ago→17-ago | brecha del tamaño de una sesión: **+4,6 KB en el único día de escritura activo** (24.131→28.703, 4-ago, ritmo julio), reparada con **13 días de latencia** |
| R2 — + pre-commit bloqueante | 18-ago→hoy | 4 días de vida: dos mordiscos, cero brechas, y el coste es exactamente la fricción de la queja |

El dato nuevo que esta re-datación destapa (y que refuerza lo que queda de mi
caso): en R1, la única sesión que escribió en la nota produjo un append de
ritmo julio que atravesó el techo sellado sin que nada lo parase. El techo
sin write-path no frena la escritura: solo deja constancia para el triaje.
**La sierra de §6 sigue siendo real, pero su motor en agosto fue el triaje y
/consolida, no el pre-commit** — que lleva 4 días y cuya aportación marginal
medida es acotar la latencia de reparación de ~13 días a 0, a cambio de la
fricción de cierre.

**c) ¿Qué aporta entonces el write-path que el ciclo triaje/consolida no
aporte?** Una sola cosa, pero no es decorativa: **es el único punto de
detección que corre sin que un humano decida lanzarlo.** El triaje del 17-ago
ocurrió porque alguien lo hizo; la cadencia real de /consolida fue irregular
(5-jul, 11-jul, 12-jul, 3-ago, 22-ago — huecos de 3 semanas). Un sistema cuyo
único enforcement es un compactador lanzado a mano es "más disciplina humana",
que es justo lo que la restricción §3 prohíbe como solución. El pre-commit
corre solo, dentro de un evento (el commit de /documenta) que ya ocurre. Eso
es lo que defiendo del write-path — la detección automática y la cota dura —
no el veredicto en el momento del cierre.

**d) Firmo la cláusula: "bytes como tripwire sí, bytes como veredicto no."**
La evidencia del expediente la sostiene limpiamente: el mismo disparo produjo
destilación real en /consolida (`5da6c59`: split + techo 27.000→20.000) y
desplazamiento puro con IOU en el cierre de sesión (`09a75eb`: rotación +
"candidato a promoción"). El cierre de sesión es mal quirófano; /consolida es
el bueno. Condiciones operativas para mi firma:

1. **Bloqueo duro en alguna cota** (techo + banda, p.ej. +15%): el
   desbordamiento grosero no entra al repo. Sin cota dura es R0, medido en
   +1,8 KB/día. (El append del 4-ago, 28.703 sobre techo 27.000, habría
   entrado *dentro* de la banda con aviso — correcto: brecha declarada con
   IOU en vez de brecha silenciosa de 13 días.)
2. **La cota dura sigue bajo trinquete** (solo baja) y **se sella con aire**
   (la guarda mecánica en `internal/ratchet` de §7.3 — sin ella, el sellado a
   ras reaparece, medido: 217 B de aire el mismo día en que prescribí 15%).
3. **El disparo en banda no exige resolución al cierre**: registra un aviso
   persistente y visible (IOU en la nota o lista de triaje) y difiere la
   destilación a /consolida. El pre-commit deja de pedir cirugía al cierre y
   pasa a pedir cita con el cirujano.
4. **El aviso en banda tiene que ser ruidoso y acumulativo** (la lista de
   "N notas en banda desde hace M días" en la salida del mismo check que ya
   existe — otro exit semantics, no maquinaria nueva). Con una latencia de
   reparación medida en ~13 días, la única red es que la lista se vea.

Con esas cuatro condiciones, lo que defendí desde la ronda 1 queda intacto en
lo esencial: hay muro (en la cota alta), hay sierra (la destilación ocurre
donde se demostró que ocurre), hay trinquete, y desaparece el veredicto al
cierre — que era la fricción que originó la auditoría y que, re-fechado todo,
resulta ser la pieza más joven (4 días) y menos probada del sistema.

---

## 9. Ratificación — voto sobre la propuesta conjunta v1

Verificación final: la brecha de enforcement 11-jul→17-ago no fue descuido
sino calendario deliberado (spec v3, `f64502c` en kbx, 3-ago: "F1.b se activa
el 17 junto con la corrección de esa línea" — activar antes dejaba al agente
con dos instrucciones contradictorias). Con eso, acepto la reformulación final
de mi A/B: **"sin instrumento vs con régimen completo"**, no "warning vs
bloqueo"; y las notas planas lo están por inactividad, no por enforcement. Mi
línea roja (nunca señal sin bloqueo en ninguna cota) sobrevive en la propuesta:
Fase 1 bloquea en el techo con aire garantizado; Fase 2 movería el bloqueo a
techo+15%.

**Voto: SÍ a Fase 1. SÍ a Fase 2. SÍ a la recomendación separada** (con un
matiz). Motivos:

- Fase 1 conserva todo lo que este informe defendió con evidencia (techo por
  nota, trinquete, tripwire, "pártela", /consolida con humano) y arregla
  exactamente lo que concedí (sellado con aire mecanizado en kbx — mi propia
  propuesta de §7.3 —, doctrina sincerada, Backlog reclasificado como
  dashboard, semántica del mordisco escrita). El punto 2 + punto 6 son la
  operacionalización correcta de "bytes como tripwire sí, veredicto no", y
  **la firmo tal como queda**: la evicción nombrada pone el criterio editorial
  donde el gate solo pone el disparo, y el anti-patrón declarado (recortar el
  delta entrante) es justo el desplazamiento-sin-juicio que `09a75eb`
  documentó.
- Fase 2 es la banda que yo mismo acepté en §7.2, pero mejor que como yo la
  pedí: **condicionada a un criterio falsable** (dos pasadas de /consolida con
  la guarda activa; si /documenta vuelve a morder en el caso normal, la banda
  gana su código). Eso es no-construir-maquinaria-hasta-demostrar-necesidad,
  que es la doctrina del propio régimen. Refinamiento menor, no bloqueante:
  cuando la banda llegue, que el warning sea acumulativo ("N notas en banda
  desde hace M días"), porque la latencia de reparación medida es ~13 días y
  un warning puntual por commit se puede normalizar como ruido.
- Recomendación separada: SÍ a penalizar `archive/` por defecto — es coherente
  con el sistema que defendí (el desplazamiento a archive ES una decisión de
  frialdad; darle plaza igual en los punteros escasos contradice esa
  decisión). Matiz: **penalizar antes que excluir duro**, al menos al
  principio — el 33% es dato del primer día de M6-06, y la doctrina destilada
  esta misma mañana en la KB dice qué pasa con los ajustes nacidos de una
  medición puntual.

Nada que objetar que la evidencia no haya zanjado ya. Firma: el consultor
adversarial, que empezó defendiendo el statu quo entero y termina firmando una
propuesta que conserva sus mecanismos y jubila su fricción — que es como debía
terminar si el proceso funcionaba.

---

## 10. Ronda 4 — el rechazo de Paul, el 78×, y la defensa que queda en pie

Paul rechazó la v1 ("consolida ya lo hemos realizado varias veces y siempre
muerde") y su corrección de vacaciones baja mi factor de contención de 180× a
78× por día activo. Respuestas, tras re-medir:

### 10.1 El A/B sobrevive a 78×, pero con la atribución aún más estrecha

Acepto el re-cálculo (mi propia cautela de "5× menos actividad" era el doble de
corta). Y añado una descomposición que lo afila: el "40 B/día activo en el
muro" es un **neto** que compensa el append del 4-ago (+1.863 B en el único
día que se escribió la nota) con la poda del triaje del 17. El bruto de
escritura en el muro fue ~1,9 KB/día activo — **el muro no disuade la
escritura; lo que contiene el stock es la poda**. La contención es del ciclo
completo (techo+triaje+consolida), y eso es exactamente "consolida desahoga,
no estabiliza": techo en el stock, no en el flujo.

Barra de confianza sobre R2 (pre-commit) en régimen de uso normal: **casi
nula como estadística** — 4-5 días de calendario, 1-2 activos, n=2 mordiscos
en un mismo cierre. Todo lo que sé de R2: en su único día activo convirtió
brecha-diferida-13-días en rotación-inmediata+IOU. Anécdota consistente con
el diseño, no prueba. El corolario del orquestador lo firmo: el sistema lleva
un mes sin carga; la vuelta de Paul ES el experimento.

### 10.2 Divergencia de medición con el addendum §4 — resuelta: es la fase de la sierra

Mi reproducción (canon = tier core+stable, paths con `core.quotepath=off`) no
daba el "canon plano": valle-11-jul→hoy = **+122 KB**. La resolución, medida:

| baseline 11-jul | persistentes → hoy | notas nuevas |
|---|---|---|
| **pico** (pre-consolida, `acf9159`) | **+12.029 B** — y las ~37 no-imán **bajan −22 KB netos**; suben 4 imanes (+34 KB) | +63 KB (9 notas) |
| **valle** (post-consolida, `91dd86d`) | +69.501 B | +52,6 KB (7) |

El "+8.671 de doctrina" del addendum cuadra exacto con baseline pico. Ambas
medidas son verdad: pico-a-hoy responde "¿el stock está acotado ciclo a
ciclo?" (sí — y mejor que plano: la mayoría de notas está POR DEBAJO de su
pico de julio); valle-a-hoy mide la amplitud de la sierra (~60-70 KB). Aviso
para la síntesis: **cualquier número de crecimiento del canon depende de en
qué fase de la sierra se ancle la baseline** — quien mida desde la otra fase
"refutará" el plano sin refutar nada. Y la producción nueva real (~50-60 KB
en 6 semanas) va mayormente a notas nuevas, no a engordar las viejas.

### 10.3 P1, literal: la v1 NO resuelve el bucle — y no debe venderse como si lo hiciera

Con producción de doctrina sostenida y stock acotado, morder periódicamente es
aritmética, no un defecto: las tres opciones existen y están medidas en este
expediente — stock sin límite (R0: 3.108 B/día activo, notas de 89 KB),
producción cero (las vacaciones: único período sin mordiscos), o bucle. La v1
cambia tres cosas del bucle, y solo esas: **coste por ciclo** (cirugía al
cierre → cita con /consolida, aire garantizado al sellar), **frecuencia en el
caso normal** (de "cada delta a nota a ras" — cinco notas a <350 B — a "cada
2-5 semanas por nota caliente"), y **calidad de la poda** (evicción nombrada).
No cambia la tasa de reposición. La frase honesta para Paul: *"sí, muerde;
con la v1 muerde menos veces, más barato y en mejor momento — y el no-morder
tiene dos precios medidos: notas de 89 KB o no aprender nada nuevo"*. En su
analogía: es un GC generacional — pedir que nunca colecte es pedir heap
infinito o programa parado; el tuning solo decide si la pausa es
stop-the-world en mitad del request (hoy) o compactación programada (v1). Y
el dato de 10.2 (persistentes bajo su pico de julio, stock acotado con churn)
es la spec fundacional cumplida: el sistema hace lo que prometió; lo que
estaba mal era el precio del ciclo.

### 10.4 Factorización de las notas-imán como acción principal: sin objeción, con dos riesgos nombrados

La contradicción landscape ("títulos mal factorizados") vs diagnóstico ("notas
coherentes de temas grandes"), resuelta mirando las notas: **ambos tienen razón
en subconjuntos distintos**. Los headings de doctrina-agentes son ~10
capítulos-concepto autónomos (Contrato de memoria, Orquestador limpio,
Recon-first, Cost pyramid, Transporte mecánico, Completitud del brief,
Consulta adversarial, Régimen de gates, Verificación independiente, Mutation
testing) y la nota ya tiene una sección "Capítulos que viven en nota propia" —
se está partiendo orgánicamente y trae su propio índice. Nota-área de libro:
landscape acierta aquí. Pragmatismo-y-pivots es más dudosa: 3 secciones
sustantivas (un destilado de campaña, un criterio, un for-agents) — da para
2-3 piezas, no 10. Los otros 8 waivers (5 proyectos, perfil, Backlog) tienen
contorno natural: diagnóstico acierta ahí. No es "el 22% mal factorizado":
son **dos imanes con título-área**, uno claro y otro parcial.

Sin objeción a elevarlo a acción principal — es "pártela, no la mutiles"
aplicada donde el flujo se concentra. Dos riesgos que la propuesta revisada
debe nombrar:

1. **Routing del próximo delta.** Hoy hay una puerta única para aprendizajes
   de agentes. Con ~10 notas-concepto, cada /documenta elige entre 10 destinos
   más la tentación de nota nueva ("casi nunca", dice el contrato). Mitigación
   barata y necesaria: doctrina-agentes queda como **nota-índice corta** (el
   patrón que su "Capítulos que viven en nota propia" ya inaugura) — puerta
   única para el routing, contenido repartido. Sin índice, la partición
   convierte fricción-de-espacio en fricción-de-routing.
2. **La venta.** Partir no baja la tasa: los +8,7 KB/6 semanas de doctrina se
   repartirán y las 2-3 hijas calientes de cada época volverán a acercarse a
   su techo — en meses, no en días, pero volverá a morder. Si la revisada dice
   "el fix del bucle es partir", el ciclo siguiente desmiente la promesa. La
   formulación honesta: "reparte el flujo y baja frecuencia y coste del
   mordisco en el punto caliente; el bucle sigue siendo el diseño" (10.3).

Sobre retrieval: el riesgo va en dirección contraria a la temida — la
agregación es por nota (entity), diez entidades más específicas mejoran el
matching y abaratan la apertura (2-3 KB vs 20). Lo que sí se pierde es
adyacencia (abrir doctrina daba el sistema doctrinal entero de una vez);
wikilinks + nota-índice la cubren.

### 10.5 El criterio de Fase 2, en unidades activas (P3 del addendum)

Propuesta concreta, observable en commits y sin métricas nuevas: contar
**eventos, no calendario** — "si en los próximos N cierres /documenta que
toquen notas canónicas (observable: commits `docs(kb)` que modifican
core/stable) hay ≥2 con rotación forzada o IOU para hacer sitio a un delta, la
banda gana su código". N=10 cierres cubre ~3-5 semanas de uso real y unas
vacaciones no lo dan por cumplido: sin cierres no corre el reloj.

---

## 11. Ronda 5 — firma de la v2 de dos patas

El orquestador retiró "canon plano" y "concentración 99,8%" (error de método:
prefijos fijos sobre KB reorganizada) y corrigió a: KB entera +163%, 21/27
notas con techo crecen, top-3 = 53%. Me refuerza (hay crecimiento real que
contener), con un matiz que dejo para la reconciliación de números canónicos:
**KB entera +163% incluye log/ y archive/, que crecen POR DISEÑO** — cada
rotación y split mueve bytes del canon hacia allí; ese número mide producción
total, no descontrol. El número que decide política de canon es el de canon, y
mi pico-a-pico (+12 KB persistentes/6 semanas) sigue pendiente de reconciliar
con el 21/27 bajo convención declarada.

**Firmo la estructura de dos patas sin jerarquía única**: (a) el ciclo v1 para
el crecimiento distribuido — es la tesis del GC generacional de §10.3; (b)
factorización solo para doctrina-agentes y desarrollo-agentico, con los dos
riesgos de §10.4 nombrados (nota-índice como puerta única de routing; no
venderla como fix del bucle). El gradiente de headings de arqueología (perfil
0% nuevos → su remedio es el ciclo, no partir; doctrina 80% cajón) confirma
por nota la resolución de §10.4.

**Adenda — el número canónico final del A/B: ~9-21×.** El 78× también cayó
(el anclaje del orquestador cogía el último commit del 3-ago, 26.840, DESPUÉS
del rebote del propio día: la serie real es 24.131→26.113→26.840, **+2.709 B
repuestos en horas** por dos /documenta de esa misma tarde). Mi A/B ha bajado
tres veces — 180× → 78× → ~9-21× — siempre en la misma dirección, y cada
corrección retiró una asimetría de anclaje que favorecía mi tesis: lo firmo
tal cual. Queda contención real de UN orden de magnitud, neta, y atribuible al
ciclo (poda), no a disuasión del muro. Lo que el rebote-en-horas mueve en la
v2: confirma que **el ciclo es la pata permanente y la factorización un alivio
puntual de los imanes** — a este ritmo de reposición, las hijas calientes
volverán a su techo pronto, así que el peso estructural de la propuesta debe
caer en la pata (a); la (b) compra alivio y mejor factorización, no tiempo.
También valida preferir el disparador primera-mordida del diagnóstico sin
umbral: con reposición así de rápida, la evidencia llega sola en cuanto haya
uso.

**Criterio de Fase 2: acepto la composición, con preferencia por el del
diagnóstico como disparador** — es más fino que el mío: distingue mordida
sobre nota ya sellada-con-aire (evidencia real de que el aire no basta →
banda) de mordida sobre techo pre-guarda (→ solo resellado de esa nota), y
excluye cutovers. De mi criterio conservo dos cosas: el observable ("mordida"
= rotación forzada o IOU visible en commit `docs(kb)`, sin métricas nuevas) y
el reloj en eventos, no calendario — que su disparador ya cumple por
construcción. Cedo el umbral "≥2 en 10 cierres": con cutovers excluidos y
pre-guarda desviado a resellado, el evento restante es exactamente el modo de
fallo que la banda arregla, y su coste es una comparación — una demostración
limpia basta.

---

## Apéndice — datos de verificación

- Distribución por tier (hoy): core n=5, 90,0 KB, max 27,8 KB (Backlog, waiver
  30k) · stable n=43, 406 KB, max 18,8 KB · log n=93, 1.422 KB, **max 98,6 KB**.
- Ratchet: 11 techos; 10 waivers reales sobre nominal, 1 ajuste a la baja
  (evidencia-y-divulgacion, 10.000 < 12.500 nominal). Entre las dos versiones
  del fichero (41b3959 → 5da6c59) el único techo preexistente que cambió
  **bajó**: doctrina-agentes 27.000→20.000. Caveat de §6.4: el ancla inicial
  fue una amnistía (techos sellados donde estaba el agua, no en el nominal), y
  hubo una brecha commiteada por encima de techo sellado (4-ago, 28.703 >
  27.000) corregida en el triaje del 17-ago (§6.5).
- Curva completa de `core/doctrina-agentes.md` (33 puntos, `git log --follow` +
  `git show <sha>:<ruta> | wc -c`): 4.967 (3-jul) → 8.493 (consolida 11-jul) →
  45.788 (2-ago) → 24.131 (split 3-ago) → 28.703 (4-ago, brecha) → 26.840
  (triaje 17-ago) → 26.999 (19-ago) → 19.783 (split 22-ago, techo a 20.000).
- Curvas medidas con `git show <sha>:<ruta> | wc -c` sobre
  `~/Documentos/proyectos/kb-demo` (commits b69215d, 1d9cc3e, 91dd86d,
  591b76e, fd71cab, 77f6cd8, b3df97c, 27f897e, 85b57e2, 5da6c59, d886505).
- Punto de uso: `exo search --json` devuelve `{permalink, ruta, score,
  type:"entity"}` — sin contenido. `recall-inject.sh` compone punteros +
  snippet bajo cap propio. `exo-recall.sh` inyecta core-index entero, cap
  6.144, con log de `truncated`.
- Índice: 145 notas, 3.082 trozos con embedding (`vectores`), agregación de
  resultados a nivel nota.
- Conversión aproximada usada: ~3,7-4 B/token para castellano técnico.
