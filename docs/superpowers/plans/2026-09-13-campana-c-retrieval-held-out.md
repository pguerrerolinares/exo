# Campaña C — Retrieval fuera de muestra (H7, H7b, H14, H24) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: usa `exo:orchestrate`
> para ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`)
> para tracking.

**Goal:** medir, con un held-out pre-registrado antes de mirar resultados, si
el hybrid sellado generaliza fuera de las 55 queries con las que se eligió
(H7), si RRF lo iguala o supera (H7b) y si el solape o el late chunking
mejoran el troceado de 900 caracteres (H14), con `acceptable_permalinks`
desde el diseño (H24). Solo se toca producción si el gate pre-registrado pasa.

**Architecture:** un único gold privado sirve a las tres preguntas. El
harness Python reutiliza `replay-engine.py` y `analyze.py` de M0. Por cada
query captura las listas FTS, vector e hybrid del binario, y calcula offline
las fusiones sellada y RRF. Un oráculo de fidelidad (la fusión offline tiene
que reproducir exactamente la del binario) es la condición de validez. Las
variantes de troceado se construyen en ramas experimentales que nunca se
mergean, sobre índices en un directorio privado. El criterio de decisión vive
en `2026-09-13-campana-c-preregistro.md`, que se congela por commit antes de
medir.

**Tech Stack:**
- Python 3.12, solo stdlib (`unittest`, `json`, `math`, `random`, `shlex`,
  `hashlib`), igual que `evals/retrieval-fase0/harness/`.
- Rust 2024, crate `exo` en `engine/`, MSRV 1.95.
- fastembed 5.17.3 · tokenizers 0.22.2 (transitiva) · sqlite-vec =0.1.9.
- hyperfine · `git worktree`.

## Hallazgos: re-verificación (2026-09-13, map ≠ territorio)

Ningún hallazgo se descarta entero. Cuatro se corrigen o se acotan, y el
recon aportó uno nuevo (N1):

- **H7 se confirma y se acota.**
  - Confirmado: `engine/src/main.rs:12-25` documenta el sweep de 15+1
    corridas sobre las mismas 55 queries, y la fusión está en
    `engine/src/buscador.rs:350-414`.
  - **Acotado:** con `BONUS_SELLADO = 0.0` (`main.rs:24`) la fórmula queda en
    `max(v, 0.6·f/f_max)`, un CombMAX ponderado. Los hiperparámetros efectivos
    son β y el umbral, no tres.
  - La rejilla in-sample era **plana**: 47–49/55 en las 15 celdas
    (`evals/retrieval-fase0/results/metrics-engine-hybrid-b*-e*.md`), y los
    umbrales 0.35 y 0.40 empatan a 49/55. El optimismo por elegir bonus y β es
    de ≤2 queries. El riesgo real es la generalización del diseño completo.
  - **Corregido:** el umbral 0.40 no es un valor sellado en el binario. Lo
    pasa el hook (`plugins/exo/scripts/recall-inject.sh:145`), mientras la
    config dice 0.35 (`~/.exo/config.toml`; default de init en `main.rs:588`).
  - **Corregido:** el default `--type fts` que el backlog cita en `main.rs:205`
    está hoy en `main.rs:215`.
  - **Corregido:** `eval.jsonl` no está en el árbol de `main`. Solo existe en
    la rama local `archivo/main-pre-reescritura` (`f847ce1`).
- **H14 se confirma y se refuerza.**
  - `MAX_CHARS = 900` (`engine/src/trozos.rs:9`); `corta_duro`
    (`trozos.rs:100-106`) sin solape.
  - Recon sobre la KB actual con un script que replica `bloques_markdown`:
    **~50% de los ~3.290 trozos salen de un corte duro**.
  - **Corregido:** "modelo long-context" vale para el modelo (8192
    posiciones), pero el pipeline trunca a 512 tokens (`tokenizer_config.json`
    del snapshot pineado: `model_max_length: 512`;
    `fastembed-5.17.3/src/common.rs:97`). Hoy no muerde con trozos de ≤300
    tokens.
  - **Corregido:** en el backlog no hay un item Alta propio de "tamaño de
    trozo". Es una línea "Relacionado" dentro del item del 48/55
    (`docs/backlog.md:108-109`).
- **H24 se confirma.** `evals/retrieval-fase0/verdict/labels.md:72` (y `:33`,
  fila 13). No está en el backlog.
- **N1 (nuevo, recon):** `exo recall --query=<prompt>` funciona en la práctica
  como vector puro. `prepara_query` (`buscador.rs:147-153`) une con AND de
  FTS5, y una frase natural no encuentra nada por FTS. Reproducido en solo
  lectura: `exo search --db ~/.exo/index.db --type fts --limit 50 --json
  "fabrica campaña"` → 29 resultados; con `"cómo decidimos el umbral de
  similitud del recall y por qué quedó en 0.40"` → 0. La fusión solo actúa
  sobre queries de palabras clave, y el hook depende del arm vector y del
  troceado. Se usa en el diseño de estratos. No es un item de esta campaña: va
  al backlog en la Task 11.

## Viabilidad de late chunking (recon obligatorio)

**VIABLE con la API pública de fastembed 5.17.3, sin `ort` directo.**
Evidencia en `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`:

- `fastembed-5.17.3/src/text_embedding/impl.rs:322`: `pub fn transform(...) ->
  Result<EmbeddingOutput>`. Tokeniza y ejecuta la sesión sin pooling. El
  `embed()` que usa hoy `engine/src/lib.rs:266` es `transform` más pooling y
  normalización.
- `src/output/embedding_output.rs:12-15`: `pub struct SingleBatchOutput { pub
  outputs, pub attention_mask_array }`. `:22` `select_output` devuelve la vista
  `[batch, seq, 768]`; `:95` `into_raw()`.
- `src/text_embedding/init.rs:142`: `pub tokenizer: Tokenizer`.
  - `tokenizers-0.22.2/src/tokenizer/encoding.rs:159` `get_offsets()` da
    offsets en bytes, `:175` `get_overflowing()`.
  - `tokenizer/mod.rs:642` `get_truncation_mut()` abre la ventana por encima
    de 512 sin tocar la config ni añadir dependencias directas.
- El ONNX cacheado (`~/.cache/huggingface/hub/models--jinaai--jina-embeddings-v2-base-es/snapshots/8e2d780d.../onnx/model.onnx`,
  641 MB) expone la salida `last_hidden_state` y no lleva pooling horneado.
  Entradas: `input_ids` y `attention_mask`.
- **Límites que condicionan el diseño** (estimaciones, no medidas):
  - A 8192 tokens, una matriz de atención float32 con 12 cabezas ocupa
    ~3,2 GB, en una máquina de 15 GB. De ahí la ventana de **2048 tokens**.
  - 9 notas superan ~8192 tokens (unos 32 KB).
  - La caché del indexer va por texto de trozo (`engine/src/indexer.rs:318-347`)
    y es incorrecta bajo late chunking. Por eso **late chunking nunca pasa a
    producción en esta campaña**: si gana, se abre una spec aparte.

## Global Constraints

- **El crate vive en `engine/`, no en la raíz.** No hay workspace de Cargo:
  todo `cargo` va con cwd `engine/` o con `--manifest-path engine/Cargo.toml`.
- **MSRV declarada: `rust-version = "1.95"`, `edition = "2024"`**
  (`engine/Cargo.toml:4,8`). El job `msrv` de CI hace `cargo check
  --all-targets --locked`. El job `lint` hace `cargo fmt --check` y `cargo
  clippy --all-targets --locked -- -D warnings`
  (`.github/workflows/ci.yml:38,46,76`).
- **`engine/scripts/test-hermetico.sh` NO se modifica.** Corre `cargo test
  --release --no-fail-fast` con `EXO_CONFIG` apuntando a un fichero
  inexistente. Ningún test nuevo puede depender de `~/.exo/config.toml`. Su
  salida verde dice literal: "NO cubre la caché del modelo ONNX (~0,6 GB), que
  las suites de indexado siguen exigiendo".
- **Modelo pineado:** `jinaai/jina-embeddings-v2-base-es`, revisión
  `8e2d780d8fd38f81ca9123ee28e4c5a968aaf21e` (`engine/src/lib.rs:167-168`),
  pooling `Mean` explícito (`lib.rs:255`). La caché la resuelve `hf_hub`:
  `$HF_HOME/hub` si existe, si no `~/.cache/huggingface/hub`.
- **Envelope v2, verbatim de `engine/src/envelope.rs`:**
  `{"schema_version":2,"command":<command>,"data":<data>}`, una línea a
  stdout. `data` de `search` = `{query, search_type, elapsed_s, results[],
  warnings?}` (`buscador.rs:36-51`). `score` es "escala informativa, no
  contractual" (`buscador.rs:19`). Ningún consumidor gatea por `score`
  (verificado con `rg score plugins/exo/scripts`: solo fixtures de test; el
  dup-gate de `write new` usa Jaccard de slugs, `escritor.rs:160-175`).
- **Harness de eval: Python stdlib**, reutilizando `evals/retrieval-fase0/harness/`
  (importar `norm` de `analyze.py`, cargar `search()` de `replay-engine.py`).
  **No se modifica ningún fichero existente de `evals/`**. Los scripts nuevos
  van en `evals/retrieval-heldout/harness/`.
- **`evals/retrieval-fase0/gate.md` es inmutable.** Tampoco se tocan
  `verdict/labels.md`, `verdict/m0-verdict.md` ni `evals/e1-read/`.
- **Repo público (desde 2026-09-02).** Ningún texto de query, prompt ni
  permalink por fila entra en git. Datos privados en
  `PRIV=~/.local/share/exo-evals/c-heldout` (`chmod 700`). Al repo, solo
  harness, agregados y sha256.
- **Líneas rojas:** no se toca `~/.exo/index.db` (solo `exo search` de
  lectura); no se escribe en la KB `~/Documentos/proyectos/wisdom-paul` (el
  snapshot es un `git clone` a `$PRIV`); nada de push, merge ni commit a
  `main` (el merge es el gate de Paul); nada externo; `git add` con rutas
  explícitas, nunca `-A`; `git -C <path>`, nunca `cd <path> && git`.
- **Pre-registro primero.** Ninguna tarea computa hit@k de un brazo sobre el
  held-out antes del commit de congelación (Task 5). Tras congelar no se afina
  nada sobre el held-out, no se añaden brazos y no se re-etiqueta.
- **Orden respecto a la campaña A:** diseño, harness y etiquetado (Tasks 0–5)
  pueden empezar ya. La medición (Tasks 6–10) y cualquier cambio de producción
  (Task 12) van **después del merge de A** (§Dependencias).
- **Régimen de fábrica:** `.superpowers/fabrica/config.md` §ACTUALIZACIÓN
  2026-08-17 dice literal "**Sin métricas nuevas, sin pre-registros nuevos, sin
  ventanas de observación.**". Esta campaña lo contradice y **no arranca sin
  la línea `OVERRIDE` de Paul** (D0). El config está desactualizado y este
  plan no lo edita.

## Decisiones abiertas (PENDIENTE-PAUL)

Se fijan en la Task 0 (D0–D3) y en la Task 5 (D4–D5), antes de congelar. Cada
una lleva una recomendación; la decisión es de Paul.

**D0 — Levantar el régimen de cierre para C.**
- (a) OVERRIDE solo para la campaña C. Pro: el item Alta del backlog
  (revisión 2026-09-04, acción (a)) pide exactamente esto. Contra: reabre la
  puerta a "métricas nuevas".
- (b) No. C no se ejecuta y el 48/55 sigue sin evidencia fuera de muestra.
- *Recomendación: (a), acotado a C.*

**D1 — Fuente y proporción del held-out.**
- (a) Solo `prompt`, los prompts reales que dispararon el hook. Pro: es la
  distribución de producción. Contra: por N1 no ejercita la fusión (H7b sin
  potencia), hay muchas nulas y etiquetar exige más juicio.
- (b) Solo `agent-search`, queries de agentes posteriores al sellado. Pro:
  misma distribución que las 55 y la que ejercita la fusión. Contra: pool
  bruto de ~60 antes de filtros.
- (c) Solo `hard`, paráfrasis sintéticas sobre notas añadidas tras el
  2026-08-17. Pro: fuga temporal imposible. Contra: son sintéticas y cargan el
  sesgo del autor.
- (d) Queries escritas por Paul. Pro: el gold más fiel. Contra: su tiempo, y
  conoce el sistema.
- (e) Mezcla estratificada fija a priori.
- *Recomendación: (e) con `prompt` 50% · `agent-search` 30% · `hard` 20%,
  reportando cada estrato aparte. Si `agent-search` no llega a su cuota tras
  filtrar, se completa con `hard` redactado en estilo palabras clave, sin
  inventar prompts.*

**D2 — Tamaño (filas no nulas).**
- 60: Paul revisa en ~40 min; potencia ~0,38 ante +10 pp.
- 100: ~1–1,5 h; potencia ~0,66 ante +10 pp; con la regla GANA, P(adoptar
  +8 pp) = 0,93.
- 150: ~2 h; 0,85; probablemente no cabe en el pool.
- Con cualquier N factible, una diferencia de 5 pp es invisible (tabla del
  pre-registro §7).
- *Recomendación: 100, con 60 como mínimo si el pool no da.*

**D3 — ¿Brazo de late chunking?**
- (a) No, solo solape. Pro: cero código nuevo en `lib.rs`. Contra: H14 queda
  a medias y el solape no es late chunking.
- (b) Sí, como rama experimental con ventana de 2048 tokens. Coste: ~120
  líneas en una rama que no se mergea más un rebuild estimado de 2–4× el de
  `base`. El de `base` se estima en ~14 min, extrapolando "~0,25 s por trozo"
  (`indexer.rs:313`) a ~3.290 trozos; es una estimación y la Task 6 lo mide.
  Kill si hay OOM o supera 3× `base`.
- (c) Aplazarlo. Contra: gastaría el held-out en una segunda comparación
  pre-registrada más tarde.
- *Recomendación: (b). El ~50% de trozos con corte duro y la ganancia que el
  paper asocia a documentos largos lo justifican como experimento acotado, y
  la regla del pre-registro impide que llegue a producción sin spec propia.*

**D4 — Criterio de adopción.**
- (α) Adopta si no pierde (NETO ≥ 0). Pro: simplicidad (RRF quita β).
  Contra: churn de código sin ganancia medida; P(adoptar un igual) alta.
- (β) NETO ≥ 3 y ARREGLA ≥ 2·ROMPE, con veto de MRR. Pro: P(adoptar peor)
  ≈ 0 y P(adoptar +8 pp) 0,81–0,93. Contra: 7–16% de adoptar un igual.
- (γ) McNemar exacto p < 0,05. Pro: la lectura clásica. Contra: con N ≤ 100
  exige algo como 9/1 y casi nunca pasa; la campaña cerraría "sin cambios" por
  diseño.
- *Recomendación: (β).*

**D5 — Relevancia para decidir (H24) y fila 13.**
- (a) Lenient `{expected} ∪ acceptable` decide, strict como descriptivo.
- (b) Strict decide.
- Por separado: overlay de la fila 13 en el reporte in-sample, sí o no.
- *Recomendación: (a), y overlay sí reportando ambas versiones. M0 no se
  toca: su gate y sus números quedan como históricos.*

**D6 — Default de `exo search --type` (backlog item Alta, acción (c)).**
Fuera de producción en C: el verdict aporta A0 frente a A2 en held-out y la
decisión se toma después, porque toca la help de `main.rs:213-216`, que es
territorio de B. *Recomendación: la decide Paul con el verdict delante.*

## Dependencias y conflictos con A y B

| Tarea C | Ficheros | Conflicto | Regla |
|---|---|---|---|
| 0–5 (diseño, harness, gold) | `evals/retrieval-heldout/**` nuevo, este plan y el pre-registro | ninguno | pueden correr ya, en paralelo con A y B |
| 6 (binario y fidelidad) | ninguno de código | **A**: según el brief toca `buscador.rs`/`recall.rs` (aperturas de DB, N+1), el schema y `resuelve_destinos`. Su plan (`2026-09-13-campana-a-recall-por-prompt.md:276-278`, en borrador en paralelo) dice que no toca `buscador.rs`, pero **cambia `vectores::knn`** (Task 3) y `recall.rs` (H2). Las dos cosas alteran la lista vector | **después del merge de A**. El oráculo de fidelidad garantiza la coherencia offline/binario, pero no la comparabilidad con los números pre-A: por eso el in-sample se re-corre con el binario post-A (Task 6) |
| 7 (rama `exp/c-solape`) | `engine/src/trozos.rs` | A no toca `trozos.rs`, pero su bench usa `trozos::trocea` para generar el corpus sintético | worktree desde `main` post-A |
| 8 (rama `exp/c-late`) | `engine/src/lib.rs`, `engine/src/indexer.rs`, `engine/tests/late_chunking.rs` | **A** toca `indexer.rs` (`resuelve_destinos`, upsert) | worktree desde `main` post-A; no se mergea nunca |
| 11 (backlog) | `docs/backlog.md` (item `:99-135`) | **B** reescribe mucho `docs/backlog.md` (H13) y A también lo toca | editar solo el bloque del item y añadir items; rebase sobre A y B si ya están mergeadas |
| 6, 9, 12 (captura y fusión) | `engine/src/buscador.rs` | **Hotfix PR #12 (mergeado 2026-09-13, `ea5b56f`)**: `busca_vector_con_embedding` fija el `k` del KNN por la consulta (arranca en `max(64, limite·8)`, ×4 hasta ≥`limite` notas distintas sobre el umbral). Es **exacto solo con la fusión sellada** (`bonus == 0`, `score = max(v,f)`): `busca_hybrid` pasa `limite` al arm vector únicamente en ese caso y cae al exhaustivo si `bonus != 0`. Con **RRF** la garantía no vale (una nota fuera del top-L vectorial puede entrar por su rango FTS, y RRF necesita el rango vectorial de los candidatos FTS) | (a) la captura offline de la lista vector para simular fusiones debe pedir la lista completa o un `--limit` holgado, nunca heredar el `limite` del hook; (b) si 12A adopta RRF, **reescribir la condición de parada** (p.ej. calcular v de los ≤50 candidatos FTS por `permalink IN (…)`) y extender el test `equivalencia_exacta_contra_exhaustiva` a RRF antes de mergear |
| 12 (producción) | `engine/src/buscador.rs`, `engine/src/main.rs:12-25,203-238,850-902`, `engine/src/recall.rs:474-497`, `engine/tests/buscador.rs`, `engine/src/trozos.rs` | **A**: exige que `Busqueda.avisos` y `.elapsed_s` sobrevivan a la reescritura de `busca_hybrid` (se conservan en 12A), y que un cambio de `trozos.rs` entre en `main` **después** de su corrida «después» (A Task 14). **B**: su Task 3 reescribe la help de `--bonus`/`--fts-scale` (`main.rs:222-232`) y su Task 11 mueve `reports/` (cita en `main.rs:14`), así que 12A borra texto que B acaba de reescribir; también `docs/arquitectura.md:222-223,289` y el README | después de A (incluida su Task 14) **y** de B. C no edita README ni `arquitectura.md`: deja la nota de hand-off a B en el review-package |

