# Verdict del consultor — diseño de M6-06, recall en el punto de uso (2026-08-22)

> Consultor fable fresco. Verificación primaria propia contra el índice vivo
> (`~/.exo/index.db`, 145 notas), el binario instalado (`~/.local/bin/exo`), los
> transcripts de `~/.claude/projects/` y la documentación oficial de hooks
> (code.claude.com/docs/en/hooks, consultada hoy). Scripts de medición en mi
> scratchpad (`fts_and_vs_or.py`, `hybrid_or_sim.py`, `prompts2.py`,
> `c3_hybrid.py`, `fts_and_realprompts.py`); KB e índice copiados para C5, cero
> escrituras en producción. No he tocado código, specs ni commits.

## Veredicto global

El mecanismo (hook `UserPromptSubmit`) sigue firmado y se sostiene. Pero **las
dos mitigaciones que la spec §3.2 dejó como candidatas —FTS-only para el coste y
"umbral más alto" para el ruido— caen las dos con dato**, y no de forma
recuperable barata. El diseño que firmo abajo es: **gate léxico en el hook +
hybrid siempre con `--refresca` + punteros con snippet, cap 1 KB, 0.40 sellado
intacto**. Coste declarado: ~0,95 s por turno sustantivo. Es el precio de un
recall que funciona; todo lo que lo esquivaba está medido y roto.

---

## 1. Verificación de claims

### C1 — "hybrid ~1 s POR PROMPT, FTS gratis" → **SE SOSTIENE**

Medido en caliente, 4 queries distintas + 3 repeticiones de la misma:

```
hybrid (recall --query): 0.97 / 0.93 / 0.94 / 0.93 s  (repes: 0.95, 0.94, 0.97)
fts    (search --type fts): ≤0.005 s en todas
```

Estable, CPU-bound (user 0,53 + sys 0,55 ≈ wall): es la carga del runtime ONNX
por proceso, no I/O ni cache fría. No hay "calentamiento" posible entre
invocaciones de CLI: cada prompt paga el segundo entero. Confirma mi riesgo nº1
del 18-ago.

### C2 — "FTS-only roto de fábrica para prompts naturales" → **SE SOSTIENE, y peor de lo que dice el recon**

Reproducido exacto: `"vamos con brainstorm de M6-06"` → 0 hits (AND implícito);
`"que sabemos del regimen de gates delegado"` → 0; `"oye, como iba lo del
cutover de kbx"` → 0. Cualquier palabra de relleno mata el recall léxico.

Lo que el recon no midió — el arreglo obvio también está roto:

- **OR global sobre el eval** (réplica exacta de `busca` en python contra el
  índice vivo): FTS-only pasa de 23/56 a 39/56 hit@5. Parece la solución…
- **…pero OR degrada el hybrid sellado**: simulé la fusión completa
  (`max(v,f)`, β=0.6, bonus=0, K_c=50, threshold 0.40 pre-fusión con el arm
  vector real del engine) sobre las 56 queries: **AND 47/56 → OR 44/56**
  (arregla 1, rompe 4). El canal FTS-OR mete ruido normalizado a β=0.6 que
  desplaza hits del arm vector. Arreglar `prepara_query` no es tocar una
  función: es abrir un camino léxico paralelo solo-hook + revalidar el eval —
  exactamente el ciclo de calibración que §0 retiró.
- **Y FTS-AND como gate o primer escalón está invertido** (hallazgo H1, abajo):
  dispara en el 75% de los prompts triviales ("sí"/"si" matchea 114 de 145
  notas por folding de acentos; "ok" 33; "vale" 29) y calla en el 84% de los
  sustantivos. Un escalonado FTS→hybrid escala justo cuando no debe y responde
  justo cuando no hay nada que decir.

**Qué se lleva por delante**: la mitigación candidata de la spec §3.2 (FTS-only)
queda invalidada entera — ni tal cual, ni arreglada, ni como escalón. El
mecanismo hook no se toca; el motor tiene que ser hybrid.

### C3 — "riesgo de REDUNDANCIA con el arranque" → **CAE en su ejemplo; el riesgo real es otro**

El ejemplo no reproduce: hoy, hybrid para `"vamos con brainstorm de M6-06"`
devuelve 3 notas de `archive/` a ~0,50 — ni core-index ni nada útil. (El recon
probablemente miró el top FTS, donde core-index sí sale 1º.)

Medición amplia — 30 prompts reales sustantivos (muestreados de transcripts,
seed fija), `recall --query --limite 3`:

