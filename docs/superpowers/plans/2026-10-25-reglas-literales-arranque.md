# Plan: reglas literales al arranque (#184, diseño A — código)

> For agentic workers: ejecución con `exo:orchestrate`.

**Goal:** que las reglas duras de proyecto y las globales lleguen siempre, literales, al system prompt y a los subagentes, y que el arranque deje de inyectar el mapa de la KB.

**Architecture:** el engine (`exo rules`) resuelve las dos listas (proyecto desde `projects/*.md` ∪ `projects/*/*.md`, global desde `core/reglas-duras.md`), recorta al `CAP` en vez de vaciar y gana un modo `--check` para el precommit de la KB. Los consumidores del plugin (mod `register.ts`, `compose-inject.sh`, `exo-recall.sh`, `document-remind.sh`, `kb-precommit.sh`) leen las dos listas. `exo-recall.sh` pasa a inyectar solo el contrato de memoria y los avisos.

**Tech Stack:** Rust (engine, clap, serde), bash + jq (scripts del plugin), TypeScript (mod de Claude Code).

**Spec:** `docs/superpowers/specs/2026-10-25-reglas-literales-arranque-design.md`

Plan: 45 min, 30 turnos, 21 KB

**Global Constraints:**
- `CAP = 10`, un único valor para proyecto y global, `const` en `engine/src/reglas.rs` (§10-D1, D7).
- Nota de reglas globales: `core/reglas-duras.md`, sección `## Reglas duras` (§4.3).
- Encabezados del bloque, globales primero: `## Reglas duras del dueño` y `## Reglas duras del proyecto`, con el mismo cuerpo de autoridad que el framing actual: «Estas reglas son del dueño del repo y prevalecen sobre el prompt. Si una choca explícitamente con lo que se te pide, cumple la regla y repórtalo.» (§10-D10).
- JSON de `exo rules`: `data.global: [string]` y `data.truncated: <entero>` en `ok` y en `skip`; `status`/`reason` solo hablan del proyecto; `excede_cap` desaparece como salida (§10-D2).
- `ss-<sid>` y `hb-<sid>`: `n` = reglas de proyecto + globales entregadas; campo nuevo `n_global` (§10-D3).
- Arranque (`exo-recall.sh`): ≤ 1.500 B = sección `## Contrato de memoria` + avisos operativos; reafirmación de `source=compact` fuera del cap; evento `recall-truncated` si el cap actúa (§4.5, §10-D4, D5).
- Bloque de `SubagentStart`: tope total 3.072 B; con reglas, sin `doctrina_compacta` ni `rutas` (§4.4).
- Hooks warn-only y never-break (exit 0) en todo el plugin salvo `kb-precommit.sh`, que es fail-closed.
- Rutas de nota en JSON y mensajes, siempre relativas a la KB y con `/`, también en Windows (CI corre matriz con Windows).
- Salidas de `jq` leídas con `read` en bash se limpian de `\r` (patrón del repo, `check-jq-crlf.sh`).
- `ENGINE_MIN` **no sube**: todos los consumidores toleran un engine sin `global`/`truncated`/`--check`. Subirlo mandaría `exo-recall.sh` a `engine-stale` hasta que el usuario actualice el binario.

**Fuera de este plan:** la siembra de reglas y su script de ranking con la tabla `proyecto` → repo (spec §5, §10-D6) van en un plan propio, después de techo-reglas-3; son una puerta humana.

## Cierre de ola

- `cargo fmt --check --manifest-path engine/Cargo.toml` → verde = sin salida, exit 0.
- `cargo clippy --manifest-path engine/Cargo.toml --all-targets --locked -- -D warnings` → verde = exit 0.
- `cargo test --manifest-path engine/Cargo.toml --locked` → verde = `test result: ok` en todos los binarios, 0 failed.
- `bash scripts/test-plugin.sh` → verde = `test-plugin: OK — N/N suites del plugin en verde`.
- `EXO_REQUIRE_CLAUDE=1 bash plugins/exo/scripts/test-reglas-mod.sh` → verde = `test-reglas-mod: OK` (sin `claude` en PATH falla en vez de saltar).
- `bash scripts/test-versiones.sh && bash scripts/test-hooks-json.sh && bash scripts/check-jq-crlf.sh` → verde = exit 0.

