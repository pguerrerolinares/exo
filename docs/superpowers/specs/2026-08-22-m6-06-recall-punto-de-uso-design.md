# M6-06 — Recall en el punto de uso (diseño)

**Estado**: diseño aprobado por Paul (2026-08-22). Cierra el último item vivo de
M6 y **desbloquea M5b** (desinstalar basic-memory).

**Procedencia**: el encargo viene de `2026-08-18-cierre-en-regimen-design.md` §3.2,
que fijó *que* entra y *por qué*, y dejó explícito que *"el diseño —qué se inyecta,
con qué formato, umbral, cap, cuándo se abstiene— merece su propio brainstorm"*.
Esto es ese brainstorm. Las adjudicaciones D1–D8, sus mediciones y sus dos
apéndices están en `consultas/2026-08-22-m6-06/consultor-m6-06.md`; el brief que
las encargó, en `consultas/2026-08-22-m6-06/brief.md`. Aquí va el diseño, no el
razonamiento que lo produjo.

---

## 0. Qué es, y qué NO se re-adjudica

Un hook `UserPromptSubmit` que busca el prompt de Paul en la KB e inyecta punteros
a lo relevante antes de que el modelo empiece a pensar. **Transporte mecánico**:
cero decisión del modelo sobre *si buscar*.

Criterio de cierre, heredado sin cambios de la spec del 18-ago:

> *Un prompt sobre un tema con notas trae sus notas sin que nadie lo pida.*

Cerrado antes de este documento y fuera de discusión aquí: el mecanismo es un hook
(no un MCP propio, caído en §3.1; no la memoria nativa de Claude Code, OFF
deliberado en `doctrina-agentes:22`); el régimen §0 sigue vigente (sin métricas
nuevas, sin ventanas, sin gates de eficacia); `--min-similitud 0.40` sigue sellado
desde M2-07.

**Superficie total**: un script nuevo + una entrada en `hooks.json`. **Cero cambios
en el engine.**

---

## 1. Lo que el dato cambió respecto a §3.2

La spec del 18-ago proponía dos mitigaciones. **Las dos caen**, medidas contra el
índice vivo (145 notas) y 272 prompts reales de Paul desde el 01-ago.

**Cae "FTS-only para este hook".** `prepara_query` (`buscador.rs:146`) une los
tokens con AND implícito, así que el prompt real *"vamos con brainstorm de M6-06"*
devuelve **0 resultados**: cualquier palabra de relleno mata el recall léxico
entero. Y el arreglo obvio no está disponible: pasar a OR sube FTS-only 23→39/56
pero **degrada el hybrid sellado 47→44/56** en simulación pareada sobre el mismo
índice. Salvar el hook rompería `exo search`, que es el producto. Con hybrid como
motor, `prepara_query` no se toca en absoluto.

Corolario medido (H1): FTS-AND **está invertido** como detector de "¿hay algo en la
KB?" — dispara en el 75% de los prompts triviales (`"sí"` → `si` matchea 114 de 145
notas por folding de acentos) y abstiene en el 84% de los sustantivos. Eso mata
también el escalonado FTS→hybrid, y lo mata por dato.

**Cae "umbral más alto que el de búsqueda".** No existe ese umbral. Hybrid a 0.40
**nunca se abstiene** sobre esta KB (30/30 prompts con 3 hits ≥0,41) y ruido y
acierto viven solapados en 0,41–0,63. Se buscó un criterio de abstención
autoreferencial que no exigiera calibrar nada — margen top1−top3 (AUC **0,49**,
medianas de topicales y directivas **idénticas**: 0,018 vs 0,018), ratio top1/top3
(0,48), top1 contra la mediana del top-10 (0,56), conteo de hits sobre cortes
altos (0,51–0,62). Ninguno separa.

El mecanismo de por qué no puede funcionar importa más que los números, porque
generaliza: **pico-contra-meseta asume ruido isótropo, y en esta KB el ruido es
material semánticamente adyacente.** 145 notas de un solo autor, densamente
interconectadas. *"firma, /documenta y cierra"* tiene uno de los picos más nítidos
de la muestra porque está genuinamente cerca de `doctrina-agentes` —que documenta
ese workflow exacto— sin que inyectarla aporte nada; y el prompt más plano de los
30 es un topical que acierta. **La utilidad-para-el-turno no vive en la geometría
del embedding, y ningún estadístico de esa geometría la recupera.** De paso
explica C6: el 0.40 no está mal calibrado para esto, es que mide otra cosa —
maximiza hit@5 (recall), no precision@3.

