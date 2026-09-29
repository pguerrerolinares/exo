# Plan 3 — pipeline A+: mutación acotada al diff en Python y JS (contrato)

> For agentic workers: se ejecuta con `exo:orchestrate`. El plan es un contrato: el cuerpo lo escribe el executor.
> Plan: ~6 min, lo escribe el padre, ~5 KB, sin bloques de código.

**Goal:** que `review-package` produzca un mutation score de verdad en repos Python y JS/TS. Hoy mutmut muta ficheros enteros (~920 mutantes por rama) y nunca termina (verdict `evals/pipeline-a-plus/linea-base-verdict.md`).

**Architecture:** tres cambios en `review-package`:
- las dos herramientas se acotan a las líneas del diff con sus flags nativos;
- hay un timeout por mutante;
- el progreso parcial se conserva cuando salta el timeout global.

Aparte, se enmienda el pre-registro de la línea base. El muestreo de mutantes queda FUERA de este plan: solo entra si, ya acotado al diff, sigue sin terminar.

**Tech Stack:** bash + awk + jq. Los tests van en `plugins/exo/scripts/test-review-package.sh`, con stubs herméticos. Las herramientas reales están en el scratchpad de la sesión: `S=/tmp/claude-1000/-home-paul-Documentos-proyectos-exo/1ba28911-00d5-417b-b399-e72e68644522/scratchpad/mut`, con mutmut 2.5.1 en `$S/venv2`, stryker 9.6.1 en `$S/js` y cargo-mutants en `$S/cargo/bin`.

**Spec:** `docs/superpowers/specs/2026-09-29-pipeline-a-plus-design.md` §4 (review-package), §5 y §6.

**Global Constraints:**
- La degradación siempre es visible y nunca bloquea. Se conserva el vocabulario `MUTACIÓN: <score> | no disponible (…) | parcial (…)`, incluida la línea por proyecto `MUTACIÓN [<dir>]: …`.
- Los flags verificados contra los binarios reales (recon del 09-29) son:
  - mutmut 2.x: `--use-patch-file <patch>` ("Only mutate lines added/changed in the given patch file"), `-m/--test-time-multiplier`, `-b/--test-time-base`.
  - stryker: `--mutate "f.ts:inicio-fin,g.ts:inicio-fin"`, usando rangos de mutación.
- El worktree temporal de mutación y las demás garantías de exo 1.5.0 no cambian.
- `orchestrate/SKILL.md` sigue ≤ 4096 B.

## Olas

- Ola 1, en paralelo: Task 1 y Task 2 (sus Files son disjuntos).
- Después de mergear: se vuelve a medir la línea base con la enmienda. Eso lo hace el orquestador, no es una tarea.

### Task 1: acotar la mutación al diff y conservar el parcial

**Files:**
- Modify: `plugins/exo/skills/orchestrate/scripts/review-package`
- Test: `plugins/exo/scripts/test-review-package.sh`
- Modify: `plugins/exo/scripts/testdata/review-package/` (stubs)
- Modify: `plugins/exo/skills/orchestrate/olas.md` (sección Mutación)

**Interfaces:**
- Produces: la CLI de `review-package` y la sección `MUTACIÓN` no cambian de forma.
- Produces: la variable nueva `EXO_MUTATION_MUTANT_TIMEOUT=<s>`, timeout por mutante con un valor por defecto razonable que decides tú y documentas. Se traduce a `-b`/`-m` en mutmut; en stryker, a su `--timeoutMS`/`--timeoutFactor` si existe (verifícalo).

**Tests:**
- `mutmut_acotado_al_diff`: el stub de mutmut recibe `--use-patch-file <p>` y ese patch contiene solo las líneas de producción del rango, con rutas relativas al proyecto. Falla si muta el fichero entero o si pasa rutas de la raíz en un subproyecto.
- `stryker_rangos`: el stub recibe `--mutate` con `fichero:inicio-fin` por cada hunk añadido o modificado. Varios hunks del mismo fichero dan varios rangos, en un solo `--mutate` separado por comas. Falla si pasa el fichero sin rango.
- `hunk_solo_borrado`: un fichero cuyo diff solo borra líneas no genera rango y no llama a la herramienta, porque no hay nada que mutar.
- `parcial_conserva_progreso`: un stub que avanza, imprime progreso y se cuelga hasta el timeout global. La salida es `parcial (timeout Ns) — progreso: k/n evaluados, c caught, s supervivientes` con los números del stub. Falla si descarta el conteo.
- `timeout_por_mutante`: `EXO_MUTATION_MUTANT_TIMEOUT` llega a la herramienta con el flag correcto.

**Verificación:**
- Los dos tests del script en verde, `bash scripts/test-plugin.sh` y `SHELLCHECK=$S/../sc/shellcheck-v0.11.0/shellcheck bash scripts/test-shellcheck.sh`.
- **Con binarios reales:**
  - (1) mutmut 2.5.1 sobre un paquete mínimo en el scratchpad, con un diff de 1 función en un fichero de 3 funciones: el número de mutantes baja frente a sin patch, y hay que reportar las dos cifras.
  - (2) stryker 9.6.1 con un rango: solo muta ese rango.
  - (3) el caso colgado. Un mutante que genera bucle infinito lo corta el timeout por mutante y la corrida termina.

**Review Focus:**
- Lectura de progreso en mutmut 2.x: el formato real de la línea de progreso ya está en el script. Stryker, en `parcial`: ¿qué escribe antes de morir? Si no hay forma fiable, `progreso: no disponible`, nunca un número inventado.
- El patch para `--use-patch-file` es relativo a la raíz del proyecto de mutmut (el cwd), igual que el fix de workspace de cargo.
- Rangos de stryker: se usan los números de línea del lado NUEVO del hunk (`+a,b`).

### Task 2: enmienda del pre-registro de la línea base

**Files:**
- Create: `evals/pipeline-a-plus/linea-base-enmienda-1.md`

**Interfaces:** ninguna.

**Tests:** n/a, es prosa de protocolo.

**Contenido:** una enmienda fechada que NO reescribe el pre-registro original. Lleva:
- **Instrumento:** exo ≥ 1.5.1, con la mutación acotada al diff (Task 1) y `EXO_MUTATION_MUTANT_TIMEOUT`. `EXO_MUTATION_TIMEOUT=3600` se mantiene.
- **Selección:** sin cambios, con las mismas ramas (campaña L, `feat/portafolio-playlists`, B1 mapa).
- **Definición de producción,** que resuelve la ambigüedad: se excluyen harness de evals, migraciones generadas y tests. En exo la selección no cambia (campaña L). Lo decidió el orquestador el 09-29 y es revertible por el dueño.
- **Agregado:** Σcaught/Σviables sobre las ramas que no queden `parcial`. Un `parcial` con progreso se reporta aparte con su k/n.
- **Criterio de muestreo, fijado antes de medir:** si una rama sigue en `parcial` con el diff acotado, la siguiente enmienda introduce una muestra aleatoria con semilla `20260929` e IC95 binomial. No se decide a posteriori.
- **Lo ya medido:** exo 13/13 se conserva si el instrumento no cambia para Rust (cargo ya usaba `--in-diff`).

**Verificación:** `bash scripts/test-rutas-personales.sh` y `bash scripts/test-docs-vivos.sh` en verde.
