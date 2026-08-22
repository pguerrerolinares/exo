# Auditoría — arqueología del problema de presupuestos

Ángulo: hechos, con commit y número. Cero propuestas. Fuente: `git log` completo
de `kb-demo` y `kbx`, contenido de notas en distintos commits, `.kbx-ratchet.json`
en sus dos versiones, código de `kbx` (`internal/budget`, `internal/rotate`), e
índice `~/.exo/index.db` (copiado, no tocado en sitio).

## 1. Cuándo nació y por qué

**`d0f1b8e`, 2026-07-03**, spec `docs/superpowers/specs/2026-07-03-memoria-v2-design.md`.
KB de 3 semanas, 97 notas, ~235k tok.

Dato clave que contradice la premisa que se está discutiendo hoy: **la propia
spec fundacional midió y descartó "el arranque inyecta demasiado"** como
problema real —textual: *"El arranque es barato [...] No es el problema"*
(~1,4k tok medidos). El problema medido y real era otro: **el coste por pull**.
Los "cores" ya estaban hipertrofiados por apéndice fechado sin destilar:
`desarrollo-agentico.md` 20,6k tok, `Backlog` 12,8k tok, `perfil` 8k tok, con
24 notas de sesión de code-graph-engine re-narrando el mismo contexto ~3,5
veces por sesión. El presupuesto nació para forzar destilado (canon vs
bitácora), no para abaratar un hook de arranque. **Es decir: el "dato que
puede cambiarlo todo" de hoy (que el presupuesto nació por la vía de
inyección-al-arranque) es una hipótesis que la propia spec fundacional ya
verificó como falsa hace siete semanas.** Eso no cierra la pregunta de hoy
(hay una vía nueva, `exo recall` en el punto de uso, que no existía), pero
hay que preguntarla contra el problema real que motivó el sistema —coste de
pull/lectura de una nota entera—, no contra un problema que nunca fue la
causa.

Presupuesto original declarado: core ≤2k tok (~8KB), stable ≤3k tok (~12KB),
log sin presupuesto de tier (pero con umbral de rotación de 20.480 B "hot
tail", ver §6). Se implementó tal cual en `kbx` el mismo día (`855fa07` spec
M4, `5cc755c` impl, 2026-07-11).

## 2. Cuántas veces ha mordido — cronología con tamaños

26 commits marcados en el recon inicial se confirman. Cronología real de
intervenciones (fecha, commit, qué pasó):

| Fecha | Commit | Evento |
|---|---|---|
| 07-03 | `d0f1b8e` | Nace el sistema |
| 07-05 | `90c2e07` | Split de 17 notas |
| 07-09 | `1d9cc3e` | Split cge/doctrina/learnings |
| 07-11 | `91dd86d` | "Pasada bootstrap": split de **10 offenders** |
| 07-12 | `89e5a77` | Split de 3 notas obesas más |
| **07-15 → 08-03** | — | **19 días sin ninguna intervención de presupuesto** |
| 08-03 | `362e998` | `rotate` F0: primera rotación, 534.247 B a `archive/` |
| 08-03 | `41b3959` | **Amnistía**: 9 techos de waiver sellados (trinquete nace aquí) |
| 08-03 | `b3df97c` | Split de 9 notas obesas más |
| 08-17 | `27f897e` | Triaje: 3 notas core "vuelven bajo su techo sellado" |
| 08-22 | (M6-06, hoy) | El presupuesto muerde **dos veces en la misma sesión** |
| 08-22 | `5da6c59` | Split de `doctrina-agentes`, techo bajado 27000→20000 |

La frecuencia **no baja, sube**: de intervenciones espaciadas semanas (jul) a
una campaña completa (rotate+amnistía+split, 6 commits en un día) el 08-03,
un triaje el 08-17 (14 días después), y dos mordidas en una sola sesión el
08-22 (5 días después). El intervalo entre incidentes se está **comprimiendo**,
no espaciando.

## 3. Caso de estudio con byte-a-byte: `core/doctrina-agentes.md`

Es la nota mejor instrumentada para responder "¿vuelve a crecer hasta el
techo?" porque se puede medir en cada commit que la tocó (`git log --follow`):

