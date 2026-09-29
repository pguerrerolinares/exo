# Campaña K — erratas al pre-registro congelado

Aprobadas por Paul el 2026-09-29, tras la auditoría
`evals/ablacion-k/auditoria-f0-t0.md`. Se registran **antes** de extraer el
pool (Task 2) y **antes** de cualquier corrida de la fase 1. Ninguna se
decidió viendo resultados de la ablación, porque todavía no hay ninguno.
El verdict final las cita.

## E1 — D3: el CLAUDE.md de los brazos va sin la sección de memoria (B1)

**Problema.** La línea 19 de `~/.claude/CLAUDE.md` ordena usar «SIEMPRE»
`exo search` y la KB `wisdom-paul`. Con D3 tal como estaba, A0 recibe la
orden de usar exo. Pasa una de dos cosas: si la herramienta está disponible
hay fuga, y si no, A0 queda penalizado de forma artificial (turnos perdidos
buscando algo que no existe).

**Errata.** En los cuatro brazos se inyecta el CLAUDE.md global **sin la
sección `## Memoria de sesiones`**, porque sin exo esa sección no
existiría. El resto del fichero (perfil, estilo, git) sigue igual en todos
los brazos, así que el sentido de D3 no cambia: la pregunta sigue siendo si
exo aporta por encima de la memoria nativa. Las instrucciones de memoria
llegan solo por el core-index (A1–A3). El recorte lo hace el harness en cada
corrida y se registra el sha256 del resultado.

**Aislamiento añadido en A0:** `--disallowedTools` sobre lecturas de la ruta
de la KB y de `~/.exo/`, y `exo` fuera de alcance (stub, como A1).

## E2 — §9: la fuga se detecta por canal y por ruta, no por texto

Sustituye el canario literal de §9, que da falsos positivos
(`evals/ablacion-k/task0-recon.md`). Cuenta como fuga en un brazo que no
debería tenerla cualquiera de estos casos:

- eventos de hook (`hook_*`, `additionalContext`) fuera de lo que el brazo
  cablea;
- un `tool_use` que invoca `exo` (A0/A1), también mediante wrappers o rutas
  absolutas;
- `recall-inject-emitted` en el log del brazo (A0/A1/A2);
- **(añadido por B1)** un `tool_use` cuya entrada contiene la ruta de la KB
  o de `~/.exo/` (A0).

Las apariciones de la cadena `=== Recall exo` dentro de un `tool_result`
**no** cuentan.

## E3 — D2: parada solo por consumo y regla simétrica de corte (I5)

Sustituye las tandas «si el ritmo es sostenible» y el freno de
`--max-budget-usd 3` fijados en `06450e0`.

- Tandas de 40 corridas. **Única regla de parada entre tandas:** la media de
  tokens de entrada por corrida de la tanda supera **2M**. Se evalúa sin
  mirar ningún resultado: ningún éxito se computa hasta cerrar la etapa.
- Tope por corrida: `--max-budget-usd 10` (unidad de lista, usada como
  límite de consumo). **Una corrida cortada cuenta como fallo, en cualquier
  brazo**, y la tasa de cortes por brazo se reporta.
- Se mantiene el tope de etapa: 240M tokens de entrada.

## E4 — §5/§7: el pool de S1 sale solo de la fuente 2, ampliada

Aprobada por Paul el 2026-09-29, cuando saltó el circuit breaker de §7 en la
Task 2 y antes de extraer ninguna regla ni redactar ninguna tarea.

**Problema.** La fuente 1 (correcciones en transcripts) está casi vacía. Los
transcripts solo se conservan desde el 2026-09-12, por la limpieza por
defecto de Claude Code: hay 484 prompts reales, 6 coincidencias con los
patrones (5 de ellas «otra vez») y 0 citas de un learning por título. Y
`learnings/` solo tiene 14 notas, así que no llega a 40.

**Errata.**

