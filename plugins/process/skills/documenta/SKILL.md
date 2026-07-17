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
permalink/tier/headings/snippet por candidata; elige "nota X, sección Y"
y lee SOLO la ganadora antes de escribir. Degradación con aviso visible:
si el engine falla, cae a búsqueda por texto y añade una línea visible al
resumen final (`<engine> unavailable → fallback`) — nunca bloquees el
cierre por esto.

Regla de oro (tabla destino completa en `routing.md`): **canon como
delta, bitácora como append, nota nueva casi nunca.** Avance de proyecto
con frente activo ⇒ delta al destilado canónico + append ≤15 líneas a su
bitácora (fecha + hechos + wikilinks, sin re-narrar). Backlog: cerrados
`[ ]`→`[x]` en UNA línea sin duplicar el detalle — solo estado abierto +
cola corta de recién-cerrado; el barrido de `[x]` viejos es de la
consolidación (/consolida), no de documenta. Síntesis transversal ⇒ nota de learnings o
doctrina; decisión o patrón sobre el propio dueño ⇒ su nota de perfil
(nunca nota nueva). Nota nueva SOLO para proyecto/tema nuevo, research
standalone, o decisión que merece nota canónica propia. No crees "una
nota por sesión" cuando el proyecto ya tiene bitácora.

## Paso 3 · Reglas de escritura

Frontmatter obligatorio en todo lo escrito: `tags` + `tier`.
Search-before-write antes de crear cualquier nota. Edita, no dupliques;
prefiere `append` sobre find_replace/replace_section — tolera mejor la
edición concurrente desde otra sesión. Títulos consistentes: el title es
el id de los wikilinks `[[...]]` — reusa el exacto al enlazar.

## Paso 4 · Commit scoped y resumen

Commitea SOLO los ficheros que esta invocación escribió o editó — nunca
`git add -A`. `git -C <repo>` (nunca `cd`). NUNCA push. Mensaje: `docs(kb):
documenta <resumen corto>`. Ante `.git/index.lock`: espera ~2s, reintenta
una vez; si sigue bloqueado, reporta para commit manual — no fuerces
borrando el lock. Resumen final: qué notas se crearon o editaron, dónde
quedaron, y el hash del commit.
