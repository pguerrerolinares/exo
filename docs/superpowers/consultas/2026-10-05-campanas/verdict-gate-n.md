# Verdict GATE — campaña N (`n-sync`), ola 3

- **Consultor**: fable, dispatch fresco (no participé en N ni en su propuesta). Hace también de review final adversarial (no hubo otra; declarado en el ledger 01:43:44).
- **Fecha**: 2026-10-05T01:54:30+02:00 · **Deliverable**: `.worktrees/n-sync`, rama `n-sync` = `497898c`, 5 commits sobre `main` `beecead` (`cd6d81d`, `805643a`, `511cab3`, `fc41aaf`, `497898c`). Solo docs: `docs/backlog.md`, `.superpowers/fabrica/config.md` (624→359), `docs/superpowers/fabrica-historico.md` (nuevo, 396), `docs/superpowers/consultas/2026-10-05-campanas/propuesta.md` (cherry-pick).
- **Régimen**: config §Ejecución de gates, 4 condiciones. (1) fresco: sí. (2) verificación primaria propia: abajo. (3) disenso: §"Qué busqué para objetar". (4) este fichero va en path versionado, en la misma rama; lo commitea el orquestador (brief: el consultor no commitea).

## VEREDICTO: **RECHAZADA** — contenido, no vía. Re-gate barato.

Motivo en una frase: el contrato vivo (`config.md`) apunta dos veces a texto que dice haber movido al histórico y que **no está allí** (diagrama M0..M7 y las dos clases pre-autorizadas de `kbx doctor`), y §Overrides remite a dos gates de calendario que ya no están "arriba". Para una campaña cuyo único entregable es «los docs dicen la verdad», un puntero falso dentro del propio contrato es el fallo de la campaña, no un detalle. Los arreglos son literales, caben en un commit y se verifican con `grep`; los doy abajo con su comprobación.

**Vía cuando se re-gatee**: docs, merge local `--no-ff` por el orquestador (`GATE-EXEC`), sin PR: no toca CI ni código, igual que C1 en la ola 2. Push = Paul.

## Bloqueantes (arreglo exacto + verificación)

