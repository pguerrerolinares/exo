# Contrato editorial: doctrina de presupuestos y evicción en /consolida — design spec

Fecha: 2026-08-23. Origen: expediente de auditoría del sistema de presupuestos
de la KB (`docs/superpowers/consultas/2026-08-22-presupuestos/`), propuesta v2,
puntos **Pata A.2, A.3, A.4 y A.5** más el test del título de §1.4.
Estado: **diseño aprobado por Paul, pendiente de plan.**
Repos: `exo` (`plugins/reflex/skills/consolida/SKILL.md`) y la KB `kb-demo`
(`core/core-index.md`). **No toca kbx.**
Depende de: la spec de la guarda de aire (`kbx`), porque la doctrina nueva
describe un comportamiento que el binario debe tener ya.
Bloquea: el runbook de la pasada de factorización, que ejecuta este contrato.

## 1. Problema

Tres huecos distintos, todos de prosa:

**(a) La doctrina miente sobre el sistema.** `core-index` describe presupuestos
por tier (core 8.500 B, stable 12.500 B) como si fueran el sistema. No lo son:
de las 46 notas core+stable, **las 11 que tienen techo sellado lo tienen por
waiver** — ninguna vive de su nominal de tier. El sistema real es techo por nota +
trinquete, y los nominales de tier son el default de una nota nueva. Describir
el default como sistema hace que quien lee no entienda por qué el gate se
comporta como se comporta.

**(b) La semántica del mordisco llega tarde.** El heredoc del pre-commit dice
literalmente *"NO subas kbx_budget_max"*, *"NO recortes la nota a lo bruto"*,
*"PÁRTELA / rota / díselo a Paul"* — y aun así, el día de la auditoría el delta
entrante se recortó **dos veces** para caber. El hook se lee **después** del
rechazo, cuando el agente ya está en modo "haz que pase". Y hay un matiz que el
hook no cubre: habla de no mutilar *la nota*, no de no recortar *lo que ibas a
escribir*. El fallo medido fue el segundo.

**(c) `/consolida` poda sin criterio de valoración.** Su SKILL.md sabe partir
canon/bitácora, rotar y archivar, pero no nombra en ningún sitio la pregunta
editorial: *¿qué párrafo del canon ya no paga su sitio?* Sin ella, podar para
dejar aire (que es lo que la guarda va a exigir) degenera en rotación por orden
de llegada: sale lo viejo por viejo, no lo que sobra.

## 2. Restricción que ordena el trabajo

`core-index` tiene **530 caracteres libres** de un cap duro de 6.144 (guard de
`compose_base` en `exo-recall.sh`; pasarse hace caer el arranque al FALLBACK en
**todas** las sesiones). Y las dos notas donde naturalmente iría más doctrina
están selladas al ras: `doctrina-agentes` con **33 B** de aire,
`desarrollo-agentico` con **49 B**. No cabe nada en ninguna hasta que el runbook
las parta.

Por tanto esta spec escribe en exactamente dos sitios: `core-index` (caro,
presupuestado al carácter abajo) y el `SKILL.md` de `/consolida` (sin techo).
Todo lo que no quepa se difiere al runbook, no se fuerza.

## 3. Cambio 1 — `core-index`, bullet de presupuestos

Se **reemplaza** la línea 19 (`- Presupuestos: …`, 325 chars) por dos bullets.
Reemplazo, no añadido: la doctrina de índices dice que un índice se poda por
evicción, y este texto está sustituyendo al que mentía, no acumulándose.

```markdown
- Presupuestos: el sistema real es **techo por nota** (`kbx_budget_max`, sellado
  en `.kbx-ratchet.json`) + **trinquete: solo baja** — subirlo, borrar el sello o
  reclasificar a `log` es rojo (`kbx ratchet`). Los nominales de tier (core 8.500,
  stable 12.500 B) son el default de una nota **nueva**: las 11 con techo sellado
  son waivers. Sellar o bajar un techo exige **15% de aire**; a ras es un mordisco
  programado para mañana.
- **Al morder el gate**: partir (canon + bitácora), rotar la bitácora o
  consolidar. Nunca subas el techo, nunca mutiles la nota y **nunca recortes el
  delta que ibas a escribir** para que quepa. Si nada cabe, deja el commit
  pendiente y dilo.
```

(En el fichero va en dos líneas sin los saltos de arriba, que son de esta spec.)

