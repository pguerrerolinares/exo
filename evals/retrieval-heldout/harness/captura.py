#!/usr/bin/env python3
"""Captura por query de las tres listas del binario (pre-registro §4):
FTS --limit 50 (= K_C), vector --limit 1000 --min-similarity 0.0, hybrid
--limit 10 --min-similarity 0.40 --bonus 0.0 --fts-scale 0.6 (explícitos: se
mide lo declarado, no lo que el binario tenga por default).

Reutiliza search() de evals/retrieval-fase0/harness/replay-engine.py sin
modificarlo. Salida privada (--out en $PRIV).
Uso: captura.py --gold G --db D --exo BIN --out F
"""
import argparse
import importlib.util
import json
import sys
from pathlib import Path

_M0 = Path(__file__).resolve().parents[2] / "retrieval-fase0" / "harness"
_spec = importlib.util.spec_from_file_location("replay_engine", _M0 / "replay-engine.py")
replay_engine = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(replay_engine)
sys.path.insert(0, str(Path(__file__).resolve().parent))
from metricas import carga_gold  # noqa: E402


def _pares(res):
    return [[r["permalink"], r["score"]] for r in res["results"]]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--gold", required=True)
    ap.add_argument("--db", required=True)
    ap.add_argument("--exo", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    filas = carga_gold(a.gold)
    with open(a.out, "w", encoding="utf-8") as fh:
        for n, fila in enumerate(filas, start=1):
            q = fila["query"]
            fts = replay_engine.search(a.exo, a.db, q, 50, "fts")
            vec = replay_engine.search(a.exo, a.db, q, 1000, "vector", min_similitud=0.0)
            hyb = replay_engine.search(a.exo, a.db, q, 10, "hybrid", min_similitud=0.40, bonus=0.0, escala_fts=0.6)
            errores = [r["error"] for r in (fts, vec, hyb) if "error" in r]
            linea = {"id": fila["id"], "errores": errores}
            if not errores:
                linea.update(fts=_pares(fts), vector=_pares(vec), hybrid=_pares(hyb))
            fh.write(json.dumps(linea, ensure_ascii=False) + "\n")
            print(f"[{n}/{len(filas)}] {fila['id']}", file=sys.stderr)


if __name__ == "__main__":
    main()
