# Campaña J — fase 1: T0 diagnóstico, pre-registro borrador y gold agéntico Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: usa `exo:orchestrate`
> para ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`)
> para tracking.

**Goal:** dejar a la campaña J lista para medir sin haber medido nada: un
diagnóstico descriptivo por fila de las 55 in-sample (T0), un pre-registro en
borrador con brazos, regla de decisión y potencia calculada (T1), y un gold
nuevo **100 % agéntico con fiabilidad medida** (decisión de Paul del
2026-09-19: revoca D-J1/D-J2; la KB la consumen agentes, así que el patrón de
relevancia es la utilidad para el agente, no el juicio de Paul), etiquetado
a ciegas por dos jueces de dos familias de modelo (fable y Kimi/Moonshot) y
aceptado solo donde coinciden. La fase termina con el commit de congelación
del pre-registro, que dispara el propio pipeline cuando el acuerdo supera el
suelo pre-registrado, y con el gate de un consultor fable. **0 h de Paul.**

**Architecture:** todo vive bajo `evals/retrieval-heldout/` y en el directorio
privado `$PRIV_J`; `engine/src` no se toca. El T0 cruza tres estados del mismo
conjunto de 55 queries (histórico 49/55, snapshot de C 43/55, producción hoy)
usando las capturas privadas de C y una captura nueva de solo lectura. El
gold se construye en cadena: queries por estrato (`prompt` real, `agent-search`
real minado y limpiado, `hard`/`archive`/`negativo` generados por un agente
fresco con checks mecánicos) → candidatos por otro agente filesystem-only →
paquetes idénticos para los dos jueces → juicio ciego e independiente (Kimi
por API con structured output estricto; fable como subagente) → acuerdo
(`acuerdo.py`: κ de Cohen, descarte de desacuerdos, auditoría del sesgo
léxico) → si pasa el suelo, congelación. El harness de C se extiende en dos
puntos (estratos nuevos y listas de exclusión repetibles). El criterio de
decisión vive en `docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md`.

**Tech Stack:**
- Python 3.12, solo stdlib (`unittest`, `json`, `sqlite3` con FTS5, `re`,
  `math`, `urllib`), igual que `evals/retrieval-heldout/harness/`.
- API Moonshot (OpenAI-compatible) en `https://api.moonshot.ai/v1` (la de
  `.ai`, no la de `.cn`); `response_format: json_schema` estricto (precedente:
  `wisdom-ai-news/src/wisdom/llm/clients.py:143-160`, que usa el SDK `openai`;
  aquí `urllib` para no añadir dependencias a `evals/`).
- Rust 2024, crate `exo` en `engine/` (solo `cargo build` para la captura de
  lectura; ningún cambio de código).
- `git clone` para el snapshot de la KB.

## Hallazgos de la planificación (2026-09-19, verificados; mandan sobre el esbozo)

- **H28 confirmado por código.** `distance_l2_sqr_float` devuelve
  `sqrt(res)` (`~/.cargo/registry/src/index.crates.io-*/sqlite-vec-0.1.9/sqlite-vec.c:389`,
  rutas SIMD `:224`, `:263`) y es la función del KNN de vec0 (`:6879`). El
  score vector del binario es `1 − √(2−2·cos)/2`, monótono en coseno; 0,40 ⇔
  cos 0,28. No es un brazo (borrador §2.2): todos los brazos usan la escala
  que el binario emite.
- **Todos los brazos del esbozo se calculan offline.** FTS5 (`OR`, `NEAR`,
  `bm25`) funciona desde el `sqlite3` de Python 3.12 sobre una copia del
  índice; `vec0` no (`no such module: vec0`), pero la lista vector ya se
  captura con `--limit 1000 --min-similarity 0.0`. CombSUM, penalización de
  `archive/`, abstención y FTS selectivo son funciones de las listas
  capturadas. **Las Tasks 3–6 del esbozo (flags de barrido en `engine/src`)
  desaparecen**: `engine/src` solo se toca en la fase 2 para el ganador.
- **T0 en dry-run (in-sample, descriptivo):** hist→S es **hit→miss 7,
  miss→hit 1, miss→miss 5** (el «6 de 55» es neto). De los 12 misses en S:
  11 `vector-lejos`, 6 con una rotación de la nota esperada en el top-5, 7
  de las 11 filas que la auditoría del 09-04 mandó re-verificar. 17/55 queries
  tienen FTS vacío con AND; 52/55 tendrían ≥1 token raro (df ≤ 25 % de las
  notas). Números que la Task 2 reproduce con el fichero público.
- **Pool de queries:** 113 `prompt` desde el 2026-09-13 (tras excluir las 55
  y las 147 de C). `agent-search`: el `reflex-retrieval-log.jsonl` termina el
  2026-08-17, pero Paul autorizó minar `~/.claude/projects/**/*.jsonl`; la
  extracción cruda está en `~/.exo/priv-j/agent-search-raw.txt` (221 líneas
  únicas, sin timestamps, mezcladas con ejemplos de docs/planes/tests: 155
  tienen forma de comando con flags, 66 llevan marcadores `*)`, `[OP`, `<qu`,
  `…`). La Task 6 la limpia con criterio escrito (`limpia_agent_search.py`) y
  cuenta; cota esperada 60–120.
- **`archive/` en C:** 24/92 esperadas y 7 aceptables vivían en `archive/`;
  274/723 plazas del top-5. Excluir rompería una de cada cuatro filas;
  penalizar es un trade-off medible, no obvio.
- **Jueces LLM, lo que se sabe:** los jueces LLM de relevancia son
  sistemáticamente engañables por coincidencia léxica (Alaofi et al., SIGIR
  2024, «LLMs can be fooled into labelling a document as relevant» — de
  memoria) y su acuerdo con humanos no sustituye al juicio humano en
  colecciones de evaluación (Clarke & Dietz 2024; Soboroff 2024 — de
  memoria; la Task 5 verifica las cuatro). Aquí no hay humano: por eso dos
  familias de modelo, acuerdo como condición de entrada, y auditoría del
  solape léxico (borrador §3).
- **El esbozo pedía «6 filas explicadas con permalinks» en un fichero del
  repo.** El repo es público (C §8): los permalinks y el texto van al detalle
  privado; el fichero público lleva ids `mNN`, rangos y causas.
- **Etiquetas muertas hoy: 0/55** (todos los `expected_permalink` de las 55
  existen en la KB HEAD `3c5e034`).

## Global Constraints

- **`engine/src` NO se toca en esta fase.** Ningún fichero bajo
  `engine/src/` ni `engine/tests/` aparece en un `git add` de este plan. El
  binario se compila solo para capturas de lectura.
- **Ninguna medición sobre el gold de J antes del commit de congelación
  (Task 8).** Ninguna tarea computa hit@k de ningún brazo sobre ninguna query
  del gold nuevo. El T0 usa solo las 55 in-sample (ya vistas por todos).
- **Envío de datos a Moonshot (Kimi), autorizado explícitamente por Paul el
  2026-09-19 en sesión** (`docs/superpowers/consultas/2026-09-15-campanas/propuesta.md`
  §7; frase literal: «no me importa mandar a kimi, continua por ahí»). Qué
  sale: por fila, la query (prompt de Paul o comando de agente) y hasta 5
  notas candidatas de la KB `wisdom-paul` recortadas a 4.000 caracteres.
  Qué no sale: ninguna etiqueta, ningún ranking, nada de `evals/` ni del
  gold de C. Solo la Task 7 llama a la API.
- **La key de Moonshot** está en
  `/home/paul/Documentos/proyectos/wisdom-ai-news/.env-keys` (variable
  `kimi_api_key`; el fichero es gitignored en ese repo). Se lee en runtime
  con `juez.py --env-keys`; **nunca** se imprime, se copia, se exporta a un
  fichero ni entra en el repo exo (público). Antes de cada commit bajo
  `evals/`: `LC_ALL=C git grep --untracked -cE 'sk-[A-Za-z0-9]{20,}' -- evals`
  = sin salida (exit 1). **Enmienda 2026-09-20:** el patrón genérico
  `'sk-'` nace en rojo hoy mismo — tres falsos positivos preexistentes en
  `evals/prep-m3/` (p.ej. «task-specific» contiene el substring `sk-`) — y
  un gate que nace en rojo se normaliza y deja de proteger. Este patrón
  exige forma de key de Moonshot (`sk-` + ≥20 caracteres alfanuméricos)
  aplicado sobre TODO `evals/`, sin acotar el directorio (acotar reduciría
  el alcance de la línea roja en vez de arreglar el falso positivo);
  `--untracked` para que el gate vea también ficheros nuevos antes del
  primer `git add`.
- **Régimen de fábrica vigente:** `.superpowers/fabrica/config.md`
  §ACTUALIZACIÓN 2026-09-19: «J fase 1 — T0 diagnóstico + pre-registro
  borrador + kit de gold … evals; **no toca `engine/src`**» y «J no congela
  su pre-registro hasta que exista el gold»; el gold ahora lo produce la
  fábrica (revocación de D-J1/D-J2, mismo día).
- **Repo público (desde 2026-09-02).** Ningún texto de query, prompt ni
  permalink por fila entra en git. Datos privados en
  `PRIV_J=~/.local/share/exo-evals/j-heldout` (`chmod 700`); se lee
  `PRIV_C=~/.local/share/exo-evals/c-heldout` (in-sample y capturas de C).
  Comprobación obligatoria antes de cada commit de un `.md` bajo `verdict/` o
  `gold-j/`: `grep -c "wisdom-paul" <fichero>` = 0.
- **Líneas rojas:** no se escribe en `~/.exo/index.db` (solo `exo search`);
  no se escribe en `~/Documentos/proyectos/wisdom-paul` (el snapshot es un
  `git clone` a `$PRIV_J`); nada de push, merge ni commit a `main`; `git add`
  con rutas explícitas, nunca `-A`; `git -C <path>`, nunca `cd <path> &&
  git`. Commits terminan con las líneas de atribución de la sesión.
- **Jueces ciegos:** ningún juez ve la etiqueta del otro, ningún ranking de
  ningún brazo, `evals/`, el gold de C ni el snapshot fuera del paquete;
  ambos reciben exactamente el mismo `paquetes.jsonl`. Los generadores de
  queries y de candidatos no ven las 55 ni el gold de C.
- **Harness: Python stdlib**, sin modificar `evals/retrieval-fase0/` ni
  `metricas.py`, `captura.py`, ni los ficheros de `verdict/` de C.
- **Tests:** `python3 -m unittest evals/retrieval-heldout/harness/test_harness.py`
  (hoy 31; al final 40) y `python3 -m unittest evals/retrieval-heldout/harness/test_gold_j.py`
  (7), desde la raíz del repo.
- **Rama de campaña:** `campana-j-fase1`, creada desde `main` una vez
  mergeada `plan-campanas-i-l-j` (este plan y el borrador). Worktree
  `.worktrees/campana-j-fase1`. Todos los comandos `python3` se corren con
  cwd en la raíz del worktree.
- **Presupuesto externo:** Kimi ≈ **$6 con `kimi-k3`** (≈ 280 llamadas ×
  ≈ 5–6 k tokens de entrada; $3 / $15 por millón, precio de lista web
  2026-09; con `kimi-k2.6`, a $0,95 / $4,00, habrían sido ≈ $2 — Paul eligió
  k3 el 2026-09-19 por calidad de juez); **tope $10**, por encima STOP. Dispatches: fable
  ≈ 12 lotes de juez + 1 review + 1 consultor-gate; sonnet ≈ 1 generador +
  ≈ 5 lotes de candidatos; opus 0.

## Decisiones firmadas por Paul 2026-09-19

Fuente de las firmas: `docs/superpowers/consultas/2026-09-15-campanas/propuesta.md`
§7 (sesión del 2026-09-19). **D-J1 y D-J2 quedaron revocadas el mismo día**
por la decisión del gold agéntico (D-J11): Paul no etiqueta ni revisa nada.
Las demás coinciden con la recomendación del planificador; las opciones
descartadas se conservan como traza del trade-off. La Task 8 copia las
firmas literales al borrador §9 (ya las lleva) y solo espera el gold y su
sha256.

| decisión | firmado | descartado (traza) |
|---|---|---|
| D-J1 modo de etiquetado | **REVOCADA** por D-J11 (era b: agente pre-etiqueta, Paul revisa) | a, b, c: todas exigían horas de Paul |
| D-J2 estrato `prompt` | **REVOCADA** por D-J11 (era 113); D-J11 lo fija en el pool entero | 60 |
| D-J3 verificación adversarial del gold | **sí** — absorbida por el diseño: dos jueces independientes + consultor-gate (Task 8 Step 4) | no |
| D-J4 regla GANA | **NETO ≥ 4** (∧ ARREGLA ≥ 2·ROMPE ∧ veto ΔMRR) | 3 (cota familiar 0,41) · 5 (potencia +5 pp 0,42–0,55) |
| D-J5 H28 | **(a)** corrección de nombre y doc, escala `s` se queda, sin brazo | (b) celda descriptiva `sellado-cos` |
| D-J6 int8 | **fuera** | dentro como celda descriptiva si K = no daemon |
| D-J7 factor P1 | **0,90** | otro valor único |
| D-J8 tope df F1 | **0,25**, con la cláusula de T0 intacta | otro valor único |
| D-J9 G1 | **tal cual**: p70 de calibración; adopta si abstiene ≥ 60 % de evaluación y ROMPE ≤ 2 | otros valores únicos |
| D-J10 relevancia | **lenient decide**, strict descriptivo | strict decide |
| **D-J11 gold agéntico** | **firmado**: queries reales de agentes (`agent-search` minado con autorización) + `prompt` + generados; jueces ciegos **fable y Kimi (Moonshot)**; entra solo con acuerdo; **desacuerdos descartados** (sin tercer juez); suelo **κ ≥ 0,60 ∧ p_o ≥ 0,70**, por debajo J PARA; envío de trozos de la KB a Moonshot autorizado | tercer juez opus para desempatar (reintroduce la correlación Claude–Claude que el diseño existe para romper); gold etiquetado por Paul (≈ 3,5–5 h) |

Trade-offs que se presentaron, en pasado:

**D-J3.** Coste: 1 fable ≈ 1 h de fábrica. Firmado sí: en C cazó 19 nulas
disfrazadas de miss y 16 aceptables omitidos. Con el gold agéntico, la
verificación es estructural (dos jueces) más el consultor-gate.

**D-J4.** NETO ≥ 3 / 4 / 5. Tabla completa en el borrador §7. Firmado 4: tres
decisiones GANA en cadena, cota familiar de adoptar algo inútil ≈ 0,32 frente
a 0,41 con 3; potencia ante +8 pp 0,91–0,94 con N=100–120.

**D-J5.** (a) corrección de nombre y documentación de
`similitud_desde_l2_cuadrado`, sin brazo; (b) además celda descriptiva
`sellado-cos`. Firmado (a): (b) es una recalibración de β disfrazada.

**D-J6.** int8 fuera: pregunta de no inferioridad que la regla GANA no puede
responder; K canibaliza el beneficio.

**D-J7.** 0,90: rompe empates a favor del vivo (0,60 → 0,54 frente a un vivo
de 0,55) sin expulsar a `archive/` del top-10.

**D-J8.** 0,25 con la cláusula de T0 intacta: el T0 puede sustituir el valor
**una vez**, con la evidencia anotada como enmienda (Task 5 Step 2).

**D-J9.** Percentil 70 de calibración; adopta si abstiene ≥ 60 % de los
negativos de evaluación y ROMPE ≤ 2.

**D-J10.** Lenient decide (D5 de C), strict descriptivo.

**D-J11 — por qué descartar los desacuerdos y no desempatar con opus.** (i)
Un tercer juez Claude resuelve cada desacuerdo fable–Kimi como 2 Claude
contra 1 Kimi: el gold quedaría etiquetado, de hecho, por una sola familia,
que es exactamente lo que el segundo juez existe para evitar. (ii) El pool
sobra: ≈ 280 filas juzgadas → con p_o ≈ 0,75, ≈ 210 entran, ≈ 100–130 no
nulas, por encima del objetivo 80–95. (iii) Descartar sesga el gold hacia
filas «claras», que suelen ser las léxicamente fáciles: ese sesgo es
medible (auditoría de solape en `acuerdo.py`) y se contiene con el guard
léxico de D-A (borrador §6); un desempate no lo haría medible. (iv) Coste:
0 opus.

## Dependencias y conflictos

| Tarea | Ficheros | Conflicto | Regla |
|---|---|---|---|
| 1, 3, 4 (harness) | `evals/retrieval-heldout/harness/{diagnostico,valida_gold,pool,limpia_agent_search,juez,acuerdo,test_harness,test_gold_j}.py` | ninguna campaña activa toca `evals/retrieval-heldout/` (matriz de colisión de la propuesta §3: solo J) | libre |
| 2 (captura «hoy») | ninguno de código; lee `~/.exo/index.db` | I y L tocan `plugins/` y `engine/src/main.rs`; la captura usa el binario de `main` en el momento de la Task 2 y lo anota | anotar `git rev-parse HEAD` del binario en el fichero público |
| 6, 7 (gold) | `evals/retrieval-heldout/gold-j/`, `verdict/gold-j-acuerdo.md`, `$PRIV_J`; red hacia Moonshot (Task 7) | ninguno en el repo; la key vive en `wisdom-ai-news/.env-keys` (solo lectura) | libre |
| 5, 8 (borrador y congelación) | `docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md` | ninguno | inmutable tras la Task 8 salvo erratas → verdict |
| fase 2 (no en este plan) | `engine/src/buscador.rs` | L (regresión `exo search` sin `vectores`) toca `buscador.rs`/`main.rs` | fase 2 después de L mergeada y de `v0.2.0` |

## Estructura de ficheros

