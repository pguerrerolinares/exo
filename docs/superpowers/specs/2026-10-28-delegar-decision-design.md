# Gate «delegar-decision»: el padre no cierra pidiendo visto bueno sin delegar

**Fecha:** 2026-10-28 · **Estado:** diseño decidido por un consultor (Fable) sobre el etiquetado del dueño; cortes de decisión firmados por el dueño antes de etiquetar; pendiente de review de la spec escrita
**Origen:** doctrina [[El padre integra, no implementa — despacho de subagentes]]; veredicto de uso real (`evals/uso-real/verdict.md`, rama `eval/uso-real-etapa1`, commit 465ae33); #183, #193

## 1. Qué se quiere

El dueño corrige a mano al agente padre por no delegar: **0,99 correcciones de
delegación por sesión** (IC 0,41–1,93; extrapolación de 7/64 del estrato al
azar sobre 898 mensajes humanos, `verdict.md`:11,23,24). Quiere un mecanismo que
lo haga sin él.

Dos variantes en las 8 correcciones etiquetadas (`~/.cache/exo-uso-real/relectura-clave.json`):

- **A — delegar trabajo** (4): «hazlos con subagentes», «que lo haga un subagente».
- **B — delegar decisiones** (4): «lanza consultor que lo decida».

Restricción medida: los avisos de texto no funcionan. El reflejo search-first
(#183) se citó 47 veces y se ignoró las 47. Funcionan los mecanismos sobre
hechos: bloquear o inyectar en el momento de la acción.

## 2. Evidencia

Los scripts y los datos están en el scratchpad de la sesión 2026-10-28: `backtest-b/`, `backtest-a/`, `turnos-8/`, `etiquetado/`. Para pasar a test hay que moverlos al repo (§7).

| prueba | resultado |
|---|---|
| B, «pregunta con ≥2 opciones» en el último párrafo | recall 2/4 → cumple la condición de invalidación pre-declarada (<3/4) |
| B, misma regla sobre el mensaje entero | recall 4/4, 5,17 disparos/sesión |
| A, volumen de Read/Bash/Edit sin Agent | no separa: el turno previo tiene 0 Read y 0 Edit; 3/4 exige ~25 disparos/sesión |
| Lectura de los 8 turnos | 7/8 cierran con propuesta o recomendación + pregunta de visto bueno, sin `Agent` en el turno. A y B son el mismo patrón |
| Detector P2∨P3 (`turnos-8/patrones.py`) | recall 7/8 in-sample, 365 disparos en 99 sesiones (3,7/sesión) |
| Etiquetado del dueño, 30 disparos al azar (semilla 20261028, sellados en `muestra.json`) | 13 consultor + 5 delegar = **18/30 = 60 %** (CP 41–77 %); 12 «?», 0 «mía» |
| Los «?» según las notas del dueño | 6 push/PR/merge/deploy/`/document` (reserva ya firmada), 5 cierres de sesión, 1 ruido |
| Con exclusión regex de reserva | 17/22 = **77 %** (CP 55–92 %, LOO 75 %); el 82 % (18/22) no es reproducible con los términos de §4.2: el ámbito estaba mal transcrito y solo daba 18/22 tratando un ítem como mención; la cifra sellada pasa a 17/22 (R12, firmado por Paul 2026-10-28) |
| Recall fuera de muestra (158 órdenes «lanza consultor/subagente») | 27 % (43/158); 40 % sobre las 108 sin Agent en el turno |

## 3. Enfoques

1. **Solo A**, umbral de volumen. Descartado: el backtest no separa (§2).
2. **Solo B**, con tres variantes:
   1. Gate con bloqueo y etiqueta de escape `[reserva]`. **Elegido.**
   2. Sin hook, cambiando el default del texto de las propuestas. Descartado: es texto, y el texto ya falló (#183).
   3. Medir primero la precisión. Se hizo, con el etiquetado de §2.
3. **A y B a la vez**, con dos mecanismos. Descartado: A no tiene disparador (§2), y los 8 turnos muestran que A y B comparten la misma forma de cierre.

Cortes firmados por el dueño antes de etiquetar: precisión ≥ 50 % → gate con
bloqueo; ≤ 20 % → sin hook; entre medias → solo registrar durante ~30
sesiones. Con 60 % en bruto y 77 % con exclusión, toca **gate con bloqueo**.
La decisión se aplicó sobre la estimación puntual. La cota baja del CP en
bruto (41 %) queda por debajo del corte, y por eso el kill (§6) está activo
desde el primer día.

## 4. Mecanismo

### 4.1 Hook

Script nuevo: `plugins/exo/scripts/delegar-decision.sh`, registrado en
`hooks.json` → `Stop`. El precedente que sigue es `sdd-wake-guard.sh`. Los
hooks Stop corren en paralelo y su orden no está documentado
(https://code.claude.com/docs/en/hooks-guide#combine-results-from-multiple-hooks),
así que no se impone ninguno.

El detector vive en un helper Python, `plugins/exo/scripts/delegar-decision.py`,
que el hook bash invoca con `python3 -I`. *Decisión del orquestador.*

**Hechos verificados de la plataforma:**
- Stop solo corre en el agente principal; los subagentes disparan
  SubagentStop. La abstención por `agent_id` es defensiva.
- `decision: block` con `reason` hace que Claude continúe el turno.
- Hay un tope de 8 continuaciones por bloqueo (`CLAUDE_CODE_STOP_HOOK_BLOCK_CAP`).
  No nos afecta, porque el tope propio es 1 (§4.3).

**Abstención** (sale con 0 y sin salida):
- hay `agent_id` (es un subagente);
- `stop_hook_active` es true;
- faltan python3 o jq;
- el transcript no se puede leer;
- el sentinel (§4.3) ya está consumido para este mensaje humano.

**Hecho que dispara.** El segmento empieza en el último input, sea humano o un
input no humano que abre turno (hand-back de subagente con `isMeta` u
`origin.kind`, o una task-notification). La segmentación es la de `turnos()`
en `backtest-b/backtest.py`. El helper la reimplementa con la lógica de
`es_mensaje_humano` de `evals/uso-real/transcripts.py`, sin importarlo, y lleva
un test de paridad contra el original (§7). *Decisión del orquestador.* El hook
dispara si se cumplen las tres cosas:

1. en el segmento no hay ningún `tool_use` de `Agent` ni de `Task`;
2. el último bloque `text` del segmento casa **P2 o P3**, la variante que se midió:
   - `tail` = sus últimos 700 caracteres;
   - **P2**: `PERM.search(tail)`;
   - **P3**: `PROP.search(último texto) and ("?" in tail or ask)`, donde `ask`
     indica que el segmento termina en `AskUserQuestion`. Es la misma
     expresión de `patrones.py`, por paridad;
   - `PERM` y `PROP` son literales de `turnos-8/patrones.py`:7-8;
3. el último texto no casa la exclusión de reserva (§4.2) ni **empieza** por la
   etiqueta `[reserva: …]`. Si la etiqueta aparece en otra posición, no cuenta.
   *Decisión del orquestador.*

**No se amplían** PERM ni PROP sin medirlo antes. Los cierres que casan P1
(cualquier `?`) y no P2∨P3 van solo a log, para poder etiquetarlos después.

**Fuente del texto:** el último bloque `text` del segmento, leído de
`transcript_path` (JSONL), por paridad con el backtest. El campo
`last_assistant_message` está documentado
(https://code.claude.com/docs/en/hooks#stop-input), pero no se usa para el
detector. *Decisión del orquestador*: revierte la del redactor.

**No se excluye** el caso «ya corrió un Agent desde el último humano». En la
muestra quitaría 4 positivos (9, 16, 18, 29) a cambio de 1 negativo.

**Acción:**
```json
{"decision": "block", "reason": "Cerraste con propuesta + petición de visto bueno sin delegar. Lanza ahora el consultor (skills/brainstorm/consultor-brief.md) o el ejecutor y cierra con su resultado; o, si la pregunta es de Paul, reescribe el cierre empezando por [reserva: externo|irreversible|re-scope|hecho-de-Paul|pedido]."}
```

**Telemetría:** `_reflex-log.sh`, con id `delegar-decision` y payload
`kind=disparo|p1_log|banda patron=P2|P3 tail=<200c>`. El campo `kind` evita que
los logs informativos contaminen la medición de FP. Es best-effort (`|| true`) y
nunca rompe el hook. *Decisión del orquestador.*

### 4.2 Reserva: lo que sí sube a Paul

Es la reserva ya firmada, no una nueva («Una regla que se cita y se ignora es
un comentario», §Régimen de gates delegado):

| etiqueta | cubre | evidencia |
|---|---|---|
| `externo` | push, publicar, PR o issue, merge, deploy, borrar, permisos, contactar a terceros, secretos | ítems 1, 2, 23, 30 |
| `irreversible` | schema, formatos, API pública, slugs y URLs | ítem 19 |
| `re-scope` | tocar algo sellado: pre-registro, umbral, criterio firmado | — |
| `hecho-de-Paul` | contexto que solo tiene él, **solo si es toda la pregunta** | ítem 11 mezclaba y Paul pidió consultor |
| `pedido` | el mensaje humano del segmento pide expresamente que se le pregunte | F2, §5 |

**No se reserva:**
- prioridad o «qué sigue» (4, 18, 29: Paul quiere que arranque);
- «¿cuál prefieres?» de diseño (pasa por consultor; Paul firma en bloque).

**Sin umbral de dinero.** El dueño trabaja con la suscripción de Claude. Cada
bloqueo y cada consultor que provoca consumen cuota de la suscripción. La
carga estimada es de 2,65 bloqueos por sesión (§4.3), y se vigila en el kill.

**Exclusión regex.** Todavía no existe como código: el consultor la calculó
sobre la marcha. Escribirla es la **tarea 1 del plan**. Se aplica a la última pregunta `¿…?` de `tail` (fallback: última línea) (R12, firmado por Paul 2026-10-28), sin
distinguir mayúsculas y con `\b`, y cubre push, PR/issue, merge, deploy, commit,
`/document` y borrar. La tarea tiene que reproducir **17/22** sobre los 30 ítems
etiquetados y **256 bloqueos** en las 99 sesiones (con el tope de §4.3). Si
no se reproduce dentro de ±10 %, se para y se vuelve al orquestador.
*Decisión del orquestador.* La calibración es in-sample (§8).

### 4.3 Tope: un bloqueo por mensaje humano

El sentinel vive en `~/.cache/exo/delegar-decision/<session_id>` y guarda el
`uuid` del último mensaje humano. Si ya bloqueó para ese uuid, se abstiene. Esto
cubre lo que `stop_hook_active` no cubre: los hand-backs posteriores reabren
segmento dentro del mismo mensaje humano. La escritura del sentinel es
best-effort: si falla, el hook se comporta como si no hubiera sentinel, y
`stop_hook_active` sigue evitando el bucle.

Carga sobre las 99 sesiones con exclusión y tope, según el cálculo del
consultor: **256 bloqueos (R12, firmado por Paul 2026-10-28) (2,59 por sesión, mediana 1, máximo 14)**. Está sin
reproducir; lo valida la tarea 1 (§4.2).

### 4.4 Banda no declarada (solo informativa)

Es el contador de cierres del padre sin `?`, sin `Agent` y sin acción
(ningún `tool_use` en el segmento). Ese es el camino por el que el padre
esquiva el gate: deja de preguntar sin delegar. Se loguea con el id
`delegar-decision:banda` y no bloquea. Su tasa base se calcula en el backtest
de §7, antes del despliegue.

### 4.5 Brainstorm: «consultor primero, Paul firma»

En `plugins/exo/skills/brainstorm/SKILL.md`, la viñeta «Enfoques» (:71-77)
manda «recomienda una con su porqué y pregunta cuál prefiere», que es
literalmente P3. Sin cambiarla, las dos reglas no tienen solución conjunta. La
reescritura:

- **Enfoques:** antes del mensaje de enfoques, despacha siempre un consultor con
  `consultor-brief.md` que elija. El mensaje a Paul lleva su veredicto, el
  trade-off de las demás vías y la condición de invalidación. Paul firma o
  corrige; no elige de un menú.
- **Se borra la viñeta «Si el usuario no tiene criterio…» (:78-81).** El padre no
  puede juzgar si Paul tiene criterio: en 17, 22 y 25 lo quiso aun teniéndolo.
  Su contenido pasa a la viñeta anterior.
- Las preguntas de intención («qué quieres lograr») siguen siendo de Paul: son
  el qué, no el cómo. Llevan la etiqueta `[reserva: hecho-de-Paul]` si casan el
  detector.

Por la regla de cutover, hay que actualizar también la doctrina que anuncia el
comportamiento viejo: cualquier mención en `consultor-brief.md` dentro del plan,
y la nota de la KB «El padre integra, no implementa» con `/document` al cerrar
(§10).

## 5. Intersección de reglas del gate: bandas sin salida

Salidas de un bloqueo: (a) lanzar Agent o Task, (b) etiqueta `[reserva]`, (c)
reescribir el cierre sin pregunta, (d) el segundo Stop con `stop_hook_active`.

- **F1 — cierre de sesión, sin salida honesta.** «¿Lo dejamos aquí?» no es
  reserva y un Agent no tiene sentido. El agente solo puede etiquetar mal o
  quitar la pregunta. En la muestra son 5 de 30, y no se pueden excluir porque
  Stop no sabe que Paul se va. **Resolución:** la salida legítima es (c), un
  cierre declarativo («Lo dejo aquí; dime si sigues»). El motivo del bloqueo no
  la menciona, para no abrir un escape general. Estos cierres caen en la banda
  no declarada (§4.4) y se cuentan. *Decisión del redactor.*
- **F2 — Paul pidió que le pregunten.** «Pregúntame antes de X» o «dame
  opciones» no cabía en ninguna etiqueta. **Resolución:** se añade
  `[reserva: pedido]`. *Decisión del redactor.*
- **F3 — subagente vivo en background.** Si el segmento contiene el `Agent`
  que lo lanzó, la condición 1 no se cumple y no hay bloqueo. Si el aviso llega
  como hand-back y el padre pregunta en ese segmento nuevo, sí hay bloqueo, y
  es correcto (positivos 9, 16, 18, 29). No hay estado imposible.
- **F4 — bloqueos simultáneos (riesgo abierto).** Los hooks Stop corren en
  paralelo. `delegar-decision` puede bloquear en el mismo Stop que
  `sdd-wake-guard.sh` o `document-remind.sh`. No está documentado cómo se
  combinan dos `decision: block`. Las peticiones son compatibles en principio
  (cron + Agent, `/document` + Agent o etiqueta), pero no está verificado.
  **El plan incluye una prueba manual** en la que se fuerzan los dos bloqueos y
  se observa qué recibe el agente.

## 6. Medición y kill

Ventana de **~30 sesiones** del padre con el hook activo. Se degrada a solo
log (sin bloqueo) si pasa **cualquiera** de estas:

1. `[reserva]` aparece en más del 50 % de los cierres posteriores a un bloqueo.
2. Paul re-etiqueta los primeros 30 bloqueos reales con la misma hoja y la precisión baja del 55 % (R12, firmado por Paul 2026-10-28).
   Informativo (no degrada solo): más del 10 % de los bloqueos con un término de reserva fuera de la pregunta indica que el ámbito de la exclusión está equivocado (R12, firmado por Paul 2026-10-28).
3. Las correcciones de delegación no bajan de **0,41 por sesión**, la cota baja del IC actual, con el mismo protocolo de `evals/uso-real/`.
4. La banda no declarada sube sobre su tasa base: el bloqueo cambió la redacción, no la conducta.

## 7. Testing

- **Unitarios** en `plugins/exo/scripts/test-delegar-decision.sh`, con el patrón
  de `test-sdd-wake-guard.sh` y transcripts sintéticos en `testdata/`. Casos:
  - las abstenciones: `agent_id`, `stop_hook_active`, sin jq, transcript ilegible, sentinel consumido;
  - P2 dispara; P3 dispara; P3 sin `?` en `tail` no dispara;
  - un `Agent` en el segmento no dispara; un `Agent` solo en un segmento anterior sí dispara;
  - cada etiqueta de reserva y la regex de exclusión no disparan;
  - el JSON del block es exacto;
  - un fallo del log no rompe el hook.
- **Regresión del detector:** el backtest sobre las 99 sesiones selladas, como
  script reproducible en `evals/uso-real/`. Fija los 256 bloqueos (R12, firmado por Paul 2026-10-28) y el recall
  7/8 sobre los 8 turnos conocidos. Lo corre a mano quien cambie PERM, PROP o
  la exclusión, porque los transcripts no están en el repo.
- **Paridad del detector:** `delegar-decision.py` y el backtest
  (`patrones.py` + `backtest.py`) tienen que dar la misma decisión sobre los 30
  ítems de la muestra y sobre los 8 turnos conocidos.
- **Paridad de segmentación:** la reimplementación del helper frente a
  `es_mensaje_humano` de `evals/uso-real/transcripts.py`, sobre los mismos
  transcripts.
- **Manual:** el bloqueo simultáneo con `document-remind.sh` (F4).

## 8. Caveats honestos

- El **77 % (17/22) es in-sample**: la regex de exclusión se diseñó mirando los mismos
  30 ítems. La cifra honesta está más cerca de la cota baja; el kill 2 la valida.
- El **recall fuera de muestra es del 27–40 %**. El gate cubre menos de la
  mitad de B. El objetivo realista es pasar de ~1 a ~0,6 correcciones por
  sesión, no a cero.
- **A no queda cubierto.** Los 5 «delegar» de la muestra son «pide permiso para
  seguir», no «lo hizo él». El caso de A en línea (el padre trabaja sin
  preguntar) no tiene disparador.
- **Sesgo de muestra:** el 69 % de los disparos son de bizkaia-now, y la hoja
  mostraba la respuesta real del dueño, lo que puede anclarle.
- **El plan B del consultor** (inyección en `UserPromptSubmit` si el kill salta)
  es texto, y por #183 se marca como **sospechoso**. Si se llega ahí, se diseña
  otro mecanismo y no se despliega ese por defecto.

## 9. Condición de invalidación

Si tras ~30 sesiones las órdenes «lanza consultor/subagente» no bajan de 0,41
por sesión, o el padre esquiva la regex (§6.4), el bloqueo cambió la redacción y
no la conducta. El mecanismo se degrada a solo log.

## 10. Al implementar

- Subir `version` en `plugins/exo/.claude-plugin/plugin.json` (hoy 1.25.0), o el gate falla.
- Actualizar `--help`, mensajes y la doctrina del repo que anuncian el brainstorm
  viejo (§4.5). La nota de la KB «El padre integra, no implementa» queda fuera
  del plan: se actualiza con `/document` al cerrar.
- Llevar `patrones.py` y el backtest del scratchpad al repo (`evals/uso-real/`) sin rutas personales.
