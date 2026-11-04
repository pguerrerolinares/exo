# `exo hook` — subproyecto B: Codex

**Fecha:** 2026-11-03 · **Estado:** diseño firmado por el dueño sección a sección (componentes; flujo e instalación; errores y matriz; testing). Cada sección pasó review adversarial de un consultor (Fable) y sus cambios están integrados. Pendiente: builder check y review de la spec escrita.
**Es un delta** sobre `2026-11-02-exo-hook-multi-harness-design.md` (spec A, «A» en adelante). Lo que no se menciona aquí sigue como en A.
**Evidencia:** `2026-11-03-exo-hook-codex/notas-consultores.md` (cinco notas de consultor con verificaciones del padre); `scripts/smoke-codex.sh` y el golden `engine/tests/fixtures/hook/codex/codex-0.166.0-probe.jsonl` (commit `5b0da25`).
**Fuentes leídas en código:** openai/codex HEAD `f7167ac` y `6b2a275` (2026-11-03, posteriores a 0.166.0); binario codex-cli 0.166.0 instalado.

## 1. Qué se quiere

Que alguien pueda **instalar exo y usarlo solo con Codex, sin Claude Code**: memoria (contrato de recall), reglas duras de proyecto, reflejos (bash-guards, search-first), índice al día y los agents `consultor`/`executor`. A dejó en Codex solo los guards migrados; sin recall, índice ni reglas, «exo en Codex» se quedaba en guards sin memoria, que es lo que menos es exo.

Para eso B:
- añade el adapter `codex`, el manifest `.codex-plugin` y `codex.json`;
- **migra a Rust `exo-recall.sh` (SessionStart) y `exo-index.sh` (Stop) en los dos harness** (patrón de A: la entrada pasa a `exo hook claude …` y el `.sh` se borra);
- **mueve todo el estado heredado** de `~/.claude/` a `~/.exo/state/`;
- inyecta en Codex las reglas duras en el `additionalContext` de `SessionStart` y las reinyecta tras compactar;
- instala los agents como roles TOML con `exo setup codex`.

## 2. Hechos medidos que condicionan el diseño

| Hecho | Fuente |
|---|---|
| Codex carga hooks de un plugin instalado vía marketplace (20 eventos) | smoke `5b0da25` |
| §7.1 de A no se cumple: `PostToolUse` Bash dispara con exit≠0 y trae `command` | smoke |
| §7.4 de A no se cumple: un `deny` de `PreToolUse` en hook de plugin impide el Bash, sin `PostToolUse` | smoke (solo con `--dangerously-bypass-approvals-and-sandbox`) |
| El matcher `Write\|Edit` casa `apply_patch`; **el matcher `Agent` no casa el spawn**: en 0.166.0 la tool es `collaborationspawn_agent`; en HEAD, `spawn_agent` con alias `Agent` (`hook_names.rs:46-50`) | smoke, código |
| `SubagentStart`/`SubagentStop` existen; los eventos del hijo traen `agent_id`/`agent_type` | smoke |
| Matchers: patrón solo `[A-Za-z0-9_\|]` → igualdad exacta; si no, regex sin anclar | `hooks/src/engine/matcher.rs:12-22` |
| `SessionStart` acepta `additionalContext` y `systemMessage`; `systemMessage` se guarda como `Warning` y **`codex exec` no lo imprime** | `session_start.rs:233-290`, `:258-262` |
| Timeout de un hook: **se descarta toda la stdout**; `SessionStart` se espera antes del primer turno; default 600 s | `command_runner.rs:369-378`, `hook_runtime.rs:132-178`, `discovery.rs:968` |
| `additionalContext` > 2.500 tokens (≈ `ceil(bytes/4)`) se derrama a fichero y se trunca por el medio; el límite es configurable por entrada (`additionalContextLimit`) | `output_spill.rs:13,70`, `discovery.rs:686` |
| **Compactar dispara `PreCompact`/`PostCompact`; `PostCompact` no puede emitir `additionalContext`** | `core/src/compact.rs:197,225`, `hooks/src/events/compact.rs:46`, schema `post-compact.output` |
| HEAD encola `SessionStart` con `source=compact` tras compactar; en 0.166.0 no hay dato (el golden solo trae `startup`) | `core/src/session/mod.rs:4801` |
| Roles: `[agents.<nombre>]` con `description`, `config_file` (ruta absoluta aceptada) y `nickname_candidates`; el `config_file` aplica `developer_instructions`, `model`, `model_reasoning_effort`, … | `config_toml.rs:808-819`, `role.rs:37-113` |
| El marketplace lee `.claude-plugin/marketplace.json`; `--sparse` se puede repetir; el nombre resultante es `exo@exo` | `codex plugin marketplace add --help`, código |
| Con engine 0.14.0, `exo hook <h> SessionStart` sale rc=0 y vacío (`runner.rs:88`): **§4.3 de A («un engine viejo se nota en SessionStart») es falso**; `--min` desconocido → clap rc=2, que Codex registra como Error | medido |
| Latencias hoy: `exo recall --content` 8 ms, `exo rules` 4 ms, `git log` de la KB 4 ms (0,22 s en frío), `exo hook claude PreToolUse` 2 ms | hyperfine 40 runs |
| Claude Code fija la versión del plugin por sesión (`.in_use/<pid>`): una sesión abierta sigue ejecutando los `.sh` viejos hasta que se cierra. El binario `exo` es compartido | caché `~/.claude/plugins/cache/exo/exo/*/.in_use` |

