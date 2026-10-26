# Medición en uso real: correcciones en decisiones ya tomadas

**Fecha:** 2026-10-26 · **Estado:** aprobada en brainstorming; definición de la unidad decidida por un consultor (Fable), pendiente de review del dueño
**Origen:** paso (a) del orden de exo (bitácora 2026-10-26, «balance de exo: aporta el canal de reglas, no el cajón»); #207, #193

## 1. Qué se quiere

El balance del 10-26 dejó un orden aceptado: a) medir en uso real; b) pasar a
gate las reglas que se siguen incumpliendo; c) re-cortar una vez las 8 notas
al CAP; d) E (el «conserje») solo si la medición lo justifica. Todo lo medido
hasta ahora es de laboratorio. Falta saber **cuántas veces corrige el dueño al
agente en cosas que ya estaban decididas**, en sus sesiones reales.

**Condición pre-registrada del orden:** si el dueño casi no corrige al agente
en cosas decididas ⇒ estabilizar exo y dedicar el tiempo a otro proyecto.
**Umbral sellado por el dueño: 0,4 correcciones por sesión** (por sesión, no
por 100 mensajes; decisión del dueño).

## 2. Evidencia de base

| dato | valor | fuente |
|---|---|---|
| Sesiones `entrypoint=cli` con mtime ≥ 2026-10-06 | 106 (bizkaia-now 65, exo 19, KB 14, resto ≤ 2) | recalculado por el orquestador |
| Sesiones `sdk-cli` (sondas y evals) en el mismo periodo | 20, excluidas | ídem |
| Mensajes humanos en esas 106 | 1 232 | filtro de §4.1 |
| Sesiones con reglas de proyecto en el `prompt_snapshot` | **14, todas de exo** | `attachment.type = prompt_snapshot` |
| Reglas de proyecto en exo hasta el 10-25 | 1 (`git -C`) | historia git de la KB |
| Siembra de 20 proyectos y reglas globales | commit de la KB del 2026-10-25 17:10 | ídem |
| Sesiones principales anteriores al 10-06 | ~47 | mtime |
| Retención de transcripts | `cleanupPeriodDays: 365` | settings del usuario |
| Tasa de correcciones (total, sin atribuir) | 0,2-0,5 por sesión, **a ojo** | consultor, 45 mensajes al azar + 80 candidatos de regex |
| `cd … && git` en Bash de bizkaia-now | 91/6 382 (1,4 %) en sesiones principales; 310/12 649 en subagentes | consultor |

**Consecuencia:** la medición real **no evalúa el canal de reglas**, porque
casi nunca estuvo puesto fuera de exo. Mide la tasa base de correcciones en
cosas decididas, casi toda sin canal. Igual decide el orden: tasa baja ⇒
estabilizar; tasa alta y atribuida a cosas fuera del canal ⇒ ampliar la
siembra o E.

## 3. Enfoques

1. **Dos etapas con juez** (consultor): juez Haiku sobre todos los mensajes,
   y después atribución con Sonnet sobre los positivos. ~16 USD.
2. **Regex y etiquetado manual**: ~130 candidatos, 0 USD y ~1 h del dueño.
   Pierde las correcciones sin marcador léxico (3-4 de 45 en la muestra) y
   deja también la atribución al dueño.
3. **El 1 con una puerta entre etapas** (elegido): la etapa 1 da una cota
   superior. Si queda claramente por debajo del umbral, se aplica la
   condición sin pagar la etapa 2.

**Unidad («decisión ya tomada»): B con guarda de fecha** (consultor). Es
texto que estaba en la KB (reglas duras, canon de proyecto, `core/`, perfil)
en un commit **anterior al primer timestamp de la sesión**. Se descartó:
- A (solo reglas duras): sin población; solo 14 sesiones expuestas, todas de
  exo.
- C entre sesiones: no hay fuente cerrada. «Dicho antes en esta sesión»
  entra como etiqueta acotada (§4.4).

La guarda corta la circularidad: el dueño corrige, `/document` lo escribe
esa noche y una lectura sin fecha lo contaría como decidido.

## 4. Diseño

### 4.1 Población
- Transcripts `~/.claude/projects/*/<sesion>.jsonl` (no los de
  `subagents/`), `entrypoint = cli`, **primer `timestamp` del fichero
  ≥ 2026-10-06**. El primer timestamp es también el ancla de la guarda de
  fecha. Las 2 sesiones con mtime posterior y arranque anterior pasan a la
  reserva.
