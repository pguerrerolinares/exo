#!/usr/bin/env python3
"""Extractor de referencia de wikilinks (oráculo M2-04, spec M2 §3:
"extractor de referencia por script + spot-check"). Camina la KB replicando
las exclusiones del walker Rust (`.claude/`, `.omc/`, `.superpowers/` fuera
en cualquier nivel; `archive/` dentro; solo `*.md`; solo notas con
`permalink:` en frontmatter), extrae con el MISMO patrón regex del engine
(`\\[\\[([^\\]]+)\\]\\]`) el set `(origen_permalink, destino_texto)` y lo
diffea contra `SELECT origen, destino_texto FROM aristas` de una DB del
engine recién construida (`--diff ENGINE_DB`).

Diff = ∅ ⇒ exit 0. Diff no vacío ⇒ lista FALTA/SOBRA y exit 1.

Es cross-check del PIPELINE (walker+parser+insert), no del regex en sí —
por eso el brief M2-04 pide además un spot-check manual de 5 notas. El
parseo de frontmatter aquí es deliberadamente más simple que el YAML real
(`yaml_serde` en Rust): basta una línea top-level `permalink: valor`:
suficiente para las notas reales de la KB (sin bloques/anchors YAML en el
campo `permalink`), y si algún día no lo fuera, el diff lo destaparía como
FALTA/SOBRA — no hace falta un parser YAML completo para este cross-check.
stdlib puro, READ-ONLY sobre la KB (jamás escribe nada)."""
import argparse
import json
import re
import sqlite3
import sys
from pathlib import Path

PROYECTO = "kb-demo"
DOTDIRS_EXCLUIDOS = {".claude", ".omc", ".superpowers"}
PATRON_WIKILINK = re.compile(r"\[\[([^\]]+)\]\]")
PATRON_PERMALINK = re.compile(r"^permalink:\s*(.+?)\s*$")


def kb_path() -> Path:
    cfg = json.loads((Path.home() / ".basic-memory/config.json").read_text())
    return Path(cfg["projects"][PROYECTO]["path"])


def walk_md(raiz: Path):
    """Replica `walker.rs`: recorrido recursivo, `*.md` solamente, excluye
    dotdirs en cualquier nivel del árbol, `archive/` SE incluye. Orden
    determinista (mismo criterio: ordenado por ruta)."""
    for p in sorted(raiz.rglob("*.md")):
        rel = p.relative_to(raiz)
        if any(parte in DOTDIRS_EXCLUIDOS for parte in rel.parts[:-1]):
            continue
        yield p


def separa_frontmatter(contenido: str):
    """Replica `nota.rs::separa_frontmatter`: primera línea `---` + primera
    línea `---` de cierre más adelante. `None` si no hay frontmatter
    delimitado (línea a línea, no offsets de bytes — igual que el Rust)."""
    lineas = contenido.splitlines()
    if not lineas or lineas[0].rstrip("\r") != "---":
        return None
    cierre = None
    for i, linea in enumerate(lineas[1:], start=1):
        if linea.rstrip("\r") == "---":
            cierre = i
            break
    if cierre is None:
        return None
    yaml_txt = "\n".join(lineas[1:cierre])
    cuerpo = "\n".join(lineas[cierre + 1 :])
    return yaml_txt, cuerpo


def permalink_de(yaml_txt: str):
    for linea in yaml_txt.splitlines():
        m = PATRON_PERMALINK.match(linea)
        if m:
            return m.group(1).strip("\"'")
    return None


def extrae_aristas(raiz: Path) -> set:
    aristas = set()
    for ruta in walk_md(raiz):
        contenido = ruta.read_text(encoding="utf-8")
        fm = separa_frontmatter(contenido)
        if fm is None:
            continue
        yaml_txt, cuerpo = fm
        permalink = permalink_de(yaml_txt)
        if not permalink:
            continue
        for destino_texto in PATRON_WIKILINK.findall(cuerpo):
            aristas.add((permalink, destino_texto))
    return aristas


def ro(path):
    return sqlite3.connect(f"file:{path}?mode=ro", uri=True)


def diff(engine_db):
    raiz = kb_path()
    ref = extrae_aristas(raiz)
    eng = set(ro(engine_db).execute("SELECT origen, destino_texto FROM aristas").fetchall())
    faltan, sobran = sorted(ref - eng), sorted(eng - ref)
    for o, d in faltan:
        print(f"FALTA en engine: {o} -> {d}")
    for o, d in sobran:
        print(f"SOBRA en engine: {o} -> {d}")
    print(f"ref={len(ref)} engine={len(eng)} faltan={len(faltan)} sobran={len(sobran)}")
    sys.exit(0 if not faltan and not sobran else 1)


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--diff", metavar="ENGINE_DB", required=True)
    a = ap.parse_args()
    diff(a.diff)
