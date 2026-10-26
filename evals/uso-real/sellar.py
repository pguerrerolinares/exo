#!/usr/bin/env python3
"""Sella la medición antes de correrla (spec 2026-10-26 §4.6): población, KB, prompts, versión, umbrales y modelos.

Si --salida ya existe se niega (exit 2) salvo --forzar. `--desde` es la palanca de la reserva (§4.1) y queda en el sello."""
import argparse, hashlib, inspect, json, os, subprocess, sys

H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, H)
import atribuir, detectar, juez, transcripts

LOTE = 10
SEMILLA = 20261026
UMBRALES = {"puerta": 0.4, "kappa": 0.4, "n_min_invalidacion": 30, "frac_invalidacion": 0.10}


def sha_sesiones(sesiones):
    return hashlib.sha256("\n".join(s["id"] for s in sesiones).encode()).hexdigest()


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--proyectos", required=True)
    ap.add_argument("--kb", default=os.environ.get("EXO_KB"))
    ap.add_argument("--salida", required=True)
    ap.add_argument("--desde", default="2026-10-06")
    ap.add_argument("--forzar", action="store_true")
    a = ap.parse_args()
    if not a.kb:
        print("sellar: falta --kb (o $EXO_KB)", file=sys.stderr)
        return 1
    if os.path.exists(a.salida) and not a.forzar:
        print(f"sellar: {a.salida} ya existe; --forzar para resellar", file=sys.stderr)
        return 2
    r = subprocess.run(["git", "-C", a.kb, "rev-parse", "HEAD"], capture_output=True, text=True)
    if r.returncode != 0:
        print(f"sellar: {a.kb} no es un repo git con commits", file=sys.stderr)
        return 1
    sesiones, _ = transcripts.poblacion(a.proyectos, a.desde)
    sello = {
        "sesiones_sha256": sha_sesiones(sesiones),
        "n_sesiones": len(sesiones),
        "desde": a.desde,
        "kb_commit": r.stdout.strip(),
        "prompt_detector_sha256": juez.sha256_fichero(detectar.PROMPT),
        "prompt_atribuidor_sha256": juez.sha256_fichero(atribuir.PROMPT),
        "claude_version": juez.claude_version(),
        "modelos": {"detector": detectar.MODELO, "atribuidor": atribuir.MODELO},
        "umbrales": UMBRALES,
        "lote": LOTE,
        "semilla": SEMILLA,
        "filtro_sha256": hashlib.sha256(inspect.getsource(transcripts.es_mensaje_humano).encode()).hexdigest(),
    }
    with open(a.salida, "w") as f:
        json.dump(sello, f, indent=2)
        f.write("\n")
    print(f"sellado: {len(sesiones)} sesiones desde {a.desde}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
