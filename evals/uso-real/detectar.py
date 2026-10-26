#!/usr/bin/env python3
"""Etapa 1 (spec 2026-10-26 §4.2): detecta correcciones del dueño en lotes de 10 mensajes con un juez Haiku aislado.

Salida pública (--salida): sesion, id y veredicto. El texto de los mensajes va solo a --cache (repo público).
Códigos de salida: 0 termina, 1 entrada inválida, 2 el sello no casa, 3 tope de coste alcanzado,
4 fallo de proceso de claude (rc != 0, timeout o salida que no es el envelope JSON)."""
import argparse, json, os, re, sys

H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, H)
import juez, transcripts

PROMPT = os.path.join(H, "detector.prompt.md")
MODELO = "claude-haiku-5-5"
LOTE = 10
MAX_TEXTO = 2000  # principio del mensaje: la corrección suele ir al inicio
MAX_PREVIO = 4000  # final del turno del asistente
TIPOS = ("rechaza-propuesta", "contradice-hecho", "repite-instruccion", "redirige")


def valida(obj):
    """Esquema de etapa 1 o None. `tipo` es null exactamente cuando correccion es `no`."""
    if not isinstance(obj, dict) or not isinstance(obj.get("id"), str) or not obj["id"]:
        return None
    c, t = obj.get("correccion"), obj.get("tipo")
    if t == "null":
        t = None
    if c == "si" and t in TIPOS or c == "no" and t is None:
        return {"id": obj["id"], "correccion": c, "tipo": t}
    return None


def construye_entrada(lote, prompt):
    lineas = "\n".join("MSG " + json.dumps({"id": m["id"], "previo": m["previo"][-MAX_PREVIO:], "texto": m["texto"][:MAX_TEXTO]},
                                           ensure_ascii=False) for m in lote)
    return re.sub(r"\{LOTE\}", lambda _: lineas, prompt)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    for a in ("proyectos", "cache", "salida"):
        ap.add_argument("--" + a, required=True)
    ap.add_argument("--sello")
    ap.add_argument("--tope-usd", type=float, default=3)
    ap.add_argument("--por-llamada-usd", default="0.10")
    ap.add_argument("--desde", default="2026-10-06")
    ap.add_argument("--solo-sesion")
    a = ap.parse_args()

    if a.sello and a.solo_sesion:
        print("detectar: --solo-sesion es incompatible con --sello", file=sys.stderr)
        return 1
    if a.sello:
        with open(a.sello) as f:
            sello = json.load(f)
        actual = {"prompt_detector_sha256": juez.sha256_fichero(PROMPT), "claude_version": juez.claude_version()}
        malos = [k for k, v in actual.items() if sello.get(k) != v]
        if malos:
            print(f"detectar: el sello no casa en {malos}", file=sys.stderr)
            return 2

    sesiones, _ = transcripts.poblacion(a.proyectos, a.desde)
    if a.solo_sesion:
        sesiones = [s for s in sesiones if s["id"] == a.solo_sesion]
    # Orden determinista: sesión (poblacion ya ordena por id) y luego ts, para que cache y lotes sean reproducibles.
    msgs = [{**m, "sesion": s["id"]} for s in sesiones for m in sorted(s["mensajes"], key=lambda m: m["ts"] or "")]

    with open(PROMPT) as f:
        prompt = f.read()
    sha = juez.sha256_fichero(PROMPT)
    cache = juez.Cache(os.path.join(a.cache, "detector.jsonl"))
    res = {}  # id -> veredicto | None (sin_veredicto)
    for m in msgs:
        c = cache.get(m["id"])
        if c is not None and c.get("prompt_sha256") == sha:
            res[m["id"]] = valida(c)
    pendientes = [m for m in msgs if m["id"] not in res]

    parcial, total = a.salida + ".parcial", 0.0

    def vuelca(destino):
        with open(destino, "w") as pub:
            for m in msgs:
                if m["id"] not in res:
                    continue
                v = res[m["id"]]
                o = {"sesion": m["sesion"], "id": m["id"], "sin_veredicto": True} if v is None else \
                    {"sesion": m["sesion"], "id": m["id"], "correccion": v["correccion"], "tipo": v["tipo"]}
                pub.write(json.dumps(o) + "\n")

    def termina(rc, aviso=None):
        if aviso:
            print(aviso, file=sys.stderr)
        vuelca(parcial)
        print(f"coste_total_usd={total:g}")
        print(f"n_sesiones={len(sesiones)}")
        return rc

    if os.path.exists(a.salida):
        os.remove(a.salida)
    for i in range(0, len(pendientes), LOTE):
        lote = pendientes[i:i + LOTE]
        for _ in range(2):  # una llamada y un reintento solo con los ids que faltan
            try:
                texto, coste = juez.llama(construye_entrada(lote, prompt), MODELO, a.por_llamada_usd)
            except juez.FalloProceso as e:
                return termina(4, f"detectar: {e}; queda {parcial}")
            total += coste
            for i_, v in juez.parsea_lote(texto, [m["id"] for m in lote], valida).items():
                m = next(m for m in lote if m["id"] == i_)
                res[i_] = v
                cache.put(i_, {**v, "sesion": m["sesion"], "texto": m["texto"], "previo": m["previo"], "prompt_sha256": sha})
            lote = [m for m in lote if m["id"] not in res]
            if total >= a.tope_usd:
                return termina(3, f"detectar: tope de {a.tope_usd} USD alcanzado ({total:.2f}); queda {parcial}")
            if not lote:
                break
        for m in lote:
            res[m["id"]] = None
    vuelca(parcial)
    os.replace(parcial, a.salida)
    print(f"coste_total_usd={total:g}")
    print(f"n_sesiones={len(sesiones)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
