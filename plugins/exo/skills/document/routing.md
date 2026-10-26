# Contrato de routing v2

**Cuándo cargar:** al decidir a qué nota y sección va cada pieza extraída
de la sesión (Paso 2 de `SKILL.md`). Destilado de
`~/.claude/commands/documenta.md` (comando propio, no MIT).

Los nombres concretos de notas de la instancia (el destilado canónico de
un proyecto, la nota de backlog, la nota de perfil del dueño, la nota de
doctrina de agentes) no se hardcodean aquí — resuélvelos vía probe contra
la KB de la instancia (`exo targets` o una búsqueda). Esta tabla usa
placeholders entre corchetes.

## Destino por tipo de pieza

| Tipo de pieza | Destino | Modo |
|---|---|---|
| Avance de proyecto con frente activo | `[destilado canónico del proyecto]` | delta: el estado sustituye, no narres; con la nota ≥85 % de su techo (`wc -c`), solo la línea de estado |
| — el mismo avance, para historial | `[bitácora del proyecto]` | append ≤15 líneas bajo cabecera única `## AAAA-MM-DD — <tema concreto>` + hechos + wikilinks al canon; el puntero del canon la cita |
| Item de backlog que se cierra | `[nota de backlog]` | edita `[ ]`→`[x]` en UNA línea, sin duplicar el detalle |
| Item de backlog nuevo | `[nota de backlog]` | añade `[ ]` |
| Síntesis transversal (aplica a ≥2 proyectos) | `[nota(s) de learnings o de doctrina]` | edita o append, según cuánta doctrina ya exista |
| Decisión, patrón u opinión sobre el propio dueño | `[nota de perfil del dueño]` | edita — nunca nota nueva |
| Proyecto o tema nuevo | nota nueva en su directorio de proyectos | crea |
| Research standalone | nota nueva en su directorio de research | crea |
| Regla **mecánica** (verificable, sin juicio) | además, `## Reglas duras`: la de un repo, en su nota canónica; la transversal, en `core/reglas-duras.md` | una línea `- ` de ≤300 B, máximo 10 por sección; si la sección está llena, la rúbrica de abajo decide cuál sale — nunca se anexa al final de una sección llena |
| Decisión que merece nota canónica propia | nota nueva | crea — es la excepción, no la regla |

## Rúbrica de reglas duras (sección llena)

Sustituye a «posición de prioridad»: decide cuál sale. Cada regla lleva coste (alto > medio > bajo) y el orden de fichero desempata.
1. **Veto:** solo entra una regla con una violación observada en la transcripción de la sesión.
2. **Choque con la última del fichero:** P (paso extra) > D (prohíbe el default) > E (el repo la encarna); si hay dato de criba A0, manda.
3. Coste alto > medio > bajo; a igual coste, orden de fichero. Una P de coste bajo va por encima de una D de coste alto.
4. La desplazada se queda en la prosa; anota en la bitácora «salió `<regla>` por `<criterio>`».
5. Si el choque no se puede responder (regla ambigua), la nueva entra por coste y orden de fichero, y la bitácora lo dice.

## Frontmatter

Toda nota escrita o editada lleva `tags` (temas/proyecto) y `tier`:
`core` = doctrina estable y muy consultada; `stable` = canon de un
proyecto vivo; `log` = bitácora append-only. El tier de una nota nueva se
decide por su función en la tabla de arriba, no por adivinanza.

## Wikilinks

`[[...]]` es contrato load-bearing: el title de la nota es su id de
enlace. Reusa el title EXACTO de la nota existente al enlazar — un title
ligeramente distinto rompe el grafo en vez de apuntar a la nota real.

## Caveat de concurrencia

Editar la MISMA nota canónica desde dos sesiones a la vez puede pisar una
edición (riesgo a nivel de fichero, no del motor de búsqueda/index). Los
ficheros de bitácora (uno por proyecto, append-only) son más seguros
frente a esto — de ahí el sesgo del Paso 3 hacia `append` sobre
`find_replace`/`replace_section`.
