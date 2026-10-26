"""Kappa, regla de decisión y bootstrap pareado de la rúbrica (spec §5.3)."""
import random

MAX_SIN_ETIQUETA = 3


def kappa(pred, verdad):
    """Kappa de Cohen; None si pe = 1 (no hay acuerdo "por azar" que descontar)."""
    n = len(pred)
    po = sum(p == v for p, v in zip(pred, verdad)) / n
    pp = sum(pred) / n
    pv = sum(verdad) / n
    pe = pp * pv + (1 - pp) * (1 - pv)
    if pe == 1:
        return None
    return (po - pe) / (1 - pe)


def decide(kappa_b, n_sin_etiqueta):
    cae = kappa_b is None or kappa_b <= 0
    return {
        "estado": "nula" if n_sin_etiqueta > MAX_SIN_ETIQUETA else "valida",
        "criterio_2": not cae,
        "rubrica": "veto + dato + coste + fichero" if cae else "D",
    }


def bootstrap_delta(pred_b, pred_base, verdad, n=10000, semilla=20261026):
    rng = random.Random(semilla)
    total = len(verdad)
    kb, kbase = kappa(pred_b, verdad), kappa(pred_base, verdad)
    delta = None if kb is None or kbase is None else kb - kbase
    deltas = []
    descartados = 0
    for _ in range(n):
        idx = [rng.randrange(total) for _ in range(total)]
        v = [verdad[i] for i in idx]
        k1 = kappa([pred_b[i] for i in idx], v)
        k2 = kappa([pred_base[i] for i in idx], v)
        if k1 is None or k2 is None:
            descartados += 1
        else:
            deltas.append(k1 - k2)
    if not deltas:
        return {"delta": delta, "ic95": [None, None], "descartados": descartados}
    deltas.sort()
    m = len(deltas)
    lo = deltas[int(0.025 * m)]
    hi = deltas[min(m - 1, int(0.975 * m))]
    return {"delta": delta, "ic95": [lo, hi], "descartados": descartados}
