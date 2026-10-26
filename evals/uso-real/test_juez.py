"""Tests de juez.py y detectar.py con un `claude` falso y transcripts sintéticos inventados."""
import json, os, shutil, stat, subprocess, sys, tempfile, unittest

H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, H)
import detectar, juez, transcripts

FAKE = '''#!/usr/bin/env python3
import json, os, re, sys
if sys.argv[1:] == ["--version"]:
    print(os.environ.get("FAKE_VERSION", "9.9.9 (Fake)")); sys.exit(0)
d = os.environ["FAKE_DIR"]
prompt = sys.argv[-1]
msgs = [json.loads(l[4:]) for l in prompt.splitlines() if l.startswith("MSG ")]
with open(os.path.join(d, "calls.jsonl"), "a") as f:
    f.write(json.dumps({"argv": sys.argv[1:], "cwd": os.getcwd(), "ids": [m["id"] for m in msgs], "prompt": prompt}) + "\\n")
if os.environ.get("FAKE_RC"):
    print("boom", file=sys.stderr); sys.exit(int(os.environ["FAKE_RC"]))
omit = os.environ.get("FAKE_OMIT")
out = []
for m in msgs:
    if m["id"] == omit:
        continue
    si = "corrige" in m["texto"]
    out.append({"id": m["id"], "correccion": "si" if si else "no", "tipo": "redirige" if si else None})
print(json.dumps({"type": "result", "result": json.dumps(out), "total_cost_usd": float(os.environ.get("FAKE_COSTE", "0.01"))}))
'''

SECRETO = "TEXTO-SECRETO-DEL-MENSAJE"


def lineas_sesion(msgs, ts0="2026-10-10T10:00:00Z"):
    """msgs: [(uuid, texto, ts)]; cada mensaje va precedido de un turno del asistente."""
    out = []
    for n, (u, t, ts) in enumerate(msgs):
        base = {"timestamp": ts, "cwd": "/w/x", "entrypoint": "cli"}
        out.append({**base, "type": "assistant", "uuid": f"a{n}",
                    "message": {"role": "assistant", "content": [{"type": "text", "text": f"turno {n}"}]}})
        out.append({**base, "type": "user", "uuid": u, "message": {"role": "user", "content": t}})
    return out


class TestParseaLote(unittest.TestCase):
    def test_parsea_lote_falta_id(self):
        res = json.dumps([{"id": "a", "correccion": "no", "tipo": None},
                          {"id": "c", "correccion": "si", "tipo": "redirige"}])
        out = juez.parsea_lote(res, ["a", "b", "c"], detectar.valida)
        self.assertEqual(sorted(out), ["a", "c"])
        self.assertNotIn("b", out)

    def test_parsea_lote_id_ajeno(self):
        res = json.dumps([{"id": "a", "correccion": "no", "tipo": None},
                          {"id": "zzz", "correccion": "no", "tipo": None}])
        self.assertEqual(sorted(juez.parsea_lote(res, ["a", "b"], detectar.valida)), ["a"])

    def test_bloque_json_basura_y_none(self):
        ok = '```json\n[{"id": "a", "correccion": "no", "tipo": null}]\n```'
        self.assertEqual(sorted(juez.parsea_lote(ok, ["a"], detectar.valida)), ["a"])
        for malo in (None, "no es json", '{"id": "a"}', '[{"id": "a", "correccion": "si", "tipo": null}]'):
            self.assertEqual(juez.parsea_lote(malo, ["a"], detectar.valida), {}, malo)


class TestValida(unittest.TestCase):
    def test_esquema(self):
        self.assertEqual(detectar.valida({"id": "a", "correccion": "si", "tipo": "redirige", "x": 1}),
                         {"id": "a", "correccion": "si", "tipo": "redirige"})
        self.assertEqual(detectar.valida({"id": "a", "correccion": "no", "tipo": None})["tipo"], None)
        self.assertIsNone(detectar.valida({"id": "a", "correccion": "no", "tipo": "null"})["tipo"])
        self.assertIsNone(detectar.valida({"id": "a", "correccion": "si", "tipo": "null"}))
        for malo in ({"id": "a", "correccion": "si", "tipo": "otro"}, {"id": "a", "correccion": "no", "tipo": "redirige"},
                     {"id": "a", "correccion": "quizá", "tipo": None}, {"correccion": "no", "tipo": None}):
            self.assertIsNone(detectar.valida(malo), malo)


