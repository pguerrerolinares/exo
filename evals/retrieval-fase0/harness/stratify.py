#!/usr/bin/env python3
"""Marca queries observation-sensitive vía FTS interno (read-only). Método
documentado en gate.md: el CLI agrega a entidad, el probe mira el índice."""
import json, sqlite3
from pathlib import Path
BASE = Path(__file__).resolve().parent.parent
db = sqlite3.connect("file:" + str(Path.home() / ".basic-memory/memory.db") + "?mode=ro", uri=True)

def fts_terms(q):
    clean = "".join(c if c.isalnum() else " " for c in q)
    terms = [t for t in clean.split() if len(t) > 2]
    return " OR ".join(terms) if terms else '"' + q + '"'

rows = [json.loads(l) for l in open(BASE / "eval.jsonl")]
out, sensitive = [], 0
for r in rows:
    if not r.get("expected_permalink"):
        continue
    try:
        res = db.execute("SELECT type FROM search_index WHERE search_index MATCH ? ORDER BY rank LIMIT 10",
                         (fts_terms(r["query"]),)).fetchall()
        obs = any(t[0] == "observation" for t in res)
    except sqlite3.OperationalError:
        obs = None  # query FTS inválida tras sanitizar; se documenta
    out.append({"query": r["query"], "observation_sensitive": obs})
    sensitive += 1 if obs else 0
with open(BASE / "results" / "stratification.jsonl", "w") as f:
    for o in out:
        f.write(json.dumps(o, ensure_ascii=False) + "\n")
print(f"{sensitive}/{len(out)} queries observation-sensitive (None = FTS inválida: {sum(1 for o in out if o['observation_sensitive'] is None)})")
