import contextlib
import hashlib
import io
import json
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import estadistica as E
import validar as V

CLAVES = {"n_universo", "n_analizado", "sin_etiqueta", "citas_no_encontradas", "kappa_b",
          "kappa_base", "precision_b", "precision_base", "confusion_b", "no_cumple_por_choque",
          "delta", "decision"}
AMBITOS = ["git", "codigo", "dominio-proyecto"]


def lab(id, choque, coste="bajo", **kw):
    return {"id": id, "choque": choque, "coste": coste, **kw}


def caso(n=12, sin=()):
    """Universo g1-0..; ambito rota; cumple y choque no correlacionados del todo."""
    ids = [f"g1-{i}" for i in range(n)]
    ambito = {i: AMBITOS[k % 3] for k, i in enumerate(ids)}
    cumple = {i: k % 4 in (0, 1) for k, i in enumerate(ids)}
    choques = ["P", "E", "D", "E", "P", "D", "E", "P", "E", "D", "P", "E"]
    et = {i: lab(i, choques[k % 12]) for k, i in enumerate(ids) if i not in sin}
    return et, ids, ambito, cumple


def vectores(et, ids, ambito, cumple):
    ok = [i for i in ids if i in et]
    return ([et[i]["choque"] != "E" for i in ok],
            [ambito[i] != "dominio-proyecto" for i in ok],
            [not cumple[i] for i in ok])


