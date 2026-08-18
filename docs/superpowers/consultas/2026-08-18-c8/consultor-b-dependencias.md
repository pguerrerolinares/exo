# Verdict — Consultor B: dependencias del cutover M3-01 (apagar `superpowers`)

- **Fecha**: 2026-08-18 · **Consultor**: fable delegado, fresco (sin participación previa en C8)
- **Alcance**: barrido de dependencias vivas a `superpowers` (B1), semántica de skill ausente (B2), adjudicación M3-03/fabrica (B3), verificación M3-02 (B4), checklist pre-apagado (B5).
- **Fuera de alcance** (consultor A/C en paralelo): mecánica de instalación del plugin `process`, sustituto de `using-superpowers` (M3-05), marketplace (M1b). Consigno hechos que les tocan, sin adjudicarlos.
- **Entorno verificado**: exo `main` @ `89dbe95` (limpio, worktree `c8-m3` @ mismo commit, checkout limpio sin cambios — es espejo, no dependencia). `~/.claude/settings.json:22` = `"superpowers@claude-plugins-official": true`. Caché instalada: superpowers 6.3.0, paul-profile 0.5.0, reflex 0.13.0, workflow-lint 0.1.0 (`installed_plugins.json`). Marketplace `agent-develop` con `autoUpdate: true` desde GitHub.

---

## Distinción previa que ordena todo el inventario

**`superpowers:<skill>` (invocación de plugin) ≠ `.superpowers/` (convención de directorio) ≠ `docs/superpowers/` (convención de rutas de docs).** Las dos últimas son nombres de filesystem que NO dependen del plugin y NO cambian con el cutover: `.gitignore` de exo (líneas 5-8), `engine/src/walker.rs:5` (`DOTDIRS_EXCLUIDOS`), el flag `ACTIVE` de fabrica, `kb-budget-check.sh`, el guard PreToolUse. La mayoría aplastante de los hits crudos del grep son de estas dos clases. **Nadie debe "limpiarlas" con motivo del cutover** — romperían fabrica, el walker y el guard sin relación alguna con el plugin.

---

## B1. Inventario de referencias vivas

Método: `grep -rn "superpowers"` sobre `~/.claude/` completo (settings, CLAUDE.md, commands/, plugins/{cache,marketplaces,data}, sin transcripts), `agent-develop/`, `exo/`, `kb-demo/`, y todos los `.claude/`, `CLAUDE.md` y `AGENTS.md` de `~/Documentos/proyectos/*/`; más `crontab -l` y los `hooks.json` de los 3 plugins propios vivos.

### Clase (i) — INVOCACIÓN VIVA: 4 hits fuente (3 ejecutables + 1 condicional)

Cada uno existe por triplicado (repo agent-develop = fuente · `~/.claude/plugins/cache/...` = lo que ejecuta HOY · `~/.claude/plugins/marketplaces/agent-develop/...` = espejo de instalación). El fix es siempre en el repo; caché y espejo se actualizan solos con el push (autoUpdate).

| # | Fichero:línea (fuente) | Texto literal | Qué muere |
|---|---|---|---|
| 1 | `agent-develop/plugins/paul-profile/skills/fabrica/SKILL.md:8-9` | "el motor por pieza es `paul-profile:orchestrate-personal` (sobre superpowers:subagent-driven-development)" | El motor por pieza de TODA sesión-fábrica apunta a dos skills que desaparecen. Réplicas: cache `paul-profile/0.5.0/skills/fabrica/SKILL.md:9`, mirror marketplaces. |
| 2 | `agent-develop/plugins/paul-profile/skills/orchestrate-personal/SKILL.md:9` | "`superpowers:subagent-driven-development` (dispatch a fresh subagent per task, ...)" | La skill entera queda sin motor base — es hoy "Paul's personal default". Réplica cache 0.5.0:9. |
| 3 | `agent-develop/plugins/reflex/skills/recon-first/SKILL.md:54` | "Para un flujo de depuración riguroso, `superpowers:systematic-debugging` es un buen [siguiente paso]" | Puntero muerto que un agente en recon-first intentará seguir. Réplica cache reflex/0.13.0:54. `evals/prep-m3/gold/debug.md:42` ya anticipó este cruce ("ambas fuentes son ahora la misma skill"). |
| 4 | `kb-demo/research/wisdom-ai-news POC — plan de implementación (Plan 1- backend).md:19` | "REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans" | **Condicional**: solo muerde si ese plan se re-ejecuta (proyecto ya entregado). Degradación, no bloqueo. |

### Clase (iii) — matcher/config: 4 hits

