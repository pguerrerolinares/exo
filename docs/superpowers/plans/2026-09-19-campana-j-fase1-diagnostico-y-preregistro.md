# Campaña J — fase 1: T0 diagnóstico, pre-registro borrador y kit de gold Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: usa `exo:orchestrate`
> para ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`)
> para tracking.

**Goal:** dejar a la campaña J lista para medir sin haber medido nada: un
diagnóstico descriptivo por fila de las 55 in-sample (T0), un pre-registro en
borrador con brazos, regla de decisión y potencia calculada (T1), y un kit de
etiquetado con el que Paul produce el gold nuevo esta semana en paralelo
(decisión #13), con el harness ya aceptando ese gold. La fase termina con el
commit de congelación del pre-registro, que solo existe cuando el gold existe.

**Architecture:** todo vive bajo `evals/retrieval-heldout/` y en el directorio
privado `$PRIV_J`; `engine/src` no se toca. El T0 cruza tres estados del mismo
conjunto de 55 queries (histórico 49/55, snapshot de C 43/55, producción hoy)
usando las capturas privadas de C y una captura nueva de solo lectura, y
explica cada miss con rangos, df de tokens y presencia de `archive/`. El
harness de C se extiende en dos puntos (estratos nuevos y listas de exclusión
repetibles) para que el gold de J valide con `valida_gold.py` sin cambiar el
schema. El criterio de decisión vive en
`docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md` y se
congela por commit en la última tarea.

**Tech Stack:**
- Python 3.12, solo stdlib (`unittest`, `json`, `sqlite3` con FTS5, `re`,
  `math`), igual que `evals/retrieval-heldout/harness/`.
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
- **Pool nuevo:** 113 `prompt` desde el 2026-09-13 (tras excluir las 55 y las
  147 de C), **0 `agent-search`** (el `reflex-retrieval-log.jsonl` termina el
  2026-08-17). El estilo palabras clave lo escribe Paul (estrato `keyword`).
- **`archive/` en C:** 24/92 esperadas y 7 aceptables vivían en `archive/`;
  274/723 plazas del top-5. Excluir rompería una de cada cuatro filas;
  penalizar es un trade-off medible, no obvio.
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
  (Task 7).** Ninguna tarea computa hit@k de ningún brazo sobre ninguna query
  del gold nuevo. El T0 usa solo las 55 in-sample (ya vistas por todos).
- **Régimen de fábrica vigente:** `.superpowers/fabrica/config.md`
  §ACTUALIZACIÓN 2026-09-19: «J fase 1 — T0 diagnóstico + pre-registro
  borrador + kit de gold … evals; **no toca `engine/src`**» y «J no congela
  su pre-registro hasta que exista el gold de Paul».
- **Repo público (desde 2026-09-02).** Ningún texto de query, prompt ni
  permalink por fila entra en git. Datos privados en
  `PRIV_J=~/.local/share/exo-evals/j-heldout` (`chmod 700`); se lee
  `PRIV_C=~/.local/share/exo-evals/c-heldout` (in-sample y capturas de C).
  Comprobación obligatoria antes de cada commit de un `.md` bajo `verdict/`:
  `grep -c "wisdom-paul" <fichero>` = 0.
- **Líneas rojas:** no se escribe en `~/.exo/index.db` (solo `exo search`);
  no se escribe en `~/Documentos/proyectos/wisdom-paul` (el snapshot es un
  `git clone` a `$PRIV_J`); nada de push, merge ni commit a `main`; `git add`
  con rutas explícitas, nunca `-A`; `git -C <path>`, nunca `cd <path> &&
  git`. Commits terminan con las líneas de atribución de la sesión.
- **Harness: Python stdlib**, sin modificar `evals/retrieval-fase0/` ni
  `metricas.py`, `captura.py`, ni los ficheros de `verdict/` de C.
- **Tests:** `python3 -m unittest evals/retrieval-heldout/harness/test_harness.py`
  desde la raíz del repo. Hoy: `Ran 31 tests … OK`. Al final del plan: 40.
- **Rama de campaña:** `campana-j-fase1`, creada desde `main` una vez
  mergeada `plan-campanas-i-l-j` (este plan y el borrador). Worktree
  `.worktrees/campana-j-fase1`. Todos los comandos `python3` se corren con
  cwd en la raíz del worktree.
- **Etiquetado sin motor** (borrador §3): quien etiquete, Paul incluido, no
  usa `exo search`, `kbx` ni `~/.exo/`. Los subagentes de `hard` y de
  verificación no leen las 55, el gold de C, `evals/` ni `reports/`.

## Decisiones firmadas por Paul 2026-09-19

Fuente de las firmas: `docs/superpowers/consultas/2026-09-15-campanas/propuesta.md`
§7 (sesión del 2026-09-19). Cada una coincide con la recomendación del
planificador; las opciones descartadas se conservan aquí como traza del
trade-off, no como algo abierto. La Task 7 copia las firmas literales al
borrador §9 (ya las lleva) y solo espera el gold y su sha256.

| decisión | firmado | descartado (traza) |
|---|---|---|
| D-J1 modo de etiquetado | **(b)** agente pre-etiqueta `prompt`+`hard`, verificador adversarial, Paul escribe `keyword`/`archive`/`negativo` y revisa todo — ≈ 3,5–4 h | (a) Paul etiqueta todo (≈ 5 h, sin anclaje); (c) como b con `prompt` a 60 (≈ 3 h) |
| D-J2 estrato `prompt` | **113** (pool entero) | 60 (muestra) |
| D-J3 verificación adversarial del gold | **sí** | no |
| D-J4 regla GANA | **NETO ≥ 4** (∧ ARREGLA ≥ 2·ROMPE ∧ veto ΔMRR) | 3 (cota familiar 0,41) · 5 (potencia +5 pp 0,42–0,55) |
| D-J5 H28 | **(a)** corrección de nombre y doc, escala `s` se queda, sin brazo | (b) celda descriptiva `sellado-cos` |
| D-J6 int8 | **fuera** | dentro como celda descriptiva si K = no daemon |
| D-J7 factor P1 | **0,90** | otro valor único |
| D-J8 tope df F1 | **0,25**, con la cláusula de T0 intacta (T0 puede sustituirlo una vez, con evidencia in-sample anotada en el borrador) | otro valor único |
| D-J9 G1 | **tal cual**: p70 de calibración; adopta si abstiene ≥ 60 % de evaluación y ROMPE ≤ 2 | otros valores únicos |
| D-J10 relevancia | **lenient decide**, strict descriptivo | strict decide |

Detalle del trade-off de cada una, tal como se presentó:

**D-J1 — Modo de etiquetado.** Tiempo estimado de Paul por modo, con los
ritmos de C (revisión ≈ 0,7 min/fila; etiquetar de cero una positiva ≈ 3 min,
una nula ≈ 0,5 min; escribir una query ≈ 1–2 min):

| modo | qué hace Paul | qué hace la fábrica | horas de Paul | no nulas esperadas |
|---|---|---|---|---|
| (a) Paul etiqueta todo | lee el kit (15 min); etiqueta 113 prompts (≈80 nulas × 0,5 + ≈33 positivas × 3 ≈ 140 min); escribe y etiqueta 15 `keyword` (30 min), 12–15 `archive` (40 min), 30 `negativo` (15 min); verifica 30 `hard` (45 min); revisa CORREGIR del verificador (20 min) | pool, snapshot, `hard`, verificación adversarial, congelación | **≈ 5 h** | 85–95 |
| (b) agente pre-etiqueta `prompt` y `hard`, verificador adversarial, Paul escribe `keyword`/`archive`/`negativo` y revisa todo | kit (15); escribe 57 queries (85 min); revisa 143 filas pre-etiquetadas con las banderas del verificador (≈ 100 min); CORREGIR (20) | todo lo demás | **≈ 3,5–4 h** | 80–90 |
| (c) como (b) con `prompt` muestreado a 60 (semilla `20260919`) | kit (15); 57 queries (85); revisa 90 filas (63); CORREGIR (20) | ídem | **≈ 3 h** | 70–80 |

- Pro (a): sin anclaje a etiquetas de agente (en C el verificador corrigió
  41/147 filas de agentes). Contra: 5 h no son «2–3 h», y la fatiga baja la
  calidad a partir de la segunda hora.
- Pro (b): el tiempo de Paul va donde solo él puede (`archive`, `negativo`,
  `keyword`) y a juzgar, no a buscar. Contra: anclaje; se mitiga con el
  verificador adversarial antes de que Paul mire.
- Firmado (b), en dos o tres sesiones de ≤ 1,5 h. La cifra «2–3 h» de la
  propuesta era optimista: solo se cumplía con (c) y sin verificación
  adversarial.

**D-J2 — Tamaño del estrato `prompt`:** 113 (todo el pool) frente a 60
(muestra). Firmado 113: más prompts = más nulas por inferibilidad (corpus
negativo débil) y ≈ 30 % de no nulas; con (b) el coste extra lo paga la
fábrica, no Paul.

**D-J3 — Verificación adversarial del gold antes de congelar.** Coste: 1
fable ≈ 1 h de fábrica, +20 min de Paul. Firmado sí (en C cazó 19 nulas
disfrazadas de miss y 16 aceptables omitidos; con lenient decidiendo, un
aceptable omitido fabrica o borra un GANA).

**D-J4 — Regla GANA:** NETO ≥ 3 / 4 / 5, siempre con `ARREGLA ≥ 2·ROMPE` y
veto ΔMRR. Tabla completa en el borrador §7. Firmado 4: tres decisiones GANA
en cadena, cota familiar de adoptar algo inútil 0,32 frente a 0,41 con 3;
potencia ante +8 pp 0,83–0,91.

**D-J5 — H28:** (a) corrección de nombre y documentación de
`similitud_desde_l2_cuadrado`, la escala `s` se queda, sin brazo; (b) además
una celda descriptiva `sellado-cos` (fusión en escala coseno, umbral 0,28).
Firmado (a): (b) es una recalibración de β disfrazada y no decide.

**D-J6 — int8:** fuera (borrador §2.8) / dentro solo si K = no daemon.
Firmado fuera: es una pregunta de no inferioridad que la regla GANA no puede
responder y cuyo beneficio canibaliza K.

**D-J7 — Factor de P1 (`archive/`):** firmado 0,90: rompe empates a favor del
vivo (0,60 → 0,54 frente a un vivo de 0,55) sin expulsar a `archive/` del
top-10.

**D-J8 — Tope df de F1:** firmado 0,25 con la cláusula de T0 intacta: el T0
aporta la cifra in-sample (dry-run: 52/55 con ≥1 raro a 0,25) y puede
sustituir el valor **una vez**, con la evidencia anotada como enmienda en el
borrador (Task 5 Step 2). Si los tokens que matan el AND tienen df 0, el tope
no importa y se deja igual.

**D-J9 — G1:** firmado tal cual: percentil 70 de calibración; adopta si
abstiene ≥ 60 % de los negativos de evaluación y ROMPE ≤ 2.

**D-J10 — Modo de relevancia:** firmado lenient decide (D5 de C), strict
descriptivo.

## Dependencias y conflictos

| Tarea | Ficheros | Conflicto | Regla |
|---|---|---|---|
| 1, 3 (harness) | `evals/retrieval-heldout/harness/{diagnostico,valida_gold,pool,test_harness}.py` | ninguna campaña activa toca `evals/retrieval-heldout/` (matriz de colisión de la propuesta §3: solo J) | libre |
| 2 (captura «hoy») | ninguno de código; lee `~/.exo/index.db` | I y L tocan `plugins/` y `engine/src/main.rs`; la captura usa el binario de `main` en el momento de la Task 2 y lo anota | anotar `git rev-parse HEAD` del binario en el fichero público |
| 4 (kit) | `evals/retrieval-heldout/kit-gold-j/`, `$PRIV_J` | ninguno | libre |
| 5, 7 (borrador y congelación) | `docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md` | ninguno | inmutable tras la Task 7 salvo erratas → verdict |
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
  `keyword|archive|negativo`, `--in-sample` repetible, `nulas_por_estrato`.
- Modify: `evals/retrieval-heldout/harness/pool.py` — `filtra(…, ventanas)`,
  `--desde/--hasta`, `--in-sample` repetible.
- Create: `evals/retrieval-heldout/verdict/diagnostico-55.md` — T0 público.
- Create: `evals/retrieval-heldout/kit-gold-j/README.md`,
  `evals/retrieval-heldout/kit-gold-j/plantilla.jsonl` — kit de Paul.
- Create: `evals/retrieval-heldout/verdict/gold-j-verificacion-resumen.md`
  (Task 6, solo recuentos).
- Modify: `docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md`
  — §4 (valores únicos, si T0 lo justifica), §9, §10 y cabecera (Task 7).
- Gitignored: `.superpowers/fabrica/packages/j-gold.md`,
  `.superpowers/fabrica/pendiente-paul.md`, `.superpowers/fabrica/ledger.md`.
- Privado: `$PRIV_J/{kb-snap/,kb-snap.commit,cap-hoy-55.jsonl,diagnostico-55-detalle.md,muestra-prompt.jsonl,pool.jsonl,notas-hard.txt,hard-candidatas.jsonl,gold-j.jsonl,gold-verificacion.md}`.

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

### Task 4: Kit de etiquetado para Paul (snapshot, pool `prompt`, candidatas `hard`, pre-etiquetado verificado, README, plantilla, package)

**Lane:** diseño (construye el oráculo de la campaña: el material del gold). Con D-J1 = (b) firmado, la fábrica **pre-etiqueta `prompt` y `hard`** con un agente fresco y pasa esas pre-etiquetas por el verificador adversarial antes de que Paul las vea; `keyword`, `archive` y `negativo` los escribe Paul.
**Oráculo:** (1) `python3 evals/retrieval-heldout/harness/valida_gold.py --gold evals/retrieval-heldout/kit-gold-j/plantilla.jsonl --kb "$PRIV_J/kb-snap" --in-sample "$PRIV_C/in-sample-55.jsonl" --in-sample "$PRIV_C/gold.jsonl"` sale con exit 1 y **exactamente** 5 líneas de error, todas `expected_permalink inexistente en el snapshot` o `acceptable inexistente en el snapshot` (la plantilla usa permalinks `kb/…` ficticios a propósito; el schema y las reglas de estrato pasan; verificado el 2026-09-19 contra las listas de exclusión reales); (2) `valida_gold.py` sobre `$PRIV_J/gold-j-preetiquetado.jsonl` en exit 0; (3) `.superpowers/fabrica/packages/j-gold.md` existe.
**Depende de:** Task 3. No espera a Paul: el modo está firmado.

**Files:**
- Create: `evals/retrieval-heldout/kit-gold-j/README.md`
- Create: `evals/retrieval-heldout/kit-gold-j/plantilla.jsonl`
- Create (gitignored): `.superpowers/fabrica/packages/j-gold.md`
- Privado: `$PRIV_J/kb-snap/`, `$PRIV_J/kb-snap.commit`, `$PRIV_J/pool.jsonl`, `$PRIV_J/muestra.jsonl` → renombrado `muestra-prompt.jsonl`, `$PRIV_J/notas-hard.txt`, `$PRIV_J/hard-candidatas.jsonl`, `$PRIV_J/gold-j-preetiquetado.jsonl`, `$PRIV_J/gold-verificacion-preetiquetas.md`

**Interfaces:**
- Consumes: `pool.py` y `valida_gold.py` de la Task 3.
- Produces: `$PRIV_J/gold-j-preetiquetado.jsonl` (filas `prompt` + `hard`, ids `j001…`, schema del borrador §3, verificadas) y `$PRIV_J/gold-verificacion-preetiquetas.md` (banderas por fila). `$PRIV_J/gold-j.jsonl` lo termina **Paul**: revisa esas filas con las banderas y añade las suyas (`keyword`, `archive`, `negativo`) con ids consecutivos; la Task 6 lo lee. `hard-candidatas.jsonl` con schema `{"query", "source": "hard", "author_expected"}` (el de C).

- [ ] **Step 1: Snapshot inmutable de la KB (lectura; sin escribir en la KB)**

```bash
export PRIV_C=~/.local/share/exo-evals/c-heldout PRIV_J=~/.local/share/exo-evals/j-heldout
S_J=$(git -C "$HOME/Documentos/proyectos/wisdom-paul" rev-parse HEAD)
git clone --quiet --no-hardlinks "$HOME/Documentos/proyectos/wisdom-paul" "$PRIV_J/kb-snap"
git -C "$PRIV_J/kb-snap" checkout --quiet --detach "$S_J"
echo "$S_J" > "$PRIV_J/kb-snap.commit"; cat "$PRIV_J/kb-snap.commit"
ls "$PRIV_J/kb-snap/archive/log" | wc -l
```
Expected: un sha de 40 hex (es `S_J` del borrador §10); ≈ 32 ficheros en `archive/log`. Si Paul rota bitácoras durante la semana, el snapshot se rehace en la Task 7 (y `valida_gold.py` dirá qué filas dejaron de resolver).

- [ ] **Step 2: Pool `prompt` desde el 2026-09-13**

```bash
python3 evals/retrieval-heldout/harness/pool.py --in-sample "$PRIV_C/in-sample-55.jsonl" --in-sample "$PRIV_C/gold.jsonl" --out-dir "$PRIV_J" --cuota prompt=0 --desde 2026-09-13T00:00:00Z --hasta "$(date -u +%Y-%m-%dT00:00:00Z)"
mv "$PRIV_J/muestra.jsonl" "$PRIV_J/muestra-prompt.jsonl"; wc -l < "$PRIV_J/muestra-prompt.jsonl"
```
Expected: JSON con `"pool": {"prompt": ≥113, "agent-search": 0}` (el 2026-09-19 eran 113; crece con la semana) y `muestra` = ese número. D-J2 = 113 firmado: se etiqueta el pool entero (lo que haya el día de la corrida), en el orden barajado con la semilla `20260913:prompt` de `muestrea`.

- [ ] **Step 3: Notas para el estrato `hard` (30 al azar, sin `archive/`, semilla 20260919)**

```bash
python3 - <<'EOF'
import os, random
from pathlib import Path
snap = Path(os.path.expanduser("~/.local/share/exo-evals/j-heldout/kb-snap"))
notas = sorted(str(p.relative_to(snap)) for p in snap.rglob("*.md")
               if not any(x.startswith(".") for x in p.relative_to(snap).parts) and not str(p.relative_to(snap)).startswith("archive/"))
