# Campaña D — Cutover completo kbx→exo: `rotate` + `stale`, y paridad pendiente

> **For agentic workers:** REQUIRED SUB-SKILL: usa `exo:orchestrate` para
> ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`) para
> tracking. Sesión-fábrica: `paul-profile:fabrica` con
> `.superpowers/fabrica/config.md` (el orquestador lo integra; este plan no
> lo edita). Gates de merge: consultor Fable con verdict commiteado
> (`.superpowers/fabrica/config.md`, «Ejecución de gates»).

**Goal:** que `exo rotate` y `exo stale` existan en el engine Rust con la
semántica de `kbx` `fe46443`, que los gates de paridad de `targets` y
`ratchet` —pre-registrados y nunca corridos— se ejecuten y adjudiquen, y que
ningún consumidor del plugin (`distill`, `kb-precommit.sh`, docs) invoque ya
el binario `kbx`.

**Architecture:** dos frentes independientes que convergen al final. (1)
**Gates existentes**: instalar Go, compilar kbx `fe46443` y rellenar los dos
registros de paridad ya escritos (`targets` de G4a, `ratchet` de G4c) con el
dato real — cero código nuevo, solo ejecutar y adjudicar. (2) **Port**:
`rotacion.rs` (rotate) y `obsolescencia.rs` (stale), cada uno con su propio
pre-registro (`2026-09-14-campana-d-preregistro-paridad-rotate-stale.md`,
congelado por commit antes de tocar `engine/src/`) y su propio gate de
paridad. Los dos frentes no se pisan en fichero y casi no comparten
dependencia: el port no necesita que Go esté instalado para escribirse y
testearse (TDD puro, sin kbx de por medio), solo su GATE de paridad lo
necesita. Por eso el orden del plan deja el port en su propio carril, fuera
del camino crítico de "¿hay Go en esta máquina?". Al final, un tercer
frente pequeño —reapuntar `distill`/`kb-precommit.sh`/docs y cerrar el
backlog— depende de que el port exista, no de que los gates hayan corrido.

**Tech Stack:** Rust edition 2024, MSRV 1.95 · `serde`/`serde_json` ·
`anyhow` · `clap` 4.6.2 derive ·
`regex` 1.13.1 (ya en el árbol; se usa también en variante `regex::bytes`) ·
`tempfile` 3.14 (dev, sin subir a producción — ver Task 5). **Sin
dependencias nuevas y sin cambio de features en `Cargo.toml`.**

## Estado de verificación de las afirmaciones del dictamen (2026-09-14, contra `b2020f7`)

El dictamen del consultor (`/home/paul/.claude/jobs/05ee55a9/tmp/dictamen-consultor-d-e.md`)
es correcto en el diagnóstico general. Tres afirmaciones puntuales, releídas
contra el código real de hoy, **no se sostienen** y cambian el plan:

- **Las dos "Important" de G4a que el brief pedía dejar abiertas YA ESTÁN
  CERRADAS**, adjudicadas por Paul el 2026-09-04 (`docs/superpowers/plans/2026-09-04-g4b-budget-y-lint.md:73-128`,
  decisiones A1 y A2) e implementadas desde entonces:
  - **A1 — whitespace del `tier`**: `engine/src/frontmatter.rs:54-60` usa
    `ESPACIOS_ASCII` (6 caracteres ASCII), **no** `char::is_whitespace()`.
    El comentario en el propio código lo dice: *"ASCII a propósito, no
    `char::is_whitespace()`: […] Degradación hacia rojo, como el resto del
    módulo (A1 del plan de G4b)"*. Esto **elimina** la divergencia 3 del
    registro de G4a (`2026-09-02-g4a-preregistro-targets.md:63-70`) —esa
    nota describe un estado del código que ya no existe—, tal y como el
    propio plan de G4b lo anuncia en su línea 319: *"A1 (ASCII), que
    elimina la divergencia 3 del registro de G4a"*.
  - **A2 — `targets` sobre una KB sin git**: `engine/src/objetivos.rs:129-136`
    comprueba `gitx::es_repo_git` **una vez, antes del bucle**, y falla con
    un mensaje accionable (exit 1, "no está versionada… corre `git init`").
    Está probado (`engine/tests/targets_cli.rs:104-126`,
    `una_kb_sin_git_da_un_error_accionable_y_no_un_fallo_por_fichero`).
    **Sí distingue** la condición — la otra mitad de la pregunta original
    ("o se declara que exige una KB versionada") queda como nota de
    documentación de coste casi cero, no como decisión de comportamiento.
  - Este plan **no reabre A1 ni A2**. La Task 3 (gate de `targets`) los cita
    como ya resueltos y espera **cero divergencia** de `tier` en la corrida
    real — si aparece alguna, es una regresión, no una divergencia conocida.
- **`engine/src/doctor.rs` no menciona `kbx` en ningún sitio hoy**
  (`grep -n kbx engine/src/doctor.rs` → vacío, verificado). El dictamen dice
  "`doctor`: kbx deja de ser dependencia mencionada" como si hubiera una
  mención que quitar; no la hay en el código. Lo que sí menciona `kbx` es la
  **prosa** de `plugins/exo/skills/distill/chequeos.md:39-40`, que describe
  un check `schema_drift` de `exo lint` que **ya no existe** (murió en G4b,
  A7: *"`schema_drift` muere aquí… existía porque kbx y exo eran dos
  binarios contra un schema compartido; con un solo binario deja de tener
  objeto"*, `engine/src/lint.rs:1-5`). Es documentación caducada de un check
  que no está, no solo una mención de `kbx` de más — se corrige en la Task 9,
  no en `doctor.rs`.
- **La nota "kbx local divergido" del backlog está CADUCADA.** Verificado
  hoy: `git -C ~/Documentos/proyectos/kbx merge-base --is-ancestor fe46443 HEAD`
  sale 0 — `fe46443` **es ancestro** de `HEAD` (`ee2b27c`) del checkout local.
  La nota en `docs/backlog.md:1386-1389` (*"El repo `kbx` local está
  divergido de `fe46443`… conflicto en `budget.go`"*) describe un estado de
  hace dos sesiones; hoy no hace falta reconciliar nada para compilar
  `fe46443` — está en la historia del checkout tal cual. Se corrige en la
  Task 10, con la cita del comando que lo demuestra.

Una afirmación del dictamen se **confirma** y merece nota aparte porque no
estaba en ninguno de los documentos que cita: **el `permalink` que
`rotate.Apply` escribe en el archivo tiene `"wisdom-paul/"` hardcodeado**
(`apply.go:159`, `internal/rotate/apply.go` de `fe46443`). Es nuevo —nadie lo
había mirado porque `rotate` nunca se leyó línea a línea hasta esta
campaña— y es exactamente la clase de cosa que "cambia comportamiento
observable" que el brief pedía declarar. Ver Decisión D-4.

## Decisiones D-n abiertas (PENDIENTE-PAUL → PENDIENTE-CONSULTOR)

Ninguna la toma la fábrica sin más. Por el régimen de `config.md`
("Ejecución de gates"), una decisión sin fuente citable la intenta primero
el consultor delegado antes de escalar a Paul — así que "abierta" aquí
significa "con recomendación y trade-offs escritos", no "bloqueada hasta que
Paul conteste".

### D-3 — Exit code de `exo rotate` cuando falla una nota individual de la barrida · no bloquea ninguna task

`kbx rotate` sigue barriendo `log/` aunque una nota falle (frontmatter sin
cerrar, fichero ilegible) y sale **2** al final si hubo algún fallo —
acumula, no aborta. El port hace lo mismo (sigue la barrida) pero necesita
decidir qué exit code final usar en exo, donde 2 es "error de uso de clap"
(reservado, nunca lo elige código de aplicación) y 3 es `GateFallido`
(reservado a "la KB está mal", un veredicto de negocio que `budget`/`lint`/
`ratchet` calculan sobre un árbol íntegro).

| Opción | Qué | A favor | En contra |
|---|---|---|---|
| **a — exit 1** | Igual que cualquier otro `anyhow::bail!` de exo | Un fichero que no se pudo procesar es un error de sistema/IO, no una decisión de negocio sobre la KB — la clase de `GateFallido` no encaja; consistente con el resto del binario | Diverge del `2` de kbx (declarado, no gatea la paridad — ver pre-registro) |
| **b — exit 3 (`GateFallido`)** | Tratar "N notas fallidas" como un hallazgo de gate | Uniforme con budget/lint/ratchet en que "3 = algo requiere tu atención" | `GateFallido` en main.rs está documentado para "la KB está mal", no para "no pude leer un fichero"; mezclar los dos diluye la distinción que el resto del binario ya mantiene |

**Recomendación: a.** Es la Task 5 quien la aplica: `rotate_cmd` termina con
`anyhow::bail!` tras imprimir cada fallo por stderr, que en `main()` cae en
la rama genérica (exit 1). Declarada en el pre-registro (divergencia 1 de
§Rotate). Si Paul prefiere (b), es un cambio de una función (`GateFallido`
en vez de `bail!`) revisable después del merge — no vale la pena bloquear
el port por esto.

### D-4 — El `permalink` que escribe `rotate` en el archivo: `"wisdom-paul/"` fijo (kbx) vs `exo::nombre_kb()` (genérico) · bloquea el Step de `construye_archivo`/`aplica` en la Task 5

`internal/rotate/apply.go:159`: `permalink := "wisdom-paul/" + …`, literal.
kbx nació para esta KB concreta y nunca tomó un nombre de proyecto como
parámetro — no hay bug que reproducir, es una constante de dominio del
propio kbx.

| Opción | Qué | A favor | En contra |
|---|---|---|---|
| **a — verbatim** | Hardcodear `"wisdom-paul/"` en `construye_archivo` | Paridad byte a byte sin condiciones con kbx | exo sirve **cualquier** KB (`--kb` apunta a cualquier ruta); escribiría un permalink falso en cuanto alguien use exo sobre una KB que no se llame `wisdom-paul` — exactamente el acoplamiento que el backlog («"exo genérico" sigue siendo el plugin de Paul», `docs/backlog.md:243-296`) ya tiene abierto como Alta |
| **b — `exo::nombre_kb()`** | Usar `[kb] name` de la config, el mismo mecanismo que ya usan `write new`/`write append` (`engine/src/escritor.rs:279`, `engine/src/main.rs:752`) | Cero código nuevo (función ya existe y ya está probada); coherente con el resto del binario; no reintroduce el acoplamiento | El campo `permalink` del archivo deja de ser byte-idéntico a kbx en la comparación literal — hay que excluirlo del criterio estricto del gate (ya declarado en el pre-registro) |

**Recomendación: b.** Ya aplicada en el código que trae la Task 5 (usa
`exo::nombre_kb()`) y ya declarada como divergencia 6 en
`2026-09-14-campana-d-preregistro-paridad-rotate-stale.md`. Si Paul prefiere
(a), el cambio es una línea en `aplica` (pasar un `&str` literal en vez de
`nombre_kb()?`) y una reversión de esa declaración — tampoco bloquea, la
Task 5 ya trae la recomendación implementada; esto queda abierto solo para
que quede escrito y Paul pueda vetarlo con conocimiento, no porque haga
falta esperar.

### Nota de documentación, no D-n — precondición "KB versionada" de A2

A2 (arriba) ya implementa el comportamiento; lo único que falta, y es coste
casi cero, es una línea en `docs/instalacion.md` o `docs/arquitectura.md`
que declare que `targets`/`ratchet`/`stale` exigen que la KB sea la raíz de
un repo git. Se pliega en la Task 9 (no merece su propia task).

## Global Constraints

Se heredan de G4a/G4b/G4c y no se repiten salvo donde esta campaña las
ejerce o las cambia:

- **El crate vive en `engine/`.** No hay workspace de Cargo. Todo comando
  `cargo` se ejecuta con cwd `engine/`.
- **Fuente del port: `fe46443`**, leído con
  `git -C ~/Documentos/proyectos/kbx show fe46443:<path>` — **nunca**
  checkout ni cambio de rama en el repo `kbx`. Confirmado ancestro de
  `HEAD` (`ee2b27c`) del checkout local: `git merge-base --is-ancestor
  fe46443 HEAD` sale 0.
- **`SCHEMA_VERSION` sigue en 2**, sin tocar. `rotate` y `stale` son
  `command` nuevos: aditivo, no breaking.
- **Claves JSON en inglés, identificadores Rust en castellano** (D7/D8),
  `#[serde(rename)]` en cada campo que difiera. Comentarios en castellano.
- **Avisos y progreso a stderr, resultado primario a stdout.** Con
  `--json`, stdout lleva el envelope y nada más.
- **Errores con `anyhow`**, `.context(...)` accionable en cada IO/parse
  falible. No se añade `thiserror`.
- **`tempfile` sigue siendo dev-dependency.** El port de las escrituras
  atómicas de `rotate` (Task 5) **no** la usa en código de producción —
  se implementa con `std::fs` puro (nombre temporal manual + `rename`),
  precisamente para no subir `tempfile` de dev a runtime sin necesidad.
- **Sin features nuevos de `serde_json`.** El feature de precisión
  arbitraria que haría falta para que el `score` de `stale` emita 2
  decimales fijos rompe `#[serde(untagged)]` con floats en todo el grafo de
  dependencias (`tokenizers` lo usa en 12 sitios) — no compensa para la
  paridad de valor. `Puntuacion` (Task 7) serializa como un `f64` normal —
  el texto puede diferir de kbx (`29.00` vs `29.0`), el valor no; decisión
  y motivo completos en la divergencia 5 del pre-registro de rotate/stale.
- **Sin crate de fechas.** exo no trae `chrono` ni `time` — ni falta:
  `indexer::git_epoch_de` ya resuelve el mismo problema pidiéndole a git el
  epoch directamente (`%at`). `stale` necesita además la cadena ISO-8601
  para el campo `last_commit` (que reutiliza `gitx::ultimo_commit`, `%aI`),
  así que Task 7 trae un parser/formateador ISO-8601↔epoch autocontenido
  (algoritmo civil-días de Howard Hinnant, dominio público, sin
  dependencia) en vez de añadir un crate para dos funciones puras de
  aritmética de calendario.
- **Clippy es gate duro**: `cargo clippy --all-targets --locked -- -D
  warnings` limpio, y `cargo fmt --check`, en cada commit que toque
  `engine/src/` o `engine/tests/`.
- **Git**: `git add <rutas explícitas>`, nunca `-A`/`.`; `git -C <path>`,
  nunca `cd <path> && git`. Push, merge a `main` y borrado de ramas
  remotas son línea roja de Paul — nunca los ejecuta la fábrica.
- **`$TMPDIR`** en los comandos de este plan es un directorio fuera del
  repo: `export TMPDIR="${TMPDIR:-$(mktemp -d)}"` al abrir la sesión del
  ejecutor. Nada de lo que va ahí se commitea.
- **Una task bloqueada por una decisión PENDIENTE-PAUL sin recomendación
  aplicable queda `encolado`** (config de fábrica). Ninguna task de este
  plan cae en ese caso: D-3 y D-4 traen recomendación ya aplicada en el
  código que trae su propia task, así que nada espera a Paul para avanzar.
- **Ninguna task escribe en `~/.local/bin`.** El `kbx` de `fe46443` se
  compila a `/tmp/campana-d/kbx` (Task 2, Step 3, `go build -o`, nunca
  `make install`); las Tasks 3/4/6/8 lo invocan como `"$KBX"`. Para probar
  el `exo` de esta rama se usa `engine/target/release/exo` por ruta — la
  Task 9 no copia nada a `~/.local/bin/exo`.

## Dependencias con la campaña E y conflictos de fichero

**Orden de merge: E primero** (más corta), **D se rebasa sobre E después**.
E toca:

| Fichero | Qué toca E | Qué toca D | Conflicto real |
|---|---|---|---|
| `engine/src/main.rs` | `busca_cmd` (mensaje `no results`); el texto del mensaje de la guarda «una DB sirve a una KB» | Bloque `Comando` (dos variantes nuevas, `Rotate`/`Stale`), `quiere_json` (dos brazos nuevos), `ejecuta` (dos brazos nuevos), dos `ArgsRotate`/`ArgsStale` nuevos, dos funciones `rotate_cmd`/`stale_cmd` nuevas | **Ninguno textual**: E edita `busca_cmd` (línea ~932 a fecha de este plan) y el mensaje de la guarda (dentro de `init_cmd`/`valida_db_para_kb`, en otra zona del fichero); D solo **añade** bloques nuevos junto a `Targets`/`Budget`/`Lint`/`Ratchet`/`Doctor` (la zona de `enum Comando`, ~L38-85; `quiere_json`, ~L431-448; `ejecuta`, ~L484-502; los `ArgsX` tras `ArgsRatchet`, ~L369-385) y una zona nueva de funciones `_cmd` junto a `ratchet_cmd`/`ratchet_seal_cmd` (~L1270-1367). Un rebase de D sobre E en este fichero es mecánico: git-merge de dos regiones no solapadas del mismo fichero, sin edición manual esperada. Si el rebase automático falla, es señal de que E se movió de sitio más de lo que este plan anticipa — revisar contra el plan de E antes de resolver a mano. |
| `plugins/exo/scripts/{subagent-inject,exo-recall,test-exo-recall,test-git-c-bash,test-hermetico}.sh` | Las cinco | D no las toca | Ninguno |
| `.github/workflows/release.yml` | Sí | D no la toca | Ninguno |
| `engine/rust-toolchain.toml` | La crea | D no la toca | Ninguno |
| `docs/backlog.md` | Cierra sus propios items (H27-ish de E) | Task 10 cierra los suyos (gates pendientes, cutover parcial, "exo genérico" acción b, nota "kbx divergido" caducada) | **Sí hay conflicto de líneas si las dos tocan el mismo bloque de la tabla de Estado o secciones adyacentes.** Mitigación: D corre su Task 10 **después** de que E haya mergeado (orden de merge ya fijado arriba), así que D edita sobre el backlog ya tocado por E — el orquestador re-localiza cada item de D **por su texto**, no por número de línea (igual que el dictamen ya advierte), precisamente porque E lo habrá desplazado. |

D **no** toca ningún fichero de `plugins/exo/scripts/` que E toque; D toca
`plugins/exo/skills/distill/{SKILL.md,rotacion.md,chequeos.md}` y
`plugins/exo/scripts/kb-precommit.sh` (Task 9), que E no menciona en su
alcance declarado.

## Ramas sugeridas

| Rama | Tareas | Lane |
|---|---|---|
| `d-preregistro` | 1 | mecánica |
| `d-gates-existentes` | 2, 3, 4 | mecánica (D-3/D-4 no aplican aquí; la única lectura es "adjudicar sin reinterpretar", ya con criterio escrito) |
| `d-port-rotate` | 5, 6 | mecánica salvo D-3/D-4, ya resueltas con recomendación aplicada en el código de la Task 5 |
| `d-port-stale` | 7, 8 | mecánica (fórmula ya firmada por Paul, cero decisión de producto pendiente) |
| `d-cutover` | 9, 10 | mecánica |

Dependencias entre ramas (no entre ficheros — dos ramas pueden abrirse en
paralelo aunque una espere el merge de otra para converger):

```
d-preregistro (Task 1)
   │
   ├──▶ d-port-rotate (Task 5 depende de 1; Task 6 depende además de d-gates-existentes/Task 2)
   │        │
   │        ▼ (Task 6 mergeada — Task 8 edita el mismo pre-registro, sección adyacente)
   ├──▶ d-port-stale (Task 7 depende de 1; Task 8 depende además de d-gates-existentes/Task 2 y de d-port-rotate/Task 6 ya mergeada)
   │        │
   │        ▼
   └──▶ d-cutover (Task 9 depende de 5 y 7; Task 10 depende de 3,4,6,8,9 — o de su estado `encolado` con evidencia)

d-gates-existentes (Task 2 → 3, 4) — no depende de nada de lo de arriba,
corre en paralelo desde el minuto uno. Si Task 2 falla (sin Go/gcc en esta
máquina), Tasks 3, 4, 6 y 8 quedan `encolado` — declarado, no bloquea 5, 7
ni 9.
```

