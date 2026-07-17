# Contrato de routing v2

**Cuándo cargar:** al decidir a qué nota y sección va cada pieza extraída
de la sesión (Paso 2 de `SKILL.md`). Destilado de
`~/.claude/commands/documenta.md` (comando propio, no MIT).

Los nombres concretos de notas de la instancia (el destilado canónico de
un proyecto, la nota de backlog, la nota de perfil del dueño, la nota de
doctrina de agentes) no se hardcodean aquí — resuélvelos vía probe contra
la KB de la instancia (`kbx targets` o una búsqueda). Esta tabla usa
placeholders entre corchetes.

## Destino por tipo de pieza

| Tipo de pieza | Destino | Modo |
|---|---|---|
| Avance de proyecto con frente activo | `[destilado canónico del proyecto]` | delta: edita el estado, no narres |
| — el mismo avance, para historial | `[bitácora del proyecto]` | append ≤15 líneas: fecha + hechos + wikilinks al canon |
| Item de backlog que se cierra | `[nota de backlog]` | edita `[ ]`→`[x]` en UNA línea, sin duplicar el detalle |
| Item de backlog nuevo | `[nota de backlog]` | añade `[ ]` |
| Síntesis transversal (aplica a ≥2 proyectos) | `[nota(s) de learnings o de doctrina]` | edita o append, según cuánta doctrina ya exista |
| Decisión, patrón u opinión sobre el propio dueño | `[nota de perfil del dueño]` | edita — nunca nota nueva |
| Proyecto o tema nuevo | nota nueva en su directorio de proyectos | crea |
| Research standalone | nota nueva en su directorio de research | crea |
| Decisión que merece nota canónica propia | nota nueva | crea — es la excepción, no la regla |

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
