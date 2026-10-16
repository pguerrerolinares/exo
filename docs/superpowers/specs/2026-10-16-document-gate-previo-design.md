# /document con gate previo — routing por número, cabeceras únicas y distill semanal

**Fecha:** 2026-10-16 · **Estado:** diseño debatido en tres rondas con consultor Fable y auditado por un segundo Fable (dos bloqueantes, ajustados; subconjunto mínimo adoptado). Pendiente de aprobación del mantenedor.
**Antecedentes:** `2026-08-23-contrato-editorial-design.md` (canon como delta, bitácora como append, distill como consolidación con juicio). Esta spec no cambia ese contrato: lo hace cumplir antes de escribir en vez de después.

## Qué resuelve

El contrato editorial se diseñó con un ciclo llenar→podar de ~17 días (ficha exo, 08-23 → 09-09: +2,4 KB). En octubre ese ciclo bajó de un día. Medido en la KB privada del mantenedor (ficha bizkaia-now, commits `documenta` desde el 09-23):

- **Presión del canon.** 52 de 55 commits `documenta` dejan la ficha por encima del 85 % de su techo nominal (10.625 de 12.500 B); 38 de 55, por encima de 11.900 B. Los tamaños previos a cada distill (12.471 / 12.484 / 12.492 B) son la firma del delta recortado para caber que ya midió la spec del 08-23 (§1b).
- **Evicción en caliente.** 19 de 56 commits `documenta` borran ≥4 líneas de la ficha: el agente poda lo que tiene delante para que el pre-commit pase. Una cabecera en `archive/` lo confiesa: «Movido del canon (para hacer sitio al cierre de la sesión de issues)».
- **Routing.** Las inserciones al canon son 0,20 de las de la bitácora (160/792 líneas). El canon se ha vuelto una cronología con lápidas: 35 menciones de fecha en 104 líneas, 19 punteros «ver bitácora».
- **Punteros que decaen.** De los 16 punteros con etiqueta de la ficha, 8 solo resuelven en `archive/` (dos saltos), 1 no resuelve en ningún sitio y 7 citan la misma etiqueta genérica «movido del canon (distill, verbatim)», que agrupa bloques distintos bajo una cabecera. Con `hot_bytes` = 20.480 B (default heredado de kbx) y 8-16 KB/día de bitácora, cada rotación (~1,5 días) manda los punteros de ayer a `archive/`.
- **Nombres de archive mal puestos.** `rotacion.rs::nombre_de_archivo` (`:209-222`) saca el rango de fechas de **todo el cuerpo** del bloque frío, no de las cabeceras `## `, y no valida la fecha. 19 de 53 archives tienen un rango que no coincide con sus cabeceras; uno lleva `2026-02-31`.
- **Distill degeneró en janitor.** Las 9 pasadas desde el 09-19 son rotate + evicción de bizkaia/exo + fecha en core-index. La promoción de doctrina y el archivado de sesiones llevan 4 semanas sin correr. La cola `no_air` mezcla frentes calientes con fichas congeladas desde julio.

**Objetivo:** que `/document` decida dónde va cada cosa **por el número antes de escribir**, que nunca pode en caliente, que los punteros sobrevivan a la rotación, y que `/distill` vuelva a ser semanal y de juicio.

## Decisiones (y alternativas descartadas)