**Consecuencia de diseño, aceptada explícitamente por Paul antes de ver el
apéndice**: la defensa contra falsos positivos se muda del umbral al **formato y
al presupuesto**. Cuando el hook dispara, inyecta hasta 3 punteros haya o no señal.
El ruido es determinista y cuesta ≤1 KB.

---

## 2. El diseño

Un script, `plugins/reflex/scripts/recall-inject.sh`, registrado en
`plugins/reflex/hooks/hooks.json` bajo `UserPromptSubmit`. Cuatro etapas en orden,
cada una capaz de abortar hacia "no inyectar nada" sin romper el turno.

### 2.1 Gate — decidir si este prompt merece una búsqueda

Léxico, dentro del hook, **sin tocar el engine**. Sobre el `.prompt` del JSON de
entrada:

1. **Skip si empieza por `<`** — turnos `user` no tecleados por un humano
   (teammate-messages, task-notifications), que existen, son grandes, y sobre los
   que la doc oficial **no dice** si el evento dispara. Se neutraliza por diseño en
   vez de apostar por comportamiento no documentado.
2. **Skip si empieza por `/` o `!`** — comandos al harness, no prompts.
3. **Skip si TODOS los tokens normalizados** están en una **lista cerrada de 127
   entradas** (stopwords de función + acks + verbos de git/sesión) o son numéricos.
   Tokenización por whitespace. Normalizar es NFD + minúsculas + strip de acentos y
   **conservando `/`, `.` y `-`** (`re.sub(r'[^a-z0-9/.-]','',token)`): quitar toda
   la puntuación construiría un gate distinto del medido.
4. **Si queda un solo token con contenido ⇒ dispara.**

**La lista y la función de normalización no se reescriben: se traducen.** El
artefacto original que produjo los números está commiteado en
`consultas/2026-08-22-m6-06/gate-artefacto.py`, y es normativo. Todos los números de
esta sección son propiedades de *esa lista exacta* — el propio test 1 (§5) solo pasa
porque `pushea`, `dos` y `repos` están en ella. Una lista reescrita de memoria
heredaría los claims sin heredar lo que los hizo ciertos, y §0 retiró la maquinaria
para volver a medirlos.

El gate por longitud (<4 tokens / <25 bytes) que se consideró primero **se retira**.
Salta 73 de 272 turnos, de los cuales solo ~6-8 tienen señal KB — su falso negativo
real es ~2-3%, no una catástrofe. Se retira igualmente por dos razones que pesan
más que el hit-rate:

- **Polaridad de fallo.** El gate léxico falla hacia **ruido visible** (token
  desconocido ⇒ dispara); el de longitud falla hacia **silencio**. Con el FP ya
  pre-pagado (≤1 KB + ~1 s) y el FN sin precio conocido, fail-open hacia ruido es
  la polaridad correcta — es "ausencia ≠ evidencia" aplicada al gate.
- **El caso que mata es el arquetipo del criterio de cierre.** `"M6-06"` a secas es
  literalmente *"un prompt sobre un tema con notas"*, y tiene **0 casos puros en
  272** precisamente porque hoy escribirlo no sirve de nada. Un gate que castiga el
  patrón de uso que la feature viene a incentivar nace obsoleto.

Trade-off declarado: tasa de disparo **71% → 86%** (+~1,8 turnos/día pagando ~1 s y
≤1 KB de ruido, del que solo ~1 de cada 13 rescatados trae señal útil *hoy*), a
cambio de **FN topical = 0**. La lista cerrada es el nuevo punto de mantenimiento,
con modo de fallo benigno: un token fuera de lista causa ruido, nunca silencio. (El
86% se midió sin las reglas 1-2, que solo pueden bajarlo y no pueden silenciar
ningún topical.)

### 2.2 Búsqueda — hybrid, siempre, tras el gate

