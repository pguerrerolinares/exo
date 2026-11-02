# `exo hook`: capa multi-harness de hooks (Claude Code → Codex → pi)

**Fecha:** 2026-11-02 · **Estado:** diseño firmado por el dueño sección a sección (arquitectura; flujo e instalación; errores, matriz y testing). Las secciones 2 y 3 se revisaron con un consultor (Fable) y las firmó el dueño. Pendiente: review de la spec escrita.
**Evidencia:** `2026-11-02-exo-hook-multi-harness/notas-consultores.md` (las tres notas de consultor, más las mediciones) y `2026-11-02-exo-hook-multi-harness/codex-0.166.0-probe.jsonl` (11 payloads reales de Codex).
**Fuentes leídas en código:** openai/codex HEAD `9da0765` (2026-11-02); sondera-ai/sondera-coding-agent-hooks (MIT), de donde se toma el diseño y no el código; pi `@earendil-works/pi-coding-agent` v1.5.0, solo por docs.

## 1. Qué se quiere

Hoy exo solo funciona en Claude Code. El dueño quiere que **alguien pueda instalar exo y usarlo solo con Codex, sin Claude Code**. Después vendrá pi. El patrón elegido es el de claude-mem y sondera: un único punto de entrada, `exo hook <harness> <evento>`, dentro del engine Rust, con un adapter fino por harness.

Se reparte en tres subproyectos, cada uno con su ciclo spec→plan:

- **A — Núcleo.** Incluye `exo hook`, el journal, la matriz, el adapter de Claude y la migración de `bash-guards` y `search-first` a Rust. **No cambia el comportamiento en Claude**, salvo las divergencias listadas en §6.4.
- **B — Codex.** Adapter de Codex, `.codex-plugin`, `codex.json`, smoke e2e y agents en TOML.
- **C — pi.** Una extensión TS fina que lanza `exo hook pi <ev>`. Necesita su propia spec, porque pi no tiene hooks de shell ni subagentes nativos.

**Esta spec detalla A y fija las interfaces que B y C consumen.** Lo que es de B se marca como **[B]**.

## 2. Por qué esta forma (enfoques)

| Vía | Veredicto |
|---|---|
| 1. Los guards bash siguen siendo la lógica y un adapter Rust adapta el payload al formato de Claude | Descartada. Codex en Windows lanza los hooks con `cmd.exe /C` y `env_clear` (`hooks/src/engine/command_runner.rs:444-500`), donde un `.sh` no corre. Un usuario de Codex en Windows tendría que instalar Git Bash + jq + python3. Además seguiría dependiendo del transcript de cada harness. |
| **2. La lógica pasa a Rust guard a guard, precedida de un journal propio** | **Elegida**, por consultor (opus) y firmada por el dueño. Lo que ata exo a Claude es el transcript y la dependencia de bash/jq, no la forma del payload, que Codex acepta casi tal cual. |
| 3. Comandos externos contra un esquema neutral de exo | Descartada: versiona un esquema que solo consume exo y mantiene toda la deuda de bash. |

Al medir resultó que solo **4** scripts parsean de verdad el transcript (`bash-guards`, `search-first`, `delegar-decision(.py)`, `subagent-bg-guard`), no 7. Los guards enganchados suman unas 1000 líneas efectivas más 137 de Python, con 110 llamadas a `jq`.

## 3. Arquitectura (módulo `engine/src/hook/`)

