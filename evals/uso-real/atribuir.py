#!/usr/bin/env python3
"""Etapa 2 (spec 2026-10-26 §4.4): atribuye cada corrección positiva de la etapa 1 con un juez Sonnet aislado.

Un positivo por llamada. Contexto: reglas del system prompt de la sesión, nota de proyecto, `core/` y mensajes previos,
con la KB (--kb o $EXO_KB) en el último commit anterior al inicio de la sesión.
Salida pública (--salida): sesion, id, atribucion, nota_ausente y commit_kb. El texto, `nota` y `linea` van solo a --cache.
Códigos de salida: 0 termina, 1 entrada inválida, 2 el sello no casa, 3 tope de coste alcanzado,
4 fallo de proceso de claude (rc != 0, timeout o salida que no es el envelope JSON)."""
import argparse, json, os, re, sys

H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, H)
import juez, kb, transcripts

PROMPT = os.path.join(H, "atribuidor.prompt.md")
MODELO = "claude-sonnet-5-5"
MAX_ANTERIORES = 8000  # se conserva el final: lo más cercano al mensaje
MAX_TEXTO = 2000
MAX_PREVIO = 4000
ATRIBUCIONES = ("regla-canal", "regla-fuera-canal", "canon-kb", "perfil-core", "dicho-en-sesion", "nueva")


def _cita(v):
    return None if v is None or v == "null" else v if isinstance(v, str) else False


def valida(obj):
    """Esquema de etapa 2 (sin id: un positivo por llamada) o None."""
    if not isinstance(obj, dict) or obj.get("atribucion") not in ATRIBUCIONES:
        return None
    nota, linea = _cita(obj.get("nota")), _cita(obj.get("linea"))
    if nota is False or linea is False:
        return None
    return {"atribucion": obj["atribucion"], "nota": nota, "linea": linea}


def construye_entrada(msg, anteriores, ctx, reglas_canal, prompt):
    sec = ["## Reglas en el system prompt de esta sesión", reglas_canal or "ninguna",
           "## Nota de proyecto en la fecha de la sesión",
           ctx["nota"] if ctx["nota"] is not None else "ausente"]
    for ruta, texto in sorted(ctx["core"].items()):
        sec += [f"## {ruta}", texto]
    sec += ["## Mensajes anteriores del dueño en esta sesión", "\n".join(anteriores)[-MAX_ANTERIORES:] or "ninguno",
            "## Mensaje a atribuir",
            "MSG " + json.dumps({"id": msg["id"], "previo": msg["previo"][-MAX_PREVIO:], "texto": msg["texto"][:MAX_TEXTO]},
                                ensure_ascii=False)]
    return re.sub(r"\{CONTEXTO\}", lambda _: "\n\n".join(sec), prompt)


