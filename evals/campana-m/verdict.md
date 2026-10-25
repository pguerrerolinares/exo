# Campaña M — verdict de la etapa 1 (adjudicación independiente)

Adjudicador fresco, sin participación en el diseño, la criba ni las corridas.
Fecha: 2026-10-25. Recomputado desde los datos crudos de
`~/.cache/exo-campana-m/corridas-principal/` con un script propio (scratchpad
de la sesión; lógica en el anexo A). `analizar.py` se ejecutó **después**, en
memoria (importando `analizar()`, sin reescribir `analisis.json`), solo para
contrastar.

Material: `evals/campana-m/preregistro.md` (manda), spec
`docs/superpowers/specs/2026-10-24-memoria-entre-sesiones-design.md` §4,
`congelacion-principal.txt`. Hashes verificados contra el pre-registro §3:
`orden-principal.txt` `261854e1…2cfb`, `sondas.tsv` `af8b5d35…4ec4`,
`neutras.txt` `fb92e13c…9036`, `gold-activo.tar` `92fd5b70…6d19`: coinciden
los cuatro.

---

## 0. Veredicto pre-registrado (literal)

> **`NI X NI N DEMOSTRARON EFECTO` + `RECOMENDACIÓN: memoria mínima`**

Etiqueta de desempate (se reporta siempre, no reetiqueta): **`DESEMPATE: a2`**.

Invalidación (1): no se dispara. Invalidación (2): **se dispara** (límite
inferior de Δ(X − A0) = 0,000 ≤ 0), así que la decisión X frente a N se toma
solo por coste. Invalidación (3): no se dispara (0 escrituras fuera de lo
previsto). R1: no (N no es más barato). R2: no. R3: sin `DAÑO`, pero el
estrato neutro está en techo y no informa (§5).

---

## 1. Resultado primario

Éxito por sonda = fracción de réplicas con `check.rc == 0` (rc 1 y rc 2 son
fallo). Bootstrap sobre sondas, 10.000 réplicas, `random.Random(20261024)`
(una instancia nueva por contraste), percentiles empíricos
`medias[int(0,025·B)]` y `medias[int(0,975·B)−1]`. Test de signos exacto
bilateral sobre las discordantes.

**Éxito por brazo**

| estrato | n | A0 | O (`ar`) | X (`a2`) | N (`n`) |
|---|---|---|---|---|---|
| sondas-suelo | 40 × 3 | **0,008** (1/120) | **0,858** (103/120) | **0,075** (9/120) | **0,033** (4/120) |
| neutras | 15 × 2 | 1,000 (30/30) | — | 1,000 (30/30) | 1,000 (30/30) |

**Contrastes emparejados (sondas, n = 40)**

| contraste | Δ medio | IC95 bootstrap | +/− | p signos |
|---|---|---|---|---|
| O − A0 | +0,850 | [+0,767, +0,925] | 39 / 0 | < 0,0001 |
| X − A0 | **+0,067** | **[0,000, +0,150]** | 3 / 0 | 0,25 |
| N − A0 | **+0,025** | **[0,000, +0,067]** | 2 / 0 | 0,50 |
| N − X | −0,042 | [−0,133, +0,033] | 2 / 3 | 1,00 |
| X − N | +0,042 | [−0,033, +0,133] | 3 / 2 | 1,00 |
| neutro X − A0 | 0,000 | [0,000, 0,000] | 0 / 0 | 1,00 |
| neutro N − A0 | 0,000 | [0,000, 0,000] | 0 / 0 | 1,00 |

Secundaria, R = (brazo − A0)/(O − A0): **X 0,078**, **N 0,029**. Con el
límite superior del IC, X recupera como mucho ≈ 18 % del techo del oráculo
(0,150/0,850) y N ≈ 8 %.

Robustez del límite inferior 0 en X − A0: solo hay 3 sondas discordantes, todas
positivas. La probabilidad de que una remuestra de 40 no contenga ninguna es
(37/40)^40 ≈ 0,044 > 0,025, así que el percentil 2,5 cae exactamente en 0 con
cualquier método de percentil. No es frágil al método; es frágil a **una**
sonda más (con 4 discordantes positivas, (36/40)^40 ≈ 0,015 y el límite
pasaría a +0,025). Lo digo porque la invalidación (2) se dispara en el borde.

