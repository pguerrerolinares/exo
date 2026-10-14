---
name: brainstorm
description: Usa antes de cualquier trabajo creativo — nueva feature, componente, funcionalidad o cambio de comportamiento. Explora intención, requisitos y diseño en diálogo antes de implementar; termina en una spec escrita, auto-revisada y aprobada por el usuario.
---

# brainstorm

Explora intención, requisitos y diseño en diálogo colaborativo antes de
implementar. Termina invocando `exo:plan` — nunca código.

Entendimiento compartido y gate por etapa destilados de `brainstorming/SKILL.md`
(superpowers 7.0.0, MIT © 2025 Jesse Vincent).

## Entendimiento compartido

El resultado es un entendimiento que el usuario pueda reconocer y corregir,
anclado en lo que quiere lograr.

1. **Descubre la intención:** resultado buscado, para quién, cómo se ve el
   éxito. Si falta, UNA pregunta sobre propósito o uso antes de proponer
   features o enfoque: conocer el género de la app no dice por qué la quiere.
2. **Devuélvelo por escrito:** resumen corto (resultado, restricciones,
   criterios de éxito) separando lo dicho de tus supuestos; invita a
   corregir e incorpora la respuesta antes de tratarlo como brief del diseño.
3. **Llévalo al diseño:** conserva ese entendimiento en el diseño y contrasta
   con él cada feature y decisión técnica.

Si la petición ya trae propósito y restricciones, refléjalos en vez de
repreguntar. La nota es breve; importa que sea exacta y corregible.

## Proceso

- Explora primero el contexto del proyecto: ficheros, docs, commits recientes.
- Antes de refinar detalles, evalúa el scope: si la petición describe varios
  subsistemas independientes, decompón en sub-proyectos, cada uno con su
  propio ciclo spec→plan→implementación. No gastes preguntas en detalles de
  un proyecto que necesita descomponerse primero.
- Preguntas de una en una; si un tema pide más, pártelo en varias preguntas.
  Objetivo: purpose, constraints, success criteria. Prefiere multiple choice
  cuando sea posible.
- Cómo se hace: todo proyecto descansa en elecciones de medio, herramientas y
  recursos (plataforma, lenguaje y librerías de una app). Averigua pronto
  cuáles quiere tomar el usuario, cuáles te delega y con qué cuenta ya. Quien
  tiene criterio quiere hablarlo; quien cuenta contigo necesita defaults
  sensatos, dichos en voz alta (un default que no oyó no pudo rechazarlo).
  Con trabajo existente, construye sobre lo que ya usa. Toda elección tuya va
  en el diseño como decisión tuya, nunca como algo acordado.
- Propón 2-3 enfoques con trade-offs, liderando con tu recomendación y su
  porqué.
- Presenta el diseño por secciones escaladas a su complejidad; valida cada
  sección con el usuario antes de seguir a la siguiente. Cubre: arquitectura,
  componentes, data flow, error handling, testing.
- Diseña para aislamiento: unidades con un propósito claro, interfaces bien
  definidas, comprensibles y testeables por separado.
- En codebases existentes: sigue los patrones actuales; mejoras targeted solo
  si afectan al trabajo — no propongas refactoring no relacionado.

## Tamaño del trabajo

Una vez entendida la intención, elige un tamaño y dilo en claro (el usuario
puede cambiarlo): **cambio pequeño** ⇒ la descripción completa va en el chat;
**proyecto** ⇒ spec escrita (abajo). Si crece a mitad de camino, para, dilo y
sube de tamaño. El gate aplica a ambos: construir exige aprobación de la
descripción completa.

## Gate: diseño antes de código

No invoques ninguna skill de implementación, ni escribas código, ni scaffold,
ni instales dependencias de producto, hasta que el usuario apruebe el diseño y
luego revise la spec escrita. Aplica a TODO proyecto, sin importar cuán simple
parezca — "demasiado simple para necesitar diseño" es la trampa más común: el
diseño puede ser corto, pero se presenta y se aprueba siempre.

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
- Gate de review del usuario: pídele que revise la spec escrita y espera su
  respuesta antes de seguir.
- Estado terminal: invoca `exo:plan`. Ninguna otra skill.

## Principios

YAGNI: quita features innecesarias de todo diseño.
