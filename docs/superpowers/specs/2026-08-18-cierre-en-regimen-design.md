# Cierre de exo en régimen — dónde estamos, a dónde vamos, qué falta

> **Qué es este documento.** Una revisión del plan de cierre
> (`plans/2026-08-17-cierre-exo-m2-a-m5b.md`) hecha con C8 ya ejecutado y con el
> terreno verificado, no recordado. No lo sustituye: cambia tres cosas y las
> justifica. El régimen §0 de aquel plan —proyecto personal, cerrar ya, sin
> métricas nuevas— sigue vigente y no se reabre.
>
> **Estado**: las decisiones de §2 (definición de terminado) y §3.1/§3.2 (M5a-01
> fuera, M6-06 dentro) las tomó Paul en la sesión de brainstorm del 2026-08-18.
> El documento queda pendiente de su revisión.
>
> **Revisado por consultor fable el 2026-08-18: FIRMA-CON-CAMBIOS**
> (`consultas/2026-08-18-cierre-regimen/consultor-cierre.md`). Los siete cambios
> están aplicados. Firmó las tres decisiones; tumbó el claim 4 y una de mis
> propuestas (resucitar los guardrails). Los hallazgos H1-H8 del verdict están
> incorporados abajo.

---

## 1. Dónde estamos

El goal del plan era *"llevar exo de engine que busca a framework que sustituye a
basic-memory y a superpowers en el uso diario de Paul"*. **La mitad de
superpowers está hecha** desde C8 (2026-08-18): `process@exo` sirve las 7 skills,
superpowers queda disabled con sus cachés como rollback.

Cerrado y verificado contra el repo:

| Campaña | Qué dejó |
|---|---|
| M0, M1a | Modelo y corpus decididos; repo y skills preparadas |
| **C5** — M2-08/09 | **E1 read cerrado**: `exo recall` vive, gate de 3 patas pasado (paridad ∅, engine-hybrid 48/55, recall <2s) |
| **C6** — M6-01/02 | Índice fresco sin daemon + cache de embeddings; **el arranque de cada sesión lo sirve el engine**, con el FALLBACK reescrito |
| **C7** — M4 | **E2 write**: `exo write new\|append`; `/documenta` va por el engine |
| **C8** — M3+M1b | Marketplace `exo`, `process@exo`, superpowers jubilado |

**El engine ya busca, escribe, sirve el arranque y sirve las skills.** Lo que
queda no es capacidad nueva: es **cortar el cordón con basic-memory sin que nada
muera en silencio**.

### 1.1 Dos hallazgos que cambian el diagnóstico

Verificados en esta sesión, no estaban en el plan:

- **El MCP de basic-memory lleva un día muerto y nadie lo notó.** Invocaciones
  reales en 14 días y 42 sesiones: 2 `search_notes`, 3 `read_note`, 16
  `edit_note` — y **cero el 2026-08-18**, el primer día completo tras C7. Las
  `edit_note` son todas anteriores a C7: ese camino lo sustituyó `exo write`.
- **El éxito del cutover desenchufó dos guardrails de reflex, y ya están
  muertos, no van a morir.** Ambos cuelgan de matchers MCP:
  - `search-before-write` (matcher `mcp__basic-memory__write_note`) protegía el
    write-path. Hoy se escribe con `exo write` por Bash: **el matcher no
    dispara**. El guardrail que evita duplicar notas está inerte desde C7.
  - `retrieval-logger` (matcher `mcp__basic-memory__read_note|search_notes|…`)
    medía el retrieval. Con el CLI no loguea nada: **la medición está ciega**
    desde el cutover del recall.

  Esto es exactamente el modo de fallo que M6-03 existía para prevenir —solo que
  ya ocurrió, y en vez de descubrirlo al desinstalar basic-memory lo descubrimos
  ahora, que es barato. **Pero la conclusión no es resucitarlos**: ver §3.4.

### 1.2 Deuda que ya no existe

Dos items del backlog están cerrados de facto y se retiran sin trabajo:

- **Pin post-compactación**: `exo-recall.sh:95` matchea `verify-before-done|verify-before-commit`.
  Arreglado en el cutover. (El bug vive aún en `basic-memory-recall.sh:138`, que
  es el script muerto: se va entero con M6-03.)
- **`reflex-baseline.sh` tragando errores de jq**: cero ocurrencias de
  `2>/dev/null` en el fichero. Arreglado.

### 1.3 Y una deuda que resultó ser un bug (H1)

