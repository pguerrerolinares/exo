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
> Re-verificado el **2026-09-11** contra `main` (`bf4ba7a`): las cifras que
> cambiaron llevan las dos fechas; §8 resume qué se movió en la semana. No es
> una spec ni un plan. Su equivalente más cercano en el repo es
> `2026-08-02-foto-as-is-framework.md`: una foto firmable para comparar
> contra ella más adelante.

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
decisiones bien pensadas sobre cómo fallar. Lo que lo rodea —del orden de
seis líneas de documentación por cada línea de código, consultorías,
gates, runbooks por cutover— es la mayor parte del repo, y es también donde
se concentra el riesgo: la documentación ya se desincroniza del código en
cuestión de días, el plugin sigue siendo personal a pesar de una ola entera
de «exo genérico», y la cifra central del retrieval (48/55) se eligió y se
reporta sobre las mismas 55 queries.

Nada de esto es un defecto de calidad del código. Son tres decisiones sin
tomar: para quién es exo, cuánto proceso merece, y qué evidencia cuenta como
evidencia.

## 3. Métricas

| Métrica | 2026-09-04 | 2026-09-11 (`bf4ba7a`) |
|---|---|---|
| Líneas de markdown (`docs/` + `evals/` + `reports/`) | 30.547 | 35.872 |
| Líneas de Rust en `engine/src/` | 5.224 | 6.682 |
| Ratio docs / código | 5,8 : 1 | 5,4 : 1 |
| Líneas de tests Rust (`engine/tests/`) | 5.629 | 7.381 |
| Líneas de shell en `plugins/exo/scripts/` | 4.545 | 4.666 |
| Líneas de comentario en `engine/src/` | 1.370 (26 %) | 1.815 (27 %) |
| Módulos en `engine/src/` | 19 | 22 (+`gate.rs`, `lint.rs`, `presupuesto.rs`) |
| Binarios de test en `engine/tests/` | 28 | 33 |
| Commits / días con actividad | 320 / 15 | 351 / 17 |
| Autores (identidades git) | 1 persona, 3 emails | idem |
| Pico de commits en un día | 77 (2026-07-17) | idem |
| `cargo check` en esta máquina (rustc 1.94.1) | falla: «requires rustc 1.95» | sigue fallando |

Lectura de la tendencia entre las dos fechas: el código creció un 28 % y la
documentación un 17 %. La proporción mejora, pero el volumen absoluto de
documentación sigue creciendo.

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

### 5.1 El proceso se ha comido al producto

**Hallazgo.** Hay 5,4 líneas de documentación por cada línea de código, y la
mayoría no es documentación de uso: son consultorías (agentes de IA jugando
el rol de consultor, con brief, informe y síntesis), verdicts de gate,
runbooks de cutover, planes por ola. 320 commits en 15 días de actividad,
con picos de 77 en un día, de un solo autor. Los mensajes de commit son
ensayos.

**Evidencia.** `git ls-files | xargs wc -l` por carpeta (§3);
`docs/superpowers/consultas/` tiene ocho carpetas fechadas con hasta nueve
informes cada una; `git log --format=%ad | sort | uniq -c`.

**Por qué importa.** No porque documentar sea malo, sino por el coste de
mantenerlo sincronizado: §5.3 muestra que la documentación de referencia
ya contradice al repo el mismo día en que se escribe. Cuando el volumen
supera lo que una persona mantiene al día, la documentación deja de ser
fuente de verdad y pasa a ser una segunda cosa que puede estar rota. Para
un exocórtex personal, el aparato de consultorías y gates está
sobredimensionado. Para un ejercicio de aprendizaje o un portfolio de
método, es defendible —pero entonces el README debería decirlo, porque hoy
se presenta como herramienta.

**Qué haría.** Escribir en el README, en dos frases, para quién es exo hoy.
De esa respuesta se deriva qué carpetas de `docs/superpowers/` pasan a
archivo histórico y cuánto proceso merece la siguiente ola.

