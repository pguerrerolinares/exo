# G4c — `exo ratchet` + cutover Implementation Plan

**For agentic workers:** la skill de ejecución es `exo:orchestrate`. Los pasos
llevan checkbox (`- [ ]`) para tracking. Cada tarea es un ciclo cerrado (test
que falla → verlo fallar → implementación mínima → verlo pasar → commit) y
merece el gate de un reviewer fresco.

**Goal:** que `exo ratchet` exista en el engine Rust con la semántica de
`kbx ratchet` de `fe46443` —trinquete solo-baja, abstención, ancla de
activación, emparejamiento de renames, guarda de aire y `--seal` atómico— y que
los consumidores que hoy invocan el binario `kbx` pasen a invocar verbos `exo`
en todo lo que ya tiene destino.

**Architecture:** el port es de lenguaje, no de datos. El fichero de sello
sigue siendo `.kbx-ratchet.json` con el mismo esquema, y los campos de
frontmatter siguen siendo `kbx_budget_max` / `kbx_orphan_ok`: son formato de
dato ya escrito en la KB viva, y renombrarlos sería una ruptura silenciosa (ver
A6). Se añade **un módulo** (`trinquete.rs`), **cuatro funciones** a `gitx.rs`
y **un subcomando** a `main.rs`. `trinquete.rs` cuelga de `presupuesto.rs`, que
ya trae construida y testeada toda la aritmética de aire (`tiene_aire`,
`techo_minimo`, `objetivo_poda`) y la resolución tier→presupuesto
(`Presupuestos::para_tier`): **este plan no escribe ni una fórmula nueva de
aire**, solo el gate que las consume. El orden va de dentro afuera: pre-registro
→ verbos de git → sellos y su IO → abstención → las tres familias de hallazgo →
`--staged` → `--seal` → CLI → cutover.

**Tech Stack:** Rust edition 2024, MSRV 1.95 · `serde`/`serde_json` · `anyhow` ·
`clap` 4.6.2 derive · `tempfile` 3.14 (dev). **Sin dependencias nuevas** —
`BTreeMap` y `std::process::Command` son de `std`.

## Global Constraints

Se heredan enteras de G4a y G4b y no se repiten salvo donde G4c las ejerce o
las cambia:

- **El crate vive en `engine/`.** No hay workspace de Cargo. Todo comando de
  cargo se ejecuta con cwd `engine/`.
- **Fuente del port: `fe46443`.** Es el único árbol con la guarda de aire
  (`6332c85`) y con el fix del sello huérfano (`0ae126d`). El checkout local de
  kbx en W11 (`ee2b27c`/`f0d0564`) **no tiene ninguno de los dos**: un port
  hecho contra él nacería con un agujero ya corregido aguas arriba. Se lee con
  `git -C C:\proyectos\homework\kbx show fe46443:<path>`. **Nada de este plan
  toca el repo kbx.**
- **Hallazgos ⇒ exit 3, errores ⇒ exit 1.** Contrato de exo desde G4b. kbx usa
  `1` = hallazgos y `2` = error; es una divergencia deliberada y está declarada
  en el pre-registro (`2026-09-09-g4c-preregistro-ratchet.md`, divergencia 1).
- **`SCHEMA_VERSION` sigue en 2 y no se toca.** `ratchet` es un `command`
  nuevo: aditivo, no breaking.
- **Claves JSON en inglés, identificadores Rust en castellano** (D7/D8), con
  `#[serde(rename)]` en cada campo que difiera. Comentarios en castellano,
  explicando el porqué.
- **Avisos y progreso a stderr, resultado primario a stdout.** Con `--json`,
  stdout lleva el envelope y nada más.
- **Errores con `anyhow`**, `.context(...)` accionable en cada IO/parse
  falible. No se añade `thiserror`.
- **Clippy es gate duro** desde el primer commit:
  `cargo clippy --all-targets --locked -- -D warnings` limpio, y
  `cargo fmt --check`. El CI corre además `cargo check` con MSRV 1.95 y la
  suite en ubuntu/windows/macos, más el gate hermético sin
  `~/.exo/config.toml`.
- **Aritmética entera, nunca floats**, en todo lo que gatee — ya resuelto en
  `presupuesto.rs` con `checked_mul` y razonado allí: saturar colapsaría los dos
  lados a `i64::MAX` y la comparación diría *que hay aire*, degradación hacia
  verde en la única función que existe para gatear.
- **Baseline verde antes de empezar**: `cargo test --locked` en `main` al
  2026-09-09 da **329 passed, 0 failed, 2 ignored** en 36 binarios. Cualquier
  regresión sobre esa cifra es un fallo de este plan.

## Adjudicaciones — las decisiones que este plan cierra

### A1 — "los nueve invariantes de la spec" no existen

El plan de G4a (`2026-09-02-g4a-plomeria-y-targets.md:1614-1616`) mete en el
alcance de G4c *"los nueve invariantes de la spec"*. **Esa lista no existe en
ningún documento del repo.** Lo que hay es *"Invariantes portables — tests
obligatorios (V13/H7)"* en `2026-08-26-exo-generico-design.md:578-606`, con
**siete** ítems, y la propia spec los llama *"los siete invariantes de arriba"*
(línea 705-706). De los siete, **solo el ítem 7 es del ratchet**, y agrupa
cuatro sub-invariantes separados por `·`:

> *"**Ratchet:** sello borrado = subir a infinito · sello huérfano no puede
> lavar una declaración (`0ae126d`) · sello corrupto en HEAD es error, no
> abstención (`load.go`) · aritmética **entera** de aire
> (`ceiling*100 >= size*115`, `check.go`)."*
> — `2026-08-26-exo-generico-design.md:599-602`

**Adjudicación:** no se inventa una lista de nueve. El port debe satisfacer los
**cuatro sub-invariantes literales de arriba**, y además los invariantes que
solo están escritos en el código Go y no en prosa (abstención, ancla,
emparejamiento de renames, atomicidad de `--seal`), que este plan transcribe
como tareas. La cifra "nueve" del plan de G4a se declara **lapsus documental**;
se anota en el backlog (Task 13) para que nadie vuelva a buscarla.

> Esto importa más de lo que parece: buscar nueve invariantes donde hay siete
> es exactamente cómo se inventa código. El agente que fuera a por la lista
> completa habría rellenado los dos que faltan.

### A2 — la doctrina de shallow/ancla/renames vive en el código, no en prosa

