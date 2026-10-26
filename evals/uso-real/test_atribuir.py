"""Tests de atribuir.py con un `claude` y un `exo` falsos, una KB git temporal y transcripts sintéticos."""
import json, os, shutil, stat, subprocess, sys, tempfile, unittest

H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, H)
import atribuir, juez

FAKE_CLAUDE = '''#!/usr/bin/env python3
import json, os, sys
if sys.argv[1:] == ["--version"]:
    print("9.9.9 (Fake)"); sys.exit(0)
d = os.environ["FAKE_DIR"]
prompt = sys.argv[-1]
msgs = [json.loads(l[4:]) for l in prompt.splitlines() if l.startswith("MSG ")]
with open(os.path.join(d, "calls.jsonl"), "a") as f:
    f.write(json.dumps({"argv": sys.argv[1:], "ids": [m["id"] for m in msgs], "prompt": prompt}) + "\\n")
if os.environ.get("FAKE_RC"):
    sys.exit(int(os.environ["FAKE_RC"]))
out = {"atribucion": os.environ.get("FAKE_ATRIB", "nueva"), "nota": os.environ.get("FAKE_NOTA_CITA") or None,
       "linea": os.environ.get("FAKE_LINEA") or None}
print(json.dumps({"type": "result", "result": json.dumps(out), "total_cost_usd": float(os.environ.get("FAKE_COSTE", "0.01"))}))
'''

FAKE_EXO = '''#!/usr/bin/env python3
import json, os, sys
a = sys.argv[1:]
with open(os.path.join(os.environ["FAKE_DIR"], "exo.jsonl"), "a") as f:
    f.write(json.dumps({"cwd": a[a.index("--cwd") + 1]}) + "\\n")
if not os.environ.get("FAKE_NOTA_HEAD"):
    sys.exit(1)
print(json.dumps({"data": {"note": os.environ["FAKE_NOTA_HEAD"]}}))
'''

SECRETO = "TEXTO-SECRETO-DEL-MENSAJE"
REGLAS = "## Reglas duras del proyecto\n- REGLA-DE-CANAL-UNICA"


def sesion_lineas(msgs, ts0, reglas=None):
    """msgs: [(uuid, texto, ts)]; cada mensaje va precedido de un turno del asistente."""
    base = {"timestamp": ts0, "cwd": "/w/x", "entrypoint": "cli"}
    out = []
    if reglas:
        out.append({**base, "type": "attachment",
                    "attachment": {"type": "prompt_snapshot", "systemPrompt": ["intro\n" + reglas]}})
    for n, (u, t, ts) in enumerate(msgs):
        b = {**base, "timestamp": ts}
        out.append({**b, "type": "assistant", "uuid": f"a{n}",
                    "message": {"role": "assistant", "content": [{"type": "text", "text": f"turno {n}"}]}})
        out.append({**b, "type": "user", "uuid": u, "message": {"role": "user", "content": t}})
    return out


def git(d, *args, fecha=None):
    env = dict(os.environ)
    if fecha:
        env["GIT_COMMITTER_DATE"] = env["GIT_AUTHOR_DATE"] = fecha
    subprocess.run(["git", "-C", d, "-c", "user.name=t", "-c", "user.email=t@t", *args],
                   check=True, capture_output=True, env=env)