## 3. Enfoques

**D1 — Estado.** Consultados opus y Fable (segunda opinión ciega); coinciden.
| Vía | Veredicto |
|---|---|
| **(1a) Convención + test de acuerdo**: la misma expresión de ruta en Rust, `_state-dir.sh` y `register.ts`; `exo config --json` la publica y un test compara | **Elegida** |
| (1b) Todo pregunta la ruta al engine | Descartada: sin engine cada `.sh` necesita igualmente una ruta de reserva (recall loguea `no-engine` justo cuando el engine no responde) → dos fuentes de verdad |
| (1c) Leer las dos rutas | Descartada: la ventana no se cierra nunca, porque `reflex-fp-review` necesita todo el histórico |
| (1d) Symlink | Descartada: Codex en Windows corre `cmd /C` con `env_clear`, y los symlinks piden privilegio |

**Migración.** El dueño firmó primero «rename único + aviso en doctor» (Fable) frente a «merge en cada arranque» (opus). La review de la sección 2 mostró que, con el binario compartido y `PreToolUse` ya en Rust, el caso «existen los dos ficheros» aparece en el primer upgrade con una sesión abierta. El dueño cambió a **append + `.migrado-<ts>`** (§4.2).

**D2 — Forma y orden.** Una sola spec con plan en olas. Se descarta partir en B1 (guards en Codex) y B2 (recall), porque B1 entregaría guards sin memoria y partiría una decisión acoplada: el estado lo consumen sobre todo recall y la reafirmación. Port nativo in-process de recall/reglas/config. Orden (Fable): **estado → index → recall → adapter Codex + roles + CI Windows**. Cada ola es su propio PR; si la ola de recall se alarga, la de Codex se desgaja a un plan aparte sin reabrir la spec.

**Reglas tras compactar en Codex.** Opciones: `PostCompact` marca y `UserPromptSubmit` reinyecta (**elegida**); medir antes de decidir; aceptar la pérdida. Se elige la marca porque cierra el hueco hoy, con lo que 0.166.0 sí dispara (§4.6).

## 4. Componentes

### 4.1 Rutas de estado (módulo `state` de la lib)
- `Rutas::desde_env()` resuelve `state_dir`, `reflex_log`, `rules_dir`, `index_log` y `canary` (el canary de `compose-inject.sh:10`, que también es estado de exo).
- **Precedencia por fichero:** variable propia (`REFLEX_LOG_FILE`, `EXO_INDEX_LOG`, `REFLEX_CANARY_FILE`) > `$EXO_STATE_DIR/<x>` > `~/.exo/state/<x>`. El home sale de `dirs::home_dir()`, el mismo anclaje que ya usa `config.rs:57`.
- `Entorno::desde_env` (`evento.rs:16`) pasa a usar `Rutas`.
- `exo config --json` añade `data.state.{dir,reflex_log,rules_dir,index_log,canary}`. Hoy solo trae `embeddings`, `index` y `kb`.
- **`plugins/exo/scripts/_state-dir.sh`** (nuevo) calcula lo mismo en bash. Lo cargan `_reflex-log.sh`, `cobertura-reglas.sh`, `reflex-fp-review.sh`, `document-remind.sh`, `subagent-inject.sh`, `compose-inject.sh` y `scripts/e2e-reglas-proyecto.sh`. `reflex-fp-adjudicate.prompt.md` cambia la ruta en el texto.
- `register.ts` sustituye `${home}/.claude/exo-rules` por la const `${EXO_STATE_DIR ?? home + "/.exo/state"}/exo-rules`.
- `engine/src/walker.rs` y `doctor.rs` dejan de citar `~/.claude/` para el estado.
- `exo-rules/` no se migra (sus ficheros duran como mucho 7 días): solo cambia de ruta. En el cutover puede salir, como mucho, un `sin_ss` (solo log) por sesión viva.

### 4.2 Migración
La hace `exo hook <h> SessionStart` (§5.1, paso 2) para `reflex-log.jsonl`, `exo-index.log` y el canary:
- **Solo existe el viejo** → `rename` al nuevo. Los dos están en el mismo FS, así que es atómico.
- **Existen los dos** → append del contenido del viejo al nuevo en **un único `write()`** sobre el nuevo abierto con `O_APPEND`, y luego `rename` del viejo a `<viejo>.migrado-<ts>`. **Nunca se borra.**
- **Solo existe el nuevo** → no hace nada (cuesta un `stat`).
- Los lectores no dependen del orden de las líneas: `cobertura-reglas.sh:52-67` ordena por `ts` y `reflex-fp-review.sh:21-28` hace `sort`.
- `exo doctor` gana un finding **informativo**: «residuo en `~/.claude/<x>` que crece», con el `cat >>` que lo resuelve. Cubre lo raro, por ejemplo un `state_dir` no escribible, donde ni siquiera el rastro `state-migrate-failed` llega a escribirse.

