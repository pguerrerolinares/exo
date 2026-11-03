# Nota consultor (opus) — enfoques del subproyecto B (Codex) — 2026-11-03

Recomendación D1: (1a) convención + test de acuerdo, con `exo config --json` exponiendo `data.state.{dir,reflex_log}` (test contra el binario real, matrix ubuntu+windows) y migración como merge idempotente en cada SessionStart de Rust (rename viejo→viejo.migrando, append a la nueva, borrar), no rename único: plugins viejos en caché recrean el fichero viejo. Precedencia: REFLEX_LOG_FILE > $EXO_STATE_DIR/reflex-log.jsonl > default. exo-rules/ no se migra (efímero, GC 7 días). Añadir campo `harness` al reflex-log (aditivo; ausente = claude).
Recomendación D2: (2a) una spec, olas: estado → SessionStart Rust en Claude con oráculo → adapter codex + codex.json + reglas → roles + Windows CI. Flag `--engine-min` en `exo hook` (engine 0.14 sale rc=0 mudo en SessionStart).
Invalidación: (1) test de acuerdo en windows-latest da rutas distintas entre dirs::home_dir() y $HOME de Git Bash → los .sh de Windows preguntan al engine; (2) merge con 4 appenders concurrentes pierde o corrompe líneas; (3) additionalContext de `exo hook codex SessionStart` > ~10 KB (2500 tokens, output_spill.rs:13) → Codex trunca y las reglas del medio se pierden.
Opciones descartadas: (1b) spawn barato pero sin engine cada .sh necesita fallback → dos fuentes de verdad; (1c) la ventana nunca cierra (reflex-fp-review necesita historia entera); (2b) B1 = guards sin memoria, parte una decisión acoplada.
Desacuerdos con el brief: skills (distill, orchestrate) ejecutan .sh también en Codex Unix; exo-recall.sh:161,167 ignora REFLEX_LOG_FILE (bug, no paridad); exo-index.sh (Stop, .sh) no corre en Codex → índice rancio; engine 0.14 no avisa en SessionStart.
Sin dato: que Codex muestre al usuario un SessionStart exit 2; approx_token_count ≈ bytes/4; home en Windows dirs vs Git Bash.

## Verificado por el padre (2026-11-03)
- output_spill.rs:13 `DEFAULT_HOOK_OUTPUT_TOKEN_LIMIT: usize = 2_500` (Codex f7167ac).
- `printf '{}' | exo hook {claude,codex} SessionStart` con exo 0.14.0 → rc=0, stdout vacío.
- skills/distill/SKILL.md:108 corre cobertura-reglas.sh; orchestrate/SKILL.md:56 usa scripts review-package.
- main.rs:569,583 resuelve_db/resuelve_kb en el binario, no en la lib.
- exo-recall.sh:161,167 lee $HOME/.claude/reflex-log.jsonl a pelo.

# Nota consultor (Fable, segunda opinión ciega) — 2026-11-03

Recomendación D1: (1a) + migración (i) rename único por el engine en `exo hook <h> SessionStart` + finding informativo de `exo doctor` sobre residuo en ~/.claude. Claude Code fija versión de plugin por sesión (`.in_use/<pid>`, barrido 24 h): una sesión abierta sigue con los .sh viejos hasta cerrarse; la pérdida con (i) es ≤ una sesión de líneas, recuperable con `cat >>`; (ii) mete rename+append+delete concurrente en el camino caliente. Falta en el inventario: compose-inject.sh:10 canary ~/.claude/reflex-inject-canary. (1b) descartada: recall loguea `no-engine` precisamente cuando el engine no responde. (1d) symlink descartada en Windows.
Recomendación D2: (2a) una spec, olas estado → index (Stop) → recall (SessionStart con `--min`) → adapter Codex + codex.json + roles + smoke + CI Windows. Si O3 se alarga, O4 se desgaja a plan aparte sin reabrir la spec.
Invalidación: (1) en Windows CI `exo config --json | jq -r .data.state.dir` (cygpath -u) ≠ `_state-dir.sh` bajo Git Bash → install.ps1 fija EXO_STATE_DIR explícito; (2) 7 días tras cutover `wc -c ~/.claude/reflex-log.jsonl` > 0 en más de una sesión → merge idempotente (ii); (3) hyperfine `exo hook claude SessionStart` sobre KB real p99 > 300 ms → gate-pendiente sale a `exo doctor`.
Desacuerdos: las suites de recall/index stubean el engine (EXO_BIN, EXO_INDEX_SETSID) y no pueden ser oráculo de un port in-process → tests Rust con KB fixture por categoría de aserción + golden de texto; plumbing bash a divergencias. §4.3 falso para 0.14 (runner.rs:88 Otro → vacío): `exo hook <h> SessionStart --min X` sin `|| exit 0`; sin flag → clap exit 2 visible; con flag y < X → systemMessage + fallback; test-versiones.sh verifica X == ENGINE_MIN. Codex emite "hook exited with code N" como HookOutputEntry Error (session_start.rs:317-322); render TUI sin dato. Testigo ss/hb NoSoportado en Codex.

