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
`orchestrate/scripts/task-brief`): `Files`, `Interfaces`, `Tests`,
`Verificación`, `Review Focus` y `Notas`. Formato parseable y plantilla en
`plan-template.md`.

## Qué contiene una tarea

Una tarea está hecha cuando el executor puede escribir una sola cosa
razonable a partir de ella. Inequívoca, no completa.

- **Tests:** nombre + aserción con los "valores" de la spec: constantes,
  literales, mensajes de error, nombres de campo/columna y códigos que fija.
  La aserción declara el fallo que caza ("falla si el descuento se aplica
  dos veces"); sin él es un ritual. Cada test se escribe y se ve fallar
  antes del código (`exo:tdd`). Sin tests: `Tests: n/a — <motivo>`.
- **Código:** firma, fichero y valores. El executor escribe el cuerpo. El
  plan no lleva cuerpos de función ni snippets de implementación: solo
  firmas, valores y comandos. Un bloque de código en `Notas` solo vale si es
  texto literal que fija la spec. `Notas` lleva un algoritmo solo si firma y
  tests no lo determinan.
- **Verificación:** comando → output esperado. Verde es la suite entera del
  proyecto, no el fichero del test.
- **Otra tarea:** se referencia por su `Interfaces` (`@Task M`); no se repite
  su código.
- **Review Focus (por tarea):** ≤5 inputs o condiciones que la spec implica
  y nadie nombra, más probable primero, cada uno con su test en esa tarea.

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

Fix inline. Handoff único: `exo:orchestrate`. Al hacer el handoff, añade al
header `Plan: <min> min, <turnos> turnos, <KB>`.