- Create: `evals/retrieval-heldout/harness/diagnostico.py` — T0: cruce
  hist/S/hoy por fila, causas, df de tokens, `archive/` y rotaciones; salida
  pública e informe privado.
- Modify: `evals/retrieval-heldout/harness/test_harness.py` — `TestDiagnostico`,
  `TestValidaGold.test_estratos_j`, `test_exclusion_multiple`,
  `TestPool.test_filtra_con_ventanas_propias`,
  `test_muestrea_cuota_cero_devuelve_estrato_entero`.
- Modify: `evals/retrieval-heldout/harness/valida_gold.py` — estratos
  `agent-search|archive|negativo` (C ya tenía `agent-search`; `keyword`
  queda admitido pero sin uso), `--in-sample` repetible, `nulas_por_estrato`.
- Modify: `evals/retrieval-heldout/harness/pool.py` — `filtra(…, ventanas)`,
  `--desde/--hasta`, `--in-sample` repetible.
- Create: `evals/retrieval-heldout/harness/limpia_agent_search.py` — limpieza
  con criterio escrito del estrato `agent-search`.
- Create: `evals/retrieval-heldout/harness/juez.py` — paquetes de juez,
  validación de respuestas, cliente Kimi (structured output), `/v1/models`.
- Create: `evals/retrieval-heldout/harness/acuerdo.py` — acuerdo, κ, fusión
  de etiquetas, descartes, auditoría léxica, gold.
- Create: `evals/retrieval-heldout/harness/test_gold_j.py` — 7 tests de los
  tres anteriores (HTTP inyectado; sin red).
- Create: `evals/retrieval-heldout/verdict/diagnostico-55.md` — T0 público.
- Create: `evals/retrieval-heldout/gold-j/README.md`,
  `evals/retrieval-heldout/gold-j/plantilla.jsonl` — método y plantilla.
- Create: `evals/retrieval-heldout/verdict/gold-j-acuerdo.md` — informe
  público del acuerdo (solo recuentos).
- Modify: `docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md`
  — §4 (tope df, si T0 lo justifica), §10 y cabecera (Task 8).
- Gitignored: `.superpowers/fabrica/ledger.md`, `.superpowers/fabrica/pendiente-paul.md`
  (solo objeciones), `.superpowers/fabrica/verdicts/j-consultor-gate.md`.
- Privado: `$PRIV_J/{kb-snap/,kb-snap.commit,cap-hoy-55.jsonl,diagnostico-55-detalle.md,muestra-prompt.jsonl,agent-search.jsonl,notas-hard.txt,notas-archive.txt,generadas.jsonl,queries.jsonl,candidatos*.jsonl,paquetes.jsonl,kimi-modelos.json,humo-kimi.jsonl,juicio-kimi.jsonl,juicio-fable*.jsonl,gold-j.jsonl,descartes.jsonl,review-preregistro.md}`.

---

### Task 1: `diagnostico.py` con tests (T0, herramienta)

**Lane:** mecánica (TDD sobre fixtures sintéticos; no lee datos privados).
**Oráculo:** `python3 -m unittest -v evals/retrieval-heldout/harness/test_harness.py` → `OK`, 36 tests.

**Files:**
- Create: `evals/retrieval-heldout/harness/diagnostico.py`
- Modify: `evals/retrieval-heldout/harness/test_harness.py`

**Interfaces:**
- Consumes: `carga_gold`, `carga_captura`, `relevantes`, `K_HIT`,
  `UMBRAL_SELLADO` de `metricas.py`; `normaliza` de `pool.py`;
  `permalinks_snapshot` de `valida_gold.py` (todos existentes, sin cambios).
- Produces (usado por la Task 2):
  - `misses_historicos(ruta_md: str) -> list[str]` (prefijos de las líneas `- MISS \`…\` →`)
  - `empareja_prefijos(prefijos: list[str], filas: list[dict]) -> dict[str, str]` (prefijo → id; `ValueError` si 0 o >1)
  - `es_rotacion(expected: str, permalink: str) -> bool`
  - `df_tokens(conn: sqlite3.Connection, query: str) -> list[tuple[str, int]]`
  - `diagnostica_fila(fila: dict, cap: dict|None, dfs: list[tuple[str,int]], estricto: bool=False) -> dict` con claves `hit5, rango_vector, score_vector, rango_vector_adm, n_fts, rango_fts, rango_hybrid, archive_top5, rotacion_top5, top5, tokens_df0, n_tokens, reverificar, causas`
  - `tokens_raros(dfs, n_notas: int) -> list[str]` · `REVERIFICAR: set[str]` · `DF_RARO = 0.25` · `CAUSAS: tuple[str, ...]`
  - `informe(filas, hist_ids: set[str], diag_s: dict, diag_hoy: dict, muertas_hoy: list[str], publico: bool, dfs=None, n_notas=0) -> str`
  - CLI: `diagnostico.py --gold IN55 --hist MD --cap-s CAP --cap-hoy CAP --db IDX --kb-hoy KB --publico OUT.md --privado OUT.md` → stdout JSON `{"no_nulas","miss_hist","miss_S","miss_hoy","muertas_hoy"}`

- [ ] **Step 1: Escribir los tests que fallan**

En `evals/retrieval-heldout/harness/test_harness.py`: (1) añadir `import sqlite3` tras `import json`; (2) añadir `import diagnostico as dg  # noqa: E402` justo después de la línea `import valida_gold as vg  # noqa: E402`; (3) insertar esta clase antes de `if __name__ == "__main__":`:

```python
class TestDiagnostico(unittest.TestCase):
    def filas(self):
        return [{"id": "m01", "query": "cge bitácora", "source": "log", "expected_permalink": "kb/log/cge-bitacora", "acceptable_permalinks": []},
                {"id": "m02", "query": "cge bitácora evaluación larga", "source": "log", "expected_permalink": "kb/x", "acceptable_permalinks": []},
                {"id": "m03", "query": "fabrica campaña", "source": "hard", "expected_permalink": "kb/y", "acceptable_permalinks": []}]

    def test_misses_historicos_y_prefijos(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "m.md"
            p.write_text("# x\n- MISS `cge bitácora` → text=miss vector=miss [both-miss]\n- MISS `fabrica camp` → text=miss vector=miss [both-miss]\n- otra línea\n", encoding="utf-8")
            pre = dg.misses_historicos(str(p))
        self.assertEqual(pre, ["cge bitácora", "fabrica camp"])
        self.assertEqual(dg.empareja_prefijos(pre, self.filas()), {"cge bitácora": "m01", "fabrica camp": "m03"})
        with self.assertRaises(ValueError):
            dg.empareja_prefijos(["cge"], self.filas())

    def test_es_rotacion(self):
        self.assertTrue(dg.es_rotacion("kb/log/exo-bitacora", "kb/archive/log/exo-bitacora-2026-07-17_2026-09-04"))
        self.assertFalse(dg.es_rotacion("kb/log/exo-bitacora", "kb/archive/log/otra-bitacora-2026-07-17_2026-09-04"))
        self.assertFalse(dg.es_rotacion("kb/core/doctrina", "kb/archive/log/exo-bitacora-2026-07-17_2026-09-04"))
        self.assertFalse(dg.es_rotacion("kb/archive/log/exo-bitacora-2026-07-17_2026-09-04", "kb/archive/log/exo-bitacora-2026-07-17_2026-09-04"))

    def test_df_tokens(self):
        c = sqlite3.connect(":memory:")
        c.execute("CREATE VIRTUAL TABLE notas_fts USING fts5(titulo, cuerpo, permalink UNINDEXED, tokenize='unicode61 tokenchars 0x2F')")
        c.execute("INSERT INTO notas_fts VALUES ('t', 'la fábrica de campañas', 'kb/a')")
        c.execute("INSERT INTO notas_fts VALUES ('t', 'campaña sola', 'kb/b')")
        self.assertEqual(dg.df_tokens(c, 'fabrica campaña "x"'), [("fabrica", 1), ("campaña", 1), ('"x"', 0)])

    def test_causas(self):
        f = self.filas()[0]
        dfs = [("cge", 3), ("bitácora", 0)]
        bajo = {"fts": [], "vector": [("kb/log/cge-bitacora", 0.30), ("kb/z", 0.5)], "hybrid": [("kb/z", 0.5)]}
        self.assertEqual(dg.diagnostica_fila(f, bajo, dfs)["causas"], ["vector-bajo-umbral", "fts-and-vacio"])
        lejos = {"fts": [("kb/q", 9.0)], "vector": [(f"kb/v{i}", 0.6 - i / 100) for i in range(6)] + [("kb/log/cge-bitacora", 0.45)],
                 "hybrid": [(f"kb/v{i}", 0.6 - i / 100) for i in range(6)]}
        self.assertEqual(dg.diagnostica_fila(f, lejos, dfs)["causas"], ["vector-lejos", "fts-sin-relevante"])
        desplaza = {"fts": [(f"kb/f{i}", 9.0 - i) for i in range(5)], "vector": [("kb/log/cge-bitacora", 0.45)],
                    "hybrid": [(f"kb/f{i}", 0.6 - i / 100) for i in range(5)] + [("kb/log/cge-bitacora", 0.45)]}
        d = dg.diagnostica_fila(f, desplaza, dfs)
        self.assertEqual(d["causas"], ["fusion-desplaza", "fts-sin-relevante"])
        self.assertEqual((d["rango_vector_adm"], d["rango_hybrid"]), (1, 6))
        rot = {"fts": [], "vector": [("kb/archive/log/cge-bitacora-2026-01-01_2026-02-02", 0.5), ("kb/log/cge-bitacora", 0.41)],
               "hybrid": [("kb/archive/log/cge-bitacora-2026-01-01_2026-02-02", 0.5), ("kb/log/cge-bitacora", 0.41)]}
        d = dg.diagnostica_fila(f, rot, dfs)
        self.assertTrue(d["hit5"] and d["rotacion_top5"] and d["archive_top5"] == 1 and d["causas"] == [])
        self.assertEqual(dg.diagnostica_fila(f, None, dfs)["causas"], ["captura-error"])

    def test_informe_publico_no_filtra_texto(self):
        filas = self.filas()
        cap = {"fts": [], "vector": [("kb/log/cge-bitacora", 0.3)], "hybrid": [("kb/n", 0.5)]}
        ds = {f["id"]: dg.diagnostica_fila(f, cap, [("cge", 2)]) for f in filas}
        pub = dg.informe(filas, {"m01"}, ds, ds, [], True)
        priv = dg.informe(filas, {"m01"}, ds, ds, [], False)
        for texto in ("cge bitácora", "kb/log/cge-bitacora", "kb/n"):
            self.assertNotIn(texto, pub)
            self.assertIn(texto, priv)
        self.assertIn("hit→miss 2 · miss→hit 0 · miss→miss 1", pub)
        self.assertIn("| m01 | log | miss | miss | miss |", pub)
        con = dg.informe(filas, {"m01"}, ds, ds, [], True, {f["id"]: [("cge", 2), ("bitácora", 90)] for f in filas}, 100)
        self.assertIn("(1 ≤ df ≤ ⌈0.25·100⌉ = 25): 3/3", con)
        self.assertEqual(dg.tokens_raros([("a", 0), ("b", 1), ("c", 25), ("d", 26)], 100), ["b", "c"])
```

- [ ] **Step 2: Correr y verlo fallar**

Run: `python3 -m unittest evals/retrieval-heldout/harness/test_harness.py`
Expected: `ModuleNotFoundError: No module named 'diagnostico'`.

- [ ] **Step 3: Implementar `diagnostico.py`**

`evals/retrieval-heldout/harness/diagnostico.py` (fichero completo):

