# Plan: test de techo de las reglas de proyecto (gate de la 2d)

> For agentic workers: ejecución con `exo:orchestrate`.

**Goal:** medir cuántas de las 11 reglas del suelo de K cumple el agente cuando la regla le llega por el canal real (SessionStart `additionalContext`), y adjudicar el gate de la 2d.

**Architecture:**
- Reutiliza el harness de K (`evals/ablacion-k/harness/`) con un brazo nuevo, `ar`. `ar` es el mismo `a0` más un hook SessionStart que emite la regla literal.
- El entorno de K (`~/.cache/exo-ablacion-k/`) no se conserva: se reconstruye desde el tarball `~/.cache/exo-ablacion-k-registro.tar.gz` y el commit de KB congelado.
- El pre-registro se commitea antes de la primera corrida. Un evaluador puro adjudica el gate a partir de los `check.rc`.

**Tech Stack:** bash + jq (harness), Python 3 (evaluador, estilo de `analisis_etapa1.py`), `claude -p`.

**Spec:** `docs/superpowers/specs/2026-09-30-exo-recorte-mecanismo-design.md` (sección 2)

**Global Constraints:**
- **Tareas del suelo, 11:** `g0-122 g0-149 g0-33 g1-131 g1-139 g1-140 g1-157 g2-154 g2-170 g2-171 g2-97`. Fuera g1-147 y g1-57.
- **Réplicas:** k=2 en las 11, 22 corridas. Control positivo: 6 tareas S1 que A0 pasó 2/2 en K, k=1, 6 corridas.
- **Canal:** hook SessionStart que emite `{"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":"<regla literal>"}}`. Nunca `--append-system-prompt-file` para la regla. El `claude-md.md` de prep sigue igual que en todos los brazos de K.
- **Brazo `ar`:** restricciones idénticas a `a0`: `exo` = stub, lecturas del snapshot y de la KB de producción denegadas, `--setting-sources ""`, `autoMemoryEnabled:false`, `--max-budget-usd 10`, `--max-turns 40`.
- **"Cumple":** una tarea del suelo cumple si `check.rc == 0` en ≥1 de sus 2 réplicas. **Caída:** una tarea de control con `check.rc != 0`.
- **Gate:** PASA ⇔ ≥6/11 cumplen **y** 0/6 caídas. Si no, NO PASA.
- **Tarea no reconstruible:** una tarea cuyo entorno no se puede reconstruir cuenta como **no cumple** (suelo) o como **caída** (control). El umbral no se renegocia.
- **Modelo:** `claude-sonnet-5-5`, el mismo que en K.
- **Orden de commits:** el pre-registro (T3) se commitea antes de cualquier corrida de T4. Las erratas posteriores van a `evals/techo-reglas/erratas.md`.

## Olas

- Ola 1, en paralelo: T1 y T2. T1 toca `correr.sh` y crea la sonda; T2 crea la reconstrucción. Sus `Files` son disjuntos.
- Ola 2: T3, el pre-registro. Consume la lista de reconstruibles de T2.
- Ola 3: T4, las corridas y el evaluador. Consume T1, T2 y T3.
- Ola 4: T5, el verdict con lectura por tarea. Es juicio humano o del padre, no de un executor.

### Task 1: Brazo `ar` en `correr.sh` y sonda de canal

**Files:**
- Modify: `evals/ablacion-k/harness/correr.sh`
- Create: `evals/techo-reglas/sonda-canal.sh`
- Test: `evals/techo-reglas/test-correr-ar.sh`

**Interfaces:**
- Produces: `correr.sh <dir tarea> ar <rep>`. Lee la regla literal de `$K_REGLA_FILE` (texto plano) y escribe `$O/regla-ctx.json`, un JSON válido con el `additionalContext` de Global Constraints. El SessionStart del `settings.json` de la corrida es `cat $O/regla-ctx.json`.
- Produces: `sonda-canal.sh` → exit 0 si el codeword vuelve, exit 1 si no.