- **No fusionar `/document` y `/distill`.** La fusión mueve la evicción al cierre de sesión, que es justo el modo de fallo medido (prisa, contexto lleno, se poda lo que está delante). Partir una nota exige el test del título de `consolidacion.md`: es trabajo de juicio, offline.
- **Gate previo por nivel, no posterior ni por delta.** Hoy el gate es el pre-commit: llega después de escribir, cuando el agente está en modo «haz que pase». Un gate por «delta previsto» no tiene mecanismo (el canon se edita con `Edit`, no hay fichero que medir) y vuelve a ser juicio. El gate mira el tamaño actual de la ruta con `wc -c`: decide el número, no el agente. El pre-commit se queda como red para `--no-verify` y para editores que no pasan por `/document`.
- **Histéresis: distill poda a 60 %, el gate muerde a 85 %.** Si distill para en el mismo umbral en que muerde el gate, el primer `/document` tras cada distill ya muerde (hoy: bizkaia 10.561 B y exo 10.563 B, a ~60 B del 85 %). Un objetivo de 75 % solo dejaba ~1.250 B (4-6 commits `documenta`; bizkaia hizo 11 el 10-16). Con 60 % quedan ~3.100 B, 10-15 commits. Alternativa descartada: gate por crecimiento desde el último distill (necesita estado y se esquiva repartiendo).
- **Si la evicción no llega al 60 % sin tocar reglas ni decisiones vivas, se parte; no se fuerza.** Medido hoy: en bizkaia, `## For agents` es ~5,3 KB de 10,6 (reglas operativas, no evictables); en exo, `## Estado` es ~8 KB de 10,6 (cronología, evictable). Son enfermedades distintas: la primera se cura partiendo por género (como ya se hizo con [[bizkaia-now — reglas de la app Android]]), la segunda moviendo a bitácora. El objetivo de 60 % no es un techo que se cumpla mutilando: no alcanzarlo con evicción editorial es la señal de partición.
- **Distill (manual) crea notas solo por partición; el auto-distill solo la propone.** Partir es la forma de liberar espacio que no pierde nada, pero crear notas sin regla reparte el problema en vez de resolverlo. Toda nota nueva de una pasada sale del test del título de `consolidacion.md` (por género o por tema), la madre la enlaza en `## Relations` y queda como puerta de routing, y los criterios miden la **familia** (madre + hijas), no solo la madre.
- **Auto-distill en subagente, al estilo `/compact` pero sin pérdida.** Como compact, se dispara solo por umbral y corre fuera del trabajo de la sesión; a diferencia de compact, **no resume**: mueve bloques verbatim o parte. Reescribir iterativamente el contexto con un LLM lo erosiona («context collapse», ACE, Zhang et al. 2025); el remedio son updates incrementales, que es el contrato canon-delta / bitácora-append. El fallo medido de la evicción en caliente tenía dos causas: contexto lleno y un objetivo numérico («que quepa»). El subagente quita la primera; la segunda la hace inocua que todo salga verbatim y se verifique literal en destino (reversible, sin pérdida). Escribe en una rama de la KB y el mantenedor decide el merge, como en la fábrica: automático en el disparo, revisado en la escritura.
- **Sin rotación en `/document`.** Con cabecera única el puntero se resuelve por grep también en `archive/`, y con `hot_bytes` = 40.960 B un distill semanal rota una vez. Rotar en cada cierre costaba un flag nuevo, precondición git, chequeo de conservación y un segundo commit.
- **`hot_bytes` = 40.960 B.** Una bitácora caliente de 80 KB que un agente lee entera siguiendo un puntero sin ancla es un coste cierto; el salto a `archive/` con cabecera única es barato.
- **Mordidas como trailer de commit, no en reflex-log ni en la bitácora.** `reflex-log.jsonl` es por máquina y hoy tiene un día de historia; la bitácora es memoria de proyecto que leen agentes. Un trailer `Gate:` en el commit `documenta` está versionado, viaja entre máquinas, no mete ruido en notas y se cuenta con `git log --grep`.
- **Cabecera única comprobada por el engine.** La regla en prosa no basta (ya hay variantes ` — ` / ` · ` en `log/`). `exo write append` lee la nota de todos modos: rechazar ahí una cabecera duplicada es mecánico y no obliga al agente a leer la bitácora.
- **Sin finding `estado_cronologico`.** `exo lint` no tiene severidad advisory: todo hallazgo cambia el exit (`lint.rs:29-36`, `:538`; `main.rs:1329`). Añadirla es superficie nueva para algo que se mide a mano con un grep una vez.
- **Partir con juicio y revisión, no en el cierre.** La partición la propone el auto-distill (o el distill manual) en rama, nunca la sesión que documenta.

## Cambios

### Engine