**Con Paul fuera del camino crítico**: el 80% del trabajo de ingeniería real
(Tasks 5, 7, 9 — los dos ports y el cutover de consumidores) no depende de
que haya Go instalado ni de ningún PENDIENTE-PAUL sin resolver. Solo las
cuatro tasks de gate (3, 4, 6, 8) dependen de Task 2, y esa es la única que
puede fallar por algo fuera del control del plan (ausencia de toolchain Go
o de `gcc` en la máquina).

---

### Task 1: congelar el pre-registro de `rotate`/`stale` — antes de una sola línea de Rust

**Lane:** mecánica. **Depende de:** nada. **Oráculo:**
`git log --oneline --name-only -- docs/superpowers/plans/2026-09-14-campana-d-preregistro-paridad-rotate-stale.md`
— el commit de este fichero debe ser **anterior** al primer commit que
toque `engine/src/rotacion.rs` u `engine/src/obsolescencia.rs`.

**Files:**
- El pre-registro ya existe, escrito por este mismo plan:
  `docs/superpowers/plans/2026-09-14-campana-d-preregistro-paridad-rotate-stale.md`.

- [ ] **Step 1: verificar que el fichero está en el árbol de trabajo y sin
  editar respecto a lo que trae la rama**

  Run: `git -C /home/paul/Documentos/proyectos/exo status --porcelain -- docs/superpowers/plans/2026-09-14-campana-d-preregistro-paridad-rotate-stale.md`
  Expected: `?? docs/superpowers/plans/2026-09-14-campana-d-preregistro-paridad-rotate-stale.md`
  (sin trackear todavía — es la primera vez que se commitea).

- [ ] **Step 2: commit, solo este fichero**

```bash
git -C /home/paul/Documentos/proyectos/exo add docs/superpowers/plans/2026-09-14-campana-d-preregistro-paridad-rotate-stale.md
git -C /home/paul/Documentos/proyectos/exo commit -m "docs(d, preregistro): congela el criterio de paridad de rotate/stale antes del port"
```

- [ ] **Step 3: verificar el oráculo**

  Run: `git -C /home/paul/Documentos/proyectos/exo log --oneline -1 -- docs/superpowers/plans/2026-09-14-campana-d-preregistro-paridad-rotate-stale.md`
  Expected: una línea, el commit del Step 2. Este hash es el que demuestra
  que el documento se escribió antes del código — anótalo en el ledger de la
  fábrica para que la Task 5/7 lo cite si alguien pregunta.

---

### Task 2: prerrequisitos — Go, gcc, kbx `fe46443` compilado, copia de la KB

**Lane:** mecánica. **Depende de:** nada. **Oráculo:** los cuatro comandos
del Step 4 salen todos con éxito, o la task declara explícitamente cuál
falló y por qué, y deja Tasks 3, 4, 6 y 8 en `encolado` sin bloquear el
resto del plan.

**Files:** ninguno dentro de `exo/` — todo lo que esta task produce vive
fuera del repo (`~/.local/go/`, un worktree temporal de `kbx`,
`/tmp/campana-d/`).

- [ ] **Step 1: instalar Go 1.26.4 sin sudo, verificando el sha256**

```bash
export TMPDIR="${TMPDIR:-$(mktemp -d)}"
GO_TARBALL="go1.26.4.linux-amd64.tar.gz"
curl -fsSL "https://go.dev/dl/${GO_TARBALL}" -o "$TMPDIR/${GO_TARBALL}"
curl -fsSL "https://go.dev/dl/${GO_TARBALL}.sha256" -o "$TMPDIR/${GO_TARBALL}.sha256" 2>/dev/null \
  || echo "go.dev no publica .sha256 aparte para este asset: usar el hash de la página de descargas https://go.dev/dl/ y compararlo a mano contra sha256sum"
sha256sum "$TMPDIR/${GO_TARBALL}"
# Comparar el hash de arriba contra el publicado en https://go.dev/dl/ para
# go1.26.4.linux-amd64.tar.gz ANTES de descomprimir. Si no coincide, PARAR
# — no instalar un tarball sin verificar.
mkdir -p "$HOME/.local"
tar -C "$HOME/.local" -xzf "$TMPDIR/${GO_TARBALL}"
export PATH="$HOME/.local/go/bin:$PATH"
go version
```

  Expected: `go version go1.26.4 linux/amd64` (o la variante de plataforma
  que corresponda). **Si `go1.26.4` no está publicado a fecha de ejecutar
  esto** (kbx fija `go 1.26.4` en su `go.mod`, verificado hoy — pero la
  fábrica puede correr semanas después de escrito este plan): instalar la
  versión estable más reciente que sea `>= 1.26.4` y verificar con
  `go version` que `go.mod` la acepta (`go build` falla alto si no). Si
  falla la descarga, la verificación de hash, o la instalación: **declarar
  el fallo tal cual** (qué comando, qué salida) y dejar Tasks 3, 4, 6, 8 en
  `encolado` — no seguir intentando con variantes no verificadas.

- [ ] **Step 2: comprobar `gcc` (cgo lo necesita — `go-sqlite3` compila con
  el tag `sqlite_fts5`)**

  Run: `gcc --version && pkg-config --version 2>/dev/null; echo "cgo enabled: $(go env CGO_ENABLED)"`
  Expected: una versión de `gcc` (cualquiera reciente sirve) y
  `CGO_ENABLED=1`. Si `gcc` no está: instalar con el gestor de paquetes de
  la máquina (`apt-get install -y build-essential` en Debian/Ubuntu, sin
  sudo si la sesión no lo tiene — en ese caso declarar el fallo y encolar
  igual que en el Step 1) — **este plan no asume sudo disponible**; si no
  lo hay, se declara el bloqueo y se sigue con el resto del plan.

- [ ] **Step 3: compilar kbx `fe46443` desde un worktree temporal, fuera de
  `exo/`**

```bash
export PATH="$HOME/.local/go/bin:$PATH"
KBX_WT="$TMPDIR/kbx-fe46443"
git -C /home/paul/Documentos/proyectos/kbx worktree add --detach "$KBX_WT" fe46443
make -C "$KBX_WT" check    # build + vet + test, tag sqlite_fts5 (Makefile de kbx)
mkdir -p /tmp/campana-d
# NUNCA `make install`: pisa $HOME/.local/bin/kbx, fuera del repo y fuera
# del alcance de esta campaña (ninguna task de este plan escribe en
# ~/.local/bin — Global Constraints). Build directo al binario desechable.
(cd "$KBX_WT" && "$HOME/.local/go/bin/go" build -tags sqlite_fts5 -o /tmp/campana-d/kbx ./cmd/kbx)
/tmp/campana-d/kbx --help 2>&1 | head -5 || echo "kbx no expone --help formal: verificar con un subcomando conocido, p.ej. 'kbx targets --help'"
```

  Expected: `make check` termina en 0 (build + `go vet` + `go test` de todo
  el módulo, verde). El `go build` deja `kbx` ejecutable en
  `/tmp/campana-d/kbx`. Si `make check` falla: **declarar qué falló**
  (paste del error) — normalmente cgo/gcc si el Step 2 se saltó, o un fallo
  de red al resolver `github.com/mattn/go-sqlite3` (el `go.sum` de kbx fija
  la versión, así que con Go instalado y red disponible esto no debería
  necesitar tocar el `go.mod`). Encolar Tasks 3, 4, 6, 8 si no se resuelve.

  **Al terminar, limpiar el worktree** (no es parte del repo exo, pero es
  buena higiene y evita que quede un worktree huérfano de `kbx` en disco):
  `git -C /home/paul/Documentos/proyectos/kbx worktree remove "$KBX_WT"`
  — **después** de que las Tasks 3, 4, 6, 8 hayan usado el binario en
  `/tmp/campana-d/kbx` (el binario compilado sobrevive a borrar el
  worktree; solo hace falta el worktree mientras se compila).

- [ ] **Step 4: copiar la KB a un tmp, de solo lectura**

```bash
mkdir -p /tmp/campana-d
git clone --no-local /home/paul/Documentos/proyectos/wisdom-paul /tmp/campana-d/kb-base
cp ~/.exo/index.db /tmp/campana-d/index.db
grep -A1 '\[kb\]' ~/.exo/config.toml | grep name
```

  Expected: el clon completa sin error; `index.db` se copia (unos 20+ MB,
  verificado en G4a que el índice real ronda ese tamaño); la última línea
  imprime `name = "wisdom-paul"` (o el nombre real configurado) — **anotar
  este valor**, lo usa la Task 6 para verificar la Decisión D-4 sin
  asumirlo. Nada de lo anterior debe escribir dentro de
  `/home/paul/Documentos/proyectos/wisdom-paul` — es un clon aparte,
  `wisdom-paul` original queda intacto.

- [ ] **Step 5: declarar el resultado**

  Si los cuatro steps anteriores terminaron en verde: anotar en el ledger
  de la fábrica "Task 2: OK — Go `<versión>`, kbx `fe46443` en
  `/tmp/campana-d/kbx` (`$KBX` en las Tasks 3/4/6/8 — nunca en
  `~/.local/bin`), KB copiada en `/tmp/campana-d/kb-base`,
  `[kb] name = <valor>`" y desbloquear Tasks 3, 4 (y, más adelante, 6 y 8).
  Si algo falló: anotar exactamente qué step y qué comando, dejar Tasks 3,
  4, 6, 8 en `encolado` con esa nota, y **no** tocar el resto del plan —
  Tasks 1, 5, 7, 9 no dependen de esto.

---

### Task 3: gate de paridad de `targets` — rellenar el registro de G4a

**Lane:** mecánica (la única lectura es adjudicar el dato, sin
reinterpretar el criterio ya fijado en 2026-09-02). **Depende de:** Task 2.
**Oráculo:** los cinco topics del script de `2026-09-02-g4a-preregistro-targets.md`
§Comandos dan PASA, y la sección "Registro de la corrida" del propio
fichero queda rellenada (no un fichero nuevo — **el mismo**).

**Files:**
- Modify: `docs/superpowers/plans/2026-09-02-g4a-preregistro-targets.md`
  (añadir una sección "## Registro de la corrida" al final, con el mismo
  formato que ya usa `2026-09-09-g4c-preregistro-ratchet.md` — este fichero
  de G4a es anterior a ese formato y no la tiene todavía).

- [ ] **Step 1: construir el binario exo de esta rama**

  Run: `cd /home/paul/Documentos/proyectos/exo/engine && cargo build --release`
  Expected: compila limpio (0 warnings con `-D warnings` si se corre
  clippy aparte; este build por sí solo no lo exige).

- [ ] **Step 2: correr el script de §Comandos tal cual está escrito en el
  pre-registro**

```bash
export PATH="$HOME/.local/go/bin:$HOME/.local/bin:$PATH"
KBX=/tmp/campana-d/kbx
mkdir -p /tmp/g4a && cp /tmp/campana-d/index.db /tmp/g4a/index.db
KB=/tmp/campana-d/kb-base
for t in "indexer" "reflex" "memoria" "kbx" "recall en el punto de uso"; do
  slug=$(echo "$t" | tr ' ' '-')
  "$KBX" targets --db /tmp/g4a/index.db --kb "$KB" --limit 10 --json "$t" \
    | jq -S '.data.candidates | map({permalink, tier, size_bytes, last_commit}) | sort_by(.permalink)' \
    > "/tmp/g4a/go-$slug.json"
  /home/paul/Documentos/proyectos/exo/engine/target/release/exo targets --db /tmp/g4a/index.db --kb "$KB" --limit 10 --json "$t" \
    | jq -S '.data.candidates | map({permalink, tier, size_bytes, last_commit}) | sort_by(.permalink)' \
    > "/tmp/g4a/rs-$slug.json"
  diff -u "/tmp/g4a/go-$slug.json" "/tmp/g4a/rs-$slug.json" \
    && echo "PASA: $t" || echo "REVISAR: $t"
done
```

  Expected: 5 líneas `PASA: <topic>`. Dado que A1 (whitespace ASCII) y A2
  (KB sin git) ya están resueltos en el código actual, **no se espera
  ninguna divergencia de `tier`** — si aparece una, es una regresión real,
  no la divergencia 3 ya cerrada del registro original.

- [ ] **Step 3: si algún topic da REVISAR, investigar antes de adjudicar**

  Mirar el `diff` completo (no solo el resumen): ¿es una diferencia de
  `tier`/`size_bytes`/`last_commit` (fallo real, según el criterio del
  documento) o un empate de rank en la frontera del `--limit 10` (la única
  divergencia de CONJUNTO admisible, según el mismo criterio)? Adjudicar
  con la cita del `diff` en mano — no se reinterpreta el criterio, se aplica.

