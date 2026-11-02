# Plan: `exo hook` — núcleo (subproyecto A)

> For agentic workers: ejecución con `exo:orchestrate`.

**Goal:** `bash-guards` y `search-first` pasan a ejecutarse dentro del engine Rust, detrás de `exo hook claude <evento>` y alimentados por un journal propio. Sin cambio de comportamiento en Claude, salvo las divergencias de la spec §6.4.

**Architecture:** Módulo nuevo `engine/src/hook/` con: tipos neutrales respecto al harness (`Evento`, `Decision`), el adapter de Claude, el journal por sesión, el writer del reflex-log, cuatro guards como funciones puras y un runner que lo une detrás del subcomando `exo hook`. El cutover cambia `hooks.json` y borra los `.sh`. Un oráculo de paridad corre las suites bash actuales contra el binario.

**Tech Stack:** Rust 1.95 (MSRV), clap 4, serde_json, regex 1, tempfile (tests); bash + jq solo en el oráculo.

**Spec:** `docs/superpowers/specs/2026-11-02-exo-hook-multi-harness-design.md`

**Global Constraints:**
- `exo hook` **siempre sale con 0**, también con harness o evento desconocidos y con un stdin ilegible. La decisión va en el JSON de stdout, y `Allow` es stdout vacío (spec §4.3).
- Todo degrada a `Allow`. Un reflejo nunca rompe lo que instrumenta (spec §5).
- Estado nuevo bajo `EXO_STATE_DIR` (por defecto `~/.exo/state`). Journal: `$EXO_STATE_DIR/sessions/claude-<session_id>.jsonl` (spec §4.2).
- El reflex-log **no se mueve**: `REFLEX_LOG_FILE`, por defecto `~/.claude/reflex-log.jsonl`, mismo esquema `{ts, reflex, session_id, agent_id, agent_type, tool, payload}` y payload ≤ 2000 caracteres (spec §3, §6.3).
- Una línea del journal = una sola llamada `write()` sobre un fichero abierto con append. `cmd` va truncado a 200 bytes en frontera UTF-8. La lectura se acota a los últimos 2 MiB. La GC borra los ficheros de más de 14 días al crear una sesión nueva (spec §4.2).
- Prioridad al combinar decisiones: `Deny`/`Block` > `Rewrite` > `Steer` > `Allow` (spec §3).
- Los ids de reflejo y los textos al agente son literales copiados de los `.sh` actuales: no se reescriben.
- Solo Claude en este plan. `Harness` tiene una única variante, `Claude`; B añade `Codex`.

## Cierre de ola

- `cargo fmt --check --manifest-path engine/Cargo.toml && cargo clippy --manifest-path engine/Cargo.toml --all-targets --locked -- -D warnings` → verde = exit 0
- `./engine/scripts/test-hermetico.sh` → verde = exit 0 (`cargo test` completo sin `~/.exo/config.toml`)
- `f=0; for t in plugins/exo/scripts/test-*.sh scripts/test-hooks-json.sh scripts/test-versiones.sh scripts/test-exec-bit.sh scripts/test-rutas-personales.sh scripts/test-shellcheck.sh scripts/test-docs-vivos.sh; do bash "$t" </dev/null >/dev/null 2>&1 || { echo "FALLA $t"; f=1; }; done; exit $f` → verde = ninguna línea `FALLA`
- `[ ! -f engine/scripts/paridad-hook.sh ] || bash engine/scripts/paridad-hook.sh` → verde = `paridad: OK` (el gate existe desde la Task 10)

## Mutación

- `engine`: manual (el `cargo-mutants` nocturno ya existente cubre `engine/src/hook/` cuando entra en main)
- `plugins/exo/scripts`: manual

## Excluir de la review

- `engine/tests/fixtures/hook/**`

## Recursos compartidos

- `~/.claude/reflex-log.jsonl` y `~/.exo/state/`: los tests **nunca** los tocan; usan siempre `REFLEX_LOG_FILE` y `EXO_STATE_DIR` en un `mktemp -d` o `tempfile`.
- `plugins/exo/.claude-plugin/plugin.json`, `.claude-plugin/marketplace.json`, `plugins/exo/ENGINE_MIN` y `engine/Cargo.toml` (`version`): solo la Task 11.
- `engine/src/lib.rs` y `engine/src/main.rs`: `lib.rs` lo toca solo la Task 2; `main.rs`, solo la Task 9.

## Olas

