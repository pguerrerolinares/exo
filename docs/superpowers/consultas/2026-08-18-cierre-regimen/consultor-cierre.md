# Verdict del consultor — spec "cierre de exo en régimen" (2026-08-18)

> Consultor fable fresco, verificación primaria propia sobre transcripts, hooks,
> engine y git. No he editado la spec ni commiteado nada. Spec revisada:
> `docs/superpowers/specs/2026-08-18-cierre-en-regimen-design.md`.

## Veredicto global: **FIRMA-CON-CAMBIOS**

Las tres decisiones de la spec son correctas y las firmo. Pero dos piezas de su
§1.2/§4 no sobreviven a la verificación tal como están escritas:

1. El "chequeo defensivo de 10 minutos de lectura" del indexer **no es una
   lectura, es un fix**: el indexer no es transaccional por nota y ya falló una
   vez en producción (evidencia abajo, hallazgo H1).
2. "Resucitar `search-before-write` y `retrieval-logger`" en M6-03 está mal
   dirigido: el primero ya está **subsumido y superado** por el dup-gate
   bloqueante de `exo write new`, y el segundo contradice el régimen §0 que la
   propia spec dice no reabrir (H2, H3).

Además hay cuatro acoplamientos a basic-memory que la spec no lista y que son
exactamente la familia de fallo que pedía buscar (H4–H7).

---

## 1. Verificación de los 5 claims

### Claim 1 — "El MCP de basic-memory está muerto" → **SOBREVIVE** (con dos precisiones)

Método propio: extraje cada bloque `tool_use` real de los `.jsonl` con
`jq 'select(.type=="assistant") | .message.content[] | select(.type=="tool_use") | .name'`
sobre los transcripts con mtime ≥ 2026-08-04 (45 ficheros; el brief decía 42
sesiones — un jsonl no es exactamente una sesión, no material).

Resultado, con timestamp de cada invocación:

| Tool | Total | Fechas |
|---|---|---|
| `edit_note` | 16 | 7× 04-ago 00:14–00:16 · 9× 17-ago 21:32–21:36 (hora local) |
| `read_note` | 3 | 04-ago · 2× 17-ago |
| `search_notes` | 2 | ambas **03-ago** (fuera de la ventana estricta de 14 días) |
| `recent_activity`, `build_context`, `write_note` | 0 | — |
| **Cualquiera el 2026-08-18** | **0** | con 27 jsonl generados ese día |

Precisiones:

- **El conteo real de lecturas en ventana estricta es 3, no 5**: las 2
  `search_notes` son del 03-ago, capturadas por mtime del fichero, no por fecha
  del evento. Refuerza la conclusión, no la debilita.
- El claim "las `edit_note` son todas anteriores a C7" sobrevive **por 3
  horas**: las últimas son del 17-ago 21:36 local y el merge de C7 es del
  18-ago 00:48 (`89dbe95`, verificado con `git log --format=%cI`). Cierto, pero
  la spec debería decir "anteriores al merge de C7" sin más épica: la ventana
  post-cutover del write-path es de horas, no de días.

El sesgo de ventana que el brief reconoce es real, pero el dato que mata a
M5a-01 no es el día 18: es que **con el MCP plenamente vivo y la doctrina
predicando usarlo ("úsalo SIEMPRE", CLAUDE.md:19), el read-path registró 3–5
lecturas en dos semanas**. Eso es adherencia, no transporte. Confirmado.

### Claim 2 — "Dos guardrails desenchufados" → **SOBREVIVE** (con un matiz)

Verificado en `hooks/hooks.json` de reflex 0.13.1: `search-before-write` cuelga
del matcher `mcp__basic-memory__write_note` y `retrieval-logger` de
`mcp__basic-memory__read_note|search_notes|build_context|recent_activity`.
El write-path real es `exo write` por Bash (C7) y el read-path `exo recall`/
`exo search` (C6): ninguno de los dos matchers ve ese tráfico. El
`reflex-retrieval-log.jsonl` lo confirma empíricamente: su última entrada es
del 17-ago 21:31 — ciego desde el cutover, como dice la spec.

Matiz: no están muertos al 100% — dispararían si algo usara el camino MCP
residual, y ese camino existe: el fallback de `/documenta` a
`mcp__basic-memory__*` (ver H6). Muertos en el camino nominal: correcto.