1. **Fuentes de S1:** las notas de `wisdom-paul` con `tier: core` o
   `tier: stable`, excluyendo `backlog/` (estado, no reglas), `research/`
   (análisis) y `archive/`. Son 47 notas (449.369 B) en el commit de la KB
   `389a0da`. Lista en el directorio privado del pool.
2. **Extracción:** agentes frescos (sonnet) listan las reglas accionables de
   cada nota con un criterio fijo (brief en `evals/ablacion-k/pool/`). No
   eligen ni redactan tareas.
3. **Muestreo:** las reglas se ordenan con la semilla `20260923`. Se toman
   en ese orden, saltando las vetadas (veto de Paul con motivo registrado),
   hasta 40. Si no llega, se para y se escala otra vez.
4. **Redacción:** otro agente fresco convierte cada regla en una tarea
   natural cuyo camino por defecto la viola, más su check. El prompt de la
   tarea no puede mencionar la regla.
5. **Snapshot de KB:** el commit de `wisdom-paul` inmediatamente anterior a
   la congelación del gold (paso 2 de §12), igual para todas las tareas
   sintéticas.
6. **Amenaza que cambia de forma (§8):** el sesgo a favor de exo ya no viene
   de «correcciones reales», sino de tareas construidas para que la regla
   importe. S1 mide si el agente sigue una regla que está en la KB en una
   situación diseñada para ello. El verdict lo reporta así.

**Complemento (opción b):** `cleanupPeriodDays: 365` en
`~/.claude/settings.json` desde el 2026-09-29, para que una réplica futura
pueda usar la fuente 1 real.

## E5 — §5: fuente y filtro de S2 (neutral)

Aprobada por Paul el 2026-09-29 (opción a), antes de muestrear S2.

**Problema.** Los repos propios sin notas en la KB solo dan unos 4 commits
útiles; casi todos los repos de Paul aparecen en la KB.

**Errata.**

1. **Fuente:** los repos de `eval-repos/` que la KB solo nombra como corpus
   de cge, sin nada sobre su código, y que son ligeros de ejecutar (Python,
   tests con sqlite): **django-oscar** y **wagtail**. Se clonan con historial
   aparte, en `~/.cache/exo-ablacion-k/s2/`, sin tocar `eval-repos/`. Los de
   JS y Java quedan fuera por el coste de setup.
2. **Filtro mecánico:**
   - commits no-merge desde el 2025-09-29;
   - asunto que casa con `fix|bug|add|support|allow|handle|prevent|feat|correct`
     y no con `docs|chore|release|bump|revert|translation|version`;
   - toca ≥ 1 fichero de test;
   - toca 1–3 ficheros `.py` de código (sin migraciones), con ≤ 60 líneas
     cambiadas;
   - no toca ningún otro fichero.

   Resultado: 111 candidatas (wagtail 98, django-oscar 13). Script:
   `evals/ablacion-k/pool/s2_candidatas.py`.
3. **Muestreo:** orden con la semilla `20260923`. Se toman en ese orden hasta
   20, saltando las que fallen la validación de la Task 4: el test del commit
   debe fallar en el padre y pasar en el commit, en el entorno fijado. No se
   equilibra por repo.
4. **Tarea y check:** el prompt sale del asunto y del cuerpo del commit
   (redactado por un agente fresco sin ver el diff). El check es el test que
   añadió el commit.

## E6 — §5: el veto de S1 lo hace un agente fresco, no Paul

Aprobada el 2026-09-29, antes de vetar ninguna regla.

**Problema.** §5 asignaba el veto a Paul suponiendo que conoce el contenido
de la KB. No es así: la KB la escribe el skill `/document`, y la implicación
de Paul fue la arquitectura. Paul no puede juzgar qué reglas son viables
mejor que un agente. Además, es la parte con sesgo declarado.

**Errata.**

1. **Quién veta:** un agente fresco (sonnet) que recibe las reglas en el
   orden de la semilla y un criterio fijo
   (`evals/ablacion-k/pool/brief-veto.md`). No sabe que el experimento mide
   memoria ni qué es exo. Puede inspeccionar en modo lectura los repos de
   `~/Documentos/proyectos` para juzgar la viabilidad.
