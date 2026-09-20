#!/usr/bin/env python3
"""Acuerdo entre jueces y construcción del gold J (borrador §3, §11).
Entrada: candidatos (id, query, source, candidatos) y dos ficheros de
respuestas (fable, kimi) con {id, expected, acceptable, razon}.
  acuerdo_fila(a, b): "estricto" (mismo expected, null incluido), "lenient"
    (el expected de uno está en los acceptable del otro) o None.
  kappa(pares): κ de Cohen (Cohen 1960) sobre el expected estricto, con cada
    permalink y null como categoría nominal.
  fusiona(a, b, tipo): expected = el común (estricto) o el que el otro juez
    admite como aceptable (lenient); acceptable = (A_f ∩ A_k) ∪ {el expected
    no elegido en un acuerdo lenient}, ≤ 2, sin el expected.
  construye(...): una fila entra al gold solo con acuerdo; los desacuerdos SE
    DESCARTAN (sin tercer juez, borrador §3) y se listan en privado. Reglas de
    estrato: negativo exige null; archive exige expected bajo archive/.
  solape_lexico(query, texto): fracción de tokens de la query (≥ 4 letras)
    presentes en la nota; auditoría del sesgo léxico de los jueces (acordadas
    frente a descartadas).
Salida pública (informe): solo recuentos, κ por estrato y la auditoría.
Exit 0 si el suelo pre-registrado pasa, 2 si no (J PARA).
Uso: acuerdo.py --candidatos C --fable F --kimi K [--snap KB] --gold-out G
     --descartes-out D --informe-out I.md --kappa-min 0.60 --po-min 0.70
"""
import argparse
import json
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from pool import normaliza  # noqa: E402


def acuerdo_fila(a, b):
    if a["expected"] == b["expected"]:
        return "estricto"
    if a["expected"] is not None and a["expected"] in b["acceptable"]:
        return "lenient"
    if b["expected"] is not None and b["expected"] in a["acceptable"]:
        return "lenient"
    return None


def kappa(pares):
    n = len(pares)
    if n == 0:
        return 0.0
    po = sum(1 for a, b in pares if a == b) / n
    ca, cb = Counter(a for a, _ in pares), Counter(b for _, b in pares)
    pe = sum(ca[k] * cb[k] for k in set(ca) | set(cb)) / (n * n)
    return 1.0 if pe == 1.0 else (po - pe) / (1 - pe)


def fusiona(a, b, tipo):
    """§3/D-J11: acceptable_permalinks son "solo los admitidos por ambos
    jueces". En un acuerdo lenient, el `expected` descartado (el del juez
    que no fija `exp`) solo entra en `extra` si el juez ganador TAMBIEN lo
    tiene en su propio `acceptable` -si no, es una nota que solo vio un
    juez y no debe poder producir un hit- (fix review 2026-09-20, I-2).
    Tie-break asimétrico declarado en el borrador §3: si ambas direcciones
    lenient valen a la vez, gana `a` (fable, por el orden de los `if`)."""
    comunes = [x for x in a["acceptable"] if x in b["acceptable"]]
    if tipo == "estricto":
        exp, extra = a["expected"], []
    elif a["expected"] is not None and a["expected"] in b["acceptable"]:
        exp = a["expected"]
        extra = [b["expected"]] if b["expected"] and b["expected"] in a["acceptable"] else []
    else:
        exp = b["expected"]
        extra = [a["expected"]] if a["expected"] and a["expected"] in b["acceptable"] else []
    acc = []
    for x in extra + comunes:
        if x != exp and x not in acc:
            acc.append(x)
    return exp, acc[:2]


def solape_lexico(query, texto):
    toks = {t for t in normaliza(query).split() if len(t) >= 4}
    if not toks:
        return 0.0
    t = normaliza(texto)
    return sum(1 for x in toks if x in t) / len(toks)


