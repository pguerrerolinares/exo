---
name: orchestrate
description: Ejecuta planes de implementación multi-tarea: orquestador padre que despacha un ejecutor fresco por tarea, review en dos etapas (spec + calidad) por tarea y review final whole-branch, con ledger durable. Usa para features, refactors o backlogs de tareas independientes.
---

# orchestrate

Orquestador padre: ejecutor fresco por tarea, contexto aislado (nunca
hereda la sesión); el padre valida y decide, los hijos implementan.
Narración: un mensaje por ola (cierre con números) y uno por Ruling no
obvio; nada por transición de tarea. Una notificación que no cambia el
estado (duplicada, idle sin novedad) no se contesta. El ledger y los
resultados llevan el resto.

Rule and continue destilado de `subagent-driven-development/SKILL.md`
(superpowers 7.0.0, MIT © 2025 Jesse Vincent).

## PARIDAD CRÍTICA — no negociable

`subagent_type: exo:executor`, **nunca** `general-purpose`, **sin**
`model` (el rol lo trae fijo — pasarlo lo pisaría). Si se pierde, reflex
v2 se desenchufa sin síntoma.

## Pre-flight y ejecución

Revisión crítica del plan + pre-flight de conflictos/mandatos-vs-rubric ⇒
un Ruling por hallazgo ANTES de empezar (incluye todo `DAG: error`,
`olas.md`). El pre-flight lleva una fila por tarea, el checker
`puede-caer`/`no-cae` por test, la fila de recursos compartidos y copia
`## Cierre de ola` al ledger (`olas.md`). Recon: refs del plan
contra el código real. Si el plan nombra una Spec, léela también: es la
autoridad de la que el plan argumenta y los conflictos internos del plan se
resuelven contra ella; sin spec alcanzable ⇒ nota en el ledger (los Rulings
sin ella son provisionales). Workspace por plan: arranca con
`scripts/sdd-workspace PLAN` (imprime su dir bajo `.superpowers/sdd/`; un marcador `plan-path` evita que dos planes del mismo nombre lo compartan,
git-ignored; ahí viven ledger, briefs, reports y packages; el de otro plan
no se lee ni se escribe). Ledger (`<workspace>/progress.md`): nunca
re-despaches tareas completas. Su primera línea es
`# SDD ledger — plan: <ruta del plan>`; si nombra otro plan (o es el
`.superpowers/sdd/progress.md` plano antiguo) es progreso AJENO: déjalo y
empieza el tuyo limpio. Sin check-ins entre tareas. Un plan en marcha no espera a un humano:
conflicto, ambigüedad, defecto del plan o tope que pedirías exceder ⇒ decide
(la spec manda, el plan la argumenta) y ledgeriza
`Ruling: <decisión> — <porqué> — <coste si está mal>`; un Ruling errado
cuesta retrabajo visible y reversible, una sesión parada cuesta el día. PARA
y pregunta SOLO ante: operación irreversible o destructiva; acción sensible
de seguridad; efecto fuera del worktree que las normas mandan preguntar
(merge, push a rama compartida, publicar); plan tan roto que todo camino es
adivinar.

## Dispatch y modelo

Una tarea por dispatch: encaje + brief (fuente de verdad) + interfaces
previas + tu resolución de ambigüedad. Handoffs como FICHEROS
(`implementer-prompt.md`, `scripts/{task-brief,review-package,
sdd-workspace,task-dag}`), nunca pegados. Contexto de KB: lo inyecta el hook SubagentStart
(`scripts/subagent-inject.sh`); en el brief, punteros solo como rutas de
fichero; sin KB ⇒ aviso visible, nunca bloquear. Brief completeness:
delta de tácitos + blindspot pass barato si no es trivial. Delegate by
default. `model` explícito SIEMPRE (salvo rol fijo):
haiku = transcripción, sonnet = juicio, top = review final + la
orquestación, una vez por rama; reviewer por tarea: sonnet de suelo, top si
el DIFF tiene concurrencia o seguridad sutil, nunca haiku; turn-count >
token-price.

## Estados, reviewer y review

