# Verdict — gate de merge campaña 1 de M2: ramas `m2-01` + `m2-02`

- **Fecha**: 2026-07-17 · **Adjudicador**: consultor Fable fresco (régimen spec madre §8, delegado; sin participación en ninguna fase de lo juzgado; sin acceso al razonamiento del orquestador — solo deliverables + criterio). Dispatch agrupado (2 gates) por presupuesto, declarado en el ledger.
- **Criterio**: spec M2 `2026-07-17-m2-e1-read-design.md` (§2, §3, §4, §5 pata 1, §7) · veredicto D1-D6 firmado (`consultas/2026-07-17-m2-breakdown/consultor-verdict.md`) · plan `2026-07-17-m2-campana1-scaffold-spec-indexer.md` (Global Constraints + Tasks 1-2) · spec madre §6.2/§8 · `envelope.go` de kbx · veto AGPL.

## VEREDICTO

- **`m2-01` (scaffold engine, commit `cb88b70`): MERGED.**
- **`m2-02` (spec indexer + gold + envelope, commits `3bd9315`..`fba17f9`): MERGED.** El envelope exo v1 queda **FIJADO** como superficie irreversible (spec madre §8) con este merge.

## Verificación primaria propia (condición b del régimen — todo re-corrido por mí, no leído de resúmenes)

### m2-01

1. `cargo test --manifest-path .worktrees/m2-01/engine/Cargo.toml` → **ok. 2 passed; 0 failed; 1 ignored** (`fts5_disponible`, `sqlite_vec_disponible` PASS; `jina_es_embebe_a_768` ignored). Idéntico a lo citado en el package.
2. `cargo test ... -- --ignored` → **re-corrido por mí también** (barato con el modelo ya cacheado): **ok. 1 passed** en 1.07s. El dato de frío (30.52s, descarga fresca 0.6 GB) está citado en el ledger (línea de estado M2-01) y en el package; nota de proceso abajo.
3. `cargo build --release && exo --version` → **`exo 0.1.0`**.
4. Diff completo contra main (`1f22112`): 6 ficheros, todos `engine/` + 1 línea `.gitignore` (`engine/target/`). Nada fuera de scope, cero líneas de indexer.
5. `Cargo.toml`: **`sqlite-vec = "=0.1.9"`** — pin exacto con `=`, no `^` (riesgo 4 de spec M2 §7 cerrado). Lockfile: **0 apariciones de openssl/native-tls, rustls presente** — la desviación rustls-tls es exactamente lo declarado.
6. Toolchain del sistema verificada: `rustc 1.97.1`.

### m2-02

1. Re-corrí desde el worktree `python3 evals/e1-read/harness/corpus-parity.py --capture-bm` → **`sellado: 115 entidades, archive=35, dotdirs_dentro=0, head=28f153a33fcd`, exit 0**, y `git diff` + `git status --porcelain` **vacíos**: el gold es idempotente byte a byte y la KB no ha driftado desde el sellado (head idéntico dentro del propio gold: `28f153a33fcd...a8a2a`).
2. Probe RO propio contra `memory.db` (independiente del de la spec y del verificador): **120 entidades kb-demo, 115 con permalink, 5 sin permalink = exactamente las 5 no-md de §6.2** (`inventory.json`, `main_es.tex`, `developercv.cls`, `fontawesome.pdf`, `main_en.tex`); **proyecto único** (id 1, kb-demo) — el JOIN del probe no excluye nada hoy y protege el gold mañana; **115/115 permalinks con prefijo `kb-demo/`**.
3. Claims load-bearing de la spec verificados en fuente primaria: tokenizer del `search_index` vivo = **`tokenize='unicode61 tokenchars 0x2F'`** literal (y `prefix='1,2,3,4'` existe con el comment de paths — el recorte de exo está bien caracterizado); chunks **n=6158, máx=900** (sostiene el techo 900 de §2.1); `envelope.go` de kbx leído por mí: `{schema_version, command, data}`, `SchemaVersion=1`, newline-terminated, doctrina de gating por exit code literal en el comment.
4. Diff completo contra main: exactamente 3 ficheros (spec .md, probe .py, gold .json), 365 líneas. **Cero líneas de indexer.**
5. Commit `fba17f9` inspeccionado línea a línea: toca **exactamente** F1 (ejemplo del contrato → `kb-demo/core/doctrina-agentes`) y F2 (§6 rellenada con `=0.1.9` + commit `cb88b70` + MSRV 1.97). **Nada nuevo introducido.**

## Adjudicaciones explícitas pedidas por el dispatch

1. **Desviaciones de m2-01 — ACEPTADAS como quedaron.**
   - *rustls-tls*: verificada en lockfile (0 openssl); des-riesga la build nativa (sin `openssl-sys`/pkg-config) sin tocar contrato alguno. Aceptación de la review opus compartida.
   - *Toolchain 1.92→1.97.1*: es estado de máquina, pero **no exige más que la declaración a Paul ya hecha**. Racional: no es línea roja del régimen (no destructivo, no externo, no cambio de permisos; reversible vía rustup), la causa es real (`cfg_select!` de libsqlite3-sys 0.38.1 — sin ella el crate no compila), y la parte durable está bien cosida: **MSRV ≥1.97 anotada en la spec del indexer §6** (fba17f9), que es el artefacto que sobrevive a la sesión. Lo único que pediría a futuro (no bloqueante) es que cambios de toolchain queden también en el ledger como línea propia, no solo en la declaración de sesión.