- **Sesión (denominador)** = conversación con al menos 1 mensaje humano. Las
  continuaciones por compactación (primer mensaje «This session is being
  continued…», 22 casos) se unen a la sesión de la que vienen, si el jsonl
  permite enlazarlas (`parentUuid`/`leafUuid` o el `sessionId` citado). Si no
  se pueden enlazar, cuentan como sesiones aparte y el resultado lo declara.
  Las sesiones cortas (24 con ≤ 3 mensajes) **no se excluyen**. *Decisión del
  orquestador:* el dueño eligió la tasa por sesión, y unir las
  continuaciones evita inflar el denominador, que es el error que favorece
  estabilizar.
- **Repo de la sesión** = directorio de `~/.claude/projects/` (el `cwd` de
  arranque). Si el `cwd` cambia dentro de la sesión, no se reatribuye.
- **Mensaje humano:** `type = user`, sin `isMeta`, sin `toolUseResult`,
  `origin.kind = human` y `promptSource ∈ {typed, queued}` cuando los campos
  existen. Sin esos campos (formato viejo, 44 casos) decide el filtro por
  texto. El texto no puede estar vacío ni empezar por `<command-`,
  `<local-command`, `<task-notification>`, `<bash-stdout>`, `Another Claude
  session`, `[Request interrupted` o `This session is being continued`.
  - `suggestion_accepted` (271 casos) queda **fuera**: es una sugerencia del
    harness aceptada, no texto del dueño. *Decisión del orquestador.*
  - Los pastes (`<pasted_content>`) cuentan como mensaje humano.
  - El filtro entra en el sello: cambiarlo después es una errata.
- Las ~47 sesiones anteriores al 10-06 son reserva. Solo entran si la
  etapa 1 da menos de 30 correcciones, para tener n suficiente en la
  etapa 2, y se declara en el resultado.

### 4.2 Etapa 1: detectar correcciones
- Unidad: un mensaje humano más el último texto del asistente que lo precede
  (≤ 1,5 k tokens, truncado por delante).
- Juez aislado con el patrón de `evals/rubrica-reglas`: `claude -p`, sin
  herramientas y sin contexto de la KB, en Haiku. Salida JSON
  `{correccion: si|no, tipo: rechaza-propuesta|contradice-hecho|repite-instruccion|redirige|null}`.
- **Validación (antes de usar el resultado):** el dueño etiqueta 80 mensajes
  a ciegas, sin ver la etiqueta del juez: 40 positivos de regex
  (`no uses|otra vez|siempre|nunca|está mal|déjate de`) y 40 al azar. Se
  sella `kappa(juez, dueño) ≥ 0,4`.
- El kappa se calcula sobre la muestra mixta de 80 tal cual, sin ponderar.
  Es un criterio de «el juez separa», no una estimación poblacional. Con
  ~0,03 correcciones por mensaje, 40 al azar no estiman el recall. Además se
  informa la precisión y el recall del juez frente al dueño en esos 80.
  *Decisión del orquestador.*
- **Contrato de etiquetado:** el sello (prompts, umbrales y lista de
  sesiones) se congela **antes** de etiquetar. Las 80 se entregan como
  fichero jsonl en `~/.cache/exo-uso-real/`, con id, mensaje y turno previo,
  sin el veredicto del juez y en orden aleatorio con semilla sellada. El
  dueño rellena `etiqueta: si|no`. El script de validación es nuevo:
  reutiliza `estadistica.py` (kappa) pero no el esquema de reglas de
  `validar.py`. Sin etiquetas no hay veredicto.
- Si `kappa < 0,4`: el juez no sirve. El dueño etiqueta a mano los
  candidatos de regex y el resultado se declara como cota inferior.

### 4.3 Puerta
Tasa = correcciones / sesiones, con IC 95 % de Poisson exacto.
- **IC superior < 0,4** ⇒ se cumple la condición: estabilizar exo. La
  etapa 2 no corre.
- En cualquier otro caso (el IC cruza o supera el 0,4) ⇒ etapa 2. La etapa 1
  cuenta todas las correcciones, así que es una cota superior de las
  decididas. Con el IC cruzando, no se estabiliza por un empate.

### 4.4 Etapa 2: atribuir
- Solo sobre los positivos de la etapa 1.
- Contexto del juez:
  - el mensaje, el turno previo del asistente y los mensajes humanos
    anteriores de la misma sesión;
  - la KB reconstruida con `git show <commit>:<ruta>` en el último commit
    anterior al primer timestamp de la sesión: nota de proyecto del repo
    (resuelta por `slug`/repo, como `exo rules`), `core/reglas-duras.md` y
    `core/` de perfil;
  - el bloque de reglas del `prompt_snapshot` de la sesión, si existe.