### 4.3 Adapter `codex` (`engine/src/hook/codex.rs`)
- **`parse`**: el `Evento` neutral de A. Clasificación de la tool:
  - `Bash` → `Shell{command}`.
  - `apply_patch` → `FileEdit{paths}`, con las rutas de las líneas `*** Add File:`/`*** Update File:`/`*** Delete File:` de `tool_input.command`. Si no hay rutas, `Other` (se abstiene, como en A).
  - Nombre que **termina en `spawn_agent`** → `Agent`. Cubre `collaborationspawn_agent` (0.166.0) y `spawn_agent` (HEAD).
  - Cualquier otro → `Other{name}`.
  - Un `source` desconocido de `SessionStart` (HEAD ya trae `fork`) no es error: se trata como `startup`.
- **`render`**:
  - `Allow` → `{}`.
  - `Deny` → `permissionDecision: deny`.
  - `Rewrite` → `allow` + `updatedInput`, como en A.
  - `SessionStart` → `{"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":…},"systemMessage"?}`.
  - `UserPromptSubmit` → `additionalContext`.
  - `PostCompact` → `{}`.
  - Sin campos desconocidos: la salida se parsea con `deny_unknown_fields`.
- **`Avisa` en Codex** se renderiza como `systemMessage` **y además** como línea dentro del `additionalContext`, porque `codex exec` no pinta el `systemMessage`.

### 4.4 `Decision`
- Gana **`Avisa(texto)`**: un aviso para el humano. En Claude es `systemMessage`; en Codex, `systemMessage` + línea de contexto (§4.3).
- Al combinar, los `Avisa` se concatenan; su prioridad es la de `Steer`.
- `SessionStart` y `UserPromptSubmit` no usan el `combinar` de A sino un **combinador propio de fragmentos** (contrato, avisos, reafirmación, reglas), con el cap de §5.1.

### 4.5 Guards nuevos
`Guard` gana variantes. `evento()` deja de ser `PreToolUse` fijo y cada guard declara el suyo. `tools()` pasa a `tools(harness)`, porque hoy devuelve nombres de Claude (`matriz.rs:42-46`). Cada guard declara además **`requiere() -> &[(&str, &str)]`**: los campos que el payload tiene que traer, por ejemplo `("source","compact")`.

| Guard | Evento | Qué hace |
|---|---|---|
| `Index` | Stop | Lanza `exo index --db <db> --json >> index_log` desacoplado (`process_group`/`setsid` en Unix, `DETACHED_PROCESS` en Windows) y vuelve sin esperar. Si falla: `index-fallback reason=detach-failed` |
| `RecallContrato` | SessionStart | `recall_arranque_contenido` in-process con la nota `<kb>/core/core-index`, limit 10, cap-bytes 6144. Extrae la sección `## Contrato de memoria`; si no existe la sección, el bloque entero; si el texto no casa, fallback embebido. `resuelve_kb`/`resuelve_db` bajan de `main.rs:569,583` a la lib |
| `RecallCompact` | SessionStart (`source=compact`) | Reafirmación de reflejos que dispararon en la sesión. Lee el reflex-log vía `Rutas`, filtrando por `session_id`, en las últimas 2000 líneas. Corrige el bug de `exo-recall.sh:161,167`, que ignoraba `REFLEX_LOG_FILE`. Mantiene el id `compact` |
| `RecallSuperpowers` | SessionStart | Aviso si `superpowers@…` sigue habilitado, en los 3 scopes que mira hoy (extiende `doctor.rs:1095`, que solo mira el de usuario) |
| `RecallGatePendiente` | SessionStart (no en compact) | Aviso de gate de `/document` pendiente: los 3 `git` de hoy con **deadline global de 3 s** (`try_wait` + `kill`; el engine no tiene `wait_timeout`). Si vence, sin aviso y `gate-timeout`. Si la KB no tiene git, no hace nada |
| `ReglasTestigo` | SessionStart (no en compact) | `reglas::resuelve(cwd)` y escribe `ss-<sid>` en `rules_dir`, para el `document-remind.sh` de Claude. Avisos: `⚠ sin reglas de proyecto para <repo> (<reason>)` y `⚠ N reglas duras recortadas…`, como `Avisa` |
| `ReglasInyecta` | SessionStart; UserPromptSubmit con marca | Mete las reglas duras (proyecto + globales, techo de 10) en el contexto. Mismos avisos que `ReglasTestigo` |
| `MarcaCompact` | PostCompact | Escribe en el journal `{"kind":"mark","code":"post-compact"}` |

### 4.6 Reglas tras compactar en Codex
- `PostCompact` deja la marca.
- El siguiente **`UserPromptSubmit`** de esa sesión que encuentre una marca sin consumir inyecta reglas + reafirmación **una vez** y escribe `{"kind":"mark","code":"post-compact-consumed"}`.
- Si antes llega un `SessionStart` con `source=compact` (HEAD lo encola), ese `SessionStart` consume la marca: la consume el primero que llegue, y nunca hay doble inyección.
- El journal (§4.2 de A) registra además `UserPromptSubmit` y `PostCompact`, y las marcas.

### 4.7 Engine mínimo
- La entrada de SessionStart pasa a **`exo hook <h> SessionStart --min <ENGINE_MIN>`**, sin `|| exit 0`.
- Un engine que no conoce `--min` falla en clap (rc=2) y se nota una vez por sesión, también en Codex como Error.
- Un engine que conoce `--min` pero es más viejo que `X` devuelve `Avisa("engine desactualizado (V < X) — actualiza el binario instalado")` + fallback embebido. En Codex se añade «sin reglas duras», porque nadie más las mete. Se salta la migración y el testigo.
- Sustituye al check de `_engine-version.sh` de exo-recall. `_engine-version.sh` se borra si nadie más lo carga.
- `ENGINE_MIN` sube a **0.15.0**, la versión que trae B.