## Estructura de ficheros

- Create: `evals/retrieval-heldout/harness/metricas.py`: métricas, fusiones
  offline, fidelidad, pareadas, reglas y CLI `fidelidad|informe|compara`.
- Create: `evals/retrieval-heldout/harness/captura.py`: captura FTS, vector e
  hybrid por query, reutilizando `search()` de `replay-engine.py`.
- Create: `evals/retrieval-heldout/harness/pool.py`: pool de candidatas, filtros
  anti-fuga y muestreo con semilla.
- Create: `evals/retrieval-heldout/harness/valida_gold.py`: validador del gold
  contra el snapshot.
- Create: `evals/retrieval-heldout/harness/test_harness.py`: `unittest` de todo
  lo anterior.
- Create: `evals/retrieval-heldout/verdict/gold-verificacion-resumen.md`,
  `agregados.md`, `agregados-in-sample.md`, `latencia.md` y `c-verdict.md`.
- Modify: `docs/superpowers/plans/2026-09-13-campana-c-preregistro.md`, solo
  §9 y §10 en la Task 5, y después inmutable.
- Ramas experimentales (sin merge): `exp/c-solape` en `.worktrees/c-solape`,
  `exp/c-late` en `.worktrees/c-late`.
- Rama de campaña: `c-retrieval-heldout`. Rama de producción (condicional):
  `c-produccion`.

---

### Task 0: Decisiones de diseño de Paul (PENDIENTE-PAUL, gate humano)

**Lane:** — (gate humano; bloquea Tasks 2–12). La Task 1 puede avanzar at-risk.
**Oráculo:** `rg -n '^OVERRIDE .*campana-c' .superpowers/fabrica/ledger.md && rg -n '^D[0-3] ' .superpowers/fabrica/pendiente-paul.md`

**Files:**
- Modify (gitignored, artefactos de fábrica): `.superpowers/fabrica/pendiente-paul.md`, `.superpowers/fabrica/ledger.md`

**Interfaces:**
- Consumes: §Decisiones abiertas de este plan.
- Produces: líneas `D0 <opción>` … `D3 <opción>` en `pendiente-paul.md` y la línea `OVERRIDE` en el ledger, que la Task 2 lee (cuotas de D1/D2) y la Task 8 (D3).

- [ ] **Step 1: Encolar las decisiones D0–D3**

Añadir a `.superpowers/fabrica/pendiente-paul.md`, copiando literal las opciones y recomendaciones de §Decisiones abiertas (D0, D1, D2, D3), con "bloquea: campaña C Tasks 2–12" y la fecha de `date -Iseconds`.

- [ ] **Step 2: Esperar el veredicto de Paul**

No se despacha ninguna tarea 2–12 hasta que existan:
```
OVERRIDE <ts> Paul: régimen de cierre levantado para campaña C (D0)
D0 <a|b>
D1 <opción; si (e), proporción exacta prompt/agent-search/hard>
D2 <N no nulas>
D3 <a|b|c>
```
Si D0 = (b): la campaña cierra aquí; Task 11 anota en el backlog "C no autorizada <fecha>".

- [ ] **Step 3: Verificar**

Run: el oráculo de la tarea.
Expected: dos coincidencias no vacías (exit 0).

---

### Task 1: Harness de métricas (`metricas.py`) con tests

**Lane:** mecánica (el oráculo lo crea la propia tarea con TDD sobre fixtures sintéticos; no depende de datos privados).
**Oráculo:** `python3 -m unittest -v evals/retrieval-heldout/harness/test_harness.py`

**Files:**
- Create: `evals/retrieval-heldout/harness/metricas.py`
- Create: `evals/retrieval-heldout/harness/test_harness.py`

**Interfaces:**
- Consumes: `norm(permalink: str|None) -> str|None` de `evals/retrieval-fase0/harness/analyze.py:12-26` (importado sin modificar).
- Produces (usado por Tasks 2, 6, 9, 12):
  - `carga_gold(ruta: str) -> list[dict]` (asigna `id="mNN"` y `acceptable_permalinks=[]` si faltan)
  - `aplica_overlay(filas: list[dict], ruta_overlay: str) -> list[dict]`
  - `carga_captura(ruta: str) -> dict[str, dict|None]`
  - `relevantes(fila: dict, estricto: bool=False) -> set[str]`
  - `hit(ranking: list[str], rel: set[str], k: int=5) -> bool` · `rr(ranking, rel, k=10) -> float`
  - `fusion_sellada(fts, vector, bonus, beta, umbral, limite) -> list[tuple[str,float]]`
  - `rrf(fts, vector, umbral, limite, k=60) -> list[tuple[str,float]]`
  - `rankings(captura: dict, brazo: str) -> dict[str, list[str]]`, brazo ∈ `sellado|sellado-035|rrf|vector|fts`
  - `fidelidad(captura: dict, brazo: str="sellado") -> list[str]` (ids discrepantes)
  - `mcnemar_exacto(b: int, c: int) -> float` · `wilson(x: int, n: int) -> tuple[float,float]` · `bootstrap_ic95(deltas: list[float], n=10000, semilla=20260913) -> tuple[float,float]`
  - `pareada(cand: dict, ref: dict, gold: list[dict], estricto: bool) -> dict` con claves `arregla, rompe, p_mcnemar, ic95_delta_mrr`
  - `decide(par: dict, neto_min: int, factor: float) -> str` (`"GANA"|"NO GANA"`) · `decide_r1(par_vector, par_fts, neto_min, factor) -> str`
  - CLI: `metricas.py fidelidad --captura F [--brazo sellado|rrf]` · `metricas.py informe --gold G [--overlay O] --captura base=F [--captura solape=F] [--captura late=F] --neto-min N --factor X [--estricto] [--p95 indice=S] [--rebuild-s indice=S] --detalle D` · `metricas.py compara --a F1 --b F2`

- [ ] **Step 1: Escribir los tests que fallan**

`evals/retrieval-heldout/harness/test_harness.py`:
```python
#!/usr/bin/env python3
"""Tests del harness de la campaña C. Uso:
python3 -m unittest -v evals/retrieval-heldout/harness/test_harness.py"""
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import metricas as m  # noqa: E402


class TestFusionSellada(unittest.TestCase):
    def test_formula_ambos_canales(self):
        r = m.fusion_sellada([("a", 10.0)], [("a", 0.6)], 0.25, 0.4, 0.0, 10)
        self.assertEqual([p for p, _ in r], ["a"])
        self.assertAlmostEqual(r[0][1], 0.6 + 0.25 * 0.4, delta=1e-12)

    def test_umbral_filtra_vector_antes_de_fusionar(self):
        self.assertEqual(m.fusion_sellada([], [("a", 0.39)], 0.0, 0.6, 0.40, 10), [])

    def test_fmax_cero_descarta_fts(self):
        self.assertEqual(m.fusion_sellada([("a", 0.0), ("b", 0.0)], [], 0.0, 0.6, 0.4, 10), [])

    def test_desempate_por_permalink_y_truncado(self):
        r = m.fusion_sellada([], [("e", 0.5), ("c", 0.5), ("a", 0.5)], 0.0, 0.6, 0.4, 2)
        self.assertEqual([p for p, _ in r], ["a", "c"])

    def test_bonus_cero_es_max_con_beta(self):
        r = m.fusion_sellada([("a", 5.0), ("b", 10.0)], [("a", 0.5)], 0.0, 0.6, 0.4, 10)
        self.assertEqual([p for p, _ in r], ["b", "a"])
        self.assertAlmostEqual(r[0][1], 0.6, delta=1e-12)
        self.assertAlmostEqual(r[1][1], 0.5, delta=1e-12)


class TestRRF(unittest.TestCase):
    def test_suma_inversos_de_rango_k60(self):
        r = m.rrf([("a", 9.0), ("b", 5.0)], [("b", 0.9), ("c", 0.5)], 0.4, 10)
        self.assertEqual([p for p, _ in r], ["b", "a", "c"])
        self.assertEqual(r[0][1], 1.0 / 62 + 1.0 / 61)
        self.assertEqual(r[1][1], 1.0 / 61)

    def test_umbral_filtra_la_lista_vector(self):
        r = m.rrf([], [("c", 0.3)], 0.4, 10)
        self.assertEqual(r, [])

    def test_una_sola_lista_conserva_orden(self):
        r = m.rrf([], [("x", 0.9), ("y", 0.8)], 0.4, 10)
        self.assertEqual([p for p, _ in r], ["x", "y"])


class TestMetricas(unittest.TestCase):
    def fila(self):
        return {"id": "c1", "expected_permalink": "kb/a", "acceptable_permalinks": ["kb/b"]}

    def test_relevantes_lenient_y_strict(self):
        self.assertEqual(m.relevantes(self.fila()), {"kb/a", "kb/b"})
        self.assertEqual(m.relevantes(self.fila(), estricto=True), {"kb/a"})
        self.assertEqual(m.relevantes({"expected_permalink": None}), set())

    def test_hit_y_rr(self):
        rel = {"kb/b"}
        ranking = ["x", "y", "kb/b"]
        self.assertTrue(m.hit(ranking, rel, k=5))
        self.assertFalse(m.hit(ranking, rel, k=2))
        self.assertAlmostEqual(m.rr(ranking, rel), 1 / 3)
        self.assertEqual(m.rr(["x"], rel), 0.0)

    def test_mcnemar_exacto(self):
        self.assertEqual(m.mcnemar_exacto(5, 1), 0.21875)
        self.assertEqual(m.mcnemar_exacto(9, 1), 0.021484375)
        self.assertEqual(m.mcnemar_exacto(3, 0), 0.25)
        self.assertEqual(m.mcnemar_exacto(0, 0), 1.0)

    def test_wilson(self):
        lo, hi = m.wilson(48, 55)
        self.assertAlmostEqual(lo, 0.760, delta=0.001)
        self.assertAlmostEqual(hi, 0.937, delta=0.001)

    def test_bootstrap_determinista(self):
        d = [0.0, 0.5, -0.25, 1.0, 0.0]
        self.assertEqual(m.bootstrap_ic95(d, n=500), m.bootstrap_ic95(d, n=500))
        self.assertEqual(m.bootstrap_ic95([0.0] * 10, n=200), (0.0, 0.0))

    def test_decide(self):
        base = {"arregla": 4, "rompe": 1, "ic95_delta_mrr": (-0.01, 0.05)}
        self.assertEqual(m.decide(base, 3, 2), "GANA")
        self.assertEqual(m.decide({**base, "ic95_delta_mrr": (-0.1, -0.01)}, 3, 2), "NO GANA")
        self.assertEqual(m.decide({**base, "arregla": 3}, 3, 2), "NO GANA")
        self.assertEqual(m.decide({"arregla": 7, "rompe": 4, "ic95_delta_mrr": (0, 1)}, 3, 2), "NO GANA")

    def test_decide_r1(self):
        ci = (-0.1, 0.1)
        pierde_vs_sellado = {"arregla": 1, "rompe": 3, "ic95_delta_mrr": ci}
        gana_vs_sellado = {"arregla": 6, "rompe": 1, "ic95_delta_mrr": (0.0, 0.2)}
        leve = {"arregla": 2, "rompe": 1, "ic95_delta_mrr": ci}
        self.assertEqual(m.decide_r1(pierde_vs_sellado, pierde_vs_sellado, 3, 2), "GENERALIZA")
        self.assertEqual(m.decide_r1(gana_vs_sellado, pierde_vs_sellado, 3, 2), "NO GENERALIZA")
        self.assertEqual(m.decide_r1(leve, pierde_vs_sellado, 3, 2), "INDETERMINADO")


class TestCargaYFidelidad(unittest.TestCase):
    def test_carga_gold_in_sample_asigna_ids(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "g.jsonl"
            p.write_text('{"query":"q1","expected_permalink":"kb/a"}\n{"query":"q2","expected_permalink":null}\n', encoding="utf-8")
            filas = m.carga_gold(str(p))
        self.assertEqual([f["id"] for f in filas], ["m01", "m02"])
        self.assertEqual(filas[0]["acceptable_permalinks"], [])

    def test_overlay(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "o.jsonl"
            p.write_text('{"query":"q1","acceptable_permalinks":["kb/b"]}\n', encoding="utf-8")
            filas = m.aplica_overlay([{"id": "m01", "query": "q1", "acceptable_permalinks": []}], str(p))
        self.assertEqual(filas[0]["acceptable_permalinks"], ["kb/b"])

    def test_fidelidad_detecta_discrepancia(self):
        buena = {"fts": [("a", 10.0)], "vector": [("b", 0.7)], "hybrid": [("b", 0.7), ("a", 0.6)]}
        mala = {"fts": [("a", 10.0)], "vector": [("b", 0.7)], "hybrid": [("a", 0.6), ("b", 0.7)]}
        self.assertEqual(m.fidelidad({"c1": buena, "c2": mala, "c3": None}), ["c2", "c3"])

    def test_carga_captura_con_error_es_none(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "c.jsonl"
            p.write_text(json.dumps({"id": "c1", "errores": ["timeout"]}) + "\n"
                         + json.dumps({"id": "c2", "errores": [], "fts": [["a", 1.0]], "vector": [], "hybrid": [["a", 0.6]]}) + "\n",
                         encoding="utf-8")
            cap = m.carga_captura(str(p))
        self.assertIsNone(cap["c1"])
        self.assertEqual(cap["c2"]["fts"], [("a", 1.0)])
        self.assertEqual(m.rankings(cap, "fts"), {"c1": [], "c2": ["a"]})


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Correr y verlo fallar**

Run: `python3 -m unittest -v evals/retrieval-heldout/harness/test_harness.py`
Expected: ERROR `ModuleNotFoundError: No module named 'metricas'`.

- [ ] **Step 3: Implementación**

`evals/retrieval-heldout/harness/metricas.py`:
```python
#!/usr/bin/env python3
"""Métricas del held-out de la campaña C (pre-registro
docs/superpowers/plans/2026-09-13-campana-c-preregistro.md §4-§6).

Réplica offline, EXACTA, de `fusiona`/`normaliza_fts` de
engine/src/buscador.rs (fusión sellada) y RRF (Cormack, Clarke & Büttcher,
SIGIR 2009, k=60) sobre las listas capturadas por captura.py. La fidelidad
(fusión offline == hybrid del binario) es condición de validez (§4).

Nunca emite texto de query ni permalinks por fila en los agregados: el repo
es público. El detalle por query va a --detalle (directorio privado).
"""
import argparse
import json
import math
import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "retrieval-fase0" / "harness"))
from analyze import norm  # noqa: E402

K_HIT = 5
K_RR = 10
RRF_K = 60
UMBRAL_SELLADO = 0.40
BETA_SELLADA = 0.6
BONUS_SELLADO = 0.0
SEMILLA = 20260913


def carga_gold(ruta):
    filas = []
    for n, linea in enumerate(Path(ruta).read_text(encoding="utf-8").splitlines(), start=1):
        if not linea.strip():
            continue
        f = json.loads(linea)
        f.setdefault("id", f"m{n:02d}")
        f.setdefault("acceptable_permalinks", [])
        filas.append(f)
    return filas


def aplica_overlay(filas, ruta_overlay):
    por_query = {}
    for linea in Path(ruta_overlay).read_text(encoding="utf-8").splitlines():
        if linea.strip():
            o = json.loads(linea)
            por_query[o["query"]] = o["acceptable_permalinks"]
    for f in filas:
        if f["query"] in por_query:
            f["acceptable_permalinks"] = por_query[f["query"]]
    return filas


def carga_captura(ruta):
    out = {}
    for linea in Path(ruta).read_text(encoding="utf-8").splitlines():
        if not linea.strip():
            continue
        c = json.loads(linea)
        if c.get("errores"):
            out[c["id"]] = None
            continue
        out[c["id"]] = {k: [(norm(p), s) for p, s in c[k]] for k in ("fts", "vector", "hybrid")}
    return out


def relevantes(fila, estricto=False):
    exp = norm(fila.get("expected_permalink"))
    if exp is None:
        return set()
    if estricto:
        return {exp}
    return {exp} | {norm(p) for p in fila.get("acceptable_permalinks", [])}


def hit(ranking, rel, k=K_HIT):
    return any(p in rel for p in ranking[:k])


def rr(ranking, rel, k=K_RR):
    for i, p in enumerate(ranking[:k], start=1):
        if p in rel:
            return 1.0 / i
    return 0.0


def _ordena(pares, limite):
    return sorted(pares, key=lambda x: (-x[1], x[0]))[:limite]


def fusion_sellada(fts, vector, bonus, beta, umbral, limite):
    """buscador.rs: normaliza_fts (f_max con fold desde 0.0; f_max == 0 ⇒
    canal FTS descartado) + fusiona (max + bonus·min, canal ausente = 0)."""
    f_max = max([0.0] + [s for _, s in fts])
    f = {} if f_max == 0.0 else {p: beta * s / f_max for p, s in fts}
    v = {p: s for p, s in vector if s >= umbral}
    pares = []
    for p in set(f) | set(v):
        vv, ff = v.get(p, 0.0), f.get(p, 0.0)
        pares.append((p, max(vv, ff) + bonus * min(vv, ff)))
    return _ordena(pares, limite)


def rrf(fts, vector, umbral, limite, k=RRF_K):
    """RRF: Σ 1/(k + rango), rango 1-based; FTS se suma antes que vector
    (mismo orden de suma que la implementación Rust de la Task 12)."""
    puntos = {}
    for lista in ([p for p, _ in fts], [p for p, s in vector if s >= umbral]):
        for rango, p in enumerate(lista, start=1):
            puntos[p] = puntos.get(p, 0.0) + 1.0 / (k + rango)
    return _ordena(list(puntos.items()), limite)


def rankings(captura, brazo):
    out = {}
    for i, c in captura.items():
        if c is None:
            out[i] = []
            continue
        if brazo == "sellado":
            pares = fusion_sellada(c["fts"], c["vector"], BONUS_SELLADO, BETA_SELLADA, UMBRAL_SELLADO, K_RR)
        elif brazo == "sellado-035":
            pares = fusion_sellada(c["fts"], c["vector"], BONUS_SELLADO, BETA_SELLADA, 0.35, K_RR)
        elif brazo == "rrf":
            pares = rrf(c["fts"], c["vector"], UMBRAL_SELLADO, K_RR)
        elif brazo == "vector":
            pares = _ordena([(p, s) for p, s in c["vector"] if s >= UMBRAL_SELLADO], K_RR)
        elif brazo == "fts":
            pares = c["fts"][:K_RR]
        else:
            raise ValueError(f"brazo desconocido: {brazo}")
        out[i] = [p for p, _ in pares]
    return out


