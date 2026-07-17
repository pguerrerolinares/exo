# Verdict de gate — rama `prep-m3-impl` (implementación de las 7 skills de `process`)

- **Fecha**: 2026-07-17T16:25:37+02:00
- **Adjudicador**: consultor fable delegado (régimen config `.superpowers/fabrica/config.md` §Ejecución de gates; dispatch fresco, sin participación en ninguna fase de la pieza).
- **Deliverable**: rama `prep-m3-impl`, base `master@548e00b`, 9 commits (`a49cc15..a8e6b3a`).
- **Criterio**: spec `docs/superpowers/specs/2026-07-17-prep-m3-process-skills-design.md` (§3 formato, §5.x diseño, §9 kill-criteria) + oráculo `evals/prep-m3/gold/*.md` con procedimiento `evals/prep-m3/README.md` (paridad 100%, 0 movimientos nuevos sin cita, 0 descartes resucitados).
- **Método**: verificación primaria propia — recorrido EXHAUSTIVO de los 135 checkboxes gold contra la implementación real (no muestreo), verificación de ausencia de TODOS los descartes (lectura + greps), barrido inverso, y spot-checks contra las fuentes originales (orchestrate-personal 0.5.0, recon-first 0.9.0, `~/.claude/commands/documenta.md`, SDD SKILL.md/implementer-prompt.md/task-reviewer-prompt.md, executing-plans, dispatching-parallel-agents, scripts). No se ha usado ningún self-report de los implementers como evidencia.

## VEREDICTO: RECHAZADA

**Paridad: 134/135.** Un único fallo de paridad, en `documenta`:

### Fallo bloqueante — documenta, gold ítem 6 (gold/documenta.md L13): sub-cláusula perdida

Gold: *"Estado del frente en el backlog: cerrados `[ ]`→`[x]` en UNA línea sin duplicar el detalle (vive en la bitácora); solo estado abierto + cola corta de recién-cerrado; **el barrido de `[x]` viejos es de /consolida, no de documenta** — línea 43"*.

Fuente (`~/.claude/commands/documenta.md` L43): *"El barrido de los `[x]` viejos a `log/backlog-diario.md` lo hace `/consolida`, no tú aquí"*.

Implementación (`plugins/process/skills/documenta/SKILL.md` L30-32): *"Backlog: cerrados `[ ]`→`[x]` en UNA línea sin duplicar el detalle — solo estado abierto + cola corta de recién-cerrado."* — las dos primeras cláusulas presentes; la tercera AUSENTE. `grep -rn 'consolida\|barrido' plugins/process/` = 0 matches (ni en `routing.md` L19, que repite solo "edita `[ ]`→`[x]` en UNA línea, sin duplicar el detalle").

Por qué no es "redacción destilada" sino regla perdida: la cláusula es una prohibición de scope con efecto conductual — asigna el barrido a otra pieza (la consolidación offline). Sin ella, la frase conservada "solo estado abierto + cola corta de recién-cerrado" describe el estado objetivo del backlog **sin actor para el barrido**, e invita a que documenta lo haga ella misma (exactamente lo que la fuente prohíbe). El gold la mantuvo deliberadamente (a diferencia del ítem 16, donde el propio gold anota la despersonalización de `sesiones/`). Criterio de cierre (README L42 + spec §10): paridad 100% — 134/135 no lo es; §9.3 por simetría: un movimiento perdido silencioso es el fallo de paridad central que este oráculo existe para cazar.

**Fix accionable (una línea, y re-gate rápido)**: reponer la cláusula en `documenta/SKILL.md` Paso 2 (y opcionalmente la fila de backlog de `routing.md`), en forma genérica si se prefiere no nombrar `/consolida` desde process (p.ej. *"el barrido de `[x]` viejos pertenece a la consolidación offline, no a esta skill"*) — la despersonalización del nombre es adjudicable; la prohibición no.

### Recomendaciones NO bloqueantes (compresiones a vigilar, mismas fix-pass si se quiere)