### 4.8 Reflex-log
- Campo nuevo **`harness`**. En Rust sale del `Evento`; `_reflex-log.sh` escribe `"claude"`; ausente se lee como claude. Es aditivo: jq ignora los campos que no conoce.
- Los ids de reflejo y `reason` de hoy se mantienen (`recall-fallback`, `recall-truncated`, `project-rules-skip` con `reason` en `[a-z_]+`, `compact`, `index-fallback`, `hook-panic`).
- Ids nuevos: `state-migrate-failed`, `gate-timeout`, `context-spill-risk`, `project-rules-emitted bytes=N` (solo Codex).
- Desaparece `recall-fallback reason=no-engine`: el binario es el propio hook.
- Medido: `cobertura-reglas.sh` (`:51-56`) y `reflex-fp-review.sh` (`:21-28`) no se rompen.

### 4.9 Ficheros del plugin y `exo setup codex`
- **`plugins/exo/.codex-plugin/plugin.json`**: `name`, `version`, `description`, `"hooks": "./hooks/codex.json"`. `skills` no se declara, porque es la ruta por defecto.
- **`plugins/exo/hooks/codex.json`**, escrito a mano (§5.4).
- **`plugins/exo/agents/codex/{consultor,executor}.toml`**: capas de rol (`developer_instructions`, `model`, `model_reasoning_effort`). El contenido sale de `agents/consultor.md` y `agents/executor.md`, adaptado a la sintaxis de Codex (`spawn_agent` en lugar de `Agent`). El engine los embebe con `include_str!`, así que el texto de los roles va versionado con el engine.
- **`exo setup codex`** (subcomando nuevo):
  - Escribe las copias en `~/.exo/codex-agents/` y el bloque `[agents.exo-consultor]`/`[agents.exo-executor]` (`description`, `config_file` absoluto a esas copias) en `~/.codex/config.toml`, entre `# >>> exo` y `# <<< exo`.
  - Es idempotente: reescribe el bloque y nunca lo duplica. Respeta lo que hay fuera de los marcadores. Escribe con tmp + rename, porque Codex también persiste ese fichero de forma atómica (`core/src/config/edit.rs:25`).
  - Si el config no parsea o hay un marcador sin pareja, **falla en voz alta** (exit≠0 con el motivo) y no toca nada.
  - `install.sh`/`install.ps1` lo invocan solo si encuentran `codex` en el `PATH`, con el patrón de `doctor` (`|| ec=$?`, `install.sh:119-123`): si falla, avisan «setup codex falló (exit N): corre `exo setup codex`» y la instalación sigue.
- **Se borran:** `exo-recall.sh`, `exo-index.sh`, `test-exo-recall.sh`, `test-exo-recall-golden.sh`, `test-exo-recall-superpowers.sh`, `test-exo-index.sh` (su contenido pasa a §7), y `_engine-version.sh` si queda sin usuarios.

## 5. Flujo e instalación

### 5.1 `exo hook <h> SessionStart --min X`
1. **Versión.** Si el engine es más viejo que `X`: `Avisa` + fallback (§4.7), `recall-fallback reason=engine-stale`, y termina.
2. **Migración** (§4.2). Si falla, sigue (`state-migrate-failed`).
3. **Parse**: `source`, `cwd`, `session_id`.
4. **Guards** de SessionStart que la matriz da como S/Degradado para el harness, en este orden: `RecallContrato` → `RecallCompact` (si `compact`) → `RecallSuperpowers` → `RecallGatePendiente` (no en compact) → `ReglasTestigo`/`ReglasInyecta` (no en compact en Claude; en Codex, `ReglasInyecta` también con `compact` y consume la marca).
5. **Cap.**
   - **Claude:** contrato + avisos a 1500 B por el final sin partir UTF-8; la reafirmación va detrás, fuera del cap. Es lo de hoy.
   - **Codex:** orden contrato (cap 1500 B) → reglas → reafirmación y avisos **al final**, porque Codex trunca por el medio. Las reglas no se recortan por bytes: ya tienen el techo de 10.
6. **Render** (§4.3). En Codex, si el `additionalContext` pasa de `4 × additionalContextLimit` bytes, el reflex-log anota `context-spill-risk bytes=N`. En Codex también se anota `project-rules-emitted bytes=N`.
7. **Exit 0.**

**Tiempo:** todo es in-process salvo el gate pendiente (deadline de 3 s). Las entradas de SessionStart llevan `"timeout": 10` en los dos ficheros. Hoy la de `hooks.json` no lleva ninguno.

### 5.2 `exo hook <h> Stop`
- Corre el guard `Index` y vuelve.
- En Claude, la entrada de `exo-index.sh` pasa a `exo hook claude Stop || exit 0`. `sdd-wake-guard`, `document-remind` y `delegar-decision` siguen como `.sh`.
- En Codex es la única entrada de `Stop`.

