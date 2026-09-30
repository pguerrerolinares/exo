# Recorte del mecanismo de memoria — recall por prompt fuera, reglas de proyecto tras un gate

**Fecha:** 2026-09-30 · **Estado:** diseño validado por secciones en sesión; sección 2 auditada por Fable (dos rondas) y ajustada.

## Problema

La campaña K (ablación pre-registrada, 09-30) no encontró efecto concluyente de exo: S1 Δ +6 pp, IC95 [−2,5, +16,3], con +22 % de tokens y +37 % de tiempo. La pregunta de partida fue si exo debería quedarse solo como memoria. La evidencia apunta al revés: la parte que peor salió es un **mecanismo** de la memoria, no la mitad de proceso.

- El recall por prompt (`recall-inject.sh`) trajo la nota fuente en 3/40 tareas, y el agente abre lo inyectado en el 6 % de los prompts.
- Las ayudas limpias (g0-12, g1-14) vinieron del core-index de arranque (`exo-recall.sh`).
- **Literatura:** el contexto recuperado sin gate degrada.
  - Shi et al. 2023, arXiv 2302.00093.
  - Liu et al. 2023, arXiv 2307.03172.
  - Self-RAG, arXiv 2310.11511.
  - Adaptive-RAG, arXiv 2403.14403.
  - Un fichero de contexto mínimo escrito por humanos da como mucho +4 %, y uno generado por LLM resta, con un coste de +20 % (Gloaguen et al., ICLR 2026, arXiv 2602.11988).
  - El cumplimiento cae con el número de instrucciones (IFScale, arXiv 2507.11538).
- **Industria:** convergencia en reglas cortas siempre en contexto y detalle a demanda del agente. Ningún producto usa la inyección por prompt como mecanismo principal (Claude Code, Letta, Copilot memory, memory tool de Anthropic).

## Decisiones

| Eje | Decisión | Descartado |
|---|---|---|
| Alcance | Recortar el mecanismo, no una mitad. Proceso y core-index se quedan | Solo memoria: conserva lo que peor salió y revierte la decisión A del 09-30. Medir antes (2×2): harían falta ~200+ pares |
| Recall por prompt | Borrar hook, script y tests | Apagarlo por flag: sería código muerto sin una medición prevista |
| Reglas de proyecto que no llegan | Test de techo pre-registrado; la 2d solo si pasa | 2a, guard en `bash-guards.sh`: sus ramas solo loguean, así que no cambia conducta. 2b, regla en el `CLAUDE.md` del repo: K no la midió, duplica canon y ensucia repos de cliente. 2c, hooks por proyecto: over-engineering sin tasa de disparo medida |

## Sección 1 — Borrar el recall por prompt

**Qué se borra:**
- La entrada `UserPromptSubmit` de `plugins/exo/hooks/hooks.json`.
- `plugins/exo/scripts/recall-inject.sh`, `test-recall-inject.sh` y `test-recall-inject-golden.sh`.
- `recall-latencia.sh` y `test-recall-latencia.sh`, **solo si** no tienen otro consumidor. El plan lo verifica con grep antes de borrar.

**Qué se barre:**
- Las referencias a `recall-inject` en los scripts hermanos:
  - `search-first.sh`
  - `subagent-inject.sh`
  - `kb-precommit.sh`
  - `_hook-ms.sh`
  - `exo-recall.sh`
  - `_engine-version.sh`
  - `_timeout.sh`
  - `test-contrato-engine.sh`
  - `test-exo-recall.sh`
  - `test-exo-index.sh`
  - `test-kb-precommit.sh`
  - `test-hook-ms.sh`
- Los dos README y `.superpowers/fabrica/config.md`.
- **Regla del barrido:** ninguna referencia viva a un fichero inexistente. Si un comentario explica un patrón compartido "igual que recall-inject", se reescribe para que se sostenga solo.

**Qué no se toca:**
- `evals/recall-coste/`, que es histórico medido.
- Las specs y planes antiguos en `docs/superpowers/`.
- El engine: `exo recall` y `exo search` siguen.
- `exo-recall.sh`, salvo sus comentarios.
- `subagent-inject.sh`, salvo sus comentarios.

**Sustituto:**
- La búsqueda queda a demanda: `exo search` vía Bash, más el reflejo `search-first.sh`.
- **Riesgo aceptado:** en headless la búsqueda a demanda es casi inexistente. En 120 corridas A3 de K hubo `exo search/targets` 1 vez, con el core-index pidiéndolo. En fábrica nada sustituye al recall hasta que el gate de la sección 2 decida.

**Verificación:**
- La suite `plugins/exo/scripts/test-*.sh` en verde.
- `grep -rn recall-inject` solo con aciertos en `evals/` y `docs/superpowers/{specs,plans,consultas}`.
- Una sesión real tras reinstalar el plugin: el prompt no recibe el bloque "Recall exo" y el arranque sí recibe el core-index.
- Bump de versión del plugin (`plugin.json` y `marketplace.json`).

## Sección 2 — Test de techo de las reglas de proyecto (gate de la 2d)

**Pregunta:** de las reglas que en K no llegaron, ¿cuáles cumple el agente cuando **sí** le llegan por el canal real? Si con la regla en contexto sigue fallando, ningún mecanismo de entrega lo arregla.

