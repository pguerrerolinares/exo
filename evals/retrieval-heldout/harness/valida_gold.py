#!/usr/bin/env python3
"""Valida un gold de held-out (C: pre-registro §3; J: borrador §3) contra el
snapshot de la KB. Exit 1 con la lista de errores; exit 0 imprime recuentos
y sha256. `--in-sample` es repetible: cada fichero JSONL con campo `query`
es una lista de exclusión (anti-fuga; para J: las 55 y el gold de C).
Uso: valida_gold.py --gold G --kb KB --in-sample IN55 [--in-sample GOLD_C]
"""
import argparse
import hashlib
import json
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from pool import jaccard, normaliza  # noqa: E402

# C: prompt | agent-search | hard. J añade keyword (palabras clave escritas
# por Paul), archive (la respuesta vive en archive/) y negativo (tema ausente
# de la KB: corpus negativo verdadero, expected null por definición).
FUENTES = {"prompt", "agent-search", "hard", "keyword", "archive", "negativo"}


def permalinks_snapshot(kb):
    out = set()
    raiz = Path(kb)
    for p in raiz.rglob("*.md"):
        if any(parte.startswith(".") for parte in p.relative_to(raiz).parts):
            continue
        lineas = p.read_text(encoding="utf-8", errors="ignore").splitlines()
        if not lineas or lineas[0].strip() != "---":
            continue
        for l in lineas[1:]:
            if l.strip() == "---":
                break
            if l.startswith("permalink:"):
                out.add(l.split(":", 1)[1].strip().strip("'\""))
                break
    return out


def valida(filas, permalinks, in55):
    errores, ids = [], Counter(f.get("id") for f in filas)
    for f in filas:
        i = f.get("id")
        if ids[i] > 1:
            errores.append(f"{i}: id duplicado")
        if not {"id", "query", "source", "expected_permalink", "acceptable_permalinks", "notes"} <= set(f):
            errores.append(f"{i}: faltan campos")
            continue
        if f["source"] not in FUENTES:
            errores.append(f"{i}: source inválido {f['source']!r}")
        n = normaliza(f["query"])
        if any(n == m or jaccard(n, m) >= 0.8 for m in in55):
            errores.append(f"{i}: query duplica una de las 55")
        exp, acc = f["expected_permalink"], f["acceptable_permalinks"]
        if f["source"] == "negativo" and exp is not None:
            errores.append(f"{i}: source negativo con expected_permalink (debe ser null)")
        if f["source"] == "archive" and (exp is None or "/archive/" not in exp):
            errores.append(f"{i}: source archive exige expected_permalink bajo archive/")
        if exp is None:
            if acc:
                errores.append(f"{i}: fila null con acceptable_permalinks")
            continue
        if exp not in permalinks:
            errores.append(f"{i}: expected_permalink inexistente en el snapshot")
        if len(acc) > 2:
            errores.append(f"{i}: más de 2 acceptable_permalinks")
        if exp in acc or len(set(acc)) != len(acc):
            errores.append(f"{i}: acceptable repite expected o se repite")
        if any(p not in permalinks for p in acc):
            errores.append(f"{i}: acceptable inexistente en el snapshot")
        if acc and "aceptable:" not in f["notes"]:
            errores.append(f"{i}: acceptable sin justificación 'aceptable:' en notes")
    return errores


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--gold", required=True)
    ap.add_argument("--kb", required=True)
    ap.add_argument("--in-sample", action="append", required=True, help="repetible: JSONL de exclusión")
    a = ap.parse_args()
    filas = [json.loads(l) for l in open(a.gold, encoding="utf-8") if l.strip()]
    in55 = [normaliza(json.loads(l)["query"]) for ruta in a.in_sample for l in open(ruta, encoding="utf-8") if l.strip()]
    errores = valida(filas, permalinks_snapshot(a.kb), in55)
    for e in errores:
        print(e, file=sys.stderr)
    no_nulas = [f for f in filas if f.get("expected_permalink")]
    print(json.dumps({
        "filas": len(filas), "no_nulas": len(no_nulas), "nulas": len(filas) - len(no_nulas),
        "no_nulas_por_estrato": dict(Counter(f.get("source") for f in no_nulas)),
        "nulas_por_estrato": dict(Counter(f.get("source") for f in filas if not f.get("expected_permalink"))),
        "con_acceptable": sum(1 for f in filas if f.get("acceptable_permalinks")),
        "sha256": hashlib.sha256(Path(a.gold).read_bytes()).hexdigest(),
        "errores": len(errores)}, ensure_ascii=False))
    sys.exit(1 if errores else 0)


if __name__ == "__main__":
    main()
