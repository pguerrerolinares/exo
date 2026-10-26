#!/usr/bin/env python3
"""Validación de la etapa 1 (spec 2026-10-26 §4.2): kappa del juez contra las etiquetas del dueño.

Códigos de salida: 0 ok, 1 entrada inválida, 2 quedan etiquetas sin rellenar (ids por stderr)."""
import argparse, json, os, sys

H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(H, "..", "rubrica-reglas"))
from estadistica import kappa  # noqa: E402

UMBRAL = 0.4


def metricas(pred, verdad):
    tp = sum(p and v for p, v in zip(pred, verdad))
    fp = sum(p and not v for p, v in zip(pred, verdad))
    fn = sum(v and not p for p, v in zip(pred, verdad))
    return {"n": len(pred), "kappa": kappa(pred, verdad) if pred else None,
            "precision": tp / (tp + fp) if tp + fp else None, "recall": tp / (tp + fn) if tp + fn else None}


def _lee_jsonl(path):
    with open(path) as f:
        return [json.loads(l) for l in f if l.strip()]


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--etapa", required=True, choices=("1",))
    for x in ("etiquetas", "detecciones", "salida"):
        ap.add_argument("--" + x, required=True)
    a = ap.parse_args()

    etiquetas = _lee_jsonl(a.etiquetas)
    sin = [e["id"] for e in etiquetas if e.get("etiqueta") not in ("si", "no")]
    if sin:
        print("validar: etiquetas sin rellenar (o distintas de si|no): " + ", ".join(sin), file=sys.stderr)
        return 2
    det = {d["id"]: d for d in _lee_jsonl(a.detecciones)}
    ausentes = [e["id"] for e in etiquetas if e["id"] not in det]
    if ausentes:
        print("validar: ids sin detección: " + ", ".join(ausentes), file=sys.stderr)
        return 1
    usadas = [e for e in etiquetas if not det[e["id"]].get("sin_veredicto")]
    pred = [det[e["id"]]["correccion"] == "si" for e in usadas]
    verdad = [e["etiqueta"] == "si" for e in usadas]
    m = metricas(pred, verdad)
    # round: kappa = 0.4 exacto en papel sale 0.3999999999999999 en float.
    m["juez_valido"] = m["kappa"] is not None and round(m["kappa"], 10) >= UMBRAL
    m["n_sin_veredicto"] = len(etiquetas) - len(usadas)
    with open(a.salida, "w") as f:
        json.dump(m, f)
        f.write("\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
