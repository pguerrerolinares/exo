import glob
import importlib.util
import json
import os
import re
import subprocess
import sys
import unittest

AQUI = os.path.dirname(os.path.abspath(__file__))
RAIZ = os.path.abspath(os.path.join(AQUI, "..", ".."))
DETECTOR = os.path.join(RAIZ, "plugins", "exo", "scripts", "delegar-decision.py")
sys.path.insert(0, AQUI)
import transcripts  # noqa: E402

spec = importlib.util.spec_from_file_location("delegar_decision", DETECTOR)
D = importlib.util.module_from_spec(spec)
spec.loader.exec_module(D)

DATOS = os.environ.get("EXO_DELEGAR_DATOS") or os.path.expanduser("~/.cache/exo-delegar")
PROYECTOS = os.path.expanduser("~/.claude/projects")


def _json(p):
    with open(p, encoding="utf-8") as f:
        return json.load(f)


def texto(t):
    return {"type": "text", "text": t}


def tool(name, **inp):
    return {"type": "tool_use", "name": name, "input": inp}


def ev(*bloques):
    return D.evalua(list(bloques))


class TestEvalua(unittest.TestCase):
    def test_p2_dispara(self):
        r = ev(texto("Propuesta lista. ¿Te parece?"))
        self.assertTrue(r["dispara"])
        self.assertEqual((r["patron"], r["kind"], r["excluido"]), ("P2", "disparo", None))
        self.assertTrue(r["tail"].endswith("¿Te parece?"))

    def test_perm_se_evalua_sobre_tail(self):
        largo = "¿Te parece?" + " relleno" * 120
        r = ev(texto(largo))
        self.assertGreater(len(largo), 700)
        self.assertFalse(r["dispara"])
        self.assertEqual(r["tail"], largo[-700:])
        self.assertIsNone(r["patron"])

    def test_p3_requiere_interrogacion(self):
        t = "Mi recomendación: X."
        r = ev(texto(t))
        self.assertFalse(r["dispara"])
        self.assertIsNone(r["patron"])
        r = ev(texto(t), tool("AskUserQuestion", questions=[]))
        self.assertTrue(r["dispara"])
        self.assertEqual(r["patron"], "P3")
        # '?' dentro de tail dispara; '?' fuera de los últimos 700 c no
        self.assertEqual(ev(texto(t + " ¿Vale?"))["patron"], "P3")
        lejos = "¿Vale? " + t + " x" * 400
        self.assertFalse(ev(texto(lejos))["dispara"])
        # PROP se busca en el texto entero, no en tail
        self.assertEqual(ev(texto(t + " x" * 400 + " ¿Qué opinas?"))["patron"], "P3")

    def test_ask_sin_prop_no_dispara_por_p3(self):
        r = ev(texto("Aquí tienes el resumen."), tool("AskUserQuestion", questions=[]))
        self.assertFalse(r["dispara"])
        self.assertEqual(r["kind"], "p1_log")

    def test_ask_es_contener_no_terminar(self):
        # R2: AskUserQuestion en cualquier posición del segmento
        r = ev(tool("AskUserQuestion", questions=[]), texto("Te propongo X."))
        self.assertEqual(r["patron"], "P3")

    def test_tool_use_no_agent_sigue_disparando(self):
        r = ev(tool("Read", file_path="x"), texto("¿Lo hago?"))
        self.assertTrue(r["dispara"])
        self.assertEqual(r["patron"], "P2")

    def test_agent_en_segmento_no_dispara(self):
        for nombre in ("Agent", "Task"):
            r = ev(texto("¿Te parece?"), tool(nombre, prompt="x"))
            self.assertFalse(r["dispara"], nombre)
            self.assertEqual(r["excluido"], "agent", nombre)

    def test_etiqueta_solo_al_inicio(self):
        r = ev(texto("[reserva: externo] ¿Lo subo?"))
        self.assertFalse(r["dispara"])
        self.assertEqual(r["excluido"], "etiqueta")
        self.assertTrue(ev(texto("¿Lo subo? [reserva: externo]"))["dispara"])
        r = ev(texto("¿Te parece? [reserva: externo]"))
        self.assertTrue(r["dispara"])
        self.assertIsNone(r["excluido"])
        r = ev(texto("¿Te parece? [reserva: otra]"))
        self.assertTrue(r["dispara"])
        r = ev(texto("[reserva: otra] ¿Te parece?"))
        self.assertTrue(r["dispara"])
        for v in ("externo", "irreversible", "re-scope", "hecho-de-Paul", "pedido"):
            for pre in ("", "  ", "\n\t "):
                r = ev(texto(f"{pre}[reserva: {v}] ¿Te parece?"))
                self.assertEqual(r["excluido"], "etiqueta", (v, pre))
                self.assertFalse(r["dispara"])

    def test_etiqueta_sobre_ultimo_texto_no_sobre_tail(self):
        largo = "[reserva: externo] " + "x " * 600 + "¿Te parece?"
        self.assertEqual(ev(texto(largo))["excluido"], "etiqueta")

    def test_exclusion_con_limite_de_palabra(self):
        r = ev(texto("¿hago push y abro el PR?"))
        self.assertEqual(r["excluido"], "reserva")
        self.assertFalse(r["dispara"])
        self.assertTrue(ev(texto("¿Te parece el subordinado?"))["dispara"])
        self.assertTrue(ev(texto("¿Te vale el commitment?"))["dispara"])
        self.assertTrue(ev(texto("¿Te parece aprobar el cambio?"))["dispara"])
        self.assertTrue(ev(texto("¿Te parece el autopush del bot?"))["dispara"])
        self.assertTrue(ev(texto("¿Te parece, ya que el pronóstico es bueno?"))["dispara"])
        self.assertTrue(ev(texto("¿Te parece el tipo de pr_ocesador?"))["dispara"])

    def test_exclusion_un_caso_por_termino(self):
        casos = ["¿Hago push?", "¿Abro el PR?", "¿Abro un issue?", "¿Hago merge?", "¿Hago el deploy?",
                 "¿Hago commit?", "¿Hago /document?", "¿Hago borrar la rama?",
                 "¿Hago PUSH?", "¿Vamos con el MERGE?", "¿Abro el Pr?"]
        for c in casos:
            r = ev(texto(c))
            self.assertEqual(r["excluido"], "reserva", c)
            self.assertFalse(r["dispara"], c)

    def test_exclusion_mira_la_ultima_pregunta_no_el_tail(self):
        r = ev(texto("Queda listo para push y PR. Antes de nada: ¿Te parece este diseño?"))
        self.assertTrue(r["dispara"])
        self.assertEqual(ev(texto("Primero ¿Hago push? y luego ¿Te parece?"))["dispara"], True)
        self.assertFalse(ev(texto("¿Te parece? Y además ¿hago push?"))["dispara"])

    def test_hago_push_no_dispara(self):
        self.assertEqual(ev(texto("¿Hago push?"))["excluido"], "reserva")

    def test_sin_interrogante_mira_la_ultima_linea(self):
        r = ev(texto("Mi recomendación: X. Hago push luego.\nLo dejo a tu criterio, y aviso por ask"), tool("AskUserQuestion"))
        self.assertTrue(r["dispara"])
        r = ev(texto("Mi recomendación: X.\nHago push ahora."), tool("AskUserQuestion"))
        self.assertEqual(r["excluido"], "reserva")

    def test_sin_inflexion(self):
        for c in ("¿Hago pushear la rama?", "¿Hago el mergeo?", "¿Te parece el deployment?", "¿Hago commitear?"):
            self.assertTrue(ev(texto(c))["dispara"], c)

    def test_exclusion_con_acentos_multibyte(self):
        # `¿` y las vocales acentuadas pasan por (?i) y \b sin romper la palabra reservada
        self.assertEqual(ev(texto("¿Te parece hacer el deploy ya?"))["excluido"], "reserva")
        self.assertTrue(ev(texto("¿Te parece así, con ñandú y ópera?"))["dispara"])
        self.assertTrue(ev(texto("¿LO HAGO ASÍ?"))["dispara"])

    def test_kinds_informativos(self):
        r = ev(texto("¿Seguro?"))
        self.assertFalse(r["dispara"])
        self.assertEqual((r["kind"], r["patron"]), ("p1_log", None))
        r = ev(texto("Listo, queda hecho."))
        self.assertFalse(r["dispara"])
        self.assertEqual(r["kind"], "banda")
        r = ev(texto("Listo."), tool("Read", file_path="x"))
        self.assertFalse(r["dispara"])
        self.assertIsNone(r["kind"])
        r = ev(tool("Read", file_path="x"))
        self.assertIsNone(r["kind"])
        self.assertIsNone(ev()["kind"])
        self.assertEqual(ev(texto("¿Te parece?"))["kind"], "disparo")

    def test_excluidos_no_son_log_ni_disparo(self):
        r = ev(texto("[reserva: pedido] ¿Te parece?"))
        self.assertIsNone(r["kind"])
        r = ev(texto("¿Hago push?"))
        self.assertIsNone(r["kind"])

    def test_el_detector_no_importa_transcripts(self):
        src = open(DETECTOR, encoding="utf-8").read()
        self.assertIsNone(re.search(r"(?m)^\s*(import|from)\s+transcripts", src))

    def test_mismo_input_bloquea_sin_abstencion(self):
        b = [texto("¿Te parece?")]
        self.assertEqual(D.evalua(b), D.evalua(list(b)))
        self.assertTrue(D.evalua(b)["dispara"])