| Fichero:línea | Texto | Veredicto |
|---|---|---|
| `~/.claude/settings.json:22` | `"superpowers@claude-plugins-official": true` | El switch mismo del cutover (M3-01, terreno del consultor A). |
| crontab línea 1 (`7 9 * * *`) → `agent-develop/plugins/reflex/scripts/a1-freeze-watch.sh:120` | `ORCH_PATH="$CACHE_DIR/paul-profile/$obs_pp_v/skills/orchestrate-personal/SKILL.md"` | Cron diario VIVO que hashea `orchestrate-personal` de la caché. Cuando la 0.6.0 sin esa skill aterrice, appendea "FREEZE ROTO" a su log cada día (verificado en el código: fichero ausente ⇒ `obs_orch=AUSENTE` ⇒ `add_break`, líneas 120-127). No es silencioso pero es ruido espurio que nadie lee: la ventana A1 cerró 2026-08-02. El plan ya lo adjudicó: "crontab -r pendiente, residuo → C6" (plan §Deuda) — **sigue sin ejecutarse a fecha de hoy** (evidencia: `crontab -l` lo lista). |
| `equipo-x-broker/.claude/settings.local.json:35` | allowlist de `.../superpowers/5.0.2/skills/brainstorming/scripts/start-server.sh` | **Inerte ya hoy**: en caché solo existen 6.1.1/6.2.0/6.3.0 (verificado con `ls`). Sin efecto con o sin cutover. |
| `a11y-crawler-v2/.claude/settings.local.json:125` | ídem con `5.0.5` | Ídem, inerte. |

Cero matchers de hooks dependientes de superpowers: los `hooks.json` vivos (reflex 0.13.0, paul-profile 0.5.0, workflow-lint 0.1.0) matchean `Bash`, `WebSearch|WebFetch` y `mcp__basic-memory__*` — ninguno nombra skills ni al plugin.

### Clase (iv) — atribución legal (M3-04, NO TOCAR): 13 hits en 11 ficheros de `exo/plugins/process/`

`README.md:5,21-27` (tabla de absorción + cita MIT), `LICENSES/superpowers.LICENSE` (copia literal), y headers "Derived from superpowers 6.1.1 (MIT © 2025 Jesse Vincent)" en: `orchestrate/scripts/{review-package:2, task-brief:2, sdd-workspace:2}`, `orchestrate/implementer-prompt.md:4`, `orchestrate/reviewer-prompt.md:5` (+ `:7` cita "orchestrate-personal (paul-profile 0.5.0, propio)" — atribución de fuente propia, se queda), `plan/plan-template.md:5`, `debug/techniques.md:9`, `tdd/anti-patterns.md:5`. El plan exige esta atribución "día uno" — cualquier limpieza que las toque es regresión de M3-04.

### Clase (ii) — documental/histórico: el resto (no ejecuta, no se toca en C8)

Conteos medidos por área (líneas con "superpowers", excluido el espejo `.worktrees/c8-m3` = 833 líneas duplicadas del propio repo):

| Área | Líneas | Naturaleza |
|---|---|---|
| `exo/docs/superpowers/` | 106 | specs/planes/consultas — describen el cutover, no lo ejecutan |
| `exo/evals/` (retrieval-fase0 + prep-m3 + e1-read) | ~670 | golds y verdicts de paridad — citan superpowers como REFERENCIA de lo absorbido; son el audit trail de M3, intocables |
| `exo/.superpowers/fabrica/` (ledger 8, reports, config) | 137 | operativo-histórico de campañas |
| `exo/` raíz (README:7-9, foto-as-is:19,105,111, engine walker) | ~15 | rutas de docs + tabla as-is (la foto es de 2026-08-02, snapshot fechado — no se reescribe) |
| `agent-develop/` docs+README+COMPANY-INTEGRATION-SPEC+marketplace.json | ~30 | **COMPANY-INTEGRATION-SPEC.md (7 hits) se queda tal cual**: describe empresas donde superpowers ES el motor real — contexto ajeno al entorno de Paul. `README.md:12,29` y `marketplace.json:29` (descripción "Layers on superpowers...") quedan stale: cosmético, foldear en el mismo PR de B3 si se quiere, no bloqueante |
| `kb-demo/` (86 líneas totales) | 12 vivas + 74 archive/bitácora | Vivas: `projects/agent-develop.md:86,128` (doctrina de layering, quedará desactualizada), `projects/exo:16,20`, `projects/pguerrero-music:46`, `projects/kbx:53`, `log/*-bitacora` (histórico append-only). `core/core-index.md` y `core/doctrina-agentes.md`: **cero hits** (los 2 matches del grep eran "fabrica"/"fabricación", falsos positivos verificados) |
| `pguerrero-me/CLAUDE.md:5,35,52-54` | 5 | solo rutas `docs/superpowers/` (convención) |
| `~/.claude/` restos (settings backups ×2, transcripts, history, reflex-log, plugin-catalog-cache) | n/a | histórico/cache del harness, fuera del camino de ejecución |
| cachés huérfanas: reflex 0.6.0/0.8.0/0.11.0/0.12.0, paul-profile 0.2.1, superpowers 6.1.1/6.2.0 | n/a | deuda C8 ya listada en el plan ("Cachés huérfanas → C8"); las de superpowers 6.x se CONSERVAN durante la ventana de rollback |

