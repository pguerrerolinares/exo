# Brief — Consultor Fable: diseño de M6-06 (recall en el punto de uso)

## Rol

Eres el consultor Fable del régimen de gates delegado del proyecto exo. Paul ha
abierto el brainstorm de **M6-06**, el último item vivo de M6 y **único bloqueador
de M5b** (desinstalar basic-memory). El orquestador hizo un recon de una hora,
midió cosas que rompen la mitigación que TÚ MISMO firmaste el 2026-08-18, y en vez
de seguir preguntando a ciegas te manda a ti.

Tu deliverable es un **veredicto escrito con adjudicación FIRMADA por decisión**
(D1…D8), no un menú de opciones: elige, razona corto, deja el trade-off explícito.
Antes de adjudicar, **verifica por tu cuenta los 6 claims de §Claims** con medición
primaria propia — no te creas el recon, está hecho deprisa.

Escribe el veredicto en
`docs/superpowers/consultas/2026-08-22-m6-06/consultor-m6-06.md`.
**No toques código de producción, no commitees, no edites specs.** Sí puedes
escribir scripts de medición en tu scratchpad y correrlos.

Mismo listón que `consultas/2026-08-18-cierre-regimen/consultor-cierre.md` (tu
verdict anterior) y `consultas/2026-08-18-m6-04-kbx/consultor-m6-04.md`.

## Qué es M6-06 (ya firmado, no se re-adjudica)

Un hook `UserPromptSubmit` que busca el prompt de Paul en la KB e inyecta lo
relevante antes de que el modelo empiece a pensar. Transporte mecánico, cero
decisión del modelo. Criterio de cierre escrito en la spec: *"Un prompt sobre un
tema con notas trae sus notas sin que nadie lo pida"*.

El problema que ataca (tu propio D1/D2 del 18-ago): el agente **no consulta la
memoria lo suficiente** — 5 llamadas al MCP en 14 días, 3 lecturas. Diagnóstico
firmado: es fallo de **adherencia**, no de transporte, y cuando el modelo no hace
algo por su cuenta no se le pide más fuerte, **se le pone delante con un hook**.

## Contexto obligatorio (léelo, no lo asumas)

Repo exo: `/home/paul/Documentos/proyectos/exo` (branch `main`, limpio, M6-04
cerrado y cutover ejecutado el 2026-08-22).
KB viva: `/home/paul/Documentos/proyectos/kb-demo` (~138 notas markdown).
Índice del engine: `~/.exo/index.db`. Binario instalado: `~/.local/bin/exo`.
`sqlite3` CLI **no está instalado**; usa `python3 -c` con el módulo `sqlite3`.

- `docs/superpowers/specs/2026-08-18-cierre-en-regimen-design.md` §3.2 (el encargo
  entero de M6-06, con sus tres riesgos), §3.4 (por qué se retiró `retrieval-logger`
  y qué permiso deja abierto a este diseño), §3.3 (orden), §4 (criterio de cierre).
- `docs/superpowers/consultas/2026-08-18-cierre-regimen/consultor-cierre.md` §D2 —
  tu propia firma del mecanismo y los dos riesgos que dejaste "de primera clase".
- `engine/src/main.rs:180-235` — flags reales de `exo recall` (`--query`, `--limite`,
  `--cap-bytes`, `--min-similitud`, `--contenido`, `--nota`, `--refresca`, `--json`).
- `engine/src/buscador.rs:138-160` — `prepara_query` (la de C2) y `busca`.
- `engine/src/buscador.rs` — `busca_hybrid` completa (fusión, bonus, escala FTS).
- `engine/src/recall.rs:420-486` — `recall_consulta` y `resuelve_rutas_absolutas`.
- `engine/src/main.rs:14-30` — defaults SELLADOS de M2-07 y por qué 0.40 no está
  hardcodeado.
- `plugins/reflex/scripts/exo-recall.sh` (108 líneas) — el hook hermano de
  `SessionStart`: patrón de fallback, guard semántico, logging de degradación,
  seams por entorno. **Es el molde estético del hook nuevo.**
- `plugins/reflex/scripts/compose-inject.sh` y `subagent-inject.sh` — el otro
  precedente de inyección (SubagentStart, cap 2KB, perfiles por agent_type).
- `plugins/reflex/hooks/hooks.json` — hoy NO hay `UserPromptSubmit`.
- `plugins/reflex/scripts/exo-index.sh` — el indexer corre en `Stop`, detached.
- `plugins/reflex/scripts/_reflex-log.sh` — el logger que ya existe (relevante a D7).
- `engine/evals/` (o donde viva `eval.jsonl`, 56 queries) — el oráculo de retrieval
  que cualquier cambio a `prepara_query` tendría que no romper.
