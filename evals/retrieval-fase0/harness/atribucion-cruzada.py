#!/usr/bin/env python3
"""Atribución cruzada de misses del arm hybrid contra los arms fts/vector
(M2-07, spec fusión §5.2/§6 — sanity-check + diagnóstica). Complemento de
`analyze.py` — NO lo modifica, lo importa como librería.

Por qué hace falta: `analyze.py <arm>` clasifica cada miss mirando, DENTRO
del mismo fichero `results/<arm>.jsonl`, las claves de `search_type` que esa
fila trae. Eso funciona para el arm bm (su API real devuelve breakdown
text/vector/hybrid en una sola respuesta), pero NO para los arms del engine:
`replay-engine.py` invoca `exo search --type <tipo>` una vez por query, así
que `results/engine-hybrid*.jsonl` solo trae la clave "hybrid" — nunca
"text"/"vector" en la misma fila. Analizado solo con `analyze.py`, la
sección "atribución de misses" del arm hybrid siempre clasifica todo como
both-miss (las claves ausentes nunca dan HIT). Este script cruza el arm
hybrid contra los arms `engine-fts`/`engine-vector` YA corridos el mismo día
sobre la MISMA DB, query por query, para clasificar de verdad cada miss y,
en particular, listar qué hits vectoriales puros se pierden en la fusión
(el dato que exige el sanity-check §6 cuando engine-hybrid < 46/55).

Uso: atribucion-cruzada.py <arm-hybrid> <arm-fts> <arm-vector>
"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from analyze import BASE, K, hit, load, norm  # noqa: E402


def main():
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    arm_hybrid, arm_fts, arm_vector = sys.argv[1], sys.argv[2], sys.argv[3]

    gold = {}
    for l in open(BASE / "eval.jsonl"):
        g = json.loads(l)
        g["expected_permalink"] = norm(g["expected_permalink"])
        if g["expected_permalink"]:
            gold[g["query"]] = g

    hybrid, _ = load(arm_hybrid)
    fts, _ = load(arm_fts)
    vector, _ = load(arm_vector)

    hits_hybrid = 0
    perdidos_de_vector = []
    perdidos_de_fts = []
    both_miss = []
    for q, g in sorted(gold.items()):
        exp = g["expected_permalink"]
        h = hit(hybrid.get(q, {}).get("hybrid", []), exp)
        t = hit(fts.get(q, {}).get("text", []), exp)
        v = hit(vector.get(q, {}).get("vector", []), exp)
        if h:
            hits_hybrid += 1
            continue
        if v:
            perdidos_de_vector.append(q)
        elif t:
            perdidos_de_fts.append(q)
        else:
            both_miss.append(q)

    print(f"# atribución cruzada — {arm_hybrid} vs {arm_fts}/{arm_vector}")
    print(f"hybrid hit@{K}: {hits_hybrid}/{len(gold)}")
    print(f"\n## misses que SÍ eran vector-HIT puro (fusión perdió un hit vectorial): {len(perdidos_de_vector)}")
    for q in perdidos_de_vector:
        print(f"- {q}")
    print(f"\n## misses que SÍ eran fts-HIT puro (fusión perdió un hit fts): {len(perdidos_de_fts)}")
    for q in perdidos_de_fts:
        print(f"- {q}")
    print(f"\n## both-miss (ni fts ni vector puro acertaban): {len(both_miss)}")
    for q in both_miss:
        print(f"- {q}")


if __name__ == "__main__":
    main()
