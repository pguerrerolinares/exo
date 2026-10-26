"""Tests de etiquetar.py y sellar.py con un `claude` falso y datos sintéticos inventados."""
import hashlib, json, os, re, shutil, stat, subprocess, sys, tempfile, unittest

H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, H)
import etiquetar, sellar

FAKE = '''#!/usr/bin/env python3
import json, os, re, sys
if sys.argv[1:] == ["--version"]:
    print(os.environ.get("FAKE_VERSION", "9.9.9 (Fake)")); sys.exit(0)
d = os.environ["FAKE_DIR"]
prompt = sys.argv[-1]
with open(os.path.join(d, "calls.jsonl"), "a") as f:
    f.write(json.dumps({"argv": sys.argv[1:], "cwd": os.getcwd(), "prompt": prompt}) + "\\n")
n = sum(1 for _ in open(os.path.join(d, "calls.jsonl")))
if n <= int(os.environ.get("FAKE_BASURA", "0")):
    result = "esto no es json"
else:
    i = os.environ.get("FAKE_ID") or re.search(r"^id: (\\S+)", prompt, re.M)[1]
    result = json.dumps({"id": i, "choque": "P", "coste": "alto", "justificacion": "SECRETO-JUST", "ruta": "SECRETO-RUTA"})
print(json.dumps({"type": "result", "result": result, "total_cost_usd": float(os.environ.get("FAKE_COSTE", "0.01"))}))
'''

NOTA = """# Nota sintética

## Contexto
Prosa restante uno.
La frase de origen azul vive aquí en la prosa.
Otra línea de prosa restante.

## Reglas duras
- VIÑETA-SECRETA nunca hagas la cosa.

## Cierre
Prosa restante final.
"""


def git(repo, *args):
    return subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t.local", "-C", repo, *args],
                          check=True, capture_output=True, text=True).stdout.strip()


def regla(i, proyecto="recon", cita="La frase de origen azul", ambito="codigo", generica=False):
    return {"id": i, "regla": f"Texto de la regla {i}.", "cita": cita, "proyecto": proyecto,
            "ambito": ambito, "generica": generica, "nota": "x.md", "check": None}


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
        self.kb = os.path.join(self.tmp, "kb"); os.makedirs(os.path.join(self.kb, "projects"))
        git(self.kb, "init", "-q")
        self.nota = os.path.join(self.kb, "projects", "recon.md")
        with open(self.nota, "w") as f:
            f.write(NOTA)
        git(self.kb, "add", "projects/recon.md"); git(self.kb, "commit", "-qm", "n")
        self.commit = git(self.kb, "rev-parse", "HEAD")
        self.pool = os.path.join(self.tmp, "pool"); os.makedirs(self.pool)
        self.cache = os.path.join(self.tmp, "cache")
        self.salida = os.path.join(self.tmp, "out.jsonl")

    def escribe_pool(self, reglas):
        with open(os.path.join(self.pool, "reglas-g0.jsonl"), "w") as f:
            for r in reglas:
                f.write(json.dumps(r) + "\n")

    def etiqueta(self, ids, extra=(), **env):
        idsf = os.path.join(self.tmp, "ids.txt")
        with open(idsf, "w") as f:
            f.write("".join(i + "\n" for i in ids))
        return subprocess.run([sys.executable, os.path.join(H, "etiquetar.py"), "--pool", self.pool, "--kb", self.kb,
                               "--commit", self.commit, "--ids", idsf, "--salida", self.salida, "--cache", self.cache,
                               *extra], env={**self.env, **env}, capture_output=True, text=True)

    def llamadas(self):
        p = os.path.join(self.fakedir, "calls.jsonl")
        if not os.path.exists(p):
            return []
        with open(p) as f:
            return [json.loads(l) for l in f]

    def publico(self):
        with open(self.salida) as f:
            return [json.loads(l) for l in f]


