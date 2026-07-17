---
name: documenta
description: Extrae decisiones, opiniones, aprendizajes y patrones de la sesión actual y los guarda en la KB siguiendo el contrato de routing: canon como delta, bitácora como append, nota nueva casi nunca. Commit scoped al cerrar.
---

# documenta

Cierra la sesión escribiendo a la KB (vía engine — hoy kbx/basic-memory/
filesystem, hasta que exista un engine dedicado).

## Paso 1 · Extrae

De la conversación actual, saca lo que merece persistir: decisiones y su
porqué, opiniones/posturas (incluidas las matizadas con trade-offs),
aprendizajes técnicos durables, patrones recurrentes. Descarta lo efímero
(estado de comandos, lo que ya vive en repo/git).

## Paso 2 · Orienta barato, luego enruta

Probe del engine antes de rutear (p.ej. `kbx targets <topic> --json`):
devuelve permalink/tier/headings/snippet por candidata; elige "nota X,
sección Y" de esa lista y lee SOLO la ganadora antes de escribir.

Degradación con aviso visible: si el engine no está o falla (exit ≠ 0),
cae a búsqueda por texto en la KB y añade al resumen final una línea
visible (`<engine> unavailable → fallback a búsqueda`). Nunca bloquees el
cierre de sesión por esto.

Regla de oro del routing (contrato completo con la tabla destino en
`routing.md`): **canon como delta, bitácora como append, nota nueva casi
nunca.**

- Avance de proyecto con frente activo ⇒ (a) delta al destilado canónico
  del proyecto + (b) append ≤15 líneas a su bitácora (fecha + hechos +
  wikilinks al canon, sin re-narrar).
- Estado del frente en el backlog: cerrados `[ ]`→`[x]` en UNA línea sin
  duplicar el detalle (vive en la bitácora); solo estado abierto + cola
  corta de recién-cerrado — el barrido de `[x]` viejos no es de esta
  skill.
- Síntesis transversal (aplica a varios proyectos) ⇒ nota de learnings o
  de doctrina. Decisión o patrón sobre el propio dueño ⇒ su nota de perfil
  (nunca nota nueva).
- Nota nueva SOLO para: proyecto/tema nuevo, research standalone, o una
  decisión que merece nota canónica propia.
- No crees "una nota por sesión" cuando el proyecto ya tiene bitácora — el
  directorio de sesiones sueltas queda solo para trabajo sin proyecto.

## Paso 3 · Reglas de escritura

Frontmatter obligatorio en todo lo escrito: `tags` + `tier`.
Search-before-write: antes de crear cualquier nota, busca si ya existe la
canónica. Edita, no dupliques; prefiere `append` sobre find_replace/
replace_section — append tolera mejor la edición concurrente desde otra
sesión. Títulos consistentes: el title es el id de los wikilinks
`[[...]]` — reusa el title exacto de la nota existente al enlazar.

## Paso 4 · Commit scoped al cerrar

Commitea SOLO los ficheros que esta invocación escribió o editó — nunca
`git add -A`. `git -C <repo>` (nunca `cd`). NUNCA push. Mensaje:
`docs(kb): documenta <resumen corto>`. Si `.git/index.lock` bloquea:
espera ~2s y reintenta una vez; si sigue bloqueado, reporta que quedó sin
commitear para hacerlo a mano — no fuerces borrando el lock.

## Resumen final

Qué notas se crearon o editaron, dónde quedaron, y el hash del commit.
