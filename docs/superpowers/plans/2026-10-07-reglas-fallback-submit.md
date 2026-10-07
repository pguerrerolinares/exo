# Plan: fallback de reglas de proyecto por `prompt.submit` bajo la política de org

> For agentic workers: ejecución con `exo:orchestrate`.

**Goal:** que las reglas de proyecto lleguen al modelo también cuando la política de org (`cc-plugin-sec-default`) se salta `prompt.compose`, y que el testigo de Stop distinga "el mod no cargó", "cargó y ningún canal entregó" y "entrega degradada".

**Architecture:** el fallback vive en el mod (`register.ts`): un hook `prompt.submit` adjunta `FRAMING` + reglas como `context` del prompt cuando compose no está vivo. El latido `hb-<sid>` gana el campo `via` (`compose|submit|none`); `session.start` lo siembra con `via:"none"` y compose/submit lo sobrescriben. `document-remind.sh` lee `via` y aplica la tabla de tres estados. No hay hook clásico nuevo ni cambios en `exo-recall.sh`.

**Tech Stack:** TypeScript (mod, `claude plugin test`), bash + jq (testigo, e2e, harness de evals).

**Spec:** `docs/superpowers/consultas/2026-10-07-reglas-fallback-sessionstart/consultor.md` (consulta; mejora la spec `docs/superpowers/specs/2026-10-06-reglas-proyecto-2d-design.md`, que T4 actualiza).

**Global Constraints:**
- Fallback en `prompt.submit` con `context`; la política NO salta `prompt.submit`, `session.start`, `turn.*` ni `session.append` (medido 2026-10-07, claude 2.1.292). SÍ salta `prompt.compose`, `prompt.context` y todos los `classic.*`.
- Latido: `~/.claude/exo-rules/hb-<sid>` = `{status:"ok"|"skip"|"error", reason?, n, error?, via:"compose"|"submit"|"none", org?}`. `via` es obligatorio en todo latido que escribe el mod. Sin `HOME` o sin `sid` no se escribe (invariante actual).
- `session.start` escribe `{status:"error", n:0, via:"none", error:"sin entrega"}` y NO pisa un `hb` existente (`/clear` no re-dispara `session.start`; `--resume` conserva sid).
- Variable de entorno del seam: `EXO_RULES_FORZAR_SUBMIT`, activa solo con el valor exacto `"1"`, leída con `$.env.get`.
- Clave de `$.store`: `"compose"` (boolean, "en la última sesión de esta máquina compose corrió"); `"org"` (string, `~/.claude.json` → `oauthAccount.organizationType`, solo pista, nunca gate).
- Regla de entrega del submit: `entregar = FORZAR || (!composeVivo(sid) && (turno >= 2 || store.compose !== true))`. Turno 1 + `store.compose === true` + compose no vivo → no entregar y hb `via:"none"`.
- Texto entregado: el mismo `FRAMING` de `register.ts:1-2` + `reglas.map(x => "- " + x).join("\n")`; no se inventa otro framing. Un bloque en `context`.
- Testigo, tres estados: sin `hb` = "no cargó" (copy actual intacto, `reason=sin_latido`); `hb.via=="none"` con `ss` ok = grita, `reason=sin_canal`; `hb.via=="submit"` = log `reason=entrega_degradada` y aviso visible.
- Copy `sin_canal` (literal): `⚠ el mod de reglas de proyecto cargó pero ningún canal entregó (via=none): SessionStart vio ok n=<SS_N>; ¿política de la org nueva o exo rules falla?`
- Copy `entrega_degradada` (literal): `ℹ reglas de proyecto entregadas por canal degradado (política de la org): n=<HB_N>`. Prefijo en una constante de shell (`DEGRADADO_PREFIJO`, por defecto `ℹ`); si el eval `ars` da placebo pasa a `⚠`.
- Aviso `entrega_degradada`: una vez por día natural (sentinel `${SENTINEL_DIR}/claude-rules-degradado-<YYYYMMDD>`); el log se escribe siempre. Solo si `hb.status=="ok"` y `n>0`.
- Eval `ars`: criterio pre-registrado de una línea: `>=4/10` cumplen = canal degradado útil; `<4` = placebo y el aviso pasa a `⚠`.
- Cap de 6.144 B y `additionalContext`: no se tocan. `exo-recall.sh` sin cambios.
- Versión del plugin: `1.7.0` en `plugin.json` y `marketplace.json` (engine y `ENGINE_MIN` no cambian).
- Verde = suites enteras: `cargo test --manifest-path engine/Cargo.toml && bash scripts/test-plugin.sh && bash scripts/test-versiones.sh && bash scripts/test-release-publish.sh && bash scripts/test-hooks-json.sh && bash scripts/plugin-bump-gate.sh` (`test-plugin.sh` recoge `test-reglas-mod.sh` y `test-document-remind.sh` por glob).

