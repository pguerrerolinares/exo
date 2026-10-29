# Plan: refresco de la semilla `engine/kb-template`

> For agentic workers: ejecución con `exo:orchestrate`.

**Goal:** poner al día la doctrina de la semilla que vuelca `exo init` con la de la KB del mantenedor desde el 2026-10-08, saneada y clean-room. Es el prerrequisito de `exo:survey` (#163).

**Architecture:** una nota semilla nueva y cinco deltas fundidos en learnings semilla existentes. Dos gates nuevos en Rust mecanizan la condición de invalidación: saneado y techo con aire por learning. El wiring (`plantilla.rs`, recuentos 17→18 y 16→17, `core-index`, AGENTS.md y docs) va en una sola tarea.

**Tech Stack:** Markdown (notas de KB con frontmatter), Rust (`engine/`, tests de integración en `engine/tests/`).

**Spec:** `n/a — cambio pequeño; diseño aprobado en chat el 2026-10-29 (tabla pieza → destino del consultor). Los valores están copiados abajo, en Global Constraints.`

**Global Constraints:**
- Fuente: la KB del mantenedor (`exo config` da la ruta). Se leen sus `learnings/` y `core/` por título; **nunca** se copia texto literal. Se reescribe en genérico: sin nombres de persona, proyectos, fechas, cifras de campañas, números de issue ni rutas.
- Barrido de saneado (gate, Task 1): regex `(?i)paul|bizkaia|kbx|lighthouses|fable|wisdom|2026-|exo#[0-9]+`, excepto el nombre de fichero `.kbx-ratchet.json`.
- `core/core-index.md` ≤ 5.222 B (`engine/tests/plantilla_presupuesto.rs`). Está en 4.131 B; la línea nueva cuesta ~270 B.
- Cada learning `stable` de la semilla ≤ 10.869 B, porque `exo::presupuesto::tiene_aire(12500, tamaño)` exige `techo*100 >= tamaño*115` (`engine/src/presupuesto.rs:62`). Si un delta no cabe, la pieza se parte o pasa a nota nueva: **nunca** se recorta el delta ni se sube el techo.
- Frontmatter de toda nota semilla: `permalink: "{{KB_NAME}}/learnings/<slug>"`, `title`, `tags`, `tier: stable`, `semilla: true`.
- Los wikilinks van por título exacto con alias de ruta, `[[Título exacto|learnings/slug]]`, igual que la semilla actual. `todos_los_wikilinks_de_la_semilla_resuelven` lo vigila.
- El principio de delegar-decision entra **sin** la etiqueta literal del hook (`[reserva: …]`), que se renombra en #227. Se nombra el hook y se remite al README del plugin.

## Cierre de ola

- `cargo fmt --check --manifest-path engine/Cargo.toml` → sin diff
- `cargo clippy --manifest-path engine/Cargo.toml --all-targets --locked -- -D warnings` → 0 warnings
- `cargo test --manifest-path engine/Cargo.toml --locked` → `test result: ok` en todos los binarios
- `exo budget --kb engine/kb-template` → ninguna nota por encima de su techo
- `bash scripts/test-docs-sin-local.sh` → exit 0
- `bash scripts/test-docs-vivos.sh` → exit 0

## Mutación

- `engine`: manual

## Excluir de la review

- `engine/Cargo.lock`

## Recursos compartidos

ninguno (ningún test toca `~/.exo/config.toml`: `exo init` se ejercita en `TempDir`)

## Olas

- Ola 1, en paralelo: T1, T2, T3, T4 y T5. Sus `Files` son disjuntos. Los gates de T1 se validan con un positivo sintético y no dependen del contenido final. La suite del cierre de ola los corre sobre el resultado de todas.

### Task 1: gates de saneado y de techo por learning

**Files:**
- Create: `engine/tests/plantilla_saneado.rs`
- Modify: `engine/tests/plantilla_presupuesto.rs`

**Interfaces:**
- Produces: `fn rastros(texto: &str) -> Vec<String>` (helper del test: devuelve cada coincidencia del barrido de Global Constraints, excepto `.kbx-ratchet.json`)

**Tests:**
- `rastros_caza_un_positivo_conocido`: `rastros("lo midió Paul el 2026-10-12 en exo#42")` da 3 coincidencias. Falla si la regex es case-sensitive o no cubre fechas o números de issue: el detector debe validarse con un positivo antes de que valga su negativo.
- `rastros_no_cuenta_el_fichero_del_trinquete`: `rastros("sellado en .kbx-ratchet.json")` da `[]`. Falla si la excepción no está y el gate nace rojo sobre `AGENTS.md:116`.
- `la_semilla_no_filtra_rastro_del_mantenedor`: para cada `(ruta, contenido)` de `exo::plantilla::FICHEROS`, `rastros(contenido)` está vacío, y el mensaje lista `ruta: coincidencia`. Falla si una nota semilla arrastra contexto personal.
- `ningun_learning_semilla_pasa_su_techo_con_aire`: cada fichero `learnings/*.md` de `FICHEROS` cumple `exo::presupuesto::tiene_aire(12_500, bytes)`, y el mensaje da ruta y bytes. Falla si un delta fundido deja un learning por encima de 10.869 B.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --locked --test plantilla_saneado --test plantilla_presupuesto` → ok. Después, la suite entera del cierre de ola.

**Review Focus:**
- La regex corre sobre el contenido **antes** de `render` (con `{{KB_NAME}}`), para que el nombre de la KB del test no dispare nada. Lo cubre `la_semilla_no_filtra_rastro_del_mantenedor`.
- Que `2026-` no case con versiones ni con otros números: lo cubre `rastros_caza_un_positivo_conocido`. Un falso positivo en la semilla actual se ve al correr el test sobre ella antes de las otras tareas.

### Task 2: delta en `fallo-silencioso.md`

**Files:**
- Modify: `engine/kb-template/learnings/fallo-silencioso.md`

**Interfaces:**
- Produces: sección sobre diseño de gates con dos apartados nombrados, **«bandas sin salida»** (intersección de reglas sin estado posible, que se nombra con su propio finding) y **«gates que solo vigilan a quien declaró una excepción»** (cubrir también lo no declarado, aunque sea informativo). Más el delta de sistemas: un hook que se desactiva sin avisar ante un input ausente, y un cambio de default es una superficie user-facing.

**Tests:** n/a — contenido. Lo cubren los gates de Task 1, `todos_los_wikilinks_de_la_semilla_resuelven` y la review.

**Verificación:** `wc -c engine/kb-template/learnings/fallo-silencioso.md` ≤ 10869 (objetivo ~7.300 B; hoy 5.777). Después, la suite del cierre de ola.

**Review Focus:**
- `learnings/la-prosa-no-es-enforcement.md:44` remite aquí a «gates que solo vigilan a quien se declaró» y a «reglas cuya intersección no tiene salida». Ambos tienen que existir con esas palabras reconocibles: hoy es una referencia colgante.
- No partas la nota en instrumentos y sistemas: es YAGNI a este tamaño.

### Task 3: delta en `hechos-del-harness-claude-code.md`

**Files:**
- Modify: `engine/kb-template/learnings/hechos-del-harness-claude-code.md`

**Interfaces:**
- Produces: sección «Subagentes y procesos largos», a partir de las notas fuente «Hechos del harness que muerden en producción» y «Hechos del harness — esperar procesos largos y subagentes idle». Solo los hechos que generalizan: la entrega de un subagente llega como mensaje, no como resultado de herramienta; `additionalContext` llega tras la herramienta; un proceso en background se espera por su artefacto o notificación, no por polling; el scratchpad muere con la sesión; la salida de un subagente puede truncarse.

**Tests:** n/a — contenido. Lo cubren los gates de Task 1.

**Verificación:** `wc -c engine/kb-template/learnings/hechos-del-harness-claude-code.md` ≤ 10869 (objetivo ~9.100 B; hoy 6.736). Después, la suite del cierre de ola.

**Review Focus:**
- Cada hecho tiene que ser comprobable contra la versión viva de Claude Code y llevar el comando o el evento que lo muestra, sin la versión exacta de la máquina del mantenedor.
- Fuera los hechos ligados a un entorno concreto: montajes de `/mnt/c`, PRs apilados.

### Task 4: deltas menores en tres learnings

**Files:**
- Modify: `engine/kb-template/learnings/instrumento-validado-antes-de-medir.md`
- Modify: `engine/kb-template/learnings/la-prosa-no-es-enforcement.md`
- Modify: `engine/kb-template/learnings/el-brief-es-el-cuello-de-botella.md`

**Interfaces:**
- Produces, en `instrumento-validado-antes-de-medir.md`: un negativo del harness de eval no se transfiere a producción; el brazo que se mide es producción, y un canario prueba que cada pieza dispara; el gold se valida con material nuevo (~+800 B).
- Produces, en `la-prosa-no-es-enforcement.md`: push > pull. Lo que tiene que cumplirse sin pedirlo va al arranque, corto y literal. Las reglas que obligan a iniciar una acción opcional se pierden entre otras reglas, y las que restringen cómo se hace algo no (~+600 B). Más 1 línea con la ruta `core/reglas-duras.md` como canal de las reglas globales (ver Task 5).
- Produces, en `el-brief-es-el-cuello-de-botella.md`: la reconsulta lleva los datos crudos y la regla de decisión, nunca tu propuesta; las cifras del consultor se recalculan antes de usarlas (~+500 B).

**Tests:** n/a — contenido. Lo cubren los gates de Task 1.

**Verificación:** `wc -c` de los tres ficheros ≤ 10869 cada uno (hoy 7.975, 6.389 y 6.379). Después, la suite del cierre de ola.

**Review Focus:**
- En `instrumento-validado` fuera las cifras de campañas (n, tasas): el principio va sin el caso.
- La línea 44 de `la-prosa-no-es-enforcement.md` no se toca; la repara Task 2.

### Task 5: nota nueva y wiring

**Files:**
- Create: `engine/kb-template/learnings/el-consultor-dictamina-el-humano-firma.md`
- Modify: `engine/kb-template/core/core-index.md`
- Modify: `engine/kb-template/AGENTS.md`
- Modify: `engine/src/plantilla.rs`
- Modify: `engine/src/inicia.rs`
- Modify: `engine/tests/plantilla.rs`
- Modify: `engine/tests/inicia.rs`
- Modify: `docs/arquitectura.md`

**Interfaces:**
- Produces: la entrada `("learnings/el-consultor-dictamina-el-humano-firma.md", include_str!(…))` en `exo::plantilla::FICHEROS`, con `FICHEROS.len() == 18`.
- Produces: la nota nueva (~4.000 B) a partir de «Los gates se adjudican con cita y se diseñan sin bandas muertas», «El humano fuera del critical path, dentro de los gates de percepción» y el principio de «Gate delegar-decision y bucle SDD». Cubre:
  - la adjudicación con cita;
  - las condiciones de legitimidad de un gate;
  - la diferencia entre auditor y consultor;
  - cuándo para el orquestador: consultar, medir, reconsultar, y pre-registrar la invalidación;
  - que al humano se le trae el dictamen y no la elección; nombra el hook `delegar-decision` y remite a `plugins/exo/README.md`.

  Enlaza `[[El padre coordina y valida; el ejecutor implementa|learnings/orquestador-limpio]]` y `[[La claridad del encargo es el cuello de botella, no la capacidad del agente|learnings/el-brief-es-el-cuello-de-botella]]`.
- Produces: 1 línea en «Doctrina compacta» de `core-index` con el mismo formato que las demás. **No** va en las 2 primeras posiciones: los primeros 550 B son los que reciben los subagentes.
- Produces: 2 líneas en AGENTS.md. Las reglas globales viven en `core/reglas-duras.md`, en la sección `## Reglas duras`, como mucho 10 y solo mecánicas. El fichero es opcional: sin él, `exo rules` no da reglas globales.

**Tests:**
- `son_dieciocho_ficheros` (renombra `son_diecisiete_ficheros`): `FICHEROS.len() == 18`. Falla si la nota nueva no está embebida.
- `vuelca_escribe_los_dieciocho_y_no_deja_placeholders` (renombra el de los diecisiete): `escritos.len() == 18`.
- En `engine/tests/inicia.rs`: `data.files == 18` y `data.index.indexed == 17` en los dos sitios que hoy esperan 17 y 16 (líneas ~495, ~497 y ~584). Falla si la nota nueva no se indexa (por frontmatter roto, por ejemplo).
- `todos_los_wikilinks_de_la_semilla_resuelven` (ya existe) sigue verde con los enlaces de la nota nueva.
- `el_core_index_semilla_cabe_con_15_por_ciento_de_aire` (ya existe) sigue verde: core-index ≤ 5.222 B.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --locked` → ok entero. `grep -rnE '\b17 ficheros|diecisiete' engine docs/arquitectura.md` → sin resultados.

**Review Focus:**
- Los comentarios de `engine/src/inicia.rs:95`, `:234` y de `engine/tests/inicia.rs:361-471` citan 17 y 16: hay que actualizarlos (regla de cutover del repo).
- Que la línea de core-index no empuje fuera de los primeros 550 B a una de las dos líneas que reciben los subagentes.
- La nota no trae la etiqueta literal `[reserva: …]` ni el nombre del mantenedor (lo cubre el gate de Task 1).
