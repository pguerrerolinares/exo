---
name: brainstorm
description: Usa antes de cualquier trabajo creativo — nueva feature, componente, funcionalidad o cambio de comportamiento. Explora intención, requisitos y diseño en diálogo antes de implementar; termina en una spec escrita, auto-revisada y aprobada por el usuario.
---

# brainstorm

Explora intención, requisitos y diseño en diálogo colaborativo antes de
implementar. Termina invocando `exo:plan` — nunca código.

Entendimiento compartido y gate por etapa destilados de `brainstorming/SKILL.md`
(superpowers 7.0.1, MIT © 2025 Jesse Vincent).

## Entendimiento compartido

El resultado es un entendimiento que el usuario pueda reconocer y corregir,
anclado en lo que quiere lograr.

1. **Descubre la intención:** resultado buscado, para quién, cómo se ve el
   éxito. Si falta, UNA pregunta sobre propósito o uso antes de proponer
   features o enfoque: conocer el género de la app no dice por qué la quiere.
2. **Devuélvelo por tramos:** cuando una parte quede clara (para qué y para
   quién; cómo se comporta una pieza), devuélvela en unas líneas antes de
   seguir preguntando, y alterna así hasta el final. Separa lo que dijo de
   lo que supones: sus palabras como suyas, lo que rellenaste como propuesta
   tuya con su razón (una opción elegida de un menú confirma el qué, no el
   porqué).
   Cierra cada tramo preguntando qué falla o falta, e incorpora la
   respuesta antes de tratar el conjunto como brief del diseño.
3. **Llévalo al diseño:** conserva ese entendimiento en el diseño y contrasta
   con él cada feature y decisión técnica.

Si la petición ya trae propósito y restricciones, refléjalos en vez de
repreguntar. Cada tramo es breve; importa que sea exacta y corregible.

## Proceso

- Mira antes de preguntar: en un proyecto existente, o si el usuario trajo
  contexto (spec, plan, handoff, ficheros, conversación previa), lee lo que
  citó y el código o material que toca, hasta poder explicar cómo funcionan
  hoy las partes relevantes y qué tocaría el cambio. Nunca preguntes lo que
  el proyecto o su contexto ya responde.
- Si ya dijo lo que quiere (spec, plan, petición detallada), el primer
  mensaje (~200 palabras) devuelve su petición con tus palabras (qué, para
  quién y por qué, hasta donde lo dijo) y añade lo que hallaste: cómo
  funciona hoy y qué tocaría el cambio; si algo que quizá no vio cambia lo
  pedido (restricción, conflicto, camino más barato), dilo claro. Cierra con
  una línea que invita a corregir y UNA pregunta: lo que su petición deja
  abierto y más cambiaría lo construido. Una spec suele estar incompleta:
  confírmala antes de construir sobre ella.
- Antes de refinar detalles, evalúa el scope: si la petición describe varios
  subsistemas independientes, decompón en sub-proyectos, cada uno con su
  propio ciclo spec→plan→implementación. No gastes preguntas en detalles de
  un proyecto que necesita descomponerse primero.
- Preguntas de una en una; si un tema pide más, pártelo en varias preguntas.
  Objetivo: purpose, constraints, success criteria. Prefiere multiple choice
  cuando sea posible. Cada pregunta, específica del proyecto y de lo que dijo,
  encadenada a su última respuesta o a un hallazgo, de modo que le haga pensar
  una consecuencia o un caso que quizá no vio; si la razón no es obvia, di qué
  esperas aprender o qué cambiaría la respuesta. Una propuesta corregible con
  su porqué suele valer más que una pregunta.
- Con algo de base, ofrece ir a mirar más a fondo (su codebase o proyecto, sus
  ficheros y datos, la web): di qué buscarías; decide él si vas.
- Cómo se hace: todo proyecto descansa en elecciones de medio, herramientas y
  recursos (plataforma, lenguaje y librerías de una app). Averigua pronto
  cuáles quiere tomar el usuario, cuáles te delega y con qué cuenta ya. Quien
  tiene criterio quiere hablarlo; quien cuenta contigo necesita defaults
  sensatos, dichos en voz alta (un default que no oyó no pudo rechazarlo).
  Con trabajo existente, construye sobre lo que ya usa. Toda elección tuya va
  en el diseño como decisión tuya, nunca como algo acordado.
