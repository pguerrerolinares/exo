"""Tests de muestra.py y validar.py con datos sintéticos."""
import json, os, subprocess, sys, tempfile, unittest

H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, H)
import detectar, muestra, validar


def msg(i, texto, previo="prev"):
    return {"id": f"m{i:03d}", "previo": previo, "texto": texto}


def corre(script, *args):
    return subprocess.run([sys.executable, os.path.join(H, script), *args], capture_output=True, text=True)


def jsonl(path, objs):
    with open(path, "w") as f:
        for o in objs:
            f.write(json.dumps(o) + "\n")


class TestMuestra(unittest.TestCase):
    def test_regex_sin_tildes(self):
        for t in ("Déjate de golds", "ESTÁ MAL", "nunca", "nuncaa"):
            self.assertTrue(muestra.candidato_regex(t), t)
        self.assertFalse(muestra.candidato_regex("vale, sigue"))

    def mensajes(self, n_casan, n_resto):
        return [msg(i, "no uses eso") for i in range(n_casan)] + [msg(1000 + i, "vale, sigue") for i in range(n_resto)]

    def test_muestra_etapa1_reproducible(self):
        ms = self.mensajes(100, 400)
        a, b = muestra.etapa1(ms, 7), muestra.etapa1(ms, 7)
        self.assertEqual([x["id"] for x in a], [x["id"] for x in b])
        ids = [x["id"] for x in a]
        self.assertEqual(len(ids), 80)
        self.assertEqual(len(set(ids)), 80)
        self.assertEqual(sum(int(i[1:]) < 1000 for i in ids), 40)
        self.assertNotEqual(ids, sorted(ids))
        self.assertFalse(all(int(i[1:]) < 1000 for i in ids[:40]))  # barajado: no regex-primero
        self.assertNotEqual(ids, [x["id"] for x in muestra.etapa1(ms, 8)])

    def test_muestra_pocos_regex(self):
        ids = [x["id"] for x in muestra.etapa1(self.mensajes(25, 400), 1)]
        self.assertEqual(len(ids), 80)
        self.assertEqual(sum(int(i[1:]) < 1000 for i in ids), 25)

    def test_muestra_ciega(self):
        # El juez da veredicto a todo; la muestra se construye solo desde los mensajes.
        ms = [{**m, "correccion": "si", "tipo": "redirige", "veredicto": "x"} for m in self.mensajes(50, 50)]
        res = muestra.etapa1(ms)
        self.assertEqual(len(res), 80)
        for it in res:
            self.assertEqual(set(it), {"id", "previo", "texto", "etiqueta"})
            self.assertIsNone(it["etiqueta"])

    def test_mismo_truncado_que_el_juez(self):
        it = muestra.etapa1([msg(1, "a" * 3000 + "FIN", "P" * 5000 + "cola")])[0]
        self.assertEqual(it["texto"], "a" * detectar.MAX_TEXTO)
        self.assertEqual(len(it["previo"]), detectar.MAX_PREVIO)
        self.assertTrue(it["previo"].endswith("cola"))

    def test_estratos_etapa2(self):
        at = [{"id": f"a{i:03d}", "atribucion": k, "previo": "p", "texto": "t", "nota": "n", "linea": "L"}
              for i, k in enumerate(["nueva"] * 60 + ["canon-kb"] * 30 + ["regla-canal"])]
        res = muestra.etapa2(at)
        ids = [x["id"] for x in res]
        self.assertEqual(len(ids), 30)
        self.assertEqual(len(set(ids)), 30)
        por = {}
        for i in ids:
            k = at[int(i[1:])]["atribucion"]
            por[k] = por.get(k, 0) + 1
        self.assertGreaterEqual(por["regla-canal"], 1)
        self.assertLessEqual(abs(por["nueva"] - 20), 1)
        self.assertLessEqual(abs(por["canon-kb"] - 10), 1)
        for it in res:
            self.assertEqual(set(it), {"id", "previo", "texto", "etiqueta", "nota", "atribucion_dueño"})