Verificado el 2026-09-09: **no hay ninguna explicación escrita** de qué es la
"abstención por shallow-clone", el "ancla de activación" ni la "absolución de
renames" en ningún documento del repo — solo se nombran en dos frases-lista
idénticas. Lo que sí existe es la implementación en `fe46443`, transcrita en las
Tasks 4 y 7 de este plan.

**Adjudicación:** el código Go es la fuente normativa para estos tres
mecanismos. Este plan los define por primera vez en prosa, en la tarea que los
implementa, y esa prosa es lo que se cita en el doc-comment del módulo. **No se
implementa nada por inferencia**: cada rama del port cita el fichero y la
función Go de la que sale.

### A3 — `--no-renames`: el port lo pasa; el Go no

El plan de G4a inventaría como trampa *"`--no-renames` explícito en
`git diff --cached`: hoy kbx depende de la config `diff.renames` del entorno"*.
**Verificado en `fe46443`: kbx NO pasa `--no-renames`.** Invoca
`git diff --cached --name-only --diff-filter=ACMR` a secas
(`internal/ratchet/staged.go`), así que con `diff.renames=true` un rename
aparece como una sola entrada `R` (el destino) y con `false` aparece como `A` +
`D`. Qué ficheros se recolectan depende de la máquina.

**Adjudicación:** el port **sí** pasa `--no-renames`. Es determinismo entre
máquinas a cambio de una divergencia declarada con kbx, y el emparejamiento de
renames del trinquete **no depende de la detección de renames de git** — es
heurístico sobre los sellos (Task 7), así que quitarle a git la detección no le
quita información a nadie. La divergencia está declarada en el pre-registro
(divergencia 4) y el gate se corre con `-c diff.renames=false` en las dos
mitades para que el criterio no dependa del entorno.

### A4 — `BTreeMap`, no `HashMap`: el determinismo es del tipo, no del test

kbx empareja renames recorriendo un `map` de Go, cuyo orden de iteración no está
garantizado; hay un test (`TestRenamePairingIsDeterministic`) que corre el
emparejamiento **20 veces** para detectar la no-determinación.

**Adjudicación:** `Sellos` es un `BTreeMap<String, i64>`. El orden es una
propiedad del tipo y no hace falta un test de 20 iteraciones para comprobar lo
que el tipo garantiza. Se porta igualmente un test de emparejamiento, pero
falsando **el resultado** (qué sello absuelve a cuál), no la estabilidad. Si en
el gate de paridad los dos binarios divergen en un caso ambiguo, el sospechoso
es kbx.

### A5 — el módulo es `trinquete`, el verbo es `ratchet`, el fichero es `.kbx-ratchet.json`

Tres nombres, tres idiomas, a propósito y por tres razones distintas:

- **`engine/src/trinquete.rs`** — identificadores Rust en castellano (D7/D8),
  como todo el engine.
- **`exo ratchet`** — el verbo del CLI va en inglés, como `budget`, `lint`,
  `targets`, `search`. Es superficie pública y sigue al resto.
- **`.kbx-ratchet.json`** — **no se renombra**. Es el fichero que la KB tiene
  hoy commiteado con 11 entradas; renombrarlo perdería el ancla y con ella todo
  el histórico del trinquete, que es precisamente lo que el trinquete existe
  para conservar. La constante se llama `FICHERO_SELLO` y su valor es
  `".kbx-ratchet.json"`, con el porqué en un comentario.

### A6 — `kbx_budget_max` y `kbx_orphan_ok` tampoco se renombran

Mismo argumento, verificado en el censo del cutover: son campos de frontmatter
escritos en notas vivas de la KB (11 con techo sellado). `frontmatter.rs:12-13`
ya lo declara. El cutover **no toca ni un campo de dato**: migra invocaciones de
binario, nada más. Cualquier cambio aquí es una ruptura de formato disfrazada de
limpieza de nombres.

### A7 — el cutover es PARCIAL, y se declara

Censo verificado el 2026-09-09; los cinco conteos del plan de G4a (5/11/3/1/1)
**coinciden exactamente** con la realidad, sin drift. Pero el plan de G4a dice
que kbx tiene 3 verbos que exo no (`rotate`, `stale`, `diff-since`): **son
cinco**, faltan `history` y `ratchet`. Tras este plan serán cuatro.

| Consumidor | Verbos que usa | Migra hoy | Bloquea |
|---|---|---|---|
| `plugins/exo/scripts/kb-precommit.sh` | `ratchet --staged`, `budget` | **los dos** (tras este plan) | `rotate`, solo citado en el mensaje de remediación (línea 58), no ejecutado |
| `plugins/exo/skills/distill/SKILL.md` | `rotate`, `budget`, `ratchet`, `doctor`, `stale` | `budget`, `ratchet`, `doctor`→`lint` | **`rotate` y `stale`**: sin destino |
| `plugins/exo/skills/document/SKILL.md` | `targets` (prosa) | sí | — |
| `plugins/exo/skills/document/routing.md` | `targets` (prosa) | sí | — |
| `plugins/exo/agents/executor.md` | `targets --json` | sí | — |

**Adjudicación:** G4c cierra la ventana H8 **para `kb-precommit.sh` y los tres
consumidores de `targets`**, y la deja abierta para `distill`, que seguirá
necesitando el binario `kbx` para `rotate` y `stale`. Eso **se escribe en el
propio SKILL.md** como estado declarado, no se disimula: un skill que invoca dos
binarios y lo dice es honesto; uno que finge haber migrado es una trampa para el
día que `kbx` no esté instalado.

### A8 — `git show HEAD:./<path>`, con el `./` explícito

Sin el `./`, git resuelve la ruta contra la raíz del repo y no contra el `-C`:
si la KB vive en un subdirectorio, los sellos de HEAD se leen **vacíos** y
cualquier subida pasa en verde. kbx tiene un test dedicado
(`TestLoadHEADResolvesSealRelativeToKBRootNotRepoRoot`) y se porta en la Task 4.
Aplica a las cuatro invocaciones: `HEAD:./<sello>`, `:./<sello>`,
`HEAD:./<nota>` y `:./<nota>`.

### A9 — las claves del sello son `String` con `/` literal

