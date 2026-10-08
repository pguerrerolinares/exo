---
permalink: "{{KB_NAME}}/AGENTS"
title: Contrato de la KB para agentes
tags: [contrato, agentes]
tier: stable
semilla: true
---

# Contrato de la KB para agentes

Esta base de conocimiento (KB) está organizada para que un agente pueda leer
y escribir en ella de forma predecible. Este documento define qué significa
cada carpeta y, sobre todo, **a dónde va un avance nuevo** — la parte que
más fácil se hace mal si se improvisa.

## Estructura de carpetas

- **`core/`** — doctrina e identidad estables: los principios de más alto
  nivel de esta KB, los que casi nunca cambian y que un agente debería leer
  primero para orientarse. Es el punto de entrada, no el sitio donde se
  vuelca trabajo del día a día.
- **`projects/`** — el destilado canónico de cada frente de trabajo: una
  nota por proyecto con su estado actual, condensado. Es la foto, no el
  vídeo.
- **`learnings/`** — principios destilados, reutilizables más allá de un
  proyecto concreto: reglas de decisión que siguen siendo ciertas aunque el
  contexto que las originó cambie o desaparezca.
- **`log/`** — bitácoras en formato append, una por proyecto (u otro ámbito
  que lo justifique). Es el vídeo: registro cronológico de qué se hizo y
  cuándo, sin reescribir entradas pasadas.
- **`archive/`** — material retirado de circulación activa pero conservado
  por si hace falta consultarlo. Un agente no debería necesitar leer aquí en
  el flujo normal de trabajo.

## `tier`: qué significa y qué valores tiene

Todas las notas llevan un campo `tier` en el frontmatter que indica su
volatilidad y cómo debe tratarlas un proceso de mantenimiento de la KB:

- **`core`** — el subconjunto mínimo que el arranque de un agente sirve
  siempre, entero (`exo recall` sin `--query`). Debería haber pocas: cada
  `core` nuevo compite por el mismo presupuesto de arranque, así que subir
  una nota de `stable` a `core` es la excepción, no el hábito.
- **`stable`** — contenido destilado, de baja frecuencia de cambio
  (`core/`, `projects/`, `learnings/`). Cuando cambia, cambia por delta
  editado sobre la nota existente, no por acumulación.
- **`log`** — contenido de bitácora, append-only (`log/`). Se espera que
  crezca por adición constante de entradas nuevas, nunca por reescritura de
  las antiguas.

Un proceso de consolidación de la KB puede tratar de forma distinta las
notas `stable` (las revisa, las funde, las poda) y las `log` (las deja
crecer y, cuando corresponde, las resume hacia una nota `stable` sin tocar
el original, o lo mueve a `archive/`). Una nota `stable` también puede
retirarse a `archive/` cuando deja de estar activa (un proyecto cerrado, un
learning ya fundido en otro); el traslado cambia la carpeta, no el `tier`
del frontmatter — sigue llevando `stable`, ahora leído como "estable pero
retirada" en vez de "estable y vigente". Esa nota vive directamente bajo
`archive/` (p. ej. `archive/proyecto-cerrado.md`), no bajo `archive/log/`:
esa subcarpeta es solo para bitácoras `log` retiradas, no para notas
`stable`.

## La regla de oro: dónde va un avance

Cuando un agente tiene algo nuevo que anotar en la KB, debe decidir entre
tres destinos. El orden de preferencia es siempre el mismo:

1. **Canon como delta.** Si el avance pertenece a un proyecto con frente
   activo (ya existe una nota en `projects/` para él), el avance se edita
   **dentro** de esa nota: se actualiza el campo que corresponda (estado
   actual, decisiones vivas, etc.). No se añade al final como si fuera un
   diario.
2. **Bitácora como append.** El mismo avance, además, recibe un apunte corto
   con fecha en la bitácora del proyecto (`log/`), añadido al final sin
   tocar lo anterior. El destilado dice "cómo está"; la bitácora dice "qué
   pasó y cuándo".
