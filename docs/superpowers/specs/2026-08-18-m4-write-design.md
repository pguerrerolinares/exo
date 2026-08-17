# M4 — write-path de exo (E2): diseño

> **Régimen de esta spec:** las decisiones de diseño las adjudicaron tres
> consultores fable independientes (contrato de escritura, forense de fallos
> reales de la KB, barrido de dependencias de basic-memory). Paul firmó sus
> decisiones por adelantado. Lo que sigue es la síntesis ejecutable, no una
> propuesta a aprobar.

**Goal:** que el engine escriba la KB, para que `/documenta` y `/consolida`
dejen de necesitar el MCP de basic-memory. Cierra la campaña C7 del plan
`2026-08-17-cierre-exo-m2-a-m5b.md`.

**Spec madre:** `2026-07-16-framework-unificado-design.md` §4.2 (write-path),
§4.3 (recortes v1), §4.4-E2 (estrangulamiento), §6.4 (OKF).

---

## 1. La decisión de fondo: qué escribe exo y qué escribe el agente

El consumidor del write-path no es un proceso ciego: es Claude Code, que ya
tiene `Write` y `Edit` como tools nativas. Escribir markdown **no es la
capacidad que falta**. Lo que desaparece cuando muera basic-memory es:

1. **Resolución de identidad** — permalink → ruta del fichero.
2. **Frontmatter correcto** en notas nuevas (permalink, type, tier, title).
3. **search-before-write** con dientes.
4. El **`file_path` de vuelta** que `/documenta` usa para el commit scoped.

De ahí el reparto adjudicado, cuyo criterio es **quién ya tiene el fichero en
contexto**:

| Operación | Dueño | Por qué |
|---|---|---|
| Nota **nueva** | `exo write new` | Slug, permalink y ruta son mecánica con línea roja encima; no se improvisan al cierre de sesión |
| **Append** a bitácora | `exo write append` | Escribe sin leer. `Edit` obligaría a cargar el fichero entero: `log/exo-bitacora.md` pesa 14 KB y el backlog ~33 KB |
| **replace_section** / edición del canon | `Edit` del agente | El flujo ya exige leer la nota ganadora para escribir el delta sin duplicar; el Read ya está pagado y `Edit` opera sobre texto exacto, sin parsear headings |

**`replace_section` sale del CLI v1.** Es la pieza de más riesgo de corrupción
y la de menos valor marginal: el matching por nombre de heading es frágil
contra la KB real (la bitácora de exo tiene dos headings casi idénticos,
"Aprendizajes técnicos de M2 (…)" ×2). Esto **deroga M4-01 del plan** y §4.2 de
la spec madre en ese punto concreto.

## 2. El agujero que hay que tapar además del write

**La ruta no es derivable del permalink.** El permalink
`kb-demo/projects/exo-framework-unificado-de-trabajo-agentico` corresponde
al fichero `projects/exo — framework unificado de trabajo agéntico.md`: el
slug come acentos, espacios y em-dashes, y eso no se invierte.

El índice sí la tiene (`schema.rs:14`, `ruta TEXT NOT NULL UNIQUE`) y
`recall --json` ya la expone (`recall.rs:40`), pero **`exo search --json` no**
(`buscador.rs:13-21`). Sin ese campo, el camino de edición se queda sin forma
de localizar el fichero cuando muera el MCP.

**Decisión:** campo aditivo `ruta` en `Resultado` de `search`. Aditivo no sube
`SCHEMA_VERSION` (`envelope.rs:4-5`).

## 3. Contrato de `exo write`

### 3.1 `exo write new`

```
exo write new --db <db> [--kb <kb>] --dir <projects/|log/|research/|...>
              --titulo <t> --from <fichero|-> [--tier <t>] [--force] [--json]
```

- **El contenido entra por fichero** (`--from`), no por argv. El agente lo
  escribe con su tool `Write` en scratch: estructurado, sin escaping de
  comillas, backticks ni `$`. `--from -` acepta stdin para uso desde scripts.
