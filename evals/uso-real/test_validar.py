"""Tests de muestra.py y validar.py con datos sintéticos."""
import json, os, subprocess, sys, tempfile, unittest

H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, H)
import detectar, muestra, validar


def msg(i, texto, previo="prev"):
    return {"id": f"m{i:03d}", "previo": previo, "texto": texto}


def corre(script, *args):
    # cwd ajeno: los imports de los CLI no pueden depender del directorio de trabajo.
    return subprocess.run([sys.executable, os.path.join(H, script), *args], capture_output=True, text=True,
                          cwd=tempfile.gettempdir())


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
        at = [{"id": f"a{i:03d}", "atribucion": k, "previo": "p", "texto": "t", "nota": "n", "linea": "L", "contexto_kb": {"nota": "NOTA", "core": {}}}
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
            self.assertEqual(set(it), {"id", "previo", "texto", "etiqueta", "contexto_kb", "atribucion_dueño"})


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


def escribe_sesion(proyectos, sid, filas):
    """filas: [(uuid, texto)]; cada mensaje va precedido de un turno del asistente `previo-<n>`."""
    os.makedirs(f"{proyectos}/p", exist_ok=True)
    with open(f"{proyectos}/p/{sid}.jsonl", "w") as f:
        for n, (u, t) in enumerate(filas):
            base = {"timestamp": f"2026-10-10T10:00:{n:02d}Z", "cwd": "/w/x", "entrypoint": "cli"}
            f.write(json.dumps({**base, "type": "assistant", "uuid": f"a{n}", "message": {
                "role": "assistant", "content": [{"type": "text", "text": f"previo-{n}"}]}}) + "\n")
            f.write(json.dumps({**base, "type": "user", "uuid": u, "message": {"role": "user", "content": t}}) + "\n")


class TestCLIMuestra(unittest.TestCase):
    def test_etapa1_y_regex_escriben_en_cache(self):
        with tempfile.TemporaryDirectory() as d:
            escribe_sesion(f"{d}/pr", "s1", [(f"u{i}", "no uses eso" if i < 3 else "vale") for i in range(6)])
            for etapa, n in (("1", 6), ("regex", 3)):
                r = corre("muestra.py", "--etapa", etapa, "--cache", f"{d}/c", "--proyectos", f"{d}/pr")
                self.assertEqual(r.returncode, 0, r.stderr)
                fich = open(f"{d}/c/etiquetar-{'etapa1' if etapa == '1' else 'regex'}.jsonl").readlines()
                self.assertEqual(len(fich), n)
            self.assertEqual(set(json.loads(fich[0])), {"id", "previo", "texto", "etiqueta"})

    def test_etapa2_lee_salida_y_cache_de_atribuir(self):
        with tempfile.TemporaryDirectory() as d:
            escribe_sesion(f"{d}/pr", "s1", [("u0", "uno"), ("u1", "dos"), ("u2", "tres")])
            jsonl(f"{d}/atrib.jsonl", [
                {"sesion": "s1", "id": "u0", "atribucion": "nueva", "nota_ausente": True, "commit_kb": None},
                {"sesion": "s1", "id": "u1", "atribucion": "canon-kb", "nota_ausente": True, "commit_kb": None},
                {"sesion": "s1", "id": "u2", "sin_veredicto": True}])
            jsonl(f"{d}/c/atribuidor.jsonl", [
                {"id": "u0", "atribucion": "nueva", "nota": "SECRETA", "linea": "L", "texto": "uno"},
                {"id": "u1", "atribucion": "canon-kb", "nota": "SECRETA", "linea": "L", "texto": "dos"}]) if os.makedirs(f"{d}/c") is None else 0
            r = corre("muestra.py", "--etapa", "2", "--cache", f"{d}/c", "--proyectos", f"{d}/pr", "--atribuciones", f"{d}/atrib.jsonl")
            self.assertEqual(r.returncode, 0, r.stderr)
            bruto = open(f"{d}/c/etiquetar-etapa2.jsonl").read()
            items = [json.loads(l) for l in bruto.splitlines()]
            self.assertEqual(sorted((i["id"], i["texto"], i["previo"]) for i in items),
                             [("u0", "uno", "previo-0"), ("u1", "dos", "previo-1")])
            self.assertNotIn("SECRETA", bruto)
            self.assertNotIn("canon-kb", bruto)
            self.assertEqual(items[0]["contexto_kb"], {"nota": None, "core": {}})

    def test_etapa2_sin_cache_para_un_positivo_exit_1(self):
        with tempfile.TemporaryDirectory() as d:
            escribe_sesion(f"{d}/pr", "s1", [("u0", "uno")])
            jsonl(f"{d}/atrib.jsonl", [{"sesion": "s1", "id": "u0", "atribucion": "nueva", "nota_ausente": True, "commit_kb": None}])
            r = corre("muestra.py", "--etapa", "2", "--cache", f"{d}/c", "--proyectos", f"{d}/pr", "--atribuciones", f"{d}/atrib.jsonl")
            self.assertEqual(r.returncode, 1)


class TestValidarExtra(unittest.TestCase):
    def test_validar_funciona_desde_cwd_ajeno(self):
        with tempfile.TemporaryDirectory() as d:
            jsonl(f"{d}/d", [{"sesion": "s", "id": "a", "correccion": "si", "tipo": "redirige"}])
            jsonl(f"{d}/e", [{"id": "a", "etiqueta": "si"}])
            r = subprocess.run([sys.executable, os.path.join(H, "validar.py"), "--etapa", "1", "--etiquetas", f"{d}/e",
                                "--detecciones", f"{d}/d", "--salida", f"{d}/o"], capture_output=True, text=True, cwd=d)
            self.assertEqual(r.returncode, 0, r.stderr)

    def test_id_sin_deteccion_exit_1(self):
        with tempfile.TemporaryDirectory() as d:
            jsonl(f"{d}/d", [])
            jsonl(f"{d}/e", [{"id": "a", "etiqueta": "si"}])
            r = corre("validar.py", "--etapa", "1", "--etiquetas", f"{d}/e", "--detecciones", f"{d}/d", "--salida", f"{d}/o")
            self.assertEqual(r.returncode, 1)
            self.assertIn("a", r.stderr)

    def test_etapa2_id_sin_atribucion_exit_1(self):
        with tempfile.TemporaryDirectory() as d:
            jsonl(f"{d}/j", [{"sesion": "s", "id": "z", "sin_veredicto": True}])
            jsonl(f"{d}/e", [{"id": "z", "atribucion_dueño": "nueva"}])
            r = corre("validar.py", "--etapa", "2", "--etiquetas", f"{d}/e", "--atribuciones", f"{d}/j", "--salida", f"{d}/o")
            self.assertEqual(r.returncode, 1)
            self.assertIn("sin atribución del juez: z", r.stderr)  # mensaje, no traceback

    def test_validar_etapa2(self):
        with tempfile.TemporaryDirectory() as d:
            juez_ = [("a", "canon-kb"), ("b", "canon-kb"), ("c", "nueva"), ("d", "regla-canal")]
            jsonl(f"{d}/j", [{"sesion": "s", "id": i, "atribucion": k, "nota_ausente": False, "commit_kb": "x"} for i, k in juez_])
            dueño = ["canon-kb", "perfil-core", "canon-kb", None]
            jsonl(f"{d}/e", [{"id": i, "atribucion_dueño": v} for (i, _), v in zip(juez_, dueño)])
            args = ("--etapa", "2", "--etiquetas", f"{d}/e", "--atribuciones", f"{d}/j", "--salida", f"{d}/o")
            r = corre("validar.py", *args)
            self.assertEqual(r.returncode, 2)
            self.assertIn("d", r.stderr.split(":")[-1])
            self.assertFalse(os.path.exists(f"{d}/o"))
            dueño[3] = "regla-canal"
            jsonl(f"{d}/e", [{"id": i, "atribucion_dueño": v} for (i, _), v in zip(juez_, dueño)])
            r = corre("validar.py", *args)
            self.assertEqual(r.returncode, 0, r.stderr)
            out = json.load(open(f"{d}/o"))
            self.assertEqual(out["n"], 4)
            self.assertEqual(out["acuerdo"], 0.5)            # a y d
            self.assertEqual(out["acuerdo_decidida"], 0.75)  # b coincide en decidida aunque no en valor; c no
            self.assertEqual(out["por_atribucion"]["canon-kb"], {"n": 2, "coinciden": 1})
            self.assertEqual(out["por_atribucion"]["nueva"], {"n": 1, "coinciden": 0})


if __name__ == "__main__":
    unittest.main()
