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


class TestJuezPresupuesto(unittest.TestCase):
    """Enmienda 2026-09-20 (orquestador, Task 4 de J): circuit breaker de
    gasto real en `juez.procesa_lote`. Sin red: el HTTP se inyecta, como en
    TestJuez.test_kimi_con_http_falso."""

    def _paquetes(self, d, ids):
        p = Path(d) / "paquetes.jsonl"
        with open(p, "w", encoding="utf-8") as fh:
            for i in ids:
                fh.write(json.dumps({"id": i, "candidatos": ["kb/a"], "texto": f"CONSULTA: {i}"}, ensure_ascii=False) + "\n")
        return p

    def test_breaker_para_en_seco_y_no_hace_la_siguiente_llamada(self):
        with tempfile.TemporaryDirectory() as d:
            ruta = self._paquetes(d, ["c1", "c2", "c3"])
            out = Path(d) / "out.jsonl"
            llamadas = []

            def http(url, key, cuerpo=None, timeout=120):
                llamadas.append(cuerpo)
                return {"choices": [{"message": {"content": '{"expected": "kb/a", "acceptable": [], "razon": "ok"}'}}],
                        "usage": {"prompt_tokens": 200000, "completion_tokens": 20000}}

            # coste por llamada = 200000/1e6*3 + 20000/1e6*15 = 0.6 + 0.3 = 0.9
            # tope 1.0: tras c1 (0.9) sigue; tras c2 (1.8) para; c3 NO se llama.
            resultado = jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                         precio_entrada=3.0, precio_salida=15.0,
                                         tope_usd=1.0, http=http)
            self.assertEqual(len(llamadas), 2)
            self.assertEqual(resultado["parada_por_tope"], "c2")
            self.assertAlmostEqual(resultado["gasto_usd"], 1.8)
            with open(out, encoding="utf-8") as fh:
                escritas = [json.loads(l)["id"] for l in fh]
            self.assertEqual(escritas, ["c1", "c2"])

    def test_coste_calculado_con_precios_conocidos(self):
        with tempfile.TemporaryDirectory() as d:
            ruta = self._paquetes(d, ["c1"])
            out = Path(d) / "out.jsonl"
            llamadas = []

            def http(url, key, cuerpo=None, timeout=120):
                llamadas.append(cuerpo)
                return {"choices": [{"message": {"content": '{"expected": "kb/a", "acceptable": [], "razon": "ok"}'}}],
                        "usage": {"prompt_tokens": 1000000, "completion_tokens": 500000}}

            # 1.000.000/1e6*3.0 + 500.000/1e6*15.0 = 3.0 + 7.5 = 10.5
            resultado = jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                         precio_entrada=3.0, precio_salida=15.0,
                                         tope_usd=100.0, http=http)
            self.assertEqual(len(llamadas), 1)
            self.assertNotIn("parada_por_tope", resultado)
            self.assertAlmostEqual(resultado["gasto_usd"], 10.5)


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
