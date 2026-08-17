---
name: documenta
description: Extrae decisiones, opiniones, aprendizajes y patrones de la sesión actual y los guarda en la KB siguiendo el contrato de routing: canon como delta, bitácora como append, nota nueva casi nunca. Commit scoped al cerrar.
---

# documenta

Cierra la sesión escribiendo a la KB vía `exo write` (M4). Degradación con
aviso visible si el engine no está: `kbx`/filesystem, nunca bloquear el cierre.

## Paso 1 · Extrae

De la conversación actual, saca lo que merece persistir: decisiones y su
porqué, opiniones/posturas (incluidas las matizadas con trade-offs),
aprendizajes técnicos durables, patrones recurrentes. Descarta lo efímero
(estado de comandos, lo que ya vive en repo/git).

## Paso 2 · Orienta barato, luego enruta

Probe del engine antes de rutear: `exo search --db <db> --type hybrid --json
"<topic>"` devuelve por candidata `permalink`, `score` y **`ruta`**. Elige
"nota X, sección Y" y lee SOLO la ganadora antes de escribir. La `ruta` es
imprescindible: el permalink NO es invertible (el slug come acentos, espacios
y em-dashes), así que sin ella no puedes localizar el fichero. `kbx targets`
sigue sirviendo para ver headings sin body mientras exista. Degradación con
aviso visible: si el engine falla, cae a búsqueda por texto y añade una línea
al resumen final (`<engine> unavailable → fallback`) — nunca bloquees el
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

## Paso 3 · Cómo se escribe cada cosa

Tres caminos según el destino. El criterio es quién tiene ya el fichero en
contexto:

| Destino | Herramienta | Por qué |
|---|---|---|
| **Bitácora** (`tier: log`) | `exo write append --from <fichero> <permalink>` | Escribe sin leer: las bitácoras pesan decenas de KB y cargarlas enteras en cada cierre es el coste que este camino evita |
| **Canon** (delta a nota core/stable) | `Edit` sobre la `ruta` que dio `search` | Ya has leído la ganadora para escribir el delta; `Edit` opera sobre texto exacto y no parsea headings |
| **Nota nueva** | `exo write new --dir <d> --titulo <t> --from <f>` | Genera permalink, slug, ruta y frontmatter completo; el permalink jamás se improvisa |

El cuerpo va **en un fichero** (`--from`), que escribes antes con `Write`.
Nunca por heredoc: el escaping de comillas, backticks y `$` es la fuente de
error más tonta y frecuente de este camino.

**Los dos rechazos, y qué significan.** Salen con exit 3 (distinto de un error
real, que es 1):

- *append a nota que no es `tier: log`* → estás a punto de anexar al canon.
  Eso es el anti-patrón que más caro ha salido en esta KB. Edita la sección
  que ya existe con `Edit`, o manda la entrada a la bitácora del frente. Solo
  `--force` si de verdad es una excepción consciente: queda registrada.
- *nota duplicada* → ya existe una canónica con slug muy parecido. Edítala en
  vez de crear otra; `--force` si de verdad es un tema nuevo.

Frontmatter obligatorio en lo que escribas a mano: `tags` + `tier`. `exo write
new` lo auto-completa y **nunca rechaza** por frontmatter — un cierre de sesión
no puede fallar por metadatos. Títulos consistentes: el title es el id de los
wikilinks `[[...]]` — reusa el exacto al enlazar.

## Paso 4 · Commit scoped y resumen

Commitea SOLO los ficheros que esta invocación escribió o editó — nunca
`git add -A`. Las rutas salen del campo `ruta_abs` de cada envelope de
`exo write` y de los `Edit` que hiciste; exo **no commitea**, a propósito.
`git -C <repo>` (nunca `cd`). NUNCA push. Mensaje: `docs(kb): documenta
<resumen corto>`. Ante `.git/index.lock`: espera ~2s, reintenta una vez; si
sigue bloqueado, reporta para commit manual — no fuerces borrando el lock.

Si el pre-commit de la KB rechaza por presupuesto, **el rechazo no se
negocia**: no subas `kbx_budget_max`, no recortes la nota, no uses
`--no-verify`. Parte la nota (lo fechado a la bitácora), o rota su cola fría.
Un commit sin hacer se arregla en un minuto; una nota mutilada, no.

Resumen final: qué notas se crearon o editaron, dónde quedaron, y el hash del
commit.
