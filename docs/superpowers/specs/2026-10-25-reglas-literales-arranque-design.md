# Reglas literales al arranque: la KB llega como reglas, no como mapa (#184, diseño A)

**Fecha:** 2026-10-25 · **Estado:** aprobada en brainstorming, pendiente de review de la spec escrita
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
| Reglas extraídas por M, `~/.cache/exo-campana-m/redaccion/reglas-g{0,1,2}.jsonl` | 541 reglas (`id, nota, cita, regla, ambito, proyecto, check, generica`): 256 transversales, el resto en ~25 proyectos; 18/25 proyectos tienen ≤10 candidatas accionables |

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
proyecto se queda **sin ninguna regla** (`engine/src/reglas.rs:207-209`),
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
(`reglas.rs:139-168`). Los proyectos que viven en subdirectorio
(`projects/pguerrero-music/`, `cge/`, `code-graph-engine/`, `code-graph-go/`,
`finnk/`, `openwisdom/`) dan hoy `sin_nota`.

- Para `projects/<dir>/` cuyo nombre case con la clave del repo (misma regla
  `slug` > stem que hoy), la candidata es la nota canónica del directorio: la
  que lleva `slug: <clave>` en frontmatter o, si no hay ninguna, la que se
  llama como el directorio. Si quedan varias, `ambigua`, como hoy.
- Sin cambio para los proyectos de primer nivel.

### 4.3 Nivel global

Hoy no existe canal para las reglas transversales.

- Sección `## Reglas duras` en `core/core-index.md`, leída por el mismo
  parser (`lee_seccion`), con su propio `CAP` global.
- `exo rules --json` devuelve `global` (lista) junto a `rules` (proyecto). Un
  repo sin nota o sin sección sigue recibiendo las globales.
- El mod `prompt.compose` (`plugins/exo/hooks/register.ts`) mete las dos
  listas bajo el framing actual: globales primero, luego las de proyecto.
  `compose-inject.sh` hace lo mismo para subagentes.
- El latido (`hb-<sid>`) y el testigo de Stop cuentan las dos listas.

### 4.4 Presupuesto de reglas en subagentes

`bloque_reglas` corta a `1024 − framing − 80` B (≈ 800 B,
`compose-inject.sh:104`). Con dos listas de hasta 10 no caben. El
presupuesto sube para que quepan las dos listas al `CAP` vigente, y el
`inject-rules-truncated` existente vigila que no se corte. El valor exacto se
fija en el plan, medido con las reglas reales sembradas.

### 4.5 Arranque podado

`exo-recall.sh` deja de inyectar el core-index entero y los punteros de
actividad reciente.

- Inyecta como máximo **1.500 B**: el contrato de memoria reducido (qué es
  la KB, cómo buscar, `/document` al cerrar) y los avisos operativos que ya
  emite (gate pendiente, superpowers, reglas no entregadas).
- **`core/core-index.md` no se edita** por este cambio: `compose-inject.sh`
  lee por awk sus secciones `## Doctrina compacta` y `## Cores` para los
  subagentes (`compose-inject.sh:73` y siguientes). Lo que cambia es qué
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

Candidatas: reglas de M con `proyecto` = clave del repo, `generica=false` y
`ambito` ∈ {`codigo`, `git`, `dominio-proyecto`}. Orden:

1. primero las que A0 **no** cumple sin regla (criba de M; definición fija:
   una regla se considera «A0 ya cumple» cuando todas las réplicas a0 de su
   tarea en `~/.cache/exo-campana-m/corridas-criba/<id>/a0-r*/check.rc` dan 0;
   sin corrida, no se excluye);
2. las que tienen `check`;
3. `git`/`codigo` antes que `dominio-proyecto`;
4. tier `core` antes que `stable` (según la nota de origen);
5. orden de fichero.

Se toman las `CAP` primeras. En los 18 proyectos con ≤10 candidatas entran
todas. En los 7 que desbordan, el dueño revisa el corte en el PR.

La exclusión «A0 ya cumple» depende del modelo con el que se hizo la criba.
Al cambiar de modelo por defecto, una regla excluida no se borra: baja al
final del ranking hasta que se rehaga la criba.

### 5.3 Global: elige el dueño

Las 76 candidatas transversales accionables no caben en 10 plazas, y el
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

## 7. Testing y medición

### 7.1 techo-reglas-3 (dilución), en paralelo con el código

Mismas 10 tareas de techo-reglas-2, canal system prompt con el framing
actual. Brazos: regla gold sola / gold + 9 distractoras reales / gold + 19,
k=2, unas 60 corridas (≈ 5 USD). Distractoras: reglas reales de las 541,
del mismo nivel y sin relación con la tarea. Pre-registro antes de correr.
**Su resultado fija el `CAP` antes de poblar:**

| gold + 9 | `CAP` |
|---|---|
| ≥ 5/10 | 10 |
| 4/10 | 7 |
| ≤ 3/10 | 5 |

### 7.2 Brazo `Ap` en el harness de M

40 sondas × 3, system prompt con las reglas sembradas tal como se despliegan
y el arranque podado. Se reportan **cobertura** (sondas cuya regla está
dentro) y **aplicación dada cobertura**. `Ap` cae si la aplicación dada
cobertura < 0,60. La cobertura de las globales no es interpretable (el dueño
diseñó M y eligió las globales); se reporta aparte y no adjudica.

### 7.3 Condición que invalida A en conjunto

**techo-reglas-3 < 5/10 con gold + 9 y `Ap` < 0,60 a la vez.** Si pasa, las
reglas estáticas no compensan su dilución y el siguiente paso es E.

### 7.4 Tests de código

- `engine`: `ExcedeCap` entrega 10 + `truncated`; nota en subdirectorio
  resuelve; varias candidatas en subdirectorio dan `ambigua`; `global` se
  lee de core-index y llega con o sin nota de proyecto.
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