## Decisiones tomadas por defecto (recomendaciones del consultor, §5)

1. **Turno 1, cuenta personal, store vacío:** se duplica una vez (submit + compose). Aceptado el hueco inverso de un turno solo cuando `store.compose===true` y la cuenta pasa a Team (cubierto por turno >= 2).
2. **Aviso degradado:** una vez por día (no por sesión); después de que el eval `ars` decida, pasa a solo log si es útil o a `⚠` si es placebo (seguimiento fuera de este plan).
3. **Eval `ars`:** tarea separada, requiere OK de Paul para lanzarse (~2 USD). Se orquesta tras el mod porque necesita el seam `FORZAR`; su veredicto solo afecta a una constante (`DEGRADADO_PREFIJO`).
4. **`organizationType`:** solo pista (campo `org` del hb y clave `org` del store, que el testigo copia al log); no es gate.
5. **Hueco de auto-compact a mitad de un turno agéntico largo (S6):** se DECLARA en la spec 2d como fuera de alcance y queda para v2 (`session.compact` + `$.session.append`).
6. **Derivadas (el consultor no las fija; elegidas por coherencia con sus tablas):**
   - Estado del mod por sid (`Map`/`Set` de módulo), no global, porque `/clear` cambia el sid sin recargar el mod.
   - Cuando el submit entrega en turno >= 2 sin compose, escribe `store.compose=false` una vez (cierra la fila "store `compose:false`" de la tabla S2).
   - `FORZAR` hace inerte a compose (no entrega, no marca `composeVivo`, no toca store ni hb). Razón: sin esto, en cuenta personal el eval `ars` y el e2e `forzado` medirían compose + submit, no el canal degradado. Supone ajustar el test `forzar_submit` del consultor (ver "Puntos de la spec que no cuadran").
   - `FORZAR` no escribe en `$.store` (un seam de test no envenena la memoria real).
   - El submit no discrimina `e.origin`: cualquier prompt cuenta como turno.
   - La sonda versionada `scripts/sonda-politica.sh` (opcional en el consultor) NO se incluye; queda como seguimiento.

## Puntos de la spec que no cuadran con el código real

- `register.test.ts` hoy NO mockea `fs.exists`/`fs.read`/`$.store`; el helper `correr` solo mockea env, session, `fs.write`, `process.run` y compose. T1 lo extiende (API verificada: `$.fs.exists`, `$.fs.read`, `mock.store(on, entries)`, `$.store.get/set` en `claude-code.d.ts` 2.1.292).
- Los tests existentes `latido_ok` y `skip_no_anade` comparan el hb con `toEqual` exacto: pasan a incluir `via:"compose"`.
- `forzar_submit` del consultor ("entrega aunque compose esté vivo") se reformula como `forzar_compose_inerte` + `forzar_entrega_submit` (ver decisión 6).
- Un `--plugin-dir plugins/exo` completo en el eval cargaría también los hooks clásicos (recall de KB) y contaminaría el brazo: T3 monta un plugin mínimo (solo el mod).
- El harness de evals corre en Linux (`/home/paul/...`) con claude 2.1.291; esta máquina es W11 con 2.1.292.

## Olas

[Orientativa: manda `orchestrate/scripts/task-dag`.]

