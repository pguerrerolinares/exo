# Gate M4 — write-path de exo (rama `m4-write`)

**GATE: MERGED**

Consultor fresco (fable), sin participación en diseño ni implementación.
Verificación primaria propia: todo lo afirmado abajo lo ejecuté yo contra el
worktree, contra una **copia** de la KB en scratchpad (jamás la real) y contra
el índice real copiado. Ningún camino encontrado por el que `exo write` pierda
o corrompa contenido existente. Hallazgos no bloqueantes: 9, listados como
deuda accionable.

---

## 1. Qué verifiqué y cómo

| Comprobación | Comando / método | Resultado |
|---|---|---|
| Tests de la rama | `cargo test` en el worktree | **111 verdes, 0 rojos** (se afirmaban 108; hay más, no menos) |
| Línea base en main | `cargo test` en `engine/` de main | 98 verdes → **+13, sin regresión** |
| Calidad de los tests | lectura de `engine/tests/escritor.rs` | No triviales: oráculo con pares reales, dup-gate calibrado contra el duplicado histórico, no-overwrite, preservación de frontmatter |
| Slug vs KB real | example temporal llamando a `exo::escritor::slug` sobre los **127 pares title/permalink** con frontmatter de kb-demo (extraídos con parser YAML, no grep) | **108/127 exactos**; 19 divergencias en 3 clases (ver hallazgo N8) |
| `anexa()` / `cola(2)` (sospecha del brief) | 6 ficheros sintéticos: sin `\n` final, con 1, con 3, cuerpo vacío, último char multibyte (`é`), 1 byte; `xxd` antes/después | **Correcto en los 6**: separador `\n\n` exacto, cero bytes perdidos. La sospecha no se materializa |
| Round-trip de frontmatter | `new` con cuerpo que trae `tags`/`kbx_budget_max` + bloque de código con `---` dentro; comparación byte a byte en python | Cuerpo **literal**, YAML del autor **literal**, el `---` del bloque de código no confunde al split (solo separa el primero) |
| No-overwrite | colisión de ruta con y sin `--force` | exit 1 siempre, fichero intacto. `--force` **no** salta colisiones |
| Guard anti-Delta | append a nota `tier: stable` de la copia | exit 3 sin escribir; `--force` escribe con `forzado: true` en el envelope y aviso a stderr |
| Dup-gate | `new --titulo "exo bitácora"` con `exo-bitacora` indexada | exit 3, candidata ~1.00; sin falsos positivos con `cge-bitacora`, `backlog-frentes-abiertos`, `zumaia-…` (tests) |
| Exit codes | todos los anteriores | 0 escrito / 3 gate / 1 error, como dice la spec |
| Permisos | `chmod 444` + append | exit 1 limpio, nada corrupto |
| Fichero sin frontmatter | append | rechaza con "(sin tier)" — seguro por defecto |
| Campo `ruta` en `search --json` | lectura de `evals/retrieval-fase0/harness/*.py`, scripts reflex de agent-develop | Parsean claves concretas; campo aditivo inofensivo. `SCHEMA_VERSION` sigue en 1 (correcto: aditivo) |
| Verificación de producción | `git -C kb-demo show --stat 1683537` | +25/−0 en `log/exo-bitacora.md`: append real sin borrar nada |
| Canary kbx | `kbx doctor --check-schema` corrido por mí hoy | `schema ok`, exit 0 — el no-op de M4-05 (§8 de la spec) se sostiene |
| Veto AGPL | inspección de `escritor.rs` completo | Rust original con diseño replicado por oráculo de la KB; basic-memory es Python — no hay copia posible ni vendorizado |
| Hardcode de instancia (C11) | grep `kb-demo` en `engine/src/` | Solo `lib.rs:71` (config RO), **preexistente** — el diff de la rama a `lib.rs` es una línea (`pub mod escritor;`). Su resolución ya está asignada a M5a en el plan |

**Líneas rojas: ninguna violada.** Sin código AGPL; los permalinks existentes
jamás se tocan (append no reescribe frontmatter — verificado con `xxd`; new no
pisa ficheros); el frontmatter se auto-completa y nunca rechaza (verificado con
cuerpo sin frontmatter y con frontmatter parcial); sin `move`; el engine no
ejecuta git (ningún uso de git en `escritor.rs`/`main.rs`).

## 2. Hallazgos

### Bloqueantes

Ninguno. La pregunta central del gate —¿se puede perder contenido?— la ataqué
por todos los flancos que se me pidieron y alguno más: no encontré ningún
camino de pérdida o corrupción de una nota existente. El no-overwrite es
absoluto, el append es un `O_APPEND` verificado en los bordes, y el tmp+rename
de `new` es atómico en el mismo filesystem.

### No bloqueantes (deuda a arrastrar, por prioridad)

1. **[alta] Título con `/` y traversal con `..`** — reproducido:
   `--titulo "hub personal / portfolio"` crea un **subdirectorio accidental**
   (`projects/hub personal /…`), y `--titulo "../../escapada"` o
   `--dir "../fuera"` **escriben fuera de la KB**. El caso de la barra es real:
   ya existe en la KB (`pguerrero.me — Hub personal / portfolio…`, que
   basic-memory aplanó a `-` en el nombre de fichero). No hay pérdida (el
   no-overwrite se mantiene incluso fuera del árbol), pero sí layout corrupto
   silencioso. **Acción:** en `escribe_nueva`, colapsar `/` del título a `-`
   (como hace la KB real) y rechazar componentes `..` en `titulo`/`dir` con
   exit 1. Ojo: colapsar, no rechazar el título — rechazar violaría la línea
   roja del cierre de sesión. ~10 líneas + 2 tests.
