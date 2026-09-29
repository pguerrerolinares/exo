# Línea base de mutation score, pipeline pre-A+ (pre-registro)

> Fijado el 2026-09-29, antes de correr ninguna medición. Lo que no esté aquí se decide después y se declara como tal en el verdict.

## Para qué

La spec `docs/superpowers/specs/2026-09-29-pipeline-a-plus-design.md` (§6) exige que A+ **no empeore el mutation score** frente a la línea base. Además, el test-writer sale de cuarentena solo si, tras 10 ramas, el score queda **por debajo de la base**. Este documento fija esa base.

## Población y selección

- **Repos:** `exo`, `pguerrero-music` y `bizkaia-now`, en `~/Documentos/proyectos/`.
- **Rama candidata:** cumple las tres condiciones.
  1. Se mergeó a `main` entre 2026-09-01 y 2026-09-28.
  2. Se ejecutó con el pipeline viejo, es decir, existe su plan en `docs/superpowers/plans/` con bloques de código (el formato previo a A+).
  3. Su diff contiene código de producción en un ecosistema que `review-package` sabe mutar: Rust con cargo-mutants, Python con mutmut 2.x, JS/TS con stryker.
- **Selección mecánica:** en cada repo, la rama candidata **más reciente** por fecha de merge. Si un repo no tiene ninguna, se toma la segunda más reciente de `exo`. El objetivo son 3 ramas.
- **Rango:** `merge-base(primer padre del merge, rama) .. segundo padre del merge`. Es el diff completo de la rama. Si el repo hace squash o fast-forward y no hay merge commit, se usa el rango de commits que cita el plan o el ledger y se declara.

## Instrumento

- `exo/plugins/exo/skills/orchestrate/scripts/review-package BASE HEAD` (exo 1.5.0). Muta en un worktree temporal y no toca el árbol de trabajo.
- Versiones fijadas: cargo-mutants 27.1.0, mutmut 2.5.1 (la 3.x no acota por CLI), stryker 9.6.1.
- `EXO_MUTATION_TIMEOUT=3600`.

## Métricas

- Por rama: `caught/viables` (la sección `MUTACIÓN`), la lista de supervivientes, el tiempo y el `parcial` si lo hubo.
- Agregado: Σcaught / Σviables. Las ramas `parcial` o `no disponible` se reportan aparte y **no entran en el agregado**.
- De cada rama se clasifican a mano hasta 10 supervivientes en tres clases: *equivalente* (ningún test podría distinguirlo), *hueco de test real* y *código muerto o no alcanzable*. Esto no cambia el score; solo lo interpreta.

## Lo que esto NO es

- Es una sola base de n≈3 ramas, que no hace inferencia. Es el punto de comparación descriptivo que pide la spec.
- No mide la corrección de las ramas, solo lo que sus tests detectan.
