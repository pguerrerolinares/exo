# Reglas literales al arranque: la KB llega como reglas, no como mapa (#184, diseño A)

**Fecha:** 2026-10-25 · **Estado:** aprobada en brainstorming; revisada por dos consultores (§11), pendiente de review del dueño
**Origen:** issue #184 (dilución del bloque de arranque), campaña M, debate con dos consultores (rastro en §9)

## 1. Qué se quiere

El dueño quiere **determinismo**: que Claude Code disponga siempre de su KB y
no la deje sin usar. Hoy la KB llega al arranque como un **mapa** (core-index
y punteros, 5.427 B), y el agente no la explota: en la campaña M buscó 0/120
veces y, cuando la regla venía dentro del bloque de 6 KB, la aplicó en 2/5
casos. La misma regla, sola, se aplicó con un éxito de 0,858.

Este diseño (A) hace que **las reglas que importan lleguen siempre, literales
y cortas, por el canal ya medido** (system prompt con framing de autoridad),
y adelgaza el arranque. Es la base estática. La recuperación por tarea (E: mod
`tool.call` con `deny` condicional y juez barato) queda fuera de esta spec y
se mide aparte (§8).

**Qué garantiza A y qué no.** Garantiza la entrega: las reglas elegidas están
en contexto en cada sesión y en cada subagente. No garantiza que estén las
reglas correctas para cada tarea (A es una selección estática), ni que el
modelo las aplique (techo medido ≈ 0,86 con la regla sola).

## 2. Evidencia de base

| fuente | resultado |
|---|---|
| Campaña M, `evals/campana-m/verdict.md` | A0 0,008 · O (regla sola por `SessionStart`) 0,858 · X (bloque `exo-recall.sh`, 6.166 B) 0,075 · N (`MEMORY.md`) 0,033. X buscó 0/120. Regla presente en el bloque: aplicada 2/5 (dilución) |
| techo-reglas-2, `evals/techo-reglas-2/verdict.md` | regla en system prompt con framing: 6/10 frente a 0/10 (PASA); por `SessionStart` a pelo (v1): 4/11 |
| Producción, `~/.claude/reflex-log.jsonl` | el canal de reglas está vacío: solo la nota de `exo` tiene `## Reglas duras` (1 regla); `bizkaia-now` da `sin_seccion` en cada arranque |
| Reglas extraídas por M, `~/.cache/exo-campana-m/redaccion/reglas-g{0,1,2}.jsonl` | 541 reglas (`id, nota, cita, regla, ambito, proyecto, check, generica`): 261 transversales, el resto en ~25 proyectos; 18/25 proyectos tienen ≤10 candidatas accionables |

## 3. Enfoques considerados

1. **A — reglas estáticas por system prompt + arranque podado.** Barato,
   reversible, canal medido, no rompe el contrato warn-only. Techo: 0,86 ×
   cobertura de la selección. **Elegido como base.**
2. **B — push por `PreToolUse` `additionalContext`.** Descartado: el
   `additionalContext` llega junto al resultado de la herramienta, después de
   ejecutarla (docs oficiales de hooks y transcript
   `bizkaia-now/3907750a…`, filas 151–156). La regla llega tarde para la
   acción que la dispara.
3. **D — gate `deny` ciego en el primer Edit/Write/Agent.** Dominado por E:
   el mismo `deny` previo a la acción, pero sin filtrar relevancia.
4. **E — mod `tool.call` con `deny` condicional** (`exo search --type fts`
   top-10, juez haiku, `deny` solo si hay una regla que aplica a esa acción).
   Es el único canal que actúa antes de la acción con la query exacta. Techo
   de recuperación medido en sondas M: top-3 10–17/40 según la query, top-10
   16–21/40. **Pospuesto:** se prototipa y se mide después de A (§8).
5. **`prompt.compose` dinámico por turno.** Descartado: cambiar el system
   prompt a mitad de sesión invalida el prefijo cacheado.

