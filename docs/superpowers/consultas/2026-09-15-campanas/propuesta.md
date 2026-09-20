# Propuesta de campañas F→K — deuda abierta de exo agrupada por sinergia real

- **Fecha**: 2026-09-15 · **Base**: `docs-backlog-estado-0915` @ `99ddd05` (árbol limpio).
- **Fuentes leídas**: `docs/backlog.md` entero (2.080 líneas, las cuatro secciones), `wisdom-paul/backlog/Backlog — exo.md` y `Backlog — Memoria v2.md`, los planes de B/D/E, `.superpowers/fabrica/config.md` §Roadmap y §2026-09-14, `verdict-decisiones.md` de D+E, y el código donde un item olía a caducado (`doctor.rs`, `buscador.rs`, `walker.rs`, `lint.rs`, `escritor.rs`, `main.rs`, `trinquete.rs`, `lib.rs`, `kb_sintetica.rs`, `hooks.json`, `kb-precommit.sh`, `recall-inject.sh`, `exo-recall.sh`, `reflex-baseline.sh`, `test-hermetico.sh`, `test-shellcheck.sh`, `ci.yml`, `release.yml`, `document/SKILL.md`, `distill/chequeos.md`, README, `arquitectura.md`).
- **Régimen**: directiva «construir antes que medir» (Paul, 2026-08-17). Ninguna campaña de abajo lleva ventana de medición como relleno; donde hay medición es porque el cambio es irreversible (retrieval) o porque el instrumento ya existe y solo se lee después de construir (latencia W11).
- Líneas del backlog citadas como `backlog:N` = `docs/backlog.md:N` en `99ddd05`. `KB-exo:N` = `Backlog — exo.md:N`.

## 1. Resumen

Cinco campañas con plan escribible hoy (F, G, H, I, J) y una sexta (K, proceso residente) que necesita `exo:brainstorm` antes de plan. La lectura previa (F = gates huérfanos + deriva de docs, G = engine, H = decisiones + diagnóstico por fila) se **confirma en F y G**, y se **parte H**: las decisiones de Paul no son campaña (van en el §5 como paquete) y el diagnóstico por fila es la Task 0 de la campaña de retrieval, no una campaña.

| Letra | Nombre | Coste | Bloqueo |
|---|---|---|---|
| **F** | Superficie publicable: docs vivos, gates estáticos que faltan y «genérico» | M | Paul: rename de `lint`, idioma de identificadores, `docs/superpowers/` |
| **G** | Engine: deuda diferida sin cambio de ranking | M-L | Paul: `walk_kb`; Linux solo para verificar `trinquete` contra kbx |
| **H** | Fail-closed: `doctor` y cutover binario↔plugin que no mienten | M | Paul: confirmar que el check de desfase vuelve a estar en alcance; `kb-precommit` fail-closed |
| **I** | Latencia del hook en W11: menos spawns, misma salida | M | Paul: métrica/umbral de reapertura (pre-registro de A) |
| **J** | Retrieval con held-out nuevo (H28, N1, fusión, abstención, `archive/`, int8) | L | **Paul etiqueta gold nuevo** (~2-3 h); G antes |
| K | Proceso residente (esbozo, sin plan) | L | Brainstorming con Paul |

**Orden recomendado y paralelismo** (derivado de la matriz del §3):

- **Ola 1, tres ramas en paralelo: F ∥ G ∥ H.** No comparten fichero salvo `docs/backlog.md` (re-anclar por texto, como en D/E) y `engine/src/main.rs` en zonas disjuntas (G: comandos `write` :832-868; H: aliases :153-315 y `--version`). Orden de merge: H → F → G (el que más toca absorbe el rebase, regla de D/E).
- **Ola 2: I después de H** (los dos reescriben `exo-recall.sh`/`recall-inject.sh`; H añade 20 líneas, I reescribe el parseo — H primero, I rebasa) **∥ J después de G** (los dos tocan `buscador.rs`; G reordena conexiones, J cambia la fusión) y después de que Paul haya etiquetado el gold.
- K cuando exista spec; independiente de todo salvo de I (comparten `recall-inject.sh`).

Por qué F/G/H primero: son las tres que no piden nada caro a Paul, cierran 24 items del backlog entre las tres, y dejan el terreno (gates, engine limpio, doctor honesto) sobre el que I y J pisan.

---

## 2. Campañas

### F — Superficie publicable: docs vivos, gates estáticos que faltan y «genérico»

**Objetivo**: que lo que se declara verdad hoy (los cuatro ficheros `core`) lo compruebe un gate y no una persona, y que el bash y el JSON del plugin que aún no pasan por CI pasen.

**Sinergia**: mismo mecanismo en todos los ítems — un `scripts/test-*.sh` con ciclo rojo-verde local, descubierto por el job `lint`/`test` de `ci.yml` (el patrón que B y E ya sellaron con `test-shellcheck.sh`, `test-versiones.sh`, `test-rutas-personales.sh`). Misma causa raíz en la mitad de docs: «deriva documental que solo se cazó porque alguien la leyó» (`backlog:274-280`). Y el mismo campo semántico en «genérico»: superficie que un tercero ve.

**Items incluidos**:
- `backlog:247-298` «La documentación de referencia contradice el repo» — acción (c): grep de afirmaciones frágiles acotado a los cuatro `core`.
- `backlog:596-635` «Los documentos del repo no llevan `tier`» — acción (c), la misma que arriba (D5=b ya cerró a y b).
- `backlog:1136-1144` «README y `arquitectura.md` §6 citan el 48/55 sin la cifra held-out» — **medio caducado**: README ya no cita 48/55 (grep vacío; lo retiró `c5c5b7f`); quedan `docs/arquitectura.md:201` y `:492`.
- `backlog:649-682` «Rutas personales y `hooks.json` sin validar en CI» — sub-propuesta 2 (schema de `hooks.json`). Verificado: nada valida `hooks.json` (solo lo citan `ci.yml` y `test-exec-bit.sh`).
- `backlog:1148-1163` «El bash inline de `run:` en `.github/workflows/*.yml` no pasa por ningún gate» — confirmado en `scripts/test-shellcheck.sh:9,23`.
- `backlog:732-765` «Un rojo del job `test` no se puede diagnosticar» — lo que queda: patrón para error de COMPILACIÓN (`engine/scripts/test-hermetico.sh`, comentario «Sigue sin haber un patrón…»).
- `backlog:785-819` «Dos endurecimientos del CI…» — lo que queda: `HF_HOME` (grep en `.github/workflows/` vacío; la ruta de caché sigue siendo el default de `hf-hub`, `ci.yml:141`, `release.yml:84`).
- `backlog:1165-1178` «El job `lint` se llama "fmt + clippy"» (`ci.yml:25`) — condicionado a la decisión de branch protection (§5).
- `backlog:300-368` «"exo genérico" sigue siendo el plugin de Paul» — acción (a), solo la mitad `Paul` → «el dueño de la KB» en 5 ficheros de lógica (re-medido hoy: `distill/SKILL.md` ×6, `distill/chequeos.md` ×1, `recall-inject.sh` ×2, `git-add-all-guard.sh` ×1, `kb-precommit.sh` ×1; `plugin.json` es `author.name`, legítimo). La mitad `kb-demo` en tests va a G (es `engine/tests/`).
- `backlog:1225-1248` «Idioma mezclado» — solo la línea en `arquitectura.md` §3.8 sobre identificadores (decisión de Paul, §5; cero código).
- `backlog:1285-1299` «Nombres y ubicaciones» — `docs/superpowers/` (decisión de Paul, §5; si es «se queda», una frase; si es rename, una task más).
- Cierre por evidencia en el sync del backlog de los caducados del §4 (evicción KB, `budget`/`cost`, `--db` en `document`, `reflex-baseline.sh`, C8 en `## Estado`).

