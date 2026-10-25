#!/usr/bin/env python3
"""Ap (spec §7.2, #190) — umbrales k(N) y potencia. Escrito ANTES de sembrar y de correr.

Uso: umbral.py [--potencia]  → tabla en stdout (la que sella preregistro.md).

Modelo nulo: éxito por corrida p_i ~ Beta(A, B) por sonda, ajustada por momentos a las 24 sondas de
proyecto del brazo O de la campaña M (65/72 = 0,903; distribución 19/3/2/0 en 3/3, 2/3, 1/3, 0/3).
Una sonda «pasa» si cumple >= 2 de 3 réplicas: q = E[3p² - 2p³]. Las sondas son independientes.
Reglas (todas con P(·|nula) calculada exacta, binomial(N, q)):
  réplica de Ap1  r(N) = máx r con P(Ap1 < r) <= 0,025
  caída absoluta  k(N) = máx k con P(Ap  < k) <= 0,025
  nivel del control s(N) = floor(q·N): el absoluto solo adjudica si Ap1 >= s(N)
  canal: además, Ap1 >= 0,80 por corrida (CANAL_MIN) para que haya réplica
--potencia simula la regla entera (pareado bootstrap 97,5 % O absoluto) con semilla fija.
"""
import math
import sys

A, B = 2.86, 0.31
N_MIN, N_MAX = 18, 24
ALFA = 0.025
CANAL_MIN = 0.80
SEMILLA, SIMS, BOOT = 20261025, 2000, 1000
MODERADA, FUERTE = 4.5 / 5.5, 2.7 / 5.5  # factor sobre p, modelo de techo-reglas-3


def momento(n):
    return math.prod((A + i) / (A + B + i) for i in range(n))


Q = 3 * momento(2) - 2 * momento(3)


def cdf_lt(n, q, k):
    return sum(math.comb(n, i) * q**i * (1 - q) ** (n - i) for i in range(k))


def umbrales(n):
    r = max(x for x in range(n + 1) if cdf_lt(n, Q, x) <= ALFA)
    k = max(x for x in range(n + 1) if cdf_lt(n, Q, x) <= ALFA)
    return r, k, math.floor(Q * n)


def potencia(n, escenario, rng):
    import numpy as np
    cae = sin_replica = 0
    for _ in range(SIMS):
        p = rng.beta(A, B, n)
        if escenario == "moderada":
            pa = p * MODERADA
        elif escenario == "fuerte":
            pa = p * FUERTE
        elif escenario.startswith("pierde"):
            pa = p.copy(); pa[rng.choice(n, int(escenario[6:]), replace=False)] = 0.05
        else:
            pa = p
        s1 = rng.binomial(3, p); s0 = rng.binomial(3, pa)
        r, k, s = umbrales(n)
        if (s1 >= 2).sum() < r or s1.sum() / (3 * n) < CANAL_MIN:
            sin_replica += 1
            continue  # sin réplica no se adjudica: no cuenta como caída
        d = (s0 - s1) / 3
        medias = np.sort(d[rng.integers(0, n, (BOOT, n))].mean(axis=1))
        pareado = medias[int(0.975 * BOOT) - 1] < 0
        absoluto = (s1 >= 2).sum() >= s and (s0 >= 2).sum() < k
        cae += pareado or absoluto
    return cae / SIMS, sin_replica / SIMS


def main():
    print(f"q = P(sonda >= 2/3 | nula) = {Q:.4f}  (Beta({A}, {B}))")
    print("N  | réplica Ap1 >= r | cae abs. si Ap < k | abs. solo si Ap1 >= s | P(no réplica) | α abs.")
    for n in range(N_MIN, N_MAX + 1):
        r, k, s = umbrales(n)
        print(f"{n} | {r} | {k} | {s} | {cdf_lt(n, Q, r):.3f} | {cdf_lt(n, Q, k):.3f}")
    if "--potencia" in sys.argv:
        import numpy as np
        rng = np.random.default_rng(SEMILLA)
        print("\nN  | P(cae): nula · moderada · fuerte · pierde3 · pierde5 | P(sin réplica | nula)")
        for n in (N_MIN, 21, N_MAX):
            fila = [potencia(n, e, rng) for e in ("nula", "moderada", "fuerte", "pierde3", "pierde5")]
            print(f"{n} | " + " · ".join(f"{c:.2f}" for c, _ in fila) + f" | {fila[0][1]:.3f}")


if __name__ == "__main__":
    main()
