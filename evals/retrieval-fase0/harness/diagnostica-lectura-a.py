#!/usr/bin/env python3
"""Diagnóstica de lectura A (M2-07, spec fusión §4.5/§5.1, blindspot B4).

`busca_hybrid` implementa SOLO la lectura B (admisión = unión, D-f2); la
lectura A (admisión = gate FTS: solo entra al ranking una entidad presente
en los resultados FTS) NO tiene code-path propio en el engine — B4 lo
prohíbe explícitamente ("NO añadas un code-path ni un flag para A"). Esta
lectura se produce POST-HOC, combinando dos corridas YA capturadas contra
la MISMA DB:

1. La corrida hybrid del centro del grid (bonus/β centrales, `--min-similitud
   0.0 --limite 10`) — YA fusionada, YA ordenada por score desc (lectura B).
2. Una corrida FTS fresca a K_c=50 (`--tipo fts --limite 50`) — el MISMO
   candidatos-FTS que `busca_hybrid` usa internamente (spec §4.2) — para
   saber, por query, qué permalinks tienen candidato FTS.

Lectura A = filtrar la lista fusionada de (1) a los permalinks presentes en
(2) ANTES de truncar a top-5 (el vector solo re-puntúa dentro del conjunto
admitido, §4.5) — nunca reordena por sí sola, solo restringe la admisión.
NO es un code-path nuevo del binario: es post-procesado en Python sobre dos
corridas reales, exactamente lo que B4 exige.

Uso: diagnostica-lectura-a.py <arm-hybrid-centro> <arm-fts-k50>
"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from analyze import BASE, K, hit, load, norm  # noqa: E402


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    arm_hybrid, arm_fts50 = sys.argv[1], sys.argv[2]

    gold = {}
    for l in open(BASE / "eval.jsonl"):
        g = json.loads(l)
        g["expected_permalink"] = norm(g["expected_permalink"])
        if g["expected_permalink"]:
            gold[g["query"]] = g

    hybrid, _ = load(arm_hybrid)
    fts50, _ = load(arm_fts50)

    hits = 0
    misses = []
    for q, g in sorted(gold.items()):
        exp = g["expected_permalink"]
        candidatos_fts = {r["permalink"] for r in fts50.get(q, {}).get("text", [])}
        fusionado = hybrid.get(q, {}).get("hybrid", [])
        admitidos_a = [r for r in fusionado if r["permalink"] in candidatos_fts]
        top5_a = admitidos_a[:K]
        h = hit(top5_a, exp)
        if h:
            hits += 1
        else:
            misses.append(q)

    print(f"# diagnóstica lectura A — {arm_hybrid} gateado por FTS de {arm_fts50}")
    print(f"hit@{K} lectura A: {hits}/{len(gold)}")
    print(f"predicción pre-registrada (spec fusión §4.5): 28-41/55")
    print(f"\n## misses de lectura A ({len(misses)}):")
    for q in misses:
        print(f"- {q}")


if __name__ == "__main__":
    main()