def construye(cands, fab, kim, textos, kappa_min, po_min):
    por_src, gold, desc = {}, [], []
    # F3 del review de rama (2026-09-20): "acuerdo forzado, null-null por
    # construcción" no es solo `source == "negativo"` -- una fila con
    # `candidatos` vacío (el paquete lleva literalmente "(sin candidatas:
    # expected debe ser null)", `juez.paquete`) es null-null por el mismo
    # motivo estructural, sea cual sea su `source` (p.ej. 26 de `prompt` sin
    # candidata plausible). `vacios` marca esos ids para excluirlos, junto a
    # `negativo`, de la línea descriptiva κ/p_o "sin acuerdo forzado" más
    # abajo -- no cambia `pasa` ni el suelo firmado (0,60 ∧ 0,70), que siguen
    # sobre TODAS las filas juzgadas.
    vacios = {c["id"] for c in cands if not c["candidatos"]}
    for c in cands:
        i = c["id"]
        if i not in fab or i not in kim:
            desc.append({"id": i, "motivo": "sin juicio de ambos"})
            continue
        tipo = acuerdo_fila(fab[i], kim[i])
        por_src.setdefault(c["source"], []).append((i, fab[i]["expected"], kim[i]["expected"], tipo))
        if tipo is None:
            desc.append({"id": i, "motivo": "desacuerdo", "fable": fab[i]["expected"], "kimi": kim[i]["expected"]})
            continue
        exp, acc = fusiona(fab[i], kim[i], tipo)
        if c["source"] == "negativo" and exp is not None:
            desc.append({"id": i, "motivo": "negativo con nota", "fable": fab[i]["expected"], "kimi": kim[i]["expected"]})
            continue
        if c["source"] == "archive" and (exp is None or "/archive/" not in exp):
            desc.append({"id": i, "motivo": "archive sin expected en archive/", "fable": fab[i]["expected"], "kimi": kim[i]["expected"]})
            continue
        notes = f"acuerdo {tipo}; fable: {fab[i]['razon']} | kimi: {kim[i]['razon']}"
        if acc:
            notes += " | aceptable: admitido por ambos jueces"
        gold.append({"id": i, "query": c["query"], "source": c["source"], "expected_permalink": exp,
                     "acceptable_permalinks": acc, "notes": notes})
    todos = [(a, b, t) for v in por_src.values() for _, a, b, t in v]
    k_total = kappa([(a, b) for a, b, _ in todos])
    po_total = sum(1 for _, _, t in todos if t) / max(1, len(todos))
    pasa = k_total >= kappa_min and po_total >= po_min
    # I-3 del review (2026-09-20): `negativo` es null-null por construcción y
    # infla κ/p_o pooled (Feinstein & Cicchetti 1990). Descriptivo: no toca
    # `pasa` ni el suelo firmado (D-J11). F3 del review de rama (2026-09-20):
    # el mismo argumento aplica a CUALQUIER fila con candidatos vacío (no
    # solo `negativo`, ver `vacios` arriba) -- también null-null forzado.
    todos_sin_neg = [(a, b, t) for s, v in por_src.items() for i, a, b, t in v if s != "negativo" and i not in vacios]
    k_sin_neg = kappa([(a, b) for a, b, _ in todos_sin_neg])
    po_sin_neg = sum(1 for _, _, t in todos_sin_neg if t) / max(1, len(todos_sin_neg))
    L = ["# Acuerdo entre jueces — gold J (fable × Kimi, ciegos)", ""]
    L.append(f"- filas juzgadas por ambos: {len(todos)} · acuerdo (estricto+lenient): {sum(1 for _, _, t in todos if t)} ({po_total:.3f}) · estricto: {sum(1 for _, _, t in todos if t == 'estricto')} · κ estricto: {k_total:.3f}")
    L.append(f"- κ / p_o sin `negativo` ni candidatos vacíos: {k_sin_neg:.3f} / {po_sin_neg:.3f} (sobre {len(todos_sin_neg)} filas; descriptivo, no decide)")
    L.append(f"- suelo pre-registrado: κ ≥ {kappa_min} y acuerdo ≥ {po_min} → {'PASA' if pasa else 'NO PASA: J PARA'}")
    L += ["", "| estrato | juzgadas | acuerdo | p_o | κ | entran al gold | no nulas |", "|---|---|---|---|---|---|---|"]
    entradas = Counter(g["source"] for g in gold)
    nonulas = Counter(g["source"] for g in gold if g["expected_permalink"])
    for s, v in sorted(por_src.items()):
        ac = sum(1 for _, a, b, t in v if t)
        L.append(f"| {s} | {len(v)} | {ac} | {ac / len(v):.3f} | {kappa([(a, b) for _, a, b, _ in v]):.3f} | {entradas[s]} | {nonulas[s]} |")
    if textos is not None:
        por_id = {c["id"]: c for c in cands}

        def sol(i):
            # Asimetría declarada (Minor M-4 del pre-registro, §3): para las
            # filas DESCARTADAS (sin fila de gold, `expected_permalink`
            # propio) se audita contra el `expected` del primer juez que lo
            # tenga -no hay un "expected del gold" que auditar, porque la
            # fila nunca entró al gold-.
            exp = fab[i]["expected"] or kim[i]["expected"]
            return solape_lexico(por_id[i]["query"], textos.get(exp, "")) if exp else None

        def sol_gold(g):
            # F2 del review de rama (2026-09-20): para las filas DEL GOLD la
            # auditoría es contra `g["expected_permalink"]` -el expected que
            # de verdad ganó la fusión (§3, D-J11)-, nunca contra
            # `fab.expected or kim.expected`: en un acuerdo lenient decidido
            # por el segundo juez, esa fórmula "or" seguía devolviendo el
            # `expected` del PRIMER juez (fable) aunque no fuera el que
            # ganó, auditando la fila contra la nota equivocada.
            exp = g["expected_permalink"]
            return solape_lexico(g["query"], textos.get(exp, "")) if exp else None

        s_ok = [x for x in (sol_gold(g) for g in gold if g["expected_permalink"]) if x is not None]
        s_no = [x for x in (sol(d["id"]) for d in desc if d["motivo"] == "desacuerdo") if x is not None]
        med = lambda v: round(sorted(v)[len(v) // 2], 2) if v else None  # noqa: E731
        L += ["", "## auditoría del sesgo léxico (fracción de tokens de la query ≥4 letras presentes en la nota esperada)", "",
              f"- filas acordadas no nulas: n={len(s_ok)} · mediana {med(s_ok)} · con solape ≥ 0,5: {sum(1 for x in s_ok if x >= 0.5)}",
              f"- desacuerdos con alguna nota propuesta: n={len(s_no)} · mediana {med(s_no)} · con solape ≥ 0,5: {sum(1 for x in s_no if x >= 0.5)}",
              "- lectura: si los desacuerdos se concentran en solape bajo, los jueces acuerdan sobre todo donde la query repite la nota (sesgo léxico); el subconjunto «léxicamente difícil» (solape < 0,5) de las no nulas es el guard de D-A (borrador §6)."]
        for g in gold:
            if g["expected_permalink"]:
                g["notes"] += f" | solape_lexico={sol_gold(g):.2f}"
    L += ["", f"- descartes: {len(desc)} (" + ", ".join(f"{k}={v}" for k, v in sorted(Counter(d['motivo'] for d in desc).items())) + ")"]
    return gold, desc, "\n".join(L) + "\n", k_total, po_total


def _carga(ruta):
    return {json.loads(l)["id"]: json.loads(l) for l in open(ruta, encoding="utf-8") if l.strip()}


def main():
    ap = argparse.ArgumentParser()
    for k in ("candidatos", "fable", "kimi", "gold-out", "descartes-out", "informe-out"):
        ap.add_argument(f"--{k}", required=True)
    ap.add_argument("--snap")
    ap.add_argument("--kappa-min", type=float, default=0.60)
    ap.add_argument("--po-min", type=float, default=0.70)
    a = ap.parse_args()
    cands = [json.loads(l) for l in open(a.candidatos, encoding="utf-8") if l.strip()]
    textos = None
    if a.snap:
        from juez import rutas_por_permalink
        textos = {p: Path(r).read_text(encoding="utf-8", errors="ignore") for p, r in rutas_por_permalink(a.snap).items()}
    gold, desc, inf, k, po = construye(cands, _carga(a.fable), _carga(a.kimi), textos, a.kappa_min, a.po_min)
    for n, g in enumerate(gold, start=1):
        g["id"] = f"j{n:03d}"
    with open(a.gold_out, "w", encoding="utf-8") as fh:
        for g in gold:
            fh.write(json.dumps(g, ensure_ascii=False) + "\n")
    with open(a.descartes_out, "w", encoding="utf-8") as fh:
        for d in desc:
            fh.write(json.dumps(d, ensure_ascii=False) + "\n")
    Path(a.informe_out).write_text(inf, encoding="utf-8")
    pasa = k >= a.kappa_min and po >= a.po_min
    print(json.dumps({"gold": len(gold), "no_nulas": sum(1 for g in gold if g["expected_permalink"]), "descartes": len(desc), "kappa": round(k, 3), "po": round(po, 3), "pasa": pasa}))
    sys.exit(0 if pasa else 2)


if __name__ == "__main__":
    main()
