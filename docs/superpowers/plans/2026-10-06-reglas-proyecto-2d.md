# Plan: 2d — reglas de proyecto en el system prompt

> For agentic workers: ejecución con `exo:orchestrate`.

**Plan:** ~25 min, ~12 turnos, 17 KB

**Goal:** que la sesión principal de Claude Code en el repo X reciba en el system prompt, con el framing sellado de #49, las reglas de `## Reglas duras` de la nota-puerta de X. Y que Paul vea cuando eso falla de forma anómala.

**Architecture:**
- `exo rules` (engine Rust) resuelve cwd → nota → sección y devuelve `ok` o `skip` en el envelope.
- Un mod (`prompt.compose`) consume ese resultado y añade la sección; además deja un latido.
- `exo-recall.sh` (SessionStart) resuelve en paralelo, guarda `ss-<sid>` y grita las anomalías.
- `document-remind.sh` (Stop) compara el latido con `ss-<sid>` y grita si el mod no entregó.

**Tech Stack:** Rust (engine, clap, serde_json), TypeScript (mod de Claude Code, `claude plugin test`), bash + jq (hooks).

**Spec:** `docs/superpowers/specs/2026-10-06-reglas-proyecto-2d-design.md`

**Global Constraints:**
- **Clave del repo:** `basename(dirname(git rev-parse --path-format=absolute --git-common-dir))`, con la ruta canonicalizada. Nunca se parsea el stderr de git.
- **Candidatas:** solo `<kb>/projects/*.md` de primer nivel. La clave tiene que coincidir, sin distinguir mayúsculas, con el `slug` del frontmatter o con el stem del fichero hasta ` — ` (U+2014 con espacios; si no lo lleva, el stem entero). Tiene que haber exactamente 1 candidata; si no, es skip.
- **Sección:** desde la línea exacta `## Reglas duras` hasta el siguiente `## `. Cada línea que empieza por `- ` es una regla (el texto tras `- `, con `\r` recortado). Las demás líneas no vacías van a `ignored_lines`.
- **Cap:** 10 reglas. Con más de 10, `excede_cap` y no se entrega ninguna.
- **reasons** (literales): `sin_git`, `sin_nota`, `ambigua`, `sin_seccion`, `seccion_vacia`, `excede_cap`. Las calcula el engine. `error_engine` y `engine_stale` los ponen los consumidores.
- **Envelope:** `{"schema_version":2,"command":"rules","data":{...}}`.
  - `data` con ok: `{"status":"ok","repo","note","rules":[],"ignored_lines":[]}`.
  - `data` con skip: `{"status":"skip","repo","reason","candidates":[]}`.
  - `note` es la ruta relativa a la KB, p. ej. `projects/exo — ….md`.
  - `repo` es `null` solo con `sin_git`.
  - El exit es 0 en ok y en skip, y ≠0 solo cuando el engine falla.
