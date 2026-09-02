# Contexto compartido — auditoría del sistema de presupuestos de la KB

Este fichero lo leen los cuatro consultores de esta auditoría. Cada uno tiene
además su propio brief con su ángulo.

## La queja de Paul, literal

> "estoy un poco hasta la polla de los temas de presupuestos... llevamos con ese
> problema casi desde el inicio y se creó consolida, pero me da la sensación que
> es un apaño más que una solución"

Esa es la pregunta a resolver: **¿el sistema de presupuestos de la KB es una
solución o un apaño que se ha institucionalizado?** Y si es lo segundo, ¿cuál es
la solución?

Contexto de por qué se pregunta ahora: hace unas horas Paul hizo la misma
objeción sobre un cap distinto (el del bloque que inyecta un hook), se auditó, y
resultó que el problema no era el número sino el formato — el contenido repetía
lo mismo tres veces. La sospecha es que aquí puede pasar algo parecido, pero
nadie lo ha verificado.

## Qué es el sistema hoy

La KB `kb-demo` (168 notas markdown, 4,3 MB) tiene un modelo de tres tiers
declarado en el frontmatter de cada nota:

- `tier: core` (7 notas) — identidad y estado; presupuesto **8.500 B/nota**.
- `tier: stable` (43 notas) — destilados canónicos; presupuesto **12.500 B/nota**.
- `tier: log` (93 notas) — bitácoras append-only; **sin límite**.

Lo hace cumplir `kbx budget` (Go, repo `~/Documentos/proyectos/kbx`), invocado
desde un pre-commit de la KB. Y hay un **trinquete** (`.kbx-ratchet.json`): once
notas tienen un techo propio sellado por encima del presupuesto nominal, y ese
techo **solo puede bajar**. La doctrina asociada, escrita en `core/core-index.md`:

> "Si una nota no cabe: **pártela** (canon + bitácora), no subas el techo ni la
> mutiles."

Y una skill, `/consolida`, que hace sleep-time compute manual: colapsa bitácoras
en destilados, chequea presupuestos, promueve doctrina repetida a core y refresca
el `core-index`.

## Datos duros del recon (verificadlos, están hechos deprisa)

- **11 de las ~50 notas con techo viven de waiver** (22%): el presupuesto nominal
  es ficción para una de cada cinco. Techos sellados actuales en
  `~/Documentos/proyectos/kb-demo/.kbx-ratchet.json`, de 10.000 a 30.000 B.
- **26 commits** de la KB mencionan presupuesto/budget/rotar/techo/consolida.
  Entre ellos: `rotate: primera rotación de bitácoras (F0) — 534247 B de cola
  fría a archive/log/`, `consolida: split canon/bitácora de 9 notas obesas`,
  `ratchet: ancla de amnistía F1.a — los 9 techos de waiver quedan sellados`,
  `consolida: parte doctrina-agentes y baja su techo de 27000 a 20000`.
- Ritmo de la KB: 68 commits en jun-2026, 170 en jul, 31 en lo que va de agosto.
- **Hoy mismo, en un solo cierre de sesión** (`/documenta` de M6-06), el
  presupuesto mordió **dos veces**: hubo que rotar histórico de una nota canónica
  a su bitácora para que cupiera un delta, y hubo que recortar un aprendizaje
  transversal porque `doctrina-agentes` está a 33 B de su techo y
  `desarrollo-agentico` a 49. Ese aprendizaje acabó en una bitácora "marcado como
  candidato a promoción", que es un eufemismo de "no cabía en su sitio".

## CORRECCIÓN (2026-08-22, tras el informe de arqueología) — LEED ESTO PRIMERO

**La sección de abajo contenía una premisa FALSA, escrita por el orquestador.**
Decía que el presupuesto nació para proteger el coste del arranque. No es cierto,
y está desmentido en la propia spec fundacional
(`kb-demo/docs/superpowers/specs/2026-07-03-memoria-v2-design.md:17`), que ya
midió el arranque y lo descartó como problema, textualmente: *"El arranque es
barato […] ≈1.4k tok. **No es el problema**"*.

**Lo que el presupuesto vino a atacar de verdad era el coste de PULL**: cores
hipertrofiados por acumulación sin destilar (el caso que la spec cita:
`desarrollo-agentico` en 20,6k tokens). Es decir, nació como **forzador de
destilado editorial**, no como optimización de transporte.

