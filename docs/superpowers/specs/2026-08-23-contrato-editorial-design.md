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
de las 46 notas core+stable, **10 tienen techo sellado por waiver** (por encima
de su nominal de tier) y una, `evidencia-y-divulgacion`, lo tiene **por debajo**:
stable con `kbx_budget_max: 10000` contra un nominal de 12.500, una
auto-restricción voluntaria que nunca aparece en el bucket `waived`. Once sellos,
diez waivers — verificado con `kbx budget --json`. El matiz refuerza el punto: un
techo por nota puede estar por encima *o por debajo* del default de su tier. El sistema real es techo por nota +
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

**Corregido tras la review del 2026-08-23. La v1 de esta spec medía la cosa
equivocada, en la unidad equivocada.** Se deja escrito el error porque es
instructivo: dio un presupuesto holgado que no existe.

El cap **no es del fichero `core-index.md`, y no es en caracteres**. Es
`--cap-bytes 6144` (`exo-recall.sh:36`, `EXO_RECALL_CAP`) aplicado al **bloque
compuesto** que sirve `exo recall`: cabecera + cuerpo de `core-index` + hasta 10
punteros de "Actividad reciente". El motor cuenta **bytes** UTF-8
(`engine/src/recall.rs`, `aplica_cap` sobre `.len()`) y corta por **líneas
enteras** desde el final.

Estado real, medido ejecutando el binario:

```
$ exo recall --db ~/.exo/index.db --contenido \
      --nota kb-demo/core/core-index --limite 10 --cap-bytes 6144
aviso: bloque de arranque truncado por --cap-bytes=6144 (6 líneas descartadas de 51)

bloque completo   6.570 B
cap               6.144 B
déficit               426 B   ← HOY, sin tocar nada
```

**El arranque ya está truncando y nadie lo había mirado**: de 10 punteros de
actividad reciente sobreviven ~4. Y **no cae al FALLBACK** — el guard semántico
del hook busca la cadena "Contrato de memoria", que está al principio y siempre
sobrevive. El único rastro es un `recall-fallback reason=truncated` en
`reflex-log.jsonl` (hook líneas 65-67) que nadie lee.

Con la doctrina nueva de §3 (+375 B) el exceso sube a **801 B**.

**Varianza, que limita lo que se puede prometer**: los punteros pesan entre 46 y
120 B según el nombre de la nota, así que la sección oscila ~450 B entre un día
de nombres cortos y uno de nombres largos. **No se puede garantizar 10/10
punteros** con ningún presupuesto razonable de `core-index`. Lo que sí se puede,
y §4.6 lo hace, es que dejar de servirlos deje de ser silencioso.

Las dos notas donde naturalmente iría más doctrina están además selladas al ras:
`doctrina-agentes` con **33 B** de aire, `desarrollo-agentico` con **49 B**. No
cabe nada en ninguna hasta que el runbook las parta.

Por tanto esta spec escribe en dos sitios —`core-index` y el `SKILL.md` de
`/consolida` (sin techo)— y **paga los 801 B con una evicción del índice**
(§3.1). Todo lo que no quepa se difiere al runbook, no se fuerza.

## 3. Cambio 1 — `core-index`

### 3.1 Pagar los 801 B: evicción, no compresión

La doctrina que entró en `875e0fc` es explícita: *un índice no se destila;
cuando muerde se le retiran **entradas muertas**, no se comprimen las vivas —
comprimir un índice lo rompe como índice.* Esta evicción la respeta: **no se
toca ni una regla viva ni un puntero vivo**.

Lo que sale es de dos clases. Entradas muertas, y —la clase que hace el grueso—
**justificaciones**: `core-index` ha ido acumulando el *porqué* de cada regla
(papers, precedentes, mediciones). El porqué pertenece a la nota destilada o a
la bitácora; el índice necesita la regla y el puntero. Sacarlo no es comprimir
una entrada viva: es sacar material que dejó de ser índice.

| Fragmento | B |
|---|---|
| entrada `cgeo` (declarada JUBILADO en la propia línea) | 173 |
| entrada `lighthouses-bot` (declarada CERRADO) | 87 |
| «la reescritura iterativa erosiona el detalle: *brevity bias*… arXiv:2510.04618» | 102 |
| «Precedente: Wikipedia excluye listas e índices del cómputo de tamaño» | 73 |
| «Más contexto en el padre = peor rendimiento (context-rot)» | 60 |
| cabecera de la nota, reescrita (ver abajo) | 60 |
| «eran el 31% del coste de carga antes de la rotación F0» | 58 |
| «sustituye al memory packet — 0/16 uso medido» | 49 |
| «contenido resuelto, cap 2KB» | 30 |
| «sustituye a superpowers» | 25 |
| «retrieve > compute» | 20 |
| **total identificado** | **737** |

Faltan **64 B** para los 801, y hay que ir a por **~866** para dejar ~65 de
margen contra la varianza de la sección de recientes. Los últimos se buscan en
la pasada con el mismo criterio —justificación, no regla— y se mide el bloque
real, no el fichero. **Si no se llega sin tocar una regla viva, se para y se
consulta**: el criterio de parada es no romper el índice, no cumplir la cifra.

Cada fragmento que sale y siga valiendo va a `log/`, no se borra. La regla de
oro de `/consolida` aplica igual aquí.

