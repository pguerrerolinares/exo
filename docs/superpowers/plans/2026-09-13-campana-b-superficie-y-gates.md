# Campaña B — Superficie pública y gates, sin cambio de comportamiento del motor

> **For agentic workers:** REQUIRED SUB-SKILL: usa `exo:orchestrate` para
> ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`) para
> tracking. Sesión-fábrica: `paul-profile:fabrica` con
> `.superpowers/fabrica/config.md` (el orquestador lo integra; este plan no lo
> edita).

**Goal:** que lo que ve alguien de fuera (`exo --help`, README, versiones,
docs) cuente el producto y no la historia de su construcción, y que CI cace
por sí sola las clases de rotura de shell que ya llegaron a release — sin
cambiar nada de lo que el engine calcula.

**Architecture:** tres ramas que no se pisan. (1) **engine** — un refactor
puro de `trinquete::comprueba_contra` (H15) y una pasada sobre los
doc-comments de clap en `engine/src/main.rs` con un test de `--help` que
impide regresar (H9, H8). (2) **thin + gates** — primero se tocan los scripts
(helper `_truncate-payload.sh`, H20; matcher del reflejo de orquestador
limpio, H21), después se ponen los dos gates estáticos que los juzgan
(exec-bit ampliado, H12; shellcheck, H11), cada uno demostrado falsable en
local; `distill/SKILL.md` se parte en procedimiento + ficheros de carga bajo
demanda (H22). (3) **docs** — gate de versiones (H16), README de producto con
ejemplo de extremo a extremo (H6), `reports/` (H26), `tier` en docs (H18) y,
al final, el backlog sincronizado con todo lo anterior y con CI (H13). Cinco
decisiones son de Paul y bloquean tareas concretas: ver «Decisiones abiertas».

**Tech Stack:** Rust 2024 (crate `exo` en `engine/`, MSRV 1.95) · `clap`
4.6.2 derive · `regex` 1.13.1 (dependencia normal, usable desde tests) ·
`tempfile` 3.14 (dev) · bash (Git Bash en Windows) + `jq` · ShellCheck 0.11.0
(binario pineado por SHA256) · GitHub Actions (`.github/workflows/ci.yml`,
`release.yml`) · Python 3 solo como herramienta de edición del ejecutor (nada
de Python entra al repo).

## Estado de verificación de los hallazgos (2026-09-13, contra `3c1918f`)

Los 14 hallazgos se re-verificaron en el árbol real; **ninguno se descarta**.
Cuatro se reproducen con matices que cambian el plan:

- **H12** es más estrecho de lo dicho y se confirma: `scripts/test-exec-bit.sh:16`
  filtra `plugins/*.sh`; los tres ejecutables sin extensión de
  `plugins/exo/skills/orchestrate/scripts/` quedan fuera (medido: con
  `task-brief` en 100644 el gate actual sigue dando `[OK]`). Fuera de
  `plugins/` hay 5 `.py` con shebang en 100644 (`evals/retrieval-fase0/harness/`),
  que se invocan con `python3` — el gate NO se amplía a todo el repo.
- **H11**: shellcheck 0.11.0 (`-x -P SCRIPTDIR`) sobre los 39 scripts
  versionados da **47 avisos, 3 de severidad warning** (2×SC2140 falsos positivos
  de glob, 1×SC2155) y 44 notas (36×SC2015 en `test-a1-gate.sh`). **Cero bugs
  reales**: todo se arregla (1) o se justifica en el sitio. Números a
  re-medir tras A.
- **H26**: `reports/` tiene 5 ficheros, pero solo **2** se citan por ruta
  (`m2-07-impl-report.md`, `m2-07-report.md`), desde **4** ficheros y 5
  líneas — uno de ellos código vivo (`engine/src/main.rs:14`). El «5 ficheros
  citados desde evals» del hallazgo no se sostiene.
- **H13** tiene **una instancia más** que el hallazgo: el item «Un rojo del
  job `test` no se puede diagnosticar» (`docs/backlog.md:614`) está a medias
  cerrado por `50aee95` (2026-09-10), que ya emite nombre de test y bloque
  `failures:`.

Hallazgos nuevos al re-verificar, que entran en tareas existentes:
- `exo write append --help` documenta `forzado: true`; la clave real del
  envelope es `forced` (`engine/src/escritor.rs:105`). → Task 3.
- `skills/distill/SKILL.md:102` dice que `exo budget` sale con **exit 1** si
  hay offenders; sale con **3** (`engine/tests/budget_lint_cli.rs:78`). → Task 8.
- `exo search` sin resultados no imprime nada y sale 0 (medido en el ejemplo
  de extremo a extremo antes de `exo index`). Es comportamiento, **fuera de
  alcance**: se anota en el backlog en Task 13.

## Decisiones abiertas (PENDIENTE-PAUL)

Ninguna la toma la fábrica. La recomendación es eso, una recomendación.

### D1 — Idioma de los mensajes de usuario del CLI (H9) · bloquea Task 3 (Task 2 no)

Hoy: doc-comments y errores propios (`rechazado:`, `aviso:`, `error:`) en
español; el cromo de clap (`Usage:`, `Options:`, `error: the following
required arguments were not provided`) en inglés; flags largos y claves del
envelope en inglés (D8/D9); `README.md:93` declara «exo es un producto **en
español**».

| Opción | Qué implica | Pros | Contras |
|---|---|---|---|
| **A** Todo en inglés | Reescribir `about`/help en inglés y los ~20 `eprintln!` de `engine/src/` | Una sola lengua con flags y envelope; mercado mayor | Contradice `README.md:93`; rompe asserts de stderr (`engine/tests/budget_lint_cli.rs:85,130,214`, `ratchet_cli.rs:111` buscan `rechazado:`) y la señal que `distill` lee (`aviso: … truncado`); churn alto |
| **B** Todo en español | Localizar el cromo de clap: `Cli::try_parse()` + mapeo de `ErrorKind` y `help_template` | Coherente con `README.md:93` | Código propio contra la API de errores de clap que hay que mantener en cada upgrade; los flags siguen en inglés igualmente |
| **C** Híbrido declarado | Texto de producto y errores propios en español; cromo de clap en inglés; metavars = nombre del flag. Una línea en `docs/arquitectura.md` §3.8 | Coste casi cero; es el estado real, escrito | Sigue habiendo dos lenguas en la misma pantalla |

**Recomendación: C.** Task 2 (metavars) no depende de D1: el metavar sigue al
nombre del flag, que ya es inglés. Task 3 escribe los textos nuevos en
español; si Paul elige **A**, Task 3 traduce esos mismos 32 textos y los
`eprintln!` pasan a ser tarea nueva; si elige **B**, la localización de clap
es un plan aparte (necesita spec), no esta campaña.

### D2 — Para quién es exo / framing del README (H6; cruza BACKLOG:Baja «Decisión abierta: proceso frente a producto», `docs/backlog.md:810`) · bloquea Task 10

| Opción | Frase de «para quién» | Implica |
|---|---|---|
| **a** Herramienta personal publicada | «exo es el sistema de trabajo de su autor, publicado tal cual (MIT). Funciona y se prueba en Linux, macOS y Windows, pero lo decide un solo usuario: sin promesa de estabilidad ni soporte.» | README honesto con el estado real; no obliga a nada más |
| **b** Producto para terceros | «exo es para quien usa Claude Code a diario en proyectos largos y quiere que el agente recuerde entre sesiones. El CLI (`--json`, envelope v2, códigos de salida) es estable dentro de una versión mayor.» | Promesa de estabilidad; obliga a cerrar BACKLOG:Alta «"exo genérico" sigue siendo el plugin de Paul» (`Paul` en 4 ficheros del plugin, `kb-demo` en 3 hooks de producción) — **fuera de esta campaña** |

**Recomendación: a**, revisable. Con (b) Task 10 cambia una frase y el
backlog gana prioridad en el item de «exo genérico». También decide la
descripción del About de GitHub (checklist H25).

### D3 — Esquema de versionado (H16; BACKLOG:Media `docs/backlog.md:137`, acción b) · bloquea Task 9

Hoy: `.claude-plugin/marketplace.json:4` `metadata.version` **1.0.0**;
`marketplace.json:8` y `plugins/exo/.claude-plugin/plugin.json:4` **1.1.2**;
`engine/Cargo.toml:3` **0.1.0** = tag `v0.1.0`. Nada comprueba que concuerden.

| Opción | Qué | Pros | Contras |
|---|---|---|---|
| **a** Dos artefactos, dos versiones, un gate | Engine: `Cargo.toml` == tag. Plugin: `plugin.json` == `marketplace.json .plugins[].version`. Se **quita** `metadata.version` | Refleja que engine y plugin tienen ciclos distintos (el plugin lleva tres bumps desde 1.0.0, el engine una release) | Dos números que explicar (en `docs/instalacion.md`, no en el README); compatibilidad binario↔plugin sigue sin chequeo (BACKLOG: «check de desfase binario↔plugin» en `exo doctor`, fuera de alcance) |
| **b** Lockstep | Un único número para todo en la próxima release | Una sola historia | El engine salta de 0.1.0 a ≥1.1.3 (promesa implícita de estabilidad), o el plugin baja de número — **sin verificar** que Claude Code detecte una bajada como actualización (su caché va por versión: `~/.claude/plugins/cache/exo/exo/1.1.2/`) |
| **c** Status quo documentado | Una línea en `docs/instalacion.md` | Cero trabajo | El tercer número sigue sin dueño y sin gate |

**Recomendación: a.** El script de Task 9 está escrito para (a); para (b)
se sustituyen sus dos comparaciones por una sola de los tres números.

### D4 — Destino de `reports/` (H26; aplazado por Paul en `docs/superpowers/plans/2026-09-10-g5b-release-doctor-instaladores.md:104-106` por ruido de diff en release) · bloquea Task 11

Citas por ruta a ficheros concretos (medidas con `git grep -n "reports/" -- . ':!reports'`):
`engine/src/main.rs:14` (código vivo),
`docs/superpowers/consultas/2026-08-22-m6-06/consultor-m6-06.md:117`,
`evals/e1-read/verdict/gate-m2-07-impl.md:69`,
`evals/e1-read/verdict/gate-m2-07-spec.md:6,59`. Otras 9 menciones son a
`reports/` como directorio en narrativa fechada.

| Opción | Qué | Pros | Contras |
|---|---|---|---|
| **a** `git mv reports evals/e1-read/reports` y actualizar las 5 citas | Junto a `evals/e1-read/verdict/`, que gatea esas mismas campañas | Ninguna cita rota en HEAD | Edita dos verdicts **firmados** (solo la ruta, pero es un documento firmado) |
| **b** Mismo `git mv`, actualizar solo `main.rs:14` | Los documentos fechados conservan la ruta de su día (`git log --follow` la resuelve) | No reescribe historia | 4 citas apuntan a una ruta que ya no existe en HEAD |
| **c** Se queda en la raíz | Cerrar el item del backlog escribiendo por qué | Cero diff | El item lleva abierto desde G2 |

Nota: el argumento de G5b era el diff de **una release**; hoy no hay release
en curso y un `git mv` puro sale como 5 renames al 100 %. **Recomendación: b.**

### D5 — `tier` en los docs del repo (H18; BACKLOG:Media `docs/backlog.md:469`) · bloquea Task 12

Medido: 75 `.md` bajo `docs/`, 72 con fecha en el nombre, **ninguno** con
frontmatter (el `tier:` que el backlog atribuía a `arquitectura.md:419` está
dentro de un bloque de código de ejemplo).

| Opción | Qué | Pros | Contras |
|---|---|---|---|
| **a** Acción literal del backlog | Frontmatter `tier: log` en los 72 fechados y `tier: core` en `README.md` + `docs/{arquitectura,instalacion,backlog}.md` | exo se aplica su propio modelo | 76 ficheros tocados; GitHub pinta el frontmatter como tabla encima del README; **ningún consumidor** lee ese `tier` hoy |
| **b** Convención por ruta | Una línea en `docs/arquitectura.md`: «vivo = README + los 3 de `docs/` sin fecha; todo lo fechado y `docs/superpowers/` es log». Las cifras frágiles salen de los vivos (Task 10) | Cero churn; ataca la causa medida (los dos casos de deriva estaban en los 4 vivos) | No es greppable por frontmatter |
| **c** Frontmatter solo en `docs/` | (a) sin tocar `README.md` | Evita la tabla en la portada | Mismo coste sin consumidor |

**Recomendación: b** hasta que exista un consumidor (p.ej. el grep de
afirmaciones frágiles que el backlog propone, que no entra en esta campaña).

## Global Constraints

Todas verificadas en `3c1918f`. **Los números de línea son de ese commit**:
tras mergear A se mueven — ancla siempre por contenido, nunca por número.

- **El crate vive en `engine/`, no en la raíz.** No hay workspace. Todo
  `cargo` con cwd `engine/`.
- **MSRV `rust-version = "1.95"`** (`engine/Cargo.toml:8`), comprobada por el
  job `msrv` con `cargo check --all-targets --locked` (`ci.yml:61-76`).
- **Gate de lint:** `cargo fmt --check` y
  `cargo clippy --all-targets --locked -- -D warnings` (`ci.yml:36-46`).
- **`engine/scripts/test-hermetico.sh` NO se modifica.** Es gate demostrado
  falsable (2026-08-27) que consumen `ci.yml:110-112` y `release.yml`. Todo
  test nuevo pasa con `EXO_CONFIG` apuntando a un fichero inexistente.
- **Envelope v2, verbatim de `engine/src/envelope.rs:9-16`:** «Emite
  `{"schema_version":2,"command":<command>,"data":<data>}` como una única
  línea JSON, newline-terminada, a **stdout**. […] Lo que SIEMPRE va a stderr,
  con o sin `--json`, son los avisos y el progreso […]. Los consumidores
  gatean por exit code, jamás por campos de `data`.» Esta campaña no toca
  ninguna clave de `data`.
- **Códigos de salida (`engine/src/main.rs:386-412`):** `3` si el error es un
  `exo::escritor::Rechazo` o un `exo::gate::GateFallido` (con
  `eprintln!("rechazado: …")`); `1` cualquier otro (`eprintln!("error: …")`);
  `0` éxito. Un error de uso de clap sale con **2** antes de ejecutar nada —
  `engine/tests/flags.rs:7-9` depende de eso.
- **Flags largos en inglés con alias español oculto** (`--limite`,
  `--titulo`, `--crea`, `--min-similitud`, `--escala-fts`, `--contenido`,
  `--nota`, `--refresca`): los fijan `engine/tests/flags.rs`. Esta campaña no
  añade, quita ni renombra ningún flag ni alias.
- **Tests de CLI:** `std::process::Command` con `env!("CARGO_BIN_EXE_exo")`,
  `tempfile`, `serde_json`. `[dev-dependencies]` = solo `tempfile`; no se
  añaden `assert_cmd` ni `predicates`.
- **«Sin cambio de comportamiento del motor»** significa: mismo stdout,
  mismo envelope, mismos exit codes y mismos hallazgos para la misma entrada.
  Cambian solo textos de `--help`. La única tarea que cambia comportamiento
  **de un hook** (no del motor) es Task 5 (H21), y lo declara.
- **Tests del plugin:** `scripts/test-plugin.sh` descubre por glob
  `plugins/exo/scripts/test-*.sh` (excluye `test-contrato-engine.sh`) y los
  invoca **directamente** (`"./$t"`), así que un test nuevo necesita 100755.
  Corren en ubuntu, windows y macos (`ci.yml:123-136`): todo bash nuevo bajo
  `plugins/` es portable a macOS y Git Bash (sin `timeout`, `date -d`,
  `stat -c`, `touch -d` a pelo — `ec74f43`).
- **El gate de exec-bit lee el índice de git**, no el working tree
  (`scripts/test-exec-bit.sh:10-11`): un fichero nuevo se añade con
  `git add` y, si hace falta, `git update-index --chmod=+x <ruta>`.
- **`.gitattributes`: `* text=auto eol=lf`.** Nada de normalizar finales de
  línea a mano.
- **`$TMPDIR`** en los comandos de este plan es un directorio temporal fuera
  del repo: `export TMPDIR="${TMPDIR:-$(mktemp -d)}"` al abrir la sesión del
  ejecutor. Nada de lo que va ahí se commitea.
- **Git:** `git add <rutas explícitas>`, nunca `-A`/`.`; `git -C <path>`,
  nunca `cd <path> && git`. Nada de push ni de tocar GitHub (línea roja del
  config de fábrica).
- **ShellCheck pineado:** v0.11.0,
  `https://github.com/koalaman/shellcheck/releases/download/v0.11.0/shellcheck-v0.11.0.linux.x86_64.tar.xz`,
  SHA256 `8c3be12b05d5c177a04c29e3c78ce89ac86f1595681cab149b65b97c4e227198`
  (medido sobre el asset el 2026-09-13). Localmente no hay `shellcheck`
  instalado: se usa ese mismo tarball desempaquetado en `$TMPDIR`.
- **Fuera de alcance, declarado:** backup y seguridad; todo lo de la campaña
  A (schema con `ruta` por KB, `tier` persistido, `--db` en `exo init`,
  avisos/`elapsed_s` en `exo recall`, transacción en `resuelve_destinos`,
  rotación de `reflex-log.jsonl`, caché de `exo config` en los hooks, bench
  de 5.000 notas, dedupe de `separa_frontmatter`) y de la C (eval held-out);
  renombrar `docs/superpowers/`; generalizar `Paul`/`kb-demo` en el plugin
  (BACKLOG:Alta); el check de desfase binario↔plugin.

## Dependencias con A y C, y conflictos de fichero

**B se ejecuta después de A.** Precondición de arranque (orquestador, antes de
despachar nada):

```bash
git -C /home/paul/Documentos/proyectos/exo log --oneline -15
exo_bin=/home/paul/Documentos/proyectos/exo/engine/target/release/exo
cargo build --release --manifest-path /home/paul/Documentos/proyectos/exo/engine/Cargo.toml
"$exo_bin" init --help | grep -- '--db'        # A mergeada ⇒ sale la línea de --db
```

Si A no está mergeada, **solo** pueden correr Task 1, 4, 5, 6 y 8 (no tocan
ficheros de A). El resto espera.

| Fichero | A | C | B (tarea) | Consecuencia |
|---|---|---|---|---|
| `engine/src/main.rs` (`ArgsInit`, doc-comments) | añade `--db` a `exo init` | puede tocar las constantes `BONUS_SELLADO`/`ESCALA_FTS_SELLADA` (`main.rs:12-25`) si cambia la fusión | Task 2, 3, 11 | B después de A. El test de Task 2 exige descripción en el `--db` nuevo; el de Task 3 caza jerga en lo que A haya escrito. Si C cambia la fórmula de fusión, el texto de `--bonus`/`--fts-scale` de Task 3 queda obsoleto: revisarlo en la review de C |
| `plugins/exo/scripts/{recall-inject,exo-recall,compose-inject,_reflex-log}.sh` | los modifica | `recall-inject.sh:145` citado | Task 7 (shellcheck) | Task 7 re-mide sobre el árbol post-A; las directivas se anclan por contenido |
| `docs/backlog.md` | cierra H1-H5 | anota H7/H14/H24 | Task 13 | Task 13 va la última, sobre el backlog ya tocado por A (y por C si mergeó) |
| `docs/instalacion.md`, `docs/arquitectura.md:285` | deberían documentar `--db` | — | Task 10 | Task 10 verifica que el `--db` de A está documentado; si no, lo añade |
| `.github/workflows/ci.yml` | — | posible job de eval | Task 6, 7, 9 | Tres tareas de B sobre el mismo job/fichero, secuenciales |
| `evals/` | — | añade `evals/retrieval-heldout/` | Task 7 excluye `evals/` del shellcheck | Sin conflicto |
| `plugins/exo/scripts/recall-latencia.sh`, `evals/recall-coste/harness/*.sh`, `engine/examples/kb_sintetica.rs` (plan `2026-09-13-campana-a-preregistro-bench.md`) | los crea | — | Task 6, 7 | `recall-latencia.sh` entra solo en exec-bit y shellcheck (descubren por índice): sus avisos se tratan en Task 7. `evals/recall-coste/` queda fuera de shellcheck por la regla de `evals/`. `engine/examples/` lo cubre clippy `--all-targets` |

**Sinergias del brief que resultaron falsas o parciales:**
- **H22 no pertenece al bloque «refactor de scripts antes del gate estático»**:
  `distill/SKILL.md` es markdown; ni shellcheck ni exec-bit lo miran. Va en
  la rama thin por afinidad, sin orden forzado.
- **H6 + H16 no «cuentan la misma historia» como acoplamiento**: el README no
  debe llevar números de versión (son justo las afirmaciones frágiles que
  derivan), así que Task 10 no espera a D3. Solo comparten la decisión de
  framing de D2 para el About.
- **«Una pasada de higiene documental» H13/H18/H26** es parcial: H18 y H26
  son dos decisiones independientes; H13 no es par de ellas sino el cierre que
  registra todo lo demás, y va al final.
- **H8 + H9 sí es real** y además se parte limpio: los metavars no dependen de
  D1 (Task 2), los textos sí (Task 3).
- **H20/H21 antes de H11/H12 sí es real**: los dos crean o modifican scripts
  que los gates juzgan (`_truncate-payload.sh` disparó SC2034 en la prueba).

## Ramas sugeridas

| Rama | Tareas | Lane |
|---|---|---|
| `b-engine-superficie` | 1 (H15), 2 (H9), 3 (H8) | mecánica (oráculos escritos en el plan) |
| `b-thin-gates` | 4 (H20), 5 (H21), 6 (H12), 7 (H11), 8 (H22) | mecánica salvo 5 y 8 (diseño) |
| `b-docs` | 9 (H16), 10 (H6), 11 (H26), 12 (H18), 13 (H13) | 10 diseño; resto mecánica tras decisión |

Routing de lanes según el config: «mecánica» = el oráculo existe como comando
literal en este plan; «diseño» = hay criterio de alcance que adjudicar con
cita.

---

### Task 1: `trinquete::comprueba_contra` partida en tres familias con nombre (H15)

**Lane:** mecánica. **Depende de A:** no. **Oráculo:**
`cd engine && cargo test --release --lib trinquete && cargo test --release --test trinquete_abstencion --test trinquete_aire --test trinquete_declaraciones --test trinquete_recolecta --test trinquete_seal --test trinquete_sellos --test trinquete_staged --test ratchet_cli && ./scripts/test-hermetico.sh`
— verde **sin tocar ningún test existente** (`git diff --stat -- engine/tests` vacío).

**Evidencia (H15):** `engine/src/trinquete.rs:581-793`, 213 líneas, tres
familias de checks con estado mutable compartido: guarda de aire por
transición (`:618-695`, ramas 1-4), cap 2× + waivers (`:697-762`), sellos
escapados de tier (`:764-785`); comparten `nacio_demasiado_grande` y
`rutas_declaradas`. Los 54 tests de integración `trinquete_*` + 6 de
`ratchet_cli` solo ejercitan el conjunto a través de `comprueba`/
`comprueba_staged`, que necesitan un repo git.

**Files:**
- Modify: `engine/src/trinquete.rs` (sustituir `:611-793` — desde
  `    // El ancla de activación (Task 4)` hasta la `}` que cierra
  `comprueba_contra` — y añadir tests al final de `mod tests`, antes de su
  `}` de cierre en `:1048`)

**Interfaces:**
- Consumes: `Sellos = BTreeMap<String, i64>` (`:28`), `Declarada { ruta, tier, max, tier_presupuesto, tamano }` (`:291-297`),
  `Hallazgo { ruta, tipo, era, ahora, limite }` (`:232-243`), `Tipo` (`:177-196`),
  `FACTOR_PRIMERA_DECLARACION: i64 = 2` (`:501`),
  `crate::presupuesto::{tiene_aire, techo_minimo, Presupuestos, NOMINALES}`
  (`Presupuestos` es `Copy`; `NOMINALES` = core 8500 / stable 12500 / log 0),
  helpers de test `sellos(&[(&str, i64)])` y `conjunto(&[&str])` (`:890-896`).
- Produces (privadas del módulo):
  - `fn guarda_de_aire(head: &Sellos, actual: &Sellos, declaradas: &[Declarada], activacion: bool, frescos_absueltos: &BTreeSet<String>, tamano_de: impl Fn(&str) -> Option<i64>) -> (Vec<Hallazgo>, BTreeSet<String>)`
  - `fn checks_de_declaracion(head: &Sellos, actual: &Sellos, declaradas: &[Declarada], activacion: bool, frescos_absueltos: &BTreeSet<String>, nacio_demasiado_grande: &BTreeSet<String>) -> Vec<Hallazgo>`
  - `fn sellos_escapados_de_tier(kb: &Path, actual: &Sellos, declaradas: &[Declarada], presupuestos: crate::presupuesto::Presupuestos) -> Vec<Hallazgo>`
  - `comprueba_contra` conserva firma y contrato.

- [ ] **Step 1: Escribir los tests que fallan**

Añadir dentro de `mod tests` de `engine/src/trinquete.rs`, justo antes de la
`}` final del módulo:

```rust
    // H15 — cada familia de `comprueba_contra`, por separado y sin git.

    fn declarada(ruta: &str, max: i64, tier_presupuesto: i64, tamano: i64) -> Declarada {
        Declarada {
            ruta: ruta.to_string(),
            tier: "core".to_string(),
            max,
            tier_presupuesto,
            tamano,
        }
    }

    fn sin_tamano(_: &str) -> Option<i64> {
        None
    }

    #[test]
    fn aire_un_sello_intacto_sin_aire_es_deuda_no_fallo() {
        // 100 B bajo techo 100: 10.000 < 11.500, sin aire. Intacto desde HEAD.
        let head = sellos(&[("a.md", 100)]);
        let actual = sellos(&[("a.md", 100)]);
        let decl = [declarada("a.md", 100, 8500, 100)];
        let (h, grandes) = guarda_de_aire(&head, &actual, &decl, false, &conjunto(&[]), sin_tamano);
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].tipo, Tipo::DeudaSinAire);
        assert_eq!(h[0].ahora, 100);
        assert_eq!(h[0].limite, 115, "techo_minimo(100) = ceil(115)");
        assert!(grandes.is_empty());
    }

    #[test]
    fn aire_un_sello_fresco_sin_aire_es_sin_aire() {
        let head = sellos(&[]);
        let actual = sellos(&[("a.md", 100)]);
        let decl = [declarada("a.md", 100, 8500, 100)];
        let (h, _) = guarda_de_aire(&head, &actual, &decl, false, &conjunto(&[]), sin_tamano);
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].tipo, Tipo::SinAire);
        assert_eq!(h[0].limite, 115);
    }

    #[test]
    fn aire_la_activacion_consagra_el_sello_fresco() {
        let head = sellos(&[]);
        let actual = sellos(&[("a.md", 100)]);
        let decl = [declarada("a.md", 100, 8500, 100)];
        let (h, _) = guarda_de_aire(&head, &actual, &decl, true, &conjunto(&[]), sin_tamano);
        assert!(h.is_empty(), "activación: {h:?}");
    }

    #[test]
    fn aire_una_nota_en_zona_muerta_nace_demasiado_grande_y_se_marca() {
        // 18.000 B: techo_minimo = 20.700 > 2 × 8.500.
        let head = sellos(&[]);
        let actual = sellos(&[("a.md", 20000)]);
        let decl = [declarada("a.md", 20000, 8500, 18000)];
        let (h, grandes) = guarda_de_aire(&head, &actual, &decl, false, &conjunto(&[]), sin_tamano);
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].tipo, Tipo::NaceDemasiadoGrande);
        assert_eq!(h[0].ahora, 18000);
        assert_eq!(h[0].limite, 17000);
        assert_eq!(grandes, conjunto(&["a.md"]));
    }

    #[test]
    fn aire_una_subida_no_se_etiqueta_ademas_como_deuda() {
        let head = sellos(&[("a.md", 100)]);
        let actual = sellos(&[("a.md", 200)]);
        let decl = [declarada("a.md", 200, 8500, 190)];
        let (h, _) = guarda_de_aire(&head, &actual, &decl, false, &conjunto(&[]), sin_tamano);
        assert!(h.is_empty(), "la subida ya es SelloSubido: {h:?}");
    }

    #[test]
    fn aire_sin_declaracion_ni_tamano_legible_se_salta() {
        let head = sellos(&[]);
        let actual = sellos(&[("huerfano.md", 100)]);
        let (h, _) = guarda_de_aire(&head, &actual, &[], false, &conjunto(&[]), sin_tamano);
        assert!(h.is_empty());
    }

    #[test]
    fn aire_sin_declaracion_usa_el_tamano_de_la_revision() {
        let head = sellos(&[]);
        let actual = sellos(&[("a.md", 100)]);
        let (h, _) = guarda_de_aire(&head, &actual, &[], false, &conjunto(&[]), |_| Some(100));
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].tipo, Tipo::SinAire);
    }

    #[test]
    fn declaracion_un_waiver_por_encima_del_sello_es_sobre_sello() {
        let head = sellos(&[("a.md", 100)]);
        let actual = sellos(&[("a.md", 100)]);
        let decl = [declarada("a.md", 150, 8500, 50)];
        let h = checks_de_declaracion(&head, &actual, &decl, false, &conjunto(&[]), &conjunto(&[]));
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].tipo, Tipo::SobreSello);
        assert_eq!((h[0].era, h[0].ahora, h[0].limite), (100, 150, 100));
    }

    #[test]
    fn declaracion_un_sello_fresco_por_encima_de_2x_es_primera_muy_alta() {
        let head = sellos(&[]);
        let actual = sellos(&[("a.md", 20000)]);
        let decl = [declarada("a.md", 20000, 8500, 100)];
        let h = checks_de_declaracion(&head, &actual, &decl, false, &conjunto(&[]), &conjunto(&[]));
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].tipo, Tipo::PrimeraMuyAlta);
        assert_eq!((h[0].ahora, h[0].limite), (20000, 17000));
    }

    #[test]
    fn declaracion_sin_sello_por_encima_de_2x_es_primera_muy_alta() {
        let decl = [declarada("a.md", 20000, 8500, 100)];
        let h = checks_de_declaracion(
            &sellos(&[]),
            &sellos(&[]),
            &decl,
            false,
            &conjunto(&[]),
            &conjunto(&[]),
        );
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].tipo, Tipo::PrimeraMuyAlta);
        assert_eq!(h[0].ahora, 20000);
    }

    #[test]
    fn declaracion_un_waiver_en_tier_sin_presupuesto_es_inerte() {
        let decl = [declarada("a.md", 500, 0, 100)];
        let h = checks_de_declaracion(
            &sellos(&[]),
            &sellos(&[]),
            &decl,
            false,
            &conjunto(&[]),
            &conjunto(&[]),
        );
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].tipo, Tipo::WaiverLogInerte);
        assert!(!h[0].tipo.rompe());
    }

    #[test]
    fn declaracion_no_apila_hallazgos_sobre_una_nota_demasiado_grande() {
        let actual = sellos(&[("a.md", 20000)]);
        let decl = [declarada("a.md", 20000, 8500, 18000)];
        let h = checks_de_declaracion(
            &sellos(&[]),
            &actual,
            &decl,
            false,
            &conjunto(&[]),
            &conjunto(&["a.md"]),
        );
        assert!(h.is_empty(), "{h:?}");
    }

    #[test]
    fn escapados_un_sello_sin_waiver_en_tier_log_escapo_del_gate() {
        let kb = tempfile::tempdir().unwrap();
        std::fs::write(kb.path().join("a.md"), "---\ntier: log\n---\nx\n").unwrap();
        std::fs::write(kb.path().join("b.md"), "---\ntier: core\n---\nx\n").unwrap();
        let actual = sellos(&[("a.md", 9000), ("b.md", 9000), ("borrada.md", 9000)]);
        let h = sellos_escapados_de_tier(kb.path(), &actual, &[], crate::presupuesto::NOMINALES);
        assert_eq!(
            h.len(),
            1,
            "solo a.md: b.md tiene presupuesto, borrada.md no se lee"
        );
        assert_eq!(h[0].ruta, "a.md");
        assert_eq!(h[0].tipo, Tipo::SelladaEscapadaDeTier);
        assert_eq!(h[0].era, 9000);
    }

    #[test]
    fn escapados_ignora_los_sellos_con_declaracion() {
        let kb = tempfile::tempdir().unwrap();
        std::fs::write(kb.path().join("a.md"), "---\ntier: log\n---\nx\n").unwrap();
        let actual = sellos(&[("a.md", 9000)]);
        let decl = [declarada("a.md", 9000, 0, 10)];
        let h = sellos_escapados_de_tier(kb.path(), &actual, &decl, crate::presupuesto::NOMINALES);
        assert!(h.is_empty());
    }
```

- [ ] **Step 2: Verlos fallar**

Run: `cd engine && cargo test --release --lib trinquete`
Expected: FAIL de compilación — `error[E0425]: cannot find function
`guarda_de_aire`` (×7), `checks_de_declaracion` (×5),
`sellos_escapados_de_tier` (×2): 14 errores.

- [ ] **Step 3: Implementación mínima**

Sustituir en `engine/src/trinquete.rs` el tramo que va desde la línea
`    // El ancla de activación (Task 4): sin sello commiteado en HEAD, esta`
hasta la `}` que cierra `comprueba_contra` (inclusive) por:

```rust
    // El ancla de activación (Task 4): sin sello commiteado en HEAD, esta
    // corrida es la que instala el trinquete y consagra lo que ya existía.
    let activacion = !anclado_en_head(kb);

    // Las tres familias, en el mismo orden en que se empujaban antes de
    // partir la función: `sort_by` es estable, así que dos hallazgos de la
    // misma ruta conservan ese orden relativo en el informe.
    let (aire, nacio_demasiado_grande) = guarda_de_aire(
        &head,
        &actual,
        declaradas,
        activacion,
        &frescos_absueltos,
        |ruta| tamano_de(kb, ruta),
    );
    hallazgos.extend(aire);
    hallazgos.extend(checks_de_declaracion(
        &head,
        &actual,
        declaradas,
        activacion,
        &frescos_absueltos,
        &nacio_demasiado_grande,
    ));
    hallazgos.extend(sellos_escapados_de_tier(
        kb,
        &actual,
        declaradas,
        presupuestos,
    ));

    hallazgos.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    Ok(Informe {
        aplicado: true,
        razon: None,
        hallazgos,
    })
}

/// La guarda de aire (Task 8): juzga TRANSICIONES, no estado. Un sello que
/// nadie toca no se juzga aunque no tenga aire — se reporta como deuda, que
/// no rompe. Sin eso, los 11 sellos reales de la KB (ninguno con 15% de
/// aire, medido) dejarían el repo en rojo permanente el día de la
/// instalación.
///
/// Devuelve los hallazgos y el conjunto de rutas marcadas
/// `NaceDemasiadoGrande`, que `checks_de_declaracion` necesita para no
/// apilarles más hallazgos.
fn guarda_de_aire(
    head: &Sellos,
    actual: &Sellos,
    declaradas: &[Declarada],
    activacion: bool,
    frescos_absueltos: &BTreeSet<String>,
    tamano_de: impl Fn(&str) -> Option<i64>,
) -> (Vec<Hallazgo>, BTreeSet<String>) {
    let declaradas_por_ruta: BTreeMap<&str, &Declarada> =
        declaradas.iter().map(|d| (d.ruta.as_str(), d)).collect();

    let mut hallazgos = Vec::new();
    let mut nacio_demasiado_grande: BTreeSet<String> = BTreeSet::new();
    for (ruta, &techo) in actual {
        let declarada = declaradas_por_ruta.get(ruta.as_str()).copied();
        let tamano = match declarada {
            Some(d) => d.tamano,
            // Sin declaración: el tamaño sale de la misma revisión que los
            // sellos. Si no se puede leer (nota borrada, sello huérfano sin
            // fichero), se salta — no se inventa un tamaño.
            None => match tamano_de(ruta) {
                Some(t) => t,
                None => continue,
            },
        };
        if crate::presupuesto::tiene_aire(techo, tamano) {
            continue; // rama 1: tiene aire, nada que reportar.
        }
        let era = head.get(ruta).copied();
        if let Some(era) = era
            && techo > era
        {
            // rama 2: ya rompió como SelloSubido; etiquetarlo además como
            // deuda mal-clasificaría una decisión de hoy como preexistente.
            continue;
        }
        let cambio = match era {
            None => true,
            Some(era) => techo < era,
        };
        if !cambio {
            // rama 3: intacto desde HEAD. Deuda, no fallo.
            hallazgos.push(Hallazgo {
                ruta: ruta.clone(),
                tipo: Tipo::DeudaSinAire,
                era: 0,
                ahora: techo,
                limite: crate::presupuesto::techo_minimo(tamano),
            });
            continue;
        }
        // rama 4: cambió (fresco o bajado). La activación consagra lo que
        // ya existía; un rename carga su techo — ninguno es una decisión
        // nueva sobre el margen.
        if activacion || frescos_absueltos.contains(ruta) {
            continue;
        }
        let fresco = era.is_none();
        if fresco
            && let Some(d) = declarada
            && d.tier_presupuesto > 0
            && crate::presupuesto::techo_minimo(tamano)
                > d.tier_presupuesto * FACTOR_PRIMERA_DECLARACION
        {
            // Zona muerta: ninguna nota de este tamaño puede tener a la vez
            // aire y respetar el cap de 2×. El remedio es partir la nota, no
            // un techo más alto.
            hallazgos.push(Hallazgo {
                ruta: ruta.clone(),
                tipo: Tipo::NaceDemasiadoGrande,
                era: 0,
                ahora: tamano,
                limite: d.tier_presupuesto * FACTOR_PRIMERA_DECLARACION,
            });
            nacio_demasiado_grande.insert(ruta.clone());
            continue;
        }
        hallazgos.push(Hallazgo {
            ruta: ruta.clone(),
            tipo: Tipo::SinAire,
            era: 0,
            ahora: techo,
            limite: crate::presupuesto::techo_minimo(tamano),
        });
    }
    (hallazgos, nacio_demasiado_grande)
}

/// Las tres familias sobre declaraciones: cap de 2× en la primera
/// declaración (sellada o no), waiver por encima del sello y waiver inerte
/// en un tier sin presupuesto. Una nota ya marcada `NaceDemasiadoGrande` no
/// recibe además estos checks: ya tiene el único hallazgo que aconseja bien,
/// y el Go la salta con el mismo set.
fn checks_de_declaracion(
    head: &Sellos,
    actual: &Sellos,
    declaradas: &[Declarada],
    activacion: bool,
    frescos_absueltos: &BTreeSet<String>,
    nacio_demasiado_grande: &BTreeSet<String>,
) -> Vec<Hallazgo> {
    let mut hallazgos = Vec::new();
    for d in declaradas {
        if nacio_demasiado_grande.contains(&d.ruta) {
            continue;
        }
        match actual.get(&d.ruta) {
            Some(&sello) => {
                // El cap de 2× sobre una nota ya sellada: no aplica en la
                // corrida de activación, si ya estaba en HEAD, o si es un
                // fresco absuelto por rename (carga su techo, no lo declara).
                if !activacion
                    && !head.contains_key(&d.ruta)
                    && !frescos_absueltos.contains(&d.ruta)
                    && d.tier_presupuesto > 0
                {
                    let limite = d.tier_presupuesto * FACTOR_PRIMERA_DECLARACION;
                    if sello > limite {
                        hallazgos.push(Hallazgo {
                            ruta: d.ruta.clone(),
                            tipo: Tipo::PrimeraMuyAlta,
                            era: 0,
                            ahora: sello,
                            limite,
                        });
                    }
                }
                // El waiver no puede rebasar el trinquete.
                if d.max > sello {
                    hallazgos.push(Hallazgo {
                        ruta: d.ruta.clone(),
                        tipo: Tipo::SobreSello,
                        era: sello,
                        ahora: d.max,
                        limite: sello,
                    });
                }
            }
            None if d.tier_presupuesto <= 0 => {
                // Waiver en un tier sin presupuesto (p.ej. log): inerte, no
                // hay techo que rebasar. Información, no fallo.
                hallazgos.push(Hallazgo {
                    ruta: d.ruta.clone(),
                    tipo: Tipo::WaiverLogInerte,
                    era: 0,
                    ahora: d.max,
                    limite: 0,
                });
            }
            None => {
                let limite = d.tier_presupuesto * FACTOR_PRIMERA_DECLARACION;
                if d.max > limite {
                    hallazgos.push(Hallazgo {
                        ruta: d.ruta.clone(),
                        tipo: Tipo::PrimeraMuyAlta,
                        era: 0,
                        ahora: d.max,
                        limite,
                    });
                }
            }
        }
    }
    hallazgos
}

/// Una nota sellada que ya no declara waiver y cuyo tier ACTUAL no tiene
/// presupuesto se reclasificó a `log` para escapar del gate: el sello es la
/// prueba de que tuvo techo. Solo mira los sellos SIN `Declarada` — con
/// `Declarada` ya pasó por `checks_de_declaracion`.
///
/// Lee el tier del **disco** también en `--staged`: es el comportamiento
/// heredado de antes de partir `comprueba_contra`, y este refactor no lo
/// cambia.
fn sellos_escapados_de_tier(
    kb: &Path,
    actual: &Sellos,
    declaradas: &[Declarada],
    presupuestos: crate::presupuesto::Presupuestos,
) -> Vec<Hallazgo> {
    let rutas_declaradas: BTreeSet<&str> = declaradas.iter().map(|d| d.ruta.as_str()).collect();
    let mut hallazgos = Vec::new();
    for (ruta, &sello) in actual {
        if rutas_declaradas.contains(ruta.as_str()) {
            continue;
        }
        let Ok(contenido) = std::fs::read_to_string(kb.join(ruta)) else {
            continue; // nota borrada: el sello huérfano se queda, nada que mirar.
        };
        let tier = crate::frontmatter::tier(&contenido);
        if !tier.is_empty() && presupuestos.para_tier(&tier).unwrap_or(0) <= 0 {
            hallazgos.push(Hallazgo {
                ruta: ruta.clone(),
                tipo: Tipo::SelladaEscapadaDeTier,
                era: sello,
                ahora: 0,
                limite: 0,
            });
        }
    }
    hallazgos
}
```

Notas para el ejecutor: `sort_by` es estable, y las tres familias se
concatenan en el mismo orden en que antes se empujaban, así que el orden del
informe no cambia ni para hallazgos de la misma ruta. `rutas_declaradas` era
el conjunto de todas las `Declarada` (se insertaba antes del `continue` de
`nacio_demasiado_grande`), por eso `sellos_escapados_de_tier` lo recalcula
entero. `sellos_escapados_de_tier` lee el tier **del disco** también en
`--staged`: comportamiento heredado, no se corrige aquí (se anota en Task 13).

- [ ] **Step 4: Verlos pasar, y la suite sin tocar**

Run: `cd engine && cargo fmt && cargo test --release --lib trinquete`
Expected: `test result: ok. 23 passed; 0 failed` (9 previos + 14 nuevos).

Run: `cd engine && cargo test --release --test trinquete_abstencion --test trinquete_aire --test trinquete_declaraciones --test trinquete_recolecta --test trinquete_seal --test trinquete_sellos --test trinquete_staged --test ratchet_cli`
Expected: 8 binarios en `ok` (6 + 11 + 7 + 4 + 9 + 7 + 10 + 6 tests).

Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && ./scripts/test-hermetico.sh`
Expected: sin salida de fmt/clippy; `test-hermetico: OK — la suite corre sin ~/.exo/config.toml; …`

Run: `git -C /home/paul/Documentos/proyectos/exo diff --stat -- engine/tests`
Expected: vacío.

(Todo lo anterior se corrió el 2026-09-13 sobre una copia de `3c1918f` con
este código exacto: 23/23, los 8 binarios verdes, clippy limpio, gate
hermético OK.)

- [ ] **Step 5: Commit**

```bash
git add engine/src/trinquete.rs
git commit -m "refactor(trinquete): comprueba_contra en tres familias con nombre y tests por familia (H15)"
```

---

### Task 2: metavars iguales al flag y ninguna opción sin descripción (H9, parte independiente de D1)

**Lane:** mecánica. **Depende de A:** sí (A añade `--db` a `ArgsInit`; este
test lo juzga). **Oráculo:**
`cd engine && cargo test --release --test help_producto --test flags`.

**Evidencia (H9, H8):** `exo search --help` muestra `--limit <LIMITE>`,
`--min-similarity <MIN_SIMILITUD>`, `--fts-scale <ESCALA_FTS>`;
`exo write new --help` `--title <TITULO>`; `exo recall --help`
`--note <NOTA>`, `--limit <LIMITE>`, `--min-similarity <MIN_SIMILITUD>`;
`exo targets --help` `--limit <LIMITE>`. Sin descripción: `--json` de `init`
(`main.rs:111-112`), `config` (`:117-118`), `write new` (`:150-151`),
`write append` (`:174-175`). Medido con `engine/target/release/exo … --help`.

**Files:**
- Create: `engine/tests/help_producto.rs`
- Modify: `engine/src/main.rs` (atributos `#[arg]` de `--limit`,
  `--min-similarity`, `--fts-scale`, `--title`, `--note`; doc-comment en los
  cuatro `json: bool` sin él)

**Interfaces:**
- Consumes: el binario `exo` vía `env!("CARGO_BIN_EXE_exo")`; `regex::Regex`.
- Produces: `engine/tests/help_producto.rs` con `fn ayuda(args: &[&str]) -> String`,
  `const PANTALLAS: &[&[&str]]`, `fn seccion<'a>(texto: &'a str, cabecera: &str) -> Vec<&'a str>`,
  `fn subcomandos(texto: &str) -> Vec<String>` — Task 3 añade un test que los reutiliza.

- [ ] **Step 1: Escribir el test que falla**

`engine/tests/help_producto.rs`:

```rust
//! La ayuda de `exo` (`--help`) es superficie de producto: la lee alguien que
//! acaba de instalar el binario, no quien lo escribió.
//!
//! Tres invariantes, cada uno falsable contra el binario real:
//! 1. toda opción lleva descripción (un `--json` en blanco no dice nada);
//! 2. el metavar de un flag es el nombre del flag en MAYÚSCULAS (`--limit
//!    <LIMIT>`): los flags largos son ingleses desde la 1.0 y un `<LIMITE>` heredado del
//!    nombre del campo mezcla idiomas en la misma línea;
//! 3. ninguna pantalla lleva jerga interna de campaña (hitos, specs, scripts
//!    del autor, símbolos de Rust) (`la_ayuda_no_lleva_jerga_interna`).
//!
//! Hermético: `EXO_CONFIG` apunta a un fichero inexistente; clap resuelve
//! `--help` antes de leer ninguna config.
use std::process::Command;

fn ayuda(args: &[&str]) -> String {
    let salida = Command::new(env!("CARGO_BIN_EXE_exo"))
        .args(args)
        .arg("--help")
        .env("EXO_CONFIG", "C:/no-existe-jamas/config.toml")
        .output()
        .expect("correr el binario");
    assert!(
        salida.status.success(),
        "{args:?} --help salió con {:?}",
        salida.status
    );
    String::from_utf8(salida.stdout).expect("help en UTF-8")
}

/// Todas las pantallas de ayuda del binario. Si se añade un subcomando y no se
/// añade aquí, `la_lista_cubre_todos_los_subcomandos` falla.
const PANTALLAS: &[&[&str]] = &[
    &[],
    &["init"],
    &["config"],
    &["index"],
    &["rebuild"],
    &["search"],
    &["write"],
    &["write", "new"],
    &["write", "append"],
    &["recall"],
    &["targets"],
    &["budget"],
    &["lint"],
    &["ratchet"],
    &["doctor"],
];

/// Líneas de una sección (`Commands:`, `Options:`, `Arguments:`) hasta la
/// siguiente línea en blanco.
fn seccion<'a>(texto: &'a str, cabecera: &str) -> Vec<&'a str> {
    texto
        .lines()
        .skip_while(|l| *l != cabecera)
        .skip(1)
        .take_while(|l| !l.trim().is_empty())
        .collect()
}

fn subcomandos(texto: &str) -> Vec<String> {
    seccion(texto, "Commands:")
        .iter()
        .filter_map(|l| l.split_whitespace().next())
        .filter(|n| *n != "help")
        .map(str::to_string)
        .collect()
}

#[test]
fn la_lista_cubre_todos_los_subcomandos() {
    let mut vistas: Vec<Vec<String>> = subcomandos(&ayuda(&[]))
        .into_iter()
        .map(|c| vec![c])
        .collect();
    vistas.extend(
        subcomandos(&ayuda(&["write"]))
            .into_iter()
            .map(|c| vec!["write".to_string(), c]),
    );
    for v in vistas {
        assert!(
            PANTALLAS
                .iter()
                .any(|p| p.iter().copied().eq(v.iter().map(String::as_str))),
            "el subcomando {v:?} no está en PANTALLAS"
        );
    }
}

#[test]
fn toda_opcion_lleva_descripcion() {
    // clap pinta la descripción en la misma línea o, cuando la columna de
    // flags es ancha, en la línea siguiente con sangría profunda. Una opción
    // sin descripción es una línea de flag «pelada» cuya siguiente línea NO es
    // esa sangría.
    let flag_pelado =
        regex::Regex::new(r"^\s+(-[A-Za-z], )?--[a-z0-9-]+( <[A-Z_]+>)?\s*$").unwrap();
    let descripcion_debajo = regex::Regex::new(r"^\s{8,}[^\s-]").unwrap();
    for p in PANTALLAS {
        let texto = ayuda(p);
        let opciones = seccion(&texto, "Options:");
        for (i, linea) in opciones.iter().enumerate() {
            if flag_pelado.is_match(linea) {
                let siguiente = opciones.get(i + 1).copied().unwrap_or("");
                assert!(
                    descripcion_debajo.is_match(siguiente),
                    "exo {} --help: opción sin descripción: {linea:?}",
                    p.join(" ")
                );
            }
        }
    }
}

#[test]
fn el_metavar_de_un_flag_es_el_nombre_del_flag() {
    let flag = regex::Regex::new(r"--([a-z0-9-]+) <([A-Z_]+)>").unwrap();
    for p in PANTALLAS {
        let texto = ayuda(p);
        for linea in seccion(&texto, "Options:") {
            if let Some(c) = flag.captures(linea) {
                let esperado = c[1].to_uppercase().replace('-', "_");
                assert_eq!(
                    &c[2],
                    esperado,
                    "exo {} --help: `--{}` muestra <{}>",
                    p.join(" "),
                    &c[1],
                    &c[2]
                );
            }
        }
    }
}
```

- [ ] **Step 2: Verlo fallar**

Run: `cd engine && cargo test --release --test help_producto`
Expected: `la_lista_cubre_todos_los_subcomandos ... ok`; FAIL en
`toda_opcion_lleva_descripcion` con
`exo init --help: opción sin descripción: "      --json               "` y en
`el_metavar_de_un_flag_es_el_nombre_del_flag` con
``exo search --help: `--limit` muestra <LIMITE>``.

Si tras A el primer fallo nombra `--db` de `init`, es que A no le puso
descripción: añádesela con el mismo texto que `ArgsIndex.db`.

- [ ] **Step 3: Implementación mínima**

Guardar como `$TMPDIR/h9-metavars.py` y correr desde la raíz del repo
(falla ruidoso si un ancla no aparece las veces esperadas; si A cambió un
ancla, ajusta el ancla, no el resultado):

```python
# Uso (cwd = raíz del repo): python3 h9-metavars.py engine/src/main.rs
# Metavar = nombre del flag en MAYÚSCULAS, y descripción para los `--json`
# que no la tenían. Falla ruidoso si un ancla no aparece las veces esperadas.
import sys
f = sys.argv[1]
s = open(f).read()
R = [
    ('#[arg(long = "limit", alias = "limite", default_value_t = 10)]',
     '#[arg(long = "limit", alias = "limite", value_name = "LIMIT", default_value_t = 10)]', 1),
    ('#[arg(long = "limit", alias = "limite", default_value_t = 5)]',
     '#[arg(long = "limit", alias = "limite", value_name = "LIMIT", default_value_t = 5)]', 1),
    ('#[arg(long = "limit", default_value_t = 10)]',
     '#[arg(long = "limit", value_name = "LIMIT", default_value_t = 10)]', 1),
    ('#[arg(long = "min-similarity", alias = "min-similitud")]',
     '#[arg(long = "min-similarity", alias = "min-similitud", value_name = "MIN_SIMILARITY")]', 2),
    ('#[arg(long = "fts-scale", alias = "escala-fts")]',
     '#[arg(long = "fts-scale", alias = "escala-fts", value_name = "FTS_SCALE")]', 1),
    ('#[arg(long = "title", alias = "titulo")]',
     '#[arg(long = "title", alias = "titulo", value_name = "TITLE")]', 1),
    ('#[arg(long = "note", alias = "nota")]',
     '#[arg(long = "note", alias = "nota", value_name = "NOTE")]', 1),
    # --json de init y config
    ('''    force: bool,
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct ArgsConfig {
    #[arg(long)]
    json: bool,
}''',
     '''    force: bool,
    /// Emite el resultado como envelope JSON en stdout.
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct ArgsConfig {
    /// Emite la config como envelope JSON en stdout (para scripts: jq no lee
    /// TOML).
    #[arg(long)]
    json: bool,
}''', 1),
    # --json de write new
    ('''    /// Salta el dup-gate de similitud. JAMÁS salta una colisión de fichero.
    #[arg(long)]
    force: bool,
    #[arg(long)]
    json: bool,''',
     '''    /// Salta el dup-gate de similitud. JAMÁS salta una colisión de fichero.
    #[arg(long)]
    force: bool,
    /// Emite el resultado como envelope JSON en stdout.
    #[arg(long)]
    json: bool,''', 1),
    # --json de write append
    ('''    force: bool,
    #[arg(long)]
    json: bool,
    /// Permalink''',
     '''    force: bool,
    /// Emite el resultado como envelope JSON en stdout.
    #[arg(long)]
    json: bool,
    /// Permalink''', 1),
]
for viejo, nuevo, n in R:
    c = s.count(viejo)
    if c != n:
        sys.exit(f"esperaba {n} ocurrencia(s), hay {c}:\n{viejo}")
    s = s.replace(viejo, nuevo)
open(f, "w").write(s)
print(f"{len(R)} reemplazos aplicados")
```

Run: `python3 "$TMPDIR/h9-metavars.py" engine/src/main.rs && cd engine && cargo fmt`
Expected: `10 reemplazos aplicados`; rustfmt parte en varias líneas los
`#[arg(...)]` que pasan de 100 columnas.

- [ ] **Step 4: Verlo pasar**

Run: `cd engine && cargo test --release --test help_producto --test flags`
Expected: `help_producto`: 3 passed; `flags`: 4 passed (los alias españoles
siguen parseando y ocultos).

Falsación (no se commitea): borra el `/// Emite el resultado como envelope
JSON en stdout.` de `ArgsInit.json`, corre el test → FAIL
`exo init --help: opción sin descripción`; restaura con
`git -C /home/paul/Documentos/proyectos/exo checkout -p engine/src/main.rs` o
deshaciendo la edición, y vuelve a verde.

Run: `cd engine && cargo clippy --all-targets --locked -- -D warnings`
Expected: sin avisos.

- [ ] **Step 5: Commit**

```bash
git add engine/tests/help_producto.rs engine/src/main.rs
git commit -m "fix(cli): metavars iguales al flag y descripcion en todo --json (H9)"
```

---

### Task 3: `--help` de producto, sin jerga de campaña (H8)

**Lane:** mecánica (oráculo literal abajo). **Depende de A:** sí.
**Bloqueada por D1**: no corre hasta que Paul conteste. Con C o B se aplica
tal cual (textos en español); con A se traducen estos mismos textos.
**Oráculo:** `cd engine && cargo test --release --test help_producto --test flags`.

**Evidencia (H8), medida con el binario de `3c1918f`:** `about = "engine del
framework exo (E1: read)"` (`main.rs:28`); `search` «Búsqueda FTS5 mínima
sobre `notas_fts` (spec §4.1, m2-05)» (`:47`); `write` «(M4/E2)» (`:49`);
`recall` «sucesor de `basic-memory-recall.sh` y `compose-inject.sh` de reflex
(M2-08, M6)» (`:53-56`); `targets`/`budget`/`lint`/`ratchet` «G4a/G4b/G4c»,
«sucesor de `kbx doctor`» (`:58-74`); `(spec §4)` en siete `--json`;
`--bonus`/`--fts-scale` «(M2-07, §5.2.6)… `BONUS_SELLADO`» (`:222-232`);
`write append --create` «(documenta.md la pide…)»; `--force` documenta
`forzado: true` cuando la clave es `forced` (`escritor.rs:105`); ejemplo de
permalink `kb-demo/…`. El test de abajo cuenta **91 líneas de jerga** en las
15 pantallas de ayuda.

**Files:**
- Modify: `engine/tests/help_producto.rs` (añadir `JERGA` y un test)
- Modify: `engine/src/main.rs` (doc-comments de `Cli`, `Comando`,
  `ComandoWrite` y `Args*`)

**Interfaces:**
- Consumes: `ayuda`, `PANTALLAS` de Task 2.
- Produces: `const JERGA: &[(&str, &str)]`, test `la_ayuda_no_lleva_jerga_interna`.

- [ ] **Step 1: Escribir el test que falla**

Añadir al final de `engine/tests/help_producto.rs`:

```rust
/// Jerga que delata que la ayuda la escribió el autor para sí mismo. Cada
/// entrada: patrón y por qué no le dice nada a un usuario.
const JERGA: &[(&str, &str)] = &[
    (
        r"\b[MGDE][0-9]+[a-z]?(-[0-9]+)?\b",
        "id de hito interno (M2-07, G4b, D6, E1)",
    ),
    (r"\b[mg][0-9]+-[0-9]+\b", "id de item de campaña (m2-05)"),
    (r"\bD-f[0-9]", "id de decisión de spec"),
    (r"§", "sección de una spec"),
    (r"\b(spec|brief|Task)\b", "documento de proceso"),
    (r"\bsucesor\b", "genealogía del código"),
    (r"kbx [a-z]", "comando de otra herramienta"),
    (r"kb-demo", "nombre de la KB del autor"),
    (
        r"basic-memory-recall|compose-inject|replay-engine|documenta\.md|\breflex\b",
        "script o skill del autor",
    ),
    (
        r"::|busca_hybrid|notas_fts|`notas`|fallido\(\)|_SELLAD",
        "símbolo interno de Rust o SQL",
    ),
];

#[test]
fn la_ayuda_no_lleva_jerga_interna() {
    let patrones: Vec<(regex::Regex, &str)> = JERGA
        .iter()
        .map(|(p, por_que)| (regex::Regex::new(p).unwrap(), *por_que))
        .collect();
    let mut hallazgos = Vec::new();
    for p in PANTALLAS {
        let texto = ayuda(p);
        for linea in texto.lines() {
            for (re, por_que) in &patrones {
                if let Some(m) = re.find(linea) {
                    hallazgos.push(format!(
                        "exo {} --help: {:?} ({por_que}) en {linea:?}",
                        p.join(" "),
                        m.as_str()
                    ));
                }
            }
        }
    }
    assert!(
        hallazgos.is_empty(),
        "jerga en la ayuda:\n{}",
        hallazgos.join("\n")
    );
}
```

- [ ] **Step 2: Verlo fallar**

Run: `cd engine && cargo test --release --test help_producto la_ayuda 2>&1 | grep -c "exo .*--help:"`
Expected: `91` en `3c1918f` (tras A puede variar: cualquier número > 0).

- [ ] **Step 3: Implementación mínima**

Guardar como `$TMPDIR/h8-reemplazos.py` y correr desde la raíz del repo.
Textos en español (status quo; si D1 = A, se traducen **estos mismos**
textos y el oráculo no cambia):

```python
# Uso: python3 h8-reemplazos.py engine/src/main.rs
import sys
f = sys.argv[1]
s = open(f).read()
R = [
# (viejo, nuevo, ocurrencias esperadas)
('about = "engine del framework exo (E1: read)"',
 'about = "Memoria persistente para agentes: indexa una KB de notas markdown y la sirve por búsqueda y recall."', 1),
('''    /// Crea `~/.exo/config.toml`. Con `--from-basic-memory`, migra los valores
    /// de `~/.basic-memory/config.json` una sola vez.
''',
 '''    /// Crea la config (`~/.exo/config.toml`) y una KB nueva desde la
    /// plantilla, ya indexada. Con `--from-basic-memory`, adopta una KB
    /// existente de basic-memory.
''', 1),
('''    /// Emite la config efectiva como envelope JSON, con las rutas ya
    /// expandidas. Existe para los consumidores en shell: jq no lee TOML.
''',
 '''    /// Muestra la config efectiva, con las rutas ya expandidas.
''', 1),
('''    /// Indexa la KB de forma incremental (mtime al invocar, sin daemon).
''',
 '''    /// Indexa la KB de forma incremental: solo lo que cambió desde la última
    /// vez.
''', 1),
('''    /// Borra la DB y reconstruye desde cero (primera clase, no cirugía —
    /// spec §3: "corrupción de índice = borrar y rebuild").
''',
 '''    /// Borra el índice y lo reconstruye desde cero. Es el remedio ante un
    /// índice corrupto.
''', 1),
('''    /// Búsqueda FTS5 mínima sobre `notas_fts` (spec §4.1, m2-05).
''',
 '''    /// Busca en la KB: texto completo (`fts`), semántica (`vector`) o las dos
    /// fusionadas (`hybrid`).
''', 1),
('''    /// Escribe en la KB (M4/E2): nota nueva o append a bitácora. File-first,
    /// sin commit y sin indexar — eso es del agente y del recall siguiente.
''',
 '''    /// Escribe en la KB: nota nueva o entrada de bitácora. No commitea ni
    /// indexa.
''', 1),
('''    /// Sirve contenido de la KB para arranque (`tier: core` + recientes) o
    /// consulta (`busca_hybrid`) — sucesor de `basic-memory-recall.sh` y
    /// `compose-inject.sh` de reflex (M2-08, M6). NO conoce reflex ni
    /// perfiles de agentes: eso lo compone el consumidor.
''',
 '''    /// Sirve memoria de la KB a un agente. Sin `--query`, el bloque de
    /// arranque (notas `tier: core` y recientes); con `--query`, las notas
    /// relevantes para esa consulta.
''', 1),
('''    /// Candidatas de la KB para un tema: FTS5 más tier/tamaño/headings de
    /// disco y último commit de git (spec §4, primer verbo portado del
    /// núcleo de `kbx`, G4a). Solo lectura: no gatea nada.
''',
 '''    /// Lista las notas candidatas a tocar para un tema, con tier, tamaño,
    /// cabeceras y último commit. Solo lectura.
''', 1),
('''    /// Presupuestos por tier sobre el árbol de ficheros (`presupuesto::analiza`,
    /// G4b). Emite el informe entero y LUEGO gatea: exit 3 si hay
    /// infractoras o notas sin tier legal, nunca por el aviso de aire.
''',
 '''    /// Comprueba el presupuesto de bytes por tier. Imprime el informe entero y
    /// sale con 3 si alguna nota lo rebasa o no declara un tier válido.
''', 1),
('''    /// Los siete checks de deriva de la KB (`lint::analiza`, G4b), sucesor de
    /// `kbx doctor` en bare mode. Emite el informe entero y LUEGO gatea:
    /// exit 3 si `ok` es falso.
''',
 '''    /// Comprueba la salud de la KB: notas huérfanas, frontmatter roto, índice
    /// desfasado y más. Imprime el informe entero y sale con 3 si hay
    /// hallazgos.
''', 1),
('''    /// El trinquete de techos declarados (`trinquete::comprueba`, G4c),
    /// sucesor de `kbx ratchet`. Emite el informe entero y LUEGO gatea: exit
    /// 3 si `informe.fallido()`. Abstención (sin historia de git utilizable)
    /// sale 0, no 3: es información, no un fallo.
''',
 '''    /// Comprueba que ningún techo de tamaño declarado suba respecto al último
    /// commit. Imprime el informe entero y sale con 3 si alguno sube; sin
    /// historia de git se abstiene y sale con 0.
''', 1),
('''    /// Preflight de ENTORNO —la máquina—, no de la KB: eso es `lint`. Emite
    /// el informe entero y LUEGO gatea: exit 3 si algún check sale `fail`.
    /// Los `warn` informan sin gatear y los `na` declaran lo que no se mide
    /// en esta plataforma, en vez de desaparecer de la lista.
''',
 '''    /// Diagnostica esta máquina (binario, config, KB, índice, modelo de
    /// embeddings y dependencias de los hooks). Cada check dice qué artefacto
    /// miró; sale con 3 si alguno falla.
''', 1),
# --- flags ---
('''    /// Emite el resultado como envelope JSON (spec §4) en stdout.
''',
 '''    /// Emite el resultado como envelope JSON en stdout.
''', 7),
('''    /// Máximo de resultados. Default 10 (replay-engine pasa el suyo
    /// explícito; flags > config).
''',
 '''    /// Máximo de resultados.
''', 1),
('''    /// Tipo de búsqueda (fts|vector|hybrid, M2-07). Default `fts`:
    /// comportamiento actual intacto si no se pasa el flag.
''',
 '''    /// Tipo de búsqueda.
''', 1),
('''    /// Umbral de similitud coseno del arm vector/hybrid. Opcional: si se
    /// omite, cae a `[embeddings] min_similarity` de `~/.exo/config.toml`
    /// (D6, precedencia flags > config). Sin efecto en `--type fts`.
''',
 '''    /// Umbral de similitud coseno de la búsqueda semántica. Si se omite,
    /// `[embeddings] min_similarity` de la config. Sin efecto en `--type fts`.
''', 1),
('''    /// Peso del canal débil en la fórmula de fusión (`bonus·min(v,f)`,
    /// spec fusión §4.4). Solo para `--type hybrid`: override puntual del
    /// sellado (M2-07, §5.2.6); si se omite, cae al default sellado
    /// `BONUS_SELLADO`.
''',
 '''    /// Peso del canal más débil al fusionar (`max + bonus·min`). Solo
    /// `--type hybrid`; si se omite, el default del engine.
''', 1),
('''    /// Anclaje β de la normalización BM25 por-query (spec fusión §4.3,
    /// D-f1). Solo para `--type hybrid`: override puntual del sellado
    /// (M2-07, §5.2.6); si se omite, cae al default sellado
    /// `ESCALA_FTS_SELLADA`.
''',
 '''    /// Escala de normalización del score de texto completo antes de
    /// fusionar. Solo `--type hybrid`; si se omite, el default del engine.
''', 1),
('''    /// Precedencia: flag > $EXO_KB > config. `exo recall` la necesita aunque
    /// solo lea del índice: `notas.ruta` es relativa, y modo arranque
    /// también relee `tier` del `.md` en disco (no está en el índice).
''',
 '''    /// Precedencia: flag > $EXO_KB > config.
''', 1),
('''    /// Texto de la consulta. Ausente ⇒ modo arranque (`tier: core` +
    /// recientes por git); presente ⇒ modo consulta (`busca_hybrid`).
''',
 '''    /// Texto de la consulta. Sin él, modo arranque (`tier: core` + recientes
    /// por git); con él, modo consulta (búsqueda híbrida).
''', 1),
('''    /// Máximo de notas. En modo arranque, tope del bloque de "recientes"
    /// (los `tier: core` siempre entran todos); en modo consulta, tope de
    /// `busca_hybrid`. Default 5 (contrato del brief para modo consulta;
    /// mismo flag, mismo default en ambos modos).
''',
 '''    /// Máximo de notas: en modo arranque, cuántas recientes (las `tier: core`
    /// entran siempre); en modo consulta, cuántos resultados.
''', 1),
('''    /// Presupuesto de bytes del bloque de salida (texto o `--json`), trunca
    /// por líneas ENTERAS. Default 2048 (brief).
''',
 '''    /// Presupuesto de bytes del bloque de salida (texto o `--json`); trunca
    /// por líneas enteras.
''', 1),
('''    /// Umbral de similitud coseno del arm vector de `busca_hybrid` (modo
    /// consulta). Sin efecto en modo arranque. Default de config si se
    /// omite (D6, mismo contrato que `search`).
''',
 '''    /// Umbral de similitud coseno en modo consulta. Si se omite, el de la
    /// config. Sin efecto en modo arranque.
''', 1),
('''    /// Modo arranque en versión CONTENIDO: vuelca el cuerpo de las notas
    /// `tier: core` + lista de recientes, en vez de una línea por nota. Es
    /// lo que consume el hook de SessionStart (paridad con el
    /// `basic-memory-recall.sh` que sustituye, que inyectaba el cuerpo del
    /// core-index, no sus rutas). Incompatible con `--query`.
''',
 '''    /// Modo arranque con el CUERPO de las notas `tier: core` y la lista de
    /// recientes, en vez de una línea por nota. Es lo que inyecta el hook de
    /// inicio de sesión. Incompatible con `--query`.
''', 1),
('''    /// Refresca el índice (indexado incremental) ANTES de servir, para no
    /// devolver un bloque de una KB rancia (M6-01, "índice fresco sin
    /// daemon"). Barato cuando nada cambió: un `stat` por fichero y ninguna
    /// carga del modelo. Si la DB no existe, la construye (bootstrap).
''',
 '''    /// Refresca el índice (incremental) ANTES de servir, para no devolver una
    /// KB rancia. Barato si nada cambió; si el índice no existe, lo construye.
''', 1),
('''    /// Emite el resultado como envelope JSON (spec §4) en stdout. Sin este
    /// flag, imprime un bloque de texto plano (el que consumirá el hook).
''',
 '''    /// Emite el resultado como envelope JSON en stdout. Sin él, un bloque de
    /// texto plano (el que inyectan los hooks).
''', 1),
('''    /// Máximo de candidatas. Default 10, igual que `kbx targets`.
''',
 '''    /// Máximo de candidatas.
''', 1),
('''    /// Crea la bitácora si no existe (documenta.md la pide con `tier: log`).
''',
 '''    /// Crea la bitácora (`tier: log`) si no existe.
''', 1),
('''    /// Anexa aunque el destino no sea `tier: log`. Queda registrado en el
    /// envelope (`forzado: true`) para que la excepción sea auditable.
''',
 '''    /// Anexa aunque el destino no sea `tier: log`. Queda registrado en el
    /// envelope (`forced: true`) para que la excepción sea auditable.
''', 1),
('''    /// Permalink de la nota destino (p.ej. `kb-demo/log/exo-bitacora`).
''',
 '''    /// Permalink de la nota destino (p.ej. `mi-kb/log/proyecto-bitacora`).
''', 1),
('''    /// Fichero SQLite del índice. Precedencia: flag > $EXO_DB > config. A
    /// diferencia de `budget`, `lint` sí lo necesita: los checks `orphan` e
    /// `index_stale` leen `notas`.
''',
 '''    /// Fichero SQLite del índice. Precedencia: flag > $EXO_DB > config.
    /// `lint` lo necesita para detectar huérfanas e índice desfasado.
''', 1),
]
for viejo, nuevo, n in R:
    c = s.count(viejo)
    if c != n:
        sys.exit(f"esperaba {n} ocurrencia(s), hay {c}:\n{viejo}")
    s = s.replace(viejo, nuevo)
open(f, "w").write(s)
print(f"{len(R)} reemplazos aplicados")
```

Run: `python3 "$TMPDIR/h8-reemplazos.py" engine/src/main.rs && cd engine && cargo fmt`
Expected: `32 reemplazos aplicados`.

Positional de `targets` (D1 = A o C): en `struct ArgsTargets`, justo encima de
`    tema: String,` añadir `    #[arg(value_name = "TOPIC")]` (hoy se pinta
`<TEMA>` junto a `<QUERY>` de `search`). Con D1 = B no se toca.

Si A escribió doc-comments nuevos (p.ej. `ArgsInit.db`) y el test los marca,
reescríbelos con el mismo criterio: qué hace para el usuario, sin ids de
hito, specs ni símbolos internos. La procedencia que se quite y valga la pena
conservar va a un comentario `//` (no lo renderiza clap), no a la ayuda.

- [ ] **Step 4: Verlo pasar**

Run: `cd engine && cargo test --release --test help_producto --test flags`
Expected: `help_producto`: 4 passed; `flags`: 4 passed.

Run: `engine/target/release/exo --help` (tras `cargo build --release` en `engine/`)
Expected: primera línea `Memoria persistente para agentes: indexa una KB de
notas markdown y la sirve por búsqueda y recall.`

Run: `cd engine && cargo clippy --all-targets --locked -- -D warnings && ./scripts/test-hermetico.sh`
Expected: sin avisos; `test-hermetico: OK`.

(Tasks 2+3 se corrieron el 2026-09-13 sobre una copia de `3c1918f`: 4/4
`help_producto`, 4/4 `flags`, clippy limpio, gate hermético OK.)

- [ ] **Step 5: Commit**

```bash
git add engine/tests/help_producto.rs engine/src/main.rs
git commit -m "docs(cli): --help de producto, sin jerga de campaña, con test que lo fija (H8)"
```

---

### Task 4: `_truncate-payload.sh` — un solo bloque de truncado para los dos reflejos de Bash (H20)

**Lane:** mecánica. **Depende de A:** no. **Oráculo:**
`plugins/exo/scripts/test-git-add-all-guard.sh && plugins/exo/scripts/test-verify-before-commit.sh`
(13/13 cada uno, sin tocarlos) **y** el diferencial del Step 4 (payload
byte-idéntico al de `HEAD` en 6 casos por reflejo).

**Evidencia (H20):** `plugins/exo/scripts/git-add-all-guard.sh:79-104` y
`verify-before-commit.sh:142-167` son idénticos (`diff` vacío), con 35 y 32
líneas de comentario casi idénticas delante (`:44-78`, `:110-141`).
Patrón de helper existente: `_timeout.sh`, `_reflex-log.sh`.

**Files:**
- Create: `plugins/exo/scripts/_truncate-payload.sh` (100755)
- Modify: `plugins/exo/scripts/git-add-all-guard.sh` (`:44-104` → 6 líneas)
- Modify: `plugins/exo/scripts/verify-before-commit.sh` (`:110-167` → 6 líneas)

**Interfaces:**
- Consumes: `$CMD` y `$PATRON` ya definidos por cada reflejo; `_reflex-log.sh`
  (cap de 2000 chars en `reflex_log`).
- Produces: `payload_truncado CMD PATRON` — asigna la global `PAYLOAD`
  (no imprime: `$(...)` se comería saltos de línea finales).

- [ ] **Step 1: Diferencial que fija el comportamiento actual**

Guardar fuera del repo como `$TMPDIR/diff-payload.sh` y `chmod +x`:

```bash
#!/usr/bin/env bash
# Diferencial: payload logueado por el guard viejo (repo real) vs nuevo (clon).
set -uo pipefail
VIEJO="$1"; NUEVO="$2"; REFLEJO="$3"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
git init -q "$TMP/r" && echo 'fn main(){}' > "$TMP/r/a.rs" && git -C "$TMP/r" add a.rs
largo="$(printf 'x%.0s' $(seq 1 300))"
ruta="$(printf 'd/%.0s' $(seq 1 200))"
multilinea="$(for i in $(seq 1 40); do echo "linea corta $i"; done)"
casos=(
  "git add -A"
  "git commit -m corto"
  "echo $largo && git add -A && git commit -m x"
  "cat <<X
$multilinea
nunca uses git add . aqui
X
git add . && git add --all && git add . && git commit -m y"
  "git -C /$ruta add -A ; git -C /$ruta commit -m z"
  "echo $largo git add .
git add .
git add .
git add .
git add .
git add .
git add -A git commit -m q"
)
fallos=0
for i in "${!casos[@]}"; do
  cmd="${casos[$i]}"
  payload="$(jq -n --arg c "$cmd" --arg w "$TMP/r" '{session_id:"s",tool_name:"Bash",tool_input:{command:$c},hook_event_name:"PreToolUse",transcript_path:"/no/existe",cwd:$w}')"
  for lado in VIEJO NUEVO; do
    : > "$TMP/$lado.jsonl"
    printf '%s' "$payload" | REFLEX_LOG_FILE="$TMP/$lado.jsonl" "${!lado}" >/dev/null 2>&1
  done
  a="$(jq -r "select(.reflex==\"$REFLEJO\") | .payload" "$TMP/VIEJO.jsonl")"
  b="$(jq -r "select(.reflex==\"$REFLEJO\") | .payload" "$TMP/NUEVO.jsonl")"
  if [ "$a" == "$b" ]; then echo "[IGUAL] caso $i (${#a} chars)"; else echo "[DISTINTO] caso $i"; diff <(echo "$a") <(echo "$b"); fallos=$((fallos+1)); fi
done
exit $fallos
```

Snapshot de los guards antes de tocarlos:

```bash
mkdir -p "$TMPDIR/viejos"
git -C /home/paul/Documentos/proyectos/exo show HEAD:plugins/exo/scripts/git-add-all-guard.sh > "$TMPDIR/viejos/git-add-all-guard.sh"
git -C /home/paul/Documentos/proyectos/exo show HEAD:plugins/exo/scripts/verify-before-commit.sh > "$TMPDIR/viejos/verify-before-commit.sh"
git -C /home/paul/Documentos/proyectos/exo show HEAD:plugins/exo/scripts/_reflex-log.sh > "$TMPDIR/viejos/_reflex-log.sh"
chmod +x "$TMPDIR"/viejos/*.sh
```

- [ ] **Step 2: Crear el helper**

`plugins/exo/scripts/_truncate-payload.sh`:

```bash
#!/usr/bin/env bash
# Helper COMPARTIDO: el payload que un reflejo de PreToolUse:Bash persiste en
# el log cuando dispara. Lo usan git-add-all-guard.sh y verify-before-commit.sh,
# que llevaban este bloque copiado línea a línea.
#
# Uso:
#   PAYLOAD="${CMD:0:120}"   # fallback si el helper no se puede cargar
#   . "$(dirname "$0")/_truncate-payload.sh" 2>/dev/null && payload_truncado "$CMD" "$PATRON"
#
# Asigna la variable global PAYLOAD en vez de imprimir: una sustitución
# `$(...)` se comería los saltos de línea finales, y el bloque que sustituye
# asignaba sin subshell.
#
# Contrato: comando corto (<=120 chars) -> el comando entero, sin marcador.
# Comando largo -> prefijo de 120 chars + " … ⟨match⟩ " + TODAS las
# ocurrencias de PATRON (hasta MATCH_MAX, deduplicadas preservando orden,
# unidas con " | "), cada una truncada POR SEPARADO a cabeza+cola. Sin
# ocurrencias (no debería pasar: el reflejo ya comprobó que PATRON casa) ->
# solo el prefijo.
#
# OJO: el prefijo se saca con expansión de parámetro (${cmd:0:120}), NO con
# `cut -c`. `cut -c` trunca POR LÍNEA, no el string completo: con un comando
# de muchas líneas cortas cada una sobrevive intacta y el "prefijo" real acaba
# siendo líneas*120 caracteres, reventando el cap de _reflex-log.sh y
# comiéndose el match.
# OJO 2: PATRON no tiene techo (un path absurdo tras `-C`, espacios sin fin
# tras `git`), así que un match gigante puede por sí solo topar el cap de 2000
# de _reflex-log.sh y comerse la sentencia que disparó. Por eso CADA ocurrencia
# se trunca por separado, cabeza Y cola: el fragmento que informa vive al
# FINAL del match.
# OJO 3: TODAS las ocurrencias y no solo la primera (`head -1`). Con 2+
# ocurrencias en el mismo comando -- una mención en prosa dentro de un heredoc
# seguida de la invocación real -- quedarse con la primera loguea la mención
# inocua y esconde la real: el falso positivo benigno que el instrumento
# existe para medir, entrando por otra puerta.
# Nota: estos cortes cuentan caracteres en locale UTF-8 pero bytes en
# LC_ALL=C; el corte puede caer a mitad de un carácter multibyte. jq lo tolera
# (carácter de reemplazo, exit 0) y el contrato best-effort aguanta.

# shellcheck disable=SC2034 # PAYLOAD es la salida: la lee el script que hace source
payload_truncado() {
  local cmd="$1" patron="$2"
  local match_head=80 match_tail=60 match_max=5
  if [ "${#cmd}" -le 120 ]; then
    PAYLOAD="$cmd"
    return 0
  fi
  local prefijo="${cmd:0:120}"
  local matches match="" m
  matches="$(printf '%s' "$cmd" | grep -Eo "$patron" | head -n "$match_max" | awk '!seen[$0]++')"
  while IFS= read -r m; do
    [ -z "$m" ] && continue
    if [ "${#m}" -gt $((match_head + match_tail)) ]; then
      m="${m:0:match_head}…${m: -match_tail}"
    fi
    if [ -z "$match" ]; then
      match="$m"
    else
      match="${match} | ${m}"
    fi
  done <<< "$matches"
  if [ -n "$match" ]; then
    PAYLOAD="${prefijo} … ⟨match⟩ ${match}"
  else
    PAYLOAD="$prefijo"
  fi
}
```

```bash
chmod +x plugins/exo/scripts/_truncate-payload.sh
git add plugins/exo/scripts/_truncate-payload.sh
```

- [ ] **Step 3: Cablear los dos reflejos**

En `plugins/exo/scripts/git-add-all-guard.sh`, sustituir desde la línea
`# payload del log: prefijo de contexto + TODAS las ocurrencias del PATRON`
hasta el `fi` que cierra el bloque `if [ "${#CMD}" -le 120 ]` (inclusive), y
en `plugins/exo/scripts/verify-before-commit.sh` el tramo equivalente (misma
primera línea, mismo `fi` final), por exactamente:

```bash
# payload del log: contexto + TODAS las ocurrencias del PATRON (mismo PATRON
# de arriba: si diverge del de deteccion, el log deja de decir por que
# disparo). Contrato y trampas en _truncate-payload.sh. Si el helper no se
# puede cargar, degrada al prefijo: el reflejo es warn-only y nunca rompe.
PAYLOAD="${CMD:0:120}"
. "$(dirname "$0")/_truncate-payload.sh" 2>/dev/null && payload_truncado "$CMD" "$PATRON"
```

La línea siguiente de cada script (`# log del disparo (best-effort…`) y el
`reflex_log` quedan como están.

- [ ] **Step 4: Verificar — tests existentes y diferencial**

Run: `plugins/exo/scripts/test-git-add-all-guard.sh | tail -1 && plugins/exo/scripts/test-verify-before-commit.sh | tail -1`
Expected: `=== Resultado: 13/13 pasaron ===` dos veces.

Run: `"$TMPDIR/diff-payload.sh" "$TMPDIR/viejos/git-add-all-guard.sh" plugins/exo/scripts/git-add-all-guard.sh zero-residuo`
Expected: 6 líneas `[IGUAL] caso N (…)`, exit 0 (tamaños medidos: 10, 0, 142, 158, 272, 140).

Run: `"$TMPDIR/diff-payload.sh" "$TMPDIR/viejos/verify-before-commit.sh" plugins/exo/scripts/verify-before-commit.sh verify-before-done`
Expected: 6 `[IGUAL]`, exit 0 (tamaños 0, 19, 142, 142, 0, 142).

Falsación del diferencial (no se commitea): cambia `match_tail=60` a `50` en
el helper → el primer diferencial da un `[DISTINTO] caso 4`; restaura.

Run: `bash scripts/test-exec-bit.sh`
Expected: `[OK] 29 scripts bajo plugins/ en 100755`.

- [ ] **Step 5: Commit**

```bash
git add plugins/exo/scripts/_truncate-payload.sh plugins/exo/scripts/git-add-all-guard.sh plugins/exo/scripts/verify-before-commit.sh
git commit -m "refactor(plugin): _truncate-payload.sh, un solo bloque de truncado para zero-residuo y verify-before-done (H20)"
```

---

### Task 5: el reflejo de orquestador limpio mira también la navegación de los MCP de navegador (H21)

**Lane:** diseño (alcance adjudicado con cita). Criterio citable: la
cabecera del propio script, `clean-orchestrator-research.sh:3-5`: «Recuerda
delegar la investigacion web a un subagente […] para no ensuciar el contexto
del PADRE». Navegar con un MCP de navegador es investigación web; navegar a
`localhost` para probar la app propia no lo es. **Cambia comportamiento de
un hook** (no del motor): declarado. **Depende de A:** no. **Oráculo:**
`plugins/exo/scripts/test-clean-orchestrator-research.sh` (5/5; 3/5 contra
`HEAD`).

**Evidencia (H21):** `plugins/exo/hooks/hooks.json:5` `"matcher":
"WebSearch|WebFetch"`. Escapan `mcp__claude-in-chrome__navigate`,
`mcp__claude-in-chrome__get_page_text`,
`mcp__plugin_playwright_playwright__browser_navigate`. Sin test propio
(no existe `test-clean-orchestrator-research.sh`).

Semántica del matcher, documentación oficial de hooks
(`https://code.claude.com/docs/en/hooks.md`): «Simple strings (letters,
digits, `_`, `-`, spaces) → Exact match or `|` separated list · Contains
special characters → JavaScript regex (unanchored)»; tools MCP:
«`mcp__<server>__<tool>`», y para servidores de plugin
«`mcp__plugin_my-plugin_db__.*`». Por eso el patrón nuevo lleva `^…$`
explícitos. Qué trae `tool_input` para una tool MCP **no está documentado**:
el script solo lee `.tool_input.url` y cae a vacío si no existe.

**Files:**
- Modify: `plugins/exo/hooks/hooks.json:5`
- Modify: `plugins/exo/scripts/clean-orchestrator-research.sh`
- Create: `plugins/exo/scripts/test-clean-orchestrator-research.sh` (100755)
- Modify: `README.md:116` y `plugins/exo/README.md:60` (columna Evento de la
  fila clean-orchestrator)

**Interfaces:**
- Consumes: payload PreToolUse `{session_id, tool_name, tool_input, agent_id?}`;
  sentinel `/tmp/claude-clean-orch-<session_id>`; `REFLEX_LOG_FILE`.
- Produces: matcher
  `^(WebSearch|WebFetch|mcp__claude-in-chrome__(navigate|get_page_text)|mcp__.*playwright.*__browser_navigate)$`;
  abstención sin consumir sentinel si `tool_input.url` es local.

- [ ] **Step 1: Escribir el test que falla**

`plugins/exo/scripts/test-clean-orchestrator-research.sh`:

```bash
#!/usr/bin/env bash
# Test standalone para clean-orchestrator-research.sh (PreToolUse, reflejo
# "orquestador limpio") y para el matcher que lo cablea en hooks/hooks.json.
# Warn-only: exit 0 SIEMPRE. Avisa 1x/sesión, solo en el padre, y no ante la
# app local.
# Fixtures en mktemp -d; nunca toca ~/.claude/reflex-log.jsonl real. Los
# sentinels del script viven en /tmp/claude-clean-orch-<session_id>: cada caso
# usa un session_id propio y el trap los borra.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/clean-orchestrator-research.sh"
HOOKS_JSON="${SCRIPT_DIR}/../hooks/hooks.json"

TMP="$(mktemp -d)"
SID="test-clean-orch-$$"
trap 'rm -rf "$TMP"; rm -f /tmp/claude-clean-orch-"$SID"-*' EXIT

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

# corre <session> <tool> <tool_input JSON> [agent_id] -> stdout del hook
corre() {
  local sesion="$1" tool="$2" input="$3" agente="${4:-}"
  jq -nc --arg s "$SID-$sesion" --arg t "$tool" --arg a "$agente" --argjson i "$input" \
    '{session_id:$s, tool_name:$t, tool_input:$i, hook_event_name:"PreToolUse"}
     + (if $a == "" then {} else {agent_id:$a} end)' \
    | REFLEX_LOG_FILE="$TMP/log.jsonl" "$HOOK"
}

avisa() { printf '%s' "$1" | jq -e '.hookSpecificOutput.additionalContext | length > 0' >/dev/null 2>&1; }

# Caso 1: WebSearch en el padre avisa; la segunda vez en la misma sesión calla.
OUT1="$(corre c1 WebSearch '{"query":"late chunking"}')"
OUT1B="$(corre c1 WebFetch '{"url":"https://arxiv.org/abs/2409.04701"}')"
if avisa "$OUT1" && [ -z "$OUT1B" ]; then
  pass "caso1: WebSearch en el padre avisa una vez por sesión"
else
  fail "caso1: WebSearch en el padre avisa una vez por sesión" "out1=$OUT1 out1b=$OUT1B"
fi

# Caso 2: dentro de un subagente calla y NO consume el sentinel.
OUT2="$(corre c2 WebSearch '{"query":"x"}' agente-1)"
OUT2B="$(corre c2 WebSearch '{"query":"x"}')"
if [ -z "$OUT2" ] && avisa "$OUT2B"; then
  pass "caso2: subagente calla sin gastar el aviso del padre"
else
  fail "caso2: subagente calla sin gastar el aviso del padre" "out2=$OUT2 out2b=$OUT2B"
fi

# Caso 3: navegar a la web con el MCP de chrome avisa.
OUT3="$(corre c3 mcp__claude-in-chrome__navigate '{"url":"https://example.com","tabId":1}')"
if avisa "$OUT3"; then
  pass "caso3: navigate de claude-in-chrome a una web avisa"
else
  fail "caso3: navigate de claude-in-chrome a una web avisa" "out3=$OUT3"
fi

# Caso 4: navegar a la app local calla y NO consume el sentinel.
OUT4="$(corre c4 mcp__plugin_playwright_playwright__browser_navigate '{"url":"http://localhost:3000/login"}')"
OUT4B="$(corre c4 mcp__claude-in-chrome__navigate '{"url":"http://127.0.0.1:8080"}')"
OUT4C="$(corre c4 WebFetch '{"url":"https://docs.rs/clap"}')"
if [ -z "$OUT4" ] && [ -z "$OUT4B" ] && avisa "$OUT4C"; then
  pass "caso4: localhost/127.0.0.1 calla sin gastar el aviso"
else
  fail "caso4: localhost/127.0.0.1 calla sin gastar el aviso" "out4=$OUT4 out4b=$OUT4B out4c=$OUT4C"
fi

# Caso 5: el matcher de hooks.json. Claude Code lo evalúa como regex JS sin
# anclar cuando lleva caracteres especiales; las anclas ^...$ van explícitas,
# así que ERE (grep -E) y JS dan lo mismo para este patrón.
MATCHER="$(jq -r '.hooks.PreToolUse[] | select(any(.hooks[]; .command | test("clean-orchestrator-research"))) | .matcher' "$HOOKS_JSON")"
malos=""
for t in WebSearch WebFetch mcp__claude-in-chrome__navigate mcp__claude-in-chrome__get_page_text \
         mcp__plugin_playwright_playwright__browser_navigate mcp__playwright__browser_navigate; do
  printf '%s' "$t" | grep -Eq -- "$MATCHER" || malos="$malos no-casa:$t"
done
for t in Bash WebSearchX mcp__claude-in-chrome__computer \
         mcp__plugin_playwright_playwright__browser_navigate_back \
         mcp__plugin_playwright_playwright__browser_click; do
  printf '%s' "$t" | grep -Eq -- "$MATCHER" && malos="$malos casa:$t"
done
if [ -n "$MATCHER" ] && [ -z "$malos" ]; then
  pass "caso5: el matcher cubre búsqueda y navegación, y nada más"
else
  fail "caso5: el matcher cubre búsqueda y navegación, y nada más" "matcher=$MATCHER;$malos"
fi

echo ""
TOTAL=$((PASS+FAIL))
echo "=== Resultado: ${PASS}/${TOTAL} pasaron ==="
[ $FAIL -eq 0 ] && exit 0 || exit 1
```

```bash
chmod +x plugins/exo/scripts/test-clean-orchestrator-research.sh
git add plugins/exo/scripts/test-clean-orchestrator-research.sh
```

- [ ] **Step 2: Verlo fallar**

Run: `plugins/exo/scripts/test-clean-orchestrator-research.sh`
Expected: `=== Resultado: 3/5 pasaron ===`, exit 1 — FAIL en `caso4`
(el hook avisa ante `http://localhost:3000/login`) y `caso5`
(`matcher=WebSearch|WebFetch; no-casa:mcp__claude-in-chrome__navigate …`).

- [ ] **Step 3: Implementación mínima**

`plugins/exo/hooks/hooks.json`, línea 5:

```json
        "matcher": "^(WebSearch|WebFetch|mcp__claude-in-chrome__(navigate|get_page_text)|mcp__.*playwright.*__browser_navigate)$",
```

`plugins/exo/scripts/clean-orchestrator-research.sh` completo:

```bash
#!/usr/bin/env bash
# PreToolUse (matcher: WebSearch, WebFetch y la navegación de los MCP de
# navegador — claude-in-chrome y playwright; regex exacta en hooks/hooks.json):
# reflejo "orquestador limpio".
# Warn-only, NUNCA bloquea (exit 0 siempre). Recuerda delegar la investigacion
# web a un subagente (Explore / research con modelo barato) para no ensuciar el
# contexto del PADRE (context-rot: mas contexto = peor rendimiento).
#
# Abstencion: (a) SOLO dispara en el PADRE -> dentro de un subagente la web-research
# YA es el patron deseado, avisar ahi es falso positivo (verificado 2026-06-26: el
# input trae `agent_id` no vacio sii corre en un subagente; session_id/transcript_path
# son COMPARTIDos con el padre y no discriminan). (b) Como mucho UNA vez por sesion
# (sentinel). Reframea al primer fallo y calla el resto -> sin nag, sin auto-ensuciar.
# Capa TRIGGER / clase event-watching del proyecto cerebro+reflejos.
set -uo pipefail

INPUT="$(cat)"
command -v jq >/dev/null 2>&1 || exit 0

SESSION_ID="$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)" || SESSION_ID=""
TOOL="$(printf '%s' "$INPUT" | jq -r '.tool_name // empty' 2>/dev/null)" || TOOL=""
AGENT_ID="$(printf '%s' "$INPUT" | jq -r '.agent_id // empty' 2>/dev/null)" || AGENT_ID=""

# Dentro de un subagente (agent_id no vacio): la investigacion web YA esta delegada,
# que es el patron deseado. Abstencion total: ni avisa (seria FP) ni consume el sentinel
# (que es por-sesion y por tanto compartido padre<->hijos).
[ -n "$AGENT_ID" ] && exit 0

# Navegar a la app local que estás probando (dev server, fichero) no es
# investigación web: abstención, y sin consumir el sentinel, para que la
# primera búsqueda web real de la sesión siga avisando. Solo mira `url`:
# WebSearch trae `query` y get_page_text no trae url, así que esos siguen
# el camino normal.
URL="$(printf '%s' "$INPUT" | jq -r '.tool_input.url // empty' 2>/dev/null)" || URL=""
case "$URL" in
  http://localhost|http://localhost[:/]*|https://localhost|https://localhost[:/]*) exit 0 ;;
  http://127.0.0.1|http://127.0.0.1[:/]*|https://127.0.0.1|https://127.0.0.1[:/]*) exit 0 ;;
  http://\[::1\]*|https://\[::1\]*|http://0.0.0.0*|file:*) exit 0 ;;
esac

SENTINEL="/tmp/claude-clean-orch-${SESSION_ID:-nosession}"

# Ya avisado en esta sesion -> calla (abstencion).
[ -f "$SENTINEL" ] && exit 0
touch "$SENTINEL" 2>/dev/null

MSG="⚠️ Reflejo orquestador limpio: estas investigando en web (${TOOL:-WebSearch/WebFetch}) desde el contexto del PADRE. Salvo consulta puntual de una sola llamada, delega a un subagente (Explore para busquedas/lecturas; research-agent con modelo barato) y quedate con la CONCLUSION, no con las fuentes. Context-rot: mas contexto en el padre = peor rendimiento. (Aviso 1x/sesion.)"

# log del disparo (best-effort, nunca rompe el warn-only)
. "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "clean-orchestrator" "$INPUT" "${TOOL}: $(printf '%s' "$INPUT" | jq -r '.tool_input.query // .tool_input.url // empty' 2>/dev/null)" || true

printf '%s' "$MSG" | jq -Rs '{hookSpecificOutput:{hookEventName:"PreToolUse",additionalContext:.}}'

exit 0
```

`README.md` (fila de la tabla de hooks) y `plugins/exo/README.md:60`:
sustituir `` `PreToolUse:WebSearch\|WebFetch` `` por
`` `PreToolUse:WebSearch\|WebFetch\|navegación MCP` ``.

- [ ] **Step 4: Verlo pasar**

Run: `plugins/exo/scripts/test-clean-orchestrator-research.sh && ./scripts/test-plugin.sh | tail -1`
Expected: `=== Resultado: 5/5 pasaron ===`; `test-plugin: OK — 11/11 suites del plugin en verde`.

Run: `jq . plugins/exo/hooks/hooks.json >/dev/null && echo JSON-OK`
Expected: `JSON-OK`.

- [ ] **Step 5: Commit**

```bash
git add plugins/exo/hooks/hooks.json plugins/exo/scripts/clean-orchestrator-research.sh plugins/exo/scripts/test-clean-orchestrator-research.sh README.md plugins/exo/README.md
git commit -m "feat(plugin): orquestador limpio tambien ante la navegacion de los MCP de navegador, no ante localhost (H21)"
```

Nota para la review: verificar en una sesión real (fuera de la fábrica, lo
hace Paul) que el matcher dispara con `mcp__claude-in-chrome__navigate`. El
test valida la regex con ERE, que para este patrón anclado equivale a la de
JS; no prueba el despachador de Claude Code.

---

### Task 6: el gate de exec-bit cubre los scripts sin extensión (H12)

**Lane:** mecánica. **Depende de A:** no. **Oráculo:**
`bash scripts/test-exec-bit.sh` verde en el árbol, y rojo en las dos
falsaciones del Step 4.

**Evidencia (H12):** `scripts/test-exec-bit.sh:16`
`git ls-files -s -- 'plugins/*.sh'`. `plugins/exo/skills/orchestrate/scripts/{review-package,sdd-workspace,task-brief}`
son bash (`#!/usr/bin/env bash`) en 100755 hoy, fuera del gate. Medido: con
`task-brief` en 100644 el gate actual sigue saliendo `[OK]` y exit 0.

**Files:**
- Modify: `scripts/test-exec-bit.sh` (reescritura completa)
- Modify: `.github/workflows/ci.yml:58` (nombre del step)

**Interfaces:**
- Consumes: índice de git (`git ls-files -s`, `git cat-file -p <blob>`).
- Produces: mismo contrato de salida — `[OK] N scripts bajo plugins/ en 100755`
  / `[FAIL] …` + exit 1.

- [ ] **Step 1: Test que falla (falsación del gate actual)**

```bash
git update-index --chmod=-x plugins/exo/skills/orchestrate/scripts/task-brief
bash scripts/test-exec-bit.sh; echo "exit=$?"
```

Expected: `[OK] 28 scripts bajo plugins/ en 100755` en `3c1918f` (`30` con
Tasks 4-5 aplicadas), `exit=0` — el gate actual **no** ve el fichero roto. Deja el 100644 puesto
para el Step 3.

- [ ] **Step 2: Implementación**

`scripts/test-exec-bit.sh` completo:

```bash
#!/usr/bin/env bash
# Gate: todo script versionado bajo plugins/ debe estar en el índice como 100755.
# "Script" = termina en .sh O su contenido empieza por `#!` (shebang).
#
# Los hooks de plugins/exo/hooks/hooks.json invocan los scripts directamente
# ("${CLAUDE_PLUGIN_ROOT}"/scripts/x.sh), sin `bash` delante. Un script
# commiteado en 100644 llega así al cache del plugin y el hook muere con
# `Permission denied` (exit 126) — non-blocking, así que la sesión arranca
# igual y nadie lo ve. Pasó con estilo-directo.sh (4e14edc) en exo 1.1.1.
#
# Por qué también el shebang y no solo `*.sh`: los scripts de
# skills/orchestrate/scripts/ (review-package, sdd-workspace, task-brief) no
# llevan extensión y la skill los invoca por ruta. Con el filtro `*.sh` de la
# primera versión de este gate quedaban fuera sin que nada lo dijera.
#
# Mira el índice de git, no el working tree: lo que se publica es el modo
# commiteado, y un `chmod +x` local sin stagear lo taparía. El contenido
# también sale del índice (`git cat-file`), por la misma razón.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

malos=""
total=0
while read -r modo blob _etapa ruta; do
  es_script=0
  case "$ruta" in
    *.sh) es_script=1 ;;
    *) [ "$(git cat-file -p "$blob" | head -c 2)" = "#!" ] && es_script=1 ;;
  esac
  [ "$es_script" -eq 1 ] || continue
  total=$((total + 1))
  if [ "$modo" != "100755" ]; then
    malos="${malos}${modo} ${ruta}"$'\n'
  fi
done < <(git ls-files -s -- plugins)

if [[ -n "$malos" ]]; then
  echo "[FAIL] scripts sin bit de ejecución en el índice (esperado 100755):" >&2
  printf '%s' "$malos" >&2
  echo "Arreglo: git update-index --chmod=+x <fichero>" >&2
  exit 1
fi

if [[ "$total" -eq 0 ]]; then
  # Un recorrido que no encuentra nada daría verde sin significado.
  echo "[FAIL] no se encontró ningún script bajo plugins/ — el recorrido está roto" >&2
  exit 1
fi

echo "[OK] ${total} scripts bajo plugins/ en 100755"
```

`.github/workflows/ci.yml`, step del job `exec-bit`:

```yaml
      - name: Todo script bajo plugins/ (.sh o con shebang) en 100755
        run: bash scripts/test-exec-bit.sh
```

- [ ] **Step 3: Verlo rojo con el fichero roto y verde tras arreglarlo**

Run: `bash scripts/test-exec-bit.sh; echo "exit=$?"`
Expected: `[FAIL] scripts sin bit de ejecución en el índice (esperado 100755):`
/ `100644 plugins/exo/skills/orchestrate/scripts/task-brief` / `exit=1`.

```bash
git update-index --chmod=+x plugins/exo/skills/orchestrate/scripts/task-brief
bash scripts/test-exec-bit.sh; echo "exit=$?"
```

Expected: `[OK] 33 scripts bajo plugins/ en 100755` con Tasks 4-5 aplicadas
(28 `.sh` de `3c1918f` + helper de Task 4 + test de Task 5 + los 3 sin
extensión), `exit=0`.

- [ ] **Step 4: Segunda falsación (un `.sh` sigue cubierto)**

```bash
git update-index --chmod=-x plugins/exo/scripts/_timeout.sh
bash scripts/test-exec-bit.sh; echo "exit=$?"     # [FAIL] … 100644 plugins/exo/scripts/_timeout.sh, exit=1
git update-index --chmod=+x plugins/exo/scripts/_timeout.sh
bash scripts/test-exec-bit.sh; echo "exit=$?"     # [OK] …, exit=0
git -C /home/paul/Documentos/proyectos/exo status --short   # sin cambios de modo pendientes
```

(Medido el 2026-09-13 en un clon con Tasks 4-5 aplicadas: `[OK] 33`, rojo con
`task-brief` en 100644, rojo con `_timeout.sh` en 100644.)

- [ ] **Step 5: Commit**

```bash
git add scripts/test-exec-bit.sh .github/workflows/ci.yml
git commit -m "ci(exec-bit): el gate cubre los scripts sin extension con shebang (H12)"
```

---

### Task 7: shellcheck en CI, con cada aviso arreglado o justificado en el sitio (H11)

**Lane:** mecánica. **Depende de A:** **sí** (A modifica cuatro scripts que
este gate juzga). **Oráculo:**
`SHELLCHECK="$TMPDIR/shellcheck-v0.11.0/shellcheck" bash scripts/test-shellcheck.sh`
verde, y rojo en la falsación del Step 5.

**Evidencia (H11):** ni `shellcheck` ni `shfmt` en el repo ni en CI
(`ci.yml` solo tiene clippy para Rust). ~5.400 líneas de shell versionadas.
Roturas de shell que llegaron a release: `estilo-directo.sh` exit 126 (sin
bit, exo 1.1.1; arreglado en `8d59ce0`, gate en `8e96243`) y los scripts no
portables a macOS (`ec74f43`).

Medición de base (2026-09-13, `3c1918f`, shellcheck 0.11.0,
`-x -P SCRIPTDIR`, 39 scripts): 47 avisos.

| Fichero:línea | Código | Tratamiento |
|---|---|---|
| `a1-gate.sh:187` | SC2140 warning | Falso positivo (glob `*` entre tramos entrecomillados) → `disable` justificado |
| `subagent-inject.sh:22` | SC2140 warning | Ídem |
| `test-git-c-bash.sh:10` | SC2155 warning | **Arreglo**: `REFLEX_LOG_FILE="$(mktemp)"` + `export REFLEX_LOG_FILE` |
| `compose-inject.sh:52` | SC2012 | `ls -t` = orden por mtime, portable → `disable` |
| `exo-index.sh:80` | SC2016 | Comillas simples a propósito (expande el bash hijo) → `disable` |
| `exo-recall.sh:25` | SC2016 | Backticks de markdown en texto literal → `disable` |
| `git-c-bash.sh:52` | SC2016 | `$` literal en clase de grep → `disable` |
| `recall-inject.sh:74` | SC2018, SC2019 | `tr 'A-Z' 'a-z'` tras plegar acentos con `sed` → `disable` (el propio comentario `:62-66` lo razona) |
| `test-git-c-bash.sh:121` | SC2016 | Literal de test → `disable` |
| `test-recall-inject.sh:114` | SC2181 | `$?` del subshell anterior → `disable` |
| `test-a1-gate.sh` ×36 | SC2015 | Idioma de aserción `[ … ] && pass \|\| fail` → `disable` de fichero |
| `_truncate-payload.sh` (Task 4) | SC2034 | `PAYLOAD` es la salida del helper → `disable` |

**Files:**
- Create: `scripts/test-shellcheck.sh` (100755)
- Modify: `.github/workflows/ci.yml` (job `lint`, step nuevo tras clippy)
- Modify: los 11 scripts de la tabla (una línea de directiva o el arreglo)

**Interfaces:**
- Consumes: `SHELLCHECK` (ruta al binario; default `shellcheck` en PATH);
  índice de git.
- Produces: `scripts/test-shellcheck.sh` — exit 0 con
  `test-shellcheck: OK — N scripts sin avisos`; exit 1 con los avisos.

- [ ] **Step 1: Binario pineado en local**

```bash
curl -fsSL -o "$TMPDIR/sc.tar.xz" https://github.com/koalaman/shellcheck/releases/download/v0.11.0/shellcheck-v0.11.0.linux.x86_64.tar.xz
echo "8c3be12b05d5c177a04c29e3c78ce89ac86f1595681cab149b65b97c4e227198  $TMPDIR/sc.tar.xz" | sha256sum -c -
tar -xJf "$TMPDIR/sc.tar.xz" -C "$TMPDIR"
"$TMPDIR/shellcheck-v0.11.0/shellcheck" --version | sed -n 2p     # version: 0.11.0
```

- [ ] **Step 2: El gate, rojo contra el árbol actual**

`scripts/test-shellcheck.sh`:

```bash
#!/usr/bin/env bash
# Gate: shellcheck sobre todo el bash versionado que se publica o que corre CI.
#
# Por qué: ~4.700 líneas de bash sin análisis estático, y dos roturas de shell
# llegaron a release (estilo-directo.sh sin bit de ejecución, exo 1.1.1). Este
# gate no habría cazado esa en concreto —la caza test-exec-bit.sh—, pero sí la
# clase: comillas, globs, `$?` indirecto, variables sin usar.
#
# Qué entra: todo fichero versionado que termina en .sh, más los ejecutables
# sin extensión cuyo shebang es sh/bash (skills/orchestrate/scripts/*). Descubre
# por el índice de git, no por lista: un script nuevo entra solo.
# Qué NO entra: evals/ (harness congelado de gates ya firmados: tocarlo
# invalida la corrida que certifica) y docs/.
#
# Cada aviso se arregla o se justifica en el sitio con
# `# shellcheck disable=SCxxxx # <por qué>`. No hay .shellcheckrc global a
# propósito: una exclusión global no dice dónde ni por qué.
#
# `-x -P SCRIPTDIR`: sigue los `. "$(dirname "$0")/_helper.sh"`, así que un
# helper roto o una función mal llamada también cuentan.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

SC="${SHELLCHECK:-shellcheck}"
if ! command -v "$SC" >/dev/null 2>&1; then
  echo "test-shellcheck: no encuentro shellcheck ('$SC'). Instálalo o pasa SHELLCHECK=<ruta>." >&2
  exit 1
fi
"$SC" --version | sed -n '2p'

ficheros=()
while read -r modo blob _etapa ruta; do
  case "$ruta" in evals/*|docs/*) continue ;; esac
  case "$ruta" in
    *.sh) ficheros+=("$ruta") ;;
    *.*) : ;;
    *)
      [ "$modo" = "100755" ] || continue
      if git cat-file -p "$blob" | head -n 1 | grep -Eq '^#!.*[/ ](ba)?sh([[:space:]]|$)'; then
        ficheros+=("$ruta")
      fi
      ;;
  esac
done < <(git ls-files -s)

if [ "${#ficheros[@]}" -eq 0 ]; then
  # Un recorrido que no encuentra nada daría verde sin haber mirado nada.
  echo "test-shellcheck: no se encontró ningún script — el recorrido está roto" >&2
  exit 1
fi

if "$SC" -x -P SCRIPTDIR "${ficheros[@]}"; then
  echo "test-shellcheck: OK — ${#ficheros[@]} scripts sin avisos"
else
  echo "test-shellcheck: avisos arriba. Arregla, o justifica en el sitio con '# shellcheck disable=SCxxxx # <por qué>'." >&2
  exit 1
fi
```

```bash
chmod +x scripts/test-shellcheck.sh
git add scripts/test-shellcheck.sh
SHELLCHECK="$TMPDIR/shellcheck-v0.11.0/shellcheck" bash scripts/test-shellcheck.sh; echo "exit=$?"
```

Expected: `version: 0.11.0`, la lista de avisos, `exit=1`. **Compara la lista
con la tabla de arriba.** Tras A habrá desplazamientos de línea y quizá
avisos nuevos en `recall-inject.sh`, `exo-recall.sh`, `compose-inject.sh` o
`_reflex-log.sh`: cada aviso nuevo se arregla si el arreglo no cambia
comportamiento, o se justifica con `# shellcheck disable=SCxxxx # <por qué>`.
Un aviso que sea **bug real con arreglo que cambia comportamiento** no se
arregla aquí: `disable` con el motivo `bug real, ver backlog` y entrada nueva
en Task 13.

- [ ] **Step 3: Directivas y el arreglo**

Guardar como `$TMPDIR/sc-directivas.py` y correr desde la raíz del repo
(inserta cada directiva justo antes de su línea ancla, con su sangría; falla
si un ancla no aparece exactamente una vez):

```python
# Inserta directivas de shellcheck justificadas ANTES de la línea ancla.
# Uso: python3 sc-directivas.py  (cwd = raíz del repo)
import sys
D = "plugins/exo/scripts/"
EDITS = [
 # (fichero, ancla exacta de la línea (strip), directiva sin sangría)
 (D+"a1-gate.sh", 'for tf in "$PROJ"/*/"$sid"/subagents/"agent-${aid}.jsonl"; do',
  "# shellcheck disable=SC2140 # glob `*` entre tramos entrecomillados: intencionado"),
 (D+"subagent-inject.sh", 'for m in "$PROJECTS_DIR"/*/"$SID"/subagents/"agent-${AID}.meta.json"; do',
  "# shellcheck disable=SC2140 # glob `*` entre tramos entrecomillados: intencionado"),
 (D+"compose-inject.sh", '{ ls "$KB"/core/*.md 2>/dev/null; ls -t "$KB"/projects/*.md 2>/dev/null | head -2; } \\',
  "# shellcheck disable=SC2012 # `ls -t` = orden por mtime; find no lo da portable (macOS/Git Bash)"),
 (D+"exo-index.sh", '"$CMD_BIN" //c start \'""\' //b "$BASH_BIN" -c \\',
  "# shellcheck disable=SC2016 # comillas simples a propósito: expande el bash hijo, no este"),
 (D+"exo-recall.sh", "FALLBACK='Tu memoria persistente es una KB de notas markdown servida por el engine `exo` (`exo recall`, `exo search --type hybrid`). Antes de empezar trabajo sustantivo, busca ahi contexto relevante. Al cerrar una sesion con decisiones/aprendizajes, documentalos con /document (busca antes de escribir; edita la nota canonica en vez de duplicar).",
  "# shellcheck disable=SC2016 # texto literal: los backticks son markdown, no sustitución"),
 (D+"git-c-bash.sh", 'if [ "${REWRITE_OK:-0}" = "1" ] \\',
  "# shellcheck disable=SC2016 # `$` literal dentro de la clase de caracteres de grep"),
 (D+"recall-inject.sh", 't="$(printf \'%s\' "$1" | sed \\',
  "# shellcheck disable=SC2018,SC2019 # los acentos ya los pliega el sed de arriba; tr solo ve ASCII"),
 (D+"test-git-c-bash.sh", 'assert_logged "path con variable → log-only" \\',
  "# shellcheck disable=SC2016 # el test pasa el `$DIR` literal, sin expandir"),
 (D+"test-recall-inject.sh", 'if [ $? -eq 0 ]; then pass "F1: \'vale *\' calla aunque el CWD tenga ficheros"',
  "# shellcheck disable=SC2181 # el exit que se mira es el del subshell de arriba, no un comando suelto"),
]
for f, ancla, directiva in EDITS:
    L = open(f).read().split("\n")
    idx = [i for i, l in enumerate(L) if l.strip() == ancla]
    if len(idx) != 1:
        sys.exit(f"{f}: esperaba 1 línea ancla, hay {len(idx)}: {ancla}")
    i = idx[0]
    sangria = L[i][: len(L[i]) - len(L[i].lstrip())]
    L.insert(i, sangria + directiva)
    open(f, "w").write("\n".join(L))