- El fichero puede traer frontmatter parcial. exo **completa lo que falte y
  nunca rechaza** (M4-03):
  - `permalink` = `<proyecto>/<dir>/<slug(titulo)>`
  - `type: note` si falta, `title` del `--titulo`, `tier` del flag
  - El prefijo de proyecto sale de la config (hoy RO de basic-memory,
    `lib.rs:60-77`; en M5a la propia). **Sin hardcode** — requisito C11.
- **Destino:** `<dir>/<titulo>.md`. Escritura por tmp+rename (atómica).
- **Dup-gate (M4-02):** solape de tokens (Jaccard ≥ 0.6) entre el slug de la
  nota nueva y los permalinks ya indexados. Si hay candidatas y no hay
  `--force` → **exit 3, no escribe**, con `data.dup_candidatas`.

  > **Disenso con lo adjudicado, fundado en evidencia.** El consultor A
  > propuso `busca_hybrid` para este gate. Se implementó así y **se reprodujo
  > un falso positivo inmediato**: el título "Zumaia — pruebas de ámbito (v2)"
  > puntuó 0.36 contra una bitácora sin relación alguna, y bloqueó la
  > escritura. La causa es de fondo: el umbral de retrieval (0.35-0.40) está
  > calibrado para *"tráeme contexto relevante"*, que no es la misma pregunta
  > que *"esto ya existe"*, y no comparten escala.
  >
  > El único duplicado real de la historia de la KB (`ai-news-bitacora`
  > creada existiendo `ai-news-pipeline-bitacora`) es un parecido **de slug**,
  > no semántico. El solape de tokens lo caza con Jaccard 0.75 y deja pasar
  > `exo-bitacora` vs `kbx-bitacora` (0.33) y `backlog-diario` vs
  > `backlog-frentes-abiertos` (0.25). Ventaja adicional: es determinista y no
  > carga el modelo ONNX, así que el gate no le añade segundos al cierre de
  > sesión.
  - `--force` salta el gate de **similitud**. Jamás salta una colisión real de
    ruta o permalink existente: eso es exit 1. **Nunca overwrite** — lo
    correcto ahí es append o edit.

### 3.2 `exo write append`

```
exo write append --db <db> [--kb <kb>] <permalink> --from <fichero|->
                 [--crea] [--json]
```

- Resuelve permalink→ruta contra el índice. **Sin fallback a walk+parse en v1**
  (deuda declarada, no implementada): si el permalink no está indexado, el
  comando falla con un mensaje que apunta a `--crea`. Con el índice refrescado
  en cada arranque de sesión (M6-01), un miss significa casi siempre que la
  nota no existe.
- Garantiza separador `\n\n` y hace **un único `write()` con `O_APPEND`**. No
  lee ni reescribe el resto del fichero.
- `--crea`: si la bitácora no existe, la crea con frontmatter de log.

### 3.3 Salida

Envelope v1 en stdout, humano y avisos a stderr, gating por exit code:

```json
{"schema_version":1,"command":"write","data":{
  "op":"new","permalink":"…","ruta_rel":"…","ruta_abs":"…",
  "creada":true,"frontmatter_completado":["permalink","type"],
  "dup_candidatas":[]}}
```

Exit: `0` escrito · `3` dup-gate (decisión pedida, no fallo) · `1` error.

### 3.4 Lo que exo NO hace

- **No commitea.** Devuelve `ruta_abs`; el agente junta las rutas de sus
  envelopes y de sus `Edit` y hace el `git -C … add <rutas>` de siempre,
  incluido el retry ante `index.lock`.
- **No indexa después de escribir.** Escribir al cierre de sesión no paga la
  carga del modelo ONNX; el `--refresca` del recall de la sesión siguiente lo
  absorbe (ya construido en M6-01, `lib.rs:79-94`).

## 4. Preservación del frontmatter (requisito duro que el plan no tenía)

`parsea_nota` (`nota.rs:23-29`) deserializa laxo a cuatro campos y **descarta
el resto**. Cualquier reescritura que pase por ese struct perdería `tags`,
`kbx_budget_max` y demás claves reales de la KB.

