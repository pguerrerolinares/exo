# Plan 2 — pipeline A+: scripts `task-dag` y mutación en `review-package` (contrato)

> For agentic workers: se ejecuta con `exo:orchestrate`. Es un contrato: el cuerpo de tests y código lo escribe el executor.
> Plan: ~8 min, escrito por el padre, ~9 KB, 0 bloques de código.

**Goal:** implementar los dos scripts del plan 2 de la spec A+ y corregir el bug del comentario de presupuesto en `compose-inject.sh`, que se detectó durante el plan 1.

**Architecture:**
- `task-dag` es un script nuevo en bash y awk. Lee un plan con el formato de `skills/plan/plan-template.md` y emite olas en JSON.
- `review-package` añade al package una sección `MUTACIÓN:`. Para rellenarla detecta la herramienta de mutación del repo y la ejecuta solo sobre los ficheros de producción del diff, con timeout.
- En la ola 2, orchestrate deja de calcular olas a mano y pasa a llamar a `task-dag`.

**Tech Stack:** bash + awk + jq. Ya son dependencias del plugin. No se añade python, para no romper la portabilidad en Git Bash y macOS. Los tests son `plugins/exo/scripts/test-*.sh`, que descubre `scripts/test-plugin.sh` y corre CI.

**Spec:** `docs/superpowers/specs/2026-09-29-pipeline-a-plus-design.md` (§4 "Plan 2", §5 y §6).

**Global Constraints:**
- Si algo falla, se degrada de forma visible y nunca bloquea. Exit ≠0 solo por un error de uso.
- `timeout` portable: reutiliza `plugins/exo/scripts/_timeout.sh` (`con_timeout`). No llames a `timeout` directamente.
- Los scripts nuevos van en `100755` (lo comprueba `scripts/test-exec-bit.sh`) y deben quedar limpios en shellcheck (lo aplica CI).
- Vocabulario literal:
  - `MUTACIÓN: <score> + supervivientes` | `MUTACIÓN: no disponible (<motivo>)` | `MUTACIÓN: parcial (<motivo>)`
  - `DAG: secuencial (<motivo>)`
  - salida de task-dag: `{"olas": [[<ids>], ...], "avisos": [...]}`
- Cabecera de atribución en los scripts derivados de superpowers: se mantiene la que ya tienen.

## Olas

- Ola 1, en paralelo: Task 1, Task 2, Task 3. Sus `Files` son disjuntos y ninguna consume nada de otra.
- Ola 2: Task 4 (wiring), que consume Task 1 y Task 2.

### Task 1: task-dag

**Files:**
- Create: `plugins/exo/skills/orchestrate/scripts/task-dag`
- Test: `plugins/exo/scripts/test-task-dag.sh`
- Create: `plugins/exo/scripts/testdata/task-dag/` (fixtures `.md`)

**Interfaces:**
- Produces: `task-dag PLAN_FILE` → stdout JSON `{"olas": [[int]], "avisos": [string]}`. Exit 0 aunque el plan esté mal formado, y exit 2 si el fichero no existe o falta el argumento.

