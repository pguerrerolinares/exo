#!/usr/bin/env python3
"""Replay del eval set vía CLI de basic-memory. Uso: replay.py <arm-name>

Interfaz real del CLI (recon Task 1, congelado en snapshot/cli-interface.md;
DIFIERE del brief original — estas resoluciones mandan):

  a) La query es un ARGUMENTO POSICIONAL (`[QUERY]`), no `--query`.
  b) No existen `--search-type` ni `--min-similarity`. Los 3 modos de
     búsqueda se seleccionan con flags booleanos. Verificado con smoke test
     (query "agent-develop bitácora", --page-size 5) antes del run completo:

       # texto (FTS, sin flag semántico):
       basic-memory tool search-notes --project kb-demo --page-size 5 \
           "agent-develop bitácora"

       # vector (embeddings puros):
       basic-memory tool search-notes --project kb-demo --vector \
           --page-size 5 "agent-develop bitácora"

       # hybrid (FTS + vector combinados):
       basic-memory tool search-notes --project kb-demo --hybrid \
           --page-size 5 "agent-develop bitácora"

     (con la config vigente en el smoke — min_similarity=0.55 — "" y
     "--hybrid" devolvieron el mismo top-5 para esa query; son rutas de
     código distintas igual, documentadas tal cual en la CLI --help.)

  c) El threshold semántico NO se puede pasar por CLI (no hay
     --min-similarity). replay.py edita ~/.basic-memory/config.json al
     arrancar, pone `semantic_min_similarity` a 0.0 (guardando el valor
     previo en el sidecar ANTES de tocar config.json, y escribiendo
     config.json de forma atómica vía tmp+os.replace) para capturar scores
     completos sin recortar, y lo RESTAURA al terminar (try/finally,
     también si el run peta). El sweep de threshold se hace offline en
     analyze.py (Task 5) sobre estos scores.
  d) Los nombres de campo reales en la salida JSON son `score`/`type`/
     `permalink` (ya coinciden con lo que este script extrae).
"""
import json
import os
import subprocess
import sys
import time
from pathlib import Path

BASE = Path(__file__).resolve().parent.parent
CONFIG_PATH = Path.home() / ".basic-memory" / "config.json"
# Sidecar: guarda el semantic_min_similarity ORIGINAL fuera de config.json.
# Necesario para sobrevivir a un crash a mitad de run: si el proceso muere
# con la config ya en 0.0 y se relanza, re-leer config.json capturaría 0.0
# como "previous" (bug) en vez del valor real pre-run. Este fichero es la
# fuente de verdad del valor a restaurar mientras haya un run en curso.
# INVARIANTE: el sidecar se escribe ANTES de tocar config.json (nunca al
# revés) — así un crash entre "leer previous" y "escribir sidecar" deja
# config.json intacto (aún en su valor original), y un crash después de
# escribir el sidecar pero antes de escribir 0.0 en config.json también es
# seguro: el siguiente resume ve el sidecar y reintenta poner 0.0.
MIN_SIMILARITY_BACKUP = BASE / "harness" / ".min_similarity_backup.json"

CMD_BASE = ["basic-memory", "tool", "search-notes", "--project", "kb-demo",
            "--page-size", "10"]

# search_type -> flags extra (la query posicional se añade aparte)
SEARCH_TYPE_FLAGS = {
    "text": [],
    "vector": ["--vector"],
    "hybrid": ["--hybrid"],
}
SEARCH_TYPES = ["hybrid", "text", "vector"]


def read_min_similarity():
    """Lee semantic_min_similarity de config.json sin tocarlo."""
    with open(CONFIG_PATH) as f:
        return json.load(f).get("semantic_min_similarity")


def write_min_similarity(value):
    """Escribe semantic_min_similarity en config.json de forma ATÓMICA:
    config REAL de Paul (no solo la del eval) — un kill a mitad de un
    open("w") directo la truncaría a medio escribir. Escribe a un .tmp y
    hace os.replace() (atómico a nivel de filesystem: nunca deja el fichero
    a medias, o queda el viejo completo o el nuevo completo)."""
    with open(CONFIG_PATH) as f:
        config = json.load(f)
    config["semantic_min_similarity"] = value
    tmp_path = CONFIG_PATH.with_name(CONFIG_PATH.name + ".tmp")
    with open(tmp_path, "w") as f:
        json.dump(config, f, indent=2)
        f.write("\n")
    os.replace(tmp_path, CONFIG_PATH)