| Componente | Contrato |
|---|---|
| `exo hook <harness> <evento>` | Subcomando clap. Lee stdin, pasa por `parse → journal → guards → combinar → render` y escribe en stdout. **Siempre sale con exit 0.** |
| `Evento` | Estructura neutral respecto al harness. Campos: `harness`, `session_id`, `cwd`, `kind` (evento) y `tool`, que es la tool clasificada (`Shell{command}`, `FileEdit{paths}`, `Agent`, `Web`, `Mcp{name}`, `Other{name}`) junto con su nombre e input crudos. Además `prompt`, `last_assistant_message`, `stop_hook_active`, `agent_id` y `agent_type`; este último hace falta para el reflex-log, `_reflex-log.sh:19-25`. El clasificador **se abstiene**: un `FileEdit` cuyas rutas no se pueden extraer pasa a `Other`. En Codex, las rutas de `apply_patch` se sacan de las líneas `*** Add/Update/Delete File:` del patch en `tool_input.command`. |
| Adapter `claude` (A), `codex` [B], `pi` [C] | Dos funciones: `parse(&[u8]) -> Result<Evento>` y `render(Decision, &Evento) -> Salida`. **En Codex, `Allow` se renderiza como `{}`**: un `allow` o `ask` explícito en `PreToolUse` es inválido (`output_parser.rs:452-459`), y la salida no puede llevar campos desconocidos. |
| `Decision` | `Allow`, `Deny(razón)`, `Rewrite{input, razón}`, `Steer(contexto)` o `Block(razón)` (este último en Stop/SubagentStop). `Rewrite` es el reescrito de git-c (`bash-guards.sh:166-176`): en Claude se renderiza como `allow` + `updatedInput`; en Codex también, porque `allow` sin `updatedInput` es inválido y con él sí es válido (`output_parser.rs:443-455`). Prioridad al combinar: `Deny`/`Block` > `Rewrite` > `Steer` > `Allow` (igual que `bash-guards.sh:316-321`). Las razones de `Deny` se concatenan y los `Steer` también. |
| Guard | Función pura `(&Evento, &Journal) -> Decision`. Cada guard declara qué evento necesita y qué decisiones puede devolver. |
| Journal | Ver §4.2. |
| Matriz | `match (Guard, Harness) -> Soporte` **sin comodín `_`**: si falta declarar una combinación, no compila. Los valores son `Soportado`, `Degradado(motivo)` y `NoSoportado(motivo)`. |
| Estado | **En A solo hay un estado nuevo, el journal**, en `~/.exo/state/`, que se puede redirigir con **`EXO_STATE_DIR`**. El estado heredado **no se mueve en A**: `reflex-log.jsonl`, `exo-rules` y `exo-index.log` siguen en `~/.claude/`. Moverlos solo en parte dejaría el log partido entre escritores Rust y `.sh` (unos 10 scripts más `register.ts`). El código Rust escribe el reflex-log en la ruta actual y **respeta `REFLEX_LOG_FILE`**, el contrato que ya usan los tests. **[B]** mueve todo el estado heredado a la vez, a través de un único resolvedor de ruta que usan Rust, los `.sh` y `register.ts`, porque un usuario que solo tiene Codex no tiene `~/.claude`. |

**Guards que no se migran.** Su entrada en `hooks.json` sigue llamando a su `.sh` directamente, sin pasar por `exo hook`. Cuando un guard se migra, su entrada pasa a `exo hook claude …` y el `.sh` se borra. **A Codex solo va lo que ya está migrado.**

**Declarados `NoSoportado` en Codex, sin intentar emularlos:** `subagent-bg-guard`, que busca `Monitor started`/`TaskStop` en el transcript del subagente (pura semántica de Claude), y `red-gate`, porque `SubagentHandback` no existe en Codex. `register.ts` queda fuera del alcance: usa la API de mods de Claude y ya llama a `exo rules --json`. En Codex, su equivalente sería `UserPromptSubmit` + `additionalContext` [B].

## 4. Flujo de datos e instalación

### 4.1 Una invocación

1. `adapter.parse(stdin)`. Si falla, se devuelve `Allow` y se registra un aviso (§5).
2. Si el evento es `PostToolUse` o `PostToolUseFailure`, se hace `journal.append` en modo fail-open. `UserPromptSubmit` no se registra en A, porque ningún guard migrado lo usa; entrará cuando migre `delegar-decision`.
3. Se ejecutan los guards de ese evento que la matriz marca como `Soportado` o `Degradado` para el harness, y sus decisiones se combinan.
4. `adapter.render` escribe stdout y el proceso **sale con 0**.
5. Se escribe una línea en `reflex-log.jsonl`, con el mismo esquema que hoy, para que `cobertura-reglas.sh` y `reflex-fp-review.sh` sigan funcionando.

### 4.2 Journal