**Regla:** el write-path jamás reconstruye un frontmatter existente. `append`
no lo toca en absoluto (escribe al final del fichero). `new` lo genera desde
cero, que es el único caso donde no hay nada que preservar.

## 5. Concurrencia y atomicidad

- **Dos sesiones, misma bitácora:** dos `write()` con `O_APPEND` → ambas
  entradas aterrizan, orden indeterminado, irrelevante en un log fechado.
  Mejor que hoy: el `edit_note` del MCP era read-modify-write.
- **Dos sesiones, mismo canon vía `Edit`:** mismo riesgo que hoy, ya
  documentado en `/documenta`. Mitigación existente: sesgo a append, commits
  scoped, KB en git. **Sin locks ni CAS nuevos** — en single-user con git
  detrás, es over-engineering.
- **El CLI muere a mitad:** `new` es tmp+rename, jamás deja una nota a medias.
  `append` es un syscall; el peor caso teórico es una entrada truncada, y sería
  visible en el `git diff` limpio que el cierre de C7 ya exige. El write no
  toca la DB: el índice no puede corromperse por esto.

## 6. Alcance de lo forzable (honestidad sobre M4-02)

El dup-gate solo es forzable en `new` (exit 3). En las ediciones queda
**advisory por skill**: si el modelo decide escribir con `Write` directo,
ningún CLI lo impide. Redes existentes: el indexer ya salta-y-avisa notas sin
permalink (`indexer.rs:104-110`), el canary de M4-05, y la reescritura de la
skill (M4-06) como barrera primaria.

La spec madre promete un enforcement que no existe en el camino de edición.
Aquí queda declarado en vez de prometido.

## 7. Qué defensas construir (calibrado contra fallos reales, no temidos)

Forense sobre los 258 commits de la KB (2026-06-13 → 2026-08-17, 153
invocaciones de `/documenta`). La jerarquía de fallos reales **no es** la que
el diseño temía:

| Modo de fallo | Veces | ¿Prevenible por código? |
|---|---|---|
| **Delta-append al canon** (`## Delta AAAA-MM-DD` en tier core/stable) | **52** | Mitad: lint sintáctico |
| Crecimiento descontrolado (consecuencia del anterior) | 4 corridas de reparación | Sí, ya resuelto por el pre-commit |
| Fragmentación intra-nota (anexar sin leer) | Backlog: 4 items duplicados | No: juicio |
| Frontmatter ausente/incompleto | 11 notas sin `type` | Sí, trivial |
| Routing equivocado | 3 | No: semántico |
| Contenido factualmente incorrecto | 3 | No: juicio |
| **Nota duplicada** (fallo de search-before-write) | **1 en 153** | Parcial, fallo raro |
| **Pérdida por edición concurrente** | **0** | No hay nada que prevenir |

El incidente más caro (12-jul → 03-ago): tres semanas de Deltas anexados al
canon dejaron `agent-solve-it` en **89,2 KB** contra un presupuesto de 12,5 KB,
y costaron una reparación de 2292+/1881− líneas más una rotación de 534 KB.

### 7.1 Defensa principal: `append` solo a `tier: log`

`exo write append` **rechaza por defecto si la nota destino no es `tier: log`**
(exit 3, sin escribir). Un append a una nota canónica es el anti-patrón por
definición: el canon se edita como delta, la bitácora se anexa.

- Escape: `--force`, que queda registrado en el envelope
  (`data.forzado: true`) para que sea auditable.
- **Alcance honesto:** esto muerde en el camino `exo write`. Si el modelo anexa
  al canon con `Edit`, el guard que lo caza es el check de `doctor` "un core
  jamás recibe appends fechados" (spec madre §6.4), no el write-path.

### 7.2 El presupuesto avisa, no rechaza — NO IMPLEMENTADO en v1

> **Estado real:** esta sección describe el diseño acordado, **no lo que el
> código hace hoy**. `exo write` no calcula presupuesto todavía. Se deja así a
> propósito: el pre-commit de la KB ya bloquea y mordió el primer día, con lo
> que el riesgo está cubierto; el aviso previo es comodidad, no protección.
> Queda como deuda declarada, no como funcionalidad prometida.

