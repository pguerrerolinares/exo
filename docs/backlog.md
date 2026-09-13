# Backlog de exo — deuda abierta

> Nota viva: deuda técnica y documental de exo con su siguiente paso accionable.
> **No duplica el plan de cierre** (`plans/2026-08-17-cierre-exo-m2-a-m5b.md`), que
> fija QUÉ campañas quedan y en qué orden. Esto es lo que está suelto: hallazgos
> de gate sin barrer y deuda encontrada fuera de campaña. Editar aquí, no
> duplicar. Cada item cita su evidencia; un item sin evidencia verificable no
> entra.
>
> Última revisión: **2026-09-13** (cierre de la campaña B —
> `docs/superpowers/plans/2026-09-13-campana-b-superficie-y-gates.md`, H6, H8,
> H9, H11, H12, H13, H15, H16, H18, H20, H21, H22, H26— sincronizado además con
> el CI que cerró fuera de campaña el 09-12/09-10. **Se mueven a `## Cerrado
> con evidencia`**: los `test-*.sh` del plugin fuera de CI (`d8aa3b6`, job
> `plugin-tests`) y `test-contrato-engine.sh` atado a esta máquina (`d8aa3b6`,
> `scripts/test-contrato-ci.sh`); **entran nuevas** el gate de shellcheck (H11,
> `cd3bfff`) y el gate de exec-bit ampliado a scripts sin extensión (H12,
> `4af859b`), los dos con su ciclo rojo-verde medido. **Se cierran in situ,
> con commit** (sin mover de sección porque el item conserva otra mitad
> abierta): el gate de versiones (H16, `3bc05aa`, acción b de «la
> documentación de referencia contradice…»), la convención `tier` por ruta en
> vez de frontmatter (H18, `2294349`, D5=b, «los documentos del repo no llevan
> tier»), el «para quién es exo hoy» del README (H6, `c5c5b7f`, «decisión
> abierta: proceso frente a producto»), el idioma D1=C de la superficie CLI
> (H8/H9, `7853e8a`/`b481189`, «idioma mezclado sin criterio único» y «el
> relato de campaña en los comentarios»), y `reports/` a
> `evals/e1-read/reports/` (H26, `7ee58ba`, «nombres y ubicaciones»). El rojo
> del job `test` sigue **a medias** (`50aee95`: ya nombra el test que falla;
> sigue sin cazar un error de compilación ni subir artifact). **Se pliega** el
> bloque «Hallazgos de hoy planificados para la campaña B» que la campaña A
> había dejado abierto, y las tres líneas sueltas «Planificado en campaña B
> (HXX)» — cada una convertida en su cierre, ninguna queda apilada. **Abre**
> un item nuevo: el mensaje de error de la guarda «una DB sirve a una KB»
> recomienda `--db`, que `exo init` no tiene (usa `$EXO_DB`). H9, H13, H15,
> H20, H21, H22 son código/UX sin item propio en este backlog —ejecutados
> según el plan, sin deuda que registrar aquí—; H25 queda fuera, es checklist
> externo de Paul, no tarea de la fábrica.)
>
> Anterior: **2026-09-11** (alineación con el árbol real tras G5b, a
> raíz de corregir el informe de la revisión crítica
> —`docs/2026-09-04-revision-critica-externa.md`—. La re-verificación que ese
> informe hizo el 09-11 por la mañana se corrió contra `bf4ba7a` **creyendo
> que era `main`**, cuando `main` estaba en `a0d538b` desde el 09-10 11:25:
> cuatro de sus afirmaciones salieron falsas. Aquí se propagan las tres que
> tocan a items —`kbx`, la MSRV y el ratio docs/código— y se re-mide lo que
> había cambiado. **Ningún item se cierra**; dos pierden parte de su
> evidencia, uno gana un hermano nuevo (`tier` en los docs del repo) y la
> release `v0.1.0` entra en `## Cerrado con evidencia`. Los seis marcados
> «lo cierra / lo subsume G5» siguen abiertos: **G5b cerró sin adoptar
> ninguno**.)
>
> Antes: **2026-09-09** (re-verificación de los diez items de la
> revisión crítica externa contra el árbol de `f86167a`: **nueve siguen vivos
> y sin tocar**, uno caducó a medias —la MSRV— y tres traían cifras ya
> movidas. Corregido in situ; los items retocados lo dicen en su cabecera.
> De paso: esta cabecera venía **rota del merge `f86167a`** —el bloque del
> 09-04 perdió su prefijo `> Última revisión:` y su `>`, y se salía del
> blockquote—, arreglada aquí).
>
> Segunda pasada ese mismo día, tras comparar el repo con `affaan-m/ECC`
> (MIT): **ocho cruces anotados, cero items nuevos**. Los dos del truncado
> mudo —«el bloque de arranque va al 96%» y «`inject-emitted` se emite aunque
> no se inyecte nada»— se marcan como la **misma clase de fallo con una sola
> forma de arreglo** (un campo del envelope, no dos avisos). Otros seis —el
> cutover binario↔scripts, los dos de fixtures atadas a esta máquina, los dos
> de endurecimiento del CI y el de `kb-demo` como fixture— quedan marcados
> **«lo cierra / lo subsume G5»**, greppables por esa cadena. La comparativa
> no añadió deuda: reordenó la que ya estaba escrita.
>
> Las afirmaciones sobre ECC se anotaron primero leyendo ficheros sueltos por
> HTTP y se **re-verificaron después contra un clone pineado en
> `5064474d4d762dc9640234a41617cccb79185cec`** (2026-09-07, v2.2.1). De las
> cuatro, una era **falsa** (qué hace su `repair`), dos había que **acotarlas**
> y una se confirmó. Corregidas in situ y con cita `fichero:línea`: sin el
> clone, tres de las cuatro habrían entrado mal.
>
> Tercera pasada del 09-09, tras clonar y leer el fuente de
> `DietrichGebert/ponytail` y `JuliusBrussee/caveman` —las dos herramientas
> virales de 2026 que el equipo usa a diario—: **dos cruces anotados, cero
> items nuevos.** Van al item del truncado mudo (precedente fail-closed con
> handle durable) y al del 48/55 in-sample (base de evidencia por cifra y
> evals de tres brazos). Un candidato a item nuevo —«los nueve hooks son
> never-block por política, sin justificar la dirección hook a hook»— se cayó
> al verificarlo: el único `exit 1` de la cadena de hooks
> (`compose-inject.sh:16-18`) lo absorbe `subagent-inject.sh:35` dentro de un
> `if` con `2>/dev/null`, así que el hook se abstiene y el invariante aguanta.
> Igual que con ECC: leer el README daba tres afirmaciones que el clone
> corrigió —la inyección en Claude Code es `SessionStart`+`SubagentStart`, no
> `UserPromptSubmit`+`PreToolUse`; los adaptadores multi-host no se generan (1
> de 11); y el escalón «inferred→replayed→verified» que la prensa les
> atribuye no existe en el código.
>
> Cuarta pasada del 09-09, esta vez de **coste de tokens** y con medición
> propia sobre los transcripts de esta máquina (549 sesiones, 331 MB):
> **cuatro items nuevos**, greppables por «pasada de coste 2026-09-09». Uno en
> Alta —el bucle de coste de la inyección, que resulta estar a un `join` por
> `session_id` de distancia porque `recall-inject.sh` ya loguea los bytes que
> emite—, dos en Media —la colisión de nombre de `exo budget` y el rediseño de
> M5a contra el spec MCP stateless de 2026-07-28— y uno en Baja que es
> explícitamente **sinergia sin dueño**, no deuda. La medición también
> **cierra una vía**: el hit rate de cache es 97,7% (98,2% en `claude-opus-5`),
> así que no hay margen en afinar caching. Nada de esto duplica items
> existentes: el grep de `cache_read` / `prompt caching` / coste de tokens
> sobre este fichero daba cero antes de la pasada.
>
> Antes: **2026-09-04** (revisión crítica externa del repo completo:
> diez items nuevos marcados «(revisión 2026-09-04)», tres de ellos en Alta;
> ninguno duplica los que ya estaban — `test-*.sh` fuera de CI,
> `test-contrato-engine.sh` atado a esta máquina, aliases españoles y
> `kb-demo` en los tests del engine ya tenían entrada y se dejan como están).
> Y antes: **2026-09-02** (G5a — CI mínimo cerrado con evidencia, deuda nueva
> de la ola anotada).

## Estado

| | |
|---|---|
| **Cerradas** | C5 (M2-08+09, cierra E1 read) · C6 (M6, cutover del recall) · C7 (M4, write-path) |
| **Pendientes** | C8 (M3+M1b, cutover de skills) → C9 (M5a, MCP + config propia) → C10 (M5b, desinstalar basic-memory) |
| **Medido** | engine-hybrid **48/55** hit@5 vs bm-hybrid 39/55, mismo día, paridad de corpus ∅, recall <2s (`evals/e1-read/verdict/m2-09-corrida.md`) |
| **Tests** | 111 verdes / 0 rojos en la rama de M4, 98 en main previo (contados por el consultor del gate en esa ola; el CI que los corre solo llegó después, en G5a — 200 tests / 28 binarios). **El 2026-09-13 (cierre de campaña B), `cargo test --release --locked --no-fail-fast` en `engine/`: 478 tests verdes en 48 binarios, 2 ignorados, 0 rojos** |
| **Release** | `v0.1.0` publicada el **2026-09-11** — tres binarios y sus tres `.sha256`, instalables por `install.sh` / `install.ps1`. Ver `## Cerrado con evidencia` |
| **Campaña A** | ejecutada el **2026-09-13** salvo Tasks 12 y 15 (esperan la medición W11 de Paul) — mergeada a `main` el **2026-09-13** vía PR #13 (`ef5100b`); veredicto en `evals/recall-coste/verdict/2026-09-campana-a.md` |
| **Campaña B** | ejecutada el **2026-09-13**, las 13 tasks (H6, H8, H9, H11, H12, H13, H15, H16, H18, H20, H21, H22, H26) integradas en la rama `campana-b` — **a fecha 2026-09-13, pendiente del gate de Paul, sin PR a `main` aún**; plan en `docs/superpowers/plans/2026-09-13-campana-b-superficie-y-gates.md`; H25 queda como checklist externo de Paul |

---

## Alta

- [ ] **(revisión 2026-09-04) El 48/55 del hybrid es un resultado in-sample:
  los parámetros se eligieron sobre las mismas 55 queries que lo reportan.**
  Evidencia: `engine/src/main.rs:12-25` documenta que `BONUS_SELLADO` y
  `ESCALA_FTS_SELLADA` son los ganadores del sweep de 15 celdas por «max
  hit@5=49/55» sobre `eval.jsonl`, y el umbral 0.40 se fijó por el mismo
  criterio; `arquitectura.md` §6 reporta 48/55 vs 39/55 sobre ese mismo
  fichero. No hay conjunto held-out, no hay intervalo de confianza (n=55) y
  el set es privado, así que la cifra no es reproducible por un tercero. La
  mejora es plausible; lo que no está es la evidencia de que generalice a
  queries que no participaron en la selección. Relacionado: el tamaño de trozo
  (900) y el default `--type fts` de `exo search` (`main.rs:205`,
  re-verificado el 09-09) frente al modo medido (`hybrid` + `--min-similarity
  0.40`).
  **Acción:** (a) redactar y congelar un held-out de queries nuevas ANTES de
  volver a tocar β, bonus, umbral o troceado; (b) reportar in-sample y
  held-out por separado en el próximo verdict; (c) decidir si el default de
  `exo search` pasa a ser el modo medido o si el README deja de presentar el
  48/55 como «lo que hace exo».
  **Cruce (2026-09-09, 2ª fuente):** dos precedentes de forma en
  `JuliusBrussee/caveman` (leído en el árbol clonado, no en su README),
  aplicables a las acciones (a) y (b). Primero, **base de evidencia explícita
  por cifra**: `docs/technical/accounting-and-evidence.md:9-17` tipa siete bases
  (`measured / inferred / provider_reported / benchmark_counterfactual /
  observed / verified / unpriced`) y **prohíbe que el runtime local marque su
  propio output como `verified`** —solo lo computa el servidor—, con
  `basis="inferred"` hardcodeado y el comentario de por qué en
  `packages/sdk/python/caveman_cloud/core.py:144` y
  `engine/ccr/store_sqlite.go:666`. El paralelo es directo: el motor de exo no
  puede firmar su propia calidad de retrieval, y hoy el 48/55 se reporta sin
  base declarada. Segundo, **diseño de tres brazos contra el baseline fácil**:
  sus evals comparan `__baseline__` / `__terse__` / skill (`CLAUDE.md:376-387`)
  justamente para no medirse contra un baseline sin instrucciones, con corpus y
  runner committeados (`evals/`, `benchmarks/`) — la forma que le falta al
  held-out de (a). Calibración de la propia fuente, para no copiar de más:
  `docs/HONEST-NUMBERS.md:31-37` publica los casos de **pérdida neta** con
  cifras (4,3M tokens con la herramienta vs 1M sin). Lo que se copia es
  reportar la fila en rojo, no el número.
  **Planificado en campaña C (H7):** el held-out fuera de muestra ya tiene
  pre-registro — `docs/superpowers/plans/2026-09-13-campana-c-preregistro.md`.

- [ ] **(revisión 2026-09-04) La documentación de referencia contradice el
  repo el mismo día en que se escribió.** Medido el 2026-09-04:
  `docs/arquitectura.md:489` afirma «**Sin CI**: no hay `.github/`» y la
  sección 7 sigue listando la suite como no hermética fuera de la máquina de
  desarrollo, cuando `.github/workflows/ci.yml` existe desde el 2026-09-02
  (`e378cbc`) y el README describe esa misma corrida en tres SO. Segundo
  caso: `.claude-plugin/marketplace.json:4,8` y
  `plugins/exo/.claude-plugin/plugin.json:4` publican `"version": "1.0.0"`
  mientras `engine/Cargo.toml:3` es `0.1.0` y no existe ninguna release. Un
  documento «derivado del código» que se desactualiza en 48 horas indica que
  el volumen documental supera lo que una persona mantiene sincronizado.
  **Re-verificado el 2026-09-11 (tarde): la acción (a) está hecha, la (b) y
  la (c) no, y el item gana dos instancias nuevas.**
  - (a) **hecha**: `3673059` retiró «Sin CI» de `arquitectura.md` el 09-11 a
    las 09:49. `grep "Sin CI" docs/arquitectura.md` → vacío. La afirmación
    hermana (suite no hermética fuera de esta máquina) sigue en `:495` y
    sigue siendo **cierta**, así que no era deriva.
  - (b) **abierta**, y con un número más: `marketplace.json:4` `1.0.0`,
    `marketplace.json:8` y `plugin.json:4` `1.1.1` (`ae1f470`),
    `Cargo.toml:3` `0.1.0`. Ahora además existe una release `v0.1.0`
    publicada, así que el crate y el tag sí concuerdan y el plugin es el que
    se sale.
  - (c) **abierta, y es lo único estructural que queda.** Su alcance ya está
    definido: los cuatro ficheros que deben ser verdad hoy (`README.md`,
    `docs/{instalacion,arquitectura,backlog}.md`, 1.939 líneas), no los 224
    `.md` del repo. Cruza con el item nuevo de `tier` en Media, que es lo que
    hace greppable ese conjunto.
  - **Instancia nueva y ya cerrada el mismo día**: `docs/instalacion.md` §2 y
    §7 afirmaron durante una hora que los instaladores «viven en una rama sin
    mergear» y que «no hay ningún tag publicado», cuando G5b ya estaba en
    `main` (11:27) y el tag existía. `0c81e87` (12:25) los retiró junto con
    los del README. Cierta un día, falsa una hora: es la instancia más corta
    que tiene el repo, y solo se cerró porque retirar los avisos era un paso
    escrito del plan, no porque nada lo comprobara.
  **Acción:** (a) ~~corregir §7 de `arquitectura.md`~~ hecha el 09-11;
  (b) alinear las tres versiones (o documentar por qué el plugin versiona
  aparte del engine); (c) añadir al `verify` de cierre un grep de las
  afirmaciones de estado más frágiles («Sin CI», recuento de tests,
  versiones) contra el árbol real, **acotado a los cuatro ficheros `core`**.
  **(campaña B, 2026-09-13, H16): acción (b) cerrada — no por alineación,
  por decisión D3=a (dos artefactos, dos versiones, cada una con su gate).**
  `scripts/test-versiones.sh` compara `plugin.json` == `marketplace.json` y,
  con un tag, `Cargo.toml` == tag; falla también si `marketplace.json` sigue
  llevando `metadata.version` (un tercer número sin dueño, retirado en el
  mismo commit). Corre en `ci.yml` (job `lint`, `:61`) y en `release.yml`
  (job `publish`, antes de publicar). Verificado en HEAD: `marketplace.json`
  sin `metadata.version`, `plugins[0].version` == `plugin.json version` ==
  `1.1.2`, `Cargo.toml` == `0.1.0`, tag `v0.1.0` — el engine y el plugin ya
  no tienen que compartir número, el gate impide que cada par diverja del
  suyo. Commit `3bc05aa`, documentado en `docs/instalacion.md` §5. Sigue
  abierta la acción (c) (el grep de afirmaciones frágiles en el `verify` de
  cierre, acotado a los cuatro `core`).

