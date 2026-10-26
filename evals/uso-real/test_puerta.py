import os
import sys
import unittest
from unittest import mock

sys.path.insert(0, os.path.dirname(__file__))
import puerta  # noqa: E402


class TestIC(unittest.TestCase):
    def test_ic_valores_referencia(self):
        for k, (lo, hi) in {
            0: (0.0, 3.6889),
            1: (0.0253, 5.5716),
            10: (4.7954, 18.3904),
            40: (28.5766, 54.4686),
        }.items():
            got = puerta.ic_poisson(k)
            self.assertAlmostEqual(got[0], lo, places=3, msg=k)
            self.assertAlmostEqual(got[1], hi, places=3, msg=k)
        # Segunda vía independiente: IC exacto = cuantiles chi-cuadrado
        # (lo = chi2.ppf(.025, 2k)/2, hi = chi2.ppf(.975, 2k+2)/2), calculados
        # una vez con scipy fuera del repo.
        lo, hi = puerta.ic_poisson(100)
        self.assertAlmostEqual(lo, 81.3640, places=3)
        self.assertAlmostEqual(hi, 121.6268, places=3)
        lo, hi = puerta.ic_poisson(2000)
        self.assertAlmostEqual(lo, 1913.2987, places=3)
        self.assertAlmostEqual(hi, 2089.6174, places=3)


class TestDecide(unittest.TestCase):
    def test_decide_por_debajo(self):
        r = puerta.decide(10, 100)
        self.assertEqual(r["estado"], "estabilizar")
        self.assertEqual(r["tasa"], 0.1)
        self.assertEqual((r["k"], r["n"]), (10, 100))
        self.assertAlmostEqual(r["ic"][1], 0.1839, places=4)

    def test_decide_cruza(self):
        r = puerta.decide(40, 100)
        self.assertEqual(r["estado"], "etapa2")
        self.assertAlmostEqual(r["ic"][0], 0.2858, places=4)
        self.assertAlmostEqual(r["ic"][1], 0.5447, places=4)
        # tasa puntual 0,30 < 0,4 pero el IC superior (0,428) cruza
        self.assertEqual(puerta.decide(30, 100)["estado"], "etapa2")

    def test_decide_borde(self):
        r27, r28 = puerta.decide(27, 100), puerta.decide(28, 100)
        self.assertEqual(r27["estado"], "estabilizar")
        self.assertAlmostEqual(r27["ic"][1], 0.3928, places=4)
        self.assertEqual(r28["estado"], "etapa2")
        self.assertAlmostEqual(r28["ic"][1], 0.4047, places=4)

    def test_decide_borde_estricto(self):
        with mock.patch.object(puerta, "ic_poisson", return_value=(0.0, 40.0)):
            self.assertEqual(puerta.decide(5, 100)["estado"], "etapa2")

    def test_decide_sin_sesiones(self):
        with self.assertRaises(ValueError):
            puerta.decide(0, 0)
        with self.assertRaises(ValueError):
            puerta.final([], 0)


class TestFinal(unittest.TestCase):
    def test_final_unidad_vacia(self):
        a = ["canon-kb"] * 2 + ["nueva"] * 28
        self.assertEqual(puerta.final(a, 100)["estado"], "unidad_vacia")
        # n < 30: la regla no aplica
        self.assertNotEqual(puerta.final(a[1:], 100)["estado"], "unidad_vacia")
        # 3/30 = 0,10 exacto: no es < 0,10
        b = ["canon-kb"] * 3 + ["nueva"] * 27
        self.assertNotEqual(puerta.final(b, 100)["estado"], "unidad_vacia")
        c = ["perfil-core"] * 2 + ["nueva"] * 38
        self.assertEqual(puerta.final(c, 100)["estado"], "unidad_vacia")

    def test_final_decididas(self):
        r = puerta.final(["regla-canal"] * 4, 100)
        self.assertEqual(r["estado"], "estabilizar")
        self.assertEqual(r["n_correcciones"], 4)
        self.assertEqual(r["decididas"], 4)
        self.assertEqual(r["por_atribucion"], {"regla-canal": 4})
        self.assertEqual(r["tasa_decididas"], 0.04)
        self.assertEqual(puerta.final(["canon-kb"] * 60, 100)["estado"], "seguir")
        r = puerta.final(["canon-kb"] * 30, 100)
        self.assertEqual(r["estado"], "seguir")
        self.assertEqual(r["tasa_decididas"], 0.3)

    def test_decididas_const(self):
        self.assertEqual(
            puerta.DECIDIDAS,
            ("regla-canal", "regla-fuera-canal", "canon-kb", "perfil-core"),
        )


if __name__ == "__main__":
    unittest.main()
