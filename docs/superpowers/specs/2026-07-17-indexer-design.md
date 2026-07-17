# Indexer del engine exo — design spec (M2-02)

- **Fecha**: 2026-07-17 · **Estado**: redactada; pendiente de gate fable del par M2-02+M2-03 (plan campaña 1, Task 2 Step 4; D2)
- **Spec madre**: `2026-07-16-framework-unificado-design.md` (§4.2 componentes, §4.3 recortes, §6.2 reglas duras). **Spec M2**: `2026-07-17-m2-e1-read-design.md` (§2 D1/D6, §4 harness, §5 gate). **Veredicto**: `docs/superpowers/consultas/2026-07-17-m2-breakdown/consultor-verdict.md` (D1-D6, firmado).
- **Implementa**: M2-03 (walker+parser+FTS5+rebuild), M2-04 (aristas), M2-06 (chunks+vectores). Esta spec fija el contrato; el gold de paridad (`evals/e1-read/gold/corpus-bm.json`) queda sellado ANTES de la primera línea del indexer (D2: spec-first, "el gold se sella ANTES de implementar").

## 0. Alcance

Cubre el read-path del índice propio: qué entra al corpus, en qué tablas vive, cómo se actualiza y con qué envelope sale. Fuera de alcance (recortes ya adjudicados, spec madre §4.3 y spec M2 §1): gramática observations/relations en el índice, move, build_context, daemon, cloud/sync, ranking por grafo (M2-04 solo indexa aristas; veredicto D2).

## 1. Contrato de corpus

Las 6 reglas duras de la spec madre §6.2, **literales**:

1. > **El indexer honra el `permalink:` del frontmatter y JAMÁS lo regenera** (verificado: `ensure_frontmatter_on_sync` los persiste, 112/112 notas). Genera solo para notas nuevas. El read-path acepta identifier con/sin prefijo de proyecto y resolución por título (read_note = 83% del tráfico real).
2. > **Recencia = git, no mtime ni created_at del índice** (ninguna nota tiene `created:`; un clone fresco resetea mtimes). kbx targets ya lo hace así.
3. > **Exclusión de dotdirs replicada** (.claude/, .omc/, .superpowers/: 24 .md fuera del índice actual) — sin esto el side-by-side de E1 compara corpus distintos.
4. > **archive/ SE INDEXA** (como hoy; 32% del índice): cambiar corpus y motor a la vez impediría atribuir deltas en E1. Si molesta en ranking, se recorta post-E1 con datos.
5. > Entidades no-markdown (5, permalink NULL): no se indexan en v1.
6. > Links a notas inexistentes se toleran (to_id NULL), jamás error de indexado.

### 1.1 Mapa regla → check

Cada regla mapeada a verificación ejecutable (plan Task 2 Step 1.1). Los checks del probe existen hoy; los del indexer son tests a escribir en M2-03/M2-04 (nombres fijados aquí como contrato).