1. **`rotacion.rs::nombre_de_archivo`** extrae fechas solo de las líneas `## ` del bloque frío (reusando `lineas_con_offset` y el tratamiento de vallas de `offsets_de_entradas`) y descarta fechas inválidas (mes 1-12, día 1-31). Si ninguna cabecera lleva fecha válida, `parte-NN` como hoy. Test con fixture donde cuerpo y cabeceras discrepan (`2026-02-31` y una fecha futura en el cuerpo). Los archives existentes **no se renombran** (los enlazan las cabeceras de las bitácoras).
2. **`--hot-bytes` default 40960** (`main.rs:438`).
3. **`exo write append` rechaza cabecera duplicada.** Si alguna línea `## ` del bloque entrante ya existe literal en la nota destino, sale con exit 3 (mismo canal que los rechazos actuales) y nombra la cabecera; `--force` la deja pasar y queda registrada como los demás `--force`. Solo mira la nota viva, no `archive/`: una cabecera repetida tras rotar es posible y la detecta P4.

### Plugin — `document/SKILL.md`

1. **Paso 2, gate previo.** Antes de editar un canon: `wc -c <ruta>`. Techo = `kbx_budget_max` del frontmatter si existe, si no el nominal del tier (core 8.500, stable 12.500). Si el tamaño es ≥ 85 % del techo, **al canon solo va la línea de estado, sustituyendo a la anterior; todo el detalle va a la bitácora**. Nunca evicción en caliente: borrar del canon para hacer sitio es trabajo de `/distill`.
2. **Cabecera única con formato fijo.** Toda entrada de bitácora empieza por `## AAAA-MM-DD — <tema concreto>` (separador ` — `, em dash con espacios). El puntero del canon cita esa cabecera: `[[<slug>-bitacora]], AAAA-MM-DD «<tema concreto>»`. Prohibidas las etiquetas genéricas («movido del canon», «delta», «detalle», «distill») sin tema detrás.
3. **Estado sustituye, no suma.** La sección de estado se reescribe; no se añaden bullets fechados por sesión.
4. **Paso 4, trailer.** Si el gate mordió, el commit lleva `Gate: <ruta> bytes=<N> limit=<techo> routed=bitacora` (una línea por nota).
5. **Paso 5, auto-distill.** Tras el commit, si el gate mordió y no existe ya la rama `distill/auto/<slug>`, lanza un subagente (`model: sonnet`, contexto limpio) con el brief de abajo y **lo espera** (síncrono, tope 10 min). Al vencer el tope, el subagente no commitea y deja su informe; el trailer `Gate:` ya está en main, así que el distill manual lo recoge. El resumen final dice en qué rama quedó, o que venció el tope.

### Auto-distill (subagente)