class TestCache(unittest.TestCase):
    def test_ultima_escritura_gana(self):
        with tempfile.TemporaryDirectory() as d:
            p = os.path.join(d, "sub", "c.jsonl")
            c = juez.Cache(p)
            self.assertIsNone(c.get("a"))
            c.put("a", {"id": "a", "v": 1}); c.put("a", {"id": "a", "v": 2})
            self.assertEqual(juez.Cache(p).get("a")["v"], 2)


class Base(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, self.tmp, True)
        self.fakedir = os.path.join(self.tmp, "fake"); os.makedirs(self.fakedir)
        self.bin = os.path.join(self.tmp, "claude")
        with open(self.bin, "w") as f:
            f.write(FAKE)
        os.chmod(self.bin, os.stat(self.bin).st_mode | stat.S_IXUSR)
        self.env = {**os.environ, "CLAUDE_BIN": self.bin, "FAKE_DIR": self.fakedir}
        self.proy = os.path.join(self.tmp, "proyectos")
        self.cache = os.path.join(self.tmp, "cache")
        self.salida = os.path.join(self.tmp, "out.jsonl")

    def escribe(self, sid, msgs, proyecto="p1"):
        d = os.path.join(self.proy, proyecto); os.makedirs(d, exist_ok=True)
        with open(os.path.join(d, sid + ".jsonl"), "w") as f:
            for l in lineas_sesion(msgs):
                f.write(json.dumps(l) + "\n")

    def mensajes(self, n, prefijo="m", texto="hola"):
        return [(f"{prefijo}{k:02d}", f"{texto} {k}", f"2026-10-10T10:{k:02d}:00Z") for k in range(n)]

    def corre(self, *extra, **env):
        return subprocess.run([sys.executable, os.path.join(H, "detectar.py"), "--proyectos", self.proy,
                               "--cache", self.cache, "--salida", self.salida, *extra],
                              capture_output=True, text=True, env={**self.env, **env})

    def calls(self):
        p = os.path.join(self.fakedir, "calls.jsonl")
        if not os.path.exists(p):
            return []
        return [json.loads(l) for l in open(p)]

    def salida_lineas(self, path=None):
        return [json.loads(l) for l in open(path or self.salida)]