- `core-index` en top-3: **0/90 hits**. `tier: core` en top-3: 9/90 (10%) —
  pero el arranque solo inyecta el CUERPO de core-index; el resto del tier core
  solo aparece listado. Redundancia real con el arranque: ~cero.
- Solape de top-3 entre turnos consecutivos de la misma sesión: 5/17 pares
  (29%) — existe, coste ~300 B/repetición con punteros. Menor.
- **Lo que el recon no vio (hallazgo H3)**: 30/30 prompts obtienen 3 hits con
  score ≥0,41 — hybrid a 0.40 sobre 145 notas **nunca se abstiene**. "firma,
  /documenta y cierra" → doctrina-agentes a 0,60; "ya lo he finalizado, revisa"
  → 3 notas de archive a 0,45. Y los aciertos buenos (pguerrero-music, exo/kbx
  cuando el prompt nombra el tema) viven en 0,47–0,63: **el mismo rango que el
  ruido**. El problema de primera clase es precisión, no redundancia.

### C4 — "no hay UserPromptSubmit hoy" → **SE SOSTIENE**

reflex 0.14.0 (versión activa en cache): `PreToolUse`, `SessionStart`, `Stop`,
`SubagentStart`. `~/.claude/settings.json`: sin bloque `hooks`. De los 13
plugins instalados, los únicos con `UserPromptSubmit` son security-guidance
(instalado **scope project → proyecto onyx**, no aplica a exo) y hookify (en
marketplace, NO instalado). Sin competidor por el turno en las sesiones de exo.

### C5 — "índice por detrás; --refresca ~1,5 s, inaceptable" → **CAE en su caracterización de coste**

Cierto que el índice va por detrás (Stop, detached) y que un `/documenta` de la
sesión en curso no se ve hasta la siguiente. Pero el coste está mal atribuido.
Medido sobre copia de KB + índice:

```
exo index, sin cambios:            0,00 s   (stat-pass, sin ONNX)
exo index, 1 nota editada:         0,98 s   (standalone: paga ONNX entero)
recall hybrid --refresca, sin cambios:   0,92 s   (vs 0,93 sin --refresca)
recall hybrid --refresca, 1 nota editada: 1,01 s
```

El 1,5 s de la cabecera de `exo-index.sh` cuenta la carga ONNX de un proceso
standalone. **Dentro de un `recall` hybrid el runtime ya está cargado: el
refresco añade ~0–80 ms.** Obliga a algo en el diseño, sí: a llevar
`--refresca` de serie (con guard de DB existente — ver H6), que cierra la
ventana intra-sesión casi gratis. Para un hook FTS-only habría sido un
problema; para hybrid es un regalo.

### C6 — "0.40 no es umbral de precisión de inyección" → **SE SOSTIENE**

Verificado en `reports/m2-07-impl-report.md` §3.3: la selección pre-registrada
optimiza **max hit@5** (49/55) y elige 0.40 como *el mayor threshold que
sostiene ese máximo* (0.45 lo baja a 46/55). Objetivo: que la buena esté entre
las 5. Precision@3 no se midió nunca. Matiz honesto: al ser "el corte más
agresivo que no pierde recall", 0.40 es lo más parecido a precisión que dio
aquel sweep — pero mi C3 demuestra que a efectos de inyección no corta nada
(30/30 con 3 hits llenos, rangos solapados). Implicación para D6: **no existe
un umbral de precisión disponible sin recalibrar**, y §0 prohíbe recalibrar.
La tolerancia a falsos positivos hay que comprarla en el formato y el
presupuesto, no en el corte.

---

## 2. Los tres datos pedidos

### Dato 1 — Contrato real de `UserPromptSubmit` (docs oficiales, hoy)

Fuente: code.claude.com/docs/en/hooks (redirect actual de docs.claude.com).

- **Inyección**: dos vías — stdout de texto plano se añade como contexto
  (excepción explícita para `UserPromptSubmit`/`SessionStart`) y
  `hookSpecificOutput.additionalContext` en JSON. El patrón JSON de
  `exo-recall.sh` sirve tal cual cambiando `hookEventName`.
- **Exit codes**: exit 0 = ok (stdout→contexto). **Exit 2 = bloquea el
  procesamiento Y BORRA el prompt** (stderr se muestra al usuario como razón).
  Otros ≠0 = aviso no bloqueante, el prompt sigue. Ver H4: en este evento un
  exit 2 accidental se come el prompt de Paul — hazard de primera clase.
- **Timeout**: default 30 s para hooks `command` en UserPromptSubmit (bajado
  del default general de 600 s), configurable por hook.
- **Orden**: el hook corre *antes* de que el modelo procese; el bloque llega
  junto al prompt en el mismo turno.