2. **Criterio de veto (cerrado):** `externo` (necesita un servicio vivo,
   una credencial, hardware o datos que no están en disco) · `multisesion`
   (no se puede observar en una sesión de ≤ 40 turnos) · `no-regla` (es un
   hecho, una opinión o una descripción) · `duplicada` (repite una regla
   anterior en el orden).
3. **Selección:** se toman en orden las 40 primeras no vetadas. Paul no
   revisa regla a regla. Si no llegan 40 entre las 80 primeras, se amplía
   la lista en el mismo orden.
4. **Hallazgo para el verdict:** las reglas de S1 las escribieron agentes
   (vía `/document`), no Paul. S1 mide si exo transmite a un agente nuevo lo
   que dejaron escrito agentes anteriores.

## E7 — Task 4: segundo filtro al redactar

Registrada el 2026-09-29, tras el veto de E6 y antes de redactar ninguna
tarea.

**Resultado del veto (E6):** 78 `ok`, 1 `externo`, 1 `no-regla`, sobre 80.
El agente aplicó «en duda, `ok`» y decidió solo por el texto, sin abrir
repos. La selección son las 40 primeras `ok`, con el corte en n = 42
(`seleccion-s1.txt`, sha256 `79e5d707c8d22f97…`).

**Errata.** Como el veto fue permisivo, el redactor de la Task 4 puede
declarar una regla `no-convertible` si no consigue una tarea de una sesión
con un check **objetivo**: un script, o una rúbrica binaria de 1–3 ítems
observables. Tiene que dar el motivo. Esa regla se sustituye por la
siguiente `ok` en el orden de la semilla. Las sustituciones y sus motivos se
reportan en el verdict. El redactor no ve ningún brazo ni ningún resultado.

### E1 — ampliación (2026-09-29)

Al construir el snapshot se vio que la cabecera del CLAUDE.md global también
apunta a la KB («Fuente de verdad: nota … en la KB `wisdom-paul` (servida por
el engine exo). Si necesitas contexto profundo, búscala»). Por el mismo
motivo que E1, esa línea se quita en los cuatro brazos. `preparar.sh` falla
si en el CLAUDE.md de los brazos queda cualquier mención a `wisdom-paul`, a
`exo` o a la sección de memoria.

### E2 — ampliación (2026-09-29, canario de los cuatro brazos)

Un `tool_use` que invoca `exo` en A0/A1 y recibe la salida del stub
(`exo: orden no encontrada`, rc 127) **no** es fuga: no pasa información.
Se registra como aviso por corrida y se reporta la tasa por brazo. Sigue
siendo fuga una invocación que no bloquea el stub. Motivo: el core-index de
A1 nombra exo varias veces, así que un intento es plausible, y con la regla
literal §9 pararía la campaña por algo inocuo.

## E8 — repos excluidos por Paul y ciclo de arreglo tras la auditoría del gold

Registrada el 2026-09-29, después de la auditoría del gold de S1
(`~/.cache/exo-ablacion-k/gold/auditoria-s1.md`, privada: 15 OK · 21
ARREGLAR · 4 DESCARTAR) y antes de correr ninguna tarea.

1. **Repos excluidos por Paul:** `pguerrero-music` y `agent-solve-it` no se
   usan como fuente. Las tareas que los usaban (`g0-149`, `g0-159`, `g1-177`,
   `g2-97`) se reescriben sobre un repo sintético (`setup.sh`). Si no es
   viable, se declaran `no-convertible` y se sustituyen en orden (E7).
2. **Descartadas por la auditoría:** `g1-1`, `g1-128`, `g2-103`, `g2-2`. Se
   sustituyen por las siguientes `ok` del orden (E7).
