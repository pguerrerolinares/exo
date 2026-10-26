"""Tests de sellar.py y resultado.py con transcripts sintéticos y un `claude` falso que solo imprime una versión."""
import json, os, re, shutil, stat, subprocess, sys, tempfile, unittest

H = os.path.dirname(os.path.abspath(__file__))

FAKE_CLAUDE = '''#!/usr/bin/env python3
import json, sys
if sys.argv[1:] == ["--version"]:
    print("9.9.9 (Fake)"); sys.exit(0)
ids = [json.loads(l[4:])["id"] for l in sys.argv[-1].splitlines() if l.startswith("MSG ")]
print(json.dumps({"type": "result", "total_cost_usd": 0.01,
                  "result": json.dumps([{"id": i, "correccion": "no", "tipo": None} for i in ids])}))
'''


def sesion(sid, n_msgs=1, ts="2026-10-10T10:00:00Z", extra=(), cwd="/w/x"):
    base = {"timestamp": ts, "cwd": cwd, "entrypoint": "cli"}
    out = []
    for n in range(n_msgs):
        out.append({**base, "type": "user", "uuid": f"{sid}-m{n}", "message": {"role": "user", "content": f"msg {n}"}})
    out.extend({**base, "type": "assistant", "uuid": f"{sid}-a{n}",
                "message": {"content": [{"type": "tool_use", "name": "Bash", "input": {"command": c}}]}}
               for n, c in enumerate(extra))
    return out


class Base(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, self.tmp, True)
        self.proy = os.path.join(self.tmp, "proy")
        self.kb = os.path.join(self.tmp, "kb")
        subprocess.run(["git", "init", "-q", self.kb], check=True)
        subprocess.run(["git", "-C", self.kb, "-c", "user.name=t", "-c", "user.email=t@t",
                        "commit", "-q", "--allow-empty", "-m", "x"], check=True)
        self.claude = os.path.join(self.tmp, "claude")
        with open(self.claude, "w") as f:
            f.write(FAKE_CLAUDE)
        os.chmod(self.claude, os.stat(self.claude).st_mode | stat.S_IXUSR)
        self.env = {**os.environ, "CLAUDE_BIN": self.claude}
        self.sello = os.path.join(self.tmp, "sello.json")
        self.res = os.path.join(self.tmp, "resultado.json")
        self.cache = os.path.join(self.tmp, "cache")

    def escribe(self, sid, lineas, sub=None, repo="repo"):
        d = os.path.join(self.proy, repo)
        os.makedirs(d, exist_ok=True)
        with open(os.path.join(d, sid + ".jsonl"), "w") as f:
            f.writelines(json.dumps(l) + "\n" for l in lineas)
        for nombre, ls in (sub or {}).items():
            sd = os.path.join(d, sid, "subagents")
            os.makedirs(sd, exist_ok=True)
            with open(os.path.join(sd, nombre + ".jsonl"), "w") as f:
                f.writelines(json.dumps(l) + "\n" for l in ls)

    def corre(self, script, *args):
        return subprocess.run([sys.executable, os.path.join(H, script), *args], env=self.env,
                              capture_output=True, text=True)

    def sella(self, *extra):
        return self.corre("sellar.py", "--proyectos", self.proy, "--kb", self.kb, "--salida", self.sello, *extra)

    def lee(self, p):
        with open(p) as f:
            return json.load(f)

    def jsonl(self, nombre, filas):
        p = os.path.join(self.tmp, nombre)
        with open(p, "w") as f:
            f.writelines(json.dumps(x) + "\n" for x in filas)
        return p

    def resultado(self, dets, atribs=None, validacion=None, validacion2=None):
        args = ["--detecciones", self.jsonl("det.jsonl", dets), "--proyectos", self.proy,
                "--sello", self.sello, "--salida", self.res, "--cache", self.cache]
        if atribs is not None:
            args += ["--atribuciones", self.jsonl("atr.jsonl", atribs)]
        for flag, v, n in (("--validacion", validacion, "val1.json"), ("--validacion2", validacion2, "val2.json")):
            if v is not None:
                p = os.path.join(self.tmp, n)
                with open(p, "w") as f:
                    json.dump(v, f)
                args += [flag, p]
        return self.corre("resultado.py", *args)


