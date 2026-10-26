"""Veredicto de la validación pre-registrada (spec 2026-10-26, §5.3). Salida sin texto de reglas."""
import argparse
import glob
import importlib.util
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import estadistica as E
from rubrica import valida_etiqueta


def _siembra():
    spec = importlib.util.spec_from_file_location(
        "siembra_reglas", os.path.join(HERE, "..", "..", "scripts", "siembra-reglas.py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def _acierto_y_confusion(pred, verdad):
    c = {"tp": 0, "fp": 0, "fn": 0, "tn": 0}
    for p, v in zip(pred, verdad):
        c[("t" if p == v else "f") + ("p" if p else "n")] += 1
    return c


def calcula(etiquetas, universo, ambito, cumple):
    sellado = set(universo)
    fuera = [i for i in etiquetas if i not in sellado]
    if fuera:
        print(f"aviso: {len(fuera)} etiqueta(s) con id fuera del universo sellado, ignoradas", file=sys.stderr)
    ok = [i for i in universo if i in etiquetas]
    pred_b = [etiquetas[i]["choque"] != "E" for i in ok]
    pred_base = [ambito[i] != "dominio-proyecto" for i in ok]
    verdad = [not cumple[i] for i in ok]

    if ok:
        kappa_b, kappa_base = E.kappa(pred_b, verdad), E.kappa(pred_base, verdad)
        delta = E.bootstrap_delta(pred_b, pred_base, verdad)
        acc_b = sum(p == v for p, v in zip(pred_b, verdad)) / len(ok)
        acc_base = sum(p == v for p, v in zip(pred_base, verdad)) / len(ok)
    else:
        kappa_b = kappa_base = acc_b = acc_base = None
        delta = {"delta": None, "ic95": [None, None], "descartados": 0}

    por_choque = {}
    for c in ("P", "D", "E"):
        v = [not cumple[i] for i in ok if etiquetas[i]["choque"] == c]
        por_choque[c] = {"n": len(v), "no_cumple": sum(v), "proporcion": sum(v) / len(v) if v else None}

    sin = len(universo) - len(ok)
    return {
        "n_universo": len(universo),
        "n_analizado": len(ok),
        "sin_etiqueta": sin,
        "citas_no_encontradas": sum(etiquetas[i].get("cita_recortada") is False for i in ok),
        "kappa_b": kappa_b,
        "kappa_base": kappa_base,
        "precision_b": acc_b,
        "precision_base": acc_base,
        "confusion_b": _acierto_y_confusion(pred_b, verdad),
        "no_cumple_por_choque": por_choque,
        "delta": delta,
        "decision": E.decide(kappa_b, sin),
    }


def _lee_etiquetas(path):
    out = {}
    with open(path) as f:
        for linea in f:
            if not linea.strip():
                continue
            obj = json.loads(linea)
            e = valida_etiqueta(obj)
            if e:
                out[e["id"]] = e
    return out


def main(argv=None):
    ap = argparse.ArgumentParser()
    for a in ("etiquetas", "universo", "pool", "criba", "salida"):
        ap.add_argument("--" + a, required=True)
    a = ap.parse_args(argv)

    siembra = _siembra()
    with open(a.universo) as f:
        universo = [l.strip() for l in f if l.strip()]
    ambito = {}
    for p in glob.glob(os.path.join(a.pool, "reglas-g*.jsonl")):
        with open(p) as f:
            for l in f:
                if l.strip():
                    r = json.loads(l)
                    ambito[r["id"]] = r["ambito"]
    cumple = {i: siembra.a0_cumple(a.criba, i) for i in universo}
    resultado = calcula(_lee_etiquetas(a.etiquetas), universo, ambito, cumple)
    with open(a.salida, "w") as f:
        json.dump(resultado, f, indent=2)
        f.write("\n")
    print(f"universo={resultado['n_universo']} analizado={resultado['n_analizado']} "
          f"sin_etiqueta={resultado['sin_etiqueta']}")


if __name__ == "__main__":
    main()
