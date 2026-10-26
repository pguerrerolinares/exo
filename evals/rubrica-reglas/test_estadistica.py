import os
import random
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import estadistica as E

T, F = True, False


def _datos():
    rng = random.Random(7)
    verdad = [rng.random() < 0.5 for _ in range(40)]
    pred_b = [v if rng.random() < 0.8 else not v for v in verdad]
    pred_base = [v if rng.random() < 0.6 else not v for v in verdad]
    return pred_b, pred_base, verdad


class TestKappa(unittest.TestCase):
    def test_kappa_perfecta(self):
        v = [T, T, F, F, T, F]
        self.assertEqual(E.kappa(v, v), 1.0)
        self.assertEqual(E.kappa([T, T, F, F], [F, F, T, T]), -1.0)

    def test_kappa_degenerada(self):
        self.assertIsNone(E.kappa([T] * 4, [T] * 4))
        self.assertEqual(E.kappa([T] * 4, [T, T, F, F]), 0.0)

    def test_kappa_conocida(self):
        # marginales 5T/5F en ambos -> pe = 0,5; 8 de 10 coinciden -> po = 0,8 -> kappa 0,6
        pred = [T] * 5 + [F] * 5
        verdad = [T] * 4 + [F] + [T] + [F] * 4
        self.assertAlmostEqual(E.kappa(pred, verdad), 0.6)


class TestDecide(unittest.TestCase):
    def test_decide_kappa_cero_cae(self):
        for k in (0.0, None):
            d = E.decide(k, 0)
            self.assertFalse(d["criterio_2"])
            self.assertEqual(d["rubrica"], "veto + dato + coste + fichero")
        d = E.decide(0.01, 0)
        self.assertTrue(d["criterio_2"])
        self.assertEqual(d["rubrica"], "D")

    def test_decide_nula(self):
        self.assertEqual(E.decide(0.5, 4)["estado"], "nula")
        self.assertEqual(E.decide(0.5, 3)["estado"], "valida")


class TestBootstrap(unittest.TestCase):
    def test_bootstrap_identicos_contiene_cero(self):
        pb, pbase, v = _datos()
        r = E.bootstrap_delta(pb, pb, v, n=500)
        self.assertEqual(r["ic95"], [0.0, 0.0])
        self.assertEqual(r["delta"], 0)
        r = E.bootstrap_delta(pb, pbase, v, n=500)
        self.assertAlmostEqual(r["delta"], E.kappa(pb, v) - E.kappa(pbase, v))
        self.assertLess(r["ic95"][0], r["ic95"][1])

    def test_bootstrap_reproducible(self):
        pb, pbase, v = _datos()
        random.seed(1)
        a = E.bootstrap_delta(pb, pbase, v, n=500)["ic95"]
        random.seed(2)
        b = E.bootstrap_delta(pb, pbase, v, n=500)["ic95"]
        c = E.bootstrap_delta(pb, pbase, v, n=500, semilla=1)["ic95"]
        self.assertEqual(a, b)
        self.assertNotEqual(a, c)

    def test_bootstrap_cuenta_descartados(self):
        v = [T, T, T, F]
        for n, lo, hi in ((10000, 2500, 4000), (2000, 500, 800)):
            r = E.bootstrap_delta(v, [T] * 4, v, n=n)
            self.assertTrue(lo <= r["descartados"] <= hi, r["descartados"])
            self.assertEqual(r["delta"], 1.0)
            self.assertEqual(r["ic95"], [1.0, 1.0])

    def test_ic95_percentiles(self):
        # 1..100 desordenado: percentil 2,5 -> índice 2 (valor 3), 97,5 -> índice 97 (valor 98)
        self.assertEqual(E._ic95(list(range(100, 0, -1))), [3, 98])

    def test_bootstrap_todo_descartado(self):
        r = E.bootstrap_delta([T] * 4, [T] * 4, [T] * 4, n=50)
        self.assertIsNone(r["delta"])
        self.assertEqual(r["ic95"], [None, None])
        self.assertEqual(r["descartados"], 50)


if __name__ == "__main__":
    unittest.main()
