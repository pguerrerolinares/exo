# Gate de merge — rama `m2-08` → `main`

**Consultor**: fable delegado, fresco (sin participación previa en esta pieza).
**Fecha**: 2026-08-17 22:57 CEST.
**Alcance**: 4 commits sobre `main` (`4abb80c`): `34548d1` (desempate M2-09a),
`2234887` (`exo recall`, M2-08), `d6ac623` (latencia + verificación),
`1b69f8c` (corrida final M2-09).

**Régimen aplicado** (config `.superpowers/fabrica/config.md` §ACTUALIZACIÓN
2026-08-17, literal): "El gate de M2-09 es INFORMATIVO, no bloqueante. Se corre
(el harness ya existe), se anota en el reporte, y un resultado peor no para la
campaña." Lo que sigue siendo bloqueante: código incorrecto, contratos rotos,
veto AGPL, permalinks regenerados, tests rotos, números maquillados.

---

## Verificación primaria propia (comandos corridos por el consultor)

Toda la evidencia de abajo es de primera mano; no se ha dado por bueno ningún
número de los reportes sin recomputarlo.

### 1. Tests — 86/86 verdes ✓

`cargo test` en `.worktrees/m2-08/engine`, desglose real por suite:
lib 34 · buscador 14 · git_epoch 2 · indexer 16 · nota 7 · recall 5 ·
schema 2 · smoke 2 (1 ignored: `jina_es_embebe_a_768`, requiere modelo) ·
walker 4 = **86 passed, 0 failed**. Coincide con el reporte ("86/86 verdes,
72 base + 2 Tarea 1 + 12 Tarea 2").

### 2. Desempate determinista (M2-09a) — correcto en ambos sitios ✓

Leído el diff completo de `engine/src/buscador.rs`:

- `busca_vector` (~línea 207): `b.1.partial_cmp(&a.1)...then_with(|| a.0.cmp(&b.0))`
  — score descendente, permalink ascendente, operandos correctos.
- `fusiona` (~línea 287): mismo patrón sobre `.score`/`.permalink`.

Exactamente los dos sitios que el brief señalaba ("`engine/src/buscador.rs:204`
(en `busca_vector`)" y ":277 (en `fusiona`)"). Tests de empate reales en los
dos niveles (unitario con 5 claves empatadas en dos órdenes de inserción;
integración con 3 entidades de embedding idéntico), verdes.

### 3. Cap de bytes — hace lo que dice ✓

Leído `engine/src/recall.rs` entero. `aplica_cap` cuenta `str::len()` (bytes,
no chars) + 1 por `\n`; trunca por líneas ENTERAS; una nota entra solo si
TODAS sus líneas caben (unidad atómica nota+snippet); `notas_finales` alimenta
a la vez el texto y el JSON ⇒ **mismo conjunto en ambos formatos** (verificado
en código y en vivo: consulta real dio 5 notas en texto y 5 en JSON, idénticas).
`recorta_bytes` corta el snippet en frontera de carácter UTF-8 (test con `é`
multibyte). Truncado ⇒ `truncado: true` + aviso a stderr con líneas perdidas,
como exige el brief ("escribe a stderr una línea con cuántas líneas se
perdieron").

### 4. `exo recall` corrido en vivo contra el índice real ✓

Binario `engine/target/release/exo` de la rama + `kb-completa.db` (138 notas):

- **Arranque texto**: exit 0, cabecera `=== Recall exo (PARCIAL — no sustituye
  tu brief) ===`, rutas absolutas, cores en orden de ruta + recientes.
- **Arranque `--json`**: exit 0, **una sola línea**, envelope
  `{schema_version, command: "recall", data}`, `tier: "core"` presente,
  `score`/`snippet` a `null` (contrato del brief, literal: "`score` y `snippet`
  son `null` en modo arranque"). Nada humano en stdout.
- **Consulta texto y `--json`**: exit 0, 5 notas con score y snippet, JSON en
  una línea, mismo conjunto en ambos formatos.
- **Consulta sin hits** (`--min-similitud 0.99`): exit 1, stdout 0 bytes,
  mensaje claro a stderr — conforme al brief ("Un recall vacío (cero notas)
  no es exit 0 con bloque vacío: es error → exit 1").
- **DB inexistente**: exit 1, `error: DB no encontrada`.

### 5. Latencia — reproducida por el consultor ✓

- Arranque: hyperfine, 20 corridas → **mean 9,9 ms ± 1,3 ms** (max 11,8 ms).
  Consistente con el p95 = 14,0 ms del reporte. El releído del frontmatter de
  los 138 `.md` (el punto duro) **no es una bomba de latencia**: era además el
  camino que el brief ordenaba explícitamente ("extiende `FrontmatterLaxo` y
  `Nota` con `tier`... parsea el frontmatter de los ficheros que lista
  `notas`... mídelo"), y está medido con margen de 10× sobre los 100 ms.
- Hybrid en frío: 3 corridas → 0,98 s / 0,98 s / 0,98 s. Consistente con el
  p95 = 1032 ms del reporte; < 2,0 s (referencia bm: 4,4 s mediana).

### 6. Números de M2-09 — recomputados desde los jsonl crudos ✓

Script propio (independiente de `analyze.py`) sobre
`evals/retrieval-fase0/results/*-m209.jsonl` + `eval.jsonl` (55 etiquetadas):

- engine-hybrid **48/55** ✓ · bm-hybrid **39/55** (su mejor threshold del
  sweep, 0.35) ✓ · bm-vector 40 ✓ · bm-text 39 ✓ · testigos engine-vector
  44 ✓ y engine-fts 25 ✓ — todos idénticos a la tabla del verdict.
- Pareada engine-hybrid vs bm-hybrid\@0.35: **ARREGLA 13 · ROMPE 4** ✓.
- Atribución de las 4 roturas, recomputada con los testigos del mismo día:
  `fabrica campaign harness...` → vector HIT, fts miss ⇒ **fusion-miss**;
  las otras 3 (`fabrica roadmap campana...`, `Frente 9 lighthouses...`,
  `esa utilidad de terminal...`) ⇒ **both-miss**. **Exactamente 1 fusion-miss
  + 3 both-miss, como afirma el verdict.** La atribución no está maquillada.
- Paridad (pata 1): gold `corpus-bm.json` vs `notas` de `kb-completa.db` →
  **gold=138 engine=138 faltan=0 sobran=0**, archive=54, dotdirs=0. Diff = ∅,
  como exige el gate ("diff de permalinks a nivel entidad = ∅, cero
  tolerancia").
- Sanity-check del gate.md ("un engine-hybrid < 46/55 señala fusión mal
  calibrada"): 48 ≥ 46, no se dispara; además hybrid (48) > vector del mismo
  día (44): la fusión aporta, no resta.

### 7. Veto AGPL — sin indicios de copia ✓

`grep -ri "basic.memory" engine/src/`: todas las apariciones son
interoperabilidad legítima (lectura RO de `~/.basic-memory/config.json`,
comentarios de contexto). `schema.rs` declara "Nombres en castellano,
deliberadamente distintos de las tablas de basic-memory". Sin ficheros Python
en el crate. Sin permalinks regenerados (paridad ∅ lo confirma de facto).

---

## Adjudicación de los puntos duros

### Pata 2 falla su tope literal (rompe 4 > 2) — NO BLOQUEA

El gate pre-registrado (gate.md pata 2, literal): "engine-hybrid rompe ≤2 y
arregla ≥ las que rompe". Salieron ARREGLA 13 / ROMPE 4: la mitad del criterio
falla. El verdict lo dice sin rodeos ("'rompe ≤ 2': NO se cumple (4). Se dice
claro, no se maquilla") y la atribución es correcta (verificada arriba). Bajo
el régimen escrito ("El gate de M2-09 es INFORMATIVO, no bloqueante") un
número por debajo del objetivo honestamente reportado no bloquea. Juicio
técnico adicional del consultor: 3 de las 4 roturas son both-miss (contenido
irrecuperable por cualquier vía del engine — no es calibración de la fusión),
la referencia bm bajó de 43 a 39 con el corpus crecido, y el balance neto es
+9. No hay problema real de calibración tapado por el régimen: hay 1
fusion-miss residual, ya anotado como residuo abierto en el verdict de M2-09.

### Snippet = primer trozo, no el trozo que casó — NO BLOQUEA, con nota

El brief pedía "una línea de snippet del trozo que casó". La implementación
sirve el primer trozo (`orden = 0`). Desviación real de la letra del brief,
pero: (a) declarada por triplicado (doc de `recall.rs`, reporte punto 3, no
escondida); (b) con razón técnica verificada por el consultor — `fusiona`
agrega a nivel entidad por máxima similitud y no expone el trozo ganador, y el
arm FTS ni siquiera tiene el mismo concepto de trozo ganador; recuperarlo
exigiría re-embeber la query y recomputar similitudes por trozo, trabajo no
trivial para un dato informativo; (c) el brief admite desviaciones
("cualquier desviación del contrato de arriba con su razón"). ¿Engaña al
consumidor? El campo se llama `snippet`, no `match`; en la práctica el primer
trozo de estas notas es título+resumen, razonablemente representativo. **Nota
para M6**: si el consumidor del hook empieza a usar el snippet como evidencia
del match, hay que renombrarlo o implementar el trozo ganador de verdad.

### Releído de 138 `.md` en modo arranque — NO BLOQUEA

No es un atajo del executor: es el camino que el brief ordena literalmente
("No añadas columnas al índice... extiende `FrontmatterLaxo` y `Nota` con
`tier: Option<String>`... parsea el frontmatter de los ficheros que lista
`notas`... mídelo"). Medido por ellos (p95 14 ms) y reproducido por mí
(mean 9,9 ms): margen de 10× sobre el presupuesto de 100 ms. Además `tier_de`
tolera fallos por nota individual sin abortar el recall.

### `corpus-parity.py` 117→138 — LEGÍTIMO, no es mover la portería

El umbral `REF_ENTIDADES` es un guard del sellado ("PARADA sin sellar"),
**no es el gate**: el gate de la pata 1 es "diff de permalinks = ∅", que se
cumple exacto (138=138, ∅, verificado por mí contra la DB). El corpus creció
de verdad: el diff de `gold/corpus-bm.json` muestra las 23 entidades nuevas
con nombres y fechas reales de julio–agosto (bitácoras archivadas, research
de agosto, inbox). El motivo está escrito en el propio fichero ("Se sube
porque el corpus creció de verdad, no para hacer pasar nada") y la tolerancia
±14 mantiene el espíritu del guard original (117±12).

---

## Hallazgos NO bloqueantes (para el orquestador)

1. **Omisión de procedimiento en el verdict de M2-09**: gate.md pata 2 exige
   "cambios pareados del subgrupo [observation-sensitive] listados aparte en
   el verdict" y `m2-09-corrida.md` no los lista. Subsanado aquí — computado
   por el consultor desde `stratification.jsonl`: subgrupo obs-sensitive
   (47/55): engine 40/47 vs bm 35/47, **ARREGLA 9 / ROMPE 4**; resto (8):
   engine 8/8 vs bm 4/8, ARREGLA 4 / ROMPE 0. Sin degradación del subgrupo
   que adjudicar (mejora neta también dentro de él; las 4 roturas ya
   atribuidas caen todas ahí). Omisión de forma, no de fondo.
2. **"Golden envelope" de la pata 3(a)**: gate.md dice "valida contra golden
   envelope"; no existe fichero golden — la validación fue contra el contrato
   (test `recall_json_forma_del_contrato` + inspección). El consultor validó
   el envelope en vivo (una línea, `schema_version: 1`, forma completa).
   Matiz de procedimiento sin efecto material.
3. **FTS puro sin desempate secundario**: `busca` usa `ORDER BY score DESC`
   en SQL (`buscador.rs:75`) sin desempate por permalink — un empate exacto
   de bm25 daría orden no reproducible en el arm FTS. Fuera del alcance de
   M2-09a (el brief señalaba solo los dos sitios en Rust); anotar para si
   algún día importa el determinismo del arm FTS aislado.
4. **Cap atómico nota+snippet**: interpretación más estricta que la letra del
   brief (que habría impreso la línea principal y parado); declarada en el
   reporte (punto 4) y es la única coherente con "mismo conjunto en texto y
   JSON". Correcta a juicio del consultor.
5. **TDD parcial en Tarea 2** declarado honestamente en el reporte ("se
   declara esta diferencia de rigor en vez de narrar un proceso que no
   ocurrió"). La Tarea 1 sí fue rojo→verde estricto. Sin efecto en la calidad
   verificada.
6. **Residuo ya anotado**: 1 fusion-miss real (`fabrica campaign harness…`,
   vector HIT / fusión pierde) — candidato si se retoma la calibración; no es
   trabajo de esta campaña.

## Hallazgos bloqueantes

Ninguno.

---

**Fundamento del verdict**: código leído y correcto en los puntos críticos;
86/86 tests corridos por el consultor; CLI verificado en vivo en los cuatro
modos con exit codes conformes al contrato del brief; todos los números de
M2-09 recomputados independientemente desde los ficheros crudos y coincidentes
(incluida la atribución 1 fusion-miss / 3 both-miss); paridad ∅ verificada
contra la DB; sin indicios de violación AGPL; las desviaciones existentes
están declaradas con razón, como permite el brief, y ninguna cae en las
categorías bloqueantes del régimen.

GATE: MERGED — 2026-08-17 22:57 CEST
