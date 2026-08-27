# Verdict del consultor — plan Ola 1C (hermeticidad + KB semilla)

- **Objeto:** `docs/superpowers/plans/2026-08-27-ola1c-hermeticidad-y-kb-semilla.md` (891 líneas, sin commitear)
- **Consultor:** independiente, fresco (régimen de gates, spec `2026-07-16-framework-unificado-design.md:153`)
- **Fecha:** 2026-08-27 · máquina W11 de Paul, repo `C:\proyectos\homework\exo`, branch `main` limpio (HEAD `05df814`)
- **Veredicto:** **APROBADO CON CAMBIOS** (4 cambios exigidos, lista al final)

---

## 1. Verificación primaria (reproducida por el consultor, no aceptada del plan)

Todas las corridas se redirigieron a fichero y el exit code se leyó de `$?` directo,
sin tubería (el aviso de método del propio plan, `:214-218`).

### 1.1 Línea base con config — CONFIRMADA

```
$ cd engine && cargo test --release --no-fail-fast > baseline.txt 2>&1; echo "EXIT=$?"
EXIT=0
$ grep "^test result" baseline.txt | awk '{p+=$4; f+=$6; i+=$8} END {print "passed="p, "failed="f, "ignored="i}'
passed=169 failed=0 ignored=1
```

Plan dice `exit 0`, 169 passed (`:59`). **Cuadra.** (Hay además 1 test `ignored`
que el plan no menciona; no altera ningún gate porque 169 es el conteo de
`passed` en ambos escenarios.)

### 1.2 Runner limpio — CONFIRMADO, reparto exacto

```
$ EXO_CONFIG=/c/ruta/que/no/existe.toml cargo test --release --no-fail-fast > limpio.txt 2>&1; echo "CARGO_EXIT=$?"
CARGO_EXIT=101
$ grep "^test result" limpio.txt | awk '{p+=$4; f+=$6; i+=$8} END {print p, f, i}'
passed=108 failed=61 ignored=1
```

Reparto por suite (extraído de `limpio.txt` con awk sobre `Running`/`test result`):

| Suite | Rojos medidos | Plan (`:69-80`) |
|---|---|---|
| indexer | 19 | 19 ✓ |
| buscador | 16 (2 verdes) | 16 ✓ |
| recall_contenido | 7 | 7 ✓ |
| guarda_modelo | 5 | 5 ✓ |
| recall | 5 | 5 ✓ |
| refresca | 4 | 4 ✓ |
| cache_embeddings | 3 | 3 ✓ |
| rechazo_envelope | 1 (3 verdes) | 1 ✓ |
| write_create_permalink | 1 | 1 ✓ |
| **Total** | **61 en 9 suites** | **61 / 9** ✓ |

Causa raíz, clasificando los 61 bloques `---- <test> stdout ----` de `limpio.txt`:

- 58 bloques contienen `no encuentro la config de exo en C:/ruta/que/no/existe.toml` — **cuadra con el plan (58)**.
- De esos 58: **57** vía `leer config de embeddings` + **1** vía `embed de la query`
  (`busca_vector_desempate_determinista_por_permalink`). El plan dice **56** (`:66`).
  Off-by-one en el sub-conteo → hallazgo MENOR-1.
- 3 bloques fallan aguas abajo sin citar config (asserts de exit-code/orden tras
  morir el paso previo o el subproceso) — consistente con el diagnóstico del plan.

También confirmado el detalle de Task 1 Step 3: el test rojo de
`write_create_permalink` panica en `tests/write_create_permalink.rs:76`
(medido: `panicked at tests\write_create_permalink.rs:76:36`).

### 1.3 Diagnóstico de causa — CONFIRMADO con matiz

```
$ grep -n "config_embeddings\|min_similitud_de_config\|config::carga" src/*.rs
src/indexer.rs:99:  let cfg = config_embeddings().context("leer config de embeddings")?;
src/lib.rs:200:     let cfg = config_embeddings()?;            (Embedder::desde_config)
src/lib.rs:286:     let cfg = config_embeddings()?;            (embedder_desde_config)
src/buscador.rs:236: None => crate::min_similitud_de_config(),
```