### 3.2 La cabecera de la nota describe mal su propio límite

Este es el origen del error de la v1 de esta spec, y hay que arreglarlo
independientemente de todo lo demás. La cabecera dice hoy dos cosas falsas:
**«6.144 caracteres»** (son bytes) y **«pasarse hace caer el arranque al
FALLBACK»** (no: trunca en silencio por el final, y el FALLBACK no se dispara
porque el guard busca "Contrato de memoria", que sobrevive siempre).

```markdown
Se inyecta al arranque (hook reflex) dentro de un bloque de **6.144 bytes**
compartido con los punteros de actividad reciente: lo que sobra se trunca **en
silencio** por el final. Doctrina completa: [[doctrina-agentes]].
```

280 B → 220 B, y deja de mentir. Es el mismo defecto que esta spec cura en el
bullet de presupuestos, en la propia nota que lo denuncia.

### 3.3 El bullet de presupuestos

Se **reemplaza** la línea 19 (`- Presupuestos: …`, 325 chars) por dos bullets.
Reemplazo, no añadido: la doctrina de índices dice que un índice se poda por
evicción, y este texto está sustituyendo al que mentía, no acumulándose.

```markdown
- Presupuestos: el sistema real es **techo por nota** (`kbx_budget_max`, sellado
  en `.kbx-ratchet.json`) + **trinquete: solo baja** — subirlo, borrar el sello o
  reclasificar a `log` es rojo (`kbx ratchet`). Los nominales de tier (core 8.500,
  stable 12.500 B) son el default de una nota **nueva**: de las 11 con techo
  sellado, 10 lo tienen por encima y una por debajo. Sellar o bajar un techo exige **15% de aire**; a ras es un mordisco
  programado para mañana.
- **Al morder el gate**: partir (canon + bitácora), rotar la bitácora o
  consolidar. Nunca subas el techo, nunca mutiles la nota y **nunca recortes el
  delta que ibas a escribir** para que quepa. Si nada cabe, deja el commit
  pendiente y dilo.
```

(En el fichero va en dos líneas sin los saltos de arriba, que son de esta spec.)

**Presupuesto**: 705 B sustituyendo a 330 → delta **+375 B**. No queda margen
libre que gastar: el bloque ya estaba 426 B sobresuscrito antes de esto, y los
801 los paga íntegros la evicción de §3.1. Se verifica corriendo `exo recall`
(§6), nunca con `wc -m` del fichero.

**Variante de reserva** si la evicción de §3.1 no llega a ~866 B sin tocar una
regla viva: fusionar los dos bullets en uno solo (~+215 B en vez de +375), lo
que baja el objetivo a ~706. Se pierde la separación visual del bullet del
mordisco, que es justo el que debe leerse antes de escribir. Antes de eso, se
consulta: puede ser preferible perder un puntero de recientes que fundir la
doctrina.

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

### 4.6 Paso 1 — el truncado del arranque deja de ser silencioso

Este es el arreglo estructural del hallazgo de §2, y cuesta una línea. El paso 1
ya inspecciona `reflex-log.jsonl` para `inject-failed` / `inject-abstained`. Se
añade la misma vigilancia para el truncado del bloque de arranque:

> Chequea también `recall-fallback` con `reason=truncated`
> (`jq 'select(.reflex=="recall-fallback" and (.detail|test("truncated")))'`).
> Cada uno es un arranque servido incompleto: el cuerpo de `core-index`
> sobrevive siempre —el guard busca "Contrato de memoria", que está al
> principio— y lo que se cae por el final son los punteros de actividad
> reciente, sin que nada lo diga. Sostenido = `core-index` está sobresuscrito y
> toca evicción del índice (entradas muertas y justificaciones, nunca comprimir
> entradas vivas). Compruébalo con el bloque real, no con `wc` del fichero:
>
>     exo recall --db ~/.exo/index.db --contenido \
>         --nota kb-demo/core/core-index --limite 10 --cap-bytes 6144
>
> Un `aviso: … truncado` en stderr es la señal.

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

1. **Bloqueante, y se mide sobre el bloque compuesto en BYTES, nunca con `wc -m`
   del fichero** (ese fue el error de la v1):

       exo recall --db ~/.exo/index.db --contenido \
           --nota kb-demo/core/core-index --limite 10 --cap-bytes 6144

   stderr **sin** `aviso: … truncado`, y el bloque ≤ 6.144 B. Ausencia de
   FALLBACK **no** es criterio: el FALLBACK no se dispara al truncar.
2. El bloque servido contiene la cabecera `--- Actividad reciente ---` **y sus
   10 punteros**. Contarlos; que aparezca la cabecera no basta.
3. Arranque de sesión nuevo: el bloque inyectado se lee y se comprueba contra
   lo anterior. No se supone.
4. `core-index` no menciona los presupuestos de tier como si fueran el sistema,
   y su cabecera dice "bytes" y "trunca en silencio", no "caracteres" y
   "FALLBACK".
5. El SKILL.md de `/consolida` nombra evicción editorial, test del título,
   lectura de `no-air-debt`, remedio del Backlog, resellado atómico y la
   vigilancia de `recall-fallback reason=truncated`.
6. Commit scoped en cada repo: `git -C` explícito, rutas explícitas, sin
   `git add -A`, sin push.