3. **Nota nueva, casi nunca.** Crear una nota nueva en `projects/` (o un
   `learning` nuevo en `learnings/`) solo se justifica cuando el avance
   corresponde a un proyecto o un principio **genuinamente nuevo**, sin nota
   previa que actualizar. Si dudas entre "actualizar lo que ya existe" o
   "crear algo nuevo", la respuesta casi siempre es actualizar.

### Ejemplo genérico

Un agente termina de implementar una parte de un proyecto que ya tenía nota
en `projects/proyecto-x.md`. El flujo correcto es:

- Editar `projects/proyecto-x.md` para reflejar el nuevo estado (delta sobre
  el destilado, no un párrafo añadido al final).
- Añadir una entrada nueva al final de `log/proyecto-x.md` con la fecha y un
  resumen corto de lo que se hizo (append, sin tocar entradas previas).
- Solo si, además, de este trabajo surge un principio reutilizable en otros
  contextos, añadir (o actualizar) una nota en `learnings/`.
- No crear una nota nueva en `projects/` para esto: el proyecto ya existe.

## Operativa: leer y escribir en una sesión

- **Busca antes de actuar.** Antes del primer Agent, Edit o Write de una
  sesión, busca contexto (`exo search`, `exo targets <tema>`) y carga las
  notas `core`/`stable` relevantes. Las notas `log` se leen por enlace,
  cuando hace falta el detalle: leer una entera cuesta.
- **La búsqueda trae el fragmento, no el desenlace.** Sigue los enlaces de
  la nota encontrada antes de concluir: la decisión final suele estar en
  otra nota o más abajo.
- **Cierra con `exo:document`.** Al terminar una sesión con decisiones o
  aprendizajes: delta sobre el canon + apunte breve en la bitácora. No crees
  una nota por sesión.
- **Edita la sección que ya existe; no anexes un bloque
  `delta AAAA-MM-DD` al final.** Así es como se fragmentan las notas. Una
  nota `core` o `stable` nunca recibe apuntes fechados: eso es bitácora.

## Presupuestos y trinquete

- **Cada nota tiene un techo de bytes.** Una nota nueva hereda el nominal de
  su tier (`exo budget`: `core` 8.500 B, `stable` 12.500 B; `log` sin
  techo). Una vez sellado (`exo ratchet --seal`, en `.kbx-ratchet.json`), el
  techo es de esa nota.
- **El trinquete solo baja.** Subir un techo, borrar el sello o reclasificar
  una nota a `log` para escapar de él rompe `exo ratchet`. Sellar o bajar un
  techo exige un 15% de aire: a ras es un mordisco programado para mañana.
- **Al morder el techo**: parte la nota (canon + bitácora), rota la bitácora
  (`exo rotate`) o consolida (`exo:distill`). Nunca subas el techo, nunca
  mutiles la nota y **nunca recortes el delta que ibas a escribir** para que
  quepa. Si nada cabe, deja el commit pendiente y dilo.
- **Al destilar, mueve bloques enteros; no re-resumas prosa.** Cada
  re-resumen pierde matices que nadie decidió perder.

## `semilla: true`: qué es y cuándo quitarlo

Las notas que venían con esta KB al crearla llevan `semilla: true` en el
frontmatter. Es una marca de origen, no un tipo de nota: dice "esto lo puso la
plantilla, no tú".

Sirve para dos cosas. Una, orientarte: si estás leyendo algo con esa marca,
todavía es texto de fábrica y probablemente hable en genérico. Y dos, poder
barrerlo — `grep -rl 'semilla: true' .` te lista de una vez todo lo que aún no
has hecho tuyo.

Cuando reescribas una de esas notas con contenido propio, **quita la línea**.
Cuando ya no quede ninguna, la KB ha dejado de ser una plantilla.

## Los índices no se destilan

Cuando una nota índice (por ejemplo, un listado de proyectos activos o de
temas) crece, el mantenimiento correcto es **retirar entradas muertas**
(proyectos cerrados, temas ya no vigentes), no comprimir o resumir las
entradas vivas. Un índice existe para que se pueda hojear rápido; resumir lo
que sigue vivo lo hace menos útil, no más.