Esto cambia el peso de los argumentos en las dos direcciones, y quiero que lo
tengáis en cuenta sin que os arrastre:

- **Debilita** el argumento "ahora hay retrieval, luego el tamaño da igual": si el
  objetivo nunca fue abaratar el transporte, tener retrieval no lo resuelve.
- **Pero abre** la pregunta de verdad: si el objetivo es calidad editorial,
  ¿es un límite de bytes el instrumento correcto para conseguirla? Un límite de
  tamaño mide síntoma, no calidad.

Juzgadlo vosotros. Lo que sigue se conserva tal cual para que quede el registro
del error, pero **leedlo sabiendo que su premisa está desmentida**.

## El dato que puede cambiarlo todo (verificadlo) — PREMISA DESMENTIDA, ver arriba

El presupuesto nació cuando **la única vía de acceso a la memoria era inyectarla
en el arranque de la sesión**: el hook de `SessionStart` mete el `core-index`
entero en cada sesión, con un cap duro de 6.144 caracteres. En ese mundo, el
tamaño de las notas core es coste fijo por sesión y limitarlo tiene sentido
evidente.

Pero **hoy hay dos vías más** que no existían entonces:
1. `exo search`/`exo recall` — retrieval híbrido (FTS5 + embeddings) sobre la KB
   entera, disponible como CLI y usado por el agente a demanda.
2. **Desde hoy mismo**, un hook `UserPromptSubmit` que inyecta punteros a notas
   relevantes en cada prompt sustantivo, sin que nadie lo pida (M6-06, mergeado
   en `c55644a`).

Pregunta que nadie se ha hecho: **si el contenido llega por retrieval y no por
inyección de arranque, ¿qué problema resuelve hoy limitar el tamaño de una nota?**
Puede que siga habiendo razones buenas (coste de leer una nota entera cuando el
agente la abre; señal/ruido dentro de la nota; que una nota gorda sea síntoma de
que mezcla temas). Pero hay que decirlas explícitamente, no darlas por supuestas.

## Restricciones que cualquier propuesta debe respetar

1. **Es una KB personal de un solo autor.** No hay equipo, no hay CI, no hay
   usuarios. Régimen §0 del proyecto: cerrar cosas, no construir maquinaria.
2. **Lo que existe funciona**: el trinquete cumple su función (los techos solo
   han bajado), y `/consolida` ha hecho trabajo real. No se tira nada sin
   demostrar que sobra.
3. **El coste del mantenimiento es de Paul**, y es tiempo suyo. Una solución que
   pida más disciplina humana no es una solución.
4. **Nada de métricas nuevas ni ventanas de medición** (régimen §0).
5. YAGNI: si la propuesta es más cara que el problema, no es una propuesta.

## Material

- KB: `~/Documentos/proyectos/kb-demo` (repo git con historia completa).
- `kbx`: `~/Documentos/proyectos/kbx` (Go) — `internal/budget`, `internal/stale`,
  `internal/rotate`, `internal/frontmatter`.
- exo: `~/Documentos/proyectos/exo` — engine Rust, plugins, specs y consultas.
- Skill `/consolida`: `~/.claude/plugins/cache/exo/reflex/0.15.0/skills/consolida/`
  (o `plugins/reflex/skills/consolida/` en el repo exo).
- Doctrina viva: `kb-demo/core/core-index.md`, `core/doctrina-agentes.md`.
- Índice del engine: `~/.exo/index.db`. **Cópialo si vas a escribir**; no lo
  toques en sitio. `sqlite3` CLI no está instalado: usa `python3` con el módulo
  `sqlite3`.

## Reglas para todos

- **Verificación primaria propia.** El recon de arriba está hecho deprisa; si un
  dato cae, decidlo y decid qué se lleva por delante.
- **No toquéis nada**: ni código de producción, ni la KB, ni commits. Scripts de
  medición en vuestro scratchpad, sí.
- **No os coordinéis entre vosotros.** Los cuatro ángulos son deliberadamente
  independientes; la síntesis la hace el orquestador. Si vuestra conclusión
  contradice la de otro, mejor.
- **Si la respuesta es "el sistema está bien", decidlo.** Un informe que fabrica
  problemas para justificar su existencia cuesta más que uno que firma.
