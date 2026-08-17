# Cierre de exo — M2-08 → M5b: plan de campañas

> **Para ejecutores agénticos:** este plan se ejecuta con `paul-profile:fabrica`
> (una campaña por sección). Cada campaña genera sus briefs de item al abrirse;
> este documento fija QUÉ entra en cada una, en qué orden y cuándo se considera
> cerrada. Los steps bite-sized viven en el brief del item, no aquí.

**Goal:** llevar exo de "engine que busca" a "framework que sustituye a
basic-memory y a superpowers en el uso diario de Paul", terminando con la
desinstalación física de basic-memory (M5b).

**Arquitectura:** las tres capas ya firmadas — thin (skills + hooks) → engine
(Rust, `engine/`) → thick (KB markdown, no se migra). El camino es
estrangulamiento: cada campaña deja el sistema funcionando y con rollback de un
flag.

**Spec madre:** `docs/superpowers/specs/2026-07-16-framework-unificado-design.md`
(roadmap §7, estrangulamiento §4.4, cutover §5.3, guardrails §5.4).
**Spec de E1:** `docs/superpowers/specs/2026-07-17-m2-e1-read-design.md`.

---

## 0. Régimen de esta fase (cambia respecto a M0-M2)

Decisión de Paul, 2026-08-17: **proyecto personal, cerrar ya**. Lo que se retira
y lo que se mantiene, explícito para que ningún consultor lo reabra:

**Se RETIRA:**
- Pre-registro de métricas nuevas, ventanas de observación y gates numéricos
  bloqueantes. Ninguna campaña de aquí en adelante espera a medir nada.
- El gate M2-09 de tres patas deja de ser **bloqueante**: se corre porque el
  harness ya existe y cuesta dos comandos, y su salida es **informativa**. Un
  número peor no para la campaña; se anota y se afina después (o no).
- `GATE-CALENDARIO-D`: **cerrado, no derogado**. Su condición literal era
  "fecha ≥ 2026-07-23 **Y** métrica D medida y cerrada". Ambas se cumplen:
  D corrida el 2026-08-02, verdict NO-PASS firmado
  (`agent-develop/docs/superpowers/evals/2026-08-02-reflex-v2-verdict.md`).
  M1b/M3/M6 quedan desbloqueados sin necesidad de override.
- El NO-PASS de D y M **no bloquea nada de exo**. Diagnóstico ya escrito: era
  cobertura de transporte, no eficacia del engine. El fix vive en reflex (A1,
  ya desplegado).
- M7 (templates para terceros) en su forma original —"KB esqueleto +
  profile.md comentado"—: **fuera del plan**. Sin consumidor, YAGNI.
  **Reabierto el 2026-08-17 con otra forma**: ver campaña C11 (bootstrap de
  instancia). Lo que se descarta es el esqueleto vacío, no el problema de
  arranque en frío.

**Se MANTIENE (líneas rojas de verdad):**
- **Veto AGPL**: jamás código ni vendorizado de basic-memory. Diseño sí.
- **Permalinks del frontmatter se honran y jamás se regeneran.** Romperlo
  corrompe el 83% del tráfico y todos los enlaces de la KB.
- **M5b (desinstalar basic-memory) gated por M6 completo y probado.** No es
  ceremonia: sin M6, mueren en silencio el recall, los matchers
  `mcp__basic-memory__*` de los hooks y kbx/consolida.
- **Gates de merge delegados a consultor fable** (Paul no los hace). Régimen
  intacto: consultor fresco, verificación primaria propia, mandato de disenso,
  verdict escrito al repo.
- **Acciones destructivas o externas siguen siendo de Paul**: desinstalar
  basic-memory, borrar la KB, tocar el marketplace público.
- La KB (`kb-demo`) es un repo git: todo write-path tiene rollback por git.

## Global Constraints

- Binario `exo`, crate único en `engine/`, edition 2024, MSRV ≥ 1.97.
- `sqlite-vec` pineado exacto `=0.1.9`; nunca `^`.
- Envelope v1 `{schema_version, command, data}` es superficie fijada: se
  extiende con campos nuevos, no se rompe la forma.
- Config: lectura RO de `~/.basic-memory/config.json` **hasta M5a**; en M5a nace
  la config propia de exo y el acoplamiento se corta (obligatorio: sin esto,
  M5b deja el engine sin config).
