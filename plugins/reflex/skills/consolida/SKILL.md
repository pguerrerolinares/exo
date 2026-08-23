---
name: consolida
description: Consolidación offline de la KB kb-demo (sleep-time compute manual): colapsa bitácoras/sesiones en destilados, chequea presupuestos por tier, promueve doctrina repetida a core y refresca core-index. Usar al cerrar un frente o semanalmente.
---

# consolida

**Qué es:** el "sleep-time compute" manual de la KB kb-demo. Mientras `/documenta`
escribe en caliente (al cerrar una sesión), `/consolida` es mantenimiento offline —
Paul lo invoca al cerrar un frente o semanalmente para que la KB no crezca sin control
ni se llene de doctrina repetida sin destilar.

**Regla de oro: nada se borra.** Todo movimiento es `git mv` o edición; si algo deja de
vivir en su sitio original, se mueve a `archive/` o `log/`, nunca se elimina. El commit
final es el registro de qué se movió y por qué.

## Procedimiento (6 pasos)

### 0. Rotación de bitácoras (antes de cualquier chequeo)

**Precondición dura:** `git -C /home/paul/Documentos/proyectos/kb-demo status --porcelain`
debe salir **vacío** antes de tocar nada. Si no sale vacío, **para** y pide a
Paul que commitee o guarde su trabajo antes de rotar — no sigas por tu
cuenta. El baseline de conservación del punto 2 (`git show HEAD:<ruta>`)
asume que `HEAD` es el estado justo antes de rotar, y la reversión del punto
3 (`git checkout -- <ruta>`) descarta lo que no esté commiteado sin forma de
recuperarlo: con el árbol sucio, ambas cosas quedan mal por construcción.

Corre primero en seco y revisa el resultado:

    /home/paul/.local/bin/kbx rotate --kb /home/paul/Documentos/proyectos/kb-demo --json

Requiere un build de `kbx` que incluya `rotate` — el binario instalado en esa
ruta todavía no trae el subcomando, porque la feature vive en una rama sin
mergear. Si no está disponible, sáltalo y continúa directo al paso 1.

Si `data.rotations` viene vacío, no hay nada que rotar: sigue directo al paso
1. Si trae entradas, repite con `--apply` y verifica antes de continuar:

1. `git -C /home/paul/Documentos/proyectos/kb-demo status --porcelain` —
   deben aparecer las bitácoras modificadas y los nuevos ficheros en
   `archive/log/`.
2. Conservación: para cada bitácora tocada, compara el número de cabeceras
   `## ` de **antes** de rotar —
   `git -C /home/paul/Documentos/proyectos/kb-demo show HEAD:<ruta-de-la-nota> | grep -c '^## '`
   (el cambio aún no está commiteado, así que `HEAD` sigue teniendo el
   contenido previo al `--apply`) — contra la suma de cabeceras `## ` en la
   nota viva actual más las del fichero nuevo en `archive/log/`
   (`grep -c '^## ' <ruta-nota-viva> <ruta-archivo-nuevo>`, sumando ambos
   conteos). Deben coincidir. Si no cuadra, hay una entrada perdida — no
   sigas, revísalo.
3. Nada se borra, misma regla de oro que el resto de la skill. Si algo salió
   mal y hay que revertir:
   - Nota viva: `git -C /home/paul/Documentos/proyectos/kb-demo checkout -- <ruta-de-la-nota>`
     (con el pathspec explícito de la nota — `git checkout --` sin ruta detrás
     no hace nada y no avisa).
   - Ficheros nuevos en `archive/log/`: `checkout` no los toca porque están
     sin trackear; bórralos a mano con las rutas exactas que ya listó
     `git status --porcelain` en el punto 1 (p.ej.
     `rm <ruta-nueva-en-archive-log>`).
   - Verifica con `git -C /home/paul/Documentos/proyectos/kb-demo status --porcelain`
     que no queda nada pendiente, y repórtalo — no continúes al paso 1 con el
     repo en ese estado.

Va antes del budget check a propósito: mueve bytes fríos fuera de las notas
calientes, así que el paso 1 evalúa el presupuesto ya sobre el estado
reducido.

### 1. Budget check

Corre `/home/paul/.local/bin/kbx budget --json`. Devuelve
`{data:{tiers:[{tier,notes,bytes,budget,delta,exceeded}], offenders:[{path,tier,size_bytes,budget}], waived:[{path,tier,size_bytes,budget}]}}`
y **exit 1 si hay algún offender** (incluye NOTIER: nota sin `tier:` o con tier
ilegal), exit 0 si limpio — mismas semantics que el viejo `kb-budget-check.sh`,
que queda retirado. Una nota que rebasa su presupuesto de tier pero cae dentro
de su `kbx_budget_max: N` de frontmatter es una excepción reconocida: exit 0,
listada en `waived` (no en `offenders`). Presupuestos por defecto: core=8.500B,
stable=12.500B, log=sin límite; excluye `archive/`, `docs/`, `.superpowers/`.