Lo que entró como "chequeo defensivo de 10 minutos" es un **fix**. No hay
fallback a vector-hash —eso se verificó y es cierto—, pero **el indexer no es
transaccional por nota**: `indexer.rs:113` committea el `mtime` en `notas`
*antes* de embeber los trozos, y no hay `BEGIN`/`COMMIT` envolviendo la
iteración. Un fallo a mitad (embed, lock, kill) deja la nota con mtime fresco y
vectores viejos o ausentes, y **la corrida siguiente la salta para siempre**:
el índice se queda mintiendo en silencio sobre esa nota.

El disparador ya ocurrió: `~/.claude/exo-index.log:13` registra un
`database is locked` reindexando `kb-demo/agents`. **Sin daño vivo**
—verificado: 138 notas, cero sin trozos, y esa nota está íntegra con el mtime
del disco—, pero el mecanismo está armado y el hook que lo dispara corre
detached, con un log que no mira nadie.

Fix: transacción por nota (o diferir el upsert de `notas` al final). ~15 líneas,
dentro de C9, sin campaña propia.

---

## 2. A dónde queremos llegar

**Definición de terminado, firmada por Paul (2026-08-18): «sistema personal en
régimen».**

Significa exactamente esto:

1. basic-memory **desinstalado**; exo es la única memoria.
2. Ningún guardrail, hook o script apuntando a algo que ya no existe —ni
   matchers MCP muertos, ni doctrina que nombre un motor jubilado.
3. La deuda que **rompe en silencio**, barrida. No la deuda cosmética.
4. El agente **consulta la memoria en el punto de uso**, no solo al arrancar.

Lo que **NO** significa, explícitamente y por YAGNI:

- **No** es "producto instalable por terceros". La campaña de **bootstrap de
  instancia** —C11 en el plan del 17-ago, renumerada fuera de la cola aquí
  porque no se abre— y el destilado de transcripts quedan especificados en aquel
  plan y **sin abrir**.
- **No** incluye M7 (templates), ya fuera desde el régimen §0.
- **No** incluye métricas nuevas, ventanas ni gates numéricos. Régimen §0.

---

## 3. Qué cambia respecto al plan del 2026-08-17

### 3.1 M5a-01 (servidor MCP propio) se CAE del plan

**Decisión.** No se construye. M5a queda reducido a M5a-02 (config propia).

**Por qué.** El plan lo justificaba con "el hot-path medido": `read_note`,
`search_notes`, `recent_activity`. El uso real de esas tres en 14 días es
**5 llamadas y cero de `recent_activity`**. Construir un servidor `rmcp` con sus
tests para eso es el over-engineering que el régimen §0 vino a cortar.

**La objeción de Paul, que es la buena, y por qué no cambia la decisión.** El
uso bajo no prueba que el MCP sobre: prueba que **el agente no consulta la
memoria lo suficiente** —debería validar lo que sabe entre sesiones mucho más a
menudo. Cierto, y es un problema real. Pero es de **adherencia, no de
transporte**: las tools llevan meses disponibles y el uso es 5. La casa ya tiene
el diagnóstico escrito —métrica D, NO-PASS **por cobertura, no por eficacia**— y
la respuesta: cuando el modelo no hace algo por su cuenta, no se le pide más
fuerte, **se le pone delante con un hook**. Es lo que hace A1 con los subagentes
y el recall con el arranque. La carencia se ataca en M6-06, no con un MCP.

**Reversible**: si tras un ciclo real duele, se construye entonces, con uso real
detrás. Nada de lo que se hace aquí lo impide.

**M5a-03 no desaparece, se absorbe.** El plan tenía un item "basic-memory
apagado pero instalado, periodo sin divergencias". Al reducirse M5a a M5a-02,
ese periodo pasa a ser el criterio de terminado §5 —una semana de trabajo normal
sin echarlo de menos— y el rollback es **re-añadir la entrada de `mcpServers`**.
Decirlo importa porque hay al menos un fichero (`~/.claude/commands/documenta.md`)
que cita M5a-03 como su fecha de caducidad.

### 3.2 Entra M6-06 — recall en el punto de uso

**Qué.** Un hook `UserPromptSubmit` que busca el prompt de Paul en la KB e
inyecta lo relevante antes de que el modelo empiece a pensar.

**Por qué encaja.** La pieza cara ya está construida: `exo recall` tiene modo
consulta (`busca_hybrid`) desde M2-08 y el umbral `0.40` está sellado desde
M2-07. Falta el hook, que es pequeño, y es el mismo patrón de transporte
mecánico que A1 y que el recall de arranque: **cero decisión del modelo**.

**Riesgos reconocidos, a resolver en su diseño**:

- **Latencia: ~1s POR PROMPT, no "~1s en frío".** Cada invocación del CLI es un
  proceso nuevo que paga la carga del ONNX; la cache de M6-01b cachea embeddings
  de trozos, **no** el arranque del modelo. Si M6-06 usa hybrid, ese segundo se
  paga en cada turno. Mitigación candidata: **FTS-only** para este hook, que no
  toca el modelo, aceptando peor recall a cambio de coste cero.
- **Gate de disparo**: los prompts triviales ("sí", "dale", "commitea") no
  merecen ni una búsqueda. Hace falta un criterio de abstención barato.
- **Tokens de contexto por turno** y **ruido si inyecta irrelevante** —el sitio
  más caro donde meter ruido—. Mitigaciones: umbral más alto que el de búsqueda,
  cap de 2-3 hits, y el patrón warn-only que reflex ya tiene.

**No cubre** el caso "afirmo algo a mitad de mi propio razonamiento sin prompt
de por medio". Ningún hook razonable lo ataja; se acepta.

> **M6-06 tiene su propio ciclo.** Este documento fija que entra y por qué. El
> diseño —qué se inyecta, con qué formato, umbral, cap, cuándo se abstiene—
> merece su propio brainstorm al abrir la campaña. Aquí no se decide.

### 3.4 Los dos guardrails desenchufados se RETIRAN, no se resucitan

Mi propuesta inicial era reapuntarlos al camino CLI. El consultor la tumbó y
tiene razón en los dos casos:

- **`search-before-write` ya está subsumido, y por algo mejor.** `exo write new`
  trae un dup-gate **bloqueante** (`escritor.rs:226`: candidatas por solape de
  slug ⇒ `Rechazo::Duplicada`, exit 3, `--force` para saltarlo). Eso es
  estrictamente más fuerte que el hook viejo, que era warn-only una vez por
  sesión. Resucitarlo sobre `PreToolUse:Bash` añadiría un cuarto script a esa
  cascada —ya corren tres en cada Bash— para avisar de algo que el engine ya
  bloquea. YAGNI: se retira con nota.
- **`retrieval-logger` reabriría el régimen §0.** Su propia cabecera se declara
  instrumentación con "ventana ~2-3 semanas; retirar cuando haya señal", y §0
  retiró métricas, ventanas y gates. Decidido por Paul (2026-08-18): **se
  retira**. Si el ciclo de M6-06 necesita observabilidad, la trae su diseño — y
  será instrumentación del hook inyector, no de lecturas del modelo que ya no
  existen.

### 3.3 M6 se reordena por coste/riesgo, no por número

C6 quedó partida (01/02 hechos). Sus restos no se ejecutan en orden numérico:

| Orden | Item | Por qué ahí |
|---|---|---|
| 1º | **M6-05** doctrina | Es solo texto y **hoy miente**: `core-index`, `doctrina-agentes`, el `CLAUDE.md` global de Paul y la description de `recon-first` siguen diciendo que la memoria persistente es el MCP, cuando el arranque ya lo sirve `exo recall`. Cuesta cada sesión y se arregla en media hora |
| 2º | **M6-04** kbx al índice del engine | Riesgo cosido: `/consolida` falla-fuerte ante `schema_drift`. La lista `consumed` del schema-canary va **en el mismo commit** o el primer `/consolida` post-cutover muere |
| 3º | **M6-03** reflex al monorepo | El gordo. Es donde se resucitan los dos guardrails desenchufados (§1.1) y donde mueren las rutas absolutas a `/home/paul/…` |
| 4º | **M6-06** recall en el punto de uso | Después de M6-03 porque toca los mismos hooks; abrirlo antes obliga a tocar reflex dos veces |

---

## 4. Los pasos

```
C9 — M6 completo ──→ C10 — M5a-02 config propia ──→ C11 — M5b desinstalar
   05 doctrina                (bloquea M5b)              (lo ejecuta Paul)
   04 kbx + canary
   03 reflex al monorepo
   06 recall en el punto de uso
```

**Nota de numeración**: en el plan del 17-ago, C9 era "M5a MCP propio", C10 era
M5b y C11 el bootstrap de instancia. Al caerse M5a-01 y quedar el bootstrap sin
abrir, la cola se renumera: **C9 = M6 completo, C10 = config propia, C11 = M5b**.
El bootstrap pierde su número y se cita por nombre mientras siga cerrado.

### C9 — M6 completo