class TestDetectar(Base):
    def test_lote_reintento_y_sin_veredicto(self):
        self.escribe("s1", self.mensajes(10))
        r = self.corre(FAKE_OMIT="m05")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(len(self.calls()), 2)
        self.assertEqual(self.calls()[1]["ids"], ["m05"])
        lineas = self.salida_lineas()
        self.assertEqual(len(lineas), 10)
        sin = [l for l in lineas if l.get("sin_veredicto")]
        self.assertEqual([l["id"] for l in sin], ["m05"])
        self.assertEqual(sum(1 for l in lineas if "correccion" in l), 9)

    def test_cache_no_repite(self):
        self.escribe("s1", self.mensajes(3))
        self.assertEqual(self.corre().returncode, 0)
        self.assertEqual(len(self.calls()), 1)
        primera = open(self.salida).read()
        os.remove(self.salida)
        self.assertEqual(self.corre().returncode, 0)
        self.assertEqual(len(self.calls()), 1)
        self.assertEqual(open(self.salida).read(), primera)
        self.escribe("s1", self.mensajes(3) + [("mNuevo", "uno nuevo", "2026-10-10T11:00:00Z")])
        os.remove(self.salida)
        self.assertEqual(self.corre().returncode, 0)
        self.assertEqual(len(self.calls()), 2)
        self.assertEqual(self.calls()[1]["ids"], ["mNuevo"])
        self.assertEqual(len(self.salida_lineas()), 4)

    def test_tope_exit_3(self):
        self.escribe("s1", self.mensajes(11))
        r = self.corre("--tope-usd", "3", FAKE_COSTE="2.0")
        self.assertEqual(r.returncode, 3, r.stderr)
        self.assertEqual(len(self.calls()), 2)
        self.assertIn("coste_total_usd=4", r.stdout)
        self.assertIn("n_sesiones=1", r.stdout)
        self.assertTrue(os.path.exists(self.salida + ".parcial"))
        self.assertFalse(os.path.exists(self.salida))
        self.assertEqual(len(self.salida_lineas(self.salida + ".parcial")), 11)

    def test_tope_borde_exacto(self):
        self.escribe("s1", self.mensajes(11))
        r = self.corre("--tope-usd", "3", FAKE_COSTE="3.0")
        self.assertEqual(r.returncode, 3, r.stderr)
        self.assertEqual(len(self.calls()), 1)

    def test_fallo_proceso_exit_4(self):
        self.escribe("s1", self.mensajes(2))
        r = self.corre(FAKE_RC="1")
        self.assertEqual(r.returncode, 4)
        self.assertIn("coste_total_usd=0", r.stdout)
        self.assertTrue(os.path.exists(self.salida + ".parcial"))

    def test_sello_no_casa_exit_2(self):
        self.escribe("s1", self.mensajes(2))
        sello = os.path.join(self.tmp, "sello.json")
        bueno = {"prompt_detector_sha256": juez.sha256_fichero(os.path.join(H, "detector.prompt.md")),
                 "claude_version": "9.9.9 (Fake)", "filtro_sha256": transcripts.filtro_sha256(),
                 "modelos": {"detector": detectar.MODELO, "atribuidor": "x"}, "lote": detectar.LOTE}
        for malo in ({**bueno, "prompt_detector_sha256": "0" * 64}, {**bueno, "claude_version": "1.0.0"},
                     {**bueno, "filtro_sha256": "0" * 64}, {**bueno, "lote": 5},
                     {**bueno, "modelos": {"detector": "otro"}}, {k: v for k, v in bueno.items() if k != "modelos"}):
            json.dump(malo, open(sello, "w"))
            r = self.corre("--sello", sello)
            self.assertEqual(r.returncode, 2, r.stderr)
            self.assertEqual(self.calls(), [])
        json.dump(bueno, open(sello, "w"))
        otro = os.path.join(self.tmp, "otro.prompt.md")
        shutil.copy(os.path.join(H, "detector.prompt.md"), otro)
        with open(otro, "a") as f:
            f.write("\nNota extra.\n")
        self.assertEqual(self.corre("--sello", sello, "--prompt", otro).returncode, 2)
        r = self.corre("--sello", sello)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(len(self.calls()), 1)

    def test_solo_sesion_incompatible_con_sello(self):
        self.escribe("s1", self.mensajes(2))
        sello = os.path.join(self.tmp, "sello.json"); json.dump({}, open(sello, "w"))
        self.assertEqual(self.corre("--sello", sello, "--solo-sesion", "s1").returncode, 1)
        self.assertEqual(self.calls(), [])

    def test_solo_sesion_restringe(self):
        self.escribe("s1", self.mensajes(2, "a")); self.escribe("s2", self.mensajes(2, "b"))
        r = self.corre("--solo-sesion", "s2")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual([l["id"] for l in self.salida_lineas()], ["b00", "b01"])
        self.assertIn("n_sesiones=1", r.stdout)

    def test_salida_publica_sin_texto(self):
        self.escribe("s1", [("m00", SECRETO + " corrige esto", "2026-10-10T10:00:00Z"),
                            ("m01", SECRETO + " otra cosa", "2026-10-10T10:01:00Z"),
                            ("m02", SECRETO + " tres", "2026-10-10T10:02:00Z")])
        r = self.corre(FAKE_OMIT="m02")
        self.assertEqual(r.returncode, 0, r.stderr)
        crudo = open(self.salida).read()
        self.assertNotIn(SECRETO, crudo)
        lineas = self.salida_lineas()
        self.assertEqual([l["id"] for l in lineas], ["m00", "m01", "m02"])
        self.assertEqual(set(lineas[0]), {"sesion", "id", "correccion", "tipo"})
        self.assertEqual((lineas[0]["correccion"], lineas[0]["tipo"]), ("si", "redirige"))
        self.assertEqual((lineas[1]["correccion"], lineas[1]["tipo"]), ("no", None))
        self.assertEqual(lineas[2], {"sesion": "s1", "id": "m02", "sin_veredicto": True})
        self.assertIn(SECRETO, open(os.path.join(self.cache, "detector.jsonl")).read())

    def test_orden_y_previo_vacio_explicito(self):
        self.escribe("sB", [("b1", "uno", "2026-10-10T10:00:00Z")])
        self.escribe("sA", [("a2", "dos", "2026-10-10T10:05:00Z"), ("a1", "uno", "2026-10-10T10:01:00Z")])
        self.assertEqual(self.corre().returncode, 0)
        c = self.calls()
        self.assertEqual(len(c), 1)
        self.assertEqual(c[0]["ids"], ["a1", "a2", "b1"])
        primero = [json.loads(l[4:]) for l in c[0]["prompt"].splitlines() if l.startswith("MSG ")][0]
        self.assertIn("previo", primero)
        self.assertEqual(primero["previo"], "turno 1")  # a1 va tras el turno de a2 en el fichero

    def test_primer_mensaje_previo_vacio(self):
        d = os.path.join(self.proy, "p1"); os.makedirs(d)
        base = {"timestamp": "2026-10-10T10:00:00Z", "cwd": "/w/x", "entrypoint": "cli"}
        with open(os.path.join(d, "s1.jsonl"), "w") as f:
            f.write(json.dumps({**base, "type": "user", "uuid": "u1", "message": {"role": "user", "content": "hola"}}) + "\n")
        self.assertEqual(self.corre().returncode, 0)
        msg = [json.loads(l[4:]) for l in self.calls()[0]["prompt"].splitlines() if l.startswith("MSG ")][0]
        self.assertEqual(msg, {"id": "u1", "previo": "", "texto": "hola"})

    def entradas(self):
        return [json.loads(l[4:]) for l in self.calls()[0]["prompt"].splitlines() if l.startswith("MSG ")]

    def test_truncado_texto_principio_previo_final(self):
        self.escribe("s1", [("m0", "a" * 5000 + "FIN", "2026-10-10T10:00:00Z")])
        p = os.path.join(self.proy, "p1", "s1.jsonl")
        lineas = [json.loads(l) for l in open(p)]
        lineas[0]["message"]["content"][0]["text"] = "x" * 1000 + "y" * 4000 + "ULTIMO"
        with open(p, "w") as f:
            for l in lineas:
                f.write(json.dumps(l) + "\n")
        self.assertEqual(self.corre().returncode, 0)
        m = self.entradas()[0]
        self.assertEqual(m["texto"], "a" * 2000)
        self.assertNotIn("FIN", m["texto"])
        self.assertTrue(m["previo"].endswith("ULTIMO"))
        self.assertEqual(len(m["previo"]), 4000)
        self.assertNotIn("x", m["previo"])

    def test_cache_se_invalida_con_otro_prompt(self):
        self.escribe("s1", self.mensajes(2))
        prompt = os.path.join(self.tmp, "detector.prompt.md")
        shutil.copy(os.path.join(H, "detector.prompt.md"), prompt)
        self.assertEqual(self.corre("--prompt", prompt).returncode, 0)
        self.assertEqual(len(self.calls()), 1)
        with open(prompt, "a") as f:
            f.write("\nNota extra.\n")
        self.assertEqual(self.corre("--prompt", prompt).returncode, 0)
        self.assertEqual(len(self.calls()), 2)

    def test_invocacion_aislada(self):
        self.escribe("s1", self.mensajes(1))
        self.assertEqual(self.corre().returncode, 0)
        a = self.calls()[0]["argv"]
        self.assertEqual(a[a.index("--model") + 1], "claude-haiku-5-5")
        self.assertEqual(a[a.index("--tools") + 1], "")
        self.assertEqual(a[a.index("--max-budget-usd") + 1], "0.10")
        self.assertIn("--no-session-persistence", a)


if __name__ == "__main__":
    unittest.main()
