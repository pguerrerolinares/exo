#!/usr/bin/env python3
"""Métricas del held-out de la campaña C (pre-registro
docs/superpowers/plans/2026-09-13-campana-c-preregistro.md §4-§6).

Réplica offline, EXACTA, de `fusiona`/`normaliza_fts` de
engine/src/buscador.rs (fusión sellada) y RRF (Cormack, Clarke & Büttcher,
SIGIR 2009, k=60) sobre las listas capturadas por captura.py. La fidelidad
(fusión offline == hybrid del binario) es condición de validez (§4).

Nunca emite texto de query ni permalinks por fila en los agregados: el repo
es público. El detalle por query va a --detalle (directorio privado).
"""
import argparse
import json
import math
import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "retrieval-fase0" / "harness"))
from analyze import norm  # noqa: E402

K_HIT = 5
K_RR = 10
RRF_K = 60
UMBRAL_SELLADO = 0.40
BETA_SELLADA = 0.6
BONUS_SELLADO = 0.0
SEMILLA = 20260913


def carga_gold(ruta):
    filas = []
    for n, linea in enumerate(Path(ruta).read_text(encoding="utf-8").splitlines(), start=1):
        if not linea.strip():
            continue
        f = json.loads(linea)
        f.setdefault("id", f"m{n:02d}")
        f.setdefault("acceptable_permalinks", [])
        filas.append(f)
    return filas


def aplica_overlay(filas, ruta_overlay):
    por_query = {}
    for linea in Path(ruta_overlay).read_text(encoding="utf-8").splitlines():
        if linea.strip():
            o = json.loads(linea)
            por_query[o["query"]] = o["acceptable_permalinks"]
    for f in filas:
        if f["query"] in por_query:
            f["acceptable_permalinks"] = por_query[f["query"]]
    return filas


def carga_captura(ruta):
    out = {}
    for linea in Path(ruta).read_text(encoding="utf-8").splitlines():
        if not linea.strip():
            continue
        c = json.loads(linea)
        if c.get("errores"):
            out[c["id"]] = None
            continue
        out[c["id"]] = {k: [(norm(p), s) for p, s in c[k]] for k in ("fts", "vector", "hybrid")}
    return out


def relevantes(fila, estricto=False):
    exp = norm(fila.get("expected_permalink"))
    if exp is None:
        return set()
    if estricto:
        return {exp}
    return {exp} | {norm(p) for p in fila.get("acceptable_permalinks", [])}


def hit(ranking, rel, k=K_HIT):
    return any(p in rel for p in ranking[:k])


def rr(ranking, rel, k=K_RR):
    for i, p in enumerate(ranking[:k], start=1):
        if p in rel:
            return 1.0 / i
    return 0.0


def _ordena(pares, limite):
    return sorted(pares, key=lambda x: (-x[1], x[0]))[:limite]


def fusion_sellada(fts, vector, bonus, beta, umbral, limite):
    """buscador.rs: normaliza_fts (f_max con fold desde 0.0; f_max == 0 ⇒
    canal FTS descartado) + fusiona (max + bonus·min, canal ausente = 0)."""
    f_max = max([0.0] + [s for _, s in fts])
    f = {} if f_max == 0.0 else {p: beta * s / f_max for p, s in fts}
    v = {p: s for p, s in vector if s >= umbral}
    pares = []
    for p in set(f) | set(v):
        vv, ff = v.get(p, 0.0), f.get(p, 0.0)
        pares.append((p, max(vv, ff) + bonus * min(vv, ff)))
    return _ordena(pares, limite)


def rrf(fts, vector, umbral, limite, k=RRF_K):
    """RRF: Σ 1/(k + rango), rango 1-based; FTS se suma antes que vector
    (mismo orden de suma que la implementación Rust de la Task 12)."""
    puntos = {}
    for lista in ([p for p, _ in fts], [p for p, s in vector if s >= umbral]):
        for rango, p in enumerate(lista, start=1):
            puntos[p] = puntos.get(p, 0.0) + 1.0 / (k + rango)
    return _ordena(list(puntos.items()), limite)