Los cuatro sitios existen donde el plan los cita. Matiz: también leen config
global `exo::kb_desde_config()` (`src/lib.rs:87-88`) y `exo::nombre_kb()`
(`src/lib.rs:96`), pero sus llamadores son solo `src/main.rs:351,485,535` (CLI),
no las rutas de librería de las 9 suites — la afirmación del plan es correcta
**como cadena causal de los 61 rojos**, no como inventario total de lecturas de
config en producción. Y ninguna de las 9 suites llama a config directamente:
`grep` sobre `tests/*.rs` solo da comentarios (`guarda_modelo.rs:15`,
`write_create_permalink.rs:4`) y las suites de config (`config.rs`,
`config_cableado.rs`, `config_cmd.rs`, `flags.rs`), que no están entre las 9. ✓

### 1.4 Conteo de la plantilla — EL PLAN TIENE RAZÓN (12, no ~11)

Lista literal de §G3 (`specs/2026-08-26-exo-generico-design.md:468-480`), contada
a mano: `core/core-index.md`, `core/doctrina.md`, `learnings/_template.md`,
4 learnings doctrinales, `projects/_template.md`, `log/_template.md`,
`archive/log/.gitkeep`, `AGENTS.md`, `README.md` = **12 entradas**. El «~11
ficheros» de la spec (`:509`) es aproximación en prosa. El plan lo corrige bien
y lo fija con el test `son_doce_ficheros`. Las 12 entradas del `FICHEROS` del
plan (`:670-682`) coinciden una a una con el árbol de la spec.

### 1.5 Límite de bytes — CONFIRMADO

- Cap: `plugins/exo/scripts/exo-recall.sh:35` → `EXO_CAP="${EXO_RECALL_CAP:-6144}"`. ✓
- Aritmética: `python -c "print(6144*0.85)"` → `5222.4` → truncado 5222. La
  expresión del plan `(6144.0 * 0.85) as usize = 5222` y el assert literal
  `5_222` son coherentes entre sí y con la spec (`:512-514`). ✓
- Dato de contexto verificado: `wc -c kb-demo/core/core-index.md` → **5.355 B**
  — el core-index real ya excede por sí solo el límite de la semilla (5.222 B),
  lo que refuerza el porqué del gate. Pero los «5.921 B» que cita el doc-comment
  del test del plan (`:523`) son el **bloque de arranque** (core-index + punteros
  git, `docs/backlog.md:38-46`), no el fichero — hallazgo MENOR-4.

---

## 2. Hallazgos

### BLOQUEANTE-1 — Task 8 rompe `exo init --from-basic-memory` y, con `--force`, machaca la KB real

Task 8 Step 5 (`plan:818-832`) cablea `prepara_kb → create_dir_all → vuelca →
git init → escribe_config → index` **incondicionalmente**, «tras resolver
`(kb, nombre, emb)`». Pero la rama `--from-basic-memory` (`src/main.rs:378-384`,
`src/inicia.rs:17-45`) resuelve `kb` a una **KB existente y poblada** (en esta
máquina: `C:/proyectos/homework/kb-demo`, 100+ notas). Con el cableado del plan:

- **Sin `--force`:** `prepara_kb` falla («no está vacía») → el camino de
  migración documentado deja de funcionar.
- **Con `--force`** (que hoy ya es necesario si existe `~/.exo/config.toml`,
  `inicia.rs:69-74`): `vuelca` **sobreescribe** `core/core-index.md`,
  `AGENTS.md`, `README.md`… de la KB real con la semilla, y el «primer commit»
  consagra el pisotón. Pérdida de contenido real en el flujo estrella de
  migración.

Ningún test del plan cubre la interacción (los de `inicia.rs`, `plan:750-773`,
solo prueban `prepara_kb` sobre tempdirs), y el checklist de §G3 del
self-review (`plan:874-878`) no menciona `--from-basic-memory`. La spec incluye
el flag en la firma del comando (`spec:492`): es requisito de §G3 sin tarea.

