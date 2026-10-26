#!/usr/bin/env python3
"""Sella el universo y el etiquetador (spec 2026-10-26, §5.2/§5.3): escribe universo.txt y sello.json."""
import argparse, glob, hashlib, json, os, sys

H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, H)
import etiquetar, rubrica
sr = etiquetar.sr


def universo(pool, criba):
    ids, desconocidos = [], set()
    for f in sorted(glob.glob(os.path.join(pool, "reglas-g*.jsonl"))):
        for l in sr.leer(f).splitlines():
            if not l.strip():
                continue
            r = json.loads(l)
            if not sr.es_candidata(r):
                continue
            if r["proyecto"] not in sr.REPO:
                desconocidos.add(r["proyecto"]); continue
            if sr.REPO[r["proyecto"]][0] is None:
                continue
            if glob.glob(os.path.join(criba, r["id"], "a0-r*", "check.rc")):
                ids.append(r["id"])
    if desconocidos:
        sys.exit(f"sellar: proyectos sin entrada en REPO: {sorted(desconocidos)}")
    return sorted(ids, key=rubrica._orden_id)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    for a in ("pool", "criba", "prompt", "kb", "commit", "salida-dir"):
        ap.add_argument("--" + a, required=True)
    a = ap.parse_args()
    ids = universo(a.pool, a.criba)
    os.makedirs(a.salida_dir, exist_ok=True)
    texto = "".join(i + "\n" for i in ids)
    with open(os.path.join(a.salida_dir, "universo.txt"), "w") as f:
        f.write(texto)
    sello = {"prompt_sha256": etiquetar.sha256_fichero(a.prompt), "kb_commit": a.commit,
             "claude_version": etiquetar.claude_version(),
             "universo_sha256": hashlib.sha256(texto.encode()).hexdigest(), "n": len(ids)}
    with open(os.path.join(a.salida_dir, "sello.json"), "w") as f:
        json.dump(sello, f, indent=2); f.write("\n")
    print(f"sellar: universo de {len(ids)} reglas")


if __name__ == "__main__":
    main()