| # | Regla §6.2 | Mecanismo de verificación | Comando / test |
|---|---|---|---|
| 1 | Permalinks honrados, jamás regenerados | El gold congela el set de permalinks del frontmatter antes de implementar; un permalink regenerado aparece como `FALTA`+`SOBRA` en el diff (exit 1). Además, test unitario del parser. | `python3 evals/e1-read/harness/corpus-parity.py --diff <engine.db>` (existe) + test M2-03 `permalink_del_frontmatter_se_honra` (a escribir) |
| 2 | Recencia = git | `notas.git_epoch` se compara contra `git log -1 --format=%ct -- <ruta>`; mtime no participa en recencia (§3). | test M2-03 `recencia_viene_de_git` (a escribir) |
| 3 | Dotdirs excluidos | Lado bm: `--capture-bm` PARA sin sellar (exit 1) si `n_dotdirs_dentro != 0` (umbral del brief M2-02, codificado en el script). Lado engine: un dotdir indexado sale como `SOBRA` en el diff. | `python3 evals/e1-read/harness/corpus-parity.py --capture-bm` (existe) + test M2-03 `walker_excluye_dotdirs` (a escribir) |
| 4 | `archive/` SE indexa | El gold registra `n_archive` (35 en la captura 2026-07-17, 30% — referencia §6.2: 32%); si el engine excluyera `archive/`, sus notas salen como `FALTA` en el diff. | `python3 evals/e1-read/harness/corpus-parity.py --diff <engine.db>` (existe) |
| 5 | No-markdown no se indexan | Lado bm: el `WHERE permalink IS NOT NULL` del probe deja fuera las 5 entidades no-md (verificado en la captura: 120 entidades, 115 con permalink). Lado engine: el walker solo admite `*.md`; un no-md indexado saldría como `SOBRA`. | `--capture-bm` / `--diff` (existen) + test M2-03 `walker_solo_markdown` (a escribir) |
| 6 | Links inexistentes se toleran | Schema: `aristas.destino_permalink` admite NULL (§2); el indexado de una nota con `[[link roto]]` termina exit 0. | test M2-04 `link_roto_no_es_error` (a escribir) |

### 1.2 Schema real de `memory.db` (probe RO 2026-07-17) y ajustes al probe del plan

Inspección con `mode=ro` (permitida y exigida por el plan §Global Constraints, D6; el veto AGPL aplica al código, no a la DB local del usuario):

- La tabla es `entity` y tiene las columnas que el plan asumía (`permalink VARCHAR` nullable, `file_path VARCHAR NOT NULL`) — el SELECT del plan era válido tal cual.
- **Ajuste 1 — filtro de proyecto**: `entity` tiene `project_id` (FK a `project`). Hoy solo existe el proyecto `kb-demo` (id 1), pero el probe filtra explícitamente con JOIN a `project.name = 'kb-demo'`: el corpus del side-by-side es kb-demo (spec M2 §1) y un segundo proyecto futuro contaminaría el gold en silencio.
- **Ajuste 2 — umbrales de parada codificados**: `--capture-bm` se niega a sellar (exit 1, sin escribir el gold) si `n_dotdirs_dentro != 0` o si el conteo sale de 117±12 (umbral del brief M2-02). Un gold malo es peor que ningún gold: es el oráculo de M2-03/M2-09.
- **Ajuste 3 — HEAD de kb-demo dentro del gold**: campo `kb_demo_head` (además del mensaje de commit), leyendo la ruta de la KB de `projects.kb-demo.path` en config (D6). El gate D4 exige pinear el estado de la KB; con el HEAD dentro del fichero el gold es autocontenido.
- Conteos de la captura: 120 entidades, 115 con permalink (5 no-md con permalink NULL — exactamente las 5 de §6.2), 0 dotdirs, 35 en `archive/`. El 115 vs el 117 de referencia de §6.2 es drift normal de la KB (riesgo 2 del veredicto); dentro del umbral ±10%.
- **Forma de los permalinks — verificado**: llevan prefijo de proyecto (`kb-demo/...`) tanto en el índice como en el frontmatter de las notas (`permalinks_include_project: true` en config; comprobado en `core/doctrina-agentes.md`: `permalink: kb-demo/core/doctrina-agentes`). Como el engine copia el frontmatter literal (§6.2 regla 1), el `--diff` compara byte a byte sin normalización — la normalización solo existe en el `norm()` de `analyze.py` para el harness de retrieval (spec M2 §4).

## 2. Schema SQLite del engine

Nombres **propios en castellano** — deliberadamente imposibles de calcar de basic-memory (`entity`/`relation`/`search_index`/`search_vector_chunks`); regla dura del brief y patrón sugerido por el plan (Task 2 Step 1.2). Una sola DB SQLite; sqlite-vec registrado vía `exo_engine::abre_db_en_memoria()`-mismo camino (M2-01).