# test-a1-gate.sh: directiva de fichero, antes del primer comando (set -uo pipefail)
f = D + "test-a1-gate.sh"
s = open(f).read()
a = "set -uo pipefail\n"
assert s.count(a) == 1
s = s.replace(a, "# shellcheck disable=SC2015 # `[ … ] && pass || fail` es el idioma de aserción de esta suite: pass nunca falla\n" + a, 1)
open(f, "w").write(s)
# test-git-c-bash.sh: SC2155 se arregla, no se silencia
f = D + "test-git-c-bash.sh"
s = open(f).read()
a = 'export REFLEX_LOG_FILE="$(mktemp)"\n'
assert s.count(a) == 1
s = s.replace(a, 'REFLEX_LOG_FILE="$(mktemp)"\nexport REFLEX_LOG_FILE\n', 1)
open(f, "w").write(s)
print("ok")
```

En `plugins/exo/scripts/_truncate-payload.sh`, justo antes de
`payload_truncado() {`:

```bash
# shellcheck disable=SC2034 # PAYLOAD es la salida: la lee el script que hace source
```

Run: `python3 "$TMPDIR/sc-directivas.py"`
Expected: `ok`.

- [ ] **Step 4: Verde, y los tests del plugin intactos**

Run: `SHELLCHECK="$TMPDIR/shellcheck-v0.11.0/shellcheck" bash scripts/test-shellcheck.sh`
Expected: `version: 0.11.0` / `test-shellcheck: OK — N scripts sin avisos`
(41 en el clon de prueba con Tasks 4-6 aplicadas).

Run: `./scripts/test-plugin.sh | tail -1 && bash scripts/test-exec-bit.sh`
Expected: `test-plugin: OK — 11/11 …`; `[OK] …`.

- [ ] **Step 5: Falsación y CI**

```bash
printf '#!/usr/bin/env bash\nrm -rf $1/tmp\n' > plugins/exo/skills/orchestrate/scripts/roto
chmod +x plugins/exo/skills/orchestrate/scripts/roto && git add plugins/exo/skills/orchestrate/scripts/roto
SHELLCHECK="$TMPDIR/shellcheck-v0.11.0/shellcheck" bash scripts/test-shellcheck.sh; echo "exit=$?"
# Expected: SC2086 sobre `roto`, "test-shellcheck: avisos arriba…", exit=1
git rm -q --cached plugins/exo/skills/orchestrate/scripts/roto && rm plugins/exo/skills/orchestrate/scripts/roto
SHELLCHECK="$TMPDIR/shellcheck-v0.11.0/shellcheck" bash scripts/test-shellcheck.sh; echo "exit=$?"   # OK, exit=0
```

`.github/workflows/ci.yml`, job `lint`, a continuación del step
`cargo clippy -D warnings` (`:43-46`):

```yaml
      # Versión pineada por SHA256: la de la imagen del runner cambia sin
      # aviso y con ella el conjunto de avisos, que es un rojo que nadie
      # provocó. `if: always()`, igual que clippy: un fmt roto no debe
      # esconder un script roto.
      - name: shellcheck 0.11.0 sobre el bash versionado
        if: always()
        run: |
          set -euo pipefail
          curl -fsSL -o /tmp/sc.tar.xz https://github.com/koalaman/shellcheck/releases/download/v0.11.0/shellcheck-v0.11.0.linux.x86_64.tar.xz
          echo "8c3be12b05d5c177a04c29e3c78ce89ac86f1595681cab149b65b97c4e227198  /tmp/sc.tar.xz" | sha256sum -c -
          tar -xJf /tmp/sc.tar.xz -C /tmp
          SHELLCHECK=/tmp/shellcheck-v0.11.0/shellcheck bash scripts/test-shellcheck.sh
```

Run: `python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/ci.yml'))" && echo YAML-OK`
Expected: `YAML-OK` (si no hay PyYAML: `ruby -ryaml -e 'YAML.load_file(".github/workflows/ci.yml")' && echo YAML-OK`).

- [ ] **Step 6: Commit**

```bash
git add scripts/test-shellcheck.sh .github/workflows/ci.yml plugins/exo/scripts/a1-gate.sh plugins/exo/scripts/subagent-inject.sh plugins/exo/scripts/compose-inject.sh plugins/exo/scripts/exo-index.sh plugins/exo/scripts/exo-recall.sh plugins/exo/scripts/git-c-bash.sh plugins/exo/scripts/recall-inject.sh plugins/exo/scripts/test-git-c-bash.sh plugins/exo/scripts/test-recall-inject.sh plugins/exo/scripts/test-a1-gate.sh plugins/exo/scripts/_truncate-payload.sh
git commit -m "ci(lint): shellcheck 0.11.0 pineado sobre el bash versionado; cada aviso arreglado o justificado (H11)"
```

La prueba de que CI lo corre es la primera corrida del PR (la ve Paul): el
step debe aparecer en el job `fmt + clippy` y salir verde.

---

### Task 8: `distill/SKILL.md` en procedimiento + ficheros de carga bajo demanda (H22)

**Lane:** diseño (qué es procedimiento y qué es soporte). Criterio citable:
el patrón de `plugins/exo/skills/debug/SKILL.md:31-32` («Técnicas de soporte
[…]: `techniques.md`») y la cabecera **Cuándo cargar** de
`debug/techniques.md:3`. **Depende de A:** no. **Oráculo:** el script del
Step 1 (verbos preservados, cabeceras de paso preservadas, tamaño) + la cifra
de exit corregida.

**Evidencia (H22):** `skills/distill/SKILL.md` 276 líneas / 15.399 bytes;
el resto de skills 50-87 líneas (`wc -l plugins/exo/skills/*/SKILL.md`).
Además, `:102` afirma «**exit 1 si hay algún offender**» y `exo budget` sale
con **3** (`engine/tests/budget_lint_cli.rs:78`: `"hallazgos son 3, no 1"`).

**Files:**
- Modify: `plugins/exo/skills/distill/SKILL.md`
- Create: `plugins/exo/skills/distill/rotacion.md`
- Create: `plugins/exo/skills/distill/chequeos.md`
- Create: `plugins/exo/skills/distill/consolidacion.md`

**Interfaces:**
- Consumes: nada ejecutable; la skill la carga Claude Code por nombre.
- Produces: `SKILL.md` ≤ 130 líneas con los 7 pasos (`### 0.` … `### 5.` y
  `### 1b.`) y punteros; tres ficheros con cabecera `**Cuándo cargar:**`.

- [ ] **Step 1: El oráculo, rojo**

Guardar como `$TMPDIR/oraculo-distill.sh`:

```bash
#!/usr/bin/env bash
# Oráculo de H22: el procedimiento sobrevive entero al reparto.
set -uo pipefail
R=/home/paul/Documentos/proyectos/exo
D=plugins/exo/skills/distill
verbos() { grep -oE '\$(EXO|KBX)_BIN [a-z-]+( --[a-z-]+)*|git -C \$KB_ROOT [a-z-]+|exo recall --[a-z-]+' | sort -u; }
pasos()  { grep -E '^### ' | sort; }
fallos=0
antes="$(git -C "$R" show HEAD:$D/SKILL.md)"
despues="$(cat "$R"/$D/*.md)"
diff <(printf '%s\n' "$antes" | verbos) <(printf '%s\n' "$despues" | verbos) || { echo "[FAIL] verbos perdidos o cambiados"; fallos=1; }
diff <(printf '%s\n' "$antes" | pasos | grep -E '^### ([0-5]|1b)\.') <(pasos < "$R/$D/SKILL.md" | grep -E '^### ([0-5]|1b)\.') || { echo "[FAIL] los pasos no siguen en SKILL.md"; fallos=1; }
n="$(wc -l < "$R/$D/SKILL.md")"; [ "$n" -le 130 ] || { echo "[FAIL] SKILL.md tiene $n líneas (>130)"; fallos=1; }
grep -q 'exit 1 si hay algún offender' "$R/$D/SKILL.md" "$R"/$D/chequeos.md 2>/dev/null && { echo "[FAIL] sigue diciendo exit 1"; fallos=1; }
for f in rotacion chequeos consolidacion; do
  grep -q '^\*\*Cuándo cargar:\*\*' "$R/$D/$f.md" 2>/dev/null || { echo "[FAIL] $f.md sin cabecera Cuándo cargar"; fallos=1; }
  grep -q "\`$f.md\`" "$R/$D/SKILL.md" || { echo "[FAIL] SKILL.md no apunta a $f.md"; fallos=1; }
done
[ "$fallos" -eq 0 ] && echo "[OK] distill repartido sin perder procedimiento ($n líneas)"
exit "$fallos"
```

Run: `bash "$TMPDIR/oraculo-distill.sh"`
Expected: `[FAIL] SKILL.md tiene 276 líneas (>130)`, `[FAIL] sigue diciendo
exit 1`, `[FAIL] rotacion.md sin cabecera Cuándo cargar` (×3, uno por fichero)
y `[FAIL] SKILL.md no apunta a rotacion.md` (×3); exit 1. (Corrido el
2026-09-13 contra `3c1918f`.)

- [ ] **Step 2: Reparto (bloques enteros, sin re-resumir)**

La regla la pone la propia skill (`SKILL.md:179-180`): «**bloques enteros,
nunca re-resumir prosa**». Reparto por líneas de `3c1918f`:

| Líneas actuales | Contenido | Destino |
|---|---|---|
| 1-47 | frontmatter, qué es, regla de oro, resolución de rutas, frontera exo/kbx | `SKILL.md` tal cual |
| 48-57 | paso 0: título y precondición dura | `SKILL.md` |
| 58-96 | paso 0: corrida en seco, `--apply`, verificación de conservación, reversión, por qué va antes | `rotacion.md` (verbatim) — en `SKILL.md` queda: el comando `$KBX_BIN rotate --kb $KB_ROOT --json`, «si `data.rotations` trae entradas, sigue `rotacion.md`» |
| 98-134 | paso 1: `budget`, `ratchet`, `no_air`, `waived`, falla-fuerte | `chequeos.md` (verbatim, **cambiando `exit 1` por `exit 3`** en la línea 102 y «mismas semantics que el viejo `kb-budget-check.sh`, que queda retirado» por «(exit 0 si limpio)») — en `SKILL.md` quedan los dos comandos (`$EXO_BIN budget --json`, `$EXO_BIN ratchet --kb $KB_ROOT --json`), «exit 3 = hay trabajo; detalle e interpretación en `chequeos.md`» y la línea de falla-fuerte |
| 136-165 | paso 1b: `lint`, `stale`, señales del reflex-log, `exo recall … --content` | `chequeos.md` (verbatim) — en `SKILL.md` quedan los comandos `$EXO_BIN lint --json`, `$KBX_BIN stale --json` y «señales de inyección rota: `chequeos.md`» |
| 167-220 | paso 2: evicción editorial, test del título, contrato canon/bitácora, backlog | `consolidacion.md` (verbatim) — en `SKILL.md` queda un resumen de ≤ 5 líneas: la nota canónica es estado vivo, lo fechado va a `log/<slug>-bitacora.md`, «antes de partir, evicción editorial y test del título: `consolidacion.md`» |
| 222-276 | pasos 3, 4, 5 y delegación | `SKILL.md` tal cual |

Cada fichero nuevo abre con:

```markdown
# <Rotación de bitácoras | Chequeos de los pasos 1 y 1b | Consolidación del paso 2>

**Cuándo cargar:** <rotacion: en el paso 0, cuando la corrida en seco trae
`data.rotations` no vacío · chequeos: en los pasos 1 y 1b, al interpretar la
salida de `budget`/`ratchet`/`lint`/`stale` o si algún binario falla ·
consolidacion: en el paso 2, antes de partir o destilar una nota `core`/
`stable` obesa, o al barrer el backlog>.
```

No se tocan las menciones a `Paul` ni `kb-demo` (BACKLOG:Alta «exo genérico»,
depende de D2, fuera de esta campaña).

- [ ] **Step 3: Oráculo verde**

Run: `bash "$TMPDIR/oraculo-distill.sh"`
Expected: `[OK] distill repartido sin perder procedimiento (N líneas)` con
N ≤ 130; exit 0.

Run: `grep -c "" plugins/exo/skills/distill/*.md`
Expected: la suma de líneas de los cuatro ficheros ≥ 276 − (líneas de
resumen sustituidas) — nada se ha perdido, solo movido.

- [ ] **Step 4: Commit**

```bash
git add plugins/exo/skills/distill/SKILL.md plugins/exo/skills/distill/rotacion.md plugins/exo/skills/distill/chequeos.md plugins/exo/skills/distill/consolidacion.md
git commit -m "docs(distill): procedimiento en SKILL.md, detalle bajo demanda; budget sale con 3, no con 1 (H22)"
```

---

### Task 9: gate de versiones (H16)

**Lane:** mecánica tras decisión. **Bloqueada por D3.** Código escrito para
**D3 = a**. **Depende de A:** no. **Oráculo:** `bash scripts/test-versiones.sh`
verde; `bash scripts/test-versiones.sh v9.9.9` rojo.

**Evidencia (H16):** `.claude-plugin/marketplace.json:4`
`"metadata": { "version": "1.0.0", … }`; `marketplace.json:8` y
`plugins/exo/.claude-plugin/plugin.json:4` `"1.1.2"`; `engine/Cargo.toml:3`
`version = "0.1.0"`; único tag `v0.1.0`. Ni `ci.yml` ni `release.yml`
comparan ninguno (`grep -n "Cargo\|version" .github/workflows/release.yml`
solo encuentra el `description` del input).

**Files:**
- Create: `scripts/test-versiones.sh` (100755)
- Modify: `.claude-plugin/marketplace.json:4`
- Modify: `.github/workflows/ci.yml` (job `lint`, step tras shellcheck)
- Modify: `.github/workflows/release.yml` (job `publish`, step antes de
  «Publicar la release»)

**Interfaces:**
- Consumes: `jq`; `engine/Cargo.toml`; los dos JSON.
- Produces: `scripts/test-versiones.sh [TAG]` — exit 0 con
  `[OK] engine X · plugin Y`; exit 1 con `[FAIL] …`.

- [ ] **Step 1: El gate, rojo**

`scripts/test-versiones.sh`:

```bash
#!/usr/bin/env bash
# Gate: los números de versión publicados no se contradicen entre sí.
#
# exo publica DOS artefactos con ciclo propio (decisión D3 de la campaña B):
#   - el engine: `engine/Cargo.toml` [package] version == tag de release `vX.Y.Z`
#     (lo que imprime `exo --version`);
#   - el plugin: `plugins/exo/.claude-plugin/plugin.json` .version ==
#     `.claude-plugin/marketplace.json` .plugins[0].version (lo que Claude Code
#     usa para detectar actualizaciones).
# Sin gate, el mismo número vivía en dos ficheros a mano y la metadata del
# marketplace llegó a decir 1.0.0 con el plugin en 1.1.2.
#
# Uso: scripts/test-versiones.sh            # coherencia del árbol
#      scripts/test-versiones.sh v0.2.0     # además, el tag casa con Cargo.toml
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

fallos=0
plugin="$(jq -r '.version // empty' plugins/exo/.claude-plugin/plugin.json)"
market="$(jq -r '.plugins[] | select(.name == "exo") | .version // empty' .claude-plugin/marketplace.json)"
engine="$(sed -n 's/^version = "\(.*\)"$/\1/p' engine/Cargo.toml | head -n 1)"

if [ -z "$plugin" ] || [ "$plugin" != "$market" ]; then
  echo "[FAIL] plugin.json dice '$plugin' y marketplace.json dice '$market'" >&2
  fallos=1
fi
if jq -e '.metadata.version' .claude-plugin/marketplace.json >/dev/null; then
  echo "[FAIL] marketplace.json lleva metadata.version: un tercer número sin dueño. Quítalo." >&2
  fallos=1
fi
if [ -z "$engine" ]; then
  echo "[FAIL] no leo la versión de engine/Cargo.toml" >&2
  fallos=1
fi
if [ "$#" -ge 1 ] && [ "$1" != "v$engine" ]; then
  echo "[FAIL] el tag '$1' no casa con engine/Cargo.toml ($engine): el binario diría otra versión" >&2
  fallos=1
fi

[ "$fallos" -eq 0 ] && echo "[OK] engine $engine · plugin $plugin"
exit "$fallos"
```

Run: `chmod +x scripts/test-versiones.sh && git add scripts/test-versiones.sh && bash scripts/test-versiones.sh; echo "exit=$?"`
Expected: `[FAIL] marketplace.json lleva metadata.version: un tercer número sin dueño. Quítalo.` / `exit=1`.

- [ ] **Step 2: Implementación**

`.claude-plugin/marketplace.json`, línea 4:

```json
  "metadata": { "pluginRoot": "./plugins" },
```

Step en `ci.yml`, job `lint`, tras el de shellcheck:

```yaml
      - name: Versiones coherentes (plugin.json == marketplace.json)
        if: always()
        run: bash scripts/test-versiones.sh
```

Step en `release.yml`, job `publish`, justo antes de `- name: Publicar la release`:

```yaml
      # Un tag que no casa con Cargo.toml publicaría binarios cuyo
      # `exo --version` dice otra cosa.
      - name: El tag casa con engine/Cargo.toml
        shell: bash
        env:
          TAG: ${{ github.event.inputs.tag || github.ref_name }}
        run: bash scripts/test-versiones.sh "$TAG"
```

- [ ] **Step 3: Verde, y rojo con un tag falso**

Run: `bash scripts/test-versiones.sh; echo "exit=$?"`
Expected: `[OK] engine 0.1.0 · plugin 1.1.2` / `exit=0`.

Run: `bash scripts/test-versiones.sh v0.1.0; echo "exit=$?"; bash scripts/test-versiones.sh v9.9.9; echo "exit=$?"`
Expected: `exit=0`; luego `[FAIL] el tag 'v9.9.9' no casa con engine/Cargo.toml (0.1.0)…` / `exit=1`.

Run: `jq . .claude-plugin/marketplace.json >/dev/null && bash scripts/test-exec-bit.sh`
Expected: sin error de jq; `[OK] …` (el script nuevo está en `scripts/`, fuera del
gate de exec-bit, pero Task 7 lo pasa por shellcheck si ya está mergeada).

Documentar el esquema: en `docs/instalacion.md`, al final de la sección 5,
añadir:

```markdown
**Versiones.** El engine y el plugin versionan por separado: `exo --version`
es la del binario (= tag de la release); el plugin lleva la suya en
`plugins/exo/.claude-plugin/plugin.json`. `scripts/test-versiones.sh` impide
que los ficheros se contradigan.
```

- [ ] **Step 4: Commit**

```bash
git add scripts/test-versiones.sh .claude-plugin/marketplace.json .github/workflows/ci.yml .github/workflows/release.yml docs/instalacion.md
git commit -m "ci: gate de versiones — plugin.json == marketplace.json y tag == Cargo.toml (H16)"
```

Si **D3 = b** (lockstep): el script compara `plugin == market == engine` y
sigue exigiendo tag == `v$engine`; el bump a un número común es un commit
aparte que decide Paul. Si **D3 = c**: esta tarea se reduce al párrafo de
`docs/instalacion.md`.

---

### Task 10: README de producto con ejemplo de extremo a extremo (H6)

**Lane:** diseño (framing). **Bloqueada por D2.** **Depende de A:** sí (la
KB nueva y el `--db` de `exo init`). **Oráculo:** el script del Step 1 (sin
hitos internos ni cifras frágiles en README e instalación) + el ejemplo
ejecutado en un `HOME` aislado en el Step 3.

**Evidencia (H6):** `README.md:1-6` abre con «Tres capas thin/engine/thick» y
pasa a Instalar sin decir qué problema resuelve; `README.md:33-56` es un
bloque de estado con jerga (M0, M1a, M2 E1, M4 E2, M6, 1A/1B/1C, B1, G5b,
G4d) fechado «Estado (2026-09-02)» con contenido del 09-11 (`v0.1.0`). Cifras
frágiles: `README.md:46` «434 tests verdes en 44 binarios» y
`docs/instalacion.md:134` «434 tests en 44 binarios».

El ejemplo se ejecutó el 2026-09-13 con `HOME` y `HF_HOME` aislados sobre el
binario de `3c1918f`: `exo init` 45 s (modelo ya en caché), `write new` crea
`learnings/Por qué SQLite.md`, `exo search` **antes** de `exo index` no
devuelve nada (write no indexa), después devuelve
`mi-kb/learnings/por-que-sqlite` primero; `exo recall --query` la sirve
primera.

**Files:**
- Modify: `README.md` (reescritura)
- Modify: `docs/instalacion.md:134`

**Interfaces:**
- Consumes: la tabla de hooks ya actualizada por Task 5.
- Produces: README sin estado de campaña; puntero a `docs/backlog.md` como
  fuente de «qué falta».

- [ ] **Step 1: El oráculo, rojo**

`$TMPDIR/oraculo-readme.sh`:

```bash
#!/usr/bin/env bash
set -uo pipefail
cd /home/paul/Documentos/proyectos/exo || exit 1
fallos=0
if grep -nE '\b(M[0-9]+[ab]?|G[0-9][a-d]?|E[12]|1[ABC]|B1|D[0-9])\b' README.md; then
  echo "[FAIL] ids de hito interno en README.md"; fallos=1; fi
if grep -nE '[0-9]+ tests|[0-9]+ binarios|Estado \(20' README.md docs/instalacion.md; then
  echo "[FAIL] cifras frágiles de estado"; fallos=1; fi
for s in '## Qué problema resuelve' '## Ejemplo' 'exo init --kb' 'exo write new' 'exo index' 'exo search'; do
  grep -qF -- "$s" README.md || { echo "[FAIL] falta en README: $s"; fallos=1; }
done
[ "$fallos" -eq 0 ] && echo "[OK] README de producto"
exit "$fallos"
```

Run: `bash "$TMPDIR/oraculo-readme.sh"`
Expected: líneas `README.md:33:…M0, M1a…` etc., `[FAIL] ids de hito`,
`[FAIL] cifras frágiles`, `[FAIL] falta en README: ## Qué problema resuelve`…; exit 1.

- [ ] **Step 2: Reescritura**

`README.md` pasa a tener, en este orden: título + párrafo de qué es; «Para
quién es hoy» (frase literal de la opción elegida en D2); «Qué problema
resuelve»; «Instalar» (actual `:8-15`, sin cambios); «Ejemplo de extremo a
extremo»; «Arquitectura» (actual `:58-89`, mermaid sin cambios); «Idioma»
(actual `:91-97`); «Capa thin: el plugin `exo`» (actual `:99-130` con la tabla
de Task 5); «Documentación»; «Atribución» (actual `:132-140`). Se elimina el
bloque de enlaces + estado de `:17-56`.

Texto nuevo (lo que no está en las secciones que se conservan):

````markdown
# exo

Memoria persistente para agentes de código. exo guarda lo que decides y
aprendes en una KB de notas markdown versionada con git, la indexa en local
(texto completo + embeddings, SQLite en un solo fichero) y se la devuelve al
agente cuando la necesita: al empezar la sesión y en cada prompt.

**Para quién es hoy:** <frase de D2, literal>

## Qué problema resuelve

Un agente como Claude Code empieza cada sesión sin memoria: las decisiones de
ayer, los errores ya diagnosticados y las convenciones del proyecto hay que
volver a contárselas. exo lo resuelve con tres piezas que se usan por
separado:

- **La KB** — notas markdown con frontmatter (`tier: core | stable | log`), en
  tu disco y en git. Se leen sin exo.
- **El engine** — `exo`, un binario sin runtime: indexa la KB y sirve búsqueda
  y recall (`exo --help`).
- **El plugin de Claude Code** — `plugins/exo/`: hooks que inyectan el recall
  al arrancar y en cada prompt, y skills de proceso (plan, tdd, debug,
  document…) que escriben en la KB.

## Ejemplo de extremo a extremo

```bash
# 1. KB nueva desde la plantilla, versionada con git e indexada.
#    La primera vez descarga el modelo de embeddings (~0,6 GB).
exo init --kb ~/mi-kb --name mi-kb

# 2. Una decisión, como nota.
printf 'Usamos SQLite con FTS5: el índice cabe en un fichero y no hay servidor que mantener.\n' > nota.md
exo write new --dir learnings --title "Por qué SQLite" --from nota.md

# 3. `write` no indexa: lo hace `exo index` (con el plugin, lo refrescan sus hooks).
exo index

# 4. Recupérala.
exo search "servidor que mantener"
exo recall --query "qué base de datos usamos" --limit 3
```

`exo search` devuelve `mi-kb/learnings/por-que-sqlite` como primer resultado,
y `exo recall` la sirve primera con su primer párrafo: es el bloque que el
plugin inyecta al agente.

## Documentación

- Cómo funciona, derivado del código: [`docs/arquitectura.md`](docs/arquitectura.md)
- Instalación, compilar desde fuente y tests: [`docs/instalacion.md`](docs/instalacion.md)
- **Qué falta y qué está roto: [`docs/backlog.md`](docs/backlog.md)** — léelo
  antes de asumir que algo está terminado.
- Historial de diseño (specs, planes, verdicts, consultorías):
  `docs/superpowers/` y `evals/`. Son instantáneas fechadas, no documentación
  viva.
````

En «Capa thin», quitar la frase «Sustituye a `superpowers` y a
`paul-profile:orchestrate-personal` en el uso diario.» solo si **D2 = b**
(con D2 = a es contexto honesto y se queda).

`docs/instalacion.md:134`: `cargo test            # suite completa: 434 tests en 44 binarios`
→ `cargo test            # suite completa`.

Verificar que `docs/instalacion.md` §4 documenta el `--db` de `exo init` que
añadió A (p.ej. para una segunda KB en la misma máquina). Si A no lo hizo,
añadir tras el bloque de `exo init --from-basic-memory`:

```markdown
Una segunda KB en la misma máquina necesita su propio índice:
`exo init --kb ~/otra-kb --name otra-kb --db ~/.exo/otra-kb.db`.
```

(adaptando la sintaxis exacta a la que muestre `exo init --help` tras A).

- [ ] **Step 3: El ejemplo, ejecutado de verdad y aislado**

No toca `~/.exo` ni la caché del modelo del usuario:

```bash
cd engine && cargo build --release && cd ..
E="$PWD/engine/target/release/exo"
T="$(mktemp -d)"; mkdir -p "$T/home"
(
  export HOME="$T/home" HF_HOME="${HF_HOME:-/home/paul/.cache/huggingface}"
  unset EXO_DB EXO_KB EXO_CONFIG
  cd "$T"
  git config --global user.email ejemplo@exo.local && git config --global user.name ejemplo
  "$E" init --kb ~/mi-kb --name mi-kb &&
  printf 'Usamos SQLite con FTS5: el índice cabe en un fichero y no hay servidor que mantener.\n' > nota.md &&
  "$E" write new --dir learnings --title "Por qué SQLite" --from nota.md &&
  "$E" index &&
  "$E" search "servidor que mantener" | head -1 &&
  "$E" recall --query "qué base de datos usamos" --limit 3 | sed -n 2p
)
rm -rf "$T"
```

Expected: `exo init` sale 0 con `config escrita en …/home/.exo/config.toml`;
`index: indexadas=1 saltadas=11 borradas=0`; la línea de search empieza por
`mi-kb/learnings/por-que-sqlite`; la de recall contiene `Por qué SQLite`.
(`git config --global` escribe en el `HOME` aislado: sin identidad, `exo init`
avisa y deja la KB sin commit, medido.)

Run: `bash "$TMPDIR/oraculo-readme.sh"`
Expected: `[OK] README de producto`.

- [ ] **Step 4: Commit**

```bash
git add README.md docs/instalacion.md
git commit -m "docs(readme): que problema resuelve y ejemplo de extremo a extremo; fuera el estado de campaña (H6)"
```

---

### Task 11: `reports/` (H26)

**Lane:** mecánica tras decisión. **Bloqueada por D4.** Escrita para
**D4 = b**; deltas para (a) y (c) al final. **Depende de A:** no (pero
toca `engine/src/main.rs:14`: si corre en la rama engine, después de Task 3).
**Oráculo:** los greps del Step 2.

**Evidencia (H26):** `git ls-files reports` = 5 ficheros
(`m2-06-report.md`, `m2-07-impl-report.md`, `m2-07-report.md`,
`m2-08-report.md`, `m6-01-report.md`). Citas por ruta:
`engine/src/main.rs:14` (`reports/m2-07-impl-report.md`),
`docs/superpowers/consultas/2026-08-22-m6-06/consultor-m6-06.md:117`
(ídem), `evals/e1-read/verdict/gate-m2-07-impl.md:69` (ídem),
`evals/e1-read/verdict/gate-m2-07-spec.md:6,59` (`reports/m2-07-report.md`).

**Files:**
- Rename: `reports/*.md` → `evals/e1-read/reports/*.md`
- Modify: `engine/src/main.rs:14`

- [ ] **Step 1: Mover y actualizar la cita viva**

```bash
mkdir -p evals/e1-read/reports
git mv reports/m2-06-report.md reports/m2-07-impl-report.md reports/m2-07-report.md reports/m2-08-report.md reports/m6-01-report.md evals/e1-read/reports/
```

`engine/src/main.rs:14`: `` `reports/m2-07-impl-report.md` `` →
`` `evals/e1-read/reports/m2-07-impl-report.md` ``.

- [ ] **Step 2: Verificar cada cita**

Run: `git ls-files reports | wc -l; git ls-files evals/e1-read/reports | wc -l`
Expected: `0`, `5`.

Run: `git grep -n "reports/m[0-9]" -- engine plugins scripts README.md docs/arquitectura.md docs/instalacion.md docs/backlog.md`
Expected: solo `engine/src/main.rs:14: … evals/e1-read/reports/m2-07-impl-report.md …`.

Run: `git grep -n "reports/m[0-9]" -- docs/superpowers evals | grep -v "evals/e1-read/reports/"`
Expected: exactamente las 4 líneas históricas (`consultor-m6-06.md:117`,
`gate-m2-07-impl.md:69`, `gate-m2-07-spec.md:6`, `gate-m2-07-spec.md:59`),
intactas por decisión D4 = b.

Run: `git diff --cached -M --stat | tail -1`
Expected: 5 renames sin inserciones ni borrados de contenido
(`5 files changed, 0 insertions(+), 0 deletions(-)` en la parte de renames).

Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
Expected: limpio (solo cambió un comentario).

- [ ] **Step 3: Commit**

```bash
git add engine/src/main.rs
git commit -m "chore: reports/ a evals/e1-read/reports/, junto a los verdicts de esas campañas (H26)"
```

Delta **D4 = a**: además, en las 4 líneas históricas sustituir
`reports/m2-07` por `evals/e1-read/reports/m2-07`, y el segundo grep del
Step 2 debe salir vacío. Delta **D4 = c**: no se mueve nada; Task 13 cierra
el item escribiendo por qué se queda.

---

### Task 12: `tier` en los docs del repo (H18)

**Lane:** mecánica tras decisión. **Bloqueada por D5.** **Depende de A:** no.

**Evidencia (H18):** `git ls-files 'docs/*.md' | wc -l` = 75; 72 con fecha en
el nombre; ninguno empieza por `---`. Vivos: `README.md`,
`docs/arquitectura.md`, `docs/instalacion.md`, `docs/backlog.md`.

**D5 = b (recomendada):**

- [ ] **Step 1:** En `docs/arquitectura.md`, al final del blockquote de
  cabecera (tras la línea que empieza por `> Este documento describe el
  sistema`… y su párrafo), añadir:

```markdown
>
> **Qué documentación es viva.** Cuatro ficheros deben ser verdad hoy:
> `README.md`, `docs/arquitectura.md`, `docs/instalacion.md` y
> `docs/backlog.md`. Todo lo que lleva fecha en el nombre y todo
> `docs/superpowers/` son instantáneas (`tier: log` por convención de ruta):
> no se actualizan, se citan con su fecha.
```

- [ ] **Step 2 (oráculo):**
  `grep -n "Qué documentación es viva" docs/arquitectura.md` → 1 línea.
- [ ] **Step 3:** `git add docs/arquitectura.md && git commit -m "docs(arquitectura): que documentacion es viva, por convencion de ruta (H18)"`

**D5 = a o c:**

- [ ] **Step 1:** guardar como `$TMPDIR/tier-docs.py` y correr desde la raíz:

```python
import subprocess, sys
vivos = {"README.md", "docs/arquitectura.md", "docs/instalacion.md", "docs/backlog.md"}
incluir_readme = sys.argv[1] == "a"   # D5=a toca README; D5=c no
rutas = subprocess.run(["git", "ls-files", "docs/*.md"], capture_output=True, text=True, check=True).stdout.split()
if incluir_readme:
    rutas.append("README.md")
for r in rutas:
    s = open(r, encoding="utf-8").read()
    if s.startswith("---\n"):
        sys.exit(f"{r} ya tiene frontmatter: revisar a mano")
    tier = "core" if r in vivos else "log"
    open(r, "w", encoding="utf-8").write(f"---\ntier: {tier}\n---\n\n{s}")
print(len(rutas))
```

Run: `python3 "$TMPDIR/tier-docs.py" a` (o `c`)
Expected: `76` (o `75`).

- [ ] **Step 2 (oráculo):** `git ls-files 'docs/*.md' | xargs grep -L '^tier: ' | wc -l` → `0`;
  `grep -l '^tier: core' docs/*.md` → los 3 vivos de `docs/`.
- [ ] **Step 3:** D5 = a: `git add -u -- docs README.md`; D5 = c:
  `git add -u -- docs`. Luego
  `git commit -m "docs: tier en el frontmatter de los docs del repo (H18)"`.

---

### Task 13: backlog sincronizado con CI y con esta campaña (H13)

**Lane:** mecánica. **Depende de A:** sí, y va **la última** de la campaña
(y después de C si C mergeó). **Oráculo:** los greps del Step 2.

**Evidencia (H13):**
- `docs/backlog.md:510` «Los scripts `test-*.sh` de `plugins/exo/scripts/`
  no entran en CI» — cerrado por `d8aa3b6` (2026-09-12): `scripts/test-plugin.sh`
  y job `plugin-tests` en 3 SO (`ci.yml:123-136`). Siguen vivas sus dos
  sub-propuestas (validador de rutas personales, validación de `hooks.json`
  contra schema).
- `docs/backlog.md:602` «`test-contrato-engine.sh` depende del índice y la KB
  reales» — cerrado por `d8aa3b6`: `scripts/test-contrato-ci.sh`
  (`ci.yml:118-121`).
- `docs/backlog.md:614` «Un rojo del job `test` no se puede diagnosticar» —
  **a medias** por `50aee95` (2026-09-10): `engine/scripts/test-hermetico.sh`
  ya emite `^test .* ... FAILED$` y el bloque `failures:`. Sigue abierto: un
  error de **compilación** no casa ningún patrón, y no hay `upload-artifact`.
- `docs/backlog.md:1186-1194` (cerrado) «Modo mudo de `busca_hybrid`… los
  avisos siempre se ven» — era falso para `exo recall` (H2), que A arregla.

**Files:**
- Modify: `docs/backlog.md`

- [ ] **Step 1: Ediciones**

1. Cabecera: nueva `> Última revisión: **<fecha de ejecución>**` que resuma
   esta pasada (campaña B: H6, H8, H9, H11, H12, H13, H15, H16, H18, H20,
   H21, H22, H26; lo que se cerró y lo que se abrió), y la anterior pasa a
   «Anterior».
2. Tabla `## Estado`, fila **Tests**: quitar la cita a `README.md:46` (ya no
   existe esa cifra) y dejar el recuento con su fecha y comando
   (`cargo test --release` en `engine/`, contado ese día).
3. Mover a `## Cerrado con evidencia` los items de `:510` (dejando en Media un
   item nuevo y corto con sus dos sub-propuestas vivas) y `:602`, citando
   `d8aa3b6` y las líneas de `ci.yml`.
4. `:614`: marcar «**a medias** (`50aee95`)» con lo que sigue abierto.
5. Item cerrado del modo mudo (`:1186`): añadir la matización «falso para
   `exo recall` hasta `<commit de A>`» (si A ya lo anotó, no duplicar).
6. Cerrar con evidencia lo que esta campaña cerró, cada uno con su commit:
   - Alta `:137` «La documentación de referencia contradice…», acción (b)
     versiones → según D3 (Task 9).
   - Media `:469` «Los documentos del repo no llevan `tier`» → según D5 (Task 12).
   - Baja `:810` «Decisión abierta: proceso frente a producto», acción
     «escribir en el README para quién es exo» → Task 10 (D2).
   - Baja `:846` «Idioma mezclado sin criterio único» → solo la parte de la
     superficie CLI, según D1 (Tasks 2-3); la de identificadores de código
     sigue abierta.
   - Baja `:859` «El relato de campaña en los comentarios… `main.rs`» → a
     medias: los doc-comments que clap renderiza (Task 3); los `//` internos
     y `buscador.rs` siguen.
   - Baja `:886` «Nombres y ubicaciones» → la parte de `reports/` según D4
     (Task 11); `docs/superpowers/` sigue.
   - Entradas nuevas en `## Cerrado con evidencia`: gate de shellcheck
     (Task 7) y exec-bit ampliado (Task 6), con su falsación medida.
7. Items nuevos (Media o Baja, con evidencia):
   - `exo search` sin resultados no imprime nada y sale 0 (medido en Task 10).
   - `trinquete::sellos_escapados_de_tier` lee el tier del disco también en
     `--staged` (Task 1): posible mezcla de revisiones; verificar contra
     `internal/ratchet` de kbx antes de tocarlo.
   - Cualquier aviso de shellcheck marcado «bug real, ver backlog» en Task 7.

- [ ] **Step 2: Oráculo**

Run: `grep -n "no entran en CI\.\*\*" docs/backlog.md | head -1`
Expected: la única coincidencia está bajo `## Cerrado con evidencia`
(comprobar con `awk '/^## Cerrado con evidencia/{c=1} /no entran en CI/{print c":"NR}' docs/backlog.md` → todas con prefijo `1:`).

Run: `grep -n "README.md:46" docs/backlog.md`
Expected: vacío.

Run: `grep -c "campaña B" docs/backlog.md`
Expected: ≥ 1.

- [ ] **Step 3: Commit**

```bash
git add docs/backlog.md
git commit -m "docs(backlog): sincronizado con CI (d8aa3b6, 50aee95) y con la campaña B (H13)"
```

---

## Checklist externo para Paul (H25) — no son tareas de la fábrica

Verificado el 2026-09-13 en solo lectura (`git ls-remote`, `gh api`): las tres
ramas remotas están mergeadas en `main` (`git merge-base --is-ancestor` = sí),
`delete_branch_on_merge` es `false`, la descripción es «Motor de retrieval
sobre una KB markdown», sin topics.

```bash
# 1. Borrar las ramas remotas ya mergeadas
git -C /home/paul/Documentos/proyectos/exo push origin --delete g4a-targets g5a-ci revision-critica-backlog
git -C /home/paul/Documentos/proyectos/exo fetch --prune origin

# 2. Que GitHub borre la rama al mergear un PR
gh repo edit pguerrerolinares/exo --delete-branch-on-merge

# 3. About coherente con el README (descripción según D2) y topics
gh repo edit pguerrerolinares/exo \
  --description "Memoria persistente para agentes de código: KB markdown + engine Rust (SQLite FTS5 + embeddings) + plugin de Claude Code" \
  --add-topic claude-code --add-topic ai-agents --add-topic memory --add-topic rust --add-topic sqlite

# Verificación
git -C /home/paul/Documentos/proyectos/exo ls-remote --heads origin        # solo main
gh api repos/pguerrerolinares/exo --jq '{description, delete_branch_on_merge, topics}'
```

## Cierre de campaña (review final de rama)

- [ ] `cd engine && ./scripts/test-hermetico.sh` → `test-hermetico: OK`
- [ ] `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings` → limpio
- [ ] `./scripts/test-plugin.sh` → `test-plugin: OK — 11/11 …`
- [ ] `bash scripts/test-exec-bit.sh && SHELLCHECK="$TMPDIR/shellcheck-v0.11.0/shellcheck" bash scripts/test-shellcheck.sh && bash scripts/test-versiones.sh` → tres `OK`
- [ ] `./scripts/test-contrato-ci.sh` → verde (no se tocó el contrato engine↔recall-inject, y debe seguir así)
- [ ] `git diff --stat 3c1918f -- engine/src` (sobre la rama B rebasada tras A, restando lo de A) → solo `main.rs` (doc-comments, `value_name`, `about`, comentario de `reports/`) y `trinquete.rs` (H15); ninguna línea de lógica en `main.rs`: `git diff <base-B> -- engine/src/main.rs | grep -E '^[-+][^-+]' | grep -vE '^[-+]\s*(///|//|#\[arg|#\[command|\)\]|long =|alias =|value_name =|default_value_t =|name = |version,|about = |$)'` → vacío
- [ ] PENDIENTE-PAUL residuales anotados en el ledger (decisiones D1-D5 no contestadas y las tareas que quedaron bloqueadas).
