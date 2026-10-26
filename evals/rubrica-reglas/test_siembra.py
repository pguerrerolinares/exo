"""Tests de siembra-reglas.py (orden viejo y --etiquetas) con datos sintéticos inventados."""
import json, os, subprocess, sys, tempfile, unittest

H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, H)
import rubrica as rb

SCRIPT = os.path.join(H, "..", "..", "scripts", "siembra-reglas.py")
NOTA_EXO = "exo — framework unificado de trabajo agéntico.md"


def regla(id, proyecto="exo", ambito="codigo", check=None, nota="projects/stable.md", generica=False):
    return {"id": id, "proyecto": proyecto, "ambito": ambito, "generica": generica, "check": check,
            "nota": nota, "cita": f"cita {id}", "regla": f"regla inventada {id}"}


def escribe(ruta, texto):
    os.makedirs(os.path.dirname(ruta), exist_ok=True)
    with open(ruta, "w") as f:
        f.write(texto)


def monta(tmp, reglas, rcs=None):
    pool, criba, kb = (os.path.join(tmp, d) for d in ("pool", "criba", "kb"))
    escribe(os.path.join(kb, "core", "nota-core.md"), "---\ntier: core\n---\n")
    escribe(os.path.join(kb, "projects", "stable.md"), "---\ntier: stable\n---\n")
    escribe(os.path.join(pool, "reglas-g0.jsonl"), "".join(json.dumps(r) + "\n" for r in reglas))
    for id, rc in (rcs or {}).items():
        escribe(os.path.join(criba, id, "a0-r1", "check.rc"), f"{rc}\n")
    return pool, criba, kb


def corre(pool, criba, kb, *extra):
    r = subprocess.run([sys.executable, "-I", SCRIPT, pool, criba, kb, "--json", *extra],
                       capture_output=True, text=True, stdin=subprocess.DEVNULL)
    assert r.returncode == 0, r.stderr
    return json.loads(r.stdout)


def reglas_viejo():
    return [
        regla("g0-1", ambito="dominio-proyecto"),
        regla("g0-2", check="ls", nota="core/nota-core.md"),
        regla("g0-3", ambito="git"),
        regla("g0-4", ambito="git", nota="core/nota-core.md"),
        regla("g0-10", check="ls"),
        regla("g0-5", proyecto="pguerrero-music", ambito="git"),
        regla("g0-6", proyecto="pguerrero-music", check="ls"),
        regla("g0-7", proyecto="CV"),
        regla("g0-8", proyecto="transversal"),
        regla("g0-9", generica=True),
        regla("g0-11", ambito="conducta"),
    ]


RCS_VIEJO = {"g0-2": 0, "g0-3": 1, "g0-5": 0}

FIXTURE = json.loads(r'''
{
 "repos": [
  {
   "repo": "exo",
   "nota": "exo — framework unificado de trabajo agéntico.md",
   "candidatas": 5,
   "desborda": false,
   "reglas": [
    {
     "id": "g0-10",
     "a0_cumple": false,
     "ambito": "codigo",
     "cita": "cita g0-10",
     "regla": "regla inventada g0-10",
     "entra": true
    },
    {
     "id": "g0-4",
     "a0_cumple": false,
     "ambito": "git",
     "cita": "cita g0-4",
     "regla": "regla inventada g0-4",
     "entra": true
    },
    {
     "id": "g0-3",
     "a0_cumple": false,
     "ambito": "git",
     "cita": "cita g0-3",
     "regla": "regla inventada g0-3",
     "entra": true
    },
    {
     "id": "g0-1",
     "a0_cumple": false,
     "ambito": "dominio-proyecto",
     "cita": "cita g0-1",
     "regla": "regla inventada g0-1",
     "entra": true
    },
    {
     "id": "g0-2",
     "a0_cumple": true,
     "ambito": "codigo",
     "cita": "cita g0-2",
     "regla": "regla inventada g0-2",
     "entra": true
    }
   ]
  },
  {
   "repo": "pguerrero-music",
   "nota": "pguerrero-music.md",
   "candidatas": 2,
   "desborda": false,
   "reglas": [
    {
     "id": "g0-6",
     "a0_cumple": false,
     "ambito": "codigo",
     "cita": "cita g0-6",
     "regla": "regla inventada g0-6",
     "entra": true
    },
    {
     "id": "g0-5",
     "a0_cumple": true,
     "ambito": "git",
     "cita": "cita g0-5",
     "regla": "regla inventada g0-5",
     "entra": true
    }
   ]
  }
 ],
 "sin_repo": {
  "CV": [
   "g0-7"
  ]
 }
}
''')


