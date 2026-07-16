# Informe — consultoría adversarial #3: CAPA THIN (Sección 3)

Consultor independiente. Verificación primaria hecha sobre: superpowers 6.1.1 instalado (skills, LICENSE, hooks, RELEASE-NOTES), plugins reflex/paul-profile reales (skills, hooks.json, scripts, git log), `~/.claude/commands/documenta.md`, `~/.claude/settings.json`, `~/.claude/plugins/known_marketplaces.json`, marketplace.json de agent-develop, y grep exhaustivo de la KB kb-demo. No re-litigo lo firmado.

## Hechos verificados (evidencia primaria)

1. **Licencia superpowers = MIT** (c) 2025 Jesse Vincent — `~/.claude/plugins/cache/claude-plugins-official/superpowers/6.1.1/LICENSE`. No hay problema AGPL-style.
2. **El hook stuck-loop de reflex está MUERTO.** Commit `590d6ca` en agent-develop: "chore(reflex): elimina stuck-loop y cost-pyramid (muertos, 0 señal)". El hooks.json actual de reflex no lo lista. La description de `recon-first` que dice "Triggered by the reflex stuck-loop hook" está **stale**.
3. **El SessionStart de superpowers** inyecta el SKILL.md íntegro de using-superpowers envuelto en `<EXTREMELY_IMPORTANT>` en startup/clear/compact (`hooks/session-start`). Es un forcing de *compliance* ("1% chance → MUST invoke"), no de *descubrimiento*: Claude Code ya lista todas las skills con sus descriptions en el contexto de forma nativa (verificado en esta misma sesión).
4. **Upstream tiene evidencia en ambos sentidos**: quitó el bootstrap en Codex porque "Codex reliably triggers skills on its own, and the bootstrap hook made the UX worse" (RELEASE-NOTES), pero lo MANTIENE en Claude Code y su test de aceptación para nuevos harnesses exige que "Let's make a react todo list" auto-dispare brainstorming en sesión limpia. Traducción: el disparo sin forcing es harness-dependiente y no está probado en Claude Code.
5. **Cero referencias vivas en la KB** a las 4 skills descartadas (writing-skills, using-git-worktrees, finishing-a-development-branch, receiving-code-review): grep sobre toda kb-demo devuelve nada. El descarte es limpio.
6. **fabrica SÍ tiene dependencias vivas de lo que muere**: su SKILL.md dice "el motor por pieza es `paul-profile:orchestrate-personal` (sobre superpowers:subagent-driven-development)". Al cutover, la primera se absorbe en `process:orchestrate` y la segunda se apaga. fabrica además usa paths `.superpowers/fabrica/` y su spec vive en `agent-develop/docs/superpowers/specs/`.
7. **fabrica lleva un hook**: `fabrica-main-guard.sh` como PreToolUse:Bash en paul-profile/hooks/hooks.json. Una "skill de instancia" suelta no puede llevar hooks — necesita seguir dentro de un plugin.
8. **El marketplace agent-develop está registrado vía GitHub** (`pguerrerolinares/agent-develop`, autoUpdate: true en known_marketplaces.json), no como path local. El patrón "marketplace en monorepo con pluginRoot ./plugins" está PROBADO — es exactamente cómo funciona agent-develop hoy. Pero su marketplace.json también lista **workflow-lint** (source: github repo aparte, enabled:true en settings) — el plan no dice dónde queda.
9. **El canal de routing por core-index ya existe**: `basic-memory-recall.sh` (SessionStart de reflex) inyecta `core/core-index` cacheado. Es instancia-side (reflex), no framework-side.
10. **El patrón "overlay con fallback" ya está probado en producción propia**: documenta.md usa `kbx targets` con fallback visible a `search_notes` ("kbx unavailable → search_notes fallback", nunca bloquea). Ese es exactamente el shape que el overlay KB-opcional necesita.

## Veredicto por eje

### Eje 1 — Mecánica de disparo: DÉBIL tal como está escrito