3. **Contrato común de check, corregido:** `check.sh <workdir> <transcript>
   <commit_inicio>`. Toda comparación del estado final se hace contra
   `<commit_inicio>`, nunca contra `HEAD` (si el agente commitea, `HEAD` ya
   no es el punto de partida). Las reglas de proceso no exigen un orden
   estricto cuando «hacer → verificar → corregir» cumple el espíritu.
4. **Checks léxicos sobre texto libre** (`g1-38`, `g1-89`, `g2-102`,
   `g2-134`, `g2-63`): quedan pendientes de la decisión de Paul sobre el κ
   (§5). Con κ, pasan a rúbrica con juez LLM; sin κ, se sustituyen.
5. **Segunda auditoría:** toda tarea tocada en este ciclo vuelve a pasar por
   un auditor fresco antes de congelar.

## E9 — todos los checks de S1 son scripts; sin juez LLM ni κ

Decidida por Paul el 2026-09-29 (opción b), antes de correr ninguna tarea.

- Las tareas cuyo cumplimiento solo se veía en la prosa del agente (`g1-38`,
  `g1-89`, `g2-102`, `g2-134`, `g2-63`) y las de tipo `rubrica` (`g1-108`,
  `g1-160`, `g1-27`, `g1-77`) se **sustituyen** por las siguientes `ok` del
  orden (E7). Las sustitutas solo admiten check `script`.
- §5 (juez LLM, adjudicación de Paul y κ ≥ 0,60) **queda sin efecto**: no
  hay tareas juzgadas por LLM. S2 ya era script (el test del commit).
- **Sesgo declarado:** esto excluye las reglas que solo se observan en lo que
  el agente escribe (brief, comunicación, decisión). S1 cubre reglas cuyo
  cumplimiento deja rastro en comandos o en ficheros.
- El gold activo se fija por lista explícita (`activas-s1.txt`) en la
  congelación. Los directorios descartados o sustituidos quedan fuera
  aunque sigan en disco.

### E8 — regla de parada del ciclo de auditoría (2026-09-29)

Segunda auditoría (`auditoria-s1-r2.md`, privada): 23 tareas → 11 OK · 12
ARREGLAR · 0 DESCARTAR. Los agujeros de la primera ronda quedan cerrados y
ninguna tarea se ha vaciado. Los 11 arreglos que quedan (`g1-77` sale por
E9) se aplican **incorporando las sondas del auditor como pruebas de
regresión** en `pruebas/`. **No hay tercera auditoría completa:** cada ronda
encuentra fallos de menor orden. El error residual de los checks se declara
en el verdict como amenaza a la validez (medición con ruido, no sesgo
dirigido: los checks no ven el brazo).

### E9 — ampliación: reglas circulares y servicios vivos (2026-09-29)

Tras redactar las sustitutas de E9 se descartan dos, con criterios fijados
antes de correr nada:

- **Circular** (`g1-26`): una regla cuyo cumplimiento consiste en usar el
  propio sistema de memoria («busca en la memoria antes de actuar») mide si
  el brazo tiene la herramienta, no si el agente sigue una regla. Favorece a
  A2/A3 por construcción. Mismo criterio aplicado antes a `g1-1`.
- **Externo** (`g1-115`): violar la regla implica llamar de verdad a la API
  privada de un tercero desde la máquina de Paul. Es el veto `externo` de E6.

Junto con `g0-27` (no-convertible), se sustituyen en orden (E7). Las
sustitutas tampoco pueden ser circulares ni depender de servicios vivos.

## Incidencia I1 — corte de infraestructura en la tanda 3 (2026-09-30)

El proceso que encadenaba las tandas 2–6 lo mató el límite de tiempo de la
herramienta que lo lanzó, a mitad de la tanda 3. Las tandas 1 y 2 están
completas (sin breakers). De la tanda 3 se completaron 29 corridas antes del
corte; las incompletas **se repiten todas, de ambos brazos**, con
`K_REANUDAR=1` (salta solo las corridas con `fin` en `meta.json`). El corte es
externo e independiente de cualquier resultado, y no se ha mirado ninguna tasa
de éxito. Desde aquí, cada tanda se lanza como proceso propio.
