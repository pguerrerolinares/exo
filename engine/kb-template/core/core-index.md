---
permalink: "{{KB_NAME}}/core/core-index"
title: core-index — mapa y presupuesto de esta KB
tags: [core, indice]
tier: stable
semilla: true
---

# core-index — mapa y presupuesto de esta KB

Este fichero es lo primero que debe leer un agente al arrancar en esta KB.
No es la doctrina en sí — es el mapa: qué hay, dónde está, y cuánto puede
pesar cada cosa antes de que el mantenimiento deje de ser sostenible.

## Qué hay y dónde

- **`core/`** — identidad y doctrina estable. Punto de entrada. Ver
  [[Doctrina de trabajo con agentes|core/doctrina]] para el desarrollo completo
  de los principios de trabajo con agentes.
- **`projects/`** — destilado canónico por proyecto: la foto, no el vídeo.
- **`learnings/`** — principios reutilizables, independientes del proyecto
  que los originó. Índice de los cuatro que trae esta semilla, más abajo.
- **`log/`** — bitácora append-only, una por proyecto.
- **`archive/`** — retirado de circulación activa; no hace falta leerlo en
  el flujo normal.

El contrato completo de routing (qué va a cada carpeta, la regla de "canon
como delta, bitácora como append, nota nueva casi nunca") está en
[[Contrato de la KB para agentes|AGENTS.md]], no aquí: este índice apunta, no
repite.

## Presupuesto por tier

Este mismo fichero es el que más se lee y el que primero rompe el
presupuesto si crece sin disciplina. La regla:

- **`core/core-index`**: cap **6.144 B**, con un 15% de aire exigido sobre
  ese cap — es decir, el contenido vivo no debería superar **5.222 B**. El
  aire es margen de mantenimiento, no invitación a llenarlo.
- El resto de notas `stable` no tiene un cap numérico fijo, pero sigue el
  mismo espíritu: un destilado que crece sin podar deja de ser un destilado.
- Las notas `log` no tienen cap — son append-only por diseño — pero se
  espera que un proceso de consolidación las resuma hacia `stable`
  periódicamente en vez de dejarlas crecer sin límite indefinidamente.

## La regla de los índices

Cuando este fichero (o cualquier nota que funcione como índice) se acerca a
su presupuesto, el mantenimiento correcto es **retirar entradas muertas**
— proyectos cerrados, principios ya fundidos en otro, punteros a notas que
ya no existen — nunca comprimir la prosa de las entradas que siguen vivas.
Un índice comprimido deja de servir como índice: se hojea para orientarse
rápido, y una entrada resumida a la mitad no orienta, confunde.

## Los cuatro learnings de esta semilla

- **[[Un orquestador delega la lectura pesada y se queda solo con la conclusión|learnings/orquestador-limpio]]**
  — el subagente descarta la mayor parte de lo que exploró; solo la
  respuesta vuelve al hilo que coordina.
- **[[En terreno desconocido, verificar el supuesto antes de seguir computando|learnings/recon-first]]**
  — la señal de que toca parar es el mismo error repitiéndose, no solo lo
  desconocido del terreno.
- **[[El fallo más caro es el que no avisa|learnings/fallo-silencioso]]** —
  el fallo que no avisa (forma válida, checks no falsables, exit 0 sin
  efecto, ausencia sin ser evidencia) es más caro que el que sí avisa.
- **[[La claridad del encargo es el cuello de botella, no la capacidad del agente|learnings/el-brief-es-el-cuello-de-botella]]**
  — subir la capacidad del agente no lo arregla: interpretará la ambigüedad
  con más sofisticación, no la eliminará.

## Cómo mantener esta nota-mapa

Esta nota es un índice a mano, distinto del índice de búsqueda que construye
`exo index` (ese sí se regenera solo — ver el README). Al añadir un
`learning` nuevo o un proyecto con frente propio, decide si merece entrada
aquí: si es de consulta frecuente, sí; si es de nicho, basta con que el
índice de búsqueda lo encuentre por consulta. Una nota-mapa que lista todo
no es un mapa, es una copia de la carpeta.