Solo se convierten a `PathBuf` en el punto de tocar disco. En Go,
`filepath.Join` acepta `/` en Windows y eso enmascara la cuestión; en Rust hay
que decidirlo, y la decisión es que la clave del sello es una cadena de
identidad —viaja a JSON y se compara con lo que git devuelve, que siempre usa
`/`— y no una ruta del sistema.

### A10 — el sello se escribe con el formato exacto de kbx

kbx no usa `json.Marshal` del mapa: serializa a mano (`writeSeals` en
`cmd/kbx/ratchet.go`) con claves ordenadas alfabéticamente, indentación de 2 y 4
espacios, valores enteros sin comillas y `\n` final.

**Adjudicación:** el port replica ese formato byte a byte. No es estética: el
sello está **commiteado en la KB**, y un `--seal` que reescriba el fichero
entero con otra indentación produce un diff de 13 líneas donde debería haber
una. El escapado no diverge en la práctica (ni Go ni `serde_json` escapan
non-ASCII, y las claves reales no llevan `<`, `>` ni `&`), pero el criterio del
gate compara el sello **parseado**, no sus bytes — declarado en el pre-registro,
divergencia 5.

## Correcciones a premisas de G4a

1. **"Los nueve invariantes"** → son siete, uno del ratchet. Ver A1.
2. **`--no-renames`** → kbx no lo pasa. Ver A3.
3. **"3 verbos que kbx tiene y exo no"** → son cinco (`rotate`, `stale`,
   `diff-since`, `history`, `ratchet`); tras este plan, cuatro. Ver A7.
4. **Las 2 rutas `/home/paul/…` de `test-git-c-bash.sh`** → verificadas, siguen
   ahí (líneas 74 y 75). No las toca este plan: son datos de un test de
   `git -C`, no del cutover. Se anotan en el backlog.

---

## Task 1: pre-registro del gate de paridad — **antes de una sola línea de Rust**

Ya escrito: `docs/superpowers/plans/2026-09-09-g4c-preregistro-ratchet.md`.

- [ ] Verificar que el pre-registro está commiteado **antes** del primer commit
      que toque `engine/src/`. Comando:
      `git log --oneline --name-only -- docs/superpowers/plans/2026-09-09-g4c-preregistro-ratchet.md`
      El commit del pre-registro debe ser anterior al primero que toque
      `engine/src/trinquete.rs`. Si no lo es, el documento deja de ser un
      pre-registro y hay que decirlo en su cabecera, como se hizo en G4a.

---

## Task 2: `gitx` — los cuatro verbos que el trinquete necesita

**Files**
- Modify: `engine/src/gitx.rs`
- Test: `engine/src/gitx.rs` (módulo `#[cfg(test)]` al final, donde ya viven los 7 tests actuales)

**Interfaces — Produces**
```rust
pub fn es_work_tree(dir: &Path) -> Result<bool>;
pub fn es_shallow(dir: &Path) -> Result<bool>;
pub fn muestra(dir: &Path, objeto: &str) -> Result<Option<String>>;
pub fn head_resuelve(dir: &Path) -> bool;
```

**Consumes:** nada nuevo. Reutiliza el idioma del módulo: `Command::new("git")`,
`.arg("-C").arg(dir)`, `.env("LC_ALL", "C").env("LANG", "C")`, `.output()`.

`muestra` devuelve `Ok(None)` cuando git sale con código ≠ 0 —que en este módulo
es señal semántica ("ese objeto no está en el árbol"), no fallo— y `Err` solo si
el proceso git no se pudo ni invocar. Es la única función de `gitx` que trata un
exit ≠ 0 como `Ok`, y el doc-comment tiene que decir por qué: el llamador
(`carga_head`) necesita distinguir "no hay sello en HEAD" de "no hay HEAD", y
esa distinción se hace encadenando `head_resuelve`, no mirando el stderr.

- [x] Escribir el test `es_shallow_detecta_un_clone_truncado`: `git init` un
      repo con un commit, clonarlo con `--depth 1` vía `file://`, y comprobar
      `es_shallow(clon) == Ok(true)` y `es_shallow(origen) == Ok(false)`. Si el
      clone falla en la máquina (no siempre se puede), el test hace `return`
      anotándolo — igual que el `t.Skipf` del test Go equivalente. Verlo fallar.
- [x] Implementar `es_shallow`: `rev-parse --is-shallow-repository`, `Ok(true)`
      si stdout trim es `"true"`. **Un error de git aquí es `Ok(true)`, no
      `Err`**: la abstención es el lado seguro, y esa es la semántica del Go
      (`err != nil || stdout == "true"` → abstiene). Comentar esa asimetría:
      es la única función del módulo que degrada hacia la abstención a
      propósito.
- [x] Verlo pasar. Commit.
- [x] Test `muestra_devuelve_none_cuando_el_objeto_no_esta`: repo con un
      commit, `muestra(repo, "HEAD:./no-existe.json")` → `Ok(None)`.
      Test `muestra_devuelve_el_contenido_committeado`: fichero commiteado y
      luego modificado en el working tree; `muestra` devuelve **lo commiteado**.
      Verlos fallar.
- [x] Implementar `muestra` y `es_work_tree` (`rev-parse
      --is-inside-work-tree`, mismo patrón que `es_repo_git` pero sin la
      comparación con `--show-toplevel`: aquí sí vale "dentro de un repo",
      porque el sello se resuelve con `./` contra el `-C`) y `head_resuelve`
      (`rev-parse --verify HEAD`, `bool`).
- [x] Verlos pasar. `cargo clippy --all-targets --locked -- -D warnings` y
      `cargo fmt --check` limpios. Commit.

---

## Task 3: `trinquete.rs` — sellos, su formato y su IO

**Files**
- Create: `engine/src/trinquete.rs`
- Modify: `engine/src/lib.rs` (añadir `pub mod trinquete;` en orden alfabético,
  entre `pub mod trozos;` y `pub mod vectores;` — ojo: alfabéticamente
  `trinquete` va **antes** que `trozos`)
- Test: `engine/tests/trinquete_sellos.rs`

**Interfaces — Produces**
```rust
pub const FICHERO_SELLO: &str = ".kbx-ratchet.json";
pub type Sellos = std::collections::BTreeMap<String, i64>;

pub fn parsea_sellos(datos: &str) -> Result<Sellos>;
pub fn carga(kb: &Path) -> Result<Sellos>;
pub fn serializa_sellos(sellos: &Sellos) -> String;
pub fn escribe_sellos(kb: &Path, sellos: &Sellos) -> Result<()>;
```