Estratificado por tier de origen (descriptivo, spec §8):

| tier | n | A0 | X | N | Δ(N − X) | +/− |
|---|---|---|---|---|---|---|
| core | 4 | 0,000 | 0,500 | 0,000 | −0,500 [−1,0, 0,0] | 0 / 2 |
| stable | 36 | 0,009 | 0,028 | 0,037 | +0,009 [−0,046, +0,065] | 2 / 1 |

Todo el efecto de X está en 2 de las 4 sondas core. En las 36 stable, X y N
son indistinguibles de A0.

**Invalidación (1).** O tiene ≥ 2/3 réplicas rc 0 en **35/40** sondas
(umbral 12). No se dispara: el canal funciona cuando la regla llega. Las 5 en
que O no llega: `g0-66`, `g0-33`, `g0-55`, `g0-110`, `g0-44`.

### Aplicación mecánica de las reglas (§4 del pre-registro)

- **R1** (`GANA N`): límite inferior de Δ(N − X) = −0,133 > −0,15 (no
  inferior), **pero** la mediana de tokens_in de N (129.527 en sondas; 118.556
  en las 55) no es menor que la de X (107.290; 104.998). **No se cumple.**
- **R2** (`EXO SOBREVIVE`): límite inferior de Δ(X − N) = −0,033 ≤ 0. **No.**
- **R3**: límite inferior en el neutro = 0,000 > −0,15 para X y para N. **Sin
  `DAÑO`.** (Ver §5: no informativo.)
- **Ni X ni N**: límites inferiores de Δ(X − A0) y Δ(N − A0) = 0,000 y 0,000,
  ambos ≤ 0 ⇒ **`NI X NI N DEMOSTRARON EFECTO` + `RECOMENDACIÓN: memoria mínima`.**
- **Desempate** (prevalencia 0,012 × Δ frente a sobrecarga fija): X
  0,012 × 0,067 = 0,0008 con 6.166 B; N 0,012 × 0,025 = 0,0003 con 15.550 B.
  X domina a N en las dos dimensiones ⇒ **`DESEMPATE: a2`**. Leído en absoluto:
  0,0008 correcciones ahorradas por sesión, frente a +14,5k tokens_in de
  mediana por sesión.

### Contraste con `analizar.py`

Coinciden todas las cifras de éxito, los cuatro IC, los IC del neutro, R,
`oraculo_ok` = 35, `frac_sucia` = 0, 0 excluidas, las medianas de R1 y el
veredicto. Tres diferencias, ninguna cambia la adjudicación:

1. **Falta la etiqueta `DESEMPATE`.** En la rama «ni X ni N»
   (`analizar.py:79-81`) la función retorna antes de calcular el desempate,
   pero el pre-registro dice que «se reporta siempre». Aquí: `DESEMPATE: a2`.
2. **Coste por media, no por mediana.** `_coste()` (`analizar.py:153-155`)
   da la media de los Δ por sonda (X +15.119, N +37.849); la mediana emparejada
   es X +14.545, N +29.889. El pre-registro no fija el estadístico del coste;
   la media de N la infla una cola (máx. 553k tokens_in en una corrida N).
3. **R1 usa la mediana solo de las sondas-suelo** (`analizar.py:211-212`); el
   pre-registro no dice el conjunto. Con las 55 sondas la conclusión es la misma.
   Cuidado, eso sí, con la mediana agrupada de las 55: es bimodal (sondas
   ≈ 130k y neutras ≈ 105k en N) y quitar una sola corrida la mueve de 118.556
   a 107.809. El Δ emparejado por sonda es el estadístico estable.

## 2. Coste

tokens_in por corrida (desde `meta.json`):

| brazo | mediana sondas | mediana neutras | Δ emparejado vs A0, 55 sondas (mediana / media) | ratio mediano | sobrecarga fija (bytes inyectados) | USD total |
|---|---|---|---|---|---|---|
| A0 | 92.591 | 75.540 | — | 1,00 | 0 | 10,56 |
| O | 96.126 | — | +658 / +722 (solo 40) | 1,01 | 210 (mediana) | 10,88 |
| X | 107.290 | 86.706 | **+14.545 / +15.119** | **1,15** | 6.166 (constante, 150/150) | 13,50 |
| N | 129.527 | 105.068 | **+29.889 / +37.849** | **1,39** | 15.550 (constante, 150/150) | 16,81 |