2. **[alta] `--force` en `new` no queda registrado** — `escritor.rs:233`
   hardcodea `forzado: false`; el envelope de un `new --force` que saltó el
   dup-gate dice `forzado: false` y no emite aviso (reproducido). Viola la
   §7.3 de la propia spec ("una vía de excepción sin rastro es peor").
   **Acción:** propagar el flag a `escribe_nueva` o setearlo en `write_new_cmd`.
3. **[media] El rechazo exit 3 no emite envelope con `--json`** — la spec §3.3
   promete `data.dup_candidatas`; en la práctica solo hay una línea humana en
   stderr. El consumidor que quiera las candidatas máquina-legibles no las
   tiene. **Acción:** emitir envelope de rechazo por stdout, o corregir la spec.
4. **[media] §7.2 (aviso de presupuesto) no está implementado** — no existe
   `presupuesto_excedido` ni lectura de `kbx_budget_max` en el write-path
   (grep vacío). La spec lo describe en presente como comportamiento de
   `exo write`. **Acción:** implementarlo o degradar la §7.2 a "pendiente";
   hoy la spec miente en este punto.
5. **[media] Fallback walk+parse no implementado** (spec §3.2 lo afirma) —
   reproducido: con índice rancio + `--crea`, una bitácora existente cuyo
   nombre de fichero no sea slug-clean se **duplica** con el mismo permalink
   en frontmatter (dos ficheros, un permalink). Riesgo práctico hoy bajo:
   las 26 bitácoras de `log/` son slug-clean, y las bitácoras que exo cree
   son autoconsistentes. **Acción:** implementar el fallback o, más barato,
   que `--crea` haga un walk de confirmación antes de crear.
6. **[baja] `--crea` con permalink de 2 segmentos** crea directorio espurio:
   `kb-demo/cosa` → `<kb>/kb-demo/cosa.md` (reproducido). El parsing
   de `write_append_cmd` asume 3 segmentos. **Acción:** exigir forma
   `<proyecto>/<dir>/<slug>` con error claro.
7. **[baja] Flag muerto `--min-similitud` en `write new`** — se acepta y se
   ignora (ningún uso en `write_new_cmd`; grep verificado). Residuo del diseño
   `busca_hybrid` derogado. Su help ("Default: el de config") es falso.
   **Acción:** eliminarlo.
8. **[baja] Divergencia del slug en 3 clases, medida:** basic-memory conserva
   `_` (10 bitácoras rotadas reales: `…2026-06-28_2026-07-05`), separa
   CamelCase (`OpenWisdom` → `open-wisdom`, 4 casos) y translitera `§` → `ss`
   (1 caso); exo colapsa `_` a `-`, no separa CamelCase y come `§`. Solo
   afecta a **notas nuevas** (el permalink queda escrito en frontmatter y se
   honra, así que es autoconsistente), pero mientras basic-memory siga vivo
   conviene vigilar que su watch no renormalice. Relevante para `/consolida`:
   las bitácoras rotadas usan `_` en el título. **Acción:** decidir
   explícitamente (alinear `_` y documentar el resto, o aceptar la divergencia
   por escrito en la spec).
9. **[baja] `SKILL.md` de documenta omite `--db`** en los comandos del Paso 3
   (`exo write append --from <fichero> <permalink>`), que es obligatorio y sin
   default (D6). Tomados literales, fallan con error de clap. **Acción:** añadir
   `--db <db>` a la tabla.

## 3. Disensos con el diseño (no bloquean)

- **El prefijo de proyecto NO sale de la config**, contra lo que dice la spec
  §3.1: sale de `kb.file_name()` (verificado: con `--kb kb-copia` el permalink
  nace `kb-copia/…`). Hoy coinciden (dir `kb-demo`, proyecto
  `kb-demo`); si en M5a la config propia permite nombre de proyecto ≠
  nombre de directorio, esto revienta en silencio. Que M5a lo herede como
  requisito explícito, no como sorpresa.
- El recorte de `replace_section` y el dup-gate por solape de slug en vez de
  retrieval me parecen **correctos y bien fundados**: el argumento del umbral
  (retrieval calibrado para "tráeme contexto", no para "esto ya existe") es
  sólido y la calibración contra el único duplicado histórico + los tres
  no-duplicados reales es exactamente cómo se calibra un guard. Sin disenso,
  lo digo porque lo verifiqué y no solo lo leí.
- `new` normaliza CRLF a LF en el cuerpo (verificado con `xxd`). Aceptable en
  una KB LF-pura, pero es una transformación silenciosa que la spec no declara.

## 4. Nota sobre la cifra de tests

Se afirmaron "108 verdes"; medí 111 en la rama y 98 en main. La discrepancia es
de conteo (probablemente suites ignoradas), no de regresión: 0 rojos en ambos.