class Base(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, self.tmp, True)
        self.fakedir = os.path.join(self.tmp, "fake"); os.makedirs(self.fakedir)
        self.claude = self.ejecutable("claude", FAKE_CLAUDE)
        self.exo = self.ejecutable("exo", FAKE_EXO)
        self.env = {**os.environ, "CLAUDE_BIN": self.claude, "EXO_BIN": self.exo, "FAKE_DIR": self.fakedir}
        self.proy = os.path.join(self.tmp, "proyectos")
        self.cache = os.path.join(self.tmp, "cache")
        self.salida = os.path.join(self.tmp, "out.jsonl")
        self.det = os.path.join(self.tmp, "det.jsonl")
        self.kb = os.path.join(self.tmp, "kb")

    def ejecutable(self, nombre, src):
        p = os.path.join(self.tmp, nombre)
        with open(p, "w") as f:
            f.write(src)
        os.chmod(p, os.stat(p).st_mode | stat.S_IXUSR)
        return p

    def sesion(self, sid, msgs, ts0, reglas=None):
        d = os.path.join(self.proy, "p1"); os.makedirs(d, exist_ok=True)
        with open(os.path.join(d, sid + ".jsonl"), "w") as f:
            for l in sesion_lineas(msgs, ts0, reglas):
                f.write(json.dumps(l) + "\n")

    def detecciones(self, filas):
        with open(self.det, "w") as f:
            for sesion, i, c in filas:
                o = {"sesion": sesion, "id": i}
                o.update({"sin_veredicto": True} if c is None else {"correccion": c, "tipo": "redirige" if c == "si" else None})
                f.write(json.dumps(o) + "\n")

    def crea_kb(self, versiones):
        """versiones: [(fecha, texto_nota)]; core/ fijo."""
        os.makedirs(self.kb)
        git(self.kb, "init", "-q")
        for fecha, texto in versiones:
            for ruta, t in (("notas/p.md", texto), ("core/reglas-duras.md", "REGLAS-GLOBALES-X"), ("core/perfil.md", "PERFIL-X")):
                os.makedirs(os.path.dirname(os.path.join(self.kb, ruta)), exist_ok=True)
                with open(os.path.join(self.kb, ruta), "w") as f:
                    f.write(t)
            git(self.kb, "add", "notas/p.md", "core/reglas-duras.md", "core/perfil.md", fecha=fecha)
            git(self.kb, "commit", "-q", "-m", "c", fecha=fecha)
        return subprocess.run(["git", "-C", self.kb, "log", "--format=%H"], capture_output=True, text=True).stdout.split()

    def corre(self, *extra, **env):
        return subprocess.run([sys.executable, os.path.join(H, "atribuir.py"), "--proyectos", self.proy,
                               "--detecciones", self.det, "--kb", self.kb, "--cache", self.cache,
                               "--salida", self.salida, *extra],
                              capture_output=True, text=True, env={**self.env, **env})

    def calls(self):
        p = os.path.join(self.fakedir, "calls.jsonl")
        return [json.loads(l) for l in open(p)] if os.path.exists(p) else []

    def lineas(self, path=None):
        return [json.loads(l) for l in open(path or self.salida)]


class TestValida(unittest.TestCase):
    def test_esquema(self):
        self.assertEqual(atribuir.valida({"atribucion": "canon-kb", "nota": "n.md", "linea": "x", "z": 1}),
                         {"atribucion": "canon-kb", "nota": "n.md", "linea": "x"})
        self.assertEqual(atribuir.valida({"atribucion": "nueva", "nota": None, "linea": "null"}),
                         {"atribucion": "nueva", "nota": None, "linea": None})
        self.assertEqual(atribuir.valida({"atribucion": "nueva"})["nota"], None)
        for malo in ({"atribucion": "otra", "nota": None, "linea": None}, {"atribucion": "nueva", "nota": 3, "linea": None},
                     {"nota": None, "linea": None}, None, []):
            self.assertIsNone(atribuir.valida(malo), malo)


class TestEntrada(unittest.TestCase):
    MSG = {"id": "m1", "texto": "texto del dueño", "previo": "lo que dijo el agente"}
    CTX = {"nota": "NOTA-X", "reglas_globales": "GLOBALES-X", "core": {"core/reglas-duras.md": "GLOBALES-X", "core/perfil.md": "PERFIL-X"},
           "nota_ausente": False}
    PROMPT = "instrucciones\n{CONTEXTO}\n"

    def test_reglas_canal_en_entrada(self):
        con = atribuir.construye_entrada(self.MSG, [], self.CTX, "REGLA-DE-CANAL-UNICA", self.PROMPT)
        self.assertIn("Reglas en el system prompt", con)
        self.assertIn("REGLA-DE-CANAL-UNICA", con)
        self.assertNotIn("ninguna", con)
        sin = atribuir.construye_entrada(self.MSG, [], self.CTX, None, self.PROMPT)
        self.assertIn("Reglas en el system prompt", sin)
        self.assertIn("ninguna", sin)
        self.assertNotIn("REGLA-DE-CANAL-UNICA", sin)

    def test_contexto_sin_duplicar_globales(self):
        e = atribuir.construye_entrada(self.MSG, [], self.CTX, None, self.PROMPT)
        self.assertEqual(e.count("GLOBALES-X"), 1)
        for t in ("NOTA-X", "PERFIL-X", "texto del dueño", "lo que dijo el agente"):
            self.assertIn(t, e)

    def test_anteriores_truncados_por_delante(self):
        ant = ["CABEZA" + "a" * 5000, "b" * 5000 + "COLA"]
        e = atribuir.construye_entrada(self.MSG, ant, self.CTX, None, self.PROMPT)
        self.assertIn("COLA", e)
        self.assertNotIn("CABEZA", e)
        self.assertLessEqual(e.count("b"), 8000 + 50)

    def test_nota_ausente_en_entrada(self):
        ctx = {**self.CTX, "nota": None, "nota_ausente": True}
        e = atribuir.construye_entrada(self.MSG, [], ctx, None, self.PROMPT)
        self.assertNotIn("NOTA-X", e)


