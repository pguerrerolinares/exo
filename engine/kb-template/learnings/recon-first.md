---
permalink: "{{KB_NAME}}/learnings/recon-first"
title: En terreno desconocido, verificar el supuesto antes de seguir computando
tags: [agentes, depuracion, verificacion, recon]
tier: stable
semilla: true
---

# En terreno desconocido, verificar el supuesto antes de seguir computando

Cuando algo falla de una forma que no se entiende, o el mismo error se repite
tras varios intentos, la reacción por defecto es probar la siguiente
variación: otro parámetro, otro enfoque, otra línea. Esa reacción asume que el
modelo mental de partida es correcto y que solo falta el ajuste fino. Con
frecuencia esa asunción es justo lo que está mal, y ningún ajuste sobre un
supuesto falso va a funcionar. **Retrieve > compute** para lo que no está en
tus pesos: reintentar lo mismo a ciegas no es progreso.

Lo implementan `exo:recon-first` (invocable directo cuando notas que das
vueltas) y `exo:debug` (root cause y recon antes de proponer fixes).

## La regla

Ante terreno desconocido o varios intentos fallidos seguidos, parar de
computar y hacer una pasada de reconocimiento: buscar qué se sabe ya, leer la
documentación o el código relevante, comprobar el supuesto que se da por
sentado. Solo después de verificarlo tiene sentido seguir intentando
soluciones sobre él.

Cada intento fallido sobre un supuesto equivocado cuesta tiempo y no reduce la
incertidumbre real: confirma, como mucho, que esa variación no basta, no que
el enfoque esté bien orientado. Una verificación dirigida sí la reduce.

## Gate de dificultad: cuándo toca

No es para toda tarea: explorar con buenos priors solo añade ruido y coste de
uso de herramientas. Aplica cuando se da alguna de estas:

- **≥3 intentos contra el mismo error** sin avanzar, aunque el intento cambie.
- **Terreno desconocido**: una API, librería o dominio que no dominas.
- **Time-box quemándose** sin señal: sensación de dar vueltas, no de progresar.

**Sin gate, no pares por ritual.** Con priors sólidos y avance real, sigue.

## Fallo en cadena: consultor limpio

Si **≥2 hipótesis fallan seguidas**, el sospechoso deja de ser el detalle y
pasa a ser un **supuesto compartido no verificado** que todas las hipótesis
heredan. La jugada no es "otra opinión" ni "otro modelo" sino **otro punto de
partida**: un agente fresco que no comparta tu marco.

- **Qué se le da**: solo (a) el problema crudo y sus fuentes primarias, (b) las
  herramientas, (c) la orden de verificar ejecutando, y (d) "si te falta un
  dato, pregunta; no asumas".
- **Qué NO se le da**: tus hipótesis, los candidatos descartados ni tu marco.
  **Su ventaja es la ignorancia de tu error**; pasárselos la anula.
- **Qué se hace con lo que devuelve**: es una hipótesis a comprobar, no una
  orden. Ver [[La lectura verifica coherencia; solo la ejecución verifica verdad|learnings/verificar-ejecutando-no-leyendo]].

## Sospecha del harness antes que del diseño

Cuando el resultado contradice lo esperado (un algoritmo simple supera
sistemáticamente a uno complejo, o un diseño razonable "no funciona"), **mira
primero el entorno de evaluación** antes de culpar al diseño: que el simulador
reproduzca la mecánica real, que haya muestras suficientes para converger y
que la señal de recompensa o la métrica no esté corrompida.

- **Valida el instrumento contra un resultado real conocido** antes de
  tomar decisiones de arquitectura con él. Ver [[Un negativo no vale si el instrumento no está validado|learnings/instrumento-validado-antes-de-medir]].
- **Dependencia rota ≠ diseño roto.** Separa "el diseño es erróneo" de "la
  dependencia falla": si lo que falla es la dependencia (un bug de runtime,
  una latencia), el diseño puede reimplementarse sin ella. Descartar el
  diseño por un fallo de infraestructura tira lo bueno con lo malo.

## Antes del primer fix, razona los edge cases

Cuando alguien cuestiona algo ("¿no será un bug?", "¿tiene sentido?"), el
recon también es mental: antes de aplicar lo primero que se te ocurra,
razona las implicaciones (reintentos, concurrencia, recarga, errores
transitorios) y expón el trade-off con el caso concreto que lo decide.

## Cuándo parar de auditar

Mira la **naturaleza de los hallazgos, no su número**. En las primeras rondas
salen desalineaciones de diseño o de estadística (un test que no prueba la
hipótesis del estimador): los tests nunca las cazan, porque codificarían el
mismo malentendido que el código. En rondas posteriores salen cableado y
contabilidad (un `KeyError`, un off-by-one, un desempate): eso lo caza la
suite al primer intento y más barato. Cuando los hallazgos migran de lo
primero a lo segundo, la revisión estática ya hizo lo único que solo ella
puede hacer; seguir puliendo da rendimiento decreciente, y toca implementar.
Excepción que no migra: la integridad de lo pre-registrado (¿el plan
implementa el spec firmado?) es invisible para los tests y se cierra antes de
la primera medición.

## Cuándo no aplica

En terreno ya conocido, con un modelo mental verificado antes y todavía
válido, iterar directamente es más eficiente que reconfirmar cada vez. La
regla es para la incertidumbre real, no para convertir cada tarea rutinaria
en una investigación. Y el recon tiene coste: el consultor limpio se reserva
para el fallo en cadena, no para cada duda. Doctrina general en
[[Doctrina de trabajo con agentes|core/doctrina]].
