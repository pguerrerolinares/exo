# Backlog de exo — deuda abierta

> Nota viva: deuda técnica y documental de exo con su siguiente paso accionable.
> **No duplica el plan de cierre** (`plans/2026-08-17-cierre-exo-m2-a-m5b.md`), que
> fija QUÉ campañas quedan y en qué orden. Esto es lo que está suelto: hallazgos
> de gate sin barrer y deuda encontrada fuera de campaña. Editar aquí, no
> duplicar. Cada item cita su evidencia; un item sin evidencia verificable no
> entra.
>
> Última revisión: **2026-09-02** (G5a — CI mínimo cerrado con evidencia,
> deuda nueva de la ola anotada).

## Estado

| | |
|---|---|
| **Cerradas** | C5 (M2-08+09, cierra E1 read) · C6 (M6, cutover del recall) · C7 (M4, write-path) |
| **Pendientes** | C8 (M3+M1b, cutover de skills) → C9 (M5a, MCP + config propia) → C10 (M5b, desinstalar basic-memory) |
| **Medido** | engine-hybrid **48/55** hit@5 vs bm-hybrid 39/55, mismo día, paridad de corpus ∅, recall <2s (`evals/e1-read/verdict/m2-09-corrida.md`) |
| **Tests** | 111 verdes / 0 rojos en la rama de M4, 98 en main previo (contados por el consultor del gate en esa ola; el CI que los corre solo llegó después, en G5a — 200 tests / 28 binarios, ver `## Cerrado con evidencia`) |

---

## Alta

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

- [ ] **`#[allow(clippy::too_many_arguments)]` en `escritor.rs` — la struct de
  parámetros que no se hizo aquí.** `escribe_nueva` toma 8 parámetros contra
  el umbral de 7 de clippy; declarado en `engine/src/escritor.rs:252` en vez
  de refactorizar, porque agrupar en una struct de parámetros toca el camino
  de escritura y sus tests, y G5a era una tarea de CI, no de refactor.
  **Acción:** introducir una struct de parámetros para `escribe_nueva` (y
  revisar sus llamadores y tests) cuando se toque ese camino por otra razón.
  Localizable con `grep -rn too_many_arguments docs/backlog.md` — el comentario
  de `escritor.rs` cita esta entrada por nombre.

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

## Baja

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
  | `33621187141` | `e378cbc` | success | verde final, 5/5 jobs |

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
  las siguientes son hit. Verificado con la duración medida arriba (frío →
  caché caliente, ~50% menos en los tres SO): el hit de caché es real, no
  solo teórico.

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
