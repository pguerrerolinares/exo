---
name: plan
description: Usa cuando tienes una spec o requisitos para una tarea multi-paso, antes de tocar código. Produce un plan de tareas bite-sized con paths, código y comandos exactos, pensado para un ejecutor sin contexto.
---

# plan

Asume un ingeniero hábil con CERO contexto del codebase y del dominio.
Documenta ficheros a tocar, código, testing y docs que necesite. Guarda el
plan en `docs/superpowers/plans/YYYY-MM-DD-<feature-name>.md` (la preferencia
del usuario sobre ubicación gana).

## Antes de las tareas

- Scope check: si la spec cubre varios subsistemas independientes, sepáralos
  en planes distintos — cada uno debe producir software funcionando y
  testeable por sí solo.
- Mapea la file structure antes de definir tareas: qué ficheros se crean o
  modifican y la responsabilidad de cada uno. Unidades con límites claros;
  ficheros que cambian juntos viven juntos.
- Task right-sizing: la tarea es la unidad mínima con su propio ciclo de test
  y digna del gate de un reviewer fresco. Pliega setup/scaffolding en la
  tarea que los necesita.
- Cada paso es una acción de 2-5 min: failing test → verlo fallar →
  implementación mínima → verlo pasar → commit.

## Header y estructura de tarea

Header obligatorio: Goal (1 frase), Architecture (2-3 frases), Tech Stack, y
Global Constraints con valores exactos copiados verbatim de la spec — toda
tarea los hereda implícitamente. Incluye el pointer "For agentic workers": la
skill de ejecución es `process:orchestrate`, pasos con checkbox (`- [ ]`)
para tracking.

Cada tarea: Files (Create/Modify/Test con paths exactos) + Interfaces
(Consumes/Produces con firmas exactas — el implementer solo ve su tarea; así
aprende los nombres y tipos vecinos). Plantilla completa en
`plan-template.md`.

## No-placeholders

Nunca: "TBD/TODO", "add appropriate error handling" sin código, "write tests
for the above" sin el test, "similar to Task N" (repite el código en vez de
referenciarlo), pasos sin el cómo, o referencias a tipos/funciones no
definidos en ninguna tarea. Paths exactos siempre, código completo en cada
paso que cambia código, comandos exactos con output esperado. DRY, YAGNI,
TDD, commits frecuentes.

## Self-review y handoff

Con ojos frescos contra la spec (checklist propio, no dispatch): cobertura
(¿cada requisito tiene tarea?), placeholder scan, consistencia de tipos y
firmas entre tareas. Fix inline; si falta una tarea, añádela.

Al terminar, handoff de ejecución a `process:orchestrate` — único destino.