1. **orchestrate, gold ítem SDD-7 (gold L23)** — "⚠️ los resuelve el orquestador; si es gap real ⇒ vuelta al implementer y re-review". Impl: `orchestrate/SKILL.md` L49-50 conserva "los ⚠️ 'cannot verify' los resuelve el orquestador"; la consecuencia explícita ("gap confirmado = failed spec review → implementer + re-review") queda implícita en el fix-loop (L50-53) y en "el padre valida SIEMPRE — nunca auto-aprobar inline" (L64-65). Lo marco PRESENTE (destilado): la vía de re-entrada existe y la skill prohíbe dar por bueno sin validar. Explicitar la consecuencia costaría media línea.
2. **orchestrate, gold ítem SDD-15 (gold L31)** — "el re-review se despacha solo con tests+comando+output presentes". Impl: la sustancia (la evidencia del fix viaja en el report y el reviewer se apoya en ella) vive distribuida en `implementer-prompt.md` L77-79 ("re-corre los tests… añade los resultados a tu report file — el reviewer no re-corre tests por ti; tu report es la evidencia") + `SKILL.md` L52-53 + `reviewer-prompt.md` L211. Lo marco PRESENTE (destilado, distribuido); el check orquestador-side pre-dispatch es redundancia de enforcement que se perdió como imperativo.

## Las 4 condiciones del régimen (config §Ejecución de gates)

1. **Fresco**: dispatch nuevo, briefeado solo con deliverable + criterio. ✓
2. **Verificación primaria propia**: oráculo ejecutado por mí checkbox a checkbox (este documento ES la corrida del oráculo); fuentes releídas de disco. ✓
3. **Mandato de disenso**: sección "Qué busqué para objetar" abajo. ✓
4. **Verdict-artifact commiteado** en path versionado, antes de cualquier GATE-EXEC. ✓ (este fichero)

## Contrato §3 (formato de skill)

- **Frontmatter** `name` + `description` en las 7; descriptions coinciden con las propuestas de spec §5.1-5.7 (documenta dice "la KB", no basic-memory — correcto per §5.7). Debug lleva las DOS puertas en la description (`debug/SKILL.md` L3) sin mención a `stuck-loop` (grep = 0). ✓
- **Castellano** con términos técnicos en inglés en bodies y reference files; scripts en inglés = "casi tal cual" per spec §5.3, correcto. Sin calcos raros detectados (busqué expresamente). ✓
- **Bodies no-blank sin frontmatter** (conteo propio, awk): brainstorm 37, plan 39, orchestrate **54**, tdd 39, debug 37, verify 43, documenta 38. Seis en rango ~30-50; orchestrate fuera por 4 — ver adjudicación abajo. ✓/desviación
- **Degradación SOLO en orchestrate y documenta**: orchestrate `SKILL.md` L33-34 (memory packet), documenta `SKILL.md` L23-26. Verificado que brainstorm/plan/tdd/debug/verify NO llevan probe ni degradación. ✓
- **Reference files = exactamente los de spec §5.x**: plan-template.md, implementer-prompt.md, reviewer-prompt.md, scripts/{task-brief,review-package,sdd-workspace}, anti-patterns.md, techniques.md (un fichero, tres secciones), routing.md (placeholders, sin nombres de instancia). brainstorm y verify sin reference files (spec: "ninguno previsto"). ✓

## Scope y atribución

- `git diff --name-only master...prep-m3-impl` = 18 ficheros, TODOS bajo `plugins/process/`. Cero activaciones: nada en `.claude/`, marketplace, settings, hooks. ✓
- `LICENSES/superpowers.LICENSE` = MIT literal © 2025 Jesse Vincent (cotejada). README con tabla de atribución = tabla spec §4. Scripts: diff contra los de superpowers 6.1.1 = idénticos salvo las 2 líneas de header de origen que la spec §8 exige. ✓

## PARIDAD CRÍTICA (framework §5.3.2) — presencia literal verificada

- `orchestrate/SKILL.md` L12-16 (en el body, sección propia "PARIDAD CRÍTICA — no negociable"): *"`subagent_type: reflex:executor`, **nunca** `general-purpose`, **sin** `model` (el rol lo trae fijo — pasarlo lo pisaría). Si se pierde, reflex v2 se desenchufa sin síntoma."* ✓
- `orchestrate/implementer-prompt.md` L7-8: *"**Dispatch:** `subagent_type: reflex:executor` — **sin** `model` (paridad crítica: el rol trae modelo fijo, pasar `model` lo pisaría)"* + L11 `Subagent (reflex:executor):` — y NO hereda la línea `model: [MODEL — REQUIRED…]` del template SDD fuente (cotejado contra el original, que la lleva en L8-9 para `general-purpose`). ✓
- El `general-purpose` de `reviewer-prompt.md` L44 es el rol REVIEWER con `model` explícito escalado — consistente con el gold (ítem SDD-23, "matiz de paridad crítica") y con el descarte que solo prohíbe `general-purpose` para el rol ejecutor. NO es descarte resucitado. ✓