def parsea(res):
    if not isinstance(res, str):
        return None
    t = res.strip()
    m = re.search(r"```(?:json)?\s*(.*?)```", t, re.S)
    if m:
        t = m.group(1).strip()
    try:
        return valida(json.loads(t))
    except ValueError:
        return None


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    for a in ("proyectos", "detecciones", "cache", "salida"):
        ap.add_argument("--" + a, required=True)
    ap.add_argument("--kb", default=os.environ.get("EXO_KB"))
    ap.add_argument("--sello")
    ap.add_argument("--tope-usd", type=float, default=25)
    ap.add_argument("--por-llamada-usd", default="0.50")
    ap.add_argument("--desde", default="2026-10-06")
    a = ap.parse_args()
    if not a.kb:
        print("atribuir: falta --kb (o $EXO_KB)", file=sys.stderr)
        return 1

    if a.sello:
        with open(a.sello) as f:
            sello = json.load(f)
        actual = {"prompt_atribuidor_sha256": juez.sha256_fichero(PROMPT), "claude_version": juez.claude_version()}
        malos = [k for k, v in actual.items() if sello.get(k) != v]
        if malos:
            print(f"atribuir: el sello no casa en {malos}", file=sys.stderr)
            return 2

    desde, hasta = transcripts.cortes(a.sello, a.desde)
    sesiones = {s["id"]: s for s in transcripts.poblacion(a.proyectos, desde, hasta)[0]}
    positivos = []
    with open(a.detecciones) as f:
        for l in f:
            d = json.loads(l)
            if d.get("correccion") != "si":
                continue
            s = sesiones.get(d["sesion"])
            if s is None or d["id"] not in {m["id"] for m in s["mensajes"]}:
                print(f"atribuir: la detección {d['sesion']}/{d['id']} no está en la población", file=sys.stderr)
                return 1
            positivos.append(d)

    with open(PROMPT) as f:
        prompt = f.read()
    sha = juez.sha256_fichero(PROMPT)
    cache = juez.Cache(os.path.join(a.cache, "atribuidor.jsonl"))
    res = {}  # id -> registro de cache | None (sin_veredicto)
    for d in positivos:
        c = cache.get(d["id"])
        if c is not None and c.get("prompt_sha256") == sha and valida(c):
            res[d["id"]] = c

    parcial, total = a.salida + ".parcial", 0.0

    def vuelca(destino):
        with open(destino, "w") as pub:
            for d in positivos:
                if d["id"] not in res:
                    continue
                c = res[d["id"]]
                o = {"sesion": d["sesion"], "id": d["id"], "sin_veredicto": True} if c is None else \
                    {"sesion": d["sesion"], "id": d["id"], "atribucion": c["atribucion"],
                     "nota_ausente": c["nota_ausente"], "commit_kb": c["commit_kb"]}
                pub.write(json.dumps(o) + "\n")

    def termina(rc, aviso=None):
        if aviso:
            print(aviso, file=sys.stderr)
        vuelca(parcial)
        print(f"coste_total_usd={total:g}")
        print(f"n_positivos={len(positivos)}")
        return rc

    kb_sesion = {}  # sesion -> (commit, ctx)

    def contexto(s):
        if s["id"] not in kb_sesion:
            commit = kb.commit_antes(a.kb, s["inicio"])
            if commit is None:
                ctx = {"nota": None, "reglas_globales": None, "core": {}, "nota_ausente": True}
            else:
                head = kb.nota_head(s["cwd"], a.kb) if s["cwd"] else None
                ruta = kb.ruta_en(a.kb, head, commit) if head else None
                ctx = kb.contexto(a.kb, commit, ruta)
            kb_sesion[s["id"]] = (commit, ctx)
        return kb_sesion[s["id"]]

    if os.path.exists(a.salida):
        os.remove(a.salida)
    for d in positivos:
        if d["id"] in res:
            continue
        s = sesiones[d["sesion"]]
        msgs = sorted(s["mensajes"], key=lambda m: m["ts"] or "")
        k = next(i for i, m in enumerate(msgs) if m["id"] == d["id"])
        commit, ctx = contexto(s)
        entrada = construye_entrada(msgs[k], [m["texto"] for m in msgs[:k]], ctx, s["reglas_canal"], prompt)
        v = None
        for _ in range(2):  # una llamada y un reintento si la salida no valida
            try:
                texto, coste = juez.llama(entrada, MODELO, a.por_llamada_usd)
            except juez.FalloProceso as e:
                return termina(4, f"atribuir: {e}; queda {parcial}")
            total += coste
            v = parsea(texto)
            if v is not None:
                c = {**v, "sesion": s["id"], "texto": msgs[k]["texto"], "nota_ausente": ctx["nota_ausente"],
                     "commit_kb": commit, "prompt_sha256": sha, "id": d["id"]}
                cache.put(d["id"], c)
                res[d["id"]] = c
            if total >= a.tope_usd:
                if v is None:
                    res[d["id"]] = None
                return termina(3, f"atribuir: tope de {a.tope_usd} USD alcanzado ({total:.2f}); queda {parcial}")
            if v is not None:
                break
        if v is None:
            res[d["id"]] = None
    vuelca(parcial)
    os.replace(parcial, a.salida)
    print(f"coste_total_usd={total:g}")
    print(f"n_positivos={len(positivos)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
