# Plan: `exo hook` — subproyecto B (Codex)

> For agentic workers: ejecución con `exo:orchestrate`.

**Goal:** que exo funcione solo con Codex (memoria, reglas duras, reflejos, índice y agents), migrando a Rust `exo-recall.sh` y `exo-index.sh` en los dos harness y moviendo todo el estado a `~/.exo/state/`.

**Architecture:** el módulo `engine/src/hook/` gana el harness `Codex` (adapter `codex.rs`), los eventos `SessionStart`/`Stop`/`UserPromptSubmit`/`PostCompact`, la decisión `Avisa` y guards nuevos (`Index`, recall partido en sub-guards, reglas, marca de compactación). Un módulo `state` de la lib resuelve las rutas de estado, que bash replica en `_state-dir.sh` y un test de acuerdo vigila. `exo setup codex` instala los roles TOML. Un gate baseline/divergencias sustituye a las suites bash de recall.

**Tech Stack:** Rust 1.95 (MSRV), edition 2024, clap 4, serde_json, toml (ya en el engine), regex 1, tempfile y jsonschema (dev); bash + jq en scripts y smokes; TS (`register.ts`) solo cambia una ruta.

**Spec:** `docs/superpowers/specs/2026-11-03-exo-hook-codex-design.md` (delta sobre `docs/superpowers/specs/2026-11-02-exo-hook-multi-harness-design.md`).

**Global Constraints:**
- `exo hook` sale con 0 salvo un `--min` desconocido para clap. Todo degrada a `Allow`/fallback y deja rastro en el reflex-log (spec §6.1). Un reflejo nunca rompe lo que instrumenta.
- **El entorno se lee una sola vez**, en `Entorno::desde_env()`, que solo llama el runner. Guards, journal y reflex reciben `&Entorno`. **Ningún test Rust usa `set_var`**: construyen el `Entorno` con rutas de `tempfile`.
- Precedencia de rutas por fichero: variable propia (`REFLEX_LOG_FILE`, `EXO_INDEX_LOG`, `REFLEX_CANARY_FILE`) > `$EXO_STATE_DIR/<x>` > `~/.exo/state/<x>`. Nombres: `reflex-log.jsonl`, `exo-rules/`, `exo-index.log`, `reflex-inject-canary` (spec §4.1).
- Esquema del reflex-log: `{ts, reflex, session_id, agent_id, agent_type, tool, payload, harness}`; payload ≤ 2000 caracteres; `harness` ausente = `claude` (spec §4.8).
- Los ids de reflejo, los `reason` y los textos que ve el agente se copian literales de `exo-recall.sh`, `exo-index.sh` y `register.ts`. Ids nuevos, exactos: `state-migrate-failed`, `gate-timeout`, `context-spill-risk`, `project-rules-emitted` (spec §4.8).
- Topes, de la spec: contrato `cap-bytes 6144`, `limit 10`; bloque final `1500 B`; reglas, techo de 10; reafirmación, últimas 2000 líneas; deadline git global `3 s`; `"timeout": 10` en SessionStart, `5` en el resto; `additionalContextLimit` 2500; canario de spill = 4 × límite = `10000 B` (spec §5.1, §5.4, §10).
- Prioridad al combinar: `Deny`/`Block` > `Rewrite` > `Steer` = `Avisa` > `Allow` (spec §4.4, §10.1).
- Clasificación de Codex: `Bash` → `Shell`; `apply_patch` → `FileEdit` (rutas de `*** Add File:`/`*** Update File:`/`*** Delete File:`); nombre que termina en `spawn_agent` → `Agent`; resto → `Other` (spec §4.3).
- `ENGINE_MIN` = **0.15.0**; engine `0.15.0`; plugin `1.33.0` (spec §4.7, §5.7).
- Roles: `model = "gpt-6.1-sol"`; `model_reasoning_effort` = `high` (consultor) y `medium` (executor) (spec §10.5).
- Inventario de paridad: **61** casos (52 de `test-exo-recall.sh`, 2 de `-golden`, 7 de `-superpowers`); más de **18** como divergencia invalida el port (spec §7.2, §8.4).

## Cierre de ola

- `cargo fmt --check --manifest-path engine/Cargo.toml && cargo clippy --manifest-path engine/Cargo.toml --all-targets --locked -- -D warnings` → verde = exit 0
- `./engine/scripts/test-hermetico.sh` → verde = exit 0
- `f=0; for t in plugins/exo/scripts/test-*.sh scripts/test-hooks-json.sh scripts/test-versiones.sh scripts/test-exec-bit.sh scripts/test-rutas-personales.sh scripts/test-shellcheck.sh scripts/test-docs-vivos.sh scripts/test-docs-sin-local.sh; do bash "$t" </dev/null >/dev/null 2>&1 || { echo "FALLA $t"; f=1; }; done; exit $f` → verde = ninguna línea `FALLA`
- `bash engine/scripts/paridad-hook.sh` → verde = `paridad: OK`
- `[ ! -f engine/tests/hook_reglas.rs ] || bash engine/scripts/paridad-recall.sh` → verde = `paridad-recall: OK` (se aplica desde la ola 3)
- `[ ! -f scripts/test-state-acuerdo.sh ] || bash scripts/test-state-acuerdo.sh` → verde = `test-state-acuerdo: OK`
- `[ ! -f scripts/test-codex-json.sh ] || bash scripts/test-codex-json.sh` → verde = exit 0

## Mutación

- `engine`: manual
- `plugins/exo/scripts`: manual
- `scripts`: manual

## Excluir de la review

- `engine/tests/fixtures/hook/**`
- `engine/tests/paridad/**`

## Recursos compartidos

- `~/.claude/` y `~/.exo/`: los tests **nunca** los tocan. En Rust se usa un `Entorno` con `tempfile`; en bash, `HOME`, `EXO_STATE_DIR` y `REFLEX_LOG_FILE` en un `mktemp -d`.
- Cuenta de Codex y de Claude del dueño: solo la usan los smokes, que corre el dueño (Task 5 y el cierre manual tras la ola 6).
- `engine/src/lib.rs`: Tasks 1, 12 y 16, en olas distintas. `engine/src/main.rs`: Tasks 1, 12, 15 y 16, en olas distintas. `engine/src/hook/mod.rs` y `engine/src/hook/guards/mod.rs`: solo la Task 7, que declara todos los módulos nuevos con su fichero vacío.
- `engine/Cargo.toml`: la Task 9 solo toca `[dev-dependencies]`; la Task 18 toca `version`.
- Versiones (`plugins/exo/.claude-plugin/plugin.json`, `.claude-plugin/marketplace.json`, `plugins/exo/.codex-plugin/plugin.json`, `plugins/exo/ENGINE_MIN`) y `.github/workflows/ci.yml`: solo la Task 18 (la 17 crea `.codex-plugin/plugin.json` con la versión `1.33.0` ya puesta).
- Gate del bump: todo va en **un PR** (como A), así que las tareas que tocan `plugins/exo/` antes de la 18 son válidas.

## Olas

Calculadas con `task-dag` (manda sobre esta lista).

