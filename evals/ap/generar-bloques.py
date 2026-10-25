#!/usr/bin/env python3
"""generar-bloques.py --kb <KB> [--sondas-m <preregistro de M>] — bloques de reglas de Ap (#190), deterministas.

Para cada una de las 40 sondas de M (ids en la línea 39 de evals/campana-m/preregistro.md; proyecto en
~/.cache/exo-campana-m/redaccion/reglas-g*.jsonl, texto gold en ~/.cache/exo-campana-m/reglas/<id>.txt):
repo git vacío con basename = proyecto, `exo rules --kb KB --cwd <repo> --json`, y el bloque desplegado
(globales + proyecto) con el render de techo-reglas-3 (= bloque() de plugins/exo/hooks/register.ts), importado.
Escribe $K_ROOT/ap/bloques/<id>/ap.txt y, solo para las de proyecto cubiertas (la gold está literal entre las
reglas de proyecto devueltas), ap1.txt (la gold sola en su lista). Versionado en evals/ap/, sin textos de reglas:
cobertura.tsv, bloques.sha256 (rutas relativas a $K_ROOT) y tareas.tsv (réplicas K = evals/ap/k.txt).
Columnas de cobertura.tsv: pos = posición 0-based de la gold en su lista; sim_max = máx difflib ratio de la gold
con las demás reglas del bloque; en_global = la gold (transversal) está entre las globales (no se cubre por esta vía).
"""
import csv, difflib, hashlib, importlib.util, json, os, re, subprocess, sys, tempfile

H = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(H, "..", ".."))
M = os.path.expanduser("~/.cache/exo-campana-m")
K_ROOT = os.path.expanduser(os.environ.get("K_ROOT", "~/.cache/exo-ablacion-k"))
ALIAS = {"cge": "code-graph-engine"}  # el basename del repo, no el nombre corto del pool

_s = importlib.util.spec_from_file_location("gb3", os.path.join(ROOT, "evals/techo-reglas-3/generar-bloques.py"))
gb3 = importlib.util.module_from_spec(_s); _s.loader.exec_module(gb3)


def sondas():
    ids = re.findall(r"g\d-\d+", open(os.path.join(ROOT, "evals/campana-m/preregistro.md")).read().splitlines()[38])
    proy = {}
    for i in range(3):
        for l in open(f"{M}/redaccion/reglas-g{i}.jsonl"):
            if l.strip():
                r = json.loads(l); proy[r["id"]] = r["proyecto"]
    if len(ids) != 40 or len(set(ids)) != 40:
        sys.exit(f"generar-bloques: se esperaban 40 ids en la línea 39, hay {len(ids)}")
    return [(i, proy[i]) for i in ids]


def reglas_de(kb, nombre):
    with tempfile.TemporaryDirectory() as t:
        d = os.path.join(t, ALIAS.get(nombre, nombre)); os.mkdir(d)
        subprocess.run(["git", "-C", d, "init", "-q"], check=True)
        p = subprocess.run(["exo", "rules", "--kb", kb, "--cwd", d, "--json"], capture_output=True, text=True)
        if p.returncode != 0:
            sys.exit(f"generar-bloques: exo rules falla para {nombre}: {p.stderr.strip()}")
        return json.loads(p.stdout)["data"]


def sha(p):
    return hashlib.sha256(open(p, "rb").read()).hexdigest()


def main():
    if "--kb" not in sys.argv:
        sys.exit(__doc__)
    kb = os.path.abspath(os.path.expanduser(sys.argv[sys.argv.index("--kb") + 1]))
    k = int(open(os.path.join(H, "k.txt")).read())
    commit = subprocess.run(["git", "-C", kb, "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip() or "sin-git"
    filas, shas, tareas = [], [], []
    for sid, proyecto in sondas():
        gold = open(f"{M}/reglas/{sid}.txt").read().rstrip("\n")
        d = reglas_de(kb, proyecto)
        glob, proy = d.get("global", []), d.get("rules", [])
        transversal = proyecto == "transversal"
        cubierta = (not transversal) and gold in proy
        lista = glob if transversal else proy
        pos = lista.index(gold) if gold in lista else ""
        otras = [r for r in glob + proy if r != gold]
        sim = max((difflib.SequenceMatcher(None, gold, r).ratio() for r in otras), default=0.0)
        bl = gb3.bloque([r.replace("\n", " ") for r in glob], [r.replace("\n", " ") for r in proy])
        out = os.path.join(K_ROOT, "ap", "bloques", sid); os.makedirs(out, exist_ok=True)
        for nombre, txt in (("ap", bl),) + ((("ap1", gb3.bloque([], [gold])),) if cubierta else ()):
            open(os.path.join(out, f"{nombre}.txt"), "w").write(txt)
            shas.append(f"{sha(os.path.join(out, nombre + '.txt'))}  ap/bloques/{sid}/{nombre}.txt")
        filas.append([sid, proyecto, "transversal" if transversal else "proyecto", "si" if cubierta else "no",
                      d.get("status", ""), d.get("reason", ""), len(glob), len(proy), len(bl.encode()), pos,
                      f"{sim:.3f}", ("si" if gold in glob else "no") if transversal else ""])
        tareas.append([sid, "transversal" if transversal else "proyecto", "{K_ROOT}/ap/bloques/%s/{brazo}.txt" % sid,
                       f"ap:{k},ap1:{k}" if cubierta else f"ap:{k}"])
    with open(os.path.join(H, "cobertura.tsv"), "w", newline="") as o:
        o.write(f"# kb={kb} commit={commit}\n")
        w = csv.writer(o, delimiter="\t", lineterminator="\n")
        w.writerow("id proyecto ambito cubierta status reason n_global n_proyecto bytes pos sim_max en_global".split())
        w.writerows(filas)
    open(os.path.join(H, "bloques.sha256"), "w").write("\n".join(shas) + "\n")
    with open(os.path.join(H, "tareas.tsv"), "w", newline="") as o:
        csv.writer(o, delimiter="\t", lineterminator="\n").writerows(tareas)
    n = sum(f[3] == "si" for f in filas)
    print(f"kb {commit[:7]}: {len(filas)} sondas, {n} de proyecto cubiertas (N={n}), "
          f"{sum(f[2] == 'proyecto' for f in filas)} de proyecto, {sum(f[2] == 'transversal' for f in filas)} transversales")


if __name__ == "__main__":
    main()