DONE → package + reviewer. DONE_WITH_CONCERNS → lee concerns antes de
seguir. NEEDS_CONTEXT → aporta y re-despacha. BLOCKED → más contexto /
modelo mayor / partir la tarea / Ruling sobre el plan y re-dispatch con el
Ruling en el brief — nunca retry sin cambiar nada.
Idle sin report y con texto de espera («sigo esperando…») = BLOCKED:
verifica el artefacto (mtime, `git log`) y reanuda con SendMessage; nunca
contestes «espero». El aviso de una tarea de fondo de un teammate idle no
te despierta: antes de cerrar turno con algo vivo fuera de tus manos, arma
un `CronCreate` one-shot a +N ≤ 20 min que siga `despertar.md` (el hook
`sdd-wake-guard.sh` te lo exige si se te olvida).
Las `## Reglas duras` del repo llegan inyectadas (SubagentStart) a `exo:executor`, `general-purpose` y `claude`: no las copies al brief; sí a otros tipos.
Reviewer (`reviewer-prompt.md`): constraints verbatim, sin directivas
open-ended, sin re-pedir tests ya corridos, nunca pre-juzgar findings, BASE
registrado antes del dispatch (nunca `HEAD~1`). Dos verdictos + pase de
over-engineering por tarea; final con `MERGE_BASE`. El reviewer escribe el report en `task-N-review.md` y devuelve <15 líneas (veredictos, C/I/M, Critical/Important con file:line, ruta). Los ⚠️ "cannot verify" y las líneas «Declined to judge» las resuelve el orquestador (Ruling, `olas.md`); el final recibe los `Review Focus` del plan.
Finding con premisa de librería/plataforma `[NO VERIFICADO]` ⇒ no entra en un fix sin un comando que la verifique (tú o un haiku); fix brief = escenario reproducible, no la receta (`olas.md`).
MUTACIÓN sin score (`no disponible`/`parcial`) al ledger: `olas.md`. Fixes
para Critical/Important; Minor al ledger, triaje en el final. Fix loop (máx 5 rondas por tarea,
resume del executor, re-review acotada, breaker con adjudicación): `olas.md`.
Plan-mandated o conflicto con el plan ⇒ lo decide el orquestador con un
Ruling ledgerizado antes de actuar (sin descartar el finding por estar
mandado, sin fix contra el plan sin Ruling). Si equivocarse cuesta caro,
añade `PR-pendiente` al Ruling: sale en «Rulings I made» y en el PR para que
lo vea el humano, sin parar la ola. Única regla de plan-mandated; `olas.md`
remite aquí.
Inline del padre, lista cerrada: comentario, doc, formato y rename sin
cambio de comportamiento. Todo lo demás va a fix dispatch con re-review, o
lleva un Ruling `inline — <qué> — <por qué no fue a dispatch>`; también tras
la review final limpia. Todo fix dispatch re-corre sus tests. Findings del final ⇒
UN fix subagent con la lista completa (una re-review, sin segunda ola).

## Olas

Calcula con `scripts/task-dag PLAN` (a mano solo si falla); `avisos` ⇒
ledger. Ola de ≥2 ⇒ un worktree por tarea, todos los dispatches en un
mensaje; resto en `olas.md`. Ambiguo ⇒ secuencial. No paralelices ante fallos relacionados,
estado completo necesario, debugging exploratorio ni estado compartido
(salvo modo mismo-worktree con `Files` disjuntos).

## Ledger, validación y red lines

`Task N: complete (commits …)` al cerrar; tras compaction manda el ledger +
`git log`. El hijo se auto-revisa, el padre valida SIEMPRE — nunca
auto-aprobar inline. Review final limpia y sus fixes integrados ⇒ ANTES de borrar, junta TODA
línea `Ruling:` del ledger en tu mensaje final bajo «Rulings I made» (orden
cronológico, cada una con su coste si está mal): es lo único que lleva al
humano las decisiones tomadas por él. Sácalas con `scripts/rulings-collect
<workspace>/progress.md` (numeradas, `PR-pendiente` marcadas con ⚠) y pégalas
también en el cuerpo del PR. Luego `scripts/sdd-workspace close PLAN` (se niega con exit 1 si el CI del PR de la
rama no está verde: un CI rojo o pendiente es hallazgo tardío con el ledger vivo,
fix dispatch + re-review; `--no-ci MOTIVO` solo si no hay PR ni CI, y queda en el ledger): archiva
ledger y `final-review.md` fuera del repo, quita worktrees y ramas `task/*` ya
integrados (sin `--force`) y borra solo el workspace de ese plan. Exit 1 =
quedó algo sin integrar o sucio: lo dices, no lo fuerzas. Nunca `rm -rf` a mano.
Número que no cuadra ⇒ recon antes de racionalizar.
Backlog autónomo: secuencial, NUNCA push/deploy desatendido, salta
decisiones del dueño explicando por qué, documenta lo que preguntarías.
Red lines: nunca main/master sin consentimiento explícito; nunca
implementers paralelos sobre el mismo estado; nunca re-despachar una tarea que el ledger marca completa.