- Corpus: dotdirs fuera, `archive/` dentro, no-markdown fuera, links rotos
  tolerados, recencia = git.
- Commits en castellano con prefijo convencional; git desde el working dir o
  `git -C`, nunca `cd && git`.

---

## Campaña 5 — M2-08 + M2-09: cerrar E1 read

**Objetivo:** `exo recall --json` existe, es rápido y el side-by-side queda
corrido. Cierra M2.

| Item | Qué | Lane | Oráculo |
|---|---|---|---|
| M2-09a | Desempate determinista por permalink en `fusiona` y `busca_vector` | mecánica | test de orden estable con scores empatados |
| M2-08 | `exo recall --json`: envelope v1, perfiles por tipo de consumidor, cap de bytes, salida de texto por stdout | mecánica | golden envelope en `tests/` + `exo recall` real sobre la KB |
| M2-08b | Latencia con hyperfine: FTS-only y hybrid en frío | mecánica | números anotados en el reporte, **informativos** |
| M2-09 | Corrida pareada engine vs bm el mismo día + paridad de corpus | corrida | `replay-engine.py` + `analyze.py` + `corpus-parity.py`, ya existen |

**Detalle de M2-08 — el único item con decisión de diseño real:** el contrato
de `data` se diseña contra su consumidor primario, `compose-inject.sh` de
reflex (`agent-develop/plugins/reflex/scripts/compose-inject.sh`), que ya está
desplegado y define hoy: perfiles por `agent_type`, cap de 2KB, bloque de texto
por stdout, exit-code gating. `exo recall` debe poder sustituirlo sin que el
hook cambie de forma. Leer ese script antes de escribir la spec del item.

**Cierre de campaña:** `exo recall` funciona contra la KB real y los tres
scripts del harness corren sin error. Los números se anotan; no se negocian ni
bloquean.

---

## Campaña 6 — M6: el cutover que hace que exo se use

Esta es la campaña que convierte el engine en el sistema vivo. Va **antes** que
M4/M5a a propósito: el 83% del tráfico real es lectura, y el recall es lo que
paga el arranque de cada sesión.

| Item | Qué | Riesgo cosido |
|---|---|---|
| M6-01 | Índice fresco sin daemon: `exo index` incremental invocado desde el hook (o desde `exo recall` si el coste lo permite) | Sin esto el recall sirve un índice rancio; basic-memory tenía watch, exo no |
| M6-02 | Cutover del hook de recall: `basic-memory-recall.sh` → `exo recall`, con el FALLBACK embebido reescrito | El fallback stale actual cita basic-memory; hay que reescribirlo o queda un fallback mentiroso |
| M6-03 | Migrar el plugin `reflex` (guardrails) al monorepo exo, con los scripts consultando el engine en vez de calcular | Matchers `mcp__basic-memory__*` dejan de matchear cuando el MCP muera (M5a): se cambian aquí, no después |
| M6-04 | Repunte de kbx al índice del engine + actualizar la lista `consumed` del schema-canary **en el mismo commit** | `/consolida` falla-fuerte ante schema_drift por diseño; sin esto el primer consolida post-cutover muere |
| M6-05 | Cutover de doctrina: `core-index`, `doctrina-agentes`, `~/.claude/CLAUDE.md` y la description de `recon-first` nombran basic-memory/superpowers por nombre | Si no, queda doctrina apuntando a un MCP muerto |

**Cierre:** una sesión real arranca con `exo recall`, `kbx doctor` corre verde
contra el índice del engine y `/consolida` no explota. Rollback = un flag en
`hooks.json`.

---

## Campaña 7 — M4: write-path (E2)

**Objetivo:** el engine escribe la KB. `/documenta` y `/consolida` dejan de
necesitar el MCP para escribir.

| Item | Qué |
|---|---|
| M4-01 | `exo write` v1: **new + append + replace_section**. Sin move (un move sin reescribir links corrompe el grafo — se queda fuera de v1, decisión firmada) |
| M4-02 | search-before-write nativo: busca duplicados y fuerza decisión merge/append/new antes de crear |
| M4-03 | Validación OKF que **auto-completa y nunca rechaza** (`type` obligatorio se rellena, no se falla): `/documenta` no puede reventar al cierre de sesión |
| M4-04 | Veto RO por defecto: escribir exige comando explícito |
| M4-05 | Canary de kbx en el mismo commit |
| M4-06 | Reapuntar `/documenta` y `/consolida` a `exo write` |