**Esbozo de tasks** (9; las 1-7 tienen ficheros disjuntos ⇒ paralelizables, 3 y 5 en secuencia por `ci.yml`):

| # | Task | Ficheros | Hecho cuando |
|---|---|---|---|
| 1 | `arquitectura.md` §6 y :201 llevan la cifra held-out (64/92, Wilson 95 %) y la advertencia de no comparabilidad | `docs/arquitectura.md` | `grep -c "64/92" docs/arquitectura.md ≥ 2` y ninguna mención de 48/55 sin la held-out en la misma sección |
| 2 | Gate `scripts/test-docs-vivos.sh`: sobre los 4 `core` comprueba (a) ausencia de afirmaciones muertas conocidas («Sin CI», «no hay ningún tag», «viven en una rama sin mergear»), (b) que cada subcomando `exo <x>` citado existe en `exo --help` (lista extraída del binario o, sin binario, de `enum Comando` de `main.rs`), (c) que toda cifra `vX.Y.Z` citada existe como tag o coincide con `Cargo.toml`/`plugin.json`, (d) que todo enlace relativo a `docs/`/`evals/`/`scripts/` resuelve a fichero existente | `scripts/test-docs-vivos.sh`, `ci.yml` job `lint` | Ciclo rojo-verde: meter «Sin CI» en `arquitectura.md` ⇒ exit 1 con fichero:línea; quitarlo ⇒ 0. Corre en CI |
| 3 | Gate `scripts/test-hooks-json.sh` con `jq`: eventos ∈ {PreToolUse, SessionStart, Stop, SubagentStart, UserPromptSubmit,…}, cada hook `type=="command"`, el `command` referencia `${CLAUDE_PLUGIN_ROOT}/scripts/<x>` y `<x>` existe y es 100755 en el índice | `scripts/test-hooks-json.sh`, `ci.yml` | Rojo-verde: renombrar un script en `hooks.json` ⇒ exit 1 |
| 4 | Extender `test-shellcheck.sh` a los bloques `run: \|` de `.github/workflows/*.yml` (extracción con `yq` no: sin dependencia nueva — `awk` sobre indentación, o `python3` ya usado como herramienta del ejecutor en E; decidir en plan) | `scripts/test-shellcheck.sh` | Rojo-verde con un `shasum` vs `sha256sum` reintroducido en un `run:` de prueba |
| 5 | `test-hermetico.sh`: patrón para `^error(\[E[0-9]+\])?:` y `could not compile` ⇒ bloque «--- error de compilación ---» en stderr | `engine/scripts/test-hermetico.sh` | Rojo-verde: un test con error de tipo en `engine/tests/smoke.rs` temporal ⇒ el bloque aparece |
| 6 | `HF_HOME` explícito en `ci.yml` y `release.yml`, ruta de caché derivada de la variable | `.github/workflows/{ci,release}.yml` | `Cache restored from key` en los 3 SO antes y después (una corrida de CI, el único gate remoto de F) |
| 7 | Renombrar job `lint` → «checks estáticos» o partirlo — **solo si Paul decide branch protection**; si no, se deja y se anota | `ci.yml` | Nombre del job refleja sus steps; Paul confirma required checks |
| 8 | «Genérico» (a): `Paul` → «el dueño de la KB» en los 5 ficheros de lógica; línea de idioma de identificadores en `arquitectura.md` §3.8; frase sobre `docs/superpowers/` | `plugins/exo/skills/distill/{SKILL,chequeos}.md`, `plugins/exo/scripts/{recall-inject,git-add-all-guard,kb-precommit}.sh`, `docs/arquitectura.md` | `git grep -c Paul -- plugins/exo` = solo `plugin.json:6`; suites `test-recall-inject.sh` y `test-git-add-all-guard.sh` verdes |
| 9 | Sync del backlog: cierra los items de F y los caducados del §4 con cita | `docs/backlog.md` | Ningún item cerrado sin commit citado |

**No entra**: el check de desfase binario↔plugin (H); todo `engine/src` (G); la medición del coste de la inyección (§4); el `tier:` en frontmatter (D5=b lo cerró por convención de ruta). **Coste** M · **Riesgo** bajo (gates estáticos; el único cambio con efecto remoto es `HF_HOME`, y lleva su criterio de caché). **Ficheros calientes**: `scripts/*.sh`, `.github/workflows/ci.yml`, `engine/scripts/test-hermetico.sh`, `docs/arquitectura.md`, `plugins/exo/scripts/{recall-inject,kb-precommit,git-add-all-guard}.sh` (solo comentarios/mensajes), `docs/backlog.md`.

---

### G — Engine: deuda diferida sin cambio de ranking

**Objetivo**: cerrar la deuda del engine que lleva meses en «cuando se toque por otra razón», con el mismo oráculo para todas: la salida es byte-idéntica antes y después (salvo dos cambios declarados: `walk_kb` y el error nuevo de permalink corto).

**Sinergia**: todo vive en `engine/src` + `engine/tests`, todo se juzga con `cargo test --release --locked` más un diff de salida `--json` sobre la KB semilla, y **ninguno toca β, bonus, umbral, troceado ni fusión** — por eso no consume held-out y puede ir antes que J. Módulos disjuntos ⇒ tasks paralelizables.

**Items incluidos**:
- `backlog:524-566` «Techos de escala» — las dos patas vivas: tres aperturas de DB en `busca_hybrid` (`buscador.rs:575` vía `busca`, `:589` vía `busca_vector`, `:605` `abre_db` propio) y un `git log -1` por nota (`indexer.rs:28` `git_epoch_de`, llamado en `:197`).
- `backlog:1046-1057` «El bench sintético es ciego al umbral» — `engine/examples/kb_sintetica.rs:7,192` genera `vector_unitario` pseudoaleatorio; hay que sembrar con vectores reales + ruido para que el arm vector aporte. Es el **oráculo de coste** de la pata anterior: se arregla primero.
- `backlog:821-845` «`walk_kb` frente a `walk_kb_excluyendo`» — Paul lo dejó FUERA de E; hoy `walk_kb` la usan `indexer.rs:159` y también `doctor.rs:312,451` (no citado en el item). Cambio de comportamiento declarado (una `NOTA.MD` pasa a indexarse; `.git/` deja de recorrerse — 276 de 314 `openat` según H29).
- `backlog:878-910` «`budget_prose_drift` tiene dos límites» — **el bloqueador caducó**: pedía «cuando exista el gate de paridad con Go»; los gates corrieron en D y kbx ya no es referencia de nada (`backlog:181`). Se arregla en Rust solo: rechazar captura que no consume la cifra entera (`lint.rs:273,311`).
- `backlog:637-647` `#[allow(clippy::too_many_arguments)]` en `escritor.rs:252` — struct de parámetros para `escribe_nueva`.
- `backlog:708-730` gate M4, vivos #5 (walk de confirmación antes de `--create`, `escritor.rs:240-247` solo mira índice) y #6 (permalink de 2 segmentos crea directorio espurio — confirmado en `main.rs:844-849`: `rsplit_once` sobre «proyecto/slug» da `dir=proyecto`). #9 caduca (§4). #8 es decisión (§5).
- `KB-exo:16` «no hay assert de dimensión/norma tras `embebe_batch`» (`lib.rs:268`); la única guarda está al leer (`vectores.rs:47-56`).
- `backlog:1086-1099` `trinquete::sellos_escapados_de_tier` lee el tier del disco en `--staged` (`trinquete.rs:830-845`). Paso previo: leer `internal/ratchet` de kbx `fe46443` (el checkout vive en la máquina Linux, `git -C ~/Documentos/proyectos/kbx show fe46443:…`) — **única dependencia Linux de G**, y solo para decidir, no para construir.
- `backlog:1310-1322` «`kb-demo` como fixture en 8 ficheros de test» — hoy son **11** (`git grep -l kb-demo -- engine/tests`) más `src/inicia.rs` y `src/lib.rs` en comentarios/plantilla; rename a `kb-test`.
- `backlog:1250-1283` «Relato de campaña en los comentarios» — solo en los módulos que G toca (`buscador.rs`, `escritor.rs`, `indexer.rs`): invariante se queda, relato al verdict con enlace.
- `backlog:243-245` D6 default de `exo search --type` (`main.rs:241`, hoy `Fts`) — una línea + test, **si Paul decide** (§5).