El dueño eligió A primero y, para el nivel global, la posición del consultor
3 (§5.3).

## 4. Piezas de engine y plugin

### 4.1 `ExcedeCap` deja de vaciar el proyecto

Hoy, con más de 10 reglas, `resuelve` devuelve `skip excede_cap` y el
proyecto se queda **sin ninguna regla** (`engine/src/reglas.rs:208-210`),
avisado solo por un `systemMessage`.

- `exo rules` entrega las **10 primeras** en orden de la sección y añade
  `truncated: <n>` al JSON; `exo-recall.sh` mantiene el aviso visible.
- `kb-precommit.sh` **rechaza** un commit que deje una sección `## Reglas
  duras` (de proyecto o global) con más de `CAP` reglas, con un mensaje que
  dice cuál desplazar. El recorte en entrega queda como red, no como camino
  normal.
- **El orden de la sección es la prioridad.** `/document` inserta cada regla
  nueva en su posición y desplaza la última si la sección está llena; nunca
  anexa al final de una sección llena.

### 4.2 Mapeo repo → nota con subdirectorios

`candidatas` solo mira `projects/*.md` de primer nivel
(`reglas.rs:140-168`). Varias notas canónicas viven en subdirectorios
(`projects/cge/`, `openwisdom/`, `finnk/`, `pguerrero-music/`…) y su nombre de
directorio no siempre casa con la clave del repo: la canónica de
`code-graph-engine` está en `cge/`, la de `open-wisdom` en `openwisdom/`.

- Candidatas = `projects/*.md` ∪ `projects/*/*.md` (un nivel), con el mismo
  ranking de hoy: `slug` > stem. **El nombre del directorio no cuenta.**
- La siembra añade `slug: <clave del repo>` a la nota canónica de cada
  proyecto que reciba reglas, así que el caso normal resuelve por `slug`.
- Varias candidatas con el mejor rango: `ambigua`, como hoy.

### 4.3 Nivel global

Hoy no existe canal para las reglas transversales.

- Nota core propia, **`core/reglas-duras.md`**, con una sección `## Reglas
  duras` leída por el mismo parser (`lee_seccion`) y el mismo `CAP` que las
  de proyecto (un solo valor, §7.1). No va en `core-index.md`: está al
  88,6 % de su presupuesto (4.432/5.000 B) y una sección intercalada se
  colaría en la doctrina que `compose-inject.sh:74` extrae por awk.
- `exo rules --json` devuelve `global` (lista) junto a `rules` (proyecto). Un
  repo sin nota o sin sección sigue recibiendo las globales.
- El mod `prompt.compose` (`plugins/exo/hooks/register.ts`) mete las dos
  listas, globales primero, cada una con su encabezado (§10, D10).
  `compose-inject.sh` hace lo mismo para subagentes.
- El latido (`hb-<sid>`) y el testigo de Stop cuentan las dos listas (§10, D3).

### 4.4 Reglas en subagentes

Hoy `bloque_reglas` corta a `1024 − framing − 80` B (≈ 800 B,
`compose-inject.sh:104`) y va fuera del cap de 2.048 B del resto del bloque.
Dos listas al `CAP` con dos encabezados son ≈ 2,4 KB: sumadas sin más, el
bloque de `SubagentStart` llegaría a 4,5–5 KB, el tamaño del bloque X que
diluyó en M.

- Los subagentes reciben **las dos listas completas** (globales y de
  proyecto), igual que el padre.
- Cuando hay reglas, el perfil deja de inyectar `doctrina_compacta` y
  `rutas` (las reglas ocupan su papel) y conserva la cabecera y la doctrina
  de `executor.md`.
- Tope total del bloque de `SubagentStart`: **3.072 B**. Si no cabe, se
  cortan reglas por el final de cada lista, con `inject-rules-truncated`.

### 4.5 Arranque podado