### 5.3 Pre/PostToolUse, UserPromptSubmit y PostCompact en Codex
- **Pre/PostToolUse:** igual que en A. El journal registra `ok: null`; verify está Degradado (solo log); search-first cuenta el `exo search` de `Bash`; `FileEdit` y `Agent` los clasifica el adapter.
- **`UserPromptSubmit`:** solo hace algo si hay una marca `post-compact` sin consumir (§4.6).
- **`PostCompact`:** escribe la marca.

### 5.4 `codex.json`
```json
{"hooks":{
  "SessionStart":[{"hooks":[{"type":"command","command":"exo hook codex SessionStart --min 0.15.0","timeout":10,"additionalContextLimit":2500}]}],
  "UserPromptSubmit":[{"hooks":[{"type":"command","command":"exo hook codex UserPromptSubmit || exit 0","timeout":5,"additionalContextLimit":2500}]}],
  "PreToolUse":[{"matcher":"^(Bash|apply_patch|collaborationspawn_agent|spawn_agent)$",
                 "hooks":[{"type":"command","command":"exo hook codex PreToolUse || exit 0","timeout":5}]}],
  "PostToolUse":[{"matcher":"^(Bash|apply_patch)$",
                  "hooks":[{"type":"command","command":"exo hook codex PostToolUse || exit 0","timeout":5}]}],
  "PostCompact":[{"hooks":[{"type":"command","command":"exo hook codex PostCompact || exit 0","timeout":5}]}],
  "Stop":[{"hooks":[{"type":"command","command":"exo hook codex Stop || exit 0","timeout":5}]}]
}}
```
Los matchers usan los nombres reales de las tools; el de `Agent` no casa el spawn. Antes de fijarlo en el plan se comprueba en el código de 0.166.0 el nombre exacto y la ubicación de `additionalContextLimit` (en la entrada o dentro del handler).

### 5.5 Cambios en `hooks.json` (Claude)
- La entrada SessionStart de `exo-recall.sh` → `exo hook claude SessionStart --min 0.15.0`, con `"timeout": 10`. `estilo-directo.sh` sigue igual.
- La entrada Stop de `exo-index.sh` → `exo hook claude Stop || exit 0`, con `"timeout": 5`.
- En Claude no se añaden `UserPromptSubmit` ni `PostCompact`: las reglas las lleva el mod y la reafirmación, el `SessionStart(compact)` de Claude.

### 5.6 Instalación en Codex (pasos del usuario)
1. `install.sh` (o `install.ps1`): instala el engine y, si hay `codex` en el `PATH`, corre `exo setup codex`.
2. `codex plugin marketplace add pguerrerolinares/exo --sparse plugins/exo --sparse .claude-plugin`.
3. `codex plugin add exo@exo`.
4. Aceptar la confianza de los hooks en el primer uso.

Los hooks corren con el `$SHELL` de la sesión y heredan su `PATH` (A §4.4): Codex lanzado desde un IDE con un `PATH` mínimo no encuentra `exo`, y §4.7 hace que se note. El README, `docs/instalacion.md` y `docs/arquitectura.md` documentan estos pasos y quitan la línea «hasta B, instalar el plugin en Codex no hace nada».

### 5.7 Versiones
`scripts/test-versiones.sh`:
- comprueba que `plugins/exo/.codex-plugin/plugin.json` `.version` es igual a las otras dos;
- comprueba que los `--min` de `hooks.json` y `codex.json` son iguales a `ENGINE_MIN`;
- **cambia la guardia literal `ENGINE_MIN == 0.14.0` (`:71-75`) a `0.15.0`**, con el porqué (B mueve el estado, y hooks.json pasa a necesitar `--min` y `exo hook … Stop`).

## 6. Errores y matriz

### 6.1 Errores
La postura de A §5 no cambia: todo degrada y deja rastro en el reflex-log.

| Error | Respuesta | Rastro |
|---|---|---|
| Falla el rename o el append de la migración | Se sigue en la ruta nueva; el viejo queda intacto | `state-migrate-failed` (si el `state_dir` no es escribible, tampoco esto: lo cubre doctor) |
| No hay índice, no hay config, el bloque viene vacío o le falta «Contrato de memoria» | Fallback embebido; en Codex, «sin reglas duras» si las reglas tampoco salen | `recall-fallback reason=no-index\|no-config\|empty\|no-contract` |
| El bloque llega truncado por cap-bytes | Se sirve igual | `recall-fallback reason=truncated` |
| Resolución de reglas: error, ambigua o sección vacía | `Avisa("⚠ sin reglas de proyecto…")` | `project-rules-skip reason=error_engine\|ambigua\|seccion_vacia\|…` |
| Más de 10 reglas | Recorte + `Avisa` | — |
| Vence el deadline de git | No sale el aviso de gate | `gate-timeout` |
| Falla el detach del index | Nada visible | `index-fallback reason=detach-failed` |
| Engine más viejo que `--min` | `Avisa` + fallback (§4.7) | `recall-fallback reason=engine-stale` |
| Panic en un guard | Se descarta ese guard (`catch_unwind`) | `hook-panic` |
| `additionalContext` de Codex > 4×límite | Se sirve igual | `context-spill-risk bytes=N` |
| `exo setup codex`: config imposible de parsear o marcador huérfano | exit≠0 con el motivo, sin tocar nada; `install.sh` avisa y sigue | stderr |

