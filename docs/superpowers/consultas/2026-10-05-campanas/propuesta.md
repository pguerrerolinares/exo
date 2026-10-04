# Propuesta de campañas N→R — ola 3: cerrar frentes de exo

- **Fecha**: 2026-10-05 · **Base**: `origin/main` @ `a3fcb5f` (PR #45 incluido; el `main` local está 3 commits detrás).
- **Método**: recon de vigencia contra el código, no contra el backlog (lección 3 de la ola 2, `config.md` bloque 2026-09-20). Fueron tres recon en paralelo: KB, specs/planes desde el 09-22, y `docs/backlog.md` + `pendiente-paul.md` + `config.md` contra el árbol.
- **Oráculo base hoy**: `cargo test --release --locked` en `engine/` ⇒ **exit 0**, 621 passed, 0 failed, 12 ignored (~40 s). Los 15 `scripts/test-*.sh` existen; no se ejecutaron en el recon.
- Citas: `backlog:N` = `docs/backlog.md:N` en `a3fcb5f` · `KB-exo:N` = `wisdom-paul/backlog/Backlog — exo.md:N` · `BIT:N` = `wisdom-paul/log/exo-bitacora.md:N`.

## 0. Urgente, antes que nada

**El watchdog de `upstream-sync` se pone rojo el 2026-10-08.** El último latido fue el 09-30 (issue #30), el watchdog caduca a los 8 días y la routine sigue DESACTIVADA (KB-exo:44). Hay dos salidas: puntuar y activar antes (campaña P) o aceptar el rojo de forma consciente.

## 1. Resumen

| Letra | Campaña | Lane | Bloqueo |
|---|---|---|---|
| **N** | Sincronía: `docs/backlog.md`, `pendiente-paul.md` y `config.md` de fábrica dicen la verdad | mecánica (docs) | decisión D1 (techos vivos o retirados); va **la última** |
| **O** | Deuda menor viva con criterio citable (engine, evals, CI, restos de macOS) | mecánica | ninguno; el job de CI va por PR |
| **P** | `upstream-sync`: deuda del bot + primera pasada puntuada contra su gate | mecánica + PAUL-STEP | la corrida de la routine y su activación son de Paul |
| **Q** | Shipeo a W11: `install.ps1` + `doctor` + aviso de superpowers + `hook_ms` publicado | mecánica, oráculo = CI Windows | la prueba en W11 y `plugin disable superpowers` son de Paul |
| R | `retirar-limites`: integrar o enterrar | **condicional a D1** | Paul |

**Orden**: O ∥ P ∥ Q en paralelo, con intersección de ficheros vacía salvo `plugin.json` (bump: se re-bumpea en orden de merge). Después R, si Paul la revive. **N cierra la ola** y absorbe los cierres de las demás. Mismo patrón que C1 en la ola 2: `docs/backlog.md` solo lo toca N.

Por qué esta forma: la instrumentación de la ola 2 dice que **la fábrica rinde en lane mecánica y deja de ser fábrica en lane de diseño experimental**. Esa ola tuvo 11 decisiones de Paul en sesión, casi todas en el Lane A (informe de cierre de la ola 2, en el ledger). Por eso esta ola es **mecánica entera**. Lo de diseño va al §4, como pre-campaña con Paul.

## 2. Obsoleto: se cierra, no se trabaja

Cada ítem se cierra en N con la evidencia citada. Ninguno abre trabajo.

| Ítem | Por qué se cierra | Evidencia |
|---|---|---|
| N1 «FTS hace AND, recall es vector puro» (backlog:1635) | hecho | `6bcc85e` (B1 FTS en OR + B2 CombSUM), `buscador.rs:188-201` |
| «Orden dentro del top: vector/RRF superan a la fusión sellada» (backlog:1794) | hecho, con validación orientativa y no pre-registrada | `6bcc85e`: CombSUM sustituye a `max(v, β·f)` |
| Abstención real, «el corpus negativo casi entero devuelve top-5» (backlog:1807) | **descartada por decisión**: la abstención por umbral «rinde poco (23/95 nulas sin perder hits) y el agente decide sobre la lista L0» | BIT:114 |
| Diagnóstico por fila de los 6/55 (backlog:1817) | datos in-sample de otro binario, anteriores a B1/B2 | `6bcc85e` |
| K R2=DAÑO, «abstención de `recall-inject`» (backlog:374) | `recall-inject` borrado | PR #35, `1e19bd0` |
| «Coste del hook completo en Windows no medido» (backlog:873) | cita scripts que ya no existen; `hooks.json` tiene hoy un único `bash-guards.sh` | `b3488df`, `1e19bd0` |
| Proceso residente **para el recall por prompt** (backlog:1553) | sin hook por prompt, el coste fijo solo se paga en SessionStart. `exo serve` sigue vivo como brainstorm aparte (§4) | `1e19bd0` |
| «Residuos de entorno del plan» (backlog:2091) | `reflex-baseline.sh` borrado; lo demás es higiene local sin criterio | `b3488df` |
| «Sinergias, dueño sin decidir» (backlog:2136) · «Proceso frente a producto» (backlog:1961) | los dos se declaran «no es deuda» o «se retira» en su propio texto | — |
| `bench.sh` `mide()` no comprueba hyperfine (backlog:1881) · `juez.py` `OSError` (backlog:1865) | instrumentos de frentes cerrados (`recall-coste` histórico, J juzgado). Se cierran como *won't fix* con el motivo | — |
| Diagrama M0..M7 de `config.md:36-47` | M0-M4 y M6 hechos, M5a decidido no construir. Solo queda vivo M5b (D3) | backlog «M5a no se construye» |
| Oráculos de `config.md:441-475` | `kbx doctor --kb …/kb-demo` (kb-demo no existe, kbx retirado), `make check` de kbx, `evals/retrieval-fase0/eval.jsonl` (no existe) | `ls` |
| ALERTA del `.gitignore` en `config.md:10-15` | resuelta: la excepción `!.superpowers/fabrica/config.md` existe | `.gitignore` |
| `pendiente-paul.md`: bloques C (D4/D5/D6), E/D (ya RESUELTAS), J (juzgado y congelado en `a936770`), L (padres cerrados), «pushear wisdom-paul» (hecho) | históricos | ledger ola 2 |
| Campaña K | cerrada sin campaña L, por decisión de Paul | BIT:71, `evals/ablacion-k/verdict-etapa1.md` |
| Worktree `.worktrees/campana-k` y ~27 ramas locales ya mergeadas | basura: ver D6 | `git branch --merged` |

## 3. Campañas

### N — Sincronía (mecánica, docs, la última)
- **Alcance**:
  - (1) `docs/backlog.md`: cerrar todo lo del §2 con su cita, registrar lo que entró desde el 09-22 y no consta (K, recorte, B1/B2/B3, CI podado, sin macOS, engine 0.3.0, PR #45) y absorber los cierres de O/P/Q/R.
  - (2) `pendiente-paul.md`: archivar los bloques históricos del §2 con una línea de cierre cada uno, sin purgar (prohibición del skill).
  - (3) `config.md`: roadmap nuevo, que sustituye a M0..M7 por la tabla de esta ola; oráculos vivos (`cargo test --release --locked`, `scripts/test-*.sh`, `plugins/exo/scripts/test-*.sh`, CI `ci.yml` + `release.yml`); fuera kbx y kb-demo; presupuesto recalibrado con la desviación D3 de la ola 2 (30 fable reales contra 8); los bloques ACTUALIZACIÓN anteriores al 09-20 se mueven a `docs/superpowers/fabrica-historico.md` como bloques enteros, sin re-resumir.
- **Oráculo**: `scripts/test-docs-vivos.sh` y `scripts/test-rutas-personales.sh` en rc=0, más un balance de ítems abiertos/cerrados que cuadre (el patrón de C1).
- **Criterio de cierre**: cero ítems `[ ]` del backlog que el recon de vigencia clasifique como (a) o (c); `config.md` sin referencias a kbx, kb-demo ni `eval.jsonl`.
- **Fuera**: la KB. Los desfases de la KB (la puerta del Backlog, que aún habla de v0.2.0 y de 14 commits por pushear; CAN:52) se corrigen con `/document` al cerrar la sesión, no en esta rama.

### O — Deuda menor viva (mecánica)
Cada ítem lleva su criterio citable del backlog:

| Ítem | Criterio |
|---|---|
| `exo recall --json` sin tabla `trozos` (backlog:2211) | test que reproduce `no such table: trozos` y pasa a degradar con aviso. `recall.rs:613-621` `primer_trozo` sin `tabla_existe` |
| Prosa «coseno» y «0.35 por defecto» (backlog:1846) | `main.rs:1023` y `tests/buscador.rs:217,347,382` dicen 0.40 en escala L2 sellada (H28) |
| `kb_sintetica.rs`, escala mezclada en el doc-comment (backlog:1687) | el doc-comment dice ≈0,28 donde hoy dice ≈0,393 |
| Suite Python de J (82 tests) fuera de CI (backlog:1723) | job de CI verde que la corre. **Toca CI ⇒ va por PR** (lección 1 de la ola 2) |
| `MAX_CHARS` no cubre `paquete()→texto` (backlog:1709) | una aserción nueva en `test_gold_j.py` |
| Restos de macOS: fallback a perl en `plugins/exo/scripts/_timeout.sh:15-28`, comentarios de `test-shellcheck.sh` | **recon previo**: comprobar que Git Bash en W11 trae `timeout` antes de quitar el fallback. Si no lo trae, se queda y se re-comenta como de Windows |

- **Oráculo**: `cargo test --release --locked`, `scripts/test-*.sh` y CI en verde.
- **No toca el ranking**: ningún cambio en `buscador.rs` fuera de comentarios.

### P — `upstream-sync`: de mergeado a activo (mecánica + PAUL-STEP)
- **Alcance de la fábrica**:
  - (1) Deuda del bot: un revert de porte sigue marcando `portado`, y U1 (`task-brief` escribe en la raíz plana) sigue vivo (KB-exo:44). Cada uno con test en `scripts/test-upstream-reconcile.sh`.
  - (2) Dejar lista la pasada de evaluación: `scripts/upstream-score.sh` contra `verdad-v6.1.1-v6.4.2.md`.
- **Gate escrito** (KB-exo:44): «0 ya-cubierto falsos y ≥90 %». Si pasa, se activa la routine (Paul). Si no pasa, el residuo va documentado y la routine sigue apagada.
- **PAUL-STEP**: disparar la routine (`/schedule`) para la primera pasada y activarla. Responder las 16 dudas de mapeo del issue #30, 3 de ellas ya decididas en doctrina pero sin marcar `respondida` (#2077, #2063, #1934).
- **Duda abierta**: no sé si la pasada de evaluación puede correr en local o exige la routine en la nube. Lo resuelve el recon pre-flight de la campaña.

### Q — Shipeo a W11 (mecánica, oráculo = CI Windows)
- **Alcance**:
  - `install.ps1`: `jq.exe` real, PATH y `exo init`.
  - Check en `exo doctor` que detecte superpowers habilitado a la vez que exo.
  - Aviso en SessionStart.
  - Publicar `hook_ms` p95 = 1671 ms en `plugins/exo/README.md`, con su fecha y su máquina (KB-exo:48).
  - Decidido: «A, exo entero, split retirado» (KB-exo:45).
- **Oráculo**: `scripts/test-install.ps1` y el job Windows de CI. **Merge por PR, nunca por push directo** (lección 1 de la ola 2: el CI del SO que no se puede probar en local es parte del fix).
- **PAUL-STEP**: instalar en W11, `claude plugin disable superpowers`, y la sonda «¿cargan los mods en W11?» (va de paso, en la misma sesión).

### R — `retirar-limites` (condicional a D1)
Es una rama local, nunca mergeada y ausente de la KB:
- 17 commits del 09-23 que retiran `exo ratchet`, `budget` y `rotate`, los techos por tier y `kbx_budget_max`, y convierten el pre-commit de la KB en gate de higiene (`exo lint --no-index`).
- Contradice el core-index de hoy («trinquete: solo baja», «prefijo kbx histórico pero VIVO»).
- Tiene conflicto en `plugin.json`: la rama dice 1.3.1 y `main` va por 1.5.12.
- Arrastra los 3 commits de `spec-search-por-trozo`, que **solapan con B3** (recall L0 con fragmento, `c7f55d9`).
- **Si se revive**: (1) separar search-por-trozo y re-specearlo contra B3 (lane diseño, fuera de esta ola); (2) rebase de la parte de límites, re-bump y migrar el core-index y las skills; (3) es una ruptura de contrato con la KB viva, así que va con `exo doctor` sobre una copia de la KB como oráculo.
- **Si se entierra**: tag `archivo/retirar-limites` y borrar las dos ramas.

## 4. Fuera de la fábrica: pre-campaña con Paul

Ninguno tiene criterio de cierre escrito. Todos piden brainstorm (`exo:brainstorm`) antes de poder ser campaña.

- **`techo-reglas-2`, «última bala»** (KB-exo:38-42). El gate ya está escrito («≥6/10, g0-33 tiene que dar la vuelta, sin tercera bala»), pero el pre-registro nuevo no existe. La nota del 10-05 (`prompt.compose` como brazo con autoridad de system prompt, BIT:111-118) cambia el diseño del brazo. Es la siguiente candidata a lane diseño **cuando haya pre-registro firmado**.
- **platform.claude.com como fuente** (core-index: «siguiente»), **mods de Claude Code** y **`exo serve`** (proceso residente para SessionStart, prior art tgrep). Son un mismo brainstorm o tres; lo decides tú.
- **Serie de medición A+** (KB-exo:34-37). No pide campaña: **las ramas de esta ola pueden alimentarla** registrando sus métricas en el ledger. Coste marginal.
- **Tasa de re-explicación**: aparcada; necesita que etiquetes 436 prompts (~40 min).
- `search-first`: leer la tasa `ok/(ok+aviso)` en `~/.claude/reflex-log.jsonl`. Es lectura, no campaña.

## 5. Decisiones para Paul

- **D0 (antes del 10-08)**: ¿se lanza P ya para intentar activar antes de que el watchdog se ponga rojo, o se acepta el rojo?
- **D1**: ¿se revive `retirar-limites` (campaña R) o se entierra con tag? Decide qué escribe N sobre techos y trinquete.
- **D2, H28 conversión real L2→L2² (backlog:1599)**: hay dos caminos.
  - (a) **Cerrar declarando** que 0,40 es un umbral en escala L2 sellada y no se convierte. Lo que quedaba con valor (que el código no mintiera) lo cubrió B2 de la ola 2.
  - (b) Abrirla: exige un held-out nuevo, porque el gold J se consumió como «orientativo» en B1/B2. Coste L.
  - Recomiendo (a).
- **D3, M5b**: ¿desinstalar basic-memory (y el binario kbx de `~/.local/bin`) o sacarlo del roadmap? Es línea roja, y es tuya.
- **D4, downrank de `archive/`** (backlog:2099): cerrar como «se queda indexado» o abrir un eval. No hay evidencia desde hace un mes. Recomiendo cerrar.
- **D5, «quitar el memory packet»** (KB-exo:43) choca con tu `~/.claude/CLAUDE.md` y con el contrato de memoria v2. ¿Se retira el ítem o se cambia el contrato?
- **D6**: ¿se borran las ~27 ramas locales ya mergeadas y el worktree de `campana-k`? `archivo/main-pre-reescritura` y `exp/c-solape` se conservan.
- **D7, régimen**: ¿gate asíncrono y tamaño como en la ola 2?
