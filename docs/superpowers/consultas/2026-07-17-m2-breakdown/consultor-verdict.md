# Veredicto — GATE-HUECO-M2: breakdown de M2 (E1 read)

- **Fecha**: 2026-07-17
- **Adjudicador**: consultor Fable fresco (régimen spec §8; sin participación previa en M2)
- **Régimen**: gates delegado — Paul delegó las decisiones de diseño del breakdown de M2 en sesión interactiva 2026-07-17 ("las preguntas que tengas las delegas a un consultor fable, yo firmo sus decisiones"). Brief con blindspot-pass haiku previo: `brief.md` (este directorio).
- **Verificación primaria**: lectura directa de spec §4.2-4.5/§6.2/§7/§8, `.superpowers/fabrica/config.md` completo, `evals/retrieval-fase0/{gate.md, verdict/m0-verdict.md, eval.jsonl}`, `harness/{replay.py, analyze.py, stratify.py, run-arm.sh}`, `results/metrics-jina-es.md`, y `~/.basic-memory/config.json` vigente (jina-es/768/0.35 confirmados en fichero).

## D1 — Layout y nombre del binario

**Adjudicación**: crate Rust único en `engine/` (con `src/lib.rs` + `src/main.rs`), binario `exo` con subcomandos; sin cargo workspace hasta que E3 lo exija.

**Racional**: un crate con lib+bin da el split de testabilidad que motivaría el workspace, sin su burocracia; convertirlo en workspace member cuando entre rmcp (E3) es un refactor mecánico de minutos, y pagar esa estructura hoy para un binario de miles de líneas es el over-engineering que el criterio veta. kbx (Go) convive sin fricción: cargo no ve fuera de `engine/` y la estructura `engine/plugins/templates` ya está fijada en spec §7-M1a. El nombre `exo`: el framework se llama exo (§10 decisión 2, firmado), `engine` es genérico y peligroso como nombre en PATH, y la mención `engine search --json` de §5.4 es ilustrativa per brief — los scripts de M6 se escribirán contra `exo`.

**Trade-off aceptado**: si E3 obliga a separar crates (deps/features de rmcp), se paga entonces una workspace-ificación que hoy habría salido casi gratis.

## D2 — Breakdown de M2: items ordenados + lane

**Adjudicación**: 9 items en este orden y con este routing:

| # | Item | Lane | Oráculo / gold |
|---|------|------|----------------|
| M2-01 | Scaffold: crate `engine/`, bin `exo`, rusqlite+FTS5 bundled compila, sqlite-vec **pineado con `=x.y.z`**, fastembed-rs descarga jina-es y embebe 1 frase a 768 dims | mecánica | `cargo build && cargo test` + smoke citados |
| M2-02 | Spec del indexer + gold de paridad de corpus + **envelope JSON fijado** (adopción del envelope versionado de kbx, spec §3) | diseño (fable en cabeza) | gold sellado antes de implementar |
| M2-03 | Indexer: walker + parser frontmatter + FTS5 + `rebuild` de primera clase (§4.3), incremental mtime/git | ejecución contra el gold de 02; el par 02+03 cierra con UN gate fable (lane diseño per config) | diff de paridad = ∅ + rebuild idempotente |
| M2-04 | Grafo de wikilinks: solo **indexado** de aristas (`to_id NULL` tolerado, §6.2); **el ranking por grafo queda explícitamente fuera de E1** | mecánica (paralelizable, filler) | extractor de referencia (`[[...]]` por script) + spot-check |
| M2-05 | Arm engine en el harness M0 (`replay-engine.py`, mismo schema jsonl; ver D3) | mecánica | `analyze.py <arm-engine>` produce metrics sin errores |
| M2-06 | Embeddings + tabla vectorial (jina-es/768, chunking propio, sqlite-vec) | mecánica | hit@5 vector vía harness; referencia bm-vector = 43/55 (`metrics-jina-es.md`) |
| M2-07 | Fusión + calibración de threshold — **clean-room desde la descripción de §4.2, jamás el código AGPL** | diseño (config ya lo fija) | gold = eval set M0; sweep con el procedimiento de `analyze.py` |
| M2-08 | `exo recall --json`: shape que necesitará el hook (definición operativa E1-b) + medición de latencia | mecánica | golden envelope + presupuestos de D4 (hyperfine) |
| M2-09 | Side-by-side final: re-captura de ambos arms el mismo día + gate D4 | corrida + gate fable fresco | `gate-m2` pre-registrado (D4) |

**Racional**: el orden compra primero el oráculo (02 antes de 03: spec-first, y el gold convierte todo lo posterior en iterable-hasta-verde), mete el harness (05) en cuanto existe `exo search --json` para que 06-08 se midan por comando, y deja fusión (07) tras tener ambas señales. El routing respeta lo ya fijado en config §Lanes (indexer y fusión = diseño, resto mecánica); el envelope va dentro del gate de 02 porque es superficie irreversible nombrada en spec §8 — una decisión, un gate, no dos. El recorte "grafo indexado sí, grafo en ranking no" es decisión explícita: la fórmula de fusión a copiar (§4.2) no incluye grafo y no hay oráculo para ranking-por-grafo — meterlo sería el scope-creep del riesgo #4 (§9).