**Cambio exigido:** rama explícita de adopción — con `--from-basic-memory` no se
ejecutan `prepara_kb`, `vuelca` ni `git init` + commit (la KB ya existe y ya
tiene contenido; el índice inicial sí aplica), más un test que lo fije: init
from-basic-memory sobre KB poblada **no toca ni un fichero de la KB**.

### BLOQUEANTE-2 — El ciclo red-green del gate (Task 4 Step 2) no puede ponerse rojo

`plan:411-419`:

```bash
git stash push tests/indexer.rs
bash scripts/test-hermetico.sh; echo "EXIT_ROJO=$?"      # Expected: EXIT_ROJO=1
```

En Task 4 el árbol está **limpio**: la hermetización de `tests/indexer.rs` se
commiteó en Task 2 Step 5 (`plan:289-292`). `git stash push` con pathspec sobre
un fichero sin cambios locales es un no-op («No local changes to save»): la
suite sigue hermética y el gate devuelve `EXIT_ROJO=0` contra un Expected de
`1`. El paso es inejecutable tal cual bajo la propia disciplina del plan
(Expected que no cuadra = parar). La ironía: el paso que existe para demostrar
falsabilidad es el que da el falso verde.

**Cambio exigido:** producir el rojo revirtiendo de verdad el estado commiteado,
p. ej. `git show <sha-pre-Task2>:engine/tests/indexer.rs > tests/indexer.rs`
(o comentar `mod common;` con sed), correr el gate, y restaurar con
`git checkout -- tests/indexer.rs`.

### BLOQUEANTE-3 — El e2e de Task 8 Step 6 indexa la KB temporal dentro del `~/.exo/index.db` REAL, y su Expected es inalcanzable

`plan:838-850`. Dos defectos en el mismo paso:

1. `init_cmd` escribe la config con `db_default = ~/.exo/index.db`
   (`src/main.rs:376`, sin flag `--db` en `ArgsInit`, `src/main.rs:71-89`).
   `EXO_CONFIG="$TMP/config.toml"` aísla la **config**, no la **db**: el
   «`exo index` inicial» del Step 5.6 abre el `~/.exo/index.db` de producción
   de esta máquina e indexa dentro los 12 ficheros de la semilla — y el
   indexer poda lo que no está bajo la KB activa (clave `borradas` del
   envelope). Resultado: el índice real de Paul queda reducido a la semilla
   hasta el siguiente re-index, compitiendo con el hook `Stop`. Es
   exactamente la clase de efecto colateral que el propio plan prohíbe al
   prohibir mover `~/.exo/config.toml` (`plan:387-389`).
2. `find "$TMP/kb" -type f | sort` con Expected «12 ficheros» (`plan:848`):
   tras `git init` + commit, `find` lista también decenas de ficheros de
   `.git/`. El Expected no puede cumplirse como está escrito.

**Cambio exigido:** aislar la db en el e2e (que el paso de index de `init`
honre `EXO_DB` como el resto de comandos, `src/main.rs:334`, y el e2e exporte
`EXO_DB="$TMP/index.db"`; o equivalente), y excluir `.git` del conteo
(`find "$TMP/kb" -type f -not -path '*/.git/*'`).

### MAYOR-1 — `con_config` borra `EXO_CONFIG` en vez de restaurarlo: agujero de falso verde futuro

`plan:168-171`: `set_var("EXO_CONFIG", ruta)` … `remove_var("EXO_CONFIG")`. En
el runner hermético, `EXO_CONFIG=$TMP/no-existe.toml` viene puesto **desde
fuera** para todo el proceso. Tras el primer test envuelto de un binario, el
centinela desaparece para el resto de ese proceso: cualquier test futuro de ese
binario que lea config sin el helper cae a `~/.exo/config.toml` — **verde en la
máquina de Paul, rojo en el CI limpio de G5**. Es el modo de fallo exacto que
esta fase existe para matar, reintroducido para tests futuros. (El defecto
existe ya en `tests/config_cableado.rs:22-24`; el plan escribe un helper NUEVO
y puede no heredarlo por tres líneas.)

**Cambio exigido:** guardar `std::env::var_os("EXO_CONFIG")` antes de `set_var`
y restaurar ese valor al salir (restaurar-o-borrar según hubiera).