**Esbozo de tasks** (11):

| # | Task | Ficheros | Hecho cuando |
|---|---|---|---|
| 1 | `kb_sintetica`: vectores = muestra de embeddings reales de la KB semilla (o de `evals/e1-read`) + ruido gaussiano σ configurable; `C-noregresión` de `compara.sh` cubre también `s2` | `engine/examples/kb_sintetica.rs`, `evals/recall-coste/harness/{bench,compara}.sh` | Con umbral 0,40 el arm vector devuelve >0 resultados en n174; `bench.sh` corre en W11 y Linux |
| 2 | `busca_hybrid` con una sola conexión: `busca`/`busca_vector` ganan variantes `_con(&Connection)`, las públicas las envuelven | `engine/src/buscador.rs`, tests | `exo search --type hybrid --json` byte-idéntico sobre la KB semilla (diff vacío); s2 n5000 p50 no sube (bench de Task 1) |
| 3 | `git_epoch_de` en batch: un `git log --format='%at' --name-only` por `indexa`, mapa ruta→epoch, fallback por nota solo para las que no aparezcan | `engine/src/indexer.rs`, `engine/src/gitx.rs`, tests | `notas.git_epoch` idéntico antes/después (`sqlite3 … ORDER BY ruta` diff vacío); `exo rebuild` n5000 en W11 baja (bench) |
| 4 | Unificar `walk_kb` sobre `walk_kb_excluyendo` (`es_md`, salta dot-dirs) — cambio declarado | `engine/src/walker.rs`, tests | Test: `NOTA.MD` se indexa; `.git/x.md` no; `doctor` y `indexa` usan la misma función |
| 5 | `budget_prose_drift`: captura que no consume la cifra entera ⇒ se ignora con test («core 1.2345 B» no produce hallazgo) | `engine/src/lint.rs`, `engine/tests/lint_presupuesto.rs` | Test nuevo rojo→verde; los 5 existentes verdes |
| 6 | `escribe_nueva(&NuevaNota{…})`: struct de parámetros, quitar el `allow` | `engine/src/escritor.rs`, `engine/src/main.rs` (:853 y `write_new_cmd`), `engine/tests/escritor.rs` | `grep too_many_arguments engine/src` vacío; clippy `-D warnings` verde |
| 7 | M4 #5 y #6: walk de confirmación (`kb.join(dir)` existe → buscar fichero con mismo slug) antes de crear; permalink con <3 segmentos ⇒ error accionable, no directorio | `engine/src/escritor.rs`, `engine/src/main.rs:837-863`, tests | Dos tests nuevos rojo→verde; sin directorio espurio en tempdir |
| 8 | Assert tras `embebe_batch`: `len()==768` y norma ∈ [0,99, 1,01] por vector, `bail!` fail-loud | `engine/src/lib.rs:268`, test | Test con embedder stub de dimensión errónea falla con mensaje |
| 9 | `trinquete --staged` lee el tier del índice de git (`git show :ruta`) — **tras** leer `internal/ratchet` de kbx y anotar si el heredado era intencional | `engine/src/trinquete.rs`, `engine/tests/ratchet_cli.rs` | Test: `tier:` editado sin `git add` no cambia el veredicto de `--staged` |
| 10 | `kb-demo` → `kb-test` en los 11 tests + comentarios de `src`; relato de campaña de `buscador.rs`/`escritor.rs` al verdict | `engine/tests/*.rs`, `engine/src/{buscador,escritor,inicia,lib}.rs` | `git grep -c kb-demo -- engine` = 0; suite verde |
| 11 | (Gated D6) default `hybrid` + `--min-similarity 0.40` en `ArgsSearch`; sync del backlog | `engine/src/main.rs:241`, `engine/tests/help_producto.rs`, `docs/backlog.md` | `exo search --help` muestra el default nuevo; test de flags |

**No entra**: nada de `buscador.rs` que cambie scores (J); el proceso residente (K); el `doctor.rs` (H); `walk_kb` si Paul mantiene el FUERA. **Coste** M-L (11 tasks, 4 de ellas con cambio de comportamiento declarado). **Riesgo** medio en Tasks 4 y 9 (gates ya demostrados falsables: rehacer su rojo-verde). **Ficheros calientes**: `buscador.rs`, `indexer.rs`, `gitx.rs`, `walker.rs`, `lint.rs`, `escritor.rs`, `lib.rs`, `trinquete.rs`, `main.rs` (:241 y :832-868), `engine/tests/*`, `engine/examples/kb_sintetica.rs`, `evals/recall-coste/harness/`.

---

### H — Fail-closed: `doctor` y cutover binario↔plugin que no mienten

**Objetivo**: que ningún componente de exo degrade «con forma válida»: el hook detecta binario viejo, `doctor` no da `ok` por un `bash.exe` de WSL ni resuelve una versión al azar, y `kb-precommit.sh` no deja pasar un commit sin gate en silencio.

**Sinergia**: **misma causa raíz** — la clase «fallo silencioso: el instrumento no reporta lo que no hizo» que el backlog ya nombra tres veces (`backlog:408-419`, `:440-449`, `:684-694`) — y **mismo gate**: cada check tiene un estado de máquina reproducible en test (binario stub, PATH falso, cache de plugin ficticia) y un veredicto `ok|warn|fail` que hoy sale mal. Además el retiro de los aliases españoles es **la primera ruptura real de compatibilidad binario↔plugin**, así que el check de desfase se construye justo cuando va a hacer falta.