```python
#!/usr/bin/env python3
"""Diagnóstico por fila de las 55 in-sample (campaña J, T0; pre-registro
borrador §2.7). DESCRIPTIVO: no decide nada, no re-etiqueta, no consume
held-out. Cruza tres estados del mismo conjunto de 55 queries:
  hist : misses del sweep sellado (metrics-engine-hybrid-b0.0-e0.6.md, 49/55,
         KB de 138 notas, 2026-07), leídos de las líneas "- MISS `...`"
  S    : captura de la campaña C (cap-base-55.jsonl, binario 4f4d2a8,
         snapshot 885246d, 174 notas) — 43/55
  hoy  : captura contra el índice de producción con el binario actual
y por fila explica el miss con las listas capturadas: rango de la nota
relevante en vector (lista completa y admitida ≥0.40), en FTS y en hybrid;
df de cada token de la query en el índice (por qué el AND da 0); cuántas
notas de archive/ y qué rotaciones de la nota esperada ocupan el top-5.

Dos salidas: --publico (ids mNN, sin texto de query ni permalinks: el repo
es público) y --privado (todo, a $PRIV_J).
Uso: diagnostico.py --gold IN55 --hist MD --cap-s CAP --cap-hoy CAP --db IDX
     --kb-hoy KB --publico OUT.md --privado OUT.md
"""
import argparse
import json
import math
import re
import sqlite3
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from metricas import K_HIT, UMBRAL_SELLADO, carga_captura, carga_gold, relevantes  # noqa: E402
from pool import normaliza  # noqa: E402
from valida_gold import permalinks_snapshot  # noqa: E402

# Filas que la auditoría del 2026-09-04 mandó re-verificar (9 SUPERADA + 2
# AMBIGUA; exo-bitacora archivada 2026-07-17_2026-09-04). Numeración 1-based
# de eval.jsonl = id mNN que asigna carga_gold (la fila 9 nula cuenta).
REVERIFICAR = {"m07", "m13", "m22", "m23", "m24", "m25", "m27", "m29", "m30", "m34", "m38"}
DF_RARO = 0.25  # F1 del borrador §4: token raro si 1 <= df <= ceil(DF_RARO * n_notas)
CAUSAS = ("vector-bajo-umbral", "vector-lejos", "fusion-desplaza", "fts-and-vacio",
          "fts-sin-relevante", "rotacion-en-top5", "etiqueta-reverificar", "captura-error")


def misses_historicos(ruta_md):
    """Prefijos (≤60 chars) de las queries MISS de un metrics-*.md de fase0."""
    return re.findall(r"^- MISS `(.*?)` → ", Path(ruta_md).read_text(encoding="utf-8"), flags=re.M)


def empareja_prefijos(prefijos, filas):
    """prefijo -> id. Gana la igualdad exacta normalizada; si no, el único
    startswith. 0 o >1 candidatos = error (no se adivina)."""
    out = {}
    for p in prefijos:
        n = normaliza(p)
        exactas = [f["id"] for f in filas if normaliza(f["query"]) == n]
        cands = exactas if len(exactas) == 1 else [f["id"] for f in filas if normaliza(f["query"]).startswith(n)]
        if len(cands) != 1:
            raise ValueError(f"prefijo sin fila única: {p!r} -> {cands}")
        out[p] = cands[0]
    return out


def rango(lista, rel):
    for i, (p, _) in enumerate(lista, start=1):
        if p in rel:
            return i
    return None


def es_rotacion(expected, permalink):
    """`<kb>/archive/log/<base>-<fechas>` es rotación de `<kb>/log/<base>`."""
    if "/archive/" in expected or "/log/" not in expected or "/archive/log/" not in permalink:
        return False
    base = expected.rsplit("/log/", 1)[1]
    return permalink.rsplit("/archive/log/", 1)[1].startswith(base + "-")


def df_tokens(conn, query):
    """df de cada token tal y como lo vería prepara_query (token entre
    comillas, comillas internas duplicadas): nº de notas con MATCH."""
    out = []
    for tok in query.split():
        frase = '"' + tok.replace('"', '""') + '"'
        try:
            n = conn.execute("SELECT count(*) FROM notas_fts WHERE notas_fts MATCH ?", (frase,)).fetchone()[0]
        except sqlite3.OperationalError:
            n = -1
        out.append((tok, n))
    return out


def diagnostica_fila(fila, cap, dfs, estricto=False):
    rel = relevantes(fila, estricto)
    exp = fila["expected_permalink"]
    d = {"id": fila["id"], "source": fila.get("source"), "hit5": False, "rango_vector": None,
         "score_vector": None, "rango_vector_adm": None, "n_fts": 0, "rango_fts": None,
         "rango_hybrid": None, "archive_top5": 0, "rotacion_top5": False, "top5": [],
         "tokens_df0": [t for t, n in dfs if n == 0], "n_tokens": len(dfs),
         "reverificar": fila["id"] in REVERIFICAR, "causas": []}
    if cap is None:
        d["causas"].append("captura-error")
        return d
    vec, fts, hyb = cap["vector"], cap["fts"], cap["hybrid"]
    vec_adm = [(p, s) for p, s in vec if s >= UMBRAL_SELLADO]
    rv, rva, rf, rh = rango(vec, rel), rango(vec_adm, rel), rango(fts, rel), rango(hyb, rel)
    sv = next((s for p, s in vec if p in rel), None)
    top5 = [p for p, _ in hyb[:K_HIT]]
    d.update(hit5=rh is not None and rh <= K_HIT, rango_vector=rv, score_vector=sv, rango_vector_adm=rva,
             n_fts=len(fts), rango_fts=rf, rango_hybrid=rh, top5=top5,
             archive_top5=sum("/archive/" in p for p in top5),
             rotacion_top5=any(es_rotacion(exp, p) for p in top5))
    if d["hit5"]:
        return d
    if sv is None or sv < UMBRAL_SELLADO:
        d["causas"].append("vector-bajo-umbral")
    elif rva is not None and rva > K_HIT:
        d["causas"].append("vector-lejos")
    elif rva is not None and rva <= K_HIT:
        d["causas"].append("fusion-desplaza")
    if len(fts) == 0:
        d["causas"].append("fts-and-vacio")
    elif rf is None:
        d["causas"].append("fts-sin-relevante")
    if d["rotacion_top5"]:
        d["causas"].append("rotacion-en-top5")
    if d["reverificar"]:
        d["causas"].append("etiqueta-reverificar")
    return d


def _f(x, nd=3):
    return "—" if x is None else (round(x, nd) if isinstance(x, float) else x)


def tokens_raros(dfs, n_notas):
    tope = math.ceil(DF_RARO * n_notas)
    return [t for t, n in dfs if 1 <= n <= tope]


def informe(filas, hist_ids, diag_s, diag_hoy, muertas_hoy, publico, dfs=None, n_notas=0):
    no_nulas = [f for f in filas if f.get("expected_permalink")]
    ids = [f["id"] for f in no_nulas]
    miss_s = {i for i in ids if not diag_s[i]["hit5"]}
    miss_hoy = {i for i in ids if not diag_hoy[i]["hit5"]}
    n = len(ids)
    L = ["# Diagnóstico por fila de las 55 in-sample — campaña J, T0 (descriptivo; no decide)", ""]
    L.append(f"- no nulas: {n} · hist misses {len(hist_ids)} ({n - len(hist_ids)}/{n}) · S misses {len(miss_s)} ({n - len(miss_s)}/{n}) · hoy misses {len(miss_hoy)} ({n - len(miss_hoy)}/{n})")
    L.append(f"- transición hist→S: hit→miss {len(miss_s - hist_ids)} · miss→hit {len(hist_ids - miss_s)} · miss→miss {len(hist_ids & miss_s)}")
    L.append(f"- transición S→hoy: hit→miss {len(miss_hoy - miss_s)} · miss→hit {len(miss_s - miss_hoy)} · miss→miss {len(miss_s & miss_hoy)}")
    L.append(f"- etiquetas cuyo expected_permalink no existe en la KB de hoy: {len(muertas_hoy)}")
    L.append(f"- filas marcadas re-verificar (auditoría 2026-09-04): {len(REVERIFICAR & set(ids))}; de ellas miss en S: {len(REVERIFICAR & miss_s)}")
    if dfs is not None:
        con_raro = sum(1 for i in ids if tokens_raros(dfs[i], n_notas))
        L.append(f"- no nulas con ≥1 token raro (1 ≤ df ≤ ⌈{DF_RARO}·{n_notas}⌉ = {math.ceil(DF_RARO * n_notas)}): {con_raro}/{n} — filas en las que F1 (FTS OR sobre raros) tendría canal léxico; de los misses en S: {sum(1 for i in miss_s if tokens_raros(dfs[i], n_notas))}/{len(miss_s)}")
    for nombre, dg, ms in (("S", diag_s, miss_s), ("hoy", diag_hoy, miss_hoy)):
        c = Counter(x for i in ms for x in dg[i]["causas"])
        L += ["", f"## causas de los {len(ms)} misses en {nombre} (una fila puede llevar varias)", ""]
        L += [f"- {k}: {c.get(k, 0)}" for k in CAUSAS]
        L.append(f"- plazas de archive/ en el top-5 de las no nulas: {sum(dg[i]['archive_top5'] for i in ids)}/{K_HIT * n}")
        L.append(f"- no nulas con FTS vacío (AND): {sum(1 for i in ids if dg[i]['n_fts'] == 0)}/{n}")
    L += ["", "## por fila (no nulas; rv=rango vector completo, sv=score vector, rva=rango vector ≥0.40, nF=candidatos FTS, rf=rango FTS, rh=rango hybrid, a5=archive en top-5, rot=rotación de la esperada en top-5, df0=tokens con df 0 / tokens)", "",
          "| id | src | hist | S | hoy | rv_S | sv_S | rva_S | nF_S | rf_S | rh_S | a5_S | rot_S | df0 | causas_S | causas_hoy |", "|" + "---|" * 16]
    for f in no_nulas:
        i = f["id"]
        s, h = diag_s[i], diag_hoy[i]
        L.append(f"| {i} | {f.get('source')} | {'miss' if i in hist_ids else 'hit'} | {'miss' if i in miss_s else 'hit'} | {'miss' if i in miss_hoy else 'hit'} | {_f(s['rango_vector'])} | {_f(s['score_vector'])} | {_f(s['rango_vector_adm'])} | {s['n_fts']} | {_f(s['rango_fts'])} | {_f(s['rango_hybrid'])} | {s['archive_top5']} | {'sí' if s['rotacion_top5'] else 'no'} | {len(s['tokens_df0'])}/{s['n_tokens']} | {','.join(s['causas']) or '—'} | {','.join(h['causas']) or '—'} |")
    if not publico:
        L += ["", "## detalle privado (query, esperada, top-5 en S, tokens con df 0)", ""]
        for f in no_nulas:
            s = diag_s[f["id"]]
            L.append(f"- {f['id']} `{f['query']}` → `{f['expected_permalink']}` · top5_S: {s['top5']} · df0: {s['tokens_df0']}")
    return "\n".join(L) + "\n"


def main():
    ap = argparse.ArgumentParser()
    for k in ("gold", "hist", "cap-s", "cap-hoy", "db", "kb-hoy", "publico", "privado"):
        ap.add_argument(f"--{k}", required=True)
    a = ap.parse_args()
    filas = carga_gold(a.gold)
    hist_ids = set(empareja_prefijos(misses_historicos(a.hist), filas).values())
    conn = sqlite3.connect(f"file:{a.db}?mode=ro", uri=True)
    dfs = {f["id"]: df_tokens(conn, f["query"]) for f in filas}
    cap_s, cap_hoy = carga_captura(a.cap_s), carga_captura(a.cap_hoy)
    no_nulas = [f for f in filas if f.get("expected_permalink")]
    diag_s = {f["id"]: diagnostica_fila(f, cap_s.get(f["id"]), dfs[f["id"]]) for f in no_nulas}
    diag_hoy = {f["id"]: diagnostica_fila(f, cap_hoy.get(f["id"]), dfs[f["id"]]) for f in no_nulas}
    n_notas = conn.execute("SELECT count(*) FROM notas").fetchone()[0]
    perms_hoy = permalinks_snapshot(a.kb_hoy)
    muertas = [f["id"] for f in no_nulas if f["expected_permalink"] not in perms_hoy]
    Path(a.publico).write_text(informe(filas, hist_ids, diag_s, diag_hoy, muertas, True, dfs, n_notas), encoding="utf-8")
    Path(a.privado).write_text(informe(filas, hist_ids, diag_s, diag_hoy, muertas, False, dfs, n_notas), encoding="utf-8")
    print(json.dumps({"no_nulas": len(no_nulas), "miss_hist": len(hist_ids),
                      "miss_S": sum(1 for d in diag_s.values() if not d["hit5"]),
                      "miss_hoy": sum(1 for d in diag_hoy.values() if not d["hit5"]),
                      "muertas_hoy": len(muertas)}))


if __name__ == "__main__":
    main()
```

- [ ] **Step 4: Correr y verlo pasar**

Run: `python3 -m unittest -v evals/retrieval-heldout/harness/test_harness.py 2>&1 | tail -3`
Expected: `Ran 36 tests` … `OK`.

- [ ] **Step 5: Commit**

```bash
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 add evals/retrieval-heldout/harness/diagnostico.py evals/retrieval-heldout/harness/test_harness.py
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 commit -m "eval(j): diagnostico por fila de las 55 in-sample (T0), con tests"
```

---

### Task 2: Corrida del T0 — captura «hoy» de lectura y `diagnostico-55.md`

**Lane:** diseño (produce el fichero público del T0 y el detalle privado).
**Oráculo:** `evals/retrieval-heldout/verdict/diagnostico-55.md` existe, su primera línea de datos dice `hist misses 6 (49/55) · S misses 12 (43/55)`, y `grep -c "wisdom-paul" evals/retrieval-heldout/verdict/diagnostico-55.md` devuelve `0`.
**Depende de:** Task 1.

**Files:**
- Create: `evals/retrieval-heldout/verdict/diagnostico-55.md`
- Privado: `$PRIV_J/cap-hoy-55.jsonl`, `$PRIV_J/cap-hoy-55.log`, `$PRIV_J/diagnostico-55-detalle.md`

**Interfaces:**
- Consumes: `diagnostico.py` (Task 1); `captura.py` y `metricas.py fidelidad` de C (sin cambios); `$PRIV_C/{in-sample-55.jsonl,cap-base-55.jsonl,idx-base.db}`; `evals/retrieval-fase0/results/metrics-engine-hybrid-b0.0-e0.6.md`.
- Produces: el fichero público con la sección `## Lectura` rellena (la Task 5 y el borrador §2.6 la citan).

- [ ] **Step 1: Directorio privado y binario de lectura**

```bash
export PRIV_C=~/.local/share/exo-evals/c-heldout PRIV_J=~/.local/share/exo-evals/j-heldout
mkdir -p "$PRIV_J" && chmod 700 "$PRIV_J"
ls "$PRIV_C/in-sample-55.jsonl" "$PRIV_C/cap-base-55.jsonl" "$PRIV_C/idx-base.db"
cargo build --release --locked --manifest-path engine/Cargo.toml 2>&1 | tail -1
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 rev-parse HEAD
engine/target/release/exo --version
```
Expected: los tres ficheros listados; `Finished`; un sha; `exo 0.1.0` (o el de `main`). Anotar el sha: es el «binario hoy».

- [ ] **Step 2: Captura «hoy» (solo lectura sobre el índice de producción)**

```bash
python3 evals/retrieval-heldout/harness/captura.py --gold "$PRIV_C/in-sample-55.jsonl" --db ~/.exo/index.db --exo engine/target/release/exo --out "$PRIV_J/cap-hoy-55.jsonl" 2> "$PRIV_J/cap-hoy-55.log"
tail -1 "$PRIV_J/cap-hoy-55.log"; wc -l < "$PRIV_J/cap-hoy-55.jsonl"
python3 evals/retrieval-heldout/harness/metricas.py fidelidad --captura "$PRIV_J/cap-hoy-55.jsonl"
```
Expected: `[56/56] m56` · `56` · `fidelidad(sellado): 0 discrepancias de 56` (exit 0). Si hay discrepancias: STOP, es el harness o el binario (no se sigue; PENDIENTE-PAUL con el log). Duración ≈ 3 min (168 búsquedas a ~1 s).

- [ ] **Step 3: Diagnóstico**

```bash
python3 evals/retrieval-heldout/harness/diagnostico.py \
  --gold "$PRIV_C/in-sample-55.jsonl" \
  --hist evals/retrieval-fase0/results/metrics-engine-hybrid-b0.0-e0.6.md \
  --cap-s "$PRIV_C/cap-base-55.jsonl" --cap-hoy "$PRIV_J/cap-hoy-55.jsonl" \
  --db "$PRIV_C/idx-base.db" --kb-hoy ~/Documentos/proyectos/wisdom-paul \
  --publico evals/retrieval-heldout/verdict/diagnostico-55.md \
  --privado "$PRIV_J/diagnostico-55-detalle.md"
grep -c "wisdom-paul" evals/retrieval-heldout/verdict/diagnostico-55.md
sed -n 3,9p evals/retrieval-heldout/verdict/diagnostico-55.md
```
Expected: stdout `{"no_nulas": 55, "miss_hist": 6, "miss_S": 12, "miss_hoy": <n>, "muertas_hoy": 0}`; `0`; las líneas de cabecera con `hit→miss 7 · miss→hit 1 · miss→miss 5` en hist→S (dry-run del 2026-09-19). `miss_hoy` es lo único desconocido: el índice de producción no es el snapshot S.

- [ ] **Step 4: Sección `## Lectura` (hecho a mano, 8 líneas, sin texto ni permalinks)**

Añadir al final de `evals/retrieval-heldout/verdict/diagnostico-55.md` esta sección, sustituyendo cada `<…>` por el valor que ya está en el propio fichero o en el stdout del Step 3 (son salidas de comando, no juicios):

```markdown
## Lectura (T0; in-sample; descriptivo; no decide nada sobre el motor)

- Condiciones: hist = `metrics-engine-hybrid-b0.0-e0.6.md` (KB 138 notas, binario del sweep de 2026-07); S = captura de C (binario `4f4d2a8`, snapshot `885246d`, 174 notas); hoy = `~/.exo/index.db` de producción con el binario `<sha del Step 1>` el `<date -I>`. Ninguna cifra es comparable entre estados en absoluto (cambian KB, distractores y binario a la vez; C §6 R1): solo se cruzan filas.
- El «6 de 55 que dejaron de acertar» del backlog es neto: hist→S es hit→miss `<n>`, miss→hit `<n>`, miss→miss `<n>`.
- De los `<n>` misses en S, `<n>` son `vector-lejos` (la nota está admitida ≥0,40 pero fuera del top-5), `<n>` `vector-bajo-umbral`, `<n>` `fusion-desplaza`. El AND deja FTS vacío en `<n>` de esos misses y en `<n>/55` del total.
- `<n>` de los `<n>` misses tienen en su top-5 una rotación de la nota esperada (`archive/log/<misma base>`), y `<n>` de las 11 filas que la auditoría del 2026-09-04 mandó re-verificar (9 SUPERADA + 2 AMBIGUA) son miss en S. Lectura: buena parte de la caída es etiqueta superada por rotación, no motor. No se re-etiqueta (C §11); se anota para el estrato `archive` del gold de J.
- `archive/` ocupa `<n>/275` plazas del top-5 de las 55 en S.
- F1 (FTS OR sobre tokens raros, tope df 25 %): `<n>/55` queries tendrían canal léxico, `<n>` de los misses. Valor para D-J8: `<0,25 se mantiene | se propone X porque …>`.
- S→hoy: hit→miss `<n>`, miss→hit `<n>`. Etiquetas muertas hoy: `<n>`.
- Lo que este fichero NO permite concluir: nada sobre qué brazo adoptar (held-out de C consumido; el de J no existe), nada sobre «el binario empeoró».
```

- [ ] **Step 5: Commit del público (el privado no se commitea)**

```bash
grep -c "wisdom-paul" evals/retrieval-heldout/verdict/diagnostico-55.md
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 add evals/retrieval-heldout/verdict/diagnostico-55.md
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 commit -m "eval(j): T0 — diagnostico por fila de las 55 (hist/S/hoy), sin texto ni permalinks"
```
Expected: `0` antes del add.

---

### Task 3: `valida_gold.py` y `pool.py` aceptan el gold de J (estratos nuevos, exclusiones repetibles, ventana)

**Lane:** mecánica (TDD; el oráculo final corre sobre el gold real de C como regresión).
**Oráculo:** `python3 -m unittest evals/retrieval-heldout/harness/test_harness.py` → 40 tests `OK`, **y** `python3 evals/retrieval-heldout/harness/valida_gold.py --gold "$PRIV_C/gold.jsonl" --kb "$PRIV_C/kb-snap" --in-sample "$PRIV_C/in-sample-55.jsonl"` sigue en exit 0 con `"errores": 0` y `"no_nulas": 92`.
**Depende de:** nada (paralelizable con la Task 1; ambas tocan `test_harness.py` en clases distintas: rebase trivial).

**Files:**
- Modify: `evals/retrieval-heldout/harness/valida_gold.py`
- Modify: `evals/retrieval-heldout/harness/pool.py`
- Modify: `evals/retrieval-heldout/harness/test_harness.py`

**Interfaces:**
- Consumes: `pool.normaliza`, `pool.jaccard`, `pool.VENTANAS`, `pool.VENTANA_FIN` (existentes).
- Produces (usado por las Tasks 4, 6, 7):
  - `valida_gold.FUENTES = {"prompt", "agent-search", "hard", "keyword", "archive", "negativo"}`; reglas nuevas en `valida(filas, permalinks, in55)`: `negativo` ⇒ `expected_permalink` null; `archive` ⇒ `expected_permalink` contiene `/archive/`.
  - CLI `valida_gold.py --gold G --kb KB --in-sample A [--in-sample B …]`; JSON de salida con clave nueva `nulas_por_estrato`.
  - `pool.filtra(candidatas, in55, ventanas=None)`; CLI `pool.py --in-sample A [--in-sample B] --out-dir D --cuota prompt=0 [--desde ISO] [--hasta ISO]` (cuota 0 = estrato entero barajado; `--desde/--hasta` aplican a todas las fuentes).

- [ ] **Step 1: Tests que fallan**

En `test_harness.py`, dentro de `class TestValidaGold`, después de `test_ids_duplicados`:

```python
    def test_estratos_j(self):
        perms = self.PERMS | {"kb/archive/log/x-2026-01-01_2026-02-02"}
        self.assertEqual(vg.valida([self.fila(id="j1", source="keyword")], perms, []), [])
        self.assertEqual(vg.valida([self.fila(id="j2", source="negativo", expected_permalink=None)], perms, []), [])
        self.assertEqual(vg.valida([self.fila(id="j3", source="archive", expected_permalink="kb/archive/log/x-2026-01-01_2026-02-02")], perms, []), [])
        self.assertTrue(vg.valida([self.fila(id="j4", source="negativo")], perms, []))
        self.assertTrue(vg.valida([self.fila(id="j5", source="archive")], perms, []))
        self.assertTrue(vg.valida([self.fila(id="j6", source="archive", expected_permalink=None)], perms, []))

    def test_exclusion_multiple(self):
        self.assertTrue(vg.valida([self.fila(query="Gold de C tal cual")], self.PERMS, [pl.normaliza("fabrica campaña"), pl.normaliza("gold de C tal cual")]))
```

Dentro de `class TestPool`, justo antes de `test_muestrea_es_independiente_del_orden_de_las_cuotas`:

```python
    def test_filtra_con_ventanas_propias(self):
        c = {"query": "una query de J", "source": "prompt", "session_id": "s", "ts": "2026-09-15T00:00:00Z"}
        self.assertEqual(pl.filtra([c], [])[0], [])
        v = {"prompt": ("2026-09-13T00:00:00Z", "2026-09-20T00:00:00Z"), "agent-search": ("2026-09-13T00:00:00Z", "2026-09-20T00:00:00Z")}
        self.assertEqual(len(pl.filtra([c], [], v)[0]), 1)

    def test_muestrea_cuota_cero_devuelve_estrato_entero(self):
        pool = [{"query": f"q{i}", "source": "prompt", "ts": f"2026-09-1{i}T00:00:00Z"} for i in range(4)]
        self.assertEqual(len(pl.muestrea(pool, [("prompt", 0)])), 4)
```

- [ ] **Step 2: Correr y verlo fallar**

Run: `python3 -m unittest evals/retrieval-heldout/harness/test_harness.py 2>&1 | tail -3`
Expected: `FAILED (failures=1, errors=1)` — FAIL `test_estratos_j` (source inválido) y ERROR `test_filtra_con_ventanas_propias` (`TypeError: filtra() takes 2 positional arguments but 3 were given`). `test_exclusion_multiple` y `test_muestrea_cuota_cero_devuelve_estrato_entero` pasan ya con el código original (la exclusión por lista era genérica y `muestrea` con n=0 no lanza): los fija para que nadie los rompa.

- [ ] **Step 3: Aplicar los cambios (diff exacto)**

`evals/retrieval-heldout/harness/valida_gold.py`:

```diff
--- a/valida_gold.py
+++ b/valida_gold.py
@@ -1,7 +1,9 @@
 #!/usr/bin/env python3
-"""Valida el gold del held-out C (pre-registro §3) contra el snapshot de la KB.
-Exit 1 con la lista de errores; exit 0 imprime recuentos y sha256.
-Uso: valida_gold.py --gold G --kb KB --in-sample IN55
+"""Valida un gold de held-out (C: pre-registro §3; J: borrador §3) contra el
+snapshot de la KB. Exit 1 con la lista de errores; exit 0 imprime recuentos
+y sha256. `--in-sample` es repetible: cada fichero JSONL con campo `query`
+es una lista de exclusión (anti-fuga; para J: las 55 y el gold de C).
+Uso: valida_gold.py --gold G --kb KB --in-sample IN55 [--in-sample GOLD_C]
 """
 import argparse
 import hashlib
@@ -13,7 +15,10 @@
 sys.path.insert(0, str(Path(__file__).resolve().parent))
 from pool import jaccard, normaliza  # noqa: E402
 
-FUENTES = {"prompt", "agent-search", "hard"}
+# C: prompt | agent-search | hard. J añade keyword (palabras clave escritas
+# por Paul), archive (la respuesta vive en archive/) y negativo (tema ausente
+# de la KB: corpus negativo verdadero, expected null por definición).
+FUENTES = {"prompt", "agent-search", "hard", "keyword", "archive", "negativo"}
 
 
 def permalinks_snapshot(kb):
@@ -49,6 +54,10 @@
         if any(n == m or jaccard(n, m) >= 0.8 for m in in55):
             errores.append(f"{i}: query duplica una de las 55")
         exp, acc = f["expected_permalink"], f["acceptable_permalinks"]
+        if f["source"] == "negativo" and exp is not None:
+            errores.append(f"{i}: source negativo con expected_permalink (debe ser null)")
+        if f["source"] == "archive" and (exp is None or "/archive/" not in exp):
+            errores.append(f"{i}: source archive exige expected_permalink bajo archive/")
         if exp is None:
             if acc:
                 errores.append(f"{i}: fila null con acceptable_permalinks")
@@ -70,10 +79,10 @@
     ap = argparse.ArgumentParser()
     ap.add_argument("--gold", required=True)
     ap.add_argument("--kb", required=True)
-    ap.add_argument("--in-sample", required=True)
+    ap.add_argument("--in-sample", action="append", required=True, help="repetible: JSONL de exclusión")
     a = ap.parse_args()
     filas = [json.loads(l) for l in open(a.gold, encoding="utf-8") if l.strip()]
-    in55 = [normaliza(json.loads(l)["query"]) for l in open(a.in_sample, encoding="utf-8") if l.strip()]
+    in55 = [normaliza(json.loads(l)["query"]) for ruta in a.in_sample for l in open(ruta, encoding="utf-8") if l.strip()]
     errores = valida(filas, permalinks_snapshot(a.kb), in55)
     for e in errores:
         print(e, file=sys.stderr)
@@ -81,6 +90,7 @@
     print(json.dumps({
         "filas": len(filas), "no_nulas": len(no_nulas), "nulas": len(filas) - len(no_nulas),
         "no_nulas_por_estrato": dict(Counter(f.get("source") for f in no_nulas)),
+        "nulas_por_estrato": dict(Counter(f.get("source") for f in filas if not f.get("expected_permalink"))),
         "con_acceptable": sum(1 for f in filas if f.get("acceptable_permalinks")),
         "sha256": hashlib.sha256(Path(a.gold).read_bytes()).hexdigest(),
         "errores": len(errores)}, ensure_ascii=False))
```

`evals/retrieval-heldout/harness/pool.py`:

```diff
--- a/pool.py
+++ b/pool.py
@@ -151,13 +151,14 @@
         yield {"query": mejor["query"], "source": "prompt", "session_id": e["session_id"], "ts": mejor["ts_str"]}
 
 
-def filtra(candidatas, in55):
+def filtra(candidatas, in55, ventanas=None):
+    ventanas = VENTANAS if ventanas is None else ventanas
     desc = {"vacia": 0, "guion": 0, "larga": 0, "fuera-de-ventana": 0, "dup-55": 0, "dup-pool": 0}
     vistos, pool = set(), []
     for c in candidatas:
         q = c["query"].strip()
         n = normaliza(q)
-        ini, fin = VENTANAS.get(c["source"], (None, VENTANA_FIN))
+        ini, fin = ventanas.get(c["source"], (None, VENTANA_FIN))
         if not n:
             desc["vacia"] += 1
         elif q.startswith("-"):
@@ -205,12 +206,17 @@
 
 def main():
     ap = argparse.ArgumentParser()
-    ap.add_argument("--in-sample", required=True)
+    ap.add_argument("--in-sample", action="append", required=True, help="repetible: JSONL de exclusión (55, gold de C)")
     ap.add_argument("--out-dir", required=True)
-    ap.add_argument("--cuota", action="append", required=True, help="source=n (prompt|agent-search)")
+    ap.add_argument("--cuota", action="append", required=True, help="source=n (prompt|agent-search); n=0 = estrato entero barajado")
+    ap.add_argument("--desde", help="inicio exclusivo de ventana para TODAS las fuentes (ISO Z); default: ventanas de C")
+    ap.add_argument("--hasta", help="fin exclusivo de ventana para TODAS las fuentes (ISO Z); default: 2026-09-13")
     a = ap.parse_args()
-    in55 = [normaliza(json.loads(l)["query"]) for l in open(a.in_sample, encoding="utf-8") if l.strip()]
-    pool, desc = filtra([*pool_search_notes(), *pool_comandos(), *pool_prompts()], in55)
+    in55 = [normaliza(json.loads(l)["query"]) for ruta in a.in_sample for l in open(ruta, encoding="utf-8") if l.strip()]
+    ventanas = None
+    if a.desde or a.hasta:
+        ventanas = {s: (a.desde or ini, a.hasta or fin) for s, (ini, fin) in VENTANAS.items()}
+    pool, desc = filtra([*pool_search_notes(), *pool_comandos(), *pool_prompts()], in55, ventanas)
     cuotas = [(c.split("=")[0], int(c.split("=")[1])) for c in a.cuota]
     try:
         muestra = muestrea(pool, cuotas)
```

- [ ] **Step 4: Verde y regresión sobre el gold real de C**

```bash
python3 -m unittest evals/retrieval-heldout/harness/test_harness.py 2>&1 | tail -3
python3 evals/retrieval-heldout/harness/valida_gold.py --gold "$PRIV_C/gold.jsonl" --kb "$PRIV_C/kb-snap" --in-sample "$PRIV_C/in-sample-55.jsonl" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d["no_nulas"], d["errores"], d["nulas_por_estrato"])'
```
Expected: `Ran 40 tests` … `OK` (35 si la Task 1 aún no está mergeada en la rama: 31 + 4); `92 0 {'prompt': 50, 'agent-search': 5}`.

- [ ] **Step 5: Commit**

```bash
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 add evals/retrieval-heldout/harness/valida_gold.py evals/retrieval-heldout/harness/pool.py evals/retrieval-heldout/harness/test_harness.py
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 commit -m "eval(j): valida_gold y pool aceptan el gold de J (estratos keyword/archive/negativo, exclusiones repetibles, ventana)"
```

---

### Task 4: Scripts del pipeline de gold (`limpia_agent_search.py`, `juez.py`, `acuerdo.py`) con tests

**Lane:** mecánica (TDD sobre fixtures sintéticos; el HTTP se inyecta como función falsa; no hay red ni datos privados).
**Oráculo:** `python3 -m unittest -v evals/retrieval-heldout/harness/test_gold_j.py` → `OK`, 7 tests; y `python3 -m unittest evals/retrieval-heldout/harness/test_harness.py` sigue en 40.
**Depende de:** Task 3 (`pool.query_de_comando`, `normaliza`, `jaccard`).

**Files:**
- Create: `evals/retrieval-heldout/harness/limpia_agent_search.py`
- Create: `evals/retrieval-heldout/harness/juez.py`
- Create: `evals/retrieval-heldout/harness/acuerdo.py`
- Create: `evals/retrieval-heldout/harness/test_gold_j.py`

**Interfaces:**
- Consumes: `pool.query_de_comando`, `pool.normaliza`, `pool.jaccard` (existentes).
- Produces (usado por las Tasks 6–8):
  - `limpia_agent_search.limpia(lineas: list[str], excluidas: list[str]) -> tuple[list[dict], dict]`; CLI `limpia_agent_search.py --raw F --excluye A [--excluye B] --out F.jsonl`
  - `juez.rutas_por_permalink(snap) -> dict[str, str]` · `juez.paquete(fila, rutas) -> dict` (`id, source, candidatos, texto`) · `juez.parsea(resp, candidatos) -> (dict|None, str|None)` · `juez.kimi(paq, key, model, reintentos=3, http=_http) -> (dict|None, str|None)` · `juez.modelos(env_keys) -> list[str]` · `juez.SISTEMA`, `juez.SCHEMA`, `juez.MAX_CHARS = 4000`, `juez.BASE = "https://api.moonshot.ai/v1"`; CLI `juez.py paquetes|modelos|kimi|valida`
  - `acuerdo.acuerdo_fila(a, b) -> "estricto"|"lenient"|None` · `acuerdo.kappa(pares) -> float` · `acuerdo.fusiona(a, b, tipo) -> (expected, acceptable)` · `acuerdo.solape_lexico(query, texto) -> float` · `acuerdo.construye(cands, fab, kim, textos, kappa_min, po_min) -> (gold, descartes, informe_md, kappa, po)`; CLI `acuerdo.py --candidatos C --fable F --kimi K [--snap KB] --gold-out G --descartes-out D --informe-out I.md --kappa-min 0.60 --po-min 0.70` (exit 0 pasa · 2 J PARA)
  - Formato de respuesta de juez (los dos jueces): JSONL `{"id", "expected": str|null, "acceptable": [str] ≤2, "razon": str}`; `expected` y cada `acceptable` deben estar en `candidatos` de esa fila.

- [ ] **Step 1: Escribir los tests que fallan**

`evals/retrieval-heldout/harness/test_gold_j.py`:

```python
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import acuerdo as ac  # noqa: E402
import juez as jz  # noqa: E402
import limpia_agent_search as la  # noqa: E402


class TestLimpiaAgentSearch(unittest.TestCase):
    def test_criterio(self):
        lineas = [
            'exo search --db ~/.exo/index.db --type hybrid --json "fusion rrf combsum"',
            'exo search --type fts --json "<query>"',
            'exo search --json "memoria v2" | jq .data',
            "exo search sin query ni comillas … (doc)",
            'exo search --json "Fusion  RRF combsum"',
            'exo search --json "fabrica campaña"',
            'exo search --json "x"',
            "cargo test --release",
        ]
        out, desc = la.limpia(lineas, [la.normaliza("fabrica campaña")])
        self.assertEqual([c["query"] for c in out], ["fusion rrf combsum"])
        self.assertEqual(desc, {"no_parsea": 1, "marcador": 3, "vacia": 1, "dup-exclusion": 1, "dup-pool": 1})


class TestJuez(unittest.TestCase):
    def test_paquete_y_parsea(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "core" / "n.md"
            p.parent.mkdir()
            p.write_text("---\ntitle: Nota A\npermalink: kb/core/n\n---\ncuerpo de la nota " + "x" * 5000, encoding="utf-8")
            rutas = jz.rutas_por_permalink(d)
            self.assertEqual(rutas, {"kb/core/n": str(p)})
            paq = jz.paquete({"id": "c1", "query": "q", "source": "prompt", "candidatos": ["kb/core/n", "kb/zz"]}, rutas)
        self.assertEqual(paq["candidatos"], ["kb/core/n", "kb/zz"])
        self.assertIn("título: Nota A", paq["texto"])
        self.assertIn("(nota no encontrada en el snapshot)", paq["texto"])
        self.assertLess(len(paq["texto"]), 2 * jz.MAX_CHARS)
        cands = paq["candidatos"]
        ok, err = jz.parsea('{"expected": "kb/core/n", "acceptable": ["kb/zz"], "razon": "r"}', cands)
        self.assertIsNone(err)
        self.assertEqual(ok["expected"], "kb/core/n")
        for mala in ('{"expected": "kb/otro", "acceptable": [], "razon": ""}',
                     '{"expected": null, "acceptable": ["kb/zz"], "razon": ""}',
                     '{"expected": "kb/core/n", "acceptable": ["kb/core/n"], "razon": ""}',
                     '{"expected": "kb/core/n", "acceptable": []}',
                     "no json"):
            self.assertIsNone(jz.parsea(mala, cands)[0], mala)
        self.assertEqual(jz.parsea('{"expected": null, "acceptable": [], "razon": "nada"}', [])[0]["expected"], None)

    def test_kimi_con_http_falso(self):
        paq = {"id": "c1", "candidatos": ["kb/a"], "texto": "CONSULTA: q"}
        llamadas = []

        def http(url, key, cuerpo=None, timeout=120):
            llamadas.append((url, cuerpo["model"], cuerpo["response_format"]["type"]))
            return {"choices": [{"message": {"content": '{"expected": "kb/a", "acceptable": [], "razon": "ok"}'}}],
                    "usage": {"prompt_tokens": 10, "completion_tokens": 5}}

        d, err = jz.kimi(paq, "k", "kimi-k3", http=http)
        self.assertIsNone(err)
        self.assertEqual((d["expected"], d["usage"]["prompt_tokens"]), ("kb/a", 10))
        self.assertEqual(llamadas, [(jz.BASE + "/chat/completions", "kimi-k3", "json_schema")])
        self.assertEqual(jz.cuerpo_peticion(paq, "m")["temperature"], 0)


class TestAcuerdo(unittest.TestCase):
    def j(self, e, a=(), r="r"):
        return {"expected": e, "acceptable": list(a), "razon": r}

    def test_acuerdo_fila_y_fusiona(self):
        self.assertEqual(ac.acuerdo_fila(self.j("a"), self.j("a")), "estricto")
        self.assertEqual(ac.acuerdo_fila(self.j(None), self.j(None)), "estricto")
        self.assertEqual(ac.acuerdo_fila(self.j("a", ["b"]), self.j("b")), "lenient")
        self.assertIsNone(ac.acuerdo_fila(self.j("a"), self.j("b")))
        self.assertIsNone(ac.acuerdo_fila(self.j(None), self.j("b")))
        self.assertEqual(ac.fusiona(self.j("a", ["b", "c"]), self.j("a", ["c", "d"]), "estricto"), ("a", ["c"]))
        self.assertEqual(ac.fusiona(self.j("a", ["b"]), self.j("b", ["a", "c"]), "lenient"), ("a", ["b"]))
        self.assertEqual(ac.fusiona(self.j("b", ["a"]), self.j("a", ["b"]), "lenient"), ("b", ["a"]))

    def test_kappa(self):
        self.assertAlmostEqual(ac.kappa([("a", "a"), ("b", "b"), ("a", "b"), ("b", "a")]), 0.0)
        self.assertEqual(ac.kappa([("a", "a"), ("b", "b")]), 1.0)
        self.assertEqual(ac.kappa([(None, None)] * 4), 1.0)
        # 2x2 clásico: po=0.8, pe=0.5 -> 0.6
        pares = [("x", "x")] * 4 + [("y", "y")] * 4 + [("x", "y"), ("y", "x")]
        self.assertAlmostEqual(ac.kappa(pares), 0.6)

    def test_construye(self):
        cands = [{"id": "c1", "query": "q1", "source": "prompt", "candidatos": ["kb/a", "kb/b"]},
                 {"id": "c2", "query": "q2", "source": "negativo", "candidatos": ["kb/a"]},
                 {"id": "c3", "query": "q3", "source": "archive", "candidatos": ["kb/log/x", "kb/archive/log/x-1"]},
                 {"id": "c4", "query": "q4", "source": "hard", "candidatos": ["kb/a", "kb/b"]},
                 {"id": "c5", "query": "q5", "source": "hard", "candidatos": ["kb/a"]}]
        fab = {"c1": self.j("kb/a"), "c2": self.j(None), "c3": self.j("kb/log/x"), "c4": self.j("kb/a"), "c5": self.j("kb/a")}
        kim = {"c1": self.j("kb/a", ["kb/b"]), "c2": self.j(None), "c3": self.j("kb/log/x"), "c4": self.j("kb/b")}
        gold, desc, inf, k, po = ac.construye(cands, fab, kim, {"kb/a": "q1 texto"}, 0.6, 0.7)
        self.assertEqual([g["id"] for g in gold], ["c1", "c2"])
        self.assertEqual(gold[0]["acceptable_permalinks"], [])
        self.assertEqual(sorted(d["motivo"] for d in desc), ["archive sin expected en archive/", "desacuerdo", "sin juicio de ambos"])
        self.assertIn("| negativo | 1 | 1 | 1.000 |", inf)
        self.assertIn("auditoría del sesgo léxico", inf)
        self.assertNotIn("q1", inf)
        self.assertNotIn("kb/a", inf)
        self.assertEqual(po, 0.75)
        self.assertIn("solape_lexico=", gold[0]["notes"])

    def test_solape_lexico(self):
        self.assertEqual(ac.solape_lexico("fusión rrf combsum", "la fusion por rrf"), 0.5)
        self.assertEqual(ac.solape_lexico("a b", "nada"), 0.0)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Correr y verlo fallar**

Run: `python3 -m unittest evals/retrieval-heldout/harness/test_gold_j.py`
Expected: `ModuleNotFoundError: No module named 'acuerdo'`.

- [ ] **Step 3: Implementar los tres scripts**

`evals/retrieval-heldout/harness/limpia_agent_search.py`:

```python
#!/usr/bin/env python3
"""Limpieza del estrato `agent-search` del gold J (borrador §3). Entrada: un
fichero de texto con una línea por comando `exo|kbx search|targets` minado de
`~/.claude/projects/**/*.jsonl` (extracción autorizada por Paul el
2026-09-19; el minado con procedencia lo hace pool.pool_comandos).
Criterio escrito, en este orden: (1) la línea parsea con
pool.query_de_comando (shlex; la query es el positional); (2) ni la línea ni
la query contienen marcadores de doc/plan/test: `< > … * [ ] \\` | { } $`,
`...`, `jq`; (3) query normalizada de ≥ 2 caracteres; (4) anti-fuga: forma
normalizada distinta y Jaccard < 0,8 frente a TODAS las listas de exclusión
(las 55 y las 147 de C); (5) dedupe por forma normalizada (gana la primera).
Salida: JSONL {"query","source":"agent-search"} y recuento JSON a stdout.
Uso: limpia_agent_search.py --raw F --excluye A [--excluye B] --out F.jsonl
"""
import argparse
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from pool import jaccard, normaliza, query_de_comando  # noqa: E402

