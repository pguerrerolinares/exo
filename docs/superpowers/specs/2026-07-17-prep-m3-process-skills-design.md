# Plugin `process` — design spec (prep-M3)

- **Fecha**: 2026-07-17
- **Estado**: borrador — pendiente de gate de consultor. El formato de skill es
  superficie irreversible interna nombrada en la spec del framework
  (`docs/superpowers/specs/2026-07-16-framework-unificado-design.md` §8:
  *"Todo review/gate que el protocolo derivaría a Paul (incluidos `GATE:` de
  merge y superficies irreversibles internas: envelope, schema del índice,
  formato de skill) lo adjudica un consultor Fable independiente"*), así que
  esta spec pasa por el régimen de gates de
  `.superpowers/fabrica/config.md` §Ejecución de gates antes de que exista
  implementación alguna.
- **Pieza**: prep-M3 = checklist de cutover §5.3 **paso 1 y solo paso 1**:
  *"Skills de process escritas y revisadas con superpowers aún activo (sin
  instalar process)"* (framework §5.3.1). Esta spec + los checklists gold de
  `evals/prep-m3/gold/` son la entrega spec-first; la implementación de las
  skills es una fase posterior, gateada por este documento y por su gold.
- **Regla de la cita**: toda decisión de diseño de este documento lleva cita
  textual de sus fuentes (framework spec, config de fábrica, o la skill fuente
  con path). Las notas de interpretación ya adjudicadas por el orquestador de
  la campaña (brief prep-m3-spec) se marcan como `[adjudicado]`.

## 1. Qué queda explícitamente FUERA

