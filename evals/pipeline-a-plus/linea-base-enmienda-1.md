# Línea base de mutation score, enmienda 1 al pre-registro

> Fijada el 2026-09-29, antes de correr la segunda medición. `linea-base-preregistro.md` (`5141d60`) no se reescribe: sigue siendo el original. Esta enmienda solo cambia lo que dice que cambia. Todo lo que fija queda fijado antes de medir; lo que no esté aquí se decide después y se declara como tal en el verdict.
>
> Quién decide qué: **el dueño eligió la opción (a)** del verdict (arreglar el instrumento) el 2026-09-29. Todo lo demás de esta enmienda (definición de "producción", exclusiones, parámetros, criterio de muestreo) **lo decidió el orquestador** y se declara así; es revertible por el dueño.

## Por qué se enmienda

El verdict (`linea-base-verdict.md`) concluyó que 13/13 con n=1 rama no sirve como línea base: mutmut mutaba ficheros enteros (~920 mutantes por rama), nunca terminaba, un mutante colgado bloqueaba la corrida y el timeout descartaba el progreso. Además, "código de producción" quedó sin definir. La decisión sobre qué hacer es del dueño; esta enmienda recoge la opción (a) del verdict, arreglar el instrumento, más la definición que faltaba.

## Qué cambia

### Instrumento

- **Commit exacto: `026c166`** (`fix(review-package)`, rama `a-plus-plan3`, plugin exo 1.5.1). Es el último commit que toca `review-package`; se usa ese SHA, no la versión, para fijar el instrumento.
  - mutmut 2.5.1: `--use-patch-file`, acotado al diff. Sin `-b`. El timeout por mutante envuelve el `--runner` (el de su config, o el default `python -m pytest -x --assert=plain`); un mutante cortado cuenta como *caught* (timeout = detectado). 🤔 (matado lento) también cuenta como *caught*.
  - stryker 9.6.1: `--mutate` con rangos de líneas. **No** se pasa `--timeoutMS`: se conserva su timeout por defecto, el de 1.5.0.
  - cargo-mutants 27.1.0: sin cambios, ya usaba `--in-diff`.
- Versiones de las herramientas, fijadas: **mutmut 2.5.1, whatthepatch 1.0.7** (la instalada, comprobada con `pip list`), **stryker 9.6.1, cargo-mutants 27.1.0**.
- Parámetros: `EXO_MUTATION_TIMEOUT=3600` y `EXO_MUTATION_MUTANT_TIMEOUT=60` (solo aplica a mutmut). Las corridas van **secuenciales, nunca en paralelo**.
- Todas las llamadas a `git diff` del script son inmunes a la config del usuario (`noprefix`, `mnemonicPrefix`, color, diff externo).

### Definición de "código de producción"

Resuelve la ambigüedad declarada en el verdict. Se **excluyen**: harness de evals, código generado y tests. Todo lo demás mutable en un ecosistema soportado cuenta como producción.

- **"Generado"** significa: migraciones de alembic/django, `*_pb2.py`, `*.gen.*` y lock files.
- Las exclusiones se aplican con `EXO_MUTATION_EXCLUDE` (pathspecs de git separados por comas, relativos a la raíz del repo), antes de agrupar por proyecto. Por repo:
  - **exo:** ninguna (`EXO_MUTATION_EXCLUDE` vacío).
  - **pguerrero-music:** ninguna. No tiene migraciones ni código generado; `uv.lock` es lock file pero no está en el diff ni es mutable.
  - **bizkaia-now:** `EXO_MUTATION_EXCLUDE='alembic/versions/**'`. Las migraciones que hay en su árbol son `0001` a `0004`, todas bajo `alembic/versions/` (en el diff entran `0003` y `0004`). `web/bun.lock` es lock file y no es mutable, no hace falta pathspec.
- En exo la selección **no cambia**: sigue siendo campaña L. Las ramas J fase 1 (`01be694`, `aaf66c0`) quedan descartadas también con esta definición, porque su único código mutable es el harness de evals.
- Esta definición la decidió el orquestador el 2026-09-29. **Es revertible por el dueño.**

### Agregado

- Σcaught / Σviables sobre las ramas que **no queden `parcial`** ni `no disponible`.
- Un `parcial` con progreso se reporta **aparte**, con su k/n (mutantes probados sobre totales), sin entrar en el agregado.

### Criterio de muestreo (fijado ahora, antes de medir)

Sustituye a la versión anterior de esta enmienda, que lo dejaba para una enmienda posterior.

- **Disparador:** solo un `parcial (timeout …)`. Un `parcial (baseline falló)` o `parcial (salida no reconocida)` **no** dispara el muestreo.
  - Esos dos permiten **un reintento**, tras un arreglo de entorno documentado. Solo entorno, nunca parámetros del script.
- **Muestra:** aleatoria simple **sin reemplazo** de **n=150 mutantes** del conjunto ya acotado al diff, o todos si hay menos. **Semilla `20260929`.** Unidad: el mutante, por su id de la herramienta.
  - Mecánica: ids ordenados tal como los lista la herramienta y `random.Random(20260929).sample(ids, 150)`.
- **Intervalo:** IC95 de **Wilson** sobre la proporción de *caught* de la muestra.

### Progreso de stryker (comparabilidad)

En el `parcial (timeout …)` de stryker, `k/n` excluye NoCoverage, `caught` incluye Runtime/CompileError y la línea llega con hasta 10 s de retraso. No es comparable con el score final ni con el de otra herramienta. Está documentado en `olas.md`.

## Qué NO cambia

- **Población y selección:** las mismas tres ramas, por el mismo criterio mecánico. Con la nueva definición se ha **verificado** que siguen siendo las mismas: el diff de cada una tiene producción no excluida y no hay otra candidata más reciente en su repo.
  - exo: campaña L (`7e29f42`, rango `5efe812..f79d817`). El diff tiene `engine/src/buscador.rs`.
  - pguerrero-music: `feat/portafolio-playlists` (`275b29f`, 2026-09-24, rango `c4a1ba1..c0bb059`). Producción no excluida: `build_playlists.py`, `discovery.py`, `families.py`, `prune_playlists.py`, `sync_library.py`. Es el merge más reciente de la ventana en ese repo.
  - bizkaia-now: B1 mapa (`aa7bf2c`, 2026-09-28, rango `cfc281b..73f9bfb`). Con `alembic/versions/**` excluido quedan 12 ficheros `.py` de `src/` y `scripts/` más 20 de `web/`. Es el merge más reciente de la ventana en ese repo.
- **Métricas por rama** y clasificación manual de hasta 10 supervivientes (*equivalente*, *hueco de test real*, *código muerto o no alcanzable*).
- **Lo ya medido:** exo 13/13 se **conserva** sin remedir. El camino de cargo no cambió en este diff: `cargo-mutants` recibe los mismos argumentos que antes y el resto de los cambios afectan solo a mutmut y a stryker.
- El pre-registro original, el verdict y los informes de `linea-base/` no se tocan.

## Lo que esto NO es

- Sigue siendo una base de n≈3 ramas, descriptiva, sin inferencia.
- No garantiza que las tres ramas terminen. Si alguna sigue `parcial (timeout …)`, se aplica el muestreo de arriba en esa misma corrida.
- stryker sigue dependiendo de que el proyecto lo declare en su `package.json` de HEAD; si bizkaia `web` sigue en `no disponible (stryker no instalado)`, se reporta así y no entra en el agregado.
