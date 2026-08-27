---
permalink: "{{KB_NAME}}/learnings/recon-first"
title: En terreno desconocido, verificar el supuesto antes de seguir computando
tags: [agentes, depuracion, verificacion]
tier: stable
semilla: true
---

# En terreno desconocido, verificar el supuesto antes de seguir computando

Cuando algo falla de una forma que no se entiende, o el mismo error se repite
tras varios intentos, la reacción por defecto suele ser probar la siguiente
variación: otro parámetro, otro enfoque, otra línea de código. Esa reacción
asume que el modelo mental de partida es correcto y que solo falta encontrar
el ajuste fino. Con frecuencia esa asunción es justo lo que está mal, y
ningún ajuste fino sobre un supuesto falso va a funcionar.

## La regla

Ante terreno desconocido o varios intentos fallidos seguidos, parar de
computar y hacer una pasada de reconocimiento: buscar qué se sabe ya sobre
esto, leer la documentación o el código relevante, comprobar el supuesto que
se está dando por sentado. Solo después de verificar el supuesto tiene
sentido seguir intentando soluciones sobre él.

## Por qué se sostiene

Cada intento fallido sobre un supuesto equivocado cuesta tiempo y no reduce
la incertidumbre real: confirma, como mucho, que esa variación concreta no
basta, no que el enfoque general esté bien orientado. Una búsqueda o una
verificación dirigida, en cambio, sí reduce la incertidumbre — dice si el
problema está donde se piensa que está. El coste de parar a mirar es casi
siempre menor que el coste del siguiente intento a ciegas, sobre todo a
partir del segundo o tercer fallo con el mismo patrón.

## Señales de que toca parar

- El mismo tipo de error aparece tres o más veces seguidas, aunque el intento
  haya cambiado.
- El terreno es nuevo: una librería, un sistema o un formato que no se ha
  tocado antes en esta sesión.
- El presupuesto de tiempo se está agotando y la sensación es de estar
  "dando vueltas" más que de progresar.

## Cuándo no aplica

En terreno ya conocido, con un modelo mental que ya se ha verificado antes y
sigue siendo válido, iterar directamente sobre el problema es más eficiente
que parar a reconfirmar cada vez. La regla es para la incertidumbre real, no
para convertir cada tarea rutinaria en una investigación.