class TestCalcula(unittest.TestCase):
    def test_baseline_git_codigo(self):
        et, ids, ambito, cumple = caso()
        r = V.calcula(et, ids, ambito, cumple)
        _, base, verdad = vectores(et, ids, ambito, cumple)
        self.assertEqual(r["kappa_base"], E.kappa(base, verdad))
        # cambiar la criba no mueve el baseline; cambiar el ámbito sí
        r2 = V.calcula(et, ids, ambito, {i: not c for i, c in cumple.items()})
        self.assertEqual(r2["kappa_base"], E.kappa(base, [not v for v in verdad]))
        amb2 = dict(ambito, **{"g1-0": "dominio-proyecto", "g1-2": "git"})
        _, base2, _ = vectores(et, ids, amb2, cumple)
        self.assertEqual(V.calcula(et, ids, amb2, cumple)["kappa_base"], E.kappa(base2, verdad))
        self.assertNotEqual(base, base2)

    def test_sin_etiqueta_excluida_de_ambos(self):
        et, ids, ambito, cumple = caso(sin=("g1-0", "g1-1"))
        r = V.calcula(et, ids, ambito, cumple)
        self.assertEqual(r["sin_etiqueta"], 2)
        self.assertEqual(r["n_analizado"], r["n_universo"] - 2)
        b, base, verdad = vectores(et, ids, ambito, cumple)
        self.assertEqual(len(b), 10)
        self.assertEqual(r["kappa_b"], E.kappa(b, verdad))
        self.assertEqual(r["kappa_base"], E.kappa(base, verdad))
        r_todas = V.calcula(caso()[0], ids, ambito, cumple)
        self.assertNotEqual(r["kappa_b"], r_todas["kappa_b"])
        self.assertNotEqual(r["kappa_base"], r_todas["kappa_base"])

    def test_decision_cableada(self):
        ids = [f"g1-{i}" for i in range(8)]
        ambito = {i: "git" for i in ids}
        cumple = {i: k < 4 for k, i in enumerate(ids)}  # no_cumple = última mitad
        # predicción invertida respecto de la verdad: kappa_b < 0
        et = {i: lab(i, "P" if k < 4 else "E") for k, i in enumerate(ids)}
        r = V.calcula(et, ids, ambito, cumple)
        self.assertLessEqual(r["kappa_b"], 0)
        self.assertFalse(r["decision"]["criterio_2"])
        # grupo D vacío: n 0, proporcion null, sin excepción
        self.assertEqual(r["no_cumple_por_choque"]["D"], {"n": 0, "no_cumple": 0, "proporcion": None})
        self.assertEqual(r["no_cumple_por_choque"]["P"], {"n": 4, "no_cumple": 0, "proporcion": 0.0})
        et2, ids2, amb2, cum2 = caso(n=12, sin=("g1-0", "g1-1", "g1-2", "g1-3"))
        self.assertEqual(V.calcula(et2, ids2, amb2, cum2)["decision"]["estado"], "nula")
        original = E.decide
        E.decide = lambda k, n: "CENTINELA"
        try:
            self.assertEqual(V.calcula(et, ids, ambito, cumple)["decision"], "CENTINELA")
        finally:
            E.decide = original

    def test_id_fuera_de_universo(self):
        et, ids, ambito, cumple = caso()
        et["zz-9"] = lab("zz-9", "P")
        del et["g1-3"]
        err = io.StringIO()
        with contextlib.redirect_stderr(err):
            r = V.calcula(et, ids, ambito, cumple)
        self.assertIn("1", err.getvalue())
        self.assertIn("universo", err.getvalue())
        self.assertEqual(r["sin_etiqueta"], 1)
        self.assertEqual(r["n_analizado"], len(ids) - 1)

    def test_citas_no_encontradas(self):
        et, ids, ambito, cumple = caso()
        et["g1-0"]["cita_recortada"] = False
        et["g1-1"]["cita_recortada"] = True
        self.assertEqual(V.calcula(et, ids, ambito, cumple)["citas_no_encontradas"], 1)

    def test_resultado_sin_texto(self):
        with tempfile.TemporaryDirectory() as d:
            pool, criba = os.path.join(d, "pool"), os.path.join(d, "criba")
            os.makedirs(pool)
            et, ids, ambito, cumple = caso()
            secretos = []
            with open(os.path.join(pool, "reglas-g1.jsonl"), "w") as f:
                for i in ids:
                    s = f"texto-secreto-{i}-xyzzy"
                    secretos.append(s)
                    f.write(json.dumps({"id": i, "ambito": ambito[i], "regla": s, "cita": "cita-" + s}) + "\n")
            for i in ids:
                for rep, rc in (("a0-r1", 0 if cumple[i] else 1), ("a0-r2", 0)):
                    p = os.path.join(criba, i, rep)
                    os.makedirs(p)
                    with open(os.path.join(p, "check.rc"), "w") as f:
                        f.write(f"{rc}\n")
            etq, uni, sal = (os.path.join(d, n) for n in ("e.jsonl", "u.txt", "r.json"))
            with open(etq, "w") as f:
                f.write("".join(json.dumps(e) + "\n" for e in et.values()))
            with open(uni, "w") as f:
                f.write("\n".join(ids) + "\n")
            with contextlib.redirect_stdout(io.StringIO()):
                V.main(["--etiquetas", etq, "--universo", uni, "--pool", pool, "--criba", criba, "--salida", sal])
            with open(sal) as f:
                crudo = f.read()
            res = json.loads(crudo)
            self.assertEqual(set(res), CLAVES)
            for s in secretos:
                self.assertNotIn(s, crudo)
            self.assertNotIn("xyzzy", crudo)
            # la verdad sale de la criba (todas las réplicas a 0), no de otra cosa
            self.assertEqual(res["n_analizado"], len(ids))
            esperado = V.calcula(et, ids, ambito, cumple)
            self.assertEqual(res["kappa_b"], esperado["kappa_b"])

    def _entorno(self, d, sin_rc=()):
        pool, criba = os.path.join(d, "pool"), os.path.join(d, "criba")
        os.makedirs(pool)
        et, ids, ambito, cumple = caso()
        with open(os.path.join(pool, "reglas-g1.jsonl"), "w") as f:
            for i in ids:
                f.write(json.dumps({"id": i, "ambito": ambito[i], "regla": "r", "cita": "c"}) + "\n")
        for i in ids:
            if i in sin_rc:
                continue
            p = os.path.join(criba, i, "a0-r1")
            os.makedirs(p)
            with open(os.path.join(p, "check.rc"), "w") as f:
                f.write("0\n" if cumple[i] else "1\n")
        etq, uni, sal = (os.path.join(d, n) for n in ("e.jsonl", "u.txt", "r.json"))
        with open(etq, "w") as f:
            f.write("".join(json.dumps(e) + "\n" for e in et.values()))
        with open(uni, "w") as f:
            f.write("\n".join(ids) + "\n")
        args = ["--etiquetas", etq, "--universo", uni, "--pool", pool, "--criba", criba, "--salida", sal]
        return args, uni, sal, ids

    def test_sello_del_universo(self):
        with tempfile.TemporaryDirectory() as d:
            args, uni, sal, _ = self._entorno(d)
            with open(uni, "rb") as f:
                sha = hashlib.sha256(f.read()).hexdigest()
            malo, bueno = os.path.join(d, "malo.json"), os.path.join(d, "bueno.json")
            with open(malo, "w") as f:
                json.dump({"universo_sha256": "0" * 64}, f)
            with open(bueno, "w") as f:
                json.dump({"universo_sha256": sha}, f)
            with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as cm:
                    V.main(args + ["--sello", malo])
                self.assertEqual(cm.exception.code, 2)
                self.assertFalse(os.path.exists(sal))
                V.main(args + ["--sello", bueno])
            self.assertTrue(os.path.exists(sal))

    def test_id_sin_rc_se_avisa(self):
        with tempfile.TemporaryDirectory() as d:
            args, _, _, ids = self._entorno(d, sin_rc=("g1-0",))
            err = io.StringIO()
            with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(err):
                V.main(args)
        self.assertIn("g1-0", err.getvalue())
        self.assertNotIn(ids[2], err.getvalue())


if __name__ == "__main__":
    unittest.main()
