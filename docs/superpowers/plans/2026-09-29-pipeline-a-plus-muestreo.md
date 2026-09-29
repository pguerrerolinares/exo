# Plan — muestreo de mutantes en `review-package` (contrato)

> For agentic workers: se ejecuta con `exo:orchestrate`. Es un contrato: el cuerpo lo escribe el executor.
> Plan: ~4 min, escrito por el padre, ~3 KB, sin bloques de código.

**Goal:** que `review-package` mida con una muestra aleatoria cuando el diff genera demasiados mutantes, usando el mismo protocolo que la base de bizkaia (`evals/pipeline-a-plus/linea-base-2-verdict.md`): n=150 sin reemplazo, semilla `20260929`, IC95 de Wilson.

**Architecture:** el muestreo es por adelantado, no tras el timeout. mutmut 2.x primero genera y enumera los mutantes acotados al diff sin ejecutar tests. Si hay más de N, corre solo una muestra de ids con `mutmut run <id>`. Esta versión solo cubre mutmut. En cargo y stryker el muestreo sale como `no aplica`, sin fallar.

**Tech Stack:** bash + awk. El muestreo tiene que ser reproducible y portable, sin python ajeno a la herramienta. Se puede usar el python del venv de mutmut, que ya existe cuando mutmut corre. Real: mutmut 2.5.1 en `S=/tmp/claude-1000/-home-paul-Documentos-proyectos-exo/1ba28911-00d5-417b-b399-e72e68644522/scratchpad/mut/venv2` y repos de prueba en `scratchpad/rv3/`.

**Global Constraints:**
- Se mantiene el vocabulario de `MUTACIÓN`. La línea muestreada tiene este formato: `MUTACIÓN [<dir>]: c/n (p%) IC95 [a%, b%] — muestra de n/total (semilla s)`.
- Variables: `EXO_MUTATION_SAMPLE=<N>` (default 150; 0 desactiva el muestreo) y `EXO_MUTATION_SEED=<s>` (default 20260929).
- El timeout global sigue aplicando, y si salta dentro de la muestra, sale `parcial` con el progreso de la muestra.
- La lección del verdict 2 es que los estados cacheados de mutmut invalidaron una pasada. El worktree temporal es nuevo en cada corrida, y la muestra no puede leer resultados de una corrida anterior.

## Olas

- Ola 1: Task 1.

### Task 1: muestreo por adelantado en mutmut

**Files:**
- Modify: `plugins/exo/skills/orchestrate/scripts/review-package`
- Test: `plugins/exo/scripts/test-review-package.sh`
- Modify: `plugins/exo/scripts/testdata/review-package/bin/mutmut`
- Modify: `plugins/exo/skills/orchestrate/olas.md`
- Modify: `plugins/exo/.claude-plugin/plugin.json`, `.claude-plugin/marketplace.json` (1.5.2)

**Interfaces:**
- Produces: `EXO_MUTATION_SAMPLE` y `EXO_MUTATION_SEED`, más la línea `MUTACIÓN` muestreada del formato fijado.

**Tests:**
- `muestra_por_encima_de_n`: con 400 mutantes y N=150 corren exactamente 150 ids. La salida lleva `muestra de 150/400 (semilla 20260929)` y un IC. Falla si corre todos o si el IC falta.
- `muestra_reproducible`: dos corridas con la misma semilla eligen los mismos ids. Con otra semilla la selección cambia.
- `por_debajo_de_n`: con 100 mutantes y N=150 corren todos, sin IC y con la línea de siempre.
- `muestreo_desactivado`: `EXO_MUTATION_SAMPLE=0` ejecuta el comportamiento completo de 1.5.1.
- `wilson_correcto`: para 33/150 da [16,1%, 29,3%] (los valores del verdict 2). Falla si usa el intervalo normal.
- `sin_cache_previa`: una segunda corrida sobre el mismo rango no reutiliza estados. Falla si termina en menos de un segundo con resultados.

**Verificación:**
- Tests en verde, más `bash scripts/test-plugin.sh`, shellcheck 0.11.0, `test-versiones` y `BASE=$(git merge-base main HEAD) bash scripts/plugin-bump-gate.sh`.
- **Real:** mutmut 2.5.1 sobre un paquete con más de 150 mutantes en el diff (genera uno en el scratchpad). Da la cifra y el tiempo.

**Review Focus:**
- La enumeración de mutantes sin ejecutar tests en mutmut 2.x. Una opción es el runner `true` más `result-ids`, como hizo el agente de medición. Verifica que eso no marca estados que luego contaminen la muestra (es la desviación 3 del verdict 2).
- Wilson con awk: precisión y el caso c=0 / c=n.
