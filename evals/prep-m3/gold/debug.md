# Gold — process:debug (paridad de movimientos)

Fuentes:
- SD = `~/.claude/plugins/cache/claude-plugins-official/superpowers/6.1.1/skills/systematic-debugging/SKILL.md` (296 líneas) + `root-cause-tracing.md`, `defense-in-depth.md`, `condition-based-waiting.md`
- RF = `~/.claude/plugins/cache/agent-develop/reflex/0.9.0/skills/recon-first/SKILL.md` (57 líneas)

Uso: ver `evals/prep-m3/README.md`.

## Frontmatter (movimiento de disparo, con corrección obligatoria)

- [ ] Description con DOS puertas: (1) bug/test que falla/comportamiento inesperado, antes de proponer fixes; (2) atascado o pre-grind (mismo error repetido, terreno desconocido, time-box) — framework §5.2 ("recon-first (description con dos puertas: bug + stuck/pre-grind…)") + SD frontmatter (línea 3) + RF frontmatter (línea 3)
- [ ] La description NO menciona el hook `stuck-loop`: está muerto (commit 590d6ca) y la description fuente de recon-first ("Triggered by the reflex `stuck-loop` hook", RF línea 3) está stale — framework §5.2 ("el hook stuck-loop está muerto — commit 590d6ca — la description actual de recon-first está stale"). Presencia de la referencia al hook = fallo.

## Movimientos — systematic-debugging

- [ ] Root cause antes de cualquier fix; el fix de síntoma es fallo — SD líneas 12-22 (destilado sin el grito)
- [ ] Fase 1 · leer los errores completos (stack trace entero, líneas, paths, códigos) — SD líneas 54-58
- [ ] Fase 1 · reproducir consistentemente; si no es reproducible ⇒ más datos, no adivinar — SD líneas 60-64
- [ ] Fase 1 · revisar cambios recientes (git diff, commits, deps, config, entorno) — SD líneas 66-70
- [ ] Fase 1 · en sistemas multi-componente: instrumentar cada boundary (qué entra/qué sale/config propagada) y correr UNA vez para ver DÓNDE rompe, antes de proponer fixes — SD líneas 72-108
- [ ] Fase 1 · trazar el data flow hacia atrás hasta el origen del valor malo; fix en la fuente, no en el síntoma — SD líneas 110-120 + root-cause-tracing.md (§Overview: "Trace backward through the call chain until you find the original trigger")
- [ ] Fase 2 · comparar contra ejemplos que funcionan: localizar código similar working, leer la referencia COMPLETA, listar todas las diferencias sin descartar "eso no puede importar" — SD líneas 122-143
- [ ] Fase 3 · hipótesis única y explícita ("creo que X porque Y"), test mínimo de una variable, verificar antes de seguir; si falla ⇒ NUEVA hipótesis, no apilar fixes; decir "no entiendo X" en vez de fingir — SD líneas 145-168
- [ ] Fase 4 · failing test que reproduce el bug ANTES del fix (vía `process:tdd`), UN fix al root cause (sin "while I'm here"), verificar que resuelve y nada más se rompe — SD líneas 170-191
- [ ] 3+ fixes fallidos ⇒ parar y cuestionar la arquitectura (cada fix revela acoplamiento nuevo en otro sitio = patrón, no hipótesis fallida); discutir con el humano antes de más fixes — SD líneas 192-213
- [ ] "No root cause" verdadero (ambiental/timing/externo): documentar lo investigado + handling apropiado (retry/timeout/mensaje) + monitoring; pero 95% de los "no root cause" son investigación incompleta — SD líneas 267-276
- [ ] Técnicas de soporte disponibles como referencia: root-cause-tracing, defense-in-depth (validar en cada capa tras hallar la causa), condition-based-waiting (esperar la condición, no un sleep) — SD líneas 278-284 + los tres .md fuente

## Movimientos — recon-first

- [ ] Anti-patrón nombrado: cambiar una variable y reintentar a ciegas ante el mismo error; repetir no es progreso; el desbloqueo es parar a recoger información — RF líneas 8-11
- [ ] Gate de dificultad: aplica con ≥3 intentos contra el mismo error / terreno desconocido / time-box quemándose; con priors sólidos y avanzando, NO frenarse por ritual — RF líneas 19-28
- [ ] Movimiento 1: parar de reintentar; nombrar explícitamente qué se intentó y por qué falló; si no sabes por qué falló, ESE es el problema — RF líneas 32-34
- [ ] Movimiento 2: retrieve > compute — error literal a la web, docs oficiales (preferibles a la memoria paramétrica, que puede estar stale), issues/changelog si huele a cambio de versión — RF líneas 35-38
- [ ] Movimiento 3: listar supuestos y verificar el más barato primero (un `--version`/`print`/`ls` resuelve más atascos que otra ronda de razonamiento) — RF líneas 39-42
- [ ] Movimiento 4: reducir el caso al mínimo aislado antes de seguir tocando el sistema entero — RF líneas 43-44
- [ ] Delegación: investigación voluminosa a un subagente barato; el orquestador se queda la conclusión, no el material crudo — RF líneas 46-50

## DESCARTES (corpus negativo)

- Referencia al hook `stuck-loop` en la description (RF línea 3): hook muerto, commit 590d6ca — framework §5.2. (Duplicado arriba como ítem verificable porque es LA corrección que esta fusión debe recoger.)
- Referencia cruzada de recon-first a `superpowers:systematic-debugging` como skill externa (RF líneas 52-56): ambas fuentes son ahora la misma skill.
- IRON LAW en mayúsculas (SD líneas 16-22) y "Violating the letter…" (SD línea 14): gritos — framework §5.2; la regla se conserva destilada.
- "your human partner's Signals You're Doing It Wrong" (SD líneas 234-243) y §Common Rationalizations (SD líneas 245-256): prosa de coaching / catálogo de excusas — framework §5.2 "se tira la prosa … y los gates dogmáticos".
- §Real-World Impact con métricas anecdóticas (SD líneas 290-296): prosa.
- Digraphs dot (SD y los tres .md de técnicas): prosa/formato.
- Párrafo de validación con citas a agent-solve-it y literatura (RF líneas 13-15): justificación, no movimiento; el porqué personal vive en la KB (framework §3).
- Ficheros de creación/test del directorio fuente (`CREATION-LOG.md`, `test-academic.md`, `test-pressure-*.md`, `find-polluter.sh`, `condition-based-waiting-example.ts`): artefactos de desarrollo de la skill original, no contenido de la skill.
