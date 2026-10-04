#!/usr/bin/env python3
"""Simulación offline de operadores de fusión (serie B, paso 2; orientación).

Captura una vez, por query, la lista FTS (limit 50) y la vector (limit 50,
min-similarity 0) de un binario, y compara operadores sin tocar el engine:
  vector     solo vector, umbral U
  combmax    max(v, beta·f/fmax)            (el sellado de hoy, bonus 0)
  combsum    v + beta·f/fmax
  rrf        sum 1/(k + rank) sobre los dos canales
Vector siempre filtrado por el umbral U (como el engine).

Uso: fusion.py --gold G --db D --exo BIN --cache C.jsonl [--jobs 6]
"""
import argparse
import json
import os
import subprocess
from concurrent.futures import ThreadPoolExecutor

from rapido import metricas

U, BETA, RRF_K = 0.40, 0.6, 60


def lista(binario, db, query, tipo):
    cmd = [binario, "search", "--db", db, "--type", tipo, "--limit", "50", "--json"]
    if tipo == "vector":
        cmd += ["--min-similarity", "0"]
    r = subprocess.run(cmd + [query], capture_output=True, text=True)
    return [[x["permalink"], x["score"]] for x in json.loads(r.stdout)["data"]["results"]]


def captura(a, filas):
    if os.path.exists(a.cache):
        return [json.loads(l) for l in open(a.cache, encoding="utf-8")]

    def una(f):
        return {"id": f["id"], "fts": lista(a.exo, a.db, f["query"], "fts"),
                "vector": lista(a.exo, a.db, f["query"], "vector")}

    with ThreadPoolExecutor(a.jobs) as ex:
        caps = list(ex.map(una, filas))
    with open(a.cache, "w", encoding="utf-8") as fh:
        for c in caps:
            fh.write(json.dumps(c, ensure_ascii=False) + "\n")
    return caps


def fusiona(cap, op):
    v = {p: s for p, s in cap["vector"] if s >= U}
    fts = cap["fts"]
    fmax = max((s for _, s in fts), default=0) or 1
    f = {p: BETA * s / fmax for p, s in fts}
    if op == "vector":
        sc = v
    elif op == "combmax":
        sc = {p: max(v.get(p, 0), f.get(p, 0)) for p in v.keys() | f.keys()}
    elif op == "combsum":
        sc = {p: v.get(p, 0) + f.get(p, 0) for p in v.keys() | f.keys()}
    elif op == "rrf":
        sc = {}
        for canal in (sorted(v, key=v.get, reverse=True), [p for p, _ in fts]):
            for i, p in enumerate(canal, 1):
                sc[p] = sc.get(p, 0) + 1 / (RRF_K + i)
    return [p for p, _ in sorted(sc.items(), key=lambda x: (-x[1], x[0]))][:10]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--gold", required=True)
    ap.add_argument("--db", required=True)
    ap.add_argument("--exo", required=True)
    ap.add_argument("--cache", required=True)
    ap.add_argument("--jobs", type=int, default=6)
    a = ap.parse_args()
    filas = [json.loads(l) for l in open(a.gold, encoding="utf-8") if l.strip()]
    caps = {c["id"]: c for c in captura(a, filas)}
    for op in ("vector", "combmax", "combsum", "rrf"):
        rks = [fusiona(caps[f["id"]], op) for f in filas]
        por = metricas(filas, rks)
        pos = [v for k, v in por.items() if k != "negativo"]
        n, h = sum(v["n"] for v in pos), sum(v["hit5"] for v in pos)
        rr = sum(v["rr"] for v in pos) / max(n, 1)
        h1 = sum(1 for f, rk in zip(filas, rks) if f["expected_permalink"] and rk
                 and rk[0] in {f["expected_permalink"], *f.get("acceptable_permalinks", [])})
        estr = " ".join(f"{k}={v['hit5']}/{v['n']}" for k, v in sorted(por.items()) if k != "negativo")
        neg = por.get("negativo", {"n": 0, "con_algo": 0})
        print(f"{op:8} hit@5 {h}/{n} hit@1 {h1} MRR@10 {rr:.3f} | {estr} | "
              f"negativos con resultado {neg['con_algo']}/{neg['n']}")


if __name__ == "__main__":
    main()