- [ ] **(revisión 2026-09-04) «exo genérico» sigue siendo el plugin de Paul
  para Paul.** Medido el 2026-09-04 sobre `plugins/exo/`: la cadena `Paul`
  aparece en 4 ficheros vivos del plugin (`skills/distill/SKILL.md` ×7,
  `scripts/recall-inject.sh` ×2, `scripts/git-add-all-guard.sh`,
  `scripts/kb-precommit.sh`) — **re-medido el 2026-09-13 (`git grep -c Paul
  -- plugins/exo`) tras el split de `distill`: 5 ficheros de lógica
  (`skills/distill/SKILL.md` ×6, `skills/distill/chequeos.md` ×1 —nuevo del
  split—, `scripts/recall-inject.sh` ×2, `scripts/git-add-all-guard.sh` ×1,
  `scripts/kb-precommit.sh` ×1); el grep también da `.claude-plugin/
  plugin.json` ×1, pero es el campo `author.name` (metadata legítima, fuera
  del alcance de esta acción)**; `kb-demo` en 8 ficheros del plugin, dos de ellos
  hooks de producción (`exo-recall.sh`, `recall-inject.sh`) y uno el
  pre-commit de la KB; y `kbx` —binario Go externo, no incluido en el repo,
  sin build decidido en Windows según la propia skill— es dependencia
  operativa de `distill` (11 menciones, pasos que se «saltan» si falta), de
  `document` (`SKILL.md:9,24,82`) y de `agents/executor.md` — re-verificado el
  09-09: `kbx` aparece en **7 ficheros** del plugin, no en los dos que este
  item citaba (los otros: `kb-precommit.sh`, `recall-inject.sh`,
  `test-recall-inject.sh`, `document/routing.md`). Súmese la barrera de
  instalación (`docs/instalacion.md`: Rust ≥1.95, toolchain C, Git Bash, jq,
  descarga de 0,6 GB, sin binario ni `install.sh`). Hoy no hay tercero que
  pueda adoptar el plugin sin leer la documentación entera. Es distinto del
  item de Baja «`kb-demo` como fixture en los tests del engine»: aquí son
  hooks y skills de producción.
  **Re-verificado el 2026-09-11 (tarde): la acción (c) está hecha, la (b) a
  medias, la (a) intacta.**
  - **`kbx`: de 7 ficheros a 5**, y el grueso ya no es dependencia real.
    `kb-precommit.sh` cortó a `exo ratchet --staged` + `exo budget` en el
    cutover de G4c (`e76e9f9`, 09-10, en `main` desde el 09-10 11:25).
    **`agents/executor.md` no contiene la cadena `kbx`**: esa cita del item
    era falsa. De los 5 que quedan, dos son texto de ejemplo y nombre de un
    caso de test; la dependencia viva es `distill` y `document` llamando a
    `kbx` por `rotate` y `stale`, que ya tiene su propio item en Baja
    («(G4c, Task 14) El cutover kbx→exo es parcial»).
  - **(c) hecha**: la release `v0.1.0` está publicada desde el 09-11 12:10,
    con `install.sh` e `install.ps1` resolviendo `releases/latest`. **Cae la
    barrera de instalación** que este item citaba (Rust, toolchain C, 0,6 GB
    y compilar desde fuente ya no son el único camino), y con ella la frase
    «hoy no hay tercero que pueda adoptar el plugin».
  - **Lo que sigue vivo, sin un solo cambio (a fecha 09-11)**: `Paul` en los
    mismos 4 ficheros con los mismos conteos (`distill/SKILL.md` ×7,
    `recall-inject.sh` ×2, `git-add-all-guard.sh`, `kb-precommit.sh`);
    `kb-demo` en 8 ficheros, tres de ellos de producción (`exo-recall.sh`,
    `recall-inject.sh`, `kb-precommit.sh`). El item sigue en Alta por esto.
    **Re-medido el 2026-09-13, tras H22 (`e5398d5`+`1018802`) partir
    `distill/SKILL.md` en `SKILL.md` + `chequeos.md`:** el conteo cambia de
    forma (`git grep -c Paul -- plugins/exo`) pero no de fondo — ver el
    detalle arriba, en la entrada del 2026-09-04.
  **Acción:** (a) sustituir «Paul» por «el usuario»/«el dueño de la KB» y
  `kb-demo` por el nombre resuelto vía `exo config` en los cuatro scripts y
  dos skills; (b) lo que queda de `kbx` (`rotate`, `stale`) lo lleva el item
  de Baja de G4c Task 14: o se porta, o se declara dependencia opcional en
  `instalacion.md` y `distill` se abstiene entera sin él;
  (c) ~~la release con binario de G5~~ **hecha el 2026-09-11**.

- [ ] **El bloque de arranque va al 96% de su cap, y desborda en silencio.**
  Medido el 2026-08-27 al validar la Task 6 de la ola 1B: el bloque que
  `exo-recall.sh` inyecta en cada `SessionStart` ocupa **5.921 B sobre un cap de
  6.144** (`EXO_CAP="${EXO_RECALL_CAP:-6144}"`, `:36`) — **223 B de aire, un
  3,6%**. La doctrina de presupuestos de la propia KB exige **15%** al sellar un
  techo, y llama a lo de estar a ras «un mordisco programado para mañana».
  **El modo de fallo es el peor posible**: la cabecera de `core-index.md` lo
  dice literalmente — «lo que sobra se trunca **en silencio** por el final». No
  hay error, no hay aviso en el log; simplemente el arranque deja de servir el
  final del bloque. Y el final es la cola de «Destilados de proyecto activos»:
  hoy, la entrada de **exo** — el proyecto en curso.
  **Por qué crece solo**: el bloque es `core-index.md` (5.355 B) **más** los
  punteros de actividad reciente, que salen de la actividad git de la KB y por
  tanto **varían solos, sin que nadie edite nada**. Una racha de commits en la
  KB puede empujarlo por encima del cap sin un solo cambio de contenido.
  No lo causó esta ola (aportó 28 B de esos 5.921), pero la ola lo hizo medible.
  **Acción, por orden de coste:** (a) que el truncado **grite** — un aviso por
  stderr y un evento en el log cuando el bloque toca el cap, hoy no hay ninguno;
  (b) pasada de `/distill` sobre `core-index` retirando entradas muertas (es
  índice: se retiran entradas, no se comprimen las vivas) — la propia entrada de
  exo está rancia, sigue diciendo «Frente: C10/M5a-02 config propia», que se
  cerró hoy; (c) revisar si el cap de 6.144 sigue siendo el correcto.
  **Cruce (2026-09-09):** misma clase de fallo que «`inject-emitted` se emite
  aunque no se inyecte nada», justo abajo — el instrumento no reporta lo que no
  hizo. Las dos acciones convergen en **un campo del envelope** (truncado /
  vacío) que el hook lea y loguee, en vez de dos avisos ad hoc por separado;
  el envelope ya existe (`schema_version == 2`). Precedente de forma verificado
  en `affaan-m/ECC` (clone `5064474`): su `memory_search` devuelve
  `diagnostics.truncated: true` al exceder el tope de escaneo —5.000 ficheros
  o 16 MiB, `scripts/lib/memory-vault.js:32-33`— en vez de recortar callando
  (`memory-vault.js:642-653`, `scripts/memory-mcp.mjs:294-301`). El campo va
  **anidado en `diagnostics`**, no en la raíz de la respuesta; sí está en raíz
  en su `memory_doctor` (`memory-mcp.mjs:333`). Decidir dónde va el nuestro es
  parte de la acción, no un detalle.
  **Cruce (2026-09-09, 2ª fuente):** el mismo patrón está resuelto
  **fail-closed** en `JuliusBrussee/caveman` (BSL-1.1, leído en el árbol
  clonado): `engine/engine.go:116-129` — si falla la escritura del backup del
  payload original, devuelve los bytes **sin transformar** más el error, con el
  comentario «a caller must never receive transformed bytes without a durable
  handle»; los metadatos de la transformación se publican solo después de que
  el backup haya ido bien (`engine.go:132`). El backup vive en SQLite
  (`~/.caveman/ccr.db`, handle = `ccr_` + 16 bytes de SHA-256 del contenido) y
  se recupera por handle. Traducido a exo: el truncado del bloque de arranque es
  hoy fail-open y sin handle —recorta y calla—, la dirección contraria.
  La pregunta que trae el precedente, y que ningún `fail-*` del harness tiene
  contestada por escrito: **¿qué es más caro aquí, pasarse o quedarse corto?**
  Los nueve hooks son never-block por política global
  (`arquitectura.md:320-323`), no por una decisión razonada hook a hook, y la
  asimetría no apunta igual en todos: en la inyección a subagentes abstenerse es
  barato —`DietrichGebert/ponytail` elige ahí fail-open a propósito y lo
  comenta, `hooks/ponytail-subagent.js:31-38`— pero en el bloque de arranque lo
  barato es gritar. No es item nuevo: es el criterio que le falta a la acción
  (a).

- [ ] **`inject-emitted` se emite aunque no se inyecte nada.** Medido el
  2026-08-27 al validar la Task 3-bis de la ola 1B: con la KB sin resolver, el
  perfil `reducido` (el del agente `executor`) compone **71 bytes de cabecera y
  cero rutas**, y `subagent-inject.sh` lo loguea igual como `inject-emitted`,
  con `bytes=70` enterrado en el payload. Un evento cuyo nombre afirma el
  efecto que no ocurrió. Los otros perfiles no lo exhiben porque su doctrina es
  estática y sobrevive sin KB (784 B): `reducido` es el único hecho solo de
  rutas, así que es el único que se queda en cero — y es el del agente que más
  disciplina necesita.
  **Causa inmediata** (esa sí se cierra en el cutover): `compose-inject.sh:29`
  resuelve la KB con `exo config --json`, subcomando nacido en la ola 1A, y el
  binario instalado del 24-08 responde `unrecognized subcommand`.
  **Acción, independiente del cutover:** que el evento distinga «compuesto con
  contenido» de «solo cabecera» — o un `inject-empty`, o un aviso cuando el
  bloque no supera el tamaño de la cabecera. Mientras el nombre del evento
  afirme más que lo ocurrido, el log no es evidencia. Detalle y medidas en
  `runbooks/2026-08-26-cutover-plugin-exo.md`.
  **Cruce (2026-09-09):** misma clase de fallo que «el bloque de arranque va al
  96% de su cap, y desborda en silencio», arriba — la forma de arreglo
  compartida (un campo del envelope, no dos avisos) está descrita allí.

- [ ] **`exo-recall.sh` no tiene suite de test.** Es el hook de SessionStart —
  lo que inyecta la KB al arrancar cada sesión — y la ola 1A lo modificó dos
  veces (Task 7, Task 8), respaldado solo por demostraciones manuales.
  `plugins/exo/scripts/` tiene `test-recall-inject.sh`,
  `test-compose-inject.sh` y `test-exo-index.sh`, pero nunca tuvo un
  `test-exo-recall.sh`.
  **Acción:** suite dedicada — cubrir el guard `no-engine`, el guard
  `no-config` y su orden relativo (ver hallazgo de la Task 8: antes de esa
  tarea, sin engine, se logueaban `no-config` Y `no-engine` para una sola
  causa), y el camino feliz.

