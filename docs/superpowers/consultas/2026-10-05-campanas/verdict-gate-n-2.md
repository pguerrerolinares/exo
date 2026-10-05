# Verdict GATE (re-gate) — campaña N (`n-sync`), ola 3

- **Consultor**: fable, dispatch fresco (no participé en N, ni en su propuesta, ni en el gate anterior, ni en el commit de fixes). Brifeado solo con el deliverable, el verdict anterior y el criterio.
- **Fecha**: 2026-10-05T02:07:48+02:00 · **Deliverable**: `.worktrees/n-sync`, rama `n-sync` = `0d9e4ef`, 7 commits sobre `main` `beecead` (los 5 del gate anterior + `eceb4e2` verdict RECHAZADA + `0d9e4ef` fixes). Worktree limpio (`git status --short` vacío). Solo docs: `docs/backlog.md`, `.superpowers/fabrica/config.md` (361 líneas), `docs/superpowers/fabrica-historico.md` (425), más `propuesta.md` y `verdict-gate-n.md`.
- **Régimen**: `main:.superpowers/fabrica/config.md` §Ejecución de gates, 4 condiciones. (1) fresco: sí. (2) verificación primaria propia: toda la evidencia de abajo la corrí yo (comandos y rc). (3) disenso: §"Qué busqué para objetar". (4) este fichero va en path versionado de la misma rama; lo commitea el orquestador antes del `GATE-EXEC` (brief: el consultor no commitea).

## VEREDICTO: **MERGED**

Los cuatro bloqueantes (B1-B4) y las cuatro imprecisiones de cita (C1-C4) del verdict anterior están arreglados de verdad, no de palabra: los dos bloques «movidos» están ahora en el histórico byte-idénticos al `main`, los punteros apuntan a donde está el texto, las citas dicen lo que la fuente dice. El commit de fixes no toca nada fuera de eso salvo registrar un `[ ]` nuevo (hallazgo del gate anterior) y cuadrar los recuentos de cabecera. Las reglas operativas vivas del contrato están íntegras respecto a `main`. Oráculos rc=0.

**Vía**: docs, merge local `--no-ff` por el orquestador (`GATE-EXEC`: borra `ACTIVE`, merge, re-corre `test-docs-vivos.sh`, re-arma `ACTIVE`), sin PR. Motivo citado: no toca código ni CI (`git diff --stat main...HEAD` = 5 ficheros `.md`), mismo criterio que C1 en la ola 2 y que el verdict anterior. **Push = Paul** (línea roja, config:299-308). Orden de merge: N la última de la ola (NB2 del gate anterior sigue vigente: `o-deuda` y `q-w11` NO están en `main` — `git merge-base --is-ancestor` → not merged para ambas — así que los «pendiente de PR» del backlog siguen siendo verdad hoy).

## 1. Bloqueantes B1-B4 — verificados uno a uno

| # | Qué pedía el verdict anterior | Comprobación que hice | Resultado |
|---|---|---|---|
| B1 | Apéndice C con el diagrama M0..M7 **verbatim** de `main:config.md:37-46`; puntero `config.md:31` «Apéndice A» → «Apéndice C» | `git show main:.superpowers/fabrica/config.md \| sed -n 37,46p` contra el bloque bajo `### Roadmap original (líneas 37-46)` del histórico (`:402-413`), `cat -A` para ver bytes, `diff` ignorando líneas vacías | **IDENTICAL** (byte a byte, incluidos los box-drawing). `config.md:31` dice ahora «Apéndice C». `grep -n 'M0 Fase 0' fabrica-historico.md` → `:407` |
| B2 | Mismo Apéndice C con las dos clases pre-autorizadas de `kbx doctor`, verbatim de `main:config.md:499-505`; puntero `config.md:242` → «Apéndice C» | `sed -n 499,505p` de main contra el bloque bajo `### Clases pre-autorizadas de kbx doctor (líneas 499-505)` (`:415-425`), misma técnica | **IDENTICAL** (las 7 líneas; el histórico añade una frase introductoria «Retiradas: kbx está desinstalado (D3)…» fuera del bloque, no dentro). `config.md:242` dice «Apéndice C». `grep 'Backfill mecánico\|Limpieza de root files'` → `:419`, `:423` |
| B3 | `config.md:313` §Overrides: quitar «gates de calendario de arriba», remitir al Apéndice A | `grep -c 'gates de calendario de arriba' config.md` → **0**. `config.md:313-315` ahora: «(los dos gates de calendario históricos, `GATE-CALENDARIO-D` y `GATE-HUECO-M2`, viven en `docs/superpowers/fabrica-historico.md`, Apéndice A, y ya no bloquean)» | OK. Y el puntero es verdadero: `## Apéndice A` en hist:241; `### GATE-CALENDARIO-D — CERRADO 2026-08-02` en hist:299; `### GATE-HUECO-M2` en hist:322 |
| B4 | `fabrica-historico.md:10` «tres apéndices» — había dos | `sed -n 10p` → «tres apéndices (A, B y C)»; `grep -n '^## Apéndice'` → `:241` A, `:336` B, `:398` C | OK, ahora es verdad |