**Modo file-first (spec §4.4-E2):** el engine escribe el markdown directamente;
mientras basic-memory siga vivo, su watch lo absorbe solo. Sin traductor
engine→MCP: se tiraría en M5a.

**Cierre:** un `/documenta` real de una sesión escribe vía exo, la KB queda bien
y `git diff` en kb-demo es limpio.

---

## Campaña 8 — M3 + M1b: cutover de skills y marketplace

Va después de M4 porque hasta aquí las skills de proceso son ortogonales al
engine, y porque las 7 skills de `process` ya están escritas con paridad
135/135 verificada (`evals/prep-m3/`): es cutover, no desarrollo.

| Item | Qué |
|---|---|
| M3-01 | Mismo día: `superpowers` disabled + `process` enabled |
| M3-02 | **`process:orchestrate` conserva el dispatch `subagent_type: reflex:executor` sin `model`** (paridad paul-profile 0.3.0). Si se pierde, reflex v2 se desenchufa sin síntoma |
| M3-03 | Actualizar `fabrica`: hoy referencia `superpowers:subagent-driven-development` y `paul-profile:orchestrate-personal`, ambas mueren en el cutover |
| M3-04 | Atribución MIT de superpowers en el repo, día uno |
| M3-05 | `using-superpowers` desaparece; su sustituto es una línea de routing en `core-index` |
| M1b-01 | Marketplace: **rename** de agent-develop (preferido por los redirects de GitHub, que mantienen el fetch y la identidad) vs repo nuevo. Decisión de Paul, es acción externa |
| M1b-02 | Decidir si `workflow-lint` entra en el marketplace nuevo |

**Rollback:** superpowers queda instalado-pero-apagado hasta que un ciclo de
trabajo real cierre sin carencias. Sin contador de no-disparos (retirado por
el régimen §0).

---

## Campaña 9 — M5a: MCP propio

| Item | Qué |
|---|---|
| M5a-01 | Servidor MCP stdio con `rmcp`: solo el hot-path medido — `read_note`, `search_notes`, `recent_activity`. **Sin `build_context`** (1 uso en 11 días) |
| M5a-02 | **Config propia de exo**, corte del acoplamiento RO a `~/.basic-memory/config.json`. Bloquea M5b: sin esto, desinstalar basic-memory deja el engine sin config |
| M5a-03 | basic-memory **apagado pero instalado**: periodo sin divergencias, con la KB escribiéndose solo por exo |

**Cierre:** un ciclo real de trabajo (sesiones normales, un `/documenta`, un
`/consolida`) con basic-memory apagado y nada roto.

---

## Campaña 10 — M5b: desinstalación (acción de Paul)

**Precondición dura:** M6 completo y probado. Es el único gate que sobrevive al
régimen §0.

Checklist antes de desinstalar:
1. El hook de recall no llama a basic-memory por ninguna vía (ni en el FALLBACK).
2. Ningún hook conserva matchers `mcp__basic-memory__*` vivos.
3. kbx apunta al índice del engine y `consumed` está actualizado.
4. `/documenta` y `/consolida` corren end-to-end sin el MCP.
5. exo tiene config propia (M5a-02).
6. La KB está commiteada y pusheada en kb-demo.

Entonces: desinstalar basic-memory. **Lo ejecuta Paul** — acción destructiva,
línea roja no delegable.

---

---

## Campaña 11 — Bootstrap de instancia (arranque en frío del USUARIO)

Abierta por Paul el 2026-08-17: *"este producto de base, el que se lo instale
no tendrá nada que indexar"*.

**El problema**: exo es un motor de retrieval sobre una KB. Con la KB vacía no
recupera nada, así que el día 1 no vale para nada y el usuario no llega nunca
al día 30 en el que tendría notas. La instancia de Paul lleva 138 notas
acumuladas a mano durante meses; ese privilegio no lo tiene quien lo instale.

