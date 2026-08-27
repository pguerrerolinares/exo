# Backlog de exo — deuda abierta

> Nota viva: deuda técnica y documental de exo con su siguiente paso accionable.
> **No duplica el plan de cierre** (`plans/2026-08-17-cierre-exo-m2-a-m5b.md`), que
> fija QUÉ campañas quedan y en qué orden. Esto es lo que está suelto: hallazgos
> de gate sin barrer y deuda encontrada fuera de campaña. Editar aquí, no
> duplicar. Cada item cita su evidencia; un item sin evidencia verificable no
> entra.
>
> Última revisión: **2026-08-18** (apertura — valoración del repo completo).

## Estado

| | |
|---|---|
| **Cerradas** | C5 (M2-08+09, cierra E1 read) · C6 (M6, cutover del recall) · C7 (M4, write-path) |
| **Pendientes** | C8 (M3+M1b, cutover de skills) → C9 (M5a, MCP + config propia) → C10 (M5b, desinstalar basic-memory) |
| **Medido** | engine-hybrid **48/55** hit@5 vs bm-hybrid 39/55, mismo día, paridad de corpus ∅, recall <2s (`evals/e1-read/verdict/m2-09-corrida.md`) |
| **Tests** | 111 verdes / 0 rojos en la rama de M4, 98 en main previo (contados por el consultor del gate, no por CI — no hay CI) |

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

- [ ] **CI mínimo — el gate que falta.** No hay `.github/`, ni `rustfmt.toml`, ni
  `clippy.toml`, ni `rust-version` en `Cargo.toml` (el plan afirma MSRV ≥ 1.97 y
  nada lo comprueba), ni `LICENSE` en la raíz. Todo lo verifica un consultor a
  mano, por gate. Verificado 2026-08-18: **el crate no compila en el portátil de
  trabajo** — `cargo check --all-targets` falla con
  `cc-rs: failed to find tool "gcc.exe"` (rusqlite bundled + sqlite-vec exigen
  toolchain C, ausente en el target GNU de esa máquina). Nada verifica el build
  fuera de la máquina de desarrollo.
  **Acción:** workflow con `cargo test` + `fmt --check` + `clippy -D warnings`,
  declarar `rust-version`, LICENSE en raíz. En un proyecto cuya tesis es "gates y
  evidencia", este es el gate más barato de todos.

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
  Cerrados en `acb312e`: traversal por `..` en `--dir`/`--titulo`, `--force` sin
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

- [ ] **Privacy-pass — va en la MISMA pasada de `filter-repo` que B1.** Las
  queries del eval son reales y están trackeadas; el snapshot del log está en
  `.gitignore`, el eval set no. Spec §6.6, espíritu clean-room.
  **Ampliado y resecuenciado el 2026-08-27** (adjudicación de B1, ver la spec
  de exo genérico): el alcance no es solo `evals/`. Hay **35 ficheros
  trackeados** que mencionan «empresa-x» — 3 correos literales en
  `specs/2026-08-26-exo-generico-design.md` y en
  `runbooks/2026-08-24-integracion-equipo-trabajo-windows.md`, y el resto
  corpora de evals derivados de la KB.
  **Por qué comparte pasada con B1:** `--mailmap` corrige la autoría, no el
  contenido. Si el privacy-pass se hace después editando ficheros en HEAD,
  esos strings siguen vivos en los diffs históricos y el gate
  `git log --all -p | grep -ci 'empresa-x\|universidad\|dev'` → 0 no puede
  pasar nunca. Y dos pasadas de filter-repo remapean dos veces los 137 SHAs
  citados en prosa. Una sola pasada: `--mailmap` + reescritura de contenido.
  **Es gate duro de publicación**: ningún push público antes de que ese grep
  dé 0.

---

## Cerrado con evidencia (para no re-proponer)

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
  (`write_create_permalink`, `rechazo_envelope` en Task 1;  `indexer` en Task
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
  anterior a su hermetización (`8d512fc2bd7a30fdde1c08cdfdce311b18b566a6`), el
  gate dio `EXIT_ROJO=1` citando `--test indexer` en el diagnóstico; restaurado
  el fichero (`restaurado OK`), el gate volvió a dar `EXIT_VERDE=0`. Este será
  el gate que consuma el CI de G5.
