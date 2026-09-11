# Revisión crítica externa de exo — 2026-09-04

> Informe completo de una revisión del repo hecha **sin contexto previo del
> proyecto**, a petición de Paul («analiza este proyecto y dime qué te parece,
> sé crítico»), por una sesión de Claude Fable 5.1 el 2026-09-04. Los diez
> items accionables que salieron de aquí viven en `docs/backlog.md` marcados
> «(revisión 2026-09-04)»; este documento es la **argumentación completa** que
> el backlog, por su formato de evidencia + acción, no puede contener: el
> veredicto, lo que está bien, las críticas con su porqué, las prioridades y
> los límites de la propia revisión.
>
> Re-verificado el **2026-09-11 por la mañana** contra `bf4ba7a`, y
> **corregido esa misma tarde** contra `0c81e87`, ya con G5b en `main` y la
> release `v0.1.0` publicada. La
> primera re-verificación se hizo contra un árbol que **no era `main`**:
> `bf4ba7a` es del 09-09 16:13, y `main` llevaba desde el 09-10 11:25 en
> `a0d538b` (merge de G4c), además de la rama `g5b-release-doctor` que se
> mergeó a las 11:27 del 09-11. Cuatro afirmaciones salieron falsas por ese
> desfase; van marcadas **[CORREGIDO 09-11]** donde estaban, y §8 las lista.
> Las cifras que cambiaron llevan sus fechas. No es una spec ni un plan. Su
> equivalente más cercano en el repo es `2026-08-02-foto-as-is-framework.md`:
> una foto firmable para comparar contra ella más adelante — y la lección de
> esta corrección es que este documento debería haberse comportado como esa
> foto en vez de prometer vigencia (§5.1).

## 1. Alcance y método

**Qué se leyó.** `README.md`, `docs/arquitectura.md`, `docs/backlog.md`,
`docs/instalacion.md`, `.github/workflows/ci.yml`, `.claude-plugin/marketplace.json`,
`engine/Cargo.toml`, `engine/scripts/test-hermetico.sh`; del engine, completos,
`main.rs`, `lib.rs`, `config.rs`, `envelope.rs`, `buscador.rs`, `recall.rs`,
`indexer.rs`, `escritor.rs`, `nota.rs`, y las cabeceras de `objetivos.rs`,
`gitx.rs`, `frontmatter.rs`; del plugin, `hooks/hooks.json`, `README.md`,
`agents/executor.md`, los tres hooks de memoria (`exo-recall.sh`,
`recall-inject.sh`, `exo-index.sh`), las skills `orchestrate`, `document` y
`distill`, y las cabeceras de `test-contrato-engine.sh` y `a1-gate.sh`;
la semilla `engine/kb-template/` (core-index y AGENTS.md); el historial git
completo (`git log`, `shortlog`, cadencia por día).

**Qué se ejecutó.** `cargo check --all-targets --locked` en `engine/`;
recuentos con `wc -l` sobre `git ls-files`; `grep` de `unwrap`/`expect`,
`TODO`, `unsafe`, densidad de comentarios, y de las cadenas `Paul`,
`kb-demo`, `kbx` sobre `plugins/exo/`.

**Qué se ejecutó en la corrección del 09-11 por la tarde.** Cuatro pasadas de
verificación mecánica en paralelo sobre `360175c`, cada afirmación con su
comando: greps de `Paul`/`kb-demo`/`kbx` sobre `plugins/exo/`; lectura de los
dos workflows y de `hooks.json`; `rustc --version`, `rustup show` y
`~/.rustup/toolchains`; `gh run list`, `gh run view --log-failed` y
`gh release list` sobre el workflow de release; los números de línea de §5.4 y
§5.6 contra el código actual; y el desglose de volumen documental de §5.1. Lo
que cambió de veredicto lleva su marca **[CORREGIDO 09-11]** en el sitio.

**Qué NO se hizo, y por tanto qué no puede afirmar este informe.** No se
corrió la suite de tests (exige el modelo ONNX de 0,6 GB); no se leyó la KB
privada ni `eval.jsonl`, así que ninguna cifra de retrieval se ha
reproducido; no se ejecutó ningún hook en vivo ni se midió su latencia; no se
revisaron `docs/superpowers/` (specs, planes, consultas) más allá de
muestrear sus títulos. Es una sola sesión y un solo revisor: donde este
informe dice «medido» hay un comando reproducible citado; donde dice «parece»
o «probablemente» es lectura, no medición.

## 2. Veredicto

exo es un proyecto con una **disciplina de ingeniería inusualmente alta para
ser de una sola persona**, y al mismo tiempo un proyecto que **pesa mucho más
en proceso que en producto**. El engine es sólido, pequeño y está lleno de
decisiones bien pensadas sobre cómo fallar. Lo que lo rodea —consultorías,
gates, runbooks por cutover, un plan por ola— es la mayor parte del repo en
volumen, y es donde se concentra el riesgo, pero **no por el volumen**: el
93 % de ese markdown son instantáneas fechadas que nadie tiene que mantener
(§5.1). El riesgo está en que todo convive en el mismo árbol con la misma
pretensión de vigencia, y en que las 1.939 líneas que sí deben ser verdad hoy
no tienen ningún gate que las obligue a serlo. Súmese que el plugin sigue
siendo personal a pesar de una ola entera de «exo genérico», y que la cifra
central del retrieval (48/55) se eligió y se reporta sobre las mismas 55
queries.

Nada de esto es un defecto de calidad del código. Son tres decisiones sin
tomar: para quién es exo, qué documentación caduca y cuál no, y qué evidencia
cuenta como evidencia.

## 3. Métricas