### MENOR-1 — Sub-conteo de causa: 57, no 56

Medido: de los 58 `no encuentro la config`, 57 llegan por
`leer config de embeddings` y 1 por `embed de la query` (§1.2). El plan dice 56
(`:66`). No cambia ninguna tarea; corregir la cifra.

### MENOR-2 — `common::MODELO` como literal duplicado

`plan:141` duplica el literal en vez de `pub const MODELO: &str =
exo::MODELO_JINA_ES;` (`src/lib.rs:148`, ya `pub`). El propio comentario de
`init_cmd` (`src/main.rs:393-397`) explica por qué un literal que diverge en
silencio es un bug: aplíquese el criterio de casa.

### MENOR-3 — La justificación de la secuencia Fase 1 → Fase 2 es falsa; las fases son casi paralelas

`plan:16-19` afirma que el gate de bytes de Task 6 «se escribe con el helper de
la Fase 1, que es por lo que las fases van en este orden». Falso por el propio
plan: el test de Task 6 usa `include_str!` (`plan:535`) y su sección Interfaces
lo admite — «`common::con_config` no hace falta aquí» (`plan:510-511`). El
acoplamiento real es solo el gate final: Task 8 Step 7 corre
`scripts/test-hermetico.sh` (Task 4). Tasks 5-7 y casi toda la 8 pueden ir en
paralelo con la Fase 1; solo el cierre (Step 7 de Task 8) exige Fase 1
terminada. Corregir el párrafo de Architecture y, si se orquesta en paralelo,
ganar ese wall-clock.

### MENOR-4 — Atribución de los 5.921 B en el doc-comment del test

`plan:522-523` presenta 5.921 B como el estado del core-index de la KB origen.
Medido: 5.921 B es el **bloque de arranque** (core-index 5.355 B + punteros de
actividad git, `docs/backlog.md:38-50`). El dato que de verdad sostiene el gate
es que el core-index real (5.355 B) ya supera él solo el límite semilla
(5.222 B). Corregir el comentario para que no nazca mintiendo.

### MENOR-5 — El barrido de fugas clean-room es débil como gate automático; y el ejecutor de Task 6 nace con contexto contaminado

Los greps (`plan:476,578`) cazan 7-8 tokens. No cazan: fechas de la historia de
Paul (la spec las prohíbe explícitamente, `spec:488-490`), nombres de proyectos
de la KB (universidad, lighthouses, spark, cliente-c, cliente-b, redmine, ocr…), ni
anécdotas sin token. Además, el ejecutor que escriba las notas recibe por hook
punteros y resúmenes de `kb-demo` en su contexto de arranque: el método
clean-room (whitelist, `framework-unificado:153`) exige escribir desde cero, y
un ejecutor que relea las notas origen está haciendo blacklist-destilado sin
saberlo. Mitigaciones baratas: (a) añadir al grep un patrón de fechas
(`20[0-9]{2}-`) y los nombres de proyectos/personas del índice de la KB; (b)
instruir en la Task 6 que el ejecutor escriba desde los NOMBRES de los
principios, sin abrir notas de `kb-demo`. El gate de revisión de Paul
(`plan:584-586`) existe y es el backstop real: con él, el riesgo residual es
aceptable — sin él, no. (Nota: `paul` como patrón da falso positivo con
«paulatino/a»; falla hacia el lado seguro.)

### MENOR-6 — «Avisar y seguir» debe cubrir también el commit fallido, no solo git ausente

`plan:826-829` degrada bien cuando `git` no está en el PATH. Pero en una
máquina fresca de tercero `git init` funciona y `git commit` **falla** sin
`user.name`/`user.email` configurados — el caso más probable en el público
objetivo de G3. Extender el warn-and-continue a cualquier fallo del paso git, y
que `"git": false` del envelope lo refleje.

### MENOR-7 — La semilla planta en la raíz lo que el lint de G4 flaggeará