## 2. Imprecisiones de cita C1-C4 — verificadas contra la fuente

| # | Cita corregida en la rama | Fuente primaria que leí | Resultado |
|---|---|---|---|
| C1 | `backlog.md:1717-1719` y `:1877`: «107 frente a 95 hits … `6bcc85e`; denominador 145 tras `441d806`» | `git show -s 6bcc85e`: «B1 (OR + CombMAX) 95/200 · B2 (OR + CombSUM) 107/200», «Gold J (200 positivas, 40 negativas)». `git show 441d806`: subject «las nulas fuera del denominador de positivas (145, no 200)», body «Comparación entre brazos intacta (mismo denominador)»; su diff en `buscador.rs` escribe literalmente «hit@5 107/145 frente a 95/145 de CombMAX» | OK. Hits, mecanismo y denominador cada uno atribuido a su commit. `grep '107/145\|95/145' backlog.md` → vacío (no queda la forma vieja) |
| C2 | `backlog.md:2438`: «reconcile 12/12, score 19/19, watchdog 14/14, tras los fixes» | Re-corridos por mí sobre `0d9e4ef`: `test-upstream-reconcile.sh` → `PASS=12 FAIL=0` rc=0; `test-upstream-score.sh` → `PASS=19 FAIL=0` rc=0; `test-upstream-watchdog.sh` → `14 PASS, 0 FAIL` rc=0 | OK, cifras exactas. `grep 'reconcile 10/10\|watchdog 13/13'` → vacío |
| C3 | `backlog.md:28-30`: «Q (`q-w11`) registrada con gate MERGED-con-condición (PR + CI Windows), no mergeada» | `497898c` en el log de la rama («registra la campaña Q (gate por PR)»); `git merge-base --is-ancestor q-w11 main` → not merged | OK, consistente con el estado real. `grep 'no registrada aún'` → vacío |
| C4 | `backlog.md:409` y `:2415`: `wisdom-paul/log/exo-bitacora.md:66-72` | `sed -n 66,72p`: `:66` «## 2026-09-30 — Campaña K: … CERRADA sin campaña L»; `:72` «**Decisión de Paul:** parar aquí; producción intacta» | OK. `grep 'exo-bitacora.md:7'` → vacío (no queda ningún `:71` suelto) |

## 3. El commit de fixes (`0d9e4ef`) no rompe nada más

`git diff 0d9e4ef~1 0d9e4ef` leído entero: 3 ficheros, +62/−14.

- `config.md`: exactamente tres hunks, los de B1, B2 y B3. Nada más.
- `fabrica-historico.md`: el hunk de B4 (línea 10) y el Apéndice C nuevo (`:398-425`), que declara su procedencia («Texto verbatim de `.superpowers/fabrica/config.md` en `main` (`beecead`)») y los rangos de línea correctos (37-46, 499-505 — comprobados arriba).
- `backlog.md`: los hunks de C1 (×2), C3, C4 (×2), C2, más **un ítem `[ ]` nuevo** (`:2385-2398`, «SIGPIPE en `scripts/_bash-versionado.sh:22`», hallazgo del gate anterior) y la cabecera recontada («Abre 4 ítems», «Quedan 6 `[ ]`»).
  - Recuento: `grep -c '^- \[ \]' backlog.md` → **6** (`:1928`, `:2138`, `:2349`, `:2358`, `:2368`, `:2385`); cuatro llevan fecha 2026-10-05. Cuadra con la cabecera.
  - El ítem nuevo cita cosas que comprobé: `_bash-versionado.sh:22` es `git cat-file -p "$blob" | head -n 1 | grep -Eq …` (leído); `review-package` pesa 32.127 bytes (`stat`); los dos gates activan `pipefail` (`test-rutas-personales.sh:19` y `test-shellcheck.sh:54`, ambos `set -uo pipefail`). Ninguna afirmación nueva sin respaldo.

## 4. Muestreo propio de cierres del backlog (distintos de los tabulados en el gate anterior)

Elegí los 8 cierres «Cerrado (campaña N…)» que la tabla del verdict anterior no verificó fila a fila. Para cada uno: ¿la cita existe y dice lo atribuido?

