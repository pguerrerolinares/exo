# Consulta — fallback de reglas de proyecto cuando la política de org se salta `prompt.compose`

**Veredicto: ADELANTE CON CAMBIOS** — el fallback hace falta, pero **no** como hook clásico de SessionStart: va **dentro del mod**, por `prompt.submit` (que la política no se salta), con latido tipado por canal y testigo que distingue tres casos.

**Fecha**: 2026-10-07. **Consultor**: Fable (sesión delegada). **Alcance**: evaluación adversarial; nada implementado. Todo lo marcado «medido» se midió hoy en esta máquina (W11, cuenta Team «Kutxabank Investment», `claude` 2.1.292, `exo` 0.5.0, plugin 1.6.1 @ `8eec2ab`) con dos mods-sonda en el scratchpad (`probe-mod/`, `probe-mod2/`, `claude -p --debug --setting-sources "" --plugin-dir`). Lo demás va marcado como inferencia.

---

## 0. Recomendación en una pantalla

1. **No añadir un hook clásico**. El canal propuesto (`SessionStart` → `additionalContext`) funciona pero es el peor de los disponibles: es el v1 que midió 4/11 (`evals/techo-reglas-2/verdict.md:16`), no se re-entrega tras `/compact` sin tocar el filtro `source != compact` de `exo-recall.sh:208`, no puede coordinarse con compose (corre antes) y en W11 un hook nuevo son ~1,7 s p95 (spec 2d §Arquitectura).
2. **El fallback vive en `register.ts`**: un hook `prompt.submit` que, cuando compose no está vivo, llama a `exo rules` y adjunta el framing sellado + reglas como `context` del prompt. **Medido hoy**: bajo la política Team, `prompt.submit` corre y su `context` llega al modelo; `prompt.compose`, `prompt.context` y todos los `classic.*` se saltan.
3. **Coordinación sin carrera**: variable de módulo `composeVivo` (la pone el hook de compose al ejecutarse) + `$.store` («en la última sesión de esta máquina compose corrió»). Turno 1 sin memoria → entregar (ausencia ≠ evidencia, se acepta una duplicación en la primera sesión de una cuenta personal); turno ≥2 → decide `composeVivo`.
4. **Latido con canal**: `hb-<sid>` pasa a `{status, n, via: "compose"|"submit"|"none", ...}`. `session.start` (que sí corre bajo la política, medido) escribe `via:"none"` nada más cargar. El testigo de Stop distingue así: **sin hb** = el mod no cargó; **hb via none** = cargó y ningún canal entregó (política nueva, o compose/submit rotos); **hb via submit** = entrega degradada por política, se loguea y se avisa una vez.
5. **Antes de fiarse del canal degradado, medirlo**: un brazo `ar-submit` en el harness de `evals/techo-reglas-2` (framing de autoridad + canal de prompt). El verdict ya avisó de que canal y framing están confundidos (`verdict.md:81`); el 4/11 del v1 era sin framing.

---

## 1. Hechos medidos hoy (sondas bajo política Team)

Mod de tier `user` cargado con `--plugin-dir`; política `cc-plugin-sec-default@builtin` sentada outermost («the organization is team»). Debug log en `~/.claude/debug/<sid>.txt` (solo existe con `--debug`; ningún hook puede contar con él en sesión normal).

| Evento / llamada | ¿Corre bajo la política? | Evidencia |
|---|---|---|
| `session.start` | **sí** (`$.session.id()` ya disponible, `e.cwd` presente) | `evt-session.start` escrito; `settled in 44.8ms` |
| `turn.start` | **sí** | `evt-turn.start` |
| `prompt.submit` con `context` | **sí, y el modelo lo lee** | respuesta `PROBE-SUBMIT-OK`; `settled in 49.8ms` |
| `$.session.append` (fila user `isMeta`) desde `session.start` | **sí, y el modelo lo lee** | respuesta `PROBE-APPEND-OK` |
| `$.session.append` desde `turn.start` | se almacena pero **no entra en la primera petición de ese turno** | fila con uuid, modelo no la lista |
| `prompt.compose` | **bypassed** | `prompt.compose bypassed by cc-plugin-sec-default (tier user); beneath runs` |
| `prompt.context` | **bypassed** | ídem |
| `classic.SessionStart`, `classic.UserPromptSubmit`, `classic.Stop` | **bypassed** | ídem (los tres) |
| `$.fs.write`, `$.session.id()` | sí | ficheros escritos |

