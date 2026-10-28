"""Tests de kill3.py: prefiltro, lote ciego y razón de tasas con transcripts sintéticos."""
import contextlib, io, json, os, shutil, sys, tempfile, unittest

H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, H)
import kill3, transcripts
from sellar import sha_sesiones

# Equivalentes sintéticos de las 8 delegaciones conocidas (los textos reales no están en el repo).
OCHO = [
    "lanza un consultor antes de decidir",
    "deberías haber delegado esto en un subagente",
    "¿por qué no despachas un executor?",
    "usa un agente en paralelo, no lo hagas tú",
    "Lánzalo con un Ejecutor",
    "haz un fork y que otro lo revise",
    "esto es trabajo para delegación, no para ti",
    "PARALELIZA las dos tareas",
]


def lineas(sid, textos, ts, cwd="/w/x"):
    base = {"timestamp": ts, "cwd": cwd, "entrypoint": "cli"}
    return [{**base, "type": "user", "uuid": f"{sid}-m{n}", "message": {"role": "user", "content": t}}
            for n, t in enumerate(textos)]


def escribe(proy, repo, sid, textos, ts, cwd="/w/x"):
    d = os.path.join(proy, repo)
    os.makedirs(d, exist_ok=True)
    with open(os.path.join(d, sid + ".jsonl"), "w") as f:
        for l in lineas(sid, textos, ts, cwd):
            f.write(json.dumps(l) + "\n")


class TestPrefiltro(unittest.TestCase):
    def test_caza_los_ocho(self):
        self.assertEqual([t for t in OCHO if not kill3.prefiltro(t)], [])

    def test_no_caza_ruido(self):
        for t in ("arregla el test", "sube el commit", "sí, adelante"):
            self.assertFalse(kill3.prefiltro(t), t)


class TestLote(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, self.tmp, True)
        self.proy = os.path.join(self.tmp, "proy")
        self.cache = os.path.join(self.tmp, "cache")
        for i in range(3):
            escribe(self.proy, "r1", f"b{i}", ["hola", "delega eso", f"otro {i}"], "2026-10-10T10:00:00Z", "/w/a")
        for i in range(4):
            escribe(self.proy, "r2", f"v{i}", ["hola", "lanza un agente", "a", "b", "c"], "2026-10-29T10:00:00Z", "/w/b")
        escribe(self.proy, "r2", "tarde", ["lanza eso"], "2026-11-05T10:00:00Z", "/w/b")
        base, _ = transcripts.poblacion(self.proy, "2026-10-06", "2026-10-26T22:35:13Z")
        self.sello = os.path.join(self.tmp, "sello.json")
        with open(self.sello, "w") as f:
            json.dump({"desde": "2026-10-06", "hasta": "2026-10-26T22:35:13Z",
                       "sesiones_sha256": sha_sesiones(base), "n_sesiones": len(base)}, f)

    def lanza(self, *extra, hasta="2026-11-01T00:00:00Z"):
        err = io.StringIO()
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(err):
            rc = kill3.main(["lote", "--proyectos", self.proy, "--sello", self.sello, "--hasta", hasta,
                             "--cache", self.cache, *extra])
        return rc, err.getvalue()

    def items(self):
        with open(os.path.join(self.cache, kill3.ITEMS)) as f:
            return [json.loads(l) for l in f]

    def clave(self):
        with open(os.path.join(self.cache, kill3.CLAVE)) as f:
            return json.load(f)

    def test_composicion_y_hasta_propio(self):
        self.assertEqual(self.lanza()[0], 0)
        c = self.clave()
        self.assertEqual(c["sesiones"], {"base": 3, "ventana": 4})  # "tarde" empieza después de --hasta
        hits = [v for v in c["items"].values() if not v["nohit"]]
        self.assertEqual(sum(v["periodo"] == "base" for v in hits), 3)
        self.assertEqual(sum(v["periodo"] == "ventana" for v in hits), 4)
        self.assertEqual(sum(v["nohit"] for v in c["items"].values()), 16)  # 16 no-hits en la ventana, menos de 40: se toman todos

    def test_ciego_al_periodo(self):
        self.lanza()
        with open(os.path.join(self.cache, kill3.ITEMS)) as f:
            crudo = f.read()
        for it in self.items():
            self.assertEqual(set(it), {"id", "previo", "texto", "etiqueta"})
            self.assertIsNone(it["etiqueta"])
        for fuga in ("2026", "/w/", "base", "ventana", "r1", "r2"):
            self.assertNotIn(fuga, crudo.replace("hola", ""), fuga)

    def test_barajado_determinista_y_mezclado(self):
        self.lanza()
        a = [i["id"] for i in self.items()]
        self.lanza("--forzar")
        self.assertEqual(a, [i["id"] for i in self.items()])
        self.lanza("--forzar", "--semilla", "7")
        self.assertNotEqual(a, [i["id"] for i in self.items()])
        per = [self.clave()["items"][i]["periodo"] for i in a]
        self.assertNotEqual(per, sorted(per))

    def test_se_niega_a_rehacer_sin_forzar(self):
        self.lanza()
        rc, err = self.lanza()
        self.assertEqual(rc, 1)
        self.assertIn("--forzar", err)

    def test_sello_que_no_casa(self):
        escribe(self.proy, "r1", "b9", ["x"], "2026-10-12T10:00:00Z")
        self.assertEqual(self.lanza()[0], 2)

    def test_ventana_solapada(self):
        self.assertEqual(self.lanza("--ventana-desde", "2026-10-20T00:00:00Z")[0], 1)