```sql
CREATE TABLE notas (
  permalink  TEXT PRIMARY KEY,   -- del frontmatter, jamás regenerado (§6.2 regla 1)
  ruta       TEXT NOT NULL UNIQUE, -- relativa a la raíz de la KB (projects.kb-demo.path)
  titulo     TEXT NOT NULL,
  tipo       TEXT,               -- frontmatter `type`, NULL si falta
  mtime      REAL NOT NULL,      -- SOLO detección de cambios al invocar (§3); jamás recencia
  git_epoch  INTEGER             -- unix epoch del último commit que tocó `ruta` (§6.2 regla 2)
);

CREATE VIRTUAL TABLE notas_fts USING fts5(
  titulo, cuerpo,
  permalink UNINDEXED,
  tokenize='unicode61 tokenchars 0x2F'
);

CREATE TABLE aristas (
  origen            TEXT NOT NULL REFERENCES notas(permalink),
  destino_texto     TEXT NOT NULL,   -- el [[wikilink]] tal cual aparece en la nota
  destino_permalink TEXT,            -- NULL si la nota destino no existe (§6.2 regla 6)
  UNIQUE (origen, destino_texto)
);

CREATE TABLE trozos (
  id        INTEGER PRIMARY KEY,
  permalink TEXT NOT NULL REFERENCES notas(permalink),
  orden     INTEGER NOT NULL,     -- posición del trozo dentro de la nota
  texto     TEXT NOT NULL,
  UNIQUE (permalink, orden)
);

CREATE VIRTUAL TABLE vectores USING vec0(embedding float[768]);
-- rowid de `vectores` = trozos.id (patrón del plan: "vec0 float[768] + rowid→chunk")
```

Decisiones y citas:

- **`notas(permalink PK, ruta, titulo, tipo, mtime, git_epoch)`**: la forma que el plan propone literalmente (Task 2 Step 1.2). `permalink` como PK materializa la regla 1: la clave del índice ES el permalink del frontmatter, no un id sintético regenerable.
- **FTS5 sobre `titulo`+`cuerpo`**: mandato literal del plan (Step 1.2 "`notas_fts` (FTS5 sobre titulo+cuerpo)").
- **Tokenizer `unicode61 tokenchars 0x2F`**: el mismo observado en el `search_index` vivo (probe RO 2026-07-17). Motivo: comparabilidad del side-by-side — un tokenizer distinto haría inatribuibles los deltas de retrieval (mismo argumento que fija §6.2 regla 4: "cambiar corpus y motor a la vez impediría atribuir deltas"). Sin `prefix=` en v1 (YAGNI; bm lo usa para búsqueda por path que el contrato de search de E1 no exige; si el side-by-side muestra misses de prefijo, se añade con datos — la atribución por `search_type` de M0 §4.1.4 los destaparía).
- **`aristas` con `destino_permalink` NULL tolerado**: regla 6 de §6.2 aterrizada a schema (el `to_id NULL` de la regla, con nombre propio). Solo indexado; el ranking por grafo queda fuera de E1 (veredicto D2).
- **`vectores` vec0 float[768]**: dims de la config de producción M0 (spec M2 §2: jina-es/768/threshold 0.35; verificado vigente en `config.json`). El índice vivo de bm usa exactamente `vec0(embedding float[768])` (probe RO) — misma familia de extensión, compatibilidad demostrada en producción.

### 2.1 Chunking propio (documentado: tamaño/solape y por qué)

Mandato: spec madre §4.2 vía plan Step 1.2 ("chunking propio documentado (tamaño/solape y por qué)"). Restricción de diseño: el ground truth del eval es **a nivel de nota** y la paridad de chunks NO se exige (spec M2 §4: "Paridad de chunks NO exigida... counts de chunks solo informativos") — así que se elige lo simple (criterio del brief M2-02):