- Ola 1, en paralelo: T1 (mod), T2 (testigo). Ficheros disjuntos; solo comparten el contrato del hb (`via`) fijado en Global Constraints.
- Ola 2: T3 (eval `ars`), consume el seam `FORZAR` de T1. El lanzamiento real exige OK de Paul.
- Ola 3: T4 (wiring: versión, e2e, spec 2d), consume T1-T3.

### Task 1: mod `prompt.submit` + latido tipado por canal

**Files:**
- Modify: `plugins/exo/hooks/register.ts`
- Test: `plugins/exo/hooks/register.test.ts`

**Interfaces:**
- Produces: `type Latido = { status: "ok" | "skip" | "error"; reason?: string; n: number; error?: string; via: "compose" | "submit" | "none"; org?: string }`
- Produces: `consultar($, home, cwd) -> Promise<{ hb: Omit<Latido, "via">; reglas: string[] }>` (misma lógica actual; `via` lo pone el llamador)
- Produces: `on("session.start", ($, e, next))` siembra el hb `via:"none"` si no existe y guarda `org` en store; devuelve `next(e)` intacto.
- Produces: `on("prompt.submit", ($, e, next))` aplica la regla de entrega de Global Constraints; con entrega devuelve `next({ ...e, context: [...(e.context ?? []), FRAMING + reglas] })`.
- Modifies: `on("prompt.compose", ...)`: marca `composeVivo(sid)`, escribe hb `via:"compose"`, `$.store.set("compose", true)` solo la primera vez por sid; con `FORZAR` devuelve `await next(e)` sin más.
- Consumes (API del host, verificada en `claude-code.d.ts`): `PromptSubmitInput.context?: readonly string[]`, `PromptSubmitResult.context`, `$.fs.exists(path): Promise<boolean>`, `$.fs.read(path): Promise<string>`, `$.store.get(key): Promise<unknown>`, `$.store.set(key, value): Promise<void>`, `$.env.get(name)`, `$.session.id()`.

**Tests:** (cada uno se escribe y se ve fallar antes del código, `exo:tdd`; el helper `correr` se extiende con mocks de `fs.exists`, `fs.read`, `mock.store` y un `submit(texto)` que dispara `$.prompt.submit`)
- `submit_entrega_sin_compose`: store vacío, sin compose previo, un submit → `context` termina en `FRAMING + "- r1\n- r2"`, hb `{status:"ok", n:2, via:"submit"}`, exactamente 1 llamada a `exo rules`. Falla si el fallback no adjunta el bloque o consulta dos veces.
- `submit_calla_si_compose_vivo`: compose corrió antes en el mismo sid → `context` sin cambios y 0 llamadas a `exo rules`. Falla si se duplica la entrega en cuenta personal.
- `turno1_store_compose_no_entrega`: store `compose:true`, turno 1, compose no vivo → sin context añadido, 0 llamadas, hb `via:"none"`. Falla si se duplica en cada sesión de una cuenta personal.
- `turno2_sin_compose_entrega_aunque_store`: store `compose:true`, dos submits sin compose → el 2.º entrega y deja `store.compose===false`. Falla si el cambio personal a Team deja sesiones sin reglas.
- `turnos_por_sid`: submit en sid A (turno 1 sin entrega por store) y luego sid B → B se trata como turno 1, no como 2. Falla si el contador es global (rompería `/clear`).
- `session_start_marca_via_none`: `session.start` escribe `{status:"error", n:0, via:"none", error:"sin entrega"}`; un compose posterior lo sobrescribe con `via:"compose"`. Falla si el testigo no tendría señal de "cargó".
- `session_start_no_pisa_hb`: `fs.exists` true → 0 escrituras. Falla si `--resume` borra un `via:"submit"` real.
- `submit_sin_hb_siembra_none`: submit sin entrega y sin hb previo (caso `/clear`) → escribe `via:"none"`. Falla si tras `/clear` el Stop ve "no cargó".
- `forzar_entrega_submit`: `EXO_RULES_FORZAR_SUBMIT=1` → entrega en turno 1 con store `compose:true`; no escribe `$.store`. Falla si el seam no fuerza o contamina el store.
- `forzar_compose_inerte`: con `FORZAR`, compose devuelve las secciones base sin `exo:reglas-proyecto`, sin hb y sin `composeVivo`. Falla si el eval/e2e medirían dos canales.
- `submit_skip_no_adjunta`: `exo rules` skip → context intacto, hb `{status:"skip", reason:"sin_seccion", n:0, via:"submit"}`.
- `submit_nunca_lanza`: `exo rules` con JSON inválido o `process.run` lanzando → `next(e)` intacto (mismo texto, mismo context), hb `status:"error", via:"submit"`, sin excepción. Falla si un fallo de `exo` rompe el prompt del usuario.
- `org_pista_en_hb`: `fs.read` de `/h/.claude.json` devuelve `{"oauthAccount":{"organizationType":"claude_team"}}` → hb escrito por submit lleva `org:"claude_team"`; `fs.read` que rechaza → hb sin `org` y sin error. Falla si la caché no documentada puede romper el hook.
- Existentes `latido_ok` y `skip_no_anade`: ajustados a `via:"compose"`.