El plan afirma "using-superpowers desaparece sin sustituto: el routing lo hacen los frontmatter descriptions + core-index". Dos problemas:

- **La premisa de la pregunta del brief es correcta a medias.** El hook no existe porque los frontmatter "no disparen" — el descubrimiento nativo funciona (las descriptions están siempre en contexto). Existe para vencer la *racionalización* ("es simple, no necesito la skill"), que es un fenómeno real y documentado por upstream (la tabla entera de Red Flags de using-superpowers ataca eso). Lo que se pierde no es visibilidad, es presión de compliance en el momento cero.
- **"Core-index hace routing" es verdad solo para la instancia de Paul** (lo inyecta el SessionStart de reflex). El framework genérico, sin KB y sin guardrails (que llega post-métrica-D), queda SOLO con frontmatter. Para Paul hoy es aceptable; para el framework como producto es un hueco sin medir.

**No compres "nada" a ciegas ni el forcing agresivo por nostalgia. Compra una medición.** El propio diseño del cutover te da la ventana: las semanas con superpowers instalado-pero-apagado. Propuesta concreta: (a) añade al core-index 2-3 líneas de routing de proceso ("construir → brainstorm primero; bug → debug primero; declarar hecho → verify") — coste ~nulo, viaja por el canal que ya existe; (b) durante la ventana de rollback, registra cada "esta skill debió disparar y no lo hizo" (misma disciplina que la métrica de FP de reflex); (c) decide con datos si hace falta más. Es tu cultura de medición aplicada a tu propio framework — sería raro no hacerlo.

### Eje 2 — El colapso 17→6: SÓLIDO en orchestrate y debug, DÉBIL en verify

- **orchestrate (4→1): legítimo.** executing-plans es *explícitamente* el fallback sin-subagentes de subagent-driven-development (su propio texto lo dice: "If subagents are available, use subagent-driven-development instead"); no son dos skills, son dos ramas del mismo momento. dispatching-parallel-agents es la misma familia (delegar con contexto construido). orchestrate-personal ya es una capa SOBRE SDD, no una skill independiente. El riesgo de "skill gorda" se gestiona con el patrón que SDD ya usa: body corto + reference files (implementer-prompt.md, task-reviewer-prompt.md viajan como references hoy). Los matices operativos que el brief teme perder (checkpoints, patrones de paralelismo) caben en references sin engordar el body. OJO: orchestrate-personal son 135 líneas de doctrina densa ya destilada — el trabajo real de esta fusión es podar SDD (418 líneas) a references, no fundir cuatro textos.
- **debug (systematic-debugging + recon-first): legítimo, y la objeción del brief es MOOT.** El stuck-loop hook que "referencia recon-first" fue eliminado hace semanas (0 señal, commit 590d6ca) — no hay trigger que romper; de hecho la description de recon-first está stale y hay que arreglarla igual. Es el mismo movimiento (para, recoge información, verifica el supuesto más barato) con dos puertas de entrada. **Matiz que sí importa**: recon-first dispara en contextos que NO son bugs (terreno desconocido antes de grindear, tarea time-boxed) — si la skill se llama `debug` y su description solo habla de bugs, pierdes el caso preventivo. La description fusionada debe llevar ambas puertas ("bug/test failure/unexpected behavior" + "stuck ≥3 intentos o terreno desconocido antes de grindear").
- **verify (verification-before-completion + requesting-code-review + gate): la fusión más floja.** Son dos momentos con dos actores y dos costes: (1) auto-verificación con comandos antes de CUALQUIER claim de "hecho" — barata, cada vez, ley de hierro; (2) despachar un reviewer subagent — cara, por tarea/rama, decisión del orquestador. Fundirlas arriesga o spam de reviewers (si el fused manda review en cada claim) o dilución de la ley de hierro (si el review opcional ablanda el "evidence before claims"). **Contrapropuesta**: `verify` = evidencia-antes-de-claims + el validation gate del parent (lo que Paul más valora); el despacho de reviewer se va a `orchestrate`, donde YA vive — el caso mandatorio #1 de requesting-code-review es literalmente "after each task in subagent-driven development", y el two-stage review es parte del flujo SDD. El review standalone pre-merge queda como sección/reference de verify. Resultado: siguen siendo 6 skills, mejor cortadas.