- **Ola 1**, en paralelo: T1 (rutas en Rust), T2 (rutas en bash y TS), T3 (baseline de paridad de recall), T4 (smokes de volcado).
- **Ola 2**, en paralelo: T5 (**manual**: el dueño corre los smokes y commitea los golden; bloquea T9 y, a través de T15, la Task 17), T6 (migración y `harness` en Rust), T7 (tipos, eventos y declaración de módulos), T8 (test de acuerdo).
- **Ola 3**, en paralelo: T9 (adapter `codex`), T10 (marcas en el journal), T11 (guard `Index`), T12 (contrato y reafirmación), T13 (superpowers y gate pendiente), T14 (reglas). Desde aquí se aplica el gate `paridad-recall.sh`.
- **Ola 4:** T15 (wiring: matriz, runner y `--min`).
- **Ola 5**, en paralelo: T16 (`exo setup codex` y roles), T17 (`codex.json`, manifest y coherencia).
- **Ola 6**, en paralelo: T18 (cutover de Claude, versiones y CI), T19 (smoke `--real` y docs).
- **Tras la ola 6 (manual):** el dueño corre `smoke-codex.sh --real` y el `hyperfine` de la spec §7.11, y empuja el tag `v0.15.0` antes de mergear (como en A).

### Task 1: `Rutas` de estado y seams del `Entorno`

**Files:**
- Create: `engine/src/state.rs`
- Modify: `engine/src/lib.rs`
- Modify: `engine/src/hook/evento.rs`
- Modify: `engine/src/main.rs`
- Test: `engine/tests/state.rs`
- Test: `engine/tests/config_cmd.rs`

**Interfaces:**
- Produces: `pub struct Rutas { pub state_dir: PathBuf, pub reflex_log: PathBuf, pub rules_dir: PathBuf, pub index_log: PathBuf, pub canary: PathBuf }`
- Produces: `Rutas::resuelve(home: &Path, var: impl Fn(&str) -> Option<OsString>) -> Rutas` (pura: el test inyecta `var`)
- Produces: `Rutas::desde_env() -> Rutas` (`dirs::home_dir()` + `std::env::var_os`, vacío = ausente)
- Produces: `pub struct Seams { pub exo_bin: Option<PathBuf>, pub exo_index: Option<PathBuf>, pub exo_kb: Option<PathBuf>, pub exo_kb_name: Option<String>, pub recall_nota: Option<String>, pub recall_cap: usize, pub bloque_cap: usize, pub recall_limite: usize, pub compact_lineas: usize, pub git_timeout_s: u64, pub claude_project_dir: Option<PathBuf>, pub codex_home: Option<PathBuf> }`
- Produces: `Entorno { state_dir, reflex_log, rutas: Rutas, seams: Seams, cdpath_definido, panic_guard }` (`state_dir`/`reflex_log` quedan como alias de `rutas.*`, para no tocar los guards de A)
- Produces: `exo config --json` → `data.state.{dir,reflex_log,rules_dir,index_log,canary}`

**Tests:**
- `state::precedencia_variable_propia`: `REFLEX_LOG_FILE=/x/r.jsonl` y `EXO_STATE_DIR=/s` dan `reflex_log=/x/r.jsonl` y `index_log=/s/exo-index.log`. Falla si `EXO_STATE_DIR` pisa a la variable propia.
- `state::default_bajo_home`: sin variables, con home `/h`, da `state_dir=/h/.exo/state`, `rules_dir=/h/.exo/state/exo-rules` y `canary=/h/.exo/state/reflex-inject-canary`. Falla si alguna ruta queda en `/h/.claude/`.
- `state::variable_vacia_es_ausente`: `EXO_STATE_DIR=""` da el default. Falla si resuelve a `""`/cwd.
- `state::seams_defaults`: sin variables, `recall_cap=6144`, `bloque_cap=1500`, `recall_limite=10`, `compact_lineas=2000`, `git_timeout_s=3`. Falla si cambia algún default de `exo-recall.sh`.
- `config_cmd::json_expone_state`: `exo config --json` con `EXO_STATE_DIR` en un tempdir trae `data.state.dir` igual a ese tempdir y conserva `kb`, `index` y `embeddings`. Falla si se pierde una clave existente.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test state --test config_cmd` → `test result: ok`. Más el Cierre de ola.

**Review Focus:**
- `exo config --json` sin config de KB: `data.state` tiene que salir igual (las rutas no dependen de la KB). Test `config_cmd::state_sin_kb`.
- Las rutas de `data.state` en Windows salen con `ruta_shell` como `kb.path`. Test: misma función.

### Task 2: `_state-dir.sh`, consumidores bash y `register.ts`

**Files:**
- Create: `plugins/exo/scripts/_state-dir.sh`
- Modify: `plugins/exo/scripts/_reflex-log.sh`
- Modify: `plugins/exo/scripts/cobertura-reglas.sh`
- Modify: `plugins/exo/scripts/reflex-fp-review.sh`
- Modify: `plugins/exo/scripts/document-remind.sh`
- Modify: `plugins/exo/scripts/subagent-inject.sh`
- Modify: `plugins/exo/scripts/compose-inject.sh`
- Modify: `plugins/exo/scripts/exo-recall.sh`
- Modify: `plugins/exo/scripts/exo-index.sh`
- Modify: `plugins/exo/scripts/reflex-fp-adjudicate.prompt.md`
- Modify: `plugins/exo/hooks/register.ts`
- Modify: `plugins/exo/hooks/register.test.ts`
- Modify: `scripts/e2e-reglas-proyecto.sh`
- Test: `plugins/exo/scripts/test-state-dir.sh`

**Interfaces:**
- Produces: `_state-dir.sh` exporta `EXO_STATE_DIR`, `REFLEX_LOG_FILE`, `EXO_RULES_DIR`, `EXO_INDEX_LOG` y `REFLEX_CANARY_FILE`, con la precedencia de los Global Constraints, sin spawns.
- Produces: `_reflex-log.sh` añade `harness:"claude"` a cada línea.
- Produces: `register.ts`: `hbPath = (home, sid) => \`${process.env.EXO_STATE_DIR || home + "/.exo/state"}/exo-rules/hb-${sid}\``

**Tests:**
- `test-state-dir: default`: `HOME=$t` sin variables → `REFLEX_LOG_FILE=$t/.exo/state/reflex-log.jsonl`. Falla si queda en `.claude`.
- `test-state-dir: variable propia gana`: `EXO_STATE_DIR=$t/s REFLEX_LOG_FILE=$t/r` → `REFLEX_LOG_FILE=$t/r` y `EXO_INDEX_LOG=$t/s/exo-index.log`.
- `test-state-dir: reflex-log lleva harness`: `reflex_log x '{}' p` escribe una línea con `.harness=="claude"`. Falla si falta el campo o rompe el JSON.
- `test-state-dir: sin rutas .claude`: `grep -n '\.claude/\(reflex-log\|exo-rules\|exo-index\|reflex-inject-canary\)'` sobre los ficheros de `Files` (menos los tests) → 0 líneas. Falla si queda un consumidor sin migrar.
- `register.test.ts`: el latido se escribe en `$EXO_STATE_DIR/exo-rules/hb-<sid>`.
- Las suites existentes (`test-exo-recall*.sh`, `test-document-remind.sh`, `test-cobertura-reglas.sh`, `test-reflex-fp-review.sh`, `test-compose-inject.sh`, `test-subagent-inject.sh`, `test-exo-index.sh`) siguen verdes, ajustando solo sus rutas de fixture.

**Verificación:** `bash plugins/exo/scripts/test-state-dir.sh` → `OK`. Más el Cierre de ola.

**Review Focus:**
- `exo-recall.sh:161,167` leía `$HOME/.claude/reflex-log.jsonl` a pelo: tiene que pasar a `$REFLEX_LOG_FILE`. Lo cubre un caso de `test-exo-recall.sh` con `REFLEX_LOG_FILE` en tmp y `source=compact`.
- `compose-inject.sh:10` conserva `REFLEX_CANARY_FILE` como variable propia.
- `_state-dir.sh` no puede usar `$(...)` ni `cd`: se carga en todos los reflejos. Lo cubre el test de shellcheck y un caso que mide 0 forks con `strace -f -e trace=clone,fork` si está disponible (si no, se omite y se dice).

