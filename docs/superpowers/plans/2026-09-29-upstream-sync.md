# Plan: upstream-sync

> For agentic workers: ejecución con `exo:orchestrate`.

**Goal:** Un bot semanal (routine cloud) que porta a exo, por PR revisado por Paul, lo que cambia en cada release de obra/superpowers, con estado versionado, watchdog que grita y un eval pre-registrado antes de programarlo.

**Architecture:** El juicio (triage + porte) vive en un prompt versionado (`docs/upstream/sync-prompt.md`). Todo lo mecánico va en scripts bash testeados en CI: reconciliar el ledger por evidencia git, comprobar referencias entre skills, vigilar el latido y puntuar el eval. La routine y el issue de latido se crean a mano tras el merge (pasos operativos, fuera de las tareas).

**Tech Stack:** bash + git + jq + gh, GitHub Actions, markdown. Estilo de tests: el de `plugins/exo/scripts/test-*.sh` (PASS/FAIL contados, `set -uo pipefail`, tmpdirs con `trap`).

**Spec:** `docs/superpowers/specs/2026-09-29-upstream-sync-design.md`

**Global Constraints:**
- Tope: máximo 5 portes por PR.
- Watchdog: rojo si el último latido tiene más de 8 días o si el issue no existe.
- Umbral del eval: 0 filas `ya cubierto` cuya verdad sea aplica/parcial, y ≥ 90% de acierto en triage.
- Nombres literales: rama `upstream-sync/<tag>`; título de PR `upstream-sync <tag>`; issue `upstream-sync: estado`; commit `port(upstream#N): <resumen>`.
- Ledger: `upstream_tag: <tag>` en línea propia. Filas `| PR | skill | triage | estado | motivo | hash |`. `triage` ∈ {aplica, parcial, ya cubierto, no aplica, duda}; `estado` ∈ {propuesto, portado, rechazado, pendiente, —}.
- Ninguna ruta de máquina concreta (`/home/<user>`, `/Users/<user>`, `C:\Users\<user>`) en ficheros versionados: lo vigila `scripts/test-rutas-personales.sh`.
- Scripts nuevos con bit de ejecución: lo vigila `scripts/test-exec-bit.sh`.
- La lista de verdad del eval NO entra en el repo.

## Precondición humana (antes de la Ola 2)

