#!/usr/bin/env python3
"""Campaña M — análisis de la etapa 1 (spec §4.4–§4.6). Escrito ANTES de ver resultados.

Uso: analizar.py <K_ROOT> <sondas.tsv> <neutras.txt>  → <K_ROOT>/analisis.json + tabla en stdout.
Brazos: a0 (A0), ar (O), a2 (X), n (N). Solo cuentan las corridas de <K_ROOT>/corridas/.
"""
import collections
import glob
import json
import math
import os
import random
import re
import statistics
import sys

SEMILLA, REPS = 20261024, 10_000
MARGEN = 0.15
PREVALENCIA = 0.012  # cota inferior (spec §4.5)
MOTIVOS_ESCRITURA = ("escribe en el snapshot de la KB", "escribe en la memoria nativa")
BRAZOS = ("a0", "ar", "a2", "n")


def exito_por_sonda(rcs):
    return rcs.count(0) / len(rcs) if rcs else 0.0


def _deltas(a, b):
    return {k: a[k] - b[k] for k in sorted(set(a) & set(b))}


def delta_bootstrap(a, b, semilla=SEMILLA, n=REPS):
    d = list(_deltas(a, b).values())
    if not d:
        return None
    rng = random.Random(semilla)
    k = len(d)
    medias = sorted(sum(d[rng.randrange(k)] for _ in range(k)) / k for _ in range(n))
    return (sum(d) / k, medias[int(0.025 * n)], medias[int(0.975 * n) - 1])


def signos(a, b):
    d = _deltas(a, b).values()
    pos, neg = sum(x > 0 for x in d), sum(x < 0 for x in d)
    m = pos + neg
    if m == 0:
        return pos, neg, 1.0
    p = sum(math.comb(m, i) for i in range(min(pos, neg) + 1)) / 2 ** m
    return pos, neg, min(1.0, 2 * p)


def cociente_R(a0, ar, x):
    return None if ar <= a0 else (x - a0) / (ar - a0)


def _domina(x, y):
    if None in (x["valor"], y["valor"], x["sobrecarga_bytes"], y["sobrecarga_bytes"]):
        return False
    return (x["valor"] > y["valor"] and x["sobrecarga_bytes"] <= y["sobrecarga_bytes"]) or \
           (x["valor"] >= y["valor"] and x["sobrecarga_bytes"] < y["sobrecarga_bytes"])


def _desempate(m):
    de = m["desempate"]
    quien = "n" if _domina(de["n"], de["a2"]) else "a2" if _domina(de["a2"], de["n"]) else "ninguno"
    return f"DESEMPATE: {quien}"


def veredicto(m):
    if m["frac_sucia"] > 0.05:
        return ["NO ADJUDICABLE"]
    sin_datos = []
    ok = [b for b in ("a2", "n") if m["neutro"][b] is not None]
    if len(ok) < 2:
        sin_datos.append("SIN DATOS R3")
    dano = [f"DAÑO {b}" for b in ok if not m["neutro"][b]["lo"] > -MARGEN]
    if m["oraculo_ok"] * 10 < 3 * m["n_sondas"]:
        return ["CUELLO = CUMPLIMIENTO"] + dano + sin_datos
    if m["d_a2_a0"] is None:
        return ["SIN DATOS X-A0"] + dano + sin_datos
    if m["d_n_a0"] is None:
        sin_datos.append("SIN DATOS N-A0")
    if not m["d_a2_a0"]["lo"] > 0:
        if m["d_n_a0"] is not None and not m["d_n_a0"]["lo"] > 0:
            return ["NI X NI N DEMOSTRARON EFECTO", "RECOMENDACIÓN: memoria mínima", _desempate(m)] + dano + sin_datos
        return ["SOLO COSTE"] + dano + sin_datos
    tk = m["tokens_mediana"]
    if m["d_n_a2"] is None:
        sin_datos.append("SIN DATOS R1")
    if m["d_a2_n"] is None:
        sin_datos.append("SIN DATOS R2")
    r1 = m["d_n_a2"] is not None and m["d_n_a2"]["lo"] > -MARGEN and tk["n"] < tk["a2"]
    r2 = m["d_a2_n"] is not None and m["d_a2_n"]["lo"] > 0 and m["d_a2_n"]["media"] >= MARGEN
    if r1 or r2 or dano:
        return (["GANA N"] if r1 else []) + (["EXO SOBREVIVE"] if r2 else []) + dano + sin_datos
    return ["NO CONCLUYENTE", _desempate(m)] + sin_datos