### Claim 3 — Deuda dada por cerrada → **SOBREVIVE**

- `exo-recall.sh:95` matchea `verify-before-done|verify-before-commit` en el
  case del pin post-compactación. Verificado con grep, línea exacta.
- `reflex-baseline.sh`: cero ocurrencias de `2>/dev/null` (grep exit 1).
- Bonus verificado: `crontab -l` vacío (el `crontab -r` pendiente del plan
  también está saldado, la bitácora lo registra).

### Claim 4 — "El engine no degrada en silencio" → **NO SOBREVIVE tal como está escrito** (H1)

Lo que la spec mira, está bien: **no existe ningún fallback a vector-hash**.
`embebe_batch` (fastembed local) propaga `Err` con contexto; `vectores::lee`
valida longitud exacta 768×4 y trata blobs corruptos como ausentes
(re-embebe); el KNN jamás recibe basura plausible. Los `.ok()?` de
`indexer.rs:33-44` son del git de recencia, confirmado.

Lo que la spec no mira y yo sí (`indexer.rs:76-146`): **el indexer no es
transaccional por nota, y el orden de escritura es el peligroso**:

1. `INSERT ... ON CONFLICT SET mtime = excluded.mtime` en `notas` (autocommit)
2. delete+insert en `notas_fts`
3. aristas
4. `reindexa_trozos_de_nota` — borra trozos/vectores viejos y **recién aquí embebe**

Si el paso 4 falla (embed, lock, kill del proceso), la nota queda con **mtime
fresco ya committeado y vectores viejos o ausentes** — y la siguiente corrida
la ve como "sin cambios" (`existentes.get(&ruta_rel) == Some(&mtime)`,
`indexer.rs:99`) y la salta. La nota desaparece del retrieval semántico **para
siempre**, hasta que su mtime vuelva a cambiar. FTS además queda actualizado y
los vectores no: inconsistencia parcial, que es peor que fallo limpio.

Y no es teórico: `~/.claude/exo-index.log` contiene ya un
`error: reindexar trozos/vectores de kb-demo/agents: borrar vector
rowid=6076: database is locked` — un fallo real a mitad del paso 4, en el hook
de Stop detached cuyo log nadie mira. Esa corrida tuvo la suerte de que la
siguiente reindexó todo, pero la ventana existe por construcción. El
`busy_timeout` de 5 s (`lib.rs:51`) reduce la probabilidad; no la elimina (una
nota grande con ONNX cargando puede tener el lock más de 5 s desde el otro
lado).

**El "chequeo de 10 minutos de lectura" de la spec §1.2 es en realidad un fix
de ~15 líneas**: transacción por nota (o mover el upsert de `notas` al final
del bucle). Sigue cabiendo dentro de C9 sin campaña propia, pero hay que
escribirlo como fix, no como lectura.

### Claim 5 — Estado de milestones → **SOBREVIVE**

Verificado contra `git log` con timestamps:
`3d7f073` (M2-08/09, E1 read, 17-ago 22:59) · `06ad42c` (M6-01/02, 17-ago
23:52) · `89dbe95` (C7/M4, 18-ago 00:48) · `4f05e8e` (C8/M3+M1b, 18-ago 07:50)
· cabeza `72ae7f3` (la propia spec). El runbook de C8 declara cutover
ejecutado y verificado, incluido el probe M3-02 verde
(`inject-emitted type=reflex:executor perfil=reducido bytes=997`). Todo en
~9 horas del 17 a la mañana del 18 — otro recordatorio de que "un día
post-cutover" es literal.

---

## 2. Juicio de las 3 decisiones

### D1 — Matar M5a-01 (servidor MCP propio): **FIRMO**

El razonamiento adherencia-vs-transporte es correcto y los datos lo apoyan:
las tools llevaban meses disponibles, la doctrina mandaba usarlas, y el
read-path registró 3–5 lecturas en 14 días. Un MCP propio no cambia el
comportamiento del modelo; un hook sí (precedente A1 verificado en el log).

A la pregunta "¿hay algo que el MCP dé y un hook no?": lo único real es el
**pull a demanda con tools descubribles en la lista del modelo**. Pero ese
pull ya existe — `Bash(exo search --type hybrid ...)` — y la descubribilidad
la sirve M6-05 (la doctrina y el FALLBACK ya lo predican). La única fricción
diferencial es el permission prompt de Bash, y eso se arregla con una línea de
allowlist (`Bash(exo *)` o equivalente en settings — hoy NO está, verificado),
no con un servidor rmcp con tests. Con eso, el MCP propio no aporta nada que
no tenga ya. Reversible como dice la spec.

