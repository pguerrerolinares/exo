#!/usr/bin/env python3
"""Eval rápido de retrieval para la serie B (orientación, no gate).

Corre `exo search` sobre un gold (formato gold-j: id, query, source,
expected_permalink, acceptable_permalinks) con uno o más brazos y saca
hit@5 / MRR@10 por estrato y, en las filas negativas, cuántas devuelven
algo (lo ideal en un negativo es devolver nada).

Uso:
  rapido.py --gold G --db D --brazo nombre=BIN:tipo[:min_sim] [...] [--jobs 4]
  p. ej. --brazo base-fts=/ruta/exo:fts --brazo b1-hyb=/ruta/exo:hybrid:0.40
"""
import argparse
import collections
import json
import subprocess
from concurrent.futures import ThreadPoolExecutor


def busca(binario, db, query, tipo, min_sim):
    cmd = [binario, "search", "--db", db, "--type", tipo, "--limit", "10", "--json"]
    if min_sim is not None:
        cmd += ["--min-similarity", min_sim]
    cmd.append(query)
    r = subprocess.run(cmd, capture_output=True, text=True)
    try:
        return [x["permalink"] for x in json.loads(r.stdout)["data"]["results"]]
    except (json.JSONDecodeError, KeyError, TypeError):
        return None


def metricas(filas, rankings):
    por = collections.defaultdict(lambda: {"n": 0, "hit5": 0, "rr": 0.0, "con_algo": 0})
    for f, rk in zip(filas, rankings):
        e = por[f["source"]]
        e["n"] += 1
        rk = rk or []
        if f["expected_permalink"] is None:
            e["con_algo"] += bool(rk)
            continue
        rel = {f["expected_permalink"], *f.get("acceptable_permalinks", [])}
        e["hit5"] += any(p in rel for p in rk[:5])
        e["rr"] += next((1 / i for i, p in enumerate(rk[:10], 1) if p in rel), 0.0)
    return por


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--gold", required=True)
    ap.add_argument("--db", required=True)
    ap.add_argument("--brazo", action="append", required=True)
    ap.add_argument("--jobs", type=int, default=4)
    a = ap.parse_args()
    filas = [json.loads(l) for l in open(a.gold, encoding="utf-8") if l.strip()]
    for spec in a.brazo:
        nombre, resto = spec.split("=", 1)
        partes = resto.split(":")
        binario, tipo = partes[0], partes[1]
        min_sim = partes[2] if len(partes) > 2 else None
        with ThreadPoolExecutor(a.jobs) as ex:
            rks = list(ex.map(lambda f: busca(binario, a.db, f["query"], tipo, min_sim), filas))
        errores = sum(r is None for r in rks)
        por = metricas(filas, rks)
        pos = [v for k, v in por.items() if k != "negativo"]
        n = sum(v["n"] for v in pos)
        h = sum(v["hit5"] for v in pos)
        rr = sum(v["rr"] for v in pos)
        neg = por.get("negativo", {"n": 0, "con_algo": 0})
        estratos = " ".join(f"{k}={v['hit5']}/{v['n']}" for k, v in sorted(por.items()) if k != "negativo")
        print(f"{nombre:14} hit@5 {h}/{n} MRR@10 {rr / max(n, 1):.3f} | {estratos} | "
              f"negativos con resultado {neg['con_algo']}/{neg['n']} | errores {errores}", flush=True)


if __name__ == "__main__":
    main()