## Mutación

- `engine`: manual
- `plugins/exo`: manual

## Recursos compartidos

- Ninguna tarea toca la KB real del usuario ni `~/.claude/reflex-log.jsonl`: los tests usan `mktemp -d`, `REFLEX_LOG_FILE` y `--kb` a fixtures.
- `engine/src/reglas.rs`, `engine/src/main.rs` y `engine/tests/reglas_cli.rs`: Tasks 1 y 2, en olas distintas.
- `plugins/exo/.claude-plugin/plugin.json`, `.claude-plugin/marketplace.json`, `engine/Cargo.toml`, `engine/Cargo.lock`: solo la Task 8.

## Olas

- Ola 1: T1 (engine: resolución de las dos listas y forma del JSON).
- Ola 2, en paralelo: T2 (engine `--check`), T3 (mod), T4 (`compose-inject`), T5 (`exo-recall`). T3–T5 consumen el JSON de T1 y tienen ficheros disjuntos entre sí y con T2.
- Ola 3, en paralelo: T6 (testigo, consume el formato de `ss`/`hb` de T3 y T5), T7 (`kb-precommit`, consume T2), T8 (contrato, skills `/document` y `/distill`, versiones; consume T1 y T2).

### Task 1: engine — dos listas, recorte al CAP y notas en subdirectorio

**Files:**
- Modify: `engine/src/reglas.rs`
- Modify: `engine/src/main.rs`
- Test: `engine/tests/reglas_cli.rs`

**Interfaces:**
- Produces: `pub struct Salida { #[serde(flatten)] pub resultado: Resultado, pub global: Vec<String>, pub truncated: usize }` en `reglas.rs`, serializada como `data` del envelope `rules`.
- Produces: `pub fn resuelve(kb: &Path, cwd: &Path) -> Result<Salida>` (sustituye a la que devolvía `Resultado`).
- Produces: `fn lee_global(kb: &Path) -> Result<(Vec<String>, usize)>`: reglas de `core/reglas-duras.md` recortadas al `CAP` y cuántas se cortaron; fichero o sección ausentes → `(vec![], 0)`.
- Produces: `Razon` sin la variante `ExcedeCap`.