**Verificación:** `claude plugin validate plugins/exo && claude plugin test plugins/exo` → todos los tests (previos + nuevos) en verde; después la suite entera (comando de Global Constraints) → verde.

**Review Focus:**
- Orden compose/submit en el turno 1 de una cuenta personal: ambos pueden escribir hb; el test debe aceptar cualquier orden (`submit_calla_si_compose_vivo` y `session_start_marca_via_none` cubren los dos sentidos).
- Latencia: el submit paga `exo rules` (timeout 3000) en el camino del eco del prompt; no debe consultar si no entrega (`turno1_store_compose_no_entrega`, `submit_calla_si_compose_vivo`).
- Estado por sid sin fugas: `turnos_por_sid`.
- `fs.read` de `~/.claude.json` puede rechazar por >4 MiB: `org_pista_en_hb`.
- Un fallo de `fs.write` del hb no debe impedir la entrega del context: `submit_nunca_lanza` (variante con `fs.write` denegado, entrega igualmente).

**Notas:** `HOME` se resuelve como hoy (`HOME` o `USERPROFILE`). El `.catch` explícito en el submit es obligatorio aunque el host deje pasar el prompt ante una excepción.

### Task 2: testigo de Stop con tres estados

**Files:**
- Modify: `plugins/exo/scripts/document-remind.sh`
- Test: `plugins/exo/scripts/test-document-remind.sh`

**Interfaces:**
- Consumes: formato de hb con `via` y `org` @Task 1 (contrato de Global Constraints; los tests escriben hb a mano, no dependen del código de T1)
- Produces: nuevas razones de log `sin_canal` y `entrega_degradada` en `project-rules-witness`; `reason=entrega_degradada` añade ` org=<org>` al payload si el hb lo trae.
- Produces: constante `DEGRADADO_PREFIJO="ℹ"` al principio del bloque del testigo (la cambia T3/T4 si el eval da placebo).

**Tests:** (en `test-document-remind.sh`, mismo estilo `pass`/`fail`; cada uno falla antes del cambio)
- `via_none_con_ss_ok_grita`: `ss` ok n=1 + `hb` `{"status":"error","n":0,"via":"none","error":"sin entrega"}` → `systemMessage` EXACTO igual al copy `sin_canal` con `n=1` y log `reason=sin_canal`; segundo Stop calla (sentinel). Falla si cae en el copy viejo `no_entrego` o se silencia.
- `via_none_con_ss_skip_calla`: `ss` skip + `hb` via none → sin mensaje, log `reason=hb_error`. Falla si grita sin reglas que entregar.
- `via_submit_log_y_aviso_una_vez`: `ss` ok n=1 + `hb` `{"status":"ok","n":1,"via":"submit"}` → `systemMessage` EXACTO igual al copy `entrega_degradada` con `n=1`, log `reason=entrega_degradada`; segundo Stop de OTRA sesión el mismo día → log sí, mensaje no (sentinel diario). Falla si avisa en cada sesión.
- `via_submit_org_en_log`: hb con `"org":"claude_team"` → el log contiene `org=claude_team`.
- `via_submit_skip_no_avisa`: `hb` `{"status":"skip","reason":"sin_seccion","n":0,"via":"submit"}` con `ss` skip → sin mensaje y sin `entrega_degradada`. Falla si avisa de degradación cuando no se entregó nada.
- `via_submit_n_distinto_adjudica`: `ss` ok n=3, `hb` ok n=2 via submit, stub del engine responde ok n=2 → `reason=kb_cambio` y `ss` refrescado (la adjudicación sigue funcionando con `via:"submit"`). Con el stub en n=3 → grita `no_entrego` con copy actual. Falla si `via` rompe `kb_cambio`.
- `sin_hb_sigue_siendo_no_cargo`: sin hb → `NOCARGO` y `reason=sin_latido`, intactos (regresión).
- `hb_sin_via_legacy`: hb `{"status":"ok","n":2}` (1.6.1) se trata como compose: sin mensaje de degradación. Falla si el testigo exige `via` y grita a latidos antiguos.
- `via_none_hb_corrupto`: `no es json{` sigue siendo `no_entrego` (regresión; no debe leerse `via` de un JSON roto).