**Backlog.** Baja · «Decisión abierta: proceso frente a producto». Cruza con
el item preexistente «Nombres y ubicaciones» (`docs/superpowers/` como
carpeta de docs de un proyecto que quiere jubilar superpowers).

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

**Backlog.** Alta · «"exo genérico" sigue siendo el plugin de Paul para
Paul». Distinto del item preexistente de Baja «`kb-demo` como fixture en 8
ficheros de test»: aquí son hooks y skills de producción.

### 5.3 Deriva documental, a pesar de la disciplina

**Hallazgo.** La documentación «derivada del código» contradice al repo, y
lo hace en plazos de horas, no de meses.

**Evidencia.**

- `docs/arquitectura.md` §7 («Qué NO está implementado») afirma «**Sin
  CI**: no hay `.github/`» (línea 489 el 09-04, línea 495 el 09-11) y que la
  suite no es hermética fuera de la máquina de desarrollo. El CI existe desde
  `e378cbc` (2026-09-02), el mismo día en que se fechó el documento, y el
  README describe esa corrida en tres SO dos párrafos más arriba del enlace a
  arquitectura.md. **Una semana después sigue sin corregir.**
- Versiones: el 09-04, `marketplace.json` y `plugin.json` publicaban `1.0.0`
  frente a `engine/Cargo.toml` `0.1.0`, sin ninguna release. El 09-11 hay
  **tres** números distintos: `marketplace.json:4` metadata `1.0.0`,
  `marketplace.json:8` y `plugin.json:4` `1.1.0`, `Cargo.toml:3` `0.1.0`.
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
(`main.rs:205`), no el modo medido (`hybrid` + `--min-similarity 0.40`).
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

**Hallazgo.** Los ~4.600 líneas de shell del plugin tienen sus propios
`test-*.sh` (diez ficheros) y ninguno se ejecuta en el workflow. El test de
contrato contra el binario real (`test-contrato-engine.sh`) tiene rutas de
esta máquina en la cabecera y `exo.exe` del `target/release` del repo como
default.

**Evidencia.** `.github/workflows/ci.yml` solo invoca
`engine/scripts/test-hermetico.sh` (= `cargo test`). Cabecera de
`test-contrato-engine.sh`: «depende de estado de ESTA máquina
(C:/Users/paul/.exo/index.db, C:/proyectos/homework/kb-demo)».

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
del orden de seis invocaciones de `jq`/`sed`/`tr`. Las cifras publicadas
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

### 5.9 No compila en esta máquina

**Hallazgo.** `cargo check --all-targets --locked` en `engine/` falla con
«exo@0.1.0 requires rustc 1.95» sobre `rustc 1.94.1`. La MSRV es correcta
(la fija `libsqlite3-sys` vía `cfg_select`; `Cargo.toml:6-7` lo documenta) y
el CI la comprueba. Lo que falta es que el repo se lo diga al toolchain
local: no hay `engine/rust-toolchain.toml`, así que el fallo aparece
**después** de resolver dependencias, que es lo que lo hace caro de
diagnosticar.

**Estado a 2026-09-11.** La re-verificación del 09-09 lo dio por «caducado a
medias» porque la máquina donde se re-verificó ya corría rustc 1.98.0. En
**esta** máquina (la de trabajo) sigue en 1.94.1 y el fallo reproduce igual.
`rust-toolchain.toml` sigue sin existir. Es un ejemplo pequeño de §5.2: el
repo funciona en la máquina del autor y tropieza en la siguiente.

**Qué haría.** `engine/rust-toolchain.toml` con `channel = "stable"` o la
MSRV, y anotar el error exacto en `instalacion.md` §1 para que sea
googleable.

**Backlog.** Media · «El repo no le dice al toolchain local qué versión
usar: falta `rust-toolchain.toml`».

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

## 6. Qué haría yo, por orden