random.Random(20260919).shuffle(notas)
Path(os.path.expanduser("~/.local/share/exo-evals/j-heldout/notas-hard.txt")).write_text("\n".join(notas[:30]) + "\n", encoding="utf-8")
print(len(notas), "elegibles; 30 elegidas")
EOF
```
Expected: `≈107 elegibles; 30 elegidas` (182 − 75 de `archive/`).

- [ ] **Step 4: Candidatas `hard` (autor fresco, distinto de cualquier etiquetador)**

Despachar un subagente **sonnet** fresco con este brief literal:

"Lee las 30 notas de `$PRIV_J/kb-snap/` listadas en `$PRIV_J/notas-hard.txt`. Por cada nota escribe UNA query en castellano que Paul haría para re-encontrarla SIN sus palabras clave literales ni su título: las 15 primeras de la lista en frase natural (16–30 palabras), las 15 siguientes en 2–5 palabras clave parafraseadas. PROHIBIDO: ejecutar `exo`, `kbx` o cualquier tool de basic-memory; leer `~/Documentos/proyectos/exo/evals/`, `reports/`, `~/.exo/`, `~/.local/share/exo-evals/` salvo `j-heldout/kb-snap` y `j-heldout/notas-hard.txt`. Devuelve JSONL `{"query", "source": "hard", "author_expected": "<permalink del frontmatter de la nota>"}`, 30 líneas, en `$PRIV_J/hard-candidatas.jsonl`."

Control de transcript: si aparece `exo `, `kbx ` o `evals/` en los comandos del subagente, se descarta su salida entera y se repite con otro (retries cap 2). Registrar en el ledger.

- [ ] **Step 5: Pre-etiquetado de `prompt` y `hard` (D-J1 = b; etiquetador fresco, distinto del autor del Step 4, filesystem-only)**

Despachar un subagente **sonnet** fresco con este brief literal:

"Etiqueta queries para un gold de retrieval sobre la KB `$PRIV_J/kb-snap/`. Entrada: `$PRIV_J/muestra-prompt.jsonl` (estrato `prompt`, EN ESE ORDEN de fichero, todas las filas) y después `$PRIV_J/hard-candidatas.jsonl` (estrato `hard`, 30 filas). Solo Read/Grep/Glob sobre `kb-snap` (títulos, frontmatter `permalink:`, `core/core-index.md` como mapa). PROHIBIDO: `exo`, `kbx`, basic-memory, leer `evals/`, `reports/`, `~/.exo/`, `~/.local/share/exo-evals/c-heldout/`. Por cada query: `expected_permalink` = la nota que un usuario razonable querría recuperar **con la query sola** (sin el contexto de la sesión: si la nota solo se identifica sabiendo de qué iba el turno, es `null`; un prompt operativo tipo 'mergea' o 'sigue' es `null`). `acceptable_permalinks`: máx 2 y SOLO si otra nota serviría razonablemente igual; cada uno con una frase `aceptable: <por qué>` en `notes`. Criterio canon vs bitácora (literal de `labels.md:70`): 'query genérica/estado → canon; query histórica/detalle fechado → bitácora'. Para `hard`, verifica `author_expected`, no lo copies. `notes` siempre con la razón. Salida: `$PRIV_J/gold-j-preetiquetado.jsonl`, schema `{"id","query","source","expected_permalink","acceptable_permalinks","notes"}`, ids `j001..` consecutivos en el orden etiquetado (primero `prompt`, luego `hard`)."

Mismo control de transcript que el Step 4. Validar:
```bash
python3 evals/retrieval-heldout/harness/valida_gold.py --gold "$PRIV_J/gold-j-preetiquetado.jsonl" --kb "$PRIV_J/kb-snap" --in-sample "$PRIV_C/in-sample-55.jsonl" --in-sample "$PRIV_C/gold.jsonl"
```
Expected: exit 0; `"filas"` = nº de prompts + 30; `no_nulas_por_estrato` con `prompt` ≈ 30 % de las filas `prompt` y `hard` ≥ 25. Errores ⇒ el etiquetador corrige esas filas (no el orquestador), máx 2 vueltas.

- [ ] **Step 6: Verificación adversarial de las pre-etiquetas (antes de que Paul las vea)**

Subagente **fable** fresco (u **opus**), que no haya participado en los Steps 4–5. Brief literal:

"Eres el verificador adversarial de un gold de retrieval pre-etiquetado por un agente (formato y criterio de `~/Documentos/proyectos/exo/evals/retrieval-fase0/verdict/labels.md`: léelo primero, es el precedente aprobado). Gold: `$PRIV_J/gold-j-preetiquetado.jsonl`. KB: `$PRIV_J/kb-snap/`, SOLO filesystem (Read/Grep/Glob). PROHIBIDO `exo`, `kbx`, basic-memory, `evals/` salvo `labels.md`, `reports/`, y `~/.local/share/exo-evals/c-heldout/`. Para CADA fila: (1) ¿existe el `expected_permalink` exacto en frontmatter?; (2) ¿es la nota que un usuario razonable querría **con la query sola**?, busca activamente una mejor; (3) cada `acceptable_permalinks`, ¿justificado o comodín que infla hits?; (4) cada `null`: grep de los términos clave y confirma que no hay nota. Veredicto por fila: CORRECTO / DEFENDIBLE / CORREGIR con la corrección exacta y la cita de la nota (línea literal). Al final: recuentos por veredicto y estrato, y las 3 filas más flojas. Escribe en `$PRIV_J/gold-verificacion-preetiquetas.md`. NO modifiques el gold: las correcciones las decide Paul al revisar."

Este fichero es lo que Paul lee fila a fila junto al pre-etiquetado: las filas `CORREGIR` van primero en su revisión.

- [ ] **Step 7: `plantilla.jsonl` (6 filas de ejemplo con permalinks ficticios `kb/…`)**

`evals/retrieval-heldout/kit-gold-j/plantilla.jsonl`:

```jsonl
{"id": "j001", "query": "cómo decidimos el umbral del recall y por qué quedó donde quedó", "source": "prompt", "expected_permalink": "kb/log/proyecto-bitacora", "acceptable_permalinks": ["kb/core/doctrina-x"], "notes": "la bitácora tiene la entrada fechada con la decisión. aceptable: la doctrina resume la misma regla sin la fecha"}
{"id": "j002", "query": "mergea y sigue", "source": "prompt", "expected_permalink": null, "acceptable_permalinks": [], "notes": "prompt operativo; ninguna nota es la respuesta (nula por inferibilidad)"}
{"id": "j003", "query": "fusion rrf combsum eval", "source": "keyword", "expected_permalink": "kb/backlog/backlog-proyecto", "acceptable_permalinks": [], "notes": "palabras clave estilo agente; el item del backlog es la única nota que compara los tres"}
{"id": "j004", "query": "qué pasó con el guard que dejaba pasar commits en silencio", "source": "hard", "expected_permalink": "kb/learnings/fallo-silencioso", "acceptable_permalinks": [], "notes": "paráfrasis sin términos literales; verificado author_expected"}
{"id": "j005", "query": "primera vez que medimos el coste del hook por prompt, en julio", "source": "archive", "expected_permalink": "kb/archive/log/proyecto-bitacora-2026-07-01_2026-07-31", "acceptable_permalinks": [], "notes": "el hecho vive en la rotación de julio; la bitácora viva ya no lo tiene"}
{"id": "j006", "query": "configuración del router wifi de casa", "source": "negativo", "expected_permalink": null, "acceptable_permalinks": [], "notes": "tema ausente de la KB: grep de 'wifi|router' da 0 notas"}
```

- [ ] **Step 8: `README.md` del kit (instrucciones para Paul)**

`evals/retrieval-heldout/kit-gold-j/README.md`:

````markdown
# Kit de gold — campaña J (retrieval con held-out nuevo)

Para: Paul. Objetivo: un fichero `~/.local/share/exo-evals/j-heldout/gold-j.jsonl`
que decide de una vez N1 (FTS selectivo), el operador de fusión, la
penalización de `archive/` y la abstención. Contrato: el pre-registro
borrador `docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md`
(§3 estratos, §11 suelos). Cuando el fichero exista y valide, se congela con
su sha256 y **entonces** empieza la medición. Hasta entonces nadie toca
`engine/src`.

## Reglas que no se negocian

1. **Sin motor.** Etiquetas con Obsidian, `grep`, `ls` o leyendo. Nada de
   `exo search`, `kbx`, ni mirar `~/.exo/`. Si etiquetas con el motor, el gold
   premia al motor actual y la campaña no mide nada.
2. **Una línea JSON por fila**, schema exacto:
   `{"id","query","source","expected_permalink","acceptable_permalinks","notes"}`.
   Ids `j001…` consecutivos. `source` ∈ `prompt | keyword | hard | archive | negativo`.
3. **`expected_permalink`** es el `permalink:` del frontmatter de la nota que
   un usuario razonable querría recuperar con esa query **sola** (sin el
   contexto de la sesión). Si la query sola no identifica ninguna nota, `null`
   (en C, 19 de 41 prompts etiquetados «por contexto» eran nulas disfrazadas).
4. **`acceptable_permalinks`**: máximo 2, solo si otra nota serviría
   razonablemente igual; cada uno con una frase `aceptable: <por qué>` en
   `notes`. Criterio canon/bitácora: «query genérica/estado → canon; query
   histórica/detalle fechado → bitácora».
5. **`notes`** siempre: por qué esa nota (o por qué ninguna). Es lo que el
   verificador y el adjudicador leen.
6. **No copies queries de las 55 ni del gold de C** (el validador las rechaza
   por Jaccard ≥ 0,8). No rotes bitácoras hasta congelar (o avisa: se rehace
   el snapshot).

## Qué hay ya en `~/.local/share/exo-evals/j-heldout/` (modo D-J1 = b, firmado el 2026-09-19)

- `kb-snap/` — snapshot de tu KB (`kb-snap.commit` dice el commit). Etiqueta
  contra él o contra tu KB viva: al congelar se valida contra el snapshot.
- `gold-j-preetiquetado.jsonl` — tus prompts reales desde el 2026-09-13 que
  dispararon `recall-inject` (todos, D-J2 = 113) y 30 paráfrasis `hard`, ya
  **pre-etiquetados por un agente fresco** que no vio las 55 ni el gold de C,
  y ya pasados por un verificador adversarial. Tu trabajo aquí es **revisar**,
  no etiquetar de cero: acepta, corrige o anula cada fila. Muchas `prompt`
  serán `null`: es normal y sirve (corpus negativo débil).
- `gold-verificacion-preetiquetas.md` — el veredicto del verificador por
  fila (CORRECTO / DEFENDIBLE / CORREGIR con cita). Empieza por las
  `CORREGIR`; en C el verificador tenía razón en 41/147 filas de agente.
- `hard-candidatas.jsonl` — las 30 paráfrasis originales con
  `author_expected`, por si quieres ver qué nota tenía en mente el autor.
- `plantilla.jsonl` (en el repo, `evals/retrieval-heldout/kit-gold-j/`) —
  6 filas de ejemplo con permalinks ficticios, una por estrato.

Cómo se arma `gold-j.jsonl`: copia `gold-j-preetiquetado.jsonl`, revisa sus
filas, y añade las tuyas al final con ids consecutivos (`keyword`, `archive`,
`negativo`). Un solo fichero, una línea por fila.

## Lo que solo tú puedes escribir (y por eso vale la pena tu tiempo)

- **`keyword` (15)**: 2–5 palabras clave, como las buscarías tú o un agente
  con `exo search`. Es el estilo de las 55 y de `agent-search` de C, que ya
  no tiene pool. Ejemplo de forma (no de contenido): `fusion rrf combsum eval`.
- **`archive` (12–15)**: preguntas cuya respuesta vive **hoy** en
  `archive/log/` — hechos de episodios de julio/agosto que una rotación se
  llevó. Para elegir: `ls ~/.local/share/exo-evals/j-heldout/kb-snap/archive/log`
  (los nombres llevan el rango de fechas). `expected_permalink` tiene que
  estar bajo `archive/` (el validador lo exige). Esto decide la penalización
  de `archive/` (decisión #4 = c): sin este estrato la decisión es a ciegas.
- **`negativo` (30)**: temas de los que tu KB **no tiene nada** (no
  «difíciles»: ausentes). `expected_permalink: null`, y en `notes` el grep que
  lo demuestra. Ejemplos de forma: una receta, un tema de otro proyecto que
  nunca documentaste, una tecnología que no usas. Los de id **par** calibran
  el umbral de abstención; los de id **impar** lo evalúan: escribe los 30
  seguidos, sin ordenarlos por dificultad.

## Tamaños y tiempo (honestos)

| estrato | filas | no nulas esperadas | tu tiempo |
|---|---|---|---|
| `prompt` | 113 (D-J2) | ≈ 30 % | 0,7 min/fila revisando las pre-etiquetas con las banderas del verificador |
| `keyword` | 15 | 15 | 2 min/fila |
| `hard` | 30 | 25–30 | 0,7 min/fila (ya pre-etiquetadas y verificadas) |
| `archive` | 12–15 | 12–15 | 3 min/fila (localizar la rotación) |
| `negativo` | 30 | 0 | 0,5 min/fila |

Modo firmado (D-J1 = b, `propuesta.md` §7, 2026-09-19): la fábrica
pre-etiqueta `prompt` y `hard` y los pasa por un verificador adversarial; tú
escribes `keyword`/`archive`/`negativo` (≈ 85 min) y revisas todo con las
banderas del verificador (≈ 100 min). **Total ≈ 3,5–4 h en dos o tres
sesiones de ≤ 1,5 h.** Suelo para que la campaña arranque: **60 no nulas,
12 negativos de evaluación, 8 `archive`**; por debajo, STOP.

## Validar antes de entregar

```bash
PRIV_C=~/.local/share/exo-evals/c-heldout; PRIV_J=~/.local/share/exo-evals/j-heldout
python3 evals/retrieval-heldout/harness/valida_gold.py --gold "$PRIV_J/gold-j.jsonl" --kb "$PRIV_J/kb-snap" --in-sample "$PRIV_C/in-sample-55.jsonl" --in-sample "$PRIV_C/gold.jsonl"
```
Exit 0 y `"errores": 0`. Cada error dice el `id` y qué falla. Entrega =
línea `GATE: GOLD-J ENTREGADO <fecha>` en `.superpowers/fabrica/packages/j-gold.md`.
````

- [ ] **Step 9: Validar la plantilla (oráculo) y el package**

```bash
python3 evals/retrieval-heldout/harness/valida_gold.py --gold evals/retrieval-heldout/kit-gold-j/plantilla.jsonl --kb "$PRIV_J/kb-snap" --in-sample "$PRIV_C/in-sample-55.jsonl" --in-sample "$PRIV_C/gold.jsonl" 2>&1 | sort | uniq -c
```
Expected: exit 1; 5 líneas de error, todas `… inexistente en el snapshot` (`expected_permalink` de j001, j003, j004, j005 + `acceptable` de j001); la línea JSON final con `"errores": 5`, `"no_nulas_por_estrato": {"prompt": 1, "keyword": 1, "hard": 1, "archive": 1}`, `"nulas_por_estrato": {"prompt": 1, "negativo": 1}`. Ningún error de `source inválido`, de `negativo con expected`, de `archive exige` ni de `duplica una de las 55`.

Escribir `.superpowers/fabrica/packages/j-gold.md` con: (1) ruta y comando para abrir cada fichero de `$PRIV_J` (`python3 -c 'import json,sys; [print(json.dumps(json.loads(l),ensure_ascii=False,indent=1)) for l in open(sys.argv[1])]' "$PRIV_J/gold-j-preetiquetado.jsonl" | less`); (2) el README anterior enlazado; (3) la tabla de tiempos; (4) los recuentos de `valida_gold.py` sobre el pre-etiquetado y los recuentos CORRECTO/DEFENDIBLE/CORREGIR del verificador con las 3 filas más flojas; (5) la tabla «Decisiones firmadas por Paul 2026-09-19» de este plan, copiada literal, con la fuente `propuesta.md` §7 (nada queda abierto); (6) la única línea requerida:

```
GATE: GOLD-J ENTREGADO <fecha>
```

- [ ] **Step 10: Commit del kit (el package y `$PRIV_J` no se commitean)**

```bash
grep -c "wisdom-paul" evals/retrieval-heldout/kit-gold-j/README.md evals/retrieval-heldout/kit-gold-j/plantilla.jsonl
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 add evals/retrieval-heldout/kit-gold-j/README.md evals/retrieval-heldout/kit-gold-j/plantilla.jsonl
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 commit -m "eval(j): kit de etiquetado del gold J (README, plantilla por estrato)"
```
Expected: `0` y `0` antes del add.

---

### Task 5: Pre-registro borrador — review adversarial (las decisiones D-J1..D-J10 ya están firmadas)

**Lane:** diseño (spec fábrica: 1 review adversarial del pre-registro de J). Sin gate humano salvo objeción del revisor.
**Oráculo:** el borrador está commiteado en la rama con la cabecera `Estado: BORRADOR`, §9 con las 10 firmas del 2026-09-19 y la fuente `propuesta.md` §7, y `$PRIV_J/review-preregistro.md` existe. Si el revisor objeta una firma con cita, existe además una entrada en `.superpowers/fabrica/pendiente-paul.md` que la reproduce literal.
**Depende de:** Task 2 (la lectura del T0 puede tocar el tope df de F1 en §4 una vez, cláusula de D-J8).

**Files:**
- Modify: `docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md` (solo §4 tope df con evidencia del T0, y erratas/ambigüedades que el review encuentre; cada cambio, una línea `> Enmienda <fecha>:` bajo la cabecera)
- Modify (gitignored): `.superpowers/fabrica/ledger.md`; `.superpowers/fabrica/pendiente-paul.md` **solo** si el revisor objeta una firma

**Interfaces:**
- Consumes: el borrador tal como está en la rama (§9 ya firmado); `evals/retrieval-heldout/verdict/diagnostico-55.md`; la tabla «Decisiones firmadas por Paul 2026-09-19» de este plan.
- Produces: borrador revisado y commiteado; `$PRIV_J/review-preregistro.md`.

- [ ] **Step 1: Review adversarial del borrador**

Despachar un subagente **fable** fresco (u **opus** si la reserva está agotada) con este brief literal:

"Eres el revisor adversarial del pre-registro borrador `docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md`. Léelo entero, más `docs/superpowers/plans/2026-09-13-campana-c-preregistro.md` (precedente) y `evals/retrieval-heldout/verdict/c-verdict.md` §3 y §8 (potencia y erratas de C). Busca: (1) reglas de §6 que se puedan leer de dos maneras; (2) brazos de §4 cuya definición no sea computable sin decisiones humanas; (3) cualquier lugar donde el resultado dependa de elegir entre celdas a la vista de los datos; (4) errores en la tabla de potencia de §7 (recomputa al menos las filas «igual, disc. 10 %» y «mejor 8 pp» con N=100 y NETO ≥ 4 con un script propio: multinomial exacta sobre (ARREGLA, ROMPE)); (5) kill-criteria de §11 que falten para que el veredicto «A0 se queda» sea válido; (6) referencias de §7 marcadas «de memoria»: comprueba en la web si existen tal cual y anota DOI o «no encontrada»; (7) las diez decisiones firmadas de §9 (fuente: `docs/superpowers/consultas/2026-09-15-campanas/propuesta.md` §7): están cerradas, NO las reabras por preferencia; solo si encuentras un error de hecho o de cálculo que invalide la base con la que se firmó una, escríbelo como OBJECIÓN numerada con cita textual y el dato que la contradice. PROHIBIDO proponer brazos nuevos o cambiar los valores únicos de §4. Escribe `$PRIV_J/review-preregistro.md` con hallazgos numerados, cada uno con cita textual y la corrección propuesta."

- [ ] **Step 2: Aplicar solo erratas y ambigüedades**

Aplicar del review únicamente: erratas numéricas confirmadas por el script del revisor, ambigüedades de redacción en §6, kill-criteria que falten en §11, y el estado de cada referencia (DOI o «retirada»). Cada cambio, una línea bajo la cabecera del borrador:
`> Enmienda 2026-09-XX (review adversarial, hallazgo N): <qué cambió, en una frase>.`
Si el T0 (Task 2, sección Lectura) propone otro tope df para F1 (cláusula de D-J8), cambiar el único valor en §4 y añadir la línea `> Enmienda … (T0, cláusula D-J8): tope df de F1 0,25 → X, evidencia: diagnostico-55.md línea «no nulas con ≥1 token raro …»`.

- [ ] **Step 3: Objeciones a firmas (solo si las hay)**

Si el review contiene una OBJECIÓN a una decisión firmada: no se toca §9. Se añade a `.superpowers/fabrica/pendiente-paul.md`, con `date -Iseconds`, un bloque `## Campaña J — objeción del revisor a D-Jn (bloquea: Task 7)` con la objeción literal (cita + dato) y las dos salidas posibles: `D-Jn SE MANTIENE` o `D-Jn CAMBIA A <opción>`. Paul responde con una de las dos; si cambia, la Task 7 sustituye esa línea en §9 citando la respuesta. Sin objeciones, este paso no escribe nada y la Task 7 no espera más que el gold.

