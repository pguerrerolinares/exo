#!/usr/bin/env python3
"""Validación de la etapa 1 (spec 2026-10-26 §4.2): kappa del juez contra las etiquetas del dueño.

Etapa 2: acuerdo del dueño con la atribución del juez (sin umbral). Códigos de salida: 0 ok, 1 entrada inválida, 2 quedan etiquetas sin rellenar (ids por stderr)."""
import argparse, json, os, sys

H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(H, "..", "rubrica-reglas"))
sys.path.insert(0, H)
from estadistica import kappa  # noqa: E402
import puerta  # noqa: E402

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


VALIDAS = puerta.DECIDIDAS + ("dicho-en-sesion", "nueva")


def acuerdo_etapa2(pares):
    """pares: [(atribucion del juez, atribucion del dueño)]."""
    por = {}
    for j, d in pares:
        r = por.setdefault(j, {"n": 0, "coinciden": 0})
        r["n"] += 1
        r["coinciden"] += j == d
    n = len(pares)
    return {"n": n, "acuerdo": sum(j == d for j, d in pares) / n if n else None,
            "acuerdo_decidida": sum((j in puerta.DECIDIDAS) == (d in puerta.DECIDIDAS) for j, d in pares) / n if n else None,
            "por_atribucion": por}


def etapa2(a):
    etiquetas = _lee_jsonl(a.etiquetas)
    sin = [e["id"] for e in etiquetas if e.get("atribucion_dueño") not in VALIDAS]
    if sin:
        print("validar: atribucion_dueño sin rellenar (o fuera de la lista): " + ", ".join(sin), file=sys.stderr)
        return 2
    juez_ = {d["id"]: d for d in _lee_jsonl(a.atribuciones)}
    ausentes = [e["id"] for e in etiquetas if "atribucion" not in juez_.get(e["id"], {})]
    if ausentes:
        print("validar: ids sin atribución del juez: " + ", ".join(ausentes), file=sys.stderr)
        return 1
    res = acuerdo_etapa2([(juez_[e["id"]]["atribucion"], e["atribucion_dueño"]) for e in etiquetas])
    with open(a.salida, "w") as f:
        json.dump(res, f)
        f.write("\n")
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--etapa", required=True, choices=("1", "2"))
    for x in ("etiquetas", "salida"):
        ap.add_argument("--" + x, required=True)
    ap.add_argument("--detecciones", help="etapa 1: --salida de detectar.py")
    ap.add_argument("--atribuciones", help="etapa 2: --salida de atribuir.py")
    a = ap.parse_args()
    if a.etapa == "2":
        if not a.atribuciones:
            print("validar: la etapa 2 necesita --atribuciones", file=sys.stderr)
            return 1
        return etapa2(a)
    if not a.detecciones:
        print("validar: la etapa 1 necesita --detecciones", file=sys.stderr)
        return 1

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