class TestEtiquetar(Base):
    def test_flags_de_aislamiento(self):
        self.escribe_pool([regla("g0-1")])
        r = self.etiqueta(["g0-1"])
        self.assertEqual(r.returncode, 0, r.stderr)
        (c,) = self.llamadas()
        argv = c["argv"]
        self.assertEqual(argv[-1], c["prompt"])
        t = argv.index("--tools")
        self.assertEqual(argv[t + 1], "")
        self.assertNotEqual(t + 2, len(argv) - 1)
        pares = [("--setting-sources", ""), ("--tools", ""), ("--output-format", "json"), ("--model", "claude-sonnet-5-5"),
                 ("--settings", '{"autoMemoryEnabled": false}'), ("--max-budget-usd", "0.10")]
        for flag, val in pares:
            self.assertEqual(argv[argv.index(flag) + 1], val, flag)
        for flag in ("-p", "--strict-mcp-config", "--no-session-persistence"):
            self.assertIn(flag, argv)
        cwd = os.path.realpath(c["cwd"])
        self.assertNotEqual(cwd, os.path.realpath(os.getcwd()))
        self.assertNotEqual(cwd, os.path.realpath(os.path.join(H, "..", "..")))

    def test_prompt_sin_seccion_ni_cita(self):
        self.escribe_pool([regla("g0-1")])
        with open(self.nota, "w") as f:  # HEAD y el working tree difieren del commit pedido
            f.write("# Otra\nTEXTO-DE-HEAD\n")
        git(self.kb, "commit", "-qam", "otra")
        with open(self.nota, "w") as f:
            f.write("# Otra\nTEXTO-DEL-WORKING-TREE\n")
        r = self.etiqueta(["g0-1"])
        self.assertEqual(r.returncode, 0, r.stderr)
        p = self.llamadas()[0]["prompt"]
        self.assertIn("Texto de la regla g0-1.", p)
        self.assertIn("Prosa restante uno.", p)
        self.assertIn("Prosa restante final.", p)
        self.assertNotIn("VIÑETA-SECRETA", p)
        self.assertNotIn("frase de origen azul", p)
        self.assertNotIn("TEXTO-DEL-WORKING-TREE", p)
        self.assertNotIn("TEXTO-DE-HEAD", p)
        self.assertNotIn("{REGLA}", p); self.assertNotIn("{NOTA}", p)

    def test_salida_publica_sin_texto(self):
        self.escribe_pool([regla("g0-1"), regla("g0-2", cita="cita que no está")])
        r = self.etiqueta(["g0-1", "g0-2"])
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(self.publico(), [{"id": "g0-1", "choque": "P", "coste": "alto", "cita_recortada": True},
                                          {"id": "g0-2", "choque": "P", "coste": "alto", "cita_recortada": False}])
        with open(self.salida) as f:
            self.assertNotIn("SECRETO", f.read())
        with open(os.path.join(self.cache, "etiquetas.jsonl")) as f:
            priv = [json.loads(l) for l in f]
        self.assertEqual([p["justificacion"] for p in priv], ["SECRETO-JUST"] * 2)
        self.assertEqual(priv[0]["ruta"], "SECRETO-RUTA")

    def test_reintenta_una_vez(self):
        self.escribe_pool([regla("g0-1")])
        r = self.etiqueta(["g0-1"], FAKE_BASURA="1")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(len(self.llamadas()), 2)
        self.assertEqual(self.publico()[0]["choque"], "P")
        shutil.rmtree(self.fakedir); os.makedirs(self.fakedir)
        r = self.etiqueta(["g0-1"], FAKE_BASURA="9")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(len(self.llamadas()), 2)
        self.assertEqual(self.publico(), [{"id": "g0-1", "sin_etiqueta": True}])

    def test_id_ajeno_es_sin_etiqueta(self):
        self.escribe_pool([regla("g0-1")])
        r = self.etiqueta(["g0-1"], FAKE_ID="g9-9")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(self.publico(), [{"id": "g0-1", "sin_etiqueta": True}])

    def test_tope_para_sin_completar(self):
        self.escribe_pool([regla("g0-1"), regla("g0-2"), regla("g0-3")])
        r = self.etiqueta(["g0-1", "g0-2", "g0-3"], ["--tope-usd", "5"], FAKE_COSTE="3.0")
        self.assertEqual(r.returncode, 3, r.stderr)
        self.assertEqual(len(self.llamadas()), 2)
        self.assertTrue(os.path.exists(self.salida + ".parcial"))
        self.assertFalse(os.path.exists(self.salida))

    def test_sello_no_casa(self):
        self.escribe_pool([regla("g0-1"), regla("g0-2"), regla("g0-3")])
        bueno = {"prompt_sha256": etiquetar.sha256_fichero(etiquetar.PROMPT), "claude_version": "9.9.9 (Fake)",
                 "kb_commit": self.commit}
        sello = os.path.join(self.tmp, "sello.json")
        for campo, val in (("prompt_sha256", "0" * 64), ("claude_version", "1.0.0"), ("kb_commit", "abc")):
            with open(sello, "w") as f:
                json.dump({**bueno, campo: val}, f)
            r = self.etiqueta(["g0-1"], ["--sello", sello])
            self.assertEqual(r.returncode, 2, campo + r.stderr)
            self.assertEqual(self.llamadas(), [], campo)
        with open(sello, "w") as f:
            json.dump({**bueno, "universo_sha256": "ignorado", "n": 3}, f)
        r = self.etiqueta(["g0-1", "g0-3"], ["--sello", sello])
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual([x["id"] for x in self.publico()], ["g0-1", "g0-3"])

    def test_repo_none_no_va_al_modelo(self):
        self.escribe_pool([regla("g0-1"), regla("g0-2", proyecto="CV")])
        r = self.etiqueta(["g0-1", "g0-2"])
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("g0-2", r.stderr)
        self.assertEqual(self.llamadas(), [])


