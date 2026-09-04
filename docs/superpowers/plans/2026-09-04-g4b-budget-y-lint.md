# G4b — `exo budget` + `exo lint` Implementation Plan

**For agentic workers:** la skill de ejecución es `exo:orchestrate`. Los pasos
llevan checkbox (`- [ ]`) para tracking. Cada tarea es un ciclo cerrado (test
que falla → verlo fallar → implementación mínima → verlo pasar → commit) y
merece el gate de un reviewer fresco.

**Goal:** que `exo budget` y `exo lint` existan en el engine Rust con la
semántica de `kbx budget` y `kbx doctor` (menos `schema_drift`), con **una
sola** fuente de verdad para los nominales de tier y para la aritmética de
aire, y que la decisión de **exit 3** se ejerza por primera vez.

**Architecture:** igual que G4a, el port es de lenguaje y no de datos: los
checks leen el mismo índice SQLite (`notas`, `aristas`) y el mismo árbol de
ficheros. Se añaden tres módulos (`presupuesto.rs`, `lint.rs`, `gate.rs`), una
función nueva en `walker.rs`, y dos subcomandos en `main.rs`. `presupuesto.rs`
es la hoja del árbol de dependencias: define los tiers, los nominales y la
aritmética de aire, y **tanto `budget` como el check `budget_exceeded` de
`lint` consumen la misma función de clasificación** — es la unificación que kbx
no tiene. El orden va de dentro afuera: primero el pre-registro del gate (antes
de tocar código), luego la corrección de `frontmatter`, luego las piezas puras,
luego las que tocan disco y DB, y por último el cableado del CLI.

**Tech Stack:** Rust edition 2024, MSRV 1.95 · `rusqlite` 0.40.1 (bundled) ·
`regex` 1.13.1 · `serde`/`serde_json` · `anyhow` · `clap` 4.6.2 derive ·
`tempfile` 3.14 (dev). Sin dependencias nuevas.

## Global Constraints

Se heredan **enteras** de G4a
(`docs/superpowers/plans/2026-09-02-g4a-plomeria-y-targets.md`, §Global
Constraints) y no se repiten aquí salvo donde G4b las ejerce o las cambia:

- **El crate vive en `engine/`.** No hay workspace de Cargo. Todo comando de
  cargo se ejecuta con cwd `engine/`.
- **Fuente del port: `origin/main` de kbx, commit `fe46443`.** Ver Corrección 1:
  a diferencia de lo que dice el plan de G4a, `no_air` **también** se porta
  desde `fe46443`, no desde el diff de `f0d0564`. Nada de este plan toca el
  repo kbx.
- **Hallazgos ⇒ exit 3, errores ⇒ exit 1.** Aquí se ejerce por primera vez.
  kbx usa `1` = hay hallazgos y `2` = error; exo usa `1` = error genérico,
  `2` = error de parseo de clap, `3` = gate rechazado. **Es una divergencia
  deliberada de exit code frente a kbx, y se declara en el pre-registro.**
- **`SCHEMA_VERSION` sigue en 2 y no se toca** (`engine/src/envelope.rs`).
  `budget` y `lint` son `command` nuevos: aditivo, no breaking.
- **Claves JSON en inglés, identificadores Rust en castellano** (D7/D8), con
  `#[serde(rename)]` en cada campo que difiera. Comentarios en castellano,
  explicando el porqué.
- **Avisos y progreso a stderr, resultado primario a stdout.** Con `--json`,
  stdout lleva el envelope y nada más.
- **Errores con `anyhow`**, `.context(...)` accionable en cada IO/parse
  falible. No se añade `thiserror`.
- **Clippy es gate duro** desde el primer commit:
  `cargo clippy --all-targets --locked -- -D warnings` limpio, y
  `cargo fmt --check`.
- **Los tres tiers son `core`, `stable`, `log`**, y sus nominales
  (8500 / 12500 / 0) viven **en una sola constante**, que nace en este plan.
- **Aritmética entera, nunca floats**, en todo lo que gatee. Y explícita ante
  overflow: Rust **panica** en debug donde Go hace wraparound silencioso, así
  que se usa `checked_mul` a conciencia en vez de heredar el silencio por
  accidente.
- **Fixture: no existe `kb-demo` en disco.** Cada test construye su KB en un
  `tempfile::tempdir()`. El helper de config compartido es
  `engine/tests/common/mod.rs::con_config`.
- Fuera de scope, y se declara en vez de disimularse: `ratchet` y el cutover de
  `kb-precommit.sh` y las skills a verbos `exo` (G4c), y
  `rotate`/`stale`/`diff-since`/`history`, que por D5 no se portan nunca.

---

## Adjudicaciones — las decisiones que este plan cierra

A1 y A2 venían abiertas del review de la PR #2 de G4a, donde el autor declaró
que no podía autocalificarse. Adjudicadas por Paul el 2026-09-04.

### A1 — `tier` filtra whitespace **ASCII**, no Unicode

El `stripWhitespace` de kbx (`internal/frontmatter/frontmatter.go:103-111`) es
un `strings.Map` que borra seis runes ASCII (`' '`, `'\t'`, `'\n'`, `'\r'`,
`'\v'`, `'\f'`) **en toda la cadena**. El `tier()` de exo
(`engine/src/frontmatter.rs`) hace un strip global equivalente, pero con
`char::is_whitespace()`, que es la propiedad Unicode `White_Space` (NBSP,
U+2007, U+3000…).

**Un solo eje de divergencia, no dos.** Los dos son strip global; lo único que
difiere es el conjunto de caracteres. (Una lectura del 2026-09-04 sugirió
además una divergencia trim-vs-strip que **no existe**: verificado en el código
de los dos lados.)

**Decisión: alinear a ASCII.** Dos razones, y la segunda pesa más que la
paridad:

1. `tier` es uno de los tres campos que el criterio del gate exige idénticos.
2. **Unicode degrada hacia verde.** Con `tier: co<NBSP>re`, el filtro ASCII deja
   `"co\u{a0}re"`, que no es un tier legal ⇒ la nota cae en `notier` ⇒ el gate
   salta, visible. El filtro Unicode lo "arregla" en silencio y la cuela como
   `core`. El propio `frontmatter.rs` declara en sus comentarios el principio
   contrario —*degradación hacia rojo, nunca hacia verde*— y `tier()` es la
   única función del módulo que lo incumple. Con G4b clasificando por tier, ese
   incumplimiento pasa de cosmético a load-bearing.

### A2 — un verbo que necesita git distingue "KB sin versionar" de "fallo de git"

`exo init --from-basic-memory` crea KBs sin `git init`. Sobre una de esas,
`targets` revienta en la primera candidata por el fail-loud de `gitx`
(invariante 6 de la spec), mientras `search`, `recall` e `index` funcionan. El
fail-loud se justificó entero desde el contrato de kbx, que asumía KB
versionada; exo no la asume.

**Decisión: distinguir, con un error accionable.** Se chequea **una vez**, antes
del bucle, si la raíz de la KB es la raíz de un work tree de git; si no lo es,
un único error accionable (exit 1) que nombra la condición y la salida. El
fail-loud por fichero **se conserva intacto** para los fallos reales de git: lo
que cambia es que una condición de la KB deje de disfrazarse de fallo de
fichero.

**No basta `rev-parse --is-inside-work-tree`.** Devuelve `true` para una KB
*anidada dentro de un repo ajeno* —un `$HOME` con los dotfiles versionados, por
ejemplo—, y ahí `git log` sobre cada nota resuelve sin error y devuelve cadena
vacía: `last_commit` sale vacío para **todas** las notas, en silencio, que es
exactamente el fallo que A2 dice venir a cerrar. Se compara además
`rev-parse --show-toplevel` con la raíz de la KB. El caso es preexistente y no
lo introduce este plan, pero A2 no puede prometer distinguir la condición y
dejar fuera la variante que degrada a verde.

Entra en **este** plan (Task 11) y no en G4c: hoy hay un camino roto en `main`,
y dejarlo una ola entera es peor que arreglarlo. `budget` y `lint` no usan git,
así que la tarea es independiente del resto.

### A3 — `no_air`: la fórmula canónica es la de `fe46443`

Ver Corrección 1 para el hallazgo. **Decisión: canónica la de `fe46443`**
(`techo*100 >= tamaño*115`, con `objetivo_poda(techo) = techo*100/115`),
reutilizada desde `presupuesto.rs`. El commit local `f0d0564` se declara
**huérfano** y se deja morir con el repo kbx.

Consecuencia fuera de este repo, y hay que decirla porque cambia trabajo ya
planificado: **la campaña de evicción de la KB está calibrada con la fórmula
huérfana.** Los umbrales canónicos son

| tier | nominal | objetivo de poda (canónico) | objetivo con la fórmula huérfana |
|---|---|---|---|
| core | 8.500 | **7.391** | 7.225 |
| stable | 12.500 | **10.869** | 10.625 |

244 bytes menos de evicción por nota en `stable`, y un censo de infractoras
menor: las que viven entre 10.626 y 10.869 dejan de estarlo. El recuento de
"19 de 58 notas stable" del 2026-09-02 hay que rehacerlo con `exo budget`
cuando exista. **Este plan no toca la KB.**

### A4 — la exclusión compara el **primer segmento** de la ruta relativa

kbx tiene dos semánticas, y la diferencia no es la que decía el plan de G4a:

- `budget` (`internal/budget/budget.go:112-120`) poda por **basename en cada
  nivel** del walk (`filepath.SkipDir`).
- `doctor` (`internal/doctor/doctor.go:31-42`, `isExcluded`) compara **solo el
  primer segmento** de la ruta relativa.

**Las listas NO difieren**: ambas son `{".superpowers", "archive", "docs"}` y
solo cambia el orden del literal. La decisión es de profundidad, no de
conjunto. Corrige eso el plan de G4a, que afirmaba que diferían.

**Medido en la KB real el 2026-09-04**: hay exactamente dos directorios con esos
nombres (`archive/` y `docs/`) y **ninguno anidado**. Las dos semánticas dan hoy
resultados idénticos, así que elegir no cambia ningún número.

**Decisión: el primer segmento** (la de `doctor`). Ante un `projects/archive/`
futuro, la semántica de `doctor` lo **chequea** y la de `budget` lo **salta en
silencio**; se elige la que degrada hacia rojo. Se declara como divergencia de
`budget` en el pre-registro, con la medición como evidencia de que no es un
cambio de comportamiento hoy.

### A5 — `.md` es case-insensitive, en los dos verbos

kbx es incoherente: `budget` usa `strings.EqualFold` (case-insensitive,
`budget.go:122`) y `doctor` usa `strings.HasSuffix(name, ".md")`
(case-sensitive, `doctor.go:240`).

**Decisión: case-insensitive en los dos.** Un `NOTA.MD` es una nota; tratarla
como no-nota la saca del gate en silencio. Se declara como divergencia de
`lint` frente a `kbx doctor` en el pre-registro: sobre una KB con un `.MD`,
`exo lint` producirá hallazgos que `kbx doctor` no produce, **y eso es
correcto**.

Nota: esto **no** alinea con `walker::walk_kb`, que sigue siendo
case-sensitive porque lo consume el indexer. Ver Residuo.

### A6 — `root_file` sigue ignorando la lista de exclusión

Confirmado: `rootFileFindings` (`doctor.go:305-325`) ni siquiera recibe el
parámetro `exclude`. **Se porta tal cual: la conducta es la correcta.**

Pero el porqué hay que decirlo bien, porque la explicación intuitiva es falsa.
No es que "sobre un fichero en la raíz no haya segmento que excluir":
`isExcluded("docs", exclude)` en Go devuelve **`true`**, porque el primer
segmento de `"docs"` es `"docs"`. Un fichero llamado `docs` en la raíz **sí**
casaría con el predicado. Lo que pasa es que `root_file` **nunca lo llama**.

La conducta correcta es la que hay —un fichero suelto en la raíz se reporta se
llame como se llame— y la razón es de dominio, no de aritmética de rutas: la
lista excluye **directorios fuera del scope de la KB**, y un fichero de
profundidad 0 no está dentro de ninguno de ellos. Aplicarle el predicado sería
un choque accidental de nombres. Se documenta así, y con un test que fija el
caso `docs`, para que la próxima lectura no "arregle" el check pasándole la
lista.

### A7 — `schema_drift` muere y se declara

Existía porque kbx y exo eran dos binarios contra un schema compartido; con un
solo binario deja de tener objeto. `kbx doctor` emite **7** tipos de finding;
`exo lint` emite **6**. Es la divergencia estructural del gate de paridad de
`lint`, y se declara antes de correrlo.

---

## Correcciones a premisas de G4a

Salieron de releer los dos códigos el 2026-09-04. Se registran porque el plan de
G4a las da por buenas y alguien las volverá a leer.

### Corrección 1 — `no_air` ya existe en `fe46443`, con otra fórmula

El plan de G4a dice: *"`no_air` se porta aparte desde el diff de `f0d0564`"*.
Es **falso**. `origin/main` (`fe46443`) tiene su propia guarda de aire, más
integrada y con más historia:

- `internal/ratchet/check.go:56-85` — `hasAir`, `minCeilingFor`,
  `pruneTargetFor`, exportadas *"so there is ONE detector of 'has enough
  air' — two copies with drifting tolerances is the failure mode this KB has
  already paid for"*.
- `internal/budget/budget.go` ya la consume para poblar `Report.NoAir`.
- `internal/budget/air_test.go` — 6 tests dedicados.

`f0d0564` (local) es una **reinvención independiente**: partió de `9395199`, un
ancestro anterior a esos 18 commits de origin. Misma clave JSON `no_air`, tipo
Go distinto (`NoAirEntry` frente a reutilizar `Offender`) y **aritmética
distinta**:

| | fórmula | umbral stable | umbral core |
|---|---|---|---|
| `f0d0564` | `tier - tier*15/100` | 10.625 | 7.225 |
| `fe46443` | `techo*100 >= tamaño*115` | 10.869 | 7.391 |

Eso explica que el conflicto de `budget.go` no sea textual: son dos semánticas
con el mismo nombre. Adjudicado en A3.

### Corrección 2 — no existe el verbo `kbx lint`

Los verbos de `fe46443` son `doctor, targets, history, diff-since, budget,
stale, rotate, ratchet`. Los seis checks viven en `internal/doctor/doctor.go`,
invocados por `kbx doctor` en bare mode. `exo lint` es un **nombre nuevo** para
el pipeline de doctor menos `schema_drift`. **El gate de paridad de `lint`
compara contra `kbx doctor`, no contra un `kbx lint` que no existe.**

### Corrección 3 — los nominales están duplicados en 2 sitios, no 3

El plan de G4a dice *"definidos tres veces"* y nombra `cmd/kbx/budget.go`,
`doctor.DefaultBudgetOptions` y *"recompuestos en `cmd/kbx/ratchet.go`"*. En
`fe46443` los sitios de **producción** son dos:

- `cmd/kbx/budget.go:22-23` — flags `budget-core` / `budget-stable`.
- `internal/doctor/doctor.go:80-81` — `DefaultBudgetOptions()`.

`cmd/kbx/main.go` no tiene un tercer literal (usa
`doctor.DefaultBudgetOptions()`). El tercer sitio es
`internal/budget/budget_test.go:13`, un helper de test. No cambia la decisión
—una sola constante— pero sí lo que hay que buscar.

### Corrección 4 — el "test de paridad" no es un arnés

El plan de G4a dice que `doctor.budgetExceededFindings` está vigilada por *"un
test de paridad dedicado"*. Lo que hay es una **regresión dirigida**:
`TestBudgetExceededLogTierOverrideIsAFinding`
(`internal/doctor/doctor_test.go`), que cubre un caso concreto — `tier: log`
(nominal 0) con `kbx_budget_max: 50` y cuerpo mayor — donde un
`if tierBudget <= 0 { continue }` prematuro hacía que doctor callase y `budget`
disparase. No ejecuta las dos implementaciones ni difea sus salidas.

En exo el problema **desaparece por construcción**: `lint` no reimplementa la
clasificación, llama a `presupuesto::clasifica`. El test se porta igual
(Task 8), pero como lo que es: la regresión que demuestra que hay una sola
implementación, no dos que coinciden.

---

## Task 1: pre-registro del gate de paridad — **antes de una sola línea de Rust**

**Files:**
- Create: `docs/superpowers/plans/2026-09-04-g4b-preregistro-budget-lint.md`

**Interfaces:**
- Consumes: nada. Es un documento.
- Produces: el criterio contra el que se juzgará el port, fijado antes de poder
  mirar ningún output.

**Por qué es la Task 1 y no la última.** En G4a esta tarea se ejecutó al final y
el documento resultante tuvo que titularse *"Registro del gate de paridad —
**NO es un pre-registro**"*, porque cuando se redactó el binario ya estaba
compilado y corrido. Un pre-registro firmado a posteriori es un check no
falsable. Aquí se escribe primero, que es lo único que lo hace valer. Y se
puede: la forma de la salida la fija kbx, no el port.

- [ ] **Step 1: Escribir el documento**

Con estas ocho secciones, la estructura de
`2026-09-02-g4a-preregistro-targets.md`:

1. **Encabezado** — declarar que **sí** es un pre-registro, con la fecha y el
   hecho verificable de que se escribe antes del primer commit de código de
   G4b. Y declarar lo que sigue sin observarse: *el lado Go no se ha corrido ni
   una vez* (`go: command not found` en W11, medido el 2026-09-04).
2. **Referencia** — kbx `fe46443` compilado en la máquina Linux; binario exo de
   la rama `g4b-budget-lint` en release; copia del índice vivo; la KB real.
3. **Qué se compara y qué no** — dos comparaciones independientes:
   `exo budget --json` contra `kbx budget --json`, y `exo lint --json` contra
   `kbx doctor --json`.