class TestSiembra(unittest.TestCase):
    def test_sin_etiquetas_no_cambia(self):
        with tempfile.TemporaryDirectory() as tmp:
            sal = corre(*monta(tmp, reglas_viejo(), RCS_VIEJO))
        self.assertEqual(sal, FIXTURE)

    def test_separador_unicode_dentro_de_json_no_parte_la_linea(self):
        r = regla("g0-1")
        r["regla"] = "texto con \u2028 y \x85 inventados"
        with tempfile.TemporaryDirectory() as tmp:
            pool, criba, kb = monta(tmp, [])
            escribe(os.path.join(pool, "reglas-g0.jsonl"), json.dumps(r, ensure_ascii=False) + "\n")
            sal = corre(pool, criba, kb)
        self.assertEqual(sal["repos"][0]["reglas"][0]["regla"], r["regla"])

    def test_con_etiquetas_ordena_por_rubrica(self):
        # g0-1/g0-2: orden viejo X(g0-2: check, core) antes que Y(g0-1); la rúbrica pone Y(P) antes que X(E)
        # g0-3 (sin corrida, E, alto) vs g0-4 (no cumple, E->D, bajo): la rúbrica pone g0-4 antes
        rs = [regla("g0-1", ambito="dominio-proyecto"),
              regla("g0-2", ambito="git", check="ls", nota="core/nota-core.md"),
              regla("g0-3"), regla("g0-4")]
        rs += [regla(f"g0-{n}") for n in range(5, 13)]
        etq = {"g0-1": ("P", "bajo"), "g0-2": ("E", "alto"), "g0-3": ("E", "alto"), "g0-4": ("E", "bajo")}
        etq.update({f"g0-{n}": ("E", "medio") for n in range(5, 13)})
        with tempfile.TemporaryDirectory() as tmp:
            pool, criba, kb = monta(tmp, rs, {"g0-4": 1})
            jl = os.path.join(tmp, "etq.jsonl")
            escribe(jl, "".join(json.dumps({"id": i, "choque": c, "coste": k}) + "\n" for i, (c, k) in etq.items()))
            sal = corre(pool, criba, kb, "--etiquetas", jl)
        reglas = sal["repos"][0]["reglas"]
        orden = [r["id"] for r in reglas]
        self.assertEqual(len(orden), 12)
        self.assertLess(orden.index("g0-1"), orden.index("g0-2"))
        self.assertLess(orden.index("g0-4"), orden.index("g0-3"))
        esperado = [i["id"] for i in rb.orden(
            [{"id": r["id"], "etiqueta": {"id": r["id"], "choque": etq[r["id"]][0], "coste": etq[r["id"]][1]},
              "criba": False if r["id"] == "g0-4" else None} for r in reglas])]
        self.assertEqual(orden, esperado)
        self.assertEqual([r["entra"] for r in reglas], [True] * 10 + [False] * 2)

    def test_etiqueta_ausente(self):
        rs = [regla("g0-1"), regla("g0-2"), regla("g0-3")]
        with tempfile.TemporaryDirectory() as tmp:
            pool, criba, kb = monta(tmp, rs)
            jl = os.path.join(tmp, "etq.jsonl")
            escribe(jl, json.dumps({"id": "g0-3", "choque": "E", "coste": "alto"}) + "\n\n"
                    + json.dumps({"id": "g0-2", "choque": "E", "coste": "alto"}) + "\n")
            sal = corre(pool, criba, kb, "--etiquetas", jl)
        self.assertEqual([r["id"] for r in sal["repos"][0]["reglas"]], ["g0-2", "g0-3", "g0-1"])

    def test_etiqueta_invalida_falla(self):
        with tempfile.TemporaryDirectory() as tmp:
            pool, criba, kb = monta(tmp, [regla("g0-1")])
            jl = os.path.join(tmp, "etq.jsonl")
            escribe(jl, json.dumps({"id": "g0-1", "choque": "Z", "coste": "alto"}) + "\n")
            r = subprocess.run([sys.executable, "-I", SCRIPT, pool, criba, kb, "--json", "--etiquetas", jl],
                               capture_output=True, text=True, stdin=subprocess.DEVNULL)
        self.assertNotEqual(r.returncode, 0)


if __name__ == "__main__":
    unittest.main()
