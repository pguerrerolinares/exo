#!/usr/bin/env python3
"""Oráculo de paridad de corpus (spec M2 §4-§5 pata 1).
--capture-bm: extrae del índice de basic-memory (RO) el set de permalinks a
nivel ENTIDAD (gotcha M0: jamás contra el output del CLI) y lo sella en gold/.
--diff ENGINE_DB: compara el índice del engine contra el gold sellado.
Exit 0 = diff vacío; exit 1 = divergencia (lista completa en stdout).

Ajustes sobre el código del plan (documentados en la spec del indexer §1.2):
filtro por proyecto kb-demo, umbrales de parada del brief M2-02 codificados
(no se sella un gold sospechoso) y HEAD de kb-demo dentro del gold."""
import argparse, json, sqlite3, subprocess, sys
from pathlib import Path

BASE = Path(__file__).resolve().parent.parent
GOLD = BASE / "gold" / "corpus-bm.json"
PROYECTO = "kb-demo"
# Umbral de parada del brief M2-02: conteo de referencia §6.2 ±10%.
REF_ENTIDADES, TOLERANCIA = 117, 12

def ro(path):
    return sqlite3.connect(f"file:{path}?mode=ro", uri=True)

def head_kb_demo():
    cfg = json.loads((Path.home() / ".basic-memory/config.json").read_text())
    kb = cfg["projects"][PROYECTO]["path"]
    return subprocess.run(
        ["git", "-C", kb, "rev-parse", "HEAD"],
        capture_output=True, text=True, check=True,
    ).stdout.strip()

def capture_bm():
    db = ro(Path.home() / ".basic-memory/memory.db")
    filas = db.execute(
        "SELECT e.permalink, e.file_path FROM entity e"
        " JOIN project p ON e.project_id = p.id"
        " WHERE p.name = ? AND e.permalink IS NOT NULL",
        (PROYECTO,),
    ).fetchall()
    permalinks = sorted(p for p, _ in filas)
    dotdirs = [f for _, f in filas if f and f.split("/")[0] in (".claude", ".omc", ".superpowers")]
    datos = {
        "kb_demo_head": head_kb_demo(),
        "n_entidades": len(permalinks),
        "n_dotdirs_dentro": len(dotdirs),   # DEBE ser 0 (exclusión §6.2)
        "n_archive": sum(1 for p in permalinks if p.startswith("archive/") or "/archive/" in p),
        "permalinks": permalinks,
    }
    if datos["n_dotdirs_dentro"] != 0:
        print(f"PARADA sin sellar: {datos['n_dotdirs_dentro']} entidades de dotdirs dentro "
              f"(deben ser 0, §6.2): {dotdirs}")
        sys.exit(1)
    if abs(datos["n_entidades"] - REF_ENTIDADES) > TOLERANCIA:
        print(f"PARADA sin sellar: {datos['n_entidades']} entidades, fuera de "
              f"{REF_ENTIDADES}±{TOLERANCIA} (brief M2-02)")
        sys.exit(1)
    GOLD.parent.mkdir(parents=True, exist_ok=True)
    GOLD.write_text(json.dumps(datos, ensure_ascii=False, indent=1) + "\n")
    print(f"sellado: {datos['n_entidades']} entidades, archive={datos['n_archive']}, "
          f"dotdirs_dentro={datos['n_dotdirs_dentro']}, head={datos['kb_demo_head'][:12]}")

def diff(engine_db):
    gold = set(json.loads(GOLD.read_text())["permalinks"])
    eng = {r[0] for r in ro(engine_db).execute("SELECT permalink FROM notas").fetchall()}
    faltan, sobran = sorted(gold - eng), sorted(eng - gold)
    for p in faltan: print(f"FALTA en engine: {p}")
    for p in sobran: print(f"SOBRA en engine: {p}")
    print(f"gold={len(gold)} engine={len(eng)} faltan={len(faltan)} sobran={len(sobran)}")
    sys.exit(0 if not faltan and not sobran else 1)

if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--capture-bm", action="store_true")
    g.add_argument("--diff", metavar="ENGINE_DB")
    a = ap.parse_args()
    capture_bm() if a.capture_bm else diff(a.diff)