| Métrica | 2026-09-04 | 09-11 mañana (`bf4ba7a`) | 09-11 tarde, post-G5b (`360175c`) |
|---|---|---|---|
| Líneas de markdown (`docs/` + `evals/` + `reports/`) | 30.547 | 35.872 | 36.922 |
| — de ellas, **documentación viva** (§5.1) | — | — | **1.939** (4 ficheros) |
| Líneas de Rust en `engine/src/` | 5.224 | 6.682 | 9.043 |
| Ratio docs / código | 5,8 : 1 | 5,4 : 1 | 4,1 : 1 (**0,21 : 1** contando solo doc viva) |
| Líneas de tests Rust (`engine/tests/`) | 5.629 | 7.381 | 10.210 |
| Líneas de shell en `plugins/exo/scripts/` | 4.545 | 4.666 | 4.713 |
| Líneas de comentario en `engine/src/` | 1.370 (26 %) | 1.815 (27 %) | 2.435 (26 %) |
| Módulos en `engine/src/` | 19 | 22 (+`gate.rs`, `lint.rs`, `presupuesto.rs`) | 24 (+`trinquete.rs`, `doctor.rs`) |
| Binarios de test en `engine/tests/` | 28 | 33 | 44 |
| Commits / días con actividad | 320 / 15 | 351 / 17 | 402 / 19 |
| Autores (identidades git) | 1 persona, 3 emails | idem | idem |
| Pico de commits en un día | 77 (2026-07-17) | idem | idem |
| `cargo check` en esta máquina | falla: «requires rustc 1.95» sobre rustc 1.94.1 | «sigue fallando» — **falso**, ver §5.9 | **pasa**: rustc 1.98.0 contra MSRV 1.95 |

Lectura de la tendencia. Entre el 09-04 y el 09-11 por la mañana el código
creció un 28 % y la documentación un 17 %. Entre la mañana y la tarde del
09-11, G5b metió un 35 % más de código y un 3 % más de documentación, y el
ratio cayó de 5,4 a 4,1 **sin que nadie tocara una línea de documentación**.
Esa es la razón de que §5.1 se haya re-fundado: un número que se mueve un
25 % por razones ajenas a lo que dice medir no es evidencia de nada.

## 4. Lo que está bien de verdad

Esta sección importa tanto como la siguiente: las críticas de §5 son
críticas a un proyecto que ya hace bien lo difícil.

- **Pensamiento en modos de fallo, no en el camino feliz.** Transacción por
  nota en el indexer (un fallo a mitad no deja mtime nuevo con vectores
  viejos); cache de embeddings por contenido exacto; WAL + `busy_timeout` de
  5 s porque el hook de cierre y el de arranque pueden solaparse; escritura
  atómica temporal + rename en el write-path; desempate determinista por
  permalink cuando el score empata (y el comentario dice que 2 de 56 queries
  reales empatan, no es hipotético); avisos explícitos cuando el arm vector
  está inerte o parcial en vez de devolver FTS puro etiquetado `hybrid`.
- **Contratos explícitos entre capas.** Envelope JSON con `schema_version`,
  exit codes con significado (0 / 1 / 3), «los consumidores gatean por exit
  code, jamás parseando `data`», stdout exclusivo del envelope con `--json`.
  El hook `recall-inject.sh` valida la forma del envelope antes de usarlo y
  distingue «vacío» de «el engine habla otro idioma».
- **Los hooks están escritos por alguien que ha sido mordido.** El hazard
  de `UserPromptSubmit` (un exit 2 borra el prompt del usuario) está
  documentado en la cabecera y explica la ausencia de `set -e`. La
  composición del bloque va en `jq` y no en bash porque `${#var}` cuenta
  caracteres y el presupuesto es en bytes. El detach del indexado en Windows
  (`cmd start` porque Git Bash no tiene `setsid`) existe porque el `|| true`
  anterior se tragó durante meses que el índice nunca se refrescaba.
- **Reproducibilidad del modelo.** Revisión de HuggingFace pineada a un sha
  (`REVISION_JINA_ES`), con test que falla si vuelve a `main`; la clave de
  caché del CI lleva ese sha; el índice registra con qué modelo se construyó
  y aborta si la config pide otro.
- **Honestidad documental.** `docs/backlog.md` dice lo que está roto con
  cifras, fichero y línea, y el README remite a él «antes de asumir que algo
  está terminado». El CI documenta en comentarios qué endurecimientos se
  decidieron no aplicar y por qué.
- **CI real en tres sistemas operativos** con `fmt --check`, `clippy -D
  warnings`, check de MSRV y la suite completa (sin `#[ignore]` en los tests
  que descargan el modelo, con la justificación escrita: «un CI que no ejerce
  indexer ni buscador es verde sin significado»).
- **Gates pre-registrados.** El gate se redacta y commitea antes de la
  corrida; los números no se renegocian. La intención metodológica es la
  correcta, y es rara. (Que la ejecución tenga un agujero, §5.4, no invalida
  la intención.)
- **Higiene de licencias.** Atribución MIT a superpowers con copia literal
  del LICENSE y reparto skill a skill; veto AGPL explícito sobre basic-memory
  («el diseño se estudió, el código no se copió»).

## 5. Críticas

Cada una lleva hallazgo, evidencia, por qué importa, qué haría, y la entrada
del backlog que la destila (greppable por su título).

### 5.1 El proceso pesa más que el producto, pero el volumen no es la enfermedad

> **[RE-FUNDADO 09-11]** La versión original de esta sección —«el proceso se
> ha comido al producto»— apoyaba el hallazgo en el ratio docs/código. Ese
> ratio mide mal: cayó de 5,4 a 4,1 en un solo día sin que nadie tocara una
> línea de documentación, porque G5b metió código (§3). Lo que sigue es el
> mismo hallazgo re-fundado sobre el desglose que faltaba, y la conclusión
> cambia de signo.

**Hallazgo.** El repo tiene 35.892 líneas de markdown en `docs/` —las 36.922
de §3 sumando además `evals/` y `reports/`—, pero **33.469 (el 93 %) viven en
`docs/superpowers/`**, y 69 de esos 74 ficheros
llevan la fecha en el nombre: son instantáneas —specs, planes, verdicts de
gate, consultorías— que nadie tiene que mantener al día. La superficie que sí
tiene que ser verdad hoy son cuatro ficheros:

| Fichero | Líneas |
|---|---|
| `docs/backlog.md` | 1.114 |
| `docs/arquitectura.md` | 507 |
| `docs/instalacion.md` | 172 |
| `README.md` | 146 |
| **Documentación viva, total** | **1.939** |

Contra 9.043 líneas de Rust en `engine/src/`, eso son **0,21 líneas de
documentación viva por línea de código**. Por esa medida exo no está
sobredocumentado: está por debajo de lo habitual. Lo que está
sobredimensionado es el **exhaust de proceso**, que es otra cosa y tiene otro
coste: se paga al generarlo, no al mantenerlo.