**Verificación:** `bash plugins/exo/scripts/test-document-remind.sh` → `0 fallos`; después la suite entera (Global Constraints) → verde.

**Review Focus:**
- La lectura de `via` va ANTES de la rama `HB_ST=error && SS_ST=ok` (`document-remind.sh:43-45`), o `via:none` caería en `no_entrego` con el copy equivocado (`via_none_con_ss_ok_grita`).
- `jq` sobre un hb sin `via` o corrupto no debe abortar el script (`hb_sin_via_legacy`, `via_none_hb_corrupto`).
- El sentinel diario no debe silenciar el sentinel por sesión ni el recordatorio `/document` (`via_submit_log_y_aviso_una_vez` más el caso de recordatorio existente).
- `kb_cambio` y `sin_verdad` siguen sin crear sentinel.

**Notas:** `date +%Y%m%d` para el sentinel diario; `REMIND_SENTINEL_DIR` ya aísla los tests.

### Task 3: eval `ars` del canal degradado

> Lanzar las corridas reales cuesta ~2 USD y REQUIERE OK EXPLÍCITO DE PAUL. Construir el brazo y sus tests (sin claude) no gasta nada; el paso de lanzamiento se detiene y pregunta.

**Files:**
- Modify: `evals/ablacion-k/harness/correr.sh`
- Create: `evals/techo-reglas-2/preregistro-ars.md`
- Create: `evals/techo-reglas-2/tareas-ars.tsv`
- Create: `evals/techo-reglas-2/test-correr-ars.sh`
- Create: `evals/techo-reglas-2/verdict-ars.md` (solo tras lanzar)

**Interfaces:**
- Consumes: seam `EXO_RULES_FORZAR_SUBMIT` y la regla de entrega @Task 1
- Produces: brazo `ars` en `correr.sh`: `a0` exacto (mismo `claude-md.md`, stub de `exo`, KB denegada, `--setting-sources ""`, `autoMemoryEnabled:false`) más un plugin mínimo en `$O/plugin-ars/` (copia de `plugins/exo/hooks/register.ts` con un `hooks.json` que solo lista `"modules": ["./register.ts"]` y un `plugin.json` mínimo), cargado con `--plugin-dir $O/plugin-ars`, `EXO_RULES_FORZAR_SUBMIT=1` y un shim `exo` en `$O/stub-ars/` (antes en PATH) que a `exo rules --json` responde `{"schema_version":2,"command":"rules","data":{"status":"ok","repo":"x","note":"x.md","rules":["<contenido de $K_REGLA_FILE sin salto final>"],"ignored_lines":[]}}`.
- Produces: `tareas-ars.tsv` = las 10 tareas del suelo de `tareas.tsv` con brazos `ars:2`, sin controles ni `a0` (se reutilizan `a0` y `arp` ya sellados de techo-reglas-2).