- [ ] **Step 4: Commit del borrador**

```bash
head -3 docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md | grep -c "Estado: BORRADOR"
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 add docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 commit -m "docs(j): pre-registro borrador tras review adversarial (sin congelar)"
```
Expected: `1`; commit con hash. Registrar en el ledger `preregistro_borrador: <hash>`.

- [ ] **Step 5: Esperar el gold**

No se despacha la Task 6 hasta que el package lleve `GATE: GOLD-J ENTREGADO` (y, si hubo objeción en el Step 3, la respuesta de Paul). La fábrica puede cerrar aquí y reabrir cuando exista el gold (decisión #13: «esta semana, en paralelo»).

---

### Task 6: Verificación adversarial del gold entregado (D-J3 = sí, firmado) y resumen publicable

**Lane:** diseño (spec fábrica §8: verificación adversarial por candidato con verdict preservado).
**Oráculo:** `valida_gold.py` sobre `$PRIV_J/gold-j.jsonl` en exit 0 tras aplicar las correcciones aceptadas por Paul, y `evals/retrieval-heldout/verdict/gold-j-verificacion-resumen.md` existe con recuentos que cuadran con el verdict privado.
**Depende de:** Task 5 y `GATE: GOLD-J ENTREGADO`.

**Alcance (para no verificar dos veces lo mismo):** las filas `prompt`/`hard` ya pasaron por el verificador en la Task 4 Step 6 antes de que Paul las revisara. Aquí se verifican **todas** las filas `keyword`, `archive` y `negativo` (las escribió y etiquetó Paul) y solo las filas `prompt`/`hard` cuya etiqueta (`expected_permalink` o `acceptable_permalinks`) difiera de `gold-j-preetiquetado.jsonl` (diff por `id`).

**Files:**
- Create: `evals/retrieval-heldout/verdict/gold-j-verificacion-resumen.md`
- Privado: `$PRIV_J/gold-verificacion.md`, `$PRIV_J/filas-a-verificar.txt`, `$PRIV_J/gold-j.jsonl` (corregido solo en las filas que Paul acepte)

**Interfaces:**
- Consumes: `$PRIV_J/gold-j.jsonl`, `$PRIV_J/gold-j-preetiquetado.jsonl`, `$PRIV_J/kb-snap/`.
- Produces: verdict por fila `CORRECTO | DEFENDIBLE | CORREGIR(<nuevo expected/acceptable/null> + razón)` sobre el alcance; gold corregido; resumen público.

- [ ] **Step 1: Validar la entrega**

```bash
python3 evals/retrieval-heldout/harness/valida_gold.py --gold "$PRIV_J/gold-j.jsonl" --kb "$PRIV_J/kb-snap" --in-sample "$PRIV_C/in-sample-55.jsonl" --in-sample "$PRIV_C/gold.jsonl"
```
Expected: exit 0. Si exit 1: devolver a Paul la lista de errores por `id` (package), sin tocar el fichero. Comprobar suelos del borrador §11: `no_nulas ≥ 60`, `nulas_por_estrato.negativo ≥ 24`, `no_nulas_por_estrato.archive ≥ 8`; si no, PENDIENTE-PAUL (no se inventan filas).

Alcance de la verificación:
```bash
python3 - <<'EOF'
import json, os
P = os.path.expanduser("~/.local/share/exo-evals/j-heldout")
pre = {r["id"]: r for r in map(json.loads, filter(str.strip, open(f"{P}/gold-j-preetiquetado.jsonl", encoding="utf-8")))}
ids = []
for r in map(json.loads, filter(str.strip, open(f"{P}/gold-j.jsonl", encoding="utf-8"))):
    p = pre.get(r["id"])
    if p is None or r["source"] in ("keyword", "archive", "negativo") or (r["expected_permalink"], sorted(r["acceptable_permalinks"])) != (p["expected_permalink"], sorted(p["acceptable_permalinks"])):
        ids.append(r["id"])
open(f"{P}/filas-a-verificar.txt", "w", encoding="utf-8").write("\n".join(ids) + "\n")
print(len(ids), "filas a verificar de", len(pre), "pre-etiquetadas +", sum(1 for _ in open(f"{P}/gold-j.jsonl")) - len(pre), "de Paul")
EOF
```
Expected: `≈ 57 + <filas cambiadas por Paul> filas a verificar de <n> pre-etiquetadas + 57 de Paul`.

- [ ] **Step 2: Despachar el verificador**

Subagente **fable** fresco (u **opus**), que no haya participado en las Tasks 4–5. Brief literal:

"Eres el verificador adversarial de un gold de retrieval (formato y criterio de `~/Documentos/proyectos/exo/evals/retrieval-fase0/verdict/labels.md`: léelo primero, es el precedente aprobado). Gold: `$PRIV_J/gold-j.jsonl`; verifica SOLO las filas cuyos ids están en `$PRIV_J/filas-a-verificar.txt` (las demás ya pasaron por un verificador antes de la revisión humana). KB: `$PRIV_J/kb-snap/`, SOLO filesystem (Read/Grep/Glob). PROHIBIDO `exo`, `kbx`, basic-memory, `evals/` salvo `labels.md`, `reports/`, y el gold de C. Para CADA fila del alcance: (1) ¿existe el `expected_permalink` exacto en frontmatter?; (2) ¿es la nota que un usuario razonable querría **con la query sola**?, busca activamente una mejor; (3) cada `acceptable_permalinks`, ¿justificado o comodín?; (4) cada `null` de `prompt`/`hard`: grep de los términos clave y confirma que no hay nota; (5) cada `negativo`: confirma con grep que el tema está ausente (si hay nota, CORREGIR a `prompt`-nula o a positiva, y dilo); (6) cada `archive`: confirma que el hecho preguntado NO está también en la bitácora viva (si está, propón la viva como `acceptable`). Veredicto por fila: CORRECTO / DEFENDIBLE / CORREGIR con la corrección exacta y la cita (línea literal). Al final: recuentos por veredicto y estrato, y las 3 filas más flojas. Escribe en `$PRIV_J/gold-verificacion.md`."

- [ ] **Step 3: Paul acepta o rechaza cada CORREGIR**

Package actualizado con la lista de `CORREGIR` (id, corrección, cita). Paul responde con `ACEPTA <ids>` / `RECHAZA <ids> <motivo>`. Se aplican **solo** las aceptadas, literal. Re-validar (Step 1). Si el suelo cae por debajo tras las correcciones (nulas nuevas), PENDIENTE-PAUL.

- [ ] **Step 4: Resumen publicable**

`evals/retrieval-heldout/verdict/gold-j-verificacion-resumen.md`:
```markdown
# Verificación adversarial del gold — campaña J (resumen publicable)

- Fecha: <date -I>
- Verificador: <modelo> fresco, filesystem-only sobre snapshot `<sha de $PRIV_J/kb-snap.commit>`
- Etiquetado: modo D-J1 = b (agente pre-etiqueta `prompt`+`hard` con verificación previa; Paul escribe `keyword`/`archive`/`negativo` y revisa todo)
- Pre-etiquetas verificadas (Task 4): <n> filas · CORRECTO: <n> · DEFENDIBLE: <n> · CORREGIR: <n>
- Filas verificadas en esta pasada: <n> / <n> del gold (estratos de Paul + filas `prompt`/`hard` que Paul cambió)
- CORRECTO: <n> · DEFENDIBLE: <n> · CORREGIR: <n> (aceptadas por Paul: <n>, rechazadas: <n>)
- Por estrato (no nulas tras corregir): prompt=<n> · keyword=<n> · hard=<n> · archive=<n>
- Nulas: prompt=<n> (inferibilidad) · negativo=<n> (verdaderas; evaluación = ids impares: <n>) · filas con acceptable_permalinks: <n>
- Veredicto completo por fila: privado (`$PRIV_J/gold-verificacion.md`) — el repo es público.
```
Los `<…>` son la salida JSON de `valida_gold.py` y los recuentos del verdict.

- [ ] **Step 5: Commit**

```bash
grep -c "wisdom-paul" evals/retrieval-heldout/verdict/gold-j-verificacion-resumen.md
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 add evals/retrieval-heldout/verdict/gold-j-verificacion-resumen.md
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 commit -m "eval(j): resumen de la verificacion adversarial del gold J"
```
Expected: `0` antes del add.

---

### Task 7: Aprobación del gold y congelación del pre-registro (PENDIENTE-PAUL + mecánica)

**Lane:** gate humano, luego mecánica. **Cierra la fase 1; bloquea toda medición (fase 2).**
**Oráculo:** `git -C <worktree> log -1 --format=%H -- docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md` devuelve el commit de congelación, la cabecera dice `Estado: CONGELADO`, y `sha256sum "$PRIV_J/gold-j.jsonl"` coincide con §10.
**Depende de:** Task 6 y la línea `GATE: GOLD-J APROBADO` de Paul. Las decisiones D-J1..D-J10 ya están firmadas (2026-09-19, `propuesta.md` §7) y en el borrador §9: la congelación **solo espera el gold y su sha256** (más la respuesta de Paul si la Task 5 registró una objeción).

**Files:**
- Modify: `docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md` (cabecera, §10; §9 solo si una objeción de la Task 5 cambió una firma; §4 solo si la cláusula D-J8 del T0 se aplicó y no se aplicó ya en la Task 5)
- Modify (gitignored): `.superpowers/fabrica/packages/j-gold.md`, `.superpowers/fabrica/ledger.md`

**Interfaces:**
- Consumes: `$PRIV_J/gold-j.jsonl` final, salida JSON de `valida_gold.py`, `$PRIV_J/kb-snap.commit`, la línea `GATE:` de Paul.
- Produces: pre-registro congelado (commit), `chmod 444 $PRIV_J/gold-j.jsonl`, `preregistro_congelado: <hash>` en el ledger. La fase 2 arranca desde aquí.

- [ ] **Step 1: Esperar la línea de Paul en el package**

```
GATE: GOLD-J APROBADO <fecha>   |   GATE: GOLD-J RECHAZADO <filas y motivo>
```
`RECHAZADO` ⇒ se corrigen las filas señaladas (vuelve a Task 6 Step 3 para esas filas), nuevo package. Si la Task 5 Step 3 dejó una objeción abierta, también su respuesta (`D-Jn SE MANTIENE` | `D-Jn CAMBIA A <opción>`).

- [ ] **Step 2: Snapshot vigente y validación final**

```bash
S_NOW=$(git -C "$HOME/Documentos/proyectos/wisdom-paul" rev-parse HEAD); cat "$PRIV_J/kb-snap.commit"; echo "$S_NOW"
python3 evals/retrieval-heldout/harness/valida_gold.py --gold "$PRIV_J/gold-j.jsonl" --kb "$PRIV_J/kb-snap" --in-sample "$PRIV_C/in-sample-55.jsonl" --in-sample "$PRIV_C/gold.jsonl"
```
Expected: exit 0 con `"errores": 0`. Si Paul rotó bitácoras durante la semana y quiere medir sobre la KB de hoy: rehacer el snapshot (`rm -rf "$PRIV_J/kb-snap"` y repetir Task 4 Step 1), re-validar; las filas que dejen de resolver vuelven a Paul (no se «arreglan» desde la fábrica).

- [ ] **Step 3: Congelar**

```bash
chmod 444 "$PRIV_J/gold-j.jsonl"
sha256sum "$PRIV_J/gold-j.jsonl"
```
Editar el borrador:
- Cabecera: sustituir el párrafo `**Estado: BORRADOR (2026-09-19).** …` entero (hasta «antes de que exista el gold.») por `**Estado: CONGELADO el <date -Iseconds>.** Inmutable desde este commit; erratas → verdict.` Conservar las líneas `> Enmienda …` de la Task 5.
- §9: ya lleva las diez firmas; solo si la Task 5 registró una objeción y Paul respondió `CAMBIA A`, sustituir esa línea y añadir `(revisada el <fecha> tras objeción del revisor: "<cita de Paul>")`. Si D-J4 cambiara así, sustituir también `NETO ≥ 4` en §6 y en §7 punto 2.
- §4: si la cláusula D-J8 del T0 se aplicó en la Task 5, ya está; si no, nada.
- §10: `S_J` = `kb-snap.commit`; `sha256` y recuentos = JSON de `valida_gold.py`; aprobación = ruta del package + fecha de la línea `GATE:`.

- [ ] **Step 4: Commit de congelación**

```bash
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 add docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 commit -m "eval(j): congela pre-registro del held-out J (gold aprobado por Paul, sha256 fijado)"
git -C /home/paul/Documentos/proyectos/exo/.worktrees/campana-j-fase1 log -1 --format='%H %cI'
```
Expected: hash y fecha. Registrar en el ledger `preregistro_congelado: <hash>`. Desde aquí `git diff <hash> -- docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md` debe salir vacío en cualquier tarea posterior. El nombre del fichero conserva `BORRADOR` a propósito: renombrarlo rompería las citas del plan y del backlog; la cabecera es la verdad.

---

## Lo que queda para la fase 2 (fuera de este plan; se planifica con el pre-registro congelado delante)

Índice `base` sobre `S_J` con el binario post-G/L (rebuild ≈ 1,5 h, `condiciones.md` de C); capturas por query (`captura.py`); módulo `brazos.py` (FTS-AND réplica + oráculo (ii), F1 con `sqlite3`, S1, P1, G1 con calibración) y extensión de `metricas.py informe` para el camino secuencial de §6 y las nulas por estrato; corrida, `j-agregados.md`, `j-verdict.md` adjudicado por fable fresco con cita textual; **solo entonces** `engine/src` para el ganador (con oráculo de reproducción exacta, §11) o cierre «A0 se queda»; backlog, `arquitectura.md` §6 y D6 confirmado. La única tarea de engine independiente del verdict es D-J5 = a (renombrar `similitud_desde_l2_cuadrado` y corregir su doc, sin cambio de comportamiento), que puede ir suelta cuando Paul quiera.

## Self-review (checklist contra el brief)

- T0 con comandos exactos y salida verificable: Tasks 1–2 (código y tests corridos en dry-run el 2026-09-19: 36/40 tests verdes, `miss_hist 6 · miss_S 12 · hit→miss 7`). ✔
- Pre-registro borrador con brazos, métrica primaria (hit@5 + abstención sobre negativos verdaderos), regla de decisión con potencia calculada por N y por número de decisiones, challenge a los 8 brazos (4 decisiones en camino secuencial; RRF, NEAR, normalización absoluta, `sellado-cos`, int8, solape/late cortados con motivo), kill-criteria y «si nada gana». ✔ (borrador §4–§7, §11)
- Kit de etiquetado: plantilla que el harness acepta (Task 3 lo hace aceptar; Task 4 lo valida), instrucciones, estratos, tamaños, estrato `archive` con la respuesta hoy en `archive/log/`, gold privado fuera del repo con sha256 en el pre-registro, estimación honesta de horas (≈ 3,5–4 h con el modo firmado, no «2–3 h»). ✔
- #4 = c: P1 como brazo medido, D-C. #12 = b: no afecta. #13: kit + Paul en paralelo. ✔
- Firmas de Paul 2026-09-19 (D-J1..D-J10, `propuesta.md` §7) reflejadas en: tabla «Decisiones firmadas», Task 4 (pre-etiquetado + verificación previa, D-J2 = 113, package sin líneas abiertas), Task 5 (review puede objetar con cita; no encola decisiones), Task 6 (alcance), Task 7 (solo espera gold + sha256), borrador §3/§4/§6/§7/§9. ✔
- `engine/src` intocado en todas las tareas; solo `cargo build` para leer. ✔
- Placeholders: los únicos `<…>` son campos que se rellenan con salidas de comando o con líneas literales de Paul en el momento del gate (Tasks 2, 4, 6, 7), como en el plan de C. ✔
- Consistencia de nombres: `diagnostico.py` importa `K_HIT`, `UMBRAL_SELLADO`, `carga_captura`, `carga_gold`, `relevantes` (existen en `metricas.py:23-75`), `normaliza` (`pool.py:40`), `permalinks_snapshot` (`valida_gold.py:19`). `pool.muestrea` con n=0 ya devolvía el estrato entero (`pool.py:187-204`); el test lo fija. ✔

## Handoff

Ejecutar con `exo:orchestrate`, tarea a tarea, en la rama `campana-j-fase1`.
Tasks 1 y 3 pueden ir en paralelo; 2 tras 1; 4 tras 3 (ya no espera a Paul:
el modo está firmado); 5 tras 2; 6 y 7 esperan al gold de Paul (decisión
#13: esta semana). La fase 2 se planifica con el pre-registro congelado
delante, nunca antes.