- Nota de la KB `[[Fallo silencioso — el instrumento que no grita]]` — las seis leyes
  que este hook puede violar con facilidad.

## Restricciones firmadas (NO adjudicables)

1. **El mecanismo es un hook `UserPromptSubmit`.** No re-adjudiques "hook sí/no", no
   resucites el MCP propio (caído, §3.1) ni la memoria nativa de Claude Code (OFF
   deliberado, `doctrina-agentes:22`).
2. **Régimen §0 vigente**: proyecto personal, cerrar ya, **sin métricas nuevas, sin
   ventanas de medición, sin gates de eficacia**. Matiz que TÚ dejaste abierto en
   §3.4: *"si el ciclo de M6-06 necesita observabilidad, la trae su diseño — y será
   instrumentación del hook inyector"*. Adjudica si hace falta y hasta dónde (D7).
3. **`--min-similitud 0.40` está sellado** desde M2-07 para el uso de búsqueda. Si
   propones otro valor para este consumidor, di explícitamente por qué eso no rompe
   el sello (uso distinto ≠ recalibración) y de dónde sale el número.
4. **YAGNI**: gana el diseño más pequeño que cumpla el criterio de cierre. El régimen
   §0 vino a cortar exactamente esto.
5. **Propiedad protegida**: el hook JAMÁS bloquea, retrasa de forma perceptible ni
   rompe un turno. Exit 0 siempre, degradación con rastro (patrón `exo-recall.sh`).
6. **Sin daemon** es la postura vigente del engine ("indexa al invocar, sin daemon",
   spec E1). Si adjudicas un proceso residente, cargas tú con demostrar que no es
   un subsistema nuevo disfrazado.

## Claims a verificar (recon de hoy, hecho deprisa; si uno cae, di qué se lleva por delante)

- **C1 — El coste de hybrid es ~1 s POR PROMPT, y FTS es gratis.** Medido en la
  máquina de Paul contra el índice vivo:
  `exo recall --db ~/.exo/index.db --query "…" --json` → **0,97 s** (user 0,57 + sys 0,55).
  `exo search --db ~/.exo/index.db --type fts --json "…"` → **0,005 s**.
  Confirma tu propio riesgo nº1 del 18-ago. Verifica con varias queries y en caliente.

- **C2 — La mitigación que firmaste (FTS-only) está ROTA DE FÁBRICA para prompts
  naturales.** `prepara_query` (`buscador.rs:146`) envuelve cada token en comillas y
  los une con **AND implícito de FTS5**. Medido:
  - `"M6-06"` → core-index + exo-bitácora, 1 ms. Perfecto.
  - `"trinquete kbx budget"` → 1 hit, 4 ms.
  - `"vamos con brainstorm de M6-06"` (**el prompt real de Paul de hoy**) → **0
    resultados**, 5 ms.
  Es decir: cualquier palabra de relleno mata el recall léxico entero. Si esto se
  confirma, "FTS-only" no es escribir un hook: es tocar el engine (query laxa / OR /
  stopwords) **y** revalidar contra `eval.jsonl` para no romper `exo search`.

- **C3 — Hay riesgo de REDUNDANCIA, no solo de ruido.** El top hit hybrid del prompt
  real de hoy fue `kb-demo/core/core-index`… que el hook de `SessionStart` **ya
  inyecta entero** en esa misma sesión. Inyectar lo ya inyectado gasta tokens en el
  sitio más caro para no aportar nada. Mide cuántos de los top-3 hybrid de prompts
  típicos de Paul caen en `tier: core` (ya inyectado) o repiten entre turnos.

- **C4 — Hoy no existe ningún hook `UserPromptSubmit` en reflex.** `hooks.json` tiene
  `PreToolUse` (3 scripts en cada Bash), `SessionStart`, `Stop` (2), `SubagentStart`.
  Verifica también si hay `UserPromptSubmit` en `~/.claude/settings.json` de Paul o en
  otros plugins instalados, que competiría por el mismo turno.

- **C5 — El índice va por detrás de la escritura dentro de la sesión.** `exo-index.sh`
  corre en `Stop`, detached. Un prompt sobre algo que se acaba de escribir con
  `/documenta` no lo encuentra hasta la sesión siguiente. Existe `--refresca`, pero
  cuesta ~1,5 s con una nota editada (cabecera de `exo-index.sh`) — inaceptable en un
  hook por turno. Confirma el coste y di si obliga a algo en el diseño.

- **C6 — El umbral 0.40 no es un umbral de precisión de inyección.** Salió de
  maximizar **hit@5** en el sweep de M2-07 (`main.rs:14-30`): optimiza *que la buena
  esté entre las 5*, no *que las 3 que inyecto sean buenas*. Son objetivos distintos
  (recall@5 vs precision@3). Verifica que la caracterización es correcta y di qué
  implica para D6.

