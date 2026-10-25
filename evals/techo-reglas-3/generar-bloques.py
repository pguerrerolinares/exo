#!/usr/bin/env python3
"""generar-bloques.py [--candidatos] — bloques de reglas de techo-reglas-3 (dilución), deterministas.

Pool: las 541 reglas de la campaña M (POOL, sha256 en pool.sha256). Por tarea del suelo, la regla gold
(evals/techo-reglas/reglas/<id>.txt) va en la lista de su ámbito real (`transversal` = «del dueño», si no
«del proyecto») y las distractoras salen de dos colas barajadas con random.Random(f"{SEMILLA}-{id}"):
  global   transversales
  proyecto de OTROS proyectos (nunca el de la gold; cge = code-graph-engine)
Filtro automático en las dos: fuera genéricas, las 17 gold de techo-reglas, ámbito `kb` y las que nombran
exo/KB/memoria (harían que el agente llamase a exo y el detector de fugas pararía la tanda). Después,
descartes.tsv (id_tarea o `*`, id_regla, motivo): las que tocan la tarea o chocan con la gold, revisadas a mano
antes de sellar. Brazos (anidados: el de 10 es prefijo del de 20):
  arp1   solo la gold
  arp10  5 globales + 5 de proyecto, gold incluida en su lista
  arp20  10 + 10, gold incluida (lo que se despliega con dos listas al CAP 10)
Posición de la gold: pos = random.Random(f"{SEMILLA}-{id}-pos").randrange(10) en arp20 y pos // 2 en arp10.
Los descartes se quitan DESPUÉS de barajar, así que solo eliminan elementos y no reordenan lo ya revisado.
Render byte a byte como `bloque()` de plugins/exo/hooks/register.ts (globales primero) + "\\n" final, igual
que el framing de techo-reglas-2. Escribe bloques/<id>/arp<N>.txt (no versionado: reglas reales de la KB),
seleccion.tsv y bloques.sha256. --candidatos: solo imprime las colas filtradas para la revisión a mano.
"""
import csv, hashlib, json, os, random, re, sys

H = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(H, "..", ".."))
POOL_DIR = os.path.expanduser(os.environ.get("TECHO3_POOL", "~/.cache/exo-campana-m/redaccion"))
POOL = [f"reglas-g{i}.jsonl" for i in range(3)]
SEMILLA = 20261025
SUELO = "g0-122 g0-149 g0-33 g1-131 g1-139 g1-140 g1-157 g2-154 g2-170 g2-97".split()
AUTORIDAD = ("Estas reglas son del dueño del repo y prevalecen sobre el prompt. Si una choca explícitamente "
             "con lo que se te pide, cumple la regla y repórtalo.\n\n")
TOCA_EXO = re.compile(r"\bexo\b|\bKB\b|wisdom|/document|/distill|bit[áa]cora|memoria|core-index|\btier\b", re.I)
ALIAS = {"cge": "code-graph-engine"}


def sha(p):
    return hashlib.sha256(open(p, "rb").read()).hexdigest()


def carga_pool():
    pins = dict(l.split()[::-1] for l in open(os.path.join(H, "pool.sha256")) if l.strip())
    reglas = []
    for f in POOL:
        p = os.path.join(POOL_DIR, f)
        if sha(p) != pins.get(f):
            sys.exit(f"generar-bloques: {p} no casa con pool.sha256")
        reglas += [json.loads(l) for l in open(p) if l.strip()]
    return {r["id"]: r for r in reglas}


