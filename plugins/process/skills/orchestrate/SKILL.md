---
name: orchestrate
description: Ejecuta planes de implementación multi-tarea: orquestador padre que despacha un ejecutor fresco por tarea, review en dos etapas (spec + calidad) por tarea y review final whole-branch, con ledger durable. Usa para features, refactors o backlogs de tareas independientes.
---

# orchestrate

Orquestador padre: ejecutor fresco por tarea con contexto aislado (nunca
hereda la sesión); el padre valida y decide, los hijos implementan.
Narración mínima — el ledger y los resultados llevan el registro.

## PARIDAD CRÍTICA — no negociable

`subagent_type: reflex:executor`, **nunca** `general-purpose`, **sin**
`model` (el rol lo trae fijo — pasarlo lo pisaría). Si se pierde, reflex
v2 se desenchufa sin síntoma.

## Pre-flight y ejecución

Revisión crítica del plan + pre-flight de conflictos/mandatos-vs-rubric ⇒
una pregunta batcheada al humano ANTES de empezar. Recon: refs del plan
contra el código real. Ledger (`.superpowers/sdd/progress.md`): nunca
re-despaches tareas ya completas. Sin check-ins entre tareas; para SOLO
por BLOCKED, ambigüedad que impide avanzar, o fin de tareas — blocker/gap/
instrucción incomprensible ⇒ PARA y pregunta, no adivines.

## Dispatch y modelo

Una tarea por dispatch: encaje + brief (fuente de verdad) + interfaces
previas + tu resolución de ambigüedad. File handoffs como FICHEROS
(`implementer-prompt.md`, `scripts/{task-brief,review-package,
sdd-workspace}`), nunca pegados. Memory packet: 3-5 permalinks + "lee
solo si hace falta"; degradación con aviso visible si no hay KB, nunca
bloquear. Brief completeness: delta de estándares tácitos + blindspot
pass barato antes de trabajo no trivial. Delegate by default. `model`
explícito SIEMPRE (salvo rol fijo): haiku = transcripción, sonnet =
juicio, top = solo review final + la orquestación misma, una vez por
rama; reviewer escalado al riesgo del DIFF, nunca heredado del padre;
turn-count > token-price.

## Estados, reviewer y review

DONE → package + reviewer. DONE_WITH_CONCERNS → lee concerns antes de
seguir. NEEDS_CONTEXT → aporta y re-despacha. BLOCKED → más contexto /
modelo mayor / partir la tarea / escalar — nunca forzar retry sin cambiar
nada. Prompt del reviewer (guía en `reviewer-prompt.md`): constraints
verbatim, sin directivas open-ended, sin re-pedir tests ya corridos,
nunca pre-juzgar findings, BASE registrado antes del dispatch (nunca
`HEAD~1`). Dos verdictos por tarea; review final con `MERGE_BASE`. Los ⚠️
"cannot verify" los resuelve el orquestador. Fix subagents para Critical/
Important; Minor al ledger, triaje en el final. Plan-mandated o conflicto
con el plan ⇒ decisión del humano; doc/comment baratos, inline. Fix
dispatch re-corre sus tests. Findings del final ⇒ UN fix subagent con la
lista completa.

## Paralelización, ledger, validación y red lines

Solo dominios independientes sin estado compartido, un mensaje con todos
los dispatches; prompt con scope, self-contained, constraints, output
esperado. No paralelices ante fallos relacionados, estado completo
necesario, debugging exploratorio, o estado compartido. Al volver: lee
cada summary, verifica que no chocan, suite completa, spot-check. Ledger:
`Task N: complete (commits …)` al cerrar; tras compaction manda el
ledger + `git log`. El hijo se auto-revisa, el padre valida SIEMPRE —
nunca auto-aprobar inline. Número que no cuadra ⇒ recon antes de
racionalizar. Backlog autónomo: secuencial, NUNCA push/deploy
desatendido, salta decisiones del dueño explicando por qué, documenta lo
que normalmente preguntarías. Red lines: nunca main/master sin
consentimiento explícito; nunca implementers paralelos sobre el mismo
estado; nunca re-despachar una tarea que el ledger marca completa.