- Hay un fichero por sesión: `$EXO_STATE_DIR/sessions/<harness>-<session_id>.jsonl`. Solo se añaden líneas.
- Cada línea guarda el **evento canónico**, no el payload crudo: `{ts, kind, tool, cmd truncado a ~200 B o paths, ok: bool|null, agent_id?}`, más registros `{"kind":"warn","code":…}` para los avisos que solo se emiten una vez por sesión.
- **`ok`** vale `true` para un `PostToolUse` de Claude, `false` para un `PostToolUseFailure` de Claude y `null` en Codex, donde no hay exit code. Si no se registraran los fallos, la rama verify daría un falso verde en la secuencia «verde → rojo → commit»: hoy avisa, y con un journal que solo guardara éxitos callaría (builder check, pregunta 1).
- **Regla de escritura: una línea = una llamada `write()`** sobre un fichero abierto con `O_APPEND` (en Rust, `OpenOptions::append`). Se midió con 8 procesos × 300 escrituras de 6 KB en ext4/WSL2 y salieron 0 líneas corruptas. `PIPE_BUF` no tiene nada que ver aquí, porque solo afecta a pipes.
- **Lectura acotada:** como mucho los últimos 2 MiB. Si se trunca, el guard lo sabe.
- **GC perezosa:** cuando se crea el fichero de una sesión nueva, se borran los de más de 14 días. No hace falta una entrada `SessionStart` solo para eso.
- **Lectura filtrada por `agent_id`:** todos los guards leen únicamente los registros de su propio contexto (el padre, los del padre; un subagente, los suyos), igual que hace hoy la racha de zero-residuo (`bash-guards.sh:233-237`). El oráculo de paridad dirá si eso iguala el comportamiento actual de verify dentro de un subagente.
- **Qué entra en el journal:**
  - En Claude, `PostToolUse` solo se dispara cuando la tool tuvo éxito y los fallos van a `PostToolUseFailure`. Los dos se registran.
  - En Codex, `PostToolUse` se dispara también con exit≠0, y en Bash `tool_response` es solo el stdout, **sin exit code** (medido: `false` y `exit 3` devuelven `""`). Por eso, en Codex, un registro de Bash en el journal **no prueba que el comando saliera bien** (`ok: null`).

### 4.3 Exit 0 y engine viejo

Codex bloquea con exit 2 en `PreToolUse`/`PostToolUse` siempre que stderr no esté vacío (`pre_tool_use.rs:268-282`). Un engine que no conoce el subcomando `hook` hace que clap escriba en stderr y salga con 2, así que **bloquearía todos los Bash**. Por eso:

- Las entradas de eventos de tool se escriben como **`exo hook <h> <ev> || exit 0`**. Funciona en `sh`/`zsh -lc` y en `cmd /C`, donde `true` no existe.
- Las entradas de **`SessionStart` van sin `|| exit 0`**. Ese evento no puede bloquear en ningún host, y así un engine viejo o ausente se nota una vez por sesión («hook exited with code 2/127»). Esto vale también para Codex, donde `exo-recall` no está.
- `ENGINE_MIN` sube a la primera versión que trae `exo hook`.

### 4.4 Instalación

**Claude (A).** Sigue el mismo canal, el marketplace `.claude-plugin`, y el usuario no tiene que hacer nada nuevo. Todas las entradas de abajo llevan `"timeout": 5`. Cambios en `hooks.json`:
- Las entradas de los guards migrados pasan a `exo hook claude <ev> || exit 0`. Es decir, `PreToolUse` `^(Bash|PowerShell)$` y `^(Agent|Task|Edit|Write|NotebookEdit)$`.
- **Entradas nuevas** para alimentar el journal: `PostToolUse` y `PostToolUseFailure` con matcher `^(Bash|PowerShell|Edit|Write|NotebookEdit)$`. Es lo único que leen los guards migrados.
- No hay entrada `UserPromptSubmit` ni `SessionStart` nuevas. Coste: un proceso de unos 3 ms por cada tool de esas cinco.