def rankings(captura, brazo):
    out = {}
    for i, c in captura.items():
        if c is None:
            out[i] = []
            continue
        if brazo == "sellado":
            pares = fusion_sellada(c["fts"], c["vector"], BONUS_SELLADO, BETA_SELLADA, UMBRAL_SELLADO, K_RR)
        elif brazo == "sellado-035":
            pares = fusion_sellada(c["fts"], c["vector"], BONUS_SELLADO, BETA_SELLADA, 0.35, K_RR)
        elif brazo == "rrf":
            pares = rrf(c["fts"], c["vector"], UMBRAL_SELLADO, K_RR)
        elif brazo == "vector":
            pares = _ordena([(p, s) for p, s in c["vector"] if s >= UMBRAL_SELLADO], K_RR)
        elif brazo == "fts":
            pares = c["fts"][:K_RR]
        else:
            raise ValueError(f"brazo desconocido: {brazo}")
        out[i] = [p for p, _ in pares]
    return out


def fidelidad(captura, brazo="sellado"):
    malas = []
    for i, c in captura.items():
        if c is None:
            malas.append(i)
            continue
        if brazo == "sellado":
            offline = fusion_sellada(c["fts"], c["vector"], BONUS_SELLADO, BETA_SELLADA, UMBRAL_SELLADO, K_RR)
        else:
            offline = rrf(c["fts"], c["vector"], UMBRAL_SELLADO, K_RR)
        motor = c["hybrid"]
        if [p for p, _ in offline] != [p for p, _ in motor] or any(
            abs(a[1] - b[1]) > 1e-12 for a, b in zip(offline, motor)
        ):
            malas.append(i)
    return sorted(malas)


def mcnemar_exacto(b, c):
    n = b + c
    if n == 0:
        return 1.0
    cola = sum(math.comb(n, i) for i in range(min(b, c) + 1)) / 2**n
    return min(1.0, 2 * cola)


def wilson(x, n, z=1.959964):
    if n == 0:
        return (0.0, 1.0)
    p = x / n
    d = 1 + z * z / n
    centro = (p + z * z / (2 * n)) / d
    medio = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / d
    return (centro - medio, centro + medio)


def bootstrap_ic95(deltas, n=10000, semilla=SEMILLA):
    if not deltas:
        return (0.0, 0.0)
    rng = random.Random(semilla)
    m = len(deltas)
    medias = sorted(sum(rng.choice(deltas) for _ in range(m)) / m for _ in range(n))
    return (medias[int(0.025 * n)], medias[int(0.975 * n) - 1])


def pareada(cand, ref, gold, estricto=False):
    arregla = rompe = 0
    deltas = []
    for fila in gold:
        rel = relevantes(fila, estricto)
        i = fila["id"]
        hc, hr = hit(cand.get(i, []), rel), hit(ref.get(i, []), rel)
        arregla += hc and not hr
        rompe += hr and not hc
        deltas.append(rr(cand.get(i, []), rel) - rr(ref.get(i, []), rel))
    return {
        "arregla": arregla,
        "rompe": rompe,
        "p_mcnemar": mcnemar_exacto(arregla, rompe),
        "ic95_delta_mrr": bootstrap_ic95(deltas),
    }


def decide(par, neto_min, factor):
    if (
        par["arregla"] - par["rompe"] >= neto_min
        and par["arregla"] >= factor * par["rompe"]
        and par["ic95_delta_mrr"][1] >= 0
    ):
        return "GANA"
    return "NO GANA"


def decide_r1(par_vector, par_fts, neto_min, factor):
    """par_x = pareada(cand=x, ref=sellado). R1 del pre-registro §6."""
    if "GANA" in (decide(par_vector, neto_min, factor), decide(par_fts, neto_min, factor)):
        return "NO GENERALIZA"
    if par_vector["rompe"] >= par_vector["arregla"] and par_fts["rompe"] >= par_fts["arregla"]:
        return "GENERALIZA"
    return "INDETERMINADO"


def _pares_kv(valores):
    out = {}
    for v in valores or []:
        k, x = v.split("=", 1)
        out[k] = x
    return out