Lista completa de eventos que engancha la política (del debug log): `classic.*, prompt.section, prompt.context, prompt.compose, skill.prompt, attribution.text, settings.read, tool.describe, command.describe, agent.offer, agent.spawn, tool.register, tool.list, tool.check, plugin.register`. **`prompt.submit`, `session.start`, `session.compact`, `turn.*` y `session.append` no están** — por eso corren. Es el comportamiento de 2.1.292; nada impide que Anthropic amplíe la lista, y el diseño de abajo grita si eso pasa.

Otros hechos:

- `~/.claude.json` → `oauthAccount.organizationType: "claude_team"`, `seatTier: "team_tier_1"`. Es caché no documentada del CLI (el subagente de docs no encontró ninguna variable pública), pero es exactamente la señal que el engine usa. Vale como **pista en el log**, no como gate del diseño.
- En esta máquina `~/.claude/exo-rules/` solo tiene `ss-*`, nunca `hb-*`; `reflex-log` tiene 4 `project-rules-witness reason=sin_latido` hoy. Coherente con la hipótesis: el mod carga (`plugin.register … admitted`) y compose nunca corre.
- `scope: "session"` en `PromptComposeSection` **no es persistencia**: es el lado del límite de caché de prompt (`shared` antes, `session` después; d.ts «PromptComposeScope»). La persistencia de la sección se debe a que compose corre en cada render del system prompt (spec 2d, hecho verificado: `hb` se reescribe por turno).
- El `additionalContext` de un hook clásico de SessionStart entra como bloque `kind: 'hook'` del **primer mensaje de usuario** (d.ts `PromptContextBlocks` / origen `{kind:'hook', event:'SessionStart'}`), no en el system prompt. Tras `/compact` el hook vuelve a disparar con `source=compact` (`SessionStartHookInput.source`) y lo que inyecte va tras el resumen; **lo anterior no se preserva por sí solo** (inferencia a partir del diseño del propio `exo-recall.sh:148-171`, que existe justo para reafirmar tras compactar). Subagentes: no reciben SessionStart (spec 2d, hecho verificado). `-p`: sí (los `ss-<sid>` del e2e lo prueban).

---

## 2. Hallazgos, por severidad

### S1 — La propuesta elige el canal más débil habiendo uno mejor sin coste

