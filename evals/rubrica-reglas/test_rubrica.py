"""Tests de rubrica.py y siembra-reglas.es_candidata con datos sintéticos inventados."""
import importlib.util, json, os, random, tempfile, unittest

H = os.path.dirname(os.path.abspath(__file__))
import sys; sys.path.insert(0, H)
import rubrica as rb

_s = importlib.util.spec_from_file_location("siembra_reglas", os.path.join(H, "..", "..", "scripts", "siembra-reglas.py"))
sr = importlib.util.module_from_spec(_s); _s.loader.exec_module(sr)


def et(choque, coste="medio"):
    return {"id": "x", "choque": choque, "coste": coste}


def item(id, choque=None, coste="medio", criba=None):
    return {"id": id, "etiqueta": et(choque, coste) if choque else None, "criba": criba}


def ids(items, vetadas=frozenset()):
    return [i["id"] for i in rb.orden(items, vetadas)]


def ambas(test, items, esperado, vetadas=frozenset()):
    test.assertEqual(ids(items, vetadas), esperado)
    test.assertEqual(ids(items[::-1], vetadas), esperado)


class TestValida(unittest.TestCase):
    def test_valida_rechaza_fuera_de_dominio(self):
        base = {"id": "g0-1", "choque": "P", "coste": "alto"}
        v = rb.valida_etiqueta(base)
        self.assertIsNotNone(v)
        self.assertEqual((v["choque"], v["coste"]), ("P", "alto"))
        self.assertIsNone(rb.valida_etiqueta({**base, "choque": "X"}))
        self.assertIsNone(rb.valida_etiqueta({**base, "coste": "altísimo"}))
        self.assertIsNone(rb.valida_etiqueta({k: v for k, v in base.items() if k != "id"}))
        self.assertIsNone(rb.valida_etiqueta(["g0-1", "P", "alto"]))
        self.assertIsNone(rb.valida_etiqueta(None))

    def test_valida_ruta_opcional(self):
        self.assertIsNotNone(rb.valida_etiqueta({"id": "g0-1", "choque": "E", "coste": "bajo"}))
        self.assertIsNotNone(rb.valida_etiqueta({"id": "g0-1", "choque": "E", "coste": "bajo", "ruta": None}))

    def test_valida_formato_publico(self):
        v = rb.valida_etiqueta({"id": "g0-1", "choque": "D", "coste": "medio", "cita_recortada": True})
        self.assertIsNotNone(v)
        self.assertEqual(v["id"], "g0-1")


class TestCriba(unittest.TestCase):
    def test_criba_sustituye_juicio(self):
        self.assertEqual(rb.choque_efectivo({"choque": "P"}, True), "E")
        self.assertEqual(rb.choque_efectivo({"choque": "E"}, False), "D")
        self.assertEqual(rb.choque_efectivo({"choque": "P"}, False), "P")
        self.assertEqual(rb.choque_efectivo(None, False), "D")


class TestOrden(unittest.TestCase):
    def test_veto_gana_a_todo(self):
        ambas(self, [item("g0-1", "E", "bajo"), item("g0-2", "P", "alto")], ["g0-1", "g0-2"], frozenset({"g0-1"}))

    def test_choque_pesa_mas_que_coste(self):
        ambas(self, [item("g0-1", "D", "alto"), item("g0-2", "P", "bajo")], ["g0-2", "g0-1"])

    def test_sin_etiqueta_al_final(self):
        ambas(self, [item("g0-1"), item("g0-2", "E", "bajo")], ["g0-2", "g0-1"])

    def test_criba_entra_en_el_orden(self):
        # P con criba cumple cuenta como E; D sin criba queda delante
        ambas(self, [item("g0-1", "P", "bajo", criba=True), item("g0-2", "D", "bajo")], ["g0-2", "g0-1"])
        # E con criba no_cumple cuenta como D; sale delante de una E sin criba
        ambas(self, [item("g0-1", "E", "bajo"), item("g0-2", "E", "bajo", criba=False)], ["g0-2", "g0-1"])

    def test_sin_etiqueta_con_criba(self):
        # ids elegidos para que el desempate por id fallaría: la sin etiqueta tiene el id menor
        sin_et = item("g0-1", criba=False)
        items = [sin_et, item("g0-2", "D", "bajo"), item("g0-3", "E", "alto"), item("g0-4")]
        ambas(self, items, ["g0-2", "g0-1", "g0-3", "g0-4"])
        ambas(self, [item("g0-1", criba=False), item("g0-2", "D", "alto")], ["g0-2", "g0-1"])
        ambas(self, [item("g0-1", criba=True), item("g0-2", "E", "bajo")], ["g0-2", "g0-1"])

    def test_empate_por_orden_de_id(self):
        items = [item("g0-10", "D"), item("g1-1", "D"), item("g0-9", "D")]
        self.assertEqual(ids(items), ["g0-9", "g0-10", "g1-1"])

    def test_orden_no_depende_de_la_entrada(self):
        items = [item("g0-1", "E", "bajo"), item("g0-2"), item("g0-3", "D", "alto"),
                 item("g0-4", "P", "bajo"), item("g0-5", "E", "alto"), item("g0-6", "D", "bajo")]
        esperado = ["g0-5", "g0-4", "g0-3", "g0-6", "g0-1", "g0-2"]
        # g0-5 vetada; luego P, D alto, D bajo, E, sin etiqueta
        rng = random.Random(7)
        for _ in range(5):
            b = items[:]; rng.shuffle(b)
            self.assertEqual(ids(b, frozenset({"g0-5"})), esperado)