- [ ] **Step 4: añadir la sección "Registro de la corrida" al pre-registro
  de G4a**

  Al final de `docs/superpowers/plans/2026-09-02-g4a-preregistro-targets.md`,
  después de la última línea existente ("Eso demuestra que el lado Rust
  produce lo que dice producir…"), añadir:

```markdown

## Registro de la corrida

- Fecha: <fecha real de la ejecución>
- Commit de exo: <git rev-parse --short HEAD de la rama d-gates-existentes>
- Commit de kbx: `fe46443`
- A1 (whitespace ASCII del tier) y A2 (KB sin git) verificados como YA
  RESUELTOS antes de esta corrida (`docs/superpowers/plans/2026-09-04-g4b-budget-y-lint.md:73-128`).
  No se esperaba divergencia de `tier` por esa causa, y <se observó /
  no se observó> ninguna.
- Topics PASA: <lista> / de 5
- Divergencias observadas y su adjudicación: <detalle o "ninguna">
- **PASA / NO PASA** (global, exige los 5 topics): <veredicto>
```

  Rellenar los `<...>` con el dato real de la corrida — nunca con un
  resultado supuesto antes de correr los comandos.

- [ ] **Step 5: commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add docs/superpowers/plans/2026-09-02-g4a-preregistro-targets.md
git -C /home/paul/Documentos/proyectos/exo commit -m "docs(d, gate-targets): registro de la corrida de paridad de targets"
```

---

### Task 4: gate de paridad de `ratchet` — rellenar el registro de G4c

**Lane:** mecánica. **Depende de:** Task 2. **Oráculo:** las tres
invocaciones de `2026-09-09-g4c-preregistro-ratchet.md` §Qué se compara dan
un veredicto, y su sección "## Registro de la corrida" (ya existe vacía en
el fichero) queda rellenada.

**Files:**
- Modify: `docs/superpowers/plans/2026-09-09-g4c-preregistro-ratchet.md`
  (rellenar la sección "## Registro de la corrida" ya presente al final del
  fichero — no crear una nueva).

- [ ] **Step 1: copia de trabajo desechable de la KB (ratchet lee HEAD/staged
  y `--seal` escribe)**

```bash
RATCHET_KB=$(mktemp -d /tmp/campana-d/ratchet-kb.XXXX)
git clone --no-local /tmp/campana-d/kb-base "$RATCHET_KB"
git -C "$RATCHET_KB" config diff.renames false
```

- [ ] **Step 2: las tres invocaciones del pre-registro, `diff.renames=false`
  fijado como config del repo en el Step 1 (Adjudicación A3, ya fijada en
  el documento — no se reinterpreta; `-c` no es flag de kbx —
  `cmd/kbx/ratchet.go` solo acepta `kb/seal/json/staged/exclude`)**

```bash
export PATH="$HOME/.local/go/bin:$HOME/.local/bin:$PATH"
EXO=/home/paul/Documentos/proyectos/exo/engine/target/release/exo
KBX=/tmp/campana-d/kbx

# 1. working tree
"$KBX" ratchet --kb "$RATCHET_KB" > /tmp/campana-d/go-ratchet-wt.txt 2>&1; echo "kbx exit=$?"
"$EXO" ratchet --kb "$RATCHET_KB" > /tmp/campana-d/rs-ratchet-wt.txt 2>&1; echo "exo exit=$?"

# 2. --staged, con un cambio preparado en el índice
echo "prueba de staged" >> "$RATCHET_KB/log/$(ls "$RATCHET_KB/log" | head -1)"
git -C "$RATCHET_KB" -c diff.renames=false add -A
"$KBX" ratchet --kb "$RATCHET_KB" --staged > /tmp/campana-d/go-ratchet-staged.txt 2>&1; echo "kbx exit=$?"
"$EXO" ratchet --kb "$RATCHET_KB" --staged > /tmp/campana-d/rs-ratchet-staged.txt 2>&1; echo "exo exit=$?"
git -C "$RATCHET_KB" reset --hard -q HEAD   # deshacer el cambio de prueba antes del paso 3 (checkout -- . restauraría desde el índice, que ya tiene el add -A de arriba)

# 3. --json (el que gatea el criterio)
"$KBX" ratchet --kb "$RATCHET_KB" --json | jq -S '.data | {applied, findings: (.findings | sort_by(.path, .kind, .limit))}' > /tmp/campana-d/go-ratchet.json
"$EXO" ratchet --kb "$RATCHET_KB" --json | jq -S '.data | {applied, findings: (.findings | sort_by(.path, .kind, .limit))}' > /tmp/campana-d/rs-ratchet.json
diff -u /tmp/campana-d/go-ratchet.json /tmp/campana-d/rs-ratchet.json && echo "PASA: ratchet --json" || echo "REVISAR: ratchet --json"
```

- [ ] **Step 3: adjudicar contra el criterio ya fijado** (§Criterio del
  pre-registro: `applied` idéntico, conjunto de `(path, kind, limit)`
  idéntico, veredicto de ruptura idéntico; `was`/`now` se anotan pero no
  gatean). Las 6 divergencias ya declaradas en el documento (exit codes,
  `--seal`+`--staged`, orden de iteración, `diff.renames`, Unicode en
  claves del sello, `no-air-debt`) **no se reabren** — si el diff cae
  dentro de una de ellas, se anota como tal, no como fallo.

- [ ] **Step 4: rellenar "Registro de la corrida" en el propio fichero**

  Editar la sección ya presente al final de
  `docs/superpowers/plans/2026-09-09-g4c-preregistro-ratchet.md`
  (`- Fecha:`, `- Commit de exo:`, etc.) con los valores reales de esta
  corrida, siguiendo exactamente los campos que el fichero ya trae.

- [ ] **Step 5: commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add docs/superpowers/plans/2026-09-09-g4c-preregistro-ratchet.md
git -C /home/paul/Documentos/proyectos/exo commit -m "docs(d, gate-ratchet): registro de la corrida de paridad de ratchet"
```

---

### Task 5: port de `rotate` — `engine/src/rotacion.rs`, TDD

**Lane:** mecánica salvo D-3/D-4, ya resueltas con recomendación aplicada
en el código de abajo. **Depende de:** Task 1 (pre-registro congelado por
commit). **No depende de Task 2** — es Rust puro, sin Go de por medio; sus
propios tests son el oráculo. **Oráculo global de la task:**
`cd engine && cargo test --release --lib rotacion && cargo test --release --test rotar_cli && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`.

**Files:**
- Create: `engine/src/rotacion.rs`
- Modify: `engine/src/lib.rs` (declarar `pub mod rotacion;`)
- Modify: `engine/src/main.rs` (ver Step D)
- Create: `engine/tests/rotar_cli.rs`

**Interfaces — Produces** (lo que Task 9 y la Task 6 consumen):
```rust
pub struct Plan<'a> { pub preambulo: &'a [u8], pub frio: &'a [u8], pub caliente: &'a [u8], pub entradas_frias: usize, pub entradas_calientes: usize }
pub fn parte(contenido: &[u8], presupuesto_caliente: i64) -> Plan<'_>;
pub fn nombre_de_archivo(slug: &str, frio: &[u8], existentes: usize) -> String;
pub fn construye_archivo(frontmatter_original: &[u8], frio: &[u8], titulo: &str, permalink: &str, archivo_previo: &str) -> Vec<u8>;
pub struct Resultado { pub nota: String, pub archivo: Option<String>, pub bytes_movidos: usize, pub entradas_frias: usize, pub rotado: bool } // Serialize
pub fn aplica(kb_root: &Path, ruta_rel: &str, presupuesto_caliente: i64, escribe: bool, nombre_kb: &str) -> Result<Resultado>;
```
CLI: subcomando `Comando::Rotate(ArgsRotate)`, función `rotate_cmd`.

#### Step A: `parte` (Split) y el corte posicional por `## `

- [ ] **A1 — test que falla**, en `engine/src/rotacion.rs` (fichero nuevo,
  el test va dentro de `#[cfg(test)] mod tests` al final):

```rust
//! `exo rotate` — divide una bitácora `tier: log` en un prefijo frío
//! (archivado) y una cola caliente (que se queda), portado de
//! `kbx/internal/rotate` (`fe46443`). El corte es posicional, no por
//! fecha: solo 34 de 60 headings de la bitácora más irregular llevan fecha
//! ISO, así que parsear fechas sería frágil justo donde más importa
//! (`rotate.go`, comentario de cabecera del paquete).

use anyhow::{Context, Result};
use serde::Serialize;
use std::borrow::Cow;
use std::path::Path;
use std::sync::LazyLock;

/// El resultado de partir un contenido en preámbulo + frío + caliente.
/// Invariante duro: `preambulo + frio + caliente` reconstruye el original
/// byte a byte — nada se pierde (kbx `rotate.Plan`).
#[derive(Debug, PartialEq, Eq)]
pub struct Plan<'a> {
    pub preambulo: &'a [u8],
    pub frio: &'a [u8],
    pub caliente: &'a [u8],
    pub entradas_frias: usize,
    pub entradas_calientes: usize,
}

/// Cada línea de `contenido` con su offset de inicio, sin el terminador
/// `\n`. Si `contenido` no acaba en `\n`, la última línea (parcial) se
/// incluye igual — mismo contrato que `splitLinesKeepOffsets` en el Go
/// original, del que dependen `offsets_de_entradas`, `bloque_frontmatter` y
/// `frontmatter_sin_terminar`: una sola función de partir en líneas para
/// las tres, en vez de tres copias que puedan derivar.
fn lineas_con_offset(contenido: &[u8]) -> Vec<(usize, &[u8])> {
    let mut out = Vec::new();
    let mut off = 0usize;
    while off <= contenido.len() {
        match contenido[off..].iter().position(|&b| b == b'\n') {
            None => {
                out.push((off, &contenido[off..]));
                break;
            }
            Some(n) => {
                out.push((off, &contenido[off..off + n]));
                off += n + 1;
            }
        }
    }
    out
}

/// Offset de cada heading `"## "` a inicio de línea, ignorando los que
/// caen dentro de una valla ``` de código.
fn offsets_de_entradas(contenido: &[u8]) -> Vec<usize> {
    let mut offs = Vec::new();
    let mut en_valla = false;
    for (off, linea) in lineas_con_offset(contenido) {
        if linea.starts_with(b"```") {
            en_valla = !en_valla;
            continue;
        }
        if !en_valla && linea.starts_with(b"## ") {
            offs.push(off);
        }
    }
    offs
}

/// Mantiene las entradas más nuevas cuyo tamaño total, junto al preámbulo,
/// entra en `presupuesto_caliente`. Al menos una entrada se queda siempre
/// caliente (kbx `rotate.Split`).
pub fn parte(contenido: &[u8], presupuesto_caliente: i64) -> Plan<'_> {
    let offs = offsets_de_entradas(contenido);
    let Some(&primera) = offs.first() else {
        return Plan {
            preambulo: contenido,
            frio: &contenido[contenido.len()..],
            caliente: &contenido[contenido.len()..],
            entradas_frias: 0,
            entradas_calientes: 0,
        };
    };
    let preambulo = &contenido[..primera];

    let mut corte = offs.len() - 1;
    for i in (0..offs.len() - 1).rev() {
        let tamano = (contenido.len() - offs[i]) as i64;
        if preambulo.len() as i64 + tamano > presupuesto_caliente {
            break;
        }
        corte = i;
    }

    Plan {
        preambulo,
        frio: &contenido[offs[0]..offs[corte]],
        caliente: &contenido[offs[corte]..],
        entradas_frias: corte,
        entradas_calientes: offs.len() - corte,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(entradas: &[&str]) -> Vec<u8> {
        let mut s = String::from("---\ntitle: x-bitacora\ntier: log\n---\n\n# x — bitácora\n\n");
        for e in entradas {
            s.push_str(e);
        }
        s.into_bytes()
    }

    #[test]
    fn mantiene_las_entradas_mas_nuevas_dentro_del_presupuesto() {
        let e1 = format!("## 2026-01-01 — uno\n{}\n\n", "a".repeat(80));
        let e2 = format!("## 2026-02-01 — dos\n{}\n\n", "b".repeat(80));
        let e3 = format!("## 2026-03-01 — tres\n{}\n\n", "c".repeat(80));
        let contenido = doc(&[&e1, &e2, &e3]);

        let p = parte(&contenido, contenido.len() as i64 - 150);

        assert_eq!(p.entradas_calientes, 1);
        assert_eq!(p.entradas_frias, 2);
        assert!(String::from_utf8_lossy(p.caliente).contains("tres"));
        assert!(String::from_utf8_lossy(p.frio).contains("uno"));
        assert!(String::from_utf8_lossy(p.frio).contains("dos"));
        assert!(String::from_utf8_lossy(p.preambulo).starts_with("---\ntitle:"));
        // Invariante duro: nada se pierde.
        let mut reconstruido = Vec::new();
        reconstruido.extend_from_slice(p.preambulo);
        reconstruido.extend_from_slice(p.frio);
        reconstruido.extend_from_slice(p.caliente);
        assert_eq!(reconstruido, contenido);
    }

    #[test]
    fn nada_rota_si_cabe() {
        let contenido = doc(&["## 2026-01-01 — uno\ncorto\n\n"]);
        let p = parte(&contenido, 100_000);
        assert_eq!(p.entradas_frias, 0);
        assert!(p.frio.is_empty());
        assert_eq!(p.entradas_calientes, 1);
    }

    #[test]
    fn siempre_queda_al_menos_una_entrada_caliente_aunque_no_quepa() {
        let vieja = "## 2026-01-01 — vieja\nx\n\n".to_string();
        let gorda = format!("## 2026-03-01 — gorda\n{}\n", "z".repeat(5000));
        let contenido = doc(&[&vieja, &gorda]);
        let p = parte(&contenido, 100);
        assert_eq!(p.entradas_calientes, 1);
        assert!(String::from_utf8_lossy(p.caliente).contains("gorda"));
    }

    #[test]
    fn sin_entradas_todo_queda_en_preambulo() {
        let contenido = b"---\ntier: log\n---\n\n# solo titulo\n".to_vec();
        let p = parte(&contenido, 10);
        assert!(p.frio.is_empty() && p.caliente.is_empty());
        assert_eq!(p.preambulo, &contenido[..]);
    }

    #[test]
    fn un_heading_dentro_de_una_valla_no_es_una_entrada() {
        let e1 = "## 2026-01-01 — uno\n```\n## no soy un heading\n```\n\n".to_string();
        let e2 = "## 2026-02-01 — dos\nx\n\n".to_string();
        let contenido = doc(&[&e1, &e2]);
        let p = parte(&contenido, 100_000);
        assert_eq!(p.entradas_calientes, 2);
    }
}
```

- [ ] **A2 — verlo fallar**

  Run: `cd engine && cargo test --release --lib rotacion`
  Expected: FAIL de compilación — `rotacion` no existe todavía como módulo
  del crate. (Es intencional: A1 ya escribió `rotacion.rs` completo con la
  implementación de `parte` — el "fallo esperado" de este paso es que el
  módulo no está declarado en `lib.rs` todavía, no que la lógica esté sin
  escribir. Sigue al Step siguiente para declararlo y verlo compilar y
  pasar.)

- [ ] **A3 — declarar el módulo y verlo pasar**

  En `engine/src/lib.rs`, añadir junto a los demás `pub mod`:

```rust
pub mod rotacion;
```

  Run: `cd engine && cargo test --release --lib rotacion`
  Expected: `test result: ok. 5 passed; 0 failed`.

- [ ] **A4 — commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add engine/src/rotacion.rs engine/src/lib.rs
git -C /home/paul/Documentos/proyectos/exo commit -m "feat(d, rotate): parte() — corte posicional por heading con fences ignorados"
```

#### Step B: nombre de archivo y documento archivado

- [ ] **B1 — tests que fallan**, añadidos a `engine/src/rotacion.rs` (código
  de producción **antes** de `#[cfg(test)]`, tests dentro):

```rust
/// Línea delimitadora de frontmatter: `---` seguida solo de espacios/tabs,
/// tolerando también un `\r` final — mismo fix que ya aplica
/// `frontmatter::es_delimitador` para el resto de exo (sin él, un checkout
/// CRLF hace que un cierre `---\r` nunca matchee y el fallback sintético se
/// dispare en silencio sobre frontmatter válido; es el "fallo silencioso
/// canónico del port" que `frontmatter.rs` ya documenta, y esta función
/// replica la misma regla en vez de reintroducir la más estrecha de kbx).
fn es_delimitador_frontmatter(linea: &[u8]) -> bool {
    let mut fin = linea.len();
    while fin > 0 && matches!(linea[fin - 1], b' ' | b'\t' | b'\r') {
        fin -= 1;
    }
    &linea[..fin] == b"---"
}

/// El bloque `---...---` inicial, o un bloque sintético `tier: log` si
/// `contenido` no empieza por un delimitador o nunca cierra.
fn bloque_frontmatter(contenido: &[u8]) -> Cow<'_, [u8]> {
    const SINTETICO: &[u8] = b"---\ntier: log\n---\n";
    let lineas = lineas_con_offset(contenido);
    let Some(&(_, primera)) = lineas.first() else {
        return Cow::Borrowed(SINTETICO);
    };
    if !es_delimitador_frontmatter(primera) {
        return Cow::Borrowed(SINTETICO);
    }
    for &(off, linea) in &lineas[1..] {
        if !es_delimitador_frontmatter(linea) {
            continue;
        }
        let mut fin = off + linea.len();
        if fin < contenido.len() && contenido[fin] == b'\n' {
            fin += 1;
        }
        return Cow::Owned(contenido[..fin].to_vec());
    }
    Cow::Borrowed(SINTETICO)
}

/// ¿Empieza `contenido` con un delimitador de frontmatter que nunca cierra?
/// Distinto de "sin frontmatter": `aplica` (Step C) rechaza este caso en vez
/// de archivar entradas frías bajo metadata sintética que borraría en
/// silencio el tier/tags/permalink real de la nota.
fn frontmatter_sin_terminar(contenido: &[u8]) -> bool {
    let lineas = lineas_con_offset(contenido);
    let Some(&(_, primera)) = lineas.first() else {
        return false;
    };
    if !es_delimitador_frontmatter(primera) {
        return false;
    }
    !lineas[1..].iter().any(|&(_, l)| es_delimitador_frontmatter(l))
}

fn offset_delimitador_cierre(fm: &[u8]) -> usize {
    let mut cierre = 0usize;
    for (off, linea) in lineas_con_offset(fm) {
        if es_delimitador_frontmatter(linea) {
            cierre = off;
        }
    }
    cierre
}

/// Reescribe `clave: valor` dentro de un bloque de frontmatter,
/// añadiéndola antes del delimitador de cierre si no existía.
fn reemplaza_clave(fm: &[u8], clave: &str, valor: &str) -> Vec<u8> {
    let patron = format!("(?m)^{}:.*$", regex::escape(clave));
    let re = regex::bytes::Regex::new(&patron).expect("patrón de clave válido");
    if re.is_match(fm) {
        return re
            .replace_all(fm, format!("{clave}: {valor}").as_bytes())
            .into_owned();
    }
    let cierre = offset_delimitador_cierre(fm);
    if cierre == 0 {
        return fm.to_vec();
    }
    let mut out = Vec::with_capacity(fm.len() + clave.len() + valor.len() + 4);
    out.extend_from_slice(&fm[..cierre]);
    out.extend_from_slice(format!("{clave}: {valor}\n").as_bytes());
    out.extend_from_slice(&fm[cierre..]);
    out
}

/// Envuelve `s` como escalar YAML de comilla simple: la única regla de
/// escape es doblar una comilla simple embebida, así que es seguro para
/// cualquier contenido — a diferencia del estilo "plain", donde `#`, `&`,
/// `*`, `:` o `[` cambian lo que la línea significa para un parser YAML. El
/// título/slug de una nota los escribe Paul libremente.
fn comilla_simple_yaml(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

static PATRON_FECHA_ISO: LazyLock<regex::bytes::Regex> =
    LazyLock::new(|| regex::bytes::Regex::new(r"\d{4}-\d{2}-\d{2}").expect("regex de fecha ISO"));

/// Deriva el nombre del archivo de las fechas ISO del bloque frío. Sin
/// fechas, cae a un número de secuencia (caso real:
/// desarrollo-agentico-bitacora, 26 de 60 headings sin fecha).
/// `existentes` es cuántos archivos ya hay para este slug.
pub fn nombre_de_archivo(slug: &str, frio: &[u8], existentes: usize) -> String {
    let mut fechas: Vec<&[u8]> = PATRON_FECHA_ISO.find_iter(frio).map(|m| m.as_bytes()).collect();
    if fechas.is_empty() {
        return format!("{slug}-parte-{:02}.md", existentes + 1);
    }
    fechas.sort_unstable();
    format!(
        "{slug}-{}_{}.md",
        String::from_utf8_lossy(fechas[0]),
        String::from_utf8_lossy(fechas[fechas.len() - 1])
    )
}

/// Construye la nota archivada: el frontmatter original con `title` y
/// `permalink` reescritos, seguido del bloque frío verbatim. Con
/// `archivo_previo` no vacío, escribe un enlace hacia atrás justo después
/// del heading de título — la nota viva solo guarda UN aviso, apuntando al
/// archivo más reciente (Step C), así que cada archivo lleva el enlace
/// hacia atrás y la cadena se recorre archivo a archivo.
pub fn construye_archivo(
    frontmatter_original: &[u8],
    frio: &[u8],
    titulo: &str,
    permalink: &str,
    archivo_previo: &str,
) -> Vec<u8> {
    let mut fm = bloque_frontmatter(frontmatter_original).into_owned();
    fm = reemplaza_clave(&fm, "title", &comilla_simple_yaml(titulo));
    fm = reemplaza_clave(&fm, "permalink", &comilla_simple_yaml(permalink));

    let mut salida = Vec::with_capacity(fm.len() + frio.len() + titulo.len() + 64);
    salida.extend_from_slice(&fm);
    salida.extend_from_slice(b"\n# ");
    salida.extend_from_slice(titulo.as_bytes());
    salida.extend_from_slice(b"\n\n");
    if !archivo_previo.is_empty() {
        salida.extend_from_slice("> Continúa el histórico anterior en [[".as_bytes());
        salida.extend_from_slice(archivo_previo.as_bytes());
        salida.extend_from_slice("]].\n\n".as_bytes());
    }
    salida.extend_from_slice(frio);
    salida
}
```

  Y en `mod tests`, añadir:

```rust
    #[test]
    fn nombre_de_archivo_usa_el_rango_de_fechas_iso() {
        let frio = b"## 2026-06-26 -- a\nx\n\n## Actualizacion 2026-07-11 -- b\ny\n\n";
        assert_eq!(
            nombre_de_archivo("agent-develop-bitacora", frio, 0),
            "agent-develop-bitacora-2026-06-26_2026-07-11.md"
        );
    }

    #[test]
    fn nombre_de_archivo_cae_a_numero_de_parte_sin_fechas() {
        let frio = b"## sin fecha\nx\n\n";
        assert_eq!(
            nombre_de_archivo("desarrollo-agentico-bitacora", frio, 2),
            "desarrollo-agentico-bitacora-parte-03.md"
        );
    }

    #[test]
    fn construye_archivo_conserva_tier_log_y_reescribe_title_y_permalink() {
        let fm = b"---\ntitle: agent-develop-bitacora\ntype: note\npermalink: wisdom-paul/log/agent-develop-bitacora\ntags:\n- bitacora\ntier: log\n---\n\n# agent-develop -- bitacora\n\n";
        let frio = b"## 2026-06-26 -- a\nx\n\n";
        let doc = construye_archivo(fm, frio, "agent-develop-bitacora 2026-06-26_2026-07-11", "wisdom-paul/archive/log/agent-develop-bitacora-2026-06-26_2026-07-11", "");
        let doc = String::from_utf8_lossy(&doc);
        assert!(doc.starts_with("---\n"));
        assert!(doc.contains("title: 'agent-develop-bitacora 2026-06-26_2026-07-11'\n"));
        assert!(doc.contains("permalink: 'wisdom-paul/archive/log/agent-develop-bitacora-2026-06-26_2026-07-11'\n"));
        assert!(doc.contains("tier: log\n"));
        assert!(doc.contains("## 2026-06-26"));
    }

    // Pin del finding de la review de kbx: un cierre `---  \n` (espacio
    // final) no debe caer al bloque sintético ni al "title/permalink no se
    // escriben" — la misma clase de bug que motivó el `\r`/espacios en
    // `es_delimitador_frontmatter`.
    #[test]
    fn construye_archivo_tolera_espacio_final_en_el_cierre_del_frontmatter() {
        let fm = b"---\ntype: note\ntags:\n- bitacora\ntier: log\n---  \n\n# x -- bitacora\n\n";
        let frio = b"## 2026-06-26 -- a\nx\n\n";
        let doc = construye_archivo(fm, frio, "nuevo-titulo", "nuevo-permalink", "");
        let doc = String::from_utf8_lossy(&doc);
        assert!(doc.contains("title: 'nuevo-titulo'\n"));
        assert!(doc.contains("permalink: 'nuevo-permalink'\n"));
        assert!(doc.contains("type: note\n"), "frontmatter original debe conservarse: {doc}");
    }

    #[test]
    fn construye_archivo_cae_a_sintetico_sin_frontmatter_de_origen() {
        let fm = b"# solo un heading\n\ntexto\n";
        let frio = b"## 2026-01-01 -- a\nx\n\n";
        let doc = construye_archivo(fm, frio, "titulo-x", "permalink-x", "");
        let doc = String::from_utf8_lossy(&doc);
        assert!(doc.starts_with("---\ntier: log\n"));
        assert!(doc.contains("title: 'titulo-x'\n"));
    }

    #[test]
    fn construye_archivo_enlaza_al_archivo_previo_cuando_se_pasa() {
        let doc = construye_archivo(b"---\ntier: log\n---\n", b"## x\n", "t", "p", "bitacora-parte-01");
        assert!(String::from_utf8_lossy(&doc).contains("[[bitacora-parte-01]]"));
    }

    #[test]
    fn frontmatter_sin_terminar_detecta_el_delimitador_sin_cierre() {
        let sin_cerrar = b"---\ntitle: x\nsin cierre aqui\n";
        assert!(frontmatter_sin_terminar(sin_cerrar));
        let normal = b"---\ntier: log\n---\ncuerpo\n";
        assert!(!frontmatter_sin_terminar(normal));
        let sin_frontmatter = b"# solo un heading\n";
        assert!(!frontmatter_sin_terminar(sin_frontmatter));
    }
```

- [ ] **B2 — verlos fallar, implementar (ya está arriba, junto al test),
  verlos pasar**

  Run: `cd engine && cargo test --release --lib rotacion`
  Expected: `test result: ok. 12 passed; 0 failed` (5 de Step A + 7 nuevos).

- [ ] **B3 — commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add engine/src/rotacion.rs
git -C /home/paul/Documentos/proyectos/exo commit -m "feat(d, rotate): nombre_de_archivo() y construye_archivo() — naming y documento archivado"
```

#### Step C: `aplica` — el `Apply` completo, con escritura atómica

- [ ] **C1 — tests que fallan**, añadidos a `mod tests`:

```rust
    fn escribe_nota(root: &std::path::Path, rel: &str, cuerpo: &str) -> std::path::PathBuf {
        let full = root.join(rel);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(&full, cuerpo).unwrap();
        full
    }

    fn nota_grande() -> String {
        let mut s = String::from("---\ntitle: p-bitacora\ntype: note\npermalink: wisdom-paul/log/p-bitacora\ntier: log\n---\n\n# p -- bitacora\n\n");
        for i in 0..10 {
            s.push_str(&format!("## 2026-0{}-01 -- entrada\n", 1 + i % 9));
            s.push_str(&"x".repeat(3000));
            s.push_str("\n\n");
        }
        s
    }

    #[test]
    fn aplica_en_dry_run_no_toca_disco() {
        let dir = tempfile::tempdir().unwrap();
        escribe_nota(dir.path(), "log/p-bitacora.md", &nota_grande());
        let antes = std::fs::read(dir.path().join("log/p-bitacora.md")).unwrap();

        let res = aplica(dir.path(), "log/p-bitacora.md", 8000, false, "wisdom-paul").unwrap();
        assert!(res.rotado);
        let despues = std::fs::read(dir.path().join("log/p-bitacora.md")).unwrap();
        assert_eq!(antes, despues, "dry-run no debe modificar la nota");
        assert!(!dir.path().join("archive/log").exists(), "dry-run no debe crear archive/");
    }

    #[test]
    fn aplica_escribe_archivo_y_encoge_la_nota() {
        let dir = tempfile::tempdir().unwrap();
        let original = nota_grande();
        escribe_nota(dir.path(), "log/p-bitacora.md", &original);

        let res = aplica(dir.path(), "log/p-bitacora.md", 8000, true, "wisdom-paul").unwrap();
        let caliente = std::fs::read(dir.path().join("log/p-bitacora.md")).unwrap();
        assert!(caliente.len() <= 8000 + 200, "nota viva demasiado grande: {}", caliente.len());
        let frio = std::fs::read(dir.path().join(res.archivo.as_ref().unwrap())).unwrap();
        assert!(String::from_utf8_lossy(&caliente).contains("archivado en"));
        assert!(String::from_utf8_lossy(&frio).contains("tier: log"));

        let total_entradas = String::from_utf8_lossy(&caliente).matches("\n## ").count()
            + String::from_utf8_lossy(&frio).matches("\n## ").count();
        assert_eq!(total_entradas, 10, "nada se borra");
    }

    #[test]
    fn aplica_no_rota_lo_que_ya_cabe() {
        let dir = tempfile::tempdir().unwrap();
        escribe_nota(dir.path(), "log/small.md", "---\ntier: log\n---\n\n# s\n\n## 2026-01-01 -- u\nx\n");
        let res = aplica(dir.path(), "log/small.md", 100_000, true, "wisdom-paul").unwrap();
        assert!(!res.rotado);
        assert!(!dir.path().join("archive/log").exists());
    }

    #[test]
    fn aplica_rechaza_frontmatter_sin_cerrar() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = String::from("---\ntitle: broken\ntier: log\n\n# broken -- bitacora\n\n");
        for i in 0..10 {
            s.push_str(&format!("## 2026-0{}-01 -- entrada\n{}\n\n", 1 + i % 9, "x".repeat(3000)));
        }
        escribe_nota(dir.path(), "log/broken.md", &s);
        let res = aplica(dir.path(), "log/broken.md", 8000, true, "wisdom-paul");
        assert!(res.is_err());
        assert!(!dir.path().join("archive/log").exists());
        let intacta = std::fs::read_to_string(dir.path().join("log/broken.md")).unwrap();
        assert_eq!(intacta, s, "una nota rechazada no debe tocarse");
    }

    #[test]
    fn aplica_dos_rotaciones_del_mismo_dia_no_se_pisan() {
        let dir = tempfile::tempdir().unwrap();
        let entradas = |n: usize, marca: &str| {
            let mut s = String::new();
            for _ in 0..n {
                s.push_str("## 2026-08-03 -- entrada\n");
                s.push_str(&marca.repeat(300));
                s.push_str("\n\n");
            }
            s
        };
        let original = format!("---\ntitle: x\ntier: log\n---\n\n# x -- bitacora\n\n{}", entradas(60, "x"));
        escribe_nota(dir.path(), "log/proyecto-x-bitacora.md", &original);

        let r1 = aplica(dir.path(), "log/proyecto-x-bitacora.md", 8000, true, "wisdom-paul").unwrap();
        assert!(r1.rotado);
        let caliente = std::fs::read_to_string(dir.path().join("log/proyecto-x-bitacora.md")).unwrap();
        let crecida = caliente + &entradas(40, "y");
        std::fs::write(dir.path().join("log/proyecto-x-bitacora.md"), &crecida).unwrap();

        let r2 = aplica(dir.path(), "log/proyecto-x-bitacora.md", 8000, true, "wisdom-paul").unwrap();
        assert!(r2.rotado);
        assert_ne!(r1.archivo, r2.archivo, "las dos rotaciones no deben escribir el mismo archivo");

        let frio1 = std::fs::read_to_string(dir.path().join(r1.archivo.unwrap())).unwrap();
        let frio2 = std::fs::read_to_string(dir.path().join(r2.archivo.unwrap())).unwrap();
        let final_caliente = std::fs::read_to_string(dir.path().join("log/proyecto-x-bitacora.md")).unwrap();
        let total = final_caliente.matches("\n## ").count() + frio1.matches("\n## ").count() + frio2.matches("\n## ").count();
        assert_eq!(total, 100, "nada se pierde ni se pisa entre dos rotaciones del mismo dia");
    }

    #[test]
    fn aplica_reconstruye_byte_a_byte() {
        let dir = tempfile::tempdir().unwrap();
        let original = nota_grande();
        escribe_nota(dir.path(), "log/p-bitacora.md", &original);
        let res = aplica(dir.path(), "log/p-bitacora.md", 8000, true, "wisdom-paul").unwrap();
        assert!(res.rotado);

        let nota_viva = std::fs::read(dir.path().join("log/p-bitacora.md")).unwrap();
        let archivo = std::fs::read(dir.path().join(res.archivo.as_ref().unwrap())).unwrap();
        let preambulo = parte(original.as_bytes(), 8000).preambulo.to_vec();

        assert!(res.bytes_movidos <= archivo.len());
        let frio_del_archivo = &archivo[archivo.len() - res.bytes_movidos..];
        let caliente_esperado = &original.as_bytes()[preambulo.len() + res.bytes_movidos..];
        assert!(caliente_esperado.len() <= nota_viva.len());
        let caliente_de_la_nota = &nota_viva[nota_viva.len() - caliente_esperado.len()..];

        let mut reconstruido = Vec::new();
        reconstruido.extend_from_slice(&preambulo);
        reconstruido.extend_from_slice(frio_del_archivo);
        reconstruido.extend_from_slice(caliente_de_la_nota);
        assert_eq!(reconstruido, original.as_bytes());
    }

    #[test]
    fn aplica_no_deja_temporales_huerfanos() {
        let dir = tempfile::tempdir().unwrap();
        escribe_nota(dir.path(), "log/p-bitacora.md", &nota_grande());
        let res = aplica(dir.path(), "log/p-bitacora.md", 8000, true, "wisdom-paul").unwrap();
        for entrada in std::fs::read_dir(dir.path().join("log")).unwrap() {
            let nombre = entrada.unwrap().file_name();
            assert!(!nombre.to_string_lossy().contains(".tmp"), "quedo un temporal: {nombre:?}");
        }
        let dir_archivo = std::path::Path::new(res.archivo.as_ref().unwrap()).parent().unwrap();
        for entrada in std::fs::read_dir(dir.path().join(dir_archivo)).unwrap() {
            let nombre = entrada.unwrap().file_name();
            assert!(!nombre.to_string_lossy().contains(".tmp"), "quedo un temporal: {nombre:?}");
        }
    }

    // Pin de la Decisión D-4: el permalink usa `nombre_kb`, no un literal
    // fijo — `aplica` toma el nombre por parámetro precisamente para que
    // esto sea observable sin montar config.
    #[test]
    fn aplica_usa_el_nombre_de_kb_pasado_como_prefijo_del_permalink() {
        let dir = tempfile::tempdir().unwrap();
        escribe_nota(dir.path(), "log/p-bitacora.md", &nota_grande());
        let res = aplica(dir.path(), "log/p-bitacora.md", 8000, true, "otra-kb").unwrap();
        let archivo = std::fs::read_to_string(dir.path().join(res.archivo.unwrap())).unwrap();
        assert!(archivo.contains("permalink: 'otra-kb/archive/log/"), "permalink: {archivo}");
    }

    // Pin del Fix 1 de la review: PATRON_AVISO debe casar el aviso real (en
    // UTF-8, no bytes latin1-escapados) o `quita_aviso_previo` es un no-op y
    // los avisos se acumulan en cada rotación.
    #[test]
    fn aplica_dos_veces_seguidas_deja_un_solo_aviso_y_encadena_los_archivos() {
        let dir = tempfile::tempdir().unwrap();
        let entradas = |n: usize, marca: &str| {
            let mut s = String::new();
            for _ in 0..n {
                s.push_str("## 2026-08-03 -- entrada\n");
                s.push_str(&marca.repeat(300));
                s.push_str("\n\n");
            }
            s
        };
        let original = format!("---\ntitle: x\ntier: log\n---\n\n# x -- bitacora\n\n{}", entradas(60, "x"));
        escribe_nota(dir.path(), "log/proyecto-x-bitacora.md", &original);

        let r1 = aplica(dir.path(), "log/proyecto-x-bitacora.md", 8000, true, "wisdom-paul").unwrap();
        assert!(r1.rotado);
        let caliente = std::fs::read_to_string(dir.path().join("log/proyecto-x-bitacora.md")).unwrap();
        let crecida = caliente + &entradas(40, "y");
        std::fs::write(dir.path().join("log/proyecto-x-bitacora.md"), &crecida).unwrap();

        let r2 = aplica(dir.path(), "log/proyecto-x-bitacora.md", 8000, true, "wisdom-paul").unwrap();
        assert!(r2.rotado);

        let nota_viva = std::fs::read_to_string(dir.path().join("log/proyecto-x-bitacora.md")).unwrap();
        assert_eq!(
            nota_viva.matches("archivado en").count(),
            1,
            "la nota viva debe llevar exactamente un aviso tras dos rotaciones: {nota_viva}"
        );

        let archivo1 = r1.archivo.unwrap();
        let archivo2 = r2.archivo.unwrap();
        let nombre1_sin_md = std::path::Path::new(&archivo1).file_stem().unwrap().to_str().unwrap();
        let contenido_archivo2 = std::fs::read_to_string(dir.path().join(&archivo2)).unwrap();
        assert!(
            contenido_archivo2.contains(&format!("[[{nombre1_sin_md}]]")),
            "archivo2 debe enlazar hacia atrás a archivo1 ({nombre1_sin_md}): {contenido_archivo2}"
        );
    }
```

- [ ] **C2 — verlos fallar**

  Run: `cd engine && cargo test --release --lib rotacion`
  Expected: FAIL de compilación — `aplica` no existe (×9 tests nuevos).

- [ ] **C3 — implementación mínima**, añadida a `engine/src/rotacion.rs`
  antes de `#[cfg(test)]`:

```rust
#[derive(Serialize)]
pub struct Resultado {
    #[serde(rename = "note")]
    pub nota: String,
    #[serde(rename = "archive", skip_serializing_if = "Option::is_none")]
    pub archivo: Option<String>,
    #[serde(rename = "moved_bytes")]
    pub bytes_movidos: usize,
    #[serde(rename = "cold_entries")]
    pub entradas_frias: usize,
    #[serde(rename = "rotated")]
    pub rotado: bool,
}

impl Resultado {
    fn vacio(nota: String) -> Self {
        Self { nota, archivo: None, bytes_movidos: 0, entradas_frias: 0, rotado: false }
    }
}

fn aviso_de_archivo(titulo: &str, entradas_frias: usize, bytes_movidos: usize) -> String {
    format!("> Histórico anterior archivado en [[{titulo}]] ({entradas_frias} entradas, {bytes_movidos} B).\n\n")
}

static PATRON_AVISO: LazyLock<regex::bytes::Regex> = LazyLock::new(|| {
    // UTF-8 real, no bytes latin1-escapados: en `regex::bytes` con Unicode
    // activo (default), `\xc3\xb3` casa el codepoint U+00C3 U+00B3, no los
    // bytes de «ó» — con eso el patrón nunca casaba «Histórico» y
    // `quita_aviso_previo` era un no-op (verificado con regex 1.13.1).
    regex::bytes::Regex::new(r"> Histórico anterior archivado en \[\[(.*)\]\] \(\d+ entradas, \d+ B\)\.\n\n")
        .expect("patrón de aviso de archivo")
});

/// Quita el primer aviso de archivo del preámbulo de `contenido`, si lo
/// hay, para que una cadena de rotaciones no acumule un aviso por rotación
/// (el preámbulo nunca rota — `parte` siempre lo deja entero caliente).
/// Devuelve el contenido sin el aviso y el título al que enlazaba.
fn quita_aviso_previo(contenido: &[u8]) -> (Vec<u8>, String, bool) {
    let offs = offsets_de_entradas(contenido);
    let (preambulo, resto): (&[u8], &[u8]) = match offs.first() {
        Some(&o) => (&contenido[..o], &contenido[o..]),
        None => (contenido, &[]),
    };
    let Some(m) = PATRON_AVISO.captures(preambulo) else {
        return (contenido.to_vec(), String::new(), false);
    };
    let total = m.get(0).unwrap();
    let previo = String::from_utf8_lossy(&m[1]).into_owned();
    let mut nuevo = Vec::with_capacity(contenido.len());
    nuevo.extend_from_slice(&preambulo[..total.start()]);
    nuevo.extend_from_slice(&preambulo[total.end()..]);
    nuevo.extend_from_slice(resto);
    (nuevo, previo, true)
}

fn nombres_de_archivo_existentes(kb_root: &Path, dir_archivo: &Path, slug: &str) -> Result<Vec<String>> {
    let ruta = kb_root.join(dir_archivo);
    let entradas = match std::fs::read_dir(&ruta) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e).with_context(|| format!("listar {}", ruta.display())),
    };
    let prefijo = format!("{slug}-");
    let mut nombres = Vec::new();
    for entrada in entradas {
        let entrada = entrada.context("leer entrada de archive/log")?;
        let nombre = entrada.file_name().to_string_lossy().into_owned();
        if nombre.starts_with(&prefijo) && nombre.ends_with(".md") {
            nombres.push(nombre);
        }
    }
    Ok(nombres)
}

fn desambigua_nombre_de_archivo(nombre: String, existentes: &[String]) -> String {
    if !existentes.iter().any(|n| n == &nombre) {
        return nombre;
    }
    let base = nombre.strip_suffix(".md").unwrap_or(&nombre);
    let mut n = 2;
    loop {
        let candidato = format!("{base}-{n}.md");
        if !existentes.iter().any(|e| e == &candidato) {
            return candidato;
        }
        n += 1;
    }
}

/// Escribe `datos` en `ruta` atómicamente: temporal en el mismo directorio,
/// `fsync`, `rename`. Cualquier lector ve o el contenido completo viejo o
/// el completo nuevo, nunca uno truncado. Sin `tempfile`: el nombre único
/// sale de PID + tiempo, sin subir esa dependencia de dev a producción.
fn escribe_fichero_atomico(ruta: &Path, datos: &[u8]) -> Result<()> {
    use std::io::Write;
    let dir = ruta.parent().context("ruta sin directorio padre")?;
    let base = ruta.file_name().and_then(|n| n.to_str()).context("nombre de fichero no UTF-8")?;
    let unico = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp = dir.join(format!("{base}.{}.{unico}.tmp", std::process::id()));
    {
        let mut f = std::fs::File::create(&tmp).with_context(|| format!("crear temporal {}", tmp.display()))?;
        f.write_all(datos).with_context(|| format!("escribir temporal {}", tmp.display()))?;
        f.sync_all().with_context(|| format!("fsync temporal {}", tmp.display()))?;
    }
    let resultado = std::fs::rename(&tmp, ruta).with_context(|| format!("renombrar {} a {}", tmp.display(), ruta.display()));
    if resultado.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    resultado
}

/// Como `escribe_fichero_atomico`, pero **nunca** pisa un fichero
/// existente (`hard_link` falla si el destino ya existe, atómicamente, sin
/// listar el directorio antes). Backstop para el archivo: aunque la
/// desambiguación de arriba tuviera un bug, esto solo puede fallar con un
/// error, nunca destruir historia ya archivada. La nota viva, que SÍ debe
/// sobrescribirse en cada rotación, sigue usando `escribe_fichero_atomico`.
fn escribe_fichero_exclusivo(ruta: &Path, datos: &[u8]) -> Result<()> {
    use std::io::Write;
    let dir = ruta.parent().context("ruta sin directorio padre")?;
    let base = ruta.file_name().and_then(|n| n.to_str()).context("nombre de fichero no UTF-8")?;
    let unico = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp = dir.join(format!("{base}.{}.{unico}.tmp", std::process::id()));
    {
        let mut f = std::fs::File::create(&tmp).with_context(|| format!("crear temporal {}", tmp.display()))?;
        f.write_all(datos).with_context(|| format!("escribir temporal {}", tmp.display()))?;
        f.sync_all().with_context(|| format!("fsync temporal {}", tmp.display()))?;
    }
    let resultado = std::fs::hard_link(&tmp, ruta)
        .with_context(|| format!("crear {} (ya existe: no se sobrescribe un archivo)", ruta.display()));
    let _ = std::fs::remove_file(&tmp);
    resultado
}

/// `fsync` del directorio, para que la entrada nueva sea durable antes de
/// tocar la nota viva (si el proceso muere entre las dos escrituras, la
/// duplicación es preferible a la pérdida). Windows no tiene fsync de
/// directorio ni permite abrirlo con `File::open` sin flags que `std::fs`
/// no expone — ahí solo se valida que la ruta es un directorio.
fn fsync_directorio(dir: &Path) -> Result<()> {
    if cfg!(windows) {
        if !dir.is_dir() {
            anyhow::bail!("fsync_directorio: {} no es un directorio", dir.display());
        }
        return Ok(());
    }
    let f = std::fs::File::open(dir).with_context(|| format!("abrir directorio {}", dir.display()))?;
    f.sync_all().with_context(|| format!("fsync de {}", dir.display()))
}

/// Rota una nota de log. Con `escribe=false` nada toca disco: el resultado
/// informa qué pasaría. Con `escribe=true` crea `archive/log/<nombre>` y
/// reescribe la nota como preámbulo + aviso de archivo + cola caliente.
/// Nada se borra jamás: cada byte del original queda en la nota o en el
/// archivo. `nombre_kb` es el prefijo del `permalink` que se escribe en el
/// archivo (Decisión D-4: `exo::nombre_kb()`, no un literal fijo).
pub fn aplica(kb_root: &Path, ruta_rel: &str, presupuesto_caliente: i64, escribe: bool, nombre_kb: &str) -> Result<Resultado> {
    let completa = kb_root.join(ruta_rel);
    let contenido = std::fs::read(&completa).with_context(|| format!("leer {}", completa.display()))?;

    if frontmatter_sin_terminar(&contenido) {
        anyhow::bail!(
            "no se puede rotar {ruta_rel:?}: el contenido empieza con un delimitador de \
             frontmatter \"---\" pero no se encontró el cierre; añade el delimitador de cierre"
        );
    }

    let (contenido, archivo_previo, _) = quita_aviso_previo(&contenido);

    let plan_inicial = parte(&contenido, presupuesto_caliente);
    if plan_inicial.entradas_frias == 0 {
        return Ok(Resultado::vacio(ruta_rel.to_string()));
    }

    let slug = Path::new(ruta_rel)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(ruta_rel)
        .to_string();
    let dir_archivo = Path::new("archive").join("log");
    let existentes = nombres_de_archivo_existentes(kb_root, &dir_archivo, &slug)
        .with_context(|| format!("listar el directorio de archivo para {ruta_rel:?}"))?;

    // Búsqueda de punto fijo: el aviso que se antepone a la cola caliente
    // consume presupuesto, así que hay que volver a partir contra lo que
    // REALMENTE queda tras reservarle sitio, y repetir hasta que el corte
    // se estabilice. `parte` solo puede mover MÁS entradas a frío al bajar
    // el presupuesto (nunca menos), así que `entradas_frias` no decrece de
    // una pasada a otra y el bucle converge en, como mucho,
    // `offsets_de_entradas(contenido).len() + 1` pasadas.
    let aviso_para = |p: &Plan| -> String {
        let nombre = nombre_de_archivo(&slug, p.frio, existentes.len());
        aviso_de_archivo(nombre.trim_end_matches(".md"), p.entradas_frias, p.frio.len())
    };
    let max_iter = offsets_de_entradas(&contenido).len() + 1;
    let mut plan = plan_inicial;
    let mut aviso = aviso_para(&plan);
    for i in 0.. {
        if i >= max_iter {
            anyhow::bail!(
                "error interno: la búsqueda de presupuesto de rotación para {ruta_rel:?} \
                 no convergió tras {max_iter} pasadas"
            );
        }
        let siguiente = parte(&contenido, presupuesto_caliente - aviso.len() as i64);
        let siguiente_aviso = aviso_para(&siguiente);
        let estable = siguiente.entradas_frias == plan.entradas_frias;
        plan = siguiente;
        aviso = siguiente_aviso;
        if estable {
            break;
        }
    }

    let nombre = desambigua_nombre_de_archivo(nombre_de_archivo(&slug, plan.frio, existentes.len()), &existentes);
    let aviso = aviso_de_archivo(nombre.trim_end_matches(".md"), plan.entradas_frias, plan.frio.len());
    let archivo_rel = dir_archivo.join(&nombre);
    let archivo_rel_str = archivo_rel.to_string_lossy().replace('\\', "/");

    let mut resultado = Resultado {
        nota: ruta_rel.to_string(),
        archivo: Some(archivo_rel_str.clone()),
        bytes_movidos: plan.frio.len(),
        entradas_frias: plan.entradas_frias,
        rotado: true,
    };
    if !escribe {
        return Ok(resultado);
    }

    let titulo = nombre.trim_end_matches(".md").to_string();
    let permalink = format!("{nombre_kb}/{}", archivo_rel_str.trim_end_matches(".md"));
    std::fs::create_dir_all(kb_root.join(&dir_archivo)).with_context(|| format!("crear {}", dir_archivo.display()))?;
    let documento = construye_archivo(plan.preambulo, plan.frio, &titulo, &permalink, &archivo_previo);
    escribe_fichero_exclusivo(&kb_root.join(&archivo_rel), &documento)?;
    fsync_directorio(&kb_root.join(&dir_archivo))?;

    let mut salida = Vec::with_capacity(plan.preambulo.len() + aviso.len() + plan.caliente.len());
    salida.extend_from_slice(plan.preambulo);
    salida.extend_from_slice(aviso.as_bytes());
    salida.extend_from_slice(plan.caliente);
    escribe_fichero_atomico(&completa, &salida)?;

    resultado.archivo = Some(archivo_rel_str);
    Ok(resultado)
}
```

- [ ] **C4 — verlos pasar**

  Run: `cd engine && cargo test --release --lib rotacion`
  Expected: `test result: ok. 21 passed; 0 failed` (12 de A+B + 9 de C).

  Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
  Expected: sin salida — limpio.

- [ ] **C5 — commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add engine/src/rotacion.rs
git -C /home/paul/Documentos/proyectos/exo commit -m "feat(d, rotate): aplica() — Apply completo con escritura atómica y punto fijo del presupuesto"
```

#### Step D: cablear `exo rotate`

**Interfaces — Consumes** de `main.rs`: `resuelve_kb`, `envelope::emite`,
`exo::frontmatter::tier`, `exo::nombre_kb()`, y el patrón exacto de
`ratchet_cmd`/`budget_cmd` para el `match` de `ejecuta`/`quiere_json`.

- [ ] **D1 — test CLI que falla**, fichero nuevo `engine/tests/rotar_cli.rs`:

```rust
//! `exo rotate` contra el binario real.
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_exo")
}

fn kb_con_bitacora(cuerpo: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("log")).unwrap();
    std::fs::write(dir.path().join("log/p-bitacora.md"), cuerpo).unwrap();
    dir
}

