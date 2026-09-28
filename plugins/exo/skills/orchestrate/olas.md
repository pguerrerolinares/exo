# Olas: mecánica

Regla: dos tareas comparten ola si sus `Files` son disjuntos y ninguna
consume, directa o transitivamente, algo que produce la otra (`Interfaces`).

## Calcular

- `scripts/task-dag PLAN` imprime `{"olas": [[ids]…], "avisos": [...]}`. Cada
  aviso se copia tal cual al ledger:
  - `DAG: secuencial (<motivo>)`: formato antiguo, Files ilegibles, `Consumes`
    sin `@Task M` o con `@Task` inexistente. Una tarea por ola.
  - `DAG: reordenado (Task i consume Task j posterior)`: consumo hacia delante
    sin ciclo; las olas ya vienen en orden topológico (desempata la numeración).
  - `DAG: error (<ciclo>)` (olas `[]`, exit 0): ciclo o autoconsumo. Es una
    decisión del humano: entra en la pregunta batcheada del pre-flight, antes de
    empezar (corregir el plan o fijar el orden). Nunca se ejecuta con error.
- La sección `## Olas` del plan es orientativa; manda `task-dag`.
- Solo si el script falla: a mano con la misma regla, leyendo `Files` e
  `Interfaces`. Ambiguo o no parseable ⇒ una tarea por ola, en orden, y
  `DAG: secuencial (<motivo>)` en el ledger.

## Ejecutar una ola

- Ola de 1 tarea: flujo normal.
- Ola de ≥2: un worktree por tarea (rama propia desde el HEAD de la ola) y
  todos los dispatches en UN mensaje. Cada executor commitea en su worktree.
  Pasa siempre rutas ABSOLUTAS del worktree principal para el brief y el
  report (el `Ruling:` va en el report; el orquestador lo copia al ledger).
- Prompt paralelo: scope, self-contained, constraints, output esperado. Al
  volver: lee cada summary, verifica que no choquen, spot-check.
- **Sin poder crear worktrees** (p.ej. sesión ya aislada en uno): paralelo en
  el MISMO worktree, `Files` disjuntos, y los executors **no commitean**
  (`[COMMIT_POLICY]` = no commitees). El orquestador:
  - commitea en serie con rutas explícitas (`git add <rutas de Files>`, nunca
    `-A`/`.`), registrando BASE_i = HEAD antes de cada commit, y hace el
    package por tarea con BASE_i..HEAD_i;
  - sin commit, cada executor corre solo los tests de su tarea; la suite
    completa la corre el orquestador tras commitear todas;
  - compara `git status --porcelain` con la unión de `Files`: cualquier
    fichero de más es BLOCKED o Ruling.
- Pipeline de reviews: la tarea N+1 no espera a que se revise la N; package +
  reviewer en cuanto cada una da DONE.

## Cerrar una ola

0. Merge solo de tareas con ambos verdictos ✅ (Critical/Important arreglados
   en su worktree, con re-review). Si una tarea sigue en review, la ola no
   cierra.
1. Merge de las ramas de la ola (en el mismo worktree no hay merge).
2. Suite completa. Roja ⇒ NO se abre la siguiente ola; fix dispatch con el
   output completo.
3. Conflicto de merge ⇒ executor de integración con los dos diffs. Si no
   resuelve, BLOCKED y pregunta al humano. Nunca `-X ours/theirs` a ciegas.
4. Solo hallazgos tardíos: Critical/Important de una tarea ya mergeada ⇒
   fix dispatch antes de abrir la siguiente ola.
5. Ola nueva solo con la anterior mergeada y en verde.

## Mutación (review-package)

`EXO_MUTATION=0` la desactiva; `EXO_MUTATION_TIMEOUT` fija el timeout en
segundos (por defecto 600; uno inválido avisa en la sección y usa 600). Muta en
un worktree temporal de `HEAD`: el árbol del usuario no se toca y vale
cualquier `BASE..HEAD`. En monorepo hay además una línea `MUTACIÓN [<dir>]: …`
por proyecto. Cargo muta solo las líneas del diff; `mutmut` 3.x siempre da
`no disponible` (no acota por CLI).

- Sin sección `MUTACIÓN:`, o `no disponible (<motivo>)`/`parcial (<motivo>)` ⇒
  esa línea al ledger; no bloquees ni corras mutación a mano.
- Coste: techo 600 s × N proyectos por package. `EXO_MUTATION=0` en re-reviews
  de fixes sin lógica nueva. En la review final, según si las tareas ya pasaron
  mutación: si todas la pasaron, `0`; si no, se deja activa.

## Ruling

El executor puede desviarse del contrato del plan (incluso de un test) si lo
deja como `Ruling: T<n> — <qué cambia> — <por qué>` en su report; el
orquestador (único escritor del ledger) lo copia allí. El reviewer lo trata como finding obligatorio. Si el Ruling cambia una interfaz, las
tareas de olas posteriores que la consumen lo reciben en su brief.

## Métricas de la serie (al cerrar la rama)

Las escribe el orquestador, una línea en el ledger: wall-clock y turnos de la fase plan (los lee del header del plan); KB del plan y
fracción de código; mutation score por herramienta; % de tests basura
(muestra clasificada por el reviewer final); nº y ancho de las olas.