```
timeout 5 exo recall --db "$EXO_INDEX" --query="$PROMPT" \
       --min-similitud 0.40 --limite 4 --cap-bytes 4000 [--refresca] --json
```

**Los números de la invocación no son los del bloque, y la diferencia importa.**
`--limite 4` pide un hit de repuesto por si el filtro de `core-index` (§2.3) quita
uno. `--cap-bytes 4000` es el cap de **fetch**: solo evita traer un payload absurdo,
y no debe confundirse con el cap de **inyección** (1024), que aplica el hook al
componer. Medido: con un cap de fetch ajustado (1400) el engine truncaba su propia
respuesta en 3 de cada 10 queries **antes** de que el hook filtrara nada, y en una de
cada diez se comía el repuesto — racionando en silencio justo lo que §2.3 vino a
arreglar. Con 4000, cero truncados.

`--query=` con el igual pegado, y no `--query "$PROMPT"`: el prompt es texto
arbitrario del usuario y si empieza por guion, clap lo parsea como flag y sale con
exit 2 (medido). Los demás flags llevan valores que controlamos nosotros.

- **Si el engine responde `truncado: true`, se registra.** Significa que su propio
  cap recortó la respuesta y el repuesto puede haber desaparecido; sin ese rastro, el
  hook entregaría menos punteros sin que nadie pudiera saberlo.
- **`--min-similitud 0.40` explícito y no negociable**: sin el flag, `recall` cae al
  **0.35 de la config RO**. El sellado de M2-07 viaja por flag hasta M5a (doctrina
  D-f3), y omitirlo sería degradación silenciosa con forma válida.
- **`--refresca` de serie, pero solo si la DB ya existe.** Dentro del proceso hybrid
  cuesta +0–80 ms (el runtime ONNX se comparte) y cierra la ventana intra-sesión: el
  indexer corre en `Stop`, así que sin él un prompt sobre lo que `/documenta` acaba
  de escribir no lo encontraría hasta la sesión siguiente. **Con DB ausente**,
  `--refresca` dispararía un bootstrap de **minutos** bajo un timeout de 30 s: un
  kill a mitad de un turno no es forma de construir un índice. Sin DB: abstención
  logueada, nunca bootstrap. (El agravante original de esta regla —"el indexer no es
  transaccional"— **ya no aplica**: `dfb2893` metió transacción por nota y es
  ancestro de HEAD. La regla se mantiene por el timeout, que basta solo.)
- **Motor**: no hay alternativa viva. FTS-AND no encuentra nada (§1), FTS-OR exige
  bifurcar el engine y revalidar las 56 queries en dos modos, el escalonado está
  invertido de fábrica, y el proceso residente es un subsistema nuevo con lifecycle
  y salud propios — contra la postura E1 vigente ("indexa al invocar, sin daemon")
  y contra §0. **Si el segundo por turno duele tras semanas de uso, ESA es la
  palanca y ese dolor es la reapertura legítima**; nada de este diseño lo impide.

### 2.3 Filtro y composición del bloque

- **Exclusión estática de `kb-demo/core/core-index`**: es lo único cuyo *cuerpo*
  ya inyecta el arranque. El resto del tier core llega al arranque como *lista*, no
  como contenido, así que un puntero con snippet y señal de relevancia al prompt
  actual añade información en vez de repetirla.
- **Sin estado por sesión** para dedup inter-turno. El solape entre turnos
  consecutivos es del 29% y cuesta ~130 B por hit repetido: no paga un fichero de
  estado con su limpieza y su modo de fallo. El `emitted` de §2.5 ya registra
  permalinks por `session_id` — es exactamente el estado que leería un v2, y queda
  disponible sin construir nada hoy.