fn nota_grande() -> String {
    let mut s = String::from("---\ntitle: p-bitacora\ntier: log\n---\n\n# p -- bitacora\n\n");
    for i in 0..10 {
        s.push_str(&format!("## 2026-0{}-01 -- entrada\n{}\n\n", 1 + i % 9, "x".repeat(3000)));
    }
    s
}

#[test]
fn dry_run_no_toca_disco_y_reporta_json() {
    let dir = kb_con_bitacora(&nota_grande());
    let salida = Command::new(bin())
        .args(["rotate", "--json", "--hot-bytes", "8000"])
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(salida.status.success(), "stderr: {}", String::from_utf8_lossy(&salida.stderr));
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(v["schema_version"], 2);
    assert_eq!(v["command"], "rotate");
    assert_eq!(v["data"]["applied"], false);
    assert_eq!(v["data"]["rotations"][0]["rotated"], true);
    assert!(!dir.path().join("archive").exists());
}

#[test]
fn apply_escribe_de_verdad() {
    let dir = kb_con_bitacora(&nota_grande());
    let salida = Command::new(bin())
        .args(["rotate", "--json", "--hot-bytes", "8000", "--apply"])
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(salida.status.success(), "stderr: {}", String::from_utf8_lossy(&salida.stderr));
    assert!(dir.path().join("archive/log").exists());
}

