# Plan: borrar el recall por prompt (`recall-inject.sh`)

> For agentic workers: ejecución con `exo:orchestrate`.

**Goal:** quitar el hook `UserPromptSubmit` y todo lo que existe solo para él, sin dejar referencias vivas a ficheros inexistentes.

**Architecture:** es pura sustracción. Se borra el script, sus tests, sus fixtures y su medidor de latencia. Los consumidores funcionales que quedan (el contrato del engine y el check de `jq` en `doctor`) se reapuntan a los consumidores vivos (`exo-recall.sh`, `subagent-inject.sh`, la prosa de `search`). Los comentarios de los scripts hermanos se reescriben para que se sostengan solos.

**Tech Stack:** bash + jq (plugin), Rust (engine, solo `doctor`), GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-09-30-exo-recorte-mecanismo-design.md` (sección 1)

**Global Constraints:**
- Ninguna referencia viva a un fichero inexistente. Un comentario "igual que recall-inject" se reescribe para que se sostenga solo.
- No se tocan `evals/recall-coste/`, ni las specs, planes y consultas antiguas en `docs/superpowers/`, ni `docs/2026-09-04-revision-critica-externa.md`, porque son históricos.
- El engine sigue: `exo recall` y `exo search` no cambian. `exo-recall.sh` y `subagent-inject.sh` solo cambian en comentarios.
- Bump del plugin a `1.5.7` en `plugins/exo/.claude-plugin/plugin.json` y `.claude-plugin/marketplace.json`.
- Verde = `./scripts/test-plugin.sh` + `cargo test --manifest-path engine/Cargo.toml` + `bash scripts/test-hooks-json.sh` + `bash scripts/test-shellcheck.sh` + `bash scripts/test-plugin-bump.sh`.

**Tras el merge (lo hace el padre, no un executor):** reinstalar el plugin (`/plugin` + `/reload-plugins`) y comprobar en una sesión real que el prompt **no** recibe el bloque `=== Recall exo (automático sobre tu prompt` y que el arranque **sí** recibe el core-index.

## Olas

- Ola 1, en paralelo: T1 y T2. Tienen `Files` disjuntos: T1 toca el plugin y la CI, T2 el engine.
- Ola 2: T3. Barre comentarios y docs, hace el bump de versión y verifica el grep global, que depende de T1 y T2.

### Task 1: Borrar el hook, el script y lo que existe solo para él

**Files:**
- Modify: `plugins/exo/hooks/hooks.json`
- Modify: `plugins/exo/scripts/test-contrato-engine.sh`
- Modify: `scripts/test-contrato-ci.sh`
- Modify: `.github/workflows/ci.yml`
- Modify: `evals/ablacion-k/harness/correr.sh`
- Test: `scripts/test-hooks-json.sh`

Borrados:
- `plugins/exo/scripts/recall-inject.sh`
- `plugins/exo/scripts/test-recall-inject.sh`
- `plugins/exo/scripts/test-recall-inject-golden.sh`
- `plugins/exo/scripts/testdata/golden-recall-inject/`, entero
- `plugins/exo/scripts/recall-latencia.sh`
- `plugins/exo/scripts/test-recall-latencia.sh`

**Interfaces:**
- Produces: `hooks.json` sin la clave `UserPromptSubmit`.
- Produces: `test-contrato-engine.sh` que solo asevera predicados de consumidores vivos.

**Tests:**
- `hooks_sin_userpromptsubmit` (en `scripts/test-hooks-json.sh`): `jq '.hooks | has("UserPromptSubmit")' plugins/exo/hooks/hooks.json` da `false`. Falla si la entrada sobrevive y el hook sigue disparando sobre un script borrado.
- `hooks_apuntan_a_scripts_existentes` (en `scripts/test-hooks-json.sh`; si ya existe un check equivalente, se reutiliza y se nombra en el commit): cada `command` de `hooks.json`, con `${CLAUDE_PLUGIN_ROOT}` sustituido por `plugins/exo`, es un fichero existente y ejecutable. Falla si algún hook apunta a un script inexistente.
- `test-contrato-engine.sh` sigue verde tras el recorte. Se conservan todos los bloques `contrato search: …` y los predicados del envelope de `exo recall` que lean `exo-recall.sh` o `subagent-inject.sh`. Falla si se borra un predicado que un consumidor vivo sí lee.

**Verificación:**
- `bash scripts/test-hooks-json.sh && ./scripts/test-plugin.sh && ./scripts/test-contrato-ci.sh` → las tres salen con 0.
- `ls plugins/exo/scripts | grep -c 'recall-inject\|recall-latencia'` → `0`.

**Review Focus:**
- Predicados de `test-contrato-engine.sh`: para cada uno que se borre, `grep` en `exo-recall.sh` y `subagent-inject.sh` del campo jq que asevera (`.data.notes`, `.data.truncated`, `.data.elapsed_s`, `.data.refresh_s`). Se borra solo si ningún consumidor vivo lo lee.
- `recall-latencia.sh`: se confirma que solo consume eventos `recall-inject-emitted` y `recall-inject-degraded`. Si algo más lo invoca (CI, README, otro script), se para y se reporta.
- El paso de `ci.yml` llamado `Contrato engine↔recall-inject.sh contra fixture propio` se renombra a `Contrato engine↔hooks y prosa contra fixture propio`, sin borrarlo. La cabecera de `test-contrato-ci.sh` y la de `test-contrato-engine.sh` se reescriben con los consumidores reales.
- `correr.sh`: el brazo `a3` queda roto por diseño. Solo se añade una línea de comentario encima del `case`: `# a3 usa recall-inject.sh, borrado tras la campaña K; reproducir a3 desde el commit b94ed74.` No se cambia comportamiento.
- En `testdata/`, se borran solo los ficheros exclusivos de `golden-recall-inject/`. Si otro test usa un fixture hermano, ese se queda.

### Task 2: El check de `jq` de `doctor` nombra a los consumidores vivos

**Files:**
- Modify: `engine/src/doctor.rs`
- Modify: `engine/src/recall.rs`
- Test: `engine/tests/doctor.rs`

**Interfaces:**
- Produces: `check_jq` con el mismo `Estado::Fail` y el nuevo detalle literal `sin jq: exo-recall.sh y subagent-inject.sh no pueden leer el envelope`.

**Tests:**
- `sin_jq_es_fail_porque_los_hooks_del_plugin_lo_exigen` (ya existe; se cambia la aserción): `c.detalle.contains("exo-recall")` y `!c.detalle.contains("recall-inject")`. Falla si el diagnóstico sigue nombrando un script borrado.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml` → verde.

**Review Focus:**
- Antes de escribir el literal, confirmar con `grep -n jq plugins/exo/scripts/subagent-inject.sh` que `subagent-inject.sh` usa jq. Si no lo usa, el literal pasa a ser `sin jq: exo-recall.sh no puede leer el envelope` y la aserción no cambia.
- El doc-comment de `check_jq` (`doctor.rs:616-619`) y el comentario de `recall.rs:611` se reescriben sin nombrar `recall-inject.sh`. En `recall.rs`, el porqué de la ruta emitida se mantiene, apuntando al consumidor vivo.

### Task 3: Barrido de comentarios y docs, bump y grep global

**Files:**
- Modify: `plugins/exo/scripts/search-first.sh`
- Modify: `plugins/exo/scripts/_hook-ms.sh`
- Modify: `plugins/exo/scripts/exo-recall.sh`
- Modify: `plugins/exo/scripts/subagent-inject.sh`
- Modify: `plugins/exo/scripts/kb-precommit.sh`
- Modify: `plugins/exo/scripts/_engine-version.sh`
- Modify: `plugins/exo/scripts/_timeout.sh`
- Modify: `plugins/exo/scripts/test-kb-precommit.sh`
- Modify: `plugins/exo/scripts/test-hook-ms.sh`
- Modify: `plugins/exo/scripts/test-exo-recall.sh`
- Modify: `plugins/exo/scripts/test-exo-index.sh`
- Modify: `scripts/test-shellcheck.sh`
- Modify: `scripts/test-rutas-personales.sh`
- Modify: `README.md`
- Modify: `plugins/exo/README.md`
- Modify: `docs/arquitectura.md`
- Modify: `docs/backlog.md`
- Modify: `.superpowers/fabrica/config.md`
- Modify: `plugins/exo/.claude-plugin/plugin.json`
- Modify: `.claude-plugin/marketplace.json`

**Interfaces:**
- Consumes: `hooks.json` sin la clave `UserPromptSubmit` @Task 1
- Consumes: `check_jq` con el nuevo detalle @Task 2

**Tests:** n/a. Son comentarios y docs; la red es el grep de Verificación y la suite.

**Verificación:**
- `git grep -n 'recall-inject\|recall-latencia' -- . ':!evals/' ':!docs/superpowers/' ':!docs/2026-09-04-revision-critica-externa.md' ':!docs/backlog.md'` → vacío.
- `git grep -n 'UserPromptSubmit' -- plugins/ README.md docs/arquitectura.md` → vacío.
- Suite de Global Constraints entera → verde.

**Review Focus:**
- `search-first.sh:111-112` explica por qué su regex no casa con el pie de `recall-inject.sh`. Si esa exclusión ya no hace falta, el comentario se borra. La regex no se toca en esta tarea: cambiar comportamiento está fuera de alcance.
- `docs/arquitectura.md`: el diagrama (líneas ~50-54 y ~395) quita el nodo `UserPromptSubmit` y el de `recall-inject.sh`. La sección de la línea ~430 se sustituye por un párrafo de 2-3 frases: se borró tras la campaña K porque trajo la nota fuente en 3/40 y costaba +22 % de tokens; la búsqueda queda a demanda (`exo search` + `search-first.sh`). Se enlaza la spec.
- `README.md:67` (diagrama) y `:125` (tabla), `plugins/exo/README.md:68`: se quita la fila y el evento.
- `plugin.json` `description`: se cambia solo si menciona el recall por prompt.
- `docs/backlog.md` queda fuera del grep por ser histórico. Se añade una línea de estado: "recall-inject borrado (spec 2026-09-30-exo-recorte-mecanismo)". No se reescriben las entradas antiguas.