class TestAtribuir(Base):
    def test_solo_positivos(self):
        self.crea_kb([("2026-10-08T00:00:00+0000", "n")])
        ms = [(f"m{k}", f"hola {k}", f"2026-10-10T10:{k:02d}:00Z") for k in range(9)]
        self.sesion("s1", ms, "2026-10-10T10:00:00Z")
        self.detecciones([("s1", f"m{k}", "si" if k < 3 else "no" if k < 8 else None) for k in range(9)])
        r = self.corre()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual([c["ids"] for c in self.calls()], [["m0"], ["m1"], ["m2"]])
        self.assertEqual([l["id"] for l in self.lineas()], ["m0", "m1", "m2"])

    def test_reglas_canal_en_entrada(self):
        self.crea_kb([("2026-10-08T00:00:00+0000", "n")])
        self.sesion("s1", [("m0", "uno", "2026-10-10T10:00:00Z")], "2026-10-10T10:00:00Z", reglas=REGLAS)
        self.sesion("s2", [("m1", "dos", "2026-10-10T11:00:00Z")], "2026-10-10T11:00:00Z")
        self.detecciones([("s1", "m0", "si"), ("s2", "m1", "si")])
        self.assertEqual(self.corre().returncode, 0)
        p = {c["ids"][0]: c["prompt"] for c in self.calls()}
        self.assertIn("REGLA-DE-CANAL-UNICA", p["m0"]); self.assertNotIn("ninguna", p["m0"])
        self.assertNotIn("REGLA-DE-CANAL-UNICA", p["m1"]); self.assertIn("ninguna", p["m1"])

    def test_commit_por_sesion(self):
        h = self.crea_kb([("2026-10-08T00:00:00+0000", "VERSION-UNO"), ("2026-10-12T00:00:00+0000", "VERSION-DOS"),
                          ("2026-10-20T00:00:00+0000", "VERSION-TRES")])
        self.sesion("s1", [("m0", "uno", "2026-10-10T10:00:00Z")], "2026-10-10T10:00:00Z")
        self.sesion("s2", [("m1", "dos", "2026-10-15T10:00:00Z")], "2026-10-15T10:00:00Z")
        self.detecciones([("s1", "m0", "si"), ("s2", "m1", "si")])
        r = self.corre(FAKE_NOTA_HEAD="notas/p.md")
        self.assertEqual(r.returncode, 0, r.stderr)
        p = {c["ids"][0]: c["prompt"] for c in self.calls()}
        self.assertIn("VERSION-UNO", p["m0"]); self.assertNotIn("VERSION-DOS", p["m0"]); self.assertNotIn("VERSION-TRES", p["m0"])
        self.assertIn("VERSION-DOS", p["m1"]); self.assertNotIn("VERSION-UNO", p["m1"]); self.assertNotIn("VERSION-TRES", p["m1"])
        self.assertEqual([l["commit_kb"] for l in self.lineas()], [h[2], h[1]])
        cwds = [json.loads(l)["cwd"] for l in open(os.path.join(self.fakedir, "exo.jsonl"))]
        self.assertEqual(cwds, ["/w/x", "/w/x"])

    def test_nota_ausente_publica(self):
        self.crea_kb([("2026-10-08T00:00:00+0000", "VERSION-UNO")])
        self.sesion("s1", [("m0", "uno", "2026-10-10T10:00:00Z")], "2026-10-10T10:00:00Z")
        self.detecciones([("s1", "m0", "si")])
        self.assertEqual(self.corre().returncode, 0)  # exo falla: cwd ya no existe
        self.assertIs(self.lineas()[0]["nota_ausente"], True)
        self.assertNotIn("VERSION-UNO", self.calls()[0]["prompt"])
        shutil.rmtree(self.cache); os.remove(self.salida)
        self.assertEqual(self.corre(FAKE_NOTA_HEAD="notas/p.md").returncode, 0)
        self.assertIs(self.lineas()[0]["nota_ausente"], False)
        self.assertIn("VERSION-UNO", self.calls()[1]["prompt"])

    def test_nota_creada_despues_es_ausente(self):
        self.crea_kb([("2026-10-20T00:00:00+0000", "VERSION-UNO")])
        self.sesion("s1", [("m0", "uno", "2026-10-10T10:00:00Z")], "2026-10-10T10:00:00Z")
        self.detecciones([("s1", "m0", "si")])
        self.assertEqual(self.corre(FAKE_NOTA_HEAD="notas/p.md").returncode, 0)
        l = self.lineas()[0]
        self.assertIs(l["nota_ausente"], True)
        self.assertIsNone(l["commit_kb"])

    def test_salida_publica_sin_texto(self):
        self.crea_kb([("2026-10-08T00:00:00+0000", "n")])
        self.sesion("s1", [("m0", SECRETO + " uno", "2026-10-10T10:00:00Z")], "2026-10-10T10:00:00Z")
        self.detecciones([("s1", "m0", "si")])
        r = self.corre(FAKE_ATRIB="canon-kb", FAKE_NOTA_CITA="notas/p.md", FAKE_LINEA="LINEA-CITADA")
        self.assertEqual(r.returncode, 0, r.stderr)
        crudo = open(self.salida).read()
        for t in (SECRETO, "LINEA-CITADA", "notas/p.md"):
            self.assertNotIn(t, crudo)
        l = self.lineas()
        self.assertEqual(len(l), 1)
        self.assertEqual(set(l[0]), {"sesion", "id", "atribucion", "nota_ausente", "commit_kb"})
        self.assertEqual((l[0]["sesion"], l[0]["id"], l[0]["atribucion"]), ("s1", "m0", "canon-kb"))
        c = open(os.path.join(self.cache, "atribuidor.jsonl")).read()
        for t in (SECRETO, "LINEA-CITADA", "notas/p.md"):
            self.assertIn(t, c)
        o = json.loads(c)
        self.assertEqual((o["nota"], o["linea"]), ("notas/p.md", "LINEA-CITADA"))

    def test_cache_y_prompt(self):
        self.crea_kb([("2026-10-08T00:00:00+0000", "n")])
        self.sesion("s1", [("m0", "uno", "2026-10-10T10:00:00Z")], "2026-10-10T10:00:00Z")
        self.detecciones([("s1", "m0", "si")])
        self.assertEqual(self.corre().returncode, 0)
        primera = open(self.salida).read()
        os.remove(self.salida)
        self.assertEqual(self.corre().returncode, 0)
        self.assertEqual(len(self.calls()), 1)
        self.assertEqual(open(self.salida).read(), primera)
        prompt = os.path.join(H, "atribuidor.prompt.md")
        orig = open(prompt).read()
        self.addCleanup(lambda: open(prompt, "w").write(orig))
        with open(prompt, "w") as f:
            f.write(orig + "\nNota extra.\n")
        os.remove(self.salida)
        self.assertEqual(self.corre().returncode, 0)
        self.assertEqual(len(self.calls()), 2)

    def test_salida_invalida_reintenta_y_sin_veredicto(self):
        self.crea_kb([("2026-10-08T00:00:00+0000", "n")])
        self.sesion("s1", [("m0", "uno", "2026-10-10T10:00:00Z")], "2026-10-10T10:00:00Z")
        self.detecciones([("s1", "m0", "si")])
        r = self.corre(FAKE_ATRIB="inventada")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(len(self.calls()), 2)
        self.assertEqual(self.lineas(), [{"sesion": "s1", "id": "m0", "sin_veredicto": True}])

    def test_anteriores_son_los_mensajes_previos(self):
        self.crea_kb([("2026-10-08T00:00:00+0000", "n")])
        self.sesion("s1", [("m0", "PRIMERO", "2026-10-10T10:00:00Z"), ("m1", "SEGUNDO", "2026-10-10T10:01:00Z"),
                           ("m2", "TERCERO", "2026-10-10T10:02:00Z")], "2026-10-10T10:00:00Z")
        self.detecciones([("s1", "m1", "si")])
        self.assertEqual(self.corre().returncode, 0)
        p = self.calls()[0]["prompt"]
        self.assertIn("PRIMERO", p)
        self.assertNotIn("TERCERO", p)

    def test_tope_exit_3(self):
        self.crea_kb([("2026-10-08T00:00:00+0000", "n")])
        ms = [(f"m{k}", f"hola {k}", f"2026-10-10T10:{k:02d}:00Z") for k in range(30)]
        self.sesion("s1", ms, "2026-10-10T10:00:00Z")
        self.detecciones([("s1", f"m{k}", "si") for k in range(30)])
        r = self.corre("--tope-usd", "25", FAKE_COSTE="10")
        self.assertEqual(r.returncode, 3, r.stderr)
        self.assertEqual(len(self.calls()), 3)
        self.assertIn("coste_total_usd=30", r.stdout)
        self.assertTrue(os.path.exists(self.salida + ".parcial"))
        self.assertFalse(os.path.exists(self.salida))
        self.assertEqual(len(self.lineas(self.salida + ".parcial")), 3)

    def test_fallo_proceso_exit_4(self):
        self.crea_kb([("2026-10-08T00:00:00+0000", "n")])
        self.sesion("s1", [("m0", "uno", "2026-10-10T10:00:00Z")], "2026-10-10T10:00:00Z")
        self.detecciones([("s1", "m0", "si")])
        r = self.corre(FAKE_RC="1")
        self.assertEqual(r.returncode, 4)
        self.assertTrue(os.path.exists(self.salida + ".parcial"))

    def test_sello_no_casa_exit_2(self):
        self.crea_kb([("2026-10-08T00:00:00+0000", "n")])
        self.sesion("s1", [("m0", "uno", "2026-10-10T10:00:00Z")], "2026-10-10T10:00:00Z")
        self.detecciones([("s1", "m0", "si")])
        sello = os.path.join(self.tmp, "sello.json")
        bueno = {"prompt_atribuidor_sha256": juez.sha256_fichero(os.path.join(H, "atribuidor.prompt.md")),
                 "claude_version": "9.9.9 (Fake)"}
        json.dump({**bueno, "prompt_atribuidor_sha256": "0" * 64}, open(sello, "w"))
        self.assertEqual(self.corre("--sello", sello).returncode, 2)
        self.assertEqual(self.calls(), [])
        json.dump(bueno, open(sello, "w"))
        self.assertEqual(self.corre("--sello", sello).returncode, 0)

    def test_deteccion_de_sesion_desconocida_exit_1(self):
        self.crea_kb([("2026-10-08T00:00:00+0000", "n")])
        self.sesion("s1", [("m0", "uno", "2026-10-10T10:00:00Z")], "2026-10-10T10:00:00Z")
        self.detecciones([("s1", "m0", "si"), ("sX", "mX", "si")])
        self.assertEqual(self.corre().returncode, 1)
        self.assertEqual(self.calls(), [])

    def test_invocacion_aislada(self):
        self.crea_kb([("2026-10-08T00:00:00+0000", "n")])
        self.sesion("s1", [("m0", "uno", "2026-10-10T10:00:00Z")], "2026-10-10T10:00:00Z")
        self.detecciones([("s1", "m0", "si")])
        self.assertEqual(self.corre().returncode, 0)
        a = self.calls()[0]["argv"]
        self.assertEqual(a[a.index("--model") + 1], "claude-sonnet-5-5")
        self.assertEqual(a[a.index("--max-budget-usd") + 1], "0.50")


if __name__ == "__main__":
    unittest.main()