## Adjudicación de la desviación declarada: orchestrate body 54 no-blank vs "~50"

**ACEPTADA (no revoco).** Fundamento: (a) el techo de spec §3.2 lleva tilde ("~30-50"), y 54 es un 8% sobre el extremo con 42 movimientos gold que cubrir — la mayor carga de las 7 con diferencia; (b) el kill-criterio §9.1 ya se ejecutó en la dirección correcta (commit `a8e6b3a`: carne a reference files, −68 líneas netas; la skill ya descarga en 5 reference files); (c) forzar −4 líneas más solo puede salir de recortar movimientos, que es exactamente lo que §9.1 prohíbe ("escalar al consultor-gate, no recortar el checklist" — escalado hecho: yo soy ese gate y lo adjudico así). Los otros seis bodies caben en rango estricto, así que la desviación no es patrón sino coste real de la fusión de 4 fuentes.

## Tabla de paridad — resumen por skill (ausentes/dudosos uno a uno)

Convención: ítems referidos por su línea en `evals/prep-m3/gold/<skill>.md`; evidencia en paths bajo `plugins/process/skills/`.

### brainstorm — 16/16 presentes
| Gold | Evidencia |
|---|---|
| L9 explorar contexto | brainstorm/SKILL.md L13 |
| L10 una pregunta a la vez, purpose/constraints/success | L18-20 |
| L11 multiple choice | L19-20 |
| L12 scope multi-subsistema antes de refinar | L14-17 |
| L13 2-3 enfoques, recomendación primero | L21-22 |
| L14 diseño antes de código | L31-36 (sección Gate) |
| L15 anti-patrón "too simple" | L34-36 |
| L16 secciones escaladas + validación incremental | L23-25 |
| L17 cobertura arquitectura/componentes/data flow/errores/testing | L24-25 |
| L18 diseño para aislamiento | L26-27 |
| L19 codebases existentes, sin refactor no relacionado | L28-29 |
| L20 spec a docs/superpowers/specs/ + commit | L40-42 |
| L21 self-review inline | L43-44 |
| L22 gate de review del usuario | L45-46 |
| L23 terminal = process:plan | L9 + L47 |
| L24 YAGNI | L51 |

### plan — 13/13 presentes
| Gold | Evidencia |
|---|---|
| L8 lector cero contexto | plan/SKILL.md L8-9 |
| L9 docs/superpowers/plans/ | L9-11 |
| L10 scope check → planes separados | L15-17 |
| L11 file structure antes de tareas | L18-20 |
| L12 task right-sizing | L21-23 |
| L13 pasos 2-5 min con ciclo | L24-25 |
| L14 header obligatorio + constraints verbatim | L29-31 + plan-template.md L18-29 |
| L15 pointer "For agentic workers" + checkbox | L31-33 + plan-template.md L14-16 |
| L16 Files + Interfaces con firmas | L35-38 + plan-template.md L37-48 |
| L17 no-placeholders | L42-45 + plan-template.md L83-96 |
| L18 recordatorio operativo | L45-47 |
| L19 self-review contra spec | L51-53 |
| L20 handoff único a process:orchestrate | L55 |

