# Backlog de exo — deuda abierta

> Nota viva: deuda técnica y documental de exo con su siguiente paso accionable.
> **No duplica el plan de cierre** (`plans/2026-08-17-cierre-exo-m2-a-m5b.md`), que
> fija QUÉ campañas quedan y en qué orden. Esto es lo que está suelto: hallazgos
> de gate sin barrer y deuda encontrada fuera de campaña. Editar aquí, no
> duplicar. Cada item cita su evidencia; un item sin evidencia verificable no
> entra.
>
> Última revisión: **2026-09-09** (re-verificación de los diez items de la
> revisión crítica externa contra el árbol de `f86167a`: **nueve siguen vivos
> y sin tocar**, uno caducó a medias —la MSRV— y tres traían cifras ya
> movidas. Corregido in situ; los items retocados lo dicen en su cabecera.
> De paso: esta cabecera venía **rota del merge `f86167a`** —el bloque del
> 09-04 perdió su prefijo `> Última revisión:` y su `>`, y se salía del
> blockquote—, arreglada aquí).
>
> Anterior: **2026-09-04** (revisión crítica externa del repo completo:
> diez items nuevos marcados «(revisión 2026-09-04)», tres de ellos en Alta;
> ninguno duplica los que ya estaban — `test-*.sh` fuera de CI,
> `test-contrato-engine.sh` atado a esta máquina, aliases españoles y
> `kb-demo` en los tests del engine ya tenían entrada y se dejan como están).
> Antes: **2026-09-02** (G5a — CI mínimo cerrado con evidencia, deuda nueva
> de la ola anotada).

## Estado

| | |
|---|---|
| **Cerradas** | C5 (M2-08+09, cierra E1 read) · C6 (M6, cutover del recall) · C7 (M4, write-path) |
| **Pendientes** | C8 (M3+M1b, cutover de skills) → C9 (M5a, MCP + config propia) → C10 (M5b, desinstalar basic-memory) |
| **Medido** | engine-hybrid **48/55** hit@5 vs bm-hybrid 39/55, mismo día, paridad de corpus ∅, recall <2s (`evals/e1-read/verdict/m2-09-corrida.md`) |
| **Tests** | 111 verdes / 0 rojos en la rama de M4, 98 en main previo (contados por el consultor del gate en esa ola; el CI que los corre solo llegó después, en G5a — 200 tests / 28 binarios, ver `## Cerrado con evidencia`) |

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
  **Acción:** (a) corregir §7 de `arquitectura.md` y el item de hermeticidad;
  (b) alinear las tres versiones (o documentar por qué el plugin versiona
  aparte del engine); (c) añadir al `verify` de cierre un grep de las
  afirmaciones de estado más frágiles («Sin CI», recuento de tests,
  versiones) contra el árbol real.

- [ ] **(revisión 2026-09-04) «exo genérico» sigue siendo el plugin de Paul
  para Paul.** Medido el 2026-09-04 sobre `plugins/exo/`: la cadena `Paul`
  aparece en 4 ficheros vivos del plugin (`skills/distill/SKILL.md` ×7,
  `scripts/recall-inject.sh` ×2, `scripts/git-add-all-guard.sh`,
  `scripts/kb-precommit.sh`); `kb-demo` en 8 ficheros del plugin, dos de ellos
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
  **Acción:** (a) sustituir «Paul» por «el usuario»/«el dueño de la KB» y
  `kb-demo` por el nombre resuelto vía `exo config` en los cuatro scripts y
  dos skills; (b) o bien portar a exo lo que `distill` necesita de `kbx`
  (G4 ya empezó por `targets`), o bien declarar `kbx` como dependencia
  opcional en `instalacion.md` y hacer que `distill` se abstenga entera sin
  él; (c) la release con binario de G5 es el prerequisito de todo lo demás.

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

## Media

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

- [ ] **(revisión 2026-09-04) El coste del hook completo en Windows no está
  medido; solo el del binario.** `plugins/exo/hooks/hooks.json` cablea
  **tres** scripts bash en cada `PreToolUse:Bash` (`git-c-bash.sh`,
  `git-add-all-guard.sh`, `verify-before-commit.sh`) y uno en cada
  `UserPromptSubmit` (`recall-inject.sh`) que lanza `exo recall --refresh`,
  `exo config --json` y del orden de seis invocaciones de `jq`/`sed`/`tr`.
  Las cifras publicadas («~10 ms», `exo-recall.sh` cabecera; «~25 ms sin
  cambios», `exo-index.sh`) miden el binario, no el hook: bajo Git Bash cada
  spawn de proceso cuesta decenas de milisegundos, así que el coste real por
  prompt y por comando Bash en Windows es desconocido.
  **Acción:** instrumentar `_reflex-log.sh` con la duración del hook (o
  medir a mano con `time` sobre un `INPUT` real) en Windows y Linux, y
  publicar la cifra en `plugins/exo/README.md`. Si el `PreToolUse:Bash`
  triple supera ~200 ms, fusionar los tres scripts en uno con un único
  parseo del JSON de entrada.

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
  **Acción:** `engine/rust-toolchain.toml` con `channel = "stable"` o la
  MSRV, para que rustup lo resuelva solo. Anotar el requisito en
  `instalacion.md` §1 con la salida exacta del error para que sea googleable.
  (El `rustup update stable` de la acción original ya está hecho.)

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

