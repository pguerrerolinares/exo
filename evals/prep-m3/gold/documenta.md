# Gold — process:documenta (paridad de movimientos)

Fuente: `~/.claude/commands/documenta.md` (comando, 69 líneas; framework §5.2: "/documenta entra a process").
Uso: ver `evals/prep-m3/README.md`.

## Movimientos

- [ ] Extraer de la sesión lo que merece persistir: decisiones y su porqué, opiniones/posturas (incluidas las matizadas con trade-offs), aprendizajes técnicos durables, patrones recurrentes; descartar lo efímero (estado de comandos, lo que ya vive en repo/git) — documenta.md líneas 9-17
- [ ] Orientación barata antes de rutear: `kbx targets <topic> --json` (permalink, tier, headings sin body, snippet por candidata); elegir "nota X, sección Y" de esa lista y leer SOLO la ganadora — líneas 20-34
- [ ] Degradación con aviso visible: si kbx no está o falla (exit ≠ 0), fallback a búsqueda vía KB/MCP + línea visible en el resumen final (`kbx unavailable → search_notes fallback`); NUNCA bloquear el cierre de sesión por esto — líneas 35-38. [Este movimiento es el patrón único de degradación de framework §5.1: "probe del CLI → si no hay engine/KB, defaults + aviso visible (patrón ya probado en /documenta)"]
- [ ] Regla de oro del routing: canon como delta, bitácora como append, nota nueva casi nunca — línea 40
- [ ] Avance de proyecto con frente activo ⇒ (a) delta al destilado canónico del proyecto + (b) append ≤15 líneas a su bitácora (fecha + hechos + wikilinks al canon, sin re-narrar) — línea 42
- [ ] Estado del frente en el backlog: cerrados `[ ]`→`[x]` en UNA línea sin duplicar el detalle (vive en la bitácora); solo estado abierto + cola corta de recién-cerrado; el barrido de `[x]` viejos es de /consolida, no de documenta — línea 43
- [ ] Síntesis transversal ⇒ learnings/doctrina; decisión/patrón sobre el propio dueño ⇒ su nota de perfil (nunca nota nueva) — líneas 44-45
- [ ] Nota nueva SOLO para: proyecto/tema nuevo, research standalone, o decisión que merece nota canónica propia — línea 46
- [ ] Frontmatter obligatorio en todo lo escrito: `tags` + `tier` — línea 47
- [ ] search-before-write: antes de crear cualquier nota, buscar si ya existe la canónica — línea 52
- [ ] Editar, no duplicar; preferir `append` sobre find_replace/replace_section (append tolera edición concurrente; find_replace es read-modify-write) — línea 53
- [ ] Títulos consistentes: el title es el id de los wikilinks; reusar el title exacto al enlazar — línea 54
- [ ] Commit scoped al cerrar: SOLO los ficheros que esta invocación escribió (nunca `git add -A`), `git -C` (nunca `cd`), NUNCA push — líneas 58-63
- [ ] Retry ante `.git/index.lock` (~2s, una vez); si sigue bloqueado, reportar para commit manual; no forzar borrando el lock — línea 65
- [ ] Resumen final: qué notas se crearon/editaron, dónde quedaron, hash del commit — línea 69

## DESCARTES (corpus negativo)

- Formato de observations `- [categoria] contenido` y relations `- tipo_relacion [[Titulo]]` (documenta.md líneas 55-56): la gramática pasa a estilo opcional — framework §6.3: "Observations pasan a bullets normales (texto indexable, sin fila propia). /documenta deja de generarlas como estructura". OJO: los wikilinks `[[...]]` SÍ se conservan (§6.3: "Wikilinks = contrato load-bearing") — solo muere la gramática de fila. Presencia del formato de observations como estructura obligatoria = fallo.
- Hardcode de tools MCP concretas (`mcp__basic-memory__*`, línea 7) y de nombres de nota de la instancia como literales del framework: la skill genérica habla de "la KB vía engine" (kbx/basic-memory/filesystem hasta E1 — framework §5.1) y resuelve los nombres de instancia vía probe — framework §3: "Framework sin nada personal". El comportamiento (buscar/escribir/editar la KB) se conserva; el binding concreto es overlay de instancia.
- Trailer `Co-Authored-By: Claude <noreply@anthropic.com>` (línea 64): estilo de mensaje de la instancia actual, no movimiento del framework; el mensaje scoped `docs(kb): documenta <resumen>` sí se conserva como patrón.