- [ ] **P1 — Paul fija la verdad del eval.** Revisa las 11 filas de confianza media de `upstream-sync-gold.md` (scratchpad de la sesión 2026-09-29; se mueve a una ruta fuera del repo que Paul elija). Cada choque de doctrina (#2077, #2078, #2318 en plan, #2319 en orchestrate…) se resuelve como divergencia deliberada o como `aplica`. Las divergencias resultantes son input de la Task 5.

## Olas

- Ola 1, en paralelo: T1, T2, T3, T4. Scripts independientes con `Files` disjuntos.
- Ola 2, en paralelo: T5 (ledger inicial + prompt; consume los nombres de T1-T2 y P1), T6 (wiring de CI + workflow watchdog; consume T1-T4).

### Task 1: check de referencias entre skills

**Files:**
- Create: `scripts/check-skill-refs.sh`
- Test: `scripts/test-check-skill-refs.sh`

**Interfaces:**
- Produces: `scripts/check-skill-refs.sh [<plugin_dir>]` (default `plugins/exo`). Exit 0 si toda referencia `exo:<nombre>` en los `.md` bajo `<plugin_dir>/skills` y `<plugin_dir>/agents` resuelve a `<plugin_dir>/skills/<nombre>/SKILL.md` o a `<plugin_dir>/agents/<nombre>.md`. Exit 1 con una línea por referencia rota: `<fichero>:<línea>: exo:<nombre> no existe`.

**Tests:**
- `refs_validas`: fixture con `skills/plan/SKILL.md` que cita `exo:orchestrate`, y `skills/orchestrate/SKILL.md` existente → exit 0, sin output. Falla si se marca rota una referencia válida.
- `ref_rota`: fixture que cita `exo:planificar` → exit 1 y la línea contiene `exo:planificar no existe` con fichero:línea. Falla si un porte que renombra una skill pasa sin avisar.
- `ref_agente`: cita `exo:executor` con `agents/executor.md` presente → exit 0. Falla si los agentes no se consideran destino válido.
- `repo_real`: sobre `plugins/exo` del repo → exit 0. Falla si el estado actual ya tiene una referencia rota: se arregla en esta tarea, no se silencia.

**Verificación:** `bash scripts/test-check-skill-refs.sh` → todas PASS; `bash scripts/check-skill-refs.sh` → exit 0.

**Review Focus:**
- Referencias en backticks y con puntuación pegada (`exo:plan.`, `` `exo:tdd` ``): se reconocen sin el signo.
- `exo:` dentro de URLs o de rutas (`exo:executor` en frontmatter `subagent_type`) cuenta igual.
- Nombres con guion (`recon-first`).

### Task 2: reconciliación del ledger por evidencia

**Files:**
- Create: `scripts/upstream-reconcile.sh`
- Test: `scripts/test-upstream-reconcile.sh`

**Interfaces:**
- Produces: `scripts/upstream-reconcile.sh <ledger.md> [<rama>]` (rama default `main`). Reescribe in situ cada fila con `estado` = `propuesto` a `portado` si existe, alcanzable desde `<rama>`, un commit cuyo subject empieza por `port(upstream#<PR>):`. Sustituye `hash` por el de ese commit en `<rama>` (tras un squash, el de la rama del PR ya no existe). Imprime por stdout una línea por fila que sigue en `propuesto`: `propuesto-sin-evidencia #<PR> <skill>`. Exit 0 siempre que el ledger sea parseable; exit 2 si no encuentra la línea `upstream_tag:` o la cabecera de la tabla de filas.

**Tests:**
- `propuesto_a_portado`: repo git temporal con commit `port(upstream#1943): ledger con scope de plan` en `main` y fila `| #1943 | orchestrate | aplica | propuesto | … | abc123 |` → la fila queda `portado` con el hash real del commit. Falla si la fila se queda `propuesto` con evidencia presente.
- `sin_evidencia`: la misma fila sin ese commit → sigue `propuesto`, y stdout contiene `propuesto-sin-evidencia #1943 orchestrate`. Falla si se declara portado sin commit (claim antes de evidencia).
- `solo_su_pr`: commit `port(upstream#19):` no promueve la fila `#1943`, ni al revés. Falla si hay match por prefijo numérico.
- `otras_filas_intactas`: filas `rechazado`, `pendiente` y `—` salen byte a byte iguales. Falla si reescribe lo que no le toca.
- `ledger_roto`: fichero sin `upstream_tag:` → exit 2. Falla si un ledger corrupto se procesa en silencio.

**Verificación:** `bash scripts/test-upstream-reconcile.sh` → todas PASS.

**Review Focus:**
- Un PR con varias filas (una por skill) y un solo commit: todas sus filas `propuesto` pasan a `portado`.
- Commit que está en una rama no mergeada pero no en `<rama>`: no cuenta.
- Tablas markdown con espacios irregulares alrededor de `|`.

### Task 3: script del watchdog

**Files:**
- Create: `scripts/upstream-watchdog.sh`
- Test: `scripts/test-upstream-watchdog.sh`

**Interfaces:**
- Produces: `scripts/upstream-watchdog.sh <comentarios.json> [<ahora_iso>] [<max_dias>]`. `<comentarios.json>` es la salida de `gh api repos/<repo>/issues/<n>/comments` (array con `created_at`), o el literal `-` si el issue no existe. `<ahora_iso>` default la fecha actual UTC; `<max_dias>` default 8. Exit 0 y `latido OK: <fecha> (<n> días)` si el último `created_at` tiene ≤ max_dias. Exit 1 con `latido caducado: último <fecha> (<n> días > <max>)`, `sin latidos` (array vacío) o `issue upstream-sync: estado no existe` (`-`).

**Tests:**
- `latido_reciente`: último comentario hace 1 día → exit 0. Falla si da rojo con latido sano.
- `latido_caducado`: hace 9 días con max 8 → exit 1 y `latido caducado`. Falla si una routine muerta pasa en verde.
- `frontera`: exactamente 8 días → exit 0. Falla si el límite es off-by-one.
- `usa_el_ultimo`: array desordenado con uno de hace 20 días y otro de hace 2 → exit 0. Falla si toma el primero y no el más reciente.
- `sin_issue` (`-`) y `sin_latidos` (`[]`) → exit 1 con su mensaje. Falla si la ausencia se lee como éxito.

**Verificación:** `bash scripts/test-upstream-watchdog.sh` → todas PASS.

**Review Focus:**
- `date -d` es GNU: el runner es ubuntu, pero el test tiene que correr igual en el Linux de Paul; nada de flags BSD.
- `created_at` con `Z`.

### Task 4: puntuador del eval

**Files:**
- Create: `scripts/upstream-score.sh`
- Test: `scripts/test-upstream-score.sh`

**Interfaces:**
- Produces: `scripts/upstream-score.sh <verdad.md> <ledger.md> [<umbral_pct>]` (umbral default 90). Empareja filas por (PR, skill); la verdad usa las columnas `PR` `skill exo` `etiqueta` de la tabla de `upstream-sync-gold.md`, y el ledger usa `PR` `skill` `triage`. Acierto: etiqueta igual, o `revertido` en la verdad con fila ausente en el ledger. Imprime `aciertos <a>/<total> (<pct>%)`, `ya-cubierto-falsos <n>` con la lista, `ausentes <n>` (en la verdad y no en el ledger) y `extra <n>` (en el ledger y no en la verdad). Exit 0 si `ya-cubierto-falsos` = 0 y pct ≥ umbral; si no, exit 1.

**Tests:**
- `todo_acierta`: verdad y ledger idénticos (5 filas) → `aciertos 5/5 (100%)`, exit 0.
- `ya_cubierto_falso_tumba`: 1 fila `aplica` en la verdad marcada `ya cubierto` en el ledger, el resto bien (19/20 = 95%) → exit 1 y `ya-cubierto-falsos 1`. Falla si el porcentaje tapa el error que no se puede tolerar.
- `bajo_umbral`: 8/10 sin ya-cubierto falsos → exit 1.
- `ausente_cuenta_como_fallo`: fila de la verdad sin fila en el ledger → no suma acierto, `ausentes 1`. Falla si el bot saca buena nota por omitir.
- `extra_no_puntua`: fila solo en el ledger → `extra 1`, no altera el porcentaje.

**Verificación:** `bash scripts/test-upstream-score.sh` → todas PASS.

**Review Focus:**
- `#2028` (release dev→main) en el ledger no debe emparejar con los PRs miembros: queda como `extra`, y los miembros ausentes cuentan como fallo. Así lo pide la spec (atribuir a miembros).
- Normalizar `#1943` vs `1943` y mayúsculas y espacios en la etiqueta.

### Task 5: ledger inicial y prompt del bot

**Files:**
- Create: `docs/upstream/ledger.md`
- Create: `docs/upstream/sync-prompt.md`

**Interfaces:**
- Consumes: `scripts/upstream-reconcile.sh <ledger.md> [<rama>]` @Task 2
- Consumes: `scripts/check-skill-refs.sh [<plugin_dir>]` @Task 1
- Produces: formato de ledger que parsean `upstream-reconcile.sh` y `upstream-score.sh` (Global Constraints).

**Tests:** `Tests: n/a — prosa; el prompt se prueba con el eval de la primera pasada (paso operativo 4), y el formato del ledger con` `bash scripts/upstream-reconcile.sh docs/upstream/ledger.md` → exit 0, sin filas.

**Verificación:** `bash scripts/upstream-reconcile.sh docs/upstream/ledger.md` → exit 0; `bash scripts/test-rutas-personales.sh` → verde; `bash scripts/test-docs-vivos.sh` → verde.

**Review Focus:**
- El prompt NO contiene ninguna etiqueta esperada ni número de PR de la verdad del eval (contaminación).
- El mapeo cubre todos los ficheros exo con cabecera de atribución (`grep -rln "superpowers 6\." plugins/exo/skills`) y declara los excluidos.
- Cada paso 0-7 de la spec aparece en el prompt con su salida al latido.

**Notas:**
- `ledger.md`: `upstream_tag: v6.1.1`. Mapeo fichero→fichero de la spec (sección "Estado versionado"), verificado contra los nombres reales de `plugins/exo/skills/`. Divergencias: las que salgan de P1 más las ya documentadas con fuente (p. ej. debug "referenciar OK, depender NO"). Tabla de filas vacía, solo cabecera.
- `sync-prompt.md`: los pasos 0-7 de la spec en orden. Incluye literalmente: el tope de 5; los nombres de rama, PR, issue y commit; que la atribución sale de `gh pr list -R obra/superpowers --state merged --search "merged:<desde>..<hasta>" --json number,title,baseRefName,files` descartando PRs de release dev→main; que la tabla `movimiento upstream → fichero:línea exo` es obligatoria en el cuerpo del PR; y que toda salida termina en latido.

### Task 6: wiring en CI y workflow watchdog

**Files:**
- Modify: `.github/workflows/ci.yml`
- Create: `.github/workflows/upstream-watchdog.yml`

**Interfaces:**
- Consumes: `scripts/test-check-skill-refs.sh`, `scripts/check-skill-refs.sh` @Task 1
- Consumes: `scripts/test-upstream-reconcile.sh` @Task 2
- Consumes: `scripts/upstream-watchdog.sh`, `scripts/test-upstream-watchdog.sh` @Task 3
- Consumes: `scripts/test-upstream-score.sh` @Task 4

**Tests:** `Tests: n/a — wiring; los tests son los de T1-T4, y aquí solo se comprueba que CI los corre.`

**Verificación:** `grep -c "test-check-skill-refs\|check-skill-refs.sh\|test-upstream-" .github/workflows/ci.yml` → 5; `bash scripts/test-exec-bit.sh` → verde; `actionlint` (si está instalado) sobre los dos workflows → sin errores. Tras el push, el job `checks estáticos` del PR queda verde con los 5 steps nuevos visibles.

**Review Focus:**
- Watchdog: `schedule` diario más `workflow_dispatch`; `permissions: issues: read`; busca el issue por título exacto (`gh issue list --search "\"upstream-sync: estado\" in:title" --state all`). Si no hay issue, pasa `-` al script y el job queda rojo.
- Los steps nuevos de `ci.yml` van en `static-checks`, con el mismo patrón `run: bash scripts/…` que los existentes.
- El watchdog no debe correr en forks: `if: github.repository == 'pguerrerolinares/exo'`.

## Pasos operativos (tras el merge; Paul + sesión, no executor)

1. Crear el issue `upstream-sync: estado` y fijarlo. El watchdog pasa de rojo a verde con el primer latido.
2. Crear la routine con `/schedule`: semanal, repos adjuntos `pguerrerolinares/exo` y `obra/superpowers`, prompt "Lee y sigue docs/upstream/sync-prompt.md del repo exo."
3. "Run now" de humo con el prompt sustituido por algo trivial: comprobar `gh pr list -R obra/superpowers`, `claude --version` en el PATH y el coste. Si algo falla, se corrige la spec antes de seguir.
4. Primera pasada real (eval): `upstream_tag: v6.1.1`. Después, `scripts/upstream-score.sh <verdad fuera del repo> <ledger de la rama del PR>`. Si hay exit 0, Paul revisa y mergea el PR (cierre de huecos del camino A). Si hay exit 1, no se programa y se revisa el prompt.