- **No documentado**: si dispara en turnos user no tecleados por el humano
  (teammate-messages, task-notifications). El diseño debe cubrirlo por gate
  (H5), no por fe.

### Dato 2 — Coste real de arreglar `prepara_query`

Medido (ver C2): el cambio a OR es trivial en líneas y **caro en consecuencias**
— FTS-only mejora 23→39/56 pero el hybrid sellado degrada 47→44/56 en
simulación exacta contra el índice vivo. No hay arreglo compartido: haría falta
bifurcar el camino léxico (flag nuevo o función paralela) y revalidar las 56
queries en ambos modos. Eso es un mini-M2-07. Con hybrid como motor del hook,
**no hace falta tocar `prepara_query` en absoluto**: cero riesgo para
`exo search`.

(Nota de honestidad metodológica: mis números absolutos difieren de los
sellados —47/56 vs 49/55— porque el índice vivo tiene 145 notas, no el snapshot
del eval; la comparación AND↔OR es pareada sobre el mismo índice y eso es lo
que soporta la conclusión.)

### Dato 3 — Distribución real de los prompts de Paul

272 prompts humanos reales desde el 01-ago (55 jsonl; filtrados
teammate-messages, task-notifications, comandos, sidechains y meta):

- Longitud: p25=23, **p50=53**, p75=153 chars.
- **Triviales** (ack-regex o ≤12 chars): 44 (**16%**). ≤25 chars: 27%.
- Sustantivos >80 chars: ~39%.
- Vocabulario: cuando el prompt nombra un tema, usa el literal de la KB
  (kbx, M6-06, playlist, sync_library, trinquete…) y hybrid acierta con
  claridad (pguerrero-music 3/3 relevantes, exo/kbx bien). El resto de
  prompts medios son **directivas de sesión** ("lanza auditor que revise el
  plan", "dale con F1") sin señal KB: hybrid devuelve ruido plausible a
  0,45–0,60. Este dato es el que decide D1 y D3.

---

## 3. Adjudicaciones (D1–D8)

### D1 — Qué inyecta: **(a) punteros + snippet. FIRMADO.**

Tres hits máx, `ruta absoluta — título · snippet` (el snippet es el primer
trozo de la nota, ~190 chars — identidad de la nota, no el trozo que matcheó;
decisión ya declarada en `recall.rs`). ~970 B medidos con 3 hits.

Por qué no (b) contenido: exige una precisión que **está medida y no existe**
(C3/C6: sin abstención, rangos solapados). 2 KB de la nota equivocada en una
fracción alta de turnos es el modo de fallo que la restricción "ruido en el
sitio más caro" prohíbe. Por qué no (c) híbrido por confianza: requiere dos
umbrales que los datos demuestran incalibrables (los scores no separan) y §0
retiró la maquinaria para calibrarlos.

**Respuesta a la objeción de adherencia** (mejor que "esta vez sí lo leerá"):
la decisión que mi D2 del 18-ago eliminó era *formular una búsqueda desde
cero* — acción que el modelo no ejecutó ni 1 vez en 14 días aun con doctrina a
favor. Leer una ruta absoluta ya puesta delante, con su título y su primera
línea, es una clase de acción distinta: un `Read` de coste trivial que el
modelo hace decenas de veces por sesión. Y en el caso frecuente (¿qué es X?
¿dónde estaba Y?) **el snippet ya ES el recall**: título + arranque de la nota
llegan empujados, sin decisión ninguna. El push existe; lo que queda opcional
es la profundización. Trade-off aceptado: habrá casos donde la nota entera era
necesaria y el modelo no la abra — se paga a cambio de que ningún falso
positivo cueste más de ~130 B.

### D2 — Motor: **hybrid siempre (tras el gate), con `--refresca`, vía `exo recall --query`. FIRMADO.**

Con C1/C2 en la mano no hay alternativa viva: FTS-AND no encuentra nada
(C2), FTS-OR exige bifurcar el engine y revalidar (dato 2), el escalonado está
invertido de fábrica (H1), y el proceso residente carga con la prohibición de
la restricción 6 (y abajo explico por qué no la levanto). `exo recall --query`
existe desde M2-08 exactamente para este consumidor: aplica los sellados
(`BONUS_SELLADO`, `ESCALA_FTS_SELLADA`), rinde punteros+snippet en texto plano
y resuelve rutas absolutas. El hook debe pasar **`--min-similitud 0.40`
explícito** (sin el flag, `recall` cae al 0.35 de la config RO — el sellado de
M2-07 viaja por flag hasta M5a, doctrina D-f3).

`--refresca`: de serie, **solo si la DB ya existe** (guard estilo
`exo-recall.sh`); si falta, abstención con log — jamás disparar un bootstrap de
minutos bajo un timeout de 30 s con un indexer aún no transaccional (H6).

Trade-off declarado, sin suavizar: **~0,95 s añadidos a cada turno
sustantivo** (~60–70% de los turnos tras el gate). Es real y es el precio de
la única búsqueda que funciona con prompts naturales. Lo acepto porque (i) el
turno de un agente dura segundos-a-minutos y el hook corre antes del primer
token, no en medio; (ii) la propiedad protegida es "no bloquea ni rompe", y
esto ni bloquea (timeout 30 s, exit 0) ni rompe; (iii) la única salida técnica
real al segundo (proceso residente con el modelo precargado) es un subsistema
nuevo con lifecycle, salud y modos de fallo propios — exactamente lo que §0 y
la spec E1 ("indexa al invocar, sin daemon") cortaron. Si tras semanas de uso
el segundo duele, ese dolor concreto es la reapertura legítima; nada del
diseño lo impide.

### D3 — Gate de disparo: **léxico, en el hook, tres reglas. FIRMADO.**

1. **Skip si el prompt empieza por `<`** (teammate-message, task-notification,
   command-wrappers): turnos no humanos, potencialmente enormes; ni búsqueda ni
   ruido (H5).
2. **Skip si <4 tokens o <25 bytes.** Cubre el 27% de los turnos (medido);
   "dale al plan", "haz push", "listo" no pagan nada.
3. **Skip si matchea una ack-regex corta y cerrada** (sí/ok/vale/dale/
   commitea…/push…/mergea…): el 16% trivial medido.

Por qué no un gate por búsqueda (la opción elegante sobre el papel): FTS-AND
como "¿hay algo en la KB?" está **invertido** — H1: 75% de acks disparan, 84%
de sustantivos abstienen. Medido, no opinado.

Contra el modo de fallo caro (gate que nunca deja pasar): los umbrales son
deliberadamente bajos (4 tokens / 25 bytes no filtran ninguna pregunta real) y
el evento `emitted` de D7 hace la tasa de disparo greppable — la ausencia
tiene nombre y tamaño, no es un cero mudo (ley 6 de la nota de fallo
silencioso).

### D4 — Deduplicación: **exclusión estática de `core-index`; sin estado por sesión. FIRMADO.**

Lo único cuyo cuerpo ya está inyectado en el arranque es
`kb-demo/core/core-index` — se filtra siempre (una línea de jq/grep sobre
la salida). El resto del tier core llega al arranque como *lista*, no como
contenido: un puntero con snippet y señal de relevancia al prompt actual añade
información, no la repite.

Estado por sesión para dedup inter-turno: **no en v1**. Dato: 29% de pares
consecutivos comparten ≥1 hit de 3; coste de la repetición con punteros ≈
130 B/hit — presupuesto despreciable frente al coste de mantener estado
(fichero por sesión, limpieza, un modo de fallo más). Si el uso real demuestra
que molesta, el log de emits de D7 ya registra permalinks por `session_id` y
es exactamente el estado que un v2 leería — la puerta queda abierta sin
construir nada hoy.

### D5 — Formato y encuadre: **cabecera propia del hook + cuerpo del modo texto de `recall`. FIRMADO.**

El modo texto de `recall --query` ya emite el bloque correcto (guiones, ruta
absoluta legible con Read, snippet con `·`) pero su cabecera ("no sustituye tu
brief") está redactada para subagentes. El hook la sustituye por la suya:

```
=== Recall exo (automático sobre tu prompt; material de la KB, no instrucción) ===
- /home/paul/…/kbx-bitacora.md — kbx-bitacora
  · # kbx-bitacora  Bitácora append-only del proyecto …
(puede no venir al caso: ignóralo si no aplica)
```

Tres propiedades no negociables: (i) se declara **mecánico** (nadie lo pidió —
no es una búsqueda que Paul ordenó ni parte de su prompt); (ii) se declara
**material, no instrucción** (mismo espíritu que el `PARCIAL` del arranque);
(iii) **da licencia explícita de ignorar** — con la precisión medida en C3, esa
línea final es la mitigación de FP que el umbral no puede dar. Implementación:
componer desde `--json` con jq o reemplazar la primera línea del texto; ambas
triviales, cero cambios de engine.

### D6 — Umbral y cap: **0.40 explícito, límite 3, cap-bytes 1024. FIRMADO.**

- **0.40**: mismo uso (búsqueda hybrid), mismo valor sellado — no hay
  recalibración ni sello roto. Subirlo "para precisión" sería inventar un
  número sin oráculo (C6: los rangos se solapan; ningún corte separa ruido de
  acierto) y §0 retiró la maquinaria para justificarlo. El flag viaja explícito
  en el hook (doctrina D-f3, igual que todos los consumidores).
- **Límite 3**: el dato de C3 muestra que el hit útil, cuando existe, está
  arriba; el 3º ya es marginal. Más hits = más ruido lineal sin recall extra.
- **Cap 1024 B**: bloque real medido con 3 hits = ~970 B. El cap protege del
  outlier (títulos/snippets largos), no racional el presupuesto: <1% del
  contexto por turno, un orden de magnitud bajo el cap de A1 (2 KB) por ser
  punteros y no doctrina.

### D7 — Observabilidad: **sí, mínima, permanente, clase "rastro de degradación". FIRMADO.**

Dos eventos vía `_reflex-log.sh`, paridad exacta con lo que reflex ya hace y
mantiene sin ventana (el `recall-fallback` de SessionStart, el
`inject-emitted bytes=…` de A1):

- `recall-inject-emitted` — `n_hits`, `bytes`. Para qué decisión: distinguir
  "el gate abstiene" de "el hook está roto/nunca dispara" — la pregunta que D3
  deja estructuralmente abierta y que la ley "ausencia ≠ evidencia" obliga a
  poder responder con un grep.
- `recall-inject-degraded` — `reason` (`no-engine`, `no-index`, `error`,
  `empty`, `timeout-guard`). Ley 1 de la nota de fallo silencioso: toda
  degradación deja señal o es un agujero.

Qué NO lleva, para no ser otro `retrieval-logger`: ni scores, ni queries, ni
tasa de acierto, ni ventana de medición, ni fecha de retirada — **se retira
cuando se retire el hook**, como sus dos hermanos. No es instrumentación de
eficacia (§0 la prohíbe); es visibilidad de degradación (§3.4 la autoriza:
"instrumentación del hook inyector").

### D8 — Alcance: **solo la sesión principal. Subagentes fuera, cerrado con nota. FIRMADO.**

Tres razones: (i) el prompt de un subagente lo compone el orquestador, que ya
recibió el recall de SU prompt y cuya doctrina (memoria v2) manda memory packet
de 3–5 permalinks en cada brief — el recall del punto de uso del subagente ya
tiene dueño; (ii) inyectar búsqueda automática en briefs compuestos con cuidado
es ruido sobre texto de precisión, el sitio donde menos se tolera; (iii) el
coste (~1 s × N subagentes × turnos) escala donde menos valor hay. Los
subagentes conservan el pull (`exo search`, allowlisted desde M6-05). Si un
ciclo real demuestra hambre de recall en subagentes, se reabre con ese dolor —
nota dejada aquí, scope cerrado.

---

## 4. Hallazgos nuevos (esto muerde y no estaba visto)

**H1 — FTS-AND está invertido como detector de "tema con notas".** Dispara en
el 75% de los prompts triviales (folding de acentos: `"sí"` → `si` matchea 114
de 145 notas; `ok` 33, `vale` 29) y abstiene en el 84% de los sustantivos.
Cualquier diseño que use FTS como gate barato o primer escalón hace
exactamente lo contrario de lo que promete. Mata el escalonado por dato, no
por gusto.

**H2 — El arreglo "obvio" de `prepara_query` (OR) degrada el instrumento
sellado**: hybrid 47→44/56 en simulación pareada exacta. La mitigación FTS-only
firmada el 18-ago no solo está rota de fábrica (C2): no tiene arreglo barato.
Lo digo alto porque es mío: **reabro y anulo esa mitigación candidata**; el
mecanismo hook y el resto del D2 del 18-ago quedan como estaban.

**H3 — Hybrid a 0.40 nunca se abstiene sobre esta KB** (30/30 prompts con 3
hits ≥0,41, ruido y acierto en el mismo rango 0,41–0,63). La otra mitigación
sugerida en §3.2 ("umbral más alto que el de búsqueda") también queda
invalidada: no existe ese umbral. La defensa contra FP se muda al formato
(D1/D5) y al presupuesto (D6).

**H4 — En `UserPromptSubmit`, un exit 2 accidental BORRA el prompt de Paul.**
Es el único evento donde un bug del script destruye input del usuario. El hook
nuevo necesita la disciplina exit-0 *más dura* que reflex tiene, no la
estándar: nada de `set -e`, toda tubería con `|| true`, y `exit 0`
incondicional al final (patrón `exo-recall.sh`, endurecido).

**H5 — Los turnos user no humanos existen y son grandes.** Teammate-messages y
task-notifications llegan como turnos `user`; la doc oficial no dice si
UserPromptSubmit dispara en ellos. La regla 1 del gate (skip si empieza por
`<`) lo neutraliza por diseño en vez de apostar por el comportamiento no
documentado. Sin ella, cada mensaje de teammate de 4 KB pagaría un segundo y
metería punteros irrelevantes.

**H6 — `--refresca` es casi gratis dentro del proceso hybrid** (+0–80 ms; el
ONNX se comparte) y cierra la ventana intra-sesión de C5. Pero con DB ausente
hace bootstrap (minutos) bajo timeout de 30 s, sobre un indexer **aún no
transaccional** (el fix H1 del 18-ago sigue pendiente de ejecutarse en C9): un
kill a mitad puede dejar una nota fuera del índice para siempre. De ahí el
guard: `--refresca` solo con DB existente; sin DB, abstención logueada.

**H7 (menor) — La cabecera del modo texto de `recall` miente en main session**
("no sustituye tu brief" — aquí no hay brief). Se tapa desde el hook (D5), cero
cambio de engine; si algún día se toca `recall`, una cabecera parametrizable
sería más honesta.

---

## 5. Qué consideré y descarté, con la razón

- **Proceso residente / daemon con el modelo precargado.** Es la única salida
  técnica real al segundo por turno — y la descarto: lifecycle propio, salud
  propia, arranque/parada, un subsistema nuevo disfrazado de optimización;
  contra la postura E1 vigente y contra §0. Reversible con dolor real medido
  en uso (dejo dicho que ESTA es la palanca si el segundo duele).
- **Arreglar `prepara_query` (OR/stopwords) para un camino FTS del hook.**
  Descartado por dato (H2): degrada el sellado o exige bifurcar y revalidar —
  un mini-M2-07 que §0 no permite y que hybrid hace innecesario.
- **Escalonado FTS→hybrid.** Descartado por dato (H1): el escalón está
  invertido.
- **Contenido inyectado con umbral alto (opciones b/c de D1).** Descartado por
  dato (C3/C6): no existe umbral que separe; cada FP costaría 2 KB en vez de
  130 B.
- **Estado de dedup por sesión.** Descartado por YAGNI con dato: 29% de solape
  × ~130 B no paga un fichero de estado con limpieza; el emit-log ya es el
  estado si un v2 lo necesita.
- **Cache de embedding de query.** Solo sirve para prompts idénticos —
  inútil por construcción.
- **Umbral distinto de 0.40 para el hook.** Sin oráculo que lo justifique
  (C6), rompería el sello sin dato; la restricción 3 exige exactamente lo que
  no puedo dar, así que me quedo con el sellado.
- **Inyección diferida (computar en Stop, inyectar al turno siguiente).**
  Recall rancio respecto al prompt que de verdad se está respondiendo;
  complejidad de estado; contradice "en el punto de uso".
- **Cubrir subagentes ya en v1.** Descartado (D8): el punto de uso del
  subagente ya lo cubre el memory packet del brief; coste × N sin señal de
  hambre real.

## 6. Nota final sobre el criterio de cierre

*"Un prompt sobre un tema con notas trae sus notas sin que nadie lo pida."*
Con este diseño, medido hoy: los prompts que nombran un tema (el caso del
criterio) traen sus notas en top-3 con claridad (pguerrero-music, exo/kbx,
agent-solve-it en mi muestra). Los que no nombran nada traen ≤1 KB de punteros
ignorables con licencia explícita de ignorar. El criterio se cumple con el
diseño más pequeño que sobrevive a los datos.

---

## Apéndice A — Abstención por margen relativo: medido y descartado (repregunta del orquestador)

Pregunta: ¿existe un estadístico autoreferencial de la distribución de scores de
cada query (pico vs meseta) que separe prompts topicales de directivas sin
señal, sin calibración externa? Respuesta: **no. AUC de azar en los estadísticos
de margen, y la intuición pico/meseta sale refutada con nombre y apellido.**

Método: los mismos 30 prompts reales (misma seed, mismo orden), etiquetados a
mano T=topical (11) / D=directiva o sin señal KB (19); `exo search --type
hybrid --min-similitud 0.40 --limite 10` por prompt; estadísticos sobre el
top-10 de cada query (script `margen.py` en scratchpad). Separación medida como
AUC (prob. de que un T supere a un D) y mejor corte simple:

| estadístico | AUC | mejor corte (accuracy) | mediana T vs D |
|---|---|---|---|
| `top1 − top3` (margen) | **0.49** | 67% | 0.018 vs **0.018** (idénticas) |
| `top1 / top3` (ratio) | **0.48** | 67% | 1.034 vs 1.037 |
| `top1 − mediana(top10)` | 0.56 | 67% | 0.044 vs 0.031 |
| nº hits ≥ 0.55 | 0.51 | 63% | 0 vs 0 |
| nº hits ≥ 0.50 | 0.62 | 70% | 1 vs 0 |
| `top1` absoluto | 0.67 | 70% | 0.508 vs 0.476 |

Baseline trivial ("di siempre D"): 63%. El mejor estadístico (top1 absoluto,
que ni siquiera es un margen: es el umbral duro ya descartado en C6/H3) gana
**2 prompts de 30** sobre el baseline. Los márgenes puros están en moneda al
aire.

La refutación cualitativa es mejor que la tabla: **los dos picos más altos de
la muestra son directivas**. "dale con lo que queda de F3" (margen 0.166) y
"firma, /documenta y cierra" (0.129, con doctrina-agentes a 0.60) producen
picos más nítidos que cualquier topical; "1. Push de master…" (D) mete **10
hits ≥0.55**; y el prompt más plano de todos (margen 0.009) es un topical
legítimo ("quiero seguir evaluando…" → agent-solve-it, acierto). El caso
degenerado (queries con solo 2–3 hits sobre el umbral, margen artificialmente
enorme) reparte 1 T y 1 D — tampoco sirve.

Por qué pasa — el mecanismo, no solo el número: la intuición pico/meseta asume
que el ruido es isótropo (una query sin señal cae "lejos de todo" y ve una
meseta plana). En esta KB el ruido no es isótropo: es **material semánticamente
adyacente**. Las 145 notas son el trabajo de una sola persona, densamente
interconectado, y una directiva como "firma, /documenta y cierra" está
*genuinamente cerca* de doctrina-agentes (que documenta exactamente ese
workflow) sin que inyectarla aporte nada al turno. El vecino más cercano de una
directiva es tan pico como el de un topical; lo que cambia es la *utilidad*,
que no vive en la geometría del embedding.

Límites declarados: n=30, etiquetas mías (2–3 ambiguas), un solo corpus. No me
mueven la conclusión: con medianas de margen **idénticas** (0.018 vs 0.018) no
hay efecto que una muestra mayor vaya a rescatar, y certificar un gate
estadístico marginal exigiría exactamente la maquinaria de calibración que §0
retiró.

**Consecuencia**: D1/D6 quedan como estaban. La inyección determinista de ≤3
punteros/≤1 KB tras el gate léxico no es resignación: es la lectura correcta
del dato — en este corpus la relevancia-para-el-turno no es separable de la
cercanía semántica por ningún estadístico barato, así que el coste del FP se
acota por formato (130 B, licencia de ignorar) en vez de fingir que un umbral
lo filtra. H3 cierra con dato, no con encogimiento de hombros.

---

## Apéndice B — Objeción del orquestador a D3 (gate por longitud vs gate léxico): medida y ACEPTADA con matices

Objeción: el gate por longitud (<4 tokens o <25 bytes) tiene un falso negativo
estructural — mata los prompts cortos Y topicales ("M6-06", "el trinquete"),
que serían el mejor caso del retrieval. Cifra estimada por resta: ~11%.

### El reparto real de los ≤25 bytes / <4 tokens (dato 2 de la repregunta)

El gate por longitud salta **73 de 272** prompts. Etiquetados uno a uno:

- **~55 son ack/decisión puro** sin ningún objeto ("di OK"×6, "dale"×4, "sí,
  borra", "perfe, continua", "como va?"…).
- **~12 son directivas cortas de sesión** con objeto session-local ("pushea
  los dos repos", "mergea a master", "borra las dos ramas", "arregla el
  README") — cero señal KB.
- **~6–8 tienen señal KB real**: "dale con el cutover", "tira con el modo
  mudo", "ok, consolida hecho", "sella la KB y pushea", "dale, /documenta",
  y 3× "implementa docs/superpowers/plans/<plan>.md" (2 tokens, 67 chars —
  la regla <4 tokens los mata pese a ser pura señal).

**La aritmética del 11% no se sostiene**: el 27% (≤25 chars) y el 16% (trivial)
se solapan casi por completo, y el residuo son las directivas git de sesión,
no topicales. El falso negativo real del gate por longitud es **~2–3% de los
prompts (~1 cada 3 días)**, no 11%.

### Gate léxico (B) sobre los 272

Implementado como propone la objeción: skip solo si TODOS los tokens
normalizados (lowercase, sin acentos) caen en una lista cerrada (~50 entradas:
función castellana + acks + verbos git/sesión sin objeto) o son numéricos.
Resultado (`gates.py` en scratchpad):

| | dispara | salta |
|---|---|---|
| Gate A (longitud) | 194 (71%) | 78 (29%) |
| Gate B (léxico) | 233 (86%) | 39 (14%) |

- **B rescata 40 prompts que A mataba.** De esos 40: ~8–10 con señal KB (los
  de arriba), **~30 falsos positivos** ("lo veo bien", "escribe el documento",
  "solventa todo", "Di solo: hola", "es web :/"). Ratio FP:TP ≈ 3:1 entre los
  rescatados.
- **B solo pierde 1 que A dejaba pasar** ("sí, merge a master y push" — ack,
  bien perdido).
- Los 39 que B salta son todos ack/git puro, verificados uno a uno. Cero
  topicales silenciados.

### La premisa "mejor caso del retrieval", comprobada — y es falsa a medias

Corrí hybrid top-3 sobre los rescatados con señal:

- `"M6-06"` (literal a secas) → core-index + exo-bitacora + reflex-bitacora.
  **Perfecto.** `"commitea el post en pguerrero-me"` → 2/3 relevantes. Bien.
- `"dale con el cutover"` → fallo-silencioso 0,60 + perfil + music. **Ruido.**
  `"tira con el modo mudo"` → agent-develop + pragmatismo + pguerrero.me.
  **Ruido.** `"ok, consolida hecho"` → ai-news + backlog. **Ruido.**
  `"dale, /documenta"` → ruido.
- `"sella la KB y pushea"` → **0 hits ≥0.40 y `recall` sale con exit 1**
  ("recall vacío (modo consulta)"). Hallazgo operativo de regalo: la
  abstención natural de hybrid existe (rara) y el CLI la señaliza como error —
  el hook debe mapear ese exit 1 a "sin bloque + log `empty`", no a
  degradación (ya estaba en D7; esto lo confirma con caso real).

El patrón: **el literal-a-secas recupera de lujo; directiva+token corto
recupera mal** — el relleno domina el embedding de una frase de 4 palabras y
el AND léxico muere. Así que el valor medido HOY del rescate de B es ~2–3
prompts útiles de 40, no 8–10.

### Adjudicación revisada de D3: **firmo el gate léxico (B). El de longitud se retira.**

No por el hit-rate medido (que es pobre en ambos gates para cortos mixtos),
sino por dos razones que pesan más:

1. **Polaridad de fallo.** El gate léxico falla hacia ruido visible (token
   desconocido ⇒ dispara); el de longitud falla hacia silencio (corto ⇒ calla).
   Con el ruido determinista ya aceptado por Paul, el FP está pre-pagado
   (≤1 KB + ~1 s) y el FN no tiene precio conocido — es la ley "ausencia ≠
   evidencia" aplicada al gate. Fail-open hacia ruido es la polaridad correcta
   de esta casa.
2. **El caso que A mata es el arquetipo del criterio de cierre.** "M6-06" a
   secas es EXACTAMENTE "un prompt sobre un tema con notas" — y es el estilo
   de prompt que el propio hook va a incentivar cuando exista. Hoy casi no
   aparece en el histórico (0 casos puros en 272; Paul siempre envuelve en
   directiva) precisamente porque hoy no sirve de nada. Un gate que castiga el
   patrón de uso que la feature invita a crear nace obsoleto.

Reglas finales del gate (sustituyen a las de D3 del verdict):

1. Skip si empieza por `<` (turnos no humanos).
2. Skip si empieza por `/` o `!` (comandos al harness, no prompts).
3. Skip si TODOS los tokens normalizados están en la lista cerrada
   (stopwords función + acks + verbos git/sesión) o son numéricos.
4. Si queda un solo token con contenido ⇒ dispara. La ack-regex del D3
   original queda subsumida por la regla 3.

Trade-off explícito, como pide el orquestador: **tasa de disparo 71%→86%**
(+~1,8 turnos/día pagando ~1 s y hasta 1 KB de ruido determinista, del que
solo ~1 de cada 13 rescatados trae señal útil hoy), a cambio de que ningún
prompt topical — presente o futuro — muera en el gate. La lista es el nuevo
punto de mantenimiento (riesgo asumido: ~50 entradas, cerrada, en el hook;
un token fuera de lista solo puede causar ruido, nunca silencio).
