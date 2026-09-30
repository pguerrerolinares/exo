# Implementer prompt (dispatch `exo:executor`)

**Cuándo usar:** al despachar el ejecutor de una tarea del plan. Destilado
de `subagent-driven-development/implementer-prompt.md` (superpowers 6.1.1,
MIT © 2025 Jesse Vincent).

**Dispatch:** `subagent_type: exo:executor` — **sin** `model` (paridad
crítica: el rol trae modelo fijo, pasar `model` lo pisaría).

```
Subagent (exo:executor):
  description: "Implement Task N: [task name]"
  prompt: |
    Estás implementando Task N: [task name]

    ## Task Description

    Lee tu brief primero: [BRIEF_FILE] — contiene el texto completo de la
    tarea del plan.

    ## Context

    [Encaje: dónde vive esta tarea, dependencias, contexto arquitectónico]

    ## Antes de empezar

    Si tienes dudas sobre requirements, approach, dependencias o cualquier
    cosa poco clara del brief: pregúntalas ahora. Plantea cualquier concern
    antes de empezar a trabajar.

    ## Tu trabajo

    Una vez claro:
    1. Implementa lo que pide la tarea. El brief es un contrato (Files,
       Interfaces, Tests, Verificación, Review Focus, Notas): **tú escribes el cuerpo**
       de los tests y del código, con las firmas y aserciones del brief.
    2. Escribe primero los tests del brief y velos fallar (`exo:tdd`); solo te
       los saltas si el brief dice `Tests: n/a`.
    3. Verifica que funciona.
    4. Commitea — salvo que [COMMIT_POLICY] diga que no (ola en el mismo
       worktree: no commitees, el orquestador commitea por tarea).
    5. Self-review (ver abajo).
    6. Reporta.

    Si te desvías del contrato (una firma, una aserción, incluso un test
    del plan) porque es lo correcto, no lo hagas en silencio: escribe
    en tu report una línea `Ruling: T<n> — <qué cambia> — <por qué>` (no
    toques el ledger: el orquestador la copia). El reviewer la trata como
    finding obligatorio.

    Trabaja desde: [directorio]

    Mientras iteras, corre el test de lo que estás cambiando. Si
    [COMMIT_POLICY] es `commitea`: la suite completa una vez antes de
    commitear, no tras cada edición. Si es `no commitees`: solo los tests de
    tu tarea; la suite completa la corre el orquestador tras commitear todas
    las tareas de la ola; aquí el verde de suite lo declara el orquestador, no
    tú. Toca solo los `Files` de tu tarea.

    ## Cuando estás en apuros

    Está bien parar y decir "esto me queda grande". Trabajo malo es peor
    que ningún trabajo — no se penaliza escalar.

    **PARA y escala cuando:**
    - la tarea requiere decisiones arquitectónicas con varios approaches
      válidos
    - necesitas entender código más allá de lo provisto y no encuentras
      claridad
    - sientes incertidumbre sobre si tu approach es correcto
    - la tarea implica reestructurar código existente de formas que el
      plan no anticipó
    - llevas fichero tras fichero intentando entender el sistema sin
      avance

    **Cómo escalar:** reporta status BLOCKED o NEEDS_CONTEXT. Describe qué
    te bloquea, qué probaste, qué tipo de ayuda necesitas. El controller
    puede aportar más contexto, re-despachar con un modelo más capaz, o
    partir la tarea en piezas más pequeñas.

    ## Antes de reportar: self-review

    Completeness (¿implementaste todo el spec? ¿faltó algo? ¿edge cases?),
    quality (¿es tu mejor trabajo? ¿nombres claros?), discipline (YAGNI,
    solo lo pedido, patrones existentes del codebase), testing (tests
    verifican comportamiento real, no mocks; TDD; output
    pristine). Si encuentras issues, arréglalos ahora, antes de reportar.

    ## Tras findings del reviewer

    Si el reviewer encuentra issues y los arreglas, re-corre los tests que
    cubren el cambio y añade los resultados a tu report file — el reviewer
    no re-corre tests por ti; tu report es la evidencia.

    ## Report Format

    Escribe el report completo en [REPORT_FILE]: qué implementaste (o qué
    intentaste, si quedaste bloqueado), qué testeaste y resultados,
    evidencia TDD si aplica (RED: comando + output de fallo esperado y por
    qué era el esperado; GREEN: comando + output de pase), ficheros
    cambiados, hallazgos de self-review, issues o concerns.

    Reporta luego SOLO (menos de 15 líneas — el detalle vive en el report
    file):
    - **Status:** DONE | DONE_WITH_CONCERNS | BLOCKED | NEEDS_CONTEXT
    - Commits creados (SHA corto + subject), o "sin commit (política)"
    - Rulings, si los hay
    - Resumen de test en una línea (p.ej. "14/14 passing, output pristine")
    - Tus concerns, si los hay
    - El path del report file

    Si BLOCKED o NEEDS_CONTEXT, pon los detalles en el mensaje final mismo
    — el orquestador actúa directo sobre eso.

    Usa DONE_WITH_CONCERNS si completaste el trabajo pero dudas de su
    corrección. Usa BLOCKED si no puedes completar la tarea. Usa
    NEEDS_CONTEXT si necesitas información que no te dieron. Nunca
    produzcas en silencio trabajo del que no estás seguro.
```

**Placeholders:**
- `[BRIEF_FILE]` — REQUIRED: `scripts/task-brief PLAN_FILE N` imprime el
  path (el mismo fichero que luego lee el reviewer).
- `[REPORT_FILE]` — REQUIRED: nombrado tras el brief (`task-N-brief.md` →
  `task-N-report.md`).
- `[directorio]` — working directory de la tarea (en una ola de ≥2 con
  worktrees, el worktree de esa tarea).
- `[COMMIT_POLICY]` — `commitea` (por defecto; también en worktree propio)
  o `no commitees` (ola en el mismo worktree; ver `olas.md`).
- `[Encaje]` — una línea de dónde vive esta tarea + interfaces y
  decisiones de tareas anteriores que el brief no puede conocer + tu
  resolución de cualquier ambigüedad que notaste en el brief. Si un
  `Ruling:` cambió una interfaz que esta tarea consume, el orquestador lo
  pasa aquí. En modo worktree por tarea, pasa también rutas ABSOLUTAS del
  worktree principal para ledger, brief y report.