Neutro aparte: X +12.824 de mediana (+15 %) y N +29.332 (+39 %), es decir, la
misma sobrecarga que en las sondas. Es casi toda fija por sesión: el bloque se
vuelve a leer en cada turno. N cuesta el doble que X y 2,5 veces más bytes de
arranque. La mediana de turnos es 5 en A0, X y N, y 6 en O.

Gasto de la principal: 51,75 USD. Acumulado de la campaña: 81,84 USD de 130.

## 3. Integridad

- **570/570 corridas** del `orden-principal.txt`, cada una con `meta.json`,
  `check.rc`, `fugas.json`, `kb-sucia.txt`, `inyectado.txt`,
  `transcript.jsonl` y `diff.patch`. `meta.fin = completed` en las 570;
  `claude-version.txt = 2.1.310` en las 570. Máximos: 19 turnos (< 40) y 0,87
  USD (< 10). Ninguna corrida cortada.
- **`kb-sucia.txt` vacío en 570/570.** 0 avisos de memoria en N, es decir,
  ninguna corrida N escribió en su carpeta de memoria.
- **Inyectado constante por brazo:** A0 vacío; X un único sha256 (`fb854526…`,
  6.166 B) en las 150; N un único sha256 (`ea87fc99…`, el `memoria_sha256` de
  la congelación) en las 150; O una regla distinta por sonda, idéntica entre
  réplicas. El bloque de X no tiene recall por prompt (`recall-inject`
  retirado; `reflex.jsonl` solo registra `project-rules-skip`).
- **Carga real de `MEMORY.md` en N:** el system prompt no queda en el
  transcript, pero las 150 corridas N registran
  `memory_paths.auto = …/<corrida>/memoria/` en `system.init`; el canario
  `corridas-canario/n-carga/n-r1` reproduce las líneas del índice y cuesta
  +7,6k tokens_in en un turno frente a su A0. La sobrecarga medida (+30k en
  unos 5 turnos) cuadra con eso.
- **La corrida marcada: `g2-160/n-r2`.** `fugas.json` dice
  `fuga: true, motivos: ["toca la KB de producción o ~/.exo"]`, y el breaker
  de `tanda.sh` paró en la tanda 15/15, que era la última: no quedó ninguna
  corrida sin hacer. Qué pasó exactamente: el agente lanzó
  `cat README.md; ls ~/.exo 2>&1 | head; ls ~/go/bin 2>&1 | head` buscando el
  binario de una herramienta. Claude Code **denegó** la llamada entera (`permission_denials`
  del evento `result`: tool_result «Permission to use Bash … has been
  denied»), así que no se ejecutó nada: ni se leyó ni se escribió. `fugas.py`
  marca cualquier `tool_use` cuyo texto mencione `~/.exo`.
  **Decisión:** la regla (3) del pre-registro y la §4.6.3 de la spec excluyen
  solo «escritura fuera de lo previsto», y aquí no hubo escritura ni lectura.
  **No se excluye.** Tasa de escrituras fuera de lo previsto: 0/570 (0 %, umbral
  5 %). `analizar.py` decide lo mismo: solo cuenta los motivos de escritura.
  Sensibilidad excluyéndola: N en `g2-160` pasa de 0/3 a 0/2, y todas las
  cifras de §1 quedan idénticas.

## 4. Diagnóstico cualitativo: por qué fallan X y N

Antes que las muestras, una medida que lo explica casi todo. Conté las
herramientas de los 240 transcripts de X y N en sondas:

- **X invocó `exo` en 3/120 corridas**, las tres en `g2-160`, y en ninguna
  hizo una búsqueda: `exo --help`, `exo lint` y `exo targets --help`. **0/120
  `exo search` o `exo recall`. 0/120 lecturas del snapshot `prep/kb`.** El
  bloque le pide buscar contexto antes del primer Agent/Edit/Write y le
  anuncia que el reflejo `search-first` avisará si no lo hace. En este brazo
  ese reflejo no está cableado (`correr.sh:61` solo pone `exo-recall.sh` en
  `SessionStart`), y el agente ignora la instrucción en prosa.