### D2 — Meter M6-06 (recall en el punto de uso): **FIRMO el mecanismo**

Consideré las alternativas y las descarto con dato: más doctrina (refutado —
es exactamente lo que había y produjo 3 lecturas en 14 días), MCP propio
(mismo problema de adherencia, ver D1), memoria nativa de Claude Code (OFF
deliberado en doctrina, doctrina-agentes:22, para no fragmentar el canon). El
hook `UserPromptSubmit` es el mismo patrón push determinista ya validado dos
veces (A1, recall de arranque). Es la respuesta correcta.

Dos riesgos que su ciclo de diseño debe tratar como de primera clase (los dejo
aquí porque afectan a la viabilidad, no solo al tuning):

1. **El coste real por prompt no es ~1s "en frío": es ~1s SIEMPRE que use
   hybrid.** Cada invocación del CLI es un proceso nuevo → paga la carga del
   runtime ONNX en cada prompt de Paul. La cache de M6-01b cachea embeddings
   de trozos, no el arranque del modelo. Salidas posibles: FTS-only para este
   hook (milisegundos, sin ONNX), o un camino de query precomputado. Si el
   diseño ignora esto, M6-06 nace con una espera de un segundo por turno.
2. **Gate de disparo**: "dale", "sí", "continúa" no deben pagar búsqueda ni
   meter ruido. Longitud mínima / heurística barata antes de tocar el engine.

### D3 — Orden de C9 (05 → 04 → 03 → 06): **FIRMO**

Busqué dependencias que lo rompieran y no las hay:

- **Ventana A1 con freeze** ("nada de `/consolida` ni `claude plugin update`
  durante la ventana", Backlog frente 1) era la candidata a romper M6-03: un
  bump de reflex durante la ventana rompería el freeze. Refutado: la ventana
  nunca se abrió (`~/.claude/a1-freeze-anchors.txt` no existe), el vigilante
  ya no está en cron (crontab vacío, bitácora lo confirma) y C8 ya hizo
  cirugía de plugins sin freeze de por medio.
- M6-04 (kbx) y M6-03 (reflex) no comparten superficie; M6-06 tras M6-03 es
  correcto por la razón que da la spec (mismos hooks).
- Única nota de scope, no de orden: `kbx` hoy apunta por defecto a
  `~/.basic-memory/memory.db` (verificado con `strings` en el binario:
  `index path (default: $KBX_DB or ~/.basic-memory/memory.db)`). Si el índice
  de exo no expone todo lo que `kbx doctor` consume, M6-04 puede requerir
  tocar el engine además de kbx. No rompe el orden; puede engordar el item.

---

## 3. Hallazgos nuevos (la familia que pedías: acoplamientos que el estrangulamiento dejó atrás)

**H1 — El indexer puede perder notas del índice en silencio y ya falló una vez.**
Detalle completo en el claim 4. Es el hallazgo más caro de la sesión: mtime
committeado antes de embeber + sin transacción por nota + hook detached cuyo
log no mira nadie. Fix de ~15 líneas dentro de C9.

**H2 — `search-before-write` no hay que resucitarlo: ya está subsumido.**
`exo write new` trae dup-gate **bloqueante** (`escritor.rs:226`: candidatas
por solape de slug ⇒ `Rechazo::Duplicada`, exit 3, `--force` para saltarlo;
`main.rs:58`). Es estrictamente más fuerte que el hook viejo, que era
warn-only 1×/sesión. Resucitarlo como hook sobre `PreToolUse:Bash` añadiría un
cuarto script a esa cascada (ya corren 3 en cada Bash) para avisar de algo que
el engine ya bloquea. YAGNI: se retira con nota, no se resucita.

**H3 — Resucitar `retrieval-logger` contradice el régimen §0.**
Su propia cabecera dice que es instrumentación de medición con "ventana
~2-3 semanas; retirar cuando haya señal". El régimen §0 retiró métricas,
ventanas y gates — y esta spec dice que §0 "sigue vigente y no se reabre".
Resucitarlo es reabrirlo. Si el ciclo de M6-06 quiere medición de FP de la
inyección, que la traiga su diseño (y será instrumentación del hook inyector,
no de lecturas del modelo que ya no existen). En M6-03: se retira.

**H4 — `compose-inject.sh:21` lee `$HOME/.basic-memory/config.json` para
resolver la KB.** El acoplamiento RO no vive solo en `engine/src/lib.rs`: vive
también en la capa hook, en el compositor de A1 que corre en **cada
SubagentStart**. La spec C10 solo nombra el hardcode del engine. Si M6-03
migra reflex sin tocar esto, C10 tendrá que volver a abrir reflex; si lo tocas
en M6-03 antes de que exista la config propia, apunta a un seam
(`EXO_KB`-style, como ya hace `exo-recall.sh`) con el default actual, y C10
cierra el círculo.

**H5 — `basic-memory-remind.sh` está vivo en `Stop` y predica el motor
jubilado.** Mensaje: *"guardar decisiones y aprendizajes en basic-memory"*.
No es un matcher muerto — dispara en cada sesión con trabajo real — es
doctrina desactualizada inyectada a Paul al cierre. Texto y nombre del script
son de M6-05/M6-03.

**H6 — El fallback de `/documenta` apunta a las tools MCP y su caducidad cita
un item que esta spec acaba de eliminar.** `~/.claude/commands/documenta.md:14-18`:
si el binario exo no existe, cae a `mcp__basic-memory__*`, y el comentario
dice "este fallback caduca en M5a-03". La spec reduce M5a a M5a-02: **M5a-03
ya no existe y nadie recolocó la caducidad**. Tras M5b ese fallback apuntaría
a tools de un server desinstalado. Va al checklist de C11.

**H7 — "Desinstalar basic-memory" sin tocar `~/.claude.json` no desinstala
nada.** El server corre vía `uvx basic-memory` (verificado en `mcpServers`):
mientras la entrada exista, uvx **reinstala el paquete al vuelo** en el
próximo arranque. La desinstalación efectiva ES quitar la entrada de
`mcpServers` (y entonces sí, purgar el paquete). El checklist de C11 debe
decirlo explícitamente; hoy no lo dice.

**H8 — Menores, para barrer de paso:**
- `~/.claude/settings.json:75` menciona basic-memory como memoria rutinaria
  (texto de config de security-review) → lista de M6-05.
- `~/.claude/CLAUDE.md:3` ("fuente de verdad: nota en basic-memory") además de
  la línea 19 que la spec ya cuenta → M6-05.
- Sin allowlist para `Bash(exo ...)`: el pull-path que M6-05 va a predicar
  paga permission prompt cada vez → una línea en settings, va con M6-05 o C10.
- `README.md` y `plugin.json` de reflex describen los hooks del mundo MCP →
  M6-03 de paso.
- Residuo: `~/.claude/commands/documenta.md.pre-m4.bak`.
- Verificado limpio (para que nadie lo re-busque): process@exo y
  paul-profile@exo sin referencias a basic-memory; settings sin hooks
  huérfanos; `.claude/` de kb-demo limpio; crontab vacío.

---

## 4. Cambios concretos propuestos a la spec (por importancia)

1. **§1.2 y §4/C9, chequeo defensivo**: reescribir como fix, no como lectura.
   "El indexer envuelve cada nota en una transacción (o difiere el upsert de
   `notas` al final de la nota); un fallo a mitad deja el índice como estaba y
   la nota se reintenta en la siguiente corrida. Evidencia del modo de fallo:
   `database is locked` en `exo-index.log`." Sigue dentro de C9, sin campaña.
2. **§4/C9, M6-03**: sustituir "resucitar `search-before-write` y
   `retrieval-logger` sobre el camino CLI" por "**retirar** ambos:
   search-before-write está subsumido por el dup-gate bloqueante de `exo write
   new` (exit 3); retrieval-logger era instrumentación de una medición que el
   régimen §0 retiró — si M6-06 necesita medición, la trae su ciclo". El
   criterio de cierre "los dos guardrails vuelven a disparar" cambia a "el
   camino de escritura queda protegido por el engine, verificado con un
   `exo write new` duplicado real". Y el punto 3 del checklist de C11 cambia
   igual.