> Corre también `kbx ratchet --kb <kb> --json` con el árbol limpio. Los findings
> `no-air-debt` listan las notas cuyo techo está sellado a ras: no bloquean nada
> (la guarda juzga transiciones, no estado), pero cada una es un mordisco
> pendiente. Su campo `limit` da el techo que cumpliría y el mensaje el tamaño
> objetivo de poda. Es la cola de trabajo de esta pasada.

Revisa `waived`: ¿siguen justificadas las excepciones reconocidas? (p.ej. un
`kbx_orphan_ok` en una nota que recuperó relaciones desaparece de `waived` por
sí solo).

**Falla-fuerte:** si el binario no está o el schema-canary rompe (lo verás como
un `schema_drift` en `doctor`, ver abajo), **para** con un mensaje accionable
(`kbx no está → make install`, "schema drift → el binario kbx y el binario exo
están desincronizados: reinstala el que vaya atrasado (`make install` en kbx,
`cargo build --release` + copia en exo) y vuelve a correr"). No degrades a mano:
/consolida es offline y deliberado, el fallo ruidoso es correcto.

### 1b. Gate de deriva + priorización

- **Deriva:** corre `/home/paul/.local/bin/kbx doctor --json`
  (`{data:{ok,findings:[{type,path,detail}], waived:[{type,path,detail}]}}`).
  `ok:true` significa limpio de findings NO waived. Las excepciones
  reconocidas (`orphan` con `kbx_orphan_ok: true`, `budget_exceeded` dentro de
  su `kbx_budget_max`) aterrizan en `waived`, no en `findings`. Sus findings
  alimentan la limpieza (WS4 del spec Fase 2): `duplicate_dir`, `orphan`,
  `bad_frontmatter`, `root_file`. No los muevas a ciegas — cada `git mv` lo
  gatea Paul.
- **Priorización:** corre `/home/paul/.local/bin/kbx stale --json`
  (`{data:{notes:[{path,tier,age_days,degree,score,...}]}}`, orden descendente
  por `score`). Úsalo para decidir QUÉ notas atacar primero en los pasos 2 y 4,
  en vez de ir a ojo. **`stale` no propaga waivers**: una nota con excepción
  reconocida (p.ej. README/metodología) seguirá apareciendo alta en `stale`
  (es advisory) — no es bug.
- Chequea inject-failed E inject-abstained en reflex-log.jsonl (jq 'select(.reflex=="inject-failed" or .reflex=="inject-abstained")'): >0 sostenido = componedor roto en silencio o payloads sin agent_type — never-break no puede significar semanas sin inyección (spec transporte §7).
- Chequea también `recall-fallback` con `reason=truncated`
  (`jq 'select(.reflex=="recall-fallback" and (.detail|test("truncated")))'`).
  Cada uno es un arranque servido incompleto: el cuerpo de `core-index`
  sobrevive siempre —el guard busca "Contrato de memoria", que está al
  principio— y lo que se cae por el final son los punteros de actividad
  reciente, sin que nada lo diga. Sostenido = `core-index` está sobresuscrito y
  toca evicción del índice (entradas muertas y justificaciones, nunca comprimir
  entradas vivas). Compruébalo con el bloque real, no con `wc` del fichero:

      exo recall --db ~/.exo/index.db --contenido \
          --nota kb-demo/core/core-index --limite 10 --cap-bytes 6144

  Un `aviso: … truncado` en stderr es la señal.

### 2. Split canon/bitácora por cada core/stable obeso

> **Evicción editorial (una vez por pasada, por cada nota que se toque).** Antes
> de mover nada por fecha, haz la pregunta de valor: *¿qué párrafo de esta nota
> ya no paga su sitio?* Candidatos: lo que se ha vuelto obvio, lo que quedó
> superado por una decisión posterior, el detalle de una iteración cuya
> conclusión ya está escrita, y el ejemplo que ilustra algo que el texto ya dice.
> Eso baja a la bitácora con su fecha. Lo que queda es lo que sigue siendo
> verdad y sigue costando de recordar.
>
> Va **antes** que el criterio cronológico y **antes** que la poda para dejar
> aire. Sin ella, podar para caber es rotación por orden de llegada: sale lo
> viejo por viejo, no lo que sobra. Y al mover: **bloques enteros, nunca
> re-resumir prosa** (la reescritura iterativa erosiona el detalle).

> **Test del título — ¿partir o destilar?** Mide qué fracción del crecimiento de
> la nota desde la última pasada cayó en cabeceras **nuevas** (`git log -p` sobre
> la nota, contando `^## ` añadidos frente a crecimiento dentro de cabeceras que
> ya existían):
>
> - **~0%** — la nota converge en estructura: engordó por dentro. Remedio:
>   evicción editorial. **No la partas.**
> - **>50% con las cabeceras nuevas afines al título** — tema amplio
>   subdividiéndose. Remedio: partir **por género** (narrativa / referencia /
>   epistemología).
> - **>50% con las cabeceras nuevas sin relación entre sí** — es un cajón, no
>   una nota. Remedio: partir **por tema**, y la madre queda como **índice
>   corto**: puerta única de routing, sin la cual la fricción de espacio se
>   convierte en fricción de routing.
> - **Entremedias** — juicio. Umbral revisable: se calibró con 4 puntos de datos.
>
> Un índice **no se destila**: cuando muerde se le retiran entradas muertas.

Mismo contrato que `/documenta` v2: la nota canónica es el **estado vivo**, editado
como delta (qué es verdad *ahora*); todo lo fechado/histórico (decisiones tomadas en
tal fecha, iteraciones superadas) se mueve a `log/<slug>-bitacora.md`. La canónica
queda dentro de presupuesto porque deja de cargar el historial completo.

**Futuros appends a la bitácora van SIEMPRE después de cualquier snapshot ya movido,
con fecha explícita** — la bitácora es un log append-only ordenado en el tiempo, no
se reescribe hacia atrás.

**Caso especial — el backlog** (`Backlog — frentes abiertos.md`, core): no crece con
Deltas fechados sino con items `[x]` cerrados que se acumulan. Barre los `[x]` que ya
no dan contexto del estado actual (deja los **últimos ~1-3 por frente**) → append
fechado a `log/backlog-diario.md`, y elimínalos del backlog. Conserva SIEMPRE todos los
`[ ]` abiertos. El backlog debe tender a ≈ **abiertos + cola corta de recién-cerrado**.
Es la única nota `core` a la que se le tolera rebasar presupuesto por ser estado vivo,
pero este flush periódico es lo que evita que se dispare. (En caliente, `/documenta`
marca el `[x]` de una línea al cerrar; aquí, offline, se barren los viejos.)

> El remedio del Backlog al morder es **cerrar y archivar frentes, no destilar
> el texto de los abiertos**. Un frente abierto se describe entero o no se
> describe; comprimirlo lo rompe como estado vivo, igual que a un índice.

### 3. Archivar sesiones de frentes cerrados

**Escanea solo lo cambiado.** No re-escanees toda la KB: corre
`/home/paul/.local/bin/kbx diff-since consolida/last --json`
(`{data:{ref,resolved,notes:[{path,permalink,status,insertions,deletions}]}}`)
para ver qué notas cambiaron desde la última consolidación.

**Bootstrap (el tag aún no existe — `git tag -l` está vacío hoy):** si
`consolida/last` no existe, `diff-since` fallará al resolver el ref. Eso **no**
es fallo-fuerte: haz un **full scan** (sin `diff-since`) esta vez. Al terminar
el paso 5 (commit), crea/mueve el tag al HEAD del repo KB:

    git -C /home/paul/Documentos/proyectos/kb-demo tag -f consolida/last HEAD

Es la única mutación del repo KB que hace esta skill más allá de commitear notas.

Para cada nota en `sesiones/` que pertenezca a un frente ya cerrado: escribe un resumen
de 1-3 líneas a la bitácora del proyecto correspondiente (`log/<proyecto>-bitacora.md`)
y `git mv` la sesión completa a `archive/sesiones/`. Actualiza los `[[wikilinks]]`
entrantes que apuntaban a esa sesión para que no queden rotos.

### 4. Promover doctrina repetida

Si un patrón o principio aparece repetido en ≥3 bitácoras (señal de que ya no es
anécdota sino doctrina estable), propón su promoción a `[[doctrina-agentes]]` o al
`learning` que corresponda. No lo hagas automático — es una propuesta a revisar, no
una escritura silenciosa.

### 5. Refrescar índice y commit

Actualiza `[[core-index]]` para que refleje los cores y destilados activos (altas,
bajas de sección, nuevos punteros a bitácoras).

> Tras podar y partir, corre `kbx ratchet --kb <kb> --seal`. Es **atómico**: o
> sella todo o no sella nada, y si falla lista cada techo sin 15% de aire con su
> objetivo de poda. Esa lista no es un error del sello: es trabajo que falta.
> **Nunca subas un techo para que pase** — el trinquete lo rechazará en el
> commit de todos modos.

Luego commit scoped con las mismas reglas git de Paul:

- `git -C /home/paul/Documentos/proyectos/kb-demo add <ruta1> <ruta2> ...` —
  **nunca** `git add -A`, **nunca** `git add .`.
- **Nunca** `cd` encadenado con `git`; usa siempre `git -C <path>`.
- **No hagas push** — esa decisión es de Paul.
- Tras el commit, avanza el marcador de consolidación:
  `git -C /home/paul/Documentos/proyectos/kb-demo tag -f consolida/last HEAD`.
  (No se pushea; es un marcador local para el `diff-since` de la próxima corrida.)

## Delegación

Delegable a un subagente sonnet: los pasos 1-4 son mecánicos/de revisión y el subagente
puede volcar los splits/movimientos propuestos como un **diff revisable antes de
commit** — no comitees a ciegas lo que produzca el subagente, revísalo tú primero
(o Paul).
