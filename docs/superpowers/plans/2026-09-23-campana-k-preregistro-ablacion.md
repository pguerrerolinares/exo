# Pre-registro — Campaña K: ¿trabaja mejor el agente con exo? (ablación)

> **Estado: REGLAS CONGELADAS el 2026-09-28 (paso 1 de §12).** El gold y los
> dos huecos de la Task 0 se congelan en el paso 2, antes de correr ningún
> brazo. Las erratas van al verdict.
>
> **Qué se ha observado al redactarlo (2026-09-23), y qué no.** Solo
> recuentos: `~/.claude/reflex-log.jsonl` tiene 314 eventos
> `recall-inject-emitted` repartidos en 80 `session_id` distintos, y hay 566
> transcripts `.jsonl` bajo `~/.claude/projects/`. También se abrió un evento
> suelto para ver el formato (lleva `session_id`, `ts` y `permalinks=` con
> los inyectados). **No se ha visto:** ninguna métrica de uso de la fase 0,
> ningún brazo corrido y ninguna tarea redactada. Las reglas de §6 se fijan a
> ciegas.

## 1. Preguntas

Todo lo medido hasta hoy mide el instrumento (hit@5, MRR, latencia) y no el
resultado. Esta campaña mide el resultado.

- **K1 (la de fondo).** ¿Acaba el agente con exo en producción más tareas
  bien que el agente sin exo, en tareas cuya respuesta correcta depende de
  memoria?
- **K2 (el daño).** En tareas donde la KB no tiene nada relevante, ¿le cuesta
  a exo corrección (ruido, distracción; Shi et al., ICML 2023,
  arXiv:2302.00093) o tokens?
- **K3 (descomposición, solo si K1 gana).** ¿Qué pieza aporta el efecto?
  - K3a: ¿se gana su coste la inyección por prompt (`recall-inject`) frente a
    buscar bajo demanda?
  - K3b: ¿se gana su coste el engine (hybrid) frente a `grep`/`Read` sobre la
    misma KB?
  - K3c: ¿aporta el `core-index` de arranque por encima del CLAUDE.md global?
- **K0 (fase 0, observacional, solo descriptiva).** De lo que inyecta
  `recall-inject` en producción, ¿cuánto llega a tocar el agente?

## 2. Hechos de partida (a re-verificar en la Task 0)

1. `recall-inject.sh` inyecta ≤1.024 B por prompt con ~1 s de coste
   (`s6-hook-entero-n174` p50 = 1.079 ms,
   `evals/recall-coste/verdict/2026-09-campana-a.md`).
2. En held-out, el estrato `prompt` acierta 11/22 en hit@5 y el corpus
   negativo devuelve algo en el top-5 en 54/55 queries nulas
   (`evals/retrieval-heldout/verdict/agregados.md`). Por eso K2 no es
   retórica.
3. El CLAUDE.md global (`~/.claude/CLAUDE.md`) es un extracto del perfil y es
   memoria nativa de Claude Code, no de exo. Para esta campaña es **línea
   base**: está en todos los brazos (§4, D3).
4. `reflex-log.jsonl` registra por inyección: `session_id`, `ts` y
   `permalinks`. Los transcripts registran los `tool_use`. Con eso K0 se puede
   calcular sin correr nada.

## 3. Fase 0 — tasa de uso (observacional, gratis)

**Unidad:** un evento `recall-inject-emitted` con `ts` < fecha de
congelación.

**«Tocado».** Un permalink inyectado cuenta como tocado si, en la misma
sesión y **después** del `ts` del evento, pasa al menos una de estas cosas:

- (a) un `Read` o `Edit` sobre su fichero;
- (b) un `tool_result` de `Grep`/`Bash` (incluidos `exo search`/`recall`) que
  contiene su ruta o su permalink;
- (c) un texto del asistente que cita su permalink o el título exacto de la
  nota.

**Métricas:**

- **U** = fracción de eventos con ≥1 permalink tocado.
- **U_base** = lo mismo, pero sustituyendo los permalinks inyectados por 3
  notas aleatorias del mismo `tier`, con semilla `20260923`. Es el control:
  el agente toca notas de la KB por su cuenta.
- **Lift** = U − U_base, con IC95 bootstrap por sesión (10.000 réplicas,
  misma semilla).

**Límite declarado:** «no tocado» no significa «inútil». El primer párrafo
inyectado puede bastar sin abrir la nota. Por eso la fase 0 **no decide
nada**. Sirve para dimensionar y queda como métrica de salud del hook.

## 4. Brazos (fase 1)