**Tests:**
- `independientes`: T1 y T2 con Files disjuntos y sin Consumes dan `[[1,2]]`. Falla si serializa tareas independientes.
- `consume`: T2 con `Consumes: \`f()\` @Task 1` da `[[1],[2]]`. Falla si ignora la arista.
- `transitivo`: T3 consume T2 y T2 consume T1, con Files disjuntos, da `[[1],[2],[3]]`.
- `fichero_compartido`: T1 y T2 con el mismo path en Files y sin Consumes dan `[[1],[2]]`, en orden de aparición.
- `mixto`: T1 y T2 independientes y T3 que consume T1 dan `[[1,2],[3]]`.
- `sin_formato`: un plan sin `**Files:**` en alguna tarea da una tarea por ola, en orden, y `avisos` contiene `DAG: secuencial (`. Falla si inventa paralelismo sin datos.
- `consume_inexistente`: `@Task 9` en un plan de 3 tareas da secuencial y un aviso.
- `fences`: los encabezados `### Task N:` dentro de bloques ``` se ignoran, igual que en `task-brief`.
- `json_valido`: toda salida pasa `jq -e .`.

**Verificación:** `plugins/exo/scripts/test-task-dag.sh` en verde y `bash scripts/test-plugin.sh` en verde.

**Review Focus:**
- La regla es "comparten ola" solo si Files son disjuntos **y** no hay consumo transitivo. El algoritmo de asignación (una tarea va a la primera ola posterior a todas sus dependencias y a las olas donde ya hay tareas con las que comparte ficheros) no debe meter en la misma ola dos tareas que comparten fichero aunque ninguna consuma a la otra.
- Paths entre backticks y con espacios.
- Encabezados `## Task N —` (formato antiguo, como el del plan 1) frente a `### Task N:`: acepta los dos, con la regex de `task-brief`.

### Task 2: mutación en review-package

**Files:**
- Modify: `plugins/exo/skills/orchestrate/scripts/review-package`
- Test: `plugins/exo/scripts/test-review-package.sh`
- Create: `plugins/exo/scripts/testdata/review-package/` (stubs de herramientas y repos mínimos)

**Interfaces:**
- Produces: la misma CLI de `review-package BASE HEAD [OUTFILE]`, con una sección nueva al final del package que empieza por la línea `## MUTACIÓN` y cuya primera línea de contenido es `MUTACIÓN: …`, con el vocabulario.
- Produces: variables de entorno `EXO_MUTATION=0`, que desactiva y da `no disponible (desactivada)`, y `EXO_MUTATION_TIMEOUT=<s>`, con 600 por defecto.

**Tests** (herramientas simuladas con stubs en `PATH`, nunca las reales):
- `sin_herramienta`: un repo sin `Cargo.toml`/`pyproject.toml`/`setup.py`/`package.json` da `MUTACIÓN: no disponible (sin herramienta para este repo)`.
- `herramienta_ausente`: un repo con `Cargo.toml` pero sin `cargo-mutants` en PATH da `no disponible (cargo-mutants no instalado)`.
- `timeout`: un stub que duerme más que `EXO_MUTATION_TIMEOUT=1` da `parcial (timeout 1s)`. El script tiene que terminar en menos de 5 s.
- `rust_ok`: el stub de `cargo mutants` escribe un resultado con 10 mutantes, 2 missed, y la salida contiene `MUTACIÓN: 8/10` y los 2 supervivientes. Falla si el score se calcula mal o si se pierden los supervivientes.
- `solo_diff`: el stub recibe como argumentos solo ficheros de producción del diff (ni tests ni `.md`). Falla si muta todo el repo.
- `diff_sin_produccion`: un diff que solo toca `.md` o tests da `no disponible (diff sin código de producción)` y no llama al stub.
- `desactivada`: `EXO_MUTATION=0` da `no disponible (desactivada)`.
- `regresion_package`: las secciones previas del package (Commits, Files changed, diff) no cambian.

**Verificación:** `plugins/exo/scripts/test-review-package.sh` en verde y `bash scripts/test-plugin.sh` en verde.

**Review Focus:**
- Invocación real de cada herramienta, contrastada con su documentación actual (`--help` o docs), no de memoria:
  - `cargo mutants` (`--in-diff`/`--file`, salida en `mutants.out/`)
  - `mutmut` (2.x `--paths-to-mutate` frente a 3.x por config)
  - `npx stryker run --mutate`
  Si una versión no admite acotar a ficheros, el resultado es `no disponible (<motivo>)`, nunca mutar el repo entero.