- Ola 1, en paralelo: T1 (bash) y T2 (esqueleto y tipos). Sus ficheros no se solapan.
- Ola 2, en paralelo: T3 (adapter Claude), T4 (journal) y T5 (reflex-log). Las tres consumen solo T2.
- Ola 3, en paralelo: T6 (git-c + zero-residuo), T7 (verify) y T8 (search-first). Ficheros de guard disjuntos.
- Ola 4: T9 (runner, matriz y CLI, el wiring). Consume T3–T8.
- Ola 5: T10 (oráculo de paridad y bench). Consume T9.
- Ola 6: T11 (cutover, bumps y docs). Consume T9 y T10.

### Task 1: Caso «verde → rojo → commit» en la suite bash actual

**Files:**
- Modify: `plugins/exo/scripts/test-bash-guards.sh`

**Interfaces:**
- Produces: caso `verify: verde y luego rojo antes del commit avisa`, en el formato `[PASS]/[FAIL] <nombre>` de la suite

**Tests:**
- `verify: verde y luego rojo antes del commit avisa`. El transcript fixture tiene `pytest` con `is_error:false`, después `pytest` con `is_error:true`, y luego llega `git commit -m x` con un `.py` staged. Esperado: una línea `verify-before-done` en el log y stdout vacío. Falla si verify mira el primer test en lugar del último, o si ignora `is_error`. **Debe pasar contra el `bash-guards.sh` actual**: el caso fija el comportamiento de hoy, no lo cambia.

**Verificación:** `bash plugins/exo/scripts/test-bash-guards.sh` → `=== Resultado: 75/75 pasaron ===`, más el Cierre de ola.

**Review Focus:**
- Los dos `tool_result` tienen que apuntar a `tool_use_id` distintos. Si el fixture reutiliza el id, el caso pasa por la razón equivocada.

### Task 2: Esqueleto `hook/` y tipos neutrales

**Files:**
- Create: `engine/src/hook/mod.rs`
- Create: `engine/src/hook/evento.rs`
- Create: `engine/src/hook/claude.rs`
- Create: `engine/src/hook/journal.rs`
- Create: `engine/src/hook/reflex.rs`
- Create: `engine/src/hook/guards/mod.rs`
- Create: `engine/src/hook/guards/git_c.rs`
- Create: `engine/src/hook/guards/zero_residuo.rs`
- Create: `engine/src/hook/guards/verify.rs`
- Create: `engine/src/hook/guards/search_first.rs`
- Modify: `engine/src/lib.rs`
- Test: `engine/tests/hook_evento.rs`

**Interfaces:**
- Produces: `pub mod hook;` en `lib.rs`. `hook/mod.rs` declara `evento, claude, journal, reflex, guards`, y `guards/mod.rs` declara `git_c, zero_residuo, verify, search_first`. Los ficheros de módulo de las Tasks 3–8 se crean aquí vacíos (solo `//!` de una línea).
- Produces: `pub enum Harness { Claude }` con `fn nombre(&self) -> &'static str` (`"claude"`)
- Produces: `pub enum EventoKind { PreToolUse, PostToolUse, PostToolUseFailure, Otro(String) }`
- Produces: `pub enum Clase { Shell, FileEdit, Agent, Web, Mcp, Other }`
- Produces: `pub struct Tool { pub nombre: String, pub clase: Clase, pub input: serde_json::Value, pub command: Option<String>, pub paths: Vec<String> }`
- Produces: `pub struct Evento { pub harness: Harness, pub kind: EventoKind, pub session_id: String, pub cwd: String, pub agent_id: String, pub agent_type: String, pub tool: Option<Tool> }`
- Produces: `pub fn clasifica(nombre: &str, input: &serde_json::Value) -> (Clase, Option<String>, Vec<String>)`
- Produces: `pub enum Decision { Allow, Deny(String), Rewrite { input: serde_json::Value, razon: String }, Steer(String), Block(String) }`
- Produces: `pub fn combinar(ds: Vec<Decision>) -> Decision`