- **N abrió su carpeta de memoria en 2/120 corridas** (`g2-160/n-r1` y `n-r3`,
  con grep y sed). En las otras 118 no leyó ningún topic file.

Así que, en la práctica, «acceso» se redujo en los dos brazos a **lo que venía
inyectado al arrancar**. Las reglas de las sondas están literalmente en el
bloque de X en 4 casos (`g0-9`, `g1-14`, `g1-15`, `g0-12`) y parafraseadas en
otro (`g2-27`). En `MEMORY.md` solo aparece un eco de una (`g1-14`), como
descripción de dos notas concretas.

### Muestra de fallos donde O acierta (10 casos, X y N)

| corrida | O | ¿le llegó la regla? | qué hizo | clase |
|---|---|---|---|---|
| `g0-12/a2-r1` (y r2, r3) | 3/3 | **Sí**, literal en el bloque | hace todo inline; O aplica la regla como primera acción. En K, con la misma línea, A3 la aplicó 2/2 | **recuperó y no aplicó** |
| `g2-27/a2-r1` | 3/3 | Sí, parafraseada en el bloque | se salta el paso que pide la regla; O lo hace y lo cita | **recuperó y no aplicó** |
| `g1-15/a2-r1`, `a2-r2` | 2/3 | Sí, literal | r1 la aplica a medias; r2 hace lo contrario | **recuperó y no aplicó** (r1 a medias) |
| `g1-147/a2-r1` | 3/3 | No: el bloque solo trae un puntero de actividad reciente relacionado. No buscó | hace el cambio sin la condición que fija la regla; rc 2 | **no recuperó** |
| `g2-160/a2-r2` | 3/3 | No. Corrió `exo targets --help` y `exo lint`, nunca `search` | otra nota del bloque contradice la regla de la sonda y el agente la sigue; rc 2 | **no recuperó + distracción por la propia memoria** (la KB contradice la regla de la sonda) |
| `g1-21/a2-r1` | 3/3 | No (no está en el core-index). No buscó | resuelve con el default; rc 2 | **no recuperó** (el rc 2 del check equivale aquí a violar) |
| `g0-9/n-r1` (y r2, r3) | 3/3 | No: la línea de `MEMORY.md` sobre el tema no contiene la regla. No abrió ningún topic file | aplica el default | **no recuperó** |
| `g1-14/n-r1` | 3/3 | A medias: `MEMORY.md` describe la regla para dos notas concretas | la aplica a esa instancia y la viola en el resto | **recuperó una instancia y no la generalizó** |
| `g2-160/n-r1` | 3/3 | **Llegó a la nota de origen** (grep sobre `memoria/` y lectura de las primeras 25 líneas), pero la línea de la regla quedó fuera del rango que leyó | aplica el default | **recuperó la nota pero no la línea de la regla** |
| `g1-147/n-r1` | 3/3 | No. `MEMORY.md` tiene el puntero relevante, pero el transcript no lo menciona | hace el cambio sin la condición | **no recuperó** (la pista estaba en el índice y no la siguió) |
| `g0-43/n-r1` | 3/3 | No | viola la regla; rc 2 porque el check no reconoce la vía de edición que usó | **no recuperó**; **check dudoso** en la etiqueta (rc 2 en vez de 1), no en el resultado |

### Los aciertos de X y N, uno por uno

| sonda | aciertos | ¿memoria? |
|---|---|---|
| `g0-9` X | 3/3 | **Sí, causal.** Las tres réplicas atribuyen la decisión a la KB citando el core-index inyectado |
| `g1-14` X | 3/3 | **Sí.** La regla está literal en el bloque y el agente la aplica, igual que en K |
| `g1-110` X | 2/3 | **No.** La regla no está en el bloque y no hubo búsqueda. `a2-r2` llega por su cuenta a la solución. Las tres de A0 se paran y reportan (rc 2): es temperamento, «arreglar» frente a «informar», como ya vio K |
| `g0-96` N | 1/3 | **No.** `n-r3` encuentra la respuesta en un documento del propio repo |
| `g2-186` N | 2/3 | **No.** Es la solución natural; no tocó la memoria |
| `g2-141` | 1/3 en A0, X y N | Ruido simétrico |