- **Unidad**: bloques markdown (separados por línea en blanco o heading), empaquetado greedy de bloques consecutivos hasta **máx 900 caracteres** por trozo; un bloque que solo exceda el máximo se corta duro en 900.
- **Solape: 0.** El solape añade filas de índice y complejidad sin beneficio medible cuando el acierto se puntúa a nivel de nota (spec M2 §4); si el hit@5 vector de M2-06 queda lejos de la referencia 43/55, el solape es el primer parámetro del sweep.
- **Por qué 900**: paridad empírica de granularidad con el índice vivo — chunks de bm medidos en RO 2026-07-17: mediana 250 chars, p90 718, **máx 900** (6.158 chunks). Igualar el techo de granularidad evita atribuir deltas del arm vector al tamaño de chunk en vez de al motor (mismo argumento de atribución de §6.2 regla 4).
- **Provisional, se calibra en M2-06** (tamaño y solape son parámetros del sweep de M2-06/07; el contrato estable es `trozos(permalink, orden, texto)` + `vectores` rowid→trozo, que no cambia al calibrar).

## 3. Incrementalidad y rebuild

Mandato del plan (Task 2 Step 1.3): "mtime/git al invocar, sin daemon; `rebuild` = borrar DB + reconstruir (primera clase)". Spec madre §4.2: "Incremental por mtime/git al invocar, sin daemon salvo que duela"; §4.3: "`rebuild` como comando de primera clase ('corrupción de índice = borrar y rebuild; jamás cirugía sobre la DB')".

- **`exo index --db <ruta>`** (al invocar; sin daemon, sin watch):
  1. Walk de la KB desde `projects.kb-demo.path` (config RO, §5): solo `*.md`, excluyendo `.claude/`, `.omc/`, `.superpowers/` (§6.2 reglas 3 y 5); `archive/` dentro (regla 4).
  2. Por nota: si `mtime` del fichero == `notas.mtime` → skip; si difiere → reparse y reindex completo de la nota (fila en `notas`, `notas_fts`, sus `aristas`, sus `trozos`+`vectores`), refrescando `git_epoch`.
  3. Notas presentes en `notas` y ausentes del walk → se borran (con sus filas dependientes).
  4. mtime es SOLO detección de cambio; la recencia que consume el ranking/recall es `git_epoch` (§6.2 regla 2). Un clone fresco resetea mtimes y provoca un reindex completo una vez — coste aceptado: el re-embed del corpus entero son minutos (spec madre §4.1.3: 117 entidades / ~5k chunks / ~1.7 MB), y evita mantener checksums para un caso que ocurre casi nunca (lo simple que cubre el caso).
- **`exo rebuild --db <ruta>`**: borra el fichero de la DB y reconstruye de cero. Primera clase, no fallback. Oráculo de M2-03: rebuild idempotente (dos rebuilds seguidos ⇒ mismo diff ∅ contra el gold).

## 4. Envelope JSON

Adopción de la forma del envelope de kbx v1 (`~/Documentos/proyectos/kbx/internal/envelope/envelope.go`; mandato del plan Task 2 §Interfaces y Step 1.4):

```json
{"schema_version": 1, "command": "<subcomando>", "data": { ... }}
```

- **`schema_version` propio de exo, arrancando en 1** (plan Step 1.4). Es el contrato exo↔consumidores, independiente del de kbx. Política de versionado (doctrina kbx: "skills consuming kbx pin on schema_version"): cambio breaking en la forma de `data` ⇒ bump; campos aditivos no lo suben.
- **Emisión**: todo output `--json` del binario sale envuelto, una línea, newline-terminated (patrón `envelope.Write` de kbx).
- **Regla de gating para consumidores** (doctrina literal del envelope de kbx, adoptada): *gate en el exit code del proceso, JAMÁS en campos informativos de `data`*. En kbx: "gate on the process exit code... NEVER gate on budget.tiers[].exceeded... those are informational". Para exo: `replay-engine.py` (M2-05), el golden-envelope check (M2-08) y los hooks futuros deciden éxito/fallo por exit code (0 = ok, ≠0 = fallo); `elapsed_s`, `score`, conteos y demás campos de `data` son informativos — se registran, no se gatea sobre ellos.

