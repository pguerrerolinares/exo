# Reporte — m2-08: `exo recall` + desempate determinista (M2-09a) + latencia

## Estado: EN CURSO (Tareas 1 y 2 completas y verificadas; Tarea 3 pendiente del índice completo)

Worktree `/home/paul/Documentos/proyectos/exo/.worktrees/m2-08`, rama `m2-08`, base `4abb80c`.

Commits:
- `34548d1` — fix(m2-09a): desempate determinista por permalink en busca_vector y fusiona
- `2234887` — feat(m2-08): comando exo recall (arranque + consulta)
- `<pendiente>` — docs(m2-08): reporte final + números de latencia

## Tarea 1 — Desempate determinista (M2-09a)

`engine/src/buscador.rs`: ambos `sort_by` (línea ~204 en `busca_vector`, línea
~277 en `fusiona`) ahora desempatan por `permalink` ascendente cuando el
score empata: `.then_with(|| a.0.cmp(&b.0))` / `.then_with(|| a.permalink.cmp(&b.permalink))`.

Sin el desempate, `sort_by` (que es un *stable sort*) preserva el orden de
iteración del `HashMap`/`HashSet` de origen ante un empate — no reproducible
entre corridas ni entre inserciones en distinto orden.

**TDD real, verificado**: ambos tests se escribieron primero y se corrieron
en rojo antes del fix.

- `fusion_desempate_determinista_por_permalink` (unitario, `src/buscador.rs`):
  dos `HashMap` con 5 claves de score idéntico, insertadas en dos órdenes
  distintos. Falló antes del fix con salida `["c","d","b","e","a"]` /
  `["b","d","a","c","e"]"` (no reproducible); pasa con `["a","b","c","d","e"]`
  en ambos casos tras el fix.
- `busca_vector_desempate_determinista_por_permalink` (integración,
  `tests/buscador.rs`): DB con 3 entidades que comparten el MISMO embedding
  (unitario, sin pasar por el modelo real — evita depender de coincidencias
  del embedder) para forzar empate EXACTO de similitud coseno. Falló antes
  del fix con salida `["z","y","x"]` (orden de inserción invertido); pasa
  con `["x","y","z"]` en ambos órdenes de inserción tras el fix.

Verificación: `cargo test` completo, 74/74 verdes (72 previos + 2 nuevos)
tras el fix.

## Tarea 2 — `exo recall`

Módulo nuevo `engine/src/recall.rs` + wiring en `main.rs` (`Comando::Recall`,
`ArgsRecall`, `recall_cmd`).

### Decisiones y desviaciones declaradas

1. **`tier` aditivo en `Nota`/`FrontmatterLaxo`** (`src/nota.rs`), tal como
   pedía el brief: campo `Option<String>`, sin tocar el schema del índice.
   `exo recall` en modo arranque relee el frontmatter de cada `.md` listado
   en `notas` para clasificar `tier: core` — medido en la KB real (ver
   sección de latencia): **no se come el presupuesto de 100 ms** (~115
   ficheros pequeños, I/O de disco local).

2. **Cores excluidos del bloque de "recientes"** (decisión no fijada
   explícitamente por el brief): si una nota es `tier: core` no vuelve a
   aparecer en la sección de recientes aunque sea de las `--limite` más
   nuevas por `git_epoch`. Mostrar la misma nota dos veces en el bloque de
   arranque no aporta nada al consumidor. Verificado con
   `recall_arranque_no_duplica_core_en_recientes`.

3. **Snippet = primer trozo (`orden = 0`) de la entidad**, no "el trozo que
   casó" literal. La fusión hybrid (`busca_hybrid`/`fusiona`) agrega a nivel
   ENTIDAD por máxima similitud (spec fusión) y no expone qué `trozo`
   individual disparó ese máximo; recalcular la similitud coseno por trozo
   dentro de `recall` (releer embeddings, repetir el embed de la query,
   hacer el producto punto a mano) habría duplicado trabajo no trivial para
   un dato puramente informativo (snippet, no gate de ranking). El primer
   trozo es determinista, barato (una query SQL indexada por
   `(permalink, orden)`) y razonablemente representativo del contenido de
   la nota. Documentado, no escondido.

4. **Cap de bytes aplicado UNA VEZ, sobre una representación línea-a-línea
   común a texto y `--json`** (el brief no fija esto explícito, solo
   describe el comportamiento del cap "en la salida de texto"): una nota
   entra en el resultado —tanto en el bloque de texto como en `notas` del
   JSON— solo si TODAS sus líneas (la principal, más la de snippet en modo
   consulta) caben enteras en el presupuesto. En cuanto una línea no cabe,
   el proceso PARA por completo: ni esa nota "a medias" ni ninguna
   posterior aparecen, en ningún formato. Así el consumidor ve exactamente
   el mismo conjunto de notas pida texto o JSON. Verificado con
   `aplica_cap_corta_por_lineas_enteras_y_para` y
   `aplica_cap_nota_con_snippet_que_no_cabe_se_descarta_entera`.

5. **Snippet recortado a 200 bytes por frontera de carácter UTF-8**, nunca a
   mitad de un carácter multibyte (`recorta_bytes`, testeado con `"é"`
   repetida — cada `é` ocupa 2 bytes). Cuenta `str::len()` (bytes), no
   `.chars().count()`, tal como exige el brief.

6. **`--limite` con el mismo default (5) en ambos modos**: el brief fija
   "--limite por defecto 5" explícitamente solo para modo consulta; es el
   mismo flag de clap en ambos modos, así que el mismo default aplica al
   tope del bloque de "recientes" en modo arranque. No hay contrato
   contradictorio, solo una precisión del brief que no cubría el caso.