class TestSellar(Base):
    def test_sellar_universo(self):
        self.escribe_pool([regla("g0-10"), regla("g0-9"), regla("g1-1"),
                           regla("g0-11"),                                  # sin criba
                           regla("g0-12", proyecto="transversal"),          # transversal
                           regla("g0-13", proyecto="CV"),                   # REPO=None
                           regla("g0-14", generica=True),                   # no candidata
                           regla("g0-15", ambito="otro")])                  # ámbito fuera
        criba = os.path.join(self.tmp, "criba")
        for i in ("g0-10", "g0-9", "g1-1", "g0-12", "g0-13", "g0-14", "g0-15"):
            os.makedirs(os.path.join(criba, i, "a0-r1"))
            with open(os.path.join(criba, i, "a0-r1", "check.rc"), "w") as f:
                f.write("0")
        out = os.path.join(self.tmp, "sellado")
        r = subprocess.run([sys.executable, os.path.join(H, "sellar.py"), "--pool", self.pool, "--criba", criba,
                            "--prompt", etiquetar.PROMPT, "--kb", self.kb, "--commit", self.commit,
                            "--salida-dir", out], env=self.env, capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        with open(os.path.join(out, "universo.txt")) as f:
            texto = f.read()
        self.assertEqual(texto, "g0-9\ng0-10\ng1-1\n")
        with open(os.path.join(out, "sello.json")) as f:
            sello = json.load(f)
        self.assertEqual(sello["n"], len(texto.splitlines()))
        self.assertEqual(sello["universo_sha256"], hashlib.sha256(texto.encode()).hexdigest())
        self.assertEqual(sello["prompt_sha256"], etiquetar.sha256_fichero(etiquetar.PROMPT))
        self.assertEqual((sello["kb_commit"], sello["claude_version"]), (self.commit, "9.9.9 (Fake)"))


if __name__ == "__main__":
    unittest.main()