4. **Divergencias declaradas antes de correr nada** — las **nueve**:
   1. A1 (ASCII), que **elimina** la divergencia 3 del registro de G4a.
   2. A4 (exclusión por primer segmento en `budget`, donde kbx poda por basename
      en cada nivel), con la medición de cero dirs anidados en la KB como
      evidencia de impacto nulo hoy.
   3. A5 (`.md` case-insensitive en `lint`, donde `kbx doctor` es sensible).
      Medido: 172/172 `.md` de la KB en minúscula, impacto nulo hoy.
   4. A7 (6 tipos de finding frente a 7).
   5. El **exit code** (kbx: 1 = hallazgos, 2 = error; exo: 3 = hallazgos,
      1 = error, 2 = clap).
   6. **Dotdirs y dotfiles**: el walk de exo salta **todo** directorio y fichero
      que empiece por `.`, a cualquier nivel; `kbx budget` solo poda los tres
      basenames de su lista y recorre `.git/`, `.claude/` y `.omc/`, y contaría
      un `.oculto.md`. Medido en la KB: solo hay `.git/` y no contiene `.md`,
      así que el impacto es cero hoy. **Se declara porque el criterio dice que
      toda diferencia no declarada es un fallo, y esta la introduce el port a
      propósito.**
   7. **Estrictez UTF-8**: el port lee las notas con `read` +
      `from_utf8_lossy` (el idioma de `objetivos.rs`), igual que Go, que trabaja
      sobre bytes. Se declara porque una versión anterior de este plan usaba
      `read_to_string`, que habría abortado el verbo entero ante un byte
      inválido.
   8. **`index_stale`**: `exo lint` emite un tipo de hallazgo que `kbx doctor`
      no tiene (Task 9). Sube a **7** los tipos de exo frente a los 7 de kbx,
      pero **no son los mismos siete**: exo no tiene `schema_drift` y kbx no
      tiene `index_stale`. El gate compara excluyendo los dos de sus
      respectivos lados.
   9. El **orden de `no_air`** (kbx ordena por `size_bytes` desc y desempata por
      ruta; el port hace lo mismo — se declara por si el desempate difiere).
5. **El corpus** — la KB real completa, no una muestra: los dos verbos recorren
   el árbol entero, así que no hay "topics" que elegir como en `targets`.
6. **Criterio** — el de abajo, verbatim, para que no se reescriba luego.
7. **Comandos** — el bloque bash ejecutable de abajo.
8. **Lo que ya está medido** — vacío en el momento de escribir, y se dice.

**Criterio, que se fija ahora:**

- `budget` **PASA** si `tiers[]` es idéntico campo a campo, y si los conjuntos
  de `offenders`, `waived`, `no_air` y `notier` son idénticos por `path`, y para
  cada `path` coinciden `tier`, `size_bytes` y `budget`.
- **`no_air` gatea como los demás.** Una versión anterior de este criterio lo
  eximía "por A3", y estaba mal razonado: la referencia declarada es `fe46443`
  compilado fresco, que tiene **la misma** fórmula que el port. Contra esa
  referencia `no_air` tiene que coincidir, y eximirlo era debilitar el gate a
  priori con una justificación que solo valdría contra el binario local de
  `f0d0564` — el que el propio documento declara que no sirve de referencia. La
  recalibración de la campaña de la KB es otra cosa y no vive en este gate.
- `lint` **PASA** si el conjunto de `findings` es idéntico por `(type, path)`
  tras excluir los de tipo `schema_drift` **del lado Go** (A7) y los de tipo
  `index_stale` **del lado Rust** (divergencia 8), y si `waived` coincide igual.
  Los `detail` se comparan y una diferencia se investiga, pero no gatea por sí
  sola: son cadenas de presentación.
- **Si el lado Rust emite un solo `index_stale`, el gate se aborta y no se
  juzga**: significa que el índice de la copia no corresponde al árbol de la KB,
  y entonces `orphan` está midiendo otra cosa en cada lado. Se re-indexa y se
  repite. Es la condición que hace no comparable la comparación.
- Cualquier diferencia **no** cubierta por una de las nueve divergencias
  declaradas es un **FALLO** del port. La resolución no es reclasificarla como
  divergencia aceptable a posteriori.
- El gate global pasa si pasan las dos comparaciones.

```bash
mkdir -p /tmp/g4b && cp ~/.exo/index.db /tmp/g4b/index.db
KB=~/…/wisdom-paul

# `exo budget` NO tiene `--db`: no toca el índice, solo el árbol de ficheros.
# `kbx budget` sí lo acepta (lo usa para el fallback de meta.kb_root), pero
# aquí sobra porque `--kb` va explícito en los dos lados.
kbx budget --kb "$KB" --json \
  | jq -S '.data | {tiers, offenders, waived, no_air, notier}' > /tmp/g4b/go-budget.json
./target/release/exo budget --kb "$KB" --json \
  | jq -S '.data | {tiers, offenders, waived, no_air, notier}' > /tmp/g4b/rs-budget.json
diff -u /tmp/g4b/go-budget.json /tmp/g4b/rs-budget.json \
  && echo "PASA: budget" || echo "REVISAR: budget"

kbx doctor --db /tmp/g4b/index.db --kb "$KB" --json \
  | jq -S '.data | {findings: [.findings[] | select(.type != "schema_drift")], waived}' \
  > /tmp/g4b/go-lint.json
./target/release/exo lint --db /tmp/g4b/index.db --kb "$KB" --json \
  | jq -S '.data | {findings: [.findings[] | select(.type != "index_stale")], waived}' \
  > /tmp/g4b/rs-lint.json
diff -u /tmp/g4b/go-lint.json /tmp/g4b/rs-lint.json \
  && echo "PASA: lint" || echo "REVISAR: lint"

# Medición aparte, que NO es el gate: el censo de no_air con la fórmula
# canónica frente al que produjo el binario local de f0d0564 el 09-02. Es el
# número que recalibra la campaña de evicción de la KB (A3).
jq -S '.data.no_air | length' /tmp/g4b/rs-budget.json
```

- [ ] **Step 2: Verificar que el documento no miente sobre su propio orden**

Run: `git log --oneline -3`
Expected: ningún commit de la rama `g4b-budget-lint` toca `engine/src/`. Si lo
toca, este documento ya no es un pre-registro y hay que titularlo como registro,
igual que en G4a.

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/plans/2026-09-04-g4b-preregistro-budget-lint.md
git commit -m "plan(g4b): el pre-registro del gate, esta vez antes del codigo"
```

---

## Task 2: `frontmatter::tier` alineado a ASCII (A1)

**Files:**
- Modify: `engine/src/frontmatter.rs`
- Test: inline, `#[cfg(test)] mod tests` del mismo fichero

**Interfaces:**
- Consumes: nada nuevo.
- Produces: `pub fn tier(contenido: &str) -> String` — misma firma, distinto
  conjunto de caracteres filtrados. Es la única función que cambia de
  comportamiento en todo el plan.

- [ ] **Step 1: Escribir los tests que fallan**

Añadir al `mod tests` de `engine/src/frontmatter.rs`:

```rust
#[test]
fn el_tier_no_absuelve_el_whitespace_unicode() {
    // A1: el NBSP NO se filtra, así que el tier queda ilegal y la nota cae en
    // `notier` — el gate salta y se ve. Filtrarlo (char::is_whitespace) la
    // colaría como `core` en silencio: degradación hacia verde, justo lo que
    // el resto de este módulo evita a propósito.
    let contenido = "---\ntier: co\u{a0}re\n---\n";
    assert_eq!(tier(contenido), "co\u{a0}re");
}

#[test]
fn el_tier_sigue_filtrando_los_seis_espacios_ascii() {
    // Paridad con stripWhitespace de kbx: los seis, en cualquier posición.
    let contenido = "---\ntier:  c o\tr\u{b}e\u{c}\r\n---\n";
    assert_eq!(tier(contenido), "core");
}

#[test]
fn el_tier_limpio_no_cambia() {
    assert_eq!(tier("---\ntier: stable\n---\n"), "stable");
}
```

- [ ] **Step 2: Correr los tests y verlos fallar**

Run: `cd engine && cargo test --lib frontmatter`
Expected: FAIL de `el_tier_no_absuelve_el_whitespace_unicode` con
`assertion \`left == right\` failed: left: "core", right: "co\u{a0}re"`. Los
otros dos pasan ya. Que falle por el **valor**, no por compilación: la firma no
cambia.

- [ ] **Step 3: Implementación**

En `engine/src/frontmatter.rs`, sustituir el cuerpo de `tier`:

```rust
/// Los seis caracteres que kbx borra del `tier`
/// (`internal/frontmatter/frontmatter.go`, `stripWhitespace`). ASCII a
/// propósito, no `char::is_whitespace()`: un NBSP dentro de `tier: co<NBSP>re`
/// debe dejar el tier ILEGAL (⇒ `notier`, gate visible) en vez de repararlo en
/// silencio y colarlo como `core`. Degradación hacia rojo, como el resto del
/// módulo (A1 del plan de G4b).
const ESPACIOS_ASCII: [char; 6] = [' ', '\t', '\n', '\r', '\u{b}', '\u{c}'];

pub fn tier(contenido: &str) -> String {
    let mut salida = String::new();
    escanea(contenido, |clave, crudo| {
        if clave == "tier" {
            salida = crudo.chars().filter(|c| !ESPACIOS_ASCII.contains(c)).collect();
            return true;
        }
        false
    });
    salida
}
```

- [ ] **Step 4: Correr los tests y verlos pasar**

Run: `cd engine && cargo test --lib frontmatter`
Expected: PASS, **exactamente 3 tests más** de los que listaba
`cargo test --lib frontmatter -- --list` antes de esta tarea. Anota el número
de partida en vez de fiarte de una cifra escrita aquí: la suite crece entre
olas y una cifra hardcodeada manda al implementador a buscar tests que no
existen.
Run: `cd engine && cargo test`
Expected: PASS, la suite entera. **Si alguno de los tests anteriores cae, no se
"arregla" el test: es que el cambio de A1 tiene un consumidor que nadie había
declarado, y eso se investiga antes de seguir.**
Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
Expected: sin salida, exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/frontmatter.rs
git commit -m "fix(frontmatter): el tier absolvia el NBSP y lo colaba como core"
```

---

## Task 3: `presupuesto.rs` — la fuente única de nominales y de aire

**Files:**
- Create: `engine/src/presupuesto.rs`
- Modify: `engine/src/lib.rs` (añadir `pub mod presupuesto;` en orden
  alfabético, entre `plantilla` y `recall`)
- Test: inline, `#[cfg(test)] mod tests` al final de
  `engine/src/presupuesto.rs`

**Interfaces:**
- Consumes: nada. Es la hoja del árbol de dependencias.
- Produces:
  - `pub const TIERS: [&str; 3] = ["core", "stable", "log"];`
  - `pub const EXCLUIDOS: [&str; 3] = [".superpowers", "archive", "docs"];`
  - `pub struct Presupuestos { pub core: i64, pub stable: i64, pub log: i64 }`
  - `pub const NOMINALES: Presupuestos` — **la única fuente de 8500/12500/0**
  - `pub fn para_tier(&self, tier: &str) -> Option<i64>` — `None` = tier ilegal
  - `pub fn tiene_aire(techo: i64, tamano: i64) -> bool`
  - `pub fn techo_minimo(tamano: i64) -> i64`
  - `pub fn objetivo_poda(techo: i64) -> i64`
  - `pub enum Clase { Infractora { presupuesto: i64 }, Waived { presupuesto: i64 }, SinAire { presupuesto: i64 }, Ok }`
  - `pub fn clasifica(tier_presupuesto: i64, override_max: Option<i64>, tamano: i64) -> Clase`

**Por qué `clasifica` es una función pura y separada del walk.** En kbx la
clasificación vive dentro del `switch` del `WalkDir` de `budget.Run`, y
`doctor.budgetExceededFindings` la reimplementa. Sacarla a una función pura es
lo que hace **imposible** que las dos vuelvan a divergir: `budget` y el check
`budget_exceeded` de `lint` llaman a la misma. La Corrección 4 explica por qué
esto no es refactor gratuito.

- [ ] **Step 1: Escribir los tests que fallan**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn los_nominales_son_los_citables() {
        assert_eq!(NOMINALES.core, 8500);
        assert_eq!(NOMINALES.stable, 12500);
        assert_eq!(NOMINALES.log, 0);
    }

    #[test]
    fn para_tier_devuelve_none_solo_en_tier_ilegal() {
        assert_eq!(NOMINALES.para_tier("core"), Some(8500));
        assert_eq!(NOMINALES.para_tier("stable"), Some(12500));
        // log es LEGAL con presupuesto 0 (= ilimitado). None es "no es un
        // tier", que es otra cosa: la distinción decide si la nota va a
        // `notier` o si simplemente no tiene techo.
        assert_eq!(NOMINALES.para_tier("log"), Some(0));
        assert_eq!(NOMINALES.para_tier(""), None);
        assert_eq!(NOMINALES.para_tier("Core"), None);
        assert_eq!(NOMINALES.para_tier("co\u{a0}re"), None);
    }

    #[test]
    fn el_aire_es_el_115_por_ciento_en_aritmetica_entera() {
        // fe46443, internal/ratchet/check.go: techo*100 >= tamaño*115.
        assert!(tiene_aire(12500, 10869));
        assert!(!tiene_aire(12500, 10870));
        assert!(tiene_aire(8500, 7391));
        assert!(!tiene_aire(8500, 7392));
    }

    #[test]
    fn el_objetivo_de_poda_es_el_mayor_tamano_que_pasa_la_guarda() {
        assert_eq!(objetivo_poda(12500), 10869);
        assert_eq!(objetivo_poda(8500), 7391);
        // El invariante que une las dos funciones: el objetivo siempre pasa,
        // y un byte más nunca. Es lo que hace accionable el mensaje.
        for techo in [1000, 8500, 12500, 99999] {
            assert!(tiene_aire(techo, objetivo_poda(techo)));
            assert!(!tiene_aire(techo, objetivo_poda(techo) + 1));
        }
    }

    #[test]
    fn el_techo_minimo_redondea_hacia_arriba() {
        // ceil(tamaño*1.15), fe46443 minCeilingFor.
        assert_eq!(techo_minimo(10000), 11500);
        assert_eq!(techo_minimo(1), 2);
        for tamano in [1, 999, 10000, 12345] {
            assert!(tiene_aire(techo_minimo(tamano), tamano));
        }
    }

    #[test]
    fn el_overflow_no_panica_y_degrada_hacia_rojo() {
        // Rust panica en overflow en debug donde Go hace wraparound silencioso,
        // así que hay que elegir explícitamente. Y la elección importa: con
        // `saturating_mul` los dos lados colapsan a i64::MAX, la comparación
        // sale `true` y el gate diría QUE HAY AIRE. Con `checked_mul`, no.
        assert!(!tiene_aire(i64::MAX, i64::MAX));
        assert!(!tiene_aire(i64::MAX, i64::MAX / 2));
        assert!(!tiene_aire(i64::MAX / 50, i64::MAX / 50));
        // Y no revienta: devuelve el techo imposible, que `tiene_aire` rechaza.
        assert_eq!(techo_minimo(i64::MAX), i64::MAX);
        assert!(!tiene_aire(techo_minimo(i64::MAX), i64::MAX));
    }

    #[test]
    fn clasifica_prefiere_infractora_sobre_falta_de_aire() {
        // fe46443, TestRunPrefersOffenderOverAirWarning: el switch es
        // ordenado, no un conjunto de condiciones independientes.
        assert!(matches!(
            clasifica(12500, None, 13000),
            Clase::Infractora { presupuesto: 12500 }
        ));
    }

    #[test]
    fn clasifica_usa_el_override_como_presupuesto_efectivo() {
        // Sobre el override: infractora contra el override, no contra el tier.
        assert!(matches!(
            clasifica(12500, Some(9000), 9500),
            Clase::Infractora { presupuesto: 9000 }
        ));
        // Bajo el override pero sobre el nominal: waived, y el presupuesto que
        // se reporta es el OVERRIDE (kbx: Budget: override).
        assert!(matches!(
            clasifica(12500, Some(20000), 13000),
            Clase::Waived { presupuesto: 20000 }
        ));
    }

    #[test]
    fn clasifica_no_avisa_de_aire_a_quien_declaro_techo() {
        // fe46443, TestRunSkipsNotesThatDeclareAWaiver: con override
        // declarado, el aire lo vigila el ratchet, no budget.
        assert!(matches!(clasifica(12500, Some(20000), 12000), Clase::Ok));
        // Sin override y a ras: SinAire, reportando el NOMINAL del tier.
        assert!(matches!(
            clasifica(12500, None, 12000),
            Clase::SinAire { presupuesto: 12500 }
        ));
    }

    #[test]
    fn clasifica_no_toca_los_tiers_ilimitados() {
        // fe46443, TestRunSkipsUnlimitedTiers: presupuesto 0 = ilimitado, ni
        // infractora ni aviso de aire. Pero un override SÍ dispara sobre log,
        // que es la regresión que doctor se había dejado (Corrección 4).
        assert!(matches!(clasifica(0, None, 999_999), Clase::Ok));
        assert!(matches!(
            clasifica(0, Some(50), 100),
            Clase::Infractora { presupuesto: 50 }
        ));
    }

    #[test]
    fn los_presupuestos_son_de_verdad_un_parametro() {
        // `analiza` y los checks toman `Presupuestos` en vez de leer NOMINALES
        // directamente. Si ningún test usa otros valores, el parámetro es
        // config especulativa y sobra: este lo ejerce, como hace el Go.
        let apretados = Presupuestos { core: 100, stable: 200, log: 50 };
        assert_eq!(apretados.para_tier("core"), Some(100));
        assert!(matches!(
            clasifica(apretados.para_tier("log").unwrap(), None, 60),
            Clase::Infractora { presupuesto: 50 }
        ));
        // Con el nominal real, esa misma nota de 60 bytes no es nada.
        assert!(matches!(clasifica(NOMINALES.log, None, 60), Clase::Ok));
    }

    #[test]
    fn el_borde_del_presupuesto_es_estrictamente_mayor() {
        // fe46443, TestRunBudgetCoreBoundaryStrictGreaterThan: 8500 exacto NO
        // es infractora.
        assert!(!matches!(clasifica(8500, None, 8500), Clase::Infractora { .. }));
        assert!(matches!(
            clasifica(8500, None, 8501),
            Clase::Infractora { presupuesto: 8500 }
        ));
    }
}
```

- [ ] **Step 2: Correr los tests y verlos fallar**

Run: `cd engine && cargo test --lib presupuesto`
Expected: FAIL de compilación — `file not found for module presupuesto` primero,
y tras crear el fichero vacío, `cannot find value NOMINALES in this scope` y
compañía. Que falle por *no existir*, no por assert.

- [ ] **Step 3: Implementación**

```rust
//! La fuente ÚNICA de los presupuestos por tier y de la aritmética de aire.
//!
//! En kbx esto vivía repartido: los nominales en dos sitios de producción sin
//! nada que los enlazara (`cmd/kbx/budget.go` y `doctor.DefaultBudgetOptions`),
//! y la clasificación de una nota reimplementada en `doctor` además de en
//! `budget`, con una regresión dirigida vigilando que no divergieran. Aquí hay
//! una sola de cada, y `budget` y `lint` las consumen las dos.

