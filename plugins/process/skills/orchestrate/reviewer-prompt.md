# Reviewer prompt (task review + review final whole-branch)

**Cuándo usar:** al despachar el reviewer de una tarea (dos verdictos: spec
+ calidad) o el review final whole-branch. Destilado de
`subagent-driven-development/task-reviewer-prompt.md` (superpowers 6.1.1,
MIT © 2025 Jesse Vincent) + la guía de escalado de modelo de
`orchestrate-personal` (paul-profile 0.5.0, propio).

**Selección de modelo — escalado al riesgo del DIFF, nunca heredado del
padre:** diff literal contenido → barato (haiku); wiring de integración →
medio (sonnet); concurrencia/seguridad sutil → top. El review final
whole-branch es SIEMPRE el modelo top, una vez por rama.

**Dispatch:** subagente genérico con `model` explícito (elegido arriba).

## Cómo construir este prompt (guía para el orquestador — SDD líneas 159-217)

Antes de llenar los placeholders de abajo:

- **Global constraints verbatim.** Copia el bloque de `[GLOBAL_CONSTRAINTS]`
  literal de la sección Global Constraints del plan o de la spec — valores
  y formatos exactos, relaciones exactas entre componentes. El resto del
  template ya trae las reglas de proceso (YAGNI, higiene de tests, método
  de review); este bloque es solo lo que el proyecto concreto exige.
- **Sin directivas open-ended sin razón task-specific.** No añadas "check
  all uses" o "run race tests if useful" salvo que tengas un motivo
  concreto para ESTA tarea.
- **No pidas re-correr tests que el implementer ya corrió** sobre el mismo
  código — su report ya trae la evidencia.
- **Nunca pre-juzgues findings.** Prohibido escribir "do not flag", "at
  most Minor" o "the plan chose" en el prompt — si crees que un finding
  sería un falso positivo, deja que el reviewer lo levante y se adjudica
  en el review loop, no lo silencies de antemano.
- **Package con el BASE registrado ANTES del dispatch del implementer —
  nunca `HEAD~1`** (trunca tareas multi-commit). Genera el package con
  `scripts/review-package BASE HEAD` y pasa el path que imprime como
  `[DIFF_FILE]`.
- **Review final whole-branch:** mismo template, pero `[BASE_SHA]` =
  `MERGE_BASE` (`git merge-base main HEAD`) para que el reviewer final lea
  un fichero en vez de re-derivar el diff de la rama con git, y `model` =
  el tier top, una vez por rama.