**Conclusión del diagnóstico.** El efecto de memoria verificable es de **2
sondas en X (6 corridas) y 0 en N**, y las dos de X vienen de reglas core
copiadas en el bloque estático de arranque, no del acceso bajo demanda. El
modo de fallo dominante es **«nunca la vio»**: el agente headless no inicia
nunca la recuperación, ni con `exo search` ni con `Read` sobre los topic
files, aunque el índice o el bloque le den la pista. El segundo modo es **«la vio y no la aplicó»**: 3 de las 5 reglas
presentes en el bloque de 6 KB de X no se cumplen, mientras que la misma regla
sola, en 210 B por el mismo canal (O), se cumple en 2/3 o 3/3. El problema es
la dilución, no el canal. Hay además un caso de memoria que **estorba**
(`g2-160/a2-r2`: otra nota de la KB contradice la regla y el agente la sigue).

## 5. Validez del instrumento

- **Neutro en techo (90/90 rc 0).** R3 «pasa», pero el estrato no tiene
  sensibilidad: no puede detectar daño porque no hay margen por debajo. No
  equivale a `SIN DATOS` en sentido formal (los datos existen) y no cambia el
  veredicto, pero **«sin daño» no queda demostrado**. Del neutro solo vale el
  coste. Las neutras son más fáciles que las S2 de K (A0 0,80 allí).
- **Sondas con check heurístico (§5.4: `g1-95`, `g2-121`, `g2-66`):
  ninguna quedó entre las 40.** La advertencia no afecta al análisis. Sí hay
  checks cuya rama rc 2 está mal etiquetada (`g1-21` y `g0-43`: dan rc 2
  ante conductas que violan la regla, en vez de rc 1). Como rc 2 cuenta como fallo y la conducta revisada viola de todos
  modos, no mueve ningún éxito. Los rc 2 están repartidos por igual (A0 12,
  X 13, N 13, O 4).
- **Suelo de A0 confirmado:** 1/120 con réplicas nuevas (`g2-141/a0-r1`). La
  regresión a la media que inducía la criba es despreciable.
- **Ancla positiva sana:** O 0,858, 35/40. El diseño era sensible: un brazo
  que entregara la regla como O habría dado Δ ≈ 0,85. No es un nulo por falta
  de potencia: los IC superiores (X 0,15, N 0,067) acotan el efecto por debajo
  de ~1/5 del techo.
- **Tier:** 36/40 sondas son stable, y ni el core-index de X ni el índice de N
  contienen reglas stable. El resultado mide sobre todo el acceso a
  conocimiento que no está en el arranque, que es justo donde los dos brazos
  dependen de que el agente decida buscar.
- **Validez de constructo de X (la amenaza más seria).** X es
  `exo-recall.sh` y nada más. El plugin de producción (`plugins/exo/hooks/hooks.json`)
  añade `search-first.sh` en `PreToolUse` (Agent/Edit/Write), el reflejo que
  el propio bloque promete («avisa si no buscaste»), además de
  `subagent-inject` y otros. Lo que la spec §4.1 define como X está bien
  medido, pero **no es exo tal como corre en producción**. El hallazgo
  central (0/120 búsquedas) es exactamente lo que ese reflejo existe para
  corregir.
- **Validez de constructo de N.** El `MEMORY.md` es mecánico: título más
  descripción o primera frase de 68 notas core y stable, decidido por el
  orquestador (spec §8). Las descripciones casi nunca contienen la regla. La auto memory nativa real escribe hechos y
  reglas con su propio criterio. N midió «índice de títulos + topic files bajo
  demanda», no «memoria nativa con contenido en forma de regla». La carpeta
  tiene 200 notas (el snapshot entero); el «286» de la spec §8 es de la KB
  viva.
- **Modo headless de un solo prompt.** `-p` con tareas cortas (mediana de 5
  turnos) favorece no explorar. En sesiones interactivas largas la propensión a
  buscar puede ser otra; esto no lo mide.

## 6. Qué decide y qué NO decide

**Decide:**

