# Olas: mecánica

Regla: dos tareas comparten ola si sus `Files` son disjuntos y ninguna
consume, directa o transitivamente, algo que produce la otra (`Interfaces`).

## Calcular

- Con `scripts/task-dag PLAN` (plan 2), si existe: stdout JSON `olas` + `avisos`.
- Sin script, lo calculas tú a mano con la misma regla, leyendo `Files` e
  `Interfaces` de cada tarea. Ambiguo o no parseable ⇒ una tarea por ola, en
  orden, y una línea `DAG: secuencial (<motivo>)` en el ledger.

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

## Ruling

El executor puede desviarse del contrato del plan (incluso de un test) si lo
deja como `Ruling: T<n> — <qué cambia> — <por qué>` en su report; el
orquestador (único escritor del ledger) lo copia allí. El reviewer lo trata como finding obligatorio. Si el Ruling cambia una interfaz, las
tareas de olas posteriores que la consumen lo reciben en su brief.

## Métricas de la serie (al cerrar la rama)

Una línea en el ledger: wall-clock y turnos de la fase plan (los lee del header del plan); KB del plan y
fracción de código; mutation score por herramienta; % de tests basura
(muestra clasificada por el reviewer final); nº y ancho de las olas.