**Tests:**
- `clasifica_bash_y_powershell_son_shell`: `("Bash", {"command":"ls"})` y `("PowerShell", …)` dan `Clase::Shell` con `command = Some("ls")`. Falla si PowerShell queda en `Other`.
- `clasifica_edit_write_notebook_son_fileedit_con_path`: `("Edit", {"file_path":"a.rs"})` da `FileEdit` y `paths=["a.rs"]`. `NotebookEdit` usa `notebook_path`. Falla si se lee una clave equivocada.
- `clasifica_fileedit_sin_path_se_abstiene`: `("Edit", {})` da `Clase::Other`. Falla si se produce un `FileEdit` sin ruta (spec §3).
- `clasifica_agent_y_task_son_agent`: `"Agent"` y `"Task"` dan `Clase::Agent`.
- `combinar_deny_gana_a_rewrite`: `[Rewrite{..}, Deny("x")]` da `Deny("x")`. Falla si el rewrite sobrevive a un deny (`bash-guards.sh:316-321`).
- `combinar_concatena_razones_y_steers`: `[Deny("a"), Deny("b")]` da `Deny("a\n\nb")`, y `[Steer("a"), Steer("b")]` da `Steer("a\n\nb")`.
- `combinar_vacio_es_allow`: `[]` da `Allow`.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_evento` → todos verdes, más el Cierre de ola.

**Review Focus:**
- Un `tool_input.command` que no sea string (null, número) da `command = None`, nunca un panic. Test: `clasifica_command_no_string_es_none`.

### Task 3: Adapter Claude (`parse` / `render`) con golden

**Files:**
- Modify: `engine/src/hook/claude.rs`
- Create: `engine/tests/fixtures/hook/claude/pre_tool_use_bash.json`
- Create: `engine/tests/fixtures/hook/claude/pre_tool_use_edit.json`
- Create: `engine/tests/fixtures/hook/claude/post_tool_use_bash.json`
- Create: `engine/tests/fixtures/hook/claude/post_tool_use_failure_bash.json`
- Test: `engine/tests/hook_claude.rs`

**Interfaces:**
- Consumes: `Evento`, `EventoKind`, `Tool`, `clasifica`, `Decision` @Task 2
- Produces: `pub fn parse(evento: &str, stdin: &[u8]) -> anyhow::Result<Evento>`
- Produces: `pub fn render(d: &Decision, ev: &Evento) -> String`

**Tests:**
- `parse_golden_pre_tool_use_bash`: sobre el fixture, da `kind=PreToolUse`, `tool.clase=Shell` y `session_id`/`cwd`/`agent_id`/`agent_type` copiados, con `""` cuando faltan. Falla si un campo ausente produce error en vez de `""`.
- `parse_json_invalido_es_err`: `b"no json"` da `Err`. Lo que se hace con ese `Err` es cosa del runner (Task 9).
- `render_allow_es_vacio`: `Allow` da `""`. Falla si emite un `allow` explícito (en Claude salta el sistema de permisos; spec §3).
- `render_deny_pre_tool_use`: da exactamente `{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"<r>"}}`.
- `render_rewrite_preserva_otros_campos_de_tool_input`: el input `{"command":"cd a && git status","description":"d","timeout":5000}`, con rewrite a `git -C a status`, da `updatedInput` con `description` y `timeout` intactos y `permissionDecision:"allow"`. Falla si `updatedInput` pierde campos (test-bash-guards, payload con `desc-original`).
- `render_steer_usa_el_evento`: `Steer("m")` en PreToolUse da `{"hookSpecificOutput":{"hookEventName":"PreToolUse","additionalContext":"m"}}`.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_claude` → verde, más el Cierre de ola.

**Review Focus:**
- Fixtures: si el CLI `claude` está disponible, captúralos con un hook de volcado (`cat >> f`) en un proyecto `mktemp -d` (`claude -p`), y escribe en un comentario del test «capturado con Claude Code <versión>». Si no lo está, constrúyelos con los campos de los payloads de `test-bash-guards.sh`/`test-search-first.sh` y escribe «construido, no capturado».

### Task 4: Journal por sesión

**Files:**
- Modify: `engine/src/hook/journal.rs`
- Test: `engine/tests/hook_journal.rs`

**Interfaces:**
- Consumes: `Evento`, `Clase`, `Harness` @Task 2
- Produces: `pub fn dir_estado() -> PathBuf` (`EXO_STATE_DIR` o `~/.exo/state`)
- Produces: `pub struct Registro { pub ts: String, pub kind: String, pub tool: String, pub clase: String, pub cmd: String, pub paths: Vec<String>, pub ok: Option<bool>, pub agent_id: String, pub code: String }` (serde; `kind` es `"tool"` o `"warn"`)
- Produces: `pub fn ruta(harness: Harness, session_id: &str) -> PathBuf`
- Produces: `pub fn registra_tool(ev: &Evento, ok: Option<bool>) -> std::io::Result<()>`
- Produces: `pub fn registra_warn(ev: &Evento, code: &str) -> std::io::Result<()>`
- Produces: `pub struct Lectura { pub registros: Vec<Registro>, pub truncado: bool }`
- Produces: `pub fn lee(harness: Harness, session_id: &str, agent_id: &str) -> std::io::Result<Option<Lectura>>` (`Ok(None)` si el fichero no existe; filtra por `agent_id` exacto)