def _leer(path):
    try:
        return open(path).read()
    except OSError:
        return ""


def _corrida(d):
    try:
        meta = json.load(open(f"{d}/meta.json"))
    except (OSError, ValueError):
        meta = {}
    rc = _leer(f"{d}/check.rc").strip()
    # Cortada, con error o sin check = fallo (E3 de K), nunca descarte
    completa = meta.get("fin") == "completed" and "error" not in meta and rc.isdigit()
    try:
        motivos = json.load(open(f"{d}/fugas.json")).get("motivos", [])
    except (OSError, ValueError):
        motivos = []
    modelo = None
    try:
        with open(f"{d}/transcript.jsonl") as f:
            for linea in f:
                ev = json.loads(linea)
                if ev.get("type") == "system" and ev.get("subtype") == "init":
                    modelo = ev.get("model")
                    break
    except (OSError, ValueError):
        pass
    ins = f"{d}/inyectado.txt"
    return {
        "rc": int(rc) if completa else 1,
        "tokens": meta.get("tokens_in"),
        "modelo": modelo,
        "version": _leer(f"{d}/claude-version.txt").split(" ")[0].strip(),
        "sucia": bool(_leer(f"{d}/kb-sucia.txt").strip())
        or any(w in x for x in motivos for w in MOTIVOS_ESCRITURA),
        "bytes": os.path.getsize(ins) if os.path.exists(ins) else None,
    }


def _congelado(k_root):
    m = re.search(r"^modelo (\S+) · claude (\S+)", _leer(f"{k_root}/congelacion-principal.txt"), re.M)
    return (m.group(1), m.group(2)) if m else (None, None)


def _por_brazo(runs, brazos):
    out = {b: collections.defaultdict(list) for b in brazos}
    for r in runs:
        if r["brazo"] in out and not r["excluida"]:
            out[r["brazo"]][r["tarea"]].append(r)
    return out


def _exito(pb):
    return {b: {t: exito_por_sonda([r["rc"] for r in rs]) for t, rs in ts.items()} for b, ts in pb.items()}


def _media(d):
    return sum(d.values()) / len(d) if d else 0.0


def _ci(a, b, semilla):
    r = delta_bootstrap(a, b, semilla)
    if r is None:
        return None
    media, lo, hi = r
    return {"media": media, "lo": lo, "hi": hi}


def _tokens_sonda(pb_brazo):
    out = {}
    for t, rs in pb_brazo.items():
        tk = [r["tokens"] for r in rs if r["tokens"] is not None]
        if tk:
            out[t] = sum(tk) / len(tk)
    return out


def _coste(a, b):
    d = _deltas(a, b)
    return {"d_tokens": statistics.median(d.values()) if d else None, "n_sondas": len(d)}