## Verificado por el padre (2026-11-03)
- runner.rs:88 `EventoKind::Otro(_) => String::new()`.
- test-exo-recall.sh: 14 líneas con `EXO_BIN=`; 52 `pass "…"` únicos.
- compose-inject.sh:10 `CANARY_FILE="${REFLEX_CANARY_FILE:-$HOME/.claude/reflex-inject-canary}"`.
- ~/.claude/plugins/cache/exo/exo/1.32.0/.in_use existe; 22 versiones de exo en caché.

# Review adversarial sección 2 (Fable) — 2026-11-03

Recomendación: (A) mantener la sección 2 con ajustes; gate pendiente se queda en SessionStart con deadline interno.
Cambios: (1) `systemMessage` en SessionStart de Codex no lo imprime `codex exec` (session_start.rs:258-262 → HookOutputEntry Warning; exec solo imprime «hook: SessionStart <status>»); TUI sin dato → en Codex `Avisa` va también como línea de additionalContext. (2) Timeout del host vacía stdout entero (command_runner.rs:371-378) y SessionStart se espera antes del primer turno → deadline global 3 s para git + timeout 10; git en wisdom-paul mide 0,00–0,22 s. (3) «ambos ficheros → no toca» es fork permanente: con el binario compartido y PreToolUse ya en Rust, una sesión Claude abierta con plugin viejo escribe en las dos rutas; → «ambos» = append viejo→nuevo en un write() + rename del viejo a `.migrado-<ts>` (nunca borrar); lectores ordenan por ts (cobertura-reglas.sh:52-67, reflex-fp-review.sh:21-28). (4) Roles: install.sh corre antes de `codex plugin add`, no hay caché de donde copiar los TOML → `include_str!` en el engine + `exo setup codex` escribe copias y bloque entre marcadores; install.sh/.ps1 lo invocan. config_file AbsolutePathBuf; campos: developer_instructions, model, model_reasoning_effort/summary, model_verbosity, personality, service_tier, features, skills (solo desactivar). (5) Matchers: solo [A-Za-z0-9_|] = Exact; si no, regex sin anclar. HEAD renombra a `spawn_agent` (alias Agent); 0.166.0 emite `collaborationspawn_agent` → clasificar por ends_with("spawn_agent"). (6) approx_token_count = ceil(bytes/4); truncate_middle; orden Codex: contrato, reglas, reafirmación/avisos. (7) Marketplace: --sparse repetible; nombre `exo@exo`. `--min` con 0.14.0 → rc=2 visible como Error. (8) Fallback de Codex debe decir «sin reglas duras».
Invalidación: hyperfine p99 > 300 ms de SessionStart sobre wisdom-paul → gate a Stop; smoke con systemMessage visible en exec.log → cambio 1 sobra; tras install con sesión abierta, ambos ficheros existen y el viejo crece → confirma cambio 3.
Sin dato: render Warning en TUI; SessionStart source=compact en 0.166.0; read_sensitive_file_to_string.

## Verificado por el padre
- command_runner.rs:369-378 timeout → stdout String::new(), outcome "timeout".
- session_start.rs:258-261 systemMessage → entries Warning.

# Review adversarial sección 3 (Fable) — 2026-11-03

