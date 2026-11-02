# Plan: `exo hook` — núcleo (subproyecto A)

> For agentic workers: ejecución con `exo:orchestrate`.

**Goal:** `bash-guards` y `search-first` pasan a ejecutarse dentro del engine Rust, detrás de `exo hook claude <evento>`, alimentados por un journal propio. En Claude el comportamiento no cambia, salvo las divergencias de la spec §6.4.

**Architecture:** Módulo nuevo `engine/src/hook/` con: tipos neutrales respecto al harness (`Evento`, `Decision`, `Entorno`), el adapter de Claude, el journal por sesión, el writer del reflex-log, cuatro guards como funciones puras `(&Evento, &Entorno) -> Decision` y un runner que lo une detrás del subcomando `exo hook`. Un oráculo de paridad corre las suites bash actuales contra el binario. El cutover cambia `hooks.json`, borra los `.sh` y añade la paridad a CI.

**Tech Stack:** Rust 1.95 (MSRV), edition 2024, clap 4, serde_json, regex 1, tempfile (tests); bash + jq solo en el oráculo.

**Spec:** `docs/superpowers/specs/2026-11-02-exo-hook-multi-harness-design.md`

**Global Constraints:**
- `exo hook` **siempre sale con 0**, también con un harness o evento desconocidos y con stdin ilegible. La decisión va en el JSON de stdout, y `Allow` = stdout vacío (spec §4.3).
- Todo degrada a `Allow`. Un reflejo nunca rompe lo que instrumenta (spec §5).
- **Las variables de entorno se leen una sola vez**, en `Entorno::desde_env()`, que solo llama el runner (Task 10). Journal, reflex y guards reciben `&Entorno`. **Ningún test Rust usa `set_var`**: construye un `Entorno` con rutas de `tempfile` (en edition 2024 `set_var` es `unsafe`, y `cargo test` corre en hilos paralelos).
- Estado nuevo en `Entorno.state_dir` = `EXO_STATE_DIR` o `~/.exo/state`. Journal en `<state_dir>/sessions/claude-<session_id>.jsonl` (spec §4.2).
- El reflex-log **no se mueve**: `Entorno.reflex_log` = `REFLEX_LOG_FILE` o `~/.claude/reflex-log.jsonl`. Esquema `{ts, reflex, session_id, agent_id, agent_type, tool, payload}` y payload de ≤ 2000 caracteres (spec §3, §6.3).
- Journal: una línea = una sola llamada `write()` en modo append; `cmd` truncado a 200 bytes en frontera UTF-8; lectura limitada a los últimos 2 MiB; GC de lo que tenga más de 14 días al crear una sesión nueva (spec §4.2).
- Prioridad al combinar decisiones: `Deny`/`Block` > `Rewrite` > `Steer` > `Allow` (spec §3).
- Los ids de reflejo y los textos que ve el agente son literales copiados de los `.sh` actuales.
- Solo Claude: `Harness` tiene una única variante, `Claude`.
- Suites de referencia: `test-bash-guards.sh` = **103** casos y `test-search-first.sh` = **16** (medido el 2026-11-03; la spec decía 74/15 y se ha corregido).

## Cierre de ola

- `cargo fmt --check --manifest-path engine/Cargo.toml && cargo clippy --manifest-path engine/Cargo.toml --all-targets --locked -- -D warnings` → verde = exit 0
- `./engine/scripts/test-hermetico.sh` → verde = exit 0 (`cargo test` completo sin `~/.exo/config.toml`)
- `f=0; for t in plugins/exo/scripts/test-*.sh scripts/test-hooks-json.sh scripts/test-versiones.sh scripts/test-exec-bit.sh scripts/test-rutas-personales.sh scripts/test-shellcheck.sh scripts/test-docs-vivos.sh; do bash "$t" </dev/null >/dev/null 2>&1 || { echo "FALLA $t"; f=1; }; done; exit $f` → verde = ninguna línea `FALLA`
- `[ ! -f engine/scripts/paridad-hook.sh ] || bash engine/scripts/paridad-hook.sh` → verde = `paridad: OK` (el gate existe desde la Task 11)

## Mutación

- `engine`: manual (el `cargo-mutants` nocturno ya existente cubre `engine/src/hook/` cuando entra en main)
- `plugins/exo/scripts`: manual

## Excluir de la review

- `engine/tests/fixtures/hook/**`
- `engine/tests/paridad/baseline.txt`

## Recursos compartidos