**Hechos que condicionan el diseño:**
- **Verificado en vivo:** `claude -p --setting-sources ""` no carga el `./CLAUDE.md` del repo. En K ningún brazo lo vio, y el estrato S1b no midió lo que decía.
- En g2-170, A3 leyó explícitamente el `CLAUDE.md` con la regla (`#c20f1e`) y la violó igual: la tarea pedía 4 colores. Hay conflicto regla-tarea, así que el techo no es 100 %.
- De las 13 del suelo de K, g1-147 no es violación y g1-57 solo muerde bajo el guard de fábrica. Quedan **11 tareas**.

**Diseño del test:**
- **Rama:** nueva, desde `main` (el harness de K, `evals/ablacion-k/harness/`, está en `main` desde el PR #32). Directorio `evals/techo-reglas/`.
- **Brazo único, canal real:** A0 más un hook SessionStart de ~5 líneas en el `settings.json` de la corrida, que emite la regla literal de la tarea como `additionalContext`. Sin exo. No se usa `--append-system-prompt-file`: el system prompt no es el canal que usaría la 2d.
- **Tareas:** las 11 del suelo, con k=2, 22 corridas.
- **Control positivo:** 6 tareas S1 que A0 pasa 2/2, con su regla inyectada por el mismo canal, k=1, 6 corridas. Detecta distracción por la inyección.
- **Total:** ~28 corridas, del orden de 4 USD.
- **Pre-registro:**
  - Qué se congela: las tareas, los controles, el texto literal de cada regla, el canal, k y el criterio.
  - Se commitea **antes** de la primera corrida.
  - Las erratas posteriores van a un fichero de erratas, como en K.

**Criterio (fijado por Paul):**
- **Pasa:** ≥6/11 tareas cumplen (una tarea "cumple" si pasa su check en ≥1 de 2 réplicas) **y** 0/6 caídas en el control positivo. Entonces se abre el ciclo de la 2d.
- **No pasa:** la sección se cierra con "las reglas no entregadas no eran entregables". Lo siguiente es revisar reglas y checks, no mecanismos.
- En cualquier caso, **lectura por tarea**. Cada tarea que falla con la regla en contexto se clasifica a mano como conflicto regla-tarea, check roto o regla mal escrita. Es diagnóstico, no adjudicación.

**Salida:** `evals/techo-reglas/verdict.md` con la tabla por tarea y el veredicto del gate.

## Contorno de la 2d (solo si el gate pasa; ciclo spec→plan propio)

Esto no es diseño: es el contorno acordado, para que el ciclo siguiente no reabra lo ya decidido.

- **Qué:** una sección `## Reglas duras` (≤10 líneas) en la nota-puerta `projects/*` de cada repo. La KB es la única fuente de verdad y no se escribe nada en los repos.
- **Entrega:** un bloque en SessionStart **con cap propio**, separado del core-index. El cap de 6.144 B de `exo-recall.sh:36` va a 5.921 B y trunca en silencio. Entrega también por SubagentStart con el perfil del executor.
- **Mapeo cwd→nota:** `basename(dirname(git rev-parse --git-common-dir))`, que cubre worktrees y subdirectorios, más un alias en el frontmatter cuando el nombre no casa. En las familias partidas, solo la nota-puerta.
- **Skip que grita:**
  - Casos: sin git, sin nota, sin sección, o más de una candidata.
  - Se loguea `project-rules-skip reason=…` en reflex-log.
  - Se emite una línea visible: "sin reglas de proyecto para `<repo>`".
  - Nunca se elige una nota a ciegas.
- **`/document`:** una frase en el contrato de routing: si una regla es de un solo repo, además va a su sección `## Reglas duras`.
- **Riesgo que ese ciclo debe medir:** el efecto de apilar la sección con el digest y la doctrina en el subagente (IFScale). El canario `k-subagente` ya existe.

## Fuera de alcance

- **`fabrica-main-guard.sh`** (agent-develop, paul-profile): su mensaje de rechazo debería decir "bájalo en una llamada aparte" (regla g1-57). Es una línea en otro repo; queda como pendiente aparte.
- **`subagent-inject.sh`:** también es inyección pasiva, pero no se midió en K. No se toca sin datos.
- **Proceso:** brainstorm, plan, orchestrate, tdd y verify quedan intactos. Siguen al día vía upstream-sync.

## Documentación en la KB (al cerrar)

- **Bitácora de exo:** K refutó *este* recall (query = prompt crudo, top-3, sin filtrar notas `log`), no la idea de recuperar por prompt. Y en headless nada lo sustituye hasta que decida el gate.
- **Learning** (en la nota de medición o en la de hechos del harness, no en una nota nueva): `--setting-sources ""` apaga el `CLAUDE.md` del repo. Cualquier ablación que quiera medir "regla en el repo" necesita `project`.
- **Canon de exo:** se actualiza el estado y se sustituye la línea de la campaña K por su consecuencia.

## Testing

- **Sección 1:** la suite de scripts en verde; el grep del barrido; la sesión real descrita arriba.
- **Sección 2:** el pre-registro commiteado antes de correr. El harness hereda los controles de fuga de K. Además, **sonda de canal**: una corrida con un codeword en el hook SessionStart debe devolverlo, antes de gastar las 28 corridas.