/// Los tres tiers legales, en orden de reporte.
pub const TIERS: [&str; 3] = ["core", "stable", "log"];

/// Directorios excluidos del scope de la KB, verbatim de kbx
/// (`budget.DefaultExclude` y `doctor.DefaultBudgetOptions`, que coinciden).
pub const EXCLUIDOS: [&str; 3] = [".superpowers", "archive", "docs"];

/// Sellar o bajar un techo exige que quede un 15% por encima del tamaño. Un
/// techo a ras es un mordisco programado para mañana: medido el 2026-08-17, un
/// sello con 1,1% de aire mordió al día siguiente, a mitad de cierre.
const FACTOR_AIRE_PCT: i64 = 115;

#[derive(Clone, Copy, Debug)]
pub struct Presupuestos {
    pub core: i64,
    pub stable: i64,
    pub log: i64,
}

/// Los nominales citables. `log` es 0 = ilimitado.
pub const NOMINALES: Presupuestos = Presupuestos { core: 8500, stable: 12500, log: 0 };

impl Presupuestos {
    /// `None` significa "no es un tier legal" (⇒ la nota va a `notier`).
    /// `Some(0)` significa "tier legal, sin techo". Confundirlos es lo que
    /// hacía que `doctor` se saltara las notas `log` con override declarado.
    pub fn para_tier(&self, tier: &str) -> Option<i64> {
        match tier {
            "core" => Some(self.core),
            "stable" => Some(self.stable),
            "log" => Some(self.log),
            _ => None,
        }
    }
}

/// ¿Deja `techo` el margen exigido sobre `tamano`? Aritmética entera a
/// propósito: sin redondeo de float en un gate.
///
/// **`checked_mul`, no `saturating_mul`.** Rust panica en overflow en debug
/// donde Go hace wraparound silencioso, así que hay que elegir; y saturar es la
/// elección equivocada: los dos lados colapsarían a `i64::MAX` y la comparación
/// diría **que hay aire**, o sea degradación hacia verde en la única función
/// del módulo que existe para gatear. Con `checked_mul`, lo que no cabe no
/// tiene aire.
pub fn tiene_aire(techo: i64, tamano: i64) -> bool {
    match (techo.checked_mul(100), tamano.checked_mul(FACTOR_AIRE_PCT)) {
        (Some(izq), Some(der)) => izq >= der,
        _ => false,
    }
}

/// El techo más bajo que pasaría la guarda para una nota de este tamaño:
/// ceil(tamano * 1,15). Si no cabe en un `i64`, no hay techo legal: se devuelve
/// `i64::MAX`, que `tiene_aire` seguirá rechazando.
pub fn techo_minimo(tamano: i64) -> i64 {
    tamano
        .checked_mul(FACTOR_AIRE_PCT)
        .and_then(|v| v.checked_add(99))
        .map(|v| v / 100)
        .unwrap_or(i64::MAX)
}

/// El mayor tamaño admisible bajo este techo. Es el número accionable del
/// mensaje: "poda a N".
///
/// Aquí `saturating_mul` sí vale, y la diferencia con `tiene_aire` es la que
/// importa: esto **no gatea**, es una cifra para un mensaje humano. Saturar
/// produce un número absurdo en un caso imposible; saturar en la guarda
/// produciría un verde.
pub fn objetivo_poda(techo: i64) -> i64 {
    techo.saturating_mul(100) / FACTOR_AIRE_PCT
}

/// En qué cae una nota. El orden de las ramas importa: una infractora nunca se
/// reporta además como falta de aire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clase {
    /// Rebasa su presupuesto efectivo (el override si lo declaró, si no el
    /// nominal del tier). `presupuesto` es el efectivo.
    Infractora { presupuesto: i64 },
    /// Rebasaría el nominal del tier, pero su override la salva.
    /// `presupuesto` es el override.
    Waived { presupuesto: i64 },
    /// Sin override declarado y a menos del 15% de aire de su nominal.
    /// Informativo: nunca gatea. `presupuesto` es el nominal del tier.
    SinAire { presupuesto: i64 },
    Ok,
}

/// La clasificación de UNA nota. Pura a propósito: es el punto donde `budget` y
/// el check `budget_exceeded` de `lint` se encuentran, de modo que no puedan
/// divergir aunque alguien edite solo uno de los dos.
pub fn clasifica(tier_presupuesto: i64, override_max: Option<i64>, tamano: i64) -> Clase {
    let efectivo = override_max.unwrap_or(tier_presupuesto);
    if efectivo > 0 && tamano > efectivo {
        return Clase::Infractora { presupuesto: efectivo };
    }
    if let Some(max) = override_max {
        if tier_presupuesto > 0 && tamano > tier_presupuesto {
            return Clase::Waived { presupuesto: max };
        }
        // Con techo declarado, el aire lo vigila el ratchet, no budget.
        return Clase::Ok;
    }
    if tier_presupuesto > 0 && !tiene_aire(tier_presupuesto, tamano) {
        return Clase::SinAire { presupuesto: tier_presupuesto };
    }
    Clase::Ok
}
```

Y registrar el módulo en `engine/src/lib.rs`, entre `plantilla` y `recall`:

```rust
pub mod presupuesto;
```

- [ ] **Step 4: Correr los tests y verlos pasar**

Run: `cd engine && cargo test --lib presupuesto`
Expected: PASS, 12 tests.
Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
Expected: sin salida, exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/presupuesto.rs engine/src/lib.rs
git commit -m "feat(presupuesto): una sola fuente de nominales y de aire, no tres"
```

---

## Task 4: `walker::walk_kb_excluyendo` — el walk parametrizable

**Files:**
- Modify: `engine/src/walker.rs`
- Test: `engine/tests/walker.rs`

**Interfaces:**
- Consumes: nada.
- Produces:
  - `pub fn walk_kb_excluyendo(raiz: &Path, excluidos: &[&str]) -> Result<(Vec<(String, String)>, Vec<String>)>`
    — un solo walk que devuelve `(subdirectorios, notas)`. Los subdirectorios
    como `(basename, ruta_relativa)`, sin filtrar por exclusión. Las notas como
    rutas **relativas** a `raiz`, con `/` como separador, ordenadas y ya
    filtradas.
  - `pub fn walk_notas(raiz: &Path, excluidos: &[&str]) -> Result<Vec<String>>`
    — azúcar para quien solo quiere las notas.
  - `pub fn excluida(rel: &str, excluidos: &[&str]) -> bool`
  - `pub fn es_md(nombre: &str) -> bool`

`String` y no `PathBuf` a propósito: las rutas relativas con `/` son la moneda
de cambio de todos los reportes de este plan, y convertirlas solo en el punto de
tocar disco evita que un `\` de Windows se cuele en el JSON. `excluida` y
`es_md` son `pub` para que **no haya una segunda copia** en `lint.rs`: el propio
`doctor.go:31-37` lleva escrito «do not reintroduce a second copy of it».

**Por qué no se toca `walk_kb`.** Lo consume el indexer, con exclusiones
distintas (`.claude`, `.omc`, `.superpowers`; **incluye** `archive/`) y filtro
`.md` case-sensitive. Cambiarlo sería un cambio de comportamiento del indexer
fuera del scope de G4b. Las dos funciones nuevas conviven con ella y devuelven
rutas **relativas**, no absolutas, porque todos los consumidores de G4b
reportan rutas relativas a la raíz de la KB.

**Las tres decisiones que este walk encarna** (A4, A5, y los dotdirs):

- Exclusión por **primer segmento** de la ruta relativa (A4), no por basename
  en cada nivel.
- Extensión `.md` **case-insensitive** (A5).
- **Todo dotdir se salta**, a cualquier nivel — lo que hace `doctor.walkTree` y
  no hace `budget.Run`. Cierra de paso un agujero de kbx: `budget` recorre
  `.git/` porque su lista de exclusión no lo contiene.

- [ ] **Step 1: Escribir los tests que fallan**

Añadir a `engine/tests/walker.rs`:

Nota: `engine/tests/walker.rs` ya tiene `use std::fs;` en su cabecera. No lo
añadas otra vez — sería `E0252`.

```rust
/// Árbol que ejercita las tres decisiones a la vez.
fn arbol_excluible() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    for sub in ["archive", "docs", "projects", "projects/archive", ".git", ".superpowers"] {
        fs::create_dir_all(p.join(sub)).unwrap();
    }
    fs::write(p.join("raiz.md"), "x").unwrap();
    fs::write(p.join("MAYUS.MD"), "x").unwrap();
    fs::write(p.join("archive/vieja.md"), "x").unwrap();
    fs::write(p.join("docs/doc.md"), "x").unwrap();
    fs::write(p.join(".superpowers/sp.md"), "x").unwrap();
    fs::write(p.join(".git/hook.md"), "x").unwrap();
    fs::write(p.join("projects/vivo.md"), "x").unwrap();
    fs::write(p.join("projects/archive/anidada.md"), "x").unwrap();
    dir
}

#[test]
fn excluye_por_primer_segmento_y_no_por_basename() {
    // A4: `archive/` de raíz se excluye; `projects/archive/` NO. La semántica
    // de basename-en-cada-nivel de kbx budget la saltaría en silencio; se elige
    // la que chequea (degradación hacia rojo). Medido el 2026-09-04: en la KB
    // real no hay ningún dir de estos anidado, así que hoy no cambia números.
    let dir = arbol_excluible();
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(rutas.contains(&"projects/archive/anidada.md".to_string()));
    assert!(!rutas.iter().any(|r| r.starts_with("archive/")));
    assert!(!rutas.iter().any(|r| r.starts_with("docs/")));
}

#[test]
fn el_filtro_md_es_case_insensitive() {
    // A5: un NOTA.MD es una nota. kbx doctor la trataba como no-nota y la
    // sacaba del gate en silencio.
    let dir = arbol_excluible();
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(rutas.contains(&"MAYUS.MD".to_string()));
}

#[test]
fn ningun_dotdir_entra_aunque_no_este_en_la_lista() {
    // .git no está en EXCLUIDOS y kbx budget lo recorre. Aquí no.
    let dir = arbol_excluible();
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(!rutas.iter().any(|r| r.starts_with(".git/")));
    assert!(!rutas.iter().any(|r| r.starts_with(".superpowers/")));
}

#[test]
fn las_rutas_son_relativas_con_barra_y_ordenadas() {
    let dir = arbol_excluible();
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let mut ordenadas = rutas.clone();
    ordenadas.sort();
    assert_eq!(rutas, ordenadas);
    assert!(rutas.iter().all(|r| !r.contains('\\')));
    assert!(rutas.iter().all(|r| !r.starts_with('/')));
}

#[test]
fn el_walk_da_los_subdirectorios_sin_filtrar_por_exclusion() {
    let dir = arbol_excluible();
    let (dirs, _) =
        exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(dirs.contains(&("archive".to_string(), "archive".to_string())));
    assert!(dirs.contains(&("archive".to_string(), "projects/archive".to_string())));
    // Los dotdirs no se reportan ni se descienden.
    assert!(!dirs.iter().any(|(b, _)| b.starts_with('.')));
}

#[test]
fn es_md_no_panica_con_nombres_multibyte() {
    // `&nombre[len-3..]` reventaría aquí: esta KB está llena de em-dashes.
    assert!(!exo::walker::es_md("añá"));
    assert!(!exo::walker::es_md("—"));
    assert!(exo::walker::es_md("nota—larga.md"));
    assert!(exo::walker::es_md("NOTA.MD"));
    assert!(!exo::walker::es_md(".md"));
}
```

- [ ] **Step 2: Correr los tests y verlos fallar**

Run: `cd engine && cargo test --test walker`
Expected: FAIL de compilación — `cannot find function walk_kb_excluyendo in
module exo::walker`. Los 4 tests de G4a siguen pasando.

- [ ] **Step 3: Implementación**

Añadir a `engine/src/walker.rs` (dejando `walk_kb` y `visita` intactos):

```rust
/// ¿Cae `rel` bajo un directorio excluido? Compara el **primer segmento** de la
/// ruta relativa, no el basename de cada nivel (A4). Ante un
/// `projects/archive/` la semántica de basename lo saltaría en silencio; esta
/// lo chequea. Tolera la barra final en las entradas de la lista, como kbx.
///
/// `pub` y en un solo sitio: `doctor.go:31-37` lleva escrito «do not
/// reintroduce a second copy of it», y una segunda copia con tolerancias que
/// derivan es el modo de fallo que esa nota documenta.
pub fn excluida(rel: &str, excluidos: &[&str]) -> bool {
    let seg = rel.split('/').next().unwrap_or(rel);
    excluidos.iter().any(|e| seg == e.trim_end_matches('/'))
}

/// A5: case-insensitive. Un `NOTA.MD` es una nota.
///
/// `to_ascii_lowercase` y no un slice de los últimos 3 bytes: `&nombre[len-3..]`
/// **panica** si ese corte cae a mitad de un carácter multibyte, y esta KB está
/// llena de nombres con em-dash y acentos. Un `.png` mal nombrado no debe
/// tumbar el verbo.
pub fn es_md(nombre: &str) -> bool {
    // `len() > 3` en bytes: un fichero llamado exactamente `.md` no es una nota
    // (y además es un dotfile, que ya se salta antes).
    nombre.len() > 3 && nombre.to_ascii_lowercase().ends_with(".md")
}

/// Recorre `raiz` **una vez** y devuelve `(subdirectorios, notas)`:
///
/// - subdirectorios como `(basename, ruta relativa)`, sin filtrar por
///   exclusión — `duplicate_dir` y `budget` la aplican en momentos distintos.
/// - notas: rutas **relativas** (separador `/`, ordenadas) de los `.md` dentro
///   del scope, ya filtradas por `excluidos`.
///
/// Un solo walk porque `lint` necesita las dos cosas y recorrer el árbol dos
/// veces no lo hace ni más simple ni más general.
///
/// Se salta **todo** lo que empiece por `.`, a cualquier nivel: cierra el
/// agujero de `kbx budget`, que recorre `.git/`, `.claude/` y `.omc/` porque su
/// lista de exclusión no los contiene. Es la divergencia 6 del pre-registro.
pub fn walk_kb_excluyendo(
    raiz: &Path,
    excluidos: &[&str],
) -> Result<(Vec<(String, String)>, Vec<String>)> {
    let mut ficheros = Vec::new();
    let mut dirs = Vec::new();
    recorre(raiz, raiz, &mut dirs, &mut ficheros)?;
    ficheros.retain(|rel| !excluida(rel, excluidos));
    ficheros.sort();
    dirs.sort();
    Ok((dirs, ficheros))
}

/// Azúcar para los llamantes que solo quieren las notas.
pub fn walk_notas(raiz: &Path, excluidos: &[&str]) -> Result<Vec<String>> {
    Ok(walk_kb_excluyendo(raiz, excluidos)?.1)
}

/// Lee una nota como texto, tolerando bytes inválidos.
///
/// `read` + `from_utf8_lossy`, **no `read_to_string`**: este es el idioma de la
/// casa (`objetivos.rs` lo hace igual y por lo mismo) y es lo que hace Go, que
/// trabaja sobre bytes. Con `read_to_string`, un solo byte inválido en una nota
/// abortaría el verbo ENTERO con exit 1 — un gate que se apaga por una nota
/// mal codificada en vez de clasificarla.
pub fn lee_nota(ruta: &Path) -> Result<String> {
    let crudo =
        std::fs::read(ruta).with_context(|| format!("leer nota {}", ruta.display()))?;
    Ok(String::from_utf8_lossy(&crudo).into_owned())
}