- `~/.claude/reflex-log.jsonl` y `~/.exo/state/`: los tests **nunca** los tocan. En Rust usan un `Entorno` con `tempfile`; en bash, `REFLEX_LOG_FILE` y `EXO_STATE_DIR` en un `mktemp -d`.
- `plugins/exo/.claude-plugin/plugin.json`, `.claude-plugin/marketplace.json`, `plugins/exo/ENGINE_MIN`, `engine/Cargo.toml` (`version`) y `.github/workflows/ci.yml`: los toca solo la Task 12.
- `engine/src/lib.rs` solo lo toca la Task 2; `engine/src/main.rs`, solo la Task 10.
- Gate del bump: CI lo corre con `BASE` = la base del PR (`ci.yml:164`), no commit a commit. Todo el plan va en **un solo PR**, así que las tareas que tocan `plugins/exo/` antes del bump (1 y 11) son válidas. El bump lo hace la Task 12.

## Olas

- Ola 1, en paralelo: T1 (bash) y T2 (esqueleto, tipos y `Entorno`).
- Ola 2, en paralelo: T3 (adapter Claude), T4 (journal) y T5 (reflex-log).
- Ola 3, en paralelo: T6 (git-c), T7 (zero-residuo), T8 (verify) y T9 (search-first). Sus ficheros son disjuntos.
- Ola 4: T10 (runner, matriz y CLI, el wiring).
- Ola 5: T11 (oráculo de paridad, baseline y bench).
- Ola 6: T12 (cutover, CI, bumps y docs).

### Task 1: Caso «verde → rojo → commit» en la suite bash actual

**Files:**
- Modify: `plugins/exo/scripts/test-bash-guards.sh`

**Interfaces:**
- Produces: caso `verify: verde y luego rojo antes del commit avisa`

**Tests:**
- `verify: verde y luego rojo antes del commit avisa`. Transcript fixture: `pytest` (`tool_result` con `is_error:false`), luego `pytest` (`is_error:true`), y entra `git commit -m x` con un `.py` en staged. Esperado: una línea `verify-before-done` en el log y stdout vacío. Falla si verify mira el primer test y no el último, o si ignora `is_error`. **Tiene que pasar contra el `bash-guards.sh` actual.**

**Verificación:** `bash plugins/exo/scripts/test-bash-guards.sh | tail -1` → `=== Resultado: 104/104 pasaron ===`. Más el Cierre de ola.

**Review Focus:**
- Los dos `tool_result` deben apuntar a `tool_use_id` distintos. Si se reutiliza el id, el caso pasa por la razón equivocada.

### Task 2: Esqueleto `hook/`, tipos neutrales y `Entorno`

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
- Produces: `pub mod hook;` en `lib.rs`. `hook/mod.rs` declara `evento, claude, journal, reflex, guards`, y `guards/mod.rs` declara `git_c, zero_residuo, verify, search_first`. Los ficheros de las Tasks 3–9 se crean vacíos, solo con un `//!` de una línea (clippy `-D warnings` pasa con módulos vacíos; lo comprobó el consultor).
- Produces: `pub struct Entorno { pub state_dir: PathBuf, pub reflex_log: PathBuf, pub cdpath_definido: bool, pub panic_guard: Option<String> }` con `pub fn desde_env() -> Entorno` (`EXO_STATE_DIR` o `~/.exo/state`; `REFLEX_LOG_FILE` o `~/.claude/reflex-log.jsonl`; `cdpath_definido` = `CDPATH` no vacía; `panic_guard` = `EXO_HOOK_PANIC`, que solo se lee con `#[cfg(debug_assertions)]`)
- Produces: `pub enum Harness { Claude }` con `fn nombre(&self) -> &'static str` (`"claude"`)
- Produces: `pub enum EventoKind { PreToolUse, PostToolUse, PostToolUseFailure, Otro(String) }`
- Produces: `pub enum Clase { Shell, FileEdit, Agent, Web, Mcp, Other }`
- Produces: `pub struct Tool { pub nombre: String, pub clase: Clase, pub input: serde_json::Value, pub command: Option<String>, pub paths: Vec<String> }`
- Produces: `pub struct Evento { pub harness: Harness, pub kind: EventoKind, pub session_id: String, pub cwd: String, pub agent_id: String, pub agent_type: String, pub crudo: String, pub tool: Option<Tool> }` (`crudo` es el stdin tal cual: lo necesita el pre-filtro de git)
- Produces: `pub fn clasifica(nombre: &str, input: &serde_json::Value) -> (Clase, Option<String>, Vec<String>)`
- Produces: `pub enum Decision { Allow, Deny(String), Rewrite { input: serde_json::Value, razon: String }, Steer(String), Block(String) }`
- Produces: `pub fn combinar(ds: Vec<Decision>) -> Decision`