- La salida de la herramienta no se puede parsear: `parcial (salida no reconocida)` más las últimas ~20 líneas en crudo.
- El timeout tiene que matar al grupo de procesos (`con_timeout` ya lo hace), sin dejar huérfanos.
- La mutación se corre en el worktree del HEAD. Si el árbol está sucio, `no disponible (árbol sucio)`, para no mutar cambios ajenos.

### Task 3: bug del presupuesto en compose-inject

**Files:**
- Modify: `plugins/exo/scripts/compose-inject.sh`
- Test: `plugins/exo/scripts/test-compose-inject.sh` (añadir caso)

**Interfaces:** ninguna.

**Contexto:** `doctrina()` dice "cap 800B por linea" y `doctrina_compacta()` dice "cap 550B por linea". Pero `cap_lines` aplica un presupuesto **total** en bytes y corta por líneas enteras: con el `executor.md` actual solo pasan los primeros ~800 B del cuerpo. Primero decide qué está mal, si el comentario o el comportamiento. Para eso busca la intención original en `git log -S "cap_lines" -- plugins/exo/scripts/compose-inject.sh`, en la spec de transporte (§5.1, que cita la cabecera del script; búscala en `docs/`) y en los tests existentes.
- Si la intención era un total, arregla los comentarios.
- Si era por línea, arregla el comportamiento siguiendo TDD. El test tiene que fallar primero.

Es cualquiera de los dos, no ambos, y la decisión con su evidencia va al report.

**Tests:**
- Si se arregla el comportamiento: `doctrina_por_linea`, donde un `executor.md` de fixture con 3 líneas de 500 B inyecta las 3. Falla con el cap total actual.
- Si solo se arreglan los comentarios: `Tests: n/a — solo comentario`, y la suite sigue 19/19.

**Verificación:** `plugins/exo/scripts/test-compose-inject.sh` en verde.

**Review Focus:**
- Si el comportamiento cambia, el bloque inyectado crece en todos los subagentes. Hay que medir los bytes antes y después con el `executor.md` real y reportarlos, porque es un coste en cada dispatch.

### Task 4: wiring de orchestrate y versión (ola 2)

**Files:**
- Modify: `plugins/exo/skills/orchestrate/SKILL.md`, `plugins/exo/skills/orchestrate/olas.md`, `plugins/exo/skills/orchestrate/reviewer-prompt.md`
- Modify: `plugins/exo/.claude-plugin/plugin.json`, `.claude-plugin/marketplace.json`, `plugins/exo/README.md`

**Interfaces:**
- Consumes: `task-dag PLAN_FILE` @Task 1
- Consumes: sección `MUTACIÓN:` @Task 2

**Tests:** `Tests: n/a — prosa y versión`. Los cubren los gates de CI citados en Verificación.

**Contenido:**
- olas.md y SKILL.md: se calcula con `task-dag`. Hacerlo a mano queda solo como fallback si el script falla. Si el JSON trae `avisos`, se copian al ledger.
- Se quita "(plan 2)" y "hasta que exista" en los sitios donde ya no aplica.
- SKILL.md: con `MUTACIÓN: no disponible`, la regla del ledger sigue igual.
- reviewer-prompt: con `MUTACIÓN: <score>`, cada superviviente es un finding (Important si es lógica de la tarea, Minor si es borde). `parcial` se trata como "cannot verify" solo para lo que no cubrió.
- Versión 1.5.0 en `plugin.json` y en `marketplace.json`.
- README: se añade una línea con los scripts nuevos.
- `orchestrate/SKILL.md` ≤ 4096 B.

**Verificación:** `bash scripts/test-versiones.sh`, `BASE=$(git merge-base main HEAD) bash scripts/plugin-bump-gate.sh`, `bash scripts/test-docs-vivos.sh`, `bash scripts/test-exec-bit.sh` y `bash scripts/test-plugin.sh`, todos en verde.

**Review Focus:**
- La PARIDAD CRÍTICA de orchestrate se mantiene intacta.
- El gold `evals/prep-m3/gold/orchestrate.md` no pierde ningún movimiento.