### Task 3: Baseline de paridad de recall y su gate

**Files:**
- Create: `engine/tests/paridad/baseline-recall.txt`
- Create: `engine/tests/paridad/asignacion-recall.txt`
- Create: `engine/scripts/paridad-recall.sh`
- Test: `engine/scripts/test-paridad-recall.sh`

**Interfaces:**
- Produces: `baseline-recall.txt`: los 61 textos de `pass "…"` (`grep -o 'pass "[^"]*"'` sobre las tres suites, únicos y ordenados con `LC_ALL=C`), uno por línea, sin las comillas.
- Produces: `asignacion-recall.txt`: `<área>\t<texto>` para cada línea del baseline, con `<área>` en `contrato|compact|avisos|reglas|fontaneria`.
- Produces: `paridad-recall.sh`: une `engine/tests/paridad/divergencias-recall-*.txt` (formato `<texto>\t<motivo>`) con las líneas `/// paridad-recall: <texto>` de `engine/tests/hook_*.rs`. Falla si una línea del baseline no está en ninguno de los dos, si sobra alguna, si un texto está en los dos, o si las divergencias pasan de 18. Imprime `paridad-recall: OK` o la lista de fallos.

**Tests:**
- `test-paridad-recall: baseline completo`: `wc -l < baseline-recall.txt` = 61. Falla si cambia el inventario.
- `test-paridad-recall: falta`: un fixture sin la línea X → sale ≠0 y nombra X.
- `test-paridad-recall: sobra`: una divergencia que no está en el baseline → sale ≠0 («ajena al baseline»).
- `test-paridad-recall: duplicada`: X en las divergencias y en un test → sale ≠0.
- `test-paridad-recall: tope 18`: 19 divergencias → sale ≠0 con el texto «invalidación §8.4».

**Verificación:** `bash engine/scripts/test-paridad-recall.sh` → `OK`. Más el Cierre de ola (el gate real se aplica desde la ola 3).

**Review Focus:**
- Las ~11 de fontanería por stub (spec §7.2) llevan el área `fontaneria`, y nadie las porta: van a divergencias en las Tasks 12-14 según su área de origen. Comprueba que el área existe para todas las líneas (caso del test «falta»).

**Notas:** el texto de cada `pass "…"` es el id; el gate compara textos exactos con `LC_ALL=C comm`, igual que `paridad-hook.sh:36-45`.

### Task 4: Smokes de volcado para recapturar los golden

**Files:**
- Modify: `scripts/smoke-codex.sh`
- Create: `scripts/smoke-claude.sh`

**Interfaces:**
- Produces: `smoke-codex.sh` (modo por defecto, el de volcado): el `codex.json` de volcado añade `UserPromptSubmit`, `PostCompact`, `PreCompact` y `"additionalContextLimit": 2500` en SessionStart. Tras el primer `codex exec` (que añade `-c model_auto_compact_token_limit=3000`), lanza `codex exec resume --last "di ok"`.
- Produces: `smoke-claude.sh`: `claude -p` con `--settings` apuntando a un fichero temporal con hooks de volcado `cat >> ev.jsonl` para `SessionStart`, `Stop`, `PreToolUse`, `PostToolUse`, `PostToolUseFailure`, `UserPromptSubmit` y `PreCompact`. Fuerza una compactación con `/compact` en un segundo `claude -p --resume`. Escribe el golden saneado en `engine/tests/fixtures/hook/claude/claude-<versión>-probe.jsonl`.

**Tests:**
- Aserciones nuevas de `smoke-codex.sh`, como `chk mal`: hooks cargados con `additionalContextLimit` presente (> 0 eventos); `PostCompact` presente; dos `UserPromptSubmit`. Como `chk info`: `SessionStart` con `source=compact`.
- Aserciones de `smoke-claude.sh`, como `chk mal`: `SessionStart` con `source=startup`; `Stop`; `PostToolUseFailure`. Como `chk info`: `SessionStart` con `source=compact`.
- `bash -n` y shellcheck limpios en los dos.

**Verificación:** `bash scripts/test-shellcheck.sh | tail -1` → `OK`; `bash scripts/test-rutas-personales.sh | tail -1` → `OK`. Los smokes **no** los corre el executor (Task 5).

**Review Focus:**
- El saneado de `smoke-claude.sh` sustituye `$HOME` por `/home/u` y el dir de trabajo por `/tmp/probe`, como el de Codex. Si no, falla `test-docs-sin-local`.
- `RAIZ` sale de la ruta del script (`git -C "$(dirname "$0")"`), no del cwd (bug corregido en `5b0da25`).

### Task 5: Recaptura de golden (manual, la hace el dueño)

**Files:**
- Modify: `engine/tests/fixtures/hook/codex/codex-0.166.0-probe.jsonl`
- Create: `engine/tests/fixtures/hook/claude/claude-<versión>-probe.jsonl`

**Interfaces:**
- Consumes: `smoke-codex.sh` y `smoke-claude.sh` @Task 4
- Produces: golden de Codex con `SessionStart`, `UserPromptSubmit` ×2, `PreToolUse`, `PostToolUse`, `PostCompact` y `Stop`; golden de Claude con `SessionStart` y `Stop`.

**Tests:** n/a — tarea manual: los produce `smoke-codex.sh` y `smoke-claude.sh` con la cuenta del dueño.

**Verificación:** `jq -r .hook_event_name engine/tests/fixtures/hook/codex/codex-0.166.0-probe.jsonl | sort -u` incluye `PostCompact` y `UserPromptSubmit`; el golden de Claude incluye `SessionStart` y `Stop`.

**Notas:** el orquestador **para antes de la ola 3** y pide al dueño: `! scripts/smoke-codex.sh --keep` y `! scripts/smoke-claude.sh --keep` desde el worktree. Si el smoke de Codex no ve `PostCompact` ni el segundo UPS, aplica la condición §8.7 y se pregunta al dueño antes de seguir. El resultado `source=compact` (sí o no) se anota en el ledger: decide la fila `RecallCompact` × codex en la Task 15.

### Task 6: Migración del estado, `harness` en el reflex-log y finding de doctor

**Files:**
- Modify: `engine/src/state.rs`
- Modify: `engine/src/hook/reflex.rs`
- Modify: `engine/src/doctor.rs`
- Test: `engine/tests/state_migra.rs`
- Test: `engine/tests/hook_reflex.rs`
- Test: `engine/tests/doctor.rs`

**Interfaces:**
- Consumes: `Rutas` @Task 1
- Produces: `pub enum Migracion { Renombrado, Fusionado { viejo_a: PathBuf }, Nada, Fallo(String) }`
- Produces: `migra(viejo: &Path, nuevo: &Path, ts: &str) -> Migracion`
- Produces: `migra_todo(home: &Path, r: &Rutas, ts: &str) -> Vec<(&'static str, Migracion)>` para `reflex-log.jsonl`, `exo-index.log` y `reflex-inject-canary` en `~/.claude/`
- Produces: `reflex::registra` añade `harness` (de `ev.harness.nombre()`)
- Produces: check de doctor `state_residuo` (informativo): un fichero viejo de los tres que existe y no está vacío → mensaje con `cat <viejo> >> <nuevo> && mv <viejo> <viejo>.migrado-<ts>`