**Tests:** (`test-correr-ars.sh`, sin claude, con `K_ENSAYO=1`, estilo `test-correr-arp.sh`)
- `ars_sin_regla_falla_ruidoso`: sin `K_REGLA_FILE` → exit 2 con mensaje `ars requiere K_REGLA_FILE no vacío`. Falla si el brazo corre sin regla.
- `ars_cmdline_carga_solo_el_mod`: el `cmdline.txt` lleva `--plugin-dir` apuntando a `plugin-ars` y `$O/plugin-ars/` NO contiene `scripts/` ni hooks de comando (`jq '.hooks' == null`). Falla si el brazo hereda los hooks clásicos (recall de KB) y contamina la medición.
- `ars_env_forzar`: el entorno de la corrida incluye `EXO_RULES_FORZAR_SUBMIT=1`; `a0` y `arp` no. Falla si el seam se filtra a otros brazos.
- `ars_shim_devuelve_la_regla`: ejecutar `$O/stub-ars/exo rules --json` imprime un envelope con `data.status=="ok"` y `data.rules[0]` igual a la regla. Falla si el mod recibe n distinto de 1.
- `ars_sin_claude_md_distinto`: `append` de `ars` == `claude-md.md` (sin framing en system prompt; el framing llega solo por el mod). Falla si se mide arp disfrazado.
- `preregistro_ars_valido`: `preregistro-ars.md` contiene la línea `>=4/10 cumplen = util; <4 = placebo`, k=2, las 10 tareas, el orden barajado y la versión de claude. Falla si el umbral no está sellado antes de correr.

**Verificación:** `bash evals/techo-reglas-2/test-correr-ars.sh && bash evals/techo-reglas-2/test-correr-arp.sh && bash evals/techo-reglas-2/validar-preregistro.sh` → todo `ok`/exit 0. Después, SOLO con OK de Paul: lanzar la tanda (20 corridas) con `correr-techo.sh` y `TECHO_EXP=evals/techo-reglas-2`/`TECHO_TAREAS=evals/techo-reglas-2/tareas-ars.tsv` → breakers sin saltar; adjudicar con `evaluar.py` y escribir `verdict-ars.md` con `ars` n/10, comparado con `arp` 6/10 y `a0` 0/10, y la decisión del prefijo (`ℹ` o `⚠`).

**Review Focus:**
- Contaminación por hooks clásicos al cargar el plugin completo (`ars_cmdline_carga_solo_el_mod`).
- Que `ars` mida el canal y no el framing: el texto entregado es el `FRAMING` del mod; comprobar que equivale a `framing.txt` con `{{REGLA}}` sustituido (mismo texto que `arp`, salvo el saltos de línea final) y anotar cualquier diferencia en `verdict-ars.md`.
- Versión de claude: las corridas `a0`/`arp` selladas son 2.1.291; si `claude --version` difiere al lanzar, escribir errata antes de adjudicar (política de re-intentos: cero).
- Cero re-intentos y breakers de `correr-techo.sh` heredados; el gasto tope no debe superar el presupuesto aprobado.

**Notas:** si el veredicto es placebo (`<4/10`), el cambio del prefijo a `⚠` lo hace T4 (una constante y su test); si no se lanza la eval, el prefijo queda en `ℹ` y se anota como pendiente en el PR.

### Task 4: wiring final, versión, e2e y spec 2d

**Files:**
- Modify: `plugins/exo/.claude-plugin/plugin.json`
- Modify: `.claude-plugin/marketplace.json`
- Modify: `scripts/e2e-reglas-proyecto.sh`
- Modify: `docs/superpowers/specs/2026-10-06-reglas-proyecto-2d-design.md`
- Modify (solo si T3 da placebo): `plugins/exo/scripts/document-remind.sh`, `plugins/exo/scripts/test-document-remind.sh`

**Interfaces:**
- Consumes: mod con `via` y seam `FORZAR` @Task 1
- Consumes: testigo de tres estados @Task 2
- Consumes: veredicto del eval `ars` (`verdict-ars.md`, opcional) @Task 3
- Produces: `plugin.json` y `marketplace.json` en `1.7.0` (los dos iguales; `test-versiones.sh`).
- Produces: e2e con casos `positivo`, `control`, `forzado`, `resume`. Todos imprimen `via` del hb.