def humano(uuid, t="hola", **extra):
    d = {"type": "user", "uuid": uuid, "message": {"role": "user", "content": t}}
    d.update(extra)
    return d


def asist(uuid, *bloques, **extra):
    d = {"type": "assistant", "uuid": uuid, "message": {"role": "assistant", "content": list(bloques)}}
    d.update(extra)
    return d


LINEAS = [
    humano("u1"),
    asist("a1", texto("uno")),
    humano("h1", "resultado del subagente", isMeta=True, origin={"kind": "peer"}),
    asist("a2", texto("dos"), tool("Read", file_path="x")),
    {"type": "user", "uuid": "r1", "toolUseResult": {}, "message": {"content": [
        {"type": "tool_result", "tool_use_id": "t", "content": "ok"}]}},
    {"type": "user", "uuid": "r2", "message": {"content": [
        {"type": "tool_result", "tool_use_id": "t", "content": "ok"}]}},
    asist("a3", texto("tres")),
    humano("n1", "<task-notification>fin</task-notification>", promptSource="system", isMeta=True),
    asist("a4", texto("cuatro")),
    asist("s1", texto("lateral"), isSidechain=True),
    humano("s2", "lateral humano", isSidechain=True),
    humano("u2", "otra cosa", origin={"kind": "human"}, promptSource="typed"),
    asist("a5", texto("final"), timestamp="2026-10-10T10:00:00Z"),
]