**Tests:**
- `state_migra::solo_viejo_renombra`: el viejo desaparece y el nuevo tiene el mismo contenido byte a byte. Falla si copia en vez de renombrar.
- `state_migra::ambos_fusiona`: viejo `A\n`, nuevo `B\n` → nuevo `B\nA\n`; el viejo pasa a `<viejo>.migrado-<ts>` con `A\n`. Falla si borra el viejo o si el append no es un único `write`.
- `state_migra::solo_nuevo_nada`: devuelve `Nada` y no crea nada en `~/.claude/`.
- `state_migra::viejo_sin_salto_final`: viejo `A` (sin `\n`) → el nuevo termina en `A\n`. Falla si dos líneas se pegan.
- `state_migra::conservacion_concurrente`: 4 hilos × 200 líneas por `OpenOptions::append` al viejo mientras corre `migra` → `nuevo + .migrado` suman 800 líneas y todas son JSON válido. Falla si se pierde o se corrompe una línea (invalidación §8.2).
- `hook_reflex::lleva_harness`: la línea escrita tiene `harness:"claude"`. Falla si falta.
- `doctor::state_residuo_informativo`: un viejo no vacío da el check con nivel informativo y el `cat >>` exacto. Falla si sale como error.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test state_migra --test hook_reflex --test doctor` → `test result: ok`. Más el Cierre de ola.

**Review Focus:**
- El dir destino no existe todavía (`~/.exo/state/` ausente): `migra` lo crea. Test `state_migra::crea_destino`.
- Cuando el viejo se renombra, `.migrado-<ts>` no pisa uno anterior con el mismo `ts`: se añade un sufijo `-1`. Test `state_migra::migrado_no_pisa`.

**Notas:** `walker.rs:64` y `doctor.rs:205-1103` citan `~/.claude/` por la caché de plugins y los settings, no por estado: no se tocan (la frase de la spec §4.1 sobre ellos no aplica).

### Task 7: Tipos: harness `Codex`, eventos nuevos, `Avisa` y declaración de módulos

**Files:**
- Modify: `engine/src/hook/evento.rs`
- Modify: `engine/src/hook/claude.rs`
- Modify: `engine/src/hook/matriz.rs`
- Modify: `engine/src/hook/runner.rs`
- Modify: `engine/src/hook/mod.rs`
- Modify: `engine/src/hook/guards/mod.rs`
- Create: `engine/src/hook/codex.rs`
- Create: `engine/src/hook/guards/index.rs`
- Create: `engine/src/hook/guards/recall.rs`
- Create: `engine/src/hook/guards/recall_avisos.rs`
- Create: `engine/src/hook/guards/reglas.rs`
- Test: `engine/tests/hook_evento.rs`
- Test: `engine/tests/hook_claude.rs`
- Test: `engine/tests/hook_matriz.rs`
- Test: `engine/tests/hook_cli.rs`

**Interfaces:**
- Produces: `Harness::Codex` (`nombre() = "codex"`)
- Produces: `EventoKind::{SessionStart, Stop, UserPromptSubmit, PostCompact}`
- Produces: `Evento.source: String` (vacío si no viene) y `Evento.prompt: String`
- Produces: `Decision::Avisa(String)`; `combinar` concatena los `Avisa` y los devuelve en `Decision::Avisa` solo si no hay nada de mayor rango, igual que `Steer`
- Produces: `claude::parse` reconoce los cuatro eventos nuevos; `claude::render` de `Avisa` → `{"systemMessage": …}`
- Produces: `matriz::soporte` cubre `(GitC|ZeroResiduo|Verify|SearchFirst, Codex)` con los valores de la spec §6.2
- Produces: los ficheros `codex.rs`, `guards/{index,recall,recall_avisos,reglas}.rs` existen y están declarados en `mod.rs`, vacíos (solo su doc-comment)

**Tests:**
- `hook_evento::avisa_rango_steer`: `combinar([Steer("a"), Avisa("b")])` no pierde ninguno de los dos. Falla si `Avisa` se descarta. `combinar([Deny("x"), Avisa("b")])` = `Deny("x")`.
- `hook_claude::parse_session_start_source`: `{"hook_event_name":"SessionStart","source":"compact",…}` da `kind=SessionStart`, `source="compact"`.
- `hook_claude::render_avisa`: `Avisa("⚠ x")` da `{"systemMessage":"⚠ x"}` exacto. Falla si añade `hookSpecificOutput`.
- `hook_matriz::codex_existentes`: `Verify × Codex` = `Degradado`, y los otros tres `Soportado`. Falla si Verify sale Soportado (falso verde, spec §6.2).
- `hook_claude::kinds_nuevos_no_rompen_pre`: el golden de A sigue parseando igual (regresión).

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_evento --test hook_claude --test hook_matriz` → `test result: ok`. Más el Cierre de ola.

**Review Focus:**
- El runner ignora (`String::new()`) los eventos nuevos hasta la Task 15: un `SessionStart` en esta ola no puede emitir nada. Test `hook_cli::session_start_aun_vacio`, que se borra en la Task 15.
- `source` desconocido (`fork`) no es error de parse. Test `hook_claude::source_desconocido`.

### Task 8: Test de acuerdo de rutas (Rust, bash y TS)

**Files:**
- Create: `scripts/test-state-acuerdo.sh`

**Interfaces:**
- Consumes: `exo config --json` → `data.state.*` @Task 1
- Consumes: `_state-dir.sh` @Task 2

**Tests:**
- `test-state-acuerdo: tres casos`: sin variables, con `EXO_STATE_DIR=$t/s` y con `REFLEX_LOG_FILE=$t/r`. En cada uno, `exo config --json | jq -r .data.state.<x>` = la variable de `_state-dir.sh`, para las cinco rutas. En Windows (`uname -s` contiene `MINGW` o `MSYS`) se compara tras `cygpath -u`. Falla con «Rust y bash discrepan en <x>: <a> ≠ <b>» (invalidación §8.1).
- `test-state-acuerdo: TS`: `grep -F` del literal `process.env.EXO_STATE_DIR || home + "/.exo/state"` en `register.ts`. Falla si `register.ts` resuelve otra raíz.

**Verificación:** `cargo build --manifest-path engine/Cargo.toml && PATH="$PWD/engine/target/debug:$PATH" bash scripts/test-state-acuerdo.sh` → `test-state-acuerdo: OK`. Más el Cierre de ola.

**Review Focus:**
- El test usa el binario del árbol (`engine/target/debug/exo`), no el instalado. Si no, prueba el `exo` 0.14 de `~/.local/bin`. Test: el script falla si `exo --version` no casa con `engine/Cargo.toml`.

### Task 9: Adapter `codex`

**Files:**
- Modify: `engine/src/hook/codex.rs`
- Modify: `engine/Cargo.toml`
- Create: `engine/tests/fixtures/hook/codex/schema/`
- Test: `engine/tests/hook_codex.rs`

**Interfaces:**
- Consumes: `Evento`, `EventoKind`, `Decision::Avisa`, `Harness::Codex` @Task 7
- Consumes: golden de Codex @Task 5
- Produces: `codex::parse(evento: &str, stdin: &[u8]) -> anyhow::Result<Evento>`
- Produces: `codex::clasifica(nombre: &str, input: &Value) -> (Clase, Option<String>, Vec<String>)`
- Produces: `codex::rutas_patch(patch: &str) -> Vec<String>`
- Produces: `codex::render(d: &Decision, ev: &Evento) -> String`
- Produces: `[dev-dependencies] jsonschema` y 6 output schemas copiados de openai/codex `6b2a275` (`codex-rs/hooks/schema/generated/{pre-tool-use,post-tool-use,session-start,user-prompt-submit,post-compact,stop}.command.output.schema.json`) con su sha256 en `schema/SHA256SUMS`

