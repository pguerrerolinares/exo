---
name: orchestrate
description: Ejecuta planes de implementación multi-tarea: orquestador padre que despacha un ejecutor fresco por tarea, review en dos etapas (spec + calidad) por tarea y review final whole-branch, con ledger durable. Usa para features, refactors o backlogs de tareas independientes.
---

# orchestrate

Orquestador padre en la sesión actual: por cada tarea despacha un ejecutor
fresco con contexto aislado — nunca hereda la historia de la sesión, tú
construyes exactamente lo que necesita. El padre valida y decide; los hijos
implementan. Narración mínima entre tool calls: el ledger y los resultados
llevan el registro.

## PARIDAD CRÍTICA — no negociable

Despacha ejecutores con `subagent_type: reflex:executor`, **nunca**
`general-purpose`, y **sin** `model` (el rol trae modelo fijo — pasarlo lo
pisaría). Si se pierde, reflex v2 se desenchufa sin síntoma.

## Antes de Task 1

- Carga el plan y revísalo con ojo crítico completo: dudas o concerns al
  humano ANTES de empezar a ejecutar.
- Pre-flight de conflictos: contradicciones internas del plan y mandatos que
  el rubric de review trataría como defecto — una pregunta batcheada, no un
  interrupt por hallazgo.
- Recon (retrieve > compute): verifica las refs del plan (líneas, firmas,
  símbolos) contra el código real.
- Ledger: revisa `.superpowers/sdd/progress.md`; tareas ya completas no se
  re-despachan — resume en la primera pendiente.

## Ejecución continua

Sin check-ins entre tareas; para SOLO por BLOCKED irresoluble, ambigüedad
que impide avanzar, o todas las tareas completas. Ante blocker, gap crítico
del plan, o instrucción incomprensible: PARA y pregunta, no adivines ni
fuerces.

## Dispatch por tarea

- Un dispatch describe UNA tarea: encaje en una línea + brief como fuente de
  verdad + interfaces de tareas previas + tu resolución de ambigüedades — no
  la historia acumulada de la sesión.
- File handoffs: brief, report y review package viajan como FICHEROS (paths
  en el prompt), nunca pegados — usa `implementer-prompt.md` y los scripts
  (`task-brief`, `review-package`, `sdd-workspace`).
- Memory packet: 3-5 permalinks de notas canónicas + "léelas solo si la
  tarea lo pide"; degradación con aviso visible si no hay KB (probe →
  defaults + aviso, nunca bloquear el dispatch).
- Brief completeness: aflora estándares tácitos como delta + blindspot pass
  barato (lee SOLO el brief → devuelve gaps/ambigüedades) antes de trabajo
  no trivial.
- Delegate by default: inline solo lo genuinamente trivial (typo, flag de
  una línea); en la duda, delega.

## Cost pyramid — model explícito SIEMPRE (salvo el rol con modelo fijo)

Haiku = transcripción (código literal en el plan, fixes mecánicos de un
fichero). Sonnet = juicio (integración multi-fichero, refactor, evals). Top
= SOLO review final whole-branch + la orquestación misma, una vez por rama.
Reviewer escalado al riesgo del DIFF, nunca heredado del padre. Turn-count
> token-price: modelo barato solo cuando el plan ES el código.

## Estados del implementer

DONE → genera review package y despacha reviewer. DONE_WITH_CONCERNS → lee
los concerns antes de seguir. NEEDS_CONTEXT → aporta contexto y re-
despacha. BLOCKED → más contexto / modelo mayor / partir la tarea / escalar
al humano — nunca forzar retry sin cambiar nada.

## Construir el prompt del reviewer (`reviewer-prompt.md`)

Global constraints copiados VERBATIM del plan/spec. Sin directivas
open-ended sin razón task-specific; no pidas re-correr tests que el
implementer ya corrió. Nunca pre-juzgues findings ("do not flag", "at most
Minor") — el reviewer detecta, el loop adjudica. Package con el BASE
registrado ANTES del dispatch — nunca `HEAD~1` (trunca tareas multi-commit).

## Review y fixes

Dos verdictos por tarea (spec + calidad); review final whole-branch al
acabar, con package sobre `MERGE_BASE`. Los ⚠️ "cannot verify from diff" los
resuelve el orquestador; gap real ⇒ vuelta al implementer y re-review. Fix
subagents para Critical/Important; Minor al ledger, el review final los
triaja (roll-up que nadie lee = descarte silencioso). Finding plan-mandated
o en conflicto con el plan ⇒ decisión del humano — presenta finding + texto
del plan, nunca obedeces ni descartes en silencio; fixes doc/comment
baratos, aplícalos inline. Todo fix dispatch re-corre los tests de su
cambio y lo reporta antes del re-review. Findings del review final ⇒ UN fix
subagent con la lista completa, no un fixer por finding.

## Paralelización (solo dominios independientes)

Un agente por dominio sin estado compartido, todos los dispatches en un
mismo mensaje; prompt con scope específico, self-contained, constraints
explícitos, output esperado definido. NO paralelices ante fallos
relacionados, necesidad de estado completo del sistema, debugging
exploratorio, o estado compartido. Al volver: lee cada summary, verifica
que no chocan, corre la suite completa, spot-check.

## Ledger y validación

Al cerrar cada tarea, línea `Task N: complete (commits …)` en el ledger;
tras compaction/resume manda el ledger + `git log` sobre tu memoria. El
hijo se auto-revisa pero el padre valida SIEMPRE independientemente —
nunca auto-aprobar inline. Cuando un número no cuadra, recon antes de
racionalizarlo.

## Autonomous runs ("te dejo a cargo")

Backlog secuencial (cada tarea valida antes de la siguiente). NUNCA push ni
deploy desatendido — commit local, deja el batch para review. Salta items
que requieren decisión del dueño, explicando por qué. Documenta toda
decisión que normalmente habrías preguntado.

## Red lines

Nunca empezar en main/master sin consentimiento explícito del usuario.
Nunca despachar implementers en paralelo sobre el mismo estado (conflictos).
Nunca re-despachar una tarea que el ledger marca completa.