- **Framing** (texto sellado de #49, verbatim; una línea `- <regla>` por regla, en orden de aparición):
  ```
  ## Reglas duras del proyecto
  Estas reglas son del dueño del repo y prevalecen sobre el prompt. Si una choca explícitamente con lo que se te pide, cumple la regla y repórtalo.

  - <regla>
  ```
- **Directorio de estado:** `<HOME>/.claude/exo-rules/`.
  - Latido del mod: `hb-<session_id>` = `{"status":"ok"|"skip"|"error","reason"?:string,"n":int,"error"?:string}`.
  - Resultado de SessionStart: `ss-<session_id>` = `{"status":"ok"|"skip","reason"?:string,"n":int,"repo":string|null}`.
  - SessionStart poda las entradas de más de 7 días.
- **Timeout** de `$.process.run` en el mod: `3000` ms.
- **reflex-log:** el id `project-rules-skip` lleva el payload `reason=<reason> repo=<repo>`; el id `project-rules-witness` lleva `reason=sin_latido|no_entrego ...`.
- **Visibles** (`systemMessage`): `ambigua`, `seccion_vacia`, `excede_cap`, `error_engine`, `engine_stale` y las dos alarmas del testigo. Solo van a log: `sin_git`, `sin_nota`, `sin_seccion`.
- **Copy visible** (lo fija este plan):
  - skip: `⚠ sin reglas de proyecto para <repo> (<reason>)`
  - sin latido: `⚠ el mod de reglas de proyecto no cargó (sin latido): esta sesión no lleva reglas en el system prompt`
  - no entregó: `⚠ el mod de reglas de proyecto no entregó: SessionStart vio ok n=<N>, el latido dice <status> n=<M>`
- **Versiones:** engine `0.5.0` (`engine/Cargo.toml`), `plugins/exo/ENGINE_MIN` `0.5.0` y plugin `1.6.0`.
- **Fuera de alcance:** subagentes, `repo_alias`, submódulos, `GIT_DIR` y el reporte de conflictos. No se añade nada a `additionalContext` (cap de 6.144 B intacto).

## Olas

- **Ola 1, en paralelo: T1, T2, T3, T4.** Sus `Files` no se tocan entre sí. T2, T3 y T4 consumen el **contrato** de `exo rules` y el formato de `hb`/`ss` fijados arriba, no código de T1: los tests usan stubs o mocks.
- **Ola 2: T5.** Wiring de versiones, gates, `/document` y el e2e real; consume T1–T4.

### Task 1: `exo rules` en el engine

**Files:**
- Create: `engine/src/reglas.rs`
- Modify: `engine/src/lib.rs`
- Modify: `engine/src/main.rs`
- Test: `engine/tests/reglas_cli.rs`
- Modify: `plugins/exo/scripts/test-contrato-engine.sh`

**Interfaces:**
- Produces: `pub fn resuelve(kb: &Path, cwd: &Path) -> anyhow::Result<Resultado>` en `exo::reglas`.
- Produces: `pub enum Resultado { Ok { repo: String, note: String, rules: Vec<String>, ignored_lines: Vec<String> }, Skip { repo: Option<String>, reason: Razon, candidates: Vec<String> } }` con `Serialize` al shape de `data` de las Global Constraints.
- Produces: `pub enum Razon { SinGit, SinNota, Ambigua, SinSeccion, SeccionVacia, ExcedeCap }`, serializado en snake_case.
- Produces: CLI `exo rules --cwd <dir> [--kb <path>] [--json]`. Por defecto `--cwd` es el directorio actual y `--kb` sale de `resuelve_kb`. Sin `--json` imprime una línea: `ok <repo>: <n> reglas` o `skip <repo|->: <reason>`.

**Tests:** (KB y repos git en `tempfile`, `GIT_CONFIG_GLOBAL`/`SYSTEM` vacíos como en `targets_cli.rs`; binario real)
- `ok_por_slug`: repo `foo` y nota `projects/Foo bar.md` con `slug: foo` y 2 reglas dan `status=ok`, `rules` con las 2 en orden y `note="projects/Foo bar.md"`. Falla si el slug no se lee o no se compara sin mayúsculas.
- `ok_por_stem_con_raya`: repo `exo` y nota `projects/exo — framework.md` sin slug dan `ok`. Falla si el stem no se corta en ` — `.
- `desde_subdirectorio_y_worktree`: con `--cwd` en `repo/a/b` y en un `git worktree add` fuera del repo, `repo` vale el nombre del repo principal. Falla si se usa `--git-common-dir` relativo (daría `.` o `..`).
- `sin_git`: un cwd tempdir sin repo da `skip`, `reason=sin_git`, `repo=null` y exit 0.
- `sin_nota`: un repo sin nota que case da `sin_nota`.
- `subcarpeta_no_compite`: `projects/pm.md` (slug `pm`) más `projects/pm/pm — guía.md` dan `ok` con la nota de primer nivel. Falla si el barrido es recursivo (caso real `pguerrero-music`).
- `ambigua`: `projects/foo.md` y `projects/x.md` con `slug: foo` dan `ambigua`, con `candidates` de longitud 2. Falla si se elige una.
- `sin_seccion` / `seccion_vacia`: una nota sin el encabezado da `sin_seccion`; con el encabezado y solo prosa da `seccion_vacia`, con la prosa en ningún sitio que se entregue.
- `seccion_hasta_siguiente_h2_y_crlf`: una sección con CRLF, una línea de prosa y un `## Otra` con `- x` debajo da `rules` sin `\r`, `ignored_lines=[prosa]` y nada de `## Otra`. Falla si arrastra `\r` o lee más allá del siguiente `## `.
- `encabezado_exacto`: `## Reglas duras (borrador)` da `sin_seccion`.
- `excede_cap`: 11 reglas dan `excede_cap` y ningún campo `rules`. Con 10 da `ok`. Falla si se trunca.
- `envelope_rules`: `--json` emite una línea con `command=="rules"` y `schema_version==2` (añadirlo también en `test-contrato-engine.sh`).

**Verificación:** `cargo test --manifest-path engine/Cargo.toml && cargo clippy --manifest-path engine/Cargo.toml --all-targets --locked -- -D warnings && cargo fmt --manifest-path engine/Cargo.toml --check` → todo verde.

**Review Focus:**
- Un cwd dentro de la propia KB (que también es un repo git) resuelve como cualquier otro repo: `sin_nota` si no hay nota `wisdom-paul`. Hay test implícito en `sin_nota`; añade uno explícito.
- Frontmatter sin `slug`, o `slug` con comillas: usa `frontmatter::valor` y quita las comillas. Test con `slug: "foo"`.
- Una línea `-x` sin espacio no es regla: va a `ignored_lines`.
- Un fallo al lanzar git (binario ausente) es error del engine (exit ≠0), no `sin_git`.
- Orden estable de `candidates` (ordenadas) para que el test no dependa del orden del FS.

### Task 2: mod `prompt.compose` con latido

**Files:**
- Create: `plugins/exo/hooks/register.ts`
- Create: `plugins/exo/hooks/register.test.ts`
- Modify: `plugins/exo/hooks/hooks.json`

**Interfaces:**
- Produces: `"modules": ["./register.ts"]` en `hooks.json`, junto a `"hooks"` (forma verificada en la sonda).
- Produces: hook `prompt.compose`. Llama a `next(e)` y, con `ok`, devuelve `{ sections: [...r.sections, SECCION] }` con `id: "exo:reglas-proyecto"`, `scope: "session"` y el texto del framing.
- Produces: el fichero `hb-<sid>` con el formato de las Global Constraints, escrito desde `prompt.compose` en **todos** los casos.

**Tests:** (`claude plugin test plugins/exo`; mock de `$.process.run`, `$.fs.write` y `$.env` desde los hooks de test)
- `ok_anade_seccion`: con un stdout de `exo rules` con `ok` y 2 reglas, la última sección tiene exactamente el framing con `- r1\n- r2`. Falla si cambia una coma del framing o el orden.
- `skip_no_anade`: con `skip` y `sin_seccion`, las secciones devueltas son las de `next` sin cambios, y `hb` vale `{"status":"skip","reason":"sin_seccion","n":0}`.
- `latido_ok`: con `ok` y 2 reglas, `$.fs.write` recibe `<HOME>/.claude/exo-rules/hb-<sid>` con `status:"ok", n:2`.
- `error_exit_no_cero`: con exit 1, no hay sección y `hb.status=="error"` con `error` no vacío. Falla si un error se trata como skip silencioso.
- `json_invalido`: con un stdout que no es JSON, el resultado es `error` y no lanza excepción (compose no puede romper la sesión).
- `timeout_3000`: la llamada a `process.run` lleva `timeoutMs: 3000` y `argv` `[exo, "rules", "--cwd", <cwd>, "--json"]`.
- `fallback_bin`: si la llamada con `exo` falla por binario ausente, se reintenta con `<HOME>/.local/bin/exo`.
- `home_userprofile`: sin `HOME` y con `USERPROFILE`, el latido va bajo `USERPROFILE`.

**Verificación:** `claude plugin validate plugins/exo && claude plugin test plugins/exo` → sin errores. `bash scripts/test-hooks-json.sh` → verde (la clave `modules` no rompe el recorrido de `.hooks`).

**Review Focus:**
- Cómo señala `$.process.run` que el binario no existe (si rechaza o devuelve un exitCode): verifícalo con la referencia `plugin-authoring` y fija el test a lo real.
- Que el mock de `$` en `claude plugin test` permita interceptar `process.run`, `fs.write` y `env.get`. Si alguno no se puede interceptar, dilo; no lo sustituyas por un test que no mira nada.
- Una excepción dentro del hook no puede impedir `next(e)`: la sesión arranca siempre.
- El latido se escribe aunque falle la creación del directorio (créalo antes; si falla, no hay sección extra y no hay excepción).
- No leer el cwd de `e` (no viene): `$.session.cwd()`.

**Notas:** la sección y el texto del framing son los de las Global Constraints, verbatim.

### Task 3: `exo-recall.sh` resuelve reglas y grita anomalías

**Files:**
- Modify: `plugins/exo/scripts/exo-recall.sh`
- Modify: `plugins/exo/scripts/test-exo-recall.sh`

**Interfaces:**
- Usa (contrato fijo, sin dependencia de tarea): el contrato `exo rules --cwd <cwd> --json` de las Global Constraints (con stub vía `EXO_BIN`).
- Produces: `<HOME>/.claude/exo-rules/ss-<sid>` con el formato de las Global Constraints.
- Produces: `systemMessage` de primer nivel en la salida JSON existente, junto a `hookSpecificOutput`, solo para los visibles.

**Tests:** (en `test-exo-recall.sh`, con el stub de `exo` que ya usa el fichero)
- `reglas_ok_silencioso`: con el stub en `ok` y n=3, `ss-<sid>` vale `{status:"ok",n:3,...}`, la salida no tiene `systemMessage` y el `additionalContext` es idéntico al de antes. Falla si se toca el bloque de recall.
- `sin_seccion_solo_log`: con `skip` y `sin_seccion`, no hay `systemMessage` y reflex-log tiene `project-rules-skip` con `reason=sin_seccion`.
- `ambigua_visible`: con `skip` y `ambigua`, `systemMessage == "⚠ sin reglas de proyecto para <repo> (ambigua)"` y la entrada en reflex-log.
- `error_engine_visible`: con el stub en exit 1, `systemMessage` contiene `(error_engine)` y `ss.status=="skip"` con `reason=error_engine`.
- `engine_stale_sin_llamar`: con `ENGINE_MIN` por encima de la versión del stub, no hay llamada a `exo rules` (el stub registra la llamada), `systemMessage` contiene `(engine_stale)`. Falla si se invoca un subcomando que el engine viejo no tiene.
- `compact_no_resuelve`: con `source=compact`, ni hay llamada a `exo rules` ni se reescribe `ss`.
- `poda_7_dias`: un `ss-viejo` con mtime de hace 8 días desaparece y uno de hace 1 día se queda.

**Verificación:** `bash scripts/test-plugin.sh` → todos `[PASS]`, incluidos `test-exo-recall-golden.sh` y `test-exo-recall-superpowers.sh` sin cambios en sus goldens.

**Review Focus:**
- El cwd que se pasa a `exo rules` es el `.cwd` del JSON de entrada del hook, no el `$PWD` del script.
- Una salida sin anomalía tiene que ser byte a byte la de antes (goldens).
- `jq` sobre un stdout de stub vacío no puede abortar el hook (`set -u`, sin `-e`).
- Sin `session_id`, no se escribe `ss` (ni en un fichero `ss-`).
- El timeout de la llamada a `exo rules` usa `_timeout.sh`, como el resto del script.

### Task 4: testigo de entrega en Stop

**Files:**
- Modify: `plugins/exo/scripts/document-remind.sh`
- Create: `plugins/exo/scripts/test-document-remind.sh`

**Interfaces:**
- Usa (contrato fijo, sin dependencia de tarea): `ss-<sid>` y `hb-<sid>` con el formato de las Global Constraints.
- Produces: un solo objeto `{"systemMessage": ...}` por disparo. Si el testigo y el recordatorio coinciden, los dos textos van unidos con `\n`.

**Tests:**
- `sin_latido_grita_una_vez`: con `ss` presente y `hb` ausente, el primer Stop emite el copy "no cargó" y escribe `project-rules-witness` con `reason=sin_latido`; el segundo Stop no emite nada del testigo (sentinel propio).
- `no_entrego`: con `ss` ok n=3 y `hb` `error` n=0, el copy es exactamente "…SessionStart vio ok n=3, el latido dice error n=0".
- `n_distinto`: con `ss` ok n=3 y `hb` ok n=2, grita "no entregó".
- `coherente_calla`: `ss` ok n=3 con `hb` ok n=3 no emite nada; `ss` skip con `hb` skip tampoco; `ss` skip con `hb` error tampoco (el skip ya gritó en SessionStart si tocaba).
- `sin_ss_solo_log`: sin `ss`, nada visible y `project-rules-witness` con `reason=sin_ss`.
- `testigo_antes_de_umbral`: con un transcript de 10 líneas (por debajo del umbral del recordatorio), el testigo grita igual. Falla si queda detrás de los `exit 0` del recordatorio.
- `recordatorio_intacto`: sin estado de reglas y con transcript por encima del umbral, emite exactamente el JSON de recordatorio de hoy.
- `ambos_unidos`: con el testigo y el recordatorio en el mismo Stop, sale una sola línea JSON cuyo `systemMessage` lleva los dos textos.

**Verificación:** `bash scripts/test-plugin.sh` → todos `[PASS]`, incluido el nuevo `test-document-remind.sh`.

**Review Focus:**
- El sentinel del testigo es distinto del del recordatorio (`/tmp/claude-document-reminded-<sid>`): uno no puede silenciar al otro.
- Un `hb` con JSON corrupto cuenta como "no entregó", no como coherente.
- `ss` con `reason=error_engine` o `engine_stale` y `hb` ausente: el mod no cargó, y eso grita aunque SessionStart ya gritara (son fallos distintos).
- `HOME` del test en un tempdir: nunca tocar `~/.claude` real.

### Task 5: wiring, versiones, `/document` y e2e

**Files:**
- Modify: `engine/Cargo.toml`
- Modify: `engine/Cargo.lock`
- Modify: `plugins/exo/ENGINE_MIN`
- Modify: `plugins/exo/.claude-plugin/plugin.json`
- Modify: `plugins/exo/skills/document/routing.md`
- Create: `plugins/exo/scripts/test-reglas-mod.sh`
- Modify: `scripts/release-publish.sh`
- Modify: `scripts/test-release-publish.sh`
- Create: `scripts/e2e-reglas-proyecto.sh`

**Interfaces:**
- Consumes: CLI `exo rules` @Task 1
- Consumes: hook `prompt.compose` y `hb-<sid>` @Task 2
- Consumes: `ss-<sid>` y `systemMessage` de SessionStart @Task 3
- Consumes: el testigo de Stop @Task 4
- Produces: `test-reglas-mod.sh` corre `claude plugin validate plugins/exo` y `claude plugin test plugins/exo`. Sin `claude` en el PATH imprime `[SKIP-GRITA] claude no está en PATH: el mod de reglas NO se ha validado` por stderr y sale 0. Con `EXO_REQUIRE_CLAUDE=1`, sin `claude` sale 1.
- Produces: `release-publish.sh` corre `EXO_REQUIRE_CLAUDE=1 plugins/exo/scripts/test-reglas-mod.sh` antes de publicar.

**Tests:**
- `test-release-publish.sh` → `release_exige_mod_validado`: con un PATH sin `claude`, `release-publish.sh` falla antes de llamar al `gh` falso. Falla si una release puede salir con el mod sin validar.
- `test-reglas-mod.sh` sin `claude` → stderr contiene `[SKIP-GRITA]` y exit 0. Con `EXO_REQUIRE_CLAUDE=1` → exit 1.
- `bash scripts/test-versiones.sh` → verde con Cargo `0.5.0` y `ENGINE_MIN` `0.5.0`.
- E2E (`scripts/e2e-reglas-proyecto.sh`; local, cuesta tokens, fuera de la suite):
  - Prepara una KB fixture con `projects/fixrepo.md`, `## Reglas duras` y `- El codeword es REGLAS-2D-7Q.`, y un repo git `fixrepo` vacío.
  - Corre `claude -p --setting-sources "" --strict-mcp-config --settings <json con Read/Grep/Glob/Bash denegadas> --plugin-dir plugins/exo` con `EXO_KB=<fixture>` desde `fixrepo`, preguntando por el codeword del system prompt.
  - Comprueba que la respuesta contiene `REGLAS-2D-7Q` y que existe `hb-<sid>` con `ok n=1`.
  - Control: el mismo comando sin la sección responde sin el codeword.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml && bash scripts/test-plugin.sh && bash scripts/test-versiones.sh && bash scripts/test-release-publish.sh && bash scripts/test-hooks-json.sh && bash scripts/plugin-bump-gate.sh` → verde. Después, `bash scripts/e2e-reglas-proyecto.sh` → `PASS` en positivo y en control (se pega la salida en el PR).

**Review Focus:**
- `EXO_KB` tiene que llegar al `exo` que lanza el mod (`$.process.run` hereda el entorno). Si no llega, el e2e lee la KB real y miente. Verifícalo en el e2e: el codeword solo existe en el fixture.
- Que `plugin.json` `1.6.0` satisfaga `plugin-bump-gate.sh` y `test-plugin-bump.sh`.
- `Cargo.lock` regenerado con `cargo check --locked` en verde.
- `test-plugin.sh` recoge `test-reglas-mod.sh` solo por el glob; no hace falta tocarlo.

**Notas:** la frase de `routing.md`, verbatim de la spec: una regla **mecánica** de un solo repo va además a `## Reglas duras` de su nota-puerta (una línea `- `, ≤10 por nota).