**Presupuesto**: 667 caracteres sustituyendo a 325 → delta **+342**, dejando
**188 caracteres libres** (3,1% del cap). Verificar con
`wc -m core/core-index.md` **antes de commitear**; si se pasa de 6.144, no se
recorta el texto: se retira una entrada muerta del índice (`lighthouses-bot`
está declarada CERRADA en la propia nota).

**Variante de reserva** si algo más necesita entrar en el mismo commit: fusionar
los dos bullets en uno solo, 520 chars, delta +195, deja 335 libres. Se pierde
la separación visual del bullet del mordisco, que es justo el que debe leerse
antes de escribir. Solo si hace falta.

**Lo que NO se toca**: el bullet de índices y destilado (línea 20) acaba de
entrar en `875e0fc` y es correcto tal cual.

## 4. Cambio 2 — `SKILL.md` de `/consolida`

Ruta: `plugins/reflex/skills/consolida/SKILL.md` en el repo `exo`. **Ese es el
source de verdad** (reflex 0.15.0); `agent-develop/plugins/reflex` se quedó en
0.13.1 y no se toca. Tras editar, el marketplace `exo-plugins` recoge el cambio
por su vía habitual — no forma parte de esta spec.

### 4.1 Paso 2 — la pregunta editorial

El paso 2 (*Split canon/bitácora por cada core/stable obeso*) gana un párrafo
que nombra la evicción como operación de primera clase, no como efecto del
split:

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

### 4.2 Paso 2 — el test del título

Segundo párrafo, el criterio de triaje que la auditoría operacionalizó (§1.4
del expediente): decide **si una nota debe partirse o solo adelgazar**, con git
y cero métricas nuevas.

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

### 4.3 Paso 1 — la lectura de la deuda de aire

En el paso 1 (*Budget check*), tras el `kbx budget --json`:

> Corre también `kbx ratchet --kb <kb> --json` con el árbol limpio. Los findings
> `no-air-debt` listan las notas cuyo techo está sellado a ras: no bloquean nada
> (la guarda juzga transiciones, no estado), pero cada una es un mordisco
> pendiente. Su campo `limit` da el techo que cumpliría y el mensaje el tamaño
> objetivo de poda. Es la cola de trabajo de esta pasada.

### 4.4 Paso 2 — el caso del Backlog

El caso especial del Backlog ya existe y ya prescribe barrer `[x]` a
`log/backlog-diario.md`. Se le añade una frase que cierra la ambigüedad
señalada en el expediente:

> El remedio del Backlog al morder es **cerrar y archivar frentes, no destilar
> el texto de los abiertos**. Un frente abierto se describe entero o no se
> describe; comprimirlo lo rompe como estado vivo, igual que a un índice.

### 4.5 Paso 5 — resellar

Al final del paso 5, antes del commit:

> Tras podar y partir, corre `kbx ratchet --kb <kb> --seal`. Es **atómico**: o
> sella todo o no sella nada, y si falla lista cada techo sin 15% de aire con su
> objetivo de poda. Esa lista no es un error del sello: es trabajo que falta.
> **Nunca subas un techo para que pase** — el trinquete lo rechazará en el
> commit de todos modos.

## 5. Lo que esta spec deja fuera, y por qué

- **Fase 2 (la banda de dos umbrales)**: condicional por eventos en el
  expediente. No se escribe doctrina de algo que aún no tiene código.
- **Downrank de `archive/` en el retrieval**: decisión independiente de Paul,
  frente propio, no entra aquí.
- **Doctrina nueva en `doctrina-agentes` o `desarrollo-agentico`**: imposible,
  33 y 49 bytes de aire. Se difiere al runbook.
- **El heredoc del pre-commit**: ya dice lo correcto. Duplicarlo no arregla que
  se lea tarde; para eso está el bullet nuevo de `core-index`, que se lee antes.

## 6. Criterio de hecho

1. `wc -m core/core-index.md` ≤ 6.144. **Bloqueante.**
2. Arranque de sesión nuevo: el recall inyecta `core-index` completo, sin
   FALLBACK. Se verifica leyendo el bloque inyectado, no suponiéndolo.
3. `core-index` no menciona los presupuestos de tier como si fueran el sistema.
4. El SKILL.md de `/consolida` nombra evicción editorial, test del título,
   lectura de `no-air-debt`, remedio del Backlog y resellado atómico.
5. Commit scoped en cada repo: `git -C` explícito, rutas explícitas, sin
   `git add -A`, sin push.