**La idea**: sembrar la KB desde lo que el usuario YA tiene en su máquina por
usar Claude Code o Codex. Dos capas, deliberadamente separadas por coste y por
riesgo:

| | Fuente | Quién lo hace | Coste | Riesgo |
|---|---|---|---|---|
| **Semilla determinista** | `~/.claude/CLAUDE.md` (o `~/.codex/AGENTS.md`), `settings.json`, comandos y plugins instalados, inventario de `~/.claude/projects/` (qué repos toca y cuánto) | el engine, sin LLM | ms | bajo: son ficheros que el usuario escribió a propósito |
| **Destilado de transcripts** | los `.jsonl` de sesiones | una skill de la capa thin, con LLM, escribiendo vía `exo write` (M4) | tokens del usuario | **alto**: los transcripts llevan rutas, secretos pegados, conversación privada |

**Decisiones de diseño propuestas** (a firmar cuando se abra la campaña):

1. La semilla determinista es automática en la instalación; el destilado de
   transcripts es **opt-in explícito y con revisión del usuario antes de
   escribir** — jamás un import silencioso de su historial.
2. El engine no llama a ningún LLM. Sigue siendo un binario determinista; el
   destilado vive en la capa thin, que es donde ya hay un modelo.
3. Todo queda local. Ninguna parte del bootstrap manda nada fuera.
4. La semilla debe producir notas markdown normales, con `permalink` y
   frontmatter — no un formato especial: lo que entra en la KB es KB.

**Dependencia**: el destilado necesita `exo write` (C7/M4). La semilla
determinista no depende de nada y podría ir ya.

**Decidido por Paul (2026-08-17)**: *"de momento es solo mío, pero en el
futuro intentaré que pueda usarlo más gente"*. Traducción operativa:

- **Se construye**: la semilla determinista. Le sirve a Paul hoy (cambiar de
  máquina, reconstruir su instancia) y es la mitad barata del bootstrap
  futuro. Entra como item cuando C7 (M4 write) esté hecho, o antes si se hace
  escribiendo ficheros directamente.
- **No se construye ahora**: el destilado de transcripts con LLM. Queda
  especificado aquí y se retoma si algún día hay usuarios.
- **Requisito transversal desde YA** (barato ahora, caro después): no cerrar
  la puerta a terceros con hardcodes de la instancia personal. Hoy hay uno
  real y localizado: `engine/src/lib.rs:59` resuelve la KB con
  `projects["kb-demo"]` literal, así que en la máquina de otro el engine
  no encuentra su KB. **Se arregla en C9/M5a**, que es donde nace la config
  propia de exo: el nombre del proyecto pasa a ser config, con la lectura RO
  de basic-memory como fallback mientras dure el side-by-side. Los scripts de
  reflex tienen el mismo patrón (`basic-memory-recall.sh`, `a1-freeze-watch.sh`
  con una ruta absoluta a `/home/paul/...`), y se barren en C6/M6-02 al
  reapuntarlos.

---

## Deuda suelta a barrer por el camino (sin campaña propia)

Se folda en la primera campaña que toque el fichero:

- README de exo dice "pre-M1a" (drift documental) → C5.
- `crontab -r` pendiente, residuo de M1a → C6 (toca entorno).
- `reflex-baseline.sh` traga errores de jq (`2>/dev/null`) → C6 o se tira
  entero con la infra de medición.
- Cachés huérfanas de reflex 0.6.0/0.8.0 → C8.
- Decisión abierta "`archive/` en el ranking": se decide con la corrida de C5
  delante, o se deja como está (indexado) y se cierra la decisión.

## Orden y por qué

```
C5 (M2-08/09) ──→ C6 (M6 cutover recall) ──→ C7 (M4 write) ──→ C8 (M3+M1b) ──→ C9 (M5a MCP) ──→ C10 (M5b)
     cierra E1        exo se USA               exo ESCRIBE        skills propias      MCP propio      adiós bm
```

Difiere del grafo de la spec §7 (que ponía M4 antes de M6) en un punto
deliberado: **M6 se adelanta**. Razón: hasta M6 el engine es un binario que
nadie invoca, y cada semana que pasa sin usarlo es una semana sin descubrir sus
fallos reales. Con M6 hecho, M4 y M5a se validan contra uso real en vez de
contra tests.
