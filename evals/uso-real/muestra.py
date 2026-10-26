#!/usr/bin/env python3
"""Muestras a ciegas para el dueño (spec 2026-10-26 §4.2 y §4.4).

Escribe `<cache>/etiquetar-etapa<k>.jsonl` (o `etiquetar-regex.jsonl`) con el texto y el previo tal como los vio
el juez (mismo truncado) y sin ningún veredicto. El texto va solo a la cache: el repo es público."""
import argparse, json, os, random, re, sys, unicodedata

H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, H)
import detectar, juez, kb, transcripts

SEMILLA = 20261026
MAX_ANTERIORES = 8000  # igual que atribuir.MAX_ANTERIORES (test_constantes_como_atribuir lo vigila cuando atribuir.py está en la rama)
REGEX = re.compile(r"no uses|otra vez|siempre|nunca|esta mal|dejate de")


def _normaliza(texto):
    return "".join(c for c in unicodedata.normalize("NFKD", texto.lower()) if not unicodedata.combining(c))


def candidato_regex(texto):
    return REGEX.search(_normaliza(texto)) is not None


def _item(m):
    return {"id": m["id"], "previo": m["previo"][-detectar.MAX_PREVIO:], "texto": m["texto"][:detectar.MAX_TEXTO],
            "etiqueta": None}


def etapa1(mensajes, semilla=SEMILLA):
    rng = random.Random(semilla)
    casan = [m for m in mensajes if candidato_regex(m["texto"])]
    resto = [m for m in mensajes if not candidato_regex(m["texto"])]
    pos = rng.sample(casan, min(40, len(casan)))
    azar = rng.sample(resto, min(80 - len(pos), len(resto)))
    sel = pos + azar
    rng.shuffle(sel)
    return [_item(m) for m in sel]


def _cuotas(cuentas, n):
    """Reparto proporcional con mínimo 1 por estrato presente y resto por mayor fracción."""
    total = sum(cuentas.values())
    n = min(n, total)
    exacto = {k: n * c / total for k, c in cuentas.items()}
    q = {k: min(cuentas[k], max(1, int(exacto[k]))) for k in cuentas}
    while sum(q.values()) < n:
        libres = [k for k in q if q[k] < cuentas[k]]
        q[max(libres, key=lambda k: (exacto[k] - q[k], k))] += 1
    while sum(q.values()) > n:
        q[max((k for k in q if q[k] > 1), key=lambda k: (q[k] - exacto[k], k))] -= 1
    return q


def etapa2(atribuidos, semilla=SEMILLA, n=30):
    rng = random.Random(semilla)
    por = {}
    for a in sorted(atribuidos, key=lambda a: a["id"]):
        por.setdefault(a["atribucion"], []).append(a)
    cuotas = _cuotas({k: len(v) for k, v in por.items()}, n)
    sel = [a for k in sorted(por) for a in rng.sample(por[k], cuotas[k])]
    rng.shuffle(sel)
    # reglas_canal, anteriores y contexto_kb son lo que vio el juez (atribuir.construye_entrada); `atribucion`, `nota` y `linea` son su veredicto y no salen.
    return [{**_item(a), "reglas_canal": a.get("reglas_canal"),
             "anteriores": a.get("anteriores"), "contexto_kb": a.get("contexto_kb"), "atribucion_dueño": None} for a in sel]


def _lee_jsonl(path):
    with open(path) as f:
        return [json.loads(l) for l in f if l.strip()]


def _contexto_kb(sesion, commit, kbdir):
    """Reconstruye lo que vio el juez de atribuir.py: nota de proyecto y core/ en `commit` (como atribuir.contexto)."""
    if commit is None:
        return {"nota": None, "core": {}}
    head = kb.nota_head(sesion["cwd"], kbdir) if sesion["cwd"] else None
    ctx = kb.contexto(kbdir, commit, kb.ruta_en(kbdir, head, commit) if head else None)
    return {"nota": ctx["nota"], "core": ctx["core"]}


def atribuidos(salida_atribuir, cache, proyectos, desde, kbdir):
    """Une la salida pública de atribuir.py, su cache (`atribuidor.jsonl`: texto) y el transcript (previo)."""
    sesiones = {s["id"]: s for s in transcripts.poblacion(proyectos, desde)[0]}
    c = juez.Cache(os.path.join(cache, "atribuidor.jsonl"))
    out = []
    for p in _lee_jsonl(salida_atribuir):
        if p.get("sin_veredicto"):
            continue
        s, reg = sesiones.get(p["sesion"]), c.get(p["id"])
        m = next((m for m in s["mensajes"] if m["id"] == p["id"]), None) if s else None
        if m is None or reg is None:
            raise ValueError(f"{p['sesion']}/{p['id']} no está en el transcript o en la cache de atribuir")
        if p["commit_kb"] is not None and not kbdir:
            raise ValueError("falta --kb (o $EXO_KB): hace falta para reconstruir el contexto de KB")
        msgs = sorted(s["mensajes"], key=lambda m: m["ts"] or "")
        k = next(i for i, x in enumerate(msgs) if x["id"] == p["id"])
        out.append({"id": p["id"], "atribucion": p["atribucion"], "texto": reg["texto"], "previo": m["previo"],
                    "reglas_canal": s["reglas_canal"] or "ninguna",
                    "anteriores": "\n".join(x["texto"] for x in msgs[:k])[-MAX_ANTERIORES:] or "ninguno",
                    "contexto_kb": _contexto_kb(s, p["commit_kb"], kbdir)})
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--etapa", required=True, choices=("1", "2", "regex"))
    ap.add_argument("--cache", default=os.path.expanduser("~/.cache/exo-uso-real"))
    ap.add_argument("--proyectos", required=True, help="directorio de transcripts")
    ap.add_argument("--desde", default="2026-10-06")
    ap.add_argument("--atribuciones", help="etapa 2: --salida de atribuir.py (su cache es <cache>/atribuidor.jsonl)")
    ap.add_argument("--kb", default=os.environ.get("EXO_KB"), help="etapa 2: repo de la KB (o $EXO_KB)")
    ap.add_argument("--semilla", type=int, default=SEMILLA)
    a = ap.parse_args()

    if a.etapa == "2":
        if not a.atribuciones:
            print("muestra: la etapa 2 necesita --atribuciones", file=sys.stderr)
            return 1
        try:
            items = etapa2(atribuidos(a.atribuciones, a.cache, a.proyectos, a.desde, a.kb), a.semilla)
        except ValueError as e:
            print(f"muestra: {e}", file=sys.stderr)
            return 1
    else:
        sesiones, _ = transcripts.poblacion(a.proyectos, a.desde)
        msgs = [m for s in sesiones for m in sorted(s["mensajes"], key=lambda m: m["ts"] or "")]
        items = etapa1(msgs, a.semilla) if a.etapa == "1" else [_item(m) for m in msgs if candidato_regex(m["texto"])]
    os.makedirs(a.cache, exist_ok=True)
    destino = os.path.join(a.cache, "etiquetar-regex.jsonl" if a.etapa == "regex" else f"etiquetar-etapa{a.etapa}.jsonl")
    with open(destino, "w") as f:
        for it in items:
            f.write(json.dumps(it, ensure_ascii=False) + "\n")
    print(f"{len(items)} ítems en {destino}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