**Tests:**
- `clasifica_bash_y_powershell_son_shell`: `("Bash", {"command":"ls"})` y `("PowerShell", …)` dan `Clase::Shell` con `command = Some("ls")`. Falla si PowerShell cae en `Other`.
- `clasifica_edit_write_notebook_son_fileedit_con_path`: `("Edit", {"file_path":"a.rs"})` da `FileEdit` y `paths=["a.rs"]`. `NotebookEdit` lee `notebook_path`.
- `clasifica_fileedit_sin_path_se_abstiene`: `("Edit", {})` da `Clase::Other` (spec §3).
- `clasifica_agent_y_task_son_agent`.
- `clasifica_command_no_string_es_none`: si `command` es null o un número, devuelve `None` y no hay panic.
- `combinar_deny_gana_a_rewrite`: `[Rewrite{..}, Deny("x")]` da `Deny("x")` (`bash-guards.sh:316-321`).
- `combinar_concatena_razones_y_steers`: `[Deny("a"), Deny("b")]` da `Deny("a\n\nb")`; `[Steer("a"), Steer("b")]` da `Steer("a\n\nb")`.
- `combinar_vacio_es_allow`.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_evento` → verde. Más el Cierre de ola.

**Review Focus:**
- `Entorno::desde_env` es la **única** función de `hook/` que lee `std::env`. Compruébalo con un grep en la review: `rg 'std::env|env::var' engine/src/hook` debe dar una sola coincidencia, en `evento.rs`.

### Task 3: Adapter Claude (`parse`/`render`) con golden

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
- `parse_golden_pre_tool_use_bash`: `kind=PreToolUse`, `tool.clase=Shell`, `session_id`/`cwd`/`agent_id`/`agent_type` copiados (`""` si faltan) y `crudo` igual al stdin. Falla si un campo ausente da error.
- `parse_json_invalido_es_err`: `b"no json"` da `Err`.
- `render_allow_es_vacio`: `Allow` da `""`.
- `render_deny_pre_tool_use`: exactamente `{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"<r>"}}`.
- `render_rewrite_preserva_otros_campos_de_tool_input`: `{"command":"cd a && git status","description":"d","timeout":5000}` con un rewrite a `git -C a status` da un `updatedInput` que conserva `description` y `timeout`, con `permissionDecision:"allow"`.
- `render_steer_usa_el_evento`: `Steer("m")` en PreToolUse da `{"hookSpecificOutput":{"hookEventName":"PreToolUse","additionalContext":"m"}}`.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_claude` → verde. Más el Cierre de ola.

**Review Focus:**
- Fixtures: si el CLI `claude` está disponible, se capturan con un hook de volcado (`cat >> f`) en un `mktemp -d` usando `claude -p`, y el test lleva el comentario «capturado con Claude Code <versión>». Si no, se construyen con los campos de los payloads de las suites y el comentario dice «construido, no capturado».

### Task 4: Journal por sesión

**Files:**
- Modify: `engine/src/hook/journal.rs`
- Test: `engine/tests/hook_journal.rs`

**Interfaces:**
- Consumes: `Entorno`, `Evento`, `Clase`, `Harness` @Task 2
- Produces: `pub struct Registro { pub ts: String, pub kind: String, pub tool: String, pub clase: String, pub cmd: String, pub paths: Vec<String>, pub ok: Option<bool>, pub agent_id: String, pub code: String }` (serde; `kind` ∈ {`"tool"`, `"warn"`})
- Produces: `pub fn ruta(en: &Entorno, h: Harness, session_id: &str) -> PathBuf`
- Produces: `pub fn registra_tool(en: &Entorno, ev: &Evento, ok: Option<bool>) -> std::io::Result<()>`
- Produces: `pub fn registra_warn(en: &Entorno, ev: &Evento, code: &str) -> std::io::Result<()>`
- Produces: `pub struct Lectura { pub registros: Vec<Registro>, pub truncado: bool }`
- Produces: `pub fn lee(en: &Entorno, h: Harness, session_id: &str, agent_id: &str) -> std::io::Result<Option<Lectura>>` (`Ok(None)` si no existe; filtra por `agent_id` exacto)