**Tests:**
- `registra_y_lee_en_orden`: tres `registra_tool` se leen en el mismo orden. Falla si el orden se pierde, porque verify depende de «el último test».
- `cmd_se_trunca_a_200_bytes_en_frontera_utf8`: un comando de 300 bytes con `ñ` cerca del byte 200 se guarda como ≤ 200 bytes de UTF-8 válido. Falla con un panic de corte de char.
- `una_linea_un_write_concurrente`: 8 hilos × 300 registros sobre la misma sesión dan 2400 líneas, todas JSON válido. Falla si las líneas se entrelazan.
- `lee_filtra_por_agent_id`: registros del padre (`""`) y de `"sub1"`; `lee(.., "")` devuelve solo los del padre (spec §4.2).
- `lee_salta_linea_corrupta`: una línea basura en medio se ignora y las demás se leen.
- `lee_cota_2_mib_marca_truncado`: con un journal de más de 2 MiB, `truncado == true` y solo llegan registros completos del final.
- `lee_inexistente_es_none`: devuelve `Ok(None)`.
- `gc_borra_mayores_de_14_dias_al_crear_sesion`: un fichero con mtime de hace 15 días se borra cuando se crea una sesión nueva; uno de hace 13 días se queda.
- `session_id_no_escapa_del_dir`: `session_id="../../x"` da una ruta dentro de `$EXO_STATE_DIR/sessions/`. Falla con path traversal.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_journal` → verde, más el Cierre de ola.

**Review Focus:**
- Sanitizado de `session_id`: todo carácter fuera de `[A-Za-z0-9_-]` se sustituye por `_`. Lo cubre `session_id_no_escapa_del_dir`.
- La GC solo corre al **crear** el fichero, no en cada escritura. Lo cubre el test de GC, que comprueba que no se borra nada al escribir en una sesión que ya existe.

### Task 5: Reflex-log en Rust y payload truncado

**Files:**
- Modify: `engine/src/hook/reflex.rs`
- Test: `engine/tests/hook_reflex.rs`

**Interfaces:**
- Consumes: `Evento` @Task 2
- Produces: `pub fn ruta_log() -> PathBuf` (`REFLEX_LOG_FILE` o `~/.claude/reflex-log.jsonl`)
- Produces: `pub fn registra(reflex: &str, ev: &Evento, payload: &str)` (best-effort, nunca falla)
- Produces: `pub fn payload_truncado(cmd: &str, patron: &regex::Regex) -> String`
- Produces: `pub fn lee_cola(n: usize) -> Vec<serde_json::Value>` (últimas `n` líneas válidas)

**Tests:**
- `registra_esquema_identico_al_sh`: la línea tiene exactamente las claves `ts, reflex, session_id, agent_id, agent_type, tool, payload`, y `ts` sigue el formato `%Y-%m-%dT%H:%M:%SZ`. Falla si cambia una clave, porque `cobertura-reglas.sh` y `reflex-fp-review.sh` las leen.
- `payload_se_corta_a_2000_chars`: un payload de 2500 caracteres se guarda con 2000.
- `payload_truncado_corto_intacto`: un cmd de ≤ 120 caracteres se devuelve igual.
- `payload_truncado_largo_con_match`: un cmd de 300 caracteres con un match del patrón da `"<120 primeros> … ⟨match⟩ <match>"`. Los matches de más de 140 caracteres quedan como `80…60`, como mucho 5 matches, deduplicados y unidos con `" | "` (`_truncate-payload.sh`).
- `registra_sin_permiso_no_falla`: con `REFLEX_LOG_FILE` en un directorio de solo lectura, no hay panic y la función devuelve sin error.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_reflex` → verde, más el Cierre de ola.

**Review Focus:**
- Los recortes de 120, 80 y 60 cuentan **caracteres**, no bytes (bash `${x:0:n}` en una locale UTF-8). Test con un cmd multibyte en `payload_truncado_largo_con_match`.

### Task 6: Guards git-c y zero-residuo

**Files:**
- Modify: `engine/src/hook/guards/git_c.rs`
- Modify: `engine/src/hook/guards/zero_residuo.rs`
- Test: `engine/tests/hook_guard_git.rs`