**Evidencia.** `git ls-files 'docs/*.md' | xargs wc -l` → 35.892;
`git ls-files docs/superpowers | grep '\.md$' | xargs wc -l` → 33.469;
`git ls-files 'docs/**/*.md' | grep -cE '/20[0-9]{2}-[0-9]{2}-[0-9]{2}'` → 69
de 74 con fecha en el nombre; `wc -l README.md
docs/{instalacion,arquitectura,backlog}.md` → 1.939. 402 commits en 19 días,
pico de 77, un solo autor, mensajes que son ensayos. El exhaust de tareas SDD
(`.superpowers/`, 1,5 MB de briefs e informes por tarea) **no** entra en el
repo: `.gitignore:5` lo excluye y solo `fabrica/config.md` está trackeado.

**Por qué importa, y por qué no importa como parecía.** Los dos fallos
documentales que este informe ha podido probar —`arquitectura.md` afirmando
«Sin CI» durante una semana, e `instalacion.md` afirmando hoy que los
instaladores «viven en una rama sin mergear» (§5.3)— están **los dos dentro de
esas 1.939 líneas**, en los ficheros más pequeños y más leídos del repo.
Ninguno lo causó el volumen; los causó que no existe ningún check. En sentido
contrario, el plan de 105 KB de G5b entregó `exo doctor` con diez checks, dos
instaladores con verificación de sha256 y un workflow de release en dos días,
con tests: el aparato de proceso está pagando por sí mismo.

Quedan dos costes reales, distintos del que denunciaba la versión original:

1. **Coste por unidad entregada.** Un brief y un informe por tarea, commit por
   micro-paso, mensajes-ensayo. Es tiempo y tokens, y compone.
2. **Frontera rota.** Hay **17 referencias** desde los cuatro ficheros vivos
   hacia `docs/superpowers/` (`grep -rn "docs/superpowers" README.md
   docs/{arquitectura,instalacion,backlog}.md | wc -l`). Lo que se declara
   archivo está siendo citado como autoridad por lo que se declara vigente.
   Ese sí es un problema literal, y es el mismo de §5.3.

**El agujero de dogfooding.** exo *es* la herramienta que resuelve
exactamente este problema —`tier: core/stable/log`, techo por nota, trinquete
(`exo ratchet`), `exo:distill`— y el repo que la construye no se la come: **de
74 documentos en `docs/`, uno lleva `tier:` en el frontmatter**. La KB del
autor tiene presupuestos sellados y un trinquete que impide subirlos; el repo
que implementa ese trinquete no tiene ni marca de expiración. La doctrina que
el propio proyecto aplica a la memoria —lo fechado es bitácora, se llega por
enlace, no promete vigencia— resolvería §5.1 y §5.3 a la vez si se aplicara a
`docs/`.

**Qué haría.**

- (a) `tier: log` en la cabecera de todo `docs/superpowers/` y de cualquier
  documento con fecha en el nombre —**este informe incluido**—: append-only,
  fechado, explícitamente fuera del contrato de «esto es verdad hoy».
- (b) `tier: core` en los cuatro vivos, y el grep de afirmaciones frágiles de
  §5.3(c) corriendo **solo sobre ellos**: 1.939 líneas caben de sobra en un
  job de CI que hoy no existe.
- (c) Con (a) y (b), «¿cuánto proceso merece la siguiente ola?» deja de ser
  urgente: el exhaust deja de ser pasivo de mantenimiento en cuanto lleva su
  marca. La decisión de §6.2 —para quién es exo— sigue en pie, pero ya no
  depende de esta.

**Backlog.** Baja · «Decisión abierta: proceso frente a producto» — el item
sigue vivo pero **hay que reescribir su evidencia**, que es el ratio. Falta
item propio para el marcado por `tier`, que es la acción concreta que sale de
aquí. Cruza con el item preexistente «Nombres y ubicaciones»
(`docs/superpowers/` como carpeta de docs de un proyecto que quiere jubilar
superpowers).

### 5.2 «exo genérico» no es genérico

**Hallazgo.** El README describe tres olas de generalización (config propia,
fusión de plugins, hermeticidad). El engine sí las cumple: cero código de
producción lee basic-memory, la precedencia `flag > env > config` está
limpia, el error de config ausente nombra el comando que la crea. El plugin
no: sigue siendo de Paul para Paul.

**Evidencia** (medida 2026-09-04, re-verificada 2026-09-11):

- `Paul` aparece en 4 ficheros vivos del plugin: `skills/distill/SKILL.md`
  (×7, «dile a Paul», «Paul lo invoca»), `scripts/recall-inject.sh` (×2, «el
  prompt de Paul»), `scripts/git-add-all-guard.sh`, `scripts/kb-precommit.sh`.
- `kb-demo` (el nombre de la KB privada del autor) en 8 ficheros del plugin,
  dos de ellos hooks de producción (`exo-recall.sh`, `recall-inject.sh`) y
  uno el pre-commit de la KB.
- `kbx`, un binario Go externo que no está en el repo y cuyo build en Windows
  «todavía no está decidido» según la propia skill, es dependencia operativa
  de `distill` (pasos que se «saltan» si falta), de `document`, de
  `agents/executor.md` y de `kb-precommit.sh`: 7 ficheros del plugin.
- `docs/instalacion.md`: el único camino es compilar desde fuente. Rust
  ≥1.95, toolchain C (MSVC en Windows, con el detalle de `--includeRecommended`
  «ganado a pulso»), Git Bash, jq, y una descarga de 0,6 GB en la primera
  indexación. Sin binario, sin `install.sh`, sin release.

**Por qué importa.** Hoy no existe un tercero que pueda instalar y usar exo
sin leer una parte sustancial de la documentación y sin tener acceso a una
herramienta privada. La distancia entre lo que el README anuncia
(«framework», «marketplace», «id de plugin exo@exo») y lo que se puede
adoptar es grande, y es exactamente la clase de discrepancia que el propio
backlog persigue en otros sitios.

**Qué haría.** (a) Sustituir «Paul» por «el usuario» y `kb-demo` por el
nombre resuelto vía `exo config --json` en los cuatro scripts y dos skills;
(b) decidir sobre `kbx`: o se porta a exo lo que `distill` necesita (G4 ya
empezó por `targets`), o se declara dependencia opcional en `instalacion.md`
y `distill` se abstiene entera sin él; (c) la release con binario de G5 es
el prerequisito de todo lo demás y debería adelantarse a cualquier feature
nueva.