class TestSellar(Base):
    def test_sello_estable(self):
        self.escribe("s1", sesion("s1"))
        self.escribe("s2", sesion("s2"))
        self.assertEqual(self.sella().returncode, 0)
        a = self.lee(self.sello)
        self.assertEqual(self.sella("--forzar").returncode, 0)
        self.assertEqual(self.lee(self.sello)["sesiones_sha256"], a["sesiones_sha256"])
        self.assertEqual(a["n_sesiones"], 2)
        # Cambiar un id sin cambiar n también cambia el hash.
        os.rename(os.path.join(self.proy, "repo", "s2.jsonl"), os.path.join(self.proy, "repo", "s3.jsonl"))
        self.sella("--forzar")
        b = self.lee(self.sello)
        self.assertEqual(b["n_sesiones"], 2)
        self.assertNotEqual(b["sesiones_sha256"], a["sesiones_sha256"])
        self.escribe("s4", sesion("s4"))
        self.sella("--forzar")
        self.assertNotEqual(self.lee(self.sello)["sesiones_sha256"], b["sesiones_sha256"])

    def test_sello_contenido(self):
        self.escribe("s1", sesion("s1"))
        self.assertEqual(self.sella("--desde", "2026-09-01").returncode, 0)
        s = self.lee(self.sello)
        self.assertEqual(s["desde"], "2026-09-01")
        self.assertEqual(s["claude_version"], "9.9.9 (Fake)")
        self.assertEqual(s["umbrales"], {"puerta": 0.4, "kappa": 0.4, "n_min_invalidacion": 30, "frac_invalidacion": 0.10})
        self.assertEqual((s["lote"], s["semilla"]), (10, 20261026))
        self.assertEqual(len(s["kb_commit"]), 40)
        self.assertEqual(set(s["modelos"]), {"detector", "atribuidor"})
        sys.path.insert(0, H)
        import transcripts
        import inspect, hashlib
        self.assertEqual(s["filtro_sha256"],
                         hashlib.sha256(inspect.getsource(transcripts.es_mensaje_humano).encode()).hexdigest())

    def test_sello_no_sobrescribe(self):
        self.escribe("s1", sesion("s1"))
        self.assertEqual(self.sella().returncode, 0)
        self.assertTrue(os.path.exists(self.sello))
        with open(self.sello, "rb") as f:
            antes = f.read()
        self.escribe("s2", sesion("s2"))
        self.assertEqual(self.sella().returncode, 2)
        with open(self.sello, "rb") as f:
            self.assertEqual(f.read(), antes)
        self.assertEqual(self.sella("--forzar").returncode, 0)
        with open(self.sello, "rb") as f:
            self.assertNotEqual(f.read(), antes)

    def test_detectar_y_atribuir_aceptan_el_sello(self):
        self.escribe("s1", sesion("s1"))
        self.assertEqual(self.sella().returncode, 0)
        cache = os.path.join(self.tmp, "cache")
        r = self.corre("detectar.py", "--proyectos", self.proy, "--cache", cache, "--salida",
                       os.path.join(self.tmp, "d.jsonl"), "--sello", self.sello)
        self.assertEqual(r.returncode, 0, r.stderr)
        r = self.corre("atribuir.py", "--proyectos", self.proy, "--cache", cache, "--salida",
                       os.path.join(self.tmp, "a.jsonl"), "--detecciones", os.path.join(self.tmp, "d.jsonl"),
                       "--kb", self.kb, "--sello", self.sello)
        self.assertEqual(r.returncode, 0, r.stderr)
        s = self.lee(self.sello)
        s["claude_version"] = "otra"
        with open(self.sello, "w") as f:
            json.dump(s, f)
        r = self.corre("detectar.py", "--proyectos", self.proy, "--cache", cache, "--salida",
                       os.path.join(self.tmp, "d.jsonl"), "--sello", self.sello)
        self.assertEqual(r.returncode, 2)

    def test_resellar_con_forzar_conserva_el_hasta(self):
        self.escribe("s1", sesion("s1"))
        self.escribe("vieja", sesion("vieja", ts="2026-10-01T10:00:00Z"))
        self.assertEqual(self.sella("--hasta", "2026-10-20T00:00:00Z").returncode, 0)
        self.escribe("tarde", sesion("tarde", ts="2026-10-21T10:00:00Z"))
        self.assertEqual(self.sella("--forzar", "--desde", "2026-09-01").returncode, 0)
        s = self.lee(self.sello)
        self.assertEqual((s["hasta"], s["n_sesiones"], s["desde"]), ("2026-10-20T00:00:00Z", 2, "2026-09-01"))
        # Un --hasta explícito manda sobre el del sello previo.
        self.sella("--forzar", "--hasta", "2026-10-22T00:00:00Z")
        self.assertEqual(self.lee(self.sello)["n_sesiones"], 2)  # s1 y tarde; vieja queda fuera por el desde por defecto

    def test_sellar_nuevo_sin_hasta_es_ahora(self):
        self.escribe("s1", sesion("s1"))
        self.sella()
        self.assertRegex(self.lee(self.sello)["hasta"], r"^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\dZ$")