fn recorre(
    raiz: &Path,
    dir: &Path,
    dirs: &mut Vec<(String, String)>,
    ficheros: &mut Vec<String>,
) -> Result<()> {
    let entradas =
        std::fs::read_dir(dir).with_context(|| format!("leer directorio {}", dir.display()))?;
    for entrada in entradas {
        let entrada = entrada.with_context(|| format!("entrada de {}", dir.display()))?;
        let ruta = entrada.path();
        let nombre = entrada.file_name().to_string_lossy().into_owned();
        if nombre.starts_with('.') {
            continue;
        }
        let rel = ruta
            .strip_prefix(raiz)
            .with_context(|| format!("{} fuera de la raíz {}", ruta.display(), raiz.display()))?
            .to_string_lossy()
            .replace('\\', "/");
        let tipo = entrada
            .file_type()
            .with_context(|| format!("file_type de {}", ruta.display()))?;
        if tipo.is_dir() {
            dirs.push((nombre, rel));
            recorre(raiz, &ruta, dirs, ficheros)?;
        } else if tipo.is_file() && es_md(&nombre) {
            ficheros.push(rel);
        }
    }
    Ok(())
}
```

Nota para el implementador: `walk_kb_excluyendo` devuelve `Vec<String>`, no
`Vec<PathBuf>` — las rutas relativas con `/` son la moneda de cambio de todos
los reportes de este plan, y convertirlas a `PathBuf` solo en el punto de tocar
disco evita que `\` se cuele en el JSON en Windows.

- [ ] **Step 4: Correr los tests y verlos pasar**

Run: `cd engine && cargo test --test walker`
Expected: PASS, 10 tests (4 de G4a + 6 nuevos).
Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
Expected: sin salida, exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/walker.rs engine/tests/walker.rs
git commit -m "feat(walker): walk parametrizable, y el .git que kbx budget recorria"
```

---

## Task 5: `presupuesto::analiza` — el informe de `budget`

**Files:**
- Modify: `engine/src/presupuesto.rs`
- Test: `engine/tests/presupuesto.rs` (Create)

**Interfaces:**
- Consumes: `walker::walk_kb_excluyendo`, `frontmatter::{tier, valor, budget_max}`,
  `presupuesto::{Presupuestos, clasifica, objetivo_poda, TIERS}`.
- Produces:
  - `pub struct FilaTier { tier: String, notas: usize, bytes: i64, presupuesto: i64, delta: i64, excedido: bool }`
    con renames `notes` / `budget` / `exceeded`.
  - `pub struct Infractora { ruta: String, tier: String, tamano_bytes: i64, presupuesto: i64 }`
    con renames `path` / `size_bytes` / `budget`.
  - `pub struct Informe { tiers, infractoras, waived, sin_aire, notier }`
    con renames `offenders` / `no_air`.
  - `pub fn excedido(&self) -> bool` — infractoras o notier no vacíos.
  - `pub fn analiza(kb: &Path, presupuestos: Presupuestos, excluidos: &[&str]) -> Result<Informe>`

- [ ] **Step 1: Escribir los tests que fallan**

Crear `engine/tests/presupuesto.rs`:

```rust
use exo::presupuesto::{self, NOMINALES};
use std::fs;

fn nota(dir: &std::path::Path, rel: &str, tier: &str, extra: &str, relleno: usize) {
    let ruta = dir.join(rel);
    fs::create_dir_all(ruta.parent().unwrap()).unwrap();
    let cabecera = format!("---\ntier: {tier}\n{extra}---\n");
    let cuerpo = "x".repeat(relleno.saturating_sub(cabecera.len()));
    fs::write(&ruta, format!("{cabecera}{cuerpo}")).unwrap();
}

#[test]
fn las_filas_de_tier_van_en_orden_y_log_es_ilimitado() {
    let dir = tempfile::tempdir().unwrap();
    nota(dir.path(), "core/a.md", "core", "", 100);
    nota(dir.path(), "log/b.md", "log", "", 99_000);
    let informe = presupuesto::analiza(dir.path(), NOMINALES, &presupuesto::EXCLUIDOS).unwrap();
    let tiers: Vec<&str> = informe.tiers.iter().map(|f| f.tier.as_str()).collect();
    assert_eq!(tiers, vec!["core", "stable", "log"]);
    let log = &informe.tiers[2];
    assert_eq!(log.presupuesto, 0);
    // delta solo tiene sentido con presupuesto: con 0 se reporta 0, no
    // "bytes - 0", que sería el tamaño entero disfrazado de exceso.
    assert_eq!(log.delta, 0);
    assert!(!log.excedido);
}

#[test]
fn una_nota_sin_tier_legal_va_a_notier_y_gatea() {
    let dir = tempfile::tempdir().unwrap();
    nota(dir.path(), "x.md", "inventado", "", 50);
    nota(dir.path(), "y.md", "co\u{a0}re", "", 50); // A1: el NBSP la deja ilegal
    let informe = presupuesto::analiza(dir.path(), NOMINALES, &presupuesto::EXCLUIDOS).unwrap();
    assert_eq!(informe.notier, vec!["x.md".to_string(), "y.md".to_string()]);
    assert!(informe.excedido());
}

#[test]
fn las_infractoras_van_por_exceso_descendente_y_desempatan_por_ruta() {
    let dir = tempfile::tempdir().unwrap();
    nota(dir.path(), "core/mucho.md", "core", "", 12_000);
    nota(dir.path(), "core/poco.md", "core", "", 9_000);
    nota(dir.path(), "core/b.md", "core", "", 9_000);
    let informe = presupuesto::analiza(dir.path(), NOMINALES, &presupuesto::EXCLUIDOS).unwrap();
    let rutas: Vec<&str> = informe.infractoras.iter().map(|o| o.ruta.as_str()).collect();
    assert_eq!(rutas, vec!["core/mucho.md", "core/b.md", "core/poco.md"]);
}

#[test]
fn el_waiver_saca_de_infractoras_y_reporta_el_override() {
    let dir = tempfile::tempdir().unwrap();
    nota(dir.path(), "core/w.md", "core", "kbx_budget_max: 20000\n", 12_000);
    let informe = presupuesto::analiza(dir.path(), NOMINALES, &presupuesto::EXCLUIDOS).unwrap();
    assert!(informe.infractoras.is_empty());
    assert_eq!(informe.waived.len(), 1);
    assert_eq!(informe.waived[0].presupuesto, 20_000);
    assert!(!informe.excedido());
}

#[test]
fn el_aire_es_informativo_y_no_gatea() {
    let dir = tempfile::tempdir().unwrap();
    // stable: objetivo de poda 10.869. 12.000 está a ras pero no excede.
    nota(dir.path(), "s.md", "stable", "", 12_000);
    let informe = presupuesto::analiza(dir.path(), NOMINALES, &presupuesto::EXCLUIDOS).unwrap();
    assert_eq!(informe.sin_aire.len(), 1);
    assert_eq!(informe.sin_aire[0].presupuesto, 12_500);
    assert!(informe.infractoras.is_empty());
    assert!(!informe.excedido(), "el aire NUNCA mueve el exit code");
}

#[test]
fn las_listas_vacias_serializan_como_array_y_nunca_como_null() {
    let dir = tempfile::tempdir().unwrap();
    nota(dir.path(), "core/ok.md", "core", "", 100);
    let informe = presupuesto::analiza(dir.path(), NOMINALES, &presupuesto::EXCLUIDOS).unwrap();
    let v = serde_json::to_value(&informe).unwrap();
    for k in ["offenders", "waived", "no_air", "notier"] {
        assert!(v[k].is_array(), "{k} no es array: {}", v[k]);
    }
}

#[test]
fn las_claves_del_informe_estan_en_ingles() {
    let dir = tempfile::tempdir().unwrap();
    nota(dir.path(), "core/big.md", "core", "", 12_000);
    let informe = presupuesto::analiza(dir.path(), NOMINALES, &presupuesto::EXCLUIDOS).unwrap();
    let v = serde_json::to_value(&informe).unwrap();
    let o = &v["offenders"][0];
    for k in ["path", "tier", "size_bytes", "budget"] {
        assert!(o.get(k).is_some(), "falta {k}: {o}");
    }
    for k in ["ruta", "tamano_bytes", "presupuesto"] {
        assert!(o.get(k).is_none(), "sobrevive la clave española {k}");
    }
    assert_eq!(o["path"], "core/big.md");
    assert_eq!(o["budget"], 8500);
    let f = &v["tiers"][0];
    for k in ["tier", "notes", "bytes", "budget", "delta", "exceeded"] {
        assert!(f.get(k).is_some(), "falta {k} en tiers[]: {f}");
    }
}

#[test]
fn el_tamano_sale_del_disco_y_no_del_indice() {
    // El invariante que hace comparable el informe con el de kbx: bytes del
    // fichero entero en disco, frontmatter incluido.
    let dir = tempfile::tempdir().unwrap();
    nota(dir.path(), "core/a.md", "core", "", 500);
    let informe = presupuesto::analiza(dir.path(), NOMINALES, &presupuesto::EXCLUIDOS).unwrap();
    let en_disco = fs::metadata(dir.path().join("core/a.md")).unwrap().len() as i64;
    assert_eq!(informe.tiers[0].bytes, en_disco);
}
```

- [ ] **Step 2: Correr los tests y verlos fallar**

Run: `cd engine && cargo test --test presupuesto`
Expected: FAIL de compilación — `cannot find function analiza in module
exo::presupuesto`, y los structs `Informe`/`Infractora`/`FilaTier` inexistentes.

- [ ] **Step 3: Implementación**

Añadir a `engine/src/presupuesto.rs`:

```rust
use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
pub struct FilaTier {
    pub tier: String,
    #[serde(rename = "notes")]
    pub notas: usize,
    pub bytes: i64,
    #[serde(rename = "budget")]
    pub presupuesto: i64,
    pub delta: i64,
    #[serde(rename = "exceeded")]
    pub excedido: bool,
}

#[derive(Serialize)]
pub struct Infractora {
    #[serde(rename = "path")]
    pub ruta: String,
    pub tier: String,
    #[serde(rename = "size_bytes")]
    pub tamano_bytes: i64,
    #[serde(rename = "budget")]
    pub presupuesto: i64,
}

#[derive(Serialize)]
pub struct Informe {
    pub tiers: Vec<FilaTier>,
    #[serde(rename = "offenders")]
    pub infractoras: Vec<Infractora>,
    pub waived: Vec<Infractora>,
    #[serde(rename = "no_air")]
    pub sin_aire: Vec<Infractora>,
    pub notier: Vec<String>,
}

impl Informe {
    /// Qué mueve el exit code: infractoras y notier. El aire y los waivers,
    /// jamás — el aire es un aviso y el waiver es una declaración aceptada.
    pub fn excedido(&self) -> bool {
        !self.infractoras.is_empty() || !self.notier.is_empty()
    }
}

/// Recorre la KB y clasifica cada nota. El tamaño sale de `metadata().len()`
/// —bytes del fichero entero en disco, frontmatter incluido—, nunca del índice:
/// es lo que hace el informe comparable con el de kbx y lo que hace que el
/// gate mida lo que de verdad pesa la nota en el repo.
pub fn analiza(kb: &Path, presupuestos: Presupuestos, excluidos: &[&str]) -> Result<Informe> {
    let rutas = crate::walker::walk_notas(kb, excluidos)?;

    let mut bytes_por_tier = std::collections::HashMap::new();
    let mut notas_por_tier = std::collections::HashMap::new();
    let mut excedido_por_tier = std::collections::HashMap::new();
    let (mut infractoras, mut waived, mut sin_aire) = (Vec::new(), Vec::new(), Vec::new());
    let mut notier = Vec::new();

    for rel in rutas {
        let absoluta = kb.join(&rel);
        let contenido = crate::walker::lee_nota(&absoluta)?;
        let tamano = std::fs::metadata(&absoluta)
            .with_context(|| format!("stat de {}", absoluta.display()))?
            .len() as i64;

        let tier = crate::frontmatter::tier(&contenido);
        let Some(tier_presupuesto) = presupuestos.para_tier(&tier) else {
            notier.push(rel);
            continue;
        };

        *bytes_por_tier.entry(tier.clone()).or_insert(0i64) += tamano;
        *notas_por_tier.entry(tier.clone()).or_insert(0usize) += 1;

        let fila = |presupuesto| Infractora {
            ruta: rel.clone(),
            tier: tier.clone(),
            tamano_bytes: tamano,
            presupuesto,
        };
        match clasifica(tier_presupuesto, crate::frontmatter::budget_max(&contenido), tamano) {
            Clase::Infractora { presupuesto } => {
                excedido_por_tier.insert(tier.clone(), true);
                infractoras.push(fila(presupuesto));
            }
            Clase::Waived { presupuesto } => waived.push(fila(presupuesto)),
            Clase::SinAire { presupuesto } => sin_aire.push(fila(presupuesto)),
            Clase::Ok => {}
        }
    }

    let tiers = TIERS
        .iter()
        .map(|t| {
            let presupuesto = presupuestos.para_tier(t).unwrap_or(0);
            let bytes = *bytes_por_tier.get(*t).unwrap_or(&0);
            FilaTier {
                tier: (*t).to_string(),
                notas: *notas_por_tier.get(*t).unwrap_or(&0),
                bytes,
                presupuesto,
                // Sin presupuesto no hay exceso: reportar `bytes - 0` sería el
                // tamaño entero disfrazado de delta.
                delta: if presupuesto > 0 { bytes - presupuesto } else { 0 },
                excedido: *excedido_por_tier.get(*t).unwrap_or(&false),
            }
        })
        .collect();

    // Infractoras y waived por exceso descendente, desempatando por ruta;
    // el aire por tamaño descendente. Determinista: el informe se difea.
    let por_exceso = |a: &Infractora, b: &Infractora| {
        (b.tamano_bytes - b.presupuesto)
            .cmp(&(a.tamano_bytes - a.presupuesto))
            .then_with(|| a.ruta.cmp(&b.ruta))
    };
    infractoras.sort_by(por_exceso);
    waived.sort_by(por_exceso);
    sin_aire.sort_by(|a, b| {
        b.tamano_bytes.cmp(&a.tamano_bytes).then_with(|| a.ruta.cmp(&b.ruta))
    });
    notier.sort();

    Ok(Informe { tiers, infractoras, waived, sin_aire, notier })
}
```

- [ ] **Step 4: Correr los tests y verlos pasar**

Run: `cd engine && cargo test --test presupuesto`
Expected: PASS, 8 tests.
Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
Expected: sin salida, exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/presupuesto.rs engine/tests/presupuesto.rs
git commit -m "feat(presupuesto): el informe de budget, con el delta que no miente en log"
```

---

## Task 6: `lint.rs` — los tres checks de disco

**Files:**
- Create: `engine/src/lint.rs`
- Modify: `engine/src/lib.rs` (`pub mod lint;`, entre `inicia` y `nota`)
- Test: `engine/tests/lint.rs` (Create)

**Interfaces:**
- Consumes: `walker::{walk_kb_excluyendo, excluida, es_md}`, `frontmatter::{tier, valor}`,
  `presupuesto::{TIERS, EXCLUIDOS}`.
- Produces:
  - `pub struct Hallazgo { tipo: String, ruta: String, detalle: String }`
    con renames `type` / `path` / `detail`.
  - `pub fn dirs_duplicados(dirs: &[(String, String)], excluidos: &[&str]) -> Result<Vec<Hallazgo>>`
    — toma los directorios del walk único, no vuelve a andar el árbol.
  - `pub fn frontmatter_malo(kb: &Path, rutas: &[String], excluidos: &[&str]) -> Result<Vec<Hallazgo>>`
  - `pub fn ficheros_en_raiz(kb: &Path) -> Result<Vec<Hallazgo>>`

`lint.rs` **no define su propio `excluida`**: importa el de `walker`. Dos copias
del predicado de scope con tolerancias que derivan es el modo de fallo que
`doctor.go:31-37` documenta con un «do not reintroduce a second copy of it».

- [ ] **Step 1: Escribir los tests que fallan**

Crear `engine/tests/lint.rs`:

```rust
use exo::lint;
use std::fs;

fn kb_con(ficheros: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (rel, contenido) in ficheros {
        let ruta = dir.path().join(rel);
        fs::create_dir_all(ruta.parent().unwrap()).unwrap();
        fs::write(ruta, contenido).unwrap();
    }
    dir
}

#[test]
fn dirs_duplicados_agrupa_por_basename_bajo_padres_distintos() {
    let dir = kb_con(&[
        ("projects/notas/a.md", "---\ntier: log\n---\n"),
        ("learnings/notas/b.md", "---\ntier: log\n---\n"),
        ("core/unico/c.md", "---\ntier: core\n---\n"),
    ]);
    let (dirs, _) =
        exo::walker::walk_kb_excluyendo(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let h = lint::dirs_duplicados(&dirs, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].tipo, "duplicate_dir");
    assert_eq!(h[0].ruta, "notas");
    assert_eq!(h[0].detalle, "learnings/notas, projects/notas");
}

