# Consulta — issue #23: el recall del hook desplaza a `exo search`

**Fecha**: 2026-09-19. **Consultor**: Fable (sesión delegada). **Alcance**: diagnóstico y
recomendación; sin implementar nada. Todo lo que aquí se afirma como medido se midió hoy
sobre esta máquina (`exo 0.1.0`, plugin `exo` en `main` @ `99ddd05`, índice real de 174 notas)
o sobre las transcripciones locales de `~/.claude/projects/`.

---

## 0. Recomendación en una pantalla

**Hacer ahora (mínimo que cubre el caso):**

1. **Pie honesto de ≤120 B** en `recall-inject.sh:223`, que conserve la licencia de ignorar
   (es la defensa contra falsos positivos de M6-06 §2.4, no se tira) y añada **alcance + comando
   literal**:

   ```
   (ignóralo si no aplica) Es top-3 de UNA query: para más, exo search --type hybrid "<q>"
   ```

   Medido: 89 B; bloques de 818/879/804 B con 3 hits y **cero recortes de snippet** en tres
   prompts reales. Con pies de hasta 113 B sigue sin recortar; a 167 B empieza a morder. `--type hybrid` es **obligatorio** en el literal: el binario
   instalado (0.1.0) sigue con `--type fts` por defecto, y un `exo search "<q>"` pelado es
   grep con pasos extra.

2. **La allowlist que M6-05 prometió y no existe**: `settings.json` tiene `Bash(grep:*)` y
   ningún `Bash(exo …)`. Hoy no muerde porque `defaultMode: auto`, pero es una desalineación
   documentada-vs-real (spec 2026-08-18 §M6-05: «*no pide permiso cada vez*») y en cualquier
   máquina en modo `default` inclina la balanza hacia grep.

3. **Instrumento de medida offline, pre-registrado**: un script que lea las transcripciones y
   reporte, por sesión sustantiva, si el primer acceso a la KB fue `exo search` o
   `grep`/`Read`. Es el mismo patrón de la campaña A (medir desde logs, sin tocar producción).
   Línea base de hoy: **5 de 13**. Umbral de reapertura: si a las tres semanas no llega a
   **≥9 de las siguientes 13** (≈70%), se escala a la opción 3.

**No hacer ahora**: el hook `PreToolUse` (estado, 4º hook sobre cada `Bash`, falsos positivos
legítimos) ni la eval con `claude plugin eval` (cara, no determinista, y el instrumento de (3)
mide lo mismo gratis sobre uso real). Ambas quedan diseñadas abajo como escalada con gatillo.

**Por qué esta y no otra**: el fallo es **probabilístico** (38% de acierto, no 0%), y cuando el
modelo sí busca **copia literalmente la línea de CLAUDE.md**. Eso dice que el literal funciona
cuando está a mano y que lo que falta es tenerlo en el punto de uso, con el alcance del bloque
declarado. Un gate mecánico se justifica cuando la prosa cercana ha fallado medida, no antes.

---

## 1. Lo que hay medido (hoy)

### 1.1 Tasa base en transcripciones reales

Script: `scratchpad/mine2.py` (no entra al repo). Universo: sesiones con al menos un
`recall-inject-emitted` en `~/.claude/reflex-log.jsonl` (61), **excluyendo** el repo `exo`
(ahí `exo search` es el sistema bajo prueba, no memoria) y las corridas aisladas de eval
(1 turno). Quedan **13 sesiones con ≥3 turnos humanos**, del 25-ago al 18-sep.

| Primer acceso a la KB en la sesión | Sesiones |
|---|---|
| `exo search` | **5** |
| `grep`/`rg`/`Grep`/`Read` sobre `wisdom-paul` | **6** |
| ninguno | 2 |

Totales en esas 13: **78 greps sobre la KB frente a 23 `exo search`** (3,4:1). Los peores
casos: `b24d2da2` (18-sep, wisdom-paul, 12 turnos): **20 greps, 0 search**; `06876e80`:
6/0; `570d039f` (ai-news, 61 turnos): 1/0.

Tres cosas que salen de leer los comandos, no solo de contarlos:

- **Cuando busca, copia el literal de CLAUDE.md** casi byte a byte:
  `exo search --db ~/.exo/index.db --type hybrid --json "…"`. En ninguna sesión inventó la
  forma; en varias la envolvió en un `for q in …` para lanzar 3-5 queries seguidas. El
  literal a mano **sí** se usa.
- **Cero rechazos de permiso** sobre comandos `exo` en las 13 sesiones. La allowlist no es
  la causa próxima (modo `auto`), aunque sea deuda.
- **No es "nunca"**: 5/13 lo hacen bien a la primera. Cualquier fix hay que evaluarlo como
  desplazamiento de una tasa, no como "arreglado".

### 1.2 Lo que el bloque inyectado dice y cuesta

Bloque real hoy (prompt del issue, `revisa el MCP server de ai-news-platform…`): 777 B, 3
hits, pie de 48 B `(puede no venir al caso: ignóralo si no aplica)`. Ni cabecera ni pie
nombran `exo search`; la única instrucción que contiene apunta a **hacer menos** (ignorar).

Coste de un pie más largo (hook real, tres prompts, snippets reales del engine ≤200 B):

| Pie | Bytes pie | Bloque (3 prompts) | Snippets recortados |
|---|---|---|---|
| actual | 48 | 777 / 838 / 763 | 0 / 0 / 0 |
| alcance + comando | 113 | 842 / 903 / 828 | 0 / 0 / 0 |
| alcance + comando + «nunca grep» | 167 | 882 / 957 / 882 | **1** / 0 / 0 |

El presupuesto por hit se deriva del cap (`(1024 − cabecera − pie − 2) / 3`), así que el
pie se paga en snippet solo cuando supera ~120 B. Por debajo, **es gratis**: el engine capa
cada snippet a 200 B (`SNIPPET_MAX_BYTES`) y el presupuesto derivado sigue por encima.

### 1.3 La forma de las herramientas

- `Grep` es **tool nativa** de Claude Code (sin Bash, sin flags que recordar) y **devuelve
  contenido** (las líneas que casan). `exo search` es un comando Bash con 3-4 flags a
  recordar y devuelve **punteros sin snippet** (permalink, tipo, score, ruta): el modelo tiene
  que encadenar un `Read` para ver algo. Con grep ve algo en un paso.
- `exo search` sin `--type hybrid` en 0.1.0 es FTS con AND implícito: *"vamos con brainstorm
  de M6-06"* → 0 resultados (medido en M6-06 §1). Si el literal que se enseña no lleva el
  flag, el agente prueba una vez, ve vacío y vuelve a grep. Razón de más para que el pie lo
  lleve escrito.
- `exo recall --query` sí devuelve snippets, pero su cabecera humana dice «*no sustituye tu
  brief*» (está redactada para subagentes) y no es el comando del contrato. No lo recomiendo
  como sustituto; lo anoto porque explica por qué el bloque del hook «parece» una búsqueda
  completa y `exo search` «parece» menos.

### 1.4 Deriva documentación-vs-estado encontrada de paso

- **M6-05 prometió allowlist** (`spec 2026-08-18 §M6-05`: «*Más una línea de allowlist para
  `Bash(exo …)`/`Bash(kbx …)` […] no pide permiso cada vez*») y **`settings.json` no la
  tiene**. La spec de M6-06 §7 la da por hecha («*allowlisted desde M6-05*»).