El pre-commit de la KB (`kbx ratchet --staged` + `kbx budget`) ya bloquea, y
mordió el primer día que se activó. Duplicar ese bloqueo en el write violaría
la línea roja "`/documenta` no puede fallar al cierre de sesión".

`exo write` **avisa** cuando la escritura deja la nota sobre su
`kbx_budget_max`: línea a stderr y `data.presupuesto_excedido: true`. El valor
de avisar antes es que el agente puede partir la nota en el momento, en vez de
descubrirlo cuando el commit rebota.

### 7.3 Toda validación lleva excepción auditable

Meta-lección medida: el primer guard de kbx produjo **falsos rojos
permanentes** (Backlog sobre-presupuesto por diseño, orphans intencionales →
exit 1 perpetuo) y dejó de discriminar deuda nueva de ruido. Se arregló
inventando la excepción declarada en frontmatter (`kbx_budget_max`,
`kbx_orphan_ok`), auditable en el diff.

Todo guard que añada este write-path (`append` fuera de log, dup-gate,
presupuesto) tiene su escape explícito y registrado. Un guard sin vía de
excepción muere por ruido y se lo acaba desactivando entero.

### 7.4 Defensas que NO se construyen

- **Locks, CAS o merge de tres vías** para concurrencia: cero incidentes en 9
  semanas de sesiones paralelas. El retry de `index.lock` no consta que haya
  saltado nunca.
- **Dup-gate pesado**: 1 caso en 153. Se mantiene porque `busca_hybrid` ya
  existe y cuesta una llamada, no porque el riesgo lo justifique.
- **Verificación de contenido**: los 3 casos de hechos incorrectos escritos
  como ciertos son juicio puro. Ningún CLI los caza.

## 8. El canary de kbx (M4-05): no-op en esta campaña, verificado

El plan exige "canary de kbx en el mismo commit" porque `/consolida` falla
fuerte ante `schema_drift` por diseño, y un cutover de índice que no actualice
la lista `consumed` mata la primera consolidación posterior.

**Aquí no aplica, y conviene dejar escrito por qué** para que nadie lo reabra:
`kbx doctor --check-schema` valida el subconjunto que kbx consume de
`~/.basic-memory/memory.db`. El write-path de exo **no toca esa DB**: escribe
markdown en el árbol de la KB, y el watch de basic-memory lo absorbe y
actualiza su propio índice (modo file-first, spec madre §4.4-E2). El schema no
se mueve, luego el canary no puede driftar por M4.

El acoplamiento real de kbx sigue intacto y pendiente: consume `entity`,
`observation`, `relation`, `search_index` y `project`, mientras el índice de
exo tiene `notas`, `aristas`, `trozos`, `notas_fts` y `vectores`. **Eso es
M6-04**, no M4, y es trabajo de ingeniería con decisión de diseño propia.

Corrección sobre el alcance de ese trabajo, verificada en el código de kbx —
importa porque reduce el riesgo del cutover:

- **El pre-commit de la KB sobrevive intacto al repunte.** `kbx ratchet` no
  abre el índice en absoluto, y `kbx budget` solo lo usa para el fallback de
  `project.path` cuando no se le pasa `--kb` explícito (`budget.go:60-63`,
  literal: *"it never reads entity/relation rows"*). El hook los llama con
  rutas explícitas, así que el guard que protege la KB **no depende** de
  M6-04. Quien consume `entity.size` es `targets` (`targets.go:83`), y se
  resuelve con un `stat()`.
- **Lo que de verdad falta en el índice de exo es `tier`**, hoy en
  `entity_metadata` de basic-memory: `targets` lo devuelve por candidata y
  `/documenta` enruta por él. Exo no indexa frontmatter. Se resuelve con una
  columna en `notas` o leyéndolo del fichero en caliente — patrón que `stale`
  ya usa.
- **`observation` muere gratis**: ningún comando de kbx la consulta hoy
  (cero `SELECT` fuera de fixtures). El canary sobre-declara.