- Con el **acceso pasivo** (bloque o índice al arrancar más herramientas
  disponibles sin empujón), ni exo ni la auto memory nativa hacen que el
  agente actúe según lo decidido antes, en reglas que no vienen literales en
  el arranque. El cuello no es la recuperación offline (saturada, spec §2) ni
  el canal (O funciona): es que **el agente no busca**, y cuando la regla está
  en un bloque grande, a menudo no la aplica.
- Entre las dos, por coste y desempate: **X es más barata que N** (+15 % frente
  a +39 % de tokens_in; 6,2 KB frente a 15,6 KB) y no peor. El `MEMORY.md`
  mecánico de N no compensa lo que cuesta.
- Lo que sí movió conducta fue **la regla escrita en el contexto de
  arranque**, y cuanto más corta y saliente, mejor (O ≫ reglas sueltas en el
  core-index > índice de títulos). Eso apoya la recomendación pre-registrada:
  un CLAUDE.md conciso con las reglas que importan.

**No decide:**

- Si **exo de producción** (con `search-first` y el resto de hooks) tiene
  efecto. No se midió.
- Si una **auto memory nativa escrita por el propio agente** (reglas, no
  títulos) supera a A0. El N de aquí es un índice mecánico.
- El daño en neutras (techo).
- El valor de escritura y mantenimiento de la KB (piloto §4.8, fuera de la
  etapa 1).
- Nada fuera de esta KB, este modelo (`claude-sonnet-5-5`) y Claude Code 2.1.310.

## 7. Recomendación

1. **Adjudicar tal cual: `NI X NI N DEMOSTRARON EFECTO`, `RECOMENDACIÓN:
   memoria mínima`**, con `DESEMPATE: a2`. Las reglas se aplicaron sin
   reinterpretación.
2. En la práctica, «memoria mínima» quiere decir llevar al arranque, en forma
   corta y literal, las reglas de alta frecuencia: un CLAUDE.md o un core-index
   **podado**. Las 2 victorias de X salen de ahí, y sus 3 fallos con la regla
   presente salen de la dilución. No abandonar la KB como almacén, porque esta
   etapa no la evalúa como escritor.
3. Antes de dar por cerrada la pregunta de exo, la medida barata que la
   cierra: **un brazo `a2+search-first`** (el hook de producción) sobre las
   mismas 40 sondas × 3 (≈ 120 corridas, ≈ 14 USD a precios de esta tanda).
   Condición que invalidaría la recomendación 1 a favor de exo: que ese brazo
   busque en ≥ 50 % de las corridas **y** dé un límite inferior de Δ frente a
   A0 > 0. Si busca y aun así no aplica, la memoria mínima queda confirmada.
   Opcionalmente, un N con `MEMORY.md` en forma de reglas, para no
   descartar la vía nativa con un índice que la condena.
4. Arreglos del instrumento para la próxima campaña: añadir a `analizar.py`
   el `DESEMPATE` en la rama «ni X ni N» y fijar mediana o media para el
   coste; hacer que `fugas.py` distinga entre intento denegado y acceso
   (`g2-160/n-r2`); diseñar las neutras con A0 < 1 para que R3 tenga
   sensibilidad; corregir las ramas rc 2 de `g1-21` y `g0-43`.

---

## Anexo A — lógica del recómputo

```
éxito(t,b)  = #(rc==0)/#réplicas            # rc de check.rc; 570 corridas de orden-principal.txt
Δ(a,b)      = [éxito(t,a) − éxito(t,b) for t in sondas]
bootstrap   : rng = random.Random(20261024); 10.000 × media de [Δ[rng.randrange(n)] for _ in range(n)]
IC95        = sorted[int(.025·B)], sorted[int(.975·B)−1]
signos      = 2·Σ_{i≤min(p,q)} C(p+q,i)/2^(p+q), tope 1
inval (1)   = #{t: #(rc_ar==0) ≥ 2} vs 12
coste       = mediana_t( media tokens_in(t,b) − media tokens_in(t,a0) ), 55 sondas (ar: 40)
conducta    : tool_use Bash con /(^|[\s;&|(])exo\s/ ; tool_use con '/memoria' o 'prep/kb' en la entrada
```

Lo ejecuté con `python3` sobre `~/.cache/exo-campana-m` y solo leí; no
modifiqué nada bajo `~/.cache`.