3. **§4/C9 M6-03 + C10**: añadir `compose-inject.sh:21` al corte del
   acoplamiento a `~/.basic-memory/config.json`. En M6-03 se le pone el seam
   (env + default actual, patrón de `exo-recall.sh`); en C10 el default pasa a
   la config propia. Sin esto, C10 reabre reflex recién migrado.
4. **§4/C11, checklist**: añadir tres puntos — (a) el fallback MCP de
   `/documenta` se retira (su caducidad citaba M5a-03, que esta spec elimina);
   (b) la desinstalación incluye quitar `basic-memory` de `mcpServers` en
   `~/.claude.json` — con uvx, dejar la entrada reinstala el paquete solo;
   (c) `basic-memory-remind.sh` ya no existe o ya no nombra basic-memory.
5. **§4/C9, M6-05**: ampliar la lista con `settings.json:75`, `CLAUDE.md:3`,
   el mensaje de `basic-memory-remind.sh`, y una línea de allowlist para
   `Bash(exo ...)` — es la pieza que hace real el pull-path que la doctrina
   nueva va a predicar, y lo que remata la ventaja residual del MCP propio.
6. **§3.1**: una línea explícita: "M5a-03 (periodo apagado-pero-instalado) se
   absorbe en el criterio §5 (una semana de trabajo normal sin echarlo de
   menos); el rollback es re-añadir la entrada de `mcpServers`". Hoy el item
   desaparece sin que nadie lo diga, y hay al menos un fichero (`documenta.md`)
   que lo cita como su fecha de caducidad.
7. **§3.2, riesgos de M6-06**: elevar el coste ONNX-por-proceso a riesgo
   nombrado (no "~1s en frío": ~1s por prompt si usa hybrid) con FTS-only como
   mitigación candidata, y añadir el gate de disparo para prompts triviales.

## 5. Disenso: qué busqué para objetar y qué refuté

Mandato de disenso cumplido con estas búsquedas deliberadas:

- **Busqué uso del MCP que desmintiera "está muerto"**: encontré 9 `edit_note`
  el 17-ago por la noche — a 3 horas del merge de C7. Sobrevive por
  timestamps (pre-C7), pero obligó a mirar el reloj de cada commit. También
  encontré que 2 de las "5 lecturas" caen fuera de la ventana de 14 días: la
  cifra real es más baja, el claim sale reforzado.
- **Busqué un bloqueador del orden de C9**: la ventana A1 con freeze era el
  candidato serio (prohíbe bumps de plugins). Refutado: anchors inexistentes,
  cron vacío, ventana nunca abierta.
- **Busqué el fallback a vector-hash** (modo de fallo de empirica): no existe
  por ninguna vía — fastembed local con `Err` propagado, validación de
  longitud exacta en `lee()`. En su lugar encontré la no-transaccionalidad del
  indexer (H1), que es la objeción que SÍ prospera contra el claim 4 tal como
  la spec lo escribe ("se cierra con una lectura de 10 minutos": no, se cierra
  con un fix).
- **Consideré objetar que eliminar M5a-03 quita la red de seguridad del
  apagado gradual**: descartado como bloqueo — con uvx, apagado y desinstalado
  son la misma operación sobre `mcpServers`, el rollback es re-añadir una
  entrada JSON, y la KB es markdown en git (el sqlite del MCP es índice
  derivado). Queda como cambio menor de redacción (propuesta 6), no como
  objeción de fondo.
- **Consideré objetar YAGNI contra M6-06 entero**: descartado — el problema de
  adherencia está medido dos veces (métrica D NO-PASS por cobertura; 3
  lecturas en 14 días con doctrina a favor), la pieza cara ya existe
  (`busca_hybrid` + umbral sellado) y el hook es pequeño. Se gana el sitio.
- **Consideré proponer vigilancia del `exo-index.log`** (el detached cuyo
  fallo nadie ve): descartado por YAGNI — infra de monitorización para un
  sistema personal es over-engineering; el fix transaccional de H1 elimina el
  daño persistente, y el residuo (índice rancio una sesión) ya está aceptado
  por diseño en M6-01.
- **Consideré objetar el matiz "los guardrails no están 100% muertos"**
  (dispararían en el camino MCP residual): descartado como objeción — el
  camino residual es el fallback de `/documenta`, que la propuesta 4 retira;
  tras eso, muertos del todo, y la lectura de la spec queda correcta.