def fidelidad(captura, brazo="sellado"):
    malas = []
    for i, c in captura.items():
        if c is None:
            malas.append(i)
            continue
        if brazo == "sellado":
            offline = fusion_sellada(c["fts"], c["vector"], BONUS_SELLADO, BETA_SELLADA, UMBRAL_SELLADO, K_RR)
        else:
            offline = rrf(c["fts"], c["vector"], UMBRAL_SELLADO, K_RR)
        motor = c["hybrid"]
        if [p for p, _ in offline] != [p for p, _ in motor] or any(
            abs(a[1] - b[1]) > 1e-12 for a, b in zip(offline, motor)
        ):
            malas.append(i)
    return sorted(malas)


def mcnemar_exacto(b, c):
    n = b + c
    if n == 0:
        return 1.0
    cola = sum(math.comb(n, i) for i in range(min(b, c) + 1)) / 2**n
    return min(1.0, 2 * cola)


def wilson(x, n, z=1.959964):
    if n == 0:
        return (0.0, 1.0)
    p = x / n
    d = 1 + z * z / n
    centro = (p + z * z / (2 * n)) / d
    medio = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / d
    return (centro - medio, centro + medio)


def bootstrap_ic95(deltas, n=10000, semilla=SEMILLA):
    if not deltas:
        return (0.0, 0.0)
    rng = random.Random(semilla)
    m = len(deltas)
    medias = sorted(sum(rng.choice(deltas) for _ in range(m)) / m for _ in range(n))
    return (medias[int(0.025 * n)], medias[int(0.975 * n) - 1])


def pareada(cand, ref, gold, estricto=False):
    arregla = rompe = 0
    deltas = []
    for fila in gold:
        rel = relevantes(fila, estricto)
        i = fila["id"]
        hc, hr = hit(cand.get(i, []), rel), hit(ref.get(i, []), rel)
        arregla += hc and not hr
        rompe += hr and not hc
        deltas.append(rr(cand.get(i, []), rel) - rr(ref.get(i, []), rel))
    return {
        "arregla": arregla,
        "rompe": rompe,
        "p_mcnemar": mcnemar_exacto(arregla, rompe),
        "ic95_delta_mrr": bootstrap_ic95(deltas),
    }


def decide(par, neto_min, factor):
    if (
        par["arregla"] - par["rompe"] >= neto_min
        and par["arregla"] >= factor * par["rompe"]
        and par["ic95_delta_mrr"][1] >= 0
    ):
        return "GANA"
    return "NO GANA"


def decide_r1(par_vector, par_fts, neto_min, factor):
    """par_x = pareada(cand=x, ref=sellado). R1 del pre-registro §6."""
    if "GANA" in (decide(par_vector, neto_min, factor), decide(par_fts, neto_min, factor)):
        return "NO GENERALIZA"
    if par_vector["rompe"] >= par_vector["arregla"] and par_fts["rompe"] >= par_fts["arregla"]:
        return "GENERALIZA"
    return "INDETERMINADO"


def _pares_kv(valores):
    out = {}
    for v in valores or []:
        k, x = v.split("=", 1)
        out[k] = x
    return out


def informe(gold, capturas, neto_min, factor, estricto, p95, rebuild_s, ruta_detalle):
    no_nulas = [f for f in gold if f.get("expected_permalink")]
    nulas = [f for f in gold if not f.get("expected_permalink")]
    brazos = {}
    for indice, cap in capturas.items():
        nombres = ["sellado", "vector", "fts", "rrf", "sellado-035"] if indice == "base" else ["sellado", "rrf"]
        for b in nombres:
            brazos[f"{indice}:{b}"] = rankings(cap, b)
    modo, otro = ("strict", "lenient") if estricto else ("lenient", "strict")
    L = [f"# Agregados — campaña C (modo de decisión: {modo})", ""]
    L.append(f"- filas: {len(gold)} · no nulas: {len(no_nulas)} · nulas (corpus negativo): {len(nulas)}")
    estratos = sorted({f.get("source", "?") for f in no_nulas})
    L.append("- no nulas por estrato: " + ", ".join(f"{e}={sum(f.get('source') == e for f in no_nulas)}" for e in estratos))
    base = capturas.get("base", {})
    activas = sum(1 for f in no_nulas if base.get(f["id"]) and base[f["id"]]["fts"])
    L.append(f"- queries no nulas con fusión activa (≥1 candidato FTS en base): {activas}")
    errores = {ind: sum(1 for c in cap.values() if c is None) for ind, cap in capturas.items()}
    L.append(f"- capturas con error por índice: {errores}")
    L += ["", f"| brazo | hit@5 {modo} [Wilson 95%] | hit@5 {otro} | hit@1 | MRR@10 |", "|---|---|---|---|---|"]
    detalle = {f["id"]: {"id": f["id"], "source": f.get("source")} for f in gold}
    for nombre, rk in brazos.items():
        x = sum(hit(rk.get(f["id"], []), relevantes(f, estricto)) for f in no_nulas)
        x2 = sum(hit(rk.get(f["id"], []), relevantes(f, not estricto)) for f in no_nulas)
        h1 = sum(hit(rk.get(f["id"], []), relevantes(f, estricto), k=1) for f in no_nulas)
        mrr = sum(rr(rk.get(f["id"], []), relevantes(f, estricto)) for f in no_nulas) / max(1, len(no_nulas))
        lo, hi = wilson(x, len(no_nulas))
        L.append(f"| {nombre} | {x}/{len(no_nulas)} [{lo:.3f}, {hi:.3f}] | {x2}/{len(no_nulas)} | {h1}/{len(no_nulas)} | {mrr:.4f} |")
        for f in gold:
            rel = relevantes(f, estricto)
            detalle[f["id"]][nombre] = {"hit5": hit(rk.get(f["id"], []), rel), "rr10": rr(rk.get(f["id"], []), rel), "n_resultados": len(rk.get(f["id"], []))}
    L += ["", "## hit@5 por estrato", "", "| brazo | " + " | ".join(estratos) + " |", "|---|" + "---|" * len(estratos)]
    for nombre, rk in brazos.items():
        celdas = []
        for e in estratos:
            fe = [f for f in no_nulas if f.get("source") == e]
            celdas.append(f"{sum(hit(rk.get(f['id'], []), relevantes(f, estricto)) for f in fe)}/{len(fe)}")
        L.append(f"| {nombre} | " + " | ".join(celdas) + " |")
    L += ["", "## corpus negativo (nulas con ≥1 resultado en top-5)", ""]
    for nombre, rk in brazos.items():
        L.append(f"- {nombre}: {sum(1 for f in nulas if rk.get(f['id'], [])[:K_HIT])}/{len(nulas)}")
    pares = {}
    L += ["", "## pareadas (cand vs ref)", "", "| cand | ref | ARREGLA | ROMPE | p McNemar exacto | IC95 ΔMRR@10 | regla GANA |", "|---|---|---|---|---|---|---|"]
    for cand, ref in [("base:vector", "base:sellado"), ("base:fts", "base:sellado"), ("base:rrf", "base:sellado"),
                      ("solape:sellado", "base:sellado"), ("late:sellado", "base:sellado"),
                      ("solape:rrf", "base:rrf"), ("late:rrf", "base:rrf")]:
        if cand in brazos and ref in brazos:
            par = pareada(brazos[cand], brazos[ref], no_nulas, estricto)
            pares[(cand, ref)] = par
            lo, hi = par["ic95_delta_mrr"]
            L.append(f"| {cand} | {ref} | {par['arregla']} | {par['rompe']} | {par['p_mcnemar']:.4f} | [{lo:.4f}, {hi:.4f}] | {decide(par, neto_min, factor)} |")
    L += ["", f"## decisiones (NETO ≥ {neto_min}, ARREGLA ≥ {factor}·ROMPE, veto ΔMRR)", ""]
    if ("base:vector", "base:sellado") in pares:
        L.append(f"- R1 (H7): {decide_r1(pares[('base:vector', 'base:sellado')], pares[('base:fts', 'base:sellado')], neto_min, factor)}")
    if ("base:rrf", "base:sellado") in pares:
        L.append(f"- R2 (H7b, RRF): {'ADOPTAR RRF' if decide(pares[('base:rrf', 'base:sellado')], neto_min, factor) == 'GANA' else 'SE QUEDA LA FUSIÓN ACTUAL'}")
    if ("solape:sellado", "base:sellado") in pares:
        g = decide(pares[("solape:sellado", "base:sellado")], neto_min, factor)
        if g == "GANA" and "solape" in p95 and "base" in p95:
            ok = float(p95["solape"]) <= 1.25 * float(p95["base"])
            L.append(f"- R3 (H14a, solape): {'ADOPTAR SOLAPE' if ok else 'NO (guard de latencia)'} (p95 base={p95['base']} s, solape={p95['solape']} s)")
        else:
            L.append(f"- R3 (H14a, solape): {'GANA retrieval; FALTA p95' if g == 'GANA' else 'SE QUEDA EL TROCEADO ACTUAL'}")
    else:
        L.append("- R3 (H14a, solape): no medido")
    if ("late:sellado", "base:sellado") in pares:
        g = decide(pares[("late:sellado", "base:sellado")], neto_min, factor)
        if g == "GANA" and "late" in rebuild_s and "base" in rebuild_s:
            ok = float(rebuild_s["late"]) <= 3 * float(rebuild_s["base"])
            L.append(f"- R4 (H14b, late): {'GANA → spec de producción aparte' if ok else 'NO (guard de coste de rebuild)'}")
        else:
            L.append(f"- R4 (H14b, late): {'GANA retrieval; FALTA rebuild_s' if g == 'GANA' else 'NO GANA'}")
    else:
        L.append("- R4 (H14b, late): no medido")
    if ruta_detalle:
        with open(ruta_detalle, "w", encoding="utf-8") as fh:
            for d in detalle.values():
                fh.write(json.dumps(d, ensure_ascii=False) + "\n")
    return "\n".join(L) + "\n"


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    a_fid = sub.add_parser("fidelidad")
    a_fid.add_argument("--captura", required=True)
    a_fid.add_argument("--brazo", default="sellado", choices=["sellado", "rrf"])
    a_inf = sub.add_parser("informe")
    a_inf.add_argument("--gold", required=True)
    a_inf.add_argument("--overlay")
    a_inf.add_argument("--captura", action="append", required=True, help="indice=ruta (base|solape|late)")
    a_inf.add_argument("--neto-min", type=int, required=True)
    a_inf.add_argument("--factor", type=float, required=True)
    a_inf.add_argument("--estricto", action="store_true")
    a_inf.add_argument("--p95", action="append", help="indice=segundos")
    a_inf.add_argument("--rebuild-s", action="append", help="indice=segundos")
    a_inf.add_argument("--detalle")
    a_cmp = sub.add_parser("compara")
    a_cmp.add_argument("--a", required=True)
    a_cmp.add_argument("--b", required=True)
    a = ap.parse_args()

    if a.cmd == "fidelidad":
        cap = carga_captura(a.captura)
        malas = fidelidad(cap, a.brazo)
        print(f"fidelidad({a.brazo}): {len(malas)} discrepancias de {len(cap)}")
        for i in malas:
            print(f"  discrepa: {i}", file=sys.stderr)
        sys.exit(1 if malas else 0)
    if a.cmd == "compara":
        ca, cb = carga_captura(a.a), carga_captura(a.b)
        distintas = sorted(i for i in set(ca) | set(cb) if ca.get(i) != cb.get(i))
        print(f"compara: {len(distintas)} queries con capturas distintas de {len(set(ca) | set(cb))}")
        sys.exit(1 if distintas else 0)
    gold = carga_gold(a.gold)
    if a.overlay:
        gold = aplica_overlay(gold, a.overlay)
    capturas = {k: carga_captura(v) for k, v in _pares_kv(a.captura).items()}
    sys.stdout.write(informe(gold, capturas, a.neto_min, a.factor, a.estricto,
                             _pares_kv(a.p95), _pares_kv(a.rebuild_s), a.detalle))


if __name__ == "__main__":
    main()
```

- [ ] **Step 4: Correr y verlo pasar**

Run: `python3 -m unittest -v evals/retrieval-heldout/harness/test_harness.py`
Expected: `OK` (19 tests). Si `test_wilson` falla por redondeo, NO se toca la tolerancia: se revisa la fórmula (Wilson 1927, z=1.959964).

- [ ] **Step 5: Commit**

```bash
git -C /home/paul/Documentos/proyectos/exo switch -c c-retrieval-heldout
git -C /home/paul/Documentos/proyectos/exo add evals/retrieval-heldout/harness/metricas.py evals/retrieval-heldout/harness/test_harness.py docs/superpowers/plans/2026-09-13-campana-c-retrieval-held-out.md docs/superpowers/plans/2026-09-13-campana-c-preregistro.md
git -C /home/paul/Documentos/proyectos/exo commit -m "eval(c): plan, borrador de pre-registro y harness de metricas del held-out"
```

---

### Task 2: Captura (`captura.py`), pool de candidatas y muestreo (`pool.py`)

**Lane:** mecánica.
**Oráculo:** `python3 -m unittest -v evals/retrieval-heldout/harness/test_harness.py` verde, más `python3 evals/retrieval-heldout/harness/pool.py ...` (Step 6) con exit 0 y cuotas cubiertas.
**Depende de:** Task 0 (D1, D2), Task 1.

**Files:**
- Create: `evals/retrieval-heldout/harness/captura.py`
- Create: `evals/retrieval-heldout/harness/pool.py`
- Modify: `evals/retrieval-heldout/harness/test_harness.py` (añadir `TestPool`)
- Privado (fuera del repo): `$PRIV/in-sample-55.jsonl`, `$PRIV/overlay-fila13.jsonl`, `$PRIV/pool.jsonl`, `$PRIV/muestra.jsonl`

**Interfaces:**
- Consumes: `carga_gold` (Task 1); `search(exo_bin, db, query, limite, tipo, min_similitud=None, bonus=None, escala_fts=None) -> dict` de `evals/retrieval-fase0/harness/replay-engine.py:55-73` (cargado con `importlib`, sin modificarlo).
- Produces:
  - `captura.py --gold G --db D --exo BIN --out F` → JSONL `{"id", "errores": [str], "fts": [[permalink, score]] (≤50), "vector": [[permalink, sim]] (umbral 0.0), "hybrid": [[permalink, score]] (≤10)}`
  - `pool.normaliza(q: str) -> str` · `pool.jaccard(a: str, b: str) -> float` · `pool.query_de_comando(cmd: str) -> str|None` · `pool.filtra(candidatas: list[dict], in55: list[str]) -> tuple[list[dict], dict]`
  - `$PRIV/muestra.jsonl`: `{"id": "c001", "query", "source", "session_id", "ts"}` en orden de muestreo (el orden ES la cola de etiquetado de la Task 3)

- [ ] **Step 1: Preparar el directorio privado y el in-sample**

```bash
export PRIV="$HOME/.local/share/exo-evals/c-heldout"
mkdir -p "$PRIV" && chmod 700 "$PRIV"
git -C /home/paul/Documentos/proyectos/exo show archivo/main-pre-reescritura:evals/retrieval-fase0/eval.jsonl > "$PRIV/in-sample-55.jsonl"
wc -l < "$PRIV/in-sample-55.jsonl"
sed -n 13p "$PRIV/in-sample-55.jsonl" | python3 -c 'import sys,json; print(json.loads(sys.stdin.read())["query"])'
printf '%s\n' '{"query":"cge evaluación head-to-head cgeo benchmark harness metodología","acceptable_permalinks":["wisdom-paul/log/cge-bitacora"]}' > "$PRIV/overlay-fila13.jsonl"
```
Expected: `56` y `cge evaluación head-to-head cgeo benchmark harness metodología`. Si la fila 13 no coincide, STOP (el overlay apuntaría a otra query).

- [ ] **Step 2: Tests que fallan de `pool`**

Añadir a `evals/retrieval-heldout/harness/test_harness.py`, antes de `if __name__ == "__main__":`:
```python
import pool as pl  # noqa: E402


class TestPool(unittest.TestCase):
    def test_normaliza(self):
        self.assertEqual(pl.normaliza("  Fábrica   Campaña "), "fabrica campana")

    def test_jaccard(self):
        self.assertEqual(pl.jaccard("a b c", "a b c"), 1.0)
        self.assertEqual(pl.jaccard("a b", "c d"), 0.0)

    def test_query_de_comando(self):
        self.assertEqual(pl.query_de_comando('exo search --db ~/.exo/index.db --type hybrid --json "fabrica campaña"'), "fabrica campaña")
        self.assertEqual(pl.query_de_comando('cd /home/paul && ~/.local/bin/exo search --db x --limite 4 --json "memoria v2" | jq .'), "memoria v2")
        self.assertEqual(pl.query_de_comando("kbx targets memoria --json"), "memoria")
        self.assertIsNone(pl.query_de_comando("cargo test --release"))

    def test_filtra(self):
        in55 = [pl.normaliza("fabrica campaña")]
        cands = [
            {"query": "Fabrica  campaña", "source": "agent-search", "session_id": "s", "ts": "2026-08-01T00:00:00Z"},
            {"query": "-revisa esto", "source": "prompt", "session_id": "s", "ts": "2026-08-23T00:00:00Z"},
            {"query": "x" * 1501, "source": "prompt", "session_id": "s", "ts": "2026-08-23T00:00:00Z"},
            {"query": "memoria v2 contrato", "source": "agent-search", "session_id": "s", "ts": "2026-09-13T08:00:00Z"},
            {"query": "memoria v2 contrato", "source": "agent-search", "session_id": "s", "ts": "2026-08-02T00:00:00Z"},
            {"query": "Memoria v2  contrato", "source": "agent-search", "session_id": "t", "ts": "2026-08-03T00:00:00Z"},
        ]
        pool_, desc = pl.filtra(cands, in55)
        self.assertEqual([c["query"] for c in pool_], ["memoria v2 contrato"])
        self.assertEqual(desc, {"vacia": 0, "guion": 1, "larga": 1, "fuera-de-ventana": 1, "dup-55": 1, "dup-pool": 1})
```

Run: `python3 -m unittest -v evals/retrieval-heldout/harness/test_harness.py`
Expected: ERROR `ModuleNotFoundError: No module named 'pool'`.

- [ ] **Step 3: Implementar `pool.py`**

`evals/retrieval-heldout/harness/pool.py`:
```python
#!/usr/bin/env python3
"""Pool de queries candidatas del held-out C y muestreo estratificado con
semilla (pre-registro §7). Lee SOLO logs locales de ~/.claude; escribe SOLO en
--out-dir (privado). No inventa queries: si una cuota no cabe, sale con 1.

Uso: pool.py --in-sample $PRIV/in-sample-55.jsonl --out-dir $PRIV \
             --cuota prompt=N1 --cuota agent-search=N2
"""
import argparse
import glob
import json
import os
import random
import shlex
import sys
import unicodedata
from datetime import datetime, timedelta
from pathlib import Path