El esquema en disco es un objeto con **una** clave, `ceilings`, cuyo valor es
un mapa `ruta → techo`:

```json
{
  "ceilings": {
    "core/doctrina-agentes.md": 20000,
    "projects/agent-develop.md": 17000
  }
}
```

- [ ] Test `parsea_el_sello_real_de_la_kb`: incrustar como fixture el
      `.kbx-ratchet.json` real (11 entradas, incluidas las claves con `—` y
      acentos) y comprobar que parsea a 11 pares, que
      `sellos["core/doctrina-agentes.md"] == 20000` y que la clave con em-dash
      (`"projects/pguerrero.me — Hub personal - portfolio con Lab explorable de LLMs.md"`)
      está presente. Verlo fallar.
- [ ] Test `un_sello_ausente_es_vacio_no_error`: `carga` sobre un directorio sin
      el fichero → `Ok(sellos.is_empty())`. **Esto es un invariante, no una
      comodidad**: es lo que permite que una KB nueva no esté en rojo.
- [ ] Test `un_ceilings_ausente_es_vacio`: `{}` parsea a mapa vacío sin error
      (el Go trata `doc.Ceilings == nil` como `Seals{}`).
- [ ] Test `un_sello_corrupto_es_error`: `{"ceilings":{` → `Err`. Es el
      sub-invariante 3 del ítem 7 de la spec: *"sello corrupto en HEAD es error,
      no abstención"*.
- [ ] Implementar `parsea_sellos` y `carga` con un struct interno
      `#[derive(Deserialize)] struct DocSello { #[serde(default)] ceilings: BTreeMap<String, i64> }`.
      Verlos pasar. Commit.
- [ ] Test `serializa_con_el_formato_exacto_de_kbx`: para un mapa de dos
      entradas, la salida es **literalmente**:
      ```
      {
        "ceilings": {
          "a.md": 100,
          "b.md": 200
        }
      }
      ```
      con `\n` final y sin coma tras la última. Y un round-trip:
      `parsea_sellos(&serializa_sellos(&s)) == s`.
- [ ] Implementar `serializa_sellos` a mano (A10): recorrer el `BTreeMap` (ya
      ordenado), `serde_json::to_string(&clave)` para cada clave —para el
      escapado correcto— y `format!` para el resto. `escribe_sellos` la escribe
      con `std::fs::write`. Verlos pasar. Commit.

---

## Task 4: abstención y ancla — `carga_head` y `anclado_en_head`

**Files**
- Modify: `engine/src/trinquete.rs`
- Test: `engine/tests/trinquete_abstencion.rs`

**Interfaces — Produces**
```rust
/// `Ok(None)` = abstención: no hay historia utilizable (sin repo, shallow, o
/// HEAD que no resuelve). `Ok(Some(sellos))` = hay historia; los sellos pueden
/// estar vacíos si el fichero aún no está commiteado, y eso NO es abstención.
pub fn carga_head(kb: &Path) -> Result<Option<Sellos>>;
pub fn anclado_en_head(kb: &Path) -> bool;
```

**Consumes:** `gitx::es_work_tree`, `gitx::es_shallow`, `gitx::muestra`,
`gitx::head_resuelve` (Task 2); `parsea_sellos` (Task 3).

Definición en prosa, escrita aquí por primera vez (A2), y que va al doc-comment:

- **Abstención por shallow-clone**: en un clon truncado (`--depth`), el objeto
  de HEAD puede no estar, y leer un sello vacío de un HEAD incompleto haría
  pasar en verde cualquier subida de techo. El trinquete prefiere no opinar a
  opinar sobre datos que no tiene: `applied: false` y exit 0.
- **Ancla de activación**: la corrida en la que `.kbx-ratchet.json` **todavía no
  existe en HEAD** es la corrida que instala el trinquete, y en ella los techos
  se sellan sin juzgarlos. Sin esa exención, el día de la instalación toda la KB
  saldría en rojo por sellos que nadie ha tenido ocasión de podar. Una vez el
  fichero está en HEAD, el ancla existe y la exención se acaba para siempre.

- [x] Test `sin_git_se_abstiene`: directorio temporal sin `.git` →
      `carga_head` devuelve `Ok(None)`. Verlo fallar.
- [x] Test `un_repo_shallow_se_abstiene` (mismo montaje que en Task 2).
- [x] Test `sin_fichero_committeado_aplica_con_sellos_vacios`: repo sano con un
      commit que **no** incluye el sello → `Ok(Some(vacío))`. **Es la
      distinción que separa el ancla de la abstención**, y confundirlas es el
      bug: abstenerse aquí desactivaría el trinquete en toda KB nueva.
- [x] Test `el_sello_se_resuelve_contra_la_kb_no_contra_la_raiz_del_repo` (A8):
      crear repo, dentro un subdirectorio `kb/`, poner el sello **en `kb/`**,
      commitear desde la raíz, y comprobar que `carga_head(repo/kb)` devuelve el
      techo. Sin el `./` este test devuelve vacío y pasa en verde el resto de la
      suite: es el test que justifica A8.
- [x] Test `un_sello_corrupto_en_head_es_error_no_abstencion`: commitear
      `{"ceilings":{` → `Err`, no `Ok(None)`.
- [x] Implementar `carga_head` en el orden exacto del Go: `es_work_tree` →
      `es_shallow` → `muestra("HEAD:./" + FICHERO_SELLO)`; si `None`, decidir
      con `head_resuelve` entre `Ok(None)` (repo sin HEAD) y
      `Ok(Some(vacío))` (HEAD sano, sello no commiteado).
- [x] Implementar `anclado_en_head`: `muestra(kb, "HEAD:./" + FICHERO_SELLO)`
      es `Some`. Verlos pasar. Commit.

---

## Task 5: `violaciones` — subir un techo y borrar un sello

**Files**
- Modify: `engine/src/trinquete.rs`
- Test: `engine/src/trinquete.rs` (unitario: es una función pura)

