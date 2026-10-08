---
permalink: "{{KB_NAME}}/core/core-index"
title: core-index — mapa de esta KB
tags: [core, indice]
tier: core
semilla: true
---

# core-index — mapa de esta KB

Lo primero que lee un agente al arrancar. Es un índice: una línea por
principio y un enlace al detalle. El arranque lo sirve entero dentro de un
cap de **6.144 B** con un 15% de aire (≤ 5.222 B vivos): ese cap de arranque
manda sobre el techo genérico de nota `core` (8.500 B). Al crecer se le
retiran entradas muertas; las vivas no se comprimen. El orden de "Doctrina
compacta" importa: a cada subagente solo le llegan sus primeros 550 B.

## Contrato de memoria

- Qué va a cada carpeta, dónde va un avance, presupuestos y trinquete:
  [[Contrato de la KB para agentes|AGENTS.md]].
- Busca antes del primer Agent/Edit/Write (`exo search`, `exo targets`);
  cierra con `exo:document` (delta al canon + append a la bitácora).
- Al morder un techo: parte, rota o consolida. Nunca subas el techo ni
  recortes el delta que ibas a escribir.

## Doctrina compacta

- **Ejecutar > leer**: solo ejecutar verifica verdad; exige el rojo real. → [[verificar-ejecutando-no-leyendo|learnings/verificar-ejecutando-no-leyendo]]
- **Fallo silencioso**: un check debe poder ponerse rojo por su caso. → [[fallo-silencioso|learnings/fallo-silencioso]]
- **Recon-first**: mismo error ≥3 veces → verifica el supuesto. → [[recon-first|learnings/recon-first]]
- **Orquestador limpio**: delega la lectura y la ejecución voluminosas; el padre detecta y valida, el ejecutor arregla. → [[orquestador-limpio|learnings/orquestador-limpio]]
- **Pirámide de coste**: `model` explícito (salvo rol con modelo fijo, como `exo:executor`); haiku transcribe, sonnet juzga e integra, Opus la review final de rama. → [[orquestador-limpio|learnings/orquestador-limpio]]
- **Brief**: la ambigüedad del encargo manda; hechos con ancla `fichero:línea` o marcados SUPUESTO; blindspot pass antes de despachar. → [[el-brief-es-el-cuello-de-botella|learnings/el-brief-es-el-cuello-de-botella]]
- **Instrumento validado**: un negativo o "0 resultados" no vale hasta validar el detector con un positivo conocido. → [[instrumento-validado-antes-de-medir|learnings/instrumento-validado-antes-de-medir]]
- **Enforcement**: una lección que no acaba en hook, test o gate se ha anotado, no aprendido; mecaniza lo verificable. → [[la-prosa-no-es-enforcement|learnings/la-prosa-no-es-enforcement]]
- **LLM como último recurso**: determinista primero, `null > inventado`; lo que no mide se aparca, no se borra. → [[construir-con-llms-y-descartar|learnings/construir-con-llms-y-descartar]]
- **Harness**: hechos de Claude Code y Windows que muerden (hooks, JSONL, `git add -A`, CP-1252); verifícalos contra la versión viva. → [[hechos-del-harness-claude-code|learnings/hechos-del-harness-claude-code]]
- **Evidencia antes que afirmación**, **cambios pequeños en el estilo de alrededor**, **revisión proporcional al riesgo**. → [[Doctrina de trabajo con agentes|core/doctrina]]

## Routing de proceso (plugin exo)

`exo:brainstorm` (diseño antes de código) · `exo:plan` (spec → plan) ·
`exo:orchestrate` (ejecutar un plan multi-tarea) · `exo:tdd` (test primero) ·
`exo:debug` (bug o atasco) · `exo:recon-first` (atasco o terreno
desconocido) · `exo:verify` (antes de declarar hecho) ·
`exo:document` (cierre de sesión) · `exo:distill` (consolidación). Si la
tarea encaja con una, invócala antes de actuar. Un subagente que ejecuta una
tarea concreta está exento.

## Cores

- **core-index** (esta nota): el único `tier: core`. Sube una nota a `core`
  solo si es lectura obligada en casi toda sesión, y añádela aquí: cada core
  compite por el mismo presupuesto de arranque.

## Proyectos activos

(Ninguno todavía. Una línea por proyecto con nota en `projects/`; se retira
al cerrarlo.)