**Tests:**
- `registra_y_lee_en_orden`: tres registros se leen en el mismo orden.
- `cmd_se_trunca_a_200_bytes_en_frontera_utf8`: con 300 bytes y una `ñ` en la frontera, guarda ≤ 200 bytes de UTF-8 válido, sin panic.
- `una_linea_un_write_concurrente`: 8 hilos × 300 registros dan 2400 líneas, todas JSON válido.
- `lee_filtra_por_agent_id`: `lee(.., "")` devuelve solo los registros del padre (spec §4.2).
- `lee_salta_linea_corrupta`.
- `lee_cota_2_mib_marca_truncado`: con un journal de más de 2 MiB da `truncado == true` y solo registros completos del final.
- `lee_inexistente_es_none`.
- `gc_borra_mayores_de_14_dias_al_crear_sesion`: un fichero de hace 15 días se borra y uno de hace 13 se queda. Escribir en una sesión que ya existe no borra nada.
- `session_id_no_escapa_del_dir`: `"../../x"` da una ruta dentro de `<state_dir>/sessions/`.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_journal` → verde. Más el Cierre de ola.

**Review Focus:**
- Sanitizado: todo carácter de `session_id` fuera de `[A-Za-z0-9_-]` se sustituye por `_`.

### Task 5: Reflex-log en Rust y payload truncado

**Files:**
- Modify: `engine/src/hook/reflex.rs`
- Test: `engine/tests/hook_reflex.rs`

**Interfaces:**
- Consumes: `Entorno`, `Evento` @Task 2
- Produces: `pub fn registra(en: &Entorno, reflex: &str, ev: &Evento, payload: &str)` (best-effort, nunca falla)
- Produces: `pub fn payload_truncado(cmd: &str, patron: &regex::Regex) -> String`
- Produces: `pub fn lee_cola(en: &Entorno, n: usize) -> Vec<serde_json::Value>`

**Tests:**
- `registra_esquema_identico_al_sh`: las claves son exactamente `ts, reflex, session_id, agent_id, agent_type, tool, payload`, y `ts` sigue el formato `%Y-%m-%dT%H:%M:%SZ`.
- `payload_se_corta_a_2000_chars`.
- `payload_truncado_corto_intacto`: con ≤ 120 caracteres se devuelve igual.
- `payload_truncado_largo_con_match`: da `"<120 primeros> … ⟨match⟩ <match>"`. Un match de más de 140 caracteres queda como `80…60`; como mucho 5 matches, deduplicados y unidos con `" | "`. Los recortes cuentan **caracteres**: el test usa un cmd con caracteres multibyte (`_truncate-payload.sh`).
- `registra_sin_destino_no_falla`: con `reflex_log` dentro de un padre que es un **fichero regular** (ENOTDIR, que se reproduce igual en Windows) no hay panic.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_reflex` → verde. Más el Cierre de ola.

### Task 6: Guard git-c

**Files:**
- Modify: `engine/src/hook/guards/git_c.rs`
- Test: `engine/tests/hook_guard_git_c.rs`

**Interfaces:**
- Consumes: `Entorno`, `Evento`, `Decision`, `Clase` @Task 2
- Consumes: `registra` @Task 5
- Produces: `pub fn git_c(ev: &Evento, en: &Entorno, cdpath: bool) -> Decision` (`Rewrite` o `Allow`; `cdpath` = si la variable `CDPATH` estaba definida, se lo pasa el runner @Task 10)

