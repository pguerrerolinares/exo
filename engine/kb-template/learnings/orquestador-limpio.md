---
permalink: "{{KB_NAME}}/learnings/orquestador-limpio"
title: Un orquestador delega la lectura pesada y se queda solo con la conclusión
tags: [agentes, orquestacion, contexto]
tier: stable
semilla: true
---

# Un orquestador delega la lectura pesada y se queda solo con la conclusión

Cuando una tarea exige leer mucho material para llegar a una conclusión
pequeña — explorar un repo grande, comparar varias fuentes, recorrer un log
extenso — el instinto es hacer esa lectura en el mismo hilo que coordina el
resto del trabajo. Es el error que más rápido degrada a un agente que
orquesta: el contexto de coordinación es un recurso limitado y compartido por
todo lo que viene después, y llenarlo de material crudo que solo hacía falta
una vez deja menos sitio para lo que sí hace falta todo el rato.

## Por qué se sostiene

Un subagente lanzado para investigar puede leer todo lo que necesite, probar
caminos que no llevan a nada, y descartar el 90% de lo que miró — y nada de
eso vuelve al que coordina. Lo único que vuelve es la respuesta a la pregunta
que se le hizo. El coordinador se queda con una conclusión y con contexto
libre para seguir coordinando; el subagente se queda con un hilo que termina
y desaparece.

## Cuándo aplica

- Investigación abierta: "¿qué hace este módulo?", "¿dónde está definido X?",
  comparar varias alternativas.
- Lectura voluminosa cuyo resultado útil es mucho más corto que el material
  que hubo que leer para llegar a él.
- Cualquier exploración donde una parte significativa del esfuerzo se va a
  descartar de todas formas.

## Cuándo no aplica

- Ediciones triviales y acotadas, donde delegar cuesta más coordinación de la
  que ahorra.
- Cuando el propio coordinador necesita el detalle fino para tomar la
  siguiente decisión, no solo la conclusión — ahí delegar sin más pierde
  información que hacía falta.

## Matiz

Delegar no es "no leer nunca nada": es no acumular en el hilo que coordina
lo que no se va a volver a necesitar. Si el coordinador va a citar un
fragmento concreto más adelante, ese fragmento sí debe quedarse; lo que no
debe quedarse es el rastro completo de cómo se llegó hasta él.