| Línea | Cierre | Cita | Comprobación | Resultado |
|---|---|---|---|---|
| `:1634` | Proceso residente para el recall: obsoleto | `1e19bd0`, PR #35; propuesta §4 nombra `exo serve` como brainstorm aparte | `git log -1 1e19bd0` = «docs(recorte): barrido … tras borrar recall-inject»; `grep 'exo serve' propuesta.md` → `:38`, `:107` | OK |
| `:1675` | H28 conversión L2→L2²: cerrado por D2 | ledger «D2 cerrar»; `28dbb2f`, `d027d23`, merge `063d07c`; `4e315fc` en `o-deuda`; `6bcc85e` | ledger:2067 (00:36:46) cita literal «D1 entiérrala con tag, D2 cerrar, D6 borra ramas» y asigna D2 a N en backlog; `28dbb2f` «vec0 devuelve L2 llana, no L2 al cuadrado (H28)»; `d027d23` «los dos “similitud coseno” vivos que H28 dejó sin corregir»; `063d07c` es merge (padres `ca40d04 d027d23`); `git branch --contains 4e315fc` → `o-deuda`, asunto «prosa caducada de H28» | OK |
| `:1907` | Sin diagnóstico por fila de qué 6 de las 55: obsoleto | `6bcc85e` (B1/B2 cambian el ranking) | el ítem cerrado (`:1896`) es efectivamente «qué 6 de las 55»; `6bcc85e` verificado en C1 | OK |
| `:1969` | `juez.py` timeout/cortes: won't fix | `a936770` (J congelado) | `a936770` «eval(j): congela pre-registro del held-out J … sha256 fijado»; `juez.py` existe en `evals/retrieval-heldout/harness/`; el ítem (`:1954-1961`) habla de `juez.py:383-388` y `:616` | OK |
| `:1989` | `bench.sh` no comprueba rc de hyperfine: won't fix | `b3488df` podó `_hook-ms.sh`/`a1-gate.sh`; `1e19bd0` | `git show --stat b3488df`: borra `_hook-ms.sh` (−68), `a1-gate.sh` (−308), `reflex-baseline.sh` (−54) y sus tests | OK |
| `:2102` | Proceso frente a producto: no es deuda | el propio ítem («un número que se mueve un 25 %…») y `c5c5b7f` | la frase está en el ítem (`:2074-2075`, y el cierre la recita en `:2103`); `c5c5b7f` = «docs(readme): qué problema resuelve y ejemplo de extremo a extremo … (campaña B, H6)» | OK. Declara «Criterio de N, no de Paul: reabrir si se quiere» — honesto |
| `:2209` | `archive/` en el ranking: cerrado por D4 | ledger «d4, ok» | ledger:2083 (00:50:47): «"d4, ok" (`archive/` se queda indexado, sin downrank)» — literal, igual que el cierre | OK |
| `:2269` | Sinergias con delegación de I/O: no es deuda | el propio ítem («sinergia, no como deuda», «ninguna en exo por ahora») | `:2264`: «**Acción:** ninguna en exo por ahora — **esto queda anotado como sinergia, no como deuda**» | OK |

8/8: ninguna cita inventada, ninguna atribución torcida.

## 5. Reglas operativas vivas de `config.md` — comparación con `main`

Extraje cada `## sección` de `main` y de la rama y las diffé:

- **§Reapertura post-cierre**: `diff` → **IDENTICAL**.
- **§Instrumentación**: **IDENTICAL**.
- **§Ejecución de gates** (las 4 condiciones, mecánica de despacho, `GATE-EXEC`, `PENDIENTE-CONSULTOR`, línea roja): difiere SOLO en (a) «sobre `eval.jsonl`» → «sobre el eval de M0»; (b) `kb-demo` → `wisdom-paul` ×2 en la línea roja (push y borrado de notas); (c) retirado el bullet «El `GATE-CALENDARIO-D` y el `GATE-HUECO-M2` de este config — un consultor NO puede autorizar adelantarlos», que muere con sus gates (D cerrado 2026-08-02, M2 cerrada en C5; ambos en hist Apéndice A). Las 4 condiciones y la línea roja (`git push` a cualquier remoto, borrado de notas sin `doctor`, `.claude/settings.json`/guards = SIEMPRE Paul) están con la letra intacta en `config.md:250-308`.
- **§Overrides de Paul**: difiere SOLO en el paréntesis de B3. «ceden SOLO ante pedido directo de Paul en sesión… se registra en el ledger ANTES» intacto.
- **Barrido global** de líneas de `main:config.md` sin equivalente verbatim en `config.md`∪`fabrica-historico.md` de la rama (no vacías, `grep -xF`): **31**. El verdict anterior predijo 30 (46−16) y mandó aplicar el criterio, no el número; lo apliqué: las 31 son (i) la ALERTA del `.gitignore` (6, resuelta: `config.md` está versionado), (ii) las notas `kb-demo/...` renombradas (3), (iii) la cabecera 09-20 re-rotulada como histórico (1), (iv) la reescritura de §Presupuesto (13, ver NB4), (v) «patrón kbx» fuera del layout (2), (vi) `eval.jsonl`/`kb-demo` ×3, (vii) el bullet de los gates de calendario (2), (viii) el paréntesis de B3 (1). **Los bloques B1/B2 ya no aparecen en la lista**: lo que se decía movido está movido.