**Tests:** (dirigidos; la paridad completa la mide la Task 11)
- `git_c_rewrite_literal`: `cd repo && git status` da `Rewrite` con `command="git -C repo status"`, razón ``Reflejo git-c: reescrito a `git -C repo status` (equivalente; evita el cd encadenado)``, y log `git-c-rewrite` con payload `"cd repo && git status -> git -C repo status"`.
- `git_c_fuera_de_allowlist_o_con_metacaracter_loguea`: `cd a && git commit`, `cd a && git log | head` y un comando multilínea dan `Allow` y log `git-c`.
- `git_c_powershell`: `Set-Location a && git status` reescribe; `cd ~/a && git status` con PowerShell no reescribe.
- `git_c_relativo_con_cdpath_no_reescribe`: `cd repo && git status` con `cdpath=true` da `Allow` y log `git-c`; con `cdpath=false`, `Rewrite`. Falla si se ignora `CDPATH`, que cambia a qué directorio va `cd`.
- `git_c_punto_punto_no_reescribe`: `cd ../x && git status` da `Allow` y log `git-c`.
- `git_c_prefiltro_sin_git_no_loguea`: un payload sin `"git"` en ningún sitio de `crudo` da `Allow` y no escribe ninguna línea de log.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_guard_git_c` → verde. Más el Cierre de ola.

**Notas:** `REWRITE_ALLOWLIST`, las dos variantes de `REWRITE_RE` y el regex de 1b son literales de `plugins/exo/scripts/bash-guards.sh:126-195`.

### Task 7: Guard zero-residuo

**Files:**
- Modify: `engine/src/hook/guards/zero_residuo.rs`
- Test: `engine/tests/hook_guard_zero_residuo.rs`

**Interfaces:**
- Consumes: `Entorno`, `Evento`, `Decision` @Task 2
- Consumes: `registra`, `payload_truncado`, `lee_cola` @Task 5
- Produces: `pub fn zero_residuo(ev: &Evento, en: &Entorno) -> Decision` (`Deny` o `Allow`)
- Produces: `pub fn deny_dirs(cmd: &str) -> Vec<String>` (port de `zr_deny_dirs`)

**Tests:** (dirigidos; la paridad completa la mide la Task 11)
- `zero_residuo_deny_literal`: `git add -A` con un cwd fuera de `/tmp` da `Deny` con el texto exacto de `bash-guards.sh` («Reflejo zero-residuo: `git add -A`/`--all`/`.` sin pathspec arrastra residuo …») y log `zero-residuo-deny`.
- `zero_residuo_tmp_solo_loguea`: con cwd `/tmp/x` da `Allow` y log `zero-residuo`.
- `zero_residuo_backoff_tras_3`: tres `zero-residuo-deny` seguidos (misma sesión y agent) → el cuarto da `Allow` y log `zero-residuo-backoff`; un `zero-residuo` plano corta la racha.
- `deny_dirs_cd_y_variable`: `S=/tmp/x; cd $S/g && git add -A` da `["/tmp/x/g"]`.
- `deny_dirs_heredoc_y_comentario`: un `git add -A` dentro de un heredoc o tras `#` no da nada; `git add -A -- ruta` (con pathspec) no da nada.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_guard_zero_residuo` → verde. Más el Cierre de ola.

**Notas:** `PATRON_ZR`, la ventana de racha (500 líneas), el umbral 3 y el texto del deny son literales de `bash-guards.sh:197-260`. El algoritmo de `deny_dirs` es el awk de `_zero-residuo-clasifica.sh`.

### Task 8: Guard verify (sobre el journal)

**Files:**
- Modify: `engine/src/hook/guards/verify.rs`
- Test: `engine/tests/hook_guard_verify.rs`

**Interfaces:**
- Consumes: `Entorno`, `Evento`, `Decision` @Task 2
- Consumes: `lee`, `Registro` @Task 4
- Consumes: `registra`, `payload_truncado` @Task 5
- Produces: `pub fn verify(ev: &Evento, en: &Entorno) -> Decision` (siempre `Allow`: solo escribe en el log, spec §6.1)
- Produces: `pub enum Veredicto { Verde, Stale, Warn }` y `pub fn veredicto(regs: &[Registro]) -> Veredicto`

**Tests:**
- `veredicto_sin_test_es_warn`.
- `veredicto_ultimo_test_ok_es_verde`.
- `veredicto_verde_luego_rojo_es_warn`: falla si se usa el primer test (spec §4.2).
- `veredicto_ok_none_es_warn` (Codex: nunca un falso verde).
- `veredicto_edit_tras_verde_es_stale`.
- `verify_solo_con_codigo_staged`: en un repo `tempfile` con solo un `.md` staged no se loguea nada; con un `.rs` staged y sin test se loguea `verify-before-done`.
- `verify_no_verify_no_dispara`.
- `verify_stale_prefijo`: el payload empieza por `"stale "`.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_guard_verify` → verde. Más el Cierre de ola.

**Review Focus:**
- `TR_RE1`/`TR_RE2` se aplican solo a registros de `Clase::Shell`. Un `FileEdit` cuyo `cmd` contiene «pytest» no cuenta.
- Si el journal es `None` o da error de I/O, el resultado es `Warn`.

**Notas:** `PATRON_VBD`, `TR_RE1`, `TR_RE2` y la regex de extensiones son literales de `bash-guards.sh:263-301`.

### Task 9: Guard search-first (sobre el journal)

**Files:**
- Modify: `engine/src/hook/guards/search_first.rs`
- Test: `engine/tests/hook_guard_search_first.rs`

**Interfaces:**
- Consumes: `Entorno`, `Evento`, `Decision` @Task 2
- Consumes: `lee`, `registra_warn` @Task 4
- Consumes: `registra` @Task 5
- Produces: `pub fn search_first(ev: &Evento, en: &Entorno) -> Decision` (`Steer` o `Allow`)

**Tests:**
- `search_first_edit_sin_path_tambien_avisa`: `tool_name:"Edit", tool_input:{}` (el payload real de `test-search-first.sh:46`) con el journal vacío da `Steer`. Falla si el guard selecciona por `Clase` en vez de por nombre: `Edit` sin ruta se clasifica como `Other`.
- `search_first_avisa_sin_busqueda`: `Steer` con el texto literal de `search-first.sh` y log `search-first`, payload `""`.
- `search_first_ok_con_busqueda_previa`: un registro con tool `Bash` y `cmd="exo search --type hybrid x"` da `Allow` y log `search-first-ok`.
- `search_first_una_vez_por_sesion`: la segunda llamada da `Allow` sin log (ya existe el `warn` con code `search-first`).
- `search_first_en_subagente_calla`.
- `search_first_journal_ilegible`: si `lee` devuelve `Err`, da `Steer` y log con `motivo=journal-ilegible`.
- `search_first_no_aplica_a_bash`.
- `search_first_regex_literal`: casan `exo.exe search`, `/usr/bin/exo targets` y `echo exo search`; no casan `exo searchx` ni `noexo search`.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_guard_search_first` → verde. Más el Cierre de ola.

