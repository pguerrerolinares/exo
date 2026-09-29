#!/usr/bin/env python3
"""Campaña K — análisis de la etapa 1 (§6 del pre-registro + erratas). Escrito ANTES de ver resultados.

Por tarea y brazo: éxito = media sobre las 2 réplicas de (check.rc == 0). Reglas de conteo:
  - rc 1 → fallo · corrida cortada (fin != completed o sin result) → fallo (E3)
  - rc 2 («no evaluable») → fallo en la primaria (conservador y simétrico); se reporta aparte
    y se da una sensibilidad que excluye las tareas con algún rc 2.
Δ por tarea = éxito(A3) − éxito(A0). Primaria: media de Δ sobre las tareas del estrato,
IC95 bootstrap sobre tareas (10.000 réplicas, semilla 20260923), test de signos exacto
sobre tareas con Δ ≠ 0.
R1 (S1):  «EXO AYUDA» si IC_inf > 0 y Δ ≥ 0,10 · «EFECTO PEQUEÑO O NULO» si IC_sup < 0,10
          · si no, «NO CONCLUYENTE».
R2 (S2):  «SIN DAÑO» si IC_inf > −0,10 · si no, «DAÑO».
Uso: analisis_etapa1.py > informe (solo agregados; sin texto de tareas).
"""
import json, math, os, random, collections

K = os.path.expanduser(os.environ.get("K_ROOT", "~/.cache/exo-ablacion-k"))
SEMILLA, REPS = 20260923, 10000


def carga():
    exito = collections.defaultdict(dict)   # tarea -> brazo -> [0/1, 0/1]
    rc2, cortadas, meta = collections.Counter(), collections.Counter(), collections.defaultdict(list)
    for l in open(f"{K}/orden-etapa1.txt"):
        t, b, r = l.split()
        d = f"{K}/corridas/{t}/{b}-r{r}"
        try:
            m = json.load(open(f"{d}/meta.json"))
        except Exception:
            m = {}
        rc = open(f"{d}/check.rc").read().strip() if os.path.exists(f"{d}/check.rc") else "x"
        ok = m.get("fin") == "completed" and rc == "0"
        if m.get("fin") != "completed":
            cortadas[b] += 1
        if rc == "2":
            rc2[(t, b)] += 1
        exito[t].setdefault(b, []).append(1 if ok else 0)
        meta[b].append(m)
    return exito, rc2, cortadas, meta


def signos(deltas):
    pos = sum(d > 0 for d in deltas); neg = sum(d < 0 for d in deltas); n = pos + neg
    if n == 0:
        return pos, neg, 1.0
    k = min(pos, neg)
    p = sum(math.comb(n, i) for i in range(k + 1)) / 2 ** n
    return pos, neg, min(1.0, 2 * p)


def bootstrap(deltas):
    rng = random.Random(SEMILLA)
    n = len(deltas)
    medias = sorted(sum(deltas[rng.randrange(n)] for _ in range(n)) / n for _ in range(REPS))
    return medias[int(0.025 * REPS)], medias[int(0.975 * REPS) - 1]


def estrato(exito, pref, excluir=()):
    ds, tabla = [], []
    for t in sorted(exito):
        if (pref == "S2") != t.startswith("s2-") or t in excluir:
            continue
        e0 = sum(exito[t]["a0"]) / len(exito[t]["a0"]); e3 = sum(exito[t]["a3"]) / len(exito[t]["a3"])
        ds.append(e3 - e0); tabla.append((e0, e3))
    return ds, tabla


def informe(nombre, ds, tabla):
    n = len(ds); media = sum(ds) / n
    lo, hi = bootstrap(ds); pos, neg, p = signos(ds)
    e0 = sum(a for a, _ in tabla) / n; e3 = sum(b for _, b in tabla) / n
    print(f"| {nombre} | {n} | {e0:.3f} | {e3:.3f} | {media:+.3f} | [{lo:+.3f}, {hi:+.3f}] | {pos}/{neg} | {p:.4f} |")
    return media, lo, hi


exito, rc2, cortadas, meta = carga()
print("| estrato | tareas | éxito A0 | éxito A3 | Δ medio | IC95 | A3>A0 / A3<A0 | p signos |")
print("|---|---|---|---|---|---|---|---|")
m1, lo1, hi1 = informe("S1", *estrato(exito, "S1"))
m2, lo2, hi2 = informe("S2", *estrato(exito, "S2"))
con_rc2 = {t for t, _ in rc2}
print()
print("Sensibilidad (excluye tareas con algún rc 2):")
print("| estrato | tareas | éxito A0 | éxito A3 | Δ medio | IC95 | A3>A0 / A3<A0 | p signos |")
print("|---|---|---|---|---|---|---|---|")
informe("S1 sin rc2", *estrato(exito, "S1", con_rc2))
print()
r1 = ("EXO AYUDA" if lo1 > 0 and m1 >= 0.10 else "EFECTO PEQUEÑO O NULO" if hi1 < 0.10 else "NO CONCLUYENTE")
r2 = "SIN DAÑO" if lo2 > -0.10 else "DAÑO"
print(f"R1 (S1): {r1}")
print(f"R2 (S2): {r2}")
print()
print(f"rc 2 por brazo: {dict(collections.Counter(b for (_, b), c in rc2.items() for _ in range(c)))}")
print(f"cortadas por brazo: {dict(cortadas)}")
for b in ("a0", "a3"):
    ms = [m for m in meta[b] if "tokens_in" in m]
    if ms:
        tk = sorted(m["tokens_in"] for m in ms); tu = sorted(m["turnos"] or 0 for m in ms)
        print(f"{b}: tokens_in mediana {tk[len(tk)//2]:,} · turnos mediana {tu[len(tu)//2]} · n {len(ms)}")