**Codex [B].**
- El repo ya funciona como marketplace de Codex, porque Codex lee `.claude-plugin/marketplace.json` (`core-plugins/src/marketplace.rs:20-25`).
- Se añade `plugins/exo/.codex-plugin/plugin.json` mínimo: `name`, `version` y **`"hooks": "./hooks/codex.json"`. Este último es obligatorio.** Codex parsea los ficheros de hooks con `deny_unknown_fields` (`config/src/hook_config.rs:13-19`). La clave `modules` del `hooks.json` de Claude hace fallar el parse y **Codex carga cero hooks**: es lo que pasa hoy si se instala el plugin en Codex. El orden de búsqueda del manifest es `.codex-plugin` > `.claude-plugin` > `.cursor-plugin` (`utils/plugins/src/plugin_namespace.rs:43-78`).
- `skills` no se declara porque ya es la ruta por defecto.
- `scripts/test-versiones.sh` se extiende al tercer fichero que lleva `version`.
- `codex.json` se escribe a mano (unas 8 entradas) con `"timeout": 5`. **No se genera.**
- Pasos del usuario: `install.sh`, luego `codex plugin marketplace add pguerrerolinares/exo` (en la doc se recomienda `--sparse`, porque Codex clona el marketplace entero y el engine va dentro) y `codex plugin add exo`. Después, aceptar la confianza de los hooks en el primer uso. Codex copia el plugin a `~/.codex/plugins/cache/<mkt>/<plugin>/<version>` y le pasa esa ruta como `PLUGIN_ROOT`/`CLAUDE_PLUGIN_ROOT`.
- Los hooks corren con el `$SHELL` de la sesión (aquí `zsh -lc`, que **no** lee `~/.profile`). El `PATH` lo hereda del entorno de la sesión que Codex reproduce, así que Codex lanzado desde un IDE con un `PATH` mínimo no encuentra `exo`. §4.3 hace ese caso visible.
- Los agents `consultor` y `executor` van como roles TOML en la config, porque el manifest no los empaqueta.
- Se añade una línea al README: hasta B, instalar el plugin en Codex no hace nada, sin dar ningún aviso.

## 5. Errores (postura: todo degrada a `Allow`)

exo no es un producto de seguridad. Sus guards son reflejos de disciplina, y la doctrina es que **un reflejo nunca rompe lo que instrumenta**. Del fail-closed de sondera se toma solo la idea de una política de degradación explícita.

