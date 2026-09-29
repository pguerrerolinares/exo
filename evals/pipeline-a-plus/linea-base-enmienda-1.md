# Línea base de mutation score, enmienda 1 al pre-registro

> Fijada el 2026-09-29, antes de correr la segunda medición. `linea-base-preregistro.md` (`5141d60`) no se reescribe: sigue siendo el original. Esta enmienda solo cambia lo que dice que cambia. Todo lo que fija queda fijado antes de medir; lo que no esté aquí se decide después y se declara como tal en el verdict.

## Por qué se enmienda

El verdict (`linea-base-verdict.md`) concluyó que 13/13 con n=1 rama no sirve como línea base: mutmut mutaba ficheros enteros (~920 mutantes por rama), nunca terminaba, un mutante colgado bloqueaba la corrida y el timeout descartaba el progreso. Además, "código de producción" quedó sin definir. La decisión sobre qué hacer es del dueño; esta enmienda recoge la opción (a) del verdict, arreglar el instrumento, más la definición que faltaba.

## Qué cambia

### Instrumento

- `review-package` de exo **≥ 1.5.1**, con la mutación acotada al diff (Task 1 del plan 3) y timeout por mutante (`EXO_MUTATION_MUTANT_TIMEOUT`).
  - mutmut 2.5.1: `--use-patch-file`.
  - stryker 9.6.1: `--mutate` con rangos de líneas.
  - cargo-mutants 27.1.0: sin cambios, ya usaba `--in-diff`.
- `EXO_MUTATION_TIMEOUT=3600` se mantiene.
- Versiones de las herramientas: las mismas del pre-registro.

### Definición de "código de producción"

Resuelve la ambigüedad declarada en el verdict. Se **excluyen**: harness de evals, migraciones generadas y tests. Todo lo demás mutable en un ecosistema soportado cuenta como producción.

- En exo la selección **no cambia**: sigue siendo campaña L. Las ramas J fase 1 (`01be694`, `aaf66c0`) quedan descartadas también con esta definición, porque su único código mutable es el harness de evals.
- Esta definición la decidió el orquestador el 2026-09-29. **Es revertible por el dueño.**

### Agregado

- Σcaught / Σviables sobre las ramas que **no queden `parcial`** ni `no disponible`.
- Un `parcial` con progreso se reporta **aparte**, con su k/n (mutantes probados sobre totales), sin entrar en el agregado.

### Criterio de muestreo (fijado antes de medir)

Si tras esta corrida una rama sigue en `parcial` con el diff acotado, la **siguiente enmienda** introduce una muestra aleatoria de mutantes con semilla `20260929` e IC95 binomial. No se decide a posteriori ni se adelanta en esta enmienda.

## Qué NO cambia

- **Población y selección:** las mismas tres ramas, por el mismo criterio mecánico.
  - exo: campaña L (`7e29f42`, rango `5efe812..f79d817`).
  - pguerrero-music: `feat/portafolio-playlists` (`275b29f`, rango `c4a1ba1..c0bb059`).
  - bizkaia-now: B1 mapa (`aa7bf2c`, rango `cfc281b..73f9bfb`).
- **Métricas por rama** y clasificación manual de hasta 10 supervivientes (*equivalente*, *hueco de test real*, *código muerto o no alcanzable*).
- **Lo ya medido:** exo 13/13 se conserva sin remedir **si el instrumento no cambia para Rust** (cargo ya usaba `--in-diff`). Si cambiara, se remide.
- El pre-registro original, el verdict y los informes de `linea-base/` no se tocan.

## Lo que esto NO es

- Sigue siendo una base de n≈3 ramas, descriptiva, sin inferencia.
- No garantiza que las tres ramas terminen. Si alguna sigue `parcial`, se aplica el criterio de muestreo de arriba en la siguiente enmienda.
- stryker sigue dependiendo de que el proyecto lo declare en su `package.json` de HEAD; si bizkaia `web` sigue en `no disponible (stryker no instalado)`, se reporta así y no entra en el agregado.
