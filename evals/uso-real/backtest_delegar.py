"""Puerta de reproducción del detector delegar-decision sobre las sesiones selladas.

Uso: python3 evals/uso-real/backtest_delegar.py [--datos DIR] [--proyectos DIR] [--sello sello.json]
JSON en stdout; exit 0 si `puerta` es "ok", 3 si "falla". Los datos personales (muestra.json, ocho.json,
db/etiquetas) se leen de --datos (por defecto $EXO_DELEGAR_DATOS o ~/.cache/exo-delegar) y no salen en el JSON.
"""
import argparse
import glob
import importlib.util
import json
import os
import statistics
import sys

AQUI = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, AQUI)
import transcripts as T  # noqa: E402

_spec = importlib.util.spec_from_file_location(
    "delegar_decision", os.path.join(AQUI, "..", "..", "plugins", "exo", "scripts", "delegar-decision.py"))
D = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(D)

SESIONES = 99
BLOQUEOS = (236, 288)
RETENIDOS = (20, 24)
POSITIVOS = 17
OCHO_MIN = 7
POSITIVAS = {"delegar", "consultor"}


def segmentos(proyectos, sello):
    """Genera (sesion, indice, clave, dueño, evalua) por cada segmento no vacío de cada sesión sellada.

    `dueño` = último mensaje humano que precede al segmento (el que lee el sentinel del hook): los
    segmentos abiertos por un hand-back comparten dueño con el humano previo.
    """
    desde, hasta = T.cortes(sello)
    ses, _ = T.poblacion(proyectos, desde, hasta)
    paths = {os.path.basename(p)[:-6]: p for p in glob.glob(os.path.join(proyectos, "*", "*.jsonl"))}
    tope = D._epoch(hasta) if hasta else None
    for s in ses:
        dueño = None
        for i, (clave, bloques) in enumerate(D.segmentar(T._lineas(paths[s["id"]]), tope)):
            yield s["id"], i, clave, dueño, D.evalua(bloques)
            if clave and clave != "NOHUMANO":
                dueño = clave


def id_segmento(sesion, i, clave):
    return clave if clave and clave != "NOHUMANO" else f"{sesion}#{i}"


def segmentos_por_id(proyectos, sello, ids):
    ids = set(ids)
    out = {}
    for sesion, i, clave, _, r in segmentos(proyectos, sello):
        sid = id_segmento(sesion, i, clave)
        if sid in ids:
            out[sid] = r
    return out


def resumen(xs):
    return {"media": round(statistics.mean(xs), 2), "mediana": statistics.median(xs), "max": max(xs)}


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--datos", default=os.environ.get("EXO_DELEGAR_DATOS") or os.path.expanduser("~/.cache/exo-delegar"))
    ap.add_argument("--proyectos", default=os.path.expanduser("~/.claude/projects"))
    ap.add_argument("--sello", default=os.path.join(AQUI, "sello.json"))
    a = ap.parse_args(argv)

    muestra = json.load(open(os.path.join(a.datos, "muestra.json")))["muestra"]
    ocho = set(json.load(open(os.path.join(a.datos, "ocho.json"))))
    etiqueta = {}
    for f in glob.glob(os.path.join(a.datos, "db", "etiquetas", "*.json")):
        d = json.load(open(f))
        etiqueta[d["id"]] = d["etiqueta"]

    sesiones, bloqueos, banda, por_id = {}, {}, {}, {}
    for sesion, i, clave, dueño, r in segmentos(a.proyectos, a.sello):
        sesiones.setdefault(sesion, None)
        bloqueos.setdefault(sesion, set())
        banda.setdefault(sesion, 0)
        por_id[id_segmento(sesion, i, clave)] = r
        if r["dispara"]:
            bloqueos[sesion].add(dueño)
        banda[sesion] += r["kind"] == "banda"

    cuenta = [len(v) for v in bloqueos.values()] or [0]
    retenidos = [m for m in muestra if por_id.get(m, {}).get("dispara")]
    out = {
        "sesiones": len(sesiones),
        "bloqueos": sum(cuenta),
        "por_sesion": resumen(cuenta),
        "muestra": {"positivos_retenidos": sum(etiqueta.get(m) in POSITIVAS for m in retenidos),
                    "retenidos": len(retenidos)},
        "ocho": {"disparan": sum(bool(por_id.get(o, {}).get("dispara")) for o in ocho)},
        "banda_base": dict(resumen(list(banda.values()) or [0]), total=sum(banda.values())),
    }
    ok = (out["sesiones"] == SESIONES and BLOQUEOS[0] <= out["bloqueos"] <= BLOQUEOS[1]
          and out["muestra"]["positivos_retenidos"] == POSITIVOS
          and RETENIDOS[0] <= out["muestra"]["retenidos"] <= RETENIDOS[1]
          and out["ocho"]["disparan"] >= OCHO_MIN)
    out["puerta"] = "ok" if ok else "falla"
    print(json.dumps(out, ensure_ascii=False, indent=1))
    return 0 if ok else 3


if __name__ == "__main__":
    sys.exit(main())