- **Dónde:** worktree de la KB en la rama `distill/auto/<slug>` desde `HEAD`. Nunca escribe en la rama principal ni hace merge ni push. **`--kb <worktree>` obligatorio** en todo comando `exo` (sin él, `exo write` escribe en el checkout principal por `[kb] path`). El índice de exo no ve la rama hasta el merge: correcto. Si la rama ya existe (otra sesión con el mismo gate), es «ya lanzado», no un error.
- **Qué:** solo la nota que mordió. Evicción editorial hasta ≤60 % del techo (misma regla de techo que el gate: `kbx_budget_max` del frontmatter o nominal), moviendo bloques enteros verbatim. El brief lleva la pregunta de `consolidacion.md` («¿qué párrafo ya no paga su sitio?») y `git log -p` de la nota: sin ellos el subagente evicta por edad, que es la rotación por orden de llegada que la spec del 08-23 ya descartó.
- **La rama no toca la bitácora.** Main sigue haciendo append al final de la misma bitácora mientras la rama espera: dos appends en EOF conflictan en cada merge. Los bloques movidos van a un sidecar `.distill/<slug>-moved.md` (una entrada `## AAAA-MM-DD — movido del canon: <tema>` por bloque) y el merge los aplica en main con `exo write append --from`, que con el rechazo de cabecera duplicada es idempotente; después se borra el sidecar. En la rama solo cambia el canon (conflicto esperado: la línea de estado, a mano).
- **No crea notas.** Si no llega al 60 % sin tocar reglas (`## For agents`, `## Reglas duras`) ni decisiones vivas, escribe una **propuesta de partición** (títulos que pasen el test del título, qué secciones van a cada hija, bytes resultantes) en el cuerpo del commit. Una hija mal nacida tiene permalink, `## Relations` y entrada en core-index: deshacerla cuesta más que hacerla, y partir es decisión editorial del mantenedor. La ejecuta él o el distill manual.
- **Verificación antes de commitear en su rama:** conservación (todo bloque que sale del canon aparece literal en el sidecar), `exo budget --kb <worktree>` sin offenders, `exo lint --kb <worktree>` limpio. Si algo falla, no commitea y deja el informe.
- **Salida:** un commit `docs(kb): auto-distill <slug> — <evicción|propuesta de partición>` en su rama, con bytes antes/después y bloques movidos en el cuerpo.
- **Ramas sin revisar:** la fábrica muestra que esperan (merge = mantenedor). La degradación es segura para los datos (todo a bitácora), pero el canon queda en «solo línea de estado» hasta el merge, y **P1 pasa a medir también la latencia de merge**. El distill semanal revisa las ramas pendientes; no las rehace automáticamente (quemaría una pasada por semana por nota caliente).
- **Ciclo de vida:** síncrono dentro de `/document`, tope 10 min. Duración esperada 2-5 min con sonnet: la evicción típica mueve ~2 KB (`5cdf292`: 10 líneas del canon, 47 a la bitácora) y las 4 últimas pasadas manuales de Opus, rotate incluido, tardaron 4-14 min. El brief acota `git log -p` a `-n 10` (el histórico entero de una ficha caliente pesa ~420 KB). Muerde como mucho una vez cada ~1,5 días por frente caliente.
- **Aviso al arrancar:** `exo-recall.sh` (SessionStart) inyecta una línea «gate pendiente en <nota>, sin rama `distill/auto/`» cuando hay un trailer `Gate:` posterior a `distill/last` sin rama ni merge. Es red para el distill manual, no un lanzador: un hook no lanza agentes y su texto se obedece de forma parcial (4/11 y 6/10 medidos en `techo-reglas`).

### Plugin — `distill/SKILL.md`, `chequeos.md`, `rotacion.md`, `consolidacion.md`

1. **Cadencia semanal**, con promoción de doctrina (paso 4) y barrido por calor obligatorios en cada pasada; si no hay nada, se dice. Primero revisa las ramas `distill/auto/*` pendientes (merge, rehacer o descartar).
2. **Objetivo de poda: ≤60 % del techo** para toda nota que se evicte (hoy el criterio implícito es salir de `no_air`, 85 %). Si no se alcanza sin tocar reglas ni decisiones vivas, partición (test del título); nunca se recorta prosa viva para llegar.
2b. **Notas nuevas solo por partición**, con las mismas reglas que el auto-distill (título que pase el test, `## Relations` en ambos sentidos, madre como puerta).
3. **Precondición por pathspec** en `rotacion.md`: `git status --porcelain -- log/ archive/log/`, no global.
4. **Conservación + nombres** en `rotacion.md`: además de contar cabeceras, comprobar que el rango del nombre del archive nuevo coincide con min/max de sus cabeceras `## `.
5. **Cola por calor** en `chequeos.md`: `no_air` cruzada con `exo stale` (`age_days`); una ficha sin commits en >30 días no es trabajo de la pasada.
6. **Disparador de partición** (para el distill manual): una nota con ≥2 trailers `Gate:` en los últimos 7 días (`git log --since=7.days --fixed-strings --grep='Gate: <ruta>'`; sin `--fixed-strings` las rutas con ` — ` y paréntesis no casan y el disparador calla) sin rama `distill/auto/` mergeada entre medias indica que el auto-distill no basta: partición con juicio.
7. **Evicción con cabecera única** en `consolidacion.md`: cada bloque movido entra con su propia cabecera `## AAAA-MM-DD — movido del canon: <tema>`, nunca varios bajo una genérica.

## Criterios de éxito (pre-registrados)

Revisión única hacia el **2026-11-06**, sobre commits de la KB desde el merge, a mano. Se normaliza **por commit `documenta`**, no por día. **Ficha activa** = destilado listado en «Destilados de proyecto activos» de `core-index`; **familia** = la ficha y las notas partidas de ella (P1-P3 se miden sobre cada miembro de la familia, para que partir no cumpla el criterio por sí solo). **Techo** = `kbx_budget_max` o nominal del tier (hoy bizkaia-now y exo: nominal 12.500, sin entrada en el ratchet).