#[test]
fn ignora_notas_que_no_son_tier_log() {
    let dir = kb_con_bitacora("---\ntier: stable\n---\n\n# no rotar\n\n## 2026-01-01 -- a\nx\n");
    let salida = Command::new(bin())
        .args(["rotate", "--hot-bytes", "1"])
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(salida.status.success());
    assert_eq!(String::from_utf8_lossy(&salida.stdout).trim(), "rotate: nothing to rotate");
}

#[test]
fn hot_bytes_no_positivo_falla_sin_ensuciar_stdout() {
    let dir = kb_con_bitacora(&nota_grande());
    let salida = Command::new(bin())
        .args(["rotate", "--hot-bytes", "0", "--json"])
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(!salida.status.success());
    assert!(salida.stdout.is_empty());
}

#[test]
fn sin_directorio_log_no_hay_nada_que_rotar() {
    let dir = tempfile::tempdir().unwrap();
    let salida = Command::new(bin())
        .args(["rotate", "--hot-bytes", "100"])
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(salida.status.success());
    assert_eq!(String::from_utf8_lossy(&salida.stdout).trim(), "rotate: nothing to rotate");
}
```

- [ ] **D2 — verlo fallar**

  Run: `cd engine && cargo test --release --test rotar_cli`
  Expected: FAIL — `exo` no reconoce el subcomando `rotate` (`error: unrecognized subcommand 'rotate'`).

- [ ] **D3 — cablear en `main.rs`**

  En `enum Comando` (junto a `Ratchet`/`Doctor`, `main.rs:80-84`), añadir:

```rust
    /// Divide una bitácora `tier: log` en frío (a `archive/log/`) y
    /// caliente (que se queda). Sin `--apply` es un dry-run: no toca disco.
    /// Solo barre el nivel superior de `log/` — igual que kbx, sin recursión.
    Rotate(ArgsRotate),