| Error | Respuesta | Rastro |
|---|---|---|
| stdin ilegible o payload que no parsea | `Allow` | Un registro `warn` en el journal, una vez por sesión, más una línea en el reflex-log con `code` |
| Panic en un guard | Se descarta ese guard y los demás siguen (`catch_unwind`; el engine no tiene `panic = "abort"`) | `panic::set_hook` escribe en el reflex-log, **no en stderr** |
| Fallo de I/O del journal | El guard pasa a modo «sin evidencia», igual que hoy con «sin transcript» (#182) | Igual que arriba |
| Timeout | Lo gestiona el host (`"timeout": 5`). **No hay watchdog interno**: los timeouts son de 600 s por defecto y no bloquean ni en Codex (`discovery.rs:968`) ni en Claude | — |

## 6. Matriz y testing

### 6.1 Matriz

- Exhaustividad garantizada al compilar (§3).
- **Test de capacidad:** para cada guard `Soportado`/`Degradado` en un harness, `render(d, harness)` tiene que producir salida para todas las decisiones `d` que el guard declara, y el evento que necesita tiene que aparecer en el **golden capturado** de ese harness. Si falla, el test da un mensaje accionable con nombre, por ejemplo: «search-first declarado Soportado en codex pero el golden de codex no contiene `<evento>`: decláralo NoSoportado o cambia el evento». Así se evita el estado imposible de un guard soportado sin evento o sin capacidad. No se escribe a mano una tabla de capacidades por harness × evento.
- **Test de coherencia con los ficheros:**
  - Todo guard `Soportado`/`Degradado` tiene su entrada en `hooks.json` o `codex.json`.
  - Ninguna entrada apunta a un guard `NoSoportado`.
  - Además se emite un **informe** (no un fallo) de las entradas `.sh` de `hooks.json` que la matriz no cubre, es decir, lo que no está declarado.
- Quedan aplazados: `exo hook matrix [--json]` y la integración con `doctor --harness`.

**Matriz de A y B:**

| Guard | claude | codex |
|---|---|---|
| bash-guards (git-c rewrite, git add -A, zero-residuo) | Soportado | Soportado (el rewrite se renderiza como `allow` + `updatedInput`) |
| bash-guards rama verify | Soportado | **Degradado**: sin exit code (`ok: null`), siempre registra «sin evidencia de verde». Igual que hoy, verify **solo escribe en el log y no emite nada al agente** (`bash-guards.sh:22-35,296-301`); en Codex no se añade un `Steer`. Leer `PostToolUse` como verde daría un falso verde. |
| search-first | Soportado | Soportado (`PreToolUse` es genérico para todas las tools, `registry.rs:168`; `apply_patch` acepta los matchers `Write`/`Edit`, `spawn_agent` acepta `Agent`). Igual que hoy, solo cuenta un `exo search` lanzado desde una tool `Shell` de nombre `Bash` (`search-first.sh:136`), que es también el nombre de la shell en Codex. |
| guards no migrados | `.sh` directo | NoSoportado hasta que se migren |

### 6.2 Tests

1. **Unitarios en Rust** por guard y por clasificador.
2. **Golden de adapters** sobre payloads reales. El de Codex es `codex-0.166.0-probe.jsonl`; el de Claude hay que capturarlo en A.
3. **Oráculo de paridad** (§6.3).
4. **CI de Windows:** comprobar que `cmd /C "nope || exit 0"` y `cmd /C "exo hook codex PreToolUse || exit 0"` devuelven 0 [B].
5. **`scripts/smoke-codex.sh`**, fuera de CI porque necesita la cuenta del dueño. Monta hooks de volcado, ejecuta `codex exec` y hace aserciones. **Regenera el golden**, así que recapturar consiste en correrlo. Se ejecuta al empezar B y en cada actualización de Codex [B].
6. El `cargo-mutants` nocturno que ya existe cubre `hook/`. Cabe esperar ruido en los strings de `Steer`.

### 6.3 Oráculo de paridad

`test-bash-guards.sh` (74 aserciones) y `test-search-first.sh` (15 casos) se ejecutan con `HOOK` apuntando a un wrapper. `EXO_STATE_DIR` apunta a un `mktemp -d` y el binario respeta `REFLEX_LOG_FILE` tal cual. Las aserciones sobre el reflex-log exigen **el mismo esquema y los mismos ids de reflejo** (`search-first-ok`, `git-c-rewrite`, `zero-residuo-backoff`…). Solo cambian los valores de `motivo` que eran propios de bash, como `sin-jq`, que desaparecen. `_zero-residuo-clasifica.sh` y `_truncate-payload.sh` se migran junto con bash-guards, porque determinan el payload que se loguea. Antes de cada caso, el wrapper hace dos cosas:

1. **Cruza cada `tool_use` del transcript fixture con su `tool_result`.** Los que no tienen `is_error: true` se reinyectan como `PostToolUse` contra `exo hook claude`, y los que sí lo tienen, como `PostToolUseFailure`. Así se reproduce lo que pasa de verdad en Claude.
2. Después invoca el evento que se está probando.

Si un `tool_use` ocurre dentro de un subagente, se reinyecta con su `agent_id`, y el guard filtra por él: el padre no ve el `exo search` que hace un hijo, igual que hoy.

**Criterio:** 100 % de los casos, menos las divergencias de §6.4.

**Caso nuevo que tiene que existir antes de migrar:** la secuencia «pytest verde → pytest rojo → git commit» debe avisar. Hoy no hay ningún caso que la cubra (`test-bash-guards.sh:407-421,632-666`), así que se añade primero a la suite bash, se comprueba que pasa contra el `.sh` actual, y solo entonces pasa a formar parte del oráculo.

**Los `.sh` huérfanos** que `bash-guards.sh:14-15` mantiene como red de regresión (los ejecuta `test-plugin.sh`) se borran junto con bash-guards, y `test-plugin.sh` se actualiza en el mismo cambio.

### 6.4 Divergencias intencionadas (cada una con un caso que la sustituye)

| Caso actual | Por qué diverge | Caso sustituto |
|---|---|---|
| bash-guards caso 4; search-first 7a/7b (sin `transcript_path` / transcript ilegible) | El guard ya no lee el transcript | «sin journal» y «journal ilegible»: avisa igual (#182) |
| search-first 15 (línea corrupta) | Ahora la línea corrupta está en el journal | Journal con una línea corrupta: se ignora esa línea |
| Casos 6/8 (jq envenenado o ausente) | Eran degradaciones propias de bash | No aplican |
| search-first casos 5 y 14 (existe el fichero sentinel en `SEARCH_FIRST_SENTINEL_DIR`, `test-search-first.sh:112-117,268`) | La regla «avisa una vez por sesión» pasa del sentinel a un registro `warn` en el journal | Aserción sobre el registro `warn` en el journal de la sesión |
| Ventana `tail -n 400` | Se sustituye por la cota de 2 MiB | Equivalente para fixtures pequeños |
| **Tools en paralelo** | Codex `exec_command` tiene `supports_parallel_tool_calls = true`, y en Claude los `PostToolUse` corren a la vez en llamadas paralelas: el `PreToolUse` de B puede leer el journal antes de que el `PostToolUse` de A lo escriba | Se documenta. Coste: como mucho un aviso falso por sesión en search-first, o un orden por finalización en verify |
| **Journal vacío tras actualizar** | En Claude, search-first no ve un `exo search` anterior a la instalación de la versión nueva dentro de la misma sesión | Se documenta. Coste: un aviso de más en la primera sesión |

## 7. Condiciones de invalidación (pre-registradas)

1. **Journal en Codex.** `PostToolUse` con `Bash` no trae `tool_input.command` o no se dispara con exit≠0. **Medido el 2026-11-02 con codex 0.166.0: no se cumple.** Se vuelve a comprobar con el smoke en cada actualización.
2. **Paridad.** Con el mapeo de `is_error`, el oráculo no da 74/74 y 15/15 menos las divergencias de §6.4. Si pasa, el journal no sustituye al transcript y se reabre §2.
3. **Latencia.** `exo hook claude PreToolUse` sobre un journal de 2 MiB da un p99 por encima de 300 ms (medido con hyperfine). Si pasa, vuelve el watchdog interno. Hoy `exo --version` tarda unos 3 ms en WSL2.
4. **[B] Distribución por plugin.** Con el plugin instalado de verdad en Codex, un `deny` de `PreToolUse` no impide el Bash. Si pasa, cae la distribución por plugin.
5. **[B] Windows.** `cmd /C "exo hook … || exit 0"` sin `exo` instalado devuelve ≠0. Si pasa, se sustituye por un wrapper `.cmd`.

## 8. Fuera de alcance

- Migrar el resto de guards (`exo-recall`, `estilo-directo`, `delegar-decision`, `document-remind`, `exo-index`, `sdd-wake-guard`, `subagent-inject`, `clean-orchestrator-research`). Cada uno entra en iteraciones posteriores con este mismo patrón.
- `clean-orchestrator-research` sobre WebSearch en Codex: `web_search` es una tool que se ejecuta en el servidor de OpenAI y no dispara hooks.
- pi (C) tiene su propia spec. Su API cambia en casi cada versión, así que habrá que fijar una versión exacta.
- Los eventos `PostToolUseFailure` y `PermissionDenied` no existen en Codex. El journal de Codex vive sin el primero (`ok: null`).

## 9. Decisiones del autor tras el builder check

Un reviewer leyó la spec como builder e hizo 8 preguntas. Todas se han cerrado como decisiones del autor (Claude); ninguna necesitaba al dueño. Están integradas en el texto, y aquí van resumidas para que se puedan corregir:

1. El journal registra los fallos de Claude (`PostToolUseFailure`, `ok: false`), y se añade el caso «verde → rojo → commit» (§4.2, §6.3).
2. Nueva variante `Rewrite{input, razón}`, soportada en los dos harness (§3).
3. A no mueve el estado heredado; B lo mueve entero (§3).
4. El binario respeta `REFLEX_LOG_FILE`; los casos del sentinel pasan a ser divergencias (§6.3, §6.4).
5. Entradas nuevas de `hooks.json`: solo `PostToolUse`/`PostToolUseFailure` para cinco tools, con GC perezosa (§4.2, §4.4).
6. Todos los guards filtran por `agent_id` (§4.2).
7. verify Degradado en Codex significa solo log, como hoy (§6.1).
8. Se mantienen el esquema del reflex-log y los ids de reflejo; `_zero-residuo-clasifica` y `_truncate-payload` se migran (§6.3).
9. Menores: search-first sigue contando solo Bash; los `.sh` huérfanos se borran junto con bash-guards.