def informe(gold, capturas, neto_min, factor, estricto, p95, rebuild_s, ruta_detalle):
    no_nulas = [f for f in gold if f.get("expected_permalink")]
    nulas = [f for f in gold if not f.get("expected_permalink")]
    brazos = {}
    for indice, cap in capturas.items():
        nombres = ["sellado", "vector", "fts", "rrf", "sellado-035"] if indice == "base" else ["sellado", "rrf"]
        for b in nombres:
            brazos[f"{indice}:{b}"] = rankings(cap, b)
    modo, otro = ("strict", "lenient") if estricto else ("lenient", "strict")
    L = [f"# Agregados — campaña C (modo de decisión: {modo})", ""]
    L.append(f"- filas: {len(gold)} · no nulas: {len(no_nulas)} · nulas (corpus negativo): {len(nulas)}")
    estratos = sorted({f.get("source", "?") for f in no_nulas})
    L.append("- no nulas por estrato: " + ", ".join(f"{e}={sum(f.get('source') == e for f in no_nulas)}" for e in estratos))
    base = capturas.get("base", {})
    activas = sum(1 for f in no_nulas if base.get(f["id"]) and base[f["id"]]["fts"])
    L.append(f"- queries no nulas con fusión activa (≥1 candidato FTS en base): {activas}")
    errores = {ind: sum(1 for c in cap.values() if c is None) for ind, cap in capturas.items()}
    L.append(f"- capturas con error por índice: {errores}")
    L += ["", f"| brazo | hit@5 {modo} [Wilson 95%] | hit@5 {otro} | hit@1 | MRR@10 |", "|---|---|---|---|---|"]
    detalle = {f["id"]: {"id": f["id"], "source": f.get("source")} for f in gold}
    for nombre, rk in brazos.items():
        x = sum(hit(rk.get(f["id"], []), relevantes(f, estricto)) for f in no_nulas)
        x2 = sum(hit(rk.get(f["id"], []), relevantes(f, not estricto)) for f in no_nulas)
        h1 = sum(hit(rk.get(f["id"], []), relevantes(f, estricto), k=1) for f in no_nulas)
        mrr = sum(rr(rk.get(f["id"], []), relevantes(f, estricto)) for f in no_nulas) / max(1, len(no_nulas))
        lo, hi = wilson(x, len(no_nulas))
        L.append(f"| {nombre} | {x}/{len(no_nulas)} [{lo:.3f}, {hi:.3f}] | {x2}/{len(no_nulas)} | {h1}/{len(no_nulas)} | {mrr:.4f} |")
        for f in gold:
            rel = relevantes(f, estricto)
            detalle[f["id"]][nombre] = {"hit5": hit(rk.get(f["id"], []), rel), "rr10": rr(rk.get(f["id"], []), rel), "n_resultados": len(rk.get(f["id"], []))}
    L += ["", "## hit@5 por estrato", "", "| brazo | " + " | ".join(estratos) + " |", "|---|" + "---|" * len(estratos)]
    for nombre, rk in brazos.items():
        celdas = []
        for e in estratos:
            fe = [f for f in no_nulas if f.get("source") == e]
            celdas.append(f"{sum(hit(rk.get(f['id'], []), relevantes(f, estricto)) for f in fe)}/{len(fe)}")
        L.append(f"| {nombre} | " + " | ".join(celdas) + " |")
    L += ["", "## corpus negativo (nulas con ≥1 resultado en top-5)", ""]
    for nombre, rk in brazos.items():
        L.append(f"- {nombre}: {sum(1 for f in nulas if rk.get(f['id'], [])[:K_HIT])}/{len(nulas)}")
    pares = {}
    L += ["", "## pareadas (cand vs ref)", "", "| cand | ref | ARREGLA | ROMPE | p McNemar exacto | IC95 ΔMRR@10 | regla GANA |", "|---|---|---|---|---|---|---|"]
    for cand, ref in [("base:vector", "base:sellado"), ("base:fts", "base:sellado"), ("base:rrf", "base:sellado"),
                      ("solape:sellado", "base:sellado"), ("late:sellado", "base:sellado"),
                      ("solape:rrf", "base:rrf"), ("late:rrf", "base:rrf")]:
        if cand in brazos and ref in brazos:
            par = pareada(brazos[cand], brazos[ref], no_nulas, estricto)
            pares[(cand, ref)] = par
            lo, hi = par["ic95_delta_mrr"]
            L.append(f"| {cand} | {ref} | {par['arregla']} | {par['rompe']} | {par['p_mcnemar']:.4f} | [{lo:.4f}, {hi:.4f}] | {decide(par, neto_min, factor)} |")
    L += ["", f"## decisiones (NETO ≥ {neto_min}, ARREGLA ≥ {factor}·ROMPE, veto ΔMRR)", ""]
    if ("base:vector", "base:sellado") in pares:
        L.append(f"- R1 (H7): {decide_r1(pares[('base:vector', 'base:sellado')], pares[('base:fts', 'base:sellado')], neto_min, factor)}")
    if ("base:rrf", "base:sellado") in pares:
        L.append(f"- R2 (H7b, RRF): {'ADOPTAR RRF' if decide(pares[('base:rrf', 'base:sellado')], neto_min, factor) == 'GANA' else 'SE QUEDA LA FUSIÓN ACTUAL'}")
    if ("solape:sellado", "base:sellado") in pares:
        g = decide(pares[("solape:sellado", "base:sellado")], neto_min, factor)
        if g == "GANA" and "solape" in p95 and "base" in p95:
            ok = float(p95["solape"]) <= 1.25 * float(p95["base"])
            L.append(f"- R3 (H14a, solape): {'ADOPTAR SOLAPE' if ok else 'NO (guard de latencia)'} (p95 base={p95['base']} s, solape={p95['solape']} s)")
        else:
            L.append(f"- R3 (H14a, solape): {'GANA retrieval; FALTA p95' if g == 'GANA' else 'SE QUEDA EL TROCEADO ACTUAL'}")
    else:
        L.append("- R3 (H14a, solape): no medido")
    if ("late:sellado", "base:sellado") in pares:
        g = decide(pares[("late:sellado", "base:sellado")], neto_min, factor)
        if g == "GANA" and "late" in rebuild_s and "base" in rebuild_s:
            ok = float(rebuild_s["late"]) <= 3 * float(rebuild_s["base"])
            L.append(f"- R4 (H14b, late): {'GANA → spec de producción aparte' if ok else 'NO (guard de coste de rebuild)'}")
        else:
            L.append(f"- R4 (H14b, late): {'GANA retrieval; FALTA rebuild_s' if g == 'GANA' else 'NO GANA'}")
    else:
        L.append("- R4 (H14b, late): no medido")
    if ruta_detalle:
        with open(ruta_detalle, "w", encoding="utf-8") as fh:
            for d in detalle.values():
                fh.write(json.dumps(d, ensure_ascii=False) + "\n")
    return "\n".join(L) + "\n"


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    a_fid = sub.add_parser("fidelidad")
    a_fid.add_argument("--captura", required=True)
    a_fid.add_argument("--brazo", default="sellado", choices=["sellado", "rrf"])
    a_inf = sub.add_parser("informe")
    a_inf.add_argument("--gold", required=True)
    a_inf.add_argument("--overlay")
    a_inf.add_argument("--captura", action="append", required=True, help="indice=ruta (base|solape|late)")
    a_inf.add_argument("--neto-min", type=int, required=True)
    a_inf.add_argument("--factor", type=float, required=True)
    a_inf.add_argument("--estricto", action="store_true")
    a_inf.add_argument("--p95", action="append", help="indice=segundos")
    a_inf.add_argument("--rebuild-s", action="append", help="indice=segundos")
    a_inf.add_argument("--detalle")
    a_cmp = sub.add_parser("compara")
    a_cmp.add_argument("--a", required=True)
    a_cmp.add_argument("--b", required=True)
    a = ap.parse_args()

    if a.cmd == "fidelidad":
        cap = carga_captura(a.captura)
        malas = fidelidad(cap, a.brazo)
        print(f"fidelidad({a.brazo}): {len(malas)} discrepancias de {len(cap)}")
        for i in malas:
            print(f"  discrepa: {i}", file=sys.stderr)
        sys.exit(1 if malas else 0)
    if a.cmd == "compara":
        ca, cb = carga_captura(a.a), carga_captura(a.b)
        distintas = sorted(i for i in set(ca) | set(cb) if ca.get(i) != cb.get(i))
        print(f"compara: {len(distintas)} queries con capturas distintas de {len(set(ca) | set(cb))}")
        sys.exit(1 if distintas else 0)
    gold = carga_gold(a.gold)
    if a.overlay:
        gold = aplica_overlay(gold, a.overlay)
    capturas = {k: carga_captura(v) for k, v in _pares_kv(a.captura).items()}
    sys.stdout.write(informe(gold, capturas, a.neto_min, a.factor, a.estricto,
                             _pares_kv(a.p95), _pares_kv(a.rebuild_s), a.detalle))


if __name__ == "__main__":
    main()
