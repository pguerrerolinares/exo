#!/usr/bin/env python3
"""Replay del eval set vía `exo search --json` (arm engine). Hermano de
replay.py — replay.py NO se generaliza ni se toca (spec M2 §4: está acoplado
al baile de config.json del CLI de basic-memory, que el engine no necesita).

Uso: replay-engine.py <arm> --db <ruta> [--exo <binario>] [--limite 5]

`--db` es obligatorio (D6: ningún default persistente — la ruta del índice
la decide quien invoca, no el script). `--exo` por defecto resuelve al
binario release del propio repo; `exo` NUNCA se asume en $PATH (no está
instalado — instalar es M5b).

MAPEO VINCULANTE (brief m2-05): el envelope del engine declara siempre
`search_type: "fts"` (contrato §4.1, sellado, no se toca); este script lo
reetiqueta a "text" en la fila jsonl porque `analyze.py` hardcodea
`["hybrid", "text", "vector"]` (L60) y los resultados reales de basic-memory
usan "text" para FTS. El mapeo vive SOLO aquí — nunca en el engine (que
fijaría "fts" para siempre) ni en analyze.py (su docstring prohíbe tocar
nada salvo norm(), y esto no es un permalink).

Vector/hybrid no existen aún (M2-06/07): SEARCH_TYPE_MAP solo cubre "fts";
el punto de extensión queda declarado, sin implementar.
"""
import argparse
import json
import subprocess
import sys
import time
from pathlib import Path

BASE = Path(__file__).resolve().parent.parent  # evals/retrieval-fase0/ (mismo patrón que replay.py)

# Binario release del propio repo, resuelto relativo al script. Nota: el
# brief da la fórmula `parents[2] / "engine/target/release/exo"`, pero desde
# este fichero (evals/retrieval-fase0/harness/replay-engine.py) `parents[2]`
# es `evals/` — el binario vive en `<repo>/engine/...`, un nivel más arriba
# (`BASE.parent.parent`, mismo `evals -> repo` que usa el propio BASE de
# arriba). Desviación documentada: se corrige a la ruta real, verificada
# contra el binario compilado en esta sesión.
EXO_DEFAULT = BASE.parent.parent / "engine" / "target" / "release" / "exo"

SEARCH_TYPE_MAP = {"fts": "text"}  # vector/hybrid: punto de extensión M2-06/07


def search(exo_bin, db, query, limite):
    cmd = [str(exo_bin), "search", "--db", str(db), "--limite", str(limite), "--json", query]
    try:
        out = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    except subprocess.TimeoutExpired:
        return {"error": "timeout"}
    if out.returncode != 0:
        return {"error": out.stderr.strip()[-500:]}
    data = json.loads(out.stdout)["data"]
    search_type = SEARCH_TYPE_MAP.get(data["search_type"], data["search_type"])
    return {"search_type": search_type, "elapsed_s": data["elapsed_s"], "results": data["results"]}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("arm", help="nombre del arm (fichero de salida: results/<arm>.jsonl)")
    ap.add_argument("--db", required=True, help="fichero SQLite del índice del engine")
    ap.add_argument("--exo", default=str(EXO_DEFAULT), help="binario exo (default: release del repo)")
    ap.add_argument("--limite", type=int, default=5, help="resultados por query (default 5, hit@5)")
    args = ap.parse_args()

    exo_bin = Path(args.exo)
    if not exo_bin.exists():
        sys.exit(f"error: binario exo no encontrado en {exo_bin} (¿cargo build --release?)")

    rows = [json.loads(l) for l in open(BASE / "eval.jsonl")]
    outpath = BASE / "results" / f"{args.arm}.jsonl"
    outpath.parent.mkdir(parents=True, exist_ok=True)

    with open(outpath, "w") as f:
        for i, row in enumerate(rows):
            res = search(exo_bin, args.db, row["query"], args.limite)
            if "error" in res:
                fila = {"query": row["query"], "search_type": "text", "error": res["error"]}
            else:
                fila = {"query": row["query"], **res}
            f.write(json.dumps(fila, ensure_ascii=False) + "\n")
            f.flush()
            print(f"[{i + 1}/{len(rows)}] {row['query'][:50]}", file=sys.stderr)
    print(f"OK → {outpath}", file=sys.stderr)


if __name__ == "__main__":
    main()
