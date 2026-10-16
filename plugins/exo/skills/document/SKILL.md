---
name: document
description: Extrae decisiones, opiniones, aprendizajes y patrones de la sesión actual y los guarda en la KB siguiendo el contrato de routing: canon como delta, bitácora como append, nota nueva casi nunca. Commit scoped al cerrar.
---

# document

Cierra la sesión escribiendo a la KB vía `exo write` (M4). Degradación con
aviso visible si el engine no está: filesystem, nunca bloquear el cierre.

## Paso 1 · Extrae

De la conversación actual, saca lo que merece persistir: decisiones y su
porqué, opiniones/posturas (incluidas las matizadas con trade-offs),
aprendizajes técnicos durables, patrones recurrentes. Descarta lo efímero
(estado de comandos, lo que ya vive en repo/git).

## Paso 2 · Orienta barato, luego enruta

Probe del engine antes de rutear: `exo search --type hybrid --limit 5 "<topic>"`
da por candidata `permalink`, `type`, `score` y **la ruta absoluta**, en cuatro
columnas separadas por tab y sin jq de por medio. Elige "nota X, sección Y" y lee
SOLO la ganadora antes de escribir. La ruta es imprescindible: el permalink NO es
invertible (el slug come acentos, espacios y em-dashes), así que sin ella no
puedes localizar el fichero. Alternativa con score y el fragmento que casa con
la query, misma búsqueda híbrida: `exo recall --query "<topic>" --limit 5`.
Si necesitas el envelope para un script: `exo search --type hybrid --json
"<topic>" | jq -r '.data.results[] | "\(.score)  \(.permalink)  \(.path)"'` — los
resultados cuelgan de `.data.results[]` (`.data` es un objeto, no un array) y el
campo es `path`, nunca `ruta`. `exo targets` sigue sirviendo para ver headings sin
body mientras exista. Degradación con
aviso visible: si el engine falla, cae a búsqueda por texto y añade una línea
al resumen final (`<engine> unavailable → fallback`) — nunca bloquees el
cierre por esto.

Regla de oro (tabla destino completa en `routing.md`): **canon como
delta, bitácora como append, nota nueva casi nunca.** Avance de proyecto
con frente activo ⇒ delta al destilado canónico + append ≤15 líneas a su
bitácora (cabecera única + hechos + wikilinks, sin re-narrar). Backlog: cerrados
`[ ]`→`[x]` en UNA línea sin duplicar el detalle — solo estado abierto +
cola corta de recién-cerrado; el barrido de `[x]` viejos es de la
consolidación (/distill), no de document. Síntesis transversal ⇒ nota de learnings o
doctrina; decisión o patrón sobre el propio dueño ⇒ su nota de perfil
(nunca nota nueva). Nota nueva SOLO para proyecto/tema nuevo, research
standalone, o decisión que merece nota canónica propia. No crees "una
nota por sesión" cuando el proyecto ya tiene bitácora.

**Gate previo (antes de editar un canon).** Mide la ruta ganadora con
`wc -c <ruta>`; no uses `exo budget`, que no lista las notas por debajo del 85 %.
Techo = `kbx_budget_max` del frontmatter si existe; si no, el nominal del tier
(core 8.500 B, stable 12.500 B). Si el tamaño es ≥ 85 % del techo, **al canon
solo va la línea de estado, sustituyendo a la anterior; todo el detalle va a la
bitácora.** El número decide, no tú. Nunca evicción en caliente: borrar del
canon para hacer sitio es trabajo de `/distill`, offline y con juicio. Por
debajo del 85 % el canon se edita como delta, pero el estado también
**sustituye, no suma**: se reescribe la sección de estado, sin bullets fechados
por sesión.

## Paso 3 · Cómo se escribe cada cosa

Tres caminos según el destino. El criterio es quién tiene ya el fichero en
contexto:

| Destino | Herramienta | Por qué |
|---|---|---|
| **Bitácora** (`tier: log`) | `exo write append --from <fichero> <permalink>` | Escribe sin leer: las bitácoras pesan decenas de KB y cargarlas enteras en cada cierre es el coste que este camino evita |
| **Canon** (delta a nota core/stable) | `Edit` sobre la `ruta` que dio `search` | Ya has leído la ganadora para escribir el delta; `Edit` opera sobre texto exacto y no parsea headings |
| **Nota nueva** | `exo write new --dir <d> --title <t> --tier <core\|stable\|log> --from <f>` | Genera permalink, slug, ruta, `title` y `type`; el permalink jamás se improvisa |

