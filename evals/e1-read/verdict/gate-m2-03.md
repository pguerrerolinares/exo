# Verdict — gate m2-03 (cierra el par M2-02+M2-03, D2)

- **Adjudicador**: consultor fable FRESCO (régimen delegado, config §Ejecución de gates). No participé en ninguna fase de lo juzgado ni en el gate anterior de la campaña; no recibí razonamiento del orquestador.
- **Fecha**: 2026-07-17 · **Rama**: `m2-03` (HEAD `1e63138`, base main `580eef0`)
- **Adjudicación**: **MERGED**

## Verificación primaria propia (no delegada)

Re-corrí los oráculos yo mismo en el worktree, sin fiarme del reporte:

1. `cargo test --manifest-path engine/Cargo.toml` → **21 passed, 0 failed, 1 ignored** (el ignorado es `jina_es_embebe_a_768`, deferred declarado de m2-01). Los 4 tests contractuales de spec §1.1 presentes con nombre exacto y aserciones reales: `permalink_del_frontmatter_se_honra`, `recencia_viene_de_git`, `walker_excluye_dotdirs`, `walker_solo_markdown`.
2. `cargo build --release` + `exo rebuild --db /tmp/gate-m2-03/exo.db --json` sobre la KB real → `{"command":"rebuild","data":{"borradas":0,"indexadas":115,"saltadas":0},"schema_version":1}`, exit 0.
3. `corpus-parity.py --diff` → `gold=115 engine=115 faltan=0 sobran=0`, exit 0.
4. Idempotencia: 2º rebuild + 2º diff → mismo resultado, ∅, exit 0.
5. `exo index` incremental sobre DB poblada → `saltadas=115` (skip por mtime operativo).
6. Diff completo `main..m2-03` inspeccionado línea a línea (15 ficheros, solo `engine/` + plan; `evals/` con **cero** líneas de diff).

## Puntos adjudicados

- **DDL §2 verbatim**: `engine/src/schema.rs` comparado literal contra spec §2 — columnas, tipos, constraints, tokenizer `unicode61 tokenchars 0x2F` y `vec0(embedding float[768])` idénticos; única desviación el `IF NOT EXISTS` autorizado por el plan (T1) y el comentario de rowid movido a doc-comment. **Conforme.**
- **mtime/git_epoch (§6.2 regla 2)**: en `indexer.rs`, `mtime` aparece SOLO en la comparación de skip (`existentes.get(&ruta_rel) == Some(&mtime)`); la recencia es `git_epoch` vía `git log -1 --format=%ct`, refrescada en cada reindex, NULL tolerado para ficheros sin commit. mtime no toca recencia en ningún punto del diff. **Conforme.**
- **Skip-sin-permalink (§6.2 regla 1)**: `parsea_nota` devuelve `Ok(None)` para frontmatter ausente, YAML ilegible o `permalink:` ausente; el indexer avisa por stderr y sigue; no existe ningún camino que genere permalink. La paridad ∅ (faltan=0) confirma que nada que bm indexa fue saltado. **Conforme.**
- **Circuit breaker gold**: `evals/e1-read/gold/corpus-bm.json` INTOCADO — 0 líneas de diff en `evals/`, ningún commit de la rama toca `gold/`. **No dispara.**
- **Veto AGPL**: basic-memory es Python; el diff es Rust original con nombres propios en castellano (`notas`/`aristas`/`trozos` vs `entity`/`relation`/`search_index`), estructura propia, cero calco. Acceso a `memory.db`/`config.json` solo RO (permitido). **Conforme.**
- **yaml_serde 0.10.4**: spot-check propio contra crates.io API — repo `github.com/yaml/yaml-serde` (org oficial YAML), descripción "serde_yaml maintained by The YAML Organization", publicado por Ingy döt Net (co-autor del spec YAML), ~790k descargas. Sucesor legítimo del serde_yaml deprecado; el plan autorizaba explícitamente el fork mantenido verificado live. **Aceptable.**
- **Deferred m2-01**: `image-models` fuera de features de fastembed (Cargo.lock adelgaza ~600 líneas de deps de imagen); registro de sqlite-vec envuelto en `std::sync::Once` (`registra_vec()`), compartido por `abre_db_en_memoria` y el nuevo `abre_db`. Build y smoke tests verdes → sin efectos colaterales. **Hechos.**
- **Config §5 / D6**: `--db` obligatorio sin default; `--kb` flag > `kb_desde_config()` (RO, error explícito sin fallback si falta config o clave); E1 no escribe nada fuera del fichero `--db`. **Conforme.**
- **Envelope §4**: una línea JSON newline-terminada a stdout con `schema_version:1`, humano/warnings a stderr, gating por exit code — verificado empíricamente en mis corridas. **Conforme.**

## Mandato de disenso — qué busqué para objetar

Busqué activamente motivos de rechazo; lo que encontré y por qué no bloquea:

1. **Cambio de permalink a misma ruta**: si el frontmatter de una nota cambia su `permalink`, el upsert `ON CONFLICT(permalink)` chocaría con el `UNIQUE(ruta)` de la fila vieja → `indexa` devuelve error (exit ≠0). No es corrupción silenciosa (falla duro y `rebuild` lo resuelve), no ocurre en el corpus real (bm persiste permalinks estables) y es pariente de la colisión de PK que la review opus ya difirió a M4. **Observación, no bloquea.**
2. **La cascada de borrado no toca `vectores`**: el borrado de ausentes limpia `notas_fts`/`aristas`/`trozos` pero no `vectores` (rowid = trozos.id). Hoy es no-op (vectores vacía hasta M2-06), pero la decisión declarada #4 del executor ("borrado completo para no dejar bomba de FKs") es incompleta en ese punto. **Debe recogerse en M2-04/M2-06** cuando se pueble; no bloquea E1.
3. **Notas sin permalink se re-parsean en cada corrida** (nunca entran a `notas`, el skip por mtime no les aplica) y no cuentan en ningún contador del Resumen. Nit de perf/observabilidad; en la KB real no existe el caso (115/115 con permalink). **No bloquea.**
4. **El walker desciende a dotdirs no listados** (p.ej. `.git/`): la spec §6.2 regla 3 fija exactamente 3 exclusiones y la implementación las sigue literal; `.git` no contiene `.md` y la paridad ∅ lo confirma empíricamente. Spec-conformante. **No bloquea.**
5. **Orden de claves del envelope** alfabético (`command,data,schema_version`) vs el orden del ejemplo de spec §4: el orden de claves JSON no es contractual y los consumidores parsean, no comparan strings. **No bloquea.**
6. Busqué también: aserciones vacías o tests renombrados (no hay; los 4 nombres fijados intactos y con aserciones reales), `^` accidental en sqlite-vec (sigue `=0.1.9`), escritura fuera de `--db` (no hay), mtime usado como recencia (no hay), calcos AGPL (no hay), y modificación del gold (no hay).

## Resultado

Las 4 condiciones de validez del régimen cumplidas (fresco, verificación primaria propia, disenso declarado, verdict-artifact commiteado antes del merge). Oráculo de la fila M2-03 de spec M2 §3 satisfecho: **diff de paridad = ∅ + rebuild idempotente**. **GATE: MERGED.**