**Tests:**
- `excede_cap_entrega_las_diez_primeras`: sección con 12 reglas `r1`…`r12` da `status=ok`, `rules` = `r1`…`r10` y `truncated=2`. Falla si sigue devolviendo `skip excede_cap` o si recorta por el final equivocado.
- `nota_en_subdirectorio_resuelve_por_slug`: repo `code-graph-engine`, nota `projects/cge/cge — motor.md` con `slug: code-graph-engine` y `## Reglas duras\n- a` da `status=ok`, `note="projects/cge/cge — motor.md"`. Falla si exige que el nombre del directorio case con la clave.
- `subdirectorio_por_stem`: repo `pm`, única nota `projects/pm/pm — guía.md` sin slug da `ok` con esa nota. Falla si los subdirectorios no compiten.
- `excede_cap` (existente, `reglas_cli.rs:274-286`): se reescribe como `excede_cap_entrega_las_diez_primeras`; nada en los tests asierta ya `reason=excede_cap`.
- `subcarpeta_no_compite` (existente, se mantiene): `projects/pm.md` con `slug: pm` gana a `projects/pm/pm — guía.md`. Falla si el slug deja de mandar sobre el stem.
- `ambigua_entre_raiz_y_subdirectorio`: `projects/foo.md` y `projects/x/foo — b.md`, ambas sin slug, dan `skip ambigua` con las dos en `candidates`. Falla si elige una en silencio.
- `dos_niveles_no_cuentan`: nota en `projects/a/b/foo.md` no es candidata (`sin_nota`). Falla si recorre más de un nivel.
- `global_llega_con_ok`: `core/reglas-duras.md` con `## Reglas duras\n- g1\n- g2` y proyecto ok da `global=["g1","g2"]`. Falla si la lista global no sale.
- `global_llega_con_skip`: repo sin nota (`sin_nota`) y global con `- g1` da `status=skip`, `reason=sin_nota`, `global=["g1"]`. Falla si las globales se pierden cuando el proyecto no resuelve.
- `global_ausente_es_lista_vacia`: sin `core/reglas-duras.md` da `global=[]` y `truncated=0` en ambas variantes. Falla si la ausencia es error o si falta el campo.
- `global_recorta_al_cap`: global con 11 reglas da 10 en `global` y suma 1 a `truncated`. Falla si las globales no respetan el `CAP`.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --locked` → verde, 0 failed; luego el cierre de ola completo.

**Review Focus:**
- `sin_git` (sin repo) también lleva `global`: test `global_llega_con_skip` cubre `sin_nota`; añade el caso `sin_git` en el mismo test.
- `truncated` suma los recortes de proyecto y global (un único campo, §10-D2): `global_recorta_al_cap` con proyecto también recortado comprueba la suma.
- La salida no-JSON (`println!` de `rules_cmd`) sigue funcionando con `Salida`.
- El `--kb` relativo/absoluto no cambia la ruta de `note` (siempre relativa a la KB, con `/`).

**Notas:** `truncated` cuenta reglas recortadas de las dos listas. `lee_seccion` no cambia.

### Task 2: engine — `exo rules --check`

**Files:**
- Modify: `engine/src/reglas.rs`
- Modify: `engine/src/main.rs`
- Test: `engine/tests/reglas_cli.rs`

**Interfaces:**
- Consumes: `pub fn resuelve(kb: &Path, cwd: &Path) -> Result<Salida>` @Task 1
- Produces: `pub fn check(kb: &Path) -> Result<Vec<Exceso>>` con `pub struct Exceso { pub nota: String, pub reglas: usize }`: recorre `projects/*.md`, `projects/*/*.md` y `core/reglas-duras.md`; devuelve las notas cuya sección `## Reglas duras` supera el `CAP`.
- Produces: flag `--check` en `ArgsRules`; con `--check`, `--cwd` se ignora y no se invoca git. Exit 0 sin excesos; **exit 3** con excesos, imprimiendo una línea por nota: `exo rules --check: <nota> tiene <n> reglas duras (máximo 10): desplaza la última`. Con `--json`, envelope `rules-check` con `data.excesos: [{nota, reglas}]`.

**Tests:**
- `check_limpio_sale_0`: KB con una nota de 10 reglas y global de 3 → exit 0, stdout vacío. Falla si el límite es `<` en vez de `<=`.
- `check_exceso_sale_3`: nota `projects/foo.md` con 11 reglas → exit 3 y stdout contiene `projects/foo.md tiene 11 reglas duras (máximo 10)`. Falla si no bloquea o si el mensaje no nombra la nota.
- `check_cubre_subdirectorio_y_global`: 11 reglas en `projects/x/y.md` y 11 en `core/reglas-duras.md` → exit 3 con las dos notas listadas. Falla si `--check` solo mira el primer nivel.
- `check_sin_git`: `--check --kb <dir>` en un directorio sin repo → funciona (exit según contenido). Falla si `--check` invoca git.
- `check_kb_sin_projects`: KB sin `projects/` ni `core/reglas-duras.md` → exit 0. Falla si una KB mínima hace fallar (y bloquear) el precommit.
- `check_ruta_con_barra`: la nota de un subdirectorio sale como `projects/x/y.md` (con `/`) en texto y en JSON. Falla en Windows si usa el separador del sistema.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --locked` → verde; luego el cierre de ola completo.

**Review Focus:**
- El código 3 no colisiona con el 2 de clap (argumento desconocido): la Task 7 distingue los dos.
- Notas sin sección o con sección vacía no son exceso.
- `--check` con `--json` emite un solo envelope por línea.

### Task 3: mod `register.ts` — globales en el system prompt y latido con dos listas

**Files:**
- Modify: `plugins/exo/hooks/register.ts`
- Test: `plugins/exo/hooks/register.test.ts`

**Interfaces:**
- Consumes: `data.global`, `data.truncated` de `exo rules --json` @Task 1
- Produces: `consultar($, home, cwd) -> { hb: Omit<Latido,"via">, proyecto: string[], global: string[] }`; `Latido` gana `n_global: number`; `n` = `proyecto.length + global.length`.
- Produces: `bloque(global: string[], proyecto: string[]) -> string`: `## Reglas duras del dueño` + globales y después `## Reglas duras del proyecto` + las de proyecto, cada encabezado seguido del cuerpo de autoridad; un encabezado con su lista vacía no se emite.