`AGENTS.md` y `README.md` van a la raíz de la KB (spec §G3, heredado por el
plan), y G4 porta el check `root_file` del doctor de kbx (`spec:540-548`). El
primer `exo lint` de un usuario fresco puede flaggear la propia semilla.
Conflicto entre gates de la spec, no del plan — pero conviene declararlo ahora
(whitelist de la semilla en el check, o excepción documentada) antes de que G4
lo descubra en rojo.

### MENOR-8 — Task 2 en el límite de tamaño para un reviewer fresco

35 tests en 2 ficheros, con el matiz semántico del `min_similarity` heredado
(`plan:262-266`). Verificado que la config real de esta máquina usa 0.35 y el
modelo jina-es — idénticos a las constantes del helper, así que el wrapping no
cambia valores efectivos HOY. Aun así, partir en 2a (indexer) y 2b (buscador)
es gratis y deja diffs revisables.

---

## 3. Respuestas a las preguntas de diseño

**¿Hermetizar por entorno vs seam de producción? — Bien descartado, con la
corrección MAYOR-1.** Razones verificadas: (a) es la acción que el backlog
adjudicó literalmente (`docs/backlog.md:33-36`: «patrón EXO_CONFIG a un
tempdir, como ya hacen config.rs y config_cableado.rs»); (b) el seam **no
cubre** a las dos suites que invocan el binario compilado como subproceso
(`write_create_permalink.rs:96` pasa `.env("EXO_CONFIG", …)`;
`rechazo_envelope` hereda el entorno del proceso) — con seam habría que
construir además la maquinaria de entorno, es decir, ambas cosas; (c) el
blast radius del seam es mayor de lo que sugieren «4 call-sites»: cambiar la
firma de `indexa()` toca `main.rs` y cada call-site de test de las 9 suites de
todos modos, y `config_embeddings()` alimenta a `Embedder::desde_config` en lo
hondo de `lib.rs:200`; (d) el coste de serialización es medible y pequeño (las
suites afectadas terminan en ≤3,2 s; el mutex solo envuelve tests que tocan
config). Para G4/G5: el CI consume `test-hermetico.sh` sea cual sea el
mecanismo, y el port de kbx opera sobre la db con rutas explícitas. El seam
sigue siendo deseable como refactor de producción propio — y el trabajo de esta
fase no se tira: los tests de subproceso lo necesitarán siempre. El comentario
de `guarda_modelo.rs:15` («no es inyectable») queda obsoleto y el plan ya manda
actualizarlo (`plan:325-329`). ✓

**¿Fase 1 → Fase 2, dependencia real? — No como está contada.** Ver MENOR-3: la
justificación del plan está contradicha por su propia Task 6. Acoplamiento real:
únicamente el gate de cierre (Task 8 Step 7 consume el script de Task 4).
Tasks 5-7 son paralelizables con la Fase 1 entera.

**Right-sizing.** Task 2 en el límite (MENOR-8); Task 3 (24 tests, 5 ficheros
pequeños) aceptable; Task 6 correctamente aislada como tarea de juicio con gate
humano; Task 8 es la más cargada y además la que concentra los tres defectos
graves — tras aplicar los cambios exigidos, valorar partir el e2e (Step 6) como
verificación separada.

**¿Cubre §G3 entero? — Todo salvo una línea, y esa línea es el BLOQUEANTE-1.**
Checklist contra `spec:465-514`: árbol de 12 ✓ (T5+T6+T7) · dirs inglés/notas
español ✓ (constraints) · `{{KB_NAME}}` único placeholder, sustituido en
permalinks y títulos ✓ (T7, test `no deja placeholders`) · 5 notas reescritas
con `semilla: true` ✓ (T6) · firma con flags, `--kb` no posicional ✓ (ya en
`ArgsInit`) · paso 1 no-vacía-salvo-force ✓ (T8 `prepara_kb`) · paso 2 volcado ✓
· paso 3 git init + commit ✓ · paso 4 config sin pisar ✓ (ya en
`escribe_config`) · paso 5 index inicial ✓ · paso 6 imprime siguiente comando ✓
· `include_str!` fichero a fichero sin macro-crate ✓ (T7) · gate ≤5.222 B ✓ (T6)
· **`--from-basic-memory` en interacción con el volcado: SIN TAREA** →
BLOQUEANTE-1.