- La nota del proyecto dice que la ola 1 (PR #26) cerró D6 con `hybrid` por defecto en
  `search`; **el binario instalado es 0.1.0 y sigue en `fts`**. Mientras no se instale 0.2.0,
  cualquier literal sin `--type hybrid` degrada en silencio con forma válida (ley 1 de
  [[Fallo silencioso]]).

---

## 2. Diagnóstico: el root cause, desafiado

Tu localización (`recall-inject.sh:223`, pie que empuja a hacer menos y no nombra el
comando) es correcta pero **incompleta**, y la parte que falta cambia la forma del fix.

**Lo que el modelo hace no es un sesgo: es una inferencia correcta sobre un bloque mal
etiquetado.** El bloque *es* el resultado de una búsqueda `hybrid` sobre el prompt —la misma
que manda el contrato— y se presenta como «Recall exo … material de la KB». Concluir «ya se
buscó» es verdad para la query literal. Lo que el contrato quiere y el bloque no da es (a)
**queries reformuladas** por faceta (el prompt de Paul no es una buena query: es una orden con
relleno), (b) **más de tres hits**, y (c) el paso de **leer la nota**. El bloque no declara
su alcance («una query, top-3, ≥0,40»), así que el modelo no tiene forma de saber que es un
*teaser* y no el resultado. Por eso el pie debe decir *qué es* el bloque antes de decir *qué
hacer*: «Es top-3 de UNA query» es lo que convierte «ya busqué» en «busqué una vez, mal».

**El segundo factor es forma de herramienta, no prosa** (§1.3): grep es nativo, sin flags y
devuelve contenido; `exo search` es Bash, con incantación, y devuelve punteros. Ninguna
frase compite con eso cuando la frase vive a 8 KB de distancia en el system prompt. Cuando
la frase vive **al lado del bloque**, con el comando escrito, el dato de §1.1 (el modelo copia
el literal cuando lo tiene) dice que sí compite.

**Sobre «contrato por prosa» (ley 3):** cierto, y por eso la recomendación viene con
**instrumento de medida y gatillo de escalada**, no con fe. Pero la ley 3 habla de prosa
como *mecanismo de acoplamiento* (un router que lee una frase). Aquí la prosa es la
*interfaz con el modelo*; no hay alternativa que no sea prosa salvo bloquear la herramienta
(`PreToolUse deny`), y bloquear grep sobre la KB rompe casos legítimos (Paul pidiendo un grep
literal, `/document` verificando un anchor, el propio `kb-precommit`). La distinción útil es
**prosa lejana vs. prosa en el punto de uso**, y eso sí tiene literatura (§5).

**Lo que NO es la causa**: la licencia de ignorar como tal. Retirarla reabriría el problema
que la motivó (M6-06 §1: `hybrid` a 0,40 **nunca se abstiene**; el pie es la única mitigación
de FP). Se conserva, comprimida.

---

## 3. Opciones, trade-offs, falsabilidad

### Opción 1 — Pie accionable (alcance + comando literal). **RECOMENDADA**

- **Qué**: `FOOTER` pasa de 48 B a 89 B (techo medido sin coste: 113 B). Conserva `ignóralo si no aplica` (el test
  existente lo pina), añade «Es top-3 de UNA query» y el comando completo con `--type hybrid`.
- **Coste**: 0 B de snippet (medido, §1.2); 0 procesos nuevos; 1 línea de bash + 2-3 asserts.
- **Pros**: barato, reversible, ataca las dos causas (alcance mal declarado y literal lejos).
- **Contras**: sigue siendo prosa; el efecto es probabilístico y hay que medirlo.
- **Test determinista** (`test-recall-inject.sh`): (i) el bloque contiene `exo search` y
  `--type hybrid`; (ii) sigue conteniendo `ignóralo si no aplica`; (iii) con el fixture
  `GORDO` (snippets de 200 B, el tamaño real del engine) el bloque **no contiene `…`** —es
  el guard contra un pie que engorde y empiece a recortar; hoy ese fixture solo comprueba
  ≤1024 y 3 hits, así que un pie de 300 B pasaría la suite. Mutación: quitar `--type hybrid`
  del pie ⇒ rojo.
- **Verificación de efecto** (no determinista): el instrumento de §4.

### Opción 2 — Recall solo como punteros, sin snippet. **RECHAZADA**

- Pierde lo que M6-06 midió como la función del snippet (decidir *si* abrir la nota) y
  **no ataca la causa**: el modelo seguiría viendo tres rutas relevantes y concluyendo «ya
  hay contexto». Encima el snippet es lo que hace que el bloque tenga valor cuando el
  prompt no nombra nada.

### Opción 3 — `PreToolUse` que avisa (sin bloquear) ante grep/rg/Grep sobre la KB sin `exo search` previo. **ESCALADA, con gatillo**

- **Cómo sería sin fichero de estado**: el hook recibe `transcript_path` en stdin; un
  `grep -q 'exo search' "$transcript_path"` responde «¿ya hubo búsqueda en esta sesión?» sin
  inventar estado propio ni limpieza. Matcher `Bash|Grep`; para `Bash` mira
  `tool_input.command` (`grep|rg` + raíz de la KB), para `Grep` mira `tool_input.path`.
  Salida: `additionalContext` con el mismo literal del pie (ver §6 para lo que el harness
  admite exactamente). Exit siempre 0.
- **Coste**: 4º hook sobre **cada** `Bash` (ya hay tres); un `jq` + un `grep` sobre un
  `.jsonl` que crece por turno; en W11 cada spawn son 25-60 ms (campaña A). Portabilidad
  de rutas Windows en `transcript_path`. Superficie nueva con sus tests.
- **Falsos positivos legítimos**: Paul pide «grepea X en la KB»; `/document` buscando un
  anchor; `distill`/`kb-precommit`; un `grep` de verificación tras haber leído la nota. El
  aviso no bloquea, pero cada FP es ruido en el contexto del padre, que es justo donde el
  ruido más cuesta (doctrina de orquestador limpio).
- **Test determinista**: sí, y bueno — transcript falso con/sin `exo search`, comando con/sin
  raíz de KB, las cuatro combinaciones; JSON válido; exit 0 con transcript inexistente.
- **Cuándo**: si el instrumento de §4 no muestra el desplazamiento esperado tras la opción 1.

### Opción 4 — Eval en `claude plugin eval` que exija ≥1 `exo search`. **NO AHORA**

- Es el único mecanismo que mide **el comportamiento del modelo** en condiciones controladas,
  pero: (i) es no determinista y con una tasa base de 38% hace falta n≥10 corridas por caso
  para ver un desplazamiento a 70% con algo de potencia; (ii) cuesta tokens reales por
  corrida; (iii) mide un prompt sintético, no las sesiones de Paul. El instrumento offline de
  §4 mide lo mismo sobre uso real, gratis, y ya existe el precedente (campaña A midió el
  recall por prompt desde `reflex-log`). Ver §6 para qué puede asertar exactamente la eval.
- **Cuándo sí**: si se quiere un gate de CI que impida regresiones del pie. Hoy no hay
  regresión que impedir: no hay pie.

### Otras que enumeraste o que salen del análisis

- **Que el hook haga la búsqueda hybrid completa**: ya la hace (`exo recall --query` es
  `hybrid` a 0,40). El problema no es el motor, es el alcance (una query) y el cap.
- **Inyectar la query sugerida**: el hook no sabe reformular; inyectar el prompt como query
  «sugerida» es inyectar lo que ya buscó. Descartada.
- **MCP tool / skill para que buscar sea «más obvio»**: el MCP propio se cerró en M6-06 §0
  (§3.1 del cierre en régimen); no lo reabro por un problema que tiene fix de 90 B. Una skill
  también la invoca el modelo: misma decisión, mismo sesgo.
- **`exo search` con snippet en salida humana**: ataca la forma de la herramienta (§1.3) y
  haría que `search` compitiera con grep en «ver algo en un paso». Toca el engine; lo anoto
  como candidato de backlog, no como parte de esto.
- **Allowlist `Bash(exo *)`**: sí, ahora (§0.2). Es deuda de M6-05, no una idea nueva.

---

## 4. Cómo se verifica que funciona (falsabilidad)

Dos capas, porque son dos preguntas distintas:

**A. ¿El hook emite lo que dice emitir?** — determinista, en `test-recall-inject.sh`, ciclo
rojo-verde-mutación (§3, opción 1). Corre en CI en los tres SO vía `test-plugin.sh`.

**B. ¿El modelo cambia de conducta?** — no determinista, se mide sobre uso real con un script
`evals/recall-conducta/…` (o donde viva la campaña A) que replique `mine2.py`:

- **Unidad**: sesión con ≥3 turnos humanos, con `recall-inject-emitted`, fuera del repo `exo`
  y de corridas aisladas.
- **Métrica primaria**: `primer acceso a KB ∈ {exo search, grep/Read, ninguno}`.
- **Métrica secundaria**: ratio greps-sobre-KB / `exo search` por sesión.
- **Línea base pre-registrada (hoy)**: 5/13 (38%) y 3,4:1.
- **Predicción si la opción 1 funciona**: ≥70% en las siguientes 13 sesiones sustantivas y
  ratio <1,5:1. **Gatillo de escalada a la opción 3**: <60% a las 13 sesiones o a las tres
  semanas, lo que llegue antes.
- **Confusores declarados**: cambio de modelo entre ventanas; sesiones donde Paul pide grep
  explícitamente (anotarlas, no filtrarlas a mano); instalación de 0.2.0 a mitad de ventana
  (cambia el default de `--type`, anotar la fecha).

Esto es exactamente lo que el hook `PreToolUse` mediría en vivo, pero sin tocar producción
ni añadir latencia. Si después hace falta el gate, el instrumento ya está y sirve para
medirlo a él.

---

## 5. Evidencia en la literatura

> **Estado de esta sección**: la verificación delegada (agente `papers-issue23`, con
> WebSearch/WebFetch) **no había vuelto** cuando se cerró el informe. Lo de abajo son citas
> de memoria del consultor: títulos y IDs de arXiv que creo correctos, **sin haber abierto
> hoy ninguna URL**. Antes de citarlas en el issue, verificar cada una. Lo que dice cada
> paper va en «mide»; lo que es inferencia mía va en «para nosotros».

**Posición de la instrucción en el contexto (soporta la opción 1)**

- Liu et al., *Lost in the Middle: How Language Models Use Long Contexts*, TACL 2024,
  arXiv:2307.03172. Mide: el rendimiento en QA multi-documento cae en forma de U según la
  posición de la información relevante; mejor al principio y al final. Para nosotros: una
  regla en el system prompt, a KBs de distancia, está en la zona mala; el pie del bloque,
  pegado al prompt, en la buena. Es inferencia: el paper mide *información*, no
  *instrucciones*.
- Guo & Vosoughi, *Serial Position Effects of Large Language Models*, 2024,
  arXiv:2406.15981. Mide: primacía y recencia en LLMs, robustas a prompting. Misma
  inferencia que arriba.
- Li et al., *Measuring and Controlling Instruction (In)Stability in Language Model
  Dialogs*, COLM 2024, arXiv:2402.10962. Mide: la adherencia a instrucciones del system
  prompt decae a lo largo de turnos («instruction drift»), y un «split-softmax» o repetir la
  instrucción la restaura. Para nosotros: es el argumento más directo a favor de re-inyectar
  la instrucción por turno, que es lo que `UserPromptSubmit` permite hacer gratis.

**La prosa no basta por sí sola (soporta medir y tener escalada)**

- Zhou et al., *Instruction-Following Evaluation for Large Language Models* (IFEval), 2023,
  arXiv:2311.07911. Mide: tasas de cumplimiento de instrucciones verificables lejos del
  100% incluso en modelos punteros.
- Mu et al., *Can LLMs Follow Simple Rules?* (RuLES), 2023, arXiv:2311.04235. Mide: los
  modelos rompen reglas explícitas del system prompt bajo presión del usuario o del
  contexto. Para nosotros: una instrucción cercana sube la tasa, no la lleva a 1; de ahí el
  gatillo de escalada de §4.

**Contexto parcial que «parece suficiente» (explica el fenómeno)**

- Joren et al., *Sufficient Context: A New Lens on Retrieval Augmented Generation Systems*,
  ICLR 2025, arXiv:2411.06037. Mide: con contexto *insuficiente*, los modelos tienden a
  responder igual en vez de abstenerse, y el error sube. Para nosotros: el bloque del hook es
  contexto insuficiente por construcción (una query, top-3) y el modelo lo trata como
  suficiente; declarar el alcance es atacar justo eso.
- Shi et al., *Large Language Models Can Be Easily Distracted by Irrelevant Context*, ICML
  2023, arXiv:2302.00093. Mide: material irrelevante en el prompt degrada el razonamiento.
  Para nosotros: justifica conservar la licencia de ignorar.
- Cuconasu et al., *The Power of Noise*, SIGIR 2024, arXiv:2401.14887. Mide: documentos
  recuperados relacionados-pero-no-útiles perjudican más que ruido aleatorio. Para
  nosotros: el ruido «semánticamente adyacente» que M6-06 §1 ya midió es el caso malo.

**Decidir cuándo recuperar**

- Mallen et al., *When Not to Trust Language Models*, ACL 2023, arXiv:2212.10511;
  Asai et al., *Self-RAG*, ICLR 2024, arXiv:2310.11511; Jeong et al., *Adaptive-RAG*,
  NAACL 2024, arXiv:2403.14403. Miden: la decisión de recuperar (o recuperar más) la toma
  mal el modelo por defecto y hay que entrenarla o gatearla con un clasificador externo.
  Para nosotros: es el argumento de fondo de la opción 3 (gate externo) si la 1 no llega.
- Huang et al., *MetaTool*, ICLR 2024, arXiv:2310.03128. Mide: los LLM fallan en «¿hace
  falta una herramienta?» y en elegir entre herramientas solapadas.

**Forma de la herramienta**

- Yang et al., *SWE-agent: Agent-Computer Interfaces Enable Automated Software
  Engineering*, NeurIPS 2024, arXiv:2405.15793. Mide: cambiar la interfaz de las
  herramientas (búsqueda que devuelve resúmenes acotados, etc.) cambia la tasa de éxito
  del mismo modelo. Para nosotros: sostiene §1.3 —`Grep` nativo con contenido frente a
  `exo search` con punteros— y el candidato de backlog «snippet en la salida humana».

**Lo que la literatura sugiere (inferencia mía)**: la instrucción cerca del punto de uso
y repetida por turno es la palanca barata con mejor respaldo; la prosa por sí sola no
llega a 100%, así que se mide; y si el modelo sigue decidiendo mal «cuándo buscar más», la
literatura de RAG adaptativo dice que la decisión se saca del modelo (gate externo), que
es la opción 3.

---

## 6. Mecánica del harness (lo que admite cada evento)

> **Estado**: el agente `guide-hooks-issue23` (docs oficiales) **no había vuelto** al cerrar.
> Lo siguiente es lo que el propio repo ya tiene verificado más lo que recuerdo de la doc;
> lo marcado «a confirmar» hay que contrastarlo con la doc de hooks antes de diseñar la opción 3.

- `UserPromptSubmit`: exit 2 **borra el prompt** (verificado en M6-06 P1); stdout plano con
  exit 0 se inyecta como contexto (P6); `hookSpecificOutput.additionalContext` se añade al
  turno sin tocar el prompt. La opción 1 no cambia nada de esto.
- `PreToolUse`: recibe `session_id`, `transcript_path`, `tool_name`, `tool_input`
  (`command` para Bash; `pattern`/`path` para Grep). Admite `permissionDecision`
  (`allow|deny|ask`) con `permissionDecisionReason`; `additionalContext` en PreToolUse
  existe en versiones recientes (**a confirmar** en la doc antes de apostar el diseño de la
  opción 3 a ello; si no, el aviso no bloqueante se haría con `ask` y razón, que interrumpe a
  Paul y no es lo que queremos).
- `transcript_path` apunta al `.jsonl` de la sesión y está al día en el momento del hook
  (**a confirmar**): es lo que hace innecesario un fichero de estado.
- `claude plugin eval`: define casos prompt→asserts y puede asertar sobre las tool calls
  del transcript; corre headless (**detalle del formato a confirmar**).
- Permisos: en `defaultMode: auto` no hay prompt; en `default`, `Bash(exo search:*)` en
  `permissions.allow` es la forma de allowlistar por prefijo.

---

## 7. Lo que este dictamen acepta a cara descubierta

- La tasa base es **n=13**, etiquetada por regex, sobre transcripciones del hilo principal
  (los subagentes viven en ficheros aparte y no cuentan: una sesión donde el padre delega la
  búsqueda a un subagente que sí ejecuta `exo search` sale como «grep primero»). Es
  suficiente para decidir el mínimo, no para publicar un número.
- «Copia el literal cuando lo tiene» es una observación sobre 23 comandos, no un experimento.
- El pie de 89 B se escribió y midió hoy (suite actual: 64/64 verde como línea base); la redacción exacta la puede afinar quien
  implemente, con la restricción dura de **≤120 B y sin perder `ignóralo si no aplica` ni
  `--type hybrid`**.
- Si Paul instala 0.2.0 (hybrid por defecto), el `--type hybrid` del pie sobra pero no
  estorba; se deja hasta que el `--help` del binario instalado diga otra cosa.
