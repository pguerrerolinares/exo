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
- **El consultor de investigación mide y recomienda.** No está atado a la cita,
  porque lo que produce no se aplica solo: el humano lo firma, y esa
  recomendación firmada pasa a ser la base escrita que faltaba. Puede
  re-encuadrar el problema y exigir medir sobre el sistema real.
- **Lo que el auditor no puede sostener con una cita** (típicamente,
  re-escopear el criterio) es lo único que sube al humano, ya como dictamen
  del consultor para firmar. El resto lo cierra el pipeline, con rastro.

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

Solo ante (a) un gate humano pre-registrado, sobre todo de percepción: lo que
hay que mirar, no deducir; (b) una decisión nueva de gusto o de alcance; (c) una
línea roja. Todo lo demás se resuelve con un Ruling visible, que anota
decisión, porqué y coste si está mal. Una pregunta pendiente se dice una vez y
queda en el ledger. Contestar «espero» a un ejecutor ocioso es un bloqueo, no
una pausa.

## Consultar, medir, reconsultar

1. Formula el síntoma como valor perdido y consulta con una condición de
   invalidación.
2. Implementa ya lo barato y reversible.
3. Mide con un instrumento listo y una receta cerrada, con muestra suficiente.
4. Reconsulta con los datos crudos, no con tu propuesta.
5. El humano firma.

Pre-registra **la condición que invalidaría la recomendación**, no un
estadístico elegido en frío: un umbral fijado antes de ver datos suele
reemplazarse con los datos delante. Y el orden de los criterios de abandono lo
aplica el controlador, no el ejecutor, que tiende a racionalizar contra ellos.

## Al humano, el dictamen

La regla se mecaniza: el hook `delegar-decision` bloquea el cierre de un turno
que trae propuesta y pide visto bueno sin haber consultado. Detalle de uso en
`plugins/exo/README.md`. El principio que sostiene el hook: **al humano se le
trae el dictamen de un consultor, no la elección entre opciones**. Una
decisión de diseño se consulta antes de preguntarla.

Relacionadas: [[El padre coordina y valida; el ejecutor implementa|learnings/orquestador-limpio]],
[[La claridad del encargo es el cuello de botella, no la capacidad del agente|learnings/el-brief-es-el-cuello-de-botella]].