**Interfaces — Produces**
```rust
#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tipo {
    #[serde(rename = "seal-raised")] SelloSubido,
    #[serde(rename = "seal-removed")] SelloRetirado,
    #[serde(rename = "over-seal")] SobreSello,
    #[serde(rename = "first-declaration-too-high")] PrimeraMuyAlta,
    #[serde(rename = "inert-log-waiver")] WaiverLogInerte,
    #[serde(rename = "sealed-escaped-tier")] SelladaEscapadaDeTier,
    #[serde(rename = "no-air")] SinAire,
    #[serde(rename = "born-too-big")] NaceDemasiadoGrande,
    #[serde(rename = "no-air-debt")] DeudaSinAire,
}

impl Tipo {
    /// Los siete que rompen el gate. `WaiverLogInerte` y `DeudaSinAire` son
    /// información: existen para verse, no para bloquear.
    pub fn rompe(self) -> bool;
}

#[derive(Serialize, Clone, Debug)]
pub struct Hallazgo {
    pub ruta: String,
    pub tipo: Tipo,
    #[serde(skip_serializing_if = "es_cero")] pub era: i64,
    #[serde(skip_serializing_if = "es_cero")] pub ahora: i64,
    #[serde(skip_serializing_if = "es_cero")] pub limite: i64,
}
```

Las claves JSON de `Hallazgo` son `path`, `kind`, `was`, `now`, `limit` (inglés,
D7/D8) vía `#[serde(rename)]`.

`Tipo::rompe` se implementa con un `match` **exhaustivo sin `_ =>`**: si mañana
se añade una variante, el compilador obliga a decidir si rompe. Es el mismo
argumento que `quiere_json` en `main.rs`, y es estrictamente mejor que el
`f.Kind != ... && f.Kind != ...` del Go, que ante un Kind nuevo lo trataría como
rompiente por accidente.

- [x] Test `un_techo_que_sube_es_violacion` / `un_techo_que_baja_no_lo_es` /
      `un_sello_nuevo_no_es_violacion`. Verlos fallar.
- [x] Test `borrar_un_sello_equivale_a_subirlo_a_infinito`: presente en `head`,
      ausente en `actual` → `SelloRetirado` con `era` poblado y `ahora` en 0.
      Es el sub-invariante 1 del ítem 7 de la spec.
- [x] Test `las_violaciones_salen_ordenadas_por_ruta`.
- [x] Implementar `pub fn violaciones(head: &Sellos, actual: &Sellos) -> Vec<Hallazgo>`.
      El orden sale gratis del `BTreeMap` (A4), pero el test lo falsa igualmente
      porque el contrato es el orden, no el tipo que lo produce.
- [x] Verlos pasar. Commit.

---

## Task 6: `recolecta` — las declaraciones del árbol

**Files**
- Modify: `engine/src/trinquete.rs`
- Test: `engine/tests/trinquete_recolecta.rs`

**Interfaces — Produces**
```rust
#[derive(Clone, Debug)]
pub struct Declarada {
    pub ruta: String,
    pub tier: String,
    pub max: i64,
    pub tier_presupuesto: i64,
    pub tamano: i64,
}

pub fn recolecta(kb: &Path, presupuestos: Presupuestos, excluidos: &[&str]) -> Result<Vec<Declarada>>;
```

**Consumes:** `walker::walk_notas` y `walker::lee_nota` (existen desde G4b),
`frontmatter::tier`, `frontmatter::budget_max`,
`presupuesto::Presupuestos::para_tier`.

Diferencia con `presupuesto::analiza`, que hay que respetar: `recolecta` trae
**solo las notas que declaran `kbx_budget_max`**, incluidas las que están por
debajo de su tier y que `budget` nunca reporta. El trinquete juzga
declaraciones; `budget` juzga tamaños.

`tier_presupuesto` guarda `para_tier(tier).unwrap_or(0)`: un tier ilegal se
trata como 0 (sin techo), igual que `log`. La distinción `None` vs `Some(0)` que
`budget` necesita aquí no aporta — lo que el trinquete hace con un tier sin
presupuesto es marcar el waiver como inerte, y eso vale para los dos casos.

- [x] Test `recolecta_trae_toda_declaracion_no_solo_las_infractoras`: KB de
      fixture con una nota `stable` de 1 KB y `kbx_budget_max: 12000`
      (muy por debajo de su tier) → aparece en el resultado. Verlo fallar.
- [x] Test `recolecta_respeta_los_excluidos`: una nota con techo dentro de
      `archive/` no aparece.
- [x] Test `recolecta_ignora_las_notas_sin_techo_declarado`.
- [x] Test `el_tamano_es_el_del_fichero_en_bytes`: nota de tamaño conocido; el
      campo `tamano` coincide con `metadata().len()`. **Ojo con CRLF**: el
      fixture se escribe con `\n` explícito para que el tamaño sea el mismo en
      los tres SO del CI.
- [x] Implementar. Ordenar por `ruta` antes de devolver. Verlos pasar. Commit.

---

## Task 7: el emparejamiento de renames, y el sello huérfano que no puede lavar

**Files**
- Modify: `engine/src/trinquete.rs`
- Test: `engine/tests/trinquete_renames.rs`

**Interfaces — Produces** (privada del módulo, pero con firma fijada)
```rust
/// Devuelve (sellos_frescos_absueltos, sellos_retirados_emparejados).
fn empareja_renames(kb: &Path, head: &Sellos, actual: &Sellos) -> (BTreeSet<String>, BTreeSet<String>);
```

**Consumes:** `gitx::muestra` (para `existio_en_head`).

Definición en prosa, escrita aquí por primera vez (A2):

- **Absolución de renames**: `git mv nota-vieja.md nota-nueva.md` retira un
  sello y declara otro en el mismo commit. Sin tratamiento, eso es a la vez un
  `seal-removed` (rojo) y una primera declaración sujeta al cap de 2× tier
  (rojo otra vez) — un rename legítimo saldría en rojo doble. El trinquete
  **empareja** cada sello retirado con un sello fresco de techo **menor o
  igual**, y absuelve a los dos. Es heurístico por construcción: sin mirar
  contenido, un rename y un borrar+crear son indistinguibles.
- **El sello huérfano que no puede lavar** (`0ae126d`): si el emparejamiento no
  comprueba que la nota retirada **existió de verdad en HEAD**, un sello
  huérfano —de una nota que nunca existió— sirve de coartada para declarar
  cualquier techo nuevo. El fixture del bug: sello huérfano de `core/junk.md` en
  HEAD, retirado en el mismo commit que declara `core/attack.md` con techo
  45.000 para una nota de 44.000 B (1,3% de aire, 2,6× el cap de su tier) →
  `fallido() == false` y **cero hallazgos, ni informativos**. La guarda es
  `git rev-parse HEAD:./<ruta>`, que resuelve la entrada del árbol sin traer el
  blob.

