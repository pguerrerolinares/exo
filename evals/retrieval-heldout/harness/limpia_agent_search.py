#!/usr/bin/env python3
"""Limpieza del estrato `agent-search` del gold J (borrador §3). Entrada: un
fichero de texto con una línea por comando `exo|kbx search|targets` minado de
`~/.claude/projects/**/*.jsonl` (extracción autorizada por Paul el
2026-09-19; el minado con procedencia lo hace pool.pool_comandos).
Criterio escrito, en este orden: (1) la línea parsea con
pool.query_de_comando (shlex; la query es el positional); (2) ni la línea ni
la query contienen marcadores de doc/plan/test: `< > … * [ ] \\` | { } $`,
`...`, `jq`; (3) query normalizada de ≥ 2 caracteres; (4) anti-fuga: forma
normalizada distinta y Jaccard < 0,8 frente a TODAS las listas de exclusión
(las 55 y las 147 de C); (5) dedupe por forma normalizada (gana la primera).
Salida: JSONL {"query","source":"agent-search"} y recuento JSON a stdout.
Uso: limpia_agent_search.py --raw F --excluye A [--excluye B] --out F.jsonl
"""
import argparse
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from pool import jaccard, normaliza, query_de_comando  # noqa: E402

MARCADORES = re.compile(r"[<>…*\[\]`|{}$]|\.\.\.|\bjq\b")


def limpia(lineas, excluidas):
    desc = {"no_parsea": 0, "marcador": 0, "vacia": 0, "dup-exclusion": 0, "dup-pool": 0}
    vistos, out = set(), []
    for l in lineas:
        if MARCADORES.search(l):
            desc["marcador"] += 1
            continue
        q = query_de_comando(l)
        if not q:
            desc["no_parsea"] += 1
            continue
        n = normaliza(q)
        if len(n) < 2:
            desc["vacia"] += 1
            continue
        if any(n == m or jaccard(n, m) >= 0.8 for m in excluidas):
            desc["dup-exclusion"] += 1
            continue
        if n in vistos:
            desc["dup-pool"] += 1
            continue
        vistos.add(n)
        out.append({"query": q.strip(), "source": "agent-search"})
    return out, desc


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--raw", required=True)
    ap.add_argument("--excluye", action="append", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    lineas = [l.rstrip("\n") for l in open(a.raw, encoding="utf-8") if l.strip()]
    excl = [normaliza(json.loads(l)["query"]) for r in a.excluye for l in open(r, encoding="utf-8") if l.strip()]
    out, desc = limpia(lineas, excl)
    with open(a.out, "w", encoding="utf-8") as fh:
        for c in out:
            fh.write(json.dumps(c, ensure_ascii=False) + "\n")
    print(json.dumps({"entrada": len(lineas), "salida": len(out), "descartes": desc}, ensure_ascii=False))


if __name__ == "__main__":
    main()