def analizar(k_root, sondas_path, neutras_path, semilla=SEMILLA):
    tier = dict(l.split("\t")[:2] for l in _leer(sondas_path).splitlines() if l.strip())
    neutras = [l.strip() for l in _leer(neutras_path).splitlines() if l.strip()]
    modelo_f, version_f = _congelado(k_root)

    runs = []
    for tarea, brazos in [(t, BRAZOS) for t in tier] + [(t, ("a0", "a2", "n")) for t in neutras]:
        for d in sorted(glob.glob(f"{k_root}/corridas/{tarea}/*-r*")):
            brazo, rep = os.path.basename(d).rsplit("-r", 1)
            if brazo not in brazos:
                continue
            r = _corrida(d)
            motivo = None
            if r["sucia"]:
                motivo = "escritura fuera de lo previsto"
            elif version_f and r["version"] != version_f:
                motivo = "versión de claude distinta de la congelada"
            elif modelo_f and r["modelo"] and r["modelo"] != modelo_f:
                motivo = "modelo distinto del congelado"
            r.update(tarea=tarea, brazo=brazo, excluida=motivo is not None, motivo=motivo,
                     corrida=f"{tarea}/{brazo}-r{rep}")
            runs.append(r)

    suelo = _por_brazo([r for r in runs if r["tarea"] in tier], BRAZOS)
    neutro = _por_brazo([r for r in runs if r["tarea"] not in tier], ("a0", "a2", "n"))
    ex, exn = _exito(suelo), _exito(neutro)

    d_n_a2 = _ci(ex["n"], ex["a2"], semilla)
    d_a2_n = _ci(ex["a2"], ex["n"], semilla)
    d_a2_a0 = _ci(ex["a2"], ex["a0"], semilla)
    d_n_a0 = _ci(ex["n"], ex["a0"], semilla)
    neu = {b: _ci(exn[b], exn["a0"], semilla) for b in ("a2", "n")}

    por_tier = {}
    for k, d in _deltas(ex["n"], ex["a2"]).items():
        t = por_tier.setdefault(tier[k], {"n": 0, "media": 0.0})
        t["media"] = (t["media"] * t["n"] + d) / (t["n"] + 1)
        t["n"] += 1

    mu = {b: _media(ex[b]) for b in BRAZOS}
    R = {b: cociente_R(mu["a0"], mu["ar"], mu[b]) for b in ("a2", "n")}

    tk = {b: _tokens_sonda({**suelo[b], **neutro[b]}) for b in ("a0", "a2", "n")}
    tkn = {b: _tokens_sonda(neutro[b]) for b in ("a0", "a2", "n")}
    coste = {b: _coste(tk[b], tk["a0"]) for b in ("a2", "n")}
    coste_neutro = {b: _coste(tkn[b], tkn["a0"]) for b in ("a2", "n")}

    sobrecarga = {}
    for b in ("ar", "a2", "n"):
        bs = [r["bytes"] for pb in (suelo, neutro) for rs in pb.get(b, {}).values()
              for r in rs if r["bytes"] is not None]
        if bs:
            sobrecarga[b] = statistics.median(bs)
    desempate = {b: {"valor": None if dd is None else PREVALENCIA * dd["media"], "sobrecarga_bytes": sobrecarga.get(b)}
                 for b, dd in (("a2", d_a2_a0), ("n", d_n_a0))}

    tmed = {b: statistics.median([r["tokens"] for rs in suelo[b].values() for r in rs
                                  if r["tokens"] is not None] or [0]) for b in ("a2", "n")}
    excluidas = [{"corrida": r["corrida"], "motivo": r["motivo"]} for r in runs if r["excluida"]]
    oraculo_ok = sum(v >= 2 / 3 - 1e-9 for v in ex["ar"].values())
    frac_sucia = sum(r["sucia"] for r in runs) / len(runs) if runs else 0.0

    m = {"n_sondas": len(tier), "oraculo_ok": oraculo_ok, "frac_sucia": frac_sucia,
         "d_n_a2": d_n_a2, "d_a2_n": d_a2_n, "d_a2_a0": d_a2_a0, "d_n_a0": d_n_a0, "tokens_mediana": tmed,
         "neutro": neu, "desempate": desempate}
    return {
        "n_sondas": len(tier), "n_neutras": len(neutras), "n_corridas": len(runs),
        "exito": mu, "exito_sonda": ex, "exito_neutro": {b: _media(exn[b]) for b in exn},
        "oraculo_ok": oraculo_ok, "frac_sucia": frac_sucia,
        "d_n_a2": d_n_a2, "d_a2_n": d_a2_n, "d_a2_a0": d_a2_a0, "d_n_a0": d_n_a0,
        "signos_n_a2": signos(ex["n"], ex["a2"]), "por_tier": por_tier,
        "neutro": neu, "R": R, "coste": coste, "coste_neutro": coste_neutro,
        "sobrecarga_bytes": sobrecarga, "tokens_mediana": tmed, "desempate": desempate,
        "excluidas": excluidas, "veredicto": veredicto(m),
    }


def main(argv):
    k, sp, npth = argv
    k = os.path.expanduser(k)
    r = analizar(k, sp, npth)
    json.dump(r, open(f"{k}/analisis.json", "w"), indent=1, ensure_ascii=False)
    print("| contraste | Δ medio | IC95 |")
    print("|---|---|---|")
    for nombre in ("d_n_a2", "d_a2_n", "d_a2_a0", "d_n_a0"):
        c = r[nombre]
        if c is None:
            print(f"| {nombre} | sin datos | |")
            continue
        print(f"| {nombre} | {c['media']:+.3f} | [{c['lo']:+.3f}, {c['hi']:+.3f}] |")
    for b, c in r["neutro"].items():
        if c is None:
            print(f"| neutro {b}-a0 | sin datos | |")
            continue
        print(f"| neutro {b}-a0 | {c['media']:+.3f} | [{c['lo']:+.3f}, {c['hi']:+.3f}] |")
    print(f"\nO >= 2/3: {r['oraculo_ok']}/{r['n_sondas']} · sucias {r['frac_sucia']:.1%} · "
          f"excluidas {len(r['excluidas'])}")
    print("veredicto:", ", ".join(r["veredicto"]))


if __name__ == "__main__":
    main(sys.argv[1:])