Tie-break, portado literal: los retirados y los frescos se ordenan por
`(techo, ruta)`, y para cada retirado se elige, entre los frescos aún libres con
techo `<=` el suyo, **el mayor**. "El mayor que quepa": el test
`TestRenamePairingPrefersTheCeilingItKept` lo fija.

- [x] Test `un_sello_retirado_absuelve_a_uno_solo_no_a_dos`. Verlo fallar.
- [x] Test `un_sello_huerfano_no_puede_hacer_de_rename`: el fixture del párrafo
      anterior, con la nota `core/junk.md` **nunca commiteada**. Debe salir
      `NaceDemasiadoGrande` (o al menos un hallazgo que rompa), no verde. Este
      test es el corazón de la tarea: si pasa en verde, el port nació con el
      agujero de `f0d0564`.
- [x] Test `un_rename_real_sigue_absuelto_tras_la_guarda`: misma forma, pero la
      nota vieja **sí** está commiteada → verde. Sin este test, la guarda podría
      implementarse "rechazando siempre" y el otro test pasaría igual.
- [x] Test `el_emparejamiento_prefiere_el_techo_que_conservo` (tie-break).
- [x] Implementar `existio_en_head(kb, ruta) -> bool` (`muestra` con
      `"HEAD:./" + ruta`, `is_some()`) y `empareja_renames`. Verlos pasar.
      Commit.

---

## Task 8: la guarda de aire — `no-air`, `no-air-debt` y `born-too-big`

**Files**
- Modify: `engine/src/trinquete.rs`
- Test: `engine/tests/trinquete_aire.rs`

**Consumes:** `presupuesto::tiene_aire`, `presupuesto::techo_minimo` — **ya
construidas y testeadas en G4b**. Esta tarea no escribe aritmética.

Constante nueva, la única de esta tarea:
```rust
/// Una primera declaración no puede sellar más de 2× el nominal de su tier.
/// Combinado con el 15% de aire, el techo legal de una nota nueva sale en
/// `tier*2/1,15 = 1,739×` su tier: 14.782 B en core, 21.739 B en stable. Por
/// encima no hay techo legal que valga, y el remedio no es un techo más alto:
/// es partir la nota.
const FACTOR_PRIMERA_DECLARACION: i64 = 2;
```

**La regla que gobierna toda la tarea, y que es lo que hay que no romper:** la
guarda de aire juzga **transiciones, no estado**. Un sello que nadie toca no se
juzga aunque no tenga aire — se reporta como `DeudaSinAire`, que no rompe. Sin
eso, los 11 sellos reales de la KB (ninguno con 15% de aire, medido) dejarían el
repo en rojo permanente el día de la instalación, y el trinquete se
desinstalaría solo.

Las cuatro ramas, en el orden exacto del Go:
1. Tiene aire → nada.
2. Estaba en HEAD y el techo **subió** → nada aquí (ya rompió como
   `SelloSubido`; etiquetarlo además como deuda sería mal-etiquetar una decisión
   de hoy como preexistente). Es el efecto secundario de `0ae126d`.
3. No cambió (`inHead && ceiling == was`) → `DeudaSinAire`, no rompe.
4. Cambió (fresco o bajado) y no está exento por ancla ni por rename →
   `NaceDemasiadoGrande` si es fresco y `techo_minimo(tamano) > tier*2`, si no
   `SinAire`.

- [x] Test `un_sello_fresco_con_aire_suficiente_esta_limpio` y
      `un_sello_fresco_a_un_byte_del_aire_falla`: el borde exacto es
      `techo_minimo(10000) == 11500`; 11500 pasa, 11499 no. Verlos fallar.
- [x] Test `un_sello_bajado_sin_aire_falla`.
- [x] Test `un_sello_intacto_sin_aire_es_deuda_no_fallo`: emite `DeudaSinAire` y
      `fallido() == false`. **El test que sostiene la instalabilidad del
      trinquete.**
- [x] Test `un_techo_subido_no_se_etiqueta_como_deuda` (rama 2).
- [x] Test `la_corrida_de_activacion_esta_exenta_del_aire` (ancla, Task 4).
- [x] Test `un_sello_renombrado_sin_aire_esta_exento` (rename, Task 7).
- [x] Test `una_primera_declaracion_en_zona_muerta_dice_parte_la_nota`
      (`NaceDemasiadoGrande`) y `el_borde_de_la_zona_muerta_esta_limpio`
      (14.782 B en core).
- [x] Test `un_sello_huerfano_sin_fichero_se_salta`: sello sin nota en disco →
      no se inventa tamaño, se salta.
- [x] Test `una_nota_log_sellada_sigue_necesitando_aire`: `log` no tiene
      presupuesto, pero un sello ya puesto sí se juzga.
- [x] Implementar el bloque de aire dentro de `comprueba_contra` (Task 9).
      Verlos pasar. Commit.

---

## Task 9: `comprueba` — las declaraciones y el informe entero

**Files**
- Modify: `engine/src/trinquete.rs`
- Test: `engine/tests/trinquete_declaraciones.rs`

**Interfaces — Produces**
```rust
#[derive(Serialize, Debug)]
pub struct Informe {
    pub aplicado: bool,
    #[serde(skip_serializing_if = "Option::is_none")] pub razon: Option<String>,
    pub hallazgos: Vec<Hallazgo>,
}

impl Informe { pub fn fallido(&self) -> bool; }

pub fn comprueba(kb: &Path, declaradas: &[Declarada], presupuestos: Presupuestos) -> Result<Informe>;
pub fn comprueba_staged(kb: &Path, declaradas: &[Declarada], presupuestos: Presupuestos) -> Result<Informe>;
```

Claves JSON: `applied`, `reason`, `findings`.

`comprueba` y `comprueba_staged` delegan en una `comprueba_contra` privada
parametrizada por dos funciones —de dónde salen los sellos actuales y de dónde
sale el tamaño de una nota— exactamente como el `checkAgainst` del Go:

```rust
fn comprueba_contra(
    kb: &Path,
    declaradas: &[Declarada],
    presupuestos: Presupuestos,
    carga_actual: impl Fn(&Path) -> Result<Sellos>,
    tamano_de: impl Fn(&Path, &str) -> Option<i64>,
) -> Result<Informe>;
```