- **≤3 punteros**, cap **1024 B** para el bloque entero. **El 1024 es el único
  número libre de este diseño**: el presupuesto por hit se *deriva* de él,
  `hit ≤ (1024 − overhead de cabecera y footer) / 3` (~280 B), calculado en el
  script desde las constantes reales en vez de escrito a mano. Si un hit se pasa,
  se recorta su **snippet** a frontera de palabra con elipsis — nunca la ruta, que
  es lo único que el modelo necesita íntegro para abrir la nota.

  **Acoplamiento declarado**: hoy el recorte por hit **no llega a activarse nunca en
  producción**, porque el engine ya capa cada snippet a 200 B
  (`engine/src/recall.rs:88`, `SNIPPET_MAX_BYTES`) y el presupuesto derivado ronda los
  240. Es decir: el invariante lo sostiene hoy una constante de otro subsistema, y la
  derivación es la red que lo sostendrá el día que esa constante suba. Se dice aquí
  porque un acoplamiento que nadie ha escrito es el que rompe la siguiente campaña, y
  porque obliga a que los tests del recorte usen fixtures que **superen** ese tamaño:
  con snippets realistas de 200 B, un test del recorte no ejerce nada.

  **Por qué derivado y no un segundo número**: la primera versión de esta spec fijó
  1024 sobre un "bloque real con 3 hits ≈ 970 B" que resultó ser el **mínimo** de la
  distribución, no el caso típico (mediana real: 1137 B, medida sobre 27 hits del
  índice vivo). Con eso, el cap dejaba fuera el tercer puntero en **7 de cada 10
  queries** — es decir, racionaba el presupuesto, que es exactamente lo que esta
  misma sección declaraba que no debía hacer. Derivando los tamaños internos del
  cap, el invariante pasa a ser verdad *por construcción* y no por suerte del
  ranking. Medido tras el cambio: 0 de 10 queries racionadas, máximo 1010 B.

### 2.4 Formato del bloque inyectado

