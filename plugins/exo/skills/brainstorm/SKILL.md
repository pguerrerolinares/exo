---
name: brainstorm
description: Usa antes de cualquier trabajo creativo — nueva feature, componente, funcionalidad o cambio de comportamiento. Explora intención, requisitos y diseño en diálogo antes de implementar; termina en una spec escrita, auto-revisada y aprobada por el usuario.
---

# brainstorm

Explora intención, requisitos y diseño en diálogo colaborativo antes de
implementar. Termina invocando `exo:plan` — nunca código.

Entendimiento compartido y gate por etapa destilados de `brainstorming/SKILL.md`
(superpowers 6.4.2, MIT © 2025 Jesse Vincent).

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
- Propón 2-3 enfoques con trade-offs, liderando con tu recomendación y su
  porqué.
- Presenta el diseño por secciones escaladas a su complejidad; valida cada
  sección con el usuario antes de seguir a la siguiente. Cubre: arquitectura,
  componentes, data flow, error handling, testing.
- Diseña para aislamiento: unidades con un propósito claro, interfaces bien
  definidas, comprensibles y testeables por separado.
- En codebases existentes: sigue los patrones actuales; mejoras targeted solo
  si afectan al trabajo — no propongas refactoring no relacionado.

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
- Self-review con ojos frescos: placeholders, consistencia interna, scope,
  ambigüedad. Arregla inline — no hace falta re-revisar.
- Gate de review del usuario: pídele que revise la spec escrita y espera su
  respuesta antes de seguir.
- Estado terminal: invoca `exo:plan`. Ninguna otra skill.

## Principios

YAGNI: quita features innecesarias de todo diseño.