Las tres familias de hallazgo sobre declaraciones:
- **`SobreSello`**: la nota declara `kbx_budget_max` mayor que su sello. El
  waiver no puede rebasar el trinquete.
- **`PrimeraMuyAlta`**: primera declaración (no está en HEAD, no emparejada por
  rename, no es corrida de activación) con techo > 2× el nominal de su tier.
- **`WaiverLogInerte`**: waiver en un tier sin presupuesto. **No rompe**: es
  información, un techo declarado donde no hay techo que rebasar.
- **`SelladaEscapadaDeTier`**: un sello existe para una nota cuyo tier actual no
  tiene presupuesto — la nota se reclasificó a `log` para escapar del gate.
  Detectado recorriendo los sellos que **no** tienen `Declarada`, leyendo el
  tier del fichero.

- [x] Test `un_waiver_por_encima_de_su_sello_falla` /
      `un_waiver_igual_o_por_debajo_pasa`. Verlos fallar.
- [x] Test `una_primera_declaracion_se_capa_a_dos_veces_el_tier`
      (tier 8500 → límite 17000).
- [x] Test `un_waiver_en_tier_log_es_inerte_no_fallo`: emite `WaiverLogInerte` y
      `fallido() == false`.
- [x] Test `una_nota_sellada_que_escapa_a_log_se_marca`.
- [x] Test `sin_git_el_informe_no_se_aplica`: `aplicado == false`, `razon`
      poblada, `fallido() == false`, y **exit 0** aguas arriba. La abstención es
      información, no fallo.
- [x] Test `los_hallazgos_salen_ordenados_por_ruta`.
- [x] Implementar `comprueba_contra` con el bloque de aire de la Task 8 y el
      emparejamiento de la Task 7. Verlos pasar. Commit.

---

## Task 10: `--staged` — juzgar el índice, no el disco

**Files**
- Modify: `engine/src/trinquete.rs`, `engine/src/gitx.rs`
- Test: `engine/tests/trinquete_staged.rs`

**Interfaces — Produces**
```rust
// en gitx.rs
pub fn md_staged(dir: &Path) -> Result<Vec<String>>;
// en trinquete.rs
pub fn carga_staged(kb: &Path) -> Result<Sellos>;
pub fn recolecta_staged(kb: &Path, presupuestos: Presupuestos, excluidos: &[&str]) -> Result<Vec<Declarada>>;
```

`md_staged` invoca
`git -C <dir> --no-renames diff --cached --name-only --diff-filter=ACMR`
— **con `--no-renames`** (A3; nótese que va como flag de `diff`, no de `git`:
el comando exacto es `diff --cached --no-renames --name-only --diff-filter=ACMR`).

El tamaño bajo `--staged` sale del índice (`git show :./<ruta>`), no del disco.
Es la diferencia que da sentido al modo: el pre-commit juzga lo que se va a
commitear, no lo que hay en el árbol.

- [x] Test `el_modo_staged_ve_el_indice_no_el_working_tree`: stagear una subida
      de techo y luego restaurar el disco; `carga` miente, `carga_staged` dice la
      verdad. Verlo fallar.
- [x] Test `staged_caza_la_subida_que_el_working_tree_esconde`: mismo montaje,
      `comprueba` pasa limpio y `comprueba_staged` la caza.
- [x] Test `staged_juzga_el_tamano_del_indice_no_el_del_disco`.
- [x] Test `sin_sello_staged_el_mapa_es_vacio`.
- [x] Implementar. Un fichero staged como borrado (`git show :./x` falla) se
      **salta**, no rompe. Verlos pasar. Commit.

---

## Task 11: `--seal` — atómico, y nunca sube

**Files**
- Modify: `engine/src/trinquete.rs`
- Test: `engine/tests/trinquete_seal.rs`

**Interfaces — Produces**
```rust
/// `min(sello_actual, declarado)`: un techo solo baja. Nunca sube, ni aunque
/// la nota lo declare más alto.
pub fn sella(actual: &Sellos, declaradas: &[Declarada]) -> Sellos;

/// Los sellos que la transición dejaría sin aire. Solo juzga los que CAMBIAN.
pub fn violaciones_de_aire(actual: &Sellos, siguiente: &Sellos, declaradas: &[Declarada]) -> Vec<Hallazgo>;
```

- [x] Test `sellar_toma_el_valor_menor_y_nunca_sube`: sello 12000, declaración
      15000 → queda 12000. Verlo fallar.
- [x] Test `sellar_se_niega_y_no_escribe_nada_si_falta_aire`: dos notas, una con
      aire y otra sin él. Tras el intento, el fichero **no ha cambiado** —ni
      siquiera para la que sí cumplía. La atomicidad es el contrato: *"o sella
      todo o no sella nada"*.
- [x] Test `sellar_lista_todos_los_infractores_no_solo_el_primero`.
- [x] Test `sellar_escribe_cuando_todo_tiene_aire`, comprobando además que el
      fichero resultante parsea y conserva las entradas que no se tocaron.
- [x] Test `las_violaciones_de_aire_solo_juzgan_techos_que_cambian`: un sello
      intacto sin aire no aparece. Es la misma regla de la Task 8, del otro
      lado.
- [x] Implementar. Verlos pasar. Commit.

---

## Task 12: cablear `exo ratchet` — el segundo exit 3

**Files**
- Modify: `engine/src/main.rs`
- Test: `engine/tests/ratchet_cli.rs`

**Interfaces — Produces**
```rust
#[derive(clap::Args)]
struct ArgsRatchet {
    /// Raíz de la KB. Precedencia: flag > $EXO_KB > config
    #[arg(long)] kb: Option<PathBuf>,
    /// Escribe `.kbx-ratchet.json` con `min(sello, declarado)`. Atómico.
    #[arg(long, conflicts_with = "staged")] seal: bool,
    /// Juzga el índice de git en vez del working tree (para el pre-commit).
    #[arg(long)] staged: bool,
    /// Emite el resultado como envelope JSON (spec §4) en stdout
    #[arg(long)] json: bool,
}
```

- [x] Añadir `Ratchet(ArgsRatchet)` a `Comando`, la rama en `ejecuta`, y **la
      rama en `quiere_json`** — que es exhaustivo sin `_ =>`, así que el
      compilador va a exigirla. Verlo fallar primero (test que invoca
      `exo ratchet --help`).
