#!/usr/bin/env python3
"""Tests del harness de la campaña C. Uso:
python3 -m unittest -v evals/retrieval-heldout/harness/test_harness.py"""
import json
import sqlite3
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
        # Ventanas por fuente (pre-registro §7): prompt desde 2026-08-22,
        # agent-search desde 2026-07-19; fin común 2026-09-13 (exclusivo).
        in55 = [pl.normaliza("fabrica campaña")]
        cands = [
            {"query": "Fabrica  campaña", "source": "agent-search", "session_id": "s", "ts": "2026-07-20T00:00:00Z"},
            {"query": "-revisa esto", "source": "prompt", "session_id": "s", "ts": "2026-08-23T00:00:00Z"},
            {"query": "x" * 1501, "source": "prompt", "session_id": "s", "ts": "2026-08-23T00:00:00Z"},
            {"query": "memoria v2 contrato", "source": "agent-search", "session_id": "s", "ts": "2026-09-13T08:00:00Z"},
            {"query": "memoria v2 contrato", "source": "agent-search", "session_id": "s", "ts": "2026-07-20T00:00:00Z"},
            {"query": "Memoria v2  contrato", "source": "agent-search", "session_id": "t", "ts": "2026-07-21T00:00:00Z"},
            # prompt anterior al inicio de su ventana (2026-08-22): fuera aunque agent-search ya la admitiría.
            {"query": "prompt antes de ventana", "source": "prompt", "session_id": "s", "ts": "2026-08-21T00:00:00Z"},
            # 2026-08-22T01:00:00+02:00 == 2026-08-21T23:00:00Z: cronológicamente ANTES del
            # inicio de la ventana prompt, pero la comparación de strings ("...T01..." >
            # "...T00...Z") la colaría como si estuviera dentro.
            {"query": "offset cruza medianoche", "source": "prompt", "session_id": "s", "ts": "2026-08-22T01:00:00+02:00"},
        ]
        pool_, desc = pl.filtra(cands, in55)
        self.assertEqual([c["query"] for c in pool_], ["memoria v2 contrato"])
        self.assertEqual(desc, {"vacia": 0, "guion": 1, "larga": 1, "fuera-de-ventana": 3, "dup-55": 1, "dup-pool": 1})

    def test_filtra_con_ventanas_propias(self):
        c = {"query": "una query de J", "source": "prompt", "session_id": "s", "ts": "2026-09-15T00:00:00Z"}
        self.assertEqual(pl.filtra([c], [])[0], [])
        v = {"prompt": ("2026-09-13T00:00:00Z", "2026-09-20T00:00:00Z"), "agent-search": ("2026-09-13T00:00:00Z", "2026-09-20T00:00:00Z")}
        self.assertEqual(len(pl.filtra([c], [], v)[0]), 1)

    def test_muestrea_cuota_cero_devuelve_estrato_entero(self):
        pool = [{"query": f"q{i}", "source": "prompt", "ts": f"2026-09-1{i}T00:00:00Z"} for i in range(4)]
        self.assertEqual(len(pl.muestrea(pool, [("prompt", 0)])), 4)

    def test_muestrea_es_independiente_del_orden_de_las_cuotas(self):
        pool_ = (
            [{"source": "prompt", "query": f"p{i}", "ts": f"2026-08-{22 + i % 8:02d}T00:00:00Z"} for i in range(12)]
            + [{"source": "agent-search", "query": f"a{i}", "ts": f"2026-07-{19 + i % 8:02d}T00:00:00Z"} for i in range(12)]
        )
        m1 = pl.muestrea(pool_, [("prompt", 5), ("agent-search", 5)])
        m2 = pl.muestrea(pool_, [("agent-search", 5), ("prompt", 5)])
        self.assertEqual([c["query"] for c in m1 if c["source"] == "prompt"],
                          [c["query"] for c in m2 if c["source"] == "prompt"])
        self.assertEqual([c["query"] for c in m1 if c["source"] == "agent-search"],
                          [c["query"] for c in m2 if c["source"] == "agent-search"])

    def test_muestrea_insuficiente_levanta_pool_insuficiente(self):
        pool_ = [{"source": "prompt", "query": "p1", "ts": "2026-08-22T00:00:00Z"}]
        with self.assertRaises(pl.PoolInsuficiente):
            pl.muestrea(pool_, [("prompt", 2)])

    def test_mas_cercano_elige_menor_delta(self):
        ts_evento = pl._ts("2026-08-23T10:00:00Z")
        mensajes = [
            {"ts": pl._ts("2026-08-23T09:59:30Z"), "query": "lejos-antes"},
            {"ts": pl._ts("2026-08-23T10:00:01Z"), "query": "cerca-despues"},
        ]
        self.assertEqual(pl.mas_cercano(ts_evento, mensajes)["query"], "cerca-despues")

    def test_mas_cercano_empate_elige_el_anterior(self):
        ts_evento = pl._ts("2026-08-23T10:00:00Z")
        mensajes = [
            {"ts": pl._ts("2026-08-23T09:59:55Z"), "query": "antes"},
            {"ts": pl._ts("2026-08-23T10:00:05Z"), "query": "despues"},
        ]
        self.assertEqual(pl.mas_cercano(ts_evento, mensajes)["query"], "antes")

    def test_mas_cercano_sin_mensajes_es_none(self):
        self.assertIsNone(pl.mas_cercano(pl._ts("2026-08-23T10:00:00Z"), []))


import valida_gold as vg  # noqa: E402
import diagnostico as dg  # noqa: E402


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


if __name__ == "__main__":
    unittest.main()