### orchestrate — 42/42 presentes (2 destilados con recomendación, arriba)
| Gold | Evidencia |
|---|---|
| L13 PARIDAD CRÍTICA reflex:executor sin model | SKILL.md L12-16 + implementer-prompt.md L7-11 (sección propia arriba) |
| L17 fresco/aislado | SKILL.md L8-9 |
| L18 ejecución continua | L23-25 |
| L19 narración mínima | L10 |
| L20 pre-flight plan review, pregunta batcheada | L19-21 |
| L21 dos verdictos + review final | L3 (description) + L49 + reviewer-prompt.md L3-4 |
| L22 estados implementer, nunca retry sin cambio | L43-46 |
| L23 ⚠️ resuelve orquestador | L49-50 (destilado; recomendación 1) |
| L24 no pre-juzgar findings | L48 + reviewer-prompt.md L30-33 |
| L25 constraints verbatim | L47 + reviewer-prompt.md L20-24, L195-198 |
| L26 file handoffs | L30-32 |
| L27 BASE registrado, nunca HEAD~1 | L48-49 + reviewer-prompt.md L34-37 |
| L28 un dispatch = una tarea | L29-30 + implementer-prompt.md L113-115 |
| L29 fix Critical/Important, Minor→ledger+triaje | L50-51 |
| L30 plan-mandated ⇒ humano | L51-52 + reviewer-prompt.md L153-157 |
| L31 fix re-corre tests / re-review con evidencia | L52-53 + implementer-prompt.md L77-79 (destilado; recomendación 2) |
| L32 final ⇒ UN fix subagent | L53-54 |
| L33 sin directivas open-ended | L47-48 + reviewer-prompt.md L26-27 |
| L34 no re-pedir tests corridos | L48 + reviewer-prompt.md L28-29, L93-100 |
| L35 review final con MERGE_BASE package | L49 + reviewer-prompt.md L38-41, L205-208 |
| L36 ledger durable, nunca re-despachar completa | L22-23 + L62-64 + L70 |
| L37 nunca main/master sin consentimiento | L68-69 |
| L38 nunca implementers paralelos mismo estado | L69-70 |
| L39 model explícito (salvo rol fijo) | L35-36 |
| L40 turn-count > token-price | L39 |
| L44 revisión crítica del plan | L19-21 |
| L45 blocker ⇒ parar y preguntar | L24-25 |
| L49 paralelizar solo dominios independientes, un mensaje | L58-59 |
| L50 cuándo NO paralelizar | L60-61 |
| L51 prompt paralelo: scope/self-contained/constraints/output | L59-60 |
| L52 al volver: summaries/choques/suite/spot-check | L61-62 |
| L56 cost pyramid | L36-38 |
| L57 reviewer escalado al riesgo del diff | L38-39 + reviewer-prompt.md L9-13 |
| L58 memory packet 3-5 permalinks | L32-33 |
| L59 degradación aviso visible (packet) | L33-34 |
| L60 brief completeness + blindspot pass | L34-35 |
| L61 delegate by default | L35 |
| L62 hijo se auto-revisa, padre valida SIEMPRE | L64-65 |
| L63 pre-flight recon de refs | L21-22 |
| L64 controller filtra y decide; doc/comment inline | L50-53 |
| L65 autonomous runs | L66-68 |
| L66 investigate, don't stop | L65-66 |

### tdd — 14/14 presentes
| Gold | Evidencia |
|---|---|
| L8 ciclo completo | tdd/SKILL.md L8-9 + L21-35 |
| L9 core principle | L8-9 |
| L10 no producción sin test que falló | L13-14 |
| L11 código previo se borra | L14-17 |
| L12 verify RED | L26-28 |
| L13 GREEN mínimo | L29-30 |
| L14 verify GREEN | L31-33 |
| L15 refactor solo en verde | L34-35 |
| L16 propiedades de buen test | L23-25 |
| L17 excepciones con permiso | L18-19 |
| L18 bug = failing test primero | L39-40 |
| L19 checklist pre-completado | L44-46 |
| L20 tabla when-stuck | L50-52 |
| L21 anti-patrones + gate | L56-58 + anti-patterns.md L11-42 (gate L17-19) |

### debug — 21/21 presentes
| Gold | Evidencia |
|---|---|
| L11 description dos puertas | debug/SKILL.md L3 |
| L12 sin stuck-loop | grep = 0 en todo el plugin |
| L16 root cause antes de fix | L8-9 |
| L17 errores completos | L13 |
| L18 reproducir | L13-14 |
| L19 cambios recientes | L14 |
| L20 boundaries multi-componente | L14-16 |
| L21 data flow hacia atrás | L16-17 + techniques.md L12-25 |
| L22 comparar con working + dependencias | L17-19 |
| L23 hipótesis única | L19-22 |
| L24 failing test + UN fix (tdd) | L22-24 |
| L25 3+ fixes ⇒ arquitectura + humano | L26-28 |
| L26 "no root cause" verdadero / 95% | L28-31 |
| L27 técnicas de soporte | L31-32 + techniques.md (tres secciones) |
| L31 anti-patrón nombrado | L37-39 |
| L32 gate de dificultad / no ritual | L36-38 |
| L33 mov 1 parar y nombrar | L41-42 |
| L34 mov 2 retrieve > compute | L43-44 |
| L35 mov 3 supuestos, más barato primero | L45 |
| L36 mov 4 reducir el caso | L46-47 |
| L37 delegación | L49-50 |