- [x] Implementar `ratchet_cmd`: resolver KB, recolectar (staged o no),
      comprobar, emitir el informe **entero antes de gatear** (mismo contrato
      que `budget_cmd` y `lint_cmd`), y devolver
      `gate::GateFallido { comando: "ratchet", detalle: ... }` si
      `informe.fallido()`.
- [x] Salida de texto: la causa primero, la deuda resumida en una línea al
      final. Portado de `TestTextOutputLeadsWithTheCauseAndSummarisesTheDebt`:
      la deuda **no** se mezcla con lo que rompe, porque si se mezcla nadie
      distingue lo que tiene que arreglar hoy de lo que arrastra desde hace
      meses.
- [x] Test `ratchet_sin_git_sale_cero`: abstención → exit 0.
- [x] Test `ratchet_con_un_waiver_sobre_su_sello_sale_tres`.
- [x] Test `ratchet_seal_y_staged_juntos_son_error_de_uso`: exit 2 (clap).
- [x] Test `el_envelope_json_lleva_command_ratchet` y valida contra el contrato
      de `envelope` (mismo patrón que `budget_lint_cli.rs`).
- [x] Verlos pasar. `cargo clippy --all-targets --locked -- -D warnings` y
      `cargo fmt --check`. Commit.

---

## Task 13: el cutover, hasta donde llega

**Files**
- Modify: `plugins/exo/scripts/kb-precommit.sh`
- Modify: `plugins/exo/skills/distill/SKILL.md`
- Modify: `plugins/exo/skills/document/SKILL.md`
- Modify: `plugins/exo/skills/document/routing.md`
- Modify: `plugins/exo/agents/executor.md`

**Nada de esta tarea toca `kbx_budget_max`, `kbx_orphan_ok` ni
`.kbx-ratchet.json`** (A6). Migra invocaciones de binario y solo eso.

- [x] `kb-precommit.sh`: `$KBX ratchet --kb "$KB" --staged` →
      `$EXO ratchet --kb "$KB" --staged`; `$KBX budget --kb "$snap"` →
      `$EXO budget --kb "$snap"`. Ajustar la variable
      (`EXO="${EXO_BIN:-$HOME/.local/bin/exo}"`) y el guard de la línea 17.
      **La degradación se conserva literal**: binario ausente ⇒ `exit 0` ⇒
      commit permitido. Cambiarla a bloqueante aquí sería un cambio de política
      colado en un cutover.
- [x] `kb-precommit.sh` línea 58: el mensaje de remediación cita
      `kbx rotate --kb <kb> --apply`. **Se queda como `kbx`** — es el binario
      que hay que invocar de verdad para eso, porque `exo rotate` no existe.
      Añadir una nota de una línea diciendo que ese verbo sigue en kbx.
- [x] `distill/SKILL.md`: migrar `budget` → `exo budget`,
      `ratchet` → `exo ratchet`, `doctor` → `exo lint`. **Dejar `rotate` y
      `stale` en `$KBX_BIN`** y escribir explícitamente, en el propio SKILL,
      que el skill invoca dos binarios y por qué (A7). Verificar que la
      degradación anunciada sigue: sin `rotate`, `distill` detecta la ausencia y
      lo dice en una línea visible, no falla-fuerte — el fallo-fuerte dejaría la
      skill inservible en Windows, que es el estado que esta ola viene a
      arreglar.
- [x] `document/SKILL.md` (línea 24), `document/routing.md` (línea 10),
      `agents/executor.md` (línea 13): `kbx targets` → `exo targets`. Los tres
      son prosa o una invocación con flags; ninguno tiene verbo sin destino.
- [x] Verificación de campo, no solo de grep: instalar el binario nuevo
      (`cargo build --release` + copiar a `~/.local/bin/exo`), hacer un commit
      de prueba en un **clon desechable** de la KB con una subida de techo
      staged, y comprobar que el pre-commit la bloquea. Un cutover que solo se
      verifica con `grep` es un cutover no verificado.
- [x] Commit.

---

## Task 14: anotar en el backlog lo que G4c deja declarado

**Files**
- Modify: `docs/backlog.md`

- [x] Anotar, con una línea cada uno: (a) el gate de paridad de `ratchet` queda
      **pendiente de la máquina Linux**, junto con el de `targets`, y los dos
      comparten prerequisito (compilar kbx `fe46443`); (b) "los nueve
      invariantes" era un lapsus — son siete, uno del ratchet (A1); (c) el
      cutover es parcial: `rotate`, `stale`, `history` y `diff-since` siguen sin
      portar, y `distill` sigue necesitando el binario `kbx` por los dos
      primeros; (d) las 2 rutas `/home/paul/…` de
      `plugins/exo/scripts/test-git-c-bash.sh:74-75` siguen ahí.
- [x] Commit.

---

## Residuo declarado

- **El gate de paridad no se corre en W11.** Sin toolchain Go no hay mitad de
  referencia. Queda escrito y pendiente, con el de `targets`.
- **El repo kbx sigue divergido** (`f0d0564` local, 1 por delante y 18 por
  detrás de `fe46443`, conflicto en `budget.go`). Este plan **no lo toca**: lee
  `fe46443` y ya. Reconciliarlo exige Go, y conviene decidir en la máquina Linux
  si se reconcilia o se deja morir cuando `exo` sustituya a `kbx`.
- **`rotate`, `stale`, `history`, `diff-since`** siguen sin portar. Mientras
  `rotate` no exista en exo, `distill` no puede cortar el cordón, y el remedio
  que el pre-commit prescribe cuando muerde (`kbx rotate`) exige kbx instalado.
  **Es el candidato natural a G4d**, y no es cosmético: `rotate` es el remedio
  que la doctrina manda aplicar cuando el gate muerde.
- **El emparejamiento de renames sigue siendo heurístico.** Borrar una nota real
  de 50.000 y crear otra de 44.000 en el mismo commit se empareja igual que un
  rename. Sin mirar contenido, son indistinguibles; el fix de `0ae126d` cierra
  el caso del sello huérfano, no este.

## Los planes que siguen

**G5b — release, instaladores, `doctor`.** Sin cambios de alcance por G4c, salvo
que el instalador ahora tiene que dejar `exo` donde `kb-precommit.sh` lo busca
(`~/.local/bin/exo`), porque tras el cutover el hook de la KB depende de él. Y
el `doctor` de G5b hereda un candidato de check que este plan no toca: el assert
de dimensión/norma tras `embebe_batch`, que hoy no existe.