#[test]
fn dirs_duplicados_no_mira_dentro_de_lo_excluido() {
    let dir = kb_con(&[
        ("projects/notas/a.md", "---\ntier: log\n---\n"),
        ("archive/notas/b.md", "---\ntier: log\n---\n"),
    ]);
    let (dirs, _) =
        exo::walker::walk_kb_excluyendo(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let h = lint::dirs_duplicados(&dirs, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(h.is_empty(), "archive/ no cuenta como segundo padre: {h:?}");
}

#[test]
fn frontmatter_malo_distingue_ausencia_de_ilegalidad() {
    let dir = kb_con(&[
        ("sin.md", "# sin frontmatter\n"),
        ("ilegal.md", "---\ntier: inventado\n---\n"),
        ("nbsp.md", "---\ntier: co\u{a0}re\n---\n"),
        ("bien.md", "---\ntier: core\n---\n"),
    ]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let h = lint::frontmatter_malo(dir.path(), &rutas, &exo::presupuesto::EXCLUIDOS).unwrap();
    let por_ruta: std::collections::HashMap<_, _> =
        h.iter().map(|f| (f.ruta.as_str(), f.detalle.as_str())).collect();
    assert_eq!(por_ruta.get("sin.md"), Some(&"NOTIER"));
    assert_eq!(por_ruta.get("ilegal.md"), Some(&"tier ilegal: inventado"));
    // A1: el NBSP no se absuelve, así que la nota sale como tier ilegal.
    assert_eq!(por_ruta.get("nbsp.md"), Some(&"tier ilegal: co\u{a0}re"));
    assert!(!por_ruta.contains_key("bien.md"));
}

#[test]
fn ficheros_en_raiz_solo_mira_profundidad_cero() {
    let dir = kb_con(&[
        ("nota.md", "---\ntier: core\n---\n"),
        ("suelto.txt", "x"),
        ("projects/otro.txt", "x"),
    ]);
    fs::write(dir.path().join(".oculto"), "x").unwrap();
    let h = lint::ficheros_en_raiz(dir.path()).unwrap();
    let rutas: Vec<&str> = h.iter().map(|f| f.ruta.as_str()).collect();
    assert_eq!(rutas, vec!["suelto.txt"]);
    assert_eq!(h[0].tipo, "root_file");
}

#[test]
fn ficheros_en_raiz_ignora_la_exclusion_a_proposito() {
    // A6: la exclusión opera sobre el primer segmento de rutas de profundidad
    // >=1. En la raíz no hay segmento que excluir, así que este check no la
    // consulta — y eso es correcto, no una inconsistencia que arreglar.
    let dir = kb_con(&[("nota.md", "---\ntier: core\n---\n")]);
    fs::write(dir.path().join("docs"), "un FICHERO llamado docs, no un dir").unwrap();
    let h = lint::ficheros_en_raiz(dir.path()).unwrap();
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].ruta, "docs");
}

#[test]
fn las_claves_del_hallazgo_estan_en_ingles() {
    let dir = kb_con(&[("suelto.txt", "x")]);
    let h = lint::ficheros_en_raiz(dir.path()).unwrap();
    let v = serde_json::to_value(&h[0]).unwrap();
    for k in ["type", "path", "detail"] {
        assert!(v.get(k).is_some(), "falta {k}: {v}");
    }
    for k in ["tipo", "ruta", "detalle"] {
        assert!(v.get(k).is_none(), "sobrevive la clave española {k}");
    }
}
```

- [ ] **Step 2: Correr los tests y verlos fallar**

Run: `cd engine && cargo test --test lint`
Expected: FAIL de compilación — `cannot find module lint`, y tras crearlo vacío,
`cannot find function dirs_duplicados`.

- [ ] **Step 3: Implementación**

Crear `engine/src/lint.rs`:

```rust
//! Los seis checks de deriva de la KB, portados de `kbx doctor` en bare mode.
//!
//! Son SEIS y no siete: `schema_drift` muere aquí (A7 del plan de G4b). Existía
//! porque kbx y exo eran dos binarios contra un schema compartido; con un solo
//! binario deja de tener objeto.

use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Debug, Clone)]
pub struct Hallazgo {
    #[serde(rename = "type")]
    pub tipo: String,
    #[serde(rename = "path")]
    pub ruta: String,
    #[serde(rename = "detail")]
    pub detalle: String,
}

impl Hallazgo {
    fn nuevo(tipo: &str, ruta: impl Into<String>, detalle: impl Into<String>) -> Self {
        Hallazgo { tipo: tipo.to_string(), ruta: ruta.into(), detalle: detalle.into() }
    }
}

use crate::walker::excluida;

/// Un mismo basename de directorio colgando de dos padres distintos. Es la
/// forma en que una KB se bifurca sin que nadie lo decida.
///
/// Toma los directorios ya recorridos en vez de volver a andar el árbol: `lint`
/// hace un solo walk y reparte.
pub fn dirs_duplicados(dirs: &[(String, String)], excluidos: &[&str]) -> Result<Vec<Hallazgo>> {
    let mut grupos: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for (basename, rel) in dirs.iter().cloned() {
        if excluida(&rel, excluidos) {
            continue;
        }
        grupos.entry(basename).or_default().push(rel);
    }
    let mut hallazgos = Vec::new();
    for (basename, mut rutas) in grupos {
        if rutas.len() < 2 {
            continue;
        }
        rutas.sort();
        hallazgos.push(Hallazgo::nuevo("duplicate_dir", basename, rutas.join(", ")));
    }
    hallazgos.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    Ok(hallazgos)
}

/// Falta la clave `tier`, o su valor no es uno de los tres legales. La
/// distinción entre las dos importa: `NOTIER` es "nadie lo declaró" y
/// `tier ilegal` es "lo declaró mal", y se arreglan distinto.
pub fn frontmatter_malo(kb: &Path, rutas: &[String], excluidos: &[&str]) -> Result<Vec<Hallazgo>> {
    let mut hallazgos = Vec::new();
    for rel in rutas {
        if excluida(rel, excluidos) {
            continue;
        }
        let absoluta = kb.join(rel);
        let contenido = crate::walker::lee_nota(&absoluta)?;
        let tier = crate::frontmatter::tier(&contenido);
        let presente = crate::frontmatter::valor(&contenido, "tier").is_some();
        if !presente {
            hallazgos.push(Hallazgo::nuevo("bad_frontmatter", rel.clone(), "NOTIER"));
        } else if !crate::presupuesto::TIERS.contains(&tier.as_str()) {
            hallazgos.push(Hallazgo::nuevo(
                "bad_frontmatter",
                rel.clone(),
                format!("tier ilegal: {tier}"),
            ));
        }
    }
    hallazgos.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    Ok(hallazgos)
}

/// Ficheros no-nota en la raíz de la KB. Profundidad 0 solamente, saltando
/// dotfiles y directorios.
///
/// **No consulta la lista de exclusión, y es correcto** (A6). Ojo con el
/// porqué, porque el intuitivo es falso: `excluida("docs", EXCLUIDOS)` devuelve
/// `true` —el primer segmento de `"docs"` es `"docs"`—, así que el predicado
/// SÍ casaría. La razón es de dominio: la lista excluye directorios fuera del
/// scope de la KB, y un fichero de profundidad 0 no está dentro de ninguno.
/// Aplicárselo sería un choque accidental de nombres, y un fichero llamado
/// `docs` en la raíz dejaría de reportarse.
pub fn ficheros_en_raiz(kb: &Path) -> Result<Vec<Hallazgo>> {
    let mut hallazgos = Vec::new();
    let entradas =
        std::fs::read_dir(kb).with_context(|| format!("leer raíz de la KB {}", kb.display()))?;
    for entrada in entradas {
        let entrada = entrada.with_context(|| format!("entrada de {}", kb.display()))?;
        let nombre = entrada.file_name().to_string_lossy().into_owned();
        let tipo = entrada.file_type().with_context(|| format!("file_type de {nombre}"))?;
        if nombre.starts_with('.') || tipo.is_dir() || nombre.to_lowercase().ends_with(".md") {
            continue;
        }
        hallazgos.push(Hallazgo::nuevo("root_file", nombre, "fichero no-nota en la raíz"));
    }
    hallazgos.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    Ok(hallazgos)
}
```

Y registrar el módulo en `engine/src/lib.rs`, entre `inicia` y `nota`:

```rust
pub mod lint;
```

- [ ] **Step 4: Correr los tests y verlos pasar**

Run: `cd engine && cargo test --test lint`
Expected: PASS, 6 tests.
Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
Expected: sin salida, exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/lint.rs engine/src/lib.rs engine/tests/lint.rs
git commit -m "feat(lint): los tres checks de disco, y el root_file que no excluye a proposito"
```

---

## Task 7: `lint::huerfanas` — el SQL y la guarda que sostiene el check

**Files:**
- Modify: `engine/src/lint.rs`
- Test: `engine/tests/lint_huerfanas.rs` (Create)

**Interfaces:**
- Consumes: `rusqlite::Connection`, `frontmatter::orphan_ok`.
- Produces:
  - `pub fn huerfanas(conn: &Connection, kb: &Path, excluidos: &[&str]) -> Result<(Vec<Hallazgo>, Vec<Hallazgo>)>`
    — `(hallazgos, waived)`.

**El invariante caro de este check.** `WHERE destino_permalink IS NOT NULL` no
es limpieza defensiva. Con **una sola** arista sin resolver en el índice, un
`NOT IN` sobre un subquery que contiene `NULL` evalúa a `NULL` para **todas**
las filas y la query devuelve **cero huérfanas, en verde, con el check
apagado**. Medido en kbx: 0 sin la guarda, 7 con ella; 23 de 573 aristas de la
KB viva están sin resolver, así que la condición no es teórica. El test tiene
que **auto-invalidarse** si el fixture deja de contener una arista sin resolver:
un fixture limpio haría pasar el test con la guarda quitada, que es exactamente
el fallo que se quiere impedir.

- [ ] **Step 1: Escribir los tests que fallan**

Crear `engine/tests/lint_huerfanas.rs`:

```rust
use exo::lint;
use rusqlite::Connection;
use std::fs;

/// KB + índice con: una huérfana pura, una huérfana con waiver, una nota
/// enlazada, y —lo que hace válido el test— una arista SIN resolver.
fn kb_e_indice() -> (tempfile::TempDir, Connection) {
    let dir = tempfile::tempdir().unwrap();
    for (rel, extra) in [
        ("huerfana.md", ""),
        ("waivada.md", "kbx_orphan_ok: true\n"),
        ("hub.md", ""),
        ("enlazada.md", ""),
    ] {
        fs::write(
            dir.path().join(rel),
            format!("---\ntier: log\n{extra}---\n# {rel}\n"),
        )
        .unwrap();
    }
    let conn = exo::abre_db_en_memoria().unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    for (permalink, ruta) in [
        ("kb/huerfana", "huerfana.md"),
        ("kb/waivada", "waivada.md"),
        ("kb/hub", "hub.md"),
        ("kb/enlazada", "enlazada.md"),
    ] {
        conn.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
             VALUES (?1, ?2, ?1, 'note', 0.0, NULL)",
            rusqlite::params![permalink, ruta],
        )
        .unwrap();
    }
    conn.execute(
        "INSERT INTO aristas (origen, destino_texto, destino_permalink)
         VALUES ('kb/hub', 'enlazada', 'kb/enlazada')",
        [],
    )
    .unwrap();
    // La arista SIN resolver: es la que arma la trampa del NOT IN.
    conn.execute(
        "INSERT INTO aristas (origen, destino_texto, destino_permalink)
         VALUES ('kb/hub', 'nota que no existe', NULL)",
        [],
    )
    .unwrap();
    (dir, conn)
}

#[test]
fn el_fixture_contiene_una_arista_sin_resolver() {
    // Auto-invalidación: si esto deja de ser cierto, el test de abajo pasaría
    // igual con la guarda IS NOT NULL quitada y dejaría de probar nada.
    let (_dir, conn) = kb_e_indice();
    let sin_resolver: i64 = conn
        .query_row("SELECT COUNT(*) FROM aristas WHERE destino_permalink IS NULL", [], |r| r.get(0))
        .unwrap();
    assert!(
        sin_resolver >= 1,
        "el fixture ya no tiene aristas sin resolver: el test de huérfanas deja de falsar su guarda"
    );
}

#[test]
fn las_huerfanas_sobreviven_a_las_aristas_sin_resolver() {
    let (dir, conn) = kb_e_indice();
    let (hallazgos, _) = lint::huerfanas(&conn, dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(
        !hallazgos.is_empty(),
        "cero huérfanas con aristas sin resolver: falta la guarda IS NOT NULL"
    );
    let rutas: Vec<&str> = hallazgos.iter().map(|h| h.ruta.as_str()).collect();
    assert!(rutas.contains(&"huerfana.md"));
    assert!(!rutas.contains(&"enlazada.md"), "una nota enlazada no es huérfana");
    assert!(!rutas.contains(&"hub.md"), "el origen de una arista no es huérfano");
}

#[test]
fn el_marcador_manda_la_huerfana_a_waived_y_no_a_hallazgos() {
    let (dir, conn) = kb_e_indice();
    let (hallazgos, waived) =
        lint::huerfanas(&conn, dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(!hallazgos.iter().any(|h| h.ruta == "waivada.md"));
    assert_eq!(waived.len(), 1);
    assert_eq!(waived[0].ruta, "waivada.md");
    assert!(waived[0].detalle.contains("waived: kbx_orphan_ok"));
}

#[test]
fn una_nota_ilegible_falla_hacia_rojo() {
    // Deriva índice/disco: el marcador no se puede leer, así que no absuelve.
    let (dir, conn) = kb_e_indice();
    fs::remove_file(dir.path().join("waivada.md")).unwrap();
    let (hallazgos, waived) =
        lint::huerfanas(&conn, dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(hallazgos.iter().any(|h| h.ruta == "waivada.md"));
    assert!(waived.is_empty());
}

#[test]
fn una_huerfana_bajo_archive_no_se_reporta_ni_en_windows() {
    // Dos cosas a la vez: que la exclusión se aplica a este check (Go tiene
    // TestOrphanExcludesNoteUnderArchive), y que se aplica aunque la fila venga
    // de la DB con el separador nativo de Windows, que es como la escribe
    // `indexer::ruta_relativa`.
    let (dir, conn) = kb_e_indice();
    fs::create_dir_all(dir.path().join("archive")).unwrap();
    fs::write(dir.path().join("archive/vieja.md"), "---\ntier: log\n---\n").unwrap();
    for ruta in ["archive/vieja.md", "archive\\otra.md"] {
        conn.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
             VALUES (?1, ?2, ?1, 'note', 0.0, NULL)",
            rusqlite::params![format!("kb/{ruta}"), ruta],
        )
        .unwrap();
    }
    let (hallazgos, _) = lint::huerfanas(&conn, dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(
        !hallazgos.iter().any(|h| h.ruta.contains("archive")),
        "una nota bajo archive/ no es huérfana reportable: {hallazgos:?}"
    );
}

#[test]
fn ninguna_ruta_reportada_lleva_separador_de_windows() {
    let (dir, conn) = kb_e_indice();
    conn.execute("UPDATE notas SET ruta = 'sub\\huerfana.md' WHERE permalink = 'kb/huerfana'", [])
        .unwrap();
    let (hallazgos, _) = lint::huerfanas(&conn, dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(
        hallazgos.iter().all(|h| !h.ruta.contains('\\')),
        "un `\\` se cuela en el JSON: {hallazgos:?}"
    );
}

#[test]
fn no_hay_filtro_por_tipo_de_nota() {
    // El viejo filtro note_type='note' escondía 57 de 138 notas reales;
    // retirado en M6-04 T3. Reintroducirlo apaga el check en silencio.
    let (dir, conn) = kb_e_indice();
    conn.execute("UPDATE notas SET tipo = 'project' WHERE permalink = 'kb/huerfana'", [])
        .unwrap();
    let (hallazgos, _) = lint::huerfanas(&conn, dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(hallazgos.iter().any(|h| h.ruta == "huerfana.md"));
}
```

- [ ] **Step 2: Correr los tests y verlos fallar**

Run: `cd engine && cargo test --test lint_huerfanas`
Expected: FAIL de compilación — `cannot find function huerfanas in module
exo::lint`.

- [ ] **Step 3: Implementación**

Añadir a `engine/src/lint.rs`:

```rust
/// Notas sin ninguna relación en el índice, en ninguna de las dos direcciones.
///
/// **`AND destino_permalink IS NOT NULL` es load-bearing, no limpieza.** Con
/// una sola arista sin resolver, `NOT IN` sobre un subquery que contiene NULL
/// evalúa a NULL para TODAS las filas y la query devuelve cero huérfanas: verde,
/// en silencio, con el check apagado. Medido en kbx: 0 sin la guarda, 7 con
/// ella, sobre 23 aristas sin resolver de 573.
///
/// **Sin filtro por `tipo`.** El viejo `note_type = 'note'` decía excluir
/// assets y en realidad escondía 57 de 138 notas markdown reales. Retirado en
/// M6-04 T3 como cambio de scope deliberado.
pub fn huerfanas(
    conn: &rusqlite::Connection,
    kb: &Path,
    excluidos: &[&str],
) -> Result<(Vec<Hallazgo>, Vec<Hallazgo>)> {
    let mut stmt = conn
        .prepare(
            "SELECT ruta, permalink
             FROM notas
             WHERE permalink NOT IN (SELECT origen FROM aristas)
               AND permalink NOT IN (
                     SELECT destino_permalink FROM aristas WHERE destino_permalink IS NOT NULL
                   )
             ORDER BY ruta",
        )
        .context("preparar la query de huérfanas")?;
    let filas = stmt
        .query_map([], |f| Ok((f.get::<_, String>(0)?, f.get::<_, String>(1)?)))
        .context("consultar huérfanas")?;

    let (mut hallazgos, mut waived) = (Vec::new(), Vec::new());
    for fila in filas {
        let (ruta, permalink) = fila.context("leer fila de huérfana")?;
        // `indexer::ruta_relativa` guarda `notas.ruta` con el separador NATIVO,
        // sin normalizar. En Windows eso significa `archive\x.md`: la exclusión
        // no matchearía nunca, y la MISMA nota saldría como `archive\x.md` en
        // `orphan` y `archive/x.md` en `bad_frontmatter`. Es el único check que
        // toma rutas de la DB en vez del walk, así que normaliza aquí.
        let ruta = ruta.replace('\\', "/");
        if excluida(&ruta, excluidos) {
            continue;
        }
        // Fallo hacia rojo: una nota ilegible (deriva índice/disco) NO se
        // absuelve — el marcador que la absolvería es justo lo que no se puede
        // leer.
        let absuelta = std::fs::read(kb.join(&ruta))
            .map(|b| crate::frontmatter::orphan_ok(&String::from_utf8_lossy(&b)))
            .unwrap_or(false);
        if absuelta {
            waived.push(Hallazgo::nuevo(
                "orphan",
                ruta,
                format!("{permalink} (waived: kbx_orphan_ok)"),
            ));
        } else {
            hallazgos.push(Hallazgo::nuevo("orphan", ruta, permalink));
        }
    }
    Ok((hallazgos, waived))
}
```

- [ ] **Step 4: Correr los tests y verlos pasar**