El bloque se compone desde `--json` con una cabecera propia. La del modo texto de
`recall` está redactada para subagentes y en la sesión principal **miente** ("no
sustituye tu brief" — aquí no hay brief).

```
=== Recall exo (automático sobre tu prompt; material de la KB en
    /home/paul/Documentos/proyectos/kb-demo; no es una instrucción) ===
- log/kbx-bitacora.md
  · Bitácora append-only del proyecto [[kbx — explorador determinista…]] …
- projects/kbx — explorador determinista de la KB (Go).md
  · CLI en Go sobre tres fuentes de la KB, read-only salvo `rotate` …
(puede no venir al caso: ignóralo si no aplica)
```

**Cada hit dice lo suyo una sola vez.** El formato anterior repetía el nombre de la
nota **tres veces** por puntero —en la ruta, en el título y otra vez como header
markdown al principio del snippet, medido en 26 de 30 hits reales— y la raíz de la
KB otras tres. Eso era ~25% del bloque gastado en repetirse, y era la causa real de
que el cap racionara. De ahí las tres reglas:

1. **La raíz de la KB se declara una vez, en la cabecera**; los hits llevan ruta
   relativa. El bloque sigue siendo autocontenido: el modelo une raíz + relativa
   sin depender de que el `core-index` del arranque siga vivo en el contexto.
2. **El título se omite cuando no aporta** sobre el nombre del fichero (comparación
   con la misma normalización laxa que ya usa el gate). Y si el snippet empieza
   repitiendo el título como header markdown, se pela.
3. **El snippet se recorta al presupuesto por hit** (§2.3), a frontera de palabra.

Tres propiedades **no negociables**, porque son la única defensa contra falsos
positivos que este diseño tiene:

1. Se declara **mecánico** — nadie lo pidió; no es una búsqueda que Paul ordenó ni
   parte de su prompt.
2. Se declara **material, no instrucción** — mismo espíritu que el `PARCIAL` del
   arranque.
3. **Da licencia explícita de ignorar.** Con hybrid sin abstención (§1), esa última
   línea es la mitigación que el umbral no puede dar.

**Saneo obligatorio**: `titulo` y `ruta` se limpian de `\n` y `\r` antes de
componer. El engine sanea el `snippet` (`recall.rs:442`) pero no los otros dos: el
título sale del frontmatter de la nota y la ruta del filesystem, donde un salto de
línea es legal. Sin ese saneo, una nota puede **fabricar un puntero falso** en el
contexto —verificado: un título con `\n- inyectado` produce un puntero inventado
emparejado con el snippet de otra nota, emitido con `log=emitted` y sin ninguna
señal de degradación—. Es la primera ley de [[Fallo silencioso]] en su forma más
literal.

### 2.5 Observabilidad — rastro de degradación, no medición

Dos eventos vía `_reflex-log.sh`, en paridad exacta con lo que reflex ya mantiene
sin ventana (`recall-fallback` de SessionStart, `inject-emitted` de A1):

- **`recall-inject-emitted`** — `n_hits`, `bytes`, permalinks. Para una decisión
  concreta: distinguir *"el gate se abstiene"* de *"el hook está roto y no dispara
  nunca"*. Es la pregunta que el gate deja estructuralmente abierta, y la ley
  "ausencia ≠ evidencia" obliga a poder responderla con un `grep`.
- **`recall-inject-degraded`** — `reason` ∈ {`no-engine`, `no-index`, `error`,
  `empty`, `timeout-guard`}.

Qué **no** lleva, para no ser otro `retrieval-logger`: ni scores, ni queries, ni
tasa de acierto, ni ventana de medición, ni fecha de retirada. **Se retira cuando se
retire el hook**, como sus dos hermanos. No es instrumentación de eficacia (§0 la
prohíbe); es visibilidad de degradación (§3.4 la autoriza explícitamente:
*"instrumentación del hook inyector"*).

---

## 3. Data flow

```
prompt de Paul
   │
   ▼
UserPromptSubmit ──> recall-inject.sh
   │
   ├─ (1) gate léxico ── skip ──────────────────────────> exit 0, sin bloque
   │                                                       (sin log: la abstención
   │                                                        normal no es evento)
   ├─ (2) guards: ¿binario? ¿DB? ── no ──> log degraded ─> exit 0, sin bloque
   │
   ├─ (3) exo recall --query --min-similitud 0.40 --limite 3 [--refresca]
   │        ├─ exit 0 con hits ─> filtrar core-index ─> componer cabecera propia
   │        │                                        └─> log emitted ─> stdout JSON
   │        ├─ exit 1 + stderr ~ "recall vacío" ────> log empty ─> exit 0, sin bloque
   │        ├─ exit 1 sin esa marca ───────────────> log error ─> exit 0, sin bloque
   │        └─ timeout (5 s) ─────────────> log timeout-guard ─> exit 0, sin bloque
   ▼
hookSpecificOutput.additionalContext  ──>  el modelo lo ve junto al prompt
```

---

## 4. Error handling y propiedades protegidas

**P1 — El hook JAMÁS destruye el prompt.** En `UserPromptSubmit` un **exit 2 borra
el prompt de Paul** (verificado en docs oficiales): es el único evento del harness
donde un bug del script destruye input del usuario. Exige la disciplina exit-0 *más
dura* que la de reflex, no la estándar: **nada de `set -e`**, toda tubería con
`|| true`, y `exit 0` incondicional al final. Cualquier ruta de fallo — binario
ausente, DB ausente, `jq` roto, engine que revienta, timeout — termina en exit 0 sin
bloque.

**P2 — La abstención natural no es degradación, pero el exit code NO la distingue.**
`exo recall --query` sale con **exit 1** cuando ningún hit supera el umbral (medido:
*"sella la KB y pushea"* → 0 hits ≥0.40). El problema es que el engine sale con
**exit 1 para cualquier error** (`main.rs:246`: solo el `Rechazo` de write usa 3), así
que la abstención es indistinguible por código de una DB corrupta, un ONNX roto o un
lock. Medido: `--db /nope/x.db` da exit 1 igual que la abstención.

El distinguidor está en **stderr**, y es estable:

- exit 1 **y** stderr contiene `recall vacío` ⇒ `reason=empty`. Abstención correcta.
- exit 1 **sin** esa marca ⇒ `reason=error`. Es un fallo.

Gatear solo por código sería un hook donde el engine roto loguea `empty` para
siempre: exactamente la degradación con forma válida que P3 jura impedir, y encima
mataría la única pregunta que §2.5 existe para responder — distinguir *"el gate se
abstiene"* de *"esto lleva un mes sin funcionar"*.

**P3 — Toda degradación deja rastro.** Ley 1 de [[Fallo silencioso]]: cada rama de
fallo emite su `degraded` con razón greppable. El precedente es directo — el
fallback silencioso del recall de arranque ya mordió una vez (F3.1).

**P4 — Timeout propio, no el del harness.** El evento trae un default de 30 s, pero
esperar 30 s por una búsqueda de ~1 s es un cuelgue con forma válida, y un fallo del
harness no deja rastro en el log del hook. El hook envuelve la llamada en un
`timeout` propio de **5 s** (5× el coste medido): al agotarse, `reason=timeout-guard`
y exit 0 sin bloque. Así el modo de fallo es suyo, es rápido y es greppable. El
único camino capaz de agotar incluso el timeout del harness es el bootstrap con DB
ausente, que P5 prohíbe.

**P5 — Nunca bootstrap desde el hook.** `--refresca` solo con DB existente (§2.2).

**P6 — Nada se escapa por stdout.** En `UserPromptSubmit`, el stdout plano de un
hook que sale con 0 **se inyecta como contexto** (es la excepción documentada del
evento, junto a `SessionStart`). Un `echo` de debug olvidado o un stderr mal
redirigido no ensucia un log: entra en el turno como si fuera material de la KB.
Todo lo que no sea el JSON final va a stderr o a `/dev/null`.

---

## 5. Testing

Siguiendo el patrón de `test-compose-inject.sh` / `test-subagent-inject.sh`, un
`test-recall-inject.sh` con seams por entorno (`EXO_BIN`, `EXO_INDEX`) para no tocar
la instalación real:

1. **Gate**: dispara con `"M6-06"`; calla con `"dale"`, `"sí, hazlo"`,
   `"pushea los dos repos"`, `"/compact"`, `"!ls"`, y con un prompt que empiece por
   `<`.
2. **P1, el test que importa**: con `EXO_BIN` apuntando a un binario que sale con 2,
   que revienta, y que escupe basura a stdout — **exit 0 en los tres casos**, y sin
   bloque. Es el test que protege el prompt de Paul.
   2b. **P4**: binario falso que duerme 30 s ⇒ el hook vuelve en ~5 s con
   `reason=timeout-guard` y exit 0. Testeable precisamente porque el timeout es
   suyo y no del harness.
3. **P2, las dos ramas**: binario falso con exit 1 **y** `recall vacío` en stderr ⇒
   `reason=empty`; exit 1 con cualquier otro stderr ⇒ `reason=error`. Sin bloque en
   ambos casos. Es el test que impide que un engine roto se disfrace de abstención.
4. **P5**: DB ausente ⇒ no aparece `--refresca` en la invocación, y sale
   `reason=no-index`.
5. **Flag sellado**: la invocación real contiene `--min-similitud 0.40`. Es un test
   de una línea contra una degradación silenciosa cara.
6. **Formato**: con salida sintética de 3 hits, el bloque lleva la cabecera propia
   (no la de subagentes), la línea de licencia de ignorar, ≤1024 B, y **no** contiene
   `core/core-index`.
7. **Salida y P6**: el stdout es JSON válido con
   `hookSpecificOutput.hookEventName == "UserPromptSubmit"` **y nada más** — ni una
   línea suelta antes o después. En este evento el stdout plano se inyecta como
   contexto, así que un `echo` perdido entra en el turno de Paul.

Y la verificación end-to-end en vivo, que es la que cierra el item: escribir `M6-06`
a secas en una sesión real y ver llegar sus notas.

---

## 6. Lo que este diseño acepta a cara descubierta

No son riesgos residuales: son costes conocidos, medidos y firmados.

- **~0,95 s añadidos a cada turno sustantivo** (~86% de los turnos tras el gate).
  Es el precio de la única búsqueda que funciona con prompts naturales. Se acepta
  porque el turno de un agente dura de segundos a minutos y el hook corre **antes**
  del primer token, no en medio.
- **Ruido determinista**: cuando dispara, inyecta hasta 3 punteros haya o no señal.
  Sin umbral que separe (§1), la defensa es formato + ≤1 KB. El riesgo real no es el
  coste en tokens sino **que Paul deje de mirarlos** y el hook se vuelva papel
  pintado; si eso pasa, el dolor es la señal de reapertura.
- **Una lista cerrada de 127 stopwords** como punto de mantenimiento nuevo.
- **La ruta deja de ser copy-pasteable de un vistazo**: el modelo une la raíz de la
  cabecera con la ruta relativa. Concatenación trivial y de fallo benigno (un `Read`
  errado y reintento), a cambio de 132 B por bloque.
- **El snippet baja a ~130–170 B en hits de ruta larga.** Suficiente para su única
  función, que es decidir **si** abrir la nota, no sustituir su lectura — y hasta
  ahora la mitad de esos bytes era el título repetido por tercera vez.
- **El hook pasa a ser escritor en ~86% de los turnos** (por `--refresca`). Con
  sesiones paralelas y el indexer de `Stop` habrá contención: el engine tiene
  `busy_timeout` de 5 s y journal `delete`, así que un lector puede bloquear al
  escritor. Peor caso: ese turno paga el timeout-guard o loguea `error` — visible,
  acotado, y nunca `empty` gracias a P2.
- **Prompts gigantes**: pegar un traceback de 8 KB cuesta ~1,4 s (el embedder trunca;
  30 KB da lo mismo) y devuelve punteros genéricos dentro del cap. Por encima de
  ~128 KB de argumento el exec falla con E2BIG ⇒ `reason=error`, exit 0. Medido y
  benigno: no se trunca la query en el hook.
- **La ventana no cubierta**: "afirmo algo a mitad de mi propio razonamiento, sin
  prompt de por medio". Ningún hook razonable la ataja; §3.2 ya la aceptaba.

---

## 7. Qué NO entra (scope cerrado con nota)

- **Subagentes.** Su punto de uso ya tiene dueño: el orquestador compone el brief y
  ya recibió el recall de *su* prompt, y la doctrina de memoria v2 manda memory
  packet en cada brief. Inyectar búsqueda automática sobre texto compuesto con
  cuidado es ruido en el sitio donde menos se tolera, y el coste escala ~1 s × N
  subagentes × turnos donde menos valor hay. Conservan el pull (`exo search`,
  allowlisted desde M6-05). Si un ciclo real demuestra hambre, se reabre con ese
  dolor.
- **Tocar `prepara_query`** (§1: degrada el sellado).
- **Estado de dedup por sesión** (§2.3: YAGNI con dato).
- **Proceso residente / daemon** (§2.2: la palanca si el segundo duele, no antes).
- **Cualquier umbral distinto de 0.40** (§1: no hay oráculo que lo justifique, y
  §0 retiró la maquinaria para construirlo).
- **Inyección diferida** (computar en `Stop`, inyectar al turno siguiente):
  contradice "en el punto de uso" y llega rancia al prompt que de verdad se responde.

---

## 8. Criterio de cierre y rollback

**Cierra cuando**: `M6-06` escrito a secas en una sesión real trae sus notas sin que
nadie lo pida, la suite de reflex está verde incluido `test-recall-inject.sh`, y el
log muestra `recall-inject-emitted` en turnos sustantivos y silencio en los acks.

**Rollback**: quitar la entrada `UserPromptSubmit` de `hooks.json` — un flag, como
todas las campañas de M6. Nada más se toca, porque nada más se ha tocado.

**Cola tras esto**: M6 cierra entero → **C10/M5a-02** (config propia: mata el
hardcode `projects["kb-demo"]` de `lib.rs:71`) → **C11/M5b** (Paul desinstala
basic-memory).

---

## Anexo — procedencia de los números

Todos los datos de este documento salen de medición primaria del consultor Fable
contra el índice vivo (`~/.exo/index.db`, 145 notas) y 272 prompts humanos reales de
Paul desde el 01-ago (55 `.jsonl`, filtrados teammate-messages,
task-notifications, comandos, sidechains y meta). Detalle, método y límites
declarados (n=30 en la muestra etiquetada a mano, etiquetas del propio consultor, un
solo corpus) en `consultas/2026-08-22-m6-06/consultor-m6-06.md` §1, §2 y apéndices
A y B.

Dos correcciones que este documento hereda y que conviene no volver a cometer: el
riesgo de redundancia con `core-index` que motivó parte del brief resultó anecdótico
(0 de 90 en top-3), y el falso negativo del gate por longitud, estimado por resta en
~11%, es en realidad ~2-3% — el 27% de prompts cortos y el 16% de triviales se
solapan casi enteros.