**Items incluidos**:
- `backlog:440-480` «Restricción de orden en el cutover binario↔scripts — nada la aplica» — el check permanente de desfase (install-state de ECC como precedente, `:463-480`). Paul lo dejó **fuera de E** (`config.md:57`); hay que confirmar que vuelve a alcance (§5).
- `KB-exo:44` «Huecos declarados de G5b»: (i) el mismo desfase; (ii) `git_bash` falso `ok` — confirmado: `doctor.rs:568` hace `busca_en_path(&entorno.path, "bash")`, que desde un PATH de PowerShell resuelve `C:\Windows\System32\bash.exe` (WSL) y da `Ok`; (iii) `script_del_plugin` ordena lexicográficamente (`doctor.rs:762`) donde el shim usa `sort -V` — el propio código lo declara deliberado («basta con que alguna resuelva», `:740-743`), así que aquí la task es alinear o documentar, no un bug seguro.
- `backlog:684-694` «`kb-precommit.sh` depende de que `exo` esté instalado — si no, degrada a "commit permitido" en silencio» (`kb-precommit.sh:20`, `exit 0`).
- `backlog:696-706` «Retirar los aliases españoles del CLI» — 10 `alias =` en `main.rs:153-315` y el test `los_flags_espanoles_siguen_parseando_como_alias` (`flags.rs:89`). Es un bump de engine (0.1.0 → 0.2.0) y por tanto exige el check de desfase antes.
- `backlog:370-438` «Bloque de arranque al 96 %» — solo la acción (c) (¿sigue bien el cap de 6.144?): una tabla en el runbook con el tamaño real hoy y la decisión; (b) es editorial de la KB (§4).
- `KB-exo:43` «Verificar `v0.1.0` en la máquina Linux» y `KB-exo:20` «Restos de B1» — **checklist de máquina para Paul**, no tasks de fábrica; entran en el runbook de la release 0.2.0.

**Esbozo de tasks** (8):

| # | Task | Ficheros | Hecho cuando |
|---|---|---|---|
| 1 | Contrato de versión mínima: `plugins/exo/ENGINE_MIN` (una línea, `0.2.0`) + `exo --version` parseable; helper `_engine-version.sh` que compara semver sin spawns extra (bash puro) | `plugins/exo/ENGINE_MIN`, `plugins/exo/scripts/_engine-version.sh`, `scripts/test-versiones.sh` (el gate ya existe: añade «`ENGINE_MIN` ≤ `Cargo.toml`») | Rojo-verde del gate; test unitario del helper con 6 pares de versiones |
| 2 | `exo-recall.sh` y `recall-inject.sh`: si `exo --version` < `ENGINE_MIN` ⇒ evento `recall-fallback reason=engine-stale` y el bloque de fallback lleva una primera línea «engine desactualizado (x < y)» — deja de degradar con forma válida | `plugins/exo/scripts/{exo-recall,recall-inject}.sh`, `test-exo-recall.sh`, `test-recall-inject.sh` | Caso nuevo en cada suite con stub `exo --version` viejo ⇒ evento y línea presentes; camino feliz sin cambios |
| 3 | `exo doctor` check `plugin_compat`: lee `ENGINE_MIN` del plugin instalado (`~/.claude/plugins/cache/exo/exo/<v>/`) y compara con la propia versión; `fail` si el binario es viejo, `warn` si no hay plugin | `engine/src/doctor.rs`, `engine/tests/doctor*.rs` | Test con cache ficticia en tempdir: tres estados |
| 4 | `check_git_bash`: `ok` solo si el `bash` resuelto responde `--version` con «msys»/«mingw» o vive bajo un `Git/` conocido; `System32\bash.exe` ⇒ `warn` «es WSL, no Git Bash» | `engine/src/doctor.rs:559-582`, tests | Test con PATH falso apuntando a un stub que imita WSL ⇒ `warn` |
| 5 | `script_del_plugin`: orden por versión (parseo `x.y.z`, sin crate nueva) y la fila del check dice QUÉ versión resuelve | `engine/src/doctor.rs:746-768`, tests | Test con `1.9.0` y `1.10.0` en cache ⇒ elige `1.10.0` |
| 6 | `kb-precommit.sh` fail-closed: sin `exo` ⇒ `exit 1` con las tres líneas de remedio (instalar release, `EXO_BIN=`, o `git commit --no-verify` consciente) — **si Paul lo decide** (§5); si no, aviso en stdout + evento en `reflex-log` | `plugins/exo/scripts/kb-precommit.sh`, suite nueva `test-kb-precommit.sh` | Caso: `EXO_BIN=/no/existe` ⇒ exit 1 y mensaje; con stub ⇒ camino feliz |
| 7 | Retirar los 10 aliases + su test; `Cargo.toml` 0.2.0; `plugin.json`/`marketplace.json` 1.2.0 con `ENGINE_MIN=0.2.0`; runbook `docs/superpowers/runbooks/2026-09-xx-release-v0.2.0.md` con el orden «binario antes que plugin» y el checklist Linux (`install.sh`, `exo doctor`, repuntar marketplace, renombrar `exo-b1-real`) | `engine/src/main.rs`, `engine/tests/flags.rs`, `engine/Cargo.toml`, `plugins/exo/.claude-plugin/plugin.json`, `.claude-plugin/marketplace.json`, runbook | `exo search --limite 5` ⇒ exit 2 de clap; `test-versiones.sh` verde; el tag lo pone Paul |
| 8 | Sync del backlog + acción (c) del cap (tabla con bytes reales del bloque hoy) | `docs/backlog.md`, runbook | Items H cerrados con commit |

**No entra**: la fusión de scripts o la reducción de spawns (I); reponer-al-sello tipo `repair` de ECC (el backlog ya dice que la mitad valiosa es la detección, `:476-480`); tocar `hooks.json` (F lo valida, I lo cambia si funde). **Coste** M. **Riesgo** medio: toca los dos hooks que corren en cada sesión/prompt (mitigado: `test-contrato-engine.sh` y las suites de E). **Dependencias**: ninguna campaña; Paul en dos decisiones (§5) y el tag de release. **Ficheros calientes**: `engine/src/doctor.rs`, `engine/src/main.rs` (:153-315), `engine/tests/{doctor,doctor_cli,flags}.rs`, `plugins/exo/scripts/{exo-recall,recall-inject,kb-precommit}.sh` y sus tests, `plugin.json`, `marketplace.json`, `scripts/test-versiones.sh`.

---

### I — Latencia del hook en W11: menos spawns, misma salida

**Objetivo**: que el hook de cada prompt en W11 pague el engine (≈1,1 s) y poco más, no ≈1,2 s de spawns de Git Bash — sin cambiar un byte del bloque inyectado.

**Sinergia**: **misma causa medida** — Task 15 de A (`evals/recall-coste/results/w11-2026-09-15.txt`, `backlog:589-594`): ≈20 spawns a 25-60 ms cada uno (`jq -n 1` ≈55 ms). Mismo oráculo para todo: las suites `test-*.sh` de los scripts tocados (salida idéntica) y el harness s6/s7 de A que ya existe para leer el después. Cumple «construir antes que medir»: se construye la reducción y se lee un instrumento que ya está.

**Items incluidos**:
- `backlog:568-594` «El coste del hook completo en Windows no está medido» — lo que queda: el `PreToolUse:Bash` triple (`hooks.json:14-28`), y la conclusión de W11 de que el shell pesa.
- `backlog:968-1000` «Proceso residente…» — **solo el sub-item W11**: `recall-latencia.sh` suma `elapsed_ms+refresh_ms` (`recall-latencia.sh:32`, tiempo interno del engine) y no ve la mitad del coste; falta `hook_ms` de reloj en `recall-inject.sh` (grep de `EPOCHREALTIME`/`hook_ms` vacío hoy). Cambiar la métrica toca el pre-registro de A ⇒ decisión de Paul antes de mirar datos (§5).
- `KB-exo:45` «Olas 2-4 … Windows» — es esto.

**Esbozo de tasks** (6; 1-4 ficheros disjuntos):

