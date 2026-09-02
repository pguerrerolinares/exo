---
permalink: "{{KB_NAME}}/core/core-index"
title: core-index — mapa y presupuesto de esta KB
tags: [core, indice]
tier: core
semilla: true
---

# core-index — mapa y presupuesto de esta KB

Este fichero es lo primero que debe leer un agente al arrancar en esta KB.
No es la doctrina en sí — es el mapa: qué hay, dónde está, y cuánto puede
pesar cada cosa antes de que el mantenimiento deje de ser sostenible.

## Contrato de memoria

Cómo debe leer y escribir en esta KB un agente: **canon como delta** (edita
la nota existente, no dupliques), **bitácora como append** (nunca reescribas
una entrada de `log/`) y **nota nueva casi nunca**. El contrato completo —
qué va a cada carpeta — vive en
[[Contrato de la KB para agentes|AGENTS.md]]: este índice apunta, no repite.

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

## Doctrina compacta

- Delega investigación y ejecución voluminosa a un agente fresco; quédate con la conclusión, no el material crudo.
- Evidencia antes que afirmación: corre el comando, enseña el resultado, y solo entonces di "hecho".
- El fallo que no avisa (exit 0, forma válida) es el que importa vigilar, no solo el que grita.
- Terreno desconocido o error repetido: verifica el supuesto antes de seguir computando.
- La ambigüedad del encargo es el cuello de botella, no la capacidad del agente.

## Cores

- **[[core-index]]** (esta nota) — el único `tier: core` de la KB semilla;
  el arranque de `exo recall` sirve siempre su cuerpo completo.
- Cuando una nota `stable` pase a ser lectura obligada en casi toda sesión,
  sube su `tier` a `core` y añade aquí una línea con su rol: es la única
  forma de que el arranque llegue a servirla sin que el agente la busque.
- Cada `core` nuevo compite por el mismo presupuesto de arranque (siguiente
  sección): promover de más sale caro en todas las sesiones futuras.

## Presupuesto por tier

Este mismo fichero es el que más se lee y el que primero rompe el
presupuesto si crece sin disciplina. La regla:

- **`core/core-index`**: cap **6.144 B**, con un 15% de aire exigido sobre
  ese cap — es decir, el contenido vivo no debería superar **5.222 B**. El
  aire es margen de mantenimiento, no invitación a llenarlo.
- El resto de notas `core` y `stable` no tiene un cap numérico fijo, pero
  sigue el mismo espíritu: un destilado que crece sin podar deja de ser un
  destilado.
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
  vigilarlo exige buscarlo activamente, no solo reaccionar al que sí grita:
  de la degradación con forma válida a la ausencia que no es evidencia.
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