def clave_sintetica(base_si_sesiones=10, ventana=30, nohits=40):
    items = {f"b{i}": {"periodo": "base", "sesion": "s", "repo": "aa", "nohit": False} for i in range(20)}
    items.update({f"v{i}": {"periodo": "ventana", "sesion": "s", "repo": "bb", "nohit": False} for i in range(30)})
    items.update({f"n{i}": {"periodo": "ventana", "sesion": "s", "repo": "bb", "nohit": True} for i in range(nohits)})
    return {"items": items, "sesiones": {"base": base_si_sesiones, "ventana": ventana},
            "mensajes": {"base": 100, "ventana": 200}, "n_nohits_pedidos": 40,
            "sesiones_por_repo": {"base": {"aa": base_si_sesiones}, "ventana": {"bb": ventana}}}


def etiquetas(clave, si_base, si_ventana, si_nohits=0):
    e = {i: "no" for i in clave["items"]}
    for pref, n in (("b", si_base), ("v", si_ventana), ("n", si_nohits)):
        for i in range(n):
            e[f"{pref}{i}"] = "si"
    return e


class TestCalcula(unittest.TestCase):
    def test_kill_en_el_umbral(self):
        c = clave_sintetica()
        r = kill3.calcula(c, etiquetas(c, 10, 8 * 3))  # tasa base 1,0; ventana 24/30 = 0,8 -> RR 0,8
        self.assertEqual(r["tasa_por_sesion"], {"base": 1.0, "ventana": 0.8})
        self.assertEqual(r["veredicto"], "kill")

    def test_no_kill_por_debajo(self):
        c = clave_sintetica()
        r = kill3.calcula(c, etiquetas(c, 10, 23))  # 23/30 / 1,0 = 0,767
        self.assertEqual(r["veredicto"], "no_kill")
        self.assertEqual(r["motivos_incompleto"], [])

    def test_desgloses(self):
        c = clave_sintetica()
        r = kill3.calcula(c, etiquetas(c, 10, 3))
        self.assertEqual(r["por_repo"]["aa"], {"sesiones": {"base": 10, "ventana": 0}, "si": {"base": 10, "ventana": 0}})
        self.assertEqual(r["por_repo"]["bb"]["si"], {"base": 0, "ventana": 3})
        self.assertEqual(r["por_mensaje_humano"]["tasa"], {"base": 0.1, "ventana": 3 / 200})

    def test_tripwire_marca_incompleto_aunque_rr_sea_bajo(self):
        c = clave_sintetica()
        r = kill3.calcula(c, etiquetas(c, 10, 0, si_nohits=2))
        self.assertEqual(r["veredicto"], "incompleto")
        self.assertTrue(r["tripwire"]["salta"])

    def test_una_delegacion_en_nohits_no_salta(self):
        c = clave_sintetica()
        r = kill3.calcula(c, etiquetas(c, 10, 0, si_nohits=1))
        self.assertFalse(r["tripwire"]["salta"])
        self.assertEqual(r["veredicto"], "no_kill")

    def test_faltan_datos_nunca_es_no_kill(self):
        casos = {
            "etiquetas sin rellenar": lambda c, e: e.pop("b0"),
            "ventana abierta": lambda c, e: c["sesiones"].update(ventana=29),
            "base sin tasa": lambda c, e: [e.__setitem__(i, "no") for i in c["items"] if i.startswith("b")],
            "pocos no-hits": lambda c, e: [(c["items"].pop(f"n{i}"), e.pop(f"n{i}")) for i in range(5)],
            "base sin sesiones": lambda c, e: c["sesiones"].update(base=0),
        }
        for nombre, muta in casos.items():
            c = clave_sintetica()
            e = etiquetas(c, 10, 0)
            muta(c, e)
            self.assertEqual(kill3.calcula(c, e)["veredicto"], "incompleto", nombre)

    def test_etiquetas_invalidas(self):
        c = clave_sintetica()
        with self.assertRaises(ValueError):
            kill3.calcula(c, {"zz": "si"})
        with self.assertRaises(ValueError):
            kill3.calcula(c, {"b0": "quizá"})


class TestCli(unittest.TestCase):
    def test_exit_3_si_incompleto_y_0_si_decide(self):
        tmp = tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, tmp, True)
        c = clave_sintetica()
        with open(os.path.join(tmp, "clave.json"), "w") as f:
            json.dump(c, f)
        for sufijo, e, esperado in (("a", etiquetas(c, 10, 0), 0), ("b", {**etiquetas(c, 10, 0), "b0": None}, 3)):
            with open(os.path.join(tmp, f"e{sufijo}.jsonl"), "w") as f:
                for i, v in e.items():
                    f.write(json.dumps({"id": i, "etiqueta": v}) + "\n")
            with contextlib.redirect_stdout(io.StringIO()):
                rc = kill3.main(["resultado", "--clave", os.path.join(tmp, "clave.json"),
                                 "--etiquetas", os.path.join(tmp, f"e{sufijo}.jsonl"), "--salida", os.path.join(tmp, f"r{sufijo}.json")])
            self.assertEqual(rc, esperado)


if __name__ == "__main__":
    unittest.main()