**Tests:**
- `e2e positivo`: la aserción pasa de `ok n=1` a `ok n=1` con `via ∈ {compose,submit}` (se imprime cuál); `ss` ok n=1. En esta máquina (Team) debe salir `via=submit`, en cuenta personal `via=compose` (o `submit` por la duplicación de turno 1). Falla si la entrega bajo la política no llega al modelo.
- `e2e forzado`: con `EXO_RULES_FORZAR_SUBMIT=1` → respuesta con el codeword `REGLAS-2D-7Q` y hb `ok n=1 via=submit`. Falla si el seam no simula la política.
- `e2e control`: sin la sección → sin codeword, hb `skip/sin_seccion` (se mantiene).
- `e2e resume`: tras el caso forzado, `claude -p --resume <sid>` con una segunda pregunta → codeword otra vez y hb `via=submit`. Cubre el punto "no verificado" del consultor (`--resume`).
- Informativo (sin gate): el e2e imprime el tiempo de pared de `exo rules` (el coste del submit en el camino del eco; sin umbral).
- Si T3 dio placebo: `via_submit_log_y_aviso_una_vez` pasa a esperar el prefijo `⚠`. Falla si el prefijo y el veredicto no coinciden.
- Spec 2d: `bash scripts/test-docs-vivos.sh` (si valida referencias de docs) en verde.

**Verificación:** suite entera (comando de Global Constraints) → verde. Después `bash scripts/e2e-reglas-proyecto.sh` → `PASS` en positivo, forzado, control y resume en esta máquina (Team); pegar la salida en el PR. Repetir en una cuenta personal si hay acceso; si no, anotarlo como no verificado.

**Review Focus:**
- `plugin-bump-gate.sh`/`test-plugin-bump.sh` aceptan `1.7.0` frente a `1.6.1`.
- El e2e usa el `exo` del repo (`PATH` con `engine/target/release`) y `EXO_KB` del fixture: el codeword solo existe ahí; `FORZAR` debe llegar al proceso del mod (`$.env.get` lee el entorno de `claude`).
- El caso `forzado` no debe depender de que compose esté bypassed: con `FORZAR` compose es inerte por construcción.
- Spec 2d actualizada con: la tabla de hechos medidos de la política en «Hechos verificados» (fila para `session.start`, `prompt.submit`, `$.session.append` = corren; `prompt.compose`, `prompt.context`, `classic.*` = bypassed); las tres filas del testigo en «Gritos»; en «Fuera de alcance» el hueco de auto-compact a mitad de turno (v2) y la re-entrega vía `$.session.append`/`session.compact` sin medir. El comentario «Nada va a additionalContext» (`exo-recall.sh:205`) y «Cap de 6.144 B: no se toca» siguen ciertos: no se editan.

**Notas:** los copys del testigo y los nombres de campo están fijados en Global Constraints; la spec solo los referencia, no los redefine.

## Preguntas abiertas

1. ¿Se acepta la derivada de que `FORZAR` haga inerte a compose (decisión 6)? Es lo que hace válido el eval `ars` en cuenta personal, pero difiere del test `forzar_submit` del consultor.
2. ¿Se incluye la sonda versionada `scripts/sonda-politica.sh` (opcional en la consulta)? Por defecto no.
3. Tras el eval `ars`: ¿el aviso pasa a solo log si el canal resulta útil? Por defecto se decide fuera de este plan.

## Self-review

- Cobertura: S1/S2 (T1), S3/S7 (T2, T4), S4 (T3), S8 y Hechos (T4), tests de §4 del consultor (T1: 7 originales + extras; T2: 4 originales + extras; T4: e2e), decisiones de §5 (arriba). S5 y S6 se declaran (no se implementan). Pendiente consciente: sonda versionada.
- Placeholders: ninguno; los textos que fijan valores (copys, claves, env) están literales en Global Constraints.
- Tipos: `Latido.via`, `FORZAR`, `store.compose`, `DEGRADADO_PREFIJO` y las razones `sin_canal`/`entrega_degradada` coinciden entre T1-T4.
- Proporción: sin cuerpos de función; solo firmas, valores y aserciones.
