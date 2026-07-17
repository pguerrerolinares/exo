# M2 — E1 read: design spec

- **Fecha**: 2026-07-17 · **Estado**: firmada (adjudicaciones por consultor Fable delegado, firma de Paul)
- **Spec madre**: `2026-07-16-framework-unificado-design.md` (§4.2 componentes, §4.4 estrangulamiento, §4.5 Rust, §6.2 reglas duras del indexer). Esta spec NO la repite: la aterriza en items ejecutables.
- **Audit trail**: brief + veredicto completo en `docs/superpowers/consultas/2026-07-17-m2-breakdown/`. GATE-HUECO-M2 abierto por Paul en sesión 2026-07-17 (commit en `.superpowers/fabrica/config.md`).

## 1. Objetivo y no-objetivos

**Objetivo**: indexer + search propios en Rust corriendo side-by-side contra basic-memory sobre el mismo corpus (kb-demo), con el eval set de M0 como oráculo. Deliverable: **"engine capaz de servir el recall, demostrado"** = (a) gate numérico §5 pasado, (b) `exo recall --json` con el envelope pactado, (c) latencia compatible con hooks.

**No-objetivos (explícitos)**: cutover de nada instalado (el hook de recall es M6); write-path (M4); MCP (M5a); ranking por grafo (solo indexado de aristas); gramática observations/relations; move; build_context; daemon; cloud/sync. `rebuild` es comando de primera clase: corrupción de índice = borrar y reconstruir, jamás cirugía.

## 2. Decisiones estructurales (adjudicadas D1, D6)

- **Layout**: crate Rust único en `engine/` con `src/lib.rs` + `src/main.rs`; binario **`exo`** con subcomandos (`exo index`, `exo search`, `exo recall`, `exo rebuild`). Sin cargo workspace hasta que E3 (rmcp) lo exija — la workspace-ificación posterior es refactor mecánico. kbx (Go) convive; la mención `engine search --json` en la spec madre era ilustrativa: los scripts de M6 se escriben contra `exo`.
- **Config**: mientras dure el side-by-side, el engine lee **read-only** la config viva de basic-memory (`~/.basic-memory/config.json`: `semantic_embedding_model`, `semantic_embedding_dimensions`, `semantic_min_similarity`, `projects.kb-demo.path`), precedencia flags CLI > config. Una sola fuente de verdad = imposible la divergencia silenciosa entre arms. Config propia de exo entra en M5a (o antes si duele). El acoplamiento caduca por diseño: side-by-side >4 semanas ⇒ se decide.
- **Deps**: rusqlite con FTS5 bundled; sqlite-vec **pineado con versión exacta `=x.y.z`** en `Cargo.toml` (la versión concreta se fija en M2-01 y se anota en la spec del indexer; pre-v1, breaking changes anunciados); fastembed-rs con `jinaai/jina-embeddings-v2-base-es`/768/threshold 0.35 (config de producción M0).
- **Veto AGPL**: de basic-memory se copia el **diseño** de la fusión (fórmula `max(v,f)+bonus·min(v,f)`, clave `(type,id)`, gate FTS, normalización BM25, threshold configurable — descripción en spec madre §4.2), jamás código ni vendorizado. Los briefs de executors de M2-07 llevan la prohibición explícita de abrir el repo de basic-memory.

## 3. Breakdown en items (adjudicado D2)

| # | Item | Lane | Oráculo / gold |
|---|------|------|----------------|
| M2-01 | Scaffold: crate `engine/`, bin `exo`, FTS5 bundled compila, sqlite-vec pineado, fastembed-rs embebe 1 frase a 768 dims | mecánica | `cargo build && cargo test` + smoke citados |
| M2-02 | Spec del indexer + gold de paridad de corpus + envelope JSON fijado (adopta el envelope versionado de kbx) | diseño (fable en cabeza) | gold sellado antes de implementar |
| M2-03 | Indexer: walker + parser frontmatter + FTS5 + `rebuild`, incremental mtime/git | ejecución contra gold de 02 (02+03 = UN gate fable) | diff de paridad = ∅ + rebuild idempotente |
| M2-04 | Grafo de wikilinks: solo indexado de aristas (`to_id NULL` tolerado); ranking por grafo FUERA de E1 | mecánica (filler paralelizable) | extractor de referencia por script + spot-check |
| M2-05 | Arm engine en harness M0: `replay-engine.py`, mismo schema jsonl | mecánica | `analyze.py <arm-engine>` sin errores |
| M2-06 | Embeddings + tabla vectorial (jina-es/768, chunking propio, sqlite-vec) | mecánica | hit@5 vector vía harness; referencia bm 43/55 |
| M2-07 | Fusión + calibración threshold (clean-room desde §4.2) | diseño | gold = eval set M0; sweep con procedimiento `analyze.py` |
| M2-08 | `exo recall --json` + medición de latencia | mecánica | golden envelope + presupuestos de latencia de §5 pata 3 (hyperfine) |
| M2-09 | Side-by-side final: ambos arms el mismo día + gate | corrida + gate fable fresco | `evals/e1-read/gate.md` pre-registrado |

