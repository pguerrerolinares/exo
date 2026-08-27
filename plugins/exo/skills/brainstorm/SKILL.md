---
name: brainstorm
description: Usa antes de cualquier trabajo creativo — nueva feature, componente, funcionalidad o cambio de comportamiento. Explora intención, requisitos y diseño en diálogo antes de implementar; termina en una spec escrita, auto-revisada y aprobada por el usuario.
---

# brainstorm

Explora intención, requisitos y diseño en diálogo colaborativo antes de
implementar. Termina invocando `process:plan` — nunca código.

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

No invoques ninguna skill de implementación, ni escribas código, hasta que el
usuario apruebe el diseño. Aplica a TODO proyecto, sin importar cuán simple
parezca — "demasiado simple para necesitar diseño" es la trampa más común: el
diseño puede ser corto, pero se presenta y se aprueba siempre.

## Después del diseño

- Escribe la spec validada a
  `docs/superpowers/specs/YYYY-MM-DD-<topic>-design.md` (la preferencia del
  usuario sobre ubicación gana) y commitéala.
- Self-review con ojos frescos: placeholders, consistencia interna, scope,
  ambigüedad. Arregla inline — no hace falta re-revisar.
- Gate de review del usuario: pídele que revise la spec escrita y espera su
  respuesta antes de seguir.
- Estado terminal: invoca `process:plan`. Ninguna otra skill.

## Principios

YAGNI: quita features innecesarias de todo diseño.
