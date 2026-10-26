"""IC exacto de Poisson, puerta de la etapa 1 y veredicto final (spec medición en uso real)."""
import math

DECIDIDAS = ("regla-canal", "regla-fuera-canal", "canon-kb", "perfil-core")
UMBRAL = 0.4
MIN_UNIDAD = 30
FRACCION_MIN = 0.10


def _log_cdf(k, lam):
    """log P(X <= k) con X ~ Poisson(lam); en log para que k grande no desborde exp."""
    logs = [-lam + i * math.log(lam) - math.lgamma(i + 1) for i in range(k + 1)]
    m = max(logs)
    return m + math.log(sum(math.exp(x - m) for x in logs))


def _biseccion(f, lo, hi):
    """Raíz de f (decreciente) en [lo, hi]."""
    for _ in range(200):
        mid = (lo + hi) / 2
        if f(mid) > 0:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


def ic_poisson(k, alfa=0.05):
    """IC exacto (Garwood) de la media de un recuento k; no divide por exposición."""
    cota = 2 * k + 10 * math.sqrt(k + 1) + 20
    mitad = math.log(alfa / 2)
    hi = _biseccion(lambda lam: _log_cdf(k, lam) - mitad, 1e-12, cota)
    if k == 0:
        return (0.0, hi)
    # P(X >= k; lam) = alfa/2  <=>  P(X <= k-1; lam) = 1 - alfa/2
    objetivo = math.log1p(-alfa / 2)
    lo = _biseccion(lambda lam: _log_cdf(k - 1, lam) - objetivo, 1e-12, cota)
    return (lo, hi)


def _tasa_ic(k, n_sesiones):
    if n_sesiones <= 0:
        raise ValueError("n_sesiones debe ser > 0")
    lo, hi = ic_poisson(k)
    return k / n_sesiones, [lo / n_sesiones, hi / n_sesiones]


def decide(k, n_sesiones, umbral=UMBRAL):
    tasa, ic = _tasa_ic(k, n_sesiones)
    return {
        "k": k,
        "n": n_sesiones,
        "tasa": tasa,
        "ic": ic,
        "estado": "estabilizar" if ic[1] < umbral else "etapa2",
    }


def final(atribuciones, n_sesiones):
    por_atribucion = {}
    for a in atribuciones:
        por_atribucion[a] = por_atribucion.get(a, 0) + 1
    decididas = sum(por_atribucion.get(a, 0) for a in DECIDIDAS)
    n = len(atribuciones)
    tasa, ic = _tasa_ic(decididas, n_sesiones)
    if n >= MIN_UNIDAD and decididas / n < FRACCION_MIN:
        estado = "unidad_vacia"
    else:
        estado = "estabilizar" if ic[1] < UMBRAL else "seguir"
    return {
        "n_correcciones": n,
        "decididas": decididas,
        "por_atribucion": por_atribucion,
        "tasa_decididas": tasa,
        "ic": ic,
        "estado": estado,
    }
