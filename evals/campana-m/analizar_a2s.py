#!/usr/bin/env python3
"""Campaña M — brazo a2s (X + reflejo search-first), enmienda 1 del pre-registro. Escrito ANTES de ver resultados.

Uso: analizar_a2s.py <K_ROOT> <sondas.tsv>  → <K_ROOT>/analisis-a2s.json + resumen en stdout.
a2s sale de <K_ROOT>/corridas-a2s/; a0, ar y a2 de <K_ROOT>/corridas-principal/ (mismas 40 sondas × 3).
Condición pre-registrada (verdict §7.3): busca en ≥ 50 % de las corridas Y límite inferior de Δ(a2s − a0) > 0.
"""
import json
import os
import re
import statistics
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from analizar import SEMILLA, _ci, _congelado, _corrida, _deltas, exito_por_sonda, signos  # noqa: E402

UMBRAL_BUSCA = 0.5
# Mismo patrón que search-first.sh, más `recall` (también recupera de la KB).
BUSCA = re.compile(r"(^|[;&|/\s])exo(\.exe)?\s+(search|targets|recall)(\s|$)")


def busca(transcript):
    try:
        with open(transcript, encoding="utf-8") as f:
            for linea in f:
                try:
                    ev = json.loads(linea)
                except ValueError:
                    continue
                if ev.get("type") != "assistant":
                    continue
                for b in (ev.get("message") or {}).get("content") or []:
                    if b.get("type") == "tool_use" and b.get("name") == "Bash" \
                            and BUSCA.search((b.get("input") or {}).get("command", "")):
                        return True
    except OSError:
        pass
    return False


def aviso(reflex_log):
    try:
        return any(json.loads(l).get("reflex") == "search-first" for l in open(reflex_log, encoding="utf-8") if l.strip())
    except (OSError, ValueError):
        return False


def veredicto(m):
    if m["frac_sucia"] > 0.05:
        return ["NO ADJUDICABLE"]
    if m["d_a2s_a0"] is None:
        return ["SIN DATOS a2s-a0"]
    efecto = m["d_a2s_a0"]["lo"] > 0
    if m["frac_busca"] >= UMBRAL_BUSCA and efecto:
        return ["EXO CON SEARCH-FIRST DEMOSTRÓ EFECTO", "RECOMENDACIÓN 1 INVALIDADA"]
    if m["frac_busca"] >= UMBRAL_BUSCA:
        return ["BUSCA Y NO APLICA", "MEMORIA MÍNIMA CONFIRMADA"]
    return ["EFECTO SIN BÚSQUEDA" if efecto else "EL REFLEJO NO HACE BUSCAR", "MEMORIA MÍNIMA CONFIRMADA"]


def _leer_brazo(base, tareas, brazo, version_f, modelo_f):
    out, runs = {}, []
    for t in tareas:
        for rep in (1, 2, 3):
            d = f"{base}/{t}/{brazo}-r{rep}"
            if not os.path.isdir(d):
                continue
            r = _corrida(d)
            motivo = "escritura fuera de lo previsto" if r["sucia"] else \
                "versión de claude distinta de la congelada" if version_f and r["version"] != version_f else \
                "modelo distinto del congelado" if modelo_f and r["modelo"] and r["modelo"] != modelo_f else None
            r.update(tarea=t, corrida=f"{t}/{brazo}-r{rep}", motivo=motivo,
                     busca=busca(f"{d}/transcript.jsonl"), aviso=aviso(f"{d}/reflex.jsonl"))
            runs.append(r)
            if motivo is None:
                out.setdefault(t, []).append(r)
    return out, runs


def analizar(k_root, sondas_path, semilla=SEMILLA):
    tareas = [l.split("\t")[0] for l in open(sondas_path).read().splitlines() if l.strip()]
    modelo_f, version_f = _congelado(k_root)
    pb, todas = {}, {}
    for brazo, carpeta in (("a2s", "corridas-a2s"), ("a0", "corridas-principal"),
                           ("ar", "corridas-principal"), ("a2", "corridas-principal")):
        pb[brazo], todas[brazo] = _leer_brazo(f"{k_root}/{carpeta}", tareas, brazo, version_f, modelo_f)
    ex = {b: {t: exito_por_sonda([r["rc"] for r in rs]) for t, rs in pb[b].items()} for b in pb}
    runs = todas["a2s"]
    n = len(runs) or 1
    tk = {b: {t: statistics.mean([r["tokens"] for r in rs if r["tokens"] is not None])
              for t, rs in pb[b].items() if any(r["tokens"] is not None for r in rs)} for b in ("a0", "a2", "a2s")}
    dtk = {b: list(_deltas(tk[b], tk["a0"]).values()) for b in ("a2", "a2s")}
    bus = [r for r in runs if r["busca"]]
    m = {
        "n_corridas": len(runs), "n_sondas": len(ex["a2s"]),
        "frac_sucia": sum(r["sucia"] for r in runs) / n,
        "excluidas": [{"corrida": r["corrida"], "motivo": r["motivo"]} for r in runs if r["motivo"]],
        "frac_busca": len(bus) / n, "frac_aviso": sum(r["aviso"] for r in runs) / n,
        "exito_si_busca": statistics.mean([r["rc"] == 0 for r in bus]) if bus else None,
        "exito_media": {b: (sum(v.values()) / len(v) if v else None) for b, v in ex.items()},
        "d_a2s_a0": _ci(ex["a2s"], ex["a0"], semilla), "d_a2s_a2": _ci(ex["a2s"], ex["a2"], semilla),
        "signos_a2s_a0": signos(ex["a2s"], ex["a0"]), "signos_a2s_a2": signos(ex["a2s"], ex["a2"]),
        "d_tokens_vs_a0": {b: {"media": statistics.mean(v), "mediana": statistics.median(v)} if v else None
                           for b, v in dtk.items()},
        "por_sonda": {t: {b: ex[b].get(t) for b in ex} for t in tareas},
    }
    m["veredicto"] = veredicto(m)
    return m


def main():
    k_root, sondas = sys.argv[1:3]
    m = analizar(k_root, sondas)
    with open(f"{k_root}/analisis-a2s.json", "w") as f:
        json.dump(m, f, ensure_ascii=False, indent=1)
    em = m["exito_media"]
    print(f"corridas a2s {m['n_corridas']} · sondas {m['n_sondas']} · sucias {m['frac_sucia']:.3f} · excluidas {len(m['excluidas'])}")
    print("éxito medio: " + " · ".join(f"{b} {v:.3f}" for b, v in em.items() if v is not None))
    print(f"busca {m['frac_busca']:.3f} · aviso emitido {m['frac_aviso']:.3f} · éxito si busca {m['exito_si_busca']}")
    for k in ("d_a2s_a0", "d_a2s_a2"):
        d = m[k]
        print(f"{k}: " + ("sin datos" if d is None else f"{d['media']:+.3f} IC95 [{d['lo']:+.3f}, {d['hi']:+.3f}]"))
    print(f"signos a2s-a0 {m['signos_a2s_a0']} · a2s-a2 {m['signos_a2s_a2']}")
    print(f"Δ tokens_in vs a0: {m['d_tokens_vs_a0']}")
    print("VEREDICTO: " + " · ".join(m["veredicto"]))


if __name__ == "__main__":
    main()