`SessionStart` + `additionalContext` es, letra por letra, el **v1** que el gate midió como «sugerencia» (`evals/techo-reglas-2/verdict.md:16`, 4/11 frente a 6/10). Hoy existe `prompt.submit` con `context` (d.ts `PromptSubmitResult.context`: «one block after the prompt as typed»), que la política **no** salta (medido). Ventajas: se adjunta **en cada prompt** (sobrevive `/compact`, `--resume` y `-p` sin lógica extra), corre en el mismo worker que compose (coordinación por variable, sin ficheros ni carreras), no añade proceso en W11 (reutiliza la llamada a `exo rules` que compose ya hacía por turno; en Team compose no corre, así que el coste es neutro) y está **cerca del punto de uso** (la literatura de instruction drift que cita la consulta de #23 §5 apunta a que repetir por turno ayuda; inferencia). Lo que **no** da: autoridad de system prompt. Eso no lo da ningún canal bajo la política; hay que medirlo (S4).

### S2 — El bash SessionStart no puede coordinarse con compose; el mod sí

`exo-recall.sh` corre **antes** del `session.start` del mod (spec 2d, 25-240 ms) y compose corre al primer prompt. Un fallback en SessionStart no sabe si compose va a correr: o duplica siempre en cuentas personales, o inventa un estado en fichero con su propia carrera. Dentro del mod: `on("prompt.compose")` pone `composeVivo = true` en una variable de módulo; `on("prompt.submit")` del turno ≥2 la lee. Solo el **turno 1** es ciego, y ahí `$.store` (persistente entre sesiones, por plugin; d.ts `$.store.get/set`) recuerda si compose corrió la última vez en esta máquina. Casos:

| Situación | Turno 1 | Turno ≥2 |
|---|---|---|
| Cuenta personal, store vacío (1ª sesión) | entrega por submit **y** compose (duplicado, una vez) | solo compose |
| Cuenta personal, store `compose:true` | solo compose | solo compose |
| Team, store vacío o `compose:false` | submit | submit |
| Cambio de cuenta personal→Team con store `compose:true` | **sin reglas** (hueco de un turno) | submit (compose no marcó) |

El hueco de un turno al cambiar de cuenta es el precio de no duplicar; alternativa: entregar siempre en turno 1 y aceptar la duplicación en cuentas personales. Pregunta para Paul (§5).

### S3 — El testigo actual confunde «no cargó» con «cargó y la política lo saltó»; el fix propuesto («distinguir») no tiene señal hoy

`document-remind.sh:31-33`: sin `hb` → «no cargó». Pero el mod **sí** carga en Team; lo que no corre es compose. Ningún hook clásico ni el mod pueden preguntar «¿estoy bypassed?» (no hay API; el debug log solo existe con `--debug`). La señal **sí** disponible: `session.start` corre bajo la política (medido). Si el mod escribe `hb-<sid>` = `{status:"error", via:"none", error:"sin entrega"}` nada más cargar, y compose/submit lo **sobrescriben** con `via:"compose"|"submit"`, el testigo tiene tres estados sin inventar nada:

- **sin hb**: el mod no cargó (register.ts roto, `modules` ausente, plugin deshabilitado) → copy actual.
- **hb `via:none`** con ss ok: cargó y ningún canal entregó → **grita** («el mod cargó pero ni compose ni submit entregaron: ¿política nueva? ¿exo rules falla?»). Cubre el día en que Anthropic añada `prompt.submit` a la lista de la política. Esto es lo que preserva «ausencia ≠ evidencia»: nunca se convierte un fallo real en silencio.
- **hb `via:submit`** con ss ok: entrega degradada por política → log `entrega_degradada` siempre; visible una vez (sentinel ya existe, `WSENT`).

La rama existente `HB_ST=error && SS_ST=ok → no_entrego` (`document-remind.sh:43-45`) ya atraparía `via:none` con el copy equivocado; basta leer `via` antes.

Dos matices: (a) `session.start` **no** vuelve a disparar en `/clear` (d.ts: «never `/clear`»; `session.end` con `reason:'clear'`); con `/clear` cambia el `session_id`, así que el hb inicial de la nueva sesión lo tiene que escribir **el primer `prompt.submit`** si no existe, no solo `session.start`. (b) Un `hb` de `session.start` sin prompt posterior (sesión abierta y cerrada) es `via:none` legítimo; el Stop no se dispara sin turno, así que no grita. Si Paul abre, no escribe y cierra: ni Stop ni grito. Correcto.

### S4 — El canal degradado está sin medir; el 4/11 no es su número

`verdict.md:81`: «canal y framing están confundidos». El v1 era SessionStart **sin** framing de autoridad. Un bloque pegado al prompt **con** el framing sellado es un tercer punto que nadie ha medido. Si sale ≥5/10, el hueco de Team es pequeño; si sale 1/10, hay que saberlo antes de contar con él. El harness existe (`evals/techo-reglas-2/`, 52 corridas = 4,81 USD); un brazo `ar-submit` de 10 tareas × 2 réplicas son ~2 USD. Se puede forzar el canal en cuenta personal con un seam de entorno (`EXO_RULES_FORZAR_SUBMIT=1`, leído con `$.env.get`), que además sirve al e2e (§4).

### S5 — Si pese a todo se hace en bash: va en `exo-recall.sh`, y el cap de 6.144 B no es el problema que parece

`exo-recall.sh:220-224` ya tiene en `$ROUT` el JSON de `exo rules` con las reglas: añadirlas a `$TEXTO` es gratis (cero spawns, sin tocar el timeout de 3 s). El **cap de 6.144 B** se aplica a `exo recall --cap-bytes` (`:36`, `:107-108`), **no** al `TEXTO` final: la reafirmación post-compact (`:165`) y el aviso superpowers (`:198`) ya se añaden por encima del cap sin truncarse. Lo que haría falta: (i) resolver también con `source=compact` (hoy `:208` lo excluye) y re-entregar; (ii) no tocar `ss-<sid>` en compact (el testigo usa ss como foto de arranque); (iii) aceptar que no hay forma de saber si compose entregó (S2). Por eso es plan B, no A. CRLF: `exo rules` ya recorta `\r` (spec §Sección); PATH sin exo: `:34` cae a `~/.local/bin/exo`; timeout: `con_timeout` con fallback perl (`_timeout.sh`).

### S6 — `$.session.append` es canal válido pero peor que `prompt.submit` para esto

Funciona bajo la política y el modelo lo lee (medido), pero solo si se llama **antes** del turno (desde `session.start`); desde `turn.start` entra «from the loop's next top» (reference.md:141) y el primer request del turno no lo lleva (medido). Y una fila en el transcript se resume en `/compact` (inferencia): habría que re-apéndiar en `session.compact` (no está en la lista de la política; sin medir). Útil como **complemento** v2 para el hueco «auto-compact a mitad de un turno agéntico largo»: con `prompt.submit` las reglas re-entran en el **siguiente** prompt del usuario, no en la siguiente petición del mismo turno. Con el system prompt ese hueco no existe. Declararlo, no taparlo en v1.

### S7 — El e2e actual (`scripts/e2e-reglas-proyecto.sh:50-51`) no distingue canal y en Team fallaría por la razón equivocada

Asevera `hb` = `ok n=1` y el codeword; en Team hoy no hay `hb` y el e2e marca FAIL sin decir «política». Con `via` en el latido, el positivo pasa a aceptar `via ∈ {compose, submit}` e imprimirlo; y hace falta un tercer caso forzado (§4).

### S8 — Deriva menor en los comentarios

`exo-recall.sh:205` («Nada va a additionalContext») y spec 2d §Gritos «Cap de 6.144 B: no se toca» siguen siendo ciertos con el diseño recomendado (el fallback va por `prompt.submit`, no por additionalContext). Si se eligiera el plan B, los dos quedarían falsos y hay que reescribirlos; la spec 2d también en «Hechos verificados» (añadir la fila de la política).

---

## 3. Diseño recomendado (concreto)

**`plugins/exo/hooks/register.ts`** (solo este fichero cambia de lógica):

```
estado de módulo: composeVivo = false
session.start:
  sid, home → escribe hb = {status:"error", n:0, via:"none", error:"sin entrega"}   (si ya existe, no pisa: /clear no re-dispara session.start)
  (pista en log, opcional) lee ~/.claude.json → oauthAccount.organizationType → $.store.set("org", tipo)
prompt.compose:
  composeVivo = true; (resto igual) ; hb.via = "compose"; $.store.set("compose", true)
prompt.submit:
  entregar = FORZAR || !composeVivo && ($.store.get("compose") !== true || esTurno≥2)
    · turno 1 y store "compose"===true → no entregar
    · turno ≥2 → entregar sii !composeVivo
  si entregar: consultar(exo rules) → si ok: next({...e, context:[...(e.context||[]), FRAMING + reglas]}); hb = {status, n, via:"submit"}
  si no entregar y no existe hb (caso /clear): hb = {…via:"none"}
```

- `esTurno≥2`: contador de módulo incrementado en `turn.start` (corre bajo la política, medido) o en el propio `prompt.submit`.
- `FORZAR` = `$.env.get("EXO_RULES_FORZAR_SUBMIT") === "1"`: seam de test/eval, documentado como tal.
- El framing es el **mismo texto sellado** (`FRAMING`, `register.ts:1-2`); no se inventa otro para el canal degradado: lo que se mide en S4 es el canal, no un framing nuevo.
- `$.store` crece una clave; en `session.end` no hace falta nada.

**`plugins/exo/scripts/document-remind.sh`**: leer `via` junto a `status`/`n` (`:37`); tabla de §2-S3. Copys nuevos (que fije el plan): «⚠ el mod de reglas cargó pero no entregó por ningún canal (via=none): …» y, una vez por sesión, «ℹ reglas de proyecto entregadas por canal degradado (política de org): n=N». `kb_cambio` sigue adjudicando igual (el `n` del latido es el mismo sea cual sea el canal).

**`exo-recall.sh`**: sin cambios. `ss-<sid>` sigue siendo la foto de arranque.

**Spec 2d**: añadir en «Hechos verificados» la tabla de §1 y en «Gritos» las tres filas del testigo; en «Fuera de alcance» el hueco S6 y la re-entrega tras auto-compact a mitad de turno.

**No hacer**: hook clásico nuevo; `UserPromptSubmit` clásico (mismo grado que SessionStart y un spawn más por prompt en W11); CLAUDE.md generado (lo lee todo el mundo que abra el repo, incluye cuentas sin exo, y es prosa en la zona «lejana» del contexto; además la sección `claudeMd` va por `prompt.context`, que la política sí controla aunque no la borre).

---

## 4. Tests necesarios (para que el fallback no sea otro instrumento mudo)

**`register.test.ts`** (`claude plugin test`, mocks como los actuales):
- `submit_entrega_sin_compose`: store vacío, ningún compose previo, `prompt.submit` → `context` acaba con `FRAMING + "- r1\n- r2"`, hb `{status:"ok", n:2, via:"submit"}`, una llamada a `exo rules`.
- `submit_calla_si_compose_vivo`: compose corrió antes → `prompt.submit` devuelve `context` sin cambios y **cero** llamadas a `exo rules`.
- `turno1_store_compose_no_entrega`: store `compose:true`, turno 1 → sin context, sin llamada, hb `via:"none"`.
- `turno2_sin_compose_entrega_aunque_store`: store `compose:true`, turno 2 sin compose → entrega (cierra el hueco del cambio de cuenta).
- `session_start_marca_via_none`: hb inicial `{status:"error", via:"none"}`; `compose` lo sobrescribe con `via:"compose"`.
- `forzar_submit`: `EXO_RULES_FORZAR_SUBMIT=1` entrega aunque compose esté vivo.
- `submit_nunca_lanza`: `exo rules` con JSON inválido → `next(e)` intacto, hb `error`, sin excepción (un `prompt.submit` que falla deja pasar el prompt, d.ts remarks; pero un `.catch` explícito es más honesto).

**`test-document-remind.sh`**: `via_none_con_ss_ok_grita` (copy nuevo, `reason=sin_canal`), `via_submit_log_y_aviso_una_vez` (`reason=entrega_degradada`, segundo Stop callado), `sin_hb_sigue_siendo_no_cargo` (copy actual intacto), `via_submit_n_distinto_adjudica` (kb_cambio sigue funcionando con `via:"submit"`).

**`scripts/e2e-reglas-proyecto.sh`**: positivo acepta `via ∈ {compose,submit}` y lo imprime; nuevo caso `forzado`: `EXO_RULES_FORZAR_SUBMIT=1` → codeword en la respuesta y hb `ok n=1 via=submit`. En esta máquina (Team) el positivo debe salir `via=submit`; en la personal, `via=compose`. **Ambas salidas se pegan en el PR**: es la única prueba de que el fallback entrega de verdad bajo la política.

**Sonda versionada** (opcional, barata): `scripts/sonda-politica.sh` = el `probe-mod` de hoy, que imprime qué eventos corren y cuáles salen `bypassed` en el debug log. Cuando cambie la versión de `claude` en la máquina de trabajo, se corre antes de culpar al código.

**Eval** (S4): brazo `ar-submit` en `evals/techo-reglas-2` con `EXO_RULES_FORZAR_SUBMIT=1`, mismas 10 tareas, k=2, pre-registro de una línea: «≥4/10 cumple → el canal degradado se declara útil; <4 → se declara placebo y el aviso de Stop pasa a ⚠».

---

## 5. Preguntas abiertas para Paul

1. **Turno 1 en cuenta personal con store vacío**: ¿duplicar una vez (submit + compose) o arriesgar el hueco de un turno al cambiar de cuenta? Recomiendo duplicar: es una vez por máquina y el framing es idéntico; el modelo ve la misma regla dos veces, no dos reglas.
2. **Visibilidad del aviso «canal degradado»**: una vez por sesión en la máquina de trabajo es una línea en cada Stop inicial, todos los días. ¿Una vez por sesión, una vez por día (`$.store` con fecha), o solo log? Recomiendo una vez por día hasta que el eval de S4 diga si el canal vale; después, solo log.
3. **¿Correr el brazo `ar-submit` antes o después de implementar?** Antes cuesta ~2 USD y una tarde con el seam `FORZAR`; decide si el aviso de Stop es «ℹ» o «⚠».
4. **`~/.claude.json` → `organizationType`**: ¿lo usamos solo como pista en el log (recomendado) o también como gate del turno 1? Como gate ahorraría la duplicación, a cambio de depender de una caché no documentada.
5. **Hueco S6** (auto-compact a mitad de turno largo en Team): ¿se declara y se deja para v2 con `session.compact` + `$.session.append`, o se quiere ya? Recomiendo declarar.

---

## 6. Lo que es inferencia y lo que no

- **Medido**: tabla de §1 completa; `organizationType` en `~/.claude.json`; estado de `exo-rules/` y `reflex-log` en esta máquina; que `prompt.submit` corre en `-p`.
- **Leído del API de esta build** (`types/claude-code.d.ts` 2.1.292): semántica de `scope`, `PromptSubmitResult.context`, `session.start` y `/clear`, `$.store`, bloques del primer mensaje, `SessionStartHookInput.source`.
- **Inferencia**: que el `context` de `prompt.submit` tenga el mismo grado de autoridad que el v1 (por eso S4 pide medirlo); que las filas `isMeta` y los bloques de hook se resuman en `/compact`; que `session.compact` corra bajo la política (no está en su lista, pero no se disparó en la sonda).
- **No verificado y debería verificarse en el plan**: `--resume` con el fallback (prompt.submit debería disparar igual; sin medir); que un `prompt.submit` que tarda 3 s (timeout de `exo rules`) no retrase visiblemente el eco del prompt en la TUI — en compose ese coste ya se pagaba por turno, pero compose no está en el camino crítico del eco del prompt y submit sí.