- [ ] **Los scripts `test-*.sh` de `plugins/exo/scripts/` no entran en CI.**
  Medido 2026-09-02: hay **10** scripts `test-*.sh` en ese directorio, de los
  que **5 referencian rutas de esta máquina** —
  `grep -nE 'paul|C:[/\\]Users|/home/[a-z]+/' plugins/exo/scripts/test-*.sh`
  marca `test-a1-gate.sh`, `test-compose-inject.sh` (ambos vía el default
  `$HOME/.claude/...` de `a1-gate.sh`/`compose-inject.sh` cuando no se
  sobreescribe por variable de entorno), `test-contrato-engine.sh`
  (`C:/Users/paul/.exo/index.db`, `C:/proyectos/homework/kb-demo`
  hardcodeados), `test-git-c-bash.sh`
  (`/home/paul/Documentos/proyectos/code-graph-go`) y
  `test-subagent-inject.sh` (mismo default de `$HOME/.claude` que
  `test-a1-gate.sh`, vía `subagent-inject.sh`). El engine ya tiene CI
  (`.github/workflows/ci.yml`); la capa thin — hooks y scripts, la mitad del
  producto — sigue sin gate automático.
  **Acción:** fixture propia por script (índice + KB + `$HOME` de prueba,
  igual que ya hacen los otros 5 de esta misma carpeta) antes de cablear un
  job de CI para `plugins/exo/`.

- [ ] **Retirar los aliases españoles del CLI en 1.1.** Los diez flags
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

- [ ] **`test-contrato-engine.sh` depende del índice y la KB reales de esta
  máquina.** Rutas cableadas (`C:/Users/paul/.exo/index.db`,
  `C:/proyectos/homework/kb-demo`, ver cabecera del script). Se abstiene
  con exit 2 si faltan — no miente sobre lo que no pudo comprobar — pero eso
  significa que en un runner limpio (o en la máquina de cualquier otra
  persona) esta suite no corre nunca.
  **Acción:** CI necesita un fixture propio (índice + KB de prueba mínimos)
  para que la suite deje de abstenerse fuera de esta máquina.

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

- [ ] **Dos endurecimientos del CI que se decidieron NO aplicar en G5a, y por
      qué.** Hallazgos Minor de la review final de rama; se anotan para que la
      omisión sea una decisión y no un olvido.
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

## Baja

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
  El item de Alta sobre deriva documental es el síntoma: el volumen ya
  supera lo que se mantiene sincronizado a mano. No es deuda técnica en sí;
  es una decisión sin tomar que genera deuda. Si exo es una herramienta
  personal, el proceso (consultorías, gates, runbooks por cutover) está
  sobredimensionado y conviene congelarlo. Si aspira a usuarios, la
  prioridad es la release de G5 y purgar lo personal (item de Alta), no más
  documentación.
  **Acción:** escribir la respuesta en el README en dos frases («para quién
  es exo hoy») y derivar de ella qué carpetas de `docs/superpowers/` pasan a
  archivo histórico. Se cruza con el item «Nombres y ubicaciones» de abajo.

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

- [ ] **Nombres y ubicaciones.** `docs/superpowers/` como carpeta de docs del
  proyecto cuyo objetivo declarado es jubilar superpowers, y `reports/` colgando
  de la raíz fuera de toda convención (los verdicts sí viven ordenados en
  `evals/*/verdict/`). **Acción:** decidir de una vez — renombrar o escribir por
  qué se queda. Barato ahora, caro cuando haya más ficheros.
  **Actualización (G2, fusión de plugins):** resuelta la incoherencia de
  nombres para `plugins/` — ya no hay `process`/`reflex`, hay un único
  `plugins/exo/`. Quedan vivas como deuda sin resolver `docs/superpowers/` y
  `reports/`; se abordan en G5.

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

---

## Cerrado con evidencia (para no re-proponer)

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
