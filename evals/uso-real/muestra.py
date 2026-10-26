#!/usr/bin/env python3
"""Muestras a ciegas para el dueño (spec 2026-10-26 §4.2 y §4.4).

Escribe `<cache>/etiquetar-etapa<k>.jsonl` (o `etiquetar-regex.jsonl`) con el texto y el previo tal como los vio
el juez (mismo truncado) y sin ningún veredicto. El texto va solo a la cache: el repo es público."""
import argparse, json, os, random, re, sys, unicodedata

H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, H)
import detectar, transcripts

SEMILLA = 20261026
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
    azar = rng.sample(resto, min(80 - len(pos) if len(casan) < 40 else 40, len(resto)))
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
    # `nota` es el contexto de KB que vio el juez; `atribucion` y `linea` son su veredicto y no salen.
    return [{**_item(a), "nota": a.get("nota"), "atribucion_dueño": None} for a in sel]


def _lee_jsonl(path):
    with open(path) as f:
        return [json.loads(l) for l in f if l.strip()]


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--etapa", required=True, choices=("1", "2", "regex"))
    ap.add_argument("--cache", default=os.path.expanduser("~/.cache/exo-uso-real"))
    ap.add_argument("--proyectos", help="etapas 1 y regex: directorio de transcripts")
    ap.add_argument("--desde", default="2026-10-06")
    ap.add_argument("--positivos", help="etapa 2: cache de atribuir.py (jsonl con atribucion, nota, texto, previo)")
    ap.add_argument("--semilla", type=int, default=SEMILLA)
    a = ap.parse_args()

    if a.etapa == "2":
        if not a.positivos:
            print("muestra: la etapa 2 necesita --positivos", file=sys.stderr)
            return 1
        items = etapa2(_lee_jsonl(a.positivos), a.semilla)
    else:
        if not a.proyectos:
            print("muestra: las etapas 1 y regex necesitan --proyectos", file=sys.stderr)
            return 1
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
