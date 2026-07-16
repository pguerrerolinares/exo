#!/usr/bin/env python3
"""Métricas de un brazo o comparación pareada. Uso: analyze.py <arm> [<arm-vs>]"""
import json, sys
from collections import defaultdict
from pathlib import Path

BASE = Path(__file__).resolve().parent.parent
K = 5
THRESHOLDS = [0.35, 0.40, 0.45, 0.50, 0.55, 0.60, 0.65]


def norm(permalink):
    """Normaliza permalinks para comparar expected_permalink (eval.jsonl, frontmatter)
    contra permalink (results/<arm>.jsonl, CLI real). Verificado manualmente (Task 5,
    2026-07-17): ambos lados ya usan formato "kb-demo/<path>" sin divergencia de
    prefijo de proyecto — la CLI de basic-memory emite permalinks con el project id
    incluido, igual que el frontmatter fuente del eval set. Se mantiene esta función
    como único punto de normalización (strip + colapso de "//") por si el formato
    diverge en un brazo futuro; un mismatch de formato aquí invalidaría todas las
    métricas en silencio, así que CUALQUIER cambio de formato de permalink debe
    normalizarse aquí y solo aquí, nunca ad-hoc en hit().
    """
    if permalink is None:
        return None
    return permalink.strip().replace("//", "/")


def load(arm):
    rows = defaultdict(dict)
    skipped_errors = 0
    for l in open(BASE / "results" / f"{arm}.jsonl"):
        r = json.loads(l)
        if r.get("error"):
            skipped_errors += 1
            continue
        # tolerancia a permalink=None: hay filas reales del CLI (item sin permalink,
        # p.ej. title="developercv.cls") que no deben romper el scoring — se normalizan
        # a None y simplemente nunca hacen match contra un expected_permalink real.
        results = [{**res, "permalink": norm(res.get("permalink"))} for res in r.get("results", [])]
        rows[r["query"]][r["search_type"]] = results
    return rows, skipped_errors


def hit(results, expected, k=K, thr=None):
    top = [r for r in results if thr is None or r.get("score") is None or r["score"] >= thr][:k]
    return any(r.get("permalink") == expected for r in top)


def main():
    arm = sys.argv[1]
    gold = {json.loads(l)["query"]: json.loads(l) for l in open(BASE / "eval.jsonl")}
    # expected_permalink normalizado con la misma norm() que el lado CLI (punto b del brief)
    for g in gold.values():
        g["expected_permalink"] = norm(g["expected_permalink"])
    labeled = {q: g for q, g in gold.items() if g["expected_permalink"]}
    data, skipped_errors = load(arm)
    lines = [f"# metrics — {arm} (hit@{K}, {len(labeled)} queries etiquetadas)\n"]
    if skipped_errors:
        lines.append(f"- filas con error (excluidas del replay): {skipped_errors}\n")

    for stype in ["hybrid", "text", "vector"]:
        hits = [q for q, g in labeled.items() if hit(data[q].get(stype, []), g["expected_permalink"])]
        lines.append(f"- **{stype}**: {len(hits)}/{len(labeled)}")

    obs = [q for q in labeled if any(r.get("type") == "observation" for r in data[q].get("hybrid", [])[:K])]
    lines.append(f"- queries con observation-hit en top-{K} (hybrid): {len(obs)} → {sorted(obs)[:10]}")

    lines.append(f"\n## sweep de threshold (hybrid, filtro por score)")
    for t in THRESHOLDS:
        n = sum(hit(data[q]["hybrid"], g["expected_permalink"], thr=t) for q, g in labeled.items() if "hybrid" in data[q])
        lines.append(f"- thr={t}: {n}/{len(labeled)}")

    lines.append(f"\n## atribución de misses (hybrid, thr=None)")
    for q, g in sorted(labeled.items()):
        if hit(data[q].get("hybrid", []), g["expected_permalink"]):
            continue
        t = hit(data[q].get("text", []), g["expected_permalink"])
        v = hit(data[q].get("vector", []), g["expected_permalink"])
        cls = "fusion-miss" if (t or v) else "both-miss"
        lines.append(f"- MISS `{q[:60]}` → text={'HIT' if t else 'miss'} vector={'HIT' if v else 'miss'} [{cls}]")

    if len(sys.argv) > 2:
        other, _ = load(sys.argv[2])
        fixed = [q for q, g in labeled.items()
                 if hit(data[q].get("hybrid", []), g["expected_permalink"])
                 and not hit(other[q].get("hybrid", []), g["expected_permalink"])]
        broken = [q for q, g in labeled.items()
                  if not hit(data[q].get("hybrid", []), g["expected_permalink"])
                  and hit(other[q].get("hybrid", []), g["expected_permalink"])]
        lines.append(f"\n## pareada {arm} vs {sys.argv[2]}: ARREGLA {len(fixed)} {fixed} · ROMPE {len(broken)} {broken}")

    out = BASE / "results" / f"metrics-{arm}.md"
    out.write_text("\n".join(lines) + "\n")
    print(out)


if __name__ == "__main__":
    main()