| Item | Qué | Cierre |
|---|---|---|
| M6-05 | Cutover de doctrina: `core-index`, `doctrina-agentes`, `~/.claude/CLAUDE.md` (líneas 3 y 19), description de `recon-first`, `settings.json:75`, y el mensaje de `basic-memory-remind.sh` (vivo en `Stop`, predica guardar en el motor jubilado). **Más una línea de allowlist para `Bash(exo …)`/`Bash(kbx …)`** | Ninguno nombra basic-memory como la memoria; la vía de consulta que predican existe **y no pide permiso cada vez** |
| M6-04 | kbx apuntando al índice del engine + lista `consumed` del schema-canary en el MISMO commit | `kbx doctor` verde contra el índice del engine; `/consolida` no explota |
| M6-03 | reflex al monorepo exo; scripts consultando el engine; **retirar** `search-before-write` y `retrieval-logger` (§3.4); barrer rutas absolutas, `basic-memory-recall.sh`, y el README/`plugin.json` de reflex que describen el mundo MCP. **Seam de KB en `compose-inject.sh:21`**, que hoy lee `~/.basic-memory/config.json` en cada `SubagentStart` (patrón env de `exo-recall.sh`, default actual) | Cero matchers `mcp__basic-memory__*`; el camino de escritura queda protegido por el engine, **verificado con un `exo write new` duplicado real**; suite de reflex verde |
| M6-06 | Recall en el punto de uso (brainstorm propio antes de implementar) | Un prompt sobre un tema con notas trae sus notas sin que nadie lo pida |
| — | **Fix del indexer (H1)**: transacción por nota, o diferir el upsert de `notas` al final. Ver §1.3 | Un fallo a mitad deja el índice como estaba y la nota se reintenta en la corrida siguiente |

**Rollback**: un flag en `hooks.json`, como todas las campañas de M6.

**Residuo a barrer de paso**: `~/.claude/commands/documenta.md.pre-m4.bak`.

**Verificado limpio por el consultor** (para que nadie lo re-busque): `process@exo`
y `paul-profile@exo` no referencian basic-memory; `settings.json` no tiene hooks
huérfanos; el `.claude/` de kb-demo está limpio; el crontab, vacío.

### C10 — M5a-02: config propia

Corta el acoplamiento RO a `~/.basic-memory/config.json` en sus **dos** sitios
—el hardcode `projects["kb-demo"]` de `engine/src/lib.rs:71` y el fallback
de `compose-inject.sh:21`, al que M6-03 ya le habrá puesto el seam—. **Bloquea
M5b**: sin esto, desinstalar basic-memory deja al engine sin config. Hacer el
seam en M6-03 es lo que evita que C10 tenga que reabrir reflex recién migrado.

Nota: este item era además el requisito transversal de C11 (en la máquina de
otro, el engine no encuentra su KB). Se hace aquí porque bloquea el cierre, no
porque se abra C11.

### C11 — M5b: desinstalación

**Lo ejecuta Paul.** Acción destructiva, línea roja no delegable. Precondición
dura —única superviviente del régimen §0— **M6 completo y probado**. Checklist
del plan original, sin cambios salvo los dos añadidos de §1.1:

1. El hook de recall no llama a basic-memory por ninguna vía, ni en el FALLBACK.
2. Ningún hook conserva matchers `mcp__basic-memory__*` vivos.
3. El camino de escritura está protegido por el dup-gate del engine (§3.4), y
   `search-before-write`/`retrieval-logger` están retirados, no colgando.
4. kbx apunta al índice del engine y `consumed` está actualizado.
5. `/documenta` y `/consolida` corren end-to-end sin el MCP, y **el fallback MCP
   de `/documenta` está retirado** (su caducidad citaba M5a-03; ver §3.1).
6. exo tiene config propia (M5a-02), en sus dos sitios.
7. `basic-memory-remind.sh` ya no existe, o ya no nombra basic-memory.
8. La KB está commiteada y pusheada en kb-demo.
9. **La desinstalación incluye quitar `basic-memory` de `mcpServers` en
   `~/.claude.json`.** El server arranca con `uvx basic-memory mcp`: mientras la
   entrada exista, uvx **reinstala el paquete al vuelo** en el próximo arranque.
   Purgar el paquete sin tocar esa entrada no desinstala nada. Quitar la entrada
   es además el rollback de §3.1, en sentido inverso.

---

## 5. Criterio de terminado

exo está cerrado cuando:

- basic-memory está desinstalado y una semana de trabajo normal no lo echa de
  menos;
- `kbx doctor`, `/documenta` y `/consolida` corren verdes sin él;
- ningún hook, script o nota de doctrina nombra un motor que no existe;
- un prompt sobre un tema con notas en la KB trae sus notas sin que nadie lo pida.

A partir de ahí exo entra en **mantenimiento**: se afina con el uso. Los frentes
que quedan escritos y sin abrir —C11 bootstrap, destilado de transcripts, MCP
propio si duele su ausencia— se reabren solo con un dolor concreto detrás.