Recomendación (B-lite): sub-guards de recall como variantes de Guard; la matriz solo Guard×Harness (fuera filas «Avisa→systemMessage» y «.sh directo»); test de capacidad exige (evento, campos) p. ej. ("source","compact"), no solo evento; coherencia con tools por harness y «≥1 guard S/Degradado por entrada»; recall-compact en Codex NoSoportado hasta que un smoke lo vea.
Cambios: (1) 0.166.0 compacta con PreCompact/PostCompact (core/src/compact.rs:197,225), PostCompact sin additionalContext (hooks/src/events/compact.rs:46; discovery.rs:686-700); binario 0.166.0 tiene «startupresumeclearcompact» pero quién fija SessionStartSource::Compact sin dato; golden solo startup → `Guard::requiere()` y finding nombrado. (2) hook_coherencia.rs::ninguna_entrada_apunta_a_un_no_soportado itera todos los guards del evento → rompe con sub-guards de soporte mixto; invariante «≥1 S/Degradado por entrada» + informe. (3) matriz.rs:42-46 tools() de Claude; tools(h) o Clase→nombres; replicar Exact/regex de Codex matcher.rs:12-22. (4) evento() fijo PreToolUse; Decision sin Avisa; combinador propio de SessionStart. (5) additionalContextLimit configurable en la entrada; umbral spill = 4×límite; reafirmación/avisos al final. (6) install.sh:12 set -euo pipefail → `exo setup codex` con patrón `|| ec=$?` como doctor (119-123); escribir tmp+rename. (7) ids: panic → `hook-panic` (ya existe); mantener `compact`; reasons de project-rules-skip con `_`. (8) state-migrate-failed silencioso si state_dir no escribible: aceptable, doctor lo marca.

## Verificado por el padre
- matriz.rs:37-46 evento() fijo PreToolUse; tools() con nombres de Claude.
- hook_coherencia.rs:89-101 itera todos los guards del evento.
- install.sh:12 `set -euo pipefail`.
- core/src/compact.rs:197,225 run_pre_compact_hooks/run_post_compact_hooks.

# Review adversarial sección 4 (Fable) — 2026-11-03

Recomendación: (A) ajustes, tres cambian qué se prueba. Más smoke y menos tests sería peor: el smoke es del dueño, fuera de CI y depende de que el modelo obedezca.
Cambios: (1) fixture de recall = tempfile + indexa(kb,&db) + common::con_config (engine/tests/recall.rs:50,104), descarga ONNX → job `test`, no `plugin-tests`; abre_db_en_memoria no sirve (recall pide rutas, recall.rs:221,337). (2) El golden de recall no sobrevive: stubea `exo recall` por EXO_BIN (test-exo-recall-golden.sh:65); escenario 1 → aserción Rust «additionalContext empieza por ## Contrato de memoria» o golden con KB fixture; escenario 2 (reafirma-compact) porta. (3) Inventario = 61 (52+2+7); ~11 de plumbing por stub a divergencias; mapeo verificable con baseline+divergencias como engine/scripts/paridad-hook.sh:36-45. (4) HEAD trae hooks/schema/generated/*.output.schema.json; post-compact.output sin hookSpecificOutput (confirma marca); session-start source incluye fork → vendorizar 6 schemas con hash, validan HEAD no 0.166.0. (5) Index sin reloj: stub bloquea en un lock del test; test-windows corre solo en push a main con cambio de engine (ci.yml:352-353) → declarado; test-exo-index.sh se borra. (6) exo config --json hoy: embeddings,index,kb; _state-dir.sh no existe; sin bun en CI → grep del literal TS. (7) `-c model_auto_compact_token_limit` existe en 0.166.0; segundo turno con `codex exec resume --last "…"`; HEAD encola SessionStartSource::Compact (core/src/session/mod.rs:4801), 0.166.0 sin dato. (8) Latencias medidas: recall 8 ms, rules 4 ms, git log 4 ms, hook PreToolUse 2 ms → medir el hook portado. (9) 4.2.4 es aserción, no invalidación; 4.2.7 no pre-registrable → finding de doctor. (10) Faltan: paridad Claude de recall bloquea merge; hook_coherencia.rs:95 asserta harness claude y solo lee hooks.json → entradas() de codex.json + gemelo de test-hooks-json.sh; test-versiones.sh:71-75 fija literal 0.14.0. (11) cmd /C raw: una línea basta.
Invalidación: >18 de 61 casos (30 %) a divergencia → portar función a función con la suite bash viva; si `codex exec resume --last` no produce segundo UserPromptSubmit, post-compact sin dato y se retira 4.2.6.

## Verificado por el padre
- engine/scripts/paridad-hook.sh:36-45 baseline/divergencias con comm.
- test-versiones.sh:71-75 literal 0.14.0.
- 52 + 2 + 7 = 61 pass únicos.
- test-exo-recall-golden.sh:65 EXO_BIN=$STUB_FELIZ.
- exo config --json keys: embeddings, index, kb.