**Review Focus:**
- Selección **por `tool.nombre`** ∈ {`Agent`, `Task`, `Edit`, `Write`, `NotebookEdit`}, no por `Clase`: es el matcher del `.sh`.
- El `warn` se escribe **antes** de decidir, igual que el `touch` del sentinel. Test: con el journal ilegible en la primera llamada, la segunda no vuelve a avisar.

**Notas:** La regex `(^|[;&|/[:space:]])exo(\.exe)?[[:space:]]+(search|targets)([[:space:]]|$)` y el texto del aviso son literales de `search-first.sh`.

### Task 10: Runner, matriz y subcomando `exo hook`

**Files:**
- Modify: `engine/src/hook/mod.rs`
- Create: `engine/src/hook/runner.rs`
- Create: `engine/src/hook/matriz.rs`
- Modify: `engine/src/main.rs`
- Test: `engine/tests/hook_cli.rs`
- Test: `engine/tests/hook_matriz.rs`

**Interfaces:**
- Consumes: `Entorno::desde_env` @Task 2
- Consumes: `parse`, `render` @Task 3
- Consumes: `registra_tool` @Task 4
- Consumes: `registra` @Task 5
- Consumes: `git_c` @Task 6
- Consumes: `zero_residuo` @Task 7
- Consumes: `verify` @Task 8
- Consumes: `search_first` @Task 9
- Produces: `pub enum Guard { GitC, ZeroResiduo, Verify, SearchFirst }` con `fn evento(&self) -> EventoKind` y `fn decisiones(&self) -> &'static [&'static str]`
- Produces: `pub enum Soporte { Soportado, Degradado(&'static str), NoSoportado(&'static str) }` y `pub fn soporte(g: Guard, h: Harness) -> Soporte` como `match` **sin `_`**
- Produces: `pub fn ejecutar(en: &Entorno, harness: &str, evento: &str, stdin: &[u8]) -> String`
- Produces: subcomando `exo hook <harness> <evento>` con posicionales `String` (no `ValueEnum`)