### 6.2 Matriz
Solo guard × harness, con `match` sin comodín. Las capacidades de render (`Avisa`) no son filas: las cubren `decisiones()` y el test de render.

| Guard | claude | codex |
|---|---|---|
| GitC, AddA, ZeroResiduo | Soportado | Soportado |
| Verify | Soportado | Degradado (`ok: null`, solo log) |
| SearchFirst | Soportado | Soportado |
| Index | Soportado | Soportado |
| RecallContrato | Soportado | Soportado |
| RecallCompact | Soportado | NoSoportado («0.166.0 compacta con PreCompact/PostCompact; ningún SessionStart del golden trae source=compact») |
| RecallSuperpowers | Soportado | NoSoportado (lee `~/.claude/settings.json`) |
| RecallGatePendiente | Soportado | Soportado |
| ReglasTestigo | Soportado | NoSoportado (nadie lee `ss-<sid>`) |
| ReglasInyecta | NoSoportado (lo hace el mod) | Soportado |
| MarcaCompact | NoSoportado (no hace falta) | Soportado |

En Codex la reafirmación tras compactar viaja con `ReglasInyecta` vía la marca (§4.6). Los `.sh` no migrados van en el informe de §6.3, no en la matriz.

### 6.3 Tests de capacidad y coherencia (los de A, ampliados)
- **Capacidad:** para cada guard S/Degradado en un harness, el golden de ese harness tiene que traer **una línea que case el evento y los campos de `requiere()`**. Si no, falla con un finding nombrado, por ejemplo: «RecallCompact declarado Soportado en codex pero ningún SessionStart del golden trae source=compact: decláralo NoSoportado o recaptura el golden». Además, `render(d, harness)` produce salida para cada `d` de `decisiones()`.
- **Coherencia** (`hook_coherencia.rs`, que hoy asserta `harness == claude` y solo lee `hooks.json`, `:95`): gana un `entradas()` de `codex.json`.
  - Cada guard S/Degradado tiene una entrada cuyo matcher cubre sus `tools(harness)`, con la semántica de matcher de cada harness (Codex: exacto si `[A-Za-z0-9_|]`, regex sin anclar si no).
  - **Cada entrada `exo hook <h> <ev>` tiene ≥1 guard S/Degradado en `(h, ev)`.** Sustituye a «ninguna entrada apunta a un NoSoportado», que con sub-guards de soporte mixto fallaría siempre.
- **Informe** (no fallo), para ver lo no declarado y no solo las excepciones:
  - los guards NoSoportado que cada entrada ignora;
  - las entradas `.sh` de `hooks.json`;
  - las tools del golden de Codex que ningún matcher de `codex.json` cubre (hoy `collaborationwait_agent`, `web_search`).
- `scripts/test-hooks-json.sh` gana un gemelo para `codex.json`, con los eventos válidos de Codex (`PostCompact`, `UserPromptSubmit`).

## 7. Testing

1. **Estado.**
   - Test de acuerdo: `exo config --json | jq -r .data.state.*` frente a `bash -c '. _state-dir.sh; …'`, en tres casos (sin env, con `EXO_STATE_DIR`, con `REFLEX_LOG_FILE`). Corre en el CI de Linux y en `test-windows` (con `cygpath -u`).
   - `register.ts`: no hay bun en CI, así que se valida con un grep del literal en el test de acuerdo.
   - Migración, unitarios: solo viejo → rename; los dos → append + `.migrado-<ts>`; solo nuevo → nada.
   - Conservación: 4 appenders × 200 líneas por `_reflex-log.sh` durante el merge → 0 líneas perdidas y `jq -c .` válido en todas.
2. **Recall: gate de paridad baseline/divergencias**, el patrón de `engine/scripts/paridad-hook.sh:36-45`.
   - Los **61** `pass` de hoy (52 de `test-exo-recall.sh`, 2 de `-golden`, 7 de `-superpowers`) se congelan en `engine/tests/fixtures/hook/baseline-recall.txt`.
   - Cada línea aparece como id en un `#[test]` de Rust o en `divergencias-recall.txt` con su motivo. El script falla si sobra o falta alguna.
   - Los ~11 casos de fontanería por stub (`no-engine`, `no-config` por binario, `engine-stale` por stub, timeout de rules, stderr de error) van a divergencias.
   - **Fixture:** `tempfile` + `indexa(kb, &db)` + `common::con_config` (`engine/tests/recall.rs:50,104`). Descarga ONNX, así que corre en el job `test`, no en `plugin-tests`.
   - El escenario 1 del golden (stubea `exo recall` por `EXO_BIN`) no sobrevive al port: lo sustituye un test Rust que exige que el `additionalContext` empiece por `## Contrato de memoria` con la KB fixture. El escenario 2 (reafirmación) se porta.
3. **Index.** Sin reloj: el `exo index` stub se bloquea en un lock que controla el test. El hook vuelve, el test suelta el lock y aparece el log. Fallo de detach → `index-fallback`. **El detach en Windows solo lo ve `test-windows`, que corre en push a main con cambios en el engine (`ci.yml:352-353`): en los PR no se ve.**
4. **Adapter `codex`.**
   - Parsea cada línea del golden 0.166.0.
   - Clasifica `Bash`, `apply_patch` (rutas) y `*spawn_agent`.
   - Render con forma exacta, validado contra 6 output schemas de HEAD vendorizados con su hash en `engine/tests/fixtures/hook/codex/schema/` (dev-dep `jsonschema`). Validan HEAD, no 0.166.0, y se dice así.