class TestValidar(unittest.TestCase):
    def datos(self, n, acuerdos):
        """Mitad pred si / mitad verdad si; (n - acuerdos) discrepancias repartidas por igual -> pe = 0.5."""
        mitad, d = n // 2, (n - acuerdos) // 2
        pred = [True] * mitad + [False] * mitad
        verdad = [True] * (mitad - d) + [False] * d + [True] * d + [False] * (mitad - d)
        return pred, verdad

    def test_validar_umbral(self):
        for n, ac, valido in ((100, 70, True), (400, 278, False)):
            pred, verdad = self.datos(n, ac)
            with tempfile.TemporaryDirectory() as d:
                ids = [f"m{i}" for i in range(n)]
                jsonl(f"{d}/e", [{"id": i, "etiqueta": "si" if v else "no"} for i, v in zip(ids, verdad)])
                jsonl(f"{d}/d", [{"sesion": "s", "id": i, "correccion": "si" if p else "no", "tipo": None}
                                 for i, p in zip(ids, pred)])
                r = corre("validar.py", "--etapa", "1", "--etiquetas", f"{d}/e", "--detecciones", f"{d}/d", "--salida", f"{d}/o")
                self.assertEqual(r.returncode, 0, r.stderr)
                out = json.load(open(f"{d}/o"))
            self.assertEqual(out["juez_valido"], valido, out)
            self.assertEqual(out["n"], n)

    def test_metricas(self):
        m = validar.metricas([True, True, False, False], [True, False, True, False])
        self.assertEqual((m["precision"], m["recall"], m["n"]), (0.5, 0.5, 4))
        self.assertIsNone(validar.metricas([False, False], [False, False])["kappa"])

    def test_validar_null_exit_2(self):
        with tempfile.TemporaryDirectory() as d:
            det = [{"sesion": "s", "id": f"m{i}", "correccion": "si", "tipo": "redirige"} for i in range(3)]
            jsonl(f"{d}/d", det)
            jsonl(f"{d}/e", [{"id": "m0", "etiqueta": "si"}, {"id": "m1", "etiqueta": None}, {"id": "m2", "etiqueta": "no"}])
            args = ("--etapa", "1", "--etiquetas", f"{d}/e", "--detecciones", f"{d}/d", "--salida", f"{d}/o")
            r = corre("validar.py", *args)
            self.assertEqual(r.returncode, 2)
            self.assertIn("m1", r.stderr)
            self.assertNotIn("m0", r.stderr)
            self.assertFalse(os.path.exists(f"{d}/o"))
            jsonl(f"{d}/e", [{"id": "m0", "etiqueta": "si"}, {"id": "m1", "etiqueta": "si"}, {"id": "m2", "etiqueta": "no"}])
            r = corre("validar.py", *args)
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertTrue(os.path.exists(f"{d}/o"))

    def test_sin_veredicto_aparte_y_kappa_none(self):
        with tempfile.TemporaryDirectory() as d:
            jsonl(f"{d}/d", [{"sesion": "s", "id": "a", "sin_veredicto": True},
                             {"sesion": "s", "id": "b", "correccion": "no", "tipo": None},
                             {"sesion": "s", "id": "c", "correccion": "no", "tipo": None}])
            jsonl(f"{d}/e", [{"id": i, "etiqueta": "no"} for i in "abc"])
            r = corre("validar.py", "--etapa", "1", "--etiquetas", f"{d}/e", "--detecciones", f"{d}/d", "--salida", f"{d}/o")
            self.assertEqual(r.returncode, 0, r.stderr)
            out = json.load(open(f"{d}/o"))
        self.assertEqual((out["n"], out["n_sin_veredicto"]), (2, 1))
        self.assertIsNone(out["kappa"])
        self.assertFalse(out["juez_valido"])


class TestCLIMuestra(unittest.TestCase):
    def test_etapa1_escribe_en_cache(self):
        with tempfile.TemporaryDirectory() as d:
            at = [{"id": f"a{i}", "atribucion": "nueva", "previo": "p", "texto": "t", "nota": None} for i in range(5)]
            jsonl(f"{d}/pos.jsonl", at)
            r = corre("muestra.py", "--etapa", "2", "--cache", f"{d}/c", "--positivos", f"{d}/pos.jsonl")
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertEqual(len(open(f"{d}/c/etiquetar-etapa2.jsonl").readlines()), 5)


if __name__ == "__main__":
    unittest.main()