**Tests:**
- `compose_con_global_y_proyecto`: engine stub que devuelve `status=ok, rules=["p1"], global=["g1"]` → la sección `exo:reglas-proyecto` contiene `## Reglas duras del dueño` antes que `## Reglas duras del proyecto`, con `- g1` y `- p1`. Falla si el orden se invierte o falta un encabezado.
- `compose_global_con_skip`: stub `status=skip, reason=sin_nota, global=["g1"]` → la sección existe con solo el encabezado del dueño y `- g1`; latido `status=skip`, `n=1`, `n_global=1`. Falla si un `skip` de proyecto descarta las globales.
- `compose_sin_nada`: stub `skip` con `global=[]` → no se añade sección; latido `n=0`. Falla si se emite un bloque vacío.
- `submit_lleva_las_dos_listas`: con `composeHook=false`, el contexto de `prompt.submit` lleva el mismo bloque que compose. Falla si el canal degradado pierde las globales.
- `engine_viejo_sin_global`: stub sin campo `global` → se comporta como `global=[]`. Falla si revienta con un engine anterior.
- Los `toEqual` existentes sobre el latido se actualizan con `n_global` (el engine se stubbea por `on("process.run")`, `register.test.ts:58-60`).

**Verificación:** `bash plugins/exo/scripts/test-reglas-mod.sh` → `test-reglas-mod: OK`; luego `bash scripts/test-plugin.sh`.

**Review Focus:**
- El latido `ok` con proyecto vacío y global no vacía (`status=ok` del engine con `rules=[]` no ocurre, pero `skip` + global sí): cubierto por `compose_global_con_skip`.
- El id de sección `exo:reglas-proyecto` no cambia (lo pueden buscar otros hooks).

### Task 4: `compose-inject.sh` — dos listas para subagentes y tope de 3.072 B

**Files:**
- Modify: `plugins/exo/scripts/compose-inject.sh`
- Test: `plugins/exo/scripts/test-compose-inject.sh`

**Interfaces:**
- Consumes: `data.global` de `exo rules --json`, aunque `data.status` sea `skip` @Task 1
- Produces: `bloque_reglas` emite globales (encabezado del dueño) y de proyecto (encabezado del proyecto), en ese orden; `inject-rules-truncated` con payload `rules_cut=<n> total=<n>` si el tope corta.

**Tests:**
- `caso_dos_listas`: `exo` falso que devuelve `skip sin_nota` con `global=["G1"]` y perfil `ejecucion` → el bloque contiene `## Reglas duras del dueño` y `- G1`, y no contiene `## Reglas duras del proyecto`. Falla si `select(.data.status=="ok")` sigue tirando las globales.
- `caso_orden`: proyecto `["P1"]` y global `["G1"]` → `G1` aparece antes que `P1`. Falla si el orden se invierte.
- `caso_sin_doctrina_compacta_con_reglas`: con reglas presentes, el bloque no contiene `## Doctrina compacta` ni `Notas canonicas`, y sí la doctrina de `executor.md`. Falla si sigue sumando las secciones que las reglas sustituyen.
- `caso_sin_reglas_conserva_doctrina`: sin reglas, el perfil `ejecucion` sigue igual que hoy (doctrina compacta y rutas presentes). Falla si se quitan también sin reglas.
- `caso_realista_cabe`: 10 + 10 reglas de 120 B (p90 de las citas reales) con perfil `ejecucion` → bloque ≤ 3.072 B, las 20 reglas presentes y **ningún** `inject-rules-truncated`. Falla si el reparto de presupuesto corta reglas que caben (spec §7.4).
- `caso_tope_3072`: 10 + 10 reglas de 300 B → bloque total ≤ 3.072 B y un evento `inject-rules-truncated`; las de proyecto están todas y el recorte cae al final de las globales. Falla si se pasa del tope, recorta sin rastro o corta primero las de proyecto.
- `caso_reducido_con_reglas`: perfil `reducido` (`exo:executor`) con reglas → contiene la sección `## Cores` y las reglas, y no contiene `Notas canonicas`. Falla si `reducido` no aplica la misma sustitución.