5. **Matriz:** capacidad y coherencia de §6.3.
6. **Post-compact:**
   - `PostCompact` → el siguiente `UserPromptSubmit` inyecta una vez y el segundo no.
   - Con `SessionStart(compact)` antes del UPS, se consume la marca.
   - Una marca de otra sesión no se toca.
7. **`exo setup codex`:**
   - Idempotente: dos ejecuciones dan el mismo fichero.
   - Respeta lo que hay fuera de los marcadores.
   - Config imposible de parsear o marcador huérfano → exit≠0 sin tocar nada.
   - El TOML escrito parsea.
   - `install.sh` sigue aunque falle.
8. **CI de Windows:** `cmd /C "nope || exit 0"` y `cmd /C "exo hook codex PreToolUse || exit 0"` → 0. Codex pasa la línea a `cmd /C` tal cual (`command_runner.rs:455-459`).
9. **`test-versiones.sh`** (§5.7).
10. **`smoke-codex.sh --real`**, fuera de CI y corrido por el dueño, con el plugin del repo instalado. Comprueba:
    - que el contexto de `SessionStart` trae el contrato y las reglas;
    - que `git add -A` se deniega;
    - que el `systemMessage` **no** aparece en `exec.log` (aserción, no invalidación);
    - compactación forzada con `-c model_auto_compact_token_limit=3000` y luego `codex exec resume --last "di ok"`: `jq -r '.hook_event_name+" "+(.source//"-")' ev.jsonl | sort | uniq -c` tiene que mostrar `PostCompact` y 2× `UserPromptSubmit`, y el UPS posterior trae las reglas.

    El modo de volcado sigue regenerando el golden. El smoke depende de que el modelo obedezca y no es red de regresión.
11. **Latencia:** `hyperfine -N --export-json` sobre `exo hook codex SessionStart --min 0.15.0` y sobre `exo hook claude SessionStart --min 0.15.0`, contra la KB real del dueño.

12. **Golden recapturados (firmado por el dueño).** Antes de la ola de matriz:
    - El modo de volcado de `smoke-codex.sh` fuerza una compactación (`-c model_auto_compact_token_limit=3000`) y un segundo turno (`codex exec resume --last`), para que el golden de Codex traiga `PostCompact`, `UserPromptSubmit` y, si existe, `SessionStart` con `source=compact`.
    - Un `scripts/smoke-claude.sh` nuevo captura con hooks de volcado un golden de Claude con `SessionStart` (`startup` y `compact`, este último forzando `/compact` o por umbral) y `Stop`.
    - Los dos los corre el dueño, fuera de CI. **Es una tarea del dueño que bloquea la ola de matriz.** El test de capacidad no admite excepciones.

## 8. Condiciones de invalidación (pre-registradas)

1. **Rutas en Windows.** En `test-windows`, el test de acuerdo da rutas distintas entre `dirs::home_dir()` y el `$HOME` de Git Bash → `install.ps1` fija `EXO_STATE_DIR` explícito en el entorno del usuario.
2. **Conservación.** El merge con 4 appenders pierde o corrompe alguna línea → se rehace la migración.
3. **Paridad de recall.** El gate baseline/divergencias no da el 100 % → **bloquea el merge** de la ola de recall.
4. **Semántica del port.** Más de 18 de los 61 casos (30 %) acaban como divergencia → el port no conserva la semántica: se porta función a función con la suite bash viva, en vez de sustituirla.
5. **Latencia.** El p99 de `exo hook <h> SessionStart` sobre la KB real pasa de 300 ms → el gate pendiente sale de `SessionStart`.
6. **Compact visible.** El smoke forzando compactación ve `SessionStart` con `source=compact` → `RecallCompact` pasa a Soportado en Codex y la marca de `PostCompact` se queda como red.
7. **Post-compact medible.** `codex exec resume --last` no produce un segundo `UserPromptSubmit` → el flujo post-compact queda sin dato y la condición 6 se retira. Si el smoke no ve ni `PostCompact` ni el UPS reinyectado, las reglas tras compactar pasan a NoSoportado en Codex y se documenta.
8. Las de A §7.4 (distribución por plugin) y §7.5 (Windows `|| exit 0`) se mantienen. La 7.4 ya está medida y no se cumple.

## 9. Fuera de alcance y declarado

- Siguen como `.sh` solo en Claude: `estilo-directo`, `delegar-decision`, `document-remind`, `sdd-wake-guard`, `subagent-inject`, `clean-orchestrator-research`, `subagent-bg-guard` y `red-gate`. En Codex son NoSoportado. En particular, **Codex no tiene `subagent-inject`**, aunque `SubagentStart` acepta `additionalContext`: es candidato a la siguiente iteración.
- **Skills que ejecutan `.sh`** (`distill` corre `cobertura-reglas.sh`; `orchestrate`, `scripts/review-package`): en Codex sobre Unix los ejecuta el agente con su Bash y funcionan con `_state-dir.sh`. En Codex sobre Windows fallan. Su texto sigue hablando de `Agent`/`subagent_type`: B solo instala los roles TOML.
- El deny de `PreToolUse` en Codex solo se midió con `--dangerously-bypass-approvals-and-sandbox`.
- No hay dato de cómo pinta la TUI de Codex los `Warning`/`Error` de hooks.
- Si `config_file` exige permisos concretos (`read_sensitive_file_to_string`): sin dato.
- `web_search` de Codex corre en el servidor de OpenAI y no dispara hooks.
- pi (C) tiene su propia spec.