**Hechos para los otros consultores (sin adjudicar):** (a) `exo/plugins/process/` NO tiene `.claude-plugin/plugin.json` ni marketplace.json — hoy no es instalable tal cual (consultor A/C); (b) `~/.claude/CLAUDE.md` y `~/.claude/commands/documenta.md`: cero referencias a superpowers — el cutover no los toca.

---

## B2. Qué pasa de verdad cuando la skill no existe

**Método (empírico, esta sesión):** invoqué el tool Skill con dos nombres no listados: `Skill(skill: "superpowers:skill-inexistente-sonda-b2")` y `Skill(skill: "process:orchestrate")` — el segundo es exactamente el mundo actual visto desde después del cutover (plugin no instalado). Resultado idéntico en ambos:

```
<tool_use_error>Unknown skill: superpowers:skill-inexistente-sonda-b2</tool_use_error>
<tool_use_error>Unknown skill: process:orchestrate</tool_use_error>
```

**Conclusiones:**
1. **La invocación explícita NO es silenciosa**: error visible e inmediato, sin carga parcial ni contenido alucinado por el tool. (Límite del método: sondeo con plugin *ausente/skill inexistente*; el caso *instalado-pero-disabled* no era sondeable — ningún plugin disabled del entorno tiene skills — pero en ambos casos la skill no aparece en el listing del system prompt, que es la superficie que cuenta.)
2. **El modo de fallo silencioso real es la referencia en prosa**: una skill/nota que dice "el motor es superpowers:X" no dispara invocación alguna — el agente simplemente procede sin la metodología (degradación sin síntoma), o improvisa tras ver el error. Los 4 hits de clase (i) son exactamente esto.
3. **Dimensionamiento del guardarraíl**: no hace falta guardarraíl de runtime (el harness ya avisa); hace falta higiene de referencias — B5 se limita a eso.

---

## B3. M3-03 — fabrica (ADJUDICADO)

### (i) Reescrituras exactas — `agent-develop/plugins/paul-profile/skills/fabrica/SKILL.md`

Único punto del fichero que cita skills que mueren son las líneas 8-9 (verificado leyendo el SKILL.md completo: el resto son rutas `.superpowers/…` —convención, se quedan— y `reflex:executor` —sigue vivo—).

```diff
-**No reinventes orquestación**: el motor por pieza es `paul-profile:orchestrate-personal`
-(sobre superpowers:subagent-driven-development). Este skill añade el protocolo de
+**No reinventes orquestación**: el motor por pieza es `process:orchestrate`
+(fusión destilada de subagent-driven-development + orchestrate-personal). Este skill añade el protocolo de
```

En el mismo bump, `plugins/paul-profile/.claude-plugin/plugin.json`: versión → `0.6.0` y descripción (línea 3) reescrita sin `orchestrate-personal` y con el layering nuevo; texto nuevo literal:

```
"description": "Paul Guerrero's personal campaign harness — fabrica (campaign harness A-thin: sesiones-fábrica largas con gate de merge asíncrono, guard de main incluido). Layers on top of process:orchestrate (plugin process de exo); does not reinvent orchestration.",
```

### (ii) `paul-profile:orchestrate-personal` — SE RETIRA (borrar el directorio de la skill en 0.6.0). Ni se mantiene ni alias.

Razón con cita: la spec madre ya lo firmó — *"Se absorben al monorepo: … paul-profile (menos fabrica)"* (framework-unificado-design, línea 14) y la tabla §5.2: *"orchestrate | subagent-driven-development, executing-plans, dispatching-parallel-agents | orchestrate-personal (cost pyramid, memory packet, blindspot pass)…"* (línea 98). El gold lo remachó: *"el layering sobre superpowers muere; process:orchestrate ES la fusión"* (`evals/prep-m3/gold/orchestrate.md:76`). Un alias re-crearía el layering que la fusión eliminó y dejaría dos rutas divergentes al mismo motor. Riesgo residual de retirarla: el hábito de invocarla da `Unknown skill` **visible** (B2) — aceptable y autocorrectivo. Consumidores vivos verificados: solo fabrica (fix en (i)) y el cron a1-freeze-watch (residuo, B5-7). `process/README.md:31,34` y `reviewer-prompt.md:7` la citan como FUENTE absorbida — atribución, se quedan.

