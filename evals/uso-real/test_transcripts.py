import json
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import transcripts  # noqa: E402


def user(texto, **extra):
    d = {"type": "user", "uuid": "u", "message": {"role": "user", "content": texto}}
    d.update(extra)
    return d


def asist(texto, uuid="a"):
    return {"type": "assistant", "uuid": uuid,
            "message": {"role": "assistant", "content": [{"type": "text", "text": texto}]}}


def escribe(path, lineas, crudo=""):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        for d in lineas:
            f.write(json.dumps(d) + "\n")
        f.write(crudo)


def sesion(ts="2026-10-10T10:00:00Z", entry="cli", humano=True):
    base = {"timestamp": ts, "cwd": "/w/x", "entrypoint": entry}
    ls = []
    if humano:
        ls.append({**base, **user("hola", uuid="m1")})
    ls.append({**base, **asist("ok")})
    return ls


class TestEsMensajeHumano(unittest.TestCase):
    def test_filtra_meta_compact_toolresult(self):
        self.assertTrue(transcripts.es_mensaje_humano(user("un mensaje normal")))
        for extra in ({"isMeta": True}, {"isCompactSummary": True}, {"toolUseResult": {"x": 1}},
                      {"toolUseResult": ""}, {"toolUseResult": {}}):
            self.assertFalse(transcripts.es_mensaje_humano(user("texto", **extra)), extra)

    def test_prefijos_excluidos(self):
        prefijos = ["<command-name>", "<local-command-stdout>", "<task-notification>",
                    "<bash-stdout>", "Another Claude session", "[Request interrupted",
                    "This session is being continued"]
        for p in prefijos:
            self.assertFalse(transcripts.es_mensaje_humano(user("  \n" + p + " resto")), p)
        self.assertTrue(transcripts.es_mensaje_humano(user("no, usa git -C")))

    def test_suggestion_accepted_fuera(self):
        def u(**kw):
            return user("hazlo", **kw)
        h = {"kind": "human"}
        self.assertFalse(transcripts.es_mensaje_humano(u(origin=h, promptSource="suggestion_accepted")))
        self.assertTrue(transcripts.es_mensaje_humano(u(origin=h, promptSource="typed")))
        self.assertTrue(transcripts.es_mensaje_humano(u(origin=h, promptSource="queued")))
        self.assertFalse(transcripts.es_mensaje_humano(u(origin=h, promptSource="otro")))
        self.assertFalse(transcripts.es_mensaje_humano(u(origin={"kind": "task-notification"})))
        self.assertTrue(transcripts.es_mensaje_humano(u()))

    def test_paste_cuenta(self):
        self.assertTrue(transcripts.es_mensaje_humano(user("<pasted_content>log</pasted_content> mira")))
        self.assertFalse(transcripts.es_mensaje_humano(user("")))

    def test_texto_solo_bloques_text(self):
        d = user([{"type": "tool_result", "content": "NO"}, {"type": "text", "text": "a"},
                  {"type": "text", "text": "b"}])
        self.assertEqual(transcripts.texto(d), "a b")


class TestLeeSesion(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def p(self, *partes):
        return os.path.join(self.tmp.name, *partes)

    def test_previo_y_truncado(self):
        largo = "".join(chr(65 + (i // 1000)) for i in range(7000))  # AAAA..GGGG
        escribe(self.p("r", "s1.jsonl"), [
            {**asist("viejo"), "timestamp": "2026-10-10T10:00:00Z", "cwd": "/w/x"},
            {**asist(largo, "a2"), "timestamp": "2026-10-10T10:00:01Z"},
            {**user("no", uuid="m1"), "timestamp": "2026-10-10T10:00:02Z"},
        ])
        s = transcripts.lee_sesion(self.p("r", "s1.jsonl"))
        m = s["mensajes"][0]
        self.assertEqual(m["id"], "m1")
        self.assertEqual(m["previo"], largo[-6000:])
        self.assertTrue(m["previo"].startswith("B"))
        self.assertEqual((s["id"], s["repo_dir"], s["cwd"], s["inicio"]),
                         ("s1", "r", "/w/x", "2026-10-10T10:00:00Z"))

    def test_reglas_canal(self):
        snap = {"type": "attachment", "attachment": {
            "type": "prompt_snapshot",
            "systemPrompt": ["intro\n## Reglas duras del proyecto\n- a\n## Otra\n- b"]}}
        escribe(self.p("r", "s2.jsonl"), [snap] + sesion())
        s = transcripts.lee_sesion(self.p("r", "s2.jsonl"))
        self.assertIn("## Reglas duras del proyecto", s["reglas_canal"])
        self.assertIn("- a", s["reglas_canal"])
        self.assertNotIn("## Otra", s["reglas_canal"])
        self.assertNotIn("- b", s["reglas_canal"])
        escribe(self.p("r", "s3.jsonl"), sesion())
        self.assertIsNone(transcripts.lee_sesion(self.p("r", "s3.jsonl"))["reglas_canal"])

    def test_reglas_canal_dos_bloques_y_primer_snapshot(self):
        def snap(txt):
            return {"type": "attachment", "attachment": {"type": "prompt_snapshot", "systemPrompt": [txt]}}
        dos = "## Reglas duras del dueño\n- d\n## Reglas duras del proyecto\n- p\n## Otra\n- o"
        escribe(self.p("r", "s5.jsonl"), [snap(dos), snap("## Reglas duras del proyecto\n- segundo")] + sesion())
        r = transcripts.lee_sesion(self.p("r", "s5.jsonl"))["reglas_canal"]
        self.assertIn("- d", r)
        self.assertIn("- p", r)
        self.assertNotIn("## Otra", r)
        self.assertNotIn("- segundo", r)

    def test_sin_json_valido_none(self):
        escribe(self.p("r", "s4.jsonl"), [], crudo="{roto\n")
        self.assertIsNone(transcripts.lee_sesion(self.p("r", "s4.jsonl")))


class TestPoblacion(unittest.TestCase):
    def test_poblacion_cortes(self):
        with tempfile.TemporaryDirectory() as tmp:
            escribe(os.path.join(tmp, "a", "sdk.jsonl"), sesion(entry="sdk-cli"))
            escribe(os.path.join(tmp, "a", "vacia.jsonl"), sesion(humano=False))
            reserva = os.path.join(tmp, "a", "reserva.jsonl")
            escribe(reserva, sesion(ts="2026-10-05T23:59:59Z"))
            os.utime(reserva, (1_800_000_000, 1_800_000_000))
            escribe(os.path.join(tmp, "a", "sid", "subagents", "sub.jsonl"), sesion())
            # línea truncada a mitad de fichero y otra al final: se saltan
            escribe(os.path.join(tmp, "b", "ok.jsonl"),
                    sesion()[:1] + [], crudo='{"type": "user", "mess\n')
            with open(os.path.join(tmp, "b", "ok.jsonl"), "a") as f:
                f.write(json.dumps({**user("segundo", uuid="m2"), "timestamp": "2026-10-10T10:01:00Z"})
                        + "\n" + '{"type": "us')
            ses, c = transcripts.poblacion(tmp)
            self.assertEqual([s["id"] for s in ses], ["ok"])
            self.assertEqual([m["id"] for m in ses[0]["mensajes"]], ["m1", "m2"])
            self.assertEqual(c, {"sdk": 1, "vacias": 1, "reserva": 1})


if __name__ == "__main__":
    unittest.main()