### 4.1 Contrato de `data` para `search` (lo consume `replay-engine.py`, M2-05)

Fijado en el plan (Task 2 §Interfaces) y el brief M2-02:

```json
{
  "query": "texto de la consulta",
  "search_type": "fts | vector | hybrid",
  "elapsed_s": 0.012,
  "results": [
    {"permalink": "core/doctrina-agentes", "type": "entity", "score": 0.83}
  ]
}
```

- `query`: la consulta tal cual llegó (tras precedencia flags > config).
- `search_type`: explícito siempre — es lo que permite la atribución de misses FTS/vector/threshold del harness (spec madre §4.1.4).
- `elapsed_s`: segundos (float) de la búsqueda, medidos dentro del binario. Informativo (la latencia gateable de D4 se mide fuera, con hyperfine — spec M2 §5 pata 3).
- `results`: ordenados por `score` descendente. `permalink` tal cual vive en `notas.permalink` (la normalización para comparar arms es del `norm()` de `analyze.py`, no del engine — spec M2 §4). `type`: `"entity"` en v1 — resultados SIEMPRE a nivel entidad, nunca filas observation (gotcha M0, spec M2 §4). `score`: el score del `search_type` usado (fusionado en hybrid); su escala es informativa, no contractual.
- El contrato de `data` de `recall` se fija en M2-08 contra golden envelope (spec M2 §5 pata 3); esta spec solo fija que saldrá envuelto en el mismo envelope v1.

## 5. Config

Adjudicación D6, literal en spec M2 §2: mientras dure el side-by-side, el engine lee **read-only** `~/.basic-memory/config.json`:

| Clave | Uso en el indexer/search |
|---|---|
| `projects.kb-demo.path` | raíz del walk (anidada: `projects["kb-demo"]["path"]`, verificado en el fichero vivo) |
| `semantic_embedding_model` | modelo fastembed (hoy `jinaai/jina-embeddings-v2-base-es`) |
| `semantic_embedding_dimensions` | dims de `vectores` (hoy 768) |
| `semantic_min_similarity` | threshold del arm vector/hybrid (hoy 0.35; calibración en M2-07) |

- **Precedencia: flags CLI > config** (D6). Sin config propia de exo en esta fase — por eso `--db <ruta>` es flag obligatorio en E1, no default persistente: un default hardcodeado sería config propia encubierta, y los harness (probe, replay-engine) pasan la ruta explícita de todos modos. La config propia entra en M5a (o antes si duele).
- El acoplamiento caduca por diseño: side-by-side >4 semanas ⇒ se decide (spec M2 §2).
- E1 no escribe NADA: ni en la KB, ni en `memory.db`, ni en `config.json` (plan §Global Constraints; probes siempre `mode=ro`).

## 6. Versión pineada de sqlite-vec

**Pendiente del valor que fije M2-01** (no mergeado al redactar esta spec; su `Cargo.toml` aún no declara sqlite-vec). Al cerrar el gate del par 02+03 se anota aquí la versión exacta:

- `Cargo.toml`: `sqlite-vec = "=X.Y.Z"` — **con `=`, jamás `^`** (spec M2 §2 y §7 riesgo 4: pre-v1 con breaking changes anunciados; un `^` accidental es riesgo silencioso).
- Dato de compatibilidad: el índice vivo de bm opera `vec0(embedding float[768])` en producción sobre esta misma máquina (probe RO 2026-07-17).

<!-- ANOTAR AQUÍ al merge: sqlite-vec = "=X.Y.Z" (fijada por M2-01, commit <hash>) -->