HOME = Path.home()
RETRIEVAL_LOG = HOME / ".claude" / "reflex-retrieval-log.jsonl"
REFLEX_LOG = HOME / ".claude" / "reflex-log.jsonl"
PROYECTOS = HOME / ".claude" / "projects"
VENTANA_INI = "2026-07-19T00:00:00Z"  # día siguiente al sellado del sweep M2-07 (ee839ac, 2026-07-18)
VENTANA_FIN = "2026-09-13T00:00:00Z"  # sesiones de la campaña C ya vieron las 55
MAX_CHARS = 1500
SEMILLA = 20260913
FLAGS_CON_VALOR = {"--db", "--kb", "--limit", "--limite", "--type", "--min-similarity",
                   "--min-similitud", "--bonus", "--fts-scale", "--escala-fts", "--cap-bytes"}
SEPARADORES = {"|", "&&", ";", "||"}


def normaliza(q):
    q = unicodedata.normalize("NFD", q.lower())
    return " ".join("".join(ch for ch in q if unicodedata.category(ch) != "Mn").split())


def jaccard(a, b):
    ta, tb = set(a.split()), set(b.split())
    return len(ta & tb) / len(ta | tb) if ta | tb else 0.0


def query_de_comando(cmd):
    try:
        toks = shlex.split(cmd)
    except ValueError:
        return None
    for i, t in enumerate(toks[:-1]):
        if os.path.basename(t) in ("exo", "kbx") and toks[i + 1] in ("search", "targets"):
            posicionales, j = [], i + 2
            while j < len(toks) and toks[j] not in SEPARADORES:
                if toks[j] in FLAGS_CON_VALOR:
                    j += 2
                    continue
                if not toks[j].startswith("-"):
                    posicionales.append(toks[j])
                j += 1
            return posicionales[-1] if posicionales else None
    return None


def _ts(s):
    return datetime.fromisoformat(s.replace("Z", "+00:00"))


def _transcript(sid):
    rutas = glob.glob(str(PROYECTOS / "*" / f"{sid}.jsonl"))
    return Path(rutas[0]) if rutas else None


def pool_search_notes():
    for linea in RETRIEVAL_LOG.read_text(encoding="utf-8").splitlines():
        e = json.loads(linea)
        if e.get("tool") == "mcp__basic-memory__search_notes":
            yield {"query": e.get("target", ""), "source": "agent-search", "session_id": e.get("session_id", ""), "ts": e.get("ts", "")}


def pool_comandos():
    for t in PROYECTOS.glob("*/*.jsonl"):
        with t.open(encoding="utf-8", errors="ignore") as fh:
            for linea in fh:
                if "search" not in linea and "targets" not in linea:
                    continue
                try:
                    e = json.loads(linea)
                except json.JSONDecodeError:
                    continue
                contenido = (e.get("message") or {}).get("content")
                if not isinstance(contenido, list):
                    continue
                for c in contenido:
                    if not isinstance(c, dict) or c.get("type") != "tool_use" or c.get("name") != "Bash":
                        continue
                    cmd = (c.get("input") or {}).get("command", "")
                    if any(x in cmd for x in ("/tmp/", "target/", "evals/", "cargo ")):
                        continue
                    q = query_de_comando(cmd)
                    if q:
                        yield {"query": q, "source": "agent-search", "session_id": e.get("sessionId", ""), "ts": e.get("timestamp", "")}


def pool_prompts():
    for linea in REFLEX_LOG.read_text(encoding="utf-8").splitlines():
        e = json.loads(linea)
        if e.get("reflex") not in ("recall-inject-emitted", "recall-inject-degraded"):
            continue
        t = _transcript(e.get("session_id", ""))
        if not t or not e.get("ts"):
            continue
        ini, fin = _ts(e["ts"]) - timedelta(seconds=60), _ts(e["ts"]) + timedelta(seconds=2)
        mejor = None
        with t.open(encoding="utf-8", errors="ignore") as fh:
            for l in fh:
                if '"user"' not in l:
                    continue
                u = json.loads(l)
                c = (u.get("message") or {}).get("content")
                if u.get("type") != "user" or u.get("isSidechain") or not isinstance(c, str) or c.lstrip().startswith("<"):
                    continue
                if ini <= _ts(u["timestamp"]) <= fin:
                    mejor = {"query": c, "source": "prompt", "session_id": e["session_id"], "ts": u["timestamp"]}
        if mejor:
            yield mejor


def filtra(candidatas, in55):
    desc = {"vacia": 0, "guion": 0, "larga": 0, "fuera-de-ventana": 0, "dup-55": 0, "dup-pool": 0}
    vistos, pool = set(), []
    for c in candidatas:
        q = c["query"].strip()
        n = normaliza(q)
        if not n:
            desc["vacia"] += 1
        elif q.startswith("-"):
            desc["guion"] += 1
        elif len(q) > MAX_CHARS:
            desc["larga"] += 1
        elif not (VENTANA_INI < c["ts"] < VENTANA_FIN):
            desc["fuera-de-ventana"] += 1
        elif any(n == m or jaccard(n, m) >= 0.8 for m in in55):
            desc["dup-55"] += 1
        elif n in vistos:
            desc["dup-pool"] += 1
        else:
            vistos.add(n)
            pool.append({**c, "query": q})
    return pool, desc


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--in-sample", required=True)
    ap.add_argument("--out-dir", required=True)
    ap.add_argument("--cuota", action="append", required=True, help="source=n (prompt|agent-search)")
    a = ap.parse_args()
    in55 = [normaliza(json.loads(l)["query"]) for l in open(a.in_sample, encoding="utf-8") if l.strip()]
    pool, desc = filtra([*pool_search_notes(), *pool_comandos(), *pool_prompts()], in55)
    rng = random.Random(SEMILLA)
    muestra = []
    for cuota in a.cuota:
        fuente, n = cuota.split("=")
        cands = sorted((c for c in pool if c["source"] == fuente), key=lambda c: (c["ts"], c["query"]))
        rng.shuffle(cands)
        if len(cands) < int(n):
            print(json.dumps({"pool": {fuente: len(cands)}, "descartes": desc}), file=sys.stderr)
            sys.exit(f"pool insuficiente para {fuente}: {len(cands)} < {n} — PENDIENTE-PAUL, no se inventan queries")
        muestra.extend(cands)  # estrato completo barajado: la Task 3 etiqueta en este orden hasta cubrir la cuota
    out = Path(a.out_dir)
    with open(out / "pool.jsonl", "w", encoding="utf-8") as fh:
        for c in pool:
            fh.write(json.dumps(c, ensure_ascii=False) + "\n")
    with open(out / "muestra.jsonl", "w", encoding="utf-8") as fh:
        for k, c in enumerate(muestra, start=1):
            fh.write(json.dumps({"id": f"c{k:03d}", **c}, ensure_ascii=False) + "\n")
    print(json.dumps({"pool": {s: sum(c["source"] == s for c in pool) for s in ("prompt", "agent-search")},
                      "muestra": len(muestra), "descartes": desc}, ensure_ascii=False))


if __name__ == "__main__":
    main()
```
Nota de diseño: `muestra.jsonl` guarda el estrato completo barajado, no solo N. La cuota `n` es el mínimo de candidatas exigible (N no nulas × 1,5 de margen para nulas). La parada secuencial de la Task 3 corta al llegar a N no nulas: eso no selecciona por resultado, porque el orden lo fija la semilla antes de etiquetar.

- [ ] **Step 4: Implementar `captura.py`**

`evals/retrieval-heldout/harness/captura.py`:
```python
#!/usr/bin/env python3
"""Captura por query de las tres listas del binario (pre-registro §4):
FTS --limit 50 (= K_C), vector --limit 1000 --min-similarity 0.0, hybrid
--limit 10 --min-similarity 0.40 --bonus 0.0 --fts-scale 0.6 (explícitos: se
mide lo declarado, no lo que el binario tenga por default).

Reutiliza search() de evals/retrieval-fase0/harness/replay-engine.py sin
modificarlo. Salida privada (--out en $PRIV).
Uso: captura.py --gold G --db D --exo BIN --out F
"""
import argparse
import importlib.util
import json
import sys
from pathlib import Path

_M0 = Path(__file__).resolve().parents[2] / "retrieval-fase0" / "harness"
_spec = importlib.util.spec_from_file_location("replay_engine", _M0 / "replay-engine.py")
replay_engine = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(replay_engine)
sys.path.insert(0, str(Path(__file__).resolve().parent))
from metricas import carga_gold  # noqa: E402