### (iii) fabrica se queda en `paul-profile` — NO migra a `process`.

Razón con cita: *"Quedan fuera: … fabrica (plugin de instancia: workflow personal + hook propio)"* (framework spec, línea 15) y *"Framework sin nada personal. Instancia de Paul = kb-demo (repo aparte) + profile.md plano + plugin fabrica"* (línea 43). `process` es la capa exportable del framework; fabrica es instancia (protocolo personal + guard PreToolUse propio). Migrarla contradiría la spec firmada sin ganancia.

### Secuencia (el riesgo "esta sesión corre bajo fabrica")

La sesión actual ejecuta fabrica desde la caché **inmutable** `paul-profile/0.5.0` — editar el repo agent-develop no puede romperla. La 0.6.0 solo llega al entorno vivo con **push a GitHub** (marketplace `autoUpdate: true`; evidencia de propagación real: reflex 0.13.0 auto-actualizado 2026-08-17 en `installed_plugins.json`), y el push es línea roja de Paul (config §Ejecución de gates). Orden adjudicado, mismo día (plan C8: "Mismo día"): **flip M3-01 primero, push de agent-develop inmediatamente después** — así no existe ninguna ventana en que fabrica 0.6.0 apunte a un `process:orchestrate` aún apagado. El "se nota tarde" se acota con el smoke B5-6/B5-8.

---

## B4. M3-02 — VERIFICADO: SE CUMPLE HOY. Sin diff.

- `exo/plugins/process/skills/orchestrate/SKILL.md:12-16`: *"## PARIDAD CRÍTICA — no negociable / `subagent_type: reflex:executor`, **nunca** `general-purpose`, **sin** `model` (el rol lo trae fijo — pasarlo lo pisaría). Si se pierde, reflex v2 se desenchufa sin síntoma."*
- `orchestrate/implementer-prompt.md:7-8`: *"**Dispatch:** `subagent_type: reflex:executor` — **sin** `model` (paridad crítica: el rol trae modelo fijo, pasar `model` lo pisaría)."* y `:11` (bloque de dispatch). Los `scripts/` no despachan (son empaquetadores de brief/review); `reviewer-prompt.md:14` usa `model` explícito **para el reviewer**, que es rol genérico — correcto por diseño.

**Mecanismo (verificado en el código de reflex, no asumido):** reflex v2 transporta la doctrina en la **definición del agente** `reflex/0.13.0/agents/executor.md` (system prompt de disciplina + `model: sonnet` en frontmatter) y en el hook SubagentStart (`subagent-inject.sh` → `compose-inject.sh`) que elige perfil por `agent_type` (`inject-profiles.json`: `"reflex:executor": "reducido"`, `"general-purpose": "ejecucion"`). Despachar `general-purpose` pierde el system prompt del rol entero (el perfil "ejecucion" del inyector solo compensa un extracto capado en bytes); pasar `model` pisa el `model: sonnet` del frontmatter (precedencia del parámetro del tool Agent sobre el frontmatter, documentada en el propio schema del harness) y rompe la pirámide de coste. En ambos casos el dispatch es válido y no produce error — ese es el "sin síntoma".

**Probe post-cutover viable ya** (lo exige la spec §5.3, línea 112): `~/.claude/reflex-log.jsonl` registra `inject-emitted … type=reflex:executor perfil=reducido` (7 entradas hoy, 114 inject-emitted totales). La nota del plan "paridad paul-profile **0.3.0**" es la versión donde nació la regla; la vigente 0.5.0 la conserva (`orchestrate-personal/SKILL.md:27,51` en caché) — imprecisión sin consecuencia.

---

## B5. Checklist ejecutable antes de apagar (cada ítem atado a su evidencia)