Cuatro brazos. Todo lo demás queda constante: modelo (D1), versión de Claude
Code, repo y commit de la tarea, CLAUDE.md global, `--max-turns` y
presupuesto de tokens.

| brazo | CLAUDE.md global | core-index al arrancar | binario `exo` en PATH | `recall-inject` por prompt |
|---|---|---|---|---|
| **A0** vanilla | sí | no | no | no |
| **A1** core + grep | sí | sí | no (la KB se lee con `grep`/`Read`) | no |
| **A2** core + exo bajo demanda | sí | sí | sí | no |
| **A3** producción | sí | sí | sí | sí |

- **Apagados en los cuatro brazos:** los reflejos que no son memoria
  (`git-c`, `zero-residuo`, `verify-before-done`, `clean-orchestrator`,
  `estilo-directo`, `document-remind`, `exo-index`) y `subagent-inject`. Lo
  que se mide es la memoria, no la doctrina.
- **Nota sobre A1:** el `core-index` le dice al agente «busca con `exo
  search`». En A1 ese binario no existe. Se sustituye esa línea por «busca en
  `$KB` con `grep`/`Read`», de modo que la instrucción sea ejecutable. Es la
  única diferencia de texto entre brazos.
- **Aislamiento (a verificar en la Task 0, no se asume):** una configuración
  por brazo en un directorio propio (`CLAUDE_CONFIG_DIR` o el mecanismo que
  la Task 0 confirme), un índice propio (`EXO_INDEX`) y
  `claude -p --output-format stream-json`. La Task 0 comprueba con un canario
  que ninguna config del usuario se cuela.

## 5. Tareas y gold

**Estratos:**

- **S1: depende de memoria.** La respuesta correcta depende de una decisión,
  convención o aprendizaje que ya estaba en la KB en el instante T de la
  tarea. Se reportan dos subestratos:
  - S1a: el conocimiento está solo en la KB (típico: algo transversal a
    proyectos);
  - S1b: también está en el repo (docs, backlog), así que A0 también puede
    encontrarlo.
- **S2: neutral.** La KB no tiene nada relevante. Mide el daño (K2).

**Pool de candidatas, extraído mecánicamente y no elegido a dedo** (el
sesgo de selección es la amenaza principal: el autor de exo sabe dónde
brilla exo):

1. Correcciones históricas en transcripts: turnos de usuario que le corrigen
   al agente algo ya decidido. Patrones: `ya (lo )?(decidimos|cerramos|dijimos)`,
   `te dije`, `como quedamos`, `eso ya`, `otra vez`, más las invocaciones de
   un learning por su título. Cada acierto es una tarea: el prompt previo a la
   corrección, en el commit de esa sesión.
2. Notas `learnings/` y destilados de proyecto: cada regla accionable da una
   tarea cuyo camino por defecto viola la regla (ejemplo de forma: una tarea
   de git en la que el reflejo natural es `cd … && git`).
3. S2: tareas de repos o temas sin notas en la KB, extraídas del historial
   de git de esos repos (commits `feat`/`fix` pequeños, revertidos a su
   padre).

**Muestreo:** del pool filtrado, muestreo aleatorio estratificado con semilla
`20260923`. Paul **no elige** tareas. Solo veta las inviables (entorno
irreproducible, secreto, >30 min humanos) y cada veto se registra con su
motivo.

**Sin fuga del futuro:** para cada tarea, la KB del brazo es un checkout de
`wisdom-paul` en el último commit estrictamente anterior a T, con el índice
reconstruido desde cero en un `EXO_INDEX` aislado. Una nota escrita *por* la
sesión original (o después) resolvería la tarea desde el futuro. La Task 3
lo comprueba tarea a tarea: ninguna nota del snapshot tiene fecha ≥ T.

**Criterio de éxito por tarea**, escrito antes de correr ningún brazo:

- **Preferido:** un check ejecutable (test, `grep` sobre el diff o sobre los
  comandos del transcript, salida esperada).
- **Si no se puede:** rúbrica binaria de 1–3 ítems, corregida por un juez
  LLM (D1-juez) que ve **solo** el diff final y la respuesta final. Antes de
  juzgar se eliminan del transcript los bloques inyectados (`=== Recall exo`)
  y cualquier marca de brazo.
- **Acuerdo:** Paul adjudica a ciegas una muestra aleatoria del 25 % de las
  corridas juzgadas por LLM. Si κ de Cohen < 0,60, el juez se invalida y solo
  cuentan las tareas con check ejecutable.

**Congelación del gold:** `gold.jsonl` privado (tarea, estrato, repo@commit,
commit de KB, check o rúbrica) en `.superpowers/fabrica/k/` (gitignored). En
el repo público solo se publica su sha256.