NOTA = """# Nota

Intro de la nota.

## Reglas duras

- Regla uno.
- Regla dos.

## Contexto

Texto del contexto.
"""


class TestRecorte(unittest.TestCase):
    def test_recorta_seccion_reglas_duras(self):
        t, _ = rb.recorta_nota(NOTA, "inexistente")
        self.assertNotIn("Reglas duras", t)
        self.assertNotIn("Regla uno", t)
        self.assertNotIn("Regla dos", t)
        self.assertIn("## Contexto", t)
        self.assertIn("Texto del contexto.", t)
        self.assertIn("Intro de la nota.", t)

    def test_recorta_linea_con_cita(self):
        texto = "Primera línea.\nSiempre  regenera EL snapshot antes de commitear los cambios.\nTercera línea.\n"
        t, ok = rb.recorta_nota(texto, "siempre regenera el snapshot antes de commitear")
        self.assertTrue(ok)
        self.assertNotIn("snapshot", t)
        self.assertIn("Primera línea.", t)
        self.assertIn("Tercera línea.", t)
        par = "Antes.\n\nSiempre regenera el snapshot antes\nde commitear los cambios del repo.\nOtra línea del párrafo.\n\nDespués.\n"
        t, ok = rb.recorta_nota(par, "siempre regenera el snapshot antes de commitear los cambios")
        self.assertTrue(ok)
        self.assertNotIn("snapshot", t)
        self.assertNotIn("Otra línea", t)
        self.assertIn("Antes.", t)
        self.assertIn("Después.", t)

    def test_cita_compara_solo_los_primeros_60(self):
        cabeza = "no uses la libreria estandar para esto sino el wrapper del repo siempre"[:60]
        t, ok = rb.recorta_nota("Antes.\n" + cabeza + " y luego otra cosa\nDespués.\n", cabeza + " cola que no aparece en la nota")
        self.assertTrue(ok)
        self.assertNotIn("libreria", t)

    def test_cita_vacia_no_recorta(self):
        t, ok = rb.recorta_nota("Una línea.\n\nOtra línea.\n", "")
        self.assertFalse(ok)
        self.assertEqual(t, "Una línea.\n\nOtra línea.\n")

    def test_cita_no_encontrada(self):
        t, ok = rb.recorta_nota(NOTA, "esta frase no aparece en ninguna parte de la nota")
        self.assertFalse(ok)
        self.assertEqual(t, "# Nota\n\nIntro de la nota.\n\n## Contexto\n\nTexto del contexto.\n")
        sin = "# Nota\n\nSolo prosa.\n"
        t, ok = rb.recorta_nota(sin, "otra cosa distinta")
        self.assertFalse(ok)
        self.assertEqual(t, sin)


POOL = [
    {"id": "g0-1", "generica": False, "ambito": "codigo", "proyecto": "p1"},
    {"id": "g0-2", "generica": True, "ambito": "codigo", "proyecto": "p1"},
    {"id": "g0-3", "generica": False, "ambito": "otro", "proyecto": "p1"},
    {"id": "g0-4", "generica": False, "ambito": "git", "proyecto": "transversal"},
    {"id": "g0-5", "generica": False, "ambito": "dominio-proyecto", "proyecto": "p1"},
]


class TestCandidata(unittest.TestCase):
    def test_es_candidata(self):
        self.assertEqual([sr.es_candidata(r) for r in POOL], [True, False, False, False, True])
        with tempfile.TemporaryDirectory() as d:
            pool, criba, kb = (os.path.join(d, x) for x in ("pool", "criba", "kb"))
            for x in (pool, criba, kb): os.mkdir(x)
            with open(os.path.join(pool, "reglas-g0.jsonl"), "w") as f:
                for r in POOL:
                    f.write(json.dumps({**r, "nota": "n.md", "cita": "c", "regla": "r"}) + "\n")
            viejo = dict(sr.REPO)
            sr.REPO["p1"] = ("repo-x", "p1.md")
            try:
                out, _ = sr.propuesta(pool, criba, kb)
                self.assertEqual([r["id"] for r in out[0]["reglas"]], ["g0-1", "g0-5"])
                orig = sr.es_candidata
                sr.es_candidata = lambda r: False
                try:
                    self.assertEqual(sr.propuesta(pool, criba, kb)[0], [])
                finally:
                    sr.es_candidata = orig
            finally:
                sr.REPO.clear(); sr.REPO.update(viejo)


if __name__ == "__main__":
    unittest.main()