**Trade-off aceptado**: 9 items = 9 merges potenciales con gate fable cada uno; se mitiga permitiendo agrupar merges adyacentes de lane mecánica (04+05, 06) en un solo gate si el ledger lo pide.

## D3 — Harness side-by-side

**Adjudicación**: reusar el harness M0 — script hermano `replay-engine.py` que invoca `exo search --json` y escribe el mismo schema jsonl; `analyze.py` intacto; oráculo de paridad de corpus = probe RO sqlite contra `memory.db` a nivel de entidad.

**Racional**: el config lo manda literalmente ("Reusar este harness como base del side-by-side de E1 … no reinventar el arnés", `config.md:146-148`) y `analyze.py` ya centraliza el único punto peligroso — la normalización de permalinks — en `norm()` (`harness/analyze.py:12-25`). `replay.py` no se generaliza: está acoplado al baile de `config.json` para el threshold del CLI de bm (`replay.py:29-37`) que el engine no necesita (threshold por flag) — un hermano de ~80 líneas con el mismo contrato de salida `{query, search_type, elapsed_s, results[{permalink,type,score}]}` es más barato que parametrizar un script validado. Paridad de corpus: script probe read-only (patrón `stratify.py:7`, URI `mode=ro`) que extrae el set de permalinks a nivel **entidad** del índice de bm y lo diffea contra el índice del engine — nunca contra el output del CLI (gotcha pre-registrado en `gate.md` §Ajustes: el CLI agrega las filas observation a su entidad; la medición va contra `search_index`). Paridad de **chunks NO exigida**: el chunking es detalle de implementación y el ground truth del eval es a nivel de nota (`eval.jsonl`); counts de chunks solo como dato informativo.

**Trade-off aceptado**: dos replays hermanos duplican algo de código de replay; se acepta a cambio de no tocar un harness sellado y byte-verificado en el verdict M0.

## D4 — Gate numérico de cierre de M2 (pre-registrar en `evals/e1-read/gate.md`, commiteado ANTES de la corrida final)

**Adjudicación**: el gate tiene tres patas, las tres obligatorias:

1. **Paridad de corpus (indexer)**: diff de sets de permalinks a nivel entidad engine-vs-índice-bm **= ∅**, cero tolerancia, con las exclusiones de §6.2 verificadas explícitamente (24 .md de dotdirs fuera, `archive/` dentro, 5 entidades no-markdown fuera, 0 permalinks regenerados).
2. **Retrieval pareado**: ambos arms re-corridos **el mismo día sobre el mismo estado de la KB** (commit git de kb-demo pineado en el verdict) — prohibido comparar contra los `results/` de julio (gotcha M0: comparar contra baseline pre-purge/pre-drift sobreestima). Gate: engine-hybrid **ROMPE ≤2 y ARREGLA ≥ las que ROMPE** respecto a bm-hybrid (pareada, patrón M0 — no proporciones; implica hit@5 engine ≥ bm, referencia hoy 43/55). Subgrupo observation-sensitive examinado por separado (patrón `gate.md` §Ajustes). Diagnóstico no-gate: si `cge bitácora` (el fusion-miss conocido, verdict M0 D3.3) queda arreglado, se anota como evidencia de fusión superior — informativo, no exigible.
3. **"Capaz de servir el recall, demostrado"**: (a) `exo recall --json` emite el envelope pactado en M2-02, validado contra golden file; (b) latencia pre-registrada: arranque+consulta FTS-only **p95 < 100 ms** (lo que un hook por-tool-call necesita, §4.5 "arranque en ms"); hybrid en frío, carga de modelo incluida, **p95 < 2.0 s** (el recall de SessionStart lo tolera; referencia bm hoy: mediana 4.4 s, verdict M0 decisión 3.1).

Ambigüedad o empate ⇒ consultor fable adjudica con este texto delante; los números no se re-negocian post-hoc (cláusula 4, patrón `gate.md`).

**Racional**: reproduce la estructura que ya funcionó en M0 (pareada + estrato + cláusula de adjudicación) y cubre las tres cosas que E1 promete (§4.4-E1): mismo corpus, retrieval no peor, e invocabilidad real desde hooks. El listón de latencia separa honestamente lo que debe ser ms (FTS/arranque) de lo que no puede serlo sin daemon (embed de query con modelo de 0.64 GB).

**Trade-off aceptado**: el listón es "paridad o mejor", no "mejor" — un engine que solo empata a 43/55 pasa; se acepta porque el deliverable de E1 es *capaz, demostrado*, y la mejora del FTS real (18/55 hoy) llegará como dato del side-by-side, no como requisito de cierre.