- La ruta del repo de la KB es un argumento o una variable de entorno,
  nunca una constante del repo público. La nota se resuelve en el commit de
  la fecha, con `git log --follow` si se renombró. Si la nota de proyecto
  no existía en esa fecha, el contexto de proyecto va vacío y la sesión se
  marca `nota_ausente`.
- Las notas canónicas existían antes de la siembra del 10-25; esa siembra
  solo añadió `## Reglas duras`. Por eso el canon sí está disponible en todo
  el periodo. Aun así, el resultado declara cuántas sesiones fueron
  `nota_ausente`: si son muchas, la invalidación del 10 % podría ser un
  artefacto.
- Juez Sonnet, salida
  `{atribucion: regla-canal|regla-fuera-canal|canon-kb|perfil-core|dicho-en-sesion|nueva, nota, linea}`.
- **Decidida** = `regla-canal`, `regla-fuera-canal`, `canon-kb` o
  `perfil-core`.
- El dueño valida 30 positivos (estratificados por atribución) y se informa
  el acuerdo. No hay umbral sellado para esta validación; si el desacuerdo es
  grande, se declara en el resultado.
- **Métrica final:** correcciones decididas por sesión, con su IC 95 %,
  frente a 0,4. Mismo criterio que la puerta: IC superior < 0,4 ⇒
  estabilizar. Si no, el desglose por atribución y repo alimenta (b), (c) y
  (d).
- **Invalidación de la unidad** (consultor): con n ≥ 30 correcciones, si
  menos del 10 % casa con la KB, B es una unidad vacía. Lo corregido es nuevo
  o de la propia sesión, y se aplica la condición directamente.

### 4.5 Violaciones deterministas (informativo, fuera de la puerta)
Conteo mecánico, sin LLM, de las reglas globales con patrón comprobable
(`cd … && git`, `git add -A`, `git stash`, `git checkout --`) sobre los
`tool_use` de Bash de las sesiones principales y de `subagents/`, por repo.
No cuestan correcciones, cuestan permisos y retrabajo. Es el insumo de (b).
Se informan los conteos crudos y la tasa por cada 100 llamadas a Bash,
separando sesiones principales y subagentes. Los patrones van con tests de
falsos positivos: `git -C`, `git stash list`/`show`, `git checkout -b` y
`git add <ruta>` no son violaciones; `cd x && git` y `cd x; git` sí. Son
regex de shell, no un parser, y lo que no casa no cuenta.

### 4.6 Sellado y repo público
- Antes de correr se sella en `sello.json`:
  - hash de la lista de sesiones;
  - commit HEAD de la KB;
  - hash de los dos prompts de juez;
  - umbrales: 0,4, kappa 0,4, n ≥ 30 y 10 %;
  - modelos.
- Al repo (público) solo van código, ids de sesión, hashes, conteos y el
  veredicto. El texto de los mensajes, los fragmentos de la KB y las
  etiquetas del dueño con su texto viven en `~/.cache/exo-uso-real/`.
- Se registra el coste de cada etapa. Hay un tope de coste por etapa con el
  mismo mecanismo que `evals/rubrica-reglas`: 3 USD en la etapa 1 y 25 USD
  en la etapa 2. Al alcanzarlo se para con exit 3. Los veredictos se
  cachean por id de mensaje en `~/.cache/exo-uso-real/`, así que la etapa se
  reanuda sin repetir llamadas. *Decisión del orquestador.*

### 4.7 Testing
TDD con fixtures sintéticas de jsonl, sin datos reales en el repo:
- filtro de mensaje humano (cada prefijo, `isMeta`, `toolUseResult`, paste);
- exclusión de `sdk-cli`;
- detección del bloque de reglas en `prompt_snapshot`;
- resolución del commit de la KB por fecha;
- IC de Poisson y lógica de la puerta (por debajo, cruzando, por encima);
- conteo de violaciones deterministas.

## 5. Presupuesto
Etapa 1 ~1 USD (≈ 3 M tokens en Haiku). Etapa 2 ~15 USD (≈ 5 M tokens en
Sonnet), solo si la puerta la abre. Etiquetado del dueño: 80 + 30 ítems
(~1 h).

## 6. Fuera de alcance
- Medir el efecto del canal (antes/después): no hay exposición (§2).
- Cambiar umbrales tras ver datos. Una errata solo antes de correr la
  etapa que afecta, como en `Ap`.
- Construir gates (b) o E (d): los decide el resultado, no esta spec.