class TestSegmentacion(unittest.TestCase):
    def test_segmentacion_paridad(self):
        casos = LINEAS + [
            humano("c1", "<command-name>x"), humano("c2", "resumen", isCompactSummary=True),
            humano("c3", "x", promptSource="queued"), humano("c4", "x", promptSource="other"),
            humano("c5", "   "), humano("c6", [{"type": "text", "text": "hola"}]),
            humano("c7", "x", origin={"kind": "human"}), humano("c8", "x", origin=None),
            {"type": "assistant", "message": {"content": []}}, {"type": "system"},
        ]
        for d in casos:
            self.assertEqual(D.es_mensaje_humano(d), transcripts.es_mensaje_humano(d), d.get("uuid"))

    def test_claves_y_cortes(self):
        segs = D.segmentar(LINEAS)
        self.assertEqual([k for k, _ in segs], ["u1", "NOHUMANO", "NOHUMANO", "u2", None])
        textos = [[b.get("text") for b in bl if b.get("type") == "text"] for _, bl in segs]
        self.assertEqual(textos, [[], ["uno"], ["dos", "tres"], ["cuatro"], ["final"]])

    def test_tope_corta_por_epoch(self):
        lineas = [humano("u1"), asist("a1", texto("antes"), timestamp="2026-10-10T09:00:00Z"),
                  asist("a2", texto("despues"), timestamp="2026-10-10T11:00:00+00:00")]
        corte = transcripts._epoch("2026-10-10T10:00:00Z")
        self.assertEqual(D._epoch("2026-10-10T10:00:00Z"), corte)
        self.assertEqual(D._epoch("2026-10-10T12:00:00+02:00"), corte)
        self.assertEqual(D._epoch("2026-10-10T10:00:00"), corte)
        segs = D.segmentar(lineas, corte)
        self.assertEqual([b.get("text") for b in segs[-1][1]], ["antes"])
        self.assertEqual([b.get("text") for b in D.segmentar(lineas)[-1][1]], ["antes", "despues"])