### Eje 3 — Overlay KB-opcional: SÓLIDO, con dos acotaciones

No es YAGNI: el principio "genérico de base, instancia = KB + profile" está firmado, y el coste del overlay es una línea-patrón, no una feature. Y no es dependencia nueva: las skills de Paul YA hacen overlay hoy (recon-first y orchestrate-personal apuntan a [[doctrina-agentes]] con la regla "si divergen, manda la nota"; documenta usa kbx con fallback visible). Acotaciones:

1. **La degradación debe ser el patrón documenta, estandarizado**: probe barato del CLI (exit ≠ 0 / no ejecutable) → defaults + una línea visible en el output ("engine unavailable → defaults"). Nunca bloquear, nunca silencioso. Una sola forma para las 6 skills, no seis variantes.
2. **No construyas read-path del engine para esto antes de E1.** Mientras el engine no exista, "vía engine" significa "vía kbx/basic-memory como hoy". El overlay es un *contrato de interfaz* (la skill llama a UN comando que algún día será el engine), no una razón para adelantar código. Si la Sección 3 se lee como "las skills necesitan engine read-path el día del cutover", eso contradice el write-path-acotado firmado — aclarar la redacción.

### Eje 4 — Licencia: RESUELTO a favor del plan

MIT, verificada en el LICENSE del cache. "Destilar el movimiento" ni siquiera roza el límite — MIT permite copiar, modificar y **publicar** derivados, incluido copy-paste literal, con la única condición de retener el copyright notice si reusas porciones sustanciales. Pragmática concreta: pon desde el día 1 una línea de atribución en el monorepo ("process distills workflows from superpowers, (c) 2025 Jesse Vincent, MIT") — cuesta nada y deja la publicación futura cerrada. La cautela del plan ("verificar licencia antes de reusar texto") queda cumplida y puede relajarse: reescribir en formato thin propio sigue siendo lo correcto por CALIDAD (el formato de superpowers es verboso para tu doctrina), no por licencia.

### Eje 5 — Pérdidas silenciosas: el descarte es limpio, el cutover tiene 3 huecos