**Cabecera única en la bitácora.** Toda entrada empieza por
`## AAAA-MM-DD — <tema concreto>` (separador ` — `, em dash con espacios). El
puntero del canon cita esa cabecera: `[[<slug>-bitacora]], AAAA-MM-DD «<tema
concreto>»`, y por eso el tema debe ser grepeable y único. Prohibidas las
etiquetas genéricas («movido del canon», «delta», «detalle», «distill») sin tema
detrás. `exo write append` rechaza (exit 3) una cabecera ya presente en la nota
viva; concreta el tema, no uses `--force` salvo excepción consciente.

El cuerpo va **en un fichero** (`--from`), que escribes antes con `Write`.
Nunca por heredoc: el escaping de comillas, backticks y `$` es la fuente de
error más tonta y frecuente de este camino.

**Los tres rechazos, y qué significan.** Salen con exit 3 (distinto de un error
real, que es 1):

- *append a nota que no es `tier: log`* → estás a punto de anexar al canon.
  Eso es el anti-patrón que más caro ha salido en esta KB. Edita la sección
  que ya existe con `Edit`, o manda la entrada a la bitácora del frente. Solo
  `--force` si de verdad es una excepción consciente: queda registrada.
- *nota duplicada* → ya existe una canónica con slug muy parecido. Edítala en
  vez de crear otra; `--force` si de verdad es un tema nuevo.
- *cabecera duplicada* → la `## AAAA-MM-DD — <tema>` ya está en la nota viva
  (o es un re-append). Concreta el tema; `--force` solo si es excepción consciente.

Frontmatter obligatorio en toda nota: `tags` + `tier`. `exo write new`
**no los inventa**: `tier` solo entra si pasas `--tier`, y `tags` solo si van
en el frontmatter del fichero `--from` (se preserva literal). Lo que sí
rellena es `permalink`, `title` y `type`. Tampoco rechaza por frontmatter —un
cierre de sesión no puede fallar por metadatos—, pero si falta alguna de las
dos lo avisa por stderr y en `frontmatter_missing` del envelope: si lo ves,
corrígelo antes del commit. Títulos consistentes: el title es el id de los
wikilinks `[[...]]` — reusa el exacto al enlazar.

## Paso 4 · Commit scoped y resumen

Commitea SOLO los ficheros que esta invocación escribió o editó — nunca
`git add -A`. Las rutas salen del campo `absolute_path` de cada envelope de
`exo write` y de los `Edit` que hiciste; exo **no commitea**, a propósito.
`git -C <repo>` (nunca `cd`). NUNCA push. Mensaje: `docs(kb): documenta
<resumen corto>`. Ante `.git/index.lock`: espera ~2s, reintenta una vez; si
sigue bloqueado, reporta para commit manual — no fuerces borrando el lock.

Si el gate mordió, el cuerpo del commit lleva un trailer por nota (una línea
cada una): `Gate: <ruta> bytes=<N> limit=<techo> routed=bitacora`. `<ruta>` va
relativa a la raíz de la KB (la `path` de `search`); `bytes` y `limit` son enteros
sin separador de miles (`limit=12500`). Va en el bloque final del mensaje, junto
a la atribución (`Co-Authored-By`…). Es lo que cuenta `git log --fixed-strings
--grep='Gate: <ruta>'` (sin `--fixed-strings` las rutas con ` — ` y paréntesis
no casan).

El pre-commit de la KB es la red, no el gate: atrapa `--no-verify` y editores
que no pasan por `/document`. Si aun así rechaza por presupuesto, **el rechazo
no se negocia**: no subas `kbx_budget_max`, no recortes el canon para que pase,
no uses `--no-verify`. Revierte el delta del canon a la línea de estado, deja el
detalle en la bitácora y deja la poda o la partición a
`/distill`. Un commit sin hacer se arregla en un minuto; una nota mutilada, no.

## Paso 5 · Auto-distill (solo si el gate mordió)

Tras el commit, y solo si el gate mordió en alguna nota y no existe ya la rama
`distill/auto/<slug>` (`slug` = último segmento del permalink;
`git -C <kb> branch --list 'distill/auto/<slug>'`): lanza un subagente
(`model: sonnet`, contexto limpio) con el brief de `auto-distill.md` y **espéralo**
(síncrono, tope 10 min). Nunca en segundo plano: el harness mata el process group
al cerrar la sesión. Rama ya existente = ya lanzado, no un error: sigue. El tope es best-effort: al
vencer el subagente no commitea y deja su informe; el trailer `Gate:` ya está en
main, así que `/distill` lo recoge. Cuando el subagente vuelva (o falle),
comprueba `git -C <kb> log --oneline HEAD..distill/auto/<slug>`: si sale vacío no
hubo commit ⇒ `git -C <kb> worktree remove --force <wt>` (si existe) y
`git -C <kb> branch -D distill/auto/<slug>`, o la rama bloquearía la nota.

Resumen final: qué notas se crearon o editaron, dónde quedaron, el hash del
commit y, si hubo auto-distill, en qué rama `distill/auto/<slug>` quedó o que
venció el tope.
