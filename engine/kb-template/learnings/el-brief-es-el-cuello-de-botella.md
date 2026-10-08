---
permalink: "{{KB_NAME}}/learnings/el-brief-es-el-cuello-de-botella"
title: La claridad del encargo es el cuello de botella, no la capacidad del agente
tags: [agentes, delegacion, brief, verificacion]
tier: stable
semilla: true
---

# La claridad del encargo es el cuello de botella, no la capacidad del agente

Cuando un agente capaz produce un resultado que no era el que hacía falta, la
conclusión fácil es que no dio la talla. Casi siempre la causa es otra: el
encargo admitía más de una lectura razonable, el agente eligió una con toda su
capacidad y nada en el proceso avisó, porque desde dentro de esa lectura todo
era coherente. Un modelo débil falla ruidoso y local; uno fuerte resuelve tus
ambigüedades con confianza, en la dirección que infirió. Subir de modelo no
cierra el hueco entre mapa y territorio: lo amplifica.

Cómo viaja el contexto al hijo es otro eje ([[El padre coordina y valida; el ejecutor implementa|learnings/orquestador-limpio]]);
aquí se trata de si el contexto está completo antes de despachar. Un brief puede
viajar perfecto y estar vacío de la intención real.

## Por qué pasa tan en silencio

Un agente sin contexto no sabe lo que no le han dicho. Ante ambigüedad no
pregunta salvo que se le permita: rellena el hueco con la lectura más plausible
y sigue. El resultado tiene toda la forma de un trabajo bien hecho porque, dada
la premisa que asumió, lo es. El desajuste está en la premisa, no en la
ejecución, y no se ve hasta comparar con lo que hacía falta. Subir la capacidad
no lo arregla: un agente más capaz interpreta la ambigüedad con más
sofisticación, no la elimina. Lo que sí: explicar el propósito (no solo el paso
a paso), qué se probó o descartó, qué decisiones puede tomar solo y qué forma
debe tener el resultado.

## Qué falta en un brief que parece completo

- **Estándares tácitos (unknown knowns).** Lo que das por sentado y no dices
  (anti-over-engineering, contrastar con fuentes, tono) el ejecutor no lo
  hereda. Todo brief de juicio, no de transcripción, lleva una línea de delta
  con los estándares que aplican a esa tarea concreta.
- **Blindspot pass antes de despachar.** En tarea no trivial, un hijo haiku lee
  solo el brief y devuelve sus ambigüedades y huecos. Coge el supuesto
  equivocado cuando cuesta una línea, no cuando ya es código.
- **Implementation notes durante la ejecución.** El ejecutor anota sus supuestos
  en un fichero de notas antes de cada paso: sirve de relevo por fichero y de
  punto de intervención en vivo.
- **Validation gate antes de aceptar.** El padre valida de forma independiente
  porque el fallo del modelo fuerte es silencioso. Antes de aceptar el diff, que
  el ejecutor parafrasee o "venda" sus decisiones de diseño; si la paráfrasis te
  sorprende, ahí afloró el gap.

## Los hechos del brief se verifican antes de escribirlos

- **Un ejecutor no puede corregir un error factual del brief**, porque el brief
  es su fuente de verdad. Un hecho inferido a la ligera (p. ej. del tamaño de un
  diff) llega al ejecutor como verdad, y el tamaño no distingue comentarios de
  lógica.
- **Cada hecho lleva ancla `fichero:línea` o va marcado como SUPUESTO.** Así el
  ejecutor sabe qué puede dar por firme y qué debe contrastar.
- **Pide verificar la premisa empírica antes de implementar.** Un ejecutor sí
  puede cazar el error de premisa si se le encarga contrastarla con datos
  reales: *"si no reproduce, para y avisa"*. Un fix inerte es peor que ninguno,
  porque declara resuelto un problema que sigue ahí.
- **El canon, el SKILL y el backlog son estado declarado, no evidencia.** `grep`
  al código antes de apoyarte en ellos. Que tres documentos repitan una
  afirmación no la corrobora: se copiaron entre sí, y la doc no verifica a la
  doc. Una función "existente" en el canon puede no estar en el código; un
  `grep` lo cierra en segundos.
- **Instancia no es contrato.** Al bootstrapear un artefacto con contrato
  (config, formato de skill), valídalo contra el contrato canónico, no contra la
  instancia de otro proyecto.

## La revisión hay que apuntarla al sitio donde está el riesgo

- **El código del plan es el sospechoso principal.** La mayoría de los bugs
  reales estaban en el código que el orquestador escribió en el plan, no en lo
  que añadieron los ejecutores. Dile al revisor que **el
  plan no es autoridad**; sin esa línea trata el código del plan como requisito
  y solo audita las desviaciones, justo al revés de donde está el riesgo.
- **Nombra el riesgo concreto, no pidas revisión genérica.** Las revisiones que
  rinden llevan encargo específico (*"traza el patrón carácter a carácter contra
  la salida real"*, *"¿el test falla si borras el arreglo?"*). Las
  genéricas devuelven prosa. Ese encargo lo paga quien tiene el contexto cruzado:
  el orquestador.
- **El test del brief también es sospechoso, y solo lo caza quien mide.** Un
  test puede pasar por una razón ajena (p. ej. el fallo lo provoca otro gate) o
  dar verde igual con y sin el defecto que debía detectar. Lo cazan los
  ejecutores que miden el antes y el después antes de escribir código. Pide el
  **rojo real**, no el "debería fallar"; si no se puede provocar, eso es el
  hallazgo.

## Disciplina raíz

Cualquier output que te sorprenda es la señal de un supuesto que no compartiste.
Trátalo así, no como error del modelo.

Si un arreglo se difiere (cosmético, requiere coordinar con otro equipo, ROI
bajo), la decisión se documenta en el commit o en la spec, nunca queda como
"tenía intención de tocarlo".

## Cuándo no aplica

El encargo detallado no sustituye a pensarlo bien: un brief minucioso pero mal
razonado produce un resultado mal razonado, solo que con más precisión. En
tareas de transcripción mecánica el blindspot pass y la línea de estándares
sobran; reservan su coste para el juicio. Aplica sobre todo a hijos sin memoria
de la conversación, que parten de cero.

Relacionadas: [[En terreno desconocido, verificar el supuesto antes de seguir computando|learnings/recon-first]],
[[La lectura verifica coherencia; solo la ejecución verifica verdad|learnings/verificar-ejecutando-no-leyendo]],
[[El fallo más caro es el que no avisa, y cada forma tiene su remedio|learnings/fallo-silencioso]],
[[Doctrina de trabajo con agentes|core/doctrina]].