- **Las 4 skills descartadas: cero referencias en la KB** (verificado por grep). using-git-worktrees además está doblemente cubierto: fabrica trae sus propias instrucciones de worktree (`.worktrees/<item>`, con razón operativa propia: el guard necesita ver el flag) y el harness tiene aislamiento nativo. Descarte ratificado.
- **Hueco 1 — fabrica referencia lo que muere** (hecho verificado #6). El día del cutover hay que tocar TAMBIÉN paul-profile: reescribir la línea de fabrica a `process:orchestrate`, retirar orchestrate-personal (absorbida), y decidir si los paths `.superpowers/fabrica/` se quedan (funcionan, pero el nombre queda huérfano de significado). El plan de cutover solo menciona superpowers-off + process-on.
- **Hueco 2 — writing-skills desaparece justo cuando más se escribe skills.** Las 6 skills se construyen con superpowers activo (bien), pero el framework va a iterar skills durante meses. Mitigación que el plan casi ya tiene: el "formato de skill" de la Sección 3 (frontmatter + body 30-50 + references) se materializa como template/doc en `templates/` del monorepo. Con eso, writing-skills muere sin pérdida.
- **Hueco 3 — recon-first tiene la description stale** (stuck-loop muerto). Trivial, pero es exactamente el tipo de deriva skill↔realidad que el framework dice querer matar; arréglalo al fusionar en debug.

### Eje 6 — Estructura: SÓLIDO; el cutover necesita checklist, no rediseño

- **Dos plugins (process + guardrails) es correcto y está probado en casa**: hoy corres reflex + paul-profile como dos plugins del mismo marketplace sin fricción. La separación es limpia de verdad: process = skills puras sin hooks (versión avanza con la doctrina); guardrails = hooks + scripts + medición de FP (versión avanza con los reflejos). Fusionarlos acoplaría dos cadencias de release distintas para ahorrar un hooks.json. Dos.
- **Marketplace en monorepo: probado, no es una apuesta.** agent-develop YA es un marketplace GitHub con `pluginRoot: ./plugins`, registrado con autoUpdate (hecho #8). El flujo real es: editas en el repo dev → push → el clone de `~/.claude/plugins/marketplaces/` se actualiza. Funciona.
- **Pero el cutover "atómico" tiene más piezas de las que el plan lista**: (1) registrar el marketplace nuevo (o renombrar/reusar el repo agent-develop — decisión pendiente que el plan no nombra); (2) editar enabledPlugins en settings.json (superpowers:false, process:true, y las entradas `@agent-develop` → `@<monorepo>` cuando toque); (3) migrar la entrada de **workflow-lint** al marketplace.json nuevo o dejarla huérfana conscientemente; (4) actualizar fabrica/paul-profile (hueco 1); (5) fabrica sigue necesitando ser PLUGIN por su guard hook (hecho #7) — "skill de instancia fuera del framework" debe leerse como "plugin de instancia", no como skill suelta en `~/.claude/skills/`. Todo cabe en un día, pero escrito como checklist, no como una frase.

## Top 3 riesgos

1. **Disparo post-cutover sin medición** (eje 1): apagar el forcing de compliance confiando en frontmatter + un core-index que solo existe en la instancia, sin instrumentar la ventana de rollback, convierte el único cambio de conducta arriesgado del plan en una apuesta sin datos.
2. **Cutover con referencias colgando** (ejes 5/6): fabrica apuntando a skills muertas y workflow-lint/settings sin migrar no rompen ruidosamente — degradan en silencio, que es peor. Sin checklist escrita, algo de esto se escapa.
3. **verify mal cortada** (eje 2): fusionar ley-de-hierro barata con review caro o bien spamea reviewers o bien ablanda el "evidence before claims" — el gate que Paul más valora es lo que está en juego.

## Cambios concretos que propongo

1. Eje 1: 2-3 líneas de routing de proceso en core-index + contador de "skill no disparó" durante la ventana de rollback como criterio de la desinstalación definitiva. La frase "desaparece sin sustituto" se cambia por "desaparece con medición".
2. Eje 2: recortar `verify` a auto-verificación + validation gate; mover el despacho de reviewer a `orchestrate` (donde ya vive en SDD); review standalone pre-merge como reference de verify.
3. Eje 2: la description de `debug` lleva las dos puertas (bug/comportamiento inesperado + stuck/terreno desconocido pre-grind); de paso muere la referencia stale a stuck-loop.
4. Eje 3: una línea en la spec fijando el patrón de degradación único (probe CLI → defaults + aviso visible, estilo documenta) y aclarando que "vía engine" = "vía kbx/basic-memory hasta E1" (contrato de interfaz, no dependencia adelantada).
5. Eje 4: línea de atribución MIT en el monorepo desde el día 1.
6. Ejes 5/6: el párrafo de cutover se convierte en checklist: marketplace (¿repo nuevo o agent-develop renombrado?), settings.json, workflow-lint, actualización de fabrica + retirada de orchestrate-personal, template de formato-de-skill en templates/, fabrica-como-plugin explícito.

## Qué ratifico sin reservas

- La absorción selectiva con MIT verificada: sin riesgo legal, ni para publicar.
- El descarte de las 4 skills no migradas: cero referencias vivas, verificado.
- La fusión orchestrate (las 4 fuentes son un solo momento con ramas) y la fusión debug (la objeción del hook es moot — ya estaba muerto).
- Skill autocontenida + overlay opcional como patrón (ya probado en documenta y en las notas-puntero de tus skills).
- Dos plugins y marketplace-en-monorepo: es literalmente cómo ya funciona agent-develop hoy.
- El cutover atómico con rollback de semanas como *diseño* (solo le falta ser checklist).