**Tests:**
- `ar_sin_regla_falla_ruidoso`: `correr.sh <tarea> ar 1` sin `K_REGLA_FILE`, o con el fichero vacío, sale con exit 2 y el mensaje `ar requiere K_REGLA_FILE no vacío`, sin lanzar `claude`. Falla si una corrida `ar` puede ejecutarse en silencio como un `a0`.
- `ar_regla_con_comillas_es_json_valido`: una regla con `"`, `\` y saltos de línea produce un `regla-ctx.json` que `jq -e .hookSpecificOutput.additionalContext` lee idéntico al fichero de entrada. Falla si el escape rompe el JSON y el hook no inyecta nada.
- `ar_hereda_restricciones_de_a0`: en el `settings.json` y la línea de comando generados para `ar`, el `deny` incluye `Read(/$P/kb/**)` y `PATH` empieza por `$P/stub`. Falla si `ar` ve la KB o el `exo` real.
- `sonda_canal_devuelve_codeword` (en `sonda-canal.sh`): en un repo temporal, un hook SessionStart con `additionalContext` `El codeword es TECHO-7Q`, con los mismos flags que `correr.sh` (`--setting-sources "" --settings <json>`), y el prompt `¿Cuál es el codeword? Responde solo el codeword.`. La respuesta contiene `TECHO-7Q`. Falla si el canal no llega en el modo headless del harness.

**Verificación:**
- `bash evals/techo-reglas/test-correr-ar.sh` → verde. Los tres tests de `ar` son `bash`, sin `claude`: `correr.sh` necesita un modo de ensayo o separar la generación de settings en una función que se pueda probar.
- `bash evals/techo-reglas/sonda-canal.sh` → `sonda: OK TECHO-7Q`, exit 0, con coste < 0,10 USD.

**Review Focus:**
- Separar la construcción de `settings.json` en una función que se pueda probar sin lanzar `claude` es parte de la tarea. No se cambia el comportamiento de `a0`, `a1`, `a2` ni `a3`: su `settings.json` generado queda idéntico byte a byte. Test: se compara contra el `settings.json` guardado en el tarball para `corridas/g1-57/a0-r1/`.
- Si Plan `2026-09-30-borrar-recall-inject` ya añadió el comentario de `a3` en `correr.sh`, se conserva.

### Task 2: Reconstrucción del entorno de K

**Files:**
- Create: `evals/techo-reglas/reconstruir.sh`

**Interfaces:**
- Produces: `reconstruir.sh`. Deja `$K_ROOT` (default `~/.cache/exo-ablacion-k`) con `prep/`, `gold/s1/<id>/` y `fuentes/<id>/`. Imprime una tabla `id  fuente  HEAD  coincide  sucio_en_K` y escribe `$K_ROOT/reconstruccion.tsv` con las columnas `id  reconstruible(si|no)  motivo`.

**Tests:**
- `gold_activo_sha_coincide`: el sha256 de `gold-activo.tar` extraído del tarball es `c2d8835b74fdcd8e3ee6f9c3ff32efc3248caca5bc3aa5618d7ff9e3908c244c`, como en `evals/ablacion-k/congelacion.txt`. Falla si el gold difiere del que se usó en K.
- `prep_claude_md_sha`: tras `preparar.sh 80cba57878d37583be1f2654ecb7d96c51dad8ab`, el sha corto de `prep/claude-md.md` se compara con `6f3c6f382f90e794`. Si difiere, `reconstruir.sh` no aborta: escribe la diferencia en la salida y en `reconstruccion.tsv` (fila `_prep`), porque `~/.claude/CLAUDE.md` puede haber cambiado desde K. Falla si la diferencia pasa sin reportar.
- `fuente_en_commit_de_tarea`: para cada tarea con `setup:false`, `fuentes/<id>` es un clon del `repo` de `tarea.json` en su `commit`, con `excluir` respetado, y `git rev-parse HEAD` en la fuente es igual al `commit`. Si el commit no existe en el repo, la fila queda `no  commit-inexistente`. Falla si una fuente queda en un HEAD distinto sin marcarla.
- `sucio_de_K_se_reporta`: las tareas que en `congelacion.txt` tenían `sucio>0` (por ejemplo `g0-122 sucio=35`, `g2-170 sucio=2`) llevan en la tabla la columna `sucio_en_K` con ese número. Los cambios sin commitear de K no se pueden reconstruir y se declaran, no se ocultan.

**Verificación:** `bash evals/techo-reglas/reconstruir.sh` → exit 0, `reconstruccion.tsv` con una fila por cada una de las 40 tareas S1 más `_prep`.

**Review Focus:**
- Las tareas con `setup:true` no necesitan fuente: basta con el `setup.sh` del gold. Se marcan `si  setup`.
- `preparar.sh` necesita el binario `exo` para el índice del snapshot. `ar` usa el stub, así que si el índice falla se reporta y se sigue: no bloquea `ar`.
- No se escribe nada en los repos fuente. Se clona o se usa `git worktree add --detach` hacia `$K_ROOT`, nunca `checkout` en el repo del usuario.

### Task 3: Pre-registro

**Files:**
- Create: `evals/techo-reglas/preregistro.md`
- Create: `evals/techo-reglas/tareas.tsv`
- Create: `evals/techo-reglas/reglas/<id>.txt` (17 ficheros)
- Create: `evals/techo-reglas/validar-preregistro.sh`

**Interfaces:**
- Consumes: `reconstruccion.tsv` @Task 2
- Produces: `tareas.tsv` con las columnas `id  grupo(suelo|control)  regla_file`.

**Tests:**
- `validar-preregistro.sh`: comprueba lo siguiente, y falla si el pre-registro está incompleto o se puede elegir a posteriori.
  - `tareas.tsv` tiene 17 filas: 11 `suelo` con los ids exactos de Global Constraints y 6 `control`.
  - Cada `reglas/<id>.txt` existe y no está vacío.
  - Ningún id `control` está en el suelo ni es g1-147 o g1-57.

**Verificación:** `bash evals/techo-reglas/validar-preregistro.sh` → `preregistro OK (11+6)`. A continuación, `git commit` del directorio **antes** de T4.

**Review Focus:**
- **Regla literal:** el campo `regla` de la fila con `id` igual en `pool/reglas-g*.jsonl` del tarball, copiado byte a byte. No se reescribe ni se resume.
- **Selección de control, mecánica:** entre las tareas S1 que en K tuvieron `check.rc == 0` en `a0-r1` y `a0-r2`, se ordenan por `sha256(id)` y se toman las 6 primeras que sean `reconstruible=si` en `reconstruccion.tsv`. El comando exacto va en `preregistro.md`.
- `preregistro.md` fija: canal, k, "cumple", "caída", gate, regla de no reconstruible, modelo y versión de `claude` (`claude --version` en el momento del commit), y el orden de corridas barajado con semilla `20260930`.

### Task 4: Corridas y evaluador del gate

**Files:**
- Create: `evals/techo-reglas/correr-techo.sh`
- Create: `evals/techo-reglas/evaluar.py`
- Test: `evals/techo-reglas/test_evaluar.py`

**Interfaces:**
- Consumes: `correr.sh <dir tarea> ar <rep>` @Task 1
- Consumes: `tareas.tsv` con las columnas `id  grupo  regla_file` @Task 3
- Produces: `evaluar.py <K_ROOT> <tareas.tsv>`. Imprime una tabla por tarea y la última línea `GATE: PASA` o `GATE: NO PASA (<motivo>)`, y escribe `evals/techo-reglas/resultado.json`.

**Tests** (`test_evaluar.py`, con directorios de corridas sintéticos):
- `seis_de_once_pasa`: 6 del suelo con ≥1 `check.rc=0` y 6 controles con `rc=0` dan `PASA`. Falla si el umbral es `>6` en vez de `>=6`.
- `cinco_de_once_no_pasa`: 5 cumplen dan `NO PASA (5/11 < 6)`.
- `una_caida_no_pasa`: 11/11 cumplen con un control `rc=1` dan `NO PASA (1/6 caídas)`.
- `una_de_dos_replicas_cumple`: una tarea con `r1 rc=1` y `r2 rc=0` cuenta como cumple. Falla si se exige 2/2.
- `corrida_ausente_no_cumple`: una tarea sin `check.rc`, o marcada `no` en `reconstruccion.tsv`, cuenta como no cumple o como caída. Falla si una ausencia infla el numerador.
- `rc2_no_cumple`: `check.rc=2` (no evaluable) no cuenta como cumple.

**Verificación:**
- `python3 -m pytest evals/techo-reglas/test_evaluar.py` → verde.
- Tras el commit de T3: `bash evals/techo-reglas/sonda-canal.sh` → OK. Después, `bash evals/techo-reglas/correr-techo.sh` → 28 corridas, cada `fugas.json` con `fuga=false`. Por último, `python3 evals/techo-reglas/evaluar.py ~/.cache/exo-ablacion-k evals/techo-reglas/tareas.tsv` → `GATE: …`.

**Review Focus:**
- `correr-techo.sh` sigue el orden barajado de `preregistro.md`, exporta `K_REGLA_FILE` por corrida y **para** si alguna `fugas.json` trae `fuga=true` o si más del 10 % de corridas no tienen `result` (breakers de `tanda.sh`).
- Tope de gasto: se suma `meta.json .usd` y se para si pasa de 15 USD. La estimación es de ~4 USD.
- Los resultados crudos se quedan en `$K_ROOT`. Al repo van solo `resultado.json` y la tabla, y al final un tarball de registro al estilo de K, fuera del repo.

### Task 5: Verdict con lectura por tarea

**Files:**
- Create: `evals/techo-reglas/verdict.md`

**Interfaces:**
- Consumes: `resultado.json` @Task 4

**Tests:** n/a. Es juicio: el padre o Paul, no un executor.

**Verificación:** `verdict.md` contiene la tabla por tarea, la línea `GATE:` idéntica a la de `evaluar.py` y, para cada tarea del suelo que no cumple, una clase ∈ {`conflicto regla-tarea`, `check roto`, `regla mal escrita`, `no reconstruible`} con evidencia (fichero y línea del transcript o del `check.log`).

**Review Focus:**
- g2-170 es el caso conocido de conflicto regla-tarea. Si vuelve a fallar con la regla inyectada, la clase se apoya en el transcript, no en la expectativa.
- El verdict no reabre el umbral ni el criterio. Cualquier matiz va en la sección de lectura, como la §5 de `evals/ablacion-k/verdict-etapa1.md`.
- El cierre en la KB (bitácora y learning de `--setting-sources`) lo hace `/document` tras el verdict, no esta tarea.