### verify — 13/13 presentes
| Gold | Evidencia |
|---|---|
| L11 límite: NO despacha reviewers (ausencia verificada + regla explícita) | verify/SKILL.md L13-15; cero dispatches en la skill |
| L15 evidencia antes de claims | L8-11 |
| L16 sin evidencia fresca no hay claim | L19-20 |
| L17 gate function 5 pasos | L26-30 |
| L18 tabla claim→evidencia | L32-35 |
| L19 red flags | L44-47 |
| L20 red-green verificado | L37-38 |
| L21 reporte de agente no es evidencia | L40-41 |
| L22 toda variante del claim | L20-22 |
| L26 gate 1 ¿funciona? | L51 |
| L27 gate 2 verificación real (UI desktop+mobile / backend real) | L52-54 |
| L28 gate 3 calidad de ingeniería | L55-58 |
| L29 gate 4 escrutinio del diff + commit atómico | L59-61 |

### documenta — 15/16 presentes, 1 PARCIAL (bloqueante)
| Gold | Evidencia |
|---|---|
| L8 extraer qué persiste | documenta/SKILL.md L13-17 |
| L9 orientación barata (targets) | L20-23 |
| L10 degradación aviso visible | L23-26 |
| L11 regla de oro del routing | L28-29 + routing.md L13 |
| L12 avance ⇒ delta + append ≤15 | L29-31 + routing.md L17-18 |
| **L13 backlog `[ ]`→`[x]`… + barrido es de /consolida** | **PARCIAL: L30-32 conserva 2 de 3 cláusulas; prohibición del barrido AUSENTE (ver fallo bloqueante)** |
| L14 transversal ⇒ learnings; dueño ⇒ perfil | L32-34 + routing.md L21-22 |
| L15 nota nueva SOLO para | L34-35 + routing.md L23-25 |
| L16 frontmatter tags+tier | L40 + routing.md L27-32 |
| L17 search-before-write | L41 |
| L18 editar no duplicar, preferir append | L41-43 + routing.md L40-46 |
| L19 títulos consistentes | L43-44 + routing.md L34-38 |
| L20 commit scoped, git -C, nunca push | L48-50 (mensaje `docs(kb): documenta …` conservado L49-50) |
| L21 retry index.lock | L50-52 |
| L22 resumen final | L52-53 |
| L23 no "una nota por sesión" | L35-36 (despersonalización de `sesiones/` correcta per gold) |

## DESCARTES — ausencia verificada (los 7 ficheros, lectura completa + greps)

- **brainstorm**: sin visual companion/scripts propios, sin spec-document-reviewer, sin digraph, sin `<HARD-GATE>`/"You MUST create a task". ✓
- **plan**: sin "Announce at start", sin `using-git-worktrees`, handoff único (sin bifurcación de modos), sin plan-document-reviewer. ✓
- **orchestrate**: sin digraphs, sin Example Workflow/Real Example/Real-World Impact, sin Advantages/Efficiency/Quality/Cost, sin announce de executing-plans ni "works much better with subagents", sin referencias a skills `superpowers:*` (grep = 0), dispatch ejecutor `general-purpose` sustituido (el de reviewer-prompt.md L44 es rol reviewer, legítimo), sin "Do NOT reinvent orchestration", sin "Fuente canónica … manda la nota" (grep basic-memory = 0). ✓
- **tdd**: sin "Violating the letter…", sin IRON LAW en mayúsculas, sin ensayo Why Order Matters, sin tablas de racionalizaciones/red flags como catálogo, sin ejemplo bug-fix completo ni digraph. ✓
- **debug**: sin stuck-loop, sin cross-ref a systematic-debugging como skill externa, sin IRON LAW, sin "Signals You're Doing It Wrong"/Common Rationalizations, sin Real-World Impact, sin digraphs, sin párrafo de validación con citas, sin artefactos de desarrollo (CREATION-LOG etc.), sin tabla "Red Flags - STOP", sin "Fuente canónica…". ✓
- **verify**: sin IRON LAW/"Skip any step = lying"/"Violating the letter", sin tabla Rationalization Prevention, sin Why This Matters/"24 failure memories", sin la parte del parent gate que despacha o coordina (verificado: cero dispatches). ✓
- **documenta**: sin gramática de observations `- [categoria]`/relations como estructura (wikilinks conservados como contrato, routing.md L34-38), sin hardcode `mcp__basic-memory__*` ni nombres de nota de instancia (routing.md usa placeholders), sin trailer Co-Authored-By (grep = 0; mensaje scoped conservado). ✓