**B1. `config.md:31` §Roadmap** dice: «El diagrama M0..M7 original está en `docs/superpowers/fabrica-historico.md`, Apéndice A, y en el spec». Falso: `grep -n 'M0 Fase 0' docs/superpowers/fabrica-historico.md` → rc=1. El bloque de `main:config.md:37-46` («Fuente única: spec §7…» + el diagrama en ```) se **borró**, no se movió.
Arreglo: añadir al histórico un **Apéndice C — «Roadmap original (diagrama M0..M7) y clases pre-autorizadas retiradas el 2026-10-05»** con el texto verbatim de `git show main:.superpowers/fabrica/config.md | sed -n 37,46p`, y cambiar en config «Apéndice A» → «Apéndice C». (Alternativa mínima: retirar la mención a `fabrica-historico.md`/«Apéndice A» entera y dejar solo «está en el spec §7»; pierde la copia, pero deja de mentir.)

**B2. `config.md:240-242` §Clases pre-autorizadas**: «(Retiradas el 2026-10-05 … su texto está en `docs/superpowers/fabrica-historico.md`, Apéndice A.)». Falso: `grep -n 'Backfill mecánico\|Limpieza de root files' docs/superpowers/fabrica-historico.md` → rc=1. Los dos bullets de `main:config.md:499-505` («- Backfill mecánico de `type:`…» hasta «§6.5) — decide el executor sin verdict.») se borraron.
Arreglo: mismo Apéndice C, verbatim de `sed -n 499,505p`; puntero «Apéndice A» → «Apéndice C».

**B3. `config.md:313` §Overrides**: «(incluidos los dos gates de calendario de arriba)». Ya no hay gates de calendario arriba (movidos a histórico Apéndice A, correctamente).
Arreglo: sustituir el paréntesis por «(los dos gates de calendario históricos, `GATE-CALENDARIO-D` y `GATE-HUECO-M2`, viven en `docs/superpowers/fabrica-historico.md`, Apéndice A, y ya no bloquean)». Verificación: `grep -c 'gates de calendario de arriba' .superpowers/fabrica/config.md` → 0.

**B4. `docs/superpowers/fabrica-historico.md:10`**: «Al final, tres apéndices» — hay dos (A y B). Con B1/B2 (Apéndice C) pasa a ser verdad; si se elige la alternativa mínima, corregir a «dos».

Verificación global del re-gate: repetir la comparación de líneas eliminadas de config contra el histórico (lo hice con un script: 364 líneas no vacías eliminadas; 46 sin equivalente verbatim; de esas, **16** son el diagrama con su cierre (`main:config.md:37-46`, 9 no vacías) y las dos clases (`:499-505`, 7) — B1/B2 —; el resto son ediciones legítimas de secciones vivas: `kb-demo`→`wisdom-paul` ×3, `eval.jsonl`→«el eval de M0» ×2, cabecera 09-20 re-rotulada como histórico, ALERTA del `.gitignore` retirada por resuelta —propuesta §2—, kbx fuera del patrón de layout, bullet de línea roja de los gates de calendario, y la reescritura de §Presupuesto, ver NB4).

## Imprecisiones de cita (arreglar en el mismo commit; no bloquean por sí solas)

**C1. 107/145 atribuido a `6bcc85e`** en dos cierres del backlog (N1, `docs/backlog.md:1715`; «Orden dentro del top», `:1874`): «hit@5 107/145 frente a 95/145 de CombMAX». El commit `6bcc85e` dice **107/200 frente a 95/200** (gold J entonces «200 positivas, 40 negativas»); el denominador 145 lo fijó **`441d806`** (2026-10-04, «las nulas fuera del denominador de positivas (145, no 200)»). No es cita inventada: los hits (107 vs 95) y el mecanismo (OR + CombSUM, `buscador.rs:185-201`, `:618`) son los del commit. Es imprecisa de denominador. Arreglo: «107 vs 95 hits sobre el gold J (`6bcc85e`; denominador 145 tras `441d806`)».

**C2. Ítem `[x]` «campaña P» (`docs/backlog.md:2421`)**: «Suites en rc=0 (reconcile 10/10, score 19/19, watchdog 13/13)» son los recuentos **pre-fix** del recon. Re-corridos por mí sobre `n-sync` (= `beecead` + docs): reconcile **12/12**, watchdog **14/14**, score 19/19, los tres rc=0. Arreglo: poner las cifras post-merge.

**C3. Cabecera del backlog (`docs/backlog.md:27-28`)**: «Q (`q-w11`) en curso: no registrada aún.)» — caducó en `497898c`, que registra Q en la fila «Ola 3» y en el ítem del `hook_ms`. Arreglo: «Q (`q-w11`) registrada con gate MERGED-con-condición (PR + CI Windows), no mergeada».

**C4. `wisdom-paul/log/exo-bitacora.md:71`** se cita (`docs/backlog.md:407` y `:2398`) como «decisión de Paul» de cerrar K sin campaña L. La línea 71 es el **Resultado**; la decisión literal está en `:72` («Decisión de Paul: parar aquí; producción intacta») bajo la cabecera `:66` («CERRADA sin campaña L»). Arreglo: citar `:66-72`.

## No bloqueantes (NB), a juicio del orquestador

- **NB1. `verdicts/ola3-q-disable-superpowers.md`** se cita como evidencia en `config.md` (Roadmap Q, lección 2) y `docs/backlog.md` (fila Ola 3). Ese path está **gitignored** (`.gitignore:7` `.superpowers/fabrica/*`): un lector del repo no puede seguir la cita. Hallazgo de **Q**, no de N: si la adjudicación B' fue PENDIENTE-CONSULTOR bajo las 4 condiciones, la condición 4 pide path versionado. Para N basta anotar «(gitignored, intra-rama §6)» o copiarlo a `docs/superpowers/consultas/…`.
- **NB2. `docs/backlog.md` sigue citando `plugins/exo/README.md` (`8c16960`)** y los commits de `o-deuda` como «pendiente de PR»: correcto hoy; si O/Q entran antes que N, N debe quitar los «No mergeado todavía» antes de mergear (orden de merge declarado: N la última).
- **NB3. Oráculos (`config.md:186`): «621 passed, 0 failed, 12 ignored el 2026-10-05»** es la cifra de `a3fcb5f` (propuesta); sobre `beecead` el ledger registra 623. Fechar «sobre `a3fcb5f`» o actualizar.
- **NB4. §Presupuesto perdió una cláusula sin decirlo**: `main:config.md` decía «≤ 20% del cap semanal de Paul, intacto» para la reserva fable. En la rama no aparece (`grep '20%\|20 %'` rc=1). La propuesta mandaba «presupuesto recalibrado», no retirar el cap. Decidir y escribirlo: re-añadir la frase o declararla retirada con motivo. Lo juzgo menor porque la reserva ≤8 y la regla «>50 % ⇒ consolidar» sí se conservan.
- **NB5. `config.md:310`** es una línea de dos espacios (residuo del bullet borrado). Cosmético.
- **NB6. Criterio literal de la propuesta §3 N «config.md sin referencias a kbx»**: quedan 5 (`config.md:7,8,236,357` «patrón kbx» como precedente de fábrica; `:63` D3 «kbx desinstalados»). Ninguna es oráculo ni herramienta viva. **Desviación aceptada** con ese motivo; `kb-demo` y `eval.jsonl` sí quedan a cero (`grep -i 'kb-demo\|eval\.jsonl'` rc=1).

## Hallazgo fuera de N que N debería registrar como `[ ]` nuevo

**El 69/68 de `test-rutas-personales.sh` es una carrera real, y afecta a `test-shellcheck.sh`.** Reproducido: `scripts/_bash-versionado.sh:22` hace `git cat-file -p "$blob" | head -n 1 | grep -Eq …` bajo `set -o pipefail` (los dos gates lo activan). Con `plugins/exo/skills/orchestrate/scripts/review-package` (32.127 bytes, sin extensión, shebang bash) `head` cierra la tubería antes de que `git cat-file` acabe de escribir ⇒ `cat-file` muere con SIGPIPE (`PIPESTATUS: 141 0`) ⇒ `pipefail` da falso al `if` ⇒ el script **se omite**. Medido: 80/100 fallos con `pipefail`, 0/100 sin él; 10/10 corridas del test real dan 68 (y mi primera dio 69: ganó la carrera). Consecuencia: **`test-shellcheck.sh` no lintea `review-package` la mayoría de las veces**, en silencio. Arreglo (fuera de N, 1 línea): consumir el blob entero antes de filtrar, p.ej. `git cat-file -p "$blob" | sed -n 1p | grep -Eq …` (sed sin `q` lee hasta EOF) o `primera=$(git cat-file -p "$blob" | sed -n 1p)`. Oráculo: 20 corridas seguidas con el mismo recuento (69) y `PIPESTATUS` sin 141.

## Verificación primaria (lo que corrí yo)

- `bash scripts/test-docs-vivos.sh` → `[OK]`, rc=0. `bash scripts/test-rutas-personales.sh` → OK, **69** la primera, **68** la segunda y las 10 siguientes (causa arriba), rc=0 siempre. `bash scripts/test-contrato-ci.sh` → 5 passed, 0 failed, rc=0.
- Suites de P sobre la rama: `test-upstream-reconcile.sh` 12/12, `test-upstream-watchdog.sh` 14/14, `test-upstream-score.sh` 19/19, rc=0.
- **Balance `docs/backlog.md`** (col. 0, `^- \[ \]` / `^- \[x\]`): main **20/62** → rama **5/82**, igual que declaró el executor. En cualquier posición 20/66 → 5/86: los 4 `[x]` extra son sub-ítems indentados históricos (líneas 467, 1169, 1174, 1182), ya cerrados en main. Cuadra: 18 flips `[ ]`→`[x]` (14 cierres de N + 4 de O marcados «pendiente de PR») + 2 `[x]` nuevos («Lo que entró 09-22→hoy», «campaña P») + 3 `[ ]` nuevos (dos residuos de O, gate de `upstream-sync`). Los 5 abiertos: backlog:1925 (prosa «coseno», abierto a medias), :2135 (relato de campaña), :2346, :2355, :2365.
- **Cherry-pick de la propuesta**: `git diff 1c7fc23 cd6d81d -- …/propuesta.md` → 0 líneas. Idéntico.
- **Movimiento al histórico**: los 5 bloques `ACTUALIZACIÓN` (09-19, 09-15, 09-14, 09-13, 08-17), «Estado a fecha de redacción original», `GATE-CALENDARIO-D`, `GATE-HUECO-M2` y las secciones Lanes/Oráculos/Corpus originales están **verbatim** (comparación línea a línea; solo las cabeceras `##`→`###` de Apéndice B). Lo que falta es B1/B2.

## Muestreo de citas (≥8; todas verificadas salvo las imprecisiones C1-C4)

| # | Cita en la rama | Comprobación | Resultado |
|---|---|---|---|
| 1 | `d33df4e` = merge PR #35 (recall-inject borrado); `1e19bd0` barrido | `git log -1`: «Merge pull request #35 … spec-recorte-mecanismo»; «docs(recorte): barrido … tras borrar recall-inject» | OK |
| 2 | `b3488df` podó `_hook-ms.sh`, `a1-gate.sh`, `reflex-baseline.sh`; `hooks.json` con un solo script bajo `Bash` | `git show --stat b3488df`: borra los 3 (+ sus tests, 1.108 líneas); `plugins/exo/hooks/hooks.json`: matcher `Bash` → solo `bash-guards.sh` | OK |
| 3 | `6bcc85e` B1+B2, `buscador.rs:185-201` (` OR `), `:618` (CombSUM) | código en la rama: `prepara_query … .join(" OR ")` en 196-201; doc-comment CombSUM en 617-620 | OK en mecanismo; **C1** en denominador |
| 4 | `exo-bitacora.md:114` «La abstención por umbral se descarta: rinde poco (23/95 nulas sin perder hits) y el agente decide sobre la lista L0» | `sed -n 114p` | OK, literal |
| 5 | `exo-bitacora.md:71` K cerrada sin L por decisión de Paul; `b94ed74`; `evals/ablacion-k/verdict-etapa1.md` R2 DAÑO | :66 cabecera «CERRADA sin campaña L», :71 resultado, :72 decisión; commit «eval(k): verdict de la etapa 1 — R1 NO CONCLUYENTE, R2 DAÑO (una tarea)»; verdict :59 «R2 = DAÑO» | OK; **C4** (línea) |
| 6 | PRs #38 `308d1d5`, #39 `6d888a1`, #40 `669e518`, #41 `ceaac09`, #42 `8cd3bd4`, #43 `603cf4a`, #44 `ca77623`, #45 `a3fcb5f`; `59ad826` release 0.3.0 | `git log -1` de cada uno: número de PR y rama coinciden | OK (9/9) |
| 7 | `o-deuda`: `789ac19` primer_trozo, `49e1a0e` kb_sintetica 0,28→0,393, `31e1d31` MAX_CHARS, `3949b2a` suite Python en static-checks, `4e315fc` prosa H28, `cbdd221` `_timeout.sh`; `0c0d2a4` verdict O | existen, mensajes coinciden; `git branch --contains 789ac19` → solo `o-deuda` (no en `main`: «pendiente de PR» es cierto); `0c0d2a4` «gate de la campaña O — MERGED por PR (consultor fable)» | OK |
| 8 | `q-w11` `8c16960` hook_ms en README | «docs(plugin): hook_ms p95 W11 y paso de desactivar superpowers; plugin 1.5.14» | OK |
| 9 | `5b4ca55` U1 = upstream #1943 portada | «port(upstream#1943): workspace por plan…» | OK |
| 10 | `docs/upstream/ledger.md:133` #2258/plan `no aplica` D8; `:54` regla de review independiente del inline → orchestrate | ambas líneas literales | OK |
| 11 | `buscador.rs:106` `SELECT count(*) FROM trozos` sin `tabla_existe`; `main.rs:258,304` nombran `similitud_desde_l2`; `inicia.rs:177` remite a §3.4 | las tres líneas | OK |
| 12 | `_timeout.sh:4-5` en `o-deuda` «Git Bash sin coreutils» | `git show o-deuda:plugins/exo/scripts/_timeout.sh` | OK |
| 13 | D3 «`which kbx basic-memory` → not found» | ejecutado: ambos not found; nada en `~/.local/bin` | OK |
| 14 | KB-exo:45 «`claude plugin disable superpowers` (avisando, nunca uninstall)»; BIT:48 | `sed -n 45p`, `48p` | OK, literal |
| 15 | `c5c5b7f` README en dos frases (proceso vs producto) | «docs(readme): qué problema resuelve y ejemplo…» (campaña B, H6) | OK |
| 16 | Decisiones D0-D7 con cita literal en `config.md` | ledger «SESIÓN OLA 3» 00:35:27 («D0 lanza P ya»), 00:36:46 («D1 entiérrala con tag, D2 cerrar, D6 borra ramas»), 00:50:47 («d3, hecho» · «d4, ok» · «d5 retíralo» · «d7 ok»), 00:56:30 («completa las campañas en sesiones nuevas…») | OK, sin inventos; D5 correctamente marcada «ítem de la KB, no aplica al config» |
| 17 | Notas KB citadas en §Fuentes: `[[desarrollo-agentico]]`, `[[doctrina-agentes]]` | `learnings/desarrollo-agentico.md`, `core/doctrina-agentes.md` existen | OK |

Ficheros citados por `config.md` nuevo que comprobé que existen: `evals/retrieval-fase0/{gate.md,verdict/m0-verdict.md,verdict/labels.md}`, spec 2026-07-16, informes consultores 2026-07-16, `evals/retrieval-heldout/harness`, `scripts/upstream-score.sh`, `scripts/test-plugin.sh`, `.github/workflows/{ci,release}.yml`, `docs/upstream/ledger.md`, `plugins/exo/scripts/`. `evals/retrieval-fase0/eval.jsonl` no existe y ya no se cita (bien).

## Juicio del punto 3 del brief: ¿el movimiento extra retira alguna regla viva?

**No.** Lo movido además de los bloques `ACTUALIZACIÓN`: (a) «Estado a fecha de redacción original» — ya rotulado «histórico» en main; (b) `GATE-CALENDARIO-D` — **CERRADO 2026-08-02** en el propio texto; (c) `GATE-HUECO-M2` — M2 (E1 read) cerró en C5, el gate no bloquea nada vivo; (d) las dos clases pre-autorizadas de `kbx doctor` — kbx desinstalado por D3, inaplicables; (e) el bullet de línea roja «un consultor NO puede adelantar GATE-CALENDARIO-D/GATE-HUECO-M2» — muere con sus gates. Lo que SÍ hizo el movimiento fue dejar huérfano el paréntesis de §Overrides (B3) y los dos punteros falsos (B1/B2). **§Ejecución de gates, §Overrides (salvo el paréntesis), §Reapertura y §Instrumentación tienen la letra intacta** respecto a main, con solo tres ediciones de nombre (`eval.jsonl`→«el eval de M0», `kb-demo`→`wisdom-paul` ×2); la línea roja (push/tag/release/permisos = Paul) está íntegra. Como el régimen no cambia de letra, este gate y los siguientes pueden ejecutarse bajo el mismo contrato sin conflicto.

Lanes/Oráculos/Corpus nuevos frente a los retirados (Apéndice B): todo lo retirado es de la era M0/M1a/M2 (kbx doctor, `make check` de kbx, harness de M0, `eval.jsonl`); lo único no muerto —«skills de process: sin oráculo mecánico, checklist de paridad verificado por el consultor», «corpus write-path sin construir (spec §4.2)», «superficies irreversibles pasan por gates»— **está** en las secciones nuevas.

## Qué busqué para objetar (mandato de disenso)

1. **Citas inventadas**: 22 hashes + 11 referencias fichero:línea + 4 líneas de KB. Ninguna inventada; una imprecisa de denominador (C1), una de línea (C4), una de recuento (C2).
2. **Decisiones de Paul reescritas o ampliadas**: comparé cada D0-D7 del config y del backlog con la cita del ledger. Ninguna ampliada; D5 bien acotada. El backlog convierte D2 en cierre de H28 exactamente como el ledger (00:36:46) lo prescribe.
3. **Cierres por «criterio de N» sin dueño**: «Proceso frente a producto» (`docs/backlog.md:2104`) se cierra como «no es deuda» y lo **declara** («Criterio de N, no de Paul: reabrir si se quiere»); «Sinergias» y «Residuos de entorno» se cierran por su propio texto. La propuesta §2 (criterio escrito) los lista. Aceptable: declarado, no escondido.
4. **Bloques resumidos en vez de movidos**: comparación línea a línea de las 364 eliminadas. Verbatim salvo B1/B2 (perdidos) y ediciones legítimas de secciones vivas.
5. **Reglas vivas perdidas en config**: encontré la cláusula del 20 % (NB4) y los tres punteros/referencias (B1-B3). Línea roja, gates, reapertura, instrumentación: íntegros.
6. **Referencias rotas**: `kb-demo`, `eval.jsonl`: cero. `kbx`: 5 inertes (NB6). Punteros al histórico: dos falsos (B1/B2). Path gitignored citado como evidencia (NB1).
7. **Balance de ítems**: recontado por mí, cuadra (arriba).
8. **Oráculos**: los tres rc=0 y re-corrí las suites de P. Investigué la no-determinación 69/68 hasta la causa raíz (SIGPIPE + pipefail) en vez de aceptarla como ruido: es un hallazgo real para `test-shellcheck.sh`.
9. **Cherry-pick alterado**: diff 0.
10. **Contradicciones internas**: cabecera del backlog contra `497898c` (C3); «tres apéndices» contra dos (B4); «621» contra 623 (NB3).

Lo que salió limpio pesa más que lo que no: la sincronía del backlog es honesta y las decisiones de Paul están citadas sin inventar. Rechazo por B1-B4 porque el propio contrato afirma algo que su histórico desmiente, y ese tipo de deriva es exactamente lo que N existía para eliminar.

## Para el re-gate

Un commit en `n-sync` con B1-B4 + C1-C4 (y NB4/NB5 si se quiere). Verificación mecánica: los `grep` de arriba más la comparación de eliminadas-vs-histórico (deben quedar 46−16 = 30 líneas sin equivalente, todas ediciones de secciones vivas; si la cifra no cuadra, manda el criterio, no el número), `test-docs-vivos.sh` rc=0. A juicio del orquestador, puede hacerlo este mismo consultor por `SendMessage` (contexto intacto, no participó en los fixes; condición 1 se sostiene) sin un dispatch fable nuevo; la línea `GATE:` del package la appendea el orquestador con el path de este verdict.