@unittest.skipUnless(os.path.isdir(DATOS) and os.path.isdir(PROYECTOS),
                     "PUERTA NO EJECUTADA: falta $EXO_DELEGAR_DATOS o ~/.claude/projects")
class TestPuertaDatosReales(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        r = subprocess.run([sys.executable, os.path.join(AQUI, "backtest_delegar.py"),
                            "--datos", DATOS, "--proyectos", PROYECTOS], capture_output=True, text=True)
        cls.rc = r.returncode
        cls.out = json.loads(r.stdout)

    def test_puerta_datos_reales(self):
        o = self.out
        self.assertEqual(o["puerta"], "ok", o)
        self.assertEqual(self.rc, 0)
        self.assertEqual(o["sesiones"], 99)
        self.assertTrue(236 <= o["bloqueos"] <= 288, o["bloqueos"])
        self.assertEqual(o["muestra"]["positivos_retenidos"], 17)
        self.assertTrue(20 <= o["muestra"]["retenidos"] <= 24)
        self.assertGreaterEqual(o["ocho"]["disparan"], 7)

    def test_paridad_con_patrones(self):
        ref = os.path.join(DATOS, "referencia")
        if not os.path.isfile(os.path.join(ref, "backtest.py")):
            self.skipTest("falta referencia/backtest.py")
        import backtest_delegar as B
        g = {"__file__": os.path.join(ref, "backtest.py")}
        exec(open(g["__file__"]).read().split("paths = {}")[0], g)
        ps = open(os.path.join(ref, "patrones.py")).read()
        cuerpo = ps[ps.index("PERM="):ps.index("pats={")]
        h = {"g": g, "re": re}
        exec(cuerpo.replace('ASKOPC=g["evalua"]', ""), h)
        info = h["info"]
        PERM, PROP = h["PERM"], h["PROP"]

        def p4(t):
            ag, last, tail, ask = info(t)
            return not ag and (bool(PERM.search(tail)) or (bool(PROP.search(last)) and ("?" in tail or ask)))

        muestra = _json(os.path.join(DATOS, "muestra.json"))["muestra"]
        ocho = list(_json(os.path.join(DATOS, "ocho.json")))
        ids = set(muestra) | set(ocho)
        self.assertEqual(len(muestra), 30)
        self.assertEqual(len(ocho), 8)
        propios = B.segmentos_por_id(PROYECTOS, os.path.join(AQUI, "sello.json"), ids)
        self.assertEqual(set(propios), ids)
        paths = {os.path.basename(p)[:-6]: p for p in glob.glob(os.path.join(g["P"], "*", "*.jsonl"))}
        visto = {}
        for s in g["ses"]:
            for i, (sid, t) in enumerate(g["turnos"](paths[s["id"]])):
                did = sid if sid and sid != "NOHUMANO" else f"{s['id']}#{i}"
                if did in ids:
                    visto[did] = p4(t) if t else False
        self.assertEqual(set(visto), ids)
        for i in ids:
            self.assertEqual(propios[i]["patron"] is not None, visto[i], i)


if __name__ == "__main__":
    unittest.main()
