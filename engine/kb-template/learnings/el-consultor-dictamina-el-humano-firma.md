---
permalink: "{{KB_NAME}}/learnings/el-consultor-dictamina-el-humano-firma"
title: El consultor dictamina con cita y el humano firma, no elige
tags: [agentes, orquestacion, gates, delegacion, decision]
tier: stable
semilla: true
---

# El consultor dictamina con cita y el humano firma, no elige

Un orquestador que se detiene a cada duda para preguntar al humano no
orquesta: reenvía. Y uno que cierra solo cada trade-off tiende a "cerrar
limpio" y a racionalizar. La salida es repartir el trabajo: un tercero
independiente dictamina, y al humano le llega una recomendación que firmar, no
un menú entre opciones.

## Auditor y consultor no son lo mismo

- **El auditor adjudica por cita.** Toda conclusión se ancla a texto escrito
  (la spec, el criterio pre-registrado). Si no hay base escrita, **rechaza**:
  no la inventa.
- **El consultor de investigación explora y propone.** La cita no lo ata,
  porque su propuesta no entra en vigor sola: la firma una persona, y esa
  firma convierte la propuesta en el texto de referencia que hasta entonces
  no existía. Puede plantear el problema de otra forma y pedir mediciones
  sobre el sistema real.
- **Solo escala al humano lo que ninguna cita respalda**, por ejemplo cambiar
  el alcance de un criterio ya fijado; llega como propuesta del consultor lista
  para firmar. Lo demás se cierra en el pipeline, dejando rastro.

## Cuándo un dictamen es legítimo

Sin las cuatro condiciones, el dictamen no vale y la decisión sube:

1. El revisor es fresco: no participó en la pieza que juzga.
2. Hace su propia verificación primaria: ejecuta, no relee el informe de otro.
3. Tiene mandato explícito de disentir.
4. Deja un veredicto con cita, commiteado.

Codifica la **condición** que hizo legítimo un precedente, no su vocabulario.
Una línea roja queda fuera de cualquier delegación: lo destructivo o externo
(borrar, publicar, tocar permisos) siempre lo autoriza el humano.

Si el disenso del consultor cae de golpe sin causa, sospecha convergencia
complaciente y rota su mandato. Y apilar revisiones del mismo material no
sustituye a una verificación primaria fresca.

## Cuándo para el orquestador

Se detiene en tres casos: un gate humano fijado de antemano (sobre todo los de
percepción, donde hay que mirar y no deducir), una decisión nueva de gusto o de
alcance, y una línea roja. En el resto decide y deja un Ruling visible con la
decisión, su motivo y lo que costaría equivocarse. Si queda algo por
preguntar, lo plantea una sola vez y lo apunta en el ledger. Responder «espero»
a un ejecutor ocioso es dejarlo bloqueado, no pausar.

## Consultar, medir, reconsultar

1. Formula el síntoma como valor perdido y consulta con una condición de
   invalidación.
2. Implementa ya lo barato y reversible.
3. Mide con un instrumento listo y una receta cerrada, con muestra suficiente.
4. Vuelve al consultor con las mediciones sin procesar, sin adjuntar tu solución.
5. El humano firma.

Registra de antemano **qué hecho tumbaría la recomendación**; no escojas
primero una métrica a ciegas: un umbral inventado sin datos suele acabar
sustituido cuando ya los tienes delante. Y el orden de los criterios de abandono lo
aplica el controlador, no el ejecutor, que tiende a racionalizar contra ellos.

## Al humano, el dictamen

El hook `delegar-decision` mecaniza esta regla; su descripción está en
`plugins/exo/README.md`. El principio: **el humano recibe un dictamen
razonado que firmar, no una lista de alternativas para elegir**. Una decisión
de diseño se consulta antes de plantearla.

Relacionadas: [[El padre coordina y valida; el ejecutor implementa|learnings/orquestador-limpio]],
[[La claridad del encargo es el cuello de botella, no la capacidad del agente|learnings/el-brief-es-el-cuello-de-botella]].