**0 descartes resucitados.**

## Barrido inverso — movimientos nuevos sin cita

Recorrido de los 7 SKILL.md + 8 reference files + 3 scripts buscando contenido no trazable a gold/spec/fuente:

- `implementer-prompt.md`: secciones "Antes de empezar", "Cuando estás en apuros", self-review por categorías, "Mientras iteras… suite completa una vez antes de commitear", report format — TODAS cotejadas línea a línea contra el `implementer-prompt.md` fuente de SDD (leído entero): traducción destilada, nada inventado. La sección "Code Organization" de la fuente se omitió — omisión, no adición, y no es movimiento gold.
- `reviewer-prompt.md`: "read-only sobre este checkout", "no confíes en el report", sección Tests, calibración (incl. "Reconoce lo que está bien hecho"), output format — cotejadas contra `task-reviewer-prompt.md` fuente (leído entero): traducción fiel. La guía de escalado de modelo viene de OP L38-40 (citada). El fallback "si falta el diff file, deriva el diff tú mismo" de la fuente se omitió — omisión menor, no adición.
- `techniques.md`: las tres secciones cotejadas conceptualmente contra los tres .md fuente de systematic-debugging; el detalle (4 capas de defense-in-depth, polling ~10ms, cuándo un timeout fijo es correcto) es contenido de esas fuentes, citadas en el header. Nada nuevo.
- `plan-template.md`, `anti-patterns.md`, `routing.md`: contenido = spec §5.2/§5.4/§5.7 + fuente citada en header. Nada nuevo.
- Scripts: idénticos a la fuente (diff), salvo header de origen exigido por spec §8.

**0 movimientos nuevos sin cita.**

## Qué busqué para objetar (mandato de disenso)

1. **Paráfrasis que pierden la regla**: releí las fuentes originales de disco (OP 0.5.0, recon-first 0.9.0, documenta.md, SDD SKILL.md L5-20/85-97/110-217/246-264/366-390, EP, DPA, ambos prompt-templates de SDD) y comparé cláusula a cláusula los ítems con más compresión. Encontré UNA regla perdida (documenta, barrido → bloqueante) y dos compresiones al límite pero con sustancia conservada (orchestrate ⚠️-gap y evidencia de re-review → recomendaciones). El resto de compresiones que revisé pierden solo ejemplos o racionales (p.ej. "--version/print/ls", "memoria paramétrica stale"), no reglas.
2. **Descartes resucitados con otras palabras**: greps por sinónimos y variantes (visual, digraph, announce, rationaliz-, works much better, Real Example, IRON/HARD, categoria], tipo_relacion, Co-Authored, superpowers:, basic-memory, stuck-loop) + lectura completa de los 18 ficheros. Único match: "prueba visual" en verify L54, que es la regla gold VBC-Gate2, no el visual companion. Nada resucitado.
3. **Punteros a reference files que no contienen lo prometido**: verifiqué cada puntero de los bodies (plan→plan-template, tdd→anti-patterns, debug→techniques, documenta→routing, orchestrate→implementer/reviewer-prompt/scripts) abriendo el destino y comprobando que lleva la carne prometida. Todos cumplen.
4. **Degradación en skills que no deben llevarla**: busqué probes/fallbacks en brainstorm/plan/tdd/debug/verify — ninguno. Solo orchestrate y documenta, como manda §3.4.
5. **Activaciones escondidas**: revisé la lista completa del diff (18 paths, todos `plugins/process/`), busqué hooks, settings, marketplace, SessionStart — nada. Los scripts escriben solo bajo `.superpowers/sdd/` del working tree (gitignored por diseño).
6. **Paridad crítica degradada a paráfrasis**: exigí presencia LITERAL de `subagent_type: reflex:executor` sin `model` en body y en implementer-prompt — está literal en ambos, y comprobé contra la fuente SDD que la línea `model: [MODEL — REQUIRED…]` del template original NO se heredó para el rol ejecutor.
7. **La desviación de 54 líneas como grieta**: intenté construir el caso para revocarla (recontar con otro criterio, buscar relleno recortable). El conteo es reproducible (54 no-blank) y el "relleno" restante son movimientos gold enunciados a una línea; recortar = perder movimientos (§9.1). No hay caso para revocar.
8. **Castellano con calcos**: barrido de los bodies buscando anglicismos forzados o calcos de estructura — el registro es el del criterio (castellano + términos técnicos en inglés); nada objetable.