2. **Envelope exo v1 — FIJADO como superficie irreversible con este merge.** Juicio de fondo, no de trámite: la forma `{schema_version, command, data}` es mínima, versionada desde 1, probada en producción en kbx, y la regla de gating (exit code, jamás campos informativos) es una adaptación *estrictamente más fuerte* de la doctrina kbx — apta para hooks. El contrato de `data` de search (`{query, search_type, elapsed_s, results[{permalink,type,score}]}`) es **idéntico** al de plan Task 2 §Interfaces y al schema jsonl que D3 fija para el harness: **`replay-engine.py` (M2-05) lo consume tal cual**, sin transformación — `search_type` explícito da la atribución de misses, `permalink` prefijado casa con `eval.jsonl` y con el `norm()` de `analyze.py` (que no quita prefijos), `type:"entity"` mata el gotcha de observation-rows de M0. Firmo la fijación.
3. **Fixes inline fba17f9 — RESUELVEN las 2 objeciones sin introducir nada.** F1 era una línea (ejemplo ahora consistente con §1.2 y con eval.jsonl); F2 era exactamente la condición de merge que el verificador adversarial impuso ("no mergear con el hueco si M2-01 ya fijó el valor") — M2-01 lo fijó (`=0.1.9`, `cb88b70`) y la spec lo anota con cita + MSRV. Diff inspeccionado: ninguna otra línea tocada.
4. **Gold — VÁLIDO como contrato firme para M2-03.** 115 entidades con head de kb-demo (`28f153a`) dentro del fichero (autocontenido, pata 1 del gate D4 pineable), umbrales de parada codificados en el probe (dotdirs≠0 y 117±12 abortan ANTES de escribir — un gold malo no puede sellarse en silencio), verificado independientemente tres veces (spec-writer, verificador adversarial, yo) con re-captura idempotente. Los 5 sin permalink son genuinamente los no-md de §6.2, no un hueco.

## Mandato de disenso (condición c): qué busqué para objetar, rama a rama

**m2-01** — busqué y no encontré:
- Pin `^` accidental de sqlite-vec (riesgo silencioso spec M2 §7.4): es `=0.1.9`.
- Scope fuera de `engine/`: diff limpio (solo +1 línea .gitignore).
- Smokes que no prueban lo que dicen (mocks): leí los tests — ejercitan FTS5 real (`CREATE VIRTUAL TABLE ... fts5` + MATCH), vec0 real (`vec_version()` + tabla float[768]) y embedding real (descarga HF + 768 dims). No hay mocks.
- `unsafe` incorrecto en el registro de sqlite-vec: es el patrón canónico documentado del crate (`sqlite3_auto_extension` + transmute de fn pointer); SQLite dedupe registros repetidos.
- openssl escondido tras la desviación rustls: 0 hits en lockfile.
- Calcos AGPL: el código sale del template del plan; nada de basic-memory.
- **Hallazgos menores, NO bloqueantes** (los dejo anotados): (a) el feature `image-models` de fastembed sigue en `Cargo.toml` — ya flaggeado por la review opus y diferido a M2-03; sostengo el diferimiento (superficie de build, no de contrato). (b) No existe fichero `reports/m2-01-report.md` — el output del executor vive en ledger + package; como lo re-verifiqué todo de primera mano, el audit trail queda cubierto por este verdict, pero el patrón reports/ se rompió para este item (nota de proceso al orquestador).

**m2-02** — busqué y no encontré:
- Gold contaminado o driftado: re-captura idempotente, head idéntico, proyecto único verificado.
- Regla §6.2 ausente o con check vacuo: las 6 literales (cotejadas contra spec madre §6.2 por mí), mapa §1.1 completo; las que solo mapean a tests de M2-03/M2-04 tienen nombre de test fijado como contrato y no existe check pre-indexer posible (mismo juicio que el verificador, alcanzado por separado).
- Divergencia con el envelope de kbx o tergiversación de su doctrina: leí `envelope.go` completo; la adaptación es más fuerte, no distinta.
- Calcos de schema AGPL: `notas/notas_fts/aristas/trozos/vectores` vs `entity/relation/search_index/search_vector_chunks` — nombres propios; la única coincidencia (`embedding` en vec0) es el idiom de la extensión, no un calco. Nadie abrió el repo de basic-memory (solo DB/config del usuario en RO, permitido por D6/plan).
- Cita que no sostiene la decisión: verifiqué yo mismo las dos más load-bearing (tokenizer y stats de chunks que justifican el 900) contra la DB viva — clavadas.
- Debilidad del oráculo ante un indexer que regenere permalinks desde path: el prefijo `kb-demo/` del frontmatter (que el path no tiene) haría explotar el diff en 115 FALTA+SOBRA — el oráculo caza el fallo de la regla 1 por construcción.
- Contrato de search insuficiente para M2-05: cotejado campo a campo contra D3 y plan §Interfaces (adjudicación 2 arriba).
- **Sin hallazgos nuevos.** Las 2 objeciones del verificador adversarial estaban bien calibradas (menores) y están cerradas.

## Condiciones del régimen

(a) Fresco: sí — este dispatch es mi primera participación en M2. (b) Verificación primaria: sección arriba, toda re-corrida. (c) Disenso: sección arriba, por rama. (d) Verdict-artifact: este fichero, commiteado en la rama `m2-02` antes del merge (patrón `evals/retrieval-fase0/verdict/`).

**El merge lo ejecuta el orquestador vía GATE-EXEC (pasos separados); este verdict no toca main.** Tras el merge de m2-02, el gold es contrato firme y M2-03 queda desbloqueado per D5.