def det(sesion_, i, si=False, sin=False):
    if sin:
        return {"sesion": sesion_, "id": i, "sin_veredicto": True}
    return {"sesion": sesion_, "id": i, "correccion": "si" if si else "no", "tipo": "redirige" if si else None}


class TestResultado(Base):
    def setUp(self):
        super().setUp()
        for n in range(10):
            self.escribe(f"s{n}", sesion(f"s{n}"))
        self.sella()

    def dets(self, k):
        return [det(f"s{n}", f"s{n}-m0", si=n < k) for n in range(10)]

    def test_resultado_juez_invalido(self):
        r = self.resultado(self.dets(1), validacion={"juez_valido": False, "kappa": 0.1, "n": 80})
        self.assertEqual(r.returncode, 0, r.stderr)
        o = self.lee(self.res)
        self.assertEqual(o["puerta"]["estado"], "juez_invalido")
        self.assertLess(o["puerta"]["tasa"], 0.4)
        self.assertEqual(o["validacion"]["juez_valido"], False)
        # Contraparte: un juez válido con la misma tasa baja estabiliza.
        self.resultado(self.dets(0), validacion={"juez_valido": True, "kappa": 0.8, "n": 80})
        self.assertEqual(self.lee(self.res)["puerta"]["estado"], "estabilizar")

    def test_resultado_juez_sin_validar(self):
        for v in ({"kappa": 0.9}, {"juez_valido": None}):
            self.resultado(self.dets(0), validacion=v)
            self.assertEqual(self.lee(self.res)["puerta"]["estado"], "juez_invalido", v)
        self.resultado(self.dets(0))
        self.assertEqual(self.lee(self.res)["puerta"]["estado"], "sin_validar")

    def test_resultado_sin_etapa2(self):
        k = 5
        self.resultado(self.dets(k), validacion={"juez_valido": True})
        o = self.lee(self.res)
        self.assertEqual(o["puerta"]["estado"], "etapa2")
        self.assertIsNone(o["final"])
        atribs = [{"sesion": f"s{n}", "id": f"s{n}-m0", "atribucion": "nueva", "nota_ausente": n == 0, "commit_kb": "c"}
                  for n in range(k)]
        self.resultado(self.dets(k), atribs=atribs, validacion={"juez_valido": True},
                       validacion2={"n": 30, "acuerdo": 0.7})
        o = self.lee(self.res)
        self.assertEqual(o["final"]["n_correcciones"], 5)
        self.assertEqual(o["final"]["por_atribucion"], {"nueva": 5})
        self.assertEqual(o["n_nota_ausente"], 1)
        self.assertEqual(o["validacion2"], {"n": 30, "acuerdo": 0.7})
        # Negativo: puerta "estabilizar" no calcula final (con k=0 no puede haber atribuciones válidas).
        self.resultado(self.dets(0), atribs=[], validacion={"juez_valido": True})
        o = self.lee(self.res)
        self.assertEqual(o["puerta"]["estado"], "estabilizar")
        self.assertIsNone(o["final"])

    def test_final_solo_si_la_puerta_abre_etapa2(self):
        # Un positivo con su atribución, pero juez inválido: la puerta no abre la etapa 2 y no hay final.
        self.resultado(self.dets(1), atribs=[{"sesion": "s0", "id": "s0-m0", "atribucion": "nueva", "nota_ausente": False,
                                              "commit_kb": "c"}], validacion={"juez_valido": False})
        o = self.lee(self.res)
        self.assertEqual(o["puerta"]["estado"], "juez_invalido")
        self.assertIsNone(o["final"])

    def test_resultado_sello_cambiado(self):
        r = self.resultado(self.dets(1))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertTrue(self.lee(self.res)["sello_ok"])
        self.escribe("nueva", sesion("nueva"))
        r = self.resultado(self.dets(1))
        self.assertEqual(r.returncode, 2)
        self.assertFalse(self.lee(self.res)["sello_ok"])

    def test_denominador_es_la_poblacion_sellada(self):
        # s8 solo tiene sin_veredicto y s9 ninguna fila: ambas siguen en el denominador (10).
        dets = [det(f"s{n}", f"s{n}-m0", si=n == 0) for n in range(8)] + [det("s8", "s8-m0", sin=True)]
        self.resultado(dets, validacion={"juez_valido": True})
        o = self.lee(self.res)
        self.assertEqual((o["puerta"]["n"], o["puerta"]["k"], o["n_sin_veredicto"]), (10, 1, 1))
        self.assertEqual(o["puerta"]["estado"], "incompleto")  # a s9 le faltan detecciones

    def test_hasta_cierra_la_poblacion_sellada(self):
        self.escribe("futura", sesion("futura", ts="2026-10-25T10:00:00Z"))
        self.assertEqual(self.sella("--forzar", "--hasta", "2026-10-20T00:00:00Z").returncode, 0)
        self.assertEqual(self.lee(self.sello)["n_sesiones"], 10)
        self.assertEqual(self.lee(self.sello)["hasta"], "2026-10-20T00:00:00Z")
        # Sesión nueva posterior y mensaje nuevo (ts posterior) en una sesión sellada.
        self.escribe("nueva", sesion("nueva", ts="2026-10-21T10:00:00Z"))
        with open(os.path.join(self.proy, "repo", "s0.jsonl"), "a") as f:
            f.write(json.dumps({"timestamp": "2026-10-21T11:00:00Z", "cwd": "/w/x", "entrypoint": "cli", "type": "user",
                                "uuid": "s0-tarde", "message": {"role": "user", "content": "tarde"}}) + "\n")
        r = self.resultado(self.dets(1))
        self.assertEqual(r.returncode, 0, r.stderr)
        o = self.lee(self.res)
        self.assertTrue(o["sello_ok"])
        self.assertEqual((o["puerta"]["n"], o["hasta"]), (10, "2026-10-20T00:00:00Z"))

    def test_detectar_con_sello_no_juzga_lo_posterior(self):
        self.sella("--forzar", "--hasta", "2026-10-20T00:00:00Z")
        self.escribe("nueva", sesion("nueva", ts="2026-10-21T10:00:00Z"))
        with open(os.path.join(self.proy, "repo", "s0.jsonl"), "a") as f:
            f.write(json.dumps({"timestamp": "2026-10-21T11:00:00Z", "cwd": "/w/x", "entrypoint": "cli", "type": "user",
                                "uuid": "s0-tarde", "message": {"role": "user", "content": "tarde"}}) + "\n")
        salida = os.path.join(self.tmp, "d.jsonl")
        r = self.corre("detectar.py", "--proyectos", self.proy, "--cache", os.path.join(self.tmp, "c"),
                       "--salida", salida, "--sello", self.sello)
        self.assertEqual(r.returncode, 0, r.stderr)
        ids = {json.loads(l)["id"] for l in open(salida)}
        self.assertEqual(ids, {f"s{n}-m0" for n in range(10)})

    def test_deterministas_principales_y_subagentes(self):
        self.escribe("s0", sesion("s0", extra=["cd x && git status", "ls"]),
                     sub={"ag": sesion("ag", extra=["git stash", "git stash", "ls", "ls"])})
        self.sella("--forzar")
        self.resultado(self.dets(0))
        d = self.lee(self.res)["deterministas"]
        self.assertEqual(d["principales"]["llamadas_bash"], 2)
        self.assertEqual(d["principales"]["por_regla"], {"cd-git": 1})
        self.assertEqual(d["subagentes"]["llamadas_bash"], 4)
        self.assertEqual(d["subagentes"]["por_regla"], {"git-stash": 2})

    def test_sin_veredicto_etapa2_se_declara(self):
        def atribs(n, sin):
            return [{"sesion": f"s{i}", "id": f"s{i}-m0", "sin_veredicto": True} if i < sin else
                    {"sesion": f"s{i}", "id": f"s{i}-m0", "atribucion": "nueva", "nota_ausente": False, "commit_kb": "c"}
                    for i in range(n)]
        v = {"juez_valido": True}
        # 1 de 5 (20 % > 10 %): final incompleto y declarado, no se estabiliza.
        self.resultado(self.dets(5), atribs=atribs(5, 1), validacion=v)
        o = self.lee(self.res)
        self.assertEqual((o["n_sin_veredicto_etapa2"], o["final"]["n_correcciones"], o["final"]["estado"]), (1, 4, "incompleto"))
        # 1 de 10 (10 % exacto): no es > 10 %, el estado sale de la puerta.
        self.resultado(self.dets(10), atribs=atribs(10, 1), validacion=v)
        o = self.lee(self.res)
        self.assertEqual(o["n_sin_veredicto_etapa2"], 1)
        self.assertNotEqual(o["final"]["estado"], "incompleto")
        self.resultado(self.dets(5), atribs=atribs(5, 0), validacion=v)
        self.assertEqual(self.lee(self.res)["n_sin_veredicto_etapa2"], 0)

    def poblacion40(self):
        for n in range(10, 40):
            self.escribe(f"s{n}", sesion(f"s{n}"))
        self.sella("--forzar")
        return [det(f"s{n}", f"s{n}-m0", si=True) for n in range(40)]

    @staticmethod
    def atr(n):
        return {"sesion": f"s{n}", "id": f"s{n}-m0", "atribucion": "nueva", "nota_ausente": False, "commit_kb": "c"}

    def test_positivos_sin_atribucion_son_incompleto(self):
        dets = self.poblacion40()
        v = {"juez_valido": True}
        # 30 de 40: faltan 10 (25 % > 10 %).
        self.resultado(dets, atribs=[self.atr(n) for n in range(30)], validacion=v)
        o = self.lee(self.res)
        self.assertEqual((o["n_atribuciones_faltantes"], o["final"]["estado"]), (10, "incompleto"))
        # 39 de 40: falta 1 (2,5 %): no incompleto, pero declarado.
        self.resultado(dets, atribs=[self.atr(n) for n in range(39)], validacion=v)
        o = self.lee(self.res)
        self.assertEqual(o["n_atribuciones_faltantes"], 1)
        self.assertNotEqual(o["final"]["estado"], "incompleto")
        # Los sin_veredicto y los faltantes suman: 2 + 3 de 40 > 10 %.
        sv = [{"sesion": f"s{n}", "id": f"s{n}-m0", "sin_veredicto": True} for n in range(2)]
        self.resultado(dets, atribs=sv + [self.atr(n) for n in range(2, 37)], validacion=v)
        self.assertEqual(self.lee(self.res)["final"]["estado"], "incompleto")

    def test_atribucion_que_no_es_positivo_exit_1(self):
        dets = self.poblacion40()
        dets[5] = det("s5", "s5-m0", si=False)
        r = self.resultado(dets, atribs=[self.atr(n) for n in range(40)], validacion={"juez_valido": True})
        self.assertEqual(r.returncode, 1)
        self.assertIn("s5-m0", r.stderr)

    def test_atribucion_duplicada_exit_1(self):
        dets = self.poblacion40()
        r = self.resultado(dets, atribs=[self.atr(n) for n in range(40)] + [self.atr(3)], validacion={"juez_valido": True})
        self.assertEqual(r.returncode, 1)
        self.assertIn("s3-m0", r.stderr)

    def test_sello_filtro_cambiado(self):
        s = self.lee(self.sello)
        s["filtro_sha256"] = "0" * 64
        with open(self.sello, "w") as f:
            json.dump(s, f)
        r = self.resultado(self.dets(1))
        self.assertEqual(r.returncode, 2)
        self.assertFalse(self.lee(self.res)["sello_ok"])

    def test_deterministas_por_repo(self):
        self.escribe("s0", sesion("s0", extra=["cd x && git status"], cwd="/w/repo"), sub={"ag": sesion("ag", extra=["git stash"])})
        self.escribe("r1", sesion("r1", extra=["ls", "ls"], cwd="/w/otro"), repo="otro")
        self.sella("--forzar")
        self.resultado(self.dets(0))
        d = self.lee(self.res)["deterministas"]
        self.assertEqual(d["principales"]["llamadas_bash"], 3)
        import hashlib
        h = lambda c: hashlib.sha256(c.encode()).hexdigest()[:8]
        pr = {n: d["por_repo"][h("/w/" + n)] for n in ("repo", "otro")}
        self.assertEqual(len(d["por_repo"]), 3)
        self.assertEqual(pr["repo"]["principales"]["por_regla"], {"cd-git": 1})
        self.assertEqual(pr["repo"]["subagentes"]["por_regla"], {"git-stash": 1})
        self.assertEqual((pr["otro"]["principales"]["llamadas_bash"], pr["otro"]["principales"]["por_regla"]), (2, {}))
        self.assertEqual(pr["otro"]["subagentes"]["llamadas_bash"], 0)

    def test_por_repo_solo_lleva_hashes_y_la_cache_los_resuelve(self):
        import hashlib
        self.escribe("h0", sesion("h0", cwd="/home/alguien"), repo="-home-alguien")
        self.escribe("k0", sesion("k0", cwd="/home/alguien/x/kb-privada"), repo="-home-alguien-x-kb-privada")
        self.sella("--forzar")
        self.resultado(self.dets(0))
        h = lambda c: hashlib.sha256(c.encode()).hexdigest()[:8]
        pr = self.lee(self.res)["deterministas"]["por_repo"]
        self.assertTrue(all(re.fullmatch(r"[0-9a-f]{8}", k) for k in pr), list(pr))
        self.assertIn(h("/home/alguien"), pr)
        with open(self.res) as f:
            txt = f.read()
        for t in ("home", "alguien", "kb-privada"):
            self.assertNotIn(t, txt)
        repos = self.lee(os.path.join(self.cache, "repos.json"))
        self.assertEqual((repos[h("/home/alguien")], repos[h("/home/alguien/x/kb-privada")], repos[h("/w/x")]),
                         ("alguien", "kb-privada", "x"))

    def test_por_repo_sin_cwd_es_desconocido(self):
        self.escribe("s0", [{k: v for k, v in l.items() if k != "cwd"} for l in sesion("s0")], repo="-home-alguien-x")
        self.sella("--forzar")
        self.resultado(self.dets(0))
        repos = self.lee(os.path.join(self.cache, "repos.json"))
        self.assertIn("desconocido", repos.values())
        with open(self.res) as f:
            self.assertNotIn("alguien", f.read())

    def test_deterministas_respetan_hasta(self):
        tarde = {"timestamp": "2026-10-21T10:00:00Z", "type": "assistant", "uuid": "t",
                 "message": {"content": [{"type": "tool_use", "name": "Bash", "input": {"command": "git stash"}}]}}
        self.escribe("s0", sesion("s0", extra=["ls"]) + [tarde], sub={"ag": sesion("ag", extra=["ls", "ls"]) + [tarde]})
        self.sella("--forzar", "--hasta", "2026-10-20T00:00:00Z")
        self.resultado(self.dets(0))
        d = self.lee(self.res)["deterministas"]["principales"]
        self.assertEqual((d["llamadas_bash"], d["por_regla"]), (1, {}))
        d = self.lee(self.res)["deterministas"]["subagentes"]
        self.assertEqual((d["llamadas_bash"], d["por_regla"]), (2, {}))

    def test_deteccion_fuera_de_la_poblacion_exit_1(self):
        for extra in ([det("sX", "sX-m0")], [det("s0", "s0-ajeno")]):
            if os.path.exists(self.res):
                os.remove(self.res)
            r = self.resultado(self.dets(1) + extra)
            self.assertEqual(r.returncode, 1, extra)
            self.assertIn("población", r.stderr)
            self.assertFalse(os.path.exists(self.res))

    def test_detecciones_incompletas_es_incompleto(self):
        v = {"juez_valido": True}
        self.resultado(self.dets(0)[:7], validacion=v)
        o = self.lee(self.res)
        self.assertEqual(o["puerta"]["estado"], "incompleto")
        self.assertEqual((o["puerta"]["n"], o["n_detecciones_faltantes"]), (10, 3))
        # Contraparte: completo, y un sin_veredicto cuenta como cubierto.
        self.resultado(self.dets(0)[:9] + [det("s9", "s9-m0", sin=True)], validacion=v)
        self.assertEqual(self.lee(self.res)["puerta"]["estado"], "estabilizar")

    def test_desde_del_sello(self):
        # Una sesión anterior al default solo entra si el sello se hizo con --desde anterior.
        self.escribe("vieja", sesion("vieja", ts="2026-10-01T10:00:00Z"))
        self.sella("--forzar")
        self.assertEqual(self.lee(self.sello)["n_sesiones"], 10)
        self.sella("--forzar", "--desde", "2026-09-01")
        self.assertEqual(self.lee(self.sello)["n_sesiones"], 11)
        r = self.resultado(self.dets(1))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(self.lee(self.res)["puerta"]["n"], 11)
        self.assertEqual(self.lee(self.res)["desde"], "2026-09-01")


if __name__ == "__main__":
    unittest.main()