**Estado a 2026-09-11 por la tarde, tras el merge de G5b.**
**[CORREGIDO 09-11]** Dos de los cuatro apoyos de esta crítica han caducado y
uno se ha reducido a la mitad:

- **`kbx`**: eran 7 ficheros el 09-09; hoy son **5**, y el grueso ya no es
  dependencia real. `kb-precommit.sh` cortó a `exo ratchet --staged` + `exo
  budget` en el cutover de G4c (`e76e9f9`, 09-10 10:23, en `main` desde el
  09-10 11:25 — es decir, **antes** de que este informe se commiteara).
  `agents/executor.md` **no** contiene la cadena `kbx`: esa cita era falsa. Lo
  que queda es `distill` y `document` invocando `kbx` para `rotate` y `stale`,
  los dos verbos sin portar, más dos apariciones que son texto de ejemplo y
  nombre de un caso de test.
- **Instalación**: `install.sh`, `install.ps1`, `release.yml` y `exo doctor`
  existen y están en `main` (PR #6, 09-11 11:27). La recomendación (c) de esta
  sección —«la release con binario de G5 es el prerequisito de todo lo demás y
  debería adelantarse a cualquier feature nueva»— se ejecutó mientras se
  escribía este informe.
- **Sigue en pie sin cambios**: `Paul` en los mismos 4 ficheros con los mismos
  conteos (hay un quinto hit, `plugin.json:6`, que es metadata de autor y otra
  categoría); `kb-demo` en 8 ficheros, tres de ellos de producción
  (`exo-recall.sh`, `recall-inject.sh`, `kb-precommit.sh`).
- **La conclusión de fondo —«hoy no existe un tercero que pueda instalar y
  usar exo»— ha caído.** Estuvo viva hasta las 12:10 del 09-11: a esa hora la
  release `v0.1.0` quedó publicada con tres binarios y sus tres `.sha256`, y
  `install.sh` / `install.ps1` los resuelven desde `releases/latest` (§5.11).
  Lo que queda de esta crítica es la mitad de higiene —`Paul` y `kb-demo` en
  producción—, no la de adopción.

**Backlog.** Alta · «"exo genérico" sigue siendo el plugin de Paul para
Paul». Distinto del item preexistente de Baja «`kb-demo` como fixture en 8
ficheros de test»: aquí son hooks y skills de producción. El item necesita
una pasada: su evidencia sobre `kbx` ya no se sostiene.

### 5.3 Deriva documental, a pesar de la disciplina

**Hallazgo.** La documentación «derivada del código» contradice al repo, y
lo hace en plazos de horas, no de meses.

**Evidencia.**

- `docs/arquitectura.md` §7 («Qué NO está implementado») afirmaba «**Sin
  CI**: no hay `.github/`» (línea 489 el 09-04) cuando el CI existía desde
  `e378cbc` (2026-09-02), el mismo día en que se fechó el documento, y el
  README describía esa corrida en tres SO dos párrafos más arriba del enlace a
  arquitectura.md. **[CORREGIDO 09-11]** La frase ya no existe: la retiró
  `3673059` el 09-11 a las 09:49, **veintiún minutos antes** de que este
  informe se commiteara (`a9d0cfd`, 10:10) diciendo «una semana después sigue
  sin corregir». La afirmación hermana —que la suite no es hermética fuera de
  la máquina de desarrollo— sigue viva en `arquitectura.md:495` y sigue siendo
  cierta.
- **Instancia nueva, del mismo día y del mismo trabajo que arregló la
  anterior.** `docs/instalacion.md` §2 y §7 dicen hoy que `install.sh`,
  `install.ps1` y el workflow «existen en la rama `g5b-release-doctor`, pero
  no están en `main`» y que «no hay ningún tag publicado». Las dos son falsas
  desde las 11:27 del 09-11: G5b está mergeado y el tag `v0.1.0` existe. El
  efecto que describen —que el §2 no se puede seguir— era correcto, pero por
  el motivo de §5.11, no por el que declaraban. La sección escrita para dejar
  de prometer una release que no existía volvió a equivocarse dos horas
  después. **[CORREGIDO 09-11 12:25]** `0c81e87` retiró los dos avisos de
  `instalacion.md` y los de `README.md` al publicarse la release; el mensaje
  de ese commit lo dice mejor que este informe: «un aviso caducado es tan
  mentira como la promesa que vino a corregir». La instancia se deja anotada
  porque es la más corta que tiene el repo —cierta un día, falsa una hora— y
  porque solo se cerró por estar escrita como paso del plan, no por un
  check.
- Versiones: el 09-04, `marketplace.json` y `plugin.json` publicaban `1.0.0`
  frente a `engine/Cargo.toml` `0.1.0`, sin ninguna release. El 09-11 hay
  **tres** números distintos: `marketplace.json:4` metadata `1.0.0`,
  `marketplace.json:8` y `plugin.json:4` `1.1.0`, `Cargo.toml:3` `0.1.0`.
  Por la tarde el plugin y el marketplace van por `1.1.1` (`ae1f470`): el
  patrón de tres números se mantiene, el valor concreto ya no.
- El propio backlog registra un caso previo: `exo-recall.sh` decía «ronda los
  4,5 KB» cuando el bloque medía 5.921 B.
- Y el merge de la revisión crítica al backlog (`f86167a`) rompió la
  cabecera del fichero (perdió el prefijo `> Última revisión:` y el
  blockquote), corregido en `f6a5b0e`. La documentación se rompe incluso al
  documentar que se rompe.

**Por qué importa.** Es el síntoma de §5.1. Un lector que se fíe de
arquitectura.md §7 hoy cree que no hay CI. Y los comentarios del código
(§5.5) tienen el mismo problema a otra escala.

**Qué haría.** (a) Corregir §7 y el item de hermeticidad; (b) alinear las
tres versiones o documentar por qué el plugin versiona aparte del engine;
(c) añadir al cierre de sesión (`verify` o el pre-commit) un grep de las
afirmaciones de estado más frágiles («Sin CI», recuento de tests, versiones)
contra el árbol real. Una afirmación de estado que no se puede grepear no
debería estar en un documento «derivado del código».

**Backlog.** Alta · «La documentación de referencia contradice el repo el
mismo día en que se escribió».

### 5.4 La ciencia del retrieval es más débil de lo que parecen los números

**Hallazgo.** La cifra central del proyecto —engine-hybrid **48/55** hit@5
frente a 39/55 de basic-memory— es un resultado **in-sample**: los
parámetros que la producen se eligieron maximizando hit@5 sobre las mismas
55 queries que la reportan.

**Evidencia.** `engine/src/main.rs:12-25`: `BONUS_SELLADO` y
`ESCALA_FTS_SELLADA` son «ganadores del sweep 15+1 corridas (grid bonus
{0,0.1,0.2,0.3,0.5} × β{0.6,0.8,1.0})», con «selección pre-registrada §5.2.4
(max hit@5=49/55 → 4 celdas empatadas en β=0.6 → menor bonus=0.0)». El umbral
0.40 se fijó por el mismo criterio. El tamaño de trozo (900 chars) sale del
mismo sweep. `arquitectura.md` §6 reporta 48/55 sobre ese mismo `eval.jsonl`,
que además es privado («no están en el repo»). n=55, sin intervalo de
confianza, sin conjunto held-out.

**Por qué importa.** La regla de «pre-registrar el gate» se cumplió en la
forma (el criterio de selección estaba escrito antes de correr) pero no en
el fondo: pre-registrar el criterio de selección sobre un conjunto no
inmuniza contra el overfitting a ese conjunto. Con 15 celdas y 55 queries,
que la mejor celda gane 9 puntos a la línea base es plausible por
generalización real, y también es plausible en parte por selección. Hoy no
hay forma de distinguirlo. Dos consecuencias prácticas: el número no debería
presentarse como «lo que hace exo» sin la etiqueta in-sample, y —como la
propia arquitectura reconoce— no hay mecanismo de recalibración para otra
KB, así que el 48/55 es una propiedad de la KB del autor tanto como del
engine.

Relacionado y menor: el default de `exo search` es `--type fts`
(`main.rs:216`; era la 205 el 09-04), no el modo medido (`hybrid` +
`--min-similarity 0.40`).
Arquitectura.md lo avisa como gotcha; sigue siendo un default que apunta al
modo que no se midió.

**Qué haría.** (a) Redactar y congelar un held-out de queries nuevas
**antes** de volver a tocar β, bonus, umbral o troceado; (b) reportar
in-sample y held-out por separado en el próximo verdict, con n en cada
cifra; (c) decidir si el default de `exo search` pasa a ser el modo medido o
si el README deja de presentar el 48/55 como cifra del producto.

**Backlog.** Alta · «El 48/55 del hybrid es un resultado in-sample».

### 5.5 Idioma mezclado y comentarios como changelog

**Hallazgo (idioma).** Identificadores y módulos en español (`buscador`,
`trozos`, `aristas`, `escritor`, `objetivos`, `inicia`), claves JSON y flags
largos en inglés desde D8 (`schema_version` 2), aliases ocultos en español,
commits, docs y comentarios en español. Cada capa eligió distinto. Un lector
del envelope no reconoce el nombre del campo en el código que lo emite
(`Busqueda.avisos` ↔ `"warnings"`, `Resumen.indexadas` ↔ `"indexed"`).

**Hallazgo (comentarios).** 1.370 de 5.224 líneas de `engine/src/` eran
comentario el 09-04 (26 %); 1.815 de 6.682 el 09-11 (27 %). Buena parte
narra historia («hallazgo del gate M6», «review opus m2-01», «§5.2.6 de la
spec de fusión», «Task 3 del brief») en vez de enunciar el contrato actual,
y las referencias apuntan a briefs y consultorías que un lector externo no
puede resolver.

**Por qué importa.** El idioma es un coste de coherencia, no de corrección:
sube la barrera para un contribuidor y baja la legibilidad cruzada
código↔JSON. Los comentarios-relato son un segundo README que envejece igual
que el primero (§5.3), con el agravante de que están al lado del código y
parecen frescos. La re-verificación del 09-09 acotó bien el hallazgo: el
porcentaje no es el problema, el relato puro se concentra en `main.rs` y
`buscador.rs`, y `lib.rs` sale exonerado porque sus comentarios enuncian el
invariante y solo añaden la procedencia. Ese es exactamente el patrón que
habría que generalizar.

**Qué haría.** Decidir por escrito qué idioma llevan los identificadores y
aplicarlo solo a módulos nuevos (no renombrar en masa). En comentarios, dejar
el invariante y su consecuencia («recencia = git, no mtime: un clone fresco
resetea mtimes») y mover el relato («hallazgo del gate M6, 2026-08-22») al
verdict o plan correspondiente con un enlace.

**Backlog.** Baja · «Idioma mezclado sin criterio único» y Baja · «Los
comentarios del engine son un segundo changelog».

### 5.6 Techos de escala por diseño, sin medición

**Hallazgo.** Cuatro decisiones del engine son O(N) por operación,
documentadas como deliberadas y sin medida más allá de la KB del autor
(138 notas):

| Decisión | Dónde | Coste |
|---|---|---|
| KNN exhaustivo con `k = COUNT(*)` | `buscador.rs:286` | un scan de todos los vectores por query |
| `HashMap` con TODOS los trozos `(id, permalink)` cargado por query | `buscador.rs:290` | memoria y tiempo lineales en trozos |
| Tres aperturas de DB por búsqueda hybrid (`busca` + `busca_vector` + `enriquece_rutas`) | `buscador.rs:461` | tres conexiones, tres `PRAGMA journal_mode`, por query |
| Un proceso `git log -1` por nota indexada | `indexer.rs:192` | N spawns de git en un `rebuild`; caros en Windows |
| Relectura del frontmatter de TODAS las notas en cada arranque para encontrar las `tier: core` | `recall.rs:235`, motivo en `nota.rs:14` | N lecturas + N parseos YAML por `SessionStart` |

La última merece párrafo propio: `tier` no se persiste en el índice «porque
forzaría un rebuild de las DB existentes». Evitar una migración de esquema a
cambio de N lecturas de disco en cada arranque es deuda disfrazada de
prudencia, y `exo rebuild` ya existe como comando de primera clase.

**Por qué importa.** Ninguna es un bug hoy. Lo que falta es saber a qué
tamaño de KB deja de valer cada una, y el proyecto no tiene ni la medida ni
un camino previsto. Una KB de 5.000 notas no es exótica para un exocórtex
de años.

**Qué haría.** Generar una KB sintética de 5.000 notas y medir `exo
rebuild`, `exo recall --content` y `exo search --type hybrid` en Linux y
Windows. Con los números, o se documenta el techo soportado en
arquitectura.md o se abre la campaña (columna `tier` + rebuild, `git log` en
batch, una conexión por comando, partición en vec0). La columna `tier` es la
más barata y la haría ya.

**Backlog.** Media · «`tier` no se persiste en el índice» y Media · «Techos
de escala declarados, sin camino ni medición».

### 5.7 La mitad de la suite no corre en CI

**Hallazgo.** Las 4.713 líneas de shell del plugin tienen sus propios
`test-*.sh` (**once** ficheros, no diez) y ninguno se ejecuta en el workflow.
El test de contrato contra el binario real (`test-contrato-engine.sh`) tiene
rutas de esta máquina en la cabecera y `exo.exe` del `target/release` del repo
como default.

**Evidencia.** `.github/workflows/ci.yml` tiene hoy cuatro jobs (`lint`,
`msrv`, `test`, `install-gate`) y ninguno toca `plugins/exo/scripts/`.
**[CORREGIDO 09-11]** el «solo invoca `test-hermetico.sh`» dejó de ser exacto:
`654c757` cableó `install-gate`, que corre `scripts/test-install.sh` y
`scripts/test-install.ps1` en ubuntu, macOS y Windows contra una release falsa
servida por `file://`. Pero esos dos scripts viven en la raíz del repo, no en
el plugin. Cabecera de `test-contrato-engine.sh`, sin cambios: «depende de
estado de ESTA máquina (C:/Users/paul/.exo/index.db,
C:/proyectos/homework/kb-demo)», y `EXO_BIN` sigue apuntando por defecto a
`engine/target/release/exo.exe`. `install-gate` es además la prueba de que el
camino existe: el fixture reproducible que este item pide ya se ha escrito una
vez, para otro gate.

**Por qué importa.** Los hooks son la parte del producto que más toca al
usuario en cada turno y la que menos se verifica automáticamente. Un cambio
de forma del envelope se detectaría en el CI del engine pero no en su
consumidor.

**Qué haría.** Fixture reproducible (KB mínima + índice) y un job de CI para
`plugins/exo/`, empezando por `test-recall-inject.sh` y
`test-contrato-engine.sh`.

**Backlog.** Ya existían antes de esta revisión: Media · «Los scripts
`test-*.sh` de `plugins/exo/scripts/` no entran en CI» y Media ·
«`test-contrato-engine.sh` depende del índice y la KB reales de esta
máquina». Se anotan aquí porque forman parte del veredicto, no porque sean
hallazgo nuevo.

### 5.8 El coste del hook completo en Windows no está medido

**Hallazgo.** `hooks.json` cablea **tres** scripts bash en cada
`PreToolUse:Bash` (`git-c-bash.sh`, `git-add-all-guard.sh`,
`verify-before-commit.sh`) y uno en cada `UserPromptSubmit`
(`recall-inject.sh`) que lanza `exo recall --refresh`, `exo config --json` y
**trece** invocaciones de `jq`/`sed`/`tr` (7 `jq`, 2 `sed`, 4 `tr`; el
recuento original decía «del orden de seis» y se quedaba corto a la mitad, en
contra de su propio argumento). Las cifras publicadas
(«~10 ms» en la cabecera de `exo-recall.sh`, «~25 ms sin cambios» en
`exo-index.sh`) miden el **binario**, no el hook.

**Por qué importa.** Bajo Git Bash en Windows cada spawn de proceso cuesta
decenas de milisegundos. El coste real por prompt y por comando Bash es
desconocido, y es un coste que el usuario paga en cada turno de cada
sesión. Si el triple `PreToolUse` pasa de ~200 ms, se nota.

**Qué haría.** Instrumentar `_reflex-log.sh` con la duración del hook (o
medir a mano con `time` sobre un `INPUT` real) en Windows y Linux, y publicar
la cifra en `plugins/exo/README.md`. Si el triple supera el umbral, fusionar
los tres scripts en uno con un único parseo del JSON de entrada.

**Backlog.** Media · «El coste del hook completo en Windows no está medido;
solo el del binario».

### 5.9 No compila en esta máquina — **caducado, y era falso al publicarse**

> **[CORREGIDO 09-11]** Esta sección afirmaba en presente un fallo que no
> reproducía. Se deja, con la corrección encima, porque es el mejor ejemplo
> que tiene el informe del fallo que él mismo denuncia en §5.3: una afirmación
> de estado re-verificada mal y publicada como fresca.

**Hallazgo original (2026-09-04).** `cargo check --all-targets --locked` en
`engine/` falló con «exo@0.1.0 requires rustc 1.95» sobre `rustc 1.94.1`. La
MSRV es correcta (la fija `libsqlite3-sys` vía `cfg_select`; `Cargo.toml:6-7`
lo documenta) y el CI la comprueba.

**Estado real a 2026-09-11.** `rustc --version` en esta máquina → **1.98.0**,
toolchain `stable`, activo y por defecto; `~/.rustup/toolchains` lo data en el
2026-08-24, o sea **antes** de la revisión original. `engine/Cargo.toml:8`
declara `rust-version = "1.95"` y `cargo metadata` resuelve limpio: **no hay
fallo de MSRV aquí**. La lectura de 1.94.1 del 09-04 venía de un `rustup
default` dejado en un bisect de MSRV (las toolchains 1.88 a 1.95 se instalaron
el 09-02), no del estado normal de la máquina. La re-verificación del 09-09 ya
lo había dado por «caducado a medias»; la versión original de esta sección
**revirtió** ese hallazgo apelando a «esta máquina», que es precisamente esta.

**Lo que sí sigue vivo, y es la mitad pequeña.** No existe
`rust-toolchain.toml`, ni en `engine/` ni en la raíz (`find . -iname
"rust-toolchain*"` → vacío). El repo sigue sin decirle al toolchain local qué
versión usar, así que en cualquier máquina con un `rustup default` viejo el
fallo aparece **después** de resolver dependencias, que es lo que lo hace caro
de diagnosticar. `docs/instalacion.md:25-28` sí documenta el mecanismo exacto
del corte.

**Qué haría.** `engine/rust-toolchain.toml` con `channel = "stable"`. Cinco
minutos, y elimina la clase entera de fallo.

**Backlog.** Media · «El repo no le dice al toolchain local qué versión
usar: falta `rust-toolchain.toml`». El item ya llevaba la marca «CADUCADO A
MEDIAS el 2026-09-09» y esa marca era la correcta; lo que estaba mal era esta
sección.

### 5.10 Observaciones menores que no llegaron al backlog

Anotadas para que no se pierdan; ninguna justifica un item propio hoy.

- **Dos parsers de frontmatter con semánticas distintas coexisten a
  propósito**: `nota.rs` (YAML real, descarta la nota sin `permalink`) y
  `frontmatter.rs` (escaneo por líneas al estilo kbx, nunca falla, degrada a
  ausente). La cabecera de `frontmatter.rs` lo justifica bien. Sigue siendo
  una nota que puede ser válida para uno e inválida para el otro, y el día
  que diverjan en un caso real nadie lo verá venir.
- **Dos idiomas git** (`indexer::git_epoch_de` fail-silent frente a
  `gitx::ultimo_commit` fail-loud), también deliberados y también
  documentados. Mismo comentario.
- **El dup-gate del write-path es Jaccard sobre tokens del slug con umbral
  0,6.** Es léxico a propósito (arquitectura §3.7 explica por qué el
  semántico daba falsos rojos), pero un umbral fijo sobre slugs cortos es
  frágil en los dos sentidos.
- **`abre_db` hace `bail!` si `PRAGMA journal_mode=WAL` no devuelve `wal`.**
  Correcto en local; en un sistema de ficheros de red o un directorio
  sincronizado por la nube, WAL puede no estar disponible y el engine entero
  deja de abrir la DB, también para leer.
- **El bloque de arranque va al 96 % de su cap y trunca en silencio.** Ya
  era el primer item de Alta del backlog antes de esta revisión; se cita
  porque es el ejemplo más limpio de «presupuesto sellado sin aire», que la
  propia doctrina de la KB prohíbe.
- **Cadencia de trabajo.** 77 commits en un día y rachas de 40 sugieren
  sesiones largas de agente con commit por micro-paso. No es un problema en
  sí; sí explica el volumen documental y el estilo de los mensajes.

### 5.11 La release se cortó en rojo dos veces antes de existir — **cerrado el mismo día**

> Hallazgo **nuevo** del 2026-09-11 por la tarde, y **cerrado** tres horas
> después de anotarse. Se deja porque lo que documenta no es el bug, que ya
> está arreglado, sino los dos modos de fallo que aparecieron en el estreno de
> la cadena de adopción y el **patrón de verificación** que los cazó. Todas
> las horas de esta sección son locales (UTC+2); los timestamps de `gh` vienen
> en UTC y se convierten.

**Qué pasó.** El merge de G5b (11:27) empujó el tag `v0.1.0` y disparó el
workflow. Falló dos veces seguidas, en dos sitios distintos, antes de
publicar:

1. **`shasum: command not found`, exit 127.** Run `34584773325`
   (11:33 local / 09:33 UTC, 10m49s), paso «Empaquetar y calcular el SHA256»,
   runner `x86_64-pc-windows-msvc`. El workflow usaba `shasum -a 256` y no
   `sha256sum` con un comentario que explicaba por qué —«este último NO existe
   en el runner de macOS»—: la elección resolvió macOS y rompió Windows, donde
   el bash de `windows-latest` no trae `shasum`. El fallo llegó **después** de
   pasar la suite, y el job `publish` quedó `skipped`: **ninguna release a
   medias**. Fix en PR #7 (`360175c`).
2. **El instalador de Windows rechazaba el binario de Windows de su propia
   release.** `sha256sum` firma con `*` delante del nombre y `shasum` con dos
   espacios; `install.ps1` solo aceptaba un formato. Fix en PR #8
   (`8a86832`).

**Estado final.** Run `34586964204` (`workflow_dispatch` sobre `main`, 11m24s)
en verde, y la release **publicada**: `gh release view v0.1.0` lista seis
artefactos —`exo-x86_64-unknown-linux-gnu`, `exo-x86_64-pc-windows-msvc.exe`,
`exo-aarch64-apple-darwin` y sus tres `.sha256`—. `0c81e87` retiró los avisos
de `README.md` e `instalacion.md` y escribió el runbook
(`docs/superpowers/runbooks/2026-09-11-g5b-release-v0.1.0.md`), que registra
también un tercer tropiezo: el clippy de una máquina Windows no ve el código
`#[cfg(unix)]`, así que salía limpio en local con un `needless_return` que el
CI de ubuntu sí caza. Verificación en W11 **con el binario de la release**:
diez filas de `doctor`, cero `fail`, búsqueda híbrida con resultados y el gate
de la KB corriendo sin bloquear. Queda pendiente la máquina Linux, que no
bloquea.

**Por qué importa igual, ya cerrado.** Los tres fallos son de la misma
familia y ninguno lo habría cazado un CI verde: dos plataformas que discrepan
en una herramienta (`shasum`/`sha256sum`) y una que oculta código al linter
(`cfg(unix)`). Lo que los cazó fue **ejercer el artefacto publicado en vez del
workflow** —instalar desde la release, correr el binario descargado— que es
exactamente lo que este informe recomendaba en su primera versión y lo que la
§5.7 sigue pidiendo para los hooks. Y la lección de documentación es la de
§5.3 en su forma más corta: los avisos de `instalacion.md` fueron ciertos
durante un día y falsos durante una hora, y solo dejaron de mentir porque
retirarlos era un paso escrito del plan.

**Backlog.** Entrada nueva en `## Cerrado con evidencia` —«Release `v0.1.0`
— el binario que no existía»— con las dos corridas, los dos fixes y los seis
artefactos. Lo que **no** cierra: los seis items marcados «lo cierra / lo
subsume G5» siguen abiertos, porque G5b se cerró sin adoptar ninguno.

## 6. Qué haría yo, por orden

> **Reordenado el 09-11 por la tarde.** El punto 1 original —la release de G5
> por delante de cualquier feature— se ejecutó durante la semana; lo que queda
> de él es cerrarlo en verde. El 3 se ha partido: la mitad ya está hecha y la
> otra mitad se ha convertido en el punto 4.

1. ~~**Cerrar la release en verde**~~ — **hecho el 09-11 a las 12:10**
   (§5.11): dos fixes (PR #7 y #8), corrida verde por `workflow_dispatch`,
   seis artefactos publicados y avisos retirados en `0c81e87`. Lo que hereda
   el puesto: **verificar el artefacto, no el workflow**, y extender ese
   patrón a lo que aún no lo tiene (§5.7).
2. **Decidir qué es el proyecto** y escribirlo en el README. Si aspira a
   usuarios, detrás de la release va la purga de `Paul` y `kb-demo` de los
   tres scripts de producción (§5.2). Lo de `kbx` ya está medio resuelto por
   el cutover de G4c.
3. **Reservar un held-out** de queries nuevas antes de tocar ningún parámetro
   más, y dejar de reportar in-sample sin etiquetarlo (§5.4). Intacto desde el
   09-04, y sigue siendo el hallazgo de más peso del informe.
4. **Marcar por `tier`** (§5.1): `log` en todo lo fechado —este informe
   incluido—, `core` en los cuatro vivos, y el grep de afirmaciones frágiles
   corriendo solo sobre esos cuatro. Es la corrección estructural de §5.3 y
   cuesta un frontmatter y un job de CI.
5. **Columna `tier` en el índice** y aceptar el rebuild (§5.6). Un día de
   trabajo, elimina N lecturas de disco por arranque.
6. **`rust-toolchain.toml`** (cinco minutos, §5.9) y el job de CI para
   `plugins/exo/` con fixture propio, que ahora tiene precedente: el
   `install-gate` de `654c757` demuestra que el patrón funciona en las tres
   plataformas.
7. **Medir** antes de rediseñar: latencia del hook completo en Windows —trece
   spawns por prompt, §5.8— y KB sintética de 5.000 notas (§5.6). Con
   números, esas dos decisiones se toman solas.

## 7. Mapa crítica → backlog

| § | Crítica | Item en `docs/backlog.md` (greppable) | Sección |
|---|---|---|---|
| 5.1 | Exhaust de proceso frente a documentación viva | «Decisión abierta: proceso frente a producto» — **a reescribir**: su evidencia es el ratio, que no mide lo que dice | Baja |
| 5.1 | Marcado por `tier` de lo que caduca | **sin item** — acción nueva del 09-11 | — |
| 5.2 | Plugin personal | «"exo genérico" sigue siendo el plugin de Paul para Paul» | Alta |
| 5.3 | Deriva documental | «La documentación de referencia contradice el repo el mismo día» | Alta |
| 5.4 | Retrieval in-sample | «El 48/55 del hybrid es un resultado in-sample» | Alta |
| 5.5 | Idioma / comentarios | «Idioma mezclado sin criterio único» · «Los comentarios del engine son un segundo changelog» | Baja |
| 5.6 | Techos de escala | «`tier` no se persiste en el índice» · «Techos de escala declarados, sin camino ni medición» | Media |
| 5.7 | Shell fuera de CI | preexistentes: «Los scripts `test-*.sh` … no entran en CI» · «`test-contrato-engine.sh` depende del índice y la KB reales» | Media |
| 5.8 | Coste de hooks | «El coste del hook completo en Windows no está medido» | Media |
| 5.9 | MSRV / toolchain | «El repo no le dice al toolchain local qué versión usar» | Media |
| 5.10 | Menores | sin item; «El bloque de arranque va al 96% de su cap» era preexistente | — |
| 5.11 | Release cortada en rojo | **sin item** — debería estar en Alta hasta que la release exista | — |

## 8. Qué ha cambiado entre el 2026-09-04 y el 2026-09-11

> **[REESCRITO 09-11 por la tarde]** La primera versión de esta sección se
> midió contra `bf4ba7a` creyendo que era `main`. No lo era: `main` estaba en
> `a0d538b` desde el 09-10 11:25 (merge de G4c) y la rama
> `g5b-release-doctor` se mergeó a las 11:27 del 09-11, hora y cuarto después
> de que este informe se commiteara. Tres de sus cinco viñetas decían que algo
> «se refuerza» cuando la evidencia ya había ido en sentido contrario. Un
> documento que se declaró re-verificado se rompió exactamente por eso
> (§5.1, §5.9).

- **Los diez items** entraron en `main` vía PR #3 (`f86167a`) y se
  re-verificaron el 09-09 (`f6a5b0e`). **Ninguno cerrado**: siguen los diez
  como `- [ ]` en `docs/backlog.md` a día de hoy.
- **G4c (en `main` el 09-10)** portó `gate`, `lint` y `presupuesto` a Rust e
  hizo el cutover de `kb-precommit.sh` de `kbx` a `exo`. `kbx` bajó de 7
  ficheros del plugin a 5 y dejó de ser dependencia del pre-commit: §5.2 se
  **debilita**. La viñeta original decía lo contrario.
- **G5b (en `main` el 09-11 a las 11:27)** añadió `exo doctor` con diez
  checks, `install.sh`, `install.ps1`, `release.yml` y el job `install-gate`;
  retiró el «Sin CI» de `arquitectura.md` (`3673059`, 09:49) y reescribió
  `instalacion.md`. El código pasó de 6.682 a 9.043 líneas y los tests de
  7.381 a 10.210 en dos días.
- **La release `v0.1.0` está publicada** desde las 12:10 del 09-11, tras dos
  fallos y dos PRs (§5.11): seis artefactos, avisos retirados y runbook
  escrito. Es el único hallazgo nuevo de esta corrección y nació cerrado.
  Cae con él la conclusión de adopción de §5.2.
- **`arquitectura.md` ya no dice «Sin CI».** Lo que sigue diciendo, y es
  cierto, es que la suite no es hermética fuera de la máquina de desarrollo.
  A cambio, `instalacion.md` estrenó dos afirmaciones falsas el mismo día
  (§5.3).
- **Las versiones** pasaron a `1.1.1` en plugin y marketplace (`ae1f470`);
  siguen siendo tres números distintos con el `0.1.0` del crate.
- **`rust-toolchain.toml` sigue sin existir**, pero el fallo de MSRV **no**
  reproduce en esta máquina (§5.9): la viñeta original afirmaba lo contrario y
  era falsa.
- **El ratio docs/código** cayó de 5,4 a 4,1 sin que nadie tocara la
  documentación, y ese movimiento es lo que obligó a re-fundar §5.1 sobre el
  desglose exhaust / documentación viva en vez de sobre el ratio.
- El backlog recibió además una pasada propia de consumo de tokens (09-09)
  que no procede de esta revisión y no se evalúa aquí.