- [ ] **Restricción de orden en el cutover binario↔scripts — nada la aplica
  hoy.** El alias oculto de D9 (ítem de retirar aliases españoles, abajo en
  Media) protege *scripts viejos → binario nuevo*. Nada protege la dirección
  contraria, que es justo la que produce un cutover real. Demostrado en la
  Task 10 (2026-08-26): ejecutando los scripts migrados del repo contra el
  binario v1 instalado, el hook de arranque no revienta — **sirve el texto de
  fallback embebido** ("Tu memoria persistente es una KB de notas markdown
  servida por..."), un bloque con forma correcta que no trae ni una nota de la
  KB. Degrada con forma válida: el peor tipo de fallo silencioso, porque nadie
  lo nota sin comparar contra lo que debería haber salido.
  **Acción:** en el cutover de la ola 1B, el binario nuevo se instala ANTES o
  en el mismo paso atómico que los scripts del plugin — nunca después. Y
  `exo doctor` debe detectar el desfase entre la versión del binario instalado
  y la versión del plugin: comprobación barata y falsable para un fallo que no
  grita.
  **Estado 2026-08-27:** la mitad del cutover está aplicada al plan — el
  Step 1½ nuevo de la Task 8 de `plans/2026-08-26-ola1b-plugin-exo.md` compila
  e instala el binario antes del plugin, y su check mira el envelope
  (`schema_version == 2`), no el mtime. **El item sigue abierto** por la otra
  mitad: el check permanente en `exo doctor` es G5 y no existe todavía.
  Medido ese mismo día: `~/.local/bin/exo.exe` es del 24-08 17:11, anterior al
  merge de la ola 1A (27-08 10:13) — el desfase no es hipotético, está vivo en
  esta máquina ahora mismo.
  **Cruce (2026-09-09):** la mitad viva —el check permanente en `exo doctor`—
  no hay que diseñarla entera: `affaan-m/ECC` (MIT) la tiene hecha como
  **install-state** —término literal suyo, `scripts/lib/install-state.js:11-13`,
  con schema `ecc.install.v1`—. Verificado contra clone `5064474`: fingerprint
  **SHA-256 por fichero** instalado (`install-lifecycle.js:203-205`, guardado
  como `contentSha256`) y un `doctor` que reporta el drift con severidad
  `ok|warning|error` (`install-lifecycle.js:1552-1571`, impresas en
  `scripts/doctor.js:44-53`; hay un cuarto estado `'missing'`, `:1605`, para
  cuando no hay install-state en absoluto).
  **Corrección de una lectura previa equivocada:** su `repair` **no** repone el
  byte sellado. En el camino normal recalcula el plan deseado contra los
  manifiestos **actuales** del repo usando solo la selección grabada en
  `state.request` (`install-lifecycle.js:1791-1826`) y reescribe únicamente lo
  `missing` o `drifted` por hash (`:1906-2151`) — así que si el manifiesto
  cambió desde la instalación, «repara» hacia el contenido nuevo, no hacia el
  original. Para `exo doctor` la mitad valiosa es la **detección** del desfase;
  el reponer-al-sello, que es lo que haría el trinquete, ahí no está y habría
  que ponerlo. **Lo cierra G5 si lo adopta.** **(2026-09-11: G5b cerró sin adoptarlo — release `v0.1.0` publicada, ver `## Cerrado con evidencia`. La marca queda huérfana: necesita dueño o campaña propia.)**

- [ ] **(pasada de coste 2026-09-09) El bucle de coste de la inyección está
  a un `join` de distancia: el emisor ya loguea los bytes que emite y nadie los
  ha cruzado con lo que cuestan.**
  Evidencia: `recall-inject.sh:335` loguea por prompt
  `emitted n_hits=$N bytes=$BYTES permalinks=$PERMALINKS`, y
  `_reflex-log.sh:15-27` le estampa `session_id` antes de escribir a
  `$HOME/.claude/reflex-log.jsonl` (`_reflex-log.sh:13`). El propio helper
  declara para qué existe —«para poder medir FP-rate por reflejo (paso 3 del
  proyecto reflejos)», `_reflex-log.sh:2-3`—: el paso de medición está
  **declarado, no ejecutado**. La otra mitad del cruce ya existe fuera del
  repo: Claude Code escribe `~/.claude/projects/<slug>/<session_id>.jsonl` con
  un `message.usage` por mensaje (`input_tokens`,
  `cache_creation_input_tokens`, `cache_read_input_tokens`, `output_tokens`,
  `output_tokens_details.thinking_tokens`). **La clave del join, `session_id`,
  está en los dos lados.**
  Medido el 2026-09-09 sobre las 60 sesiones mayores de esta máquina (549 en
  total, 331 MB, 7.920 mensajes de assistant): 2.266M tokens de input, de los
  cuales **97,7% son `cache_read`** (2.213,13M) y 2,3% `cache_creation`; hit
  rate 98,2% en `claude-opus-5`. Lectura: el coste no está en **fallar** la
  cache —ahí no hay margen— sino en **releer un prefijo grande a lo largo de
  miles de turnos**, que es exactamente la línea que `recall-inject` alimenta
  en cada prompt. exo ya trata la inyección como recurso presupuestado
  (`engine/src/presupuesto.rs`, lint `budget_prose_drift`, techo del bloque de
  arranque); lo que no tiene es el lado del **coste**.
  **Acción:** (a) pre-registrar el gate ANTES de medir, como en
  `evals/e1-read/gate.md` — la pregunta no es si el bloque es útil cuando
  acierta, sino si ahorra turnos y tool-calls suficientes para pagar su
  presencia en el prefijo cacheado de **todos** los turnos siguientes; (b) un
  harness que cruce `reflex-log.jsonl` por `session_id` contra los transcripts
  y reporte bytes inyectados frente a tokens por sesión; (c) decidir si eso
  vive en `evals/` o pasa a verbo del engine (ver el item de naming de
  `budget` en Media).
  **Aviso de medición:** exo **no está instalado en esta máquina** —
  `~/.claude/plugins/` solo tiene `kbi@kbi-standards` habilitado y `~/.exo` no
  existe—, así que aquí no hay `reflex-log.jsonl` y **su ausencia no prueba
  nada** sobre el emisor: el cruce hay que correrlo donde exo corre de verdad.
  Y ojo con el diseño best-effort del sink (`|| true`, `2>/dev/null`,
  `_reflex-log.sh:4`): un log ausente es indistinguible de cero disparos, así
  que el harness debe **exigir** el fichero, no tolerar su falta.

## Media

- [ ] **(NUEVO, revisión final campaña B, 2026-09-13) En `release.yml`, el
  check de versiones corre DESPUÉS de los tres builds (hasta 3×60 min),
  no antes.** `publish` (`.github/workflows/release.yml:105-136`) declara
  `needs: build` (la matriz linux/windows/macos, `timeout-minutes: 60`
  cada leg) y solo dentro de `publish`, tras el inventario de artifacts,
  corre «El tag casa con `engine/Cargo.toml`»
  (`.github/workflows/release.yml:131-136`: `bash scripts/test-versiones.sh
  "$TAG"`) — un check de segundos que no toca ningún artifact de build.
  Agravante medido el 2026-09-13: `scripts/test-versiones.sh` no existe en
  el árbol del único tag publicado (`v0.1.0`) — se añadió después, en
  `3bc05aa` (campaña B, H16) — así que un `workflow_dispatch` con `tag:
  v0.1.0` haría `checkout ref: v0.1.0` (`release.yml:45-47` para `build`,
  `:111-112` para `publish`), correría los tres builds completos, y
  fallaría en `publish` con el script inexistente: hasta 3 horas-runner
  gastadas antes de descubrir un desajuste que un job previo barato habría
  cazado en segundos.
  **Acción (no aplicada aquí):** mover el check de versiones a un job
  previo y barato (o al principio de `build`) del que `publish` (y
  arguably los propios legs de `build`) dependan con `needs:`, para que
  falle rápido sin gastar la matriz.

- [ ] **(revisión 2026-09-04) `tier` no se persiste en el índice y cada
  arranque relee el frontmatter de TODAS las notas desde disco.**
  `engine/src/nota.rs:14` lo declara: «el índice NO lo persiste (no hay
  columna nueva en `schema.rs` — forzaría un rebuild de las DB
  existentes)». Consecuencia en `engine/src/recall.rs:235`:
  `recall_arranque` llama a `tier_de(&ruta_abs)` por cada fila de `notas`,
  es decir, N lecturas y N parseos YAML en cada `SessionStart` solo para
  encontrar las notas `core`. Evitar una migración de esquema a cambio de N
  lecturas de disco por arranque es deuda disfrazada de prudencia; con 138
  notas no se nota, con miles sí, y `exo rebuild` ya existe como primera
  clase.
  **Acción:** columna `tier` en `notas` (+ bump de `meta` para que `verifica_
  modelo`/una guarda equivalente exija `exo rebuild` a los índices viejos) y
  `recall_arranque` filtrando en SQL. Borrar `tier_de` y su relectura.
  **(campaña A, 2026-09-13):** puerta C-H17a cerrada: s4 n5000 p95 = 47 ms ≤
  250; no se toca. Veredicto: evals/recall-coste/verdict/2026-09-campana-a.md.

- [ ] **(revisión 2026-09-04) Techos de escala declarados, sin camino ni
  medición.** Cuatro decisiones del engine son O(N) por operación y están
  documentadas como deliberadas, pero ninguna tiene medida más allá de la KB
  del autor (138 notas): KNN exhaustivo con `k = COUNT(*)`
  (`engine/src/buscador.rs:286`); un `HashMap` con TODOS los trozos cargado
  en memoria por query (`buscador.rs:290`); tres aperturas de la DB por
  búsqueda hybrid (`busca` + `busca_vector` + `buscador.rs:461`); y un
  proceso `git log -1` por nota indexada (`engine/src/indexer.rs:192`), que
  en un `rebuild` son N spawns de git, caros en Windows. Ninguna es un bug
  hoy; lo que falta es saber a qué tamaño de KB deja de valer cada una.
  **Acción:** generar una KB sintética de 5.000 notas y medir `exo rebuild`,
  `exo recall --content` y `exo search --type hybrid` en Linux y Windows.
  Con los números, o se documenta el techo soportado en `arquitectura.md`
  o se abre la campaña (índice particionado en vec0, `git log` en batch,
  una conexión por comando).
  **(campaña A, 2026-09-13):** medido con KB sintética de 174/1000/5000 notas
  (evals/recall-coste/). Encontrado y arreglado un techo DURO no listado: KNN
  de vec0 limitado a k=4096 (H27). Números y derivaciones en el veredicto.
  **H27 cerrado por PR #11** (`fix-knn-tope-vec0`, mergeado 2026-09-13,
  `f2207e2`): barrido SQL manual con `vec_distance_l2` cuando el KNN pide más
  de 4096. Efecto secundario medido: el barrido cuesta ≈0,09 ms/vector — a
  5.000 notas (`s2-query-n5000`) el p50 sube de 980 a 10.828 ms y deja el
  hook en timeout. Hotfix en curso (rama `fix-knn-k-por-consulta`, PR
  pendiente): fijar el k del KNN al de la consulta en vez de `k = COUNT(*)`,
  con el Threshold Algorithm de Fagin et al. (2003) — exacto, no aproximado.
  Prototipo a 5.000 notas: 10.179 → 1.106 ms. No hace falta ANN hasta
  ~500k trozos (Aumüller et al., ANN-Benchmarks, 2020).

- [ ] **(revisión 2026-09-04) El coste del hook completo en Windows no está
  medido; solo el del binario.** `plugins/exo/hooks/hooks.json` cablea
  **tres** scripts bash en cada `PreToolUse:Bash` (`git-c-bash.sh`,
  `git-add-all-guard.sh`, `verify-before-commit.sh`) y uno en cada
  `UserPromptSubmit` (`recall-inject.sh`) que lanza `exo recall --refresh`,
  `exo config --json` y **trece** invocaciones de `jq`/`sed`/`tr` (re-contado
  el 2026-09-11 sobre las 339 líneas del script: 7 `jq`, 2 `sed`, 4 `tr`; el
  «del orden de seis» original se quedaba corto a la mitad, en contra del
  propio argumento del item).
  Las cifras publicadas («~10 ms», `exo-recall.sh` cabecera; «~25 ms sin
  cambios», `exo-index.sh`) miden el binario, no el hook: bajo Git Bash cada
  spawn de proceso cuesta decenas de milisegundos, así que el coste real por
  prompt y por comando Bash en Windows es desconocido.
  **Acción:** instrumentar `_reflex-log.sh` con la duración del hook (o
  medir a mano con `time` sobre un `INPUT` real) en Windows y Linux, y
  publicar la cifra en `plugins/exo/README.md`. Si el `PreToolUse:Bash`
  triple supera ~200 ms, fusionar los tres scripts en uno con un único
  parseo del JSON de entrada.
  **(campaña A, 2026-09-13):** Linux: s6 hook entero n174 p50 = 1079 ms, s7
  config+jq = 4 ms. W11: pendiente (D5). El término dominante es la carga del
  modelo (≈0,95 s de ≈1 s), no el shell.

- [ ] **(revisión 2026-09-04 · CADUCADO A MEDIAS el 2026-09-09) El repo no
  le dice al toolchain local qué versión usar: falta `rust-toolchain.toml`.**
  Medido el 2026-09-04: `cargo check --all-targets --locked` en `engine/`
  fallaba con «exo@0.1.0 requires rustc 1.95» sobre `rustc 1.94.1`. La MSRV
  es correcta (la fija `libsqlite3-sys` vía `cfg_select`, ver
  `Cargo.toml:6-7`) y el CI la comprueba.
  **Lo que caducó (mitad de máquina)**: el 2026-09-09 esta máquina corre
  `rustc 1.98.0` / `cargo 1.98.0`, así que el fallo ya no reproduce. Se
  arregló solo, por actualización, no por acción sobre el repo.
  **Lo que sigue vivo (mitad de repo)**: `engine/rust-toolchain.toml` no
  existe. El repo sigue sin declarar el toolchain, así que la próxima máquina
  —o esta tras un `rustup default` distinto— repite el mismo tropiezo, y
  además falla **después** de resolver dependencias, que es lo que lo hacía
  caro de diagnosticar.
  **Re-confirmado el 2026-09-11**: `rustc 1.98.0` / `cargo 1.98.0`, toolchain
  `stable` activo y por defecto, instalado el 2026-08-24 según
  `~/.rustup/toolchains` —o sea, **antes** de la revisión del 09-04, cuya
  lectura de 1.94.1 venía de un `rustup default` dejado en el bisect de MSRV
  del 09-02—. `find . -iname "rust-toolchain*"` sigue vacío. La marca
  «CADUCADO A MEDIAS» de este item era **correcta**; el informe de la revisión
  crítica la revirtió por error el 09-11 por la mañana y se ha corregido en su
  §5.9. `docs/instalacion.md:25-28` ya documenta el mecanismo del corte, así
  que de la acción solo queda el fichero.
  **Acción:** `engine/rust-toolchain.toml` con `channel = "stable"` o la
  MSRV, para que rustup lo resuelva solo. (El `rustup update stable` de la
  acción original ya está hecho, y la anotación en `instalacion.md` también.)

- [ ] **(revisión 2026-09-11) Los documentos del repo no llevan `tier`, así
  que nada distingue lo que debe ser verdad hoy de lo que solo fue verdad un
  día.** Medido el 2026-09-11: `docs/` tiene **35.892** líneas de markdown en
  74 ficheros, de las que **33.469 (el 93 %) viven en `docs/superpowers/`** y
  **69 de los 74 llevan la fecha en el nombre** —specs, planes, verdicts,
  consultorías, runbooks: instantáneas que nadie tiene que mantener—. La
  superficie que sí debe ser verdad hoy son cuatro ficheros y **1.939
  líneas**: `docs/backlog.md` (1.114), `docs/arquitectura.md` (507),
  `docs/instalacion.md` (172) y `README.md` (146). Contra las 9.043 de Rust
  en `engine/src/`, eso es **0,21 líneas de documentación viva por línea de
  código**. Y sin embargo **de los 74 documentos solo uno lleva `tier:` en el
  frontmatter**: exo implementa `tier: core/stable/log`, techo por nota y
  `exo ratchet` para la KB, y no se lo aplica a su propio repo. Dos
  consecuencias medidas: los dos únicos fallos de deriva que se han podido
  probar (el «Sin CI» de `arquitectura.md` y los avisos de `instalacion.md`)
  están **los dos dentro de esas 1.939 líneas**, no en el 93 % restante; y hay
  **17 referencias** desde los cuatro ficheros vivos hacia `docs/superpowers/`
  (`grep -rn "docs/superpowers" README.md docs/{arquitectura,instalacion,backlog}.md
  | wc -l`), es decir, lo que se declara archivo está siendo citado como
  autoridad por lo que se declara vigente.
  **Acción:** (a) `tier: log` en la cabecera de todo `docs/superpowers/` y de
  cualquier documento con fecha en el nombre —incluido el informe de la
  revisión crítica—; (b) `tier: core` en los cuatro vivos; (c) el grep de
  afirmaciones frágiles del item de Alta corriendo **solo sobre los `core`**.
  Es la forma barata de cerrar la deriva documental: 1.939 líneas caben en un
  job de CI, 35.892 no. Cruza con «Decisión abierta: proceso frente a
  producto» (Baja), cuya evidencia —el ratio docs/código— este item
  reemplaza.
  **(campaña B, 2026-09-13, H18): D5=b — las acciones (a) y (b) se cierran
  por una vía distinta a la propuesta, sin frontmatter.** En vez de una
  columna `tier:` por fichero, la distinción vive como **convención por
  ruta**, declarada en la cabecera de `docs/arquitectura.md` (commit
  `2294349`): «Cuatro ficheros deben ser verdad hoy: `README.md`,
  `docs/arquitectura.md`, `docs/instalacion.md` y `docs/backlog.md`. Todo lo
  que lleva fecha en el nombre y todo `docs/superpowers/` son instantáneas
  (`tier: log` por convención de ruta): no se actualizan, se citan con su
  fecha.» Ningún fichero individual lleva `tier:` en el frontmatter — la
  regla se aplica por patrón de ruta, no por etiqueta. Sigue abierta la
  acción (c), compartida con la del item de Alta arriba: el grep de
  afirmaciones frágiles acotado a los cuatro vivos no existe todavía.