## 6. Oráculos (corridos por mí en el worktree, rc capturado con `$?` explícito)

| Oráculo | Salida | rc |
|---|---|---|
| `bash scripts/test-docs-vivos.sh` | `[OK] test-docs-vivos: README.md/docs/{arquitectura,instalacion}.md sin frases muertas…` | **0** |
| `bash scripts/test-contrato-ci.sh` | `5 passed, 0 failed` | **0** |
| `bash scripts/test-rutas-personales.sh` | `OK — 69 scripts sin rutas personales` | **0** |
| `test-upstream-{reconcile,score,watchdog}.sh` (para C2) | 12/12 · 19/19 · 14/14 | 0 · 0 · 0 |

(Nota de método: mi primer intento de capturar rc con `PIPESTATUS` bajo zsh salió vacío; lo repetí con `$?` sin pipe. Solo cuenta la segunda corrida.)

## 7. No bloqueantes heredados del gate anterior (estado hoy; a juicio del orquestador)

- **NB1** (`verdicts/ola3-q-disable-superpowers.md` gitignored citado como evidencia): sigue en `config.md:37` y `:90`. Hallazgo de Q, no de N.
- **NB3** («621 passed» en `config.md:186` frente a 623 sobre `beecead`): sigue. Cosmético/fechable.
- **NB4** (cap «≤ 20% del cap semanal de Paul» de §Presupuesto): `grep '20 \?%'` en config y histórico → vacío; no se re-añadió ni se declaró retirado. Mi posición: es un cap caído en silencio de una sección que el brief no nombra entre las reglas vivas, el gate anterior lo dejó al orquestador, y §Instrumentación nunca midió «% del cap semanal de Paul» (cuenta dispatches por filas) — era letra muerta. Recomiendo una línea en §Presupuesto declarándolo retirado con ese motivo, en el siguiente toque al config; no lo convierto en bloqueante en un re-gate cuando el primero no lo fue.
- **NB5** (`config.md:310` línea de dos espacios): sigue. Cosmético.
- **NB6** (5 menciones inertes a `kbx` como precedente): desviación aceptada en el gate anterior; no la reabro.

## Qué busqué para objetar (mandato de disenso)

1. **Que el «verbatim» fuera un resumen o una transcripción con erratas**: comparé con `cat -A` (bytes, no apariencia) y `diff` ignorando vacías contra `git show main:…` directo. Byte-idéntico en ambos bloques. Nada.
2. **Que los punteros arreglados apuntaran a otro sitio vacío** (el fallo exacto del gate anterior): seguí cada uno hasta la línea destino (hist:241/299/322/398/407/419/423). Todos llegan.
3. **Citas inventadas en cierres que nadie había tabulado**: 8 cierres, 11 hashes, 2 decisiones del ledger, 4 auto-citas de ítem, 1 referencia a propuesta. Cero inventadas.
4. **Que el fix de C1 fuera cosmético**: leí el body y el diff de `441d806`, no solo el subject. Confirma 107/145 vs 95/145 literal y «comparación entre brazos intacta».
5. **Que el commit de fixes colara cambios de fondo**: leí el diff entero; lo único no pedido es el `[ ]` nuevo, y cada afirmación de ese ítem la comprobé (pipeline, tamaño, `pipefail` en los dos gates).
6. **Contradicción cabecera/recuento**: 6 `[ ]` reales = 6 declarados; 4 fechados 2026-10-05 = «Abre 4».
7. **Reglas vivas perdidas**: diff por sección contra main; el único cap caído es el 20 % (NB4), ya conocido y no bloqueante.
8. **Que los «pendiente de PR» del backlog hubieran caducado** porque O/Q entraran antes: `merge-base --is-ancestor` → ni `o-deuda` ni `q-w11` están en `main`. Siguen siendo verdad; N debe seguir mergeando la última.
9. **Oráculos con rc fiable**: tres gates + tres suites, todos rc=0 capturado explícitamente.

No encontré nada que sostenga un rechazo. Lo que queda (NB1-NB5) es menor, estaba declarado y no afecta a la verdad del contrato ni del backlog.

## Para el orquestador

Línea a appendear al review-package: `GATE: MERGED (consultor fable, 2026-10-05T02:07:48+02:00, verdict=docs/superpowers/consultas/2026-10-05-campanas/verdict-gate-n-2.md)`. Commitear este fichero en `n-sync` ANTES del `GATE-EXEC`; merge `--no-ff` local; post-merge `bash scripts/test-docs-vivos.sh` rc=0; push = Paul.