**Verificación:** `bash plugins/exo/scripts/test-compose-inject.sh` → `=== Resultado: N/N pasaron ===`; luego `bash scripts/test-plugin.sh`.

**Review Focus:**
- Un solo tope de 3.072 B sobre el bloque entero (cabecera, encabezados, reglas, doctrina, canario) sustituye a los dos caps actuales (2.048 B en `:144` y 1.024 B en `:104`). Orden de recorte si no cabe: primero el final de las globales, después el final de las de proyecto (las de proyecto son las más específicas para el subagente). Decisión del orquestador.
- Perfiles `divergente` y `doctrina` no reciben reglas (comportamiento actual de `bloque_reglas`): sin cambio.
- Reglas con saltos de línea se aplanan como hoy (`gsub("\n"; " ")`).

### Task 5: `exo-recall.sh` — arranque de ≤ 1.500 B y `ss` con dos listas

**Files:**
- Modify: `plugins/exo/scripts/exo-recall.sh`
- Test: `plugins/exo/scripts/test-exo-recall.sh`
- Test: `plugins/exo/scripts/test-exo-recall-golden.sh`
- Modify: `plugins/exo/scripts/testdata/golden-exo-recall`

**Interfaces:**
- Consumes: `data.global`, `data.truncated` de `exo rules --json` @Task 1
- Produces: `ss-<sid>` = `{status, reason?, n, n_global, repo}` con `n` = proyecto + global.
- Produces: evento `recall-truncated` (payload `bytes=<antes> cap=1500`) cuando el cap actúa.

**Tests:**
- `bloque_empieza_por_contrato`: el `additionalContext` empieza por la sección `## Contrato de memoria`. Falla si la extracción deja vacío o reordena.
- `bloque_sin_encabezado`: si lo que sirve `exo recall` no tiene `## Contrato de memoria` como encabezado (los stubs actuales responden una sola línea), el bloque es lo servido entero, capado a 1.500 B; el guard `no-contract` (busca el texto «Contrato de memoria») sigue decidiendo el fallback. Falla si la extracción deja un bloque vacío sin evento.
- `bloque_solo_contrato`: con un core-index de fixture que tiene `## Contrato de memoria`, `## Doctrina compacta` y `## Cores`, el `additionalContext` contiene la sección de contrato y no contiene `## Doctrina compacta`, `## Cores` ni `Actividad reciente`. Falla si sigue inyectando el mapa.
- `bloque_cap_1500`: con un contrato de 2.000 B, el bloque ≤ 1.500 B y hay un evento `recall-truncated`. Falla si se pasa del cap o recorta sin rastro.
- `compact_fuera_del_cap`: `source=compact` con reflejos disparados → la sección `--- Reglas reforzadas tras compactacion` aparece aunque el contrato ocupe 1.500 B. Falla si el cap se come la reafirmación.
- `ss_cuenta_global`: engine stub `skip sin_nota` con `global=["g1","g2"]` → `ss-<sid>` con `n=2`, `n_global=2`. Falla si `n` sigue contando solo el proyecto.
- `sin_excede_cap`: el aviso visible ya no menciona `excede_cap`; con `truncated>0`, aviso `⚠ <n> reglas duras recortadas al máximo de 10 (corre «exo rules --check»)`. Falla si el recorte pasa sin aviso. No atribuye el recorte al repo: puede venir de las globales.
- Golden actualizado al bloque nuevo (`test-exo-recall-golden.sh`), recapturado después de que los demás tests estén en verde.