- [ ] **`#[allow(clippy::too_many_arguments)]` en `escritor.rs` — la struct de
  parámetros que no se hizo aquí.** `escribe_nueva` toma 8 parámetros contra
  el umbral de 7 de clippy; declarado en `engine/src/escritor.rs:252` en vez
  de refactorizar, porque agrupar en una struct de parámetros toca el camino
  de escritura y sus tests, y G5a era una tarea de CI, no de refactor.
  **Acción:** introducir una struct de parámetros para `escribe_nueva` (y
  revisar sus llamadores y tests) cuando se toque ese camino por otra razón.
  Esta entrada es localizable con `grep -rn too_many_arguments docs/backlog.md`;
  el `#[allow(clippy::too_many_arguments)]` en sí vive en
  `engine/src/escritor.rs:252` — el comentario de código no cita esta entrada
  del backlog por nombre.

- [ ] **Rutas personales y `hooks.json` sin validar en CI — las dos
  sub-propuestas vivas del item de los `test-*.sh` del plugin (cerrado el
  2026-09-12, `d8aa3b6`, ver `## Cerrado con evidencia`).**
  **Un validador**, no una fixture por script: `affaan-m/ECC` encadena en su
  `npm test` (`package.json:472`) un
  `scripts/ci/validate-no-personal-paths.js`. Con un gate equivalente, la
  limpieza que pide el item de Alta «"exo genérico" sigue siendo el plugin de
  Paul» **deja de poder regresar**, y con ella cae el item de Baja de
  `kb-demo` como fixture. **Copiarlo tal cual no basta** (clone `5064474`):
  sus regex (`validate-no-personal-paths.js:41-42`) cubren `/Users/<nombre>`
  y `C:\Users\<nombre>`, **pero no `/home/<user>`** — hay que añadirle el
  patrón POSIX y la lista de nombres propios, o no cazaría hoy la única
  ofensora que queda (`test-git-c-bash.sh`, ver el item de Baja de G4c
  Task 14). Segundo robable de la misma cadena: `scripts/ci/validate-hooks.js`,
  que valida `hooks/hooks.json` contra `schemas/hooks.schema.json` con Ajv
  (`:9,12,144-145`); `plugins/exo/hooks/hooks.json` tiene nueve hooks y
  ninguna validación.
  **Acción:** los dos gates, cuando tengan dueño; ninguno se escribió en la
  campaña B (fuera de su alcance — B tocó CI de shellcheck y exec-bit, no
  este).
  **Lo cierra G5 si lo adopta.** **(2026-09-11: G5b cerró sin adoptarlo — release `v0.1.0` publicada, ver `## Cerrado con evidencia`. La marca queda huérfana: necesita dueño o campaña propia.)**

- [ ] **(G4c, Task 13) `kb-precommit.sh` depende de que `exo` esté instalado
  en cada máquina — si no, el gate degrada a "commit permitido" en
  silencio.** `plugins/exo/scripts/kb-precommit.sh:18,20`: si el binario no
  está en `$EXO_BIN` ni en `$HOME/.local/bin/exo(.exe)`, imprime un aviso en
  stderr y sale `exit 0` — el commit pasa como si el gate no existiera. Tras
  el cutover kbx→exo de la Task 13, esto ya no es hipotético: cualquier
  máquina donde Paul retome G4d o trabajo sobre la KB sin haber instalado
  `exo` en esa ruta tiene el hook enlazado pero sin protección real.
  **Acción:** verificar `exo` instalado como parte de arrancar trabajo en una
  máquina nueva, o subir el aviso de stderr a algo que no pase desapercibido
  (el hook está enlazado, pero el gate no protege nada).

- [ ] **Retirar los aliases españoles del CLI en la 1.1 del engine** (versión
  propia del engine, distinta de la del plugin). Los diez flags
  renombrados en la ola 1A (`--limite`→`--limit`, `--titulo`→`--title`,
  `--contenido`→`--content`, `--nota`→`--note`, `--refresca`→`--refresh`,
  `--crea`→`--create`, `--min-similitud`→`--min-similarity`,
  `--escala-fts`→`--fts-scale`) mantienen el nombre viejo como `alias` oculto
  para que un plugin cacheado no muera a mitad de un hook durante el cutover.
  Al retirarlos, borrar también el test
  `los_flags_espanoles_siguen_parseando_como_alias` de `engine/tests/flags.rs`
  — si no, el borrado se ve rojo y alguien "arregla" el test reponiendo el
  alias.

- [ ] **Barrer los hallazgos vivos del gate M4** (`evals/e1-read/verdict/gate-m4.md`).
  Cerrados en `2f5f545`: traversal por `..` en `--dir`/`--titulo`, `--force` sin
  rastro en el envelope, flag muerto `--min-similitud` en `write new`. Cerrado
  en la Task 9 de ola 1A (2026-08-26): **#3** — el rechazo exit 3 ahora emite
  envelope con `--json` (`{"command":"write","data":{"reason":...}}`, claves en
  inglés por D8; `Rechazo::data` en `escritor.rs`, test
  `engine/tests/rechazo_envelope.rs`, spec corregida en
  `2026-08-18-m4-write-design.md`). Cerrado también en la ola 1A (M5a-02, ver
  `## Cerrado con evidencia`): el **disenso del consultor** — el prefijo de
  proyecto sale de `[kb] name` en la config propia, no de `kb.file_name()`.
  **Vivos 4, por orden de daño:**
  - **#5 [media]** sin fallback walk+parse (la spec §3.2 lo afirma en presente):
    con índice rancio, un `--crea` puede dejar **dos ficheros con el mismo
    permalink**. Riesgo hoy bajo (las 26 bitácoras de `log/` son slug-clean).
    Mínimo: walk de confirmación antes de crear.
  - **#8 [baja]** divergencia de slug medida **19/127** frente a basic-memory
    (`_` conservado en 10 bitácoras rotadas, CamelCase separado, `§`→`ss`).
    Autoconsistente, pero conviene decidirlo **por escrito antes de M5b**,
    porque las bitácoras rotadas de `/consolida` usan `_` en el título.
  - **#6 [baja]** `--crea` con permalink de 2 segmentos crea directorio espurio;
    `write_append_cmd` asume 3.
  - **#9 [baja]** el `SKILL.md` de `documenta` omite `--db` en los comandos del
    Paso 3; tomados literales fallan con error de clap.

- [ ] **Un rojo del job `test` no se puede diagnosticar desde el CI.**
  `engine/scripts/test-hermetico.sh:19` manda toda la salida de `cargo test`
  a `$TMP/out.txt` y el `trap ... EXIT` de la línea 16 la borra al salir. En
  fallo (líneas 23-26) solo se emiten las líneas que casan `^test result:
  FAILED|targets failed|--test `: nombre del binario y recuento, cero
  nombres de test, cero aserciones, cero backtrace. El workflow pone
  `RUST_BACKTRACE: 1` y el script tira esa salida igualmente. Peor: un error
  de COMPILACIÓN de la suite no casa ninguno de los tres patrones y se vería
  como una sola línea de exit. Consecuencia: un rojo exclusivo de
  `windows-latest` o de macOS arm64 es irreproducible en la máquina del
  autor e ilegible en el CI.
  **Ya no es una predicción por lectura de código: está medido.** La rotura
  deliberada de `fee361d` metió un `#[test]` que hace `panic!` con un mensaje
  explícito, y en la corrida `33624081143` los tres jobs `test` fallaron
  emitiendo exactamente `test result: FAILED. 4 passed; 1 failed` y
  `` `--test flags` `` — el binario y el recuento. **Ni el nombre del test ni
  el mensaje del panic aparecen en ningún log de CI**, aun estando el panic
  puesto a propósito para ser encontrado.
  **A medias (`50aee95`, 2026-09-10):** el script ya no se detiene en
  `test result: FAILED|targets failed|--test`. Ahora emite el nombre de cada
  test caído (`grep -E '^test .* \.\.\. FAILED$'`, `test-hermetico.sh:32`) y
  el bloque `failures:` de cargo con la aserción y el panic
  (`sed -n '/^failures:$/,/^test result: FAILED/p'`, `:34`), además del
  resumen anterior. **Sigue abierto:** un error de COMPILACIÓN de la suite
  (`cargo test` en `:19`) sigue sin casar ningún patrón de los tres —no hay
  grep que lo cace, exactamente el caso peor que este item señalaba— y
  `ci.yml` sigue sin `actions/upload-artifact` para el log completo. No
  cambia el veredicto del gate, solo lo que cuenta al fallar.
  **Acción propuesta:** campaña propia — `tee` o un `EXO_HERMETICO_LOG`
  opt-in en el script (con su propio ciclo rojo-verde, porque el script es
  un gate ya demostrado falsable y tocarlo invalida esa evidencia), más
  `actions/upload-artifact` en el job. No se arregla aquí, solo se anota.

- [ ] **Hoy el CI no bloquea nada.** Paul decidió explícitamente no proteger
  `main` por ahora — no hay branch protection ni required status checks.
  Consecuencia: un PR rojo se puede mergear igualmente, así que el CI hoy es
  una notificación, no un gate de merge. Ver la matización añadida al item
  cerrado «CI mínimo — el gate que faltaba» en `## Cerrado con evidencia`.
  **Acción:** activar branch protection con required status checks
  (`lint`, `msrv`, `test` en los tres SO) cuando se decida que main debe
  quedar protegida.
  **Cruce (2026-09-09):** con una cadena de release como la de `affaan-m/ECC`
  esto deja de ser una decisión suelta: su job de verificación exige que el
  commit del tag sea **exactamente `origin/main`** antes de empaquetar, con lo
  que la protección de rama pasa a ser precondición del release. Verificado
  contra clone `5064474`: `.github/workflows/release.yml:30-37`, step «Require
  the release commit to equal origin main», compara `git rev-parse HEAD` contra
  `git rev-parse origin/main` y sale con `exit 1` si difieren, en el job
  `verify`, antes de cualquier paso de empaquetado.
  **Lo subsume G5 si adopta esa cadena.** **(2026-09-11: G5b cerró sin adoptarlo — release `v0.1.0` publicada, ver `## Cerrado con evidencia`. La marca queda huérfana: necesita dueño o campaña propia.)**

- [ ] **Dos endurecimientos del CI que se decidieron NO aplicar en G5a, y por
      qué.** Hallazgos Minor de la review final de rama; se anotan para que la
      omisión sea una decisión y no un olvido.
      **Cruce (2026-09-09):** los dos son precondiciones de un release
      reproducible, no mejoras de higiene sueltas. Si G5 monta la cadena de
      custodia del artefacto —`--locked` en el job que de verdad corre la
      suite, `HF_HOME` explícito y su ruta de caché derivada— entran con ella
      en vez de necesitar campaña propia. **Lo subsume G5 si adopta esa
      cadena.** **(2026-09-11: G5b cerró sin adoptarlo — release `v0.1.0` publicada, ver `## Cerrado con evidencia`. La marca queda huérfana: necesita dueño o campaña propia.)**
  - **`HF_HOME` sin fijar.** La ruta del paso de caché
    (`~/.cache/huggingface/hub/models--jinaai--jina-embeddings-v2-base-es`)
    depende hoy del **default de `hf-hub` 0.5.0**, que es un detalle de una
    dependencia: `Cache::from_env` usa `$HF_HOME/hub` si la variable está
    puesta y `~/.cache/huggingface/hub` si no. Si ese default cambia en un
    upgrade, o si una imagen de runner empieza a exportar `HF_HOME`, la caché
    deja de acertar **sin que nada lo reporte** — degradación silenciosa de
    coste, siempre en verde. No se fijó en esta campaña porque cambiar la
    variable obliga a cambiar la ruta del paso de caché en el mismo commit, y
    equivocarse ahí rompe el acierto de caché ya demostrado
    (`Cache restored from key` en los tres SO, corrida `33621187141`).
    **Acción:** fijar `HF_HOME` explícito y la ruta derivada de él en un
    commit propio, verificando el `Cache restored` de los tres SO antes y
    después.
  - **`--locked` no llega al job `test`.** `lint` y `msrv` sí lo tienen
    (`ci.yml:46`, `:63`), pero el job que de verdad corre la suite invoca
    `engine/scripts/test-hermetico.sh`, que llama a `cargo test` sin
    `--locked`, y ese script no se toca (es un gate demostrado falsable).
    Consecuencia: el job que más importa puede resolver un árbol de
    dependencias distinto del `Cargo.lock` commiteado.
    **Acción:** entra en la misma campaña que la deuda de diagnosticabilidad
    del script, arriba — las dos exigen tocarlo y por tanto rehacer su ciclo
    rojo-verde.

- [ ] **`walker::walk_kb` frente a `walk_kb_excluyendo`: conviven con semánticas
  distintas desde G4b.** Verificado en `engine/src/walker.rs`: `walk_kb`
  (`:11-40`, la que usa `indexer::indexa` en `indexer.rs:158`) compara la
  extensión con `== Some("md")` sin normalizar mayúsculas — un `NOTA.MD` no se
  indexa — y solo excluye `.claude/`, `.omc/` y `.superpowers/`
  (`DOTDIRS_EXCLUIDOS`, `:5`), así que camina dentro de `.git/` sin nada que lo
  frene. `walk_kb_excluyendo` (`:85-140`), nacida en esta misma ola para
  `lint`, usa `es_md` (case-insensitive, A5, `:61-65`) y su `recorre` salta
  cualquier directorio que empiece por `.` (`:124`), `.git/` incluido. Las dos
  funciones nuevas no tienen ninguno de los dos problemas; la asimetría vive
  dentro del mismo módulo. Alinearlas es un cambio de comportamiento del
  índice (una `NOTA.MD` empezaría a indexarse; `.git/*.md` dejaría de
  recorrerse), no una limpieza de estilo, así que no se toca en G4b.
  **Acción:** cuando se toque `indexer::indexa` por otra razón, decidir si
  adopta la semántica de `walk_kb_excluyendo` (candidata natural a fusionar
  en una sola función) o si la divergencia es deliberada y se documenta como
  tal.