| # | Task | Ficheros | Hecho cuando |
|---|---|---|---|
| 1 | `hook_ms` desde `$EPOCHREALTIME` (bash ≥5, sin spawn) en `recall-inject.sh` y `exo-recall.sh`; `recall-latencia.sh` lo usa como métrica si Paul lo firma, si no lo reporta al lado | `recall-inject.sh`, `exo-recall.sh`, `recall-latencia.sh`, sus tests | Payload `emitted` lleva `hook_ms=`; test de `recall-latencia.sh` con fixtures nuevas |
| 2 | `recall-inject.sh`: un solo programa `jq` que emite todos los campos en una línea (`@tsv` → `read -r`), `tr`/`sed` → expansiones bash; misma salida | `recall-inject.sh`, `test-recall-inject.sh` | `grep -c "jq " recall-inject.sh` ≤ 3 (hoy 14); las ~40 aserciones de la suite verdes; bloque byte-idéntico sobre fixture |
| 3 | `exo-recall.sh`: mismo tratamiento; el nombre de KB se lee del envelope de `exo config --json` una sola vez | `exo-recall.sh`, `test-exo-recall.sh` | Suite de E (7 casos) verde; spawns contados con `set -x \| grep -c '^+'` bajan |
| 4 | `PreToolUse:Bash`: pre-filtro bash puro en cada guard (`case "$STDIN" in *git*)`, sin `jq`) antes de parsear; salida nula = mismo exit 0 | `git-c-bash.sh`, `git-add-all-guard.sh`, `verify-before-commit.sh`, `_truncate-payload.sh`, tests | Un `Bash` sin `git` no lanza ningún `jq` (trace); las 3 suites verdes |
| 5 | (Opcional, si 4 no basta) fundir los tres en `bash-guards.sh` con un parseo; `hooks.json` pasa a un comando | `hooks.json`, script nuevo, `test-hooks-json.sh` de F | Mismo veredicto en los casos de las tres suites |
| 6 | Leer el instrumento: s6/s7 del harness de A en W11 y Linux, tabla antes/después en `evals/recall-coste/results/`; sync del backlog | `evals/recall-coste/results/w11-<fecha>.txt`, `docs/backlog.md` | p50 del hook en W11 < 1,5 s (engine ≈1,1 s + shell ≤0,4 s); Linux no empeora |

**No entra**: el daemon (K); cambiar el cap o el contenido del bloque; `exo config` cacheado en disco (si hace falta, es K). **Coste** M. **Riesgo** medio-bajo: reescritura de parseo en scripts con suites densas; contrato con el engine cubierto por `test-contrato-engine.sh`. **Dependencias**: H antes (mismos ficheros). **Ficheros calientes**: `plugins/exo/scripts/{recall-inject,exo-recall,recall-latencia,git-c-bash,git-add-all-guard,verify-before-commit,_truncate-payload}.sh` y sus `test-*.sh`, `hooks.json` (solo Task 5).

---

### J — Retrieval con held-out nuevo

**Objetivo**: decidir de una vez, con un gold nuevo y un solo pre-registro, las cinco preguntas de ranking que la campaña C dejó abiertas — y a partir de ahí fijar D6.

**Sinergia**: **mismo gate caro**: todo lo de aquí cambia qué trozos entran o en qué orden, el held-out de C está consumido (`c-verdict.md` §11) y cada item lo dice en su Acción. Medirlos por separado costaría un gold por item; juntos cuesta uno. Es la única campaña donde la medición es la sustancia, y está justificada por la directiva: el resultado es difícil de revertir (cambia la calidad de cada sesión) y el error sale caro.

**Coste declarado que nadie más paga**: Paul etiqueta un gold nuevo (C fueron 92 queries no nulas en tres estratos + 55 nulas; presupuestar 60-100 + un corpus negativo propio: **2-3 h de Paul**). Sin eso, J no arranca.

**Items incluidos**:
- `backlog:1124-1134` «Sin diagnóstico por fila de qué 6 de las 55 dejaron de acertar» — **Task 0**, in-sample, no consume nada, informa el diseño de las demás.
- `backlog:1002-1014` H28 `distance` L2 vs L2² (`buscador.rs` ~:226): el umbral 0,40 equivale a coseno 0,28.
- `backlog:1016-1044` N1 FTS AND ⇒ `recall --query` es vectorial puro en prosa; OR simple ya descartado (ARREGLA 1 · ROMPE 4). Quedan OR selectivo / NEAR / términos raros.
- `KB-exo:71-76` «El operador de fusión, no sus parámetros» (CombSUM; el barrido histórico se corrió con el FTS conjuntivo) y `backlog:1101-1112` ranking: vector y RRF superan a A0 en hit@1/MRR; normalización FTS por fuerza absoluta.
- `backlog:1114-1122` abstención real ante corpus negativo (54/55 nulas devuelven top-5).
- `backlog:1305-1308` + `KB-exo:63-70` + `Memoria v2:26` downrank de `archive/` — un `WHERE`/penalización; el gold debe incluir queries cuya respuesta vive HOY en `archive/log/` (las 9 SUPERADA) para que la decisión no sea a ciegas.
- `KB-exo:45` cuantizar jina int8 (top-4 solapa 2-4/4 con fp32 ⇒ reindex + held-out) — arm opcional; el ahorro (~360 ms vs ~950 ms de carga) lo canibaliza K si K se hace.
- `backlog:243-245` D6 default de `exo search --type` — se cierra con este verdict (si G ya aplicó `hybrid` como default, J solo confirma o ajusta el umbral).

**Esbozo de tasks** (9):

| # | Task | Ficheros | Hecho cuando |
|---|---|---|---|
| 0 | Diagnóstico por fila de las 55 (hit→miss entre `b0.0-e0.6` y el snapshot de C): tabla fila, causa (fusion-miss / vector-miss / gold rancio) | `evals/retrieval-heldout/diagnostico-55.md` | 6 filas explicadas con permalinks; sin re-etiquetar |
| 1 | Pre-registro: brazos (A0 sellado, H28-corregido+umbral recalibrado, FTS-selectivo, CombSUM, RRF, abstención por gap, archive-downrank, int8), métrica primaria hit@5 + abstención sobre negativos, NETO ≥ 3, N, sha256 del gold — **congelado por commit antes de tocar `engine/src`** | `docs/superpowers/plans/2026-xx-campana-j-preregistro.md` | Commit anterior a cualquier cambio de `buscador.rs` |
| 2 | Gold nuevo (Paul) + corpus negativo propio; `evals/retrieval-heldout/harness` acepta el fichero | `evals/retrieval-heldout/gold-j.jsonl` (privado, sha256 en el pre-registro) | sha256 en el pre-registro coincide |
| 3 | H28: `similitud_desde_l2` correcta + flag temporal `--umbral-cos` para el barrido | `engine/src/buscador.rs`, `vectores.rs`, tests | Ranking idéntico (monótona); umbral equivalente documentado |
| 4 | FTS selectivo: `OR` con mínimo de términos / `NEAR` / términos raros por IDF, como modo `--fts-mode` medible | `engine/src/buscador.rs`, `schema.rs` si hace falta `fts5vocab` | Los tres modos corren en el harness |
| 5 | Operadores de fusión: CombSUM y RRF con parada por acuerdo, detrás de `--fusion` | `engine/src/buscador.rs` (`fusiona`) | Harness los ejercita |
| 6 | Abstención por gap top1-top2 / calibración; `archive/` con penalización configurable | `engine/src/{buscador,recall}.rs` | Corpus negativo: % de abstención por brazo |
| 7 | Corrida y verdict (`c-verdict`-style), decisión por NETO; **solo el ganador se sella**; los flags de barrido se retiran o se documentan | `evals/retrieval-heldout/verdict/j-verdict.md`, `engine/src/main.rs` (sellados) | Verdict firmado; `main.rs:12-25` actualizado con base declarada |
| 8 | Sync del backlog, `arquitectura.md` §6 con la cifra nueva, D6 cerrado | `docs/backlog.md`, `docs/arquitectura.md` | Items J cerrados con cita al verdict |