Nada de esta lista es acción de esta pieza ni de su implementación posterior;
todo está gateado por `GATE-CALENDARIO-D` (config §GATE-CALENDARIO-D:
*"Bloquea: … M3 cutover real (paso 2 del checklist §5.3 en adelante) … y
cualquier cambio que altere marketplace/skills/recall del agente"*):

- **Instalar el plugin `process`** o registrarlo en marketplace alguno.
- **Deshabilitar superpowers** (§5.3 paso 2: *"Mismo día: superpowers
  disabled + process enabled"* — cutover real, futuro).
- **Tocar fabrica** (§5.3 paso 2: *"actualizar fabrica"* es ítem del cutover,
  no de prep-M3) **o `/consolida`** (framework §5.2: *"`/consolida` se queda
  en el plugin reflex hasta M6"*).
- **Activar** la línea de routing en core-index o el contador de no-disparos
  (§6 de esta spec los DISEÑA; §5.3 pasos 2-3 los activan).
- §5.3 pasos 2-4 completos (probe post-cutover, ventana de rollback,
  desinstalación).

## 2. Layout del plugin

```
plugins/process/
├── README.md                      # qué es, tabla de atribución (§8)
├── LICENSES/superpowers.LICENSE   # MIT literal © 2025 Jesse Vincent (§8)
└── skills/
    ├── brainstorm/SKILL.md
    ├── plan/
    │   ├── SKILL.md
    │   └── plan-template.md
    ├── orchestrate/
    │   ├── SKILL.md
    │   ├── implementer-prompt.md
    │   ├── reviewer-prompt.md
    │   └── scripts/{task-brief,review-package,sdd-workspace}
    ├── tdd/
    │   ├── SKILL.md
    │   └── anti-patterns.md
    ├── debug/
    │   ├── SKILL.md
    │   └── techniques.md
    ├── verify/SKILL.md
    └── documenta/
        ├── SKILL.md
        └── routing.md
```

- Ubicación `plugins/`: framework §7-M1a fija la estructura del monorepo
  (*"estructura engine/plugins/templates"*). El layout interno sigue el patrón
  de agent-develop (config §Clases pre-autorizadas: *"siguiendo el patrón ya
  usado por kbx (cmd/, internal/) y por agent-develop (estructura de
  plugin)"*), pero como esta pieza toca el formato de skill, NO va por clase
  pre-autorizada sino por gate (misma cláusula: *"salvo que toque el envelope
  JSON o el formato de skill, que son superficies irreversibles"*).
- 7 skills: brainstorm, plan, orchestrate, tdd, debug, verify, documenta —
  exactamente las filas de la tabla framework §5.2 más `/documenta`
  (framework §5.2: *"`/documenta` entra a process"*). No hay octava skill.
- Reference files **dentro del directorio de cada skill**, nunca en la KB:
  framework §5.1: *"reference files empaquetados en el directorio de la skill
  (progressive disclosure; la carne va ahí, NO en la KB — si la doctrina
  genérica viviera en la KB, el framework quedaría vacío y las skills
  dependerían del engine en runtime)"*.

## 3. Formato de skill (el contrato que gatea el consultor)

Por framework §5.1: *"Frontmatter de disparo + body ~30-50 líneas + reference
files empaquetados en el directorio de la skill … Overlay personal con patrón
único de degradación: probe del CLI → si no hay engine/KB, defaults + aviso
visible (patrón ya probado en /documenta)"*.

1. **Frontmatter de disparo**: `name` + `description`. La description es el
   trigger: dice cuándo dispara, en 1-3 frases, sin imperativos a gritos. Las
   descriptions propuestas están en §5 por skill. Idioma: castellano con
   términos técnicos en inglés — el patrón ya operativo en las fuentes propias
   (`recon-first`, `/documenta`) y el registro de trabajo de Paul.
2. **Body ~30-50 líneas** `[adjudicado]`: el rango cuenta SOLO el body del
   SKILL.md destilado, sin frontmatter ni reference files. El body es router:
   los movimientos esenciales del checklist gold, enunciados como reglas de
   una-dos líneas, con punteros a los reference files para la carne
   (framework §3: *"Ningún procedimiento vive en una skill: la skill es
   router + reference files"*).
3. **Reference files**: en esta fase se ESPECIFICAN (nombre + contenido
   previsto, §5 por skill); NO se escriben `[adjudicado]` — son
   implementación posterior.
4. **Overlay personal — patrón único de degradación** `[adjudicado]`: UN
   MISMO patrón para las 7 skills, no uno por skill. Es el patrón ya probado
   en `/documenta` (`~/.claude/commands/documenta.md` §Paso 2: *"Fallback
   visible: si kbx no está o falla (exit ≠ 0 / no ejecutable), vuelve a
   search_notes como antes y añade al resumen final una línea: `kbx
   unavailable → search_notes fallback`. Nunca bloquees el cierre de sesión
   por esto"*). Generalizado:

   > Toda dependencia de instancia (engine, KB, perfil) se resuelve con:
   > **probe del CLI → si no hay engine/KB, defaults + aviso visible en el
   > output final → nunca bloquear la tarea.**

   - "Vía engine" significa hoy: *"kbx/basic-memory/filesystem hasta que E1
     exista"* (framework §5.1).
   - Skills con dependencia de instancia en v1: `documenta` (routing contra
     la KB) y `orchestrate` (memory packet). El resto no consultan engine/KB
     en v1: el patrón les aplica vacuamente y no llevan probe. El patrón es
     único; el número de probes no.
5. **Se tira al destilar** (framework §5.2: *"Absorber = extraer el movimiento
   esencial y reescribirlo thin; se tira la prosa, los gritos y los gates
   dogmáticos"*): digraphs dot, ejemplos-transcript completos, tablas de
   racionalizaciones como diálogo, iron laws en mayúsculas, secciones
   "Real-World Impact"/"Why This Matters", y los "Announce at start". El
   detalle por skill vive en la sección DESCARTES de su gold.

## 4. Fuente única del mapeo skill→fuentes

La tabla framework §5.2 es la fuente única; esta spec no añade mapeos:

| Skill | Absorbe de superpowers 6.1.1 (MIT) | Absorbe propio |
|---|---|---|
| brainstorm | brainstorming | — |
| plan | writing-plans | — |
| orchestrate | subagent-driven-development, executing-plans, dispatching-parallel-agents | orchestrate-personal (cost pyramid, memory packet, blindspot pass) + reviewer-dispatch escalado al riesgo del diff |
| tdd | test-driven-development | — |
| debug | systematic-debugging | recon-first (dos puertas: bug + stuck/pre-grind) |
| verify | verification-before-completion | gate de validación del padre de orchestrate-personal |
| documenta | — | `~/.claude/commands/documenta.md` |

Versiones fuente leídas para esta spec (las instaladas hoy):
- superpowers **6.1.1**: `~/.claude/plugins/cache/claude-plugins-official/superpowers/6.1.1/skills/`
- paul-profile **0.5.0**: `~/.claude/plugins/cache/agent-develop/paul-profile/0.5.0/skills/orchestrate-personal/SKILL.md`
- reflex **0.9.0**: `~/.claude/plugins/cache/agent-develop/reflex/0.9.0/skills/recon-first/SKILL.md`
- `/documenta`: `~/.claude/commands/documenta.md`

El checklist de movimientos por skill (el oráculo de la implementación) vive
en `evals/prep-m3/gold/<skill>.md` — config §Oráculos: *"Skills de process
(prep-M3): sin oráculo mecánico — el 'oráculo' es el checklist de paridad de
movimientos vs la skill superpowers absorbida (spec §5.2, tabla), verificado
por el consultor-gate"*.

## 5. Diseño por skill

### 5.1 brainstorm

- **Fuente**: `skills/brainstorming/SKILL.md` (159 líneas).
- **Description propuesta**: «Usa antes de cualquier trabajo creativo — nueva
  feature, componente, funcionalidad o cambio de comportamiento. Explora
  intención, requisitos y diseño en diálogo antes de implementar; termina en
  una spec escrita, auto-revisada y aprobada por el usuario.»
- **Body**: los ~14 movimientos del gold (explorar contexto, una pregunta a la
  vez, detección de scope multi-subsistema, 2-3 enfoques con trade-offs,
  diseño por secciones con validación incremental, diseño-antes-de-código,
  anti-patrón "too simple", spec a `docs/superpowers/specs/`, self-review,
  gate de review del usuario, handoff a `plan`, YAGNI).
- **Reference files**: ninguno previsto. La fuente cabe destilada en el body;
  si en implementación no cabe, aplica el kill-criterio §9.1 antes de
  recortar movimientos.
- **Se tira**: el visual companion completo (SKILL.md §Visual Companion +
  `visual-companion.md` + `scripts/` — servidor node, HTML, browser): es
  maquinaria opcional de presentación, no un movimiento de diseño; §5.2 manda
  *"extraer el movimiento esencial"*. `spec-document-reviewer-prompt.md`: el
  body 6.1.1 ya no lo referencia (el self-review es inline: *"This is a
  checklist you run yourself — not a subagent dispatch"*). Detalle en gold
  §DESCARTES.

### 5.2 plan

- **Fuente**: `skills/writing-plans/SKILL.md` (174 líneas).
- **Description propuesta**: «Usa cuando tienes una spec o requisitos para una
  tarea multi-paso, antes de tocar código. Produce un plan de tareas
  bite-sized con paths, código y comandos exactos, pensado para un ejecutor
  sin contexto.»
- **Body**: premisa del lector sin contexto, scope check, file structure antes
  de tareas, task right-sizing, pasos 2-5 min, regla no-placeholders,
  self-review (cobertura/placeholders/consistencia de tipos), handoff a
  `process:orchestrate`.
- **Reference files**: `plan-template.md` — header obligatorio del plan
  (goal/architecture/tech stack/global constraints verbatim) + estructura de
  tarea completa (Files/Interfaces/steps con código) + la lista literal de
  no-placeholders. Es la carne de formato que no cabe en 30-50 líneas.
- **Delta de diseño**: el handoff de la fuente ofrece dos modos
  (subagent-driven vs executing-plans). En process ambos viven en
  `orchestrate` (tabla §5.2), así que el handoff es único: invocar
  `process:orchestrate`. La referencia de la fuente a
  `superpowers:using-git-worktrees` no se migra (framework §5.2: *"No se
  migra lo no usado (finishing-a-development-branch, writing-skills,
  using-git-worktrees, …): 0 referencias vivas en la KB (verificado)"*).

### 5.3 orchestrate

- **Fuentes**: `skills/subagent-driven-development/SKILL.md` (418 líneas, la
  columna vertebral), `skills/executing-plans/SKILL.md` (70),
  `skills/dispatching-parallel-agents/SKILL.md` (185), y
  `orchestrate-personal/SKILL.md` (135) — de esta última, por framework §5.2:
  *"cost pyramid, memory packet, blindspot pass + reviewer-dispatch escalado
  al riesgo del diff"*.
- **Description propuesta**: «Ejecuta planes de implementación multi-tarea:
  orquestador padre que despacha un ejecutor fresco por tarea, review en dos
  etapas (spec + calidad) por tarea y review final whole-branch, con ledger
  durable. Usa para features, refactors o backlogs de tareas independientes.»
- **Body**: división del trabajo padre/hijos, ejecución continua, pre-flight
  (plan review + recon de refs), cost pyramid con reviewer escalado al riesgo
  del diff, dispatch `reflex:executor` (ver §7 — requisito de paridad
  crítica), file handoffs, ledger durable, manejo de estados del
  implementador, paralelización solo de dominios independientes, review de
  dos verdictos + review final, fix dispatches, red lines (nunca en
  main/master sin consentimiento, nunca implementadores paralelos sobre el
  mismo estado, nunca re-despachar tarea completa).
- **Reference files**:
  - `implementer-prompt.md` — destilado del template de SDD (contrato de
    preguntas-antes-de-empezar, escalación BLOCKED/NEEDS_CONTEXT,
    self-review, report file + respuesta corta), con el dispatch default
    `subagent_type: reflex:executor` sin `model`.
  - `reviewer-prompt.md` — destilado de `task-reviewer-prompt.md` (dos
    verdictos, "do not trust the report", calibración
    Critical/Important/Minor, ⚠️ items, output format) + la guía de escalado
    del modelo del reviewer al riesgo del diff (orchestrate-personal). Sirve
    también para el review final whole-branch: la fuente de SDD delega ese
    template en `../requesting-code-review/code-reviewer.md`, y
    requesting-code-review NO se migra como skill (no está en la tabla §5.2);
    el movimiento "broad final review" sí es de SDD, y §5.1 obliga a
    empaquetar su carne en el directorio de la skill.
  - `scripts/task-brief`, `scripts/review-package`, `scripts/sdd-workspace` —
    casi tal cual (MIT; son el mecanismo de file handoffs), con header de
    origen. Nota: escriben bajo `.superpowers/sdd/` del working tree; el
    `.gitignore` de exo ya ignora `.superpowers/` (comportamiento deseado
    para scratch, cf. ALERTA del config sobre la excepción de
    `fabrica/config.md`).
- **Delta de diseño**: `executing-plans` no sobrevive como modo separado — su
  contenido no-duplicado (revisión crítica del plan antes de ejecutar; parar
  ante blocker en vez de adivinar) se funde en el body; la bifurcación
  "misma sesión vs sesión paralela" muere con la fusión que la tabla §5.2
  decide. `dispatching-parallel-agents` entra como sección de paralelización
  (cuándo sí/cuándo no + estructura de prompt), no como skill aparte.

### 5.4 tdd

- **Fuente**: `skills/test-driven-development/SKILL.md` (371 líneas) +
  `testing-anti-patterns.md`.
- **Description propuesta**: «Usa al implementar cualquier feature o bugfix,
  antes de escribir código de producción. Test primero, verlo fallar por la
  razón esperada, código mínimo, verde, refactor.»
- **Body**: ciclo red-green-refactor con los dos verifies obligatorios,
  regla "código sin test previo se borra y reescribe", GREEN mínimo sin
  extras, propiedades de buen test, excepciones legítimas (con permiso:
  prototipo desechable, código generado, config), bug = test que lo
  reproduce primero, checklist pre-completado, tabla when-stuck.
- **Reference files**: `anti-patterns.md` — destilado de
  `testing-anti-patterns.md`: no testear comportamiento de mocks, no añadir
  métodos test-only a producción, no mockear sin entender la dependencia.
- **Se tira**: los ensayos anti-racionalización (§Why Order Matters, ~50
  líneas de diálogo) y las tablas de excusas — se destilan a 1-2 reglas; los
  gritos ("Iron Law", "Violating the letter…"). Detalle en gold §DESCARTES.

### 5.5 debug

- **Fuentes**: `skills/systematic-debugging/SKILL.md` (296 líneas) + sus
  técnicas (`root-cause-tracing.md`, `defense-in-depth.md`,
  `condition-based-waiting.md`) y `recon-first/SKILL.md` (57 líneas).
- **Description propuesta** (dos puertas, framework §5.2: *"recon-first
  (description con dos puertas: bug + stuck/pre-grind…)"*): «Dos puertas:
  (1) ante cualquier bug, test que falla o comportamiento inesperado, antes
  de proponer fixes; (2) cuando estás atascado — mismo error ≥3 veces,
  terreno desconocido, time-box quemándose — o antes de grindear en solitario
  algo no familiar. Root cause y recon antes de computar.»
  - **Corrección obligatoria**: la description NO menciona el hook
    `stuck-loop`. Framework §5.2: *"el hook stuck-loop está muerto — commit
    590d6ca — la description actual de recon-first está stale"*. La
    description fuente dice *"Triggered by the reflex `stuck-loop` hook"* —
    eso se elimina, no se hereda.
- **Body**: las 4 fases (root cause → patrón → hipótesis única/test mínimo →
  fix con failing test, enlazando a `tdd`), evidencia por boundary en
  sistemas multi-componente, regla 3-fixes-fallidos ⇒ cuestionar arquitectura
  y discutir con el humano, y la puerta recon: parar de reintentar, buscar el
  error literal / docs oficiales (retrieve > compute), listar supuestos y
  verificar el más barato, reducir el caso, delegar investigación voluminosa.
- **Reference files**: `techniques.md` — un fichero, tres secciones
  destiladas: root-cause-tracing (trazar hacia atrás hasta el trigger
  original), defense-in-depth (validar en cada capa tras hallar la causa),
  condition-based-waiting (esperar la condición, no un sleep arbitrario).
- **Delta de diseño**: la referencia cruzada de recon-first a
  `superpowers:systematic-debugging` desaparece — ambas fuentes son ahora la
  misma skill.

### 5.6 verify

- **Fuentes**: `skills/verification-before-completion/SKILL.md` (139 líneas)
  + la sección "Parent validation gate" de `orchestrate-personal/SKILL.md`
  (líneas 82-99).
- **Description propuesta**: «Usa antes de declarar trabajo completo,
  arreglado o pasando — antes de commitear, de crear un PR o de aceptar el
  trabajo de un subagente. Evidencia fresca del comando antes de cualquier
  claim.»
- **Body**: la gate function (identificar comando → correr completo → leer
  output → recién entonces afirmar), tabla claim→evidencia, red flags
  ("should/probably", satisfacción pre-verificación, confiar el report de un
  agente), red-green verificado para tests de regresión, checklist
  línea-a-línea contra requirements, y el gate del padre: tests/linter/
  type-checker/security, verificación real donde aplique (UI: driving
  desktop+mobile, "un build pasando NO es prueba visual"; backend:
  endpoint/DB real), calidad de ingeniería como gate de release
  (reuse-first, tooling del lockfile, deps explícitas, DRY), escrutinio del
  diff antes del commit atómico.
- **Límite de diseño (de la tabla §5.2, no negociable en implementación)**:
  *"Solo auto-verificación barata pre-commit; el reviewer-dispatch (caro,
  pre-merge) vive en orchestrate — mezclarlos = spam de reviews o dilución
  del gate"*. `verify` NO despacha reviewers ni subagentes: todo lo que
  requiere otro agente pertenece a `orchestrate`.
- **Reference files**: ninguno previsto (mismo criterio que brainstorm).

### 5.7 documenta

- **Fuente**: `~/.claude/commands/documenta.md` (comando; framework §5.2:
  *"`/documenta` entra a process"*).
- **Description propuesta**: «Extrae decisiones, opiniones, aprendizajes y
  patrones de la sesión actual y los guarda en la KB siguiendo el contrato de
  routing: canon como delta, bitácora como append, nota nueva casi nunca.
  Commit scoped al cerrar.»
- **Body**: extraer (qué persiste / qué se descarta), orientación barata con
  `kbx targets` + fallback visible (el seed del patrón §3.4), regla de oro
  del routing, search-before-write / editar-no-duplicar / preferir append,
  commit scoped (nunca `git add -A`, nunca `cd`, nunca push, retry ante
  index.lock), resumen final.
- **Reference files**: `routing.md` — el contrato de routing v2 destilado
  (tabla destino por tipo de pieza, reglas de frontmatter tags+tier, caveat
  de concurrencia). Los NOMBRES concretos de notas de la instancia
  ([[Backlog — frentes abiertos]], [[Paul - perfil de trabajo]]) no se
  hardcodean en el genérico: se resuelven vía probe contra la KB (framework
  §3: *"Framework sin nada personal"* + *"el porqué y los deltas personales
  viven en la KB"*); el reference file lleva el contrato con placeholders.
- **Delta de diseño obligatorio (framework §6.3)**: la skill destilada deja
  de emitir observations/relations como estructura — §6.3: *"Observations
  pasan a bullets normales (texto indexable, sin fila propia). /documenta
  deja de generarlas como estructura"*. Los wikilinks `[[...]]` se mantienen
  (§6.3: *"Wikilinks `[[...]]` = contrato load-bearing"*). Las líneas de la
  fuente que mandan formato `- [categoria] contenido` y `- tipo_relacion
  [[Titulo]]` van a DESCARTES del gold con esta cita.

## 6. Sustituto de using-superpowers (diseño; activación = M3, fuera de scope)

Framework §5.2: *"`using-superpowers` (SessionStart) desaparece. Su función
real es compliance anti-racionalización, no descubrimiento. Sustituto mínimo:
línea de routing en core-index + contador de 'skill que debió disparar y no
disparó' durante la ventana de rollback, como criterio pre-registrado de
desinstalación."* Aquí se diseñan ambas piezas; **escribir la línea en
core-index y empezar a contar son actos del cutover (§5.3 pasos 2-3) y NO se
ejecutan en esta pieza** (GATE-CALENDARIO-D).

1. **Línea de routing en core-index** (nota core-index de kb-demo; texto
   propuesto, una sola línea):
   > Proceso de trabajo → plugin `process`: brainstorm (diseño antes de
   > código) · plan (spec→plan) · orchestrate (ejecutar plan multi-tarea) ·
   > tdd (test primero) · debug (bug o atasco) · verify (antes de declarar
   > hecho) · documenta (cierre de sesión). Si una debió disparar y no
   > disparó, apúntalo en `exo/evals/prep-m3/no-disparos.md`.
2. **Contador de no-disparos**: fichero versionado
   `evals/prep-m3/no-disparos.md` en exo, append-only, una línea por evento:
   `- <fecha ISO> | <skill que debió disparar> | <contexto en una línea>`.
   Lo appendea quien detecte el no-disparo (el propio agente al notarlo, o
   Paul). Sin hook: un hook nuevo alteraría el entorno del agente y eso está
   D-gateado (config: *"cualquier cambio que altere marketplace/skills/recall
   del agente"*).
3. **Criterio pre-registrado de desinstalación** (se sella aquí, se evalúa en
   §5.3.4): superpowers se desinstala cuando un ciclo real de trabajo
   completo cierra con **0 entradas nuevas** en el contador (§5.3.3:
   *"superpowers instalado-pero-apagado ≥ un ciclo real de trabajo (rollback
   de un flag) + contador de no-disparos activo"*; §5.3.4: *"Desinstalar
   cuando el ciclo cierre sin carencias"*). ≥1 entrada ⇒ arreglar la
   description de la skill afectada y reiniciar el ciclo, no desinstalar.
   Esto responde al riesgo §9.7 del framework (*"Quitar el forcing de
   superpowers degrada disparo de skills → Contador de no-disparos
   pre-registrado §5.2"*).

**No se migran** (framework §5.2: *"No se migra lo no usado
(finishing-a-development-branch, writing-skills, using-git-worktrees, …): 0
referencias vivas en la KB (verificado). Se añade si duele."*):
finishing-a-development-branch, writing-skills, using-git-worktrees,
using-superpowers (sustituido, arriba), requesting-code-review y
receiving-code-review como skills (el único movimiento que process necesita
de ese vecindario — el template del review final — queda absorbido en
`orchestrate/reviewer-prompt.md`, §5.3), y el resto del catálogo superpowers
no listado en la tabla §5.2.

## 7. Paridad crítica: dispatch `reflex:executor` sin `model`

Requisito de diseño de `process:orchestrate`, sellado aquí Y como ítem del
gold (`evals/prep-m3/gold/orchestrate.md`, ítem PARIDAD-CRÍTICA):

> `process:orchestrate` conserva el dispatch `subagent_type: reflex:executor`
> **sin** parámetro `model`.

- Framework §5.3.2: *"`process:orchestrate` conserva el dispatch
  `subagent_type: reflex:executor` sin `model` (paridad paul-profile 0.3.0 —
  si se pierde, reflex v2 se desenchufa sin síntoma)"*.
- Fuente del movimiento: orchestrate-personal SKILL.md: *"Despacha los
  ejecutores como `subagent_type: reflex:executor`, nunca `general-purpose`"*
  y *"Salvo roles con `model` fijo en su definición — p.ej. `reflex:executor`
  — donde NO debes pasar `model` en el dispatch (lo pisarías)"*.
- Consecuencia sobre los reference files: `implementer-prompt.md` destilado
  usa `reflex:executor` como dispatch default y NO hereda del template de SDD
  la línea `model: [MODEL — REQUIRED…]` para ese rol (pasarlo pisaría el
  modelo del rol). La selección explícita de modelo sigue aplicando a los
  demás dispatches (reviewers, research) per cost pyramid.
- El probe post-cutover que verifica `agent_type` en reflex-log es del paso 2
  de §5.3: futuro, fuera de esta pieza.
- Riesgo que esto mitiga: framework §9.5 (*"Perder el dispatch reflex:executor
  en el cutover → reflex v2 desenchufado sin síntoma"*).

## 8. Atribución MIT (día 1)

Framework §2.3: *"superpowers se jubila por absorción selectiva (licencia MIT
verificada, © 2025 Jesse Vincent; atribución en el repo desde el día 1)"* y
§5.2: *"Atribución MIT en el repo día 1"*. Cómo queda en el repo:

- `plugins/process/LICENSES/superpowers.LICENSE`: copia literal del LICENSE
  de superpowers 6.1.1 (*"MIT License / Copyright (c) 2025 Jesse Vincent"*,
  verificado en `~/.claude/plugins/cache/claude-plugins-official/superpowers/6.1.1/LICENSE`).
- `plugins/process/README.md`, sección "Atribución": la tabla §4 de esta spec
  (skill destilada → skill(s) fuente superpowers 6.1.1 + upstream
  `obra/superpowers` de Jesse Vincent), declarando que brainstorm, plan,
  orchestrate, tdd, debug y verify son obras derivadas por destilación.
- Los scripts que se llevan casi tal cual (`task-brief`, `review-package`,
  `sdd-workspace`) conservan su header y añaden una línea de origen
  (`# Derived from superpowers 6.1.1 (MIT © 2025 Jesse Vincent)`).
- Ambos ficheros (LICENSES + README con la tabla) entran **en el primer
  commit de implementación del plugin** — eso es "día 1" del código derivado;
  esta spec no copia aún código.

## 9. Kill-criteria PRE-REGISTRADOS de la implementación

Pre-registro para la fase de implementación futura (config §Roadmap ítem 2 la
gatea; framework §8: *"Kill-criteria pre-registrados por pieza en su spec
(patrón ya practicado)"*). En esta fase solo se documentan; no hay retries
que ejecutar.

1. **Overflow del body**: si una skill no cabe en ~30-50 líneas de body sin
   perder movimientos de su checklist gold ⇒ mover carne a reference files
   (progressive disclosure, framework §5.1). Si aun así pierde movimientos ⇒
   **escalar al consultor-gate, no recortar el checklist**.
2. **Cap de retries por eval: 2** (config §Presupuesto: *"cap retries por
   eval: 2 (default spec §8)"*). Si tras 2 retries la skill implementada no
   pasa su checklist gold ⇒ parar y escalar, no tercer retry.
3. **Cero movimientos nuevos sin cita**: la implementación no añade
   movimientos que no estén en el gold; añadir uno exige cita de fuente +
   pasar por el consultor-gate (regla de la cita, config §Fuentes de criterio
   escrito). Un movimiento nuevo silencioso = fallo de paridad aunque el
   resto del checklist esté al 100%.
4. **Paridad crítica intocable**: el ítem §7 (reflex:executor sin model) no
   admite retry-negociación — si un retry lo pierde, el retry es inválido.

## 10. Criterio de cierre de la implementación futura

La skill implementada se acepta cuando un revisor (consultor-gate, config
§Ejecución de gates) marca su checklist gold con **paridad 100% de
movimientos, 0 movimientos nuevos sin cita** — verificación
presente/ausente, sin juicio estético. Procedimiento en
`evals/prep-m3/README.md`.