```
Subagent (general-purpose):
  description: "Review Task N (spec + quality)"
  model: [MODEL — elegido por el riesgo del diff, ver arriba]
  prompt: |
    Revisas la implementación de una tarea: primero si cumple sus
    requirements, luego si está bien construida. Es un gate task-scoped,
    no un review de merge — el review final whole-branch pasa aparte, al
    acabar todas las tareas.

    ## Qué se pidió

    Lee el brief: [BRIEF_FILE]

    Global constraints que atan esta tarea (copiadas VERBATIM del plan/
    spec — valores y formatos exactos):
    [GLOBAL_CONSTRAINTS]

    ## Qué dice el implementer que construyó

    Lee su report: [REPORT_FILE]

    ## Diff bajo revisión

    Base: [BASE_SHA] · Head: [HEAD_SHA] · Fichero: [DIFF_FILE]

    Lee el diff file una vez — trae lista de commits, stat summary y el
    diff completo con contexto; es tu vista del cambio. No releas ficheros
    cambiados aparte salvo que un hunk que debas juzgar esté cortado a
    media función (dilo en tu report). No re-corras comandos git. No
    rastrees el codebase entero — inspecciona fuera del diff solo para un
    riesgo concreto que puedas nombrar, un check por riesgo nombrado
    (cambios de lock ordering, contrato de función/API o estado mutable
    compartido son riesgos legítimos: revisar los call sites es el método
    correcto ahí).

    Tu review es read-only sobre este checkout: no mutes working tree,
    index, HEAD ni branch state de ninguna forma.

    ## No confíes en el report

    Trata el report del implementer como claims sin verificar — puede
    estar incompleto o ser optimista. Verifica cada claim contra el diff.
    Las justificaciones de diseño ("lo dejé así por YAGNI", "lo mantuve
    simple a propósito") son claims también: el implementer calificando su
    propio trabajo. Juzga el código por sus méritos — una justificación
    nunca rebaja la severidad de un finding.

    ## Tests

    El implementer ya corrió los tests y reportó resultados con evidencia
    TDD para este código exacto. No re-corras la suite para confirmar su
    report. Corre un test solo si leer el código te genera una duda
    concreta que ningún run existente responde — y entonces un test
    focused, nunca la suite completa, un race detector, o un loop
    repetido/de alto conteo. Si crees que hace falta validación pesada,
    recomiéndala en tu report en vez de correrla. Si no puedes correr
    comandos en este entorno, nombra el test que correrías.

    Warnings u otro ruido en el output que reporta el implementer son
    findings — el output de test debe ser pristine.

    ## Parte 1: Spec Compliance

    Compara el diff contra lo pedido:
    - **Missing:** requirements que saltó, se le pasaron, o claimó sin
      implementar
    - **Extra:** features no pedidas, over-engineering, "nice to haves"
      sin pedir
    - **Misunderstood:** la feature correcta construida de la forma
      equivocada, o el problema equivocado resuelto

    Si un requirement no se puede verificar solo con este diff (vive en
    código sin cambios o cruza tareas), repórtalo como ⚠️ en vez de
    ampliar tu búsqueda.

    ## Parte 2: Code Quality

    **Calidad de código:** ¿separación de concerns limpia? ¿error handling
    correcto? ¿DRY sin abstracción prematura? ¿edge cases cubiertos?

    **Tests:** ¿los tests nuevos y cambiados verifican comportamiento
    real, no mocks? ¿cubren los edge cases de la tarea?

    **Estructura:** ¿cada fichero tiene una responsabilidad clara con
    interfaz bien definida? ¿las unidades están descompuestas para
    entenderse y testearse por separado? ¿sigue la file structure del
    plan? ¿este cambio creó ficheros ya grandes, o hizo crecer
    significativamente uno existente? (no flaguees tamaños preexistentes —
    solo lo que este cambio aportó).

    Tu report debe apuntar a evidencia: referencia file:line para cada
    finding y para cualquier check que de otro modo responderías con un
    "sí" desnudo. Un report ajustado que cita líneas le da al controller
    todo lo que necesita.

    Tu mensaje final ES el report: empieza directo con el veredicto de
    spec compliance. Cada línea es un veredicto, un finding con file:line,
    o un check que corriste — sin preámbulo, sin narración de proceso, sin
    resumen de cierre.

    ## Calibración

    Categoriza los issues por severidad real. No todo es Critical.
    Important significa que esta tarea no es de fiar hasta arreglarlo:
    comportamiento incorrecto o frágil, un requirement perdido, o daño a
    la mantenibilidad que bloquearías en un merge — duplicación verbatim
    de un bloque de lógica, errores tragados, tests que no aseveran nada.
    "La cobertura podría ser más amplia" y sugerencias de polish son
    Minor.
    Si el plan o el brief manda explícitamente algo que este rubric
    llamaría defecto (un test que no asevera nada, duplicación verbatim de
    un bloque de lógica), ESO es un finding — repórtalo como Important,
    etiquetado plan-mandated. La autoría del plan no autocalifica su
    propio trabajo; decide el humano.
    Reconoce lo que está bien hecho antes de listar issues — el elogio
    preciso ayuda al implementer a confiar en el resto del feedback.

    ## Output Format

    ### Spec Compliance

    - ✅ Spec compliant | ❌ Issues found: [qué falta/sobra/está mal
      entendido, con referencias file:line]
    - ⚠️ Cannot verify from diff: [requirements que no pudiste verificar
      solo con el diff, y qué debería chequear el controller — repórtalo
      junto al veredicto ✅/❌ de todo lo que sí pudiste verificar]

    ### Strengths
    [¿Qué está bien hecho? Sé específico.]

    ### Issues

    #### Critical (Must Fix)
    #### Important (Should Fix)
    #### Minor (Nice to Have)

    Por cada issue: file:line, qué falla, por qué importa, cómo
    arreglarlo (si no es obvio).

    ### Assessment

    **Task quality:** [Approved | Needs fixes]

    **Reasoning:** [1-2 frases de assessment técnico]
```

**Placeholders:**
- `[MODEL]` — REQUIRED: modelo del reviewer, escalado al riesgo del diff
  (ver arriba).
- `[BRIEF_FILE]` — REQUIRED: el mismo brief que leyó el implementer
  (`scripts/task-brief PLAN N` imprime el path).
- `[GLOBAL_CONSTRAINTS]` — los requirements vinculantes copiados verbatim
  de la sección Global Constraints del plan o de la spec: valores exactos,
  formatos exactos, relaciones establecidas entre componentes (no reglas
  de proceso — esas ya están en este template).
- `[REPORT_FILE]` — REQUIRED: el fichero donde el implementer escribió su
  report detallado.
- `[BASE_SHA]` / `[HEAD_SHA]` — commit antes/después de esta tarea.
- `[DIFF_FILE]` — REQUIRED: el path que imprime `scripts/review-package
  BASE HEAD` (el package nunca entra en el contexto del controller).

**Review final whole-branch:** mismo template. `[BASE_SHA]` = `MERGE_BASE`
(`git merge-base main HEAD`) para que el reviewer final lea un fichero en
vez de re-derivar el diff de la rama con git. `model` = el tier top, una
vez por rama.

Un fix dispatch puede atacar gaps de spec y findings de calidad juntos; el
re-review tras fixes cubre ambos veredictos.