**Interfaces:**
- Consumes: `Evento`, `Decision`, `Clase` @Task 2
- Consumes: `registra`, `payload_truncado`, `lee_cola` @Task 5
- Produces: `pub fn git_c(ev: &Evento) -> Decision` (`Rewrite` o `Allow`)
- Produces: `pub fn zero_residuo(ev: &Evento) -> Decision` (`Deny` o `Allow`)
- Produces: `pub fn deny_dirs(cmd: &str) -> Vec<String>` (port de `zr_deny_dirs`)

**Tests:**
- Port 1:1 de los casos de ramas 1a, 1b y 2 de `test-bash-guards.sh`, un `#[test]` por caso y con el mismo nombre. Fallan si cambia un rewrite, un id de log o un deny.
- `git_c_rewrite_literal`: `cd repo && git status` da `Rewrite` con `command = "git -C repo status"` y la razón ``Reflejo git-c: reescrito a `git -C repo status` (equivalente; evita el cd encadenado)``, y loguea `git-c-rewrite` con el payload `"cd repo && git status -> git -C repo status"`.
- `git_c_no_rewrite_fuera_de_allowlist_o_con_metacaracter`: `cd a && git commit`, `cd a && git log | head` y un comando multilínea dan `Allow` y loguean `git-c`.
- `zero_residuo_deny_literal`: `git add -A` con cwd fuera de `/tmp` da `Deny` con el texto exacto de `bash-guards.sh` («Reflejo zero-residuo: `git add -A`/`--all`/`.` sin pathspec arrastra residuo …») y loguea `zero-residuo-deny`.
- `zero_residuo_backoff_tras_3`: con 3 registros `zero-residuo-deny` seguidos en el reflex-log para la misma sesión y el mismo agent, el cuarto da `Allow` y loguea `zero-residuo-backoff`. Falla si la racha no se corta en un `zero-residuo` plano.
- `deny_dirs_heredoc_y_cd`: `cd /tmp/x && git add -A` da `["/tmp/x"]`, y un heredoc que contiene `git add -A` no produce nada.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_guard_git` → verde, más el Cierre de ola.

**Review Focus:**
- Pre-filtro: si `"git"` no aparece en ningún sitio del payload crudo, los dos guards devuelven `Allow` **sin loguear**. Hay un caso «git en otro campo», en `test-bash-guards.sh:726`.
- PowerShell: no se reescribe un path con `~`, se acepta `Set-Location` y la rama 1b casa también `sl`, `pushd` y `push-location`.
- Si `CDPATH` está definido, no se reescribe un path relativo.

**Notas:** Los valores literales (`REWRITE_ALLOWLIST`, `REWRITE_RE` en sus dos variantes, el regex de 1b, `PATRON_ZR`, la ventana de racha de 500 líneas, el umbral 3 y el texto del deny) se copian de `plugins/exo/scripts/bash-guards.sh:126-260`, que es la fuente de verdad hasta la Task 11. El algoritmo de `deny_dirs` es el del awk de `_zero-residuo-clasifica.sh`.

### Task 7: Guard verify (sobre el journal)

**Files:**
- Modify: `engine/src/hook/guards/verify.rs`
- Test: `engine/tests/hook_guard_verify.rs`

**Interfaces:**
- Consumes: `Evento`, `Decision` @Task 2
- Consumes: `lee`, `Lectura`, `Registro` @Task 4
- Consumes: `registra`, `payload_truncado` @Task 5
- Produces: `pub fn verify(ev: &Evento) -> Decision` (siempre `Allow`: solo escribe en el log, spec §6.1)
- Produces: `pub enum Veredicto { Verde, Stale, Warn }` y `pub fn veredicto(regs: &[Registro]) -> Veredicto`

**Tests:**
- `veredicto_sin_test_es_warn`.
- `veredicto_ultimo_test_ok_es_verde`: un registro Shell `pytest` con `ok=Some(true)` da `Verde`.
- `veredicto_verde_luego_rojo_es_warn`: `pytest` ok=true y después `pytest` ok=false da `Warn`. Falla si se usa el primer test (spec §4.2, builder check 1).
- `veredicto_ok_none_es_warn`: `ok=None` da `Warn` (el caso Codex; nunca un falso verde).
- `veredicto_edit_tras_verde_es_stale`: `pytest` ok=true y después un `FileEdit` da `Stale`.
- `verify_solo_con_codigo_staged`: `git commit -m x` en un repo `tempfile` con solo un `.md` staged no loguea nada; con un `.rs` staged y sin test, loguea `verify-before-done`.
- `verify_no_verify_no_dispara`: `git commit --no-verify` no loguea nada.
- `verify_stale_prefijo`: el payload del log empieza por `"stale "`.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_guard_verify` → verde, más el Cierre de ola.