```
07-03  4.967 B   (nace, tier core, presupuesto nominal 8.500)
07-09  7.469 B   (tras un split)
07-11  8.493 B   (tras "pasada bootstrap" — YA casi en el nominal, recién limpiada)
07-17  9.325 → 11.066 B
07-18  13.829 → 18.256 B
07-25  22.745 → 25.591 B
07-27  31.215 → 34.596 B
07-28  36.709 → 39.784 → 43.880 B
08-02  45.788 B   (5,4× el presupuesto nominal, sin ninguna intervención en 3 semanas)
08-03  24.131 B   (split del día de la amnistía — sigue siendo 2,8× el nominal)
08-03  26.113 → 26.840 B
08-04  28.703 B   (YA por encima del techo sellado ese mismo día en 27.000)
08-17  26.840 B   (triaje del trinquete: recorte de 1.863 B, NO fue un split real)
08-18  26.869 B
08-19  26.999 B   (a 1 B del techo sellado — recompuesto en 2 días)
08-22  19.783 B   (split real, y el techo mismo baja de 27.000 a 20.000)
08-22  19.967 B   (99,8% del nuevo techo, el mismo día)
```

Lectura: **cada "arreglo" de esta nota duró entre 2 días y 3 semanas antes de
volver a estar pegada al techo.** El "triaje" del 08-17 no fue un split — fue
un recorte de 1.863 B que la devolvió a menos del 1% de margen en 48 horas
(08-19: 26.999/27.000). El único evento que rompió el patrón fue partir de
verdad *y* bajar el techo el mismo día (08-22) — y esa nota ya cerró hoy a
99,8% del nuevo techo.

## 4. El patrón no es de una nota: es de casi toda la KB

Escaneadas todas las notas `core`/`stable` activas (fuera de `archive/`,
`docs/`, `.superpowers/` — mismas exclusiones que usa `kbx budget`) contra su
presupuesto nominal o su techo de waiver, **hoy, ahora mismo**:

| Nota | Tamaño | % de su techo/presupuesto |
|---|---|---|
| Paul - perfil de trabajo.md | 17.991 B | **100,0%** de 18.000 (waiver) |
| core/doctrina-agentes.md | 19.967 B | 99,8% de 20.000 (waiver) |
| learnings/desarrollo-agentico.md | 18.951 B | 99,7% de 19.000 (waiver) |
| projects/agent-solve-it.md | 18.848 B | 99,2% de 19.000 (waiver) |
| projects/lighthouses-bot.md | 15.904 B | 99,4% de 16.000 (waiver) |
| projects/kbx (Go).md | 12.490 B | 99,9% de 12.500 (nominal) |
| projects/finanzas-empresa-x.md | 12.374 B | 99,0% de 12.500 (nominal) |
| projects/pguerrero.me.md | 13.738 B | 98,1% de 14.000 (waiver) |
| projects/pguerrero-music.md | 15.680 B | 98,0% de 16.000 (waiver) |
| learnings/pragmatismo-y-pivots.md | 14.323 B | 95,5% de 15.000 (waiver) |
| Backlog — frentes abiertos.md | 27.831 B | 92,8% de 30.000 (waiver) |
| projects/agent-develop.md | 15.696 B | 92,3% de 17.000 (waiver) |

**11 de las 12 notas más grandes de la KB están entre el 92% y el 100,0% de su
límite, sea nominal o de waiver, en este instante.** Esto no es un puñado de
notas que ocasionalmente rozan el techo: es la práctica totalidad de las notas
grandes de la KB viviendo pegadas a su límite de forma permanente. Encaja con
la lectura de "bucle": el sistema no mantiene las notas por debajo de un
tamaño razonable, las mantiene *exactamente en* su tamaño máximo permitido —
sea cual sea ese máximo, nominal o inflado por waiver.

De los 4 archivos `tier: core` que sí cuentan para presupuesto (excluyendo
`docs/`), **3 de 4 (75%) solo existen hoy gracias a un waiver** por encima del
presupuesto original de diseño (8.500 B); el único que cabe cómodo bajo el
nominal es `core/core-index.md` (5.232 B, 61,6%), la nota más nueva y más
pequeña por construcción (es un índice de punteros, no contenido).

## 5. El trinquete: ¿"solo baja" es cierto?

El propio `core/doctrina-agentes.md`, escrito **hoy mismo** en la sesión
M6-06, defiende la doctrina: *"todo cap nacido como presupuesto con mecanismo
aguantó o bajó [...] el de kbx solo ha bajado"*. Con los datos crudos delante,
esa afirmación es cierta solo si se cuenta desde el ancla de amnistía
(`41b3959`, 08-03) en adelante — el trinquete, por diseño, no registra nada
anterior a su propio sellado.