- [ ] **`indexer::ruta_relativa` guarda `notas.ruta` con el separador nativo
  del SO.** `engine/src/indexer.rs:461-473` arma la ruta relativa con
  `to_string_lossy()` sin `.replace('\\', "/")`, así que en Windows la DB
  persiste `notas.ruta` con `\`. G4b lo normaliza **al leer**, en los dos
  únicos consumidores de `notas.ruta` que toca este plan: `lint::huerfanas`
  (`lint.rs:183`) y `lint::indice_rancio` (`lint.rs:360`, con comentario
  explícito — «Mismo motivo que en `huerfanas`: `notas.ruta` lleva separador
  nativo»). La causa sigue en el indexer; el próximo consumidor de
  `notas.ruta` que no conozca este parche vuelve a tropezar en Windows.
  **Acción:** normalizar en `ruta_relativa` al escribir, no en cada lector, y
  borrar entonces los dos `.replace('\\', "/")` de `lint.rs`.

- [ ] **`budget_prose_drift` tiene dos límites conocidos, ninguno arreglado
  aquí.** Los dos viven en la misma pareja regex+parse de
  `engine/src/lint.rs` (`TIER_Y_CIFRA`, `:271-274`, y el parse de
  `deriva_de_prosa`, `:310`).
  - **Punto ciego por adyacencia (medido 2026-09-04).** La regex exige que la
    cifra vaya pegada al tier (`\b(core|stable|log)\b[:\s]+([0-9]+...)`), así
    que en la KB real `core/doctrina-agentes.md:54` («`core` (… 8.500 B …)»)
    es invisible para el check, mientras que `core-index.md:19` («core
    8.500») sí se ve. Es el precio deliberado de no tener falsos positivos
    (`la_deriva_de_prosa_calla_ante_una_mencion_vaga`,
    `engine/tests/lint_presupuesto.rs:234`: «Falsos positivos son peores que
    fallos aquí: un gate que grita se ignora»), pero no estaba declarado como
    límite conocido hasta ahora.
  - **Trunca en vez de rechazar una cifra mal agrupada (medido 2026-09-04,
    review de la Task 8).** La captura de `TIER_Y_CIFRA` solo admite grupos
    de tres dígitos exactos tras el punto: `"core 1.2345 B"` captura
    `"1.234"` (verificado con la regex equivalente), y el parse de `:310`
    produce el hallazgo *"cita core 1234B"* — una cifra que no está en el
    texto. Es un falso positivo en el único check cuyo test de regresión
    declara que los falsos positivos pesan más que los fallos. **Es heredado
    literal del Go**: `internal/doctor/doctor.go:397` en `fe46443` tiene la
    misma regex y el mismo parse
    (`strconv.ParseInt(strings.ReplaceAll(rawFigure, ".", ""), 10, 64)`), y
    `doctor_test.go` no cubre una cifra mal agrupada en ninguno de sus cinco
    tests de `budgetProseDrift` — ni la suite Rust (`lint_presupuesto.rs`) lo
    hace tampoco. Arreglarlo aquí divergiría del binario de referencia y
    abriría una décima divergencia en un gate que todavía no se ha podido
    correr ni una vez contra Go. Decisión de no arreglarlo tomada en la
    review de la Task 8 (2026-09-04).
  **Acción:** ampliar la regex, o rechazar explícitamente una captura que no
  consume toda la cifra, es trabajo para cuando una cita real mal formada
  haga daño de verdad, o para cuando exista el gate de paridad con Go y el
  fix se pueda decidir en los dos binarios a la vez.

- [ ] **La campaña de evicción de la KB está descalibrada (A3, G4b).** El
  censo "19 de 58 notas stable" y el objetivo de poda de 10.625 (medidos el
  2026-09-02) salen de la fórmula huérfana del commit local `f0d0564` de kbx
  (`objetivo_poda = tier - tier*15/100`). G4b adjudicó A3: la fórmula
  canónica es la de `fe46443` (`techo*100 >= tamaño*115`,
  `objetivo_poda(techo) = techo*100/115`), ya portada a
  `engine/src/presupuesto.rs`. Con la canónica los umbrales son **10.869**
  (stable) y **7.391** (core) — 244 B más de margen por nota en stable, y un
  censo menor: las notas entre 10.626 y 10.869 dejan de estar en poda. Este
  plan no toca la KB.
  **Acción:** rehacer el censo y el objetivo de poda con `exo budget` sobre
  la KB real, con los umbrales canónicos (10.869 stable / 7.391 core), antes
  de ejecutar cualquier evicción.

- [ ] **(pasada de coste 2026-09-09) `exo budget` va a colisionar de nombre:
  el planeado mide tamaño de KB y el que hace falta mide coste de tokens.**
  Evidencia: `docs/arquitectura.md:486` y `docs/instalacion.md:119-120` listan
  `exo budget` como planeado y remiten su diseño a la sección G5 de
  `docs/superpowers/specs/2026-08-26-exo-generico-design.md`. La semántica
  heredada es **tamaño de nota**: `kbx budget/stale/doctor`
  (`docs/2026-08-02-foto-as-is-framework.md:21`), los umbrales en bytes de
  `engine/src/presupuesto.rs` (10.869 stable / 7.391 core, ver el item de poda
  en Media) y el lint `budget_prose_drift`. **Nada de ese diseño cubre coste
  de API.** Con el item de Alta encima de la mesa, el repo necesita las dos
  medidas a la vez y `budget` solo puede significar una.
  **Acción:** (a) fijar el vocabulario antes de que G5 lo implemente —
  propuesta: `budget` = bytes de KB (lo ya diseñado), `cost` = tokens gastados
  (nuevo); (b) si se decide un solo verbo, que la distinción viaje en el
  envelope y no en un `--flag`, para que el consumidor no adivine; (c) dejar
  la decisión escrita en la spec de G5, no solo aquí.

- [ ] **(pasada de coste 2026-09-09) M5a (MCP propio) se diseñó contra un MCP
  con estado; la revisión 2026-07-28 del spec lo abarató y dejó el diseño sin
  releer.**
  Evidencia: `docs/arquitectura.md` §7 lista «MCP propio (M5a) y
  desinstalación de basic-memory (M5b): pendientes», y el `## Estado` de este
  backlog lo encadena como C9. La revisión **2026-07-28** del spec de MCP
  convierte el protocolo en request/response **sin estado**: retira el
  intercambio `initialize`/`initialized` y la cabecera `Mcp-Session-Id`,
  elimina el endpoint GET de stream, exige cabeceras `Mcp-Method` y `Mcp-Name`
  en todo POST de Streamable HTTP, y hace **cacheables** las respuestas de
  `tools/list`, `prompts/list`, `resources/list` y `resources/read` vía
  `ttlMs` y `cacheScope`. El ejercicio pasa de implementar una máquina de
  estados con handshake a implementar **un endpoint HTTP**. Fuente de segunda
  mano — el análisis «Los 15 build your own de AI Engineering», fuera de este
  repo (árbol `proyectos/IA/pocs/`), §«MCP crudo», que cita el blog oficial
  del spec; **verificar contra el spec antes de implementar**, no contra ese
  análisis.
  **Acción:** (a) releer la sección M5a de la spec de C9/G5 contra la revisión
  nueva antes de escribir código — cualquier diseño de handshake es ya peso
  muerto; (b) `ttlMs`/`cacheScope` sobre `list` toca directamente el item de
  Alta: las respuestas de `list` **son prefijo**, así que cachearlas es la
  misma palanca medida allí; (c) evaluar `defer_loading` para los tools poco
  usados, que los mantiene fuera del prefijo cacheado y solo entran cuando el
  modelo los busca.

- [ ] **(NUEVO, 2026-09-13) Proceso residente para el coste fijo del recall
  por prompt — decisión de Paul: al backlog, no se hace ahora.** Medido por
  el consultor: de los ~950 ms de `exo recall --query`, ~910 ms son carga
  del modelo (`fs::read` de `model.onnx`, 641 MB, ≈190 ms + sesión ORT con
  tokenizer ≈720 ms); el arranque del proceso son 3,5 ms y el embed de la
  query 18 ms — el coste fijo es casi todo el presupuesto.
  Prototipo de proceso caliente (`caliente.rs`): 27-31 ms por prompt sobre la
  KB real, 112 ms a 95k trozos.
  Diseño pendiente de brainstorming con Paul: transporte (TCP en localhost /
  socket unix / named pipe en Windows), ciclo de vida (spawn perezoso,
  muerte por inactividad ~20 min, precalentado desde el hook de arranque),
  check de versión binario/índice, fallback del hook si el proceso no
  responde, ~1 GB de RSS residente.
  Alternativa sin daemon: pesos externos + `commit_from_file` en ORT
  (~400 ms, mismos scores que hoy).
  Descartado sin eval propio: `model_quantized.onnx` int8 (~360 ms, pero el
  top-4 solo solapa 2-4 de 4 con el fp32 actual → cambia retrieval, exige
  campaña C + reindex antes de adoptarlo).
  Descartados por medida (sin ganancia): más threads intra-op, optimización
  de grafo, cachear el embedding de la query, embed en paralelo al arranque.
  Criterio de reapertura ya implementado:
  `plugins/exo/scripts/recall-latencia.sh` (campaña A, Task 13, D4).

- [ ] **(NUEVO, H28, para campaña C) La `distance` de vec0 es L2, no L2²; el
  umbral 0,40 del hook equivale a coseno 0,28.** Medido por el consultor
  sobre la KB real: los embeddings están normalizados (norma 1,000000) y
  `similitud_desde_l2_cuadrado` (`engine/src/buscador.rs` ~:226) calcula
  `1 − sqrt(2−2cos)/2` — monótona en coseno, así que el ranking no cambia —
  pero interpreta la `distance` de vec0 como L2² cuando en realidad es L2
  (`sqlite-vec.c:224/263`). El umbral 0,40 del hook equivale a coseno 0,28, y
  β de la fusión híbrida está calibrado sobre esa escala desplazada.
  **Acción:** arreglarlo cambia qué trozos entran o no en el umbral — pasa
  por el held-out de la campaña C (H7) antes de tocarlo, no se corrige suelto.

- [ ] **(NUEVO, N1, para campaña C) Con un prompt natural, FTS5 hace AND de
  todos los tokens y da 0 candidatos: `exo recall --query` es en la práctica
  vectorial puro.** Medido por el consultor sobre prompts naturales; la
  fusión híbrida (bonus, β) solo actúa de verdad en queries de palabras
  clave, no en el uso real del hook de arranque.
  **Acción:** decidir en la campaña C (relajar a OR, o algún matching
  parcial) con el held-out de H7 delante.

- [ ] **(NUEVO, 2026-09-13) El bench sintético de la campaña A es ciego al
  umbral de similitud.** Vectores aleatorios en 768 dimensiones dan coseno
  ≈ ±0,04 entre sí; ninguno pasa el umbral 0,40 del hook, así que el arm
  vector nunca aporta resultados en el bench sintético
  (`evals/recall-coste/`) y cualquier heurística que dependa del umbral sale
  bien ahí y mal en producción.
  **Acción:** generador con vectores reales (de la KB real) + ruido, no
  aleatorios uniformes.
  Además, `C-noregresión` (campaña A) se pre-registró solo sobre `s3`/`s5` y
  por eso no vio la subida real de `s2` (KNN) tras cerrar H27 — se detectó a
  mano en el veredicto, no por el criterio. Ver
  `evals/recall-coste/verdict/2026-09-campana-a.md`.

- [x] **(revisión 2026-09-13 · plegado el 2026-09-13, campaña B) Hallazgos
  de hoy — cada uno cerrado con su commit; este bloque no queda «planificado»
  y «cerrado» a la vez.** Diseño y tareas en
  `docs/superpowers/plans/2026-09-13-campana-b-superficie-y-gates.md`.
  - **H15** cerrado (`fde9ec7`): `trinquete::comprueba_contra` (213 líneas)
    partido en tres familias con nombre y tests por familia.
  - **H8/H9** cerrado (`7853e8a`, `b481189`): `--help` de producto sin jerga
    de campaña; metavars iguales al flag y ninguna opción de `--json` sin
    descripción. Ver además el item de Baja «idioma mezclado» y el de
    «relato de campaña en los comentarios», arriba.
  - **H20/H21** cerrado (`62b5152`, `be6c200`+`018e593`): un solo
    `_truncate-payload.sh` para los dos reflejos de Bash que truncan
    payload; el matcher de «orquestador limpio» cubre también la
    navegación de los MCP de navegador, no solo `localhost`.
  - **H11/H12** cerrado (`cd3bfff`, `4af859b`): shellcheck 0.11.0 en CI sobre
    los scripts versionados (45 en el árbol a esa fecha, no 39/41 como
    estimaba el plan); el gate de exec-bit cubre también los scripts sin
    extensión con shebang. Ver `## Cerrado con evidencia`.
  - **H22** cerrado (`e5398d5`+`1018802`): `distill/SKILL.md` a
    procedimiento en el cuerpo, detalle bajo demanda en ficheros aparte.
  - **H13**: este mismo commit — el backlog, re-sincronizado con CI y con
    esta campaña al cierre de la B.
  - **H25: NO lo cierra la fábrica.** Sigue como checklist externo de Paul
    (borrar ramas remotas mergeadas, `delete-branch-on-merge`, `gh repo
    edit` con el About del README) — fuera del plan de tasks, ver la
    sección «Checklist externo para Paul (H25)» del plan de campaña B.

- [ ] **(revisión 2026-09-13) H14 y H24, planificados para la campaña C (sin
  item propio en este backlog).** Diseño en
  `docs/superpowers/plans/2026-09-13-campana-c-preregistro.md`.
  - **H14:** si el solape entre trozos o el late chunking mejoran el
    troceado fijo de 900 caracteres.
  - **H24:** el gold de `evals/retrieval-fase0/` admite `acceptable_permalinks`
    secundarios, no solo un permalink correcto por query.

- [ ] **(NUEVO, campaña B, 2026-09-13) El error de la guarda «una DB sirve a
  una KB» recomienda `--db`, que `exo init` no tiene.** Medido al documentar
  la guarda H1 en la Task 10 (`docs/instalacion.md` §4): `exo init` no
  acepta flag `--db` — resuelve el índice por `$EXO_DB` o el default de la
  config. `comprueba_kb_root` (`engine/src/indexer.rs:484-490`) sí se
  invoca desde `exo init` (`inicia.rs:125`) además de `exo index`/`exo
  rebuild` (que sí tienen `--db`, `ArgsIndex.db`), y su mensaje de error es
  literal para los tres: «Una DB sirve a UNA KB: usa otra `--db` para
  esta…» — correcto cuando lo dispara `index`/`rebuild`, engañoso cuando lo
  dispara `init`, donde la única vía real es `EXO_DB=<ruta> exo init …`.
  `docs/instalacion.md:109-110` ya tuvo que aclararlo aparte: «(`exo init`
  no tiene flag `--db`; usa la variable de entorno).»
  **Acción:** el mensaje deja de asumir un flag que no todos sus llamadores
  tienen — o se parametriza por comando (`init` → menciona `$EXO_DB`,
  `index`/`rebuild` → menciona `--db`), o se reescribe genérico («usa otro
  índice para esta KB: `--db` en `index`/`rebuild`, `$EXO_DB` en `init`»).
  **Deuda hermana (revisión final campaña B, 2026-09-13): `exo init` no
  respeta `$EXO_DB` al escribir `[index] db` en la config.** `init_cmd`
  calcula `db_objetivo` con la precedencia `$EXO_DB` > default
  (`engine/src/main.rs:560-563`) y lo usa para la indexación inicial
  (`resuelve_db(None)` en `engine/src/main.rs:629`), pero
  `escribe_config` recibe `db_default` — SIEMPRE `~/.exo/index.db`, nunca
  `db_objetivo` — como el valor que graba en `[index] db`
  (`engine/src/main.rs:622`, `exo::inicia::escribe_config` en
  `engine/src/inicia.rs:131-171`). Medido el 2026-09-13 en un `HOME`
  aislado: `EXO_CONFIG=~/.exo/otra-kb.toml EXO_DB=~/.exo/otra-kb.db exo
  init --kb ~/otra-kb --name otra-kb` indexa `otra-kb.db` en el `init`,
  pero `otra-kb.toml` queda con `db = "~/.exo/index.db"` — un `exo search`
  posterior con solo `EXO_CONFIG` (sin `EXO_DB`) sale 0 y responde desde el
  índice de la PRIMERA KB, sin avisar. Receta que sí funciona (editar
  `[index] db` a mano tras el `init`) documentada en `docs/instalacion.md`
  §4. **Acción:** que `escribe_config` reciba y grabe `db_objetivo`, no
  `db_default`.