```

  Junto a `struct ArgsRatchet` (tras `main.rs:385`), añadir:

```rust
#[derive(clap::Args)]
struct ArgsRotate {
    /// Raíz de la KB. Precedencia: flag > $EXO_KB > config.
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Presupuesto en bytes para la cola caliente que se queda en la nota.
    #[arg(long = "hot-bytes", value_name = "HOT_BYTES", default_value_t = 20480)]
    presupuesto_caliente: i64,
    /// Escribe de verdad. Sin este flag es un dry-run: nada toca disco.
    #[arg(long)]
    apply: bool,
    /// Emite el resultado como envelope JSON en stdout.
    #[arg(long)]
    json: bool,
}
```

  En `quiere_json` (`main.rs:431-448`), añadir `Comando::Rotate(a) => a.json,`
  antes de `Comando::Write(w) =>`. En `ejecuta` (`main.rs:484-502`), añadir
  `Comando::Rotate(args) => rotate_cmd(args),` antes de `Comando::Write(sub) =>`.

  Junto a `ratchet_cmd`/`ratchet_seal_cmd` (tras `main.rs:1367`), añadir:

```rust
/// `exo rotate`: barre `log/` (solo el nivel superior — igual que kbx, sin
/// recursión ni el resto de la KB) y rota cada nota `tier: log` cuya cola
/// fría exceda el presupuesto. Un fallo en una nota no aborta la barrida:
/// se acumula y el exit code final lo refleja con `bail!` (exit 1 — D-3:
/// no es un `GateFallido`, es un fichero que no se pudo procesar).
fn rotate_cmd(args: ArgsRotate) -> Result<()> {
    let kb = resuelve_kb(args.kb)?;
    if args.presupuesto_caliente <= 0 {
        anyhow::bail!("rotate: --hot-bytes tiene que ser > 0, se recibió {}", args.presupuesto_caliente);
    }
    let nombre_kb = exo::nombre_kb().unwrap_or_else(|e| {
        eprintln!("aviso: rotate usa prefijo 'kb' — sin [kb] name: {e:#}");
        "kb".to_string()
    });

    let dir_log = kb.join("log");
    let mut rutas: Vec<PathBuf> = match std::fs::read_dir(&dir_log) {
        Ok(e) => e
            .filter_map(|r| r.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("md"))
            .collect(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(e).with_context(|| format!("leer {}", dir_log.display())),
    };
    rutas.sort();

    let mut resultados = Vec::new();
    let mut fallidas = Vec::new();
    for ruta_abs in rutas {
        let rel = format!("log/{}", ruta_abs.file_name().unwrap().to_string_lossy());
        let contenido = match std::fs::read(&ruta_abs) {
            Ok(c) => c,
            Err(e) => {
                fallidas.push(format!("{rel}: {e}"));
                continue;
            }
        };
        if exo::frontmatter::tier(&String::from_utf8_lossy(&contenido)) != "log" {
            continue;
        }
        match exo::rotacion::aplica(&kb, &rel, args.presupuesto_caliente, args.apply, &nombre_kb) {
            Ok(r) => {
                if r.rotado {
                    resultados.push(r);
                }
            }
            Err(e) => fallidas.push(format!("{rel}: {e}")),
        }
    }

    if args.json {
        envelope::emite(
            "rotate",
            serde_json::json!({ "applied": args.apply, "hot_bytes": args.presupuesto_caliente, "rotations": resultados }),
        );
    } else if resultados.is_empty() {
        println!("rotate: nothing to rotate");
    } else {
        let verbo = if args.apply { "moved" } else { "would move" };
        for r in &resultados {
            println!(
                "{}: {verbo} {} B ({} entries) -> {}",
                r.nota, r.bytes_movidos, r.entradas_frias, r.archivo.as_deref().unwrap_or("")
            );
        }
    }

    if !fallidas.is_empty() {
        for f in &fallidas {
            eprintln!("rotate: {f}");
        }
        anyhow::bail!("{} nota(s) fallaron durante la barrida", fallidas.len());
    }
    Ok(())
}
```

- [ ] **D4 — verlo pasar**

  Run: `cd engine && cargo test --release --test rotar_cli`
  Expected: `test result: ok. 5 passed; 0 failed`.

  Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
  Expected: limpio. (`quiere_json` y `ejecuta` son `match` exhaustivos sin
  `_ =>` — si falta un brazo, esto no compila; es la comprobación de que D3
  no se dejó ningún `match` a medias.)

- [ ] **D5 — commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add engine/src/main.rs engine/tests/rotar_cli.rs
git -C /home/paul/Documentos/proyectos/exo commit -m "feat(d, rotate): cablea 'exo rotate' — ArgsRotate, rotate_cmd, tests de CLI"
```

---

### Task 6: gate de paridad de `rotate` — pre-registro de la Task 1

**Lane:** mecánica (el criterio y las divergencias ya están escritos en el
pre-registro; adjudicar es aplicar, no decidir). **Depende de:** Task 2
(kbx compilado + KB copiada) y Task 5 (binario exo con `rotate`). **Oráculo:**
el bloque §Comandos §Rotate de
`2026-09-14-campana-d-preregistro-paridad-rotate-stale.md`, con el control
rojo-verde del instrumento en verde ANTES del PASA/NO PASA real.

**Files:**
- Modify: `docs/superpowers/plans/2026-09-14-campana-d-preregistro-paridad-rotate-stale.md`
  (rellenar "### Rotate" bajo "## Registro de la corrida").

- [ ] **Step 1: verificar la precondición de la Decisión D-4** (el
  `permalink` del archivo depende de `[kb] name`)

  Run: `grep -A1 '\[kb\]' ~/.exo/config.toml | grep name`
  Expected: `name = "wisdom-paul"` (o el valor anotado en la Task 2, Step 4).
  Si difiere, anotarlo — el criterio ya excluye `permalink` de la
  comparación estricta precisamente por esto (divergencia 6 del
  pre-registro), así que no bloquea, solo cambia qué valor se espera ver.

- [ ] **Step 2: control rojo-verde del instrumento, antes de confiar en un
  PASA**

```bash
export PATH="$HOME/.local/go/bin:$HOME/.local/bin:$PATH"
EXO=/home/paul/Documentos/proyectos/exo/engine/target/release/exo
KBX=/tmp/campana-d/kbx
KB_BASE=/tmp/campana-d/kb-base
CTRL=$(mktemp -d /tmp/campana-d/ctrl-apply.XXXX)
git clone --no-local "$KB_BASE" "$CTRL"
# --hot-bytes 1 fuerza a exo a archivar TODO menos la última entrada; si
# kbx (con un --hot-bytes normal) archiva menos, el diff de archive/ tiene
# que dar REVISAR. El lado de control de kbx escribe en SU PROPIO clon
# desechable (CTRL_GO), nunca en $KB_BASE — $KB_BASE es de solo lectura
# para el resto del pre-registro (declarado así en "## Comandos").
CTRL_GO=$(mktemp -d /tmp/campana-d/ctrl-go.XXXX)
git clone --no-local "$KB_BASE" "$CTRL_GO"
"$EXO" rotate --kb "$CTRL" --apply --hot-bytes 1 --json > /dev/null
"$KBX" rotate --kb "$CTRL_GO" --apply --json > /dev/null
diff -qr "$CTRL/archive" "$CTRL_GO/archive" 2>&1 | head -5
echo "^ tiene que mostrar diferencia (REVISAR) — si no muestra nada, el comparador no está midiendo, PARAR y revisar el script antes de seguir"
rm -rf "$CTRL" "$CTRL_GO"
```

  Expected: el `diff -qr` muestra diferencias (más ficheros o distinto
  contenido en el lado de `--hot-bytes 1`). Si sale limpio (sin diferencia),
  **no seguir** — el `diff -qr` no está detectando nada y el gate real de
  abajo no vale.

- [ ] **Step 3: correr el bloque §Comandos completo del pre-registro**
  (dry-run, `--apply` por binario en su propio clon, y los tres `diff`)

  Ejecutar literalmente el script de la sección "## Comandos" →
  "# --- rotate: …" del fichero
  `2026-09-14-campana-d-preregistro-paridad-rotate-stale.md`, sustituyendo
  `./target/release/exo` por la ruta real
  (`/home/paul/Documentos/proyectos/exo/engine/target/release/exo`).

- [ ] **Step 4: adjudicar contra las 6 divergencias ya declaradas** — no se
  reinterpretan, se aplican. Prestar atención particular a la divergencia 4
  (disambiguación) y 6 (permalink, ya cubierta en el Step 1).

- [ ] **Step 5: rellenar "### Rotate" bajo "## Registro de la corrida"** en
  el propio pre-registro, con los valores reales (fecha, commits, cada
  `coincide: sí/no`, divergencias observadas, PASA/NO PASA).

- [ ] **Step 6: commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add docs/superpowers/plans/2026-09-14-campana-d-preregistro-paridad-rotate-stale.md
git -C /home/paul/Documentos/proyectos/exo commit -m "docs(d, gate-rotate): registro de la corrida de paridad de rotate"
```

---

### Task 7: port de `stale` — `engine/src/obsolescencia.rs`, TDD con los 5 axiomas

**Lane:** mecánica (fórmula y pesos ya FIRMADOS por Paul,
`.superpowers/fabrica/PENDIENTE-PAUL-m4-stale-formula.md`, "Adjudicación de
Paul" — cero decisión de producto pendiente). **Depende de:** Task 1.
**No depende de Task 2.** **Oráculo global:**
`cd engine && cargo test --release --lib obsolescencia && cargo test --release --test obsolescencia_cli && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`.

**Files:**
- Create: `engine/src/obsolescencia.rs`
- Modify: `engine/src/lib.rs` (`pub mod obsolescencia;`)
- Modify: `engine/src/main.rs` (ver Step D)
- Create: `engine/tests/obsolescencia_cli.rs`

**Interfaces — Produces:**
```rust
pub const EDAD_SIN_COMMIT_DIAS: i64 = 36_500;
pub const PESO_TIER_CORE: f64 = 1.5;
pub const PESO_TIER_STABLE: f64 = 1.0;
pub const PESO_TIER_LOG: f64 = 0.5;
pub const PESO_TIER_SIN_TIER: f64 = PESO_TIER_LOG;
pub const PESO_DECAIMIENTO_DEGREE: f64 = 0.2;
pub struct Puntuacion(f64); // Serialize como f64 normal (ver divergencia 5 del pre-registro)
pub fn puntua(edad_dias: i64, degree: i64, tier: &str) -> Puntuacion;
pub struct Nota { pub path: String, pub permalink: String, pub tier: String, pub ultimo_commit: String, pub sin_commit: bool, pub edad_dias: i64, pub degree: i64, pub score: Puntuacion } // Serialize
pub struct Informe { pub now: String, pub notes: Vec<Nota> } // Serialize
pub fn calcula(conn: &rusqlite::Connection, kb: &Path, excluidos: &[&str], ahora_epoch: i64) -> Result<Informe>;
pub fn epoch_utc_de_iso8601(marca: &str) -> Result<i64>;
```
CLI: `Comando::Stale(ArgsStale)`, función `stale_cmd`.

#### Step A: la fórmula — `Puntuacion`, `puntua()`

- [ ] **A1 — tests que fallan**, `engine/src/obsolescencia.rs` (fichero
  nuevo):

```rust
//! `exo stale` — urgencia de actualización de cada nota: edad de su
//! último commit, grado en el grafo de relaciones y tier, combinados en
//! una puntuación. Puerto de `kbx/internal/stale` (`fe46443`). La fórmula
//! y los pesos están FIRMADOS por Paul
//! (`.superpowers/fabrica/PENDIENTE-PAUL-m4-stale-formula.md`,
//! "Adjudicación de Paul", 2026-07-11) — no son una decisión de esta
//! campaña, se copian verbatim.

use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;

/// Edad fija para una nota sin commits (spec-review H1: constante, nunca
/// relativa a la corrida ni a la composición del fixture).
pub const EDAD_SIN_COMMIT_DIAS: i64 = 36_500;

/// Pesos de la fórmula (bloque agrupado, un re-peso es un edit de una
/// línea — mismo contrato que el `const` block de kbx). `PESO_TIER_CORE`/
/// `STABLE`/`LOG` escalan la edad cruda por tier; `PESO_TIER_SIN_TIER`
/// (tier ausente o ilegal) puntúa como `log`, la urgencia más baja —
/// decisión ya tomada: `exo budget` ya expone `notier` como su propia
/// alarma de higiene, y catapultar NOTIER al tope del ranking de `stale`
/// diluiría la señal que `stale` existe para dar. `PESO_DECAIMIENTO_DEGREE`
/// descuenta la urgencia por conectividad como `1/(1+degree*peso)`: degree
/// 0 no divide por cero, y el descuento crece monótono con degree.
pub const PESO_TIER_CORE: f64 = 1.5;
pub const PESO_TIER_STABLE: f64 = 1.0;
pub const PESO_TIER_LOG: f64 = 0.5;
pub const PESO_TIER_SIN_TIER: f64 = PESO_TIER_LOG;
pub const PESO_DECAIMIENTO_DEGREE: f64 = 0.2;

fn peso_de_tier(tier: &str) -> f64 {
    match tier {
        "core" => PESO_TIER_CORE,
        "stable" => PESO_TIER_STABLE,
        "log" => PESO_TIER_LOG,
        _ => PESO_TIER_SIN_TIER,
    }
}

/// Urgencia de una nota, redondeada a 2 decimales antes de envolver.
/// Serializa como un `f64` normal (formato más corto que redondea exacto,
/// p. ej. `29.0` en vez de `29.00`). kbx usa `Score.MarshalJSON` para fijar
/// el texto en 2 decimales; el port **no** replica ese formateo — el
/// feature de `serde_json` que haría falta rompe `#[serde(untagged)]` con
/// floats en todo el grafo de dependencias (`tokenizers` lo usa en 12
/// sitios, ver Global Constraints). El valor es idéntico, solo el texto
/// difiere — divergencia 5 del pre-registro.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize)]
pub struct Puntuacion(f64);

impl Puntuacion {
    pub fn valor(self) -> f64 {
        self.0
    }
}

/// Combina edad, degree y tier en una urgencia (M4 spec §3, "Formula"):
/// edad escalada por el peso del tier, descontada por conectividad.
/// Redondeada a 2 decimales antes de envolver.
pub fn puntua(edad_dias: i64, degree: i64, tier: &str) -> Puntuacion {
    let cruda = edad_dias as f64 * peso_de_tier(tier) / (1.0 + degree as f64 * PESO_DECAIMIENTO_DEGREE);
    Puntuacion((cruda * 100.0).round() / 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn puntua_reproduce_el_golden_de_kbx() {
        // Nueve pares únicos (edad, degree, tier) => score, verbatim de
        // cmd/kbx/testdata/stale_golden.json (fe46443) — dos de las 10
        // notas del golden repiten el mismo par (log/alpha-bitacora y
        // sesiones/2026-07-01, age_days=13/degree=2/tier=log => 4.64), así
        // que hay 9 pares únicos, no 10.
        let casos = [
            (43, 2, "core", 46.07),
            (43, 1, "stable", 35.83),
            (29, 0, "stable", 29.00),
            (29, 0, "", 14.50),
            (29, 5, "stable", 14.50),
            (29, 1, "", 12.08),
            (13, 1, "stable", 10.83),
            (29, 2, "", 10.36),
            (13, 2, "log", 4.64),
        ];
        for (edad, degree, tier, esperado) in casos {
            let got = puntua(edad, degree, tier).valor();
            assert!(
                (got - esperado).abs() < 1e-9,
                "puntua({edad}, {degree}, {tier:?}) = {got}, want {esperado}"
            );
        }
    }

    // --- Axiomas de la spec M4 §3 (mismos 5 que TestScore_Axiom{1..5} en kbx) ---

    const TIERS: [&str; 4] = ["core", "stable", "log", ""];

    #[test]
    fn axioma_1_determinismo() {
        for tier in TIERS {
            for edad in [0, 1, 13, 29, 43, EDAD_SIN_COMMIT_DIAS] {
                for degree in [0, 1, 2, 4, 10] {
                    let primero = puntua(edad, degree, tier).valor();
                    for _ in 0..5 {
                        assert_eq!(puntua(edad, degree, tier).valor(), primero);
                    }
                }
            }
        }
    }

    #[test]
    fn axioma_2_monotonia_de_edad() {
        let edades = [0, 1, 5, 13, 29, 43, 100, 1000, EDAD_SIN_COMMIT_DIAS];
        for tier in TIERS {
            for degree in [0, 1, 2, 4, 10] {
                for w in edades.windows(2) {
                    let (joven, vieja) = (puntua(w[0], degree, tier).valor(), puntua(w[1], degree, tier).valor());
                    assert!(vieja >= joven, "tier={tier} degree={degree}: {vieja} < {joven}");
                }
            }
        }
    }

    #[test]
    fn axioma_3_monotonia_de_degree() {
        let degrees = [0, 1, 2, 3, 4, 10, 50];
        for tier in TIERS {
            for edad in [1, 13, 29, 43, 1000] {
                for w in degrees.windows(2) {
                    let (menos, mas) = (puntua(edad, w[0], tier).valor(), puntua(edad, w[1], tier).valor());
                    assert!(mas <= menos, "tier={tier} edad={edad}: {mas} > {menos}");
                }
            }
        }
    }

    #[test]
    fn axioma_4_orden_de_tier() {
        for edad in [1, 13, 29, 43, 1000] {
            for degree in [0, 1, 2, 4, 10] {
                let core = puntua(edad, degree, "core").valor();
                let stable = puntua(edad, degree, "stable").valor();
                let log = puntua(edad, degree, "log").valor();
                assert!(core >= stable, "edad={edad} degree={degree}: core {core} < stable {stable}");
                assert!(stable >= log, "edad={edad} degree={degree}: stable {stable} < log {log}");
            }
        }
    }

    #[test]
    fn axioma_5_forma_finita_no_negativa_y_dos_decimales() {
        for tier in TIERS {
            for edad in [0, 1, 13, 1000, EDAD_SIN_COMMIT_DIAS] {
                for degree in [0, 1, 4, 50] {
                    let s = puntua(edad, degree, tier).valor();
                    assert!(s.is_finite() && s >= 0.0, "puntua({edad},{degree},{tier:?}) = {s}");
                }
            }
        }
    }
}
```

- [ ] **A2 — verlos fallar, declarar el módulo, verlos pasar**

  En `engine/src/lib.rs`: `pub mod obsolescencia;`

  Run: `cd engine && cargo test --release --lib obsolescencia`
  Expected: `test result: ok. 6 passed; 0 failed`.

- [ ] **A3 — commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add engine/src/obsolescencia.rs engine/src/lib.rs
git -C /home/paul/Documentos/proyectos/exo commit -m "feat(d, stale): puntua() — formula firmada, 5 axiomas"
```

#### Step B: ISO-8601 ↔ epoch, sin crate de fechas

- [ ] **B1 — tests que fallan**, añadidos a `engine/src/obsolescencia.rs`
  (producción antes de `mod tests`):

```rust
/// Días desde 1970-01-01 hasta la fecha civil dada (calendario
/// gregoriano). Algoritmo de Howard Hinnant
/// (howardhinnant.github.io/date_algorithms.html, dominio público);
/// aritmética entera exacta para cualquier año, incluidos los anteriores a
/// 1970 (da negativo). exo no trae ningún crate de fechas —
/// `indexer::git_epoch_de` sortea el problema pidiéndole el epoch a git
/// directamente (`%at`); aquí hace falta además la cadena ISO para
/// `last_commit`, así que se resuelve con esta función pura en vez de
/// añadir una dependencia para dos conversiones de calendario.
fn dias_desde_epoch_civil(anio: i64, mes: i64, dia: i64) -> i64 {
    let y = if mes <= 2 { anio - 1 } else { anio };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let mp = (mes + 9) % 12;
    let doy = (153 * mp + 2) / 5 + dia - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Inverso de `dias_desde_epoch_civil`: fecha civil para un número de días
/// desde 1970-01-01.
fn civil_desde_dias_epoch(dias: i64) -> (i64, i64, i64) {
    let z = dias + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn parsea_offset_minutos(off: &str) -> Result<i64> {
    let (signo_txt, resto) = off.split_at(1);
    let signo = if signo_txt == "-" { -1 } else { 1 };
    let mut partes = resto.split(':');
    let (Some(Ok(hh)), Some(Ok(mm))) = (partes.next().map(str::parse::<i64>), partes.next().map(str::parse::<i64>)) else {
        anyhow::bail!("huso horario ilegible: {off:?}");
    };
    Ok(signo * (hh * 60 + mm))
}

/// Convierte una fecha-hora en formato `git log --format=%aI` (ISO-8601
/// estricto: `YYYY-MM-DDTHH:MM:SS±HH:MM`, el que produce
/// `gitx::ultimo_commit`, o con sufijo `Z`) a segundos UTC desde epoch.
pub fn epoch_utc_de_iso8601(marca: &str) -> Result<i64> {
    let (fecha, resto) = marca.split_once('T').with_context(|| format!("fecha ISO-8601 sin 'T': {marca:?}"))?;
    let mut pf = fecha.split('-');
    let (Some(Ok(anio)), Some(Ok(mes)), Some(Ok(dia))) =
        (pf.next().map(str::parse::<i64>), pf.next().map(str::parse::<i64>), pf.next().map(str::parse::<i64>))
    else {
        anyhow::bail!("fecha ISO-8601 ilegible: {marca:?}");
    };

    let (hms, offset_min) = if let Some(hms) = resto.strip_suffix('Z') {
        (hms, 0)
    } else if let Some(i) = resto.rfind(['+', '-']) {
        let (hms, off) = resto.split_at(i);
        (hms, parsea_offset_minutos(off)?)
    } else {
        anyhow::bail!("fecha ISO-8601 sin huso horario: {marca:?}");
    };

    let mut ph = hms.split(':');
    let (Some(Ok(h)), Some(Ok(m)), Some(Ok(s))) =
        (ph.next().map(str::parse::<i64>), ph.next().map(str::parse::<i64>), ph.next().map(str::parse::<i64>))
    else {
        anyhow::bail!("hora ISO-8601 ilegible: {marca:?}");
    };

    let dias = dias_desde_epoch_civil(anio, mes, dia);
    Ok(dias * 86_400 + h * 3600 + m * 60 + s - offset_min * 60)
}

/// Formatea segundos-epoch UTC como RFC3339 con sufijo `Z`, sin fracción
/// de segundo — igual que `now.UTC().Format(time.RFC3339)` en Go.
pub fn formatea_rfc3339_utc(epoch: i64) -> String {
    let dias = epoch.div_euclid(86_400);
    let seg_del_dia = epoch.rem_euclid(86_400);
    let (anio, mes, dia) = civil_desde_dias_epoch(dias);
    let (h, m, s) = (seg_del_dia / 3600, (seg_del_dia / 60) % 60, seg_del_dia % 60);
    format!("{anio:04}-{mes:02}-{dia:02}T{h:02}:{m:02}:{s:02}Z")
}

/// `floor((ahora - commit) / 86400s)` sobre instantes UTC absolutos
/// (spec-review M1: "duration-based, never calendar-day arithmetic in any
/// timezone"). `div_euclid` para que un `ahora` anterior al commit
/// redondee hacia abajo, no hacia cero.
fn edad_en_dias(ahora_epoch: i64, commit_epoch: i64) -> i64 {
    (ahora_epoch - commit_epoch).div_euclid(86_400)
}
```

  Y tests en `mod tests`:

```rust
    #[test]
    fn epoch_de_iso8601_reproduce_el_now_del_golden() {
        // "2026-07-14T22:00:00Z", el `now` del golden de kbx.
        let e = epoch_utc_de_iso8601("2026-07-14T22:00:00Z").unwrap();
        assert_eq!(formatea_rfc3339_utc(e), "2026-07-14T22:00:00Z");
    }

    #[test]
    fn epoch_de_iso8601_respeta_el_huso_horario() {
        // 10:00 +02:00 == 08:00 Z.
        let con_offset = epoch_utc_de_iso8601("2026-06-01T10:00:00+02:00").unwrap();
        let en_z = epoch_utc_de_iso8601("2026-06-01T08:00:00Z").unwrap();
        assert_eq!(con_offset, en_z);
    }

    #[test]
    fn formatea_rfc3339_utc_es_el_inverso_de_epoch_utc_de_iso8601() {
        for marca in ["1970-01-01T00:00:00Z", "2026-01-01T00:00:00Z", "2026-12-31T23:59:59Z", "2000-02-29T12:00:00Z"] {
            let e = epoch_utc_de_iso8601(marca).unwrap();
            assert_eq!(formatea_rfc3339_utc(e), marca, "round-trip de {marca}");
        }
    }

    #[test]
    fn edad_en_dias_coincide_con_el_golden() {
        // now=2026-07-14T22:00:00Z, last_commit core/core-index=2026-06-01T10:00:00+02:00 => age_days=43.
        let ahora = epoch_utc_de_iso8601("2026-07-14T22:00:00Z").unwrap();
        let commit = epoch_utc_de_iso8601("2026-06-01T10:00:00+02:00").unwrap();
        assert_eq!(edad_en_dias(ahora, commit), 43);
    }
```

- [ ] **B2 — verlos fallar, ya están implementados arriba, verlos pasar**

  Run: `cd engine && cargo test --release --lib obsolescencia`
  Expected: `test result: ok. 10 passed; 0 failed` (6 de A + 4 de B).

- [ ] **B3 — commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add engine/src/obsolescencia.rs
git -C /home/paul/Documentos/proyectos/exo commit -m "feat(d, stale): ISO-8601 <-> epoch sin crate de fechas (Hinnant), edad_en_dias"
```

#### Step C: `calcula` — degree por SQL, tier del disco, git, orden

- [ ] **C1 — tests que fallan**, añadidos a `mod tests` (usan un `Connection`
  de `rusqlite` en memoria más un repo git real de usar y tirar, mismo
  patrón que `engine/tests/targets_cli.rs::kb_con_indice`):

```rust
    fn kb_con_git_y_db(notas: &[(&str, &str, &str)]) -> (tempfile::TempDir, rusqlite::Connection) {
        // notas: (ruta_rel, tier, cuerpo_extra)
        let dir = tempfile::tempdir().unwrap();
        let kb = dir.path().to_path_buf();
        let cfg = kb.join("gitconfig-vacio");
        std::fs::write(&cfg, "").unwrap();
        let git = |args: &[&str], fecha: Option<&str>| {
            let mut cmd = std::process::Command::new("git");
            cmd.arg("-C").arg(&kb).args(args)
                .env("GIT_CONFIG_GLOBAL", &cfg).env("GIT_CONFIG_SYSTEM", &cfg)
                .env("GIT_AUTHOR_NAME", "f").env("GIT_AUTHOR_EMAIL", "f@k.local")
                .env("GIT_COMMITTER_NAME", "f").env("GIT_COMMITTER_EMAIL", "f@k.local");
            if let Some(f) = fecha {
                cmd.env("GIT_AUTHOR_DATE", f).env("GIT_COMMITTER_DATE", f);
            }
            assert!(cmd.output().unwrap().status.success(), "git {args:?}");
        };
        for (ruta, tier, extra) in notas {
            let full = kb.join(ruta);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(&full, format!("---\ntier: {tier}\n---\n\n# {ruta}\n\n{extra}\n")).unwrap();
        }
        git(&["init", "-q"], None);
        git(&["add", "."], None);
        git(&["commit", "-q", "-m", "inicial"], Some("2026-06-01T10:00:00+02:00"));

        let conn = rusqlite::Connection::open_in_memory().unwrap();
        // `crate::`, no `exo::`: este test vive dentro de `engine/src/`
        // (mod tests del propio lib.rs), que no tiene
        // `extern crate self as exo` — `exo::` solo resuelve desde fuera
        // del crate (p. ej. `engine/tests/*.rs`).
        crate::schema::crea_schema(&conn).unwrap();
        for (i, (ruta, _, _)) in notas.iter().enumerate() {
            conn.execute(
                "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch) VALUES (?1, ?2, ?3, 'note', 0.0, NULL)",
                rusqlite::params![format!("kb/{i}"), ruta, format!("n{i}")],
            ).unwrap();
        }
        (dir, conn)
    }

    #[test]
    fn calcula_lee_tier_del_disco_y_degree_cero_sin_aristas() {
        let (dir, conn) = kb_con_git_y_db(&[("a.md", "stable", ""), ("log/b.md", "log", "")]);
        let ahora = epoch_utc_de_iso8601("2026-09-14T00:00:00Z").unwrap();
        let informe = calcula(&conn, dir.path(), &["archive", "docs", ".superpowers"], ahora).unwrap();
        assert_eq!(informe.notes.len(), 2);
        let a = informe.notes.iter().find(|n| n.path == "a.md").unwrap();
        assert_eq!(a.tier, "stable");
        assert_eq!(a.degree, 0);
        assert!(!a.sin_commit);
    }

    #[test]
    fn calcula_cuenta_degree_como_origen_mas_destino() {
        let (dir, conn) = kb_con_git_y_db(&[("a.md", "core", ""), ("b.md", "core", "")]);
        conn.execute(
            "INSERT INTO aristas (origen, destino_texto, destino_permalink) VALUES ('kb/0', 'b', 'kb/1')",
            [],
        ).unwrap();
        let ahora = epoch_utc_de_iso8601("2026-09-14T00:00:00Z").unwrap();
        let informe = calcula(&conn, dir.path(), &[], ahora).unwrap();
        let a = informe.notes.iter().find(|n| n.path == "a.md").unwrap();
        let b = informe.notes.iter().find(|n| n.path == "b.md").unwrap();
        assert_eq!(a.degree, 1, "a es origen de una arista");
        assert_eq!(b.degree, 1, "b es destino de esa arista");
    }

    #[test]
    fn calcula_excluye_por_primer_segmento_como_walker_excluida() {
        let (dir, conn) = kb_con_git_y_db(&[("archive/vieja.md", "log", ""), ("viva.md", "stable", "")]);
        let ahora = epoch_utc_de_iso8601("2026-09-14T00:00:00Z").unwrap();
        let informe = calcula(&conn, dir.path(), &["archive", "docs", ".superpowers"], ahora).unwrap();
        assert_eq!(informe.notes.len(), 1);
        assert_eq!(informe.notes[0].path, "viva.md");
    }

    #[test]
    fn calcula_una_nota_sin_commits_usa_la_edad_centinela() {
        let (dir, conn) = kb_con_git_y_db(&[("a.md", "core", "")]);
        std::fs::write(dir.path().join("nueva.md"), "---\ntier: stable\n---\n\n# nueva\n").unwrap();
        conn.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch) VALUES ('kb/nueva', 'nueva.md', 'nueva', 'note', 0.0, NULL)",
            [],
        ).unwrap();
        let ahora = epoch_utc_de_iso8601("2026-09-14T00:00:00Z").unwrap();
        let informe = calcula(&conn, dir.path(), &[], ahora).unwrap();
        let n = informe.notes.iter().find(|n| n.path == "nueva.md").unwrap();
        assert!(n.sin_commit);
        assert_eq!(n.edad_dias, EDAD_SIN_COMMIT_DIAS);
        assert_eq!(n.ultimo_commit, "");
    }

    #[test]
    fn calcula_tolera_un_fichero_no_utf8_en_vez_de_reventar() {
        // El fixture de kbx incluye informe.pdf, tipo='report', 0 aristas:
        // no debe tumbar la corrida entera.
        let (dir, conn) = kb_con_git_y_db(&[("a.md", "core", "")]);
        std::fs::write(dir.path().join("informe.pdf"), [0xFF, 0xFE, 0x00, 0x01]).unwrap();
        conn.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch) VALUES ('kb/informe', 'informe.pdf', 'informe', 'report', 0.0, NULL)",
            [],
        ).unwrap();
        let ahora = epoch_utc_de_iso8601("2026-09-14T00:00:00Z").unwrap();
        // `gitx::ultimo_commit` devuelve `Ok("")` para un fichero sin
        // commit (no un `Err`) — igual que el test vecino
        // `calcula_una_nota_sin_commits_usa_la_edad_centinela`, así que
        // informe.pdf no tumba la corrida: degrada a tier "" (bytes no
        // UTF-8, ningún tier válido) y a la edad centinela.
        let informe = calcula(&conn, dir.path(), &[], ahora).unwrap();
        let n = informe.notes.iter().find(|n| n.path == "informe.pdf").unwrap();
        assert_eq!(n.tier, "");
        assert!(n.sin_commit);
        assert_eq!(n.edad_dias, EDAD_SIN_COMMIT_DIAS);
    }

    #[test]
    fn calcula_ordena_desc_por_score_con_path_como_desempate() {
        let (dir, conn) = kb_con_git_y_db(&[("z-vieja.md", "core", ""), ("a-vieja.md", "core", "")]);
        let ahora = epoch_utc_de_iso8601("2026-09-14T00:00:00Z").unwrap();
        let informe = calcula(&conn, dir.path(), &[], ahora).unwrap();
        // Mismo tier, mismo commit => mismo score; desempata por path ascendente.
        assert_eq!(informe.notes[0].path, "a-vieja.md");
        assert_eq!(informe.notes[1].path, "z-vieja.md");
    }

    #[test]
    fn calcula_tier_ilegal_normaliza_a_vacio() {
        let (dir, conn) = kb_con_git_y_db(&[("a.md", "urgentisimo", "")]);
        let ahora = epoch_utc_de_iso8601("2026-09-14T00:00:00Z").unwrap();
        let informe = calcula(&conn, dir.path(), &[], ahora).unwrap();
        assert_eq!(informe.notes[0].tier, "");
    }
```

- [ ] **C2 — verlos fallar**

  Run: `cd engine && cargo test --release --lib obsolescencia`
  Expected: FAIL de compilación — `calcula`, `Informe`, `Nota` no existen.

- [ ] **C3 — implementación**, añadida antes de `#[cfg(test)]`:

```rust
#[derive(Serialize)]
pub struct Nota {
    pub path: String,
    pub permalink: String,
    pub tier: String,
    #[serde(rename = "last_commit")]
    pub ultimo_commit: String,
    #[serde(rename = "uncommitted")]
    pub sin_commit: bool,
    #[serde(rename = "age_days")]
    pub edad_dias: i64,
    pub degree: i64,
    pub score: Puntuacion,
}

/// `Notes` nunca es `null` en el JSON (M4 spec §3): `Vec::new()` serializa
/// como `[]`, no hace falta ningún tratamiento especial.
#[derive(Serialize)]
pub struct Informe {
    pub now: String,
    pub notes: Vec<Nota>,
}

struct FilaNota {
    permalink: String,
    ruta: String,
    degree: i64,
}

/// Cuenta, por nota, las filas de `aristas` donde el permalink de la nota
/// aparece como `origen` O como `destino_permalink` (degree 0 = huérfana),
/// como suma de dos conteos independientes. SQL literal de
/// `stale.degreeQuery` (`fe46443`) — ya sin el filtro `tipo='note'` que
/// escondía 57 de 138 notas reales, retirado en M6-04 T3 (comentario del
/// propio Go, replicado aquí porque exo hereda esa misma corrección).
const CONSULTA_DEGREE: &str = "SELECT notas.permalink,
       notas.ruta,
       (SELECT COUNT(*) FROM aristas WHERE aristas.origen = notas.permalink) +
       (SELECT COUNT(*) FROM aristas WHERE aristas.destino_permalink = notas.permalink) AS degree
FROM notas
ORDER BY notas.ruta";

fn normaliza_tier(tier: &str) -> String {
    if crate::presupuesto::TIERS.contains(&tier) {
        tier.to_string()
    } else {
        String::new()
    }
}

/// Urgencia de actualización de cada nota de la KB bajo `kb`: edad de su
/// último commit, grado en el grafo de relaciones y tier, combinados en
/// `puntua`. Lee degree+permalink del índice (solo lectura), tier del
/// frontmatter en disco, y último commit de git (`gitx::ultimo_commit`,
/// fail-loud — mismo contrato que `targets`). Ordenado desc por score,
/// `path` ascendente como desempate (M4 spec §3, axioma 5).
pub fn calcula(conn: &rusqlite::Connection, kb: &Path, excluidos: &[&str], ahora_epoch: i64) -> Result<Informe> {
    let mut stmt = conn.prepare(CONSULTA_DEGREE).context("stale: preparar la consulta de degree")?;
    let filas: Vec<FilaNota> = stmt
        .query_map([], |f| {
            Ok(FilaNota { permalink: f.get(0)?, ruta: f.get(1)?, degree: f.get(2)? })
        })
        .context("stale: ejecutar la consulta de degree")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("stale: leer las filas de degree")?;

    let mut notas = Vec::new();
    for fila in filas {
        if crate::walker::excluida(&fila.ruta, excluidos) {
            continue;
        }

        let absoluta = kb.join(&fila.ruta);
        let bytes = std::fs::read(&absoluta).with_context(|| format!("stale: leer {}", absoluta.display()))?;
        // Lossy, no `read_to_string`: el fixture de kbx incluye un
        // `informe.pdf` no-UTF8 entre las notas (M6-04 T3 quitó el filtro
        // por tipo), y degradar el tier a "" en vez de reventar es el
        // mismo contrato best-effort que `objetivos::busca_objetivos` ya
        // aplica para lectura de disco.
        let contenido = String::from_utf8_lossy(&bytes);
        let tier = normaliza_tier(&crate::frontmatter::tier(&contenido));

        let ultimo = crate::gitx::ultimo_commit(kb, &fila.ruta).with_context(|| format!("stale: {}", fila.ruta))?;
        let (ultimo_commit, sin_commit, edad_dias) = if ultimo.is_empty() {
            (String::new(), true, EDAD_SIN_COMMIT_DIAS)
        } else {
            let epoch = epoch_utc_de_iso8601(&ultimo).with_context(|| format!("stale: parsear last_commit de {}", fila.ruta))?;
            (ultimo, false, edad_en_dias(ahora_epoch, epoch))
        };

        notas.push(Nota {
            path: fila.ruta,
            permalink: fila.permalink,
            tier: tier.clone(),
            ultimo_commit,
            sin_commit,
            edad_dias,
            degree: fila.degree,
            score: puntua(edad_dias, fila.degree, &tier),
        });
    }

    notas.sort_by(|a, b| {
        b.score
            .valor()
            .partial_cmp(&a.score.valor())
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
    });

    Ok(Informe { now: formatea_rfc3339_utc(ahora_epoch), notes: notas })
}
```

- [ ] **C4 — verlos pasar**

  Run: `cd engine && cargo test --release --lib obsolescencia`
  Expected: `test result: ok. 17 passed; 0 failed` (10 de A+B + 7 de C).

  Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
  Expected: limpio.

- [ ] **C5 — commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add engine/src/obsolescencia.rs
git -C /home/paul/Documentos/proyectos/exo commit -m "feat(d, stale): calcula() — degree por SQL, tier de disco, git fail-loud, orden"
```

#### Step D: cablear `exo stale`

- [ ] **D1 — test CLI que falla**, fichero nuevo `engine/tests/obsolescencia_cli.rs`:

```rust
//! `exo stale` contra el binario real, y contra el golden de kbx (oráculo
//! de valores, no comparación binario-contra-binario — ver el pre-registro
//! de la campaña D, sección "Stale").
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_exo")
}

fn kb_con_indice_y_git() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let kb = dir.path().to_path_buf();
    let cfg = kb.join("gitconfig-vacio");
    std::fs::write(&cfg, "").unwrap();
    let git = |args: &[&str]| {
        let s = std::process::Command::new("git")
            .arg("-C").arg(&kb).args(args)
            .env("GIT_CONFIG_GLOBAL", &cfg).env("GIT_CONFIG_SYSTEM", &cfg)
            .env("GIT_AUTHOR_NAME", "f").env("GIT_AUTHOR_EMAIL", "f@k.local")
            .env("GIT_COMMITTER_NAME", "f").env("GIT_COMMITTER_EMAIL", "f@k.local")
            .env("GIT_AUTHOR_DATE", "2026-06-01T10:00:00+02:00")
            .env("GIT_COMMITTER_DATE", "2026-06-01T10:00:00+02:00")
            .output().unwrap();
        assert!(s.status.success(), "git {args:?}");
    };
    std::fs::write(kb.join("a.md"), "---\ntier: core\n---\n\n# a\n").unwrap();
    git(&["init", "-q"]);
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "inicial"]);

    let db = dir.path().join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch) VALUES ('kb/a', 'a.md', 'a', 'note', 0.0, NULL)",
        [],
    ).unwrap();
    drop(conn);
    (dir, db)
}

#[test]
fn el_envelope_lleva_command_stale_y_schema_version_2() {
    let (dir, db) = kb_con_indice_y_git();
    let salida = Command::new(bin())
        .args(["stale", "--json", "--now", "2026-09-14T00:00:00Z"])
        .arg("--db").arg(&db).arg("--kb").arg(dir.path())
        .output().unwrap();
    assert!(salida.status.success(), "stderr: {}", String::from_utf8_lossy(&salida.stderr));
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(v["schema_version"], 2);
    assert_eq!(v["command"], "stale");
    assert_eq!(v["data"]["now"], "2026-09-14T00:00:00Z");
    assert_eq!(v["data"]["notes"][0]["path"], "a.md");
    assert_eq!(v["data"]["notes"][0]["tier"], "core");
    // El score ya no se fuerza a 2 decimales fijos en el texto (ver
    // Global Constraints y divergencia 5 del pre-registro): el valor es lo
    // que importa, no el formato. Se compara por
    // VALOR, vía el `Value` ya parseado, no con un patrón sobre el texto
    // crudo.
    let score = v["data"]["notes"][0]["score"].as_f64().unwrap();
    assert!(score > 0.0, "a.md tiene commit y tier=core: el score debe ser positivo, no {score}");
}

#[test]
fn sin_now_usa_el_reloj_de_pared_y_no_falla() {
    let (dir, db) = kb_con_indice_y_git();
    let salida = Command::new(bin())
        .args(["stale", "--json"])
        .arg("--db").arg(&db).arg("--kb").arg(dir.path())
        .output().unwrap();
    assert!(salida.status.success(), "stderr: {}", String::from_utf8_lossy(&salida.stderr));
}

#[test]
fn now_invalido_falla_como_error_de_uso() {
    let (dir, db) = kb_con_indice_y_git();
    let salida = Command::new(bin())
        .args(["stale", "--now", "no-es-una-fecha"])
        .arg("--db").arg(&db).arg("--kb").arg(dir.path())
        .output().unwrap();
    assert!(!salida.status.success());
}

#[test]
fn la_salida_humana_nombra_el_path_y_el_score() {
    let (dir, db) = kb_con_indice_y_git();
    let salida = Command::new(bin())
        .args(["stale", "--now", "2026-09-14T00:00:00Z"])
        .arg("--db").arg(&db).arg("--kb").arg(dir.path())
        .output().unwrap();
    assert!(salida.status.success());
    let texto = String::from_utf8_lossy(&salida.stdout);
    assert!(texto.contains("a.md"), "salida: {texto}");
    assert!(texto.contains("score="), "salida: {texto}");
}
```

- [ ] **D2 — verlo fallar**

  Run: `cd engine && cargo test --release --test obsolescencia_cli`
  Expected: FAIL — subcomando `stale` no reconocido.

- [ ] **D3 — cablear en `main.rs`**

  En `enum Comando` (junto a `Rotate`, si la Task 5 ya mergeó en esta rama
  local — si no, junto a `Ratchet`/`Doctor`):

```rust
    /// Urgencia de actualización de cada nota: edad de su último commit,
    /// grado en el grafo de relaciones y tier, combinados en una
    /// puntuación. Solo lectura.
    Stale(ArgsStale),
```

  Junto a `ArgsRatchet`/`ArgsRotate`:

```rust
#[derive(clap::Args)]
struct ArgsStale {
    /// Fichero SQLite del índice. Precedencia: flag > $EXO_DB > config.
    #[arg(long)]
    db: Option<PathBuf>,
    /// Raíz de la KB. Precedencia: flag > $EXO_KB > config.
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Override del reloj, RFC3339 (por defecto: la hora real; los tests
    /// deterministas siempre lo pasan).
    #[arg(long)]
    now: Option<String>,
    /// Emite el resultado como envelope JSON en stdout.
    #[arg(long)]
    json: bool,
}
```

  En `quiere_json`: `Comando::Stale(a) => a.json,`. En `ejecuta`:
  `Comando::Stale(args) => stale_cmd(args),`.

  Junto a `rotate_cmd`:

```rust
/// `exo stale`: urgencia de actualización por nota (`obsolescencia::calcula`).
/// Solo lectura — el único exit no-cero es 1, un error de IO/parseo; la
/// obsolescencia en sí es información, no un veredicto de gate (kbx: "the
/// only non-zero exit is 2 (IO/usage)" — misma idea, exit distinto porque
/// en exo 2 es de clap).
fn stale_cmd(args: ArgsStale) -> Result<()> {
    let db_ruta = resuelve_db(args.db)?;
    if !db_ruta.exists() {
        anyhow::bail!("DB no encontrada: {} — corre `exo index` primero", db_ruta.display());
    }
    let kb = resuelve_kb(args.kb)?;
    let conn = exo::abre_db(&db_ruta)?;

    let ahora_epoch = match args.now {
        Some(marca) => exo::obsolescencia::epoch_utc_de_iso8601(&marca)
            .with_context(|| format!("stale: --now inválido: {marca:?}"))?,
        None => std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .context("stale: reloj del sistema anterior a 1970")?
            .as_secs() as i64,
    };

    let informe = exo::obsolescencia::calcula(&conn, &kb, &exo::presupuesto::EXCLUIDOS, ahora_epoch)?;

    if args.json {
        envelope::emite("stale", serde_json::to_value(&informe)?);
    } else {
        println!("now: {}", informe.now);
        for n in &informe.notes {
            let commit = if n.sin_commit { "(uncommitted)".to_string() } else { n.ultimo_commit.clone() };
            println!(
                "{:<40} tier={:<6} age_days={:<6} degree={:<3} last_commit={} score={:.2}",
                n.path, n.tier, n.edad_dias, n.degree, commit, n.score.valor()
            );
        }
    }
    Ok(())
}
```

- [ ] **D4 — verlo pasar**

  Run: `cd engine && cargo test --release --test obsolescencia_cli`
  Expected: `test result: ok. 4 passed; 0 failed`.

  Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
  Expected: limpio.

- [ ] **D5 — commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add engine/src/main.rs engine/tests/obsolescencia_cli.rs
git -C /home/paul/Documentos/proyectos/exo commit -m "feat(d, stale): cablea 'exo stale' — ArgsStale, stale_cmd, tests de CLI"
```

---

### Task 8: gate de paridad de `stale` — pre-registro de la Task 1

**Lane:** mecánica. **Depende de:** Task 2, Task 7, y Task 6 ya mergeada en
la rama local — Task 6 (`d-port-rotate`) y esta Task 8 (`d-port-stale`)
editan secciones adyacentes del mismo pre-registro ("### Rotate" y
"### Stale" bajo "## Registro de la corrida"); correrlas en paralelo desde
ramas distintas sobre el mismo fichero es la clase de conflicto que un
merge automático no garantiza resolver limpio. Task 8 espera a que
`d-port-rotate` esté mergeada antes de rellenar su sección. **Oráculo:** el
bloque §Comandos §Stale del pre-registro, con el control rojo-verde en
verde antes del PASA/NO PASA.

**Files:**
- Modify: `docs/superpowers/plans/2026-09-14-campana-d-preregistro-paridad-rotate-stale.md`
  (rellenar "### Stale" bajo "## Registro de la corrida").

- [ ] **Step 1: el oráculo de valores contra el golden ya está cubierto —
  verificarlo, no repetirlo**

  El test `puntua_reproduce_el_golden_de_kbx` (Task 7, Step A) ya compara
  la fórmula contra los 9 pares únicos `(edad, degree, tier) => score` del
  golden de kbx (con tolerancia de punto flotante); `edad_en_dias_coincide_con_el_golden`
  (Task 7, Step B) ya cubre el caso `age_days=43` del mismo golden. El
  fixture sintético completo de 10 notas **no** se reconstruye en Rust —
  el criterio congelado en el pre-registro (§Stale, PASA-1) es justo estos
  dos tests, no una comparación binario-contra-binario del fixture.
  Confirmar que los dos siguen en verde:

  Run: `cd engine && cargo test --release --lib obsolescencia::tests::puntua_reproduce_el_golden_de_kbx obsolescencia::tests::edad_en_dias_coincide_con_el_golden`
  Expected: `test result: ok. 2 passed`. Si alguno falla, el gate de la KB
  real de abajo no tiene sentido correrlo — arreglar primero.

- [ ] **Step 2: control rojo-verde del instrumento**

```bash
export PATH="$HOME/.local/go/bin:$HOME/.local/bin:$PATH"
EXO=/home/paul/Documentos/proyectos/exo/engine/target/release/exo
NOW="2026-09-14T12:00:00+02:00"
# Referencia real.
"$EXO" stale --db /tmp/campana-d/index.db --kb /tmp/campana-d/kb-base --now "$NOW" --json \
  | jq -S '.data.notes | map({path, score})' > /tmp/campana-d/rs-stale-ctrl-real.json
# Con PESO_TIER_CORE deliberadamente roto a 1.6 (edit de una línea en
# engine/src/obsolescencia.rs, revertido justo después) y recompilado:
sed -i 's/pub const PESO_TIER_CORE: f64 = 1.5;/pub const PESO_TIER_CORE: f64 = 1.6;/' /home/paul/Documentos/proyectos/exo/engine/src/obsolescencia.rs
(cd /home/paul/Documentos/proyectos/exo/engine && cargo build --release)
"$EXO" stale --db /tmp/campana-d/index.db --kb /tmp/campana-d/kb-base --now "$NOW" --json \
  | jq -S '.data.notes | map({path, score})' > /tmp/campana-d/rs-stale-ctrl-roto.json
diff -u /tmp/campana-d/rs-stale-ctrl-real.json /tmp/campana-d/rs-stale-ctrl-roto.json | head -10
echo "^ tiene que mostrar diferencia en al menos las notas tier=core — si no muestra nada, el comparador no mide, PARAR"
# Revertir el sabotaje y recompilar ANTES de seguir.
sed -i 's/pub const PESO_TIER_CORE: f64 = 1.6;/pub const PESO_TIER_CORE: f64 = 1.5;/' /home/paul/Documentos/proyectos/exo/engine/src/obsolescencia.rs
(cd /home/paul/Documentos/proyectos/exo/engine && cargo build --release)
git -C /home/paul/Documentos/proyectos/exo diff --stat -- engine/src/obsolescencia.rs   # tiene que salir vacío
```

  Expected: el `diff` del medio muestra cambios en las notas `tier: core`
  (score escalado ×1.6/1.5 respecto al real); el `git diff --stat` final
  sale **vacío** (el sabotaje quedó revertido, no commiteado).

- [ ] **Step 3: correr el bloque §Comandos §Stale del pre-registro** (KB
  real, `--now` fijo en los dos lados, más la comparación de orden)