| | Definición | Medida | Hoy | Umbral |
|---|---|---|---|---|
| **P1** | Commits `documenta` que dejan una ficha activa >85 % de su techo | `git log -i --grep=documenta --format=%h -- <ficha>` + `git show <h>:<ficha> \| wc -c` | bizkaia 52/55 | ≤20 % |
| **P2** | Commits `documenta` con ≥4 líneas borradas en una ficha activa **fuera de su sección de estado** | `git log -p -i --grep=documenta -- <ficha>`, borrados fuera del hunk de `## Estado` | bizkaia 19/56 (sin excluir estado) | ≤10 % |
| **P4** | Punteros `[[x-bitacora]], fecha «label»` de fichas activas: punteros distintos que resuelven a cabeceras distintas, cada una única en `log/` ∪ `archive/log/`, y ninguna con etiqueta de la lista negra | grep de cada etiqueta, hits = 1, sin dos punteros a la misma cabecera | 8/16 | ≥90 % |

Diagnóstico (no son criterio de éxito): **P3** routing bruto canon/bitácora en commits `documenta` (hoy 0,20; esperado ≤0,10); **P6** trailers `Gate:` por nota y semana (esperado ≤2 desde la semana 2); **P7** marcas de fecha en el cuerpo de fichas activas (grep de ISO ≤ hoy y `MM-DD` entre paréntesis o tras `el|del|desde el`, contando cada fecha una vez; hoy exo entre 9 y 18 según normalización, bizkaia 13 ISO). Regression check, no criterio: **P5** archives nuevos con rango ≠ cabeceras = 0 (hoy 19/53 históricos).

Lectura cruzada: P1+P2 bien y P3 mal ⇒ el canon crece por hijas; P4 mal ⇒ siguen las etiquetas genéricas o la cita no coincide con la cabecera; P6 alto en semana 3 ⇒ la partición no se hizo o la hija es un cajón.

## Fuera de alcance

- Renombrar los 19 archives mal nombrados (los enlazan cabeceras).
- Rotación desde `/document` y `exo rotate --note`.
- Severidad advisory en `exo lint`.
- `hot_bytes` por nota en frontmatter.
- Arreglar a mano los punteros actuales de las fichas (deuda de la KB, no del plugin; se hace aparte).

## Riesgos

- **El gate degrada el canon a tablero de líneas de estado** si «sustituye» no se cumple: lo vigila P7.
- **Histéresis mal calibrada:** si con 60/85 el gate muerde en la primera semana igualmente, la señal es P6; se parte, nunca se sube el gate.
- **Proliferación de notas:** partir sin test del título convierte una ficha obesa en un cajón de hijas. Lo vigilan la medida por familia y que solo el distill manual cree notas.
- **El subagente muere con la sesión.** `/document` es la skill de cierre y el harness mata el process group al terminar (`exo-index.sh:49-52`: solo sobrevive lo lanzado con `setsid -f nohup`). Un `Agent` en segundo plano cae con la sesión. Decidido: síncrono con tope (arriba). Descartado `setsid -f nohup claude -p`: el único precedente headless del repo (`scripts/e2e-reglas-proyecto.sh:13-15`) corre con deny-list de lectura; escribir exigiría `bypassPermissions` o una allow-list, y el fallo acabaría en un log que nadie lee — el caso canónico de [[Fallo silencioso en sistemas — lo que devuelve al romperse]]. Descartado también lanzar desde un hook (SessionStart/Stop): un hook no lanza agentes y el de Stop muere igual con el process group.
- **Ramas acumuladas sin revisar:** si el mantenedor no mergea, el canon sigue lleno y el gate muerde en cada cierre (todo a bitácora, sin pérdida). Degradación segura; el distill semanal las limpia.
- **Subagente que mutila:** la verificación de conservación (cada bloque movido aparece literal en destino) es mecánica y bloquea su commit.
- **Cabecera duplicada legítima** (dos sesiones el mismo día con el mismo tema): el rechazo obliga a concretar el tema; `--force` existe para la excepción.