- **`project` no tiene equivalente** y no es opcional: `/consolida` llama a
  `kbx budget --json` y `kbx doctor --json` **sin** `--kb`.

## 10. Cobertura que este cutover deja ciega (hallazgo del barrido)

Nada se rompe porque exo escriba markdown con basic-memory vivo — verificado
por ambos lados: exo no abre `memory.db`, y el watcher de bm absorbe incluso
los renames atómicos. Pero **dos guardrails dejan de ver el camino nuevo**, que
es un fallo silencioso por definición:

1. **El reflejo search-before-write** (`hooks.json:31` de reflex 0.13.0)
   matchea `mcp__basic-memory__write_note`. Una escritura vía `exo write` es un
   Bash: el reflejo no la cubre. **Mitigado por diseño**: `exo write new` hace
   el search-before-write nativo (dup-gate, M4-02), así que la garantía se
   mueve del hook al engine en vez de perderse. Queda declarado, no supuesto.
2. **El retrieval-logger** (`hooks.json:40`) solo registra lecturas MCP. Desde
   el cutover del recall (M6-02) y este de escritura, las lecturas vía
   `exo search`/`exo recall` **desaparecen de la telemetría**. La medición del
   hot-path queda sesgada **desde hoy**, no en M5b — y es justo la telemetría
   que justificó el scope de M5a-01 (qué tools del MCP merecen sucesor).
   Acción: portar el logger al camino de exo, o declarar que la medición se
   cierra aquí y M5a-01 se congela con los datos ya recogidos.

**El fallback de `/documenta` a basic-memory tiene fecha de caducidad**: es
sano hoy, pero post-desinstalación degradaría a tools inexistentes. Se retira
en **M5a-03**, cuando el MCP quede apagado. Anotado también en el propio
fichero del comando.

**Verificación exigida al cierre** (sustituye al cambio de código):
`kbx doctor --check-schema` verde DESPUÉS de que exo haya escrito en la KB real
y basic-memory lo haya absorbido. Es además la comprobación que pide la spec
madre §4.4-E2 ("doctor verifica que el índice absorbió lo esperado").

## 9. Items de la campaña, revisados

| Item | Estado | Nota |
|---|---|---|
| M4-01 `exo write` v1 | **Recortado**: `new` + `append` | `replace_section` fuera: se lo queda `Edit` |
| M4-02 search-before-write | Hecho, **recalibrado** | Solape de slug, no retrieval semántico |
| M4-03 validación que auto-completa | Hecho | Nunca rechaza por frontmatter |
| M4-04 veto RO por defecto | Hecho | Escribir exige el subcomando `write` explícito |
| M4-05 canary de kbx | **No-op justificado** | Se cierra con verificación, no con código (§8) |
| M4-06 reapuntar `/documenta` | Hecho | Con degradación visible a basic-memory |
| — `ruta` en `search --json` | **Añadido** | No estaba en el plan; sin él el camino de edición nace cojo |
| — guard anti-Delta en `append` | **Añadido** | La defensa de mayor ROI según el forense |

### 9.1 `/consolida` no se toca, y por qué

M4-06 dice "reapuntar `/documenta` **y `/consolida`**". Se reapunta el primero
y se deja el segundo, deliberadamente:

1. **`/consolida` no escribe por el MCP.** Todo su diagnóstico va por `kbx`
   (`rotate`, `budget`, `doctor`, `stale`, `diff-since`) y sus escrituras las
   hace declarando "mismo contrato que `/documenta` v2" — o sea, **hereda** el
   cambio sin que haya que editarlo.
2. **Vive en el marketplace** (`agent-develop`, plugin `reflex`). Modificarlo
   es publicar, y publicar es acción externa reservada a Paul. Además su
   migración al monorepo ya tiene item propio: M6-03.
3. Lo que sí le afectará de verdad es el cambio de fuente de datos de `kbx`
   (M6-04), no el write-path.

Cuando se toque, el cambio es de una línea: los appends a `log/<slug>-bitacora`
y a `log/backlog-diario` pasan a `exo write append`, que es justo el patrón
para el que se construyó.