Orden razonado: 02 antes de 03 compra el oráculo (spec-first); 05 en cuanto exista `exo search --json` para que 06-08 se midan por comando; 07 tras tener señal FTS y señal vector. Merges mecánicos adyacentes (04+05, 06) pueden agrupar gate si el ledger lo pide.

## 4. Harness side-by-side (adjudicado D3)

- **Reuso, no reescritura**: `analyze.py` intacto (la normalización de permalinks vive en su `norm()`); `replay.py` NO se generaliza (está acoplado al baile de `config.json` del CLI de bm que el engine no necesita).
- Nuevo hermano `replay-engine.py` (~80 líneas): invoca `exo search --json`, escribe el mismo contrato `{query, search_type, elapsed_s, results[{permalink,type,score}]}`.
- **Paridad de corpus**: script probe **read-only** (patrón `stratify.py`, URI `mode=ro`) que diffea sets de permalinks a nivel **entidad** entre el índice del engine y el `search_index` de basic-memory — nunca contra output del CLI (gotcha M0: el CLI agrega filas observation a su entidad). Paridad de chunks NO exigida (el ground truth es a nivel de nota); counts de chunks solo informativos.

## 5. Gate de cierre (adjudicado D4 — pre-registrar en `evals/e1-read/gate.md` y commitear ANTES de la corrida final)

Tres patas obligatorias:

1. **Paridad de corpus**: diff de permalinks a nivel entidad = **∅**, cero tolerancia; exclusiones §6.2 verificadas explícitamente (dotdirs fuera, `archive/` dentro, 5 entidades no-md fuera, 0 permalinks regenerados).
2. **Retrieval pareado**: ambos arms re-corridos el mismo día sobre el mismo estado de la KB (commit de kb-demo pineado en el verdict); prohibido comparar contra `results/` de julio. Gate: engine-hybrid **rompe ≤2 y arregla ≥ las que rompe** vs bm-hybrid (referencia hoy 43/55). Subgrupo observation-sensitive examinado aparte. `cge bitácora` (fusion-miss conocido) = diagnóstico informativo, no exigible.
3. **Recall demostrado**: (a) `exo recall --json` valida contra golden envelope; (b) latencia: arranque+consulta FTS-only **p95 < 100 ms**; hybrid en frío con carga de modelo **p95 < 2.0 s** (referencia bm hoy: mediana 4.4 s).

Ambigüedad o empate ⇒ consultor fable adjudica con este texto delante; los números no se renegocian post-hoc.

## 6. Primera campaña de fábrica (adjudicado D5)

- Entra solo **M2-01 + M2-02**; M2-03 arranca únicamente si 02 cierra su gate esa misma noche.
- **Reserva fable: 6 → 8 dispatches/noche** para esta campaña (señal pre-anunciada en config tras el 60% de M1a); cap semanal ≤20% intacto, contador gates-vs-spec separado.
- El detalle de items de esta spec §3 se traslada al config de fábrica al abrir la campaña (el config decía "no detallar más hasta abrir el gate" — ya está abierto).

## 7. Riesgos a vigilar (del veredicto)

1. **Modelo en frío** (0.64 GB ONNX): medir el primer embed en M2-06, no esperar a M2-09; si revienta el presupuesto, la salida es arquitectura del recall (FTS-first, cache), nunca relajar el gate.
2. **Drift de corpus**: la validez del gate depende de re-correr ambos arms el mismo día.
3. **Contaminación AGPL** en M2-07: prohibición explícita en el brief del executor.
4. **Pineo sqlite-vec**: `=x.y.z` exacto; un `^` accidental es riesgo silencioso.
5. **Ratio gates/reserva** >50% tras dos noches con reserva 8 ⇒ consolidar items, no inflar la reserva en caliente.

## 8. Testing

- Unit tests por módulo (parser frontmatter, extractor wikilinks, fusión) en el crate.
- Los oráculos por item (§3) son la verificación de integración: paridad de corpus, harness, golden envelope.
- El eval set de M0 (56 queries) queda como test de regresión permanente del engine (ya establecido en spec madre §4.1.5).
