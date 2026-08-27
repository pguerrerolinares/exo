---
permalink: "{{KB_NAME}}/learnings/fallo-silencioso"
title: El fallo más caro es el que no avisa
tags: [agentes, verificacion, calidad]
tier: stable
semilla: true
---

# El fallo más caro es el que no avisa

Un fallo que sale con un mensaje de error, un test en rojo o un proceso que
termina con código distinto de cero se detecta solo: alguien lo ve y actúa.
El fallo peligroso es el otro tipo: el que tiene forma válida, no dispara
ninguna alarma, y se descubre — si se descubre — mucho después y por
casualidad. Vigilar activamente por este segundo tipo, no solo por el
primero, es una disciplina aparte.

## Formas concretas que toma

- **Degradación con forma válida**: el sistema responde con algo que parece
  correcto — una respuesta bien formada, un fichero con la estructura
  esperada — pero el contenido es un resultado degradado o parcial, no el
  resultado real. Nada en la forma delata que algo falló.
- **Checks no falsables**: una comprobación que, tal como está escrita, no
  podría fallar nunca — coincide siempre, o solo prueba una condición trivial
  en vez de la que importa. Pasa siempre, y por eso no prueba nada.
- **Contrato por prosa**: una regla que solo existe como comentario o
  documentación, sin nada en el código o en un test que la haga cumplir. Se
  puede violar sin que nada lo impida ni lo señale.
- **Exit 0 no es efecto**: que un proceso termine sin error no significa que
  haya producido el efecto esperado. Puede haber terminado sin hacer nada, o
  haber hecho solo una parte, y seguir devolviendo éxito.
- **Composición**: piezas que por separado están bien pueden combinarse de
  una forma que rompe el conjunto, sin que ninguna pieza individual muestre
  ningún síntoma.
- **Ausencia no es evidencia**: que una búsqueda no encuentre nada, o que un
  log no muestre ningún error, no prueba que todo esté bien — puede significar
  que se buscó en el sitio equivocado o que el error no llegó a registrarse.

## Qué hacer con esto

Cuando se diseña una verificación, preguntarse explícitamente: ¿esta
comprobación podría pasar aunque el efecto real no haya ocurrido? Si la
respuesta es sí, no es un gate — es una comprobación que da tranquilidad sin
dar garantía. La forma de exigirse rigor aquí es ver el ciclo completo:
provocar el fallo real, ver a la comprobación detectarlo, y solo entonces
confiar en que detecta lo que dice detectar.