## 6. Métricas y reglas de decisión (fijadas antes de correr)

**Réplicas:** k = 2 por tarea y brazo (el agente es estocástico; D2). El
resultado por tarea y brazo es la tasa de éxito, en {0, ½, 1}.

**Primaria:** Δ = media por tarea de (éxito brazo cand − éxito brazo ref),
emparejada por tarea. IC95 por bootstrap sobre tareas (10.000 réplicas,
semilla `20260923`) y test de signos exacto sobre las tareas discordantes.

**Secundarias (descriptivas, no deciden):** tokens totales, turnos, wall-clock,
violaciones de una decisión registrada (anotadas en la rúbrica) y veces que
el agente pregunta en vez de actuar (en `-p` no hay usuario: preguntar es no
terminar).

**Carga de la prueba en el brazo más complejo.** Por defecto gana lo simple,
y una pieza de exo tiene que demostrar que se gana su coste (D4).

### Etapa 1: K1 y K2, solo A3 contra A0

- **R1 (K1, sobre S1).**
  - `EXO AYUDA` si el límite inferior del IC95 de Δ(A3−A0) > 0 **y**
    Δ ≥ 0,10.
  - `EFECTO PEQUEÑO O NULO` si el límite superior del IC95 < 0,10.
  - `NO CONCLUYENTE` en cualquier otro caso. Se reporta el IC tal cual y no
    se amplía N a posteriori.
- **R2 (K2, sobre S2), no-inferioridad.** `SIN DAÑO` si el límite inferior
  del IC95 de Δ(A3−A0) > −0,10. Si no se cumple: `DAÑO`, y abre ítem Alta en
  el backlog sobre la abstención de `recall-inject`, con independencia de R1.
- **Parada:** si R1 ≠ `EXO AYUDA`, **la etapa 2 no se corre**. No tiene
  sentido descomponer un efecto que no se ha visto.
  - Con `EFECTO PEQUEÑO O NULO`: recomendación pre-registrada de reducir exo
    en producción a core-index + búsqueda bajo demanda (A2) o menos, y de
    dejar de invertir en ranking.
  - Con `NO CONCLUYENTE`: no se toca producción y se declara el techo de lo
    que N permite ver (§7).

### Etapa 2: K3, solo si R1 = `EXO AYUDA`

Tres comparaciones sobre S1, corregidas por Holm al α = 0,05 (test de
signos):

| regla | cand vs ref | «gana» si… | si no gana (D4) |
|---|---|---|---|
| **R3 (K3a)** | A3 vs A2 | Holm-significativo y Δ ≥ 0,05 | `recall-inject` pasa a off por defecto |
| **R4 (K3b)** | A2 vs A1 | Holm-significativo y Δ ≥ 0,05 | el hybrid se queda solo para `exo search` manual; se congela la inversión en ranking |
| **R5 (K3c)** | A1 vs A0 | Holm-significativo y Δ ≥ 0,05 | el core-index se reduce a lo que ya cubre el CLAUDE.md |

**Desempate por coste:** si una comparación no gana y el brazo simple cuesta
menos tokens o menos latencia, gana el brazo simple, sin discusión.

## 7. Tamaño y lo que permite ver

- **Objetivo:** S1 = 40 tareas (S1a ≥ 20), S2 = 20.
- **Etapa 1:** 60 tareas × 2 brazos × 2 réplicas = **240 corridas**.
- **Etapa 2:** +40 × 2 brazos (A1, A2) × 2 = **160 corridas** más.

**Potencia (aproximación McNemar a nivel de tarea, α = 0,05, 1−β = 0,80):**
con 40 tareas se ven diferencias de **≈25 pp** si la discordancia ronda el
35 %. Con k = 2 las réplicas apenas reducen la varianza por tarea, y **un
efecto de 10 pp no es visible con este N**. Por eso R1 tiene la salida
`NO CONCLUYENTE`, y por eso la etapa 2 (tres comparaciones con Holm) solo ve
efectos grandes. Se declara antes: un «no gana» en la etapa 2 significa «no
demostrado», no «demostrado nulo». D4 decide qué hacer con eso.

**Si el pool no llega a 40 en S1:** la Task 2 para y lo escala. No se
inventan tareas para rellenar.

## 8. Amenazas a la validez declaradas

- **Headless ≠ interactivo.** Sin Paul no hay corrección a mitad de tarea,
  y ahí es donde la memoria evita el «ya lo decidimos». Esto sesga *contra*
  exo; se acepta.
- **Modelo.** Si se corre con Sonnet 5 y en el día a día se usa Opus, el
  efecto puede no transferir (D1).