Pero lo que pasó **antes** de esa ancla es exactamente la historia que la
frase omite: `doctrina-agentes` pasó de un presupuesto nominal de diseño de
**8.500 B** a un techo sellado de **27.000 B** —una subida del 218%— el mismo
día en que se declaró "amnistía" para 9 notas que llevaban semanas
incumpliendo. La amnistía **es** una subida de techo, solo que ocurrió una
vez, en bloque, con nombre ceremonial ("ancla de amnistía") en vez de nueve
subidas sueltas. Desde ese ancla, es verdad que solo ha bajado (27.000→20.000
hoy). Si la pregunta de Paul es "¿los caps se han subido alguna vez?", la
respuesta correcta con los números es: **sí, en bloque, una vez, disfrazada de
evento fundacional del propio mecanismo que ahora se cita como prueba de que
"solo baja".**

## 6. Coste de mantenerlo — lo que dicen las bitácoras

- **Construir el mecanismo de rotación (`kbx rotate`) costó una campaña SDD
  completa**: 5 tareas, review por tarea + review final de rama en Opus, 13
  commits (`log/kbx-bitacora.md`, entrada 2026-08-03). Se cazaron **4 bugs de
  pérdida de datos** antes de tocar la KB real (colisión de nombre por rango
  de fechas — 44 entradas afectadas; `filepath.Glob` con metacaracteres — 36
  entradas; frontmatter degradado en silencio; escrituras no atómicas).
  Ninguno lo habría cazado la suite en verde.
- **5 campañas explícitas de `/consolida`** con prefijo de commit dedicado
  (`91dd86d`, `8a623c9`, `89e5a77`, `b3df97c`, `5da6c59`), más el `rotate` y
  la amnistía del 08-03 como campaña separada.
- **Hoy mismo, en una sola sesión de cierre de M6-06**, el presupuesto mordió
  dos veces y generó, además del propio recorte: (a) un aprendizaje nuevo en
  `learnings/Fallo silencioso.md` sobre "un límite mal dimensionado raciona en
  silencio", (b) una sección entera de doctrina nueva ("la regla de los caps")
  que no cupo en ningún core y quedó aparcada en una bitácora "candidata a
  promoción" porque las dos notas candidatas a recibirla estaban a 33 B y 49 B
  de su techo. El propio mecanismo de presupuesto impidió que la doctrina
  *sobre* el presupuesto se guardara en el sitio que le tocaba.
- Cita textual de Paul, recuperada de la bitácora de hoy (`log/doctrina-agentes-bitacora.md`):
  *"los caps no es la primera vez que dan problema y siempre hemos subido,
  ¿no es un apaño esto?"* — es la misma pregunta que motiva esta auditoría,
  formulada y parcialmente respondida ya una vez hoy, horas antes de que se
  lanzara esta consulta.

## 7. Cómo crece la KB de verdad

- Ritmo de commits: 68 (jun) → 170 (jul) → 31 (ago, hasta el 22 — proyecta a
  ~44/mes, bajando). Confirma el recon.
- Tamaño hoy: 168 notas .md, 2.056.314 B de contenido markdown puro (2,06 MB;
  el 4,3 MB del recon incluye overhead que no es contenido — no se ha podido
  reconciliar exactamente, diferencia no crítica para el argumento).
- 109 notas activas (fuera de `archive/`): 504.661 B en 36 notas `log`
  (14.018 B de media — y **sin presupuesto de tier**, pero con umbral de
  rotación de 20.480 B "hot tail" que si se supera también dispara
  intervención: la afirmación de "log sin límite" del contexto es cierta para
  `kbx budget`, pero no para `kbx rotate`, que sí tiene su propio techo), 41
  `stable` (395.595 B, media 9.648 B) y 4 `core` con presupuesto real
  (126.266 B incluyendo Backlog/perfil/doctrina/core-index, media inflada por
  los 3 waivers).
- Crecen las notas de proyecto activo y las bitácoras del frente que está
  caliente esa semana; el resto (research fechada, proyectos cerrados como
  `lighthouses-bot`, `cliente-c`, `otros-menores.md`) está muy por debajo de su
  presupuesto y lleva semanas sin tocarse — la disparidad entre "notas
  pegadas al 99% del techo" y "notas al 30-50%" se explica enteramente por
  actividad reciente, no por presupuesto per se.

## 8. Qué pasó con lo podado — ¿se ha vuelto a leer?

El `rotate` de 08-03 movió 534.247 B (11 bitácoras) a `archive/log/`, y el
mismo día se rescataron 143.014 B más de research a `archive/research/`
(estaban solo en scratchpad efímero). Contra la hipótesis de "se archiva y se
olvida":

- **`archive/` no está excluido de retrieval, solo de presupuesto.** El
  índice del engine (`~/.exo/index.db`, copiado para esta auditoría) tiene
  **145 notas indexadas, de las cuales 59 (40,7%) son de `archive/`**. El
  código de `kbx budget` excluye `archive` del cómputo de bytes
  (`internal/budget/budget.go: DefaultExclude`), pero eso no toca el índice
  de retrieval de `exo`, que es un mecanismo aparte.