## Decisiones a adjudicar (FIRMA cada una)

- **D1 — ¿Qué inyecta: punteros o contenido?**
  (a) **Punteros + snippet**: 3 hits máx, `permalink + título + 1 línea`, ~400 B; el
  modelo decide si lee. Barato, tolerante a falsos positivos. Contra: reintroduce una
  decisión del modelo, que es justo lo que tu D2 quiso eliminar.
  (b) **Contenido inyectado** (cap 2 KB, patrón A1/arranque): push puro, cero decisión.
  Contra: exige precisión alta ⇒ empuja a hybrid ⇒ ~1 s/turno; y cada FP es ruido en
  el sitio más caro.
  (c) **Híbrido por confianza** (score alto ⇒ contenido, medio ⇒ puntero, bajo ⇒
  abstención). Contra: dos umbrales que calibrar sin métricas (§0 las retiró).
  Adjudica, y si eliges (a), responde a la objeción de adherencia con algo mejor que
  "esta vez sí lo leerá".

- **D2 — Motor de búsqueda.** FTS (¿arreglando `prepara_query`? ¿cuánto cuesta y qué
  riesgo tiene para `exo search`?) · hybrid siempre (~1 s/turno) · escalonado
  (FTS y solo si falla, hybrid) · proceso residente. Con el dato de C1/C2 en la mano.

- **D3 — Gate de disparo / abstención barata.** Tu riesgo nº2. "dale", "sí",
  "commitea", "continúa" no deben pagar búsqueda ni meter ruido. ¿Qué criterio, y por
  qué ese? Ojo al modo de fallo caro: un gate que se abstiene demasiado convierte
  M6-06 en un hook que no dispara nunca — y eso es **fallo silencioso de manual**
  (ausencia ≠ evidencia).

- **D4 — Deduplicación.** Contra el `core-index` ya inyectado en arranque, y contra lo
  ya inyectado en turnos anteriores de la MISMA sesión (¿estado por sesión? ¿dónde
  vive? ¿lo vale?).

- **D5 — Formato y encuadre del bloque.** Cómo se marca para que el modelo lo lea como
  **material de la KB**, no como instrucción de Paul ni como resultado de una búsqueda
  que él pidió. Precedente a respetar: el `=== Recall exo (PARCIAL — no sustituye tu
  brief) ===` del arranque.

- **D6 — Umbral y cap.** Cuántos hits, qué corte, qué presupuesto de bytes por turno,
  con C6 en la mano.

- **D7 — Observabilidad.** ¿Lleva instrumentación (vía `_reflex-log.sh`) o no? §0 dice
  no a métricas; §3.4 te deja la puerta abierta para el hook. Si dices que sí, di
  **qué evento, para qué decisión futura, y cuándo se retira** — o será otro
  `retrieval-logger`.

- **D8 — Alcance.** ¿Solo la sesión principal, o también el prompt de los subagentes
  (que hoy reciben doctrina por `SubagentStart` pero no recall por su tarea)? Si es
  fuera de scope, dilo y ciérralo con nota.

## Cosas de las que quiero DATO, no opinión

1. **Contrato real del hook `UserPromptSubmit` en esta versión de Claude Code**:
   formato de salida que inyecta contexto (`hookSpecificOutput.additionalContext` vs
   stdout crudo), semántica de exit codes (¿un exit 2 bloquea el prompt?), timeout, y
   si el bloque inyectado lo ve el modelo antes o después del prompt de Paul.
   Verifícalo contra documentación oficial o el propio harness — **no de memoria**.
2. **Coste real de arreglar `prepara_query`** para tolerar prompts naturales, y si eso
   degrada las 56 queries de `eval.jsonl` (córrelas si el oráculo es ejecutable).
3. **Distribución real de los prompts de Paul**: si puedes muestrear transcripciones
   de sesiones en `~/.claude/`, dime qué fracción son triviales (D3), qué fracción
   nombran un tema con notas (D1/D2), y con qué vocabulario — literal de la KB o
   parafraseado. Ese dato decide D2 mejor que cualquier argumento.

## Formato del veredicto

Como el anterior: verificación de claims primero (con el dato y cómo lo mediste),
adjudicaciones firmadas después, hallazgos nuevos al final (la familia "esto va a
morder y nadie lo ha visto"), y una sección corta de **qué consideraste y
descartaste, con la razón**. Si algo de lo firmado el 18-ago ya no se sostiene con el
dato de hoy, dilo alto: reabrirlo ahora es barato, después no.