MARCADORES = re.compile(r"[<>…*\[\]`|{}$]|\.\.\.|\bjq\b")


def limpia(lineas, excluidas):
    desc = {"no_parsea": 0, "marcador": 0, "vacia": 0, "dup-exclusion": 0, "dup-pool": 0}
    vistos, out = set(), []
    for l in lineas:
        if MARCADORES.search(l):
            desc["marcador"] += 1
            continue
        q = query_de_comando(l)
        if not q:
            desc["no_parsea"] += 1
            continue
        n = normaliza(q)
        if len(n) < 2:
            desc["vacia"] += 1
            continue
        if any(n == m or jaccard(n, m) >= 0.8 for m in excluidas):
            desc["dup-exclusion"] += 1
            continue
        if n in vistos:
            desc["dup-pool"] += 1
            continue
        vistos.add(n)
        out.append({"query": q.strip(), "source": "agent-search"})
    return out, desc


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--raw", required=True)
    ap.add_argument("--excluye", action="append", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    lineas = [l.rstrip("\n") for l in open(a.raw, encoding="utf-8") if l.strip()]
    excl = [normaliza(json.loads(l)["query"]) for r in a.excluye for l in open(r, encoding="utf-8") if l.strip()]
    out, desc = limpia(lineas, excl)
    with open(a.out, "w", encoding="utf-8") as fh:
        for c in out:
            fh.write(json.dumps(c, ensure_ascii=False) + "\n")
    print(json.dumps({"entrada": len(lineas), "salida": len(out), "descartes": desc}, ensure_ascii=False))


if __name__ == "__main__":
    main()
```

`evals/retrieval-heldout/harness/juez.py`:

```python
#!/usr/bin/env python3
"""Juez de relevancia para el gold J (borrador §3): dos jueces ciegos e
independientes, fable (subagente de la fábrica) y Kimi (Moonshot, otra
familia de modelo). Tres funciones puras y dos con red:
  paquete(fila, rutas)  -> lo que ve un juez: query + candidatos (permalink,
                           título, primeros MAX_CHARS caracteres del cuerpo).
                           Nunca etiquetas, rankings ni el otro juez.
  parsea(resp, cands)   -> valida el JSON del juez contra SCHEMA (expected ∈
                           candidatos ∪ {null}; acceptable ⊆ candidatos, ≤2).
  kimi(paq, key, model) -> POST {BASE}/chat/completions (OpenAI-compatible,
                           response_format json_schema estricto, urllib).
  modelos(env_keys)     -> GET {BASE}/models para fijar el modelo en §10.
La key se lee en runtime de --env-keys (variable kimi_api_key, fichero
gitignored de otro repo) y NUNCA se imprime ni se escribe. Envío de datos a
Moonshot autorizado por Paul el 2026-09-19 (propuesta.md §7).
Uso: juez.py paquetes --candidatos C --snap KB --out P.jsonl
     juez.py modelos --env-keys F
     juez.py kimi --paquetes P.jsonl --env-keys F --model M --out R.jsonl [--max N]
     juez.py valida --respuestas R.jsonl --candidatos C
"""
import argparse
import json
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

BASE = "https://api.moonshot.ai/v1"
MAX_CHARS = 4000
SISTEMA = (
    "Eres un juez de relevancia para un buscador de notas personales en castellano. "
    "Recibes UNA consulta y hasta cinco notas candidatas (permalink, título y el inicio del cuerpo). "
    "Decide qué nota querría recuperar un agente de programación que lanzó esa consulta SOLA, sin más contexto: "
    "expected = el permalink de la mejor nota, o null si ninguna responde a la consulta (una consulta operativa "
    "tipo 'mergea y sigue', o un tema que ninguna candidata cubre, es null). acceptable = hasta dos permalinks más "
    "que servirían razonablemente igual (canon frente a bitácora: consulta genérica/estado → canon; consulta "
    "histórica/fechada → bitácora), o lista vacía. Si la consulta pide un hecho concreto y la nota que lo contiene "
    "es una rotación archivada, elige la archivada. No premies que la nota repita literalmente las palabras de la "
    "consulta: premia que la nota RESPONDA. Responde solo con el JSON pedido."
)
SCHEMA = {
    "type": "object",
    "properties": {
        "expected": {"type": ["string", "null"]},
        "acceptable": {"type": "array", "items": {"type": "string"}, "maxItems": 2},
        "razon": {"type": "string"},
    },
    "required": ["expected", "acceptable", "razon"],
    "additionalProperties": False,
}


def rutas_por_permalink(snap):
    out = {}
    raiz = Path(snap)
    for p in raiz.rglob("*.md"):
        if any(x.startswith(".") for x in p.relative_to(raiz).parts):
            continue
        lineas = p.read_text(encoding="utf-8", errors="ignore").splitlines()
        if not lineas or lineas[0].strip() != "---":
            continue
        for l in lineas[1:]:
            if l.strip() == "---":
                break
            if l.startswith("permalink:"):
                out[l.split(":", 1)[1].strip().strip("'\"")] = str(p)
                break
    return out


def nota(permalink, rutas):
    ruta = rutas.get(permalink)
    if ruta is None:
        return {"permalink": permalink, "titulo": "", "cuerpo": "(nota no encontrada en el snapshot)"}
    texto = Path(ruta).read_text(encoding="utf-8", errors="ignore")
    titulo, cuerpo = "", texto
    if texto.startswith("---"):
        partes = texto.split("---", 2)
        if len(partes) == 3:
            for l in partes[1].splitlines():
                if l.startswith("title:"):
                    titulo = l.split(":", 1)[1].strip().strip("'\"")
            cuerpo = partes[2]
    return {"permalink": permalink, "titulo": titulo, "cuerpo": cuerpo.strip()[:MAX_CHARS]}


def paquete(fila, rutas):
    cands = [nota(p, rutas) for p in fila["candidatos"][:5]]
    texto = [f"CONSULTA: {fila['query']}", "", f"CANDIDATAS ({len(cands)}):"]
    for i, c in enumerate(cands, start=1):
        texto += ["", f"[{i}] permalink: {c['permalink']}", f"título: {c['titulo']}", "cuerpo:", c["cuerpo"]]
    if not cands:
        texto.append("(sin candidatas: expected debe ser null)")
    return {"id": fila["id"], "source": fila["source"], "candidatos": [c["permalink"] for c in cands], "texto": "\n".join(texto)}


def parsea(respuesta, candidatos):
    """(dict válido, None) o (None, error)."""
    try:
        d = json.loads(respuesta) if isinstance(respuesta, str) else respuesta
    except json.JSONDecodeError as e:
        return None, f"json: {e}"
    if not isinstance(d, dict) or set(d) != {"expected", "acceptable", "razon"}:
        return None, "claves"
    exp, acc = d["expected"], d["acceptable"]
    if exp is not None and exp not in candidatos:
        return None, f"expected fuera de candidatos: {exp}"
    if not isinstance(acc, list) or len(acc) > 2 or len(set(acc)) != len(acc) or any(a not in candidatos or a == exp for a in acc):
        return None, "acceptable inválido"
    if exp is None and acc:
        return None, "null con acceptable"
    return {"expected": exp, "acceptable": list(acc), "razon": str(d["razon"])[:500]}, None


def _key(env_keys):
    for l in Path(env_keys).read_text(encoding="utf-8").splitlines():
        if l.startswith("kimi_api_key="):
            return l.split("=", 1)[1].strip().strip("'\"")
    raise SystemExit("kimi_api_key no encontrada en --env-keys")


def _http(url, key, cuerpo=None, timeout=120):
    data = None if cuerpo is None else json.dumps(cuerpo).encode("utf-8")
    req = urllib.request.Request(url, data=data, method="POST" if data else "GET",
                                 headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read().decode("utf-8"))


def modelos(env_keys):
    return sorted(m["id"] for m in _http(f"{BASE}/models", _key(env_keys), timeout=30)["data"])


def cuerpo_peticion(paq, model):
    return {
        "model": model,
        "temperature": 0,
        "messages": [{"role": "system", "content": SISTEMA}, {"role": "user", "content": paq["texto"]}],
        "response_format": {"type": "json_schema", "json_schema": {"name": "juicio", "schema": SCHEMA, "strict": True}},
    }


def kimi(paq, key, model, reintentos=3, http=_http):
    ultimo = None
    for i in range(reintentos):
        try:
            r = http(f"{BASE}/chat/completions", key, cuerpo_peticion(paq, model))
            d, err = parsea(r["choices"][0]["message"]["content"], paq["candidatos"])
            if d is not None:
                uso = r.get("usage", {})
                return {"id": paq["id"], **d, "usage": {k: uso.get(k) for k in ("prompt_tokens", "completion_tokens")}}, None
            ultimo = err
        except (urllib.error.URLError, KeyError, TimeoutError, json.JSONDecodeError) as e:
            ultimo = f"{type(e).__name__}: {str(e)[:120]}"
        time.sleep(2 ** i)
    return None, ultimo


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    a_p = sub.add_parser("paquetes")
    a_p.add_argument("--candidatos", required=True)
    a_p.add_argument("--snap", required=True)
    a_p.add_argument("--out", required=True)
    a_m = sub.add_parser("modelos")
    a_m.add_argument("--env-keys", required=True)
    a_k = sub.add_parser("kimi")
    a_k.add_argument("--paquetes", required=True)
    a_k.add_argument("--env-keys", required=True)
    a_k.add_argument("--model", required=True)
    a_k.add_argument("--out", required=True)
    a_k.add_argument("--max", type=int, default=0)
    a_v = sub.add_parser("valida")
    a_v.add_argument("--respuestas", required=True)
    a_v.add_argument("--candidatos", required=True)
    a = ap.parse_args()
    if a.cmd == "paquetes":
        rutas = rutas_por_permalink(a.snap)
        n = 0
        with open(a.out, "w", encoding="utf-8") as fh:
            for l in open(a.candidatos, encoding="utf-8"):
                if l.strip():
                    fh.write(json.dumps(paquete(json.loads(l), rutas), ensure_ascii=False) + "\n")
                    n += 1
        print(json.dumps({"paquetes": n}))
    elif a.cmd == "modelos":
        print(json.dumps(modelos(a.env_keys)))
    elif a.cmd == "kimi":
        key = _key(a.env_keys)
        hechos = {json.loads(l)["id"] for l in open(a.out, encoding="utf-8") if l.strip()} if Path(a.out).exists() else set()
        errores, n, tok = [], 0, [0, 0]
        with open(a.out, "a", encoding="utf-8") as fh:
            for l in open(a.paquetes, encoding="utf-8"):
                if not l.strip():
                    continue
                paq = json.loads(l)
                if paq["id"] in hechos or (a.max and n >= a.max):
                    continue
                d, err = kimi(paq, key, a.model)
                n += 1
                if d is None:
                    errores.append({"id": paq["id"], "error": err})
                    continue
                tok[0] += d["usage"]["prompt_tokens"] or 0
                tok[1] += d["usage"]["completion_tokens"] or 0
                fh.write(json.dumps(d, ensure_ascii=False) + "\n")
                fh.flush()
                print(f"[{n}] {paq['id']}", file=sys.stderr)
        print(json.dumps({"llamadas": n, "ok": n - len(errores), "errores": errores, "prompt_tokens": tok[0], "completion_tokens": tok[1], "model": a.model}, ensure_ascii=False))
        sys.exit(1 if errores else 0)
    else:
        cands = {json.loads(l)["id"]: json.loads(l)["candidatos"] for l in open(a.candidatos, encoding="utf-8") if l.strip()}
        filas = [json.loads(l) for l in open(a.respuestas, encoding="utf-8") if l.strip()]
        malas = []
        for r in filas:
            _, err = parsea({k: r.get(k) for k in ("expected", "acceptable", "razon")}, cands.get(r["id"], []))
            if err:
                malas.append(f"{r['id']}: {err}")
        for m in malas:
            print(m, file=sys.stderr)
        print(json.dumps({"respuestas": len(filas), "invalidas": len(malas)}))
        sys.exit(1 if malas else 0)


if __name__ == "__main__":
    main()
```

`evals/retrieval-heldout/harness/acuerdo.py`:

```python
#!/usr/bin/env python3
"""Acuerdo entre jueces y construcción del gold J (borrador §3, §11).
Entrada: candidatos (id, query, source, candidatos) y dos ficheros de
respuestas (fable, kimi) con {id, expected, acceptable, razon}.
  acuerdo_fila(a, b): "estricto" (mismo expected, null incluido), "lenient"
    (el expected de uno está en los acceptable del otro) o None.
  kappa(pares): κ de Cohen (Cohen 1960) sobre el expected estricto, con cada
    permalink y null como categoría nominal.
  fusiona(a, b, tipo): expected = el común (estricto) o el que el otro juez
    admite como aceptable (lenient); acceptable = (A_f ∩ A_k) ∪ {el expected
    no elegido en un acuerdo lenient}, ≤ 2, sin el expected.
  construye(...): una fila entra al gold solo con acuerdo; los desacuerdos SE
    DESCARTAN (sin tercer juez, borrador §3) y se listan en privado. Reglas de
    estrato: negativo exige null; archive exige expected bajo archive/.
  solape_lexico(query, texto): fracción de tokens de la query (≥ 4 letras)
    presentes en la nota; auditoría del sesgo léxico de los jueces (acordadas
    frente a descartadas).
Salida pública (informe): solo recuentos, κ por estrato y la auditoría.
Exit 0 si el suelo pre-registrado pasa, 2 si no (J PARA).
Uso: acuerdo.py --candidatos C --fable F --kimi K [--snap KB] --gold-out G
     --descartes-out D --informe-out I.md --kappa-min 0.60 --po-min 0.70
"""
import argparse
import json
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from pool import normaliza  # noqa: E402


def acuerdo_fila(a, b):
    if a["expected"] == b["expected"]:
        return "estricto"
    if a["expected"] is not None and a["expected"] in b["acceptable"]:
        return "lenient"
    if b["expected"] is not None and b["expected"] in a["acceptable"]:
        return "lenient"
    return None


def kappa(pares):
    n = len(pares)
    if n == 0:
        return 0.0
    po = sum(1 for a, b in pares if a == b) / n
    ca, cb = Counter(a for a, _ in pares), Counter(b for _, b in pares)
    pe = sum(ca[k] * cb[k] for k in set(ca) | set(cb)) / (n * n)
    return 1.0 if pe == 1.0 else (po - pe) / (1 - pe)


def fusiona(a, b, tipo):
    comunes = [x for x in a["acceptable"] if x in b["acceptable"]]
    if tipo == "estricto":
        exp, extra = a["expected"], []
    elif a["expected"] is not None and a["expected"] in b["acceptable"]:
        exp, extra = a["expected"], ([b["expected"]] if b["expected"] else [])
    else:
        exp, extra = b["expected"], ([a["expected"]] if a["expected"] else [])
    acc = []
    for x in extra + comunes:
        if x != exp and x not in acc:
            acc.append(x)
    return exp, acc[:2]


def solape_lexico(query, texto):
    toks = {t for t in normaliza(query).split() if len(t) >= 4}
    if not toks:
        return 0.0
    t = normaliza(texto)
    return sum(1 for x in toks if x in t) / len(toks)


def construye(cands, fab, kim, textos, kappa_min, po_min):
    por_src, gold, desc = {}, [], []
    for c in cands:
        i = c["id"]
        if i not in fab or i not in kim:
            desc.append({"id": i, "motivo": "sin juicio de ambos"})
            continue
        tipo = acuerdo_fila(fab[i], kim[i])
        por_src.setdefault(c["source"], []).append((fab[i]["expected"], kim[i]["expected"], tipo))
        if tipo is None:
            desc.append({"id": i, "motivo": "desacuerdo", "fable": fab[i]["expected"], "kimi": kim[i]["expected"]})
            continue
        exp, acc = fusiona(fab[i], kim[i], tipo)
        if c["source"] == "negativo" and exp is not None:
            desc.append({"id": i, "motivo": "negativo con nota", "fable": fab[i]["expected"], "kimi": kim[i]["expected"]})
            continue
        if c["source"] == "archive" and (exp is None or "/archive/" not in exp):
            desc.append({"id": i, "motivo": "archive sin expected en archive/", "fable": fab[i]["expected"], "kimi": kim[i]["expected"]})
            continue
        notes = f"acuerdo {tipo}; fable: {fab[i]['razon']} | kimi: {kim[i]['razon']}"
        if acc:
            notes += " | aceptable: admitido por ambos jueces"
        gold.append({"id": i, "query": c["query"], "source": c["source"], "expected_permalink": exp,
                     "acceptable_permalinks": acc, "notes": notes})
    todos = [p for v in por_src.values() for p in v]
    k_total = kappa([(a, b) for a, b, _ in todos])
    po_total = sum(1 for _, _, t in todos if t) / max(1, len(todos))
    pasa = k_total >= kappa_min and po_total >= po_min
    L = ["# Acuerdo entre jueces — gold J (fable × Kimi, ciegos)", ""]
    L.append(f"- filas juzgadas por ambos: {len(todos)} · acuerdo (estricto+lenient): {sum(1 for _, _, t in todos if t)} ({po_total:.3f}) · estricto: {sum(1 for _, _, t in todos if t == 'estricto')} · κ estricto: {k_total:.3f}")
    L.append(f"- suelo pre-registrado: κ ≥ {kappa_min} y acuerdo ≥ {po_min} → {'PASA' if pasa else 'NO PASA: J PARA'}")
    L += ["", "| estrato | juzgadas | acuerdo | p_o | κ | entran al gold | no nulas |", "|---|---|---|---|---|---|---|"]
    entradas = Counter(g["source"] for g in gold)
    nonulas = Counter(g["source"] for g in gold if g["expected_permalink"])
    for s, v in sorted(por_src.items()):
        ac = sum(1 for _, _, t in v if t)
        L.append(f"| {s} | {len(v)} | {ac} | {ac / len(v):.3f} | {kappa([(a, b) for a, b, _ in v]):.3f} | {entradas[s]} | {nonulas[s]} |")
    if textos is not None:
        por_id = {c["id"]: c for c in cands}

        def sol(i):
            exp = fab[i]["expected"] or kim[i]["expected"]
            return solape_lexico(por_id[i]["query"], textos.get(exp, "")) if exp else None

        s_ok = [x for x in (sol(g["id"]) for g in gold if g["expected_permalink"]) if x is not None]
        s_no = [x for x in (sol(d["id"]) for d in desc if d["motivo"] == "desacuerdo") if x is not None]
        med = lambda v: round(sorted(v)[len(v) // 2], 2) if v else None  # noqa: E731
        L += ["", "## auditoría del sesgo léxico (fracción de tokens de la query ≥4 letras presentes en la nota esperada)", "",
              f"- filas acordadas no nulas: n={len(s_ok)} · mediana {med(s_ok)} · con solape ≥ 0,5: {sum(1 for x in s_ok if x >= 0.5)}",
              f"- desacuerdos con alguna nota propuesta: n={len(s_no)} · mediana {med(s_no)} · con solape ≥ 0,5: {sum(1 for x in s_no if x >= 0.5)}",
              "- lectura: si los desacuerdos se concentran en solape bajo, los jueces acuerdan sobre todo donde la query repite la nota (sesgo léxico); el subconjunto «léxicamente difícil» (solape < 0,5) de las no nulas es el guard de D-A (borrador §6)."]
        for g in gold:
            if g["expected_permalink"]:
                g["notes"] += f" | solape_lexico={sol(g['id']):.2f}"
    L += ["", f"- descartes: {len(desc)} (" + ", ".join(f"{k}={v}" for k, v in sorted(Counter(d['motivo'] for d in desc).items())) + ")"]
    return gold, desc, "\n".join(L) + "\n", k_total, po_total


def _carga(ruta):
    return {json.loads(l)["id"]: json.loads(l) for l in open(ruta, encoding="utf-8") if l.strip()}


def main():
    ap = argparse.ArgumentParser()
    for k in ("candidatos", "fable", "kimi", "gold-out", "descartes-out", "informe-out"):
        ap.add_argument(f"--{k}", required=True)
    ap.add_argument("--snap")
    ap.add_argument("--kappa-min", type=float, default=0.60)
    ap.add_argument("--po-min", type=float, default=0.70)
    a = ap.parse_args()
    cands = [json.loads(l) for l in open(a.candidatos, encoding="utf-8") if l.strip()]
    textos = None
    if a.snap:
        from juez import rutas_por_permalink
        textos = {p: Path(r).read_text(encoding="utf-8", errors="ignore") for p, r in rutas_por_permalink(a.snap).items()}
    gold, desc, inf, k, po = construye(cands, _carga(a.fable), _carga(a.kimi), textos, a.kappa_min, a.po_min)
    for n, g in enumerate(gold, start=1):
        g["id"] = f"j{n:03d}"
    with open(a.gold_out, "w", encoding="utf-8") as fh:
        for g in gold:
            fh.write(json.dumps(g, ensure_ascii=False) + "\n")
    with open(a.descartes_out, "w", encoding="utf-8") as fh:
        for d in desc:
            fh.write(json.dumps(d, ensure_ascii=False) + "\n")
    Path(a.informe_out).write_text(inf, encoding="utf-8")
    pasa = k >= a.kappa_min and po >= a.po_min
    print(json.dumps({"gold": len(gold), "no_nulas": sum(1 for g in gold if g["expected_permalink"]), "descartes": len(desc), "kappa": round(k, 3), "po": round(po, 3), "pasa": pasa}))
    sys.exit(0 if pasa else 2)


if __name__ == "__main__":
    main()
```

- [ ] **Step 4: Verde**

Run: `python3 -m unittest -v evals/retrieval-heldout/harness/test_gold_j.py 2>&1 | tail -3 && python3 -m unittest evals/retrieval-heldout/harness/test_harness.py 2>&1 | tail -1`
Expected: `Ran 7 tests … OK` (corridos así el 2026-09-19) y `OK` (40).

- [ ] **Step 5: Commit**

```bash
grep -rn "kimi_api_key=" evals/retrieval-heldout/harness/*.py | grep -v 'startswith("kimi_api_key=")' ; echo "sin key literal: $?"
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 add evals/retrieval-heldout/harness/limpia_agent_search.py evals/retrieval-heldout/harness/juez.py evals/retrieval-heldout/harness/acuerdo.py evals/retrieval-heldout/harness/test_gold_j.py
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 commit -m "eval(j): pipeline de gold agentico — limpieza agent-search, juez Kimi/fable, acuerdo kappa"
```
Expected: el `grep` no encuentra nada (`sin key literal: 1`).

---

### Task 5: Pre-registro borrador — review adversarial (antes de juzgar; las decisiones D-J3..D-J11 ya están firmadas)

**Lane:** diseño (spec fábrica: 1 review adversarial del pre-registro de J). Sin gate humano salvo objeción del revisor.
**Oráculo:** el borrador está commiteado en la rama con la cabecera `Estado: BORRADOR`, §9 con las firmas del 2026-09-19 y la fuente `propuesta.md` §7, y `$PRIV_J/review-preregistro.md` existe. Si el revisor objeta una firma con cita, existe además una entrada en `.superpowers/fabrica/pendiente-paul.md` que la reproduce literal.
**Depende de:** Task 2 (la lectura del T0 puede tocar el tope df de F1 en §4 una vez, cláusula de D-J8). **Debe terminar antes de la Task 7** (los jueces no corren con un pre-registro sin revisar).

**Files:**
- Modify: `docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md` (solo §4 tope df con evidencia del T0, y erratas/ambigüedades que el review encuentre; cada cambio, una línea `> Enmienda <fecha>:` bajo la cabecera)
- Modify (gitignored): `.superpowers/fabrica/ledger.md`; `.superpowers/fabrica/pendiente-paul.md` **solo** si el revisor objeta una firma

**Interfaces:**
- Consumes: el borrador tal como está en la rama (§9 firmado); `evals/retrieval-heldout/verdict/diagnostico-55.md`; la tabla «Decisiones firmadas por Paul 2026-09-19» de este plan.
- Produces: borrador revisado y commiteado; `$PRIV_J/review-preregistro.md`.

- [ ] **Step 1: Review adversarial del borrador**

Despachar un subagente **fable** fresco (u **opus** si la reserva está agotada) con este brief literal:

"Eres el revisor adversarial del pre-registro borrador `docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md`. Léelo entero, más `docs/superpowers/plans/2026-09-13-campana-c-preregistro.md` (precedente) y `evals/retrieval-heldout/verdict/c-verdict.md` §3 y §8 (potencia y erratas de C). Busca: (1) reglas de §6 que se puedan leer de dos maneras; (2) brazos de §4 cuya definición no sea computable sin decisiones humanas; (3) cualquier lugar donde el resultado dependa de elegir entre celdas a la vista de los datos; (4) errores en la tabla de potencia de §7 (recomputa al menos las filas «igual, disc. 10 %» y «mejor 8 pp» con N=120 y NETO ≥ 4 con un script propio: multinomial exacta sobre (ARREGLA, ROMPE)); (5) kill-criteria de §11 que falten para que «A0 se queda» y «J PARA por acuerdo insuficiente» sean veredictos válidos; (6) referencias de §7 y §3 marcadas «de memoria» (Carterette 2012, Boytsov 2013, Sakai 2016, Maurer 1995, Westfall & Krishen 2001, Landis & Koch 1977, Feinstein & Cicchetti 1990, Thomas et al. 2024, Faggioli et al. 2023, Clarke & Dietz 2024, Soboroff 2024, Alaofi et al. 2024): comprueba en la web si existen tal cual y anota DOI/arXiv o «no encontrada»; (7) el diseño de jueces de §3: ¿hay alguna forma en que un juez vea el ranking de un brazo o la etiqueta del otro? ¿el suelo κ ≥ 0,60 ∧ p_o ≥ 0,70 es computable con `acuerdo.py` tal como está?; (8) las decisiones firmadas de §9 (fuente `propuesta.md` §7): están cerradas, NO las reabras por preferencia; solo si encuentras un error de hecho o de cálculo que invalide la base con la que se firmó una, escríbelo como OBJECIÓN numerada con cita textual y el dato que la contradice. PROHIBIDO proponer brazos nuevos o cambiar los valores únicos de §4. Escribe `$PRIV_J/review-preregistro.md` con hallazgos numerados, cada uno con cita textual y la corrección propuesta."

- [ ] **Step 2: Aplicar solo erratas y ambigüedades**

Aplicar del review únicamente: erratas numéricas confirmadas por el script del revisor, ambigüedades de redacción en §6, kill-criteria que falten en §11, y el estado de cada referencia (DOI o «retirada»). Cada cambio, una línea bajo la cabecera del borrador:
`> Enmienda 2026-09-XX (review adversarial, hallazgo N): <qué cambió, en una frase>.`
Si el T0 (Task 2, sección Lectura) propone otro tope df para F1 (cláusula de D-J8), cambiar el único valor en §4 y añadir la línea `> Enmienda … (T0, cláusula D-J8): tope df de F1 0,25 → X, evidencia: diagnostico-55.md línea «no nulas con ≥1 token raro …»`.

- [ ] **Step 3: Objeciones a firmas (solo si las hay)**

Si el review contiene una OBJECIÓN a una decisión firmada: no se toca §9. Se añade a `.superpowers/fabrica/pendiente-paul.md`, con `date -Iseconds`, un bloque `## Campaña J — objeción del revisor a D-Jn (bloquea: Task 8)` con la objeción literal (cita + dato) y las dos salidas posibles: `D-Jn SE MANTIENE` o `D-Jn CAMBIA A <opción>`. Paul responde con una de las dos; si cambia, la Task 8 sustituye esa línea en §9 citando la respuesta. Sin objeciones, este paso no escribe nada.

- [ ] **Step 4: Commit del borrador**

```bash
head -3 docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md | grep -c "Estado: BORRADOR"
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 add docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 commit -m "docs(j): pre-registro borrador tras review adversarial (sin congelar)"
```
Expected: `1`; commit con hash. Registrar en el ledger `preregistro_borrador: <hash>`.

---

### Task 6: Generación del gold — snapshot, queries por estrato, checks mecánicos, candidatos y paquetes

**Lane:** diseño (construye el material que juzgan las dos familias de modelo). 0 h de Paul.
**Oráculo:** `$PRIV_J/paquetes.jsonl` existe con una línea por fila de `$PRIV_J/candidatos.jsonl`; `python3 evals/retrieval-heldout/harness/valida_gold.py --gold evals/retrieval-heldout/gold-j/plantilla.jsonl --kb "$PRIV_J/kb-snap" --in-sample "$PRIV_C/in-sample-55.jsonl" --in-sample "$PRIV_C/gold.jsonl"` sale con exit 1 y **exactamente** 5 líneas de error, todas `… inexistente en el snapshot` (plantilla con permalinks `kb/…` ficticios a propósito; verificado el 2026-09-19); el recuento de `agent-search` limpio está anotado en el ledger.
**Depende de:** Tasks 3 y 4.

**Files:**
- Create: `evals/retrieval-heldout/gold-j/README.md` (cómo se construyó el gold; sin texto de queries)
- Create: `evals/retrieval-heldout/gold-j/plantilla.jsonl`
- Privado: `$PRIV_J/kb-snap/`, `$PRIV_J/kb-snap.commit`, `$PRIV_J/pool.jsonl`, `$PRIV_J/muestra-prompt.jsonl`, `$PRIV_J/agent-search.jsonl`, `$PRIV_J/notas-hard.txt`, `$PRIV_J/notas-archive.txt`, `$PRIV_J/generadas.jsonl`, `$PRIV_J/queries.jsonl`, `$PRIV_J/candidatos.jsonl`, `$PRIV_J/paquetes.jsonl`

**Interfaces:**
- Consumes: `pool.py`, `limpia_agent_search.py`, `juez.py paquetes` (Tasks 3–4); `~/.exo/priv-j/agent-search-raw.txt` (extracción autorizada por Paul; se RE-EXTRAE en el Step 3 parseando el JSON — la de 221 líneas de la sesión de planificación estaba truncada).
- Produces: `$PRIV_J/queries.jsonl` con `{"id": "q001…", "query", "source", "topic_terms"?: [str], "author_expected"?: str}`; `$PRIV_J/candidatos.jsonl` con `{"id", "query", "source", "candidatos": [permalink ≤5]}`; `$PRIV_J/paquetes.jsonl` (salida de `juez.py paquetes`). Los ids `qNNN` son los que usan los dos jueces; `acuerdo.py` renumera a `jNNN` al construir el gold.

- [ ] **Step 1: Directorio privado y snapshot inmutable de la KB (lectura; sin escribir en la KB)**

```bash
export PRIV_C=~/.local/share/exo-evals/c-heldout PRIV_J=~/.local/share/exo-evals/j-heldout
mkdir -p "$PRIV_J" && chmod 700 "$PRIV_J"
S_J=$(git -C "$HOME/Documentos/proyectos/wisdom-paul" rev-parse HEAD)
git clone --quiet --no-hardlinks "$HOME/Documentos/proyectos/wisdom-paul" "$PRIV_J/kb-snap"
git -C "$PRIV_J/kb-snap" checkout --quiet --detach "$S_J"
echo "$S_J" > "$PRIV_J/kb-snap.commit"; cat "$PRIV_J/kb-snap.commit"
ls "$PRIV_J/kb-snap/archive/log" | wc -l
```
Expected: un sha de 40 hex (`S_J` del borrador §10); ≈ 32 ficheros en `archive/log`.

- [ ] **Step 2: Estrato `prompt` (pool entero desde el 2026-09-13)**

```bash
python3 evals/retrieval-heldout/harness/pool.py --in-sample "$PRIV_C/in-sample-55.jsonl" --in-sample "$PRIV_C/gold.jsonl" --out-dir "$PRIV_J" --cuota prompt=0 --desde 2026-09-13T00:00:00Z --hasta "$(date -u +%Y-%m-%dT00:00:00Z)"
mv "$PRIV_J/muestra.jsonl" "$PRIV_J/muestra-prompt.jsonl"; wc -l < "$PRIV_J/muestra-prompt.jsonl"
```
Expected: JSON con `"pool": {"prompt": ≥113, "agent-search": 0}` (113 el 2026-09-19) y `muestra` = ese número.

- [ ] **Step 3: Estrato `agent-search` (re-extracción + limpieza con criterio escrito)**

**La extracción cruda de la sesión de planificación es defectuosa** (verificado
por el orquestador el 2026-09-19): se hizo con `grep -oE 'exo search [^"\\]{3,200}'`
sobre el JSONL, que corta en la comilla escapada `\"` — las queries entre
comillas (el caso normal) quedaron truncadas o fuera (44 de las 221 líneas
acaban en un flag, sin texto de query). Se re-extrae parseando el JSON: solo
bloques `tool_use` con `name == "Bash"` cuyo `input.command` contiene
`exo search`, uno por línea (los saltos de línea del comando se aplanan a
espacio), dedupe. Sobrescribe el fichero crudo; la interfaz de
`limpia_agent_search.py` no cambia.

```bash
python3 - <<'EOF'
import json, pathlib
raiz = pathlib.Path.home() / ".claude" / "projects"
vistos, n_ficheros = set(), 0
for f in sorted(raiz.rglob("*.jsonl")):
    n_ficheros += 1
    with f.open(encoding="utf-8", errors="replace") as fh:
        for linea in fh:
            if "exo search" not in linea:
                continue
            try:
                ev = json.loads(linea)
            except json.JSONDecodeError:
                continue
            contenido = (ev.get("message") or {}).get("content")
            if not isinstance(contenido, list):
                continue
            for b in contenido:
                if isinstance(b, dict) and b.get("type") == "tool_use" and b.get("name") == "Bash":
                    cmd = str((b.get("input") or {}).get("command", ""))
                    if "exo search" in cmd:
                        vistos.add(" ".join(cmd.split()))
out = pathlib.Path.home() / ".exo" / "priv-j" / "agent-search-raw.txt"
out.parent.mkdir(parents=True, exist_ok=True)
out.write_text("".join(c + "\n" for c in sorted(vistos)), encoding="utf-8")
print(json.dumps({"ficheros": n_ficheros, "comandos": len(vistos)}))
EOF
```
Expected: JSON `{"ficheros": ≥442, "comandos": <m>}`. `<m>` es la `entrada` del
paso siguiente (sustituye al 221 de abajo, que era el recuento defectuoso). Un
comando puede ser compuesto (`cd … && exo search …`); `limpia_agent_search.py`
extrae el argumento de query de cada `exo search` que contenga — si su parser
solo admite líneas que empiezan por `exo search`, amplíalo en esta misma task
con test (TDD) antes de correr la limpieza.

```bash
python3 evals/retrieval-heldout/harness/limpia_agent_search.py --raw ~/.exo/priv-j/agent-search-raw.txt --excluye "$PRIV_C/in-sample-55.jsonl" --excluye "$PRIV_C/gold.jsonl" --out "$PRIV_J/agent-search.jsonl"
```
Expected: JSON `{"entrada": 221, "salida": <n>, "descartes": {...}}`. Cota a priori: 155 de las 221 líneas tienen forma de comando con flags (`exo search --t|--d|--j…`); las otras 66 son ejemplos de docs/planes/tests (`*)`, `[OP`, `<qu`, `sin `, `…`) que el filtro de marcadores tira. `<n>` esperado entre 60 y 120; se anota en el ledger y en el README del gold. **Si `<n>` < 30, el estrato se conserva pero se declara flojo en el borrador §11 (no decide por estrato: la decisión es sobre el total).** Ventana temporal: el fichero crudo no trae timestamps; la anti-fuga contra las 55 y las 147 de C sustituye a la ventana (todo lo que ya estaba en un gold anterior sale; lo demás es posterior o nunca usado).

- [ ] **Step 4: Notas para `hard` (30) y `archive` (20), al azar con semilla 20260919**

```bash
python3 - <<'EOF'
import os, random
from pathlib import Path
snap = Path(os.path.expanduser("~/.local/share/exo-evals/j-heldout/kb-snap"))
rel = lambda p: str(p.relative_to(snap))
todas = sorted(rel(p) for p in snap.rglob("*.md") if not any(x.startswith(".") for x in p.relative_to(snap).parts))
vivas = [n for n in todas if not n.startswith("archive/")]
archivo = [n for n in todas if n.startswith("archive/log/")]
random.Random(20260919).shuffle(vivas); random.Random(20260919).shuffle(archivo)
out = Path(os.path.expanduser("~/.local/share/exo-evals/j-heldout"))
(out / "notas-hard.txt").write_text("\n".join(vivas[:30]) + "\n", encoding="utf-8")
(out / "notas-archive.txt").write_text("\n".join(archivo[:20]) + "\n", encoding="utf-8")
print(len(vivas), "vivas →", 30, "|", len(archivo), "archive/log →", min(20, len(archivo)))
EOF
```
Expected: `≈107 vivas → 30 | 32 archive/log → 20`.

- [ ] **Step 5: Generar `hard`, `archive` y `negativo` (subagente sonnet fresco, filesystem-only)**

Brief literal:

"Generas queries para un gold de retrieval sobre la KB `$PRIV_J/kb-snap/` (solo Read/Grep/Glob; PROHIBIDO `exo`, `kbx`, basic-memory, `~/.exo/`, `evals/`, `reports/`, `~/.local/share/exo-evals/c-heldout/`). Tres lotes, en JSONL a `$PRIV_J/generadas.jsonl`, campos `{"query","source","author_expected"?,"topic_terms"?}`:
(A) `hard`, 30: una por nota de `$PRIV_J/notas-hard.txt`, en ese orden: una query en castellano que un agente haría para re-encontrar la nota SIN sus palabras clave literales ni su título; las 15 primeras en frase natural (16–30 palabras), las 15 siguientes en 2–5 palabras clave parafraseadas. `author_expected` = permalink del frontmatter.
(B) `archive`, 20: una por nota de `$PRIV_J/notas-archive.txt`: elige un hecho concreto y fechado que esté en esa nota archivada y NO en la bitácora viva del mismo nombre (compruébalo con grep en `log/`); escribe la pregunta que haría un agente que necesita ese hecho. `author_expected` = permalink de la nota archivada.
(C) `negativo`, 40: temas de los que la KB NO tiene nada (no «difíciles»: ausentes). Para cada uno, 3–5 `topic_terms` (sustantivos característicos del tema, en castellano y sin tildes) y una query natural de 6–15 palabras. Antes de escribir cada uno, `grep -ril <term>` sobre `$PRIV_J/kb-snap` para cada term: si alguno da ≥1 fichero, cambia de tema. Varía dominios (cocina, deporte, otras tecnologías que no aparecen en la KB, geografía, trámites…).
Sin `author_expected` en (C). No mires ningún fichero fuera del snapshot."

Control de transcript como en C: si aparece `exo `, `kbx ` o `evals/` en sus comandos, se descarta la salida entera y se repite (retries cap 2).

- [ ] **Step 6: Checks mecánicos de `negativo` y `archive`, y unión en `queries.jsonl`**

```bash
python3 - <<'EOF'
import json, os, subprocess
from pathlib import Path
P = Path(os.path.expanduser("~/.local/share/exo-evals/j-heldout")); snap = P / "kb-snap"
import sys; sys.path.insert(0, "evals/retrieval-heldout/harness")
from valida_gold import permalinks_snapshot
perms = permalinks_snapshot(str(snap))
gen = [json.loads(l) for l in open(P / "generadas.jsonl", encoding="utf-8") if l.strip()]
ok, desc = [], {"negativo_con_hits": 0, "archive_expected_malo": 0}
for g in gen:
    if g["source"] == "negativo":
        hits = [t for t in g["topic_terms"] if subprocess.run(["grep", "-ril", "--include=*.md", t, str(snap)], capture_output=True, text=True).stdout.strip()]
        if hits:
            desc["negativo_con_hits"] += 1; continue
    if g["source"] == "archive" and (g.get("author_expected") not in perms or "/archive/" not in g["author_expected"]):
        desc["archive_expected_malo"] += 1; continue
    ok.append(g)
filas = [json.loads(l) for l in open(P / "muestra-prompt.jsonl", encoding="utf-8") if l.strip()]
filas = [{"query": f["query"], "source": "prompt"} for f in filas]
filas += [json.loads(l) for l in open(P / "agent-search.jsonl", encoding="utf-8") if l.strip()]
filas += ok
with open(P / "queries.jsonl", "w", encoding="utf-8") as fh:
    for n, f in enumerate(filas, start=1):
        fh.write(json.dumps({"id": f"q{n:03d}", **f}, ensure_ascii=False) + "\n")
from collections import Counter
print(json.dumps({"total": len(filas), "por_estrato": dict(Counter(f["source"] for f in filas)), "descartes_mecanicos": desc}))
EOF
```
Expected: `total` ≈ 113 + `<n agent-search>` + 30 + ≤20 + ≤40; `negativo_con_hits` pequeño (el generador ya grepeó); `archive_expected_malo` = 0. Registrar el JSON en el ledger.

- [ ] **Step 7: Candidatos por fila (subagente sonnet fresco, distinto del generador, filesystem-only)**

Brief literal (se despacha en lotes de ≤ 60 filas; cada lote un subagente fresco):

"Para cada fila de `$PRIV_J/queries.jsonl` con id entre `<desde>` y `<hasta>`, propone hasta 5 notas candidatas de `$PRIV_J/kb-snap/` que podrían responder a la query (solo Read/Grep/Glob; PROHIBIDO `exo`, `kbx`, basic-memory, `~/.exo/`, `evals/`, `reports/`, `~/.local/share/exo-evals/c-heldout/`, y los campos `author_expected`/`topic_terms` de la fila: NO los leas, trabaja solo con `query`). Método obligatorio, para no depender solo de coincidencias literales: (1) al menos 2 candidatas por navegación temática desde `core/core-index.md` y los títulos de `log/`, `learnings/`, `projects/`, `backlog/`; (2) el resto por grep de términos y sinónimos; (3) si la query menciona un hecho fechado y hay una rotación en `archive/log/` de la bitácora relevante, incluye la rotación Y la bitácora viva. Si no encuentras ninguna nota plausible, lista vacía. Salida: `$PRIV_J/candidatos-<desde>-<hasta>.jsonl`, una línea por fila `{"id","query","source","candidatos":[permalinks del frontmatter, ≤5, de más a menos plausible]}`."

Unir: `cat "$PRIV_J"/candidatos-*.jsonl > "$PRIV_J/candidatos.jsonl"; wc -l < "$PRIV_J/candidatos.jsonl"` = `total` del Step 6. Comprobar que cada permalink existe: `python3 -c 'import json,sys; sys.path.insert(0,"evals/retrieval-heldout/harness"); from valida_gold import permalinks_snapshot; P=permalinks_snapshot(sys.argv[1]); bad=[(r["id"],p) for r in map(json.loads, open(sys.argv[2])) for p in r["candidatos"] if p not in P]; print(len(bad), bad[:5])' "$PRIV_J/kb-snap" "$PRIV_J/candidatos.jsonl"` → `0 []` (si no, el lote se devuelve al subagente).

- [ ] **Step 8: Paquetes de juez (lo único que ven los jueces)**

```bash
python3 evals/retrieval-heldout/harness/juez.py paquetes --candidatos "$PRIV_J/candidatos.jsonl" --snap "$PRIV_J/kb-snap" --out "$PRIV_J/paquetes.jsonl"
python3 -c 'import json,sys; L=[len(json.loads(l)["texto"]) for l in open(sys.argv[1]) if l.strip()]; L.sort(); print(len(L), "paquetes; chars p50", L[len(L)//2], "p90", L[9*len(L)//10], "max", L[-1], "total", sum(L))' "$PRIV_J/paquetes.jsonl"
```
Expected: `{"paquetes": <total>}`; max ≤ 5·4000 + ~600 chars; `total` ≈ 3–5 M chars (≈ 1–1,5 M tokens para Kimi).

- [ ] **Step 9: `plantilla.jsonl` y `README.md` del gold (público, sin texto de queries)**

`evals/retrieval-heldout/gold-j/plantilla.jsonl`:

```jsonl
{"id": "j001", "query": "cómo decidimos el umbral del recall y por qué quedó donde quedó", "source": "prompt", "expected_permalink": "kb/log/proyecto-bitacora", "acceptable_permalinks": ["kb/core/doctrina-x"], "notes": "acuerdo lenient; fable: la bitácora tiene la entrada fechada | kimi: la doctrina resume la regla | aceptable: admitido por ambos jueces | solape_lexico=0.40"}
{"id": "j002", "query": "mergea y sigue", "source": "prompt", "expected_permalink": null, "acceptable_permalinks": [], "notes": "acuerdo estricto; fable: prompt operativo | kimi: ninguna candidata responde"}
{"id": "j003", "query": "fusion rrf combsum eval", "source": "agent-search", "expected_permalink": "kb/backlog/backlog-proyecto", "acceptable_permalinks": [], "notes": "acuerdo estricto; fable: único item que compara los tres | kimi: idem | solape_lexico=0.75"}
{"id": "j004", "query": "qué pasó con el guard que dejaba pasar commits en silencio", "source": "hard", "expected_permalink": "kb/learnings/fallo-silencioso", "acceptable_permalinks": [], "notes": "acuerdo estricto; fable: paráfrasis del learning | kimi: idem | solape_lexico=0.20"}
{"id": "j005", "query": "primera vez que medimos el coste del hook por prompt, en julio", "source": "archive", "expected_permalink": "kb/archive/log/proyecto-bitacora-2026-07-01_2026-07-31", "acceptable_permalinks": [], "notes": "acuerdo estricto; fable: el hecho vive en la rotación | kimi: la viva ya no lo tiene | solape_lexico=0.50"}
{"id": "j006", "query": "configuración del router wifi de casa", "source": "negativo", "expected_permalink": null, "acceptable_permalinks": [], "notes": "acuerdo estricto; fable: tema ausente | kimi: ninguna candidata; topic_terms sin hits en grep"}
```

`evals/retrieval-heldout/gold-j/README.md`:

````markdown
# Gold J — cómo se construyó (100 % agéntico, fiabilidad medida)

Contrato: `docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md`
§3 (estratos, jueces, suelo) y §11 (kill-criteria). El gold vive fuera del
repo (`~/.local/share/exo-evals/j-heldout/gold-j.jsonl`); aquí solo el método,
la plantilla y los recuentos. Decisión de Paul del 2026-09-19 (`propuesta.md`
§7): la KB la consumen agentes, no Paul; el patrón de relevancia es la
utilidad para el agente, y el gold lo etiquetan jueces LLM de dos familias.

## Pipeline

1. **Queries** por estrato: `prompt` (prompts reales que dispararon
   `recall-inject` desde el 2026-09-13, `pool.py`), `agent-search` (comandos
   `exo search` reales de agentes, minados de los transcripts con autorización
   de Paul y limpiados con `limpia_agent_search.py`: criterio en su docstring),
   `hard` (paráfrasis por un agente que no vio ningún gold), `archive` (hechos
   que hoy viven solo en `archive/log/`), `negativo` (temas ausentes; ausencia
   comprobada con `grep` de `topic_terms` sobre la KB entera y, además, por
   los jueces). Anti-fuga: nada con Jaccard ≥ 0,8 frente a las 55 ni a las
   147 de C.
2. **Candidatos** (≤ 5 por query) por un agente filesystem-only, con al menos
   2 por navegación temática y no por grep. Es el cuello léxico declarado.
3. **Dos jueces ciegos e independientes** sobre el mismo paquete (query +
   candidatos con título y primeros 4.000 caracteres; sin etiquetas, sin
   rankings, sin el otro juez): **fable** (Claude) y **Kimi** (Moonshot,
   `juez.py`, modelo fijado en el pre-registro §10). Kimi recibe trozos de la
   KB: **envío autorizado explícitamente por Paul el 2026-09-19**.
4. **Acuerdo** (`acuerdo.py`): una fila entra solo si los dos jueces
   coinciden (estricto, o el expected de uno es aceptable para el otro). Los
   desacuerdos se descartan (sin tercer juez) y se cuentan. Suelo
   pre-registrado: κ de Cohen ≥ 0,60 y acuerdo ≥ 0,70 sobre todas las filas
   juzgadas; por debajo, J PARA (es un resultado).
5. **Auditoría del sesgo léxico**: solape query↔nota en acordadas frente a
   descartadas, y el subconjunto «léxicamente difícil» como guard de D-A.

## Recuentos (se rellenan al construir; sin texto de queries)

- snapshot `S_J`: <sha> · queries juzgadas: <n> por estrato <…>
- acuerdo: p_o <x> · κ <x> · descartes <n> (desacuerdo <n>, negativo con nota <n>, archive fuera de archive/ <n>)
- gold: <n> filas · <n> no nulas · por estrato <…> · negativos de evaluación (ids impares de `negativo`) <n>
- coste Kimi: <n> llamadas · <n> prompt tokens · <n> completion tokens · modelo <id>
````

- [ ] **Step 10: Commit (privado no se commitea)**

```bash
grep -c "wisdom-paul" evals/retrieval-heldout/gold-j/README.md evals/retrieval-heldout/gold-j/plantilla.jsonl
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 add evals/retrieval-heldout/gold-j/README.md evals/retrieval-heldout/gold-j/plantilla.jsonl
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 commit -m "eval(j): gold agentico — metodo, plantilla y recuentos (sin texto de queries)"
```
Expected: `0` y `0` antes del add.

---

### Task 7: Juicio a ciegas (Kimi + fable), acuerdo, suelo y gold

**Lane:** diseño (produce el gold y su medida de fiabilidad). **Aquí se envían datos a Moonshot** (autorización de Paul, 2026-09-19, `propuesta.md` §7).
**Oráculo:** `acuerdo.py` sale con exit 0 (suelo pasa) y `valida_gold.py` sobre `$PRIV_J/gold-j.jsonl` en exit 0; o bien `acuerdo.py` sale con exit 2 y el verdict `evals/retrieval-heldout/verdict/gold-j-acuerdo.md` dice `NO PASA: J PARA` (resultado válido; la Task 8 no se despacha).
**Depende de:** Tasks 5 y 6.

**Files:**
- Create: `evals/retrieval-heldout/verdict/gold-j-acuerdo.md` (informe público de `acuerdo.py`: solo recuentos)
- Modify: `evals/retrieval-heldout/gold-j/README.md` (§Recuentos)
- Privado: `$PRIV_J/kimi-modelos.json`, `$PRIV_J/juicio-kimi.jsonl`, `$PRIV_J/juicio-fable.jsonl`, `$PRIV_J/gold-j.jsonl`, `$PRIV_J/descartes.jsonl`

**Interfaces:**
- Consumes: `$PRIV_J/paquetes.jsonl`, `$PRIV_J/candidatos.jsonl`; `juez.py`, `acuerdo.py`, `valida_gold.py`; la key en `/home/paul/Documentos/proyectos/wisdom-ai-news/.env-keys` (variable `kimi_api_key`; el fichero es gitignored en ese repo; **nunca** se copia, imprime ni entra en el repo exo).
- Produces: `$PRIV_J/gold-j.jsonl` (schema del borrador §3, ids `j001…`), κ y p_o para §10, modelo Kimi para §10.

- [ ] **Step 1: Fijar el modelo de Kimi (runtime)**

```bash
ENV_KEYS=/home/paul/Documentos/proyectos/wisdom-ai-news/.env-keys
python3 evals/retrieval-heldout/harness/juez.py modelos --env-keys "$ENV_KEYS" | tee "$PRIV_J/kimi-modelos.json"
```
Expected: lista JSON de ids. **Regla de elección, escrita antes de ver la lista:** el modelo es `kimi-k3` si aparece; si no, el id `kimi-k2*` de mayor versión que aparezca; si no hay ninguno `kimi-k2*`, PENDIENTE-PAUL (no se elige a ojo). El id elegido va a `KIMI_MODEL` y al borrador §10 en la Task 8. Precio de lista (web, 2026-09; verificar en platform.moonshot.ai): kimi-k3 $3 / $15 por millón de tokens (entrada / salida), kimi-k2.6 $0,95 / $4,00. **Errata corregida 2026-09-20:** esta tabla tenía los precios invertidos (decía kimi-k3 $0,95/$4,00 y "K3" $3/$15); la versión correcta es la de §Global Constraints, de donde sale la estimación de ≈$6 para el job completo con k3.

- [ ] **Step 2: Humo con 3 paquetes**

```bash
KIMI_MODEL=<id del Step 1>
python3 evals/retrieval-heldout/harness/juez.py kimi --paquetes "$PRIV_J/paquetes.jsonl" --env-keys "$ENV_KEYS" --model "$KIMI_MODEL" --out "$PRIV_J/humo-kimi.jsonl" --max 3
python3 evals/retrieval-heldout/harness/juez.py valida --respuestas "$PRIV_J/humo-kimi.jsonl" --candidatos "$PRIV_J/candidatos.jsonl"
```
Expected: `{"llamadas": 3, "ok": 3, "errores": [], "prompt_tokens": <n>, ...}` y `{"respuestas": 3, "invalidas": 0}`. Si `errores` no está vacío por `response_format`: STOP y PENDIENTE-PAUL con el mensaje (sin la key), no se improvisa otro modo de salida.

- [ ] **Step 3: Kimi sobre todos los paquetes (reanudable)**

```bash
python3 evals/retrieval-heldout/harness/juez.py kimi --paquetes "$PRIV_J/paquetes.jsonl" --env-keys "$ENV_KEYS" --model "$KIMI_MODEL" --out "$PRIV_J/juicio-kimi.jsonl" 2> "$PRIV_J/juicio-kimi.log"
python3 evals/retrieval-heldout/harness/juez.py valida --respuestas "$PRIV_J/juicio-kimi.jsonl" --candidatos "$PRIV_J/candidatos.jsonl"
```
Expected: `ok` = nº de paquetes (el script reanuda por id si se corta; reintentos 3 con backoff); `invalidas: 0`. Anotar `prompt_tokens`/`completion_tokens` y el coste (`tokens × precio`) en el ledger. Presupuesto: ≈ 280 llamadas × ≈ 5–6 k tokens ≈ 1,5 M tokens de entrada + ≈ 30 k de salida ≈ **$2 con kimi-k3** (≈ $6 con K3); tope autorizado por este plan: **$10**; por encima, STOP.

- [ ] **Step 4: fable como juez (ciego; lotes de 25 paquetes; un subagente fresco por lote)**

Brief literal, con `<desde>`/`<hasta>` por lote:

"Eres un juez de relevancia. Lee `$PRIV_J/paquetes.jsonl` y trata SOLO las líneas con id entre `<desde>` y `<hasta>`. Para cada una, el campo `texto` contiene una CONSULTA y hasta cinco notas CANDIDATAS (permalink, título, inicio del cuerpo). Aplica exactamente estas instrucciones: «<contenido literal de `juez.SISTEMA`, copiado de `evals/retrieval-heldout/harness/juez.py`>». Devuelve por línea un JSON `{"id","expected","acceptable","razon"}` donde `expected` es uno de los permalinks listados en `candidatos` de esa línea o `null`, `acceptable` ⊆ `candidatos` (≤2, sin repetir `expected`) y `razon` ≤ 300 caracteres. Escribe las líneas en `$PRIV_J/juicio-fable-<desde>-<hasta>.jsonl`. PROHIBIDO: leer cualquier otro fichero (ni el snapshot, ni `evals/`, ni `~/.local/share/exo-evals/c-heldout/`, ni ningún `juicio-*.jsonl`), ejecutar `exo`/`kbx`, o buscar en la KB: juzgas solo con lo que hay en el paquete, igual que el otro juez."

Después: `cat "$PRIV_J"/juicio-fable-*.jsonl > "$PRIV_J/juicio-fable.jsonl"` y `python3 evals/retrieval-heldout/harness/juez.py valida --respuestas "$PRIV_J/juicio-fable.jsonl" --candidatos "$PRIV_J/candidatos.jsonl"` → `invalidas: 0` (una línea inválida se devuelve al mismo lote, cap 2). Control de transcript: si un lote leyó fuera de `paquetes.jsonl`, se descarta y se repite con otro subagente.

- [ ] **Step 5: Acuerdo, suelo y gold**

```bash
python3 evals/retrieval-heldout/harness/acuerdo.py --candidatos "$PRIV_J/candidatos.jsonl" --fable "$PRIV_J/juicio-fable.jsonl" --kimi "$PRIV_J/juicio-kimi.jsonl" --snap "$PRIV_J/kb-snap" --gold-out "$PRIV_J/gold-j.jsonl" --descartes-out "$PRIV_J/descartes.jsonl" --informe-out evals/retrieval-heldout/verdict/gold-j-acuerdo.md --kappa-min 0.60 --po-min 0.70; echo "exit=$?"
grep -c "wisdom-paul" evals/retrieval-heldout/verdict/gold-j-acuerdo.md
python3 evals/retrieval-heldout/harness/valida_gold.py --gold "$PRIV_J/gold-j.jsonl" --kb "$PRIV_J/kb-snap" --in-sample "$PRIV_C/in-sample-55.jsonl" --in-sample "$PRIV_C/gold.jsonl"
```
Expected: JSON `{"gold": <n>, "no_nulas": <n>, "descartes": <n>, "kappa": <x>, "po": <x>, "pasa": true}` y `exit=0`; `0`; `valida_gold` exit 0 con `"errores": 0`. **Suelos del borrador §11 con el N real:** `no_nulas ≥ 60`, `nulas_por_estrato.negativo ≥ 24`, `no_nulas_por_estrato.archive ≥ 8`; los estratos con `p_o < 0,60` en la tabla del informe se marcan «flojo» (se reportan, no deciden por sí solos). Si `exit=2`: J PARA — se commitea el informe con `NO PASA`, se anota en el ledger y en el backlog (Task 8 no aplica); es un resultado.

- [ ] **Step 6: Recuentos públicos y commit**

Rellenar `evals/retrieval-heldout/gold-j/README.md` §Recuentos con la salida de `acuerdo.py`, `valida_gold.py` y el JSON de coste del Step 3 (números, modelo, sin texto).

```bash
grep -c "wisdom-paul" evals/retrieval-heldout/verdict/gold-j-acuerdo.md evals/retrieval-heldout/gold-j/README.md
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 add evals/retrieval-heldout/verdict/gold-j-acuerdo.md evals/retrieval-heldout/gold-j/README.md
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 commit -m "eval(j): gold J juzgado a ciegas por fable y Kimi — acuerdo, kappa y recuentos"
```
Expected: `0` y `0` antes del add.

---

### Task 8: Congelación del pre-registro (la dispara el pipeline) y gate del consultor fable

**Lane:** mecánica + consultor-gate (fable, como en C). **Cierra la fase 1; bloquea toda medición (fase 2).** Sin horas de Paul: la congelación la dispara `acuerdo.py` con exit 0 (Task 7 Step 5); el gate del pre-registro congelado lo firma un consultor fable fresco.
**Oráculo:** `git -C <worktree> log -1 --format=%H -- docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md` devuelve el commit de congelación, la cabecera dice `Estado: CONGELADO`, `sha256sum "$PRIV_J/gold-j.jsonl"` coincide con §10, y `.superpowers/fabrica/verdicts/j-consultor-gate.md` termina en `GATE: PRE-REGISTRO J CONGELADO <fecha>`.
**Depende de:** Task 7 con `pasa: true` (y, si la Task 5 registró una objeción, la respuesta de Paul).

**Files:**
- Modify: `docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md` (cabecera, §10; §9 solo si una objeción de la Task 5 cambió una firma)
- Create (gitignored): `.superpowers/fabrica/verdicts/j-consultor-gate.md`
- Modify (gitignored): `.superpowers/fabrica/ledger.md`

**Interfaces:**
- Consumes: `$PRIV_J/gold-j.jsonl`, salida JSON de `valida_gold.py` y de `acuerdo.py`, `$PRIV_J/kb-snap.commit`, `$PRIV_J/kimi-modelos.json` + `KIMI_MODEL`.
- Produces: pre-registro congelado (commit), `chmod 444 $PRIV_J/gold-j.jsonl`, `preregistro_congelado: <hash>` en el ledger. La fase 2 arranca desde aquí.

- [ ] **Step 1: Validación final y sha**

```bash
S_NOW=$(git -C "$HOME/Documentos/proyectos/wisdom-paul" rev-parse HEAD); cat "$PRIV_J/kb-snap.commit"; echo "$S_NOW"
python3 evals/retrieval-heldout/harness/valida_gold.py --gold "$PRIV_J/gold-j.jsonl" --kb "$PRIV_J/kb-snap" --in-sample "$PRIV_C/in-sample-55.jsonl" --in-sample "$PRIV_C/gold.jsonl"
chmod 444 "$PRIV_J/gold-j.jsonl"; sha256sum "$PRIV_J/gold-j.jsonl"
```
Expected: exit 0 con `"errores": 0`; un sha256. Si la KB de hoy difiere de `S_J`, no importa: el gold se midió y se mide contra `S_J` (índice `base` de la fase 2 se construye sobre `kb-snap`).

- [ ] **Step 2: Congelar**

Editar el borrador:
- Cabecera: sustituir el párrafo `**Estado: BORRADOR (2026-09-19).** …` entero (hasta «antes de que exista el gold.») por `**Estado: CONGELADO el <date -Iseconds>.** Inmutable desde este commit; erratas → verdict.` Conservar las líneas `> Enmienda …` de la Task 5.
- §9: ya lleva las firmas; solo si la Task 5 registró una objeción y Paul respondió `CAMBIA A`, sustituir esa línea y añadir `(revisada el <fecha> tras objeción del revisor: "<cita de Paul>")`. Si D-J4 cambiara así, sustituir también `NETO ≥ 4` en §6 y en §7 punto 2.
- §10: `S_J` = `kb-snap.commit`; `sha256` y recuentos = JSON de `valida_gold.py`; `κ`, `p_o`, descartes = JSON de `acuerdo.py`; modelo Kimi = `KIMI_MODEL` con la lista de `kimi-modelos.json` y los tokens/coste del ledger; «aprobación del gold» = `acuerdo.py exit 0 el <fecha>` (no hay línea de Paul).

- [ ] **Step 3: Commit de congelación**

```bash
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 add docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 commit -m "eval(j): congela pre-registro del held-out J (gold agentico, acuerdo sobre el suelo, sha256 fijado)"
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 log -1 --format='%H %cI'
```
Expected: hash y fecha. Registrar en el ledger `preregistro_congelado: <hash>`. Desde aquí `git diff <hash> -- docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md` debe salir vacío en cualquier tarea posterior. El nombre del fichero conserva `BORRADOR` a propósito (las citas del plan y del backlog apuntan a él; la cabecera es la verdad).

- [ ] **Step 4: Gate del consultor fable (sustituye al gate humano)**

Despachar un subagente **fable** fresco (que no haya sido juez ni revisor) con este brief literal:

"Eres el consultor-gate del pre-registro congelado `docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md` (commit `<hash>`). Comprueba, con comandos y sin fiarte del ledger: (1) `git diff <hash> -- <fichero>` vacío; (2) `sha256sum ~/.local/share/exo-evals/j-heldout/gold-j.jsonl` = §10; (3) `valida_gold.py` exit 0 con los recuentos de §10; (4) re-ejecuta `acuerdo.py` con los mismos argumentos de la Task 7 a un fichero temporal y compara κ, p_o y recuentos con `evals/retrieval-heldout/verdict/gold-j-acuerdo.md` (deben coincidir); (5) abre 10 filas del gold al azar (semilla 20260919) más las 5 de menor `solape_lexico` en `notes` y, leyendo las notas en `kb-snap`, di si la etiqueta es defendible; (6) confirma que ningún fichero commiteado contiene texto de query ni permalinks por fila (`grep -c wisdom-paul` = 0 en `verdict/gold-j-acuerdo.md`, `gold-j/README.md`, `verdict/diagnostico-55.md`); (7) confirma que ningún fichero del repo contiene la key (`LC_ALL=C git grep --untracked -cE 'sk-[A-Za-z0-9]{20,}' -- evals` sin salida, exit 1). Escribe `.superpowers/fabrica/verdicts/j-consultor-gate.md` con cada comprobación, su comando y su salida, y termina con UNA línea: `GATE: PRE-REGISTRO J CONGELADO <fecha>` o `GATE: PRE-REGISTRO J RECHAZADO <motivo>`. Si RECHAZADO, no se mide nada: la fábrica corrige y repite este gate."

`RECHAZADO` ⇒ se corrige lo señalado (si toca el gold, el sha cambia y la congelación se rehace desde el Step 1), nuevo gate. Registrar la línea final en el ledger.

---

## Lo que queda para la fase 2 (fuera de este plan; se planifica con el pre-registro congelado delante)

Índice `base` sobre `S_J` con el binario post-G/L (rebuild ≈ 1,5 h, `condiciones.md` de C); capturas por query (`captura.py`); módulo `brazos.py` (FTS-AND réplica + oráculo (ii), F1 con `sqlite3`, S1, P1, G1 con calibración) y extensión de `metricas.py informe` para el camino secuencial de §6, las nulas por estrato y el guard léxico de D-A; corrida, `j-agregados.md`, `j-verdict.md` adjudicado por fable fresco con cita textual; **solo entonces** `engine/src` para el ganador (con oráculo de reproducción exacta, §11) o cierre «A0 se queda»; backlog, `arquitectura.md` §6 y D6 confirmado. La única tarea de engine independiente del verdict es D-J5 = a (renombrar `similitud_desde_l2_cuadrado` y corregir su doc, sin cambio de comportamiento), que puede ir suelta cuando Paul quiera.

## Self-review (checklist contra el brief)

- T0 con comandos exactos y salida verificable: Tasks 1–2 (código y tests corridos en dry-run el 2026-09-19: `miss_hist 6 · miss_S 12 · hit→miss 7`). ✔
- Pre-registro borrador con brazos, métrica primaria (hit@5 + abstención sobre negativos verdaderos), regla de decisión con potencia calculada por N (60–180) y por número de decisiones, challenge a los 8 brazos (4 decisiones en camino secuencial), kill-criteria incluido «J PARA por acuerdo insuficiente» y «si nada gana». ✔
- Gold 100 % agéntico con fiabilidad medida (revocación de D-J1/D-J2, 2026-09-19): estratos `prompt`, `agent-search` (limpieza con criterio escrito), `hard`, `archive`, `negativo` (ausencia mecánica + jueces); dos jueces ciegos de dos familias (fable, Kimi); entrada solo por acuerdo, desacuerdos descartados; suelo κ ≥ 0,60 ∧ p_o ≥ 0,70 con referencias marcadas por confianza; sesgo léxico declarado con auditoría y guard; congelación disparada por el pipeline y gate por consultor fable; 0 h de Paul. ✔
- Exposición de datos a Moonshot declarada en Global Constraints, Task 7 y borrador §8; key en runtime, nunca en el repo; `grep` de key antes de commitear y en el gate. ✔
- #4 = c: P1 como brazo medido, D-C. #12 = b: no afecta. #13: sustituida por el gold agéntico (revocación). ✔
- `engine/src` intocado en todas las tareas; solo `cargo build` para leer. ✔
- Código verificado: `diagnostico.py` (5 tests), extensiones de `valida_gold.py`/`pool.py` (4 tests), `limpia_agent_search.py`/`juez.py`/`acuerdo.py` (7 tests, HTTP inyectado) — todo verde el 2026-09-19 en scratchpad. ✔
- Placeholders: los únicos `<…>` son campos que se rellenan con salidas de comando en el momento de la corrida (Tasks 2, 6, 7, 8), como en el plan de C. ✔

## Handoff

Ejecutar con `exo:orchestrate`, tarea a tarea, en la rama `campana-j-fase1`.
Tasks 1 y 3 en paralelo; 2 tras 1; 4 tras 3; 5 tras 2 (y antes de 7); 6 tras
4; 7 tras 5 y 6; 8 tras 7 con `pasa: true`. Nada espera a Paul salvo una
objeción del revisor (Task 5 Step 3). La fase 2 se planifica con el
pre-registro congelado delante, nunca antes.