**No entra**: el daemon (K); la latencia (I); cualquier item que no cambie ranking. **Coste** L (engine + harness + Paul). **Riesgo** alto: es el único cambio del año al motor medido; por eso el pre-registro. **Dependencias**: G merged (Task 2 de G reordena `busca_hybrid`), gold de Paul. **Ficheros calientes**: `engine/src/{buscador,vectores,recall,main}.rs`, `evals/retrieval-heldout/**`.

---

### K — Proceso residente (esbozo; sin plan hasta `exo:brainstorm`)

`backlog:968-1000`, `KB-exo:42`: ~910 ms de ~950 ms son carga del modelo; caliente 27-31 ms. Las preguntas abiertas son de diseño (transporte, ciclo de vida, versión binario/índice, fallback del hook, ~1 GB RSS) y Paul pidió brainstorming — no se planifica aquí. Plan B sin daemon: pesos externos + `commit_from_file` (~400 ms). Toca `engine/src/{lib,recall,main}.rs` + `recall-inject.sh` + `exo-recall.sh` ⇒ después de I. Coste L. Criterio de reapertura ya construido (`recall-latencia.sh`), que I hace honesto en W11.

---

## 3. Matriz de colisión

| Fichero / módulo | F | G | H | I | J | K |
|---|---|---|---|---|---|---|
| `docs/backlog.md` (task de sync) | ● | ● | ● | ● | ● | ● |
| `engine/src/main.rs` | | ● `:241`, `:832-868` | ● `:153-315`, `--version` | | ● sellados `:12-25`, `ArgsSearch` | ● |
| `engine/src/buscador.rs` | | ● conexiones | | | ● fusión/umbral | |
| `engine/src/doctor.rs` | | ● (usa `walk_kb`) | ● | | | |
| `engine/src/{walker,indexer,gitx,lint,escritor,lib,trinquete}.rs` | | ● | | | `lib.rs` si int8 | `lib.rs` |
| `engine/tests/*` | | ● | ● `flags`, `doctor*` | | ● | |
| `engine/examples/kb_sintetica.rs`, `evals/recall-coste/` | | ● | | ● results | | |
| `plugins/exo/scripts/{exo-recall,recall-inject}.sh` | ● comentarios | | ● check versión | ● reescritura parseo | | ● |
| `plugins/exo/scripts/kb-precommit.sh` | ● comentario | | ● fail-closed | | | |
| `plugins/exo/scripts/{git-c-bash,git-add-all-guard,verify-before-commit,_truncate-payload}.sh` | ● comentario en uno | | | ● | | |
| `plugins/exo/hooks/hooks.json` | ● valida | | | ● (Task 5) | | |
| `plugins/exo/skills/distill/*` | ● | | | | | |
| `scripts/*.sh`, `.github/workflows/*` | ● | | ● `test-versiones.sh` | | | |
| `docs/arquitectura.md`, README | ● | | runbook | | ● §6 | |
| `plugin.json`, `marketplace.json`, `Cargo.toml` | | | ● | | | |
| `evals/retrieval-heldout/` | | | | | ● | |

**Derivación**: F, G y H son disjuntas salvo `backlog.md` (re-anclar por texto) y `main.rs` (zonas disjuntas G/H; misma regla que D/E) ⇒ **tres ramas paralelas**. I comparte con H los dos hooks de recall ⇒ **I después de H**. J comparte con G `buscador.rs` ⇒ **J después de G** (y del gold). K después de I. Si Paul mantiene «`walk_kb` FUERA», G y H dejan de compartir `doctor.rs` en absoluto.

---

## 4. Items que no encajan en ninguna campaña

**Decisión de Paul (van al §5, no a la fábrica)**: D6 (`backlog:243-245`) · D-4 permalink (`backlog:181`) · `budget` vs `cost` (`backlog:926-941`) · downrank `archive/` (`backlog:1305`, `KB-exo:63`, `Memoria v2:26`) · idioma de identificadores (`backlog:1225-1248`) · M5b desinstalar basic-memory (`KB-exo:46`, línea roja) · M5a-01 MCP propio (`backlog:943-966`, `KB-exo:77`) · branch protection (`backlog:767-783`) · `docs/superpowers/` (`backlog:1285-1299`) · slug #8 antes de M5b (`backlog:722-726`) · «proceso frente a producto» (`backlog:1180-1223`: es la decisión de fondo; la acción ya se hizo, H6) · cap 6.144 (c) (`backlog:407`).

**Máquina Linux / fuera del repo**: `KB-exo:43` instalar `v0.1.0` y `exo doctor` en Linux · `KB-exo:20` repuntar marketplace y renombrar `exo-b1-real` · `backlog:1301-1303` `crontab -r` y cachés de reflex 0.6.0/0.8.0 (residuos de máquina, no verificables desde el repo). H los mete en el checklist del runbook, no en tasks. H25 (`backlog:1081-1084`) sigue siendo checklist de Paul en GitHub.

**Editorial de la KB, no de exo**: `backlog:370-438` acción (b) (evicción de `core-index`, los 126 B de `Memoria v2:24`) · `KB-exo:47-61` presupuestos (partir `doctrina-agentes`, factorizar) · todo `Memoria v2` salvo `archive/`. La fábrica no edita canon de Paul.

**Medición pura, sin construcción — se difiere por directiva**: `backlog:482-520` «El bucle de coste de la inyección está a un `join` de distancia». Es un harness `reflex-log × transcripts` para saber si la inyección paga su prefijo cacheado. No construye nada y solo importa si Paul quiere cuestionar la existencia de la inyección (§5, última decisión). Si la respuesta es sí, es una campaña S propia (harness en `evals/`, pre-registro, exige `reflex-log.jsonl` presente), no relleno de otra.

**No-deuda**: `backlog:1324-1347` sinergias sin dueño (delegación de I/O, evals fuera de exo) — el propio item dice «no como deuda» · `KB-exo:17-19` empirica (prior art del Frente 1, sonda al mantenedor) · `KB-exo:79` A1 cerrado sin validar · `KB-exo:45` «validez temporal» (sin definición operativa en ningún fichero leído; no se puede planificar).

**Caducados o mal registrados (con evidencia de código)** — F los cierra o corrige en su sync:

| Item | Qué dice el backlog | Qué hay hoy |
|---|---|---|
| `backlog:912-924` evicción de la KB descalibrada, «rehacer el censo con `exo budget`» | abierto | **Hecho el 2026-09-10**: `Memoria v2:18` «La lista de partida la midió `exo budget` … eran 20, no 19», commit `efd9abc`. Cerrar con esa cita |
| `backlog:926-941` `exo budget` «planeado» en `arquitectura.md:486` e `instalacion.md:119-120` | abierto | `grep budget docs/arquitectura.md` vacío; `instalacion.md:34` lo lista como verbo existente; `Comando::Budget` en `main.rs:68` desde G4b. La colisión se resolvió de hecho (`budget` = bytes); queda solo el nombre de un verbo de coste hipotético (§5) |
| `backlog:729-730` M4 #9 «`document/SKILL.md` omite `--db` … fallan con error de clap» | abierto | `ArgsWriteAppend.db: Option<PathBuf>` (`main.rs:172-176`), resuelve por config desde M5a-02; `SKILL.md:55,57` sin `--db` son correctos. Caducado |
| `backlog:1136-1144` «README y `arquitectura.md` §6 citan el 48/55» | abierto | README no contiene 48/55 ni hit@5 (grep vacío; B `c5c5b7f`). Solo `arquitectura.md:201,492`. Acotar |
| `backlog:1302` «`reflex-baseline.sh` traga errores de `jq` con `2>/dev/null`» | abierto | El script no tiene `2>/dev/null` sobre salida de `jq`; `:12-14` valida cada línea con `jq -e .` y lista las inválidas por stderr. Caducado |
| `backlog:1310-1322` «`kb-demo` en 8 ficheros de test» | abierto, 8 | 11 ficheros (`git grep -l kb-demo -- engine/tests`) + `src/inicia.rs`, `src/lib.rs`. Abierto, conteo viejo |
| `backlog:173` `## Estado` «Pendientes: C8 (M3+M1b, cutover de skills)» | pendiente | Hecho en el repo: 9 skills + `agents/executor.md` en `plugins/exo/`, marketplace `exo` (`.claude-plugin/marketplace.json`), fusión G2. Lo que queda es de máquina (`KB-exo:20`). Mal registrado |
| `backlog:878-910` `budget_prose_drift` «para cuando exista el gate de paridad con Go» | bloqueado | Los gates corrieron en D y kbx dejó de ser dependencia (`backlog:181`). Bloqueador caducado; el fix es de Rust solo (G Task 5) |
| `backlog:243` cita `main.rs:215` para el default de `--type` | | Hoy `main.rs:241` (`default_value_t = TipoBusqueda::Fts`) |
| `backlog:821-845` `walk_kb` «la que usa `indexer::indexa`» | | También `doctor.rs:312,451`: unificar afecta a `doctor` |
| `KB-exo:44` `script_del_plugin` ordena lexicográficamente | hueco | Cierto (`doctor.rs:762`), pero el código lo declara deliberado (`:740-743`): es alinear-o-documentar, no bug |

---

## 5. Paquete de decisiones para Paul

| # | Decisión | Opciones | Recomendación | Desbloquea |
|---|---|---|---|---|
| 1 | **D6** default de `exo search --type` (`main.rs:241`) | a) sigue `fts` · b) `hybrid` + `--min-similarity 0.40` (el modo medido) | **b**. El held-out ya lo decidió: A0 gana a FTS en 41 filas y FTS a A0 en 0 (`c-verdict.md`). El coste (≈1 s de modelo) es el que el hook ya paga en cada prompt. J podrá ajustar el umbral, no el tipo | G Task 11 |
| 2 | **D-4** permalink de `rotate`: `nombre_kb()` vs `"wisdom-paul/"` | a) literal · b) `nombre_kb()` (ya en código) | **b**; cerrar el item con el verdict de D+E | Cierra `backlog:181` |
| 3 | `budget` vs `cost` | a) `exo cost` verbo · b) vive en `evals/` · c) nada hasta que exista | **b/c**: `budget`=bytes ya está sellado; un cruce reflex-log×transcripts es medición, no producto. Solo si se aprueba la decisión 12 | Cierra `backlog:926-941` |
| 4 | Downrank de `archive/` | a) nada · b) excluir · c) penalizar, medido en J | **c**: el 33 % de punteros del día 1 vino de archivo, pero 9 filas SUPERADA viven allí. Se decide con gold que incluya esas queries | J Task 6 |
| 5 | Idioma de identificadores de código | a) español (estado real, 26 módulos) · b) inglés en módulos nuevos · c) rename masivo | **a, escrito en una línea** en §3.8: identificadores en español, claves JSON/flags en inglés (D8/D9). Cero código | F Task 8 |
| 6 | **M5b** desinstalar basic-memory (línea roja) | sí / no / cuándo | **Sí, ya**: C10 está desbloqueado desde el 08-26, cero código lo lee, `exo init --from-basic-memory` sobrevive. Precondición: el checklist C10 del plan de cierre (`/document` y `/distill` end-to-end sin MCP). Acción tuya, no de la fábrica | Cierra C9/C10 del `## Estado` |
| 7 | **M5a-01** MCP propio | a) construir (stateless, spec 07-28) · b) diferir hasta echarlo de menos · c) cerrar «no se construye» | **c**: el hot-path es CLI, un MCP añade superficie y prefijo cacheado. Si M5b pasa sin él, no hace falta | Cierra `backlog:943-966`, `KB-exo:77` |
| 8 | Branch protection en `main` | a) no · b) required checks `lint`, `msrv`, `test×3`, `plugin-tests×3`, `install×3` | **b**: los 10 últimos merges fueron PR (#13-#22) y el CI lleva verde; hoy es notificación, no gate. Antes: renombrar el job `lint` (F Task 7) | F Task 7 y cierra `backlog:767-783`, `:1165-1178` |
| 9 | `walk_kb` unificado sobre `walk_kb_excluyendo` | a) FUERA (como en E) · b) unificar, `rebuild` recomendado | **b**: hoy dos semánticas en el mismo módulo y `doctor` usa la vieja; `.git/` come 276/314 `openat`. Cambio declarado en `arquitectura.md` | G Task 4 |
| 10 | Check de desfase binario↔plugin en `doctor` + hooks | a) sigue fuera · b) entra en H | **b**: el retiro de aliases es la primera ruptura real; sin el check, un plugin nuevo contra binario viejo sirve fallback con forma válida (`backlog:445-449`) | H Tasks 1-3 |
| 11 | `kb-precommit.sh` sin `exo` | a) `exit 0` + aviso (hoy) · b) aviso ruidoso + evento · c) `exit 1` fail-closed | **c**: el hook se instala a propósito y `doctor` ya diagnostica; un commit sin gate «permitido en silencio» es el patrón que H existe para matar. Escape consciente: `--no-verify` | H Task 6 |
| 12 | Métrica de reapertura del daemon | a) `elapsed_ms+refresh_ms` (pre-registro de A) · b) `hook_ms` de reloj, mismo umbral 1.500 ms p95 por SO | **b**: en W11 (a) no dispara nunca aunque cada prompt pague 2,3 s. Decidirlo antes de mirar la ventana | I Task 1 |
| 13 | Gold nuevo para J | cuándo, cuántas (60-100 + negativos) | Tras G; **2-3 h tuyas**; incluir queries cuya respuesta vive en `archive/log/` | J |
| 14 | `docs/superpowers/` | a) se queda + una frase · b) rename (`docs/fabrica/`) | **a**: 17 referencias desde los `core` y cientos desde docs fechadas que no se editan; el nombre cuesta una frase, el rename rompe citas | F Task 8 |
| 15 | Slug (#8 del gate M4): 19/127 divergen de basic-memory | a) exo canónico · b) imitar bm | **a**, por escrito, antes de M5b: las bitácoras rotadas ya usan `_` | Cierra `backlog:722-726` |
| 16 | Cap de 6.144 B del bloque de arranque | a) subir · b) mantener + evicción editorial de `core-index` (126 B) | **b**: la doctrina pide 15 % de aire al sellar, no un cap que respire; es editorial tuyo | Cierra `backlog:407` (c) |
| 17 | ¿Medir si la inyección paga su prefijo? (`backlog:482-520`) | a) no, por ahora · b) campaña S de harness | **a** salvo que quieras cuestionar la inyección; la cache está al 97,7 %, no hay margen en cachear más | Nada hoy |