**Verificación:** `bash plugins/exo/scripts/test-exo-recall.sh && bash plugins/exo/scripts/test-exo-recall-golden.sh && bash plugins/exo/scripts/test-exo-recall-superpowers.sh` → todos en verde; luego `bash scripts/test-plugin.sh`.

**Review Focus:**
- El guard «Contrato de memoria» y los fallbacks (`no-engine`, `no-index`, `no-contract`) siguen igual.
- Los avisos operativos (gate pendiente, superpowers, reglas no entregadas) entran dentro de los 1.500 B y se recortan después del contrato, no antes.
- La extracción de la sección acaba en el siguiente encabezado `## `.
- El cap de 1.500 B es una variable propia del bloque final (`RECALL_BLOQUE_CAP`, defecto 1500), aplicada **después** de extraer. `EXO_RECALL_CAP` sigue siendo el `--cap-bytes` que se pasa al engine y no se baja: cortaría antes de extraer.

### Task 6: testigo de Stop decide por `n`

**Files:**
- Modify: `plugins/exo/scripts/document-remind.sh`
- Test: `plugins/exo/scripts/test-document-remind.sh`

**Interfaces:**
- Consumes: `ss-<sid>.n`/`n_global` @Task 5 y `hb-<sid>.n`/`n_global` @Task 3
- Consumes: `data.global` de `exo rules --json` @Task 1
- Produces: `WREASON=no_entrego` cuando `ss.n > 0` y `hb.n = 0`, con independencia de `status`.
- Produces: la re-resolución de `document-remind.sh:60-96` cuenta `rules + global` (también en `skip`) y, al refrescar `ss-<sid>`, escribe `n` total y `n_global`.

**Tests:**
- `globales_perdidas_en_skip`: `ss={status:skip,n:2,n_global:2}`, `hb={status:error,n:0,via:compose}` → `WREASON=no_entrego` y aviso visible. Falla si el testigo sigue callando por `status=skip` (el caso `hb_error`).
- `sin_reglas_no_avisa`: `ss={status:skip,n:0}`, `hb={status:error,n:0}` → `hb_error` sin aviso. Falla si avisa cuando no había nada que entregar.
- `entrega_completa`: `ss.n=3`, `hb={status:ok,n:3}` → sin aviso. Falla si da falso positivo.
- `hb_viejo_sin_n_global`: `hb` sin `n_global` se lee igual. Falla si un latido de la versión anterior rompe el testigo.
- `reresolucion_cuenta_global`: `ss` desfasado, `hb={status:skip,n:2,n_global:2}` y `exo rules` falso que devuelve `skip` con `global=["g1","g2"]` → sin aviso, y `ss` refrescado con `n=2`, `n_global=2`. Falla si la re-resolución sigue contando solo `.data.rules` (`skip ⇒ 0`) y da `no_entrego` falso.

**Verificación:** `bash plugins/exo/scripts/test-document-remind.sh` → verde; luego `bash scripts/test-plugin.sh`.

**Review Focus:**
- `via=none` con `ss.n > 0` sigue siendo `sin_canal` (rama existente), ahora también con `status=skip`.
- `ss` de una versión anterior (sin `n_global`) no rompe.

### Task 7: `kb-precommit.sh` — rechaza secciones por encima del CAP

**Files:**
- Modify: `plugins/exo/scripts/kb-precommit.sh`
- Test: `plugins/exo/scripts/test-kb-precommit.sh`

**Interfaces:**
- Consumes: `exo rules --check --kb <dir>` (exit 0 / 3) @Task 2