**Riesgo clean-room.** Los gates automáticos solos no bastan (MENOR-5); el gate
de revisión de Paul es el que sostiene la garantía y el plan lo hace duro («no
commitees sin su OK»). Con las dos mitigaciones baratas de MENOR-5, suficiente.

**`git init` por `Command` vs `git2`.** La conclusión es correcta; el argumento,
a medias. Bajo D4 (binarios precompilados) el toolchain C de `git2` caería en
el CI de release, no en el camino del usuario — la frase del plan (`:827-828`)
sobrevende D4. Las razones que sí sostienen la decisión: superficie de
auditoría cero-dependencias para un repo a publicar (el propio plan la invoca
en `plantilla.rs`, `:661-662`), cross-compile a 3 plataformas sin C, y los
usuarios de `cargo install` que no pasan por los binarios. El «avisar y seguir»
es correcto y consistente con la doctrina de degradación anunciada de la spec
(«binario ausente ⇒ gate apagado sin romper nada» es lo que hay que evitar, y
aquí la degradación grita por stderr y queda en `"git": false` del envelope) —
solo falta extenderlo al commit fallido (MENOR-6).

---

## 4. Veredicto

**APROBADO CON CAMBIOS.** El plan es sólido donde más caro era equivocarse: las
cifras de la línea base son exactas al test (61/9/101, reparto por suite
clavado), el diagnóstico causal es correcto, la corrección 12-vs-~11 contra la
spec es acertada, la aritmética del gate de bytes cuadra, y la elección
entorno-vs-seam está bien adjudicada. Los defectos están concentrados en los
pasos de verificación y cableado de las Tasks 4 y 8.

Cambios exigidos (sin ellos, no ejecutar):

1. **Task 8 Step 5** — rama de adopción para `--from-basic-memory`: sin
   `prepara_kb`, sin `vuelca`, sin `git init`+commit sobre una KB existente;
   test que fije que la KB poblada no se toca. (BLOQUEANTE-1)
2. **Task 4 Step 2** — sustituir `git stash push` (no-op sobre árbol limpio)
   por una reversión real del fichero commiteado para producir el rojo, con
   restauración posterior. (BLOQUEANTE-2)
3. **Task 8 Step 6** — aislar la db del e2e (p. ej. `EXO_DB` honrado por el
   index de `init` + exportado en el e2e) para no escribir en el
   `~/.exo/index.db` real, y excluir `.git/` del conteo de ficheros.
   (BLOQUEANTE-3)
4. **Task 1 Step 1** — `con_config` guarda y **restaura** el valor previo de
   `EXO_CONFIG` en vez de borrarlo. (MAYOR-1)

Recomendados (MENOR-1…8): corregir 56→57, `MODELO = exo::MODELO_JINA_ES`,
sincerar el párrafo de Architecture sobre el acoplamiento de fases (y
paralelizar si se quiere), corregir la atribución de 5.921 B, endurecer el grep
clean-room + instrucción de no abrir `kb-demo` al ejecutor de Task 6,
warn-and-continue también en commit fallido, declarar la tensión semilla-raíz vs
`root_file` de G4, y partir Task 2 en 2a/2b.

## 5. Evidencia bruta

- `baseline.txt` y `limpio.txt` en el scratchpad de la sesión del consultor
  (los agregados relevantes están pegados arriba; los exit codes se leyeron
  sin tubería).
- Ficheros leídos: el plan entero; `specs/2026-08-26-exo-generico-design.md`
  (§Decisiones, §G3, §G4); `specs/2026-07-16-framework-unificado-design.md:120-160`;
  `docs/backlog.md:20-60,195-225`; `engine/src/{lib,main,inicia,indexer,buscador,config}.rs`
  (secciones citadas); `engine/tests/{config_cableado,write_create_permalink,guarda_modelo}.rs`;
  `engine/Cargo.toml` (edition 2024, tempfile 3.14);
  `plugins/exo/scripts/exo-recall.sh:35`; `~/.exo/config.toml` (lectura, para
  verificar que las constantes del helper coinciden con los valores efectivos
  de esta máquina: 0.35 / jina-es / 768 — coinciden).
