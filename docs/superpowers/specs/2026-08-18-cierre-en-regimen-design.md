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
  ahora, que es barato.

### 1.2 Deuda que ya no existe

Dos items del backlog están cerrados de facto y se retiran sin trabajo:

- **Pin post-compactación**: `exo-recall.sh:95` matchea `verify-before-done|verify-before-commit`.
  Arreglado en el cutover. (El bug vive aún en `basic-memory-recall.sh:138`, que
  es el script muerto: se va entero con M6-03.)
- **`reflex-baseline.sh` tragando errores de jq**: cero ocurrencias de
  `2>/dev/null` en el fichero. Arreglado.

Queda en pie, barato: **el chequeo defensivo de degradación silenciosa** del
indexer. Lectura superficial de `vectores.rs`/`indexer.rs` no encuentra ningún
camino de fallback a vector hash (el modo de fallo de `empirica`); los `.ok()?`
de `indexer.rs:33-44` son de la llamada a git para recencia, no del embed. Se
cierra con una lectura de 10 minutos dentro de la campaña, no con una campaña.

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

### 3.2 Entra M6-06 — recall en el punto de uso

**Qué.** Un hook `UserPromptSubmit` que busca el prompt de Paul en la KB e
inyecta lo relevante antes de que el modelo empiece a pensar.

**Por qué encaja.** La pieza cara ya está construida: `exo recall` tiene modo
consulta (`busca_hybrid`) desde M2-08 y el umbral `0.40` está sellado desde
M2-07. Falta el hook, que es pequeño, y es el mismo patrón de transporte
mecánico que A1 y que el recall de arranque: **cero decisión del modelo**.

**Riesgos reconocidos, a resolver en su diseño**: latencia por turno (~1s en
frío, menos con la cache de M6-01b), tokens de contexto por turno, y ruido si
inyecta irrelevante —el sitio más caro donde meter ruido—. Mitigaciones
candidatas: umbral más alto que el de búsqueda, cap de 2-3 hits, y el patrón
warn-only con medición de FP que reflex ya tiene.

**No cubre** el caso "afirmo algo a mitad de mi propio razonamiento sin prompt
de por medio". Ningún hook razonable lo ataja; se acepta.

> **M6-06 tiene su propio ciclo.** Este documento fija que entra y por qué. El
> diseño —qué se inyecta, con qué formato, umbral, cap, cuándo se abstiene—
> merece su propio brainstorm al abrir la campaña. Aquí no se decide.

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
| M6-05 | Cutover de doctrina: `core-index`, `doctrina-agentes`, `~/.claude/CLAUDE.md`, description de `recon-first` | Ninguno nombra basic-memory como la memoria; la vía de consulta que predican es la que existe |
| M6-04 | kbx apuntando al índice del engine + lista `consumed` del schema-canary en el MISMO commit | `kbx doctor` verde contra el índice del engine; `/consolida` no explota |
| M6-03 | reflex al monorepo exo; scripts consultando el engine; **resucitar `search-before-write` y `retrieval-logger`** sobre el camino CLI; barrer rutas absolutas y `basic-memory-recall.sh` | Cero matchers `mcp__basic-memory__*`; los dos guardrails vuelven a disparar; suite de reflex verde |
| M6-06 | Recall en el punto de uso (brainstorm propio antes de implementar) | Un prompt sobre un tema con notas trae sus notas sin que nadie lo pida |
| — | Chequeo defensivo: el indexer no degrada a vector hash sin avisar (10 min de lectura) | Verificado o arreglado |

**Rollback**: un flag en `hooks.json`, como todas las campañas de M6.

### C10 — M5a-02: config propia

Corta el acoplamiento RO a `~/.basic-memory/config.json` y mata el hardcode
`projects["kb-demo"]` de `engine/src/lib.rs:71`. **Bloquea M5b**: sin esto,
desinstalar basic-memory deja el engine sin config.

Nota: este item era además el requisito transversal de C11 (en la máquina de
otro, el engine no encuentra su KB). Se hace aquí porque bloquea el cierre, no
porque se abra C11.

### C11 — M5b: desinstalación

**Lo ejecuta Paul.** Acción destructiva, línea roja no delegable. Precondición
dura —única superviviente del régimen §0— **M6 completo y probado**. Checklist
del plan original, sin cambios salvo los dos añadidos de §1.1:

1. El hook de recall no llama a basic-memory por ninguna vía, ni en el FALLBACK.
2. Ningún hook conserva matchers `mcp__basic-memory__*` vivos.
3. `search-before-write` y `retrieval-logger` disparan sobre el camino CLI.
4. kbx apunta al índice del engine y `consumed` está actualizado.
5. `/documenta` y `/consolida` corren end-to-end sin el MCP.
6. exo tiene config propia (M5a-02).
7. La KB está commiteada y pusheada en kb-demo.

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