- [ ] **(NUEVO, campaña B, 2026-09-13, H15) `trinquete::sellos_escapados_de_tier`
  lee el tier del disco también en `--staged`.** El propio código lo
  declara (`engine/src/trinquete.rs:830-832`, comentario añadido al partir
  `comprueba_contra` en tres familias): «Lee el tier del **disco** también
  en `--staged`: es el comportamiento heredado de antes de partir
  `comprueba_contra`, y este refactor no lo cambia.» Con `--staged`, el
  resto del trinquete mira el índice de git (lo que se va a commitear), pero
  esta función concreta sigue leyendo `kb.join(ruta)` del working tree —
  posible mezcla de revisiones si el working tree y el staged divergen (un
  `tier:` editado sin `git add`).
  **Acción:** antes de tocarlo, verificar contra `internal/ratchet` de kbx
  (`fe46443`) si el comportamiento heredado es intencional o es el mismo
  tipo de deuda que ya viven otras funciones de este módulo — no se toca a
  ciegas un gate que ya se demostró falsable.

## Baja

- [ ] **(NUEVO, revisión final campaña B, 2026-09-13) El bash inline de
  `run:` en `.github/workflows/*.yml` no pasa por ningún gate.**
  `scripts/test-shellcheck.sh` solo analiza ficheros `.sh` versionados (y
  ejecutables sin extensión con shebang sh/bash bajo `plugins/`) — su propio
  comentario dice «Qué NO entra» y los workflows no están en la lista. Ya
  rompió una release real: `sha256sum` (GNU) vs `shasum` (macOS/BSD) en
  `release.yml` tumbó el push del tag `v0.1.0` en el runner de Windows
  (`shasum: command not found`, exit 127; fix PR #7 `360175c`) y, ya
  arreglado hacia el otro lado, rompió `install.ps1` (formato de firma
  distinto según el comando; fix PR #8 `8a86832`) — tabla de corridas en
  `## Cerrado con evidencia`, «Release `v0.1.0` — el binario que no
  existía». **Acción:** extender
  `test-shellcheck.sh` (o un gate hermano) a los bloques `run: |` de
  `.github/workflows/*.yml` — extraerlos a ficheros temporales o usar el
  soporte de shellcheck para YAML embebido — o documentar explícitamente
  que ese bash queda fuera de todo gate y por qué.

- [ ] **(NUEVO, revisión final campaña B, 2026-09-13) El job `lint` de
  `ci.yml` se llama «fmt + clippy» pero ya corre shellcheck y el gate de
  versiones.** `.github/workflows/ci.yml:24-61`: el job `lint` (`name: fmt +
  clippy`) tiene cuatro steps — `cargo fmt --check`, `cargo clippy`,
  shellcheck sobre el bash versionado, y «Versiones coherentes
  (`plugin.json` == `marketplace.json`)» (`bash scripts/test-versiones.sh`)
  — dos de los cuales no son ni fmt ni clippy. **No renombrar a ciegas:**
  si GitHub tiene branch protection con required status checks por nombre
  de job (`lint`) o de step, renombrar el job rompe la protección hasta que
  alguien actualice la config del repo (fuera de este árbol de código) —
  verificar `required_status_checks` del repo antes de tocarlo. **Acción:**
  o renombrar `lint` a algo que cubra las cuatro cosas («checks estáticos»,
  «lint + gates estáticos») coordinando el cambio de required checks, o
  separar shellcheck y versiones a su propio job con nombre propio.

- [ ] **(H29, Baja, 2026-09-13) El walker entra en `.git/`.** Medido por el
  consultor con `strace` sobre un `exo index`: 276 de 314 `openat` caen
  dentro de `.git/`. Solo dato — cruza con el item de Media
  «`walker::walk_kb` frente a `walk_kb_excluyendo`» (arriba), que ya
  documenta que `walk_kb` no frena en directorios que empiezan por `.`.

- [ ] **(revisión 2026-09-04 · cifras RE-MEDIDAS el 2026-09-09) Decisión
  abierta: proceso frente a producto.**
  Medido el 2026-09-04 con `wc -l` sobre `git ls-files`: **30.547** líneas de
  markdown en `docs/` + `evals/` + `reports/` frente a **5.224** de Rust en
  `engine/src/` (ratio 6:1), más 5.629 de tests Rust y 4.545 de shell. 320
  commits en 15 días de actividad, un solo autor, picos de 77 commits/día.
  **Re-medido el 2026-09-09: 35.527 markdown · 6.682 Rust en `engine/src/` ·
  7.381 de tests Rust · 4.692 de shell → ratio docs/código 6:1 → 5,3:1.** En
  cinco días el código creció un 28% y la documentación un 16%: la tendencia
  que este item denunciaba **se ha invertido**, aunque la decisión de fondo
  siga sin tomarse. 345 commits en total, autoría única confirmada (342 bajo
  el mismo nombre), pico de 77 commits/día el 2026-07-17.
  **Re-medido el 2026-09-11 (tarde), y la evidencia de este item se retira:**
  36.922 markdown · **9.043** Rust en `engine/src/` · 10.210 de tests · 4.713
  de shell → ratio **4,1:1**. Cayó de 5,4 a 4,1 **en un solo día y sin que
  nadie tocara una línea de documentación**, porque G5b metió código: un
  número que se mueve un 25 % por razones ajenas a lo que dice medir no
  sostiene un item. Y el desglose que faltaba lo invierte del todo: el 93 % de
  ese markdown son instantáneas fechadas y la documentación **viva** son 1.939
  líneas, 0,21 por línea de código. Por volumen exo no está sobredocumentado;
  lo que está sobredimensionado es el exhaust de proceso, que se paga al
  generarlo y no al mantenerlo. **La evidencia medible de esta decisión pasa
  al item nuevo de `tier` (Media)**; lo que queda aquí es la decisión de fondo,
  que sigue sin tomarse.
  Dos costes que sí resisten la medición: el **coste por unidad entregada**
  (un brief y un informe por tarea, commit por micro-paso, mensajes-ensayo:
  402 commits en 19 días) y la **frontera rota** de las 17 referencias desde
  los cuatro ficheros vivos hacia `docs/superpowers/`. En contra, el dato que
  este item nunca tuvo: el plan de 105 KB de G5b entregó `exo doctor`, dos
  instaladores, el workflow de release y la release publicada en dos días,
  con tests — el aparato de proceso está pagando por sí mismo.
  **Acción:** escribir la respuesta en el README en dos frases («para quién
  es exo hoy»). De la mitad operativa —qué carpetas dejan de mantenerse— se
  encarga el marcado por `tier`. Se cruza con «Nombres y ubicaciones» de
  abajo.
  **(campaña B, 2026-09-13, H6): la acción del README está hecha, D2=a
  literal.** Commit `c5c5b7f` reescribe el README con una sección «Para
  quién es hoy»: «exo es el sistema de trabajo de su autor, publicado tal
  cual (MIT). Funciona y se prueba en Linux, macOS y Windows, pero lo decide
  un solo usuario: sin promesa de estabilidad ni soporte.» Se retira además
  el bloque de estado con jerga de campaña (M0, M1a, G5b…) y la cifra
  frágil de tests. Lo que queda abierto no es la acción, es la **decisión de
  fondo** que el propio título nombra (proceso frente a producto): escribir
  la respuesta no es tomarla.

- [ ] **(revisión 2026-09-04) Idioma mezclado sin criterio único.** Medido
  sobre `engine/src/`: identificadores y módulos en español (`buscador`,
  `trozos`, `aristas`, `escritor`, `objetivos`, `inicia`), claves JSON y
  flags largos en inglés desde D8 (`SCHEMA_VERSION` 2), aliases ocultos en
  español, commits, docs y comentarios en español. Cada capa eligió distinto
  y el resultado es que un contribuidor externo necesita las dos lenguas y
  un lector del envelope no reconoce los nombres del código que lo emite
  (`Busqueda.avisos` ↔ `"warnings"`, `Resumen.indexadas` ↔ `"indexed"`).
  **Acción:** decidir por escrito (una línea en `arquitectura.md` §3.8 o en
  `CONTRIBUTING`) qué idioma llevan identificadores de código, y aplicarlo
  solo a módulos nuevos hasta que un refactor toque los viejos. No renombrar
  en masa: el coste hoy es de coherencia, no de corrección.
  **(campaña B, 2026-09-13, H8/H9): cerrada solo la parte de la superficie
  CLI, con D1=C.** `docs/arquitectura.md` §3.8 declara la convención
  (commit `7853e8a`): «Idioma de la ayuda (D1 = C): los textos de producto y
  los errores propios van en español; el cromo que pinta clap (`Usage:`,
  `Options:`, `Commands:`…) y los metavars (`--limit <LIMIT>`, igual al
  nombre del flag) se quedan en inglés.» Aplicada en `--help` (91 líneas de
  jerga interna reescritas, `7853e8a`) y en los seis metavars que mezclaban
  idioma en la misma línea (`b481189`), con test de regresión
  (`engine/tests/help_producto.rs`, `la_ayuda_no_lleva_jerga_interna`). **La
  parte de identificadores de código (módulos y funciones en español,
  claves JSON en inglés) sigue abierta** — D1=C fija el idioma de la
  superficie de usuario, no el de `engine/src/`.

- [ ] **(revisión 2026-09-04 · re-medido y ACOTADO el 2026-09-09) El relato
  de campaña en los comentarios se concentra en `main.rs` y `buscador.rs`, y
  referencia briefs que no están en el repo.** Medido el 2026-09-04: **1.370**
  de las 5.224 líneas de `engine/src/*.rs` eran comentario (26 %); `main.rs`
  241/881, `recall.rs` 155/671, `lib.rs` 131/333. **Re-medido el 2026-09-09
  con criterio explícito `^\s*(//|///|//!)`: 1.815/6.682 = 27,2 %**
  (`main.rs` 275/1043, `recall.rs` 155/671 clavado, `lib.rs` 131/336). El
  criterio del auditor es reproducible y la densidad no baja pese al código
  nuevo — pero el porcentaje **no era el hallazgo**, y el muestreo lo acota:
  la mayoría de esos comentarios sí enuncian el invariante y solo le añaden
  la procedencia. `lib.rs:29-31` dice «(deferred de campaña 1, review opus
  m2-01: `sqlite3_auto_extension` es acumulativo — registrar dos veces
  duplica el extension point)», que es exactamente lo que la Acción de abajo
  pide **conservar**, no lo que pide mover. Contando líneas de comentario con
  marcador de brief/spec/§/Task, el relato puro vive en **`main.rs` (32) y
  `buscador.rs` (30)**; `lib.rs` (11) sale exonerado. Sigue en pie el riesgo
  de fondo: un comentario que cuenta por qué se cambió algo envejece igual
  que el README de la sección de Alta, y ya hay un caso medido
  (`exo-recall.sh` decía «ronda los 4,5 KB» cuando eran 5.921 B, ver primer
  item de Alta).
  **Acción:** al tocar un módulo por otra razón, dejar en el código el
  invariante y su consecuencia («recencia = git, no mtime: un clone fresco
  resetea mtimes») y mover el relato («hallazgo del gate M6, 2026-08-22») al
  verdict o al plan correspondiente con un enlace. Candidatos por densidad de
  relato **medida**: `main.rs` y `buscador.rs` (el item original decía
  `lib.rs` y `main.rs`).
  **(campaña B, 2026-09-13, H8): a medias.** El commit `7853e8a` reescribe
  las 91 líneas de jerga interna (ids de hito, specs, símbolos de Rust/SQL,
  nombres de scripts del autor) que vivían en los doc-comments `///` que
  clap renderiza en las 15 pantallas de `--help` de `main.rs` — esos son los
  únicos que un usuario del binario llega a ver, y por eso salen primero.
  **No tocado:** los comentarios `//` internos de `main.rs` (el relato que
  no sale a `--help`) y todo `buscador.rs`, que sigue con sus 30 líneas de
  relato medidas arriba.

- [ ] **Nombres y ubicaciones.** `docs/superpowers/` como carpeta de docs del
  proyecto cuyo objetivo declarado es jubilar superpowers, y `reports/` colgando
  de la raíz fuera de toda convención (los verdicts sí viven ordenados en
  `evals/*/verdict/`). **Acción:** decidir de una vez — renombrar o escribir por
  qué se queda. Barato ahora, caro cuando haya más ficheros.
  **Actualización (G2, fusión de plugins):** resuelta la incoherencia de
  nombres para `plugins/` — ya no hay `process`/`reflex`, hay un único
  `plugins/exo/`. Quedan vivas como deuda sin resolver `docs/superpowers/` y
  `reports/`; se abordan en G5.
  **(campaña B, 2026-09-13, H26): cerrada la parte de `reports/`, D4=b.**
  Commit `7ee58ba` mueve los 5 ficheros a `evals/e1-read/reports/`, junto a
  los verdicts de esas campañas; `engine/src/main.rs` (comentario que citaba
  `reports/m2-07-impl-report.md`) actualizado en el mismo commit. Verificado:
  `git ls-files | grep reports/` da los 5 en su ruta nueva, ninguno en la
  raíz. **`docs/superpowers/` sigue sin resolver** — sigue en G5/sin dueño.

- [ ] **Residuos de entorno del plan** (ya listados allí, se repiten aquí para no
  perderlos): `crontab -r` pendiente de M1a · `reflex-baseline.sh` traga errores
  de `jq` con `2>/dev/null` · cachés huérfanas de reflex 0.6.0/0.8.0.

- [ ] **Decisión abierta: `archive/` en el ranking.** Es el 32%–39% del índice
  (54 de 138 notas). Se decide con la corrida de C5 delante o se cierra
  declarando que se queda indexado. Llevar la decisión abierta indefinidamente es
  peor que cualquiera de las dos opciones.

- [ ] **`kb-demo` como fixture por defecto en 8 ficheros de test.** Medido
  el 2026-09-01: `engine/tests/{buscador,config,escritor,indexer,inicia,nota,
  recall,recall_contenido}.rs` usan literalmente `"kb-demo"` como nombre
  de KB / permalink de partida en sus fixtures. En un repo que se publica, el
  nombre de la KB privada del autor no debería ser el fixture por defecto de
  la suite.
  **Acción:** renombrar a un fixture neutro (`kb-test`, ya en uso en algunos
  tests hermetizados de la Pista A, es candidato natural) antes de publicar.
  Deuda menor — no bloquea nada hoy.
  **Cruce (2026-09-09):** lo cierra —y lo mantiene cerrado— el mismo validador
  de nombres y rutas personales anotado en el item de Media «rutas personales
  y `hooks.json` sin validar en CI» (el item de `test-*.sh` que lo señalaba
  se cerró en la campaña B, ver `## Cerrado con evidencia`). **Lo cierra G5 si lo adopta.** **(2026-09-11: G5b cerró sin adoptarlo — release `v0.1.0` publicada, ver `## Cerrado con evidencia`. La marca queda huérfana: necesita dueño o campaña propia.)**

- [ ] **(pasada de coste 2026-09-09) Sinergias anotadas, dueño sin decidir: el
  mecanismo de guards de exo sirve para delegar I/O, y el método de evals de
  exo sirve fuera de exo.**
  Evidencia: `plugins/exo/hooks/hooks.json:3-27` ya cablea tres
  `PreToolUse:Bash` en producción (`git-c-bash.sh` en `:18`,
  `git-add-all-guard.sh` en `:22`, `verify-before-commit.sh` en `:26`); **no
  hay guard sobre el tamaño de un `Read`**. El patrón de delegación de I/O de
  Spotify (análisis fuera de este repo, árbol `proyectos/IA/pocs/`, §3.2) usa
  exactamente ese mecanismo: bloquear la lectura cara y redirigir a un worker
  barato, dejando pasar las lecturas dirigidas (`offset`/`limit`, pipes).
  Medido el 2026-09-09 sobre repos reales, el umbral de 350 líneas es un
  Pareto casi perfecto: en `backend-finnk` 4,4% de los ficheros concentran
  34,1% de las líneas; en `NorlinePlus` 8,8% concentran 48,7% (15.638
  ficheros, 2,5M líneas, máximo 16.675); en `frontend-web-privada` 3,4%
  concentran 25,6%. En la dirección contraria, el método de evals de este repo
  —gate pre-registrado (`evals/e1-read/gate.md`), `verdict/` separado del
  harness, atribución cruzada
  (`evals/retrieval-fase0/harness/atribucion-cruzada.py`)— es transferible a
  cualquier catálogo de skills, no solo al de exo.
  **Acción:** ninguna en exo por ahora — **esto queda anotado como sinergia,
  no como deuda**. Antes de construir hay que decidir dueño: el mecanismo de
  hooks vive aquí, pero el worker barato vive fuera (plataforma on-prem) y el
  catálogo de skills a evaluar también. Si se decide que exo asume delegación,
  esto sube a item propio con su gate; si no, se cierra como «no es de exo».

- [ ] **(G4c, Task 14) Gate de paridad `ratchet`+`targets`: pendiente de
  máquina Linux, mismo prerequisito.** Ninguno de los dos corre en W11 sin
  toolchain Go (ver "Residuo declarado",
  `docs/superpowers/plans/2026-09-09-g4c-ratchet-y-cutover.md:874-875`);
  comparten prerequisito — compilar kbx en `fe46443` — así que conviene
  correrlos juntos en la misma sesión Linux, no por separado. El repo `kbx`
  local está divergido de `fe46443` (`f0d0564`, 1 por delante y 18 por
  detrás, conflicto en `budget.go`, mismo plan:876-877): quien vaya a
  compilar `fe46443` para el gate necesita saberlo.

- [ ] **(G4c, Task 14) "Los nueve invariantes de la spec" era un lapsus —
  son siete.** El plan de G4a
  (`docs/superpowers/plans/2026-08-26-g4a-plomeria-y-targets.md:1614-1616`)
  hablaba de nueve; esa lista no existe en ningún documento del repo
  (adjudicación A1,
  `docs/superpowers/plans/2026-09-09-g4c-ratchet-y-cutover.md:71-92`). Lo que
  existe son los siete *"Invariantes portables — tests obligatorios
  (V13/H7)"* de `docs/superpowers/specs/2026-08-26-exo-generico-design.md:
  578-606` (la propia spec los llama así en `:705-706`); de los siete, solo
  el ítem 7 es del ratchet (cuatro sub-invariantes: sello borrado = subir a
  infinito, sello huérfano no lava una declaración, sello corrupto en HEAD
  es error no abstención, aritmética entera de aire). Anotado para que nadie
  vuelva a buscar la lista de nueve que no existe.

- [ ] **(G4c, Task 14) El cutover kbx→exo es parcial: `rotate`, `stale`,
  `history` y `diff-since` siguen sin portar.** Verificado hoy: `exo --help`
  no lista esos cuatro verbos.
  `plugins/exo/skills/distill/SKILL.md` sigue necesitando el binario `kbx`
  por `rotate` (`:60`), `stale` (`:146`) y `diff-since` (`:225`). `rotate` es
  el candidato natural a G4d, y no es cosmético: es el remedio que
  `kb-precommit.sh` prescribe en su mensaje de rechazo (`kbx rotate --kb <kb>
  --apply`) cuando el gate muerde — mientras no exista en `exo`, ese remedio
  sigue exigiendo tener `kbx` instalado.

- [ ] **(G4c, Task 14) Las 2 rutas `/home/paul/…` de
  `test-git-c-bash.sh:74-75` siguen ahí.** Fixtures de test hardcodeadas a
  `/home/paul/Documentos/proyectos/code-graph-go`, verificado hoy sin
  cambios. Deuda de higiene de test-fixtures personales — ya hay un item
  hermano en Media sobre «rutas personales y `hooks.json` sin validar en
  CI» (que ya cita este mismo script) que la cierra si se adopta.

- [ ] **(NUEVO, campaña B, 2026-09-13, medido en Task 10) `exo search` sin
  resultados no imprime nada y sale 0.** `busca_cmd` (`engine/src/main.rs`,
  rama sin `--json`): si `resultado.results` está vacío, el `for` no itera
  y la función vuelve `Ok(())` — ni un `no results` en stdout ni un aviso en
  stderr, exit 0 igual que con resultados. Contraste en el propio binario:
  `targets_cmd` (`main.rs:970-971`) sí distingue el caso vacío e imprime
  `no candidates` antes de salir. Un script que encadena `exo search` no
  puede diferenciar «sin resultados» de «no filtré la salida», y un humano
  ve una terminal en blanco sin saber si el comando corrió.
  **Acción:** alinear `busca_cmd` con el contrato de `targets_cmd` — un
  `no results` explícito (fuera del envelope `--json`, que ya distingue por
  `results: []`) cuando `resultado.results` está vacío.

---

## Cerrado con evidencia (para no re-proponer)

- [x] **Gate de shellcheck en CI, cada aviso arreglado o justificado en el
  sitio: cerrado el 2026-09-13 (campaña B, H11, `cd3bfff`).**
  `scripts/test-shellcheck.sh` descubre por índice de git (excluye `evals/`
  y `docs/`), corre `shellcheck -x -P SCRIPTDIR` sobre el bash versionado y
  falla si algún fichero avisa. En CI (`ci.yml`, job `lint`, `:51-58`) usa un
  binario de shellcheck 0.11.0 pineado por URL+SHA256, descargado a `/tmp` —
  la versión de la imagen del runner no decide el conjunto de avisos.
  **45 scripts en el árbol** (recon post-campaña A/T4/T5: sube de 39 a 45,
  no a 41 como estimaba el plan de B — 3 nuevos de A, 2 de B, 1 previo que
  faltaba en el recuento). De los 47 avisos originales, 11 ya tenían
  tratamiento decidido en el plan (10 `disable` puntuales + un fichero
  completo); dos hallazgos nuevos sobre ficheros que A tocó y la tabla del
  plan no cubría: `test-recall-latencia.sh` (mismo idioma de aserción que
  `test-a1-gate.sh`, mismo `disable` de fichero) y `test-exo-recall.sh`
  (`HOOK_RC` muerta — la asignación se borra, sin cambio de comportamiento).
  **Sin bugs reales pendientes de backlog**: los dos hallazgos nuevos son
  `disable` justificado y arreglo sin cambio de comportamiento; ninguno pedía
  una entrada nueva aquí.
  **Falsación medida:** un script `roto` con `SC2086` sin comillas puso el
  gate en rojo; revertido antes de este commit (nunca llegó a estar
  trackeado).

- [x] **Gate de exec-bit ampliado a scripts sin extensión con shebang:
  cerrado el 2026-09-13 (campaña B, H12, `4af859b`).**
  `git ls-files -s -- 'plugins/*.sh'` dejaba fuera del gate los scripts de
  `skills/orchestrate/scripts/` (`review-package`, `sdd-workspace`,
  `task-brief`): bash sin extensión, invocados por ruta. Con `task-brief` en
  modo `100644` (no ejecutable) el gate seguía saliendo `[OK]`, exit 0 — un
  script que rompería en el harness de Claude Code no lo detectaba.
  `scripts/test-exec-bit.sh` ahora recorre todo `git ls-files -s --
  plugins` y marca «script» cualquier ruta `*.sh`, o cuyo contenido —leído
  del índice de git, no del working tree— empiece por shebang. Corre en su
  propio job (`ci.yml`, `exec-bit`, `:74`).
  **Falsación medida:** `[FAIL]` con `task-brief` y con `_timeout.sh` en
  `100644`; `[OK]` tras restaurar el modo. Sin cambios de modo pendientes en
  el árbol.

- [x] **Los scripts `test-*.sh` de `plugins/exo/scripts/` no entran en CI.**
  Y `test-contrato-engine.sh` dependía del índice y la KB reales de esta
  máquina. **Los dos cerrados el 2026-09-12 (`d8aa3b6`).**
  `scripts/test-plugin.sh` descubre por glob TODOS los `test-*.sh` del
  plugin (un test nuevo entra solo, sin lista a mano), los invoca directos
  —ejerciendo también el bit de ejecución, como los llama el harness de
  Claude Code— y falla si el glob no encuentra nada (un glob roto no debe
  dar verde). Job `plugin-tests` en ubuntu/windows/macos (`ci.yml:138-154`):
  los hooks corren en los tres SO para los que se publican binarios. Local:
  10/10 antes de este commit, en rojo con `test-estilo-directo.sh` en modo
  `644` (el bug real que motivó la ola: ese test fallaba 0/2, exit 126, y el
  hook llegó roto a exo 1.1.1 porque ningún CI lo ejercía).
  `test-contrato-engine.sh` seguía atado a rutas de esta máquina
  (`C:/Users/paul/.exo/index.db`, `C:/proyectos/homework/kb-demo`) y se
  abstenía con exit 2 fuera de ella. `scripts/test-contrato-ci.sh` monta el
  fixture que pedía —KB semilla + commit + índice vía `exo init`, con
  `EXO_CONFIG` y `EXO_DB` temporales— y le pasa `EXO_INDEX`/`EXO_KB`
  explícitos; `EXO_DB` es obligatorio: `init` graba `db = ~/.exo/index.db`
  en la config nueva aunque `EXO_CONFIG` apunte a otro sitio, y sin `EXO_DB`
  la suite indexaba la semilla sobre el índice real de esta máquina (medido:
  pisa `meta.kb_root`). Corre como step del job `test`
  (`ci.yml:133-136`, `if: always()` para que un rojo del gate hermético no
  esconda un contrato roto detrás), que ya trae el binario de release y el
  modelo ONNX cacheados. `test-contrato-engine.sh` queda excluido del glob
  de `test-plugin.sh` (necesita el binario compilado) en favor del fixture.
  **Vivas, sin dueño, las dos sub-propuestas del item original** — ver
  `## Media`, «rutas personales y `hooks.json` sin validar en CI».

- [x] **Release `v0.1.0` — el binario que no existía: cerrado el 2026-09-11
  (G5b).** `install.sh`, `install.ps1`, `.github/workflows/release.yml` y
  `exo doctor` entraron en `main` con el PR #6 (`f001667`, 11:27). La release
  está publicada y tiene **seis artefactos**: `exo-x86_64-unknown-linux-gnu`,
  `exo-x86_64-pc-windows-msvc.exe`, `exo-aarch64-apple-darwin` y sus tres
  `.sha256`. Los instaladores resuelven `releases/latest`, verifican el
  SHA256 **antes** de copiar nada y abortan en macOS Intel y Linux ARM en vez
  de dejar un binario que no arranca.

  **No salió a la primera, y eso es la mitad del valor de esta entrada:**

  | Corrida | Disparo | Conclusión | Qué demuestra |
  |---|---|---|---|
  | `34584773325` | push del tag `v0.1.0` | failure | `shasum: command not found`, exit 127, en el runner `x86_64-pc-windows-msvc`. El workflow usaba `shasum -a 256` —y no `sha256sum`— con un comentario explicando que el segundo no existe en macOS: la elección resolvió macOS y rompió Windows. Falló **después** de pasar la suite y `publish` quedó `skipped`: ninguna release a medias. Fix en PR #7 (`360175c`) |
  | — | — | failure en local | `install.ps1` **rechazaba el binario de Windows de su propia release**: `sha256sum` firma con `*` delante del nombre y `shasum` con dos espacios, y el instalador solo aceptaba un formato. Fix en PR #8 (`8a86832`) |
  | `34586964204` | `workflow_dispatch` sobre `main` | success | 11m24s, seis artefactos publicados |

  Tercer tropiezo, registrado en el runbook: **el clippy de una máquina
  Windows no ve el código `#[cfg(unix)]`**, así que salía limpio en local con
  un `needless_return` que el CI de ubuntu sí caza (`45a8bc3`).

  **Verificado en W11 con el binario de la release** —no con el del
  `target/release` local—: diez filas de `exo doctor`, cero `fail`, búsqueda
  híbrida con resultados y el gate de la KB corriendo sin bloquear. El binario
  que había instalado era anterior a G5b y no tenía `doctor`. Queda pendiente
  la máquina Linux, que no bloquea. Detalle completo:
  `docs/superpowers/runbooks/2026-09-11-g5b-release-v0.1.0.md`.

  **Los tres fallos son la misma familia** —dos plataformas que discrepan en
  una herramienta, una que oculta código al linter— y **ninguno lo habría
  cazado un CI verde**: los cazó ejercer el artefacto publicado. Es el mismo
  argumento que sostienen los items abiertos de fixtures y de `test-*.sh`
  fuera de CI.

  **Lo que esta release NO cierra**: los seis items marcados «lo cierra / lo
  subsume G5». G5b se cerró sin adoptar ninguno — siguen abiertos y ahora sin
  campaña asignada.

- [x] **CI mínimo — el gate que faltaba: cerrado el 2026-09-02 (G5a).**
  `.github/workflows/ci.yml` corre en cada PR contra `main`: cinco jobs —
  `fmt + clippy` (`lint`), `MSRV declarada (1.95)` (`msrv`) y `test` en
  `ubuntu-latest`, `windows-latest`, `macos-latest`. Verificado contra la API
  de GitHub del PR #1 (`g5a-ci` → `main`), no de oídas:

  | Corrida | SHA | Conclusión | Qué demuestra |
  |---|---|---|---|
  | `33619260543` | `e378cbc` | success | los 5 jobs verdes en frío |
  | `33619930840` | `9958218` | failure | gate de **fmt** dispara; `clippy` queda `skipped` |
  | `33620326356` | `e378cbc` | success | verde de vuelta tras retirar la rotura |
  | `33620849572` | `5151872` | failure | gate de **clippy** dispara solo: `fmt --check` success, `clippy -D warnings` failure citando `ptr_arg` |
  | `33621187141` | `e378cbc` | success | verde 5/5 tras cerrar las Tasks 1-4 |
  | `33624081143` | `fee361d` | failure | los **cuatro jobs restantes** disparan, cada uno por su causa; `lint` verde |
  | `33624497824` | `9da4272` | success | verde final, 5/5 jobs |

  **Los cinco jobs se han visto rojos por su propia causa.** Hallazgo de la
  review final de rama: tras las cuatro tareas había **un gate demostrado y
  cuatro afirmados** — solo `lint` había fallado nunca. `test`×3 y `msrv` son
  precisamente los que ejercen lo que no se puede probar en local (Git Bash en
  Windows, el `cd` y el bit de ejecución del script en macOS/Linux, ONNX
  Runtime en `aarch64-apple-darwin`, la caché del modelo). Se cerró con una
  rotura única (`fee361d`) de dos causas distintas: un `#[test]` que hace
  `panic!` y `rust-version = "1.98"` en `engine/Cargo.toml`. Resultado medido
  en `33624081143` — `msrv` rojo citando `requires rustc 1.98`, los tres
  `test` rojos, y **`lint` verde en la misma corrida**, que de paso demuestra
  que los jobs son independientes. Rotura retirada con `reset --hard` +
  `--force-with-lease`; no está en la rama.

  **La falsabilidad se demostró en dos pasadas, no en una.** La primera
  rotura (`9958218`) tumbaba fmt y clippy a la vez; como los steps del job
  `lint` son secuenciales, `fmt --check` falló primero y `clippy -D warnings`
  quedó **`skipped`** — la mitad del gate en la que se invirtió toda la Task 2
  (los 12 avisos de clippy a cero) no se había visto disparar todavía. Hizo
  falta una segunda rotura, rustfmt-limpia y que solo violara clippy
  (`5151872`), para probar esa mitad (corrida `33620849572`). Un backlog que
  solo contara el verde final habría dejado ese hueco sin registrar.

  **Duración del job `test`, en frío → con caché** (`Swatinem/rust-cache@v2`
  + caché del modelo pineada por revisión): ubuntu `4m20s → 2m3s` · macos
  `5m57s → 3m6s` · windows `7m1s → 3m44s`. La caché recorta ~50% en los tres
  SO.

  **Añadido sobre lo pedido por el ítem original:** el job `msrv` corre
  `cargo check --all-targets --locked` bajo el toolchain **1.95.0** exacto y
  pasa en verde — la MSRV declarada en `engine/Cargo.toml` (`rust-version =
  "1.95"`) deja de ser una afirmación sin comprobar. De paso confirma que
  `as_chunks` (introducido en la Task 2 de esta misma ola) está disponible
  bajo 1.95 sin necesitar fallback.

  **Acción tomada:** workflow con `cargo fmt --check` + `cargo clippy
  --all-targets -- -D warnings` + `cargo check --all-targets --locked` (MSRV)
  + `./engine/scripts/test-hermetico.sh` (el gate hermético de la Task 1C, sin
  reinventar el comando de test) en los tres SO. `rust-version` ya estaba
  declarado en `Cargo.toml`; `LICENSE` en la raíz, ver el commit
  `a6a2a11`.

  **Matización (2026-09-02):** este gate hoy **notifica, no bloquea**. Paul
  decidió explícitamente no proteger `main` por ahora — no hay branch
  protection ni required status checks — así que un PR rojo se puede
  mergear igualmente. Ver el nuevo item de deuda en `## Media` («hoy el CI
  no bloquea nada»).

- [x] **Caché del modelo de embeddings: cerrado el 2026-09-02 (G5a).** El job
  `test` de `.github/workflows/ci.yml` cachea
  `~/.cache/huggingface/hub/models--jinaai--jina-embeddings-v2-base-es` con
  `actions/cache@v4` y clave `hf-jina-es-8e2d780d-${{ runner.os }}` — el sha
  del snapshot pineado (`8e2d780d…`, ya cerrado como ítem de este backlog),
  no la rama ni el commit, así que un acierto de caché no vuelve a subir
  nada. **Decisión: cachear, no marcar `#[ignore]`.** Las nueve suites
  (`indexer`, `buscador`, `recall_contenido`, `guarda_modelo`, `recall`,
  `refresca`, `cache_embeddings`, `rechazo_envelope`,
  `write_create_permalink`) siguen ejerciendo indexer y buscador de verdad en
  cada corrida — un CI que no los ejerce es verde sin significado. El coste
  es una descarga de ~615 MB en la primera corrida por SO (miss de caché);
  las siguientes son hit.

  **El hit de caché del modelo está verificado por log, no por el delta de
  duración.** El delta de duración medido arriba (frío → caché caliente,
  ~50% menos en los tres SO) **no aísla la caché del modelo**: la misma
  medición incluye `Swatinem/rust-cache@v2` cacheando `target/` de cargo, y
  en un crate que compila `rusqlite` bundled + `ort` + `fastembed` en
  `--release`, la recompilación domina esos minutos. Esa cifra se conserva
  como dato de coste total del job, no como prueba de la caché del modelo.
  La prueba real está en los logs de GitHub Actions del propio step de
  caché: en la corrida fría (`33619260543`), el paso «Caché del modelo
  jina-es (revisión pineada)» registra `Cache not found for input keys:
  hf-jina-es-8e2d780d-Linux` / `-Windows` / `-macOS`, y su paso Post
  `Cache saved with key: hf-jina-es-8e2d780d-{Linux,Windows,macOS}`; en la
  corrida caliente (`33621187141`) el mismo paso registra `Cache restored
  from key: hf-jina-es-8e2d780d-Linux` / `-Windows` / `-macOS`, los tres SO.
  Eso es lo que prueba el hit de caché del modelo — no el delta de
  duración.

- [x] **Privacy-pass + colapso de autoría (B1): cerrado el 2026-09-02.** Una
  sola pasada de `git filter-repo` sobre un clon fresco combinó `--mailmap`
  (colapsa cinco identidades de autoría a una), `--replace-text` +
  `--replace-message` (redacta contenido y mensajes de commit) y
  `--paths-from-file --invert-paths` (borra de la historia **35 ficheros** de
  corpora crudos derivados de la KB privada — el eval set de
  `evals/e1-read/` y `evals/retrieval-fase0/`). `--prune-empty auto` podó
  además los 2 commits que solo tocaban esos corpora: **278 commits antes de
  la pasada, 276 después**.
  **Los cuatro gates de publicación**, medidos rojo antes y verde después
  sobre tres superficies de fuga (contenido en diffs, mensajes de commit,
  objetos del repo) más identidades:
  **G1 = 3525 → 0 · G2 = 27 → 0 · G3 = 4724 → 0 · G4 = 5 identidades → 2**
  (`Paul Guerrero <pguerrerolinares@gmail.com>` de autor, `GitHub
  <noreply@github.com>` de committer conservado — el único commit hecho por
  la web UI).
  Suite verde tras la pasada: Σ 200 tests, 28 binarios, 0 fallos. Detalle
  completo, decisiones adjudicadas y el ensayo previo sobre clon desechable en
  `docs/superpowers/specs/2026-08-26-exo-generico-design.md` §B1.

- [x] **M5a-02 config propia: cerrado el 2026-08-26.** El engine arranca con
  `~/.exo/config.toml` (`engine/src/config.rs`), con precedencia
  `flag > env > config > error accionable` y sin fallback a basic-memory: la
  única lectura que sobrevive es `exo init --from-basic-memory`, explícita y
  borrable (`engine/src/inicia.rs`). Cierra de paso el disenso del gate M4 de
  este mismo backlog (ítem Media, «Barrer los hallazgos vivos del gate M4») —
  el prefijo de permalink sale de `[kb] name`, no de `kb.file_name()`.
  Verificado (ola 1A, Task 11, 2026-08-26):
  `grep -rn "basic-memory/config.json" engine/src/ | grep -v inicia.rs` sin
  salida, y `grep -rn "kb-demo" engine/src/ | grep -v '///' | grep -v '//'`
  sin salida. Las quince menciones restantes de "basic-memory" en
  `engine/src/` son históricas o de linaje de diseño (comentarios), revisadas
  una a una.

  **Corrección (review de pre-merge de la rama `ola1a-config-propia`,
  2026-08-26, cerrado en el mismo commit de este arreglo):** el cierre de
  arriba solo cubría el camino `write new` (`write_new_cmd`, que ya llamaba a
  `exo::nombre_kb()`). El camino `--create` de `write append`
  (`write_append_cmd`, `engine/src/main.rs`) se quedó fuera: seguía derivando
  el prefijo de `kb.file_name()` en vez de `exo::nombre_kb()`, así que
  `exo write append --create` con un `[kb] name` de config distinto del
  basename del directorio de `--kb` creaba el fichero con el prefijo
  equivocado. El grep de arriba (`kb-demo`) no podía detectarlo porque el
  bug no contiene esa cadena. Arreglado sustituyendo el `kb.file_name()` de
  `write_append_cmd` por `exo::nombre_kb()?` — el mismo mecanismo que
  `write_new_cmd`. Evidencia: `grep -rn 'file_name()' engine/src/main.rs`
  ahora solo devuelve el comentario histórico de la línea 483 (que documenta
  el propio cierre de M5a-02), sin ninguna llamada real a `file_name()` para
  derivar el prefijo de permalink. Cubierto además por un test de integración
  nuevo (`engine/tests/write_create_permalink.rs`) que monta un `[kb] name`
  distinto del basename del tempdir de la KB y comprueba el permalink real,
  tanto en el envelope como en el frontmatter del fichero creado en disco.

- [x] **Rot documental del README: cerrado el 2026-08-26.** El bloque de
  estado citaba "M0, M1a y M2 (E1 read) cerrados · M4 (E2 write) cerrado" sin
  mencionar la ola 1A de config propia. Actualizado en la Task 11 de la ola
  1A, con puntero a este backlog y a
  `docs/superpowers/specs/2026-08-26-exo-generico-design.md`. Llevaba dos
  campañas abierto (anotado ya en C5).

- [x] **Revisión de HuggingFace pineada: cerrado el 2026-08-22.** `repo_hf`
  (`lib.rs`) resuelve `jinaai/jina-embeddings-v2-base-es` contra el sha
  `8e2d780d…`, el snapshot que generó la línea base del eval; un modelo ajeno
  sigue cayendo a `main` pero el engine lo avisa por stderr. Anotado en la spec
  de fusión §4.6b. 2 tests unitarios vistos fallar primero.

- [x] **Modo mudo de `busca_hybrid`: cerrado el 2026-08-22.** `Busqueda` gana
  `avisos: Vec<String>` (aditivo, omitido cuando está vacío, `search_type`
  intacto porque lo comparan los scripts del eval). `avisos_cobertura_vector`
  compara `vectores` contra `trozos` y distingue arm INERTE (0 vectores) de
  cobertura PARCIAL (con cifras); corpus vacío no avisa, que es el falso rojo
  simétrico. Los avisos salen además por **stderr con y sin `--json`**, así que
  nunca contaminan el envelope y siempre se ven. 4 tests nuevos vistos fallar
  primero (`tests/buscador.rs`), 124 verdes en la suite. Se mantiene el
  contrato de Task 3 (0 vectores ⇒ 0 resultados, no error): avisa, no falla.
  **(campaña A, 2026-09-13):** el cierre era parcial — `exo recall` descartaba
  los avisos (H2). Reabierto y cerrado en la campaña A: `warnings`/
  `elapsed_s`/`refresh_s` en el envelope de recall y `engine-warning` en el
  log del hook.

- [x] **exo NO degrada a vector-hash como `empirica`** (2026-08-18, lectura de
  `buscador.rs` e `indexer.rs`): un fallo de embed sube por `?` con contexto
  (`indexer.rs:247`, `buscador.rs:241`) y aborta el comando con exit ≠ 0. No hay
  fallback silencioso a hash ni basura *válida* entrando al índice. **La mitad
  mala de la respuesta** es el modo mudo de `busca_hybrid`, promovido a item de
  prioridad alta arriba.

- [x] **Válvula de embed de query vía API de Jina: no se activa.** Era
  condicional a que el hybrid frío no bajara de p95 < 2 s, y la corrida de M2-09
  mide recall < 2 s. Queda anotada por si el corpus crece; nunca OpenAI ni otro
  modelo (rompería la atribución del eval). GPU sigue descartada: no ataca la
  latencia de arranque.

- [x] **Veto AGPL sostenido bajo verificación adversarial**: el consultor del
  gate M4 inspeccionó `escritor.rs` completo — Rust original, diseño replicado
  contra oráculos de la KB de producción, sin copia ni vendorizado posible
  (basic-memory es Python).

- [x] **Permalinks del frontmatter jamás regenerados**, verificado con `xxd` en
  el gate M4 y con paridad de corpus ∅ en M2-09 (138/138, 0 regenerados).

- [x] **La suite de tests no es hermética — depende de `~/.exo/config.toml`:
  cerrado el 2026-08-27 (ola 1C, Tasks 1–4).** El item citaba una cifra de
  partida de **7 suites / 59 tests**, medida en otra ola (1A, 2026-08-26) —
  **esa cifra es incorrecta para esta medición y no debe repetirse como si lo
  fuera**. La cifra real de partida de la ola 1C, medida el 2026-08-27 con
  `EXO_CONFIG` apuntando a una ruta inexistente y
  `cargo test --release --no-fail-fast`, es **`CARGO_EXIT=101`, 9 suites / 61
  tests en rojo** (`indexer` 19, `buscador` 16, `recall_contenido` 7,
  `guarda_modelo` 5, `recall` 5, `refresca` 4, `cache_embeddings` 3,
  `rechazo_envelope` 1, `write_create_permalink` 1). El cuello era de
  producción, no de los tests: cuatro puntos leen config global
  (`src/indexer.rs:99`, `src/lib.rs:200`, `src/lib.rs:286`,
  `src/buscador.rs:236`) y las 9 suites lo heredaban por ahí.
  **Acción tomada:** helper compartido `engine/tests/common/mod.rs::con_config`
  (Task 1) — monta un `config.toml` temporal, apunta `EXO_CONFIG` a él bajo un
  `Mutex` de proceso, y restaura el valor previo al salir. Las 9 suites
  (`write_create_permalink`, `rechazo_envelope` en Task 1; `indexer` en Task
  2a; `buscador` en Task 2b; `recall`, `recall_contenido`, `guarda_modelo`,
  `refresca`, `cache_embeddings` en Task 3) pasan a usarlo.
  **Cifra final**, verificada tras hermetizar las 9: con `EXO_CONFIG` a una
  ruta inexistente, `cargo test --release --no-fail-fast` da `CARGO_EXIT=0`,
  **169 passed, 0 failed** — idéntico al recuento con config real.
  **Gate anti-regresión (Task 4):** `engine/scripts/test-hermetico.sh` corre
  la suite entera con `EXO_CONFIG` a un fichero inexistente y falla si
  `cargo test` no sale 0 (sin tubería: mide el exit code de `cargo`
  directamente, no el del último comando de un pipe). Verificado falsable con
  un ciclo red-green real: revertido `engine/tests/indexer.rs` al commit
  anterior a su hermetización (`2f7d8ec541fa5b26b199d1323e7562753883509b`), el
  gate dio `EXIT_ROJO=1` citando `--test indexer` en el diagnóstico; restaurado
  el fichero (`restaurado OK`), el gate volvió a dar `EXIT_VERDE=0`. Este será
  el gate que consuma el CI de G5.

  **Alcance sincerado (2026-09-01):** esta hermeticidad es respecto a
  `~/.exo/config.toml`, no respecto al entorno completo. Queda una segunda
  dependencia sin cerrar: nueve de estas suites indexan cuerpos no vacíos, y
  eso carga el modelo ONNX de embeddings (~0,6 GB) vía `hf_hub`
  (`engine/src/indexer.rs:330` → `con_embedder_de_proceso`) la primera vez
  que corre en la máquina. En un runner de verdad limpio, sin caché de
  HuggingFace, la suite sigue en rojo — por esa razón, no por config.
  `engine/tests/smoke.rs:31` marca esa dependencia con `#[ignore]`; las nueve
  suites de indexado no siguen esa convención. Anotado como item nuevo del
  backlog, adjudicado a G5 — cerrado el 2026-09-02, ver arriba «Caché del
  modelo de embeddings».

  **El punto de encuentro nació rojo:** la primera corrida de la fusión de
  las dos pistas dio `HERMETICO=1`, no verde.
  `init_con_nombre_valido_produce_frontmatter_parseable_e_indexable` (nacida
  en la Pista B) lanzaba `exo index` como subproceso pasándole `--kb` y
  `--db` explícitos pero no `EXO_CONFIG`; bajo `test-hermetico.sh` el padre
  lleva esa variable a una ruta inexistente a propósito, el hijo la heredaba
  y moría leyendo la config de embeddings — ninguna pista podía verlo sola,
  porque cada una era verde en su propio worktree. Arreglado en `01225ff`
  (`.env("EXO_CONFIG", &config)` explícito en el test). Es el argumento
  entero a favor del punto de encuentro único: un fallo de composición
  invisible a cualquiera de las dos pistas por separado.