`exo-recall.sh` deja de inyectar el core-index entero y los punteros de
actividad reciente.

- Inyecta como máximo **1.500 B**: la sección `## Contrato de memoria` del
  core-index (≈ 990 B tal como la sirve `exo recall --content`) y los avisos operativos que ya
  emite (gate pendiente, superpowers, reglas no entregadas).
- **`core/core-index.md` no se edita** por este cambio: `compose-inject.sh`
  lee por awk sus secciones `## Doctrina compacta` y `## Cores` para los
  subagentes (`compose-inject.sh:74`). Lo que cambia es qué
  inyecta `exo-recall.sh`, no el fichero.
- Los escritores no dependen del bloque de arranque (`/document` usa `exo
  search`/`recall`; `/distill` edita core-index como nota). Sí hay que
  reescribir el chequeo «arranque servido incompleto» de
  `skills/distill/chequeos.md:85-93`, que asume el cap de 6.144 B.
- La reafirmación de reflejos en `source=compact` se mantiene.

## 5. Contenido: qué reglas entran

### 5.1 Forma de cada línea

Cada línea de `## Reglas duras` es la **`cita` literal de la nota del dueño**,
no la paráfrasis del LLM que la extrajo. Si la cita no se sostiene sola como
imperativo, se reescribe a mano en el PR de siembra. La siembra va en un PR
revisado por diff; no hay lint aparte.

### 5.2 Proyecto: ranking determinista

Candidatas: reglas de M con `generica=false` y `ambito` ∈ {`codigo`, `git`,
`dominio-proyecto`}, asignadas a su repo por una **tabla `proyecto` → repo
explícita** en el script de siembra: el campo `proyecto` del JSONL no es la
clave del repo en al menos 7 casos (`pguerrero.me`, `open-wisdom`, `cge` y
`code-graph-engine` que son el mismo repo, `lighthouses_aicontest`, `CV`,
`ai-news-platform`, `kbi-broker`). Orden:

1. primero las que A0 **no** cumple sin regla (criba de M; definición fija:
   una regla se considera «A0 ya cumple» cuando todas las réplicas a0 de su
   tarea en `~/.cache/exo-campana-m/corridas-criba/<id>/a0-r*/check.rc` dan 0;
   sin corrida, no se excluye);
2. las que tienen `check`;
3. `git`/`codigo` antes que `dominio-proyecto`;
4. tier `core` antes que `stable` (según la nota de origen);
5. orden de fichero.

Se toman las `CAP` primeras. En los proyectos con ≤10 candidatas entran
todas; en los que desbordan (al menos 7, más `cge` + `code-graph-engine`
fundidos), el dueño revisa el corte en el PR.

**Presupuestos de nota.** Sembrar desborda notas que ya van justas
(`llm-apps` 11.869/12.500 B, `ocr-ml-docs` 11.941 B; `cge` y `kbx` se quedan
sin aire). Esas notas se parten antes de sembrar, según la doctrina de la KB;
el plan lo cuenta como paso previo, no como sorpresa del precommit.

La exclusión «A0 ya cumple» depende del modelo con el que se hizo la criba.
Al cambiar de modelo por defecto, una regla excluida no se borra: baja al
final del ranking hasta que se rehaga la criba.

### 5.3 Global: elige el dueño

Las 76 candidatas transversales accionables (de 261) no caben en 10 plazas, y el
script ciego cubre solo 2 de las 16 sondas transversales de M. El dueño elige
las `CAP` globales con el ranking de §5.2 como propuesta.

**El nivel global no tiene instrumento propio** (posición del consultor 3,
elegida por el dueño). Los detectores existentes (`git-c`, `zero-residuo`,
`verify-before-done`) cubren como mucho 3 reglas, y `git-c` está confundido
(372 de 607 disparos son de subagentes y la regla ya estaba en `CLAUDE.md`).
El nivel global queda cubierto por la medida de dilución (§7.1); **su efecto
regla a regla no se mide**, y esta spec lo deja dicho.

