---
name: orchestrate
description: Ejecuta planes de implementación multi-tarea: orquestador padre que despacha un ejecutor fresco por tarea, review en dos etapas (spec + calidad) por tarea y review final whole-branch, con ledger durable. Usa para features, refactors o backlogs de tareas independientes.
---

# orchestrate

Orquestador padre: ejecutor fresco por tarea, contexto aislado (nunca
hereda la sesión); el padre valida y decide, los hijos implementan.
Narración mínima — el ledger y los resultados llevan el registro.

## PARIDAD CRÍTICA — no negociable

`subagent_type: exo:executor`, **nunca** `general-purpose`, **sin**
`model` (el rol lo trae fijo — pasarlo lo pisaría). Si se pierde, reflex
v2 se desenchufa sin síntoma.

## Pre-flight y ejecución

Revisión crítica del plan + pre-flight de conflictos/mandatos-vs-rubric ⇒
UNA pregunta batcheada al humano ANTES de empezar (incluye todo `DAG: error`,
`olas.md`). Recon: refs del plan
contra el código real. Ledger (`.superpowers/sdd/progress.md`): nunca
re-despaches tareas completas. Sin check-ins entre tareas; para SOLO por
BLOCKED, ambigüedad que impide avanzar o fin de tareas — blocker/gap/
instrucción incomprensible ⇒ PARA y pregunta, no adivines.

## Dispatch y modelo

Una tarea por dispatch: encaje + brief (fuente de verdad) + interfaces
previas + tu resolución de ambigüedad. Handoffs como FICHEROS
(`implementer-prompt.md`, `scripts/{task-brief,review-package,
sdd-workspace,task-dag}`), nunca pegados. Memory packet: 3-5 permalinks + "lee solo
si hace falta"; sin KB ⇒ aviso visible, nunca bloquear. Brief completeness:
delta de tácitos + blindspot pass barato si no es trivial. Delegate by
default. `model` explícito SIEMPRE (salvo rol fijo):
haiku = transcripción, sonnet = juicio, top = review final + la
orquestación, una vez por rama; reviewer por tarea: sonnet de suelo, top si
el DIFF tiene concurrencia o seguridad sutil, nunca haiku; turn-count >
token-price.

## Estados, reviewer y review

DONE → package + reviewer. DONE_WITH_CONCERNS → lee concerns antes de
seguir. NEEDS_CONTEXT → aporta y re-despacha. BLOCKED → más contexto /
modelo mayor / partir la tarea / escalar — nunca retry sin cambiar nada.
Reviewer (`reviewer-prompt.md`): constraints verbatim, sin directivas
open-ended, sin re-pedir tests ya corridos, nunca pre-juzgar findings, BASE
registrado antes del dispatch (nunca `HEAD~1`). Dos verdictos + pase de
over-engineering por tarea; final con `MERGE_BASE`. Los ⚠️ "cannot verify" los resuelve el orquestador.
MUTACIÓN sin score (`no disponible`/`parcial`) al ledger: `olas.md`. Fix
subagents para Critical/Important; Minor al ledger, triaje en el final.
Plan-mandated o conflicto con el plan ⇒ decide el humano; doc/comment
baratos, inline. Todo fix dispatch re-corre sus tests. Findings del final ⇒
UN fix subagent con la lista completa.

## Olas

Calcula con `scripts/task-dag PLAN` (a mano solo si falla); `avisos` ⇒
ledger. Ola de ≥2 ⇒ un worktree por tarea, todos los dispatches en un
mensaje; resto en `olas.md`. Ambiguo ⇒ secuencial. No paralelices ante fallos relacionados,
estado completo necesario, debugging exploratorio ni estado compartido
(salvo modo mismo-worktree con `Files` disjuntos).

## Ledger, validación y red lines

`Task N: complete (commits …)` al cerrar; tras compaction manda el ledger +
`git log`. El hijo se auto-revisa, el padre valida SIEMPRE — nunca
auto-aprobar inline. Número que no cuadra ⇒ recon antes de racionalizar.
Backlog autónomo: secuencial, NUNCA push/deploy desatendido, salta
decisiones del dueño explicando por qué, documenta lo que preguntarías.
Red lines: nunca main/master sin consentimiento explícito; nunca
implementers paralelos sobre el mismo estado; nunca re-despachar una tarea que el ledger marca completa.