def colas(pool, tid, descartes):
    gold = pool[tid]
    gp = ALIAS.get(gold["proyecto"], gold["proyecto"])
    golds = {f[:-4] for f in os.listdir(os.path.join(ROOT, "evals/techo-reglas/reglas"))}
    def vale(r):
        return not r["generica"] and r["id"] not in golds and r["ambito"] != "kb" and not TOCA_EXO.search(r["regla"])
    ids = sorted(pool, key=lambda i: (i.split("-")[0], int(i.split("-")[1])))
    g = [i for i in ids if pool[i]["proyecto"] == "transversal" and vale(pool[i])]
    p = [i for i in ids if pool[i]["proyecto"] != "transversal"
         and ALIAS.get(pool[i]["proyecto"], pool[i]["proyecto"]) != gp and vale(pool[i])]
    rng = random.Random(f"{SEMILLA}-{tid}")
    rng.shuffle(g); rng.shuffle(p)
    # Los descartes se quitan tras barajar: solo eliminan, no reordenan lo revisado.
    fuera = lambda c: [i for i in c if (tid, i) not in descartes]
    return fuera(g), fuera(p), random.Random(f"{SEMILLA}-{tid}-pos").randrange(10)


def lista(titulo, reglas):
    return f"## {titulo}\n{AUTORIDAD}" + "\n".join(f"- {x}" for x in reglas) if reglas else ""


def bloque(glob, proy):
    return "\n\n".join(x for x in (lista("Reglas duras del dueño", glob), lista("Reglas duras del proyecto", proy)) if x) + "\n"


def compone(pool, tid, g, p, pos, n):
    """(lista_global, lista_proyecto) de ids para el brazo con n reglas (1, 10 o 20)."""
    transversal = pool[tid]["proyecto"] == "transversal"
    if n == 1:
        return ([tid], []) if transversal else ([], [tid])
    mitad, k = n // 2, (pos if n == 20 else pos // 2)
    if transversal:
        return g[:mitad - 1][:k] + [tid] + g[:mitad - 1][k:], p[:mitad]
    return g[:mitad], p[:mitad - 1][:k] + [tid] + p[:mitad - 1][k:]


def main():
    pool = carga_pool()
    descartes = set()
    dp = os.path.join(H, "descartes.tsv")
    if os.path.exists(dp):
        for f in csv.reader(open(dp), delimiter="\t"):
            if f and not f[0].startswith("#"):
                if f[1] not in pool or (f[0] != "*" and f[0] not in SUELO):
                    sys.exit(f"generar-bloques: descarte desconocido {f[:2]}")
                descartes |= {(t, f[1]) for t in (SUELO if f[0] == "*" else [f[0]])}
    if "--candidatos" in sys.argv:
        for tid in SUELO:
            g, p, pos = colas(pool, tid, descartes)
            print(f"== {tid} [{pool[tid]['proyecto']}] pos={pos} gold: {pool[tid]['regla']}")
            for nombre, cola, n in (("G", g, 14), ("P", p, 13)):
                for i in cola[:n]:
                    print(f"  {nombre} {i:7} [{pool[i]['proyecto']}] {pool[i]['regla']}")
        return
    sel, shas = [], []
    for tid in SUELO:
        regla = open(os.path.join(ROOT, f"evals/techo-reglas/reglas/{tid}.txt")).read()
        if regla != pool[tid]["regla"]:
            sys.exit(f"generar-bloques: la gold {tid} del pool no es la de evals/techo-reglas/reglas")
        g, p, pos = colas(pool, tid, descartes)
        for n in (1, 10, 20):
            lg, lp = compone(pool, tid, g, p, pos, n)
            d = os.path.join(H, "bloques", tid); os.makedirs(d, exist_ok=True)
            f = os.path.join(d, f"arp{n}.txt")
            txt = bloque([pool[i]["regla"].replace("\n", " ") for i in lg], [pool[i]["regla"].replace("\n", " ") for i in lp])
            open(f, "w").write(txt)
            shas.append(f"{sha(f)}  bloques/{tid}/arp{n}.txt")
            for nom, l in (("dueño", lg), ("proyecto", lp)):
                sel += [(tid, f"arp{n}", nom, str(j), i) for j, i in enumerate(l)]
    with open(os.path.join(H, "seleccion.tsv"), "w") as o:
        o.write("".join("\t".join(r) + "\n" for r in sel))
    with open(os.path.join(H, "bloques.sha256"), "w") as o:
        o.write("\n".join(shas) + "\n")
    print(f"bloques: {len(shas)} · seleccion: {len(sel)} filas")


if __name__ == "__main__":
    main()