- **Hay una cita, en la propia KB, de que esto pasó de verdad**:
  `log/exo-bitacora.md:146` — *"El ensanchamiento de alcance de T3 se ve al
  primer uso: `kbx targets "memoria"` devuelve ahora notas de
  `docs/superpowers/plans/` y `archive/sesiones/`, que el filtro `tipo='note'`
  escondía. Es el cambio esperado, no ruido de un port roto."* Es decir: hay
  al menos un caso documentado, verificado en producción, de contenido
  archivado resurfaciendo en una búsqueda real.
- **Cada nota archivada queda enlazada desde la cabecera de la bitácora activa
  correspondiente** (patrón "aviso enlazado" citado en `core/core-index.md`),
  y hay 8 notas activas fuera de `archive/` con enlaces vivos y explícitos
  hacia notas concretas de `archive/` (no solo hacia el directorio en
  abstracto) — ver `Backlog`, `core/core-index.md`,
  `projects/kbx (Go).md`, `log/kbx-bitacora.md`, `log/agent-solve-it-bitacora.md`,
  `log/openwisdom-bitacora.md`, entre otras.
- No existe un log de queries en el índice (`~/.exo/index.db` no tiene tabla
  de histórico de búsquedas), así que **no se puede medir cuántas veces se ha
  leído** cada nota archivada — solo que el mecanismo que la haría legible
  (índice + enlaces) está intacto y al menos una vez se comprobó que
  funciona en producción. No hay evidencia de que lo archivado esté muerto;
  tampoco hay telemetría para afirmar que se usa activamente.

## Adenda — Ronda 1 (respuesta al coordinador de síntesis)

### A. Régimen pre-08-17: ¿warning ignorado o ausencia total de gate?

**Ausencia total, y documentada como decisión deliberada.** El pre-commit hook
(`kb-precommit.sh`) se creó el **2026-08-17** (`3aebd5d`, repo `agent-develop`)
y se instaló como symlink en `kb-demo/.git/hooks/pre-commit` ese mismo día
(reubicado a `exo/plugins/reflex/scripts/` el 08-18, `7ade295`, sin cambio de
contenido). Antes de esa fecha **no existía ningún hook de git** en el repo de
la KB — ni el de `kbx` ni un `kb-budget-check.sh` anterior (ese script bash,
creado 07-03 junto con `/consolida`, nunca se conectó a un hook; era invocable
solo a mano o desde `/consolida`).