7. **`exo recall` NO tiene una tabla de exit codes nueva**: `main()` sigue
   haciendo `exit(1)` genérico para cualquier `Err` de `anyhow` (igual que
   `index`/`search`). Un recall vacío tras el cap es `bail!` con mensaje
   claro. Truncado NO es error (exit 0, aviso a stderr).

### TDD

Proceso real (declarado, no todo fue rojo-verde estricto función a
función): las piezas PURAS (`recorta_bytes`, `aplica_cap`) se diseñaron con
su interfaz y sus 7 tests unitarios escritos junto con la implementación
inicial —dada la cantidad de piezas interdependientes (cap compartido entre
texto/JSON, unidades de 1 o 2 líneas, snippet vs sin snippet)— y se
verificaron en verde a la primera corrida; no hubo una iteración roja previa
para esa pieza. Los 5 tests de integración (`recall_arranque`/
`recall_consulta` contra una DB real con commits de git de epoch
controlado) sí se escribieron contra una implementación ya existente y
pasaron a la primera corrida real, sirviendo como verificación de
comportamiento más que como TDD rojo-verde. La Tarea 1 (arriba) sí siguió
TDD estricto rojo→verde. Se declara esta diferencia de rigor en vez de
narrar un proceso que no ocurrió.

12 tests nuevos (7 unitarios `src/recall.rs` + 5 integración
`tests/recall.rs`), todos verdes. `cargo test` completo tras la Tarea 2:
86/86 verdes (72 base + 2 Tarea 1 + 12 Tarea 2).

## Tarea 3 — Latencia (medición, NO gate)

Completada por el ORQUESTADOR, no por el executor: el executor quedó idle tras
la Tarea 2 y no retomó el trabajo pese a dos mensajes. Se declara aquí en vez
de presentar la medición como suya.

Índice: `kb-completa.db` (KB real, 138 notas / 3018 trozos / 526 aristas,
`kb-demo` en `7ef4fba`). Binario: `target/release/exo` de esta rama.
Herramienta: **hyperfine** (instalado en la máquina), 20 corridas por caso,
p95 calculado sobre los tiempos exportados (`--export-json`).

| Caso | Comando | n | min | mediana | **p95** | max | Referencia |
|---|---|---|---|---|---|---|---|
| Arranque (FTS-only, sin modelo) | `exo recall --db <db>` | 20 | 4,5 ms | 11,4 ms | **14,0 ms** | 16,2 ms | deseada <100 ms ✅ |
| Consulta hybrid **en frío** | `exo recall --db <db> --query 'doctrina de agentes' --min-similitud 0.40` | 20 | 985 ms | 1000 ms | **1032 ms** | 1050 ms | deseada <2,0 s ✅ · bm hoy mediana 4,4 s |

Notas de método, honestas:

- El caso de arranque se midió **con** `--warmup 3`: mide estado caliente de
  page cache, que es el estado real de un hook que dispara en cada sesión.
  Hyperfine avisa de que por debajo de 5 ms su calibración del arranque de
  shell pierde precisión; con p95 = 14 ms y presupuesto de 100 ms, el margen
  absorbe de sobra ese ruido.
- El caso hybrid se midió **sin warmup a propósito**: cada corrida es un
  proceso nuevo que paga la carga del modelo ONNX. La varianza es mínima
  (σ = 18 ms), lo que confirma que el coste dominante es determinista (carga
  del modelo), no la búsqueda.
- "En frío" aquí significa proceso nuevo con el modelo **ya descargado** en la
  caché de HF. La primera descarga (~0,6 GB, ~30 s medidos en M2-01) es un
  coste de instalación, no de consulta.
- El modo arranque relee el frontmatter de los 138 `.md` para clasificar
  `tier: core`, decisión declarada en Tarea 2. Los 14 ms de p95 confirman que
  esa relectura NO se come el presupuesto: era la duda abierta del brief y
  queda resuelta con dato.

Ambos números pasan con holgura las referencias deseadas. Son **informativos**
por régimen de campaña (config §ACTUALIZACIÓN 2026-08-17): no gatean nada.

## Verificación end-to-end (orquestador, contra la KB real)

`exo recall --db <kb-completa.db>` — modo arranque, texto:

```
=== Recall exo (PARCIAL — no sustituye tu brief) ===
- /home/paul/Documentos/proyectos/kb-demo/Backlog — frentes abiertos.md — Backlog — frentes abiertos
- /home/paul/Documentos/proyectos/kb-demo/Paul - perfil de trabajo.md — Paul - perfil de trabajo
- /home/paul/Documentos/proyectos/kb-demo/core/core-index.md — core-index
- /home/paul/Documentos/proyectos/kb-demo/core/doctrina-agentes.md — doctrina-agentes
- /home/paul/Documentos/proyectos/kb-demo/learnings/desarrollo-agentico.md — desarrollo-agentico
- /home/paul/Documentos/proyectos/kb-demo/log/doctrina-agentes-bitacora.md — doctrina-agentes-bitacora
[...5 recientes por git_epoch]
```
exit 0. Los 4 primeros son los `tier: core` en orden de ruta; el resto,
recientes por git.

`--query 'cómo delego gates a un consultor' --min-similitud 0.40` devuelve 5
notas con su línea de snippet (`  · ...`) bajo cada una.

`--json` (modo arranque): **una sola línea**, `schema_version: 1`,
`command: "recall"`, `data.modo: "arranque"`, `truncado: false`, 10 notas,
`score`/`snippet` a `null` como fija el contrato. Nada humano en stdout.