def search(query, stype):
    cmd = CMD_BASE + SEARCH_TYPE_FLAGS[stype] + [query]
    try:
        out = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    except subprocess.TimeoutExpired:
        return {"error": "timeout"}
    if out.returncode != 0:
        return {"error": out.stderr[-500:]}
    data = json.loads(out.stdout)
    results = data.get("results", data if isinstance(data, list) else [])
    return {"results": [{"permalink": r.get("permalink"), "type": r.get("type"),
                         "score": r.get("score")} for r in results[:10]]}


def load_done(outpath):
    """Lee las filas ya escritas (resume tras un run interrumpido) y repara
    el fichero si la última línea quedó a medio escribir (kill a mitad de
    escritura): las líneas corruptas se descartan y el fichero se reescribe
    solo con las válidas, para que el append posterior parta de un jsonl
    limpio. Devuelve el set de pares (query, search_type) ya cubiertos."""
    done = set()
    if not outpath.exists():
        return done
    valid_lines = []
    for line in outpath.read_text().splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            d = json.loads(line)
        except json.JSONDecodeError:
            continue  # línea corrupta/parcial (proceso murió a mitad de escritura) → se descarta
        valid_lines.append(line)
        done.add((d.get("query"), d.get("search_type")))
    outpath.write_text("".join(l + "\n" for l in valid_lines))
    return done


def main(arm):
    rows = [json.loads(l) for l in open(BASE / "eval.jsonl")]
    outpath = BASE / "results" / f"{arm}.jsonl"
    done = load_done(outpath)
    if done:
        print(f"Resume: {len(done)} pares (query,search_type) ya en {outpath}, se saltan.",
              file=sys.stderr)
    with open(outpath, "a") as f:
        for i, row in enumerate(rows):
            for stype in SEARCH_TYPES:
                if (row["query"], stype) in done:
                    continue
                t0 = time.monotonic()
                res = search(row["query"], stype)
                f.write(json.dumps({"query": row["query"], "search_type": stype,
                                    "elapsed_s": round(time.monotonic() - t0, 2), **res},
                                   ensure_ascii=False) + "\n")
                f.flush()
            print(f"[{i+1}/{len(rows)}] {row['query'][:50]}", file=sys.stderr)
    print(f"OK → {outpath}", file=sys.stderr)


if __name__ == "__main__":
    if MIN_SIMILARITY_BACKUP.exists():
        # Resume tras un crash: config.json ya está en 0.0 (dejado por el run
        # anterior); el valor real a restaurar es el del sidecar, no el que
        # esté ahora mismo en config.json.
        previous_min_similarity = json.loads(MIN_SIMILARITY_BACKUP.read_text())["semantic_min_similarity"]
        write_min_similarity(0.0)  # asegura 0.0 (no-op si ya lo estaba)
        print(f"Resume: previous recuperado de {MIN_SIMILARITY_BACKUP}: {previous_min_similarity}",
              file=sys.stderr)
    else:
        # Orden crítico: sidecar ANTES de tocar config.json. Si el proceso
        # muere entre estas dos líneas, config.json queda intacto (aún en su
        # valor original) — nunca en el estado "0.0 sin sidecar" que rompía
        # el resume.
        previous_min_similarity = read_min_similarity()
        MIN_SIMILARITY_BACKUP.write_text(json.dumps({"semantic_min_similarity": previous_min_similarity}))
        write_min_similarity(0.0)
    try:
        main(sys.argv[1])
    finally:
        write_min_similarity(previous_min_similarity)
        if MIN_SIMILARITY_BACKUP.exists():
            MIN_SIMILARITY_BACKUP.unlink()
        print(f"config.json restaurado: semantic_min_similarity={previous_min_similarity}", file=sys.stderr)