## D5 — Primera campaña de fábrica

**Adjudicación**: la primera campaña entra solo con **M2-01 (mecánica) + M2-02 (diseño, fable en cabeza)**, con M2-03 arrancando únicamente si 02 cierra su gate esa misma noche; y **la reserva fable sube de 6 a 8 dispatches/noche para esta campaña**, manteniendo intacto el cap semanal ≤20% y el contador separado gates-vs-spec.

**Racional**: replica el patrón ya rodado en config §Lanes (mecánica en paralelo con diseño secuencial); M2-02 es el item de máximo apalancamiento — produce el oráculo que convierte las noches 2-N en lane mecánica — y M2-01 des-riesga las tres dependencias nativas (FTS5 bundled, sqlite-vec pineado, fastembed-rs+jina) sin bloquear a nadie. La subida de reserva es exactamente la señal que el propio config pre-anuncia ("si el ledger muestra ratio gates/reserva alto, es señal para subir la reserva en la siguiente rama de config, NO para saltarse la regla", `config.md:176-179`): M1a gastó el 60% en gates, y esta noche lleva 2 gates (merge de 01, gate del par diseño) más redacción y review adversarial fable de 02 — con 6, la reserva estrangula el critical path.

**Trade-off aceptado**: la primera noche puede cerrar sin una línea del indexer mergeada (solo scaffold + spec/gold); comprar el oráculo primero es lo que abarata todo lo demás.

## D6 — Config del engine

**Adjudicación**: mientras dure el side-by-side, el engine lee la config viva de basic-memory (`~/.basic-memory/config.json`, read-only) para modelo/dims/threshold/ruta de la KB, con precedencia flags CLI > config; la config propia de exo entra como ítem de M5a (o antes si duele), no en el día 1.

**Racional**: la divergencia silenciosa entre arms es el fallo que invalidaría E1 entero — con una sola fuente de verdad no puede existir, y duplicar día 1 obligaría a construir un check de divergencia: maquinaria extra para vigilar un problema autoinfligido (YAGNI). Todas las claves necesarias existen hoy en ese fichero (verificado por lectura directa: `semantic_embedding_model`, `semantic_embedding_dimensions`, `semantic_min_similarity`, `projects.kb-demo.path`), leer un JSON al arranque cuesta sub-ms (compatible con hooks), y el veto AGPL (restricción 2) aplica a código, no a leer un fichero de config del usuario. La ventana de acoplamiento está acotada por diseño: side-by-side >4 semanas ⇒ se decide (restricción 5) — es un puente con caducidad, no una dependencia estructural.

**Trade-off aceptado**: en M5a se paga una migración de config one-shot que "config propia día 1" habría amortizado — trivial, contra semanas de riesgo de arms silenciosamente divergentes.

---

## Riesgos que el orquestador debe vigilar

- **Carga del modelo en frío**: 0.64 GB de ONNX puede comerse el presupuesto hybrid <2 s. Medirlo en M2-06 al primer embed, no esperar a M2-09; si revienta, la salida es una decisión de arquitectura del recall (FTS-first, cache de sesión), nunca relajar el gate ya registrado.
- **Drift de corpus**: la KB seguirá creciendo entre el sellado del eval (07-17) y la corrida final; los `expected_permalink` siguen válidos (permalinks jamás se regeneran) pero los rankings se mueven — la validez del gate depende de re-correr AMBOS arms el mismo día; cualquier comparación contra `results/` de julio invalida el verdict.
- **Contaminación AGPL en M2-07**: el brief del executor de fusión debe llevar la descripción de la fórmula copiada de spec §4.2 y la prohibición explícita de abrir el repo de basic-memory "para aclarar una duda" (restricción 2, veto también a vendorizar).
- **Pineo de sqlite-vec**: en M2-01, versión exacta `=x.y.z` en `Cargo.toml` y anotada en la spec del indexer; un `^` accidental es el riesgo #8 de §9 en silencio.
- **Ratio gates/reserva**: si con reserva 8 el ratio sigue >50% tras dos noches, la señal es consolidar items (agrupar merges mecánicos adyacentes, cf. trade-off D2), no volver a inflar la reserva en caliente.

## Preguntas que SÍ requieren a Paul

No hay: ninguna adjudicación toca línea roja (nada destructivo, nada externo — los `git push` a remotos siguen siendo de Paul per config §Ejecución de gates — y GATE-HUECO-M2 ya lo abrió Paul en sesión el 2026-07-17). Las dos decisiones más cercanas al borde quedan dentro del mandato: la subida de reserva 6→8 la delega el brief explícitamente y no toca el cap semanal, y la lectura RO de `~/.basic-memory/config.json` no modifica nada instalado.

---

*FIRMADO por Paul en sesión 2026-07-17 ("firmo, continua") — delegación declarada en la misma sesión: "yo firmo sus decisiones".*