Y no es un descuido: la propia spec de `kbx` (`f64502c`, 2026-08-03, "v3 —
rotación de bitácoras primero, hook sincronizado al cierre de A1") lo declara
explícito — *"La doctrina inyectada ordena el anti-patrón. core-index.md:19
dice 'una nota que no cabe declara kbx_budget_max, no se mutila', se inyecta
en cada arranque y está congelada hasta el 17-ago. Activar el hook antes deja
a cada agente con dos instrucciones contradictorias, y el que cierra sesión a
las 3am improvisa: recorte en caliente, fuga a log/ o --no-verify. F1.b se
activa el 17 junto con la corrección de esa línea."*

No hay ningún commit ni entrada de bitácora entre 07-11 y 08-17 que mencione
una ejecución manual de `kbx budget` con offenders visibles. **El A/B del
abogado (agent-solve-it +77,7 KB en 16 días sin bloqueo vs 0 B con bloqueo) es
"nada vs bloqueo", no "warning ignorado vs bloqueo".** No hay evidencia de que
nadie viera una señal y la ignorara durante ese periodo — la señal
simplemente no corría en ningún sitio automático.

### B. Velocidad de relleno reciente (régimen con hook activo, ≥08-17)

Solo hay 2-5 días de datos bajo el hook real (instalado 08-17), así que esto
es una muestra corta, no una tasa estable. Con esa salvedad, medido por
`git log --follow` + tamaño en cada commit:

| Nota | Ventana | Δ bytes | B/día |
|---|---|---|---|
| `Backlog — frentes abiertos.md` | 08-17→08-19 (26.425→29.994) | +3.569 B | **~1.785 B/día** |
| `core/doctrina-agentes.md` | 08-17→08-19 (26.840→26.999) | +159 B | ~80 B/día |
| `Paul - perfil de trabajo.md` | 08-17→08-19 (17.801→17.991) | +190 B | ~95 B/día |
| `learnings/desarrollo-agentico.md` | 08-03→08-22 | 0 B | **0** (sin tocar 19 días, parada al 99,7% del techo) |
| `projects/agent-solve-it.md` | 08-03→08-22 | 0 B | **0** (sin tocar 19 días, parada al 99,2% del techo) |

**No hay una tasa única — hay dos regímenes distintos por nota.** `Backlog`
(el frente activo de la semana) devora un headroom de 3 KB en menos de 2 días.
`doctrina-agentes` y `perfil`, en su fase activa reciente, tardarían ~5-6
semanas en agotar 3 KB al ritmo de ~80-95 B/día. Y `desarrollo-agentico` /
`agent-solve-it` llevan 19 días **sin un solo byte de cambio**, aparcadas justo
bajo su techo — no están "rellenando", están simplemente inactivas ahí donde
el último split las dejó. Un headroom fijo (15-20%) le compra semanas a una
nota tranquila y días a la nota que está caliente esa semana — el dimensionado
correcto depende de cuál sea el frente activo, no de una tasa media de la KB.

### C. Qué era el relleno de 1.863 B en doctrina-agentes (08-17→08-19)

`git diff 27f897e 85b57e2 -- core/doctrina-agentes.md` (el diff real, no
inferido): **no es reacreción de lo recortado.** Son dos cosas nuevas y
legítimas:

1. Una reescritura de la sección "Contrato de memoria" para reflejar el
   cutover real de M6-04 (basic-memory → KB markdown/engine exo) — contenido
   que tenía que cambiar sí o sí, no era el material recortado en el triaje.
2. Una viñeta de doctrina genuinamente nueva: *"**No hagas polling**: el
   harness avisa cuando un hijo acaba; un watcher que lances hay que
   MATARLO, no basta con no crear más."* — aprendizaje de una sesión distinta,
   sin relación con lo que se había recortado el 08-17.

Conclusión con los datos delante: en este caso concreto **no es patología de
acumulación** (no volvió el mismo contenido que se cortó) — es doctrina nueva
legítima llegando a un ritmo (~80 B/día en fase activa) que un techo ya
ajustado al límite no puede absorber sin volver a rozarlo. Esto pesa a favor
del argumento de "el techo está dimensionado por debajo del ritmo real",
aunque solo se ha verificado este caso, no los demás mordiscos de la
cronología (§3 del cuerpo del informe sí incluye ciclos completos de
crecimiento libre de 3+ semanas — ese patrón es anterior al hook y no se puede
juzgar con el mismo criterio).

### D. Extra — ¿el 86% de M6-06 es trazable?

**Sí, trazable a un artefacto medido, no es humo.** Fuente:
`exo/docs/superpowers/specs/2026-08-22-m6-06-recall-punto-de-uso-design.md` y
`exo/docs/superpowers/consultas/2026-08-22-m6-06/consultor-m6-06.md`: dataset
de **272 prompts humanos reales de Paul desde el 01-ago** (55 archivos jsonl,
filtrados), contra los cuales se corrió el gate léxico real. Tabla citada en
`consultor-m6-06.md:529`: *Gate B (léxico) | 233 (86%) | 39 (14%)* — 233/272 =
85,66%, redondeado a 86%. Es una medición empírica sobre tráfico real, con el
dataset y el script (`gate-artefacto.py`) presentes en el repo, no una
estimación de boca.

## Adenda — Ronda 2 (fechar el enforcement y verificar la sierra)

### 1. Fechas finas de enforcement — confirmadas, con matiz sobre el "18-ago"

Confirmado con `git log`: `kbx budget` nace **11-jul** (`5cc755c`), el trinquete
nace **3-ago** (`41b3959`). El **pre-commit bloqueante** (`kb-precommit.sh`,
que corre `kbx ratchet --staged` + `kbx budget`) se **creó y commiteó el
17-ago a las 20:49:40** (`3aebd5d`, repo `agent-develop`) — **35 segundos
después** del commit del triaje que cerró las brechas (`27f897e`, 20:49:05,
mismo día, misma sesión). El movimiento a `7ade295` (18-ago) es una
**reubicación de fichero** cuando reflex entra en el monorepo exo, no la
instalación original — el script no cambia de contenido. `.git/hooks/pre-commit`
no está versionado, así que no hay commit que fije la hora exacta en que el
symlink local empezó a apuntar ahí, pero la secuencia (triaje → script del
hook, 35s después, misma sesión) deja claro que el bloqueo **entró en vigor el
17-ago por la noche**, no el 18. El "18-ago" del orquestador es la fecha del
*rename*, no de la activación.

**Entre el 3-ago y el 17-ago (14 días), nada hacía cumplir los techos
sellados.** No hay un solo commit ni entrada de bitácora en ese rango que
documente una corrida manual de `kbx budget`/`kbx ratchet` (verificado con
`git log --since --until` sobre ambos repos). El propio triaje del 17-ago es
la primera vez que alguien vuelve a mirar el trinquete desde que se selló.

**La curva plana de `agent-solve-it` desde el 3-ago (0 B en 19 días) no es
enforcement — es inactividad del proyecto.** Sin gate corriendo entre el 3 y
el 17, y sin pre-commit real hasta la noche del 17, no hay ningún mecanismo
que hubiera podido frenarla si alguien la hubiera tocado. Se quedó plana
porque nadie escribió ahí, no porque algo la contuviera.

### 2. La brecha del 04-ago — confirmada, y no fue solo doctrina-agentes

Verificado byte a byte: el commit `9238ade` (**2026-08-04, 02:16**) deja
**tres** de las nueve notas selladas el día anterior por encima de su propio
techo, no una:

| Nota | Techo sellado (03-ago) | Tamaño en 9238ade (04-ago) | Brecha |
|---|---|---|---|
| `core/doctrina-agentes.md` | 27.000 | 28.703 | **+1.703 B** |
| `Paul - perfil de trabajo.md` | 18.000 | 19.428 | **+1.428 B** |
| `Backlog — frentes abiertos.md` | 30.000 | 30.584 | **+584 B** |

Las otras 6 notas selladas (`desarrollo-agentico`, `pragmatismo-y-pivots`,
`agent-develop`, `agent-solve-it`, `lighthouses-bot`, `pguerrero.me`) estaban
dentro de su techo ese mismo día. Y las tres en brecha son, exactamente, **las
"tres notas core" que el commit `27f897e` del 17-ago dice que "vuelven bajo su
techo sellado"** — confirmado con `git log`: ninguna de las tres recibió ni un
solo commit entre el 04-ago y el 17-ago (verificado con `git log --follow
--since --until` por fichero, cero resultados en los tres casos salvo un
commit el mismo 17-ago). **Las tres pasaron 13 días exactos en brecha,
completamente intactas — ni se corrigieron ni se agravaron — hasta que el
triaje las tocó minutos antes de que naciera el hook que las habría bloqueado
desde el principio.**

Con la cronología del punto 1: sí, la brecha se explica enteramente por "no
había gate aún" — no hubo ninguna oportunidad mecánica de detectarla ni
corregirla en esos 13 días.

### 3. La hipótesis de la sierra — no se sostiene con la tasa que propone el abogado, aunque el patrón cualitativo (crece-choca-parte) sí aparece

**El ~10 B/día "en el muro" no aparece en mi serie.** Con los mismos datos que
ya reporté (ronda 1): doctrina-agentes hace ~80-130 B/día cerca de su techo
(159 B en 2 días 08-17→08-19, o 130 B en el tramo 08-18→08-19 si se mide día a
día); Backlog hace ~240 B/día pegado a su techo (29.754→29.994, 08-18→08-19).
Son 8-24× el ~10 B/día que cita el abogado, no algo compatible con esa cifra.

**El diferencial de 180× (1.800 vs 10) no sobrevive ni con la normalización
por actividad.** Con las tasas reales medidas (80-240 B/día cerca del muro,
~1.785 B/día lejos de él en Backlog 08-17), el diferencial real es de
**~7-22×**, no 180×. Aplicando el factor 5× de menos actividad en agosto que
el abogado invoca, el ~1.800 B/día "libre" pasaría a compararse contra un
"muro" de ~360 B/día esperado si la única causa fuera el volumen de
actividad — y lo medido (80-240) queda por debajo incluso de eso, así que el
freno cerca del muro es real, pero no del tamaño (180×) que sostiene la
metáfora de la sierra fina.

**Lo que sí aparece, y es distinto de una sierra continua, es un patrón más
tosco de tres fases**, visible completo en doctrina-agentes: (a) crecimiento
libre hasta perforar el techo (24.131→28.703 en 1 día, justo tras sellarse);
(b) **una meseta plana de 13 días en brecha, sin correcciones ni nuevo
crecimiento** (no es "desplazamiento a lo largo del muro", es inactividad
total mientras nada vigilaba); (c) corrección de golpe (triaje) seguida de
crecimiento lento pegado al nuevo techo *ya bajo enforcement real* hasta el
split del 22-ago. La meseta de la fase (b) no encaja con "sierra" — una sierra
predice fricción continua contra el muro, y aquí hay 13 días de nada seguidos
de un salto.

### 4. Las 4 notas sin waiver — el patrón NO es uniforme, y 3 de 4 debilitan la lectura de "atractor puro"

Tamaños de hoy verificados y correctos: `kbx.md` 12.490, `finanzas-empresa-x.md`
12.374, `code-intelligence.md` 12.331, `ocr-ml-docs.md` 12.326 — los cuatro
entre 98,6% y 99,9% del nominal 12.500. Pero la serie temporal completa de
cada una cuenta **dos historias distintas**:

- **`kbx.md` sí encaja con "creció hasta el techo y se frenó"**: nace 4.064 B
  (11-jul), crece orgánicamente en un día hasta 14.284 B (11-jul,
  **ya por encima** del nominal), se recorta a 9.687 (`89e5a77`, 12-jul), y
  vuelve a crecer orgánicamente **durante 3 semanas** hasta 12.494 (03-ago) —
  y ahí se queda, prácticamente sin moverse, 16 días (12.490 hoy). Este caso
  es el único de los cuatro con una curva de aproximación real al techo.

- **`finanzas-empresa-x.md`, `code-intelligence.md` y `ocr-ml-docs.md` NO
  encajan.** Las tres **nacieron ya por encima del nominal** (13.515, mínimo
  histórico; 13.918; 14.839 respectivamente), se recortaron **una sola vez**
  en pasadas de consolida masivas (`90c2e07`, 05-jul, que tocó 17 notas de
  golpe), aterrizaron justo debajo de 12.500 al recortar, y desde entonces
  llevan **36-48 días completamente congeladas** (+14 B de diferencia entre
  el recorte y hoy, en las tres). No hay "creció hasta el muro y se paró" —
  hay "alguien la recortó una vez a mano hasta quedar justo por debajo del
  límite que estaba mirando, y nadie la ha vuelto a tocar desde".

**Lectura con los datos delante**: la proximidad al nominal de estas 3/4 notas
no es evidencia de un atractor activo y recurrente — es el resultado
mecánico y esperable de cómo recorta una persona (o agente) cuando tiene un
número delante que tiene que cumplir: se recorta hasta *justo* por debajo, no
hasta la mitad, porque no hay razón para recortar de más. Eso es real y vale
la pena decirlo, pero es un fenómeno de **un solo evento de edición**, no una
fuerza continua tirando de la nota hacia el techo mes tras mes. El único caso
de la muestra que sí muestra una fuerza continua y repetida (crecimiento
orgánico prolongado, no un solo tijeretazo) es `kbx.md` — y las notas con
waiver de la ronda 1 (`Backlog`, `doctrina-agentes`, `perfil`) son las que de
verdad sostienen la lectura de atractor recurrente, porque en esas sí hay
múltiples ciclos completos de crecer→tocar el techo→recortar visibles en la
cronología (§2-3 del cuerpo del informe). Generalizar "los techos son
atractores" a partir de las 4 notas sin waiver sobregeneraliza: 3 de 4 son un
artefacto de una única edición manual, no un ciclo repetido.

## Adenda — Ronda 3 (ratificación, chequeo factual ítem por ítem)

**1. Headroom ≥15%, dimensionado 3-4 KB ≈ 2-5 semanas.** Tasas activas
medidas: doctrina-agentes ~80-130 B/día, perfil ~95 B/día — dentro del rango
80-180 B/día que cita la propuesta (no llegué a medir el techo de 180, pero
no lo contradigo). La aritmética exacta da un rango algo más ancho que "2-5
semanas": 3.000/180=16,7 días (2,4 sem) a 4.000/80=50 días (7,1 sem) — el
extremo lento se va a 7 semanas, no 5. Sobre "cadencia de consolida
19-22 días": los huecos reales entre eventos de consolidación medidos son
**1, 2, 4, 5, 14 y 22 días** (mediana 4,5) — 19-22 es el hueco *más largo*
registrado, no el típico, y el más reciente (17-ago→22-ago) fue de solo 5
días. Esto no contradice el dimensionado — si consolida corre más seguido de
lo asumido, el headroom sobra en vez de faltar — pero "19-22 días" no es una
cadencia representativa, es el outlier.

**2, 3, 6.** Decisiones de proceso/nomenclatura, no verificables con datos de
serie temporal — no tengo hechos que las contradigan. Sobre 6 (nunca recortar
el delta): un matiz real, documentado hoy mismo — `log/doctrina-agentes-bitacora.md:215`
registra que un puntero nuevo (el delta) **se recortó dos veces** para caber,
no que se rotara histórico existente. Es un caso pequeño (un puntero, no una
sección) pero es una excepción real y reciente a "nunca se recorta el delta",
no hipotética.

**4. Backlog como "dashboard" — mi serie NO lo sostiene.** La historia
completa (`git log --follow`, 172 commits) muestra el mismo patrón de
acreción-hasta-el-techo que el resto de la KB, no una dinámica acotada que
respire con la apertura/cierre de frentes: creció sin freno hasta 51.842 B
(02-ago, sin ningún waiver aún): 6× el nominal core. El mismo día en que se
selló su techo de 30.000 (`b3df97c`, baja a 23.155), **volvió a subir a
29.999 B en el mismo día** (`fb0f0c3`), 1 B bajo el techo nuevo. Se repitió
tras el triaje del 17-ago: de 26.425 sube a 29.997 **ese mismo día**
(`83333bd`), y llega a 29.994 el 19-ago. Backlog es, de hecho, **la nota más
rápida en volver a pegarse al techo de toda la muestra** (~1.785 B/día activo,
la tasa más alta medida) — lo contrario de un dashboard estable. Sí confirmo
el número de velocidad (~1.785 B/día); no confirmo que su dinámica sea de
tipo distinto a las demás notas con waiver. Vale la pena decirlo así en la
síntesis en vez de darlo por sentado.

**5.** Confirmado con datos ya reportados: 3 de 4 notas `core` con presupuesto
real solo existen hoy por waiver (Backlog, doctrina-agentes, perfil); solo
`core-index.md` cabe en el nominal.

**7. Priorización de la próxima /consolida.** Confirmado con datos actuales:
perfil a 9 B de su techo (17.991/18.000), doctrina-agentes a 33 B
(19.967/20.000) — ambas con actividad esta semana. `desarrollo-agentico` (49 B
de margen) y `agent-solve-it` (152 B de margen) llevan **0 B de cambio desde
el 03-ago** (19 días), confirmado en rondas anteriores. Priorizar las activas
es coherente con la serie: las inactivas no tienen presión de escritura detrás,
solo cercanía estática al techo.

**8. Fiabilidad del criterio "morder en el caso normal".** Los episodios de
mordida SÍ son observables y datables en commits con fiabilidad alta — cada
uno de los 7 episodios de la cronología (§2 del cuerpo) tiene un commit con
mensaje explícito (`rotate:`, `ratchet:`, `consolida:`, `triaje`). La
ambigüedad real está en "caso normal": el caso más reciente de mordida
(doctrina-agentes, 17→19-ago) fue en parte una reescritura forzada por un
cutover de arquitectura (M6-04, basic-memory→exo) — contenido que había que
cambiar sí o sí — mezclado con una viñeta de doctrina nueva genuina. Si eso
cuenta como "caso normal" o como "evento excepcional que no debería disparar
la banda", no está definido, y sí conviene cerrarlo ahora: dos pasadas de
consolida bajo el nuevo régimen van a ver poco volumen (2 notas activas,
19 días de muestra hasta ahora), y sin criterio explícito de qué excluir
(cutovers de arquitectura, migraciones puntuales) el conteo de "cuántas veces
mordió en caso normal" puede discutirse ex post en cualquier dirección.

**Separada — 33% de punteros a frío.** Confirmado con el dato primario:
`~/.claude/reflex-log.jsonl`, los 4 eventos reales `recall-inject-emitted` de
hoy (M6-06 en producción) contienen 12 permalinks en total, de los cuales
**4 apuntan a `archive/`** — exactamente 33,3%. Cifra correcta, verificada
sobre el log crudo, no solo repetida de segunda mano. Ojo: son solo 4 eventos
(la feature se desplegó hoy), muestra mínima.

**La afirmación histórica de cierre — la firmo tal cual.** "El A/B del abogado
comparaba 'sin instrumento' (...) contra 'régimen completo', y sus 19 días de
deriva cero (...) se explican por inactividad, no por enforcement" — es
exactamente lo que mi arqueología sostiene con los commits y fechas de las
rondas 1 y 2: cero hooks ni corridas manuales documentadas 07-11→17-ago, y
las dos notas con 0 B de cambio en 19 días no tuvieron ningún mecanismo que
las hubiera frenado si alguien las hubiera tocado.

## Notas metodológicas

- Bytes = `wc -c` sobre el fichero en disco, la misma métrica que usa
  `internal/budget/budget.go` (`os.Stat`, comparación estricta `>`, exclusión
  de `archive/`, `docs/`, `.superpowers/`). Los dos casos que salían "EXCEDE"
  en un primer barrido (`docs/superpowers/plans/2026-07-03-memoria-v2.md`,
  `docs/superpowers/specs/2026-07-03-memoria-v2-design.md`) están en `docs/`,
  excluido del cómputo real — no son offenders reales, son ruido del script.
- Solo lectura en todo momento: ningún commit, ningún write en `kb-demo`
  ni `kbx`; `~/.exo/index.db` se copió antes de consultarlo.