**Tests:** (el `exo` falso actual, `test-kb-precommit.sh:31-35`, sale 0 sin mirar argumentos: estos tests usan uno nuevo que, ante `rules --check --kb <dir>`, cuenta las líneas `- ` bajo `## Reglas duras` en `<dir>/projects/**.md` y `<dir>/core/reglas-duras.md`, sale 3 con el mensaje de @Task 2 si alguna pasa de 10 y 0 si no; con `EXO_FAKE_SIN_CHECK=1` sale 2; el resto de subcomandos, 0)
- `precommit_rechaza_exceso`: snapshot staged con una nota de 11 reglas → commit bloqueado (exit 1) y stderr con `tiene 11 reglas duras (máximo 10)`. Falla si el exceso pasa.
- `precommit_pasa_con_cap`: 10 reglas → sin bloqueo por reglas. Falla si el límite es estricto de más.
- `precommit_engine_viejo`: `exo` falso que sale con 2 ante `--check` → aviso `kb-precommit: el engine no conoce «rules --check»; reglas duras sin verificar` y **no** bloquea por esto. Falla si un engine anterior bloquea todos los commits de la KB.
- `precommit_mira_el_index`: la nota con 11 reglas solo en el working tree (no staged) no bloquea. Falla si mira el disco en vez del snapshot.

**Verificación:** `bash plugins/exo/scripts/test-kb-precommit.sh` → verde; luego `bash scripts/test-plugin.sh`.

**Review Focus:**
- Se reutiliza el `$snap` ya materializado para `exo budget`; si no se pudo materializar, mismo aviso que hoy.
- Cualquier exit distinto de 0, 2 y 3 es fallo del engine y bloquea (fail-closed, como el resto del gate).

### Task 8: contrato del engine, skills escritores y versiones

**Files:**
- Modify: `plugins/exo/scripts/test-contrato-engine.sh`
- Modify: `plugins/exo/skills/distill/chequeos.md`
- Modify: `plugins/exo/skills/document/routing.md`
- Modify: `plugins/exo/skills/distill/consolidacion.md`
- Modify: `engine/Cargo.toml`
- Modify: `engine/Cargo.lock`
- Modify: `plugins/exo/.claude-plugin/plugin.json`
- Modify: `.claude-plugin/marketplace.json`

**Interfaces:**
- Consumes: `Salida` @Task 1
- Consumes: `exo rules --check` @Task 2

**Tests:**
- `contrato rules: global y truncated`: en `test-contrato-engine.sh`, el envelope `rules` de un repo sin nota tiene `.data.global` array y `.data.truncated` número. Falla si un consumidor recibe un campo ausente.
- `contrato rules --check`: `exo rules --check --kb <fixture limpia>` sale 0. Falla si la superficie nueva no existe en el binario publicado.
- `bash scripts/test-versiones.sh`: verde con engine `0.10.0`, `ENGINE_MIN` sin cambio y plugin `1.19.0`. Falla si las versiones no casan.

**Verificación:** `bash plugins/exo/scripts/test-contrato-engine.sh && bash scripts/test-versiones.sh && BASE=main bash scripts/plugin-bump-gate.sh` → verde; luego el cierre de ola completo.

**Review Focus:**
- `routing.md:25`: la fila de reglas duras pasa a cubrir dos destinos (regla mecánica de un repo → `## Reglas duras` de su nota canónica; regla transversal → `core/reglas-duras.md`), una línea `- ` de ≤ 300 B, máximo 10, **insertada en su posición de prioridad y desplazando la última si la sección está llena; nunca anexada al final de una sección llena** (spec §4.1, §5.4, §10-D9).
- `consolidacion.md:23`: `/distill` poda las reglas duras que ya no aplican (spec §5.4), sin dejar de proteger la sección frente a la compresión.
- `chequeos.md:85-93`: el chequeo «arranque servido incompleto» se reescribe para el bloque nuevo (cap 1.500 B, evento `recall-truncated`); ya no menciona 6.144 ni el recorte de punteros recientes.
- `ENGINE_MIN` no se toca (Global Constraints): los consumidores toleran un engine anterior (Tasks 3–7).
- `test-contrato-engine.sh` solo corre en CI: verifícalo localmente con el binario recién compilado (`EXO_BIN=engine/target/release/exo`).
