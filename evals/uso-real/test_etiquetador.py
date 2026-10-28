"""Tests de etiquetador.py: incrustación de ítems en la plantilla y parseo de la hoja del kill 2."""
import contextlib, io, json, os, re, shutil, sys, tempfile, unittest

H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, H)
import etiquetador


def cfg(html):
    m = re.search(r"const CFG = (.*);\n", html)
    return json.loads(m.group(1).replace("<\\/", "</"))


class TestEtiquetador(unittest.TestCase):
    def setUp(self):
        self.d = tempfile.mkdtemp()

    def tearDown(self):
        shutil.rmtree(self.d)

    def p(self, nombre):
        return os.path.join(self.d, nombre)

    def lote(self, items):
        with open(self.p("lote.jsonl"), "w") as f:
            for it in items:
                f.write(json.dumps(it, ensure_ascii=False) + "\n")
        return self.p("lote.jsonl")

    def test_kill3_incrusta_el_lote_entero_sin_cerrar_el_script(self):
        items = [{"id": "a1", "previo": "cierre </script><script>alert(1)", "texto": "lanza un consultor", "etiqueta": None},
                 {"id": "b2", "previo": "", "texto": "vale", "etiqueta": "no"}]
        html = etiquetador.pagina("kill3", etiquetador.items_kill3(self.lote(items)))
        self.assertEqual(html.count("</script>"), 1)
        c = cfg(html)
        self.assertEqual(c["items"], items)
        self.assertEqual(c["ops"], [["si", "sí: pide delegar"], ["no", "no"]])
        self.assertEqual(c["exporta"], "completo")

    def test_la_clave_del_lote_cambia_con_los_ids_y_no_con_el_orden(self):
        a = [{"id": "a", "previo": "", "texto": "x"}, {"id": "b", "previo": "", "texto": "y"}]
        k = lambda its: cfg(etiquetador.pagina("kill3", its))["lote"]
        self.assertEqual(k(a), k(a[::-1]))
        self.assertNotEqual(k(a), k(a[:1]))

    def test_ids_repetidos_o_items_incompletos_fallan(self):
        with self.assertRaises(ValueError):
            etiquetador.pagina("kill3", [{"id": "a", "previo": "", "texto": ""}] * 2)
        with self.assertRaises(ValueError):
            etiquetador.items_kill3(self.lote([{"id": "a", "texto": "sin previo"}]))

    def test_kill2_parsea_la_hoja_contra_la_muestra(self):
        with open(self.p("hoja.md"), "w") as f:
            f.write("# Hoja\n\n## 1. exo\n\n- **Tú antes:** hazlo\n- **Cierre del padre:** ¿A o B?\n"
                    "- **Tu respuesta:** B\n\n<sub>id 100ce8b4</sub>\n")
        with open(self.p("muestra.json"), "w") as f:
            json.dump({"muestra": ["100ce8b4-a717"]}, f)
        items = etiquetador.items_kill2(self.p("hoja.md"), self.p("muestra.json"))
        self.assertEqual(items, [{"id": "100ce8b4-a717", "tema": "exo", "antes": "hazlo", "cierre": "¿A o B?", "resp": "B"}])
        with open(self.p("muestra.json"), "w") as f:
            json.dump({"muestra": ["ffff"]}, f)
        with self.assertRaises(ValueError):
            etiquetador.items_kill2(self.p("hoja.md"), self.p("muestra.json"))

    def test_cli_escribe_la_pagina_y_sale_con_1_si_falta_la_entrada(self):
        lote = self.lote([{"id": "a", "previo": "", "texto": "x", "etiqueta": "si"}])
        with contextlib.redirect_stdout(io.StringIO()) as out:
            self.assertEqual(etiquetador.main(["kill3", "--items", lote, "--salida", self.p("e.html")]), 0)
        self.assertIn("1 ítems (1 ya etiquetados)", out.getvalue())
        self.assertTrue(os.path.exists(self.p("e.html")))
        with contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(etiquetador.main(["kill3", "--items", self.p("no.jsonl"), "--salida", self.p("f.html")]), 1)


if __name__ == "__main__":
    unittest.main()
