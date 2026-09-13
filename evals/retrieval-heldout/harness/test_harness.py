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


if __name__ == "__main__":
    unittest.main()