Run: `cd engine && cargo test --test lint_huerfanas`
Expected: PASS, 7 tests.
Run: quitar a mano `AND destino_permalink IS NOT NULL` del SQL y
`cd engine && cargo test --test lint_huerfanas`
Expected: **FAIL** de `las_huerfanas_sobreviven_a_las_aristas_sin_resolver` con
"cero huérfanas con aristas sin resolver". Restaurar la guarda. Este paso no es
opcional: es lo único que demuestra que el test falsa lo que dice falsar.
Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
Expected: sin salida, exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/lint.rs engine/tests/lint_huerfanas.rs
git commit -m "feat(lint): huerfanas, con la guarda NULL que apaga el check si falta"
```

---

## Task 8: `lint` — presupuesto y deriva de prosa, sobre la implementación única

**Files:**
- Modify: `engine/src/lint.rs`
- Test: `engine/tests/lint_presupuesto.rs` (Create)

**Interfaces:**
- Consumes: `presupuesto::{Presupuestos, clasifica, Clase, TIERS}`.
- Produces:
  - `pub fn presupuesto_excedido(kb: &Path, rutas: &[String], presupuestos: Presupuestos, excluidos: &[&str]) -> Result<(Vec<Hallazgo>, Vec<Hallazgo>)>`
  - `pub fn deriva_de_prosa(kb: &Path, rutas: &[String], presupuestos: Presupuestos, excluidos: &[&str]) -> Result<Vec<Hallazgo>>`
  - `pub fn analiza(conn: &Connection, kb: &Path, presupuestos: Presupuestos, excluidos: &[&str]) -> Result<InformeLint>`
  - `pub struct InformeLint { ok: bool, hallazgos: Vec<Hallazgo>, waived: Vec<Hallazgo> }`
    con rename `findings`.

**Lo que cambia frente a kbx.** `presupuesto_excedido` **no reimplementa** el
switch: llama a `presupuesto::clasifica`, la misma función que usa `budget`. En
kbx eran dos implementaciones vigiladas por una regresión dirigida
(Corrección 4); aquí es una sola y el test lo demuestra en vez de vigilarlo.

`deriva_de_prosa` audita las **cifras citadas en la prosa** de las notas `core`
contra las que el binario aplica de verdad: el caso motivador es un
`core-index.md` diciendo "≤3.900 bytes" cuando el tool aplicaba 8500 — una cifra
fantasma que no imponía nadie. Solo notas `core` (son las que se inyectan cada
sesión, donde un número obsoleto hace daño real) y **sin waiver posible**.

- [ ] **Step 1: Escribir los tests que fallan**

Crear `engine/tests/lint_presupuesto.rs`:

```rust
use exo::lint;
use exo::presupuesto::NOMINALES;
use std::fs;