1. **[repo agent-develop, rama]** Reescribir `fabrica/SKILL.md:8-9` con el diff de B3(i) → hit B1-i-1.
2. **[misma rama]** Borrar `plugins/paul-profile/skills/orchestrate-personal/` y actualizar `plugin.json` (descripción B3(i), versión 0.6.0) → hit B1-i-2. Opcional no bloqueante: `README.md:12,29` y `marketplace.json:29` de agent-develop (descripciones stale, clase ii).
3. **[misma rama]** `reflex/skills/recon-first/SKILL.md:54`: `superpowers:systematic-debugging` → `process:debug`; bump reflex 0.13.1 → hit B1-i-3 (respaldado por `gold/debug.md:42`).
4. **[gate]** Rama de agent-develop gateada por consultor (régimen del config) — es cambio de skills, superficie del régimen.
5. **[Paul — M3-01]** Flip en `~/.claude/settings.json`: `superpowers@claude-plugins-official: false` + alta de `process` (mecánica exacta: consultor A) → hit B1-iii-1.
6. **[Paul — mismo día, inmediatamente después del flip]** Push de agent-develop (0.6.0 + 0.13.1); autoUpdate propaga a caché/espejo. Orden flip→push adjudicado en B3 (Secuencia).
7. **[Paul o sesión autorizada — deuda C6 aún pendiente]** `crontab -r` (única línea: a1-freeze-watch). Sin esto, la 0.6.0 dispara "FREEZE ROTO" espurio diario → hit B1-iii-2. Ya adjudicado por el plan (§Deuda), aquí solo se le pone fecha: antes o junto al push del paso 6.
8. **[probe M3-02, post-cutover]** Un dispatch real de `process:orchestrate` y verificar `grep inject-emitted ~/.claude/reflex-log.jsonl | tail -1` → `type=reflex:executor perfil=reducido` → evidencia B4. En la MISMA sesión-fábrica nueva, smoke de fabrica 0.6.0: el listing debe mostrar `process:*` y fabrica debe resolver su motor sin `Unknown skill`.
9. **[higiene KB, flujo normal, no bloqueante]** Delta post-cutover en `kb-demo/projects/agent-develop.md` (líneas 86,128) vía search-before-write; opcional en `research/wisdom-ai-news POC:19`. Bitácoras y `archive/` NO se tocan → hits B1-i-4 y clase ii KB.
10. **[NO HACER]** No tocar atribución MIT (clase iv, es M3-04), ni COMPANY-INTEGRATION-SPEC.md, ni ninguna ruta `.superpowers/`/`docs/superpowers/` (convenciones de filesystem, ver distinción previa). No borrar cachés superpowers 6.x durante la ventana de rollback (plan C8 §Rollback: "instalado-pero-apagado"); las cachés huérfanas reflex/paul-profile viejas son la deuda C8 aparte que ya lista el plan.

Sin ítems de relleno: cada línea cuelga de un hit de B1 o de una cita del plan/spec verificada aquí.

---

## Qué busqué para objetar (mandato de disenso)

- **"Algún hook vivo matchea superpowers y muere con el cutover"** — leí los `hooks.json` de reflex 0.13.0, paul-profile 0.5.0 y workflow-lint 0.1.0: matchers de `Bash`, `WebSearch|WebFetch` y `mcp__basic-memory__*` únicamente. REFUTADA.
- **"El fallo de skill ausente es silencioso o alucina"** (premisa implícita del miedo del brief) — sonda empírica doble en B2: error visible en ambos casos. REFUTADA a nivel tool; el silencio real está en la prosa (reformulación, no confirmación cómoda).
- **"El brief/plan dice paridad paul-profile 0.3.0"** — la instalada es 0.5.0; verifiqué que 0.5.0 conserva la regla (cache `orchestrate-personal:27,51`). El plan arrastra un número de versión histórico: imprecisión real del plan, sin consecuencia operativa.
- **"fabrica necesita más reescrituras que 2 líneas"** — lectura completa del SKILL.md + grep de sus templates: el resto son rutas `.superpowers/` y `reflex:executor`. REFUTADA.
- **"Retirar orchestrate-personal rompe un consumidor no obvio"** — grep de consumidores en todo el entorno: solo fabrica (se reescribe) y el cron a1 (residuo con `crontab -r` ya adjudicado). El único hallazgo real del disenso: **el cron a1-freeze-watch sigue vivo hoy pese a que el plan lo daba a C6** — sin B5-7, el cutover genera ruido espurio diario.
- **"Algo en ~/.claude/CLAUDE.md, commands/ o los CLAUDE.md de proyectos invoca superpowers"** — grep completo: cero invocaciones (pguerrero-me solo rutas de docs). REFUTADA.
- **"Los settings.local.json de otros proyectos ejecutan scripts de superpowers"** — 2 entradas de allowlist apuntan a cachés 5.0.x que ya no existen (solo 6.x en disco): inertes hoy, con o sin cutover.
- **"core-index/doctrina-agentes dependen de superpowers"** — grep: los únicos matches eran "fabrica(ción)", falsos positivos. El núcleo de la KB está limpio; M3-05 (línea de routing en core-index) parte de cero, no de una limpieza.