### 5.4 Mantenimiento

`/document` decide al escribir si una decisión es regla dura (proyecto o
global) y la inserta con su posición (§4.1). `/distill` poda las que ya no
aplican. No hay cola de curación humana.

## 6. Error handling

- Engine: cualquier fallo resolviendo reglas sigue siendo `skip` con
  `reason`, nunca rompe el arranque. `global` vacío no es error.
- `ambigua` en subdirectorio se trata como hoy (aviso visible, sin reglas de
  proyecto); las globales se entregan igual.
- Recorte por `CAP` en entrega: aviso visible + campo `truncated`; el
  precommit impide que sea el estado normal.
- Subagentes: si el presupuesto corta, `inject-rules-truncated` lleva
  `session_id`/`agent_id` (#188).
- Cuentas con la política de Team/Enterprise: el mod cae al canal
  `prompt.submit`, que entrega las reglas como contexto de usuario, no en el
  system prompt. El 6/10 de techo-reglas-2 no aplica a ese canal.

## 7. Testing y medición

### 7.1 techo-reglas-3 (dilución), en paralelo con el código

Mismas 10 tareas de techo-reglas-2, canal system prompt con el framing
actual. Brazos: regla gold sola / gold + 9 distractoras reales / gold + 19,
k=2, unas 60 corridas (≈ 5 USD). Distractoras: reglas reales de las 541,
sin relación con la tarea, repartidas bajo los dos encabezados como en
producción. Pre-registro antes de correr. **Adjudica el brazo gold + 19**,
que es lo que se despliega con dos listas al `CAP` 10. Su resultado fija el
`CAP` antes de sembrar:

| gold + 19 | `CAP` |
|---|---|
| ≥ 5/10 | 10 |
| 4/10 | 7 |
| ≤ 3/10 | 5 |

Si el pre-registro y la corrida no caben en la semana del código, se siembra
con `CAP` 10 y se baja después (bajar el `CAP` reordena, no borra).

### 7.2 Brazo `Ap` en el harness de M

40 sondas × 3, system prompt con las reglas sembradas tal como se despliegan
y el arranque podado. Se reportan **cobertura** (sondas cuya regla está
dentro) y **aplicación dada cobertura**. La cobertura de las globales no es
interpretable (el dueño diseñó M y eligió las globales); se reporta aparte y
no adjudica.

Unidad, umbrales y réplica: `evals/ap/preregistro.md` (#190), que sustituye
al «< 0,60» original. Ese umbral no fijaba la unidad, y con la base de
techo-reglas-2 no discriminaba. Resumen: control `Ap1` (la regla gold sola,
por el mismo canal) como réplica; gate primario pareado `Ap − Ap1` sobre la
fracción de réplicas por sonda (bootstrap unilateral 97,5 %); secundario
absoluto con sondas ≥2/3 < k(N), solo si `Ap1` está a nivel; umbrales en
función de N, sellados por `evals/ap/umbral.py`.

### 7.3 Condiciones que invalidan A

Cada una basta por sí sola:

- **Canal + dilución:** techo-reglas-3 ≤ 3/10 con gold + 9 (diluye incluso
  con la mitad de reglas).
- **Selección de proyecto:** `Ap` cae según `evals/ap/preregistro.md`
  (con réplica de `Ap1`; sin réplica no se adjudica).

Si se cumple alguna, las reglas estáticas no compensan y el siguiente paso es
E.

### 7.4 Tests de código

- `engine`: `ExcedeCap` entrega 10 + `truncated`; nota en subdirectorio
  resuelve por `slug` aunque el directorio no case (caso `cge/` para
  `code-graph-engine`); dos candidatas con el mismo rango dan `ambigua`;
  `global` se lee de `core/reglas-duras.md` y llega con o sin nota de
  proyecto, también en `skip`; `--check` rechaza `CAP+1`.
- `test-compose-inject.sh`: con reglas, el bloque de subagente omite
  `doctrina_compacta` y `rutas` y no pasa de 3.072 B.
- `test-document-remind.sh`: `ss` con `n > 0` y `hb` con `n = 0` avisa
  aunque `status` sea `skip`.
- `kb-precommit`: sección con `CAP+1` reglas se rechaza; con `CAP` pasa.
- `register.test.ts`: el bloque compuesto lleva globales y de proyecto, en
  ese orden.
- `test-compose-inject.sh`: las dos listas caben al `CAP` vigente sin
  `inject-rules-truncated`.
- `test-exo-recall*.sh`: el bloque ≤ 1.500 B; golden nuevo.

## 8. Fuera de alcance

- **E** (mod `tool.call` + juez + `deny` condicional): spec propia tras A,
  medida contra `Ap` (cae si el límite inferior de Δ(E − Ap) ≤ 0, si hay
  `deny` en más del 20 % de neutras o si la latencia mediana por Edit pasa de
  2 s).
- La capa de reglas como artefacto de primera clase (globs, ámbito por ruta).
- Retirar `search-first` (#183 sigue abierta; los datos de producción no lo
  refutan).

## 9. Rastro de la decisión

- Consultor 1 (fable), cuatro rondas: notas `consultor-184.md` a
  `consultor-184-r4.md` en el scratchpad de la sesión. Aportó el timing de
  `PreToolUse`, la latencia de `exo search` (hybrid 1,6–2,3 s, `fts` ≈ 0 s),
  el techo de recuperación, el footgun de `ExcedeCap` y el ranking de §5.2.
- Consultor 3 (fable, fresco): `consultor3-184.md`. Aportó el mapeo con
  subdirectorios, el presupuesto de subagentes, el no tocar `core-index.md`
  y la posición sobre el nivel global que eligió el dueño.
- Cifras que el orquestador re-ejecutó: timing en el transcript, latencias,
  BM25 top-3 (10, 12, 12, 17/40), cobertura ciega (19/24 proyecto, 2/16
  transversal), `ExcedeCap`, subdirectorios, cap de subagentes. La exclusión
  «A0 ya cumple» dio 43, 57 y 67 según el criterio; §5.2 fija la definición.

## 10. Decisiones cerradas tras el builder check (del orquestador)

Un builder fresco leyó la spec como quien tiene que planificarla. Sus 10
preguntas se cierran sin cambiar lo que pidió el dueño; son decisiones del
orquestador, corregibles en la review.

- **D1 · De dónde saca el precommit el `CAP`.** El `CAP` vive solo en el
  engine (`const`, un único valor para proyecto y global). `exo rules` gana
  un modo de validación sobre una raíz de KB (`exo rules --check --kb
  <dir>`) que recorre `projects/*.md`, `projects/*/*.md` y
  `core/reglas-duras.md`, y sale ≠ 0 si alguna sección supera
  el `CAP`. `kb-precommit.sh` lo llama sobre su snapshot staged. No vale `exo
  lint` (exige índice, `kb-precommit.sh:72`). Superficie nueva del engine:
  entra en `contrato-engine`.
- **D2 · Forma del JSON.** `global: [..]` y `truncated: <n>` (0 si no hay
  recorte) van en `data` en **las dos** variantes, `ok` y `skip`. `status` y
  `reason` siguen hablando solo del nivel de proyecto, así que `sin_seccion`,
  `seccion_vacia` y sus avisos no cambian de significado. `excede_cap`
  desaparece como salida (ahora es `ok` + `truncated`); el `case` de
  `exo-recall.sh` que lo menciona se limpia. Los tres consumidores
  (`register.ts`, `compose-inject.sh`, `exo-recall.sh`) leen `global` aunque
  `status` sea `skip`.
- **D3 · Testigos.** `n` pasa a ser el total entregado (`rules` + `global`)
  en `ss-<sid>` y en `hb-<sid>`, y se añade `n_global` a los dos. El testigo
  de `document-remind.sh:38-56` deja de decidir por `status` (que con D2
  habla solo del proyecto) y decide por `n`: **`n > 0` en `ss` y `n = 0` en
  `hb` es «no entregó»**. Así se ven las globales perdidas en repos sin nota
  (`sin_seccion` en 314 de 339 arranques), que hoy el testigo calla.
- **D4 · Fuente del contrato reducido.** `exo-recall.sh` extrae la sección
  `## Contrato de memoria` del core-index servido por `exo recall --content`
  (la ruta, el índice y los fallbacks actuales se mantienen) y descarta el
  resto. La verdad sigue en la KB y `/distill` la ve. Hoy esa sección ocupa
  ≈ 990 B servida; si crece, el cap de D5 la recorta con aviso.
- **D5 · Qué cubren los 1.500 B.** Contrato + avisos operativos, con cap en
  código y evento `recall-truncated` si actúa. La reafirmación de reflejos
  en `source=compact` va fuera del cap (solo tras compactar). El golden
  fija el caso normal.
- **D6 · Siembra.** Orden: código → techo-reglas-3 (fija `CAP`) → siembra
  (con la salvedad de calendario de §7.1).
  La siembra es una puerta humana (el dueño elige las globales y revisa el
  corte de los 7 proyectos que desbordan), no una tarea de ejecutor; el plan
  la deja como paso manual con el script del ranking como ayuda. Si una nota
  supera su presupuesto de bytes al añadir reglas, se aplica la doctrina de
  la KB (partir o consolidar), nunca subir el techo. El presupuesto de
  subagentes (§4.4) se fija tras la siembra, con las reglas reales.
- **D7 · Gates.** El código se construye con `CAP = 10` y techo-reglas-3
  solo puede bajarlo (cambio de una constante). Si se cumple §7.3, no se
  revierte nada automáticamente: se para, se reporta y se decide con los
  datos (el siguiente paso previsto es la spec de E). Las medidas §7.1 y
  §7.2 tienen pre-registro propio; el plan de código las referencia, no las
  incluye.
- **D8 · Subdirectorios.** Sustituida por §4.2 tras la review: candidatas
  `projects/*.md` ∪ `projects/*/*.md`, `slug` > stem, sin mirar el nombre
  del directorio; la siembra añade `slug:`.
- **D9 · Formato de cada regla.** Una línea `- ` por regla, una sola línea,
  ≤ 300 B. Los wikilinks se dejan como texto. Una cita de varias líneas o
  más larga se reescribe a mano en el PR de siembra (§5.1).
- **D10 · Framing.** Dos encabezados con el mismo cuerpo de autoridad:
  «Reglas duras del dueño» (globales, primero) y «Reglas duras del proyecto»
  (las actuales). El tope de §4.4 cuenta los dos.

## 11. Review de la spec escrita (dos consultores)

Notas: `review-spec-184-c1.md` y `review-spec-184-c3.md` en el scratchpad de
la sesión. Ambos dieron «no lista para plan» con dos bloqueos coincidentes,
ya aplicados:

- **Globales fuera de `core-index.md`** → `core/reglas-duras.md` (§4.3).
- **Mapeo de subdirectorios** que resolvía 0 de los 2 proyectos reales con
  repo → `slug` en `projects/**` sin mirar el directorio, y la siembra añade
  `slug:` (§4.2).

IMPORTA aplicados: techo-reglas-3 adjudica con gold + 19 (§7.1); las
condiciones de invalidación son independientes (§7.3); bloque de subagentes
acotado a 3.072 B (§4.4); testigo decide por `n` (D3); tabla `proyecto` →
repo y notas a partir antes de sembrar (§5.2); canal degradado de Team
(§6). Descartado: lint de existencia de la cita (el PR revisado por diff lo
cubre, posición del consultor 3).