- **Tareas extraídas de correcciones.** Son justo los casos donde el agente
  sin memoria falló una vez: sesgo *a favor* de exo en S1. S2 compensa en
  parte, y se reporta S1 por origen del pool (1 vs 2).
- **La KB cambia con el tiempo.** Cada tarea usa su propio snapshot, así que
  el resultado es de exo con la KB real de cada momento, no de una KB ideal.

## 9. Circuit breakers

- **Fuga de brazo:** si un transcript de A0/A1 contiene `=== Recall exo` o un
  `exo` ejecutado, o uno de A2 contiene una inyección por prompt, se para
  todo y se arregla el aislamiento. Las corridas previas se tiran.
- **Infra:** si >10 % de las corridas de un brazo fallan por infraestructura
  (timeout, error de API), se para. No se reponen solo las del brazo que
  falló.
- **Coste:** se para al alcanzar el tope de D2. Se reporta lo corrido y no se
  cuenta como resultado.

## 10. Decisiones de Paul que se fijan en la congelación

- **D1. FIJADA (2026-09-28): brazos con Sonnet 5.5** (id exacto a verificar
  en la Task 0; si no existe, se para y se consulta, no se sustituye en
  silencio). Juez: **Opus 5.5** (propuesto el 2026-09-28 sin objeción de
  Paul); distinto del modelo de los brazos, como exige §5.
- **D2. FIJADA (2026-09-28): k = 2.** Tope fijado el 2026-09-28 con la
  Task 0 (`evals/ablacion-k/task0-recon.md`), en **consumo de cuota** y no
  en USD: las corridas van con la suscripción (OAuth), así que el
  `total_cost_usd` es precio de lista y no facturación.
  - Etapa 1: tope de **240M tokens de entrada** (240 corridas × ~1M, cota
    alta estimada).
  - Se corre en **tandas de 40 corridas**. Tras la primera, Paul revisa
    `/usage` y se decide si el ritmo es sostenible.
  - Si una tanda consume bastante más de lo estimado, se para y se
    recalcula antes de seguir.
  - Freno por corrida: `--max-budget-usd 3` (unidad de lista, usado solo
    como límite de consumo).
- **D3. FIJADA (2026-09-28): el CLAUDE.md global está presente en A0.** La
  pregunta es «¿aporta exo sobre la memoria nativa de Claude Code?»; quitarlo
  atribuiría a exo lo que da el CLAUDE.md.
- **D4. FIJADA (2026-09-28): la carga de la prueba recae en la pieza
  compleja.** Una comparación que no gana en la etapa 2 lleva a su columna
  «si no gana» de §6: la pieza pasa a off por defecto, instalada y
  reactivable. «No demostrado» no se lee como «demostrado nulo» (§7).
- **D5. FIJADA (2026-09-28): A1 (grep) se incluye.** Sin A1, A2 contra A0
  mezcla el core-index con el engine y R4 no tiene contra qué medirse.

## 11. Tasks (esbozo; el plan de ejecución se escribe tras congelar)

0. **Recon del harness.** Flags reales de `claude -p`, aislamiento de config
   y plugins por brazo, canario de fuga. Mide coste por corrida en 3 tareas
   de prueba **fuera del pool**.
1. **Fase 0:** script de U / U_base / lift sobre el log y los transcripts.
   Informe descriptivo.
2. **Extracción del pool** (§5), filtros, muestreo con semilla y vetos
   registrados.
3. **Snapshot de KB por tarea**, índice aislado y comprobación de no fuga
   del futuro.
4. **Checks y rúbricas** por tarea. Congelación del gold (sha256 en §12).
5. **Etapa 1** (A0, A3). Juez, muestra de κ y verdict R1/R2 por un
   adjudicador fresco que recomputa sin el script del harness.
6. **Etapa 2** (A1, A2), solo si R1 = `EXO AYUDA`.

## 12. Congelación

Congelación en dos pasos. Ningún brazo se corre antes del paso 2.

1. **Reglas (§1–§10):** congeladas en el commit que introduce este texto,
   el 2026-09-28. Tras ese commit, §1–§10 solo cambian por errata declarada
   en el verdict, salvo los dos huecos que la Task 0 tiene que medir, y que
   se rellenan en el paso 2 sin tocar nada más: el id exacto de Sonnet 5.5
   (D1) y el tope de coste (D2).
2. **Gold y huecos de la Task 0:** al terminar la Task 4, antes de la
   primera corrida de la etapa 1.
   - Commit: _pendiente_
   - sha256 de `gold.jsonl`: _pendiente_
   - id del modelo de brazos: `claude-sonnet-5-5` (Task 0) · tope: D2 (§10)