**Tests:**
- `cli_harness_desconocido_sale_0_sin_stdout`: `exo hook foo PreToolUse` da exit 0 y stdout vacío. Falla si clap responde con exit 2, que **bloquearía** (spec §4.3).
- `cli_stdin_ilegible_sale_0`: da exit 0, stdout vacío y una línea en el reflex-log con `reflex="hook-error"`, payload `code=parse`.
- `cli_post_tool_use_registra_ok_true` y `cli_post_tool_use_failure_registra_ok_false`.
- `cli_post_tool_use_no_dispara_guards`: `git add -A` en PostToolUse no da deny.
- `cli_pre_tool_use_rewrite_end_to_end`.
- `cli_panic_en_guard_no_rompe`: `EXO_HOOK_PANIC=<guard>`, solo con `#[cfg(debug_assertions)]` y leído en `desde_env`, hace que los demás guards sigan, el exit sea 0, el reflex-log reciba `hook-panic` con el nombre del guard y stderr quede vacío (`panic::set_hook`).
- `matriz_capacidad_claude`: para cada `Guard` `Soportado`/`Degradado`, sus `decisiones()` renderizan algo no vacío (salvo `Allow`) y su `evento()` aparece en algún fixture de `engine/tests/fixtures/hook/claude/`. Mensaje si falla: «`<guard>` declarado Soportado en claude pero el golden de claude no contiene `<evento>`: decláralo NoSoportado o cambia el evento» (spec §6.1).

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --test hook_cli --test hook_matriz` → verde; `printf 'x' | cargo run -q --manifest-path engine/Cargo.toml -- hook claude PreToolUse; echo $?` → `0`. Más el Cierre de ola.

**Review Focus:**
- Tests CLI: lanzan el binario con `EXO_STATE_DIR` y `REFLEX_LOG_FILE` en el `Command` hijo (`.env(...)`), **no** con `set_var`.
- `--help` de `exo hook` dice qué hace y que siempre sale con 0. Si `engine/tests/help_producto.rs` enumera subcomandos, el nuevo entra ahí.

**Notas:** `main.rs` tiene un `match` exhaustivo sin `_` en `quiere_json` (`main.rs:525`): el brazo `Comando::Hook` se añade ahí también. El runner pasa `en.cdpath_definido` como el `cdpath` de `git_c`.

### Task 11: Oráculo de paridad, baseline y bench

**Files:**
- Create: `engine/scripts/hook-wrapper-claude.sh`
- Create: `engine/scripts/paridad-hook.sh`
- Create: `engine/tests/paridad/baseline.txt`
- Create: `engine/tests/paridad/divergencias.txt`
- Create: `engine/scripts/bench-hook.sh`
- Modify: `plugins/exo/scripts/test-bash-guards.sh`
- Modify: `plugins/exo/scripts/test-search-first.sh`

**Interfaces:**
- Consumes: `exo hook claude <evento>` @Task 10
- Produces: `hook-wrapper-claude.sh`, que lee `EXO_BIN` (por defecto `engine/target/debug/exo`)
- Produces: `paridad-hook.sh`, que sale con 0 e imprime `paridad: OK` si y solo si el conjunto de líneas `[PASS]` normalizadas de las dos suites es igual a `baseline.txt` menos `divergencias.txt`
- Produces: `bench-hook.sh`, que imprime `p99_ms=<n>` y sale con 0 si y solo si `n < 300`

**Tests:**
- `baseline.txt` se genera **una vez**, ejecutando las suites contra los `.sh` actuales (que todavía existen en esta ola). Contiene sus 104 + 16 líneas `[PASS]`, normalizadas con `sed -E 's/\([0-9]+ bytes\)/(N bytes)/'`. La comparación usa `[PASS]` y no `[FAIL]` porque algunos nombres de `[FAIL]` difieren de su `[PASS]` (`test-bash-guards.sh:602`) y 4 llevan un tamaño calculado (líneas 276, 296, 529, 551).
- `divergencias.txt`: una línea `<[PASS] normalizado>\t<fila de spec §6.4>` por cada caso que diverge. La fila «jq envenenado o ausente» incluye `caso6` de search-first y la **Sección D** de bash-guards (cuentan invocaciones de jq). **Un `[PASS]` que falte y no esté en la lista es un bug, no una divergencia.**
- `paridad-hook.sh` contra el binario de la Task 10 da `paridad: OK`. Es la condición de invalidación 2 (spec §7).
- Falsabilidad: quitar una línea de `divergencias.txt` hace que `paridad-hook.sh` salga con ≠0 y nombre el caso.
- `bench-hook.sh` sobre un journal sintético de 2 MiB da `p99_ms` menor que 300 (invalidación 3). El valor medido se anota en el report; `hyperfine` está disponible.

**Verificación:** `bash engine/scripts/paridad-hook.sh` → `paridad: OK`; `bash engine/scripts/bench-hook.sh` → `p99_ms=<n>` con `n < 300`. Más el Cierre de ola.

**Review Focus:**
- Reinyección (spec §6.3): `tool_use` con `tool_result.is_error:true` → `PostToolUseFailure`; con `is_error:false` **o sin `tool_result`** → `PostToolUse`. Los fixtures de search-first no tienen ningún `tool_result`: si no se reinyectan, fallan los 7 casos `search-first-ok`.
- Los eventos reinyectados llevan `session_id`, `agent_id` y `cwd` del payload exterior.
- Aislamiento: `EXO_STATE_DIR=<base>/<sha1 de transcript_path, o "none-<session_id>">`, porque los casos comparten `session_id="test-sid"`.
- Las suites pasan a `HOOK="${HOOK:-<valor actual>}"`. Es el único cambio en ellas.

### Task 12: Cutover — `hooks.json`, borrado de `.sh`, CI, bumps y docs

**Files:**
- Modify: `plugins/exo/hooks/hooks.json`
- Modify: `scripts/test-hooks-json.sh`
- Modify: `.github/workflows/ci.yml`
- Modify: `plugins/exo/.claude-plugin/plugin.json`
- Modify: `.claude-plugin/marketplace.json`
- Modify: `plugins/exo/ENGINE_MIN`
- Modify: `engine/Cargo.toml`
- Modify: `engine/scripts/paridad-hook.sh`
- Create: `engine/tests/hook_coherencia.rs`
- Modify: `README.md`
- Modify: `docs/arquitectura.md`

**Interfaces:**
- Consumes: `soporte`, `Guard` @Task 10
- Consumes: `paridad-hook.sh` @Task 11
- Produces: `hooks.json` con `PreToolUse` `^(Bash|PowerShell|Agent|Task|Edit|Write|NotebookEdit)$` → `exo hook claude PreToolUse || exit 0`, y `PostToolUse`/`PostToolUseFailure` `^(Bash|PowerShell|Edit|Write|NotebookEdit)$` → `exo hook claude PostToolUse || exit 0` / `exo hook claude PostToolUseFailure || exit 0`, las tres con `"timeout": 5`.

**Tests:**
- `test-hooks-json.sh` acepta `PostToolUse` y `PostToolUseFailure` y commands que casen `^exo hook claude (PreToolUse|PostToolUse|PostToolUseFailure) \|\| exit 0$` con `timeout` 5. Sigue rechazando `UserPromptSubmit`. Falsable: rechaza `exo hook claude PreToolUse` sin `|| exit 0`.
- `hook_coherencia::todo_guard_soportado_tiene_entrada`: cada `Guard` `Soportado`/`Degradado` tiene en `hooks.json` una entrada de su evento cuyo matcher cubre sus tools, y ninguna entrada apunta a un `NoSoportado`. Además imprime un **informe**, no un fallo, de las entradas `.sh` que la matriz no cubre (spec §6.1).
- CI: un paso nuevo `bash engine/scripts/paridad-hook.sh` en el job `test`, después de `test-contrato-ci.sh` (`ci.yml:282`). Sin él, los 120 casos de las suites dejan de correr en CI cuando salen del glob.

**Verificación:** `bash scripts/test-hooks-json.sh && bash scripts/test-versiones.sh && BASE=origin/main bash scripts/plugin-bump-gate.sh && bash engine/scripts/paridad-hook.sh` → todo verde (sin `BASE`, el gate se abstiene y da un verde que no mira nada). Más el Cierre de ola.

**Review Focus:**
- Borrar con `git rm` y rutas explícitas: `bash-guards.sh`, `search-first.sh`, `git-c-bash.sh`, `git-add-all-guard.sh`, `verify-before-commit.sh`, `_truncate-payload.sh`, `_zero-residuo-clasifica.sh`, `test-git-c-bash.sh`, `test-git-add-all-guard.sh` y `test-verify-before-commit.sh`. Antes, `grep -rn` de cada nombre en `scripts/`, `plugins/`, `docs/`, `engine/` y `README.md`: no puede quedar ninguna referencia viva.
- `git mv` de `test-bash-guards.sh` y `test-search-first.sh` a `engine/tests/paridad/`, con `HOOK` por defecto = `engine/scripts/hook-wrapper-claude.sh`. Comprobar que no dependen de ficheros hermanos borrados (`$SCRIPT_DIR/_reflex-log.sh` y similares). `paridad-hook.sh` se apunta a la ruta nueva.
- Versiones: plugin `1.31.0` → `1.32.0` (los dos manifests), `ENGINE_MIN` `0.8.0` → `0.14.0`, engine `0.13.0` → `0.14.0`.
- Docs: README y `docs/arquitectura.md` pasan a nombrar `exo hook`. `test-docs-vivos.sh` en verde.

**Notas:** El commit declara lo que no se ha visto correr: el CI de Windows del cutover, el comportamiento de Claude Code ante `exo: command not found` + exit 0 por `|| exit 0`, y que hasta publicar el engine 0.14.0 un `exo` viejo en el PATH deja los guards migrados sin efecto (solo avisa `ENGINE_MIN` en SessionStart). La release del engine se publica **antes** que el plugin 1.32.0.
