---
name: plan
description: Usa cuando tienes una spec o requisitos para una tarea multi-paso, antes de tocar código. Produce un plan-contrato de tareas (ficheros, interfaces, tests con aserciones, sin cuerpos de código) para un ingeniero capaz que conoce la interfaz y el test.
---

# plan

El plan es un contrato: el conjunto de decisiones que el executor no puede
tomar solo. El lector es un ingeniero capaz que, con la interfaz y el test
delante, escribe el cuerpo. Guarda el plan en
`docs/superpowers/plans/YYYY-MM-DD-<feature-name>.md` (la preferencia del
usuario sobre ubicación gana).

## Antes de las tareas

- Scope check: si la spec cubre varios subsistemas independientes, sepáralos
  en planes distintos: cada uno produce software funcionando y testeable.
- Mapea la file structure antes de definir tareas: qué se crea o modifica y la
  responsabilidad de cada fichero. Límites claros; lo que cambia junto vive
  junto.
- Tarea = unidad con resultado verificable y su propio ciclo de test, digna
  del gate de un reviewer fresco. Diff esperado ≲ 400 líneas. Las triviales
  (setup, scaffolding, un rename) se pliegan en la tarea que las necesita.
- El wiring (registrar, exportar, conectar) va en una sola tarea por ola, no
  repartido: es lo que hace chocar a las tareas paralelas.

## Header y estructura de tarea

Header: Goal (1 frase), Architecture (2-3 frases), Tech Stack, `Spec: <ruta>`
y Global Constraints con valores exactos copiados verbatim de la spec; toda
tarea los hereda. Sección "Olas" (abajo). Pointer "For agentic workers": la
skill de ejecución es `exo:orchestrate`.

Cada tarea, con encabezado literal `### Task N: <nombre>` (lo parsea
`orchestrate/scripts/task-brief`): `Files`, `Interfaces`, `Tests` y `Notas`.
`Review Focus` va una vez, a nivel de plan. Formato parseable de
`Files`/`Interfaces` y plantilla en `plan-template.md`.

## Qué contiene una tarea

Una tarea está hecha cuando el executor puede escribir una sola cosa
razonable a partir de ella. Inequívoca, no completa.

- **Test:** nombre + aserción con los valores exactos de la spec. La
  aserción declara el fallo que caza ("falla si el descuento se aplica dos
  veces"). Sin ese fallo nombrado, es un ritual, no un test.
- **Código:** firma, fichero y valores que fija la spec. El executor escribe
  el cuerpo. `Notas` lleva un algoritmo solo si firma y tests no lo
  determinan, o copy exacto que fija la spec.
- **Verificación:** el comando y el output que significa "pasó".
- **Otra tarea:** se referencia por su `Interfaces`; no se repite su código.
- **Review Focus:** ≤5 inputs o condiciones que la spec implica y nadie
  nombra, más probable primero. Cada uno lleva su test en la tarea dueña.

Nunca: "TBD/TODO", "add error handling" o "handle edge cases" sin decir cuál,
"write tests for the above" sin nombre ni aserción, tipos o funciones que
ninguna tarea define.

## Olas

Sección del plan que agrupa tareas. Regla: dos tareas comparten ola si sus
`Files` son disjuntos y ninguna consume, directa o transitivamente, algo que
produce la otra. `orchestrate` ejecuta ola a ola.

## Self-review y handoff

Con ojos frescos contra la spec, checklist propio (no dispatch):
1. Cobertura: ¿cada requisito tiene tarea?
2. Placeholders: nada de lo anterior.
3. Tipos: nombres y firmas coinciden entre tareas.
4. Proporción: "un plan más largo que el código que describe ya es el
   código". Si dominan los bloques de código, sustituye cuerpos por firma +
   nombre de test + aserción y comprueba que cada tarea sigue siendo
   inequívoca.

Fix inline. Handoff único: `exo:orchestrate`.