---

## 6. Decisiones de Paul (2026-09-15)

Aceptadas, sobre las recomendaciones de la tabla de §5:

- **#1 (D6, default de `exo search --type`)**: **aceptada la recomendación
  b** — `hybrid` + `--min-similarity 0.40`. Entra en G (Task 11 del esbozo
  de G).
- **#5 (idioma de identificadores de código)**: **aceptada la recomendación
  a** — español, una línea en `arquitectura.md` §3.8: identificadores en
  español, claves JSON/flags en inglés (D8/D9). Cero código. Entra en F
  (Task 8).
- **#8 (branch protection)**: **aceptada la recomendación b, en dos pasos**
  — primero F renombra el job `lint` → algo que refleje sus steps (Task 7
  de F), y branch protection con los required checks se activa DESPUÉS,
  fuera de la fábrica (acción de Paul en GitHub).
- **#9 (`walk_kb` unificado)**: **aceptada la recomendación b** — unificar
  sobre `walk_kb_excluyendo`, cambio de comportamiento declarado
  (`.git/` deja de recorrerse, `NOTA.MD` empieza a indexarse). Entra en G
  (Task 4 del esbozo de G).
- **#10 (check de desfase binario↔plugin en `doctor` + hooks)**: **aceptada
  la recomendación b** — entra en H. Deroga la exclusión que la campaña E
  había fijado («check de desfase en `doctor` FUERA»).
- **#11 (`kb-precommit.sh` sin `exo`)**: **aceptada la recomendación c** —
  fail-closed, `exit 1`, con escape consciente documentado
  (`git commit --no-verify`). Entra en H.
- **#14 (`docs/superpowers/`)**: **aceptada la recomendación a** — se queda
  con su nombre actual; una frase en `arquitectura.md` explicándolo (17
  referencias desde los `core`, el rename rompería citas sin aportar
  nada). Entra en F (Task 8).
- **Retirar los aliases españoles del CLI → `engine` 0.2.0**: aceptada.
  Entra en H, y es la primera ruptura real de compatibilidad
  binario↔plugin — la razón por la que el check de desfase (decisión #10)
  se construye justo en esta campaña, no después.

El resto de la tabla de §5 (decisiones #2, #3, #4, #6, #7, #12, #13, #15,
#16, #17) **sigue sin decidir** a fecha de este commit — no se asume
ninguna recomendación de esas filas por el hecho de estar escritas aquí.

## 7. Decisiones de Paul (2026-09-19)

Tomadas en sesión, una a una, sobre la tabla de §5. Con esto el paquete queda
**vacío**: ninguna fila de §5 sigue sin decidir.

- **#2 (D-4, prefijo del permalink de `rotate`)**: **b** — `nombre_kb()`, lo
  que ya hay en código. Se cierra el item con cita a este párrafo.
- **#3 (`budget` vs `cost`)**: **c** — nada hasta que exista un `cost`. Con
  #17 = a no hay nada que nombrar; `budget` = bytes queda sellado.
- **#4 (downrank de `archive/`)**: **c** — penalizar, no excluir, como brazo
  medido de J. El gold de J incluye queries cuya respuesta vive en
  `archive/log/`. Sin gold no se toca.
- **#6 (M5b, desinstalar basic-memory)**: **sí, tras el checklist C10**. La
  fábrica prepara y corre en seco las comprobaciones automatizables (runbook,
  campaña L); la desinstalación la ejecuta Paul (línea roja).
- **#7 (M5a-01, MCP propio)**: **c** — no se construye. Se reabre solo si
  alguien lo echa de menos con un caso concreto.
- **#8 (branch protection)**: **ejecutada el 2026-09-19** por el orquestador
  con autorización explícita de Paul en sesión: 12 required checks (los de
  F Task 7, verificados contra los check-runs de `5efe812`), `strict: false`,
  sin review obligatoria (autor único), `enforce_admins: false`,
  force-push y borrado de `main` prohibidos.
- **#12 (métrica de reapertura del daemon)**: **b** — `hook_ms` de reloj,
  umbral 1.500 ms p95 por SO. Enmienda el pre-registro de A por decisión
  escrita, **antes** de mirar ninguna ventana con `hook_ms`. Entra en I Task 1.
- **#13 (gold nuevo para J)**: **esta semana, en paralelo** a la fábrica. La
  fase 1 de J entrega a Paul un kit de etiquetado.
- **#15 (slug)**: **a** — el slug de exo es canónico; la divergencia 19/127
  con basic-memory queda aceptada por escrito antes de M5b.
- **#16 (cap de 6.144 B del bloque de arranque)**: **b** — se mantiene;
  evicción editorial de entradas muertas de `core-index` (es índice: no se
  comprime) hasta ≥15% de aire. Trabajo de Paul en la KB, no de la fábrica.
- **#17 (medir si la inyección paga su prefijo)**: **a** — no, por ahora.
- **`v0.2.0`**: se tagea **después de mergear la campaña L**, que arregla la
  regresión de `exo search` (default `hybrid`, de G) contra una DB sin tabla
  `vectores`. I no bloquea la release (puede ir en 0.2.1).
- **Campañas**: la siguiente fábrica ejecuta **I ∥ L** en dos lanes y **J
  fase 1** (T0 diagnóstico + pre-registro borrador + kit de gold) at-risk.
  K (daemon) sigue fuera: pide brainstorm antes de plan.
- **Campaña J, decisiones del plan de fase 1**
  (`docs/superpowers/plans/2026-09-19-campana-j-fase1-diagnostico-y-preregistro.md`),
  firmadas el mismo día, todas con la recomendación del plan: D-J1 = (b)
  agente pre-etiqueta `prompt`/`hard`, Paul escribe `keyword`/`archive`/
  `negativo` y revisa (≈3,5–4 h, no las 2–3 h de §J, que eran optimistas) ·
  D-J2 = 113 · D-J3 = sí, verificación adversarial del gold · **D-J4 =
  NETO ≥ 4** · D-J5 = (a) H28 solo nombre y doc · D-J6 = int8 fuera · D-J7 =
  0,90 · D-J8 = 0,25 · D-J9 = tal cual · D-J10 = lenient decide.
- **D-J1/D-J2 REVOCADAS el mismo día** por Paul: «ya ni leo la kb, todo se
  maneja a través de los agentes; ya no me importa que sea legible para mí,
  sino para los propios agentes». El gold de J pasa a ser **100 % agéntico,
  0 h de Paul**: queries reales de agentes minadas de los transcripts
  (estrato `agent-search`, sustituye a `keyword`; minado autorizado por
  Paul), jueces a ciegas **fable + Kimi (Moonshot)** —otra familia de modelo,
  para romper la correlación de errores Claude–Claude— y un suelo de acuerdo
  pre-registrado por debajo del cual J para. **Paul autoriza explícitamente
  enviar trozos de la KB `wisdom-paul` y queries de agentes a la API de
  Moonshot** («no me importa mandar a kimi, continua por ahí»). Por la misma
  tesis, la evicción editorial de `core-index` (#16) la hace un agente, no
  Paul.
