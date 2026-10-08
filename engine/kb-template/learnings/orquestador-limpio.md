---
permalink: "{{KB_NAME}}/learnings/orquestador-limpio"
title: El padre coordina y valida; el ejecutor implementa
tags: [agentes, orquestacion, contexto, delegacion, coste]
tier: stable
semilla: true
---

# El padre coordina y valida; el ejecutor implementa

El hilo que coordina es un recurso escaso: todo lo que entra en su contexto se
relee en cada turno, y más contexto en el padre es peor rendimiento
(context-rot). Por eso el padre no lee material voluminoso ni implementa: manda
a un subagente, se queda con la conclusión y valida de forma independiente lo
que vuelve. Un subagente puede leer todo, probar caminos muertos y descartar la
mayor parte de lo que miró; nada de eso regresa al padre, solo la respuesta. El hilo
del hijo termina; el del padre sigue ligero.

## Qué delega el padre

- **Lecturas que solo necesitas resumidas.** Investigación abierta, comparar
  alternativas, recorrer un log extenso: a un subagente de exploración. Hacer
  el trabajo voluminoso inline es el anti-patrón.
- **El padre DETECTA, el ejecutor ARREGLA.** Parchear tú un hueco "porque es
  más rápido que un round-trip" es cierto e irrelevante: el coste no es el
  tiempo, es que el material crudo se queda en tu contexto y acabas haciendo
  de ejecutor. Di "esta mutación pasa tu suite, cúbrela", no escribas el test.
  `SendMessage` reanuda al agente con su contexto intacto: más barato que un
  dispatch nuevo y que parchear tú.
- **File handoffs, no paste.** Brief, reporte del hijo y paquete de review
  viajan como ficheros; en el prompt va la ruta. Lo que pegas o lo que un hijo
  devuelve como texto queda residente y se relee cada turno.

## Pirámide de coste

- **`model` explícito siempre.** Omitirlo hereda el del padre y rompe la
  pirámide en silencio.
- **haiku = transcripción** (fix mecánico de un fichero, extracción literal);
  **sonnet = juicio e integración** (multi-fichero, refactors, wiring que
  "parece transcripción" pero compone sistema); **Opus = la review final
  de branch**, una por branch; por excepción, una review de tarea con riesgo
  de concurrencia o seguridad sutil.
- **Turn-count vence a token-price.** Un modelo barato en trabajo ambiguo
  tarda 2-3 veces más turnos y sale más caro. Tier barato solo cuando el texto
  del plan ES el código; tier medio como suelo para reviewers y para hijos que
  trabajan desde prosa.
- **El reviewer escala al riesgo del diff, elegido explícitamente**: literal y
  contenido, barato; wiring de integración, medio; concurrencia o seguridad
  sutil, Opus. Nunca hereda el modelo del padre.

## Qué no se delega al modelo barato

- **El criterio de selección.** Un modelo barato califica con generosidad
  (calificó como dignos de análisis profundo a casi todos los candidatos); el
  orquestador re-decide.
- **La fiabilidad del agregado.** Valida invariantes del conjunto (cobertura
  1:1, unicidad) en vez de confiar ficha a ficha.
- **Los checks estructurales.** Enlaces, frontmatter, existencia de rutas: un
  script determinista vence a un agente (un reviewer perdió su mensaje final
  tras mucho trabajo y no entregó nada).
- **El formato de salida.** Schema forzado con reintento, no texto libre.

## Transporte mecánico

- **El contexto llega por hook al arrancar el subagente**, no por una
  instrucción en el brief que se espera que se cumpla. Ver
  [[Una regla que se cita y se ignora es un comentario|learnings/la-prosa-no-es-enforcement]].
- **Entrega no es uso.** Mide ambas: que el contexto llegó y que se usó.
- **Punteros como rutas de fichero**, nunca URIs de herramientas MCP: los
  agentes usan bien Read/Grep y mal las tools custom.
- **A los agentes de research no se les inyecta digest**: buscar es su
  trabajo.

## Operar con subagentes vivos

- **No pongas polling ni wait-loops caseros en el brief.** Los jobs largos los
  lanza y vigila el padre (con `run_in_background`, que re-invoca al
  terminar). Los ejecutores no lanzan subagentes.
- **Un agente idle tras entrega no es incidencia.** Verifica por artefacto
  (mtime, `git log`), empuja con `SendMessage`, no re-despaches. No infieras
  abandono de un estado intermedio: dos escritores sobre el mismo estado es la
  línea roja.

## El bucle de ejecución de un plan

`exo:orchestrate` implementa este bucle con `exo:executor`.

- **Subagente fresco por tarea**, con solo brief e interfaces previas; ledger
  durable que sobrevive a la compactación.
- **El reviewer detecta; el controlador decide.** Un hallazgo real se escala,
  no se obedece ni se descarta a ciegas.
- **Pre-flight recon antes de la tarea 1**: verifica el plan contra el código
  real.
- **Plan = contrato sin cuerpos de código**: firmas, valores y tests como
  nombre más aserción; el ejecutor escribe el cuerpo. Un plan con código
  completo es implementar dos veces.
- **El controlador re-verifica el ground truth.** Re-corre la suite en los
  puntos de riesgo; el self-report no basta (ver
  [[La lectura verifica coherencia; solo la ejecución verifica verdad|learnings/verificar-ejecutando-no-leyendo]]).
- **Review whole-branch final, siempre.** Caza lo que vive entre tareas: el
  gate nuevo aplicado a sí mismo, la cadena de un mecanismo repartido en
  varias tareas, la promesa del código que el propio código no cumple.
- **El consultor que adjudica un trade-off** debe poder re-encuadrar el
  problema y exigir medición sobre el sistema real, no solo rankear los
  supuestos del brief.

## Cuándo no aplica

- Ediciones triviales y acotadas, donde delegar cuesta más coordinación de la
  que ahorra.
- Cuando el padre necesita el detalle fino para su siguiente decisión, no solo
  la conclusión. Si va a citar un fragmento, ese se queda; lo que no se queda
  es el rastro de cómo se llegó a él.

Relacionadas: [[La claridad del encargo es el cuello de botella, no la capacidad del agente|learnings/el-brief-es-el-cuello-de-botella]],
[[Una regla que se cita y se ignora es un comentario|learnings/la-prosa-no-es-enforcement]],
[[El harness tiene hechos que muerden en producción: verificarlos contra la versión viva|learnings/hechos-del-harness-claude-code]].