def _pares(res):
    return [[r["permalink"], r["score"]] for r in res["results"]]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--gold", required=True)
    ap.add_argument("--db", required=True)
    ap.add_argument("--exo", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    filas = carga_gold(a.gold)
    with open(a.out, "w", encoding="utf-8") as fh:
        for n, fila in enumerate(filas, start=1):
            q = fila["query"]
            fts = replay_engine.search(a.exo, a.db, q, 50, "fts")
            vec = replay_engine.search(a.exo, a.db, q, 1000, "vector", min_similitud=0.0)
            hyb = replay_engine.search(a.exo, a.db, q, 10, "hybrid", min_similitud=0.40, bonus=0.0, escala_fts=0.6)
            errores = [r["error"] for r in (fts, vec, hyb) if "error" in r]
            linea = {"id": fila["id"], "errores": errores}
            if not errores:
                linea.update(fts=_pares(fts), vector=_pares(vec), hybrid=_pares(hyb))
            fh.write(json.dumps(linea, ensure_ascii=False) + "\n")
            print(f"[{n}/{len(filas)}] {fila['id']}", file=sys.stderr)


if __name__ == "__main__":
    main()
```

- [ ] **Step 5: Tests verdes y humo de `captura.py` sin datos privados**

Run: `python3 -m unittest -v evals/retrieval-heldout/harness/test_harness.py`
Expected: `OK` (23 tests).

Humo de lectura sobre el índice real (solo `exo search`, nada se escribe fuera de `$PRIV`):
```bash
printf '%s\n' '{"id":"h0","query":"fabrica campaña","expected_permalink":null}' > "$PRIV/humo.jsonl"
python3 evals/retrieval-heldout/harness/captura.py --gold "$PRIV/humo.jsonl" --db "$HOME/.exo/index.db" --exo "$HOME/.local/bin/exo" --out "$PRIV/humo-cap.jsonl"
python3 evals/retrieval-heldout/harness/metricas.py fidelidad --captura "$PRIV/humo-cap.jsonl"
```
Expected: `fidelidad(sellado): 0 discrepancias de 1` (exit 0). Si discrepa ya en el binario instalado, STOP y diagnosticar antes de seguir: es el mismo oráculo que usará la Task 6.

- [ ] **Step 6: Generar el pool y la muestra**

Con N = D2 y proporciones de D1 (ejemplo con la recomendación, N=100: prompt 50, agent-search 30; `hard` lo escribe la Task 3):
```bash
python3 evals/retrieval-heldout/harness/pool.py --in-sample "$PRIV/in-sample-55.jsonl" --out-dir "$PRIV" --cuota prompt=75 --cuota agent-search=45
```
Expected: exit 0 y un JSON con `pool`, `muestra` y `descartes`. Si sale `pool insuficiente para …`: se registra en `pendiente-paul.md` con el JSON de stderr y las opciones (bajar N a 60, rebalancear hacia `hard`), y la tarea para. **No** se relajan los filtros.

- [ ] **Step 7: Commit (solo código)**

```bash
git -C /home/paul/Documentos/proyectos/exo add evals/retrieval-heldout/harness/pool.py evals/retrieval-heldout/harness/captura.py evals/retrieval-heldout/harness/test_harness.py
git -C /home/paul/Documentos/proyectos/exo commit -m "eval(c): captura y pool de candidatas con filtros anti-fuga"
```

---

### Task 3: Snapshot de la KB, estrato `hard` y etiquetado del gold (filesystem-only)

**Lane:** diseño (construye el oráculo: gold + corpus negativo).
**Oráculo:** `python3 evals/retrieval-heldout/harness/valida_gold.py --gold "$PRIV/gold.jsonl" --kb "$PRIV/kb-snap" --in-sample "$PRIV/in-sample-55.jsonl"` con exit 0 y recuentos ≥ cuotas.
**Depende de:** Task 2.

**Files:**
- Create: `evals/retrieval-heldout/harness/valida_gold.py`
- Modify: `evals/retrieval-heldout/harness/test_harness.py` (añadir `TestValidaGold`)
- Privado: `$PRIV/kb-snap/` (clone), `$PRIV/hard-candidatas.jsonl`, `$PRIV/gold.jsonl`

**Interfaces:**
- Consumes: `normaliza`, `jaccard` (Task 2); `$PRIV/muestra.jsonl`.
- Produces: `valida_gold.valida(filas: list[dict], permalinks: set[str], in55: list[str]) -> list[str]` (errores) · `valida_gold.permalinks_snapshot(kb: str) -> set[str]` · `$PRIV/gold.jsonl` con el schema del pre-registro §3.

- [ ] **Step 1: Snapshot inmutable de la KB (lectura del repo de la KB, sin escribir en él)**

```bash
S=$(git -C "$HOME/Documentos/proyectos/wisdom-paul" rev-parse HEAD)
git clone --quiet --no-hardlinks "$HOME/Documentos/proyectos/wisdom-paul" "$PRIV/kb-snap"
git -C "$PRIV/kb-snap" checkout --quiet --detach "$S"
echo "$S" > "$PRIV/kb-snap.commit"; cat "$PRIV/kb-snap.commit"
```
Expected: un sha de 40 hex. Ese es `S` del pre-registro §10.

- [ ] **Step 2: Tests que fallan del validador**

Añadir a `test_harness.py` antes de `if __name__ == "__main__":`:
```python
import valida_gold as vg  # noqa: E402


class TestValidaGold(unittest.TestCase):
    PERMS = {"kb/a", "kb/b", "kb/c"}

    def fila(self, **kw):
        base = {"id": "c001", "query": "q nueva", "source": "prompt", "expected_permalink": "kb/a",
                "acceptable_permalinks": [], "notes": "canon del proyecto"}
        base.update(kw)
        return base

    def test_fila_buena(self):
        self.assertEqual(vg.valida([self.fila()], self.PERMS, []), [])

    def test_errores(self):
        casos = [
            self.fila(expected_permalink="kb/zzz"),
            self.fila(acceptable_permalinks=["kb/a"], notes="aceptable: igual"),
            self.fila(acceptable_permalinks=["kb/b"], notes="sin justificar"),
            self.fila(acceptable_permalinks=["kb/b", "kb/c", "kb/a"], notes="aceptable: x"),
            self.fila(expected_permalink=None, acceptable_permalinks=["kb/b"], notes="aceptable: x"),
            self.fila(source="log"),
            self.fila(query="fabrica campaña"),
        ]
        for i, c in enumerate(casos):
            with self.subTest(i=i):
                self.assertTrue(vg.valida([c], self.PERMS, ["fabrica campana"]))

    def test_ids_duplicados(self):
        self.assertTrue(vg.valida([self.fila(), self.fila(query="otra")], self.PERMS, []))
```
Run: `python3 -m unittest -v evals/retrieval-heldout/harness/test_harness.py`
Expected: ERROR `No module named 'valida_gold'`.

- [ ] **Step 3: Implementar `valida_gold.py`**

```python
#!/usr/bin/env python3
"""Valida el gold del held-out C (pre-registro §3) contra el snapshot de la KB.
Exit 1 con la lista de errores; exit 0 imprime recuentos y sha256.
Uso: valida_gold.py --gold G --kb KB --in-sample IN55
"""
import argparse
import hashlib
import json
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from pool import jaccard, normaliza  # noqa: E402

FUENTES = {"prompt", "agent-search", "hard"}


def permalinks_snapshot(kb):
    out = set()
    raiz = Path(kb)
    for p in raiz.rglob("*.md"):
        if any(parte.startswith(".") for parte in p.relative_to(raiz).parts):
            continue
        lineas = p.read_text(encoding="utf-8", errors="ignore").splitlines()
        if not lineas or lineas[0].strip() != "---":
            continue
        for l in lineas[1:]:
            if l.strip() == "---":
                break
            if l.startswith("permalink:"):
                out.add(l.split(":", 1)[1].strip().strip("'\""))
                break
    return out


def valida(filas, permalinks, in55):
    errores, ids = [], Counter(f.get("id") for f in filas)
    for f in filas:
        i = f.get("id")
        if ids[i] > 1:
            errores.append(f"{i}: id duplicado")
        if not {"id", "query", "source", "expected_permalink", "acceptable_permalinks", "notes"} <= set(f):
            errores.append(f"{i}: faltan campos")
            continue
        if f["source"] not in FUENTES:
            errores.append(f"{i}: source inválido {f['source']!r}")
        n = normaliza(f["query"])
        if any(n == m or jaccard(n, m) >= 0.8 for m in in55):
            errores.append(f"{i}: query duplica una de las 55")
        exp, acc = f["expected_permalink"], f["acceptable_permalinks"]
        if exp is None:
            if acc:
                errores.append(f"{i}: fila null con acceptable_permalinks")
            continue
        if exp not in permalinks:
            errores.append(f"{i}: expected_permalink inexistente en el snapshot")
        if len(acc) > 2:
            errores.append(f"{i}: más de 2 acceptable_permalinks")
        if exp in acc or len(set(acc)) != len(acc):
            errores.append(f"{i}: acceptable repite expected o se repite")
        if any(p not in permalinks for p in acc):
            errores.append(f"{i}: acceptable inexistente en el snapshot")
        if acc and "aceptable:" not in f["notes"]:
            errores.append(f"{i}: acceptable sin justificación 'aceptable:' en notes")
    return errores


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--gold", required=True)
    ap.add_argument("--kb", required=True)
    ap.add_argument("--in-sample", required=True)
    a = ap.parse_args()
    filas = [json.loads(l) for l in open(a.gold, encoding="utf-8") if l.strip()]
    in55 = [normaliza(json.loads(l)["query"]) for l in open(a.in_sample, encoding="utf-8") if l.strip()]
    errores = valida(filas, permalinks_snapshot(a.kb), in55)
    for e in errores:
        print(e, file=sys.stderr)
    no_nulas = [f for f in filas if f.get("expected_permalink")]
    print(json.dumps({
        "filas": len(filas), "no_nulas": len(no_nulas), "nulas": len(filas) - len(no_nulas),
        "no_nulas_por_estrato": dict(Counter(f.get("source") for f in no_nulas)),
        "con_acceptable": sum(1 for f in filas if f.get("acceptable_permalinks")),
        "sha256": hashlib.sha256(Path(a.gold).read_bytes()).hexdigest(),
        "errores": len(errores)}, ensure_ascii=False))
    sys.exit(1 if errores else 0)


if __name__ == "__main__":
    main()
```
Run: `python3 -m unittest -v evals/retrieval-heldout/harness/test_harness.py`
Expected: `OK` (26 tests).

- [ ] **Step 4: Estrato `hard` (autor fresco, distinto del etiquetador)**

Lista de notas elegibles:
```bash
git -C "$PRIV/kb-snap" log --diff-filter=A --since=2026-08-17 --name-only --format= -- '*.md' | sort -u > "$PRIV/notas-nuevas.txt"; wc -l < "$PRIV/notas-nuevas.txt"
```
Despachar un subagente **sonnet** fresco con este brief literal:
"Lee notas de `$PRIV/kb-snap/` listadas en `$PRIV/notas-nuevas.txt` (mezcla `learnings/`, `log/`, `backlog`, excluye `archive/` salvo que falten). Por cada nota elegida escribe UNA query en castellano que Paul haría para re-encontrarla SIN sus palabras clave literales ni su título. La mitad en frase natural, la otra mitad en estilo 2–5 palabras clave parafraseadas. PROHIBIDO: ejecutar `exo`, `kbx` o tools de basic-memory; leer `~/Documentos/proyectos/exo/evals/`, `reports/`, `~/.exo/`, `~/.local/share/exo-evals/` salvo `kb-snap` y `notas-nuevas.txt`. Devuelve JSONL `{"query", "source":"hard", "author_expected":"<permalink del frontmatter>"}` en `$PRIV/hard-candidatas.jsonl`, tantas como la cuota `hard` × 1,5."
Registrar en el review-package los comandos que corrió el subagente (del transcript): si aparece `exo `, `kbx ` o `evals/`, se descarta su salida entera y se repite con otro subagente (retries cap 2).

- [ ] **Step 5: Etiquetado (etiquetador fresco, filesystem-only)**

Despachar un subagente **sonnet** fresco (no el autor del Step 4) con este brief literal:
"Etiqueta queries para un gold de retrieval sobre la KB `$PRIV/kb-snap/`. Entrada: `$PRIV/muestra.jsonl` (estratos `prompt` y `agent-search`, EN ESE ORDEN de fichero) y `$PRIV/hard-candidatas.jsonl`. Solo Read/Grep/Glob sobre `kb-snap` (títulos, frontmatter `permalink:`, `core/core-index.md` como mapa). PROHIBIDO: `exo`, `kbx`, basic-memory, leer `evals/`, `reports/`, `~/.exo/`. Por cada query en orden: `expected_permalink` = la nota que un usuario razonable querría recuperar, o `null` si ninguna (un prompt operativo tipo 'mergea' o 'sigue' es null). `acceptable_permalinks`: máx 2 y SOLO si otra nota serviría razonablemente igual; cada uno con una frase `aceptable: <por qué>` en `notes`. Criterio canon vs bitácora (literal de `labels.md:70`): 'query genérica/estado → canon; query histórica/detalle fechado → bitácora'. Para `hard`, verifica `author_expected`, no lo copies. `notes` siempre con la razón. PARADA: por estrato, deja de etiquetar cuando lleves <cuota> filas NO nulas; las nulas etiquetadas antes de parar se quedan. Salida: `$PRIV/gold.jsonl`, schema `{"id","query","source","expected_permalink","acceptable_permalinks","notes"}`, ids `c001..` consecutivos en el orden etiquetado."
Mismo control de transcript que el Step 4.

- [ ] **Step 6: Validar**

Run: el oráculo de la tarea.
Expected: exit 0; `no_nulas_por_estrato` ≥ cuotas de D1/D2; `errores: 0`. Errores ⇒ el etiquetador corrige esas filas (no el orquestador), máx 2 vueltas.

- [ ] **Step 7: Commit (solo código)**

```bash
git -C /home/paul/Documentos/proyectos/exo add evals/retrieval-heldout/harness/valida_gold.py evals/retrieval-heldout/harness/test_harness.py
git -C /home/paul/Documentos/proyectos/exo commit -m "eval(c): validador del gold contra snapshot de la KB"
```

---

### Task 4: Verificación adversarial del gold (modelo fuerte, fila por fila)

**Lane:** diseño (spec fábrica §8: "Verificación adversarial por candidato OBLIGATORIA con verdict preservado", en fable/opus).
**Oráculo:** `valida_gold.py` (Task 3) sigue en exit 0 tras aplicar las correcciones, y `evals/retrieval-heldout/verdict/gold-verificacion-resumen.md` existe con recuentos que cuadran con el verdict privado.
**Depende de:** Task 3.

**Files:**
- Create: `evals/retrieval-heldout/verdict/gold-verificacion-resumen.md` (solo recuentos)
- Privado: `$PRIV/gold-verificacion.md` (verdict completo por fila), `$PRIV/gold.jsonl` (corregido)

**Interfaces:**
- Consumes: `$PRIV/gold.jsonl`, `$PRIV/kb-snap/`.
- Produces: gold corregido; verdict por fila con `CORRECTO | DEFENDIBLE | CORREGIR(<nuevo expected/acceptable/null> + razón)`.

- [ ] **Step 1: Despachar el verificador**

Subagente **fable** (u **opus** si la reserva de fable del ledger está agotada), fresco y que no haya participado en las Tasks 2–3. Brief literal:
"Eres el verificador adversarial de un gold de retrieval (formato y criterio de `~/Documentos/proyectos/exo/evals/retrieval-fase0/verdict/labels.md`: léelo primero, es el precedente aprobado). Gold: `$PRIV/gold.jsonl`. KB: `$PRIV/kb-snap/`, SOLO filesystem (Read/Grep/Glob). PROHIBIDO `exo`, `kbx`, basic-memory, `evals/` salvo `labels.md`, `reports/`. Para CADA fila: (1) ¿existe el `expected_permalink` exacto en frontmatter?; (2) ¿es la nota que un usuario razonable querría?, busca activamente una mejor; (3) cada `acceptable_permalinks`, ¿está justificado o es un comodín que infla hits?; (4) cada `null`, haz grep de los términos clave y confirma que no hay nota. Veredicto por fila: CORRECTO / DEFENDIBLE / CORREGIR con la corrección exacta y la cita de la nota (línea literal). Al final: recuentos por veredicto y por estrato, y las 3 filas más flojas. Escribe en `$PRIV/gold-verificacion.md`."

- [ ] **Step 2: Aplicar correcciones**

El etiquetador de la Task 3 (o un sonnet fresco con el verdict delante) aplica **solo** las filas `CORREGIR`, literal. Sin cambios fuera de esas filas.

Run: `python3 evals/retrieval-heldout/harness/valida_gold.py --gold "$PRIV/gold.jsonl" --kb "$PRIV/kb-snap" --in-sample "$PRIV/in-sample-55.jsonl"`
Expected: exit 0. Si una corrección convierte filas en `null` y un estrato queda por debajo de su cuota, se reanuda la parada secuencial de la Task 3 Step 5 **por el mismo orden** (siguientes de `muestra.jsonl`), y esas filas nuevas pasan otra vez por el Step 1 (retries cap 2).

- [ ] **Step 3: Resumen publicable**

`evals/retrieval-heldout/verdict/gold-verificacion-resumen.md`:
```markdown
# Verificación adversarial del gold — campaña C (resumen publicable)

- Fecha: <salida de `date -I`>
- Verificador: <modelo> fresco, filesystem-only sobre snapshot `<sha de $PRIV/kb-snap.commit>`
- Filas verificadas: <n> / <n> (todas)
- CORRECTO: <n> · DEFENDIBLE: <n> · CORREGIR: <n> (aplicadas: <n>)
- Por estrato (no nulas tras corregir): prompt=<n> · agent-search=<n> · hard=<n>
- Nulas (corpus negativo): <n> · filas con acceptable_permalinks: <n>
- Veredicto completo por fila: privado (`$PRIV/gold-verificacion.md`) — el repo es público.
```
Los `<…>` se rellenan con los valores de `valida_gold.py` y del verdict (son salidas de comando, no decisiones).

- [ ] **Step 4: Commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add evals/retrieval-heldout/verdict/gold-verificacion-resumen.md
git -C /home/paul/Documentos/proyectos/exo commit -m "eval(c): resumen de la verificacion adversarial del gold"
```

---

### Task 5: Aprobación del gold por Paul y congelación del pre-registro (PENDIENTE-PAUL + mecánica)

**Lane:** gate humano, y luego mecánica. **Bloquea toda medición (Tasks 6–12).**
**Oráculo:** `git -C /home/paul/Documentos/proyectos/exo log -1 --format=%H -- docs/superpowers/plans/2026-09-13-campana-c-preregistro.md` devuelve el commit de congelación, y `sha256sum "$PRIV/gold.jsonl"` coincide con §10 del pre-registro.
**Depende de:** Task 4.

**Files:**
- Modify: `docs/superpowers/plans/2026-09-13-campana-c-preregistro.md` (§9 y §10 únicamente; bloque de estado de la cabecera)
- Modify (gitignored): `.superpowers/fabrica/packages/c-retrieval-heldout-gold.md`, `.superpowers/fabrica/pendiente-paul.md`

**Interfaces:**
- Consumes: `$PRIV/gold.jsonl`, resumen de la Task 4, decisiones D0–D3 de la Task 0.
- Produces: pre-registro congelado (commit) con D4, D5, `S`, sha256 y recuentos; `chmod 444 $PRIV/gold.jsonl`.

- [ ] **Step 1: Review-package para Paul**

`.superpowers/fabrica/packages/c-retrieval-heldout-gold.md` con:
- ruta del gold privado y comando para abrirlo: `python3 -c 'import json,sys; [print(json.dumps(json.loads(l),ensure_ascii=False,indent=1)) for l in open(sys.argv[1])]' "$PRIV/gold.jsonl" | less`;
- salida de `valida_gold.py`;
- `gold-verificacion-resumen.md` y las 3 filas más flojas del verdict privado;
- alcance de revisión propuesto: **todas** las nulas + todas las filas con `acceptable_permalinks` + todas las `CORREGIR` + 20 filas al azar (`python3 -c 'import json,random; f=[json.loads(l)["id"] for l in open("'"$PRIV"'/gold.jsonl")]; print(sorted(random.Random(20260913).sample(f, 20)))'`) — Paul puede ampliarlo a todo;
- D4 y D5 con opciones y recomendación, copiadas de §Decisiones abiertas.

- [ ] **Step 2: Esperar el veredicto**

Líneas requeridas en el package:
```
GATE: GOLD APROBADO <fecha>   |   GATE: GOLD RECHAZADO <filas y motivo>
D4 NETO>=<n> ARREGLA>=<x>*ROMPE
D5 <lenient|strict> overlay-fila13=<si|no>
```
`RECHAZADO` ⇒ se corrigen las filas señaladas (vuelve a Task 4 Step 2 para esas filas), nuevo package. Nada se mide mientras tanto.

- [ ] **Step 3: Congelar**

```bash
chmod 444 "$PRIV/gold.jsonl"
python3 evals/retrieval-heldout/harness/valida_gold.py --gold "$PRIV/gold.jsonl" --kb "$PRIV/kb-snap" --in-sample "$PRIV/in-sample-55.jsonl"
cat "$PRIV/kb-snap.commit"
```
Editar el pre-registro:
- Cabecera: sustituir `**Estado: BORRADOR hasta el commit de congelación (Task 5 del plan` … hasta el fin de ese párrafo por `**Estado: CONGELADO el <date -Iseconds>.** Inmutable desde este commit; erratas → verdict.`
- §6: si D4 ≠ (3, 2), sustituir `NETO ≥ 3` y `ARREGLA ≥ 2·ROMPE` por los valores de Paul.
- §9: rellenar cada `____` con la línea literal de Paul (D0–D3 de la Task 0, D4–D5 del Step 2).
- §10: `S` = contenido de `kb-snap.commit`; binario = `pendiente: post-A (Task 6 lo anota en agregados.md, no aquí)`; `sha256` y recuentos = salida JSON de `valida_gold.py`; aprobación = ruta del package + fecha de la línea `GATE:`.

- [ ] **Step 4: Commit de congelación**

```bash
git -C /home/paul/Documentos/proyectos/exo add docs/superpowers/plans/2026-09-13-campana-c-preregistro.md
git -C /home/paul/Documentos/proyectos/exo commit -m "eval(c): congela pre-registro del held-out (gold aprobado por Paul, sha256 fijado)"
git -C /home/paul/Documentos/proyectos/exo log -1 --format='%H %cI'
```
Expected: hash y fecha. Registrar en el ledger `preregistro_congelado: <hash>`. Desde aquí, `git diff <hash> -- docs/superpowers/plans/2026-09-13-campana-c-preregistro.md` debe salir vacío en cualquier tarea posterior.

---

### Task 6: Binario de medición post-A, índice `base` y oráculo de fidelidad

**Lane:** mecánica.
**Oráculo:** `python3 evals/retrieval-heldout/harness/metricas.py fidelidad --captura "$PRIV/cap-base.jsonl"` y lo mismo con `cap-base-55.jsonl`, ambos `0 discrepancias` (exit 0).
**Depende de:** Task 5 **y merge de la campaña A en `main`**.

**Files:**
- Privado: `$PRIV/exo-medicion`, `$PRIV/idx-base.db`, `$PRIV/rebuild-base.time`, `$PRIV/cap-base.jsonl`, `$PRIV/cap-base-55.jsonl`
- Create: `evals/retrieval-heldout/verdict/condiciones.md`

**Interfaces:**
- Consumes: `captura.py`, `metricas.py fidelidad` (Tasks 1–2); `exo rebuild --db <ruta> --kb <ruta>` (`engine/src/main.rs:181-193`, `ArgsIndex`).
- Produces: `$PRIV/exo-medicion` (binario congelado para Tasks 7–9), `idx-base.db`, capturas `base`, tiempo de rebuild `base` en segundos (Task 9 lo pasa como `--rebuild-s base=`).

- [ ] **Step 1: Verificar que A está mergeada y la congelación intacta**

```bash
git -C /home/paul/Documentos/proyectos/exo fetch --quiet origin 2>/dev/null; git -C /home/paul/Documentos/proyectos/exo log --oneline -20 main
git -C /home/paul/Documentos/proyectos/exo diff --stat "$(git -C /home/paul/Documentos/proyectos/exo log -1 --format=%H -- docs/superpowers/plans/2026-09-13-campana-c-preregistro.md)" -- docs/superpowers/plans/2026-09-13-campana-c-preregistro.md
sha256sum "$PRIV/gold.jsonl"
```
Expected: el merge de A visible en `main`; diff vacío; sha256 == §10. Cualquier otra cosa ⇒ STOP.

- [ ] **Step 2: Compilar y congelar el binario**

```bash
git -C /home/paul/Documentos/proyectos/exo rev-parse main
cargo build --release --locked --manifest-path /home/paul/Documentos/proyectos/exo/engine/Cargo.toml
cp /home/paul/Documentos/proyectos/exo/engine/target/release/exo "$PRIV/exo-medicion"
"$PRIV/exo-medicion" --version
```
(Si la rama de trabajo no es `main`, compilar desde un worktree de `main`: `git -C /home/paul/Documentos/proyectos/exo worktree add .worktrees/c-medicion main` y `--manifest-path .worktrees/c-medicion/engine/Cargo.toml`.)

- [ ] **Step 3: Rebuild del índice `base` desde el snapshot**

```bash
/usr/bin/time -v "$PRIV/exo-medicion" rebuild --db "$PRIV/idx-base.db" --kb "$PRIV/kb-snap" --json > "$PRIV/rebuild-base.json" 2> "$PRIV/rebuild-base.time"
grep -E 'Elapsed|Maximum resident' "$PRIV/rebuild-base.time"
python3 -c 'import json,sys; d=json.load(open(sys.argv[1]))["data"]; print({k: d[k] for k in ("unreadable","chunks_embedded","chunks_reused")})' "$PRIV/rebuild-base.json"
```
Expected: exit 0; `chunks_reused: 0` (rebuild desde cero); tiempo y RSS pico anotados. **Nunca** `--db` por defecto: `~/.exo/index.db` no se toca.

- [ ] **Step 4: Capturas `base`**

```bash
python3 evals/retrieval-heldout/harness/captura.py --gold "$PRIV/gold.jsonl" --db "$PRIV/idx-base.db" --exo "$PRIV/exo-medicion" --out "$PRIV/cap-base.jsonl"
python3 evals/retrieval-heldout/harness/captura.py --gold "$PRIV/in-sample-55.jsonl" --db "$PRIV/idx-base.db" --exo "$PRIV/exo-medicion" --out "$PRIV/cap-base-55.jsonl"
python3 -c 'import json,sys; print(sum(1 for f in sys.argv[1:] for l in open(f) if json.loads(l)["errores"]))' "$PRIV/cap-base.jsonl" "$PRIV/cap-base-55.jsonl"
```
Expected: último comando `0`. Si no, circuit breaker del pre-registro §11 (reintentar captura completa, cap 2).

- [ ] **Step 5: Oráculo de fidelidad**

Run: el oráculo de la tarea (dos comandos).
Expected: `fidelidad(sellado): 0 discrepancias de <n>` en ambos. **No** se ejecuta `metricas.py informe` en esta tarea (ningún hit@k antes de tener todas las capturas).
Si discrepa: STOP, `retries` +1 en ledger; diagnosticar (¿A cambió `K_C`, el orden FTS o la normalización?) leyendo `engine/src/buscador.rs` post-A; al tercer fallo, PENDIENTE-PAUL.

- [ ] **Step 6: Condiciones publicables**

`evals/retrieval-heldout/verdict/condiciones.md`:
```markdown
# Condiciones de medición — campaña C

- Pre-registro congelado: `<hash del commit de la Task 5>`
- Binario: `main` en `<git rev-parse main>` (post-campaña A), `cargo build --release --locked`, `exo --version` = `<salida>`
- Snapshot KB: `<kb-snap.commit>`
- Modelo: jinaai/jina-embeddings-v2-base-es @ 8e2d780d8fd38f81ca9123ee28e4c5a968aaf21e
- Índice base: rebuild <Elapsed> · RSS pico <kB> · `chunks_embedded` <n> · `unreadable` <n> (de `rebuild-base.json`)
- Fidelidad sellado: held-out 0/<n> discrepancias · in-sample 0/56
```

```bash
git -C /home/paul/Documentos/proyectos/exo add evals/retrieval-heldout/verdict/condiciones.md
git -C /home/paul/Documentos/proyectos/exo commit -m "eval(c): condiciones de medicion y fidelidad del indice base"
```

---

### Task 7: Rama experimental `exp/c-solape` e índice `solape`

**Lane:** mecánica.
**Oráculo:** `cargo test --release --lib trozos --manifest-path .worktrees/c-solape/engine/Cargo.toml` verde + `rebuild --json` exit 0 con `chunks_embedded` > el de `base`.
**Depende de:** Task 6. **Nunca se mergea** (el cambio de producción, si procede, es la Task 12, re-implementado con TDD sobre `main`).

**Files:**
- Modify (solo en el worktree): `.worktrees/c-solape/engine/src/trozos.rs:1-9` (doc + constante), `:96-106` (`corta_duro`), `:156-165` (test `bloque_que_excede_900_se_corta_duro_sin_solape`)
- Privado: `$PRIV/exo-solape`, `$PRIV/idx-solape.db`, `$PRIV/rebuild-solape.{json,time}`

**Interfaces:**
- Consumes: `main` post-A; `$PRIV/kb-snap`.
- Produces: `trozos::trocea(cuerpo: &str) -> Vec<String>` con la misma firma; `SOLAPE_CHARS: usize = 180`; binario `$PRIV/exo-solape`; `idx-solape.db`.

- [ ] **Step 1: Worktree**

```bash
git -C /home/paul/Documentos/proyectos/exo worktree add .worktrees/c-solape -b exp/c-solape main
```
(`.worktrees/` no está en `.gitignore`: no hacer `git add` de ese directorio desde el árbol principal.)

- [ ] **Step 2: Test que falla**

En `.worktrees/c-solape/engine/src/trozos.rs`, sustituir el test `bloque_que_excede_900_se_corta_duro_sin_solape` entero por:
```rust
    #[test]
    fn bloque_que_excede_900_se_corta_con_solape_de_180() {
        let bloque: String = (0..1000).map(|i| char::from(b'a' + (i % 26) as u8)).collect();
        let trozos = trocea(&bloque);
        assert_eq!(trozos.len(), 2);
        assert_eq!(trozos[0].chars().count(), 900);
        assert_eq!(trozos[1].chars().count(), 280);
        // el segundo trozo empieza 720 chars dentro: sus 180 primeros son la cola del primero
        let chars: Vec<char> = bloque.chars().collect();
        assert_eq!(trozos[1], chars[720..].iter().collect::<String>());
        assert_eq!(&trozos[0][720..], &trozos[1][..180]);
    }

    #[test]
    fn corte_duro_no_emite_trozo_final_contenido_en_el_anterior() {
        // 900 + 720 = 1620: la segunda ventana llega justo al final, no hay tercera
        let bloque = "x".repeat(1620);
        let trozos = trocea(&bloque);
        assert_eq!(trozos.iter().map(|t| t.chars().count()).collect::<Vec<_>>(), vec![900, 900]);
    }
```
(Los bloques son ASCII: el slicing por bytes `[720..]` es seguro en el test.)

Run: `cargo test --release --lib trozos --manifest-path /home/paul/Documentos/proyectos/exo/.worktrees/c-solape/engine/Cargo.toml`
Expected: FAIL en `bloque_que_excede_900_se_corta_con_solape_de_180` (`left: 100, right: 280`).

- [ ] **Step 3: Implementación mínima**

En el mismo fichero, cabecera `:1-9`:
```rust
//! Chunking propio del engine (spec `2026-07-17-indexer-design.md` §2.1):
//! unidad = bloques markdown, empaquetado greedy hasta máx 900 chars por
//! trozo; un bloque que solo exceda el máximo se corta en ventanas de 900 con
//! paso 720 (solape de 180). RAMA EXPERIMENTAL `exp/c-solape` de la campaña
//! C (pre-registro 2026-09-13 §4): NO se mergea.

/// Techo de un trozo, en caracteres Unicode (no bytes) — spec §2.1.
const MAX_CHARS: usize = 900;
/// Solape entre ventanas consecutivas de un corte duro (20% de MAX_CHARS,
/// valor único declarado en el pre-registro, no afinado).
const SOLAPE_CHARS: usize = 180;
```
Y `corta_duro` `:96-106`:
```rust
/// Corte duro de un bloque que por sí solo excede `MAX_CHARS`: ventanas de
/// `MAX_CHARS` caracteres con paso `MAX_CHARS - SOLAPE_CHARS`; la última
/// ventana termina en el final del bloque. Trabaja sobre `Vec<char>` para no
/// partir un carácter UTF-8 multibyte.
fn corta_duro(bloque: &str) -> Vec<String> {
    let chars: Vec<char> = bloque.chars().collect();
    let paso = MAX_CHARS - SOLAPE_CHARS;
    let mut trozos = Vec::new();
    let mut inicio = 0;
    loop {
        let fin = (inicio + MAX_CHARS).min(chars.len());
        trozos.push(chars[inicio..fin].iter().collect());
        if fin == chars.len() {
            break;
        }
        inicio += paso;
    }
    trozos
}
```

- [ ] **Step 4: Verde**

Run: `cargo test --release --lib trozos --manifest-path /home/paul/Documentos/proyectos/exo/.worktrees/c-solape/engine/Cargo.toml`
Expected: PASS (todos los tests de `trozos`, incluido `empaquetado_junta_cuando_cabe_bajo_900`, que confirma que el empaquetado de bloques cortos no cambia).

- [ ] **Step 5: Commit en la rama experimental, binario e índice**

```bash
git -C /home/paul/Documentos/proyectos/exo/.worktrees/c-solape add engine/src/trozos.rs
git -C /home/paul/Documentos/proyectos/exo/.worktrees/c-solape commit -m "exp(c): solape de 180 chars en corta_duro (NO MERGEAR)"
cargo build --release --locked --manifest-path /home/paul/Documentos/proyectos/exo/.worktrees/c-solape/engine/Cargo.toml
cp /home/paul/Documentos/proyectos/exo/.worktrees/c-solape/engine/target/release/exo "$PRIV/exo-solape"
/usr/bin/time -v "$PRIV/exo-solape" rebuild --db "$PRIV/idx-solape.db" --kb "$PRIV/kb-snap" --json > "$PRIV/rebuild-solape.json" 2> "$PRIV/rebuild-solape.time"
grep -E 'Elapsed|Maximum resident' "$PRIV/rebuild-solape.time"
python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["chunks_embedded"])' "$PRIV/rebuild-solape.json"
```
Expected: exit 0; `chunks_embedded` mayor que el de `base` (estimación de recon: ~+25% en trozos de corte duro). La consulta se mide con `$PRIV/exo-medicion`, no con `exo-solape` (el troceado solo afecta al indexado; el binario de búsqueda es el mismo — Task 9).

---

### Task 8: Rama experimental `exp/c-late` e índice `late` (CONDICIONAL: D3 = sí)

**Lane:** diseño (pieza nueva con riesgo de memoria; spec-first ligero = esta tarea + recon pre-flight obligatorio del ejecutor contra la crate).
**Oráculo:** `cargo test --release --test late_chunking --manifest-path .worktrees/c-late/engine/Cargo.toml` verde + `rebuild --json` exit 0 sin OOM y en ≤ 3× el tiempo de `base`.
**Depende de:** Task 6. Si D3 ≠ sí: la tarea se marca `no aplica` en el ledger y R4 queda "no medido". **Nunca se mergea.**

**Files:**
- Modify (worktree): `.worktrees/c-late/engine/src/lib.rs` (añadir `VENTANA_LATE_TOKENS` y `Embedder::embebe_late` dentro de `impl Embedder`, tras `embebe_batch`, hoy `lib.rs:263-269`)
- Modify (worktree): `.worktrees/c-late/engine/src/indexer.rs` (bloque `recien_embebidos` de `reindexa_trozos_de_nota`, hoy `indexer.rs:340-350`)
- Create (worktree): `.worktrees/c-late/engine/tests/late_chunking.rs`
- Privado: `$PRIV/exo-late`, `$PRIV/idx-late.db`, `$PRIV/rebuild-late.{json,time}`

**Interfaces:**
- Consumes: `fastembed::OutputKey`, `TextEmbedding::transform(&mut self, texts, batch_size: Option<usize>) -> Result<EmbeddingOutput>`, `EmbeddingOutput::into_raw(self) -> Vec<SingleBatchOutput>`, `SingleBatchOutput::select_output(&self, &impl OutputPrecedence) -> anyhow::Result<ArrayView<f32, IxDyn>>`, `te.tokenizer.encode(&str, bool) -> Result<Encoding>`, `Encoding::get_offsets() -> &[(usize, usize)]`, `Encoding::get_overflowing() -> &Vec<Encoding>`, `Tokenizer::get_truncation_mut() -> Option<&mut TruncationParams>` (evidencia en §Viabilidad).
- Produces: `pub const VENTANA_LATE_TOKENS: usize = 2048;` · `Embedder::embebe_late(&mut self, trozos: &[String], ventana: usize) -> Result<Vec<Vec<f32>>>` (un vector L2-normalizado de 768 por trozo, mismo orden).

- [ ] **Step 1: Recon pre-flight (sin código)**

El ejecutor confirma en `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` las firmas del bloque Consumes (líneas citadas en §Viabilidad) y que `select_output(...).as_slice()` existe en la versión de `ndarray` que resuelve `engine/Cargo.lock`. Si alguna firma no casa: STOP y PENDIENTE-PAUL con la evidencia (no se sustituye por `ort` directo sin decisión).

- [ ] **Step 2: Worktree y test que falla**

```bash
git -C /home/paul/Documentos/proyectos/exo worktree add .worktrees/c-late -b exp/c-late main
```
`.worktrees/c-late/engine/tests/late_chunking.rs`:
```rust
//! Late chunking experimental (campaña C, pre-registro 2026-09-13 §4).
//! Exige la caché local del modelo ONNX, como el resto de suites que embeben.
use exo::{VENTANA_LATE_TOKENS, con_embedder_de_proceso};

fn coseno(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

#[test]
fn late_da_un_vector_unitario_por_trozo_en_orden() {
    let trozos = vec![
        "La fábrica de campañas deja a Paul fuera del critical path.".to_string(),
        "El gate de merge es asíncrono y se registra en el package.".to_string(),
        "Receta de lentejas con chorizo.".to_string(),
    ];
    let v = con_embedder_de_proceso(|e| e.embebe_late(&trozos, VENTANA_LATE_TOKENS)).unwrap();
    assert_eq!(v.len(), 3);
    for x in &v {
        assert_eq!(x.len(), 768);
        let norma: f32 = x.iter().map(|c| c * c).sum::<f32>().sqrt();
        assert!((norma - 1.0).abs() < 1e-4, "norma {norma}");
    }
}

#[test]
fn late_con_un_solo_trozo_se_parece_al_embedding_normal() {
    let t = vec!["Late chunking agrupa tokens por trozo tras pasar la nota entera.".to_string()];
    let late = con_embedder_de_proceso(|e| e.embebe_late(&t, VENTANA_LATE_TOKENS)).unwrap();
    let normal = con_embedder_de_proceso(|e| e.embebe_batch(&t)).unwrap();
    // difieren solo en que late excluye [CLS]/[SEP] del mean pooling
    assert!(coseno(&late[0], &normal[0]) > 0.9, "coseno {}", coseno(&late[0], &normal[0]));
}

#[test]
fn late_ventana_larga_no_trunca_y_restaura_512_para_queries() {
    // ~30 trozos de 900 chars ≈ >2048 tokens: obliga a varias macro-ventanas
    let trozos: Vec<String> = (0..30)
        .map(|i| format!("Entrada {i} de bitácora: ").repeat(40).chars().take(900).collect())
        .collect();
    let v = con_embedder_de_proceso(|e| e.embebe_late(&trozos, VENTANA_LATE_TOKENS)).unwrap();
    assert_eq!(v.len(), 30);
    let largo = "palabra ".repeat(3000);
    // tras embebe_late la truncación vuelve a 512: la query de producción no cambia
    let tokens = con_embedder_de_proceso(|e| Ok(e.tokens_truncados(&largo))).unwrap();
    assert_eq!(tokens, 512);
}
```
Run: `cargo test --release --test late_chunking --manifest-path /home/paul/Documentos/proyectos/exo/.worktrees/c-late/engine/Cargo.toml`
Expected: FAIL de compilación (`no function or associated item named embebe_late`, `VENTANA_LATE_TOKENS` no existe).

- [ ] **Step 3: Implementación en `lib.rs`**

Junto a `MODELO_JINA_ES` (`lib.rs:167`):
```rust
/// Ventana de late chunking en tokens (campaña C, pre-registro §4). 2048 y
/// no 8192: a 8192 la atención float32 de 12 cabezas son ~3,2 GB por matriz.
pub const VENTANA_LATE_TOKENS: usize = 2048;
```
Dentro de `impl Embedder`, tras `embebe_batch`:
```rust
    /// Late chunking (Günther et al., 2024, arXiv:2409.04701) por
    /// macro-ventanas: agrupa trozos consecutivos hasta `ventana` tokens, pasa
    /// cada ventana UNA vez por el modelo, y embebe cada trozo como el mean
    /// pooling L2-normalizado del `last_hidden_state` de SUS tokens (sin
    /// especiales). La truncación del tokenizer se abre a `ventana` solo
    /// durante la llamada y se restaura después (las queries siguen a 512).
    pub fn embebe_late(&mut self, trozos: &[String], ventana: usize) -> Result<Vec<Vec<f32>>> {
        let original = self
            .te
            .tokenizer
            .get_truncation_mut()
            .map(|t| std::mem::replace(&mut t.max_length, ventana))
            .context("tokenizer sin truncación configurada")?;
        let resultado = self.embebe_late_sin_restaurar(trozos, ventana);
        if let Some(t) = self.te.tokenizer.get_truncation_mut() {
            t.max_length = original;
        }
        resultado
    }

    fn embebe_late_sin_restaurar(&mut self, trozos: &[String], ventana: usize) -> Result<Vec<Vec<f32>>> {
        use fastembed::OutputKey;
        const SEP: &str = "\n\n";
        let presupuesto = ventana.saturating_sub(16);
        let mut salida = Vec::with_capacity(trozos.len());
        let mut i = 0;
        while i < trozos.len() {
            let mut texto = String::new();
            let mut spans: Vec<(usize, usize)> = Vec::new();
            let mut tokens = 0usize;
            let mut j = i;
            while j < trozos.len() {
                let n = self
                    .te
                    .tokenizer
                    .encode(trozos[j].as_str(), false)
                    .map_err(|e| anyhow::anyhow!("tokenizar trozo {j}: {e}"))?
                    .len();
                if j > i && tokens + n > presupuesto {
                    break;
                }
                if j > i {
                    texto.push_str(SEP);
                }
                let ini = texto.len();
                texto.push_str(&trozos[j]);
                spans.push((ini, texto.len()));
                tokens += n;
                j += 1;
            }
            let enc = self
                .te
                .tokenizer
                .encode(texto.as_str(), true)
                .map_err(|e| anyhow::anyhow!("tokenizar ventana: {e}"))?;
            anyhow::ensure!(enc.get_overflowing().is_empty(), "ventana truncada: {tokens} tokens > {ventana}");
            let offsets = enc.get_offsets().to_vec();
            let lotes = self
                .te
                .transform([texto.as_str()], Some(1))
                .context("transform de la ventana")?
                .into_raw();
            let lote = lotes.first().context("transform sin lotes")?;
            let tensor = lote.select_output(&OutputKey::ByName("last_hidden_state"))?;
            let forma = tensor.shape().to_vec();
            anyhow::ensure!(
                forma.len() == 3 && forma[0] == 1 && forma[1] == offsets.len(),
                "forma {forma:?} no casa con {} offsets",
                offsets.len()
            );
            let dim = forma[2];
            let datos = tensor.as_slice().context("tensor no contiguo")?;
            for (ini, fin) in spans {
                let mut v = vec![0f32; dim];
                let mut n = 0usize;
                for (t, (a, b)) in offsets.iter().enumerate() {
                    if b > a && *a >= ini && *a < fin {
                        for (x, y) in v.iter_mut().zip(&datos[t * dim..(t + 1) * dim]) {
                            *x += *y;
                        }
                        n += 1;
                    }
                }
                anyhow::ensure!(n > 0, "trozo sin tokens en la ventana");
                let norma = v.iter().map(|x| x * x).sum::<f32>().sqrt();
                anyhow::ensure!(norma > 0.0, "vector nulo");
                salida.push(v.into_iter().map(|x| x / norma).collect());
            }
            i = j;
        }
        Ok(salida)
    }

    /// Tokens de `texto` con la truncación ACTUAL del tokenizer (test de que
    /// `embebe_late` la restaura).
    pub fn tokens_truncados(&self, texto: &str) -> usize {
        self.te.tokenizer.encode(texto, true).map(|e| e.len()).unwrap_or(0)
    }
```

- [ ] **Step 4: Cablear en el indexer (solo en la rama)**

En `.worktrees/c-late/engine/src/indexer.rs`, sustituir el bloque `let recien_embebidos: HashMap<String, Vec<f32>> = if pendientes.is_empty() { … } else { … };` por:
```rust
    // EXP campaña C (late chunking): los vectores dependen de la nota entera,
    // así que se embeben TODOS los trozos de la nota juntos. Solo válido con
    // rebuild desde cero (`previos` vacío); el cache por texto es incorrecto
    // bajo late chunking y esta rama no se mergea.
    let recien_embebidos: HashMap<String, Vec<f32>> = if pendientes.is_empty() {
        HashMap::new()
    } else {
        let vectores = con_embedder_de_proceso(|embedder| {
            embedder.embebe_late(&textos, crate::VENTANA_LATE_TOKENS)
        })
        .with_context(|| format!("late chunking de {} trozos de {permalink}", textos.len()))?;
        let mut mapa = HashMap::new();
        for (texto, vector) in textos.iter().zip(vectores) {
            mapa.entry(texto.clone()).or_insert(vector);
        }
        mapa
    };
```
(Un texto repetido dentro de la misma nota se queda con el vector de su primera aparición: declarado.)

- [ ] **Step 5: Verde**

Run: `cargo test --release --test late_chunking --manifest-path /home/paul/Documentos/proyectos/exo/.worktrees/c-late/engine/Cargo.toml`
Expected: PASS (3 tests). Si `late_con_un_solo_trozo_se_parece_al_embedding_normal` da coseno ≤ 0.9: STOP y diagnóstico (offsets o pooling mal), no se baja el umbral.
Run: `cargo test --release --test indexer --manifest-path /home/paul/Documentos/proyectos/exo/.worktrees/c-late/engine/Cargo.toml`
Expected: puede fallar el test de cache de embeddings (`tests/cache_embeddings.rs` no se corre aquí); `tests/indexer.rs` debe pasar — si falla, anotar cuál en el review-package: la rama es experimental y no pasa por CI.

- [ ] **Step 6: Commit en la rama, binario e índice con guard de memoria**

```bash
git -C /home/paul/Documentos/proyectos/exo/.worktrees/c-late add engine/src/lib.rs engine/src/indexer.rs engine/tests/late_chunking.rs
git -C /home/paul/Documentos/proyectos/exo/.worktrees/c-late commit -m "exp(c): late chunking por macro-ventanas de 2048 tokens (NO MERGEAR)"
cargo build --release --locked --manifest-path /home/paul/Documentos/proyectos/exo/.worktrees/c-late/engine/Cargo.toml
cp /home/paul/Documentos/proyectos/exo/.worktrees/c-late/engine/target/release/exo "$PRIV/exo-late"
BASE_S=$(python3 -c 'import re,sys; m=re.search(r"Elapsed.*: (?:(\d+):)?(\d+):(\d+(?:\.\d+)?)", open(sys.argv[1]).read()); h,mi,s=m.groups(); print(int(int(h or 0)*3600+int(mi)*60+float(s)))' "$PRIV/rebuild-base.time")
timeout $((BASE_S*3)) /usr/bin/time -v "$PRIV/exo-late" rebuild --db "$PRIV/idx-late.db" --kb "$PRIV/kb-snap" --json > "$PRIV/rebuild-late.json" 2> "$PRIV/rebuild-late.time"; echo "rc=$?"
grep -E 'Elapsed|Maximum resident' "$PRIV/rebuild-late.time"
```
Expected: `rc=0`. `rc=124` (timeout 3× base), `rc=137` (OOM killer) o `Maximum resident set size` cerca de la RAM total (`free -k`) con swap disparado ⇒ circuit breaker §11: brazo **no medido**, anotarlo en ledger y seguir a Task 9 sin `late`. **No** se reintenta con ventana menor (sería afinar).

---

### Task 9: Medición: capturas de variantes, latencia e informe agregado

**Lane:** mecánica.
**Oráculo:** `metricas.py fidelidad` en 0 discrepancias para `cap-solape.jsonl` y `cap-late.jsonl` (si existe), seguido de `metricas.py informe ... > evals/retrieval-heldout/verdict/agregados.md` con exit 0.
**Depende de:** Task 6, Task 7 y Task 8 (o su marca `no aplica`/`no medido`).

**Files:**
- Create: `evals/retrieval-heldout/verdict/agregados.md`, `evals/retrieval-heldout/verdict/agregados-in-sample.md`, `evals/retrieval-heldout/verdict/latencia.md`
- Privado: `$PRIV/cap-{solape,late}{,-55}.jsonl`, `$PRIV/lat-{base,solape,late}.json`, `$PRIV/detalle.jsonl`, `$PRIV/detalle-55.jsonl`

**Interfaces:**
- Consumes: `$PRIV/exo-medicion`, `idx-{base,solape,late}.db`, `cap-base*.jsonl`, D4/D5 del pre-registro congelado (§6, §9).
- Produces: los agregados que adjudica la Task 10.

- [ ] **Step 1: Capturas de las variantes (mismo binario de búsqueda)**

```bash
for IDX in solape late; do
  [ -f "$PRIV/idx-$IDX.db" ] || { echo "sin índice $IDX (no medido)"; continue; }
  python3 evals/retrieval-heldout/harness/captura.py --gold "$PRIV/gold.jsonl" --db "$PRIV/idx-$IDX.db" --exo "$PRIV/exo-medicion" --out "$PRIV/cap-$IDX.jsonl"
  python3 evals/retrieval-heldout/harness/captura.py --gold "$PRIV/in-sample-55.jsonl" --db "$PRIV/idx-$IDX.db" --exo "$PRIV/exo-medicion" --out "$PRIV/cap-$IDX-55.jsonl"
  python3 evals/retrieval-heldout/harness/metricas.py fidelidad --captura "$PRIV/cap-$IDX.jsonl" || exit 1
done
```
Expected: `fidelidad(sellado): 0 discrepancias` por cada índice presente. Un `late` con rebuild abortado en la Task 8 no tiene `idx-late.db` completo: si el fichero existe pero `rebuild-late.json` está vacío, borrar `idx-late.db` del privado antes de este paso.

- [ ] **Step 2: Latencia (R3)**

```bash
for IDX in base solape late; do
  [ -f "$PRIV/idx-$IDX.db" ] || continue
  hyperfine --warmup 3 --runs 30 --export-json "$PRIV/lat-$IDX.json" \
    "$PRIV/exo-medicion search --db $PRIV/idx-$IDX.db --type hybrid --limit 5 --min-similarity 0.40 --json 'memoria persistente de sesiones'"
done
for IDX in base solape late; do
  [ -f "$PRIV/lat-$IDX.json" ] && python3 -c 'import json,sys; t=sorted(json.load(open(sys.argv[1]))["results"][0]["times"]); print(sys.argv[2], round(t[int(0.95*len(t))-1],4))' "$PRIV/lat-$IDX.json" "$IDX"
done
```
Expected: una línea `<índice> <p95 en s>` por índice. Correr los tres seguidos, sin otra carga pesada en la máquina.

`evals/retrieval-heldout/verdict/latencia.md`:
```markdown
# Latencia — campaña C (pre-registro §7, protocolo R3)

- Binario: `$PRIV/exo-medicion` (commit en `condiciones.md`); query fija `memoria persistente de sesiones`; hyperfine --warmup 3 --runs 30
| índice | p95 (s) | chunks_embedded (rebuild) | rebuild (s) | RSS pico rebuild |
|---|---|---|---|---|
| base | <p95> | <n> | <s> | <kB> |
| solape | <p95> | <n> | <s> | <kB> |
| late | <p95 o "no medido"> | <n> | <s> | <kB> |
- Guard R3: p95 solape ≤ 1,25 × p95 base → <sí/no>
- Guard R4: rebuild late ≤ 3 × rebuild base → <sí/no/no medido>
```
(Valores = salidas de los comandos; `rebuild (s)` con el mismo parseo de `Elapsed` que la Task 8 Step 6.)

- [ ] **Step 3: Informe held-out (decisorio)**

Con `N`=NETO mínimo y `X`=factor de D4, y `--estricto` solo si D5 = strict:
```bash
p95() { python3 -c 'import json,sys; t=sorted(json.load(open(sys.argv[1]))["results"][0]["times"]); print(t[int(0.95*len(t))-1])' "$1"; }
segs() { python3 -c 'import re,sys; m=re.search(r"Elapsed.*: (?:(\d+):)?(\d+):(\d+(?:\.\d+)?)", open(sys.argv[1]).read()); h,mi,s=m.groups(); print(int(h or 0)*3600+int(mi)*60+float(s))' "$1"; }
ARGS=(--captura base="$PRIV/cap-base.jsonl")
[ -f "$PRIV/cap-solape.jsonl" ] && ARGS+=(--captura solape="$PRIV/cap-solape.jsonl" --p95 base="$(p95 "$PRIV/lat-base.json")" --p95 solape="$(p95 "$PRIV/lat-solape.json")")
[ -f "$PRIV/cap-late.jsonl" ] && ARGS+=(--captura late="$PRIV/cap-late.jsonl" --rebuild-s base="$(segs "$PRIV/rebuild-base.time")" --rebuild-s late="$(segs "$PRIV/rebuild-late.time")")
python3 evals/retrieval-heldout/harness/metricas.py informe --gold "$PRIV/gold.jsonl" "${ARGS[@]}" --neto-min N --factor X --detalle "$PRIV/detalle.jsonl" > evals/retrieval-heldout/verdict/agregados.md
```
Expected: exit 0; el fichero contiene las secciones `pareadas` y `decisiones`. **Se corre una sola vez.** Si hay que re-correrlo por un error del harness (no por el resultado), se anota en ledger con el diff del harness y el motivo.

- [ ] **Step 4: Informe in-sample (descriptivo, sesgado a favor de A0)**

```bash
ARGS55=(--captura base="$PRIV/cap-base-55.jsonl")
[ -f "$PRIV/cap-solape-55.jsonl" ] && ARGS55+=(--captura solape="$PRIV/cap-solape-55.jsonl")
[ -f "$PRIV/cap-late-55.jsonl" ] && ARGS55+=(--captura late="$PRIV/cap-late-55.jsonl")
{ echo "> In-sample: A0 se eligió sobre estas 55. DESCRIPTIVO, no decide (pre-registro §6 R2). Líneas 'decisiones' de abajo NO aplican."; echo; \
  python3 evals/retrieval-heldout/harness/metricas.py informe --gold "$PRIV/in-sample-55.jsonl" "${ARGS55[@]}" --neto-min N --factor X --detalle "$PRIV/detalle-55.jsonl"; } > evals/retrieval-heldout/verdict/agregados-in-sample.md
```
Si D5 incluye `overlay-fila13=si`, añadir al mismo fichero una segunda sección con `--overlay "$PRIV/overlay-fila13.jsonl"` (mismo comando, `>>`, precedida de `## Con overlay fila 13 (H24)`).
Comprobar que ningún agregado filtra texto privado:
```bash
python3 -c 'import json,sys; qs=[json.loads(l)["query"] for f in sys.argv[2:] for l in open(f) if l.strip()]; t=open(sys.argv[1]).read()+open(sys.argv[1].replace("agregados.md","agregados-in-sample.md")).read(); bad=[q for q in qs if len(q)>12 and q in t]; print(len(bad)); sys.exit(1 if bad else 0)' evals/retrieval-heldout/verdict/agregados.md "$PRIV/gold.jsonl" "$PRIV/in-sample-55.jsonl"
```
Expected: `0` (el texto fijo de la cabecera no contiene queries).

- [ ] **Step 5: Commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add evals/retrieval-heldout/verdict/agregados.md evals/retrieval-heldout/verdict/agregados-in-sample.md evals/retrieval-heldout/verdict/latencia.md
git -C /home/paul/Documentos/proyectos/exo commit -m "eval(c): agregados held-out e in-sample, latencia"
```

---

### Task 10: Verdict adjudicado (regla de la cita)

**Lane:** diseño (adjudicación — fable, spec fábrica §6).
**Oráculo:** `evals/retrieval-heldout/verdict/c-verdict.md` existe, cada decisión R1–R4 lleva cita textual del pre-registro congelado, y `rg -c '^> ' evals/retrieval-heldout/verdict/c-verdict.md` ≥ 4.
**Depende de:** Task 9.

**Files:**
- Create: `evals/retrieval-heldout/verdict/c-verdict.md`
- Create (gitignored): `.superpowers/fabrica/verdicts/c-retrieval-heldout.md` (copia con el trail de adjudicación)

**Interfaces:**
- Consumes: pre-registro congelado, `condiciones.md`, `agregados.md`, `agregados-in-sample.md`, `latencia.md`, `gold-verificacion-resumen.md`; `$PRIV/detalle.jsonl` (el adjudicador puede leerlo para muestrear, no lo cita por fila).
- Produces: decisiones R1–R4 firmes que consumen las Tasks 11 y 12: `ADOPTAR RRF | SE QUEDA LA FUSIÓN ACTUAL`, `ADOPTAR SOLAPE | SE QUEDA EL TROCEADO ACTUAL`, `R4 …`, `R1 …`.

- [ ] **Step 1: Despachar el adjudicador**

Subagente **fable** fresco (no participó en Tasks 1–9). Brief literal:
"Adjudica la campaña C contra `docs/superpowers/plans/2026-09-13-campana-c-preregistro.md` (congelado en `<hash>`; verifica con `git diff <hash> -- <ruta>` vacío y `sha256sum $PRIV/gold.jsonl` == §10). Primero re-corre tú `metricas.py informe` con los argumentos de `agregados.md` y confirma que sale idéntico (`diff`). Muestrea 5 pareadas discordantes de `$PRIV/detalle.jsonl` y abre las notas del snapshot para confirmar que el hit/miss es real. Emite `evals/retrieval-heldout/verdict/c-verdict.md` con: (1) R1–R4, cada una con la cifra y una cita textual (bloque `> `) del §6 del pre-registro; (2) base de evidencia declarada por cifra: `measured / held-out / N=<n> / binario <commit> / snapshot <S>`, y `measured / in-sample (sesgado)` para las 55; (3) la tabla de potencia del §7 aplicada al N real: qué diferencia NO podía verse; (4) filas en rojo sin maquillar (brazos que pierden, estratos donde A0 cae); (5) qué hace la Task 12 (nada / RRF / solape / ambas) y qué va a spec aparte (late si R4 GANA). Sin cita textual del pre-registro = decisión inválida → PENDIENTE-PAUL. Ninguna query ni permalink por fila en el fichero (repo público)."

- [ ] **Step 2: Comprobar forma y privacidad**

```bash
rg -c '^> ' evals/retrieval-heldout/verdict/c-verdict.md
python3 -c 'import json,sys; qs=[json.loads(l)["query"] for f in sys.argv[2:] for l in open(f) if l.strip()]; t=open(sys.argv[1]).read(); print(sum(1 for q in qs if len(q)>12 and q in t))' evals/retrieval-heldout/verdict/c-verdict.md "$PRIV/gold.jsonl" "$PRIV/in-sample-55.jsonl"
```
Expected: ≥4 y `0`.

- [ ] **Step 3: Commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add evals/retrieval-heldout/verdict/c-verdict.md
git -C /home/paul/Documentos/proyectos/exo commit -m "eval(c): verdict de la campana C (held-out pre-registrado)"
```

---

### Task 11: Backlog, H24, N1 y cierre de la campaña (sin producción)

**Lane:** mecánica.
**Oráculo:** `rg -n 'campaña C|held-out' docs/backlog.md` muestra el item actualizado y `git -C /home/paul/Documentos/proyectos/exo diff --stat main -- engine/src/buscador.rs engine/src/trozos.rs` vacío en la rama de campaña.
**Depende de:** Task 10. (Si la campaña B tocó `docs/backlog.md`, rebase primero.)

**Files:**
- Modify: `docs/backlog.md` — item Alta "(revisión 2026-09-04) El 48/55 del hybrid es un resultado in-sample" (hoy `:99-135`); tabla `## Estado` fila **Medido** (hoy `:91`); añadir items en Media/Baja.

**Interfaces:**
- Consumes: `c-verdict.md`.
- Produces: backlog sincronizado; worktrees experimentales retirados (ramas conservadas).

- [ ] **Step 1: Actualizar el item Alta**

Añadir al final del item (tras el párrafo de «Calibración de la propia fuente…»), texto literal con los valores del verdict:
```markdown
  **Campaña C (2026-09-13 → <fecha verdict>), held-out pre-registrado:**
  acción (a) HECHA — gold privado de <n> queries no nulas (<estratos>),
  aprobado por Paul, sha256 en `docs/superpowers/plans/2026-09-13-campana-c-preregistro.md` §10;
  acción (b) HECHA — `evals/retrieval-heldout/verdict/c-verdict.md` reporta
  held-out y in-sample por separado con base declarada. Resultado: R1 <GENERALIZA|NO GENERALIZA|INDETERMINADO>
  (A0 held-out <x>/<n>, Wilson [<lo>, <hi>]); R2 <…>; R3 <…>; R4 <…>.
  Lo que el N no podía ver: <frase del verdict §3>. El held-out queda
  CONSUMIDO: volver a tocar β, umbral o troceado exige uno nuevo.
  Acción (c) (default de `exo search --type`, `main.rs:215`): sigue abierta,
  PENDIENTE-PAUL con A0 vs A2 held-out delante (D6 del plan de C).
```
Si R1–R4 dejan todo como está, marcar el checkbox del item como `[x]` **solo** para (a)+(b) y dejar (c) como sub-item abierto `- [ ] (c) …` debajo.

- [ ] **Step 2: Fila Medido de `## Estado`**

Sustituir la celda de **Medido** por:
`engine-hybrid **48/55** hit@5 in-sample (`evals/e1-read/verdict/m2-09-corrida.md`) · held-out **<x>/<n>** (campaña C, `evals/retrieval-heldout/verdict/c-verdict.md`) — no comparables entre sí: distinta fuente de queries`

- [ ] **Step 3: Items nuevos**

En `## Media` (literal):
```markdown
- [ ] **(campaña C, N1) `exo recall --query=<prompt>` es vector puro en la práctica.**
  `prepara_query` (`engine/src/buscador.rs:147-153`) une tokens con AND de
  FTS5: una frase natural da 0 candidatos FTS (reproducido 2026-09-13:
  `fabrica campaña` → 29, `cómo decidimos el umbral de similitud del recall y
  por qué quedó en 0.40` → 0), así que la fusión no actúa en el hook y la
  calidad del recall depende del arm vector y del troceado. Estrato `prompt`
  de la campaña C: <hit@5 A0 vs A1 del verdict>. **Acción:** decidir si el
  arm FTS del modo consulta debe usar OR/NEAR o extracción de términos, con
  su propio pre-registro (el held-out de C ya está consumido).
```
En `## Baja` (literal; H24 nunca había subido):
```markdown
- [ ] **(campaña C, H24) Gold single-label en `evals/retrieval-fase0/`.**
  `verdict/labels.md:72` pedía `acceptable_permalinks` empezando por la fila
  13. Campaña C: el held-out los admite desde el diseño; las 55 se reportan
  <con y sin overlay de la fila 13 | sin overlay> en
  `evals/retrieval-heldout/verdict/agregados-in-sample.md`; `gate.md` de M0
  intacto. Cerrar si no aparece un segundo caso.
```
Si R4 GANA, añadir en `## Media`: `- [ ] **(campaña C) Late chunking ganó en held-out: spec de producción** — caché por (hash de nota, span) en vez de texto (`engine/src/indexer.rs:318-347`), ventana y memoria; cifras en `c-verdict.md`.`
Si R1 = NO GENERALIZA, añadir en `## Alta`: `- [ ] **(campaña C) La fusión sellada no generaliza fuera de muestra** — <cifra A1/A2 vs A0>; PENDIENTE-PAUL: qué hacer, sin tunear sobre el held-out consumido.`

- [ ] **Step 4: Retirar worktrees (conservar ramas y privados)**

```bash
git -C /home/paul/Documentos/proyectos/exo worktree remove .worktrees/c-solape
[ -d /home/paul/Documentos/proyectos/exo/.worktrees/c-late ] && git -C /home/paul/Documentos/proyectos/exo worktree remove .worktrees/c-late
[ -d /home/paul/Documentos/proyectos/exo/.worktrees/c-medicion ] && git -C /home/paul/Documentos/proyectos/exo worktree remove .worktrees/c-medicion
git -C /home/paul/Documentos/proyectos/exo branch --list 'exp/c-*'
```
Expected: ramas `exp/c-solape` (y `exp/c-late`) siguen listadas. `$PRIV` no se borra (material de auditoría; backup fuera de alcance).

- [ ] **Step 5: Commit y package**

```bash
git -C /home/paul/Documentos/proyectos/exo add docs/backlog.md
git -C /home/paul/Documentos/proyectos/exo commit -m "docs(backlog): resultado de la campana C (held-out), N1 y H24"
```
Review-package `.superpowers/fabrica/packages/c-retrieval-heldout.md` con: resumen del verdict, comandos de oráculo y salida, **nota de hand-off a la campaña B** (el README y `docs/arquitectura.md` §6 siguen presentando el 48/55 sin la cifra held-out: el cambio de texto es de B), e instrucción de merge de `c-retrieval-heldout`. Si R2 y R3 no adoptan nada, **la campaña cierra aquí** y la Task 12 se marca `no aplica`.

---

### Task 12: Cambio de producción (CONDICIONAL: solo lo que el verdict adopte)

**Lane:** mecánica (el oráculo existe: la captura medida del brazo adoptado).
**Oráculo:**
- 12A (RRF): `python3 evals/retrieval-heldout/harness/metricas.py fidelidad --captura "$PRIV/cap-prod-rrf.jsonl" --brazo rrf` → 0 discrepancias, más `engine/scripts/test-hermetico.sh` en OK.
- 12B (solape): `python3 evals/retrieval-heldout/harness/metricas.py compara --a "$PRIV/cap-solape.jsonl" --b "$PRIV/cap-prod-solape.jsonl"` → 0 distintas, más `test-hermetico.sh` en OK.

**Depende de:** Task 10 con `ADOPTAR RRF` y/o `ADOPTAR SOLAPE`; **merge de A y de B** en `main` (conflictos en `buscador.rs`, `recall.rs` y `main.rs`, ver §Dependencias). Si el verdict no adopta nada: `no aplica`. Late chunking **nunca** entra aquí.
**Rama:** `c-produccion` desde `main`, separada de `c-retrieval-heldout`.

**Files (12A, RRF):**
- Modify: `engine/src/buscador.rs`: añadir `RRF_K` y `fusiona_rrf`; `busca_hybrid` pierde `bonus` y `escala_fts`; se borran `normaliza_fts` (`:350-372`), `fusiona` (`:374-414`) y los tests de `mod tests_fusion` que las ejercitan (`:480-601`), sustituidos por los de RRF.
- Modify: `engine/src/main.rs`: se borran `BONUS_SELLADO`/`ESCALA_FTS_SELLADA` con su doc (`:12-25`) y los campos `bonus`/`escala_fts` de `ArgsSearch` (`:222-233`); se actualizan las llamadas (`:850-860`, `:895-902`).
- Modify: `engine/src/recall.rs:474-497`: `recall_consulta` pierde `bonus` y `escala_fts`.
- Modify: `engine/tests/buscador.rs:272-279, 303, 326, 422, 454, 473`: `busca_hybrid(&db, q, limite, min_sim)`.
- Modify: `evals/retrieval-heldout/harness/captura.py`: flag `--fusion rrf` que no pasa `--bonus`/`--fts-scale`.

**Files (12B, solape):**
- Modify: `engine/src/trozos.rs:1-9, 96-106, 156-165`, con el mismo diff que la Task 7 (que no se mergea: se re-aplica con TDD sobre `main`).

**Interfaces:**
- Produces (12A): `fn fusiona_rrf(fts_ordenado: &[String], vector_ordenado: &[String], limite: usize) -> Vec<Resultado>` · `pub fn busca_hybrid(db_ruta: &Path, query: &str, limite: usize, min_similitud: Option<f64>) -> Result<Busqueda>` · `pub fn recall_consulta(db_ruta: &Path, query: &str, limite: usize, min_similitud: Option<f64>) -> Result<RecallBruto>`.
- Produces (12B): `trocea` con la misma firma y `SOLAPE_CHARS = 180`.
- Si A cambió estas firmas o movió líneas, se localizan por nombre de función. **Las firmas de arriba y los tests son lo contractual; los números de línea, no.**

#### 12A — RRF

- [ ] **Step A1: Tests que fallan (unitarios)**

En `engine/src/buscador.rs`, dentro de `mod tests_fusion`, añadir:
```rust
    fn lista(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|s| s.to_string()).collect()
    }

    /// RRF (Cormack, Clarke & Büttcher, SIGIR 2009), k = 60, rango 1-based;
    /// FTS se suma antes que vector (mismo orden que el harness de la campaña C).
    #[test]
    fn rrf_suma_inversos_de_rango_k60() {
        let r = fusiona_rrf(&lista(&["a", "b"]), &lista(&["b", "c"]), 10);
        let orden: Vec<&str> = r.iter().map(|x| x.permalink.as_str()).collect();
        assert_eq!(orden, vec!["b", "a", "c"]);
        assert_eq!(r[0].score, 1.0 / 62.0 + 1.0 / 61.0);
        assert_eq!(r[1].score, 1.0 / 61.0);
        assert_eq!(r[2].score, 1.0 / 62.0);
    }

    #[test]
    fn rrf_desempate_por_permalink_y_truncado_post_fusion() {
        let r = fusiona_rrf(&lista(&["z"]), &lista(&["y"]), 1);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].permalink, "y");
    }

    #[test]
    fn rrf_con_una_sola_lista_conserva_su_orden() {
        let r = fusiona_rrf(&[], &lista(&["x", "y", "w"]), 10);
        let orden: Vec<&str> = r.iter().map(|x| x.permalink.as_str()).collect();
        assert_eq!(orden, vec!["x", "y", "w"]);
    }
```
Run: `cargo test --release --lib tests_fusion --manifest-path /home/paul/Documentos/proyectos/exo/engine/Cargo.toml`
Expected: FAIL de compilación (`cannot find function fusiona_rrf`).

- [ ] **Step A2: Implementación**

En `engine/src/buscador.rs`, sustituir `normaliza_fts` y `fusiona` (con sus doc-comments) por:
```rust
/// Constante de Reciprocal Rank Fusion (Cormack, Clarke & Büttcher, SIGIR
/// 2009). Adoptada por la campaña C sobre held-out pre-registrado
/// (`evals/retrieval-heldout/verdict/c-verdict.md`), sin afinar: el valor del
/// paper.
const RRF_K: f64 = 60.0;

/// Fusión RRF de dos rankings ya ordenados (FTS por `-bm25` desc, vector por
/// similitud desc con umbral pre-fusión): `score(e) = Σ 1/(RRF_K + rango)`,
/// rango 1-based, lista ausente = 0. Libre de escala: no normaliza BM25 ni
/// pondera canales. Orden por score desc, desempate por permalink asc (M2-09a),
/// truncado a `limite` DESPUÉS de fusionar.
fn fusiona_rrf(fts_ordenado: &[String], vector_ordenado: &[String], limite: usize) -> Vec<Resultado> {
    let mut puntos: HashMap<String, f64> = HashMap::new();
    for lista in [fts_ordenado, vector_ordenado] {
        for (i, permalink) in lista.iter().enumerate() {
            *puntos.entry(permalink.clone()).or_insert(0.0) += 1.0 / (RRF_K + (i + 1) as f64);
        }
    }
    let mut resultados: Vec<Resultado> = puntos
        .into_iter()
        .map(|(permalink, score)| Resultado {
            permalink,
            tipo: "entity".to_string(),
            score,
            ruta: None,
        })
        .collect();
    resultados.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.permalink.cmp(&b.permalink))
    });
    resultados.truncate(limite);
    resultados
}
```
En `busca_hybrid`, la firma pasa a `pub fn busca_hybrid(db_ruta: &Path, query: &str, limite: usize, min_similitud: Option<f64>) -> Result<Busqueda>`, y el cuerpo entre las dos búsquedas y la fusión queda así:
```rust
    const K_C: usize = 50;
    let fts = busca(db_ruta, query, K_C)?;
    let fts_ordenado: Vec<String> = fts.results.into_iter().map(|r| r.permalink).collect();

    let vector = busca_vector(db_ruta, query, usize::MAX, min_similitud)?;
    let avisos = vector.avisos;
    let vector_ordenado: Vec<String> = vector.results.into_iter().map(|r| r.permalink).collect();

    let mut results = fusiona_rrf(&fts_ordenado, &vector_ordenado, limite);
```
(El resto de la función, `enriquece_rutas` y la construcción de `Busqueda`, queda como A lo haya dejado.) Se borran de `mod tests_fusion` los tests que llaman a `fusiona` o a `normaliza_fts`: `fusion_formula_ambos_canales`, `fusion_conserva_candidato_solo_vector`, `fusion_conserva_candidato_solo_fts`, `fusion_clave_entidad_una_fila_por_permalink`, `normalizacion_bm25_monotona`, `normalizacion_bm25_query_sin_fmax`, `fusion_bonus_cero_es_max`, `fusion_orden_desc_truncado_post_fusion` y `fusion_desempate_determinista_por_permalink`. La helper `mapa` se borra si queda sin uso.

En `engine/src/main.rs`:
- Borrar las líneas `:12-25` (doc y constantes).
- Borrar de `ArgsSearch` los campos `bonus` y `escala_fts` con sus `#[arg]` y su doc.
- En `busca_cmd`, la rama pasa a `TipoBusqueda::Hybrid => busca_hybrid(&db, &args.query, args.limite, args.min_similitud)?,`.
- En el modo consulta de recall: `recall_consulta(&db, q, args.limite, args.min_similitud)?`.

En `engine/src/recall.rs`, `recall_consulta`:
- Se quitan los parámetros `bonus: f64, escala_fts: f64`.
- La llamada pasa a `crate::buscador::busca_hybrid(db_ruta, query, limite, min_similitud)?`.
- El doc-comment de `:474-482` cambia "con los defaults sellados (`bonus`/`escala_fts` …)" por "con la fusión RRF de `busca_hybrid`".

En `engine/tests/buscador.rs` hay que quitar los dos últimos argumentos de cada `busca_hybrid(...)`: la llamada multilínea de `:272-279` y las de `:303`, `:326`, `:422`, `:454` y `:473`. El test `threshold_filtra_vector_pre_fusion` (`:297-314`) conserva su aserción sin cambios, porque RRF también mantiene el candidato solo-FTS. Solo cambia su doc-comment: "debe sobrevivir con `score == f`" pasa a "debe sobrevivir (RRF: candidato solo-FTS con score `1/(60+rango)`)".

- [ ] **Step A3: Verde, fmt, clippy y gate hermético**

```bash
cargo fmt --manifest-path /home/paul/Documentos/proyectos/exo/engine/Cargo.toml
cargo clippy --all-targets --locked --manifest-path /home/paul/Documentos/proyectos/exo/engine/Cargo.toml -- -D warnings
cargo test --release --lib tests_fusion --manifest-path /home/paul/Documentos/proyectos/exo/engine/Cargo.toml
/home/paul/Documentos/proyectos/exo/engine/scripts/test-hermetico.sh
```
Expected: clippy sin warnings, `tests_fusion` en PASS y `test-hermetico: OK`.

- [ ] **Step A4: Oráculo, que la producción reproduzca el brazo medido**

En `evals/retrieval-heldout/harness/captura.py`:
- añadir `ap.add_argument("--fusion", default="sellada", choices=["sellada", "rrf"])`;
- sustituir la línea `hyb = ...` por:
```python
            if a.fusion == "rrf":
                hyb = replay_engine.search(a.exo, a.db, q, 10, "hybrid", min_similitud=0.40)
            else:
                hyb = replay_engine.search(a.exo, a.db, q, 10, "hybrid", min_similitud=0.40, bonus=0.0, escala_fts=0.6)
```
Luego:
```bash
cargo build --release --locked --manifest-path /home/paul/Documentos/proyectos/exo/engine/Cargo.toml
python3 evals/retrieval-heldout/harness/captura.py --fusion rrf --gold "$PRIV/gold.jsonl" --db "$PRIV/idx-base.db" --exo /home/paul/Documentos/proyectos/exo/engine/target/release/exo --out "$PRIV/cap-prod-rrf.jsonl"
python3 evals/retrieval-heldout/harness/metricas.py fidelidad --captura "$PRIV/cap-prod-rrf.jsonl" --brazo rrf
```
Expected: `fidelidad(rrf): 0 discrepancias de <n>`. Si hay discrepancias, la producción no es el brazo medido: STOP (retries cap 2).
(`idx-base.db` se construyó con el binario post-A. Si 12A cambia el schema, cosa que no debería, se reconstruye con el binario nuevo y se recaptura `base` antes.)

- [ ] **Step A5: Commit**

```bash
git -C /home/paul/Documentos/proyectos/exo switch -c c-produccion main
git -C /home/paul/Documentos/proyectos/exo add engine/src/buscador.rs engine/src/main.rs engine/src/recall.rs engine/tests/buscador.rs evals/retrieval-heldout/harness/captura.py
git -C /home/paul/Documentos/proyectos/exo commit -m "feat(search): fusion hybrid por RRF k=60 (campana C, held-out pre-registrado); retira --bonus/--fts-scale"
```
El review-package de producción tiene que declarar: **cambio de CLI incompatible** (`--bonus` y `--fts-scale` desaparecen; clap sale con 2 si se pasan); la escala de `score` del hybrid pasa a ~0,016–0,033, que es informativa y no contractual (`buscador.rs:19`); y el hand-off a B para `docs/arquitectura.md:222-223,289` y el README.

#### 12B — Solape

- [ ] **Step B1: Test que falla**

En `engine/src/trozos.rs`, sustituir el test `bloque_que_excede_900_se_corta_duro_sin_solape` por los dos tests literales de la Task 7 Step 2 (`bloque_que_excede_900_se_corta_con_solape_de_180` y `corte_duro_no_emite_trozo_final_contenido_en_el_anterior`).
Run: `cargo test --release --lib trozos --manifest-path /home/paul/Documentos/proyectos/exo/engine/Cargo.toml`
Expected: FAIL (`left: 100, right: 280`).

- [ ] **Step B2: Implementación**

Aplicar el código literal de la Task 7 Step 3 (cabecera con `SOLAPE_CHARS` y `corta_duro` con paso). Cambiar la cabecera del módulo: donde dice "RAMA EXPERIMENTAL `exp/c-solape` … NO se mergea", poner "Adoptado por la campaña C (`evals/retrieval-heldout/verdict/c-verdict.md`)".

- [ ] **Step B3: Verde y gate**

```bash
cargo fmt --manifest-path /home/paul/Documentos/proyectos/exo/engine/Cargo.toml
cargo clippy --all-targets --locked --manifest-path /home/paul/Documentos/proyectos/exo/engine/Cargo.toml -- -D warnings
/home/paul/Documentos/proyectos/exo/engine/scripts/test-hermetico.sh
```
Expected: `test-hermetico: OK`. Si cae un test de `tests/cache_embeddings.rs` o `tests/indexer.rs` que fijaba el número de trozos de un fixture con bloques de más de 900 caracteres, se actualiza el número esperado con el cálculo de paso 720, y el review-package lista cada test tocado con su valor viejo y nuevo.

- [ ] **Step B4: Oráculo, que el índice de producción sea el índice medido**

```bash
cargo build --release --locked --manifest-path /home/paul/Documentos/proyectos/exo/engine/Cargo.toml
/home/paul/Documentos/proyectos/exo/engine/target/release/exo rebuild --db "$PRIV/idx-prod-solape.db" --kb "$PRIV/kb-snap" --json > "$PRIV/rebuild-prod-solape.json"
python3 evals/retrieval-heldout/harness/captura.py --gold "$PRIV/gold.jsonl" --db "$PRIV/idx-prod-solape.db" --exo "$PRIV/exo-medicion" --out "$PRIV/cap-prod-solape.jsonl"
python3 evals/retrieval-heldout/harness/metricas.py compara --a "$PRIV/cap-solape.jsonl" --b "$PRIV/cap-prod-solape.jsonl"
```
Expected: `compara: 0 queries con capturas distintas`. La captura usa `exo-medicion` a propósito, para que solo varíe el índice.

- [ ] **Step B5: Commit y paso de Paul (línea roja: índice real)**

```bash
git -C /home/paul/Documentos/proyectos/exo add engine/src/trozos.rs
git -C /home/paul/Documentos/proyectos/exo commit -m "feat(index): solape de 180 chars en cortes duros (campana C, held-out pre-registrado)"
```
Hay que añadir a `pendiente-paul.md` y al package, literal: "Tras mergear, **`exo rebuild`** en cada máquina (Linux y W11). El indexado incremental decide por `mtime` (`engine/src/indexer.rs`) y no re-trocea las notas que no cambiaron. Sin rebuild, el índice queda mezclando trozos viejos y nuevos sin avisar, y el hook `--refresh` no lo arregla. Duración medida del rebuild con solape: `<Elapsed de rebuild-solape.time>`." La fábrica no ejecuta ese rebuild: `~/.exo/index.db` es línea roja.

---

## Self-review (checklist contra el brief)

- **Re-verificación:** hecha. Ningún hallazgo sale entero y hay 4 correcciones, anotadas en §Hallazgos. N1 es nuevo y va al backlog en la Task 11.
- **Pre-registro primero:** el documento hermano define métricas (hit@5 primaria, MRR@10 secundaria y por qué no nDCG), tamaño con potencia exacta, regla GANA con características operativas, qué pasa si nada gana y circuit breakers. La congelación por commit (Task 5) va antes de la primera captura (Task 6).
- **Nada de afinar sobre el held-out:** los brazos y parámetros son fijos (pre-registro §4). R3 y R4 no eligen celda. El held-out queda consumido.
- **Producción solo si pasa el gate:** la Task 12 es condicional, final y en rama propia. Late chunking nunca entra.
- **Held-out:** fuentes, filtros anti-fuga, etiquetador filesystem-only distinto del autor, verificación adversarial y aprobación de Paul como gate que bloquea la medición (Task 5).
- **Global Constraints** comprobados en código: `engine/Cargo.toml:4,8`, `test-hermetico.sh`, `lib.rs:167-168,255`, `envelope.rs` y `ci.yml`.
- **Harness:** reutiliza `analyze.norm` y `replay-engine.search` sin modificarlos. Código de las Tasks 1–3 verificado en scratch el 2026-09-13: 26 tests en OK, `fidelidad` con 0 discrepancias contra el binario instalado en 3 queries (FTS de 29, 6 y 0 candidatos) e `informe` sobre fixtures sin errores.
- **Decisiones de Paul:** D0–D6 con opciones y recomendación.
- **Lane y oráculo:** declarados en cada tarea.
- **Dependencias con A y B:** tabla en §Dependencias.
- **Placeholders:** los `<…>` que quedan son valores de salida de comandos de pasos anteriores (recuentos, hashes, cifras del verdict), nunca decisiones ni código.

## Handoff

Ejecución con `exo:orchestrate`, bajo el protocolo de `paul-profile:fabrica`.
Orden: Task 0 → (1 ∥ 2) → 3 → 4 → 5 → **[merge de A]** → 6 → (7 ∥ 8) → 9 → 10 → 11 → **[merge de A y B]** → 12 (condicional).