1. **Decidir qué es el proyecto** y escribirlo en el README. Todo lo demás
   depende de esto. Si es personal: congelar el proceso, archivar la mitad
   de `docs/superpowers/`, seguir. Si aspira a usuarios: la release con
   binario de G5 pasa por delante de cualquier feature, y con ella la purga
   de `Paul`, `kb-demo` y `kbx` del plugin.
2. **Reservar un held-out** de queries nuevas antes de tocar ningún
   parámetro más, y dejar de reportar in-sample sin etiquetarlo.
3. **Corregir arquitectura.md §7 y las versiones** hoy, y añadir el grep de
   afirmaciones frágiles al cierre. Es barato y detiene la sangría de §5.3.
4. **Columna `tier` en el índice** y aceptar el rebuild. Un día de trabajo,
   elimina N lecturas por arranque.
5. **`rust-toolchain.toml`** y el job de CI para `plugins/exo/` con fixture
   propio. Lo primero son cinco minutos; lo segundo cierra el agujero de
   verificación más grande que queda.
6. **Medir** antes de rediseñar: latencia de hooks en Windows y KB sintética
   de 5.000 notas. Con números, las decisiones de §5.6 y §5.8 se toman
   solas.

## 7. Mapa crítica → backlog

| § | Crítica | Item en `docs/backlog.md` (greppable) | Sección |
|---|---|---|---|
| 5.1 | Proceso frente a producto | «Decisión abierta: proceso frente a producto» | Baja |
| 5.2 | Plugin personal | «"exo genérico" sigue siendo el plugin de Paul para Paul» | Alta |
| 5.3 | Deriva documental | «La documentación de referencia contradice el repo el mismo día» | Alta |
| 5.4 | Retrieval in-sample | «El 48/55 del hybrid es un resultado in-sample» | Alta |
| 5.5 | Idioma / comentarios | «Idioma mezclado sin criterio único» · «Los comentarios del engine son un segundo changelog» | Baja |
| 5.6 | Techos de escala | «`tier` no se persiste en el índice» · «Techos de escala declarados, sin camino ni medición» | Media |
| 5.7 | Shell fuera de CI | preexistentes: «Los scripts `test-*.sh` … no entran en CI» · «`test-contrato-engine.sh` depende del índice y la KB reales» | Media |
| 5.8 | Coste de hooks | «El coste del hook completo en Windows no está medido» | Media |
| 5.9 | MSRV / toolchain | «El repo no le dice al toolchain local qué versión usar» | Media |
| 5.10 | Menores | sin item; «El bloque de arranque va al 96% de su cap» era preexistente | — |

## 8. Qué ha cambiado entre el 2026-09-04 y el 2026-09-11

- Los diez items entraron en `main` vía PR #3 (`f86167a`) y fueron
  re-verificados el 09-09 (`f6a5b0e`): nueve vivos sin tocar, uno
  retitulado (MSRV), tres con cifras actualizadas. Ninguno cerrado.
- El código creció un 28 % (tres módulos nuevos del port de kbx: `gate.rs`,
  `lint.rs`, `presupuesto.rs`; cinco binarios de test más) y la documentación
  un 17 %. La tendencia de §5.1 se ha suavizado, no invertido.
- `kbx` pasó de estar citado en dos ficheros del plugin a estar en siete;
  §5.2 se refuerza.
- `arquitectura.md` sigue diciendo «Sin CI» (ahora en la línea 495). Las
  versiones pasaron de dos números distintos a tres (plugin 1.1.0 por el hook
  nuevo de SessionStart, metadata del marketplace 1.0.0, crate 0.1.0). §5.3
  se refuerza.
- `rust-toolchain.toml` sigue sin existir; en la máquina de trabajo el
  `cargo check` sigue fallando. §5.9 sigue vivo entero, no a medias.
- El backlog recibió además una pasada propia de consumo de tokens (09-09)
  que no procede de esta revisión y no se evalúa aquí.