- Enfoques: cuando entiendas lo que quiere y antes de escribir nada, idea
  varias alternativas reales y descarta las débiles (también las que van
  contra su porqué y las que el trabajo existente excluye). Antes del mensaje,
  despacha SIEMPRE un consultor con `consultor-brief.md` que elija entre las
  2-3 que queden, ancladas en lo que el proyecto ya hace; el padre no puede
  juzgar si el usuario tiene criterio (UX, a11y, rendimiento, CI incluidos).
  El mensaje, propio, lleva el veredicto del consultor, el trade-off de las
  demás vías y su condición de invalidación: el usuario firma o corrige, no
  elige de un menú. Si solo hay una vía sensata, dilo y explica por qué las
  demás no encajan. Es para el cómo; el qué (la intención) lo describe él
  primero y sigue siendo suyo: esas preguntas llevan `[reserva: hecho-de-Paul]`
  si casan el detector.
- Presenta el diseño por secciones escaladas a su complejidad; valida cada
  sección con el usuario antes de seguir a la siguiente (el cierre de esa
  validación empieza por `[reserva: pedido]`: lo pide este skill, no el
  hilo). Cubre: arquitectura,
  componentes, data flow, error handling, testing.
- Diseña para aislamiento: unidades con un propósito claro, interfaces bien
  definidas, comprensibles y testeables por separado.
- En codebases existentes: sigue los patrones actuales; mejoras targeted solo
  si afectan al trabajo — no propongas refactoring no relacionado.

## Tamaño del trabajo

Una vez entendida la intención y elegido el enfoque, elige un tamaño y dilo en
claro (el usuario puede cambiarlo): **cambio pequeño** ⇒ la descripción
completa va en el chat; **proyecto** ⇒ spec escrita (abajo), con una parte
**Enfoques**: las 2-3 formas consideradas, su trade-off, cuál eligió el
usuario y por qué. Si crece a mitad de camino, para, dilo y sube de tamaño. El
gate aplica a ambos: construir exige aprobación de la descripción completa.

## Gate: diseño antes de código

No invoques ninguna skill de implementación, ni escribas código, ni scaffold,
ni instales dependencias de producto, hasta que el usuario apruebe el diseño y
luego revise la spec escrita; y no escribas la spec sin haber puesto antes los
enfoques en su propio mensaje. Aplica a TODO proyecto, sin importar cuán simple
parezca — "demasiado simple para necesitar diseño" es la trampa más común: el
diseño puede ser corto, pero se presenta y se aprueba siempre. Los cierres
que piden esa aprobación (diseño, spec escrita) empiezan por
`[reserva: pedido]`: es este gate quien pide la firma.

Una respuesta aprueba la etapa que realmente se presentó: aprobar una idea o
su alcance no aprueba artefactos que aún no existen; el visto bueno del
diseño conversacional solo permite escribir la spec, y el de la spec escrita
solo permite invocar `exo:plan`. Retoma en la primera etapa incompleta. La
exploración de solo lectura del proyecto sigue permitida.

## Después del diseño

- Escribe la spec validada a
  `docs/superpowers/specs/YYYY-MM-DD-<topic>-design.md` (la preferencia del
  usuario sobre ubicación gana) y commitéala.
- Builder check en lugar de self-review: despacha un subagente fresco que lea
  la spec como un builder competente del dominio que no puede hablar con el
  usuario y devuelva solo las preguntas que aún necesitaría para planificar
  (dónde tendría que adivinar y adivinar mal importa, con por qué cambia el
  plan; ordenadas por importancia; «nada importante» si no hay). Avisa al
  usuario de que un reviewer la lee antes y puedes volver una vez con
  preguntas. Clasifica cada una: **ya respondida** (por la conversación o el
  proyecto: ponla en la spec), **menor** (un builder sensato la cierra sin
  cambiar lo construido: decídela tú y márcala como decisión tuya) o **del
  usuario** (solo él puede responder y cambia lo construido). Las del usuario
  van en un solo mensaje, la más importante primero; actualiza la spec y
  entrégala con la lista corta de decisiones tuyas. Es la única ronda de
  preguntas del check. Sin herramienta de subagentes, léela tú como ese builder.
- Si la spec descansa en hechos comprobables (defaults de librería, cifras,
  límites de plataforma o de la forja), review adversarial con ejecutables
  antes del plan: consultor fresco con `consultor-brief.md` que verifique esas
  premisas (fuente, bytecode, recalcular) en vez de leerlas.
- Gate de review del usuario: pídele que revise la spec escrita y espera su
  respuesta antes de seguir.
- Estado terminal: invoca `exo:plan`. Ninguna otra skill.

## Principios

YAGNI: quita features innecesarias de todo diseño.