**Tests:**
- `hook_codex::golden_parsea_todo`: cada línea del golden 0.166.0 (Task 5) parsea sin error. Falla si una línea real rompe el parse.
- `hook_codex::apply_patch_rutas`: patch con `*** Add File: a.txt`, `*** Update File: src/b.rs` y `*** Delete File: c` da `["a.txt","src/b.rs","c"]`. Sin cabeceras → `Clase::Other`.
- `hook_codex::spawn_por_sufijo`: `collaborationspawn_agent` y `spawn_agent` → `Agent`; `collaborationwait_agent` → `Other`. Falla si clasifica por igualdad.
- `hook_codex::allow_es_objeto_vacio`: `render(Allow)` en PreToolUse = `{}`. Falla si emite `permissionDecision: allow` (inválido en Codex, A §3).
- `hook_codex::avisa_doble`: `render(Avisa("⚠ x"))` en SessionStart trae `systemMessage:"⚠ x"` **y** la línea `⚠ x` dentro de `hookSpecificOutput.additionalContext`.
- `hook_codex::schemas`: el render de cada `Decision` válida en cada evento valida contra su schema; un campo extra falla (`deny_unknown_fields`).

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_codex` → `test result: ok`. Más el Cierre de ola.

**Review Focus:**
- `PostCompact` y `Stop` con `Steer`/`Avisa`: el schema de `post-compact` no tiene `hookSpecificOutput`, así que `Steer` se descarta y se renderiza `{}`. Test `hook_codex::post_compact_sin_contexto`.
- Un patch con CRLF (`\r\n`). Test `hook_codex::patch_crlf`.

**Notas:** los schemas son de HEAD, no de 0.166.0 (spec §7.4): el doc-comment del test lo dice.

### Task 10: Marcas de compactación en el journal

**Files:**
- Modify: `engine/src/hook/journal.rs`
- Test: `engine/tests/hook_journal.rs`

**Interfaces:**
- Consumes: `EventoKind::{PostCompact, UserPromptSubmit}` @Task 7
- Produces: `registra_marca(en: &Entorno, ev: &Evento, code: &str) -> std::io::Result<()>` (kind `"mark"`)
- Produces: `marca_pendiente(en: &Entorno, ev: &Evento) -> bool`: hay un `post-compact` posterior al último `post-compact-consumed` en la sesión
- Produces: `registra_evento(en: &Entorno, ev: &Evento)` para `UserPromptSubmit`/`PostCompact` (kind `"event"`)

**Tests:**
- `hook_journal::marca_y_consumo`: `post-compact` → `pendiente=true`; tras `post-compact-consumed` → `false`; otro `post-compact` → `true`. Falla si el consumo es permanente.
- `hook_journal::marca_de_otra_sesion`: una marca en la sesión A no la ve la B.
- `hook_journal::marca_y_agent_id`: una marca con `agent_id` de un subagente no cuenta para el padre (filtro de A §4.2).

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_journal` → `test result: ok`. Más el Cierre de ola.

**Review Focus:**
- Una línea corrupta entre la marca y el consumo se ignora y no rompe `marca_pendiente`. Test `hook_journal::marca_con_linea_corrupta`.

### Task 11: Guard `Index`

**Files:**
- Modify: `engine/src/hook/guards/index.rs`
- Test: `engine/tests/hook_guard_index.rs`

**Interfaces:**
- Consumes: `Entorno.seams.{exo_bin, exo_index}`, `Entorno.rutas.index_log` @Task 1
- Produces: `index(ev: &Evento, en: &Entorno) -> Decision` (siempre `Allow`)
- Produces: `lanza_desacoplado(bin: &Path, db: &Path, log: &Path) -> std::io::Result<()>`: Unix con `process_group(0)` y stdin/stdout/stderr redirigidos (stdout y stderr en append a `log`); Windows con `creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)`
- Produces: binario lanzado = `seams.exo_bin` o, si no, `std::env::current_exe()`