## 10. Decisiones del autor (Claude), corregibles

1. `Avisa` tiene la prioridad de `Steer` al combinar.
2. Umbral de `context-spill-risk` = 4 × `additionalContextLimit`.
3. Un `source` desconocido de `SessionStart` se trata como `startup`.
4. La marca post-compact la consume el primero que llega (`SessionStart(compact)` o UPS).
5. Los roles TOML se derivan de `agents/*.md` con la sintaxis de Codex. Llevan solo `developer_instructions`, `model` y `model_reasoning_effort`.
   - **Modelo (firmado por el dueño: equivalente en OpenAI).** Sonnet → `gpt-6.1-sol` («workhorse»), Opus/Fable → `gpt-6-astra` («frontier»), Haiku → `gpt-6-luna`, según `~/.codex/models_cache.json` del 2026-11-03. Los dos agents de Claude dicen `model: sonnet`, así que los dos roles usan `gpt-6.1-sol`. Esfuerzo (decisión mía): consultor `high`, executor `medium`.
   - **Solo lectura del consultor:** la capa de rol de Codex no admite sandbox (`role.rs:37-113`), así que la restricción va como instrucción en `developer_instructions`. No se puede forzar; queda declarado.
   - **Modelo rotado:** `exo doctor` gana un finding si el `model` de un rol no aparece en `$CODEX_HOME/models_cache.json`.
6. `exo-rules/` no se migra, solo cambia de ruta.

### 10.1 Builder check (decisiones del autor)
7. **Reafirmación compartida.** Es una sola función, `reafirmacion(sid, rutas)`. La usa `RecallCompact` (Claude) y la usa `ReglasInyecta` cuando consume la marca, venga del UPS o de un `SessionStart(compact)` de Codex. En Codex, toda inyección tras compactar lleva reglas + reafirmación.
8. **Doble inyección.** No se añade un consumo atómico. En Codex, `SessionStart` se espera antes del primer turno (`hook_runtime.rs:132-178`), así que no corre a la vez que un UPS de la misma sesión. Queda documentado «como mucho dos inyecciones» (misma clase que la divergencia de tools en paralelo de A §6.4).
9. **`CODEX_HOME`.** `exo setup codex` escribe en `$CODEX_HOME/config.toml` y, si no está definido, en `~/.codex/config.toml`. Los `config_file` van siempre a `~/.exo/codex-agents/`, junto a `index.db`. No dependen de `EXO_STATE_DIR`, porque no son estado.
10. **Seams que sobreviven al port.** `EXO_INDEX`, `EXO_KB`, `EXO_KB_NAME`, `EXO_RECALL_NOTA`, `EXO_RECALL_CAP`, `RECALL_BLOQUE_CAP`, `EXO_RECALL_LIMITE`, `EXO_RECALL_COMPACT_LINEAS`, `EXO_GIT_TIMEOUT` (pasa a ser el deadline global) y `CLAUDE_PROJECT_DIR`. `EXO_BIN` solo sirve ahora para el guard `Index`: lanza `EXO_BIN` si está definido y, si no, `std::env::current_exe()`. Es el seam del stub de §7.3. Desaparece `EXO_RULES_TIMEOUT`, porque las reglas son in-process. Los casos de los 61 que dependían de `EXO_BIN` como stub del engine van a divergencias y cuentan para la condición 4.
11. **Texto de las reglas en Codex.** Rust tiene su copia del preámbulo y de las cabeceras («Reglas duras del dueño», «Reglas duras del proyecto»). Un test hace grep de esos literales en `register.ts` (`:2`, `:29`) y falla si divergen.
12. **`exo setup codex` y las tablas.**
    - El bloque se añade al final del fichero la primera vez. Si los marcadores ya existen, se reescribe en su sitio.
    - Falla en voz alta si existe `[agents.exo-consultor]` o `[agents.exo-executor]` fuera de los marcadores.
    - También falla si el resultado, al parsearlo, cambia el valor de cualquier clave ajena al bloque. Por ejemplo, contenido sin cabecera detrás de `# <<< exo`, que quedaría dentro de la última tabla del bloque.
13. **`additionalContextLimit`.** Antes de fijar `codex.json`, el plan comprueba el nombre y la ubicación de la clave en el tag `rust-v0.166.0` de openai/codex. Además, el modo de volcado del smoke instala un `codex.json` con la clave y comprueba que Codex sigue cargando los hooks: Codex parsea con `deny_unknown_fields`, y una clave mal puesta lo tumba todo. Si no existe en 0.166.0, se quita y el canario usa 2.500 tokens.
14. **`ENGINE_MIN`.** `plugins/exo/ENGINE_MIN` sigue siendo la única fuente de verdad, y la leen `doctor.rs:222` y `test-versiones.sh:56`. `SessionStart` no lee el fichero: recibe `--min` en la línea de la entrada. `test-versiones.sh` exige que los `--min` de `hooks.json` y `codex.json` sean iguales al fichero.