  Ejecutar literalmente la sección "# --- stale: …" de "## Comandos" en
  `2026-09-14-campana-d-preregistro-paridad-rotate-stale.md`.

- [ ] **Step 4: adjudicar contra las 5 divergencias declaradas** (fórmula
  ya firmada, sin `--stale-exclude`, exclusión por primer segmento vía
  `walker::excluida`, reuso de `gitx::ultimo_commit`, formato JSON del
  score) — no se reinterpretan.

- [ ] **Step 5: rellenar "### Stale" bajo "## Registro de la corrida"** en
  el pre-registro, con el resultado real.

- [ ] **Step 6: commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add docs/superpowers/plans/2026-09-14-campana-d-preregistro-paridad-rotate-stale.md
git -C /home/paul/Documentos/proyectos/exo commit -m "docs(d, gate-stale): registro de la corrida de paridad de stale"
```

---

### Task 9: reapuntar consumidores — ningún skill/hook invoca ya `kbx`

**Lane:** mecánica. **Depende de:** Task 5 y Task 7 (necesita que `exo
rotate`/`exo stale` existan de verdad antes de documentarlos como
disponibles). **Oráculo:** `grep -rn "KBX_BIN\|kbx " plugins/exo/skills/distill/
plugins/exo/scripts/kb-precommit.sh` no encuentra nada salvo la nota
histórica que se deja adrede (ver Step 5), más una verificación de campo
(Step 6).

**Files:**
- Modify: `plugins/exo/skills/distill/SKILL.md`
- Modify: `plugins/exo/skills/distill/rotacion.md`
- Modify: `plugins/exo/skills/distill/chequeos.md`
- Modify: `plugins/exo/scripts/kb-precommit.sh`
- Modify: `docs/arquitectura.md` (§3.8, tabla de superficie de CLI)

- [ ] **Step 1: `distill/SKILL.md` — la sección de resolución de rutas
  pierde `$KBX_BIN`**

  Reemplazar (líneas 19-46 a fecha de este plan):

```markdown
### Resolución de rutas — antes de cualquier paso

Antes de ejecutar cualquier paso de este procedimiento, resuelve dos valores
con el mismo seam que usa `plugins/exo/scripts/test-contrato-engine.sh`:

- `$KB_ROOT` — raíz de la KB:
  `${EXO_KB:-$(exo config --json | jq -r '.data.kb.path // empty')}`. Si sale
  vacío, es **abstención ruidosa**: para y dile a Paul que ni `$EXO_KB` ni
  `exo config --json` resolvieron nada — no sigas con el procedimiento.
- `$KBX_BIN` — binario `kbx`: `${KBX_BIN:-$(command -v kbx)}`. `kbx` es una
  herramienta externa que puede no estar instalada en esta máquina (p.ej.
  Windows, donde su build todavía no está decidido). Si `$KBX_BIN` sale
  vacío, dilo explícitamente y **salta cada paso que dependa de `kbx`** en
  vez de fingir que corrió — no hay abstención silenciosa que valga para un
  paso que simplemente no se ejecutó. Es la misma disciplina que ya aplica
  el paso "Falla-fuerte" del Budget check (para con mensaje accionable si
  el binario que toca falta): generalízala al resto de usos de `kbx` en este
  procedimiento.
- `$EXO_BIN` — binario `exo`: `${EXO_BIN:-$(command -v exo)}`.

**Este skill invoca dos binarios, y lo dice por escrito**: `exo` para
`budget`/`ratchet`/`lint` (cutover G4c) y `kbx` para `rotate`/`stale`/
`diff-since`, que todavía no tienen destino en `exo`. Un skill que finge
haber migrado del todo es una trampa para el día que `kbx` no esté
instalado — mejor declarar la frontera tal cual está.

Todos los comandos de las secciones siguientes usan `$KB_ROOT`, `$KBX_BIN` y
`$EXO_BIN` — ninguna ruta literal.
```

  por:

```markdown
### Resolución de rutas — antes de cualquier paso

Antes de ejecutar cualquier paso de este procedimiento, resuelve dos valores
con el mismo seam que usa `plugins/exo/scripts/test-contrato-engine.sh`:

- `$KB_ROOT` — raíz de la KB:
  `${EXO_KB:-$(exo config --json | jq -r '.data.kb.path // empty')}`. Si sale
  vacío, es **abstención ruidosa**: para y dile a Paul que ni `$EXO_KB` ni
  `exo config --json` resolvieron nada — no sigas con el procedimiento. La
  KB tiene que ser la raíz de un repo git: `budget`/`ratchet`/`lint`/
  `rotate`/`stale` lo exigen (`exo::gitx::es_repo_git`, decisión A2 de G4b);
  sobre una KB sin versionar, `budget`/`lint` funcionan igual y el resto
  falla con un mensaje que nombra la condición y el remedio (`git init`).
- `$EXO_BIN` — binario `exo`: `${EXO_BIN:-$(command -v exo)}`.

**Desde la campaña D (2026-09-14) este skill invoca un solo binario.** `exo`
cubre `budget`/`ratchet`/`lint` (cutover G4c) y, desde ahora,
`rotate`/`stale` también. `history` y `diff-since` **no existen en `exo`**
—no se portan, se sustituyen por `git` directo (paso 3)— así que ningún
paso de este procedimiento depende ya de `kbx`.

Todos los comandos de las secciones siguientes usan `$KB_ROOT` y `$EXO_BIN`
— ninguna ruta literal, y ningún `$KBX_BIN`.
```

- [ ] **Step 2: paso 0 (rotación) — `$EXO_BIN`, sin caveat de feature-branch**

  Reemplazar:

```markdown
Corre `$KBX_BIN rotate --kb $KB_ROOT --json`. Si `data.rotations` trae entradas, sigue `rotacion.md`; si el binario no trae `rotate`, sáltalo y ve al paso 1.
```

  por:

```markdown
Corre `$EXO_BIN rotate --kb $KB_ROOT --json`. Si `data.rotations` trae entradas, sigue `rotacion.md`; si viene vacío, sigue directo al paso 1.
```

- [ ] **Step 3: paso 1b (gate de deriva) — `$EXO_BIN stale`**

  Reemplazar:

```markdown
Corre `$EXO_BIN lint --json` y `$KBX_BIN stale --json`. Señales de inyección rota: `chequeos.md`.
```

  por:

```markdown
Corre `$EXO_BIN lint --json` y `$EXO_BIN stale --json`. Señales de inyección rota: `chequeos.md`.
```

- [ ] **Step 4: paso 3 (archivar sesiones) — `diff-since` sustituido por
  `git` directo**

  Reemplazar:

```markdown
### 3. Archivar sesiones de frentes cerrados

**Escanea solo lo cambiado.** No re-escanees toda la KB: corre
`$KBX_BIN diff-since distill/last --json`
(`{data:{ref,resolved,notes:[{path,permalink,status,insertions,deletions}]}}`)
para ver qué notas cambiaron desde la última consolidación.

**Bootstrap (el tag aún no existe — `git tag -l` está vacío hoy):** si
`distill/last` no existe, `diff-since` fallará al resolver el ref. Eso **no**
es fallo-fuerte: haz un **full scan** (sin `diff-since`) esta vez. Al terminar
el paso 5 (commit), crea/mueve el tag al HEAD del repo KB:

    git -C $KB_ROOT tag -f distill/last HEAD

Es la única mutación del repo KB que hace esta skill más allá de commitear notas.
```

  por:

```markdown
### 3. Archivar sesiones de frentes cerrados

**Escanea solo lo cambiado.** No re-escanees toda la KB: `exo` no trae
`diff-since` (decisión de la campaña D — no se porta: con git ya delante,
duplicarlo dentro del binario no añade nada que `git diff`/`git log` no den
ya). Corre en su lugar:

    git -C $KB_ROOT diff --stat distill/last..HEAD -- '*.md'
    git -C $KB_ROOT diff --name-status distill/last..HEAD -- '*.md'

para ver qué notas cambiaron desde la última consolidación (el segundo
comando da el estado por fichero: `A`/`M`/`D`).

**Bootstrap (el tag aún no existe — `git tag -l` está vacío hoy):** si
`distill/last` no existe, los dos `git diff` de arriba fallan al resolver la
referencia. Eso **no** es fallo-fuerte: haz un **full scan** (sin diff) esta
vez. Al terminar el paso 5 (commit), crea/mueve el tag al HEAD del repo KB:

    git -C $KB_ROOT tag -f distill/last HEAD

Es la única mutación del repo KB que hace esta skill más allá de commitear notas.
```

- [ ] **Step 5: `rotacion.md` — sin `$KBX_BIN` ni el caveat de "puede no
  traer el subcomando"**

  Reemplazar:

```markdown
Corre primero en seco y revisa el resultado:

    $KBX_BIN rotate --kb $KB_ROOT --json

Requiere un build de `kbx` que incluya `rotate`: el binario instalado puede no
traer todavía el subcomando, porque la feature vive en una rama sin mergear.
Si no está disponible, sáltalo y continúa directo al paso 1.
```

  por:

```markdown
Corre primero en seco y revisa el resultado:

    $EXO_BIN rotate --kb $KB_ROOT --json
```

- [ ] **Step 6: `chequeos.md` — retirar la descripción de `schema_drift`
  (murió en G4b) y `$EXO_BIN stale`**

  Reemplazar:

```markdown
**Falla-fuerte:** si el binario no está o el schema-canary rompe (lo verás como
un `schema_drift` en `lint`, ver abajo), **para** con un mensaje accionable
(`exo no está → cargo build --release en engine/ + copia a
$HOME/.local/bin/exo(.exe)`, "schema drift → el binario kbx y el binario exo
están desincronizados: reinstala el que vaya atrasado (`make install` en kbx,
`cargo build --release` + copia en exo) y vuelve a correr"). No degrades a mano:
/distill es offline y deliberado, el fallo ruidoso es correcto.
```

  por:

```markdown
**Falla-fuerte:** si el binario no está, **para** con un mensaje accionable
(`exo no está → cargo build --release en engine/ + copia a
$HOME/.local/bin/exo(.exe)`). No degrades a mano: /distill es offline y
deliberado, el fallo ruidoso es correcto. (El check `schema_drift` que esta
sección citaba murió en G4b — `exo lint` emite 6 tipos de finding, no 7;
existía solo mientras kbx y exo convivían contra el mismo schema,
`engine/src/lint.rs:1-5` — así que ya no hay "schema drift" que mirar.)
```

  Y, en el mismo fichero, reemplazar:

```markdown
- **Priorización:** corre `$KBX_BIN stale --json`
```

  por:

```markdown
- **Priorización:** corre `$EXO_BIN stale --json`
```

- [ ] **Step 7: `kb-precommit.sh` — el remedio del gate ya no exige `kbx`**

  Reemplazar (líneas 59-63):

```bash
  1. Si la nota creció con histórico: PÁRTELA. Mueve lo fechado a su
     bitácora (log/<slug>-bitacora.md). El canon se queda con el destilado.
  2. Si la bitácora es la que ha crecido: kbx rotate --kb <kb> --apply
     archiva su cola fría en archive/log/. (Sigue en kbx: exo no tiene
     todavía el verbo rotate.)
```

  por:

```bash
  1. Si la nota creció con histórico: PÁRTELA. Mueve lo fechado a su
     bitácora (log/<slug>-bitacora.md). El canon se queda con el destilado.
  2. Si la bitácora es la que ha crecido: exo rotate --kb <kb> --apply
     archiva su cola fría en archive/log/.
```

- [ ] **Step 8: `docs/arquitectura.md` §3.8 — añadir `rotate` y `stale` a
  la tabla de superficie de CLI**

  Insertar, después de la fila de `exo targets` y antes de la de
  `exo doctor`:

```markdown
| `exo rotate` | Divide una bitácora `tier: log` en frío (a `archive/log/`) y caliente, portado de `kbx rotate`. Solo el nivel superior de `log/`, sin recursión | `--hot-bytes` (20480), `--apply`, `--kb`, `--json` |
| `exo stale` | Urgencia de actualización por nota (edad de último commit, degree, tier), portado de `kbx stale`. Solo lectura | `--now`, `--db`, `--kb`, `--json` |
```

- [ ] **Step 9: verificación de campo, no solo grep** — construir el
  binario de esta rama (sin instalarlo: es un build sin mergear, no pisa
  `~/.local/bin/exo` — Global Constraints) y comprobar en un clon
  desechable que `distill` (leído a mano, no ejecutado como agente aquí)
  ya no tiene ningún paso que cite `kbx`:

```bash
cargo build --release --manifest-path /home/paul/Documentos/proyectos/exo/engine/Cargo.toml
EXO=/home/paul/Documentos/proyectos/exo/engine/target/release/exo   # por ruta, nunca copiado a ~/.local/bin
grep -rn "KBX_BIN\|kbx " /home/paul/Documentos/proyectos/exo/plugins/exo/skills/distill/ /home/paul/Documentos/proyectos/exo/plugins/exo/scripts/kb-precommit.sh
```

  Expected: sin resultados (o, si algo aparece, que sea prosa histórica
  deliberada — a fecha de este plan no se deja ninguna). Si aparece algo
  inesperado, es un paso de este Step 1-7 que quedó a medias.

- [ ] **Step 10: commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add plugins/exo/skills/distill/SKILL.md plugins/exo/skills/distill/rotacion.md plugins/exo/skills/distill/chequeos.md plugins/exo/scripts/kb-precommit.sh docs/arquitectura.md
git -C /home/paul/Documentos/proyectos/exo commit -m "docs(d, cutover): distill/kb-precommit.sh/arquitectura.md ya no invocan kbx"
```

---

### Task 10: backlog — cerrar con evidencia, corregir lo caducado

**Lane:** mecánica. **Depende de:** Tasks 3, 4, 6, 8, 9 — o de su estado
`encolado` con nota, si Task 2 falló. **Oráculo:** cada edición cita el
commit o el fichero que la respalda; ninguna se basa en "debería estar
hecho", todas en lo que el resto de las tasks dejó escrito.

**Files:**
- Modify: `docs/backlog.md`

**Nota de re-localización**: si la campaña E ya mergeó (orden de merge
fijado en §Dependencias con E), los números de línea de abajo se habrán
movido. Localizar cada item **por su texto** (las citas literales entre
comillas), nunca por el número — la propia disciplina que el dictamen del
consultor ya pedía.

- [ ] **Step 1: corregir la nota "kbx local divergido" — CADUCADA,
  independiente del resultado de los gates**

  Reemplazar, dentro del item que empieza
  `**(G4c, Task 14) Gate de paridad "ratchet"+"targets": pendiente de
  máquina Linux, mismo prerequisito.**` (`docs/backlog.md:1381-1389` a fecha
  de este plan), la frase:

```markdown
El repo `kbx`
  local está divergido de `fe46443` (`f0d0564`, 1 por delante y 18 por
  detrás, conflicto en `budget.go`, mismo plan:876-877): quien vaya a
  compilar `fe46443` para el gate necesita saberlo.
```

  por:

```markdown
**CADUCADO (campaña D, 2026-09-14): el repo `kbx` local ya NO está
  divergido.** Verificado con
  `git -C ~/Documentos/proyectos/kbx merge-base --is-ancestor fe46443 HEAD`
  (sale 0 — `fe46443` es ancestro de `HEAD`, hoy `ee2b27c`). No hace falta
  reconciliar nada para compilar `fe46443`; la Task 2 de la campaña D lo
  hizo desde un worktree sin tocar el checkout local.
```

- [ ] **Step 2: si Tasks 3 y 4 dieron PASA — mover el ítem de gates a
  "Cerrado con evidencia"; si no, dejarlo abierto con el estado real**

  **Si el "Registro de la corrida" de `2026-09-02-g4a-preregistro-targets.md`
  y de `2026-09-09-g4c-preregistro-ratchet.md` dicen los dos PASA:** mover
  el ítem completo (`docs/backlog.md:1381-1389`, ya con la corrección del
  Step 1) a la sección `## Cerrado con evidencia` del propio `backlog.md`,
  como `[x]`, añadiendo la cita de los dos commits de las Tasks 3 y 4.

  **Si alguno dio NO PASA, o Task 2 falló y quedaron `encolado`:** el ítem
  se queda en su sitio (abierto), pero se actualiza con el estado real —
  qué corrió, qué no, y por qué (citando la Task 2/3/4 correspondiente) —
  en vez de dejar la redacción vieja ("pendiente de máquina Linux") cuando
  ya no es verdad que sea SOLO por falta de máquina Linux.

- [ ] **Step 3: cerrar "(G4c, Task 14) El cutover kbx→exo es parcial"**

  Reemplazar el ítem completo (`docs/backlog.md:1405-1413`):

```markdown
- [ ] **(G4c, Task 14) El cutover kbx→exo es parcial: `rotate`, `stale`,
  `history` y `diff-since` siguen sin portar.** Verificado hoy: `exo --help`
  no lista esos cuatro verbos.
  `plugins/exo/skills/distill/SKILL.md` sigue necesitando el binario `kbx`
  por `rotate` (`:60`), `stale` (`:146`) y `diff-since` (`:225`). `rotate` es
  el candidato natural a G4d, y no es cosmético: es el remedio que
  `kb-precommit.sh` prescribe en su mensaje de rechazo (`kbx rotate --kb <kb>
  --apply`) cuando el gate muerde — mientras no exista en `exo`, ese remedio
  sigue exigiendo tener `kbx` instalado.
```

  por (solo si Task 5 y Task 7 mergearon, es decir el port existe de
  verdad — verificar con `grep -c "Rotate(ArgsRotate)\|Stale(ArgsStale)"
  engine/src/main.rs`, esperado 2):

```markdown
- [x] **(G4c, Task 14) El cutover kbx→exo YA NO es parcial en lo que puede
  serlo: `rotate` y `stale` están portados** (campaña D, 2026-09-14,
  `engine/src/rotacion.rs` y `engine/src/obsolescencia.rs`). `exo --help`
  lista los dos verbos. Ningún consumidor de `plugins/exo/` invoca ya
  `kbx` (verificado: `grep -rn "KBX_BIN\|kbx " plugins/exo/skills/distill/
  plugins/exo/scripts/kb-precommit.sh` sin resultados). `history` y
  `diff-since` **no se portan por decisión, no por pendiente**: se
  sustituyen por `git diff`/`git log` directo en `distill/SKILL.md` paso 3
  — con git ya delante, duplicarlo dentro del binario no añadía nada. `kbx`
  puede seguir existiendo como herramienta; deja de ser dependencia de
  `exo`.
```

- [ ] **Step 4: cerrar la acción (b) de "«exo genérico» sigue siendo el
  plugin de Paul"**

  Dentro del ítem que empieza `**(revisión 2026-09-04) «exo genérico» sigue
  siendo el plugin de Paul.**` (`docs/backlog.md:243-296`), en su bloque
  `**Acción:**` final, reemplazar:

```markdown
  **Acción:** (a) sustituir «Paul» por «el usuario»/«el dueño de la KB» y
  `kb-demo` por el nombre resuelto vía `exo config` en los cuatro scripts y
  dos skills; (b) lo que queda de `kbx` (`rotate`, `stale`) lo lleva el item
  de Baja de G4c Task 14: o se porta, o se declara dependencia opcional en
  `instalacion.md` y `distill` se abstiene entera sin él;
  (c) ~~la release con binario de G5~~ **hecha el 2026-09-11**.
```

  por:

```markdown
  **Acción:** (a) sustituir «Paul» por «el usuario»/«el dueño de la KB» y
  `kb-demo` por el nombre resuelto vía `exo config` en los cuatro scripts y
  dos skills — **sigue abierta**, fuera del alcance de la campaña D;
  (b) ~~lo que queda de `kbx` (`rotate`, `stale`)~~ **hecha el 2026-09-14
  (campaña D): se portó, no se declaró opcional** — `rotate` y `stale`
  viven en `exo`, `distill` ya no depende de `kbx` para nada;
  (c) ~~la release con binario de G5~~ **hecha el 2026-09-11**. El ítem
  sigue abierto solo por (a).
```

- [ ] **Step 5: anotar la nota "kbx local divergido" del diagnóstico del
  dictamen como afirmación caducada corregida** (meta — no es un ítem del
  backlog, es una entrada de higiene sobre el propio proceso): si el
  backlog tiene una fila de "Estado" tabular al principio del fichero
  (como la que ya existe para las campañas A/B/C, `docs/backlog.md:1-30`
  a fecha de este plan), añadir una fila para D siguiendo el mismo formato
  — commit de referencia, qué cerró, qué queda abierto (D-4 si Paul no lo
  ha zanjado, la acción (a) de "exo genérico").

- [ ] **Step 6: commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add docs/backlog.md
git -C /home/paul/Documentos/proyectos/exo commit -m "docs(d, backlog): cierra gates de paridad y cutover de rotate/stale, corrige la nota caducada de kbx divergido"
```

---

## Residuo declarado

- **La acción (a) del ítem "«exo genérico» sigue siendo el plugin de
  Paul"** (sustituir «Paul»/`kb-demo` por nombres resueltos vía `exo
  config`) sigue abierta. No es de esta campaña ni de E, por lo que se sabe
  de su alcance declarado.
- **Decisión D-4** (permalink del archivo: `nombre_kb()` vs literal
  `"wisdom-paul"`) queda con la recomendación ya aplicada en el código,
  pero formalmente abierta hasta que Paul la vea. Revertirla es un cambio
  de una línea en `aplica` (Task 5, Step C3).
- **`kbx` como herramienta sigue existiendo** en
  `~/Documentos/proyectos/kbx` — este plan no lo borra ni lo deprecia como
  repo, solo dejó de ser una dependencia de `exo`/el plugin. Qué hacer con
  el repo `kbx` en sí (archivarlo, seguir manteniéndolo aparte) no es
  decisión de esta campaña.
- **El gate de paridad de `budget`/`lint` (G4b) sigue con su "Registro"
  vacío** (`2026-09-04-g4b-preregistro-budget-lint.md`) — no estaba en el
  alcance que el dictamen ni Paul pidieron para D (solo `targets` y
  `ratchet`). Queda como candidato de una futura campaña si hace falta.
- **Si Task 2 (prerrequisitos) falla** por falta de `gcc`/toolchain C en
  esta máquina concreta, las Tasks 3, 4, 6 y 8 quedan `encolado` y el
  backlog (Task 10, Step 2) lo refleja tal cual — el port (Tasks 5, 7, 9)
  no se ve afectado, y el plan entero sigue siendo mergeable con los gates
  pendientes de una máquina con Go.

## Cierre de campaña (review final de rama)

Antes de pedir el gate de merge del consultor Fable (config de fábrica,
§Ejecución de gates): `cd engine && cargo test --release && cargo fmt
--check && cargo clippy --all-targets --locked -- -D warnings &&
./scripts/test-hermetico.sh`, todo en verde; `git diff --stat` contra
`main` no toca ningún fichero de la campaña E declarado en
§Dependencias con la campaña E; y las dos secciones "Registro de la
corrida" del pre-registro de rotate/stale, más los dos registros de G4a/G4c,
están rellenadas (con PASA, NO PASA, o `encolado` explícito — nunca en
blanco).