**Review Focus:**
- Los regex de test runner `TR_RE1`/`TR_RE2` se aplican solo a registros de `Clase::Shell` (Bash o PowerShell). Un `Edit` cuyo `cmd` contenga «pytest» no cuenta como test.
- El `git diff --cached --name-only --diff-filter=ACM` se ejecuta con `-C <cwd>` cuando `cwd` no está vacío. Si git falla, no se loguea nada (igual que hoy).
- Journal `None` o error de I/O da `Warn`, igual que «sin transcript» hoy.

**Notas:** `PATRON_VBD`, `TR_RE1`, `TR_RE2` y la regex de extensiones de código se copian de `bash-guards.sh:263-301`.

### Task 8: Guard search-first (sobre el journal)

**Files:**
- Modify: `engine/src/hook/guards/search_first.rs`
- Test: `engine/tests/hook_guard_search_first.rs`

**Interfaces:**
- Consumes: `Evento`, `Decision`, `Clase` @Task 2
- Consumes: `lee`, `registra_warn` @Task 4
- Consumes: `registra` @Task 5
- Produces: `pub fn search_first(ev: &Evento) -> Decision` (`Steer` o `Allow`)

**Tests:**
- `search_first_avisa_sin_busqueda`: el primer `Edit` del padre con el journal vacío da `Steer` con el texto literal de `search-first.sh` («Reflejo search-first: primer trabajo sustantivo de la sesión sin `exo search`/`exo targets` previo. …») y loguea `search-first` con payload `""`.
- `search_first_ok_con_busqueda_previa`: un registro Shell con tool `Bash` y `cmd="exo search --type hybrid x"` da `Allow` y loguea `search-first-ok`.
- `search_first_una_vez_por_sesion`: la segunda llamada de la misma sesión da `Allow` sin loguear, porque ya existe el registro `warn` con code `search-first` (spec §6.4, sustituto del sentinel).
- `search_first_en_subagente_calla`: con `agent_id` no vacío da `Allow` sin loguear y sin registro.
- `search_first_journal_ilegible`: si `lee` devuelve `Err`, da `Steer` y loguea con payload `motivo=journal-ilegible`.
- `search_first_no_aplica_a_bash`: `tool.clase=Shell` da `Allow` sin efectos (el matcher original no incluye Bash).
- `search_first_regex_literal`: `exo.exe search`, `/usr/bin/exo targets` y `echo exo search` casan; `exo searchx` y `noexo search` no.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_guard_search_first` → verde, más el Cierre de ola.

**Review Focus:**
- El registro `warn` se escribe **antes** de decidir, igual que el `touch` del sentinel en `search-first.sh`. Si no, un fallo deja la sesión avisando en cada Edit. Lo cubre `search_first_una_vez_por_sesion` con un journal ilegible en la primera llamada.
- Aplica a las clases `Agent` y `FileEdit` y además a los nombres crudos `Task` y `NotebookEdit`, aunque `NotebookEdit` no traiga ruta. Test: `NotebookEdit` sin `notebook_path` también avisa.

**Notas:** La regex `(^|[;&|/[:space:]])exo(\.exe)?[[:space:]]+(search|targets)([[:space:]]|$)` y el texto del aviso son literales de `search-first.sh`.

### Task 9: Runner, matriz y subcomando `exo hook`

**Files:**
- Modify: `engine/src/hook/mod.rs`
- Create: `engine/src/hook/runner.rs`
- Create: `engine/src/hook/matriz.rs`
- Modify: `engine/src/main.rs`
- Test: `engine/tests/hook_cli.rs`
- Test: `engine/tests/hook_matriz.rs`

**Interfaces:**
- Consumes: `parse`, `render` @Task 3
- Consumes: `registra_tool` @Task 4
- Consumes: `registra` @Task 5
- Consumes: `git_c`, `zero_residuo` @Task 6
- Consumes: `verify` @Task 7
- Consumes: `search_first` @Task 8
- Produces: `pub enum Guard { GitC, ZeroResiduo, Verify, SearchFirst }` con `fn evento(&self) -> EventoKind` y `fn decisiones(&self) -> &'static [&'static str]`
- Produces: `pub enum Soporte { Soportado, Degradado(&'static str), NoSoportado(&'static str) }` y `pub fn soporte(g: Guard, h: Harness) -> Soporte`, un `match` **sin `_`**
- Produces: `pub fn ejecutar(harness: &str, evento: &str, stdin: &[u8]) -> String`
- Produces: subcomando `exo hook <harness> <evento>` (los dos posicionales son `String`, no `ValueEnum`)

**Tests:**
- `cli_harness_desconocido_sale_0_sin_stdout`: `exo hook foo PreToolUse` da exit 0 y stdout vacío. Falla si clap rechaza el valor con exit 2, que **bloquearía** la tool (spec §4.3).
- `cli_stdin_ilegible_sale_0`: stdin `no json` da exit 0, stdout vacío y una línea en el reflex-log con `reflex="hook-error"` y payload `code=parse`.
- `cli_post_tool_use_registra_ok_true` y `cli_post_tool_use_failure_registra_ok_false`: tras cada evento, el journal tiene el registro con `ok` correcto y stdout vacío.
- `cli_pre_tool_use_rewrite_end_to_end`: `cd a && git status` da el JSON de rewrite de la Task 3.
- `cli_panic_en_guard_no_rompe`: con un guard que entra en pánico (inyectado con `EXO_HOOK_PANIC=<guard>`, solo en `#[cfg(debug_assertions)]`), los demás guards corren, el exit es 0 y en el reflex-log aparece `hook-panic` con el nombre del guard. Stderr queda vacío (`panic::set_hook`).
- `matriz_capacidad_claude`: para cada `Guard` `Soportado`/`Degradado` en Claude, todas sus `decisiones()` renderizan algo no vacío salvo `Allow`, y su `evento()` aparece en algún fixture de `engine/tests/fixtures/hook/claude/`. El mensaje de fallo dice «`<guard>` declarado Soportado en claude pero el golden de claude no contiene `<evento>`: decláralo NoSoportado o cambia el evento» (spec §6.1).

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_cli --test hook_matriz` → verde, y `printf 'x' | cargo run -q --manifest-path engine/Cargo.toml -- hook claude PreToolUse; echo $?` → `0`. Más el Cierre de ola.

**Review Focus:**
- `PostToolUse` y `PostToolUseFailure` **solo** escriben en el journal y no ejecutan guards de PreToolUse. Test: `cli_post_tool_use_no_dispara_guards` (un `git add -A` en PostToolUse no da deny).
- `--help` de `exo hook` dice qué hace y que siempre sale con 0 (regla del dueño sobre `--help` en un cutover). Test: `help_producto.rs` lo incluye si ese test enumera subcomandos.

### Task 10: Oráculo de paridad y bench de latencia

**Files:**
- Create: `engine/scripts/hook-wrapper-claude.sh`
- Create: `engine/scripts/paridad-hook.sh`
- Create: `engine/scripts/paridad-divergencias.txt`
- Create: `engine/scripts/bench-hook.sh`
- Modify: `plugins/exo/scripts/test-bash-guards.sh`
- Modify: `plugins/exo/scripts/test-search-first.sh`

**Interfaces:**
- Consumes: `exo hook claude <evento>` @Task 9
- Produces: `paridad-hook.sh` → exit 0 e imprime `paridad: OK` sii el conjunto de líneas `[FAIL] <nombre>` de las dos suites es **igual** al de `paridad-divergencias.txt`
- Produces: `bench-hook.sh` → imprime `p99_ms=<n>` y sale con 0 sii `n < 300`

**Tests:**
- `paridad-hook.sh` contra el binario de la Task 9 da `paridad: OK`. Es la condición de invalidación 2 (spec §7): cada nombre de `paridad-divergencias.txt` lleva al lado (`<nombre>\t<fila de §6.4>`) la fila de la spec que lo justifica, y **un `[FAIL]` fuera de esa lista es un bug, no una divergencia**.
- Falsabilidad: quitar una línea de `paridad-divergencias.txt` hace que `paridad-hook.sh` salga con ≠0 y nombre el caso.
- `bench-hook.sh` sobre un journal sintético de 2 MiB da `p99_ms` por debajo de 300 (condición de invalidación 3). Anota el valor medido en el report.

**Verificación:** `bash engine/scripts/paridad-hook.sh` → `paridad: OK`; `bash engine/scripts/bench-hook.sh` → `p99_ms=<n>` con `n < 300`. Más el Cierre de ola.

**Review Focus:**
- El wrapper aísla el estado **por transcript fixture**: `EXO_STATE_DIR=<base>/<sha1 de transcript_path, o "none-<session_id>">`. Así los casos que comparten `session_id="test-sid"` no se contaminan entre sí y la regla de «una vez por sesión» se conserva dentro de un caso.
- Reinyección: `tool_use` con `is_error:true` va como `PostToolUseFailure`, y el resto como `PostToolUse`. Un `tool_use` sin `tool_result` no se reinyecta, porque en Claude no llega ningún Post (spec §6.3).
- `test-bash-guards.sh` y `test-search-first.sh` pasan a `HOOK="${HOOK:-<valor actual>}"`. Ese es el único cambio en ellos.

### Task 11: Cutover — `hooks.json`, borrado de `.sh`, bumps y docs

**Files:**
- Modify: `plugins/exo/hooks/hooks.json`
- Modify: `scripts/test-hooks-json.sh`
- Modify: `plugins/exo/.claude-plugin/plugin.json`
- Modify: `.claude-plugin/marketplace.json`
- Modify: `plugins/exo/ENGINE_MIN`
- Modify: `engine/Cargo.toml`
- Modify: `engine/scripts/paridad-hook.sh`
- Create: `engine/tests/hook_coherencia.rs`
- Modify: `README.md`
- Modify: `docs/arquitectura.md`

**Interfaces:**
- Consumes: `soporte`, `Guard` @Task 9
- Consumes: `paridad-hook.sh` @Task 10
- Produces: `hooks.json` con `PreToolUse` `^(Bash|PowerShell|Agent|Task|Edit|Write|NotebookEdit)$` → `exo hook claude PreToolUse || exit 0`, y `PostToolUse` / `PostToolUseFailure` con `^(Bash|PowerShell|Edit|Write|NotebookEdit)$` → `exo hook claude PostToolUse || exit 0` / `exo hook claude PostToolUseFailure || exit 0`. Las tres llevan `"timeout": 5`.

**Tests:**
- `test-hooks-json.sh` acepta `PostToolUse` y `PostToolUseFailure` y acepta commands que casen `^exo hook claude (PreToolUse|PostToolUse|PostToolUseFailure) \|\| exit 0$`, siempre que lleven `timeout` 5. Sigue rechazando `UserPromptSubmit`. Falsable: un command `exo hook claude PreToolUse` **sin** `|| exit 0` se rechaza.
- `hook_coherencia::todo_guard_soportado_tiene_entrada`: cada `Guard` `Soportado`/`Degradado` en Claude tiene en `hooks.json` una entrada de su evento cuyo matcher cubre sus tools, y ninguna entrada apunta a un guard `NoSoportado`. Además imprime un **informe**, no un fallo, de las entradas `.sh` que la matriz no cubre (spec §6.1).
- `paridad-hook.sh` sigue dando `paridad: OK` con las suites ya movidas.

**Verificación:** `bash scripts/test-hooks-json.sh && bash scripts/test-versiones.sh && bash scripts/plugin-bump-gate.sh && bash engine/scripts/paridad-hook.sh` → todo verde. Más el Cierre de ola.

**Review Focus:**
- Borrar `bash-guards.sh`, `search-first.sh`, `git-c-bash.sh`, `git-add-all-guard.sh`, `verify-before-commit.sh`, `_truncate-payload.sh`, `_zero-residuo-clasifica.sh` y sus `test-git-c-bash.sh`, `test-git-add-all-guard.sh` y `test-verify-before-commit.sh` con `git rm` y rutas explícitas. Antes, `grep -rn` de cada nombre en `scripts/`, `plugins/`, `docs/` y `README.md`: ninguna referencia viva puede quedar (incluido `scripts/test-plugin.sh`).
- `test-bash-guards.sh` y `test-search-first.sh` se mueven con `git mv` a `engine/tests/paridad/` (fuera del glob `plugins/exo/scripts/test-*.sh` del Cierre de ola, donde fallarían por las divergencias), con `HOOK` por defecto apuntando a `engine/scripts/hook-wrapper-claude.sh`. `paridad-hook.sh` se apunta a la nueva ruta.
- Versiones: plugin `1.31.0` → `1.32.0` en los dos manifests; `ENGINE_MIN` `0.8.0` → `0.14.0`; engine `0.13.0` → `0.14.0`.
- Docs: README y `docs/arquitectura.md` dejan de describir bash-guards y search-first como `.sh` y nombran `exo hook`. `test-docs-vivos.sh` en verde.

**Notas:** El commit declara lo que no se ha visto correr: el CI de Windows del cutover, y que hasta publicar el engine 0.14.0 un `exo` antiguo en el PATH deja los guards migrados sin efecto (el `|| exit 0` los silencia y `exo-recall` avisa por `ENGINE_MIN`). Hay que publicar la release del engine **antes** del plugin 1.32.0.