**Tests:**
- `hook_guard_index::no_espera`: un stub que se bloquea en un lock que tiene el test. `index()` vuelve con el stub vivo; el test suelta el lock y el log aparece. Falla si el guard espera al hijo. Sin reloj.
- `hook_guard_index::argumentos`: el stub recibe exactamente `index --db <exo_index> --json`.
- `hook_guard_index::fallo_detach`: `exo_bin` inexistente → `Allow` y una línea `index-fallback` con `reason=detach-failed`.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_guard_index` → `test result: ok`. Más el Cierre de ola.

**Review Focus:**
- Sin `exo_index` → usa el default de `resuelve_db` (config). Si falla, `index-fallback reason=no-db`. Test `hook_guard_index::sin_db`.
- En Windows el test solo corre en `test-windows`, post-merge (spec §7.3). El doc-comment lo dice.

### Task 12: `RecallContrato`, `reafirmacion` y resolvers en la lib

**Files:**
- Modify: `engine/src/hook/guards/recall.rs`
- Modify: `engine/src/lib.rs`
- Modify: `engine/src/main.rs`
- Create: `engine/tests/paridad/divergencias-recall-contrato.txt`
- Test: `engine/tests/hook_recall.rs`

**Interfaces:**
- Consumes: `Entorno.seams` @Task 1; `baseline-recall.txt`, `asignacion-recall.txt` @Task 3
- Produces: `exo::resuelve_db(flag: Option<PathBuf>) -> Result<PathBuf>` y `exo::resuelve_kb(flag: Option<PathBuf>) -> Result<PathBuf>` en la lib (main.rs los reexporta sin cambiar su semántica)
- Produces: `pub struct Fragmentos { pub contrato: String, pub avisos: Vec<String>, pub reafirmacion: String, pub reglas: String }`
- Produces: `recall_contrato(ev: &Evento, en: &Entorno) -> (String, Vec<&'static str>)` (texto + `reason`s de `recall-fallback` que registrar)
- Produces: `reafirmacion(sid: &str, en: &Entorno) -> String`: lee el reflex-log por `rutas.reflex_log`, en las últimas `seams.compact_lineas` líneas
- Produces: `const FALLBACK: &str` con el texto literal de `exo-recall.sh:26-30`

**Tests:** con KB fixture (`tempfile` + `indexa` + `common::con_config`, `engine/tests/recall.rs:50,104`). Cada test lleva `/// paridad-recall: <texto>` si cubre un caso del baseline del área `contrato` o `compact`.
- `hook_recall::contrato_extrae_seccion`: el additionalContext empieza por `## Contrato de memoria` y no incluye el `## ` siguiente.
- `hook_recall::sin_contrato_fallback`: una nota sin «Contrato de memoria» da `FALLBACK` + reason `no-contract`.
- `hook_recall::sin_indice`: db inexistente da `FALLBACK` + `no-index`.
- `hook_recall::reafirma_git_c`: un reflex-log con `git-c-rewrite` en la sesión da `- nunca \`cd X && git ...\`: usa \`git -C X\`.` exacto, bajo la cabecera `--- Reglas reforzadas tras compactacion (dispararon esta sesion) ---`.
- `hook_recall::reafirma_respeta_reflex_log_file`: con `rutas.reflex_log` en tmp, lee de ahí. Falla con el bug de `exo-recall.sh:161`.
- `hook_recall::reafirma_otra_sesion`: los disparos de otra sesión no cuentan.
- Las líneas del área `fontaneria` asignadas a contrato van a `divergencias-recall-contrato.txt` con su motivo.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_recall` → `test result: ok`. Más el Cierre de ola.

**Review Focus:**
- `EXO_RECALL_NOTA` y `EXO_KB_NAME` pisan la nota `<kb>/core/core-index`. Test `hook_recall::seam_nota`.
- Bloque truncado por `cap-bytes`: el contrato se sirve y queda la reason `truncated`. Test `hook_recall::truncado`.
- Mover `resuelve_db`/`resuelve_kb` no puede cambiar `exo search`/`exo recall`: corre `engine/tests/precedencia.rs` entera.

### Task 13: `RecallSuperpowers` y `RecallGatePendiente`

**Files:**
- Modify: `engine/src/hook/guards/recall_avisos.rs`
- Create: `engine/tests/paridad/divergencias-recall-avisos.txt`
- Test: `engine/tests/hook_recall_avisos.rs`

**Interfaces:**
- Consumes: `Entorno.seams.{exo_kb, git_timeout_s, claude_project_dir}` @Task 1; baseline @Task 3
- Produces: `superpowers(home: &Path, proyecto: &Path) -> Option<String>`: aviso literal `AVISO exo: <clave> sigue habilitado junto a exo (skills y hooks duplicados). Desactívalo: claude plugin disable <clave>`, con la misma precedencia de 3 ficheros que `exo-recall.sh:211-236`
- Produces: `gate_pendiente(kb: &Path, deadline: Duration) -> Result<Vec<String>, GateTimeout>`: avisos `AVISO exo: gate pendiente en <ruta>, sin auto-distill que lo resuelva (corre /distill)`
- Produces: `corre_con_deadline(cmd: Command, hasta: Instant) -> Option<Output>`: `try_wait` + `kill` al vencer

**Tests:** cada uno con `/// paridad-recall:` si cubre el área `avisos`.
- `hook_recall_avisos::superpowers_true_en_proyecto`: `{"superpowers@a":false,"superpowers@b":true}` en `.claude/settings.json` del proyecto → aviso con `superpowers@b`. Falla si mira solo la primera clave.
- `hook_recall_avisos::superpowers_local_pisa`: `settings.local.json` con `false` pisa a `true` de usuario → sin aviso.
- `hook_recall_avisos::gate_pendiente_aviso`: KB git fixture con un commit `Gate: notas/x.md bytes=9000 limit=8000 routed=bitacora` sin auto-distill posterior → aviso con `notas/x.md`.
- `hook_recall_avisos::gate_resuelto_por_tamano`: la nota pesa menos del 85 % de `limit` → sin aviso.
- `hook_recall_avisos::gate_deadline`: un `git` stub que duerme 10 s con un deadline de 200 ms vuelve en menos de 2 s con `GateTimeout`. Falla si el deadline es por comando y no global.
- `hook_recall_avisos::kb_sin_git`: KB sin `.git` → `Ok(vec![])`.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_recall_avisos` → `test result: ok`. Más el Cierre de ola.

**Review Focus:**
- Rutas de nota con espacios, ` — ` y paréntesis, con `core.quotePath=false` (`exo-recall.sh:257`). Test `hook_recall_avisos::ruta_con_em_dash`.
- Línea `Nota: <ruta>` en un auto-distill resuelve el gate (`exo-recall.sh:244`). Test `hook_recall_avisos::resuelto_por_nota`.
- El stub de `git` se inyecta por `PATH` del `Command`, nunca por `set_var`.

### Task 14: `ReglasTestigo` y `ReglasInyecta`

**Files:**
- Modify: `engine/src/hook/guards/reglas.rs`
- Create: `engine/tests/paridad/divergencias-recall-reglas.txt`
- Test: `engine/tests/hook_reglas.rs`

**Interfaces:**
- Consumes: `Entorno.rutas.rules_dir` @Task 1; `exo::reglas::resuelve(kb, cwd) -> Result<Salida>`; baseline @Task 3
- Produces: `pub struct ResultadoReglas { pub bloque: String, pub avisos: Vec<String>, pub status: String, pub reason: String, pub repo: Option<String>, pub n: usize, pub n_global: usize }`
- Produces: `resuelve_reglas(kb: &Path, cwd: &Path) -> ResultadoReglas` (avisos: `⚠ sin reglas de proyecto para <repo> (<reason>)` si `reason` ∈ {`ambigua`,`seccion_vacia`,`error_engine`,`engine_stale`}, y `⚠ <N> reglas duras recortadas al máximo de 10 (corre «exo rules --check»)`)
- Produces: `escribe_testigo(rules_dir: &Path, sid: &str, r: &ResultadoReglas)`: `ss-<sid>` con `{status, reason?, n, n_global, repo}`, con tmp+rename, más el GC de `ss-*`/`hb-*` de más de 7 días
- Produces: `bloque_reglas(global: &[String], proyecto: &[String]) -> String`: el formato de `register.ts:2,25-29`, globales primero, con `## Reglas duras del dueño` / `## Reglas duras del proyecto` + `AUTORIDAD` + `- <regla>`

**Tests:** cada uno con `/// paridad-recall:` si cubre el área `reglas`.
- `hook_reglas::testigo_forma`: `ss-<sid>` es `{"status":"ok","n":3,"n_global":1,"repo":"exo"}` exacto (sin `reason` si está vacío). Falla si cambia el esquema que lee `document-remind.sh`.
- `hook_reglas::sid_con_barra`: un `sid` con `/` no escribe testigo (`exo-recall.sh:344`).
- `hook_reglas::bloque_igual_a_register`: el preámbulo `AUTORIDAD` y las dos cabeceras se extraen por regex de `plugins/exo/hooks/register.ts` y tienen que ser iguales a las constantes Rust. Falla si divergen (spec §10.1.11).
- `hook_reglas::recorte_avisa`: 12 reglas → bloque con 10 y aviso con `2 reglas duras recortadas`.
- `hook_reglas::ambigua_avisa`: `reason=ambigua` → aviso `⚠ sin reglas de proyecto para <repo> (ambigua)`.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_reglas` → `test result: ok`. Más el Cierre de ola, ahora con `paridad-recall.sh` → `paridad-recall: OK`.

**Review Focus:**
- `n` suma proyecto + globales, como `exo-recall.sh:328` (`RN=$((RN + RNG))`). Test `hook_reglas::n_suma`.
- `status=skip` fuerza `n=0` (`exo-recall.sh:323`). Test `hook_reglas::skip_n_cero`.

### Task 15: Wiring: matriz, runner y `--min`

**Files:**
- Modify: `engine/src/hook/matriz.rs`
- Modify: `engine/src/hook/runner.rs`
- Modify: `engine/src/main.rs`
- Test: `engine/tests/hook_matriz.rs`
- Test: `engine/tests/hook_cli.rs`

**Interfaces:**
- Consumes: `codex::{parse,render}` @Task 9; `registra_marca`, `marca_pendiente`, `registra_evento` @Task 10; `index` @Task 11; `recall_contrato`, `reafirmacion`, `FALLBACK` @Task 12; `superpowers`, `gate_pendiente` @Task 13; `resuelve_reglas`, `escribe_testigo`, `bloque_reglas` @Task 14; `migra_todo` @Task 6
- Produces: `Guard::{Index, RecallContrato, RecallCompact, RecallSuperpowers, RecallGatePendiente, ReglasTestigo, ReglasInyecta, MarcaCompact}` con `evento()`, `requiere() -> &'static [(&'static str, &'static str)]`, `tools(h: Harness)` y `decisiones()`. `soporte` cubre la tabla de la spec §6.2, sin comodín; la fila `RecallCompact × Codex` sale del resultado de la Task 5
- Produces: `ArgsHook { harness, evento, #[arg(long)] min: Option<String> }`
- Produces: `runner::ejecutar(en, harness, evento, min: Option<&str>, stdin) -> String`, con los flujos de la spec §5.1-5.3: versión → migración → parse → guards → fragmentos → cap → render
- Produces: `ensambla(h: Harness, f: &Fragmentos) -> String`: Claude = contrato + avisos con cap `bloque_cap` sin partir UTF-8, y luego la reafirmación; Codex = contrato (cap) → reglas → reafirmación → avisos

**Tests:**
- `hook_cli::min_desconocido_rc2`: el binario con `--min 9.9.9`, si la versión es menor, da `systemMessage` con `engine desactualizado (0.15.0 < 9.9.9) — actualiza el binario instalado` y el `FALLBACK`; en Codex añade `sin reglas duras`.
- `hook_cli::session_start_claude`: KB fixture → `additionalContext` empieza por `## Contrato de memoria`, `ss-<sid>` escrito y **sin** reglas en el contexto. Falla si Claude inyecta reglas (lo hace el mod).
- `hook_cli::session_start_codex`: KB fixture con reglas → contexto en el orden contrato → reglas → avisos; sin `ss-<sid>`.
- `hook_cli::cap_claude_utf8`: contrato + avisos de 1600 B con un carácter de 3 B en el corte → ≤ 1500 B y UTF-8 válido; reafirmación intacta detrás.
- `hook_cli::post_compact_y_ups`: Codex `PostCompact` → `{}`; el siguiente `UserPromptSubmit` trae las reglas + la reafirmación; el segundo UPS → `{}`.
- `hook_cli::session_start_compact_consume_marca`: `PostCompact` → `SessionStart(compact)` en Codex inyecta y consume; el UPS siguiente no inyecta.
- `hook_cli::spill_canario`: contexto Codex > 10000 B → reflex `context-spill-risk bytes=<N>`.
- `hook_cli::migra_en_arranque`: con `~/.claude/reflex-log.jsonl` en un HOME de tmp, `SessionStart` lo deja en la ruta nueva.
- `hook_cli::stop_lanza_index`: `Stop` vuelve vacío y el stub de `exo_bin` recibe `index`.
- `hook_matriz::requiere_compact`: `RecallCompact.requiere()` = `[("source","compact")]`.
- `hook_cli::gate_timeout_rastro`: `RecallGatePendiente` devuelve `GateTimeout` → el contexto sale sin aviso de gate y el reflex-log trae `gate-timeout`. Falla si el timeout se calla.
- `hook_cli::migra_fallo_rastro`: un destino no escribible (dir de solo lectura en tmp) → `SessionStart` sale igual y el reflex-log (en otra ruta escribible vía `REFLEX_LOG_FILE`) trae `state-migrate-failed`.
- `hook_cli::reglas_skip_rastro`: `reason=ambigua` → línea `project-rules-skip` con `reason=ambigua repo=<repo>`, como `exo-recall.sh:335`.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_matriz --test hook_cli` → `test result: ok`. Más el Cierre de ola (`paridad-hook.sh` sigue en `OK`: los guards de A no cambian).

**Review Focus:**
- Engine más viejo que `--min`: se salta la migración y el testigo (spec §4.7). Test `hook_cli::stale_no_migra`.
- `source=compact` en Claude: no corre `RecallGatePendiente` ni `ReglasTestigo`. Test `hook_cli::compact_claude_sin_gate`.
- Panic en un sub-guard: el resto del contexto sale igual y queda `hook-panic`. Test `hook_cli::panic_subguard` (con `EXO_HOOK_PANIC=recall-contrato`, solo en debug).
- `project-rules-emitted bytes=N` solo en Codex. Test en `session_start_codex`.
- Un harness desconocido sigue saliendo vacío con rc=0 (regresión de A).

### Task 16: `exo setup codex`, roles TOML e instaladores

**Files:**
- Create: `engine/src/setup_codex.rs`
- Create: `plugins/exo/agents/codex/consultor.toml`
- Create: `plugins/exo/agents/codex/executor.toml`
- Modify: `engine/src/lib.rs`
- Modify: `engine/src/main.rs`
- Modify: `engine/src/doctor.rs`
- Modify: `install.sh`
- Modify: `install.ps1`
- Test: `engine/tests/setup_codex.rs`
- Test: `scripts/test-install.sh`

**Interfaces:**
- Produces: `setup_codex::aplica(config: &str, agentes_dir: &Path) -> Result<String>` (pura: texto entrada → texto salida)
- Produces: `setup_codex::ejecuta(codex_home: &Path, exo_root: &Path) -> Result<()>`: escribe `<exo_root>/codex-agents/{consultor,executor}.toml` (`include_str!` de `plugins/exo/agents/codex/`) y `<codex_home>/config.toml` con tmp+rename
- Produces: subcomando `exo setup codex`; `codex_home` = `seams.codex_home` (`CODEX_HOME`) o `~/.codex`
- Produces: bloque literal entre `# >>> exo` y `# <<< exo` con `[agents.exo-consultor]` y `[agents.exo-executor]`, cada uno con `description` y `config_file` absoluto
- Produces: TOML de rol con `developer_instructions` (derivado de `plugins/exo/agents/{consultor,executor}.md`, con `spawn_agent` en lugar de `Agent`), `model = "gpt-6.1-sol"` y `model_reasoning_effort` = `"high"`/`"medium"`
- Produces: check de doctor `codex_rol_modelo` (informativo): el `model` de un rol que no aparece en `$CODEX_HOME/models_cache.json` `.models[].slug`
- Produces: `install.sh`/`install.ps1`: si `codex` está en el `PATH`, `exo setup codex` con `|| ec=$?` y aviso `setup codex falló (exit N): corre \`exo setup codex\``

**Tests:**
- `setup_codex::idempotente`: `aplica(aplica(x)) == aplica(x)`.
- `setup_codex::respeta_fuera`: lo que hay antes y después de los marcadores queda byte a byte igual.
- `setup_codex::tabla_fuera_falla`: `[agents.exo-consultor]` fuera de los marcadores → `Err` que la nombra; no se escribe nada.
- `setup_codex::marcador_huerfano_falla`: `# >>> exo` sin cierre → `Err`.
- `setup_codex::no_altera_ajenas`: contenido sin cabecera detrás del bloque (que caería en la última tabla) → `Err`. Falla si cambia el valor de una clave ajena al parsear (spec §10.1.12).
- `setup_codex::toml_valido`: la salida y los dos roles parsean con `toml`; el rol del consultor tiene `model_reasoning_effort = "high"`.
- `setup_codex::respeta_codex_home`: con `codex_home` en tmp escribe ahí, no en `~/.codex`.
- `test-install: setup codex no rompe`: un `exo` stub en el que `setup codex` sale con 3 → `install.sh` termina con 0 y muestra el aviso.
- `doctor::rol_modelo_rotado`: una caché sin `gpt-6.1-sol` → check informativo.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test setup_codex --test doctor && bash scripts/test-install.sh` → verde. Más el Cierre de ola.

**Review Focus:**
- El texto de solo lectura del consultor va en `developer_instructions` (la capa de rol no admite sandbox, spec §10.5). Test: `toml_valido` comprueba que el string contiene «solo lectura».
- `install.ps1` no lanza por exit nativo (`install.ps1:177`): hay que comprobar `$LASTEXITCODE`. Lo cubre `scripts/test-install.ps1` si corre en `test-windows`; si no, se dice.

### Task 17: `codex.json`, `.codex-plugin` y tests de coherencia

**Files:**
- Create: `plugins/exo/hooks/codex.json`
- Create: `plugins/exo/.codex-plugin/plugin.json`
- Create: `scripts/test-codex-json.sh`
- Modify: `engine/tests/hook_coherencia.rs`
- Modify: `engine/tests/hook_matriz.rs`

**Interfaces:**
- Consumes: `Guard`, `requiere`, `tools(h)`, `soporte` @Task 15; golden @Task 5
- Produces: `codex.json` = el bloque literal de la spec §5.4, con `--min 0.15.0`
- Produces: `.codex-plugin/plugin.json` = `{"name":"exo","version":"1.33.0","description":<la de marketplace.json>,"hooks":"./hooks/codex.json"}`

**Tests:**
- `hook_matriz::capacidad_con_campos`: para cada guard S/Degradado en cada harness, el golden de ese harness tiene una línea con `hook_event_name == evento()` y todos los campos de `requiere()`. Si no, falla con el texto exacto: `<guard> declarado Soportado en <harness> pero ningún <evento> del golden trae <campo>=<valor>: decláralo NoSoportado o recaptura el golden`.
- `hook_coherencia::codex_cubre_tools`: cada guard S/Degradado en codex tiene una entrada en `codex.json` cuyo matcher cubre `tools(Codex)`, con la semántica de Codex (exacto si `[A-Za-z0-9_|]`, regex sin anclar si no).
- `hook_coherencia::al_menos_uno`: cada entrada `exo hook <h> <ev>` (de `hooks.json` y de `codex.json`) tiene ≥1 guard S/Degradado en `(h, ev)`. Sustituye a `ninguna_entrada_apunta_a_un_no_soportado`.
- `hook_coherencia::informe_no_declarado`: imprime (no falla) los NoSoportado que cada entrada ignora, las entradas `.sh` y las tools del golden de Codex que no cubre ningún matcher.
- `test-codex-json`: `codex.json` es JSON válido, solo trae eventos de Codex (`SessionStart`, `UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `PostCompact`, `Stop`), ninguna clave desconocida en el handler (`type`, `command`, `timeout`, `additionalContextLimit`, `statusMessage`, `commandWindows`, `async`), y `additionalContextLimit` solo en `SessionStart`/`UserPromptSubmit`. Las entradas de tool llevan `|| exit 0`; la de SessionStart, no.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_matriz --test hook_coherencia && bash scripts/test-codex-json.sh` → verde. Más el Cierre de ola.

**Review Focus:**
- La semántica de matcher de Claude no es la de Codex: el test usa una función por harness. Test: un matcher `Bash` (exacto en Codex) no casa `BashX`.
- Codex parsea el fichero con `deny_unknown_fields`: una clave de más tumba **todos** los hooks. Lo cubre `test-codex-json`.

### Task 18: Cutover de Claude, versiones y CI

**Files:**
- Modify: `plugins/exo/hooks/hooks.json`
- Modify: `plugins/exo/ENGINE_MIN`
- Modify: `plugins/exo/.claude-plugin/plugin.json`
- Modify: `.claude-plugin/marketplace.json`
- Modify: `engine/Cargo.toml`
- Modify: `scripts/test-versiones.sh`
- Modify: `scripts/test-plugin.sh`
- Modify: `scripts/test-hooks-json.sh`
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: `runner::ejecutar` con `--min` y `Stop` @Task 15
- Consumes: `codex.json` @Task 17
- Consumes: `paridad-recall.sh` @Task 3
- Consumes: `test-state-acuerdo.sh` @Task 8
- Consumes: `divergencias-recall-*.txt` @Task 14
- Consumes: `exo setup codex` @Task 16
- Produces: `hooks.json`: SessionStart `exo hook claude SessionStart --min 0.15.0` con `"timeout": 10`, en lugar de `exo-recall.sh`; Stop `exo hook claude Stop || exit 0` con `"timeout": 5`, en lugar de `exo-index.sh`
- Produces: se borran `exo-recall.sh`, `exo-index.sh`, `test-exo-recall.sh`, `test-exo-recall-golden.sh`, `test-exo-recall-superpowers.sh` y `test-exo-index.sh`. `_engine-version.sh` se queda (lo usa `test-versiones.sh:61`)
- Produces: `ENGINE_MIN` = `0.15.0`; engine `0.15.0`; plugin `1.33.0` en los tres manifests
- Produces: `test-versiones.sh`: tres `version` iguales; `--min` de `hooks.json` y `codex.json` = `ENGINE_MIN`; la guardia `:71-75` pasa de `0.14.0` a `0.15.0` con el porqué
- Produces: CI: `paridad-recall.sh` y `test-state-acuerdo.sh` en `plugin-tests`; en `test-windows`, `cmd /C "nope || exit 0"` y `cmd /C "exo hook codex PreToolUse || exit 0"` → 0, más `test-state-acuerdo.sh` con Git Bash

**Tests:**
- `test-versiones`: `--min 0.14.0` en `codex.json` con `ENGINE_MIN=0.15.0` → falla con un mensaje que nombra el fichero.
- `test-hooks-json`: la entrada SessionStart de `exo hook` **no** lleva `|| exit 0` y la de Stop sí.
- `test-plugin.sh` ya no descubre los tests borrados, y sigue verde.
- CI Windows: los dos `cmd /C` devuelven 0 (invalidación A §7.5).

**Verificación:** el Cierre de ola completo → verde, con `paridad-recall: OK` y `paridad: OK`.

**Review Focus:**
- Un test que todavía apunte a `exo-recall.sh` o `exo-index.sh` (por glob o por nombre). `grep -rn 'exo-recall.sh\|exo-index.sh' plugins scripts engine .github` → solo las menciones históricas en docs.
- El job `plugin-tests` no descarga ONNX: `paridad-recall.sh` solo hace grep, no ejecuta los tests Rust.

### Task 19: `smoke-codex.sh --real` y docs vivas

**Files:**
- Modify: `scripts/smoke-codex.sh`
- Modify: `README.md`
- Modify: `docs/instalacion.md`
- Modify: `docs/arquitectura.md`
- Modify: `plugins/exo/README.md`

**Interfaces:**
- Consumes: `exo setup codex` @Task 16
- Consumes: `codex.json` @Task 17
- Produces: `smoke-codex.sh --real`: instala el plugin del repo (marketplace local apuntando al worktree) en un `CODEX_HOME` aislado con `exo setup codex`, y comprueba:
  - que el contexto de `SessionStart` trae `## Contrato de memoria` y las reglas;
  - que `git add -A` se deniega;
  - que `systemMessage` **no** aparece en `exec.log`;
  - compactación forzada + `resume --last`: el UPS posterior trae las reglas.
- Produces: docs con los pasos de la spec §5.6 y la tabla de la §6.2 resumida. Se quita «hasta B, instalar el plugin en Codex no hace nada». Las rutas de estado pasan a `~/.exo/state/`.

**Tests:**
- `bash -n` y shellcheck limpios; `test-docs-vivos.sh` y `test-docs-sin-local.sh` verdes.
- `--real` no lo corre el executor (cierre manual tras la ola 6).

**Verificación:** `bash scripts/test-docs-vivos.sh && bash scripts/test-docs-sin-local.sh && bash scripts/test-shellcheck.sh` → verde. Más el Cierre de ola.

**Review Focus:**
- `docs/arquitectura.md` §4.3 decía que un engine viejo se nota en SessionStart: hay que corregirlo a `--min` (spec §2).
- `plugins/exo/README.md:105,163` citan `~/.claude/exo-rules` y `~/.claude/reflex-log.jsonl`.
