#!/usr/bin/env python3
"""generar-bloques.py — bloques de la mini-tanda techo-reglas-3b: g0-33 con 3, 5 o 10 globales.

Reutiliza las colas, el render y los descartes de techo-reglas-3 (misma semilla, mismo pool). La lista «del
dueño» lleva N globales con la gold incluida, en la posición min(4, N-1) (4 es la de g0-33 en arp20). La lista
«del proyecto» son las mismas 10 distractoras de arp20. Brazos arp9NN: 9 = mini-tanda, NN = nº de globales.
arp910 es byte a byte el arp20 de g0-33 en techo-reglas-3. Escribe bloques/g0-33/arp9NN.txt (no versionado) y
bloques.sha256.
"""
import hashlib, importlib.util, os, sys

H = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("gb3", os.path.join(H, "..", "techo-reglas-3", "generar-bloques.py"))
gb3 = importlib.util.module_from_spec(spec); spec.loader.exec_module(gb3)
TID = "g0-33"


def main():
    pool = gb3.carga_pool()
    descartes = set()
    for f in open(os.path.join(gb3.H, "descartes.tsv")):
        f = f.rstrip("\n").split("\t")
        if f and f[0] and not f[0].startswith("#"):
            descartes |= {(t, f[1]) for t in (gb3.SUELO if f[0] == "*" else [f[0]])}
    g, p, _ = gb3.colas(pool, TID, descartes)
    d = os.path.join(H, "bloques", TID); os.makedirs(d, exist_ok=True)
    shas = []
    for n in (3, 5, 10):
        k = min(4, n - 1)
        dueño = g[:n - 1][:k] + [TID] + g[:n - 1][k:]
        txt = gb3.bloque([pool[i]["regla"] for i in dueño], [pool[i]["regla"] for i in p[:10]])
        f = os.path.join(d, f"arp9{n:02d}.txt"); open(f, "w").write(txt)
        shas.append(f"{hashlib.sha256(txt.encode()).hexdigest()}  bloques/{TID}/arp9{n:02d}.txt")
    open(os.path.join(H, "bloques.sha256"), "w").write("\n".join(shas) + "\n")
    print("\n".join(shas))


if __name__ == "__main__":
    sys.exit(main())