## Consecuencia

Un fix de una línea en `documenta` (± las dos explicitaciones recomendadas en orchestrate) y re-gate. El cap de retries (§9.2: 2 por eval) no está consumido para documenta. El resto de la rama queda verificado y no necesita re-verificación completa en el re-gate: bastará re-correr el checklist de documenta + re-grep de descartes sobre el diff del fix.

---

# ADDENDUM — Re-gate tras el fix `fc51e09`

- **Fecha**: 2026-07-17 (mismo consultor, re-gate acotado según la consecuencia prescrita arriba).
- **Alcance verificado**: `git show fc51e09` completo — UN fichero (`plugins/process/skills/documenta/SKILL.md`), 2 inserciones / 1 borrado (parte la línea 32 y añade la cláusula). Nada más tocado; working tree limpio. El resto de la rama es byte-idéntico a lo ya verificado (134/135 + paridad crítica + scope + descartes de las otras 6 skills siguen válidos sin re-verificar).

## Re-ejecución del gold de documenta: 16/16 PRESENTES

- **Ítem 6 (gold L13), el bloqueante**: ahora COMPLETO. `documenta/SKILL.md` L30-33: *"Backlog: cerrados `[ ]`→`[x]` en UNA línea sin duplicar el detalle — solo estado abierto + cola corta de recién-cerrado; **el barrido de `[x]` viejos es de la consolidación (/consolida), no de documenta**."* Las tres cláusulas del checkbox presentes; la formulación nombra `/consolida` igual que el gold (referencia textual, no activación — /consolida sigue en reflex, nada se instala ni enlaza).
- **Los otros 15 ítems**: re-verificados sobre el fichero post-fix (los números de línea corren +1 a partir de L33): extraer L13-17 ✓; targets L20-23 ✓; degradación L22-26 ✓; regla de oro L27-28 ✓; delta+append L28-30 ✓; transversal/perfil L33-35 ✓; nota nueva SOLO L35-36 ✓; no "nota por sesión" L36-37 ✓; frontmatter L41 ✓; search-before-write L42 ✓; append preferido L42-44 ✓; títulos L44-45 ✓; commit scoped/`git -C`/nunca push/mensaje `docs(kb)` L49-51 ✓; retry index.lock L51-53 ✓; resumen final L53-54 ✓.
- **Body**: 39 no-blank (era 38; +1 por la cláusula) — en rango ~30-50. ✓

## DESCARTES de documenta re-verificados por ausencia

Grep post-fix sobre `documenta/`: sin gramática `- [categoria]`/`tipo_relacion` como estructura, sin `mcp__*` hardcodeado, sin trailer Co-Authored-By, sin `sesiones/`. Único match de `basic-memory`: L8 "vía engine — hoy kbx/basic-memory/filesystem" — es la formulación que la spec §3.4/framework §5.1 exige literalmente, no el hardcode de tools MCP que el descarte prohíbe; ya estaba en la versión gateada y fc51e09 no la toca. **0 descartes resucitados.**

## Qué busqué para objetar (re-gate)

Que el fix introdujera algo más que la cláusula (diff completo leído: no), que rompiera el rango del body (39: no), que la mención a `/consolida` fuera una activación o dependencia nueva (es texto, el plugin no gana hooks/skills/refs instalables), y que el line-shift dejara algún ítem vecino cortado (releído el fichero entero: no).

## VEREDICTO FINAL: MERGED

**Paridad 135/135, 0 movimientos nuevos sin cita, 0 descartes resucitados.** Criterio de cierre de spec §10 y README cumplido. Desviación orchestrate 54 no-blank: aceptada (adjudicación arriba, sin cambios). Las dos recomendaciones no bloqueantes de orchestrate quedan como mejora futura opcional, no condición.
