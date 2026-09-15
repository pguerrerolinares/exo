# Consolidación del paso 2

**Cuándo cargar:** en el paso 2, antes de partir o destilar una nota
`core`/`stable` obesa, o al barrer el backlog.

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

Mismo contrato que `/document` v2: la nota canónica es el **estado vivo**, editado
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
pero este flush periódico es lo que evita que se dispare. (En caliente, `/document`
marca el `[x]` de una línea al cerrar; aquí, offline, se barren los viejos.)

> El remedio del Backlog al morder es **cerrar y archivar frentes, no destilar
> el texto de los abiertos**. Un frente abierto se describe entero o no se
> describe; comprimirlo lo rompe como estado vivo, igual que a un índice.