fn kb_con(ficheros: &[(&str, String)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (rel, contenido) in ficheros {
        let ruta = dir.path().join(rel);
        fs::create_dir_all(ruta.parent().unwrap()).unwrap();
        fs::write(ruta, contenido).unwrap();
    }
    dir
}

fn nota(tier: &str, extra: &str, relleno: usize) -> String {
    let cabecera = format!("---\ntier: {tier}\n{extra}---\n");
    let cuerpo = "x".repeat(relleno.saturating_sub(cabecera.len()));
    format!("{cabecera}{cuerpo}")
}

#[test]
fn el_override_dispara_aunque_el_tier_sea_ilimitado() {
    // LA regresión de paridad de kbx: `tier: log` (nominal 0) con
    // kbx_budget_max: 50 y cuerpo mayor. Un `if tierBudget <= 0 { continue }`
    // prematuro hacía que doctor callase mientras budget disparaba — dos
    // comandos discrepando sobre la misma KB. Aquí no puede pasar: los dos
    // llaman a presupuesto::clasifica.
    let dir = kb_con(&[("l.md", nota("log", "kbx_budget_max: 50\n", 400))]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let (h, w) =
        lint::presupuesto_excedido(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS)
            .unwrap();
    assert_eq!(h.len(), 1, "el override sobre un tier ilimitado tiene que disparar");
    assert!(w.is_empty());
    assert_eq!(h[0].tipo, "budget_exceeded");
}

#[test]
fn lint_y_budget_no_pueden_discrepar() {
    // El test de paridad, como lo que es: una sola implementación. Se compara
    // el conjunto de rutas que cada verbo considera infractoras.
    let dir = kb_con(&[
        ("core/grande.md", nota("core", "", 12_000)),
        ("core/ok.md", nota("core", "", 100)),
        ("core/w.md", nota("core", "kbx_budget_max: 20000\n", 12_000)),
        ("log/l.md", nota("log", "kbx_budget_max: 50\n", 400)),
    ]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let (h, w) =
        lint::presupuesto_excedido(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS)
            .unwrap();
    let informe =
        exo::presupuesto::analiza(dir.path(), NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();

    let de_lint: std::collections::BTreeSet<&str> = h.iter().map(|f| f.ruta.as_str()).collect();
    let de_budget: std::collections::BTreeSet<&str> =
        informe.infractoras.iter().map(|o| o.ruta.as_str()).collect();
    assert_eq!(de_lint, de_budget, "lint y budget discrepan sobre quién es infractora");

    let waived_lint: std::collections::BTreeSet<&str> = w.iter().map(|f| f.ruta.as_str()).collect();
    let waived_budget: std::collections::BTreeSet<&str> =
        informe.waived.iter().map(|o| o.ruta.as_str()).collect();
    assert_eq!(waived_lint, waived_budget, "lint y budget discrepan sobre los waivers");
}

#[test]
fn las_notas_sin_tier_legal_no_disparan_dos_veces() {
    // Una deriva, un tipo de hallazgo: ya las coge bad_frontmatter.
    let dir = kb_con(&[("x.md", nota("inventado", "", 99_000))]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let (h, _) =
        lint::presupuesto_excedido(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS)
            .unwrap();
    assert!(h.is_empty());
}

#[test]
fn la_deriva_de_prosa_solo_mira_notas_core() {
    // OJO con la frase: la regex exige que la cifra vaya PEGADA al tier (solo
    // `:` o espacios entre medias). "El presupuesto de core es 3.900 B" NO
    // matchea, por el `es`. Esa exigencia es deliberada —es lo que evita que
    // "el presupuesto y las 3 notas core" cuente como cita— pero convierte
    // cualquier frase de prueba mal redactada en un test que pasa sin probar
    // nada.
    let citando = |tier: &str, cifra: &str| {
        format!("---\ntier: {tier}\n---\n\nPresupuesto duro: {tier} {cifra} B.\n")
    };
    let dir = kb_con(&[
        ("core/c.md", citando("core", "3.900")),
        ("stable/s.md", citando("stable", "3.900")),
    ]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let h = lint::deriva_de_prosa(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS)
        .unwrap();
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].ruta, "core/c.md");
    assert_eq!(h[0].tipo, "budget_prose_drift");
    assert_eq!(h[0].detalle, "cita core 3900B, el tool aplica 8500B");
}

#[test]
fn la_deriva_de_prosa_acepta_el_separador_de_miles_y_la_cifra_correcta() {
    let dir = kb_con(&[(
        "c.md",
        "---\ntier: core\n---\n\nPresupuesto: core 8.500 bytes, stable 12500 B.\n".to_string(),
    )]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let h = lint::deriva_de_prosa(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS)
        .unwrap();
    assert!(h.is_empty(), "las cifras correctas no derivan: {h:?}");
}

#[test]
fn la_deriva_de_prosa_parsea_las_cuatro_grafias() {
    // Portado de TestBudgetProseDriftParsesFigureVariants. Es el ÚNICO test que
    // guarda la regex: cuatro grafías con cuatro cifras distintas, para que un
    // miss del parser no pueda pasar por "no había deriva".
    let cuerpo = "---\ntier: core\n---\n\n\
        Presupuesto: core 1.000 B\n\
        Presupuesto: core 2000 bytes\n\
        Presupuesto: core: 3.000\n\
        Presupuesto: core   4000 B\n";
    let dir = kb_con(&[("c.md", cuerpo.to_string())]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let h = lint::deriva_de_prosa(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS)
        .unwrap();
    let citadas: Vec<&str> = h.iter().map(|f| f.detalle.as_str()).collect();
    assert_eq!(h.len(), 4, "una grafía no se parseó: {citadas:?}");
    for esperada in ["cita core 1000B", "cita core 2000B", "cita core 3000B", "cita core 4000B"] {
        assert!(citadas.iter().any(|d| d.starts_with(esperada)), "falta {esperada}: {citadas:?}");
    }
}

#[test]
fn la_deriva_de_prosa_calla_ante_una_mencion_vaga() {
    // Falsos positivos son peores que fallos aquí: un gate que grita se ignora.
    let dir = kb_con(&[(
        "c.md",
        "---\ntier: core\n---\n\nHablemos del presupuesto y de las 3 notas core.\n".to_string(),
    )]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let h = lint::deriva_de_prosa(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS)
        .unwrap();
    assert!(h.is_empty(), "mención vaga tratada como cita: {h:?}");
}

#[test]
fn el_informe_de_lint_es_ok_solo_sin_hallazgos() {
    let dir = kb_con(&[("core/ok.md", nota("core", "", 100))]);
    let conn = exo::abre_db_en_memoria().unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    let informe =
        lint::analiza(&conn, dir.path(), NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(informe.ok);
    let v = serde_json::to_value(&informe).unwrap();
    assert!(v["findings"].is_array());
    assert!(v["waived"].is_array());
    assert!(v.get("hallazgos").is_none());
}

#[test]
fn lint_no_emite_schema_drift() {
    // A7: son seis tipos, no siete.
    let dir = kb_con(&[("suelto.txt", "x".to_string())]);
    let conn = exo::abre_db_en_memoria().unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    let informe =
        lint::analiza(&conn, dir.path(), NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(!informe.ok);
    assert!(informe.hallazgos.iter().all(|h| h.tipo != "schema_drift"));
}
```

- [ ] **Step 2: Correr los tests y verlos fallar**

Run: `cd engine && cargo test --test lint_presupuesto`
Expected: FAIL de compilación — `cannot find function presupuesto_excedido`,
`deriva_de_prosa`, `analiza` en `exo::lint`.

- [ ] **Step 3: Implementación**

Añadir a `engine/src/lint.rs`:

```rust
use crate::presupuesto::{Clase, Presupuestos};

/// Notas que rebasan su presupuesto. **No reimplementa la clasificación**:
/// llama a `presupuesto::clasifica`, la misma que usa `exo budget`. En kbx eran
/// dos implementaciones (`budget.Run` y `doctor.budgetExceededFindings`)
/// vigiladas por una regresión dirigida; aquí no pueden divergir porque solo
/// hay una.
///
/// Las notas sin tier o con tier ilegal se saltan: ya disparan
/// `bad_frontmatter`. Una deriva, un tipo de hallazgo.
pub fn presupuesto_excedido(
    kb: &Path,
    rutas: &[String],
    presupuestos: Presupuestos,
    excluidos: &[&str],
) -> Result<(Vec<Hallazgo>, Vec<Hallazgo>)> {
    let (mut hallazgos, mut waived) = (Vec::new(), Vec::new());
    for rel in rutas {
        if excluida(rel, excluidos) {
            continue;
        }
        let absoluta = kb.join(rel);
        let contenido = crate::walker::lee_nota(&absoluta)?;
        let tier = crate::frontmatter::tier(&contenido);
        let Some(tier_presupuesto) = presupuestos.para_tier(&tier) else {
            continue;
        };
        let tamano = std::fs::metadata(&absoluta)
            .with_context(|| format!("stat de {}", absoluta.display()))?
            .len() as i64;
        match crate::presupuesto::clasifica(
            tier_presupuesto,
            crate::frontmatter::budget_max(&contenido),
            tamano,
        ) {
            Clase::Infractora { presupuesto } => hallazgos.push(Hallazgo::nuevo(
                "budget_exceeded",
                rel.clone(),
                format!("{tamano}B > {presupuesto}B ({tier})"),
            )),
            Clase::Waived { presupuesto } => waived.push(Hallazgo::nuevo(
                "budget_exceeded",
                rel.clone(),
                format!("{tamano}B ≤ {presupuesto}B (waived: kbx_budget_max)"),
            )),
            // El aviso de aire es de `budget`, no de `lint`: lint gatea y el
            // aire no debe gatear.
            Clase::SinAire { .. } | Clase::Ok => {}
        }
    }
    hallazgos.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    waived.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    Ok((hallazgos, waived))
}

/// Una línea habla de presupuestos si menciona "presupuesto"/"budget". Solo
/// esas se escanean: acota el check a prosa que dice estar citando un
/// presupuesto, en vez de a cualquier número que caiga cerca de un tier.
static LINEA_DE_PRESUPUESTO: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"(?i)presupuesto|budget").unwrap());

/// Un tier seguido inmediatamente de una cifra: "core 8.500 B", "stable 12500".
/// El "." como separador de miles porque la KB está escrita en castellano.
/// Exigir que la cifra vaya pegada al tier es lo que evita que la frase
/// "el presupuesto y las 3 notas core" cuente como cita.
static TIER_Y_CIFRA: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"\b(core|stable|log)\b[:\s]+([0-9]+(?:\.[0-9]{3})*)\s*(?:B\b|bytes\b)?")
        .unwrap()
});

/// Notas `core` cuya prosa cita una cifra de presupuesto que ya no es la que el
/// binario aplica. El caso motivador: un `core-index.md` diciendo "≤3.900
/// bytes" cuando el tool aplicaba 8.500 — una cifra fantasma que no imponía
/// nadie.
///
/// Solo `core`: son las notas que se inyectan cada sesión, donde un número
/// obsoleto hace daño real. Sin waiver posible: si una nota necesita citar una
/// cifra histórica a propósito, eso va en una `stable`/`log`, que este check no
/// mira.
pub fn deriva_de_prosa(
    kb: &Path,
    rutas: &[String],
    presupuestos: Presupuestos,
    excluidos: &[&str],
) -> Result<Vec<Hallazgo>> {
    let mut hallazgos = Vec::new();
    for rel in rutas {
        if excluida(rel, excluidos) {
            continue;
        }
        let absoluta = kb.join(rel);
        let contenido = crate::walker::lee_nota(&absoluta)?;
        if crate::frontmatter::tier(&contenido) != "core" {
            continue;
        }
        for linea in contenido.lines() {
            if !LINEA_DE_PRESUPUESTO.is_match(linea) {
                continue;
            }
            for c in TIER_Y_CIFRA.captures_iter(linea) {
                let tier = &c[1];
                // Una cifra que no parsea se salta: inventar un hallazgo desde
                // una línea no parseada es cómo un gate empieza a gritar y
                // acaba ignorado.
                let Ok(citada) = c[2].replace('.', "").parse::<i64>() else {
                    continue;
                };
                let aplicada = presupuestos.para_tier(tier).unwrap_or(0);
                if citada != aplicada {
                    hallazgos.push(Hallazgo::nuevo(
                        "budget_prose_drift",
                        rel.clone(),
                        format!("cita {tier} {citada}B, el tool aplica {aplicada}B"),
                    ));
                }
            }
        }
    }
    hallazgos.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    Ok(hallazgos)
}

#[derive(Serialize)]
pub struct InformeLint {
    pub ok: bool,
    #[serde(rename = "findings")]
    pub hallazgos: Vec<Hallazgo>,
    pub waived: Vec<Hallazgo>,
}

/// El pipeline de los SEIS checks, en orden fijo:
/// `duplicate_dir → orphan → bad_frontmatter → root_file → budget_exceeded →
/// budget_prose_drift`. `schema_drift` no está y no vuelve (A7).
pub fn analiza(
    conn: &rusqlite::Connection,
    kb: &Path,
    presupuestos: Presupuestos,
    excluidos: &[&str],
) -> Result<InformeLint> {
    // Un solo walk del árbol para los seis checks. `root_file` lee la raíz
    // aparte porque solo mira profundidad 0.
    let (dirs, rutas) = crate::walker::walk_kb_excluyendo(kb, excluidos)?;
    let (mut hallazgos, mut waived) = (Vec::new(), Vec::new());

    hallazgos.extend(dirs_duplicados(&dirs, excluidos)?);
    let (h, w) = huerfanas(conn, kb, excluidos)?;
    hallazgos.extend(h);
    waived.extend(w);
    hallazgos.extend(frontmatter_malo(kb, &rutas, excluidos)?);
    hallazgos.extend(ficheros_en_raiz(kb)?);
    let (h, w) = presupuesto_excedido(kb, &rutas, presupuestos, excluidos)?;
    hallazgos.extend(h);
    waived.extend(w);
    hallazgos.extend(deriva_de_prosa(kb, &rutas, presupuestos, excluidos)?);

    Ok(InformeLint { ok: hallazgos.is_empty(), hallazgos, waived })
}
```

- [ ] **Step 4: Correr los tests y verlos pasar**

Run: `cd engine && cargo test --test lint_presupuesto`
Expected: PASS, 9 tests.
Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
Expected: sin salida, exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/lint.rs engine/tests/lint_presupuesto.rs
git commit -m "feat(lint): presupuesto y prosa sobre la implementacion unica, no una copia"
```

---

## Task 9: `index_stale` — que `lint` no pueda dar verde con el índice apagado

**Files:**
- Modify: `engine/src/lint.rs`
- Test: `engine/tests/lint_indice.rs` (Create)

**Interfaces:**
- Consumes: `notas.ruta` del índice, y las rutas del walk que `analiza` ya tiene.
- Produces:
  - `pub fn indice_rancio(conn: &Connection, rutas: &[String]) -> Result<Vec<Hallazgo>>`
    — tipo de hallazgo `index_stale`.

**Por qué esto existe, y por qué no es scope creep.** El check `orphan` es el
único de los seis que lee el índice. Sobre un índice **vacío** —nadie ha corrido
`exo index`— la query devuelve cero filas, `orphan` no encuentra nada y `lint`
sale **`ok: true`, exit 0**. El instrumento no puede ver nada y responde
"limpio": es el caso de manual de *exit 0 no es evidencia de efecto*, y una
versión anterior de este plan llegaba a **codificarlo en un test como
comportamiento esperado**.

kbx tiene el mismo agujero, así que esto es una **mejora sobre el original**, no
una infidelidad del port — y por eso está declarada como divergencia 8 del
pre-registro. Es barato porque `analiza` ya tiene el walk de disco delante: solo
hay que compararlo con lo que el índice cree saber.

- [ ] **Step 1: Escribir los tests que fallan**

Crear `engine/tests/lint_indice.rs`:

```rust
use exo::lint;
use exo::presupuesto::NOMINALES;
use std::fs;

fn kb_y_conn(notas: &[&str], indexadas: &[&str]) -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempfile::tempdir().unwrap();
    for rel in notas {
        let ruta = dir.path().join(rel);
        fs::create_dir_all(ruta.parent().unwrap()).unwrap();
        fs::write(ruta, "---\ntier: log\n---\n# x\n").unwrap();
    }
    let conn = exo::abre_db_en_memoria().unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    for rel in indexadas {
        conn.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
             VALUES (?1, ?2, ?1, 'note', 0.0, NULL)",
            rusqlite::params![format!("kb/{rel}"), rel],
        )
        .unwrap();
    }
    (dir, conn)
}

#[test]
fn un_indice_vacio_sobre_una_kb_con_notas_no_puede_dar_verde() {
    // EL caso: sin `exo index`, `orphan` no ve nada y `lint` decía "ok".
    let (dir, conn) = kb_y_conn(&["a.md", "b.md"], &[]);
    let informe = lint::analiza(&conn, dir.path(), NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(!informe.ok, "índice vacío y KB con notas: no puede salir ok");
    let stale: Vec<_> = informe.hallazgos.iter().filter(|h| h.tipo == "index_stale").collect();
    assert_eq!(stale.len(), 1, "el índice vacío es UN hallazgo, no uno por nota");
    assert!(stale[0].detalle.contains("exo index"), "el detalle tiene que decir qué hacer");
}

#[test]
fn una_nota_en_disco_que_el_indice_no_conoce_es_un_hallazgo_por_nota() {
    let (dir, conn) = kb_y_conn(&["a.md", "b.md", "c.md"], &["a.md"]);
    let h = lint::indice_rancio(&conn, &["a.md".into(), "b.md".into(), "c.md".into()]).unwrap();
    let rutas: Vec<&str> = h.iter().map(|f| f.ruta.as_str()).collect();
    assert_eq!(rutas, vec!["b.md", "c.md"]);
    assert!(h.iter().all(|f| f.tipo == "index_stale"));
}

#[test]
fn un_indice_al_dia_no_dice_nada() {
    let (dir, conn) = kb_y_conn(&["a.md"], &["a.md"]);
    let h = lint::indice_rancio(&conn, &["a.md".into()]).unwrap();
    assert!(h.is_empty(), "índice al día: {h:?}");
    // Y el informe entero sale limpio.
    let informe = lint::analiza(&conn, dir.path(), NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(informe.ok, "{:?}", informe.hallazgos);
}

#[test]
fn el_separador_de_windows_del_indice_no_inventa_deriva() {
    // `notas.ruta` viene con separador nativo; si no se normaliza, en W11 toda
    // nota en subdirectorio parecería no indexada y `index_stale` gritaría
    // sobre una KB perfectamente al día.
    let (_dir, conn) = kb_y_conn(&["sub/a.md"], &[]);
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES ('kb/sub/a', 'sub\\a.md', 'a', 'note', 0.0, NULL)",
        [],
    )
    .unwrap();
    let h = lint::indice_rancio(&conn, &["sub/a.md".into()]).unwrap();
    assert!(h.is_empty(), "el `\\` del índice se leyó como nota distinta: {h:?}");
}

#[test]
fn una_kb_vacia_con_indice_vacio_no_es_deriva() {
    // Sin notas en disco no hay nada que indexar: no es un índice apagado.
    let (dir, conn) = kb_y_conn(&[], &[]);
    let informe = lint::analiza(&conn, dir.path(), NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(informe.ok, "{:?}", informe.hallazgos);
}
```

- [ ] **Step 2: Correr los tests y verlos fallar**

Run: `cd engine && cargo test --test lint_indice`
Expected: FAIL de compilación — `cannot find function indice_rancio in module
exo::lint`. Y una vez exista, `un_indice_vacio_sobre_una_kb_con_notas_no_puede_dar_verde`
fallando con `ok == true`: **ese es exactamente el fallo silencioso que la tarea
viene a matar, y hay que verlo antes de arreglarlo.**

- [ ] **Step 3: Implementación**

Añadir a `engine/src/lint.rs`:

```rust
/// Notas que están en disco y el índice no conoce.
///
/// El check `orphan` es el único de los seis que lee el índice, así que sobre un
/// índice vacío o desfasado **no encuentra nada y `lint` sale verde**: el
/// instrumento no puede ver y responde "limpio". kbx tiene el mismo agujero;
/// aquí no, porque `analiza` ya tiene el walk de disco delante y compararlos es
/// barato.
///
/// El índice **vacío** se reporta como UN hallazgo, no uno por nota: es una
/// condición del entorno con una sola acción (`exo index`), y N hallazgos
/// idénticos convierten un informe accionable en ruido.
pub fn indice_rancio(conn: &rusqlite::Connection, rutas: &[String]) -> Result<Vec<Hallazgo>> {
    let mut stmt = conn.prepare("SELECT ruta FROM notas").context("leer rutas del índice")?;
    let indexadas: std::collections::BTreeSet<String> = stmt
        .query_map([], |f| f.get::<_, String>(0))
        .context("consultar rutas del índice")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("leer fila del índice")?
        .into_iter()
        // Mismo motivo que en `huerfanas`: `notas.ruta` lleva separador nativo.
        .map(|r| r.replace('\\', "/"))
        .collect();

    if rutas.is_empty() {
        return Ok(Vec::new());
    }
    if indexadas.is_empty() {
        return Ok(vec![Hallazgo::nuevo(
            "index_stale",
            "",
            format!("{} nota(s) en disco y el índice vacío — corre `exo index`", rutas.len()),
        )]);
    }

    let mut hallazgos: Vec<Hallazgo> = rutas
        .iter()
        .filter(|r| !indexadas.contains(*r))
        .map(|r| {
            Hallazgo::nuevo("index_stale", r.clone(), "en disco y no en el índice — corre `exo index`")
        })
        .collect();
    hallazgos.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    Ok(hallazgos)
}
```

Y engancharlo en `analiza`, **al final del pipeline**, después de
`deriva_de_prosa`:

```rust
    hallazgos.extend(indice_rancio(conn, &rutas)?);
```

- [ ] **Step 4: Correr los tests y verlos pasar**

Run: `cd engine && cargo test --test lint_indice`
Expected: PASS, 5 tests.
Run: `cd engine && cargo test --test lint_presupuesto`
Expected: PASS. **Si `el_informe_de_lint_es_ok_solo_sin_hallazgos` cae, es
porque su fixture tiene una nota en disco y un índice vacío: hay que indexarla
en el fixture, no relajar el assert.**
Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
Expected: sin salida, exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/lint.rs engine/tests/lint_indice.rs engine/tests/lint_presupuesto.rs
git commit -m "feat(lint): index_stale, porque un indice apagado salia en verde"
```

---

## Task 10: `gate.rs` + cablear `exo budget` y `exo lint` — el primer exit 3

**Files:**
- Create: `engine/src/gate.rs`
- Modify: `engine/src/lib.rs` (`pub mod gate;`, entre `frontmatter` y `gitx`)
- Modify: `engine/src/main.rs`
- Modify: `engine/tests/contrato_envelope.rs`
- Test: `engine/tests/budget_lint_cli.rs` (Create)

**Interfaces:**
- Consumes: `presupuesto::analiza`, `lint::analiza`, `envelope::emite`.
- Produces:
  - `pub struct GateFallido { pub comando: &'static str, pub detalle: String }`,
    con `Display` y `std::error::Error`.
  - Subcomandos `Budget(ArgsBudget)` y `Lint(ArgsLint)` en `enum Comando`.

**Por qué un tipo nuevo y no `escritor::Rechazo`.** `Rechazo` lleva un `data()`
que `main` emite como envelope en la rama de error, porque en `write` el detalle
del rechazo **es** la respuesta. Aquí no: `budget` y `lint` ya han emitido su
informe completo por stdout antes de gatear, así que el gate solo necesita el
exit code y una línea a stderr. Meterlos en `Rechazo` obligaría a emitir un
segundo envelope o a inventar un `data()` vacío.

- [ ] **Step 1: Escribir los tests que fallan**

Crear `engine/tests/budget_lint_cli.rs`:

```rust
use std::fs;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_exo")
}

fn kb(ficheros: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (rel, contenido) in ficheros {
        let ruta = dir.path().join(rel);
        fs::create_dir_all(ruta.parent().unwrap()).unwrap();
        fs::write(ruta, contenido).unwrap();
    }
    dir
}

/// DB con el schema y las notas dadas ya indexadas. **Indexadas de verdad**:
/// desde la Task 9, una KB con notas en disco y el índice vacío emite
/// `index_stale`, así que un fixture con la DB vacía no puede aseverar "limpio".
fn db_con(dir: &std::path::Path, notas: &[&str]) -> std::path::PathBuf {
    let db = dir.join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    for rel in notas {
        conn.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
             VALUES (?1, ?2, ?1, 'note', 0.0, NULL)",
            rusqlite::params![format!("kb/{rel}"), rel],
        )
        .unwrap();
    }
    db
}

#[test]
fn budget_limpio_sale_cero_con_el_envelope_completo() {
    let dir = kb(&[("core/ok.md", "---\ntier: core\n---\nx\n")]);
    let salida = Command::new(bin())
        .args(["budget", "--json"])
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(salida.status.success(), "stderr: {}", String::from_utf8_lossy(&salida.stderr));
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(v["schema_version"], 2);
    assert_eq!(v["command"], "budget");
    assert!(v["data"]["tiers"].is_array());
    assert_eq!(v["data"]["offenders"].as_array().unwrap().len(), 0);
}

#[test]
fn budget_con_infractoras_sale_tres_y_emite_igual() {
    // El primer ejercicio de la decisión de exit 3: hallazgos != error. El
    // informe se emite ENTERO antes de gatear, porque quien lo consume necesita
    // saber QUÉ falló, no solo que falló.
    let grande = format!("---\ntier: core\n---\n{}", "x".repeat(9000));
    let dir = kb(&[("core/big.md", grande.as_str())]);
    let salida = Command::new(bin())
        .args(["budget", "--json"])
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert_eq!(salida.status.code(), Some(3), "hallazgos son 3, no 1");
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(v["data"]["offenders"][0]["path"], "core/big.md");
    assert!(String::from_utf8_lossy(&salida.stderr).contains("budget"));
}

#[test]
fn el_aire_solo_nunca_saca_del_cero() {
    // stable a ras (12.000 de 12.500, objetivo de poda 10.869): aviso, no gate.
    let ras = format!("---\ntier: stable\n---\n{}", "x".repeat(11_950));
    let dir = kb(&[("s.md", ras.as_str())]);
    let salida = Command::new(bin())
        .args(["budget", "--json"])
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(salida.status.success(), "el aire NUNCA gatea");
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(v["data"]["no_air"].as_array().unwrap().len(), 1);
}

#[test]
fn lint_con_hallazgos_sale_tres_y_lint_limpio_sale_cero() {
    // Dos notas y una arista entre ellas: si no, ambas serían huérfanas y el
    // caso "limpio" no existiría. Y las dos indexadas, o salta `index_stale`.
    let dir = kb(&[
        ("core/hub.md", "---\ntier: core\n---\n[[ok]]\n"),
        ("core/ok.md", "---\ntier: core\n---\nx\n"),
    ]);
    let db = db_con(dir.path(), &["core/hub.md", "core/ok.md"]);
    {
        let conn = exo::abre_db(&db).unwrap();
        conn.execute(
            "INSERT INTO aristas (origen, destino_texto, destino_permalink)
             VALUES ('kb/core/hub.md', 'ok', 'kb/core/ok.md')",
            [],
        )
        .unwrap();
    }
    let limpio = Command::new(bin())
        .args(["lint", "--json"])
        .arg("--kb")
        .arg(dir.path())
        .arg("--db")
        .arg(&db)
        .output()
        .unwrap();
    assert!(limpio.status.success());
    let v: serde_json::Value = serde_json::from_slice(&limpio.stdout).unwrap();
    assert_eq!(v["command"], "lint");
    assert_eq!(v["data"]["ok"], true);

    fs::write(dir.path().join("suelto.txt"), "x").unwrap();
    let sucio = Command::new(bin())
        .args(["lint", "--json"])
        .arg("--kb")
        .arg(dir.path())
        .arg("--db")
        .arg(&db)
        .output()
        .unwrap();
    assert_eq!(sucio.status.code(), Some(3));
    let v: serde_json::Value = serde_json::from_slice(&sucio.stdout).unwrap();
    assert_eq!(v["data"]["ok"], false);
    assert_eq!(v["data"]["findings"][0]["type"], "root_file");
}

#[test]
fn una_db_inexistente_es_error_uno_no_gate_tres() {
    // La distinción que justifica los dos códigos: "la KB está mal" (3) frente
    // a "el binario no pudo trabajar" (1).
    let dir = kb(&[("core/ok.md", "---\ntier: core\n---\nx\n")]);
    let salida = Command::new(bin())
        .args(["lint", "--json"])
        .arg("--kb")
        .arg(dir.path())
        .arg("--db")
        .arg(dir.path().join("no-existe.db"))
        .output()
        .unwrap();
    // Un índice que no existe no es "la KB está mal": es que el binario no
    // puede trabajar. Distinguirlo de `index_stale` —que sí es exit 3— es la
    // razón de tener dos códigos.
    assert_eq!(salida.status.code(), Some(1));
    assert!(salida.stdout.is_empty(), "un error no ensucia stdout");
}

#[test]
fn la_salida_humana_no_lleva_json() {
    let dir = kb(&[("core/ok.md", "---\ntier: core\n---\nx\n")]);
    let salida = Command::new(bin()).args(["budget"]).arg("--kb").arg(dir.path()).output().unwrap();
    assert!(salida.status.success());
    let texto = String::from_utf8_lossy(&salida.stdout);
    assert!(texto.contains("core"));
    assert!(!texto.trim_start().starts_with('{'));
}
```

Y añadir a `engine/tests/contrato_envelope.rs` el gate de claves para los dos
verbos nuevos, con la misma forma que `las_claves_de_targets_estan_en_ingles`.
El assert de **valor** no es adorno: la presencia sola no detecta un swap de
renames entre dos campos del mismo tipo.

```rust
#[test]
fn las_claves_de_budget_estan_en_ingles() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("core")).unwrap();
    std::fs::write(
        dir.path().join("core/big.md"),
        format!("---\ntier: core\n---\n{}", "x".repeat(9000)),
    )
    .unwrap();
    let informe = exo::presupuesto::analiza(
        dir.path(),
        exo::presupuesto::NOMINALES,
        &exo::presupuesto::EXCLUIDOS,
    )
    .unwrap();
    let v = serde_json::to_value(&informe).unwrap();

    for k in ["tiers", "offenders", "waived", "no_air", "notier"] {
        assert!(v.get(k).is_some(), "falta {k} en el informe: {v}");
    }
    for k in ["infractoras", "sin_aire"] {
        assert!(v.get(k).is_none(), "sobrevive la clave española {k}");
    }
    let o = &v["offenders"][0];
    assert_eq!(o["path"], "core/big.md");
    assert_eq!(o["tier"], "core");
    assert_eq!(o["budget"], 8500);
    // El swap que la presencia sola no vería: size_bytes y budget son ambos
    // enteros, así que se comprueba cuál es cuál.
    assert!(o["size_bytes"].as_i64().unwrap() > o["budget"].as_i64().unwrap());
    let f = &v["tiers"][0];
    assert_eq!(f["tier"], "core");
    assert_eq!(f["budget"], 8500);
    for k in ["notes", "bytes", "delta", "exceeded"] {
        assert!(f.get(k).is_some(), "falta {k} en tiers[]: {f}");
    }
    for k in ["notas", "presupuesto", "excedido"] {
        assert!(f.get(k).is_none(), "sobrevive la clave española {k}");
    }
}

#[test]
fn las_claves_de_lint_estan_en_ingles() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("suelto.txt"), "x").unwrap();
    let conn = exo::abre_db_en_memoria().unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    let informe = exo::lint::analiza(
        &conn,
        dir.path(),
        exo::presupuesto::NOMINALES,
        &exo::presupuesto::EXCLUIDOS,
    )
    .unwrap();
    let v = serde_json::to_value(&informe).unwrap();

    for k in ["ok", "findings", "waived"] {
        assert!(v.get(k).is_some(), "falta {k}: {v}");
    }
    assert!(v.get("hallazgos").is_none(), "sobrevive la clave española hallazgos");
    assert_eq!(v["ok"], false);
    let h = &v["findings"][0];
    for k in ["type", "path", "detail"] {
        assert!(h.get(k).is_some(), "falta {k}: {h}");
    }
    for k in ["tipo", "ruta", "detalle"] {
        assert!(h.get(k).is_none(), "sobrevive la clave española {k}");
    }
    // type y path son ambos strings: sin comprobar el valor, un swap pasaría.
    assert_eq!(h["type"], "root_file");
    assert_eq!(h["path"], "suelto.txt");
}
```

- [ ] **Step 2: Correr los tests y verlos fallar**

Run: `cd engine && cargo test --test budget_lint_cli`
Expected: FAIL — `error: unrecognized subcommand 'budget'` (clap, exit 2) en
todos los tests que invocan el binario.

- [ ] **Step 3: Implementación**

Crear `engine/src/gate.rs`:

```rust
//! El gate de dominio: "la KB está mal" no es "el binario falló".
//!
//! `escritor::Rechazo` no vale aquí: lleva un `data()` que `main` emite como
//! envelope en la rama de error, porque en `write` el detalle del rechazo ES la
//! respuesta. `budget` y `lint` ya han emitido su informe completo antes de
//! gatear, así que solo necesitan el exit code y una línea a stderr.
#[derive(Debug)]
pub struct GateFallido {
    pub comando: &'static str,
    pub detalle: String,
}

impl std::fmt::Display for GateFallido {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.comando, self.detalle)
    }
}

impl std::error::Error for GateFallido {}
```

En `engine/src/main.rs`:

1. Args y variantes del subcomando:

```rust
#[derive(clap::Args)]
struct ArgsBudget {
    #[arg(long)]
    kb: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct ArgsLint {
    #[arg(long)]
    db: Option<PathBuf>,
    #[arg(long)]
    kb: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}
```

Añadir `Budget(ArgsBudget)` y `Lint(ArgsLint)` a `enum Comando`, las dos ramas
a `quiere_json` (que es exhaustivo a propósito y no compilará sin ellas) y a
`ejecuta`.

2. Los dos comandos:

```rust
fn budget_cmd(args: ArgsBudget) -> Result<()> {
    let kb = resuelve_kb(args.kb)?;
    let informe = exo::presupuesto::analiza(
        &kb,
        exo::presupuesto::NOMINALES,
        &exo::presupuesto::EXCLUIDOS,
    )?;

    if args.json {
        envelope::emite("budget", serde_json::to_value(&informe)?);
    } else {
        for fila in &informe.tiers {
            println!(
                "{}\tnotas={}\tbytes={}\tpresupuesto={}\tdelta={}",
                fila.tier, fila.notas, fila.bytes, fila.presupuesto, fila.delta
            );
        }
        for o in &informe.infractoras {
            println!("offender: {} ({}) {}/{} bytes", o.ruta, o.tier, o.tamano_bytes, o.presupuesto);
        }
        for w in &informe.waived {
            println!("waived: {} ({}) {}/{} bytes", w.ruta, w.tier, w.tamano_bytes, w.presupuesto);
        }
        for na in &informe.sin_aire {
            println!(
                "no-air: {} ({}) {}/{} bytes a ras — poda a {} para el 15% de aire",
                na.ruta,
                na.tier,
                na.tamano_bytes,
                na.presupuesto,
                exo::presupuesto::objetivo_poda(na.presupuesto)
            );
        }
        for n in &informe.notier {
            println!("notier: {n}");
        }
    }

    // El informe se emite ENTERO antes de gatear: quien lo consume necesita
    // saber QUÉ falló, no solo que falló.
    if informe.excedido() {
        return Err(exo::gate::GateFallido {
            comando: "budget",
            detalle: format!(
                "{} nota(s) sobre presupuesto, {} sin tier legal",
                informe.infractoras.len(),
                informe.notier.len()
            ),
        }
        .into());
    }
    Ok(())
}

fn lint_cmd(args: ArgsLint) -> Result<()> {
    let db_ruta = resuelve_db(args.db)?;
    if !db_ruta.exists() {
        anyhow::bail!("DB no encontrada: {} — corre `exo index` primero", db_ruta.display());
    }
    let kb = resuelve_kb(args.kb)?;
    let conn = exo::abre_db(&db_ruta)?;
    let informe = exo::lint::analiza(
        &conn,
        &kb,
        exo::presupuesto::NOMINALES,
        &exo::presupuesto::EXCLUIDOS,
    )?;

    if args.json {
        envelope::emite("lint", serde_json::to_value(&informe)?);
    } else if informe.hallazgos.is_empty() {
        println!("ok");
    } else {
        for h in &informe.hallazgos {
            println!("{}\t{}\t{}", h.tipo, h.ruta, h.detalle);
        }
    }

    if !informe.ok {
        return Err(exo::gate::GateFallido {
            comando: "lint",
            detalle: format!("{} hallazgo(s)", informe.hallazgos.len()),
        }
        .into());
    }
    Ok(())
}
```

3. En `main()`, añadir el downcast del gate nuevo **antes** del `eprintln!` de
error genérico y **después** del de `Rechazo`:

```rust
if let Some(gate) = e.downcast_ref::<exo::gate::GateFallido>() {
    // Sin envelope: el informe ya salió por stdout. Aquí solo el gate.
    eprintln!("rechazado: {gate}");
    std::process::exit(3);
}
```

- [ ] **Step 4: Correr los tests y verlos pasar**

Run: `cd engine && cargo test --test budget_lint_cli`
Expected: PASS, 6 tests.
Run: `cd engine && cargo test --test contrato_envelope`
Expected: PASS, los de G4a más los 2 nuevos.
Run: `cd engine && cargo test`
Expected: PASS, toda la suite.
Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
Expected: sin salida, exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/gate.rs engine/src/lib.rs engine/src/main.rs \
        engine/tests/budget_lint_cli.rs engine/tests/contrato_envelope.rs
git commit -m "feat(cli): exo budget y exo lint, y el exit 3 que G4a solo declaro"
```

---

## Task 11: `targets` distingue "KB sin git" de "fallo de git" (A2)

**Files:**
- Modify: `engine/src/gitx.rs`
- Modify: `engine/src/objetivos.rs`
- Test: `engine/tests/targets_cli.rs`

**Interfaces:**
- Consumes: `std::process::Command` (git como subproceso, nunca `git2`).
- Produces: `pub fn es_repo_git(dir: &Path) -> Result<bool>` en `gitx`.

**Lo que NO cambia.** El fail-loud por fichero (invariante 6) se queda intacto:
si `git log` falla sobre una nota concreta, `busca_objetivos` sigue abortando el
resultado entero. Lo que cambia es que una **condición de la KB** deje de
disfrazarse de fallo de fichero. La asimetría declarada en G4a —disco
best-effort, git fail-loud— se conserva.

- [ ] **Step 1: Escribir los tests que fallan**

Añadir a `engine/tests/targets_cli.rs`:

```rust
#[test]
fn una_kb_sin_git_da_un_error_accionable_y_no_un_fallo_por_fichero() {
    // `exo init --from-basic-memory` crea KBs sin git. Ahí `search`, `recall` e
    // `index` funcionan y `targets` reventaba en la primera candidata con un
    // mensaje de fallo de git por fichero, que no dice qué hacer. Ahora la
    // condición se detecta una vez, antes del bucle (A2).
    let (dir, db) = kb_con_indice_sin_git();
    let salida = Command::new(bin())
        .args(["targets", "--json", "--limit", "5"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("alpha")
        .output()
        .unwrap();

    assert_eq!(salida.status.code(), Some(1), "es un error, no un gate");
    let stderr = String::from_utf8_lossy(&salida.stderr);
    assert!(
        stderr.contains("no está versionada") || stderr.contains("git init"),
        "el error tiene que nombrar la condición y la salida: {stderr}"
    );
    // Y una sola vez, no una por candidata.
    assert_eq!(stderr.matches("git init").count(), 1);
    assert!(salida.stdout.is_empty());
}

#[test]
fn es_repo_git_distingue_las_dos_condiciones() {
    let sin = tempfile::tempdir().unwrap();
    assert!(!exo::gitx::es_repo_git(sin.path()).unwrap());
    let (con, _db) = kb_con_indice();
    assert!(exo::gitx::es_repo_git(con.path()).unwrap());
}
```

Y añadir el helper `kb_con_indice_sin_git()`, copia de `kb_con_indice()` sin los
`git init`/`add`/`commit` — **quitando también** el closure que invoca `git` y
las variables de entorno que solo servían para él: si se quedan, son variables
sin usar y `clippy -D warnings` deja la tarea en rojo.

Y un tercer test para la variante que A2 dice cerrar:

```rust
#[test]
fn una_kb_anidada_en_un_repo_ajeno_no_cuenta_como_versionada() {
    // `--is-inside-work-tree` diría true aquí, `git log` resolvería sin error y
    // last_commit saldría vacío para TODAS las notas, en silencio. Es el caso
    // real de un $HOME con los dotfiles versionados.
    let (repo, _db) = kb_con_indice();
    let anidada = repo.path().join("kb-dentro");
    std::fs::create_dir_all(&anidada).unwrap();
    assert!(!exo::gitx::es_repo_git(&anidada).unwrap());
}
```

- [ ] **Step 2: Correr los tests y verlos fallar**

Run: `cd engine && cargo test --test targets_cli`
Expected: FAIL — `cannot find function es_repo_git in module exo::gitx`, y el
primer test fallando con un stderr que habla de `git log` sobre un fichero, no
de la KB.

- [ ] **Step 3: Implementación**

En `engine/src/gitx.rs`:

```rust
/// ¿Es `dir` **la raíz** de un work tree de git? Es una **condición de la KB**,
/// no un fallo: `exo init --from-basic-memory` crea KBs sin versionar, donde
/// `search`, `recall` e `index` funcionan perfectamente. Distinguirla de un
/// fallo de git por fichero es lo que convierte un mensaje inútil en uno
/// accionable (A2 del plan de G4b).
///
/// **La raíz, no "dentro de algún repo".** `--is-inside-work-tree` devuelve
/// `true` para una KB anidada en un repo ajeno —un `$HOME` con los dotfiles
/// versionados—, y ahí `git log` sobre cada nota resuelve sin error y devuelve
/// cadena vacía: `last_commit` sale vacío para TODAS las notas, en silencio.
/// Comparar con `--show-toplevel` cierra esa puerta.
pub fn es_repo_git(dir: &Path) -> Result<bool> {
    let salida = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .with_context(|| format!("invocar git en {}", dir.display()))?;
    if !salida.status.success() {
        return Ok(false);
    }
    let raiz = String::from_utf8_lossy(&salida.stdout).trim().to_string();
    if raiz.is_empty() {
        return Ok(false);
    }
    // `canonicalize` en los dos lados: git devuelve la ruta con `/` y resuelve
    // symlinks, y en Windows además difiere en el prefijo de unidad.
    let (a, b) = (std::fs::canonicalize(dir), std::fs::canonicalize(&raiz));
    Ok(matches!((a, b), (Ok(a), Ok(b)) if a == b))
}
```

En `engine/src/objetivos.rs`, al principio de `busca_objetivos`, antes del bucle
de candidatas:

```rust
// Una vez, no una por candidata: el fail-loud de `ultimo_commit` es correcto
// para un fallo de git sobre una nota, pero una KB sin versionar no es eso.
if !crate::gitx::es_repo_git(kb)? {
    anyhow::bail!(
        "la KB {} no está versionada —o cuelga de un repo ajeno, que para esto es \
         lo mismo— y `targets` necesita git para last_commit. Corre `git init` en \
         la raíz de la KB, o usa `exo search` / `exo recall`, que no lo necesitan",
        kb.display()
    );
}
```

- [ ] **Step 4: Correr los tests y verlos pasar**

Run: `cd engine && cargo test --test targets_cli`
Expected: PASS, todos los de G4a más los 2 nuevos.
Run: `cd engine && cargo test`
Expected: PASS, toda la suite.
Run: `cd engine && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings`
Expected: sin salida, exit 0.

- [ ] **Step 5: Commit**

```bash
git add engine/src/gitx.rs engine/src/objetivos.rs engine/tests/targets_cli.rs
git commit -m "fix(targets): una KB sin git es una condicion, no un fallo por fichero"
```

---

## Task 12: anotar en el backlog lo que G4b deja declarado

**Files:**
- Modify: `docs/backlog.md`

**Interfaces:**
- Consumes: el §Residuo declarado de este plan.
- Produces: entradas de backlog para lo que no se arregla aquí.

**Por qué es una tarea y no una frase en el Residuo.** La versión anterior de
este plan decía «anotarlo en `docs/backlog.md`» y ninguna tarea lo hacía: un
residuo declarado solo en el plan de la ola que lo generó es un residuo que se
pierde en cuanto la rama se mergea.

- [ ] **Step 1: Añadir las cuatro entradas**

En `docs/backlog.md`, con el formato de las entradas existentes:

1. **`walker::walk_kb` frente a `walk_kb_excluyendo`**: la primera es
   case-sensitive en `.md` y recorre `.git/`; la segunda no. Conviven en el
   mismo módulo desde G4b. La consume el indexer, así que alinearlas es un
   cambio de comportamiento del índice, no una limpieza.
2. **`indexer::ruta_relativa` guarda `notas.ruta` con separador nativo.** G4b lo
   normaliza al leer en dos sitios; la causa sigue en el indexer y el próximo
   consumidor de `notas.ruta` vuelve a tropezar en Windows.
3. **Punto ciego de `budget_prose_drift`**: la regex exige la cifra pegada al
   tier, y `core/doctrina-agentes.md:54` («`core` (… 8.500 B …)») es invisible
   para el check. Medido el 2026-09-04.
4. **La campaña de evicción de la KB está descalibrada** (A3): el censo y el
   objetivo de poda salen de la fórmula huérfana de `f0d0564`. Rehacer con
   `exo budget` y los umbrales canónicos (10.869 stable / 7.391 core).

- [ ] **Step 2: Verificar que no se duplican**

Run: `grep -n "walk_kb\|ruta_relativa\|prose_drift\|10.625" docs/backlog.md`
Expected: cada tema una sola vez. Si ya había una entrada del mismo tema, se
edita en vez de añadir una segunda.

- [ ] **Step 3: Commit**

```bash
git add docs/backlog.md
git commit -m "docs(backlog): lo que G4b declara y no arregla"
```

---

## Residuo declarado

Lo que este plan **no** arregla, dicho en vez de disimulado:

- **El gate de paridad no se corre aquí.** No hay toolchain Go en W11
  (`go: command not found`, medido el 2026-09-04). El pre-registro (Task 1)
  queda escrito y el gate es **lo primero de la máquina Linux**, junto con el de
  `targets` que G4a dejó pendiente. Los dos comparten binario de kbx: compilar
  `fe46443` una vez sirve para ambos.
- **La campaña de presupuestos de la KB está descalibrada** (A3). El censo de
  "19 de 58 notas stable" y el objetivo de 10.625 salen de la fórmula huérfana.
  Rehacerlo con `exo budget` cuando exista. **Este plan no toca la KB.**
- **El repo kbx sigue divergido y esto no lo arregla.** `main` local (`f0d0564`)
  va 1 por delante y 18 por detrás de `origin/main`, con conflicto en
  `budget.go` que ahora sabemos que es **semántico**, no textual (Corrección 1).
  Con A3 adjudicado, la resolución natural es abandonar `f0d0564`: no aporta
  nada que `fe46443` no tenga mejor. Decidir en la máquina Linux si se reconcilia
  o se deja morir cuando `exo` sustituya a `kbx`.
- **`walker::walk_kb` sigue siendo case-sensitive y recorriendo `.git/`.** Lo
  consume el indexer y cambiarlo es un cambio de comportamiento fuera de scope.
  La función nueva de G4b no tiene ninguno de los dos problemas, así que la
  divergencia entre las dos vive dentro del mismo módulo. Se anota en
  `docs/backlog.md` (Task 12), porque es exactamente la clase de asimetría que
  dentro de seis meses se lee como un bug.
- **`indexer::ruta_relativa` sigue guardando `notas.ruta` con separador
  nativo.** G4b lo normaliza **al leer**, en `huerfanas` e `indice_rancio`, que
  son los dos únicos consumidores de rutas de la DB en este plan. La causa está
  en el indexer y no se toca aquí: cualquier consumidor futuro de `notas.ruta`
  vuelve a tropezar. Va al backlog.
- **`budget_prose_drift` tiene un punto ciego heredado y ahora medido**: la
  regex exige que la cifra vaya pegada al tier, así que en la KB real
  `core/doctrina-agentes.md:54` cita «`core` (… 8.500 B …)» y es **invisible**,
  mientras que `core-index.md:19` («core 8.500») sí se ve. Es el precio
  deliberado de no tener falsos positivos, pero no estaba declarado. Se declara;
  ampliar la regex es trabajo para cuando una cita mal formada haga daño de
  verdad.
- **`kb-demo` sigue como nombre de KB en 8 ficheros de test.** Ninguna tarea de
  G4b los toca, así que no se aprovecha para limpiarlo. Sigue en el backlog.
- **`deriva_de_prosa` no audita `no_air`.** Si una nota `core` cita el objetivo
  de poda (10.869 / 7.391), el check no lo compara con nada: solo conoce los
  nominales. Es una omisión heredada de kbx y se declara; ampliarlo es trabajo
  para cuando alguna nota cite esas cifras, no antes.
- **La deprecación de Node 20 en el CI** (`actions/checkout@v4`,
  `actions/cache@v4`) sigue avisando en cada run. No rompe nada; es deuda para
  G5b.

---

## Los planes que siguen

**G4c — `ratchet` + cutover.** Alcance ya fijado al final del plan de G4a. Con
G4b hecho, hereda `presupuesto.rs` entero: `tiene_aire`, `techo_minimo` y
`objetivo_poda` son exactamente la aritmética que la guarda de aire del ratchet
necesita, y ya están construidas y testeadas aquí. Las trampas inventariadas
—`git show HEAD:./<path>` con el `./` explícito, `BTreeMap` en vez de `HashMap`
para renames, `--no-renames` explícito, las claves del sello como `String` con
`/` literal, H13— siguen vigentes.

**G5b — release, instaladores, `doctor`.** Sin cambios de alcance por G4b, salvo
que el `doctor` de exo ya no tiene que preocuparse por `schema_drift` como tipo
de finding de `lint` (A7): si se quiere un canary de schema, es un verbo o un
flag propio, no un check de `lint`.
