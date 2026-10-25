#!/usr/bin/env python3
"""generar-orden.py → el ORDEN barajado de Ap (semilla 20261025, igual que techo-reglas-3) sobre evals/ap/tareas.tsv.
Comando: python3 evals/ap/generar-orden.py > evals/ap/orden.txt
El preregistro está sellado, así que el orden va aparte; correr-techo.sh lo lee de evals/ap/orden.txt."""
import csv, os, random

H = os.path.dirname(os.path.abspath(__file__))
L = []
for i, _, _, b in csv.reader(open(os.path.join(H, "tareas.tsv")), delimiter="\t"):
    for x in b.split(","):
        a, k = x.split(":")
        L += [(a, i, r) for r in range(1, int(k) + 1)]
L.sort()
random.Random(20261025).shuffle(L)
print("# Orden de corridas de Ap (fuera del preregistro sellado). Generado con:")
print("#   python3 evals/ap/generar-orden.py > evals/ap/orden.txt   (semilla 20261025, tareas.tsv)")
print("<!-- ORDEN-BEGIN -->")
print("\n".join(f"{a} {i} {r}" for a, i, r in L))
print("<!-- ORDEN-END -->")
